#!/usr/bin/env bash
# Run a seed suite in parallel.
# Usage: harness/suite.sh SUITE [OUTDIR]
#   SUITE = name in harness/suites/, or a path to a suite file
#   Env: BORG_IMPL=c|rust (default c), JOBS (default nproc), ANGBAND_BIN
#        (as run.sh), ALLOW_UNKNOWN_IMPL=1, ALLOW_STALE_BUILD=1 (below)
# Suite file lines: NAME SEED RACE CLASS TURNS [FLAGS]
#   '#' starts a comment (whole line or end of line); FLAGS as in run.sh.
# Before any run it checks the binary (exit 2 if refused):
#   * implementation: read from the executable (harness/identity.sh
#     binary-impl), from the CMakeCache.txt of its build dir and from its
#     BUILD_ID. The known ones must agree and equal BORG_IMPL. If none is
#     known it refuses unless ALLOW_UNKNOWN_IMPL=1.
#   * build identity: BUILD_ID (written by harness/build.sh) must sit in the
#     binary's build dir, its binary_sha256 must equal the binary's, and its
#     source_sha256 must equal this checkout's build inputs (a stale binary
#     otherwise). ALLOW_STALE_BUILD=1 runs anyway.
#   An override that was needed is recorded in the manifest (overrides=),
#   and harness/diff then fails on this output dir.
# The suite runs into a fresh OUTDIR.partial.<pid> and replaces OUTDIR when
# it ends, so OUTDIR never mixes results of two passes. It writes
#   OUTDIR/<NAME>/{keys.log,summary.json,stderr.log,config.txt}
#   OUTDIR/manifest    implementation and build identity of the binary,
#                      checkout revision, suite sha256, overrides, and per
#                      run run_config.<NAME>=<FLAGS> <sha256 of its borg.txt>
#   OUTDIR/timing.txt  one line per run (FAILED/MISSING marked)
#   OUTDIR/DONE        last, only if every run succeeded (harness/diff
#                      refuses an output dir without it)
#   OUTDIR/FAILED      instead of DONE if any run failed, or if the binary
#                      changed while the suite ran
# A failed run does not stop the others. Exit status: 0 if all runs
# succeeded, 1 otherwise, 2 on bad input or a refused binary.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SUITE="$1"
SFILE="$ROOT/harness/suites/$SUITE"
case "$SUITE" in */*) SFILE="$(realpath -m "$SUITE")"; SUITE="$(basename "$SUITE")";; esac
[ -f "$SFILE" ] || { echo "no suite $SFILE" >&2; exit 2; }
IMPL="${BORG_IMPL:-c}"
case "$IMPL" in c|rust) ;; *) echo "BORG_IMPL must be c or rust" >&2; exit 2;; esac
BIN="${ANGBAND_BIN:-$ROOT/build/$IMPL/game/angband}"
[ -x "$BIN" ] || { echo "missing $BIN (run harness/build.sh)" >&2; exit 2; }
RBIN="$(realpath "$BIN")"   # for the build dir; runs use BIN as given
OUT="$(realpath -m "${2:-$ROOT/harness/out/$IMPL/$SUITE}")"
JOBS="${JOBS:-$(nproc)}"
WORK="$OUT.partial.$$"
ID="$ROOT/harness/identity.sh"

# Parse the suite file: strip comments and whitespace, check field counts.
RUNS=()       # "NAME SEED RACE CLASS TURNS FLAGS", single-spaced
NAMES=()
lineno=0
while IFS= read -r line || [ -n "$line" ]; do
    lineno=$((lineno + 1))
    line="${line%%#*}"
    read -r -a f <<< "$line" || true
    [ ${#f[@]} -eq 0 ] && continue
    if [ ${#f[@]} -lt 5 ] || [ ${#f[@]} -gt 6 ]; then
        echo "$SFILE:$lineno: expected NAME SEED RACE CLASS TURNS [FLAGS]" >&2
        exit 2
    fi
    for n in "${NAMES[@]}"; do
        [ "$n" = "${f[0]}" ] && { echo "$SFILE:$lineno: duplicate run ${f[0]}" >&2; exit 2; }
    done
    NAMES+=("${f[0]}")
    RUNS+=("${f[0]} ${f[1]} ${f[2]} ${f[3]} ${f[4]} ${f[5]:--}")
done < "$SFILE"
[ ${#RUNS[@]} -gt 0 ] || { echo "suite $SUITE has no runs" >&2; exit 2; }

# Binary checks (before any run)
refuse() { echo "suite.sh: REFUSED: $*" >&2; exit 2; }
BUILD_DIR="$(dirname "$(dirname "$RBIN")")"
BIN_SHA=$(sha256sum < "$BIN" | cut -d' ' -f1)
IMPL_ELF=$("$ID" binary-impl "$BIN")
CACHE_IMPL=$(sed -n 's/^BORG_IMPL:[A-Z]*=//p' "$BUILD_DIR/CMakeCache.txt" 2>/dev/null || true)
CACHE_IMPL=${CACHE_IMPL:-unknown}
BID="$BUILD_DIR/BUILD_ID"
bid() { [ -f "$BID" ] && sed -n "s/^$1=//p" "$BID" | head -1 || true; }
BID_IMPL=$(bid impl); BID_IMPL=${BID_IMPL:-unknown}
OVERRIDES=()

# 1. implementation
BIN_IMPL=unknown
for v in "$IMPL_ELF" "$CACHE_IMPL" "$BID_IMPL"; do
    [ "$v" = unknown ] && continue
    if [ "$BIN_IMPL" = unknown ]; then BIN_IMPL="$v"
    elif [ "$BIN_IMPL" != "$v" ]; then
        refuse "$BIN: inconsistent implementation (executable: $IMPL_ELF," \
               "CMakeCache: $CACHE_IMPL, BUILD_ID: $BID_IMPL)"
    fi
done
if [ "$BIN_IMPL" = unknown ]; then
    [ "${ALLOW_UNKNOWN_IMPL:-}" = 1 ] || \
        refuse "$BIN: implementation unknown (no rustc/C Borg in the executable," \
               "no CMakeCache.txt or BUILD_ID); set ALLOW_UNKNOWN_IMPL=1 to run it as BORG_IMPL=$IMPL"
    echo "suite.sh: WARNING: implementation of $BIN unknown, ALLOW_UNKNOWN_IMPL=1" >&2
    OVERRIDES+=(ALLOW_UNKNOWN_IMPL)
    IMPL_CHECK="override: implementation unknown, run as $IMPL"
elif [ "$BIN_IMPL" != "$IMPL" ]; then
    refuse "BORG_IMPL=$IMPL but $BIN is a $BIN_IMPL build (executable: $IMPL_ELF," \
           "CMakeCache: $CACHE_IMPL, BUILD_ID: $BID_IMPL)"
else
    IMPL_CHECK=ok
fi

# 2. build identity
STALE=()
SRC_IMPL="$BID_IMPL"; [ "$SRC_IMPL" = unknown ] && SRC_IMPL="$BIN_IMPL"
CUR_SRC=unknown
if [ "$SRC_IMPL" != unknown ]; then CUR_SRC=$("$ID" source-hash "$SRC_IMPL"); fi
if [ ! -f "$BID" ]; then
    STALE+=("no BUILD_ID in $BUILD_DIR (not built by harness/build.sh, or the build failed)")
else
    [ "$(bid binary_sha256)" = "$BIN_SHA" ] || \
        STALE+=("binary sha256 $BIN_SHA differs from BUILD_ID's $(bid binary_sha256) (binary changed after the build)")
    [ "$(bid source_sha256)" = "$CUR_SRC" ] || \
        STALE+=("build inputs changed since the build (BUILD_ID source_sha256 $(bid source_sha256), checkout $CUR_SRC; rebuild with harness/build.sh)")
fi
if [ ${#STALE[@]} -gt 0 ]; then
    if [ "${ALLOW_STALE_BUILD:-}" != 1 ]; then
        printf 'suite.sh: %s\n' "${STALE[@]}" >&2
        refuse "$BIN does not match its build identity; set ALLOW_STALE_BUILD=1 to run it anyway"
    fi
    printf 'suite.sh: WARNING: %s (ALLOW_STALE_BUILD=1)\n' "${STALE[@]}" >&2
    OVERRIDES+=(ALLOW_STALE_BUILD)
    BUILD_CHECK="override: $(IFS='; '; echo "${STALE[*]}")"
else
    BUILD_CHECK=ok
fi

rm -rf "$WORK"
mkdir -p "$WORK"

# Provenance
GIT_REV=$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)
if [ -n "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ]; then DIRTY=yes; else DIRTY=no; fi
{
    echo "suite=$SUITE"
    echo "suite_sha256=$(sha256sum < "$SFILE" | cut -d' ' -f1)"
    echo "borg_txt_sha256=$(sha256sum < "$ROOT/harness/borg.txt" | cut -d' ' -f1)"
    echo "borg_impl_env=$IMPL"
    echo "binary=$BIN"
    echo "binary_sha256=$BIN_SHA"
    echo "binary_impl=$BIN_IMPL"
    echo "binary_elf_impl=$IMPL_ELF"
    echo "binary_cmake_borg_impl=$CACHE_IMPL"
    echo "build_id_impl=$BID_IMPL"
    for k in git_rev git_dirty inputs_dirty source_sha256 binary_sha256 cc rustc built_utc; do
        v=$(bid "$k"); if [ -n "$v" ]; then echo "build_id_$k=$v"; fi
    done
    echo "checkout_source_sha256=$CUR_SRC"
    echo "impl_check=$IMPL_CHECK"
    echo "build_check=$BUILD_CHECK"
    echo "overrides=$(IFS=,; echo "${OVERRIDES[*]:-none}")"
    echo "git_rev=$GIT_REV"
    echo "git_dirty=$DIRTY"
    echo "jobs=$JOBS"
    echo "checkpoint_every=${CHECKPOINT_EVERY:-0}"
    echo "run_timeout_seconds=${RUN_TIMEOUT_SECONDS:-0}"
    echo "save_root=${SAVE_ROOT:-$ROOT/harness/saves}"
    echo "started_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$WORK/manifest"

export BIN
START=$(date +%s.%N)
set +e
printf '%s\n' "${RUNS[@]}" | \
  xargs -P "$JOBS" -n 6 bash -c \
    'if ANGBAND_BIN="$BIN" "$0" "$1/$2" "$3" "$4" "$5" "$6" "$7"; then echo "done $2"; else echo "FAILED $2" >&2; exit 1; fi' \
    "$ROOT/harness/run.sh" "$WORK"
XRC=$?
set -e
END=$(date +%s.%N)

FAIL=0
[ $XRC -eq 0 ] || FAIL=1
for d in "${NAMES[@]}"; do
    [ -s "$WORK/$d/summary.json" ] || { echo "MISSING $d" >&2; FAIL=1; }
done
BIN_SHA_END=$(sha256sum < "$BIN" | cut -d' ' -f1)
if [ "$BIN_SHA_END" != "$BIN_SHA" ]; then
    echo "$BIN changed while the suite ran (sha256 now $BIN_SHA_END)" >&2; FAIL=1
fi
WALL=$(echo "$END - $START" | bc)
if [ $FAIL = 0 ]; then RESULT=OK; else RESULT=FAILED; fi
{
  echo "suite=$SUITE impl=$IMPL jobs=$JOBS wall_seconds=$WALL result=$RESULT"
  for d in "${NAMES[@]}"; do
    python3 -c 'import json,sys; s=json.load(open(sys.argv[1])); print(sys.argv[2], s["stop_reason"], "turns=%d"%s["game_turns"], "pturns=%d"%s["player_turns"], "clev=%d"%s["clevel"], "maxdepth=%d"%s["max_depth"], "keys=%d"%s["keys"], "wall=%.1f"%s["wall_seconds"], s["race"], s["class"])' "$WORK/$d/summary.json" "$d" 2>/dev/null || echo "$d MISSING/FAILED"
  done
} > "$WORK/timing.txt"
# Effective config of each run: its flags and the sha256 of its borg.txt
# (stock suites: borg_txt_sha256 differs from the manifest's stock hash
# only in the respawn race/class lines).
for d in "${NAMES[@]}"; do
    f="$WORK/$d/config.txt"
    if [ -f "$f" ]; then
        echo "run_config.$d=$(sed -n 's/^flags=//p' "$f") $(sed -n 's/^borg_txt_sha256=//p' "$f")"
    else
        echo "run_config.$d=missing"
    fi
done >> "$WORK/manifest"
echo "binary_sha256_end=$BIN_SHA_END" >> "$WORK/manifest"
echo "finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ) result=$RESULT" >> "$WORK/manifest"

# Replace OUTDIR; the marker is written last.
rm -rf "$OUT"
mv "$WORK" "$OUT"
if [ $FAIL = 0 ]; then
    date -u +%Y-%m-%dT%H:%M:%SZ > "$OUT/DONE"
else
    echo "suite $SUITE failed (exit $XRC)" > "$OUT/FAILED"
fi
cat "$OUT/timing.txt"
exit $FAIL

#!/usr/bin/env bash
# Make a regression reference from a suite output (docs/ENGINE.md, References).
#   harness/make-reference.sh SUITE_OUTDIR REFDIR
# Per run: keys.log, messages.log and state.log (xz) and summary.json, the
# files harness/diff compares; and REFDIR/REFERENCE with what made it (the
# build's commit, source and binary hashes and each run's configuration, from
# the suite's manifest, and the profile). The build must come from committed
# inputs, and the suite must have run without overrides.
# REFDIR must not exist. A reference is regenerated only in a PR that says why.
set -euo pipefail
OUT="$1"; REF="$2"
[ -f "$OUT/DONE" ] || { echo "make-reference: $OUT is not a finished suite output" >&2; exit 2; }
[ ! -e "$REF" ] || { echo "make-reference: $REF exists" >&2; exit 2; }
# Everything is checked before anything is written (M2 review, finding 8),
# and the reference is made in a temporary directory, renamed into place
# only when complete: a refused or failed run leaves no REFDIR.
# Provenance comes from the suite output itself, never from today's checkout
# or build (E review, finding 8): the manifest's build identity and each
# run's configuration (its flags and borg.txt, which differ per run in the
# respawn lines). Every run has the same, nonempty profile identity.
m() { sed -n "s/^$1=//p" "$OUT/manifest" | head -1; }
fail() { echo "make-reference: $*" >&2; exit 2; }
[ "$(m overrides)" = none ] || fail "$OUT ran with overrides $(m overrides)"
[ "$(m build_id_inputs_dirty)" = no ] || fail "$OUT was built from uncommitted build inputs; commit, rebuild and rerun"
for k in suite suite_sha256 build_id_git_rev build_id_source_sha256 build_id_engine_upstream binary_sha256; do
    [ -n "$(m "$k")" ] || fail "$OUT/manifest has no $k"
done
runs=0
profiles=""
for run in "$OUT"/*/; do
    [ -f "$run/summary.json" ] || continue
    runs=$((runs + 1))
    for log in keys messages state; do [ -f "$run/$log.log" ] || fail "$run has no $log.log"; done
    prof=$(sed -n 's/^profile=//p' "$run/config.txt" 2>/dev/null | head -1)
    prof_sha=$(sed -n 's/^profile_sha256=//p' "$run/config.txt" 2>/dev/null | head -1)
    [ -n "$prof" ] && [ -n "$prof_sha" ] || fail "$run has no profile identity in config.txt"
    [ "$(grep -c "^run_config\.$(basename "$run")=" "$OUT/manifest")" = 1 ] || fail "$OUT/manifest has no run_config for $(basename "$run")"
    if [ -z "$profiles" ]; then profiles="profile=$prof profile_sha256=$prof_sha"
    elif [ "$profiles" != "profile=$prof profile_sha256=$prof_sha" ]; then
        fail "the runs of $OUT differ in profile ($profiles; $run: $prof $prof_sha)"
    fi
done
[ "$runs" -gt 0 ] || fail "$OUT has no runs"
TMP="$REF.partial.$$"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP"
for run in "$OUT"/*/; do
    name=$(basename "$run")
    [ -f "$run/summary.json" ] || continue
    mkdir "$TMP/$name"
    for log in keys messages state; do xz -9e -c "$run/$log.log" > "$TMP/$name/$log.log.xz"; done
    cp "$run/summary.json" "$TMP/$name/summary.json"
done
{
    echo "suite=$(m suite)"
    echo "suite_sha256=$(m suite_sha256)"
    echo "git_rev=$(m build_id_git_rev)"
    echo "source_sha256=$(m build_id_source_sha256)"
    echo "engine_upstream=$(m build_id_engine_upstream)"
    echo "binary_sha256=$(m binary_sha256)"
    printf '%s\n' $profiles
    grep '^run_config\.' "$OUT/manifest"
    echo "environment=TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8"
} > "$TMP/REFERENCE"
mv "$TMP" "$REF"
trap - EXIT
echo "$REF: $(ls -d "$REF"/*/ | wc -l) runs"

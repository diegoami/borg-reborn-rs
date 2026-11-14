#!/usr/bin/env bash
# One deterministic Borg run.
# Usage: harness/run.sh OUTDIR SEED RACE CLASS TURNS [FLAGS] [-- extra borgh options]
#   FLAGS: comma-separated, "-" for none.
#     exact  (borg-reborn-rs, T1.4) passes -exact-turns: the run stops at
#            engine turn TURNS itself, from borgh's per-turn hook, instead of
#            after the first key at or past it; summary.json then says
#            "exact_turns": true and game_turns equals TURNS for a survivor
#     menu   passes -menu-letters: race/class are picked with the birth
#            menu's own letters, so the indices mean p_race.txt/class.txt
#            order
#     random-birth  passes -random-birth (race and class from the seeded menu)
#     finalsave, living-save, terminal-save  write OUTDIR/final.save at the
#            stop (-save-on-stop, -save-living-on-stop, -save-final-state)
#     keep-load-rng  passes -keep-load-rng (with save=NAME or -load)
#     key-limit=N  stops after N keys (-keys N)
#     save=NAME  loads harness/saves/NAME (or SAVE_ROOT/NAME)
#   The C Borgs' flags of borg-rs's run.sh (cheat, cheatlive, munchkin, lunal,
#   selflunal, NAME=VALUE rewrites of borg.txt) and the dropped frontend
#   options (frames, rune-pattern) are gone: only the Rust Borg is built here
#   (docs/SPEC.md §6), and it does not read borg.txt.
#   Env: BORG_IMPL=rust (the default; build/rust), ANGBAND_BIN (override binary)
#     CHECKPOINT_EVERY=N fixed game-turn capture cadence (0 = off).
#     RUN_TIMEOUT_SECONDS=N optional wall timeout per child (0 = off).
#     SAVE_ROOT=DIR fixture root for save=NAME (default harness/saves).
#     BORG_PROFILE=FILE the reborn Borg's profile (rust arm only; default
#            harness/profiles/reborn-default.txt). Its SHA-256 goes into
#            config.txt; the Borg writes OUTDIR/params.json (the parameter
#            table with the values used) and OUTDIR/trace.jsonl (one line
#            per think), T2.4.
#   process.json records the supervised child result, never a forged summary.
# Produces OUTDIR/keys.log (the oracle), OUTDIR/messages.log (every message
# added to the message log, Borg notes included), OUTDIR/state.log (one
# engine-state line per key), OUTDIR/summary.json, OUTDIR/stderr.log,
# OUTDIR/config.txt (flags and sha256 of the run's user/borg.txt; not part of
# the oracle, harness/diff does not read it)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMPL="${BORG_IMPL:-rust}"
SAVE_ROOT="${SAVE_ROOT:-$ROOT/harness/saves}"
CHECKPOINT_EVERY="${CHECKPOINT_EVERY:-0}"
[[ "$CHECKPOINT_EVERY" =~ ^(0|[1-9][0-9]{0,8}|1000000000)$ ]] || {
    echo "run.sh: CHECKPOINT_EVERY must be 0..1000000000" >&2; exit 2; }
BIN="${ANGBAND_BIN:-$ROOT/build/$IMPL/game/angband}"
[ -x "$BIN" ] || { echo "missing $BIN (run harness/build.sh)" >&2; exit 2; }
OUT="$(realpath -m "$1")"; SEED="$2"; RACE="$3"; CLASS="$4"; TURNS="$5"; shift 5
FLAGS="-"
if [ $# -gt 0 ] && [ "$1" != "--" ]; then FLAGS="$1"; shift; fi
[ "${1:-}" = "--" ] && shift
SAVE_NAME=""
BIRTH_ARGS=(-n)
BORGH_FLAGS=()
if [ "$FLAGS" != "-" ]; then
    IFS=, read -r -a FL <<< "$FLAGS"
    for fl in "${FL[@]}"; do
        case "$fl" in
            menu) BORGH_FLAGS+=(-menu-letters);;
            exact) BORGH_FLAGS+=(-exact-turns);;  # borg-reborn-rs run contract (T1.4)
            random-birth) BORGH_FLAGS+=(-random-birth);;
            finalsave) BORGH_FLAGS+=(-save-on-stop "$OUT/final.save");;
            living-save) BORGH_FLAGS+=(-save-living-on-stop "$OUT/final.save");;
            terminal-save) BORGH_FLAGS+=(-save-final-state "$OUT/final.save");;
            keep-load-rng) BORGH_FLAGS+=(-keep-load-rng);;
            key-limit=*)
                key_limit="${fl#key-limit=}"
                [[ "$key_limit" =~ ^[1-9][0-9]{0,8}$ ]] || { echo 'run.sh: key-limit must be 1..999999999' >&2; exit 2; }
                BORGH_FLAGS+=(-keys "$key_limit");;
            save=*)
                [ -z "$SAVE_NAME" ] || { echo "run.sh: duplicate save flag" >&2; exit 2; }
                SAVE_NAME="${fl#save=}"
                [[ "$SAVE_NAME" =~ ^[a-zA-Z0-9_-]+\.save$ ]] || {
                    echo "run.sh: save must be a filename under SAVE_ROOT" >&2; exit 2; }
                [ -s "$SAVE_ROOT/$SAVE_NAME" ] || {
                    echo "run.sh: missing fixture $SAVE_NAME" >&2; exit 2; }
                ;;
            *) echo "run.sh: unknown flag '$fl'" >&2; exit 2;;
        esac
    done
fi
[ ! -e "$OUT" ] || { echo "run.sh: output exists; preserve it and use a new directory: $OUT" >&2; exit 2; }
# Extra borgh options that change which game starts (re-review R3): a save
# loaded with -load, and -keep-load-rng. config.txt records them for the
# death exporter. An extra -load is handled like save=NAME (re-review S1,
# S2): its file is copied to OUT/save/input.save, which borgh is given by
# absolute path (run.sh changes directory before it starts the game), and
# -n is dropped, since borgh loads only without it.
EXTRA_LOAD=""
EXTRA_KEEP_RNG=0
EXTRA=("$@")
for ((i = 0; i < ${#EXTRA[@]}; i++)); do
    case "${EXTRA[$i]}" in
        -load) EXTRA_LOAD="${EXTRA[$((i + 1))]:-}"
               [ -f "$EXTRA_LOAD" ] || { echo "run.sh: -load: no save file '$EXTRA_LOAD'" >&2; exit 2; }
               [ -z "$SAVE_NAME" ] || { echo "run.sh: both save=$SAVE_NAME and -load given" >&2; exit 2; }
               EXTRA[$((i + 1))]="$OUT/save/input.save"
               i=$((i + 1));;
        -keep-load-rng) EXTRA_KEEP_RNG=1;;
    esac
done
mkdir -p "$OUT"/{user,save,archive,panic,scores}
# The reborn Borg's profile and outputs (T2.4); the C Borgs ignore both.
if [ "$IMPL" = rust ]; then
    PROFILE="$(realpath "${BORG_PROFILE:-$ROOT/harness/profiles/reborn-default.txt}")"
    [ -f "$PROFILE" ] || { echo "run.sh: no profile $PROFILE" >&2; exit 2; }
    # Records name the profile relative to the checkout (no machine paths
    # in dossiers); the Borg reads it by its absolute path.
    PROFILE_NAME="${PROFILE#"$ROOT"/}"
    export BORG_PROFILE="$PROFILE" BORG_PROFILE_NAME="$PROFILE_NAME" BORG_RUN_DIR="$OUT"
fi
if [ "${BORG_TOOLKIT_NATIVE:-0}" = 1 ]; then
    mkdir "$OUT/native"
    export BORG_RECORD_DIR="$OUT/native"
    export BORG_TOOLKIT_HEADLESS=1
fi
if [ -n "$SAVE_NAME" ]; then
    # The engine may autosave: never give it the committed fixture's path.
    cp "$SAVE_ROOT/$SAVE_NAME" "$OUT/save/input.save"
    BIRTH_ARGS=()
    BORGH_FLAGS+=(-load "$OUT/save/input.save")
elif [ -n "$EXTRA_LOAD" ]; then
    cp "$EXTRA_LOAD" "$OUT/save/input.save"
    BIRTH_ARGS=()
fi
if [ "$CHECKPOINT_EVERY" != 0 ]; then
    mkdir -p "$OUT/checkpoints"
    BORGH_FLAGS+=(-checkpoint-every "$CHECKPOINT_EVERY" -checkpoint-dir "$OUT/checkpoints")
fi
# borg-rs's core (BORG_CORE=borg-rs, #207) reads borg.txt: it gets its own,
# borg-rs's harness/borg.txt at the pin (borg-core/borg.txt).
BORG_TXT="$ROOT/harness/borg.txt"
[ "${BORG_CORE:-}" = borg-rs ] && BORG_TXT="$ROOT/borg-core/borg.txt"
# The run's user/borg.txt, as before: the Rust Borg does not read it, but
# the zoo's adapter copies harness/borg.txt (docs/ZOO-INTERFACE.md) and its
# hash is part of a death's encounter family.
sed -e "s/^borg_respawn_race = .*/borg_respawn_race = $RACE/" \
    -e "s/^borg_respawn_class = .*/borg_respawn_class = $CLASS/" \
    -e "s/^borg_cheat_death = .*/borg_cheat_death = FALSE/" \
    "$BORG_TXT" > "$OUT/user/borg.txt"
# borg-rs's core: our overrides of its settings (borg-core/overrides.txt).
# BORG_CORE_OVERRIDES=off keeps borg-rs's own settings: the parity check
# against borg-rs's build at the pin (docs/PILOT.md).
if [ "${BORG_CORE:-}" = borg-rs ] && [ "${BORG_CORE_OVERRIDES:-on}" != off ]; then
    while IFS= read -r line; do
        case "$line" in ''|'#'*) continue;; esac
        name="${line%% =*}"
        grep -q "^$name = " "$OUT/user/borg.txt" || {
            echo "run.sh: override '$name' is not a borg.txt setting" >&2; exit 2; }
        sed -i "s/^$name = .*/$line/" "$OUT/user/borg.txt"
    done < "$ROOT/borg-core/overrides.txt"
fi
{
    echo "flags=$FLAGS"
    echo "borg_txt_sha256=$(sha256sum < "$OUT/user/borg.txt" | cut -d' ' -f1)"
    if [ -n "$SAVE_NAME" ] || [ -n "$EXTRA_LOAD" ]; then
        # The loaded save's hash is part of a death's encounter family.
        echo "save_sha256=$(sha256sum < "$OUT/save/input.save" | cut -d' ' -f1)"
    fi
    if [ "$EXTRA_KEEP_RNG" = 1 ]; then echo "keep_load_rng=1"; fi
    if [ "$IMPL" = rust ]; then
        echo "profile=$PROFILE_NAME"
        echo "profile_sha256=$(sha256sum < "$PROFILE" | cut -d' ' -f1)"
    fi
    if [ "$CHECKPOINT_EVERY" != 0 ]; then echo "checkpoint_every=$CHECKPOINT_EVERY"; fi
} > "$OUT/config.txt"
cd "$(dirname "$BIN")"   # the game finds ./lib relative to the cwd
DATA_ARGS=()
if [ -n "${BORGH_DATA_ROOT:-}" ]; then
    DATA_ARGS=(-dgamedata="$BORGH_DATA_ROOT/gamedata" -dpref="$BORGH_DATA_ROOT/customize" -dhelp="$BORGH_DATA_ROOT/help" -dscreens="$BORGH_DATA_ROOT/screens")
fi
set +e
TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8 python3 "$ROOT/harness/supervise.py" \
    --receipt "$OUT/process.json" --diagnostics "$OUT" --timeout "${RUN_TIMEOUT_SECONDS:-0}" --stall-timeout "${STALL_TIMEOUT_SECONDS:-0}" -- \
    "$BIN" -mborgh "${BIRTH_ARGS[@]}" -uBorg \
    -duser="$OUT/user" -dsave="$OUT/save" -darchive="$OUT/archive" \
    -dpanic="$OUT/panic" -dscores="$OUT/scores" "${DATA_ARGS[@]}" -- \
    -seed "$SEED" -race "$RACE" -class "$CLASS" -turns "$TURNS" \
    -keylog "$OUT/keys.log" -msglog "$OUT/messages.log" -statelog "$OUT/state.log" \
    -summary "$OUT/summary.json" -screen "${BORGH_FLAGS[@]}" "${EXTRA[@]}" \
    2>"$OUT/stderr.log"
RC=$?
set -e
if [ $RC -ne 0 ] || [ ! -s "$OUT/summary.json" ]; then
    echo "run failed (rc=$RC): $OUT" >&2; exit 1
fi
# borg-reborn-rs: the depth stream (T1.4), then the death dossier
# (tools/export-death.py, RULES 12) for every run that died a natural death.
if [ -f "$ROOT/tools/depth-stream.py" ]; then
    python3 "$ROOT/tools/depth-stream.py" "$OUT" --cap "$TURNS"
fi
if [ -f "$ROOT/tools/export-death.py" ]; then
    python3 "$ROOT/tools/export-death.py" "$OUT" --impl "$IMPL" \
        --build-id "$(dirname "$(dirname "$BIN")")/BUILD_ID" --flags "$FLAGS"
fi

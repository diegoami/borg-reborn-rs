#!/usr/bin/env bash
# One deterministic Borg run.
# Usage: harness/run.sh OUTDIR SEED RACE CLASS TURNS [FLAGS] [-- extra borgh options]
#   FLAGS: comma-separated, "-" for none.
#     exact  (borg-reborn-rs, T1.4) passes -exact-turns: the run stops at
#            engine turn TURNS itself, from borgh's per-turn hook, instead of
#            after the first key at or past it; summary.json then says
#            "exact_turns": true and game_turns equals TURNS for a survivor
#     cheat  sets borg_cheat_death = TRUE
#     menu   passes -menu-letters: race/class are picked with the birth
#            menu's own letters, so the indices mean p_race.txt/class.txt
#            order (without it, indices >= 7 pick other entries; see
#            docs/harness.md)
#     cheatlive  passes -cheat-live: the game option cheat_live is on, so with
#            borg_cheat_death off (no cheat flag) the Borg reincarnates
#            at death (borg_respawn_race/class) and the run goes on
#     finalsave writes final.save using the engine at the normal stop boundary.
#   Config flags (each rewrites one line of the run's user/borg.txt; with
#   none of them the file is the same as before, byte for byte):
#     munchkin   sets borg_munchkin_start = TRUE
#     lunal      sets borg_lunal_mode = TRUE
#     selflunal  sets borg_self_lunal = TRUE
#     NAME=VALUE sets any option of borg.txt: the line "NAME = ..." becomes
#                "NAME = VALUE", e.g. borg_money_scum_amount=5000 or
#                borg_uses_swaps=FALSE. NAME must be the option's full
#                name and have exactly one "NAME = " line in
#                harness/borg.txt (else the run fails); VALUE is TRUE,
#                FALSE or an integer. Applied after the lines run.sh sets
#                itself (respawn race/class); borg_cheat_death is set
#                with the cheat flag only.
#   Env: BORG_IMPL=c|rust (selects build/<impl>), ANGBAND_BIN (override binary)
#     CHECKPOINT_EVERY=N fixed game-turn capture cadence (0 = off).
#     RUN_TIMEOUT_SECONDS=N optional wall timeout per child (0 = off).
#     SAVE_ROOT=DIR fixture root for save=NAME (default harness/saves).
#   process.json records the supervised child result, never a forged summary.
# Produces OUTDIR/keys.log (the oracle), OUTDIR/messages.log (every message
# added to the message log, Borg notes included), OUTDIR/state.log (one
# engine-state line per key), OUTDIR/summary.json, OUTDIR/stderr.log,
# OUTDIR/config.txt (flags and sha256 of the run's user/borg.txt; not part of
# the oracle, harness/diff does not read it)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMPL="${BORG_IMPL:-c}"
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
CHEAT=FALSE
SAVE_NAME=""
BIRTH_ARGS=(-n)
BORGH_FLAGS=()
CONFIG_SED=()   # extra borg.txt rewrites from config flags
SET_SED=()      # NAME=VALUE flags, applied last
SET_CHECK=()    # the lines they must produce
if [ "$FLAGS" != "-" ]; then
    IFS=, read -r -a FL <<< "$FLAGS"
    for fl in "${FL[@]}"; do
        case "$fl" in
            cheat) CHEAT=TRUE;;
            menu) BORGH_FLAGS+=(-menu-letters);;
            exact) BORGH_FLAGS+=(-exact-turns);;  # borg-reborn-rs run contract (T1.4)
            random-birth) BORGH_FLAGS+=(-random-birth);;
            finalsave) BORGH_FLAGS+=(-save-on-stop "$OUT/final.save");;
            living-save) BORGH_FLAGS+=(-save-living-on-stop "$OUT/final.save");;
            terminal-save) BORGH_FLAGS+=(-save-final-state "$OUT/final.save");;
            frames) BORGH_FLAGS+=(-frames "$OUT/frames.jsonl");;
            keep-load-rng) BORGH_FLAGS+=(-keep-load-rng);;
            rune-pattern) BORGH_FLAGS+=(-stop-rune-pattern);;
            key-limit=*)
                key_limit="${fl#key-limit=}"
                [[ "$key_limit" =~ ^[1-9][0-9]{0,8}$ ]] || { echo 'run.sh: key-limit must be 1..999999999' >&2; exit 2; }
                BORGH_FLAGS+=(-keys "$key_limit");;
            cheatlive) BORGH_FLAGS+=(-cheat-live);;
            munchkin) CONFIG_SED+=(-e "s/^borg_munchkin_start = .*/borg_munchkin_start = TRUE/");;
            lunal) CONFIG_SED+=(-e "s/^borg_lunal_mode = .*/borg_lunal_mode = TRUE/");;
            selflunal) CONFIG_SED+=(-e "s/^borg_self_lunal = .*/borg_self_lunal = TRUE/");;
            save=*)
                [ -z "$SAVE_NAME" ] || { echo "run.sh: duplicate save flag" >&2; exit 2; }
                SAVE_NAME="${fl#save=}"
                [[ "$SAVE_NAME" =~ ^[a-zA-Z0-9_-]+\.save$ ]] || {
                    echo "run.sh: save must be a filename under SAVE_ROOT" >&2; exit 2; }
                [ -s "$SAVE_ROOT/$SAVE_NAME" ] || {
                    echo "run.sh: missing fixture $SAVE_NAME" >&2; exit 2; }
                ;;
            *=*)
                name="${fl%%=*}"; val="${fl#*=}"
                [[ "$name" =~ ^borg_[a-z_]+$ ]] || {
                    echo "run.sh: flag '$fl': bad option name '$name'" >&2; exit 2; }
                [[ "$val" =~ ^(TRUE|FALSE|-?[0-9]+)$ ]] || {
                    echo "run.sh: flag '$fl': value must be TRUE, FALSE or an integer" >&2; exit 2; }
                [ "$name" != borg_cheat_death ] || {
                    echo "run.sh: flag '$fl': use the cheat flag for borg_cheat_death" >&2; exit 2; }
                n=$(grep -c "^$name = " "$ROOT/harness/borg.txt" || true)
                [ "$n" = 1 ] || {
                    echo "run.sh: flag '$fl': $n lines '$name = ' in harness/borg.txt (unknown option?)" >&2; exit 2; }
                SET_SED+=(-e "s/^$name = .*/$name = $val/")
                SET_CHECK+=("$name = $val");;
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
# Pinned config; respawn race/class follow the run (cheat death stays off)
sed -e "s/^borg_respawn_race = .*/borg_respawn_race = $RACE/" \
    -e "s/^borg_respawn_class = .*/borg_respawn_class = $CLASS/" \
    -e "s/^borg_cheat_death = .*/borg_cheat_death = $CHEAT/" \
    "${CONFIG_SED[@]}" \
    "$ROOT/harness/borg.txt" > "$OUT/user/borg.txt"
if [ ${#SET_SED[@]} -gt 0 ]; then
    sed -i "${SET_SED[@]}" "$OUT/user/borg.txt"
    for line in "${SET_CHECK[@]}"; do
        grep -qx "$line" "$OUT/user/borg.txt" || {
            echo "run.sh: no '$line' line in the run's borg.txt" >&2; exit 2; }
    done
fi
# Each config flag must have changed its line (a renamed option would
# otherwise be silently ignored).
for fl in munchkin:borg_munchkin_start lunal:borg_lunal_mode selflunal:borg_self_lunal; do
    case ",$FLAGS," in
        *",${fl%%:*},"*)
            grep -qx "${fl#*:} = TRUE" "$OUT/user/borg.txt" || {
                echo "run.sh: flag ${fl%%:*}: no '${fl#*:} = TRUE' line in borg.txt" >&2; exit 2; };;
    esac
done
{
    echo "flags=$FLAGS"
    echo "borg_txt_sha256=$(sha256sum < "$OUT/user/borg.txt" | cut -d' ' -f1)"
    if [ -n "$SAVE_NAME" ] || [ -n "$EXTRA_LOAD" ]; then
        # The loaded save's hash is part of a death's encounter family.
        echo "save_sha256=$(sha256sum < "$OUT/save/input.save" | cut -d' ' -f1)"
    fi
    if [ "$EXTRA_KEEP_RNG" = 1 ]; then echo "keep_load_rng=1"; fi
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
# borg-reborn-rs: the depth stream (T1.4), then the death dossier (RULES 12),
# on any arm. The recording launchers (run-borg-headless.py, start-borg.sh,
# produce_death.py) run a frozen copy of this script without tools/; they
# record deaths themselves in borg-rs's full dossier format (tools/README.md).
if [ -f "$ROOT/tools/depth-stream.py" ]; then
    python3 "$ROOT/tools/depth-stream.py" "$OUT" --cap "$TURNS"
fi
if [ -f "$ROOT/tools/export-death.py" ]; then
    python3 "$ROOT/tools/export-death.py" "$OUT" --impl "$IMPL" \
        --build-id "$(dirname "$(dirname "$BIN")")/BUILD_ID" --flags "$FLAGS"
fi

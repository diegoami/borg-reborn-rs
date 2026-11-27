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
#     start-depth=N  synthetic birth at depth N; no ascent/recall, descent allowed
#     start-monsters=SPEC  with start-depth or save=NAME (#722): monsters
#            placed near the player after the level is generated (or the save
#            loaded, in the dungeon), NAME:COUNT[:DIST] joined by
#            '+', '_' for spaces (yellow_worm_mass:6+green_worm_mass:4:8)
#   The C Borgs' flags of borg-rs's run.sh (cheat, cheatlive, munchkin, lunal,
#   selflunal, NAME=VALUE rewrites of borg.txt) and the dropped frontend
#   options (frames, rune-pattern) are gone: only the Rust Borg is built here
#   (docs/SPEC.md §6). Its default core (borg-rs's, #606) reads the run's
#   user/pborg.yaml (borg_* settings, borg_param_*, borg_reborn_*); the reborn
#   core (BORG_CORE=reborn) reads BORG_PROFILE instead.
#   Env: BORG_IMPL=rust (the default; build/rust), ANGBAND_BIN (override binary)
#     CHECKPOINT_EVERY=N fixed game-turn capture cadence (0 = off).
#     RUN_TIMEOUT_SECONDS=N optional wall timeout per child (0 = off).
#     SAVE_ROOT=DIR fixture root for save=NAME (default harness/saves).
#     BORG_SCARY_LIST=FILE the named scary monsters as data (#518; read by
#            the borg-rs core with caution_scary_named_monsters = 1). It wins
#            over the list pborg.yaml names (harness/NAME, shipped:
#            scary_monsters.txt) and over borg_scary_list = none, under its
#            own file name. Either is copied to OUTDIR/user and selected in
#            the run's pborg.yaml with a line naming its SHA-256 (part of
#            pborg_yaml_sha256); config.txt records borg_scary_list and
#            borg_scary_list_sha256. borg_scary_list = none without
#            BORG_SCARY_LIST keeps the compiled C list: no file, no hash.
#     PBORG_SET="name=value;..." rewrites existing keys of the run's
#            pborg.yaml (part of pborg_yaml_sha256): harness/pborg.yaml's copy, or
#            borg-core/pborg.yaml's with BORG_CORE_OVERRIDES=off, which has no
#            borg_param_* or borg_reborn_* keys. Names and values are limited
#            to [A-Za-z0-9_.-]; anything else fails the run.
#     BORG_PROFILE=FILE the reborn Borg's profile (rust arm only; default
#            harness/profiles/reborn-default.txt). Its SHA-256 goes into
#            config.txt; the Borg writes OUTDIR/params.json (the parameter
#            table with the values used) and OUTDIR/trace.jsonl (one line
#            per think), T2.4.
#   process.json records the supervised child result, never a forged summary.
# Produces OUTDIR/keys.log (the oracle), OUTDIR/messages.log (every message
# added to the message log, Borg notes included), OUTDIR/state.log (one
# engine-state line per key), OUTDIR/summary.json, OUTDIR/stderr.log,
# OUTDIR/config.txt (flags and sha256 of the run's pborg.yaml; not part of
# the oracle, harness/diff does not read it)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMPL="${BORG_IMPL:-rust}"
SAVE_ROOT="${SAVE_ROOT:-$ROOT/harness/saves}"
CHECKPOINT_EVERY="${CHECKPOINT_EVERY:-0}"
[[ "$CHECKPOINT_EVERY" =~ ^(0|[1-9][0-9]{0,8}|1000000000)$ ]] || {
    echo "run.sh: CHECKPOINT_EVERY must be 0..1000000000" >&2; exit 2; }
BIN="${ANGBAND_BIN:-$ROOT/build/$IMPL/game/angband}"
# The errands, their ladders and the plans were retired (#1339): a plan
# would change the arm, so asking for one stops the run.
if [ -n "${BORG_ERRANDS_PLAN:-}" ]; then
    echo "run.sh: BORG_ERRANDS_PLAN: the plans were retired (#1339)" >&2; exit 2
fi
# The named scary monsters as data (#518), pinned by its SHA-256.
SCARY=""
if [ -n "${BORG_SCARY_LIST+set}" ]; then
    [ -n "$BORG_SCARY_LIST" ] && [ -f "$BORG_SCARY_LIST" ] || {
        echo "run.sh: BORG_SCARY_LIST=$BORG_SCARY_LIST: no such file" >&2; exit 2; }
    SCARY="$(realpath "$BORG_SCARY_LIST")"
    unset BORG_SCARY_LIST
fi
[ -x "$BIN" ] || { echo "missing $BIN (run harness/build.sh)" >&2; exit 2; }
OUT="$(realpath -m "$1")"; SEED="$2"; RACE="$3"; CLASS="$4"; TURNS="$5"; shift 5
FLAGS="-"
if [ $# -gt 0 ] && [ "$1" != "--" ]; then FLAGS="$1"; shift; fi
[ "${1:-}" = "--" ] && shift
SAVE_NAME=""
BIRTH_ARGS=(-n)
BORGH_FLAGS=()
START_DEPTH=""
START_MONSTERS=""
START_BUILD=""
START_BUILD_SHA=""
set_start_depth() {
    [ -z "$START_DEPTH" ] || { echo 'run.sh: duplicate start-depth' >&2; exit 2; }
    [[ "$1" =~ ^[1-9][0-9]{0,2}$ ]] || { echo 'run.sh: start-depth must be 1..999 (and below engine max_depth)' >&2; exit 2; }
    START_DEPTH="$1"
}
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
            start-depth=*)
                set_start_depth "${fl#start-depth=}"
                BORGH_FLAGS+=(-start-depth "$START_DEPTH");;
            start-monsters=*)
                [ -z "$START_MONSTERS" ] || { echo 'run.sh: duplicate start-monsters' >&2; exit 2; }
                START_MONSTERS="${fl#start-monsters=}"
                [[ "$START_MONSTERS" =~ ^[A-Za-z_\'-]+:[0-9]{1,3}(:[0-9]{1,2})?(\+[A-Za-z_\'-]+:[0-9]{1,3}(:[0-9]{1,2})?)*$ ]] || {
                    echo 'run.sh: start-monsters must be NAME:COUNT[:DIST] joined by +' >&2; exit 2; }
                # Review of #741: the frontend's bounds, COUNT 1..100, DIST 1..40.
                IFS=+ read -r -a SM_ENTRIES <<< "$START_MONSTERS"
                for sm in "${SM_ENTRIES[@]}"; do
                    IFS=: read -r _ sm_count sm_dist <<< "$sm"
                    { [ "$sm_count" -ge 1 ] && [ "$sm_count" -le 100 ] && { [ -z "$sm_dist" ] || { [ "$sm_dist" -ge 1 ] && [ "$sm_dist" -le 40 ]; }; }; } || {
                        echo 'run.sh: start-monsters needs COUNT 1..100 and DIST 1..40' >&2; exit 2; }
                done
                BORGH_FLAGS+=(-start-monsters "$START_MONSTERS");;
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
        -start-depth)
            set_start_depth "${EXTRA[$((i + 1))]:-}"
            if [ "$FLAGS" = '-' ]; then FLAGS="start-depth=$START_DEPTH"
            else FLAGS="$FLAGS,start-depth=$START_DEPTH"; fi
            i=$((i + 1));;
        -start-build)
            [ -z "$START_BUILD" ] || { echo 'run.sh: duplicate start-build' >&2; exit 2; }
            START_BUILD="${EXTRA[$((i + 1))]:-}"
            [ -f "$START_BUILD" ] && [ ! -L "$START_BUILD" ] || { echo 'run.sh: start-build requires a regular record file' >&2; exit 2; }
            [ "$(wc -c < "$START_BUILD")" -le 16384 ] || { echo 'run.sh: start-build record too large' >&2; exit 2; }
            EXTRA[$((i + 1))]="$OUT/start.build"
            i=$((i + 1));;
    esac
done
[ -z "$START_BUILD" ] || [ -n "$START_DEPTH" ] || { echo 'run.sh: start-build requires start-depth' >&2; exit 2; }
[ -z "$START_MONSTERS" ] || [ -n "$START_DEPTH" ] || [ -n "$SAVE_NAME" ] || [ -n "${EXTRA_LOAD:-}" ] || { echo 'run.sh: start-monsters requires start-depth, save=NAME or -load' >&2; exit 2; }
if [ -n "$START_DEPTH" ]; then
    [ -z "$SAVE_NAME$EXTRA_LOAD" ] || { echo 'run.sh: start-depth cannot load a save' >&2; exit 2; }
    for option in "${BORGH_FLAGS[@]}" "${EXTRA[@]}"; do
        [ "$option" != '-random-birth' ] || { echo 'run.sh: start-depth cannot use random birth' >&2; exit 2; }
    done
fi
mkdir -p "$OUT"/{user,save,archive,panic,scores}
if [ -n "$START_BUILD" ]; then
    cp "$START_BUILD" "$OUT/start.build"
    chmod 400 "$OUT/start.build"
    START_BUILD_SHA=$(sha256sum < "$OUT/start.build" | cut -d' ' -f1)
fi
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
# The parity run uses the donor's YAML defaults.
PBORG_TEMPLATE="$ROOT/harness/pborg.yaml"
if [ "${BORG_CORE_OVERRIDES:-on}" = off ] && [ -z "${BORG_CORE+set}" ]; then
    export BORG_CORE=borg-rs
fi
if [ "${BORG_CORE:-borg-rs}" = borg-rs ] && [ "${BORG_CORE_OVERRIDES:-on}" = off ]; then
    PBORG_TEMPLATE="$ROOT/borg-core/pborg.yaml"
fi
SETTINGS="$OUT/user/pborg.yaml"
edit_settings() { python3 "$ROOT/harness/pborg-settings.py" "$@"; }
get_setting() { edit_settings "$SETTINGS" get "$1"; }
update_settings() {
    edit_settings "$SETTINGS" edit "$@" > "$OUT/.pborg.yaml" && mv "$OUT/.pborg.yaml" "$SETTINGS"
}
BIRTH_SETTINGS=(--set "borg_respawn_race=$RACE" --set "borg_respawn_class=$CLASS"
                --set borg_cheat_death=FALSE)
if [ "${BORG_CORE:-}" = reborn ]; then
    BIRTH_SETTINGS+=(--set borg_reborn_core=reborn)
fi
edit_settings "$PBORG_TEMPLATE" edit "${BIRTH_SETTINGS[@]}" > "$SETTINGS"

# BORG_TXT_SET is the historical name for the same override interface.
OVERRIDES="${PBORG_SET-${BORG_TXT_SET:-}}"
if [ -n "$OVERRIDES" ]; then
    IFS=';' read -ra SETS <<< "$OVERRIDES"
    CHANGES=()
    for kv in "${SETS[@]}"; do
        [[ "$kv" =~ ^[A-Za-z0-9_@]+=[A-Za-z0-9_.-]+$ ]] || {
            echo "run.sh: PBORG_SET: invalid entry '$kv'" >&2; exit 2; }
        CHANGES+=(--set "$kv")
    done
    update_settings --existing "${CHANGES[@]}"
fi
SELECTED_PLAN="$(get_setting borg_plan)"
if [ -n "$SELECTED_PLAN" ]; then
    echo "run.sh: $PBORG_TEMPLATE selects a plan; the plans were retired (#1339)" >&2
    exit 2
fi
# The selected scary list travels with the run. Its SHA-256 is in the YAML
# receipt and config.txt so changing the list changes the encounter identity.
SCARY_NAME="$(get_setting borg_scary_list)"
SCARY_SRC=""; SCARY_SHA=""
if [ -n "$SCARY" ]; then
    SCARY_SRC="$SCARY"; SCARY_NAME="$(basename "$SCARY")"
elif [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    SCARY_SRC="$ROOT/harness/$SCARY_NAME"
fi
if [ -n "$SCARY_SRC" ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ && "$SCARY_NAME" != none &&
       "$SCARY_NAME" != pborg.yaml && "$SCARY_NAME" != borg.txt &&
       "$SCARY_NAME" != errands_plan.txt ]] || {
        echo "run.sh: scary list name '$SCARY_NAME': a plain file name is expected" >&2; exit 2; }
    [ -f "$SCARY_SRC" ] || { echo "run.sh: scary list $SCARY_SRC: no such file" >&2; exit 2; }
    cp "$SCARY_SRC" "$OUT/user/$SCARY_NAME"
    SCARY_SHA="$(sha256sum < "$SCARY_SRC" | cut -d' ' -f1)"
    update_settings --set "borg_scary_list=$SCARY_NAME"
    printf '\n# harness/run.sh (#518): scary list sha256 %s\n' "$SCARY_SHA" >> "$SETTINGS"
fi
# World variants are consumed by the runner. Their names and data hashes
# are recorded without passing the adapter-only key to the Borg.
WORLD_NAME="$(get_setting borg_world)"
WORLD_SRC=""; WORLD_SHA=""; WORLD_DATA_SHA=""
if [ -n "${BORG_WORLD:-}" ]; then
    WORLD_SRC="$BORG_WORLD"; WORLD_NAME="$(basename "$BORG_WORLD" .txt)"
    [[ "$WORLD_NAME" =~ ^[A-Za-z0-9_.-]+$ ]] || {
        echo "run.sh: BORG_WORLD file name '$WORLD_NAME': a plain name is expected" >&2; exit 2; }
elif [ -n "$WORLD_NAME" ] && [ "$WORLD_NAME" != none ]; then
    [[ "$WORLD_NAME" =~ ^[A-Za-z0-9_.-]+$ ]] || {
        echo "run.sh: world name '$WORLD_NAME': a plain name is expected" >&2; exit 2; }
    WORLD_SRC="$ROOT/harness/worlds/$WORLD_NAME.txt"
fi
update_settings --remove borg_world
if [ -n "$WORLD_SRC" ]; then
    [ -f "$WORLD_SRC" ] || { echo "run.sh: world $WORLD_SRC: no such file" >&2; exit 2; }
    [ -z "${BORGH_DATA_ROOT:-}" ] || { echo "run.sh: a world and BORGH_DATA_ROOT together" >&2; exit 2; }
    WORLD_SHA="$(sha256sum < "$WORLD_SRC" | cut -d' ' -f1)"
    WORLD_DATA_SHA="$(python3 "$ROOT/scripts/make-world.py" --config "$WORLD_SRC" --lib "$(dirname "$BIN")/lib" --out "$OUT/world" 2>"$OUT/world.log")" || {
        echo "run.sh: world $WORLD_NAME: $(tail -1 "$OUT/world.log")" >&2; exit 2; }
    BORGH_DATA_ROOT="$OUT/world"
    printf '\n# harness/run.sh (#704): world %s, config sha256 %s, data sha256 %s\n' \
        "$WORLD_NAME" "$WORLD_SHA" "$WORLD_DATA_SHA" >> "$SETTINGS"
fi
{
    echo "flags=$FLAGS"
    [ -z "$START_MONSTERS" ] || echo "start_monsters=$START_MONSTERS"
    if [ -n "$START_DEPTH" ]; then
        echo 'synthetic=1'
        echo "start_depth=$START_DEPTH"
        if [ -n "$START_BUILD_SHA" ]; then
            echo 'start_build=record'
            echo "start_build_sha256=$START_BUILD_SHA"
        else echo 'start_build=birth'; fi
        echo 'dungeon_mode=one_way'
        echo 'synthetic_contract=borg-reborn-rs/synthetic-start@2'
    fi
    echo "pborg_yaml_sha256=$(sha256sum < "$SETTINGS" | cut -d' ' -f1)"
    [ -z "$OVERRIDES" ] || echo "pborg_set=$OVERRIDES"
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
    # The core the environment selects: it changes play, so the record says
    # it, and tools/rerun-death.py sets it again (review #227, finding 8).
    # Only those set; a run without any keeps the record it always had. (The
    # errand variables went with the errands, #1339.)
    for v in BORG_CORE BORG_CORE_OVERRIDES; do
        if [ -n "${!v+set}" ]; then echo "$(echo "$v" | tr '[:upper:]' '[:lower:]')=${!v}"; fi
    done
    if [ -z "${BORG_CORE+set}" ]; then
        core="$(get_setting borg_reborn_core)"
        echo "borg_core=${core:-reborn}"
    fi
    # The scary list (#518): its name, and hash when one was copied.
    echo "borg_scary_list=${SCARY_NAME:-none}"
    if [ -n "$SCARY_SHA" ]; then echo "borg_scary_list_sha256=$SCARY_SHA"; fi
    # The world (#704): only when one is played, so a run without one keeps
    # the record it always had.
    if [ -n "$WORLD_SHA" ]; then
        echo "borg_world=$WORLD_NAME"
        echo "borg_world_sha256=$WORLD_SHA"
        echo "borg_world_data_sha256=$WORLD_DATA_SHA"
    fi
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
# Check the engine's receipt before accepting or exporting the run. A binary
# without this capability, or an actual town start, is a harness failure.
python3 - "$OUT/summary.json" "$START_DEPTH" "$START_BUILD_SHA" <<'PYSYNTHETIC'
import json, sys
summary = json.load(open(sys.argv[1]))
requested = sys.argv[2]
build_hash = sys.argv[3]
if requested:
    valid = (summary.get('synthetic') is True and summary.get('loaded') is False
             and summary.get('synthetic_contract') == 'borg-reborn-rs/synthetic-start@2'
             and summary.get('start_depth') == int(requested)
             and summary.get('initial_depth') == int(requested)
             and summary.get('initial_cave_depth') == int(requested)
             and summary.get('initial_generation_pending') is False
             and summary.get('start_build') == ('record' if build_hash else 'birth')
             and (summary.get('start_build_sha256') == build_hash if build_hash else not summary.get('start_build_sha256'))
             and summary.get('dungeon_mode') == 'one_way'
             and summary.get('game_options', {}).get('birth_force_descend') is True
             and summary.get('game_options', {}).get('birth_no_recall') is True)
    if not valid:
        raise SystemExit('run.sh: synthetic start identity mismatch')
elif summary.get('synthetic') or summary.get('start_depth'):
    raise SystemExit('run.sh: unexpected synthetic start')
PYSYNTHETIC
if [ -n "$START_BUILD_SHA" ]; then
    [ "$(sha256sum < "$OUT/start.build" | cut -d' ' -f1)" = "$START_BUILD_SHA" ] || { echo 'run.sh: start-build changed during execution' >&2; exit 1; }
fi
# Review of #741: a start-monsters run that placed fewer monsters than it
# asked for is measuring another colony than its label says; say so.
if [ -n "$START_MONSTERS" ]; then
    python3 - "$OUT/summary.json" <<'PYSHORT' >&2
import json, sys
s = json.load(open(sys.argv[1]))
req, got = s.get("start_monsters_requested", 0), s.get("start_monsters_placed", 0)
if got < req:
    print(f"run.sh: warning: start-monsters placed {got} of {req} ({s.get('start_monsters')})")
PYSHORT
fi
# borg-reborn-rs (#219): environment overrides are part of the run identity.
if [ -f "$OUT/params-core.json" ]; then
    echo "params_core_sha256=$(sha256sum < "$OUT/params-core.json" | cut -d' ' -f1)" >> "$OUT/config.txt"
fi
# borg-reborn-rs: the depth stream (T1.4), then the death dossier
# (tools/export-death.py, RULES 12) for every run that died a natural death.
if [ -f "$ROOT/tools/depth-stream.py" ]; then
    python3 "$ROOT/tools/depth-stream.py" "$OUT" --cap "$TURNS"
fi
if [ -z "$START_DEPTH" ] && [ -f "$ROOT/tools/export-death.py" ]; then
    python3 "$ROOT/tools/export-death.py" "$OUT" --impl "$IMPL" \
        --build-id "$(dirname "$(dirname "$BIN")")/BUILD_ID" --flags "$FLAGS"
fi

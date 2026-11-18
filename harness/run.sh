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
#   The C Borgs' flags of borg-rs's run.sh (cheat, cheatlive, munchkin, lunal,
#   selflunal, NAME=VALUE rewrites of borg.txt) and the dropped frontend
#   options (frames, rune-pattern) are gone: only the Rust Borg is built here
#   (docs/SPEC.md §6), and it does not read borg.txt.
#   Env: BORG_IMPL=rust (the default; build/rust), ANGBAND_BIN (override binary)
#     CHECKPOINT_EVERY=N fixed game-turn capture cadence (0 = off).
#     RUN_TIMEOUT_SECONDS=N optional wall timeout per child (0 = off).
#     SAVE_ROOT=DIR fixture root for save=NAME (default harness/saves).
#     BORG_ERRANDS_PLAN=FILE an errands plan (docs/GAME-PLAN-LANGUAGE.md;
#            needs the borg-rs core, the default). It is copied to OUTDIR/user and
#            selected in the run's borg.txt with a line naming its SHA-256,
#            so the plan's bytes are part of borg_txt_sha256 (#352).
#     BORG_SCARY_LIST=FILE the named scary monsters as data (#518; read by
#            the borg-rs core with caution_scary_named_monsters = 1). It wins
#            over the list the borg.txt names (harness/NAME, shipped:
#            scary_monsters.txt) and over borg_scary_list = none, under its
#            own file name. Either is copied to OUTDIR/user and selected in
#            the run's borg.txt with a line naming its SHA-256 (part of
#            borg_txt_sha256); config.txt records borg_scary_list and
#            borg_scary_list_sha256. borg_scary_list = none without
#            BORG_SCARY_LIST keeps the compiled C list: no file, no hash.
#     BORG_TXT_SET="name=value;..." rewrites existing keys of the run's
#            borg.txt (part of borg_txt_sha256): harness/borg.txt's copy, or
#            borg-core/borg.txt's with BORG_CORE_OVERRIDES=off, which has no
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
# The game reads BORG_LADDERS from its own directory: a relative path is
# made absolute here, so the file hashed below is the file it reads
# (re-review #227, 7). A missing file is refused rather than silently
# replaced by the built-in ladders.
if [ -n "${BORG_LADDERS:-}" ]; then
    [ -f "$BORG_LADDERS" ] || { echo "run.sh: BORG_LADDERS=$BORG_LADDERS: no such file" >&2; exit 2; }
    BORG_LADDERS="$(realpath "$BORG_LADDERS")"; export BORG_LADDERS
fi
# An errands plan (#305, #352), resolved here like BORG_LADDERS. The game
# reads the run's own copy through borg.txt, never this path.
PLAN=""
if [ -n "${BORG_ERRANDS_PLAN+set}" ]; then
    [ -n "$BORG_ERRANDS_PLAN" ] && [ -f "$BORG_ERRANDS_PLAN" ] || {
        echo "run.sh: BORG_ERRANDS_PLAN=$BORG_ERRANDS_PLAN: no such file" >&2; exit 2; }
    [ "${BORG_CORE:-borg-rs}" = borg-rs ] || {
        echo "run.sh: BORG_ERRANDS_PLAN needs the borg-rs core" >&2; exit 2; }
    PLAN="$(realpath "$BORG_ERRANDS_PLAN")"
    unset BORG_ERRANDS_PLAN
fi
# The named scary monsters as data (#518), pinned like the plan.
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
# harness/borg.txt is borg-rs's borg.txt (its core reads it) with our
# settings: no stair-scumming and no town scumming (owner, 2026-11-14), and
# the keys borg_reborn_core / borg_reborn_errand (#207, #208).
# BORG_CORE_OVERRIDES=off gives borg-rs's core its pristine copy instead
# (borg-core/borg.txt): the parity check against borg-rs's build at the pin.
BORG_TXT="$ROOT/harness/borg.txt"
[ "${BORG_CORE:-borg-rs}" = borg-rs ] && [ "${BORG_CORE_OVERRIDES:-on}" = off ] && BORG_TXT="$ROOT/borg-core/borg.txt"
# The run's user/borg.txt, as before: the Rust Borg does not read it, but
# the zoo's adapter copies harness/borg.txt (docs/ZOO-INTERFACE.md) and its
# hash is part of a death's encounter family.
sed -e "s/^borg_respawn_race = .*/borg_respawn_race = $RACE/" \
    -e "s/^borg_respawn_class = .*/borg_respawn_class = $CLASS/" \
    -e "s/^borg_cheat_death = .*/borg_cheat_death = FALSE/" \
    "$BORG_TXT" > "$OUT/user/borg.txt"
# #606 (owner): a run with no BORG_CORE plays the core harness/borg.txt
# names (borg_reborn_core = borg-rs), the one the zoo runs and the one that
# reads this file. BORG_CORE=reborn selects the reborn core (it reads the
# profile above, not this file).
# BORG_TXT_SET="name=value;name=value": rewrite existing keys of the run's
# borg.txt (a test pinning a setting whose shipped default moved, #479).
if [ -n "${BORG_TXT_SET:-}" ]; then
    IFS=';' read -ra SETS <<< "$BORG_TXT_SET"
    for kv in "${SETS[@]}"; do
        name="${kv%%=*}"; value="${kv#*=}"
        [[ "$kv" == *=* && "$name" =~ ^[A-Za-z0-9_]+$ && "$value" =~ ^[A-Za-z0-9_.-]+$ ]] || {
            echo "run.sh: BORG_TXT_SET: invalid entry '$kv' (name=value, [A-Za-z0-9_.-])" >&2; exit 2; }
        grep -q "^$name = " "$OUT/user/borg.txt" || { echo "run.sh: BORG_TXT_SET: no key $name in borg.txt" >&2; exit 2; }
        sed -i "s/^$name = .*/$name = $value/" "$OUT/user/borg.txt"
    done
fi
# A plan selected in the borg.txt itself would escape the identity below.
if grep -Eq '^[[:space:]]*borg_plan[[:space:]]*=' "$OUT/user/borg.txt"; then
    echo "run.sh: $BORG_TXT selects a plan; harness runs select plans with BORG_ERRANDS_PLAN" >&2
    exit 2
fi
# The scary list (#518, review of #535). Precedence:
#  1. BORG_SCARY_LIST=FILE: that file, under its own file name;
#  2. otherwise the list the borg.txt names, copied from harness/NAME;
#  3. borg_scary_list = none (or no key): the compiled C list, no file, no hash.
# A copied list goes next to the run's borg.txt; its one selecting line is
# rewritten with the list's SHA-256, so its bytes are part of
# borg_txt_sha256, and config.txt records its name and hash.
SCARY_NAME="$(sed -n 's/^[[:space:]]*borg_scary_list[[:space:]]*=[[:space:]]*//p' "$OUT/user/borg.txt" | tail -1 | tr -d '[:space:]')"
SCARY_SRC=""; SCARY_SHA=""
if [ -n "$SCARY" ]; then
    SCARY_SRC="$SCARY"; SCARY_NAME="$(basename "$SCARY")"
elif [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    SCARY_SRC="$ROOT/harness/$SCARY_NAME"
fi
if [ -n "$SCARY_SRC" ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ && "$SCARY_NAME" != none ]] || {
        echo "run.sh: scary list name '$SCARY_NAME': a plain file name is expected" >&2; exit 2; }
    [ -f "$SCARY_SRC" ] || { echo "run.sh: scary list $SCARY_SRC: no such file" >&2; exit 2; }
    cp "$SCARY_SRC" "$OUT/user/$SCARY_NAME"
    SCARY_SHA="$(sha256sum < "$SCARY_SRC" | cut -d' ' -f1)"
    sed -i '/^[[:space:]]*borg_scary_list[[:space:]]*=/d' "$OUT/user/borg.txt"
    printf '\n# harness/run.sh (#518): scary list sha256 %s\nborg_scary_list = %s\n' \
        "$SCARY_SHA" "$SCARY_NAME" >> "$OUT/user/borg.txt"
fi
if [ -n "$PLAN" ]; then
    # The SHA-256 line puts the plan's bytes into borg_txt_sha256, which the
    # cemetery's encounter family already hashes. It names no path, so the
    # same plan under another name is the same starting game.
    cp "$PLAN" "$OUT/user/errands_plan.txt"
    PLAN_SHA="$(sha256sum < "$PLAN" | cut -d' ' -f1)"
    PLAN_NAME="${PLAN#"$ROOT"/}"
    printf '\n# harness/run.sh (#352): errands plan sha256 %s\nborg_plan = errands_plan.txt\n' \
        "$PLAN_SHA" >> "$OUT/user/borg.txt"
fi
{
    echo "flags=$FLAGS"
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
    # The settings the environment gives the Borg (docs/ERRANDS.md): they
    # change play, so the record says them, and tools/rerun-death.py sets
    # them again (review #227, finding 8). Only those set, empty ones too
    # (an empty BORG_ERRAND turns off the borg.txt fallback; re-review 6):
    # a run without any keeps the record it always had.
    for v in BORG_CORE BORG_CORE_OVERRIDES BORG_ERRAND BORG_STOPSCARED BORG_ERRAND_BUDGET BORG_LADDERS; do
        if [ -n "${!v+set}" ]; then echo "$(echo "$v" | tr '[:upper:]' '[:lower:]')=${!v}"; fi
    done
    if [ -n "${BORG_LADDERS:-}" ]; then echo "borg_ladders_sha256=$(sha256sum < "$BORG_LADDERS" | cut -d' ' -f1)"; fi
    # The plan named relative to the checkout (no machine paths in dossiers).
    if [ -n "$PLAN" ]; then
        echo "borg_errands_plan=$PLAN_NAME"
        echo "borg_errands_plan_sha256=$PLAN_SHA"
    fi
    # The scary list (#518): its name, and hash when one was copied.
    echo "borg_scary_list=${SCARY_NAME:-none}"
    if [ -n "$SCARY_SHA" ]; then echo "borg_scary_list_sha256=$SCARY_SHA"; fi
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
# The game preserves the plan it parsed as BORG_RUN_DIR/errands_plan.txt.
if [ -n "$PLAN" ] && ! cmp -s "$OUT/user/errands_plan.txt" "$OUT/errands_plan.txt"; then
    echo "run.sh: the game did not load the plan: $OUT" >&2; exit 1
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

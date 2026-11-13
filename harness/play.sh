#!/usr/bin/env bash
# A town session (#191): the Borg plays the dungeon, the player plays the town.
#
# Usage: harness/play.sh OUTDIR [CHARACTERS]
#
#   OUTDIR      where the session is recorded (it must not exist)
#   CHARACTERS  how many characters to play, one after the other (default 1)
#
# The handover (BORG_TOWN_PLAYER=1, borg-reborn-rs/src/town.rs): while the
# character is in town the Borg's input hook yields and the engine reads the
# keyboard, so the player has the controls in town and the Borg has them in
# the dungeon. The Borg decides nothing in town: no shop, no equipment, no
# prompt answered. The player shops and leaves (stairs or recall); the Borg
# plays the dungeon and comes back when it wants to.
#
# In the game: create or load the character, then Ctrl-Z and z start the
# Borg. In the dungeon any key stops it (Ctrl-Z, z starts it again).
#
# What is recorded, per character, in OUTDIR/char-N:
#
#   state.jsonl  the town states, one per distinct state, in borg-rs's
#                state record shape (schema 3, docs/TOWN-PLAN.md)
#   objects.log  the objects the run touched, numbered: what the Borg used
#                and bought, and what appeared in the pack (the player's
#                purchases are `acquire` lines). Also printed on the console.
#   trace.jsonl, params.json
#                the Borg's decisions in the dungeon (BORG_RUN_DIR)
#   feedback.md, corrections.jsonl
#                the owner's feedback, asked when the character ends
#                (harness/feedback.sh)
#
# A town session is not a measured run (RULES 12): the handover changes play.
#
# The front end is X11 (watch-borg.sh's build, build/watch/), so the game
# needs a display. BORG_PROFILE is passed through to the Borg as usual.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ $# -ge 1 ] && [ $# -le 2 ] || { echo "usage: harness/play.sh OUTDIR [CHARACTERS]" >&2; exit 2; }
OUT="$(realpath -m "$1")"
CHARACTERS="${2:-1}"
[[ "$CHARACTERS" =~ ^[1-9][0-9]*$ ]] || { echo "play.sh: CHARACTERS must be a number" >&2; exit 2; }
[ ! -e "$OUT" ] || { echo "play.sh: output exists; use a new directory: $OUT" >&2; exit 2; }

"$ROOT/watch-borg.sh" --build-only
BIN="$ROOT/build/watch/game/angband"
[ -x "$BIN" ] || { echo "play.sh: missing $BIN" >&2; exit 2; }
mkdir -p "$OUT"

cd "$(dirname "$BIN")"   # the game finds ./lib relative to the cwd
for ((c = 1; c <= CHARACTERS; c++)); do
    DIR="$OUT/char-$c"
    mkdir -p "$DIR"/{user,save,archive,panic,scores}
    {
        echo "character=$c"
        echo "commit=$(git -C "$ROOT" rev-parse HEAD)"
        echo "profile=${BORG_PROFILE:-defaults}"
    } > "$DIR/config.txt"
    echo "play.sh: character $c of $CHARACTERS - Ctrl-Z then z starts the Borg."
    echo "play.sh: you have the controls in town; the Borg plays the dungeon."
    BORG_TOWN_PLAYER=1 \
    BORG_TOWN_STATE="$DIR/state.jsonl" \
    BORG_OBJECT_LOG="$DIR/objects.log" \
    BORG_RUN_DIR="$DIR" \
        ./angband -mx11 -uBorg \
        -duser="$DIR/user" -dsave="$DIR/save" -darchive="$DIR/archive" \
        -dpanic="$DIR/panic" -dscores="$DIR/scores" \
        || echo "play.sh: character $c ended with rc=$?"
    bash "$ROOT/harness/feedback.sh" "$DIR"
done
echo "play.sh: done - $OUT"

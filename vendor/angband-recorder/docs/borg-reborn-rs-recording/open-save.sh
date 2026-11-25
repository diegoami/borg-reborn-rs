#!/usr/bin/env bash
# Open a copy of a save in an X11 window:  open-save.sh SAVEFILE [NAME]
#   SAVEFILE  e.g. ~/angband-recordings/1/final.save or .../checkpoints/checkpoint-000010-t0000500003.save
#   NAME      copy name (default: derived from the file); the copy lives in ~/angband-play/NAME and is
#             reused next time, so you can keep playing it.
# Uses the X11 build of the main borg-reborn-rs checkout (its ./watch-borg.sh --build-only).
# The original file is never touched. Ctrl-Z then z starts the Borg; any key stops it.
# For the Borg to use the parameters: source run-env.sh first.
set -euo pipefail
: "${BORG_REBORN_RS:=$HOME/projects/angband/borg-reborn-rs}"
SRC="$(realpath "$1")"
NAME="${2:-$(basename "$(dirname "$(dirname "$SRC")")")-$(basename "$SRC" .save)}"
NAME="${NAME//[^A-Za-z0-9_-]/_}"
BIN_DIR="$BORG_REBORN_RS/build/watch/game"
[ -x "$BIN_DIR/angband" ] || { echo "open-save.sh: no X11 build; run $BORG_REBORN_RS/watch-borg.sh --build-only" >&2; exit 2; }
DIR="$HOME/angband-play/$NAME"
if [ ! -e "$DIR/save/$NAME" ]; then
    mkdir -p "$DIR"/{user,save,archive,panic,scores}
    cp "$SRC" "$DIR/save/$NAME"
    RUN="$(dirname "$SRC")"; [ -f "$RUN/user/borg.txt" ] || RUN="$(dirname "$RUN")"
    if [ -f "$RUN/user/borg.txt" ]; then cp "$RUN/user/borg.txt" "$DIR/user/"; fi
    if [ -f "$RUN/user/errands_plan.txt" ]; then cp "$RUN/user/errands_plan.txt" "$DIR/user/"; fi
    echo "open-save.sh: copied $SRC -> $DIR/save/$NAME"
else
    echo "open-save.sh: continuing the existing copy $DIR/save/$NAME"
fi
unset BORG_ERRANDS_PLAN BORG_TOOLKIT_NATIVE BORG_RECORD_DIR ANGBAND_BIN CHECKPOINT_EVERY
cd "$BIN_DIR"
exec ./angband -mx11 -duser="$DIR/user" -dsave="$DIR/save" -darchive="$DIR/archive" \
    -dpanic="$DIR/panic" -dscores="$DIR/scores" -u"$NAME"

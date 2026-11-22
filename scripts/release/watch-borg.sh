#!/usr/bin/env bash
# Watch the Borg play: the release package's launcher (#819, #823, #824).
#   ./watch-borg.sh [--terminal] [NAME]
# Opens the game as character NAME (default RustBorg): in an X11 window on
# Linux when there is a display, otherwise (macOS, no display, or
# --terminal) in this terminal with the curses frontend (at least 80x24).
# Saves and settings live in $BORG_PLAY_DATA_DIR (default: data/ next to this
# script); the first start copies the Borg's settings (borg/) there, and
# later starts keep any change made to data/user/borg.txt. Create or load a
# character, then press Ctrl-Z and z to start the Borg; any key stops it.
# In a source checkout, use the repository's own watch-borg.sh instead.
set -euo pipefail
PKG="$(cd "$(dirname "$0")" && pwd)"
DATA="${BORG_PLAY_DATA_DIR:-$PKG/data}"
MODE=""
NAME=RustBorg
while [ "$#" -gt 0 ]; do
    case "$1" in
        --terminal) MODE=gcu; shift ;;
        -h|--help) sed -n '2,11p' "$0"; exit 0 ;;
        -*|"") echo "Usage: $0 [--terminal] [NAME]" >&2; exit 2 ;;
        *) NAME="$1"; shift ;;
    esac
done
if [ -z "$MODE" ]; then
    if [ "$(uname -s)" = Linux ] && [ -n "${DISPLAY:-}" ]; then MODE=x11; else MODE=gcu; fi
fi
mkdir -p "$DATA/user" "$DATA/save" "$DATA/archive" "$DATA/panic" "$DATA/scores"
for f in "$PKG"/borg/*.txt; do
    [ -e "$DATA/user/$(basename "$f")" ] || cp "$f" "$DATA/user/"
done
# Angband refuses to start without a UTF-8 locale. C.UTF-8 is glibc's;
# macOS has en_US.UTF-8.
case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *UTF-8*|*utf-8*|*utf8*|*UTF8*) ;;
    *) if [ "$(uname -s)" = Darwin ]; then export LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
       else export LANG=C.UTF-8 LC_ALL=C.UTF-8; fi ;;
esac
echo "Create or load a character; Ctrl-Z then z starts the Borg. Any key stops it."
[ "$MODE" = x11 ] || sleep 1
cd "$PKG"   # the game finds ./lib relative to the working directory
# Angband resolves -u immediately: select the save directory first.
exec ./angband -m"$MODE" -duser="$DATA/user" -dsave="$DATA/save" -darchive="$DATA/archive" \
    -dpanic="$DATA/panic" -dscores="$DATA/scores" -u"$NAME"

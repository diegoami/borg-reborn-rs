#!/usr/bin/env bash
# Watch the Borg play in a window: the release package's launcher (#819).
#   ./watch-borg.sh [NAME]
# Opens the game as character NAME (default RustBorg) with the X11 frontend.
# Saves and settings live in $BORG_PLAY_DATA_DIR (default: data/ next to this
# script); the first start copies the Borg's settings (borg/) there, and
# later starts keep any change made to data/user/borg.txt. Create or load a
# character, then press Ctrl-Z and z to start the Borg; any key stops it.
# Needs a display and the X11 library (libx11-6).
# In a source checkout, use the repository's own watch-borg.sh instead.
set -euo pipefail
PKG="$(cd "$(dirname "$0")" && pwd)"
DATA="${BORG_PLAY_DATA_DIR:-$PKG/data}"
NAME="${1:-RustBorg}"
case "$NAME" in -h|--help) sed -n '2,10p' "$0"; exit 0;; -*|"") echo "Usage: $0 [NAME]" >&2; exit 2;; esac
mkdir -p "$DATA"/{user,save,archive,panic,scores}
for f in "$PKG"/borg/*.txt; do
    [ -e "$DATA/user/$(basename "$f")" ] || cp "$f" "$DATA/user/"
done
[ -n "${DISPLAY:-}" ] || { echo "watch-borg.sh: no DISPLAY; for a game without a window use ./play-headless.sh" >&2; exit 2; }
# Angband refuses to start without a UTF-8 locale; C.UTF-8 is in glibc.
case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *UTF-8*|*utf-8*|*utf8*|*UTF8*) ;;
    *) export LANG=C.UTF-8 LC_ALL=C.UTF-8 ;;
esac
echo "Create or load a character; Ctrl-Z then z starts the Borg. Any key stops it."
cd "$PKG"   # the game finds ./lib relative to the working directory
# Angband resolves -u immediately: select the save directory first.
exec ./angband -mx11 -duser="$DATA/user" -dsave="$DATA/save" -darchive="$DATA/archive" \
    -dpanic="$DATA/panic" -dscores="$DATA/scores" -u"$NAME"

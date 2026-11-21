#!/usr/bin/env bash
# Watch the Rust Borg play in a window (docs/SPEC.md §8, §11; task TE.6).
#   ./watch-borg.sh [--build-only] [--reset-settings] [--name NAME | NAME]
# Builds our integration with the X11 frontend (build/watch/, the same
# engine, patches, frontend and Borg as harness/build.sh) and opens the game
# as character NAME (default RustBorg), with its saves and settings in
# .angband-reborn/ (kept between builds). The first start writes the shipped
# settings (harness/borg.txt and its scary list, prepared as a run's, #826)
# into .angband-reborn/user/, so the borg-rs core plays them; later starts
# keep that borg.txt (yours to edit) and say when it differs from the
# shipped one; --reset-settings writes the shipped one again. Create or load a character, then
# press Ctrl-Z and z to start the Borg; any key stops it. Needs the X11
# development libraries (libx11-dev) and a display.
# A watched game is not a measured run: its death is not recorded (RULES 12;
# to record one, reproduce it with harness/run.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BUILD="$ROOT/build/watch"
DATA="${BORG_PLAY_DATA_DIR:-$ROOT/.angband-reborn}"
BUILD_ONLY=0
RESET=0
NAME=RustBorg
NAME_SET=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --build-only) BUILD_ONLY=1; shift ;;
        --reset-settings) RESET=1; shift ;;
        --name)
            if [ "$#" -lt 2 ] || [ -z "$2" ] || [[ "$2" = -* ]]; then
                echo "--name requires a character name" >&2; exit 2
            fi
            if [ "$NAME_SET" = 1 ]; then
                echo "Specify the character name only once" >&2; exit 2
            fi
            NAME="$2"; NAME_SET=1; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--build-only] [--reset-settings] [--name NAME | NAME]"
            echo "Default name: RustBorg. Use the same name to reload its save."
            exit 0 ;;
        -*) echo "Unknown option: $1" >&2; exit 2 ;;
        *)
            if [ "$NAME_SET" = 1 ] || [ -z "$1" ]; then
                echo "Specify one nonempty character name" >&2; exit 2
            fi
            NAME="$1"; NAME_SET=1; shift ;;
    esac
done
if [ -f "$BUILD/CMakeCache.txt" ] && ! grep -qx "CMAKE_HOME_DIRECTORY:INTERNAL=$ROOT/engine" "$BUILD/CMakeCache.txt"; then
    rm -rf "$BUILD"
fi
cmake -S "$ROOT/engine" -B "$BUILD" -DCMAKE_BUILD_TYPE=Release -DSUPPORT_X11=ON >"$BUILD.cmake.log" 2>&1 || {
    cat "$BUILD.cmake.log"; exit 1; }
make -C "$BUILD" -j"$(nproc)" >"$BUILD.make.log" 2>&1 || { tail -50 "$BUILD.make.log"; exit 1; }
echo "Built: $BUILD/game/angband"
if [ "$BUILD_ONLY" = 1 ]; then
    [ "$RESET" = 0 ] || echo "watch-borg.sh: --reset-settings applies when the game starts, not with --build-only" >&2
    exit 0
fi
mkdir -p "$DATA"/{user,save,archive,panic,scores}
# #826: without a borg.txt the Borg writes the C Borg's defaults, which name
# no core, and the frozen reborn core plays instead of borg-rs.
SHIPPED=$(mktemp -d "${TMPDIR:-/tmp}/borg-settings.XXXXXXXX")
trap 'rm -rf "$SHIPPED"' EXIT
"$ROOT/harness/prepare-borg-txt.sh" "$SHIPPED"
if [ "$RESET" = 1 ] || [ ! -e "$DATA/user/borg.txt" ]; then
    cp "$SHIPPED/borg.txt" "$DATA/user/"
    echo "Settings: the shipped harness/borg.txt, in $DATA/user/borg.txt"
elif cmp -s "$SHIPPED/borg.txt" "$DATA/user/borg.txt"; then
    echo "Settings: the shipped harness/borg.txt"
else
    echo "Settings: your $DATA/user/borg.txt (it differs from the shipped harness/borg.txt; --reset-settings takes the shipped one)"
fi
# The files borg.txt names (the scary list): every shipped one that is
# missing, whatever borg.txt is (review of #866: a kept borg.txt naming a
# list that is not there made the core exit at start).
for f in "$SHIPPED"/*; do
    [ -e "$f" ] && [ "$(basename "$f")" != borg.txt ] || continue
    { [ "$RESET" = 1 ] || [ ! -e "$DATA/user/$(basename "$f")" ]; } && cp "$f" "$DATA/user/"
done
LIST="$(sed -n 's/^[[:space:]]*borg_scary_list[[:space:]]*=[[:space:]]*//p' "$DATA/user/borg.txt" | tail -1 | tr -d '[:space:]')"
if [ -n "$LIST" ] && [ "$LIST" != none ] && [ ! -e "$DATA/user/$LIST" ]; then
    echo "watch-borg.sh: $DATA/user/borg.txt names the scary list $LIST, which is not in $DATA/user/ (the Borg would stop at start): copy it there, or --reset-settings" >&2
    exit 2
fi
rm -rf "$SHIPPED"; trap - EXIT   # exec below would skip the trap
echo "Create or load a character; Ctrl-Z then z starts the Rust Borg. Any key stops it."
cd "$BUILD/game"   # the game finds ./lib relative to the working directory
# Angband resolves -u immediately: select the save directory first.
exec ./angband -mx11 -duser="$DATA/user" -dsave="$DATA/save" -darchive="$DATA/archive" \
    -dpanic="$DATA/panic" -dscores="$DATA/scores" -u"$NAME"

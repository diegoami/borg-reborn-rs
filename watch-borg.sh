#!/usr/bin/env bash
# Watch the Rust Borg play in a window (docs/SPEC.md §8, §11; task TE.6).
#   ./watch-borg.sh [--build-only] [--reset-settings] [--name NAME | NAME]
# Builds our integration with the X11 frontend (build/watch/, the same
# engine, patches, frontend and Borg as harness/build.sh) and opens the game
# as character NAME (default RustBorg), with its saves and settings in
# .angband-reborn/ (kept between builds). The first start writes the shipped
# settings (harness/pborg.yaml and its scary list, prepared as a run's, #826)
# into .angband-reborn/user/, so the borg-rs core plays them; later starts
# keep that pborg.yaml (yours to edit) and say when it differs from the
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
DATA="$(cd "$DATA" && pwd)"
# Install PBorg settings in their own file, beside any existing CBorg configuration.
SHIPPED=$(mktemp -d "${TMPDIR:-/tmp}/borg-settings.XXXXXXXX")
trap 'rm -rf "$SHIPPED"' EXIT
"$ROOT/harness/prepare-pborg-config.sh" "$SHIPPED"
if [ "$RESET" = 1 ] || [ ! -e "$DATA/user/pborg.yaml" ]; then
    cp "$SHIPPED/pborg.yaml" "$DATA/user/"
    echo "Settings: the shipped harness/pborg.yaml, in $DATA/user/pborg.yaml"
elif cmp -s "$SHIPPED/pborg.yaml" "$DATA/user/pborg.yaml"; then
    echo "Settings: the shipped harness/pborg.yaml"
else
    echo "Settings: your $DATA/user/pborg.yaml (it differs from the shipped harness/pborg.yaml; --reset-settings takes the shipped one)"
fi
# Restore shipped scary-list files missing from the user directory.
for f in "$SHIPPED"/*; do
    [ -e "$f" ] && [ "$(basename "$f")" != pborg.yaml ] || continue
    { [ "$RESET" = 1 ] || [ ! -e "$DATA/user/$(basename "$f")" ]; } && cp "$f" "$DATA/user/"
done
GAME_DIRS=(-duser="$DATA/user" -dsave="$DATA/save" -darchive="$DATA/archive"
    -dpanic="$DATA/panic" -dscores="$DATA/scores")
if ! (cd "$BUILD/game" && ./angband -mborgh "${GAME_DIRS[@]}" -- -check-pborg-settings); then
    echo "watch-borg: check $DATA/user/pborg.yaml and its scary list, or use --reset-settings" >&2
    exit 2
fi
rm -rf "$SHIPPED"; trap - EXIT   # exec below would skip the trap
echo "Create or load a character; Ctrl-Z then z starts PBorg by default. Ctrl-Z then B switches Borgs. Any key stops the active Borg."
cd "$BUILD/game"   # the game finds ./lib relative to the working directory
# Angband resolves -u immediately: select the save directory first.
exec ./angband -mx11 "${GAME_DIRS[@]}" -u"$NAME"

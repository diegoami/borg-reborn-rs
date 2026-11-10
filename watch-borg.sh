#!/usr/bin/env bash
# Watch the Rust Borg play in a window (docs/SPEC.md §8, §11; task TE.6).
#   ./watch-borg.sh [--build-only] [NAME]
# Builds our integration with the X11 frontend (build/watch/, the same
# engine, patches, frontend and Borg as harness/build.sh) and opens the game
# as character NAME (default RustBorg), with its saves and settings in
# .angband-reborn/ (kept between builds). Create or load a character, then
# press Ctrl-Z and z to start the Borg; any key stops it. Needs the X11
# development libraries (libx11-dev) and a display.
# A watched game is not a measured run: its death is not recorded (RULES 12;
# to record one, reproduce it with harness/run.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BUILD="$ROOT/build/watch"
DATA="${BORG_PLAY_DATA_DIR:-$ROOT/.angband-reborn}"
BUILD_ONLY=0
[ "${1:-}" = --build-only ] && { BUILD_ONLY=1; shift; }
NAME="${1:-RustBorg}"
if [ -f "$BUILD/CMakeCache.txt" ] && ! grep -qx "CMAKE_HOME_DIRECTORY:INTERNAL=$ROOT/engine" "$BUILD/CMakeCache.txt"; then
    rm -rf "$BUILD"
fi
cmake -S "$ROOT/engine" -B "$BUILD" -DCMAKE_BUILD_TYPE=Release -DSUPPORT_X11=ON >"$BUILD.cmake.log" 2>&1 || {
    cat "$BUILD.cmake.log"; exit 1; }
make -C "$BUILD" -j"$(nproc)" >"$BUILD.make.log" 2>&1 || { tail -50 "$BUILD.make.log"; exit 1; }
echo "Built: $BUILD/game/angband"
[ "$BUILD_ONLY" = 0 ] || exit 0
mkdir -p "$DATA"/{user,save,archive,panic,scores}
echo "Create or load a character; Ctrl-Z then z starts the Rust Borg. Any key stops it."
cd "$BUILD/game"   # the game finds ./lib relative to the working directory
exec ./angband -mx11 -u"$NAME" -duser="$DATA/user" -dsave="$DATA/save" -darchive="$DATA/archive" \
    -dpanic="$DATA/panic" -dscores="$DATA/scores"

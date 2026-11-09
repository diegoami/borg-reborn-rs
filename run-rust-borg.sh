#!/usr/bin/env bash
# Build and launch interactive Angband with the Rust Borg backend.
set -euo pipefail

PROJECT_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PLAY_BUILD="$PROJECT_ROOT/build/play-rust"
PLAY_DATA="${BORG_PLAY_DATA_DIR:-$PROJECT_ROOT/.angband-reborn}"
BUILD_ONLY=0
PLAY_CHARACTER="${BORG_PLAY_CHARACTER:-RustBorg}"
RESTORE_SLOT=""

case "${1:-}" in
    -h|--help)
        cat <<'EOF'
Usage: ./run-rust-borg.sh [--build-only] [Angband arguments]

Builds the Rust Borg and starts Angband's graphical X11 frontend.
Create or load a character, then press Ctrl-Z followed by z to start the Borg.
Confirm the initial Borg warning if shown. Press any key to stop the Borg.

Saves and settings: .angband-reborn/ (preserved between builds).
GUI sessions are recorded in .angband-reborn/recordings/ for verified replay.
The initial borg.txt uses the stock settings, with cheat death off.
Pass -n to start a new character, or -uNAME to select another save name.
--build-only builds and checks the executable without opening a window.
Set JOBS to control build parallelism (default: 4).
Uses the same 28-pixel Courier main-window font as better-borg-c/start-borg.sh.
Set BORG_PLAY_FONT to override it.
Opens one main window by default; arguments after -- override frontend options.
Set BORG_PLAY_DATA_DIR to use a separate save/settings directory.
Use ./start-borg.sh NAME to select a named character.
--from-backup[=1..5] restores a rolling backup, preserving the current save.

Requires CMake, a C compiler, Cargo/Rust, and X11 development libraries.
EOF
        exit 0
        ;;
    --build-only) BUILD_ONLY=1; shift ;;
esac

mkdir -p "$PLAY_BUILD"
printf 'Building Angband with the Rust Borg...\n'
CONFIGURE=0
for option in 'BORG_IMPL:STRING=rust' 'SUPPORT_BORG:BOOL=ON' \
    'SUPPORT_X11_FRONTEND:BOOL=ON' 'SUPPORT_BORG_HARNESS:BOOL=OFF' \
    'SUPPORT_BORG_RECORDING:BOOL=ON' 'SUPPORT_TEST_FRONTEND:BOOL=ON' \
    'CMAKE_BUILD_TYPE:STRING=Release' 'SC_INSTALL:BOOL=ON' 'BORG_RS_FEATURES:STRING='; do
    if ! grep -Fxq -- "$option" "$PLAY_BUILD/CMakeCache.txt" 2>/dev/null; then
        CONFIGURE=1
        break
    fi
done
# The build system automatically reconfigures when its CMake inputs change.
if [ "$CONFIGURE" = 1 ]; then
    cmake -S "$PROJECT_ROOT/angband" -B "$PLAY_BUILD" \
    -DCMAKE_BUILD_TYPE=Release -DBORG_IMPL=rust -DSUPPORT_BORG=ON \
    -DSUPPORT_X11_FRONTEND=ON -DSUPPORT_GCU_FRONTEND=OFF \
    -DSUPPORT_NCURSES_FRONTEND=OFF -DSUPPORT_SDL_FRONTEND=OFF \
    -DSUPPORT_SDL2_FRONTEND=OFF -DSUPPORT_BORG_HARNESS=OFF \
    -DSUPPORT_BORG_RECORDING=ON -DSUPPORT_TEST_FRONTEND=ON -DBUILD_DOC=OFF -DSC_INSTALL=ON -DBORG_RS_FEATURES= \
    >"$PLAY_BUILD/configure.log" 2>&1 || {
        cat "$PLAY_BUILD/configure.log" >&2
        exit 1
    }
fi
cmake --build "$PLAY_BUILD" --parallel "${JOBS:-4}" \
    >"$PLAY_BUILD/build.log" 2>&1 || {
        tail -n 60 "$PLAY_BUILD/build.log" >&2
        exit 1
    }
if [ "$("$PROJECT_ROOT/harness/identity.sh" binary-impl "$PLAY_BUILD/game/angband")" != rust ]; then
    printf 'The built executable could not be verified as the Rust Borg.\n' >&2
    exit 1
fi

if [ "$BUILD_ONLY" = 1 ]; then
    printf 'Built: %s\n' "$PLAY_BUILD/game/angband"
    exit 0
fi

GAME_ARGS=()
FRONTEND_ARGS=0
for argument in "$@"; do
    if [ "$FRONTEND_ARGS" = 1 ]; then GAME_ARGS+=("$argument"); continue; fi
    case "$argument" in
        --from-backup) RESTORE_SLOT=1;;
        --from-backup=*) RESTORE_SLOT="${argument#--from-backup=}";;
        -u?*) PLAY_CHARACTER="${argument#-u}";;
        --) FRONTEND_ARGS=1; GAME_ARGS+=("$argument");;
        *) GAME_ARGS+=("$argument");;
    esac
done
[[ "$PLAY_CHARACTER" =~ ^[A-Za-z0-9][A-Za-z0-9_.-]{0,63}$ ]] || { echo 'Invalid character name' >&2; exit 2; }
[[ -z "$RESTORE_SLOT" || "$RESTORE_SLOT" =~ ^[1-5]$ ]] || { echo 'Backup slot must be 1..5' >&2; exit 2; }
set -- "${GAME_ARGS[@]}"
mkdir -p "$PLAY_DATA"/{user,save,archive,panic,scores}
PLAY_DATA="$(cd -- "$PLAY_DATA" && pwd)"
mkdir -p "$PLAY_DATA/save/.locks"
exec 9>"$PLAY_DATA/save/.locks/$PLAY_CHARACTER.lock"
flock -n 9 || { echo "Character '$PLAY_CHARACTER' is already open through a launcher" >&2; exit 1; }
if [ -n "$RESTORE_SLOT" ]; then
    python3 "$PROJECT_ROOT/tools/borg_saves.py" restore "$PLAY_DATA/save/$PLAY_CHARACTER" "$RESTORE_SLOT"
fi
python3 "$PROJECT_ROOT/tools/borg_saves.py" backup "$PLAY_DATA/save/$PLAY_CHARACTER"
if [ ! -e "$PLAY_DATA/user/borg.txt" ]; then
    cp "$PROJECT_ROOT/angband/src/borg/borg.txt" "$PLAY_DATA/user/borg.txt"
fi
# Match better-borg-c/start-borg.sh; the window-specific setting also overrides
# a small FONT_0 saved by an earlier launch. Keep fixed for auxiliary windows.
export ANGBAND_X11_FONT="${ANGBAND_X11_FONT:-fixed}"
export ANGBAND_X11_FONT_0="${BORG_PLAY_FONT:-${ANGBAND_X11_FONT_0:--adobe-courier-medium-r-normal--28-*-*-*-p-*-iso8859-1}}"
PLAY_FRONTEND_OPTIONS=0
for argument in "$@"; do
    if [ "$argument" = -- ]; then PLAY_FRONTEND_OPTIONS=1; break; fi
done
if [ "$PLAY_FRONTEND_OPTIONS" = 0 ]; then
    set -- "$@" -- -n1
fi
printf 'Create/load a character; Ctrl-Z then z starts the Rust Borg. Any key stops it.\n'
exec python3 "$PROJECT_ROOT/tools/borg_gui.py" \
    "$PLAY_BUILD/game/angband" "$PLAY_DATA" "$PLAY_CHARACTER" "$@"

#!/usr/bin/env bash
# Install the window launcher (angband-reborn-record) for a recording build:  install-launcher.sh [BUILD_DIR]
# BUILD_DIR: a build made by setup.sh (default: $BRR_BUILD, else ~/angband-builds/current).
# Builds the game of that revision with X11 AND the recorder (setup.sh's build has no window), builds
# the recorder's own command-line tool from the build's vendored copy, and copies both, the data,
# the launcher and watch-recording.py into BUILD_DIR/launcher, so a later rebuild cannot replace the
# executable that recordings need. Then links ~/.local/bin/angband-reborn-record to it
# (NO_LINK=1 skips; the previous target is printed). Needs the libX11 and OpenSSL headers.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
if [ $# -ge 1 ]; then BRR_BUILD="$1"
elif [ -z "${BRR_BUILD:-}" ]; then
    [ -d "$HOME/angband-builds/current" ] || { echo "install-launcher.sh: give a build directory (none is current)" >&2; exit 2; }
    BRR_BUILD="$(realpath "$HOME/angband-builds/current")"
fi
BRR_BUILD="$(realpath "$BRR_BUILD")"
[ -f "$BRR_BUILD/recorder-pin.json" ] && [ -d "$BRR_BUILD/engine" ] || { echo "install-launcher.sh: $BRR_BUILD is not a recording build" >&2; exit 2; }
DEST="$BRR_BUILD/launcher"
[ ! -e "$DEST" ] || { echo "install-launcher.sh: $DEST exists; remove it (or make a new build) first" >&2; exit 2; }
REV="$(git -C "$BRR_BUILD" rev-parse HEAD)"
TAG="$(python3 -c "import json,sys;p=json.load(open(sys.argv[1]));print(p['tag'],p['commit'])" "$BRR_BUILD/recorder-pin.json")"
echo "install-launcher.sh: ${REV:0:8}, recorder $TAG, into $DEST"
cd "$BRR_BUILD"
flock /tmp/angband-validation.lock bash -c '
    cmake -S engine -B build/rec-x11 -DCMAKE_BUILD_TYPE=Release -DSUPPORT_BORG_RECORDING=ON -DSUPPORT_X11=ON >build-rec-x11.cmake.log 2>&1 &&
    make -C build/rec-x11 -j"$(nproc)" >build-rec-x11.make.log 2>&1 &&
    cargo build --release --locked --manifest-path vendor/angband-recorder/Cargo.toml --target-dir build/recorder-cli >build-recorder-cli.log 2>&1' \
    || { tail -30 build-rec-x11.cmake.log build-rec-x11.make.log build-recorder-cli.log 2>/dev/null; exit 1; }
mkdir -p "$DEST"
cp -a build/rec-x11/game "$DEST/game"
cp build/recorder-cli/release/angband-recorder "$DEST/angband-recorder"
cp "$HERE/angband-reborn-record" "$HERE/watch-recording.py" "$DEST/"
chmod +x "$DEST/angband-reborn-record" "$DEST/angband-recorder"
{
    echo "git_rev=$REV"
    echo "frontend=X11"
    echo "recorder_pin=$TAG"
    echo "installed=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "game_sha256=$(sha256sum "$DEST/game/angband" | cut -d' ' -f1)"
    echo "recorder_cli_sha256=$(sha256sum "$DEST/angband-recorder" | cut -d' ' -f1)"
} > "$DEST/INSTALL-RECEIPT"
cat "$DEST/INSTALL-RECEIPT"
if [ "${NO_LINK:-0}" != 1 ]; then
    mkdir -p "$HOME/.local/bin"
    echo "install-launcher.sh: previous angband-reborn-record -> $(readlink "$HOME/.local/bin/angband-reborn-record" || echo none)"
    ln -sfnT "$DEST/angband-reborn-record" "$HOME/.local/bin/angband-reborn-record"
    echo "install-launcher.sh: ready; run: angband-reborn-record CHARACTER"
else
    echo "install-launcher.sh: ready; run: $DEST/angband-reborn-record CHARACTER"
fi

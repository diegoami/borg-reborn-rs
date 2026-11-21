#!/usr/bin/env bash
# Assemble the Linux release package (#819).
#   scripts/release/package-linux.sh VERSION HEADLESS_BUILD X11_BUILD OUTDIR [DOCS_DIR]
# HEADLESS_BUILD and X11_BUILD are build directories made by harness/build.sh
# (BORG_X11=OFF and BORG_X11=ON): each holds game/angband, game/lib and
# BUILD_ID. DOCS_DIR, if given, holds the converted Angband manual (html/).
# Writes OUTDIR/borg-reborn-rs-VERSION-linux-x86_64.tar.gz and prints its path.
# The package plays what a harness run plays: harness/borg.txt (class files
# folded in) and its scary list, prepared as harness/run.sh prepares a run's
# user/borg.txt; the launchers in scripts/release/ copy them into the
# player's data directory.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
[ $# -ge 4 ] || { sed -n '2,11p' "$0" >&2; exit 2; }
VERSION="$1"; HEADLESS="$(realpath "$2")"; X11="$(realpath "$3")"; OUT="$(realpath -m "$4")"
DOCS="${5:-}"
[[ "$VERSION" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "package-linux.sh: bad version '$VERSION'" >&2; exit 2; }
for b in "$HEADLESS" "$X11"; do
    [ -x "$b/game/angband" ] && [ -d "$b/game/lib" ] && [ -s "$b/BUILD_ID" ] || {
        echo "package-linux.sh: $b is not a harness/build.sh build directory" >&2; exit 2; }
done
grep -qx 'SUPPORT_X11:BOOL=ON' "$X11/CMakeCache.txt" || { echo "package-linux.sh: $X11 was not built with BORG_X11=ON" >&2; exit 2; }
grep -qx 'SUPPORT_X11:BOOL=OFF' "$HEADLESS/CMakeCache.txt" || { echo "package-linux.sh: $HEADLESS was built with X11" >&2; exit 2; }
# Both binaries come from this checkout's build inputs.
SRC=$("$ROOT/harness/identity.sh" source-hash rust)
for b in "$HEADLESS" "$X11"; do
    grep -qx "source_sha256=$SRC" "$b/BUILD_ID" || { echo "package-linux.sh: $b/BUILD_ID is not this checkout's build" >&2; exit 2; }
    grep -qx "binary_sha256=$(sha256sum < "$b/game/angband" | cut -d' ' -f1)" "$b/BUILD_ID" || {
        echo "package-linux.sh: $b/game/angband does not match its BUILD_ID" >&2; exit 2; }
done

NAME="borg-reborn-rs-$VERSION-linux-$(uname -m)"
STAGE="$OUT/$NAME"
rm -rf "$STAGE"; mkdir -p "$STAGE"/{borg/plans,borg/class,docs,LICENSES}

# The game: the X11 build (both frontends) and the headless one (no X11
# library needed), sharing the game data.
cp "$X11/game/angband" "$STAGE/angband"
cp "$HEADLESS/game/angband" "$STAGE/angband-headless"
cp -a "$HEADLESS/game/lib" "$STAGE/lib"
diff -r "$HEADLESS/game/lib" "$X11/game/lib" >/dev/null || { echo "package-linux.sh: the two builds' lib/ differ" >&2; exit 1; }
cp "$X11/BUILD_ID" "$STAGE/BUILD_ID.angband"
cp "$HEADLESS/BUILD_ID" "$STAGE/BUILD_ID.angband-headless"
# A packaged binary is not at the path the BUILD_ID names.
sed -i 's|^binary=.*|binary=angband|' "$STAGE/BUILD_ID.angband"
sed -i 's|^binary=.*|binary=angband-headless|' "$STAGE/BUILD_ID.angband-headless"

# The Borg's settings, as harness/run.sh writes a run's user/borg.txt
# before it substitutes the run's race and class: the world key removed
# (none: upstream's data), the scary list's selecting line moved to the end
# under its SHA-256.
BT="$STAGE/borg/borg.txt"
sed '/^[[:space:]]*borg_world[[:space:]]*=/d' "$ROOT/harness/borg.txt" > "$BT"
SCARY_NAME="$(sed -n 's/^[[:space:]]*borg_scary_list[[:space:]]*=[[:space:]]*//p' "$BT" | tail -1 | tr -d '[:space:]')"
if [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ ]] && [ -f "$ROOT/harness/$SCARY_NAME" ] || {
        echo "package-linux.sh: scary list '$SCARY_NAME' not in harness/" >&2; exit 1; }
    cp "$ROOT/harness/$SCARY_NAME" "$STAGE/borg/$SCARY_NAME"
    sed -i '/^[[:space:]]*borg_scary_list[[:space:]]*=/d' "$BT"
    printf '\n# harness/run.sh (#518): scary list sha256 %s\nborg_scary_list = %s\n' \
        "$(sha256sum < "$ROOT/harness/$SCARY_NAME" | cut -d' ' -f1)" "$SCARY_NAME" >> "$BT"
fi
# The class files are already folded into borg.txt; shipped for reading.
cp "$ROOT"/harness/borg_*.txt "$STAGE/borg/class/"
cp "$ROOT"/harness/plans/* "$STAGE/borg/plans/"

# Launchers and the short guide.
install -m 755 "$ROOT/scripts/release/watch-borg.sh" "$STAGE/watch-borg.sh"
install -m 755 "$ROOT/scripts/release/play-headless.sh" "$STAGE/play-headless.sh"
cp "$ROOT/scripts/release/PLAYING.txt" "$STAGE/PLAYING.txt"
cp "$ROOT/README.md" "$STAGE/README.md"
[ ! -f "$ROOT/docs/user/GUIDE.md" ] || cp "$ROOT/docs/user/GUIDE.md" "$STAGE/docs/GUIDE.md"
if [ -n "$DOCS" ] && [ -d "$DOCS/html" ]; then
    cp -a "$DOCS/html" "$STAGE/docs/angband-manual"
else
    # Unconverted, as upstream ships it in its source tree.
    mkdir -p "$STAGE/docs/angband-manual-rst"
    cp "$ROOT"/engine/angband/docs/*.rst "$STAGE/docs/angband-manual-rst/"
fi

# Licences of every part.
cp "$ROOT/LICENSE" "$STAGE/LICENSES/borg-reborn-rs-GPL-2.0.txt"
cp "$ROOT/engine/angband/docs/copying.rst" "$STAGE/LICENSES/angband-copying.rst"
cp "$ROOT/borg-core/PIN" "$STAGE/LICENSES/borg-core-PIN.txt"
python3 "$ROOT/scripts/release/rust-licenses.py" "$ROOT" > "$STAGE/LICENSES/rust-crates.txt"

ENGINE=$(sed -n 's/^commit=//p' "$ROOT/engine/UPSTREAM")
{
    echo "borg-reborn-rs $VERSION (Linux $(uname -m))"
    echo
    echo "commit:          $(git -C "$ROOT" rev-parse HEAD)"
    echo "angband:         upstream $ENGINE, unchanged (engine/angband/), with the"
    echo "                 harness hooks of engine/patches/ and the borgh frontend"
    echo "borg core:       borg-rs, vendored as borg-core/ ($(head -1 "$ROOT/borg-core/PIN" 2>/dev/null || echo 'see borg-core/PIN'))"
    echo "settings:        borg/borg.txt sha256 $(sha256sum < "$BT" | cut -d' ' -f1)"
    echo "                 (from harness/borg.txt sha256 $(sha256sum < "$ROOT/harness/borg.txt" | cut -d' ' -f1))"
    echo "source_sha256:   $SRC"
    echo
    echo "Licences: LICENSES/ (Angband: angband-copying.rst; borg-rs's core and"
    echo "ours: GPL-2.0; the Rust crates linked in: rust-crates.txt)."
} > "$STAGE/RELEASE.txt"

(cd "$STAGE" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
TAR="$OUT/$NAME.tar.gz"
tar -C "$OUT" --owner=0 --group=0 --numeric-owner --sort=name -czf "$TAR" "$NAME"
echo "$TAR"

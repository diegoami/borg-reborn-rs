#!/usr/bin/env bash
# Assemble a release package (#819; macOS #824, Windows #823).
#   scripts/release/package.sh PLATFORM VERSION OUTDIR DOCS_DIR NAME=BUILD [NAME=BUILD ...]
# PLATFORM is linux, macos or windows. Each NAME=BUILD is a build directory
# made by harness/build.sh (game/angband[.exe], game/lib, BUILD_ID), packaged
# as the executable NAME (NAME.exe on Windows); the first one's lib/ is
# shipped and the others' must be the same. DOCS_DIR holds the converted
# Angband manual (html/); "-" ships the manual's .rst sources instead.
# Writes OUTDIR/borg-reborn-rs-VERSION-PLATFORM-ARCH.tar.gz (.zip on Windows)
# and prints its path. Runs where the build ran: bash with GNU coreutils
# (macOS: Homebrew's) and Python 3.
# The package plays what a harness run plays: harness/borg.txt (class files
# folded in) and its scary list, prepared as harness/run.sh prepares a run's
# user/borg.txt; the launchers in scripts/release/ copy them into the
# player's data directory.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
[ $# -ge 5 ] || { sed -n '2,15p' "$0" >&2; exit 2; }
PLATFORM="$1"; VERSION="$2"; OUT="$3"; DOCS="$4"; shift 4
case "$PLATFORM" in linux|macos|windows) ;; *) echo "package.sh: PLATFORM is linux, macos or windows" >&2; exit 2;; esac
[[ "$VERSION" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "package.sh: bad version '$VERSION'" >&2; exit 2; }
EXE=""; [ "$PLATFORM" != windows ] || EXE=".exe"
mkdir -p "$OUT"; OUT="$(cd "$OUT" && pwd)"
sha() { python3 -c 'import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"; }

SRC=$("$ROOT/harness/identity.sh" source-hash rust)
BUILDS=()
for spec in "$@"; do
    name="${spec%%=*}"; dir="${spec#*=}"
    [[ "$name" =~ ^[a-z][a-z0-9-]*$ ]] && [ "$name" != "$spec" ] || { echo "package.sh: '$spec' is not NAME=BUILD" >&2; exit 2; }
    [ -f "$dir/game/angband$EXE" ] && [ -d "$dir/game/lib" ] && [ -s "$dir/BUILD_ID" ] || {
        echo "package.sh: $dir is not a harness/build.sh build directory" >&2; exit 2; }
    # Every binary comes from this checkout's build inputs.
    grep -qx "source_sha256=$SRC" "$dir/BUILD_ID" || { echo "package.sh: $dir/BUILD_ID is not this checkout's build" >&2; exit 2; }
    grep -qx "binary_sha256=$(sha "$dir/game/angband$EXE")" "$dir/BUILD_ID" || {
        echo "package.sh: $dir/game/angband$EXE does not match its BUILD_ID" >&2; exit 2; }
    # The frontends the package promises (PLAYING.txt; review of #851, the
    # check package-linux.sh had): angband-headless without X11 or curses,
    # Linux's angband with both, macOS's and Windows' with curses only.
    want=""
    case "$name:$PLATFORM" in
        angband-headless:*) want="SUPPORT_X11:BOOL=OFF SUPPORT_GCU:BOOL=OFF";;
        angband:linux) want="SUPPORT_X11:BOOL=ON SUPPORT_GCU:BOOL=ON";;
        angband:*) want="SUPPORT_X11:BOOL=OFF SUPPORT_GCU:BOOL=ON";;
    esac
    for w in $want; do
        grep -qx "$w" "$dir/CMakeCache.txt" || { echo "package.sh: $name ($dir) is not built with $w" >&2; exit 2; }
    done
    BUILDS+=("$name=$dir")
done

ARCH=$(uname -m); [ "$ARCH" != aarch64 ] || ARCH=arm64
NAME="borg-reborn-rs-$VERSION-$PLATFORM-$ARCH"
STAGE="$OUT/$NAME"
rm -rf "$STAGE"; mkdir -p "$STAGE"/borg/plans "$STAGE"/borg/class "$STAGE"/docs "$STAGE"/LICENSES

# The game: each build as its own executable, sharing the game data.
FIRST=""
for b in "${BUILDS[@]}"; do
    name="${b%%=*}"; dir="${b#*=}"
    cp "$dir/game/angband$EXE" "$STAGE/$name$EXE"
    chmod 755 "$STAGE/$name$EXE"
    cp "$dir/BUILD_ID" "$STAGE/BUILD_ID.$name"
    # A packaged binary is not at the path the BUILD_ID names.
    sed -i "s|^binary=.*|binary=$name$EXE|" "$STAGE/BUILD_ID.$name"
    if [ -z "$FIRST" ]; then
        FIRST="$dir"; cp -R "$dir/game/lib" "$STAGE/lib"
    else
        diff -r "$FIRST/game/lib" "$dir/game/lib" >/dev/null || { echo "package.sh: the builds' lib/ differ" >&2; exit 1; }
    fi
done

# The Borg's settings, as harness/run.sh writes a run's user/borg.txt
# before it substitutes the run's race and class: the world key removed
# (none: upstream's data), the scary list's selecting line moved to the end
# under its SHA-256.
BT="$STAGE/borg/borg.txt"
sed '/^[[:space:]]*borg_world[[:space:]]*=/d' "$ROOT/harness/borg.txt" > "$BT"
SCARY_NAME="$(sed -n 's/^[[:space:]]*borg_scary_list[[:space:]]*=[[:space:]]*//p' "$BT" | tail -1 | tr -d '[:space:]')"
if [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ ]] && [ -f "$ROOT/harness/$SCARY_NAME" ] || {
        echo "package.sh: scary list '$SCARY_NAME' not in harness/" >&2; exit 1; }
    cp "$ROOT/harness/$SCARY_NAME" "$STAGE/borg/$SCARY_NAME"
    sed -i '/^[[:space:]]*borg_scary_list[[:space:]]*=/d' "$BT"
    printf '\n# harness/run.sh (#518): scary list sha256 %s\nborg_scary_list = %s\n' \
        "$(sha "$ROOT/harness/$SCARY_NAME")" "$SCARY_NAME" >> "$BT"
fi
# The class files are already folded into borg.txt; shipped for reading.
cp "$ROOT"/harness/borg_*.txt "$STAGE/borg/class/"
cp "$ROOT"/harness/plans/* "$STAGE/borg/plans/"

# Launchers and the short guide.
if [ "$PLATFORM" = windows ]; then
    # CRLF, as cmd.exe and Notepad expect.
    for f in play-headless.ps1 play-headless.cmd watch-borg.cmd PLAYING.txt; do
        src="$ROOT/scripts/release/windows/$f"; [ -f "$src" ] || src="$ROOT/scripts/release/$f"
        python3 -c 'import sys; d=open(sys.argv[1],"rb").read().replace(b"\r\n",b"\n"); open(sys.argv[2],"wb").write(d.replace(b"\n",b"\r\n"))' "$src" "$STAGE/$f"
    done
else
    install -m 755 "$ROOT/scripts/release/watch-borg.sh" "$STAGE/watch-borg.sh"
    install -m 755 "$ROOT/scripts/release/play-headless.sh" "$STAGE/play-headless.sh"
fi
[ "$PLATFORM" = windows ] || cp "$ROOT/scripts/release/PLAYING.txt" "$STAGE/PLAYING.txt"
cp "$ROOT/README.md" "$STAGE/README.md"
[ ! -f "$ROOT/docs/user/GUIDE.md" ] || cp "$ROOT/docs/user/GUIDE.md" "$STAGE/docs/GUIDE.md"
if [ "$DOCS" != - ] && [ -d "$DOCS/html" ]; then
    cp -R "$DOCS/html" "$STAGE/docs/angband-manual"
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
case "$PLATFORM" in
    macos|windows) echo "OpenSSL 3 (libcrypto) is linked in statically: Apache License 2.0, https://www.openssl.org/source/license.html" \
        > "$STAGE/LICENSES/openssl.txt";;
esac

ENGINE=$(sed -n 's/^commit=//p' "$ROOT/engine/UPSTREAM")
{
    echo "borg-reborn-rs $VERSION ($PLATFORM $ARCH)"
    echo
    echo "commit:          $(git -C "$ROOT" rev-parse HEAD)"
    echo "angband:         upstream $ENGINE, unchanged (engine/angband/), with the"
    echo "                 harness hooks of engine/patches/ and the borgh frontend"
    echo "borg core:       borg-rs, vendored as borg-core/ ($(head -1 "$ROOT/borg-core/PIN"))"
    echo "settings:        borg/borg.txt sha256 $(sha "$BT")"
    echo "                 (from harness/borg.txt sha256 $(sha "$ROOT/harness/borg.txt"))"
    echo "source_sha256:   $SRC"
    echo "executables:     $(for b in "${BUILDS[@]}"; do printf '%s%s ' "${b%%=*}" "$EXE"; done)"
    echo
    echo "Licences: LICENSES/ (Angband: angband-copying.rst; borg-rs's core and"
    echo "ours: GPL-2.0; the Rust crates linked in: rust-crates.txt)."
} > "$STAGE/RELEASE.txt"

# Checksums of every file, and the archive (Python: the same bytes from the
# same files on every platform; no GNU tar or zip needed).
python3 - "$OUT" "$NAME" "$PLATFORM" <<'PY'
import hashlib, os, stat, sys, tarfile, zipfile
out, name, platform = sys.argv[1:]
stage = os.path.join(out, name)
files = sorted(os.path.relpath(os.path.join(d, f), stage)
               for d, _, fs in os.walk(stage) for f in fs)
with open(os.path.join(stage, 'SHA256SUMS'), 'w', newline='\n') as sums:
    for f in files:
        if f != 'SHA256SUMS':
            h = hashlib.sha256(open(os.path.join(stage, f), 'rb').read()).hexdigest()
            sums.write(f'{h}  {f.replace(os.sep, "/")}\n')
files = sorted(set(files) | {'SHA256SUMS'})
def mode(f):
    return 0o755 if os.stat(os.path.join(stage, f)).st_mode & stat.S_IXUSR else 0o644
if platform == 'windows':
    path = os.path.join(out, name + '.zip')
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in files:
            info = zipfile.ZipInfo(name + '/' + f.replace(os.sep, '/'), (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = mode(f) << 16
            z.writestr(info, open(os.path.join(stage, f), 'rb').read())
else:
    path = os.path.join(out, name + '.tar.gz')
    with tarfile.open(path, 'w:gz') as t:
        dirs = set()
        for f in files:
            d = os.path.dirname(f)
            while d:
                dirs.add(d); d = os.path.dirname(d)
        dirs = sorted(dirs)
        for d in [''] + dirs:
            info = tarfile.TarInfo(name + ('/' + d.replace(os.sep, '/') if d else ''))
            info.type, info.mode = tarfile.DIRTYPE, 0o755
            t.addfile(info)
        for f in files:
            info = t.gettarinfo(os.path.join(stage, f), name + '/' + f.replace(os.sep, '/'))
            info.uid = info.gid = 0; info.uname = info.gname = ''; info.mode = mode(f); info.mtime = 0
            with open(os.path.join(stage, f), 'rb') as fh:
                t.addfile(info, fh)
print(path)
PY

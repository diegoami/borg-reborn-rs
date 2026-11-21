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
# before it substitutes the run's race and class (harness/prepare-borg-txt.sh,
# shared with watch-borg.sh and harness/play.sh).
BT="$STAGE/borg/borg.txt"
"$ROOT/harness/prepare-borg-txt.sh" "$STAGE/borg"
# The class files are already folded into borg.txt; shipped for reading.
cp "$ROOT"/harness/borg_*.txt "$STAGE/borg/class/"
cp "$ROOT"/harness/plans/* "$STAGE/borg/plans/"

# A game started directly (no -d options: double-clicking angband.exe, or
# ./angband) uses lib/user, lib/save, lib/scores, lib/panic and
# lib/user/archive (DEFAULT_LIB_PATH ./lib/). The engine creates them only on
# Unix (create_needed_dirs), so on Windows a save failed (the owner, on
# v0.1.0-alpha.1): ship them, empty, on every platform; drop what games in the
# build tree may have left there; and give lib/user the shipped settings, so
# that game plays the borg-rs core as the launchers' do (#826).
rm -rf "$STAGE/lib/save" "$STAGE/lib/scores" "$STAGE/lib/panic" "$STAGE/lib/user"
mkdir -p "$STAGE/lib/save" "$STAGE/lib/scores" "$STAGE/lib/panic" "$STAGE/lib/user/archive"
"$ROOT/harness/prepare-borg-txt.sh" "$STAGE/lib/user"

# The short guide. The game is started directly (angband, angband.exe): the
# owner, for v0.1.0-alpha.1, ships no launcher and no headless runner (lib/
# above holds what the launcher used to set up; CI plays the package with
# scripts/release/play-headless.sh and windows/play-headless.ps1, which take
# the package folder as BORG_PACKAGE_DIR).
if [ "$PLATFORM" = windows ]; then
    # CRLF, as Notepad expects.
    python3 -c 'import sys; d=open(sys.argv[1],"rb").read().replace(b"\r\n",b"\n"); open(sys.argv[2],"wb").write(d.replace(b"\n",b"\r\n"))' \
        "$ROOT/scripts/release/PLAYING.txt" "$STAGE/PLAYING.txt"
else
    cp "$ROOT/scripts/release/PLAYING.txt" "$STAGE/PLAYING.txt"
fi
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
# Every directory, so that empty ones (lib/save, ...) are in the archive too.
dirs = set()
for f in files:
    d = os.path.dirname(f)
    while d:
        dirs.add(d); d = os.path.dirname(d)
dirs |= {os.path.relpath(os.path.join(r, x), stage) for r, ds, _ in os.walk(stage) for x in ds}
dirs = sorted(dirs)
def mode(f):
    return 0o755 if os.stat(os.path.join(stage, f)).st_mode & stat.S_IXUSR else 0o644
if platform == 'windows':
    path = os.path.join(out, name + '.zip')
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        for d in [''] + dirs:
            info = zipfile.ZipInfo(name + '/' + (d.replace(os.sep, '/') + '/' if d else ''), (1980, 1, 1, 0, 0, 0))
            info.external_attr = (0o40755 << 16) | 0x10
            z.writestr(info, b'')
        for f in files:
            info = zipfile.ZipInfo(name + '/' + f.replace(os.sep, '/'), (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = mode(f) << 16
            z.writestr(info, open(os.path.join(stage, f), 'rb').read())
else:
    path = os.path.join(out, name + '.tar.gz')
    with tarfile.open(path, 'w:gz') as t:
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

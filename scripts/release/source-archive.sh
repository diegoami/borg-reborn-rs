#!/usr/bin/env bash
# The release's source archive (#819).
#   scripts/release/source-archive.sh VERSION OUTDIR
# Writes OUTDIR/borg-reborn-rs-VERSION-src.tar.gz and prints its path: the
# committed tree at HEAD (engine/angband at its upstream commit, our patches,
# frontend and build, borg-core/, the harness, vendor/angband-recorder) plus
# the Rust crates Cargo.lock names (cargo vendor, each with its own licence)
# and a .cargo/config.toml that selects them, so
#     cargo build --offline / BORG_IMPL=rust harness/build.sh
# build it without the network. The other vendor/ checkouts scripts/setup.sh
# fetches (borg-rs, better-borg-c, borg-zoo, ...) are references to read,
# never build inputs (pins.json), and are not included.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
[ $# -eq 2 ] || { sed -n '2,12p' "$0" >&2; exit 2; }
VERSION="$1"; OUT="$(realpath -m "$2")"
[[ "$VERSION" =~ ^[A-Za-z0-9._-]+$ ]] || { echo "source-archive.sh: bad version '$VERSION'" >&2; exit 2; }
[ -z "$(git -C "$ROOT" status --porcelain --untracked-files=no)" ] || {
    echo "source-archive.sh: the checkout has uncommitted changes; the archive is HEAD" >&2; exit 2; }
NAME="borg-reborn-rs-$VERSION-src"
STAGE="$OUT/$NAME"
rm -rf "$STAGE"; mkdir -p "$STAGE"
git -C "$ROOT" archive --format=tar HEAD | tar -C "$STAGE" -xf -
(cd "$STAGE" && cargo vendor --locked --versioned-dirs cargo-vendor > cargo-vendor.config.toml 2> "$OUT/cargo-vendor.log") || {
    cat "$OUT/cargo-vendor.log" >&2; exit 1; }
mkdir -p "$STAGE/.cargo"
mv "$STAGE/cargo-vendor.config.toml" "$STAGE/.cargo/config.toml"
# The vendored crates build the same Cargo.lock offline.
(cd "$STAGE" && cargo metadata --offline --locked --format-version 1 >/dev/null)
{
    echo "borg-reborn-rs $VERSION source"
    echo "commit: $(git -C "$ROOT" rev-parse HEAD)"
    echo "engine: upstream Angband $(sed -n 's/^commit=//p' "$ROOT/engine/UPSTREAM") (engine/angband/, unchanged)"
    echo "borg-core: borg-rs $(head -1 "$ROOT/borg-core/PIN") (vendored, docs/architecture/borg-core.md)"
    echo "cargo-vendor/: the crates of Cargo.lock, selected by .cargo/config.toml"
    echo
    echo "Build (cmake, a C compiler, Rust, libssl-dev; X11 also libx11-dev):"
    echo "  cmake -S engine -B build -DCMAKE_BUILD_TYPE=Release [-DSUPPORT_X11=ON]"
    echo "  make -C build -j\$(nproc)          # build/game/angband and build/game/lib"
    echo "harness/build.sh, and the harness, read their inputs with git: run"
    echo "  git init -q && git add -A && git commit -qm source"
    echo "first, then BORG_IMPL=rust harness/build.sh (BUILD_ID's git_rev is then that"
    echo "commit, not the release's; source_sha256 is the release's)."
    echo
    echo "Licences: LICENSE (GPL-2.0), engine/angband/docs/copying.rst,"
    echo "cargo-vendor/*/LICENSE*."
} > "$STAGE/SOURCE.txt"
TAR="$OUT/$NAME.tar.gz"
tar -C "$OUT" --owner=0 --group=0 --numeric-owner --sort=name -czf "$TAR" "$NAME"
rm -rf "$STAGE"
echo "$TAR"

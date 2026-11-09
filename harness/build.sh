#!/usr/bin/env bash
# Build the Rust Borg on our own integration (docs/SPEC.md §3.6; task TE.4).
#   BORG_IMPL=rust harness/build.sh       -> build/rust/game/angband
#   BORG_TEST_FEATURES=F harness/build.sh -> build/rust-test/ (test builds)
# The zoo calls this script by name and reads build/<impl>/game/angband and
# build/<impl>/BUILD_ID (docs/ZOO-INTERFACE.md). BORG_IMPL defaults to rust,
# the only implementation built here (docs/SPEC.md §6). Prints the binary.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMPL="${BORG_IMPL:-rust}"
[ "$IMPL" = rust ] || { echo "build.sh: BORG_IMPL must be rust (docs/SPEC.md §6), not '$IMPL'" >&2; exit 2; }
BUILD="$ROOT/build/$IMPL"
TEST_FEATURES="${BORG_TEST_FEATURES:-}"
[ -z "$TEST_FEATURES" ] || BUILD="$ROOT/build/$IMPL-test"
mkdir -p "$ROOT/build"
ID="$ROOT/harness/identity.sh"
rm -f "$BUILD/BUILD_ID"
# A build directory configured from another source (borg-rs's tree before
# TE.4) is replaced, not reused.
if [ -f "$BUILD/CMakeCache.txt" ] && ! grep -qx "CMAKE_HOME_DIRECTORY:INTERNAL=$ROOT/engine" "$BUILD/CMakeCache.txt"; then
    rm -rf "$BUILD"
fi
SRC_BEFORE=$("$ID" source-hash "$IMPL")
cmake -S "$ROOT/engine" -B "$BUILD" -DCMAKE_BUILD_TYPE=Release -DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
    -DBORG_FEATURES="$TEST_FEATURES" -DSUPPORT_X11="${BORG_X11:-OFF}" >"$BUILD.cmake.log" 2>&1 || {
    cat "$BUILD.cmake.log"; exit 1; }
make -C "$BUILD" -j"$(nproc)" >"$BUILD.make.log" 2>&1 || {
    tail -50 "$BUILD.make.log"; exit 1; }
BIN="$BUILD/game/angband"
SRC_AFTER=$("$ID" source-hash "$IMPL")
if [ "$SRC_BEFORE" != "$SRC_AFTER" ]; then
    echo "build inputs changed during the build; no BUILD_ID written" >&2; exit 1
fi
ELF_IMPL=$("$ID" binary-impl "$BIN")
if [ "$ELF_IMPL" != rust ]; then
    echo "$BIN reads as implementation '$ELF_IMPL', expected 'rust'" >&2; exit 1
fi
if [ -n "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ]; then DIRTY=yes; else DIRTY=no; fi
{
    echo "impl=$IMPL"
    echo "git_rev=$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "git_dirty=$DIRTY"
    echo "inputs_dirty=$("$ID" inputs-dirty "$IMPL")"
    echo "source_sha256=$SRC_AFTER"
    echo "engine_upstream=$(sed -n 's/^commit=//p' "$ROOT/engine/UPSTREAM")"
    echo "binary=$BIN"
    echo "binary_sha256=$(sha256sum < "$BIN" | cut -d' ' -f1)"
    echo "cc=$(cc --version 2>/dev/null | head -1)"
    echo "rustc=$(rustc --version 2>/dev/null || echo unknown)"
    echo "built_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    [ -z "$TEST_FEATURES" ] || echo "test_features=$TEST_FEATURES"
} > "$BUILD/BUILD_ID.tmp"
mv "$BUILD/BUILD_ID.tmp" "$BUILD/BUILD_ID"
echo "$BIN"

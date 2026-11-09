#!/usr/bin/env bash
# Build the harness binary.
#   BORG_IMPL=c    (default) engine + C Borg            -> build/c
#   BORG_IMPL=rust engine + Rust Borg staticlib         -> build/rust
# Usage: harness/build.sh            (uses $BORG_IMPL)
# After a successful build it writes build/<impl>/BUILD_ID (key=value
# lines): the implementation, the git revision and dirty flags, the sha256
# of the build inputs (harness/identity.sh source-hash) and the sha256 of
# the linked binary. harness/suite.sh refuses a binary that does not match
# its BUILD_ID, or whose BUILD_ID does not match the current checkout. A
# failed build leaves no BUILD_ID.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMPL="${BORG_IMPL:-c}"
BUILD="$ROOT/build/$IMPL"
mkdir -p "$ROOT/build"
case "$IMPL" in c|rust) ;; *) echo "BORG_IMPL must be c or rust" >&2; exit 2;; esac
ID="$ROOT/harness/identity.sh"
rm -f "$BUILD/BUILD_ID"
SRC_BEFORE=$("$ID" source-hash "$IMPL")
# Always pass BORG_IMPL: it is a CMake cache variable, so a build dir once
# configured with another value would otherwise keep it.
cmake -S "$ROOT/angband" -B "$BUILD" -DCMAKE_BUILD_TYPE=Release \
    -DSUPPORT_BORG_HARNESS=ON -DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
    -DBORG_IMPL="$IMPL" >"$BUILD.cmake.log" 2>&1 || {
    cat "$BUILD.cmake.log"; exit 1; }
make -C "$BUILD" -j"$(nproc)" >"$BUILD.make.log" 2>&1 || {
    tail -50 "$BUILD.make.log"; exit 1; }
BIN="$BUILD/game/angband"

# Build identity
SRC_AFTER=$("$ID" source-hash "$IMPL")
if [ "$SRC_BEFORE" != "$SRC_AFTER" ]; then
    echo "build inputs changed during the build; no BUILD_ID written" >&2; exit 1
fi
ELF_IMPL=$("$ID" binary-impl "$BIN")
if [ "$ELF_IMPL" != "$IMPL" ]; then
    echo "$BIN reads as implementation '$ELF_IMPL', expected '$IMPL'" >&2; exit 1
fi
if [ -n "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ]; then DIRTY=yes; else DIRTY=no; fi
{
    echo "impl=$IMPL"
    echo "git_rev=$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "git_dirty=$DIRTY"
    echo "inputs_dirty=$("$ID" inputs-dirty "$IMPL")"
    echo "source_sha256=$SRC_AFTER"
    echo "binary=$BIN"
    echo "binary_sha256=$(sha256sum < "$BIN" | cut -d' ' -f1)"
    echo "cc=$(cc --version 2>/dev/null | head -1)"
    if [ "$IMPL" = rust ]; then echo "rustc=$(rustc --version 2>/dev/null || echo unknown)"; fi
    echo "built_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$BUILD/BUILD_ID.tmp"
mv "$BUILD/BUILD_ID.tmp" "$BUILD/BUILD_ID"
echo "$BIN"

#!/usr/bin/env bash
# Build identity helpers, used by harness/build.sh and harness/suite.sh.
# Usage:
#   harness/identity.sh inputs IMPL       list the build inputs of IMPL
#   harness/identity.sh source-hash IMPL  sha256 over the build inputs of IMPL
#   harness/identity.sh inputs-dirty IMPL yes/no: inputs differ from HEAD
#   harness/identity.sh binary-impl BIN   c, rust or unknown, read from BIN
#
# Build inputs (paths relative to the checkout root), docs/SPEC.md §3.6:
#   rust  engine/angband/ engine/patches/ engine/frontend/ engine/CMakeLists.txt
#         engine/sources.cmake engine/apply-patches.sh engine/UPSTREAM
#         borg-reborn-rs/ borg-core/ Cargo.toml Cargo.lock
# They are the files git tracks under those paths plus the untracked files
# that .gitignore does not exclude, as they are in the working tree (not as
# committed): source-hash is sha256 over "<git blob id> <path>" lines, so
# it depends only on file contents and names. It does not change when a
# change is committed, nor when files outside the inputs change.
#
# binary-impl reads the executable itself: "rust" if its .comment section
# names rustc (the Rust staticlib is linked in), "c" if not and it defines
# do_cmd_borg (the C Borg is linked in), "unknown" otherwise (no Borg, a
# stripped C binary, not an ELF file, no readelf/nm).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

inputs_of() {
    case "$1" in
        # docs/SPEC.md §3.6: the pristine tree, our patches, frontend and build
        # files, and the Rust Borg (engine/tests/ are not build inputs).
        rust) echo "engine/angband engine/patches engine/frontend engine/CMakeLists.txt engine/sources.cmake engine/apply-patches.sh engine/UPSTREAM borg-reborn-rs borg-core Cargo.toml Cargo.lock";;
        *) echo "identity.sh: the only implementation is rust, not '$1' (docs/SPEC.md §6)" >&2; exit 2;;
    esac
}

list_inputs() {
    local paths; paths=$(inputs_of "$1")
    # shellcheck disable=SC2086
    git -C "$ROOT" -c core.quotepath=off ls-files -z -c -o --exclude-standard \
        -- $paths | sort -zu | while IFS= read -r -d '' f; do
        # a tracked file deleted in the working tree is not an input
        [ -e "$ROOT/$f" ] || [ -L "$ROOT/$f" ] || continue
        printf '%s\0' "$f"
    done
}

case "${1:-}" in
    inputs)
        list_inputs "$2" | tr '\0' '\n';;
    source-hash)
        inputs_of "$2" >/dev/null
        # Shared workspaces can have callers in separate PID namespaces;
        # $$ alone does not give a unique name on the shared filesystem.
        HASH_TMP=$(mktemp -d "${TMPDIR:-/tmp}/borg-input-hash.XXXXXXXX")
        trap 'rm -f "$HASH_TMP/paths" "$HASH_TMP/blobs"; rmdir "$HASH_TMP"' EXIT
        list_inputs "$2" > "$HASH_TMP/paths"
        n=$(tr -cd '\0' < "$HASH_TMP/paths" | wc -c)
        [ "$n" -gt 0 ] || { echo "identity.sh: no inputs for $2 under $ROOT" >&2; exit 1; }
        # Capture this step directly so a failed read cannot be hidden by
        # process substitution and followed by a successful partial hash.
        (cd "$ROOT" && xargs -0 git hash-object -- < "$HASH_TMP/paths") > "$HASH_TMP/blobs"
        paste -d' ' "$HASH_TMP/blobs" <(tr '\0' '\n' < "$HASH_TMP/paths") \
          | { echo "borg-reborn-rs build inputs v2 impl=$2"; cat;
              } \
          | sha256sum | cut -d' ' -f1;;
    inputs-dirty)
        # shellcheck disable=SC2046
        if [ -n "$(git -C "$ROOT" status --porcelain -- $(inputs_of "$2"))" ]; then
            echo yes; else echo no; fi;;
    binary-impl)
        BIN="$2"
        if readelf -p .comment "$BIN" 2>/dev/null | grep -q 'rustc version '; then
            echo rust
        elif nm --defined-only "$BIN" 2>/dev/null | awk '$2=="T" && $3=="do_cmd_borg"{f=1} END{exit !f}'; then
            echo c
        else
            echo unknown
        fi;;
    *)
        sed -n '2,8p' "$0" >&2; exit 2;;
esac

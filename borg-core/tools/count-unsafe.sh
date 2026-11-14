#!/usr/bin/env bash
# Count `unsafe` (fns, blocks, impls) per Rust source file of the borg-rs
# crate, ignoring comments. Prints a table and the total.
# Usage: borg-rs/tools/count-unsafe.sh [--summary]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)/src"
cd "$ROOT"
total=0
rows=""
for f in $(find . -name '*.rs' | sort); do
    n=$(sed -e 's://.*$::' "$f" | { grep -o '\bunsafe\b' || true; } | wc -l)
    total=$((total + n))
    if [ "$n" -gt 0 ]; then rows+="$(printf '%6d  %s' "$n" "${f#./}")"$'\n'; fi
done
if [ "${1:-}" != "--summary" ]; then printf '%s' "$rows" | sort -rn; fi
conv=$(ls *.rs | { grep -v -E '^(lib|bridge)\.rs$' || true; } | wc -l)
left=0
if [ -d c2rust ]; then left=$(ls c2rust/*.rs | { grep -v mod.rs || true; } | wc -l); fi
echo "total unsafe: $total   (c2rust modules left: $left, converted files: $conv)"

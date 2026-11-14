#!/usr/bin/env bash
# Regenerate the c2rust transpilation of angband/src/borg/*.c into
# borg-rs/src/c2rust/ (raw, then post-processed for stable Rust).
# Requires: c2rust 0.22.1 (LLVM 18), a C harness build (harness/build.sh)
# for compile_commands.json.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="$(mktemp -d)"
python3 - "$ROOT/build/c/compile_commands.json" "$WORK/compile_commands.json" <<'PY'
import json, re, sys
d = json.load(open(sys.argv[1]))
b = [x for x in d if '/src/borg/' in x['file']]
for x in b:
    # drop optimisation and the git-derived build id (not used by the Borg)
    x['command'] = re.sub(r' -DBUILD_ID=\S*| -O\d', '', x['command'])
json.dump(b, open(sys.argv[2], 'w'), indent=1)
PY
# -r: keep header declarations in per-header modules (with source
# locations) so tools/reorganize.py can deduplicate them across files
(cd "$WORK" && c2rust transpile compile_commands.json -o "$WORK/out" \
    -r --disable-refactoring --overwrite-existing >/dev/null)
rm -rf "$ROOT/borg-rs/src/c2rust" "$ROOT/borg-rs/src/headers"
python3 "$ROOT/borg-rs/tools/reorganize.py" "$WORK/out/src" "$ROOT/borg-rs/src" \
    2> "$WORK/reorganize.log" || { cat "$WORK/reorganize.log"; exit 1; }
rustfmt --edition 2021 "$ROOT/borg-rs/src/headers/mod.rs" "$ROOT/borg-rs/src/c2rust/mod.rs"
rm -rf "$WORK"
echo "transpiled into $ROOT/borg-rs/src/{c2rust,headers}"

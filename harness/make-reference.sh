#!/usr/bin/env bash
# Make a regression reference from a suite output (docs/ENGINE.md, References).
#   harness/make-reference.sh SUITE_OUTDIR REFDIR
# Per run: keys.log, messages.log and state.log (xz) and summary.json, the
# files harness/diff compares; and REFDIR/REFERENCE with what made it (the
# commit, the build's source and binary hashes, the run configuration hashes).
# REFDIR must not exist. A reference is regenerated only in a PR that says why.
set -euo pipefail
OUT="$1"; REF="$2"
[ -f "$OUT/DONE" ] || { echo "make-reference: $OUT is not a finished suite output" >&2; exit 2; }
[ ! -e "$REF" ] || { echo "make-reference: $REF exists" >&2; exit 2; }
mkdir -p "$REF"
for run in "$OUT"/*/; do
    name=$(basename "$run")
    [ -f "$run/summary.json" ] || continue
    mkdir "$REF/$name"
    for log in keys messages state; do xz -9e -c "$run/$log.log" > "$REF/$name/$log.log.xz"; done
    cp "$run/summary.json" "$REF/$name/summary.json"
done
first=$(ls -d "$OUT"/*/ | head -1)
{
    echo "suite=$(sed -n 's/^suite=//p' "$OUT/manifest")"
    echo "suite_sha256=$(sed -n 's/^suite_sha256=//p' "$OUT/manifest")"
    echo "git_rev=$(git -C "$(dirname "$0")/.." rev-parse HEAD)"
    sed -n 's/^\(source_sha256\|binary_sha256\|engine_upstream\)=/\1=/p' "$(dirname "$0")/../build/rust/BUILD_ID"
    grep -E '^(borg_txt_sha256|profile|profile_sha256)=' "$first/config.txt"
    echo "environment=TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8"
} > "$REF/REFERENCE"
echo "$REF: $(ls -d "$REF"/*/ | wc -l) runs"

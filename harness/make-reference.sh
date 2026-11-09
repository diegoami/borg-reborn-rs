#!/usr/bin/env bash
# Make a regression reference from a suite output (docs/ENGINE.md, References).
#   harness/make-reference.sh SUITE_OUTDIR REFDIR
# Per run: keys.log, messages.log and state.log (xz) and summary.json, the
# files harness/diff compares; and REFDIR/REFERENCE with what made it (the
# build's commit, source and binary hashes and each run's configuration, from
# the suite's manifest, and the profile). The build must come from committed
# inputs, and the suite must have run without overrides.
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
# Provenance from the suite output itself, never from today's checkout or
# build (E review, finding 8): the manifest's build identity and each run's
# configuration (its flags and borg.txt, which differ per run in the respawn
# lines). The profile must be the same for every run.
m() { sed -n "s/^$1=//p" "$OUT/manifest" | head -1; }
[ "$(m overrides)" = none ] || { echo "make-reference: $OUT ran with overrides $(m overrides)" >&2; exit 2; }
[ "$(m build_id_inputs_dirty)" = no ] || {
    echo "make-reference: $OUT was built from uncommitted build inputs; commit, rebuild and rerun" >&2; exit 2; }
profiles=$(for run in "$OUT"/*/; do [ -f "$run/summary.json" ] || continue
    grep -E '^(profile|profile_sha256)=' "$run/config.txt" | sort | tr '\n' ' '; echo; done | sort -u)
[ "$(printf '%s\n' "$profiles" | wc -l)" = 1 ] || {
    echo "make-reference: the runs of $OUT differ in profile:" >&2; printf '%s\n' "$profiles" >&2; exit 2; }
{
    echo "suite=$(m suite)"
    echo "suite_sha256=$(m suite_sha256)"
    echo "git_rev=$(m build_id_git_rev)"
    echo "source_sha256=$(m build_id_source_sha256)"
    echo "engine_upstream=$(m build_id_engine_upstream)"
    echo "binary_sha256=$(m binary_sha256)"
    printf '%s\n' $profiles
    grep '^run_config\.' "$OUT/manifest"
    echo "environment=TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8"
} > "$REF/REFERENCE"
echo "$REF: $(ls -d "$REF"/*/ | wc -l) runs"

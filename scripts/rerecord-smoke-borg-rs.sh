#!/usr/bin/env bash
# Re-record harness/reference/smoke-borg-rs after a change to how the default
# (borg-rs) core plays, such as a shipped value in harness/borg.txt (AGENTS.md,
# "Which core a run plays").
#
#   1. edit harness/borg.txt (or borg-core/src/) and commit it;
#   2. scripts/rerecord-smoke-borg-rs.sh
#
# It builds the committed tree, plays the smoke suite on the default core,
# compares it with the old reference (for the record), replaces the reference
# and commits it on its own. Nothing is pushed. Run it from the top of a
# worktree with a clean working tree: harness/make-reference.sh refuses a
# build from uncommitted inputs (untracked files are not checked here; that
# refusal is the guarantee).
#
#   --no-commit   leave the new reference staged instead of committing it
#   OUT           suite output directory (default: a fresh one under /tmp)
#
# BORG_CEMETERY_DIR, if set, is passed to the suite as usual.
set -euo pipefail
commit=1
OUT=""
for a in "$@"; do
    case "$a" in
        --no-commit) commit=0 ;;
        -h|--help) sed -n '2,/^set -e/{/^set -e/d;p}' "$0"; exit 0 ;;
        *) OUT="$a" ;;
    esac
done
cd "$(git rev-parse --show-toplevel)"
REF=harness/reference/smoke-borg-rs
LOCK=/tmp/angband-validation.lock
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
    echo "rerecord: commit your changes first (git status shows changes)" >&2
    exit 2
fi
if [ -n "${BORG_CORE:-}" ] && [ "${BORG_CORE}" != borg-rs ]; then
    echo "rerecord: BORG_CORE=$BORG_CORE; this reference is the default borg-rs core's" >&2
    exit 2
fi
# CI plays harness/borg.txt; BORG_CORE_OVERRIDES=off would play
# borg-core/borg.txt instead and record another game (review of #633).
if [ "${BORG_CORE_OVERRIDES:-on}" != on ]; then
    echo "rerecord: BORG_CORE_OVERRIDES=$BORG_CORE_OVERRIDES changes the borg.txt the run reads; unset it" >&2
    exit 2
fi
if [ -z "$OUT" ]; then
    OUT=$(mktemp -d /tmp/smoke-borg-rs.XXXXXX)
    rmdir "$OUT"
fi
[ ! -e "$OUT" ] || { echo "rerecord: $OUT exists" >&2; exit 2; }

echo "== build $(git rev-parse --short HEAD)"
BORG_IMPL=rust flock "$LOCK" harness/build.sh
echo "== smoke -> $OUT"
flock "$LOCK" harness/suite.sh smoke "$OUT"

echo "== against the old reference"
if harness/diff --suite smoke --expect-impl rust "$OUT" "$REF" > "$OUT/old-diff.txt" 2>&1; then
    grep 'runs identical' "$OUT/old-diff.txt" | sed 's/^/  /'
    echo "rerecord: the games are identical to the reference; nothing to re-record"
    exit 0
fi
grep -E 'runs identical' "$OUT/old-diff.txt" | sed 's/^/  /'
echo "  (full diff: $OUT/old-diff.txt)"

echo "== re-record"
# Built beside the old one and swapped in only when complete, so a failed
# re-record leaves the old reference in place.
NEW="$REF.new.$$"
harness/make-reference.sh "$OUT" "$NEW"
git rm -rq "$REF"
rm -rf "$REF"
mv "$NEW" "$REF"
git add "$REF"
harness/diff --suite smoke --expect-impl rust "$OUT" "$REF" | grep 'runs identical' | sed 's/^/  /'

if [ "$commit" = 1 ]; then
    changed=$(git diff --cached --name-only -- "$REF" | wc -l)
    git commit -q -m "Re-record smoke-borg-rs for $(git rev-parse --short HEAD) ($changed files changed)"
    echo "== committed $(git rev-parse --short HEAD); push when ready"
else
    echo "== staged; commit it with your change"
fi

#!/usr/bin/env bash
# Copy the pristine upstream tree and apply our patch series to the copy.
#   engine/apply-patches.sh DEST
# DEST must not exist. engine/angband/ itself is never changed (it stays
# upstream's bytes, docs/ENGINE.md). The patches apply in name order.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$1"
[ ! -e "$DEST" ] || { echo "apply-patches: $DEST exists" >&2; exit 2; }
mkdir -p "$(dirname "$DEST")"
cp -a "$ROOT/engine/angband" "$DEST"
for p in "$ROOT"/engine/patches/*.patch; do
    patch -s -p1 -d "$DEST" --forward --no-backup-if-mismatch < "$p" || {
        echo "apply-patches: $(basename "$p") does not apply" >&2; exit 1; }
done

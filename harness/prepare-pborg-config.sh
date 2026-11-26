#!/usr/bin/env bash
# Copy the shipped PBorg settings and selected scary list into a user directory.
#   harness/prepare-pborg-config.sh DIR
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ $# -eq 1 ] || { sed -n '2,3p' "$0" >&2; exit 2; }
DIR="$1"
mkdir -p "$DIR"
TEMPLATE="$ROOT/harness/pborg.yaml"
SCARY_NAME="$(python3 "$ROOT/harness/pborg-settings.py" "$TEMPLATE" get borg_scary_list)"
if [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ &&
       "$SCARY_NAME" != pborg.yaml && "$SCARY_NAME" != borg.txt ]] && [ -f "$ROOT/harness/$SCARY_NAME" ] || {
        echo "prepare-pborg-config.sh: scary list '$SCARY_NAME' not in harness/" >&2; exit 1; }
    cp "$ROOT/harness/$SCARY_NAME" "$DIR/$SCARY_NAME"
fi
cp "$TEMPLATE" "$DIR/pborg.yaml"

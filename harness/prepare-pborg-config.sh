#!/usr/bin/env bash
# Write the Borg's settings for a game's user directory (#826).
#   harness/prepare-pborg-config.sh DIR
# Writes DIR/pborg.yaml and the scary list it selects, as harness/run.sh
# prepares a run's user/pborg.yaml before it sets the run's race and class:
# harness/borg.txt (class files folded in) with the world key removed (none:
# upstream's data) and the scary list's selecting line moved to the end
# under its SHA-256, the list copied beside it. Used by watch-borg.sh,
# harness/play.sh and the release package (scripts/release/package.sh), so a
# watched game plays the shipped settings on the borg-rs core, as runs do.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ $# -eq 1 ] || { sed -n '2,10p' "$0" >&2; exit 2; }
DIR="$1"
mkdir -p "$DIR"
sha() { python3 -c 'import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"; }
BT="$DIR/pborg.yaml"
TMP="$BT.tmp.$$"
sed '/^[[:space:]]*borg_world[[:space:]]*=/d' "$ROOT/harness/borg.txt" > "$TMP"
SCARY_NAME="$(sed -n 's/^[[:space:]]*borg_scary_list[[:space:]]*=[[:space:]]*//p' "$TMP" | tail -1 | tr -d '[:space:]')"
if [ -n "$SCARY_NAME" ] && [ "$SCARY_NAME" != none ]; then
    [[ "$SCARY_NAME" =~ ^[A-Za-z0-9_.-]+$ ]] && [ -f "$ROOT/harness/$SCARY_NAME" ] || {
        rm -f "$TMP"; echo "prepare-pborg-config.sh: scary list '$SCARY_NAME' not in harness/" >&2; exit 1; }
    cp "$ROOT/harness/$SCARY_NAME" "$DIR/$SCARY_NAME"
    sed '/^[[:space:]]*borg_scary_list[[:space:]]*=/d' "$TMP" > "$TMP.2" && mv "$TMP.2" "$TMP"
    printf '\n# harness/run.sh (#518): scary list sha256 %s\nborg_scary_list = %s\n' \
        "$(sha "$ROOT/harness/$SCARY_NAME")" "$SCARY_NAME" >> "$TMP"
fi
python3 "$ROOT/harness/pborg-config.py" "$TMP" > "$TMP.2"
mv "$TMP.2" "$BT"

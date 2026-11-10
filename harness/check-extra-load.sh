#!/usr/bin/env bash
# A save loaded through run.sh's extra borgh options plays like save=NAME
# (M1 re-review S1, S2).
#   harness/check-extra-load.sh [WORKDIR]
# Plays a short living run that writes final.save, then loads that save
# twice: as save=final.save,keep-load-rng (SAVE_ROOT) and, from another
# directory and by a relative path, as `-- -load FILE -keep-load-rng`. Both
# must resume the saved game at its turn, play Borg keys, record the save's
# hash, and play the same keys. Uses BORG_IMPL (default and only: rust) like run.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(realpath -m "${1:-$ROOT/harness/out/check-extra-load}")"
[ ! -e "$WORK" ] || { echo "check-extra-load: $WORK exists; use a new directory" >&2; exit 2; }
mkdir -p "$WORK"
"$ROOT/harness/run.sh" "$WORK/first" 101 0 0 1000 finalsave >/dev/null
[ -s "$WORK/first/final.save" ] || { echo "check-extra-load: no final.save (did the character die?)" >&2; exit 1; }
SAVE_ROOT="$WORK/first" "$ROOT/harness/run.sh" "$WORK/flag" 7 0 0 2000 save=final.save,keep-load-rng >/dev/null
mkdir "$WORK/elsewhere"
(cd "$WORK/elsewhere" && "$ROOT/harness/run.sh" "$WORK/extra" 7 0 0 2000 - -- -load ../first/final.save -keep-load-rng >/dev/null)
python3 - "$WORK" <<'EOF'
import hashlib, json, sys
from pathlib import Path
w = Path(sys.argv[1])
first = json.loads((w / 'first/summary.json').read_text())
want_save = hashlib.sha256((w / 'first/final.save').read_bytes()).hexdigest()
bad = []
for name in ('flag', 'extra'):
    s = json.loads((w / name / 'summary.json').read_text())
    config = dict(l.split('=', 1) for l in (w / name / 'config.txt').read_text().splitlines() if '=' in l)
    keys = (w / name / 'keys.log').read_bytes()
    if s.get('loaded') is not True or s.get('loaded_turn') != first['game_turns']:
        bad.append(f"{name}: loaded {s.get('loaded')} at turn {s.get('loaded_turn')}, the save is at {first['game_turns']}")
    if not s.get('game_turns', 0) > first['game_turns'] or not keys:
        bad.append(f"{name}: did not play on (turn {s.get('game_turns')}, {len(keys.splitlines())} keys)")
    if config.get('save_sha256') != want_save:
        bad.append(f'{name}: config.txt save_sha256 {config.get("save_sha256")} is not the save\'s')
if first.get('loaded') is not False:
    bad.append(f"first: a fresh birth says loaded {first.get('loaded')!r}")
if (w / 'flag/keys.log').read_bytes() != (w / 'extra/keys.log').read_bytes():
    bad.append('the extra-option load played other keys than save=NAME')
for b in bad:
    print('FAIL:', b)
print('extra-option load: ' + ('FAILED' if bad else f"ok, resumed at turn {first['game_turns']} like save=NAME"))
sys.exit(1 if bad else 0)
EOF

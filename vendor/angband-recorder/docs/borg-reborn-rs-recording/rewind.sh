#!/usr/bin/env bash
# Go back to the turns before a recorded game's end:  rewind.sh RECORDING [PLAYER_TURNS] [--open]
#   RECORDING     e.g. ~/angband-recordings/NAME
#   PLAYER_TURNS  how far back to start the view (default 50; 1 player turn = 10 game turns)
#   --open        then open the last save before that point in an X11 window (open-save.sh)
# Plays the recorded screens from that point to the end (death or stop), slowly, in the terminal,
# and names the nearest earlier checkpoint save. The opened save is a copy; the Borg in it starts
# with fresh memory, so playing on from there is not guaranteed to end the same way.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -ge 1 ] || { echo "usage: rewind.sh RECORDING [PLAYER_TURNS] [--open]" >&2; exit 2; }
REC="$(realpath "$1")"; shift
BACK=50 OPEN=0
for arg in "$@"; do
    case "$arg" in
        --open) OPEN=1 ;;
        *) [[ "$arg" =~ ^[0-9]+$ ]] || { echo "rewind.sh: bad PLAYER_TURNS '$arg'" >&2; exit 2; }; BACK=$arg ;;
    esac
done
read -r END FROM SAVE < <(python3 - "$REC" "$BACK" <<'EOF'
import json, re, sys
from pathlib import Path
rec, back = Path(sys.argv[1]), int(sys.argv[2])
end = json.load(open(rec / 'summary.json'))['game_turns']
start = max(0, end - 10 * back)
best = ''
for f in sorted((rec / 'checkpoints').glob('checkpoint-*-t*.save')):
    t = int(re.search(r'-t(\d+)\.save$', f.name).group(1))
    if t <= start:
        best = str(f)
print(end, start, best or '-')
EOF
)
echo "rewind.sh: the game ended at turn $END; showing from turn $FROM ($BACK player turns back)."
sleep 1
"$HERE/watch-recording.py" "$REC/native" --from-turn "$FROM" --delay "${VIEW_DELAY:-0.3}"
python3 -c "import json,sys;d=json.load(open(sys.argv[1]));print('rewind.sh: end:', d['stop_reason'], ('- killed by ' + d['death_cause']) if d.get('dead') else '')" "$REC/summary.json"
if [ "$SAVE" = - ]; then
    echo "rewind.sh: no checkpoint save at or before turn $FROM"
    exit 0
fi
echo "rewind.sh: nearest earlier save: $SAVE"
if [ "$OPEN" = 1 ]; then
    exec "$HERE/open-save.sh" "$SAVE"
else
    echo "rewind.sh: to play on from it in a window: $HERE/open-save.sh $SAVE"
fi

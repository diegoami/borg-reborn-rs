#!/usr/bin/env bash
# Record a game and watch it live:  record.sh NAME SEED RACE CLASS [TURNS]
#   NAME   recording directory under ~/angband-recordings (must not exist)
#   SEED   a number; RACE, CLASS: menu indices (README.md); TURNS: game turns (default 1000000)
# Starts harness/run.sh in the background (headless, recorder on, a save every CHECKPOINT_EVERY
# game turns) and shows the screens live with watch-recording.py --follow. Ctrl-C closes the view
# only: the game keeps running and its log is NAME.run.log. Set NO_VIEW=1 to skip the view.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -ge 4 ] && [ $# -le 5 ] || { echo "usage: record.sh NAME SEED RACE CLASS [TURNS]" >&2; exit 2; }
NAME=$1 SEED=$2 RACE=$3 CLASS=$4 TURNS=${5:-1000000}
[[ "$SEED" =~ ^[0-9]+$ ]] || { echo "record.sh: SEED must be a number, not '$SEED'" >&2; exit 2; }
OUT="$HOME/angband-recordings/$NAME"
[ ! -e "$OUT" ] || { echo "record.sh: $OUT exists" >&2; exit 2; }
source "$HERE/run-env.sh"
mkdir -p "$HOME/angband-recordings"
LOG="$HOME/angband-recordings/$NAME.run.log"
# Its own session, so Ctrl-C in the view never reaches the game.
cd "$BRR_BUILD"
setsid bash -c 'flock /tmp/angband-validation.lock harness/run.sh "$@"; echo "run.sh exit $?"' run \
    "$OUT" "$SEED" "$RACE" "$CLASS" "$TURNS" menu,terminal-save > "$LOG" 2>&1 < /dev/null &
GAME=$!
echo "record.sh: recording into $OUT (log $LOG); waiting for the lock and the first frames..."
if [ "${NO_VIEW:-0}" != 1 ]; then
    "$HERE/watch-recording.py" "$OUT/native" --follow --delay "${VIEW_DELAY:-0.02}" || true
fi
if [ ! -e "$OUT/native/complete.json" ] && kill -0 "$GAME" 2>/dev/null; then
    echo "record.sh: the game is still running (pid $GAME); see $LOG. Results will be in $OUT."
    exit 0
fi
echo "record.sh: recording complete; waiting for run.sh to finish..."
wait "$GAME" || true
python3 - "$OUT/summary.json" <<'EOF'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except OSError:
    sys.exit(f"record.sh: no summary.json; see the run log")
print(f"record.sh: {d['race']} {d['class']} stopped by {d['stop_reason']} at turn {d['game_turns']}: "
      f"level {d['clevel']}, max depth {d['max_depth']}"
      + (f", killed by {d['death_cause']}" if d.get('dead') else ", alive"))
EOF

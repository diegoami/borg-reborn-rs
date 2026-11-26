#!/usr/bin/env bash
# Replay a recording and compare it:  replay.sh RECORDING_DIR OUT_DIR
# Runs the recording's own command (from RECORDING_DIR/process.json) with its paths moved to
# OUT_DIR, feeding the recorded inputs back, and compares keys, state log and final save.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -eq 2 ] || { echo "usage: replay.sh RECORDING_DIR OUT_DIR" >&2; exit 2; }
D="$(realpath "$1")"; R="$(realpath -m "$2")"
[ ! -e "$R" ] || { echo "replay.sh: $R exists" >&2; exit 2; }
source "$HERE/run-env.sh"
mkdir -p "$R"/{user,save,archive,panic,scores}
cp "$D/user/borg.txt" "$R/user/"
if [ -f "$D/user/errands_plan.txt" ]; then cp "$D/user/errands_plan.txt" "$R/user/"; fi
if [ -d "$D/checkpoints" ]; then mkdir -p "$R/checkpoints"; fi
# The original command, with the recording's directory replaced by the output directory.
mapfile -d '' CMD < <(python3 - "$D" "$R" <<'EOF'
import json, sys
d, r = sys.argv[1], sys.argv[2]
cmd = json.load(open(d + '/process.json'))['command']
sys.stdout.write(''.join(a.replace(d, r) + '\0' for a in cmd))
EOF
)
if [ ! -x "${CMD[0]}" ]; then
    # The build may have moved since (same bytes); the recorder checks the executable's hash anyway.
    echo "replay.sh: ${CMD[0]} is gone; using $ANGBAND_BIN" >&2
    CMD[0]="$ANGBAND_BIN"
fi
cd "$(dirname "${CMD[0]}")"
# BORG_ERRANDS_PLAN stays unset: run.sh does not pass it to the game, which reads the plan from borg.txt.
flock /tmp/angband-validation.lock env -u BORG_RECORD_DIR -u BORG_TOOLKIT_NATIVE -u BORG_ERRANDS_PLAN \
  -u CHECKPOINT_EVERY -u ANGBAND_BIN \
  ANGBAND_REPLAY_DIR="$D/native" ANGBAND_RECORD_DIR="$R/native" BORG_TOOLKIT_HEADLESS=1 BORG_RUN_DIR="$R" \
  BORG_PROFILE="$BRR_BUILD/harness/profiles/reborn-default.txt" BORG_PROFILE_NAME=harness/profiles/reborn-default.txt \
  TZ=UTC LANG=C.UTF-8 LC_ALL=C.UTF-8 "${CMD[@]}" > "$R/stderr.log" 2>&1 \
  || echo "replay.sh: game exited with rc=$?; see $R/stderr.log"
for f in keys.log state.log final.save; do cmp -s "$D/$f" "$R/$f" && echo "$f identical" || echo "$f DIFFERS"; done

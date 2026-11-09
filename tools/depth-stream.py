#!/usr/bin/env python3
"""Derive the depth stream of a harness run (task T1.4, issue #7).

  tools/depth-stream.py RUN_DIR --cap TURNS

Reads RUN_DIR/state.log (one engine-state line per Borg key, written by
borgh: "<game turn> <x>,<y> d<depth>/<max depth> hp..."), and
RUN_DIR/summary.json, and writes RUN_DIR/depths.json:

  boundary            "exact" if the run stopped at engine turn TURNS itself
                      (run.sh flag exact, borgh -exact-turns), else "after-key"
  cap                 TURNS
  stop_turn, alive, max_depth_at_stop   from summary.json
  first_turn_at_depth {depth: engine turn of the first key at which the
                      maximum depth was at least that depth}
  granularity         the first Borg key after arrival; a level change takes
                      effect when the Borg's next key is logged

The primary endpoint (GOALS.md) is read from it: success means alive at the
boundary with max_depth_at_stop >= D (or a victory before TURNS).
"""

import argparse
import json
import re
import sys
from pathlib import Path

STATE = re.compile(r'^(\d+) \S+ d(\d+)/(\d+) ')


def stream(state_lines):
    first = {}
    best = 0
    for line in state_lines:
        m = STATE.match(line)
        if not m:
            continue
        turn, max_depth = int(m.group(1)), int(m.group(3))
        while best < max_depth:
            best += 1
            first.setdefault(best, turn)
    return first


def derive(run_dir, cap):
    run_dir = Path(run_dir)
    summary = json.loads((run_dir / 'summary.json').read_text())
    state = run_dir / 'state.log'
    first = stream(state.read_text(errors='replace').splitlines()) if state.exists() else {}
    # The final state (summary) may show a depth no key was logged at.
    final_max = summary.get('max_depth') or 0
    stop_turn = summary.get('game_turns')
    for d in range(1, final_max + 1):
        first.setdefault(d, stop_turn)
    exact = bool(summary.get('exact_turns'))
    out = {
        'schema': 'borg-reborn-rs/depths@1',
        'boundary': 'exact' if exact else 'after-key',
        'cap': cap,
        'stop_turn': stop_turn,
        'stop_reason': summary.get('stop_reason'),
        'alive': not summary.get('dead'),
        'max_depth_at_stop': final_max,
        'first_turn_at_depth': {str(d): first[d] for d in sorted(first)},
        'granularity': 'first Borg key after arrival (state.log); final depth from summary.json',
        'source': 'state.log' if state.exists() else 'summary.json only (no state.log)',
    }
    if exact and out['alive'] and summary.get('stop_reason') == 'turns' and stop_turn != cap:
        raise SystemExit(f'depth-stream: exact boundary but stopped at {stop_turn}, not {cap}')
    (run_dir / 'depths.json').write_text(json.dumps(out, indent=2) + '\n')
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('run_dir')
    ap.add_argument('--cap', type=int, required=True)
    a = ap.parse_args(argv)
    derive(a.run_dir, a.cap)
    return 0


if __name__ == '__main__':
    sys.exit(main())

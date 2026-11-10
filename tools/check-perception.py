#!/usr/bin/env python3
"""Perception against the engine (task T2.5, #16).

  tools/check-perception.py RUN_OR_SUITE_DIR...

For runs of a test-perception build (BORG_TEST_FEATURES=test-perception
harness/build.sh): each think's truth.jsonl line is the Borg's observation
and world compared with the engine through other routes (engine.rs, ENGINE
TRUTH). This tool fails if any think has a difference, and checks the status
against the frontend's own state.log, written by independent C code: every
think's position, depth, HP, SP, food, gold, experience, level and timed
effects appear in a state.log line at the same game turn (the key that think
returned is logged with the state it saw).

Coverage is required (M2 review, finding 4): every run (a directory with
summary.json) of a suite directory must have a truth.jsonl, with exactly one
line per Borg decision (summary.json's borg_decisions) and at least one.

It prints, per run and in total, what was compared and what the Borg does
not see by design (the panel limit, which the C Borg has too).
"""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

STATE = re.compile(r'^(\d+) (-?\d+),(-?\d+) d(-?\d+)/(-?\d+) hp(-?\d+)/(-?\d+) sp(-?\d+)/(-?\d+) f(-?\d+) '
                   r'au(-?\d+) xp(-?\d+) lv(-?\d+) t(\S+) ')
COUNTS = ('panel_grids', 'map_compared', 'map_stale_off_panel', 'map_unknown_to_world', 'monsters_compared',
          'monsters_off_panel', 'gear_compared')


def state_lines(path):
    """{turn: set of status tuples} from state.log."""
    out = defaultdict(set)
    for line in path.read_text().splitlines():
        m = STATE.match(line)
        if not m:
            continue
        v = [int(x) for x in m.groups()[:13]]
        timed = () if m.group(14) == '-' else tuple(tuple(int(n) for n in t.split(':')) for t in m.group(14).split(','))
        out[v[0]].add((*v[1:], timed))
    return out


def think_state(r):
    s = r['state']
    return (s['x'], s['y'], s['depth'], s['max_depth'], s['hp'], s['max_hp'], s['sp'], s['max_sp'], s['food'],
            s['gold'], s['exp'], s['level'], tuple(tuple(t) for t in s['timed']))


def check_run(run):
    truth = run / 'truth.jsonl'
    totals = dict.fromkeys(COUNTS, 0)
    if not truth.exists():
        return 0, totals, ['no truth.jsonl (not a test-perception build, or the log was lost)']
    states = state_lines(run / 'state.log') if (run / 'state.log').exists() else {}
    thinks, problems = 0, []
    for line in truth.read_text().splitlines():
        r = json.loads(line)
        thinks += 1
        for k in COUNTS:
            totals[k] += r[k]
        problems += [f"turn {r['turn']}: {d}" for d in r['differences']]
        if think_state(r) not in states.get(r['turn'], ()):
            problems.append(f"turn {r['turn']}: status {think_state(r)} is in no state.log line of that turn")
    decisions = json.loads((run / 'summary.json').read_text()).get('borg_decisions') if (run / 'summary.json').exists() else None
    if thinks == 0:
        problems.append('truth.jsonl has no think')
    if decisions is None:
        problems.append('no summary.json with borg_decisions: the coverage cannot be checked')
    elif thinks != decisions:
        problems.append(f'{thinks} truth lines for {decisions} Borg decisions')
    return thinks, totals, problems


def main(argv):
    runs = []
    for d in map(Path, argv):
        # A run directory, or a suite directory: then every run in it.
        runs += [d] if (d / 'summary.json').exists() else sorted(p for p in d.iterdir() if (p / 'summary.json').exists())
    if not runs:
        print('check-perception: no runs (directories with summary.json)', file=sys.stderr)
        return 2
    grand, bad, all_thinks = dict.fromkeys(COUNTS, 0), 0, 0
    for run in runs:
        thinks, totals, problems = check_run(run)
        all_thinks += thinks
        for k in COUNTS:
            grand[k] += totals[k]
        bad += len(problems)
        print(f"{run.name}: {thinks} thinks, {len(problems)} differences; "
              + ', '.join(f'{k} {v}' for k, v in totals.items()))
        for p in problems[:20]:
            print(f'  DIFFERENCE {p}')
        if len(problems) > 20:
            print(f'  ... and {len(problems) - 20} more')
    print(f"{len(runs)} runs, {all_thinks} thinks, {bad} differences; " + ', '.join(f'{k} {v}' for k, v in grand.items()))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

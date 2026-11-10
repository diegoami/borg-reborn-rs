#!/usr/bin/env python3
"""Check decision traces for loops hidden at one game turn.

  tools/check-no-time.py RUN_OR_SUITE_DIR ...

Level generation can change depth without advancing game time, so Holds and
wall-clock stalls are not sufficient evidence of progress. For every trace,
this reports the largest action counts at one turn and fails if Ascend/Descend
exceeds the profile's declared level cap, or if non-store actions exceed the
ordinary no-time budget plus its one forced stand-still action.
"""

import collections
import json
import sys
from pathlib import Path

STORE_ACTIONS = {'Buy', 'Sell'}
STAIRS = {'Ascend', 'Descend'}


def action_name(choice):
    return choice.split(' ', 1)[0]


def check_run(run):
    trace_path = run / 'trace.jsonl'
    if not trace_path.exists():
        return None
    turns = collections.defaultdict(list)
    for line in trace_path.read_text().splitlines():
        entry = json.loads(line)
        turns[entry['turn']].append(action_name(entry['choice']))
    params = json.loads((run / 'params.json').read_text())
    values = {p['name']: p['value'] for p in params['parameters']}
    level_cap = values['max_level_changes_per_turn']
    ordinary_cap = values['max_tries_per_turn'] + 1  # the forced stand-still
    problems = []
    for turn, actions in sorted(turns.items()):
        levels = sum(a in STAIRS for a in actions)
        ordinary = sum(a not in STORE_ACTIONS for a in actions)
        if levels > level_cap:
            problems.append(f'turn {turn}: {levels} level changes > cap {level_cap}')
        if ordinary > ordinary_cap:
            problems.append(f'turn {turn}: {ordinary} non-store actions > cap {ordinary_cap}')
    return {
        'run': run.name,
        'turns': len(turns),
        'max_actions': max((len(a) for a in turns.values()), default=0),
        'max_level_changes': max((sum(a in STAIRS for a in actions) for actions in turns.values()), default=0),
        'level_cap': level_cap,
        'max_nonstore': max((sum(a not in STORE_ACTIONS for a in actions) for actions in turns.values()), default=0),
        'ordinary_cap': ordinary_cap,
        'problems': problems,
    }


def main(argv):
    runs = []
    for arg in argv:
        root = Path(arg)
        runs.extend(sorted(p.parent for p in root.glob('**/trace.jsonl')))
    if not runs:
        print('check-no-time: no decision traces', file=sys.stderr)
        return 2
    failures = 0
    for run in runs:
        result = check_run(run)
        failures += len(result['problems'])
        print(f"{run}: {result['turns']} turns, max actions {result['max_actions']}, "
              f"non-store {result['max_nonstore']}/{result['ordinary_cap']}, "
              f"level changes {result['max_level_changes']}/{result['level_cap']}")
        for problem in result['problems']:
            print('  ' + problem)
    print(f'{len(runs)} traces, {failures} no-time budget violations')
    return int(bool(failures))


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

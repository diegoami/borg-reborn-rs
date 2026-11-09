#!/usr/bin/env python3
"""Check the exact-boundary run contract (task T1.4, issue #7).

  harness/check-boundary.py EXACT_DIR NORMAL_DIR [CAP]

EXACT_DIR is a suite output run with the `exact` flag (e.g. suite smoke-exact),
NORMAL_DIR the same suite without it, same arm and build. For every run:
  * the exact run's keys.log is a prefix of the normal run's keys.log (the
    hook stops play; it never changes it);
  * a survivor stopped at the cap itself (game_turns == cap, exact_turns);
  * a death before the cap is the same death (identical keys and cause);
  * depths.json exists, says "exact", and agrees with summary.json.
Exit status 1 on any failure; one line per run.
"""

import json
import sys
from pathlib import Path


def runs(d):
    return sorted(p.name for p in Path(d).iterdir() if (p / 'summary.json').exists())


def main(argv):
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    exact, normal = Path(argv[0]), Path(argv[1])
    names = runs(exact)
    if names != runs(normal):
        print(f'FAIL: run sets differ: {names} vs {runs(normal)}')
        return 1
    bad = 0
    for n in names:
        e, m = exact / n, normal / n
        se, sm = (json.loads((x / 'summary.json').read_text()) for x in (e, m))
        ke, km = (e / 'keys.log').read_bytes(), (m / 'keys.log').read_bytes()
        cap = int(argv[2]) if len(argv) > 2 else json.loads((e / 'depths.json').read_text())['cap']
        problems = []
        if not km.startswith(ke):
            problems.append('exact keys are not a prefix of the normal keys')
        if not se.get('exact_turns'):
            problems.append('summary has no exact_turns')
        if se.get('dead'):
            if not sm.get('dead') or ke != km or se.get('death_cause') != sm.get('death_cause'):
                problems.append('death before the cap differs from the normal run')
        elif se.get('stop_reason') == 'turns' and se.get('game_turns') != cap:
            problems.append(f"survivor stopped at {se.get('game_turns')}, not {cap}")
        try:
            d = json.loads((e / 'depths.json').read_text())
            if d['boundary'] != 'exact' or d['max_depth_at_stop'] != se.get('max_depth') \
                    or d['alive'] == bool(se.get('dead')):
                problems.append('depths.json disagrees with summary.json')
        except (OSError, ValueError, KeyError) as x:
            problems.append(f'depths.json unreadable: {x}')
        status = 'ok' if not problems else 'FAIL: ' + '; '.join(problems)
        kind = 'death' if se.get('dead') else f"alive at {se.get('game_turns')}"
        print(f"{n}: {kind}, max depth {se.get('max_depth')}, keys {len(ke.splitlines())}/{len(km.splitlines())}: {status}")
        bad += bool(problems)
    print(f'{len(names) - bad}/{len(names)} runs meet the exact-boundary contract')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

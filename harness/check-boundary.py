#!/usr/bin/env python3
"""Check the exact-boundary run contract (task T1.4, issue #7).

  harness/check-boundary.py EXACT_DIR NORMAL_DIR [CAP]

EXACT_DIR is a suite output run with the `exact` flag (e.g. suite smoke-exact),
NORMAL_DIR the same suite without it, same arm and build. For every run:
  * the exact run's keys.log is a prefix of the normal run's keys.log (the
    hook stops play; it never changes it);
  * a survivor stopped at the cap itself (game_turns == cap, exact_turns);
    any other stop of a living character (stall, Borg stop, key or level
    cap) fails, since it never reached the boundary;
  * a death or a victory before the cap is the same as in the normal run
    (identical keys, and the same cause of death);
  * the suite file matches the manifest's suite_sha256;
  * depths.json exists, says "exact", and agrees with summary.json.
Exit status 1 on any failure; one line per run.
"""

import hashlib
import json
import sys
from pathlib import Path


def runs(d):
    return sorted(p.name for p in Path(d).iterdir() if (p / 'summary.json').exists())


def listed(d):
    """The runs the suite in d's manifest lists, or None without a manifest.
    The suite file must match the manifest's suite_sha256; a manifest whose
    suite cannot be found at that hash raises ValueError."""
    m = Path(d) / 'manifest'
    if not m.exists():
        return None
    kv = dict(l.split('=', 1) for l in m.read_text().splitlines() if '=' in l)
    want = kv.get('suite_sha256')
    root = Path(__file__).resolve().parent.parent
    for c in (kv.get('suite_file'), root / 'harness/suites' / kv.get('suite', '')):
        if c and Path(c).is_file():
            if want and hashlib.sha256(Path(c).read_bytes()).hexdigest() != want:
                continue
            return sorted(l.split('#', 1)[0].split()[0] for l in Path(c).read_text().splitlines()
                          if len(l.split('#', 1)[0].split()) >= 5)
    raise ValueError(f'{d}: no suite file matches the manifest (suite={kv.get("suite")}, '
                     f'suite_file={kv.get("suite_file")}, suite_sha256={want})')


def main(argv):
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    exact, normal = Path(argv[0]), Path(argv[1])
    names = runs(exact)
    if not names:
        print(f'FAIL: no runs with a summary in {exact}')
        return 1
    if names != runs(normal):
        print(f'FAIL: run sets differ: {names} vs {runs(normal)}')
        return 1
    for d in (exact, normal):
        try:
            want = listed(d)
        except ValueError as x:
            print(f'FAIL: {x}')
            return 1
        if want is not None and want != names:
            print(f'FAIL: {d}: runs without a summary: {sorted(set(want) - set(names))}')
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
        elif se.get('stop_reason') == 'winner':
            if sm.get('stop_reason') != 'winner' or ke != km or not (se.get('game_turns') or cap) < cap:
                problems.append('victory before the cap differs from the normal run')
        elif se.get('stop_reason') != 'turns':
            # A stall, a Borg stop, a key or level cap: the run ended before
            # the boundary, so it does not exercise the contract.
            problems.append(f"survivor stopped by {se.get('stop_reason')!r} at {se.get('game_turns')}, "
                            f"before the cap {cap}")
        elif se.get('game_turns') != cap:
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

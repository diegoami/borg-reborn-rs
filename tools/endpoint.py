#!/usr/bin/env python3
"""The primary endpoint of docs/GOALS.md (task T1.7, issue #10).

Per arm and per class: the share of **all attempted runs** that are alive at
engine turn T having reached dungeon level D (or won before T). Operational
outcomes (stalls, timeouts, crashes, harness failures, configuration errors,
...) are non-successes, reported separately, never relabelled as deaths.

Inputs, any mix, each as ARM=SOURCE (repeat an arm to pool sources):

  --local ARM=DIR      a harness/suite.sh output directory; the suite file it
                       ran (manifest) lists every attempted run, so a run with
                       no summary.json counts as a harness failure
  --zoo ARM=FILE[:SPECIMEN[:PROFILE]]
                       borg-zoo result cells (JSON lines, the results branch);
                       optional filters on specimen and profile

  --depth D --turns T  the endpoint (GOALS D7: G1 uses D=5, T=100000)
  --pair A B           paired comparison of arms A and B on the runs they share
                       (same seed and class): discordant counts and an exact
                       McNemar p-value
  --allow-after-key    accept runs whose boundary was "after the first key at
                       or past T" (borgh without the exact flag, borg-zoo's
                       borg-rs adapter today); refused otherwise, because the
                       endpoint is defined at T itself
  --json               machine-readable output

borg-zoo's own goal_rate divides by death/turn_cap/completion cells only;
this tool does not (GOALS, *Observation and scoring rule*).
"""

import argparse
import json
import math
import sys
from collections import defaultdict
from pathlib import Path

SUCCESS_KINDS = ('turn_cap', 'completion')
STOP_TO_OUTCOME = {'turns': 'turn_cap', 'death': 'death', 'winner': 'completion', 'stalled': 'stalled',
                   'borg_stopped': 'borg_stopped', 'blocked': 'harness_failure', 'keys': 'decision_cap',
                   'clev': 'turn_cap', 'depth': 'turn_cap'}


def wilson(k, n, z=1.959963984540054):
    if n == 0:
        return (None, None)
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (max(0.0, c - h), min(1.0, c + h))


def mcnemar_exact(b, c):
    """Two-sided exact McNemar p-value from the discordant counts."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * tail)


def run(arm, seed, cls, outcome, alive, max_depth, stop_turn, boundary, source, victory=False):
    return dict(arm=arm, seed=seed, cls=cls or 'unknown', outcome=outcome, alive=alive,
                max_depth=max_depth or 0, stop_turn=stop_turn, boundary=boundary,
                victory=victory, source=source)


def suite_runs(suite_file):
    names = []
    for line in Path(suite_file).read_text().splitlines():
        line = line.split('#', 1)[0].split()
        if len(line) >= 5:
            names.append((line[0], int(line[1])))
    return names


def load_local(arm, d):
    d = Path(d)
    manifest = {}
    if (d / 'manifest').exists():
        manifest = dict(l.split('=', 1) for l in (d / 'manifest').read_text().splitlines() if '=' in l)
    suite = manifest.get('suite_file') or manifest.get('suite')
    root = Path(__file__).resolve().parent.parent
    listed = None
    for candidate in (suite, root / 'harness/suites' / str(suite)):
        if candidate and Path(candidate).is_file():
            listed = suite_runs(candidate)
            break
    if listed is None:
        listed = [(p.name, None) for p in sorted(d.iterdir()) if p.is_dir()]
    out = []
    for name, seed in listed:
        s = d / name / 'summary.json'
        if not s.exists():
            out.append(run(arm, seed, None, 'harness_failure', False, 0, None, 'unknown', f'{d}/{name}'))
            continue
        summ = json.loads(s.read_text())
        depths = {}
        if (d / name / 'depths.json').exists():
            depths = json.loads((d / name / 'depths.json').read_text())
        boundary = depths.get('boundary') or ('exact' if summ.get('exact_turns') else 'after-key')
        outcome = 'death' if summ.get('dead') else STOP_TO_OUTCOME.get(summ.get('stop_reason'), 'harness_failure')
        out.append(run(arm, summ.get('seed', seed), summ.get('class'), outcome, not summ.get('dead'),
                       summ.get('max_depth'), summ.get('game_turns'), boundary, f'{d}/{name}',
                       victory=summ.get('stop_reason') == 'winner'))
    return out


def load_zoo(arm, spec):
    path, *filters = spec.split(':')
    specimen = filters[0] if filters else None
    profile = filters[1] if len(filters) > 1 else None
    out = []
    for line in Path(path).read_text().splitlines():
        if not line.strip():
            continue
        c = json.loads(line)
        if specimen and c.get('specimen') != specimen or profile and c.get('profile') != profile:
            continue
        final = c.get('final') or {}
        boundary = c.get('boundary') or ('after-key' if c.get('harness_kind') == 'borg-rs' else 'exact')
        out.append(run(arm, c.get('seed'), c.get('played_class') or c.get('class'), c.get('outcome'),
                       c.get('outcome') in SUCCESS_KINDS, final.get('max_depth'), final.get('turn'),
                       boundary, f"{path}:{c.get('specimen')}:{c.get('seed')}",
                       victory=c.get('outcome') == 'completion'))
    return out


def success(r, depth, turns):
    if r['victory'] and (r['stop_turn'] is None or r['stop_turn'] <= turns):
        return True
    return (r['outcome'] == 'turn_cap' and r['alive'] and r['max_depth'] >= depth
            and r['stop_turn'] is not None and r['stop_turn'] >= turns)


def summarise(runs, depth, turns):
    by = defaultdict(list)
    for r in runs:
        by[(r['arm'], r['cls'])].append(r)
        by[(r['arm'], 'ALL')].append(r)
    rows = []
    for (arm, cls), rs in sorted(by.items(), key=lambda kv: (kv[0][0], kv[0][1] != 'ALL', kv[0][1])):
        n = len(rs)
        k = sum(success(r, depth, turns) for r in rs)
        outcomes = defaultdict(int)
        for r in rs:
            outcomes[r['outcome']] += 1
        lo, hi = wilson(k, n)
        rows.append(dict(arm=arm, cls=cls, attempted=n, successes=k, rate=k / n if n else None,
                         ci95=[lo, hi], deaths=outcomes.get('death', 0),
                         operational={o: v for o, v in sorted(outcomes.items())
                                      if o not in ('death', 'turn_cap', 'completion')},
                         outcomes=dict(sorted(outcomes.items()))))
    return rows


def paired(runs, a, b, depth, turns):
    key = lambda r: (r['seed'], r['cls'])
    ra = {key(r): r for r in runs if r['arm'] == a}
    rb = {key(r): r for r in runs if r['arm'] == b}
    shared = sorted(set(ra) & set(rb), key=str)
    only_a = sum(success(ra[k], depth, turns) and not success(rb[k], depth, turns) for k in shared)
    only_b = sum(success(rb[k], depth, turns) and not success(ra[k], depth, turns) for k in shared)
    return dict(a=a, b=b, shared=len(shared), a_only_success=only_a, b_only_success=only_b,
                mcnemar_p=mcnemar_exact(only_a, only_b))


def fmt_rate(r):
    if r['rate'] is None:
        return '—'
    lo, hi = r['ci95']
    return f"{100 * r['rate']:.1f}% [{100 * lo:.1f}, {100 * hi:.1f}]"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--local', action='append', default=[], metavar='ARM=DIR')
    ap.add_argument('--zoo', action='append', default=[], metavar='ARM=FILE[:SPECIMEN[:PROFILE]]')
    ap.add_argument('--depth', type=int, required=True)
    ap.add_argument('--turns', type=int, required=True)
    ap.add_argument('--pair', nargs=2, action='append', default=[], metavar=('A', 'B'))
    ap.add_argument('--allow-after-key', action='store_true')
    ap.add_argument('--json', action='store_true')
    a = ap.parse_args(argv)
    runs = []
    for spec in a.local:
        arm, d = spec.split('=', 1)
        runs += load_local(arm, d)
    for spec in a.zoo:
        arm, f = spec.split('=', 1)
        runs += load_zoo(arm, f)
    if not runs:
        ap.error('no runs: give --local and/or --zoo')
    after_key = [r for r in runs if r['boundary'] != 'exact' and r['outcome'] == 'turn_cap']
    if after_key and not a.allow_after_key:
        print(f'endpoint: {len(after_key)} surviving runs were not observed at the exact boundary '
              f'(e.g. {after_key[0]["source"]}); rerun with the exact flag or pass --allow-after-key',
              file=sys.stderr)
        return 2
    rows = summarise(runs, a.depth, a.turns)
    pairs = [paired(runs, x, y, a.depth, a.turns) for x, y in a.pair]
    result = dict(schema='borg-reborn-rs/endpoint@1', depth=a.depth, turns=a.turns,
                  definition='alive at turn T with max depth >= D, or a victory before T; '
                             'denominator: every attempted run',
                  boundary_after_key_allowed=a.allow_after_key, rows=rows, pairs=pairs)
    if a.json:
        print(json.dumps(result, indent=2))
        return 0
    print(f'Primary endpoint: alive at turn {a.turns} with max depth >= {a.depth} (or victory), '
          f'over all attempted runs\n')
    print('| Arm | Class | Attempted | Successes | Rate [95% CI] | Deaths | Operational |')
    print('| --- | --- | --- | --- | --- | --- | --- |')
    for r in rows:
        ops = ', '.join(f'{k} {v}' for k, v in r['operational'].items()) or '—'
        print(f"| {r['arm']} | {r['cls']} | {r['attempted']} | {r['successes']} | {fmt_rate(r)} | {r['deaths']} | {ops} |")
    for p in pairs:
        print(f"\nPaired {p['a']} vs {p['b']} on {p['shared']} shared runs: {p['a_only_success']} succeed only "
              f"with {p['a']}, {p['b_only_success']} only with {p['b']}; exact McNemar p = {p['mcnemar_p']:.3g}")
    return 0


if __name__ == '__main__':
    sys.exit(main())

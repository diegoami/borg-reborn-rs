#!/usr/bin/env python3
"""The primary endpoint of docs/GOALS.md (task T1.7, issue #10).

Per arm and per class: the share of **all attempted runs** that are alive at
engine turn T having reached dungeon level D (or won before T). Operational
outcomes (stalls, timeouts, crashes, harness failures, configuration errors,
...) are non-successes, reported separately, never relabelled as deaths.

Inputs, any mix, each as ARM=SOURCE (repeat an arm to pool sources):

  --local ARM=DIR      a harness/suite.sh output directory; the suite file it
                       ran (manifest) lists every attempted run, so a run with
                       no summary.json counts as a harness failure; so does
                       a run whose run.sh failed (run_exit), every run of a
                       suite whose binary changed while it ran
                       (binary_changed), and a run whose depths.json
                       contradicts its summary.json (evidence_conflict).
                       An incomplete output (no DONE or FAILED) is refused
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
# Boundary rule of zoo cells without a "boundary" field, by harness kind, at
# borg-zoo 3f6b76e: the better-borg-c observer stops at turn T itself; the
# borg-rs adapter after the first key at or past T; Neo-original turns an
# after-cap death into turn_cap and Mandos stops at the first loop boundary at
# or after the cap (docs/DESIGN.md). Any other kind must carry the field.
ZOO_BOUNDARY = {'better-borg-c': 'exact', 'borg-rs': 'after-key', 'neo-original': 'after-key',
                'mandos': 'after-key'}
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
    """[(name, seed, class index)] of a suite file."""
    out = []
    for line in Path(suite_file).read_text().splitlines():
        line = line.split('#', 1)[0].split()
        if len(line) >= 5:
            out.append((line[0], int(line[1]), int(line[3])))
    return out


def resolve_suite(d, manifest):
    """The suite file a local output ran: the manifest's resolved path (suite.sh
    since the M1 review), else a stock suite of that name verified by its hash.
    Refuses rather than guessing, so no attempted run can disappear."""
    import hashlib
    root = Path(__file__).resolve().parent.parent
    want = manifest.get('suite_sha256')
    candidates = [manifest.get('suite_file'), root / 'harness/suites' / str(manifest.get('suite'))]
    for c in candidates:
        if c and Path(c).is_file():
            if want and hashlib.sha256(Path(c).read_bytes()).hexdigest() != want:
                continue
            return Path(c)
    raise SystemExit(f'endpoint: cannot resolve the suite of {d} (manifest suite={manifest.get("suite")}, '
                     f'suite_file={manifest.get("suite_file")}); refusing to guess its attempted runs')


def suite_status(d, manifest):
    """'ok' or 'failed' for a complete harness/suite.sh output, 'binary_changed'
    if the binary's sha256 at the end differs from the start (then no run of
    it is evidence). Refuses an incomplete or contradictory output."""
    done, failed = (d / 'DONE').exists(), (d / 'FAILED').exists()
    if done == failed:
        raise SystemExit(f'endpoint: {d} has {"both DONE and FAILED" if done else "neither DONE nor FAILED"}; '
                         'not a complete suite.sh output')
    start, end = manifest.get('binary_sha256'), manifest.get('binary_sha256_end')
    result = manifest.get('finished_utc', '').rpartition('result=')[2]
    if not start or not end or result not in ('OK', 'FAILED'):
        raise SystemExit(f'endpoint: {d}: manifest lacks binary_sha256, binary_sha256_end or the result line; '
                         'not a complete suite.sh output')
    if (result == 'OK') != done:
        raise SystemExit(f'endpoint: {d}: manifest result={result} contradicts its {"DONE" if done else "FAILED"} marker')
    if start != end:
        return 'binary_changed'
    return 'ok' if done else 'failed'


def depth_conflict(summ, depths):
    """Why depths.json contradicts summary.json, or None."""
    want = {'max_depth_at_stop': summ.get('max_depth') or 0, 'stop_turn': summ.get('game_turns'),
            'alive': not summ.get('dead'), 'boundary': 'exact' if summ.get('exact_turns') else 'after-key'}
    bad = [f'{k} {depths.get(k)!r} vs summary {v!r}' for k, v in want.items() if depths.get(k) != v]
    return '; '.join(bad) or None


def load_local(arm, d):
    d = Path(d)
    if not (d / 'manifest').exists():
        raise SystemExit(f'endpoint: {d} has no manifest (not a harness/suite.sh output)')
    manifest = dict(l.split('=', 1) for l in (d / 'manifest').read_text().splitlines() if '=' in l)
    listed = suite_runs(resolve_suite(d, manifest))
    status = suite_status(d, manifest)
    raw = []
    for name, seed, cls_idx in listed:
        run_dir = d / name
        summ = json.loads((run_dir / 'summary.json').read_text()) if (run_dir / 'summary.json').exists() else None
        proc = json.loads((run_dir / 'process.json').read_text()) if (run_dir / 'process.json').exists() else None
        raw.append((name, seed, cls_idx, run_dir, summ, proc))
    # Class names by index from the runs that have a summary, so a run without
    # one still counts in its class (borgh without menu letters maps indices
    # of 7 and above to other classes: take the name the suite actually played).
    # Only an index that played one class names it; otherwise the run without
    # a summary goes to its own row rather than to a guessed class.
    played = defaultdict(set)
    for _, _, idx, _, summ, _ in raw:
        if summ and summ.get('class'):
            played[idx].add(summ['class'])
    out = []
    for name, seed, idx, run_dir, summ, proc in raw:
        names = played.get(idx, set())
        cls = (summ or {}).get('class') or (next(iter(names)) if len(names) == 1 else
                                            f'class index {idx}' + (' (ambiguous)' if names else ''))
        if status == 'binary_changed':
            out.append(run(arm, seed, cls, 'binary_changed', False, 0, None, 'unknown', str(run_dir)))
            continue
        # run.sh's own exit status (suite.sh records it): a run whose depth
        # stream or death export failed is not evidence, whatever its receipt.
        rc_file = run_dir / 'run_exit'
        rc = rc_file.read_text().strip() if rc_file.exists() else None
        run_ok = rc == '0' or (rc is None and status == 'ok')
        failed = not run_ok or proc is None or proc.get('status') != 'success' or proc.get('exit_code') != 0 \
            or proc.get('timed_out') or proc.get('stalled')
        if summ is None or failed:
            outcome = 'wall_timeout' if proc and proc.get('timed_out') else \
                'stalled' if proc and proc.get('stalled') else 'harness_failure'
            out.append(run(arm, seed, cls, outcome, False, 0, None, 'unknown', str(run_dir)))
            continue
        depths = json.loads((run_dir / 'depths.json').read_text()) if (run_dir / 'depths.json').exists() else None
        if depths is not None and depth_conflict(summ, depths):
            out.append(run(arm, seed, cls, 'evidence_conflict', False, 0, None, 'unknown', str(run_dir)))
            continue
        boundary = (depths or {}).get('boundary') or ('exact' if summ.get('exact_turns') else 'after-key')
        outcome = 'death' if summ.get('dead') else STOP_TO_OUTCOME.get(summ.get('stop_reason'), 'harness_failure')
        out.append(run(arm, summ.get('seed', seed), cls, outcome, not summ.get('dead'),
                       summ.get('max_depth'), summ.get('game_turns'), boundary, str(run_dir),
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
        boundary = c.get('boundary') or ZOO_BOUNDARY.get(c.get('harness_kind') or 'better-borg-c')
        if boundary is None:
            raise SystemExit(f"endpoint: zoo cell {path}:{c.get('specimen')}:{c.get('seed')} has harness kind "
                             f"{c.get('harness_kind')!r} and no boundary field; refusing to assume one")
        out.append(run(arm, c.get('seed'), c.get('played_class') or c.get('class'), c.get('outcome'),
                       c.get('outcome') in SUCCESS_KINDS, final.get('max_depth'), final.get('turn'),
                       boundary, f"{path}:{c.get('specimen')}:{c.get('seed')}",
                       victory=c.get('outcome') == 'completion'))
    return out


def success(r, depth, turns):
    """GOALS: a verified victory before T, or alive at T with max depth >= D.
    At the exact boundary a survivor must stop at T itself; an after-key
    survivor (only with --allow-after-key) at or after T."""
    if r['victory']:
        return r['stop_turn'] is not None and r['stop_turn'] < turns
    if r['outcome'] != 'turn_cap' or not r['alive'] or r['max_depth'] < depth or r['stop_turn'] is None:
        return False
    return r['stop_turn'] == turns if r['boundary'] == 'exact' else r['stop_turn'] >= turns


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
    ra, rb = {}, {}
    for arm, table in ((a, ra), (b, rb)):
        for r in runs:
            if r['arm'] != arm:
                continue
            if key(r) in table:
                raise SystemExit(f'endpoint: arm {arm} has two runs for seed {r["seed"]} class {r["cls"]} '
                                 f'({table[key(r)]["source"]}, {r["source"]}); pairing needs one run per key')
            table[key(r)] = r
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

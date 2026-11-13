#!/usr/bin/env python3
"""How much a suite's runs played, not only whether they lived (#201).

  tools/progress.py SUITE_OUT [SUITE_OUT ...] [--json]

"Alive" counted a Borg that had stopped playing: 38 of the full suite's 39
survivors walked between two grids on DL1 for about 85,000 turns. Every
survival figure is reported with these, per run and per suite output:

  frozen   game turns at the end of the run spent in windows of WINDOW turns
           that each visit at most FROZEN_GRIDS distinct grids (0: the run
           kept moving to the end)
  grids    distinct (depth, grid) positions the character stood on
  depth    maximum depth reached (from the state log's dN/MAX field)
  clevel   the character level at the end (state log's lvN)
  exp      the experience at the end (state log's xpN)
  turns    the last game turn of the run

Reads RUN/state.log and RUN/summary.json of every run directory.
"""

import argparse
import json
import sys
from pathlib import Path

WINDOW = 5000
FROZEN_GRIDS = 12


def field(parts, prefix):
    for p in parts:
        if p.startswith(prefix) and p[len(prefix):len(prefix) + 1].isdigit():
            return p[len(prefix):]
    return None


def run_progress(run):
    windows = {}
    grids = set()
    turn = depth = maxdepth = clevel = exp = 0
    for line in (run / 'state.log').read_text().splitlines():
        parts = line.split()
        if len(parts) < 3 or not parts[0].isdigit():
            continue
        turn = int(parts[0])
        d = parts[2].lstrip('d').split('/')
        depth, maxdepth = int(d[0]), max(maxdepth, int(d[-1]))
        pos = (depth, parts[1])
        grids.add(pos)
        windows.setdefault(turn // WINDOW, set()).add(pos)
        clevel = int(field(parts, 'lv') or clevel)
        exp = int(field(parts, 'xp') or exp)
    # The last window is partial: judge only the full ones.
    full = sorted(windows)[:-1]
    frozen = 0
    for k in reversed(full):
        if len(windows[k]) > FROZEN_GRIDS:
            break
        frozen += WINDOW
    summary = run / 'summary.json'
    dead = json.loads(summary.read_text()).get('dead') if summary.exists() else None
    return {'run': run.name, 'dead': bool(dead), 'frozen': frozen, 'grids': len(grids),
            'depth': maxdepth, 'clevel': clevel, 'exp': exp, 'turns': turn}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('outs', nargs='+', type=Path)
    ap.add_argument('--json', action='store_true')
    a = ap.parse_args()
    for out in a.outs:
        rows = [run_progress(r) for r in sorted(out.iterdir()) if (r / 'state.log').exists()]
        if a.json:
            print(json.dumps({'suite': str(out), 'runs': rows}))
            continue
        print(f'# {out}')
        print(f"{'run':24} {'dead':>4} {'frozen':>7} {'grids':>6} {'depth':>5} {'clev':>4} {'exp':>6}")
        for r in rows:
            print(f"{r['run']:24} {'yes' if r['dead'] else '':>4} {r['frozen']:>7} {r['grids']:>6} "
                  f"{r['depth']:>5} {r['clevel']:>4} {r['exp']:>6}")
        alive = [r for r in rows if not r['dead']]
        moving = [r for r in alive if r['frozen'] < 20000]
        mean = lambda k, rs: sum(r[k] for r in rs) / len(rs) if rs else 0
        print(f"runs {len(rows)}: alive {len(alive)}, alive and moving (frozen < 20,000 turns) "
              f"{len(moving)}; mean frozen {mean('frozen', rows):,.0f} turns, "
              f"mean max depth {mean('depth', rows):.2f}, mean clevel {mean('clevel', rows):.2f}, "
              f"reached DL5 {sum(r['depth'] >= 5 for r in rows)}")
    return 0


if __name__ == '__main__':
    sys.exit(main())

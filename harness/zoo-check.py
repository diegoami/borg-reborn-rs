#!/usr/bin/env python3
"""Run borg-zoo's own borg-rs adapter against this checkout (task T1.8, #11).

  harness/zoo-check.py [--impl rust c-pristine c-bbc-m7 ...] [--suite smoke] [--work DIR]

Loads the zoo at its pin (vendor/borg-zoo, scripts/setup.sh), adds in memory
one specimen per implementation: kind "borg-rs", repository
diegoami/borg-reborn-rs, ref = this checkout's HEAD. It then calls the zoo's
own harness.build and harness.run_cell, exactly as a zoo workflow would. The
zoo's files are not changed. It prints one line per cell (outcome, max depth,
turn, play hash) and exits 1 if a cell is a harness failure or a config error.
Commit first: the zoo refuses a checkout that is not at the specimen's ref.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ZOO = ROOT / 'vendor' / 'borg-zoo'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--impl', nargs='+', default=['rust', 'c-pristine', 'c-bbc-m7'])
    ap.add_argument('--suite', default='smoke')
    ap.add_argument('--work', type=Path, default=ROOT / 'harness/out/zoo-check')
    a = ap.parse_args(argv)
    sys.path.insert(0, str(ZOO))
    from zoo import catalogue as cat_mod, harness  # noqa: E402 (the pinned zoo)
    head = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip()
    cat = cat_mod.catalogue(ZOO)
    for impl in a.impl:
        cat['specimens']['specimens'][f'reborn-{impl}'] = {
            'kind': 'borg-rs', 'impl': impl, 'repository': 'diegoami/borg-reborn-rs', 'ref': head,
            'carries': 0, 'label': f'borg-reborn-rs {impl} at {head[:12]} (local zoo check, not in the catalogue)'}
    suite = cat['suites'][a.suite]
    bad = 0
    for impl in a.impl:
        name = f'reborn-{impl}'
        build = a.work / 'builds' / impl
        manifest = harness.build(name, None, build, cat=cat, source=ROOT)
        print(f"{name}: built {manifest['source_commit'][:12]} binary {manifest['binary_sha256'][:12]}")
        for seed in suite['seeds']:
            cell = {'specimen': name, 'profile': 'default', 'suite': a.suite, 'seed': seed}
            rec = harness.run_cell(cell, build, None, a.work / 'cells', cat=cat)
            final = rec.get('final') or {}
            print(f"  seed {seed}: {rec['outcome']:<15} max depth {final.get('max_depth')} turn {final.get('turn')} "
                  f"play {str(rec.get('play_sha256'))[:12]} first_turn_at_depth {rec.get('first_turn_at_depth')}")
            (a.work / 'cells.jsonl').parent.mkdir(parents=True, exist_ok=True)
            with (a.work / 'cells.jsonl').open('a') as f:
                f.write(json.dumps(rec) + '\n')
            bad += rec['outcome'] in ('harness_failure', 'config_error')
    print(f'{bad} cells with harness failure or config error')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())

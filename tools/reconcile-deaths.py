#!/usr/bin/env python3
"""Check that every death in a temporary registry is recorded in docs/deaths/.

  tools/reconcile-deaths.py TEMP_REGISTRY [--registry docs/deaths]

CI writes its runs' dossiers to a temporary registry (M1 review): a natural
death there whose (encounter family, key stream) is not in docs/deaths/ is a
death that would vanish with the runner. Exit 1 and list them; record them by
rerunning the same suites locally from the same commit (RULES 12).
"""

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def keys(reg):
    out = {}
    for rec_path in sorted(Path(reg).glob('D*/record.json')):
        rec = json.loads(rec_path.read_text())
        for a in rec.get('attempts', []):
            if a.get('outcome') == 'death':
                out[(rec.get('encounter_family'), a.get('result_sha256'))] = rec_path.parent.name
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('temp')
    ap.add_argument('--registry', default=str(ROOT / 'docs/deaths'))
    a = ap.parse_args(argv)
    known = keys(a.registry)
    new = {k: v for k, v in keys(a.temp).items() if k not in known}
    for (_, sha), case in sorted(new.items(), key=lambda kv: kv[1]):
        rec = json.loads((Path(a.temp) / case / 'record.json').read_text())
        print(f"UNRECORDED: {case}: {rec['raw_cause']} at turn {rec['terminal'].get('turn')} (keys {sha[:12]})")
    print(f"{len(keys(a.temp))} deaths in {a.temp}; {len(new)} not in {a.registry}")
    return 1 if new else 0


if __name__ == '__main__':
    sys.exit(main())

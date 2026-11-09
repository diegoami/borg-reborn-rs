#!/usr/bin/env python3
"""Check that every death in a temporary registry is recorded in docs/deaths/.

  tools/reconcile-deaths.py TEMP_REGISTRY [--registry docs/deaths] [--runs DIR ...]

CI writes its runs' dossiers to a temporary registry (M1 review): a natural
death there whose (encounter family, key stream) is not in docs/deaths/ is a
death that would vanish with the runner. Exit 1 and list them; record them by
rerunning the same suites locally from the same commit (RULES 12).

--runs also checks the run outputs themselves (E review, finding 3): every
run under DIR (a directory with summary.json) that died a natural death must
have a dossier in TEMP_REGISTRY with its key stream. A death the exporter
missed fails here, even when TEMP_REGISTRY is empty.
"""

import argparse
import hashlib
import importlib.util
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


def not_natural():
    spec = importlib.util.spec_from_file_location('export_death', ROOT / 'tools/export-death.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.not_natural


def unexported(runs, temp):
    """The natural deaths under the run directories with no dossier in temp."""
    why_not = not_natural()
    exported = {sha for _, sha in keys(temp)}
    missing, deaths = [], 0
    for root in runs:
        for summary_path in sorted(Path(root).glob('**/summary.json')):
            run = summary_path.parent
            summary = json.loads(summary_path.read_text())
            config = dict(l.split('=', 1) for l in (run / 'config.txt').read_text().splitlines()
                          if '=' in l) if (run / 'config.txt').exists() else {}
            if why_not(summary, config.get('flags', '-')) is not None:
                continue
            deaths += 1
            sha = hashlib.sha256((run / 'keys.log').read_bytes()).hexdigest() if (run / 'keys.log').exists() else None
            if sha not in exported:
                missing.append((run, summary))
    return deaths, missing


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('temp')
    ap.add_argument('--registry', default=str(ROOT / 'docs/deaths'))
    ap.add_argument('--runs', nargs='+', default=[], metavar='DIR')
    a = ap.parse_args(argv)
    failed = False
    if a.runs:
        deaths, missing = unexported(a.runs, a.temp)
        for run, summary in missing:
            print(f"UNEXPORTED: {run}: {summary.get('death_cause')} at turn {summary.get('game_turns')}")
        print(f"{deaths} natural deaths in the runs; {len(missing)} without a dossier in {a.temp}")
        failed = bool(missing)
    known = keys(a.registry)
    new = {k: v for k, v in keys(a.temp).items() if k not in known}
    for (_, sha), case in sorted(new.items(), key=lambda kv: kv[1]):
        rec = json.loads((Path(a.temp) / case / 'record.json').read_text())
        print(f"UNRECORDED: {case}: {rec['raw_cause']} at turn {rec['terminal'].get('turn')} (keys {sha[:12]})")
    print(f"{len(keys(a.temp))} deaths in {a.temp}; {len(new)} not in {a.registry}")
    return 1 if new or failed else 0


if __name__ == '__main__':
    sys.exit(main())

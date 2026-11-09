#!/usr/bin/env python3
"""Run borg-zoo's own borg-rs adapter against this checkout (task T1.8, #11).

  harness/zoo-check.py [--suite smoke] [--work DIR]

The contract it checks is docs/ZOO-INTERFACE.md. Only the Rust Borg is built
here (docs/SPEC.md §6).

Loads the zoo at its pin (vendor/borg-zoo, scripts/setup.sh), adds in memory
one specimen per implementation: kind "borg-rs", repository
diegoami/borg-reborn-rs, ref = this checkout's HEAD. It then calls the zoo's
own harness.build and harness.run_cell, exactly as a zoo workflow would. The
zoo's files are not changed. It prints one line per cell (outcome, max depth,
turn, play hash) and exits 1 if a cell is a harness failure or a config error.

Evidence rules (M1 review):
* the checkout must be clean (the specimen's ref is HEAD, and the adapter
  checks only the commit); vendor/borg-zoo must be at its pin and clean
  too (re-review N9: tracked changes and untracked files outside its
  .gitignore are refused), since its adapter is the code under check;
  the code imported is then a fresh `git archive` of the pin, so an ignored
  file in the checkout (a cached .pyc, an extension module) cannot stand in
  for the pinned source (re-review R4);
* the zoo adapter deletes keys.log, so a natural death would leave no
  dossier: every death cell is rerun through harness/run.sh (same seed,
  character, cap and menu letters), its key stream is checked against the
  cell's play hash, and run.sh's exporter records it (RULES 12).
"""

import argparse
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ZOO = ROOT / 'vendor' / 'borg-zoo'


def zoo_identity_error(zoo, pin):
    """None if the zoo checkout is exactly the pinned commit, else why not."""
    head = subprocess.check_output(['git', '-C', str(zoo), 'rev-parse', 'HEAD'], text=True).strip()
    if head != pin:
        return f'vendor/borg-zoo is at {head[:12]}, pin {pin[:12]} (scripts/setup.sh --checkout)'
    dirty = subprocess.check_output(['git', '-C', str(zoo), 'status', '--porcelain', '--untracked-files=all'],
                                    text=True).strip()
    if dirty:
        return f'vendor/borg-zoo has changes at its pin: {dirty.splitlines()[0].strip()} ...'
    return None


def pinned_tree(zoo, pin, dest):
    """Write the pinned zoo tree, exactly the committed files, to dest."""
    data = subprocess.check_output(['git', '-C', str(zoo), 'archive', '--format=tar', pin])
    with tarfile.open(fileobj=io.BytesIO(data)) as tar:
        tar.extractall(dest, filter='data')
    return Path(dest)


def import_zoo(tree):
    """Import the zoo's catalogue and harness from tree, and only from there."""
    if any(m == 'zoo' or m.startswith('zoo.') for m in sys.modules):
        raise SystemExit('zoo-check: a zoo package is already imported; run zoo-check in a fresh process')
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(tree))
    from zoo import catalogue, harness  # noqa: E402 (the pinned zoo)
    for mod in (catalogue, harness):
        if not Path(mod.__file__).resolve().is_relative_to(Path(tree).resolve()):
            raise SystemExit(f'zoo-check: {mod.__name__} came from {mod.__file__}, not the pinned tree')
    return catalogue, harness


def repatriate(impl, rec, suite, work, cat_mod):
    """Rerun a zoo death through harness/run.sh, check the keys, record the dossier."""
    import hashlib
    race, cls = cat_mod.character(suite, rec['seed']) or ('Human', 'Warrior')
    out = work / 'repatriated' / impl / str(rec['seed'])
    if out.exists():
        import shutil
        shutil.rmtree(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    env = dict(__import__('os').environ, BORG_IMPL=impl)
    r = subprocess.run([str(ROOT / 'harness/run.sh'), str(out), str(rec['seed']),
                        str(cat_mod.RACES.index(race)), str(cat_mod.CLASSES.index(cls)), str(suite['cap']), 'menu'],
                       env=env, capture_output=True, text=True)
    keys = out / 'keys.log'
    got = hashlib.sha256(keys.read_bytes()).hexdigest() if keys.exists() else None
    ok = r.returncode == 0 and got == rec.get('play_sha256')
    print(f"    death repatriated through harness/run.sh: {'keys match the zoo cell' if ok else 'MISMATCH'}"
          f"{(' ' + r.stderr.strip().splitlines()[-1]) if r.stderr.strip() else ''}")
    return ok


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--impl', nargs='+', default=['rust'], choices=['rust'])
    ap.add_argument('--suite', default='smoke')
    ap.add_argument('--work', type=Path, default=ROOT / 'harness/out/zoo-check')
    a = ap.parse_args(argv)
    head = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip()
    if subprocess.check_output(['git', '-C', str(ROOT), 'status', '--porcelain'], text=True).strip():
        raise SystemExit('zoo-check: the checkout has uncommitted changes; commit first (the cells name HEAD)')
    pin = json.loads((ROOT / 'pins.json').read_text())['pins']['borg-zoo']['commit']
    why = zoo_identity_error(ZOO, pin)
    if why:
        raise SystemExit(f'zoo-check: {why}')
    tmp = tempfile.TemporaryDirectory(prefix='zoo-check-pin-')
    tree = pinned_tree(ZOO, pin, tmp.name)
    cat_mod, harness = import_zoo(tree)
    cat = cat_mod.catalogue(tree)
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
            if rec['outcome'] == 'death':
                bad += not repatriate(impl, rec, suite, a.work, cat_mod)
    print(f'{bad} cells with harness failure or config error')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())

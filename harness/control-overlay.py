#!/usr/bin/env python3
"""Source trees for the control C Borgs (T1.3, #6; GOALS D8).

A control is our engine tree (angband/, never edited) with its src/borg/
replaced by a pinned C Borg source, built with BORG_IMPL=c in CMake terms.
The controls and their sources are the "controls" section of pins.json.

  harness/control-overlay.py --list               control names, one per line
  harness/control-overlay.py --identity IMPL      one line for the build-input hash
  harness/control-overlay.py --make IMPL OUTDIR   write the overlaid source tree

--make copies angband/ to OUTDIR/angband (mtimes kept), links OUTDIR/harness
to this repository's harness/ (the engine's CMake includes
../harness/measurement/c-adapter.h), replaces OUTDIR/angband/src/borg with the
pinned files from vendor/<repo> (cloned on demand if missing), and checks the
result blob for blob against the pinned tree. Any difference is an error.
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def controls():
    return json.loads((ROOT / 'pins.json').read_text()).get('controls', {})


def ensure_repo(repo, repository, commit):
    """A partial clone of the pinned repository with the commit, made on
    demand, so a fresh checkout (a borg-zoo build, CI) needs no setup step."""
    if not (repo / '.git').exists():
        repo.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(['git', 'clone', '-q', '--filter=blob:none', '--no-checkout',
                        f'https://github.com/{repository}.git', str(repo)], check=True)
    has = subprocess.run(['git', '-C', str(repo), 'cat-file', '-e', commit + '^{commit}'],
                         capture_output=True).returncode == 0
    if not has:
        subprocess.run(['git', '-C', str(repo), 'fetch', '-q', 'origin', commit], check=True)


def spec(impl):
    c = controls()
    if impl not in c:
        raise SystemExit(f'control-overlay: unknown control {impl!r} (known: {", ".join(sorted(c))})')
    s = dict(c[impl])
    pins = json.loads((ROOT / 'pins.json').read_text())['pins']
    repository = pins[s['pin']]['repository']
    s['repo'] = ROOT / 'vendor' / repository.split('/')[1]
    ensure_repo(s['repo'], repository, s['commit'])
    return s


def git(repo, *args):
    r = subprocess.run(['git', '-C', str(repo), *args], capture_output=True)
    if r.returncode:
        raise SystemExit(f'control-overlay: git {" ".join(args)} failed in {repo}: '
                         f'{r.stderr.decode().strip()} (run scripts/setup.sh)')
    return r.stdout


def pinned_tree(s):
    """{relative path: blob id} of the pinned source path."""
    out = {}
    for line in git(s['repo'], 'ls-tree', '-r', s['commit'], '--', s['path']).decode().splitlines():
        meta, p = line.split('\t', 1)
        out[p[len(s['path']):].lstrip('/')] = meta.split()[2]
    if not out:
        raise SystemExit(f"control-overlay: {s['path']} is empty at {s['commit']}")
    prefetch(s['repo'], sorted(set(out.values())))
    return out


def prefetch(repo, blobs):
    """Fetch the blobs a partial clone lacks in one request: fetching them one
    by one through lazy fetches is slow and fails on large sets."""
    env = dict(os.environ, GIT_NO_LAZY_FETCH='1')
    r = subprocess.run(['git', '-C', str(repo), 'cat-file', '--batch-check'], input='\n'.join(blobs) + '\n',
                       capture_output=True, text=True, env=env)
    missing = [line.split()[0] for line in r.stdout.splitlines() if line.endswith(' missing')]
    if missing:
        subprocess.run(['git', '-C', str(repo), '-c', 'fetch.negotiationAlgorithm=noop', 'fetch', '-q',
                        '--no-tags', '--no-write-fetch-head', '--filter=blob:none', 'origin', *missing], check=True)


def identity(impl):
    s = spec(impl)
    tree = git(s['repo'], 'rev-parse', f"{s['commit']}:{s['path']}").decode().strip()
    return f"control {impl} {s['pin']} {s['commit']} {s['path']} tree {tree}"


def make(impl, out):
    s = spec(impl)
    out = Path(out)
    files = pinned_tree(s)
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    (out / 'harness').symlink_to(ROOT / 'harness')
    shutil.copytree(ROOT / 'angband', out / 'angband', symlinks=True)
    borg = out / 'angband' / 'src' / 'borg'
    shutil.rmtree(borg)
    for rel, blob in sorted(files.items()):
        target = borg / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(git(s['repo'], 'cat-file', 'blob', blob))
    # Blob for blob: what will be compiled is exactly the pinned source.
    got = {}
    for f in sorted(p for p in borg.rglob('*') if p.is_file()):
        got[f.relative_to(borg).as_posix()] = subprocess.run(
            ['git', 'hash-object', str(f)], capture_output=True, text=True, check=True).stdout.strip()
    if got != files:
        raise SystemExit(f'control-overlay: {borg} differs from {s["commit"]}:{s["path"]}')
    return len(files)


def main(argv):
    if argv[:1] == ['--list']:
        print('\n'.join(sorted(controls())))
    elif argv[:1] == ['--identity'] and len(argv) == 2:
        print(identity(argv[1]))
    elif argv[:1] == ['--make'] and len(argv) == 3:
        n = make(argv[1], argv[2])
        print(f'control-overlay: {argv[1]}: {n} files of {spec(argv[1])["commit"][:12]}:'
              f'{spec(argv[1])["path"]} overlaid in {argv[2]}', file=sys.stderr)
    else:
        print(__doc__, file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

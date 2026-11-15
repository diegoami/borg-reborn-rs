#!/usr/bin/env python3
"""borg-core/ is borg-rs's crate vendored at borg-core/PIN (docs/PROVENANCE.md,
docs/SPEC.md §2 amendment). This tool keeps that copy honest and refreshable
(review #227, finding 9).

  scripts/borg-core-vendor.py delta [--repo vendor/borg-rs]
      our changes to borg-rs at the pin, as a patch (paths under borg-core/)
  scripts/borg-core-vendor.py check [--repo vendor/borg-rs]
      borg-core/ is borg-rs at the pin, plus our own files (OURS) and the
      files carrying our hooks (HOOKED); exit 1 otherwise
  scripts/borg-core-vendor.py refresh REV [--repo vendor/borg-rs]
      re-vendor at REV: each file merged three ways (base: borg-rs at the
      pin, theirs: borg-rs at REV, ours: borg-core/), our own files kept,
      PIN written; conflicts are left marked in the file and listed (exit 1)

borg-rs's layout maps onto borg-core/ as MAP says. A refresh goes in its own
PR, with the parity check of docs/HANDOVER.md (BORG_CORE=borg-rs, no errand,
against a fresh build of borg-rs at REV).
"""

import argparse
import difflib
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / 'borg-core'

# borg-rs path (a directory ends in '/') -> path under borg-core/.
MAP = [
    ('borg-rs/', ''),
    ('harness/borg.txt', 'borg.txt'),
    ('angband/src/borg/borg-init.c', 'c-reference/borg-init.c'),
    ('angband/src/borg/borg.txt', 'c-reference/borg.txt'),
]
# Files of ours only: never in borg-rs.
OURS = {'PIN', 'ladders.txt', 'src/shell.rs', 'src/params.rs', 'src/core_trace.rs'}
# borg-rs files that carry our changes (marked `borg-reborn-rs (#2xx)`, or
# the package rename and lints table in Cargo.toml).
HOOKED = {
    'Cargo.toml', 'src/borg_item_use.rs', 'src/borg_fight_defend.rs', 'src/borg_fight_attack.rs', 'src/borg_caution.rs', 'src/borg_danger.rs', 'src/borg_escape.rs', 'src/borg_recover.rs', 'src/borg_io.rs', 'src/borg_commands.rs', 'src/borg_init.rs', 'src/borg_messages.rs', 'src/borg_think.rs',
    'src/borg_think_dungeon.rs', 'src/borg_think_dungeon_util.rs', 'src/engine.rs', 'src/lib.rs',
}


def git(repo, *args, check=True, binary=False):
    r = subprocess.run(['git', '-C', str(repo), *args], capture_output=True, text=not binary)
    if check and r.returncode:
        raise SystemExit(f"borg-core-vendor: git {' '.join(args)}: {r.stderr if not binary else r.stderr.decode()}")
    return r


def donor(repo, rev):
    """{borg-core path: blob id} of borg-rs at rev, through MAP."""
    out = {}
    for src, dst in MAP:
        listing = git(repo, 'ls-tree', '-r', rev, '--', src.rstrip('/')).stdout
        for line in listing.splitlines():
            meta, path = line.split('\t', 1)
            rel = dst + path[len(src):] if src.endswith('/') else dst
            out[rel] = meta.split()[2]
    return out


def blob(repo, oid):
    return git(repo, 'cat-file', 'blob', oid, binary=True).stdout


def ours():
    """{borg-core path: bytes} of the working tree's borg-core/."""
    return {str(p.relative_to(CORE)): p.read_bytes() for p in sorted(CORE.rglob('*')) if p.is_file()}


def pin():
    return (CORE / 'PIN').read_text().split()[0]


def delta(repo):
    base, mine = donor(repo, pin()), ours()
    out = []
    for rel in sorted(set(base) | set(mine)):
        old = blob(repo, base[rel]) if rel in base else None
        new = mine.get(rel)
        if old == new:
            continue
        text = lambda b: [] if b is None else b.decode('utf-8', 'surrogateescape').splitlines(keepends=True)
        out.extend(difflib.unified_diff(text(old), text(new),
                                        '/dev/null' if old is None else f'a/borg-core/{rel}',
                                        '/dev/null' if new is None else f'b/borg-core/{rel}'))
    return ''.join(out)


def check(repo):
    base, mine = donor(repo, pin()), ours()
    problems = []
    for rel in sorted(set(base) | set(mine)):
        if rel in OURS:
            if rel in base:
                problems.append(f'{rel}: declared ours, but borg-rs has it')
            continue
        if rel not in mine:
            problems.append(f'{rel}: in borg-rs at the pin, missing here')
        elif rel not in base:
            problems.append(f'{rel}: not in borg-rs at the pin, and not declared ours')
        elif blob(repo, base[rel]) != mine[rel] and rel not in HOOKED:
            problems.append(f'{rel}: differs from borg-rs at the pin, and not declared hooked')
    return problems


def refresh(repo, rev):
    rev = git(repo, 'rev-parse', rev + '^{commit}').stdout.strip()
    base, theirs, mine = donor(repo, pin()), donor(repo, rev), ours()
    conflicts = []
    for rel in sorted(set(base) | set(theirs) | set(mine)):
        if rel in OURS:
            continue
        path = CORE / rel
        b = blob(repo, base[rel]) if rel in base else None
        t = blob(repo, theirs[rel]) if rel in theirs else None
        o = mine.get(rel)
        if t == b:
            continue  # borg-rs did not change it: ours stands
        if o == b:  # we did not change it: take theirs
            if t is None:
                path.unlink()
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(t)
            continue
        if t is None or o is None or b is None:
            conflicts.append(f'{rel}: changed on one side, added or removed on the other')
            continue
        with tempfile.TemporaryDirectory() as tmp:
            fb, ft = Path(tmp) / 'base', Path(tmp) / 'theirs'
            fb.write_bytes(b), ft.write_bytes(t)
            r = subprocess.run(['git', 'merge-file', '-L', 'ours', '-L', 'pin', '-L', rev[:12], str(path), str(fb), str(ft)],
                               capture_output=True)
            if r.returncode:
                conflicts.append(f'{rel}: conflict, marked in the file')
    (CORE / 'PIN').write_text(rev + '\n')
    return conflicts


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('command', choices=('delta', 'check', 'refresh'))
    ap.add_argument('rev', nargs='?')
    ap.add_argument('--repo', type=Path, default=ROOT / 'vendor/borg-rs')
    a = ap.parse_args(argv)
    if a.command == 'delta':
        sys.stdout.write(delta(a.repo))
        return 0
    if a.command == 'check':
        problems = check(a.repo)
        for p in problems:
            print(f'borg-core-vendor: {p}')
        print(f"borg-core/ against borg-rs {pin()[:12]}: {'ok' if not problems else f'{len(problems)} problems'}")
        return 1 if problems else 0
    if not a.rev:
        ap.error('refresh needs REV')
    conflicts = refresh(a.repo, a.rev)
    for c in conflicts:
        print(f'borg-core-vendor: {c}')
    print(f"borg-core/ re-vendored at {pin()[:12]}: {len(conflicts)} conflicts")
    return 1 if conflicts else 0


if __name__ == '__main__':
    sys.exit(main())

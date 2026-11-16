#!/usr/bin/env python3
"""borg-core/ is borg-rs's crate vendored at borg-core/PIN (docs/PROVENANCE.md,
docs/SPEC.md §2 amendment). This tool keeps that copy honest and refreshable
(review #227, finding 9).

  scripts/borg-core-vendor.py delta [--repo vendor/borg-rs]
      our changes to borg-rs at the pin, as a patch (paths under borg-core/)
  scripts/borg-core-vendor.py check [--repo vendor/borg-rs]
      borg-core/ is borg-rs at the pin, plus our own files (OURS) and the
      files carrying our hooks (HOOKED), whose donor blobs and receiving bytes
      must match scripts/borg-core-hooks.json; exit 1 otherwise
  scripts/borg-core-vendor.py check-recorded
      verify recorded donor inventory and receiving bytes offline in CI; no
      reference checkout, build or external code is used
  scripts/borg-core-vendor.py record-hooks [--repo vendor/borg-rs]
      explicitly record reviewed hook bytes; never run implicitly in check
      or refresh. Review source delta and manifest changes together.
  scripts/borg-core-vendor.py refresh REV [--repo vendor/borg-rs]
      re-vendor at REV: each file merged three ways (base: borg-rs at the
      pin, theirs: borg-rs at REV, ours: borg-core/), our own files kept,
      PIN written; conflicts are left marked in the file and listed (exit 1)

borg-rs's layout maps onto borg-core/ as MAP says. A refresh goes in its own
PR, with the parity check of docs/HANDOVER.md (BORG_CORE=borg-rs, no errand,
against a fresh build of borg-rs at REV).
"""

import argparse
import hashlib
import json
import re
import difflib
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / 'borg-core'
HOOK_LOCK = ROOT / 'scripts/borg-core-hooks.json'

# borg-rs path (a directory ends in '/') -> path under borg-core/.
MAP = [
    ('borg-rs/', ''),
    ('harness/borg.txt', 'borg.txt'),
    ('angband/src/borg/borg-init.c', 'c-reference/borg-init.c'),
    ('angband/src/borg/borg.txt', 'c-reference/borg.txt'),
]
# Files of ours only: never in borg-rs.
OURS = {
    'PIN',
    'ladders.txt',
    'src/core_trace.rs',
    'src/decision_api.rs',
    'src/decision_snapshot.rs',
    'src/decision_tape.rs',
    'src/decisions.rs',
    'src/errands_plan.rs',
    'src/memory.rs',
    'src/params.rs',
    'src/shadow.rs',
    'src/shell.rs',
    'src/threat_facts.rs',
}
# borg-rs files that carry our changes (marked `borg-reborn-rs (#2xx)`, or
# the package rename and lints table in Cargo.toml).
HOOKED = {
    # #343: Clone/PartialEq for owned town/trip memory.
    'src/borg_town_live.rs',
    'src/borg_trip.rs',
    'Cargo.toml',
    'build.rs',
    'src/borg.rs',
    'src/borg_caution.rs',
    'src/borg_cave.rs',
    'src/borg_cave_light.rs',
    'src/borg_cave_view.rs',
    'src/borg_commands.rs',
    'src/borg_danger.rs',
    'src/borg_escape.rs',
    'src/borg_fight_attack.rs',
    'src/borg_fight_defend.rs',
    'src/borg_flow.rs',
    'src/borg_flow_glyph.rs',
    'src/borg_flow_kill.rs',
    'src/borg_flow_misc.rs',
    'src/borg_flow_stairs.rs',
    'src/borg_flow_take.rs',
    'src/borg_formulas.rs',
    'src/borg_formulas_calc.rs',
    'src/borg_init.rs',
    'src/borg_inventory.rs',
    'src/borg_io.rs',
    'src/borg_item.rs',
    'src/borg_item_activation.rs',
    'src/borg_item_use.rs',
    'src/borg_item_val.rs',
    'src/borg_item_wear.rs',
    'src/borg_magic.rs',
    'src/borg_messages.rs',
    'src/borg_messages_react.rs',
    'src/borg_power.rs',
    'src/borg_prepared.rs',
    'src/borg_projection.rs',
    'src/borg_recover.rs',
    'src/borg_store.rs',
    'src/borg_store_sell.rs',
    'src/borg_think.rs',
    'src/borg_think_dungeon.rs',
    'src/borg_think_dungeon_util.rs',
    'src/borg_update.rs',
    'src/engine.rs',
    'src/headers/option_h.rs',
    'src/lib.rs',
    'src/shadow.rs',
    'src/state.rs',
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


def hook_manifest(repo):
    """An explicit reviewed byte contract, never regenerated by check/refresh."""
    base, mine = donor(repo, pin()), ours()
    paths = sorted((HOOKED - OURS) & set(base))
    missing = set(HOOKED - OURS) - set(base)
    if missing:
        raise ValueError('hook declarations missing from donor: ' + ', '.join(sorted(missing)))
    return {'schema': 'borg-reborn-rs/vendor-hooks@1', 'donor_pin': pin(),
            'donor_files': dict(sorted(base.items())),
            'files': {rel: {'donor_blob': base[rel],
                            'receiving_sha256': hashlib.sha256(mine[rel]).hexdigest()}
                      for rel in paths}}


def verify_hooks(base, mine):
    try:
        lock = json.loads(HOOK_LOCK.read_text())
    except (OSError, ValueError) as error:
        return [f'hook manifest: missing or invalid ({error})']
    problems = []
    if not isinstance(lock, dict):
        return ['hook manifest: expected an object']
    if lock.get('donor_files') != base:
        problems.append('hook manifest: donor inventory differs from pinned reference')
    if lock.get('schema') != 'borg-reborn-rs/vendor-hooks@1' or lock.get('donor_pin') != pin():
        problems.append('hook manifest: schema or donor pin differs')
    expected = set(HOOKED - OURS)
    files = lock.get('files')
    if not isinstance(files, dict) or set(files) != expected:
        return problems + ['hook manifest: declared file inventory differs']
    for rel in sorted(expected):
        entry = files[rel]
        if not isinstance(entry, dict) or entry.get('donor_blob') != base.get(rel):
            problems.append(f'{rel}: hooked donor blob differs from manifest')
        if rel not in mine or not isinstance(entry, dict) or entry.get('receiving_sha256') != hashlib.sha256(mine[rel]).hexdigest():
            problems.append(f'{rel}: hooked content differs from reviewed manifest')
    return problems


def check(repo):
    base, mine = donor(repo, pin()), ours()
    problems = verify_hooks(base, mine)
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


def check_recorded():
    """Offline CI attestation: no reference checkout or external code dependency.

    The reference audit/recording command proves the donor inventory separately;
    CI checks this exact recorded inventory and all receiving bytes locally.
    """
    try:
        lock = json.loads(HOOK_LOCK.read_text())
        base = lock['donor_files']
        if not isinstance(base, dict) or not base or any(
                not isinstance(value, str) or not re.fullmatch('[0-9a-f]{40}', value)
                for value in base.values()):
            return ['hook manifest: invalid recorded donor inventory']
    except (OSError, ValueError, KeyError, TypeError) as error:
        return [f'hook manifest: missing or invalid ({error})']
    mine = ours()
    problems = verify_hooks(base, mine)
    for rel in sorted(set(base) | set(mine)):
        if rel in OURS:
            if rel in base:
                problems.append(f'{rel}: declared ours, but recorded donor has it')
            continue
        if rel not in mine:
            problems.append(f'{rel}: in recorded donor, missing here')
        elif rel not in base:
            problems.append(f'{rel}: not in recorded donor, and not declared ours')
        elif rel not in HOOKED:
            data = mine[rel]
            oid = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if oid != base[rel]:
                problems.append(f'{rel}: differs from recorded donor, and not declared hooked')
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
        if t is None and o is None:
            continue  # both sides removed it (re-review #227, 8)
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
    ap.add_argument('command', choices=('delta', 'check', 'refresh', 'record-hooks', 'check-recorded'))
    ap.add_argument('rev', nargs='?')
    ap.add_argument('--repo', type=Path, default=ROOT / 'vendor/borg-rs')
    a = ap.parse_args(argv)
    if a.command == 'record-hooks':
        HOOK_LOCK.write_text(json.dumps(hook_manifest(a.repo), indent=2) + '\n')
        print('Recorded exact hook bytes; review the source delta and manifest together')
        return 0
    if a.command == 'delta':
        sys.stdout.write(delta(a.repo))
        return 0
    if a.command in ('check', 'check-recorded'):
        problems = check(a.repo) if a.command == 'check' else check_recorded()
        for p in problems:
            print(f'borg-core-vendor: {p}')
        print(f"borg-core/ {a.command} at {pin()[:12]}: {'ok' if not problems else f'{len(problems)} problems'}")
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

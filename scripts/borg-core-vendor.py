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
OURS = {'PIN', 'ladders.txt', 'src/shell.rs', 'src/params.rs', 'src/core_trace.rs', 'src/attack_preference.rs', 'src/errands_plan.rs', 'src/scary_list.rs', 'src/borg_lightless.rs', 'src/borg_town_hunt.rs', 'src/borg_cursed.rs', 'src/borg_fast_fragile.rs', 'src/borg_kit.rs', 'src/borg_lone_target.rs', 'src/borg_unseen_belief.rs', 'src/borg_trap_way.rs'}
# borg-rs files that carry our changes (marked `borg-reborn-rs (#2xx)`, or
# the package rename and lints table in Cargo.toml).
HOOKED = {
    'src/borg_item_enchant.rs', 'src/borg_item_decurse.rs',  # #680 neutral policy extraction
    'src/borg_item_wear.rs', 'src/borg_store_buy.rs',
    'src/borg_item_decurse.rs', 'src/borg_item_analyze.rs',  # #830: can_decurse and borg_object_value_known are shared with the cursed-equipment module; the swap decurse reads a scroll only for an item worth its price
    'src/borg_think_store.rs', 'src/borg_home_power.rs',  # #433 inventory parameters
    'src/borg_reincarnate.rs',  # #447: refresh plan cap after reincarnation
    'src/borg_trip.rs',  # #795: trace the live trip preparedness check
    'src/borg_prepared.rs',  # #447: independent plan cap
    'src/borg_headers/borg_h.rs', 'src/borg_town_solve.rs',  # #810: Black Market reserve
    'src/borg_town_usefulness.rs',  # #400/#443: accepted receiving planner item values
    'src/borg_town_capability.rs',  # #380: retain physical Phase Door value
    'src/borg_town_shadow.rs',  # #460: no knowledge-only purchases (effective_params)
    'src/borg_town_dispose.rs',  # #1048: physical Home stock policy facts
    'src/borg_town_utility.rs',  # #451: Remove Hunger disposition test
    'src/borg_town_dump.rs',  # #797: Phase Door stock and first-contact receipts
    'src/borg_town_record.rs',  # #651: plan schema 12 (what a visit bought is not stored)
    'src/state.rs',  # #840: the basic kit's state
    'src/borg_flow_take.rs',  # #732: C's take leashes read from c_leash (parameters)
    'src/borg_magic.rs', 'src/borg_magic_play.rs',  # #694: Fox Form's study rating (magic_fox_form_study_rating)
    'src/borg_trait.rs',  # #451: Remove Hunger scrolls counted beside the rations under the town plan
    'src/borg_town_budget.rs',  # #493: explicit gold reserve in Budget::of
    'src/borg_junk.rs',  # #470: discard Deep Descent on sight
    'src/borg_item.rs',  # #775: defer uninscribing carried items until normal form
    'src/borg_flow_kill.rs',  # #503: no breeder scary level without an easy escape; melee classes hunt breeders; named scary monsters behind a flag
    'src/borg_flow_stairs.rs',  # #557: no flight down a staircase while rising
    'src/borg_flow_misc.rs',  # #597: the twitch refuses lava
    'src/borg_update.rs',  # #503: door-opened scare behind caution_scary_named_monsters; #929: a miss in view is not a wall
    'src/borg_store_sell.rs',  # #470: historical chooser sells one of each unaware kind
    'src/borg_town_live.rs',  # #507: a planned purchase asks for its copies
    'src/borg.rs',  # #530: the scary goals and scary stand flags
    'src/borg_store_buy.rs',  # #507: types the purchase quantity, logs the copies bought
    'src/borg_object_log.rs',  # #507: a purchase of several copies is one log line
    'src/borg_cave_light.rs',  # #796: level-local dark-entry attempt tracking
    'src/borg_light.rs',  # #685: no detection spell by panel below a level; #884: radius branches, dark frontier and Light Beam read the own light radius
    'src/borg_flow_dark.rs',  # #884: flow_dark_2 looks just outside the own light radius
    'src/borg_projection.rs',  # #884: borg_los_light, line of sight through projectable grids only; #929: no wall guess on the Borg's grid
    'src/borg_flow.rs',  # #551: a Druid's flight step casts Fox Form first
    'build.rs',  # #328: receiving recorder integrity checker path
    'src/headers/option_h.rs',  # #328: receiving upstream player_options ABI

    'Cargo.toml', 'src/borg_power.rs', 'src/borg_item_use.rs', 'src/borg_fight_defend.rs', 'src/borg_fight_attack.rs', 'src/borg_caution.rs', 'src/borg_danger.rs', 'src/borg_escape.rs', 'src/borg_recover.rs', 'src/borg_io.rs', 'src/borg_commands.rs', 'src/borg_init.rs', 'src/borg_messages.rs', 'src/borg_think.rs',
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

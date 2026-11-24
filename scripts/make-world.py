#!/usr/bin/env python3
"""Make a world: a copy of the game data with some monsters made more common
(#704, owner 2026-11-19: "we could test breeders, like using a engine where
breeders are more frequent, it should be driveable by configuration file").

  scripts/make-world.py --config harness/worlds/NAME.txt --lib LIB --out DIR

LIB is the game's lib directory (the build's `lib/`, upstream's data). DIR
gets `gamedata/`, `customize/`, `help/` and `screens/`, the four directories
`harness/run.sh` passes with `-dgamedata`, `-dpref`, `-dhelp` and
`-dscreens`. Only the data changes: the engine is upstream byte for byte
(RULES 9). It prints the SHA-256 of the generated `gamedata/` files, which
`harness/run.sh` records.

The config is `key = value` lines (`#` starts a comment):

  breeder_weight = N        every race without MULTIPLY gets its rarity
                            times N, so breeders are N times as likely
                            against the rest (`mon-make.c`: weight 100 /
                            rarity); the level's monster count is unchanged
  breeder_depth_shift = N   breeders' `depth:` N levels shallower (at least 1)
  breeders_only = a, b      only breeders whose name contains one of these
                            are favoured (the rest count as non-breeders)
  breeders_except = a, b    breeders whose name contains one of these are not
  repro_max = N             `constants.txt` `mon-gen:repro-max` (100), the
                            engine's cap on live breeders
  favour = a, b             any races, breeders or not, named exactly (case
                            does not matter) that favour_weight favours:
                            every race not named gets its rarity times the
                            weight, as breeder_weight does for breeders
                            (a world for one pest, e.g. the grey mushroom
                            patch); it needs favour_weight; a name that
                            matches no race is refused
  favour_weight = N         the weight for `favour`, at least 2; it needs
                            `favour`; not together with the breeder_*
                            settings

A rarity above 100 would never be generated (100 / rarity is 0), so a
weight that pushes one past it is refused.
"""

import argparse
import hashlib
import shutil
import sys
from pathlib import Path

KEYS = {'breeder_weight', 'breeder_depth_shift', 'breeders_only', 'breeders_except', 'repro_max',
        'favour', 'favour_weight'}
DIRS = ('gamedata', 'customize', 'help', 'screens')


def read_config(path):
    """The config's settings, as strings; refuses an unknown key."""
    out = {}
    for n, line in enumerate(Path(path).read_text().splitlines(), 1):
        line = line.split('#', 1)[0].strip()
        if not line:
            continue
        key, sep, value = line.partition('=')
        key, value = key.strip(), value.strip()
        if not sep or key not in KEYS:
            raise SystemExit(f'make-world: {path}:{n}: unknown setting {line!r} (known: {", ".join(sorted(KEYS))})')
        if key in out:
            raise SystemExit(f'make-world: {path}:{n}: {key} is set twice')
        out[key] = value
    return out


def number(cfg, key, default, least):
    """A whole-number setting, at least LEAST; a clean refusal otherwise."""
    raw = cfg.get(key, str(default))
    try:
        n = int(raw)
    except ValueError:
        raise SystemExit(f'make-world: {key} = {raw!r}: a whole number is expected') from None
    if n < least:
        raise SystemExit(f'make-world: {key} = {n}: at least {least} is expected')
    return n


def names(value):
    return [v.strip().lower() for v in value.split(',') if v.strip()] if value else []


def records(text):
    """monster.txt split into records: (lines of the record), each starting
    at a `name:` line; lines before the first record are a header record."""
    out, cur = [], []
    for line in text.splitlines(keepends=True):
        if line.startswith('name:') and cur:
            out.append(cur)
            cur = []
        cur.append(line)
    if cur:
        out.append(cur)
    return out


def transform_monsters(text, cfg, max_depth):
    """The monster list with the config applied, and a summary."""
    weight = number(cfg, 'breeder_weight', 1, 1)
    shift = number(cfg, 'breeder_depth_shift', 0, 0)
    only, never = names(cfg.get('breeders_only', '')), names(cfg.get('breeders_except', ''))
    named = names(cfg.get('favour', ''))
    # The key, not the parsed list, picks the mode: `favour = , ,` is a
    # mistake, not "no favour" (review of #971).
    if 'favour' in cfg and not named:
        raise SystemExit('make-world: favour names no race')
    if 'favour_weight' in cfg and not named:
        raise SystemExit('make-world: favour_weight needs favour (the races it favours)')
    if named:
        both = sorted(k for k in cfg if k.startswith('breeder'))
        if both:
            raise SystemExit(f'make-world: favour and {", ".join(both)} together: choose one')
        # A weight is required and above 1: `favour` alone (weight 1) would
        # write the game's own data and report a favoured race (review of
        # #971), so a run could not tell it from the normal world.
        if 'favour_weight' not in cfg:
            raise SystemExit('make-world: favour needs favour_weight (at least 2)')
        weight = number(cfg, 'favour_weight', 2, 2)
    favoured, others = [], 0
    out = []
    for rec in records(text):
        head = rec[0]
        if not head.startswith('name:'):
            out.extend(rec)
            continue
        name = head[5:].strip()
        flags = ' '.join(l for l in rec if l.startswith('flags:'))
        breeder = 'MULTIPLY' in flags
        lname = name.lower()
        if named:
            # `favour`: the named races, breeders or not, are the favoured ones.
            breeder = lname in named
        if breeder and only and not any(o in lname for o in only):
            breeder = False
        if breeder and any(x in lname for x in never):
            breeder = False
        new = []
        for line in rec:
            if line.startswith('rarity:') and not breeder and weight > 1:
                r = int(line[7:].strip())
                if r * weight > 100:
                    raise SystemExit(f'make-world: {name}: rarity {r} x {weight} > 100 would never be generated')
                line = f'rarity:{r * weight}\n'
            elif line.startswith('depth:') and breeder and shift:
                d = int(line[6:].strip())
                line = f'depth:{min(max_depth, max(1, d - shift))}\n'
            new.append(line)
        out.extend(new)
        if breeder:
            favoured.append(name)
        else:
            others += 1
    missing = [n for n in named if n not in {f.lower() for f in favoured}]
    if missing:
        raise SystemExit(f'make-world: favour names no race: {", ".join(missing)}')
    return ''.join(out), favoured, others


def transform_constants(text, cfg):
    if 'repro_max' not in cfg:
        return text
    n = number(cfg, 'repro_max', 100, 1)
    lines, done = [], False
    for line in text.splitlines(keepends=True):
        if line.startswith('mon-gen:repro-max:'):
            line, done = f'mon-gen:repro-max:{n}\n', True
        lines.append(line)
    if not done:
        raise SystemExit('make-world: constants.txt has no mon-gen:repro-max line')
    return ''.join(lines)


def max_depth_of(constants):
    for line in constants.splitlines():
        if line.startswith('world:max-depth:'):
            return int(line.split(':')[2])
    raise SystemExit('make-world: constants.txt has no world:max-depth line')


def gamedata_sha256(gamedata):
    h = hashlib.sha256()
    for p in sorted(Path(gamedata).rglob('*')):
        if p.is_file():
            h.update(p.relative_to(gamedata).as_posix().encode() + b'\0')
            h.update(p.read_bytes())
    return h.hexdigest()


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--config', required=True)
    ap.add_argument('--lib', required=True, help="the game's lib directory")
    ap.add_argument('--out', required=True, help='a directory that does not exist yet')
    a = ap.parse_args(argv)
    cfg = read_config(a.config)
    lib, out = Path(a.lib), Path(a.out)
    if out.exists():
        raise SystemExit(f'make-world: {out} exists')
    for d in DIRS:
        if not (lib / d).is_dir():
            raise SystemExit(f'make-world: {lib / d}: no such directory')
    out.mkdir(parents=True)
    for d in DIRS:
        shutil.copytree(lib / d, out / d)
    gd = out / 'gamedata'
    constants = (gd / 'constants.txt').read_text()
    monsters, favoured, others = transform_monsters((gd / 'monster.txt').read_text(), cfg, max_depth_of(constants))
    (gd / 'monster.txt').write_text(monsters)
    (gd / 'constants.txt').write_text(transform_constants(constants, cfg))
    kind = 'races' if 'favour' in cfg else 'breeders'
    print(f'world: {len(favoured)} {kind} favoured ({", ".join(favoured)}); {others} other races', file=sys.stderr)
    print(gamedata_sha256(gd))


if __name__ == '__main__':
    main()

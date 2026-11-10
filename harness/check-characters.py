#!/usr/bin/env python3
"""Every run of a suite output plays the race and class its suite line asks for.

  harness/check-characters.py OUTDIR GAMEDATA_DIR

Reads OUTDIR/manifest's suite file, and for each run compares summary.json's
race and class names with the names at the requested indices in
GAMEDATA_DIR/p_race.txt and class.txt (their order is the birth menu's, which
the `menu` flag selects by). Exit 1 on any mismatch. E review, finding 1: the
full suite had no `menu`, so 20 of its 45 runs played another character.
"""

import json
import re
import sys
from pathlib import Path


def names(path):
    return re.findall(r'^name:(.+)$', Path(path).read_text(), re.M)


def main(argv):
    out, data = Path(argv[0]), Path(argv[1])
    races, classes = names(data / 'p_race.txt'), names(data / 'class.txt')
    manifest = dict(l.split('=', 1) for l in (out / 'manifest').read_text().splitlines() if '=' in l)
    bad = []
    for line in Path(manifest['suite_file']).read_text().splitlines():
        body = line.split('#', 1)[0].split()
        if not body:
            continue
        name, race, cls = body[0], int(body[2]), int(body[3])
        flags = body[5].split(',') if len(body) > 5 else []
        summary = out / name / 'summary.json'
        if not summary.exists():
            continue
        s = json.loads(summary.read_text())
        if 'random-birth' in flags:
            continue
        want = (races[race], classes[cls])
        got = (s.get('race'), s.get('class'))
        if got != want:
            bad.append(f'{name}: plays {got[0]} {got[1]}, the line asks for {want[0]} {want[1]}'
                       + ('' if 'menu' in flags else ' (no menu flag)'))
    for b in bad:
        print('check-characters:', b, file=sys.stderr)
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

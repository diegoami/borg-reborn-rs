#!/usr/bin/env python3
"""The engine patch that makes our engine borg-rs's (docs/PILOT.md).

  tools/engine-patch.py OUR_TREE BORG_RS_ANGBAND > engine/patches/08-borg-rs-engine.patch.body

OUR_TREE is our tree after patches 01-07 (`engine/apply-patches.sh` without
08), BORG_RS_ANGBAND borg-rs's `angband/` at the pin. Two parts:

* the game data: every file of `lib/gamedata/` that differs (v12 adds the
  COMPLETE_REST flag to `player_timed.txt`, which the engine's new end of
  a rest reads);
* the code: for every file our build compiles (engine/sources.cmake, plus
  main.c and the headers), the difference from ours to borg-rs's, without:

* borg-rs's recording hooks: borg-rs's files are read as our build sees
  them, with `BORG_RECORDING` undefined (its `#ifdef BORG_RECORDING` regions
  resolved: the `#if` branch removed, an `#else` branch kept);
* `main.h`: patch 01 guards the frontend's declarations on purpose.

Writes a unified diff (`a/src/...`, `b/src/...`) on stdout and the number
of files on stderr. scripts/repin-borg-rs.sh calls it at every re-pin.
"""

import difflib
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IFDEF = re.compile(r'^\s*#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)$')
SKIP = {'main.h'}


def compiled_files(ours):
    names = set(re.findall(r'src/([A-Za-z0-9_/.-]+\.c)', (ROOT / 'engine/sources.cmake').read_text()))
    names.add('main.c')
    names |= {p.name for p in Path(ours).glob('*.h')}
    return sorted(n for n in names if n not in SKIP)


def without_recording(lines):
    """`lines` as the preprocessor sees them with BORG_RECORDING undefined:
    only the conditionals on BORG_RECORDING are resolved, the others kept."""
    out, stack = [], []  # stack: (is_recording_block, keeping)
    for line in lines:
        m = IFDEF.match(line)
        keeping = all(k for _, k in stack)
        if m:
            kw, rest = m.group(1), m.group(2).strip()
            if kw in ('ifdef', 'ifndef', 'if'):
                rec = 'BORG_RECORDING' in rest
                if rec:
                    stack.append((True, kw == 'ifndef'))
                    continue
                stack.append((False, True))
            elif kw == 'else' and stack and stack[-1][0]:
                stack[-1] = (True, not stack[-1][1])
                continue
            elif kw == 'endif' and stack:
                rec, _ = stack.pop()
                if rec:
                    continue
            elif kw == 'elif' and stack and stack[-1][0]:
                raise SystemExit('engine-patch: #elif in a BORG_RECORDING block')
        if keeping:
            out.append(line)
    return out


def main():
    ours_root, theirs_root = sys.argv[1], sys.argv[2]
    out, kept = [], 0
    for f in sorted(p.name for p in Path(theirs_root, 'lib/gamedata').iterdir() if p.is_file()):
        pa, pb = os.path.join(ours_root, 'lib/gamedata', f), os.path.join(theirs_root, 'lib/gamedata', f)
        a = open(pa, encoding='latin-1').read().splitlines(keepends=True) if os.path.exists(pa) else []
        b = open(pb, encoding='latin-1').read().splitlines(keepends=True)
        if a != b:
            kept += 1
            out += difflib.unified_diff(a, b, f'a/lib/gamedata/{f}', f'b/lib/gamedata/{f}', n=3)
    ours, theirs = os.path.join(ours_root, 'src'), os.path.join(theirs_root, 'src')
    for f in compiled_files(ours):
        pa, pb = os.path.join(ours, f), os.path.join(theirs, f)
        if not (os.path.exists(pa) and os.path.exists(pb)):
            continue
        a = open(pa, encoding='latin-1').read().splitlines(keepends=True)
        b = without_recording(open(pb, encoding='latin-1').read().splitlines(keepends=True))
        if a != b:
            kept += 1
            out += difflib.unified_diff(a, b, f'a/src/{f}', f'b/src/{f}', n=3)
    sys.stdout.write(''.join(out))
    print(f'engine-patch: {kept} files differ', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())

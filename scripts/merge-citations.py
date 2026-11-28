#!/usr/bin/env python3
"""Git merge driver for the parameter citations (#1382).

borg-core/src/params.rs and docs/CALIBRATION.md cite the line that reads each
parameter ("borg_item_use.rs:1364"). Any change to a cited file moves those
numbers, so two PRs that touch unrelated code conflict on citation lines.
This driver merges the three versions with the line numbers masked, then puts
the numbers back (ours first, else theirs, else the base). The numbers it
keeps can be stale for the merged sources; run

    python3 scripts/check-param-citations.py --fix

after the merge (CI's check fails until then). Real conflicts (a parameter
changed on both sides) are left as conflict markers, as git does.

Registered by scripts/setup.sh (merge.citations.driver, wired to the two files
in .gitattributes); a clone without it merges them the ordinary way.

    merge-citations.py BASE OURS THEIRS    # the result is written to OURS; exit 1 on a conflict
"""
import re
import subprocess
import sys
import tempfile
from pathlib import Path

# "borg_x.rs:123" in params.rs entries, `borg_x.rs:123` in the CALIBRATION.md tables.
CITE = re.compile(r'(["`])(borg_[a-z0-9_]+\.rs):\d+\1')


def mask(line):
    return CITE.sub(r'\1\2:N\1', line)


def numbers(text):
    """{masked line: [original lines in order]} for the lines with a citation."""
    out = {}
    for line in text.split('\n'):
        masked = mask(line)
        if masked != line:
            out.setdefault(masked, []).append(line)
    return out


def merge(base, ours, theirs):
    """(merged text, conflict count); the citations of the merged lines are restored."""
    with tempfile.TemporaryDirectory() as tmp:
        paths = []
        for name, text in (('ours', ours), ('base', base), ('theirs', theirs)):
            p = Path(tmp, name)
            p.write_text('\n'.join(mask(l) for l in text.split('\n')))
            paths.append(str(p))
        run = subprocess.run(['git', 'merge-file', '-p', '-L', 'HEAD', '-L', 'base', '-L', 'theirs', *paths],
                             capture_output=True, text=True)
    if run.returncode < 0 or run.returncode > 127:
        raise RuntimeError(run.stderr)
    maps = [numbers(t) for t in (ours, theirs, base)]
    seen = {}
    lines = []
    for line in run.stdout.split('\n'):
        for known in maps:
            if line in known:
                # the n-th copy of a masked line takes the n-th original (a repeated line keeps its numbers)
                n = seen.get(line, 0)
                seen[line] = n + 1
                line = known[line][min(n, len(known[line]) - 1)]
                break
        lines.append(line)
    return '\n'.join(lines), run.returncode


def main(argv):
    base_p, ours_p, theirs_p = (Path(a) for a in argv[1:4])
    ours = ours_p.read_text()
    try:
        merged, conflicts = merge(base_p.read_text(), ours, theirs_p.read_text())
        ours_p.write_text(merged)
    except Exception as exc:  # fall back to git's own merge of the unmasked files
        print(f'merge-citations: {exc}; merging as usual', file=sys.stderr)
        conflicts = subprocess.run(['git', 'merge-file', str(ours_p), str(base_p), str(theirs_p)]).returncode
    if conflicts == 0 and ours_p.read_text() != ours:
        print('merge-citations: run python3 scripts/check-param-citations.py --fix', file=sys.stderr)
    return 1 if conflicts else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))

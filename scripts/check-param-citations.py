#!/usr/bin/env python3
"""Check (and fix) the parameter source citations (#601).

Every parameter in borg-core/src/params.rs cites the line that reads it,
"borg_caution.rs:2447", and docs/CALIBRATION.md repeats the citation in its
tables (not the bare "Source line" column of the P2 tables, which cites the
5d9abb8f snapshot their headers name). A citation is right when the cited line mentions `Param::<name>`, or
lies inside a function that does (a table of parameters read in one place,
such as the attack preferences, cites that function's line).

    python3 scripts/check-param-citations.py          # check; exit 1 on any wrong citation
    python3 scripts/check-param-citations.py --fix    # point wrong citations at their first read

--fix rewrites a wrong citation, in params.rs and in the CALIBRATION.md rows
that repeat it, to the first line in the cited file that mentions
`Param::<name>`. A parameter with no read in its cited file is reported and
left alone (the registry test fixes the file by the name's prefix).
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / 'borg-core' / 'src'
PARAMS = SRC / 'params.rs'
CALIBRATION = ROOT / 'docs' / 'CALIBRATION.md'

# name => (default, "unit", min, max, "file.rs:line", "description"),
ENTRY = re.compile(r'^(\s*)([a-z][a-z0-9_]*) => \((.*?), "((borg_[a-z0-9_]+\.rs):(\d+))", ')
# Parameters read in another module through a call the cited file makes: the
# call is their read in the cited file (the registry test fixes the file by
# the name's prefix). The attack preferences are read in
# attack_preference.rs's family table, called from borg_attack (#403).
INDIRECT = {
    'attack_preference_': re.compile(r'\battack_preference::evaluate\b'),
}

FN_START = re.compile(r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+\w+')


def entries(text):
    """(line index in params.rs, name, file, line) of every parameter."""
    out = []
    for i, line in enumerate(text.splitlines()):
        m = ENTRY.match(line)
        if m:
            out.append((i, m.group(2), m.group(5), int(m.group(6))))
    return out


def strip_noise(line, in_block=False):
    """(code, in_block): the line without string, raw-string and char
    literals and comments, so braces inside them do not count. in_block says
    whether a /* */ comment is open at the line's start, and at its end."""
    out, i = [], 0
    while i < len(line):
        if in_block:
            end = line.find('*/', i)
            if end < 0:
                return ''.join(out), True
            i, in_block = end + 2, False
            continue
        raw = re.match(r'b?r(#*)"', line[i:])
        if raw and not (i and (line[i - 1].isalnum() or line[i - 1] == '_')):
            close = '"' + raw.group(1)
            end = line.find(close, i + raw.end())
            out.append('""')
            i = len(line) if end < 0 else end + len(close)
            continue
        m = re.match(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'', line[i:])
        if m:
            out.append('""')
            i += m.end()
            continue
        if line.startswith('//', i):
            break
        if line.startswith('/*', i):
            in_block, i = True, i + 2
            continue
        out.append(line[i])
        i += 1
    return ''.join(out), in_block


def fn_spans(lines):
    """(first, last) 1-based line spans of every fn, from its signature to
    its closing brace (nested fns included)."""
    spans = []
    block = []  # block[i]: a /* */ comment is open at line i's start
    state = False
    for line in lines:
        block.append(state)
        state = strip_noise(line, state)[1]
    for start, line in enumerate(lines):
        if block[start] or not FN_START.match(line):
            continue
        depth, opened, state = 0, False, False
        for j in range(start, len(lines)):
            code, state = strip_noise(lines[j], state)
            if not opened and ';' in code and '{' not in code:
                break  # a declaration without a body
            for ch in code:
                if ch == '{':
                    depth += 1
                    opened = True
                elif ch == '}':
                    depth -= 1
            if opened and depth <= 0:
                spans.append((start + 1, j + 1))
                break
    return spans


class Source:
    def __init__(self, path):
        self.lines = path.read_text().splitlines()
        self.spans = fn_spans(self.lines)
        self._reads = {}

    def reads(self, name):
        """1-based lines that mention Param::<name> (or, for a parameter
        read through another module, the call into it: INDIRECT)."""
        if name not in self._reads:
            pat = re.compile(r'\bParam::' + re.escape(name) + r'\b')
            for prefix, call in INDIRECT.items():
                if name.startswith(prefix):
                    pat = call
            self._reads[name] = [i + 1 for i, l in enumerate(self.lines) if pat.search(l)]
        return self._reads[name]

    def ok(self, name, line):
        reads = self.reads(name)
        if line in reads:
            return True
        return any(a <= line <= b and any(a <= r <= b for r in reads) for a, b in self.spans)


def check(params_text, sources):
    """(wrong, unread): wrong = [(index, name, file, line, first read)],
    unread = [(name, file, line)] for parameters with no read in the file."""
    wrong, unread = [], []
    for index, name, file, line in entries(params_text):
        src = sources(file)
        if src is None or not src.reads(name):
            unread.append((name, file, line))
        elif not src.ok(name, line):
            wrong.append((index, name, file, line, src.reads(name)[0]))
    return wrong, unread


def fix_params(text, wrong):
    lines = text.split('\n')
    for index, name, file, line, read in wrong:
        old = f'"{file}:{line}", '
        assert old in lines[index], (name, lines[index][:80])
        lines[index] = lines[index].replace(old, f'"{file}:{read}", ', 1)
    return '\n'.join(lines)


ROW = re.compile(r'^\| `([a-z][a-z0-9_]*)` \|')
FILE_CITE = re.compile(r'`(borg_[a-z0-9_]+\.rs):(\d+)`')


def registry(params_text):
    """{name: (file, line)} of params.rs."""
    return {name: (file, line) for _, name, file, line in entries(params_text)}


def calibration_rows(text, cited):
    """(row index, name, the row's citation) for every docs/CALIBRATION.md
    table row of a registered parameter that cites `file:line` in the
    parameter's own file. The bare "Source line" column of the P2 tables is
    left out: those tables cite a pinned snapshot (5d9abb8f, stated in their
    headers), not the live tree."""
    out = []
    for i, row in enumerate(text.split('\n')):
        m = ROW.match(row)
        if not m or m.group(1) not in cited:
            continue
        name = m.group(1)
        file, _ = cited[name]
        own = [int(n) for f, n in FILE_CITE.findall(row) if f == file]
        if own:
            out.append((i, name, own[0]))
    return out


def check_calibration(text, cited):
    """[(row index, name, the row's line, the registry's line)] that disagree."""
    return [(i, name, line, cited[name][1]) for i, name, line in calibration_rows(text, cited)
            if line != cited[name][1]]


def fix_calibration(text, cited):
    """Rewrite every disagreeing row to the registry's citation."""
    rows = text.split('\n')
    for i, name, line, want in check_calibration(text, cited):
        file = cited[name][0]
        rows[i] = rows[i].replace(f'`{file}:{line}`', f'`{file}:{want}`')
    return '\n'.join(rows)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--fix', action='store_true', help='point wrong citations at their first read')
    args = ap.parse_args(argv)
    cache = {}

    def sources(file):
        if file not in cache:
            path = SRC / file
            cache[file] = Source(path) if path.is_file() else None
        return cache[file]

    text = PARAMS.read_text()
    wrong, unread = check(text, sources)
    for name, file, line in unread:
        print(f'no read of Param::{name} in {file} (cited {file}:{line})')
    for _, name, file, line, read in wrong:
        print(f'{name}: cites {file}:{line}, first read {file}:{read}')
    if args.fix and wrong:
        text = fix_params(text, wrong)
        PARAMS.write_text(text)
        print(f'fixed {len(wrong)} citations in {PARAMS.relative_to(ROOT)}')
        wrong = check(text, sources)[0]
    cited = registry(text)
    calibration = CALIBRATION.read_text()
    rows = check_calibration(calibration, cited)
    for i, name, line, want in rows:
        print(f'{CALIBRATION.relative_to(ROOT)}:{i + 1}: {name} cites line {line}, params.rs {cited[name][0]}:{want}')
    if args.fix and rows:
        calibration = fix_calibration(calibration, cited)
        CALIBRATION.write_text(calibration)
        print(f'fixed {len(rows)} rows in {CALIBRATION.relative_to(ROOT)}')
        rows = check_calibration(calibration, cited)
    print(f'{len(cited)} parameters: {len(wrong)} wrong citations, {len(unread)} with no read in the cited file; '
          f'{len(rows)} CALIBRATION.md rows that disagree')
    return 1 if wrong or unread or rows else 0


if __name__ == '__main__':
    sys.exit(main())

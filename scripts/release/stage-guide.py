#!/usr/bin/env python3
"""Stage the play guide for a release package (#817, review of #1386).

  scripts/release/stage-guide.py SRC OUT COMMIT

The package ships docs/user/GUIDE.md as docs/GUIDE.md, where its links
relative to the repository (../../README.md, ../CALIBRATION.md, ...) would
point outside the package or at files it does not ship. Every relative link
becomes a GitHub link to the same file at COMMIT, the commit the package is
built from; anchors within the guide and external links are kept. Links in
inline code and fenced code are left alone. Exits 1 if a link names a file
that does not exist in the repository.
"""
import re
import sys
from pathlib import Path

REPO_URL = 'https://github.com/diegoami/borg-reborn-rs'
ROOT = Path(__file__).resolve().parents[2]
LINK = re.compile(r'\]\(([^)\s]+)\)')


def rewrite(text, src, commit):
    """The guide's text with its relative links made absolute, and the list
    of targets that do not exist."""
    base = src.resolve().parent
    missing = []

    def absolute(match):
        target = match.group(1)
        if target.startswith('#') or '://' in target or target.startswith('mailto:'):
            return match.group(0)
        path, _, anchor = target.partition('#')
        resolved = (base / path).resolve()
        try:
            rel = resolved.relative_to(ROOT).as_posix()
        except ValueError:
            missing.append(target)
            return match.group(0)
        if not resolved.exists():
            missing.append(target)
        kind = 'tree' if resolved.is_dir() else 'blob'
        url = f'{REPO_URL}/{kind}/{commit}/{rel}' + (f'#{anchor}' if anchor else '')
        return f']({url})'

    out = []
    fenced = False
    for line in text.splitlines(keepends=True):
        if line.lstrip().startswith('```'):
            fenced = not fenced
            out.append(line)
            continue
        if fenced:
            out.append(line)
            continue
        # Inline code spans are kept as they are: odd pieces of a split on
        # backticks are code.
        parts = line.split('`')
        for i in range(0, len(parts), 2):
            parts[i] = LINK.sub(absolute, parts[i])
        out.append('`'.join(parts))
    return ''.join(out), missing


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    src, dest, commit = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
    if not re.fullmatch(r'[0-9a-f]{7,40}', commit):
        sys.exit(f'stage-guide: bad commit {commit!r}')
    text, missing = rewrite(src.read_text(), src, commit)
    if missing:
        sys.exit('stage-guide: links to files not in the repository: ' + ', '.join(missing))
    dest.write_text(text)


if __name__ == '__main__':
    main()

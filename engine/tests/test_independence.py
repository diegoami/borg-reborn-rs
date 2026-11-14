"""No build, test or check reads borg-rs (docs/SPEC.md §2, §10 exit point 8;
task TE.9, #79).

Code may name borg-rs where it records where something was copied from (a
comment); it may not use a borg-rs checkout, crate, tree or build. The
reference tooling (scripts/: the refresh report reads every pinned source,
borg-rs among them, as a reference) is not a build or a check of the Borg.

Run: python3 -m unittest discover -s engine/tests
"""

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCOPE = ['engine', 'harness', 'tools', 'borg-reborn-rs', '.github', 'Cargo.toml', 'Cargo.lock', 'watch-borg.sh']
SUFFIXES = {'.py', '.sh', '.rs', '.c', '.h', '.yml', '.yaml', '.toml', '.cmake', '.txt', '.lock', ''}
SKIP = {'engine/angband', 'harness/out', 'harness/reference', 'engine/tests/test_independence.py'}
# Uses of borg-rs, as opposed to mentions of where a file came from.
USES = [
    (r'vendor/borg-rs\b', 'a borg-rs checkout'),
    (r'-p\s+borg-rs\b|--package\s+borg-rs\b|name\s*=\s*"borg-rs"', 'the borg-rs crate'),
    (r'\blibborg_rs\b', "borg-rs's staticlib"),
    (r'(?<![\w-])angband/(src|CMakeLists|lib)\b', "borg-rs's former engine tree at angband/"),
    (r'github\.com/diegoami/borg-rs(\.git)?\b', 'a borg-rs clone or download'),
]


def files():
    for top in SCOPE:
        p = ROOT / top
        paths = [p] if p.is_file() else sorted(q for q in p.rglob('*') if q.is_file())
        for q in paths:
            rel = q.relative_to(ROOT).as_posix()
            if any(rel == s or rel.startswith(s + '/') for s in SKIP) or '__pycache__' in rel or '/target/' in rel:
                continue
            if q.suffix in SUFFIXES or q.name == 'CMakeLists.txt':
                yield rel, q


def code_lines(text):
    """Lines that are not comments (#, //, /* */ and doc comments)."""
    for n, line in enumerate(text.splitlines(), 1):
        s = line.strip()
        if s.startswith(('#', '//', '/*', '*', '"""', "'''")) or not s:
            continue
        yield n, line


class IndependenceTest(unittest.TestCase):
    def test_no_build_test_or_check_uses_borg_rs(self):
        found = []
        for rel, path in files():
            try:
                text = path.read_text()
            except UnicodeDecodeError:
                continue
            for n, line in code_lines(text):
                # Paths into our own pristine tree are ours.
                line = line.replace('engine/angband/', 'engine/ANGBAND/')
                for pattern, what in USES:
                    if re.search(pattern, line):
                        found.append(f'{rel}:{n}: {what}: {line.strip()[:100]}')
        self.assertEqual(found, [], '\n'.join(found))

    def test_the_workspace_has_only_our_crates(self):
        # borg-core is borg-rs's crate vendored at a pin (the option-C pilot,
        # #207, owner 2026-11-14): a copy in this repository (docs/SPEC.md
        # §2.2, docs/PROVENANCE.md), not a borg-rs checkout or crate.
        self.assertRegex((ROOT / 'Cargo.toml').read_text(), r'members = \["borg-reborn-rs", "borg-core"\]')
        self.assertFalse((ROOT / 'borg-rs').exists())
        self.assertFalse((ROOT / 'angband').exists())


if __name__ == '__main__':
    unittest.main()

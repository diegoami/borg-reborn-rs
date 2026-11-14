"""The engine tree and the patch series (docs/ENGINE.md; task TE.1, #71).

Run: python3 -m unittest discover -s engine/tests
"""

import re
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENGINE = ROOT / 'engine'
UPSTREAM = dict(l.split('=', 1) for l in (ENGINE / 'UPSTREAM').read_text().splitlines() if '=' in l)
GUARDS = ('BORG_HARNESS', 'USE_BORGH')


def git(*args):
    return subprocess.run(['git', '-C', str(ROOT), *args], capture_output=True, text=True)


def strip_guarded(text):
    """The file as a compiler sees it with the harness guards undefined."""
    return re.sub(r'#ifdef (?:%s)\n(?:.*\n)*?#endif[^\n]*\n' % '|'.join(GUARDS), '', text)


def unguarded_patches():
    """The patches that change the game on purpose, each declaring it in its
    header with 'Not guarded:' and the reason. Today only 08, borg-rs's engine
    (owner, 2026-11-14), until the shared engine repository replaces it."""
    return {p.name for p in (ENGINE / 'patches').glob('*.patch')
            if 'Not guarded:' in p.read_text().split('--- a/', 1)[0]}


def code(text):
    """The text without its empty lines: blank lines are not code."""
    return '\n'.join(l for l in text.splitlines() if l.strip())


class EngineTreeTest(unittest.TestCase):
    def test_the_tree_is_upstreams_bytes(self):
        tree = git('rev-parse', 'HEAD:engine/angband').stdout.strip()
        self.assertEqual(tree, UPSTREAM['tree'], 'engine/angband/ differs from upstream e473720 at HEAD')
        dirty = git('status', '--porcelain', '--', 'engine/angband').stdout.strip()
        self.assertEqual(dirty, '', 'engine/angband/ has local changes')

    def test_the_recorded_tree_is_the_commits(self):
        vendor = ROOT / 'vendor/angband'
        if not (vendor / '.git').exists():
            # CI fetches the commit and requires the check (E review, finding 7).
            if os.environ.get('ENGINE_UPSTREAM_REQUIRED'):
                self.fail('no vendor/angband checkout, and ENGINE_UPSTREAM_REQUIRED is set')
            self.skipTest('no vendor/angband checkout (scripts/setup.sh)')
        r = subprocess.run(['git', '-C', str(vendor), 'rev-parse', UPSTREAM['commit'] + '^{tree}'],
                           capture_output=True, text=True)
        self.assertEqual(r.stdout.strip(), UPSTREAM['tree'])


class PatchSeriesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dest = Path(cls.tmp.name) / 'tree'
        cls.applied = subprocess.run([str(ENGINE / 'apply-patches.sh'), str(cls.dest)], capture_output=True, text=True)
        # The guarded series on its own: what the guard check reads.
        cls.guarded = Path(cls.tmp.name) / 'guarded'
        subprocess.run(['cp', '-a', str(ENGINE / 'angband'), str(cls.guarded)], check=True)
        for p in sorted((ENGINE / 'patches').glob('*.patch')):
            if p.name not in unguarded_patches():
                subprocess.run(['patch', '-s', '-p1', '-d', str(cls.guarded), '--forward',
                                '--no-backup-if-mismatch'], stdin=p.open(), check=True)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_series_applies(self):
        self.assertEqual(self.applied.returncode, 0, self.applied.stderr)

    def test_every_patch_says_why(self):
        for p in sorted((ENGINE / 'patches').glob('*.patch')):
            head = p.read_text().split('--- a/', 1)[0]
            self.assertTrue('Guarded by' in head or 'Not guarded:' in head, p.name)

    def test_only_the_engine_patch_is_unguarded(self):
        self.assertLessEqual(unguarded_patches(), {'08-borg-rs-engine.patch'})

    def test_with_the_guards_off_the_files_are_upstreams(self):
        # Every changed line sits inside an #ifdef BORG_HARNESS / USE_BORGH
        # block (blank lines aside), except one declared change: mon-make.c splits a declaration
        # with an initialiser into a declaration and an assignment, so the
        # guarded line can go between them. Same code with the guards off.
        declared = {'src/mon-make.c': ('\tstruct tm *date = localtime(&cur_time);\n',
                                       '\tstruct tm *date;\n\n\tdate = localtime(&cur_time);\n')}
        changed = set()
        for p in sorted((ENGINE / 'patches').glob('*.patch')):
            if p.name not in unguarded_patches():
                changed.update(re.findall(r'^\+\+\+ b/(\S+)', p.read_text(), re.M))
        self.assertTrue(changed)
        for rel in sorted(changed):
            upstream = (ENGINE / 'angband' / rel).read_text()
            patched = strip_guarded((self.guarded / rel).read_text())
            if rel in declared:
                old, new = declared[rel]
                upstream = upstream.replace(old, new)
            self.assertEqual(code(patched), code(upstream), f'{rel}: an unguarded change')

    def test_the_pristine_tree_is_untouched(self):
        dirty = git('status', '--porcelain', '--', 'engine/angband').stdout.strip()
        self.assertEqual(dirty, '')


if __name__ == '__main__':
    unittest.main()

"""scripts/borg-core-vendor.py (review #227, finding 9). Run: python3 -m unittest discover -s scripts/tests"""

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('borg_core_vendor', HERE.parent / 'borg-core-vendor.py')
bv = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bv)


def git(repo, *args):
    return subprocess.run(['git', '-C', str(repo), *args], check=True, capture_output=True, text=True).stdout.strip()


def commit(repo, files, message):
    for rel, text in files.items():
        (repo / rel).parent.mkdir(parents=True, exist_ok=True)
        (repo / rel).write_text(text)
    git(repo, 'add', '-A')
    git(repo, '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-qm', message)
    return git(repo, 'rev-parse', 'HEAD')


LINES = ''.join(f'line {i}\n' for i in range(1, 8))


class VendorTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        self.repo, core = t / 'borg-rs', t / 'borg-core'
        self.repo.mkdir()
        git(self.repo, 'init', '-q')
        self.pin = commit(self.repo, {'borg-rs/src/a.rs': LINES, 'borg-rs/src/b.rs': 'b\n',
                                      'harness/borg.txt': 'x = 1\n', 'angband/src/borg/borg.txt': 'x = 1\n',
                                      'angband/src/borg/borg-init.c': 'int x;\n'}, 'pin')
        files = {'src/a.rs': LINES.replace('line 2\n', 'line 2 // borg-reborn-rs (#208) hook\n'),
                 'src/b.rs': 'b\n', 'borg.txt': 'x = 1\n', 'c-reference/borg.txt': 'x = 1\n',
                 'c-reference/borg-init.c': 'int x;\n', 'src/shell.rs': 'ours\n', 'ladders.txt': 'l\n',
                 'PIN': self.pin + '\n'}
        for rel, text in files.items():
            (core / rel).parent.mkdir(parents=True, exist_ok=True)
            (core / rel).write_text(text)
        self.core = core
        self.saved = (bv.CORE, bv.HOOKED)
        bv.CORE, bv.HOOKED = core, {'src/a.rs'}

    def tearDown(self):
        bv.CORE, bv.HOOKED = self.saved
        self.tmp.cleanup()

    def test_check_and_delta(self):
        self.assertEqual(bv.check(self.repo), [])
        d = bv.delta(self.repo)
        self.assertIn('+line 2 // borg-reborn-rs (#208) hook', d)
        self.assertIn('+++ b/borg-core/src/shell.rs', d)
        (self.core / 'src/b.rs').write_text('changed\n')
        self.assertEqual(bv.check(self.repo), ['src/b.rs: differs from borg-rs at the pin, and not declared hooked'])

    def test_refresh_merges_both_sides(self):
        new = commit(self.repo, {'borg-rs/src/a.rs': LINES.replace('line 6\n', 'line 6 upstream\n'),
                                 'borg-rs/src/c.rs': 'new file\n'}, 'later')
        self.assertEqual(bv.refresh(self.repo, new), [])
        a = (self.core / 'src/a.rs').read_text()
        self.assertIn('line 2 // borg-reborn-rs (#208) hook', a, 'our hook stays')
        self.assertIn('line 6 upstream', a, "borg-rs's change comes in")
        self.assertEqual((self.core / 'src/c.rs').read_text(), 'new file\n')
        self.assertEqual((self.core / 'src/shell.rs').read_text(), 'ours\n')
        self.assertEqual((self.core / 'PIN').read_text().strip(), new)

    def test_refresh_reports_a_conflict(self):
        new = commit(self.repo, {'borg-rs/src/a.rs': LINES.replace('line 2\n', 'line 2 upstream\n')}, 'clash')
        self.assertEqual(bv.refresh(self.repo, new), ['src/a.rs: conflict, marked in the file'])
        self.assertIn('<<<<<<< ours', (self.core / 'src/a.rs').read_text())


if __name__ == '__main__':
    unittest.main()

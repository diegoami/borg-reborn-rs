"""scripts/borg-core-vendor.py (review #227, finding 9). Run: python3 -m unittest discover -s scripts/tests"""

import importlib.util
import json
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
        self.saved = (bv.CORE, bv.HOOKED, bv.HOOK_LOCK)
        bv.CORE, bv.HOOKED = core, {'src/a.rs'}
        bv.HOOK_LOCK = t / 'hooks.json'
        self.record_hooks()

    def tearDown(self):
        bv.CORE, bv.HOOKED, bv.HOOK_LOCK = self.saved
        self.tmp.cleanup()

    def record_hooks(self):
        bv.HOOK_LOCK.write_text(json.dumps(bv.hook_manifest(self.repo)))

    def test_undeclared_hook_content_change_is_rejected(self):
        (self.core / 'src/a.rs').write_text('arbitrary policy replacement\n')
        self.assertIn('src/a.rs: hooked content differs from reviewed manifest', bv.check(self.repo))

    def test_missing_manifest_and_wrong_pin_are_rejected(self):
        bv.HOOK_LOCK.unlink()
        self.assertTrue(any('hook manifest' in p for p in bv.check(self.repo)))
        self.record_hooks()
        lock = json.loads(bv.HOOK_LOCK.read_text());lock['donor_pin'] = '0' * 40
        bv.HOOK_LOCK.write_text(json.dumps(lock))
        self.assertIn('hook manifest: schema or donor pin differs', bv.check(self.repo))

    def test_hook_inventory_and_donor_blob_are_checked(self):
        lock = json.loads(bv.HOOK_LOCK.read_text());lock['files']['extra.rs'] = {}
        bv.HOOK_LOCK.write_text(json.dumps(lock))
        self.assertIn('hook manifest: declared file inventory differs', bv.check(self.repo))
        self.record_hooks()
        lock = json.loads(bv.HOOK_LOCK.read_text());lock['files']['src/a.rs']['donor_blob'] = '0' * 40
        bv.HOOK_LOCK.write_text(json.dumps(lock))
        self.assertIn('src/a.rs: hooked donor blob differs from manifest', bv.check(self.repo))

    def test_refresh_does_not_implicitly_accept_new_hook_bytes(self):
        before = bv.HOOK_LOCK.read_bytes()
        new = commit(self.repo, {'borg-rs/src/a.rs': LINES.replace('line 6\n','line 6 new\n')}, 'new donor')
        self.assertEqual(bv.refresh(self.repo, new), [])
        self.assertEqual(bv.HOOK_LOCK.read_bytes(), before)
        self.assertTrue(bv.check(self.repo))
        self.record_hooks()
        self.assertEqual(bv.check(self.repo), [])

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

    def test_a_file_removed_on_both_sides_is_no_conflict(self):
        (self.core / 'src/b.rs').unlink()
        git(self.repo, 'rm', '-q', 'borg-rs/src/b.rs')
        new = commit(self.repo, {}, 'drop b')
        self.assertEqual(bv.refresh(self.repo, new), [])
        self.assertFalse((self.core / 'src/b.rs').exists())

    def test_refresh_reports_a_conflict(self):
        new = commit(self.repo, {'borg-rs/src/a.rs': LINES.replace('line 2\n', 'line 2 upstream\n')}, 'clash')
        self.assertEqual(bv.refresh(self.repo, new), ['src/a.rs: conflict, marked in the file'])
        self.assertIn('<<<<<<< ours', (self.core / 'src/a.rs').read_text())


if __name__ == '__main__':
    unittest.main()

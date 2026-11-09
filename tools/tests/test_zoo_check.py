"""Tests for harness/zoo-check.py's zoo identity check. Run: python3 -m unittest discover -s tools/tests"""

import importlib.machinery
import importlib.util
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('zoo_check', HERE.parent.parent / 'harness/zoo-check.py')
zc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(zc)


def git(repo, *args):
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True).strip()


class ZooIdentityTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.zoo = Path(self.tmp.name)
        git(self.zoo, 'init', '-q')
        (self.zoo / 'zoo').mkdir()
        (self.zoo / 'zoo' / 'borgrs.py').write_text('ADAPTER = 1\n')
        (self.zoo / '.gitignore').write_text('__pycache__/\n')
        git(self.zoo, 'add', '-A')
        git(self.zoo, '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '-qm', 'pin')
        self.pin = git(self.zoo, 'rev-parse', 'HEAD')

    def tearDown(self):
        self.tmp.cleanup()

    def test_clean_pin_passes_and_ignored_files_do_not_count(self):
        (self.zoo / 'zoo' / '__pycache__').mkdir()
        (self.zoo / 'zoo' / '__pycache__' / 'borgrs.pyc').write_bytes(b'x')
        self.assertIsNone(zc.zoo_identity_error(self.zoo, self.pin))

    def test_other_commit_is_refused(self):
        self.assertIn('pin', zc.zoo_identity_error(self.zoo, '0' * 40))

    def test_modified_adapter_at_the_pin_is_refused(self):
        (self.zoo / 'zoo' / 'borgrs.py').write_text('ADAPTER = 2\n')
        self.assertIn('changes at its pin', zc.zoo_identity_error(self.zoo, self.pin))

    def test_untracked_module_at_the_pin_is_refused(self):
        (self.zoo / 'zoo' / 'extra.py').write_text('')
        self.assertIn('changes at its pin', zc.zoo_identity_error(self.zoo, self.pin))


    def test_import_reads_the_pinned_tree_not_cached_bytecode(self):
        # Re-review R4: an ignored .pyc in the checkout is not the pinned code.
        cache = self.zoo / 'zoo' / '__pycache__'
        cache.mkdir()
        (cache / f'borgrs.{sys.implementation.cache_tag}.pyc').write_bytes(b'tampered')
        (self.zoo / 'zoo' / f'borgrs{importlib.machinery.EXTENSION_SUFFIXES[0]}').write_bytes(b'tampered')
        with tempfile.TemporaryDirectory() as out:
            tree = zc.pinned_tree(self.zoo, self.pin, out)
            files = sorted(str(f.relative_to(tree)) for f in tree.rglob('*') if f.is_file())
            self.assertEqual(files, ['.gitignore', 'zoo/borgrs.py'])

    def test_an_already_imported_zoo_is_refused(self):
        sys.modules['zoo'] = types.ModuleType('zoo')
        try:
            with self.assertRaises(SystemExit):
                zc.import_zoo(self.zoo)
        finally:
            del sys.modules['zoo']


if __name__ == '__main__':
    unittest.main()

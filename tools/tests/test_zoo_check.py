"""Tests for harness/zoo-check.py's zoo identity check. Run: python3 -m unittest discover -s tools/tests"""

import importlib.util
import subprocess
import tempfile
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


if __name__ == '__main__':
    unittest.main()

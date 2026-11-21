"""Tests for scripts/check-reference.py on a synthetic pinned repository.

Run: python3 -m unittest discover -s scripts/tests
"""

import contextlib
import importlib.util
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('check_reference', HERE.parent / 'check-reference.py')
cr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cr)

ENV = dict(os.environ, GIT_AUTHOR_NAME='t', GIT_AUTHOR_EMAIL='t@t', GIT_COMMITTER_NAME='t',
           GIT_COMMITTER_EMAIL='t@t', GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1')


def sh(cwd, *args):
    return subprocess.run(args, cwd=cwd, env=ENV, capture_output=True, text=True, check=True).stdout.strip()


class CheckReference(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / 'scripts').mkdir()
        (self.root / 'scripts/refresh-sources.py').write_bytes((HERE.parent / 'refresh-sources.py').read_bytes())
        source = self.root / 'vendor/src'
        source.mkdir(parents=True)
        sh(source, 'git', 'init', '-q')
        (source / 'docs').mkdir()
        (source / 'docs/A.md').write_text('a\n')
        (source / 'docs/B.md').write_text('b\n')
        (source / 'docs/data.json').write_text('{}\n')
        sh(source, 'git', 'add', '-A')
        sh(source, 'git', 'commit', '-q', '-m', 'one')
        commit = sh(source, 'git', 'rev-parse', 'HEAD')
        pins = {'pins': {'src': {'repository': 'o/src', 'commit': commit}},
                'copies': [{'pin': 'src', 'from': 'docs', 'to': 'reference/src',
                            'select': 'suffix', 'suffixes': ['.md']}]}
        (self.root / 'pins.json').write_text(json.dumps(pins))
        (self.root / 'reference/src').mkdir(parents=True)
        (self.root / 'reference/README.md').write_text('ours\n')
        (self.root / 'reference/src/A.md').write_text('a\n')
        (self.root / 'reference/src/B.md').write_text('b\n')

    def tearDown(self):
        self.tmp.cleanup()

    def run_check(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = cr.main(['--root', str(self.root)])
        return status, out.getvalue()

    def test_exact_copies_pass(self):
        status, out = self.run_check()
        self.assertEqual(status, 0, out)
        self.assertIn('2 copies checked, 0 problems', out)

    def test_changed_copy_fails(self):
        (self.root / 'reference/src/A.md').write_text('edited\n')
        status, out = self.run_check()
        self.assertEqual(status, 1)
        self.assertIn('DIFFERS  reference/src/A.md', out)

    def test_missing_and_extra_fail(self):
        (self.root / 'reference/src/B.md').unlink()
        (self.root / 'reference/src/ours.md').write_text('not a copy\n')
        status, out = self.run_check()
        self.assertEqual(status, 1)
        self.assertIn('MISSING  reference/src/B.md', out)
        self.assertIn('EXTRA    reference/src/ours.md', out)

    def test_missing_clone_is_its_own_status(self):
        subprocess.run(['rm', '-rf', str(self.root / 'vendor/src')], check=True)
        status, out = self.run_check()
        self.assertEqual(status, 2)
        self.assertIn('run scripts/setup.sh', out)


if __name__ == '__main__':
    unittest.main()

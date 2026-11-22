"""Forward only pinned Cemetery native bytes, including outside CI."""
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cemetery_tools import locate, forward


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), '-c', 'user.name=Test',
                                    '-c', 'user.email=test@example.invalid', *args],
                                   stderr=subprocess.DEVNULL).decode().strip()


def fixture(root, cemetery_name='cemetery', source_name='source'):
    repo = root / source_name
    repo.mkdir()
    cemetery = root / cemetery_name
    native = cemetery / 'scripts/native'
    native.mkdir(parents=True)
    (native / 'API_VERSION').write_text('1\n')
    (native / 'example.py').write_text('def owner():\n    return SOURCE_ROOT\n')
    git(cemetery, 'init', '-q')
    git(cemetery, 'add', '.')
    git(cemetery, 'commit', '-qm', 'pinned native implementation')
    pin = git(cemetery, 'rev-parse', 'HEAD')
    (repo / 'death-tools-pin.txt').write_text(pin + '\n')
    return repo, cemetery, native, pin


class CemeteryPointerTest(unittest.TestCase):
    def test_explicit_destination_and_api_guard(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, cemetery, native, _ = fixture(Path(tmp))
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(cemetery)):
                self.assertEqual(locate(repo), cemetery)
                (native / 'API_VERSION').write_text('2\n')
                with self.assertRaisesRegex(RuntimeError, 'API 1'):
                    locate(repo)

    def test_linked_worktree_finds_sibling_of_primary_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, cemetery, _, _ = fixture(root, 'borg-cemetery', 'borg-reborn-rs')
            worktree = repo / '.worktrees/task'
            worktree.mkdir(parents=True)
            (worktree / 'death-tools-pin.txt').write_bytes((repo / 'death-tools-pin.txt').read_bytes())
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                self.assertEqual(locate(worktree), cemetery)

    def test_ci_pin_is_found_when_policy_environment_is_cleared(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, cemetery, _, _ = fixture(root, 'source/vendor/borg-cemetery')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                self.assertEqual(locate(repo), cemetery)

    def sibling_and_vendor(self, root):
        """A primary checkout with a sibling Cemetery whose main has moved its
        native tools past the pin, and a vendor/ clone at the pin (#902)."""
        repo, sibling, native, pin = fixture(root, 'borg-cemetery', 'borg-reborn-rs')
        vendor = repo / 'vendor/borg-cemetery'
        vendor.parent.mkdir()
        git(root, 'clone', '-q', str(sibling), str(vendor))
        git(vendor, 'checkout', '-q', '--detach', pin)
        (native / 'example.py').write_text('def owner():\n    return "later"\n')
        git(sibling, 'commit', '-qam', 'native tools move past the pin')
        return repo, sibling, vendor

    def test_sibling_past_the_pin_falls_back_to_the_pinned_vendor_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, sibling, vendor = self.sibling_and_vendor(Path(tmp))
            stderr = io.StringIO()
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'), \
                    patch('sys.stderr', stderr):
                self.assertEqual(locate(repo), vendor)
            self.assertIn(f'using {vendor}', stderr.getvalue())
            self.assertIn(f'{sibling} does not match', stderr.getvalue())

    def test_a_sibling_at_the_pin_is_still_preferred(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, sibling, _, _ = fixture(root, 'borg-cemetery', 'borg-reborn-rs')
            vendor = repo / 'vendor/borg-cemetery'
            vendor.parent.mkdir()
            git(root, 'clone', '-q', str(sibling), str(vendor))
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                self.assertEqual(locate(repo), sibling)

    def test_an_explicit_checkout_past_the_pin_never_falls_back(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, sibling, _ = self.sibling_and_vendor(Path(tmp))
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(sibling)), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                with self.assertRaisesRegex(RuntimeError, 'refusing to execute'):
                    locate(repo)

    def test_no_matching_checkout_names_the_setup_command(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, _, vendor = self.sibling_and_vendor(Path(tmp))
            (vendor / 'scripts/native/unexpected.py').write_text('raise AssertionError()\n')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch(
                    'cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                with self.assertRaisesRegex(RuntimeError, 'refusing to execute.*scripts/setup.sh'):
                    locate(repo)

    def test_forwarded_functions_keep_the_actual_consumer_context(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, cemetery, _, _ = fixture(Path(tmp))
            namespace = dict(__file__=str(repo / 'tools/example.py'), __name__='test')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(cemetery)):
                forward('example.py', namespace)
            self.assertEqual(namespace['owner'](), repo)

    def test_later_docs_head_with_identical_native_bytes_is_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, cemetery, _, pin = fixture(Path(tmp))
            (cemetery / 'README.md').write_text('later evidence and docs\n')
            git(cemetery, 'add', '.')
            git(cemetery, 'commit', '-qm', 'docs-only head')
            self.assertNotEqual(git(cemetery, 'rev-parse', 'HEAD'), pin)
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(cemetery)):
                self.assertEqual(locate(repo), cemetery)

    def test_stale_api_1_and_dirty_native_code_cannot_execute(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo, cemetery, native, _ = fixture(Path(tmp))
            marker = Path(tmp) / 'natural-registry'
            unsafe = f'from pathlib import Path\nPath({str(marker)!r}).mkdir()\n'
            (native / 'example.py').write_text(unsafe)
            namespace = dict(__file__=str(repo / 'tools/example.py'), __name__='__main__')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(cemetery)):
                for committed in (False, True):
                    if committed:
                        git(cemetery, 'add', '.')
                        git(cemetery, 'commit', '-qm', 'different API-1 code')
                    with self.assertRaisesRegex(SystemExit, 'refusing to execute'):
                        forward('example.py', namespace)
                    self.assertFalse(marker.exists())

    def test_missing_pin_object_file_and_untracked_source_are_refused(self):
        for change in ('pin', 'file', 'extra'):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as tmp:
                repo, cemetery, native, _ = fixture(Path(tmp))
                if change == 'pin':
                    (repo / 'death-tools-pin.txt').write_text('f' * 40 + '\n')
                elif change == 'file':
                    (native / 'example.py').unlink()
                else:
                    (native / 'unexpected.py').write_text('raise AssertionError()\n')
                with patch.dict(os.environ, BORG_CEMETERY_DIR=str(cemetery)):
                    with self.assertRaisesRegex(RuntimeError, 'refusing to execute'):
                        locate(repo)


if __name__ == '__main__':
    unittest.main()

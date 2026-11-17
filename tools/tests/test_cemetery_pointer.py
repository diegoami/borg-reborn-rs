"""Forward only pinned Cemetery native bytes, including outside CI."""
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


    def test_extra_replay_settings_apply_before_canonical_cli_dispatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, cemetery, native, _ = fixture(root)
            (native / 'rerun.py').write_text(
                'SETTINGS = ("BORG_CORE",)\n'
                'def settings(): return SETTINGS\n'
                'def main(): return 0 if SETTINGS == ("BORG_SHADOW", "BORG_CORE") else 7\n'
                'if __name__ == "__main__": raise SystemExit(main())\n')
            git(cemetery, 'add', '.')
            git(cemetery, 'commit', '-qm', 'pin replay fixture')
            (repo / 'death-tools-pin.txt').write_text(git(cemetery, 'rev-parse', 'HEAD') + '\n')
            for name in ('test_module', '__main__'):
                namespace = dict(__file__=str(root / 'source/tools/rerun.py'), __name__=name)
                with patch.dict(os.environ, BORG_CEMETERY_DIR=str(root / 'cemetery')):
                    if name == '__main__':
                        with self.assertRaises(SystemExit) as stopped:
                            forward('rerun.py', namespace, extra_settings=('BORG_SHADOW',))
                        self.assertEqual(stopped.exception.code, 0)
                    else:
                        forward('rerun.py', namespace, extra_settings=('BORG_SHADOW',))
                self.assertEqual(namespace['settings'](), ('BORG_SHADOW', 'BORG_CORE'))


if __name__ == '__main__':
    unittest.main()

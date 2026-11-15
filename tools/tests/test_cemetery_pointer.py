"""Compatibility pointers locate cemetery-owned code without copying it."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cemetery_tools import locate, forward


class CemeteryPointerTest(unittest.TestCase):
    def test_explicit_destination_and_api_guard(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            native = root / 'cemetery/scripts/native'
            native.mkdir(parents=True)
            (native / 'API_VERSION').write_text('1\n')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(root / 'cemetery')):
                self.assertEqual(locate(root / 'source'), root / 'cemetery')
                (native / 'API_VERSION').write_text('2\n')
                with self.assertRaisesRegex(RuntimeError, 'API 1'):
                    locate(root / 'source')

    def test_linked_worktree_finds_sibling_of_primary_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            native = root / 'borg-cemetery/scripts/native'
            native.mkdir(parents=True)
            (native / 'API_VERSION').write_text('1\n')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch('cemetery_tools.subprocess.check_output', return_value=str(root / 'borg-reborn-rs/.git') + '\n'):
                self.assertEqual(locate(root / 'borg-reborn-rs/.worktrees/task'), root / 'borg-cemetery')

    def test_ci_pin_is_found_when_policy_environment_is_cleared(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo = root / 'source'
            native = repo / 'vendor/borg-cemetery/scripts/native'
            native.mkdir(parents=True)
            (native / 'API_VERSION').write_text('1\n')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=''), patch('cemetery_tools.subprocess.check_output', return_value=str(repo / '.git') + '\n'):
                self.assertEqual(locate(repo), repo / 'vendor/borg-cemetery')

    def test_forwarded_functions_keep_the_actual_consumer_context(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            native = root / 'cemetery/scripts/native'
            native.mkdir(parents=True)
            (native / 'API_VERSION').write_text('1\n')
            (native / 'example.py').write_text('def owner():\n    return SOURCE_ROOT\n')
            namespace = dict(__file__=str(root / 'source/tools/example.py'), __name__='test')
            with patch.dict(os.environ, BORG_CEMETERY_DIR=str(root / 'cemetery')):
                forward('example.py', namespace)
            self.assertEqual(namespace['owner'](), root / 'source')

    def test_extra_replay_settings_apply_before_canonical_cli_dispatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            native = root / 'cemetery/scripts/native'
            native.mkdir(parents=True)
            (native / 'API_VERSION').write_text('1\n')
            (native / 'rerun.py').write_text(
                'SETTINGS = ("BORG_CORE",)\n'
                'def settings(): return SETTINGS\n'
                'def main(): return 0 if SETTINGS == ("BORG_SHADOW", "BORG_CORE") else 7\n'
                'if __name__ == "__main__": raise SystemExit(main())\n')
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

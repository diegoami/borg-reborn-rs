"""Actual native startup with missing, malformed and unreadable borg.txt files."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game'


@unittest.skipUnless((GAME / 'angband').is_file(), 'native build required')
class BorgFileConfigTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='borg-file-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def launch(self, name, content=None, overrides=None, directory=False, raw_user=False):
        out = self.root / name
        out.mkdir()
        for kind in ['user', 'save', 'archive', 'panic', 'scores']:
            (out / kind).mkdir()
        user = out / 'user'
        if raw_user:
            user = out / os.fsdecode(b'user-\xff')
            user.mkdir()
        config = user / 'borg.txt'
        if directory:
            config.mkdir()
        elif content is not None:
            config.write_bytes(content)
        argv = ['./angband', '-mborgh', '-n', '-uBorg']
        argv += [f'-d{kind}={user if kind == "user" else out / kind}'
                 for kind in ['user', 'save', 'archive', 'panic', 'scores']]
        argv += ['--', '-seed', '29', '-race', '0', '-class', '0', '-turns', '200',
                 '-keylog', str(out / 'keys.log'), '-summary', str(out / 'summary.json'),
                 '-statelog', str(out / 'state.log'), '-menu-letters', '-exact-turns']
        env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
        env.update(TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
        env.update(overrides or {})
        result = subprocess.run(argv, cwd=GAME, env=env, text=True,
                                capture_output=True, timeout=60)
        return out, result

    def test_missing_and_valid_files_keep_their_core_selection(self):
        out, result = self.launch('missing')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        self.assertFalse((out / 'params-core.json').exists())
        for core in [0, 1]:
            out, result = self.launch(f'valid-{core}', f'borg_reborn_core={core}\n'.encode())
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((out / ('params-core.json' if core else 'params.json')).is_file())
            self.assertFalse(json.loads((out / 'summary.json').read_text())['dead'])

    def test_invalid_utf8_and_unreadable_paths_do_not_select_a_default_arm(self):
        for core in [0, 1]:
            _, result = self.launch(f'invalid-{core}',
                                    f'borg_reborn_core={core}\n# '.encode() + b'\xff')
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn('cannot read borg.txt', result.stderr)
            self.assertIn('UTF-8', result.stderr)
        _, result = self.launch('directory', directory=True)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('cannot read borg.txt', result.stderr)

    def test_environment_precedence_and_parameter_loader_read_policy(self):
        content = b'borg_reborn_core=1\n# \xff'
        out, result = self.launch('env-original', content, {'BORG_CORE': 'reborn'})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        _, result = self.launch('env-vendored', content, {
            'BORG_CORE': 'borg-rs', 'BORG_STOPSCARED': 'off', 'BORG_ERRAND': 'none',
        })
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('cannot read borg.txt', result.stderr)
        plan = self.root / 'explicit-plan.txt'
        plan.write_text('[default]\n  level <= 50\n    max-depth 1\n')
        _, result = self.launch('env-plan-parameters', content, {
            'BORG_CORE': 'borg-rs', 'BORG_STOPSCARED': 'off', 'BORG_ERRAND': 'none',
            'BORG_ERRANDS_PLAN': str(plan),
        })
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('core parameters:', result.stderr)
        self.assertIn('cannot read borg.txt', result.stderr)

    def test_plan_core_isolation_preserves_lazy_environment_priority(self):
        content = b'borg_reborn_core=0\nborg_plan=missing-plan.txt\n'
        _, result = self.launch('file-original-plan', content)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('plans require BORG_CORE=borg-rs', result.stderr)
        out, result = self.launch('env-original-file-plan', content, {'BORG_CORE': 'reborn'})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        _, result = self.launch('env-original-explicit-plan', b'# \xff', {
            'BORG_CORE': 'reborn', 'BORG_ERRANDS_PLAN': 'missing-plan.txt',
        })
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('plans require BORG_CORE=borg-rs', result.stderr)

    @unittest.skipUnless(os.name == 'posix', 'OS-byte paths require POSIX')
    def test_valid_config_in_a_non_utf8_user_directory(self):
        out, result = self.launch('raw-path', b'borg_reborn_core=1\n', raw_user=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params-core.json').is_file())

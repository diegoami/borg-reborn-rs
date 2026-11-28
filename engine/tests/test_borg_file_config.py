"""Actual native startup with missing, malformed and unreadable pborg.yaml files."""
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
        config = user / 'pborg.yaml'
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
        self.assertIn('pborg.yaml is missing', result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        self.assertFalse((out / 'params-core.json').exists())
        for core in [0, 1]:
            out, result = self.launch(f'valid-{core}', f'borg_reborn_core: {core}\n'.encode())
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((out / ('params-core.json' if core else 'params.json')).is_file())
            self.assertFalse(json.loads((out / 'summary.json').read_text())['dead'])

    def test_watcher_preflight_uses_the_yaml_scariness_setting(self):
        user = self.root / 'user'
        user.mkdir()
        (user / 'pborg.yaml').write_text(
            'borg_reborn_core: "borg-rs"\nborg_scary_list: "custom.txt" # edited setting\n')
        argv = ['./angband', '-mborgh']
        argv.extend(f'-d{kind}={self.root / kind}' for kind in ('user', 'save', 'archive', 'panic', 'scores'))
        argv += ['--', '-check-pborg-settings']
        env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
        failed = subprocess.run(argv, cwd=GAME, env=env, text=True, capture_output=True, timeout=5)
        self.assertEqual(failed.returncode, 2, failed.stderr)
        self.assertIn(str(user / 'custom.txt'), failed.stderr)
        (user / 'custom.txt').write_text('')
        ok = subprocess.run(argv, cwd=GAME, env=env, text=True, capture_output=True, timeout=5)
        self.assertEqual(ok.returncode, 0, ok.stderr)

    def test_invalid_utf8_and_unreadable_paths_do_not_select_a_default_arm(self):
        for core in [0, 1]:
            _, result = self.launch(f'invalid-{core}',
                                    f'borg_reborn_core: {core}\n# '.encode() + b'\xff')
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn('pborg.yaml: cannot read configuration', result.stderr)
            self.assertIn('UTF-8', result.stderr)
        _, result = self.launch('directory', directory=True)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('pborg.yaml: cannot read configuration', result.stderr)

    def test_environment_precedence_and_parameter_loader_read_policy(self):
        content = b'borg_reborn_core: 1\n# \xff'
        out, result = self.launch('env-original', content, {'BORG_CORE': 'reborn'})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        _, result = self.launch('env-vendored', content, {
            'BORG_CORE': 'borg-rs', 'BORG_STOPSCARED': 'off', 'BORG_ERRAND': 'none',
        })
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('pborg.yaml: cannot read configuration', result.stderr)

    def test_a_retired_plan_stops_a_borg_rs_run(self):
        # #1339: the plans were retired; one that would apply stops the run.
        content = b'borg_reborn_core: 1\nborg_plan: errands_plan.txt\n'
        _, result = self.launch('file-plan', content)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('pborg.yaml borg_plan: "errands_plan.txt" would select', result.stderr)
        _, result = self.launch('env-plan', b'borg_reborn_core: 1\n', {
            'BORG_CORE': 'borg-rs', 'BORG_ERRANDS_PLAN': 'plan.txt',
        })
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('BORG_ERRANDS_PLAN: "plan.txt" would select', result.stderr)
        # Review of #1340: an inert variable does not mask an active file line.
        for value in ['', 'none']:
            _, result = self.launch(f'masked-plan-{value or "empty"}', content,
                                    {'BORG_CORE': 'borg-rs', 'BORG_ERRANDS_PLAN': value})
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn('pborg.yaml borg_plan: "errands_plan.txt" would select', result.stderr)
        # The reborn core never read the file's plan line ...
        out, result = self.launch('original-plan', b'borg_reborn_core: 0\nborg_plan: x.txt\n')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params.json').is_file())
        # ... but an explicit plan in the environment stops it too (review of #1340).
        _, result = self.launch('original-env-plan', b'borg_reborn_core: 0\n',
                                {'BORG_ERRANDS_PLAN': 'plan.txt'})
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('BORG_ERRANDS_PLAN: "plan.txt" would select', result.stderr)
        out, result = self.launch('original-env-plan-none', b'borg_reborn_core: 0\n',
                                  {'BORG_ERRANDS_PLAN': 'none'})
        self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(os.name == 'posix', 'OS-byte paths require POSIX')
    def test_valid_config_in_a_non_utf8_user_directory(self):
        out, result = self.launch('raw-path', b'borg_reborn_core: 1\n', raw_user=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'params-core.json').is_file())

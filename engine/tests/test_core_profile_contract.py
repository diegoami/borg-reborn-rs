"""Track P3: real pinned zoo profile -> borg.txt -> core parameter parsing.

Small contract cells, not a population experiment. Import the zoo's
committed tree, never its working Python files or cached modules. The run
uses the pinned adapter, converting its historical settings at the game boundary. The frontend derives
the diagnostic directory from -summary; profile settings alone select the core.

#540: the environment overrides (BORG_CORE, BORG_ERRAND, BORG_ERRAND_BUDGET,
BORG_STOPSCARED) are the core's own contract. The current adapter clears every
ambient BORG_* variable, so those tests run the game directly (`direct`): the
pinned adapter's own borg.txt (`config_text`) and command line (`command`),
with the override in the environment.
"""
import hashlib
import importlib
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game'
ZOO = Path(os.environ.get('BORG_ZOO_CONTRACT_REPO', ROOT / 'vendor/borg-zoo'))
AVAILABLE = (GAME / 'angband').is_file() and (ZOO / '.git').exists()
# The adapter's word for a core that stops before play with exit 2: a
# harness failure, or a configuration error on a specimen that pins a scary
# list or plan (borg-zoo#233).
REFUSED = ('harness_failure', 'config_error')
if os.environ.get('BORG_ZOO_CONTRACT_REQUIRED') == '1' and not AVAILABLE:
    raise RuntimeError('core profile contract requires the build and pinned vendor/borg-zoo')


@unittest.skipUnless(AVAILABLE, 'build and pinned vendor/borg-zoo required')
class CoreProfileContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        spec = importlib.util.spec_from_file_location('core_profile_zoo_check', ROOT / 'harness/zoo-check.py')
        checker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(checker)
        pin = json.loads((ROOT / 'pins.json').read_text())['pins']['borg-zoo']['commit']
        error = checker.zoo_identity_error(ZOO, pin)
        if error:
            raise AssertionError(error)
        tree = cls.root / 'pinned-zoo'
        tree.mkdir()
        checker.pinned_tree(ZOO, pin, tree)
        cls.catalogue, _ = checker.import_zoo(tree)
        cls.legacy_zoo_command = staticmethod(checker.legacy_zoo_command)
        cls.adapter = importlib.import_module('zoo.borgrs')
        if not Path(cls.adapter.__file__).resolve().is_relative_to(tree):
            raise AssertionError('adapter did not come from the pinned archive')
        cls.build = cls.root / 'build'
        cls.build.mkdir()
        (cls.build / 'game').symlink_to(GAME, target_is_directory=True)
        (cls.build / 'borg.txt').write_bytes((ROOT / 'harness/borg.txt').read_bytes())
        ident = dict(l.split('=', 1) for l in (GAME.parent / 'BUILD_ID').read_text().splitlines() if '=' in l)
        cls.ref = ident['git_rev']
        cls.binary_sha = hashlib.sha256((GAME / 'angband').read_bytes()).hexdigest()
        if cls.binary_sha != ident['binary_sha256']:
            raise AssertionError('build identity does not match the binary')
        # #535: the shipped borg.txt names a scary list; the specimen pins it
        # and the build carries it, as the zoo's build does (borg-zoo#233).
        name = 'scary_monsters.txt'
        (cls.build / 'scary_lists').mkdir()
        (cls.build / 'scary_lists' / name).write_bytes((ROOT / 'harness' / name).read_bytes())
        cls.scary_lists = {name: hashlib.sha256((ROOT / 'harness' / name).read_bytes()).hexdigest()}
        (cls.build / 'build.json').write_text(json.dumps({
            'source_commit': cls.ref, 'impl': 'rust', 'binary_sha256': cls.binary_sha,
            'scary_lists': cls.scary_lists,
        }))

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    @staticmethod
    def write_pborg_config(path, legacy):
        converted = subprocess.check_output(
            ['python3', str(ROOT / 'harness/pborg-config.py'), '-'], input=legacy, text=True)
        (path / 'pborg.yaml').write_text(converted)

    def cell(self, name, settings, overrides=None):
        profile = {'description': 'core parameter contract', 'settings': settings}
        cat = {
            'profiles': {name: profile},
            'suites': {'contract': {'cap': 300, 'wall_seconds': 600, 'birth': 'fixed'}},
            'specimens': {'specimens': {'reborn': {
                'ref': self.ref, 'impl': 'rust', 'kind': 'borg-rs',
                'exact_turns': True, 'depth_stream': True,
                'scary_lists': self.scary_lists,
            }}},
        }
        cell = {'suite': 'contract', 'specimen': 'reborn', 'profile': name, 'seed': 29}
        out = self.root / 'cells' / self.catalogue.cell_key(cell)
        # No ambient core, errand or parameter override can make this pass.
        env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
        env.update(overrides or {})
        command = self.legacy_zoo_command(self.adapter.command)
        with patch.dict(os.environ, env, clear=True), patch.object(self.adapter, 'command', command):
            record = self.adapter.run_cell(cell, self.build, self.root / 'cells', cat)
        self.assertEqual(record['profile_sha256'], self.catalogue.profile_sha256(profile))
        config = (out / 'user/borg.txt').read_text()
        self.assertEqual(record['config_sha256'], hashlib.sha256(config.encode()).hexdigest())
        self.assertIsNone(self.adapter.check_config(config, profile))
        return out, record

    def direct(self, name, settings, overrides):
        """#540: the pinned adapter's borg.txt and command line for this
        profile, run directly with `overrides` in the environment (the
        adapter clears every ambient BORG_* variable). Returns the cell
        directory and the adapter's outcome vocabulary for the exit."""
        profile = {'description': 'core environment contract', 'settings': settings}
        suite = {'cap': 300, 'wall_seconds': 600, 'birth': 'fixed'}
        spec = {'ref': self.ref, 'impl': 'rust', 'kind': 'borg-rs',
                'exact_turns': True, 'depth_stream': True}
        cell = {'suite': 'contract', 'specimen': 'reborn', 'profile': name, 'seed': 29}
        out = self.root / 'direct' / name
        for d in ('user', 'save', 'archive', 'panic', 'scores'):
            (out / d).mkdir(parents=True)
        text = self.adapter.config_text(self.build, profile, 0, 0)
        # The scary list the written borg.txt names travels with it, pinned
        # as the adapter and harness/run.sh pin it (borg-zoo#233, #535): the
        # exact bytes of the build's copy, its SHA-256 checked against the
        # specimen and written next to the selecting line.
        names = [v.strip() for k, _, v in (l.partition('=') for l in text.splitlines())
                 if k.strip() == 'borg_scary_list' and v.strip() not in ('', 'none')]
        scary = None
        if names:
            name = names[-1]
            data = (self.build / 'scary_lists' / name).read_bytes()
            sha = hashlib.sha256(data).hexdigest()
            self.assertEqual(sha, self.scary_lists[name], 'the build carries the pinned list')
            (out / 'user' / name).write_bytes(data)
            text = '\n'.join(l for l in text.splitlines() if l.partition('=')[0].strip() != 'borg_scary_list')
            text += f'\n# direct (#535): scary list sha256 {sha}\nborg_scary_list = {name}\n'
            scary = {'name': name, 'sha256': sha}
        self.write_pborg_config(out / 'user', text)
        argv = self.adapter.command(self.build / 'game' / 'angband', out, cell, 0, 0, suite, spec)
        env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
        env.update(overrides, TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
        with (out / 'stderr.log').open('w') as err:
            rc = subprocess.run(argv, cwd=self.build / 'game', env=env, stdout=subprocess.DEVNULL,
                                stderr=err, timeout=suite['wall_seconds']).returncode
        summary_path = out / 'summary.json'
        summary = json.loads(summary_path.read_text()) if summary_path.is_file() else None
        if rc != 0 and not summary:
            return out, {'outcome': 'harness_failure', 'error': f'exit code {rc}', 'summary': None,
                         'scary_list': scary}
        stop = summary.get('stop_reason') if summary else None
        outcome = {'turns': 'turn_cap', 'borg_stopped': 'borg_stopped', 'death': 'death'}.get(stop, stop)
        return out, {'outcome': outcome, 'error': None, 'summary': summary, 'scary_list': scary}

    def test_integer_profile_selects_core_and_resolves_parameters(self):
        examples = json.loads((ROOT / 'docs/examples/core-calibration-profiles.json').read_text())
        settings = dict(examples['profiles']['core-recover-rest-8']['settings'])
        settings['borg_param_power_equipment_unidentified_weapon_bonus'] = 0
        settings['borg_param_attack_preference_mage_spell_permille'] = 1600
        settings['borg_param_town_phase_reserve__warrior'] = 3
        out, record = self.cell('core-trial', settings)
        self.assertEqual(record['outcome'], 'turn_cap', record)
        settings = {s['name']: s for s in json.loads((out / 'params-core.json').read_text())}
        for name, value in [('recover_rest_sp_tenths', 8), ('power_equipment_unidentified_weapon_bonus', 0)]:
            self.assertEqual((settings[name]['value'], settings[name]['origin']), (value, 'borg.txt'))
        # #520: the legacy per-class name sets the Mage's class value.
        mage = settings['attack_preference_spell_permille']['class_values']['mage']
        self.assertEqual((mage['value'], mage['origin']), (1600, 'borg.txt'))
        receipt = json.loads((out / 'settings-resolved.json').read_text())
        self.assertEqual(receipt['schema'], 'borg-settings-resolved@1')
        self.assertEqual((receipt['class_index'], receipt['class_name']), (0, 'warrior'))
        self.assertEqual(receipt['effective_settings']['town_phase_reserve'], 3)
        self.assertNotIn('attack_preference_spell_permille', receipt['effective_settings'])
        self.assertTrue((out / 'trace-core.jsonl').stat().st_size)
        self.assertEqual(record['binary_sha256'], self.binary_sha)
        self.assertEqual(json.loads((out / 'summary.json').read_text())['game_turns'], 300)
        # #535: the shipped scary list reached the game, pinned (borg-zoo#233).
        self.assertEqual((out / 'user/scary_monsters.txt').read_bytes(),
                         (ROOT / 'harness/scary_monsters.txt').read_bytes())
        self.assertEqual(record['scary_list']['sha256'], self.scary_lists['scary_monsters.txt'], record)

    def test_invalid_profile_value_fails_instead_of_using_default(self):
        out, record = self.cell('invalid', {
            'borg_reborn_core': 1, 'borg_param_recover_rest_sp_tenths': 11,
        })
        self.assertIn(record['outcome'], REFUSED, record)
        self.assertIn('exit code 2', record['error'])
        self.assertIn('recover_rest_sp_tenths: 11 outside 0..=10', (out / 'stderr.log').read_text())
        self.assertFalse((out / 'params-core.json').exists())

    def test_zero_keeps_original_core_profiles_working(self):
        out, record = self.cell('original', {'borg_reborn_core': 0})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params.json').is_file())
        self.assertFalse((out / 'params-core.json').exists())

    def test_invalid_numeric_core_selector_is_not_an_original_core_run(self):
        for value in [2, 7, True]:
            with self.subTest(value=value):
                out, record = self.cell(f'invalid-core-{value}', {'borg_reborn_core': value})
                self.assertIn(record['outcome'], REFUSED, record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('pborg.yaml borg_reborn_core: invalid core selector',
                              (out / 'stderr.log').read_text())
                self.assertFalse((out / 'params-core.json').exists())
                self.assertFalse((out / 'params.json').exists())

    def test_environment_selector_precedence_in_actual_process(self):
        for value in ['', 'true', '2', 'borg_rs']:
            with self.subTest(value=value):
                out, record = self.direct('invalid-env-' + (value or 'empty'),
                                       {'borg_reborn_core': 1}, {'BORG_CORE': value})
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('BORG_CORE: invalid core selector',
                              (out / 'stderr.log').read_text())
        out, record = self.direct('valid-env-wins', {'borg_reborn_core': 7},
                                {'BORG_CORE': 'borg-rs'})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params-core.json').exists())
        # The direct run plays the shipped list, pinned by its hash (#535).
        name = 'scary_monsters.txt'
        self.assertEqual(record['scary_list'], {'name': name, 'sha256': self.scary_lists[name]}, record)
        self.assertIn(f"scary list sha256 {self.scary_lists[name]}\nborg_scary_list: \"{name}\"\n",
                      (out / 'user/pborg.yaml').read_text())
        self.assertEqual((out / 'user' / name).read_bytes(), (ROOT / 'harness' / name).read_bytes())

    def test_invalid_errand_profile_is_not_an_unconstrained_core_run(self):
        for value in [2, True]:
            with self.subTest(value=value):
                out, record = self.cell(f'invalid-errand-{value}', {
                    'borg_reborn_core': 1, 'borg_reborn_errand': value,
                })
                self.assertIn(record['outcome'], REFUSED, record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('pborg.yaml borg_reborn_errand: invalid errand selector',
                              (out / 'stderr.log').read_text())
                self.assertFalse((out / 'trace-core.jsonl').exists())

    def test_errand_environment_precedence_and_invalid_command_lines(self):
        for value in ['; ;', 'true', 'dvie 2', '/town; typo']:
            with self.subTest(value=value):
                out, record = self.direct('invalid-errand-env-' + str(len(value)),
                                       {'borg_reborn_core': 1}, {'BORG_ERRAND': value})
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('BORG_ERRAND: invalid errand selector',
                              (out / 'stderr.log').read_text())
        for name, value in [('none', 'none'), ('empty', ''), ('command', '/dive 2')]:
            out, record = self.direct('valid-errand-env-' + name,
                                   {'borg_reborn_core': 1, 'borg_reborn_errand': 7},
                                   {'BORG_ERRAND': value})
            self.assertEqual(record['outcome'], 'turn_cap', record)
            self.assertTrue((out / 'params-core.json').exists())

    def test_original_core_ignores_vendored_errand_configuration(self):
        out, record = self.direct('original-ignores-errand',
                               {'borg_reborn_core': 0, 'borg_reborn_errand': 7},
                               {'BORG_ERRAND': 'dvie 2'})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params.json').exists())
        self.assertFalse((out / 'params-core.json').exists())

    def test_invalid_stopscared_settings_fail_with_source_and_value(self):
        for value in [0, -1, 1001]:
            out, record = self.cell(f'invalid-fear-file-{value}', {
                'borg_reborn_core': 1, 'borg_reborn_stopscared': value,
            })
            self.assertIn(record['outcome'], REFUSED, record)
            self.assertIn('exit code 2', record['error'])
            self.assertIn('pborg.yaml borg_reborn_stopscared: invalid stopscared value',
                          (out / 'stderr.log').read_text())
        for value in ['bogus', '0', '1001', 'off; /dive 2']:
            out, record = self.direct('invalid-fear-env-' + str(len(value)),
                                   {'borg_reborn_core': 1}, {'BORG_STOPSCARED': value})
            self.assertEqual(record['outcome'], 'harness_failure', record)
            self.assertIn('exit code 2', record['error'])
            self.assertIn('BORG_STOPSCARED: invalid stopscared value',
                          (out / 'stderr.log').read_text())

    def test_valid_fear_overrides_and_original_core_isolation(self):
        for name, value in [('off', 'off'), ('none', 'none'), ('empty', '')]:
            out, record = self.direct('valid-fear-' + name,
                                   {'borg_reborn_core': 1, 'borg_reborn_stopscared': 1001},
                                   {'BORG_STOPSCARED': value})
            self.assertEqual(record['outcome'], 'turn_cap', record)
            self.assertTrue((out / 'params-core.json').exists())
        _, record = self.cell('valid-fear-file-false', {
            'borg_reborn_core': 1, 'borg_reborn_stopscared': False,
        })
        self.assertEqual(record['outcome'], 'turn_cap', record)
        out, record = self.direct('original-ignores-fear',
                               {'borg_reborn_core': 0, 'borg_reborn_stopscared': 1001},
                               {'BORG_STOPSCARED': 'bogus'})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params.json').exists())

    def test_non_utf8_fear_environment_is_not_silently_ignored(self):
        out, record = self.direct('invalid-fear-bytes', {'borg_reborn_core': 1},
                               {'BORG_STOPSCARED': os.fsdecode(b'\xff')})
        self.assertEqual(record['outcome'], 'harness_failure', record)
        self.assertIn('exit code 2', record['error'])
        self.assertIn('BORG_STOPSCARED: invalid non-UTF-8 stopscared value',
                      (out / 'stderr.log').read_text())

    def test_invalid_errand_budget_cannot_become_unbounded(self):
        values = ['bogus', '', '-1', '9223372036854775808', '1.5']
        for i, value in enumerate(values):
            out, record = self.direct(f'invalid-budget-{i}', {'borg_reborn_core': 1},
                                   {'BORG_ERRAND': '/dive 2', 'BORG_ERRAND_BUDGET': value})
            self.assertEqual(record['outcome'], 'harness_failure', record)
            self.assertIn('exit code 2', record['error'])
            self.assertIn('BORG_ERRAND_BUDGET: invalid budget',
                          (out / 'stderr.log').read_text())
        out, record = self.direct('invalid-budget-bytes', {'borg_reborn_core': 1},
                               {'BORG_ERRAND': '/dive 2',
                                'BORG_ERRAND_BUDGET': os.fsdecode(b'\xff')})
        self.assertEqual(record['outcome'], 'harness_failure', record)
        self.assertIn('BORG_ERRAND_BUDGET: invalid non-UTF-8 budget',
                      (out / 'stderr.log').read_text())

    def test_valid_budget_and_inactive_core_isolation(self):
        for value in ['0', 'none', '9223372036854775807']:
            _, record = self.direct('valid-budget-' + value, {'borg_reborn_core': 1},
                                  {'BORG_ERRAND': '/dive 2', 'BORG_ERRAND_BUDGET': value})
            self.assertEqual(record['outcome'], 'turn_cap', record)
        out, record = self.direct('finite-budget', {'borg_reborn_core': 1},
                               {'BORG_ERRAND': '/dive 2', 'BORG_ERRAND_BUDGET': '1'})
        # The core's own stop, read from its summary (#540: run directly).
        self.assertEqual(record['outcome'], 'borg_stopped', record)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertEqual(summary['stop_reason'], 'borg_stopped')
        self.assertLess(summary['game_turns'], 300)
        for name, settings, errand in [
            ('original', {'borg_reborn_core': 0}, '/dive 2'),
            ('inactive', {'borg_reborn_core': 1}, 'none'),
        ]:
            _, record = self.direct('ignored-budget-' + name, settings,
                                  {'BORG_ERRAND': errand, 'BORG_ERRAND_BUDGET': 'bogus'})
            self.assertEqual(record['outcome'], 'turn_cap', record)

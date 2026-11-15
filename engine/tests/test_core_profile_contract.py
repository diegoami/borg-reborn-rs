"""Track P3: real pinned zoo profile -> borg.txt -> core parameter parsing.

Small contract cells, not a population experiment. Import the zoo's
committed tree, never its working Python files or cached modules. The run
uses the adapter unchanged, with no BORG_* environment. The frontend derives
the diagnostic directory from -summary; profile settings alone select the core.
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
        (cls.build / 'build.json').write_text(json.dumps({
            'source_commit': cls.ref, 'impl': 'rust', 'binary_sha256': cls.binary_sha,
        }))

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def cell(self, name, settings, overrides=None):
        profile = {'description': 'core parameter contract', 'settings': settings}
        cat = {
            'profiles': {name: profile},
            'suites': {'contract': {'cap': 300, 'wall_seconds': 600, 'birth': 'fixed'}},
            'specimens': {'specimens': {'reborn': {
                'ref': self.ref, 'impl': 'rust', 'kind': 'borg-rs',
                'exact_turns': True, 'depth_stream': True,
            }}},
        }
        cell = {'suite': 'contract', 'specimen': 'reborn', 'profile': name, 'seed': 29}
        out = self.root / 'cells' / self.catalogue.cell_key(cell)
        # No ambient core, errand or parameter override can make this pass.
        env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
        env.update(overrides or {})
        with patch.dict(os.environ, env, clear=True):
            record = self.adapter.run_cell(cell, self.build, self.root / 'cells', cat)
        self.assertEqual(record['profile_sha256'], self.catalogue.profile_sha256(profile))
        config = (out / 'user/borg.txt').read_text()
        self.assertEqual(record['config_sha256'], hashlib.sha256(config.encode()).hexdigest())
        self.assertIsNone(self.adapter.check_config(config, profile))
        return out, record

    def test_integer_profile_selects_core_and_resolves_parameters(self):
        examples = json.loads((ROOT / 'docs/examples/core-calibration-profiles.json').read_text())
        settings = dict(examples['profiles']['core-recover-rest-8']['settings'])
        settings['borg_param_power_equipment_unidentified_weapon_bonus'] = 0
        out, record = self.cell('core-trial', settings)
        self.assertEqual(record['outcome'], 'turn_cap', record)
        settings = {s['name']: s for s in json.loads((out / 'params-core.json').read_text())}
        for name, value in [('recover_rest_sp_tenths', 8), ('power_equipment_unidentified_weapon_bonus', 0)]:
            self.assertEqual((settings[name]['value'], settings[name]['origin']), (value, 'borg.txt'))
        self.assertTrue((out / 'trace-core.jsonl').stat().st_size)
        self.assertEqual(record['binary_sha256'], self.binary_sha)
        self.assertEqual(json.loads((out / 'summary.json').read_text())['game_turns'], 300)

    def test_invalid_profile_value_fails_instead_of_using_default(self):
        out, record = self.cell('invalid', {
            'borg_reborn_core': 1, 'borg_param_recover_rest_sp_tenths': 11,
        })
        self.assertEqual(record['outcome'], 'harness_failure', record)
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
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('borg.txt borg_reborn_core: invalid core selector',
                              (out / 'stderr.log').read_text())
                self.assertFalse((out / 'params-core.json').exists())
                self.assertFalse((out / 'params.json').exists())

    def test_environment_selector_precedence_in_actual_process(self):
        for value in ['', 'true', '2', 'borg_rs']:
            with self.subTest(value=value):
                out, record = self.cell('invalid-env-' + (value or 'empty'),
                                       {'borg_reborn_core': 1}, {'BORG_CORE': value})
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('BORG_CORE: invalid core selector',
                              (out / 'stderr.log').read_text())
        out, record = self.cell('valid-env-wins', {'borg_reborn_core': 7},
                                {'BORG_CORE': 'borg-rs'})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params-core.json').exists())

    def test_invalid_errand_profile_is_not_an_unconstrained_core_run(self):
        for value in [2, True]:
            with self.subTest(value=value):
                out, record = self.cell(f'invalid-errand-{value}', {
                    'borg_reborn_core': 1, 'borg_reborn_errand': value,
                })
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('borg.txt borg_reborn_errand: invalid errand selector',
                              (out / 'stderr.log').read_text())
                self.assertFalse((out / 'trace-core.jsonl').exists())

    def test_errand_environment_precedence_and_invalid_command_lines(self):
        for value in ['', '; ;', 'true', 'dvie 2', '/town; typo']:
            with self.subTest(value=value):
                out, record = self.cell('invalid-errand-env-' + str(len(value)),
                                       {'borg_reborn_core': 1}, {'BORG_ERRAND': value})
                self.assertEqual(record['outcome'], 'harness_failure', record)
                self.assertIn('exit code 2', record['error'])
                self.assertIn('BORG_ERRAND: invalid errand selector',
                              (out / 'stderr.log').read_text())
        for name, value in [('none', 'none'), ('command', '/dive 2')]:
            out, record = self.cell('valid-errand-env-' + name,
                                   {'borg_reborn_core': 1, 'borg_reborn_errand': 7},
                                   {'BORG_ERRAND': value})
            self.assertEqual(record['outcome'], 'turn_cap', record)
            self.assertTrue((out / 'params-core.json').exists())

    def test_original_core_ignores_vendored_errand_configuration(self):
        out, record = self.cell('original-ignores-errand',
                               {'borg_reborn_core': 0, 'borg_reborn_errand': 7},
                               {'BORG_ERRAND': 'dvie 2'})
        self.assertEqual(record['outcome'], 'turn_cap', record)
        self.assertTrue((out / 'params.json').exists())
        self.assertFalse((out / 'params-core.json').exists())

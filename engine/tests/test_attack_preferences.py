"""Native class/family preferences: real selection, eligibility and trace parity.

This is a bounded regression check, not a population/survival comparison. The
single seeded Mage process tests the production selector and original helpers.
Synthetic unit vectors cover the other classes; do not call those native cases.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(BINARY.is_file(), 'native build required')
class AttackPreferencesTest(unittest.TestCase):
    def test_real_weighted_choice_and_trace_on_off_preserve_streams(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            env = {k: v for k, v in os.environ.items() if not k.startswith('BORG_')}
            # Keep tooling and evidence destinations, but no ambient policy.
            for key in ['BORG_CEMETERY_DIR']:
                if key in os.environ:
                    env[key] = os.environ[key]
            # This is the native core, whose calibration/trace contract is
            # separate from the original core's observation-replay registry.
            # Retain deaths outside the temporary run directories and let CI
            # reconcile them against the independently pinned native archive.
            storage = Path(os.environ.get('BORG_DEATH_RECORDS_DIR',
                                          str(Path(tempfile.gettempdir()) / 'borg-tests/deaths')))
            registry = Path(os.environ.get('BORG_ATTACK_PREFERENCE_DEATHS_DIR',
                                           str(storage.parent / 'attack-preference-deaths')))
            env['BORG_DEATH_RECORDS_DIR'] = str(registry.resolve())
            env.update(BORG_CORE='borg-rs', BORG_CORE_OVERRIDES='off',
                       BORG_PARAM_ATTACK_PREFERENCE_MAGE_SPELL_PERMILLE='500',
                       BORG_PARAM_ATTACK_PREFERENCE_MAGE_MELEE_PERMILLE='2000')
            for name, tracing in [('on', 'on'), ('off', 'off')]:
                current = dict(env, BORG_CORE_TRACE=tracing)
                result = subprocess.run(
                    [str(ROOT / 'harness/run.sh'), str(root / name),
                     # #470: seed 1014; 1011's first 3000 turns no longer
                     # meet a selection the profile changes.
                     '1014', '2', '1', '3000', 'menu'],
                    cwd=ROOT, env=current, capture_output=True, text=True, timeout=180)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            for name in ['keys.log', 'messages.log', 'state.log']:
                self.assertEqual((root / 'on' / name).read_bytes(),
                                 (root / 'off' / name).read_bytes(), name)
            self.assertFalse((root / 'off/trace-core.jsonl').exists())
            settings = {p['name']: p for p in json.loads((root / 'on/params-core.json').read_text())}
            # #520: one parameter per family; the Mage value is a class value
            # (the legacy per-class environment name is its alias).
            preference = settings['attack_preference_spell_permille']['class_values']['mage']
            self.assertEqual((preference['value'], preference['origin']), (500, 'environment'))
            seen = changed = rejected = 0
            with (root / 'on/trace-core.jsonl').open() as trace:
                for line in trace:
                    row = json.loads(line)
                    self.assertEqual(row['schema'], 'borg-reborn-rs/core-trace@3')
                    for ranking in row['attacks']:
                        candidates = ranking['candidates']
                        self.assertEqual([c['attack_id'] for c in candidates],
                                         list(range(len(candidates))))
                        raw_best = weighted_best = None
                        for candidate in candidates:
                            seen += 1
                            raw = candidate['raw_utility']
                            weight = candidate['effective_weight']
                            self.assertEqual(candidate['class_name'], 'mage')
                            self.assertEqual(candidate['adjusted_utility'],
                                             max(1, min(2147483647, raw * weight // 1000)) if raw > 0 else raw)
                            if raw <= 0:
                                rejected += 1
                                self.assertEqual(candidate['eligibility'], 'nonpositive_return')
                            elif raw_best is None or raw > raw_best['raw_utility']:
                                raw_best = candidate
                            if candidate['adjusted_utility'] > (weighted_best['adjusted_utility'] if weighted_best else 0):
                                weighted_best = candidate
                        winner = weighted_best['attack_id'] if weighted_best else None
                        self.assertEqual(ranking['winner'], winner)
                        if raw_best and weighted_best and raw_best['attack_id'] != winner:
                            changed += 1
                            self.assertTrue(row['acted'])
                            self.assertTrue(row['keys'])
            self.assertGreater(seen, 0)
            self.assertGreater(rejected, 0)
            self.assertGreater(changed, 0, 'non-neutral profile never changed a real selection')

"""Versioned shadow gates reject missing, vacuous and inconsistent evidence."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
spec = importlib.util.spec_from_file_location('check_shadow', Path(__file__).resolve().parents[2] / 'harness/check-shadow.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def valid_row():
    return {'schema': 'borg-reborn-rs/shadow-stats@3', 'surface': 'r3-attack-defence',
            'mode': 'check', 'ported_points': module.R2_PORTED + ['attack', 'defence'], 'unported_points': [],
            'counts': {'thinks': 1, 'mismatches': 0, 'comparisons': 6,
                       'danger_calls': 1, 'danger_comparisons': 1, 'monster_comparisons': 1,
                       'wall_comparisons': 1, 'stateful_comparisons': 1,
                       'caution_calls': 1, 'escape_calls': 9, 'recovery_calls': 1,
                       'attack_calls': 9, 'pure_attack_comparisons': 2, 'defence_calls': 1,
                       'tape_events': 21, 'policy_markers': 19, 'dependency_requests': 2,
                       'stateful_points': dict.fromkeys(module.ENTRY_POINTS, 1),
                       'pure_points': dict.fromkeys(module.PURE_POINTS, 1),
                       'root_points': {'caution.borg_caution': 1}}}


class ShadowCheckTest(unittest.TestCase):
    def write(self, root, row):
        (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
        (root / 'shadow.jsonl').write_text('')

    def test_missing_zero_and_divergent_evidence_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            with self.assertRaisesRegex(ValueError, 'no shadow'):
                module.check([root])
            row = valid_row()
            row['counts'] = {k: (1 if k == 'thinks' else {} if isinstance(v, dict) else 0)
                             for k, v in row['counts'].items()}
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'insufficient'):
                module.check([root])
            self.write(root, valid_row())
            self.assertEqual(module.check([root])[1]['comparisons'], 6)
            (root / 'shadow.jsonl').write_text('{"original":false,"ported":true}\n')
            with self.assertRaisesRegex(ValueError, 'nonzero'):
                module.check([root])

    def test_wall_comparisons_cannot_substitute_for_danger(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            row = valid_row(); row['counts']['danger_comparisons'] = 0
            row['counts']['wall_comparisons'] += 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'without comparisons'):
                module.check([root])

    def test_aggregate_totals_cannot_hide_a_missing_entry(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); self.write(root, row)
            self.assertTrue(module.check([root], True)[1]['entry_points'])
            row['counts']['stateful_points']['escape.borg_caution_teleport'] = 0
            row['counts']['escape_calls'] -= 1
            row['counts']['policy_markers'] -= 1
            row['counts']['dependency_requests'] += 1
            self.write(root, row)
            module.check([root])
            with self.assertRaisesRegex(ValueError, 'caution_teleport'):
                module.check([root], True)
            row['counts']['escape_calls'] += 1; self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'inconsistent individual'):
                module.check([root], True)

    def test_event_and_comparison_identities_are_independently_checked(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for field in ['tape_events', 'policy_markers', 'dependency_requests',
                          'pure_attack_comparisons', 'attack_calls', 'comparisons',
                          'stateful_comparisons', 'defence_calls']:
                with self.subTest(field=field):
                    row = valid_row(); row['counts'][field] += 1; self.write(root, row)
                    with self.assertRaisesRegex(ValueError, 'inconsistent'):
                        module.check([root], True)

    def test_internal_markers_do_not_substitute_for_standalone_pure_comparisons(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row()
            row['counts']['pure_points']['attack.borg_thrust_damage_one'] = 0
            row['counts']['pure_attack_comparisons'] -= 1
            row['counts']['attack_calls'] -= 1; row['counts']['comparisons'] -= 1
            self.write(root, row)
            module.check([root])
            with self.assertRaisesRegex(ValueError, 'standalone pure'):
                module.check([root], True)

    def test_legacy_is_explicit_and_raw_event_meanings_are_preserved(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); row['schema'] = 'borg-reborn-rs/shadow-stats@1'
            row.pop('surface'); row['ported_points'] = module.R2_PORTED
            c = row['counts']; c['stateful_points'] = dict.fromkeys(module.R2_ENTRY_POINTS, 1)
            c['comparisons'] = 4; c['dependency_requests'] = 20
            for field in ['tape_events', 'policy_markers', 'pure_points', 'pure_attack_comparisons', 'attack_calls']:
                c.pop(field)
            self.write(root, row); before = (root / 'shadow-stats.jsonl').read_bytes()
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root])
            totals = module.check([root], True, legacy_r2=True)[1]
            self.assertEqual((totals['tape_events'], totals['policy_markers'], totals['dependency_requests']),
                             (20, 11, 9))
            self.assertEqual(before, (root / 'shadow-stats.jsonl').read_bytes())

    def test_wrong_surface_unknown_entries_and_boolean_counters_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for variant in ['surface', 'entry', 'boolean']:
                row = valid_row()
                if variant == 'surface': row['surface'] = 'old-surface'
                elif variant == 'entry': row['counts']['stateful_points']['fake.entry'] = 1
                else: row['counts']['comparisons'] = True
                self.write(root, row)
                with self.assertRaises(ValueError): module.check([root])

    def test_attack_only_evidence_requires_explicit_legacy_reader(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row()
            row['schema'] = 'borg-reborn-rs/shadow-stats@2'
            row['surface'] = 'r3-attack'; row['ported_points'].remove('defence')
            counts = row['counts']; counts.pop('defence_calls')
            counts['stateful_points'].pop('defence.borg_defend')
            counts['tape_events'] -= 1; counts['policy_markers'] -= 1
            self.write(root, row)
            before = (root / 'shadow-stats.jsonl').read_bytes()
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], True)
            self.assertEqual(module.check([root], True, legacy_attack=True)[1]['policy_markers'], 18)
            self.assertEqual(before, (root / 'shadow-stats.jsonl').read_bytes())

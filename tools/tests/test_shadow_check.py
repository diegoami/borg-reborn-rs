"""The CI shadow gate rejects missing, vacuous and divergent comparisons."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('check_shadow', Path(__file__).resolve().parents[2] / 'harness/check-shadow.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ShadowCheckTest(unittest.TestCase):
    def test_missing_evidence_and_zero_comparisons_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            with self.assertRaisesRegex(ValueError, 'no shadow'):
                module.check([root])
            row = {'schema': 'borg-reborn-rs/shadow-stats@1', 'mode': 'check',
                   'ported_points': ['danger.wall_like', 'danger', 'danger.one_monster', 'caution', 'escape', 'recovery'],
                   'unported_points': [],
                   'counts': {'thinks': 1, 'mismatches': 0, 'comparisons': 0,
                              'danger_calls': 0, 'caution_calls': 0, 'escape_calls':0, 'recovery_calls':0, 'stateful_comparisons':0, 'danger_comparisons': 0,
                              'monster_comparisons': 0, 'wall_comparisons': 0}}
            (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
            (root / 'shadow.jsonl').write_text('')
            with self.assertRaisesRegex(ValueError, 'insufficient'):
                module.check([root])
            row['counts'].update(comparisons=6, danger_calls=1, danger_comparisons=1, monster_comparisons=1, wall_comparisons=1, stateful_comparisons=3, caution_calls=1, escape_calls=1, recovery_calls=1)
            (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
            self.assertEqual(module.check([root])[1]['comparisons'], 6)
            (root / 'shadow.jsonl').write_text('{"original":false,"ported":true}\n')
            with self.assertRaisesRegex(ValueError, 'nonzero'):
                module.check([root])

    def test_wall_comparisons_cannot_substitute_for_danger(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            row = {'schema':'borg-reborn-rs/shadow-stats@1','mode':'check',
                   'ported_points':['danger.wall_like','danger','danger.one_monster','caution','escape','recovery'],
                   'unported_points':[],
                   'counts':{'thinks':1,'mismatches':0,'comparisons':3,'danger_calls':1,
                             'caution_calls':1,'danger_comparisons':0,'monster_comparisons':1,'wall_comparisons':2}}
            (root/'shadow-stats.jsonl').write_text(json.dumps(row)+'\n')
            (root/'shadow.jsonl').write_text('')
            with self.assertRaisesRegex(ValueError,'without comparisons'):
                module.check([root])

    def test_aggregate_totals_cannot_hide_missing_individual_entries(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            entries = dict.fromkeys(module.ENTRY_POINTS, 1)
            row = {'schema': 'borg-reborn-rs/shadow-stats@1', 'mode': 'check',
                   'ported_points': ['danger.wall_like', 'danger', 'danger.one_monster', 'caution', 'escape', 'recovery'],
                   'unported_points': [],
                   'counts': {'thinks': 1, 'mismatches': 0, 'comparisons': 6,
                              'danger_calls': 1, 'danger_comparisons': 1,
                              'monster_comparisons': 1, 'wall_comparisons': 1,
                              'stateful_comparisons': 3, 'caution_calls': 1,
                              'escape_calls': 9, 'recovery_calls': 1,
                              'stateful_points': entries}}
            path = root / 'shadow-stats.jsonl'
            (root / 'shadow.jsonl').write_text('')
            path.write_text(json.dumps(row) + '\n')
            self.assertEqual(module.check([root], True)[1]['stateful_points'], entries)
            entries['escape.borg_caution_teleport'] = 0
            row['counts']['escape_calls'] -= 1
            path.write_text(json.dumps(row) + '\n')
            module.check([root])  # Aggregate coverage still passes.
            with self.assertRaisesRegex(ValueError, 'caution_teleport'):
                module.check([root], True)
            row['counts']['escape_calls'] += 1
            path.write_text(json.dumps(row) + '\n')
            with self.assertRaisesRegex(ValueError, 'inconsistent individual'):
                module.check([root], True)

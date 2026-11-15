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
                   'ported_points': ['danger.wall_like'],
                   'counts': {'thinks': 1, 'mismatches': 0, 'comparisons': 0,
                              'danger_calls': 2, 'caution_calls': 1}}
            (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
            (root / 'shadow.jsonl').write_text('')
            with self.assertRaisesRegex(ValueError, 'insufficient'):
                module.check([root])
            row['counts']['comparisons'] = 3
            (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
            self.assertEqual(module.check([root])[1]['comparisons'], 3)
            (root / 'shadow.jsonl').write_text('{"original":false,"ported":true}\n')
            with self.assertRaisesRegex(ValueError, 'nonzero'):
                module.check([root])

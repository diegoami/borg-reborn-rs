"""Power correspondence rejects a literal drift or a return/parameter change."""
import importlib.util
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
spec = importlib.util.spec_from_file_location('power_audit', Path(__file__).resolve().parents[1] / 'audit-power-port.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class PowerPortTest(unittest.TestCase):
    def test_all_production_bodies_correspond(self):
        self.assertEqual(len(audit.audit()), 11)

    def test_literal_parameter_and_return_drift_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ['borg-core/src/borg_power.rs', 'borg-next/src/stateful/power.rs', 'borg-next/src/lib.rs']:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(audit.ROOT / name, target)
            path = root / 'borg-next/src/stateful/power.rs'
            original = path.read_text()
            for before, after in [('.wrapping_mul(2)', '.wrapping_mul(3)'),
                                  ('Param::power_inventory_afuel_weight', 'Param::power_inventory_food_weight'),
                                  ('(i64::from(value) + x) as i32', '(i64::from(value) - x) as i32')]:
                with self.subTest(edit=before):
                    self.assertTrue(before in original, 'mutation anchor missing: '+before)
                    path.write_text(original.replace(before, after, 1))
                    with self.assertRaisesRegex(ValueError, 'correspondence differs'):
                        audit.audit(root)

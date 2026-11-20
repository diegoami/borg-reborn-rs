"""The carry must not admit an unrelated policy, registry change or ID alias."""
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('p4_carry', ROOT / 'scripts/check-p4-carry.py')
carry = importlib.util.module_from_spec(spec)
spec.loader.exec_module(carry)


class P4CarryTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for name in (*carry.MODULES, 'borg-core/src/params.rs',
                     'docs/R528-CARRY.json', 'docs/R437-NATIVE-SEAMS.patch', 'docs/R436-NATIVE-SEAMS.patch', 'docs/R434-NATIVE-SEAMS.patch', 'docs/R528-P4-SITES.patch',
                     'docs/M13-INVENTORY-PARAMETERS.json'):
            destination = self.root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, destination)

    def test_exact_carry_is_verified_offline(self):
        self.assertEqual(carry.verify(self.root), 362)

    def test_existing_default_cannot_change_with_the_carry(self):
        path = self.root / 'borg-core/src/params.rs'
        path.write_text(path.read_text().replace('caution_bounce_healthy_tenths => (7,',
                                                'caution_bounce_healthy_tenths => (8,', 1))
        with self.assertRaisesRegex(ValueError, 'existing receiving spec'):
            carry.verify(self.root)

    def test_extra_native_policy_change_is_rejected(self):
        path = self.root / carry.MODULES[0]
        path.write_text(path.read_text() + '\n// unrelated receiving edit\n')
        with self.assertRaisesRegex(ValueError, 'beyond accepted P4 delta'):
            carry.verify(self.root)

    def test_validation_and_bounds_regressions_are_also_protected(self):
        path = self.root / 'borg-core/src/params.rs'
        original = path.read_text()
        for before, after in [
            ('|(a, b)| a > b', '|(a, b)| a >= b'),
            ('for valid in [0, 2, 255]', 'for valid in [0, 2, 254]'),
        ]:
            with self.subTest(edit=before):
                self.assertEqual(original.count(before), 1)
                path.write_text(original.replace(before, after, 1))
                with self.assertRaisesRegex(ValueError, 'non-spec parameter'):
                    carry.verify(self.root)
        path.write_text(original)

    def test_contract_mutation_and_wrong_id_mapping_are_rejected(self):
        path = self.root / 'docs/M13-INVENTORY-PARAMETERS.json'
        original = path.read_bytes()
        path.write_bytes(original + b'\n')
        with self.assertRaisesRegex(ValueError, 'contract bytes'):
            carry.verify(self.root)
        path.write_bytes(original)
        path = self.root / 'docs/R528-CARRY.json'
        metadata = json.loads(path.read_text())
        metadata['parameter_id_mapping'][0]['accepted_id'] += 1
        path.write_text(json.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'ID mapping'):
            carry.verify(self.root)


if __name__ == '__main__':
    unittest.main()

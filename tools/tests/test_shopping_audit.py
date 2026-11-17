"""Tests for tools/shopping-audit.py. Run: python3 -m unittest discover -s tools/tests"""

import importlib.util
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('shopping_audit', HERE.parent / 'shopping-audit.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class CapabilityNamesTest(unittest.TestCase):
    def test_names_come_from_the_capability_module(self):
        # #496 review F3: the audit's capability names are borg_town_capability::NAMES.
        self.assertEqual(audit.CAPS[2], 'healing HP')
        self.assertEqual(audit.CAPS[58], 'healing reserve')
        self.assertEqual(len(audit.CAPS), len(set(audit.CAPS)))

    def test_a_count_mismatch_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'cap.rs'
            path.write_text('pub const NAMES: [&str; 3] = ["a", "b"];\n')
            with self.assertRaises(SystemExit):
                audit.capability_names(str(path))
            path.write_text('pub const NAMES: [&str; 2] = ["a",\n    "b"];\n')
            self.assertEqual(audit.capability_names(str(path)), ['a', 'b'])


if __name__ == '__main__':
    unittest.main()

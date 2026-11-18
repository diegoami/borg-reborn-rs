"""#520: the class files are folded into harness/borg.txt."""
import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/borg-class-files.py'
spec = importlib.util.spec_from_file_location('borg_class_files', SCRIPT)
bcf = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bcf)


class ClassFilesTest(unittest.TestCase):
    def test_borg_txt_carries_the_class_files(self):
        result = subprocess.run([sys.executable, str(SCRIPT), 'check'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_lines_use_the_legacy_alias_or_the_class_suffix(self):
        self.assertEqual(bcf.class_lines('mage', 'borg_param_attack_preference_spell_permille = 2000\n'),
                         ['borg_param_attack_preference_mage_spell_permille = 2000'])
        self.assertEqual(bcf.class_lines('ranger', 'borg_param_attack_arrow_point_blank_divisor = 2'),
                         ['borg_param_attack_arrow_point_blank_divisor_ranger = 2'])
        self.assertEqual(bcf.class_lines('mage', '# c\n\nborg_param_caution_cornered = 0 # off\n'),
                         ['borg_param_caution_cornered@mage = 0'])

    def test_bad_lines_are_refused(self):
        for bad in ['borg_respawn_class = 1', 'borg_param_x = y', 'borg_param_x@mage = 1']:
            with self.assertRaises(SystemExit):
                bcf.class_lines('mage', bad)

    def test_a_damaged_block_is_refused(self):
        for text in [bcf.BEGIN + '\nx\n', 'x\n' + bcf.END + '\n', bcf.END + '\n' + bcf.BEGIN + '\n',
                     bcf.BEGIN + '\n' + bcf.END + '\n' + bcf.BEGIN + '\n' + bcf.END + '\n']:
            with self.assertRaises(SystemExit):
                bcf.folded(text)

    def test_every_class_has_a_file(self):
        for cls in bcf.CLASSES:
            self.assertTrue((ROOT / f'harness/borg_{cls}.txt').exists(), cls)


if __name__ == '__main__':
    unittest.main()

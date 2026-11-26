"""The R4.5 evidence gate requires a successful receipt for every named cell."""
import importlib.util
from pathlib import Path
import tempfile
import unittest


path = Path(__file__).resolve().parents[2] / 'harness/check-r45-suite.py'
spec = importlib.util.spec_from_file_location('r45_suite_check', path)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)

CELL = 'full-0-warrior-0'
PIN = '1' * 40


def write_fields(path, **fields):
    path.write_text(''.join(f'{key}={value}\n' for key, value in fields.items()))


class R45SuiteCheckTest(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.cell = self.root / CELL
        self.cell.mkdir()
        digest = '2' * 64
        self.manifest = {
            'suite': 'full', 'suite_sha256': gate.SUITES['full'], 'borg_txt_sha256': gate.SHIPPED,
            'build_id_git_rev': PIN, 'build_id_git_dirty': 'no', 'build_id_inputs_dirty': 'no',
            'binary_sha256': digest, 'build_id_binary_sha256': digest, 'binary_sha256_end': digest,
            'build_id_source_sha256': digest, 'checkout_source_sha256': digest,
            f'run_config.{CELL}': f'menu {digest}',
        }
        write_fields(self.root / 'manifest', **self.manifest)
        self.config = {'borg_core': 'borg-rs', 'borg_core_overrides': 'on',
                       'borg_shadow': 'check', 'borg_txt_sha256': digest}
        write_fields(self.cell / 'config.txt', **self.config)
        (self.cell / 'run_exit').write_text('0\n')
        for filename in gate.SHADOW_FILES:
            (self.cell / filename).touch()

    def check(self, mode='check', names=None, pin=PIN):
        return gate.arm_receipt(self.root, mode, 'full', names or {CELL}, pin)

    def test_receipt_accepts_the_pinned_native_without_the_shadow_field(self):
        self.manifest['build_id_git_rev'] = gate.NATIVE
        write_fields(self.root / 'manifest', **self.manifest)
        del self.config['borg_shadow']
        write_fields(self.cell / 'config.txt', **self.config)
        for filename in gate.SHADOW_FILES:
            (self.cell / filename).unlink()
        self.check(mode='off', pin=gate.NATIVE)

    def test_missing_cell_and_missing_shadow_file_fail(self):
        with self.assertRaisesRegex(ValueError, 'missing cells'):
            self.check(names={CELL, 'full-0-warrior-1'})
        (self.cell / 'shadow-stats.jsonl').unlink()
        with self.assertRaisesRegex(ValueError, 'missing shadow-stats.jsonl'):
            self.check()

    def test_failed_cell_wrong_arm_and_changed_config_fail(self):
        (self.cell / 'run_exit').write_text('2\n')
        with self.assertRaisesRegex(ValueError, 'unsuccessful cell'):
            self.check()
        (self.cell / 'run_exit').write_text('0\n')
        with self.assertRaisesRegex(ValueError, 'expected recorded shadow mode switch'):
            self.check(mode='switch')
        self.config['borg_txt_sha256'] = '3' * 64
        write_fields(self.cell / 'config.txt', **self.config)
        with self.assertRaisesRegex(ValueError, 'effective settings hash differs'):
            self.check()

    def test_wrong_source_and_changed_binary_fail(self):
        with self.assertRaisesRegex(ValueError, 'expected build_id_git_rev'):
            self.check(pin='a' * 40)
        self.manifest['binary_sha256_end'] = '3' * 64
        write_fields(self.root / 'manifest', **self.manifest)
        with self.assertRaisesRegex(ValueError, 'binary_sha256_end differs'):
            self.check()


if __name__ == '__main__':
    unittest.main()

"""A merge with different folder suffixes must not hide numeric collisions."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('death_ids', Path(__file__).resolve().parents[1] / 'check-death-ids.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class DeathIdsTest(unittest.TestCase):
    def record(self, root, directory, label=None):
        path = root / directory / 'record.json'
        path.parent.mkdir()
        path.write_text(json.dumps({'id': label or directory}))

    def test_cross_track_suffixes_and_padding_do_not_hide_collisions(self):
        for other in ['D2449-rust-seed1043', 'D02449-rust-seed1043']:
            with self.subTest(other=other), tempfile.TemporaryDirectory() as name:
                root = Path(name)
                self.record(root, 'D2449-rust-seed1011')
                self.record(root, other)
                with self.assertRaisesRegex(ValueError, 'duplicate numeric dossier D2449'):
                    module.check(root)

    def test_legacy_label_is_retained_and_bad_identity_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.record(root, 'D0526-recorded', 'D0526')
            self.record(root, 'D2450-rust-seed102')
            self.assertEqual(module.check(root), 2)
            self.record(root, 'D2451-rust-seed1020', 'D2452-rust-seed1020')
            with self.assertRaisesRegex(ValueError, 'folder and record'):
                module.check(root)

    def test_missing_or_empty_registry_cannot_pass(self):
        with tempfile.TemporaryDirectory() as name:
            with self.assertRaisesRegex(ValueError, 'no dossier'):
                module.check(Path(name))
            with self.assertRaisesRegex(ValueError, 'missing registry'):
                module.check(Path(name) / 'missing')

    def test_full_qualified_storage_ids_are_not_numeric_zero(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for char in ('a', 'b'):
                self.record(root, 'D0000-q' + char * 64)
            self.assertEqual(module.check(root), 2)
            self.record(root, 'D0000-qshort')
            with self.assertRaisesRegex(ValueError, 'invalid qualified storage'):
                module.check(root)

"""Real-recording controls for immutable death identities and noninflated counts."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import borg_deaths as deaths
import borg_runtime as runtime

ROOT = Path(__file__).resolve().parents[1]


class DeathRecords(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        value = os.environ.get('BORG_DEATH_TEST_RECORDING')
        if not value:
            raise unittest.SkipTest('Set BORG_DEATH_TEST_RECORDING to a real completed natural-death bundle')
        cls.original = Path(value).resolve()
        assert deaths.describe(cls.original) is not None

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='borg-death-control-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / 'bundle'
        shutil.copytree(self.original, self.bundle)
        self.output = self.root / 'records/D0001-test'

    def capture(self):
        return deaths.write_dossier(self.bundle, self.output, 'D0001')

    def test_real_death_exports_native_identity_and_original_windows(self):
        record = self.capture()
        expected = json.loads((self.bundle / 'run/summary.json').read_text())
        self.assertEqual(record['raw_cause'], expected['death_cause'])
        death_turn = expected['game_turns']
        self.assertEqual(record['identity']['turn'], death_turn)
        self.assertEqual(json.loads((self.output / 'assessment.json').read_text())['classification'], 'unassessed')
        for window in record['recording']['selected_original_frames']:
            self.assertEqual(window['turn'], death_turn - window['offset_engine_turns'])
            self.assertTrue((self.output / window['text']).is_file())
        covered = {w['offset_engine_turns'] for w in record['recording']['selected_original_frames']}
        covered.update(w['offset_engine_turns'] for w in record['recording']['unavailable_offsets'])
        self.assertEqual(covered, set(deaths.OFFSETS))
        for anchor in record['recording']['nearby_original_anchors']:
            self.assertEqual(anchor['offset_engine_turns'], death_turn - anchor['turn'])
            self.assertIn('not an exact', anchor['qualification'])

    def test_cap_cannot_be_exported_as_death(self):
        path = self.bundle / 'recording.json'
        record = json.loads(path.read_text());record['summary']['dead'] = False
        runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'summary differs|natural death'):
            self.capture()
        self.assertFalse(self.output.exists())

    def test_tampered_raw_log_is_rejected_before_write(self):
        with (self.bundle / 'run/messages.log').open('a') as stream:
            stream.write('tampered')
        with self.assertRaisesRegex(ValueError, 'changed'):
            self.capture()
        self.assertFalse(self.output.exists())

    def test_identity_cannot_be_replaced_or_assessment_erased(self):
        self.capture()
        assessment = self.output / 'assessment.json';assessment.write_text('{"classification":"reviewed","actions":[]}\n')
        before = (self.output / 'record.json').read_bytes()
        self.capture()
        self.assertEqual(json.loads(assessment.read_text())['classification'], 'reviewed')
        path = self.bundle / 'recording.json';record = json.loads(path.read_text());record['build_id']['git_rev'] = 'different-source';runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'immutable'):
            self.capture()
        self.assertEqual((self.output / 'record.json').read_bytes(), before)

    def test_reexport_deduplicates_same_real_death(self):
        records = self.root / 'registry'
        with patch.dict(os.environ, {'BORG_DEATH_RECORDS_DIR': str(records)}):
            first = deaths.maybe_register(self.bundle)
            second = deaths.maybe_register(self.bundle)
        self.assertEqual(first, second)
        index = json.loads((records / 'index.json').read_text())
        self.assertEqual((index['cases'], index['distinct_starting_conditions'], index['death_process_attempts']), (1, 1, 1))

    def test_unchanged_repeat_is_not_a_second_starting_condition(self):
        record = deaths.write_dossier(self.bundle, self.output, 'D0001', repeat=[self.bundle])
        self.assertEqual(len(record['attempts']), 2)
        index, _ = deaths.aggregate(self.output.parent)
        self.assertEqual(index['distinct_starting_conditions'], 1)
        self.assertEqual(index['death_process_attempts'], 1)

    def test_repeat_divergence_is_rejected(self):
        repeated = self.root / 'repeat';shutil.copytree(self.bundle, repeated)
        path = repeated / 'recording.json';record = json.loads(path.read_text());record['build_id']['binary_sha256'] = 'changed';runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'different build|build binary differs'):
            deaths.write_dossier(self.bundle, self.output, 'D0001', repeat=[repeated])
        self.assertFalse(self.output.exists())

    def test_duplicate_dossier_does_not_inflate_family_or_attempt_count(self):
        self.capture()
        second = self.output.parent / 'D0002-test';shutil.copytree(self.output, second)
        path = second / 'record.json';record = json.loads(path.read_text());record['id'] = 'D0002';runtime.write(path, record)
        index, _ = deaths.aggregate(self.output.parent)
        self.assertEqual((index['cases'], index['distinct_starting_conditions'], index['death_process_attempts']), (2, 1, 1))

    def test_dossier_resolves_to_original_and_hash_mismatch_is_rejected(self):
        self.capture()
        spec = importlib.util.spec_from_file_location('replay_cli', ROOT / 'replay-borg.py');cli = importlib.util.module_from_spec(spec);spec.loader.exec_module(cli)
        self.assertEqual(cli.resolve_recording(self.output), self.bundle)
        path = self.bundle / 'recording.json';record = json.loads(path.read_text());record['character'] = 'changed';runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'manifest differs'):
            cli.resolve_recording(self.output)

    def test_issue_metadata_can_be_attached_without_replacing_evidence(self):
        self.capture()
        before = (self.output / 'terminal.map.txt').read_bytes()
        record = deaths.write_dossier(self.bundle, self.output, 'D0001', issue='https://example.test/investigation')
        self.assertEqual(record['investigation'], 'https://example.test/investigation')
        self.assertEqual((self.output / 'terminal.map.txt').read_bytes(), before)

    def test_curated_native_evidence_cannot_be_silently_replaced(self):
        self.capture()
        with (self.output / 'terminal.map.txt').open('a') as output:
            output.write('changed')
        with self.assertRaisesRegex(ValueError, 'evidence changed'):
            self.capture()

    def test_legacy_dossier_cannot_claim_recorded_movie(self):
        self.output.mkdir(parents=True)
        runtime.write(self.output / 'record.json', {'recording': None})
        spec = importlib.util.spec_from_file_location('replay_cli_legacy', ROOT / 'replay-borg.py');cli = importlib.util.module_from_spec(spec);spec.loader.exec_module(cli)
        with self.assertRaisesRegex(ValueError, 'no complete recording'):
            cli.resolve_recording(self.output)


if __name__ == '__main__':
    unittest.main()

"""Backup/lock safety and real frozen-engine recording/replay controls."""
import argparse
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import borg_runtime as runtime
import borg_saves as saves


class SaveSafety(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='borg-save-test-')
        self.addCleanup(self.tmp.cleanup)
        self.save = Path(self.tmp.name) / 'Hero'

    def test_five_slots_restore_preserves_current(self):
        for n in range(7):
            self.save.write_text(str(n))
            saves.backup(self.save)
        folder = self.save.parent / 'backups/Hero'
        self.assertEqual([(folder / f'save.{n}').read_text() for n in range(1, 6)], ['6','5','4','3','2'])
        self.save.write_text('current')
        saves.restore(self.save, 3)
        self.assertEqual(self.save.read_text(), '4')
        self.assertEqual(next(folder.glob('before-restore-*')).read_text(), 'current')

    def test_publish_rejects_noncooperating_writer(self):
        self.save.write_text('old');original = saves.digest(self.save)
        final = self.save.parent / 'final';final.write_text('new')
        self.save.write_text('outside writer')
        with self.assertRaisesRegex(ValueError, 'changed'):
            saves.publish(self.save, final, original)
        self.assertEqual(self.save.read_text(), 'outside writer')

    def test_same_character_lock_conflicts(self):
        with runtime.character_lock(self.save):
            with self.assertRaisesRegex(ValueError, 'already open'):
                with runtime.character_lock(self.save):
                    pass

    def test_name_cannot_escape_save_root(self):
        for value in ('../Hero', '/tmp/Hero', '..', 'a/b', ''):
            with self.subTest(value=value), self.assertRaises(ValueError):
                runtime.character_name(value)


class RecordingSafety(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory(prefix='borg-record-test-')
        cls.previous = os.environ.get('BORG_PLAY_DATA_DIR')
        os.environ['BORG_PLAY_DATA_DIR'] = cls.tmp.name
        args = argparse.Namespace(character='Recorded', new=True, from_backup=None,
                                  seed=42, race=0, cls=0, turns=1000, keys=1000,
                                  wall_seconds=30, checkpoint_every=100)
        cls.recording = runtime.record_run(args)

    @classmethod
    def tearDownClass(cls):
        if cls.previous is None:
            os.environ.pop('BORG_PLAY_DATA_DIR', None)
        else:
            os.environ['BORG_PLAY_DATA_DIR'] = cls.previous
        cls.tmp.cleanup()

    def clone(self):
        tmp = tempfile.TemporaryDirectory(prefix='borg-record-negative-')
        self.addCleanup(tmp.cleanup)
        target = Path(tmp.name) / 'record'
        shutil.copytree(self.recording, target)
        return target

    def test_real_recording_replays_exactly(self):
        self.assertTrue(runtime.verify(self.recording)['verified'])

    def test_modified_state_is_rejected_before_replay(self):
        copy = self.clone()
        with (copy / 'run/state.log').open('ab') as f:f.write(b'corruption\n')
        with self.assertRaisesRegex(ValueError, 'Recording file changed'):
            runtime.verify(copy)

    def test_modified_engine_is_rejected(self):
        copy = self.clone()
        with (copy / 'runtime/game/angband').open('ab') as f:f.write(b'changed')
        with self.assertRaisesRegex(ValueError, 'Recording file changed'):
            runtime.verify(copy)

    def test_wrong_rerun_seed_cannot_be_verified(self):
        copy = self.clone()
        record = json.loads((copy / 'recording.json').read_text());record['profile']['seed'] = 43
        runtime.write(copy / 'recording.json', record)
        with self.assertRaisesRegex(ValueError, 'diverges'):
            runtime.verify(copy)

    def test_legacy_recording_is_not_seed_only_replay(self):
        copy = self.clone()
        record = json.loads((copy / 'recording.json').read_text());record['schema'] = 'legacy'
        runtime.write(copy / 'recording.json', record)
        with self.assertRaisesRegex(ValueError, 'Unsupported'):
            runtime.verify(copy)

    def test_incomplete_recording_is_only_viewable_unverified(self):
        copy = self.clone()
        record = json.loads((copy / 'recording.json').read_text())
        record['status'] = 'failed';record.pop('files_sha256')
        runtime.write(copy / 'recording.json', record)
        self.assertEqual(runtime.validate(copy, require_complete=False)['status'], 'failed')
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            runtime.verify(copy)


if __name__ == '__main__':
    unittest.main()

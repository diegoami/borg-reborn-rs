"""Failure controls for a real GUI recording (created by gui-replay/run.py)."""
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

import borg_gui as gui
import borg_runtime as runtime


@unittest.skipUnless(os.environ.get('BORG_GUI_TEST_RECORDING'), 'set BORG_GUI_TEST_RECORDING to a complete GUI fixture recording')
class GuiRecordingControls(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='borg-gui-control-')
        self.addCleanup(self.tmp.cleanup)
        self.bundle = Path(self.tmp.name) / 'recording'
        shutil.copytree(os.environ['BORG_GUI_TEST_RECORDING'], self.bundle)
        (self.bundle / 'verification.json').unlink(missing_ok=True)

    def rehash(self, relative):
        path = self.bundle / 'recording.json'
        record = json.loads(path.read_text())
        record['files_sha256'][relative] = runtime.sha(self.bundle / relative)
        runtime.write(path, record)

    def test_original_reruns(self):
        self.assertTrue(gui.verify(self.bundle)['verified'])

    def test_changed_frozen_executable_rejected(self):
        binary = self.bundle / 'runtime/game/angband'
        with binary.open('ab') as output:
            output.write(b'changed')
        with self.assertRaisesRegex(ValueError, 'file changed'):
            gui.verify(self.bundle)

    def test_absent_event_stream_rejected(self):
        (self.bundle / 'run/events.txt').unlink()
        with self.assertRaises((OSError, ValueError)):
            gui.verify(self.bundle)

    def test_extra_terminal_input_cannot_pass_as_matching_prefix(self):
        path = self.bundle / 'run/events.txt'
        with path.open('a') as output:
            output.write(path.read_text().splitlines()[0] + '\n')
        self.rehash('run/events.txt')
        with self.assertRaisesRegex(ValueError, 'diverges|complete input'):
            gui.verify(self.bundle)

    def test_clock_exhaustion_cannot_pass(self):
        path = self.bundle / 'run/clock.txt'
        path.write_text('')
        self.rehash('run/clock.txt')
        with self.assertRaisesRegex(ValueError, 'failed|diverges|complete input'):
            gui.verify(self.bundle)

    def test_recorded_actions_are_compared_to_actual_rust_output(self):
        path = self.bundle / 'run/actions.txt'
        lines = path.read_text().splitlines()
        self.assertTrue(lines, 'Use the actual Borg intervention fixture')
        columns = lines[0].split()
        columns[3] = str(int(columns[3]) + 1)
        path.write_text(' '.join(columns) + '\n' + '\n'.join(lines[1:]) + '\n')
        self.rehash('run/actions.txt')
        with self.assertRaisesRegex(ValueError, 'actions.txt'):
            gui.verify(self.bundle)

    def test_changed_file_input_cannot_pass(self):
        record = json.loads((self.bundle / 'recording.json').read_text())
        origin = (self.bundle / 'initial-profile/save' / record['character']).read_bytes()
        path = next(p for p in (self.bundle / 'run').glob('file-input-*.bin') if p.read_bytes() == origin)
        path.write_bytes(b'not an Angband save\n')
        self.rehash(str(path.relative_to(self.bundle)))
        with self.assertRaisesRegex(ValueError, 'failed|diverges|complete input'):
            gui.verify(self.bundle)

    def test_changed_native_save_cannot_pass(self):
        path = self.bundle / 'final.save'
        path.write_bytes(path.read_bytes() + b'changed')
        self.rehash('final.save')
        record_path = self.bundle / 'recording.json'
        record = json.loads(record_path.read_text())
        record['final_sha256'] = runtime.sha(path)
        runtime.write(record_path, record)
        with self.assertRaisesRegex(ValueError, 'final native save'):
            gui.verify(self.bundle)

    def test_incomplete_requires_explicit_unverified_view(self):
        path = self.bundle / 'recording.json'
        record = json.loads(path.read_text())
        record['status'] = 'incomplete'
        record.pop('files_sha256')
        runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            gui.verify(self.bundle)
        self.assertEqual(gui.validate(self.bundle, require_complete=False)['status'], 'incomplete')

    def test_declared_native_message_stream_cannot_be_removed_with_its_hash(self):
        path = self.bundle / 'recording.json'
        record = json.loads(path.read_text())
        stream = self.bundle / 'run/native-messages.jsonl'
        if not stream.exists():
            self.skipTest('Use a new GUI recording with the native message stream')
        stream.unlink()
        record.setdefault('captured_views', {})['native_messages'] = True
        record['files_sha256'].pop('run/native-messages.jsonl')
        runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'missing required'):
            gui.verify(self.bundle)

    def test_native_message_mutation_after_rehash_is_rejected(self):
        stream = self.bundle / 'run/native-messages.jsonl'
        if not stream.exists():
            self.skipTest('Use a new GUI recording with the native message stream')
        with stream.open('a') as output:
            output.write('{"frame":99999,"turn":99999,"type":0,"text":"invented"}\n')
        self.rehash('run/native-messages.jsonl')
        with self.assertRaisesRegex(ValueError, 'native-messages.jsonl'):
            gui.verify(self.bundle)

    def test_manifest_cannot_omit_required_or_add_untracked_input(self):
        path = self.bundle / 'recording.json'
        record = json.loads(path.read_text())
        record['files_sha256'].pop('run/clock.txt')
        runtime.write(path, record)
        with self.assertRaisesRegex(ValueError, 'missing required'):
            gui.verify(self.bundle)


if __name__ == '__main__':
    unittest.main()

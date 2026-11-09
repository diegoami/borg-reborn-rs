"""Takeover trust-boundary and CLI controls; real X11 checks are separate."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import unittest
from unittest import mock

import human_takeover as human

ROOT = human.ROOT


class TakeoverControls(unittest.TestCase):
    def test_sigterm_becomes_cleanup_exception_and_handler_is_restored(self):
        before = signal.getsignal(signal.SIGTERM)
        with self.assertRaises(InterruptedError), human.owned_child_signals():
            signal.raise_signal(signal.SIGTERM)
        self.assertEqual(signal.getsignal(signal.SIGTERM), before)

    def test_loaded_state_requires_identical_rng_and_player(self):
        expected = {'turn': 42, 'x': 2, 'hp': 7, 'rng': [1, 2, 3], 'borg_active': True, 'dead': False}
        actual = {**expected, 'borg_active': False}
        human.same_state(expected, actual)
        for field, value in [('rng', [1, 2, 4]), ('turn', 43), ('hp', 6), ('x', 3), ('borg_active', True), ('dead', True)]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                human.same_state(expected, {**actual, field: value})

    def test_patch_requires_exact_unique_sites(self):
        for text in ('no historical site', 'hook hook'):
            with self.assertRaises(ValueError):
                human.replace_once(text, 'hook', 'capture')

    def test_gui_recording_cannot_be_misread_as_headless(self):
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory)
            (bundle / 'recording.json').write_text(json.dumps({'schema': 'borg-rs-gui-recording@1'}))
            with mock.patch.object(human, 'qualified_engine') as build:
                with self.assertRaisesRegex(ValueError, 'Legacy GUI.*frozen'):
                    human.prepare(bundle, 1)
                build.assert_not_called()

    def test_branch_input_tamper_fails_before_gui(self):
        with tempfile.TemporaryDirectory() as directory:
            branch = Path(directory)
            (branch / 'engine').write_text('tampered')
            (branch / 'human-branch.json').write_text(json.dumps({'schema': human.SCHEMA, 'status': 'ready',
                'files_sha256': {'engine': 'wrong'}}))
            with self.assertRaisesRegex(ValueError, 'frozen input changed'):
                human.launch(branch)

    def test_cli_rejects_conflicting_or_incomplete_modes(self):
        for args in [('--play-from-turn', '-1'), ('--prepare-only',), ('--play-from-turn', '2', '--view-only'),
                     ('--play-from-turn', '2', '--from-turn', '1'), ('--play-from-turn', '2', '--verify-only')]:
            with self.subTest(args=args):
                result = subprocess.run([str(ROOT / 'replay-borg.py'), '/not/a/recording', *args], capture_output=True)
                self.assertEqual(result.returncode, 2)


@unittest.skipUnless(os.environ.get('BORG_HUMAN_TEST_BRANCH'), 'set BORG_HUMAN_TEST_BRANCH to a captured branch')
class CapturedPrefixControls(unittest.TestCase):
    def setUp(self):
        self.branch = Path(os.environ['BORG_HUMAN_TEST_BRANCH'])
        self.record = json.loads((self.branch / 'human-branch.json').read_text())
        self.bundle = Path(self.record['original_recording'])

    def test_original_prefix_qualifies(self):
        boundary, state = human.check_prefix(self.bundle, self.branch / 'prefix', self.branch / 'capture')
        self.assertEqual(boundary, self.record['boundary'])
        human.same_state(state, self.record['initial_state'])

    def test_changes_to_each_original_channel_are_rejected(self):
        for name in ('keys.log', 'state.log', 'frames.jsonl', 'messages.log', 'final.save'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                prefix = Path(temporary) / 'prefix'
                shutil.copytree(self.branch / 'prefix', prefix)
                path = prefix / name
                if name == 'frames.jsonl':
                    lines = path.read_bytes().splitlines(keepends=True)
                    frame = json.loads(lines[0]);frame['turn'] += 1
                    lines[0] = (json.dumps(frame) + '\n').encode()
                    path.write_bytes(b''.join(lines))
                else:
                    with path.open('ab') as stream:
                        stream.write(b'corruption\n')
                with self.assertRaises(ValueError):
                    human.check_prefix(self.bundle, prefix, self.branch / 'capture')

    def test_truncated_message_prefix_and_missing_checkpoints_are_rejected(self):
        for problem in ('messages', 'checkpoint'):
            with self.subTest(problem=problem), tempfile.TemporaryDirectory() as temporary:
                prefix = Path(temporary) / 'prefix'
                shutil.copytree(self.branch / 'prefix', prefix)
                if problem == 'messages':
                    (prefix / 'messages.log').write_bytes(b'')
                else:
                    next((prefix / 'checkpoints').glob('*.save')).unlink()
                with self.assertRaises(ValueError):
                    human.check_prefix(self.bundle, prefix, self.branch / 'capture')

    def test_preparation_refuses_existing_output(self):
        with mock.patch.object(human, 'qualified_engine') as build:
            with self.assertRaisesRegex(ValueError, 'already exists'):
                human.prepare(self.bundle, self.record['boundary']['requested_turn'], self.branch)
            build.assert_not_called()


if __name__ == '__main__':
    unittest.main()

"""Regression controls for the M5 medium/low tooling findings."""
from argparse import Namespace
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import borg_runtime as runtime
import borg_deaths as deaths


class PreflightSafety(unittest.TestCase):
    def test_unavailable_build_cannot_restore_or_rotate_backups(self):
        with tempfile.TemporaryDirectory(prefix='borg-m5-preflight-') as temporary:
            root = Path(temporary)
            save = root / 'save/Hero'
            slot = root / 'save/backups/Hero/save.2'
            slot.parent.mkdir(parents=True)
            save.write_bytes(b'current save')
            slot.write_bytes(b'selected backup')
            args = Namespace(character='Hero', new=False, from_backup=2)
            before = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
            with patch.dict(os.environ, {'BORG_PLAY_DATA_DIR': str(root)}), patch.object(runtime, 'build', side_effect=ValueError('build unavailable')):
                with self.assertRaisesRegex(ValueError, 'build unavailable'):
                    runtime.record_run(args)
            after = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file() and '.locks' not in p.parts}
            self.assertEqual(after, before)


    @unittest.skipUnless(os.environ.get('BORG_M5_CONTINUATION_RESULT'), 'Set BORG_M5_CONTINUATION_RESULT to exercise actual dead-backup metadata')
    def test_actual_invalid_backup_candidates_leave_named_and_backup_bytes_unchanged(self):
        root = runtime.ROOT
        directory, binary, identity = runtime.build()
        for problem in ('race', 'class', 'dead', 'corrupt'):
            with self.subTest(problem=problem), tempfile.TemporaryDirectory(prefix='borg-m5-rejected-') as temporary:
                profile = Path(temporary)
                save = profile / 'save/Hero'
                slot = profile / 'save/backups/Hero/save.2'
                slot.parent.mkdir(parents=True)
                shutil.copy2(root / 'harness/saves/depth50.save', save)
                if problem == 'corrupt':
                    slot.write_bytes(b'not an Angband save')
                elif problem == 'dead':
                    result = json.loads(Path(os.environ['BORG_M5_CONTINUATION_RESULT']).read_text())
                    shutil.copy2(Path(result['first']) / 'run/final.save', slot)
                else:
                    shutil.copy2(root / 'docs/validation/2026-11-07-post-m4-investigation/death-checkpoints/live-druid.save', slot)
                args = Namespace(character='Hero', new=False, from_backup=2, race=3 if problem == 'race' else None,
                                 cls=6 if problem == 'class' else None)
                before = {str(p.relative_to(profile)): p.read_bytes() for p in profile.rglob('*') if p.is_file()}
                with patch.dict(os.environ, {'BORG_PLAY_DATA_DIR': str(profile)}), patch.object(runtime, 'build', return_value=(directory, binary, identity)):
                    with self.assertRaises(ValueError):
                        runtime.record_run(args)
                after = {str(p.relative_to(profile)): p.read_bytes() for p in profile.rglob('*') if p.is_file() and '.locks' not in p.parts}
                self.assertEqual(after, before)


@unittest.skipUnless(os.environ.get('BORG_M5_CONTINUATION_RESULT'), 'Set BORG_M5_CONTINUATION_RESULT to the actual same-origin two-run result.json')
class ContinuationCounting(unittest.TestCase):
    def test_real_seed_variants_preserve_two_attempts_under_one_case(self):
        result = json.loads(Path(os.environ['BORG_M5_CONTINUATION_RESULT']).read_text())
        first, second = (Path(result[key]) for key in ('first', 'second'))
        self.assertTrue(result['same_origin'])
        self.assertTrue(all(result['gameplay_equal'].values()))
        self.assertTrue(result['refusal_preserved_save'])
        self.assertTrue(result['refusal_preserved_backups'])
        a, b = deaths.describe(first), deaths.describe(second)
        self.assertEqual(a['identity'], b['identity'])
        self.assertIsNone(a['identity']['seed'])
        self.assertNotEqual(a['original']['profile']['seed'], b['original']['profile']['seed'])
        with tempfile.TemporaryDirectory(prefix='borg-m5-counting-') as temporary:
            registry = Path(temporary) / 'deaths'
            with patch.dict(os.environ, {'BORG_DEATH_RECORDS_DIR': str(registry)}):
                left, right = deaths.maybe_register(first), deaths.maybe_register(second)
            self.assertEqual(left, right)
            index = json.loads((registry / 'index.json').read_text())
            self.assertEqual((index['cases'], index['distinct_starting_conditions'], index['death_process_attempts']), (1, 1, 2))
            record = json.loads((left / 'record.json').read_text())
            self.assertEqual({attempt['requested_seed'] for attempt in record['attempts']}, {a['original']['profile']['seed'], b['original']['profile']['seed']})

    def test_legacy_continuation_families_group_without_rewriting_identity(self):
        before = os.environ.get('BORG_M5_LEGACY_REGISTRY')
        if not before:
            self.skipTest('Set BORG_M5_LEGACY_REGISTRY to the before-fix registry')
        with tempfile.TemporaryDirectory(prefix='borg-m5-legacy-') as temporary:
            target = Path(temporary) / 'deaths'
            shutil.copytree(before, target)
            originals = {str(p.relative_to(target)): p.read_bytes() for p in target.glob('D*/record.json')}
            deaths.update_index(target)
            index = json.loads((target / 'index.json').read_text())
            self.assertEqual((index['cases'], index['distinct_starting_conditions'], index['death_process_attempts']), (2, 1, 2))
            for name, data in originals.items():
                self.assertEqual((target / name).read_bytes(), data)


if __name__ == '__main__':
    unittest.main()

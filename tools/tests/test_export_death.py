"""Tests for tools/export-death.py on synthetic run directories.

Run: python3 -m unittest discover -s tools/tests
"""

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('export_death', HERE.parent / 'export-death.py')
ed = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ed)


def make_run(base, name, dead=True, seed=101, keys='abc\n'):
    run = Path(base) / name
    run.mkdir(parents=True)
    (run / 'summary.json').write_text(json.dumps({
        'seed': seed, 'race': 'Human', 'race_idx': 0, 'class': 'Warrior', 'class_idx': 0,
        'stop_reason': 'death' if dead else 'turns', 'game_turns': 12345, 'clevel': 4,
        'max_depth': 3, 'depth': 2, 'dead': dead, 'death_cause': 'a cave spider' if dead else '',
        'keys': 99}))
    (run / 'keys.log').write_text(keys)
    (run / 'messages.log').write_text('You die.\n')
    (run / 'config.txt').write_text('flags=-\nborg_txt_sha256=feed\n')
    return run


class ExportTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.reg = self.base / 'deaths'
        os.environ['BORG_DEATH_RECORDS_DIR'] = str(self.reg)
        self.build = self.base / 'BUILD_ID'
        self.build.write_text('impl=c\ngit_rev=1234567\ngit_dirty=no\nbinary_sha256=beef\nsource_sha256=cafe\n')

    def tearDown(self):
        os.environ.pop('BORG_DEATH_RECORDS_DIR', None)
        self.tmp.cleanup()

    def test_alive_run_writes_nothing(self):
        run = make_run(self.base, 'alive', dead=False)
        self.assertEqual(ed.export(run, 'c', self.build), (None, 'not dead'))
        self.assertFalse(self.reg.exists() and any(self.reg.iterdir()))

    def test_death_writes_a_dossier(self):
        run = make_run(self.base, 'dead')
        case, what = ed.export(run, 'c', self.build, 'cheat')
        self.assertEqual((case, what), ('D0001-c-seed101', 'exported'))
        d = self.reg / case
        rec = json.loads((d / 'record.json').read_text())
        self.assertEqual(rec['terminal']['cause'], 'a cave spider')
        self.assertEqual(rec['terminal']['turn'], 12345)
        self.assertEqual(rec['identity']['flags'], 'cheat')
        self.assertEqual(rec['identity']['binary_sha256'], 'beef')
        self.assertIn('keys.log', rec['attempts'][0]['raw_sha256'])
        self.assertTrue(rec['encounter_family'].startswith('start-'))
        self.assertEqual(json.loads((d / 'assessment.json').read_text())['classification'], 'unassessed')
        self.assertIn('a cave spider at turn 12345', (d / 'report.md').read_text())
        self.assertFalse((d / 'keys.log').exists(), 'raw logs are not copied into the dossier')

    def test_partial_suite_directory_is_recorded_as_final(self):
        run = make_run(self.base / 'out.partial.4242', 'dead')
        case, _ = ed.export(run, 'c', self.build)
        rec = json.loads((self.reg / case / 'record.json').read_text())
        self.assertEqual(rec['attempts'][0]['raw_directory'], str((self.base / 'out' / 'dead').resolve()))

    def test_repeat_is_not_a_new_case(self):
        ed.export(make_run(self.base, 'one'), 'c', self.build)
        self.assertEqual(ed.export(make_run(self.base, 'two'), 'c', self.build), ('D0001-c-seed101', 'repeat'))
        self.assertEqual(len([d for d in self.reg.iterdir() if d.is_dir()]), 1)

    def test_interrupted_dossier_is_repaired_not_accepted(self):
        case, _ = ed.export(make_run(self.base, 'one'), 'c', self.build)
        d = self.reg / case
        (d / 'report.md').unlink()
        (d / 'assessment.json').unlink()
        record_before = (d / 'record.json').read_bytes()
        got = ed.export(make_run(self.base, 'two'), 'c', self.build)
        self.assertEqual(got, (case, 'repaired: assessment.json, report.md'))
        self.assertTrue((d / 'report.md').exists() and (d / 'assessment.json').exists())
        self.assertEqual((d / 'record.json').read_bytes(), record_before, 'existing files are kept')
        self.assertEqual(ed.export(make_run(self.base, 'three'), 'c', self.build), (case, 'repeat'))

    def test_filled_in_assessment_is_never_overwritten(self):
        case, _ = ed.export(make_run(self.base, 'one'), 'c', self.build)
        a = self.reg / case / 'assessment.json'
        a.write_text('{"classification": "assessed by a person"}')
        (self.reg / case / 'report.md').unlink()
        ed.export(make_run(self.base, 'two'), 'c', self.build)
        self.assertIn('assessed by a person', a.read_text())

    def test_stale_temporary_directory_is_removed(self):
        self.reg.mkdir(parents=True)
        (self.reg / '.tmp-D0001-c-seed101').mkdir()
        (self.reg / '.tmp-D0001-c-seed101' / 'record.json').write_text('{half')
        case, what = ed.export(make_run(self.base, 'one'), 'c', self.build)
        self.assertEqual((case, what), ('D0001-c-seed101', 'exported'))
        self.assertFalse((self.reg / '.tmp-D0001-c-seed101').exists())

    def test_case_without_record_fails_loudly(self):
        (self.reg / 'D0001-c-seed101').mkdir(parents=True)
        with self.assertRaises(SystemExit):
            ed.export(make_run(self.base, 'one'), 'c', self.build)

    def test_rebuilt_binary_of_same_source_is_a_repeat(self):
        case, _ = ed.export(make_run(self.base, 'one'), 'c', self.build)
        rebuilt = self.base / 'BUILD_ID2'
        rebuilt.write_text('impl=c\ngit_rev=89abcde\ngit_dirty=yes\nbinary_sha256=0ther\nsource_sha256=cafe\n')
        self.assertEqual(ed.export(make_run(self.base, 'two'), 'c', rebuilt), (case, 'repeat'))

    def test_changed_source_is_a_new_family(self):
        ed.export(make_run(self.base, 'one'), 'c', self.build)
        changed = self.base / 'BUILD_ID2'
        changed.write_text('impl=c\ngit_rev=89abcde\ngit_dirty=no\nbinary_sha256=beef\nsource_sha256=d1ff\n')
        case, what = ed.export(make_run(self.base, 'two'), 'c', changed)
        self.assertEqual((case, what), ('D0002-c-seed101', 'exported'))

    def test_numbering_and_family(self):
        ed.export(make_run(self.base, 'one'), 'c', self.build)
        case, _ = ed.export(make_run(self.base, 'two', seed=102), 'rust', self.build)
        self.assertEqual(case, 'D0002-rust-seed102')
        fam = {json.loads((d / 'record.json').read_text())['encounter_family']
               for d in self.reg.iterdir() if d.is_dir()}
        self.assertEqual(len(fam), 2, 'different seed and implementation: different start')

    def test_same_start_different_play_is_same_family(self):
        a, _ = ed.export(make_run(self.base, 'one'), 'c', self.build)
        b, what = ed.export(make_run(self.base, 'two', keys='other\n'), 'c', self.build)
        self.assertEqual(what, 'exported')
        fams = [json.loads((self.reg / c / 'record.json').read_text())['encounter_family'] for c in (a, b)]
        self.assertEqual(fams[0], fams[1])


if __name__ == '__main__':
    unittest.main()

"""Real capture/application tests for the restricted declarative P1 tier."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'build/rust/game/angband'
FIXTURE = ROOT / 'engine/tests/fixtures/warrior-level5.build'


@unittest.skipUnless(BIN.exists(), 'build required')
class CharacterBuildTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.env = {key: os.environ[key] for key in ('PATH', 'PYTHONPATH', 'LD_LIBRARY_PATH') if key in os.environ}
        # The capture expects the reborn core's character (#606: the harness
        # default is the borg-rs core).
        self.env.update(BORG_CORE='reborn', BORG_DEATH_RECORDS_DIR=str(self.base / 'natural'), RUN_TIMEOUT_SECONDS='30')
        if 'BORG_CEMETERY_DIR' in os.environ:
            self.env['BORG_CEMETERY_DIR'] = os.environ['BORG_CEMETERY_DIR']

    def tearDown(self):
        self.tmp.cleanup()

    def launch(self, name, flags='menu,start-depth=10', extra=(), turns=1000, player_class=0):
        out = self.base / name
        result = subprocess.run(['bash', str(ROOT / 'harness/run.sh'), str(out), '101', '0', str(player_class), str(turns), flags,
                                 '--', *extra], env=self.env, capture_output=True, text=True, timeout=45)
        return out, result

    def test_observed_tier_applies_exact_hash_level_and_native_hitpoints(self):
        runs = []
        for index in range(2):
            out, result = self.launch(f'record-{index}', extra=('-start-build', str(FIXTURE)))
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads((out / 'summary.json').read_text())
            self.assertEqual(summary['start_build'], 'record')
            self.assertEqual(summary['synthetic_contract'], 'borg-reborn-rs/synthetic-start@2')
            self.assertEqual(summary['start_build_sha256'], hashlib.sha256(FIXTURE.read_bytes()).hexdigest())
            self.assertEqual(summary['initial_clevel'], 5)
            self.assertEqual(summary['initial_max_hp'], 72)
            self.assertEqual(summary['initial_max_sp'], 0)
            self.assertFalse(summary['loaded'])
            self.assertIn('hp72/72', (out / 'state.log').read_text().splitlines()[0])
            self.assertIn('start_build_sha256=' + summary['start_build_sha256'], (out / 'config.txt').read_text())
            self.assertFalse((out / 'save/input.save').exists())
            runs.append(out)
        for name in ('keys.log', 'state.log', 'messages.log'):
            self.assertEqual((runs[0] / name).read_bytes(), (runs[1] / name).read_bytes())
        self.assertFalse((self.base / 'natural').exists())

    def test_capture_is_a_read_only_diagnostic_with_an_engine_digest(self):
        record = self.base / 'observed.build'
        measured, result = self.launch('capture', 'menu', ('-capture-build', str(record), '-capture-build-level', '5'), turns=20000)
        self.assertEqual(result.returncode, 0, result.stderr)
        control, result = self.launch('control', 'menu', turns=20000)
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads((measured / 'summary.json').read_text())
        self.assertEqual(summary['build_capture_level'], 5)
        self.assertEqual(summary['build_capture_status'], 'captured')
        self.assertEqual(summary['build_capture_requested_level'], 5)
        self.assertEqual(summary['build_capture_sha256'], hashlib.sha256(record.read_bytes()).hexdigest())
        self.assertEqual(record.read_bytes(), FIXTURE.read_bytes())
        for name in ('keys.log', 'state.log', 'messages.log'):
            self.assertEqual((measured / name).read_bytes(), (control / name).read_bytes(), name)
        self.assertNotIn('synthetic', summary)

    def test_capture_never_overwrites_an_existing_file(self):
        # #858: the record is created new (fopen "x" on POSIX, _O_EXCL on
        # Windows); an existing file is refused and left as it was.
        record = self.base / 'existing.build'
        record.write_bytes(b'keep me\n')
        measured, result = self.launch('existing', 'menu', ('-capture-build', str(record), '-capture-build-level', '5'), turns=20000)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('cannot create synthetic build capture', (measured / 'stderr.log').read_text())
        self.assertEqual(record.read_bytes(), b'keep me\n')

    def test_unreached_capture_is_explicit_without_changing_play(self):
        record = self.base / 'unreached.build'
        measured, result = self.launch('unreached', 'menu',
            ('-capture-build', str(record), '-capture-build-level', '15'), turns=300)
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads((measured / 'summary.json').read_text())
        self.assertEqual(summary['build_capture_status'], 'absent')
        self.assertEqual(summary['build_capture_requested_level'], 15)
        self.assertNotIn('build_capture_sha256', summary)
        self.assertNotIn('build_capture_turn', summary)
        self.assertFalse(record.exists())
        control, result = self.launch('unreached-control', 'menu', turns=300)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('build_capture_status', json.loads((control / 'summary.json').read_text()))
        for name in ('keys.log', 'state.log', 'messages.log'):
            self.assertEqual((measured / name).read_bytes(), (control / name).read_bytes())

    def test_invalid_or_unsupported_records_never_fall_back_to_birth(self):
        original = FIXTURE.read_text()
        bad = [original.replace('character-build@1', 'character-build@9'),
               original.replace('identity:0:0:5', 'identity:1:0:5'),
               original.replace('identity:0:0:5', 'identity:0:0:50'),
               original.replace('vitals:70:84:72:0', 'vitals:70:84:73:0'),
               original.replace('vitals:70:84:72:0', 'vitals:0:84:72:0'),
               original.replace('stats:17:10:10:16:16', 'stats:999:10:10:16:16'),
               original.replace('hp-rolls:19:', 'hp-rolls:20:'),
               original.replace('item:16:2:1:6:', 'artifact:16:2:1:6:'),
               original.replace('end\n', 'unknown:1\nend\n'), original + 'trailing\n', original[:-1],
               original.replace('knowledge:', '\nknowledge:')]
        for index, text in enumerate(bad):
            record = self.base / f'bad-{index}.build'
            record.write_text(text)
            out, result = self.launch(f'bad-{index}', extra=('-start-build', str(record)))
            self.assertNotEqual(result.returncode, 0, index)
            self.assertFalse((out / 'summary.json').exists(), index)
        _, result = self.launch('without-depth', 'menu', ('-start-build', str(FIXTURE)))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('requires start-depth', result.stderr)
        _, result = self.launch('duplicate', extra=('-start-build', str(FIXTURE), '-start-build', str(FIXTURE)))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('duplicate start-build', result.stderr)

    def test_a_record_names_the_class_the_run_started(self):
        # The tier is no longer Human Warrior only: the record's race and
        # class are the started character's, its mana is checked against the
        # native calculation like its hit points, and nothing else about the
        # class (spell knowledge) is restored.
        for name, player_class, hp, sp in (('ranger', 7, 57, 6), ('necromancer', 4, 40, 12)):
            fixture = ROOT / f'engine/tests/fixtures/{name}-level5.build'
            out, result = self.launch(name, extra=('-start-build', str(fixture)), player_class=player_class)
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads((out / 'summary.json').read_text())
            self.assertEqual(summary['start_build_sha256'], hashlib.sha256(fixture.read_bytes()).hexdigest())
            self.assertEqual((summary['initial_clevel'], summary['initial_max_hp'], summary['initial_max_sp']), (5, hp, sp))
            self.assertFalse((out / 'save/input.save').exists())
            # A record for another class, or another mana value, never falls back to birth.
            for index, text in enumerate((
                    fixture.read_text(),  # started as the Warrior below
                    fixture.read_text().replace(f'identity:0:{player_class}:5', 'identity:0:0:5'),
                    fixture.read_text().replace(f':{hp}:{sp}\n', f':{hp}:{sp + 1}\n'))):
                record = self.base / f'{name}-bad-{index}.build'
                record.write_text(text)
                bad, result = self.launch(f'{name}-bad-{index}', extra=('-start-build', str(record)),
                                          player_class=0 if index == 0 else player_class)
                self.assertNotEqual(result.returncode, 0, (name, index))
                self.assertFalse((bad / 'summary.json').exists(), (name, index))

    def test_both_cores_start_only_after_the_requested_cave_exists(self):
        for core in ('reborn', 'borg-rs'):
            self.env['BORG_CORE'] = core
            for variant, extra in (('birth', ()), ('record', ('-start-build', str(FIXTURE)))):
                outputs = []
                for repeat in range(2):
                    out, result = self.launch(f'{core}-{variant}-{repeat}', extra=extra, turns=300)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    summary = json.loads((out / 'summary.json').read_text())
                    self.assertEqual(summary['initial_cave_depth'], 10)
                    self.assertIs(summary['initial_generation_pending'], False)
                    self.assertIn(summary['stop_reason'], ('death', 'turns'))
                    self.assertGreater(summary['keys'], 0)
                    outputs.append(out)
                for name in ('keys.log', 'state.log', 'messages.log'):
                    self.assertEqual((outputs[0] / name).read_bytes(), (outputs[1] / name).read_bytes())
        self.assertFalse((self.base / 'natural').exists())

    def test_saved_synthetic_lineage_cannot_resume_as_a_natural_run(self):
        out, result = self.launch('saved-synthetic', 'menu,start-depth=5,living-save,key-limit=1')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((out / 'final.save').is_file())
        resumed, result = self.launch('unmarked-reload', 'menu', ('-load', str(out / 'final.save')))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('synthetic one-way save', (resumed / 'stderr.log').read_text())
        self.assertFalse((resumed / 'summary.json').exists())
        self.assertFalse((self.base / 'natural').exists())


if __name__ == '__main__':
    unittest.main()

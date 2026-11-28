"""Real native caster capture/application; synthetic mutations are test inputs."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(BIN.exists(), 'build required')
class CasterBuildTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.env = {key: os.environ[key] for key in
                    ('PATH', 'PYTHONPATH', 'LD_LIBRARY_PATH', 'BORG_CEMETERY_DIR') if key in os.environ}
        self.env.update(BORG_DEATH_RECORDS_DIR=str(self.base / 'natural'), RUN_TIMEOUT_SECONDS='20')

    def tearDown(self):
        self.tmp.cleanup()

    def launch(self, name, race=4, cls=1, record=None, capture=None, turns=1):
        out = self.base / name
        flags = 'menu,start-depth=3' if record else 'menu'
        args = []
        if record:
            args += ['-start-build', str(record)]
        if capture:
            args += ['-capture-build', str(capture), '-capture-build-level', '1']
        result = subprocess.run(['bash', str(ROOT / 'harness/run.sh'), str(out), '202',
            str(race), str(cls), str(turns), flags, '--', *args], env=self.env,
            capture_output=True, text=True, timeout=35)
        return out, result

    def capture(self, name, race=4, cls=1):
        record = self.base / (name + '.build')
        out, result = self.launch(name, race, cls, capture=record)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(record.is_file(), (out / 'stderr.log').read_text())
        return record, out

    def test_four_classes_capture_without_changing_play_and_apply_deterministically(self):
        for race, cls in ((4, 1), (8, 2), (0, 3), (2, 4)):
            with self.subTest(race=race, cls=cls):
                record, captured = self.capture(f'capture-{cls}', race, cls)
                control, result = self.launch(f'control-{cls}', race, cls)
                self.assertEqual(result.returncode, 0, result.stderr)
                text = record.read_text()
                self.assertTrue(text.startswith('schema:borg-reborn-rs/character-build@2\n'))
                observed = [int(v) for v in text.splitlines()[3].split(':')[1:]]
                outputs = []
                for repeat in range(2):
                    out, result = self.launch(f'applied-{cls}-{repeat}', race, cls, record=record, turns=100)
                    self.assertEqual(result.returncode, 0, (out / 'stderr.log').read_text())
                    summary = json.loads((out / 'summary.json').read_text())
                    self.assertEqual(summary['initial_hp'], observed[1])
                    self.assertEqual(summary['initial_sp'], observed[2])
                    self.assertEqual(summary['start_build_schema'], 'borg-reborn-rs/character-build@2')
                    self.assertEqual(summary['start_build_sha256'], hashlib.sha256(record.read_bytes()).hexdigest())
                    self.assertEqual(summary['initial_cave_depth'], 3)
                    outputs.append(out)
                for stream in ('keys.log', 'state.log', 'messages.log'):
                    self.assertEqual((captured / stream).read_bytes(), (control / stream).read_bytes())
                    self.assertEqual((outputs[0] / stream).read_bytes(), (outputs[1] / stream).read_bytes())
        self.assertFalse((self.base / 'natural').exists())

    def test_learned_worked_flags_and_zero_hp_mana_are_preserved(self):
        record, _ = self.capture('learned-source')
        lines = record.read_text().splitlines()
        observed = lines[3].split(':')
        observed[2:] = ['0', '0']
        lines[3] = ':'.join(observed)
        # Explicit test construction: learned/tried Magic Missile, no study left.
        lines[8] = 'spells:30:0'
        lines[9] = 'spell:3:0'
        record.write_text('\n'.join(lines) + '\n')
        out, result = self.launch('learned-applied', record=record)
        self.assertEqual(result.returncode, 0, (out / 'stderr.log').read_text())
        summary = json.loads((out / 'summary.json').read_text())
        self.assertEqual(summary['initial_hp'], 0)
        self.assertEqual(summary['initial_sp'], 0)
        self.assertEqual(summary['initial_learned_spells'], 1)
        self.assertEqual(summary['initial_worked_spells'], 1)
        self.assertEqual(summary['initial_new_spells'], 0)

    def test_inconsistent_spell_mana_identity_and_device_records_fail_closed(self):
        record, _ = self.capture('invalid-source')
        original = record.read_text()
        vitals = original.splitlines()[4].split(':')
        bad_mana = ':'.join(vitals[:4] + [str(int(vitals[4]) + 1)])
        variants = [original.replace('spells:30:1', 'spells:29:1'),
                    original.replace('spells:30:1', 'spells:30:999'),
                    original.replace('spell:0:99', 'spell:8:99', 1),
                    original.replace('spell:0:99', 'spell:2:0', 1),
                    original.replace('spell:0:99', 'spell:5:0', 1),
                    original.replace('spell:0:99\nspell:0:99', 'spell:1:0\nspell:1:0', 1),
                    original.replace('spell:0:99', 'spell:1:30', 1),
                    original.replace('spell:0:99\nspell:0:99', 'spell:0:99\nspell:1:0', 1),
                    original.replace(original.splitlines()[4], bad_mana),
                    original.replace('identity:4:1:1', 'identity:0:1:1'),
                    original.replace('end\n', 'item:60:1:1:-1:0:0:0:32000:0:1\nend\n')]
        for index, text in enumerate(variants):
            with self.subTest(index=index):
                record.write_text(text)
                out, result = self.launch(f'invalid-{index}', record=record)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((out / 'summary.json').exists())

    def test_declared_equipped_armor_changes_native_ac_without_invented_mana(self):
        record, _ = self.capture('armor-source')
        baseline, result = self.launch('armor-baseline', record=record)
        self.assertEqual(result.returncode, 0, result.stderr)
        before = json.loads((baseline / 'summary.json').read_text())
        # Explicit mechanics fixture, not a claimed observed kit: same mundane
        # body kind/slot as the existing run-derived Warrior fixture.
        text = record.read_text().replace('end\n', 'item:16:2:1:6:0:0:3:0:0:1\nend\n')
        record.write_text(text)
        armored, result = self.launch('armor-applied', record=record)
        self.assertEqual(result.returncode, 0, (armored / 'stderr.log').read_text())
        after = json.loads((armored / 'summary.json').read_text())
        self.assertGreater(after['initial_ac'], before['initial_ac'])
        self.assertEqual(after['initial_to_a'], before['initial_to_a'] + 3)
        self.assertEqual(after['initial_max_sp'], before['initial_max_sp'])
        record.write_text(text.replace('end\n', 'item:16:2:1:6:0:0:3:0:0:1\nend\n'))
        out, result = self.launch('armor-duplicate-slot', record=record)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((out / 'summary.json').exists())


if __name__ == '__main__':
    unittest.main()

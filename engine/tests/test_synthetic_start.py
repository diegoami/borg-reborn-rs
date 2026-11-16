"""Real synthetic-start contract checks; run under the engine validation lock."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(BIN.exists(), 'no build (harness/build.sh)')
class SyntheticStartTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.env = dict(os.environ, BORG_IMPL='rust', RUN_TIMEOUT_SECONDS='20',
                        BORG_DEATH_RECORDS_DIR=str(self.base / 'natural-deaths'))
        for key in list(self.env):
            if key.startswith('BORG_') and key not in ('BORG_IMPL', 'BORG_DEATH_RECORDS_DIR', 'BORG_CEMETERY_DIR'):
                self.env.pop(key)
        for key in ('ANGBAND_BIN', 'BORGH_DATA_ROOT', 'CHECKPOINT_EVERY', 'SAVE_ROOT'):
            self.env.pop(key, None)

    def tearDown(self):
        self.tmp.cleanup()

    def launch(self, name, flags='menu,start-depth=10', extra=(), turns=5000):
        out = self.base / name
        argv = [str(ROOT / 'harness/run.sh'), str(out), '101', '0', '0', str(turns), flags]
        if extra:
            argv += ['--', *extra]
        result = subprocess.run(argv, env=self.env, text=True, capture_output=True, timeout=30)
        return out, result

    def frontend(self, extra):
        directories = []
        for kind in ('user', 'save', 'archive', 'panic', 'scores'):
            path = self.base / kind
            path.mkdir(exist_ok=True)
            directories.append(f'-d{kind}={path}')
        return subprocess.run([str(BIN), '-mborgh', '-n', *directories, '--', '-seed', '101', *extra],
                              cwd=BIN.parent, env=self.env, capture_output=True, text=True, timeout=20)

    def test_depths_repeat_determinism_and_no_natural_death_export(self):
        # Deliberately make the exporter unavailable: synthetic runs must not
        # invoke even an old toolkit. Direct exporter rejection is tested in cemetery.
        self.env['BORG_CEMETERY_DIR'] = str(self.base / 'no-toolkit')
        saw_death = False
        for depth in (5, 10, 20):
            runs = []
            for repeat in range(2):
                out, result = self.launch(f'd{depth}-{repeat}', f'menu,start-depth={depth}')
                self.assertEqual(result.returncode, 0, result.stderr[-2000:])
                summary = json.loads((out / 'summary.json').read_text())
                self.assertIs(summary['synthetic'], True)
                self.assertIs(summary['loaded'], False)
                self.assertEqual(summary['synthetic_contract'], 'borg-reborn-rs/synthetic-start@2')
                self.assertEqual(summary['start_depth'], depth)
                self.assertEqual(summary['initial_depth'], depth)
                self.assertEqual(summary['initial_cave_depth'], depth)
                self.assertIs(summary['initial_generation_pending'], False)
                self.assertEqual(summary['start_build'], 'birth')
                self.assertEqual(summary['dungeon_mode'], 'one_way')
                for option in ('birth_force_descend', 'birth_no_recall'):
                    self.assertIs(summary['game_options'][option], True)
                states = (out / 'state.log').read_text().splitlines()
                self.assertTrue(states)
                self.assertIn(f' d{depth}/{depth} ', states[0])
                self.assertTrue(all(int(re.search(r' d(\d+)/', row)[1]) >= depth for row in states))
                config = (out / 'config.txt').read_text()
                for line in ('synthetic=1', f'start_depth={depth}', 'start_build=birth', 'dungeon_mode=one_way'):
                    self.assertIn(line + '\n', config)
                self.assertFalse((out / 'save/input.save').exists())
                saw_death |= summary['dead'] is True and summary['stop_reason'] == 'death'
                runs.append(out)
            for log in ('keys.log', 'state.log', 'messages.log'):
                self.assertEqual((runs[0] / log).read_bytes(), (runs[1] / log).read_bytes(), log)
        self.assertTrue(saw_death, 'bounded suite must exercise a real synthetic death')
        self.assertFalse((self.base / 'natural-deaths').exists())

    def test_extra_option_has_the_same_recorded_identity(self):
        out, result = self.launch('extra', 'menu', ('-start-depth', '10'), turns=200)
        self.assertEqual(result.returncode, 0, result.stderr[-2000:])
        self.assertIn('flags=menu,start-depth=10\n', (out / 'config.txt').read_text())
        self.assertIn('synthetic=1\n', (out / 'config.txt').read_text())

    def test_frontend_rejects_invalid_depths_without_success_summary(self):
        for index, value in enumerate(('0', '-1', '10junk', '1.5', '', '999999999999999999999999', '999')):
            summary = self.base / f'bad-{index}.json'
            result = self.frontend(['-summary', str(summary), '-start-depth', value])
            self.assertNotEqual(result.returncode, 0, value)
            self.assertIn('start-depth', result.stderr)
            self.assertFalse(summary.exists(), value)

    def test_frontend_rejects_load_random_birth_and_duplicate_depth(self):
        for extra in (('-load', 'missing.save'), ('-random-birth',), ('-start-depth', '20')):
            result = self.frontend(['-start-depth', '10', *extra])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('start-depth', result.stderr)

    def test_harness_rejects_bad_and_conflicting_flags(self):
        save = self.base / 'input.save'
        save.write_bytes(b'not a save')
        self.env['SAVE_ROOT'] = str(self.base)
        cases = [('menu,start-depth=0', ()), ('start-depth=-1', ()),
                 ('start-depth=10x', ()), ('start-depth=10,start-depth=20', ()),
                 ('start-depth=10', ('-start-depth', '20')),
                 ('start-depth=10', ('-load', str(save))),
                 ('start-depth=10,save=input.save', ()),
                 ('save=input.save,start-depth=10', ()),
                 ('start-depth=10,random-birth', ()), ('menu', ('-start-depth',))]
        for index, (flags, extra) in enumerate(cases):
            out, result = self.launch(f'conflict-{index}', flags, extra)
            self.assertNotEqual(result.returncode, 0, (flags, extra))
            self.assertIn('start-depth', result.stderr)
            self.assertFalse(out.exists())

    def test_harness_rejects_a_successful_child_with_a_bad_start_receipt(self):
        fake = self.base / 'fake-engine'
        fake.write_text('#!/usr/bin/env python3\nimport json, sys\n'
                        'from pathlib import Path\n'
                        'p = Path(sys.argv[sys.argv.index("-summary") + 1])\n'
                        'p.write_text(json.dumps(json.loads(Path(__file__).with_suffix(".json").read_text())))\n')
        fake.chmod(0o755)
        self.env['ANGBAND_BIN'] = str(fake)
        receipt = dict(synthetic=True, loaded=False,
                       synthetic_contract='borg-reborn-rs/synthetic-start@2', start_depth=10,
                       initial_depth=10, initial_cave_depth=10, initial_generation_pending=False, start_build='birth', dungeon_mode='one_way',
                       game_options=dict(birth_force_descend=True, birth_no_recall=True))
        for index, changes in enumerate(({'synthetic': False}, {'initial_depth': 0},
                                         {'start_depth': 20}, {'loaded': True},
                                         {'synthetic_contract': 'unsupported'},
                                         {'game_options': {}})):
            fake.with_suffix('.json').write_text(json.dumps(dict(receipt, **changes)))
            out, result = self.launch(f'bad-receipt-{index}')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('synthetic start identity mismatch', result.stderr)
            self.assertTrue((out / 'summary.json').exists(), 'retain rejected evidence')
        fake.with_suffix('.json').write_text(json.dumps(receipt))
        _, result = self.launch('unexpected-synthetic', 'menu')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('unexpected synthetic start', result.stderr)
        self.assertFalse((self.base / 'natural-deaths').exists())


if __name__ == '__main__':
    unittest.main()

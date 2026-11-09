"""The zoo contract (docs/ZOO-INTERFACE.md; task TE.7, #77).

The build's outputs the zoo copies and reads, and one run of the binary with
exactly the command line the zoo's adapter builds (zoo/borgrs.py, command()).
The run part needs a build (harness/build.sh); CI's engine job has one.

Run: python3 -m unittest discover -s engine/tests
"""

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game'
FIELDS = ('dead', 'stop_reason', 'race', 'class', 'death_cause', 'game_turns', 'depth', 'max_depth', 'clevel', 'keys')
STOP_REASONS = {'turns', 'death', 'clevel', 'depth', 'keys', 'stalled', 'borg_stopped', 'blocked'}


class BuildContractTest(unittest.TestCase):
    def test_the_files_the_zoo_copies_exist(self):
        self.assertTrue((ROOT / 'harness/build.sh').is_file())
        self.assertTrue((ROOT / 'harness/borg.txt').is_file())

    def test_another_implementation_fails_loudly(self):
        r = subprocess.run([str(ROOT / 'harness/build.sh')], env=dict(os.environ, BORG_IMPL='c-pristine'),
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)

    @unittest.skipUnless((GAME / 'angband').exists(), 'no build (harness/build.sh)')
    def test_build_id_matches_the_binary(self):
        ident = dict(l.split('=', 1) for l in (GAME.parent / 'BUILD_ID').read_text().splitlines() if '=' in l)
        self.assertEqual(ident['binary_sha256'], hashlib.sha256((GAME / 'angband').read_bytes()).hexdigest())
        self.assertTrue(ident.get('rustc'))
        self.assertTrue((GAME / 'lib/gamedata').is_dir(), 'the data travels with the game directory')


@unittest.skipUnless((GAME / 'angband').exists(), 'no build (harness/build.sh)')
class RunContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        out = cls.out = Path(cls.tmp.name)
        for d in ('user', 'save', 'archive', 'panic', 'scores'):
            (out / d).mkdir()
        (out / 'user/borg.txt').write_text((ROOT / 'harness/borg.txt').read_text())
        # zoo/borgrs.py command(), with exact_turns and depth_stream on.
        argv = ['./angband', '-mborgh', '-n', '-uBorg'] + [f'-d{d}={out / d}' for d in ('user', 'save', 'archive', 'panic', 'scores')] + \
            ['--', '-seed', '29', '-race', '0', '-class', '0', '-turns', '300', '-keylog', str(out / 'keys.log'),
             '-summary', str(out / 'summary.json'), '-menu-letters', '-exact-turns', '-statelog', str(out / 'state.log')]
        env = dict(os.environ, TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
        cls.result = subprocess.run(argv, cwd=GAME, env=env, capture_output=True, text=True, timeout=600)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_run_writes_what_the_zoo_reads(self):
        self.assertEqual(self.result.returncode, 0, self.result.stderr[-2000:])
        s = json.loads((self.out / 'summary.json').read_text())
        for field in FIELDS:
            self.assertIn(field, s)
        self.assertIn(s['stop_reason'], STOP_REASONS)
        self.assertEqual((s['race'], s['class']), ('Human', 'Warrior'))
        if not s['dead']:
            self.assertEqual((s['stop_reason'], s['game_turns']), ('turns', 300), 'the exact boundary')

    def test_the_logs_have_the_zoo_formats(self):
        keys = (self.out / 'keys.log').read_text().splitlines()
        self.assertTrue(keys)
        self.assertRegex(keys[0], r'^\d+ \d+ [0-9a-f]+ [0-9a-f]+$')
        states = (self.out / 'state.log').read_text().splitlines()
        self.assertEqual(len(states), len(keys), 'one state line per key')
        self.assertRegex(states[0], r'^\d+ \d+,\d+ d\d+/\d+')


if __name__ == '__main__':
    unittest.main()

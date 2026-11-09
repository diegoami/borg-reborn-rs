"""Our build (docs/SPEC.md §3.6; task TE.4, #74).

The source list is upstream's minus the C Borg, and, when a build exists
(build/rust/game/angband, made by harness/build.sh), the binary keeps the
frontend's contract (§3.5): it answers the contract's flags and writes the
contract's summary.json fields.

Run: python3 -m unittest discover -s engine/tests
"""

import json
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'build/rust/game/angband'


def listed(cmake_text, start):
    m = re.search(re.escape(start) + r'\n(.*?)\n\)', cmake_text, re.S)
    return [l.strip() for l in m.group(1).splitlines() if l.strip() and not l.strip().startswith('#')]


class SourceListTest(unittest.TestCase):
    def test_ours_is_upstreams_without_the_c_borg(self):
        upstream = listed((ROOT / 'engine/angband/CMakeLists.txt').read_text(), 'add_library(OurCoreLib OBJECT')
        ours = listed((ROOT / 'engine/sources.cmake').read_text(), 'set(ANGBAND_CORE_SOURCES')
        self.assertEqual(ours, [s for s in upstream if not s.startswith('src/borg/')])
        self.assertTrue(any(s.startswith('src/borg/') for s in upstream), 'upstream lists the C Borg')

    def test_the_build_inputs_are_ours(self):
        r = subprocess.run([str(ROOT / 'harness/identity.sh'), 'inputs', 'rust'], capture_output=True, text=True, check=True)
        inputs = r.stdout.split()
        self.assertTrue(any(p.startswith('engine/angband/') for p in inputs))
        self.assertTrue(any(p.startswith('engine/patches/') for p in inputs))
        for p in inputs:
            self.assertFalse(p.startswith(('angband/', 'borg-rs/', 'harness/measurement/')), p)


@unittest.skipUnless(BIN.exists(), 'no build (harness/build.sh)')
class BinaryContractTest(unittest.TestCase):
    """One short run through harness/run.sh with every output flag."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.out = Path(cls.tmp.name) / 'run'
        env = dict(os.environ, BORG_IMPL='rust', BORG_DEATH_RECORDS_DIR=str(Path(cls.tmp.name) / 'deaths'))
        cls.result = subprocess.run([str(ROOT / 'harness/run.sh'), str(cls.out), '101', '0', '0', '200', 'menu,exact'],
                                 env=env, capture_output=True, text=True)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_run_succeeds(self):
        self.assertEqual(self.result.returncode, 0, self.result.stderr[-2000:])

    def test_the_outputs_exist(self):
        for name in ('keys.log', 'messages.log', 'state.log', 'summary.json', 'process.json'):
            self.assertTrue((self.out / name).is_file(), name)

    def test_the_summary_has_the_contract_fields(self):
        s = json.loads((self.out / 'summary.json').read_text())
        for field in ('seed', 'race', 'race_idx', 'class', 'class_idx', 'stop_reason', 'game_turns', 'player_turns',
                      'depth', 'max_depth', 'clevel', 'max_clevel', 'dead', 'death_cause', 'keys', 'exact_turns'):
            self.assertIn(field, s)
        self.assertEqual(s['seed'], 101)
        self.assertEqual(s['stop_reason'], 'turns')
        self.assertEqual(s['game_turns'], 200, 'the exact boundary')
        # The options the Rust Borg sets (docs/ENGINE.md).
        self.assertIs(s['game_options']['auto_more'], True)
        self.assertNotIn('easy_alter', s['game_options'], 'not an option of 4.2.6')
        self.assertIs(s['game_options']['cheat_live'], False)

    def test_an_unknown_flag_is_refused(self):
        r = subprocess.run([str(BIN), '-mborgh', '-n', '--', '-seed', '1', '-frames', 'x'],
                           cwd=BIN.parent, capture_output=True, text=True, timeout=60)
        self.assertNotEqual(r.returncode, 0)


if __name__ == '__main__':
    unittest.main()

"""The headless frontend's contract (docs/SPEC.md §3.5; task TE.2, #72).

The zoo calls the frontend's mode, flags and output fields, so they are a
fixed contract. Until our build exists (TE.4) this checks the source: every
contract flag is parsed, every contract field of summary.json is written,
and the dropped options are gone. TE.4 adds the same checks on a binary.

Run: python3 -m unittest discover -s engine/tests
"""

import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRONTEND = ROOT / 'engine/frontend/main-borgh.c'

# docs/SPEC.md §3.5.
FLAGS = {'-seed', '-race', '-class', '-menu-letters', '-turns', '-exact-turns', '-depth', '-keys', '-load',
         '-keep-load-rng', '-start-depth', '-start-build', '-keylog', '-msglog', '-statelog', '-summary', '-screen'}
FIELDS = {'seed', 'race', 'race_idx', 'class', 'class_idx', 'stop_reason', 'game_turns', 'player_turns', 'depth',
          'max_depth', 'clevel', 'max_clevel', 'dead', 'death_cause', 'keys', 'exact_turns', 'loaded',
          'effective_settings', 'synthetic'}
# Not carried from borg-rs's borgh (docs/PROVENANCE.md).
DROPPED = {'-states', '-frames', '-make-fixture', '-near-morgoth', '-stop-rune-pattern', '-cheat-live'}


class FrontendContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = FRONTEND.read_text()
        cls.flags = set(re.findall(r'streq\(argv\[i\], "(-[a-z-]+)"\)', cls.text))

    def test_every_contract_flag_is_parsed(self):
        self.assertEqual(FLAGS - self.flags, set())

    def test_the_dropped_options_are_gone(self):
        self.assertEqual(DROPPED & self.flags, set())
        for word in ('native-replay', 'frames_fp', 'states_fp'):
            self.assertNotIn(word, self.text)

    def test_optional_recorder_has_only_guarded_lifecycle_hooks(self):
        # Owner #210 permits the external recorder; the old frame/observer
        # implementation remains absent from this frontend. Its two lifecycle
        # calls disappear from the default build.
        blocks = re.findall(r'#ifdef BORG_RECORDING\n(.*?)#endif', self.text, re.S)
        self.assertEqual(len(blocks), 2)
        self.assertTrue(any('ar_finish()' in block for block in blocks))
        self.assertTrue(any('ar_snapshot_enter()' in block for block in blocks))
        default = re.sub(r'#ifdef BORG_RECORDING\n.*?#endif', '', self.text, flags=re.S)
        self.assertNotIn('ar_finish', default)
        self.assertNotIn('ar_snapshot_enter', default)
        self.assertNotIn('BORG_RECORDING', default)

    def test_every_contract_field_is_written(self):
        written = set(re.findall(r'\\"([a-z_]+)\\":', self.text))
        self.assertEqual(FIELDS - written, set())

    def test_the_mode_is_registered_as_borgh(self):
        patch = (ROOT / 'engine/patches/01-frontend-registration.patch').read_text()
        self.assertIn('{ "borgh", help_borgh, init_borgh, false, true }', patch)
        self.assertRegex(self.text, r'errr init_borgh\(int argc, char \*argv\[\]\)')
        self.assertRegex(self.text, r'const char help_borgh\[\]')

    def test_it_compiles_against_the_patched_tree(self):
        cc = subprocess.run(['sh', '-c', 'command -v cc'], capture_output=True, text=True).stdout.strip()
        if not cc:
            self.skipTest('no C compiler')
        with tempfile.TemporaryDirectory() as t:
            tree = Path(t) / 'tree'
            subprocess.run([str(ROOT / 'engine/apply-patches.sh'), str(tree)], check=True)
            r = subprocess.run([cc, '-fsyntax-only', '-std=gnu99', '-DBORG_HARNESS', '-DUSE_BORGH', '-DALLOW_BORG',
                                f'-I{tree}/src', f'-I{FRONTEND.parent}', str(FRONTEND)],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)


if __name__ == '__main__':
    unittest.main()

"""Exercise the shared Borg prompt and conflicting settings in one executable."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(GAME.is_file(), 'native build required')
class BorgSelectionTest(unittest.TestCase):
    def run_game(self, out, *flags, core='borg-rs', c_stop=128, p_stop=128, debug=False):
        directories = ('user', 'save', 'archive', 'panic', 'scores')
        for name in directories:
            (out / name).mkdir()
        (out / 'user/borg.txt').write_text(f'borg_stop_dlevel = {c_stop}\n')
        (out / 'user/pborg.yaml').write_text(
            f'borg_reborn_core: {core}\nborg_stop_dlevel: {p_stop}\n')
        argv = [str(GAME), '-mborgh', '-n']
        argv += [f'-d{name}={out / name}' for name in directories]
        argv += ['-uBorg', '--', '-seed', '29', '-race', '0', '-class', '0',
                 '-turns', '300', '-menu-letters', '-msglog', str(out / 'messages.log'),
                 '-summary', str(out / 'summary.json'), *flags]
        env = {key: value for key, value in os.environ.items()
               if not key.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
        if debug:
            env['BORGH_DEBUG'] = '1'
        run = subprocess.run(argv, cwd=GAME.parent, env=env,
                             capture_output=True, text=True, timeout=20)
        self.assertEqual(run.returncode, 0, run.stderr)
        return run.stderr, (out / 'messages.log').read_text(), json.loads((out / 'summary.json').read_text())

    def test_default_and_switch_use_the_selected_borgs_config(self):
        for cborg in (False, True):
            for should_stop in (False, True):
                with self.subTest(cborg=cborg, stop=should_stop), tempfile.TemporaryDirectory() as tmp:
                    c_stop = 0 if cborg == should_stop else 128
                    p_stop = 0 if cborg != should_stop else 128
                    flags = ('-cborg',) if cborg else ()
                    _, messages, summary = self.run_game(Path(tmp), *flags, c_stop=c_stop, p_stop=p_stop)
                    self.assertEqual('CBorg selected.' in messages, cborg)
                    self.assertEqual('Initializing PBorg... (pborg.yaml)' in messages, not cborg)
                    self.assertEqual('Initializing the Borg... (borg.txt)' in messages, cborg)
                    if should_stop:
                        self.assertIn('Auto-stop for user DLevel.', messages)
                    else:
                        self.assertNotIn('Auto-stop for user DLevel.', messages)
                        self.assertEqual(summary['stop_reason'], 'turns')

    def test_help_lists_and_switches_both_borgs(self):
        for core in ('borg-rs', 'reborn'):
            with self.subTest(core=core), tempfile.TemporaryDirectory() as tmp:
                screens, messages, summary = self.run_game(Path(tmp), '-borg-help', core=core, debug=True)
                for borg, other in (('PBorg', 'CBorg'), ('CBorg', 'PBorg')):
                    self.assertIn(f'{borg} command (? for help):', screens)
                    self.assertNotIn(f'B for {other}', screens)
                    self.assertIn(f"Command 'B' switch to {other}", screens)
                    self.assertIn(f'{borg} selected. Press Ctrl-Z for commands.', screens)
                    self.assertEqual(messages.count(f'{borg} selected.'), 1)
                self.assertLess(messages.index('CBorg selected.'), messages.index('PBorg selected.'))
                self.assertNotIn('That is not a legal Borg command.', messages)
                self.assertEqual(summary['stop_reason'], 'turns')
                receipt = 'params-core.json' if core == 'borg-rs' else 'params.json'
                self.assertTrue((Path(tmp) / receipt).is_file(), f'{core} started after switching back')


if __name__ == '__main__':
    unittest.main()

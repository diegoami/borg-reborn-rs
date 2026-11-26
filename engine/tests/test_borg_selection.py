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
    def test_default_and_switch_use_the_selected_borgs_config(self):
        for cborg in (False, True):
            for should_stop in (False, True):
                with self.subTest(cborg=cborg, stop=should_stop), tempfile.TemporaryDirectory() as tmp:
                    out = Path(tmp)
                    directories = ('user', 'save', 'archive', 'panic', 'scores')
                    for name in directories:
                        (out / name).mkdir()
                    c_stop = 0 if cborg == should_stop else 128
                    p_stop = 0 if cborg != should_stop else 128
                    (out / 'user/borg.txt').write_text(f'borg_stop_dlevel = {c_stop}\n')
                    (out / 'user/pborg.yaml').write_text(
                        f'borg_reborn_core: borg-rs\nborg_stop_dlevel: {p_stop}\n')
                    argv = [str(GAME), '-mborgh', '-n']
                    argv += [f'-d{name}={out / name}' for name in directories]
                    argv += ['-uBorg', '--', '-seed', '29', '-race', '0', '-class', '0',
                             '-turns', '300', '-menu-letters', '-msglog', str(out / 'messages.log'),
                             '-summary', str(out / 'summary.json')]
                    if cborg:
                        argv.append('-cborg')
                    env = {key: value for key, value in os.environ.items()
                           if not key.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
                    run = subprocess.run(argv, cwd=GAME.parent, env=env,
                                         capture_output=True, text=True, timeout=20)
                    self.assertEqual(run.returncode, 0, run.stderr)
                    messages = (out / 'messages.log').read_text()
                    summary = json.loads((out / 'summary.json').read_text())
                    self.assertEqual('CBorg selected.' in messages, cborg)
                    self.assertEqual('Initializing PBorg... (pborg.yaml)' in messages, not cborg)
                    self.assertEqual('Initializing the Borg... (borg.txt)' in messages, cborg)
                    if should_stop:
                        self.assertIn('Auto-stop for user DLevel.', messages)
                    else:
                        self.assertNotIn('Auto-stop for user DLevel.', messages)
                        self.assertEqual(summary['stop_reason'], 'turns')


if __name__ == '__main__':
    unittest.main()

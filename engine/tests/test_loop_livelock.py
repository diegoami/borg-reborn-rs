"""#421: a level loop must not bounce between town and DL1 at a frozen game
turn when the core refuses the next level for a reason town cannot change.

borg-zoo's small tier (borg-zoo#123) found Half-Troll Rogue 926210318 with
the ambitious plan and the live town planner crossing the DL0/DL1 stairs
1,098 times at one game turn on main 4ecb93d7 ("loop not prepared for DL2;
town overrides the loop" / "restart after town"). The borg.txt is the zoo
adapter's (zoo/borgrs.py config_text)."""
import collections
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'
SETTINGS = {
    'borg_respawn_race': '7', 'borg_respawn_class': '6',
    'borg_cheat_death': 'FALSE', 'borg_respawn_winners': 'FALSE',
    'borg_reborn_core': '1', 'borg_reborn_errand': 'none',
    'borg_town_plan': '1', 'borg_trip_policy': '0',
}


@unittest.skipUnless(GAME.is_file(), 'harness/build.sh required')
class LoopLivelockTest(unittest.TestCase):
    def test_unchangeable_refusal_holds_instead_of_bouncing(self):
        text = (ROOT / 'harness/borg.txt').read_text()
        for name, value in SETTINGS.items():
            text, n = re.subn(rf'^{name} = .*$', f'{name} = {value}', text, flags=re.M)
            self.assertEqual(n, 1, name)
        # The loop-levels form of errands_plan_ambitious.txt (sha256
        # 9d7d8a39…, shipped until the plans became max-depth caps).
        text += '\nborg_plan = errands_plan_ambitious.txt\n'
        with tempfile.TemporaryDirectory(prefix='loop-livelock-') as tmp:
            out = Path(tmp)
            for d in ['user', 'save', 'archive', 'panic', 'scores']:
                (out / d).mkdir()
            (out / 'user/borg.txt').write_text(text)
            (out / 'user/errands_plan_ambitious.txt').write_bytes(
                (ROOT / 'engine/tests/fixtures/loop-levels-ambitious-plan.txt').read_bytes())
            env = {k: v for k, v in os.environ.items()
                   if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
            env.update(TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
            args = [str(GAME), '-mborgh', '-n', '-uBorg']
            args += [f'-d{d}={out / d}' for d in ['user', 'save', 'archive', 'panic', 'scores']]
            args += ['--', '-seed', '926210318', '-race', '7', '-class', '6', '-turns', '100000',
                     '-keylog', str(out / 'keys.log'), '-summary', str(out / 'summary.json'),
                     '-menu-letters', '-exact-turns', '-msglog', str(out / 'messages.log')]
            result = subprocess.run(args, cwd=GAME.parent, env=env,
                                    capture_output=True, text=True, timeout=600)
            self.assertEqual(result.returncode, 0, result.stderr)
            per_turn = collections.Counter(
                line.split()[0] for line in (out / 'keys.log').read_text().splitlines())
            turn, keys = per_turn.most_common(1)[0]
            self.assertLess(keys, 1000, f'{keys} keys at game turn {turn}')
            messages = (out / 'messages.log').read_text()
            self.assertIn('still not prepared for DL2 after town; the core plays DL1', messages)

"""Replay the town inventory-menu loop from #775 on the shipped Borg core."""

from collections import Counter
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(GAME.is_file(), 'native build required')
class TownUninscribeTest(unittest.TestCase):
    def test_fox_form_reversion_does_not_open_inventory(self):
        with tempfile.TemporaryDirectory(prefix='town-uninscribe-') as tmp:
            out = Path(tmp) / 'run'
            env = {key: value for key, value in os.environ.items()
                   if key == 'BORG_CEMETERY_DIR'
                   or not key.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
            env.update(
                BORG_CORE='borg-rs',
                BORG_CORE_TRACE='off',
                BORG_TXT_SET='borg_reborn_errand=none;borg_respawn_winners=FALSE;'
                             'borg_param_caution_stairs_flight_ignore_danger=1',
                BORG_DEATH_RECORDS_DIR=str(Path(tmp) / 'deaths'),
            )
            subprocess.run(
                [str(ROOT / 'harness/run.sh'), str(out), '934061548', '1', '2',
                 '100000', 'menu,exact'],
                cwd=ROOT, env=env, check=True, timeout=120,
            )
            summary = json.loads((out / 'summary.json').read_text())
            self.assertEqual(summary['stop_reason'], 'turns', summary)
            self.assertEqual(summary['game_turns'], 100000)

            with (out / 'keys.log').open() as keys:
                counts = Counter(int(line.split(maxsplit=1)[0]) for line in keys)
            turn, count = counts.most_common(1)[0]
            # Zoo's frozen-turn gate (docs/ZOO-INTERFACE.md).
            self.assertLess(count, 1000, f'{count} keys sent on game turn {turn}')

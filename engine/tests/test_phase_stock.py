"""Native M13 shopping from a Mage who already knows Phase Door."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'
FIXTURE = json.loads((ROOT / 'harness/fixtures/planmage-town-phase.json').read_text())
ARCHIVE = Path(os.environ.get('BORG_PHASE_STOCK_FIXTURE_ZIP',
    str(ROOT / 'vendor/borg-cemetery' / FIXTURE['archive_path'])))


@unittest.skipUnless(GAME.is_file(), 'native build required')
class PhaseStockTest(unittest.TestCase):
    def test_learned_spells_do_not_remove_item_value(self):
        if not ARCHIVE.is_file():
            if os.environ.get('BORG_PHASE_STOCK_FIXTURE_REQUIRED') == '1':
                self.fail(f'missing mandatory shopping fixture: {ARCHIVE}')
            self.skipTest('set BORG_PHASE_STOCK_FIXTURE_ZIP to the pinned cemetery fixture')
        self.assertEqual(hashlib.sha256(ARCHIVE.read_bytes()).hexdigest(), FIXTURE['archive_sha256'])
        with tempfile.TemporaryDirectory(prefix='phase-stock-') as tmp, zipfile.ZipFile(ARCHIVE) as source:
            out = Path(tmp)
            for name in ['user', 'save', 'archive', 'panic', 'scores', 'diagnostics']:
                (out / name).mkdir()
            save = source.read('town.save')
            self.assertEqual(hashlib.sha256(save).hexdigest(), FIXTURE['save_sha256'])
            (out / 'input.save').write_bytes(save)
            config = subprocess.check_output(['python3', str(ROOT / 'harness/pborg-config.py'), '-'],
                                             input=source.read('borg.txt'))
            (out / 'user/pborg.yaml').write_bytes(config)
            env = {k: v for k, v in os.environ.items()
                   if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
            env.update(BORG_CORE='borg-rs', BORG_ERRAND='/shopping',
                       BORG_RUN_DIR=str(out / 'diagnostics'),
                       BORG_TOWN_PLAN_RECORD=str(out / 'town-plan.jsonl'),
                       TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
            args = [str(GAME), '-mborgh']
            args += [f'-d{k}={out / k}' for k in ['user', 'save', 'archive', 'panic', 'scores']]
            args += ['-uShopping', '--', '-seed', '0', '-race', '4', '-class', '1',
                     '-load', str(out / 'input.save'), '-keep-load-rng',
                     '-turns', str(FIXTURE['turn'] + 3000), '-keys', '10000', '-menu-letters',
                     '-summary', str(out / 'summary.json'), '-msglog', str(out / 'messages.log')]
            result = subprocess.run(args, cwd=GAME.parent, env=env,
                                    capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads((out / 'summary.json').read_text())
            self.assertEqual(summary['loaded_turn'], FIXTURE['turn'])
            self.assertFalse(summary['dead'], summary)
            self.assertEqual(summary['stop_reason'], 'borg_stopped')
            self.assertEqual(summary['depth'], 0)
            self.assertIn('You bought a Scroll of Phase Door for 15 gold.',
                          (out / 'messages.log').read_text())
            plans = [json.loads(line) for line in (out / 'town-plan.jsonl').read_text().splitlines()]
            # #400: learned spells (this Mage has Find Traps, Detect Monsters
            # and Light Room) no longer zero any item's demand; before, TRAPS,
            # MONSTERS and ILLUMINATE were 0 here.
            curves = {c[0]: c for c in plans[-1]['capability_curves']}
            for cap in [1, 4, 5, 10, 11, 12, 13, 14, 15]:
                self.assertGreater(curves[cap][1], 0, cap)
            phase = next(c for c in plans[-1]['capability_curves'] if c[0] == 4)
            self.assertGreater(phase[1], 0)
            self.assertEqual(phase[3], 1000)  # One physical scroll remains in the final kit.

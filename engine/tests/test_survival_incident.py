"""Native counterfactuals from source-qualified player-cemetery saves.

A save restores the engine, not Borg memory. Original-source fresh-Borg
branches from all four fixtures die; the full original replay also verifies.
"""
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
FIXTURE = json.loads((ROOT / 'harness/fixtures/planmage-wild-cat.json').read_text())
ARCHIVE = Path(os.environ.get('BORG_SURVIVAL_FIXTURE_ZIP',
    str(ROOT / 'vendor/player-cemetery/recordings' / FIXTURE['archive'])))


@unittest.skipUnless(GAME.is_file(), 'native build required')
class SurvivalIncidentTest(unittest.TestCase):
    def test_preserved_wild_cat_positions_survive_without_loot_detours(self):
        if not ARCHIVE.is_file():
            if os.environ.get('BORG_SURVIVAL_FIXTURES_REQUIRED') == '1':
                self.fail(f'missing mandatory incident archive: {ARCHIVE}')
            self.skipTest('set BORG_SURVIVAL_FIXTURE_ZIP to the pinned cemetery archive')
        self.assertEqual(hashlib.sha256(ARCHIVE.read_bytes()).hexdigest(), FIXTURE['archive_sha256'])
        with tempfile.TemporaryDirectory(prefix='wild-cat-regression-') as tmp, zipfile.ZipFile(ARCHIVE) as source:
            for name, point in FIXTURE['saves'].items():
                with self.subTest(turn=point['turn']):
                    out = Path(tmp) / str(point['turn'])
                    for part in ['user', 'save', 'archive', 'panic', 'scores', 'diagnostics']:
                        (out / part).mkdir(parents=True)
                    data = source.read(name)
                    self.assertEqual(hashlib.sha256(data).hexdigest(), point['sha256'])
                    (out / 'starting.save').write_bytes(data)
                    for config in ['borg.txt', 'errands_plan.txt']:
                        (out / 'user' / config).write_bytes(source.read(config))
                    env = {k:v for k,v in os.environ.items()
                           if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
                    env.update(BORG_CORE='borg-rs', BORG_RUN_DIR=str(out / 'diagnostics'),
                               BORG_ERRANDS_PLAN=str(out / 'user/errands_plan.txt'),
                               TZ='UTC', LANG='C.UTF-8', LC_ALL='C.UTF-8')
                    argv = [str(GAME), '-mborgh']
                    argv += [f'-d{k}={out / k}' for k in ['user','save','archive','panic','scores']]
                    argv += ['-uIncident', '--', '-seed','0', '-race','4', '-class','1',
                             '-load',str(out/'starting.save'), '-keep-load-rng',
                             '-turns',str(point['turn']+2000), '-keys','3000', '-menu-letters',
                             '-summary',str(out/'summary.json'), '-msglog',str(out/'messages.log')]
                    result = subprocess.run(argv, cwd=GAME.parent, env=env,
                                            capture_output=True, text=True, timeout=30)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    summary = json.loads((out/'summary.json').read_text())
                    self.assertFalse(summary['dead'], summary)
                    self.assertEqual(summary['stop_reason'], 'turns')
                    self.assertEqual(summary['hp'], summary['max_hp'])
                    # Loading emits the save's old message history at turn0;
                    # assertions concern decisions made after this checkpoint.
                    messages = '\n'.join(line for line in (out/'messages.log').read_text().splitlines()
                        if len(line.split(maxsplit=3)) == 4
                        and int(line.split(maxsplit=3)[1]) >= point['turn'])
                    self.assertNotIn('Flowing toward Scum item', messages)
                    if point['turn'] in [411648,411658]:
                        self.assertIn('Casting Phase Door', messages)
                        self.assertIn('Escaping a trapped or lethal retreat', messages)

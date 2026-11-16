"""Native counterfactuals from source-qualified Borg Cemetery saves.

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
    str(ROOT / 'vendor/cemetery-evidence' / FIXTURE['archive_path'])))


@unittest.skipUnless(GAME.is_file(), 'native build required')
class SurvivalIncidentTest(unittest.TestCase):
    def test_saved_positions_heal_through_the_wild_cat_fight(self):
        self.run_saved_positions()

    def test_flee_healing_still_obeys_the_clw_and_csw_limits(self):
        self.run_saved_positions(disable_tiers=True)

    def run_saved_positions(self, disable_tiers=False):
        if not ARCHIVE.is_file():
            if os.environ.get('BORG_SURVIVAL_FIXTURES_REQUIRED') == '1':
                self.fail(f'missing mandatory incident archive: {ARCHIVE}')
            self.skipTest('set BORG_SURVIVAL_FIXTURE_ZIP to the pinned cemetery archive')
        self.assertEqual(hashlib.sha256(ARCHIVE.read_bytes()).hexdigest(), FIXTURE['archive_sha256'])
        with tempfile.TemporaryDirectory(prefix='wild-cat-regression-') as tmp, zipfile.ZipFile(ARCHIVE) as source:
            for name, point in FIXTURE['saves'].items():
                if disable_tiers and point['turn'] != 411668:
                    continue
                with self.subTest(turn=point['turn']):
                    out = Path(tmp) / str(point['turn'])
                    for part in ['user', 'save', 'archive', 'panic', 'scores', 'diagnostics']:
                        (out / part).mkdir(parents=True)
                    data = source.read(name)
                    self.assertEqual(hashlib.sha256(data).hexdigest(), point['sha256'])
                    (out / 'starting.save').write_bytes(data)
                    for config in ['borg.txt', 'errands_plan.txt']:
                        (out / 'user' / config).write_bytes(source.read(config))
                    if disable_tiers:
                        with (out / 'user/borg.txt').open('a') as config:
                            config.write('\nborg_param_caution_clw_max_level = 1\n'
                                         'borg_param_caution_clw_max_loss = 0\n'
                                         'borg_param_caution_csw_max_level = 1\n'
                                         'borg_param_caution_csw_max_loss = 0\n')
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
                    self.assertEqual(summary['loaded_turn'], point['turn'])
                    rows = [json.loads(line) for line in
                            (out/'diagnostics/trace-core.jsonl').read_text().splitlines()]
                    if disable_tiers:
                        self.assertTrue(summary['dead'], summary)
                        # The ordinary wound/level gates exclude the opening
                        # CLW/CSW; later status cures retain their own policy.
                        self.assertNotEqual(rows[0]['keys'], [ord('q'), ord('c')])
                    else:
                        self.assertFalse(summary['dead'], summary)
                        self.assertEqual(summary['hp'], summary['max_hp'])
                        self.assertTrue(any(r['check'] == 'borg_caution'
                                            and r['danger'] >= r['hp']
                                            and r['keys'] == [ord('q'), ord('c')]
                                            for r in rows), rows[:10])

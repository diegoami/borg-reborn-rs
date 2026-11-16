"""Bounded loop errands on real engine-generated dungeon saves.

Run after harness/build.sh under /tmp/angband-validation.lock. Fixture
creation uses normal birth/play, not edited character or engine state.
"""
from collections import Counter
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(GAME.is_file(), 'harness/build.sh required')
class LevelLoopTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='level-loop-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_dungeon_only_loop_waits_on_a_safe_level_when_descent_is_blocked(self):
        # #421: a constructed positive interval isolates the reported
        # ambitious-plan failure without waiting for character growth.
        # Before the fix this seed produces 1,409 keys at game turn 2,092,
        # repeatedly returning to town because it cannot yet enter DL2.
        out = self.root / 'blocked-descent'
        for name in ['user', 'save', 'archive', 'panic', 'scores']:
            (out / name).mkdir(parents=True)
        config = (ROOT / 'harness/borg.txt').read_text()
        config = config.replace('borg_town_plan = 0', 'borg_town_plan = 1')
        config += '\nborg_plan = errands_plan.txt\n'
        (out / 'user/borg.txt').write_text(config)
        (out / 'user/errands_plan.txt').write_text(
            '[default]\n  level <= 49\n    loop-levels 3 9\n')
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
        env.update(BORG_CORE='borg-rs', BORG_ERRAND='none',
                   BORG_RUN_DIR=str(out), LANG='C.UTF-8', LC_ALL='C.UTF-8', TZ='UTC')
        command = [str(GAME), '-mborgh', '-n']
        command += [f'-d{name}={out / name}'
                    for name in ['user', 'save', 'archive', 'panic', 'scores']]
        command += ['-uStair421', '--', '-seed', '1019921749', '-race', '1',
                    '-class', '0', '-exact-turns', '-turns', '5000',
                    '-keylog', str(out / 'keys.log'),
                    '-msglog', str(out / 'messages.log'),
                    '-statelog', str(out / 'state.log'),
                    '-summary', str(out / 'summary.json')]
        subprocess.run(command, cwd=GAME.parent, env=env, check=True,
                       capture_output=True, text=True, timeout=40)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertFalse(summary['dead'], summary)
        self.assertEqual(summary['game_turns'], 5000)
        bursts = Counter(int(line.split()[0]) for line in
                         (out / 'state.log').read_text().splitlines())
        self.assertLess(max(bursts.values()), 1000, 'frozen-turn stair loop')
        frames = [json.loads(line) for line in
                  (out / 'trace-core.jsonl').read_text().splitlines()]
        self.assertTrue(any(r['depth'] == 1 and r['errand']['target'] == 3
                            and r['keys'] and r['keys'] != [60] for r in frames),
                        'the blocked waypoint should remain while the core plays DL1')

    def run_game(self, name, errand, seed=1010, cls=0, budget=0, extra=()):
        out = self.root / name
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
        env.update(BORG_CORE='borg-rs', BORG_ERRAND=errand,
                   BORG_ERRAND_BUDGET=str(budget),
                   BORG_ERRAND_OUT=str(out) + '.outcomes',
                   BORG_DEATH_RECORDS_DIR='/tmp/borg-level-loop-test-deaths',
                   RUN_TIMEOUT_SECONDS='30')
        subprocess.run([str(ROOT / 'harness/run.sh'), str(out), str(seed),
                        '0', str(cls), '100000', 'menu,finalsave', '--', *extra],
                       env=env, check=True, capture_output=True, text=True,
                       timeout=40)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertFalse(summary['dead'], summary)
        return out, summary

    def test_return_by_recall_then_resume_and_budget_drops_the_queue(self):
        fixture, start = self.run_game('fixture', '', extra=('-depth', '1'))
        self.assertEqual(start['depth'], 1)
        out, summary = self.run_game(
            'loop', '/explored 1; /loop-levels 1 0; /shopping', budget=1200,
            extra=('-load', str(fixture / 'final.save'), '-keep-load-rng'))
        messages = (out / 'messages.log').read_text()
        self.assertIn('loop DL1 complete, target DL0', messages)
        self.assertIn('The air about you becomes charged', messages)
        self.assertIn('You feel yourself yanked upwards', messages)
        self.assertIn('loop restart after town, target DL1', messages)
        outcomes = [json.loads(line) for line in
                    Path(str(out) + '.outcomes').read_text().splitlines()]
        self.assertEqual([r['outcome'] for r in outcomes], ['budget'])
        self.assertEqual(outcomes[0]['errand'], 'LoopLevels(1, 0)')
        self.assertEqual(summary['stop_reason'], 'borg_stopped')
        self.assertGreaterEqual(summary['game_turns'] - start['game_turns'], 1200)

    def test_stairs_fallback_without_recall_keeps_cycling(self):
        fixture, start = self.run_game('fixture', '', seed=1001, cls=1,
                                       extra=('-depth', '1'))
        self.assertEqual(start['depth'], 1)
        self.assertIn('You drop a Scroll of Word of Recall',
                      (fixture / 'messages.log').read_text())
        out, summary = self.run_game(
            'loop', '/explored 1; /loop-levels 1 0', seed=1001, cls=1,
            budget=2000,
            extra=('-load', str(fixture / 'final.save'), '-keep-load-rng'))
        messages = (out / 'messages.log').read_text()
        self.assertNotIn('The air about you becomes charged', messages)
        self.assertGreaterEqual(messages.count('loop DL1 complete, target DL0'), 2)
        self.assertIn('loop restart after town, target DL1', messages)
        frames = [json.loads(line) for line in
                  (out / 'trace-core.jsonl').read_text().splitlines()]
        self.assertTrue(any(r['depth'] == 0 for r in frames))
        self.assertTrue(any(60 in r['keys'] for r in frames), 'no upstairs command')
        self.assertEqual(summary['stop_reason'], 'borg_stopped')

    def test_town_override_restarts_a_dungeon_interval_instead_of_aborting(self):
        out, summary = self.run_game(
            'loop', '/loop-levels 1 2', budget=1000)
        messages = (out / 'messages.log').read_text()
        self.assertIn('loop town overrides the loop, target DL0', messages)
        self.assertIn('loop restart after town, target DL1', messages)
        outcomes = [json.loads(line) for line in
                    Path(str(out) + '.outcomes').read_text().splitlines()]
        self.assertEqual([r['outcome'] for r in outcomes], ['budget'])
        self.assertEqual(summary['stop_reason'], 'borg_stopped')

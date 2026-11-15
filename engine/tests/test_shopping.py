"""Fixed-start checks of the real shopping errand and its handoffs.

Run under the shared validation lock after harness/build.sh. No mock shops
or fake completion predicates: the engine generates the stores and town.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(GAME.is_file(), 'harness/build.sh required')
class ShoppingTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='shopping-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def run_game(self, name, errand, seed=1001, cls=1, extra=(), budget=0):
        out = self.root / name
        outcome = self.root / (name + '.jsonl')
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(('BORG_', 'BORGH_'))}
        env.update(BORG_CORE='borg-rs', BORG_ERRAND=errand,
                   BORG_ERRAND_BUDGET=str(budget),
                   BORG_ERRAND_OUT=str(outcome), RUN_TIMEOUT_SECONDS='30',
                   BORG_DEATH_RECORDS_DIR=str(Path(tempfile.gettempdir()) /
                                              'borg-shopping-test-deaths'))
        subprocess.run([str(ROOT / 'harness/run.sh'), str(out), str(seed),
                        '0', str(cls), '20000', 'menu,finalsave', '--', *extra],
                       env=env, check=True, capture_output=True, text=True,
                       timeout=40)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertFalse(summary['dead'], summary)
        outcomes = [json.loads(line) for line in outcome.read_text().splitlines()] \
            if outcome.exists() else []
        return out, summary, outcomes

    def test_shops_home_and_two_rounds_hand_back_in_town(self):
        out, summary, outcomes = self.run_game('rounds', '/shopping; /shopping')
        self.assertEqual([r['outcome'] for r in outcomes], ['done', 'done'])
        self.assertTrue(all(r['depth'] == 0 for r in outcomes))
        self.assertEqual(summary['max_depth'], 0)
        self.assertEqual(summary['stop_reason'], 'borg_stopped')
        messages = (out / 'messages.log').read_text()
        self.assertIn('You bought', messages)
        self.assertIn('to the home', messages)
        self.assertIn('You drop a Scroll of Word of Recall', messages)
        # Handoff happens outside the store menu, and repeating a finished
        # round has no further transactions or game turns to consume.
        self.assertEqual(outcomes[0]['turn'], outcomes[1]['turn'])
        frames = [json.loads(line) for line in
                  (out / 'trace-core.jsonl').read_text().splitlines()]
        self.assertEqual(frames[-1]['context'], 'town')

    def test_town_danger_aborts_and_drops_the_next_errand(self):
        _, summary, outcomes = self.run_game(
            'danger', '/shopping; /dive 1', seed=1002, cls=3)
        self.assertEqual(len(outcomes), 1)
        self.assertEqual(outcomes[0]['outcome'],
                         'aborted: the core needs to leave town')
        self.assertEqual(summary['max_depth'], 0)
        self.assertEqual(summary['stop_reason'], 'borg_stopped')

    def test_dungeon_call_aborts_without_taking_a_step(self):
        fixture, before, _ = self.run_game(
            'fixture', '', seed=1010, cls=0, extra=('-depth', '1'))
        self.assertEqual(before['depth'], 1)
        _, after, outcomes = self.run_game(
            'dungeon', '/shopping', seed=1010, cls=0,
            extra=('-load', str(fixture / 'final.save'), '-keep-load-rng'))
        self.assertEqual(outcomes[0]['outcome'], 'aborted: shopping requires town')
        self.assertEqual(after['depth'], 1)
        self.assertEqual(after['game_turns'], before['game_turns'])

    def test_budget_drops_the_next_errand(self):
        _, summary, outcomes = self.run_game(
            'budget', '/shopping; /dive 1', budget=1)
        self.assertEqual([r['outcome'] for r in outcomes], ['budget'])
        self.assertEqual(summary['depth'], 0)
        self.assertEqual(summary['stop_reason'], 'borg_stopped')

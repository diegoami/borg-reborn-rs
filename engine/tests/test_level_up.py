"""Native checks for leveling, returning and handing control back in town."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless((ROOT / 'build/rust/game/angband').is_file(), 'native build required')
class LevelUpTest(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix='borg-level-up-')
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)

    def run_mage(self, name, level=2, budget=0, stop_level=51):
        out = self.root / name
        receipt = self.root / f'{name}.jsonl'
        env = {key: value for key, value in os.environ.items()
               if not key.startswith(('BORG_', 'BORGH_'))
               or key in {'BORG_CEMETERY_DIR', 'BORG_DEATH_RECORDS_DIR'}}
        env.update(BORG_CORE='borg-rs', BORG_ERRAND=f'/level-up {level}',
                   BORG_ERRAND_BUDGET=str(budget), BORG_ERRAND_OUT=str(receipt),
                   BORG_TXT_SET=f'borg_stop_clevel={stop_level}', RUN_TIMEOUT_SECONDS='30')
        result = subprocess.run(
            [str(ROOT / 'harness/run.sh'), str(out), '102', '2', '1', '2000', 'menu'],
            env=env, capture_output=True, text=True, timeout=40)
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertFalse(summary['dead'], summary)
        self.assertEqual(summary['stop_reason'], 'borg_stopped', summary)
        outcomes = [json.loads(line) for line in receipt.read_text().splitlines()] \
            if receipt.is_file() else []
        return summary, outcomes, (out / 'messages.log').read_text()

    def test_reaches_current_level_and_uses_recall(self):
        summary, outcomes, messages = self.run_mage('recall', stop_level=3)
        outcome, = outcomes
        self.assertEqual(outcome['errand'], 'LevelUp(2)')
        self.assertEqual(outcome['outcome'], 'done')
        self.assertEqual(outcome['clevel'], 2)
        self.assertEqual(outcome['depth'], 0)
        self.assertEqual(summary['depth'], 0)
        self.assertIn('Welcome to level 2.', messages)
        self.assertGreater(messages.index('Reading Scroll of Word of Recall'),
                           messages.index('Welcome to level 2.'))

    def test_same_budget_expires_while_returning(self):
        summary, outcomes, messages = self.run_mage('budget', budget=800)
        outcome, = outcomes
        self.assertEqual(outcome['errand'], 'LevelUp(2)')
        self.assertEqual(outcome['outcome'], 'budget')
        self.assertEqual(outcome['turn'], 801)
        self.assertEqual(outcome['clevel'], 2)
        self.assertEqual(summary['depth'], 1)
        self.assertIn('Reading Scroll of Word of Recall', messages)

    def test_stops_in_town_if_target_is_already_met(self):
        summary, outcomes, _ = self.run_mage('already-met', level=1)
        outcome, = outcomes
        self.assertEqual(outcome['outcome'], 'done')
        self.assertEqual(outcome['turn'], 1)
        self.assertEqual(summary['max_depth'], 0)

    def test_configured_auto_stop_still_takes_precedence(self):
        for stop in (1, 2):
            with self.subTest(stop=stop):
                summary, outcomes, messages = self.run_mage(f'auto-stop-{stop}', stop_level=stop)
                self.assertEqual(summary['clevel'], stop)
                self.assertEqual(outcomes, [])
                self.assertIn('Auto-stop for user CLevel', messages)

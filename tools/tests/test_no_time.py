"""Tests for tools/check-no-time.py."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('check_no_time', HERE.parent / 'check-no-time.py')
nt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nt)


class NoTimeTest(unittest.TestCase):
    def run_with(self, choices):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        run = Path(tmp.name) / 'run'
        run.mkdir()
        params = {'parameters': [
            {'name': 'max_tries_per_turn', 'value': 2},
            {'name': 'max_level_changes_per_turn', 'value': 2},
        ]}
        (run / 'params.json').write_text(json.dumps(params))
        (run / 'trace.jsonl').write_text(''.join(
            json.dumps({'turn': turn, 'choice': choice}) + '\n'
            for turn, choice in choices
        ))
        return nt.check_run(run)

    def test_two_level_changes_are_the_cap_not_a_reset(self):
        result = self.run_with([(5, 'Descend'), (5, 'Ascend')])
        self.assertEqual(result['max_level_changes'], 2)
        self.assertEqual(result['problems'], [])
        result = self.run_with([(5, 'Descend'), (5, 'Ascend'), (5, 'Descend')])
        self.assertEqual(result['problems'], ['turn 5: 3 level changes > cap 2'])

    def test_store_actions_do_not_hide_the_ordinary_budget(self):
        result = self.run_with([(5, 'Buy { ware: 0 }'), (5, 'Sell { item: 0, qty: 1 }'),
                                (5, 'Walk(6)'), (5, 'Walk(6)'), (5, 'Hold')])
        self.assertEqual(result['max_actions'], 5)
        self.assertEqual(result['max_nonstore'], 3)
        self.assertEqual(result['problems'], [])
        result = self.run_with([(5, 'Walk(6)')] * 4)
        self.assertEqual(result['problems'], ['turn 5: 4 non-store actions > cap 3'])


if __name__ == '__main__':
    unittest.main()

"""tools/check-perception.py (task T2.5, #16). Run: python3 -m unittest discover -s tools/tests"""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('check_perception', HERE.parent / 'check-perception.py')
cp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cp)

STATE = '11 123,37 d1/1 hp9/9 sp2/2 f8999 au325 xp0 lv1 t6:100 o- p0 e0 r0\n'


def think(turn=11, hp=9, differences=()):
    return {'turn': turn, 'differences': list(differences), 'map_compared': 5, 'map_stale_off_panel': 0,
            'map_unknown_to_world': 1, 'monsters_compared': 2, 'monsters_off_panel': 0,
            'state': {'x': 123, 'y': 37, 'depth': 1, 'max_depth': 1, 'hp': hp, 'max_hp': 9, 'sp': 2, 'max_sp': 2,
                      'food': 8999, 'gold': 325, 'exp': 0, 'level': 1, 'timed': [[6, 100]]}}


def run(base, *thinks, state=STATE):
    r = Path(base) / 'run'
    r.mkdir()
    (r / 'truth.jsonl').write_text(''.join(json.dumps(t) + '\n' for t in thinks))
    (r / 'state.log').write_text(state)
    return r


class CheckTest(unittest.TestCase):
    def test_agreement_passes(self):
        with tempfile.TemporaryDirectory() as t:
            thinks, totals, problems = cp.check_run(run(t, think(), think()))
            self.assertEqual((thinks, problems, totals['map_compared']), (2, [], 10))
            self.assertEqual(cp.main([t]), 0)

    def test_a_status_not_in_state_log_fails(self):
        with tempfile.TemporaryDirectory() as t:
            _, _, problems = cp.check_run(run(t, think(hp=8)))
            self.assertEqual(len(problems), 1)
            self.assertIn('state.log', problems[0])

    def test_a_think_with_no_state_line_at_its_turn_fails(self):
        with tempfile.TemporaryDirectory() as t:
            _, _, problems = cp.check_run(run(t, think(turn=12)))
            self.assertEqual(len(problems), 1)

    def test_an_engine_difference_fails(self):
        with tempfile.TemporaryDirectory() as t:
            run(t, think(differences=['monsters: visible but not seen: index 3']))
            self.assertEqual(cp.main([t]), 1)

    def test_no_truth_log_is_an_error(self):
        with tempfile.TemporaryDirectory() as t:
            self.assertEqual(cp.main([t]), 2)


if __name__ == '__main__':
    unittest.main()

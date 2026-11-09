"""Tests for tools/depth-stream.py. Run: python3 -m unittest discover -s tools/tests"""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('depth_stream', HERE.parent / 'depth-stream.py')
ds = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ds)


def run(tmp, states, **summary):
    d = Path(tmp)
    (d / 'state.log').write_text('\n'.join(states) + '\n')
    base = {'game_turns': 100000, 'max_depth': 0, 'dead': False, 'stop_reason': 'turns'}
    base.update(summary)
    (d / 'summary.json').write_text(json.dumps(base))
    return d


class DepthStreamTest(unittest.TestCase):
    def test_first_turn_at_each_depth(self):
        lines = ['10 1,1 d0/0 hp20/20', '500 2,2 d1/1 hp20/20', '900 3,3 d0/1 hp20/20',
                 '1200 4,4 d3/3 hp18/20', '1300 4,5 d2/3 hp18/20', '-', '1400 -']
        self.assertEqual(ds.stream(lines), {1: 500, 2: 1200, 3: 1200})

    def test_exact_boundary_and_final_depth_from_summary(self):
        with tempfile.TemporaryDirectory() as t:
            d = run(t, ['10 1,1 d1/1 hp1/1'], game_turns=100000, max_depth=2, exact_turns=True)
            out = ds.derive(d, 100000)
            self.assertEqual(out['boundary'], 'exact')
            self.assertEqual(out['first_turn_at_depth'], {'1': 10, '2': 100000})
            self.assertTrue(json.loads((d / 'depths.json').read_text())['alive'])

    def test_after_key_boundary_without_flag(self):
        with tempfile.TemporaryDirectory() as t:
            out = ds.derive(run(t, [], game_turns=100004), 100000)
            self.assertEqual(out['boundary'], 'after-key')
            self.assertEqual(out['first_turn_at_depth'], {})

    def test_exact_survivor_must_stop_at_the_cap(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                ds.derive(run(t, [], game_turns=100004, exact_turns=True), 100000)

    def test_death_before_cap_is_fine(self):
        with tempfile.TemporaryDirectory() as t:
            out = ds.derive(run(t, [], game_turns=7195, dead=True, stop_reason='death',
                                exact_turns=True), 100000)
            self.assertFalse(out['alive'])


if __name__ == '__main__':
    unittest.main()

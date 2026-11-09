"""Tests for tools/endpoint.py. Run: python3 -m unittest discover -s tools/tests"""

import contextlib
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('endpoint', HERE.parent / 'endpoint.py')
ep = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ep)

OUTCOMES = ['death', 'turn_cap', 'completion', 'stalled', 'wall_timeout', 'crash',
            'harness_failure', 'config_error', 'borg_stopped', 'decision_cap']


def cell(seed, outcome, max_depth=6, turn=100000, cls='Warrior', specimen='x', kind='better-borg-c'):
    return {'specimen': specimen, 'profile': 'default', 'seed': seed, 'outcome': outcome,
            'played_class': cls, 'harness_kind': kind,
            'final': {'max_depth': max_depth, 'turn': turn if outcome == 'turn_cap' else 5000}}


def write_cells(tmp, cells):
    p = Path(tmp) / 'cells.jsonl'
    p.write_text('\n'.join(json.dumps(c) for c in cells) + '\n')
    return p


def run_main(args):
    out = io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
        code = ep.main(args)
    return code, out.getvalue()


class EndpointTest(unittest.TestCase):
    def test_every_outcome_kind_is_in_the_denominator(self):
        with tempfile.TemporaryDirectory() as t:
            f = write_cells(t, [cell(i, o) for i, o in enumerate(OUTCOMES)])
            code, out = run_main(['--zoo', f'arm={f}', '--depth', '5', '--turns', '100000', '--json'])
            self.assertEqual(code, 0)
            row = json.loads(out)['rows'][0]
            self.assertEqual(row['attempted'], len(OUTCOMES))
            # turn_cap at depth 6 succeeds; completion (victory) succeeds; nothing else does.
            self.assertEqual(row['successes'], 2)
            self.assertEqual(row['deaths'], 1)
            self.assertEqual(set(row['operational']), set(OUTCOMES) - {'death', 'turn_cap', 'completion'})

    def test_depth_floor_and_turn_cap(self):
        with tempfile.TemporaryDirectory() as t:
            f = write_cells(t, [cell(1, 'turn_cap', max_depth=4), cell(2, 'turn_cap', max_depth=5),
                                cell(3, 'turn_cap', max_depth=9, turn=50000)])
            _, out = run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000', '--json'])
            self.assertEqual(json.loads(out)['rows'][0]['successes'], 1, 'depth 4 fails; a stop before T fails')

    def test_after_key_survivors_are_refused_unless_allowed(self):
        with tempfile.TemporaryDirectory() as t:
            f = write_cells(t, [cell(1, 'turn_cap', turn=100004, kind='borg-rs')])
            code, _ = run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000'])
            self.assertEqual(code, 2)
            code, out = run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000',
                                  '--allow-after-key', '--json'])
            self.assertEqual(code, 0)
            self.assertTrue(json.loads(out)['boundary_after_key_allowed'])

    def test_per_class_rows_and_specimen_filter(self):
        with tempfile.TemporaryDirectory() as t:
            f = write_cells(t, [cell(1, 'turn_cap', cls='Mage', specimen='a'),
                                cell(2, 'death', cls='Warrior', specimen='a'),
                                cell(3, 'turn_cap', cls='Warrior', specimen='b')])
            _, out = run_main(['--zoo', f'a={f}:a', '--depth', '5', '--turns', '100000', '--json'])
            rows = {r['cls']: r for r in json.loads(out)['rows']}
            self.assertEqual(rows['ALL']['attempted'], 2)
            self.assertEqual((rows['Mage']['successes'], rows['Warrior']['successes']), (1, 0))

    def test_paired_mcnemar(self):
        with tempfile.TemporaryDirectory() as t:
            a = [cell(i, 'turn_cap' if i < 8 else 'death', specimen='A') for i in range(10)]
            b = [cell(i, 'turn_cap' if i < 2 else 'death', specimen='B') for i in range(10)]
            f = write_cells(t, a + b)
            _, out = run_main(['--zoo', f'A={f}:A', '--zoo', f'B={f}:B', '--depth', '5', '--turns', '100000',
                               '--pair', 'A', 'B', '--json'])
            p = json.loads(out)['pairs'][0]
            self.assertEqual((p['shared'], p['a_only_success'], p['b_only_success']), (10, 6, 0))
            self.assertAlmostEqual(p['mcnemar_p'], 2 / 2 ** 6)

    def test_wilson_and_mcnemar_values(self):
        lo, hi = ep.wilson(0, 5)
        self.assertEqual(lo, 0.0)
        self.assertAlmostEqual(hi, 0.4345, places=3)
        self.assertEqual(ep.mcnemar_exact(0, 0), 1.0)
        self.assertAlmostEqual(ep.mcnemar_exact(1, 2), 1.0)

    def test_local_suite_dir_counts_missing_runs_as_harness_failures(self):
        with tempfile.TemporaryDirectory() as t:
            d = Path(t) / 'out'
            suite = Path(t) / 'suite'
            suite.write_text('one 1 0 0 1000 exact\ntwo 2 0 0 1000 exact\n')
            (d / 'one').mkdir(parents=True)
            (d / 'one' / 'summary.json').write_text(json.dumps(
                {'seed': 1, 'class': 'Warrior', 'stop_reason': 'turns', 'game_turns': 1000,
                 'max_depth': 5, 'dead': False, 'exact_turns': True}))
            (d / 'manifest').write_text(f'suite={suite}\n')
            _, out = run_main(['--local', f'x={d}', '--depth', '5', '--turns', '1000', '--json'])
            row = json.loads(out)['rows'][0]
            self.assertEqual((row['attempted'], row['successes']), (2, 1))
            self.assertEqual(row['operational'], {'harness_failure': 1})


if __name__ == '__main__':
    unittest.main()

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

    def local(self, t, runs, failed=(), suite_ok=None, binary_end='b' * 64, run_exit=None, depths=None):
        """A suite output: runs = [(name, seed, class_idx, summary or None)].
        suite_ok defaults to no failed run; run_exit and depths map run names
        to run.sh's exit status and to a depths.json."""
        d = Path(t) / 'out'
        suite = Path(t) / 'suite'
        suite.write_text(''.join(f'{n} {seed} 0 {c} 1000 exact\n' for n, seed, c, _ in runs))
        for n, seed, c, summ in runs:
            (d / n).mkdir(parents=True)
            ok = n not in failed
            (d / n / 'process.json').write_text(json.dumps(
                {'status': 'success' if ok else 'failure', 'exit_code': 0 if ok else 1,
                 'timed_out': False, 'stalled': False}))
            if summ is not None:
                (d / n / 'summary.json').write_text(json.dumps(summ))
            if n in (run_exit or {}):
                (d / n / 'run_exit').write_text(f'{run_exit[n]}\n')
            if n in (depths or {}):
                (d / n / 'depths.json').write_text(json.dumps(depths[n]))
        ok = not failed if suite_ok is None else suite_ok
        (d / 'manifest').write_text(f'suite=suite\nsuite_file={suite}\nbinary_sha256={"b" * 64}\n'
                                    f'binary_sha256_end={binary_end}\n'
                                    f'finished_utc=2026-11-09T00:00:00Z result={"OK" if ok else "FAILED"}\n')
        (d / ('DONE' if ok else 'FAILED')).write_text('x\n')
        return d

    def endpoint(self, d, depth=5):
        _, out = run_main(['--local', f'x={d}', '--depth', str(depth), '--turns', '1000', '--json'])
        return {r['cls']: r for r in json.loads(out)['rows']}

    def test_binary_changed_suite_has_no_successes(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1))], suite_ok=False, binary_end='c' * 64,
                           run_exit={'one': 0})
            row = self.endpoint(d)['ALL']
            self.assertEqual((row['attempted'], row['successes'], row['operational']),
                             (1, 0, {'binary_changed': 1}))

    def test_failed_suite_scores_only_runs_whose_run_sh_succeeded(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1)), ('two', 2, 0, self.alive(2)),
                               ('three', 3, 0, self.alive(3))],
                           suite_ok=False, run_exit={'one': 0, 'two': 1})
            row = self.endpoint(d)['ALL']
            # two: run.sh failed after a good receipt; three: no exit status in a FAILED suite.
            self.assertEqual((row['successes'], row['operational']), (1, {'harness_failure': 2}))

    def test_incomplete_or_contradictory_output_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1))])
            (d / 'DONE').unlink()
            with self.assertRaises(SystemExit):
                self.endpoint(d)
            (d / 'FAILED').write_text('x\n')   # FAILED marker, manifest says OK
            with self.assertRaises(SystemExit):
                self.endpoint(d)

    def test_depths_contradicting_the_summary_is_not_a_success(self):
        with tempfile.TemporaryDirectory() as t:
            good = {'boundary': 'exact', 'stop_turn': 1000, 'alive': True, 'max_depth_at_stop': 5}
            bad = dict(good, max_depth_at_stop=1)
            d = self.local(t, [('one', 1, 0, self.alive(1)), ('two', 2, 0, self.alive(2))],
                           depths={'one': good, 'two': bad})
            row = self.endpoint(d)['ALL']
            self.assertEqual((row['successes'], row['operational']), (1, {'evidence_conflict': 1}))

    def test_missing_run_of_an_index_that_played_two_classes_is_not_guessed(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 8, self.alive(1, cls='Mage')),
                               ('two', 2, 8, self.alive(2, cls='Warrior')), ('three', 3, 8, None)])
            rows = self.endpoint(d)
            self.assertEqual((rows['Mage']['attempted'], rows['Warrior']['attempted']), (1, 1))
            self.assertEqual(rows['class index 8 (ambiguous)']['operational'], {'harness_failure': 1})

    def alive(self, seed, turn=1000, depth=5, cls='Warrior'):
        return {'seed': seed, 'class': cls, 'stop_reason': 'turns', 'game_turns': turn,
                'max_depth': depth, 'dead': False, 'exact_turns': True}

    def test_missing_run_stays_in_its_class(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1)), ('two', 2, 0, None)])
            _, out = run_main(['--local', f'x={d}', '--depth', '5', '--turns', '1000', '--json'])
            rows = {r['cls']: r for r in json.loads(out)['rows']}
            self.assertEqual((rows['Warrior']['attempted'], rows['Warrior']['successes']), (2, 1))
            self.assertEqual(rows['Warrior']['operational'], {'harness_failure': 1})
            self.assertNotIn('unknown', rows)

    def test_exact_survivor_past_the_cap_is_not_a_success(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1, turn=1001))])
            _, out = run_main(['--local', f'x={d}', '--depth', '5', '--turns', '1000', '--json'])
            self.assertEqual(json.loads(out)['rows'][0]['successes'], 0)

    def test_failed_process_is_a_harness_failure_whatever_the_summary_says(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1))], failed={'one'})
            _, out = run_main(['--local', f'x={d}', '--depth', '5', '--turns', '1000', '--json'])
            row = json.loads(out)['rows'][0]
            self.assertEqual((row['successes'], row['operational']), (0, {'harness_failure': 1}))

    def test_unresolvable_suite_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            d = self.local(t, [('one', 1, 0, self.alive(1))])
            (d / 'manifest').write_text('suite=no-such-suite\nsuite_file=/nowhere/suite\n')
            with self.assertRaises(SystemExit):
                run_main(['--local', f'x={d}', '--depth', '5', '--turns', '1000'])

    def test_victory_needs_a_turn_before_t(self):
        r = ep.run('a', 1, 'Warrior', 'completion', True, 50, None, 'exact', 's', victory=True)
        for turn, ok in ((None, False), (5000, True), (100000, False), (100001, False)):
            r['stop_turn'] = turn
            self.assertEqual(ep.success(r, 5, 100000), ok, turn)

    def test_duplicate_pair_keys_are_refused(self):
        with tempfile.TemporaryDirectory() as t:
            f = write_cells(t, [cell(1, 'turn_cap', specimen='A'), cell(1, 'death', specimen='A'),
                                cell(1, 'turn_cap', specimen='B')])
            with self.assertRaises(SystemExit):
                run_main(['--zoo', f'A={f}:A', '--zoo', f'B={f}:B', '--depth', '5', '--turns', '100000',
                          '--pair', 'A', 'B'])

    def test_zoo_boundary_by_harness_kind(self):
        with tempfile.TemporaryDirectory() as t:
            for kind in ('neo-original', 'mandos', 'borg-rs'):
                f = write_cells(t, [cell(1, 'turn_cap', kind=kind)])
                code, _ = run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000'])
                self.assertEqual(code, 2, kind)
            f = write_cells(t, [cell(1, 'turn_cap', kind='something-new')])
            with self.assertRaises(SystemExit):
                run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000', '--allow-after-key'])
            c = cell(1, 'turn_cap', kind='something-new')
            c['boundary'] = 'exact'
            f = write_cells(t, [c])
            code, out = run_main(['--zoo', f'a={f}', '--depth', '5', '--turns', '100000', '--json'])
            self.assertEqual((code, json.loads(out)['rows'][0]['successes']), (0, 1))


if __name__ == '__main__':
    unittest.main()

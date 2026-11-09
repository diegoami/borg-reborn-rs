"""Tests for harness/check-boundary.py. Run: python3 -m unittest discover -s tools/tests"""

import contextlib
import hashlib
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('check_boundary', HERE.parent.parent / 'harness/check-boundary.py')
cb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cb)

CAP = 1000


def summary(stop='turns', turn=CAP, dead=False, exact=True):
    return {'stop_reason': stop, 'game_turns': turn, 'dead': dead, 'max_depth': 2,
            'exact_turns': exact, 'death_cause': 'Grip' if dead else None}


def output(root, name, summ, keys=b'a\nb\n', suite_sha=None):
    d = Path(root) / name
    suite = Path(root) / 'suite'
    suite.write_text(f'one 1 0 0 {CAP} exact\n')
    (d / 'one').mkdir(parents=True)
    (d / 'one' / 'summary.json').write_text(json.dumps(summ))
    (d / 'one' / 'keys.log').write_bytes(keys)
    (d / 'one' / 'depths.json').write_text(json.dumps(
        {'boundary': 'exact' if summ['exact_turns'] else 'after-key', 'cap': CAP,
         'max_depth_at_stop': 2, 'alive': not summ['dead']}))
    sha = suite_sha or hashlib.sha256(suite.read_bytes()).hexdigest()
    (d / 'manifest').write_text(f'suite=suite\nsuite_file={suite}\nsuite_sha256={sha}\n')
    return d


def check(exact, normal):
    with contextlib.redirect_stdout(io.StringIO()) as out:
        code = cb.main([str(exact), str(normal)])
    return code, out.getvalue()


class CheckBoundaryTest(unittest.TestCase):
    def test_survivor_at_the_cap_passes(self):
        with tempfile.TemporaryDirectory() as t:
            e = output(t, 'e', summary())
            n = output(t, 'n', summary(turn=CAP + 3, exact=False), keys=b'a\nb\nc\n')
            self.assertEqual(check(e, n)[0], 0)

    def test_wrong_suite_hash_fails(self):
        with tempfile.TemporaryDirectory() as t:
            e = output(t, 'e', summary(), suite_sha='0' * 64)
            n = output(t, 'n', summary(turn=CAP + 3, exact=False), keys=b'a\nb\nc\n')
            code, out = check(e, n)
            self.assertEqual(code, 1)
            self.assertIn('no suite file matches', out)

    def test_early_operational_stop_fails(self):
        with tempfile.TemporaryDirectory() as t:
            for stop in ('borg_stopped', 'stalled', 'keys'):
                e = output(t, f'e-{stop}', summary(stop=stop, turn=10))
                n = output(t, f'n-{stop}', summary(stop=stop, turn=10, exact=False))
                code, out = check(e, n)
                self.assertEqual(code, 1, stop)
                self.assertIn('before the cap', out)

    def test_same_death_before_the_cap_passes(self):
        with tempfile.TemporaryDirectory() as t:
            e = output(t, 'e', summary(stop='death', turn=500, dead=True))
            n = output(t, 'n', summary(stop='death', turn=500, dead=True, exact=False))
            self.assertEqual(check(e, n)[0], 0)


if __name__ == '__main__':
    unittest.main()

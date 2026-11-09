"""tools/rerun-death.py (task TE.6, #76). Run: python3 -m unittest discover -s tools/tests"""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('rerun_death', HERE.parent / 'rerun-death.py')
rd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rd)


def dossier(root, schema='borg-reborn-rs/death-record@1', impl='rust', flags='menu,exact', seed=102):
    d = Path(root) / 'D0001-rust-seed102'
    d.mkdir()
    (d / 'record.json').write_text(json.dumps({
        'schema': schema, 'identity': {'impl': impl, 'seed': seed, 'flags': flags, 'source_sha256': 'abc', 'git_rev': 'f00'},
        'terminal': {'turn': 743, 'race_idx': 2, 'class_idx': 1},
        'attempts': [{'outcome': 'death', 'result_sha256': 'k' * 64}]}))
    (d / 'summary.json').write_text(json.dumps({'seed': seed, 'race_idx': 2, 'class_idx': 1, 'game_turns': 743}))
    (d / 'config.txt').write_text(f'flags={flags}\nborg_txt_sha256=x\nprofile=harness/profiles/reborn-default.txt\n')
    return d


class PlanTest(unittest.TestCase):
    def test_a_dossier_gives_the_same_run(self):
        with tempfile.TemporaryDirectory() as t:
            p = rd.plan(dossier(t))
        self.assertEqual((p['seed'], p['race'], p['class']), (102, 2, 1))
        self.assertEqual(p['cap'], 744, 'the death comes before the cap')
        self.assertEqual(p['flags'], 'menu', 'no exact stop: the game ends at the death')
        self.assertEqual(p['profile'], 'harness/profiles/reborn-default.txt')
        self.assertEqual(p['keys_sha256'], 'k' * 64)

    def test_only_rust_harness_dossiers(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, schema='borg-rs/death-record@1'))
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, impl='c-pristine'))
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, flags='save=x.save,keep-load-rng'))

    def test_another_build_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            d = dossier(t)
            with self.assertRaises(SystemExit) as e:
                rd.main([str(d), str(Path(t) / 'out')])
            self.assertIn('check out f00', str(e.exception))


if __name__ == '__main__':
    unittest.main()

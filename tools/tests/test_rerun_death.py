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


def dossier(root, schema='borg-reborn-rs/death-record@1', impl='rust', flags='menu,exact', seed=102,
            summary_extra=None, config_extra=''):
    d = Path(root) / 'D0001-rust-seed102'
    d.mkdir()
    (d / 'record.json').write_text(json.dumps({
        'schema': schema, 'identity': {'impl': impl, 'seed': seed, 'flags': flags, 'source_sha256': 'abc', 'git_rev': 'f00'},
        'terminal': {'turn': 743, 'race_idx': 2, 'class_idx': 1},
        'attempts': [{'outcome': 'death', 'result_sha256': 'k' * 64}]}))
    (d / 'summary.json').write_text(json.dumps({'seed': seed, 'race_idx': 2, 'class_idx': 1, 'game_turns': 743,
                                                **(summary_extra or {})}))
    (d / 'config.txt').write_text(f'flags={flags}\nborg_txt_sha256=x\nprofile=harness/profiles/reborn-default.txt\n'
                                  + config_extra)
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

    def test_the_runs_settings_come_back(self):
        # Review #227, finding 8: the planner restores a run's settings. Since
        # #1339 the errands, /stopscared and the plans are retired, so on a
        # current build a restored active BORG_ERRAND or BORG_STOPSCARED stops
        # the game at start (exit 2, check_retired_errand_settings) instead of
        # replaying the death without its errand: it fails loudly, and the
        # death replays at its own build (identity.git_rev). The cemetery-owned
        # planner may refuse such a dossier itself; until it does, this test
        # pins what it restores.
        with tempfile.TemporaryDirectory() as t:
            p = rd.plan(dossier(t, config_extra='borg_core=borg-rs\nborg_errand=/dive 3; /walk-back\n'
                                                 'borg_stopscared=50\n'))
        self.assertEqual(p['env'], {'BORG_CORE': 'borg-rs', 'BORG_ERRAND': '/dive 3; /walk-back',
                                    'BORG_STOPSCARED': '50'})
        with tempfile.TemporaryDirectory() as t:
            self.assertEqual(rd.plan(dossier(t))['env'], {'BORG_CORE': 'reborn'},
                             'a historical unset controller must not inherit the modern native default')
        with tempfile.TemporaryDirectory() as t:
            p = rd.plan(dossier(t, config_extra='borg_errand=\n'))
        self.assertEqual(p['env'], {'BORG_CORE': 'reborn', 'BORG_ERRAND': ''}, 'set but empty stays set')

    def test_a_changed_ladders_file_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            ladders = Path(t) / 'ladders.txt'
            ladders.write_text('warrior = trips:1*\n')
            good = rd.hashlib.sha256(ladders.read_bytes()).hexdigest()
            p = rd.plan(dossier(t, config_extra=f'borg_ladders={ladders}\nborg_ladders_sha256={good}\n'))
            self.assertEqual(p['env']['BORG_LADDERS'], str(ladders))
            ladders.write_text('warrior = trips:1,2*\n')
            (Path(t) / 'D0001-rust-seed102').rename(Path(t) / 'old')
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, config_extra=f'borg_ladders={ladders}\nborg_ladders_sha256={good}\n'))

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

    def test_a_load_through_the_extra_options_is_refused(self):
        # E review, finding 4: run.sh -- -load FILE has flags=-.
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, flags='-', config_extra='save_sha256=ab\n'))
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit):
                rd.plan(dossier(t, flags='-', summary_extra={'loaded': True}))

    def test_another_build_is_refused(self):
        with tempfile.TemporaryDirectory() as t:
            d = dossier(t)
            with self.assertRaises(SystemExit) as e:
                rd.main([str(d), str(Path(t) / 'out')])
            self.assertIn('check out f00', str(e.exception))


if __name__ == '__main__':
    unittest.main()

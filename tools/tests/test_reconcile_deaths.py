"""tools/reconcile-deaths.py --runs (E review, finding 3). Run: python3 -m unittest discover -s tools/tests"""

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('reconcile_deaths', HERE.parent / 'reconcile-deaths.py')
rc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rc)


def run(base, name, dead=True, keys='k\n'):
    r = Path(base) / 'runs' / name
    r.mkdir(parents=True)
    (r / 'summary.json').write_text(json.dumps({'dead': dead, 'stop_reason': 'death' if dead else 'turns',
                                                'death_cause': 'a cave spider' if dead else '', 'game_turns': 9}))
    (r / 'keys.log').write_text(keys)
    (r / 'config.txt').write_text('flags=menu\n')
    return hashlib.sha256(keys.encode()).hexdigest()


def build_id(base, source='cafe'):
    path = Path(base) / 'BUILD_ID'
    path.write_text(f'source_sha256={source}\n')
    return path


def family(source='cafe'):
    summary = {'dead': True, 'stop_reason': 'death', 'death_cause': 'a cave spider', 'game_turns': 9}
    return rc.exporter().run_family(summary, {'flags': 'menu'}, 'rust', source, 'menu')


def dossier(registry, case, sha, fam=None):
    d = Path(registry) / case
    d.mkdir(parents=True)
    (d / 'record.json').write_text(json.dumps({'encounter_family': fam or family(), 'raw_cause': 'x',
                                               'terminal': {'turn': 9},
                                               'attempts': [{'outcome': 'death', 'result_sha256': sha}]}))


class RunsTest(unittest.TestCase):
    def test_an_unexported_death_fails_even_with_an_empty_registry(self):
        with tempfile.TemporaryDirectory() as t:
            run(t, 'a')
            run(t, 'alive', dead=False)
            (Path(t) / 'temp').mkdir()
            deaths, missing = rc.unexported([Path(t) / 'runs'], Path(t) / 'temp', build_id(t))
            self.assertEqual((deaths, [m.name for m, _ in missing]), (1, ['a']))
            self.assertEqual(rc.main([str(Path(t) / 'temp'), '--registry', str(Path(t) / 'temp'),
                                      '--runs', str(Path(t) / 'runs'), '--build-id', str(build_id(t))]), 1)

    def test_an_exported_death_passes(self):
        with tempfile.TemporaryDirectory() as t:
            sha = run(t, 'a')
            dossier(Path(t) / 'temp', 'D0001-rust-seed1', sha)
            self.assertEqual(rc.unexported([Path(t) / 'runs'], Path(t) / 'temp', build_id(t)), (1, []))
            self.assertEqual(rc.main([str(Path(t) / 'temp'), '--registry', str(Path(t) / 'temp'),
                                      '--runs', str(Path(t) / 'runs'), '--build-id', str(build_id(t))]), 0)

    def test_the_same_keys_under_another_family_are_another_death(self):
        # M2 review, finding 7: a dossier of another build (or seed, or
        # character) with the same key stream does not cover this death.
        with tempfile.TemporaryDirectory() as t:
            sha = run(t, 'a')
            dossier(Path(t) / 'temp', 'D0001-rust-seed1', sha, fam=family(source='other'))
            deaths, missing = rc.unexported([Path(t) / 'runs'], Path(t) / 'temp', build_id(t))
            self.assertEqual((deaths, len(missing)), (1, 1))

    def test_a_suite_run_takes_its_build_from_the_manifest(self):
        with tempfile.TemporaryDirectory() as t:
            sha = run(t, 'a')
            (Path(t) / 'runs' / 'manifest').write_text('build_id_source_sha256=fromsuite\n')
            dossier(Path(t) / 'temp', 'D0001-rust-seed1', sha, fam=family(source='fromsuite'))
            self.assertEqual(rc.unexported([Path(t) / 'runs'], Path(t) / 'temp', build_id(t, 'cafe')), (1, []))


if __name__ == '__main__':
    unittest.main()

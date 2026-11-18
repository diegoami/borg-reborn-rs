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
    def retained_pair(self, root, *, seed=1, race=0, cls=0, save=None, keys='k\n'):
        """Synthetic registry records, not new native-game observations."""
        sha = hashlib.sha256(b'k\n').hexdigest()
        for registry, fam, result, start in [
            ('canonical', 'old-build', sha, (1, 0, 0, None)),
            ('temporary', 'new-build', hashlib.sha256(keys.encode()).hexdigest(),
             (seed, race, cls, save)),
        ]:
            directory = Path(root) / registry
            dossier(directory, 'D0001', result, fam=fam)
            case = directory / 'D0001'
            record = json.loads((case / 'record.json').read_text())
            record['identity'] = {'impl': 'rust', 'seed': start[0]}
            record['terminal'].update(race_idx=start[1], class_idx=start[2])
            (case / 'record.json').write_text(json.dumps(record))
            (case / 'config.txt').write_text(
                f'flags={registry}\nborg_txt_sha256={registry}\n' +
                (f'save_sha256={start[3]}\n' if start[3] is not None else ''))
        return [str(Path(root) / 'temporary'), '--registry', str(Path(root) / 'canonical')]

    def test_same_start_and_keys_reuse_canonical_game_across_builds_and_settings(self):
        with tempfile.TemporaryDirectory() as temporary:
            args = self.retained_pair(temporary)
            canonical = Path(temporary) / 'canonical/D0001/record.json'
            original = canonical.read_bytes()
            self.assertEqual(rc.main(args), 0)
            self.assertEqual(rc.main(args + ['--exact']), 1)
            self.assertEqual(canonical.read_bytes(), original)

    def test_different_start_or_keys_still_require_a_canonical_game(self):
        for change in [dict(seed=2), dict(race=1), dict(cls=1),
                       dict(save='another-save'), dict(keys='different keys\n')]:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                self.assertEqual(rc.main(self.retained_pair(temporary, **change)), 1)

    def test_unknown_start_cannot_alias_a_different_build(self):
        with tempfile.TemporaryDirectory() as temporary:
            args = self.retained_pair(temporary)
            path = Path(temporary) / 'temporary/D0001/record.json'
            record = json.loads(path.read_text())
            del record['identity']
            path.write_text(json.dumps(record))
            self.assertEqual(rc.main(args), 1)

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

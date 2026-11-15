"""Tests for tools/death_registry.py, the registry's index generator.

Run: python3 -m unittest discover -s tools/tests
"""

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('death_registry', HERE.parent / 'death_registry.py')
dr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dr)


def case(root, name, cause='a cave spider', family='start-1', turn=1234):
    path = Path(root) / name
    path.mkdir(parents=True)
    (path / 'record.json').write_text(json.dumps({
        'schema': 'borg-reborn-rs/death-record@1',
        'id': name, 'character': 'Human Warrior', 'raw_cause': cause,
        'encounter_family': family,
        'terminal': {'turn': turn},
        'identity': {'seed': 101, 'source': 'abcdef0123456789'},
        'attempts': [{'outcome': 'death', 'result_sha256': 'a' * 64,
                      'source_commit': 'abcdef0123456789'}],
    }))
    (path / 'assessment.json').write_text(json.dumps({'classification': 'unassessed'}))
    (path / 'report.md').write_text(f'# {name}\n')
    return path


class RegistryTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.reg = Path(self.tmp.name) / 'deaths'
        self.reg.mkdir()
        os.environ['BORG_DEATH_RECORDS_DIR'] = str(self.reg)
        case(self.reg, 'D0001-rust-seed101')

    def tearDown(self):
        os.environ.pop('BORG_DEATH_RECORDS_DIR', None)
        self.tmp.cleanup()

    def test_the_registry_index_names_the_canonical_home(self):
        # The generator rewrites README.md from its template, so a hand
        # written notice there would be lost on the next run. The pointer
        # to the canonical registry has to live in the template.
        dr.update_index(self.reg)
        readme = (self.reg / 'README.md').read_text()
        self.assertIn('borg-cemetery', readme)
        self.assertIn('POINTER.md', readme)
        self.assertIn('BORG_DEATH_RECORDS_DIR', readme)

    def test_the_pointer_survives_a_second_regeneration(self):
        dr.update_index(self.reg)
        dr.update_index(self.reg)
        readme = (self.reg / 'README.md').read_text()
        self.assertIn('borg-cemetery', readme)
        # The notice is one block, not one per regeneration.
        self.assertEqual(readme.count('The canonical registry lives in'), 1,
                         'the pointer is duplicated by the template')
        self.assertEqual(readme.count('(POINTER.md)'), 1)

    def test_the_index_json_lists_the_cases(self):
        dr.update_index(self.reg)
        index = json.loads((self.reg / 'index.json').read_text())
        self.assertEqual(index['cases'], 1)
        self.assertEqual(index['schema'], 'borg-rs/death-index@1')
        self.assertIn('D0001-rust-seed101', (self.reg / 'README.md').read_text())

    def test_the_registry_directory_is_overridable(self):
        # The same seam every tool honours: a checkout can keep the
        # dossiers outside the tree.
        other = Path(self.tmp.name) / 'elsewhere'
        other.mkdir()
        case(other, 'D0200-other')
        os.environ['BORG_DEATH_RECORDS_DIR'] = str(other)
        self.assertEqual(dr.registry_root(), other.resolve())


if __name__ == '__main__':
    unittest.main()

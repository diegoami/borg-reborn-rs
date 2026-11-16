"""Real exporter -> qualified import, including concurrent isolated registries."""
import concurrent.futures
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

from test_export_death import ed, make_run

HERE = Path(__file__).resolve().parent
SCRIPT = HERE.parent / 'import-deaths.py'
spec = importlib.util.spec_from_file_location('qualified_import', SCRIPT)
imp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(imp)


class QualifiedImportTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.build = self.root / 'BUILD_ID'
        self.build.write_text('impl=rust\ngit_rev=abc\nsource_sha256=source-a\nbinary_sha256=bin\n')
        self.dest = self.root / 'archive'

    def exported(self, name, seed=101, keys='abc\n'):
        run = make_run(self.root, name, seed=seed, keys=keys)
        registry = self.root / (name + '-registry')
        before = os.environ.get('BORG_DEATH_RECORDS_DIR')
        os.environ['BORG_DEATH_RECORDS_DIR'] = str(registry)
        try:
            case, _ = ed.export(run, 'rust', self.build)
        finally:
            if before is None:
                os.environ.pop('BORG_DEATH_RECORDS_DIR', None)
            else:
                os.environ['BORG_DEATH_RECORDS_DIR'] = before
        return registry / case

    def test_same_numeric_ids_from_two_branches_preserve_all_evidence(self):
        left = self.exported('left')
        right = self.exported('right', seed=102)
        self.assertTrue(left.name.startswith('D0001-') and right.name.startswith('D0001-'))
        results = imp.import_packages([left, right], self.dest)
        self.assertNotEqual(results[0][0], results[1][0])
        for source, (identifier, _) in zip([left, right], results):
            original = json.loads((source / 'record.json').read_text())
            target = self.dest / identifier
            record = json.loads((target / 'record.json').read_text())
            self.assertEqual(record, dict(original, id=identifier))
            self.assertEqual((target / imp.ORIGINAL).read_bytes(), (source / 'record.json').read_bytes())
            for file in source.iterdir():
                if file.name != 'record.json':
                    self.assertEqual((target / file.name).read_bytes(), file.read_bytes())
            self.assertEqual(imp.package(target)[0], identifier)
        self.assertEqual(json.loads((self.dest / 'index.json').read_text())['distinct_starting_conditions'], 2)

    @unittest.skipUnless(os.name == 'posix', 'POSIX directory access modes')
    def test_published_package_preserves_source_directory_access(self):
        for mode in [0o750, 0o700]:
            source = self.exported(f'access-{mode}')
            source.chmod(mode)
            (source / 'report.md').chmod(0o600)
            (source / 'record.json').chmod(0o600)
            previous = os.umask(0o022)
            try:
                identifier, _ = imp.import_packages([source], self.dest)[0]
            finally:
                os.umask(previous)
            target = self.dest / identifier
            for name in ['report.md', 'record.json', imp.ORIGINAL, imp.RECEIPT]:
                self.assertEqual((target / name).stat().st_mode & 0o777, 0o600)
            self.assertEqual(target.stat().st_mode & 0o777, mode)
            self.assertEqual(imp.package(target)[0], identifier)
            self.assertEqual((target / imp.ORIGINAL).read_bytes(),
                             (source / 'record.json').read_bytes())

    @unittest.skipUnless(os.name == 'posix', 'POSIX directory access modes')
    def test_directory_and_file_modes_are_bounded_without_changing_evidence(self):
        for source_mode in [0o777, 0o755, 0o750, 0o700, 0o3777]:
            for mask in [0o000, 0o022, 0o027, 0o077]:
                with self.subTest(source_mode=oct(source_mode), umask=oct(mask)):
                    source = self.exported(f'modes-{source_mode}-{mask}')
                    source.chmod(source_mode)
                    for name in ['record.json', 'report.md']:
                        (source / name).chmod(0o600)
                    (source / 'summary.json').chmod(0o664)
                    expected_id, expected_files = imp.package(source)
                    real_write = Path.write_bytes

                    def check_private(asset, data):
                        if asset.parent.name.startswith('.import-'):
                            self.assertEqual(asset.parent.stat().st_mode & 0o777, 0o700)
                        return real_write(asset, data)

                    previous = os.umask(mask)
                    try:
                        with mock.patch.object(Path, 'write_bytes', check_private):
                            identifier, _ = imp.import_packages([source], self.dest)[0]
                    finally:
                        os.umask(previous)
                    target = self.dest / identifier
                    self.assertEqual(identifier, expected_id)
                    self.assertEqual(target.stat().st_mode & 0o7777,
                                     source_mode & ~mask & 0o777)
                    self.assertEqual(source.stat().st_mode & 0o7777, source_mode)
                    for name, data in expected_files.items():
                        self.assertEqual((target / name).read_bytes(), data)
                        original = source / name
                        if not original.exists():
                            original = source / 'record.json'
                        self.assertEqual((target / name).stat().st_mode & 0o7777,
                                         original.stat().st_mode & ~mask & 0o666)
                    modes = {p.name: p.stat().st_mode for p in target.iterdir()}
                    directory_mode = target.stat().st_mode
                    previous = os.umask(mask ^ 0o077)
                    try:
                        self.assertEqual(imp.import_packages([source], self.dest),
                                         [(identifier, 'already retained')])
                    finally:
                        os.umask(previous)
                    self.assertEqual(target.stat().st_mode, directory_mode)
                    self.assertEqual({p.name: p.stat().st_mode for p in target.iterdir()}, modes)
                    self.assertEqual(list(self.dest.glob('.import-*')), [])

    def test_identical_import_and_reimport_are_idempotent(self):
        source = self.exported('one')
        identifier, _ = imp.import_packages([source], self.dest)[0]
        before = {p.name: p.read_bytes() for p in (self.dest / identifier).iterdir()}
        self.assertEqual(imp.import_packages([source, self.dest / identifier], self.dest),
                         [(identifier, 'already retained'), (identifier, 'already retained')])
        self.assertEqual(before, {p.name: p.read_bytes() for p in (self.dest / identifier).iterdir()})
        self.assertEqual(json.loads((self.dest / 'index.json').read_text())['death_process_attempts'], 1)

    def test_repeated_measurement_packages_do_not_inflate_independence(self):
        one = self.exported('repeat-one')
        two = self.exported('repeat-two')
        result = imp.import_packages([one, two], self.dest)
        self.assertNotEqual(result[0][0], result[1][0], 'different provenance must remain retained')
        index = json.loads((self.dest / 'index.json').read_text())
        self.assertEqual(index['cases'], 2)
        self.assertEqual(index['distinct_starting_conditions'], 1)
        self.assertEqual(index['death_process_attempts'], 1)

    def test_concurrent_process_imports_have_no_overwrite_or_partial_case(self):
        sources = [self.exported('parallel-a'), self.exported('parallel-b', seed=102)]
        def run(source):
            return subprocess.run([sys.executable, str(SCRIPT), '--registry', str(self.dest), str(source)],
                                  capture_output=True, text=True, timeout=30)
        with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
            results = list(pool.map(run, sources * 3))
        for result in results:
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(list(self.dest.glob('D*'))), 2)
        self.assertEqual(list(self.dest.glob('.import-*')), [])
        for case in self.dest.glob('D*'):
            imp.package(case)

    def test_hash_damage_and_unknown_bare_ids_are_rejected(self):
        source = self.exported('bad')
        (source / 'summary.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            imp.import_packages([source], self.dest)
        good = self.exported('good')
        legacy = self.root / 'legacy.json'
        legacy.write_text(json.dumps({'schema': 'borg-reborn-rs/legacy-dossiers@1', 'cases': {}}))
        imp.import_packages([good], self.dest)
        self.assertEqual(imp.check_registry(self.dest, legacy), 1)
        import shutil
        shutil.copytree(good, self.dest / good.name)
        with self.assertRaisesRegex(ValueError, 'provisional numeric ID'):
            imp.check_registry(self.dest, legacy)

    def test_existing_package_and_legacy_identities_cannot_be_overwritten(self):
        source = self.exported('historical')
        identifier, _ = imp.import_packages([source], self.dest)[0]
        target = self.dest / identifier
        (target / 'report.md').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'existing package differs'):
            imp.import_packages([source], self.dest)
        legacy = self.root / 'legacy.json'
        original = json.loads((source / 'record.json').read_text())
        legacy.write_text(json.dumps({'schema': 'borg-reborn-rs/legacy-dossiers@1',
                                     'cases': {source.name: imp.legacy_identity(original)}}))
        self.assertEqual(imp.check_registry(source.parent, legacy), 1)
        original['identity']['source_sha256'] = 'wrong'
        (source / 'record.json').write_text(json.dumps(original))
        with self.assertRaisesRegex(ValueError, 'historical identity changed'):
            imp.check_registry(source.parent, legacy)

    def test_historical_payload_damage_and_missing_cases_fail_the_gate(self):
        source = self.exported('frozen')
        record = json.loads((source / 'record.json').read_text())
        legacy = self.root / 'legacy.json'
        legacy.write_text(json.dumps({'schema': 'borg-reborn-rs/legacy-dossiers@1',
                                     'cases': {source.name: imp.legacy_identity(record)}}))
        self.assertEqual(imp.check_registry(source.parent, legacy), 1)
        (source / 'summary.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'historical file hash changed'):
            imp.check_registry(source.parent, legacy)
        with self.assertRaisesRegex(ValueError, 'historical cases disappeared'):
            imp.check_registry(self.root / 'empty', legacy)

    def test_new_package_mutation_and_symlink_sources_are_rejected(self):
        source = self.exported('linked')
        linked = self.root / 'symlink'
        linked.symlink_to(source, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'real dossier directory'):
            imp.import_packages([linked], self.dest)
        identifier, _ = imp.import_packages([source], self.dest)[0]
        target = self.dest / identifier
        (target / 'report.md').write_text('changed analysis in immutable evidence')
        with self.assertRaises(ValueError):
            imp.package(target)

    def test_canonical_cemetery_requires_its_owned_registrar(self):
        from cemetery_tools import locate
        with self.assertRaisesRegex(ValueError, 'owned by its registrar'):
            imp.import_packages([self.exported('canonical')], locate(imp.ROOT) / 'deaths')


if __name__ == '__main__':
    unittest.main()

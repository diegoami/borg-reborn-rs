"""External evidence cannot bypass legacy, byte-identity or reconciliation checks."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from test_export_death import ed, make_run

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('external_evidence', ROOT / 'tools/prepare-validation-evidence.py')
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


class CemeteryEvidenceTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.cemetery = self.root / 'cemetery'
        subprocess.run(['git', 'init', '-q', str(self.cemetery)], check=True)
        self.base = self.cemetery / 'runs/validation/attack'
        self.base.mkdir(parents=True)
        build = self.root / 'BUILD_ID'
        build.write_text('impl=rust\ngit_rev=abc\nsource_sha256=input-a\nbinary_sha256=bin\n')
        run = make_run(self.root, 'native')
        registry = self.root / 'temporary-registry'
        from unittest.mock import patch
        with patch.dict(os.environ, {'BORG_DEATH_RECORDS_DIR': str(registry)}):
            case, _ = ed.export(run, 'rust', build)
        identifier, files = evidence.imports.package(registry / case)
        self.case = self.base / 'archive' / identifier
        self.case.mkdir(parents=True)
        for name, data in files.items():
            (self.case / name).write_bytes(data)
        self.historical = self.root / 'historical'
        self.historical.mkdir()
        self.legacy = self.root / 'legacy.json'
        self.legacy.write_text(json.dumps({'schema':'borg-reborn-rs/legacy-dossiers@1', 'cases':{}}))
        self.receipt = {'schema':'borg-reborn-rs/cemetery-validation-receipt@1',
                        'repository':'diegoami/borg-cemetery', 'root':'runs/validation/attack'}
        self.refresh()

    def refresh(self):
        manifest = self.base / 'manifest.json'
        inventory = {p.relative_to(self.base).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                     for p in self.base.rglob('*') if p.is_file() and p != manifest}
        manifest.write_text(json.dumps({'schema':'borg-cemetery/validation-bundle@1',
                                       'qualified_packages':1, 'files_sha256':inventory}))
        subprocess.run(['git', '-C', str(self.cemetery), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(self.cemetery), '-c', 'user.name=Test', '-c',
                        'user.email=test@example.test', 'commit', '-qm', 'Evidence', '--allow-empty'], check=True)
        self.receipt.update(commit=subprocess.check_output(['git', '-C', str(self.cemetery), 'rev-parse', 'HEAD'], text=True).strip(),
                            manifest_sha256=evidence.digest(manifest))

    def prepare(self):
        return evidence.prepare(self.cemetery, [self.receipt], self.root / 'mirror', self.historical, self.legacy)

    def test_verified_external_package_is_copied_with_original_bytes(self):
        self.assertEqual(self.prepare(), 1)
        mirror = self.root / 'mirror' / self.case.name
        self.assertEqual(evidence.imports.package(mirror), evidence.imports.package(self.case))
        self.assertFalse(mirror.is_symlink())
        with self.assertRaisesRegex(ValueError, 'output exists'):
            self.prepare()

    def test_wrong_commit_repository_and_manifest_are_rejected(self):
        for name, value, message in [('commit', '0'*40, 'checkout differs'),
                                     ('repository','someone/else','exact repository'),
                                     ('manifest_sha256','0'*64,'manifest hash')]:
            old = self.receipt[name];self.receipt[name] = value
            with self.subTest(field=name), self.assertRaisesRegex(ValueError, message):
                self.prepare()
            self.receipt[name] = old

    def test_missing_changed_and_unexpected_files_are_rejected(self):
        path = self.case / 'assessment.json';original = path.read_bytes()
        path.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'payload hash'):
            self.prepare()
        path.unlink()
        with self.assertRaisesRegex(ValueError, 'inventory differs'):
            self.prepare()
        path.write_bytes(original)
        (self.base / 'unexpected').write_text('extra')
        with self.assertRaisesRegex(ValueError, 'inventory differs'):
            self.prepare()

    def test_rehashed_mutated_packet_still_fails_qualified_identity(self):
        (self.case / 'assessment.json').write_text('{}')
        self.refresh()
        with self.assertRaises(ValueError):
            self.prepare()

    def test_symlinks_and_path_traversal_are_rejected(self):
        manifest = self.base / 'manifest.json';saved = manifest.read_bytes();manifest.unlink()
        target = self.root / 'manifest';target.write_bytes(saved);manifest.symlink_to(target)
        with self.assertRaisesRegex(ValueError, 'manifest hash'):
            self.prepare()
        manifest.unlink();manifest.write_bytes(saved)
        (self.base / 'linked').symlink_to(self.case, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'symlinks'):
            self.prepare()
        for value in ['../attack', '/tmp', 'runs/../attack', 'runs\\attack', './runs']:
            with self.subTest(path=value), self.assertRaisesRegex(ValueError, 'unsafe evidence path'):
                evidence.relative(value)

    def test_legacy_identity_remains_mandatory(self):
        self.legacy.write_text(json.dumps({'schema':'borg-reborn-rs/legacy-dossiers@1', 'cases':{'D0001':{}}}))
        with self.assertRaisesRegex(ValueError, 'historical cases disappeared'):
            self.prepare()

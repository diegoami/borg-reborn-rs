#!/usr/bin/env python3
"""Verify pinned cemetery evidence and build a disposable reconciliation mirror.

Canonical incident bytes remain in borg-cemetery. This only checks receipts
and materializes their immutable packages beside the unchanged historical
archive for the existing cemetery-owned reconciler.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('qualified_import', ROOT / 'tools/import-deaths.py')
imports = importlib.util.module_from_spec(spec)
spec.loader.exec_module(imports)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def relative(value):
    path = PurePosixPath(value)
    if not value or path.is_absolute() or any(p in ('', '.', '..') for p in value.split('/')) or '\\' in value:
        raise ValueError('unsafe evidence path')
    return path


def verify(cemetery, receipt):
    if receipt.get('schema') != 'borg-reborn-rs/cemetery-validation-receipt@1':
        raise ValueError('unsupported cemetery receipt')
    if receipt.get('repository') != 'diegoami/borg-cemetery' or not re.fullmatch('[0-9a-f]{40}', receipt.get('commit', '')):
        raise ValueError('cemetery evidence needs an exact repository/commit')
    actual = subprocess.check_output(['git', '-C', str(cemetery), 'rev-parse', 'HEAD'], text=True).strip()
    if actual != receipt['commit']:
        raise ValueError('cemetery checkout differs from receipt commit')
    base = cemetery.joinpath(*relative(receipt['root']).parts)
    # Check every component, including ancestors of the dataset, before reading.
    current = cemetery
    for component in relative(receipt['root']).parts:
        current /= component
        if current.is_symlink():
            raise ValueError('evidence symlinks are forbidden')
    manifest = base / 'manifest.json'
    if manifest.is_symlink() or not manifest.is_file() or digest(manifest) != receipt['manifest_sha256']:
        raise ValueError('cemetery manifest hash differs')
    expected = json.loads(manifest.read_text())
    if expected.get('schema') != 'borg-cemetery/validation-bundle@1':
        raise ValueError('unsupported cemetery manifest')
    inventory = expected['files_sha256']
    actual_paths = set()
    for path in base.rglob('*'):
        if path.is_symlink():
            raise ValueError('evidence symlinks are forbidden')
        if path.is_file() and path != manifest:
            actual_paths.add(path.relative_to(base).as_posix())
    if actual_paths != set(inventory):
        raise ValueError('cemetery evidence inventory differs')
    for name, expected_hash in inventory.items():
        path = base.joinpath(*relative(name).parts)
        if not re.fullmatch('[0-9a-f]{64}', expected_hash) or digest(path) != expected_hash:
            raise ValueError('cemetery payload hash differs: ' + name)
    cases = sorted((base / 'archive').glob('D*'))
    if len(cases) != expected['qualified_packages']:
        raise ValueError('cemetery package count differs')
    for case in cases:
        if not imports.QUALIFIED.fullmatch(case.name):
            raise ValueError('external validation package is not qualified')
        imports.package(case)
    return cases


def prepare(cemetery, receipts, output, historical=None, legacy=None):
    historical = historical or ROOT / 'docs/deaths'
    legacy = legacy or ROOT / 'docs/deaths/legacy-cases.json'
    if output.exists():
        raise ValueError('output exists; preserve it and choose a fresh path')
    # The legacy inventory and qualified-package checks remain unchanged.
    imports.check_registry(historical, legacy)
    external = []
    for receipt in receipts:
        external.extend(verify(cemetery, receipt))
    shutil.copytree(historical, output)
    for case in external:
        target = output / case.name
        if target.exists():
            if imports.package(target)[1] != imports.package(case)[1]:
                raise ValueError('conflicting cemetery package')
        else:
            shutil.copytree(case, target)
    count = imports.check_registry(output, legacy)
    print(f'{count} verified historical/cemetery packages in disposable reconciliation mirror')
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cemetery', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--receipt', type=Path, action='append', required=True)
    args = parser.parse_args()
    try:
        prepare(args.cemetery, [json.loads(path.read_text()) for path in args.receipt], args.output)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'validation-evidence: {error}\n')


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Verify the exact pinned R438 checkout, inventory and canonical identities."""
import argparse
import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path, PurePosixPath


def contained(root, raw):
    rel = PurePosixPath(raw)
    if rel.is_absolute() or '..' in rel.parts or str(rel) != raw or not rel.parts:
        raise ValueError('unsafe payload path')
    path = root.joinpath(*rel.parts)
    if any(root.joinpath(*rel.parts[:i]).is_symlink() for i in range(1, len(rel.parts) + 1)):
        raise ValueError('payload symlink escape')
    if not path.resolve().is_relative_to(root):
        raise ValueError('unsafe payload containment')
    return path


def verify_payload(root, manifest_sha256):
    root = Path(root).resolve()
    manifest = contained(root, 'manifest.json')
    if hashlib.sha256(manifest.read_bytes()).hexdigest() != manifest_sha256:
        raise ValueError('manifest hash differs')
    value = json.loads(manifest.read_text())
    if value['schema'] != 'borg-reborn-rs/r438-prepared-evidence@1':
        raise ValueError('unsupported manifest')
    seen = set()
    for row in value['files']:
        path = contained(root, row['path'])
        if row['path'] in seen:
            raise ValueError('duplicate payload path')
        seen.add(row['path'])
        if path.stat().st_size != row['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != row['sha256']:
            raise ValueError('payload differs: ' + row['path'])
    if 'qualified-originals.json' not in seen:
        raise ValueError('qualified originals index is not sealed by manifest')
    originals = json.loads((root / 'qualified-originals.json').read_text())
    if len(originals) != value['qualified_originals']:
        raise ValueError('qualified original count differs')
    packages = set()
    for entry in originals:
        package = contained(root, entry['qualified_package'])
        record = contained(root, entry['qualified_package'] + '/.source-record.json')
        for name in ['.source-record.json', 'record.json']:
            if str((package / name).relative_to(root)) not in seen:
                raise ValueError('qualified record is not sealed by manifest')
        if hashlib.sha256(record.read_bytes()).hexdigest() != entry['record_sha256']:
            raise ValueError('original record differs')
        packages.add(entry['qualified_package'])
    if len(packages) != len(originals):
        raise ValueError('duplicate qualified original')
    allowed = seen | {'manifest.json'} | {p + '/death-id.json' for p in packages}
    actual = set()
    for path in root.rglob('*'):
        if path.is_symlink():
            raise ValueError('payload symlink escape')
        if path.is_file():
            actual.add(str(path.relative_to(root)))
    if not seen <= actual or actual - allowed:
        raise ValueError('undeclared or missing payload files')
    return {'files': len(seen), 'qualified_originals': len(originals)}


def verify_checkout(cemetery, receipt):
    cemetery = Path(cemetery).resolve()
    actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=cemetery, text=True).strip()
    if actual != receipt['commit']:
        raise ValueError('cemetery checkout differs from immutable receipt')
    root = contained(cemetery, receipt['root'])
    scopes = [receipt['root'], 'deaths/ALIASES.json', 'scripts/native']
    dirty = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all', '--', *scopes], cwd=cemetery, text=True)
    if dirty.strip():
        raise ValueError('evidence working tree is not clean')
    result = verify_payload(root, receipt['manifest_sha256'])
    identity_path = contained(cemetery, 'scripts/native/death_identity.py')
    expected = subprocess.check_output(['git', 'show', receipt['commit'] + ':scripts/native/death_identity.py'], cwd=cemetery)
    if identity_path.read_bytes() != expected:
        raise ValueError('canonical identity implementation differs from pin')
    spec = importlib.util.spec_from_file_location('r438_canonical_identity', identity_path)
    identity = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(identity)
    aliases = json.loads(contained(cemetery, 'deaths/ALIASES.json').read_text())['records']
    prefix = receipt['root'] + '/'
    records = sorted((root / 'archive').glob('*/record.json'))
    expected_aliases = []
    for record in records:
        value = identity.ensure(record.parent, check=True)
        expected_aliases.append({'id': value['id'], 'aliases': value['aliases'],
                                 'path': str(record.parent.relative_to(cemetery)),
                                 'record_sha256': value['record_sha256']})
    relevant = [r for r in aliases if r['path'].startswith(prefix)]
    if sorted(relevant, key=lambda r: r['path']) != sorted(expected_aliases, key=lambda r: r['path']):
        raise ValueError('canonical alias inventory differs')
    if len(records) != receipt['qualified_originals'] or result['files'] != receipt['payload_files']:
        raise ValueError('receipt inventory differs')
    result['canonical_packages'] = len(records)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--cemetery', type=Path, required=True)
    parser.add_argument('--receipt', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(verify_checkout(args.cemetery, json.loads(args.receipt.read_text()))))

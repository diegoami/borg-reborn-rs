#!/usr/bin/env python3
"""Import immutable native validation packages without branch-local numeric IDs.

Exports and catalogue generation remain owned by borg-cemetery. This integration
step only packages an existing dossier for the historical CI archive. The full
original record and all payload bytes are retained; measurement independence is
still determined by encounter family and result hashes, never package counts.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
QUALIFIED = re.compile(r'D0000-q[0-9a-f]{64}')
RECEIPT = 'import.json'
ORIGINAL = '.source-record.json'
SCHEMA = 'borg-reborn-rs/dossier-import@1'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode()


def legacy_identity(record):
    attempts = record.get('attempts') or []
    return {
        'record_sha256': digest(encoded(record)),
        'identity_sha256': digest(encoded(record.get('identity'))),
        'encounter_family': record.get('encounter_family'),
        'first_result_sha256': attempts[0].get('result_sha256') if attempts else None,
        'id': record.get('id'),
        'files_manifest_sha256': digest(encoded(record.get('files_sha256', {}))),
    }


def package(source):
    """Return deterministic target bytes; never modify the source directory."""
    source = Path(source)
    if source.is_symlink() or not source.is_dir():
        raise ValueError(f'{source}: expected a real dossier directory')
    entries = list(source.iterdir())
    if any(p.is_symlink() or not p.is_file() for p in entries):
        raise ValueError(f'{source}: native validation packages must contain regular files only')
    qualified = QUALIFIED.fullmatch(source.name) is not None
    if qualified:
        receipt = json.loads((source / RECEIPT).read_bytes())
        if receipt.get('schema') != SCHEMA:
            raise ValueError(f'{source}: unsupported import receipt')
        raw_record = (source / ORIGINAL).read_bytes()
        names = {p.name for p in entries} - {'record.json', ORIGINAL, RECEIPT}
    else:
        if (source / RECEIPT).exists() or (source / ORIGINAL).exists():
            raise ValueError(f'{source}: reserved import metadata in unqualified package')
        raw_record = (source / 'record.json').read_bytes()
        names = {p.name for p in entries} - {'record.json'}
    original = json.loads(raw_record)
    if original.get('schema') != 'borg-reborn-rs/death-record@1':
        raise ValueError(f'{source}: only native harness dossiers are imported here')
    if not qualified and original.get('id') != source.name:
        raise ValueError(f'{source}: source record ID differs from directory name')
    required = {'assessment.json', 'build.json', 'config.txt', 'summary.json', 'report.md'}
    if not required <= names:
        raise ValueError(f'{source}: incomplete native dossier payload')
    if (not isinstance(original.get('identity'), dict)
            or not isinstance(original.get('terminal'), dict)
            or not isinstance(original.get('files_sha256'), dict)
            or not isinstance(original.get('encounter_family'), str)
            or not isinstance(original.get('attempts'), list)
            or not original['attempts']
            or any(not isinstance(a, dict) or not {'outcome', 'result_sha256', 'source_commit'} <= set(a)
                   for a in original['attempts'])):
        raise ValueError(f'{source}: malformed native dossier record')
    payload = {name: (source / name).read_bytes() for name in sorted(names)}
    for name, expected in original.get('files_sha256', {}).items():
        if name not in payload or digest(payload[name]) != expected:
            raise ValueError(f'{source}: source file hash mismatch: {name}')
    descriptor = {
        'schema': SCHEMA,
        'original_record_sha256': digest(raw_record),
        'payload_sha256': {name: digest(data) for name, data in payload.items()},
    }
    identifier = 'D0000-q' + digest(encoded(descriptor))
    canonical = dict(original, id=identifier)
    receipt = dict(descriptor, id=identifier, original_id=original['id'])
    files = dict(payload, **{'record.json': encoded(canonical), ORIGINAL: raw_record,
                            RECEIPT: encoded(receipt)})
    if qualified:
        if source.name != identifier or {p.name for p in entries} != set(files):
            raise ValueError(f'{source}: qualified ID or file inventory changed')
        if any((source / name).read_bytes() != data for name, data in files.items()):
            raise ValueError(f'{source}: qualified package was modified')
    return identifier, files


def registry_module():
    spec = importlib.util.spec_from_file_location('import_registry', ROOT / 'tools/death_registry.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def import_packages(sources, destination):
    from cemetery_tools import locate
    destination = Path(destination).resolve()
    if destination == (locate(ROOT) / 'deaths').resolve():
        raise ValueError('The canonical cemetery is owned by its registrar; use that delivery workflow')
    prepared = [(Path(source), package(source)) for source in sources]
    owner = registry_module()
    result = []
    with owner.registry_lock(destination):
        for source, (identifier, files) in prepared:
            final = destination / identifier
            if final.exists():
                if final.is_symlink() or {p.name for p in final.iterdir()} != set(files):
                    raise ValueError(f'{final}: existing package differs; nothing is overwritten')
                if any(not (final / name).is_file() or (final / name).is_symlink()
                       or (final / name).read_bytes() != data for name, data in files.items()):
                    raise ValueError(f'{final}: existing package differs; nothing is overwritten')
                result.append((identifier, 'already retained'))
                continue
            stage = Path(tempfile.mkdtemp(prefix='.import-', dir=destination))
            try:
                for name, data in files.items():
                    (stage / name).write_bytes(data)
                # Keep staging private while incomplete, then preserve source access.
                # Special permission bits are not part of an evidence package.
                stage.chmod(source.stat().st_mode & 0o777)
                stage.rename(final)
            finally:
                if stage.exists():
                    shutil.rmtree(stage)
            result.append((identifier, 'imported'))
        # Formatting belongs to the cemetery-owned implementation, not this tool.
        owner.update_index(destination)
    return result


def check_registry(root, legacy):
    root = Path(root)
    known = json.loads(Path(legacy).read_text())
    if known.get('schema') != 'borg-reborn-rs/legacy-dossiers@1':
        raise ValueError('unsupported legacy inventory')
    cases = {p.name: p for p in root.glob('D*') if p.is_dir()}
    missing = set(known['cases']) - set(cases)
    if missing:
        raise ValueError('historical cases disappeared: ' + ', '.join(sorted(missing)))
    for name, case in sorted(cases.items()):
        if case.is_symlink():
            raise ValueError(f'{case}: dossier symlinks are not permitted')
        if name in known['cases']:
            record = json.loads((case / 'record.json').read_bytes())
            if legacy_identity(record) != known['cases'][name]:
                raise ValueError(f'{case}: historical identity changed')
            for filename, expected in record.get('files_sha256', {}).items():
                asset = case / filename
                if not asset.is_file() or asset.is_symlink() or digest(asset.read_bytes()) != expected:
                    raise ValueError(f'{case}: historical file hash changed: {filename}')
        elif QUALIFIED.fullmatch(name):
            package(case)
        else:
            raise ValueError(f'{case}: provisional numeric ID; import this dossier with tools/import-deaths.py')
    return len(cases)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('sources', type=Path, nargs='*')
    ap.add_argument('--registry', type=Path, default=ROOT / 'docs/deaths')
    ap.add_argument('--check', action='store_true')
    ap.add_argument('--legacy', type=Path, default=ROOT / 'docs/deaths/legacy-cases.json')
    args = ap.parse_args()
    try:
        if args.check:
            if args.sources:
                ap.error('--check takes no sources')
            print(f'{check_registry(args.registry, args.legacy)} historical/qualified dossiers verified')
        else:
            if not args.sources:
                ap.error('provide one or more native dossier directories')
            for identifier, status in import_packages(args.sources, args.registry):
                print(f'{identifier}: {status}')
    except (OSError, ValueError, KeyError) as error:
        ap.exit(2, f'import-deaths: {error}\n')


if __name__ == '__main__':
    main()

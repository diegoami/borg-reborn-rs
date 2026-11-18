#!/usr/bin/env python3
"""Check the isolated accepted P4 carry offline, without an external checkout."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MODULES = tuple('borg-core/src/' + name + '.rs' for name in (
    'borg_power', 'borg_item_wear', 'borg_store_buy', 'borg_store_sell',
    'borg_think_store', 'borg_home_power'))
SPEC = re.compile(r'^\s*(\w+)\s*=>\s*(\(.*\)),\s*$', re.M)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def encoded(value):
    return json.dumps(value, separators=(',', ':')).encode()


def verify(root):
    root = Path(root)
    metadata = json.loads((root / 'docs/R528-CARRY.json').read_text())
    if metadata.get('schema') != 'borg-reborn-rs/r528-p4-carry@1':
        raise ValueError('unsupported P4 carry metadata')
    parameters = (root / 'borg-core/src/params.rs').read_text()
    rows = SPEC.findall(parameters)
    count = metadata['receiving_original_registry_count']
    if len(rows) != metadata['receiving_registry_count']:
        raise ValueError('receiving registry count changed')
    if digest(encoded(rows[:count])) != metadata['receiving_original_specs_sha256']:
        raise ValueError('an existing receiving spec or ID changed')
    remainder = SPEC.sub('', parameters).encode()
    if digest(remainder) != metadata['receiving_non_spec_params_sha256']:
        raise ValueError('non-spec parameter validation/test code changed')
    added = rows[count:]
    if len(added) != metadata['p4_new_count'] or digest(encoded(added)) != metadata['accepted_p4_specs_sha256']:
        raise ValueError('accepted P4 definitions/order changed')
    first = metadata['accepted_registry_count'] - len(added)
    expected = [{'name': name, 'accepted_id': first + i, 'receiving_id': count + i}
                for i, (name, _) in enumerate(added)]
    if metadata['parameter_id_mapping'] != expected:
        raise ValueError('accepted-to-receiving ID mapping changed')
    contract = root / 'docs/M13-INVENTORY-PARAMETERS.json'
    if digest(contract.read_bytes()) != metadata['accepted_contract_sha256']:
        raise ValueError('accepted contract bytes changed')
    names = {p['name'] for p in json.loads(contract.read_text())['parameters']}
    if names != {name for name, _ in added}:
        raise ValueError('accepted contract parameter set differs')
    patch = root / 'docs/R528-P4-SITES.patch'
    if digest(patch.read_bytes()) != metadata['p4_patch_sha256']:
        raise ValueError('accepted neutral-site patch changed')
    if set(metadata['receiving_original_module_sha256']) != set(MODULES):
        raise ValueError('unexpected carried module inventory')
    with tempfile.TemporaryDirectory(prefix='p4-carry-source-') as temporary:
        target = Path(temporary)
        for name in MODULES:
            source = root / name
            if source.is_symlink():
                raise ValueError('carried source must be a regular file')
            destination = target / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
        result = subprocess.run(['git', 'apply', '--reverse', str(patch.resolve())],
                                cwd=target, capture_output=True, text=True)
        if result.returncode:
            raise ValueError('carried source cannot reverse the accepted neutral-site patch')
        for name, expected_hash in metadata['receiving_original_module_sha256'].items():
            if digest((target / name).read_bytes()) != expected_hash:
                raise ValueError('source includes changes beyond accepted P4 delta: ' + name)
    return len(added)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        print(f'P4 carry: {verify(args.root)} accepted definitions; existing registry and source delta verified')
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'P4 carry: {error}\n')


if __name__ == '__main__':
    main()

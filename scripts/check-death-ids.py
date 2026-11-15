#!/usr/bin/env python3
"""Consumer archive integrity check; never allocates or rewrites death IDs."""
import argparse
import json
from pathlib import Path
import re


def number(label):
    if not isinstance(label, str):
        raise ValueError(f'invalid dossier label type: {type(label).__name__}')
    match = re.fullmatch(r'D([0-9]+)(?:-[^/]+)?', label)
    if not match:
        raise ValueError(f'invalid dossier label: {label!r}')
    return int(match[1])


def check(registry):
    registry = Path(registry)
    if not registry.is_dir():
        raise ValueError(f'missing registry: {registry}')
    seen = {}
    records = sorted(registry.glob('D*/record.json'))
    if not records:
        raise ValueError(f'no dossier records: {registry}')
    for path in records:
        label = path.parent.name
        numeric = number(label)
        record = json.loads(path.read_text())
        # Legacy D0526-recorded has record id D0526. Compare numeric IDs,
        # retaining that original record rather than rewriting history.
        if number(record.get('id', '')) != numeric:
            raise ValueError(f'{path}: folder and record numeric IDs differ')
        if numeric in seen:
            raise ValueError(f'duplicate numeric dossier D{numeric:04d}: '
                             f'{seen[numeric]} and {label}; import needs a preserved mapping')
        seen[numeric] = label
    return len(records)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('registry', type=Path, nargs='?', default=Path('docs/deaths'))
    args = parser.parse_args()
    try:
        count = check(args.registry)
    except (ValueError, OSError) as error:
        parser.exit(1, f'dossier-ID check failed: {error}\n')
    print(f'{count} local dossier records have unique numeric IDs; '
          'cross-branch references still require an exact registry commit')


if __name__ == '__main__':
    main()

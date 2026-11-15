#!/usr/bin/env python3
"""Require real shadow comparisons, native boundary coverage and zero mismatches.

Usage: harness/check-shadow.py RUN_OR_SUITE [RUN_OR_SUITE ...]
An empty output directory must not be mistaken for a successful shadow run.
"""
import argparse
import json
from pathlib import Path


def check(roots):
    files = sorted({p for root in roots for p in Path(root).rglob('shadow-stats.jsonl')})
    if not files:
        raise ValueError('no shadow statistics found')
    totals = {'comparisons': 0, 'danger_calls': 0, 'caution_calls': 0}
    for path in files:
        previous = 0
        last = None
        for line in path.read_text().splitlines():
            row = json.loads(line)
            if row['schema'] != 'borg-reborn-rs/shadow-stats@1' or row['mode'] not in ('check', 'switch'):
                raise ValueError(f'{path}: wrong schema or mode')
            if row['counts']['thinks'] != previous + 1:
                raise ValueError(f'{path}: missing think statistics')
            previous = row['counts']['thinks']
            last = row
        if last is None:
            raise ValueError(f'{path}: empty statistics')
        if last['ported_points'] != ['danger.wall_like']:
            raise ValueError(f'{path}: unexpected port coverage')
        if last['counts']['mismatches']:
            raise ValueError(f'{path}: reported shadow mismatches')
        mismatch = path.with_name('shadow.jsonl')
        if not mismatch.exists() or mismatch.read_bytes():
            raise ValueError(f'{mismatch}: absent log or nonzero mismatches')
        for name in totals:
            totals[name] += last['counts'][name]
    if not all(totals.values()):
        raise ValueError(f'insufficient comparison/boundary coverage: {totals}')
    return len(files), totals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('roots', nargs='+', type=Path)
    args = parser.parse_args()
    try:
        count, totals = check(args.roots)
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f'shadow check failed: {error}\n')
    print(f'{count} shadow runs: {totals}; zero mismatches')


if __name__ == '__main__':
    main()

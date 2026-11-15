#!/usr/bin/env python3
"""Require real shadow comparisons, native boundary coverage and zero mismatches.

Usage: harness/check-shadow.py RUN_OR_SUITE [RUN_OR_SUITE ...]
An empty output directory must not be mistaken for a successful shadow run.
"""
import argparse
import json
from pathlib import Path


ENTRY_POINTS = (
    'caution.borg_caution', 'escape.borg_recall', 'escape.borg_surrounded',
    'escape.borg_freedom', 'escape.borg_caution_phase',
    'escape.borg_caution_teleport', 'escape.borg_allow_teleport',
    'escape.borg_shadow_shift', 'escape.borg_dimension_door',
    'escape.borg_escape', 'recovery.borg_recover',
)


def check(roots, require_all_entries=False):
    files = sorted({p for root in roots for p in Path(root).rglob('shadow-stats.jsonl')})
    if not files:
        raise ValueError('no shadow statistics found')
    totals = {'comparisons': 0, 'danger_calls': 0, 'caution_calls': 0, 'danger_comparisons': 0, 'monster_comparisons': 0, 'wall_comparisons': 0, 'stateful_comparisons':0, 'escape_calls':0, 'recovery_calls':0}
    entry_counts = {name: 0 for name in ENTRY_POINTS}
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
        if last['ported_points'] != ['danger.wall_like', 'danger', 'danger.one_monster', 'caution', 'escape', 'recovery']:
            raise ValueError(f'{path}: unexpected port coverage')
        if last['unported_points'] != []:
            raise ValueError(f'{path}: unexpected unported points')
        counts = last['counts']
        if counts['danger_comparisons'] != counts['danger_calls']:
            raise ValueError(f'{path}: danger calls were observed without comparisons')
        if counts['stateful_comparisons'] > sum(counts[k] for k in ('caution_calls','escape_calls','recovery_calls')):
            raise ValueError(f'{path}: stateful calls were observed without comparisons')
        if counts['comparisons'] != sum(counts[k] for k in ('danger_comparisons', 'monster_comparisons', 'wall_comparisons', 'stateful_comparisons')):
            raise ValueError(f'{path}: inconsistent comparison counts')
        if last['counts']['mismatches']:
            raise ValueError(f'{path}: reported shadow mismatches')
        mismatch = path.with_name('shadow.jsonl')
        if not mismatch.exists() or mismatch.read_bytes():
            raise ValueError(f'{mismatch}: absent log or nonzero mismatches')
        entries = counts.get('stateful_points', {})
        for name, hits in entries.items():
            if name not in entry_counts or type(hits) is not int or hits < 0:
                raise ValueError(f'{path}: invalid entry coverage {name}: {hits}')
            entry_counts[name] += hits
        if require_all_entries and sum(entries.values()) != sum(counts[k] for k in ('caution_calls', 'escape_calls', 'recovery_calls')):
            raise ValueError(f'{path}: inconsistent individual entry coverage')
        for name in totals:
            totals[name] += last['counts'][name]
    if not all(totals.values()):
        raise ValueError(f'insufficient comparison/boundary coverage: {totals}')
    if require_all_entries:
        missing = [name for name, hits in entry_counts.items() if hits == 0]
        if missing:
            raise ValueError(f'missing individual entry coverage: {missing}')
    totals['stateful_points'] = entry_counts
    return len(files), totals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('roots', nargs='+', type=Path)
    parser.add_argument('--require-all-entries', action='store_true',
                        help='require a positive hit count for all 11 public stateful entry points across these runs (not branch coverage)')
    args = parser.parse_args()
    try:
        count, totals = check(args.roots, args.require_all_entries)
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f'shadow check failed: {error}\n')
    print(f'{count} shadow runs: {totals}; zero mismatches')


if __name__ == '__main__':
    main()

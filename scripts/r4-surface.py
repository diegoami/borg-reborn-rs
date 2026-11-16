#!/usr/bin/env python3
"""Reproduce R4's source inventory; lexical reach is not a semantic audit.

No game, external checkout, Git operation or parameter reclassification runs.
The JSON locks the receiving files as read during #431, including test code.
Use --write deliberately when a reviewed R4 slice changes this inventory.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

from rust_ports import functions, mask_non_code

ROOT = Path(__file__).resolve().parent.parent
PRIMARY = (
    'power', 'item_wear', 'item_use', 'item_activation', 'store',
    'store_buy', 'store_sell', 'think_store', 'home_power',
)
ADJACENT = ('item_val', 'item_analyze', 'item_id', 'item_decurse', 'item_enchant')
# Explicitly shared, not silently counted as an independently ported R4 body.
SHARED = (
    'item', 'inventory', 'home_notice', 'formulas', 'buy_strategy', 'trip',
    'town_budget', 'town_capability', 'town_dispose', 'town_dump', 'town_live',
    'town_packing', 'town_record', 'town_shadow', 'town_solve', 'town_stock',
    'town_traits', 'town_usefulness', 'town_utility',
)


def inventory():
    registry_path = ROOT / 'borg-core/src/params.rs'
    registry = registry_path.read_bytes()
    declared = set(re.findall(r'\b(\w+)\s*=>\s*\(', mask_non_code(registry.decode())))
    rows = []
    for role, names in [('primary', PRIMARY), ('adjacent', ADJACENT), ('shared', SHARED)]:
        for name in names:
            path = Path(f'borg-core/src/borg_{name}.rs')
            raw = (ROOT / path).read_bytes()
            text = raw.decode()
            code = mask_non_code(text)
            params = sorted(set(re.findall(r'\bParam::(\w+)', code)))
            missing = set(params) - declared
            if missing:
                raise ValueError(f'{path}: undeclared parameters: {sorted(missing)}')
            defs = functions(text)
            rows.append({
                'path': str(path), 'role': role,
                'sha256': hashlib.sha256(raw).hexdigest(),
                'physical_lines_including_tests': len(text.splitlines()),
                'function_definitions_including_tests': sum(map(len, defs.values())),
                'public_function_spellings_including_methods': re.findall(
                    r'\bpub(?:\([^)]*\))?\s+fn\s+(\w+)', code),
                'named_parameters': params,
                'direct_crate_namespaces': sorted(set(re.findall(r'\bcrate::(\w+)', code))),
                'direct_engine_symbol_spellings': sorted(set(re.findall(r'\bengine::(\w+)', code))),
                'direct_state_group_spellings': sorted(set(re.findall(r'\bst\.(\w+)', code))),
                'named_trait_spellings': sorted(set(re.findall(r'\bBI_[A-Z_0-9]+', code))),
            })
    return {
        'schema': 'borg-reborn-rs/r4-source-inventory@1',
        'runtime_donor': (ROOT / 'borg-core/PIN').read_text().strip(),
        'accepted_r3_source': '02c273db1938ab61e18b22458d229155a0204164',
        'parameter_registry_sha256': hashlib.sha256(registry).hexdigest(),
        'limits': 'Lexical spellings, including cfg(test) bodies/methods; aliases, macros and transitive calls require manual review. No coverage, independent-policy, parameter-completeness or performance claim.',
        'totals': {role: {
            'files': sum(r['role'] == role for r in rows),
            'physical_lines_including_tests': sum(r['physical_lines_including_tests'] for r in rows if r['role'] == role),
            'function_definitions_including_tests': sum(r['function_definitions_including_tests'] for r in rows if r['role'] == role),
        } for role in ('primary', 'adjacent', 'shared')},
        'modules': rows,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='explicitly replace the source audit')
    args = parser.parse_args()
    target = ROOT / 'docs/R4-SOURCE-INVENTORY.json'
    expected = json.dumps(inventory(), indent=2) + '\n'
    if args.write:
        target.write_text(expected)
    elif target.read_text() != expected:
        parser.exit(1, 'R4 source inventory differs; audit the source delta before an explicit --write\n')
    print('R4 source inventory: ' + ('recorded' if args.write else 'verified'))


if __name__ == '__main__':
    main()

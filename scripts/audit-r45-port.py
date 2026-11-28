#!/usr/bin/env python3
"""Verify the receiving owned function bodies and explicitly reviewed rewrites."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

from rust_ports import functions, mask_non_code, without_test_items

ROOT = Path(__file__).resolve().parents[1]
TARGET = '24be98732748230062882fe2b7d9601dba5e2cc0'
RECEIPT = ROOT / 'docs/R45-PORT-SOURCE.json'
# Pin the R5 flow policy and its updated dependencies alongside the R4.5 receipt.
R5_RECEIVING_FILES = {
    'borg-core/src/borg_caution.rs': '4d0de570382d361e2b82a676949e609345150f7f61fd73b11f2e047f8a3404b5',
    'borg-core/src/decision_api.rs': '477b2a8058689909a1202b360d580a72b14761eed7516b22472945e488adfe76',
    'borg-core/src/decisions.rs': 'adf325ff40696294e2574077d6538e88eedf3091364e5d8778f5c75a14bf12fa',
    'borg-core/src/memory.rs': 'd147735a90b4016613343659e3a2250aa93140a43d017f81c470669082d892fe',
    'borg-next/src/stateful/caution.rs': '1c2308a13e1db61ecd5f7061638dc19d68e4839091e53a6f7f8d721d404605c4',
    'borg-next/src/stateful/flow.rs': '1eea14c8a71ca3e2c8e43befdba3a04b68227fdfca9631be19a99c061cd693fb',
    'borg-next/src/stateful/mod.rs': 'f1157e0f03e9a74ee5ba8cf134bff7033f6a136a5199b66436d75646adf86fbe',
}
MODULES = {
    'caution': 'borg_caution', 'escape': 'borg_escape', 'recovery': 'borg_recover',
    'attack': 'borg_fight_attack', 'fast_fragile': 'borg_fast_fragile',
    'attack_preference': 'attack_preference', 'defence': 'borg_fight_defend',
    'item_use': 'borg_item_use', 'power': 'borg_power', 'equipment_wear': 'borg_item_wear',
    'equipment_identify': 'borg_item_id', 'home_power': 'borg_home_power',
    'store_buy': 'borg_store_buy', 'store_sell': 'borg_store_sell', 'think_store': 'borg_think_store',
}
REWRITES = {
    'typed values are read in array order before the predicate': (
        'caution.stats_needing_fix', 'caution.winning_fight', 'caution.borg_caution',
        'recovery.holding_for_monster_ago', 'recovery.impaired',
    ),
    'remove same-type conversion or redundant reference': (
        'caution.desperate_flow', 'caution.immobile_attacker', 'caution.kept_attacker_next_to',
        'caution.immobile_attacker_next_to', 'caution.mobile_danger_at',
        'caution.immobile_step_away', 'caution.fight_incoming_with', 'attack.dangerous_town_race',
    ),
    'iterate the same eight i32 stair scores in index order': ('caution.best_step_away',),
    'closure formatting or let-else terminators': (
        'caution.listed_ranged', 'caution.flight_recall_survivable', 'caution.recall_escape_allowed',
        'caution.throw_first', 'caution.leave_stairs_on_monster', 'caution.nearest_mover_seen',
        'caution.unseen_bump_blocked', 'caution.unseen_bump_response', 'escape.fight_lost',
        'escape.teleport_curse', 'attack.ignored', 'attack.attack_on_sight',
        'fast_fragile.named_in_list', 'fast_fragile.has_cure',
    ),
    'named stat offsets and spell read projection': ('power.borg_power_equipment', 'power.borg_power_inventory'),
    'already imported item-id and inventory seam paths': ('equipment_wear.borg_item_worth_id',),
}
LITERALS = re.compile(
    r'(?:[bc])?"(?:[^"\\]|\\.)*"'
    r"|'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'"
)
MARKER = re.compile(
    r'if\s+(?:borg_rs|crate)::decision_tape::active\(\)\s*\{\s*'
    r'(?:borg_rs|crate)::decision_tape::mark\([^;]+;\s*\}'
)
OPERATORS = frozenset(('&&', '||', '<<', '>>', '<=', '>=', '==', '!=', '+=', '-=',
                       '*=', '/=', '%=', '&=', '|=', '^=', '->', '=>', '::', '..'))


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def trait_projections(source):
    return {
        (field, name): index
        for field, body in re.findall(r'trait_group!\(\w+,\s*\w+,\s*(\w+)\s*\{(.*?)\}\);', source, re.S)
        for name, index in re.findall(r'(\w+)\s*=>\s*(BI_\w+)', body)
    }


def normalize(body, traits):
    body = MARKER.sub('', body)
    literals = LITERALS.findall(body)
    code = mask_non_code(body)
    code = re.sub(r'(?<!\.)\btr\(\s*(\w+)\s*,\s*(BI_\w+)\s*\)', r'\1.borg.tr(\2)', code)
    code = re.sub(r'(?<!\.)\b(?:b\.tr|t|tr)\((BI_\w+)\)', r'st.borg.tr(\1)', code)
    code = re.sub(r'\bpub(?:\(crate\))?\s+', '', code)
    code = re.sub(
        r'([&|<>=!+*/%:^.\-])\s+(?=([&|<>=!+*/%:^.\-]))',
        lambda m: m[1] + ('__GAP__' if m[1] + m[2] in OPERATORS else ''), code,
    )
    code = re.sub(r'\s+', '', code)
    for name, native in MODULES.items():
        code = code.replace(f'crate::stateful::{name}::', f'crate::{native}::')
    code = code.replace('borg_rs::decision_api::', 'crate::decision_api::')
    for alias, native in {
        'r45_attack_types': 'attack_preference', 'r45_bump_types': 'borg_caution',
        'phase_types': 'borg_escape', 'headers': 'headers', 'borg_headers': 'borg_headers',
        'borg': 'borg', 'borg_cave': 'borg_cave', 'borg_update': 'borg_update',
    }.items():
        code = code.replace(f'crate::decision_api::{alias}::', f'crate::{native}::')
    code = code.replace('crate::decision_api::attack_types::Family', 'crate::attack_preference::Family')
    code = code.replace('borg_rs::threat_facts::param', 'crate::params::value')
    code = code.replace('borg_rs::threat_facts::Param', 'crate::params::Param')
    code = code.replace('crate::state::BorgState', 'BorgState').replace('resolved_param', 'params::resolved')
    code = re.sub(
        r'crate::Read::new\((\w+)\)\.(\w+)\(\)\.(\w+)\(\)',
        lambda m: f'{m[1]}.borg.tr({traits[m[2], m[3]]})' if (m[2], m[3]) in traits else m[0],
        code,
    )
    code = re.sub(r'(?<!\.)\btr\((\w+),((?:crate::borg_headers::borg_trait_h::)?BI_\w+)\)',
                  r'\1.borg.tr(\2)', code)
    code = code.replace('crate::borg_headers::borg_trait_h::BI_', 'BI_')
    code = re.sub(r'letsimulated_mana=([^;]+);super::set_mana\((\w+),simulated_mana\);',
                  r'\2.borg.trait_[BI_CURSPasusize]=\1;', code)
    for index, setter in {'BI_CURSP': 'mana', 'BI_RCONF': 'confusion_resistance',
                          'BI_FRACT': 'free_action'}.items():
        code = re.sub(fr'super::set_{setter}\((\w+),([^;]+)\);',
                      rf'\1.borg.trait_[{index}asusize]=\2;', code)
    code = re.sub(r'\b(\w+)_native\(', r'\1(', code)
    code = re.sub(r'let(?:tr|t)=\|i(?::u32)?\|st.borg.tr\(i\);', '', code)
    code = re.sub(r',+(?=[)\]}])', '', code)
    code = re.sub(r'=>\{([^{};]+)\}', r'=>\1,', code)
    code = re.sub(r',(?=(?:[A-Z_][A-Z_0-9]*|_)=>)', '', code)
    return code, literals


def measure(root, *, receiving_r5=False):
    projections = trait_projections((root / 'borg-next/src/lib.rs').read_text())
    rewrites = {name: reason for reason, names in REWRITES.items() for name in names}
    active = {}
    pinned = {}
    modules = {}

    def read_production(path, native=False):
        source = (root / path).read_text()
        active[path] = digest(source)
        if native:
            original = subprocess.check_output(
                ['git', 'show', f'{TARGET}:{path}'], cwd=root, text=True)
            pinned[path] = digest(original)
        return functions(without_test_items(source))

    inventory = read_production('borg-core/src/borg_inventory.rs', native=True)
    for port, native in MODULES.items():
        core = read_production(f'borg-core/src/{native}.rs', native=True)
        owned = read_production(f'borg-next/src/stateful/{port}.rs')
        if port == 'equipment_wear':
            core['borg_item_worth_id'] = inventory['borg_item_worth_id']
        paired = {}
        for name, bodies in sorted(owned.items()):
            native_name = name + '_native' if name + '_native' in core else name
            before = core.get(native_name, ())
            if len(before) != 1 or len(bodies) != 1:
                raise ValueError(f'{port}.{name}: expected one body in each arm')
            left, left_literals = normalize(before[0], projections)
            right, right_literals = normalize(bodies[0], projections)
            qualified = f'{port}.{name}'
            if left_literals != right_literals:
                raise ValueError(f'{qualified}: quoted literals differ')
            row = {'native': native_name}
            if left == right:
                if qualified in rewrites:
                    raise ValueError(f'{qualified}: remove stale rewrite')
                row['normalized_sha256'] = digest(left)
            elif qualified in rewrites:
                row.update(rewrite=rewrites.pop(qualified), native_sha256=digest(left), owned_sha256=digest(right))
            else:
                raise ValueError(f'{qualified}: normalized bodies differ')
            paired[name] = row
        owned_native = {row['native'] for row in paired.values()}
        modules[port] = {'paired': paired, 'native_only': sorted(set(core) - owned_native)}

    if rewrites:
        raise ValueError(f'unknown reviewed rewrites: {sorted(rewrites)}')
    for path in ('borg-next/src/lib.rs', 'borg-next/src/stateful/mod.rs', 'borg-core/src/decision_api.rs',
                 'borg-core/src/decisions.rs', 'borg-core/src/threat_facts.rs'):
        active[path] = digest((root / path).read_text())
    if receiving_r5:
        for path in R5_RECEIVING_FILES:
            active[path] = hashlib.sha256((root / path).read_bytes()).hexdigest()
    return {'schema': 'borg-reborn-rs/r45-port-source@1', 'receiving_main': TARGET,
            'pinned_native_files': pinned, 'receiving_files': active, 'modules': modules}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true', help='record after reviewing receiving source changes')
    mode.add_argument('--receiving-r5', action='store_true',
                      help='verify the reviewed R5 files alongside the R4.5 receipt')
    args = parser.parse_args()
    actual = measure(ROOT, receiving_r5=args.receiving_r5)
    if args.write:
        RECEIPT.write_text(json.dumps(actual, indent=2) + '\n')
    else:
        expected = json.loads(RECEIPT.read_text())
        if args.receiving_r5:
            expected['receiving_files'].update(R5_RECEIVING_FILES)
        if actual != expected:
            raise SystemExit('receiving proof differs from its recorded source or reviewed rewrites')
    counts = Counter('reviewed rewrite' if 'rewrite' in row else 'normalized match'
                     for module in actual['modules'].values() for row in module['paired'].values())
    print(json.dumps(counts, sort_keys=True))


if __name__ == '__main__':
    main()

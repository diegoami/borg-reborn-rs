#!/usr/bin/env python3
"""Require non-vacuous, internally consistent shadow evidence and zero mismatches.

R5 uses @10 with --r5 for committed-flow direction selection. The pinned R4.5
receiving policy still uses @9 with --r45.
R4 item-use uses @5 with --r4-item-use; --legacy-r4-item-use reads immutable @4. The immutable r3-attack-defence reader is @3. --legacy-r3-attack reads @2. --legacy-r2 explicitly reads immutable
@1 evidence: its dependency_requests field means total tape events. Returned
legacy totals derive the marker/request split without changing any raw file.
"""
import argparse
import json
from pathlib import Path

R2_ENTRY_POINTS = (
    'caution.borg_caution', 'escape.borg_recall', 'escape.borg_surrounded',
    'escape.borg_freedom', 'escape.borg_caution_phase',
    'escape.borg_caution_teleport', 'escape.borg_allow_teleport',
    'escape.borg_shadow_shift', 'escape.borg_dimension_door',
    'escape.borg_escape', 'recovery.borg_recover',
)
ATTACK_ENTRY_POINTS = (
    'attack.borg_mon_blow_effect', 'attack.borg_thrust_damage_one',
    'attack.borg_launch_bolt', 'attack.borg_attack_aux_launch',
    'attack.borg_attack_aux_spell_bolt',
    'attack.borg_calculate_attack_effectiveness', 'attack.borg_attack',
)
PURE_POINTS = ATTACK_ENTRY_POINTS[:2]
ATTACK_SURFACE = R2_ENTRY_POINTS + ATTACK_ENTRY_POINTS
ENTRY_POINTS = ATTACK_SURFACE + ('defence.borg_defend',)
ITEM_USE_ENTRY_POINTS = ('item_use.borg_quaff_crit', 'item_use.borg_quaff_potion', 'item_use.borg_quaff_unknown', 'item_use.borg_read_scroll', 'item_use.borg_read_unknown', 'item_use.borg_eat', 'item_use.borg_eat_unknown', 'item_use.borg_eat_food_any', 'item_use.borg_equips_rod', 'item_use.borg_zap_rod', 'item_use.borg_use_staff', 'item_use.borg_use_unknown', 'item_use.borg_use_staff_fail', 'item_use.borg_equips_staff_fail', 'item_use.borg_aim_wand', 'item_use.borg_equips_ring', 'item_use.borg_activate_ring', 'item_use.borg_equips_dragon', 'item_use.borg_activate_dragon', 'item_use.borg_activate_item', 'item_use.borg_equips_item', 'item_use.borg_activate_failure', 'item_use.borg_use_things', 'item_use.borg_recharging')
R4_ENTRY_POINTS = ENTRY_POINTS + ITEM_USE_ENTRY_POINTS
STORE_ENTRY_POINTS = ('store.borg_power_home', 'store.borg_think_shop_buy_useful', 'store.borg_think_home_buy_useful', 'store.borg_think_shop_grab_interesting', 'store.borg_think_home_grab_useless', 'store.borg_think_home_buy_swap_weapon', 'store.borg_think_home_buy_swap_armour', 'store.borg_think_shop_buy', 'store.take_selected_for_sale', 'store.confirm_purchase', 'store.borg_min_item_quantity', 'store.borg_think_home_sell_useful', 'store.borg_think_shop_sell_useless', 'store.borg_think_shop_sell', 'store.sell_one_selected', 'store.borg_count_sell', 'store.borg_choose_shop', 'store.borg_think_store', 'store.town_strategy_disabled')
R45_RETIRED = frozenset((
    'escape.borg_caution_phase', 'item_use.borg_quaff_unknown', 'item_use.borg_read_unknown',
    'item_use.borg_eat_unknown', 'item_use.borg_use_unknown', 'store.sell_one_selected',
))
R45_ADDED = (
    'caution.fight_before_flight', 'caution.recall_wait_no_explore', 'caution.lightless_self_care',
    'attack.attack_on_sight', 'recovery.holding_for_monster', 'store.sell_selected_quantity',
)
FLOW_STEP_POINT = 'flow.select_step'
R2_PORTED = ['danger.wall_like', 'danger', 'danger.one_monster', 'caution', 'escape', 'recovery']
NUMERIC = ('comparisons', 'danger_calls', 'danger_comparisons',
           'monster_comparisons', 'wall_comparisons', 'stateful_comparisons',
           'caution_calls', 'escape_calls', 'recovery_calls')


def checked_map(value, allowed, path):
    if not isinstance(value, dict) or any(k not in allowed or type(v) is not int or v < 0
                                          for k, v in value.items()):
        raise ValueError(f'{path}: invalid entry coverage')
    return value


def check(roots, require_all_entries=False, legacy_r2=False, legacy_attack=False, expected_mode=None, r4_item_use=False, legacy_item_use=False, r4_power=False, r4_equipment=False, r4_store=False, constructed_roots=(), r45=False, r5=False):
    if expected_mode not in (None, 'check', 'switch'):
        raise ValueError('expected mode must be check or switch')
    if legacy_r2 and legacy_attack:
        raise ValueError('select only one legacy surface')
    if sum(map(bool, (legacy_r2, legacy_attack, r4_item_use, legacy_item_use, r4_power, r4_equipment, r4_store, r45, r5))) > 1:
        raise ValueError("select only one shadow surface")
    receiving_surface = r45 or r5
    store_surface = r4_store or receiving_surface
    equipment_surface = r4_equipment or store_surface
    power_surface = r4_power or equipment_surface
    r4_item_use = r4_item_use or power_surface
    item_surface = r4_item_use or legacy_item_use
    if item_surface and (legacy_r2 or legacy_attack):
        raise ValueError("R4 cannot use a legacy surface")
    complete = not (legacy_r2 or legacy_attack)
    files = sorted({p for root in roots for p in Path(root).rglob('shadow-stats.jsonl')})
    if not files:
        raise ValueError('no shadow statistics found')
    declared_constructed = {p.resolve() for root in constructed_roots for p in Path(root).rglob('shadow-stats.jsonl')}
    if constructed_roots and (not declared_constructed or not declared_constructed <= {p.resolve() for p in files}):
        raise ValueError('constructed roots must contain statistics included in the checked roots')
    run_coverage = {}
    schema = 'borg-reborn-rs/shadow-stats@' + ('10' if r5 else '9' if r45 else '8' if store_surface else '7' if equipment_surface else '6' if power_surface else '5' if r4_item_use else '4' if legacy_item_use else '1' if legacy_r2 else '2' if legacy_attack else '3')
    allowed = R4_ENTRY_POINTS + ('power.borg_power', 'power.borg_power_equipment') if power_surface else R4_ENTRY_POINTS if item_surface else R2_ENTRY_POINTS if legacy_r2 else ENTRY_POINTS if complete else ATTACK_SURFACE
    if equipment_surface:
        allowed = allowed + ('equipment.borg_test_stuff', 'equipment.borg_swap_rings', 'equipment.borg_wear_rings', 'equipment.borg_backup_swap', 'equipment.borg_wear_stuff', 'equipment.borg_best_stuff', 'equipment.borg_clear_best', 'equipment.borg_wear_recharge', 'equipment.borg_object_fully_id')
    if store_surface:
        allowed = allowed + STORE_ENTRY_POINTS
    if receiving_surface:
        allowed = tuple(name for name in allowed if name not in R45_RETIRED) + R45_ADDED
    if r5:
        allowed += (FLOW_STEP_POINT,)
    fields = NUMERIC + (() if legacy_r2 else ('attack_calls', 'pure_attack_comparisons')) + (('defence_calls',) if complete else ()) + (('item_use_calls',) if item_surface else ()) + (('readonly_comparisons',) if r4_item_use else ()) + (('power_calls',) if power_surface else ()) + (('equipment_calls',) if equipment_surface else ()) + (('store_calls',) if store_surface else ())
    if r5:
        fields += ('flow_calls',)
    totals = dict.fromkeys((*fields, 'tape_events', 'policy_markers', 'dependency_requests'), 0)
    entries = dict.fromkeys(allowed, 0)
    pure_totals = dict.fromkeys(PURE_POINTS, 0) if not legacy_r2 else {}
    readonly_allowed = ({'item_use.borg_equips_item', 'power.borg_power_equipment'} if power_surface else {'item_use.borg_equips_item'} if r4_item_use else set())
    if store_surface:
        readonly_allowed = readonly_allowed | {'store.town_strategy_disabled','store.borg_power_home','store.borg_min_item_quantity'}
    if r5:
        readonly_allowed.add(FLOW_STEP_POINT)
    readonly_totals = dict.fromkeys(readonly_allowed, 0)
    for path in files:
        previous, last, file_mode = 0, None, None
        for line in path.read_text().splitlines():
            row = json.loads(line)
            if row.get('schema') != schema or row.get('mode') not in ('check', 'switch'):
                raise ValueError(f'{path}: wrong schema or mode (expected {schema})')
            mode = row['mode']
            if expected_mode is not None and mode != expected_mode:
                raise ValueError(f'{path}: expected {expected_mode} mode, got {mode}')
            if file_mode is not None and mode != file_mode:
                raise ValueError(f'{path}: mode changed within a run')
            file_mode = mode
            if row['counts']['thinks'] != previous + 1:
                raise ValueError(f'{path}: missing think statistics')
            previous, last = row['counts']['thinks'], row
        if last is None:
            raise ValueError(f'{path}: empty statistics')
        if not legacy_r2 and last.get('surface') != ('r5' if r5 else 'r45' if r45 else 'r4-store' if store_surface else 'r4-equipment' if equipment_surface else 'r4-power' if power_surface else 'r4-item-use-readonly' if r4_item_use else 'r4-item-use' if legacy_item_use else 'r3-attack-defence' if complete else 'r3-attack'):
            raise ValueError(f'{path}: unexpected entry surface')
        if last.get('ported_points') != R2_PORTED + ([] if legacy_r2 else ['attack', 'defence'] if complete else ['attack']) + (['item_use'] if item_surface else []) + (['power'] if power_surface else []) + (['equipment'] if equipment_surface else []) + (['store'] if store_surface else []) + (['flow'] if r5 else []):
            raise ValueError(f'{path}: unexpected port coverage')
        if last.get('unported_points') != []:
            raise ValueError(f'{path}: unexpected unported boundaries')
        counts = last['counts']
        required = (*fields, 'mismatches', 'dependency_requests') + (() if legacy_r2 else
                    ('tape_events', 'policy_markers'))
        if any(type(counts.get(k)) is not int or counts[k] < 0 for k in required):
            raise ValueError(f'{path}: missing or invalid numeric counters')
        if counts['danger_comparisons'] != counts['danger_calls']:
            raise ValueError(f'{path}: danger calls were observed without comparisons')
        stateful = checked_map(counts.get('stateful_points'), allowed, path)
        pure = {} if legacy_r2 else checked_map(counts.get('pure_points'), PURE_POINTS, path)
        readonly = checked_map(counts.get('readonly_points'), readonly_allowed, path) if r4_item_use else {}
        if r4_item_use and sum(readonly.values()) != counts['readonly_comparisons']:
            raise ValueError(f'{path}: inconsistent read-only root counts')
        if any(hits > stateful.get(name, 0) for name, hits in readonly.items()):
            raise ValueError(f'{path}: read-only roots without policy markers')
        root_allowed = set(allowed) - set(PURE_POINTS) - readonly_allowed
        roots_ = checked_map(counts.get('root_points'), root_allowed, path)
        if sum(roots_.values()) != counts['stateful_comparisons']:
            raise ValueError(f'{path}: inconsistent root comparison counts')
        # Both values come from one visits loop: this is integrity, not an
        # independent proof of coverage or of any branch/effect executing.
        for prefix in ('caution', 'escape', 'recovery') + (() if legacy_r2 else ('attack',)) + (('defence',) if complete else ()) + (('item_use',) if item_surface else ()) + (('power',) if power_surface else ()) + (('equipment',) if equipment_surface else ()) + (('store',) if store_surface else ()) + (('flow',) if r5 else ()):
            observed = sum(v for k, v in stateful.items() if k.startswith(prefix + '.'))
            if prefix == 'attack':
                observed += sum(pure.values())
            if observed != counts[prefix + '_calls']:
                raise ValueError(f'{path}: inconsistent individual entry coverage')
        markers = sum(stateful.values())
        tape = counts['dependency_requests'] if legacy_r2 else counts['tape_events']
        requests = tape - markers if legacy_r2 else counts['dependency_requests']
        if requests < 0 or (not legacy_r2 and counts['policy_markers'] != markers):
            raise ValueError(f'{path}: inconsistent policy markers')
        if tape != markers + requests:
            raise ValueError(f'{path}: inconsistent tape event identity')
        pure_count = sum(pure.values())
        if not legacy_r2 and pure_count != counts['pure_attack_comparisons']:
            raise ValueError(f'{path}: inconsistent pure comparison counts')
        parts = ('danger_comparisons', 'monster_comparisons', 'wall_comparisons', 'stateful_comparisons') + (('readonly_comparisons',) if r4_item_use else ())
        if counts['comparisons'] != sum(counts[k] for k in parts) + pure_count:
            raise ValueError(f'{path}: inconsistent comparison counts')
        if counts['stateful_comparisons'] + (counts['readonly_comparisons'] if r4_item_use else 0) > markers:
            raise ValueError(f'{path}: stateful roots without policy entries')
        if counts['mismatches']:
            raise ValueError(f'{path}: reported shadow mismatches')
        mismatch = path.with_name('shadow.jsonl')
        if not mismatch.exists() or mismatch.read_bytes():
            raise ValueError(f'{mismatch}: absent log or nonzero mismatches')
        for name in fields:
            totals[name] += counts[name]
        for name, value in (('tape_events', tape), ('policy_markers', markers),
                            ('dependency_requests', requests)):
            totals[name] += value
        for name, hits in stateful.items():
            entries[name] += hits
        for name, hits in readonly.items():
            readonly_totals[name] += hits
        for name, hits in pure.items():
            entries[name] += hits
            pure_totals[name] += hits
        if constructed_roots:
            label = 'constructed_runs' if path.resolve() in declared_constructed else 'natural_runs'
            if label not in run_coverage:
                run_coverage[label] = {'runs': 0, 'entry_points': dict.fromkeys(allowed, 0),
                                       'readonly_points': dict.fromkeys(readonly_allowed, 0)}
            group = run_coverage[label]
            group['runs'] += 1
            for mapping in (stateful, pure):
                for name, hits in mapping.items():
                    group['entry_points'][name] += hits
            for name, hits in readonly.items():
                group['readonly_points'][name] += hits
    if not all(totals[name] for name in fields):
        raise ValueError(f'insufficient comparison/boundary coverage: {totals}')
    if require_all_entries:
        missing_readonly = sorted(name for name, hits in readonly_totals.items() if hits == 0)
        if missing_readonly:
            raise ValueError(f'missing read-only root coverage: {missing_readonly}')
        missing = [name for name, hits in entries.items() if hits == 0]
        if missing:
            raise ValueError(f'missing individual entry coverage: {missing}')
        if not legacy_r2 and not all(pure_totals.values()):
            raise ValueError('missing standalone pure attack comparison coverage')
    totals['entry_points'] = entries
    totals['pure_points'] = pure_totals
    totals['readonly_points'] = readonly_totals
    if constructed_roots:
        for label in ('natural_runs', 'constructed_runs'):
            run_coverage.setdefault(label, {'runs': 0, 'entry_points': dict.fromkeys(allowed, 0),
                                            'readonly_points': dict.fromkeys(readonly_allowed, 0)})
        natural, constructed = run_coverage['natural_runs'], run_coverage['constructed_runs']
        run_coverage['only_in_constructed_runs'] = {
            'entry_points': sorted(n for n in allowed if not natural['entry_points'][n] and constructed['entry_points'][n]),
            'readonly_points': sorted(n for n in readonly_allowed if not natural['readonly_points'][n] and constructed['readonly_points'][n])}
        run_coverage['limits'] = 'Run provenance is explicitly declared by --constructed-root. Fixture-enabled runs also contain live harness calls; these are not per-call synthetic labels or branch/consumption proof.'
        totals['coverage_by_run_provenance'] = run_coverage
    return len(files), totals


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('roots', nargs='+', type=Path)
    parser.add_argument('--require-all-entries', action='store_true',
                        help='require every public entry; entry counts do not prove branch coverage')
    parser.add_argument('--legacy-r2', action='store_true',
                        help='explicit @1 reader; derive event meanings without rewriting raw evidence')
    parser.add_argument('--legacy-r3-attack', action='store_true',
                        help='explicit @2 attack reader; cannot satisfy complete defence coverage')
    parser.add_argument('--legacy-r4-item-use', action='store_true', help='explicit historical @4 all-stateful item-use reader')
    parser.add_argument('--r4-store', action='store_true', help='require complete @8 store/home surface')
    parser.add_argument('--r45', action='store_true', help='require the @9 receiving policy surface')
    parser.add_argument('--r5', action='store_true', help='require the @10 flow policy surface')
    parser.add_argument('--r4-equipment', action='store_true', help='require complete @7 equipment surface')
    parser.add_argument('--r4-power', action='store_true', help='require complete @6 power and item-use surface')
    parser.add_argument('--r4-item-use', action='store_true', help='require complete @5 read-only/mutable item-use surface; historical readers cannot satisfy it')
    parser.add_argument('--expect-mode', choices=('check', 'switch'),
                        help='require this mode for every record in the supplied arm')
    parser.add_argument('--constructed-root', action='append', type=Path, default=[],
                        help='declare fixture-enabled run roots for a separate coverage partition; combined strict gates are unchanged')
    parser.add_argument('--report', type=Path, help='write a new JSON gate report; refuses to overwrite an existing report')
    args = parser.parse_args()
    try:
        count, totals = check(args.roots, args.require_all_entries, args.legacy_r2, args.legacy_r3_attack, args.expect_mode, args.r4_item_use, args.legacy_r4_item_use, args.r4_power, args.r4_equipment, args.r4_store, args.constructed_root, r45=args.r45, r5=args.r5)
        if args.report:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            with args.report.open('x') as output:
                json.dump({'schema': 'borg-reborn-rs/shadow-gate-report@1',
                           'require_all_entries': args.require_all_entries, 'expected_mode': args.expect_mode,
                           'surface': 'r5' if args.r5 else 'r45' if args.r45 else 'r4-store' if args.r4_store else 'other-explicit-reader',
                           'constructed_root_count': len(args.constructed_root), 'runs': count,
                           'counts': totals, 'mismatches': 0}, output, indent=2)
                output.write('\n')
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f'shadow check failed: {error}\n')
    print(f'{count} shadow runs (legacy_r2={args.legacy_r2}): {totals}; zero mismatches')


if __name__ == '__main__':
    main()

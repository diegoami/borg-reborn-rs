"""Versioned shadow gates reject missing, vacuous and inconsistent evidence."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
spec = importlib.util.spec_from_file_location('check_shadow', Path(__file__).resolve().parents[2] / 'harness/check-shadow.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def valid_row():
    return {'schema': 'borg-reborn-rs/shadow-stats@3', 'surface': 'r3-attack-defence',
            'mode': 'check', 'ported_points': module.R2_PORTED + ['attack', 'defence'], 'unported_points': [],
            'counts': {'thinks': 1, 'mismatches': 0, 'comparisons': 6,
                       'danger_calls': 1, 'danger_comparisons': 1, 'monster_comparisons': 1,
                       'wall_comparisons': 1, 'stateful_comparisons': 1,
                       'caution_calls': 1, 'escape_calls': 9, 'recovery_calls': 1,
                       'attack_calls': 9, 'pure_attack_comparisons': 2, 'defence_calls': 1,
                       'tape_events': 21, 'policy_markers': 19, 'dependency_requests': 2,
                       'stateful_points': dict.fromkeys(module.ENTRY_POINTS, 1),
                       'pure_points': dict.fromkeys(module.PURE_POINTS, 1),
                       'root_points': {'caution.borg_caution': 1}}}


class ShadowCheckTest(unittest.TestCase):
    def write(self, root, row):
        (root / 'shadow-stats.jsonl').write_text(json.dumps(row) + '\n')
        (root / 'shadow.jsonl').write_text('')

    def test_r4_requires_its_exact_surface_and_all_item_entries(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], legacy_item_use=True)
            row.update(schema='borg-reborn-rs/shadow-stats@4', surface='r4-item-use',
                       ported_points=module.R2_PORTED + ['attack', 'defence', 'item_use'])
            counts = row['counts']
            counts['item_use_calls'] = len(module.ITEM_USE_ENTRY_POINTS)
            counts['stateful_points'].update(dict.fromkeys(module.ITEM_USE_ENTRY_POINTS, 1))
            counts['policy_markers'] += len(module.ITEM_USE_ENTRY_POINTS)
            counts['tape_events'] += len(module.ITEM_USE_ENTRY_POINTS)
            self.write(root, row)
            module.check([root], require_all_entries=True, expected_mode='check', legacy_item_use=True)
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root])
            counts['stateful_points'][module.ITEM_USE_ENTRY_POINTS[0]] = 0
            counts['item_use_calls'] -= 1
            counts['policy_markers'] -= 1
            counts['tape_events'] -= 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'missing individual entry'):
                module.check([root], require_all_entries=True, legacy_item_use=True)

    def test_readonly_root_partition_and_legacy_separation(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row()
            row.update(schema='borg-reborn-rs/shadow-stats@5', surface='r4-item-use-readonly',
                       ported_points=module.R2_PORTED + ['attack', 'defence', 'item_use'])
            counts=row['counts']; counts['item_use_calls']=len(module.ITEM_USE_ENTRY_POINTS)
            counts['stateful_points'].update(dict.fromkeys(module.ITEM_USE_ENTRY_POINTS, 1))
            counts['policy_markers']+=len(module.ITEM_USE_ENTRY_POINTS)
            counts['tape_events']+=len(module.ITEM_USE_ENTRY_POINTS)
            counts['readonly_comparisons']=1
            counts['readonly_points']={'item_use.borg_equips_item':1}
            counts['comparisons']+=1
            self.write(root,row)
            module.check([root], require_all_entries=True, r4_item_use=True)
            with self.assertRaisesRegex(ValueError,'wrong schema'):
                module.check([root], legacy_item_use=True)
            for field, value, error in [
                ('readonly_comparisons',2,'read-only root counts'),
                ('readonly_points',{'item_use.borg_activate_item':1},'entry coverage'),
                ('root_points',{'item_use.borg_equips_item':1},'entry coverage'),
                ('comparisons',6,'comparison counts'),
            ]:
                changed=copy.deepcopy(row);changed['counts'][field]=value;self.write(root,changed)
                with self.assertRaisesRegex(ValueError,error):
                    module.check([root], r4_item_use=True)

    def test_power_requires_v6_entries_and_immutable_equipment_root(self):
        row = valid_row()
        row.update(schema='borg-reborn-rs/shadow-stats@6', surface='r4-power',
                   ported_points=module.R2_PORTED + ['attack', 'defence', 'item_use', 'power'])
        c = row['counts']
        c['stateful_points'].update(dict.fromkeys(module.ITEM_USE_ENTRY_POINTS, 1))
        c['stateful_points'].update({'power.borg_power':1, 'power.borg_power_equipment':1})
        added = len(module.ITEM_USE_ENTRY_POINTS) + 2
        c.update(item_use_calls=len(module.ITEM_USE_ENTRY_POINTS), power_calls=2,
                 readonly_comparisons=2, readonly_points={'item_use.borg_equips_item':1, 'power.borg_power_equipment':1},
                 stateful_comparisons=2, comparisons=9)
        c['root_points']['power.borg_power'] = 1
        c['policy_markers'] += added
        c['tape_events'] += added
        with tempfile.TemporaryDirectory() as name:
            root=Path(name);self.write(root,row)
            module.check([root], r4_power=True, require_all_entries=True)
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], r4_item_use=True)
            for key,value,error in [('readonly_points',{'item_use.borg_equips_item':2},'read-only roots'),
                                    ('root_points',{'power.borg_power_equipment':2},'entry coverage'),
                                    ('power_calls',1,'individual entry coverage')]:
                changed=copy.deepcopy(row);changed['counts'][key]=value;self.write(root,changed)
                with self.assertRaisesRegex(ValueError,error):
                    module.check([root],r4_power=True,require_all_entries=True)

    def test_nested_equipment_marker_cannot_replace_each_readonly_root(self):
        row = valid_row()
        row.update(schema='borg-reborn-rs/shadow-stats@6', surface='r4-power',
                   ported_points=module.R2_PORTED + ['attack', 'defence', 'item_use', 'power'])
        c=row['counts']
        c['stateful_points'].update(dict.fromkeys(module.ITEM_USE_ENTRY_POINTS,1))
        c['stateful_points'].update({'power.borg_power':1, 'power.borg_power_equipment':1})
        c['root_points']['power.borg_power']=1
        c.update(item_use_calls=len(module.ITEM_USE_ENTRY_POINTS),power_calls=2,
                 stateful_comparisons=2,readonly_comparisons=1,comparisons=8)
        added=len(module.ITEM_USE_ENTRY_POINTS)+2
        c['policy_markers']+=added
        c['tape_events']+=added
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            for missing,present in [('power.borg_power_equipment','item_use.borg_equips_item'),
                                    ('item_use.borg_equips_item','power.borg_power_equipment')]:
                for explicitly_zero in [False,True]:
                    with self.subTest(missing=missing,explicitly_zero=explicitly_zero):
                        changed=copy.deepcopy(row)
                        changed['counts']['readonly_points']={present:1}
                        if explicitly_zero:
                            changed['counts']['readonly_points'][missing]=0
                        self.write(root,changed)
                        # All policy markers and arithmetic are otherwise valid.
                        module.check([root],r4_power=True)
                        with self.assertRaisesRegex(ValueError,'missing read-only root coverage'):
                            module.check([root],r4_power=True,require_all_entries=True)

    def test_equipment_surface_requires_v7_and_all_equipment_entries(self):
        row=valid_row()
        names=('borg_test_stuff','borg_swap_rings','borg_wear_rings','borg_backup_swap',
               'borg_wear_stuff','borg_best_stuff','borg_clear_best','borg_wear_recharge','borg_object_fully_id')
        row.update(schema='borg-reborn-rs/shadow-stats@7',surface='r4-equipment',
                   ported_points=module.R2_PORTED+['attack','defence','item_use','power','equipment'])
        c=row['counts']
        c['stateful_points'].update(dict.fromkeys(module.ITEM_USE_ENTRY_POINTS,1))
        c['stateful_points'].update({'power.borg_power':1,'power.borg_power_equipment':1})
        c['stateful_points'].update({'equipment.'+name:1 for name in names})
        added=len(module.ITEM_USE_ENTRY_POINTS)+2+len(names)
        c['policy_markers']+=added;c['tape_events']+=added
        c.update(item_use_calls=len(module.ITEM_USE_ENTRY_POINTS),power_calls=2,equipment_calls=len(names),
                 readonly_comparisons=2,readonly_points={'item_use.borg_equips_item':1,'power.borg_power_equipment':1},
                 stateful_comparisons=3,comparisons=10)
        c['root_points'].update({'power.borg_power':1,'equipment.borg_test_stuff':1})
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);self.write(root,row)
            module.check([root],r4_equipment=True,require_all_entries=True)
            with self.assertRaisesRegex(ValueError,'wrong schema'):
                module.check([root],r4_power=True)
            missing=copy.deepcopy(row)
            del missing['counts']['stateful_points']['equipment.borg_wear_rings']
            for field in ['equipment_calls','policy_markers','tape_events']:missing['counts'][field]-=1
            self.write(root,missing);module.check([root],r4_equipment=True)
            with self.assertRaisesRegex(ValueError,'missing individual entry coverage'):
                module.check([root],r4_equipment=True,require_all_entries=True)

    def test_store_surface_requires_v8_entries_and_immutable_root(self):
        row=valid_store_row()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);self.write(root,row)
            module.check([root],r4_store=True,require_all_entries=True,expected_mode='check')
            with self.assertRaisesRegex(ValueError,'wrong schema'):
                module.check([root],r4_equipment=True)
            missing=copy.deepcopy(row)
            del missing['counts']['readonly_points']['store.town_strategy_disabled']
            missing['counts']['readonly_comparisons']-=1;missing['counts']['comparisons']-=1
            self.write(root,missing);module.check([root],r4_store=True)
            with self.assertRaisesRegex(ValueError,'missing read-only root coverage'):
                module.check([root],r4_store=True,require_all_entries=True)
            missing=copy.deepcopy(row)
            del missing['counts']['stateful_points']['store.confirm_purchase']
            for name in ['store_calls','policy_markers','tape_events']:missing['counts'][name]-=1
            self.write(root,missing);module.check([root],r4_store=True)
            with self.assertRaisesRegex(ValueError,'missing individual entry coverage'):
                module.check([root],r4_store=True,require_all_entries=True)

    def test_receiving_surface_replaces_retired_markers(self):
        row = valid_receiving_row()
        counts = row['counts']
        points = counts['stateful_points']
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write(root, row)
            module.check([root], r45=True, require_all_entries=True)
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], r4_store=True)

            point = 'caution.lightless_self_care'
            del points[point]
            counts['caution_calls'] -= 1
            counts['policy_markers'] -= 1
            counts['tape_events'] -= 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'missing individual entry coverage'):
                module.check([root], r45=True, require_all_entries=True)

            points['escape.borg_caution_phase'] = 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'entry coverage'):
                module.check([root], r45=True)

    def test_r5_requires_the_flow_roots_and_checks_their_counter(self):
        row = valid_receiving_row()
        row.update(schema='borg-reborn-rs/shadow-stats@10', surface='r5',
                   ported_points=row['ported_points'] + ['flow'])
        counts = row['counts']
        points = dict.fromkeys(module.FLOW_POINTS, 1)
        counts['stateful_points'].update(points)
        counts['readonly_points'].update(points)
        counts['flow_calls'] = len(points)
        for field in ('policy_markers', 'tape_events', 'readonly_comparisons', 'comparisons'):
            counts[field] += len(points)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write(root, row)
            _, totals = module.check([root], r5=True, require_all_entries=True)
            self.assertEqual(totals['flow_calls'], len(points))
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], r45=True)

            counts['flow_calls'] += 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'inconsistent individual entry coverage'):
                module.check([root], r5=True)
            counts['flow_calls'] -= 1

            del counts['readonly_points']['flow.plan_up_stair']
            counts['readonly_comparisons'] -= 1
            counts['comparisons'] -= 1
            self.write(root, row)
            module.check([root], r5=True)
            with self.assertRaisesRegex(ValueError, 'missing read-only root coverage'):
                module.check([root], r5=True, require_all_entries=True)

    def test_declared_run_partition_keeps_combined_strict_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);natural=root/'natural';constructed=root/'constructed'
            natural.mkdir();constructed.mkdir()
            row=valid_store_row();missing=copy.deepcopy(row)
            del missing['counts']['readonly_points']['store.town_strategy_disabled']
            missing['counts']['readonly_comparisons']-=1;missing['counts']['comparisons']-=1
            del missing['counts']['stateful_points']['store.sell_one_selected']
            for name in ['store_calls','policy_markers','tape_events']:missing['counts'][name]-=1
            self.write(natural,missing);self.write(constructed,row)
            before=(natural/'shadow-stats.jsonl').read_bytes()
            _,totals=module.check([root],r4_store=True,require_all_entries=True,constructed_roots=[constructed])
            split=totals['coverage_by_run_provenance']
            self.assertEqual(split['natural_runs']['runs'],1);self.assertEqual(split['constructed_runs']['runs'],1)
            self.assertEqual(split['only_in_constructed_runs']['readonly_points'],['store.town_strategy_disabled'])
            self.assertEqual(split['only_in_constructed_runs']['entry_points'],['store.sell_one_selected'])
            self.assertEqual(totals['comparisons'],row['counts']['comparisons']+missing['counts']['comparisons'])
            with self.assertRaisesRegex(ValueError,'missing read-only root coverage'):
                module.check([natural],r4_store=True,require_all_entries=True)
            self.assertEqual(before,(natural/'shadow-stats.jsonl').read_bytes())
            with self.assertRaisesRegex(ValueError,'included in the checked roots'):
                module.check([natural],r4_store=True,constructed_roots=[constructed])

    def test_cli_report_records_gate_options_and_never_overwrites(self):
        import subprocess,sys
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);self.write(root,valid_store_row());report=root/'gate.json'
            cmd=[sys.executable,str(module.__file__),'--r4-store','--require-all-entries','--expect-mode','check',str(root),'--constructed-root',str(root),'--report',str(report)]
            result=subprocess.run(cmd,capture_output=True,text=True);self.assertEqual(result.returncode,0,result.stderr)
            value=json.loads(report.read_text());self.assertTrue(value['require_all_entries']);self.assertEqual(value['expected_mode'],'check')
            self.assertEqual(value['counts']['coverage_by_run_provenance']['natural_runs']['runs'],0)
            self.assertEqual(value['counts']['coverage_by_run_provenance']['constructed_runs']['runs'],1)
            saved=report.read_bytes();self.assertNotEqual(subprocess.run(cmd,capture_output=True).returncode,0);self.assertEqual(report.read_bytes(),saved)

    def test_expected_arm_mode_rejects_the_other_valid_mode(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); self.write(root, row)
            module.check([root], expected_mode='check')
            with self.assertRaisesRegex(ValueError, 'expected switch mode'):
                module.check([root], expected_mode='switch')
            row['mode'] = 'switch'; self.write(root, row)
            module.check([root], expected_mode='switch')
            with self.assertRaisesRegex(ValueError, 'expected check mode'):
                module.check([root], expected_mode='check')

    def test_mode_cannot_change_inside_a_statistics_stream(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); first = valid_row(); second = copy.deepcopy(first)
            second['mode'] = 'switch'; second['counts']['thinks'] = 2
            self.write(root, first)
            with (root / 'shadow-stats.jsonl').open('a') as out:
                out.write(json.dumps(second) + '\n')
            with self.assertRaisesRegex(ValueError, 'mode changed within a run'):
                module.check([root])

    def test_missing_zero_and_divergent_evidence_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            with self.assertRaisesRegex(ValueError, 'no shadow'):
                module.check([root])
            row = valid_row()
            row['counts'] = {k: (1 if k == 'thinks' else {} if isinstance(v, dict) else 0)
                             for k, v in row['counts'].items()}
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'insufficient'):
                module.check([root])
            self.write(root, valid_row())
            self.assertEqual(module.check([root])[1]['comparisons'], 6)
            (root / 'shadow.jsonl').write_text('{"original":false,"ported":true}\n')
            with self.assertRaisesRegex(ValueError, 'nonzero'):
                module.check([root])

    def test_wall_comparisons_cannot_substitute_for_danger(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            row = valid_row(); row['counts']['danger_comparisons'] = 0
            row['counts']['wall_comparisons'] += 1
            self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'without comparisons'):
                module.check([root])

    def test_aggregate_totals_cannot_hide_a_missing_entry(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); self.write(root, row)
            self.assertTrue(module.check([root], True)[1]['entry_points'])
            row['counts']['stateful_points']['escape.borg_caution_teleport'] = 0
            row['counts']['escape_calls'] -= 1
            row['counts']['policy_markers'] -= 1
            row['counts']['dependency_requests'] += 1
            self.write(root, row)
            module.check([root])
            with self.assertRaisesRegex(ValueError, 'caution_teleport'):
                module.check([root], True)
            row['counts']['escape_calls'] += 1; self.write(root, row)
            with self.assertRaisesRegex(ValueError, 'inconsistent individual'):
                module.check([root], True)

    def test_event_and_comparison_identities_are_independently_checked(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for field in ['tape_events', 'policy_markers', 'dependency_requests',
                          'pure_attack_comparisons', 'attack_calls', 'comparisons',
                          'stateful_comparisons', 'defence_calls']:
                with self.subTest(field=field):
                    row = valid_row(); row['counts'][field] += 1; self.write(root, row)
                    with self.assertRaisesRegex(ValueError, 'inconsistent'):
                        module.check([root], True)

    def test_internal_markers_do_not_substitute_for_standalone_pure_comparisons(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row()
            row['counts']['pure_points']['attack.borg_thrust_damage_one'] = 0
            row['counts']['pure_attack_comparisons'] -= 1
            row['counts']['attack_calls'] -= 1; row['counts']['comparisons'] -= 1
            self.write(root, row)
            module.check([root])
            with self.assertRaisesRegex(ValueError, 'standalone pure'):
                module.check([root], True)

    def test_legacy_is_explicit_and_raw_event_meanings_are_preserved(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row(); row['schema'] = 'borg-reborn-rs/shadow-stats@1'
            row.pop('surface'); row['ported_points'] = module.R2_PORTED
            c = row['counts']; c['stateful_points'] = dict.fromkeys(module.R2_ENTRY_POINTS, 1)
            c['comparisons'] = 4; c['dependency_requests'] = 20
            for field in ['tape_events', 'policy_markers', 'pure_points', 'pure_attack_comparisons', 'attack_calls']:
                c.pop(field)
            self.write(root, row); before = (root / 'shadow-stats.jsonl').read_bytes()
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root])
            totals = module.check([root], True, legacy_r2=True)[1]
            self.assertEqual((totals['tape_events'], totals['policy_markers'], totals['dependency_requests']),
                             (20, 11, 9))
            self.assertEqual(before, (root / 'shadow-stats.jsonl').read_bytes())

    def test_wrong_surface_unknown_entries_and_boolean_counters_fail(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for variant in ['surface', 'entry', 'boolean']:
                row = valid_row()
                if variant == 'surface': row['surface'] = 'old-surface'
                elif variant == 'entry': row['counts']['stateful_points']['fake.entry'] = 1
                else: row['counts']['comparisons'] = True
                self.write(root, row)
                with self.assertRaises(ValueError): module.check([root])

    def test_attack_only_evidence_requires_explicit_legacy_reader(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); row = valid_row()
            row['schema'] = 'borg-reborn-rs/shadow-stats@2'
            row['surface'] = 'r3-attack'; row['ported_points'].remove('defence')
            counts = row['counts']; counts.pop('defence_calls')
            counts['stateful_points'].pop('defence.borg_defend')
            counts['tape_events'] -= 1; counts['policy_markers'] -= 1
            self.write(root, row)
            before = (root / 'shadow-stats.jsonl').read_bytes()
            with self.assertRaisesRegex(ValueError, 'wrong schema'):
                module.check([root], True)
            self.assertEqual(module.check([root], True, legacy_attack=True)[1]['policy_markers'], 18)
            self.assertEqual(before, (root / 'shadow-stats.jsonl').read_bytes())


def valid_store_row():
    row=valid_row()
    equipment=('borg_test_stuff','borg_swap_rings','borg_wear_rings','borg_backup_swap',
               'borg_wear_stuff','borg_best_stuff','borg_clear_best','borg_wear_recharge','borg_object_fully_id')
    row.update(schema='borg-reborn-rs/shadow-stats@8',surface='r4-store',
               ported_points=module.R2_PORTED+['attack','defence','item_use','power','equipment','store'])
    c=row['counts']
    extra=(*module.ITEM_USE_ENTRY_POINTS,'power.borg_power','power.borg_power_equipment',
           *('equipment.'+name for name in equipment),*module.STORE_ENTRY_POINTS)
    c['stateful_points'].update(dict.fromkeys(extra,1))
    c['policy_markers']+=len(extra);c['tape_events']+=len(extra)
    c.update(item_use_calls=len(module.ITEM_USE_ENTRY_POINTS),power_calls=2,
             equipment_calls=len(equipment),store_calls=len(module.STORE_ENTRY_POINTS),
             readonly_comparisons=5,readonly_points={'item_use.borg_equips_item':1,
             'power.borg_power_equipment':1,'store.town_strategy_disabled':1,
             'store.borg_power_home':1,'store.borg_min_item_quantity':1},
             stateful_comparisons=4,comparisons=14)
    c['root_points'].update({'power.borg_power':1,'equipment.borg_test_stuff':1,'store.borg_think_shop_buy_useful':1})
    return row


def valid_receiving_row():
    row = valid_store_row()
    row.update(schema='borg-reborn-rs/shadow-stats@9', surface='r45')
    counts = row['counts']
    points = counts['stateful_points']
    for name in module.R45_RETIRED:
        counts[name.split('.')[0] + '_calls'] -= points.pop(name)
    for name in module.R45_ADDED:
        points[name] = 1
        counts[name.split('.')[0] + '_calls'] += 1
    counts['policy_markers'] = sum(points.values())
    counts['tape_events'] = counts['policy_markers'] + counts['dependency_requests']
    return row

"""Owner-review semantics, prospective bounds and owned-child stall controls.

Synthetic control fixtures are not natural game outcomes; real integration
uses the root commands separately with private profiles/registries.
"""
import importlib.util
from contextlib import nullcontext
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

import borg_runtime as runtime
import borg_deaths as deaths
import death_workflow as workflow

ROOT = Path(__file__).resolve().parents[1]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


queue = load('rust_review_queue', ROOT / 'replay_next_death.py')
producer = load('rust_death_producer', ROOT / 'produce_death.py')


class OwnerQueue(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='borg-queue-control-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.records = self.root / 'deaths'
        self.records.mkdir()
        patch = mock.patch.dict(os.environ, {'BORG_DEATH_RECORDS_DIR': str(self.records), 'BORG_DEATH_WORKFLOW_DIR': str(self.root / 'workflow')})
        patch.start();self.addCleanup(patch.stop)

    def case(self, case, family, recorded=True, qualified_family=None):
        folder = self.records / (case + '-fixture');folder.mkdir()
        record = {'schema': deaths.SCHEMA, 'id': case, 'identity': {'turn': 77, 'seed': 1, 'encounter_family': family},
                  'encounter_family': family, 'raw_cause': 'a synthetic foe'}
        if recorded:
            bundle = self.root / (case + '-bundle');(bundle / 'runtime/game').mkdir(parents=True)
            (bundle / 'runtime/game/angband').write_bytes(b'synthetic queue-only executable fixture')
            (bundle / 'frames.jsonl.gz').write_bytes(b'synthetic queue-only frame fixture')
            hashes = {str(p.relative_to(bundle)): workflow.sha(p) for p in bundle.rglob('*') if p.is_file()}
            workflow.write(bundle / 'recording.json', {'schema': runtime.SCHEMA, 'status': 'complete', 'files_sha256': hashes})
            record['recording'] = {'raw_directory': str(bundle), 'manifest_sha256': workflow.sha(bundle / 'recording.json')}
        workflow.write(folder / 'record.json', record)
        assessment = {'classification': 'unassessed', 'actions': []}
        if qualified_family:
            assessment.update(encounter_family=qualified_family, encounter_family_qualification={'status': 'verified'})
        workflow.write(folder / 'assessment.json', assessment)
        return folder

    def args(self, **kw):
        return SimpleNamespace(mark_reviewed=kw.get('mark_reviewed'), mark_unreviewed=kw.get('mark_unreviewed'),
            list=kw.get('list', False), json=False, case=kw.get('case'), timeout=90, from_turn=0,
            view_only=kw.get('view_only', False), verify_only=kw.get('verify_only', False))

    def test_family_mark_is_explicit_and_does_not_touch_assessment(self):
        self.case('D0001', 'original', False)
        second = self.case('D0002', 'variant', qualified_family='original')
        assessment = (second / 'assessment.json').read_bytes()
        self.assertEqual(len(workflow.queue()), 1)
        self.assertEqual(workflow.queue()[0]['playable']['case'], 'D0002')
        workflow.mark('D0002', True)
        self.assertTrue(workflow.queue()[0]['reviewed'])
        self.assertEqual((second / 'assessment.json').read_bytes(), assessment)
        workflow.mark('D0001', False)
        self.assertFalse(workflow.queue()[0]['reviewed'])

    def test_missing_or_corrupt_recording_is_skipped_not_marked(self):
        old = self.case('D0001', 'old', False)
        new = self.case('D0002', 'new')
        bundle = Path(json.loads((new / 'record.json').read_text())['recording']['raw_directory'])
        (bundle / 'runtime/game/angband').write_bytes(b'tampered')
        self.assertTrue(all(group['playable'] is None for group in workflow.queue()))
        self.assertFalse(any(group['reviewed'] for group in workflow.queue()))
        self.assertFalse((workflow.artifacts() / 'owner-reviews.json').exists())

    def test_successful_replay_does_not_mark_review_complete(self):
        self.case('D0001', 'old', False)
        self.case('D0002', 'new')
        with mock.patch.object(queue.subprocess, 'run', return_value=SimpleNamespace(returncode=0)) as execute:
            self.assertEqual(queue.run(self.args(verify_only=True)), 0)
        command = execute.call_args.args[0]
        self.assertIn('--output', command)
        self.assertIn('--verify-only', command)
        self.assertIn('0', command)
        self.assertFalse((workflow.artifacts() / 'owner-reviews.json').exists())

    def test_view_only_omits_verification_output_and_preserves_review_state(self):
        self.case('D0001', 'new')
        with mock.patch.object(queue.subprocess, 'run', return_value=SimpleNamespace(returncode=0)) as execute:
            self.assertEqual(queue.run(self.args(view_only=True)), 0)
        self.assertIn('--view-only', execute.call_args.args[0])
        self.assertNotIn('--output', execute.call_args.args[0])
        self.assertFalse(workflow.queue()[0]['reviewed'])

    def test_local_unreviewed_override_wins_and_explicit_case_can_replay_reviewed(self):
        self.case('D0001', 'new')
        workflow.write(self.records / 'owner-reviews.json', {'reviews': {'new': {'reviewed': True}}})
        self.assertTrue(workflow.queue()[0]['reviewed'])
        with mock.patch.object(queue.subprocess, 'run', return_value=SimpleNamespace(returncode=0)) as execute:
            self.assertEqual(queue.run(self.args()), 0);execute.assert_not_called()
            self.assertEqual(queue.run(self.args(case='D0001')), 0);execute.assert_called_once()
        workflow.mark('D0001', False)
        self.assertFalse(workflow.queue()[0]['reviewed'])

    def test_legacy_loaded_seed_variants_share_one_review_family(self):
        folders = [self.case('D0001', 'old-seed1'), self.case('D0002', 'old-seed2')]
        originals = []
        for seed, folder in enumerate(folders, 1):
            record = workflow.read(folder / 'record.json')
            record['identity'].update(start_save='same-origin', seed=seed, config_sha256='same-config')
            record.update(recording_schema=runtime.SCHEMA, terminal={'race_idx':0, 'class_idx':0})
            workflow.write(folder / 'record.json', record)
            originals.append((folder / 'record.json').read_bytes())
        self.assertEqual(len(workflow.queue()), 1)
        workflow.mark('D0002', True)
        self.assertTrue(workflow.queue()[0]['reviewed'])
        self.assertEqual([(p / 'record.json').read_bytes() for p in folders], originals)

    def test_missing_manifest_cannot_be_selected_as_playable(self):
        folder = self.case('D0001', 'new')
        bundle = Path(workflow.read(folder / 'record.json')['recording']['raw_directory'])
        (bundle / 'recording.json').unlink()
        self.assertIsNone(workflow.queue()[0]['playable'])
        with self.assertRaises(ValueError):
            queue.run(self.args(case='D0001'))


class Collection(unittest.TestCase):
    def options(self, **kw):
        values = dict(games=None, seed=None, seeds=None, birth='random', turns=100, timeout=2,
                      stall_timeout=1, keys=1000, checkpoint_every=100, death_issue=None, build=None)
        values.update(kw)
        return SimpleNamespace(**values)

    def test_seed_bounds_and_modes_are_prospective(self):
        self.assertEqual(producer.seeds_for(self.options(games=2, seed=0xffffffff)), [0xffffffff, 0])
        for args in (self.options(seeds=[1,1]), self.options(seeds=[]), self.options(seeds=[1], games=1), self.options(seeds=[-1])):
            with self.assertRaises(ValueError):producer.seeds_for(args)

    def test_random_births_are_fixed_before_outcomes_and_use_supported_menus(self):
        pin = workflow.source_commit('HEAD')
        first = producer.prospective_births(list(range(20)), 'random', pin)
        self.assertEqual(first, producer.prospective_births(list(range(20)), 'random', pin))
        self.assertTrue(all(p['race'] is None and p['class'] is None for p in first))
        self.assertTrue(all(p['race'] == p['class'] == 0 for p in producer.prospective_births([1,2], 'fixed', pin)))

    def test_non_rust_or_non_harness_build_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / 'BUILD_ID').write_text('impl=c\nsource_sha256=x\n')
            (directory / 'CMakeCache.txt').write_text('SUPPORT_BORG_HARNESS:BOOL=OFF\n')
            with self.assertRaises(ValueError):workflow.validate_build(directory, 'x')

    def test_changed_or_extra_prepared_inputs_are_rejected(self):
        with tempfile.TemporaryDirectory(prefix='borg-prepared-control-') as temporary:
            root = Path(temporary);binary = root / 'game/angband';binary.parent.mkdir();binary.write_bytes(b'synthetic cached image')
            source = root / 'source/input';source.parent.mkdir();source.write_bytes(b'original')
            commit = workflow.source_commit('HEAD');hashes = workflow.tool_hashes()
            manifest = {'schema':'borg-rs/death-prepared-runtime@1','complete':True,'source_commit':commit,
                        'source_inputs_sha256':workflow.source_inputs(commit),'tools_sha256':hashes,'cache':str(root),
                        'build_directory':str(root / 'build'),'binary':str(binary),'build_id':{'binary_sha256':workflow.sha(binary)},
                        'files_sha256':{str(p.relative_to(root)):workflow.sha(p) for p in (binary,source)}}
            workflow.validate_prepared(manifest,commit,hashes)
            source.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'input changed'):workflow.validate_prepared(manifest,commit,hashes)
            source.write_bytes(b'original');(root / 'source/extra').write_bytes(b'extra')
            with self.assertRaisesRegex(ValueError,'file set changed'):workflow.validate_prepared(manifest,commit,hashes)

    def test_live_tool_edit_is_rejected_before_starting_another_game(self):
        with mock.patch.object(workflow, 'sha', return_value='changed-after-import'):
            with self.assertRaisesRegex(ValueError,'changed after this process loaded'):
                workflow.tool_hashes()

    def test_retirement_is_not_a_natural_death(self):
        with tempfile.TemporaryDirectory(prefix='borg-retirement-control-') as temporary:
            bundle = Path(temporary);(bundle / 'run').mkdir()
            workflow.write(bundle / 'run/process.json', {'status':'success'})
            workflow.write(bundle / 'run/summary.json', {'dead':True,'stop_reason':'death','death_cause':'Retiring','race':'Human','class':'Warrior'})
            self.assertEqual(producer.outcome(bundle)['outcome'], 'retirement')

    def simulate(self, outcomes, export_error=False):
        with tempfile.TemporaryDirectory(prefix='borg-collect-control-') as temporary:
            root = Path(temporary);output = root / 'collection';output.mkdir()
            cache = root / 'cache';cache.mkdir();(cache / 'prepared.json').write_text('{}')
            births = [{'seed': seed, 'race': 0, 'class': 0, 'race_name': 'Human', 'class_name': 'Warrior'} for seed in [3,4,5]]
            calls = []
            def prepared(*_):
                plan = workflow.read(output / 'collection.json')
                self.assertEqual(plan['seeds'], [3,4,5]);self.assertEqual(plan['status'], 'preparing')
                return {'cache': str(cache)}
            def game(options, **kwargs):
                calls.append(options.seed)
                plan = workflow.read(output / 'collection.json');self.assertEqual(len(plan['runs']), len(calls))
                bundle = kwargs['profile_root'] / 'recordings/run';(bundle / 'run').mkdir(parents=True)
                outcome = outcomes[len(calls) - 1]
                workflow.write(bundle / 'recording.json', {'fixture': True})
                workflow.write(bundle / 'run/process.json', {'status': 'stall' if outcome == 'stall' else 'success'})
                workflow.write(bundle / 'run/summary.json', {'dead': outcome == 'death', 'stop_reason': {'death':'death','cap':'turns','keys':'keys','stall':'unknown'}[outcome], 'death_cause':'synthetic foe', 'race':'Human','class':'Warrior'})
                if outcome == 'stall':raise ValueError('synthetic stall')
                return bundle
            with mock.patch.dict(os.environ, {'BORG_DEATH_WORKFLOW_DIR': str(root / 'artifacts')}), \
                 mock.patch.object(workflow, 'locked', return_value=nullcontext()), \
                 mock.patch.object(workflow, 'tool_hashes', return_value={}), \
                 mock.patch.object(workflow, 'prepare_runtime', side_effect=prepared), \
                 mock.patch.object(workflow, 'validate_prepared'), \
                 mock.patch.object(runtime, 'validate'), \
                 mock.patch.object(runtime, 'record_run', side_effect=game), \
                 mock.patch.object(deaths, 'maybe_register', side_effect=ValueError('export failed') if export_error else None,
                                  return_value=root / 'D0001') as export:
                try:code = producer.collect(self.options(), output, births, 'synthetic-pin')
                except ValueError:code = 1
            return code, calls, workflow.read(output / 'collection.json'), export.call_count

    def test_every_exported_death_continues_without_extra_seed_or_budget(self):
        code, calls, plan, exports = self.simulate(['cap','death','death'])
        self.assertEqual((code,calls,exports), (0,[3,4,5],2))
        self.assertEqual(plan['counts'], {'turn_cap':1,'death':2})
        self.assertEqual(plan['status'], 'completed')

    def test_no_death_exhausts_only_the_planned_games(self):
        code,calls,plan,exports = self.simulate(['cap','keys','cap'])
        self.assertEqual((code,calls,exports), (0,[3,4,5],0))
        self.assertEqual(plan['counts'], {'turn_cap':2,'key_cap':1})
        self.assertEqual(plan['status'], 'limits_reached_without_death')

    def test_stall_stops_and_is_not_a_death(self):
        code,calls,plan,exports = self.simulate(['stall'])
        self.assertEqual((code,calls,exports), (1,[3],0))
        self.assertEqual(plan['counts'], {'stall':1})
        self.assertEqual(plan['status'], 'failed')

    def test_export_failure_preserves_real_outcome_and_never_advances(self):
        code,calls,plan,exports = self.simulate(['death'], export_error=True)
        self.assertEqual((code,calls,exports), (1,[3],1))
        self.assertEqual(plan['counts'], {'death':1})
        self.assertEqual(plan['status'], 'failed')


class StallSupervisor(unittest.TestCase):
    def supervise(self, script, seconds=.35):
        with tempfile.TemporaryDirectory(prefix='borg-stall-control-') as temporary:
            root = Path(temporary)
            child = root / 'child.py';child.write_text(script)
            process = subprocess.run([sys.executable, str(ROOT / 'harness/supervise.py'), '--receipt', str(root / 'process.json'),
                '--diagnostics', str(root), '--timeout', '5', '--stall-timeout', str(seconds), '--', sys.executable, str(child)],
                cwd=root, capture_output=True, text=True, timeout=10)
            return process.returncode, json.loads((root / 'process.json').read_text())

    def test_zero_clock_activity_stalls_and_preserves_failure_receipt(self):
        code, receipt = self.supervise("import time\nf=open('keys.log','w')\nwhile True:\n f.write('1 1 61 0\\n'); f.flush(); time.sleep(.03)\n")
        self.assertEqual(code,125);self.assertEqual(receipt['status'],'stall')
        self.assertTrue(receipt['stalled']);self.assertEqual(receipt['last_observed_turn'],1)

    def test_native_clock_advances_even_when_keys_remain_at_one_tick(self):
        script = "import time,json,os\nos.mkdir('native')\nk=open('keys.log','w');k.write('1 1 61 0\\n');k.flush()\nf=open('native/frames.jsonl','w')\nfor t in range(8):\n f.write(json.dumps({'turn':t})+'\\n'); f.flush(); time.sleep(.06)\nopen('summary.json','w').write('{}')\n"
        code, receipt = self.supervise(script)
        self.assertEqual(code, 0); self.assertFalse(receipt['stalled'])

    def test_malformed_complete_native_clock_fails_honestly(self):
        code, receipt = self.supervise("import os,time\nos.mkdir('native')\nopen('native/frames.jsonl','wb').write(b'{bad}\\n')\ntime.sleep(2)\n")
        self.assertEqual(code, 3); self.assertEqual(receipt['status'], 'supervisor_error')

    def test_advancing_world_turns_are_not_stalled(self):
        code,receipt = self.supervise("import time\nf=open('keys.log','w')\nfor t in range(8):\n f.write(f'{t} 1 61 0\\n'); f.flush(); time.sleep(.06)\nopen('summary.json','w').write('{}')\n")
        self.assertEqual(code,0);self.assertEqual(receipt['status'],'success')
        self.assertFalse(receipt['stalled'])


if __name__ == '__main__':
    unittest.main()

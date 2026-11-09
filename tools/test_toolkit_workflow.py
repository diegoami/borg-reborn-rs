"""Simulated contract controls; these are not native games or survival evidence."""
import importlib.util
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock
from contextlib import nullcontext

import borg_runtime as runtime
import borg_replay_viewer as viewer
import death_workflow as workflow
import run_report
import toolkit_common as ui

ROOT = runtime.ROOT
spec = importlib.util.spec_from_file_location('toolkit_producer', ROOT / 'produce_death.py')
producer = importlib.util.module_from_spec(spec); spec.loader.exec_module(producer)


class BatchContracts(unittest.TestCase):
    def simulate(self, outcomes, export_fail=None):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); output = root / 'collection'; output.mkdir()
            cache = root / 'cache'; cache.mkdir(); (cache / 'prepared.json').write_text('{}')
            calls, exports = [], []
            births = producer.prospective_births(list(range(100, 100 + len(outcomes))), 'random', 'fixture')
            args = SimpleNamespace(birth='random', turns=100, keys=1000, timeout=2, stall_timeout=1,
                                   checkpoint_every=100, death_issue=None, build=None)
            def game(options, **kwargs):
                calls.append(options.seed)
                b = kwargs['profile_root'] / 'recordings/run'; (b / 'run').mkdir(parents=True)
                kind = outcomes[len(calls)-1]
                process = 'timeout' if kind == 'timeout' else 'exit_failure' if kind == 'crash' else 'success'
                workflow.write(b / 'recording.json', {'status': 'timeout' if kind == 'timeout' else 'complete', 'finalized': True})
                workflow.write(b / 'run/process.json', {'status': process})
                workflow.write(b / 'run/summary.json', {'dead': kind == 'death', 'stop_reason': 'death' if kind == 'death' else 'turns',
                    'death_cause': 'synthetic foe', 'race': 'fixture', 'class': 'fixture'})
                if kind == 'crash':
                    raise ValueError('simulated crash')
                return b
            def export(bundle, **kwargs):
                exports.append(bundle)
                if len(exports) == export_fail:
                    raise ValueError('simulated export failure')
                return root / f'D{len(exports):04d}'
            with mock.patch.dict(os.environ, {'BORG_DEATH_WORKFLOW_DIR': str(root / 'artifacts')}), \
                mock.patch.object(workflow, 'locked', return_value=nullcontext()), \
                mock.patch.object(workflow, 'prepare_runtime', return_value={'cache': str(cache)}), \
                mock.patch.object(workflow, 'validate_prepared'), \
                mock.patch.object(runtime, 'record_run', side_effect=game), \
                mock.patch.object(runtime, 'validate', side_effect=lambda b, **kw: workflow.read(b / 'recording.json')), \
                mock.patch.object(producer.deaths, 'maybe_register', side_effect=export):
                try:
                    rc = producer.collect(args, output, births, 'fixture-pin')
                except ValueError:
                    rc = 1
            plan = workflow.read(output / 'collection.json')
            self.assertEqual(workflow.sha(output / 'plan.json'), plan['plan_sha256'])
            self.assertEqual(plan['seeds'], [b['seed'] for b in births])
            for entry in plan['runs']:
                self.assertTrue(Path(entry['report']).exists())
                commands = (Path(entry['directory']) / 'commands.txt').read_text()
                self.assertIn('replay-borg.py', commands); self.assertIn('play-from-replay.py', commands)
                self.assertNotIn('\033', commands)
            return rc, calls, plan, exports

    def test_ten_deaths_each_attempted_and_exported(self):
        rc, calls, p, exports = self.simulate(['death'] * 10)
        self.assertEqual((rc, len(calls), len(exports)), (0, 10, 10))
        self.assertEqual(len({e['dossier'] for e in p['runs']}), 10)
        self.assertEqual(p['counts'], {'death': 10})

    def test_mixed_caps_timeouts_deaths_continue(self):
        rc, calls, p, exports = self.simulate(['death', 'cap', 'timeout', 'death'])
        self.assertEqual((rc, len(calls), len(exports)), (0, 4, 2))
        self.assertEqual(p['counts'], {'death': 2, 'turn_cap': 1, 'wall_timeout': 1})
        self.assertIn('--view-only', p['runs'][2]['commands']['watch'])

    def test_crash_after_success_stops_retaining_prior_death(self):
        rc, calls, p, exports = self.simulate(['death', 'crash', 'death'])
        self.assertEqual((rc, len(calls), len(exports)), (1, 2, 1))
        self.assertEqual(p['status'], 'failed'); self.assertIn('dossier', p['runs'][0])

    def test_export_failure_after_success_stops(self):
        rc, calls, p, exports = self.simulate(['death', 'death', 'death'], export_fail=2)
        self.assertEqual((rc, len(calls), len(exports)), (1, 2, 2))
        self.assertEqual(p['counts'], {'death': 2}); self.assertIn('dossier', p['runs'][0])

    def test_pre_recording_failure_has_a_report_and_an_honest_missing_movie(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            entry = {'seed': 9, 'directory': str(root / 'attempt'), 'outcome': 'harness_failure', 'error': 'native startup failed'}
            with mock.patch.object(runtime, 'record_run') as game:
                producer.report_attempt(entry, None)
                game.assert_not_called()
            self.assertTrue(Path(entry['report']).exists())
            self.assertEqual(entry['commands'], {})
            self.assertIn('movie', entry['commands_unavailable'].lower())
            summary = run_report.collection_summary(root, {'runs': [entry]})
            self.assertIn(Path(entry['report']).name, summary.read_text())

    def test_check_build_missing_does_not_build_or_ask(self):
        args = SimpleNamespace(check_build=True, no_build=False, yes=False)
        with mock.patch.object(workflow, 'matching_runtime', return_value=None), mock.patch.object(workflow, 'compatible_engine', return_value=None), mock.patch('builtins.input') as ask:
            self.assertFalse(workflow.build_offer(args, 'pin', {})); ask.assert_not_called()

    def test_no_color_in_redirected_stored_output(self):
        with mock.patch.object(sys.stdout, 'isatty', return_value=False):
            self.assertEqual(ui.color('command', '36'), 'command')
        with mock.patch.object(sys.stdout, 'isatty', return_value=True), mock.patch.dict(os.environ, {'NO_COLOR': ''}):
            self.assertEqual(ui.color('command', '36'), 'command')


class Movies(unittest.TestCase):
    def movie(self, root, width=80, height=24, **kw):
        row = dict(index=1, turn=1, phase='command', full=True, width=width, height=height,
                   hp=10, sp=0, level=1, depth=0, cells=[[0, 64, 1]], palette=None)
        row.update(kw)
        (root / 'frames.jsonl').write_text(json.dumps(row) + '\n')

    def test_real_wide_dimensions_are_accepted(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.movie(root, width=600, height=30)
            f = viewer.Frames(root)
            self.assertEqual(len(f.get(0)[1]), 18000); f.close()

    def test_zero_oversized_geometry_and_cells_are_rejected(self):
        for options in [dict(width=0), dict(width=4097), dict(width=4096, height=4096), dict(cells=[[-1, 65, 1]]),
                        dict(cells=[[0, 65, 1], [0, 66, 1]]), dict(index=2)]:
            with self.subTest(options=options), tempfile.TemporaryDirectory() as temp:
                root = Path(temp); self.movie(root, **options)
                with self.assertRaises(ValueError): viewer.Frames(root)

    def test_finalized_malformed_tail_is_not_hidden(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.movie(root)
            with (root / 'frames.jsonl').open('a') as out: out.write('{bad')
            with self.assertRaisesRegex(ValueError, 'Malformed'): viewer.Frames(root)

    def test_combat_cutoff_does_not_expose_later_same_tick_damage(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.movie(root)
            events = [dict(kind='message', frame=2, turn=1, type='HIT', text='A foe hits you.'),
                      dict(kind='damage', frame=2, turn=1, before=10, after=7, cause='a foe', requested=3)]
            (root / 'combat.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
            f = viewer.Frames(root); log = viewer.ReplayLog(root, f)
            row = f.get(0)[0]
            hidden = log.window(row, 0, 20, 140, mode=3)[0]
            self.assertNotIn('loss 3', str(hidden))
            shown = log.window({**row, 'index': 2}, 1, 20, 140, mode=3)[0]
            self.assertEqual(str(shown).count('A foe hits you.'), 1)
            self.assertIn('loss 3', str(shown)); f.close()


class SavedCounters(unittest.TestCase):
    def save_bytes(self, version=1):
        def block(name, data):
            return struct.pack('<16sIII', name.encode(), version, len(data), sum(data)&0xffffffff)+data+b'\0'*(-len(data)%4)
        p=b'Measure\0alive\0history\0Human\0normal\0Warrior\0'+bytes(9)+b'\x06'+bytes(58)+b'Humanoid\0'+struct.pack('<H',0)+bytes(4)
        p+=struct.pack('<IIIHhhhHhhHhh',47,120,110,0,3,30,25,0,10,8,0,3,2)
        p+=bytes(16)+b'\0'+struct.pack('<II',10000,400)
        lore=b'\x01orc\0'+struct.pack('<HH',3,0)+b'No more monsters\0'
        misc=struct.pack('<IIHHBi',0,0,0,0,0,1000)
        return b'SaveVNLA'+block('player',p)+block('misc',misc)+block('monster memory',lore)

    def test_saved_counters_and_unsupported_checksums_versions(self):
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'save'; path.write_bytes(self.save_bytes())
            info=run_report.saved_character(path)
            self.assertEqual((info['recorded_kills'], info['resting_turns'], info['energy'], info['turn']), (3,400,10000,1000))
            for data in (b'bad', self.save_bytes(version=2), self.save_bytes()[:-1]):
                path.write_bytes(data)
                with self.assertRaises(ValueError): run_report.saved_character(path)
            data=bytearray(self.save_bytes()); data[40]^=1; path.write_bytes(data)
            with self.assertRaises(ValueError): run_report.saved_character(path)


if __name__ == '__main__': unittest.main()

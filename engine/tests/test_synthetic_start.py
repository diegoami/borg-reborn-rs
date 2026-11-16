"""Real generated-depth starts; run after harness/build.sh (also in engine CI)."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / 'build/rust/game/angband'


@unittest.skipUnless(BIN.is_file(), 'harness/build.sh required')
class SyntheticStartTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='synthetic-start-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
        self.env.update(BORG_CEMETERY_DIR=os.environ.get('BORG_CEMETERY_DIR', str(ROOT / 'vendor/borg-cemetery')),
                        BORG_DEATH_RECORDS_DIR=str(self.root / 'natural'), RUN_TIMEOUT_SECONDS='30')

    def run_game(self, name, flags='menu,start-depth=10', extra=(), core='reborn', turns=5000):
        out = self.root / name
        result = subprocess.run([str(ROOT / 'harness/run.sh'), str(out), '101', '0', '0',
                                 str(turns), flags, '--', *extra],
                                env=dict(self.env, BORG_CORE=core), capture_output=True, text=True, timeout=40)
        return out, result

    def check_start(self, out, result, depth):
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads((out / 'summary.json').read_text())
        self.assertTrue(summary['synthetic'])
        self.assertFalse(summary['loaded'])
        self.assertEqual((summary['start_depth'], summary['start_build']), (depth, 'birth'))
        self.assertIn(summary['stop_reason'], ('death', 'turns'))
        self.assertGreater(summary['keys'], 0)
        self.assertGreater(summary['game_turns'], 1)
        config = dict(line.split('=', 1) for line in (out / 'config.txt').read_text().splitlines())
        self.assertEqual((config['synthetic'], config['start_depth'], config['start_build']),
                         ('1', str(depth), 'birth'))
        states = (out / 'state.log').read_text().splitlines()
        self.assertRegex(states[0], rf' d{depth}/{depth} ')
        self.assertTrue(all(int(re.search(r' d(\d+)/', line)[1]) >= depth for line in states))
        messages = (out / 'messages.log').read_text()
        self.assertNotIn('yanked upwards', messages)
        self.assertNotIn('air about you becomes charged', messages)
        self.assertTrue(summary['game_options']['birth_force_descend'])
        self.assertTrue(summary['game_options']['birth_no_recall'])
        self.assertFalse((self.root / 'natural').exists())
        return summary

    def test_depth_10_20_and_repeat_across_environment_changes(self):
        for depth in (10, 20):
            out, result = self.run_game(f'd{depth}', f'menu,start-depth={depth}')
            summary = self.check_start(out, result, depth)
            # At least one actual synthetic death exercises automatic export exclusion.
            if depth == 20:
                self.assertTrue(summary['dead'], summary)
            self.env.update(TZ='Pacific/Honolulu', LANG='C', LC_ALL='C')
            again, result = self.run_game(f'd{depth}-repeat', 'menu', ('-start-depth', str(depth), '-start-build', 'birth'))
            self.check_start(again, result, depth)
            for stream in ('keys.log', 'state.log', 'messages.log'):
                self.assertEqual((out / stream).read_bytes(), (again / stream).read_bytes(), stream)

    def test_vendored_core_accepts_only_the_synthetic_options(self):
        out, result = self.run_game('core', core='borg-rs', turns=300)
        self.check_start(out, result, 10)
        self.assertNotIn('STARTUP FAILURE', (out / 'messages.log').read_text())

    def test_invalid_inputs_and_load_conflicts_fail(self):
        save = self.root / 'unused.save'
        save.write_bytes(b'not loaded')
        self.env['SAVE_ROOT'] = str(self.root)
        cases = [('', ('-start-depth',)),
                 ('start-depth=', ()), ('start-depth=0', ()), ('start-depth=-1', ()),
                 ('start-depth=abc', ()), ('start-depth=10x', ()),
                 ('start-depth=10,start-depth=20', ()),
                 ('start-depth=10', ('-start-depth', '20')),
                 ('start-depth=10,start-build=advanced', ()),
                 ('start-build=birth', ()),
                 ('start-depth=10', ('-load', str(save))),
                 ('start-depth=10,save=unused.save', ()),
                 ('menu', ('-start-depth', '128')),
                 ('menu', ('-start-depth', '0')),
                 ('menu', ('-start-depth', '10x')),
                 ('menu', ('-start-depth', '9999999999999999999999999999')),
                 ('menu', ('-start-depth', '10', '-start-build', 'kit:foo'))]
        for i, (flags, extra) in enumerate(cases):
            with self.subTest(flags=flags, extra=extra):
                out, result = self.run_game(f'invalid{i}', flags or '-', extra)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((out / 'summary.json').exists())
                self.assertFalse((self.root / 'natural').exists())

    def test_frontend_itself_rejects_malformed_or_conflicting_options(self):
        for i, options in enumerate([
            ('-start-depth', '0'), ('-start-depth', '-1'),
            ('-start-depth', '10junk'), ('-start-depth', '01'),
            ('-start-depth', '999999999999999999999999999999'),
            ('-start-depth', '128'), ('-start-depth',),
            ('-start-depth', '10', '-start-depth', '20'),
            ('-start-depth', '10', '-load', '/missing.save'),
            ('-start-depth', '10', '-start-build', 'advanced'),
            ('-start-build', 'birth'),
        ]):
            with self.subTest(options=options):
                out = self.root / f'direct{i}'
                out.mkdir()
                dirs = []
                for kind in ('user', 'save', 'archive', 'panic', 'scores'):
                    (out / kind).mkdir()
                    dirs.append(f'-d{kind}={out / kind}')
                summary = out / 'summary.json'
                result = subprocess.run([str(BIN), '-mborgh', '-n', '-uBorg', *dirs, '--',
                                         '-seed', '101', '-race', '0', '-class', '0',
                                         '-summary', str(summary), *options],
                                        cwd=BIN.parent, env=self.env, capture_output=True, text=True, timeout=20)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('borgh:', result.stderr)
                self.assertFalse(summary.exists())

    def test_a_saved_synthetic_character_cannot_be_relabelled_as_natural(self):
        out, result = self.run_game('save', 'menu,start-depth=10,finalsave', turns=2)
        self.check_start(out, result, 10)
        save = out / 'final.save'
        self.assertTrue(save.is_file())
        loaded, result = self.run_game('reload', 'menu', ('-load', str(save)))
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((loaded / 'summary.json').exists())
        self.assertIn('synthetic one-way save', (loaded / 'stderr.log').read_text())

    def test_direct_export_of_a_synthetic_death_is_rejected(self):
        out, result = self.run_game('death', 'menu,start-depth=20')
        summary = self.check_start(out, result, 20)
        self.assertTrue(summary['dead'])
        result = subprocess.run(['python3', str(ROOT / 'tools/export-death.py'), str(out),
                                 '--impl', 'rust', '--build-id', str(ROOT / 'build/rust/BUILD_ID')],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('not natural: synthetic', result.stderr)
        self.assertFalse((self.root / 'natural').exists())

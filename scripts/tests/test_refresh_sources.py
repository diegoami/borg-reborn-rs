"""Tests for scripts/refresh-sources.py on synthetic git repositories.

Run: python3 -m unittest discover -s scripts/tests
"""

import importlib.util
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('refresh_sources', HERE.parent / 'refresh-sources.py')
rs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rs)

ENV = dict(os.environ, GIT_AUTHOR_NAME='t', GIT_AUTHOR_EMAIL='t@t', GIT_COMMITTER_NAME='t',
           GIT_COMMITTER_EMAIL='t@t', GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1')


def sh(cwd, *args):
    return subprocess.run(args, cwd=cwd, env=ENV, capture_output=True, text=True, check=True).stdout.strip()


def write(base, files):
    for rel, text in files.items():
        p = Path(base) / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)


def commit(repo, files, msg):
    write(repo, files)
    sh(repo, 'git', 'add', '-A')
    sh(repo, 'git', 'commit', '-q', '-m', msg)
    return sh(repo, 'git', 'rev-parse', 'HEAD')


DANGER_A = 'int borg_danger(int y)\n{\n    return 1;\n}\n\nint other(void)\n{\n    return 0;\n}\n'
DANGER_B = DANGER_A.replace('return 1;', 'return 2;')


class RefreshTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        self.origin = t / 'origin' / 'src'
        self.origin.mkdir(parents=True)
        sh(self.origin, 'git', 'init', '-q', '-b', 'main')
        self.a = commit(self.origin, {
            'docs/deaths/README.md': 'registry\n',
            'docs/deaths/index.json': '{}\n',
            'docs/deaths/D0001-a/report.md': '# D0001 first death\n',
            'docs/deaths/D0001-a/record.json': json.dumps({'character': 'Human Warrior', 'raw_cause': 'a kobold',
                                                           'encounter_family': 'start-1'}),
            'docs/deaths/D0001-a/assessment.json': json.dumps({'classification': 'unassessed'}),
            'docs/deaths/D0001-a/snapshots/frame.txt': 'big\n',
            'src/borg/borg-danger.c': DANGER_A,
        }, 'A')
        self.root = t / 'project'
        (self.root / 'vendor').mkdir(parents=True)
        sh(t, 'git', 'clone', '-q', str(self.origin), str(self.root / 'vendor' / 'src'))
        sh(self.root / 'vendor' / 'src', 'git', 'checkout', '-q', '--detach', self.a)
        write(self.root, {
            'pins.json': json.dumps({'pins': {'src': {
                'repository': 'someone/src', 'commit': self.a, 'read_on': '2026-11-08',
                'watch_tags': '4.2.6-impact-v*', 'borg_dir': 'src/borg'}},
                'copies': [{'pin': 'src', 'from': 'docs/deaths', 'to': 'reference/deaths/better-borg-c',
                            'select': 'death-registry'}]}),
            'reference/README.md': f'copied at {self.a}\n',
            'docs/notes.md': 'We lifted borg-danger.c:borg_danger:1-4 and nothing else.\n',
        })
        rs.update(self.root, ['src'], fetch=False, today='2026-11-08')  # copies at A
        self.b = commit(self.origin, {
            'docs/deaths/D0001-a/report.md': '# D0001 first death, reassessed\n',
            'docs/deaths/D0002-b/report.md': '# D0002 second death\n',
            'docs/deaths/D0002-b/record.json': json.dumps({'character': 'Gnome Mage', 'raw_cause': 'a soldier'}),
            'docs/deaths/D0002-b/assessment.json': json.dumps({'classification': 'unassessed'}),
            'docs/deaths/LESSONS.md': '# lessons for a redesign\n',
            'docs/deaths/README.md': 'registry, updated\n',
            'src/borg/borg-danger.c': DANGER_B,
        }, 'B')
        sh(self.origin, 'git', 'tag', '4.2.6-impact-v8')

    def tearDown(self):
        self.tmp.cleanup()

    def report(self, **kw):
        out = io.StringIO()
        rs.report(self.root, ['src'], issues=False, out=out, **kw)
        return out.getvalue()

    def test_report_lists_what_moved(self):
        text = self.report()
        self.assertIn('**1 commits** since pin', text)
        self.assertIn('4 added, 2 changed, 0 removed', text)
        self.assertIn('**new registry documents:** LESSONS.md', text)
        self.assertIn('changed registry documents: README.md', text)
        self.assertIn('**new death cases:** D0002-b', text)
        self.assertIn('updated cases: D0001-a', text)
        self.assertIn('**new tags** `4.2.6-impact-v*`: 4.2.6-impact-v8', text)
        self.assertIn('**cited C Borg functions changed:** borg-danger.c:borg_danger', text)

    def test_engine_tree_resync_is_flagged(self):
        pins = json.loads((self.root / 'pins.json').read_text())
        tree_a = sh(self.root / 'vendor/src', 'git', 'rev-parse', f'{self.a}:src/borg')
        pins['engine_tree'] = {'pin': 'src', 'path': 'src/borg', 'tree': tree_a}
        (self.root / 'pins.json').write_text(json.dumps(pins))
        self.assertIn('**engine tree re-sync available:** `src/borg/`', self.report())
        rs.update(self.root, ['src'], today='2026-11-09')
        pins = json.loads((self.root / 'pins.json').read_text())
        self.assertEqual(pins['engine_tree']['tree'], tree_a, 'update never re-syncs the engine tree')
        self.assertIn('re-sync available', self.report(), 'still flagged until the re-sync PR')

    def test_report_is_read_only(self):
        before = (self.root / 'pins.json').read_text()
        self.report()
        self.assertEqual(before, (self.root / 'pins.json').read_text())
        self.assertFalse((self.root / 'reference/deaths/better-borg-c/D0002-b').exists())

    def test_update_moves_pin_copies_and_index(self):
        moved = rs.update(self.root, ['src'], today='2026-11-09')
        self.assertEqual(moved, [('src', self.a, self.b)])
        pin = json.loads((self.root / 'pins.json').read_text())['pins']['src']
        self.assertEqual((pin['commit'], pin['read_on']), (self.b, '2026-11-09'))
        ref = self.root / 'reference/deaths/better-borg-c'
        self.assertTrue((ref / 'D0002-b/report.md').exists())
        self.assertIn('reassessed', (ref / 'D0001-a/report.md').read_text())
        self.assertFalse((ref / 'D0001-a/snapshots').exists(), 'case subdirectories are not copied')
        self.assertEqual(sh(self.root / 'vendor/src', 'git', 'rev-parse', 'HEAD'), self.b)
        self.assertIn(self.b, (self.root / 'reference/README.md').read_text())
        index = (self.root / rs.INDEX).read_text()
        self.assertIn('D0002-b', index)
        self.assertIn('a soldier', index)
        self.assertIn('2 cases.', index)
        self.assertTrue((ref / 'LESSONS.md').exists())
        self.assertIn('at pin', self.report(), 'after the update the pin is current')

    def test_failed_update_changes_nothing(self):
        pins = json.loads((self.root / 'pins.json').read_text())
        pins['pins']['broken'] = {'repository': 'someone/missing', 'commit': self.a, 'read_on': '2026-11-08'}
        (self.root / 'pins.json').write_text(json.dumps(pins))
        before = {p: p.read_bytes() for p in (self.root / 'reference').rglob('*') if p.is_file()}
        pins_before = (self.root / 'pins.json').read_bytes()
        with self.assertRaises(SystemExit):
            rs.update(self.root, ['src', 'broken'], today='2026-11-09')
        after = {p: p.read_bytes() for p in (self.root / 'reference').rglob('*') if p.is_file()}
        self.assertEqual(before, after, 'copies changed although the update failed')
        self.assertEqual((self.root / 'pins.json').read_bytes(), pins_before)
        self.assertEqual(sh(self.root / 'vendor/src', 'git', 'rev-parse', 'HEAD'), self.a)

    def test_dirty_vendor_is_not_moved(self):
        (self.root / 'vendor/src/local.txt').write_text('mine\n')
        rs.update(self.root, ['src'], today='2026-11-09')
        self.assertEqual(sh(self.root / 'vendor/src', 'git', 'rev-parse', 'HEAD'), self.a)

    def test_offline_is_reported_not_fatal(self):
        sh(self.root / 'vendor/src', 'git', 'remote', 'set-url', 'origin', str(self.root / 'nowhere'))
        text = self.report()
        self.assertIn('fetch failed, using what is already fetched', text)

    def test_branch_pin_follows_its_branch_in_its_own_clone(self):
        sh(self.origin, 'git', 'checkout', '-q', '-b', 'manual', self.a)
        c = commit(self.origin, {'docs/strategy/README.md': '# manual\n'}, 'C')
        sh(self.origin, 'git', 'checkout', '-q', 'main')
        sh(self.root, 'git', 'clone', '-q', str(self.origin), str(self.root / 'vendor' / 'src-manual'))
        pins = json.loads((self.root / 'pins.json').read_text())
        pins['pins']['manual'] = {'repository': 'someone/src', 'branch': 'manual', 'vendor': 'src-manual',
                                  'commit': self.a, 'read_on': '2026-11-08'}
        pins['copies'].append({'pin': 'manual', 'from': 'docs/strategy', 'to': 'reference/strategy',
                               'select': 'all'})
        (self.root / 'pins.json').write_text(json.dumps(pins))
        out = io.StringIO()
        rs.report(self.root, ['manual'], issues=False, out=out)
        self.assertIn('## manual (someone/src, branch manual)', out.getvalue())
        self.assertIn('**1 commits** since pin', out.getvalue())
        self.assertIn('copy `docs/strategy` → `reference/strategy`: 1 added', out.getvalue())
        self.assertEqual(rs.update(self.root, ['manual'], today='2026-11-09'), [('manual', self.a, c)])
        self.assertEqual((self.root / 'reference/strategy/README.md').read_text(), '# manual\n')
        self.assertEqual(sh(self.root / 'vendor/src-manual', 'git', 'rev-parse', 'HEAD'), c)
        self.assertEqual(sh(self.root / 'vendor/src', 'git', 'rev-parse', 'HEAD'), self.a,
                         'the default-branch clone is not touched')

    def test_no_fetch_uses_local_state(self):
        text = self.report(fetch=False)
        self.assertIn('nothing new', text)


if __name__ == '__main__':
    unittest.main()

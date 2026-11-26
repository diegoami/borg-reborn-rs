"""#518 / #535: harness/run.sh selects, copies and pins the scary list.

Precedence (review of #535): BORG_SCARY_LIST=FILE wins, under its own file
name; otherwise the list the borg.txt names is copied from harness/; with
borg_scary_list: "none" the compiled C list runs, with no file and no hash.
Each case inspects the run's user/pborg.yaml, the copied files and config.txt,
and plays a short borg-rs game, which stops before play on a missing or
invalid list.
"""
import hashlib
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / 'build/rust/game/angband'
SHIPPED = 'scary_monsters.txt'
C_LIST = 'scary_monsters_c.txt'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


@unittest.skipUnless(GAME.exists(), 'no build (harness/build.sh)')
class ScaryListHarnessTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def run_sh(self, name, **extra):
        env = {k: v for k, v in os.environ.items() if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
        if 'BORG_CEMETERY_DIR' in os.environ:
            env['BORG_CEMETERY_DIR'] = os.environ['BORG_CEMETERY_DIR']
        env['BORG_DEATH_RECORDS_DIR'] = str(self.root / 'deaths')
        env['BORG_CORE'] = 'borg-rs'
        env.update(extra)
        out = self.root / name
        r = subprocess.run([str(ROOT / 'harness/run.sh'), str(out), '29', '0', '0', '200', 'menu'],
                           env=env, capture_output=True, text=True, timeout=120)
        config = dict(l.split('=', 1) for l in (out / 'config.txt').read_text().splitlines() if '=' in l)
        borg_txt = (out / 'user/pborg.yaml').read_text()
        lines = [l for l in borg_txt.splitlines() if l.partition(':')[0].strip() == 'borg_scary_list']
        return r, out, config, borg_txt, lines

    def assert_played(self, r, out):
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        self.assertTrue((out / 'keys.log').read_text().strip(), 'the game played')

    def test_the_shipped_list_is_copied_and_pinned(self):
        r, out, config, borg_txt, lines = self.run_sh('shipped')
        self.assert_played(r, out)
        digest = sha(ROOT / 'harness' / SHIPPED)
        self.assertEqual(lines, [f'borg_scary_list: "{SHIPPED}"'])
        self.assertIn(f'scary list sha256 {digest}\nborg_scary_list: "{SHIPPED}"\n', borg_txt)
        self.assertEqual((out / 'user' / SHIPPED).read_bytes(), (ROOT / 'harness' / SHIPPED).read_bytes())
        self.assertEqual((config['borg_scary_list'], config['borg_scary_list_sha256']), (SHIPPED, digest))

    def test_borg_scary_list_wins_under_its_own_name(self):
        r, out, config, borg_txt, lines = self.run_sh('override', BORG_SCARY_LIST=str(ROOT / 'harness' / C_LIST))
        self.assert_played(r, out)
        digest = sha(ROOT / 'harness' / C_LIST)
        self.assertEqual(lines, [f'borg_scary_list: "{C_LIST}"'])
        self.assertIn(f'scary list sha256 {digest}\nborg_scary_list: "{C_LIST}"\n', borg_txt)
        self.assertEqual((out / 'user' / C_LIST).read_bytes(), (ROOT / 'harness' / C_LIST).read_bytes())
        self.assertFalse((out / 'user' / SHIPPED).exists(), 'only the selected list is copied')
        self.assertEqual((config['borg_scary_list'], config['borg_scary_list_sha256']), (C_LIST, digest))

    def test_none_runs_the_compiled_list_with_no_file_and_no_hash(self):
        r, out, config, borg_txt, lines = self.run_sh('none', BORG_TXT_SET='borg_scary_list=none')
        self.assert_played(r, out)
        self.assertEqual(lines, ['borg_scary_list: "none"'])
        self.assertNotIn('scary list sha256', borg_txt)
        self.assertEqual([p.name for p in (out / 'user').glob('scary_monsters*')], [])
        self.assertEqual(config['borg_scary_list'], 'none')
        self.assertNotIn('borg_scary_list_sha256', config)

    def test_borg_scary_list_wins_over_none(self):
        r, out, config, _, lines = self.run_sh('none-override', BORG_TXT_SET='borg_scary_list=none',
                                               BORG_SCARY_LIST=str(ROOT / 'harness' / C_LIST))
        self.assert_played(r, out)
        self.assertEqual(lines, [f'borg_scary_list: "{C_LIST}"'])
        self.assertEqual(config['borg_scary_list_sha256'], sha(ROOT / 'harness' / C_LIST))

    def test_an_invalid_list_stops_before_play(self):
        bad = self.root / 'bad.txt'
        bad.write_text((ROOT / 'harness' / SHIPPED).read_text().replace('soldier | ', 'soldiers | ', 1))
        r, out, config, _, lines = self.run_sh('invalid', BORG_SCARY_LIST=str(bad))
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(lines, ['borg_scary_list: "bad.txt"'])
        self.assertIn("unknown monster 'soldiers'", (out / 'stderr.log').read_text())
        self.assertFalse((out / 'keys.log').exists() and (out / 'keys.log').read_text().strip())


if __name__ == '__main__':
    unittest.main()

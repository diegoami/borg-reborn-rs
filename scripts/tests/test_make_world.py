"""Tests for scripts/make-world.py (#704) on a small synthetic lib/.

Run: python3 -m unittest discover -s scripts/tests
"""

import importlib.util
import io
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('make_world', HERE.parent / 'make-world.py')
mw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mw)

MONSTERS = '''# header
name:soldier
depth:2
rarity:1
flags:MALE

name:white worm mass
depth:1
rarity:1
flags:MULTIPLY | NEVER_BLOW

name:yellow worm mass
depth:3
rarity:2
flags:MULTIPLY

name:giant white mouse
depth:1
rarity:1
flags:RAND_50 | MULTIPLY
'''


def field(text, name, key):
    rec = [r for r in mw.records(text) if r[0].strip() == f'name:{name}'][0]
    return int([line for line in rec if line.startswith(key + ':')][0].split(':')[1])


class MakeWorldTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.lib = Path(self.tmp.name) / 'lib'
        for d in mw.DIRS:
            (self.lib / d).mkdir(parents=True)
        (self.lib / 'gamedata' / 'monster.txt').write_text(MONSTERS)
        (self.lib / 'gamedata' / 'constants.txt').write_text('mon-gen:repro-max:100\nworld:max-depth:128\n')
        (self.lib / 'gamedata' / 'object.txt').write_text('untouched\n')

    def tearDown(self):
        self.tmp.cleanup()

    def make(self, config):
        cfg = Path(self.tmp.name) / 'w.txt'
        cfg.write_text(config)
        out = Path(self.tmp.name) / 'out'
        so, se = io.StringIO(), io.StringIO()
        with redirect_stdout(so), redirect_stderr(se):
            mw.main(['--config', str(cfg), '--lib', str(self.lib), '--out', str(out)])
        return out, so.getvalue().strip()

    def test_breeders_are_favoured_against_the_rest(self):
        out, sha = self.make('breeder_weight = 4\nbreeder_depth_shift = 2\nrepro_max = 50\n')
        m = (out / 'gamedata' / 'monster.txt').read_text()
        self.assertEqual(field(m, 'soldier', 'rarity'), 4, 'a non-breeder is rarer')
        self.assertEqual(field(m, 'white worm mass', 'rarity'), 1, 'a breeder keeps its rarity')
        self.assertEqual(field(m, 'yellow worm mass', 'depth'), 1, 'shallower by 2')
        self.assertEqual(field(m, 'white worm mass', 'depth'), 1, 'never shallower than 1')
        self.assertIn('mon-gen:repro-max:50', (out / 'gamedata' / 'constants.txt').read_text())
        self.assertEqual((out / 'gamedata' / 'object.txt').read_text(), 'untouched\n')
        self.assertEqual(len(sha), 64)
        self.assertTrue(all((out / d).is_dir() for d in mw.DIRS))

    def test_only_and_except_choose_the_breeders(self):
        out, _ = self.make('breeder_weight = 2\nbreeders_only = worm\nbreeders_except = yellow\n')
        m = (out / 'gamedata' / 'monster.txt').read_text()
        self.assertEqual(field(m, 'white worm mass', 'rarity'), 1, 'a favoured worm')
        self.assertEqual(field(m, 'yellow worm mass', 'rarity'), 4, 'excepted: weighted like the rest')
        self.assertEqual(field(m, 'giant white mouse', 'rarity'), 2, 'not a worm: weighted like the rest')

    def test_no_setting_is_the_games_own_data(self):
        out, sha = self.make('# nothing\n')
        self.assertEqual((out / 'gamedata' / 'monster.txt').read_text(), MONSTERS)
        out2 = Path(self.tmp.name) / 'out2'
        with redirect_stdout(io.StringIO()) as so, redirect_stderr(io.StringIO()):
            mw.main(['--config', str(Path(self.tmp.name) / 'w.txt'), '--lib', str(self.lib), '--out', str(out2)])
        self.assertEqual(so.getvalue().strip(), sha, 'the data hash is deterministic')

    def test_refusals(self):
        with self.assertRaises(SystemExit):
            self.make('breeder_wieght = 4\n')
        with self.assertRaises(SystemExit):
            self.make('breeder_weight = 101\n')  # 1 x 101 > 100: never generated


if __name__ == '__main__':
    unittest.main()

"""#826: one helper prepares a game's borg.txt for watched games, town sessions and packages."""
import hashlib
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / 'harness/prepare-borg-txt.sh'


class PrepareBorgTxtTest(unittest.TestCase):
    def test_prepares_as_run_sh_does(self):
        with tempfile.TemporaryDirectory() as d:
            subprocess.run([str(HELPER), d], check=True)
            text = (Path(d) / 'borg.txt').read_text()
            shipped = (ROOT / 'harness/borg.txt').read_text()
            self.assertNotRegex(text, r'(?m)^\s*borg_world\s*=')
            self.assertRegex(text, r'(?m)^borg_reborn_core = borg-rs$')
            scary = hashlib.sha256((ROOT / 'harness/scary_monsters.txt').read_bytes()).hexdigest()
            self.assertTrue(text.endswith(f'\n# harness/run.sh (#518): scary list sha256 {scary}\n'
                                          'borg_scary_list = scary_monsters.txt\n'))
            self.assertEqual((Path(d) / 'scary_monsters.txt').read_bytes(),
                             (ROOT / 'harness/scary_monsters.txt').read_bytes())
            # Every other line is the shipped file's, in order.
            kept = [l for l in shipped.splitlines() if not l.lstrip().startswith(('borg_world', 'borg_scary_list'))]
            self.assertEqual(text.splitlines()[:len(kept)], kept)

    def test_watch_borg_restores_a_missing_scary_list(self):
        # Review of #866: a kept borg.txt naming a list that is not there
        # made the core exit at start.
        text = (ROOT / 'watch-borg.sh').read_text()
        self.assertIn('for f in "$SHIPPED"/*; do', text)
        self.assertIn('names the scary list $LIST, which is not in', text)

    def test_every_launcher_uses_it(self):
        for f in ('watch-borg.sh', 'harness/play.sh', 'scripts/release/package.sh'):
            self.assertIn('harness/prepare-borg-txt.sh', (ROOT / f).read_text(), f)


if __name__ == '__main__':
    unittest.main()

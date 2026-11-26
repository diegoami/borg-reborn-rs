"""#826: one helper prepares a game's pborg.yaml for watched games, town sessions and packages."""
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / 'harness/prepare-pborg-config.sh'


class PreparePBorgConfigTest(unittest.TestCase):
    def test_copies_the_shipped_yaml_and_its_scary_list(self):
        with tempfile.TemporaryDirectory() as d:
            subprocess.run([str(HELPER), d], check=True)
            text = (Path(d) / 'pborg.yaml').read_text()
            self.assertNotRegex(text, r'(?m)^\s*borg_world\s*:')
            self.assertRegex(text, r'(?m)^borg_reborn_core: "borg-rs"$')
            self.assertEqual(text, (ROOT / 'harness/pborg.yaml').read_text())
            self.assertEqual((Path(d) / 'scary_monsters.txt').read_bytes(),
                             (ROOT / 'harness/scary_monsters.txt').read_bytes())
            self.assertEqual({p.name for p in Path(d).iterdir()}, {'pborg.yaml', 'scary_monsters.txt'})
            self.assertRegex(text, r'(?m)^borg_param_\w+__mage: -?\d+$')

    def test_watch_borg_restores_a_missing_scary_list(self):
        # Review of #866: a kept pborg.yaml naming a list that is not there
        # made the core exit at start.
        text = (ROOT / 'watch-borg.sh').read_text()
        self.assertIn('for f in "$SHIPPED"/*; do', text)

    def test_every_launcher_uses_it(self):
        for f in ('watch-borg.sh', 'harness/play.sh', 'scripts/release/package.sh'):
            self.assertIn('harness/prepare-pborg-config.sh', (ROOT / f).read_text(), f)


if __name__ == '__main__':
    unittest.main()

"""A full native pack gives up junk before exploration supplies."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"
FIXTURE = ROOT / "engine/tests/fixtures/exploration-full-pack.build"


@unittest.skipUnless(GAME.is_file(), "native build required")
class ExplorationPackTest(unittest.TestCase):
    def run_case(self, harmful):
        with tempfile.TemporaryDirectory(prefix="exploration-pack-") as directory:
            root = Path(directory)
            paths = {name: root / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            for name in ("borg.txt", "scary_monsters.txt"):
                shutil.copyfile(ROOT / "harness" / name, paths["user"] / name)
            build = root / "full.build"
            kit = FIXTURE.read_text()
            if not harmful:
                kit = kit.replace("item:26:34:4:", "item:26:30:4:")
            build.write_text(kit)
            messages = root / "messages.log"
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_PARAM_JUNK_EXPLORATION_FIRST="1",
                       BORG_PARAM_JUNK_INFERIOR_SPARES="0", BORG_PARAM_TOWN_SHOP_OVERLOADED="1",
                       BORG_PARAM_ATTACK_THROW_JUNK_POTIONS="1",
                       BORG_PARAM_ITEM_USE_IDLE="0")
            result = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-start-depth", "3",
                 "-start-build", str(build), "-turns", "120", "-msglog", str(messages)],
                env=env, cwd=GAME.parent, capture_output=True, text=True, timeout=30,
            )
            log = messages.read_text()
            self.assertEqual(result.returncode, 0, result.stderr + log[-4000:])
            self.assertNotRegex(log, r"You drop [^\n]*Word of Recall")
            return log[-6000:]

    def test_drops_known_harmful_potions_before_usable_supplies(self):
        log = self.run_case(harmful=True)
        self.assertIn("You drop 4 Potions of Sleep {ignore}", log)

    def test_drops_outclassed_weapon_before_usable_supplies(self):
        log = self.run_case(harmful=False)
        self.assertIn("# Dropping inferior spare to make pack room Trident", log)
        self.assertRegex(log, r"You drop [^\n]*Trident[^\n]*\{borg ignore")


if __name__ == "__main__":
    unittest.main()

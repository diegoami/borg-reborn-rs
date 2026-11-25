"""Run the constructed #1239 Priest kit through native town commands."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"


@unittest.skipUnless(GAME.is_file(), "test-town-loadout build required")
class TownLoadoutTest(unittest.TestCase):
    def run_case(self, case):
        with tempfile.TemporaryDirectory(prefix="town-loadout-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            for name in ("borg.txt", "scary_monsters.txt"):
                shutil.copyfile(ROOT / "harness" / name, paths["user"] / name)
            messages = Path(directory) / "messages.log"
            env = {
                key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))
            }
            env.update(
                BORG_CORE="borg-rs",
                BORG_TEST_TOWN_LOADOUT=case,
                BORG_PARAM_TOWN_LOADOUT_POWER_LOSS="3000",
                BORG_PARAM_TOWN_QUIVER_TRIM_RESERVE__PRIEST="20" if case == "trim" else "0",
                BORG_PARAM_TOWN_RECALL_URGENT_DEPTH="5",
                BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1",
                BORG_PARAM_CAUTION_SCARY_NAMED_MONSTERS="0",
                BORG_PARAM_CAUTION_SCARY_DAMAGE_PERCENT="0",
            )
            env["BORG_PARAM_TOWN_LIGHT_RESERVE_TURNS__PRIEST"] = "0"
            env["BORG_PARAM_TOWN_STAPLE_OIL__PRIEST"] = "0"
            result = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "3", "-menu-letters",
                 "-msglog", str(messages), "-keys", "750"],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            log = messages.read_text()
            self.assertEqual(result.returncode, 0, result.stderr + log[-4000:])
            self.assertIn(f"#1239 town loadout passed: {case}", result.stdout, log[-6000:])
        return log

    def test_trim_and_full_home_swap_store_the_mace(self):
        log = self.run_case("trim")
        self.assertLess(log.index("# Town headroom dropping"), log.index("# Town loadout: preparing a lighter kit"))
        self.assertLess(log.index("# Town loadout: preparing a lighter kit"), log.index("# Town loadout: displaced gear stored at Home"))

    def test_full_home_without_storage_has_no_safe_proposal(self):
        self.run_case("no-storage")

    def test_heavier_home_upgrade_can_be_proposed_after_a_strength_gain(self):
        self.run_case("restore")


if __name__ == "__main__":
    unittest.main()

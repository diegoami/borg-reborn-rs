"""Exercise the shipped immobile decisions on fabricated Borg observations."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"


@unittest.skipUnless(GAME.is_file(), "test-stair-handoff build required")
class ImmobileDecisionTest(unittest.TestCase):
    def run_case(self, case, override=None):
        with tempfile.TemporaryDirectory(prefix="immobile-decision-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            if override is not None:
                key = f"borg_param_{override}"
                config, count = re.subn(fr"^{key} = 0$", f"{key} = 1", config, flags=re.M)
                self.assertEqual(count, 1)
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_IMMOBILE_DECISION=case)
            result = subprocess.run(
                [
                    str(GAME), "-mborgh", "-n",
                    *(f"-d{name}={path}" for name, path in paths.items()),
                    "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters",
                    "-start-depth", "5", "-keys", "250",
                ],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f"immobile decision passed: {case}", result.stdout)

    def test_cornered_phase(self):
        self.run_case("phase-shipped")
        self.run_case("phase-override", "escape_paralyser_fight_lost")

    def test_cornered_without_phase(self):
        self.run_case("phase-shipped-no-scroll")
        self.run_case("phase-override-no-scroll", "escape_paralyser_fight_lost")

    def test_never_move_memory(self):
        self.run_case("memory-shipped")
        self.run_case("memory-override", "caution_remember_never_move")

    def test_stair_in_reach(self):
        self.run_case("stair-in-reach")

    def test_strategic_retreat(self):
        self.run_case("strategic-retreat")


if __name__ == "__main__":
    unittest.main()

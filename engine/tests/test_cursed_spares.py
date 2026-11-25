"""Run the native cursed spare disposal regression from #1231."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"


@unittest.skipUnless(GAME.is_file(), "test-cursed-spares build required")
class CursedSparesTest(unittest.TestCase):
    def test_dispose_cursed_spares(self):
        with tempfile.TemporaryDirectory(prefix="cursed-spares-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            for name in ("borg.txt", "scary_monsters.txt"):
                shutil.copyfile(ROOT / "harness" / name, paths["user"] / name)
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_CURSED_SPARES="1")
            run = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{key}={value}" for key, value in paths.items()),
                 "--", "-seed", "101", "-race", "1", "-class", "1", "-menu-letters", "-start-depth", "5", "-keys", "250"],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        self.assertIn("#1231 cursed spare actions passed", run.stdout)


if __name__ == "__main__":
    unittest.main()

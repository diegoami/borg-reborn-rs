"""Exercise the P-Borg map filename prompt in native Angband (#1303)."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"


@unittest.skipUnless(GAME.is_file(), "native build required")
class MapExportTest(unittest.TestCase):
    def test_export_then_continue_playing(self):
        for case in ("cancel", "open_failure", "success"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as tmp:
                out = Path(tmp)
                dirs = {name: out / name for name in ("user", "save", "archive", "panic", "scores")}
                for path in dirs.values():
                    path.mkdir()
                write_pborg_config(dirs["user"], "borg_reborn_core: borg-rs\nborg_save_death: 1\n")
                messages = out / "messages.log"
                env = {key: value for key, value in os.environ.items()
                       if not key.startswith(("BORG_", "BORGH_", "ANGBAND_"))}
                env.update(BORGH_TEST_MAP_EXPORT=case, BORGH_DEBUG="1")
                run = subprocess.run(
                    [str(GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in dirs.items()),
                     "-uMapExport", "--", "-seed", "29", "-race", "0", "-class", "0",
                     "-turns", "300", "-msglog", str(messages)],
                    cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=20,
                )
                self.assertEqual(run.returncode, 0, run.stderr[-6000:])
                self.assertIn("Borg map File:", run.stderr)
                summary = json.loads(run.stdout)
                self.assertEqual(summary["stop_reason"], "turns")
                self.assertGreater(summary["keys"], 0)
                self.assertFalse(summary["dead"])
                self.assertFalse(list(dirs["panic"].iterdir()))
                self.assertEqual("Cannot open that file." in messages.read_text(),
                                 case == "open_failure")
                maps = list(dirs["archive"].glob("*.map"))
                if case == "success":
                    self.assertEqual(len(maps), 1)
                    content = maps[0].read_text()
                    dungeon = content.split("Borg Compile Date: ", 1)[1].split("[Last Messages]", 1)[0]
                    self.assertTrue(any(len(row) == 198 and "@" in row for row in dungeon.splitlines()))
                    inventory = content.split("[Character Inventory]", 1)[1].split("[Home Inventory", 1)[0]
                    self.assertRegex(inventory, r"(?m)^[a-z]\) .+")
                else:
                    self.assertEqual(maps, [])


if __name__ == "__main__":
    unittest.main()

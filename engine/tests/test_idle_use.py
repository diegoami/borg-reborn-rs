"""Exercise the Borg's idle consumable policy in the native engine."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"
WARRIOR = ROOT / "engine/tests/fixtures/warrior-level5.build"

# Angband's kind numbers for the pinned engine in lib/gamedata/object.txt.
TREASURE_DETECTION = (25, 5)
HOLY_CHANT = (25, 29)
HEROISM = (26, 25)
SLIME_MOLD_JUICE = (26, 33)


@unittest.skipUnless(GAME.is_file(), "native build required")
class IdleUseTest(unittest.TestCase):
    def run_case(self, supplies, enabled=1, starter_monster=False):
        with tempfile.TemporaryDirectory(prefix="idle-use-") as directory:
            fixture = Path(directory) / "idle-use.build"
            base = [
                line for line in WARRIOR.read_text().splitlines()
                if line != "end" and not line.startswith(("item:25:", "item:26:"))
            ]
            items = [
                f"item:{tval}:{sval}:{quantity}:-1:0:0:0:0:0:1"
                for (tval, sval), quantity in supplies
            ]
            fixture.write_text("\n".join([*base, *items, "end", ""]))
            out = Path(directory) / "run"
            env = {key: value for key, value in os.environ.items()
                   if not key.startswith(("BORG_", "BORGH_", "ANGBAND_BIN"))}
            env.update(
                BORG_CORE="borg-rs",
                BORG_CORE_TRACE="1",
                BORG_TXT_SET=f"borg_param_item_use_idle={enabled}",
            )
            flags = "menu,start-depth=1"
            if starter_monster:
                flags = "menu,start-depth=3,start-monsters=white_worm_mass:1:1"
            result = subprocess.run(
                [str(ROOT / "harness/run.sh"), str(out), "101", "0", "0", "450",
                 flags, "--", "-start-build", str(fixture)],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            trace = [json.loads(line) for line in (out / "trace-core.jsonl").read_text().splitlines()]
            return trace, (out / "messages.log").read_text()

    def test_reads_or_quaffs_the_singleton_at_depth_one(self):
        for chant, juice, key, item in (
            (2, 1, "q", "Potions of Slime Mold Juice"),
            (1, 2, "r", "Scrolls of Holy Chant"),
        ):
            with self.subTest(chant=chant, juice=juice):
                trace, messages = self.run_case(
                    [(HEROISM, 5), (HOLY_CHANT, chant), (SLIME_MOLD_JUICE, juice)]
                )
                idle = [row for row in trace if row["check"] == "borg_use_idle"]
                self.assertTrue(idle, messages)
                self.assertEqual(idle[0]["depth"], 1)
                self.assertIn(ord(key), idle[0]["keys"])
                self.assertTrue(any(
                    f"You have no more {item}" in line and int(line.split()[1]) == idle[0]["turn"]
                    for line in messages.splitlines()
                ), messages)

    def test_treasure_detection_uses_the_last_scroll_at_depth_one(self):
        trace, messages = self.run_case([(HEROISM, 5), (TREASURE_DETECTION, 1)])
        idle = next(row for row in trace if row["check"] == "borg_use_idle")
        self.assertEqual(idle["depth"], 1)
        self.assertIn(ord("r"), idle["keys"])
        self.assertTrue(any(
            "You have no more Scrolls of Treasure Detection" in line
            and int(line.split()[1]) == idle["turn"]
            for line in messages.splitlines()
        ), messages)

    def test_visible_monster_and_combat_precede_idle_use(self):
        trace, messages = self.run_case([(SLIME_MOLD_JUICE, 1)], starter_monster=True)
        checks = [row["check"] for row in trace]
        self.assertIn("You have destroyed the white worm mass", messages)
        self.assertIn("borg_attack", checks)
        self.assertIn("borg_use_idle", checks)
        self.assertLess(checks.index("borg_attack"), checks.index("borg_use_idle"))

    def test_switch_off_skips_idle_use(self):
        trace, messages = self.run_case([(SLIME_MOLD_JUICE, 1)], enabled=0)
        self.assertFalse(any(row["check"] == "borg_use_idle" for row in trace), messages)


if __name__ == "__main__":
    unittest.main()

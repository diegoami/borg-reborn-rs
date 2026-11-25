"""Run #1253 through the Borg and native Angband inventory commands."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"
BASE_KIT = ROOT / "engine/tests/fixtures/warrior-level5.build"

# tval and sval from list-tvals.h and object.txt at the pin in engine/UPSTREAM.
BLINDNESS = (26, 35)
SLEEP = (26, 34)
CONFUSION = (26, 36)
SLIME_MOLD_JUICE = (26, 33)
CURE_LIGHT_WOUNDS = (26, 8)
TELEPORTATION = (25, 2)
SUMMON_MONSTER = (25, 15)
SUMMONING = (22, 1)
HASTE_MONSTER = (23, 30)
DARKNESS_WAND = (23, 19)


@unittest.skipUnless(GAME.is_file(), "test-junk-consumables build required")
class JunkConsumablesTest(unittest.TestCase):
    def launch(self, items, *, distance=None, monster="giant_white_mouse", keys=45):
        with tempfile.TemporaryDirectory(prefix="junk-consumables-") as directory:
            out = Path(directory)
            paths = {name: out / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            for name in ("borg.txt", "scary_monsters.txt"):
                shutil.copyfile(ROOT / "harness" / name, paths["user"] / name)
            # Keep equipped gear, food, and fuel from the known 72 HP build.
            # Replace its other pack contents with the items under test.
            records = [
                line for line in BASE_KIT.read_text().splitlines()
                if line != "end" and (
                    not line.startswith("item:") or line.split(":")[4] != "-1"
                    or line.startswith(("item:19:", "item:28:"))
                )
            ]
            for (tval, sval), qty, known, charges in items:
                records.append(f"item:{tval}:{sval}:{qty}:-1:0:0:0:{charges}:0:{int(known)}")
            build = out / "start.build"
            build.write_text("\n".join([*records, "end"]) + "\n")

            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(
                BORG_CORE="borg-rs",
                BORG_RUN_DIR=str(out),
                BORG_PARAM_ATTACK_THROW_JUNK_POTIONS="1",
                BORG_PARAM_JUNK_DISCARD_HARMFUL_IN_TOWN="1",
                BORGH_TEST_JUNK_CONSUMABLES="town" if distance is None else "throw",
            )
            messages = out / "messages.log"
            args = [
                str(GAME), "-mborgh", "-n", *(f"-d{key}={value}" for key, value in paths.items()),
                "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters",
                "-keys", str(keys), "-msglog", str(messages), "-start-build", str(build),
            ]
            if distance is not None:
                args.extend(["-start-depth", "1", "-start-monsters", f"{monster}:1:{distance}"])
            run = subprocess.run(args, cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            frames = [json.loads(line) for line in (out / "trace-core.jsonl").read_text().splitlines()]
            return messages.read_text(), frames

    def test_max_range_throw_uses_the_singleton(self):
        kit = [(BLINDNESS, 1, True, 0), (SLEEP, 2, True, 0),
               (CONFUSION, 1, False, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, frames = self.launch(kit, distance=10)
        self.assertIn(ord("v"), frames[0]["keys"])
        self.assertIn("Throwing painful object 'Potion of Blindness'", messages)
        self.assertIn("You have no more Potions of Blindness", messages)
        self.assertNotIn("Throwing painful object 'Potions of Sleep'", messages)
        self.assertEqual(frames[1]["healing_potions"], 2)

        messages, frames = self.launch(kit, distance=11)
        self.assertNotIn(ord("v"), frames[0]["keys"])
        self.assertTrue(any(ord("v") in frame["keys"] for frame in frames[1:]))
        self.assertIn("You have no more Potions of Blindness", messages)

    def test_unknown_and_useful_potions_are_kept(self):
        kit = [(CONFUSION, 1, False, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, frames = self.launch(kit, distance=3)
        self.assertNotIn("Junk potion throw", messages)
        self.assertEqual(frames[0]["healing_potions"], 2)

    def test_zero_dice_throw_is_an_option(self):
        messages, _ = self.launch([(SLIME_MOLD_JUICE, 1, True, 0)], distance=4)
        self.assertIn("You have no more Potions of Slime Mold Juice", messages)

    def test_stronger_melee_attack_wins(self):
        messages, frames = self.launch([(BLINDNESS, 1, True, 0)], distance=1)
        self.assertNotIn(ord("v"), frames[0]["keys"])
        self.assertIn("Attacking with weapon 'Dagger", messages)
        self.assertNotIn("Junk potion throw", messages)

    def test_dangerous_contact_uses_escape(self):
        kit = [(BLINDNESS, 1, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0),
               (TELEPORTATION, 3, True, 0)]
        messages, frames = self.launch(kit, distance=1, monster="hill_giant")
        self.assertGreaterEqual(frames[0]["danger"], frames[0]["hp"])
        self.assertIn("You have 2 Scrolls of Teleportation", messages)
        self.assertNotIn("Junk potion throw", messages)

    def test_town_drops_harmful_stacks_before_shopping(self):
        kit = [(BLINDNESS, 1, True, 0), (SLEEP, 2, True, 0),
               (CONFUSION, 1, False, 0), (CURE_LIGHT_WOUNDS, 2, True, 0),
               (SUMMON_MONSTER, 3, True, 0), (SUMMONING, 1, True, 4),
               (HASTE_MONSTER, 1, True, 4), (DARKNESS_WAND, 1, True, 4)]
        messages, frames = self.launch(kit, keys=180)
        self.assertEqual([frame["check"] for frame in frames[:5]], ["borg_discard_town_consumable"] * 5)
        self.assertEqual(frames[5]["healing_potions"], 2)
        self.assertEqual(messages.count("# Dropping harmful town consumable"), 5)
        for amount, item in (("2", "Potions of Sleep"), ("a", "Potion of Blindness"),
                             ("3", "Scrolls of Summon Monster"), ("a", "Staff of Summoning"),
                             ("a", "Wand of Haste Monster")):
            with self.subTest(item=item):
                self.assertIn(f"You drop {amount} {item}", messages)
                self.assertNotIn(f"You have {amount} {item}", messages)
        self.assertNotIn("You drop a Wand of Darkness", messages)


if __name__ == "__main__":
    unittest.main()

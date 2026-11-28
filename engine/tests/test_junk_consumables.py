"""Run #1253 through the Borg and native Angband inventory commands."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"
BASE_KIT = ROOT / "engine/tests/fixtures/warrior-level5.build"
RANGER_KIT = ROOT / "engine/tests/fixtures/ranger-level5.build"
NECROMANCER_KIT = ROOT / "engine/tests/fixtures/necromancer-level5.build"
RANGER, NECROMANCER = 7, 4

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
CONTEMPLATION = (26, 19)
NIMBLENESS = (26, 20)
SECOND_SIGHT = (29, 1)
EXPERIENCE = (26, 7)
STAR_ENLIGHTENMENT = (26, 23)


@unittest.skipUnless(GAME.is_file(), "test-junk-consumables build required")
class JunkConsumablesTest(unittest.TestCase):
    def launch(self, items, *, distance=None, monster="giant_white_mouse", keys=45, params=None,
               player_class=0, base_kit=BASE_KIT):
        with tempfile.TemporaryDirectory(prefix="junk-consumables-") as directory:
            out = Path(directory)
            paths = {name: out / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            write_pborg_config(paths["user"])
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            # Keep equipped gear, food, and fuel from the known 72 HP build.
            # Replace its other pack contents with the items under test.
            records = [
                line for line in base_kit.read_text().splitlines()
                if line != "end" and (
                    not line.startswith("item:") or line.split(":")[4] != "-1"
                    or line.startswith(("item:19:", "item:28:", "item:32:", "item:33:"))
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
                BORG_PARAM_TOWN_SHOP_OVERLOADED="2",
                BORGH_TEST_JUNK_CONSUMABLES="town" if distance is None else "throw",
            )
            env.update({f"BORG_PARAM_{name.upper()}": str(value) for name, value in (params or {}).items()})
            messages = out / "messages.log"
            args = [
                str(GAME), "-mborgh", "-n", *(f"-d{key}={value}" for key, value in paths.items()),
                "--", "-seed", "101", "-race", "0", "-class", str(player_class), "-menu-letters",
                "-keys", str(keys), "-msglog", str(messages), "-start-build", str(build),
            ]
            if distance is not None:
                args.extend(["-start-depth", "1", "-start-monsters", f"{monster}:1:{distance}"])
            run = subprocess.run(args, cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            frames = [json.loads(line) for line in (out / "trace-core.jsonl").read_text().splitlines()]
            return messages.read_text(), frames

    def test_max_range_throw_uses_the_singleton(self):
        # Sleep sorts before Blindness in the native pack. Equal damage
        # must therefore displace the first candidate with the singleton.
        kit = [(SLEEP, 2, True, 0), (BLINDNESS, 1, True, 0),
               (CONFUSION, 1, False, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, frames = self.launch(kit, distance=10)
        self.assertIn(ord("v"), frames[0]["keys"])
        self.assertIn("Throwing painful object 'Potion of Blindness'", messages)
        self.assertIn("You have no more Potions of Blindness", messages)
        self.assertNotIn("Throwing painful object 'Potions of Sleep'", messages)
        self.assertEqual(frames[1]["healing_potions"], 2)

        messages, frames = self.launch(kit, distance=11)
        self.assertNotIn(ord("v"), frames[0]["keys"])
        self.assertNotIn("Discarding known harmful stack", messages)
        self.assertTrue(any(ord("v") in frame["keys"] for frame in frames[1:]))
        self.assertIn("You have no more Potions of Blindness", messages)

    def test_unknown_and_useful_potions_are_kept(self):
        kit = [(CONFUSION, 1, False, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, frames = self.launch(kit, distance=3)
        self.assertNotIn("Junk potion throw", messages)
        self.assertNotIn("# Dropping junk", messages)
        self.assertEqual(frames[0]["healing_potions"], 2)

    def test_zero_dice_throw_is_an_option(self):
        messages, frames = self.launch([(SLIME_MOLD_JUICE, 1, True, 0)], distance=4)
        self.assertIn(ord("v"), frames[0]["keys"])
        self.assertIn("Throwing painful object 'Potion of Slime Mold Juice'", messages)
        self.assertIn("You have no more Potions of Slime Mold Juice", messages)

    def test_stronger_melee_attack_wins(self):
        messages, frames = self.launch([(BLINDNESS, 1, True, 0)], distance=1)
        self.assertNotIn(ord("v"), frames[0]["keys"])
        self.assertIn("Attacking with weapon 'Dagger", messages)
        self.assertNotIn("Junk potion throw", messages)
        self.assertNotIn("You have no more Potions of Blindness", messages)

    def test_dangerous_contact_uses_escape(self):
        kit = [(BLINDNESS, 1, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0),
               (TELEPORTATION, 3, True, 0)]
        messages, frames = self.launch(kit, distance=1, monster="hill_giant")
        self.assertGreaterEqual(frames[0]["danger"], frames[0]["hp"])
        self.assertIn("You have 2 Scrolls of Teleportation", messages)
        self.assertNotIn("Junk potion throw", messages)
        self.assertNotIn("You have no more Potions of Blindness", messages)

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

    def test_town_drinks_the_stat_trade_it_accepts_and_sells_the_one_it_refuses(self):
        # #1371. The fixture is a Human Warrior (Strength 17, Intelligence
        # and Wisdom 10, Dexterity and Constitution 16). Its class drinks
        # Nimbleness: one Dexterity table step against one step of a random
        # other stat. It never drinks Contemplation (borg_junk's table).
        # Second Sight is a kind the core never uses (#443, #1049).
        kit = [(NIMBLENESS, 1, True, 0), (CONTEMPLATION, 1, True, 0),
               (SECOND_SIGHT, 2, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, frames = self.launch(kit, keys=120, params={"town_stat_trade_policy": 1})
        self.assertEqual(frames[1]["check"], "borg_use_things")
        self.assertIn("# Quaffing Potion of Nimbleness.", messages)
        self.assertIn("You feel very dextrous!", messages)
        self.assertIn("# Selling Potion of Contemplation", messages)
        self.assertIn("You had a Potion of Contemplation", messages)
        self.assertNotIn("Quaffing Potion of Contemplation", messages)
        self.assertIn("You had 2 Mushrooms of Second Sight", messages)
        self.assertNotIn("Quaffing Potion of Cure Light Wounds", messages)
        self.assertTrue(all(frame["healing_potions"] >= 2 for frame in frames), "the healing reserve stays")

        # The earlier play, the control: both potions stay in the pack.
        messages, _ = self.launch(kit, keys=120, params={"town_stat_trade_policy": 0})
        self.assertNotIn("Quaffing Potion of Nimbleness", messages)
        self.assertNotIn("Selling Potion of Contemplation", messages)
        self.assertIn("You had 2 Mushrooms of Second Sight", messages)

    def test_dungeon_drops_a_refused_stat_potion_the_engine_picked_up(self):
        # Review of #1372 (P3). pickup_always picks up whatever the Borg steps
        # on, so the floor filter alone leaves a refused stack in the pack. A
        # Warrior never drinks Contemplation; a quiet dungeon turn drops that
        # stack ("This item only": it is then not picked up again) and keeps
        # Nimbleness, which it drinks, and its healing reserve.
        kit = [(CONTEMPLATION, 1, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        params = {"town_stat_trade_policy": 1, "town_stat_trade_floor": 1}
        messages, frames = self.launch(kit, distance=11, keys=30, params=params)
        self.assertIn("# Dropping refused stat potion", messages)
        self.assertIn("You drop a Potion of Contemplation", messages)
        self.assertNotIn("Quaffing Potion of Contemplation", messages)
        self.assertNotIn("Quaffing Potion of Cure Light Wounds", messages)

        # Floor switch off, the control: the stack stays in the pack.
        messages, _ = self.launch(kit, distance=11, keys=30,
                                  params={"town_stat_trade_policy": 1, "town_stat_trade_floor": 0})
        self.assertNotIn("Dropping refused stat potion", messages)

    def test_town_drinks_experience_and_star_enlightenment_on_purpose(self):
        # #1373, #1374. Both have no harmful effect and their benefit is the
        # drink. Experience was drunk only by junk cleanup; *Enlightenment*
        # was on WORTHLESS, so a carried copy was sold or dropped.
        kit = [(EXPERIENCE, 1, True, 0), (STAR_ENLIGHTENMENT, 1, True, 0),
               (CURE_LIGHT_WOUNDS, 2, True, 0)]
        params = {"item_use_drink_experience": 1, "item_use_star_enlightenment": 1}
        messages, frames = self.launch(kit, keys=120, params=params)
        self.assertEqual(frames[1]["check"], "borg_use_things")
        self.assertIn("# Quaffing Potion of Experience.", messages)
        self.assertIn("You feel more experienced.", messages)
        self.assertIn("# Quaffing Potion of *Enlightenment*.", messages)
        self.assertIn("You feel very smart!", messages)
        self.assertIn("You feel very wise!", messages)
        self.assertNotIn("Selling Potion of *Enlightenment*", messages)
        self.assertNotIn("Quaffing Potion of Cure Light Wounds", messages)

        # The earlier play, the control: the kinds are not drunk on purpose.
        no = {"item_use_drink_experience": 0, "item_use_star_enlightenment": 0}
        messages, _ = self.launch(kit, keys=120, params=no)
        self.assertNotIn("Quaffing Potion of Experience", messages)
        self.assertNotIn("Quaffing Potion of *Enlightenment*", messages)
        self.assertNotIn("You feel more experienced.", messages)

    def test_experience_waits_for_a_quiet_turn(self):
        # #1373 (review): the drink takes a turn, and borg_use_things also
        # runs before the caution checks. With a hill giant next to the Borg
        # (danger above a tenth of its hit points) the potion stays in the pack.
        kit = [(EXPERIENCE, 1, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        params = {"item_use_drink_experience": 1}
        messages, frames = self.launch(kit, distance=1, monster="hill_giant", keys=20, params=params)
        self.assertGreaterEqual(frames[0]["danger"], frames[0]["hp"] // 10)
        self.assertNotIn("Quaffing Potion of Experience", messages)
        self.assertNotEqual(frames[0]["check"], "borg_use_things")

    def test_a_ranger_drinks_nimbleness_and_sells_contemplation(self):
        # #1371 (owner: "a Ranger would choose to drink Nimbleness"), natively
        # with the Ranger fixture (#1378): Strength 18/40, Intelligence 14,
        # Wisdom 18/20, Dexterity 18/50, Constitution 18/30. Its class table
        # refuses Contemplation.
        kit = [(NIMBLENESS, 1, True, 0), (CONTEMPLATION, 1, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, _ = self.launch(kit, keys=120, params={"town_stat_trade_policy": 1},
                                  player_class=RANGER, base_kit=RANGER_KIT)
        self.assertIn("# Quaffing Potion of Nimbleness.", messages)
        self.assertIn("You feel very dextrous!", messages)
        self.assertIn("# Selling Potion of Contemplation", messages)
        self.assertNotIn("Quaffing Potion of Contemplation", messages)
        messages, _ = self.launch(kit, keys=120, params={"town_stat_trade_policy": 0},
                                  player_class=RANGER, base_kit=RANGER_KIT)
        self.assertNotIn("Quaffing Potion of Nimbleness", messages)
        self.assertNotIn("Selling Potion of Contemplation", messages)

    def test_a_necromancer_sells_the_torches_it_cannot_use(self):
        # #1375, natively with the Necromancer fixture (#1378). The Scrolls of
        # Light are dropped as a kind either way (#1273); the Wooden Torches
        # (an Unlight class sees without one) are sold under the one rule and
        # kept without it.
        kit = [((25, 23), 2, True, 0), (CURE_LIGHT_WOUNDS, 2, True, 0)]
        messages, _ = self.launch(kit, keys=150, params={"town_refused_kind_policy": 1},
                                  player_class=NECROMANCER, base_kit=NECROMANCER_KIT)
        self.assertIn("# Discarding Scrolls of Light (Necromancer)", messages)
        self.assertIn("# Selling Wooden Torches (5000 turns)", messages)
        messages, _ = self.launch(kit, keys=150, params={"town_refused_kind_policy": 0},
                                  player_class=NECROMANCER, base_kit=NECROMANCER_KIT)
        self.assertIn("# Discarding Scrolls of Light (Necromancer)", messages)
        self.assertNotIn("Selling Wooden Torch", messages)


if __name__ == "__main__":
    unittest.main()

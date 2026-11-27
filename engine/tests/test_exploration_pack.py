"""Inferior native gear gives way to exploration supplies in town and dungeon."""

import os
import re
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"
FEATURE_GAME = ROOT / "build/rust-test/game/angband"
FIXTURES = ROOT / "engine/tests/fixtures"


class ExplorationPackTest(unittest.TestCase):
    def run_case(self, harmful, *, fixture=None, enabled=True, full_pack=True):
        game = FEATURE_GAME if fixture else GAME
        if not game.is_file():
            self.skipTest(f"native build required for {game.parent.parent.name}")
        with tempfile.TemporaryDirectory(prefix="exploration-pack-") as directory:
            root = Path(directory)
            paths = {name: root / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            write_pborg_config(paths["user"])
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            build = root / "full.build"
            kit_name = "exploration-full-pack.build" if full_pack else "exploration-town-spares.build"
            kit = (FIXTURES / kit_name).read_text()
            if not harmful:
                kit = kit.replace("item:26:34:4:", "item:26:30:4:")
            if fixture == "gear":
                kit = kit.replace("item:16:2:1:6:0:0:0:", "item:16:5:1:6:0:0:20:")
                kit = kit.replace("item:25:33:1:-1:", "item:17:1:1:-1:")
            build.write_text(kit)
            messages = root / "messages.log"
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_PARAM_JUNK_EXPLORATION_FIRST="1" if enabled else "0",
                       BORG_PARAM_JUNK_GEAR_ALTERNATIVES="0",
                       BORG_PARAM_JUNK_INFERIOR_SPARES="0", BORG_PARAM_TOWN_SHOP_OVERLOADED="1",
                       BORG_PARAM_ATTACK_THROW_JUNK_POTIONS="1",
                       BORG_PARAM_ITEM_USE_IDLE="0")
            if fixture:
                env["BORG_TEST_EXPLORATION_PACK"] = fixture
            if fixture == "weight-blocked-upgrade":
                env["BORG_PARAM_TOWN_SHOP_OVERLOADED"] = "2"
            if fixture == "ranked-cleanup":
                # #1307: the shipped ranked policy with Home backups.
                env["BORG_PARAM_JUNK_GEAR_ALTERNATIVES"] = "2"
                env["BORG_PARAM_TOWN_GEAR_HOME_BACKUPS"] = "1"
                env["BORG_PARAM_TOWN_SHOP_OVERLOADED"] = "2"
            result = subprocess.run(
                [str(game), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-start-depth", "3",
                 "-start-build", str(build), "-turns", "120", "-msglog", str(messages)],
                env=env, cwd=game.parent, capture_output=True, text=True, timeout=30,
            )
            log = messages.read_text()
            self.assertEqual(result.returncode, 0, result.stderr + log[-4000:])
            if fixture:
                self.assertIn(f"#1264 exploration pack passed: {fixture}", result.stdout, log[-4000:])
            if enabled:
                self.assertNotRegex(log, r"You drop [^\n]*Word of Recall")
            return log[-6000:]

    def test_reserve_survives_even_when_recall_is_the_only_candidate(self):
        self.run_case(harmful=False, fixture="recall-control")
        control = self.run_case(harmful=False, fixture="recall-control", enabled=False)
        self.assertRegex(control, r"You drop [^\n]*Word of Recall")

    def test_comparison_uses_known_properties_after_identification_and_preserves_real_swaps(self):
        self.run_case(harmful=False, fixture="gear")

    def test_ranked_spares_count_worn_pack_and_home(self):
        self.run_case(harmful=False, fixture="ranked-gear")

    def test_ranked_cleanup_routines_do_not_block_each_other(self):
        self.run_case(harmful=False, fixture="ranked-cleanup")

    def test_weight_blocked_home_upgrade_leaves_the_carried_upgrade_available(self):
        self.run_case(harmful=False, fixture="weight-blocked-upgrade")

    def test_drops_known_harmful_potions_before_usable_supplies(self):
        log = self.run_case(harmful=True)
        self.assertIn("You drop 4 Potions of Sleep {ignore}", log)

    def test_drops_outclassed_weapon_before_usable_supplies(self):
        log = self.run_case(harmful=False)
        self.assertIn("# Dropping inferior spare to make pack room Trident", log)
        self.assertRegex(log, r"You drop [^\n]*Trident[^\n]*\{borg ignore")

    def test_cheap_dungeon_drop_uses_the_shared_rule_without_pack_pressure(self):
        log = self.run_case(harmful=False, full_pack=False)
        self.assertIn("# Dropping inferior spare Chain Mail", log)
        self.assertRegex(log, r"You drop [^\n]*Chain Mail[^\n]*\{borg ignore")


@unittest.skipUnless(FEATURE_GAME.is_file(), "test-junk-consumables build required")
class TownSpareGearTest(unittest.TestCase):
    def run_town(self, planner, *, arrival, enabled=True):
        with tempfile.TemporaryDirectory(prefix="town-spare-gear-") as directory:
            root = Path(directory)
            paths = {name: root / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/pborg.yaml").read_text()
            self.assertIn("\nborg_town_plan: 1\n", config)
            write_pborg_config(paths["user"],
                config.replace("\nborg_town_plan: 1\n", f"\nborg_town_plan: {planner}\n")
            )
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(
                BORG_CORE="borg-rs", BORGH_TEST_JUNK_CONSUMABLES="town",
                BORG_PARAM_JUNK_EXPLORATION_FIRST="1" if enabled else "0",
                BORG_PARAM_JUNK_GEAR_ALTERNATIVES="0",
                BORG_PARAM_JUNK_INFERIOR_SPARES="0",
                BORG_PARAM_TOWN_SHOP_OVERLOADED="2" if arrival else "1",
                BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1",
            )
            messages = root / "messages.log"
            result = subprocess.run(
                [str(FEATURE_GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101" if arrival else "102", "-race", "0", "-class", "0", "-menu-letters",
                 "-start-build", str(FIXTURES / "exploration-town-spares.build"),
                 "-keys", "130" if arrival else "300", "-msglog", str(messages)],
                env=env, cwd=FEATURE_GAME.parent, capture_output=True, text=True, timeout=30,
            )
            log = messages.read_text()
            self.assertEqual(result.returncode, 0, result.stderr + log[-4000:])
            return log

    def test_town_arrival_sells_or_drops_inferior_gear_and_keeps_upgrades_and_recall(self):
        log = self.run_town(planner=1, arrival=True)
        self.assertRegex(log, r"You drop [^\n]*Chain Mail[^\n]*\{borg ignore surplus")
        self.assertRegex(log, r"You (?:sold|had) 2 Tridents")
        self.assertNotRegex(log, r"You drop [^\n]*Trident")
        self.assertNotRegex(log, r"You have (?!no more )[^\n]*Chain Mail[^\n]*borg ignore")
        self.assertNotRegex(log, r"You (?:drop|sold|had|have no more) [^\n]*Word of Recall")
        # The extra armour and the more accurate Rapier are still available
        # when normal gearing resumes after arrival unloading.
        self.assertLess(log.index("You drop Chain Mail"), log.index("You are wearing a Pair of Leather Boots"))
        self.assertLess(log.index("You had 2 Tridents"), log.index("You are wielding a Rapier"))

    def test_arrival_uses_the_legacy_comparison_with_flag_off(self):
        log = self.run_town(planner=1, arrival=True, enabled=False)
        self.assertNotRegex(log, r"You drop [^\n]*Trident")
        self.assertNotIn("borg ignore surplus", log)
        self.assertLess(log.index("You are wielding a Rapier"), log.index("You had 2 Tridents"))

    def test_both_town_choosers_sell_the_whole_inferior_stack(self):
        for planner in (0, 1):
            with self.subTest(planner=planner):
                log = self.run_town(planner, arrival=False)
                sales = list(re.finditer(r"You (?:sold|had) (a|\d+) Tridents?\b", log))
                self.assertEqual([sale[1] for sale in sales], ["2"], log)
                sold_at = sales[0].start()
                self.assertNotRegex(log[:sold_at], r"You (?:drop|sold|had|have no more) [^\n]*Word of Recall")
                self.assertNotRegex(log[:sold_at], r"You (?:drop|sold|had) [^\n]*(?:Dagger|Rapier|Leather Boots)")


if __name__ == "__main__":
    unittest.main()

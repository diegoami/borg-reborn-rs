"""Run the constructed #1239 Priest kit through native town commands."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest


from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"


@unittest.skipUnless(GAME.is_file(), "test-town-loadout build required")
class TownLoadoutTest(unittest.TestCase):
    def run_case(self, case):
        reserve = case.startswith("reserve-overweight")
        arrival = reserve or case in ("arrival", "arrival-shop")
        ammo_reserve = "57" if reserve or case == "arrival-shop" else "20" if case in ("trim", "arrival") else "0"
        with tempfile.TemporaryDirectory(prefix="town-loadout-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            write_pborg_config(paths["user"])
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            messages = Path(directory) / "messages.log"
            env = {
                key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))
            }
            env.update(
                BORG_CORE="borg-rs",
                BORG_TEST_TOWN_LOADOUT=case,
                BORG_PARAM_TOWN_SHOP_OVERLOADED="2" if arrival else "1",
                BORG_PARAM_JUNK_EXPLORATION_FIRST="1" if reserve or case == "reserve-swap" else "0",
                BORG_PARAM_TOWN_LOADOUT_POWER_LOSS="3000",
                BORG_PARAM_TOWN_QUIVER_TRIM_RESERVE__PRIEST=ammo_reserve,
                BORG_PARAM_TOWN_RECALL_URGENT_DEPTH="5",
                BORG_PARAM_TOWN_HOME_RESERVE="1" if reserve or case in ("reserve-overlap", "reserve-swap", "arrival-shop") else "0",
                BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1",
                BORG_PARAM_CAUTION_SCARY_NAMED_MONSTERS="0",
                BORG_PARAM_CAUTION_SCARY_DAMAGE_PERCENT="0",
            )
            env["BORG_PARAM_TOWN_LIGHT_RESERVE_TURNS__PRIEST"] = "0"
            env["BORG_PARAM_TOWN_STAPLE_OIL__PRIEST"] = "0"
            result = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "3", "-menu-letters",
                 "-msglog", str(messages), "-keys", "1800" if arrival else "750"],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            log = messages.read_text()
            self.assertEqual(result.returncode, 0, result.stderr + log[-4000:])
            marker = "#1264 exploration reserve" if reserve else "#1239 town loadout"
            self.assertIn(f"{marker} passed: {case}", result.stdout, log[-6000:])
        return log

    def test_trim_and_full_home_swap_store_the_mace(self):
        log = self.run_case("trim")
        self.assertLess(log.index("# Town headroom dropping"), log.index("# Town loadout: preparing a lighter kit"))
        self.assertLess(log.index("# Town loadout: preparing a lighter kit"), log.index("# Town loadout: displaced gear stored at Home"))

    def test_arrival_trims_and_deposits_gear_before_shopping(self):
        log = self.run_case("arrival")
        dropped = log.index("# Dropping surplus Rounded Pebbles")
        exchange = log.index("# Town loadout: preparing a lighter kit")
        deposit = log.index("# Town loadout: displaced gear stored at Home")
        recall = re.search(r"You have .*\bWord of Recall", log)
        self.assertIsNotNone(recall, log[-4000:])
        self.assertLess(dropped, exchange)
        self.assertLess(exchange, deposit)
        self.assertLess(deposit, recall.start())

    def test_full_home_without_storage_has_no_safe_proposal(self):
        self.run_case("no-storage")

    def test_departs_overweight_with_recall_from_home(self):
        log = self.run_case("reserve-overweight")
        restock = log.index("# Town restock: accepting weight for an exploration reserve")
        self.assertRegex(log[restock:], r"You have .*\bWord of Recall")

    def test_departs_overweight_with_recall_from_a_shop(self):
        log = self.run_case("reserve-overweight-shop")
        restock = log.index("# Town restock: accepting weight for an exploration reserve")
        self.assertRegex(log[restock:], r"You bought .*\bWord of Recall")

    def test_arrival_buys_lighter_gear_and_stores_displaced_piece(self):
        log = self.run_case("arrival-shop")
        exchange = log.index("# Town loadout: preparing a lighter kit")
        bought = log.index("You bought ", exchange)
        deposit = log.index("# Town loadout: displaced gear stored at Home", bought)
        self.assertRegex(log[bought:deposit], r"You are (?:wielding|wearing) ")
        recall = re.search(r"You have .*\bWord of Recall", log[deposit:])
        self.assertIsNotNone(recall, log[-4000:])

    def test_heavier_home_upgrade_can_be_proposed_after_a_strength_gain(self):
        self.run_case("restore")

    def test_excluded_effect_item_is_evicted_ahead_of_a_priority_potion(self):
        self.run_case("reserve-overlap")

    def test_resist_swap_at_home_is_never_replacement_stock(self):
        # #1317: a Ring of Resist Poison with no carried power stays protected.
        self.run_case("reserve-swap")


if __name__ == "__main__":
    unittest.main()

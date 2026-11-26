"""Native regressions for inferior spare equipment disposal."""

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
FIXTURE = ROOT / "engine/tests/fixtures/inferior-sandals.build"


@unittest.skipUnless(GAME.is_file(), "native build required")
class InferiorSparesTest(unittest.TestCase):
    def run_sandals(self, legacy_source):
        # The fixture leaves to-hit unknown and gives the pack pair a hidden
        # penalty. Angband renders [1,+0] {??} from the actual object.
        with tempfile.TemporaryDirectory(prefix="inferior-sandals-") as directory:
            root = Path(directory)
            paths = {key: root / key for key in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            if legacy_source == "file":
                config += "\nborg_param_junk_inferior_sandal_ac_margin = 7\n"
            write_pborg_config(paths["user"], config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(
                BORG_CORE="borg-rs",
                BORG_PARAM_JUNK_INFERIOR_SPARES="1",
                BORG_PARAM_JUNK_EXPLORATION_FIRST="0",
            )
            if legacy_source == "environment":
                env["BORG_PARAM_JUNK_INFERIOR_SANDAL_AC_MARGIN"] = "7"
            messages = root / "messages.log"
            run = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{key}={path}" for key, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-start-depth", "3",
                 "-start-build", str(FIXTURE), "-turns", "700", "-msglog", str(messages)],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            self.assertIn("ignoring retired sandal junking setting", run.stderr)
            return messages.read_text()

    def test_retired_margin_does_not_drop_unknown_sandals(self):
        for source in ("file", "environment"):
            with self.subTest(source=source):
                self.assertNotRegex(self.run_sandals(source), r"You drop [^\n]*Leather Sandals")


@unittest.skipUnless(
    (ROOT / "build/rust-test/game/angband").is_file(), "test-town-review build required"
)
class TownSpareSaleTest(unittest.TestCase):
    def test_both_choosers_sell_the_whole_stack(self):
        for planner in (0, 1):
            with self.subTest(planner=planner):
                messages = self.run_town(planner)
                quantities = re.findall(
                    r"You (?:had|sold) (\d+|a) Pairs? of Leather Sandals \[1,\+0\]", messages
                )
                sandals = "\n".join(line for line in messages.splitlines() if "Leather Sandals" in line)
                self.assertIn("You are wearing a Pair of Leather Sandals [1,+7]", sandals)
                self.assertEqual(quantities, ["3"], sandals)

    def test_redundant_home_stack_is_withdrawn_and_disposed(self):
        messages = self.run_town(1, ranked_home=True)
        sandals = "\n".join(line for line in messages.splitlines() if "Leather Sandals" in line or "# Home reserve:" in line)
        self.assertIn("# Home reserve: withdrawing low-priority stock", sandals)
        self.assertRegex(sandals, r"You drop 3 Pairs of Leather Sandals \[1,\+0\] \{borg ignore\}")
        # Angband also says "You drop" when putting gear safely in Home.
        self.assertNotRegex(sandals, r"You (?:had|sold)[^\n]*\[1,\+4\]|You drop[^\n]*\[1,\+4\] \{borg ignore")

    def test_full_home_supplies_upgrade_and_keeps_two_useful_backups(self):
        messages = self.run_town(1, gear_case="gear-full-home")
        sandals = "\n".join(line for line in messages.splitlines() if "Leather Sandals" in line)
        self.assertRegex(sandals, r"You are wearing .*\[1,\+11\]")
        for bonus in (8, 2):
            self.assertRegex(sandals, rf"You drop [^\n]*\[1,\+{bonus}\] \([a-z]\)")
        for bonus in (7, 4):
            self.assertRegex(sandals, rf"You drop [^\n]*\[1,\+{bonus}\] \{{borg ignore")

    def test_two_copies_of_best_ring_are_worn(self):
        messages = self.run_town(1, gear_case="gear-rings")
        rings = "\n".join(line for line in messages.splitlines() if "Ring" in line)
        self.assertEqual(len(re.findall(r"You are wearing .*Ring of Protection \[\+12\]", rings)), 2, rings)
        for bonus in (7, 4):
            self.assertRegex(rings, rf"You drop [^\n]*Ring of Protection \[\+{bonus}\] \([a-z]\)")
        self.assertRegex(rings, r"You drop [^\n]*Ring of Protection \[\+3\] \{borg ignore")

    def run_town(self, planner, *, ranked_home=False, gear_case=None):
        game = ROOT / "build/rust-test/game/angband"
        with tempfile.TemporaryDirectory(prefix="inferior-sandal-sale-") as directory:
            root = Path(directory)
            paths = {key: root / key for key in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            default = "\nborg_town_plan = 1\n"
            self.assertIn(default, config)
            config = config.replace(default, f"\nborg_town_plan = {planner}\n")
            write_pborg_config(paths["user"], config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            # Exercise the two selling choosers with the original town order.
            env.update(BORG_CORE="borg-rs", BORG_TEST_INFERIOR_SPARE_SALE="1",
                       BORG_PARAM_TOWN_SHOP_OVERLOADED="1", BORG_PARAM_JUNK_EXPLORATION_FIRST="0",
                       BORG_PARAM_TOWN_GEAR_HOME_BACKUPS="0")
            if ranked_home or gear_case:
                env.update(BORG_TEST_INFERIOR_SPARE_SALE=gear_case or "ranked-home",
                           BORG_PARAM_JUNK_EXPLORATION_FIRST="1", BORG_PARAM_JUNK_GEAR_ALTERNATIVES="2",
                           BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1", BORG_PARAM_TOWN_GEAR_HOME_BACKUPS=str(int(bool(gear_case))))
            messages = root / "messages.log"
            run = subprocess.run(
                [str(game), "-mborgh", "-n", *(f"-d{key}={path}" for key, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0",
                 "-keys", "1400" if gear_case else "800" if ranked_home else "500", "-msglog", str(messages)],
                cwd=game.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            return messages.read_text()


if __name__ == "__main__":
    unittest.main()

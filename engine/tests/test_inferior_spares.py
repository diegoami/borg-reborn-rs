"""Native regression for the #1236 unknown Leather Sandals comparison."""

import os
import re
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"
FIXTURE = ROOT / "engine/tests/fixtures/inferior-sandals.build"


@unittest.skipUnless(GAME.is_file(), "native build required")
class InferiorSparesTest(unittest.TestCase):
    def run_sandals(self, margin, quantity=1):
        # The fixture leaves to-hit unknown and gives the pack pair a hidden
        # penalty. Angband renders [1,+0] {??} from the actual object.
        with tempfile.TemporaryDirectory(prefix="inferior-sandals-") as directory:
            root = Path(directory)
            paths = {key: root / key for key in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            shutil.copyfile(ROOT / "harness/borg.txt", paths["user"] / "borg.txt")
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(
                BORG_CORE="borg-rs",
                BORG_PARAM_JUNK_INFERIOR_SPARES="1",
                BORG_PARAM_JUNK_EXPLORATION_FIRST="0",
                BORG_PARAM_JUNK_INFERIOR_SANDAL_AC_MARGIN=str(margin),
            )
            messages = root / "messages.log"
            build = root / "sandals.build"
            build.write_text(FIXTURE.read_text().replace("item:10:1:1:-1:", f"item:10:1:{quantity}:-1:"))
            run = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{key}={path}" for key, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-start-depth", "3",
                 "-start-build", str(build), "-turns", "700", "-msglog", str(messages)],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            return messages.read_text()

    def test_unknown_sandals_drop_with_ignore_inscription(self):
        messages = self.run_sandals(margin=7)
        self.assertRegex(messages, r"# Dropping inferior spare [^\n]*Leather Sandals \[1,\+0\] \{\?\?\}")
        self.assertRegex(messages, r"You drop [^\n]*Leather Sandals \[1,\+0\] \{borg ignore, \?\?\}")
        self.assertIn("You have no more Pairs of Leather Sandals [1,+0] {borg ignore, ??}", messages)
        self.assertRegex(messages, r"# Destroying floor item [^\n]*Leather Sandals")
        self.assertNotRegex(messages, r"You have (?:a Pair|\d+ Pairs) of Leather Sandals[^\n]*borg ignore")

    def test_high_value_stack_skips_priority_drop(self):
        messages = self.run_sandals(margin=7, quantity=4)
        self.assertNotRegex(messages, r"# Dropping inferior spare [^\n]*Leather Sandals")

    def test_disabled_unknown_rule_keeps_the_sandals(self):
        self.assertNotRegex(self.run_sandals(margin=0), r"You drop [^\n]*Leather Sandals")


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

    def run_town(self, planner):
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
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            # Exercise the two selling choosers with the original town order.
            env.update(BORG_CORE="borg-rs", BORG_TEST_INFERIOR_SPARE_SALE="1",
                       BORG_PARAM_TOWN_SHOP_OVERLOADED="1", BORG_PARAM_JUNK_EXPLORATION_FIRST="0")
            messages = root / "messages.log"
            run = subprocess.run(
                [str(game), "-mborgh", "-n", *(f"-d{key}={path}" for key, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0",
                 "-keys", "500", "-msglog", str(messages)],
                cwd=game.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            return messages.read_text()


if __name__ == "__main__":
    unittest.main()

"""Native regression for #1395: no store re-entry loop at a frozen game turn.

Entering a store with `5` and leaving it cost no game time
(cmd-cave.c:do_cmd_hold sets `energy_use = 0` after the store UI), so a
planner that keeps a store destination while the store's handler leaves
with nothing done can re-enter it without the game clock moving. The #1395
Warrior did so 1,320 times. The fixture starts the ranked-home town case of
`test_inferior_spares.py`, which shops at the General Store (store 1),
including two purchases there at the same game turn (a follow-up in one
visit). `BORG_TEST_REFUSE_STORE=0` (feature `test-town-review`) makes that
store refuse every transaction and keep the destination, as the Warrior's
store handler did. `BORG_TEST_DROP_STORE_COMMAND=0` drops a queued command
there instead, as if the game had refused it.
"""

from collections import Counter
import json
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
HOLD = "did nothing at this game turn; not entering it again until something changes (#1395)"
REFUSED = "test: the store refuses (BORG_TEST_REFUSE_STORE)"


@unittest.skipUnless(GAME.is_file(), "test-town-review build required")
class StoreLoopTest(unittest.TestCase):
    def test_refused_store_is_not_reentered_at_a_frozen_turn(self):
        messages, trace = self.run_town(refuse=0)
        exits = [row["store_exit"] for row in trace if row.get("store_exit")]
        refused = [row["turn"] for row in trace
                   if row.get("store_exit", {}).get("shop") == 0 and row["store_exit"]["reason"] == REFUSED]
        # The trace names why the store was left with nothing done, the store
        # action chosen outside before entering and the one chosen inside.
        self.assertTrue(refused, exits[:5])
        first = next(e for e in exits if e["shop"] == 0 and e["reason"] == REFUSED)
        self.assertEqual(first["requested"]["shop"], 0, first)
        self.assertEqual(len(first["requested"]["at"]), 4, first)
        self.assertEqual([(a["action"]["shop"], a["outcome"]) for a in first["attempts"]], [(0, REFUSED)], first)
        # Without the hold the Borg entered the store 177 times at turn 141.
        turn, visits = Counter(refused).most_common(1)[0]
        self.assertEqual(visits, 1, f"store 1 entered {visits} times at game turn {turn}")
        store_thinks = Counter(row["turn"] for row in trace if row["context"] == "store")
        self.assertLessEqual(max(store_thinks.values()), 2, store_thinks.most_common(3))
        self.assertIn("# Town plan: store 1 " + HOLD, messages)

    def test_rejected_queued_command_keeps_the_entry_request(self):
        # Re-review of #1414: the purchase is queued, then dropped
        # (BORG_TEST_DROP_STORE_COMMAND), so the game never carries it out.
        # The store_exit that ends the visit still names the outside request
        # and each attempt inside, the failed purchase first.
        _, trace = self.run_town(drop=0)
        exits = [row["store_exit"] for row in trace if row.get("store_exit")]
        failed = [e for e in exits if e["shop"] == 0
                  and any(a["outcome"] == "the purchase made no progress" for a in e["attempts"])]
        self.assertTrue(failed, exits[:5])
        exit = failed[0]
        self.assertIsNotNone(exit["requested"], exit)
        self.assertEqual((exit["requested"]["shop"], exit["requested"]["code"]), (0, 1), exit)
        self.assertGreaterEqual(len(exit["attempts"]), 2, exit)
        self.assertEqual(exit["attempts"][0]["action"]["shop"], 0, exit)

    def test_ordinary_shopping_keeps_its_same_turn_follow_up(self):
        messages, trace = self.run_town()
        self.assertNotIn(HOLD, messages)
        self.assertFalse([row for row in trace if row.get("store_exit", {}).get("reason") == REFUSED])
        purchases = Counter(re.findall(r"^\S+ (\d+) \S+ # Town plan executing action 1 at shop (\d+) ",
                                       messages, re.M))
        self.assertTrue([key for key, count in purchases.items() if count >= 2],
                        "no follow-up purchase at the same store and game turn: " + str(purchases))

    def run_town(self, refuse=None, drop=None):
        with tempfile.TemporaryDirectory(prefix="store-loop-") as directory:
            root = Path(directory)
            paths = {key: root / key for key in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            write_pborg_config(paths["user"], (ROOT / "harness/pborg.yaml").read_text())
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            # test_inferior_spares.py's ranked-home case.
            env.update(BORG_CORE="borg-rs", BORG_TEST_INFERIOR_SPARE_SALE="ranked-home",
                       BORG_PARAM_TOWN_SHOP_OVERLOADED="1", BORG_PARAM_JUNK_EXPLORATION_FIRST="1",
                       BORG_PARAM_JUNK_GEAR_ALTERNATIVES="2", BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1",
                       BORG_PARAM_TOWN_GEAR_HOME_BACKUPS="0", BORG_RUN_DIR=str(root))
            if refuse is not None:
                env["BORG_TEST_REFUSE_STORE"] = str(refuse)
            if drop is not None:
                env["BORG_TEST_DROP_STORE_COMMAND"] = str(drop)
            run = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{key}={path}" for key, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-keys", "900",
                 "-msglog", str(root / "messages.log")],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
            trace = [json.loads(line) for line in (root / "trace-core.jsonl").read_text().splitlines()]
            return (root / "messages.log").read_text(), trace


if __name__ == "__main__":
    unittest.main()

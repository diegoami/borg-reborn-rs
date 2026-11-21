"""Exercise the adjacent-stair handoff in the shipped borg-rs core.

The former smoke reference returns to the same down stair repeatedly after
turn 7,446. This test runs the engine and checks the outcome independently
of the reference files.
"""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"


def state_at(states, turn):
    return [state for state in states if int(state[0]) == turn]


@unittest.skipUnless(GAME.is_file(), "harness/build.sh required")
class StairHandoffTest(unittest.TestCase):
    def run_case(self, seed, player_class, stop_turn):
        with tempfile.TemporaryDirectory(prefix="stair-handoff-") as directory:
            out = Path(directory) / "game"
            env = {key: val for key, val in os.environ.items()
                   if not key.startswith(("BORG_", "BORGH_", "ANGBAND_BIN"))}
            if cemetery := os.environ.get("BORG_CEMETERY_DIR"):
                env["BORG_CEMETERY_DIR"] = cemetery
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off")
            result = subprocess.run(
                [str(ROOT / "harness/run.sh"), str(out), str(seed), "0", str(player_class), str(stop_turn), "menu"],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            messages = (out / "messages.log").read_text().splitlines()
            states = [row.split() for row in (out / "state.log").read_text().splitlines()]
            keys = [row.split() for row in (out / "keys.log").read_text().splitlines()]

        return messages, states, keys

    def test_desperate_fallback_makes_way_for_recall(self):
        messages, states, keys = self.run_case(101, 0, 8000)

        # An earlier ordinary descent remains available despite the handoff.
        self.assertIn(["541", "1", "3e", "0"], keys)
        self.assertEqual([state[2] for state in state_at(states, 541)][:2], ["d0/1", "d1/1"])

        # The actual trigger has to occur. Checking only the absence of the
        # bounce could silently pass if a later change moved the start.
        self.assertTrue(any(" 7446 0 # Standing on dn-stairs" in row for row in messages))
        self.assertTrue(any(" 7446 0 # no up stairs found, exploring for them" in row for row in messages))
        self.assertEqual(state_at(states, 7446)[0][1], "41,48")
        self.assertEqual(state_at(states, 7456)[0][1], "41,47")
        self.assertFalse(any(" 7456 0 # Flowing toward down-stairs at cost 1" in row for row in messages))
        self.assertTrue(any(" 7456 0 # Fleeing the level (recall)" in row for row in messages))
        self.assertEqual(state_at(states, 7466)[0][1], "41,47")

    def test_ordinary_fallback_does_not_restart_down_stair_loop(self):
        messages, states, _ = self.run_case(204, 1, 119000)
        self.assertTrue(any(" 118839 0 # Standing on dn-stairs" in row for row in messages))
        self.assertTrue(any(" 118839 0 # Flowing toward dark-" in row for row in messages))
        self.assertEqual(state_at(states, 118839)[0][1], "129,22")
        self.assertEqual(state_at(states, 118849)[0][1], "129,21")
        # The first patch reached (129,20), reversed twice, and started over.
        # Check the whole window so a three-square loop also fails the test.
        for turn in (118859, 118869, 118879):
            self.assertNotIn("129,22", [state[1] for state in state_at(states, turn)])
        self.assertTrue(state_at(states, 118879))

if __name__ == "__main__":
    unittest.main()

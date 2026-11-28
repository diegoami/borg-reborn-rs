"""Exercise the adjacent-stair handoff in the shipped borg-rs core.

The former smoke reference returns to the same down stair repeatedly after
turn 7,446. This test runs the engine and checks the outcome independently
of the reference files.
"""

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"
TEST_GAME = ROOT / "build/rust-test/game/angband"

# Five successful committed routes and one blocked matching route each request a step.
EXPLORATION_STEP_REQUESTS = 6


def state_at(states, turn):
    return [state for state in states if int(state[0]) == turn]


def has_stair_handoff_build():
    build_id = TEST_GAME.parent.parent / "BUILD_ID"
    if not TEST_GAME.is_file() or not build_id.is_file():
        return False
    features = next((line.removeprefix("test_features=") for line in build_id.read_text().splitlines()
                     if line.startswith("test_features=")), "")
    return "test-stair-handoff" in features.split(",")


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

    def test_known_up_route_continues_past_adjacent_down_stair(self):
        messages, states, _ = self.run_case(224, 0, 144000)
        self.assertTrue(any(" 143021 0 # Standing on dn-stairs" in row for row in messages))
        self.assertTrue(any(" 143021 0 # Flowing toward up-stairs at cost 4" in row for row in messages))
        self.assertEqual(state_at(states, 143021)[0][1], "162,35")
        self.assertEqual(state_at(states, 143031)[0][1], "163,34")

        later = state_at(states, 143041)
        self.assertTrue(later)
        self.assertNotIn("162,35", [state[1] for state in later])
        self.assertTrue(any(int(state[0]) > 143041 and state[2].startswith("d0/") for state in states))


@unittest.skipUnless(has_stair_handoff_build(), "test-stair-handoff build required")
class EmergencyStairHandoffTest(unittest.TestCase):
    def run_case(self, case, weight, *, shadow="off"):
        with tempfile.TemporaryDirectory(prefix="emergency-stair-handoff-") as directory:
            root = Path(directory)
            paths = {kind: root / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            overrides = {
                "caution_stairs_flight_threat_weight": weight,
                "caution_desperate_adjacent_down_distance": 3,
            }
            if case == "down-stair-plan":
                overrides.update(caution_rising_hold_flee_descent=1, caution_recall_wait_as_usual=0)
            if case == "combined-stair-plan":
                overrides.update(caution_rising_hold_flee_descent=1, caution_stairs_clean_path=2,
                                 caution_stairs_clean_path_flee=1, caution_stairs_block_danger_percent=0,
                                 caution_stairs_flight_ignore_danger=1)
            for name, value in overrides.items():
                key = f"borg_param_{name}"
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_STAIR_HANDOFF=case, BORG_RUN_DIR=str(root), BORG_SHADOW=shadow)
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "3", "-class", "6", "-menu-letters", "-start-depth", "5", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            shadow_cases = {
                "flow-step": ("flow direction selection passed", "readonly_points", "flow.select_step", 8),
                "exploration-flow": ("exploration flow passed", "root_points", "flow.continue_exploration", 8),
                "up-stair-plan": ("up-stair plan passed", "readonly_points", "flow.plan_up_stair", 10 if weight else 8),
                "down-stair-plan": ("down-stair plan passed", "root_points", "flow.plan_down_stair", 9 if weight else 7),
                "combined-stair-plan": ("combined stair plan passed", "root_points", "flow.plan_combined_stairs", 11 if weight else 14),
            }
            if case in shadow_cases:
                message, points, point, count = shadow_cases[case]
                self.assertIn(message, result.stdout)
                stats = root / "shadow-stats.jsonl"
                if shadow == "off":
                    self.assertFalse(stats.exists())
                else:
                    self.assertEqual((root / "shadow.jsonl").read_bytes(), b"")
                    row = json.loads(stats.read_text().splitlines()[-1])
                    self.assertEqual(row["mode"], shadow)
                    self.assertEqual(row["counts"][points][point], count)
                    nested_steps = {
                        "combined-stair-plan": 0 if weight else 1,
                        "exploration-flow": EXPLORATION_STEP_REQUESTS,
                    }.get(case, 0)
                    if nested_steps:
                        self.assertEqual(row["counts"]["readonly_points"]["flow.select_step"], nested_steps)
                    self.assertEqual(row["counts"]["flow_calls"], count + nested_steps)
            elif case == "stair-preview-wipe":
                self.assertIn("stair preview wipe passed", result.stdout)
            else:
                self.assertIn(f"emergency stair handoff passed: {case}", result.stdout)
            settings = {row["name"]: row for row in json.loads((root / "params-core.json").read_text())}
            resolved = json.loads((root / "settings-resolved.json").read_text())
            self.assertEqual((resolved["class_index"], resolved["class_name"]), (6, "rogue"))
            for name, value in overrides.items():
                self.assertEqual((settings[name]["value"], settings[name]["origin"]), (value, "borg.txt"))
                self.assertEqual(resolved["effective_settings"].get(name, settings[name]["default"]), value)

    def test_flight_moves_past_the_two_floor_squares_and_descends(self):
        for weight in (0, 5):
            for case in ("one-step", "two-steps"):
                with self.subTest(weight=weight, case=case):
                    self.run_case(case, weight)

    def test_unweighted_handoff_cannot_walk_toward_a_farther_stair(self):
        self.run_case("blocked", 0)

    def test_flow_direction_selection_preserves_native_results_and_rng(self):
        for shadow in ("off", "check", "switch"):
            with self.subTest(shadow=shadow):
                self.run_case("flow-step", 0, shadow=shadow)

    def test_committed_exploration_preserves_goal_results_and_fallthrough(self):
        for shadow in ("off", "check", "switch"):
            with self.subTest(shadow=shadow):
                self.run_case("exploration-flow", 0, shadow=shadow)

    def test_stair_preview_consumes_wipe_and_restores_cached_danger(self):
        self.run_case("stair-preview-wipe", 0)

    def test_up_stair_plan_preserves_refusals_weight_and_move_cap(self):
        for shadow in ("off", "check", "switch"):
            for weight in (0, 5):
                with self.subTest(shadow=shadow, weight=weight):
                    self.run_case("up-stair-plan", weight, shadow=shadow)

    def test_down_stair_plan_preserves_refusals_weight_and_move_cap(self):
        for shadow in ("off", "check", "switch"):
            for weight in (0, 5):
                with self.subTest(shadow=shadow, weight=weight):
                    self.run_case("down-stair-plan", weight, shadow=shadow)

    def test_combined_stair_plan_preserves_refusals_and_fallback_order(self):
        for shadow in ("off", "check", "switch"):
            for weight in (0, 5):
                with self.subTest(shadow=shadow, weight=weight):
                    self.run_case("combined-stair-plan", weight, shadow=shadow)


if __name__ == "__main__":
    unittest.main()

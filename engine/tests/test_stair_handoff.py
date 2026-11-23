"""Exercise the adjacent-stair handoff in borg-rs with its incident settings.

The former smoke reference returns to the same down stair repeatedly after
turn 7,446. This test runs the engine and checks the outcome independently
of the reference files.
"""

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


def state_at(states, turn):
    return [state for state in states if int(state[0]) == turn]


# The town settings the recorded incident predates (#859's five switches off,
# then #863: the staples' decay, the prepared remedies, the variety depth gate,
# and the Word of Recall and Phase Door purchases back to #864's values; #901's
# two attack settings). The
# incident classes are Warrior (0) and Mage (1), so only their class lines exist.
PINNED_TOWN_SETTINGS = [
    ("town_departure_free_weight", 0), ("town_departure_free_slots", 0),
    ("town_store_keep_min", 0), ("town_recall_curve_scale_percent", 0),
    ("town_recall_urgent", 0),
    ("town_prepared_percent", 0), ("town_variety_min_depth", 0),
    ("town_recall_reserve", 1), ("buy_recall_from_gold", 400),
    ("town_phase_reserve", 0), ("town_phase_reserve_percent", 100),
    ("town_device_percent", 0),
    ("caution_recall_blow_model", 0),
    # #901: the confused attack by direction and the point-blank oil, shipped
    # after the incident.
    ("attack_confused_by_direction", 0), ("attack_throw_point_blank_divisor", 0),
    # #983: testing unknown potions when safe, shipped after the incident.
    ("item_test_unknown_potions", 0),
    # #1034 enabled the HP floor at 0 HP after these incidents were recorded.
    ("kill_live_zero_hp", 0),
    # The incidents also predate the Home stock and known-unwanted policies.
    ("town_home_stack_limit", 0), ("town_home_reject_unwanted", 0),
    # #1046 shipped the lone-slow bundle (#878) after these incidents.
    ("lone_slow_attack", 0), ("escape_phase_lost_damage_only", 0),
    ("attack_reserve_spell_damage_only", 0), ("attack_reserve_faint_percent", 0),
    ("caution_unseen_bump_phase_turns", 0), ("caution_unseen_bump_phase_min_danger", 0),
    # #1056 shipped attack_reserve_spell_overexert = 0 after these incidents.
    ("attack_reserve_spell_overexert", 1),
    # #1072 shipped the #926 fast-fragile bundle after these incidents.
    ("fast_fragile_heal_floor", 0), ("fast_fragile_escape_quantile", 0), ("fast_fragile_count_dot", 0),
    # #1078 shipped the engine's light radius after these incidents. Without
    # this pin the games take another route and never reach the incident grids.
    ("cave_light_engine_radius", 0),
    # #1094 shipped #1066's two Phase Door switches after these incidents.
    ("escape_phase_unknown_scary", 0), ("escape_phase_newcomer_wait", 0),
    # #1100 shipped the position belief (#879) at decay 5 after these incidents.
    ("unseen_belief", 0), ("unseen_decay_percent", 20),
    # #1085 shipped breadth-first exploration after these incidents.
    ("explore_breadth_first", 0),
    # #1067 shipped the trap rules after these incidents.
    ("trap_recover", 0), ("trap_way_back", 0), ("trap_leave_level_count", 0),
    # #1123 enabled the retreat guards after these incidents.
    ("caution_retreat_reentry_ticks", 0), ("caution_retreat_oscillation_limit", 0),
    # #1061 shipped stair_scum (an exploration mode under 20 max HP) after
    # these incidents; it keeps a CL1 Borg by the stairs and changes the route.
    ("stair_scum", 0),
    # #1062 shipped the stair-leash switches after these incidents.
    ("flow_leash_any_stair", 0), ("flow_leash_unknown_stair_steps", 0), ("flow_leash_teleported_recall", 0),
    # #1098 shipped the mixed walk and the C exploration's stair exit after
    # these incidents (owner, 2026-11-23: "merge it but with all settings on").
    ("explore_mixed", 0), ("explore_c_leave_when_done", 0),
] + [(f"town_staple_{kind}", 0) for kind in ("clw", "csw", "ccw", "phase", "oil", "food", "light")] + [
    (f"town_staple_{kind}__{cls}", 0) for cls in ("warrior", "mage") for kind in ("clw", "csw", "phase", "oil")
]


@unittest.skipUnless(GAME.is_file(), "harness/build.sh required")
class StairHandoffTest(unittest.TestCase):
    def run_case(self, seed, player_class, stop_turn):
        with tempfile.TemporaryDirectory(prefix="stair-handoff-") as directory:
            out = Path(directory) / "game"
            env = {key: val for key, val in os.environ.items()
                   if not key.startswith(("BORG_", "BORGH_", "ANGBAND_BIN"))}
            if cemetery := os.environ.get("BORG_CEMETERY_DIR"):
                env["BORG_CEMETERY_DIR"] = cemetery
            # The exact incident turns/grids below were captured before #859.
            # Pin its five new town switches off (and #863's, PINNED_TOWN_SETTINGS) and use the scary list at
            # f57499d0, before #869's grey-mushroom attack entry; #880's Recall
            # forecast is pinned to #758's rule (the incidents predate it). These settings
            # preserve the recorded incident turns/grids asserted below.
            env.update(
                BORG_CORE="borg-rs", BORG_CORE_TRACE="off",
                BORG_SCARY_LIST=str(ROOT / "harness/fixtures/stair-handoff-scary-monsters.txt"),
                BORG_TXT_SET=";".join(f"borg_param_{name}={value}" for name, value in PINNED_TOWN_SETTINGS),
            )
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


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class EmergencyStairHandoffTest(unittest.TestCase):
    def run_case(self, case, weight, player_class=6, depth=5):
        with tempfile.TemporaryDirectory(prefix="emergency-stair-handoff-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # #1061: the stair_scum mode (shipped after these cases) is not
            # what they test; pin it off.
            # #1098: nor the mixed walk and the C exploration's stair exit.
            for key in ("borg_param_stair_scum", "borg_param_explore_mixed", "borg_param_explore_c_leave_when_done"):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = 0", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = 0\n"
            for name, value in {"stairs_flight_threat_weight": weight, "desperate_adjacent_down_distance": 3}.items():
                key = f"borg_param_caution_{name}"
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_STAIR_HANDOFF=case)
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "3", "-class", str(player_class), "-menu-letters", "-start-depth", str(depth), "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f"emergency stair handoff passed: {case}", result.stdout)

    def test_flight_moves_past_the_two_floor_squares_and_descends(self):
        for weight in (0, 5):
            for case in ("one-step", "two-steps"):
                with self.subTest(weight=weight, case=case):
                    self.run_case(case, weight)

    def test_dark_exploration_cannot_keep_restarting_the_same_stair_search(self):
        self.run_case("interrupted-search", 5)

    def test_dark_exploration_cannot_keep_restarting_the_two_step_floor_search(self):
        self.run_case("interrupted-floor-search", 5, player_class=5, depth=3)

    def test_breeder_rest_veto_prevents_new_and_continued_recovery_routes(self):
        self.run_case("breeder-recovery", 5, player_class=1, depth=1)

    def test_unweighted_handoff_cannot_walk_toward_a_farther_stair(self):
        self.run_case("blocked", 0)


if __name__ == "__main__":
    unittest.main()

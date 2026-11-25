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
    # #1250: keep the original use and timing of Magic Mapping.
    ("item_use_idle", 0),
    # #1253: keep the recorded inventory actions and the turns they took.
    ("attack_throw_junk_potions", 0), ("junk_discard_harmful_in_town", 0),
    ("town_departure_free_weight", 0), ("town_departure_free_slots", 0),
    ("town_store_keep_min", 0), ("town_recall_curve_scale_percent", 0),
    ("town_recall_urgent", 0),
    # #1239 lets Town acquisitions exceed the old weight allowance. Keep
    # the shopping rules that lead to the recorded stair incident grids.
    ("town_shop_overloaded", 0), ("town_loadout_power_loss", 0),
    ("town_prepared_percent", 0), ("town_variety_min_depth", 0),
    ("town_recall_reserve", 1), ("buy_recall_from_gold", 400),
    ("town_phase_reserve", 0), ("town_phase_reserve_percent", 100),
    ("town_device_percent", 0),
    ("caution_recall_blow_model", 0),
    # #901: the confused attack by direction and the point-blank oil, shipped
    # after the incident.
    ("attack_confused_by_direction", 0), ("attack_throw_point_blank_divisor", 0),
    # #1034 enabled the HP floor at 0 HP after these incidents were recorded.
    ("kill_live_zero_hp", 0),
    # #1211 shipped the recovery-loop switches after these incidents: the
    # prep-buff hold and the full-HP mana rest move the long runs' turns.
    ("defend_prep_mana_starved", 0), ("recover_mana_rest_full_hp", 0),
    # The incidents predate the Home stock, known-unwanted and reserve policies.
    ("town_home_stack_limit", 0), ("town_home_reject_unwanted", 0), ("town_home_reserve", 0),
    # The incident paths predate the three standing and descent fixes.
    ("caution_note_standing_on_stairs", 0),
    ("stairs_descent_gates_without_upstair", 0),
    ("caution_standing_descent_gated", 0),
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
    # #995 changed hallway lighting after the recorded stair incidents.
    ("light_hallway_repeat_guard", 0),
    # #1094 shipped #1066's two Phase Door switches after these incidents.
    ("escape_phase_unknown_scary", 0), ("escape_phase_newcomer_wait", 0),
    # #1100 shipped the position belief (#879) at decay 5 after these incidents.
    ("unseen_belief", 0), ("unseen_decay_percent", 20),
    # #1128 shipped #885's dark-floor light and #917's rubble sight after
    # these incidents.
    ("update_dark_clears_light", 0), ("view_rubble_blocks_sight", 0),
    # #1085 shipped breadth-first exploration after these incidents.
    ("explore_breadth_first", 0),
    # #1067 shipped the trap rules after these incidents.
    ("trap_recover", 0), ("trap_way_back", 0), ("trap_leave_level_count", 0),
    # #1123 enabled the retreat guards after these incidents.
    ("caution_retreat_reentry_ticks", 0), ("caution_retreat_oscillation_limit", 0),
    # #1061 shipped stair_scum (an exploration mode under 20 max HP) after
    # these incidents; it keeps a CL1 Borg by the stairs and changes the route.
    ("stair_scum", 0),
    # #1148 shipped the walk-away from immobile monsters after these incidents.
    ("caution_walk_away_immobile", 0),
    # #1062 shipped the stair-leash switches after these incidents.
    ("flow_leash_any_stair", 0), ("flow_leash_unknown_stair_steps", 0), ("flow_leash_teleported_recall", 0),
    # #1098 shipped the mixed walk and the C exploration's stair exit after
    # these incidents (owner, 2026-11-23: "merge it but with all settings on").
    ("explore_mixed", 0), ("explore_c_leave_when_done", 0),
    # #1130 shipped the committed brave stair walk after these incidents.
    ("flow_brave_walk_commit", 0),
    # #1140 shipped deal_with_poison (the poison triage) after these
    # incidents (owner, 2026-11-23: "merge with deal_with_poison 1").
    ("deal_with_poison", 0),
    # #941: the speed flag synced to the engine timer, shipped after the
    # incident (seed 224 drinks Speed at 24,213; at 0 the flag stays set to the
    # end, which the recorded turns depend on).
    ("trait_timed_sync", 0),
    # #1184 shipped ranged shopping and resupply after these incidents.
    ("town_ranged_readiness", 0), ("trip_ranged_readiness", 0),
    # These recorded routes predate the #1194 combat-healing policy.
    ("caution_combat_heal", 0),
    # #1159 changes the path to the incident. Keep the original route
    # selection for these exact-turn replays.
    # #1214 adds a town return when healing potions run low.
    ("restock_healing", 0),
    ("caution_rising_skip_held_stairs", 0),
    # #1200 shipped surplus throwing after these incidents. Use the
    # original throw range and scoring to reach the recorded stair grids.
    ("attack_throw_surplus", 0),
    # #1236 changed ordinary spare disposal after these exact-turn replays.
    ("junk_inferior_spares", 0),
    # #1217 changed shallow preparedness after these recorded routes.
    ("prepared_dl2_min_hp", 30),
    ("prepared_dl2_min_clevel__warrior", 0), ("prepared_dl2_min_clevel__mage", 0),
    ("prepared_dl34_min_hp__warrior", 50), ("prepared_dl34_min_clevel__warrior", 4),
    ("prepared_dl34_min_hp__mage", 60), ("prepared_dl34_min_clevel__mage", 11),
    ("prepared_dl59_min_hp__warrior", 60), ("prepared_dl59_min_clevel__warrior", 6),
    ("prepared_dl59_min_hp__mage", 80), ("prepared_dl59_min_clevel__mage", 15),
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
        self.assertIn(["7446", "1", "38", "0"], keys)
        self.assertFalse(any(" 7456 0 # Flowing toward down-stairs at cost 1" in row for row in messages))
        self.assertTrue(any(" 7456 0 # Fleeing the level (recall)" in row for row in messages))
        self.assertEqual(state_at(states, 7466)[0][1], "41,47")

    def test_ordinary_fallback_does_not_restart_down_stair_loop(self):
        messages, states, keys = self.run_case(204, 1, 119000)
        self.assertTrue(any(" 118839 0 # Standing on dn-stairs" in row for row in messages))
        self.assertTrue(any(" 118839 0 # Flowing toward dark-" in row for row in messages))
        self.assertEqual(state_at(states, 118839)[0][1], "129,22")
        self.assertEqual(state_at(states, 118849)[0][1], "129,21")
        self.assertIn(["118839", "1", "38", "0"], keys)
        # The first patch reached (129,20), reversed twice, and started over.
        # Check the whole window so a three-square loop also fails the test.
        for turn in (118859, 118869, 118879):
            self.assertNotIn("129,22", [state[1] for state in state_at(states, turn)])
        self.assertTrue(state_at(states, 118879))

    def test_known_up_route_continues_past_adjacent_down_stair(self):
        messages, states, keys = self.run_case(224, 0, 144000)
        self.assertTrue(any(" 143021 0 # Standing on dn-stairs" in row for row in messages))
        self.assertTrue(any(" 143021 0 # Flowing toward up-stairs at cost 4" in row for row in messages))
        self.assertEqual(state_at(states, 143021)[0][1], "162,35")
        self.assertEqual(state_at(states, 143031)[0][1], "163,34")
        self.assertIn(["143021", "1", "39", "0"], keys)

        later = state_at(states, 143041)
        self.assertTrue(later)
        self.assertNotIn("162,35", [state[1] for state in later])
        self.assertTrue(any(int(turn) > 143041 and key == "3c" for turn, _, key, _ in keys))
        self.assertTrue(any(int(state[0]) > 143041 and state[2].startswith("d0/") for state in states))


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class EmergencyStairHandoffTest(unittest.TestCase):
    def run_case(self, case, weight=None, player_class=6, depth=5, **settings):
        with tempfile.TemporaryDirectory(prefix="emergency-stair-handoff-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            if case in ("one-step", "two-steps", "blocked", "rising-hold"):
                # These flight maps model the original descent policy. The
                # synthetic characters lack supplies for the next depth.
                # MoreRefusedGatesTest covers the gate with both values.
                key = "borg_param_stairs_descent_gates_without_upstair"
                config = re.sub(fr"^{key} = .*$", f"{key} = 0", config, flags=re.M)
            if weight is not None:
                # #1061: the stair_scum mode (shipped after these cases) is not
                # what they test; pin it off.
                # #1098: nor the mixed walk and the C exploration's stair exit.
                # #1140: nor the poison triage (deal_with_poison).
                for key in ("borg_param_stair_scum", "borg_param_explore_mixed", "borg_param_explore_c_leave_when_done",
                            "borg_param_deal_with_poison"):
                    config, changed = re.subn(fr"^{key} = .*$", f"{key} = 0", config, flags=re.M)
                    if changed == 0:
                        config += f"\n{key} = 0\n"
                caution = {"stairs_flight_threat_weight": weight, "desperate_adjacent_down_distance": 3}
                for name, value in (caution | settings).items():
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

    def test_pursuit_budget_checks_the_reachable_route_and_its_handoff(self):
        cases = (("pursuit-budget-off", 0), ("pursuit-budget", 100),
                 ("pursuit-budget-slowed", 100), ("pursuit-budget-handoff", 100))
        for weight in (0, 5):
            for case, budget in cases:
                with self.subTest(weight=weight, case=case):
                    self.run_case(case, weight, desperate_walk_cost_percent=budget)

    def test_known_up_handoff_with_shipped_settings(self):
        self.run_case("known-up-handoff", player_class=0, depth=2)

    def test_stair_handoff_descends_with_original_descent_policy(self):
        self.run_case("one-step")

    def test_rising_hold_filters_and_releases_down_stair_flight(self):
        self.run_case("rising-hold", player_class=0, depth=2)

    def test_rising_hold_cancels_an_unweighted_stair_flow(self):
        self.run_case("rising-hold", 0, player_class=0, depth=2)

    def test_dark_exploration_cannot_keep_restarting_the_same_stair_search(self):
        self.run_case("interrupted-search")

    def test_dark_exploration_cannot_keep_restarting_the_two_step_floor_search(self):
        self.run_case("interrupted-floor-search", 5, player_class=5, depth=3)

    def test_breeder_rest_veto_prevents_new_and_continued_recovery_routes(self):
        self.run_case("breeder-recovery", 5, player_class=1, depth=1)

    def test_unweighted_handoff_cannot_walk_toward_a_farther_stair(self):
        self.run_case("blocked", 0)


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class HealingRestockTest(unittest.TestCase):
    def run_case(self, case, depth, live=True):
        with tempfile.TemporaryDirectory(prefix="healing-restock-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            if not live:
                config = config.replace("borg_town_plan = 1", "borg_town_plan = 0")
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_STAIR_HANDOFF=f"healing-restock-{case}")
            if case.startswith("town"):
                # Isolate purchase execution from the unrelated town-monster flight.
                env.update(BORG_PARAM_CAUTION_TOWN_NEVER_SCARY="1",
                           BORG_PARAM_CAUTION_SCARY_NAMED_MONSTERS="0",
                           BORG_PARAM_CAUTION_SCARY_DAMAGE_PERCENT="0")
            options = ["-start-depth", str(depth)] if depth else []
            messages = Path(directory) / "messages.log"
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{name}={path}" for name, path in paths.items()),
                 "--", "-seed", "101", "-race", "3", "-class", "0", "-menu-letters", *options,
                 "-msglog", str(messages), "-keys", "300"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f"#1214 healing restock passed: {case}", result.stdout)
            return messages.read_text()

    def test_shallow_depletion_blocked_route_and_retry_funding(self):
        self.run_case("dungeon", 2)

    def test_short_reserve_returns_while_one_potion_remains(self):
        self.run_case("short", 2)

    def test_affordable_restock_uses_the_native_store(self):
        for live in (True, False):
            with self.subTest(live=live):
                self.run_case("town", 0, live)

    def test_legacy_restock_sells_the_unknown_potion_before_buying(self):
        messages = self.run_case("town-legacy-unknown", 0, live=False)
        sale = re.search(r"You had a Potion of Cure Light Wounds", messages)
        purchase = re.search(r"You bought [^\n]*\bPotion\b", messages)
        self.assertIsNotNone(sale)
        self.assertIsNotNone(purchase)
        self.assertLess(sale.start(), purchase.start())


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class LeaveLevelUpstairsTest(unittest.TestCase):
    """#1173: the dungeon decision's leave entry point on an exhausted level.

    The recorded Ranger (Borg261124105138, track-p c5501d3c) spent 521
    decisions walking between secret-door search targets on DL3 with two
    reachable up stairs and no down stair: the power dive set the direction
    before the exhausted branch could allow the fallback to up stairs.
    """

    def run_case(self, case):
        with tempfile.TemporaryDirectory(prefix="leave-level-upstairs-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # #1061's stair_scum, #1098's mixed walk and #1140's poison
            # triage are not what the case tests. The prepared_* gates make
            # a birth-level character dive-prone (the incident's CL8 Ranger
            # was prepared for DL3 and for DL8); the breadth-first and
            # mixed walks are pinned off so the level's exhaustion is C's
            # ("nothing reachable left within the leash"), which needs
            # explore_c_leave_when_done = 1.
            pinned = [
                # The birth Warrior's Berserk Strength would consume the
                # turn whose stair-flow decision this fixture inspects.
                ("item_use_idle", 0),
                ("stair_scum", 0), ("explore_mixed", 0), ("deal_with_poison", 0),
                ("explore_breadth_first", 0),
                ("explore_c_leave_when_done", 1),
                ("prepared_dl2_min_clevel", 0), ("prepared_dl2_min_clevel__warrior", 0),
                ("prepared_dl2_min_hp", 0), ("prepared_dl34_min_hp", 0),
                ("prepared_dl34_min_clevel", 0), ("prepared_dl59_min_hp", 0),
                ("prepared_dl59_min_clevel", 0), ("prepared_cure_min", 0),
                ("prepared_recall_depth", 0),
                # the per-class values override the base setting
                ("prepared_dl34_min_hp__warrior", 0), ("prepared_dl34_min_clevel__warrior", 0),
                ("prepared_dl59_min_hp__warrior", 0), ("prepared_dl59_min_clevel__warrior", 0),
                # a birth-level character passes the clevel gate only with the margin
                ("prepared_clevel_depth_margin", 7),
            ]
            for key, value in pinned:
                name = f"borg_param_{key}"
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{name} = {value}\n"
            # the trip policy's own prepared estimate would ascend a birth
            # character regardless of the case's decision state. Replaced in
            # place: a line after the file's formula section is not read as
            # a setting.
            for key, value in (("borg_trip_policy", 0),):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF=case)
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters", "-start-depth", "3", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("leave-level-upstairs passed (#1173)", result.stdout)

    def test_exhausted_dive_falls_back_to_a_reachable_up_stair(self):
        self.run_case("leave-level-upstairs")


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class StaleStairMoreTest(unittest.TestCase):
    """#1178: caution's standing descent on a stale stair_more flag.

    stair_more is set before the descent flows it belongs to and survives
    their failure until a new level. A later think standing on a FEAT_MORE
    grid then takes the stair on the stale flag at 0 (C's
    defect); at 1 caution's standing take applies more_refused's descent
    gates and the hungry character refuses it.
    """

    def run_case(self, gated):
        with tempfile.TemporaryDirectory(prefix="stale-stair-more-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # The prepared_* gates make a birth-level character fully
            # prepared for its depth and for five deeper, as in
            # LeaveLevelUpstairsTest: prepared for its own depth keeps the
            # "suitable level" leash (it fires for a character unprepared
            # for the depth it is on) and the wander leash out, and
            # prepared for five deeper is the dive's own trigger
            # ("playing too shallow"). The exploration modes and the poison
            # triage are not what the case tests;
            # explore_c_leave_when_done 0 keeps the exhaustion out.
            pinned = [
                ("item_use_idle", 0),
                ("stair_scum", 0), ("explore_mixed", 0), ("deal_with_poison", 0),
                ("explore_breadth_first", 0), ("explore_c_leave_when_done", 0),
                ("prepared_dl2_min_clevel", 0), ("prepared_dl2_min_clevel__warrior", 0),
                ("prepared_dl2_min_hp", 0), ("prepared_dl34_min_hp", 0),
                ("prepared_dl34_min_clevel", 0), ("prepared_dl59_min_hp", 0),
                ("prepared_dl59_min_clevel", 0), ("prepared_cure_min", 0),
                ("prepared_recall_depth", 0),
                # the per-class values override the base setting
                ("prepared_dl34_min_hp__warrior", 0), ("prepared_dl34_min_clevel__warrior", 0),
                ("prepared_dl59_min_hp__warrior", 0), ("prepared_dl59_min_clevel__warrior", 0),
                # a birth-level character passes the clevel gate only with the margin
                ("prepared_clevel_depth_margin", 7),
                # the switch under test
                ("caution_standing_descent_gated", gated),
                # #1178 review: the fresh-emergency phase needs the leave
                # rule on (a recovering Borg on the staircase a monster
                # threatens takes the stairs); it ships 0.
                ("caution_stairs_leave_on_monster", 1),
            ]
            for key, value in pinned:
                name = f"borg_param_{key}"
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{name} = {value}\n"
            # the trip policy's own prepared estimate would ascend a birth
            # character regardless of the case's decision state. Replaced in
            # place: a line after the file's formula section is not read as
            # a setting.
            for key, value in (("borg_trip_policy", 0),):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF="stale-stair-more")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters", "-start-depth", "3", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("stale-stair-more passed (#1178)", result.stdout)

    def test_shipped_off_the_stale_flag_takes_the_standing_descent(self):
        self.run_case(0)

    def test_gated_on_the_stale_flag_is_refused(self):
        self.run_case(1)


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class MoreRefusedGatesTest(unittest.TestCase):
    """#1177: the descent gates without a seen up staircase.

    more_refused's four descent refusals sit behind C's wrapper "if there
    are no down stairs, don't filter use of up stairs": with no up staircase
    seen, a hungry birth-level Warrior dives past the hunger gate and takes
    a reachable down staircase at 0; at 1 the gates apply and
    the descent is refused.
    """

    def run_case(self, gates):
        with tempfile.TemporaryDirectory(prefix="more-refused-gates-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # The prepared_* gates make a birth-level character dive-prone,
            # as in LeaveLevelUpstairsTest. The exploration modes and the
            # poison triage are not what the case tests;
            # explore_c_leave_when_done = 1 lets the underfoot phases mark
            # the level exhausted for the leave decision's own exhausted
            # branch.
            pinned = [
                ("item_use_idle", 0),
                ("stair_scum", 0), ("explore_mixed", 0), ("deal_with_poison", 0),
                ("explore_breadth_first", 0), ("explore_c_leave_when_done", 1),
                ("prepared_dl2_min_clevel", 0), ("prepared_dl2_min_clevel__warrior", 0),
                ("prepared_dl2_min_hp", 0), ("prepared_dl34_min_hp", 0),
                ("prepared_dl34_min_clevel", 0), ("prepared_dl59_min_hp", 0),
                ("prepared_dl59_min_clevel", 0), ("prepared_cure_min", 0),
                ("prepared_recall_depth", 0),
                # the per-class values override the base setting
                ("prepared_dl34_min_hp__warrior", 0), ("prepared_dl34_min_clevel__warrior", 0),
                ("prepared_dl59_min_hp__warrior", 0), ("prepared_dl59_min_clevel__warrior", 0),
                # a birth-level character passes the clevel gate only with the margin
                ("prepared_clevel_depth_margin", 7),
                # the switch under test
                ("stairs_descent_gates_without_upstair", gates),
            ]
            for key, value in pinned:
                name = f"borg_param_{key}"
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{name} = {value}\n"
            # the trip policy's own prepared estimate would ascend a birth
            # character regardless of the case's decision state, and with
            # the town plan's capability mode the notice does not count
            # plain rations as food (the descent gates would then have no
            # food lever at all). Both replaced in place: a line after the
            # file's formula section is not read as a setting.
            for key, value in (("borg_trip_policy", 0), ("borg_town_plan", 0)):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF="more-refused-gates")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters", "-start-depth", "3", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("more-refused-gates passed (#1177)", result.stdout)

    def test_shipped_off_the_hungry_dive_takes_the_reachable_down_stair(self):
        self.run_case(0)

    def test_gates_on_the_hungry_dive_is_refused(self):
        self.run_case(1)



@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class NoteStandingFlagsTest(unittest.TestCase):
    """#1176: caution's danger + standing-on-up-stair trigger.

    note_standing's on_upstair/on_dnstair are dead in C (declared false,
    only ever assigned false), which mutes the danger arm of the
    choose_stairs trigger: a low-level character in real danger standing on
    a tracked up staircase uses the escape path at 0. At 1 the standing
    flags let caution take the up stairs first.
    """

    def run_case(self, flags_on):
        with tempfile.TemporaryDirectory(prefix="note-standing-flags-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # The prepared_* gates keep the character's preparedness clean
            # for choose_stairs's own prepared arm. The exploration modes
            # and the poison triage are not what the case tests.
            pinned = [
                ("stair_scum", 0), ("explore_mixed", 0), ("deal_with_poison", 0),
                ("explore_breadth_first", 0), ("explore_c_leave_when_done", 0),
                ("prepared_dl2_min_clevel", 0), ("prepared_dl2_min_clevel__warrior", 0),
                ("prepared_dl2_min_hp", 0), ("prepared_dl34_min_hp", 0),
                ("prepared_dl34_min_clevel", 0), ("prepared_dl59_min_hp", 0),
                ("prepared_dl59_min_clevel", 0), ("prepared_cure_min", 0),
                ("prepared_recall_depth", 0),
                # the per-class values override the base setting
                ("prepared_dl34_min_hp__warrior", 0), ("prepared_dl34_min_clevel__warrior", 0),
                ("prepared_dl59_min_hp__warrior", 0), ("prepared_dl59_min_clevel__warrior", 0),
                # a birth-level character passes the clevel gate only with the margin
                ("prepared_clevel_depth_margin", 7),
                # the switch under test
                ("caution_note_standing_on_stairs", flags_on),
            ]
            for key, value in pinned:
                name = f"borg_param_{key}"
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{name} = {value}\n"
            # the trip policy's own prepared estimate would ascend a birth
            # character regardless of the case's decision state. Replaced in
            # place: a line after the file's formula section is not read as
            # a setting.
            for key, value in (("borg_trip_policy", 0),):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF="note-standing-flags")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters", "-start-depth", "3", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("note-standing-flags passed (#1176)", result.stdout)

    def test_shipped_off_the_dead_trigger_takes_no_stair(self):
        self.run_case(0)

    def test_flags_on_the_danger_standing_trigger_takes_the_up_stair(self):
        self.run_case(1)


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class DarkVeinLoopTest(unittest.TestCase):
    """Recorded treasure vein loops driven through production exploration.

    #1189: the recorded DL1 vein loop driven through the production
    exploration — borg_think_dungeon on the fabricated recorded region
    (run_dark_vein_loop_regression, borg_think_dungeon_util.rs), not a test
    copy of the candidate filter. The recorded Half-Troll Warrior
    (Borg261124143216, the issue's evidence) walked 444666 between
    (113,44) and (110,44) for 4,000 turns: ring 10 the treasure vein at
    (109,44), ring 11 the vein at (114,45). With explore_stepable_goals = 1
    (shipped) the veins are no goals and the Borg walks out of the corridor
    (or leaves the level when nothing is left); with 0 the first think
    commits twice — the recorded cost-1 aim beside the vein whose step
    never comes, then the aim back east — and the corridor loop repeats.

    #1227 uses the recorded DL4 corridor to check legacy exploration with
    the filter off, with it on, and with only a reachable up stair left.
    """

    def run_case(self, setting, *, legacy=False):
        planner = "legacy" if legacy else "dark"
        case = f"{planner}-vein-loop-{setting}"
        with tempfile.TemporaryDirectory(prefix=f"{planner}-vein-loop-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # #1189 used breadth-first before #1201 switched it off.
            # The legacy Paladin exceeds its shipped depth-adjusted mixed cap.
            pins = [
                ("borg_param_item_use_idle", 0),
                ("borg_param_explore_stepable_goals", int(setting != "off")),
                ("borg_param_explore_breadth_first", 0 if legacy else 1),
                ("borg_param_stair_scum", 0),
            ]
            if legacy:
                pins += [
                    ("borg_param_explore_mixed", 1),
                    ("borg_param_explore_mix_max_clevel__paladin", 8),
                ]
            for name, value in pins:
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                assert changed, f"the shipped file ships {name}"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF=case)
            # Recorded characters: Half-Elf Paladin at DL4 (#1227), or
            # Half-Troll Warrior at DL1 (#1189). The fixtures pin the traits.
            race, player_class, depth = ("1", "5", "4") if legacy else ("7", "0", "1")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", race, "-class", player_class,
                 "-menu-letters", "-start-depth", depth, "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f"{planner} vein loop passed: {setting}", result.stdout)

    def test_stepable_goals_walks_out_of_the_recorded_corridor(self):
        self.run_case("on")

    def test_without_the_switch_the_recorded_loop_repeats(self):
        self.run_case("off")

    def test_legacy_goals_reach_the_recorded_frontier(self):
        self.run_case("on", legacy=True)

    def test_legacy_goals_with_filter_off_return_to_the_vein(self):
        self.run_case("off", legacy=True)

    def test_legacy_exhaustion_hands_off_to_a_usable_up_stair(self):
        self.run_case("exit", legacy=True)


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class BreederDoorLoopTest(unittest.TestCase):
    """#1206: the breeder containment must not close the door the flight
    just opened.

    close_breeder_door skips only the currently selected next grid; the
    flow replans after each terrain change, so the just-opened door lost
    its protection and was closed again, then re-opened — 35 open/close
    pairs on the recorded Kobold Mage. The fixture runs the flight turn by
    turn, applying the door state after each action: the path's door opens
    exactly once, the unrelated open door beside it stays closed
    (the containment still works), and the Borg walks past the door.
    """

    def run_case(self):
        with tempfile.TemporaryDirectory(prefix="breeder-door-loop-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # The exploration modes and the poison triage are not what the
            # case tests.
            pinned = [
                ("stair_scum", 0), ("explore_mixed", 0), ("deal_with_poison", 0),
                ("explore_breadth_first", 0), ("explore_c_leave_when_done", 0),
            ]
            for key, value in pinned:
                name = f"borg_param_{key}"
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{name} = {value}\n"
            for key, value in (("borg_trip_policy", 0),):
                config, changed = re.subn(fr"^{key} = .*$", f"{key} = {value}", config, flags=re.M)
                if changed == 0:
                    config += f"\n{key} = {value}\n"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF="breeder-door-loop")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "0", "-menu-letters", "-start-depth", "3", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("breeder-door-loop passed (#1206)", result.stdout)

    def test_the_containment_never_closes_the_flight_door_again(self):
        self.run_case()


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class StaleKoboldDoorTest(unittest.TestCase):
    def test_alter_reconciles_the_stale_target_and_the_stair_route_continues(self):
        with tempfile.TemporaryDirectory(prefix="stale-kobold-door-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            for name in ("borg.txt", "scary_monsters.txt"):
                shutil.copyfile(ROOT / "harness" / name, paths["user"] / name)
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_CORE_TRACE="off", BORG_TEST_STAIR_HANDOFF="stale-kobold-door")
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "8", "-class", "6", "-menu-letters", "-start-depth", "1", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("stale-kobold-door passed (#1229)", result.stdout)


@unittest.skipUnless(TEST_GAME.is_file(), "test-stair-handoff build required")
class RecoveryManaLoopTest(unittest.TestCase):
    """#1211: the Priest pacing between recovery grids on DL19, spending the
    regenerated mana on Heroism.

    The recorded Priest (Borg261124230918, CL26, full HP, SP 0-2/47, five
    remembered orc archers pricing the corridor at danger 20-25) cast every
    regenerated point into Heroism (defense type 12, "some danger") and paced
    between recovery grids for 438 decisions: the mana rest's own danger gate
    (17) refused every grid the walk arrived at. defend_prep_mana_starved
    holds the prep casts while the mana is what the walk is for;
    recover_mana_rest_full_hp lets the full-HP mana rest read the general
    recovery gate the Borg's other recovery decisions already trust. The
    stale duplicate DarkVeinLoopTest from the #1197 sync is removed here.
    """

    def run_case(self, on):
        with tempfile.TemporaryDirectory(prefix="recovery-mana-loop-") as directory:
            paths = {kind: Path(directory) / kind for kind in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            config = (ROOT / "harness/borg.txt").read_text()
            # The switches under test, replaced in place (they ship 1), and
            # the modes that are not what the case tests.
            pinned = [
                ("borg_param_defend_prep_mana_starved", 1 if on else 0),
                ("borg_param_recover_mana_rest_full_hp", 1 if on else 0),
                ("borg_param_stair_scum", 0), ("borg_param_explore_mixed", 0),
                ("borg_param_deal_with_poison", 0), ("borg_param_explore_c_leave_when_done", 0),
            ]
            for name, value in pinned:
                config, changed = re.subn(fr"^{name} = .*$", f"{name} = {value}", config, flags=re.M)
                assert changed, f"the shipped file ships {name}"
            (paths["user"] / "borg.txt").write_text(config)
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: val for key, val in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_TEST_STAIR_HANDOFF=f"recovery-mana-loop-{'on' if on else 'off'}")
            # The recorded character: a Human Priest (class 3) at DL19. Its
            # birth kit feeds the think; the fixture pins the traits, the
            # spell and the five remembered orcs.
            result = subprocess.run(
                [str(TEST_GAME), "-mborgh", "-n", *(f"-d{kind}={path}" for kind, path in paths.items()),
                 "--", "-seed", "101", "-race", "0", "-class", "3", "-menu-letters", "-start-depth", "1", "-keys", "250"],
                cwd=TEST_GAME.parent, env=env, capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn(f"recovery mana loop passed: {'on' if on else 'off'}", result.stdout)

    def test_the_recorded_loop_reproduces_with_the_parameters_off(self):
        self.run_case(False)

    def test_the_cast_is_declined_and_the_borg_rests_with_the_parameters_on(self):
        self.run_case(True)

if __name__ == "__main__":
    unittest.main()

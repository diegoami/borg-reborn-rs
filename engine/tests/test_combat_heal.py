"""Native decision regressions for the three watched deaths in #1194."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust-test/game/angband"
CASES = {
    "phase-pending-poison": (7, 7, 2),
    "tengu-early": (7, 0, 5),
    "tengu-light": (7, 0, 5),
    "tengu-phase-fight": (7, 0, 5),
    "tengu-unsafe-phase": (7, 0, 5),
    "tengu-recall": (7, 0, 5),
    "paladin-wand": (0, 5, 8),
    "paladin-poison": (0, 5, 8),
    "rogue-fight-race": (3, 6, 5),
}


@unittest.skipUnless(GAME.is_file(), "test-combat-heal build required")
class CombatHealTest(unittest.TestCase):
    def run_case(self, case, enabled, pending_poison=False):
        race, player_class, depth = CASES[case]
        with tempfile.TemporaryDirectory(prefix="combat-heal-") as directory:
            paths = {name: Path(directory) / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            write_pborg_config(paths["user"])
            shutil.copyfile(ROOT / "harness/scary_monsters.txt", paths["user"] / "scary_monsters.txt")
            env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
            env.update(
                BORG_CORE="borg-rs",
                BORG_TEST_COMBAT_HEAL=case,
                BORG_PARAM_CAUTION_COMBAT_HEAL=str(int(enabled)),
                BORG_PARAM_ATTACK_UNKNOWN_DEVICE_PARANOIA_CHANCE="0",
            )
            if case == "tengu-early":
                # Reproduce the early-healing arm at 175/225 HP.
                env["BORG_PARAM_CAUTION_COMBAT_HEAL_MIN_LOSS"] = "20"
            if case == "phase-pending-poison":
                env.update(
                    BORG_PARAM_ESCAPE_PHASE_ADJACENT_RULE="1",
                    BORG_PARAM_ESCAPE_PHASE_LOST_TEST="0",
                    BORG_PARAM_ESCAPE_PHASE_LOST_DAMAGE_ONLY="0",
                    BORG_PARAM_ESCAPE_PHASE_LOST_DANGER_PERCENT="100",
                    BORG_PARAM_ESCAPE_PHASE_NEWCOMER_WAIT="0",
                    BORG_PARAM_ESCAPE_PHASE_UNKNOWN_SCARY="1",
                    BORG_PARAM_ESCAPE_PHASE_UNKNOWN_HP_PERCENT="50",
                    BORG_PARAM_ESCAPE_PHASE_LOST_POISON_PENDING=str(int(pending_poison)),
                )
            run = subprocess.run(
                [str(GAME), "-mborgh", "-n", *(f"-d{key}={value}" for key, value in paths.items()),
                 "--", "-seed", "101", "-race", str(race), "-class", str(player_class),
                 "-menu-letters", "-start-depth", str(depth), "-keys", "250"],
                cwd=GAME.parent, env=env, capture_output=True, text=True, timeout=30,
            )
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        result = "#1083 Phase verdict" if case == "phase-pending-poison" else "#1194 combat healing"
        self.assertIn(f"{result} passed: {case}", run.stdout)

    def test_combat_healing(self):
        for case in CASES:
            if case == "phase-pending-poison":
                continue
            with self.subTest(case=case):
                self.run_case(case, enabled=True)

    def test_phase_uses_the_engine_poison_budget_only_when_enabled(self):
        for pending_poison in (False, True):
            with self.subTest(pending_poison=pending_poison):
                self.run_case("phase-pending-poison", enabled=True, pending_poison=pending_poison)

    def test_disabled_retains_the_original_choices(self):
        for case in ("tengu-unsafe-phase", "paladin-wand", "paladin-poison", "rogue-fight-race"):
            with self.subTest(case=case):
                self.run_case(case, enabled=False)


if __name__ == "__main__":
    unittest.main()

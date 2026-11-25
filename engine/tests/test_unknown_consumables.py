"""Verify identification by town sale and deliberate use of known items."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game/angband"


@unittest.skipUnless(GAME.is_file(), "native build required")
class UnknownConsumablesTest(unittest.TestCase):
    def replay_messages(self, healing_restock):
        with tempfile.TemporaryDirectory(prefix="unknown-consumables-") as directory:
            output = Path(directory) / "run"
            env = {
                name: value for name, value in os.environ.items()
                if not name.startswith(("BORG_", "BORGH_", "ANGBAND_"))
                or name in ("BORG_CEMETERY_DIR", "BORG_DEATH_RECORDS_DIR")
            }
            # On the prior core these overrides took the safe tasting branch
            # with the Light Purple Potion at full HP and zero danger.
            env.update(
                BORG_CORE="borg-rs",
                # The historical confusion encounter uses the original route.
                BORG_PARAM_RESTOCK_HEALING=healing_restock,
                # Match the route to the confusion encounter used by this replay.
                BORG_PARAM_CAUTION_RISING_SKIP_HELD_STAIRS="0",
                BORG_PARAM_ATTACK_THROW_SURPLUS="0",
                # Keep the route to the historical identification and sale.
                BORG_PARAM_ITEM_USE_IDLE="0",
                BORG_PARAM_EXPLORE_MIX_DEPTH_ADJUSTED_LEVEL="0",
                BORG_PARAM_PREPARED_DL2_MIN_HP="30",
                BORG_PARAM_PREPARED_DL2_MIN_CLEVEL__WARRIOR="0",
                BORG_PARAM_PREPARED_DL34_MIN_HP__WARRIOR="50",
                BORG_PARAM_PREPARED_DL34_MIN_CLEVEL__WARRIOR="4",
                BORG_PARAM_PREPARED_DL59_MIN_HP__WARRIOR="60",
                BORG_PARAM_PREPARED_DL59_MIN_CLEVEL__WARRIOR="6",
                BORG_PARAM_ITEM_TEST_UNKNOWN_POTIONS="1",
                BORG_PARAM_ITEM_TEST_UNKNOWN_MIN_HP="1",
                BORG_PARAM_ITEM_TEST_UNKNOWN_MIN_HP_PERCENT="1",
            )
            result = subprocess.run(
                [str(ROOT / "harness/run.sh"), str(output), "101", "0", "0", "30000", "menu"],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            messages = (output / "messages.log").read_text()

        self.assertNotIn("# Testing unknown potion", messages)
        self.assertNotIn("# Quaffing Light Purple Potion", messages)
        self.assertNotIn("# Quaffing Orange Speckled Potion", messages)
        self.assertNotRegex(messages, r'# Reading Scrolls? titled "co pro eleas"')
        return messages

    def assert_events_in_order(self, messages, events):
        offset = 0
        for event in events:
            position = messages.find(event, offset)
            self.assertGreaterEqual(position, offset, event)
            offset = position + len(event)

    def test_unknown_potions_and_scrolls_reach_town_and_known_items_are_usable(self):
        messages = self.replay_messages("0")
        self.assert_events_in_order(messages, [
            "You have a Light Purple Potion",
            "You have 4 Orange Speckled Potions",
            "# Selling Light Purple Potion",
            "# Selling Orange Speckled Potions",
            "You had a Potion of Cure Light Wounds",
            '# Selling Scrolls titled "co pro eleas"',
            "You had a Scroll of Blessing",
            "# Reading Scroll of Blessing.",
            "# Quaffing Potion of Cure Serious Wounds.",
            "You are no longer confused.",
        ])

    def test_healing_restock_preserves_the_planned_unknown_potion_sale(self):
        messages = self.replay_messages("1")
        self.assert_events_in_order(messages, [
            "You have 4 Orange Speckled Potions",
            "# Returning to town: low on healing potions",
            "# Selling Orange Speckled Potions",
            "You had a Potion of Cure Light Wounds",
        ])


if __name__ == "__main__":
    unittest.main()

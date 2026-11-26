"""Exercise Scroll of Light policy through Angband's native commands."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


from pborg_fixture import write_pborg_config

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "build/rust/game"
SCROLL_LIGHT = "equip:scroll:Light:6:6:none"


@unittest.skipUnless((GAME / "angband").is_file(), "native build required")
class LightScrollsTest(unittest.TestCase):
    def play(self, class_id=0, *, enabled=1, monster=None):
        with tempfile.TemporaryDirectory(prefix="light-scrolls-") as directory:
            out = Path(directory)
            lib = out / "lib"
            gamedata = lib / "gamedata"
            gamedata.mkdir(parents=True)
            for entry in (GAME / "lib").iterdir():
                if entry.name != "gamedata":
                    (lib / entry.name).symlink_to(entry)
            for entry in (GAME / "lib/gamedata").iterdir():
                if entry.name != "class.txt":
                    (gamedata / entry.name).symlink_to(entry)
            # Birth equipment is known. The synthetic-build importer accepts
            # only Human Warriors, so give each class its kit via class data.
            name = "Necromancer" if class_id == 4 else "Warrior"
            classes = (GAME / "lib/gamedata/class.txt").read_text()
            marker = f"name:{name}\n"
            self.assertEqual(classes.count(marker), 1)
            (gamedata / "class.txt").write_text(classes.replace(marker, f"{marker}{SCROLL_LIGHT}\n"))

            paths = {name: out / name for name in ("user", "save", "archive", "panic", "scores")}
            for path in paths.values():
                path.mkdir()
            scary = "scary_monsters.txt"
            (paths["user"] / scary).write_bytes((ROOT / "harness" / scary).read_bytes())
            config = (ROOT / "harness/borg.txt").read_text().splitlines()
            setting = "borg_param_light_scroll_idle_spend"
            config = [line for line in config if not line.startswith(f"{setting} =")]
            write_pborg_config(paths["user"], "\n".join([*config, f"{setting} = {enabled}", ""]))

            env = {k: v for k, v in os.environ.items() if not k.startswith(("BORG_", "BORGH_"))}
            env.update(BORG_CORE="borg-rs", BORG_RUN_DIR=str(out))
            messages = out / "messages.log"
            states = out / "state.log"
            args = [
                str(GAME / "angband"), "-mborgh", "-n",
                *(f"-d{key}={value}" for key, value in paths.items()),
                "--", "-seed", "101", "-race", "0", "-class", str(class_id), "-menu-letters",
                "-keys", "120", "-msglog", str(messages), "-statelog", str(states),
                "-start-depth", "1",
            ]
            if monster:
                args += ["-start-monsters", monster]
            result = subprocess.run(args, cwd=out, env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            trace = [json.loads(line) for line in (out / "trace-core.jsonl").read_text().splitlines()]
            positions = dict(line.split()[:2] for line in states.read_text().splitlines())
            return trace, messages.read_text(), positions

    def test_necromancer_drops_the_known_stack_in_one_action(self):
        trace, messages, _ = self.play(4)
        self.assertEqual(trace[0]["check"], "borg_discard_light_scrolls")
        self.assertIn("You drop 6 Scrolls of Light {ignore}", messages)
        self.assertIn("You have no more Scrolls of Light", messages)
        self.assertNotIn("You are surrounded by a white light.", messages)

    def test_warrior_spends_the_stack_while_moving_between_dark_squares(self):
        trace, messages, positions = self.play()
        reads = [row for row in trace if row["check"] == "borg_check_light" and ord("r") in row["keys"]]
        self.assertEqual(len(reads), 6, messages)
        self.assertEqual(len({positions[str(row["turn"])] for row in reads}), 6, messages)
        self.assertEqual(messages.count("You are surrounded by a white light."), 6)

    def test_urgent_combat_takes_priority_over_a_light_scroll(self):
        trace, messages, _ = self.play(monster="white_worm_mass:1:1")
        before_kill, killed, _ = messages.partition("You have destroyed the white worm mass")
        self.assertTrue(killed, messages)
        self.assertNotIn("# Reading Scrolls of Light", before_kill)
        self.assertIn(ord("q"), trace[0]["keys"])

    def test_zero_disables_opportunistic_reading(self):
        _, messages, _ = self.play(enabled=0)
        self.assertNotIn("# Illuminating the dungeon with a scroll", messages)


if __name__ == "__main__":
    unittest.main()

"""Replay #1294 to 90,000 turns, past the old stall at turn 80,484.

The harness allows 90 wall-clock seconds for the native game on shared CI.
"""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
BIN = ROOT / "build/rust/game/angband"


@unittest.skipUnless(BIN.is_file(), "no build (harness/build.sh)")
class BlackguardRecoveryTest(unittest.TestCase):
    def test_blackguard_gets_past_the_mana_rest_stall(self):
        with tempfile.TemporaryDirectory(prefix="blackguard-rest-") as directory:
            out = Path(directory) / "run"
            env = {
                key: value for key, value in os.environ.items()
                if not key.startswith(("BORG_", "BORGH_")) and key != "ANGBAND_BIN"
            }
            if "BORG_CEMETERY_DIR" in os.environ:
                env["BORG_CEMETERY_DIR"] = os.environ["BORG_CEMETERY_DIR"]
            env.update(BORG_IMPL="rust", BORG_CORE="borg-rs", BORG_CORE_TRACE="1",
                       BORG_DEATH_RECORDS_DIR=str(out.parent / "deaths"),
                       RUN_TIMEOUT_SECONDS="90", STALL_TIMEOUT_SECONDS="0")
            result = subprocess.run(
                [str(ROOT / "harness/run.sh"), str(out), "312081895", "7", "8", "90000", "menu,exact"],
                env=env, capture_output=True, text=True, timeout=105,
            )
            if result.returncode != 0:
                receipt = out / "process.json"
                status = json.loads(receipt.read_text())["status"] if receipt.is_file() else "no process receipt"
                self.fail(f"Blackguard replay failed ({status}, exit {result.returncode})\n{result.stderr}")
            last_turn = health_rests = 0
            with (out / "trace-core.jsonl").open() as trace:
                for line in trace:
                    decision = json.loads(line)
                    last_turn = decision["turn"]
                    if decision["check"] == "borg_recover":
                        keys = decision["keys"][:2]
                        self.assertNotEqual(keys, [ord("R"), ord("*")], f"mana rest at {last_turn}")
                        health_rests += keys == [ord("R"), ord("&")]
            self.assertGreater(health_rests, 0, "the seed exercises ordinary recovery")
            self.assertGreater(last_turn, 80484, "the Borg keeps deciding past the old stall")

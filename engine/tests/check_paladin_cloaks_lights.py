"""Replay the private Paladin cloak and light checkpoint from #1105."""

import argparse
import re
import tempfile
from pathlib import Path

from check_paladin_daggers import replay


def check(binary, fixture, run):
    log = replay(binary, fixture, run, 1,
                 "b7a10359247bc0148bc81b3e1b7f63bdb76a0b60a003f488f06363147ca40d2a",
                 (1984589, 1987000))
    disposal = r"You (?:drop|sold|had) [^\n]*"
    for spare in ("Wooden Torch of Brightness", "Lantern of Brightness", "Cloak of Stealth [1,+2]", "Cloak [1,+0]"):
        assert re.search(disposal + re.escape(spare), log), spare
    for keeper in ("Phial of Galadriel", "Lantern (Everburning)", "Cloak of Thorongil", "Cloak of Stealth [1,+4]"):
        assert not re.search(disposal + re.escape(keeper), log), keeper
    print("Paladin cleared the fueled spares and weaker cloaks, keeping the permanent pair and stronger stealth cloak")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted evidence bundle")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-paladin-cloaks-lights-") as directory:
        check(args.binary.resolve(), args.fixture.resolve(), Path(directory))

"""Replay the private A2 checkpoint for #1272 under weight and slot pressure."""

import argparse
from pathlib import Path
import re
import tempfile

from check_weapon_hoard import replay


def check(binary, fixture):
    configurations = [
        ("saved profile", {}, (58, 10, 45, 10)),
        ("pack-slot pressure", {"BORG_PARAM_TOWN_DEPARTURE_FREE_SLOTS": "12"}, (59, 6, 34, 7)),
    ]
    pattern = r"# Quiver (before|after) trim: (\d+) usable shots, (\d+) free pack slots"
    for label, settings, expected in configurations:
        with tempfile.TemporaryDirectory(prefix="a2-quiver-trim-") as directory:
            _, messages = replay(binary, fixture, Path(directory), "A2", settings)
        entries = re.findall(pattern, messages)
        assert len(entries) >= 2, (label, entries)
        before, after = entries[:2]
        assert before[0] == "before" and after[0] == "after", (label, entries)
        actual = tuple(map(int, before[1:] + after[1:]))
        assert actual == expected, (label, actual, expected)
        assert all(int(shots) >= 30 for phase, shots, _ in entries if phase == "after"), (label, entries)
        assert not re.search(r"# Town headroom dropping .* of Slay Animal", messages), label
        print(f"A2 {label}: {actual[0]} to {actual[2]} usable shots, {actual[1]} to {actual[3]} free pack slots")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted A2 fixture from #1269")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    check(args.binary.resolve(), args.fixture.resolve())

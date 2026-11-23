#!/usr/bin/env python3
"""Native first-town regression for Half-Orc Ranger 2095979189 (#797)."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
COMMON = [
    "4 Rations of Food for 12 gold.",
    "3 Crimson Potions for 60 gold.",
]
EXPECTED_PURCHASES = {
    0: COMMON + [
        "20 Arrows (1d4) (+0,+0) for 20 gold.",
        "26 Arrows (1d4) (+0,+0) for 26 gold.",
        'a Scroll titled "inget fautium" for 15 gold.',
        "12 Arrows (1d4) (+0,+0) for 12 gold.",
    ],
    2: COMMON + [
        '2 Scrolls titled "inget fautium" for 30 gold.',
        "20 Arrows (1d4) (+0,+0) for 20 gold.",
        "11 Arrows (1d4) (+0,+0) for 11 gold.",
    ],
}


def check_arm(out, reserve):
    env = {name: value for name, value in os.environ.items()
           if not name.startswith(("BORG_", "BORGH_", "ANGBAND_"))
           or name in ("BORG_CEMETERY_DIR", "BORG_DEATH_RECORDS_DIR")}
    # Keep the original fuel policy for the exact Phase Door purchase trace,
    # and #987's first-cure floor off (the Ranger ships 2; with it the cures
    # are bought first and the trace's order changes).
    env.update(BORG_CORE="borg-rs", BORG_IMPL="rust",
               BORG_TXT_SET=f"borg_param_town_phase_reserve__ranger={reserve};"
                            "borg_param_town_phase_reserve_percent=100;"
                            "borg_param_town_light_reserve_turns=0;"
                            "borg_param_town_first_cure__ranger=0")
    subprocess.run([str(ROOT / "harness/run.sh"), str(out), "2095979189",
                    "6", "7", "1900", "exact,menu"], cwd=ROOT, env=env,
                   check=True, timeout=90)
    summary = json.loads((out / "summary.json").read_text())
    assert summary["stop_reason"] == "turns" and summary["game_turns"] == 1900, summary
    assert (summary["seed"], summary["class_idx"], summary["race_idx"]) == (2095979189, 7, 6)
    departure = None
    with (out / "state.log").open() as states:
        for fields in map(str.split, states):
            if len(fields) >= 3 and fields[2].startswith("d") and not fields[2].startswith("d0/"):
                departure = int(fields[0])
                break
    assert departure is not None, (reserve, "no dungeon departure")

    messages = [(int(turn), note) for _, turn, _, note in
                (line.split(" ", 3) for line in (out / "messages.log").read_text().splitlines())]
    purchases = [note.removeprefix("You bought ") for turn, note in messages
                 if turn < departure and note.startswith("You bought ")]
    assert purchases == EXPECTED_PURCHASES[reserve], (reserve, departure, purchases)

    rows = [json.loads(row) for row in (out / "phase-observations.jsonl").read_text().splitlines()]
    assert all(row["schema"] == "borg-phase-observations@1" for row in rows)
    alchemist_stock = [row["phase_scrolls"] for row in rows
                       if row["type"] == "shop_visit" and row["shop"] == 4
                       and row["game_turn"] < departure]
    assert alchemist_stock == [40, 40], (reserve, alchemist_stock)
    dungeon_contact = [row for row in rows if row["type"] == "first_monster_contact"
                       and row["depth"] > 0]
    assert len(dungeon_contact) == 1, (reserve, dungeon_contact)
    contact = dungeon_contact[0]
    first_sighting = next((turn for turn, note in messages
                           if turn >= departure and note.startswith("# Creating a monster ")), None)
    assert first_sighting is not None, (reserve, "no dungeon monster sighting")
    assert contact["game_turn"] == first_sighting, (reserve, contact, first_sighting)
    # Each character meets its first foe with the scrolls it bought: 1, or the
    # reserve's 2. With C's light radius the reserve-2 character picked up four
    # more first (6); with the engine's (#1078, shipped) it sees a foe at turn
    # 473, before those pickups.
    assert contact["phase_scrolls"] == (1 if reserve == 0 else 2), (reserve, contact)
    print(f"Ranger reserve {reserve}: first town purchases and contact kit matched")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, help="new directory for replay logs on failure")
    args = parser.parse_args()
    out = args.out
    if out is None:
        out = Path(tempfile.mkdtemp(prefix="ranger-phase-"))
    else:
        out.mkdir()
    try:
        for reserve in (0, 2):
            check_arm(out / f"reserve-{reserve}", reserve)
    except BaseException:
        print(f"Ranger regression logs retained in {out.resolve()}", file=sys.stderr)
        raise
    else:
        shutil.rmtree(out)


if __name__ == "__main__":
    main()

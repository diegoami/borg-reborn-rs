#!/usr/bin/env python3
"""Native first-town regression for Half-Orc Ranger 2095979189 (#797)."""

import json
import os
from pathlib import Path
import subprocess
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
    env.update(BORG_CORE="borg-rs", BORG_IMPL="rust",
               BORG_TXT_SET=f"borg_param_town_phase_reserve__ranger={reserve};"
                            "borg_param_town_phase_reserve_percent=100")
    subprocess.run([str(ROOT / "harness/run.sh"), str(out), "2095979189",
                    "6", "7", "1900", "exact,menu"], cwd=ROOT, env=env,
                   check=True, timeout=90)
    summary = json.loads((out / "summary.json").read_text())
    assert summary["stop_reason"] == "turns" and summary["game_turns"] == 1900, summary
    assert (summary["seed"], summary["class_idx"], summary["race_idx"]) == (2095979189, 7, 6)
    with (out / "state.log").open() as states:
        departure = next(int(turn) for turn, _, depth, *_ in map(str.split, states)
                         if not depth.startswith("d0/"))
    purchases = []
    for row in (out / "messages.log").read_text().splitlines():
        _, turn, _, message = row.split(" ", 3)
        if int(turn) < departure and message.startswith("You bought "):
            purchases.append(message.removeprefix("You bought "))
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
    assert departure <= contact["game_turn"], (reserve, contact)
    # The reserve-2 character picked up four more scrolls before seeing a foe.
    assert contact["phase_scrolls"] == (1 if reserve == 0 else 6), (reserve, contact)
    print(f"Ranger reserve {reserve}: first town purchases and contact kit matched")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory(prefix="ranger-phase-") as tmp:
        for reserve in (0, 2):
            check_arm(Path(tmp) / f"reserve-{reserve}", reserve)

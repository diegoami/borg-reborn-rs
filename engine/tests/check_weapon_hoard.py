"""Reload the checksum-pinned private fixtures linked in issue #1269."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


SAVES = {
    "A1": ("b3023d3e11aac0f5548cb48d0ae4ee6e490c40aecbd260b4c81d03643021e4b4", (0, 0), 1905934, 1955934),
    "A2": ("cd19e65fde08faf726c18d6f00e3b42f61ef779e21a3772bf8ad2c3e9ec4e5cb", (1, 1), 4087306, 4177306),
    "A3": ("ddefc96911524bf7284b06fc17d33ec5392f1d17c67ab279c55d1acb1bbd2031", (3, 7), 1617717, 1647717),
}
SETTINGS_HASHES = {
    "borg.txt": "7b4a889d2aea8881be958950bb2881756cd160e640079502d8ef6359694a0096",
    "scary_monsters.txt": "f35bfc49af22ee19a1ea0270d2ed471595bf3104813fc69891312d0fccece75c",
}
DROPPED = (
    "Dagger of Slay Dragon (1d4) (+3,+3)",
    "Whip of Slay Dragon (1d3) (+2,+5)",
    "Whip of Slay Troll (1d3) (+5,+1)",
    "Main Gauche of Frost (1d5) (+4,+6)",
)
KEPT = (
    "Dagger of Slay Dragon (1d6) (+4,+1)",
    "Spear of *Slay Troll* (1d6) (+1,+1) <+2>",
    "Sling of Power (x2) (+12,+15)",
)
ELEMENTAL_DAGGERS = tuple(f"Dagger '{name}' (1d4) (+9,+12) [+10]" for name in ("Dethanc", "Nimthanc", "Narthanc"))
TULWAR = "Tulwar of *Slay Dragon* (2d4) (+2,+4) <+2>"
SHOVEL = "Shovel of Digging (1d2) (+4,+3) <+4>"


def replay(binary, fixture, run, character, overrides=None):
    save = f"{character}.save"
    digest, (race, player_class), loaded_turn, stop_turn = SAVES[character]
    hashes = {save: digest, **SETTINGS_HASHES}
    for name, expected in hashes.items():
        digest = hashlib.sha256((fixture / name).read_bytes()).hexdigest()
        assert digest == expected, f"{name} checksum mismatch: {digest}"

    paths = {name: run / name for name in ("user", "save", "archive", "panic", "scores")}
    for path in paths.values():
        path.mkdir()
    for name in hashes:
        shutil.copyfile(fixture / name, paths["save" if name == save else "user"] / name)
    town = run / "town.jsonl"
    messages = run / "messages.log"
    summary = run / "summary.json"
    env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
    env.update(BORG_TOWN_DUMP=str(town), BORG_CORE="borg-rs", TZ="UTC", LC_ALL="C.UTF-8")
    # These numeric checkpoints predate the cross-location gear cutoff.
    env["BORG_PARAM_JUNK_GEAR_ALTERNATIVES"] = "0"
    if overrides:
        env.update(overrides)
    subprocess.run(
        [str(binary), "-mborgh", *(f"-d{name}={path}" for name, path in paths.items()),
         "--", "-seed", "1", "-race", str(race), "-class", str(player_class), "-load", str(paths["save"] / save),
         "-keep-load-rng", "-exact-turns", "-turns", str(stop_turn), "-msglog", str(messages),
         "-summary", str(summary)],
        cwd=binary.parent, env=env, check=True, timeout=120,
    )
    receipt = json.loads(summary.read_text())
    assert receipt["loaded_turn"] == loaded_turn and receipt["stop_reason"] == "turns", receipt
    decisions = [row for line in town.read_text().splitlines() if (row := json.loads(line))["type"] == "decision"]
    return decisions, messages.read_text()


def check_warrior(decisions, log):
    first = {item["desc"] for item in decisions[0]["pack"]}
    initial_home = {item["desc"] for item in decisions[0]["home"]}
    assert all(f"You drop a {name} {{borg ignore}}" in log for name in DROPPED)
    assert first.isdisjoint(DROPPED) and set(KEPT + (TULWAR, SHOVEL,)).issubset(first), first
    assert {"Rod of Light", "Wand of Magic Missile (8 charges)", "Staff of Mapping (5 charges)"} <= first
    assert len(initial_home) == 24, initial_home
    last = {item["desc"] for item in decisions[-1]["pack"]}
    last_home = {item["desc"] for item in decisions[-1]["home"]}
    assert set(KEPT + (SHOVEL,)).issubset(last | last_home), (last, last_home)
    assert TULWAR in last_home and TULWAR not in last, (last, last_home)
    assert {"Main Gauche of Lightning (1d5) (+4,+3)", "Rapier of Slay Orc (1d6) (+2,+4)",
            "Tulwar of Flame (2d4) (+6,+8)"}.isdisjoint(last_home), last_home
    assert (last | last_home).intersection(ELEMENTAL_DAGGERS) == {ELEMENTAL_DAGGERS[0]}, (last, last_home)
    print("A1 dropped four redundant carried weapons, brought three devices to town, and cleared three duplicate Home weapons.")


def check_mage(decisions, _log):
    first = {item["desc"] for item in decisions[0]["pack"]}
    assert set(ELEMENTAL_DAGGERS[:2]) <= first and len(decisions[0]["home"]) == 24, decisions[0]
    unwanted = {
        "Studded Leather Armour (-1) [12,+2]", "Studded Leather Armour of Resist Fire (-1) [12,+1]",
        "Pair of Iron Shod Boots of Slow Descent [4,+3]", "Pair of Iron Shod Boots [4,+2]",
        "Pair of Iron Shod Boots [4,-1]", "Sling of Accuracy (x2) (+15,+8)",
    }
    last = decisions[-1]
    pack = {item["desc"] for item in last["pack"]}
    home = {item["desc"] for item in last["home"]}
    worn = {item["desc"] for item in last["equipment"]}
    assert home.intersection(ELEMENTAL_DAGGERS) == {ELEMENTAL_DAGGERS[2]}, home
    assert pack.isdisjoint(ELEMENTAL_DAGGERS) and (pack | home).isdisjoint(unwanted), (pack, home)
    assert len(home) <= 18 and "Pair of Iron Shod Boots of Slow Descent [4,+4]" in home | worn, (worn, home)
    assert "Potion of Healing" in home | pack, (home, pack)
    print(f"A2 reduced Home from 24 to {len(home)} stacks and kept the Healing potion and stronger boots available.")


def check_ranger(decisions, log):
    first = {item["desc"] for item in decisions[0]["pack"]}
    accuracy = {"Sling of Accuracy (x2) (+17,+6)", "Sling of Accuracy (x2) (+15,+8)"}
    assert {name for name in first if name.startswith("Sling")} == accuracy, first
    assert log.count("# Dropping inferior spare Sling (x2)") >= 8
    last = decisions[-1]
    kit = {item["desc"] for place in ("pack", "home", "equipment") for item in last[place]}
    assert {name for name in kit if name.startswith("Sling")} == accuracy, kit
    print("A3 dropped eight inferior slings and cleared the weaker Home sling. The two accuracy/damage options survived.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("character", choices=SAVES)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted #1269 fixture")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-weapon-hoard-") as directory:
        run = replay(args.binary.resolve(), args.fixture.resolve(), Path(directory), args.character)
        {"A1": check_warrior, "A2": check_mage, "A3": check_ranger}[args.character](*run)

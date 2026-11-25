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
    "A1": ("b3023d3e11aac0f5548cb48d0ae4ee6e490c40aecbd260b4c81d03643021e4b4", 0, 1905934, 1955934),
    "A2": ("cd19e65fde08faf726c18d6f00e3b42f61ef779e21a3772bf8ad2c3e9ec4e5cb", 1, 4087306, 4177306),
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
    "Dagger 'Narthanc' (1d4) (+9,+12) [+10]",
    "Dagger 'Nimthanc' (1d4) (+9,+12) [+10]",
    "Sling of Power (x2) (+12,+15)",
)
TULWAR = "Tulwar of *Slay Dragon* (2d4) (+2,+4) <+2>"
SHOVEL = "Shovel of Digging (1d2) (+4,+3) <+4>"


def replay(binary, fixture, run, character):
    save = f"{character}.save"
    digest, race_class, loaded_turn, stop_turn = SAVES[character]
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
    subprocess.run(
        [str(binary), "-mborgh", *(f"-d{name}={path}" for name, path in paths.items()),
         "--", "-seed", "1", "-race", str(race_class), "-class", str(race_class), "-load", str(paths["save"] / save),
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
    assert "Dagger 'Dethanc' (1d4) (+9,+12) [+10]" in last_home, last_home
    print("A1 dropped four redundant carried weapons, brought three devices to town, and cleared three duplicate Home weapons.")


def check_mage(decisions, _log):
    daggers = {"Dagger 'Dethanc' (1d4) (+9,+12) [+10]", "Dagger 'Nimthanc' (1d4) (+9,+12) [+10]"}
    first = {item["desc"] for item in decisions[0]["pack"]}
    assert daggers <= first and len(decisions[0]["home"]) == 24, decisions[0]
    for decision in decisions[1:]:
        pack = {item["desc"] for item in decision["pack"]}
        home = {item["desc"] for item in decision["home"]}
        if home >= daggers and pack.isdisjoint(daggers):
            assert "Dagger 'Narthanc' (1d4) (+9,+12) [+10]" in home, home
            print("A2 stored Dethanc and Nimthanc independently at Home alongside Narthanc.")
            return
    raise AssertionError("A2 did not store both carried daggers at Home")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("character", choices=SAVES)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted #1269 fixture")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-weapon-hoard-") as directory:
        run = replay(args.binary.resolve(), args.fixture.resolve(), Path(directory), args.character)
        (check_warrior if args.character == "A1" else check_mage)(*run)

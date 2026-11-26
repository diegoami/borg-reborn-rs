"""Replay the P-Borg checkpoint from #1304 with its recorded YAML."""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


START_TURN = 2696014
STOP_TURN = 2705000
HASHES = {
    "normal.save": "6bc69acc1b262d16d57c4cf20a2dbb6a721609242dbb1be6f2d2dbe86857cf66",
    "pborg.yaml": "1cecdf84d3f246bf94fd83d85399a9a7b2d78d57eaa400a045d5fb2ce2569081",
}
WEAPONS = {
    "Short Sword": "+4,+1",
    "Trident": "+2,+5",
    "Dagger": "+6,+2",
    "Morning Star": "+4,+3",
}


def check(binary, fixture, run):
    paths = {kind: run / kind for kind in ("user", "save", "archive", "panic", "scores")}
    for path in paths.values():
        path.mkdir()
    for name, digest in HASHES.items():
        source = fixture / name
        assert hashlib.sha256(source.read_bytes()).hexdigest() == digest, name
        shutil.copyfile(source, paths["save" if name == "normal.save" else "user"] / name)
    harness = Path(__file__).resolve().parents[2] / "harness"
    shutil.copyfile(harness / "scary_monsters.txt", paths["user"] / "scary_monsters.txt")
    messages = run / "messages.log"
    summary = run / "summary.json"
    env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
    env.update(TZ="UTC", LC_ALL="C.UTF-8")
    subprocess.run(
        [str(binary), "-mborgh", *(f"-d{kind}={path}" for kind, path in paths.items()),
         "--", "-seed", "1", "-race", "6", "-class", "6", "-load", str(paths["save"] / "normal.save"),
         "-keep-load-rng", "-exact-turns", "-turns", str(STOP_TURN), "-msglog", str(messages),
         "-summary", str(summary)],
        cwd=binary.parent, env=env, check=True, timeout=120,
    )
    receipt = json.loads(summary.read_text())
    assert receipt["loaded_turn"] == START_TURN and receipt["stop_reason"] == "turns", receipt
    log = messages.read_text(errors="replace")
    assert "Initializing PBorg... (pborg.yaml)" in log
    sandals = re.findall(r"You drop (\d+) Pairs of Leather Sandals \[1,\+0\] \{borg ignore(?: surplus)?\}", log)
    assert sum(map(int, sandals)) in (6, 7), sandals
    assert not re.search(r"You have (?:[2-9]|\d{2,}) Pairs of Leather Sandals \[1,\+0\]", log)
    for weapon, bonus in WEAPONS.items():
        description = rf"{weapon}s? [^\n]*\({re.escape(bonus)}\)"
        assert re.search(rf"You have no more {description}", log), weapon
        assert not re.search(rf"You have (?:a|an|\d+) {description}", log), weapon
    # Home deposits are reported as drops by the native store command.
    outside_home_deposits = re.sub(
        r"# Town plan executing action 5 at shop 7[^\n]*\n[^\n]*# Selling [^\n]*\n[^\n]*You drop [^\n]*",
        "", log,
    )
    assert not re.search(
        r"You (?:drop|sold) [^\n]*(?:Narthanc|Forasgil|Colannon|Shovel(?: of Digging| \(1d2\) \(\+5,\+5\)))",
        outside_home_deposits,
    )
    print("Rogue checkpoint: sandal stack trimmed, four weapon slots freed, distinct gear kept")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted evidence bundle")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-rogue-spare-gear-") as directory:
        check(args.binary.resolve(), args.fixture.resolve(), Path(directory))

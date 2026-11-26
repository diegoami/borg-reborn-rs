"""Replay the private Paladin save from #1279 with both gear policies."""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


SAVE = "Borg261126031916.save"
HASHES = {
    SAVE: "cfa380896fafbe7bc8f9f47a8716a0ec16014028623c7084f9915447af3d87b8",
    "borg-settings/borg.txt": "f6350e6c3493415dca3f0613a6886520e93d2e48967c0318c475b3cce46fed8d",
    "borg-settings/errands_plan.txt": "084bc3e8999697bd8edf7fd302ea75d65b50ef2714afdd3a47354a1f24137e87",
    "borg-settings/scary_monsters.txt": "5a4ec4b6221f2487d19bd3146c9c36ed7e3a619b1bf9d14dbd9a8f14ba187690",
}


def check(binary, fixture, run, home_backups):
    paths = {kind: run / kind for kind in ("user", "save", "archive", "panic", "scores")}
    for path in paths.values():
        path.mkdir(parents=True)
    for name, digest in HASHES.items():
        source = fixture / name
        assert hashlib.sha256(source.read_bytes()).hexdigest() == digest, name
        shutil.copyfile(source, paths["save" if name == SAVE else "user"] / source.name)
    messages = run / "messages.log"
    summary = run / "summary.json"
    env = {key: value for key, value in os.environ.items() if not key.startswith(("BORG_", "BORGH_"))}
    env.update(BORG_CORE="borg-rs", BORG_PARAM_TOWN_GEAR_HOME_BACKUPS=str(home_backups), TZ="UTC", LC_ALL="C.UTF-8")
    subprocess.run(
        [str(binary), "-mborgh", *(f"-d{kind}={path}" for kind, path in paths.items()),
         "--", "-seed", "1", "-race", "0", "-class", "5", "-load", str(paths["save"] / SAVE),
         "-keep-load-rng", "-exact-turns", "-turns", "1205000", "-msglog", str(messages),
         "-summary", str(summary)],
        cwd=binary.parent, env=env, check=True, timeout=120,
    )
    receipt = json.loads(summary.read_text())
    assert receipt["loaded_turn"] == 1196638 and receipt["stop_reason"] == "turns", receipt
    log = messages.read_text()
    for bonus in ("+6,+2", "+3,+3"):
        assert re.search(rf"You (?:drop|sold|had) [^\n]*Dagger[^\n]*\({re.escape(bonus)}\)", log), (home_backups, bonus)
    assert not re.search(r"You (?:drop|sold|had) [^\n]*(?:Narthanc|Nimthanc)", log), home_backups
    print(f"Paladin with Home backups {home_backups}: both ordinary daggers cleared, both named daggers kept")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted evidence bundle")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-paladin-daggers-") as directory:
        for home_backups in (0, 1):
            check(args.binary.resolve(), args.fixture.resolve(), Path(directory) / str(home_backups), home_backups)

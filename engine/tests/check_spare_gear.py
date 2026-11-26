"""Replay the private Blackguard checkpoint from #1276 at both cutoffs."""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


SAVE = "Borg261126010617.save"
HASHES = {
    SAVE: "2313b828bd236e18c9dcf9140282264cc5151df539f0107a3fa7c7782804af92",
    "borg-settings/borg.txt": "f186f886208b4d77e29c1ebe97dac6eb5e1af34589fbc0d43c7b2c5f89bc3225",
    "borg-settings/errands_plan.txt": "084bc3e8999697bd8edf7fd302ea75d65b50ef2714afdd3a47354a1f24137e87",
    "borg-settings/scary_monsters.txt": "5a4ec4b6221f2487d19bd3146c9c36ed7e3a619b1bf9d14dbd9a8f14ba187690",
}
STACKS = {
    "robes": (3, "Robes [2,+0]"),
    "daggers": (5, "Daggers (1d4) (+0,+0)"),
    "spears": (2, "Spears (1d6) (+0,+0)"),
}


def replay(binary, fixture, run, cutoff):
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
    env.update(BORG_CORE="borg-rs", BORG_PARAM_JUNK_GEAR_ALTERNATIVES=str(cutoff), TZ="UTC", LC_ALL="C.UTF-8")
    subprocess.run(
        [str(binary), "-mborgh", *(f"-d{kind}={path}" for kind, path in paths.items()),
         "--", "-seed", "1", "-race", "6", "-class", "8", "-load", str(paths["save"] / SAVE),
         "-keep-load-rng", "-exact-turns", "-turns", "1022000", "-msglog", str(messages),
         "-summary", str(summary)],
        cwd=binary.parent, env=env, check=True, timeout=120,
    )
    receipt = json.loads(summary.read_text())
    assert receipt["loaded_turn"] == 1020544 and receipt["stop_reason"] == "turns", receipt
    log = messages.read_text()
    return {name for name, (count, item) in STACKS.items()
            if re.search(rf"You drop {count} {re.escape(item)} \{{borg ignore(?: surplus)?\}}", log)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path, help="directory containing the extracted evidence bundle")
    parser.add_argument("--binary", type=Path, default=Path("build/rust/game/angband"))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="borg-spare-gear-") as directory:
        for cutoff in (0, 2, 3):
            cleared = replay(args.binary.resolve(), args.fixture.resolve(), Path(directory) / str(cutoff), cutoff)
            if cutoff == 0:
                assert not cleared, cleared
            if cutoff == 2:
                assert cleared == set(STACKS), cleared
            if cutoff == 3:
                assert cleared == {"daggers"}, cleared
            print(f"Blackguard cutoff {cutoff} cleared: {', '.join(sorted(cleared)) or 'none'}")

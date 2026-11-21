#!/usr/bin/env python3
"""The Rust crates linked into the game (#819), with their licences.

Usage: scripts/release/rust-licenses.py ROOT

Walks `cargo metadata` (Cargo.lock, default features) from the borg-reborn-rs
package along normal dependencies only: build and dev dependencies are not
linked into the staticlib. Prints one line per crate, sorted.
"""

import json
import subprocess
import sys


def main() -> int:
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=root, check=True, capture_output=True, text=True,
        ).stdout
    )
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    start = [i for i, p in packages.items() if p["name"] == "borg-reborn-rs"]
    if len(start) != 1:
        print("rust-licenses.py: no single borg-reborn-rs package", file=sys.stderr)
        return 1
    seen, todo = set(), list(start)
    while todo:
        pid = todo.pop()
        if pid in seen:
            continue
        seen.add(pid)
        for dep in nodes[pid]["deps"]:
            if any(k["kind"] is None for k in dep["dep_kinds"]):
                todo.append(dep["pkg"])
    print("# Rust crates angband and angband-headless are built from (cargo metadata, normal dependencies; proc-macros run at build time only)")
    print("# name version licence source")
    missing = 0
    for pid in sorted(seen, key=lambda i: (packages[i]["name"], packages[i]["version"])):
        p = packages[pid]
        lic = p.get("license") or (f"see {p['license_file']}" if p.get("license_file") else None)
        if lic is None:
            missing += 1
            lic = "UNKNOWN"
        src = "this repository" if p.get("source") is None else (p.get("repository") or p["source"])
        print(f"{p['name']} {p['version']} {lic} {src}")
    if missing:
        print(f"rust-licenses.py: {missing} crate(s) without a licence", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

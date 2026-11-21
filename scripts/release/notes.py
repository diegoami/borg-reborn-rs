#!/usr/bin/env python3
"""Draft release notes (#819).

Usage: scripts/release/notes.py --version V [--milestone NAME] [--since REF]
                                [--zoo URL] [--build-id FILE ...]

Markdown on stdout: what the release is (the commit, upstream Angband's, the
borg-rs core's pin), the PLAN.md lines naming the milestone, the handover
notes added or changed since REF (by default the newest earlier v* tag; none: every
note), the zoo's measurement of the commit when given, and the build
identities. A draft for the owner to edit before publishing.
"""

import argparse
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def git(*args: str) -> str:
    return subprocess.run(["git", "-C", str(ROOT), *args], check=True,
                          capture_output=True, text=True).stdout.strip()


def previous_tag(version: str) -> str | None:
    tags = [t for t in git("tag", "--list", "v*", "--sort=-v:refname", "--merged", "HEAD").splitlines()
            if t and t != version]
    return tags[0] if tags else None


def handover_notes(since: str | None) -> list[tuple[str, str]]:
    if since:
        # Added or changed: a task's later commits may amend its note (RULES 5b).
        names = git("diff", "--name-only", "--diff-filter=AM", f"{since}..HEAD", "--", "docs/handover/").splitlines()
    else:
        names = git("ls-files", "docs/handover/").splitlines()
    notes = []
    for name in sorted(n for n in names if n.endswith(".md") and not n.endswith("README.md")):
        path = ROOT / name
        if not path.is_file():
            continue
        title = next((l.lstrip("# ").strip() for l in path.read_text().splitlines() if l.startswith("#")), name)
        notes.append((name, title))
    return notes


def plan_rows(milestone: str) -> list[str]:
    """The PLAN.md table rows whose label cell is the milestone (`P1`), or
    whose title is (`**M3.`, the older table without labels), each under its
    table's header. Other cells may cite a milestone ("as P1"):
    they do not make a row that milestone's."""
    label = f"`{milestone}`"
    # The older M table has no label column: its rows start "| **M3. ...".
    def title_of(cells: list[str]) -> bool:
        return len(cells) > 1 and cells[1].startswith(f"**{milestone}.")
    lines = [l.rstrip() for l in (ROOT / "PLAN.md").read_text().splitlines()]
    out: list[str] = []
    for i, line in enumerate(lines):
        cells = [c.strip() for c in line.split("|")]
        if not line.lstrip().startswith("|") or not (label in cells or title_of(cells)):
            continue
        start = i
        while start > 0 and lines[start - 1].lstrip().startswith("|"):
            start -= 1
        if start + 1 >= i:  # the match is the header or its separator
            continue
        header = lines[start:start + 2]
        if header[0] not in out:
            out += ([""] if out else []) + header
        out.append(line)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", required=True)
    ap.add_argument("--milestone", default="")
    ap.add_argument("--since", default="")
    ap.add_argument("--zoo", default="")
    ap.add_argument("--build-id", action="append", default=[])
    a = ap.parse_args()

    commit = git("rev-parse", "HEAD")
    upstream = next(l[7:] for l in (ROOT / "engine/UPSTREAM").read_text().splitlines() if l.startswith("commit="))
    core_pin = (ROOT / "borg-core/PIN").read_text().split()[0]
    since = a.since or previous_tag(a.version)
    out = [
        f"# borg-reborn-rs {a.version}",
        "",
        "*Draft: edit before publishing.*",
        "",
        "Angband with the Borg playing it. The game is upstream Angband "
        f"[`{upstream[:8]}`](https://github.com/angband/angband/commit/{upstream}) unchanged; "
        "our harness hooks (`engine/patches/`), the `borgh` frontend and the Borg are linked in. "
        f"The Borg plays the borg-rs core vendored at `{core_pin[:8]}` (`borg-core/`) with the "
        "settings of `harness/borg.txt`.",
        "",
        f"- commit: [`{commit[:12]}`](https://github.com/diegoami/borg-reborn-rs/commit/{commit})",
        "- platforms: Linux x86_64, macOS arm64 (Apple silicon) and Windows x86_64",
        "- install: unpack your platform's package and read `PLAYING.txt`; `SHA256SUMS.txt` on this page checks the downloads",
        "",
    ]
    if a.milestone:
        rows = plan_rows(a.milestone)
        out += [f"## Milestone {a.milestone}", ""]
        out += rows or [f"PLAN.md has no row naming {a.milestone}."]
        out.append("")
    out += ["## Measurement", ""]
    out.append(f"borg-zoo's measurement of this commit: {a.zoo}" if a.zoo else
               "**Missing:** no borg-zoo measurement of this exact commit was given. "
               "Request a run of the release commit through borg-zoo before publishing (RULES 20).")
    out.append("")
    notes = handover_notes(since)
    out += [f"## Handover notes since {since or 'the start'}", ""]
    out += [f"- [{t}](https://github.com/diegoami/borg-reborn-rs/blob/{commit}/{n})" for n, t in notes] or ["(none)"]
    out.append("")
    if a.build_id:
        out += ["## Build identity", ""]
        for f in a.build_id:
            out += [f"`{pathlib.Path(f).name}`", "", "```", pathlib.Path(f).read_text().strip(), "```", ""]
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main())

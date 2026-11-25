#!/usr/bin/env python3
"""Rank the first observed failed descent requirement per cell, trip and depth."""

import argparse
from bisect import bisect_right
from collections import Counter, defaultdict
import json
from pathlib import Path


LEGACY_PREFIX = b"# heading up (bored and unable to dive: "


def structured_checks(cell):
    with (cell / "trace-core.jsonl").open() as records:
        for line in records:
            row = json.loads(line)
            for check in row.get("prepared", ()):
                depth = check["proposed_depth"]
                if depth != check["current_depth"] + 1:
                    continue
                failure = check["requirement"]
                yield row["trip_id"], depth, None if failure is None else failure["name"]


def legacy_checks(cell):
    """Recover only the documented bored-and-unable-to-dive notes."""
    timeline = []
    turns = []
    last_depth = trip = 0
    with (cell / "state.log").open("rb") as states:
        for line in states:
            turn, _, depths, *_ = line.split()
            depth = int(depths[1:].split(b"/")[0])
            if depth and not last_depth:
                trip += 1
            timeline.append((trip, depth))
            turns.append(int(turn))
            last_depth = depth
    with (cell / "messages.log").open("rb") as messages:
        for line in messages:
            _, turn, _, note = line.rstrip(b"\n").split(b" ", 3)
            if not note.startswith(LEGACY_PREFIX):
                continue
            step = bisect_right(turns, int(turn)) - 1
            if step >= 0:
                trip, depth = timeline[step]
                if depth:
                    yield trip, depth + 1, note[len(LEGACY_PREFIX):-1].decode()


def summarize(cells, checks):
    counts = defaultdict(Counter)
    observed = Counter()
    for cell in cells:
        character = json.loads((cell / "summary.json").read_text())["class"]
        first_failure = set()
        attempted = set()
        for trip, depth, failure in checks(cell):
            key = (trip, depth)
            attempted.add(key)
            if failure is not None and key not in first_failure:
                first_failure.add(key)
                counts[character, depth][failure] += 1
        for _, depth in attempted:
            observed[character, depth] += 1
    return [(character, depth, total,
             sorted(counts[character, depth].items(), key=lambda item: (-item[1], item[0])))
            for (character, depth), total in sorted(observed.items())]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("roots", nargs="+", type=Path, help="retained cell directories or parents")
    parser.add_argument("--legacy", action="store_true",
                        help="count only historical bored-and-unable-to-dive notes")
    args = parser.parse_args()
    cells = sorted({path.resolve().parent
                    for root in args.roots for path in root.rglob("summary.json")})
    if not cells:
        parser.error("no cell summaries found")
    rows = summarize(cells, legacy_checks if args.legacy else structured_checks)
    if not rows:
        parser.error("no preparedness observations found (historic cells may need --legacy)")
    print("class\tproposed_depth\tdistinct_trips_observed\tfirst_failed_requirement\ttrips")
    for character, depth, total, reasons in rows:
        for reason, count in reasons or [("prepared", 0)]:
            print(f"{character}\t{depth}\t{total}\t{reason}\t{count}")


if __name__ == "__main__":
    main()

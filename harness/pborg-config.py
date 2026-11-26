#!/usr/bin/env python3
"""Convert a known PBorg legacy settings file to pborg.yaml."""
import argparse
import json
import re
import sys
from pathlib import Path

SETTING = re.compile(r"(borg_[a-zA-Z0-9_@]+)\s*=\s*(.*)")
BEGIN = "[BEGIN FORMULA SECTION]"
END = "[END FORMULA SECTION]"


def convert(text):
    old_header = ("# The Borg's settings (harness/borg.txt). The zoo and ~/borg-play copy this\n"
                  "# file as the run's user/borg.txt; the vendored borg-rs core reads it.")
    text = text.removeprefix(old_header) if text.startswith(old_header) else text
    lines = ["# PBorg settings. Place this file at user/pborg.yaml."]
    keys = set()
    formulas = None
    for line in text.splitlines():
        if formulas is not None:
            if line.strip() == END:
                lines.extend(["formulas: |-", *("  " + row for row in formulas)])
                formulas = None
            else:
                formulas.append(line.expandtabs())
            continue
        if line.strip() == BEGIN:
            formulas = []
            continue
        setting = SETTING.fullmatch(line.strip())
        if not setting:
            if line.strip() and not line.lstrip().startswith("#"):
                line = "# " + line
            lines.append(line)
            continue
        key, raw = setting.groups()
        if key in keys:
            raise ValueError(f"duplicate setting {key}")
        keys.add(key)
        value = raw.strip()
        if re.fullmatch(r"-?(0|[1-9][0-9]*)", value):
            scalar = value
        elif value.upper() in ("TRUE", "FALSE"):
            scalar = value.lower()
        else:
            scalar = json.dumps(value, ensure_ascii=False)
        lines.append(f"{key}: {scalar}")
    if formulas is not None:
        raise ValueError("unterminated formula section")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("legacy_file", type=Path)
    args = parser.parse_args()
    try:
        text = (sys.stdin.buffer.read().decode("utf-8") if args.legacy_file == Path("-")
                else args.legacy_file.read_text(encoding="utf-8"))
        sys.stdout.buffer.write(convert(text).encode("utf-8"))
    except (OSError, ValueError) as error:
        parser.exit(2, f"{args.legacy_file}: {error}\n")


if __name__ == "__main__":
    main()

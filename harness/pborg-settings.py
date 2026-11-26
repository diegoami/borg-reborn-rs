#!/usr/bin/env python3
"""Read and edit the shipped PBorg YAML mapping without reformatting it."""
import argparse
import json
import re
import sys
from pathlib import Path

KEY = re.compile(r"borg_[a-zA-Z0-9_@]+")
ENTRY = re.compile(r"(borg_[a-zA-Z0-9_@]+):[ \t]*(.*)")


def scalar(value):
    if re.fullmatch(r"-?(0|[1-9][0-9]*)", value):
        return value
    if value.upper() in ("TRUE", "FALSE"):
        return value.lower()
    return json.dumps(value, ensure_ascii=False)


def settings(text):
    """Decode the single-line scalar entries in the shipped template."""
    values = {}
    for line in text.splitlines():
        match = ENTRY.fullmatch(line)
        if match:
            name, value = match.groups()
            if name in values:
                raise ValueError(f"duplicate setting {name}")
            if value.startswith('"'):
                value, end = json.JSONDecoder().raw_decode(value)
                tail = match[2][end:].strip()
                if tail and not tail.startswith('#'):
                    raise ValueError(f"invalid value for {name}")
            else:
                value = re.split(r"\s+#", value, maxsplit=1)[0].strip()
                if value in ('true', 'false'):
                    value = value.upper()
            values[name] = value
    return values


def edit(text, overrides, *, existing=False, remove=()):
    values = settings(text)
    changes = dict(overrides)
    for name, value in changes.items():
        if not KEY.fullmatch(name) or any(c in value for c in '\r\n\0'):
            raise ValueError(f"invalid setting: {name}")
        if existing and name not in values:
            raise ValueError(f"no key {name} in pborg.yaml")
    lines = []
    for line in text.splitlines(keepends=True):
        match = ENTRY.fullmatch(line.rstrip('\r\n'))
        if match and match[1] in remove:
            continue
        if match and match[1] in changes:
            name = match[1]
            line = f"{name}: {scalar(changes.pop(name))}\n"
        lines.append(line)
    if changes and lines and not lines[-1].endswith('\n'):
        lines.append('\n')
    lines.extend(f"{name}: {scalar(value)}\n" for name, value in changes.items())
    return ''.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('path', type=Path)
    commands = parser.add_subparsers(dest='command', required=True)
    get = commands.add_parser('get')
    get.add_argument('name')
    change = commands.add_parser('edit')
    change.add_argument('--set', dest='overrides', action='append', default=[])
    change.add_argument('--existing', action='store_true')
    change.add_argument('--remove', action='append', default=[])
    args = parser.parse_args()
    try:
        text = args.path.read_text(encoding='utf-8')
        if args.command == 'get':
            value = settings(text).get(args.name, '')
            sys.stdout.buffer.write((value + '\n').encode('utf-8'))
        else:
            pairs = [item.split('=', 1) for item in args.overrides]
            result = edit(text, pairs, existing=args.existing, remove=args.remove)
            sys.stdout.buffer.write(result.encode('utf-8'))
    except (OSError, ValueError) as error:
        parser.exit(2, f"{args.path}: {error}\n")


if __name__ == '__main__':
    main()

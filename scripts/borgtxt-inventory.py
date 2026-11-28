#!/usr/bin/env python3
"""Inventory of harness/pborg.yaml against the code (#576).

For every setting the vendored core declares (borg_init.rs BORG_SETTINGS,
RETIRED_SETTINGS), every shell key and every parameter of the registry
(params.rs), report:

- where the code reads it (file and function);
- its compiled default and the value harness/pborg.yaml ships, if any;
- the last borg-zoo definition that set it (optional: --zoo DIR, a borg-zoo
  checkout; "last" is the highest issue number).

Writes Markdown to stdout. Read only.

    python3 scripts/borgtxt-inventory.py [--zoo ../borg-zoo] > report.md
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC_DIRS = [ROOT / "borg-core" / "src", ROOT / "borg-reborn-rs" / "src"]
# Keys the shell reads itself (borg-reborn-rs, docs/ZOO-INTERFACE.md).
SHELL_KEYS = ["borg_reborn_core", "borg_scary_list"]


_SOURCES = None


def sources():
    """(file, lines, test) for every Rust file; test[i] is True inside a
    `#[cfg(test)] mod` block (tracked by its braces)."""
    global _SOURCES
    if _SOURCES is None:
        _SOURCES = []
        for d in SRC_DIRS:
            for f in sorted(d.rglob("*.rs")):
                lines = f.read_text(errors="replace").split("\n")
                test, depth, pending = [False] * len(lines), 0, False
                for i, line in enumerate(lines):
                    if line.strip().startswith("#[cfg(test)]"):
                        pending = True
                    elif pending and re.match(r"\s*(pub(\(\w+\))?\s+)?mod\s+\w+\s*\{", line):
                        depth, pending = 0, False
                        depth += line.count("{") - line.count("}")
                        test[i] = True
                        continue
                    elif pending and line.strip() and not line.strip().startswith("#"):
                        pending = False
                    if depth > 0:
                        test[i] = True
                        depth += line.count("{") - line.count("}")
                _SOURCES.append((f, lines, test))
    return _SOURCES


def enclosing_fn(lines, i):
    for j in range(i, -1, -1):
        m = re.search(r"\bfn\s+(\w+)", lines[j])
        if m:
            return m.group(1)
    return "?"


def readers(pattern, skip_files=()):
    """file:function for every non-test line matching pattern."""
    out = []
    rx = re.compile(pattern)
    for f, lines, test in sources():
        if f.name in skip_files:
            continue
        for i, line in enumerate(lines):
            code = line.lstrip()
            if (rx.search(line) and not test[i] and not code.startswith("//")
                    and not code.startswith("use ") and not re.match(r"^[A-Z_, ]+,?$", code)):
                site = f"{f.relative_to(ROOT).as_posix().split('/src/')[-1]}:{enclosing_fn(lines, i)}"
                if not site.endswith(":?") and site not in out:
                    out.append(site)
    return out


def core_settings():
    text = (ROOT / "borg-core/src/borg_init.rs").read_text()
    body = text[text.index("pub const BORG_SETTINGS"):text.index("];", text.index("pub const BORG_SETTINGS"))]
    settings = [(m.group(2), m.group(1), m.group(3))
                for m in re.finditer(r'\b([bi])\(c"(\w+)",\s*(-?\w+)\)', body)]
    rbody = text[text.index("pub const RETIRED_SETTINGS"):]
    retired = re.findall(r'retired\(c"(\w+)"', rbody[:rbody.index("];")])
    return settings, retired


def registry():
    text = (ROOT / "borg-core/src/params.rs").read_text()
    n = r"(-?[\d_]+)"
    rx = re.compile(r'^\s+(\w+) => \(' + n + r', "([^"]*)", ' + n + ", " + n
                    + r', "([^"]*)", "((?:[^"\\]|\\.)*)"\),?$', re.M)
    return [dict(name=m[1], default=int(m[2].replace("_", "")), unit=m[3],
                 min=int(m[4].replace("_", "")), max=int(m[5].replace("_", "")),
                 source=m[6], description=m[7]) for m in rx.finditer(text)]


def borg_txt():
    """key -> (value, section, line); class lines keep their @class."""
    out, section = {}, ""
    for n, line in enumerate((ROOT / "harness/pborg.yaml").read_text().split("\n"), 1):
        m = re.match(r"^#+\s*(.*?)\s*#*\s*$", line)
        if line.startswith("###") and m:
            section = m.group(1).strip("# ")
            continue
        m = re.match(r"^([a-z_0-9@]+):\s*(\S*)", line)
        if m:
            out[m.group(1)] = (m.group(2), section, n)
    return out


CLASSES = ["warrior", "mage", "druid", "priest", "necromancer", "paladin", "rogue",
           "ranger", "blackguard"]


def alias(name, pnames):
    """The registry name behind a pre-#520 per-class alias (docs/ZOO-INTERFACE.md,
    "Class values"), or the name itself."""
    if name in pnames:
        return name
    base, _, cls = name.rpartition("__")
    if cls in CLASSES and base in pnames:
        return base  # #696: <name>__<class>
    for c in CLASSES:
        for cand in (name.replace(f"_{c}_", "_", 1), name.removesuffix(f"_{c}")):
            if cand != name and cand in pnames:
                return cand
    return name


def zoo_settings(zoo):
    """key -> (issue, definition) of the last definition that sets it."""
    last = {}
    if not zoo:
        return last
    for f in sorted(Path(zoo, "definitions").glob("*.y*ml")):
        text = f.read_text(errors="replace")
        m = re.search(r"^issue:\s*(\d+)", text, re.M)
        issue = int(m.group(1)) if m else 0
        for key in set(re.findall(r"\b(borg_[a-z0-9_@]+)\s*:", text)):
            if key not in last or issue >= last[key][0]:
                last[key] = (issue, f.stem)
    return last


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--zoo", help="a borg-zoo checkout, for the last arm that set each key")
    args = ap.parse_args()
    shipped = borg_txt()
    zoo = zoo_settings(args.zoo)
    settings, retired = core_settings()
    params = registry()
    pnames = {p["name"] for p in params}

    def zoo_of(key):
        z = zoo.get(key)
        return f"#{z[0]} {z[1]}" if z else ""

    print("# harness/pborg.yaml inventory (#576)\n")
    print(f"`harness/pborg.yaml`: {len(shipped)} settings. Core settings: {len(settings)}; "
          f"retired: {len(retired)}; registry parameters: {len(params)}.\n")

    print("## Core settings (borg_init.rs BORG_SETTINGS)\n")
    print("| key | kind | default | shipped | section | read in | last zoo definition |")
    print("|---|---|---|---|---|---|---|")
    for name, kind, default in settings:
        r = readers(rf"\b{name.upper()}\b", skip_files=("borg_h.rs",))
        r = [x for x in r if not x.startswith("borg_init.rs:")] or ["(only borg_init.rs)"]
        s = shipped.get(name)
        print(f"| `{name}` | {kind} | {default} | {s[0] if s else ''} | {s[1] if s else ''} | "
              f"{', '.join(r)} | {zoo_of(name)} |")

    print("\n## Retired settings (read, ignored, warned)\n")
    for name in retired:
        s = shipped.get(name)
        print(f"- `{name}`" + (f": still in borg.txt, line {s[2]}" if s else ""))

    print("\n## Shell keys\n")
    print("| key | shipped | read in |")
    print("|---|---|---|")
    for name in SHELL_KEYS:
        s = shipped.get(name)
        print(f"| `{name}` | {s[0] if s else ''} | {', '.join(readers(rf'\"{name}\"')) or '-'} |")

    unknown = [k for k in shipped if not k.startswith("borg_param_") and k not in
               {n for n, _, _ in settings} | set(retired) | set(SHELL_KEYS)]
    print("\n## Keys in borg.txt the code does not declare\n")
    print("\n".join(f"- `{k}` (line {shipped[k][2]}, {shipped[k][1]})" for k in unknown) or "None.")

    print("\n## Registry parameters written in borg.txt\n")
    print("| parameter | default | shipped | section | readers | last zoo definition |")
    print("|---|---|---|---|---|---|")
    for key, (value, section, _) in shipped.items():
        if not key.startswith("borg_param_"):
            continue
        name = key[len("borg_param_"):]
        base = alias(name.split("@")[0], pnames)
        p = next((q for q in params if q["name"] == base), None)
        if base != name.split("@")[0]:
            name = f"{name} (alias of {base}@class)"
        n = len(readers(rf"Param::{base}\b", skip_files=("params.rs",))) if p else 0
        print(f"| `{name}` | {p['default'] if p else 'NOT IN REGISTRY'} | {value} | {section} | {n} | {zoo_of(key)} |")

    written = {alias(k[len("borg_param_"):].split("@")[0], pnames)
               for k in shipped if k.startswith("borg_param_")}
    print("\n## Candidates to expose: registry parameters not in borg.txt that a zoo "
          "definition set or that cite one of this project's issues\n")
    print("| parameter | default | unit | source | zoo | issues |")
    print("|---|---|---|---|---|---|")
    for p in params:
        if p["name"] in written:
            continue
        z = zoo_of("borg_param_" + p["name"])
        ours = re.findall(r"#(?:4\d\d|5\d\d|6\d\d)\b", p["description"])
        if z or ours:
            print(f"| `{p['name']}` | {p['default']} | {p['unit']} | {p['source']} | {z} | {' '.join(ours)} |")

    unread = [p["name"] for p in params
              if not readers(rf"Param::{p['name']}\b", skip_files=("params.rs",))]
    print(f"\n## Registry parameters with no `Param::` reader ({len(unread)})\n")
    print("Read through a family or alias (attack preferences, per-class names), or not at all:\n")
    print(", ".join(f"`{n}`" for n in unread) or "None.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Check that reference/ holds exactly the copies pins.json declares (#816).

Every rule of the `copies` section of pins.json names a pinned repository,
a source directory and a selection. For each rule this script lists the
selected files of the source at its pin (in the clone under vendor/, made
by scripts/setup.sh) and compares them with reference/, byte for byte:

  DIFFERS  a copy whose bytes are not its source's at the pin
  MISSING  a selected source file with no copy
  EXTRA    a file under reference/ that no rule selects (README.md and the
           generated deaths/INDEX.md excepted)

It reads nothing from the network and writes nothing. Exit status 0 when
every copy matches, 1 when anything differs, 2 when a pinned clone or
commit is missing (run scripts/setup.sh).

  python3 scripts/check-reference.py           report problems only
  python3 scripts/check-reference.py --verbose also list every file checked

The selection is refresh-sources.py's own (`selected`), so this check and
`refresh-sources.py --update` agree on what a copy is. Our own documents
live in docs/ and are not copies; other authors' documents stay in
engine/angband/ (docs/DOCUMENTATION-MAP.md).
"""

import argparse
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# Files of reference/ that are ours and no copy.
OWN = {'reference/README.md', 'reference/deaths/INDEX.md'}


def refresh_module():
    spec = importlib.util.spec_from_file_location('refresh_sources', ROOT / 'scripts/refresh-sources.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--verbose', action='store_true')
    ap.add_argument('--root', type=Path, default=ROOT, help=argparse.SUPPRESS)
    args = ap.parse_args(argv)
    root = args.root
    rs = refresh_module()
    data = json.loads((root / 'pins.json').read_text())
    problems, checked, expected = [], 0, set()
    for rule in data['copies']:
        pin = data['pins'][rule['pin']]
        source = rs.Source(root, rule['pin'], pin)
        if not (source.repo / '.git').exists():
            print(f"no clone of {rule['pin']} in {source.repo.relative_to(root)}: run scripts/setup.sh")
            return 2
        if rs.git(source.repo, 'cat-file', '-e', pin['commit'] + '^{commit}').returncode:
            print(f"{rule['pin']}: pin {pin['commit'][:12]} not in the clone: run scripts/setup.sh")
            return 2
        for rel, blob in sorted(rs.selected(rule, source.tree(pin['commit'], rule['from'])).items()):
            local = Path(rule['to']) / rel
            expected.add(local.as_posix())
            checked += 1
            path = root / local
            if not path.is_file():
                problems.append(f'MISSING  {local}  ({rule["pin"]} {rule["from"]}/{rel})')
                continue
            data_at_pin = subprocess.run(['git', '-C', str(source.repo), 'cat-file', 'blob', blob],
                                         capture_output=True, check=True).stdout
            if path.read_bytes() != data_at_pin:
                problems.append(f'DIFFERS  {local}  ({rule["pin"]} {pin["commit"][:12]})')
            elif args.verbose:
                print(f'ok       {local}')
    for path in sorted((root / 'reference').rglob('*')):
        rel = path.relative_to(root).as_posix()
        if path.is_file() and rel not in expected and rel not in OWN:
            problems.append(f'EXTRA    {rel}')
    for line in problems:
        print(line)
    print(f'{checked} copies checked, {len(problems)} problems')
    return 1 if problems else 0


if __name__ == '__main__':
    sys.exit(main())

#!/usr/bin/env python3
"""Check a consumer's vendored recorder, including added and removed files."""
import argparse
import hashlib
import json
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('pin', type=Path)
p.add_argument('vendor', type=Path)
a = p.parse_args()
pin = json.loads(a.pin.read_text())
root = a.vendor.resolve()
expected = pin['files_sha256']
actual = {}
for path in root.rglob('*'):
    relative = path.relative_to(root)
    if '__pycache__' in relative.parts or 'target' in relative.parts or '.git' in relative.parts:
        continue
    if path.is_symlink():
        raise SystemExit('recorder pin rejects symlinks: ' + str(relative))
    if path.is_file():
        actual[str(relative)] = hashlib.sha256(path.read_bytes()).hexdigest()
if actual != expected:
    changed = sorted(k for k in set(actual) | set(expected) if actual.get(k) != expected.get(k))
    raise SystemExit('recorder pin mismatch: ' + ', '.join(changed))
print('recorder pin OK:', pin['tag'], pin['commit'])

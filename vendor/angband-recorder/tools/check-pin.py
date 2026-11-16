#!/usr/bin/env python3
"""Seal regular release files; only root target/, .git/, __pycache__/ are excluded."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat

EXCLUDED_ROOT_DIRS = {'target', '.git', '__pycache__'}


def check(pin_path, vendor):
    # Resolve only after rejecting a symlinked root, including a dangling link.
    if not stat.S_ISDIR(vendor.lstat().st_mode):
        raise ValueError('recorder vendor root must be a real directory, not a symlink')
    root = vendor.resolve()
    pin = json.loads(pin_path.read_text())
    expected = pin['files_sha256']
    for name in expected:
        path = PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or str(path) != name:
            raise ValueError('unsafe recorder pin member: ' + name)
    actual = {}
    def on_error(error):
        raise ValueError('recorder pin traversal failed: ' + str(error))
    for directory, dirs, files in os.walk(root, followlinks=False, onerror=on_error):
        base = Path(directory)
        for name in list(dirs) + files:
            path = base / name
            relative = path.relative_to(root)
            mode = path.lstat().st_mode
            if not stat.S_ISREG(mode) and not stat.S_ISDIR(mode):
                raise ValueError('recorder pin rejects symlinks/special files: ' + str(relative))
            if stat.S_ISDIR(mode):
                if base == root and name in EXCLUDED_ROOT_DIRS:
                    dirs.remove(name)
            else:
                actual[str(relative)] = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        changed = sorted(k for k in set(actual) | set(expected) if actual.get(k) != expected.get(k))
        raise ValueError('recorder pin mismatch: ' + ', '.join(changed))
    print('recorder pin OK:', pin['tag'], pin['commit'])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('pin', type=Path)
    p.add_argument('vendor', type=Path)
    args = p.parse_args()
    try:
        check(args.pin, args.vendor)
    except (OSError, ValueError, KeyError) as error:
        p.exit(1, str(error) + '\n')


if __name__ == '__main__':
    main()

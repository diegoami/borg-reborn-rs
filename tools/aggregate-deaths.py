#!/usr/bin/env python3
"""Regenerate the Rust death registry without counting replays as independent lives."""
import argparse
from pathlib import Path
import borg_deaths

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--root', type=Path, default=borg_deaths.registry_root())
args = parser.parse_args()
with borg_deaths.registry_lock(args.root):
    borg_deaths.update_index(args.root)

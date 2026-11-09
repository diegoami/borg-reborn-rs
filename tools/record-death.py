#!/usr/bin/env python3
"""Export a curated Rust GUI/headless death dossier from validated original evidence."""
import argparse
from pathlib import Path
import borg_deaths

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--run', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--id', required=True)
parser.add_argument('--repeat', action='append', default=[], type=Path)
parser.add_argument('--issue')
parser.add_argument('--investigation')
args = parser.parse_args()
with borg_deaths.registry_lock(args.output.resolve().parent):
    borg_deaths.write_dossier(args.run, args.output.resolve(), args.id, issue=args.investigation or args.issue, repeat=args.repeat)

#!/usr/bin/env python3
"""Launch/resume the independent human save prepared by replay --play-from-turn."""
import argparse
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / 'tools'))
import human_takeover


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('branch', type=Path)
    args = parser.parse_args(argv)
    try:
        human_takeover.launch(args.branch)
        return 0
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())

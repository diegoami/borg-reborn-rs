#!/usr/bin/env python3
"""Compare games a package played with a smoke reference (#823, #824).

Usage: scripts/release/compare-games.py REFERENCE_DIR NAME=GAME_DIR [...]

REFERENCE_DIR is harness/reference/smoke-borg-rs (each NAME/ holds keys.log,
messages.log and state.log, xz-compressed) or a harness/suite.sh output of
the same platform (the logs uncompressed); each GAME_DIR is the OUTDIR of a
package's play-headless run. Every log must be byte-identical. Python only
(lzma), so it runs where no xz is installed: a clean macOS or Windows.
Exit 1 on the first difference, naming the file and the first differing line.
"""

import lzma
import pathlib
import sys


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    ref = pathlib.Path(sys.argv[1])
    bad = 0
    for spec in sys.argv[2:]:
        name, _, game = spec.partition('=')
        for log in ('keys.log', 'messages.log', 'state.log'):
            plain = ref / name / log
            want = plain.read_bytes() if plain.exists() else lzma.decompress((ref / name / f'{log}.xz').read_bytes())
            have = (pathlib.Path(game) / log).read_bytes()
            if want == have:
                print(f'{name}/{log}: identical ({len(have)} bytes)')
                continue
            bad += 1
            line = next((i for i, (a, b) in enumerate(zip(want.split(b'\n'), have.split(b'\n')), 1) if a != b),
                        min(want.count(b'\n'), have.count(b'\n')) + 1)
            print(f'{name}/{log}: DIFFERS from the reference at line {line} '
                  f'({len(have)} bytes, reference {len(want)})')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())

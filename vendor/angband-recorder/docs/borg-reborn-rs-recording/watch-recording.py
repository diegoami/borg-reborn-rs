#!/usr/bin/env python3
"""Play a recording's screen frames in the terminal.

  watch-recording.py RECORDING/native [--from-turn T] [--to-turn T] [--delay S] [--every N] [--follow]

Reads native/frames.jsonl (the same frames `angband-recorder view` rebuilds) and redraws the
80x24 screen at each command frame. Ctrl-C stops. --every N shows every Nth command frame.
--follow keeps reading while the recording is still being written, until native/complete.json
appears (a live view of a running recording; it lags behind when the game is faster).
"""
import argparse, json, sys, time
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('native', type=Path)
p.add_argument('--from-turn', type=int, default=0)
p.add_argument('--to-turn', type=int, default=None)
p.add_argument('--delay', type=float, default=0.05)
p.add_argument('--every', type=int, default=1)
p.add_argument('--follow', action='store_true')
a = p.parse_args()


def lines(path):
    """Complete lines of path; with --follow, waits for more until the recording completes."""
    while a.follow and not path.exists():
        time.sleep(0.2)
    with open(path) as f:
        pending = ''
        while True:
            chunk = f.readline()
            if chunk:
                pending += chunk
                if pending.endswith('\n'):
                    yield pending
                    pending = ''
                continue
            if not a.follow or (a.native / 'complete.json').exists():
                return
            time.sleep(0.2)


screen, dims, n, last = [], (0, 0), 0, None
out = sys.stdout
out.write('\x1b[?25l\x1b[2J')
try:
    for line in lines(a.native / 'frames.jsonl'):
        fr = json.loads(line)
        w, h = fr['width'], fr['height']
        if (w, h) != dims:
            screen, dims = [' '] * (w * h), (w, h)
        for at, code, *_ in fr['cells']:
            c = chr(code)
            screen[at] = c if c.isprintable() else ' '
        turn = fr.get('turn') or 0
        if fr.get('phase') not in ('command', 'final', 'exit') or turn < a.from_turn:
            continue
        if a.to_turn is not None and turn > a.to_turn:
            break
        n += 1
        if n % a.every and fr.get('phase') == 'command':
            continue
        rows = [''.join(screen[i:i + w]) for i in range(0, w * h, w)]
        last = (f"frame {fr.get('index')}  turn {turn}  depth {fr.get('depth')}  "
                f"HP {fr.get('hp')}/{fr.get('max_hp')}")
        out.write('\x1b[H' + '\n'.join(rows) + f"\n\x1b[7m {last} \x1b[0m\x1b[K")
        out.flush()
        time.sleep(a.delay)
except KeyboardInterrupt:
    pass
finally:
    out.write('\x1b[?25h\n')
    if last is None:
        print('watch-recording.py: no command, final or exit frame in that range')

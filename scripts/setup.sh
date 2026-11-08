#!/usr/bin/env bash
# Tools check and pinned checkouts in vendor/. Safe to rerun.
#
# Each related repository is cloned into vendor/<name> and checked out
# (detached) at its pin in pins.json, so a session reading vendor/ reads the
# pinned revision. An existing checkout at another revision is never moved
# unless --checkout is given and its working tree is clean; otherwise it is
# reported as WRONG REVISION. Additional commits a pin names (tags of
# interest, baselines) are fetched so `git -C vendor/<name> show COMMIT:path`
# works for them.
#
#   scripts/setup.sh              report problems, exit 0 (session start)
#   scripts/setup.sh --strict     exit 1 on any problem, including a checkout
#                                 that is not at its pin
#   scripts/setup.sh --checkout   move clean checkouts to their pins
#
# Adapted from borg-agent's scripts/setup.sh.
set -u
cd "$(dirname "$0")/.."
strict=0
move=0
for arg in "$@"; do
    case "$arg" in
        --strict) strict=1 ;;
        --checkout) move=1 ;;
        *) printf 'setup: unknown argument %s\n' "$arg" >&2; exit 2 ;;
    esac
done
problems=0
say() { printf 'setup: %s\n' "$*"; }

for tool in git cargo python3 cmake cc; do
    command -v "$tool" >/dev/null 2>&1 || { say "missing tool: $tool"; problems=1; }
done
[ -e /usr/include/X11/Xlib.h ] || say "note: X11 headers missing (needed only for the GUI launchers from M1; e.g. apt-get install libx11-dev)"

if command -v python3 >/dev/null 2>&1 && command -v git >/dev/null 2>&1; then
    mkdir -p vendor
    SETUP_MOVE=$move python3 - <<'PY' || problems=1
import json, os, subprocess, sys
from pathlib import Path

move = os.environ.get('SETUP_MOVE') == '1'
pins = json.loads(Path('pins.json').read_text())['pins']


def git(dest, *args, check=False):
    return subprocess.run(['git', '-C', str(dest), *args], capture_output=True, text=True, check=check)


def have(dest, commit):
    return git(dest, 'cat-file', '-e', commit + '^{commit}').returncode == 0


ok = True
for name, pin in sorted(pins.items()):
    repo = pin['repository']
    dest = Path('vendor') / repo.split('/')[1]
    fresh = not dest.exists()
    if fresh:
        print(f'setup: cloning {repo}', flush=True)
        subprocess.run(['git', 'clone', '-q', '--filter=blob:none', '--no-checkout',
                        f'https://github.com/{repo}.git', str(dest)], check=False)
    extra = list((pin.get('tag_of_interest') or {}).values())
    if 'baseline' in pin:
        extra.append(pin['baseline']['commit'])
    for commit in [pin['commit']] + extra:
        if not have(dest, commit):
            git(dest, 'fetch', '-q', 'origin', commit)
        if not have(dest, commit):
            print(f'setup: {name} {commit[:12]} MISSING')
            ok = False
    if not have(dest, pin['commit']):
        continue
    head = git(dest, 'rev-parse', 'HEAD').stdout.strip()
    # A clone made with --no-checkout has an empty index: nothing to lose.
    never_checked_out = git(dest, 'ls-files').stdout == ''
    dirty = not never_checked_out and git(dest, 'status', '--porcelain').stdout != ''
    if head == pin['commit'] and not never_checked_out:
        print(f"setup: {name} at pin {pin['commit'][:12]} ok")
        continue
    if fresh or never_checked_out or (move and not dirty):
        r = git(dest, 'checkout', '-q', '--detach', pin['commit'])
        if r.returncode == 0:
            print(f"setup: {name} checked out at pin {pin['commit'][:12]}")
            continue
        print(f"setup: {name} checkout of {pin['commit'][:12]} failed: {r.stderr.strip()}")
        ok = False
        continue
    why = 'local changes' if dirty else 'rerun with --checkout to move it'
    print(f"setup: {name} WRONG REVISION: at {head[:12]}, pin {pin['commit'][:12]} ({why})")
    ok = False
sys.exit(0 if ok else 1)
PY
fi

if [ $problems -ne 0 ]; then
    say "finished with problems (see above)"
    [ $strict -eq 1 ] && exit 1
else
    say "ready"
fi
exit 0

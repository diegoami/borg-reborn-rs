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
# vendor/borg-cemetery is kept at death-tools-pin.txt, not pins.json: it is
# the death tools' fallback checkout (tools/cemetery_tools.py, #902), so a
# clean checkout always follows the pin, without --checkout. It is a full
# checkout (about 2 GB: the exporter registers across all its trees), so it
# lives once in the primary checkout's vendor/, shared by every worktree.
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


def sync(name, repo, dest, commit, extra, move):
    """Clone dest if needed and check it out at commit; False on a problem."""
    ok = True
    fresh = not dest.exists()
    if fresh:
        print(f'setup: cloning {repo}', flush=True)
        subprocess.run(['git', 'clone', '-q', '--filter=blob:none', '--no-checkout',
                        f'https://github.com/{repo}.git', str(dest)], check=False)
    for c in [commit] + extra:
        if not have(dest, c):
            git(dest, 'fetch', '-q', 'origin', c)
        if not have(dest, c):
            print(f'setup: {name} {c[:12]} MISSING')
            ok = False
    if not have(dest, commit):
        return False
    head = git(dest, 'rev-parse', 'HEAD').stdout.strip()
    # A clone made with --no-checkout has an empty index: nothing to lose.
    never_checked_out = git(dest, 'ls-files').stdout == ''
    dirty = not never_checked_out and git(dest, 'status', '--porcelain').stdout != ''
    if head == commit and not never_checked_out:
        print(f"setup: {name} at pin {commit[:12]} ok")
        return ok
    if fresh or never_checked_out or (move and not dirty):
        r = git(dest, 'checkout', '-q', '--detach', commit)
        if r.returncode == 0:
            print(f"setup: {name} checked out at pin {commit[:12]}")
            return ok
        print(f"setup: {name} checkout of {commit[:12]} failed: {r.stderr.strip()}")
        return False
    why = 'local changes' if dirty else 'rerun with --checkout to move it'
    print(f"setup: {name} WRONG REVISION: at {head[:12]}, pin {commit[:12]} ({why})")
    return False


ok = True
for name, pin in sorted(pins.items()):
    repo = pin['repository']
    # A pin on a non-default branch names its own clone (see pins.json).
    dest = Path('vendor') / pin.get('vendor', repo.split('/')[1])
    extra = list((pin.get('tag_of_interest') or {}).values())
    if 'baseline' in pin:
        extra.append(pin['baseline']['commit'])
    ok = sync(name, repo, dest, pin['commit'], extra, move) and ok

# The death tools' pinned checkout (#902): always moved to the pin when clean,
# in the primary checkout's vendor/ (a linked worktree shares it).
death_pin = Path('death-tools-pin.txt').read_text().strip()
common = git('.', 'rev-parse', '--path-format=absolute', '--git-common-dir').stdout.strip()
primary = Path(common).parent if common else Path('.')
(primary / 'vendor').mkdir(exist_ok=True)
ok = sync('borg-cemetery (death-tools-pin.txt)', 'diegoami/borg-cemetery',
          primary / 'vendor/borg-cemetery', death_pin, [], True) and ok
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

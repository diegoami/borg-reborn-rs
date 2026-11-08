#!/usr/bin/env bash
# Tools check and pinned clones into vendor/. Safe to rerun.
# Never fails a session start: problems are printed, and the exit status is
# 0 unless --strict is given. Adapted from borg-agent's scripts/setup.sh.
set -u
cd "$(dirname "$0")/.."
strict=0
[ "${1:-}" = "--strict" ] && strict=1
problems=0
say() { printf 'setup: %s\n' "$*"; }

for tool in git cargo python3 cmake cc; do
    command -v "$tool" >/dev/null 2>&1 || { say "missing tool: $tool"; problems=1; }
done
[ -e /usr/include/X11/Xlib.h ] || say "note: X11 headers missing (needed only for the GUI launchers from M1; e.g. apt-get install libx11-dev)"

if command -v python3 >/dev/null 2>&1 && command -v git >/dev/null 2>&1; then
    mkdir -p vendor
    python3 - <<'PY' || problems=1
import json, subprocess, sys
from pathlib import Path

pins = json.loads(Path('pins.json').read_text())['pins']
ok = True
for name, pin in sorted(pins.items()):
    repo = pin['repository']
    dest = Path('vendor') / repo.split('/')[1]
    if not dest.exists():
        print(f'setup: cloning {repo}', flush=True)
        subprocess.run(['git', 'clone', '-q', '--filter=blob:none', '--no-checkout',
                        f'https://github.com/{repo}.git', str(dest)], check=False)
    commits = [pin['commit']] + [c for c in (pin.get('tag_of_interest') or {}).values()]
    if 'baseline' in pin:
        commits.append(pin['baseline']['commit'])
    for commit in commits:
        def have():
            return subprocess.run(['git', '-C', str(dest), 'cat-file', '-e', commit + '^{commit}'],
                                  capture_output=True).returncode == 0
        if not have():
            subprocess.run(['git', '-C', str(dest), 'fetch', '-q', 'origin', commit], check=False)
        good = have()
        print(f"setup: {name} {commit[:12]} {'ok' if good else 'MISSING'}")
        ok &= good
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

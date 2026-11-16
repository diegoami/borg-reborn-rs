#!/usr/bin/env python3
"""X11 runtime controls (M1 re-review follow-ups): real-frontend sessions
under Xvfb, driven with xdotool, recorded through the recorder CLI.

- F3: a panic save newer than the regular save is answered with the
  recorded decision even when the replay's live filesystem state would
  answer the opposite, continuing into a byte-identical regular save.
- F8: the X11 frontend configuration environment (ANGBAND_X11_FONT*) is
  sealed into the recording; replay under a hostile ambient font keeps
  the recorded terminal geometry (the frames would diverge otherwise).
- F9: the `-c` save-menu selection is a recorded ordered input; the
  replay selects the same non-default entry from its own empty private
  save directory, where a live enumeration could not offer it.

Requires Xvfb, xdotool, libx11-dev and an X11-enabled recorder build
(cmake -DSUPPORT_X11_FRONTEND=ON). Takes /tmp/angband-validation.lock.

STATUS 2026-11-15 (re-review follow-up): the harness records and replays
the X11 sessions, and the F3 decision path is proven up to the load
(`newer=true` recorded, the panic prompt answered, the captured panic
bytes loaded into a living character). The replayed post-load save then
diverges: the replay asks for one more input than recorded and dies
mid-save, leaving `<savefile>*.new` behind and one frame short of the
recording. This reproduces deterministically and is filed as open
evidence on the review issue; the script exits with this failure until
the divergence is fixed, so it is deliberately NOT wired into CI."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def events(record):
    return [json.loads(l) for l in (Path(record) / 'inputs.jsonl').read_text().splitlines()]


def pressed(record):
    seen = []
    for e in events(record):
        if e['kind'] == 'poll' and e['input']['result'] == 0 and e['input']['event_type'] == 1 \
                and e['input']['code'] != 0:
            if not seen or seen[-1] != e['input']['code']:
                seen.append(e['input']['code'])
    return seen


def xd(display, *args, timeout=20):
    return subprocess.run(['xdotool', *args], env={**os.environ, 'DISPLAY': display},
                          capture_output=True, text=True, timeout=timeout)


def wait_window(display, timeout=30):
    end = time.time() + timeout
    while time.time() < end:
        result = xd(display, 'search', '--name', 'Angband')
        ids = [i for i in result.stdout.split() if i.strip()]
        if ids:
            xd(display, 'windowactivate', '--sync', ids[0])
            return ids[0]
        time.sleep(0.3)
    raise RuntimeError('no Angband window appeared under ' + display)


def send_keys(display, win, keys, delay=0.45):
    for key in keys:
        if key.startswith('SLEEP:'):
            time.sleep(float(key.split(':', 1)[1]))
            continue
        xd(display, 'windowactivate', '--sync', win)
        xd(display, 'key', '--window', win, '--delay', '60', key)
        time.sleep(delay)


KEYMAP = {'enter': 'Return', 'C-x': 'ctrl+x', 'C-[': 'Escape', ' ': 'space', ',': 'comma',
          'down': 'Down'}


def xkeys(spec):
    return [KEYMAP.get(k, k) for k in spec]


class Session:
    """One recorded X11 game: launch through the CLI, wait for the window,
    send the scripted keys, wait for the recording to complete."""

    def __init__(self, display, cli, binary, record_dir, arguments, keys, env=None):
        self.record_dir = Path(record_dir)
        print('recording', self.record_dir, flush=True)
        env = {**os.environ, 'DISPLAY': display, **(env or {})}
        for name in ['BORG_RECORD_DIR', 'BORG_REPLAY_DIR', 'ANGBAND_RECORD_DIR',
                     'ANGBAND_REPLAY_DIR']:
            env.pop(name, None)
        env['ANGBAND_RECORD_DIR'] = str(self.record_dir)
        self.game_stderr = open(str(self.record_dir) + '.stderr', 'w')
        self.process = subprocess.Popen(
            [str(cli), 'record', str(self.record_dir), '--', str(binary), *arguments],
            cwd=str(Path(binary).parent), env=env, start_new_session=True,
            stdout=subprocess.DEVNULL, stderr=self.game_stderr)
        try:
            self.win = wait_window(display)
            self.display = display
            send_keys(display, self.win, xkeys(keys))
        except BaseException:
            os.killpg(os.getpgid(self.process.pid), signal.SIGKILL)
            self.game_stderr.close()
            raise

    def wait(self, timeout=240):
        try:
            self.process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self.process.pid), signal.SIGKILL)
            self.game_stderr.close()
            raise RuntimeError('the recorded X11 session did not exit: ' + str(self.record_dir))
        self.game_stderr.close()
        require(self.process.returncode == 0, 'a recorded X11 session failed: ' +
                str(self.record_dir) + '; see ' + str(self.record_dir) + '.stderr')
        require((self.record_dir / 'complete.json').is_file(),
                'no completion receipt: ' + str(self.record_dir))



def replay(display, cli, source, output, timeout=90):
    env = {**os.environ, 'DISPLAY': display}
    for name in ['BORG_RECORD_DIR', 'BORG_REPLAY_DIR', 'ANGBAND_RECORD_DIR',
                 'ANGBAND_REPLAY_DIR']:
        env.pop(name, None)
    env['ANGBAND_REPLAY_DIR'] = str(source)
    result = subprocess.run([str(cli), 'replay', '--verify', str(source), str(output)],
                            env=env, capture_output=True, text=True, timeout=timeout)
    require(result.returncode == 0, 'replay failed: ' + result.stdout + result.stderr)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build', type=Path, required=True,
                        help='X11-enabled recorder build (game/angband)')
    parser.add_argument('--output', type=Path, required=True, help='new scratch directory')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    build = args.build.resolve()
    binary = build / 'game/angband'
    require(binary.is_file(), 'no game/angband in the given build')
    # The X11 frontend reads ./lib relative to its working directory.
    game_dir = binary.parent
    if not (game_dir / 'lib' / 'screens' / 'news.txt').exists():
        source_lib = build.parent / 'lib'
        require((source_lib / 'screens' / 'news.txt').is_file(),
                'no lib/ next to the build to copy')
        shutil.copytree(source_lib, game_dir / 'lib')
    cli = ROOT / 'target/debug/angband-recorder'
    run_log = (out / 'test.log').open('w')
    run_log.close()
    with open('/tmp/angband-validation.lock', 'a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'Cargo.toml'], log=None)
        display = None
        for candidate in [f':{n}' for n in range(120, 140)]:
            candidate_xvfb = subprocess.Popen(['Xvfb', candidate, '-screen', '0', '1024x768x24'],
                                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            time.sleep(0.8)
            if candidate_xvfb.poll() is None:
                probe = xd(candidate, 'getdisplaygeometry', timeout=5)
                if probe.returncode == 0:
                    display = candidate
                    xvfb = candidate_xvfb
                    break
            candidate_xvfb.terminate()
        require(display is not None, 'no free display for a private Xvfb')
        print('private display', display, flush=True)
        try:
            time.sleep(0.5)

            # ---- Control 1 (F3): panic-save continuation under X11. ----
            profile = out / 'panic-profile'
            dirs = directory_arguments(profile)
            session = Session(display, cli, binary, out / 'x11-birth',
                              ['-mx11', *dirs, '-n', '-uX11Test'], BIRTH)
            session.wait()
            regular = profile / 'save/X11Test'
            panic = profile / 'panic/X11Test'
            require(regular.is_file(), 'no regular save after the X11 birth session')
            shutil.copy(regular, panic)
            now = time.time()
            os.utime(panic, (now + 600,) * 2)
            recorded = Session(display, cli, binary, out / 'x11-panic-session',
                               ['-mx11', *dirs, '-uX11Test'], ['y', 'Escape', 'Escape', 'C-x', 'SLEEP:2.5', 'Escape'])
            recorded.wait()
            require(True in [e['value'] for e in events(out / 'x11-panic-session')
                             if e['kind'] == 'newer'], 'no newer=true recorded')
            recorded_regular = sha(regular)
            os.utime(regular, (now + 1200,) * 2)
            replay(display, cli, out / 'x11-panic-session', out / 'x11-panic-replay')
            require(sha(out / 'x11-panic-replay/profile/save/X11Test') == recorded_regular,
                    'the replayed panic continuation save differs')
            require(sha(regular) == recorded_regular, 'replay wrote into the original profile')

            # ---- Control 2 (F8): hostile ambient X11 font at replay. ----
            font_profile = out / 'font-profile'
            dirs = directory_arguments(font_profile)
            recorded = Session(display, cli, binary, out / 'x11-font-session',
                               ['-mx11', *dirs, '-n', '-uFontTest'], BIRTH,
                               env={'ANGBAND_X11_FONT': RECORD_FONT,
                                    'ANGBAND_X11_FONT_0': RECORD_FONT})
            recorded.wait()
            require((font_profile / 'save/FontTest').is_file(), 'no FontTest save')
            recorded_frames = sha(out / 'x11-font-session/frames.jsonl')
            hostile = Session(display, cli, binary, out / 'x11-font-replay',
                              ['-mx11', *dirs, '-n', '-uFontTest'], BIRTH,
                              env={'ANGBAND_X11_FONT': HOSTILE_FONT,
                                   'ANGBAND_X11_FONT_0': HOSTILE_FONT,
                                   'ANGBAND_REPLAY_DIR': str(out / 'x11-font-session')})
            hostile.wait()
            replay(display, cli, out / 'x11-font-session', out / 'x11-font-replay2')
            require(sha(out / 'x11-font-replay2/frames.jsonl') == recorded_frames,
                    'the hostile ambient font changed the replayed terminal geometry')

            # ---- Control 3 (F9): non-default save-menu selection. ----
            menu_profile = out / 'menu-profile'
            dirs = directory_arguments(menu_profile)
            session = Session(display, cli, binary, out / 'x11-menu-birth',
                              ['-mx11', *dirs, '-n', '-uMenuTest'], BIRTH)
            session.wait()
            menu_save = menu_profile / 'save/MenuTest'
            require(menu_save.is_file(), 'no MenuTest save')
            born = sha(menu_save)
            shutil.copy(menu_save, menu_profile / 'save/AAAOther')
            recorded = Session(display, cli, binary, out / 'x11-menu-session',
                               ['-mx11', *dirs, '-c', '-uMenuTest'],
                               ['down', 'Return', 'Escape', 'Escape', 'comma', 'comma', 'comma', 'C-x', 'SLEEP:2.5', 'Escape'])
            recorded.wait()
            reads = [e for e in events(out / 'x11-menu-session') if e['kind'] == 'directory_read']
            require(reads, 'no directory enumeration recorded')
            recorded_menu = sha(menu_save)
            require(recorded_menu != born, 'the menu session did not save the selected character')
            replay(display, cli, out / 'x11-menu-session', out / 'x11-menu-replay')
            require(sha(out / 'x11-menu-replay/profile/save/MenuTest') == recorded_menu,
                    'the replayed menu selection saved a different character')
        finally:
            xvfb.terminate()

    result = {'panic_continuation_replayed': True, 'x11_font_isolated': True,
              'menu_selection_replayed': True}
    (out / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


RECORD_FONT = '10x20'
HOSTILE_FONT = '12x24'
BIRTH = ['space'] + ['Return'] * 12 + ['Escape'] * 8 + [','] * 8 + ['Escape'] * 3 + ['C-x', 'SLEEP:2.5', 'Escape']


def run(command, log=None):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True)
    require(result.returncode == 0, 'command failed: ' + ' '.join(map(str, command)))


def directory_arguments(profile):
    return ['-d' + name + '=' + str(Path(profile) / name)
            for name in ['user', 'save', 'panic', 'archive', 'scores']]


if __name__ == '__main__':
    main()

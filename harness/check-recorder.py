#!/usr/bin/env python3
"""Check the consumer's real C/Rust link and native recorder round trip."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import signal
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
exe = a.binary.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
with open('/tmp/angband-validation.lock', 'a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    for mode in ['record', 'replay']:
        env = {**os.environ, 'ANGBAND_RECORD_DIR': str(out / mode), 'BORG_TOOLKIT_HEADLESS': '1'}
        for key in ['BORG_RECORD_DIR', 'BORG_REPLAY_DIR', 'ANGBAND_REPLAY_DIR']:
            env.pop(key, None)
        if mode == 'replay':
            env['ANGBAND_REPLAY_DIR'] = str(out / 'record')
        command = [str(exe), '-mborgh', '-n', '-uRecorderCheck']
        command += ['-d' + k + '=' + str(out / 'profile' / k) for k in ['user', 'save', 'archive', 'panic', 'scores']]
        command += ['--', '-seed', '42', '-race', '0', '-class', '0', '-turns', '300',
                    '-summary', str(out / (mode + '-summary.json')), '-save-on-stop', str(out / (mode + '.save'))]
        (out / (mode + '-checkpoints')).mkdir()
        command += ['-checkpoint-every', '50', '-checkpoint-dir', str(out / (mode + '-checkpoints'))]
        with (out / (mode + '.log')).open('w') as log:
            with subprocess.Popen(command, cwd=exe.parent, env=env, stdin=subprocess.DEVNULL,
                                  stdout=log, stderr=subprocess.STDOUT, start_new_session=True) as child:
                try:
                    status = child.wait(timeout=30)
                except BaseException:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                    raise
        assert status == 0, (mode, (out / (mode + '.log')).read_text())
        assert (out / mode / 'complete.json').exists(), 'missing native completion receipt'
        summary = json.loads((out / (mode + '-summary.json')).read_text())
        assert not summary['dead'], 'short integration control unexpectedly died'
    assert (out / 'record.save').read_bytes() == (out / 'replay.save').read_bytes(), 'final engine saves differ'
    original_checkpoints = {p.name:p.read_bytes() for p in (out / 'record-checkpoints').glob('*.save')}
    replay_checkpoints = {p.name:p.read_bytes() for p in (out / 'replay-checkpoints').glob('*.save')}
    assert original_checkpoints and original_checkpoints == replay_checkpoints, 'forked checkpoints differ'
    events = [json.loads(l) for l in (out / 'record/inputs.jsonl').read_text().splitlines()]
    assert any(e['kind'] == 'bot' for e in events), 'no bot actions captured'
    assert (out / 'record/frames.jsonl').stat().st_size, 'no observed frames'
    # Replaying a loaded capture must read its sealed starting save even when
    # that working path is absent, then save using live private-session state.
    loaded_profile = out / 'loaded-profile'
    for kind in ['user', 'save', 'archive', 'panic', 'scores']:
        (loaded_profile / kind).mkdir(parents=True)
    starting_save = out / 'loaded-start.save'
    shutil.copy2(out / 'record.save', starting_save)
    working_save = loaded_profile / 'save/input.save'
    shutil.copy2(starting_save, working_save)
    for mode in ['record', 'replay']:
        env = {**os.environ, 'ANGBAND_RECORD_DIR': str(out / ('loaded-' + mode)),
               'BORG_TOOLKIT_HEADLESS': '1', 'TZ': 'UTC0' if mode == 'record' else 'UTC+11'}
        for key in ['BORG_RECORD_DIR', 'BORG_REPLAY_DIR', 'ANGBAND_REPLAY_DIR']:
            env.pop(key, None)
        if mode == 'replay':
            assert not working_save.exists(), 'loaded replay must use captured file bytes'
            env['ANGBAND_REPLAY_DIR'] = str(out / 'loaded-record')
        command = [str(exe), '-mborgh', '-uRecorderLoaded']
        command += ['-d' + k + '=' + str(loaded_profile / k)
                    for k in ['user', 'save', 'archive', 'panic', 'scores']]
        command += ['--', '-seed', '42', '-race', '0', '-class', '0', '-turns', '600',
                    '-load', str(working_save), '-summary', str(out / ('loaded-' + mode + '-summary.json')),
                    '-save-on-stop', str(working_save)]
        with (out / ('loaded-' + mode + '.log')).open('w') as log:
            with subprocess.Popen(command, cwd=exe.parent, env=env, stdin=subprocess.DEVNULL,
                                  stdout=log, stderr=subprocess.STDOUT, start_new_session=True) as child:
                try:
                    status = child.wait(timeout=30)
                except BaseException:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                    raise
        assert status == 0, (mode, (out / ('loaded-' + mode + '.log')).read_text())
        assert not json.loads((out / ('loaded-' + mode + '-summary.json')).read_text())['dead']
        assert (out / ('loaded-' + mode) / 'complete.json').is_file()
        working_save.rename(out / ('loaded-' + mode + '.save'))
    assert (out / 'loaded-record.save').read_bytes() == (out / 'loaded-replay.save').read_bytes(), 'loaded-session saves differ'
    for name in ['record', 'replay', 'loaded-record', 'loaded-replay']:
        identity = json.loads((out / name / 'recording.json').read_text())
        assert identity['schema'] == 'angband-recorder-streams@2', 'consumer must use schema v2'
    # Build the generic reader outside any enclosing consumer workspace.
    shutil.copytree(ROOT / 'vendor/angband-recorder', out / 'reader-source')
    subprocess.run(['cargo', 'run', '--quiet', '--locked', '--manifest-path',
                    str(out / 'reader-source/Cargo.toml'), '--target-dir', str(out / 'reader-build'),
                    '--', 'verify', str(out / 'record'), str(out / 'replay')], check=True)
    subprocess.run(['cargo', 'run', '--quiet', '--locked', '--manifest-path',
                    str(out / 'reader-source/Cargo.toml'), '--target-dir', str(out / 'reader-build'),
                    '--', 'verify', str(out / 'loaded-record'), str(out / 'loaded-replay')], check=True)
print('Consumer schema-v2 round trips passed: fresh and captured-load final saves plus forked checkpoints')

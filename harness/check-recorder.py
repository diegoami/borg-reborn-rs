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
    # Build the generic reader outside any enclosing consumer workspace.
    shutil.copytree(ROOT / 'vendor/angband-recorder', out / 'reader-source')
    subprocess.run(['cargo', 'run', '--quiet', '--locked', '--manifest-path',
                    str(out / 'reader-source/Cargo.toml'), '--target-dir', str(out / 'reader-build'),
                    '--', 'verify', str(out / 'record'), str(out / 'replay')], check=True)
print('Consumer recorder round trip passed, including final native save bytes')

#!/usr/bin/env python3
"""Bounded, Borg-independent round trip on the exact vanilla engine + hooks."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
ENGINE = '42cd156ac9f95ddf4311e88dd7ad462292d6d1d6'

def run(command, *, cwd=None, env=None, text=None, log=None, timeout=180):
    with subprocess.Popen(list(map(str, command)), cwd=cwd, env=env,
                          stdin=subprocess.PIPE if text is not None else subprocess.DEVNULL,
                          stdout=log, stderr=subprocess.STDOUT, start_new_session=True, text=True) as child:
        try:
            child.communicate(text, timeout=timeout)
        except BaseException:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise
        if child.returncode:
            raise RuntimeError(f'command exited {child.returncode}: {command}; inspect log')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine', type=Path, required=True, help='Git repository containing the pinned vanilla commit')
    parser.add_argument('--output', type=Path, required=True, help='new scratch directory, retained for evidence')
    parser.add_argument('--prepared-build', type=Path, help='reuse an already built exact patched vanilla tree')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    with open('/tmp/angband-validation.lock', 'a') as lock, (out / 'test.log').open('w') as log:
        fcntl.flock(lock, fcntl.LOCK_EX)
        build = args.prepared_build.resolve() if args.prepared_build else out / 'build'
        if not args.prepared_build:
            archive = out / 'vanilla.tar'
            run(['git', '-C', args.engine, 'archive', ENGINE, '-o', archive], log=log)
            source = out / 'source'
            source.mkdir()
            with tarfile.open(archive) as tar:
                tar.extractall(source, filter='data')
            run(['patch', '-p1', '--input', ROOT / 'patches/angband-4.2.6-42cd156.patch'], cwd=source, log=log)
            run(['cmake', '-S', source, '-B', build, '-DSUPPORT_BORG=OFF', '-DSUPPORT_BORG_RECORDING=ON',
                 '-DANGBAND_RECORDER_DIR=' + str(ROOT), '-DSUPPORT_TEST_FRONTEND=ON', '-DSUPPORT_X11_FRONTEND=OFF',
                 '-DCMAKE_BUILD_TYPE=Release', '-DBUILD_DOC=OFF'], log=log)
            run(['cmake', '--build', build, '-j', '4'], log=log)
        run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'Cargo.toml'], log=log)
        cli = ROOT / 'target/debug/angband-recorder'
        binary = build / 'game/angband'
        keys = 'key space\n' + 'key enter\n' * 12 + 'key C-[\n' * 8 + 'key ,\n' * 8 + 'key C-[\n' * 3 + 'key C-x\nkey C-[\n'
        command = [binary, '-mtest', '-n', '-uRecorderTest']
        command += ['-d' + name + '=' + str(out / 'profile' / name) for name in ['user', 'save', 'panic', 'archive', 'scores']]
        run([cli, 'record', out / 'record', '--', *command], cwd=build / 'game', text=keys, log=log, timeout=30)
        saved = {str(p.relative_to(out / 'profile/save')):hashlib.sha256(p.read_bytes()).hexdigest()
                 for p in (out / 'profile/save').rglob('*') if p.is_file()}
        assert saved, 'no final engine save'
        run([cli, 'replay', '--verify', out / 'record', out / 'replay'], log=log, timeout=30)
        repeated = {str(p.relative_to(out / 'profile/save')):hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in (out / 'profile/save').rglob('*') if p.is_file()}
        assert saved == repeated, 'final save differs'
        frames = [json.loads(l) for l in (out / 'record/frames.jsonl').read_text().splitlines()]
        assert len(frames) > 10 and frames[-1]['turn'] >= 81 and frames[-1]['phase'] == 'exit'
        events = [json.loads(l) for l in (out / 'record/inputs.jsonl').read_text().splitlines()]
        assert any(e['kind'] == 'poll' and e['input']['result'] != 0 for e in events), 'empty polls absent'
        assert any(e['kind'] == 'clock' for e in events), 'clock absent'
        # The verifier must reject a changed RNG observation.
        frame_file = out / 'replay/frames.jsonl'
        original = frame_file.read_bytes()
        frame_file.write_bytes(original.replace(b'"quick":0', b'"quick":1', 1))
        bad = subprocess.run([str(cli), 'verify', str(out / 'record'), str(out / 'replay')], capture_output=True)
        assert bad.returncode != 0, 'changed RNG accepted'
        frame_file.write_bytes(original)
        result = {'engine':ENGINE,'frames':len(frames),'turn':frames[-1]['turn'],
                  'inputs':len(events),'final_saves':saved,'round_trip':'PASS','changed_rng_rejected':True,
                  'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
                  'prepared_build': bool(args.prepared_build)}
        (out / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))

if __name__ == '__main__':
    main()

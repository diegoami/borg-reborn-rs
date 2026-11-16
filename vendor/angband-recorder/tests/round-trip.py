#!/usr/bin/env python3
"""Bounded, Borg-independent round trip on the exact vanilla engine + hooks."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]
ENGINE = '42cd156ac9f95ddf4311e88dd7ad462292d6d1d6'


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


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


def save_hashes(directory):
    result = {str(p.relative_to(directory)):hashlib.sha256(p.read_bytes()).hexdigest()
              for p in directory.rglob('*') if p.is_file()}
    require(bool(result), 'no final engine save was produced')
    return result


def validate_channels(record):
    frames = [json.loads(l) for l in (record / 'frames.jsonl').read_text().splitlines()]
    require(len(frames) > 10 and frames[-1]['turn'] >= 81 and frames[-1]['phase'] == 'exit', 'missing final observed game frames')
    events = [json.loads(l) for l in (record / 'inputs.jsonl').read_text().splitlines()]
    families = {kind:sum(e['kind'] == kind for e in events) for kind in ['poll','clock','file','exists','terminal_init']}
    require(all(n > 0 for n in families.values()), 'missing mandatory external hook family: ' + str(families))
    require(sum(e['kind']=='poll' and e['input']['result']!=0 for e in events)>0, 'empty polls absent')
    require(sum(e['kind']=='poll' and e['input']['result']==0 for e in events)>0, 'successful polls absent')
    require(sum(e['kind']=='file' and e['present'] for e in events)>0, 'no captured file bytes')
    require(sum(e['kind']=='file' and not e['present'] for e in events)>0, 'no captured absent file')
    messages = [json.loads(l) for l in (record / 'messages.jsonl').read_text().splitlines()]
    require(bool(messages), 'no native messages captured')
    return frames, events, families, messages


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine', type=Path, required=True, help='Git repository containing the pinned vanilla commit')
    parser.add_argument('--output', type=Path, required=True, help='new scratch directory, retained for evidence')
    parser.add_argument('--prepared-build', type=Path, help='diagnostic reuse; source identity is explicitly UNVERIFIED')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    with open('/tmp/angband-validation.lock', 'a') as lock, (out / 'test.log').open('w') as log:
        fcntl.flock(lock, fcntl.LOCK_EX)
        build = args.prepared_build.resolve() if args.prepared_build else out / 'build'
        if not args.prepared_build:
            archive = out / 'vanilla.tar'
            run(['git', '-C', args.engine, 'archive', ENGINE, '-o', archive], log=log)
            source = out / 'source'; source.mkdir()
            with tarfile.open(archive) as tar:
                tar.extractall(source, filter='data')
            run(['patch', '-p1', '--input', ROOT / 'patches/angband-4.2.6-42cd156.patch'], cwd=source, log=log)
            run(['cmake', '-S', source, '-B', build, '-DSUPPORT_BORG=OFF', '-DSUPPORT_BORG_RECORDING=ON',
                 '-DANGBAND_RECORDER_DIR=' + str(ROOT), '-DSUPPORT_TEST_FRONTEND=ON', '-DSUPPORT_X11_FRONTEND=OFF',
                 '-DCMAKE_BUILD_TYPE=Release', '-DBUILD_DOC=OFF'], log=log)
            run(['cmake', '--build', build, '--target', 'OurExecutable', 'TransferLib', 'unittest-z-quark-quark', '-j', '4'], log=log)
            run([build / 'unittests/z-quark/quark/quark'], log=log)
        run(['cargo', 'build', '--locked', '--manifest-path', ROOT / 'Cargo.toml'], log=log)
        cli = ROOT / 'target/debug/angband-recorder'; binary = build / 'game/angband'
        keys = 'key space\n' + 'key enter\n'*12 + 'key C-[\n'*8 + 'key ,\n'*8 + 'key C-[\n'*3 + 'key C-x\n' + 'key C-[\n'*8
        command = [binary, '-mtest', '-n', '-uRecorderTest']
        command += ['-d' + name + '=' + str(out / 'profile' / name) for name in ['user','save','panic','archive','scores']]
        record_env = {**os.environ, 'TZ':'Pacific/Kiritimati', 'BORG_REPLAY_WIDTH':'80','BORG_REPLAY_HEIGHT':'24',
                      'BORG_REPLAY_KEY_SIZE':'256','BORG_REPLAY_SOFT_CURSOR':'0','BORG_REPLAY_COMPLEX_INPUT':'0'}
        run([cli,'record',out/'record','--',*command], cwd=build/'game', env=record_env, text=keys, log=log, timeout=30)
        saved = save_hashes(out / 'profile/save')
        # Preserve the originals as evidence, then remove the old targets. A
        # no-op replay saver must fail rather than reuse old bytes.
        original_saves = out / 'original-saves'; original_saves.mkdir()
        for name in saved:
            source = out / 'profile/save' / name
            destination = original_saves / name; destination.parent.mkdir(parents=True,exist_ok=True)
            source.rename(destination)
        replay_env = {**os.environ, 'TZ':'UTC0', 'BORG_REPLAY_WIDTH':'1','BORG_REPLAY_HEIGHT':'1',
                      'BORG_REPLAY_KEY_SIZE':'1','BORG_REPLAY_SOFT_CURSOR':'1','BORG_REPLAY_COMPLEX_INPUT':'1'}
        run([cli,'replay','--verify',out/'record',out/'replay'], env=replay_env, log=log, timeout=30)
        repeated = save_hashes(out / 'replay/profile/save')
        require(saved == repeated, 'replay final save bytes differ')
        require(not list((out/'profile/save').glob('*')), 'replay wrote into original save profile')
        require(not list((build / 'game').rglob('*.new')), 'replay wrote outside the replay profile')
        frames,events,families,messages = validate_channels(out/'record')
        frame_file = out / 'replay/frames.jsonl'; original = frame_file.read_bytes()
        changed = original.replace(b'"quick":0',b'"quick":1',1)
        require(changed != original, 'RNG negative control did not mutate a frame')
        frame_file.write_bytes(changed)
        bad = subprocess.run([str(cli),'verify',str(out/'record'),str(out/'replay')],capture_output=True)
        require(bad.returncode != 0, 'changed RNG accepted')
        frame_file.write_bytes(original)
        verified_source = not bool(args.prepared_build)
        result = {'engine':ENGINE if verified_source else None,'source_identity':'verified export+patch' if verified_source else 'UNVERIFIED prepared build',
                  'frames':len(frames),'turn':frames[-1]['turn'],'inputs':len(events),'hook_families':families,'messages':len(messages),
                  'final_saves':saved,'replay_recreated_separate_saves':True,'ambient_environment_isolated':True,
                  'round_trip':'PASS','changed_rng_rejected':True,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
                  'prepared_build':bool(args.prepared_build),'python_optimization':__debug__ is False}
        (out/'result.json').write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps(result))

if __name__ == '__main__':
    main()

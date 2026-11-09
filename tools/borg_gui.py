"""Record interactive Rust Borg sessions; verify by rerunning their external inputs.

Engine recorder adapted from better-borg-c 08a5cd019; see IMPORTED.json.
No C Borg state or decisions are substituted during replay.
"""
from datetime import datetime, timezone
import fcntl
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

import borg_runtime as runtime
from toolkit_common import serialized

ROOT = runtime.ROOT
SCHEMA = 'borg-rs-gui-recording@1'
STREAMS = ('events.txt', 'clock.txt', 'pid.txt', 'actions.txt', 'frames.jsonl',
           'file-inputs.txt', 'geometry.json', 'record-complete.json')
PROFILE_DIRS = ('user', 'save', 'archive', 'panic', 'scores')


def environment():
    # Calendar and locale are explicit; never archive credentials or DISPLAY.
    env = {k: v for k, v in os.environ.items() if not k.startswith(('BORG_', 'ANGBAND_', 'LLVM_PROFILE'))}
    env.update(LC_ALL='C.UTF-8', LANG='C.UTF-8', TZ='UTC')
    return env


def command(binary, data, profile, character, arguments, frontend='x11', archive=None):
    return [str(binary), '-m' + frontend,
            *['-d' + option + '=' + str(data / key) for option, key in (('gamedata', 'gamedata'), ('pref', 'customize'), ('screens', 'screens'), ('help', 'help'))],
            *['-d' + key + '=' + str(archive if key == 'archive' and archive is not None else profile / key) for key in PROFILE_DIRS],
            '-u' + character, *arguments]


def start(binary, profile, character, arguments, *, lock_fd=9, data_source=None,
          build_identity=None, session_context=None, on_start=None):
    """The shell owns fd9; this parent retains it until its sole child exits."""
    character = runtime.character_name(character)
    binary, profile = binary.resolve(), profile.resolve()
    lock = profile / 'save/.locks' / (character + '.lock')
    held = os.fstat(lock_fd)
    expected = lock.stat()
    if (held.st_dev, held.st_ino) != (expected.st_dev, expected.st_ino):
        raise ValueError('Launch GUI recording through start-borg.sh or run-rust-borg.sh')
    fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    before_frontend = arguments[:arguments.index('--')] if '--' in arguments else arguments
    if any(arg.startswith(('-d', '-m', '-u')) for arg in before_frontend):
        raise ValueError('Use BORG_PLAY_DATA_DIR and start-borg.sh NAME to select the recorded profile')
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    bundle = profile / 'recordings' / (stamp + '-' + character)
    frozen = bundle / 'runtime/game'
    frozen.mkdir(parents=True)
    shutil.copy2(binary, frozen / 'angband')
    for name in ('gamedata', 'customize', 'screens', 'help'):
        shutil.copytree((data_source or ROOT / 'angband/lib') / name, frozen / 'lib' / name)
    # Freeze local timezone rules, including a TZ file selected by the owner.
    zone = os.environ.get('TZ')
    zone_file = None
    if zone is None:
        zone_file = Path('/etc/localtime')
    elif zone:
        candidate = Path(zone.lstrip(':'))
        if candidate.is_absolute():
            zone_file = candidate
        else:
            candidate = Path('/usr/share/zoneinfo') / candidate
            if candidate.is_file():
                zone_file = candidate
        # A POSIX TZ rule string is self-contained when no zone file exists.
    if zone_file is not None:
        shutil.copyfile(zone_file, bundle / 'runtime/timezone')
        zone = 'frozen-file'
    if build_identity and build_identity.get('overlay'):
        for name, digest in build_identity['overlay_sha256'].items():
            source = ROOT / name
            if runtime.sha(source) != digest:
                raise ValueError('Visible engine overlay differs from retained source: ' + name)
            target = bundle / 'runtime/tool-source' / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
    initial = bundle / 'initial-profile'
    for name in PROFILE_DIRS:
        if name == 'save':
            (initial / name).mkdir(parents=True)
            save = profile / name / character
            if save.exists():
                shutil.copy2(save, initial / name / character)
        else:
            shutil.copytree(profile / name, initial / name)
    output = bundle / 'run'
    output.mkdir()
    session_archive = output / 'archive'
    shutil.copytree(initial / 'archive', session_archive)
    record = {'schema': SCHEMA, 'status': 'recording', 'character': character,
              'arguments': arguments, 'frontend': 'x11',
              'verification_model': 'rerun frozen Rust executable with original terminal polls/events, clock, RNG initialization PID, and file-read bytes; compare actions, frames, world fingerprints and final native save',
              'scope': 'engine observations; fresh Borg memory on load; no whole-private-Borg-state claim',
              'build_id': {'git_rev': subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip(),
                           'source_sha256': subprocess.check_output([str(ROOT / 'harness/identity.sh'), 'source-hash', 'rust'], text=True).strip(),
                           'binary_sha256': runtime.sha(frozen / 'angband')},
              'started': stamp, 'timezone': zone, 'origin_sha256': runtime.sha(initial / 'save' / character) if (initial / 'save' / character).exists() else None}
    if build_identity is not None:
        record['build_id'] = {**build_identity, 'binary_sha256': runtime.sha(frozen / 'angband')}
    if session_context is not None:
        record['session_context'] = session_context
    runtime.write(bundle / 'runtime/build-id.json', record['build_id'])
    runtime.write(bundle / 'recording.json', record)
    immutable = {str(p.relative_to(bundle)): runtime.sha(p) for folder in (bundle / 'runtime', initial) for p in folder.rglob('*') if p.is_file()}
    record['runtime_sha256'] = immutable
    runtime.write(bundle / 'recording.json', record)
    env = environment()
    env.update({k: v for k, v in os.environ.items() if k.startswith('ANGBAND_X11_')})
    env['BORG_RECORD_DIR'] = str(output)
    human_environment(record, env, output)
    env['TZ'] = ':' + str(bundle / 'runtime/timezone') if zone == 'frozen-file' else zone
    if env['TZ'] is None:
        raise ValueError('No timezone context recorded')
    print('Recording:', bundle, flush=True)
    print('Replay:', ROOT / 'replay-borg.py', bundle, flush=True)
    started = time.monotonic()
    with (output / 'engine.log').open('w') as log:
        child = subprocess.Popen(command(frozen / 'angband', frozen / 'lib', profile, character, arguments, archive=session_archive),
                                 cwd=frozen, env=env, stdout=log, stderr=subprocess.STDOUT, pass_fds=(lock_fd,))
        try:
            if on_start is not None:
                on_start(child, output)
            code = child.wait()
        except BaseException:
            # Keep the character lock until our child has actually stopped.
            child.terminate()
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
            raise
    record.update(returncode=code, duration_seconds=time.monotonic() - started)
    for name, digest in immutable.items():
        if runtime.sha(bundle / name) != digest:
            raise ValueError('Frozen input changed during recording: ' + name)
    # Archive output is per session, so simultaneous named games cannot mix
    # Borg.map/death logs. Preserve changed files only; initial files stay frozen.
    record['native_archive'] = []
    for source in list(session_archive.rglob('*')):
        if not source.is_file():
            continue
        relative = source.relative_to(session_archive)
        before = initial / 'archive' / relative
        if before.exists() and before.stat().st_mtime_ns == source.stat().st_mtime_ns and runtime.sha(before) == runtime.sha(source):
            source.unlink()
            continue
        record['native_archive'].append(str(relative))
    final = profile / 'save' / character
    if final.exists():
        shutil.copy2(final, bundle / 'final.save')
        record['final_sha256'] = runtime.sha(bundle / 'final.save')
    else:
        record['final_sha256'] = None
    if code != 0 or not (output / 'record-complete.json').exists() or not (output / 'frames.jsonl').stat().st_size:
        record.update(status='incomplete')
        runtime.write(bundle / 'recording.json', record)
        raise ValueError(f'GUI exited without a complete game recording (status {code}); evidence retained at {bundle}; ' + (output / 'engine.log').read_text()[-800:])
    with (output / 'frames.jsonl').open('rb') as src, gzip.open(bundle / 'frames.jsonl.gz', 'wb') as dst:
        shutil.copyfileobj(src, dst)
    with gzip.open(bundle / 'frames.jsonl.gz', 'rb') as stream:
        if hashlib.file_digest(stream, 'sha256').hexdigest() != runtime.sha(output / 'frames.jsonl'):
            raise ValueError('Lossless frame compression check failed; original raw frames retained')
    with (output / 'frames.jsonl').open() as stream:
        first_frame = json.loads(next(stream))
    record['captured_views'] = {'inventory': 'inventory' in first_frame, 'spells': 'spells' in first_frame,
                                'native_messages': (output / 'native-messages.jsonl').is_file()}
    record.update(status='complete')
    record['files_sha256'] = {str(p.relative_to(bundle)): runtime.sha(p) for p in bundle.rglob('*') if p.is_file() and p.name != 'recording.json'}
    runtime.write(bundle / 'recording.json', record)
    print(f'GUI recording complete: {bundle}', flush=True)
    if not session_context or session_context.get('kind') != 'human_takeover':
        import borg_deaths
        borg_deaths.maybe_register(bundle, issue=os.environ.get('BORG_DEATH_ISSUE'))
    return bundle


def human_environment(record, env, output):
    context = record.get('session_context', {})
    if context.get('visible_seed') is not None:
        env['BORG_VISIBLE_SEED'] = str(context['visible_seed'])
    if context.get('kind') == 'human_takeover':
        if context.get('calendar') == 'original-gui':
            env['BORG_HUMAN_GUI_CALENDAR'] = '1'
        env.update(BORG_HUMAN_SEED=str(context['seed']), BORG_HUMAN_READY=str(output / 'human-ready.json'),
                   BORG_HUMAN_APPROVE=str(output / 'human-approved'))


def validate(bundle, require_complete=True):
    bundle = bundle.resolve()
    record = json.loads((bundle / 'recording.json').read_text())
    if record.get('schema') != SCHEMA or (require_complete and record.get('status') != 'complete'):
        raise ValueError('Unsupported or incomplete GUI recording; no verified replay claim')
    hashes = record.get('files_sha256')
    if not isinstance(hashes, dict):
        if not require_complete:
            return record
        raise ValueError('GUI recording has no finalized file hashes')
    required = {'runtime/game/angband', 'run/' + STREAMS[0], 'run/record-complete.json', 'frames.jsonl.gz'}
    required.update('run/' + name for name in STREAMS)
    if record.get('captured_views', {}).get('native_messages'):
        required.add('run/native-messages.jsonl')
    if record.get('final_sha256'):
        required.add('final.save')
    if record.get('build_id', {}).get('overlay'):
        required.add('run/combat.jsonl')
    if not required.issubset(hashes):
        raise ValueError('GUI recording manifest is missing required streams/runtime')
    actual = {str(p.relative_to(bundle)) for p in bundle.rglob('*') if p.is_file()} - {'recording.json', 'verification.json'}
    if actual != set(hashes):
        raise ValueError('GUI recording file set changed')
    for name, digest in hashes.items():
        path = (bundle / name).resolve()
        if not path.is_relative_to(bundle) or runtime.sha(path) != digest:
            raise ValueError('GUI recording file changed: ' + name)
    for name, digest in record.get('runtime_sha256', {}).items():
        path = (bundle / name).resolve()
        if not path.is_relative_to(bundle) or runtime.sha(path) != digest:
            raise ValueError('Frozen GUI origin/runtime binding changed: ' + name)
    runtime.validate_binary(bundle, record)
    if record.get('status') == 'complete':
        footer = json.loads((bundle / 'run/record-complete.json').read_text())
        for field, filename in (('inputs', 'events.txt'), ('frames', 'frames.jsonl')):
            with (bundle / 'run' / filename).open('rb') as stream:
                count = sum(1 for line in stream if line.endswith(b'\n'))
            if type(footer.get(field)) is not int or footer[field] != count:
                raise ValueError('GUI complete input/frame footer count diverges: ' + field)
        with gzip.open(bundle / 'frames.jsonl.gz', 'rb') as stream:
            if hashlib.file_digest(stream, 'sha256').hexdigest() != runtime.sha(bundle / 'run/frames.jsonl'):
                raise ValueError('GUI compressed movie differs')
    return record


@serialized
def verify(bundle, output=None, timeout=None, *, engine=None):
    bundle = bundle.resolve()
    record = validate(bundle)
    source_hash = hashlib.sha256((runtime.sha(Path(__file__)) + runtime.sha(Path(runtime.__file__))).encode()).hexdigest()
    manifest_hash = runtime.sha(bundle / 'recording.json')
    cached = bundle / 'verification.json'
    if engine is not None and output is None:
        raise ValueError('Historical GUI helper verification needs retained output')
    if engine is None and output is None and timeout is None and cached.exists():
        prior = json.loads(cached.read_text())
        if prior.get('verified') is True and prior.get('manifest_sha256') == manifest_hash and prior.get('verifier_sha256') == source_hash:
            return prior
    with runtime.replay_workspace(output) as temporary:
        profile = temporary / 'profile'
        shutil.copytree(bundle / 'initial-profile', profile)
        output_run = temporary / 'run'
        output_run.mkdir()
        geometry = json.loads((bundle / 'run/geometry.json').read_text())
        env = environment()
        env['TZ'] = ':' + str(bundle / 'runtime/timezone') if record['timezone'] == 'frozen-file' else record['timezone']
        env.update(BORG_RECORD_DIR=str(output_run), BORG_REPLAY_DIR=str(bundle / 'run'), BORG_REPLAY_SYS=record['frontend'],
                   BORG_REPLAY_WIDTH=str(geometry['width']), BORG_REPLAY_HEIGHT=str(geometry['height']), BORG_REPLAY_KEY_SIZE=str(geometry['key_size']),
                   BORG_REPLAY_SOFT_CURSOR=str(geometry['soft_cursor']), BORG_REPLAY_COMPLEX_INPUT=str(geometry['complex_input']))
        human_environment(record, env, output_run)
        frozen = bundle / 'runtime/game'
        args = record['arguments']
        # X11-only options do not apply to the dummy display, whose input calls
        # are supplied by the recorder. Actual Rust decisions still execute.
        args = args[:args.index('--')] if '--' in args else args
        result = subprocess.run(command(Path(engine).resolve() if engine else frozen / 'angband', frozen / 'lib', profile, record['character'], args, 'test'),
                                cwd=frozen, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                                timeout=timeout if timeout is not None else max(30, record['duration_seconds'] * 2 + 30), stdin=subprocess.DEVNULL)
        if result.returncode:
            raise ValueError('GUI replay failed; original preserved: ' + result.stdout[-1500:])
        streams = (*STREAMS, 'native-messages.jsonl') if (bundle / 'run/native-messages.jsonl').exists() else STREAMS
        if (bundle / 'run/combat.jsonl').exists():
            streams = (*streams, 'combat.jsonl')
        for name in streams:
            if not (output_run / name).exists() or runtime.sha(output_run / name) != runtime.sha(bundle / 'run' / name):
                # Save only a useful first divergence, never overwrite originals.
                if name == 'frames.jsonl' and (output_run / name).exists():
                    with (output_run / name).open() as after, (bundle / 'run' / name).open() as before:
                        for n, (a, b) in enumerate(zip(before, after), 1):
                            if a != b:
                                raise ValueError(f'GUI replay diverges: frames.jsonl row {n}')
                raise ValueError('GUI replay diverges or did not consume complete input: ' + name)
        if record.get('session_context', {}).get('kind') == 'human_takeover':
            if runtime.sha(output_run / 'human-ready.json') != runtime.sha(bundle / 'run/human-ready.json'):
                raise ValueError('Human session initial state diverges')
        originals = {p.name for p in (bundle / 'run').glob('file-input-*.bin')}
        repeated = {p.name for p in output_run.glob('file-input-*.bin')}
        if originals != repeated:
            raise ValueError('GUI replay file-read set diverges')
        for name in originals:
            if runtime.sha(bundle / 'run' / name) != runtime.sha(output_run / name):
                raise ValueError('GUI replay file-read bytes diverge: ' + name)
        for name in record.get('native_archive', []):
            if runtime.sha(profile / 'archive' / name) != runtime.sha(bundle / 'run/archive' / name):
                raise ValueError('GUI replay native death archive diverges: ' + name)
        final = profile / 'save' / record['character']
        if (runtime.sha(final) if final.exists() else None) != record['final_sha256']:
            raise ValueError('GUI replay final native save diverges')
    proof = {'verified': True, 'manifest_sha256': manifest_hash, 'verifier_sha256': source_hash, 'model': record['verification_model']}
    if engine is None:
        runtime.write(cached, proof)
    else:
        proof['historical_helper_sha256'] = runtime.sha(Path(engine))
    if output is not None:
        runtime.write(output.resolve() / 'verification.json', proof)
    return proof


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise InterruptedError('GUI launcher interrupted')
    signal.signal(signal.SIGTERM, interrupted)
    try:
        if len(sys.argv) < 4:
            raise ValueError('Usage: borg_gui.py BINARY PROFILE CHARACTER [Angband options]')
        start(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], sys.argv[4:])
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        sys.exit(str(error))

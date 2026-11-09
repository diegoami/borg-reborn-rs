"""Rust-Borg adapters for the imported native save and terminal viewer tools.

Record automated headless runs, freeze their origin/runtime and verify reruns.
Human GUI interventions require a separate recorder; they are not inferred.
"""
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import gzip
import hashlib
import json
import itertools
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile
import time
import signal

from toolkit_common import serialized

import borg_saves as saves

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = 'borg-rs-headless-recording@1'


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def data_root():
    return Path(os.environ.get('BORG_PLAY_DATA_DIR', ROOT / '.angband-reborn')).resolve()


def character_name(name):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,63}', name):
        raise ValueError('Character name must be 1-64 letters/digits/underscore/dot/hyphen, starting with a letter or digit')
    return name


@contextmanager
def character_lock(save):
    lock = save.parent / '.locks' / (save.name + '.lock')
    lock.parent.mkdir(parents=True, exist_ok=True)
    with lock.open('a') as f:
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError('Character is already open through a launcher') from error
        yield


@serialized
def build():
    # Immutable toolkit caches preserve every historical engine. No refresh of
    # build/rust or a historical helper is performed by named play.
    import death_workflow as workflow
    from types import SimpleNamespace
    pin = workflow.source_commit('HEAD')
    args = SimpleNamespace(build=None)
    prepared = workflow.prepare_runtime(args, pin, workflow.tool_hashes())
    return Path(prepared['build_directory']), Path(prepared['binary']), prepared['build_id']


def inspect_save(save, directory, binary):
    """Native deserialize, engine fields only, exit before Borg/world execution."""
    source = ROOT / 'tools/inspect-save.c'
    cache = (directory / 'CMakeCache.txt').read_text().splitlines()
    engine_source = Path(next(line.split('=', 1)[1] for line in cache if line.startswith('CMAKE_HOME_DIRECTORY:INTERNAL=')))
    recipe = ['cc', '-O2', '-DALLOW_BORG', '-DBORG_HARNESS', '-DNDEBUG', '-I' + str(engine_source / 'src')]
    key = hashlib.sha256((sha(source) + sha(binary) + repr(recipe)).encode()).hexdigest()[:20]
    output = ROOT / 'build/save-inspect' / key
    reader = output / 'angband'
    if not reader.exists():
        output.mkdir(parents=True, exist_ok=True)
        obj = output / 'inspect.o'
        subprocess.run([*recipe, '-c', str(source), '-o', str(obj)], check=True)
        argv = [v for v in shlex.split((directory / 'CMakeFiles/OurExecutable.dir/link.txt').read_text())
                if not v.startswith('-Wl,--dependency-file=')]
        argv[argv.index('-o') + 1] = str(reader)
        argv += [str(obj), '-Wl,--wrap=savefile_load']
        subprocess.run(argv, cwd=directory, check=True)
        write(output / 'identity.json', {'game': sha(binary), 'source': sha(source), 'reader': sha(reader)})
    meta = json.loads((output / 'identity.json').read_text())
    if meta['game'] != sha(binary) or meta['source'] != sha(source) or meta['reader'] != sha(reader):
        raise ValueError('Save metadata reader changed; preserve cache and investigate')
    with tempfile.TemporaryDirectory(prefix='borg-save-info-') as tmp:
        dirs = [f'-d{name}={tmp}' for name in ('user', 'save', 'archive', 'panic', 'scores')]
        dirs += [f'-d{option}={engine_source / "lib" / name}' for option, name in (('gamedata','gamedata'), ('pref','customize'), ('screens','screens'), ('help','help'))]
        cmd = [str(reader), '-mborgh', *dirs,
               '--', '-seed', '1', '-race', '0', '-class', '0', '-turns', '2147483647', '-load', str(save.resolve())]
        result = subprocess.run(cmd, cwd=directory / 'game', capture_output=True, text=True, timeout=15)
        if result.returncode:
            raise ValueError('Cannot inspect this save; restore a living backup or start --new: ' + result.stderr[-500:])
        return json.loads(result.stdout)


def frozen_run(bundle, output, timeout=None, *, engine=None, extra_observations=True, takeover=None, native_recording=True, progress=False):
    bundle = bundle.resolve()
    record = json.loads((bundle / 'recording.json').read_text())
    profile = record['profile']
    runtime = bundle / 'runtime'
    env = {k: v for k, v in os.environ.items() if not k.startswith(('BORG_', 'ANGBAND_', 'LLVM_PROFILE'))}
    env.update(BORG_IMPL='rust', ANGBAND_BIN=str(Path(engine).resolve() if engine is not None else runtime / 'game/angband'),
               SAVE_ROOT=str(runtime / 'harness/saves'), BORGH_DATA_ROOT=str(runtime / 'game/lib'),
               RUN_TIMEOUT_SECONDS=str(timeout if timeout is not None else profile['wall_seconds']), CHECKPOINT_EVERY=str(profile['checkpoint_every']),
               STALL_TIMEOUT_SECONDS=str(profile.get('stall_seconds', 0)),
               LC_ALL='C.UTF-8', LANG='C.UTF-8', TZ='UTC')
    if not extra_observations:
        env['BORG_NO_EXTRA_OBSERVATIONS'] = '1'
    if takeover is not None:
        env.update(BORG_TAKEOVER_TURN=str(takeover['turn']), BORG_TAKEOVER_DIR=str(takeover['directory']))
    if record.get('toolkit'):
        env['BORG_TOOLKIT_HEADLESS'] = '1'
    if record.get('toolkit') and native_recording:
        env['BORG_TOOLKIT_NATIVE'] = '1'
        if (bundle / 'run/native').exists() and output.resolve() != (bundle / 'run').resolve():
            env['BORG_REPLAY_DIR'] = str(bundle / 'run/native')
    command = [str(runtime / 'harness/run.sh'), str(output), str(profile['seed']),
               str(profile['race']), str(profile['class']), str(profile['turn_limit']), profile['flags']]
    if not progress:
        return subprocess.run(command, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    started = time.monotonic()
    with tempfile.TemporaryFile() as console:
        child = subprocess.Popen(command, env=env, stdout=console, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            while True:
                try:
                    code = child.wait(timeout=5)
                    break
                except subprocess.TimeoutExpired:
                    tick = 'not yet observed'
                    keys = output / 'keys.log'
                    if keys.exists():
                        with keys.open('rb') as stream:
                            stream.seek(max(0, keys.stat().st_size - 8192))
                            lines = stream.read().split(b'\n')[:-1]
                        if lines and lines[-1].split() and lines[-1].split()[0].isdigit():
                            tick = lines[-1].split()[0].decode('ascii')
                    print(f'Progress: {time.monotonic()-started:.1f} wall seconds; engine tick {tick}', flush=True)
        except BaseException:
            # The session contains only our runner/supervisor/engine children.
            if child.poll() is None:
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL); child.wait()
            raise
        console.seek(0)
        return subprocess.CompletedProcess(command, code, console.read().decode('utf-8'))



@serialized
def record_run(args, *, prepared=None, profile_root=None, export_death=True):
    data_directory = Path(profile_root).resolve() if profile_root is not None else data_root()
    source_root = Path(prepared['source_root']) if prepared else ROOT
    runner_root = Path(prepared['runner_root']) if prepared else ROOT / 'harness'
    save = data_directory / 'save' / character_name(args.character)
    with character_lock(save):
        if prepared is None:
            directory, binary, identity = build()
            cached = json.loads((binary.parent.parent / 'prepared.json').read_text())
            source_root = Path(cached['source_root'])
            runner_root = Path(cached['runner_root'])
        else:
            directory, binary, identity = Path(prepared['build_directory']), Path(prepared['binary']), prepared['build_id']
        if args.from_backup is not None and args.from_backup not in range(1, 6):
            raise ValueError('Backup slot must be 1 through 5')
        candidate = save.parent / 'backups' / save.name / f'save.{args.from_backup}' if args.from_backup is not None else save
        if args.from_backup is not None and not candidate.is_file():
            raise ValueError('Backup does not exist: ' + str(candidate))
        origin = None
        with tempfile.TemporaryDirectory(prefix='borg-save-preflight-') as temporary:
            snapshot = Path(temporary) / 'origin.save'
            if candidate.exists() and not args.new:
                shutil.copy2(candidate, snapshot)
                origin = inspect_save(snapshot, directory, binary)
                if origin['dead']:
                    raise ValueError('Character is dead; use --from-backup or --new')
                if args.turns > 2147483647 - origin['turn']:
                    raise ValueError('Additional tick budget exceeds the native signed clock range; named save preserved')
                if (args.race is not None and args.race != origin['race']) or (args.cls is not None and args.cls != origin['class']):
                    raise ValueError('Requested race/class differs from the loaded character')
            if args.from_backup is not None:
                saves.restore(save, args.from_backup, validated_source=snapshot)
        initial = saves.backup(save)
        if origin:
            origin = inspect_save(save.parent / 'backups' / save.name / 'save.1', directory, binary)
            if origin['dead']:
                raise ValueError('Character changed to a dead save; named file preserved')
        stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
        bundle = data_directory / 'recordings' / (stamp + '-' + args.character)
        runtime = bundle / 'runtime'
        (runtime / 'harness/saves').mkdir(parents=True)
        (runtime / 'game/lib').mkdir(parents=True)
        shutil.copy2(binary, runtime / 'game/angband')
        for name in ('gamedata', 'customize', 'screens', 'help'):
            shutil.copytree(source_root / 'angband/lib' / name, runtime / 'game/lib' / name)
        for name in ('run.sh', 'supervise.py'):
            shutil.copy2(runner_root / name, runtime / 'harness' / name)
        tool_source = Path((prepared or cached)['tool_source_root'])
        shutil.copytree(tool_source, runtime / 'tool-source')
        config = data_directory / 'user/borg.txt'
        shutil.copy2(Path(prepared['config']) if prepared else config if config.exists() else ROOT / 'harness/borg.txt', runtime / 'harness/borg.txt')
        cfg = runtime / 'harness/borg.txt'
        cfg.write_text(re.sub(r'^borg_delay_factor = .*$', 'borg_delay_factor = 0', cfg.read_text(), flags=re.M))
        flags = f'menu,terminal-save,frames,key-limit={args.keys}'
        if not origin and args.race is None and args.cls is None:
            flags += ',random-birth'
        if getattr(args, 'munchkin', False):
            flags += ',munchkin'
        if origin:
            shutil.copy2(save.parent / 'backups' / save.name / 'save.1', runtime / 'harness/saves/origin.save')
            if sha(runtime / 'harness/saves/origin.save') != initial:
                raise ValueError('Frozen starting save differs from its verified backup')
            flags += ',save=origin.save,keep-load-rng'
        profile = {'seed': args.seed, 'race': origin['race'] if origin else (args.race or 0),
                   'class': origin['class'] if origin else (args.cls or 0),
                   'turn_limit': (origin['turn'] if origin else 0) + args.turns,
                   'flags': flags, 'wall_seconds': args.wall_seconds, 'checkpoint_every': args.checkpoint_every}
        if getattr(args, 'stall_seconds', 0):
            profile['stall_seconds'] = args.stall_seconds
        record = {'schema': SCHEMA, 'status': 'running', 'character': args.character,
                  'profile': profile, 'build_id': identity, 'origin': origin,
                  'original_named_sha256': initial, 'run_kind': 'backup branch' if args.from_backup is not None else 'save continuation; fresh Borg' if origin else 'new birth',
                  'rng_origin': 'saved engine RNG preserved; fresh Borg' if origin else 'seeded birth',
                  'calendar': 'seeded headless harness holiday context pinned to epoch',
                  'verification_model': 'rerun original Rust Borg from original origin; exact keys/messages/player-state/terminal frames/final save and scheduled world saves; not whole-Borg private-state shadow',
                  'toolkit': identity.get('overlay'),
                  'stop_boundary': 'next living native top-level command after tick/key threshold; pending commands finish',
                  'movie_channel': 'run/native/frames.jsonl' if identity.get('overlay') else 'run/frames.jsonl',
                  'runtime_sha256': {str(p.relative_to(bundle)): sha(p) for p in runtime.rglob('*') if p.is_file()}}
        if prepared:
            record['collection_context'] = prepared['context']
        write(runtime / 'build-id.json', identity)
        record['runtime_sha256']['runtime/build-id.json'] = sha(runtime / 'build-id.json')
        write(bundle / 'recording.json', record)
        print('Immutable session evidence:', bundle, flush=True)
        result = frozen_run(bundle, bundle / 'run', progress=True)
        (bundle / 'console.log').write_text(result.stdout)
        process = json.loads((bundle / 'run/process.json').read_text()) if (bundle / 'run/process.json').exists() else {}
        summary = json.loads((bundle / 'run/summary.json').read_text()) if (bundle / 'run/summary.json').exists() else None
        consistent_timeout = (process.get('status') == 'timeout' and process.get('engine_returncode') == 0
            and summary and summary.get('stop_reason') == 'wall_timeout' and (bundle / 'run/native/record-complete.json').exists())
        if result.returncode and not consistent_timeout:
            record['status'] = 'failed'
            record['files_sha256'] = {str(p.relative_to(bundle)): sha(p) for p in bundle.rglob('*') if p.is_file() and p.name != 'recording.json'}
            write(bundle / 'recording.json', record)
            raise ValueError(f'Headless run failed; named save preserved. Evidence: {bundle}')
        final = bundle / 'run/final.save'
        assert final.is_file() and final.stat().st_size
        for name, digest in record['runtime_sha256'].items():
            if sha(bundle / name) != digest:
                raise ValueError('Frozen runtime input changed; named save preserved: ' + name)
        frames = bundle / record.get('movie_channel', 'run/frames.jsonl')
        compressed = bundle / '.frames.jsonl.gz.partial'
        with frames.open('rb') as source, gzip.open(compressed, 'wb') as target:
            shutil.copyfileobj(source, target)
        with gzip.open(compressed, 'rb') as source:
            assert hashlib.sha256(source.read()).hexdigest() == sha(frames)
        os.replace(compressed, bundle / 'frames.jsonl.gz')
        record.update(status='timeout' if consistent_timeout else 'complete', summary=summary, publication='pending', finalized=True)
        record['files_sha256'] = {str(p.relative_to(bundle)): sha(p) for p in bundle.rglob('*')
                                 if p.is_file() and p.name != 'recording.json'}
        write(bundle / 'recording.json', record)
        validate(bundle, require_complete=not consistent_timeout)
        if consistent_timeout:
            record['publication'] = 'not published: finalized timeout; prefix qualification required'
            write(bundle / 'recording.json', record)
            print('Finalized timeout movie:', bundle, flush=True)
            return bundle
        try:
            caps = json.loads((bundle / 'run/checkpoints.json').read_text())['captures'] if (bundle / 'run/checkpoints.json').exists() else []
            if caps and (summary['dead'] or initial is None):
                recovery = caps[-1] if summary['dead'] else caps[0]
                saves.backup(save, bundle / 'run/checkpoints' / recovery['file'])
                record['recovery_backup'] = {'slot': 1, 'turn': recovery['game_turn'],
                                             'kind': 'last living checkpoint' if summary['dead'] else 'first recorded checkpoint'}
            saves.publish(save, final, initial)
        except (OSError, ValueError) as error:
            record['publication'] = 'refused: ' + str(error)
            write(bundle / 'recording.json', record)
            # borg-reborn-rs (M1 review): a confirmed natural death is recorded
            # even when the save cannot be published (RULES 12).
            if export_death and summary['dead']:
                import borg_deaths
                borg_deaths.maybe_register(bundle, issue=getattr(args, 'death_issue', None) or os.environ.get('BORG_DEATH_ISSUE'))
            raise ValueError(f'{error}; complete recording retained at {bundle}') from error
        record.update(publication='published', published_save_sha256=sha(save))
        write(bundle / 'recording.json', record)
        print(f"{summary['stop_reason']}: level{summary['clevel']}, depth{summary['depth']}, dead={summary['dead']}")
        print('Save:', save)
        print('Recording:', bundle)
        print('Replay:', ROOT / 'replay-borg.py', bundle)
        if export_death:
            import borg_deaths
            borg_deaths.maybe_register(bundle, issue=getattr(args, 'death_issue', None) or os.environ.get('BORG_DEATH_ISSUE'))
        return bundle


def validate(bundle, require_complete=True):
    bundle = bundle.resolve()
    record = json.loads((bundle / 'recording.json').read_text())
    if record.get('schema') != SCHEMA or (require_complete and record.get('status') != 'complete'):
        raise ValueError('Unsupported or incomplete recording; no verified replay claim is available')
    if not isinstance(record.get('files_sha256'), dict):
        if not require_complete:
            return record
        raise ValueError('Recording has no finalized file hashes')
    for name, digest in record['files_sha256'].items():
        path = (bundle / name).resolve()
        if not path.is_relative_to(bundle) or sha(path) != digest:
            raise ValueError('Recording file changed: ' + name)
    for name, digest in record.get('runtime_sha256', {}).items():
        path = (bundle / name).resolve()
        if not path.is_relative_to(bundle) or sha(path) != digest:
            raise ValueError('Frozen runtime input binding changed: ' + name)
    validate_binary(bundle, record)
    if record.get('toolkit'):
        validate_toolkit(bundle, record)
    return record


@contextmanager
def replay_workspace(output):
    temporary = output is None
    workspace = Path(tempfile.mkdtemp(prefix='borg-rust-replay-')) if temporary else output.resolve()
    if not temporary:
        workspace.mkdir(parents=True, exist_ok=False)
    try:
        yield workspace
    except BaseException as error:
        write(workspace / 'verification.json', {'verified': False, 'error': str(error)})
        if temporary and isinstance(error, Exception):
            raise ValueError(f'{error}; failed replay evidence retained at {workspace}') from error
        raise
    else:
        if temporary:
            shutil.rmtree(workspace)



OBSERVATION_FIELDS = frozenset(('inventory', 'spells', 'max_hp', 'max_sp', 'caster'))


def compare_observation_extension(original, extended):
    """Only absent new observation fields may be added; every old value stays."""
    with original.open() as before, extended.open() as after:
        for number, (left, right) in enumerate(itertools.zip_longest(before, after), 1):
            if left is None or right is None:
                raise ValueError(f'Observation replay frame count diverges at row {number}')
            expected, actual = json.loads(left), json.loads(right)
            for field in OBSERVATION_FIELDS - expected.keys():
                actual.pop(field, None)
            if expected != actual:
                fields = sorted(key for key in expected.keys() | actual.keys() if key not in expected or key not in actual or expected[key] != actual[key])
                raise ValueError(f'Observation replay original frame diverges at row {number}: {fields}')


@serialized
def verify(bundle, output=None, timeout=None, *, engine=None, observation_extension=False):
    if (engine is not None) != observation_extension or (observation_extension and output is None):
        raise ValueError('Observation rebuild verification requires its engine and retained output')
    bundle = bundle.resolve()
    record = validate(bundle)
    source_hash = sha(Path(__file__))
    manifest_hash = sha(bundle / 'recording.json')
    cached = bundle / 'verification.json'
    if engine is None and output is None and timeout is None and cached.exists():
        prior = json.loads(cached.read_text())
        if prior.get('verified') is True and prior.get('manifest_sha256') == manifest_hash and prior.get('verifier_sha256') == source_hash:
            return prior
    with replay_workspace(output) as workspace:
        output_run = workspace / 'run'
        result = frozen_run(bundle, output_run, timeout=timeout, **({"engine": engine} if engine is not None else {}))
        output_run_path = output_run
        if result.returncode:
            detail = ''
            for name in ('native/frames.jsonl', 'state.log', 'keys.log'):
                actual, original = output_run / name, bundle / 'run' / name
                if actual.exists() and original.exists() and sha(actual) != sha(original):
                    detail = first_difference(original, actual) + '\n'
                    break
            stderr = (output_run / 'stderr.log').read_text() if (output_run / 'stderr.log').exists() else ''
            raise ValueError(detail + 'Replay failed; original evidence preserved: ' + stderr[-1000:] + result.stdout[-500:])
        process = json.loads((output_run_path / 'process.json').read_text())
        if process['status'] != 'success' or process['engine_returncode'] != 0:
            raise ValueError('Replay process did not complete successfully')
        for filename in ('keys.log', 'messages.log', 'state.log', 'frames.jsonl', 'final.save', 'config.txt'):
            if filename == 'frames.jsonl' and observation_extension:
                compare_observation_extension(bundle / 'run' / filename, output_run_path / filename)
                continue
            if sha(output_run_path / filename) != sha(bundle / 'run' / filename):
                raise ValueError(first_difference(bundle / 'run' / filename, output_run_path / filename))
        if record.get('toolkit'):
            compare_native(bundle / 'run/native', output_run_path / 'native')
        native_original = {str(p.relative_to(bundle / 'run/archive')): p for p in (bundle / 'run/archive').rglob('*') if p.is_file()}
        native_repeated = {str(p.relative_to(output_run_path / 'archive')): p for p in (output_run_path / 'archive').rglob('*') if p.is_file()}
        if native_original.keys() != native_repeated.keys():
            raise ValueError('Replay native Borg archive set diverges')
        native_clock_differences = []
        for name, original in native_original.items():
            repeated = native_repeated[name]
            if sha(original) == sha(repeated):
                continue
            # Historical headless engines pin seasonal monster availability,
            # but their Rust death-log header reads the real wall clock. Keep
            # those original bytes/hashes; compare every other log byte.
            if name != 'borg-log.txt':
                raise ValueError('Replay native Borg archive diverges: ' + name)
            left, right = original.read_bytes().splitlines(keepends=True), repeated.read_bytes().splitlines(keepends=True)
            if len(left) != len(right):
                raise ValueError('Replay native death log length diverges')
            for number, (a, b) in enumerate(zip(left, right), 1):
                if a == b:
                    continue
                if number != 1 or not all(re.fullmatch(rb'[0-9]{4}/[0-9]{2}/[0-9]{2} [0-9]{2}:[0-9]{2}\n', value) for value in (a, b)):
                    raise ValueError('Replay native death log body diverges')
                native_clock_differences.append({'line': number, 'original': a.decode().strip(), 'rerun': b.decode().strip()})
        before, after = (json.loads((p / 'summary.json').read_text()) for p in (bundle / 'run', output_run_path))
        before.pop('wall_seconds', None);after.pop('wall_seconds', None)
        if before != after:
            raise ValueError('Replay summary diverges')
        originals = set(p.name for p in (bundle / 'run/checkpoints').glob('*.save'))
        repeated = set(p.name for p in (output_run_path / 'checkpoints').glob('*.save'))
        if originals != repeated:
            raise ValueError('Replay checkpoint set diverges')
        for filename in (bundle / 'run/checkpoints').glob('*.save'):
            if sha(filename) != sha(output_run_path / 'checkpoints' / filename.name):
                raise ValueError('Replay world snapshot diverges: ' + filename.name)
    proof = {'verified': True, 'manifest_sha256': manifest_hash, 'verifier_sha256': source_hash,
             'model': record['verification_model'], 'original_death_not_new_sample': record['summary']['dead'],
             'native_archive_body_verified': True, 'native_death_log_wall_clock_differences': native_clock_differences}
    if observation_extension:
        proof['observation_extension'] = {'fields': sorted(OBSERVATION_FIELDS), 'engine_sha256': sha(Path(engine)), 'original_engine_sha256': sha(bundle / 'runtime/game/angband'), 'scope': 'historical-source observation-only rebuild; every original channel/value and native save matches'}
    else:
        write(cached, proof)
    if output is not None:
        write(output.resolve() / 'verification.json', proof)
    return proof


def first_difference(original, repeated):
    if original.suffix in ('.log', '.jsonl', '.txt', '.json'):
        with original.open('rb') as left, repeated.open('rb') as right:
            for row, (a, b) in enumerate(itertools.zip_longest(left, right), 1):
                if a != b:
                    detail = ''
                    if original.suffix == '.jsonl' and a and b:
                        try:
                            x, y = json.loads(a), json.loads(b)
                            fields = sorted(k for k in x.keys() | y.keys() if k not in x or k not in y or x[k] != y[k])
                            detail = f" tick={x.get('turn')} fields={fields}"
                        except (ValueError, AttributeError):
                            pass
                    return f'Replay diverges: {original.name} row={row}{detail}; original={a!r} repeated={b!r}'[:1500]
    return 'Replay diverges: ' + original.name


def compare_native(original, repeated):
    # Parent-owned native streams only. Auxiliary world scratch files can be
    # overwritten by the next forked anchor and are still compared at the end.
    before = {str(p.relative_to(original)): p for p in original.rglob('*') if p.is_file()}
    after = {str(p.relative_to(repeated)): p for p in repeated.rglob('*') if p.is_file()}
    if before.keys() != after.keys():
        raise ValueError('Replay native stream set diverges: ' + str(sorted(before.keys() ^ after.keys())))
    for name, path in before.items():
        if sha(path) != sha(after[name]):
            raise ValueError(first_difference(path, after[name]))


def validate_toolkit(bundle, record):
    import toolkit_native
    if record['toolkit'] != toolkit_native.VERSION:
        raise ValueError('Unsupported Rust toolkit extension')
    hashes = record.get('files_sha256', {})
    required = {'runtime/game/angband', 'runtime/harness/borg.txt', 'runtime/harness/run.sh',
        'runtime/harness/supervise.py', 'run/keys.log', 'run/messages.log', 'run/state.log',
        'run/summary.json', 'run/process.json', 'run/config.txt', 'run/final.save', 'frames.jsonl.gz'}
    required.update('run/native/' + name for name in ('events.txt', 'clock.txt', 'pid.txt', 'actions.txt',
        'frames.jsonl', 'file-inputs.txt', 'geometry.json', 'native-messages.jsonl', 'record-complete.json'))
    if record.get('finalized') or record.get('status') == 'complete':
        if not required <= hashes.keys():
            raise ValueError('Finalized Rust toolkit recording missing required files: ' + str(sorted(required - hashes.keys())))
        actual = {str(p.relative_to(bundle)) for p in bundle.rglob('*') if p.is_file()} - {'recording.json', 'verification.json'}
        if actual != hashes.keys():
            raise ValueError('Rust toolkit recording file set changed')
        native = bundle / 'run/native'
        footer = json.loads((native / 'record-complete.json').read_text())
        for field, name in (('inputs', 'events.txt'), ('frames', 'frames.jsonl')):
            with (native / name).open('rb') as stream:
                count = sum(1 for line in stream if line.endswith(b'\n'))
            if type(footer.get(field)) is not int or footer[field] != count:
                raise ValueError('Native footer count differs: ' + field)
        with gzip.open(bundle / 'frames.jsonl.gz', 'rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        if digest != sha(native / 'frames.jsonl'):
            raise ValueError('Compressed original movie differs from native frames')
        from borg_replay_viewer import Frames
        movie = Frames(native)
        movie.close()


def validate_binary(bundle, record):
    identity = record.get('build_id')
    if not identity:
        if record.get('profile') or record.get('frontend') or record.get('toolkit'):
            raise ValueError('Missing actual Rust build identity; no qualified replay claim')
        return  # Legacy minimal fixtures carry no qualified build claim.
    binary = bundle / 'runtime/game/angband'
    if sha(binary) != identity.get('binary_sha256'):
        raise ValueError('Recorded build binary differs from frozen executable')
    if subprocess.check_output([str(ROOT / 'harness/identity.sh'), 'binary-impl', str(binary)], text=True).strip() != 'rust':
        raise ValueError('Rust recording requires a Rust executable; C-only bundles are refused')
    if record.get('origin') is not None:
        origin = bundle / 'runtime/harness/saves/origin.save'
        if sha(origin) != record.get('original_named_sha256'):
            raise ValueError('Frozen origin save differs from named-start identity')
    elif record.get('schema') == 'borg-rs-gui-recording@1' and record.get('origin_sha256'):
        origin = bundle / 'initial-profile/save' / record['character']
        if sha(origin) != record['origin_sha256']:
            raise ValueError('Frozen GUI origin save differs from recorded start')
    frozen_identity = bundle / 'runtime/build-id.json'
    if frozen_identity.exists() and json.loads(frozen_identity.read_text()) != identity:
        raise ValueError('Recording build identity differs from immutable frozen build provenance')

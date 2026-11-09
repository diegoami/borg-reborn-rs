"""Source-pinned, verified headless replay -> independent human native save.

Capture is before a top-level command at/after the requested world turn. A
native save/load is a serialized counterfactual, not an uninterrupted replay
or a restoration of private Borg memory. Originals are never overwritten.
"""
import fcntl
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import tarfile
import tempfile
import time
from toolkit_common import serialized
import display_font

import borg_gui as gui
import borg_runtime as runtime
import borg_saves
import death_workflow as workflow

ROOT = runtime.ROOT
SCHEMA = 'borg-rs-human-branch@1'
HELPERS = ('human_takeover.py', 'human-takeover.inc', 'inspect-human-save.c')


@contextmanager
def owned_child_signals():
    def stop(signum, frame):
        raise InterruptedError('Human launcher interrupted')
    previous = signal.signal(signal.SIGTERM, stop)
    try:
        yield
    finally:
        signal.signal(signal.SIGTERM, previous)


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError('Historical takeover hook differs; refusing an unqualified rebuild')
    return text.replace(old, new)


def patch(source):
    path = source / 'angband/src/main-borgh.c'
    text = path.read_text()
    text = replace_once(text, 'static void borgh_checkpoint(void)\n{',
                        '#include "human-takeover.inc"\n\nstatic void borgh_checkpoint(void)\n{\n\thuman_capture();')
    if 'borgh_command_hook = borgh_checkpoint;' not in text:
        text = replace_once(text, 'borgh_command_hook = opt_checkpoint_dir ? borgh_checkpoint : NULL;',
                            'borgh_command_hook = (opt_checkpoint_dir || getenv("BORG_TAKEOVER_TURN")) ? borgh_checkpoint : NULL;')
    path.write_text(text)
    shutil.copy2(ROOT / 'tools/human-takeover.inc', source / 'angband/src/human-takeover.inc')
    native = source / 'angband/src/native-replay.c'
    with native.open('a') as stream:
        stream.write('\nbool human_private_snapshot(const char *path) { return snapshot(path); }\n')
        stream.write(GUI_CAPTURE)
    text = native.read_text()
    text = replace_once(text, 'void record_frame(const char *phase, long decisions)', 'static void human_gui_capture(const char *phase);\nvoid record_frame(const char *phase, long decisions)')
    text = replace_once(text, '    fflush(frames);\n}', '    fflush(frames);\n    human_gui_capture(phase);\n}')
    native.write_text(text)
    path = source / 'angband/src/main.c'
    path.write_text(replace_once(path.read_text(), '\tinit_angband();',
        '#ifdef BORG_HARNESS\n\t{ extern void borgh_human_mode_init(void); borgh_human_mode_init(); }\n#endif\n\tinit_angband();'))


def qualified_engine(bundle, record, timeout):
    pin = workflow.source_commit((record.get('collection_context') or {}).get('committed_source') or record['build_id'].get('git_rev'))
    if workflow.source_inputs(pin) != record['build_id']['source_sha256']:
        raise ValueError('Historical source does not match the recorded build; takeover cannot use newer gameplay')
    helpers = {name: runtime.sha(ROOT / 'tools' / name) for name in HELPERS}
    helpers['verifier'] = runtime.sha(Path(runtime.__file__))
    key = hashlib.sha256(json.dumps({'pin': pin, 'stamp': record['build_id'].get('build_stamp'), 'helpers': helpers}, sort_keys=True).encode()).hexdigest()
    base = workflow.artifacts() / 'human-builds' / key
    with workflow.locked(base.with_suffix('.lock')):
        base.mkdir(parents=True, exist_ok=True)
        complete = base / 'build.json'
        if not complete.exists():
            source = base / 'source'
            source.mkdir(exist_ok=False)
            archive = base / 'source.tar'
            subprocess.run(['git', '-C', str(ROOT), 'archive', '--format=tar', '-o', str(archive), pin,
                            'angband', 'borg-reborn-rs', 'borg-rs', 'Cargo.toml', 'Cargo.lock', 'harness/measurement'], check=True)
            with tarfile.open(archive) as stream:
                stream.extractall(source, filter='data')
            original = {str(p.relative_to(source)): runtime.sha(p) for p in source.rglob('*') if p.is_file()}
            if record.get('toolkit') or record.get('build_id', {}).get('overlay'):
                apply_original_overlay(bundle, source, record)
                original = {str(p.relative_to(source)): runtime.sha(p) for p in source.rglob('*') if p.is_file()}
            patch(source)
            changed = sorted(name for name, digest in original.items() if runtime.sha(source / name) != digest)
            if changed != ['angband/src/main-borgh.c', 'angband/src/main.c', 'angband/src/native-replay.c']:
                raise ValueError('Unexpected historical production source change')
            (source / 'angband/version').write_text(record['build_id'].get('build_stamp') or pin + '-human')
            directory = base / 'binary'
            env = {**os.environ, 'GIT_CEILING_DIRECTORIES': str(source), 'LC_ALL': 'C.UTF-8', 'LANG': 'C.UTF-8'}
            commands = [ ['cmake', '-S', str(source / 'angband'), '-B', str(directory), '-DCMAKE_BUILD_TYPE=Release',
                           '-DBORG_IMPL=rust', '-DSUPPORT_BORG=ON', '-DSUPPORT_BORG_HARNESS=ON', '-DBORG_RS_FEATURES=',
                           '-DSUPPORT_BORG_RECORDING=ON', '-DSUPPORT_X11_FRONTEND=ON', '-DSUPPORT_TEST_FRONTEND=ON',
                           '-DSC_INSTALL=ON', '-DBUILD_DOC=OFF'],
                         ['cmake', '--build', str(directory), '--target', 'OurExecutable', '--parallel', os.environ.get('JOBS', '4')] ]
            print('Building historical Rust image for human takeover:', pin, flush=True)
            for label, command in zip(('configure', 'build'), commands):
                with (base / (label + '.log')).open('w') as log:
                    subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=timeout)
            engine = directory / 'game/angband'
            if subprocess.check_output([str(ROOT / 'harness/identity.sh'), 'binary-impl', str(engine)], text=True).strip() != 'rust':
                raise ValueError('Historical image is not Rust')
            obj = base / 'inspect.o'
            subprocess.run(['cc', '-O2', '-c', str(ROOT / 'tools/inspect-human-save.c'), '-o', str(obj)], check=True)
            argv = [v for v in shlex.split((directory / 'CMakeFiles/OurExecutable.dir/link.txt').read_text())
                    if not v.startswith('-Wl,--dependency-file=')]
            reader = base / 'inspect-save'
            argv[argv.index('-o') + 1] = str(reader)
            subprocess.run([*argv, str(obj), '-Wl,--wrap=savefile_load'], cwd=directory, check=True)
            runtime.write(complete, {'source_commit': pin, 'source_sha256': record['build_id']['source_sha256'],
                'engine': str(engine), 'reader': str(reader), 'engine_sha256': runtime.sha(engine),
                'reader_sha256': runtime.sha(reader), 'helpers': helpers, 'changed_existing_files': changed,
                'original_files_sha256': original})
        build = json.loads(complete.read_text())
        engine = Path(build['engine'])
        if build['helpers'] != helpers or runtime.sha(engine) != build['engine_sha256'] or runtime.sha(Path(build['reader'])) != build['reader_sha256']:
            raise ValueError('Historical takeover build changed')
        control = base / ('control-' + runtime.sha(bundle / 'recording.json'))
        receipt = control / 'takeover-control.json'
        if not receipt.exists():
            print('Preparing strict prefix qualification for finalized timeout...' if record.get('status') == 'timeout' else 'Qualifying historical image against the entire original trajectory and saves...', flush=True)
            if record.get('status') == 'timeout':
                # No full-replay certificate for the incomplete original. The
                # actual selected prefix will still be compared before use.
                control.mkdir(parents=True, exist_ok=False)
                runtime.write(receipt, {'verified': False, 'eligible_for_prefix': True, 'scope': 'prefix verification pending; incomplete original',
                    'files_sha256': {}, 'manifest_sha256': runtime.sha(bundle / 'recording.json'), 'engine_sha256': runtime.sha(engine)})
                return build, receipt
            if record.get('schema') == gui.SCHEMA:
                gui.verify(bundle, output=control, timeout=timeout, engine=engine)
            else:
                runtime.verify(bundle, output=control, timeout=timeout, engine=engine, observation_extension=True)
            if runtime.sha(control / 'run/frames.jsonl') != runtime.sha(bundle / 'run/frames.jsonl'):
                raise ValueError('Takeover-disabled original raw frames diverge')
            hashes = {str(p.relative_to(control)): runtime.sha(p) for p in control.rglob('*') if p.is_file()}
            runtime.write(receipt, {'verified': True, 'files_sha256': hashes,
                'manifest_sha256': runtime.sha(bundle / 'recording.json'), 'engine_sha256': runtime.sha(engine)})
        proof = json.loads(receipt.read_text())
        if not (proof.get('verified') or record.get('status') == 'timeout' and proof.get('eligible_for_prefix')) or proof['engine_sha256'] != runtime.sha(engine) or proof['manifest_sha256'] != runtime.sha(bundle / 'recording.json'):
            raise ValueError('Takeover control identity changed')
        for name, digest in proof['files_sha256'].items():
            if runtime.sha(control / name) != digest:
                raise ValueError('Takeover control evidence changed: ' + name)
        return build, receipt


def check_prefix(bundle, output, capture):
    boundary = json.loads((capture / 'boundary.json').read_text())
    keys = boundary['keys_before']
    state = json.loads((capture / 'captured-state.json').read_text())
    if boundary['phase'] != 'before_game_command' or boundary['actual_turn'] < boundary['requested_turn'] or state['dead'] or state['turn'] != boundary['actual_turn']:
        raise ValueError('Unsafe or incorrectly selected capture boundary')
    for name in ('keys.log', 'state.log'):
        with (bundle / 'run' / name).open('rb') as before:
            expected = b''.join(before.readline() for _ in range(keys))
        if (output / name).read_bytes() != expected:
            raise ValueError('Takeover replay prefix diverges: ' + name)
    with (bundle / 'run/frames.jsonl').open('rb') as before, (output / 'frames.jsonl').open('rb') as after:
        for _ in range(keys):
            original_frame, captured_frame = before.readline(), after.readline()
            if not original_frame or original_frame != captured_frame:
                raise ValueError('Takeover replay frame prefix diverges')
        terminal_line = after.readline()
        if not terminal_line:
            raise ValueError('Takeover terminal frame missing')
        terminal = json.loads(terminal_line)
        if terminal['phase'] != 'human_takeover' or terminal['turn'] != state['turn'] or after.read():
            raise ValueError('Takeover terminal frame differs')
    original_messages = (bundle / 'run/messages.log').read_bytes()
    messages = (output / 'messages.log').read_bytes()
    # Messages stamped K may occur before or after the next decision. Every
    # earlier key's messages must exist, and any retained K messages must be
    # the exact original byte prefix (never invented ordering at one turn).
    required_bytes = sum(len(line) for line in original_messages.splitlines(keepends=True)
                         if int(line.split(None, 1)[0]) < keys)
    if len(messages) < required_bytes or not original_messages.startswith(messages):
        raise ValueError('Takeover replay message prefix diverges')
    index = bundle / 'run/checkpoints/index.jsonl'
    checkpoints = [json.loads(line) for line in index.read_text().splitlines()] if index.exists() else []
    expected_checkpoints = [row for row in checkpoints if row['game_turn'] < state['turn'] and row['keys'] <= keys]
    index = output / 'checkpoints/index.jsonl'
    observed = [json.loads(line) for line in index.read_text().splitlines()] if index.exists() else []
    if observed != expected_checkpoints or {p.name for p in (output / 'checkpoints').glob('*.save')} != {row['file'] for row in expected_checkpoints}:
        raise ValueError('Takeover prefix checkpoint set differs')
    for checkpoint in (output / 'checkpoints').glob('*.save'):
        if runtime.sha(checkpoint) != runtime.sha(bundle / 'run/checkpoints' / checkpoint.name):
            raise ValueError('Takeover prefix checkpoint diverges')
    if runtime.sha(capture / 'snapshot.save') != runtime.sha(output / 'final.save'):
        raise ValueError('Private capture and terminal save differ')
    native = bundle / 'run/native'
    if not native.exists():
        anchors = [row for row in checkpoints if row['game_turn'] == state['turn'] and row['keys'] == keys]
        if not anchors:
            raise ValueError('Legacy selected command has no original native-save anchor; strict takeover is unavailable at this boundary')
        if runtime.sha(capture / 'snapshot.save') != runtime.sha(bundle / 'run/checkpoints' / anchors[0]['file']):
            raise ValueError('Legacy selected save differs from the original checkpoint')
    if native.exists():
        for name in ('events.txt', 'clock.txt', 'actions.txt', 'frames.jsonl', 'file-inputs.txt', 'native-messages.jsonl', 'combat.jsonl'):
            if not (native / name).exists():
                continue
            observed = (output / 'native' / name).read_bytes()
            if not (native / name).read_bytes().startswith(observed):
                raise ValueError('Takeover native prefix diverges: ' + name)
        expected = None
        with (native / 'frames.jsonl').open() as stream:
            for line in stream:
                frame = json.loads(line)
                if frame['phase'] == 'command' and frame['turn'] == state['turn'] and frame['decisions'] == keys:
                    expected = frame
                    break
        if expected is None or expected.get('dead') or expected['hp'] <= 0:
            raise ValueError('Capture does not match an original living command')
        fingerprint = 14695981039346656037
        for byte in (capture / 'snapshot.save').read_bytes():
            fingerprint = ((fingerprint ^ byte) * 1099511628211) & 0xffffffffffffffff
        if f'{fingerprint:016x}' != expected['world']:
            raise ValueError('Captured native save fingerprint differs from original command')
        for path in (output / 'native').glob('file-input-*.bin'):
            if runtime.sha(path) != runtime.sha(native / path.name):
                raise ValueError('Takeover external-file prefix differs: ' + path.name)
    return boundary, state


def same_state(expected, actual):
    # The quick RNG is used for flavor initialization on load; complex game
    # RNG index/state and the serialized player fields are checked exactly.
    left = {k: v for k, v in expected.items() if k != 'borg_active'}
    right = {k: v for k, v in actual.items() if k != 'borg_active'}
    if left != right:
        fields = ', '.join(k for k in left.keys() | right.keys() if left.get(k) != right.get(k))
        raise ValueError('Loaded human state differs: ' + fields)
    if actual['borg_active'] or actual['dead']:
        raise ValueError('Human session must be living with Borg off')


def inspect(branch, save, timeout=30):
    with tempfile.TemporaryDirectory(prefix='borg-human-inspect-') as temporary:
        result = subprocess.run([str(branch / 'runtime/inspect-save'), '-mborgh',
            *[f'-d{name}={temporary}' for name in ('user','save','archive','panic','scores')],
            '-dgamedata=' + str(branch / 'runtime/lib/gamedata'), '--', '-seed', '1', '-race', '0', '-class', '0',
            '-turns', '2147483647', '-load', str(save)], cwd=branch / 'runtime', env=gui.environment(), capture_output=True, text=True, timeout=timeout)
        if result.returncode:
            raise ValueError('Cannot read living human save: ' + result.stderr[-500:])
        return json.loads(result.stdout)


@serialized
def _prepare(bundle, turn, output=None, timeout=600):
    bundle = bundle.resolve()
    # Fail unsupported origins before interpreting a headless manifest.
    record = json.loads((bundle / 'recording.json').read_text())
    if record.get('schema') != runtime.SCHEMA:
        raise ValueError('Human takeover currently supports headless recordings; GUI movies remain viewable')
    record = runtime.validate(bundle, require_complete=False)
    if record.get('status') != 'complete' and not (record.get('status') == 'timeout' and record.get('finalized')):
        raise ValueError('Only complete or finalized consistent timeout movies can qualify a prefix')
    if turn == 'latest':
        turn = latest_turn(bundle, record)
    if turn < 0 or turn > record['summary']['game_turns']:
        raise ValueError('Requested turn is outside the original trajectory')
    output = (output or runtime.data_root() / 'human-branches' / (workflow.now().replace(':', '-') + '-t' + str(turn))).resolve()
    if output.exists():
        raise ValueError('Human branch directory already exists; use play-branch.py to resume: ' + str(output))
    build, receipt = qualified_engine(bundle, record, timeout)
    output.mkdir(parents=True, exist_ok=False)
    capture = output / 'capture'
    capture.mkdir()
    try:
        print('Capturing first living command boundary at/after turn', turn, flush=True)
        result = runtime.frozen_run(bundle, output / 'prefix', timeout=timeout, engine=Path(build['engine']),
            takeover={'turn': turn, 'directory': capture})
        (output / 'capture-console.log').write_text(result.stdout)
        if result.returncode:
            raise ValueError('Takeover capture failed: ' + result.stdout[-1000:])
        if not (capture / 'boundary.json').exists():
            raise ValueError('No living top-level command boundary at/after this turn; choose an earlier turn')
        boundary, state = check_prefix(bundle, output / 'prefix', capture)
        frozen = output / 'runtime'
        frozen.mkdir()
        shutil.copy2(build['engine'], frozen / 'angband')
        shutil.copy2(build['reader'], frozen / 'inspect-save')
        shutil.copytree(bundle / 'runtime/game/lib', frozen / 'lib')
        profile = output / 'profile'
        for name in gui.PROFILE_DIRS:
            (profile / name).mkdir(parents=True)
        # Use the captured prefix's user files, never future/death lore.
        shutil.copytree(output / 'prefix/user', profile / 'user', dirs_exist_ok=True)
        character = 'HumanBorg'
        shutil.copy2(capture / 'snapshot.save', profile / 'save' / character)
        loaded = inspect(output, profile / 'save' / character)
        same_state(state, loaded)
        immutable = {str(p.relative_to(output)): runtime.sha(p) for folder in (frozen, capture) for p in folder.rglob('*') if p.is_file()}
        branch = {'schema': SCHEMA, 'status': 'ready', 'original_recording': str(bundle),
            'original_manifest_sha256': runtime.sha(bundle / 'recording.json'), 'boundary': boundary,
            'initial_state': loaded, 'character': character, 'seed': record['profile']['seed'],
            'build_id': {'git_rev': build['source_commit'], 'original_source_sha256': build['source_sha256'],
                         'takeover_overlay_sha256': build['helpers'], 'binary_sha256': build['engine_sha256']},
            'full_control_receipt': str(receipt), 'full_control_sha256': runtime.sha(receipt),
            'prefix_verified': True, 'original_complete': record.get('status') == 'complete',
            'files_sha256': immutable, 'independent_life': False,
            'scope': 'verified original prefix; native serialized engine snapshot; Borg off; private Borg memory and uninterrupted future equivalence not claimed'}
        runtime.write(output / 'human-branch.json', branch)
        print(f"Human save ready at turn {state['turn']}, depth {state['depth']}, position ({state['x']},{state['y']}), HP {state['hp']}, SP {state['sp']}.", flush=True)
        print('Save:', profile / 'save' / character, flush=True)
        print('Launch/resume:', ROOT / 'play-branch.py', output, flush=True)
        return output
    except BaseException as error:
        runtime.write(output / 'failed.json', {'error': str(error), 'usable': False})
        raise


def launch(branch):
    branch = branch.resolve()
    record = json.loads((branch / 'human-branch.json').read_text())
    if record.get('schema') != SCHEMA or record.get('status') != 'ready':
        raise ValueError('Not a prepared human branch')
    for name, digest in record['files_sha256'].items():
        path = (branch / name).resolve()
        if not path.is_relative_to(branch) or runtime.sha(path) != digest:
            raise ValueError('Human branch frozen input changed: ' + name)
    display_font.probe()
    profile = branch / 'profile'
    save = profile / 'save' / record['character']
    lock = profile / 'save/.locks' / (record['character'] + '.lock')
    lock.parent.mkdir(exist_ok=True)
    with lock.open('a') as held, owned_child_signals():
        try:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError('This human branch is already open') from error
        expected = inspect(branch, save)
        if expected['dead']:
            raise ValueError('This human character is dead; prepare a new branch from the original to try again')
        borg_saves.backup(save)
        context = {'kind': 'human_takeover', 'seed': record['seed'], 'original_recording': record['original_recording'],
                   'original_manifest_sha256': record['original_manifest_sha256'], 'boundary': record['boundary'],
                   'independent_life': False, 'scope': record['scope'], 'calendar': record.get('calendar')}

        def ready(child, output):
            path = output / 'human-ready.json'
            deadline = time.monotonic() + 30
            while not path.exists() or not path.read_text().endswith('\n'):
                if child.poll() is not None or time.monotonic() > deadline:
                    raise ValueError('Human GUI did not reach a safe command boundary; see ' + str(output / 'engine.log'))
                time.sleep(.05)
            actual = json.loads(path.read_text())
            same_state(expected, actual)
            (output / 'human-approved').write_text('validated player state and complex game RNG; Borg off\n')
            print(f"YOU control the game at turn {actual['turn']}. Ctrl-X saves and exits. Save: {save}", flush=True)

        os.environ.setdefault('ANGBAND_X11_FONT', 'fixed')
        os.environ.setdefault('ANGBAND_X11_FONT_0', os.environ.get('BORG_PLAY_FONT', '-adobe-courier-medium-r-normal--28-*-*-*-p-*-iso8859-1'))
        return gui.start(branch / 'runtime/angband', profile, record['character'], [], lock_fd=held.fileno(),
            data_source=branch / 'runtime/lib', build_identity=record['build_id'], session_context=context, on_start=ready)


def latest_turn(bundle, record):
    native = bundle / ('run/frames.jsonl' if record.get('schema') == gui.SCHEMA else 'run/native/frames.jsonl')
    if native.exists():
        last = None
        with native.open() as stream:
            for line in stream:
                frame = json.loads(line)
                if frame['phase'] == 'command' and not frame.get('dead') and frame.get('hp', 0) > 0 :
                    last = frame['turn']
        if last is None:
            raise ValueError('No recorded living command boundary')
        return last
    # Legacy headless recordings lack command frames. Their native capture
    # rounds forward; never choose a terminal death key as a living command.
    rows = (bundle / 'run/keys.log').read_text().splitlines()
    turns = sorted({int(row.split()[0]) for row in rows if int(row.split()[0]) < record['summary']['game_turns']})
    if not turns:
        raise ValueError('Legacy movie has no qualified living command tick')
    return turns[-1]


BUILD_POLICY = 'ask'


def apply_original_overlay(bundle, source, record):
    from types import ModuleType
    frozen = bundle / 'runtime/tool-source'
    if not frozen.exists():
        raise ValueError('Missing frozen observer sources; restore a verified original archive')
    expected = record['build_id']['overlay_sha256']
    for name, digest in expected.items():
        if runtime.sha(frozen / name) != digest:
            raise ValueError('Original observer source hash differs: ' + name)
    path = frozen / 'tools/toolkit_native.py'
    module = ModuleType('frozen_rust_toolkit_overlay')
    module.__file__ = str(path)
    exec(compile(path.read_bytes(), str(path), 'exec'), module.__dict__)
    module.ROOT = frozen
    module.apply(source)


def prepare(bundle, turn, output=None, timeout=600):
    # Offer outside the native lock. Permission is for the separate helper only.
    if output is not None and Path(output).exists():
        raise ValueError('Human branch directory already exists; resume the prepared branch or choose a new output')
    bundle = Path(bundle).resolve()
    record = json.loads((bundle / 'recording.json').read_text())
    if record.get('schema') not in (runtime.SCHEMA, gui.SCHEMA):
        raise ValueError('Unsupported Rust recording schema for takeover')
    if record.get('schema') == gui.SCHEMA and not record.get('build_id', {}).get('overlay'):
        raise ValueError('Legacy GUI takeover requires its missing frozen observer/build sources; original movie remains viewable')
    backend = gui if record['schema'] == gui.SCHEMA else runtime
    record = backend.validate(bundle, require_complete=False)
    if record.get('status') != 'complete' and not (record.get('schema') == runtime.SCHEMA and record.get('status') == 'timeout' and record.get('finalized')):
        raise ValueError('Active, crash, or unfinalized recordings cannot qualify a human prefix')
    if turn != 'latest' and (not isinstance(turn, int) or turn < 0):
        raise ValueError('Use a nonnegative tick or latest')
    helpers = {name: runtime.sha(ROOT / 'tools' / name) for name in HELPERS}
    helpers['verifier'] = runtime.sha(Path(runtime.__file__))
    pin = workflow.source_commit((record.get('collection_context') or {}).get('committed_source') or record['build_id'].get('git_rev'))
    key = hashlib.sha256(json.dumps({'pin': pin, 'stamp': record['build_id'].get('build_stamp'), 'helpers': helpers}, sort_keys=True).encode()).hexdigest()
    path = workflow.artifacts() / 'human-builds' / key / 'build.json'
    if not path.exists():
        print('A separate historical Rust takeover helper is required:', pin, flush=True)
        if BUILD_POLICY == 'no-build':
            raise ValueError('--no-build: historical helper is unavailable')
        if BUILD_POLICY != 'yes':
            import sys
            if not sys.stdin.isatty() or input('Build this historical helper? [y/N] ').strip().lower() not in ('y', 'yes'):
                raise ValueError('Historical helper build not accepted; use --yes to accept')
    if record.get('schema') == gui.SCHEMA:
        return _prepare_gui(bundle, turn, output=output, timeout=timeout)
    return _prepare(bundle, turn, output=output, timeout=timeout)


GUI_CAPTURE = r'''
static void human_gui_capture(const char *phase)
{
    const char *requested = getenv("BORG_TAKEOVER_TURN");
    const char *target = getenv("BORG_TAKEOVER_DIR");
    char path[2048]; FILE *fp;
    extern void borgh_human_dump(FILE *);
    if (!getenv("BORG_TAKEOVER_GUI") || !requested || !target || !streq(phase, "command") ||
        !character_dungeon || !player || player->is_dead || player->chp <= 0 ||
        player->upkeep->generate_level || turn < strtol(requested, NULL, 10)) return;
    strnfmt(path, sizeof(path), "%s/captured-state.json", target);
    fp = fopen(path, "w"); if (!fp) quit("GUI capture state failed");
    borgh_human_dump(fp); if (fclose(fp)) quit("GUI capture state close failed");
    strnfmt(path, sizeof(path), "%s/boundary.json", target);
    fp = fopen(path, "w"); if (!fp) quit("GUI capture boundary failed");
    fprintf(fp, "{\"requested_turn\":%ld,\"actual_turn\":%ld,\"keys_before\":%ld,\"frame\":%llu,\"phase\":\"before_game_command\"}\n",
        strtol(requested, NULL, 10), (long)turn, record_key_count(), frame_count);
    if (fclose(fp)) quit("GUI capture boundary close failed");
    strnfmt(path, sizeof(path), "%s/snapshot.save", target);
    if (file_exists(path) || !snapshot(path)) quit("GUI private snapshot failed");
    _exit(0);
}
'''


@serialized
def _prepare_gui(bundle, turn, output=None, timeout=600):
    record = gui.validate(bundle)
    if turn == 'latest':
        turn = latest_turn(bundle, record)
    if turn < 0:
        raise ValueError('Requested turn is outside the original movie')
    from borg_replay_viewer import Frames
    frames = Frames(bundle)
    last = frames.get(len(frames.index)-1)[0]['turn']; frames.close()
    if turn > last:
        raise ValueError('Requested turn is outside the original movie')
    build, receipt = qualified_engine(bundle, record, timeout)
    output = Path(output or runtime.data_root() / 'human-branches' / (workflow.now().replace(':', '-')+'-gui-t'+str(turn))).resolve()
    output.mkdir(parents=True, exist_ok=False)
    capture = output / 'capture'; capture.mkdir()
    try:
        profile = output / 'reconstruction-profile'; shutil.copytree(bundle / 'initial-profile', profile)
        run = output / 'prefix'; run.mkdir()
        geometry = json.loads((bundle / 'run/geometry.json').read_text())
        env = gui.environment()
        env['TZ'] = ':' + str(bundle / 'runtime/timezone') if record['timezone'] == 'frozen-file' else record['timezone']
        env.update(BORG_RECORD_DIR=str(run), BORG_REPLAY_DIR=str(bundle / 'run'), BORG_REPLAY_SYS=record['frontend'],
            BORG_REPLAY_WIDTH=str(geometry['width']), BORG_REPLAY_HEIGHT=str(geometry['height']),
            BORG_REPLAY_KEY_SIZE=str(geometry['key_size']), BORG_REPLAY_SOFT_CURSOR=str(geometry['soft_cursor']),
            BORG_REPLAY_COMPLEX_INPUT=str(geometry['complex_input']), BORG_TAKEOVER_GUI='1',
            BORG_TAKEOVER_TURN=str(turn), BORG_TAKEOVER_DIR=str(capture))
        gui.human_environment(record, env, run)
        args = record['arguments']; args = args[:args.index('--')] if '--' in args else args
        frozen = bundle / 'runtime/game'
        result = subprocess.run(gui.command(Path(build['engine']), frozen / 'lib', profile, record['character'], args, 'test'),
            cwd=frozen, env=env, capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL)
        (output / 'capture-console.log').write_text(result.stdout+result.stderr)
        if result.returncode or not (capture / 'boundary.json').exists():
            raise ValueError('No qualified living GUI command; capture diagnostics retained')
        boundary = json.loads((capture / 'boundary.json').read_text())
        state = json.loads((capture / 'captured-state.json').read_text())
        for name in ('events.txt', 'clock.txt', 'actions.txt', 'frames.jsonl', 'file-inputs.txt', 'native-messages.jsonl', 'combat.jsonl'):
            original, actual = bundle / 'run' / name, run / name
            if original.exists() and (not actual.exists() or not original.read_bytes().startswith(actual.read_bytes())):
                raise ValueError('GUI takeover original prefix diverges: ' + name)
        last_frame = json.loads((run / 'frames.jsonl').read_text().splitlines()[-1])
        if last_frame['index'] != boundary['frame'] or last_frame['phase'] != 'command' or state['dead']:
            raise ValueError('GUI capture command/frame differs')
        fingerprint = 14695981039346656037
        for byte in (capture / 'snapshot.save').read_bytes():
            fingerprint = ((fingerprint ^ byte)*1099511628211)&0xffffffffffffffff
        if f'{fingerprint:016x}' != last_frame['world']:
            raise ValueError('GUI snapshot differs from original command fingerprint')
        for path in run.glob('file-input-*.bin'):
            if runtime.sha(path) != runtime.sha(bundle / 'run' / path.name):
                raise ValueError('GUI file-read prefix differs')
        frozen = output / 'runtime'; frozen.mkdir()
        shutil.copy2(build['engine'], frozen / 'angband'); shutil.copy2(build['reader'], frozen / 'inspect-save')
        shutil.copytree(bundle / 'runtime/game/lib', frozen / 'lib')
        profile = output / 'profile'
        for name in gui.PROFILE_DIRS: (profile / name).mkdir(parents=True)
        shutil.copytree(output / 'reconstruction-profile/user', profile / 'user', dirs_exist_ok=True)
        character = 'HumanBorg'; shutil.copy2(capture / 'snapshot.save', profile / 'save' / character)
        loaded = inspect(output, profile / 'save' / character); same_state(state, loaded)
        immutable = {str(p.relative_to(output)): runtime.sha(p) for folder in (frozen, capture) for p in folder.rglob('*') if p.is_file()}
        branch = {'schema':SCHEMA, 'status':'ready', 'original_recording':str(bundle),
            'original_manifest_sha256':runtime.sha(bundle / 'recording.json'), 'boundary':boundary,
            'initial_state':loaded, 'character':character, 'seed':record.get('session_context',{}).get('visible_seed') or 1,
            'calendar':'original-gui', 'build_id':{'git_rev':build['source_commit'],
                'original_source_sha256':build['source_sha256'], 'binary_sha256':build['engine_sha256'],
                'takeover_overlay_sha256':build['helpers']}, 'files_sha256':immutable,
            'full_control_receipt':str(receipt), 'full_control_sha256':runtime.sha(receipt),
            'independent_life':False, 'scope':'verified GUI original prefix; serialized native save; no private Borg memory restoration'}
        runtime.write(output / 'human-branch.json',branch)
        print('GUI human save ready at tick',state['turn'],'; resume:',ROOT / 'play-branch.py',output,flush=True)
        return output
    except BaseException as error:
        runtime.write(output / 'failed.json',{'error':str(error),'usable':False}); raise

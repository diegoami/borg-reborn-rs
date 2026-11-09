"""Qualified player-known observations for immutable historical Rust recordings.

Adapted from better-borg-c's replay-inventory.py workflow; preserve originals,
rebuild historical algorithms, add only declared observer fields, compare all
old channels/saves. The extension is never a new independent death or a loaded
checkpoint continuation. Source/paths/hashes are recorded; see IMPORTED.json.
"""
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile

import borg_runtime as runtime
import death_workflow as workflow

ROOT = runtime.ROOT
HEADERS = ('replay-observations.h', 'replay-native-inventory.h', 'replay-native-spells.h')


class ObservationUnavailable(ValueError):
    """Original movie is valid, but extra native views cannot be qualified."""



def first_frame(bundle):
    with gzip.open(bundle / 'frames.jsonl.gz', 'rt') as stream:
        return json.loads(next(stream))


def caster(bundle):
    summary = bundle / 'run/summary.json'
    if not summary.exists():
        return None
    name = json.loads(summary.read_text()).get('class')
    data = (bundle / 'runtime/game/lib/gamedata/class.txt').read_text()
    for block in ('\n' + data).split('\nname:')[1:]:
        lines = block.splitlines()
        if lines[0] == name:
            return any(line.startswith('spell:') for line in lines)
    return None


def needed(bundle):
    frame = first_frame(bundle)
    return 'inventory' not in frame or ('spells' not in frame and caster(bundle) is not False)


def patch_headless(source):
    """Unique historical sites; no replacement of a Borg/game algorithm."""
    path = source / 'angband/src/main-borgh.c'
    text = path.read_text()
    if '#include "replay-observations.h"' in text:
        raise ValueError('Historical source already contains these observers')
    start = '/* Observation only. Compatible terminal-frame schema'
    end = '\tfputs("}\\n", frames_fp);\n}\n\n/**\n * Write the summary'
    if text.count(start) != 1 or text.count(end) != 1:
        raise ValueError('Historical recorder extension site differs; refuse an unqualified rebuild')
    text = text.replace(start, '#define REPLAY_OBSERVATION_STREAM frames_fp\n#define REPLAY_ADD_VITALS\n#include "replay-observations.h"\n#undef REPLAY_ADD_VITALS\n#undef REPLAY_OBSERVATION_STREAM\n\n' + start)
    text = text.replace(end, '\treplay_extra_observations(full);\n' + end)
    path.write_text(text)
    for name in HEADERS:
        shutil.copy2(ROOT / 'angband/src' / name, source / 'angband/src' / name)


def cache_identity(bundle):
    manifest = runtime.sha(bundle / 'recording.json')
    helpers = {name: runtime.sha(ROOT / 'angband/src' / name) for name in HEADERS}
    helpers['adapter'] = runtime.sha(Path(__file__))
    helpers['verifier'] = runtime.sha(Path(runtime.__file__))
    key = hashlib.sha256(json.dumps({'manifest': manifest, 'helpers': helpers}, sort_keys=True).encode()).hexdigest()
    return workflow.artifacts() / 'observations' / key, manifest, helpers


def cached(bundle):
    base, original_sha, helpers = cache_identity(bundle)
    receipt = base / 'qualified.json'
    if not receipt.exists():
        return None
    value = workflow.read(receipt)
    output = Path(value['output'])
    if value['original_manifest_sha256'] != original_sha or value['helpers'] != helpers:
        raise ValueError('Historical observation cache identity differs')
    if runtime.sha(output / 'run/frames.jsonl') != value['frames_sha256'] or runtime.sha(output / 'verification.json') != value['verification_sha256'] or not workflow.read(output / 'verification.json').get('verified'):
        raise ValueError('Qualified observation cache changed; preserve it and investigate')
    return output / 'run'


def prepare(bundle, timeout=600):
    bundle = bundle.resolve()
    record = runtime.validate(bundle)
    if record['schema'] != runtime.SCHEMA:
        raise ObservationUnavailable('Historical GUI source is not qualified for this adapter; original view remains available')
    context = record.get('collection_context') or {}
    pin = context.get('committed_source') or record['build_id'].get('git_rev')
    pin = workflow.source_commit(pin)
    if workflow.source_inputs(pin) != record['build_id']['source_sha256']:
        raise ObservationUnavailable('Historical source/dirty-build inputs cannot be reconstructed from this pin; original view remains available')
    base, original_sha, helpers = cache_identity(bundle)
    with workflow.locked(base.with_suffix('.lock')):
        existing = cached(bundle)
        if existing is not None:
            return existing
        base.mkdir(parents=True, exist_ok=True)
        build = base / 'build'
        complete = build / 'complete.json'
        if not complete.exists():
            source = build / 'source'
            source.mkdir(parents=True, exist_ok=False)
            archive = build / 'historical-source.tar'
            subprocess.run(['git', '-C', str(ROOT), 'archive', '--format=tar', '-o', str(archive), pin,
                            'angband', 'borg-reborn-rs', 'borg-rs', 'Cargo.toml', 'Cargo.lock', 'harness/measurement'], check=True)
            with tarfile.open(archive) as stream:
                stream.extractall(source, filter='data')
            original_files = {str(p.relative_to(source)): runtime.sha(p) for p in source.rglob('*') if p.is_file()}
            patch_headless(source)
            changed = [name for name, digest in original_files.items() if runtime.sha(source / name) != digest]
            if changed != ['angband/src/main-borgh.c']:
                raise ValueError('Historical production source changed outside the declared recorder site')
            stamp = record['build_id'].get('build_stamp', pin + '-observation-extension')
            (source / 'angband/version').write_text(stamp)
            directory = build / 'binary'
            env = {**os.environ, 'GIT_CEILING_DIRECTORIES': str(source), 'LC_ALL': 'C.UTF-8', 'LANG': 'C.UTF-8'}
            commands = [ ['cmake', '-S', str(source / 'angband'), '-B', str(directory), '-DCMAKE_BUILD_TYPE=Release',
                           '-DBORG_IMPL=rust', '-DSUPPORT_BORG=ON', '-DSUPPORT_BORG_HARNESS=ON', '-DBORG_RS_FEATURES=',
                           '-DCMAKE_EXPORT_COMPILE_COMMANDS=ON', '-DSC_INSTALL=ON', '-DBUILD_DOC=OFF'],
                         ['cmake', '--build', str(directory), '--target', 'OurExecutable', '--parallel', os.environ.get('JOBS', '4')] ]
            print('Rebuilding historical Rust source with player-known observations:', pin, flush=True)
            for label, command in zip(('configure', 'build'), commands):
                with (build / (label + '.log')).open('w') as log:
                    subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
            engine = directory / 'game/angband'
            if subprocess.check_output([str(ROOT / 'harness/identity.sh'), 'binary-impl', str(engine)], text=True).strip() != 'rust':
                raise ValueError('Historical observation image is not Rust')
            workflow.write(complete, {'source_commit': pin, 'original_source_sha256': record['build_id']['source_sha256'],
                                      'observer_headers': helpers, 'changed_existing_files': changed,
                                      'engine': str(engine), 'engine_sha256': runtime.sha(engine),
                                      'original_files_sha256': original_files})
        metadata = workflow.read(complete)
        engine = Path(metadata['engine'])
        if runtime.sha(engine) != metadata['engine_sha256'] or metadata['observer_headers'] != helpers:
            raise ValueError('Historical observer build changed')
        output = base / ('run-' + workflow.now().replace(':', '-'))
        proof = runtime.verify(bundle, output=output, timeout=timeout, engine=engine, observation_extension=True)
        # A second same-input run with observers disabled demonstrates that
        # inventory descriptions/spell queries did not mutate knowledge/RNG.
        control = output / 'observer-off'
        result = runtime.frozen_run(bundle, control, timeout=timeout, engine=engine, extra_observations=False)
        (output / 'observer-off-console.log').write_text(result.stdout)
        if result.returncode:
            raise ValueError('Observer-disabled control failed; evidence retained: ' + str(output))
        for name in ('keys.log', 'messages.log', 'state.log', 'frames.jsonl', 'final.save'):
            if runtime.sha(control / name) != runtime.sha(bundle / 'run' / name):
                raise ValueError('Observer-disabled control diverges: ' + name)
        proof['observer_off_original_channels_match'] = True
        workflow.write(output / 'verification.json', proof)
        workflow.write(base / 'qualified.json', {'output': str(output), 'original_manifest_sha256': original_sha,
            'helpers': helpers, 'frames_sha256': runtime.sha(output / 'run/frames.jsonl'),
            'verification_sha256': runtime.sha(output / 'verification.json'),
            'scope': 'historical source; all original channels/values and native saves match; declared read-only fields added separately'})
        return output / 'run'

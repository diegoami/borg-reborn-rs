"""Rust adaptation of better-borg-c's curated death records and aggregation.

Pinned donor f3ced6219768; see IMPORTED.json. GPL-2.0-or-later or Angband
licence (angband/docs/copying.rst). Native C measurement-specific readers are
replaced with validated Rust GUI/headless recording readers.
"""
from contextlib import contextmanager
from collections import Counter, defaultdict
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile

import borg_runtime as runtime
import borg_replay_viewer as movie

ROOT = runtime.ROOT
SCHEMA = 'borg-rs/death-record@1'
OFFSETS = (5, 12, 25, 40, 60, 82, 105)


def read(path):
    return json.loads(path.read_text())


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def atomic_json(path, value):
    temp = path.with_name('.' + path.name + '.tmp')
    runtime.write(temp, value)
    os.replace(temp, path)


def registry_root():
    return Path(os.environ.get('BORG_DEATH_RECORDS_DIR', ROOT / 'docs/deaths')).resolve()


def discussion_issue():
    explicit = os.environ.get('BORG_DEATH_ISSUE')
    if explicit:
        return explicit
    path = registry_root() / 'discussion.json'
    return read(path).get('issue') if path.exists() else None


@contextmanager
def registry_lock(root):
    root.mkdir(parents=True, exist_ok=True)
    with (root / '.registry.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def family(seed, race, cls, config, origin):
    return 'start-' + fingerprint({'seed': None if origin is not None else seed, 'race': race, 'class': cls, 'config': config, 'origin': origin})


def canonical_identity(record):
    """Compare legacy continuation dossiers without rewriting their identity."""
    identity = dict(record['identity'])
    if record.get('recording_schema') == runtime.SCHEMA and identity.get('start_save') is not None:
        terminal = record['terminal']
        identity['seed'] = None
        identity['encounter_family'] = family(None, terminal['race_idx'], terminal['class_idx'],
                                               identity['config_sha256'], identity['start_save'])
    return identity


def describe(bundle):
    """Validate retained evidence; do not infer a death from a process exit."""
    bundle = bundle.resolve()
    raw = read(bundle / 'recording.json')
    if raw.get('independent_life') is False or raw.get('session_context', {}).get('kind') == 'human_takeover':
        return None
    if raw.get('schema') == runtime.SCHEMA:
        raw = runtime.validate(bundle)
        summary = raw['summary']
        if read(bundle / 'run/summary.json') != summary:
            raise ValueError('Recording terminal summary differs')
        if not summary['dead'] or summary['stop_reason'] != 'death':
            return None
        config = bundle / 'run/user/borg.txt'
        config_hash = runtime.sha(config)
        if summary.get('cheat_live') or re.search(r'^\s*borg_cheat_death\s*=\s*(?:TRUE|1)\s*$', config.read_text(), re.M | re.I):
            raise ValueError('Cheat-enabled deaths are diagnostic, not natural death records')
        terminal = {'turn': summary['game_turns'], 'level': summary['clevel'], 'depth': summary['depth'],
                    'dead': True, 'cause': summary['death_cause'], 'race_idx': summary.get('played_race_idx', summary['race_idx']), 'class_idx': summary.get('played_class_idx', summary['class_idx'])}
        origin = raw.get('original_named_sha256') if raw.get('origin') is not None else None
        seed = summary['seed'] if origin is None else None
        run_kind = raw.get('run_kind', 'unknown')
        context = family(seed, terminal['race_idx'], terminal['class_idx'], config_hash, origin)
        build = raw['build_id']
        limits = raw['profile']
    else:
        import borg_gui
        raw = borg_gui.validate(bundle)
        viewer = movie.Frames(bundle)
        try:
            terminal = viewer.get(len(viewer.index) - 1)[0]
            first = viewer.get(0)[0]
        finally:
            viewer.close()
        if not terminal.get('dead'):
            return None
        if not terminal.get('cause'):
            raise ValueError('This old GUI recording lacks native cause data; do not invent a death cause')
        if terminal.get('cheat_live'):
            raise ValueError('Cheat-live death is not eligible for natural death export')
        terminal = {k: terminal[k] for k in ('turn', 'level', 'depth', 'dead', 'cause', 'race_idx', 'class_idx')}
        seed = None
        config = bundle / 'initial-profile/user/borg.txt'
        config_hash = runtime.sha(config) if config.exists() else None
        origin = raw.get('origin_sha256')
        run_kind = 'interactive GUI; recorded human inputs'
        # GUI calendar/PID and human inputs can change the world. Keep them in
        # the starting-condition family rather than conflating two GUI lives.
        context = 'start-' + fingerprint({'origin': origin, 'first_world': first.get('world'),
                    'rng': first.get('rng'), 'clock': runtime.sha(bundle / 'run/clock.txt'),
                    'events': runtime.sha(bundle / 'run/events.txt'), 'config': config_hash})
        build = raw.get('build_id', {'binary_sha256': runtime.sha(bundle / 'runtime/game/angband'), 'git_rev': 'not captured'})
        limits = {'duration_seconds': raw['duration_seconds'], 'frontend': raw['frontend']}
    if runtime.sha(bundle / 'runtime/game/angband') != build['binary_sha256']:
        raise ValueError('Recorded build binary differs from actual frozen executable')
    if terminal['cause'] in ('Retiring', 'Quitting', 'Interrupting') or not terminal['cause']:
        return None
    identity = {'source': build.get('git_rev', 'unknown'), 'source_sha256': build.get('source_sha256'),
                'binary': build['binary_sha256'], 'seed': seed, 'start_save': origin,
                'config_sha256': config_hash, 'turn': terminal['turn'], 'cause': terminal['cause'],
                'encounter_family': context}
    return {'identity': identity, 'encounter_family': context, 'raw_cause': terminal['cause'],
            'terminal': terminal, 'build': build, 'limits': limits, 'character': raw['character'],
            'run_kind': run_kind, 'calendar_context': raw.get('calendar', raw.get('timezone')),
            'recording_schema': raw['schema'], 'collection_context': raw.get('collection_context'), 'original': raw}


def recorded_windows(bundle, out, death_turn):
    """Select actual original frames at exact requested turns; never interpolate."""
    viewer = movie.Frames(bundle)
    selected, unavailable, anchors = [], [], []
    anchor_frames = {}
    try:
        for offset in OFFSETS:
            wanted = death_turn - offset
            frame, chars, attrs, palette = viewer.get(viewer.at_turn(wanted))
            exact = frame['turn'] == wanted
            if not exact:
                unavailable.append({'offset_engine_turns': offset, 'requested_turn': wanted,
                                    'reason': 'No original frame at this exact engine turn'})
                if frame['turn'] > wanted or frame['turn'] >= death_turn:
                    continue
                if frame['index'] in anchor_frames:
                    anchor_frames[frame['index']]['requested_offsets_engine_turns'].append(offset)
                    continue
                prefix = f'snapshots/recorded-anchor-t{frame["turn"]}'
            else:
                prefix = f'snapshots/recorded-T-minus-{offset}'
            text = f"Original frame {frame['index']}; engine turn {frame['turn']}; phase {frame['phase']}; HP {frame['hp']}, SP {frame['sp']}, depth {frame['depth']}\n"
            text += 'Logical cells; diagnostics are actual engine state, not extra Borg knowledge.\n\n'
            text += '\n'.join(''.join(movie.glyph(c) for c in chars[row * frame['width']:(row + 1) * frame['width']])
                              for row in range(frame['height'])) + '\n'
            (out / (prefix + '.txt')).write_text(text)
            payload = dict(frame, reconstructed_chars=chars, reconstructed_attrs=attrs, reconstructed_palette=palette)
            runtime.write(out / (prefix + '.json'), payload)
            item = {'offset_engine_turns': death_turn - frame['turn'], 'turn': frame['turn'], 'phase': frame['phase'],
                    'frame_index': frame['index'], 'text': prefix + '.txt', 'json': prefix + '.json'}
            if exact:
                selected.append(item)
            else:
                item.update(requested_offsets_engine_turns=[offset], qualification='Nearest earlier actual observation; not an exact requested offset or interpolated state')
                anchors.append(item)
                anchor_frames[frame['index']] = item
        frame, chars, _, _ = viewer.get(len(viewer.index) - 1)
        (out / 'terminal.map.txt').write_text('\n'.join(''.join(movie.glyph(c) for c in chars[row * frame['width']:(row + 1) * frame['width']])
                                                        for row in range(frame['height'])) + '\n')
    finally:
        viewer.close()
    return selected, unavailable, anchors


def write_dossier(bundle, output, case, issue=None, repeat=()):
    bundle = bundle.resolve()
    description = describe(bundle)
    if description is None:
        raise ValueError('A confirmed natural death is required; caps, failures and retirement are not deaths')
    if not re.fullmatch(r'D[0-9]{4,}', case):
        raise ValueError('Death id must be D followed by at least four digits')
    prior = read(output / 'record.json') if output.exists() else None
    if prior:
        if canonical_identity(prior) != description['identity'] or prior['id'] != case:
            raise ValueError('Existing death identity is immutable; use a new dossier')
        for name, digest in prior.get('files_sha256', {}).items():
            path = (output / name).resolve()
            if not path.is_relative_to(output.resolve()) or runtime.sha(path) != digest:
                raise ValueError('Existing death evidence changed: ' + name)
        if (prior.get('recording') or {}).get('manifest_sha256') != runtime.sha(bundle / 'recording.json'):
            raise ValueError('Preserve the original recording; add another attempt with --repeat')
    attempts = []
    for role, path in [('original', bundle), *[('repeat', p.resolve()) for p in repeat]]:
        desc = describe(path)
        if not desc or desc['identity'] != description['identity']:
            raise ValueError('Repeat has a different build, start or terminal identity')
        if role == 'repeat':
            channels = ('frames.jsonl', 'final.save', 'keys.log', 'messages.log', 'state.log') if description['recording_schema'] == runtime.SCHEMA else ('frames.jsonl', 'final.save', 'events.txt', 'actions.txt', 'clock.txt', 'file-inputs.txt')
            for name in channels:
                original_file = bundle / 'run' / name if (bundle / 'run' / name).exists() else bundle / name
                repeated_file = path / 'run' / name if (path / 'run' / name).exists() else path / name
                if runtime.sha(original_file) != runtime.sha(repeated_file):
                    raise ValueError('Unchanged repeat diverged: ' + name)
        attempts.append({'role': role, 'outcome': 'death', 'raw_directory': str(path),
                         'source_commit': desc['build'].get('git_rev', 'unknown'),
                         'binary_sha256': desc['build']['binary_sha256'],
                         'result_sha256': runtime.sha(path / 'recording.json'),
                         'requested_seed': desc['original'].get('profile', {}).get('seed'),
                         'seed_role': 'recorded GUI inputs' if desc['recording_schema'] != runtime.SCHEMA else 'ignored on load; saved engine RNG' if desc['identity']['start_save'] is not None else 'birth input'})
    if prior:
        known = {attempt['result_sha256'] for attempt in prior['attempts']}
        for attempt in attempts:
            if attempt['result_sha256'] not in known:
                prior['attempts'].append(attempt)
                known.add(attempt['result_sha256'])
        if issue:
            prior.update(issue=issue, investigation=issue)
        atomic_json(output / 'record.json', prior)
        return prior
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.death-partial-', dir=output.parent) as temporary:
        staging = Path(temporary) / 'case'
        (staging / 'snapshots').mkdir(parents=True)
        selected, unavailable, anchors = recorded_windows(bundle, staging, description['terminal']['turn'])
        runtime.write(staging / 'build.json', description['build'])
        runtime.write(staging / 'input.json', {'profile': description['limits'], 'run_kind': description['run_kind'],
                      'calendar': description['calendar_context'], 'identity': description['identity']})
        for name in ('messages.log', 'state.log', 'keys.log', 'actions.txt'):
            source = bundle / 'run' / name
            if source.exists():
                # Bounded output; full original streams remain in the bundle.
                from collections import deque
                with source.open(errors='replace') as stream:
                    tail = list(deque(stream, maxlen=250))
                (staging / name).write_text(''.join(tail))
        for source in (bundle / 'run/archive').glob('*'):
            if source.is_file() and (source.suffix == '.map' or source.name in ('borg-log.txt', 'borg.dat')):
                target = staging / 'native' / source.name
                target.parent.mkdir(exist_ok=True)
                shutil.copy2(source, target)
        character_output = bundle / 'run/user/CharOutput.txt'
        if character_output.is_file():
            shutil.copy2(character_output, staging / 'CharOutput.txt')
        checkpoints = bundle / 'run/checkpoints.json'
        living = [{**row, 'load_validation': 'not attempted by exporter; original fork-save hash retained'} for row in read(checkpoints)['captures']] if checkpoints.exists() else []
        record = {k: v for k, v in description.items() if k != 'original'}
        record.update(schema=SCHEMA, id=case, attempts=attempts, issue=issue, investigation=issue,
                      living_checkpoints=living,
                      independence_note='Curated case; repeats, continuations, GUI intervention and backup branches are not an independent mortality cohort',
                      recording={'schema': description['recording_schema'], 'raw_directory': str(bundle),
                                 'manifest_sha256': runtime.sha(bundle / 'recording.json'), 'frames_sha256': runtime.sha(bundle / 'frames.jsonl.gz'),
                                 'selected_original_frames': selected, 'unavailable_offsets': unavailable, 'nearby_original_anchors': anchors,
                                 'replay_command': f'./replay-borg.py {output} --from-turn {max(0, description["terminal"]["turn"] - 105)}',
                                 'qualification': 'Original frames, not an assumed verified rerun or mid-run Borg-memory restore'})
        record['files_sha256'] = {str(p.relative_to(staging)): runtime.sha(p) for p in staging.rglob('*') if p.is_file()}
        runtime.write(staging / 'record.json', record)
        runtime.write(staging / 'assessment.json', {'classification': 'unassessed', 'confidence': 'unassessed', 'findings': [], 'feedback': [], 'actions': []})
        discussion = f'[Discuss this death]({issue})' if issue else 'Discussion: unassigned; attach an investigation URL after triage.'
        (staging / 'report.md').write_text(f'# {case}: {description["raw_cause"]} at turn {description["terminal"]["turn"]}\n\n{discussion}\n\n'
            'See record.json/build.json for immutable identity and original-frame windows. Full runs, living saves and builds remain in the linked recording bundle.\n\n'
            '## Evaluation and diagnosis\n\nUnassessed. Death alone does not establish a defect.\n\n'
            '## Action items\n\nRecord evidence, feedback and issue links before changing behavior.\n')
        os.replace(staging, output)
    return record


def aggregate(root):
    records, causes, actions = [], defaultdict(set), {}
    for path in sorted(root.glob('D*/record.json')):
        record = read(path)
        if record['schema'] != SCHEMA:
            raise ValueError('Unknown death record schema: ' + str(path))
        assessment = read(path.with_name('assessment.json'))
        records.append((path, record, assessment))
        context = canonical_identity(record)['encounter_family']
        if assessment.get('encounter_family_qualification', {}).get('status') == 'verified':
            context = assessment.get('encounter_family', context)
        causes[record['raw_cause']].add(context)
        for action in assessment.get('actions', []):
            key = (action.get('issue'), action.get('key', action['summary']))
            item = actions.setdefault(key, {**action, 'cases': []})
            item['cases'].append(record['id'])
    attempts = {attempt['result_sha256']: attempt for _, record, _ in records for attempt in record['attempts'] if attempt['outcome'] == 'death'}
    summary = {'schema': 'borg-rs/death-index@1', 'cases': len(records),
               'distinct_starting_conditions': len(set.union(set(), *causes.values())),
               'death_process_attempts': len(attempts), 'causes_by_distinct_start': {k: len(v) for k, v in sorted(causes.items())},
               'versions': dict(Counter(a['source_commit'] for a in attempts.values())), 'actions': list(actions.values()),
               'limit': 'Selected cases; repeats/reference variants and backup branches are not independent lives or a mortality-rate estimate'}
    text = '# Rust Borg death records\n\nEvery confirmed natural death gets an immutable source/build/input identity, original observations, an assessment and linked actions. Full raw runs/saves/builds stay outside Git. Death alone does not establish a bug.\n\n'
    text += f"Dossiers: {summary['cases']}; distinct starting conditions: {summary['distinct_starting_conditions']}; retained process attempts: {summary['death_process_attempts']} (including repeats). This registry is not a survival-rate estimate.\n\n"
    text += '| Case | Discussion | Actual source | Seed | Raw cause | Death turn | Diagnosis |\n| --- | --- | --- | --- | --- | --- | --- |\n'
    for path, record, assessment in records:
        discussion = f'[Discuss]({record["investigation"]})' if record.get('investigation') else 'unassigned'
        effective_seed = canonical_identity(record)['seed']
        seed_label = effective_seed if effective_seed is not None else 'not used (saved RNG)' if record.get('recording_schema') == runtime.SCHEMA else 'recorded GUI inputs'
        text += f'| [{record["id"]}]({path.parent.name}/report.md) | {discussion} | `{record["identity"]["source"][:12]}` | {seed_label} | {record["raw_cause"]} | {record["identity"]["turn"]} | {assessment["classification"]} |\n'
    text += '\n## Aggregated actions\n\n'
    for action in actions.values():
        link = f'[{action["summary"]}]({action["issue"]})' if action.get('issue') else action['summary']
        text += f'- {link} — {action["status"]}; cases {", ".join(action["cases"])}.\n'
    text += '\n[Earlier nine-class cohort and assessments](cohort-2026-11-07.md) · [Feedback process](FEEDBACK.md).\n\n'
    text += 'Export with tools/record-death.py, curate assessment.json/report.md, regenerate with tools/aggregate-deaths.py. Original-C defect leads go to neo-angband-tracker under the repository policy; no automatic issues or gameplay changes are made by the logger.\n'
    return summary, text


def update_index(root):
    summary, text = aggregate(root)
    atomic_json(root / 'index.json', summary)
    temporary = root / '.README.md.tmp'
    temporary.write_text(text)
    os.replace(temporary, root / 'README.md')


def maybe_register(bundle, issue=None):
    issue = issue or discussion_issue()
    description = describe(bundle)
    if description is None:
        return None
    root = registry_root()
    with registry_lock(root):
        for path in root.glob('D*/record.json'):
            record = read(path)
            if canonical_identity(record) == description['identity']:
                reference = record.get('recording')
                if reference:
                    write_dossier(Path(reference['raw_directory']), path.parent, record['id'], issue=issue, repeat=[bundle])
                    update_index(root)
                print('Death already recorded:', path.parent, flush=True)
                return path.parent
        numbers = [int(read(p)['id'][1:]) for p in root.glob('D*/record.json')]
        case = f'D{max(numbers, default=0) + 1:04d}'
        output = root / (case + '-recorded')
        write_dossier(bundle, output, case, issue=issue)
        update_index(root)
        print('Natural death dossier:', output, '(unassessed; replay/diagnosis pending)', flush=True)
        return output

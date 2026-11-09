"""The death registry's index (docs/deaths/index.json and README.md).

Ours since TE.6 (#76): taken from borg-rs's tools/borg_deaths.py (its
registry_lock, canonical_identity and aggregate; docs/PROVENANCE.md) when the
copied recording toolkit was dropped. index.json and the README's case rows
regenerate byte for byte; the README's closing instructions name our tools,
not the removed ones. It reads the dossiers of
tools/export-death.py (schema borg-reborn-rs/death-record@1) and the ones the
copied GUI tooling wrote before TE.6 (borg-rs/death-record@1, e.g. D0526).
"""

import fcntl
import hashlib
import json
import os
from collections import Counter, defaultdict
from contextlib import contextmanager
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASIC_SCHEMA = 'borg-reborn-rs/death-record@1'
RECORDED_SCHEMA = 'borg-rs/death-record@1'
RECORDING_SCHEMA = 'borg-rs-headless-recording@1'


def read(path):
    return json.loads(Path(path).read_text())


def write_json(path, value):
    temp = path.with_name('.' + path.name + '.tmp')
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    os.replace(temp, path)


def registry_root():
    return Path(os.environ.get('BORG_DEATH_RECORDS_DIR', ROOT / 'docs/deaths')).resolve()


@contextmanager
def registry_lock(root):
    root.mkdir(parents=True, exist_ok=True)
    with (root / '.registry.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def fingerprint(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def family(seed, race, cls, config, origin):
    return 'start-' + fingerprint({'seed': None if origin is not None else seed, 'race': race, 'class': cls,
                                   'config': config, 'origin': origin})


def canonical_identity(record):
    """A record's identity, with the fields the index reads."""
    identity = dict(record['identity'])
    if record.get('schema') == BASIC_SCHEMA:
        identity.setdefault('source', identity.get('git_rev') or 'unknown')
        identity.setdefault('turn', record['terminal'].get('turn'))
        identity.setdefault('encounter_family', record.get('encounter_family'))
        return identity
    if record.get('recording_schema') == RECORDING_SCHEMA and identity.get('start_save') is not None:
        terminal = record['terminal']
        identity['seed'] = None
        identity['encounter_family'] = family(None, terminal['race_idx'], terminal['class_idx'],
                                              identity['config_sha256'], identity['start_save'])
    return identity


def aggregate(root):
    records, causes, actions = [], defaultdict(set), {}
    for path in sorted(root.glob('D*/record.json')):
        record = read(path)
        if record['schema'] not in (RECORDED_SCHEMA, BASIC_SCHEMA):
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
    # One attempt per (encounter family, key stream); the same keys from
    # different starts are different attempts (M1 review).
    attempts = {(canonical_identity(record)['encounter_family'] or record.get('encounter_family'), attempt['result_sha256']): attempt
                for _, record, _ in records for attempt in record['attempts'] if attempt['outcome'] == 'death'}
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
        ident = canonical_identity(record)
        effective_seed = ident['seed']
        seed_label = (effective_seed if effective_seed is not None
                      else 'not used (saved RNG)' if record.get('recording_schema') == RECORDING_SCHEMA else 'recorded GUI inputs')
        text += f'| [{record["id"]}]({path.parent.name}/report.md) | {discussion} | `{str(ident["source"])[:12]}` | {seed_label} | {record["raw_cause"]} | {ident["turn"]} | {assessment["classification"]} |\n'
    text += '\n## Aggregated actions\n\n'
    for action in actions.values():
        link = f'[{action["summary"]}]({action["issue"]})' if action.get('issue') else action['summary']
        text += f'- {link} — {action["status"]}; cases {", ".join(action["cases"])}.\n'
    links = [f'[{label}]({name})' for name, label in (('cohort-2026-11-07.md', 'Earlier nine-class cohort and assessments'),
                                                       ('FEEDBACK.md', 'Feedback process')) if (root / name).exists()]
    if links:
        text += '\n' + ' · '.join(links) + '.\n\n'
    text += 'Each run that dies is exported by tools/export-death.py (harness/run.sh). Curate assessment.json and report.md by hand; regenerate this index with `python3 tools/death_registry.py`; rerun a case with its logs with tools/rerun-death.py. Original-C defect leads go to neo-angband-tracker under the repository policy; no automatic issues or gameplay changes are made by the logger.\n'
    return summary, text


def update_index(root):
    summary, text = aggregate(root)
    write_json(root / 'index.json', summary)
    temporary = root / '.README.md.tmp'
    temporary.write_text(text)
    os.replace(temporary, root / 'README.md')


def main():
    root = registry_root()
    with registry_lock(root):
        update_index(root)
    print(f'{root}: index regenerated')


if __name__ == '__main__':
    main()

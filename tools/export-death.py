#!/usr/bin/env python3
"""Write a basic death dossier for a harness run that ended in a natural death.

Called by harness/run.sh after every run (task T1.2, issue #5; RULES 12), so
every game played in this repository, on any arm, leaves a dossier when it
dies. Runs that did not die are ignored. A repeat of an exported run (same
seed, build, configuration and key stream) is reported, never counted as a
new death.

  tools/export-death.py RUN_DIR --impl IMPL --build-id FILE [--flags F]

The dossier goes to $BORG_DEATH_RECORDS_DIR, default docs/deaths/, as
DNNNN-IMPL-seedSEED/ with:
  record.json      borg-rs-style record: identity, terminal state, attempts,
                   encounter family (same start state = same family)
  assessment.json  "unassessed"; filled in by a person or a later task
  report.md        one-screen summary
  summary.json, config.txt, build.json   the run's own small files
Raw logs (keys.log, messages.log, state.log) stay in RUN_DIR, outside Git;
the record names their SHA-256 (RULES 18). T1.6 upgrades these dossiers to
borg-rs's full format and T3.6 adds the decision trace.
"""

import argparse
import fcntl
import hashlib
import json
import os
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = 'borg-reborn-rs/death-record@1'


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def read_kv(path):
    out = {}
    if path and Path(path).exists():
        for line in Path(path).read_text().splitlines():
            if '=' in line:
                k, v = line.split('=', 1)
                out[k] = v
    return out


def registry_dir():
    return Path(os.environ.get('BORG_DEATH_RECORDS_DIR') or ROOT / 'docs' / 'deaths')


def existing(reg):
    """(case dirs sorted, {result_sha256: case name})."""
    cases = sorted(d for d in reg.iterdir() if d.is_dir() and re.match(r'D\d{4}', d.name)) if reg.exists() else []
    seen = {}
    for d in cases:
        try:
            rec = json.loads((d / 'record.json').read_text())
        except (OSError, ValueError):
            continue
        for a in rec.get('attempts', []):
            if a.get('result_sha256'):
                seen.setdefault((rec.get('encounter_family'), a['result_sha256']), d.name)
    return cases, seen


def export(run_dir, impl, build_id_path, flags='-'):
    run_dir = Path(run_dir).resolve()
    # harness/suite.sh runs into OUTDIR.partial.<pid> and renames it to OUTDIR
    # when the suite ends: record where the logs will be.
    final_dir = re.sub(r'\.partial\.\d+(?=/|$)', '', str(run_dir))
    summary = json.loads((run_dir / 'summary.json').read_text())
    if not summary.get('dead'):
        return None, 'not dead'
    keys = run_dir / 'keys.log'
    if not keys.exists():
        raise SystemExit(f'export-death: {keys} missing; cannot identify the run')
    build = read_kv(build_id_path)
    config = read_kv(run_dir / 'config.txt')
    keys_sha = sha256_file(keys)
    start = '|'.join(str(x) for x in (build.get('binary_sha256', 'unknown'), impl, summary.get('seed'),
                                       summary.get('race_idx'), summary.get('class_idx'),
                                       config.get('borg_txt_sha256', 'unknown'), config.get('save_sha256', '')))
    family = 'start-' + hashlib.sha256(start.encode()).hexdigest()
    reg = registry_dir()
    reg.mkdir(parents=True, exist_ok=True)
    with open(reg / '.registry.lock', 'w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        cases, seen = existing(reg)
        if (family, keys_sha) in seen:
            return seen[(family, keys_sha)], 'repeat'
        n = max([int(d.name[1:5]) for d in cases] + [0]) + 1
        case = f"D{n:04d}-{impl}-seed{summary.get('seed')}"
        out = reg / case
        out.mkdir()
        files = {}
        for name in ('summary.json', 'config.txt'):
            if (run_dir / name).exists():
                shutil.copy(run_dir / name, out / name)
                files[name] = sha256_file(out / name)
        (out / 'build.json').write_text(json.dumps(build, indent=2, sort_keys=True) + '\n')
        files['build.json'] = sha256_file(out / 'build.json')
        raw = {n: sha256_file(run_dir / n) for n in ('keys.log', 'messages.log', 'state.log')
               if (run_dir / n).exists()}
        character = ' '.join(x for x in (summary.get('race'), summary.get('class')) if x)
        record = {
            'schema': SCHEMA,
            'id': case,
            'run_kind': 'harness',
            'character': character,
            'raw_cause': summary.get('death_cause') or 'unknown',
            'terminal': {
                'cause': summary.get('death_cause') or 'unknown', 'dead': True,
                'depth': summary.get('depth'), 'max_depth': summary.get('max_depth'),
                'level': summary.get('clevel'), 'turn': summary.get('game_turns'),
                'race_idx': summary.get('race_idx'), 'class_idx': summary.get('class_idx'),
            },
            'identity': {
                'impl': impl, 'seed': summary.get('seed'), 'flags': flags,
                'git_rev': build.get('git_rev'), 'git_dirty': build.get('git_dirty'),
                'source_sha256': build.get('source_sha256'), 'binary_sha256': build.get('binary_sha256'),
                'borg_txt_sha256': config.get('borg_txt_sha256'),
            },
            'attempts': [{
                'role': 'original', 'outcome': 'death', 'raw_directory': final_dir,
                'result_sha256': keys_sha, 'raw_sha256': raw,
                'source_commit': build.get('git_rev'), 'binary_sha256': build.get('binary_sha256'),
            }],
            'encounter_family': family,
            'independence_note': 'One independent death per encounter family; repeats of the same run are not new cases.',
            'files_sha256': files,
            'investigation': None,
            'issue': None,
        }
        (out / 'record.json').write_text(json.dumps(record, indent=2) + '\n')
        (out / 'assessment.json').write_text(json.dumps({
            'classification': 'unassessed', 'confidence': 'none',
            'cause_category': None, 'findings': [], 'actions': [], 'feedback': []}, indent=2) + '\n')
        (out / 'report.md').write_text(
            f"# {case}: {record['raw_cause']} at turn {summary.get('game_turns')}\n\n"
            f"{character}, character level {summary.get('clevel')}, depth {summary.get('depth')} "
            f"(max {summary.get('max_depth')}), seed {summary.get('seed')}, implementation `{impl}`, "
            f"flags `{flags}`.\n\n"
            f"Build `{build.get('git_rev', 'unknown')}` (dirty: {build.get('git_dirty', '?')}), binary "
            f"`{(build.get('binary_sha256') or 'unknown')[:12]}`. Keys `{keys_sha[:12]}`; raw logs stay in "
            f"`{final_dir}` (not committed).\n\n"
            "**Unassessed.** Basic dossier written by `tools/export-death.py` at the end of the run "
            "(RULES 12). Upgrade it with the full death tooling (T1.6) before investigating.\n")
        return case, 'exported'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('run_dir')
    ap.add_argument('--impl', required=True)
    ap.add_argument('--build-id', required=True, help='build/<impl>/BUILD_ID of the binary that ran')
    ap.add_argument('--flags', default='-')
    a = ap.parse_args(argv)
    case, what = export(a.run_dir, a.impl, a.build_id, a.flags)
    if what == 'exported':
        print(f'export-death: natural death recorded as {registry_dir() / case}', file=sys.stderr)
    elif what == 'repeat':
        print(f'export-death: repeat of {case}; not a new case', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())

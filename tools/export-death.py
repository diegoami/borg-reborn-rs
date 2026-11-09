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


# Thinks of the decision trace a dossier keeps.
TRACE_TAIL = 200

REQUIRED = ('record.json', 'assessment.json', 'report.md', 'build.json', 'summary.json', 'config.txt')


def existing(reg):
    """(case dirs sorted, {(family, result_sha256): case name}).

    A case directory without a readable record.json is an export that
    failed before this tool wrote atomically; it is reported, never
    silently skipped or renumbered around.
    """
    cases = sorted(d for d in reg.iterdir() if d.is_dir() and re.match(r'D\d{4}', d.name)) if reg.exists() else []
    seen, broken = {}, []
    for d in cases:
        try:
            rec = json.loads((d / 'record.json').read_text())
        except (OSError, ValueError):
            broken.append(d.name)
            continue
        for a in rec.get('attempts', []):
            if a.get('result_sha256'):
                seen.setdefault((rec.get('encounter_family'), a['result_sha256']), d.name)
    if broken:
        raise SystemExit('export-death: case directories without a readable record.json: '
                         + ', '.join(broken) + '. Repair or remove them by hand, then rerun.')
    return cases, seen


def dossier(case, run_dir, final_dir, impl, flags, summary, build, config, keys_sha, family):
    """The files of a basic dossier, {name: bytes}."""
    files = {}
    for name in ('summary.json', 'config.txt', 'depths.json', 'params.json'):
        if (run_dir / name).exists():
            files[name] = (run_dir / name).read_bytes()
    # The reborn Borg's decision trace (T2.4): its last TRACE_TAIL lines,
    # the thinks that led to the death.
    trace_lines = 0
    if (run_dir / 'trace.jsonl').exists():
        lines = (run_dir / 'trace.jsonl').read_bytes().splitlines(keepends=True)
        trace_lines = len(lines)
        files['trace-tail.jsonl'] = b''.join(lines[-TRACE_TAIL:])
    files['build.json'] = (json.dumps(build, indent=2, sort_keys=True) + '\n').encode()
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
            # The character played (a random birth reports it apart from the
            # requested indices), as borg-rs's full exporter does.
            'race_idx': summary.get('played_race_idx', summary.get('race_idx')),
            'class_idx': summary.get('played_class_idx', summary.get('class_idx')),
        },
        'identity': {
            'impl': impl, 'seed': summary.get('seed'), 'flags': flags,
            'git_rev': build.get('git_rev'), 'git_dirty': build.get('git_dirty'),
            'source_sha256': build.get('source_sha256'), 'binary_sha256': build.get('binary_sha256'),
            'borg_txt_sha256': config.get('borg_txt_sha256'),
            'profile_sha256': config.get('profile_sha256'),
        },
        'decision_trace': ({'file': 'trace-tail.jsonl', 'lines': min(trace_lines, TRACE_TAIL), 'of': trace_lines}
                           if 'trace-tail.jsonl' in files else None),
        'attempts': [{
            'role': 'original', 'outcome': 'death', 'raw_directory': final_dir,
            'result_sha256': keys_sha, 'raw_sha256': raw,
            'source_commit': build.get('git_rev'), 'binary_sha256': build.get('binary_sha256'),
        }],
        'encounter_family': family,
        'independence_note': 'One independent death per encounter family; repeats of the same run are not new cases.',
        'files_sha256': {n: hashlib.sha256(b).hexdigest() for n, b in sorted(files.items())},
        'investigation': None,
        'issue': None,
    }
    files['record.json'] = (json.dumps(record, indent=2) + '\n').encode()
    files['assessment.json'] = (json.dumps({
        'classification': 'unassessed', 'confidence': 'none',
        'cause_category': None, 'findings': [], 'actions': [], 'feedback': []}, indent=2) + '\n').encode()
    files['report.md'] = (
        f"# {case}: {record['raw_cause']} at turn {summary.get('game_turns')}\n\n"
        f"{character}, character level {summary.get('clevel')}, depth {summary.get('depth')} "
        f"(max {summary.get('max_depth')}), seed {summary.get('seed')}, implementation `{impl}`, "
        f"flags `{flags}`.\n\n"
        f"Build `{build.get('git_rev', 'unknown')}` (dirty: {build.get('git_dirty', '?')}), binary "
        f"`{(build.get('binary_sha256') or 'unknown')[:12]}`. Keys `{keys_sha[:12]}`; raw logs stay in "
        f"`{final_dir}` (not committed).\n\n"
        + (f"Decision trace: the last {min(trace_lines, TRACE_TAIL)} of {trace_lines} thinks in "
           f"`trace-tail.jsonl`; parameters in `params.json` (profile "
           f"`{(config.get('profile_sha256') or 'unknown')[:12]}`).\n\n" if 'trace-tail.jsonl' in files else '')
        + "**Unassessed.** Basic dossier written by `tools/export-death.py` at the end of the run "
        "(RULES 12). Upgrade it with the full death tooling (T1.6) before investigating.\n").encode()
    return files


# Causes that are not deaths by play (engine: cmd-misc.c "Retiring";
# ui-signals.c "Interrupting", a signal; the player quitting). borg-rs's full
# exporter (tools/borg_deaths.py) rejects the same, and an empty cause.
NOT_NATURAL_CAUSES = {'Retiring', 'Quitting', 'Interrupting', 'Ripe Old Age'}


def not_natural(summary, flags='-'):
    """None for a natural death, else why the run is not one. Only natural
    deaths enter the registry (RULES 12; as borg-rs's full exporter)."""
    flag_set = set(flags.split(','))
    if not summary.get('dead'):
        return 'not dead'
    if summary.get('stop_reason') != 'death':
        return f"not natural: stop reason {summary.get('stop_reason')!r}"
    if summary.get('cheat_live') or 'cheatlive' in flag_set or 'cheat' in flag_set:
        return 'not natural: cheat death or cheat_live'
    if not (summary.get('death_cause') or '').strip():
        return 'not natural: no cause of death'
    if summary.get('death_cause') in NOT_NATURAL_CAUSES:
        return f"not natural: cause {summary.get('death_cause')!r}"
    return None


def family_of(impl, seed, race_idx, class_idx, source_sha256, borg_txt_sha256, save_sha256=None):
    """The encounter family: what fixes the starting game. It uses the build-input
    hash (harness/identity.sh source-hash), not the binary's bytes: rebuilding the
    same source gives a different binary hash (the build stamps the commit) but the
    same game, so a rerun of the same run is a repeat, not a new death."""
    start = '|'.join(str(x) for x in ('v2', impl, seed, race_idx, class_idx, source_sha256 or 'unknown',
                                       borg_txt_sha256 or 'unknown', save_sha256 or ''))
    return 'start-' + hashlib.sha256(start.encode()).hexdigest()


def export(run_dir, impl, build_id_path, flags='-'):
    """Returns (case, what): what is 'not dead', 'exported', 'repeat', or
    'repaired: NAMES' when a repeat found its dossier incomplete and wrote
    the missing files (existing files are never overwritten)."""
    run_dir = Path(run_dir).resolve()
    # harness/suite.sh runs into OUTDIR.partial.<pid> and renames it to OUTDIR
    # when the suite ends: record where the logs will be.
    final_dir = re.sub(r'\.partial\.\d+(?=/|$)', '', str(run_dir))
    summary = json.loads((run_dir / 'summary.json').read_text())
    why = not_natural(summary, flags)
    if why:
        return None, why
    keys = run_dir / 'keys.log'
    if not keys.exists():
        raise SystemExit(f'export-death: {keys} missing; cannot identify the run')
    build = read_kv(build_id_path)
    config = read_kv(run_dir / 'config.txt')
    keys_sha = sha256_file(keys)
    flag_set = set(flags.split(','))
    # A loaded save that keeps its own RNG starts the same game whatever the
    # seed; keep-load-rng without a save has no effect, so the seed stays. A
    # random birth starts from the character actually played. Whether a save
    # was loaded is borgh's own evidence (summary "loaded"), which also covers
    # `run.sh ... -- -load FILE`; run.sh writes keep_load_rng=1 to config.txt
    # when -keep-load-rng came that way too (re-review R3).
    keeps_rng = 'keep-load-rng' in flag_set or config.get('keep_load_rng') == '1'
    seed = None if summary.get('loaded') and keeps_rng else summary.get('seed')
    race = summary.get('played_race_idx', summary.get('race_idx'))
    cls = summary.get('played_class_idx', summary.get('class_idx'))
    family = family_of(impl, seed, race, cls,
                       build.get('source_sha256'), config.get('borg_txt_sha256'), config.get('save_sha256'))
    reg = registry_dir()
    reg.mkdir(parents=True, exist_ok=True)
    with open(reg / '.registry.lock', 'w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        # Leftovers of an export interrupted before its rename: never a case.
        for stale in reg.glob('.tmp-*'):
            shutil.rmtree(stale)
        cases, seen = existing(reg)
        args = (run_dir, final_dir, impl, flags, summary, build, config, keys_sha, family)
        if (family, keys_sha) in seen:
            case = seen[(family, keys_sha)]
            out = reg / case
            missing = [n for n in REQUIRED if not (out / n).exists()]
            if not missing:
                return case, 'repeat'
            files = dossier(case, *args)
            written = [name for name in missing if name in files]
            for name in written:
                (out / name).write_bytes(files[name])
            if not written:  # e.g. a run without config.txt: nothing to restore
                return case, 'repeat'
            return case, 'repaired: ' + ', '.join(written)
        n = max([int(d.name[1:5]) for d in cases] + [0]) + 1
        case = f"D{n:04d}-{impl}-seed{summary.get('seed')}"
        tmp = reg / f'.tmp-{case}'
        tmp.mkdir()
        for name, data in dossier(case, *args).items():
            (tmp / name).write_bytes(data)
        os.rename(tmp, reg / case)  # atomic: a case directory is complete or absent
        return case, 'exported'


def aggregate(reg):
    """Regenerate the registry's index.json and README.md (borg-rs's
    aggregate-deaths.py), as borg-rs's own registration does after an export."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import borg_deaths
    with borg_deaths.registry_lock(reg):
        borg_deaths.update_index(reg)


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
    elif what.startswith('repaired'):
        print(f'export-death: repeat of {case}; its dossier was incomplete, {what}', file=sys.stderr)
    elif what.startswith('not natural'):
        print(f'export-death: {what}; no dossier', file=sys.stderr)
    if case:
        # Also on repeats and repairs: an export interrupted after its rename
        # but before aggregation leaves the index stale (M1 review).
        aggregate(registry_dir())
    return 0


if __name__ == '__main__':
    sys.exit(main())

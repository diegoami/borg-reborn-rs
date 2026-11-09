#!/usr/bin/env python3
"""Rerun a recorded death with its logs (task TE.6, #76; docs/SPEC.md §8, §11).

  tools/rerun-death.py docs/deaths/D0664-rust-seed102 OUTDIR [--checkpoint-every N] [--any-build]

A run is deterministic: the same build, seed, character and flags play the
same game. This reruns a dossier's game through harness/run.sh with the
Borg's logs on: the observation of every think, the world model's digest
after each, the knowledge (docs/ENGINE.md), and, with --checkpoint-every,
an engine save every N game turns. It then checks that the key stream
equals the recorded one (record.json's result_sha256), which shows the
rerun is the same game.

The build must be the death's: build/rust/BUILD_ID's source_sha256 equals
the dossier's, else the tool says which commit to check out and build.
--any-build reruns anyway; the result is then not a reproduction.

OUTDIR gets the run directory (OUTDIR/run: keys, messages, state, summary,
trace, params, checkpoints) and the logs (OUTDIR/observations.jsonl,
worlds.txt, knowledge.json). The rerun's death is not exported to the
registry again (it would be a repeat).
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def read_kv(path):
    if not path.exists():
        return {}
    return dict(l.split('=', 1) for l in path.read_text().splitlines() if '=' in l)


def plan(case_dir):
    """What to run for a dossier: (seed, race, class, cap, flags, profile, expected keys hash, source)."""
    record = json.loads((case_dir / 'record.json').read_text())
    if record.get('schema') != 'borg-reborn-rs/death-record@1':
        raise SystemExit(f'rerun-death: {case_dir.name} is not a harness dossier ({record.get("schema")}); '
                         'only tools/export-death.py dossiers can be rerun')
    ident, term = record['identity'], record['terminal']
    if ident.get('impl') != 'rust':
        raise SystemExit(f'rerun-death: {case_dir.name} is a {ident.get("impl")} death; only the Rust Borg is built here')
    config = read_kv(case_dir / 'config.txt')
    flags = config.get('flags', ident.get('flags') or '-')
    # Requested indices: the run's own (summary.json), not the played ones.
    summary = json.loads((case_dir / 'summary.json').read_text())
    # A loaded save (save=NAME, or -load among the extra options) starts
    # another game than the seed's: the dossier alone cannot replay it.
    if 'save=' in flags or 'save_sha256' in config or summary.get('loaded'):
        raise SystemExit(f'rerun-death: {case_dir.name} loaded a save; it cannot be rerun from the dossier alone')
    return {
        'seed': ident['seed'] if ident.get('seed') is not None else summary.get('seed'),
        'race': summary.get('race_idx', term.get('race_idx')),
        'class': summary.get('class_idx', term.get('class_idx')),
        # The death came before this cap: the rerun stops there or earlier.
        'cap': int(term.get('turn') or summary.get('game_turns')) + 1,
        'flags': flags.replace('exact', '').replace(',,', ',').strip(',') or '-',
        'profile': config.get('profile'),
        'keys_sha256': record['attempts'][0]['result_sha256'],
        'source_sha256': ident.get('source_sha256'),
        'git_rev': ident.get('git_rev'),
    }


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('case', type=Path)
    ap.add_argument('out', type=Path)
    ap.add_argument('--checkpoint-every', type=int, default=0)
    ap.add_argument('--any-build', action='store_true')
    a = ap.parse_args(argv)
    # run.sh runs in its own directory: relative paths would land elsewhere.
    a.case, a.out = a.case.resolve(), a.out.resolve()
    p = plan(a.case)
    build = read_kv(ROOT / 'build/rust/BUILD_ID')
    if build.get('source_sha256') != p['source_sha256'] and not a.any_build:
        raise SystemExit(f"rerun-death: build/rust is source {str(build.get('source_sha256'))[:12]}, the death's is "
                         f"{str(p['source_sha256'])[:12]}: check out {p['git_rev']} and run harness/build.sh "
                         '(or pass --any-build: then the rerun is not a reproduction)')
    if a.out.exists():
        raise SystemExit(f'rerun-death: {a.out} exists')
    a.out.mkdir(parents=True)
    # The build checked above, never an inherited ANGBAND_BIN.
    env = {k: v for k, v in os.environ.items() if k != 'ANGBAND_BIN'}
    env.update(BORG_IMPL='rust',
               BORG_OBSERVATION_LOG=str(a.out / 'observations.jsonl'),
               BORG_WORLD_LOG=str(a.out / 'worlds.txt'),
               BORG_KNOWLEDGE_LOG=str(a.out / 'knowledge.json'),
               BORG_DEATH_RECORDS_DIR=str(a.out / 'not-exported'),
               CHECKPOINT_EVERY=str(a.checkpoint_every))
    if p['profile']:
        env['BORG_PROFILE'] = str(ROOT / p['profile'])
    cmd = [str(ROOT / 'harness/run.sh'), str(a.out / 'run'), str(p['seed']), str(p['race']), str(p['class']),
           str(p['cap']), p['flags']]
    r = subprocess.run(cmd, env=env, capture_output=True, text=True)
    keys = a.out / 'run/keys.log'
    got = hashlib.sha256(keys.read_bytes()).hexdigest() if keys.exists() else None
    same = got == p['keys_sha256']
    summary = json.loads((a.out / 'run/summary.json').read_text()) if (a.out / 'run/summary.json').exists() else {}
    print(f"{a.case.name}: rerun {'reproduces' if same else 'DOES NOT reproduce'} the recorded keys "
          f"({str(got)[:12]} vs {p['keys_sha256'][:12]}); {summary.get('stop_reason')} at turn {summary.get('game_turns')}"
          f"{' (' + summary['death_cause'] + ')' if summary.get('death_cause') else ''}")
    missing = [n for n in ('observations.jsonl', 'worlds.txt', 'knowledge.json')
               if not (a.out / n).exists() or not (a.out / n).stat().st_size]
    if missing:
        print(f"rerun-death: no logs written: {', '.join(missing)}", file=sys.stderr)
    print(f'logs in {a.out}')
    if r.returncode and not same:
        sys.stderr.write(r.stderr[-2000:])
    return 0 if (same or a.any_build) and not missing else 1


if __name__ == '__main__':
    sys.exit(main())

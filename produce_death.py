#!/usr/bin/env python3
"""Collect a natural recorded Rust Borg death within a prospective bounded plan."""
# Adapted from better-borg-c 22f817c96843; see tools/IMPORTED.json/licence.
import argparse
from collections import Counter
from pathlib import Path
import secrets
import sys
import subprocess
import signal
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
import death_workflow as workflow
import borg_runtime as runtime
import borg_deaths as deaths
import run_report
import toolkit_common as ui


def seeds_for(args):
    if args.seeds is not None:
        if args.games is not None or args.seed is not None:
            raise ValueError('Use --seeds OR --games/--seed')
        seeds = args.seeds
    else:
        first = args.seed if args.seed is not None else secrets.randbits(32)
        seeds = [(first + index) % (2**32) for index in range(args.games if args.games is not None else 3)]
    if not seeds or len(seeds) > 1000 or len(set(seeds)) != len(seeds) or any(not 0 <= seed <= 0xffffffff for seed in seeds):
        raise ValueError('Use 1..1000 distinct uint32 seeds')
    return seeds


def prospective_births(seeds, birth, commit):
    # Native '*' consumes the seeded engine's own race/class draws. Choices
    # are deliberately not predicted using a different random generator.
    return [{'seed': seed, 'race': None if birth == 'random' else 0,
             'class': None if birth == 'random' else 0,
             'race_name': 'native random' if birth == 'random' else 'Human',
             'class_name': 'native random' if birth == 'random' else 'Warrior'} for seed in seeds]


def outcome(bundle):
    summary_path, process_path = bundle / 'run/summary.json', bundle / 'run/process.json'
    summary = workflow.read(summary_path) if summary_path.exists() else None
    process = workflow.read(process_path) if process_path.exists() else {}
    if summary and summary.get('dead') and summary.get('stop_reason') == 'death':
        result = 'retirement' if summary.get('death_cause') in ('Retiring', 'Quitting', 'Interrupting') else 'death'
    elif process.get('status') == 'timeout':
        result = 'wall_timeout'
    elif process.get('status') != 'success':
        result = process.get('status', 'harness_failure')
    else:
        result = {'turns': 'turn_cap', 'keys': 'key_cap', 'completion': 'completion'}.get(summary.get('stop_reason') if summary else None, 'unexpected_stop')
    return {'outcome': result, 'cause': summary.get('death_cause') if summary else None,
            'character': f"{summary['race']} {summary['class']}" if summary else None,
            'final': summary, 'process_status': process.get('status'),
            'qualification': 'recording/export validation pending'}


def collect(args, output, births, commit):
    hashes = workflow.tool_hashes()
    args.death_issue = args.death_issue or deaths.discussion_issue()
    plan = {'schema': 'borg-rs/death-collection@1', 'created_at': workflow.now(), 'source_commit': commit,
            'seeds': [entry['seed'] for entry in births], 'prospective_births': births,
            'turn_cap': args.turns, 'key_cap': args.keys,
            'cap_boundary': 'first living native top-level command after threshold; menus may complete beyond key threshold', 'wall_limit_per_game_seconds': args.timeout,
            'stall_timeout_seconds': args.stall_timeout, 'checkpoint_every': args.checkpoint_every,
            'stop_rule': 'all listed seeds; continue after every exported death and ordinary cap/timeout; stop on operational/export error',
            'munchkin': getattr(args, 'munchkin', False), 'birth_policy': args.birth, 'character_selection': 'seeded native random menu (*) or explicit fixed birth',
            'safety': 'cheat death/live and winner respawn off; native death evidence on',
            'tools_sha256': hashes, 'issue': args.death_issue, 'runs': [], 'counts': {}, 'status': 'preparing',
            'limit': 'Death collection, not a survival-rate cohort; no retrospective seed/budget enlargement'}
    contract = {key: value for key, value in plan.items() if key not in ('runs', 'counts', 'status')}
    workflow.write(output / 'plan.json', contract)
    plan['plan_sha256'] = workflow.sha(output / 'plan.json')
    workflow.write(output / 'collection.json', plan)
    try:
        ui.section('Build preparation')
        print('Preparing the pinned Rust recording runtime...', flush=True)
        with workflow.locked(workflow.primary_root() / 'build/death-workflow/prepare.lock'):
            prepared = workflow.prepare_runtime(args, commit, hashes)
        plan.update(prepared_manifest=str(Path(prepared['cache']) / 'prepared.json'),
                    prepared_manifest_sha256=workflow.sha(Path(prepared['cache']) / 'prepared.json'), status='running')
        workflow.write(output / 'collection.json', plan)
        for index, birth in enumerate(births, 1):
            workflow.validate_prepared(prepared, commit, hashes)
            profile = output / f"game-{index:03d}-seed-{birth['seed']}"
            entry = {**birth, 'directory': str(profile), 'outcome': 'harness_failure'}
            plan['runs'].append(entry)
            workflow.write(output / 'collection.json', plan)
            options = SimpleNamespace(character='CollectedBorg', new=True, from_backup=None, seed=birth['seed'],
                race=birth['race'], cls=birth['class'], munchkin=getattr(args, 'munchkin', False), turns=args.turns, keys=args.keys, wall_seconds=args.timeout,
                stall_seconds=args.stall_timeout, checkpoint_every=args.checkpoint_every, death_issue=args.death_issue)
            ui.section(f'Game {index}/{len(births)}')
            birth_label = 'native random race/class' if birth['race'] is None else birth['race_name'] + ' ' + birth['class_name']
            print(f"Seed {birth['seed']}; {birth_label}; cap {args.turns} engine ticks", flush=True)
            bundle = None
            try:
                if workflow.sha(output / 'plan.json') != plan['plan_sha256']:
                    raise ValueError('Prospective plan changed during collection')
                game_prepared = {**prepared, 'context': {**prepared.get('context', {}), 'collection_plan': contract, 'collection_plan_sha256': plan['plan_sha256']}}
                bundle = runtime.record_run(options, prepared=game_prepared, profile_root=profile, export_death=False)
                entry.update(outcome(bundle), recording=str(bundle))
                recording = runtime.validate(bundle, require_complete=entry['outcome'] != 'wall_timeout')
                if entry['outcome'] == 'wall_timeout' and not (recording.get('status') == 'timeout' and recording.get('finalized')):
                    raise ValueError('Timeout recording did not finalize consistently')
                workflow.validate_prepared(prepared, commit, hashes)
                entry['qualification'] = 'finalized incomplete timeout; view-only or strict prefix' if entry['outcome'] == 'wall_timeout' else 'complete frozen recording; rerun verification and assessment remain separate'
                entry['recording_manifest_sha256'] = workflow.sha(bundle / 'recording.json')
                if entry['outcome'] == 'death':
                    plan['counts'] = dict(Counter(item['outcome'] for item in plan['runs']))
                    workflow.write(output / 'collection.json', plan)
                    entry['export_attempted'] = True
                    dossier = deaths.maybe_register(bundle, issue=args.death_issue)
                    if dossier is None:
                        raise ValueError('Terminal death was not eligible for natural-death export')
                    entry.update(dossier=str(dossier), assessment='unassessed; owner replay and causal investigation pending')
                    plan.update(status='natural_death_recorded', counts=dict(Counter(item['outcome'] for item in plan['runs'])))
                    workflow.write(output / 'collection.json', plan)
                    print('Natural death:', dossier, flush=True)
                    print('Next: ./replay_next_death.py; viewing alone does not mark owner review complete.', flush=True)
                if entry['outcome'] not in ('death', 'turn_cap', 'key_cap', 'completion', 'wall_timeout'):
                    raise ValueError('Operational stop: ' + entry['outcome'])
            except (Exception, KeyboardInterrupt) as error:
                candidates = list((profile / 'recordings').glob('*'))
                if bundle is None and len(candidates) == 1:
                    bundle = candidates[0]
                if bundle is not None:
                    entry.update(outcome(bundle), recording=str(bundle))
                entry['error'] = str(error)
                if bundle is not None and entry.get('outcome') == 'death' and not entry.get('export_attempted'):
                    entry['export_attempted'] = True
                    try:
                        dossier = deaths.maybe_register(bundle, issue=args.death_issue)
                        if dossier is not None:
                            entry['dossier'] = str(dossier)
                    except (OSError, ValueError, KeyError) as export_error:
                        entry['export_error'] = str(export_error)
                raise
            finally:
                report_attempt(entry, bundle)
                run_report.collection_summary(output, plan)
            plan['counts'] = dict(Counter(item['outcome'] for item in plan['runs']))
            workflow.write(output / 'collection.json', plan)
        plan['status'] = 'completed' if plan['counts'].get('death') else 'limits_reached_without_death'
        workflow.write(output / 'collection.json', plan)
        print_summary(output, plan)
        return 0
    except (Exception, KeyboardInterrupt) as error:
        plan.update(status='failed', error=str(error), counts=dict(Counter(item['outcome'] for item in plan['runs'])))
        workflow.write(output / 'collection.json', plan)
        run_report.collection_summary(output, plan)
        print_summary(output, plan)
        raise


def report_attempt(entry, bundle):
    report_dir = Path(entry['directory']) / 'reports'
    try:
        if bundle is None:
            raise ValueError('Recording did not start; original movie and manual prefix are unavailable')
        report, _ = run_report.create(bundle, entry, report_dir)
    except (OSError, ValueError, KeyError) as error:
        report_dir.mkdir(parents=True, exist_ok=True)
        report = report_dir / ('failure-' + str(__import__('time').time_ns()) + '.md')
        report.write_text(f"# Attempt seed {entry['seed']}\n\nOutcome: {entry['outcome']}.\n\nOperational error: {entry.get('error', str(error))}\n\nReport evidence unavailable: {error}\n\nCounters, native sheet and missing movie data are unavailable; no new game was started for this report.\n")
        workflow.write(report.with_suffix('.json'), {'schema': 'borg-rs/run-report@1', 'seed': entry['seed'], 'outcome': entry['outcome'], 'operational_error': entry.get('error'), 'unavailable': str(error)})
    if bundle is None:
        entry.update(report=str(report), commands={}, commands_unavailable='No original movie was captured')
        print('Report:', report, 'Watch/manual unavailable: no captured movie.', flush=True)
        return
    entry['report'] = str(report)
    commands = {'watch': [str(ROOT / 'replay-borg.py'), str(bundle)],
                'manual': [str(ROOT / 'play-from-replay.py'), str(bundle), '--from-turn', 'latest']}
    if entry['outcome'] == 'wall_timeout':
        commands['watch'].append('--view-only')
    entry['commands'] = {name: __import__('shlex').join(argv) for name, argv in commands.items()}
    (Path(entry['directory']) / 'commands.txt').write_text('\n'.join(entry['commands'].values()) + '\n')
    friendly = {'death': 'Died naturally', 'turn_cap': 'Alive at engine tick limit', 'key_cap': 'Alive at Borg event limit', 'wall_timeout': 'Alive at finalized wall limit', 'completion': 'Completed'}.get(entry['outcome'], entry['outcome'])
    print('Outcome:', ui.color(friendly, '32' if entry['outcome'] in ('turn_cap', 'completion') else '33'), flush=True)
    print('Report:', report, flush=True)
    for title, argv in commands.items():
        print(title.title() + ':', flush=True)
        ui.command(argv)


def print_summary(output, plan):
    ui.section('Batch results')
    print('Actual attempts:', len(plan['runs']), 'Outcomes:', plan['counts'], flush=True)
    print('Comparison:', output / 'reports.md', flush=True)
    print('Deaths:', flush=True)
    for entry in plan['runs']:
        if entry['outcome'] == 'death':
            print(entry['seed'], entry.get('dossier', 'export unavailable'), flush=True)


def existing_collection(directory, reports=False):
    plan = workflow.read(directory / 'collection.json')
    if plan.get('schema') != 'borg-rs/death-collection@1':
        raise ValueError('Unsupported collection schema')
    for entry in plan['runs']:
        if reports and entry.get('recording'):
            report_attempt(entry, Path(entry['recording']))
        else:
            print('Report:', entry.get('report', 'unavailable'), flush=True)
            for command in entry.get('commands', {}).values():
                print('\n' + ui.color(command, '36') + '\n', flush=True)
    if reports:
        run_report.collection_summary(directory, plan)
        workflow.write(directory / 'collection.json', plan)
    print_summary(directory, plan)
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--munchkin', action='store_true', help='explicitly enable new munchkin games; default off')
    parser.add_argument('--yes', action='store_true', help='accept a needed pinned build')
    parser.add_argument('--no-build', action='store_true', help='require a compatible existing toolkit build')
    parser.add_argument('--check-build', action='store_true', help='check compatibility, then exit without games/build/lock')
    parser.add_argument('--commands', type=Path, help='reprint an existing collection; no game/build')
    parser.add_argument('--reports', type=Path, help='backfill existing reports; no new games')
    parser.add_argument('--birth', choices=('random', 'fixed'), default='random', help='prospectively varied race/class, or Human Warrior')
    parser.add_argument('--games', type=int, help='maximum games; default3')
    parser.add_argument('--seed', type=int, help='first uint32 seed; default random')
    parser.add_argument('--seeds', nargs='+', type=int)
    parser.add_argument('--turns', type=int, default=200000, help='absolute world-turn cap per new game')
    parser.add_argument('--timeout', type=int, default=120)
    parser.add_argument('--stall-timeout', type=int, default=30, help='seconds without advancement in the flushed key-log world turn')
    parser.add_argument('--ref', default='HEAD', help='committed borg-rs source pinned before collection')
    parser.add_argument('--build', type=Path, help='existing ordinary Rust harness build matching --ref')
    parser.add_argument('--output', type=Path, help='new directory under harness/out or BORG_DEATH_WORKFLOW_DIR')
    parser.add_argument('--death-issue', default=None, help='optional discussion link; never borrow a C repository issue')
    parser.add_argument('--keys', type=int, default=50000, help='independent Rust Borg event cap per game')
    parser.add_argument('--checkpoint-every', type=int, default=1000)
    args = parser.parse_args(argv)
    if args.yes and args.no_build:
        parser.error('Choose --yes or --no-build')
    if (args.games is not None and not 1 <= args.games <= 1000) or not 1 <= args.turns <= 2147483647 or min(args.timeout, args.stall_timeout, args.checkpoint_every) < 1 or not 1 <= args.keys <= 999999999 or (args.seed is not None and not 0 <= args.seed <= 0xffffffff):
        parser.error('Use1..1000 games, uint32 seeds, positive limits and supported turn/key budgets')
    try:
        if args.commands or args.reports:
            if args.commands and args.reports:
                raise ValueError('Choose --commands or --reports')
            if args.reports:
                with ui.native_lock():
                    return existing_collection(args.reports.resolve(), reports=True)
            return existing_collection(args.commands.resolve())
        commit = workflow.source_commit(args.ref)
        hashes = workflow.tool_hashes()
        available = workflow.build_offer(args, commit, hashes)
        if args.check_build:
            return 0 if available else 1
        seeds = seeds_for(args)
        commit = workflow.source_commit(args.ref)
        births = prospective_births(seeds, args.birth, commit)
        output = (args.output or workflow.artifacts() / 'collections' / workflow.now().replace(':', '-')).resolve()
        allowed = workflow.primary_root() / 'harness/out'
        if not output.is_relative_to(allowed) and not output.is_relative_to(workflow.artifacts()):
            raise ValueError('Keep full output below primary harness/out or BORG_DEATH_WORKFLOW_DIR')
        output.mkdir(parents=True, exist_ok=False)
        ui.section('Prospective plan')
        print(f'Pinned source: {commit}\nProspective seeds: {seeds}\nEvidence: {output}', flush=True)
        print(f'Limits per game: {args.turns} engine ticks, {args.keys} Borg events, {args.timeout} wall seconds, {args.stall_timeout} unchanged-clock seconds', flush=True)
        with ui.native_lock():
            return collect(args, output, births, commit)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        print(str(error) or 'Interrupted; partial evidence retained', file=sys.stderr)
        return 1


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise InterruptedError('Collection interrupted; partial evidence retained')
    signal.signal(signal.SIGTERM, interrupted)
    raise SystemExit(main())

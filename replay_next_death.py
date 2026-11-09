#!/usr/bin/env python3
"""Replay the next recorded encounter not explicitly marked reviewed by its owner."""
# Adapted from better-borg-c 22f817c96843; see tools/IMPORTED.json/licence.
import argparse
from pathlib import Path
import subprocess
import sys
import json

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
import death_workflow as workflow


def run(args):
    play_turn = getattr(args, 'play_from_turn', None)
    if args.mark_reviewed or args.mark_unreviewed:
        reviewed = bool(args.mark_reviewed)
        case = workflow.mark(args.mark_reviewed or args.mark_unreviewed, reviewed)
        print(f"{case['case']} encounter family marked {'reviewed' if reviewed else 'unreviewed'}.")
        return 0
    groups = workflow.queue()
    if args.list:
        if args.json:
            print(json.dumps(groups, indent=2))
        else:
            for group in groups:
                names = ', '.join(case['case'] for case in group['cases'])
                status = 'reviewed by owner' if group['reviewed'] else 'not reviewed by owner'
                replay = group['playable']['case'] if group['playable'] else 'no available full recording'
                print(f'{names}: {status}; replay: {replay}')
                for case in group['cases']:
                    if case['recording'] != 'recording available':
                        print(f"  {case['case']}: {case['recording']}")
        return 0
    selected = workflow.choose_case(args.case) if args.case else next((group['playable'] for group in groups if not group['reviewed'] and group['playable']), None)
    if selected is None:
        print('No unreviewed encounter with an available full recording.')
        print('Use --list for legacy/missing/reviewed cases; ./produce_death.py collects new recorded games.')
        return 0
    if selected['recording'] != 'recording available':
        raise ValueError(selected['recording'])
    print(f"Replay {selected['case']}: {selected['cause']} at turn {selected['turn']}.", flush=True)
    print(f"When finished reviewing: ./replay_next_death.py --mark-reviewed {selected['case']}", flush=True)
    print('Closing the viewer does not mark a review complete.', flush=True)
    command = [sys.executable, str(ROOT / 'replay-borg.py'), selected['directory'], '--timeout', str(args.timeout)]
    if not args.view_only and play_turn is None:
        command += ['--output', str(workflow.artifacts() / 'replays' / workflow.now().replace(':', '-'))]
    if args.from_turn is not None:
        command += ['--from-turn', str(args.from_turn)]
    if args.view_only:
        command += ['--view-only']
    if args.verify_only:
        command += ['--verify-only']
    if play_turn is not None:
        command += ['--play-from-turn', str(play_turn)]
    if getattr(args, 'prepare_only', False):
        command += ['--prepare-only']
    return subprocess.run(command, cwd=ROOT).returncode


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--list', action='store_true')
    mode.add_argument('--case')
    mode.add_argument('--mark-reviewed', metavar='CASE')
    mode.add_argument('--mark-unreviewed', metavar='CASE')
    parser.add_argument('--json', action='store_true', help='JSON with --list')
    parser.add_argument('--from-turn', type=int)
    view = parser.add_mutually_exclusive_group()
    view.add_argument('--view-only', action='store_true')
    view.add_argument('--verify-only', action='store_true')
    view.add_argument('--play-from-turn', type=int, metavar='TURN', help='create an independent human save at/after this turn')
    parser.add_argument('--prepare-only', action='store_true')
    parser.add_argument('--timeout', type=int, default=600)
    args = parser.parse_args(argv)
    if (args.prepare_only and args.play_from_turn is None) or (args.play_from_turn is not None and (args.play_from_turn < 0 or args.from_turn is not None)):
        parser.error('--prepare-only needs --play-from-turn; use a nonnegative turn without --from-turn')
    if args.timeout < 1 or (args.from_turn is not None and args.from_turn < 0) or (args.json and not args.list):
        parser.error('Use positive timeout, nonnegative turn and --json only with --list')
    try:
        return run(args)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())

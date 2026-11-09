#!/usr/bin/env python3
"""Verify a frozen Rust GUI/headless recording, then inspect its original frames."""
import argparse
import json
from pathlib import Path
import sys
import subprocess

sys.path.insert(0, str(Path(__file__).resolve().parent / 'tools'))
import borg_runtime
import borg_gui
import borg_replay_viewer
import replay_observations


def resolve_recording(path):
    path = path.resolve()
    if path.name == 'record.json' and path.is_file():
        path = path.parent
    if (path / 'recording.json').exists():
        return path
    if (path / 'record.json').exists():
        record = json.loads((path / 'record.json').read_text())
        reference = record.get('recording')
        if not reference:
            raise ValueError('This historical death has no complete recording; retain its original logs and assessments')
        bundle = Path(reference['raw_directory']).resolve()
        if not (bundle / 'recording.json').exists():
            raise ValueError('Restore the full original recording bundle: ' + str(bundle))
        if borg_runtime.sha(bundle / 'recording.json') != reference['manifest_sha256']:
            raise ValueError('Death dossier recording manifest differs')
        return bundle
    return path


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('recording', type=Path, help='recording bundle or death dossier directory')
    parser.add_argument('--from-turn', '--turn', dest='turn', type=int, help='first engine turn to inspect; default: last 105 turns')
    parser.add_argument('--dump-turn', type=int, help='print the recorded frame at/before this turn')
    parser.add_argument('--dump', action='store_true', help='print --from-turn/--turn without an interactive terminal')
    parser.add_argument('--play-from-turn', type=int, metavar='TURN', help='verify and copy a headless game at the next living command boundary, then play with Borg off')
    parser.add_argument('--prepare-only', action='store_true', help='prepare --play-from-turn save without opening X11')
    parser.add_argument('--combat-log', action='store_true', help='open captured native fighting log; missing legacy data remains unavailable')
    parser.add_argument('--yes', action='store_true', help='accept separate historical takeover helper build')
    parser.add_argument('--no-build', action='store_true')
    parser.add_argument('--verify-only', action='store_true')
    parser.add_argument('--view-only', '--recording-only', dest='recording_only', action='store_true', help='view with an UNVERIFIED badge')
    parser.add_argument('--output', type=Path, help='new directory retaining verification evidence, or the independent human branch with --play-from-turn')
    parser.add_argument('--timeout', type=int, help='positive wall limit for the verification run')
    args = parser.parse_args(argv)
    if args.play_from_turn is not None:
        if args.play_from_turn < 0 or args.turn is not None or args.dump_turn is not None or args.dump or args.verify_only or args.recording_only:
            parser.error('--play-from-turn requires a nonnegative turn and cannot be combined with viewing/verification modes')
    elif args.prepare_only:
        parser.error('--prepare-only requires --play-from-turn')
    if args.verify_only and args.recording_only:
        parser.error('--verify-only cannot be combined with --view-only/--recording-only')
    if (args.timeout is not None and args.timeout < 1) or any(t is not None and t < 0 for t in (args.turn, args.dump_turn)):
        parser.error('Use nonnegative engine turns and a positive timeout')
    if args.recording_only and args.output:
        parser.error('--output retains verification evidence; omit it with --view-only')
    existed = args.output.exists() if args.output else False
    try:
        directory = resolve_recording(args.recording)
        if args.play_from_turn is not None:
            import human_takeover
            human_takeover.BUILD_POLICY = 'no-build' if args.no_build else 'yes' if args.yes else 'ask'
            if not args.prepare_only:
                import display_font
                display_font.probe()
            branch = human_takeover.prepare(directory, args.play_from_turn, output=args.output, timeout=args.timeout or 600)
            if not args.prepare_only:
                human_takeover.launch(branch)
            return 0
        schema = json.loads((directory / 'recording.json').read_text()).get('schema')
        backend = borg_gui if schema == borg_gui.SCHEMA else borg_runtime
        backend.validate(directory, require_complete=not args.recording_only)
        verified = not args.recording_only
        if verified:
            proof = backend.verify(directory, output=args.output, timeout=args.timeout)
            print('VERIFIED frozen-origin rerun: ' + proof['model'], flush=True)
        if not args.verify_only:
            frames = directory if (directory / 'frames.jsonl.gz').exists() else directory / 'run'
            display = frames
            dump = args.dump or args.dump_turn is not None
            if schema == borg_runtime.SCHEMA and not dump and replay_observations.needed(directory):
                display = replay_observations.cached(directory) or frames
                if display == frames and verified:
                    try:
                        display = replay_observations.prepare(directory, timeout=args.timeout or 600)
                    except replay_observations.ObservationUnavailable as error:
                        print(str(error) + '; showing the verified original movie without guessed fields.', flush=True)
            supplemental = display != frames
            if supplemental:
                print('VERIFIED historical native observations; original recording unchanged.' if verified else 'View-only: using the previously qualified native-observation cache.', flush=True)
            borg_replay_viewer.show(display, turn=args.dump_turn if args.dump_turn is not None else args.turn,
                                   verified=verified, dump=dump, combat_log=args.combat_log, caster=replay_observations.caster(directory), supplemental=supplemental)
        return 0
    except (ValueError, OSError, KeyError, AssertionError, subprocess.SubprocessError) as error:
        if args.output and not existed and args.play_from_turn is None:
            args.output.mkdir(parents=True, exist_ok=True)
            borg_runtime.write(args.output / 'verification.json', {'verified': False, 'error': str(error)})
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())

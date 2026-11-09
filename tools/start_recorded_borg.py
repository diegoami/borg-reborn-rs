#!/usr/bin/env python3
"""Launch a named recorded native Rust game; start the Borg manually with Ctrl-Z z."""
import argparse
import fcntl
import os
from pathlib import Path
import secrets
import signal
import subprocess
import sys

import borg_gui as gui
import borg_runtime as runtime
import borg_saves as saves
import death_workflow as workflow
import display_font
from toolkit_common import native_lock


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('name', nargs='?', default=os.environ.get('BORG_PLAY_CHARACTER', 'RustBorg'))
    parser.add_argument('--seed', type=int)
    parser.add_argument('--from-backup', nargs='?', type=int, const=1, choices=range(1, 6))
    parser.add_argument('--build-only', action='store_true')
    args, extra = parser.parse_known_args(argv)
    try:
        runtime.character_name(args.name)
        if args.seed is not None and not 0 <= args.seed <= 0xffffffff:
            raise ValueError('Seed must be uint32')
        if not args.build_only:
            display_font.probe()
        pin = workflow.source_commit('HEAD')
        with native_lock():
            from types import SimpleNamespace
            prepared = workflow.prepare_runtime(SimpleNamespace(build=None), pin, workflow.tool_hashes())
        if args.build_only:
            print('Built immutable Rust engine:', prepared['binary']); return 0
        profile = runtime.data_root()
        for name in gui.PROFILE_DIRS:
            (profile / name).mkdir(parents=True, exist_ok=True)
        save = profile / 'save' / args.name
        lock = save.parent / '.locks' / (args.name + '.lock')
        lock.parent.mkdir(exist_ok=True)
        with lock.open('a') as held:
            try:
                fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise ValueError('Character is already open through a launcher') from error
            if args.from_backup is not None:
                import tempfile, shutil
                with tempfile.TemporaryDirectory(prefix='borg-visible-restore-') as temporary:
                    frozen = Path(temporary) / 'backup.save'
                    shutil.copy2(save.parent / 'backups' / save.name / f'save.{args.from_backup}', frozen)
                    with native_lock():
                        meta = runtime.inspect_save(frozen, Path(prepared['build_directory']), Path(prepared['binary']))
                    if meta['dead']:
                        raise ValueError('Selected backup is dead; current save preserved')
                    saves.restore(save, args.from_backup, validated_source=frozen)
            saves.backup(save)
            if not (profile / 'user/borg.txt').exists():
                import shutil
                shutil.copy2(prepared['config'], profile / 'user/borg.txt')
            fresh = not save.exists() or '-n' in extra
            seed = (args.seed if args.seed is not None else secrets.randbits(32)) if fresh else None
            context = {'kind': 'named_visible', 'visible_seed': seed, 'source_commit': pin,
                       'rng_origin': 'seeded birth' if fresh else 'native saved RNG; fresh Borg'}
            arguments = extra if '--' in extra else extra + ['--', '-n1']
            print('Create/load your character. Ctrl-Z then z starts the Rust Borg; Ctrl-X saves and exits.', flush=True)
            gui.start(Path(prepared['binary']), profile, args.name, arguments, lock_fd=held.fileno(),
                      data_source=Path(prepared['source_root']) / 'angband/lib', build_identity=prepared['build_id'], session_context=context)
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr); return 1


if __name__ == '__main__':
    def interrupted(signum, frame):
        raise InterruptedError('Visible launcher interrupted; session evidence retained')
    signal.signal(signal.SIGTERM, interrupted)
    raise SystemExit(main())

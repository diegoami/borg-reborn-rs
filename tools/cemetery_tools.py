"""Thin API-checked pointer to death tools owned by borg-cemetery."""
import os
from pathlib import Path
import subprocess


def locate(repo):
    explicit = os.environ.get('BORG_CEMETERY_DIR')
    if explicit:
        cemetery = Path(explicit).expanduser().resolve()
    else:
        try:
            common = subprocess.check_output(['git', '-C', str(repo), 'rev-parse', '--path-format=absolute', '--git-common-dir'], text=True, stderr=subprocess.DEVNULL).strip()
            primary = Path(common).parent
        except (OSError, subprocess.CalledProcessError):
            primary = repo
        cemetery = primary.parent / 'borg-cemetery'
        if not (cemetery / 'scripts/native/API_VERSION').is_file():
            # CI tests deliberately clear policy BORG_* variables. Its pinned
            # checkout is still discoverable without treating tooling as policy.
            candidates = (repo / 'vendor/borg-cemetery', primary / 'vendor/borg-cemetery')
            cemetery = next((p for p in candidates if (p / 'scripts/native/API_VERSION').is_file()), cemetery)
    api = cemetery / 'scripts/native/API_VERSION'
    if not api.is_file() or api.read_text().strip() != '1':
        raise RuntimeError('The death tools now live in borg-cemetery. Update/clone https://github.com/diegoami/borg-cemetery beside this repository, or set BORG_CEMETERY_DIR to its checkout (death-tool API 1 required).')
    return cemetery


def forward(name, namespace, *, extra_settings=()):
    repo = Path(namespace['__file__']).resolve().parent.parent
    try:
        cemetery = locate(repo)
    except RuntimeError as error:
        raise SystemExit(str(error))
    target = cemetery / 'scripts/native' / name
    namespace.update(SOURCE_ROOT=repo, __file__=str(target))
    # The canonical implementation remains the only implementation. Track R
    # adds its recorded shadow mode to the canonical settings table before
    # CLI dispatch, so imports and actual reruns use the same replay rules.
    deferred_cli = bool(extra_settings) and namespace.get('__name__') == '__main__'
    if deferred_cli:
        namespace['__name__'] = '_cemetery_compat_definition'
    try:
        exec(compile(target.read_text(), str(target), 'exec'), namespace)
    finally:
        if deferred_cli:
            namespace['__name__'] = '__main__'
    if extra_settings:
        current = namespace.get('SETTINGS')
        if not isinstance(current, tuple):
            raise RuntimeError('cemetery replay API must export its SETTINGS tuple')
        namespace['SETTINGS'] = tuple(extra_settings) + tuple(v for v in current if v not in extra_settings)
    if deferred_cli:
        raise SystemExit(namespace['main']())

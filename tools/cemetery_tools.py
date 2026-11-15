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
    api = cemetery / 'scripts/native/API_VERSION'
    if not api.is_file() or api.read_text().strip() != '1':
        raise RuntimeError('The death tools now live in borg-cemetery. Update/clone https://github.com/diegoami/borg-cemetery beside this repository, or set BORG_CEMETERY_DIR to its checkout (death-tool API 1 required).')
    return cemetery


def forward(name, namespace):
    repo = Path(namespace['__file__']).resolve().parent.parent
    try:
        cemetery = locate(repo)
    except RuntimeError as error:
        raise SystemExit(str(error))
    target = cemetery / 'scripts/native' / name
    namespace.update(SOURCE_ROOT=repo, __file__=str(target))
    exec(compile(target.read_text(), str(target), 'exec'), namespace)

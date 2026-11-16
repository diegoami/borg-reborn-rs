"""Thin API-checked pointer to death tools owned by borg-cemetery."""
import hashlib
import os
import re
from pathlib import Path
import subprocess


def verify_native_pin(repo, cemetery):
    """Permit later data/docs heads only when native tool bytes match our pin."""
    try:
        pin = (repo / 'death-tools-pin.txt').read_text().strip()
        if not re.fullmatch('[0-9a-f]{40}', pin):
            raise ValueError('death-tools-pin.txt must contain an immutable commit')
        result = subprocess.run(
            ['git', '-C', str(cemetery), 'ls-tree', '-r', '-z', pin + '^{commit}', '--', 'scripts/native'],
            check=True, capture_output=True)
        entries = result.stdout.split(b'\0')
        expected = {}
        for entry in filter(None, entries):
            meta, path = entry.split(b'\t', 1)
            mode, kind, oid = meta.split()
            if kind != b'blob' or mode not in (b'100644', b'100755'):
                raise ValueError('native pin contains a non-regular file')
            relative = Path(os.fsdecode(path)).relative_to('scripts/native')
            expected[relative] = oid.decode('ascii')
        if not expected:
            raise ValueError('native tool subtree is absent from the pinned commit')
        native = cemetery / 'scripts/native'
        if native.is_symlink():
            raise ValueError('native tool directory is a symlink')
        for relative, oid in expected.items():
            path = native / relative
            if path.is_symlink() or not path.is_file():
                raise ValueError(f'missing/non-regular native file: {relative}')
            data = path.read_bytes()
            actual = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if actual != oid:
                raise ValueError(f'native file differs from pin: {relative}')
        # Python imports may create caches; executable/source additions are not
        # part of the pinned implementation and must not be loaded implicitly.
        actual_paths = {path.relative_to(native) for path in native.rglob('*')
                        if (path.is_file() or path.is_symlink())
                        and '__pycache__' not in path.relative_to(native).parts
                        and path.suffix != '.pyc'}
        if actual_paths != set(expected):
            raise ValueError('native subtree contains files outside the pin')
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise RuntimeError(
            'Cemetery native tools do not match death-tools-pin.txt; refusing to execute. '
            'Fetch/check out the pinned commit or use a checkout with identical native files. '
            f'({error})') from error


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
    verify_native_pin(repo, cemetery)
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

#!/usr/bin/env python3
"""Verify retained R4.5 parity arms using shadow-stats@9 evidence."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess


HARNESS = Path(__file__).resolve().parent
NATIVE = '24be98732748230062882fe2b7d9601dba5e2cc0'
SHIPPED = 'dd21cc3e95f5ac0dc504d1f821f148befe356df74d4eb69b17d3b549edc4e046'
SUITES = {
    'full': '7c81c28c03be3e6c188cb555fa8c917402bae0819d0791a1a810e031214d2eed',
    'smoke': 'ee385771607f85e06257062def13fdf0bfe60773772a547280c5c0d9ec24339e',
}
ARMS = ('off', 'check', 'switch')
SHADOW_FILES = ('shadow.jsonl', 'shadow-stats.jsonl')
R5_GUIDANCE = 'For current R5 evidence, use harness/check-shadow.py --r5.'


def fields(path):
    values = {}
    for line in path.read_text().splitlines():
        key, separator, value = line.partition('=')
        if separator:
            if key in values:
                raise ValueError(f'{path}: duplicate {key}')
            values[key] = value
    return values


def require_fields(path, expected):
    values = fields(path)
    for key, value in expected.items():
        if values.get(key) != value:
            raise ValueError(f'{path}: expected {key}={value}, got {values.get(key)!r}')
    return values


def arm_receipt(root, mode, suite, names, commit):
    manifest = require_fields(root / 'manifest', {
        'suite': suite,
        'suite_sha256': SUITES[suite],
        'borg_txt_sha256': SHIPPED,
        'build_id_git_rev': commit,
        'build_id_git_dirty': 'no',
        'build_id_inputs_dirty': 'no',
    })
    for key in ('binary_sha256', 'build_id_source_sha256'):
        if not re.fullmatch(r'[0-9a-f]{64}', manifest.get(key, '')):
            raise ValueError(f'{root}/manifest: invalid {key}')
    for field, original in (('build_id_binary_sha256', 'binary_sha256'),
                            ('binary_sha256_end', 'binary_sha256'),
                            ('checkout_source_sha256', 'build_id_source_sha256')):
        if manifest.get(field) != manifest[original]:
            raise ValueError(f'{root}/manifest: {field} differs from {original}')
    actual = {p.name for p in root.iterdir() if p.is_dir()}
    if actual != names:
        raise ValueError(f'{root}: missing cells {sorted(names - actual)}, extra cells {sorted(actual - names)}')
    for filename in SHADOW_FILES:
        expected = {root / name / filename for name in names} if mode != 'off' else set()
        actual = set(root.rglob(filename))
        if actual != expected:
            missing = sorted(str(p.relative_to(root)) for p in expected - actual)
            extra = sorted(str(p.relative_to(root)) for p in actual - expected)
            raise ValueError(f'{root}: missing {filename} {missing}, extra {filename} {extra}')
    configs = {}
    for name in sorted(names):
        run = root / name
        if (run / 'run_exit').read_text().strip() != '0':
            raise ValueError(f'{run}: unsuccessful cell')
        config = require_fields(run / 'config.txt', {
            'borg_core': 'borg-rs', 'borg_core_overrides': 'on',
        })
        # The fixed native pin predates the shadow option and omits the field.
        if config.get('borg_shadow', 'off') != mode:
            raise ValueError(f'{run}: expected recorded shadow mode {mode}')
        digest = config.get('borg_txt_sha256', '')
        if not re.fullmatch(r'[0-9a-f]{64}', digest) or manifest.get(f'run_config.{name}') != f'menu {digest}':
            raise ValueError(f'{run}: effective settings hash differs from the arm manifest')
        configs[name] = digest
    return manifest['binary_sha256'], manifest['build_id_source_sha256'], configs


def validate(suite, roots, candidate):
    suite_path = HARNESS / 'suites' / suite
    if hashlib.sha256(suite_path.read_bytes()).hexdigest() != SUITES[suite]:
        raise ValueError(f'{suite_path}: frozen suite hash differs')
    names = {line.split()[0] for line in suite_path.read_text().splitlines()
             if line.strip() and not line.lstrip().startswith('#')}
    native, check, switch = [arm_receipt(root, mode, suite, names, commit)
                             for root, mode, commit in zip(roots, ARMS, (NATIVE, candidate, candidate),
                                                           strict=True)]
    if check[:2] != switch[:2]:
        raise ValueError('check and switch have different binary or build-input hashes')
    for mode, receipt in (('check', check), ('switch', switch)):
        if receipt[2] != native[2]:
            changed = sorted(name for name in names if receipt[2][name] != native[2][name])
            raise ValueError(f'{mode}: effective settings differ from native for {changed}')
    for mode, root in zip(ARMS[1:], roots[1:], strict=True):
        subprocess.run([str(HARNESS / 'check-shadow.py'), '--r45', '--expect-mode', mode, root],
                       check=True, stdout=subprocess.DEVNULL)
        print(f'{mode}: zero shadow mismatches across {len(names)} cells', flush=True)
        subprocess.run([str(HARNESS / 'diff'), '--suite', suite, '--expect-impl', 'rust',
                        roots[0], root], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=R5_GUIDANCE)
    parser.add_argument('--suite', choices=SUITES, default='full',
                        help='use smoke only for a local rehearsal')
    parser.add_argument('--candidate', required=True, help='exact candidate source commit')
    parser.add_argument('native', type=Path)
    parser.add_argument('check', type=Path)
    parser.add_argument('switch', type=Path)
    args = parser.parse_args()
    if not re.fullmatch(r'[0-9a-f]{40}', args.candidate) or args.candidate == NATIVE:
        parser.error('--candidate must be a full Git SHA distinct from the native pin')
    try:
        validate(args.suite, (args.native, args.check, args.switch), args.candidate)
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        parser.exit(1, f'R4.5 suite gate failed for shadow-stats@9 evidence: {exc}\n{R5_GUIDANCE}\n')


if __name__ == '__main__':
    main()

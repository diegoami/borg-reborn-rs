"""Fail closed if a stateful policy gains an unrecorded dependency (R2 #286)."""
import re
from pathlib import Path
from rust_ports import mask_non_code

NATIVE = ('borg_caution', 'borg_escape', 'borg_recover')
PORTS = ('caution', 'escape', 'recovery')
NATIVE_NAMESPACES = {'decision_api', 'decision_tape', 'decisions', 'shadow',
                     'params', 'borg', 'borg_cave', 'borg_update', 'state',
                     'headers', 'borg_headers', *NATIVE}
PORT_NAMESPACES = {'decision_api', 'decision_tape', 'memory', 'threat_facts'}


def inspect(text, port=False):
    """This is a closed import/path gate, not a Rust semantic analysis.

    Compilation verifies that every callable API name has a wrapper. The
    gate prevents a future carry from bypassing that API with a direct core
    call, even if the carried native code would otherwise compile.
    """
    code = mask_non_code(text)
    prefix = 'borg_rs' if port else 'crate'
    allowed = PORT_NAMESPACES if port else NATIVE_NAMESPACES
    errors = []
    for m in re.finditer(r'\b' + prefix + r'::(\w+)', code):
        if m[1] not in allowed:
            errors.append(f'unrecorded namespace {prefix}::{m[1]}')
    for m in re.finditer(r'\b' + prefix + r'::decision_tape::(\w+)\s*\(', code):
        if m[1] not in ('active', 'mark'):
            errors.append(f'policy may not drive the tape through {m[1]}')
    exact_imports = {'threat_facts': {'param','Param'}, 'memory': {'BorgState'}} if port else {
        'params': {'value','Param'}, 'borg': {'BorgTime'},
        'borg_cave': {'AUTO_MAX_X','AUTO_MAX_Y'}, 'borg_update': {'Detect'},
        'state': {'BorgState'}}
    for m in re.finditer(r'\buse\s+'+prefix+r'::(\w+)::([^;]+);',code,re.S):
        namespace,tail=m.groups()
        if namespace in exact_imports:
            names=[v.strip().split(' as ')[0].strip() for v in tail.strip().strip('{}').split(',') if v.strip()]
            if not set(names)<=exact_imports[namespace]:
                errors.append(f'unrecorded import from {namespace}')
    # Keep policy imports explicit: a grouped root import could conceal an
    # unrecorded namespace from this deliberately small path checker.
    if re.search(r'\buse\s+'+prefix+r'::\s*[{{*]',code):
        errors.append('policy must use explicit dependency namespaces')
    if re.search(r'\b(?:std|libc)::',code):
        errors.append('policy bypasses the closed dependency surface')
    calls = re.finditer(r'\b'+prefix+r'::(\w+)::([\w:]+)\s*\(',code)
    for call in calls:
        namespace,name=call.groups()
        permitted = {'decision_api','decision_tape'} if port else {'decision_api','decision_tape','shadow','decisions',*NATIVE}
        if namespace not in permitted:
            errors.append(f'unrecorded call {prefix}::{namespace}::{name}')
    if port and re.search(r'\.tr\s*\(', code):
        errors.append('port reads an unnamed native trait')
    if re.search(r'\b(?:unsafe|extern)\b', code):
        errors.append('policy bypasses the safe dependency boundary')
    return errors


def check(root):
    root = Path(root)
    for directory, names, port in [('borg-core/src', NATIVE, False),
                                    ('borg-next/src/stateful', PORTS, True)]:
        for name in names:
            path = root / directory / (name + '.rs')
            errors = inspect(path.read_text(), port)
            if errors:
                raise ValueError(f'{path}: ' + '; '.join(errors))
    return len(NATIVE) + len(PORTS)


if __name__ == '__main__':
    print(f'{check(Path(__file__).resolve().parents[1])} stateful policy files have a closed dependency surface')

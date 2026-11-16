"""Syntactic stateful dependency checks; manual Rust review remains required."""
import re
from pathlib import Path
from rust_ports import mask_non_code

NATIVE = ('borg_caution', 'borg_escape', 'borg_recover', 'borg_fight_attack', 'borg_fight_defend')
PORTS = ('caution', 'escape', 'recovery', 'attack', 'defence')
NATIVE_NAMESPACES = {'decision_api', 'decision_tape', 'decisions', 'shadow',
                     'params', 'borg', 'borg_cave', 'borg_update', 'state',
                     'headers', 'borg_headers', *NATIVE}
PARENT_ESCAPE = {'borg_caution_phase', 'borg_escape', 'borg_freedom',
                 'borg_recall', 'borg_surrounded'}
PORT_NAMESPACES = {'decision_api', 'decision_tape', 'memory', 'threat_facts'}


# Exact audited code exceptions, never a file-wide exemption. Changing these
# requires reviewing the dispatch types / sole native BI write explicitly.
DISPATCH_IMPORT = 'use borg_rs::{decisions::{Decision,Value},memory::BorgState,};'
MANA_ADAPTER = ('fn set_mana(st:&mut BorgState,value:i32){'
                'st.borg.trait_[borg_rs::memory::traits::BI_CURSP as usize]=value;}')


DEFENCE_ADAPTERS = tuple(
    'fn '+name+'(st:&mut BorgState,value:i32){'
    'st.borg.trait_[borg_rs::memory::traits::'+trait+' as usize]=value;}'
    for trait,name in [('BI_RCONF','set_confusion_resistance'),('BI_FRACT','set_free_action')])


def inspect(text, port=False, dispatch=False, policy=None):
    """Catch known import/path bypasses, not arbitrary Rust semantics.

    Compilation checks API names/types; manual review must still inspect
    fields, methods, macros and every carried dependency.
    """
    code = re.sub(r'\s*::\s*', '::', mask_non_code(text))
    # Raw identifier spelling must not hide the same dependency/method.
    code = re.sub(r'\br#(?=\w)', '', code)
    code = re.sub(r'\buse\s*::', 'use ', code)
    if port and dispatch:
        # Normalize whitespace only for matching the exact adapter/import.
        # Remove just these snippets, then inspect all remaining code.
        for snippet in (DISPATCH_IMPORT, MANA_ADAPTER, *DEFENCE_ADAPTERS):
            pattern = r'\s*'.join(re.escape(token) for token in
                                     re.findall(r'\w+|[^\w\s]', snippet))
            code = re.sub(pattern, '', code, count=1)
    if port:
        errors = []
        if policy in ('attack', 'defence'):
            module = 'borg_fight_attack' if policy == 'attack' else 'borg_fight_defend'
            if re.search(r'\bborg_rs::decision_api::'+module+'::', code):
                errors.append(policy+' may not delegate its own policy through a shared native result')
        if re.search(r'\buse\s*(?:\{|crate::\s*[{{*])', code):
            errors.append('policy must not hide local/core paths in a grouped root import')
        # A parent may expose private root imports and Decision::native.
        # Remove only the audited local escape import and setter call prefix.
        def local_escape(match):
            names = [name.strip() for name in match[1].split(',') if name.strip()]
            if names and set(names) <= PARENT_ESCAPE:
                return ''
            return match[0]
        code = re.sub(r'\buse\s+super::escape::\{([^}]+)\}\s*;', local_escape, code)
        code = re.sub(r'\bsuper::(?:set_mana|set_confusion_resistance|set_free_action)\s*(?=\()', '', code)
        if re.search(r'\b(?:super|self)::', code):
            errors.append('policy bypasses the audited parent dependency paths')
        if re.search(r'\buse\s+(?:crate|super|self)\s*(?:as\b|;)', code):
            errors.append('policy may not alias a local crate/module')
        if re.search(r'(?:\.\s*|::)native\b', code):
            errors.append('port may not delegate native policy')
        if re.search(r'\binclude(?:_str|_bytes)?\s*!|#\s*!?\s*\[[^\]]*\bpath\s*=', code):
            errors.append('policy may not include uninspected source files')
        if re.search(r'\buse\s+borg_rs\s*(?:as\b|;)', code):
            errors.append('policy may not alias/import the core crate')
        for match in re.finditer(r'\buse\s+borg_rs\s*::([^;]+);', code):
            if re.search(r'\bas\b', re.sub(r'\bparam\s+as\s+p\b', 'param', match[1])):
                errors.append('policy may not alias core dependencies')
        if re.search(r'\btrait_\b|\.\s*tr\s*\(', code):
            errors.append('port reads or writes an unnamed native trait')
        # The full memory/type path is restricted even outside use statements.
        for match in re.finditer(r'\bborg_rs\s*::\s*(memory|threat_facts|decisions)\s*::\s*(\w+)', code):
            allowed_names = {'memory': {'BorgState'},
                             'threat_facts': {'param', 'Param'},
                             'decisions': {'Decision', 'Value'} if dispatch else set()}
            if match[2] not in allowed_names[match[1]]:
                errors.append(f'unrecorded path borg_rs::{match[1]}::{match[2]}')
        if re.search(r'\buse\s+borg_rs\s*::\s*(?:memory|threat_facts|decision_tape|decisions)\s*;', code):
            errors.append('policy may not import a dependency namespace')
        for match in re.finditer(r'\bcrate\s*::\s*(\w+)', code):
            if match[1] != 'Read':
                errors.append(f'unrecorded local namespace crate::{match[1]}')
    else:
        errors = []
    code = re.sub(r'\s*::\s*', '::', code)
    prefix = 'borg_rs' if port else 'crate'
    allowed = (PORT_NAMESPACES | ({'decisions'} if dispatch else set())) if port else NATIVE_NAMESPACES
    for m in re.finditer(r'\b' + prefix + r'::(\w+)', code):
        if m[1] not in allowed:
            errors.append(f'unrecorded namespace {prefix}::{m[1]}')
    for m in re.finditer(r'\b' + prefix + r'::decision_tape::(\w+)\s*\(', code):
        if m[1] not in ('active', 'mark'):
            errors.append(f'policy may not drive the tape through {m[1]}')
    exact_imports = {'threat_facts': {'param','Param'}, 'memory': {'BorgState'},
                     'decisions': {'Decision', 'Value'} if dispatch else set()} if port else {
        'params': {'value','Param'}, 'borg': {'BorgTime'},
        'borg_cave': {'AUTO_MAX_X','AUTO_MAX_Y','BorgGrid','DUNGEON_HGT','DUNGEON_WID'}, 'borg_update': {'Detect'},
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
    if re.search(r'\b(?:unsafe|extern)\b', code):
        errors.append('policy bypasses the safe dependency boundary')
    return errors


def check(root):
    root = Path(root)
    paths = [(root / 'borg-core/src' / (name + '.rs'), False)
             for name in NATIVE]
    directory = root / 'borg-next/src/stateful'
    # Required roots must exist; discover all additional Rust modules too.
    for name in (*PORTS, 'mod'):
        if not (directory / (name + '.rs')).is_file():
            raise ValueError(f'missing stateful module {name}')
    paths += [(path, True) for path in sorted(directory.rglob('*.rs'))]
    for path, port in paths:
        errors = inspect(path.read_text(), port,
                         dispatch=port and path == directory / 'mod.rs',
                         policy=next((name for name in ('attack','defence') if port and
                             (path.stem == name or name in path.relative_to(directory).parts[:-1])),None))
        if errors:
            raise ValueError(f'{path}: ' + '; '.join(errors))
    return len(paths)


if __name__ == '__main__':
    print(f'{check(Path(__file__).resolve().parents[1])} stateful files pass syntactic dependency checks (manual review required)')

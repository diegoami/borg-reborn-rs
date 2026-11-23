#!/usr/bin/env python3
"""Compare carried item-use function bodies after declared seam normalization.

This checks source correspondence, not runtime parity or arbitrary Rust semantics.
"""
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from rust_ports import functions, mask_non_code

ROOT = Path(__file__).resolve().parents[1]


def audit(root=ROOT):
    root = Path(root)
    manifest = json.loads((root / 'docs/R1038-ITEM-USE-SOURCE.json').read_text())
    patch = root / 'docs/R1038-ITEM-USE-SEAMS.patch'
    if hashlib.sha256(patch.read_bytes()).hexdigest() != manifest['native_seams_sha256']:
        raise ValueError('item-use seam hash differs')
    with tempfile.TemporaryDirectory() as directory:
        target = Path(directory) / manifest['receiving_path']
        source = root / manifest['receiving_path']
        if source.is_symlink():
            raise ValueError('item-use native source symlink')
        target.parent.mkdir(parents=True)
        shutil.copyfile(source, target)
        result = subprocess.run(['git', 'apply', '--reverse', str(patch.resolve())], cwd=directory, capture_output=True)
        if result.returncode:
            raise ValueError('cannot reverse item-use seams')
        if hashlib.sha256(target.read_bytes()).hexdigest() != manifest['receiving_original_sha256']:
            raise ValueError('accepted item-use baseline differs')
        accepted_text = target.read_text()
    native_path = root / 'borg-core/src/borg_item_use.rs'
    port_path = root / 'borg-next/src/stateful/item_use.rs'
    native_text, port_text = native_path.read_text(), port_path.read_text()
    native = functions(native_text.split('// Test-only defence fixture entry.', 1)[0].split('#[cfg(test)]', 1)[0])
    port = functions(port_text.split('// Test-only defence fixture entry.', 1)[0])
    traits = {}
    declaration = (root / 'borg-next/src/lib.rs').read_text()
    for _, _, field, body in re.findall(
            r'trait_group!\((\w+),\s*(\w+),\s*(\w+)\s*\{(.*?)\}\);',
            declaration, re.S):
        for name, index in re.findall(r'(\w+)\s*=>\s*(BI_\w+)', body):
            traits[field + '().' + name + '()'] = index

    def remove_markers(text):
        return re.sub(
            r'if\s+(?:borg_rs|crate)::decision_tape::active\(\)\s*\{\s*'
            r'(?:borg_rs|crate)::decision_tape::mark\([^;]+;\s*\}', '', text)

    def tokens(text):
        text = text.replace('crate::decision_api::', 'crate::')
        text = re.sub(r'\bpub(?:\(crate\))?\s+', '', text)
        text = re.sub(r'\s+', '', mask_non_code(text))
        text = re.sub(r',+(?=[)\]}])', '', text)
        text = re.sub(r'=>\{([^{};]+)\}', r'=>\1,', text)
        return re.sub(r',(?=(?:[A-Z_][A-Z_0-9]*|_)=>)', '', text)

    accepted_functions = functions(accepted_text)
    rows = []
    for name, bodies in port.items():
        assert len(bodies) == 1, name
        before = native.get(name + '_native', native.get(name))[0]
        before = before.replace(name + '_native', name)
        before = re.sub(r'st\.borg\.tr\((BI_\w+)\)',r'tr(st, \1)',before)
        after = bodies[0].replace('borg_rs::decision_api::', 'crate::decision_api::')
        for suffix, index in traits.items():
            after = after.replace('crate::Read::new(st).' + suffix, 'tr(st, ' + index + ')')
        after = after.replace('crate::decision_api::headers::', 'crate::headers::')
        after = after.replace('crate::decision_api::borg_headers::', 'crate::borg_headers::')
        before, after = remove_markers(before), remove_markers(after)
        expected = accepted_functions[name][0]
        if name == 'borg_equips_item':
            # The reviewed immutable facade strengthens this one input contract.
            expected = expected.replace('st: &mut BorgState', 'st: &BorgState')
        expected = re.sub(r'st\.borg\.tr\((BI_\w+)\)', r'tr(st, \1)', expected)
        if tokens(before) != tokens(expected):
            raise ValueError('accepted item-use baseline differs: ' + name)
        literals = lambda text: re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"', text)
        characters = lambda text: re.findall(r"(?:b)?'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", text)
        if literals(before) != literals(expected) or characters(before) != characters(expected):
            raise ValueError('accepted item-use literals differ: ' + name)
        row = dict(function=name, normalized_tokens_equal=tokens(before) == tokens(after),
                   quoted_literals_equal=literals(before) == literals(after),
                   character_literals_equal=characters(before) == characters(after))
        if not (row['normalized_tokens_equal'] and row['quoted_literals_equal'] and row['character_literals_equal']):
            raise ValueError('item-use correspondence differs: ' + name)
        rows.append(row)
    expected = set(native) - {'default', 'tr'} - {n + '_native' for n in port}
    expected |= {n for n in port if n + '_native' in native}
    assert set(port) == expected, (set(port) - expected, expected - set(port))
    def declarations(text):
        code = mask_non_code(text)
        found = {}
        for match in re.finditer(r'^(const|enum)\s+(\w+)', code, re.M):
            depth = 0
            opened = False
            for end in range(match.end(), len(code)):
                char = code[end]
                if char in '([{':
                    depth += 1
                    opened = True
                elif char in ')]}':
                    depth -= 1
                if depth == 0 and (char == ';' or
                        (match[1] == 'enum' and opened and char == '}')):
                    found[match[2]] = text[match.start():end + 1]
                    break
        return found
    native_declarations, port_declarations = declarations(native_text), declarations(port_text)
    accepted_declarations = declarations(accepted_text)
    if accepted_declarations.keys() != native_declarations.keys():
        raise ValueError('accepted item-use module declarations differ')
    for name in accepted_declarations:
        if tokens(accepted_declarations[name]) != tokens(native_declarations[name]):
            raise ValueError('accepted item-use declaration differs: ' + name)
    assert native_declarations.keys() == port_declarations.keys()
    for name in native_declarations:
        before, after = native_declarations[name], port_declarations[name]
        assert tokens(before) == tokens(after), name
        assert re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"', before) == re.findall(
            r'(?:b)?"(?:[^"\\]|\\.)*"', after), name
    sha = lambda text: hashlib.sha256(text.encode()).hexdigest()
    return dict(
        schema='borg-reborn-rs/r435-source-audit@1',
        native_sha256=sha(native_text), port_sha256=sha(port_text),
        scope='All 37 policy function bodies and module constants; imports and memory definitions require separate review.',
        identical_declarations=sorted(native_declarations),
        normalization='Named Read projections, API type paths, entry markers, native suffix/visibility, rustfmt trailing commas and braces around a single match-arm expression only. Quoted string and character sequences compared separately.',
        functions=rows, accepted_baseline_verified=True)


if __name__ == '__main__':
    print(json.dumps(audit(), indent=2))

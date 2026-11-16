#!/usr/bin/env python3
"""Compare carried attack function bodies after declared seam normalization.

This checks source correspondence, not runtime parity or arbitrary Rust semantics.
"""
import hashlib
import json
import re
from pathlib import Path
from rust_ports import functions, mask_non_code

ROOT = Path(__file__).resolve().parents[1]


def main():
    native_path = ROOT / 'borg-core/src/borg_fight_attack.rs'
    port_path = ROOT / 'borg-next/src/stateful/attack.rs'
    native_text, port_text = native_path.read_text(), port_path.read_text()
    native = functions(native_text.split('#[cfg(test)]', 1)[0])
    port = functions(port_text)
    traits = {}
    declaration = (ROOT / 'borg-next/src/lib.rs').read_text()
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
        text = re.sub(r'\bpub(?:\(crate\))?\s+', '', text)
        text = re.sub(r'\s+', '', mask_non_code(text))
        text = re.sub(r',+(?=[)\]}])', '', text)
        text = re.sub(r'=>\{([^{};]+)\}', r'=>\1,', text)
        return re.sub(r',(?=(?:[A-Z_][A-Z_0-9]*|_)=>)', '', text)

    rows = []
    for name, bodies in port.items():
        assert len(bodies) == 1, name
        before = native.get(name + '_native', native.get(name))[0]
        before = before.replace(name + '_native', name)
        after = bodies[0].replace('borg_rs::decision_api::', 'crate::decision_api::')
        for suffix, index in traits.items():
            after = after.replace('crate::Read::new(st).' + suffix, 'tr(st, ' + index + ')')
        after = re.sub(
            r'let simulated_mana\s*=\s*([^;]+);\s*'
            r'super::set_mana\(st,\s*simulated_mana\);',
            r'st.borg.trait_[BI_CURSP as usize] = \1;', after)
        after = re.sub(r'super::set_mana\(st,\s*([^;]+)\);',
                       r'st.borg.trait_[BI_CURSP as usize] = \1;', after)
        after = after.replace('crate::decision_api::headers::', 'crate::headers::')
        after = after.replace('crate::decision_api::borg_headers::', 'crate::borg_headers::')
        before, after = remove_markers(before), remove_markers(after)
        literals = lambda text: re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"', text)
        row = dict(function=name, normalized_tokens_equal=tokens(before) == tokens(after),
                   quoted_literals_equal=literals(before) == literals(after))
        assert row['normalized_tokens_equal'] and row['quoted_literals_equal'], name
        rows.append(row)
    expected = set(native) - {'default', 'tr'} - {n + '_native' for n in port}
    expected |= {n for n in port if n + '_native' in native}
    assert set(port) == expected, (set(port) - expected, expected - set(port))
    sha = lambda text: hashlib.sha256(text.encode()).hexdigest()
    print(json.dumps(dict(
        schema='borg-reborn-rs/r364-source-audit@1',
        native_sha256=sha(native_text), port_sha256=sha(port_text),
        scope='All 71 policy function bodies; imports, module constants and memory definitions require separate review.',
        normalization='Named Read projections, mana adapter/local temporary, API type paths, entry markers, native suffix/visibility, rustfmt trailing commas and braces around a single match-arm expression only. Quoted string sequences compared separately.',
        functions=rows), indent=2))


if __name__ == '__main__':
    main()

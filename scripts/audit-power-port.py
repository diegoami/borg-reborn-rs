#!/usr/bin/env python3
"""Verify all R434 valuation bodies against native, with explicit memory seams."""
import json
import re
from pathlib import Path
from rust_ports import functions, mask_non_code

ROOT = Path(__file__).resolve().parents[1]


def audit(root=ROOT):
    root = Path(root)
    native = (root / 'borg-core/src/borg_power.rs').read_text().split('#[cfg(test)]', 1)[0]
    port = (root / 'borg-next/src/stateful/power.rs').read_text()
    # This audit supports functions/constants, not new static/impl/macro definitions.
    for source in (native, port):
        if re.search(r'(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:static\b|impl\b|macro_rules!)', mask_non_code(source)):
            raise ValueError('unsupported power module declaration requires review')
    traits = {}
    for _, _, group, body in re.findall(r'trait_group!\((\w+),\s*(\w+),\s*(\w+)\s*\{(.*?)\}\);',
                                        (root / 'borg-next/src/lib.rs').read_text(), re.S):
        for field, index in re.findall(r'(\w+)\s*=>\s*(BI_\w+)', body):
            traits[group + '().' + field + '()'] = index
    native = native.replace('borg_power_equipment_native', 'borg_power_equipment').replace('borg_power_native', 'borg_power')
    def remove_markers(text):
        return re.sub(r'if\s+(?:crate|borg_rs)::decision_tape::active\(\)\s*\{\s*(?:crate|borg_rs)::decision_tape::mark\([^;]+;\s*\}', '', text)
    def normalize(text):
        text = remove_markers(text)
        text = re.sub(r'\bpub(?:\(crate\))?\s+', '', text)
        text = mask_non_code(text)
        # Preserve separated punctuation which whitespace removal would fuse
        # into a different Rust operator (notably && versus & &).
        compound = {'&&','||','<<','>>','<=','>=','==','!=','+=','-=','*=','/=','%=','&=','|=','^=','->','=>','::','..'}
        text = re.sub(r'([&|<>=!+*/%:^.\-])\s+(?=([&|<>=!+*/%:^.\-]))',
                      lambda m: m[1] + ('__PUNCT_GAP__' if m[1]+m[2] in compound else ''), text)
        text = re.sub(r'\s+', '', text)
        text = text.replace('lettr=|i:u32|st.borg.tr(i);','')
        text = text.replace('st.borg.tr(BI_MAXDEPTH)', 'tr(BI_MAXDEPTH)')
        text = text.replace('borg_rs::decision_api::','crate::decision_api::')
        for suffix,index in traits.items():
            text = text.replace('crate::Read::new(st).' + suffix, 'tr(' + index + ')')
        for method,expression in [('stat_index(spell_statasu32)','BI_STR_INDEX+spell_statasu32'),
                                 ('stat_adjustment(i)','BI_ASTR+i'),
                                 ('current_stat(spell_statasu32)','BI_CSTR+spell_statasu32')]:
            text=text.replace('crate::Read::new(st).'+method,'tr('+expression+')')
        text=text.replace('letspell=crate::Read::new(st).spell(si);','letspell=&st.magic.magics[si];')
        text=text.replace('crate::Read::new(st).spell(si).level','st.magic.magics[si].level')
        text = re.sub(r'=>\{([^{};]+)\}',r'=>\1,',text)
        return re.sub(r',+(?=[)\]}])', '', text)
    before, after = functions(native), functions(port)
    if before.keys() != after.keys() or len(before) != 11:
        raise ValueError('power function inventory changed')
    for name in before:
        a,b=before[name][0],after[name][0]
        literals=lambda t:re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"', remove_markers(t))
        if normalize(a) != normalize(b) or literals(a)!=literals(b):
            raise ValueError('power body/literal correspondence differs: ' + name)
    # Module constants, including slot/class indices, must also correspond.
    constants = lambda s: re.findall(r'^const\s+[^;]+;', s, re.M)
    if list(map(normalize, constants(native))) != list(map(normalize, constants(port))):
        raise ValueError('power constants differ')
    return sorted(before)


if __name__ == '__main__':
    print(json.dumps({'schema': 'borg-reborn-rs/r434-source-audit@1',
                      'functions': audit(), 'scope': 'exact bodies/literals/constants after explicit API/Read/marker seams; not runtime proof'}))

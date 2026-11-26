#!/usr/bin/env python3
"""Audit exact equipment bodies and the separately reviewed native seam patch."""
import hashlib,json,re,shutil,subprocess,tempfile
from pathlib import Path
from rust_ports import functions,mask_non_code
ROOT=Path(__file__).resolve().parents[1]


def audit(root=ROOT):
    root=Path(root)
    manifest=json.loads((root/'docs/R436-SOURCE.json').read_text())
    patch=root/'docs/R436-NATIVE-SEAMS.patch'
    if hashlib.sha256(patch.read_bytes()).hexdigest()!=manifest['native_seams_sha256']:
        raise ValueError('equipment native seam patch changed')
    with tempfile.TemporaryDirectory() as directory:
        temp=Path(directory)
        for name in manifest['receiving_original_sha256']:
            p=root/name
            if p.is_symlink():raise ValueError('equipment source symlink')
            target=temp/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target)
        result=subprocess.run(['git','apply','--reverse',str(patch.resolve())],cwd=temp,capture_output=True)
        if result.returncode:raise ValueError('cannot reverse equipment native seams')
        for name,expected in manifest['receiving_original_sha256'].items():
            if hashlib.sha256((temp/name).read_bytes()).hexdigest()!=expected:
                raise ValueError('native policy outside audited equipment seams: '+name)
    traits={}
    for _,_,group,body in re.findall(r'trait_group!\((\w+),\s*(\w+),\s*(\w+)\s*\{(.*?)\}\);',(root/'borg-next/src/lib.rs').read_text(),re.S):
        for field,index in re.findall(r'(\w+)\s*=>\s*(BI_\w+)',body):traits[group+'().'+field+'()']=index
    def markers(t):
        return re.sub(r'if\s+(?:borg_rs|crate)::decision_tape::active\(\)\s*\{\s*(?:borg_rs|crate)::decision_tape::mark\([^;]+;\s*\}','',t)
    def normalize(t):
        t=markers(t)
        t=re.sub(r'\bpub(?:\(crate\))?\s+','',t)
        t=re.sub(r'\s+','',mask_non_code(t))
        t=t.replace('let t=|i|st.borg.tr(i);'.replace(' ',''),'')
        t=t.replace('borg_rs::decision_api::','crate::decision_api::')
        for suffix,index in traits.items():t=t.replace('crate::Read::new(st).'+suffix,'st.borg.tr('+index+')')
        t=re.sub(r'\bt\((BI_\w+)\)',r'st.borg.tr(\1)',t)
        t=t.replace('crate::borg_item_id::borg_item_note_needs_id','borg_item_note_needs_id')
        t=t.replace('crate::decision_api::borg_inventory::borg_wield_slot','borg_wield_slot')
        t=re.sub(r'=>\{([^{};]+)\}',r'=>\1,',t)
        return re.sub(r',+(?=[)\]}])','',t)
    count=0
    for port_name,sources in manifest['owned_bodies'].items():
        port=(root/port_name).read_text();ported=functions(port)
        if set(ported)!=set(sources):raise ValueError('equipment owned function inventory differs')
        for name,source_name in sources.items():
            native=(root/source_name).read_text().split('#[cfg(test)]',1)[0]
            found=functions(native)
            body=found.get(name+'_native',found.get(name))[0]
            body=body.replace('fn '+name+'_native(', 'fn '+name+'(')
            after=ported[name][0]
            quoted=lambda t:re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"',markers(t))
            chars=lambda t:re.findall(r"(?:b)?'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'",markers(t))
            if normalize(body)!=normalize(after) or quoted(body)!=quoted(after) or chars(body)!=chars(after):
                raise ValueError('equipment body/literal correspondence differs: '+name)
            count+=1
        native=(root/next(iter(sources.values()))).read_text().split('#[cfg(test)]',1)[0]
        constants=lambda t:re.findall(r'^const\s+[^;]+;',t,re.M)
        if list(map(normalize,constants(native)))!=list(map(normalize,constants(port))):raise ValueError('equipment constants differ')
    return count


if __name__=='__main__':print('Equipment audit:',audit(),'exact production bodies; native seam fold-back verified')

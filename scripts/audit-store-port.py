#!/usr/bin/env python3
"""R437: verify owned bodies against native and recovered accepted source."""
import hashlib,json,re,shutil,subprocess,tempfile
from pathlib import Path
from rust_ports import functions,mask_non_code
ROOT=Path(__file__).resolve().parents[1]

def audit(root=ROOT):
    root=Path(root);manifest=json.loads((root/'docs/R437-SOURCE.json').read_text())
    if hashlib.sha256((root/'docs/M13-INVENTORY-PARAMETERS.md').read_bytes()).hexdigest()!=manifest['accepted_fixed_literals_sha256']:
        raise ValueError('accepted fixed-literal classification changed')
    patch=root/'docs/R437-NATIVE-SEAMS.patch'
    if hashlib.sha256(patch.read_bytes()).hexdigest()!=manifest['native_seams_sha256']:raise ValueError('store seam patch changed')
    baseline={}
    with tempfile.TemporaryDirectory() as temporary:
        target=Path(temporary)
        for name in manifest['receiving_original_sha256']:
            source=root/name
            if source.is_symlink():raise ValueError('store source symlink')
            out=target/name;out.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,out)
        result=subprocess.run(['git','apply','--reverse',str(patch.resolve())],cwd=target,capture_output=True)
        if result.returncode:raise ValueError('cannot reverse store native seams')
        for name,wanted in manifest['receiving_original_sha256'].items():
            raw=(target/name).read_bytes()
            if hashlib.sha256(raw).hexdigest()!=wanted:raise ValueError('accepted store baseline differs: '+name)
            baseline[name]=raw.decode()
    traits={}
    for _,_,group,body in re.findall(r'trait_group!\((\w+),\s*(\w+),\s*(\w+)\s*\{(.*?)\}\);',(root/'borg-next/src/lib.rs').read_text(),re.S):
        for field,index in re.findall(r'(\w+)\s*=>\s*(BI_\w+)',body):traits[group+'().'+field+'()']=index
    def markers(text):
        return re.sub(r'if\s+(?:crate|borg_rs)::decision_tape::active\(\)\s*\{\s*(?:crate|borg_rs)::decision_tape::mark\([^;]+;\s*\}','',text)
    def normalize(text):
        text=markers(text)
        text=re.sub(r'\bpub(?:\(crate\))?\s+','',text)
        text=text.replace('borg_rs::decision_api::','crate::').replace('crate::decision_api::','crate::')
        text=text.replace('crate::borg_store_buy::reset_money_scum','reset_money_scum')
        text=text.replace('town_strategy_disabled_native(', 'town_strategy_disabled(')
        # Rust enforces constness for these pure inherited valuation/predicate inputs.
        if re.search(r'fn (?:borg_power_home(?:_aux[12])?|borg_min_item_quantity)\(', text):
            text=text.replace('st: &mut BorgState','st: &BorgState')
        text=re.sub(r'\s+','',mask_non_code(text))
        for suffix,index in traits.items():text=text.replace('crate::Read::new(st).'+suffix,'st.borg.tr('+index+')')
        text=text.replace('engine::kb_info()[tvalasusize].max_stack','engine::base_max_stack(tval)')
        text=text.replace('letb=&st.borg;','').replace('b.tr(', 'st.borg.tr(')
        text=re.sub(r'=>\{([^{};]+)\}',r'=>\1,',text)
        return re.sub(r',+(?=[)\]}])','',text)
    def literals(text):
        text=markers(text)
        return (re.findall(r'(?:b)?"(?:[^"\\]|\\.)*"',text),re.findall(r"(?:b)?'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'",text))
    def equal(a,b,label):
        if normalize(a)!=normalize(b) or literals(a)!=literals(b):raise ValueError('store body/literal correspondence differs: '+label)
    count=0
    for port_name,sources in manifest['owned_bodies'].items():
        port=(root/port_name).read_text();ported=functions(port)
        if set(ported)!=set(sources):raise ValueError('store owned inventory differs: '+port_name)
        for name,source_name in sources.items():
            current=functions((root/source_name).read_text().split('#[cfg(test)]',1)[0])
            native=current.get(name+'_native',current.get(name))[0].replace('fn '+name+'_native(','fn '+name+'(')
            expected=functions(baseline[source_name].split('#[cfg(test)]',1)[0])[name][0]
            equal(native,expected,name+' accepted-to-native')
            equal(native,ported[name][0],name+' native-to-owned')
            count+=1
        source_name=next(iter(sources.values()))
        consts=lambda t:re.findall(r'^(?:const|type)\s+[^;]+;',t,re.M)
        if list(map(normalize,consts(port)))!=list(map(normalize,consts(baseline[source_name]))):raise ValueError('store constants/types changed: '+port_name)
    return count

if __name__=='__main__':print('Store audit:',audit(),'exact production bodies; accepted source and native seams verified')

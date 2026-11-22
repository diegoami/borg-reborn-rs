#!/usr/bin/env python3
"""Verify the immutable R438 payload inventory; never infer publication from local files."""
import argparse,hashlib,importlib.util,json,subprocess
from pathlib import Path,PurePosixPath

def verify_payload(root, manifest_sha256):
 root=Path(root).resolve();manifest=root/'manifest.json'
 if hashlib.sha256(manifest.read_bytes()).hexdigest()!=manifest_sha256:raise ValueError('manifest hash differs')
 value=json.loads(manifest.read_text())
 if value['schema']!='borg-reborn-rs/r438-prepared-evidence@1':raise ValueError('unsupported manifest')
 seen=set()
 for row in value['files']:
  rel=PurePosixPath(row['path'])
  if rel.is_absolute() or '..' in rel.parts or str(rel) in seen:raise ValueError('unsafe or duplicate payload path')
  seen.add(str(rel));p=root.joinpath(*rel.parts)
  if p.is_symlink() or not p.resolve().is_relative_to(root):raise ValueError('payload symlink escape')
  if p.stat().st_size!=row['bytes'] or hashlib.sha256(p.read_bytes()).hexdigest()!=row['sha256']:raise ValueError('payload differs: '+str(rel))
 originals=json.loads((root/'qualified-originals.json').read_text())
 if len(originals)!=value['qualified_originals']:raise ValueError('qualified original count differs')
 for entry in originals:
  rel=PurePosixPath(entry['qualified_package'])
  if rel.is_absolute() or '..' in rel.parts:raise ValueError('unsafe qualified path')
  record=root.joinpath(*rel.parts)/'.source-record.json'
  if hashlib.sha256(record.read_bytes()).hexdigest()!=entry['record_sha256']:raise ValueError('original record differs')
 return {'files':len(seen),'qualified_originals':len(originals)}

if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--cemetery',type=Path,required=True);parser.add_argument('--receipt',type=Path,required=True);args=parser.parse_args()
 receipt=json.loads(args.receipt.read_text());actual=subprocess.check_output(['git','rev-parse','HEAD'],cwd=args.cemetery,text=True).strip()
 if actual!=receipt['commit']:raise SystemExit('cemetery checkout differs from immutable receipt')
 result=verify_payload(args.cemetery/receipt['root'],receipt['manifest_sha256'])
 identity_path=args.cemetery/'scripts/native/death_identity.py'
 expected=subprocess.check_output(['git','show',receipt['commit']+':scripts/native/death_identity.py'],cwd=args.cemetery)
 if identity_path.read_bytes()!=expected:raise SystemExit('canonical identity implementation differs from pin')
 spec=importlib.util.spec_from_file_location('r438_canonical_identity',identity_path)
 identity=importlib.util.module_from_spec(spec);spec.loader.exec_module(identity)
 aliases=json.loads((args.cemetery/'deaths/ALIASES.json').read_text())['records']
 prefix=receipt['root']+'/'
 records=list((args.cemetery/receipt['root']/'archive').glob('*/record.json'))
 for record in records:
  value=identity.ensure(record.parent,check=True)
  relative=str(record.parent.relative_to(args.cemetery))
  rows=[r for r in aliases if r['path']==relative]
  if len(rows)!=1 or rows[0]['id']!=value['id'] or rows[0]['record_sha256']!=value['record_sha256']:raise SystemExit('canonical alias differs')
 if len(records)!=receipt['qualified_originals'] or result['files']!=receipt['payload_files']:raise SystemExit('receipt inventory differs')
 result['canonical_packages']=len(records)
 print(json.dumps(result))

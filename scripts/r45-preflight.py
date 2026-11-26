#!/usr/bin/env python3
"""Measure the fixed R4.5 carry scope without changing runtime inputs."""
import argparse,hashlib,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
R4='40cbaf272193abb44dd80fad2cfeec24b627be22'
TARGET='24be98732748230062882fe2b7d9601dba5e2cc0'
HISTORICAL='2f74e1c5d57ce2b41ded1b5b694174b91b51f265'
R23={'borg_danger.rs','borg_caution.rs','borg_escape.rs','borg_recover.rs','borg_fight_attack.rs','borg_fight_defend.rs'}
R4FILES={'borg_power.rs','borg_item_use.rs','borg_item_wear.rs','borg_item_id.rs','borg_home_power.rs','borg_store_buy.rs','borg_store_sell.rs','borg_think_store.rs'}
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True)
def source(ref,path):return git('show',ref+':'+path).strip()
def measure():
 out={'schema':'borg-reborn-rs/r45-preflight@1','accepted_r4':R4,'owner_named_main':TARGET,'historical_common_base':HISTORICAL,'p6_promotion':'f57499d09ba30e84c2e06e5ac08a008c856f9a6d','unpromoted_track_p_excluded':True,'comparisons':{}}
 for label,base in [('historical_to_target',HISTORICAL),('r4_to_target',R4)]:
  rows=[]
  for line in git('diff','--numstat',base,TARGET,'--','borg-core/src').splitlines():
   added,removed,name=line.split('\t',2);leaf=Path(name).name
   rows.append({'path':name,'added':int(added),'removed':int(removed),'scope':'R2/R3 owned primary' if leaf in R23 else 'R4 owned primary' if leaf in R4FILES else 'shared or partial; dependency audit required'})
  out['comparisons'][label]={'files':len(rows),'added':sum(r['added'] for r in rows),'removed':sum(r['removed'] for r in rows),'rows':rows}
 out['new_native_files']=git('diff','--name-only','--diff-filter=A',HISTORICAL,TARGET,'--','borg-core/src').splitlines()
 out['pins']={ref:{path:source(sha,path) for path in ['engine/UPSTREAM','borg-core/PIN','recorder-pin.json']} for ref,sha in [('r4',R4),('target',TARGET)]}
 out['shipped_settings']={ref:{'sha256':hashlib.sha256(git('show',sha+':harness/borg.txt').encode()).hexdigest(),'bytes':len(git('show',sha+':harness/borg.txt').encode())} for ref,sha in [('r4',R4),('target',TARGET)]}
 out['limits']='Direct source counts include shadow seams in R4-to-target deltas; they are not policy-only or port-effort counts. No carry implementation, receiving merge, runtime build, parity, acceptance of unpromoted P work or R5 start is claimed.'
 return out
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');a=p.parse_args();v=measure();dest=ROOT/'docs/R45-PREFLIGHT.json'
 if a.write:dest.write_text(json.dumps(v,indent=2)+'\n')
 else:assert json.loads(dest.read_text())==v,'preflight differs; inspect exact pins before explicit --write'
 print(json.dumps({k:{x:y for x,y in v['comparisons'][k].items() if x!='rows'} for k in v['comparisons']}))

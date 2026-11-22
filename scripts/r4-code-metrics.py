#!/usr/bin/env python3
"""Measure the accepted combined R4 source; no gameplay or timing inference."""
import argparse,hashlib,json
from pathlib import Path
from rust_ports import functions
ROOT=Path(__file__).resolve().parents[1]
FILES=['power','item_use','equipment_wear','equipment_identify','home_power','store_buy','store_sell','think_store']
def measure():
 rows=[]
 for name in FILES:
  p=ROOT/'borg-next/src/stateful'/f'{name}.rs';text=p.read_text();production=text.split('#[cfg(test)]',1)[0];found=functions(production)
  rows.append({'module':name,'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'physical_file_lines':len(text.splitlines()),'production_region_lines':len(production.splitlines()),'function_definitions':sum(len(v) for v in found.values()),'function_definition_lines':sum(len(body.splitlines()) for defs in found.values() for body in defs)})
 return {'schema':'borg-reborn-rs/r4-code-metrics@1','accepted_integration':'dc1491b9d31972406c43e71e358842f9b411df97','modules':rows,'totals':{key:sum(r[key] for r in rows) for key in ['physical_file_lines','production_region_lines','function_definitions','function_definition_lines']},'method':'Physical lines include imports/comments/formatting. Production region ends at first cfg(test); lexical function definitions include helpers. Function definition lines include signatures and braces. File hashes pin scope. This is not branch coverage, effort, speed or survival evidence.'}
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--write',action='store_true');a=parser.parse_args();result=measure();p=ROOT/'docs/R4-CODE-METRICS.json'
 if a.write:p.write_text(json.dumps(result,indent=2)+'\n')
 else:
  assert json.loads(p.read_text())==result,'metrics differ; inspect source before explicit --write'
 print(json.dumps(result['totals']))

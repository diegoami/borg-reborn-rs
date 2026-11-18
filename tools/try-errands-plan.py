#!/usr/bin/env python3
"""Run the experimental class/level planner, outside the measured harness.
Use the shared validation lock. Output and plan bytes are preserved; no
benchmark result or normal cemetery/rerun compatibility is claimed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--plan', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--via-borg-txt', action='store_true', help='select user/errands_plan.txt through borg_plan in borg.txt')
    p.add_argument('--seed', type=int, default=1010)
    p.add_argument('--race', type=int, default=0)
    p.add_argument('--class', dest='class_index', type=int, default=0)
    p.add_argument('--turns', type=int, default=3000)
    p.add_argument('--timeout', type=int, default=30)
    p.add_argument('--binary', type=Path, default=ROOT/'build/rust/game/angband')
    a = p.parse_args()
    if a.turns <= 0 or a.timeout <= 0: p.error('turns and timeout must be positive')
    plan = a.plan.read_bytes()
    out = a.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    for name in ['user','save','archive','panic','scores']: (out/name).mkdir()
    shutil.copy2(ROOT/'harness/borg.txt',out/'user/borg.txt')
    # #535: the scary list the shipped borg.txt names travels with it.
    for line in (out/'user/borg.txt').read_text().splitlines():
        key, _, value = line.partition('=')
        if key.strip() == 'borg_scary_list' and value.strip() not in ('', 'none'):
            shutil.copy2(ROOT/'harness'/value.strip(), out/'user'/value.strip())
    source = out/'errands_plan.input.txt';source.write_bytes(plan)
    exe = a.binary.resolve()
    env = {k:v for k,v in os.environ.items() if not k.startswith(('BORG_','BORGH_','ANGBAND_'))}
    env.update(BORG_CORE='borg-rs',BORG_ERRANDS_PLAN=str(source),BORG_RUN_DIR=str(out),LANG='C.UTF-8',LC_ALL='C.UTF-8',TZ='UTC')
    if a.via_borg_txt:
        (out/'user/errands_plan.txt').write_bytes(plan)
        with (out/'user/borg.txt').open('a') as f: f.write('\nborg_plan = errands_plan.txt\n')
        env.pop('BORG_ERRANDS_PLAN',None)
    command = [str(exe),'-mborgh','-n']+[f'-d{k}={out/k}' for k in ['user','save','archive','panic','scores']]
    command += ['-uPlanPOC','--','-seed',str(a.seed),'-race',str(a.race),'-class',str(a.class_index),'-menu-letters','-turns',str(a.turns),'-keylog',str(out/'keys.log'),'-msglog',str(out/'messages.log'),'-statelog',str(out/'state.log'),'-summary',str(out/'summary.json')]
    record={'kind':'experimental-plan-poc-not-a-benchmark','command':command,'environment':{k:v for k,v in env.items() if k.startswith('BORG_')},'binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'plan_sha256':hashlib.sha256(plan).hexdigest()}
    with (out/'console.log').open('w') as log:
        try:
            r=subprocess.run(command,cwd=exe.parent,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=a.timeout)
            record['exit_code']=r.returncode
        except subprocess.TimeoutExpired:
            record['timeout']=True
            (out/'poc.json').write_text(json.dumps(record,indent=2)+'\n')
            raise SystemExit('POC timed out; evidence retained in '+str(out))
    (out/'poc.json').write_text(json.dumps(record,indent=2)+'\n')
    if r.returncode == 0 and not (out/'errands_plan.txt').is_file():
        raise SystemExit('binary did not acknowledge plan support; inspect '+str(out))
    print(out)
    raise SystemExit(r.returncode)

if __name__=='__main__': main()

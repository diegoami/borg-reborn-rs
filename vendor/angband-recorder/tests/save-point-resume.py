#!/usr/bin/env python3
"""Load a retained native point in a fresh process and verify its new future.
This is a branch test, not equality with the parent's arbitrary suffix.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import time
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tests'))
import importlib.util
spec = importlib.util.spec_from_file_location('round_trip', ROOT / 'tests/round-trip.py')
round_trip = importlib.util.module_from_spec(spec)
spec.loader.exec_module(round_trip)

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--proof', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--display', default=os.environ.get('DISPLAY', ':0'))
a = p.parse_args()
proof = a.proof.resolve()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
receipt = json.loads((proof / 'result.json').read_text())
binary = proof / 'build/game/angband'
round_trip.require(receipt['source_identity'] == 'verified export+patch', 'unverified parent source')
round_trip.require(hashlib.sha256(binary.read_bytes()).hexdigest() == receipt['binary_sha256'], 'parent executable hash changed')
points = json.loads((proof / 'record/checkpoints/index.json').read_text())['saves']
point = next(s for s in points if s['turn'] == 41)
for name in ['user','save','archive','panic','scores']:
    (out / 'profile' / name).mkdir(parents=True)
shutil.copy2(proof / 'record/checkpoints' / point['file'], out / 'profile/save/Resume')
cli = ROOT / 'target/debug/angband-recorder'
args = [binary, '-mx11']
args += [f'-d{name}={out / "profile" / name}' for name in ['user','save','archive','panic','scores']]
args += ['-uResume']
env = {**os.environ, 'DISPLAY':a.display, 'ANGBAND_SAVE_WINDOW':'3', 'ANGBAND_SAVE_EVERY':'20'}
with (out / 'test.log').open('w') as log:
    def windows():
        r = subprocess.run(['xdotool','search','--name','Angband'], env=env, capture_output=True, text=True)
        return set(r.stdout.split())
    before = windows()
    child = subprocess.Popen([str(cli),'record',str(out/'record'),'--',*map(str,args)],
        cwd=binary.parent, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        deadline = time.monotonic()+15
        while time.monotonic() < deadline:
            fresh = windows()-before
            if fresh: break
            if child.poll() is not None: raise RuntimeError('recording exited before its window')
            time.sleep(.1)
        else: raise RuntimeError('recording window did not appear')
        window=sorted(fresh)[0]
        for key in ['space','Escape','comma','comma','comma','comma','ctrl+x','Escape']:
            subprocess.run(['xdotool','key','--window',window,key],env=env,capture_output=True)
            time.sleep(.2)
        child.wait(timeout=30)
        round_trip.require(child.returncode == 0, 'save-point recording failed')
    except BaseException:
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
        raise
    frames = [json.loads(l) for l in (out/'record/frames.jsonl').read_text().splitlines()]
    first = next(f for f in frames if f['phase'] == 'command')
    round_trip.require(first['turn'] == point['turn'], 'fresh birth or wrong save loaded')
    round_trip.require(frames[-1]['turn'] > point['turn'], 'branch did not advance')
    round_trip.run([cli,'replay','--verify',out/'record',out/'replay'], env=env, log=log, timeout=30)
result = {'source_proof':str(proof),'binary_sha256':receipt['binary_sha256'],
          'parent_save_point':point, 'first_loaded_turn':first['turn'],
          'last_turn':frames[-1]['turn'], 'native_save_branch_replay':'PASS',
          'parent_suffix_equality_claimed':False,'bot_memory_restored':False,
          'independent_life':False}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))

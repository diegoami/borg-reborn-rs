#!/usr/bin/env python3
"""M1 F2/F7/F10: native narrowing, actual executable identity and calendar context."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT=Path(__file__).resolve().parents[1]
SOURCE=r'''
#include "recorder.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
int ar_host_save(const char *path) { (void)path; return -1; }
const char *ar_host_extra(bool full) { (void)full; return "{}"; }
void ar_host_free_extra(const char *text) { (void)text; }
int main(void) {
    int64_t geometry[5]={80,24,256,0,0};
    ar_early_init("abi-review-control"); ar_init("abi-review-control",geometry);
    ar_input input={0};input.take=1;input.event_type=1;input.code=32;
    input.width=80;input.height=24;input.mapped=1;ar_poll(&input);
    if (input.code!=32) return 3;
    time_t recorded=time(NULL), christmas_boundary=1608811200;
    struct tm *date=localtime(&christmas_boundary);char message[128];
    if (!date) return 4;
    snprintf(message,sizeof(message),"%d-%02d-%02d:%ld",date->tm_year+1900,date->tm_mon+1,date->tm_mday,(long)recorded);
    ar_message(1,0,message);ar_finish();return 0;
}
'''

def require(value,message):
    if not value:raise RuntimeError(message)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    with open('/tmp/angband-validation.lock','a') as lock,(out/'test.log').open('w') as log:
        fcntl.flock(lock,fcntl.LOCK_EX)
        subprocess.run(['cargo','build','--release','--locked','--features','ffi','--manifest-path',str(ROOT/'Cargo.toml')],check=True,stdout=log,stderr=subprocess.STDOUT)
        source=out/'host.c';source.write_text(SOURCE)
        for mode,flags in [('release',['-O2']),('debug',['-O0','-g'])]:
            subprocess.run(['cc',*flags,'-I'+str(ROOT/'include'),str(source),str(ROOT/'target/release/libangband_recorder.a'),'-Wl,--wrap=time','-lpthread','-ldl','-lm','-o',str(out/mode)],check=True,stdout=log,stderr=subprocess.STDOUT)
        def run(binary,name,original=None,tz='UTC0'):
            env=dict(os.environ)
            for key in ['BORG_RECORD_DIR','BORG_REPLAY_DIR','ANGBAND_RECORD_DIR','ANGBAND_REPLAY_DIR']:
                env.pop(key,None)
            env.update(ANGBAND_RECORD_DIR=str(out/name),TZ=tz)
            if original:env['ANGBAND_REPLAY_DIR']=str(original)
            result=subprocess.run([str(out/binary)],env=env,capture_output=True,text=True,timeout=15)
            (out/(name+'.log')).write_text(result.stdout+result.stderr)
            return result
        require(run('release','record',tz='Pacific/Kiritimati').returncode==0,'native capture failed')
        text=json.loads((out/'record/messages.jsonl').read_text())['text']
        require(text.startswith('2020-12-25:'),'calendar boundary did not exercise the selected timezone')
        require(run('release','replay',out/'record',tz='UTC0').returncode==0,'captured calendar context did not replay')
        subprocess.run(['cargo','run','--quiet','--locked','--manifest-path',str(ROOT/'Cargo.toml'),'--','verify',str(out/'record'),str(out/'replay')],check=True,stdout=log,stderr=subprocess.STDOUT)
        # F24: a named zone that only resolves through TZDIR. Recording with
        # TZDIR=/nonexistent seals the unresolvable POSIX name; replay must
        # restore the captured TZDIR (staying on the recorded UTC reading)
        # rather than silently resolving through the ambient zone directory.
        env=dict(os.environ)
        for key in ['BORG_RECORD_DIR','BORG_REPLAY_DIR','ANGBAND_RECORD_DIR','ANGBAND_REPLAY_DIR']:env.pop(key,None)
        env.update(ANGBAND_RECORD_DIR=str(out/'tzdir-record'),TZDIR='/nonexistent',TZ='New_York')
        result=subprocess.run([str(out/'release')],env=env,capture_output=True,text=True,timeout=15)
        (out/'tzdir-record.log').write_text(result.stdout+result.stderr)
        require(result.returncode==0,'TZDIR recording failed')
        text=json.loads((out/'tzdir-record/messages.jsonl').read_text())['text']
        require(text.startswith('2020-12-24:'),'the control did not capture an unresolvable-zone reading: '+text)
        env2=dict(os.environ)
        for key in ['BORG_RECORD_DIR','BORG_REPLAY_DIR','ANGBAND_RECORD_DIR','ANGBAND_REPLAY_DIR']:env2.pop(key,None)
        env2.update(ANGBAND_RECORD_DIR=str(out/'tzdir-replay'),ANGBAND_REPLAY_DIR=str(out/'tzdir-record'),TZDIR='/usr/share/zoneinfo/America',TZ='UTC0')
        result=subprocess.run([str(out/'release')],env=env2,capture_output=True,text=True,timeout=15)
        (out/'tzdir-replay.log').write_text(result.stdout+result.stderr)
        require(result.returncode==0,'the captured TZDIR context did not replay')
        subprocess.run(['cargo','run','--quiet','--locked','--manifest-path',str(ROOT/'Cargo.toml'),'--','verify',str(out/'tzdir-record'),str(out/'tzdir-replay')],check=True,stdout=log,stderr=subprocess.STDOUT)
        require(run('debug','wrong-binary',out/'record').returncode!=0,'different executable accepted')
        require(not (out/'wrong-binary/complete.json').exists(),'wrong executable wrote a receipt')
        malformed=out/'wide-source';shutil.copytree(out/'record',malformed)
        events=[json.loads(l) for l in (malformed/'inputs.jsonl').read_text().splitlines()]
        for event in events:
            if event['kind']=='poll':event['input']['code']=2**32+32
        (malformed/'inputs.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
        row=(malformed/'events.txt').read_text().split();row[4]=str(2**32+32);(malformed/'events.txt').write_text(' '.join(row)+'\n')
        require(run('release','wide-key',malformed).returncode!=0,'wide key truncated and accepted')
        require(not (out/'wide-key/complete.json').exists(),'wide key wrote a receipt')
        result={'calendar_boundary_replayed':True,'tzdir_reapplied':True,'different_binary_rejected':True,'native_wide_key_rejected':True,
                'release_sha256':hashlib.sha256((out/'release').read_bytes()).hexdigest(),'debug_sha256':hashlib.sha256((out/'debug').read_bytes()).hexdigest()}
        (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

if __name__=='__main__':main()

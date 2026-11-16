"""Review F12: evidence checks survive Python optimization and reject absent hooks/saves."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT=Path(__file__).resolve().parent/'round-trip.py'
spec=importlib.util.spec_from_file_location('round_trip_gate',SCRIPT)
gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)

class GateControls(unittest.TestCase):
    def test_optimization_cannot_remove_the_validation(self):
        code='import importlib.util; s=importlib.util.spec_from_file_location("gate",'+repr(str(SCRIPT))+'); m=importlib.util.module_from_spec(s);s.loader.exec_module(m);m.require(False,"negative")'
        result=subprocess.run(['python3','-O','-c',code],capture_output=True,text=True)
        self.assertNotEqual(result.returncode,0);self.assertIn('negative',result.stderr)
    def test_missing_replay_save_and_missing_hook_families_fail(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)
            with self.assertRaisesRegex(RuntimeError,'no final engine save'):gate.save_hashes(root)
            # A complete-looking record: more than ten frames, an exit frame at
            # turn 81, every mandatory input family, empty and successful
            # polls, present and absent files, and native messages.
            frames=[{'index':i+1,'turn':81,'phase':'exit' if i==10 else 'command','width':2,'height':1,'full':1,'cells':[]} for i in range(11)]
            (root/'frames.jsonl').write_text(''.join(json.dumps(f)+'\n' for f in frames))
            events=[{'kind':'terminal_init','geometry':[2,1,256,0,0]},
                    {'kind':'poll','input':{'wait':1,'take':1,'result':0,'event_type':1,'code':32,'mods':0,'x':0,'y':0,'button':0,'head':0,'tail':0,'disconnect':0,'width':2,'height':1,'mapped':1}},
                    {'kind':'poll','input':{'wait':1,'take':1,'result':1,'event_type':0,'code':0,'mods':0,'x':0,'y':0,'button':0,'head':0,'tail':0,'disconnect':0,'width':2,'height':1,'mapped':1}},
                    {'kind':'clock','value':0},
                    {'kind':'exists','value':False},
                    {'kind':'file','present':True,'sha256':'a'*64},
                    {'kind':'file','present':False,'sha256':None}]
            (root/'inputs.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
            (root/'messages.jsonl').write_text(json.dumps({'frame':0,'turn':1,'type':0,'text':'hello'})+'\n')
            gate.validate_channels(root)
            # Removing one required family must fail on the family check.
            kept=[e for e in events if e['kind']!='clock']
            (root/'inputs.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in kept))
            with self.assertRaisesRegex(RuntimeError,'mandatory external hook family'):gate.validate_channels(root)

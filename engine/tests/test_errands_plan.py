"""Runtime POC checks, separate from benchmark/death-rerun identity."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[2]
GAME=ROOT/'build/rust/game/angband'
PLAN='[default]\n  level <= 5\n    loop-levels 0 2\n[mage]\n  level == 1\n    loop-levels 0 1\n  level > 1\n    loop-levels 0 2\n'
@unittest.skipUnless(GAME.is_file(),'harness/build.sh required')
class ErrandsPlanTest(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='plan-poc-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)
    def run_poc(self,name,text=PLAN,cls=0,seed=1010,turns=1000,via=False):
        source=self.root/(name+'.txt');source.write_text(text)
        out=self.root/name
        env={k:v for k,v in os.environ.items() if not k.startswith(('BORG_','BORGH_','ANGBAND_'))}
        r=subprocess.run(['python3',str(ROOT/'tools/try-errands-plan.py'),'--plan',str(source),'--out',str(out),'--class',str(cls),'--seed',str(seed),'--turns',str(turns)]+(['--via-borg-txt'] if via else []),env=env,capture_output=True,text=True,timeout=40)
        return r,out
    def test_shipped_file_loads_with_the_native_parser(self):
        text=(ROOT/'harness/plans/errands_plan.txt').read_text()
        r,out=self.run_poc('shipped',text,cls=1,seed=1001)
        self.assertEqual(r.returncode,0,r.stderr)
        self.assertEqual((out/'errands_plan.txt').read_text(),text)
        self.assertIn('class=mage',(out/'messages.log').read_text())

    def test_missing_class_is_rejected_before_core_initialization(self):
        r,out=self.run_poc('missing','[mage]\n  level <= 5\n    loop-levels 0 1\n')
        self.assertEqual(r.returncode,2)
        self.assertIn('no [warrior] or [default] plan',(out/'console.log').read_text())
        self.assertFalse((out/'errands_plan.txt').exists(), 'configuration acknowledged too early')
        self.assertEqual((out/'keys.log').stat().st_size,0)

    def test_bad_borg_txt_does_not_silently_disable_the_plan(self):
        for name,kind in [('bytes','invalid-utf8'),('directory','unreadable')]:
            out=self.root/name;out.mkdir()
            for directory in ['user','save','archive','panic','scores']:(out/directory).mkdir()
            config=out/'user/borg.txt'
            if kind=='invalid-utf8':config.write_bytes(b'borg_reborn_core = borg-rs\nborg_plan = errands_plan.txt\n\xff')
            else:config.mkdir()
            env={k:v for k,v in os.environ.items() if not k.startswith(('BORG_','BORGH_','ANGBAND_'))}
            command=[str(GAME),'-mborgh','-n']+[f'-d{k}={out/k}' for k in ['user','save','archive','panic','scores']]
            command+=['-uBadConfig','--','-seed','1010','-race','0','-class','0','-menu-letters','-turns','100','-keylog',str(out/'keys.log')]
            r=subprocess.run(command,cwd=GAME.parent,env=env,capture_output=True,text=True,timeout=10)
            self.assertEqual(r.returncode,2,(kind,r.stderr))
            self.assertIn('borg.txt',r.stderr)
            self.assertEqual((out/'keys.log').stat().st_size,0)

    def test_default_and_class_override(self):
        for name,cls,seed,route in [('warrior',0,1010,'loop-levels 0 2'),('mage',1,1001,'loop-levels 0 1')]:
            r,out=self.run_poc(name,cls=cls,seed=seed,via=(name=='mage'))
            self.assertEqual(r.returncode,0,r.stderr)
            notes=[l for l in (out/'messages.log').read_text().splitlines() if '# Plan:' in l]
            self.assertEqual(len(notes),1,notes)
            self.assertIn('class='+name,notes[0]);self.assertIn(route,notes[0])
            self.assertEqual((out/'errands_plan.txt').read_text(),PLAN)
            self.assertFalse(json.loads((out/'summary.json').read_text())['dead'])
    def test_active_loop_is_replaced_when_level_changes(self):
        r,out=self.run_poc('growth',cls=1,seed=1001,turns=20000)
        self.assertEqual(r.returncode,0,r.stderr)
        notes=[l for l in (out/'messages.log').read_text().splitlines() if '# Plan:' in l]
        self.assertTrue(any('level=1 ' in l and 'loop-levels 0 1' in l for l in notes),notes)
        self.assertTrue(any('level=2 ' in l and 'loop-levels 0 2' in l for l in notes),notes)
        self.assertFalse(json.loads((out/'summary.json').read_text())['dead'])
    def test_gap_hands_back_and_invalid_plan_fails(self):
        r,out=self.run_poc('gap','[default]\n  level > 5\n    loop-levels 0 1\n')
        self.assertEqual(r.returncode,0,r.stderr)
        self.assertEqual(json.loads((out/'summary.json').read_text())['stop_reason'],'borg_stopped')
        self.assertIn('no phase for warrior level 1',(out/'messages.log').read_text())
        r,out=self.run_poc('invalid','[mage]\n  level <= 5\n    ladder\n',cls=1)
        self.assertEqual(r.returncode,2)
        self.assertIn('supports only loop-levels',(out/'console.log').read_text())
    def test_standard_harness_refuses_experimental_plan_identity(self):
        path=self.root/'plan.txt';path.write_text(PLAN)
        env=dict(os.environ,BORG_CORE='borg-rs',BORG_ERRANDS_PLAN=str(path))
        r=subprocess.run([str(ROOT/'harness/run.sh'),str(self.root/'measured'),'1010','0','0','1000','menu'],env=env,capture_output=True,text=True,timeout=10)
        self.assertEqual(r.returncode,2,r.stderr)
        self.assertIn('diagnostic only',r.stderr)

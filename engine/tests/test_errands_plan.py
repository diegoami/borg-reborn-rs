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
    def test_shipped_plans_cover_all_classes_and_stop_at_50(self):
        lines=(ROOT/'harness/profiles/borg-rs/errands_plan.txt').read_text().splitlines()
        sections={};name=None
        for line in lines:
            if line.startswith('['): name=line[1:-1];sections[name]=[]
            elif line.startswith('  ') and not line.startswith('    '):
                sections[name].append(line.strip())
        classes={'warrior','mage','druid','priest','necromancer','paladin','rogue','ranger','blackguard'}
        self.assertEqual(set(sections),classes|{'default'})
        def matches(condition,level):
            tokens=condition.split()
            if tokens[0]=='level':
                value=int(tokens[2]);return level<=value if tokens[1]=='<=' else level==value
            return int(tokens[0])<level<=int(tokens[-1])
        for name,guards in sections.items():
            for level in range(1,50):
                self.assertEqual(sum(matches(g,level) for g in guards),1,(name,level))
            self.assertFalse(any(matches(g,50) for g in guards),name)
        self.assertNotEqual(sections['warrior'],sections['mage'])

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

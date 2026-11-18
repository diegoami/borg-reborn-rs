"""Runtime checks of errands plans: the POC driver and measured harness runs (#352)."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[2]
GAME=ROOT/'build/rust/game/angband'
PLAN='[default]\n  level <= 5\n    max-depth 2\n[mage]\n  level == 1\n    max-depth 1\n  level > 1\n    max-depth 2\n'
CLASSES=['warrior','mage','druid','priest','necromancer','paladin','rogue','ranger','blackguard']


def shipped_phases(name):
    """{section: [(low, high, max_depth)]} of a shipped plan. Strict on the
    forms the shipped files use; anything else fails, and the native parser
    loads both files in ErrandsPlanTest (#352)."""
    import re
    sections,section,pending={},None,None
    for raw in (ROOT/'harness/plans'/name).read_text().splitlines():
        if not raw.strip() or raw.lstrip().startswith('#'):continue
        if re.fullmatch(r'\[(\w+)\]',raw):
            section=raw[1:-1];sections[section]=[];continue
        m=re.fullmatch(r'  (?:level <= (\d+)|(\d+) < level <= (\d+)|level == (\d+))',raw)
        if m:
            a,b,c,d=m.groups()
            pending=(1,int(a)) if a else (int(b)+1,int(c)) if b else (int(d),int(d));continue
        m=re.fullmatch(r'    max-depth (\d+)',raw)
        if not (m and pending and section):raise ValueError(f'{name}: unexpected line {raw!r}')
        sections[section].append((*pending,int(m[1])));pending=None
    return sections


class ShippedPlansTest(unittest.TestCase):
    def phase(self,phases,level):
        hits=[p for p in phases if p[0]<=level<=p[1]]
        self.assertLessEqual(len(hits),1,level)
        return hits[0] if hits else None
    def test_every_class_has_one_phase_per_level_and_none_at_50(self):
        for name in ['errands_plan.txt','errands_plan_ambitious.txt']:
            plan=shipped_phases(name)
            self.assertEqual(set(plan),set(CLASSES)|{'default'},name)
            for section,phases in plan.items():
                for level in range(1,50):self.assertIsNotNone(self.phase(phases,level),(name,section,level))
                self.assertIsNone(self.phase(phases,50),(name,section))
    def test_the_ambitious_plan_is_never_shallower(self):
        shipped,ambitious=shipped_phases('errands_plan.txt'),shipped_phases('errands_plan_ambitious.txt')
        for section in shipped:
            for level in range(1,50):
                s,a=self.phase(shipped[section],level),self.phase(ambitious[section],level)
                self.assertGreaterEqual(a[2],s[2],(section,level,s,a))


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
        for name in ['errands_plan.txt','errands_plan_ambitious.txt']:
            text=(ROOT/'harness/plans'/name).read_text()
            r,out=self.run_poc('shipped-'+name,text,cls=1,seed=1001)
            self.assertEqual(r.returncode,0,r.stderr)
            self.assertEqual((out/'errands_plan.txt').read_text(),text)
            self.assertIn('class=mage',(out/'messages.log').read_text())

    def test_missing_class_is_rejected_before_core_initialization(self):
        r,out=self.run_poc('missing','[mage]\n  level <= 5\n    max-depth 1\n')
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
        for name,cls,seed,route in [('warrior',0,1010,'max-depth 2'),('mage',1,1001,'max-depth 1')]:
            r,out=self.run_poc(name,cls=cls,seed=seed,via=(name=='mage'))
            self.assertEqual(r.returncode,0,r.stderr)
            notes=[l for l in (out/'messages.log').read_text().splitlines() if '# Plan:' in l]
            self.assertEqual(len(notes),1,notes)
            self.assertIn('class='+name,notes[0]);self.assertIn(route,notes[0])
            self.assertEqual((out/'errands_plan.txt').read_text(),PLAN)
            self.assertFalse(json.loads((out/'summary.json').read_text())['dead'])
    def test_active_cap_is_replaced_when_level_changes(self):
        # #493: seed 1002 (seed 1001's Mage now jumps from level 1 to 3 in one
        # kill and dies of poison before the next think, with the gold reserve
        # off); the test is about the cap following the level, not the game.
        # #503: seed 1003 (with breeders no longer fled and the named scary
        # monsters off, seed 1002's Mage dies before turn 20,000).
        # #516: seed 1004 (with Boldness capped and rests no longer arming
        # Detect Invisible, seed 1003's Mage stays at level 1 until turn 20,000).
        r,out=self.run_poc('growth',cls=1,seed=1004,turns=20000)
        self.assertEqual(r.returncode,0,r.stderr)
        notes=[l for l in (out/'messages.log').read_text().splitlines() if '# Plan:' in l]
        self.assertTrue(any('level=1 ' in l and 'max-depth 1' in l for l in notes),notes)
        self.assertTrue(any('level=2 ' in l and 'max-depth 2' in l for l in notes),notes)
        self.assertFalse(json.loads((out/'summary.json').read_text())['dead'])
    def test_gap_hands_back_and_invalid_plan_fails(self):
        r,out=self.run_poc('gap','[default]\n  level > 5\n    max-depth 1\n')
        self.assertEqual(r.returncode,0,r.stderr)
        self.assertEqual(json.loads((out/'summary.json').read_text())['stop_reason'],'borg_stopped')
        self.assertIn('no phase for warrior level 1',(out/'messages.log').read_text())
        r,out=self.run_poc('invalid','[mage]\n  level <= 5\n    ladder\n',cls=1)
        self.assertEqual(r.returncode,2)
        self.assertIn('a phase is max-depth N',(out/'console.log').read_text())
    def test_max_depth_caps_the_core_without_an_errand(self):
        # A max-depth phase installs no errand: the core plays as itself,
        # and borg_prepared refuses every level below the cap.
        r,out=self.run_poc('cap','[default]\n  level <= 50\n    max-depth 1\n',seed=1010,turns=30000)
        self.assertEqual(r.returncode,0,r.stderr)
        messages=(out/'messages.log').read_text()
        self.assertIn('max-depth 1',[l for l in messages.splitlines() if '# Plan:' in l][0])
        self.assertNotIn('# Errand:',messages)
        # The core's own cap (borg_no_deeper): it may step onto the level
        # below when it leaves a level by the nearest stair, and climbs back
        # at once ("Needing to get back on correct depth"). It never stays.
        import re
        depths=[int(m[1]) for m in re.finditer(r' d(\d+)/',(out/'state.log').read_text())]
        self.assertLess(sum(d>1 for d in depths),len(depths)/50,'time below the cap')
        self.assertLessEqual(json.loads((out/'summary.json').read_text())['depth'],2,
                             'allow the accepted single-step N+1 excursion')
    def run_measured(self,name,plan=None,core='borg-rs',turns=1000):
        env={k:v for k,v in os.environ.items() if not k.startswith(('BORG_','BORGH_','ANGBAND_'))}
        if 'BORG_CEMETERY_DIR' in os.environ: env['BORG_CEMETERY_DIR']=os.environ['BORG_CEMETERY_DIR']
        env['BORG_DEATH_RECORDS_DIR']=str(self.root/'deaths')
        if core: env['BORG_CORE']=core
        if plan is not None: env['BORG_ERRANDS_PLAN']=str(plan)
        out=self.root/name
        r=subprocess.run([str(ROOT/'harness/run.sh'),str(out),'1010','0','0',str(turns),'menu'],env=env,capture_output=True,text=True,timeout=60)
        return r,out
    def config(self,out):
        return dict(l.split('=',1) for l in (out/'config.txt').read_text().splitlines() if '=' in l)

    def test_measured_harness_run_records_the_plan_identity(self):
        # #352: harness/run.sh runs plans; their bytes are in borg_txt_sha256.
        import hashlib
        plan=self.root/'plan.txt';plan.write_text(PLAN)
        same=self.root/'same-bytes-other-name.txt';same.write_text(PLAN)
        other=self.root/'other.txt';other.write_text(PLAN.replace('max-depth 2','max-depth 3',1))
        r,out=self.run_measured('plan',plan)
        self.assertEqual(r.returncode,0,r.stderr)
        sha=hashlib.sha256(PLAN.encode()).hexdigest()
        config=self.config(out)
        self.assertEqual(config['borg_errands_plan_sha256'],sha)
        self.assertEqual(config['borg_errands_plan'],str(plan))
        self.assertIn(f'errands plan sha256 {sha}\nborg_plan = errands_plan.txt\n',(out/'user/borg.txt').read_text())
        self.assertEqual((out/'errands_plan.txt').read_text(),PLAN,'the game preserves the plan it loaded')
        self.assertIn('# Plan:',(out/'messages.log').read_text())
        families={}
        for name,source in [('same',same),('other',other),('none',None)]:
            r,run=self.run_measured(name,source)
            self.assertEqual(r.returncode,0,r.stderr)
            families[name]=self.config(run)['borg_txt_sha256']
        self.assertEqual(families['same'],config['borg_txt_sha256'],'a plan is identified by its bytes, not its name')
        self.assertNotEqual(families['other'],config['borg_txt_sha256'])
        self.assertNotEqual(families['none'],config['borg_txt_sha256'])

    def test_measured_harness_refuses_unidentified_or_wrong_core_plans(self):
        plan=self.root/'plan.txt';plan.write_text(PLAN)
        for name,source,core,why in [('missing',self.root/'absent.txt','borg-rs','no such file'),
                                     ('empty','','borg-rs','no such file'),
                                     ('core',plan,None,'needs BORG_CORE=borg-rs')]:
            r,out=self.run_measured(name,source,core)
            self.assertEqual(r.returncode,2,(name,r.stderr))
            self.assertIn(why,r.stderr)
            self.assertFalse(out.exists(),name)

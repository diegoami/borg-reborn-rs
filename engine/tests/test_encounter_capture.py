"""Native command capture must leave both controllers' gameplay and RNG intact."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
GAME=ROOT/'build/rust/game/angband'


@unittest.skipUnless(GAME.is_file(),'native build required')
class EncounterCaptureTest(unittest.TestCase):
    def test_logging_preserves_streams_and_level_one_starts(self):
        base=Path(os.environ.get('BORG_CAPTURE_TEST_OUT',str(ROOT/'harness/out/encounter-capture')))
        base.mkdir(parents=True,exist_ok=True)
        root=Path(tempfile.mkdtemp(prefix='identity-',dir=base))
        env={k:v for k,v in os.environ.items() if not k.startswith(('BORG_','ANGBAND_','BORGH_'))
             and k not in ('SAVE_ROOT','CHECKPOINT_EVERY','RUN_TIMEOUT_SECONDS')}
        for key in ('BORG_CEMETERY_DIR','BORG_DEATH_RECORDS_DIR'):
            if key in os.environ:env[key]=os.environ[key]
        for core in ('reborn','borg-rs'):
            for race,cls,seed in ((4,1,1001),(0,0,101)):
                stem=f'{core}-{cls}'
                summaries=[]
                for logging in (False,True):
                    name=stem+('-on' if logging else '-off');out=root/name
                    current=dict(env,BORG_CORE=core,CHECKPOINT_EVERY='100')
                    if logging:
                        current.update(BORG_ENCOUNTER_LOG=str(out/'encounters.jsonl'),
                                       BORG_ENCOUNTER_KNOWLEDGE_LOG=str(out/'knowledge.json'))
                    result=subprocess.run(['bash',str(ROOT/'harness/run.sh'),str(out),str(seed),str(race),str(cls),'3000','menu,exact'],
                                          cwd=ROOT,env=current,capture_output=True,text=True,timeout=180)
                    self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                    summaries.append(json.loads((out/'summary.json').read_text()))
                for stream in ('keys.log','state.log','messages.log'):
                    self.assertEqual((root/(stem+'-on')/stream).read_bytes(),(root/(stem+'-off')/stream).read_bytes(),(stem,stream))
                for field in ('game_rng','dead','stop_reason','game_turns','hp','clevel','max_depth','game_options'):
                    self.assertEqual(summaries[0][field],summaries[1][field],(stem,field))
                out=root/(stem+'-on')
                frames=[json.loads(line) for line in (out/'encounters.jsonl').read_text().splitlines()]
                self.assertGreater(len(frames),1)
                self.assertEqual(frames[0]['status']['level'],1)
                self.assertIn('inventory',frames[0]);self.assertIn('equipment',frames[0])
                self.assertTrue(all({'turn','status','monsters'} <= row.keys() for row in frames))
                self.assertTrue(json.loads((out/'knowledge.json').read_text())['races'])
                if core=='borg-rs' and cls==1:
                    full=root/(stem+'-full')
                    current=dict(env,BORG_CORE=core,CHECKPOINT_EVERY='100',
                                 BORG_ENCOUNTER_LOG=str(full/'encounters.jsonl'),
                                 BORG_ENCOUNTER_KNOWLEDGE_LOG=str(full/'knowledge.json'),
                                 BORG_OBSERVATION_LOG=str(full/'observations.jsonl'))
                    result=subprocess.run(['bash',str(ROOT/'harness/run.sh'),str(full),str(seed),str(race),str(cls),'3000','menu,exact'],
                                          cwd=ROOT,env=current,capture_output=True,text=True,timeout=180)
                    self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                    def channels(rows):
                        return [(r['turn'],{k:r['status'][k] for k in ('hp','max_hp','sp','max_sp','level','depth')},
                                 [(m['index'],m['race']) for m in r['monsters']]) for r in rows]
                    full_frames=[json.loads(line) for line in (full/'encounters.jsonl').read_text().splitlines()]
                    self.assertEqual(channels(frames),channels(full_frames))
                    for stream in ('keys.log','state.log','messages.log'):
                        self.assertEqual((full/stream).read_bytes(),(out/stream).read_bytes())
                # A saved start must be observed at its original engine turn,
                # before the first played command, with full native gear data.
                checkpoint=json.loads((out/'checkpoints/index.jsonl').read_text().splitlines()[0])
                loaded=root/(stem+'-loaded')
                current=dict(env,BORG_CORE=core,SAVE_ROOT=str(out/'checkpoints'),
                             BORG_ENCOUNTER_LOG=str(loaded/'encounters.jsonl'),
                             BORG_ENCOUNTER_KNOWLEDGE_LOG=str(loaded/'knowledge.json'))
                if core=='borg-rs':current['BORG_OBSERVATION_LOG']=str(loaded/'observations.jsonl')
                result=subprocess.run(['bash',str(ROOT/'harness/run.sh'),str(loaded),str(seed),str(race),str(cls),
                                       str(checkpoint['game_turn']+100),'menu,exact,keep-load-rng,save='+checkpoint['file']],
                                      cwd=ROOT,env=current,capture_output=True,text=True,timeout=180)
                self.assertEqual(result.returncode,0,result.stdout+result.stderr)
                first=json.loads((loaded/'encounters.jsonl').read_text().splitlines()[0])
                self.assertEqual(first['turn'],checkpoint['game_turn'])
                self.assertIn('inventory',first)
                if core=='borg-rs':self.assertTrue((loaded/'observations.jsonl').is_file())

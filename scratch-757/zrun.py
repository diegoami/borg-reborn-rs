# zrun.py GAME HARNESS OUT SEED RACE CLASS TURNS [key=value ...]: one game with borg.txt overrides, as the zoo adapter runs it
import sys,os,re,subprocess,shutil,json
from pathlib import Path
game,harness,out,seed,race,cls,turns=sys.argv[1:8]
settings=dict({'borg_reborn_core':'borg-rs','borg_reborn_errand':'none'},**dict(kv.split('=',1) for kv in sys.argv[8:]))
def override(text,k,v):
    pat=re.compile(r'^[ \t]*'+re.escape(k)+r'[ \t]*=.*$',re.M)
    return pat.sub(lambda m:f'{k} = {v}',text) if pat.search(text) else text+('' if text.endswith('\n') else '\n')+f'{k} = {v}\n'
out=Path(out); shutil.rmtree(out,ignore_errors=True)
for d in ('user','save','archive','panic','scores'): (out/d).mkdir(parents=True)
t=Path(harness,'borg.txt').read_text()
for k,v in [('borg_respawn_race',race),('borg_respawn_class',cls),('borg_cheat_death','FALSE'),('borg_respawn_winners','FALSE')]+sorted(settings.items()):
    t=override(t,k,v)
(out/'user/borg.txt').write_text(t)
name=re.findall(r'^[ \t]*borg_scary_list[ \t]*=[ \t]*(\S*)[ \t]*$',t,re.M)[-1]
if name!='none': shutil.copy(Path(harness,name),out/'user'/name)
argv=[str(Path(game,'angband')),'-mborgh','-n','-uBorg']+[f'-d{d}={out/d}' for d in ('user','save','archive','panic','scores')]+['--','-seed',seed,'-race',race,'-class',cls,'-turns',turns,'-keylog',str(out/'keys.log'),'-summary',str(out/'summary.json'),'-menu-letters','-exact-turns','-statelog',str(out/'state.log'),'-msglog',str(out/'messages.log')]
env={k:v for k,v in os.environ.items() if not k.startswith('BORG_')}; env.update(TZ='UTC',LANG='C.UTF-8',LC_ALL='C.UTF-8')
rc=subprocess.run(argv,cwd=game,env=env,stdout=subprocess.DEVNULL,stderr=open(out/'stderr.log','w')).returncode
try: d=json.load(open(out/'summary.json')); print(out.name,'rc=%d'%rc,d['stop_reason'],'CL%d'%d['max_clevel'])
except Exception as e: print(out.name,'rc=%d'%rc,'NO SUMMARY',e)

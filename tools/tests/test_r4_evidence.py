"""Reject changed or escaped canonical evidence even with an honestly rebuilt manifest."""
import hashlib,importlib.util,json,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('r4_verify',Path(__file__).resolve().parents[1]/'verify-r4-evidence.py')
verify=importlib.util.module_from_spec(spec);spec.loader.exec_module(verify)
class R4EvidenceTest(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
  self.payload=self.root/'qualified-originals.json';self.payload.write_text('[]')
  self.value={'schema':'borg-reborn-rs/r438-prepared-evidence@1','qualified_originals':0,'files':[{'path':'qualified-originals.json','bytes':2,'sha256':hashlib.sha256(b'[]').hexdigest()}]}
 def seal(self):
  p=self.root/'manifest.json';p.write_text(json.dumps(self.value));return hashlib.sha256(p.read_bytes()).hexdigest()
 def test_intact_payload(self):self.assertEqual(verify.verify_payload(self.root,self.seal()),{'files':1,'qualified_originals':0})
 def test_changed_payload_rejected(self):
  seal=self.seal();self.payload.write_text('[0]')
  with self.assertRaisesRegex(ValueError,'payload differs'):verify.verify_payload(self.root,seal)
 def test_escape_rejected_even_when_manifest_declares_it(self):
  self.value['files'][0]['path']='../outside.json'
  with self.assertRaisesRegex(ValueError,'unsafe'):verify.verify_payload(self.root,self.seal())
 def test_duplicate_rejected(self):
  self.value['files']*=2
  with self.assertRaisesRegex(ValueError,'duplicate'):verify.verify_payload(self.root,self.seal())
 def test_symlink_rejected(self):
  self.payload.rename(self.root/'original.json');self.payload.symlink_to('original.json')
  with self.assertRaisesRegex(ValueError,'symlink'):verify.verify_payload(self.root,self.seal())

class R4CommandLineTest(unittest.TestCase):
 """Exercise a real Git checkout and CLI; the pinned identity service is a fixture."""
 def setUp(self):
  import subprocess
  self.subprocess=subprocess;self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
  self.outer=Path(self.temp.name);self.repo=self.outer/'cemetery';self.repo.mkdir()
  self.root=self.repo/'runs/case';self.package=self.root/'archive/package';self.package.mkdir(parents=True)
  self.original=b'{"fixture":"original"}'
  for name in ['record.json','.source-record.json']:(self.package/name).write_bytes(self.original)
  self.metadata={'id':'D-fixture','aliases':['package'],'record_sha256':hashlib.sha256(self.original).hexdigest()}
  (self.package/'death-id.json').write_text(json.dumps(self.metadata))
  self.index=[{'qualified_package':'archive/package','record_sha256':self.metadata['record_sha256']}]
  (self.repo/'scripts/native').mkdir(parents=True);self.identity=self.repo/'scripts/native/death_identity.py'
  # Dependency double: consumer tests cover pin/alias enforcement, not Cemetery's ID algorithm.
  self.identity.write_text('import json\ndef ensure(directory,check=False):\n return json.loads((directory/"death-id.json").read_text())\n')
  (self.repo/'deaths').mkdir();self.aliases=self.repo/'deaths/ALIASES.json'
  self.aliases.write_text(json.dumps({'records':[dict(self.metadata,path='runs/case/archive/package')]}))
  (self.repo/'.gitignore').write_text('__pycache__/\n')
  self.git('init','-q');self.git('config','user.name','Fixture');self.git('config','user.email','fixture@example.invalid')
  self.receipt={'root':'runs/case','payload_files':3,'qualified_originals':1}
  self.seal_commit()
 def git(self,*args):return self.subprocess.check_output(['git','-C',str(self.repo),*args],text=True,stderr=self.subprocess.DEVNULL).strip()
 def seal_commit(self):
  (self.root/'qualified-originals.json').write_text(json.dumps(self.index))
  files=[]
  for name in ['qualified-originals.json','archive/package/record.json','archive/package/.source-record.json']:
   p=self.root/name;files.append({'path':name,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
  self.manifest={'schema':'borg-reborn-rs/r438-prepared-evidence@1','qualified_originals':1,'files':files}
  self.commit_manifest()
 def commit_manifest(self):
  p=self.root/'manifest.json';p.write_text(json.dumps(self.manifest));self.receipt['manifest_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
  self.git('add','-A');self.git('commit','-qm','fixture');self.receipt['commit']=self.git('rev-parse','HEAD')
 def run_cli(self,error=None):
  receipt=self.outer/'receipt.json';receipt.write_text(json.dumps(self.receipt))
  result=self.subprocess.run(['python3',str(Path(verify.__file__).resolve()),'--cemetery',str(self.repo),'--receipt',str(receipt)],text=True,capture_output=True)
  if error is None:
   self.assertEqual(result.returncode,0,result.stderr);self.assertEqual(json.loads(result.stdout),{'files':3,'qualified_originals':1,'canonical_packages':1})
  else:self.assertNotEqual(result.returncode,0);self.assertIn(error,result.stderr)
 def test_intact_checkout(self):self.run_cli()
 def test_commit_mismatch(self):self.receipt['commit']='0'*40;self.run_cli('immutable receipt')
 def test_identity_script_drift(self):self.identity.write_text('# changed');self.run_cli('not clean')
 def test_modified_alias(self):self.aliases.write_text('{}');self.run_cli('not clean')
 def test_untracked_report(self):(self.root/'extra.json').write_text('{}');self.run_cli('not clean')
 def test_committed_undeclared_report(self):
  (self.root/'extra.json').write_text('{}');self.git('add','-A');self.git('commit','-qm','extra');self.receipt['commit']=self.git('rev-parse','HEAD');self.run_cli('undeclared')
 def test_committed_ghost_alias(self):
  a=json.loads(self.aliases.read_text());a['records'].append(dict(self.metadata,path='runs/case/archive/ghost'));self.aliases.write_text(json.dumps(a));self.git('add','-A');self.git('commit','-qm','ghost');self.receipt['commit']=self.git('rev-parse','HEAD');self.run_cli('alias inventory differs')
 def test_original_record_hash(self):self.index[0]['record_sha256']='0'*64;self.seal_commit();self.run_cli('original record differs')
 def test_receipt_count(self):self.receipt['payload_files']=99;self.run_cli('receipt inventory differs')
 def test_original_index_must_be_manifested(self):
  self.manifest['files']=[r for r in self.manifest['files'] if r['path']!='qualified-originals.json'];self.commit_manifest();self.run_cli('index is not sealed')
 def test_qualified_package_escape(self):self.index[0]['qualified_package']='../outside';self.seal_commit();self.run_cli('unsafe')
 def test_qualified_package_symlink(self):
  (self.root/'archive/link').symlink_to('package',target_is_directory=True);self.index[0]['qualified_package']='archive/link';self.seal_commit();self.run_cli('symlink')

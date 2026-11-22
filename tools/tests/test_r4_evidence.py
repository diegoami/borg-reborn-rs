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

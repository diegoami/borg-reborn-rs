"""The accepted pre-seam source cannot be reblessed by matching native/port edits."""
import difflib,hashlib,importlib.util,json,shutil,subprocess,sys,tempfile,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
spec=importlib.util.spec_from_file_location('item_use_audit',Path(__file__).resolve().parents[1]/'audit-item-use-port.py')
audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
class ItemUseAuditTest(unittest.TestCase):
 def test_all_37_bodies_and_accepted_baseline(self):
  result=audit.audit();self.assertEqual(len(result['functions']),37);self.assertTrue(result['accepted_baseline_verified'])
 def test_port_only_and_matching_mutants_rejected(self):
  with tempfile.TemporaryDirectory() as directory:
   root=Path(directory)/'copy';base=Path(directory)/'baseline'
   names=['borg-core/src/borg_item_use.rs','borg-next/src/stateful/item_use.rs','borg-next/src/lib.rs','docs/R1038-ITEM-USE-SOURCE.json','docs/R1038-ITEM-USE-SEAMS.patch']
   for name in names:
    p=root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(audit.ROOT/name,p)
   native=root/names[0];accepted=base/names[0];accepted.parent.mkdir(parents=True);shutil.copyfile(native,accepted)
   subprocess.run(['git','apply','--reverse',str(root/names[-1])],cwd=base,check=True,capture_output=True)
   before='item.timeout >= 1';after='item.timeout >= 2'
   port=root/names[1];s=port.read_text();self.assertIn(before,s);port.write_text(s.replace(before,after,1))
   with self.assertRaisesRegex(ValueError,'correspondence differs'):audit.audit(root)
   s=native.read_text();self.assertIn(before,s);native.write_text(s.replace(before,after,1))
   patch=''.join(difflib.unified_diff(accepted.read_text().splitlines(True),native.read_text().splitlines(True),fromfile='a/'+names[0],tofile='b/'+names[0]))
   (root/names[-1]).write_text(patch);manifest=json.loads((root/names[-2]).read_text());manifest['native_seams_sha256']=hashlib.sha256(patch.encode()).hexdigest();(root/names[-2]).write_text(json.dumps(manifest))
   with self.assertRaisesRegex(ValueError,'accepted item-use baseline differs'):audit.audit(root)

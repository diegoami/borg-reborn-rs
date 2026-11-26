"""Equipment ownership must reject altered restoration or native policy drift."""
import importlib.util
from pathlib import Path
import shutil,sys,tempfile,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
spec=importlib.util.spec_from_file_location('equipment_audit',Path(__file__).resolve().parents[1]/'audit-equipment-port.py')
audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)


class EquipmentAuditTest(unittest.TestCase):
    def test_all_owned_bodies_and_native_seams(self):
        self.assertEqual(audit.audit(),37)

    def test_restore_quantity_and_unowned_native_drift_rejected(self):
        names=['docs/R436-SOURCE.json','docs/R436-NATIVE-SEAMS.patch','borg-next/src/lib.rs',
               'borg-next/src/stateful/equipment_wear.rs','borg-next/src/stateful/equipment_identify.rs',
               *['borg-core/src/'+m+'.rs' for m in ['borg_item_wear','borg_item_id','borg_inventory','borg_io','borg_store']]]
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            for name in names:
                target=root/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(audit.ROOT/name,target)
            path=root/'borg-next/src/stateful/equipment_wear.rs';original=path.read_text()
            import re
            changed,n=re.subn(r'add_iqty\(st,\s*i,\s*-1\)', 'add_iqty(st, i, 1)',original,count=1)
            self.assertEqual(n,1);path.write_text(changed)
            with self.assertRaisesRegex(ValueError,'body/literal correspondence'):
                audit.audit(root)
            path.write_text(original)
            path=root/'borg-core/src/borg_item_wear.rs';path.write_text(path.read_text()+'\n// unexpected unowned alteration\n')
            with self.assertRaises(ValueError):audit.audit(root)

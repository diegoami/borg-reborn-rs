"""Reject drift even when the native and owned store implementations agree."""
import difflib,hashlib,importlib.util,json,shutil,subprocess,tempfile,unittest,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
spec=importlib.util.spec_from_file_location('store_audit',Path(__file__).resolve().parents[1]/'audit-store-port.py')
audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)

class StorePortTest(unittest.TestCase):
    def test_all_owned_bodies_match_accepted_source(self):
        self.assertEqual(audit.audit(),87)

    def test_matching_native_and_port_threshold_drift_is_rejected(self):
        manifest=json.loads((audit.ROOT/'docs/R437-SOURCE.json').read_text())
        names=[*manifest['receiving_original_sha256'],*manifest['owned_bodies'],
               'docs/R437-SOURCE.json','docs/R437-NATIVE-SEAMS.patch',
               'docs/M13-INVENTORY-PARAMETERS.md','borg-next/src/lib.rs']
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)/'mutant';baseline=Path(directory)/'baseline'
            for name in names:
                p=root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(audit.ROOT/name,p)
            for name in manifest['receiving_original_sha256']:
                p=baseline/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(root/name,p)
            subprocess.run(['git','apply','--reverse',str(root/'docs/R437-NATIVE-SEAMS.patch')],cwd=baseline,check=True,capture_output=True)
            before='item.value > param(Param::sell_batch_max_item_value)'
            after='item.value >= param(Param::sell_batch_max_item_value)'
            port=root/'borg-next/src/stateful/store_sell.rs';original=port.read_text()
            self.assertEqual(original.count(before),1);port.write_text(original.replace(before,after,1))
            with self.assertRaisesRegex(ValueError,'native-to-owned'):audit.audit(root)
            native=root/'borg-core/src/borg_store_sell.rs';text=native.read_text()
            self.assertEqual(text.count(before),1);native.write_text(text.replace(before,after,1))
            # Declare the mutant seam honestly; accepted-to-native proof must still reject it.
            patch=''.join(''.join(difflib.unified_diff((baseline/name).read_text().splitlines(True),(root/name).read_text().splitlines(True),fromfile='a/'+name,tofile='b/'+name)) for name in manifest['receiving_original_sha256'])
            (root/'docs/R437-NATIVE-SEAMS.patch').write_text(patch)
            manifest['native_seams_sha256']=hashlib.sha256(patch.encode()).hexdigest()
            (root/'docs/R437-SOURCE.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError,'accepted-to-native'):audit.audit(root)

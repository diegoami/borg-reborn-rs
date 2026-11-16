"""Review F19: real CLI rejects namespace escapes and non-regular source entries."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

CHECKER = Path(__file__).resolve().parents[1] / 'tools/check-pin.py'


class PinControls(unittest.TestCase):
    def test_root_symlinks_special_files_and_nested_exclusions_fail(self):
        with tempfile.TemporaryDirectory() as folder:
            base=Path(folder);root=base/'vendor';root.mkdir()
            (root/'source.rs').write_text('source')
            pin=base/'pin.json';pin.write_text(json.dumps({'tag':'test','commit':'test','files_sha256':{'source.rs':hashlib.sha256(b'source').hexdigest()}}))
            def result(path=root):
                return subprocess.run(['python3',str(CHECKER),str(pin),str(path)],capture_output=True,text=True)
            self.assertEqual(result().returncode,0)
            link=base/'link';link.symlink_to(root,target_is_directory=True)
            self.assertNotEqual(result(link).returncode,0)
            os.mkfifo(root/'fifo');self.assertNotEqual(result().returncode,0);(root/'fifo').unlink()
            for name in ['target','.git','__pycache__']:
                nested=root/'src'/name;nested.mkdir(parents=True);(nested/'unsealed.rs').write_text('bad')
                self.assertNotEqual(result().returncode,0);(nested/'unsealed.rs').unlink();nested.rmdir()
            # Documented root-level generated caches are excluded; not aliases.
            target=root/'target';target.mkdir();(target/'cache').write_text('generated')
            self.assertEqual(result().returncode,0)
            (target/'cache').unlink();target.rmdir();target.symlink_to(base/'outside',target_is_directory=True)
            self.assertNotEqual(result().returncode,0)

    def test_review_f29_traversal_errors_are_not_silently_ignored(self):
        if os.geteuid()==0:
            self.skipTest('unreadable directories are readable by root')
        with tempfile.TemporaryDirectory() as folder:
            base=Path(folder);root=base/'vendor';root.mkdir()
            (root/'source.rs').write_text('source')
            locked=root/'locked';locked.mkdir();(locked/'unsealed.rs').write_text('bad')
            pin=base/'pin.json';pin.write_text(json.dumps({'tag':'test','commit':'test','files_sha256':{'source.rs':hashlib.sha256(b'source').hexdigest()}}))
            locked.chmod(0o000)
            try:
                result=subprocess.run(['python3',str(CHECKER),str(pin),str(root)],capture_output=True,text=True)
            finally:
                locked.chmod(0o755)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('traversal',result.stderr)

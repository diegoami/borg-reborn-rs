"""Historical ordinary references stay comparable; synthetic identity is checked."""
import contextlib
import io
import json
from pathlib import Path
import runpy
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
DIFF = runpy.run_path(str(ROOT / 'harness/diff'))


class SyntheticDiffTest(unittest.TestCase):
    def test_absent_marker_means_ordinary_and_true_is_never_ignored(self):
        with tempfile.TemporaryDirectory() as tmp:
            a, b = Path(tmp) / 'a', Path(tmp) / 'b'
            fixture = ROOT / 'harness/reference/smoke-rust/smoke-warrior-1'
            shutil.copytree(fixture, a)
            shutil.copytree(fixture, b)
            summary = json.loads((b / 'summary.json').read_text())
            for synthetic, expected in ((False, True), (True, False)):
                with self.subTest(synthetic=synthetic), contextlib.redirect_stdout(io.StringIO()):
                    summary['synthetic'] = synthetic
                    (b / 'summary.json').write_text(json.dumps(summary))
                    self.assertEqual(DIFF['compare_run']('test', str(a), str(b)), expected)

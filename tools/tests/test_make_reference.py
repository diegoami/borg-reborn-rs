"""harness/make-reference.sh (E review finding 8, M2 review finding 8), on synthetic suite outputs.

Run: python3 -m unittest discover -s tools/tests
"""

import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'harness/make-reference.sh'

MANIFEST = '''suite=smoke
suite_sha256=aa
build_id_git_rev=bb
build_id_inputs_dirty=no
build_id_source_sha256=cc
build_id_engine_upstream=dd
binary_sha256=ee
overrides={overrides}
run_config.r1=menu f1
run_config.r2=menu f2
'''


def suite(base, overrides='none', profile=True):
    out = Path(base) / 'out'
    for name in ('r1', 'r2'):
        r = out / name
        r.mkdir(parents=True)
        for log in ('keys', 'messages', 'state'):
            (r / f'{log}.log').write_text(f'{name} {log}\n')
        (r / 'summary.json').write_text('{}\n')
        (r / 'config.txt').write_text('flags=menu\n' + ('profile=p.txt\nprofile_sha256=ff\n' if profile else ''))
    (out / 'manifest').write_text(MANIFEST.format(overrides=overrides))
    (out / 'DONE').write_text('')
    return out


def make(out, ref):
    return subprocess.run(['bash', str(SCRIPT), str(out), str(ref)], capture_output=True, text=True)


class MakeReferenceTest(unittest.TestCase):
    def test_a_good_suite_gives_its_provenance(self):
        with tempfile.TemporaryDirectory() as t:
            ref = Path(t) / 'ref'
            r = make(suite(t), ref)
            self.assertEqual(r.returncode, 0, r.stderr)
            text = (ref / 'REFERENCE').read_text()
            for line in ('git_rev=bb', 'source_sha256=cc', 'binary_sha256=ee', 'profile=p.txt', 'profile_sha256=ff',
                         'run_config.r1=menu f1', 'run_config.r2=menu f2'):
                self.assertIn(line + '\n', text)
            self.assertTrue((ref / 'r1/keys.log.xz').exists())
            self.assertEqual(sorted(p.name for p in Path(t).iterdir()), ['out', 'ref'], 'no partial directory left')

    def test_overrides_are_refused_before_anything_is_written(self):
        with tempfile.TemporaryDirectory() as t:
            ref = Path(t) / 'ref'
            r = make(suite(t, overrides='ALLOW_STALE_BUILD'), ref)
            self.assertEqual(r.returncode, 2)
            self.assertEqual(sorted(p.name for p in Path(t).iterdir()), ['out'], 'a refused reference leaves nothing')

    def test_runs_without_a_profile_identity_are_refused(self):
        with tempfile.TemporaryDirectory() as t:
            ref = Path(t) / 'ref'
            r = make(suite(t, profile=False), ref)
            self.assertEqual(r.returncode, 2)
            self.assertIn('profile', r.stderr)
            self.assertFalse(ref.exists())


if __name__ == '__main__':
    unittest.main()

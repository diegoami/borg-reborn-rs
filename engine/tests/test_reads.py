"""The D3 audit: every engine symbol engine.rs uses is in docs/ENGINE-READS.md,
and nothing there is stale (docs/SPEC.md §5; task TE.3, #73).

Run: python3 -m unittest discover -s engine/tests
"""

import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENGINE = ROOT / 'borg-reborn-rs/src/engine.rs'
READS = ROOT / 'docs/ENGINE-READS.md'


def used():
    text = ENGINE.read_text()
    symbols = set(re.findall(r'\bb::([A-Za-z_][A-Za-z0-9_]*)', text))
    # Declared by hand in engine.rs (upstream declares it in no header).
    symbols.update(n for n in ('inkey_hack',) if re.search(r'static mut %s\b' % n, text))
    return symbols


def listed():
    rows = re.findall(r'^\| `([A-Za-z_][A-Za-z0-9_]*)` \| ([ABKXWS]) \|', READS.read_text(), re.M)
    return {name: reason for name, reason in rows}


class EngineReadsTest(unittest.TestCase):
    def test_every_symbol_used_is_listed(self):
        self.assertEqual(used() - set(listed()), set(), 'engine.rs uses symbols docs/ENGINE-READS.md does not list')

    def test_every_symbol_listed_is_used(self):
        self.assertEqual(set(listed()) - used(), set(), 'docs/ENGINE-READS.md lists symbols engine.rs no longer uses')

    def test_only_engine_rs_touches_the_bindings(self):
        for p in (ROOT / 'borg-reborn-rs/src').glob('*.rs'):
            if p.name in ('engine.rs', 'bindings.rs', 'lib.rs'):
                continue
            self.assertNotRegex(p.read_text(), r'\bbindings::|crate::bindings', p.name)

    def test_the_bindings_are_what_the_headers_generate(self):
        r = subprocess.run(['sh', '-c', 'command -v bindgen'], capture_output=True, text=True)
        if not r.stdout.strip():
            self.skipTest('no bindgen (cargo install bindgen-cli --version 0.73.2 --locked)')
        r = subprocess.run([str(ROOT / 'engine/bindings/generate.sh'), '--check'], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)


if __name__ == '__main__':
    unittest.main()

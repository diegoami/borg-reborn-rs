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


ROW = r'^\| `([A-Za-z_][A-Za-z0-9_]*)` \| ([ABKXWST]) \|'
# engine.rs's test-only checks (T2.5, feature test-perception) start here;
# the reason-T symbols are used nowhere above it.
TRUTH_MARKER = '// ENGINE TRUTH (feature test-perception'


def sections():
    """{section title: {name: reason}} of docs/ENGINE-READS.md's tables."""
    out, title = {}, None
    for line in READS.read_text().splitlines():
        if line.startswith('## '):
            title = line[3:].strip()
        m = re.match(ROW, line)
        if m and title:
            out.setdefault(title, {})[m.group(1)] = m.group(2)
    return out


def listed():
    """The symbols engine.rs names (b::NAME): every table but the types."""
    return {n: r for t, rows in sections().items() if not t.startswith('Types') for n, r in rows.items()}


def types_listed():
    return {n: r for t, rows in sections().items() if t.startswith('Types') for n, r in rows.items()}


class EngineReadsTest(unittest.TestCase):
    def test_every_symbol_used_is_listed(self):
        self.assertEqual(used() - set(listed()) - set(types_listed()), set(), 'engine.rs uses symbols docs/ENGINE-READS.md does not list')

    def test_every_symbol_listed_is_used(self):
        self.assertEqual(set(listed()) - used(), set(), 'docs/ENGINE-READS.md lists symbols engine.rs no longer uses')

    def test_every_type_listed_is_a_bound_struct(self):
        # The types whose fields engine.rs reads (E review, finding 10). The
        # test cannot infer field types, so the list is kept by hand; each
        # entry must at least be a struct of the bindings.
        bound = set(re.findall(r'^pub (?:struct|union|type) ([A-Za-z_][A-Za-z0-9_]*)\b', (ROOT / 'borg-reborn-rs/src/bindings.rs').read_text(), re.M))
        self.assertEqual(set(types_listed()) - bound, set(), 'types listed that the bindings do not define')

    def test_every_field_read_belongs_to_a_listed_type(self):
        # M2 review, finding 5: a field engine.rs reads (`x.field`, not a
        # method call or a range) that only bound structs declare must belong
        # to at least one listed type, unless our own crate's types have a
        # field of that name too (a heuristic: field types are not inferred).
        engine = ENGINE.read_text()
        fields = set(re.findall(r'(?<=[\w)\]])\.([a-z_][a-z0-9_]*)\b(?!\s*[(!])', engine))
        owners = {}
        bindings = (ROOT / 'borg-reborn-rs/src/bindings.rs').read_text()
        for m in re.finditer(r'^pub (?:struct|union) ([A-Za-z_0-9]+) \{(.*?)^\}', bindings, re.M | re.S):
            for f in re.findall(r'^\s*pub ([a-z_][a-z0-9_]*):', m.group(2), re.M):
                owners.setdefault(f, set()).add(m.group(1))
        ours = set()
        for f in (ROOT / 'borg-reborn-rs/src').glob('*.rs'):
            if f.name != 'bindings.rs':
                ours.update(re.findall(r'^\s+(?:pub )?([a-z_][a-z0-9_]*): ', f.read_text(), re.M))
        listed = set(types_listed())
        unlisted = {f: sorted(owners[f]) for f in fields
                    if f in owners and not owners[f] & listed and f not in ours}
        self.assertEqual(unlisted, {}, 'fields read from types docs/ENGINE-READS.md does not list')

    def test_the_truth_reads_are_test_only(self):
        # Reason T: reads of the test-perception check (T2.5), never compiled
        # into the Borg that plays. They appear only below the marker, which
        # opens a #[cfg(feature = "test-perception")] module.
        text = ENGINE.read_text()
        self.assertIn(TRUTH_MARKER, text)
        above, below = text.split(TRUTH_MARKER, 1)
        self.assertIn('#[cfg(feature = "test-perception")]\npub mod truth', below)
        truth = {n for n, r in {**listed(), **types_listed()}.items() if r == 'T'}
        self.assertTrue(truth, 'no reason-T reads listed')
        used_above = set(re.findall(r'\bb::([A-Za-z_][A-Za-z0-9_]*)', above))
        self.assertEqual(truth & used_above, set(), 'reason-T symbols used outside the test-perception check')

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

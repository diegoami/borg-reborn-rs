"""A future carry must not silently bypass stateful effect recording."""
from pathlib import Path
import sys
import tempfile
import shutil
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import decision_surface as surface


class DecisionSurfaceTest(unittest.TestCase):
    def test_actual_policy_surface_is_closed(self):
        self.assertEqual(surface.check(Path(__file__).resolve().parents[2]), 7)

    def test_direct_core_engine_and_tape_bypasses_are_refused(self):
        for code, port in [('use crate::engine;', False),
                           ('crate::borg_new::effect(st);', False),
                           ('borg_rs::shell::execute(st);', True),
                           ('borg_rs::decision_tape::engine("x", || ());', True),
                           ('st.borg.tr(BI_CURHP)', True)]:
            self.assertTrue(surface.inspect(code, port), code)
        self.assertFalse(surface.inspect('// use crate::engine;\nlet text="crate::engine::effect()";'))
        self.assertFalse(surface.inspect('use crate::decision_api::engine; engine::randint0(100);'))

    def test_review_bypasses_are_refused(self):
        for code in [
            'use borg_rs as b; b::engine::randint0(3);',
            'use borg_rs::threat_facts as tf;',
            'st.borg.trait_[BI_CURHP as usize]',
            'borg_rs::memory::traits::BI_CURHP',
            'crate::shadow::register();',
            'use borg_rs :: threat_facts as tf;',
            'use borg_rs::memory;',
            'use borg_rs::memory::{traits::BI_CURHP};',
            'use borg_rs::decision_tape as tape;',
        ]:
            with self.subTest(code=code):
                self.assertTrue(surface.inspect(code, port=True), code)
        self.assertFalse(surface.inspect('use borg_rs::threat_facts::{param as p, Param};', port=True))

    def test_dispatch_exception_is_narrow(self):
        root = Path(__file__).resolve().parents[2]
        text = (root / 'borg-next/src/stateful/mod.rs').read_text()
        self.assertFalse(surface.inspect(text, port=True, dispatch=True))
        self.assertTrue(surface.inspect(text, port=True))
        # A second read or another BI write in the audited file is forbidden.
        for extra in ['let hp = st.borg.trait_[BI_CURHP as usize];',
                      'crate::shadow::register();']:
            self.assertTrue(surface.inspect(text + extra, port=True, dispatch=True))
        self.assertTrue(surface.inspect(text.replace('BI_CURSP', 'BI_CURHP'), port=True, dispatch=True))

    def test_new_and_nested_modules_are_inspected(self):
        root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory)
            for name in surface.NATIVE:
                path = fixture / 'borg-core/src' / (name + '.rs')
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(root / path.relative_to(fixture), path)
            shutil.copytree(root / 'borg-next/src/stateful', fixture / 'borg-next/src/stateful')
            extra = fixture / 'borg-next/src/stateful/new/nested.rs'
            extra.parent.mkdir()
            extra.write_text('use borg_rs as b;')
            with self.assertRaisesRegex(ValueError, 'nested.rs'):
                surface.check(fixture)
            extra.write_text('fn local_helper() {}')
            self.assertEqual(surface.check(fixture), 8)
            dispatch = fixture / 'borg-next/src/stateful/mod.rs'
            dispatch.write_text(dispatch.read_text() + 'crate::shadow::register();')
            with self.assertRaisesRegex(ValueError, 'mod.rs'):
                surface.check(fixture)

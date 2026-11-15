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

    def test_parent_paths_and_native_delegation_are_refused(self):
        probe = """pub fn zz_probe(st: &mut BorgState) -> i32 {
            let a = st.borg.trait_.get(super::super::BI_CURSP as usize).copied().unwrap_or(0);
            let _ = super::Decision::Caution.native(st);
            a
        }"""
        for code in [probe,
                     'super::Decision::Freedom {y,x}.native(st);',
                     'let t = &mut st.borg.trait_; t[i] = 0;',
                     'st.borg.trait_.get(i)',
                     'let native = decision.native; native(st);',
                     'Decision::native(request, st);',
                     'use super as parent;', 'use crate as root;',
                     'use ::borg_rs as core;',
                     'use {borg_rs as core};',
                     'use crate::{stateful::Decision};',
                     'self::parent::Decision::Caution.native(st);',
                     'use super::escape::{borg_freedom, other};',
                     'use super::escape::*;',
                     'use super::escape::{borg_freedom as native};']:
            with self.subTest(code=code):
                self.assertTrue(surface.inspect(code, port=True), code)
        for code in ['super::set_mana(st, value);',
                     'use super::escape::{borg_freedom, borg_escape};']:
            self.assertFalse(surface.inspect(code, port=True), code)

    def test_outside_source_and_dispatch_imports_are_refused(self):
        for code in ['include!("../x.rs");',
                     '#[path = "../x.rs"] mod outside;',
                     '#[cfg_attr(feature="bypass", path="../x.rs")] mod outside;',
                     'use borg_rs::decisions::*;',
                     'use borg_rs::decisions::{Decision, other};',
                     'use borg_rs :: decisions :: {Decision, other};']:
            with self.subTest(code=code):
                self.assertTrue(surface.inspect(code, port=True, dispatch=True), code)
        self.assertFalse(surface.inspect('use borg_rs::decisions::{Decision, Value};', port=True, dispatch=True))
        self.assertFalse(surface.inspect('// trait_ include!("x"); super::Decision.native(st)\nlet text = "trait_ super::super #[path = x]";', port=True))

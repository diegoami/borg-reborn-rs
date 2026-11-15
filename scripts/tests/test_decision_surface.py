"""A future carry must not silently bypass stateful effect recording."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import decision_surface as surface


class DecisionSurfaceTest(unittest.TestCase):
    def test_actual_policy_surface_is_closed(self):
        self.assertEqual(surface.check(Path(__file__).resolve().parents[2]), 6)

    def test_direct_core_engine_and_tape_bypasses_are_refused(self):
        for code, port in [('use crate::engine;', False),
                           ('crate::borg_new::effect(st);', False),
                           ('borg_rs::shell::execute(st);', True),
                           ('borg_rs::decision_tape::engine("x", || ());', True),
                           ('st.borg.tr(BI_CURHP)', True)]:
            self.assertTrue(surface.inspect(code, port), code)
        self.assertFalse(surface.inspect('// use crate::engine;\nlet text="crate::engine::effect()";'))
        self.assertFalse(surface.inspect('use crate::decision_api::engine; engine::randint0(100);'))

"""#1382: params.rs and CALIBRATION.md merge without the cited line numbers."""
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / 'scripts/merge-citations.py'
spec = importlib.util.spec_from_file_location('merge_citations', SCRIPT)
mc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mc)

BASE = '''\
    a => (1, "flag", 0, 1, "borg_x.rs:10", "A."),
    b => (2, "flag", 0, 1, "borg_x.rs:20", "B."),
    c => (3, "flag", 0, 1, "borg_x.rs:30", "C."),
| `a` | 1 | flag | 0-1 | `borg_x.rs:10` |
'''


class MergeCitations(unittest.TestCase):
    def test_citation_only_changes_on_both_sides_do_not_conflict(self):
        ours = BASE.replace('x.rs:20', 'x.rs:25').replace('x.rs:30', 'x.rs:35')
        theirs = BASE.replace('x.rs:20', 'x.rs:27').replace('x.rs:10', 'x.rs:12')
        merged, conflicts = mc.merge(BASE, ours, theirs)
        self.assertEqual(conflicts, 0)
        self.assertEqual(merged, ours)  # our numbers; check-param-citations.py --fix settles them

    def test_a_new_parameter_from_the_other_side_keeps_its_citation(self):
        theirs = BASE + '    d => (4, "flag", 0, 1, "borg_x.rs:44", "D."),\n'
        ours = BASE.replace('x.rs:30', 'x.rs:35')
        merged, conflicts = mc.merge(BASE, ours, theirs)
        self.assertEqual(conflicts, 0)
        self.assertEqual(merged, ours + '    d => (4, "flag", 0, 1, "borg_x.rs:44", "D."),\n')

    def test_a_real_change_on_both_sides_is_still_a_conflict(self):
        ours = BASE.replace('"B."', '"Ours."')
        theirs = BASE.replace('"B."', '"Theirs."')
        merged, conflicts = mc.merge(BASE, ours, theirs)
        self.assertEqual(conflicts, 1)
        self.assertIn('<<<<<<< HEAD', merged)
        self.assertIn('"Ours."', merged)
        self.assertIn('"Theirs."', merged)


if __name__ == '__main__':
    unittest.main()

"""#1382: a PR into main that belongs on a track branch is flagged."""
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / 'scripts/check-pr-base.py'
spec = importlib.util.spec_from_file_location('check_pr_base', SCRIPT)
cpb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cpb)


def v(base='main', head='task/T1-x', labels=(), files=(), issue=()):
    return cpb.violation(base, head, set(labels), list(files), set(issue))


class CheckPrBase(unittest.TestCase):
    def test_a_core_change_into_main_is_flagged(self):
        message = v(files=['docs/x.md', 'borg-core/src/params.rs'])
        self.assertIn('changes borg-core/', message)
        self.assertIn('track-p or feature/track-r', message)

    def test_a_track_label_on_the_closed_issue_names_the_branch(self):
        self.assertIn('Retarget it to track-p', v(files=['docs/x.md'], issue=['track:P', 'task']))
        self.assertIn('Retarget it to feature/track-r', v(issue=['track:R']))

    def test_docs_and_process_changes_may_go_to_main(self):
        self.assertIsNone(v(files=['docs/x.md', 'scripts/a.py'], issue=['process']))

    def test_other_bases_promotions_and_the_escape_label_pass(self):
        self.assertIsNone(v(base='track-p', files=['borg-core/a.rs']))
        self.assertIsNone(v(base='feature/track-r', files=['borg-core/a.rs']))
        self.assertIsNone(v(head='track-p', files=['borg-core/a.rs']))
        self.assertIsNone(v(head='feature/track-r', issue=['track:R']))
        self.assertIsNone(v(files=['borg-core/a.rs'], labels=['base-main-ok']))


if __name__ == '__main__':
    unittest.main()

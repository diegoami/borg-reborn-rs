"""#1382: a PR into main that belongs on a track branch is flagged."""
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / 'scripts/check-pr-base.py'
spec = importlib.util.spec_from_file_location('check_pr_base', SCRIPT)
cpb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cpb)


def v(base='main', head='task/T1-x', labels=(), files=(), issue=(), cross=False):
    return cpb.violation(base, head, set(labels), list(files), set(issue), cross)


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

    def test_a_fork_branch_named_like_a_track_branch_is_not_a_promotion(self):
        self.assertIn('changes borg-core/', v(head='track-p', files=['borg-core/a.rs'], cross=True))

    def test_issues_named_without_a_closing_keyword_are_found(self):
        body = 'Part of #1004. Stacked on #12. Refs #7, closes #9 and fixes #3.'
        self.assertEqual(cpb.referenced_issues(body), [3, 7, 9, 1004])
        self.assertEqual(cpb.referenced_issues(None), [])


if __name__ == '__main__':
    unittest.main()

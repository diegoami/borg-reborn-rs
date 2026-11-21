"""#819: the release workflow runs on demand only; its scripts parse."""
import importlib.util
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / '.github/workflows/release.yml'
spec = importlib.util.spec_from_file_location('release_notes', ROOT / 'scripts/release/notes.py')
notes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notes)


def on_block(text):
    """The workflow's top-level `on:` block, without comments."""
    m = re.search(r'^on:\n((?:[ #].*\n|\n)*)', text, re.M)
    assert m, 'no on: block'
    return '\n'.join(l for l in m.group(1).splitlines() if l.strip() and not l.lstrip().startswith('#'))


class ReleaseWorkflowTest(unittest.TestCase):
    def test_triggers_are_dispatch_and_version_tags_only(self):
        block = on_block(WORKFLOW.read_text())
        events = re.findall(r'^  ([a-z_]+):', block, re.M)
        self.assertEqual(events, ['workflow_dispatch', 'push'])
        push = block[block.index('  push:'):]
        self.assertRegex(push, r'tags: \["v\*"\]')
        self.assertNotIn('branches', push)

    def test_the_release_is_a_draft(self):
        text = WORKFLOW.read_text()
        self.assertEqual(text.count('gh release create'), 1)
        self.assertIn('--draft', text[text.index('gh release create'):])

    def test_scripts_parse(self):
        for script in sorted((ROOT / 'scripts/release').glob('*.sh')):
            result = subprocess.run(['bash', '-n', str(script)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, f'{script.name}: {result.stderr}')

    def test_plan_rows_come_with_their_header(self):
        rows = notes.plan_rows('P1')
        self.assertTrue(rows, 'PLAN.md names milestone P1')
        self.assertTrue(rows[0].startswith('|') and set(rows[1].replace('|', '').split()) <= {'---', ':---', '---:'})
        self.assertFalse(notes.plan_rows('NO-SUCH-MILESTONE-819'))


if __name__ == '__main__':
    unittest.main()

"""#819: the release workflow runs on demand only; its scripts parse."""
import importlib.util
import py_compile
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

    def test_build_only_runs_skip_the_release(self):
        # #823/#824: release off checks the packages without a draft; a tag push always drafts.
        text = WORKFLOW.read_text()
        for name in ('since:', 'release:', 'type: boolean'):
            self.assertIn(name, on_block(text))
        job = text[text.index('\n  release:\n'):]
        self.assertIn("if: needs.setup.outputs.release == 'true'", job)
        self.assertIn("github.event_name == 'push' || github.event.inputs.release != 'false'", text)

    def test_every_platform_is_packaged_and_played(self):
        text = WORKFLOW.read_text()
        for job, runner, platform in (('linux', 'ubuntu-24.04', 'linux'), ('macos', 'macos-latest', 'macos'),
                                      ('windows', 'windows-latest', 'windows')):
            body = text[text.index(f'\n  {job}:\n'):]
            body = body[:body.index('\n\n  ', 1)]
            self.assertIn(f'runs-on: {runner}', body)
            self.assertIn(f'scripts/release/package.sh {platform} ', body)
        self.assertIn('needs: [setup, source, linux, linux-clean, macos, windows]', text)

    def test_a_direct_start_has_its_folders_and_settings(self):
        # Owner, v0.1.0-alpha.1: "save failed" when angband.exe was started
        # directly; the engine creates lib/save etc. only on Unix.
        text = (ROOT / 'scripts/release/package.sh').read_text()
        self.assertIn('mkdir -p "$STAGE/lib/save" "$STAGE/lib/scores" "$STAGE/lib/panic" "$STAGE/lib/user/archive"', text)
        self.assertIn('"$ROOT/harness/prepare-borg-txt.sh" "$STAGE/lib/user"', text)
        self.assertIn("info.external_attr = (0o40755 << 16) | 0x10", text)

    def test_scripts_parse(self):
        for script in sorted((ROOT / 'scripts/release').glob('*.sh')):
            result = subprocess.run(['bash', '-n', str(script)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, f'{script.name}: {result.stderr}')
        for script in sorted((ROOT / 'scripts/release').glob('*.py')):
            py_compile.compile(str(script), cfile=None, doraise=True, optimize=-1)

    def test_plan_rows_come_with_their_header(self):
        rows = notes.plan_rows('P1')
        self.assertTrue(rows, 'PLAN.md names milestone P1')
        self.assertTrue(rows[0].startswith('|') and set(rows[1].replace('|', '').split()) <= {'---', ':---', '---:'})
        self.assertFalse(notes.plan_rows('NO-SUCH-MILESTONE-819'))

    def test_plan_rows_are_the_milestones_own(self):
        # Review of #827, finding 1: P2's row cites "as P1", R3-R5's "as R2".
        for milestone in ('P1', 'R2'):
            rows = notes.plan_rows(milestone)
            self.assertEqual(len(rows), 3, rows)
            self.assertIn(f'| `{milestone}` |', rows[2])
        # The older table has no label column: the bold title names the row.
        rows = notes.plan_rows('M3')
        self.assertEqual(len(rows), 3, rows)
        self.assertTrue(rows[2].startswith('| **M3. '))
        self.assertFalse(notes.plan_rows('M7'))


if __name__ == '__main__':
    unittest.main()

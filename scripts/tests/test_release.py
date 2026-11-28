"""#819: the release workflow runs on demand only; its scripts parse."""
import importlib.util
import py_compile
import re
import subprocess
import tempfile
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
        self.assertIn("needs.setup.outputs.release == 'true'", job.split('runs-on')[0])
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
        self.assertIn('"$ROOT/harness/prepare-pborg-config.sh" "$STAGE/lib/user"', text)
        self.assertIn("info.external_attr = (0o40755 << 16) | 0x10", text)

    def test_macos_only_when_asked_and_no_launcher(self):
        # Owner, v0.1.0-alpha.1: no watch-borg launcher, no macOS package by default.
        text = WORKFLOW.read_text()
        block = on_block(text)
        macos = block[block.index('      macos:'):]
        self.assertIn('default: false', macos.split('\n      ', 1)[1].split('\n  ', 1)[0] + macos)
        self.assertIn("if: needs.setup.outputs.macos == 'true'", text)
        self.assertFalse((ROOT / 'scripts/release/watch-borg.sh').exists())
        self.assertFalse((ROOT / 'scripts/release/windows/watch-borg.cmd').exists())
        shipped = (ROOT / 'scripts/release/package.sh').read_text().split('# The short guide.')[1].split('cp "$ROOT/README.md"')[0]
        for launcher in ('watch-borg', 'play-headless.sh"', 'play-headless.ps1 ', 'install -m'):
            self.assertNotIn(launcher, shipped.split('\nif [')[1], launcher)

    def test_the_shipped_readme_is_angbands_with_the_borg(self):
        # Owner: players get upstream Angband's README with the Borg added,
        # not the repository's.
        text = (ROOT / 'scripts/release/package.sh').read_text()
        self.assertIn('"$ROOT/engine/angband/README.md" "$ROOT/scripts/release/README-borg.md" "$STAGE/README.md"', text)
        self.assertNotIn('cp "$ROOT/README.md"', text)

    def test_the_packaged_guide_links_only_outside_the_package(self):
        # Review of #1386: the guide ships as docs/GUIDE.md, where its links
        # relative to the repository break; staged, each is a GitHub link at
        # the package's commit, and each names a file that exists.
        text = (ROOT / 'scripts/release/package.sh').read_text()
        self.assertIn('scripts/release/stage-guide.py', text)
        self.assertNotIn('cp "$ROOT/docs/user/GUIDE.md"', text)
        commit = 'a' * 40
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'GUIDE.md'
            result = subprocess.run(
                ['python3', str(ROOT / 'scripts/release/stage-guide.py'),
                 str(ROOT / 'docs/user/GUIDE.md'), str(out), commit],
                capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            staged = out.read_text()
        prose = re.sub(r'```.*?```', '', staged, flags=re.S)
        prose = re.sub(r'`[^`\n]*`', '', prose)
        targets = re.findall(r'\]\(([^)\s]+)\)', prose)
        self.assertGreater(len(targets), 10)
        prefix = 'https://github.com/diegoami/borg-reborn-rs/'
        for target in targets:
            if target.startswith('#'):
                continue
            self.assertIn('://', target, f'relative link left in the staged guide: {target}')
            if target.startswith(prefix):
                self.assertIn(f'/{commit}/', target)
        # A link to a file that does not exist is refused.
        # A link to a file that does not exist is refused; one in inline code
        # is not a link.
        spec = importlib.util.spec_from_file_location('stage_guide', ROOT / 'scripts/release/stage-guide.py')
        stage = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(stage)
        _, missing = stage.rewrite('[x](../no-such-file.md) `[y](../also-not.md)`',
                                   ROOT / 'docs/user/GUIDE.md', commit)
        self.assertEqual(missing, ['../no-such-file.md'])

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

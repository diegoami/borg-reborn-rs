"""The carry watcher handles Rust literals, nested comments and real git revisions."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('rust_ports', Path(__file__).resolve().parents[1] / 'rust_ports.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class RustPortsTest(unittest.TestCase):
    def test_boundaries_ignore_braces_and_fake_functions_in_literals_and_comments(self):
        source = '''
// fn fake() { }
fn first<'a>(s: &'a str) -> bool {
    let x = r###"} fn bogus() {"###;
    let c = '}'; /* outer /* nested } */ } */
    s == "{\\\"}" || x.is_empty() || c == '{'
}
fn second<const N: usize>() -> usize { N }
trait T { fn declaration(&self); }
'''
        result = module.functions(source)
        self.assertEqual(set(result), {'first', 'second'})
        self.assertIn('let c', result['first'][0])
        self.assertNotIn('fn second', result['first'][0])
        self.assertTrue(result['first'][0].endswith('}'))

    def test_changed_added_removed_functions_and_ported_subset(self):
        with tempfile.TemporaryDirectory() as name:
            repo = Path(name)
            def git(*args):
                return subprocess.run(['git', '-C', str(repo), *args], check=True, capture_output=True, text=True).stdout.strip()
            git('init', '-q')
            git('config', 'user.name', 'Test')
            git('config', 'user.email', 'test@example.invalid')
            source = repo / 'borg-rs/src/danger.rs'
            source.parent.mkdir(parents=True)
            source.write_text('fn wall_like() -> bool { false }\nfn removed() {}\n')
            git('add', '.')
            git('commit', '-qm', 'base')
            base = git('rev-parse', 'HEAD')
            source.write_text('pub fn wall_like() -> bool { true }\nfn added() {}\n')
            git('add', '.')
            git('commit', '-qm', 'changed')
            head = git('rev-parse', 'HEAD')
            manifest = {'ports': [{'native_path': 'borg-rs/src/danger.rs', 'native_function': 'wall_like', 'point': 'danger.wall_like'}]}
            rows = module.changed(repo, base, head, manifest)
            self.assertEqual({r['function']: r['change'] for r in rows['changed_functions']},
                             {'added': 'added', 'removed': 'removed', 'wall_like': 'changed'})
            self.assertEqual([r['point'] for r in rows['ported_changes']], ['danger.wall_like'])
            self.assertEqual(module.changed(repo, head, head, manifest)['ported_changes'], [])
            with self.assertRaisesRegex(ValueError, 'unknown commit'):
                module.changed(repo, 'missing-revision', head, manifest)
            manifest['ports'][0]['native_function'] = 'missing_function'
            with self.assertRaisesRegex(ValueError, 'missing in both'):
                module.changed(repo, base, head, manifest)

    def test_unbalanced_function_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unbalanced'):
            module.functions('fn unfinished() {')

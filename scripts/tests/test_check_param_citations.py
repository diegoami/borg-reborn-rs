"""#601: every parameter cites the line that reads it."""
import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/check-param-citations.py'
spec = importlib.util.spec_from_file_location('check_param_citations', SCRIPT)
cpc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cpc)

RUST = '''\
/// A doc comment.
pub(crate) fn alpha(st: &mut BorgState) -> bool {
    let x = 1;
    if p(Param::caution_alpha) > x {
        return true;
    }
    let s = "a { brace in a string";
    false
}

fn beta() -> i32 {
    let table = [Param::attack_beta_one, Param::attack_beta_two];
    table.len() as i32
}

fn gamma() {
    let c = crate::attack_preference::evaluate(1, 2, 3);
}
'''


def source(text):
    with tempfile.NamedTemporaryFile('w', suffix='.rs', delete=False) as f:
        f.write(text)
    return cpc.Source(Path(f.name))


def params(*rows):
    return '\n'.join(f'    {name} => (1, "flag", 0, 1, "{cite}", "Text."),' for name, cite in rows)


class CitationTest(unittest.TestCase):
    def test_the_repository_is_clean(self):
        result = subprocess.run([sys.executable, str(SCRIPT)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_a_citation_is_right_on_the_read_or_in_its_function(self):
        src = source(RUST)
        self.assertEqual(src.reads('caution_alpha'), [4])
        self.assertTrue(src.ok('caution_alpha', 4), 'the read itself')
        self.assertTrue(src.ok('caution_alpha', 2), 'the function that reads it')
        self.assertTrue(src.ok('caution_alpha', 8), 'past a brace in a string, still inside')
        self.assertFalse(src.ok('caution_alpha', 1), 'the doc comment above')
        self.assertFalse(src.ok('caution_alpha', 12), 'another function')
        self.assertTrue(src.ok('attack_beta_two', 11), 'a table read in one function')

    def test_a_parameter_read_through_another_module_cites_the_call(self):
        src = source(RUST)
        self.assertEqual(src.reads('attack_preference_melee_permille'), [17])

    def test_fix_points_a_wrong_citation_at_the_first_read(self):
        src = source(RUST)
        text = params(('caution_alpha', 'borg_x.rs:12'), ('attack_beta_one', 'borg_x.rs:11'))
        wrong, unread = cpc.check(text, lambda file: src)
        self.assertEqual(unread, [])
        self.assertEqual([(n, l, r) for _, n, _, l, r in wrong], [('caution_alpha', 12, 4)])
        fixed = cpc.fix_params(text, wrong)
        self.assertIn('"borg_x.rs:4"', fixed)
        self.assertEqual(cpc.check(fixed, lambda file: src), ([], []))

    def test_a_parameter_never_read_in_its_file_is_reported(self):
        src = source(RUST)
        wrong, unread = cpc.check(params(('caution_gone', 'borg_x.rs:4')), lambda file: src)
        self.assertEqual((wrong, unread), ([], [('caution_gone', 'borg_x.rs', 4)]))

    def test_calibration_rows_agree_with_the_registry(self):
        cited = {'caution_alpha': ('borg_x.rs', 4), 'defend_beta': ('borg_y.rs', 30)}
        doc = '\n'.join([
            '| `caution_alpha` | 0 | flag | 0–1 | `borg_x.rs:12` |',
            '| `caution_alpha` | 0 | flag | 0–1 | `borg_x.rs:4` (read in `borg_z.rs`) |',
            '| `unknown_name` | 5 | turns | 99 |',
            '| `caution_alpha` | 0 | flag | 0–1 | no citation |',
        ])
        self.assertEqual([(i, n, l, w) for i, n, l, w in cpc.check_calibration(doc, cited)],
                         [(0, 'caution_alpha', 12, 4)])
        fixed = cpc.fix_calibration(doc, cited)
        self.assertEqual(cpc.check_calibration(fixed, cited), [])
        self.assertIn('`borg_x.rs:4`', fixed.split('\n')[0])

    def test_the_p2_tables_bare_source_lines_are_left_alone(self):
        # The P2 tables cite the 5d9abb8f snapshot their headers name, not the live tree.
        cited = {'defend_beta': ('borg_y.rs', 30)}
        doc = '| Name | Default | Unit | Source line |\n| `defend_beta` | 5 | turns | 31 |'
        self.assertEqual(cpc.check_calibration(doc, cited), [])
        self.assertEqual(cpc.fix_calibration(doc, cited), doc)

    def test_braces_in_block_comments_and_raw_strings_do_not_count(self):
        src = source('''\
fn alpha() {
    /* a } brace
       and another } in a comment */
    let r = r#"}"#;
    let x = Param::caution_alpha;
}

fn beta() {}
''')
        self.assertEqual(src.spans, [(1, 6), (8, 8)])
        self.assertTrue(src.ok('caution_alpha', 1))

if __name__ == '__main__':
    unittest.main()

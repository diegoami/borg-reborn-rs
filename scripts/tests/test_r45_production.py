"""Receiving inventories must retain policy after interleaved test items."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from rust_ports import functions, without_test_items


class ProductionItemsTest(unittest.TestCase):
    def test_interleaved_nested_modules_literals_and_later_production(self):
        source = '''fn before() {}
#[cfg(test)]
mod tests {
    fn fixture() { let s = r#"} #[cfg(test)] mod fake {"#; }
    mod nested { fn hidden() {} }
}
#[cfg(feature = "real")]
pub fn after() -> bool { true }
#[cfg(not(feature = "real"))]
pub fn after() -> bool { false }
'''
        actual = without_test_items(source)
        self.assertEqual(set(functions(actual)), {'before', 'after'})
        self.assertEqual(len(functions(actual)['after']), 2)
        self.assertEqual(len(actual), len(source))
        self.assertEqual(actual.count('\n'), source.count('\n'))

    def test_standalone_test_function_and_const_end_before_next_policy(self):
        source = '''#[cfg(test)]
#[allow(dead_code)]
pub fn fixture<'a>(s: &'a str) -> bool { s == "}" }
fn real() {}
#[cfg(test)]
const TABLE: &[i32] = &[{ 1 }, 2];
fn later() {}
'''
        actual = without_test_items(source)
        self.assertEqual(set(functions(actual)), {'real', 'later'})
        self.assertNotIn('TABLE', actual)

    def test_unsupported_test_condition_or_item_fails_closed(self):
        for source in ['#[cfg(any(test, feature = "x"))] fn f() {}',
                       '#![cfg(test)]\nfn f() {}',
                       '#[cfg(test)] struct Fixture {}',
                       '#[cfg(test)] mod tests {']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                without_test_items(source)

    def test_other_cfg_arms_are_retained_without_claiming_cfg_evaluation(self):
        source = '#[cfg(not(feature = "x"))]\nfn f() {}\n'
        self.assertEqual(without_test_items(source), source)

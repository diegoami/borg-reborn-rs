"""Source correspondence must detect changed action keys, not only code shape."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
AUDITS = [(name, ROOT / 'scripts' / ('audit-' + name + '-port.py'))
          for name in ('attack', 'defence')]
AUDITS = [(name, path) for name, path in AUDITS if path.exists()]


class PolicyAuditTest(unittest.TestCase):
    def run_audit(self, name, path, change_key=False, native_tail='', port_tail=''):
        spec = importlib.util.spec_from_file_location('audit_' + name, path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        target = ROOT / 'borg-next/src/stateful' / (name + '.rs')
        canonical = subprocess.check_output(
            ['git', 'show', 'HEAD:' + str(target.relative_to(ROOT))], cwd=ROOT, text=True)
        self.assertIn("key(b'5')", canonical)
        if change_key:
            canonical = canonical.replace("key(b'5')", "key(b'6')", 1)
        native_file = 'borg_fight_attack.rs' if name == 'attack' else 'borg_fight_defend.rs'
        native = ROOT / 'borg-core/src' / native_file
        read = Path.read_text
        def read_text(path, *args, **kwargs):
            if path == target:
                return canonical + port_tail
            text = read(path, *args, **kwargs)
            return text + native_tail if path == native else text
        output = io.StringIO()
        with patch.object(Path, 'read_text', read_text), contextlib.redirect_stdout(output):
            module.main()
        return json.loads(output.getvalue())

    def test_canonical_policy_and_character_literals_match(self):
        for name, path in AUDITS:
            with self.subTest(policy=name):
                rows = self.run_audit(name, path)['functions']
                self.assertTrue(rows)
                self.assertTrue(all(row['character_literals_equal'] for row in rows))

    def test_changed_action_key_is_rejected_without_mutating_runtime_inputs(self):
        for name, path in AUDITS:
            with self.subTest(policy=name), self.assertRaises(AssertionError):
                self.run_audit(name, path, change_key=True)

    def test_late_production_is_audited_after_test_and_fixture_items(self):
        native_only_test = '\n#[cfg(test)]\nmod extra_tests { fn fixture() {} }\n'
        late_policy = '\nfn receiving_policy() -> i32 { 17 }\n'
        for name, path in AUDITS:
            with self.subTest(policy=name):
                native_tail = native_only_test + late_policy
                report = self.run_audit(name, path, native_tail=native_tail, port_tail=late_policy)
                self.assertIn('receiving_policy', [row['function'] for row in report['functions']])
                with self.assertRaises(AssertionError):
                    self.run_audit(name, path, native_tail=native_tail)
                with self.assertRaisesRegex(AssertionError, 'receiving_policy'):
                    self.run_audit(name, path, native_tail=native_tail,
                                   port_tail=late_policy.replace('17', '18'))

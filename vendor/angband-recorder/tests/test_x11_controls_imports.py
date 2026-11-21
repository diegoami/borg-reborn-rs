"""tests/x11-controls.py is only run on a full X11 build, so a name it forgets to
import would show up only when a session times out (issue #13: signal.SIGKILL in
the timeout handlers, never imported). Check its module-level imports statically."""
import ast
from pathlib import Path
import unittest

SOURCE = Path(__file__).with_name('x11-controls.py')
MODULES = {'argparse', 'fcntl', 'hashlib', 'json', 'os', 'shutil', 'signal', 'subprocess', 'time'}


class X11ControlsImports(unittest.TestCase):
    def test_every_stdlib_module_it_uses_is_imported(self):
        tree = ast.parse(SOURCE.read_text())
        imported = {alias.asname or alias.name.split('.')[0]
                    for node in ast.walk(tree) if isinstance(node, ast.Import)
                    for alias in node.names}
        used = {node.value.id for node in ast.walk(tree)
                if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name)
                and node.value.id in MODULES}
        self.assertEqual(sorted(used - imported), [], 'used but not imported')


if __name__ == '__main__':
    unittest.main()

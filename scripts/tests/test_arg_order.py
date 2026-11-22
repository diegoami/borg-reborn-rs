"""#855: build.sh knows which order the compiler evaluates call arguments in."""
import shutil
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROBE = ROOT / 'harness/arg-order.sh'


def order(cc):
    return subprocess.run([str(PROBE), cc], capture_output=True, text=True, check=True).stdout.strip()


class ArgOrderTest(unittest.TestCase):
    @unittest.skipUnless(shutil.which('gcc'), 'no gcc')
    def test_gcc_here_is_the_references_order(self):
        # CI and the zoo build with GCC on x86-64: the references' games.
        if subprocess.run(['uname', '-m'], capture_output=True, text=True).stdout.strip() == 'x86_64':
            self.assertEqual(order('gcc'), 'rtl')

    @unittest.skipUnless(shutil.which('clang'), 'no clang')
    def test_clang_is_left_to_right(self):
        self.assertEqual(order('clang'), 'ltr')

    def test_build_refuses_other_games_unless_asked(self):
        text = (ROOT / 'harness/build.sh').read_text()
        guard = text.index('ARG_ORDER=$(')
        self.assertLess(guard, text.index('cmake -S'), 'the probe runs before the build')
        self.assertIn('[ "$ARG_ORDER" != rtl ] && [ "${BORG_ALLOW_OTHER_GAMES:-0}" != 1 ]', text)
        self.assertIn('echo "arg_order=$ARG_ORDER"', text)


if __name__ == '__main__':
    unittest.main()

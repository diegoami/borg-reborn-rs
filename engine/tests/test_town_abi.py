"""Receiving C ABI used by the M13 donor option adapter (#328)."""
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class TownAbiTest(unittest.TestCase):
    def test_native_options_remain_the_standard_store_layout(self):
        # Pair with engine::receiving_options_tests in borg-core. This reads
        # the receiving C headers, independently of the donor Rust bindings.
        source = '''
#include "angband.h"
#include <stddef.h>
_Static_assert(OPT_MAX == 46, "audit the donor option adapter on option changes");
_Static_assert(sizeof(struct player_options) == 50, "player_options ABI");
_Static_assert(offsetof(struct player_options, hitpoint_warn) == 46, "option tail ABI");
'''
        with tempfile.TemporaryDirectory() as folder:
            result = subprocess.run(
                ['cc', '-std=c11', '-DALLOW_BORG',
                 '-I' + str(ROOT / 'engine/angband/src'), '-x', 'c', '-c',
                 '-o', str(Path(folder) / 'options.o'), '-'],
                input=source, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)

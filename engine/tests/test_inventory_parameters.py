"""Bounded native calibration check (#433), not a survival comparison."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless((ROOT / 'build/rust/game/angband').is_file(), 'native build required')
class InventoryParametersTest(unittest.TestCase):
    def test_a_tight_town_budget_changes_real_native_choices(self):
        with tempfile.TemporaryDirectory(prefix='inventory-parameters-') as tmp:
            root = Path(tmp)
            # Keep exported deaths outside the disposable run directories.
            registry = Path(os.environ.get('BORG_DEATH_RECORDS_DIR',
                                           str(Path(tempfile.gettempdir()) / 'borg-tests/deaths')))
            base = {k: v for k, v in os.environ.items()
                    if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_'))}
            if 'BORG_CEMETERY_DIR' in os.environ:
                base['BORG_CEMETERY_DIR'] = os.environ['BORG_CEMETERY_DIR']
            base.update(BORG_CORE='borg-rs', BORG_CORE_OVERRIDES='off',
                        BORG_DEATH_RECORDS_DIR=str(registry.resolve()))
            for name, budget in [('default', None), ('tight', '1')]:
                env = dict(base)
                if budget is not None:
                    env['BORG_PARAM_SHOPPING_MAX_TURNS'] = budget
                result = subprocess.run([str(ROOT / 'harness/run.sh'), str(root / name),
                                         '1010', '0', '0', '3000', 'menu'],
                                        cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertFalse(json.loads((root / name / 'summary.json').read_text())['dead'])
            settings = {p['name']: p for p in json.loads((root / 'tight/params-core.json').read_text())}
            self.assertEqual(settings['shopping_max_turns']['value'], 1)
            self.assertEqual(settings['shopping_max_turns']['origin'], 'environment')
            self.assertNotEqual((root / 'default/keys.log').read_bytes(),
                                (root / 'tight/keys.log').read_bytes())

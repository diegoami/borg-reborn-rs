"""#1339 (review of #1340): harness/run.sh accepts the retired errand and
plan settings at their off values, as the game does, and refuses them on.

A stub game marks that the run reached the launch, then fails; only the
checks before the launch are under test, so no native build is needed."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class RetiredHarnessSettingsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='retired-harness-')
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.marker = self.base / 'launched'
        fake = self.base / 'angband'
        fake.write_text(f'#!/bin/sh\ntouch {self.marker}\nexit 3\n')
        fake.chmod(0o755)
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('BORG_', 'BORGH_', 'ANGBAND_', 'PBORG_'))}
        self.env.update(ANGBAND_BIN=str(fake), BORG_CORE='borg-rs')

    def run_sh(self, name, **env):
        self.marker.unlink(missing_ok=True)
        argv = [str(ROOT / 'harness/run.sh'), str(self.base / name), '1', '0', '0', '100', 'menu']
        return subprocess.run(argv, env=dict(self.env, **env), text=True,
                              capture_output=True, timeout=60)

    def test_off_values_reach_the_launch(self):
        for name, env in [('plan-env-none', {'BORG_ERRANDS_PLAN': 'none'}),
                          ('plan-env-empty', {'BORG_ERRANDS_PLAN': ''}),
                          ('set-errand-none', {'PBORG_SET': 'borg_reborn_errand=none'}),
                          ('set-fear-off', {'PBORG_SET': 'borg_reborn_stopscared=off'}),
                          ('set-plan-none', {'PBORG_SET': 'borg_plan=none'})]:
            with self.subTest(name=name):
                result = self.run_sh(name, **env)
                self.assertNotIn('were retired', result.stderr)
                self.assertTrue(self.marker.exists(), result.stderr)

    def test_on_values_stop_before_the_launch(self):
        for name, env in [('plan-env', {'BORG_ERRANDS_PLAN': 'plan.txt'}),
                          ('set-errand', {'PBORG_SET': 'borg_reborn_errand=ladder'}),
                          ('set-fear', {'PBORG_SET': 'borg_reborn_stopscared=50'}),
                          ('set-plan', {'PBORG_SET': 'borg_plan=errands_plan.txt'})]:
            with self.subTest(name=name):
                result = self.run_sh(name, **env)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn('retired (#1339)', result.stderr)
                self.assertFalse(self.marker.exists())


if __name__ == '__main__':
    unittest.main()

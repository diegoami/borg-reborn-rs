"""Stage PBorg settings from a legacy test fixture."""
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def write_pborg_config(user, text=None):
    if text is None:
        text = (ROOT / 'harness/borg.txt').read_text()
    converted = subprocess.check_output(
        ['python3', str(ROOT / 'harness/pborg-config.py'), '-'], input=text.encode('utf-8'))
    (user / 'pborg.yaml').write_bytes(converted)

"""Stage the shipped PBorg YAML for an engine test."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def write_pborg_config(user, text=None):
    if text is None:
        text = (ROOT / 'harness/pborg.yaml').read_text(encoding='utf-8')
    (user / 'pborg.yaml').write_bytes(text.encode('utf-8'))

#!/usr/bin/env python3
"""Compatibility pointer: implementation is owned by diegoami/borg-cemetery."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from cemetery_tools import forward
forward('rerun-death.py', globals(), extra_settings=('BORG_SHADOW',))

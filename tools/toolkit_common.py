"""Shared heavy-work lock and readable terminal output; movie-only uses neither."""
from contextlib import contextmanager
from functools import wraps
import fcntl
import os
from pathlib import Path
import shlex
import sys

_depth = 0


@contextmanager
def native_lock():
    global _depth
    if _depth:
        _depth += 1
        try:
            yield
        finally:
            _depth -= 1
        return
    with Path('/tmp/angband-validation.lock').open('a') as held:
        try:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print('Waiting for shared native validation lock: /tmp/angband-validation.lock', flush=True)
            fcntl.flock(held, fcntl.LOCK_EX)
        _depth = 1
        try:
            yield
        finally:
            _depth = 0


def serialized(function):
    @wraps(function)
    def wrapper(*args, **kwargs):
        with native_lock():
            return function(*args, **kwargs)
    return wrapper


def color(text, code):
    enabled = sys.stdout.isatty() and 'NO_COLOR' not in os.environ and os.environ.get('TERM') != 'dumb'
    return f'\033[{code}m{text}\033[0m' if enabled else text


def section(title):
    print('\n' + color(title, '1'), flush=True)


def command(argv):
    value = shlex.join([str(v) for v in argv])
    print('\n' + color(value, '36') + '\n', flush=True)
    return value

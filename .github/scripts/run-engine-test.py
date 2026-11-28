"""Run one engine test module or its selected unittest classes."""

from pathlib import Path
import sys
import unittest


def main():
    path = Path(sys.argv[1]).resolve()
    if not path.is_file():
        sys.exit(f'Missing engine test module: {path}')

    loader = unittest.defaultTestLoader
    if len(sys.argv) > 2:
        sys.path.insert(0, str(path.parent))
        tests = loader.loadTestsFromNames(f'{path.stem}.{name}' for name in sys.argv[2:])
    else:
        tests = loader.discover(str(path.parent), pattern=path.name)

    if tests.countTestCases() == 0:
        sys.exit(f'No tests loaded from {path}')
    return not unittest.TextTestRunner().run(tests).wasSuccessful()


if __name__ == '__main__':
    sys.exit(main())

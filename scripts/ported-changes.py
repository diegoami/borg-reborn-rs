#!/usr/bin/env python3
"""List changed native Rust functions and the subset already ported in Track R.

Read only. The default base is borg-core/PIN, not the reference pin.
This reports source changes; the shadow run judges behavior.
"""
import argparse
import json
from pathlib import Path
from rust_ports import changed

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=ROOT / 'vendor/borg-rs')
    parser.add_argument('--base', default=(ROOT / 'borg-core/PIN').read_text().split()[0])
    parser.add_argument('--head', default='origin/HEAD')
    parser.add_argument('--manifest', type=Path, default=ROOT / 'docs/ported-functions.json')
    parser.add_argument('--json', action='store_true')
    args = parser.parse_args()
    try:
        report = changed(args.repo, args.base, args.head, json.loads(args.manifest.read_text()))
    except (ValueError, OSError) as error:
        parser.exit(1, str(error) + '\n')
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f'{args.base} -> {args.head}: {len(report["changed_functions"])} changed Rust functions')
        for row in report['changed_functions']:
            print(f'  {row["path"]}:{row["function"]}: {row["change"]}')
        print(f'{len(report["ported_changes"])} ported functions changed')
        for row in report['ported_changes']:
            print(f'  {row["point"]}: {row["path"]}:{row["function"]}: {row["change"]}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

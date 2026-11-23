#!/usr/bin/env python3
"""Pin the complete receiving production surface, including interleaved policy.

This inventory does not attest semantic parity or update historical receipts.
It reads immutable Git objects, never a potentially half-resolved merge tree.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from rust_ports import functions, mask_non_code, without_test_items

ROOT = Path(__file__).resolve().parents[1]
R4 = '40cbaf272193abb44dd80fad2cfeec24b627be22'
TARGET = '24be98732748230062882fe2b7d9601dba5e2cc0'
MODULES = ('borg_danger', 'borg_caution', 'borg_escape', 'borg_recover',
           'borg_fight_attack', 'borg_fight_defend', 'borg_item_use',
           'borg_power', 'borg_item_wear', 'borg_item_id', 'borg_home_power',
           'borg_store_buy', 'borg_store_sell', 'borg_think_store')
NEW_MODULES = ('attack_preference', 'borg_cursed', 'borg_fast_fragile',
               'borg_kit', 'borg_lightless', 'borg_town_hunt', 'scary_list')


def source(ref, path):
    return subprocess.check_output(['git', 'show', ref + ':' + path], cwd=ROOT, text=True)


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


def signature(body):
    # These modules' signatures contain no const-generic block expressions.
    # Preserve references, argument order, widths, return types and visibility.
    return ' '.join(body[:mask_non_code(body).index('{')].split())


def measure():
    out = {'schema': 'borg-reborn-rs/r45-receiving-surface@1',
           'accepted_r4': R4, 'owner_named_main': TARGET, 'modules': {},
           'new_modules': {}, 'entry_migrations': []}
    for name in MODULES + NEW_MODULES:
        path = 'borg-core/src/' + name + '.rs'
        raw = source(TARGET, path)
        production = without_test_items(raw)
        definitions = functions(production)
        rows = {name: [{'signature': signature(body), 'sha256': sha(body)}
                       for body in bodies] for name, bodies in sorted(definitions.items())}
        record = {'path': path, 'source_sha256': sha(raw),
                  'production_sha256': sha(production),
                  'definitions': sum(map(len, rows.values())), 'functions': rows}
        out['modules' if name in MODULES else 'new_modules'][name] = record
        if name not in MODULES:
            continue
        old = functions(without_test_items(source(R4, path)))
        for old_name in sorted(n for n in old if n.endswith('_native')):
            public_name = old_name.removesuffix('_native')
            old_sig = signature(old[old_name][0])
            old_sig = re.sub(r'^pub\(crate\) fn ', 'pub fn ', old_sig).replace(old_name + '(', public_name + '(')
            new_sig = [signature(body) for body in definitions.get(public_name, [])]
            # Token spacing alone is not an API migration.
            compact = lambda s: re.sub(r'\s+', '', s)
            if not new_sig or [compact(s) for s in new_sig] != [compact(old_sig)]:
                out['entry_migrations'].append({'module': name, 'entry': public_name,
                                               'r4_signature': old_sig,
                                               'receiving_signatures': new_sig,
                                               'status': 'retired' if not new_sig else 'signature changed'})
    out['limits'] = ('Source inventory only; cfg arms other than cfg(test) remain visible. '
                     'All definitions, including helpers, are listed; this is not an owned-policy '
                     'classification, dependency closure, source equivalence proof or runtime parity result. '
                     'Historical R4 source/seam manifests remain unchanged.')
    return out


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    result = measure()
    dest = ROOT / 'docs/R45-RECEIVING-SURFACE.json'
    if args.write:
        dest.write_text(json.dumps(result, indent=2) + '\n')
    elif json.loads(dest.read_text()) != result:
        raise SystemExit('receiving inventory differs; inspect pinned sources before --write')
    print(json.dumps({'modules': len(result['modules']), 'new_modules': len(result['new_modules']),
                      'definitions': sum(r['definitions'] for r in result['modules'].values()),
                      'entry_migrations': result['entry_migrations']}, indent=2))

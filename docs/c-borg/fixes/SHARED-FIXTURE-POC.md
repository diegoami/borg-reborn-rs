# CBUG-0066 shared decision-boundary POC — native exporter

Owner task: [native315](https://github.com/diegoami/better-borg-c/issues/315).
Coordination and canonical contract/corpus: [workspace40](https://github.com/diegoami/angband-workspace/issues/40).
TypeScript consumer: [Mandos217](https://github.com/diegoami/mandos/issues/217).

This tools-only producer reuses the existing nine-case CBUG-0066 fixture.
It does not adopt another fix, modify a production source/ledger, compare
world generation, or claim that Neo danger arithmetic or damage matches C.
The boundary is the actual native backup selector after its candidate danger
and critical-trait evaluation. TypeScript consumes those explicit observations.

## Exact runtime and fixture identities

- Before: pristine `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`.
- Fixed: first actual consumer `39986cc52f3f70727ada24b6def5de98d0bd36c8`.
  Its `src/`, `lib/` and CMake tree equals source correction
  `ebcd255ba9064fc44a35d693f42f9df3dc00bd88`; the only runtime difference
  from pristine is `src/borg/borg-item-wear.c`.
- Original fixture: commit `1b67370d0dc56d9b7ef34e5356e63d21a7095545`,
  `docs/audit/c-reports/CBUG-0066/expanded/scenario.c`, raw SHA256
  `b26f955689acaf552545019189ade029e82344a0c508d9c77d5b1581015a70b6`.
- Wear module SHA256 before:
  `a7f1427419a3f2b3c7810aa7761d9c31d13e632633ccdb6f86998c1d4ad02bd9`;
  fixed `7f6d4eb2e5e33139f23786a2cf51031418f5d3c214a78eb1723aa203c1acb750`.

Do not substitute current product main: it contains later independent fixes.
Both fixture and observer compile against each pinned runtime's own headers,
Core objects and unit helpers. Full header/helper-source identities must match
across the narrow pair. Source, object, command, compiler and data hashes stay
in the generated build/results manifests. Preparation changes only build-ID
and isolated private-path metadata; it does not patch source C functions.

## Observation and qualification

`native-swap-fixture-observer.c` links with GNU `--wrap` for exactly
`borg_backup_swap`, `borg_danger` and `borg_keypress`. Each wrapper calls its
original once. The selector wrapper reads actual entry gates/items; the danger
wrapper records the returned raw score and critical traits before the caller's
protection guards; keypress observation copies the real queue writes without
consuming them. No additional danger evaluation or RNG draw is introduced.

The original fixture remains unchanged except its pre-existing allowed absolute
output/profile relocation. It still consumes its keys, equips the actual item,
schedules actual spell damage and executes original assertions. The export
cross-checks its `DECISION` and `SELECTED` rows, evaluation call count/order,
selected indexes and existing trial dangers. Raw original output must be
byte-identical between plain and observed runs after removing only new POC
lines; original HP/key/assertion traces are retained. An absent item is null,
not a trial -1 or a fabricated zero-score candidate.

Nine native before/fixed input objects must match. Five protective decisions
change; four controls (armour only, weapon only, tie, no benefit) stay unchanged.
Four protective old results are **no swap**, not wearing the worse item, since
the selected candidate has no benefit or fails the acceptance ceiling. The
both-helpful case chooses weapon danger174 before, armour danger120 after,
with initial danger239/avoidance450. These values are actual native results.
The original fixture depth is50 and swaps are enabled. The caller danger
requirement is met; this does not claim all native/TS caller guards equivalent.
Native HP effects are context only, not cross-engine assertions or independent
survival evidence. This POC ships no production TS correction.

## Reproduction and outputs

Run from a new isolated tooling checkout with normal native prerequisites.
If evidence/source objects are absent, fetch those exact immutable commits
from our source tracker read-only; do not select a mutable branch tip:

```sh
git fetch --no-tags https://github.com/diegoami/neo-angband-tracker.git 1b67370d0dc56d9b7ef34e5356e63d21a7095545
flock /tmp/angband-validation.lock python3 tools/export-swap-fixtures.py --output artifacts/fixture-poc/new-run
```

The output directory must be fresh. `native-before.json`, `native-fixed.json`
and `cases.json` use the agreed `angband/borg-fixture-*@1` result/corpus shapes.
`cases.json` is generated from actual C input/action observations, never an
illustrative table or a duplicated chooser. Physical keys/indexes/slots remain
in evidence; normalized actions are none or wield armour/weapon. Input SHA256
uses recursively sorted JSON UTF-8, compact separators, no trailing newline.

`export.json`, archived sources, matching builds/helpers, original/relocated
fixture, exact executed exporter and observer snapshots, plain/observed logs
and compiler commands retain failure and source provenance. Do not overwrite
an earlier attempt or describe an expected failing old assertion as an
infrastructure failure. The original native test process can exit zero even
when the known-bug assertion is red; numerical/fixture output is authoritative.

The parent freezes the canonical corpus and compares actual inputs before
fix-state expectations. A mismatch is an investigation signal, not an automatic
port bug. Full fixture and result receipts, numerical gates and PR identity
belong on issue315; current primary and owner death files remain untouched.

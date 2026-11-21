# Tporod-17 carry: verified product scenario

Consumer [371](https://github.com/diegoami/better-borg-c/issues/371),
canonical source
[17](https://github.com/diegoami/neo-angband-tracker/issues/17)
(tporod-17; source fix `d752bad27`, evidence fixture `92ca00fdc`).
Applied as a `cherry-pick -x` of source
`d752bad27185ee7a709459b0e83186d9b0d16d7f`, patch
`patches/0096-tporod-class-exception.patch` (applied commit
`3206b884b5`).

## Defect

The Black Market exception for the Rod of Teleport Other is meant for
classes that cannot cast Teleport Other themselves ("mage and rogue can
cast teleport away"), but it read `!= CLASS_MAGE && == CLASS_ROGUE`, so
it applied to Rogues only. Rogues saved gold for a rod they do not
need, and other classes fell back to the general Black Market rule that
waits for much more gold. The correction uses `!=` for both classes, as
the Recall and Speed rod exceptions next to it do.

## Pins and provenance

Before: merged main `71f61a615` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-store-buy.c` is otherwise pristine on the product except
for the carried urgentshop-23/starvealch-18/homebuyqty-19/tradehist-20
changes, which the source base predates: the 3-way merge preserves them
and the layered-module check proves the source diff still reproduces
the product modules. The verifier proves that the source diff applied
to the before module equals the product module and that the applied
commit is the only `src`/`lib` difference from the before commit.
Fixture: kit `92ca00fdc` at head `a85a78156`, asserted byte-identical
to the run pin.

## Verified results (`artifacts/tp17/scenarios`, 2026-11-09)

- `warrior` — fail before, pass after: before the warrior neither
  saved for nor bought the rod (`saves_for_rod=0`), after it saves for
  it (`=1`, no purchase in this scenario).
- `rogue` — flips the other way, as intended: before the rogue saved
  for the rod it cannot use (`saves_for_rod=1`), after it stops
  (`=0`).
- `warrior-buy` — **diverges from the kit's recorded rows, disclosed**:
  on the kit's pristine base the scenario cannot afford the rod even
  while saving (row `saves=1/buys=0`, failing). On this product the
  carried sibling rod fixes (patch 0091 tele-away defence, patch 0095
  `BI_ATPORTOTHER` power) change the scenario: before this fix the
  warrior neither saves nor buys (row `saves=0/buys=0`, failing); after
  this fix it saves **and completes the purchase** (`saves=1/buys=1`,
  passing) — the class exception working together with the sibling
  carries. Attribution to the sibling patches follows from their
  recorded tree scopes.
- `mage` control: 1/1 in both phases, identical rows and byte-identical
  log (`34c7edd7…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes.

## Scope and limits

Bounded Black Market saving/purchase scenarios across four classes.
They prove the class exception and the purchase decision; no survival
claim.

## Exact next work

1. PR from `task/carry-tporod`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the torodpower tip cohort.
3. Receipts on issue 371, source 17 and checkpoint 1.

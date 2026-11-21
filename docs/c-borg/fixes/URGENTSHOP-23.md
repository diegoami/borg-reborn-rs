# Urgentshop-23 carry: verified product scenario

Consumer [336](https://github.com/diegoami/better-borg-c/issues/336),
canonical source
[23](https://github.com/diegoami/neo-angband-tracker/issues/23)
(urgentshop-23; source fix `09c28cd59`, evidence fixture `dbd9ddf1b`).
Applied as the second step of the dim-urg PR (`cherry-pick -x` of source
`09c28cd59c948fa03fb49349d30e7236b6d067ed`, patch
`patches/0087-alchemy-poison-cut-cures.patch`), stacked on dimdoor-21
(patch 0086).

## Defect

When the Borg is poisoned or bleeding in town, `borg_choose_shop()` set
the store goal and `borg_think_shop_buy_useful()` only weighed the wares
of that one store — both using store index 3, which was the Temple in
older versions. In 4.2 index 3 is the Bookseller and cure potions are
sold in the Alchemy shop, index 4. A poisoned or bleeding character with
gold and `Potions of Cure Light Wounds` on sale chose no shop at all,
although the same character buys them when healthy. The correction uses
index 4, the Alchemy shop, in both places (`src/borg/borg-store-buy.c`,
`src/borg/borg-think-store.c`).

## Pins and provenance

Before: merged main `8b549c2f9` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit); step before: the dimdoor-21 step tip (snapshotted build
`build/native-before-urg`, gate result
`artifacts/native-before-urg/result.json`). Both modules are otherwise
pristine on the product. The verifier proves that the source diff
applied to each before module equals the product module, that the fix
commit's own scope is the two files here, and that the tree-level scope
from before to the PR tip is those plus dimdoor-21's `borg-escape.c`.
Fixture: kit `dbd9ddf1b` at head `e0ecd1ef7`, asserted byte-identical to
the run pin.

## Verified results (`artifacts/us23/scenarios`, 2026-11-09)

- `poisoned` / `bleeding` — fail before, pass after: before 0/1
  (`buys_cure=0`; the borg flowed to the Bookseller and bought
  nothing), after 1/1 (`buys_cure=1`; the Alchemy cure is bought).
- `healthy` control: 1/1 in both phases, log byte-identical
  before/after (`97cff8b0…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness correction recorded in the verifier commit: BEFORE
  was corrected from the branch base to this step's actual before
  commit (the dimdoor step tip), and the tree-level scope assert now
  matches that step (`borg-escape.c` is proven by its own verifier).

## Scope and limits

Bounded town shopping scenarios in the three condition states. They
prove the store choice and the cure purchase; no survival claim.

## Exact next work

1. Agent evidence review plus required hosted checks on the stacked PR,
   then normal merge.
2. Frozen v1 cohort at the merged tip against the hurtstun tip cohort.
3. Receipts on issue 336, source 23 and checkpoint 1.

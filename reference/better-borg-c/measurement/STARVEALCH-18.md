# Starvealch-18 carry: verified product scenario

Consumer [359](https://github.com/diegoami/better-borg-c/issues/359),
canonical source
[18](https://github.com/diegoami/neo-angband-tracker/issues/18)
(starvealch-18; source fix `0fd9bc4bd`, evidence fixture `98aa8ad02`).
Applied as a `cherry-pick -x` of source
`0fd9bc4bd6f069e0cbab664e7541e3202764a9a4`, patch
`patches/0090-remove-hunger-alchemist.patch` (applied commit
`2260c791fb`).

## Defect

When the Borg carries no food, `borg_think_shop_buy_useful()` only
weighed the General Store and the Home, and a starving check let food
and Scrolls of Remove Hunger through — but written as
`tval != scroll && sval != Remove Hunger`, it let any scroll (or any
item with that sval) through instead of only that scroll. Remove Hunger
is sold by the Alchemist, which the store filter skipped, and the
scrolls were never counted as food, so buying one never raised the
Borg's power. A Borg with no food in town, with gold, and no General
Store food chose no shop at all. The correction admits the Alchemist
(store 4) in the filter when out of food, writes the check as "food, or
a Scroll of Remove Hunger", and counts those scrolls as food only when
no other food is carried.

## Pins and provenance

Before: merged main `0a75c0736` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). Both modules are otherwise pristine on the product except for
the carried urgentshop-23 store filter, which the source base predates:
the cherry-pick's 3-way merge preserves it (`k != 4` plus its comment)
alongside the new filter clause — the layered-module check proves the
source diff still reproduces the product modules. The verifier proves
that the source diff applied to the before modules equals the product
modules and that the applied commit is the only `src`/`lib` difference
from the before commit. Fixture: kit `98aa8ad02` at head `99db3c5ca`,
asserted byte-identical to the run pin.

## Verified results (`artifacts/sa18/scenarios`, 2026-11-09)

- `starving` — fail before, pass after: before nothing was bought
  (`buys_food=0 buys_remove_hunger=0`; the Alchemist was filtered out
  and the scroll was never counted as food), after the Remove Hunger
  scroll is bought (`buys_remove_hunger=1`).
- `general_food` control: 1/1 (General Store food bought) in both
  phases, log byte-identical before/after (`0f15bec6…`).
- `fed` control: 1/1 (nothing bought) in both phases, log
  byte-identical before/after (`30921762…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run; the
  cherry-pick's 3-way merge preserved the carried urgentshop-23 store
  filter (verified in the merged file).

## Scope and limits

Bounded town shopping scenarios in three food states. They prove the
store choice and the Remove Hunger purchase; no survival claim.

## Exact next work

1. PR from `task/carry-starvealch`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the spastic tip cohort.
3. Receipts on issue 359, source 18 and checkpoint 1.

# Modcredit-70 carry: verified product scenario

Consumer [311](https://github.com/diegoami/better-borg-c/issues/311),
canonical source
[70](https://github.com/diegoami/neo-angband-tracker/issues/70)
(modcredit-70; source fix `46c6188e`, evidence fixture `426696724`).
Fix applied as the first step of the mod-neck PR (`cherry-pick -x` of
source `46c6188eccb54bd0571e0a527145f7fd7ed756c3`, patch
`patches/0082-item-value-modifier-credit.patch`), stacked with the
neckdead-47 cleanup (patch 0083).

## Defect

The item value guess missed stat and other modifier bonuses, so items
whose worth is in their modifiers were misvalued (a ring of a useful
stat was sold while a worthless one was kept). The correction credits
the modifiers. Valuation only; no shopping outcome measured.

## Pins and provenance

Before: merged main `71995a193` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-item-analyze.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `426696724` at head
`0af6c5af0`, asserted byte-identical to the run pin.

## Verified results (`artifacts/mc70/scenarios`, 2026-11-08)

- `statring` / `speed` — fail before, pass after: before 0/1 (`sold_a=1
  sold_b=0`; the modifier-carrying item was misvalued and sold), after
  1/1 (`sold_a=0 sold_b=1`; the better item is kept).
- `nomods` control: 1/1 in both phases, log byte-identical before/after
  (`5d71af95…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness corrections recorded in the verifier commit: the
  generated statring assertion keys were corrected, and the tree-level
  scope check asserts the fix commit's own scope plus the documented
  stacked file (neckdead-47).

## Scope and limits

Bounded valuation scenario driving the production item value guess. It
proves the modifier credit; no shopping outcome claim.

## Exact next work

1. Step 2 (neckdead-47 cleanup) in the same PR with a per-step gate;
   agent evidence review plus required hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the lightwest tip cohort.
3. Receipts on issue 311, source 70 and checkpoint 1.

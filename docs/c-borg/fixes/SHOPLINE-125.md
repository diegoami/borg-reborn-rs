# Shopline-125 carry: verified product scenario

Consumer [307](https://github.com/diegoami/better-borg-c/issues/307),
canonical source
[125](https://github.com/diegoami/neo-angband-tracker/issues/125)
(shopline-125; source fix `9d35babe`, evidence fixture `330deefd6`).
Fix applied as the third step of the defend-rest-shop PR
(`cherry-pick -x` of source
`9d35babe8bd8e38b2d522e00ce159d740919718f`, patch
`patches/0080-shop-entrance-avoidance.patch`), stacked on banisharte-37
(patch 0078) and restmult-149 (patch 0079).

## Defect

The shop-entrance avoidance covered only one layout case, so the borg
could step onto (and enter) a shop that is not its goal while passing
it. The correction avoids every non-goal shop entrance. Movement
decision only; no shopping outcome measured.

## Pins and provenance

Before: the restmult-149 step tip `72e169b1ed` (snapshotted build
`build/native-before-shop`, gate result
`artifacts/native-before-shop/result.json`, 965/965 + 5/5 at that
commit). The verifier proves that the source diff applied to the before
`borg-flow.c` equals the product module and that the applied commit is
the only `src`/`lib` difference from the before commit. Fixture: kit
`330deefd6` at head `9ce7a0a47`, asserted byte-identical to the run pin.

## Verified results (`artifacts/sl125/scenarios`, 2026-11-08)

- `inline` — fail before, pass after: before 0/1
  (`steps_onto_other_shop=1`; only one layout case was covered), after
  1/1 (`steps_onto_other_shop=0`).
- `offrow` control: 1/1 in both phases, log byte-identical before/after
  (`a4b22217…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. A harness correction is recorded in the verifier commit: the
  generated `BEFORE` constant initially pointed at the branch base; it
  points at this step's actual before commit (the step-2 tip).

## Scope and limits

Bounded movement scenario on the production shop-entrance avoidance. It
proves the non-goal entrance is avoided; no shopping outcome claim.

## Exact next work

1. Agent evidence review plus required hosted checks on the stacked PR,
   then normal merge.
2. Frozen v1 cohort at the merged tip against the 0077-after cohort.
3. Receipts on issue 307, source 125 and checkpoint 1.

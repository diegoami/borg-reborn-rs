# Restmult-149 carry: verified product scenario

Consumer [306](https://github.com/diegoami/better-borg-c/issues/306),
canonical source
[149](https://github.com/diegoami/neo-angband-tracker/issues/149)
(restmult-149; source fix `e66740cc`, evidence fixture `e76b239b3`).
Fix applied as the second step of the defend-rest-shop PR
(`cherry-pick -x` of source
`e66740cc4563224ca5dddab5d7bd543d8c8f1342`, patch
`patches/0079-rest-mult-kill-timer.patch`), stacked on banisharte-37
(patch 0078) and the carried 0064 in the same file.

## Defect

The rest check dropped the multiplier-kill timer, so on the same turn as
a multiplier kill the borg rested one turn it should not have. The
correction keeps the timer in the check. Rest decision only; no combat
effect measured.

## Pins and provenance

Before: the banisharte-37 step tip `8f2669a1f1` (snapshotted build
`build/native-before-rest`, gate result
`artifacts/native-before-rest/result.json`, 965/965 + 5/5 at that
commit). The verifier proves that the source diff applied to the before
`borg-flow-misc.c` (which already carries the 0064 reverse-flow
correction) equals the product module and that the applied commit is the
only `src`/`lib` difference from the before commit. Fixture: kit
`e76b239b3` at head `c6052b4a`, asserted byte-identical to the run pin.

## Verified results (`artifacts/rm149/scenarios`, 2026-11-08)

- `sameturn` — fail before, pass after: before 0/1
  (`rests_one_turn_after_the_kill=1`; the timer was dropped from the
  rest check), after 1/1 (`rests_one_turn_after_the_kill=0`).
- `recent` control: 1/1 in both phases, log byte-identical before/after
  (`45fe361c…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. A harness correction is recorded in the verifier commit: the
  generated `BEFORE` constant initially pointed at the branch base; it
  points at this step's actual before commit (the step-1 tip), and the
  tree-level scope check asserts the fix commit's own scope plus the
  documented stacked file.

## Scope and limits

Bounded rest-decision scenario on the production rest check. It proves
the timer is kept; no survival claim.

## Exact next work

1. Step 3 (shopline-125) in the same PR with a per-step gate; agent
   evidence review plus required hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the 0077-after cohort.
3. Receipts on issue 306, source 149 and checkpoint 1.

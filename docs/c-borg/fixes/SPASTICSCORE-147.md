# Spasticscore-147 carry: verified product scenario on a consumer-designed fixture

Consumer [288](https://github.com/diegoami/better-borg-c/issues/288),
canonical source
[147](https://github.com/diegoami/neo-angband-tracker/issues/147)
(spasticscore-147; source fix `6e54def8`, source kit `5b38022e`).
Applied as a `cherry-pick -x` of source
`6e54def846e04ee7a6a203bfcb1623128fc7d696`, patch
`patches/0089-spastic-own-search-count.patch` (applied commit recorded
below). **This carry un-defers the earlier deferral** (recorded on
issue 288 and source 147): the source kit's choreography was designed
for the pristine base where the #146 reverse flow is broken and
exercises nothing once this product's patch 0064 lets the reverse flow
reach other grids. The fixture here is consumer-designed for the
0064-corrected flow, adapted from the source kit's arena; the source
fix commit and its own kit remain canonical.

## Defect

`borg_flow_spastic()` scores each candidate grid with a visitation
penalty read from `ag->xtra` — but `ag` had been reused by the
neighbour loops and points at the **north-west neighbour** when the
score is computed (for a north-wall candidate, a wall grid that is
never searched). A grid that had never been searched was refused when
that neighbour had been, and a grid that had been searched repeatedly
was searched again when that neighbour had not. The correction reads
the candidate grid's own count (`borg_grids[y][x].xtra`).

## Fixture (`docs/audit/c-reports/spasticscore-147-product/`)

22x22 closed room, ordinary granite north wall; the Borg stands on
C=(12,2). Fixture steps write the Borg's search history directly (the
`xtra` bookkeeping the production search itself maintains): C and its
west neighbour X=(11,2) carry 10 (both already searched); the corner
grids (1,2) and (20,2) carry 20, which the production
`xtra >= clevel` limit excludes from both phases (they would otherwise
dominate via their permanent-wall diagonals). Modes:

* `mismatch` — the disagreement the bug scores: C's north-west
  neighbour is a wall grid carrying 0.
* `consistent` — the north-west wall grids carry the same counts as
  the grids they are scored for; both penalties agree.

Observable: after the call, the chosen grid is the only grid with flow
cost 0; the fixture prints `OUTCOME rc goal_x goal_y`. With the bug the
flow re-picks C — the grid the Borg already searched — and reports no
step (`rc=0`, the goal is the Borg's own grid); after the fix the fresh
grid (13,2) wins and the call steps (`rc=1`).

## Pins and provenance

Before: merged main `aa96f142a` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-flow-misc.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: consumer-designed, pinned at
its own commit `3d6a0953cb` (the fixture is part of this carry; there is
no foreign kit pin).

## Verified results (`artifacts/sp147/scenarios`, 2026-11-09)

- `mismatch` — the flip: before the flow re-picks C=(12,2), the grid
  the Borg already searched (`rc=0`, the goal is the Borg's own grid so
  there is no step); after the fix the fresh grid (13,2) wins and the
  call steps (`rc=1`, `goal=(13,2)`).
- `consistent` control: both phases pick (13,2), `rc=1`, log
  byte-identical before/after (`39e0b25f…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness notes recorded in the verifier commit and the kit
  note above: the fixture is consumer-designed and pinned at its own
  commit; the diagnostic candidate dump is env-gated and does not run
  in verification.

## Scope and limits

Bounded bored-search scenarios on a flat test arena with written search
history. They prove the penalty source (the candidate's own count, not
its north-west neighbour's) and the resulting flow target; no survival
claim.

## Exact next work

1. PR from `task/carry-spastic`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the deadtake tip cohort.
3. Receipts on issue 288, source 147 (amending the deferral) and
   checkpoint 1.

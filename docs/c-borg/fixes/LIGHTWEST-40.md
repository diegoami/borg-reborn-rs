# Lightwest-40 carry: verified product scenario

Consumer [309](https://github.com/diegoami/better-borg-c/issues/309),
canonical source
[40](https://github.com/diegoami/neo-angband-tracker/issues/40)
(lightwest-40; source fix `6dd5874b`, evidence fixture `483a7a12c`).
Fix applied as `69e020d3b` (`cherry-pick -x` of source
`6dd5874b444b00b8185fce58c823002ab6c544ad`, patch
`patches/0081-light-west-radius-edge.patch`).

## Defect

The Borg works out which grids its light reaches. The east block adds
the three grids two columns east of the borg (the borg's row and the
rows above and below). The west block added the borg's row and then the
two grids two rows above and below — which the north and south blocks
add already — instead of the rows above and below. So the grids one row
up and down and two columns west were never marked as lit, and a
monster that had stood there was treated as not having been in the
light. The correction adds the grids one row above and below. Light
bookkeeping only; no survival claim.

## Pins and provenance

Before: merged main `4d4f2c13a` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-cave-light.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `483a7a12c` at head
`8e9e1da6a`, asserted byte-identical to the run pin.

## Verified results (`artifacts/lw40/scenarios`, 2026-11-08)

- `west` — fail before, pass after: before 0/1 (`ghost_kept=1`; a monster
  that had stood one row off and two columns west counted as
  never-in-light), after 1/1 (`ghost_kept=0`).
- `east` control: 1/1 in both phases, log byte-identical before/after
  (`6a1b9a56…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. A harness correction is recorded in the verifier commit: the
  verifier was generated from a stacked-carry file and inherited its
  tree-level scope assertion (expecting the defend-rest-shop stacked
  files); reverted to the single-file scope this carry actually has.

## Scope and limits

Bounded light-bookkeeping scenario driving the production
`borg_update_light`. It proves the west-radius-edge grids are marked;
no survival claim.

## Exact next work

1. PR from `task/carry-lightwest`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the defend-rest-shop tip
   cohort (`4d4f2c13a`).
3. Receipts on issue 309, source 40 and checkpoint 1.

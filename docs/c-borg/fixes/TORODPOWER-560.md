# Torodpower-560 carry: verified product scenario

Consumer [369](https://github.com/diegoami/better-borg-c/issues/369),
canonical source
[560](https://github.com/diegoami/neo-angband-tracker/issues/560)
(torodpower-560; source fix `fc444ab17`, evidence fixture `12a59c5c4`).
Applied as a `cherry-pick -x` of source
`fc444ab17e8dee8ccdbb908f4e3bb760114089b8`, patch
`patches/0095-tele-other-rod-power.patch` (applied commit
`ce6af424b4`).

## Defect

The Teleport Other availability trait (`BI_ATPORTOTHER`) was filled
only from a carried Wand of Teleport Other (its charges) and from the
Teleport Other spell. A Rod of Teleport Other added nothing, so
`borg_power()` did not rise for it and the shop step never bought one:
a Warrior with enough gold left the rod in the Black Market. The
correction counts each carried rod as one, like one wand charge; a Borg
that already has the spell or a full stack of wand charges values it as
before.

## Pins and provenance

Before: merged main `188277504` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-trait.c` is otherwise pristine on the product except for
the carried starvealch-18 counting changes, which the source base
predates: the 3-way merge preserves them and the layered-module check
proves the source diff still reproduces the product modules. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `12a59c5c4` at head
`4141bfa68`, asserted byte-identical to the run pin.

## Verified results (`artifacts/tp560/scenarios`, 2026-11-09)

- `rod` — fail before, pass after: before the rod was left in the shop
  (`buys_rod=0`), after it is bought (`=1`).
- `wand` control: 1/1 in both phases (a Borg already carrying wand
  charges does not buy the rod), log byte-identical before/after
  (`e138ca84…`).
- `mage` control: 1/1 in both phases (a Borg with the spell does not
  buy the rod), log byte-identical before/after (`e914400a…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run; the 3-way
  merge preserved the carried starvealch-18 counting changes in
  borg-trait.c (verified in the merged file).

## Scope and limits

Bounded shop-purchase scenarios (rod shopper, wand owner, spell owner).
They prove the availability counting and the purchase; no survival
claim.

## Exact next work

1. PR from `task/carry-torodpower`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the tradehist tip cohort.
3. Receipts on issue 369, source 560 and checkpoint 1.

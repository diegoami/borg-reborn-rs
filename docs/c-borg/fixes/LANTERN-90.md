# Lantern-90 carry: verified product scenario

Consumer [296](https://github.com/diegoami/better-borg-c/issues/296),
canonical source
[90](https://github.com/diegoami/neo-angband-tracker/issues/90)
(CBUG-0074, lantern-90; source fix `bd4698a8`, evidence fixture
`505180c12`). Fix applied as `69a62b869` (`cherry-pick -x` of source
`bd4698a850e9fb926d74c9a26be298716a33e5ba`, patch
`patches/0076-lantern-scan-first.patch`).

## Defect

With no flask of oil, `borg_refuel_lantern()` looks for a spare lantern
to refuel from. The scan is meant to walk the lanterns from the first
one, skipping Everburning and empty ones, but it started at
`borg_slot(TV_LIGHT, sv_light_lantern)` — and lanterns do not stack, so
`borg_slot()` returns the last lantern in the pack. If that one was
empty or Everburning the scan ended at once, an earlier lantern with
fuel was never used, and the wielded lantern ran dry. The correction
starts the scan at the first pack slot and falls back to -1 when no
lantern is found. Refuelling decision only; no light-duration
measurement beyond the fixture.

## Pins and provenance

Before: merged main `f7e489157` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-light.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `505180c12` at head
`8ba35640`, asserted byte-identical to the run pin.

## Verified results (`artifacts/ln90/scenarios`, 2026-11-08)

- `empty` — fail before, pass after: before 0/1 (`refuels=0`; the scan
  started at the last lantern and ended at once on the dead/Everburning
  one, so the fueled spare earlier in the pack was never used), after
  1/1 (`refuels=1`; the fueled spare is found and used).
- `single` control: 1/1 in both phases, log byte-identical before/after
  (`a215c696…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on the first complete run; no harness
  corrections were needed.

Native gate on the product: **965/965 unit + 5/5 frontend** (before
snapshot gate and product tip gate); tooling **129/129**.
Prior-fixture matrix not rerun (uncommitted runner; known gap recorded
on checkpoint 1 and the milestone 6 review).

## Scope and limits

Bounded refuel scenario driving the production `borg_refuel_lantern`
decision. It proves the fueled spare is found and used when the last
lantern is dead; no survival claim.

## Exact next work

1. PR from `task/carry-lantern`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort seeds 11/29/47 at the merged tip against the
   0075-after cohort at `f7e489157`.
3. Receipts on issue 296, source 90 and checkpoint 1.

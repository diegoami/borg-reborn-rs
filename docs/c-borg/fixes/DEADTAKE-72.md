# Deadtake-72 carry: verified product scenario

Consumer [338](https://github.com/diegoami/better-borg-c/issues/338),
canonical source
[72](https://github.com/diegoami/neo-angband-tracker/issues/72)
(deadtake-72; source fix `9492d74ce`, evidence fixture `6332987b90`).
Applied as a `cherry-pick -x` of source
`9492d74ce57ae63fca41866113bc7f35463c5704`, patch
`patches/0088-lunal-take-dead-entries.patch` (applied commit recorded
below).

## Defect

The Lunal mode take routine tried to skip dead entries of the object
list with a test on the address of the kind record (`!k_ptr`), which is
never NULL, so entries that had been forgotten (zeroed, kind 0) were
treated like live ones. Their location is (0,0), the corner of the
level, and when that grid was in view it became a flow target alongside
the real items, so the Borg could step towards the corner instead of
the item. The correction tests the kind index (`take->kidx`), as the
other take routines do.

## Pins and provenance

Before: merged main `8b26042e0` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-flow-take.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `6332987b90` — the evidence branch holds a single
commit (the package itself; captured artifacts pending), so the
generator commit is the run pin.

**Kit note:** at carry time the kit's generators (scenario, runner,
profile) were complete, but the lane's captured before/after rows had
not been pushed yet ("follow in a later commit" on the evidence head).
This verifier therefore asserts the rows measured on consumer cores;
the flip and control follow from the scenario's own self-check
(`native-c/deadtake-72 finished: 1/1 passed`).

**Cross-check complete (2026-11-09, milestone-8 review):** the lane
pushed its captured rows at `35987d71` and all four (phase, mode)
combinations agree with the consumer-measured rows — before/`dead-entry`
fails with `walks_toward_gold=0`; after/`dead-entry` passes with
`walks_toward_gold=1`; both phases' `live-only` pass with
`walks_toward_gold=1`. The pending obligation recorded here, on
consumer #338, on source 72 and in the checkpoint is discharged; the
receipts stand unamended.

## Verified results (`artifacts/dt72/scenarios`, 2026-11-09)

- `dead-entry` — fail before, pass after: before 0/1
  (`walks_toward_gold=0`; the forgotten corner entry competed as a flow
  target), after 1/1 (`walks_toward_gold=1`).
- `live-only` control: 1/1 in both phases, log byte-identical
  before/after (`1aa7b4b7…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness notes recorded in the verifier commit and the kit
  note above: the run pin is the generator commit itself (the evidence
  branch holds a single commit), and the asserted rows are the measured
  consumer rows pending the lane's captured artifacts.

## Scope and limits

Bounded item-flow scenarios with and without a forgotten dead entry in
view. They prove the flow target selection; no survival claim.

## Exact next work

1. PR from `task/carry-deadtake`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the dim-urg tip cohort.
3. Cross-check the lane's recorded rows when published; receipts on
   issue 338, source 72 and checkpoint 1.

# Wearskip-85 carry: verified product scenario

Consumer [294](https://github.com/diegoami/better-borg-c/issues/294),
canonical source
[85](https://github.com/diegoami/neo-angband-tracker/issues/85)
(CBUG-0069, wearskip-85; source fix `5d73e0c21`, evidence fixture
`620235082`). Fix applied as `7b97f0f88` (`cherry-pick -x` of source
`5d73e0c2120b450428f2417db325041c1b6d3da9`, patch
`patches/0075-best-stuff-skip-count.patch`).

## Defect

At home the Borg plans its best set of equipment and then makes the
changes one step at a time through `borg_one_step_wearing_best()`. When
the next change cannot be made because the pack or the home is full, the
function calls itself with skip + 1 to try the following change instead.
The skip counter was declared inside the search loop, so it was reset to
skip on every pass and never reached zero: with skip > 0 every remaining
planned change was skipped, not just the first skip of them. The step
then did nothing, and when the list ended on empty entries the plan was
discarded. A change that could still be made — for example wearing a
cloak into an empty slot, which also frees the pack slot the blocked
change needs — was never tried. The correction declares the counter
once, before the loop.

## Pins and provenance

Before: merged main `3e50961cd` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-item-wear.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `620235082` at head
`3735a7e7`, asserted byte-identical to the run pin.

## Verified results (`artifacts/ws85/scenarios`, 2026-11-08)

- `tight` — fail before, pass after: before 0/1 (`actions=2
  wears_scale_mail=0 wears_cloak=0`; with one change blocked, every
  remaining planned change was skipped and the plan dropped), after 1/1
  (`actions=5 wears_scale_mail=1 wears_cloak=1`; the cloak goes into the
  empty slot even though a pack-needing change was skipped).
- `roomy` / `nocloak` controls: 1/1 in both phases, logs byte-identical
  before/after (`36bed668…`, `ea86f8b1…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on the first complete run; no harness
  corrections were needed.

Native gate on the product: **965/965 unit + 5/5 frontend** (before
snapshot gate and product tip gate); tooling **129/129**.
Prior-fixture matrix not rerun (uncommitted runner; known gap recorded
on checkpoint 1 and the milestone 6 review).

## Scope and limits

Bounded home best-equipment scenario driving the production one-step
plan. It proves the plan is no longer dropped when one change is blocked
and the still-possible changes are made; no survival claim.

## Exact next work

1. PR from `task/carry-wearskip`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort seeds 11/29/47 at the merged tip against the
   0073/0074-after cohort at `3e50961cd`.
3. Receipts on issue 294, source 85 and checkpoint 1.

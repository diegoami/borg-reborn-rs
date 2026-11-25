# Map-wall-capture carry: verified product scenario, ready for adoption

Consumer [148](https://github.com/diegoami/better-borg-c/issues/148), canonical
source [212](https://github.com/diegoami/neo-angband-tracker/issues/212)
(mapwall-212). Fix applied as
`641234dbd3e5d661e911e810e55df991a3a3a9b3` (import of source PR387
`5bd789ee0ff0a471162d7e3ebefb1ac4d501803c`, patch
`patches/0039-map-wall-change-capture.patch`).

## Defect

In `borg_update_map()` the `old_wall` capture ran AFTER the grid's feature
had already been overwritten with what the game shows now, so the
wall-change comparison compared the new feature against itself and never
detected a change: stale flow data, stone-to-mud knowledge, icky detection
flags and their timestamps survived a grid changing under the Borg (for
example a door closing or opening on a watched grid). The correction moves
the capture before the overwrite. No memory error; no death or survival
claim.

## Pins and layered provenance

Before build: thirty-eight-carry `21fbfba5c4d3a2cc77707fea885590479cec18c3`
(retained tree `../outoflight`, normal and ASan cores, gate recorded at
that commit, source-identical to merged main `c32c5d6d5` over
src/lib/CMakeLists.txt/cmake — the recording-feature merge changed nothing
under the build inputs). `borg-update.c` already carries 0018, so the
verifier proves that the source diff applied to the before module equals
the product module; the applied commit's diff body is byte-identical to
the source commit's diff. The source evidence fixture at `9fe10e42` is
used unchanged.

## Verified results (`artifacts/m212/scenarios`, 2026-11-07)

- normal-before `door-close` (floor → closed door between two production
  map passes): 0/1, `invalidated=0 flow255=0` — the stale flow, knowledge,
  icky flags and stamps all survive.
- normal-after `door-close`: 1/1, `invalidated=1 flow255=1` — fully
  invalidated, the flow reset to the new-wall value.
- `door-open` (closed door → floor): before 0/1 stale; after 1/1,
  `invalidated=1 flow255=0` — knowledge/icky and the stamps cleared while
  the flow value stays (only new walls get 255).
- `no-change`: nothing invalidated on every phase, 1/1, byte-identical
  before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **54/54** (the recording-feature tooling set on
current main). Regression matrix
(`artifacts/m212-carry/regression.py`, local-only): all **126 prior native
fixture cases** match the recorded thirty-eight-carry tally — 123
whole-log identical (including the brand-240, stairtrack-232, acidarc-252
and outoflight-229 cases), the dispel-248 undead case passing with its
asserted losdispel log change, and the two dispel-259 cases asserted-red
at the pinned post-losdispel values.

## Frozen v1 measurement: legitimate divergence, outcomes unchanged

Unlike carries 0035-0038, this fix is **behavior-active on ordinary
maps**: corrected wall-change invalidation changes the Borg's flow
decisions wherever a feature changes during play. Post-merge frozen v1
measurement on a prepared observer build of merged main (seeds 11/29/47,
cap 20000, wall 120, stall 30):

- All three seeds still finish **turn_cap "(alive and well)"** — no
  deaths, no survival regression (final states: seed11 hp 28 depth 1,
  seed29 hp 49 depth 2, seed47 hp 58 depth 1).
- Traces diverge from the recorded 34-carry run, as expected from the
  fix: seed11 substantially (3,746 added / 3,552 removed of 4,453
  recorded lines; first new note `Flowing toward dark-1 at cost 1`
  beside a door interaction at tick 1340), seed47 substantially (3,397 /
  2,720 of 4,598; an added flow note at tick 2757), seed29 barely (32 /
  31 of 5,503; a bravery boost and level-exit re-evaluation at tick
  6037). The examined divergence points are additional or re-ordered
  flow notes after ordinary feature changes — consistent with the
  correction's purpose. Three divergence points were examined (one per
  seed); the remaining changed lines are not individually inspected, and
  this is a decision-stream change, not a survival or speed claim.
- This is the measured impact of the carry, recorded rather than
  asserted away; the recorded 34-carry traces remain valid at their
  recorded commits.

## Scope and limits

Bounded native production map-pass scenario with one watched grid and
three feature transitions. It proves the wall-change detection and its
correction, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-mapwall`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and record the divergence
   from the 34-carry traces (this fix is behavior-active; see above).
3. Mark 148 carried/open+patch-active; receipts on issue 148, source 212
   and checkpoint 1; evidence archive under `artifacts/m212-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`).

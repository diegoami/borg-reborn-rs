# Enlightenment-use carry: verified product scenario, ready for adoption

Consumer [157](https://github.com/diegoami/better-borg-c/issues/157), canonical
source [81](https://github.com/diegoami/neo-angband-tracker/issues/81)
(CBUG-0065, priority low). Fix applied as
`2c3d8172c38bd83090f6869f775893b31216bf13` (import of source PR275
`526906c0f367cd690bdd92b99ac1e114fd84b1a2`, patch
`patches/0040-use-enlightenment.patch`).

## Defect

In `borg_use_things()` the force-item potion branch recognized
Enlightenment and guarded town use, but never called the potion helper in
that branch — a known Enlightenment potion sat unused in the dungeon while
stairs beyond walls stayed unknown. The correction adds the missing
`borg_quaff_potion()` call after the town guard (and corrects a scroll
comment to potion). The town prohibition, unknown-flavor skip,
unrelated-potion skip and the Augmentation path are untouched. Priority
low: a measured information change (map knowledge), no combat benefit
measured; saving the potion for a later level is a separate strategy
question.

## Pins and layered provenance

Before build: thirty-nine-carry `641234dbd3e5d661e911e810e55df991a3a3a9b3`
(retained tree `../mapwall`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `d2db13a13` over
src/lib/CMakeLists.txt/cmake — the recording-feature and death-phase-docs
merges changed nothing under the build inputs). `borg-item-use.c` already
carries 0005/0006, so the verifier proves that the source diff applied to the
before module equals the product module; the applied commit's diff body is
byte-identical to the source commit's diff. The source evidence fixture at
`8eb4b995` (evidence PR276) is used unchanged.

## Verified results (`artifacts/e065/scenarios`, 2026-11-07)

- normal-before `dungeon` (known Enlightenment potion, down stair beyond a
  wall): 0/1, `used=0 potions_spent=0 stairs_known=0` — the potion sits
  unused and the stair stays unknown.
- normal-after `dungeon`: 1/1, `used=1 potions_spent=1 stairs_known=1
  stairs_feat=6` — one potion drunk; the native player map memory now
  knows the down stair (feat matches the arena's native_stairs=6).
- Five controls, 1/1 on every phase, byte-identical before/after:
  `town-control` (Enlightenment kept in town), `unknown-control`
  (unknown flavor kept), `unrelated-control` (True Seeing kept),
  `augmentation-dungeon` and `augmentation-town` (Augmentation drunk in
  both, as before the fix).
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **56/56** (the recording-feature and
death-phase-feedback tooling sets on current main). Regression matrix
(`artifacts/e065-carry/regression.py`, local-only): all **129 prior native
fixture cases** match the recorded thirty-nine-carry tally — 126
whole-log identical (including the mapwall-212 cases), the dispel-248
undead case passing with its asserted losdispel log change, and the two
dispel-259 cases asserted-red at the pinned post-losdispel values.

## Frozen v1 measurement: behavior-neutral on the cohort

Post-merge frozen v1 measurement on a prepared observer build of merged
main (seeds 11/29/47, cap 20000, wall 120, stall 30): all three seeds
finish **turn_cap "(alive and well)"** with **whole-trace SHA256
identical to the thirty-nine-carry (mapwall) run** — this carry is
behavior-neutral on the cohort (no v1 Warrior carried a recognized
Enlightenment potion at depth >= 1 during these runs). The seeds' traces
diverge from the older 34-carry recording because mapwall-212 (patch
0039) is behavior-active; that divergence predates this carry and is
recorded in `MAP-WALL-CAPTURE.md`.

## Scope and limits

Bounded native useful-item scenario in a quiet constructed arena accepted
by the safety check. It proves the missing drink and the information it
restores; it does not run full exploration, travel or combat, and it does
not measure a faster escape, avoided attack, injury or death benefit.
Saving the potion for a later level is a separate strategy question, out
of scope.

## Exact next work

1. PR from `task/carry-enlightenment`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   thirty-nine-carry (mapwall) run — this carry changes behavior only if
   a v1 Warrior carries a recognized Enlightenment potion at depth >= 1.
3. Mark 157 carried/open+patch-active; receipts on issue 157, source 81
   and checkpoint 1; evidence archive under `artifacts/e065-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`).

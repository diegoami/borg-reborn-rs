# Healchance-364 carry: verified product scenario

Consumer [363](https://github.com/diegoami/better-borg-c/issues/363),
canonical source
[364](https://github.com/diegoami/neo-angband-tracker/issues/364)
(healchance-364; source fix `b366550f7`, evidence fixture `3adc1b812`).
Applied as a `cherry-pick -x` of source
`b366550f776d4d7a6ac52031529f3254a5914d38`, patch
`patches/0092-heal-chance-signs.patch` (applied commit `73322d06cd`).

## Defect

`borg_heal()` draws a chance roll and returns without healing below the
threshold, so a lower chance means a skip is more likely. Three
adjustments before the threshold test had the sign opposite to their
comments: fighting a unique subtracted 10 (healed *less* next to a
unique); the danger-at-current-hp case subtracted 75 (a borg whose
danger was at or above its current hp but below maximum skipped every
healing step whenever it had lost less than 75% of its hit points, and
three times in four above that); and the risky-Borg adjustment added 5
(a risky Borg healed *more* often). The correction flips the three
signs; the class adjustment and thresholds are unchanged. A borg in
real danger carrying something that heals enough now uses it instead of
falling through to escapes such as reading Word of Recall.

## Pins and provenance

Before: merged main `e4cbcf27b` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-caution.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `3adc1b812` at head
`f88fee1e3`, asserted byte-identical to the run pin.

## Verified results (`artifacts/hc364/scenarios`, 2026-11-09)

- `danger` — fail before, pass after: before no healing at all
  (`heals=0`; the sign flip made the borg skip every healing step
  despite real danger), after 100 heals with the heal witnessed and the
  potion confirmed (`witness_healed=1, drank_healing=1`,
  `hp_after=374`).
- `unique` — flips 18→32 heals (next to a unique, healing is now more
  likely as the comment states).
- `risky` — flips 27→20 heals (a risky Borg now heals less often in
  the fight, as the comment states).
- `control` — 1/1 in both phases, identical rows and byte-identical
  log (`3b25a0b3…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness note recorded in the verifier commit: the kit's
  `scenario.c` calls `tval_is_potion()` without including `obj-tval.h`
  (an implicit-declaration error on this compiler); the verifier forces
  the include rather than editing the pinned fixture.

## Scope and limits

Bounded heal-decision scenarios in three danger configurations. They
prove the chance direction and the resulting heal; no survival claim.

## Exact next work

1. PR from `task/carry-healchance`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the torodvalue tip cohort.
3. Receipts on issue 363, source 364 and checkpoint 1.

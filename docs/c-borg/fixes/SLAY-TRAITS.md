# Slay traits carry: verified product scenario, ready for adoption

Consumer [69](https://github.com/diegoami/better-borg-c/issues/69), canonical
source [239](https://github.com/diegoami/neo-angband-tracker/issues/239)
(slay-239, source priority high). Fix applied as
`6bcdd28ace1248e223ac381d61051964bb50a592` (import of source PR304
`9ab78ae605ca4463268a669121d69b216285c2f1`, patch
`patches/0017-slay-trait-accumulation.patch`).

## Defect

The inventory scan in `borg_notice_equipment()` assigned the eight slay
traits per equipment slot instead of accumulating them, so the last
occupied slot — usually the light, with no slays — erased the wielded
weapon's slays: a level-31 warrior's Slay Animal blade was valued exactly
like a plain one (17, identical to the same attack with the slay removed
by hand; the forced-on reading is 20). The correction accumulates across
slots and derives the never-set kill traits (the race-destroying slays
share the race flag with the largest multiplier), whose ×5 thrust
branches were dead.

## Pins and provenance

Before build: sixteen-carry `b2d5128b721958660a4cf892c4b2671f4384adcd`
(retained tree `../vampire-strike`, normal and ASan cores, gate at
`81b15066`, source-identical to `b2d5128b` over src/lib).
`borg-trait.c` is pristine in this product, so the imported module is
byte-identical to the reviewed source commit (module SHA256
`92fd28f2aac5d900be47b1cf1304fb4cc854cb970bf6af34e35fa9b56cf0c23c`).

## Verified results (`artifacts/slay/scenarios`, 2026-11-06)

- normal-before `animal`: 0/1, `auto=17 == without=17 < with=20` — the
  weaponless light slot erased the weapon's Slay Animal.
- normal-after `animal`: 1/1, `auto=20 == with=20 > without=17` — the
  slay counts, matching the forced-on reading exactly.
- `nonanimal` control (kobold, no animal flag): `19/19/19`, 1/1 on every
  phase, byte-identical logs before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/slay-carry/regression.py`): all **79 prior native
fixture cases whole-log identical** to their baselines (74 fourteen-carry
+ 2 munchkin + 3 vampire). Tooling 29/29.

## Scope and limits

Bounded native rating scenario: the thrust value of the wielded weapon
with the trait as the notice computed it, forced off and forced on;
weapon/torch placement per the report. Trait figures are the Borg's
heuristic; no combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-slay-traits`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 69 carried/open+patch-active; receipts on issue 69, source 239
   and checkpoint 1; evidence archive under `artifacts/slay-carry/`.
4. Then recheck source207: the ranked lead 290 (queued-detection panel
   overrun, memory-safety) still has segfaulting source harness runs and
   is not yet demonstrated-ready; 260/263/258/259 are undemonstrated
   valuation leads. With no ready carry, the queue returns to the
   documented order (low-priority 81 / discretionary death work) unless a
   new demonstration lands.

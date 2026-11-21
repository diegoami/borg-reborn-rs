# Staff-holiness carry: verified product scenario, ready for adoption

Consumer [87](https://github.com/diegoami/better-borg-c/issues/87), canonical
source [262](https://github.com/diegoami/neo-angband-tracker/issues/262)
(staff-262). Fix applied as `13fe885cb11a61c1a2ff9c2e53dd338b86bbddb0`
(import of source PR329 `fe34f38d0c998cf34ace02d09bf97774f38e65df`, stacked
on the losdispel correction already carried as `0489305b`; patch
`patches/0023-staff-holy-radius.patch`).

## Defect

The staff-holiness maneuver kept the bolt radius of zero and valued a
single harpy on a line: 28; the LOS-dispel radius of ten values the sight
lines: 60. The lesser-acid-bolt control (a genuine single-target
activation) is byte-identical at 28. The Wonder half of the report is
deliberately left a bolt (aimed random effect).

## Pins and layered provenance

Before build: twenty-two-carry `7ea6c922e54f033b95ec665fdc89c6d8443eeb5a`
(retained tree `../losdispel`, normal and ASan cores, gate at `0489305b`,
source-identical to `7ea6c922e` over src/lib). `borg-fight-attack.c`
already carries 0011/0013/0014/0016/0019/0020/0021/0022; the verifier
proves the source diff applied to the before module equals the product
module.

## Verified results (`artifacts/s262/scenarios`, 2026-11-06)

- normal-before `holy`: 0/1, `holy=28` (a single harpy on a line).
- normal-after `holy`: 1/1, `holy=60` — exactly the source evidence's
  value, the sight lines valued.
- `control` (lesser acid bolt): `bolt2=28`, 1/1 every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/s262-carry/regression.py`): all **91 prior native
fixture cases** pass — 89 identical plus the two documented
losdispel-326 interactions carried forward asserted (dispel-248 undead
`direct=−1→0`; dispel-259 both modes +296). Tooling 29/29.

## Scope and limits

Bounded rating scenario: the maneuver values for the worn staff-holiness
activator and the genuine single-target control. Rating figures are the
Borg's heuristic; no combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-staff-holy`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 87 carried/open+patch-active; receipts on issue 87, source 262 and
   checkpoint 1; evidence archive under `artifacts/s262-carry/`.
4. Then recheck source207 for newer demonstrations (source-ranked 261,
   then 240/241/246/250/287) before low-priority 81 or discretionary
   death work.

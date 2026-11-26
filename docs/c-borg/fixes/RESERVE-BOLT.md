# Reserve bolt carry: verified product scenario, ready for adoption

Consumer [78](https://github.com/diegoami/better-borg-c/issues/78), canonical
source [258](https://github.com/diegoami/neo-angband-tracker/issues/258)
(reserve-258). Fix applied as `21c672a856079167820d22dbf299f39c98faa24d`
(import of source PR318 `7b4ecd6b888a1d87b526f273c769449eb013ad9b`, patch
`patches/0020-reserve-bolt-veto-scope.patch`).

## Defect

The monster loop in `borg_attack_aux_spell_bolt_reserve()` broke at the
first live monster, so the hit-points veto applied to whatever was scanned
first instead of the near monsters it describes. A level-14 mage with no
mana (the emergency premise) facing a kobold two grids away while a young
red dragon stands fourteen grids away — observed first, so it owns the
lower kill index — had its emergency magic missile valued at 0, while
`near_monsters` could never exceed one and the fainting guard never
failed. The correction scans all monsters and scopes the veto to the near
ones: the dragon is skipped and the maneuver is valued at 40.

## Pins and layered provenance

Before build: nineteen-carry `db3dd535a7f46fae5db9116361b80ac1e020329b`
(retained tree `../activ-lookup`, normal and ASan cores, gate at
`7323c8df`, source-identical to `db3dd535a` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014/0016/0019; the
verifier proves the source diff applied to the before module equals the
product module.

## Verified results (`artifacts/r258/scenarios`, 2026-11-06)

- normal-before `veto`: 0/1, `value=0` — the distant dragon vetoes the
  emergency bolt for the nearby kobold.
- normal-after `veto`: 1/1, `value=40`.
- `control` (kobold only): `value=50`, 1/1 on every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/r258-carry/regression.py`): all **85 prior native
fixture cases whole-log identical** to their baselines (83 + 2 activ).
Tooling 29/29.

## Scope and limits

Bounded rating scenario: the exact production call the chooser makes for
the emergency magic missile, target list filled the way the chooser fills
it. Rating figures are the Borg's heuristic; no combat-outcome, frequency
or survival claim.

## Exact next work

1. PR from `task/carry-reserve-bolt`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 78 carried/open+patch-active; receipts on issue 78, source 258 and
   checkpoint 1; evidence archive under `artifacts/r258-carry/`.
4. Then recheck source207 for newer demonstrations (source-ranked 259/
   262/261) — with none ready, the queue returns to the documented order
   (low-priority 81 / discretionary death work).

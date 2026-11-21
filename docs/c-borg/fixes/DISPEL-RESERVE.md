# Dispel reserve carry: verified product scenario, ready for adoption

Consumer [81](https://github.com/diegoami/better-borg-c/issues/81), canonical
source [259](https://github.com/diegoami/neo-angband-tracker/issues/259)
(dispel-259). Fix applied as `c9f24544fe6fa309ad9ea926952e45db679f060d`
(import of source PR320 `e5da34177368960c9707df6c7b8d55e6c4ab081a`, patch
`patches/0021-dispel-reserve-single-penalty.patch`).

## Defect

The mana-needed-for-final-teleport penalty in
`borg_attack_aux_spell_bolt_reserve()` was applied by two back-to-back
blocks — one keyed on the class reserve and one on a fixed six whose
condition is a strict subset of every class reserve (all are six or more)
— so the penalty simply doubled. A level-21 priest five points inside the
final-teleport band casting Dispel Undead against three vampire bats
valued at −21127 before the correction and −10627 after, exactly one
750-times-cost block (10500) apart.

## Pins and layered provenance

Before build: twenty-carry `ff1e4085a517cc27deeb867e54e99de3d1cc8380`
(retained tree `../reserve-bolt`, normal and ASan cores, gate at
`21c672a8`, source-identical to `ff1e4085a` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014/0016/0019/0020; the
verifier proves the source diff applied to the before module equals the
product module. Process note: two verifier-assert iterations (delta
magnitude then direction) are retained under `artifacts/d259/scenarios-
arithmetic-fix` and `scenarios-assert-direction-1`; the fixture's own
assertions passed unchanged throughout — only the runner's redundant
post-hoc delta check was corrected.

## Verified results (`artifacts/d259/scenarios`, 2026-11-06)

- normal-before `reserve`: 0/1, `value=−21127` (doubled penalty).
- normal-after `reserve`: 1/1, `value=−10627` — exactly the source
  evidence's value, one 10500 block above the before.
- `clear` control (out of band): `value=−127`, 1/1 every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/d259-carry/regression.py`): all **87 prior native
fixture cases whole-log identical** to their baselines (85 + 2 reserve).
Tooling 29/29.

## Scope and limits

Bounded rating scenario: the exact production chooser call for Dispel
Undead with the reserve-band mana state the report describes. Rating
figures are the Borg's heuristic; no combat-outcome, frequency or
survival claim.

## Exact next work

1. PR from `task/carry-dispel-reserve`, agent evidence review plus
   required hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 81 carried/open+patch-active; receipts on issue 81, source 259
   and checkpoint 1; evidence archive under `artifacts/d259-carry/`.
4. Then recheck source207 for newer demonstrations (source-ranked 262/
   261) — with none ready, the documented order resumes.

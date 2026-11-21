# Blind Frighten carry: verified product scenario, ready for adoption

Consumer [90](https://github.com/diegoami/better-borg-c/issues/90), canonical
source [261](https://github.com/diegoami/neo-angband-tracker/issues/261)
(blind-261). Fix applied as `713d9ad4603d7bc46eaf32de3469fa7344e6f892`
(import of source PR331 `a49b7a6c15f6353cd427af28903355fe258af4d5`, patch
`patches/0024-blind-frighten-fear.patch`).

## Defect

The blind-creature maneuver casts Frighten but passed the **confusion**
attack type to the launcher: against dragons whose danger the model
suppresses by 408 each under confusion but not at all under fear
(probe-recorded), the fear spell carried a phantom valuation of 955 —
chooser-winning confusion it cannot deliver. The one-token correction
matches the dedicated Frighten maneuver's fear scoring.

## Pins and layered provenance

Before build: twenty-three-carry `db40081f19b43ddc29964b06c1987ffc5b5f3072`
(retained tree `../staff-holy`, normal and ASan cores, gate at
`13fe885cb`, source-identical to `db40081f1` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014/0016/0019/0020/
0021/0022/0023; the verifier proves the source diff applied to the before
module equals the product module.

## Why the control expectation is product-derived (measured)

The pinned control asserts the dedicated Frighten maneuver at exactly −5
(pristine). On the layered product that maneuver measures **−4** — the
losdispel-326 arrival-break correction removed a leaked −1 in the launch
path — on both the before and after builds. The carry's actual contract
(blind must value exactly what the dedicated Frighten maneuver values)
holds unchanged: after the fix `blind=-4 == frighten=-4`. The adapted
control (`tools/blind-frighten-scenario.c`, two edits plus the provenance
header) computes the expectation from the same production calls the
chooser uses (`borg_launch_bolt(0, clevel, TURN_ALL, ...)` minus the mana
cost), so no pristine constant is baked in and no assertion is weakened;
the dedicated maneuver stays byte-identical before/after. The original
red attempt is retained under `artifacts/b261/scenarios`.

## Verified results (`artifacts/b261/adapted`, 2026-11-06)

- normal-before `blind`: 0/1, `blind=955` (phantom confusion valuation),
  `frighten=-4`.
- normal-after `blind`: 1/1, `blind=-4 == frighten=-4` — exactly the
  dedicated maneuver's value.
- `control` (dedicated Frighten): `frighten=-4` (production-derived
  expectation, `launch=16` printed), 1/1 on every phase,
  byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/b261-carry/regression.py`): all **93 prior native
fixture cases** pass (91 identical plus the two documented losdispel
interactions asserted). Tooling 29/29.

## Scope and limits

Bounded rating scenario: the maneuver values for Blind Creature and the
dedicated Frighten, against dragons whose danger suppression differs
under confusion and fear. Rating figures are the Borg's heuristic; no
combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-blind-frighten`, agent evidence review plus
   required hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 90 carried/open+patch-active; receipts on issue 90, source 261
   and checkpoint 1; evidence archive under `artifacts/b261-carry/`.
4. Then brand-240 (brand-need counted twice, PR333/334, low priority),
   then recheck source207 for newer demonstrations (246/250/287).

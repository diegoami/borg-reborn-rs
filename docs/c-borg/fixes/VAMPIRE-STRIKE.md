# Vampire Strike carry: verified product scenario, ready for adoption

Consumer [66](https://github.com/diegoami/better-borg-c/issues/66), canonical
source [257](https://github.com/diegoami/neo-angband-tracker/issues/257)
(vampire-257). Fix applied as `81b15066286c8f3818a309ac00d86bdf0b628367`
(import of source PR293 `0119d5daee9b8a07a8096331eea78dbbb6c26050`, patch
`patches/0016-vampire-strike-distance.patch`).

## Defect

The closest-target scan in `borg_attack_aux_vampire_strike()` left its
distance variable holding the last entry examined, so the "require
relatively close" guard tested a distant bystander instead of the chosen
target: facing a kobold at five grids with a hill giant at twenty — the
closeness limit itself — the full production attack chooser queued
nothing. The one-token correction (`dist -> best_dist`) casts (real
`'m'`+book+spell keys).

## Pins and layered provenance

Before build: fifteen-carry `d2893cfde77317486c0b9f378a96b440a463d3ac`
(retained tree `../munchkin-stair`, normal and ASan cores, gate at
`2cf521b8b`, source-identical to `d2893cfd` over src/lib).
`borg-fight-attack.c` already carries 0011/0013/0014; the verifier proves
the before module + the source one-token diff equals the product module
(the guard line occurs exactly once), and the patch round trip from the
fifteen-carry module reproduces the product module SHA256
`29198964c0c1367486df2df1cff66da1084cabbf5fbd28d0e9d92c80d75171fa`.

## Verified results (`artifacts/v257/scenarios`, 2026-11-06)

The fixture drives the **full production attack chooser** in acting mode.

- normal-before `near-far`: 0/1, `acted=0 keys=0 cast_spell=0` — the
  bystander veto.
- normal-after `near-far`: 1/1, `acted=1 keys=3 cast_spell=1`.
- `near-near` control (both monsters close): `acted=1 keys=3 cast_spell=1`,
  1/1 on every phase, byte-identical before/after.
- `far` control (only the distant monster): `acted=0 keys=0`, 1/1 on every
  phase, byte-identical before/after.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/v257-carry/regression.py`): all **76 prior native
fixture cases whole-log identical** to their baselines (60 nine-carry +
3 glyph + 3 missile + 3 tp-other + 3 dispel + 2 arc + 2 munchkin; the
munchkin group uses the evidence-prepared profile verbatim). Tooling 29/29.

## Scope and limits

Bounded native scenario on the production chooser with the class, spell
book and monster placement the report describes. Danger figures are the
Borg's heuristic; no combat-outcome, frequency or survival claim.

## Exact next work

1. PR from `task/carry-vampire-strike`, agent evidence review plus
   required hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 66 carried/open+patch-active; receipts on issue 66, source 257
   and checkpoint 1; evidence archive under `artifacts/v257-carry/`.
4. Then recheck source207 for newly demonstrated fixes (ranked leads 243/
   245 stair family, 249 pending owner intent) before low-priority 81 or
   discretionary death work — all seven of today's demonstrated pairs are
   carried once this merges.

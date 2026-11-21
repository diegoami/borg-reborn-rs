# Arc angle carry: verified product scenario, ready for adoption

Consumer [60](https://github.com/diegoami/better-borg-c/issues/60), canonical
source [253](https://github.com/diegoami/neo-angband-tracker/issues/253)
(arc-253). Fix applied as `92003d66c2ad5e1f5ab95fe1851d77824b8369e8` (import
of source PR297 `7f33e9fbf83059a0a1ed5e530f463124ce1e4be0`, patch
`patches/0014-arc-grid-angle.patch`).

## Defect

The angle test in `borg_launch_arc_at_location()` reoriented the spell
target (`y - y1`, `x - x1`) instead of the scanned grid (`ry - y1`,
`rx - x1`), so every grid received the same angle difference: a 60-degree
Acid Spray was valued as hitting everything in range and line of sight —
17 with an off-arc rear kobold counted, against a reference of 6 with it
hidden from the Borg's knowledge. The two-token correction returns the
value to 6; the in-arc control (two northern monsters, 44) is preserved.

## Pins and layered provenance

Before build: thirteen-carry `9e31b9aa0d3f0ae37aab96377a4416b78c2187ef`
(retained tree `../dispel-filter`, normal and ASan cores, gate at
`23d8ed57`, source-identical to `9e31b9aa0` over src/lib).
`borg-fight-attack.c` already carries 0011 (missile) and 0013 (dispel);
the verifier proves the before module + the source two-token diff equals
the product module (the misoriented lines occur exactly once), and the
patch round trip from the thirteen-carry module reproduces the product
module SHA256
`b60e088a3b037a0e7c6c0709521de6acec6012ce71ef13b5e7916df38d006c23`.
The fixture's contract is relational (the reference is measured with the
bystander's kill hidden, then restored), so no adaptation was needed: the
unchanged evidence pin ran green on this layered product.

## Verified results (`artifacts/arc/scenarios`, 2026-11-06)

- normal-before `arc-bystander`: 0/1, `value=17 > reference=6` — the
  off-arc rear kobold inflates the arc's coverage.
- normal-after `arc-bystander`: 1/1, `value=6 == reference=6`.
- `arc-both` control (in-arc second monster): `value=44 > reference=6`,
  1/1 on every phase, byte-identical logs before/after — legitimate
  coverage preserved.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/arc-carry/regression.py`): all **72 prior native
fixture cases whole-log identical** to their baselines (60 nine-carry +
3 glyph + 3 missile + 3 tp-other + 3 dispel). Tooling 29/29.

## Scope and limits

Bounded native rating scenario on the production arc evaluation with the
temp list filled the way the attack chooser fills it; the reference hides
the bystander's kill record for the measurement and restores it. Danger
figures are the Borg's heuristic; no combat-outcome, frequency or
survival claim.

## Exact next work

1. PR from `task/carry-arc-angle`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 60 carried/open+patch-active; receipts on issue 60, source 253
   and checkpoint 1; evidence archive under `artifacts/arc-carry/`.
4. Then recheck source207 for newly demonstrated fixes (the ranked leads
   named 252 ball/arc edge scans and 249 pending owner intent) before
   low-priority 81 or discretionary death work.

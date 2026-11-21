# Maim Foe real-pass return carry: verified product scenario, ready for adoption

Consumer [214](https://github.com/diegoami/better-borg-c/issues/214), canonical
source [256](https://github.com/diegoami/neo-angband-tracker/issues/256)
(maimfoe-256). Fix applied as
`c61e90c9cdfb466336d3f564a1a3709f3daecc65` (import of source PR389-head
`91822d82ab394afc70d2640d1c2cc735af328056`, patch
`patches/0047-maimfoe-real-pass-score.patch`).

## Defect

`borg_attack_aux_maim_foe()`'s real (execution) pass — the chooser's re-run
with `borg_simulate` false — saved the best target (`b_i`/`b_d`), aimed at
it, and then returned `d`, the loop variable holding the score of the last
monster examined. With two adjacent monsters where the dangerous one
(master lich, score 234 with the danger bonus) is examined first and a
kobold (score 30) last, the real pass reported 30 — the kobold's score —
for a cast aimed at the lich. The correction returns `b_d`, the best
tracked score; the trailing `if (borg_simulate) return d;` behind it was
dead code (the simulation path already returned `b_d`). The chooser
currently discards this real-path value, so the measured wrongness is the
reported score itself — value misreport; no death or survival claim.

## Pins and layered provenance

Before build: forty-six-carry `7be6518287d8bb20b08c3e59f58c8f3707e36ae2`
(retained tree `../detectcarry`, normal and ASan cores, gate recorded at
that commit, source-identical to current origin/main `a8322f483` over
src/lib/CMakeLists.txt/cmake — the interleaved main advance was
death-lane tooling/docs only). `borg-fight-attack.c` already carries
earlier corrections, so the verifier proves that the source diff applied
to the before module equals the product module; the applied commit's
diff body is byte-identical to the source commit's diff. The source
evidence fixture at `01ddb076c` is used unchanged; the receipt chain is
checked in the verifier against the run pin `ea1443bd`
(receipt.json sha256 map: scenario.c `550d8618…`, borg.txt `80738da2…`
— both match; the evidence head adds only the receipt/replay artifacts
on top of the pin).

## Verified results (`artifacts/mf256/scenarios`, 2026-11-08)

- `wrong-return` (production `borg_calculate_attack_effectiveness(
  BF_SPELL_MAIM_FOE)` with `borg_simulate` false and a filled two-entry
  target list): normal-before 0/1, `value=30` — the last-scanned
  kobold's score for a cast aimed at the lich; normal-after 1/1,
  `value=234` — the best tracked score.
- `single` (one monster: last and best are the same): 234 both, 1/1,
  byte-identical before/after.
- `none` (nothing adjacent): 0 both, 1/1, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- **Cross-repo byte match:** all four normal-phase log SHA256s are
  byte-identical to the source evidence kit's recorded logs
  (`before-wrong-return.log` `9aa9be4f…`, `after-wrong-return.log`
  `bb89a6ac…`, `before/after-single.log` `26b4487c…`,
  `before/after-none.log` `f1318ad3…`) — the consumer reproduction of the
  unchanged fixture matches the source run byte for byte.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5
frontend** numerically read; tooling **95/95** (main's tooling set grew
from 85 to 95 with the death-lane takeover additions merged since the
last carry). Regression matrix (`artifacts/mf256-carry/regression.py`,
local-only): all **148 prior native fixture cases** match the recorded
forty-six-carry tally — 145 whole-log identical (including the
gloves-205 and detectcarry-219 cases), the dispel-248 undead case
passing with its asserted losdispel log change, and the two dispel-259
cases asserted-red at the pinned post-losdispel values.

## Scope and limits

Bounded chooser real-pass scenario with one two-monster case and two
controls. It proves the return-value correction, not a gameplay
frequency or survival claim; the chooser discards this value today, so
no behavioral change is expected on the measurement cohort.

## Exact next work

1. PR from `task/carry-maimfoe`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-six-carry lineage run — expected byte-identical (the chooser
   discards the corrected value).
3. Mark 214 carried/open+patch-active; receipts on issue 214, source
   256 and checkpoint 1; evidence archive under `artifacts/mf256-carry/`.
4. Then recalldepth-310 (patch 0048, source PR #373, pins staged); the
   newly demonstrated dontreact-14 (source PR #405 green, 26 SUCCESS)
   queues after it.

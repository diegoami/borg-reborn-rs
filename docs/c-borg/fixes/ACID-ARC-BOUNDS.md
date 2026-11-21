# Acid-arc-bounds carry: verified product scenario, ready for adoption

Consumer [141](https://github.com/diegoami/better-borg-c/issues/141), canonical
source [252](https://github.com/diegoami/neo-angband-tracker/issues/252)
(acidarc-252). Fix applied as
`894965c78bce5c53a3bbdc498ace40a825fb0679` (import of source PR382
`2a4125e44f8bcb30e7170416c68f8713cc271994`, patch
`patches/0037-acid-arc-bounds.patch`).

## Defect

The blast loops in `borg_launch_bolt_at_location()` and
`borg_launch_arc_at_location()` ran `y2 - rad <= ry < y2 + rad` (likewise
rx, and y1/max for arcs), so grids exactly rad grids south or east of the
center never entered the blast scan. Any monster standing on that excluded
edge contributed nothing: Acid Spray aimed at a giant exactly ten grids
south rated zero and the Borg cast nothing, while the same giant one grid
closer was scored and sprayed. The correction makes the four loop bounds
inclusive. No memory error; no death or survival claim.

## Pins and layered provenance

Before build: thirty-six-carry `faa86f83b12f7f43cdde21f48faf045f9e385c60`
(retained tree `../stairtrack`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `fa81bacb8` over
src/lib/CMakeLists.txt/cmake). `borg-fight-attack.c` already carries
0011/0013/0014/0016/0019/0020/0021/0022/0023/0024, so the verifier proves
that the source diff applied to the before module equals the product
module; the applied commit's diff body is byte-identical to the source
commit's diff (hunk offsets differ only by the layered line shift). The
source evidence fixture at `e40c642a` is used unchanged.

## Verified results (`artifacts/a252/scenarios`, 2026-11-07)

- normal-before `edge` (giant exactly max_range south, on the excluded
  edge): 0/1, `acted=0 keys=0 cast=0` — the blast never counts it.
- normal-after `edge`: 1/1, `acted=1 keys=17 cast=1`, sequence
  `* p 2 2 2 2 2 2 2 2 2 2 5 m b f 5` — the recorded Acid Spray cast.
- `near` (eight grids): `acted=1 keys=15 cast=1`, 1/1 on every phase,
  byte-identical before/after.
- `empty`: refused on every phase, 1/1, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Retained verifier attempts (diagnosed and corrected, never by weakening a
product assertion): `scenarios-attempt1-verifier-seq-constant` (the
verifier compared the stripped sequence against a constant that still
carried the `DECISION ...` prefix) and `scenarios-attempt2-verifier-near-constant`
(the near control's key count was written as the edge's 17 instead of
15 for the eight-grid giant). Both runs' product logs are identical to
the final attempt's.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **41/41**. Regression matrix
(`artifacts/a252-carry/regression.py`, local-only): all **119 prior native
fixture cases** match the recorded thirty-six-carry tally — 116 whole-log
identical (including the brand-240 and stairtrack-232 cases), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production attack-chooser scenario for one blast-edge
geometry and two controls. It proves the blast-scan bounds and their
correction, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-acidarc`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare outcomes and trace
   hashes with the recorded thirty-four-carry run (unchanged through the
   thirty-fifth and thirty-sixth carries).
3. Mark 141 carried/open+patch-active; receipts on issue 141, source 252
   and checkpoint 1; evidence archive under `artifacts/a252-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`).

# Out-of-light-gate carry: verified product scenario, ready for adoption

Consumer [144](https://github.com/diegoami/better-borg-c/issues/144), canonical
source [229](https://github.com/diegoami/neo-angband-tracker/issues/229)
(outoflight-229). Fix applied as
`21fbfba5c4d3a2cc77707fea885590479cec18c3` (import of source PR384
`54a26b3b3fe4e12ed4d8fc3380219727b48eda2b`, patch
`patches/0038-dying-light-gate.patch`).

## Defect

`borg_think_dungeon_light()`'s entry test was
`!BI_LIGHT && (BI_LIGHT <= 0 || timeout <= 3)` — once the first clause
holds, the parenthesised clause is always true, so the about-to-burn-out
case (timeout <= 3) never fired on its own: a lit lantern three turns from
dying was left to go dark even with a spare torch or oil in the pack. The
correction makes the test `(!BI_LIGHT || (timeout <= 3 && !NO_FUEL))`: the
dying-but-lit light starts the out-of-light maneuvers, while fuel-less
(everburning) lights — whose timeout is permanently zero — keep their
NO_FUEL guard so the corrected clause cannot false-trigger every turn. No
memory error; no death or survival claim.

## Pins and layered provenance

Before build: thirty-seven-carry `894965c78bce5c53a3bbdc498ace40a825fb0679`
(retained tree `../acidarc`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `75ba9614b` over
src/lib/CMakeLists.txt/cmake). `borg-think-dungeon-util.c` already carries
0032 and 0036, so the verifier proves that the source diff applied to the
before module equals the product module; the applied commit's diff body is
byte-identical to the source commit's diff (hunk offsets differ only by the
layered line shift). The source evidence fixture at `e1638569` is used
unchanged.

## Verified results (`artifacts/o229/scenarios`, 2026-11-07)

- normal-before `dying` (lantern lit, timeout 3, oil + spare torch
  carried): 0/1, `acted=0 keys=0` — the borg lets the light die.
- normal-after `dying`: 1/1, `acted=1 keys=2`, sequence `w e` — the
  borg_wear_stuff swap to the fresh torch.
- `lit` (timeout 1500) and `everburn` (fuel-less, timeout 0): skip on
  every phase, 1/1, byte-identical before/after.
- `dead` (burned out): the same `w e` swap on every phase, 1/1,
  byte-identical before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **41/41**. Regression matrix
(`artifacts/o229-carry/regression.py`, local-only): all **122 prior native
fixture cases** match the recorded thirty-seven-carry tally — 119 whole-log
identical (including the brand-240, stairtrack-232 and acidarc-252 cases),
the dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production light-routine scenario with one dying light and
three controls. It proves the entry gate and its correction, not a gameplay
frequency or survival claim.

## Exact next work

1. PR from `task/carry-outoflight`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare outcomes and trace
   hashes with the recorded thirty-four-carry run (unchanged through the
   thirty-fifth through thirty-seventh carries).
3. Mark 144 carried/open+patch-active; receipts on issue 144, source 229
   and checkpoint 1; evidence archive under `artifacts/o229-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`).

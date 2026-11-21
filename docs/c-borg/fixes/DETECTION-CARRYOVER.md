# Detection-carryover carry: verified product scenario, ready for adoption

Consumer [205](https://github.com/diegoami/better-borg-c/issues/205), canonical
source [219](https://github.com/diegoami/neo-angband-tracker/issues/219)
(detectcarry-219). Fix applied as
`7be6518287d8bb20b08c3e59f58c8f3707e36ae2` (import of source PR391-head
`3d104a8b82aafeb421869c62736c8046b6272d82`, patch
`patches/0046-detection-carryover.patch`).

## Defect

`borg_update()`'s depth-change branch clears the per-panel detection flags
— trap, door, evil — but not `borg_detect_obj`, so object-detection
knowledge carried over to the new level: the new level's panels still
counted as object-detected from the previous level's detection. The
correction adds `borg_detect_obj[y][x] = false;` to the clear. Stale-state
defect; no death or survival claim.

## Pins and layered provenance

Before build: forty-five-carry `175ef6feb555bbf67bcdfff7f628d686decf63a8`
(retained tree `../gloves`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `4a3b69b52` over
src/lib/CMakeLists.txt/cmake). `borg-update.c` already carries
0018/0039, so the verifier proves that the source diff applied to the
before module equals the product module; the applied commit's diff body is
byte-identical to the source commit's diff. The source evidence fixture at
`52cc5bc4` (evidence PR392 head; fixtures byte-identical to the pinned
`55e00023`, receipt.json records the chain) is used unchanged.

## Verified results (`artifacts/dc219/scenarios`, 2026-11-07)

- normal-before `new-level` (production `borg_update()` across a depth
  change 30→31 with a planted panel flag): 0/1,
  `obj_cleared=0 wall_cleared=1` — the object-detection flag survives
  while the wall flag clears.
- normal-after `new-level`: 1/1, `obj_cleared=1 wall_cleared=1` — both
  clear.
- `same-level` (no depth change): flags untouched on every phase, 1/1,
  byte-identical before/after.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Retained attempts: `scenarios-attempt5-wrong-evidence-ref` and
`scenarios-attempt6-fabricated-fix-sha` — verifier constant mistakes (the
calllight evidence head pasted instead of this carry's; a hand-expanded
abbreviated fix SHA), corrected without touching any product assertion.
The final attempt's normal-phase logs are identical across attempts.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **85/85** (the current main tooling set).
Regression matrix (`artifacts/dc219-carry/regression.py`, local-only): all
**146 prior native fixture cases** match the recorded forty-five-carry
tally — 143 whole-log identical (including the gloves-205 cases), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production update-pass scenario with one depth-change case
and one control. It proves the flag clearing and its correction, not a
gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-detectcarry`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-five-carry lineage run — expected behavior-neutral unless a v1
   Warrior carries object detection across a level change during the
   cohort.
3. Mark 205 carried/open+patch-active; receipts on issue 205, source 219
   and checkpoint 1; evidence archive under `artifacts/dc219-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`); source #399 remains a separately owned
   source finding.

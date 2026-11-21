# Gloves-armour-swap carry: verified product scenario, ready for adoption

Consumer [201](https://github.com/diegoami/better-borg-c/issues/201), canonical
source [205](https://github.com/diegoami/neo-angband-tracker/issues/205)
(gloves-205). Fix applied as
`175ef6feb555bbf67bcdfff7f628d686decf63a8` (import of source PR377-head
`c52527eed3cf272cb62a16a773f25ed09f6cac36`, patch
`patches/0045-gloves-armour-swap.patch`).

## Defect

`borg_notice_armour_swap()`'s armour-swap case list accepted HELM, CROWN,
SHIELD, CLOAK, SOFT_ARMOR, HARD_ARMOR and DRAG_ARMOR but never TV_GLOVES,
so free-action gauntlets in the pack could never be chosen as the armour
swap when the Borg has no free action. The correction adds
`case TV_GLOVES:` to the list. Eligibility defect; no death or survival
claim.

## Pins and layered provenance

Before build: forty-four-carry `a22d8d09e4a04f17de1680a6047dac759e46e54f`
(retained tree `../calllight`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `4a3b69b52` over
src/lib/CMakeLists.txt/cmake). `borg-trait-swap.c` is pristine (no carried
patches touch it), so the verifier proves that the source diff applied to
the pristine before module equals the product module; the applied commit's
diff body is byte-identical to the source commit's diff. The source
evidence fixture at `7ff787ac` (evidence PR378 head; fixtures byte-identical
to the pinned `9a6927a4`, receipt.json records the chain) is used unchanged.

## Verified results (`artifacts/gl205/scenarios`, 2026-11-07)

- normal-before `glove-fa` (free-action gauntlets in the pack, no free
  action worn, production `borg_notice_armour_swap()`): 0/1,
  `armour_swap=0 free_act=0` — the gauntlets are never chosen.
- normal-after `glove-fa`: 1/1, `armour_swap=1 free_act=1` — chosen as the
  armour swap.
- `plain` (plain glove, not worth a swap) and `have-fa` (free action
  already worn): skip on every phase, 1/1, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Retained attempts: `scenarios-attempt1-mkdir` (the verifier tried to
re-create its own TemporaryDirectory), `scenarios-attempt2-fabricated-sha`
(a hand-expanded abbreviated SHA instead of the real before commit) —
invocation/verifier mistakes, corrected without touching any product
assertion. The final attempt's normal-phase logs are identical to attempt
1's.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **85/85** (the current main tooling set).
Regression matrix (`artifacts/gl205-carry/regression.py`, local-only): all
**143 prior native fixture cases** match the recorded forty-four-carry
tally — 140 whole-log identical (including the calllight-231 cases), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production armour-swap scenario with one omission case and
two controls. It proves the gloves eligibility and its correction, not a
gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-gloves`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-four-carry lineage run — expected behavior-neutral unless a v1
   Warrior packs free-action gauntlets without wearing free action during
   the cohort.
3. Mark 201 carried/open+patch-active; receipts on issue 201, source 205
   and checkpoint 1; evidence archive under `artifacts/gl205-carry/`.
4. Recheck source207 for newer demonstrations; tapunlife-249 remains
   blocked (see `TAP-UNLIFE.md`); source #399 remains a separately owned
   source finding.

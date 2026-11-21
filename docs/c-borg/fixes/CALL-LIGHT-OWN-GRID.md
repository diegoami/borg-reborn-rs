# Call-light-own-grid carry: verified product scenario, ready for adoption

Consumer [196](https://github.com/diegoami/better-borg-c/issues/196), canonical
source [231](https://github.com/diegoami/neo-angband-tracker/issues/231)
(calllight-231). Fix applied as
`6b43497330399bfc8711289a3e873868a8af910f` (import of source PR395-head
`cda4fb662cabee8a49fbdbcaf4383dc7be1fff1c`, patch
`patches/0044-call-light-own-grid.patch`).

## Defect

The out-of-light routine's call-light scan loops over the eight neighbours
of the Borg, but the open-doorway special case inside tests the Borg's OWN
grid (`y == borg.c.y && x == borg.c.x`) — a case that could never fire —
so the is-the-door-lit cheat (`square_isglow`) never ran and an open
doorway underfoot kept whatever glow mark the Borg's map happened to hold.
The correction widens the loop to `ii <= 8` (index 8 of ddx_ddd/ddy_ddd is
the own grid), which both repairs the doorway mark from the game and lets
the routine light a dark own grid like any other, matching the section's
stated purpose. No memory error; no death or survival claim.

## Pins and layered provenance

Before build: forty-three-carry `bdd3c2ea2c120c5200f182b1676b860942abc71e`
(retained tree `../excavate`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `4a3b69b52` over
src/lib/CMakeLists.txt/cmake — the concurrent documentation merges changed
nothing under the build inputs). `borg-think-dungeon-util.c` already
carries 0032/0036/0038/0041/0043, so the verifier proves that the source
diff applied to the before module equals the product module; the applied
commit's +/- lines are byte-identical to the source commit's diff. The
source evidence fixture at `85de924fa` (evidence PR396 head; fixtures
byte-identical to the pinned `f7058cd5`, receipt.json records the chain)
is used unchanged.

## Verified results (`artifacts/cl231/scenarios`, 2026-11-07)

- normal-before `own-open-door` (Borg in an open doorway the game reports
  lit, own grid planted without the glow mark, production
  `borg_think_dungeon_light()`): 0/1, `ownglow=0` — the doorway mark is
  never repaired.
- normal-after `own-open-door`: 1/1, `ownglow=1` — the scan visits the own
  grid and repairs the mark from the game.
- `floor` (plain lit floor): `ownglow=1`, 1/1 on every phase,
  byte-identical before/after.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **81/81** (the current main tooling set).
Regression matrix (`artifacts/cl231-carry/regression.py`, local-only): all
**141 prior native fixture cases** match the recorded forty-three-carry
tally — 138 whole-log identical (including the excavate-236 cases), the
dispel-248 undead case passing with its asserted losdispel log change,
and the two dispel-259 cases asserted-red at the pinned post-losdispel
values.

## Scope and limits

Bounded native production light-routine scenario with one doorway
situation and one control. It proves the scan's own-grid visit and its
mark repair, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-calllight`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-three-carry (excavate-lineage) run — expected behavior-neutral
   unless a v1 Warrior stands in an open doorway with a stale glow mark
   during the cohort.
3. Mark 196 carried/open+patch-active; receipts on issue 196, source 231
   and checkpoint 1; evidence archive under `artifacts/cl231-carry/`.
4. Recheck source207 for newer demonstrations (the source lane's
   calllight receipt is pending); tapunlife-249 remains blocked (see
   `TAP-UNLIFE.md`); source #399 remains a separately owned source
   finding.

# Drophole-136 carry: verified product scenario

Consumer [290](https://github.com/diegoami/better-borg-c/issues/290),
canonical source
[136](https://github.com/diegoami/neo-angband-tracker/issues/136)
(drophole-136; source PR450 `ca904787`, evidence PR449). Fix applied as the
second step of the swap-drop PR (`cherry-pick -x` of source
`ca904787777ca8fed7e113f52f1a6ae4fa02daeb`, patch
`patches/0074-drop-hold-equal-comparisons.patch`), stacked on
trollswap-201 (patch 0073).

## Defect

The high-level "don't crush cool stuff" valuation of `borg_drop_hole()` —
used when a desperate drop must free a slot — used `!=` where the original
borg used `==` in the rod, staff, wand, scroll and potion tests, so those
tests protected the wrong items. A level-40 Warrior with a full pack
dropped a spare sellable amulet (Slow Digestion, or a 25,000-gold ESP)
while keeping a worthless Potion of Salt Water. The correction compares
with `==` and keeps the staff spare-count conditions
(`BI_GOOD_S_CHG`/`BI_ASTFDEST` < 2, computed with the item removed).
Which item gets thrown away changes; no combat or survival effect.

## Pins and provenance

Before: the trollswap-201 step tip `288556d2b` (snapshotted build
`build/native-before`, gate result `artifacts/native-before/result.json`,
965/965 + 5/5 at that commit). The verifier proves that the source diff
applied to the before `borg-junk.c` equals the product module (the file is
otherwise pristine) and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `29ebf656` at head
`ac57d516`, asserted byte-identical to the run pin. The kit's fixture
interventions are documented there (experience granted, carried kit
removed, pack built with known artifact weapons, runes learned, arena map
copied).

## Verified results (`artifacts/dh136/scenarios`, 2026-11-08)

- `salt-water` / `spare-esp` — fail before, pass after: before 0/1
  (`dropped_amulet=1 dropped_potion=0`; the spare amulet goes, the Salt
  Water stays), after 1/1 (`dropped_amulet=0 dropped_potion=1`; the Salt
  Water goes, the amulet stays). One floor item in both.
- `restore-mana` (protected potion) and `level30` (valuation block
  skipped) controls: 1/1 in both phases, logs byte-identical (`7d6c24e1…`,
  `6538452 0…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes.

Native gate at the applied tip: **965/965 unit + 5/5 frontend** (before
snapshot gate and product gate); tooling **129/129**. A harness correction
is recorded in the verifier commit: the first expected-outcome dicts
omitted the fixture's `floor_items` field. Prior-fixture matrix not rerun
(uncommitted runner; known gap recorded on checkpoint 1 and the milestone
6 review).

## Scope and limits

Bounded full-pack drop scenario. It proves which item the desperate drop
chooses; potions were measured, and after the fix the unlisted rod, wand,
staff and scroll kinds become cheaper to drop (the kit's honest scope note
is retained).

## Exact next work

1. Stacked in the same PR as trollswap-201; per-step gates; agent evidence
   review plus required hosted checks, then normal merge.
2. Frozen v1 cohort at the merged pair tip against the 0071-after cohort at
   `6002bfa4b`.
3. Receipts on issues 289/290, sources 201/136 and checkpoint 1.

# Cap unknown-rune spares and give them to a shop

Issue #1308, stacked on #1307 (PR #1311). Second of four fixes from the
equipment routine audit of `8d85f44f`.

## Found

An apparently plain item marked `{??}` always has a property the player has
not learned. `object_runes_known` compares the item's actual runes with the
player's rune knowledge (`obj-knowledge.c:678-746`). A plain item is rune-known
as soon as the player knows its combat runes. The #1304 native save confirms
hidden egos on all four "plain" melee spares.

Under the default `birth_no_selling` every shop quote is 0
(`store.c:636-638`). With `junk_gear_alternatives` above 0, `spare_sale`
therefore never sold, and every ranked town disposal dropped the item on
the floor. A shop still accepts a `{??}` item (`store.c:532-534`), and the
sale teaches every unknown rune (`store.c:1948-1952`). Unknown-rune gear
outside the plain melee and armour `{??}` pattern had no cleanup route.

## Done

- `unknown_rune_spare` and `unknown_rune_excess` (`borg_spare_gear.rs`):
  - The pack carries at most one unknown-rune spare per equipment slot, two
    for rings, chosen by visible value, then weight, then pack position.
    Diggers form their own group. A stack keeps one copy.
  - Artifacts, known activations, known curses, swaps, staged loadouts and
    inscriptions other than the game's tags (`??`, `ignore`, `tried`) are
    excluded.
- Where the excess is released:
  - the dungeon drop (`borg_drop_inferior_spare` through
    `ranked_pack_spare`);
  - town arrival unloading;
  - `organize_gear`;
  - pack-full cleanup, as whole stacks only.
- `spare_sale` gives unknown-rune gear to a shop that accepts it even for no
  gold. Fully known spares keep the earlier rule.
- The town planner gives pack unknown-rune equipment the `Renewable` Home
  role, so it is never stored at Home. Its unknown runes still protect it
  from the planner's own sale or drop.

## Review fixes (owner, PR #1312)

- **Identification first.** While Identify Rune is usable, the cap
  releases nothing. `borg_drop_inferior_spare` runs before `borg_test_stuff`
  in the dungeon think, so the cap could discard a spare before the
  identification attempt. Once identification is used up, the cap applies.
- **No floor drop in town.** For unknown-rune gear, town disposal offers the
  largest quantity a shop accepts. With no taker it waits instead of falling
  back to `DROP_SURPLUS`. This covers arrival unloading and `organize_gear`,
  including the `{??}` releases of the ranked plan.

## Checked

- Native `ranked-cleanup` cases cover:
  - no release while identification is usable;
  - the lower-valued of two unknown daggers;
  - an unknown armour stack trimmed to one copy, and not released by
    pack-full cleanup;
  - an unknown launcher kept beside an unknown weapon;
  - a deliberate inscription;
  - three unknown rings;
  - the active swap.
- Release unit test `unknown_rune_tags_and_shop_learning`.
- The shipped smoke suite is identical to `smoke-borg-rs`.

## Not checked

- The town gift needs a native object with unknown runes in a real shop. The
  fixtures replace the Borg's view but not the game's objects, so no native
  test covers it. Only the decision rule is unit-tested.
- The Home-role change has no native test.
- Identification priority is unchanged. `borg_test_stuff` already gives every
  pack item with unknown runes its value plus `wear_identify_magic_bonus`
  (20,000).

## Left

#1309 and #1310.

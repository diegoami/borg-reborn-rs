# Supplies never replace protected Home gear

Issue #1317. The issue suspected that a supply coming into a full Home
could push out a resist swap. A swap usually adds no power when it is
not worn, and `home_replacement` promotes Home gear with power ≤ 0
while supplies are below the third target (`borg_town_utility.rs`,
`promotes_supply`).

## Found

At the shipped settings this cannot happen, and nothing in play
changes.

- With `junk_exploration_first` 1 and `junk_gear_alternatives` > 0
  (both shipped), every known Home item in a ranked slot is a
  `ranked_item`. Ranked slots are equipment, rings, amulets, diggers and
  permanent lights. `kit_of` sets `protect_disposal` for it
  (`borg_town_shadow.rs`, the `KitItem` literal). `home_replacement`
  skips protected stacks before it looks at power.
- The loadout's own eviction (`borg_town_loadout.rs`, `eviction` and
  `expendable_rank`) returns no rank for any gear role. It evicts only
  rejected, renewable or common stock, light-utility consumables and gear
  identical to the worn piece.
- Redundant Home gear still leaves, through the ranked plan's own
  release (`redundant_home_gear`), not through supply replacement.
- The power of a Home gear stack is its gain when worn (`equipped_gain`),
  not when carried. A resist ring with free ring slots therefore has
  positive power. A swap's power is ≤ 0 only when it would displace
  better worn gear.

The eviction path the issue describes is still reached with
`junk_gear_alternatives` 0 or `junk_exploration_first` 0, the earlier
cleanup kept for comparison. That is unchanged: it is the historical
control.

## Done

- Native case `reserve-swap` (`engine/tests/test_town_loadout.py`).
  The `reserve-overlap` full Home also holds a known Ring of Resist
  Poison, which the worn kit lacks, and the case runs at the shipped
  ranked policy. It checks:
  - the ring classifies as gear;
  - the live planner's kit (`kit_for_test`, test feature only) marks it
    gear and `protect_disposal`;
  - the loadout's eviction still chooses the excluded Second Sight.
- Unit test `a_priority_potion_replaces_expendable_stock_not_a_zero_power_resist_swap`
  (review of #1322). A full Home below the supply third holds a power-0
  resist swap, protected as `kit_of` marks it, a sellable `Common` stack
  and protected gear filler. An incoming late potion:
  - picks the `Common` stack in `home_replacement`;
  - in the full `dispositions`, sells the `Common` stack, stores the
    potion and keeps the swap and the filler.

  With no expendable stock left, the swap is promoted only when it is
  unprotected.
- `town_keep_last_class_book`'s citation moves seven lines
  (`params.rs`, `docs/CALIBRATION.md`).

No shipped play changes. The new code is behind the `test-town-loadout`
feature or in tests.

## Left

- Nothing for this issue. #1315 covers swaps leaving Home again, #1316
  the number of backups, and #1318 the spare dungeon books (stacked on
  this branch).

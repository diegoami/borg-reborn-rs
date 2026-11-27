# A spare dungeon book is a scarce Home reserve

Issue #1318, stacked on #1317 (both touch the native town fixture). The
manual says to keep a spare of each spellbook you rely on (ch. 11 §7,
§12.12 item 4). Elements do not destroy dungeon books: `init.c:268-273`
gives them `EL_INFO_IGNORE` for the base elements (review of #1323). A
dungeon book cannot be bought back, and scarcity is the reason for the
reserve.

## Found

- `borg_town_home::consumable` made town books `Renewable`, which is
  correct. The Bookseller's `always:<book tval>` lines add every book
  whose `class_book` has no `dungeon` flag (`store.c:208-230`), so
  town books read as `NormalStock::Staple`.
- Dungeon books read as `Unlisted`. Their tval is not in the device and
  consumable list, so they fell to `Role::Ordinary`, rank 0: the first
  stack `home_replacement` would choose. `keep_last_book` protects only
  one copy across the pack and Home.

## Done

- A known, uncursed dungeon book the class can browse
  (`obj_kind_can_browse`) is now `Scarce`. It counts toward the Home
  supply third, `home_replacement` does not evict it for `Common` or
  `Renewable` stock, and its first spare can be stashed (`stash_for_later`
  requires a reserve role). A dungeon book of another class stays
  `Ordinary`. Town books are unchanged.
- One spare is the reserve. In `kit_of`, another copy when Home already
  holds that book becomes `Renewable`, mirroring how a `LatePotion`
  duplicate becomes `Scarce`. A single carried copy is the one the Borg
  reads and keeps its earlier role (`Ordinary`), so it is never stashed as
  a spare.
- Review of #1323: the one-copy rule also holds inside a stack. The
  reserved stack, the one `keep_last_book` marks, is the only stack that is
  both `Scarce` and `keep_last_book`. From it the planner deposits one copy
  at a time, and `prune_low_priority_home` releases a Home stack's copies
  above one. Unit test `one_dungeon_book_copy_is_the_home_reserve`: from a
  carried stack of three, one copy is stored; a Home stack of three drops
  two; a single Home copy is kept.
- Native case `reserve-book` (Priest fixture). Home holds
  [Healing and Sanctuary] and the Mage's [Arcane Control], and the pack
  carries a second [Healing and Sanctuary]. The case checks:
  - the Priest book classifies `Scarce`, the Mage book does not, and the
    carried town book stays `Renewable`;
  - in the live planner's kit, the Home copy is `Scarce` and the single
    carried copy keeps `Ordinary`.
- Unit test `a_spare_dungeon_book_survives_an_incoming_common_item`.
- `town_keep_last_class_book`'s citation moves with the new code.

## Checks

- `engine/tests/test_town_loadout.py`: 10 of 10 on a
  `test-town-loadout` build.
- `cargo test --release -p borg-core` (1014 passed), the workspace tests,
  clippy, `cargo fmt --check -p borg-reborn-rs`,
  `scripts/check-param-citations.py`.
- Shipped P-Borg smoke: 4/4 runs identical to `smoke-borg-rs`, so no
  re-record.

## Left

- Not measured: the change only matters once a caster finds a dungeon
  book. They are generated from DL30 (the first of each class), DL40 and
  DL60 (`class.txt`, `book-properties`). The 3M-turn games of borg-zoo#805
  kept casters at DL7–15, so no zoo run has reached them yet.

## 2026-11-23: the unknown-potion test needs 60 HP, unless a known poison cure is carried (#1063)

From `track-p` (not stacked on #1071). #1060's PR #1071 adds the same
`item_test_unknown_min_hp` (compiled 25) together with its hold after a bad
test; this branch adds the floor on its own, with #1071's parameter name,
default and description, so whichever merges second resolves a small conflict
in `params.rs`, `borg_item_use.rs` (the floor's signature and unit tests),
`harness/borg.txt` and `docs/CALIBRATION.md` by keeping both: #1071's hold, and
this PR's shipped 60 and cure exemption.

**Found** (borg-cemetery study `studies/lvl1-deaths-trackp-206cf4e7`,
diegoami/borg-cemetery#138). Human Ranger 142490685 (CL2, 18/18 HP) drank its
unknown "Cloudy Potions" as #983's test on DL1's up staircase: a Potion of
Poison (`TIMED_INC:POISONED` 10+2d7, object.txt; 1 HP a world tick and no
regeneration, game-world.c:514-516), with no cure (its Neutralize Poison had
gone into an earlier test). It died 180 game turns later. #983's note "none
kills by itself" was wrong at low HP.

**Done.**
- `item_test_unknown_min_hp` (compiled 25, **shipped 60**; owner,
  2026-11-23: "put minimum hp at 60"): the test needs at least this many
  current HP, besides the percent floor.
- `item_test_unknown_min_hp_cure_exempt` (compiled 0, **shipped 1**): a
  known Cure Critical Wounds (potion or device, `BI_ACCW`) or Neutralize
  Poison / poison-curing food (`BI_ACUREPOIS`) waives the HP floor; the percent
  floor (50) stays. All are `CURE:POISONED` in object.txt (:3205, :3287);
  unaware flavours are not counted (`borg_notice_inventory` skips them), and
  `borg_caution`'s poison cure drinks them.
- `unknown_potion_min_hp` (`borg_item_use.rs`), unit-tested with the
  Ranger's numbers. The "none kills by itself" sentence is corrected in
  `harness/borg.txt` and in `item_test_unknown_potions`' description.
  `docs/CALIBRATION.md` has a #1063 section.

**Not done, and why.** #1063's point 2 (a poisoned Borg with no cure that
the poison can kill goes to the stairs, or up to town) cannot arise at the
shipped values: without a cure the test needs 60 HP, and Poison costs at most
24. With a cure, `borg_caution` cures it (with #1071, its hold first runs
`borg_recover`). Point 3 (the "stopping cure", optional) is #1068's (#1028)
arithmetic, in another PR.

**Checked.** fmt, clippy, the workspace and `borg-core` tests,
`scripts/tests`, `check-param-citations.py` (2,433, 0 wrong). Smoke: 2/4
identical to `track-p`'s reference (`60e04a21`); smoke-warrior-1 and
smoke-mage-2 diverge at their first old potion test (game turns 5,703, at
20/20 HP, and 16,798): 6 and 1 tests before, 0 now. Re-recorded; 4/4 identical
to the new reference. The suite needs a borg-cemetery checkout at
`death-tools-pin.txt` (`BORG_CEMETERY_DIR`); the sibling checkout here had
moved past it. `borg-core-vendor.py check` was not run here (no `vendor/borg-rs` in this
container); CI runs it. The Ranger's seed was not replayed: the study's
replays need the zoo's per-death configs, and on the head that start already
plays another game and lives.

**Left.** The population effect: a borg-zoo run (borg-zoo#703's
normal-world potion-test run, at 60 and at 25, with and without the
exemption).

**Owner.** At 60, a character under 60 HP tests no unknown potion unless it
carries a known CCW or Neutralize Poison, so most characters below CL10 learn
their flavours later. 60 is the owner's margin over Poison's 24, a hypothesis
until the zoo measures it (RULES 14). Whether the exemption should also count
a curing spell (Cure Poison, Herbal Curing) is open: today it counts only items.

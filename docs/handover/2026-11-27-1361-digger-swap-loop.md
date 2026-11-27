## 2026-11-27: A digger does not take the combat weapon's place (#1361)

The recorded level 27 Dwarf Priest `Borg261127181053` (Track P build
`f51625f8`) stood on a DL5 staircase and wielded 2,026 times, alternating
'Narthanc' and a Shovel of Earthquakes (1d2) (+18,+11) <+4, +1>. The owner's
diagnosis was a disagreement between routine equipment selection and the
rule that re-equips a combat weapon while a digger is wielded.

## Found

- Two paths choose the weapon slot under the shipped ranked gear plan
  (`town_gear_home_backups: 1`), both inside `borg_wear_stuff`:
  - the ranked plan (`borg_spare_gear::gear_plan`) ranks weapons by
    equipment power and leaves diggers out (`planned_slot`). A digger in
    the weapon slot is only for tunnelling (`inferior`);
  - the routine trial ranks every other pack item, diggers included, by the
    whole kit's power.
- The trial wielded the shovel. The rule that wears equipment while a
  digger is wielded (`borg_think_dungeon.rs:1085`) then ran the plan, which
  put the dagger back. The shovel's +4 STR is the likely reason the trial
  wanted it.
- In the fixture the trial preferred the shovel only when three things held:
  - the +4 STR lifted the Priest's encumbrance slowness;
  - the Priest had fire resistance from elsewhere;
  - the Priest had another ranged attack, since `borg_ranged::wear_coverage`
    vetoes losing Narthanc's Fire Bolt.

  The recorded save's kit is not available to this session (see *Not
  checked*), so this is a constructed equivalent, not the recorded kit.
- A second disagreement with a lone digger: `borg_remove_stuff` took the
  wielded shovel off to bare hands (power with 1,371,215, without
  1,377,180). In the pack the shovel earns the tunnelling swap reward of
  #1320, and the wear trial then put it back on. In the fixture this
  happened at thinks 5, 57, 60 and 239, with the wear cooldown between
  them, until the Borg found a Rapier.
- The Borg never needs to wield a digger to tunnel. The engine wields a
  carried digger for each tunnelling turn and puts the weapon back
  (`cmd-cave.c:534-583`).

## Done

- New flag `wear_digger_keeps_weapon`, read in `borg_item_wear.rs`:
  - Under the ranked plan, the routine wear trial skips a digger while a
    combat weapon is worn (`digger_waits_for_tunnelling`). A digger is still
    worn when the weapon slot is empty or holds a digger.
  - The remove trial keeps a wielded, uncursed digger
    (`keeps_wielded_digger`, called from `borg_junk.rs`). A better weapon
    still replaces it through the wear paths.
  - The default 0 keeps both inherited trials. It ships at 1 in
    `harness/pborg.yaml`.
- `docs/CALIBRATION.md`: a new section and its parameter row. The citation
  of `junk_inferior_spares` moved with its line.
- Native cases `digger-swap-loop` and `digger-only` (feature
  `test-stair-handoff`, `engine.rs:setup_test_digger_kit`,
  `borg_item_wear.rs:digger_fixture`). The Priest plays live on DL5 for
  60 thinks.
  - `digger-swap-loop`: Narthanc is wielded and the shovel carried. The
    weapon slot must not change, and the Borg must leave its starting square.
  - `digger-only`: only the shovel is carried and nothing is wielded. It
    must be wielded once and kept.
- `engine/tests/test_stair_handoff.py DiggerSwapLoopTest`, also added to CI.
  It runs both cases at the shipped settings, and runs each with the flag at
  0, where each must fail: the trial wields the shovel, or the remove trial
  takes it off.
- `scripts/borg-core-vendor.py`: the `HOOKED` notes for `borg_item_wear.rs`
  and `borg_junk.rs` name the change.

## Checked

- `DiggerSwapLoopTest`: 4 of 4 pass.
- The other native suites of CI's list pass on the same test build.
- The four cargo commands and the citation check pass.
- The shipped smoke suite is 4 of 4 identical to `smoke-borg-rs`, so it
  needs no re-record.

## Not checked

- **The recorded save.** The evidence bundle is a release of
  `diegoami/angband-workspace`, outside this session's repository scope, so
  `repro.save` was not replayed. The owner can check the fix there with the
  issue's command on a build of this branch (`-seed 1 -race 5 -class 3
  -load repro.save -keep-load-rng -exact-turns -turns 5153379`). It should
  show no alternating wield and a move off the staircase.
- No zoo run. The trial change applies only while a digger and a combat
  weapon are both carried. The remove change applies only to a wielded
  digger.

## Left

- The wear and remove trials disagree because the tunnelling swap reward
  counts only for a carried digger, while a wielded digger digs as well.
  This flag stops the churn without changing that valuation.
- With the flag at 1, the ranked plan still chooses between a worn digger
  and a pack weapon by equipment power. If it ranks the digger higher, the
  digger stays wielded. This is stable, but it is not the "only for
  tunnelling" rule. The fixture does not reach this state.

## For the owner

- Whether the swap reward should also count a wielded digger, which would
  remove the cause of the second disagreement rather than its symptom.
- Whether the ranked plan should always prefer a combat weapon over a worn
  digger.

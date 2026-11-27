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

  The fixture was built before the recorded save reached this session, so
  it is a constructed equivalent, not the recorded kit. The save itself is
  replayed under *Checked*.
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
    (`keeps_wielded_digger`, called from `borg_junk.rs`).
  - The ranked plan (`borg_spare_gear::gear_plan`) replaces a worn digger
    with any combat weapon it can wear, whatever their power. The owner's
    rule, from the PR discussion: "Digger is digger". A digger is wielded
    only when no combat weapon is at hand.
  - From the Codex (GPT-6) review on the PR (P2): `borg_can_dig`
    (`borg_flow.rs`) counts the digging skill with the best carried digger
    in the weapon slot (`carried_digger_skill`), since the weapon now stays
    wielded and the engine digs with the carried digger. Without this, a
    weak-digging Priest would refuse a dig the engine makes.
  - From the Codex re-review (P2): a missing carried digger is no skill
    rather than a skill of 0. `carried_digger_skill` returns `None` when
    the flag is 0, when no digger is carried and when the worn weapon is
    sticky. At 20 twitches magma's threshold falls to 0, and a skill of 0
    had allowed a dig the engine gives no chance (`player-calcs.c:1656`).
  - From the second Codex re-review (P2): a present but weak tool must also
    give the terrain a chance. `engine_dig_chance` is the engine's chance
    by terrain, and the carried branch needs it above 0. A plain Shovel
    gives skill 30 at 20 twitches, which met granite's lowered threshold
    of 30, though the engine digs granite only above skill 40.
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
    must be wielded and kept through think 260. With the flag at 0 the
    remove trial took it off at thinks 57 and 239 (review P3). At think 260
    a plain Dagger (1d4) (+0,+0), far below the shovel in power, enters the
    pack. The plan must then wield the dagger, at think 263, and keep it.
  - `digger-dig-gate`, a Human Priest with the same kit and Narthanc
    wielded. The worn skill is 5, below granite's 40, and the carried
    shovel's is 52. At 1, `borg_can_dig` allows granite. At 0 it refuses
    it, as before. Then, with the shovel gone and 20 twitches, magma must be
    refused at both values. The reading at `9224054a` fails this at both.
    Before that, the same kit with a plain Shovel (skill 30) and 20
    twitches must not dig granite. The reading at `73eecd27` allows it.
- `engine/tests/test_stair_handoff.py DiggerSwapLoopTest`, also added to CI.
  It runs both cases at the shipped settings, and runs each with the flag at
  0, where each must fail. In `digger-swap-loop` the trial wields the shovel.
  In `digger-only` the remove trial takes the shovel off at think 57. The
  dig-gate case runs at both values and checks each one's answer.
- `scripts/borg-core-vendor.py`: the `HOOKED` notes for `borg_item_wear.rs`
  and `borg_junk.rs` name the change.

## Checked

- `DiggerSwapLoopTest`: 6 of 6 pass. A mutation check with the remove
  guard alone disabled fails `digger-only` at think 57 (review P3).
- The other native suites of CI's list pass on the same test build.
- The four cargo commands and the citation check pass.
- The shipped smoke suite is 4 of 4 identical to `smoke-borg-rs`, so it
  needs no re-record.
- After the re-review fix and a merge of `track-p` (the errands'
  retirement, #1339), every check in this section was rerun with the same
  results. `cargo test --release -p borg-core` now runs 1007 tests,
  including `engine_dig_chance_follows_calc_digging_chances`.
- **The recorded save**, from the canonical case, borg-cemetery#164
  (`incidents/2026-11-27-watch-dwarf-priest-shovel-loop-5153079`, archive
  SHA-256 `eb5e20c2…`, internal checksums verified). This branch's shipped
  build replayed `repro.save` with the case's own `borg-settings` and its
  command (`-seed 1 -race 5 -class 3 -load … -keep-load-rng -exact-turns
  -turns 5153379`), from turn 5,153,079 to 5,153,379:

  | `wear_digger_keeps_weapon` | thinks | `borg_wear_stuff` wields | moves (flow) |
  | --- | --- | --- | --- |
  | 0 (the case's settings) | 29 | 28 | 0 |
  | 1 (shipped) | 32 | 0 | 27 |

  These numbers were taken before and again after the plan rule was added,
  with the same result. At 0 the loop is the recorded one, alternating `:1114` and `:1086`, as in
  the live trace. At 1 the Priest drops surplus Phase Door scrolls and
  explores DL5. In the messages log, the 19 wields stamped turn 0 are the
  message history restored from the save. They are not played.

## Not checked

- No zoo run. The trial change applies only while a digger and a combat
  weapon are both carried. The remove change applies only to a wielded
  digger.
- No live dig in a test. `digger-dig-gate` checks the Borg's gate, and the
  digging itself is the engine's swap (`cmd-cave.c:546-571`).

## Left

- The wear and remove trials disagree because the tunnelling swap reward
  counts only for a carried digger, while a wielded digger digs as well.
  This flag stops the churn without changing that valuation.

## For the owner

- Settled: the plan always prefers a combat weapon over a worn digger. The
  swap reward is left as it is: with the digger rules in place, its
  wielded/carried difference no longer moves the weapon slot.

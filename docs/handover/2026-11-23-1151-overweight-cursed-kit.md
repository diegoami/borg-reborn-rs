## 2026-11-23: an overweight Mage's torches, cursed spares and stat curse (#1151)

**Done.** Four fixes for the CL17 Gnome Mage stuck at **Slow (−12)**
([borg-cemetery scenario](https://github.com/diegoami/borg-cemetery/tree/0cca725f4f59bd70f91467ba6af6c6fa8604a1e6/scenarios/mage-overweight-cursed-kit-9221204)).
The rules are in `docs/CALIBRATION.md` ("An overweight Mage's torches, cursed
spares and stat curse").

- `trait_curse_stats` (compiled 0, **shipped 1**): a worn dullness,
  sickliness, weakness or clumsiness curse takes its stats in the Borg's own
  stats, as in the engine. `borg_notice` took the stats off `BI_CSTR`.. and
  then overwrote them with `stat_cur`, so the curse never reached `BI_STR`,
  the weight allowance or the power model. This is a C Borg defect too
  (`borg-trait.c` does the same); it is not yet in the C defect catalogue.
- Plan schema 20 (older records keep their schema and replay as before):
  - a fuel stack splits at the light reserve (`Model::fuel_keep`);
  - over the weight allowance with no room at Home, the copies past a
    stack's floor are sold or dropped;
  - a known-cursed spare that adds no power is not protected as unknown
    over the allowance (`KitItem::cursed_spare`, kit flag 128);
  - a Remove Curse scroll bought for a curse is not charged against the
    weight, because it is read in town.

**Checked.**
- CI's commands pass, as do `check-param-citations.py` (twenty town
  citations were moved by the tool after the new methods shifted lines),
  `borg-core-vendor.py check`, and the `engine`, `scripts` and `tools` tests.
- The smoke suite on the default core: 4/4 identical to `smoke-borg-rs`, so
  there is no re-record.
- Regression tests on the observed kit:
  - `an_overweight_mage_unloads_surplus_torches_and_a_cursed_spare`: schema
    20 keeps four torches, sells thirty and drops the axe; the dagger with
    no known curse stays; schema 19 keeps everything.
  - `an_overweight_borg_still_buys_the_remove_curse_it_reads_in_town`.
  - Unit tests for `fuel_keep`, `decurses_now` and `curse_stat_add`.
- The owner's save, from turn 9,221,204, for 200,000 game turns: the
  `track-p` head (`bf76b64d`) against this branch, both built with a local,
  uncommitted frontend change so the Borg starts at all (#1154). On this
  branch the Borg takes the cap off at once: power with it 1,796,542,
  without it 1,862,497. Max HP goes from 105 to 113 and SP from 24 to 33.
  It drops the cap, then in town sells 32 torches and drops the axe. The
  first departure room reads −0.8 lb, where `track-p` read −101.8 lb at
  each of its five departures. The other carried fuel (a lantern, flasks)
  met the rest of the light reserve. It reaches DL4, while `track-p` stays on DL1. Remove Curse was
  not needed: removing the cap ends the curse. Neither arm died. This is one
  named case, not a measurement (RULES 14, 20).

**Found.** A loaded save whose first screen has a `-more-` prompt swallows
the frontend's `^Z` start, which is why the Cemetery reload issued no keys
(#1154, a frontend PR of its own under RULES 22).

**Left.**
- #1154 (frontend start on a loaded save).
- The before/after trial on the owner's save is not yet a Cemetery trial
  report. The owner decides whether it should be.
- The zoo measures `trait_curse_stats` against 0 and schema 20's town
  rules, if the owner wants a comparison before P8's exit run.

**For the owner.** `trait_curse_stats` is shipped at 1, because it is a
perception fix (the Borg's stats now match the engine's). Ship 0 if you want
it measured first.

**Review of #1155 (Codex, `1d1cc919`).** All four findings are fixed.
- *P1, split drops:* `execute_outside` drops only the planned copies of a
  split fuel stack (torches, flasks), as it does for Restore Life Levels.
  Other drops stay whole, because applying the split to every drop moved a
  smoke game in #989's review. The excess rule drops only a fuel stack.
  A stack the executor would drop whole needs a buyer. Test:
  `a_split_drop_keeps_the_reserved_copies`.
- *P1, reserves:* excess applies only when the stack's floor keeps every
  weighted reserve row (`Model::reserve_keep`). Test:
  `selling_for_weight_keeps_every_reserve`, where six Phase Door against a
  reserve of six stay. With the gate removed, the test sells them.
- *P2, oil:* the light floor counts a copy's physical fuel
  (`KitItem::fuel`, a new kit column in schema 20). Test:
  `the_light_floor_counts_physical_flasks`, where two flasks stay, not six.
- *P2, Remove Curse:* the weight exemption applies only while the scroll
  can be read, and only to the copies needed now. Tests:
  `an_overweight_borg_still_buys_the_remove_curse_it_reads_in_town`
  (blind, and one copy) and `a_remove_curse_ware_is_read_now_only_for_a_curse`.

**Re-review of #1155 (Codex, `99741c93`).** All three findings are fixed.
- *P1, reserves when storing:* the reserve is the stack's floor before the
  move is priced (`reserve_keep` in place of `fuel_keep`), so six Phase Door
  against a reserve of six are neither stored nor sold. Nine against six,
  with a full Home, sell the three past it. Test:
  `selling_for_weight_keeps_every_reserve`. It fails with the light floor
  alone.
- *P2, dropping the whole surplus:* a disposal that happens only because the
  pack is over its allowance moves just the copies that cover the deficit.
  At 1 lb over, one torch is dropped, not thirty. Same test.
- *P2, Remove Curse after enchanting stops:* the plan reads the live
  `reads_scroll_in_town`, which covers `borg_enchanting`'s town timer and
  `borg_read_scroll`'s light. The plan record keeps it (`spells` bit 1,
  schema 20). Test: `an_overweight_borg_still_buys_the_remove_curse_it_reads_in_town`.
- `track-p` (`85f8800b`, #1148) is merged in. The conflicts were the
  parameter citations in `params.rs` and `docs/CALIBRATION.md`, which were
  taken from `track-p` and regenerated with `check-param-citations.py --fix`.

**Second re-review of #1155 (Codex, `b22e6e4b`).** The finding is fixed. The
ordinary disposal path could plan dropping one Phase Door of seven (a
reserve of six, no buyer, a full Home), and the executor drops that stack
whole. When a reserve sets the floor, a drop of a stack the executor would
drop whole (anything but fuel, Restore Life Levels or a kind at its Home
limit) is now kept instead. A split set only by `town_store_keep_min` keeps
its old behaviour, which the smoke game relies on (#989). Test:
`selling_for_weight_keeps_every_reserve`, over and within the allowance. It
fails without the guard.


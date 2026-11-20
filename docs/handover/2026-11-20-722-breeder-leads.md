## 2026-11-20: two breeder leads from #722's analysis

- **A one-think pause latched for the level.** With `caution_breeder_stoppable`, a think with danger at or above the HP pauses the stand for that think only (`breeders_in_danger`). But `breeder_stand` false then also triggered three level-long effects:
  - the "ignoring breeders" leave of `borg_check_breeders`;
  - the breeder flee of `borg_caution`;
  - the breeder scary trigger of `borg_near_monster_type`.

  `breeder_stand_paused` (`borg_caution.rs`) now keeps those three from following a pause. The normal caution still acts that think: retreat, escape. Unit test `breeder_pause_tests`.
- **A blind Borg skipped its mana rest to hunt.** `fit_to_hunt` now counts blindness, as #644's `impaired` does (`borg_flow_kill.rs`, `breeder_hunt_now_with`).
- No new parameter. Both are fixes of the shipped policy's own intent.
- **Measured** (classes-18 plus `809881907`, 100,000 game turns, main against the fix):
  - 17 of 19 keys identical;
  - deaths 1 and 1 (the same Druid, an invisible clear icky thing);
  - sum of max CL 102 to 104; sum of max DL 35 to 36.
- The real-character breeder test bed (living saves plus `-start-monsters`) is PR #741's change; its results are on #722.

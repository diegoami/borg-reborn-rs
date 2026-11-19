## 2026-11-19: no rest with a monster near (#615, rule a)

Owner, on #615's rest analysis (borg-cemetery#74): "ok do a"; b and d as a
resting-place rule (#642: a staircase, else a room corner away from the
entrances, the owner's empty-room routine), c after a kill check (#643), and
a new e: no rest while breeders are known (#644).

- **Done:** `recover_rest_monster_turns` (game turns, compiled and shipped 0)
  and `recover_rest_monster_distance` (20) hold the rod, HP/SP and mana rests
  while an awake monster that can move was near recently. Stacked on #640
  (#567), whose memory scan it shares. Details in docs/CALIBRATION.md.
- **Found:** held while confused, a Ranger walked into lava in town and
  died; the rule now keeps the rest when confused, blind, hallucinating or
  stunned.
- **Checked** (after the review of #645): off plays #640's keys in 18 of 18
  classes-18 starts. At 100: 2 deaths against 1 (the same Priest, and a
  Druid 9,000 game turns after its last held rest), sum of CL 96 against
  101, rests started 1,203 against 1,893. At 200: the same one death, sum of
  CL 97. The reviewer's 66 games: 3 deaths at 100 and 200 against 6 off.
- **Review of #645, fixed:** the clock clears when the monster that set it
  dies (a forgotten monster still counts: #567's case); afraid is impaired;
  the walk to a recovery grid waits too; a unit test
  (`no_rest_soon_after_a_monster_was_near`); the docs say what
  `borg_check_rest` and the game do.
- **Left:** a zoo pilot (off, 100, 200); #642, #643, #644. The owner
  decides the shipped value.

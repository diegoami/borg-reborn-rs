## 2026-11-19: no Recall rest right after a monster was near (#567)

From `priest-809881907` (borg-zoo#227, #539): the Priest saw Grip, read its
only Word of Recall, forgot Grip a think later (out of view, danger 0) and
rested for Recall; Grip came back and bit.

- **Done:** `caution_recall_rest_monster_turns` (game turns, compiled and
  shipped 0) and `caution_recall_rest_monster_distance` (20 grids). While
  Recall is pending, an awake monster known within the distance in the last
  N game turns, on this level, holds the three Recall rests ("Resting for
  Recall", "Waiting for Recall", the wait with no light). The Borg stays still one turn instead (`,`;
  a rest count of 1 would repeat the previous count). `borg_caution` records
  the game turn each think. Monsters that never move do not count. Details
  in docs/CALIBRATION.md.
- **Checked:**
  - with the switch off, the 18 classes-18 starts at 100,000 turns play main
    `13e1187c`'s keys in 18 of 18 games;
  - with 200, it fires 135 times in 9 games. Deaths 1 against 1 (the same
    Priest at 8,158, before any watch). Sum of CL 95 against 101: the
    Blackguard 7 to 3, its Recall firing at the same game turn in both and the
    games diverging after it. A lead, not a measurement;
  - unit test `no_recall_rest_soon_after_a_monster_was_near`.
- **Found:**
  - the case does not replay on main from the 1,721 save (`~/borg-play/
    priest-809881907`): main fights Grip at 1,991 instead of reading Recall,
    also with #539's calming off. Only the original build reaches the scene;
  - in a first version a floating eye (never moves) held every Recall wait in
    that replay, hence the exclusion.
- **Related:** #615's analysis (borg-cemetery#74) ranks this rule, applied
  to every rest, as its first candidate: 1,609 dungeon rests start with an
  awake monster seen recently, and 24% of them are interrupted.
- **Review of #640, fixed:** only a monster seen in this update refreshes
  the watch (`kill.seen`); stale records had refreshed it every think (192 of
  420 watches at "0 game turns ago" in the reviewer's run; now 6 of 111, the
  longest run 3 thinks). The third Recall wait (no light,
  `borg_think_dungeon_light`) is held too. A test of the scan
  (`mover_seen_within`). Main `3ba17e2c` merged (2171).
- **Re-measured** at the fixed head: off plays main's keys in 18 of 18; at
  200, 7 of 18 play differently, sum of CL 97 against 101, the same one
  death.
- **Owner:** #567's "the case replayed from the 1,721 save" cannot be met
  on today's core (main fights Grip instead of reading Recall); decide
  whether the issue closes without it.
- **Left:** a zoo pilot (off, 100, 200 game turns). The owner decides the
  shipped value.

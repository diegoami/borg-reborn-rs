## 2026-11-19: no Recall rest right after a monster was near (#567)

From `priest-809881907` (borg-zoo#227, #539): the Priest saw Grip, read its
only Word of Recall, forgot Grip a think later (out of view, danger 0) and
rested for Recall; Grip came back and bit.

- **Done:** `caution_recall_rest_monster_turns` (game turns, compiled and
  shipped 0) and `caution_recall_rest_monster_distance` (20 grids). While
  Recall is pending, an awake monster known within the distance in the last
  N game turns, on this level, holds both Recall rests ("Resting for
  Recall", "Waiting for Recall"). The Borg stays still one turn instead (`,`;
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
- **Left:** a zoo pilot (off, 100, 200 game turns). The owner decides the
  shipped value.

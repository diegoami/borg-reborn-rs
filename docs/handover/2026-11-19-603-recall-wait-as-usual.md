## 2026-11-19: play by the ordinary rules while Word of Recall counts down (#603)

Owner: "I do not want it to behave differently after Word of Recall." While `goal.recalling` is set, the core switched to Recall-only rules (no flight by stairs or Teleport Level, its own buy-time escapes, fighting a scary monster because recalling). The engine keeps the countdown through a level change, so none of them is needed for the jump.

- `caution_recall_wait_as_usual` (0, off in `harness/borg.txt`): 1 makes `recall_restricts` false, so the seven places the issue lists act as without Recall:
  - the flee and leave goals (`borg_think_dungeon.rs`);
  - stairs down in caution, and the staircase as an escape (`borg_caution.rs`);
  - the stair flows (`borg_flow_stairs.rs`);
  - Teleport Level (`borg_escape.rs`);
  - fighting because recalling (`borg_fight_attack.rs`).

  `buy_time_for_recall` is skipped. Kept: resting or waiting in place when safe, the town buffs before the jump, and "not bored".
- The measurement is in the PR (classes-18, the switch at 0 against 1).

Left: a zoo pilot. Related: #582 and #583 (what the Borg does during the wait), #567 (no rest with a monster near).

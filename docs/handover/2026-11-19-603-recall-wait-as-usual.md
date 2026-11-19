## 2026-11-19: play by the ordinary rules while Word of Recall counts down (#603)

Owner: "I do not want it to behave differently after Word of Recall." While `goal.recalling` is set, the core switched to Recall-only rules (no flight by stairs or Teleport Level, its own buy-time escapes, fighting a scary monster because recalling). The engine keeps the countdown through a level change, so none of them is needed for the jump.

- `caution_recall_wait_as_usual` (0, off in `harness/borg.txt`): 1 makes `recall_restricts` false, so the seven places the issue lists act as without Recall:
  - the flee and leave goals (`borg_think_dungeon.rs`);
  - stairs down in caution, and the staircase as an escape (`borg_caution.rs`);
  - the stair flows (`borg_flow_stairs.rs`);
  - Teleport Level (`borg_escape.rs`);
  - fighting because recalling (`borg_fight_attack.rs`).

  `buy_time_for_recall` is skipped. Kept: resting or waiting in place when safe, the town buffs before the jump, and "not bored".
- Only on DL2 and deeper (owner, after the first measurement): in town and on DL1 a staircase crosses between town and dungeon. In one game a Mage that had read Recall in town to go down walked to the down staircase instead of waiting; had it got there, the Recall would have fired on DL1 and taken it back to town. A Borg fleeing up from DL1 would be taken down to its deepest level.
- The measurement is in the PR (classes-18, the switch at 0 against 1).

Left: a zoo pilot. Related: #582 and #583 (what the Borg does during the wait), #567 (no rest with a monster near).

**Review of #627** (first round): the change is sound.
- `every_switched_site_asks_recall_restricts` counts the switch's check in each of the five files, so a site put back to `goal.recalling` fails the test.
- The PR now lists every kept read of `goal.recalling`. That includes the engine-counter sync (`borg_trait.rs`), which must stay. It also names the gap in restoring mana: restoring mana has no ordinary equivalent of `buy_time_for_recall`.
- At 1 the Recall still fires and still lands the Borg in town.
- **For the owner:** a Borg out of light (`borg_think_dungeon_util.rs:209`) waits for the Recall with no danger test, before `borg_caution`, so the switch does not reach it. Making it a wait only when safe changes play at 0 too.
- Merged with main after #595 and #631: the registry holds 2161.

## 2026-11-22: Cures near death, and learning unknown cures (#985)

**Core:** every check ran on the default **borg-rs** core (`borg-core/`,
`harness/borg.txt`). None ran on the reborn core.

### Found

Seed 1895660873 replays key for key on track-p `4b2c2fcf` (`31c994254ecdb188`).
The issue's lead is **refuted**. The bravery boost held back no cure:
- `boost_bravery` resets the avoidance to the HP after its rerun;
- the brave think does not call `borg_caution`;
- every fatal `borg_attack` think has avoidance = HP (2, 1, 1, 0) in `trace-core.jsonl`.

The cause:
1. **The cures were unaware flavours.** The "6 CLW, 2 Neutralize Poison" in the
   gear dump are the game's names. The Borg held 6 Tangerine (sval 8), 2
   Orange (sval 14) and 2 Purple (sval 27) potions, all unaware. #470 never
   uses an unaware item, so `borg_heal_outmatched` applied at HP 2/25,
   danger 44, and found nothing to drink. This was seen in a temporary
   instrumented build, not committed.
2. **The kinds were never learnt.** On every town visit after the pickups,
   C's "Nothing to sell in town (leaving)" fired (CL < 6, gold < 10,
   `borg_count_sell` < 5) before the town planner could sell one of each
   unaware kind (#470).
3. In `borg_heal`, the gate "Dont do these in the middle of a fight" compares
   the danger with 2/10 of the avoidance, which is the HP. So the lower the
   HP, the less danger a cure tolerates: at 6 HP a danger of 2 already
   refuses Neutralize Poison. This did not decide this game (no cure was
   known), but it would with a known one.

### Done

Two switches, both 0 compiled and shipped (today's play; `smoke-borg-rs` 3/3
identical, not re-recorded):
- `caution_cure_hp_floor_percent`: below this percent of the max HP, a
  poisoned or bleeding Borg passes the fight gate.
- `sell_poor_town_unknown_hold`: the poor-town exit waits while an unaware
  kind is carried.

Replays (one start each, no claim):

| arm | keys sha256 | end |
|---|---|---|
| defaults | `31c994254ecdb188` | the same death |
| floor 30 | `31c994254ecdb188` | the same death |
| hold 1 | `3810978f26a68560` | 100,000-turn cap at CL8 (it sells one Orange Potion at 6561) |

Unit tests cover both predicates and their defaults. Details are in
`docs/CALIBRATION.md` ("#985").

### Left

- **The Cemetery case is not posted.** This session had no write access to
  borg-cemetery. The package (README, replay script, arms, evidence) has to
  be posted there from the session's scratchpad or rebuilt with the commands
  in the PR.
- The zoo measures both switches. Its request issue is the owner's call
  (RULES 20).
- The Ranger sells its first spellbook at CL1, turn 221 (a new issue).
- The boost's brave flow still walks a poisoned Borg back into the worms at
  HP 7 (#720).

### For the owner

Should the hold also apply above CL5 or with gold? Today it covers only the
C exit's own conditions.

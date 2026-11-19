## 2026-11-19: rest at the stairs (#614)

Owner, from Elf Druid 1678765969's death in borg-zoo#312 (a fox fleeing to
stairs 11 steps away stopped to rest for mana and was caught): once
committed to a stair, reach it before resting; rest on the stairs and take
them if a monster appears. In any form, not only a fox.

- **Done:** `recover_rest_reach_stairs` (steps, 0 = off) holds the rod, HP
  and mana rests on a flight to a staircase at most that many steps away.
  `caution_stairs_leave_on_monster` (flag) takes the stairs when a monster
  threatens a Borg that is recovering on a staircase. Both are compiled and
  shipped 0. Details and the evidence table are in docs/CALIBRATION.md.
- **Found:**
  - held at any distance, the rule changed 16 of 18 games and two died, one
    after holding its rests on a 138-step flight; so the switch is a
    distance;
  - a new monster wipes the flow goal for a think, and the rest slipped in
    then; the flight's last step is now remembered (`borg.flight_steps`,
    `flight_time`) and holds the rest for 10 Borg-time units;
  - the scene: the Druid now reaches the stairs (game turn 67,268), rests on
    the next level's stairs and lives;
  - on the 18 starts at 20: 2 deaths against 0 (a Mage, a Druid), both
    thousands of turns after the games differ. The leave rule alone: no
    deaths, sum of CL 102 against 99.
- **Review of #616, fixed:**
  - the leave rule never fires in town (it dove into DL1) and needs a real
    threat (`caution_stairs_leave_danger_percent`, 10);
  - its direction is a switch (`caution_stairs_leave_down`, 1 = up or down);
  - the flow to a recovery spot is held too;
  - the flight memory is a parameter (`recover_rest_reach_stairs_memory`)
    and forgets a flight on another level.

  On the 18 starts the leave rule fires 23 times instead of 72, and 10 of 18
  games stay identical. Its one death (a Priest among breeders) comes 23,000
  turns after its last leave.
- **Left:** a zoo pilot (off, reach 20, leave, leave up only, both). The general question of
  where and when to rest is #615. Phase Door during a flight is #584.
- **Owner:** the shipped values, after the pilot.

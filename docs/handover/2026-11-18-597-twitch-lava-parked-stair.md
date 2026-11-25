## 2026-11-18: the twitch never steps into lava; a parked staircase is not lost (#597)

priest-809881907, replayed from birth on main with the shipped `borg.txt`, died "burning to a cinder in lava" in town at full HP (53/53), game turn 92,185. Traced with debug notes:
- **The staircase.** The only staircase it knew, the town's down staircase at (32,1), had a monster marked on it from turn 90,895. `enqueue_stairs` skips such a staircase, so every stair flow had no goal, and each turn fell through to the twitch.
- **The twitch.** `borg_twitchy` refused walls only, and stepped into lava: 100 + 1d100 fire damage (`player-util.c:918`), one step.

Two settings, compiled 0 (C), shipped 1:
- `caution_twitch_avoid_lava`: the twitch's random pick and its fallback pass refuse lava without fire immunity. If every direction is wall or lava, the Borg rests.
- `caution_stair_parked_fallback`: when a monster is marked on every known staircase of a kind, the stair flow enqueues them anyway.

priest-809881907 from birth, 120,000 turns:
- both on: alive, CL8, no twitch;
- the fallback alone: the same;
- lava avoidance alone: alive, but it twitches 585 times;
- both off: dies in lava.

Smoke is 3/3 identical. The classes-18 comparison is in the PR. `borg_flow_misc.rs` is now declared hooked.

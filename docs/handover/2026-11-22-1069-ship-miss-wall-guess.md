## 2026-11-22: Ship update_miss_wall_guess = 1 (#1069)

The owner asked for this on 2026-11-22, after #1005 merged ("I would set it to 1").

`harness/borg.txt` now ships `borg_param_update_miss_wall_guess = 1`. #929's fix (#1005) stays compiled 0, so `BORG_CORE_OVERRIDES=off` runs and `borg-core/borg.txt` are unchanged. The shipped core now:
- does not take a miss at a monster in view for a wall;
- never guesses a wall on the Borg's own grid or on the target's grid;
- always allows the next shot.

All checks ran on the default core, borg-rs. `smoke-borg-rs` is re-recorded (`scripts/rerecord-smoke-borg-rs.sh`). At 1 the three smoke games play the same key streams; only messages.log changes, with "a miss, not a wall" notes in place of C's guesses. The two Fang games (Paladin 1194051723, Druid 1024822556) play the same at 0 and at 1.

**Left:** no zoo measurement was made before shipping; that was the owner's call. One should follow, and it should cover the light-0 case (Necromancer 798380952), where the granite under the Borg persists at 0. That is the case where 1 can change play.

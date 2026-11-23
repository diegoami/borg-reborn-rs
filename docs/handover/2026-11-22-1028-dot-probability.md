## 2026-11-22: Own cut and poison counted over the exact window, and the potion names in the trace (#1028)

**Core.** Every check here ran on the **default borg-rs core** (`borg-core/`,
`harness/borg.txt`). This follows #926 (PR #1002, merged as `4383976b`),
whose re-review left the residuals R1–R5 in #1028.

**Done.**
- **R1.** `dot_kills` (`borg_fast_fragile.rs`) counts the world ticks of
  the coming window exactly (`window_ticks`): the player acts before the
  turn's tick, the world ticks every tenth turn, and the player gains
  `turn_energy` a turn (game-world.c:1016-1095). The count uses the game
  turn and the Borg's energy (a new `engine::player_energy` accessor in the
  hooked `engine.rs`). The Borg drinks the stopping potion when ticks times
  the HP a tick takes it below 0. At normal speed every window holds one
  tick, as before; a hasted Borg's windows hold one tick and none in turn.
  The first version of this PR compared a probability averaged over the two
  phases with a new `fast_fragile_dot_death_percent`; the review of #1068
  (F2) showed that it spends the cure on the safe windows too, so the
  parameter is gone and the count is exact.
- **R4.** The cure note names the potion ("Cure Serious Wounds", "Neutralize
  Poison"), not the enum.
- **Review of #1068** (MiniMax-M3.1, named by the owner: pass with
  findings). F1: the note now gives the ticks and the HP a tick the rule
  counted. F2: the exact window count above. F3: the tests are in
  `tests_1028`. F4: borg-zoo#672 is re-pointed to this PR's head.
- Everything acts only with `fast_fragile_count_dot = 1`, which ships 0.

**Not done, and why.**
- **R2** (the stopping cure outside attack on sight, for a Borg fleeing a
  fast-fragile monster) changes the heal chain or the flight in
  `borg_caution.rs`, the area #981/#982 change. The owner decides whether
  and where.
- **R3** (an end-to-end case for the branch that drinks the potion) is a
  constructed scenario, borg-cemetery's (RULES 12). The owner decides.
- **R5** asked to update the #926 handover's pins. A task changes only its
  own note (RULES 5b), so the #926 note stays; the current pins are here.
  PR #1002 merged at head `345cf356`.

**Checked** (default borg-rs core, merged with `track-p` `21c0d9c1`). `cargo fmt
--check -p borg-reborn-rs`, clippy, `cargo test --release -p borg-core` (710
passed; `tests_1028`), the workspace tests, `scripts/tests` and
`scripts/check-param-citations.py` all pass. Smoke is 3/3 identical to
`smoke-borg-rs`. Both #926 seeds play the same keys on this branch as on the
bare `track-p` tip, with the defaults and with `fast_fragile_count_dot = 1`:
1398933929 `9653a0d480ef1dd2` (dead at 1830); 1294425391 `2a7f2615f8c44b0b`
(alive at the cap; the hash moved with `track-p` since #1002, through #1057
and others).

**Not checked.** No seed exercises the branch where the Borg drinks the
potion (R3), so R1 has unit tests only.

**Owner.** `fast_fragile_count_dot` is a hypothesis (RULES 14). borg-zoo#672's
`dot` arm now names this PR's head; the rule it measures is not the one #1002
shipped (that one compared the expected damage).

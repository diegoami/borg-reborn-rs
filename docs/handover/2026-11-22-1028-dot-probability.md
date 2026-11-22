## 2026-11-22: Own cut and poison as a probability of death, and the potion names in the trace (#1028)

**Core.** Every check here ran on the **default borg-rs core** (`borg-core/`,
`harness/borg.txt`). This follows #926 (PR #1002, merged as `4383976b`),
whose re-review left the residuals R1–R5 in #1028.

**Done.**
- **R1.** `dot_kills` (`borg_fast_fragile.rs`) asks for the probability
  that the Borg's own cut and poison take it below 0 HP before its next
  action, not for the expected damage. That is `dot_death_probability`: the
  world ticks in the window are `floor(10 / energy)`, or one more with the
  fractional part as its probability, the mixture `turn_pmf` takes for
  monsters. It is compared with the new `fast_fragile_dot_death_percent`
  (default 50, 1–100). At normal speed the probability is 0 or 1, so any
  value plays the same; only a hasted or slowed Borg sees a difference.
  Examples: +10, HP 2, a Deep Gash gives 1/2 (the expectation, 1.5, said no
  cure); +40, HP 1, a Severe Cut gives 10/42.
- **R4.** The cure note names the potion ("Cure Serious Wounds", "Neutralize
  Poison"), not the enum.
- The new parameter is in `params.rs`, `harness/borg.txt` (50) and
  `docs/CALIBRATION.md`. It acts only with `fast_fragile_count_dot = 1`,
  which ships 0.

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

**Checked** (default borg-rs core, `track-p` `7a466d73`). `cargo fmt
--check -p borg-reborn-rs`, clippy, `cargo test --release -p borg-core` (704
passed, 2 new tests), the workspace tests, `scripts/tests` and
`scripts/check-param-citations.py` all pass. Smoke is 3/3 identical to
`smoke-borg-rs`. Both #926 seeds play the same keys on this branch as on the
bare `track-p` tip, with the defaults and with `fast_fragile_count_dot = 1`:
1398933929 `9653a0d480ef1dd2` (dead at 1830); 1294425391 `2a7f2615f8c44b0b`
(alive at the cap; the hash moved with `track-p` since #1002, through #1057
and others).

**Not checked.** No seed exercises the branch where the Borg drinks the
potion (R3), so R1 has unit tests only.

**Owner.** The value 50 is a hypothesis (RULES 14). borg-zoo#672's `dot` arm
names PR #1002's head; if the zoo measures `dot`, it should build this
branch once merged.

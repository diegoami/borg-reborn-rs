## 2026-11-24: The descent gates without a seen up staircase (#1177)

Track P (P9), branch `task/T1177-more-refused-gates`, target `track-p`.
Based on `track-p` at `3c46575d`.

### What was done

- The defect (`borg-core/src/borg_flow_stairs.rs`, `more_refused`): C wraps
  its four descent refusals — not prepared for the next depth unless
  `brave`, hungry or weak or low on food, a held sale, no light — in *"if
  there are no down stairs, don't filter use of up stairs"*
  (`borg-flow-stairs.c:193`): with no up staircase seen, all four are
  skipped and a character dives past gates that would have stopped it. The
  Rust mirrors C exactly; the defect is tracked upstream (better-borg-c
  #540, neo-angband-tracker #629), so this is a decision, not an upstream
  repair.
- The switch: `stairs_descent_gates_without_upstair` (flag, shipped 0 per
  the P9 rule, PLAN.md): 1 applies the four gates whether or not an up
  staircase has been seen; 0 keeps C's play. Read in `more_refused`
  (`borg_flow_stairs.rs:982`); every caller inherits it (the stair flows,
  the trap-way descent, caution's flee-descent checks). The params
  registry's prefix↔module table gained `stairs` → `flow_stairs`
  (`params.rs::defaults_and_registry_are_valid`), which the new param's
  name relies on.
- Shipped settings: `harness/borg.txt` gains the parameter at 0 (play and
  the P8 measurement unchanged); `docs/CALIBRATION.md` gains the section
  and the table row.
- The regression (through the dungeon decision entry point, per the
  #1174 review's standard, using `docs/DECISION-FIXTURES.md`'s recipe): a
  native case `more-refused-gates` (`run_more_refused_regression`), driven
  by `engine/tests/test_stair_handoff.py::MoreRefusedGatesTest` (CI wired).
  One arm runs per driver case, since the setting is fixed for the whole
  process: a fabricated DL3 with a reachable down staircase, no up
  staircase anywhere, and a food-less pack (the hunger gate's grounds) on a
  dive-prone birth Warrior (the `prepared_*` pins, as in #1173):
  - at 0 the dive commits `GOAL_BORE` and steps toward the down staircase;
  - at 1 no descent flow commits and no stair is taken in either think.

### How it was checked

- Teeth: with the gate disabled (`|| false &&`), the 1 arm fails — the
  hungry dive commits `GOAL_BORE` (6). Restored: both cases pass. (A first
  teeth attempt asserted "no `>`" and passed disabled: a dive queues a
  movement digit, not a stair key — the flow commit is the observable.)
- `EmergencyStairHandoffTest` 5/5, `LeaveLevelUpstairsTest` OK,
  `MoreRefusedGatesTest` 2/2 on the test build.
- CI's commands green: `cargo fmt --check -p borg-reborn-rs`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (915 passed),
  `python3 scripts/check-param-citations.py` (2524 parameters, 0 wrong).
- Smoke (default borg-rs core, shipped settings): 4/4 identical to
  `harness/reference/smoke-borg-rs` — no re-record.

### What is left

- The zoo comparison that turns the switch on is a separate request
  (#1177's CALIBRATION section says the value is a hypothesis); it joins
  the #1175 pattern when the owner wants it run.

### For the owner to decide

- Nothing in the task; the PR waits for review per RULES 1a.

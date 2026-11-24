## 2026-11-24: Caution's standing descent and the stale stair flag (#1178)

Track P (P9), branch `task/T1178-stale-stair-more`, target `track-p`.
Based on `track-p` at `3c46575d`.

### What was done

- The defect: `stair_more` is set **before** the descent flows it belongs to
  (the leaving block, the power dive, `borg_leave_level`'s dive) and
  survives their failure until a new level clears it (`new_level`,
  `borg-update.c:2172` in C too). A later think standing on a down
  staircase then takes it in caution's "Take stairs down" on the stale
  flag, past the descent gates the dive itself would have applied
  (`more_refused`): a hungry or unprepared character descends standing
  where the dive would have been refused.
- The shape: clearing the flag on a failed flow was **rejected** — it would
  also break the same-think handoff that lets caution complete a fresh
  standing descent (`borg_leave_level` sets the flag, caution consumes it
  later in the same think), and the stale completion is C's intended
  mechanism (the dive block runs after caution, so its flag is only ever
  consumed by a later think). The switch instead gates caution's standing
  take with the descent gates: a stale flag completes a descent only when
  the gates agree, and a flag the same think's own dive set passes them
  again. The hungry standing descent C allows is refused at 1.
- The switch: `caution_standing_descent_gated` (flag, shipped 0 per the P9
  rule, PLAN.md), read at the standing take (`borg_caution.rs:4703`).
- The drift: the insertion shifted 65 `borg_caution.rs` citations in
  `params.rs` (and CALIBRATION.md's table rows); all 65 re-pointed (+12
  lines), `check-param-citations.py` 2524/0 again.
- The regression (`stale-stair-more`, `StaleStairMoreTest`, CI wired),
  built with `docs/DECISION-FIXTURES.md`'s recipe: think 1 through
  `borg_think_dungeon` — a full larder, a dive-prone character (the
  `prepared_*` pins, as in #1173), a dive that fails on no known down
  staircase, the flag survives; then the food is taken away, the notice
  traits are set as an empty larder reads, and the Borg is placed on a
  `FEAT_MORE` grid. Think 2 drives `borg_caution` **directly** (the
  `EmergencyStairHandoffTest` pattern): the decision under test is
  caution's standing take, and the barren level's full dungeon tail
  (spastic searches, bravery boosts, the twitchy recall read) would only
  bury it. At the shipped 0 the stale flag takes the descent (`>`); at 1
  the gates refuse it.
- Fixture lessons folded into the recipe's base (#1173 pattern): think 1
  needs a **full** larder — the restock gate of `prepared()` fails on
  `BI_FOOD < 3` for every depth, so a food-less character can neither dive
  nor keep the leashes out; and think 1's notices rebuild the staircase
  track from the cave's marks, which fabricated grids do not carry, so the
  think-2 state re-asserts the fabricated track knowledge.

### How it was checked

- Teeth: with the gate disabled, arm 1 fails — `more_refused` refuses
  (food 0) but the take proceeds and `>` is queued. Restored: both cases
  pass.
- `EmergencyStairHandoffTest` 5/5, `LeaveLevelUpstairsTest` OK,
  `StaleStairMoreTest` 2/2.
- CI's commands green: `cargo fmt --check -p borg-reborn-rs`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (915 passed),
  `python3 scripts/check-param-citations.py` (2524 parameters, 0 wrong
  after the drift fix).
- Smoke (default borg-rs core, shipped settings): 4/4 identical to
  `harness/reference/smoke-borg-rs` — no re-record.

### What is left

- The zoo comparison that turns the switch on is a separate request (the
  CALIBRATION section says the value is a hypothesis).

### For the owner to decide

- Nothing in the task; the PR waits for review per RULES 1a.

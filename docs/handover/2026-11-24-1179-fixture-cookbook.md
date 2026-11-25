## 2026-11-24: The decision-fixture cookbook (#1179)

Track P (P9), branch `task/T1179-fixture-cookbook`, target `track-p`.
Based on `track-p` at `3c46575d` (after the #1174 merge).

### What was done

- `docs/DECISION-FIXTURES.md`: the recipe for building a decision fixture —
  driving `borg_think_dungeon` on a fabricated state — written from the
  traps hit while building the #1173 regression and its #1174 review round.
  Covers the pieces table (native fixture, driver, hook, build, CI wiring),
  the ten-step recipe (track **and** grid feat for a staircase; aware items
  with real kinds; merged stacks instead of junk-crushed duplicates; the
  synthetic start's `birth_force_descend`; the per-think state resets and
  the twitch guard; the power sentinel; in-place settings replacement and
  its two traps — per-class `__class` overrides and non-param settings like
  `borg_trip_policy`; driving the decision entry point with one queued key
  per think; the teeth check; `std::process::exit(0)`; the smoke identity
  check for shipped-off parameters), and the worked example.
- Indexed in `docs/README.md` (Architecture, next to CORE-TRACE.md).
- No code changed; `harness/borg.txt` untouched.

### How it was checked

- Docs-only change: `cargo fmt --check -p borg-reborn-rs`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core`,
  `cargo test --release -p borg-core` (911 passed) and
  `python3 scripts/check-param-citations.py` (2522, 0 wrong) re-run green.
- No smoke run needed: no code or setting changed.

### What is left

- The recipe is prose, not a helper: if the next fixture turns it into a
  fixture-builder module, that is a new task.

### For the owner to decide

- Nothing in the task; the PR waits for review per RULES 1a.

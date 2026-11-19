## 2026-11-19: ZOO-INTERFACE for borg-zoo PR #127 (#414)

`docs/ZOO-INTERFACE.md` now matches the zoo's borg-rs adapter after borg-zoo PR #127 (RULES 7). Documentation only; no code or play changes.

- **The loaded-plan copy** is guaranteed output: `BORG_RUN_DIR/errands_plan.txt`, which defaults to the parent of `-summary`. It holds the plan's bytes as parsed and is absent in no-plan arms. A change to it is a contract change. Sources: `errands_plan.rs` (plan load), `main-borgh.c` (`BORG_RUN_DIR` default).
- **Cleared variables:** every inherited `BORG_*`. `BORG_ERRANDS_PLAN` is the plan variable; `BORG_PLAN` was a stale name.
- **The frozen-turn loop flag:** 1,000 or more keys at one game turn → `stalled`, `frozen_turn_loop`. The core's guard against borg-rs#90's loop is `home_moved`. No known legitimate behaviour reaches 1,000.
- **The reproduction on current main** (Human Warrior seed 501, `errands_plan_ambitious.txt`, 100,000 turns, `harness/run.sh`): 12,163 keys, at most 30 at one game turn, no burst of 1,000. At `1c4b351b` it gave 69,838 keys and 12 bursts.

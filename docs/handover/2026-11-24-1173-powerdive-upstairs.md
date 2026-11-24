## 2026-11-24: Power dive ignores reachable up stairs after exploration is exhausted (#1173)

Track P, branch `task/T1173-powerdive-upstairs`, target `track-p`.

### What was done

- Verified the recorded incident against the trace and messages
  (`~/angband-recordings/diagnostics/20261124-105142.ckrnXT/`): exactly 521
  Borg decisions between turns 177,047 and 182,257 on DL3, all movement keys,
  zero danger, no goals set (`borg_flow_old` continuations and
  `borg_flow_spastic`). At 177,047 the notes show the loop whole:
  `power dive, playing too shallow` → `C exploration: nothing reachable left
  within the leash, leaving the level (#1098)` → `Exploration from the
  stairs: no staircase to leave by, exploring on (#1098)` → `Flowing toward
  spastic`. The suspected cause is confirmed: in `borg_leave_level`, the
  power dive sets `g = 1` before the exhausted branch, which engages the
  `either_way` fallback only when `g == 0`; the failed-down fallback (#1098
  re-review) then cannot try the known up stairs, and the level stands down
  to the secret-door search every think.
- Fix (`borg-core/src/borg_think_dungeon_util.rs`, `borg_leave_level`): after
  the descent flow fails, when this think's exploration is exhausted
  (`exhausted_this_think`), this call has not tried the up stairs yet
  (`up_tried`, set in the `g < 0` block), and descent is not forced, the
  known up stairs are tried (`GOAL_BORE`, like the #1098 fallbacks). Setting
  `stair_less` lets `borg_caution`'s "Take stairs up" press `<` when the
  Borg reaches or stands on the staircase; `release_after_failed_exit` stays
  for the case where no staircase can be taken. Shipped settings unchanged;
  the fallback is scoped to the exhausted mark, so timer-only leaving, dives
  with a working down stair, retreat holds and forced descent are untouched.
- Regression through the dungeon decision entry point
  (`borg_think_dungeon`), as a `test-stair-handoff` native fixture case
  (`leave-level-upstairs`, `run_leave_level_regression`) driven by
  `engine/tests/test_stair_handoff.py::LeaveLevelUpstairsTest` (CI wired):
  - the incident: exhausted + dive + no down stair + two reachable up
    stairs → the Borg flows to an up stair and, once on it, takes it (`<`);
  - a reachable down stair keeps the dive (`# Powerdiving.`);
  - forced descent (`birth_force_descend`) still forbids the up stair and
    stands the level down;
  - an explicit retreat (`goal.rising`) holds its descent (#483), and an
    explicit leaving (`goal.leaving`, the restock decisions' way) still ends
    in the up-staircase flow. The `need_restock` recall at depth >= 5 sits
    inside the unchanged `g < 0` block.
  The fixture drives a birth-level Warrior on a fabricated fully-explored
  DL3 (a known floor sea, two up stairs within the leash, no down stair);
  its own borg.txt pins the `prepared_*` gates (base and `__warrior`) to 0
  and `prepared_clevel_depth_margin` to 7 so the character is dive-prone
  like the incident's CL8 Ranger, pins `explore_breadth_first`,
  `explore_mixed`, `stair_scum`, `deal_with_poison` off and
  `explore_c_leave_when_done` on so the exhaustion is C's leashed walk, and
  turns `borg_trip_policy` off (the trip's own preparedness estimate would
  ascend a birth character) and keeps the shipped town plan: its food
  counting counts an aware ration stack; the earlier draft's failure came
  from unaware fabricated stacks and a zero-value duplicate ration stack
  crushed as junk (#1179). `harness/borg.txt` is untouched.
- Review round (Codex review of `c811d38f` vs `track-p` at `7bf58639`):
  one P2 finding — standing on the very down stair the dive chose, the
  dive's flow has no step to queue (cost 0) and fell into the new
  fallback, walking the Borg off the staircase toward an up stair. Fixed
  in `borg_leave_level`: after the failed flow, an exhausted think takes
  the stair underfoot with the guards of caution's own stair take
  (`recall_restricts`, `borg_prep_leave_level_spells`, then `>`) instead
  of falling to the fallbacks. A new fixture phase fabricates the Borg on
  a `FEAT_MORE` grid with `stair_more` clear and a reachable up stair and
  expects `>`; it fails without the take (the fallback queues the walk
  off, key `4`). The down stair's grid feat is now fabricated too (the
  tracked stair's grid is that feat in a real game).

### How it was checked

- CI's commands: `cargo fmt --check -p borg-reborn-rs`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --exclude borg-core` (all green),
  `cargo test --release -p borg-core` (911 passed),
  `python3 scripts/check-param-citations.py` (2522 parameters, 0 wrong).
- `LeaveLevelUpstairsTest` passes on the test build and fails without the
  fix (checked by rebuilding with the fallback gate disabled: the incident
  think queues no up-stair flow) and without the review fix (the new
  standing-on-the-down-stair phase walks off toward the up stair, key `4`).
- Smoke (default borg-rs core, shipped settings): 4/4 identical to
  `harness/reference/smoke-borg-rs` (keys, messages, states) — no
  re-record.
- Save replay (a copy of `.angband-reborn/save/Borg261124105138`, sha256
  `210df0ee38ceb66e…`, loaded into this build at its pause turn 182,257,
  3,700 turns): the character resumes and leaves the level upward within
  ~490 turns (`# Trip: return via stairs … → # Going to stairs up. I'm
  bored.` at 182,657 → `# Leaving via up stairs.` at 182,747), with no
  multi-hundred-decision search loop. Note (RULES 26): the replay does NOT
  reproduce the recorded 521-decision loop itself — a load rebuilds the
  Borg's working memory, so the reloaded level is not "exhausted" in its
  map and the escape goes through the pre-existing trip-policy return; the
  loop's exact decision state is what the regression fabricates, and it is
  the regression that fails without the fix.

### What is left

- The zoo comparison of the fix's play effect is not requested by the
  issue; filed as a request: #1175.
- Findings from the same investigation recorded as P9 follow-ups: #1177
  (`more_refused` skips the descent gates when no up staircase is seen),
  #1178 (stale `stair_more` set before its flows), #1176
  (`note_standing`'s on-up/dn-stair flags are never true) and #1179
  (the decision-fixture cookbook).

### For the owner to decide

- Nothing in the task; the PR waits for review per RULES 1a.

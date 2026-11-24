# Decision fixtures: the recipe

How to build a **decision fixture** — a test that drives the shipped core's
dungeon decision (`borg_think_dungeon`) on a fabricated state and asserts the
decision. The working example is `#1173`/`#1174`
(`leave-level-upstairs`): everything below was learned building it and its
review round. Use this recipe before writing the next fixture, so the traps
are not rediscovered one build at a time.

## The pieces

| Piece | Where |
| --- | --- |
| The native fixture (state fabrication, phases, assertions) | `#[cfg(feature = "test-stair-handoff")]` in `borg-core/src/borg_think_dungeon_util.rs` (`run_leave_level_regression`), dispatched by case name in `run_handoff_regression` (`borg-core/src/borg_flow_stairs.rs`) |
| The Python driver (engine launch, settings pins, pass/fail) | `engine/tests/test_stair_handoff.py`, one `unittest` class per fixture |
| The hook | `BORG_TEST_STAIR_HANDOFF=<case>` runs the case after `borg_update`, before the decision, in `borg_think.rs` |
| The build | `BORG_IMPL=rust BORG_TEST_FEATURES=test-stair-handoff flock /tmp/angband-validation.lock harness/build.sh` → `build/rust-test/game/angband` |
| CI | `.github/workflows/ci.yml` runs `python3 engine/tests/test_stair_handoff.py <TestClass>` per class |
| The shipped settings | `harness/borg.txt` is read by the driver and pinned per case; the fixture never edits it |

## The recipe

1. **Fabricate the level.** `borg_init_cave`, then fill the map with
   `FEAT_FLOOR` and place the features the decision reads. A staircase must
   exist **twice**: in the track the flows search (`st.stairs.less/more`) *and*
   as the grid's feat (`st.cave.grid_mut(...)`). The #1174 review round caught
   the missing half: a tracked stair whose grid is still floor failed the
   standing-take's `FEAT_MORE` check.
2. **Fabricate the pack from the real birth kit.** Blank the spare wearables
   (they would be wielded before the decision under test). Fabricated items
   must be `aware`/`ident = true` with a real `kind` — the notice counts only
   aware items, and an unaware ration stack is invisible to the food
   counting. Find kinds by scanning `engine::kind` for `tval`/`sval`
   (`st.item_val.sv_*` has the svals).
3. **Merge duplicate stacks.** A second, zero-value stack is crushed as junk
   mid-think. Put rations into the kit's own ration stack (`iqty += 6`)
   instead of a new stack.
4. **Reset the per-think state the synthetic start dirties.** The synthetic
   start ships `birth_force_descend` set — unset it in setup
   (`engine::option_set(c"birth_force_descend", false)`), and let the phase
   that tests forced descent set it itself. Clear the goals
   (`goal.type_0/rising/leaving/fleeing/less`), `stair_less`/`stair_more`,
   the leash, the exhaustion marks; pre-set `times_twitch = 2` so a failing
   think stops before the twitchy tail; advance `time.now`/`time.level`;
   clear kills, takes, the vein track, the danger map, and `borg_flow_clear`.
5. **Sentinel the power.** Mid-think item decisions read a stale
   `st.borg.power`; set the sentinel (`st.borg.power = i32::MAX / 2`) so no
   take-off/wear decision can hijack the decision under test.
6. **Pin the settings in the driver, in place.** Copy `harness/borg.txt` and
   `re.subn` each `borg_param_<name> = …` line to the value the case needs.
   Appending at the end of the file does nothing: a line after the file's
   formula section is not read as a setting. Two extra traps:
   - **Per-class settings override the base.** A fabricated Warrior reads
     `prepared_dl34_min_hp__warrior`, not the base line — pin both.
   - **Some settings are not `borg_param_*`.** `borg_trip_policy` (a plain
     setting) also needs an in-place replace; the trip's own preparedness
     estimate otherwise ascends a birth character before the decision under
     test ever runs.
7. **Drive the decision entry point.** Loop
   `crate::borg_think_dungeon::borg_think_dungeon(st)`, consuming one queued
   movement key per think as the next observation (`take_step`: the
   Borg's map is the fabricated cave, so the step always lands). Assert on
   the queued keys (`>` / `<`), the goal type, and the flags
   (`stair_less`, the exhaustion marks) — the observable decision.
8. **Prove the teeth.** Disable the fix under test (`if false && …` is
   enough), rebuild, run the driver, and watch the new case fail *with the
   unfixed decision*. Restore, rebuild, pass.
9. **End the process.** The fixture's last line is `std::process::exit(0)`:
   the fabricated observation must never be applied to the live engine's
   unrelated cave.
10. **Check the shipped play is untouched.** If the fix ships behind a
    parameter at its neutral setting (the P9 rule), the change must not
    alter shipped play: rebuild the plain `BORG_IMPL=rust` build and run
    `harness/suite.sh smoke` + `harness/diff` against
    `harness/reference/smoke-borg-rs` — 4/4 identical, no re-record. Then
    the standard CI commands (`cargo fmt --check -p borg-reborn-rs`,
    `cargo clippy --workspace --all-targets -- -D warnings`,
    `cargo test --workspace --exclude borg-core`,
    `cargo test --release -p borg-core`,
    `scripts/check-param-citations.py`).

## Worked example

`#1173` (`docs/handover/2026-11-24-1173-powerdive-upstairs.md`): the
incident phase fabricates the recorded decision state — an exhausted, fully
explored DL3, two reachable up stairs, no down stair, a dive-prone
birth-level Warrior — and walks the Borg to an up staircase and takes it
(`<`). Its review round added the standing-on-the-down-stair phase and the
grid-feat half of trap 1. The phases for the counterfactuals (down stair
keeps the dive; forced descent; an explicit retreat; an explicit leaving)
sit in the same fixture, each with its own setup, so one driver run covers
the decision and its neighbours.

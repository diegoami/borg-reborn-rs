## 2026-11-28: Release the empty lantern shell and stop chasing surplus lanterns (#1413)

### What was done

- `light_spare_lanterns` (shipped 1 in `harness/pborg.yaml`, compiled default 0,
  `docs/CALIBRATION.md`, RULES 28) adds two rules in `borg_light.rs`:
  - **Retention.** `surplus_light` now also names a plain empty lantern
    shell (`empty_lantern_shell`) while a clean lantern with fuel is worn.
    The shell is known, clean, not an artifact or ego, carries no inscription
    or unknown rune and is no brighter than the worn lantern. Every caller of
    `surplus_light_drop` sees it: the dungeon's `borg_drop_inferior_spare`
    (drops with `borg ignore`), the full-pack clearing, the town planner's
    spare drop and the Home disposal. A worn lantern that is empty, cursed or
    not a lantern keeps every shell.
  - **Acquisition.** `borg_flow_take` (both scans) skips a floor lantern when
    `lantern_fuel_on_hand` (worn lantern + clean carried lanterns + clean
    Flasks of Oil) is at least `fuel_goal` (the trip's turns or
    `town_light_reserve_turns`, scaled by `explore_bfs_light_percent`). Below
    the goal the walk stays, so a character that needs the fuel still takes it.
- Five unit tests (`borg_light::spare_lantern_tests`) cover the shell
  (plain, special, worn light empty/cursed/torch), the fuel count at the
  issue's numbers (4,824 + 15,000 + 7,500 = 27,324) and the goal comparison.

### What was found

Read from `track-p` at `4101f6ec` and the Cemetery evidence (PR 167 in
borg-cemetery), not replayed:

- A refuel from a lantern stack splits one copy off and empties it
  (`engine/angband/src/cmd-obj.c:1008`, `refill_lamp`). That is the 0-turn
  shell at turn 889,425, when the worn lantern fell under 1,000 turns
  (`light_maintenance`). No flask was preferred then, so the stack was the
  donor.
- Nothing released the shell: `surplus_light` returns early unless the worn
  light is permanent; the gear plan ranks permanent lights alone; the junk
  path keeps a 35-gold lantern above 15 gold at DL under 10 and sees no power
  gain from its removal at any depth (no weight or slot term); the cursed
  spare path is for cursed lights. The town planner counts the shell as zero
  fuel and would drop it on a visit; the Druid had not been to town since
  turn 442,642.
- Acquisition had no fuel test: `borg_new_take` values a floor lantern at its
  cost, and a lantern walked over is picked up by `pickup_always`. For a
  lantern wearer the trait counting (`BI_AFUEL`) credits only flasks, so spare
  lanterns neither raise nor lower borg power.
- At capture the stock (27,324 turns) is above the 20,000 reserve and below
  the 30,000 goal that the shipped 150 percent breadth-first scaling gives.
  The Druid's third floor pickup (turn 876,626) came with roughly 17,000 to
  25,000 turns on hand (the worn lantern had about 2,300 left, two full
  lanterns, and perhaps the flask), still under 30,000, so the new rule would
  still have taken it. What changes is the shell, and any lantern picked up
  once the carried fuel reaches 30,000.

### Review of #1415 (OpenAI Codex, GPT-6, on `b32394c2`)

- P1: the new helpers moved `light_hallway_repeat_guard`'s read; its registry
  citation and the new switch's are updated (`scripts/check-param-citations.py`).
- P2: the fuel count credited every donor turn. The engine caps the lamp at
  15,000 and empties the donor (`refill_lamp`), and the Borg refuels below
  1,000 turns, so a copy now adds at most 15,000 - 999 turns
  (`lantern_fuel_on_hand`, with a boundary test: 900 worn and two 15,000-turn
  donors count 28,902, not 30,900).
- P3: the dungeon drop ignored the shell when `junk_gear_alternatives` was 0 or
  `junk_exploration_first` off. `borg_drop_inferior_spare` and the full-pack
  clearing now take the shell first (`borg_light::lantern_shell_slot`), under
  the unchanged safe-drop check. Not covered by a test: it needs a game.

### How it was checked, and not

- `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace --exclude borg-core`
  and `cargo test --release -p borg-core` pass.
- Smoke at the shipped settings equals `smoke-borg-rs` (4/4 identical), so no
  re-record. Those games stay on DL1-2 and never carry a lantern, so this
  checks only that nothing else moved.
- Not checked: an engine game that carries a lantern stack, refuels from it
  and drops the shell; the exact-turn stair replays of
  `engine/tests/test_stair_handoff.py`; the effect on survival or depth,
  which is not measured (RULES 14), the owner asking for a zoo arm
  (`light_spare_lanterns` 0 against 1, RULES 20) when wanted.

### What is left, and owner decisions

- The floor object's runes are not read (fair information), so an ego or
  Everburning lantern is skipped like a plain one while the fuel goal is met.
  Reading `fully_known` would spare such a lantern, but whether a plain
  lantern counts as fully known on the floor was not established, so the
  rule could then never fire. The owner decides whether to accept this or ask
  for a check in a game.
- Full spares above the goal are not released once carried (only not chased).
  Rule 27's curve above the reserve argues for keeping them; an issue can
  ask for a drop above a higher cap if the pack slots matter.
- The refuel donor is unchanged (a flask first, then the first lantern with
  fuel). Choosing the lowest-fuel donor would leave a bigger spare but still
  a shell.
- Separate leads from the same capture, not touched: the lantern was bought
  for 1,638 gold at shop 6, and the worn lantern swapped with a spare about
  every 50,000 game turns (turns 476,675 to 859,393).

# borg-rs and borg-reborn-rs, layer by layer

*A companion to [REDESIGN.md](REDESIGN.md) (#205). It compares how each
Borg works, at borg-rs `4ed3263` and this repository `e9a6a521`. Every
borg-rs reference is to `borg-rs/src/` at that pin. Every
borg-reborn-rs reference is to `borg-reborn-rs/src/`.*

## Summary

| | borg-rs | borg-reborn-rs |
|---|---|---|
| origin | the C Borg ported line by line (c2rust lineage), verified byte for byte against C up to v10; its own family since | designed here from the C Borg's lessons, the manuals and the engine |
| size | 68,887 lines of logic, 65 modules mirroring the C files | 15,000 lines, 18 modules |
| state | one `BorgState` of 30+ per-C-file partitions (`state.rs:46`), passed as `&mut` everywhere | `World` (memory), `Knowledge` (static data), `Params` (numbers), `FirstStrategy` (modes and goals), passed by `&` where they are only read |
| perception | the screen, the message line and the Borg's own memory, plus engine reads ("cheats") | the engine's own structures, every read listed (docs/ENGINE-READS.md) and checked against the engine (T2.5) |
| decision | a fixed priority list: 96 `if borg_x(st) { return true; }` checks in `borg_think_dungeon` (`borg_think_dungeon.rs:1316`, 1,077 lines) | modes chosen by priority rules with goals, budgets and cooldowns (`strategy.rs:choose_mode`), then a step per mode |
| `unsafe` | 279 occurrences, `static mut` in 21 files | 28, all in `engine.rs` |
| tests | 122 unit tests; parity by key-stream regression | 152 unit tests, decision fixtures from deaths (docs/FIXTURES.md), key-stream references (`harness/reference/`) |

## The control loop

Both keep the C Borg's `generate_keypress` shape, which the engine's
`inkey_hack` calls:
- check death and the user's key;
- answer a prompt;
- pop a queued key;
- otherwise think.

- **borg-rs**: `borg.rs:408 generate_keypress`, via `borg_entry_point`
  (`borg.rs:296`). A think pushes keys onto a queue (`borg_io.rs:192
  borg_keypress`, `:246 borg_keypresses`) and returns `true`. A think that
  queues nothing falls through to the next check.
- **borg-reborn-rs**: `borg.rs:337 next_key`. `think` returns a `Decision`:
  - one `Action`;
  - the mode, goal and rule that chose it;
  - the candidates;
  - the threat.

  The executor turns the action into keys and records its expected effect,
  which the next think checks. A think that queues nothing is a defect
  (ARCHITECTURE.md *The control loop*).

**Consequence for the pilot:** the loops are compatible. borg-rs's think
can run behind our `next_key`, provided its key queue is drained through
our executor or passed through unchanged.

## Perception (L1) and memory (L2)

- **borg-rs** rebuilds its map from **the screen** every frame
  (`borg_update.rs:307`, "Update the map based on visual info on the
  screen") and the message line (`borg_messages.rs`). It keeps:
  - its own grid record (`CaveState`);
  - its monster and object lists (`KillState`, `TakeState`);
  - the panel it has seen.

  Its view is what the screen shows plus what it remembers. That view is
  partial, it can be wrong, and its play depends on those errors. It also
  reads some engine state directly, as the C Borg does ("cheats").
- **borg-reborn-rs** reads the engine's structures each think
  (`engine.rs perceive`) into a typed `Observation`:
  - the status;
  - the inventory and equipment;
  - the panel's grids;
  - the monsters with their real race;
  - the stores.

  `World::update` folds it into memory. The observation is serialisable and
  replays into the same `World` without the game (`world.rs` replay tests).

**Consequence:** this is the main obstacle to option A. Our observation
cannot reproduce borg-rs's view, because borg-rs's view includes the
screen's misreadings. Under option C, borg-rs keeps its own perception,
and ours runs beside it for the trace and the errands.

## Knowledge (L3)

- **borg-rs**: `borg_init.rs`, `borg_formulas*.rs`, `borg_trait*.rs`:
  - the C Borg's tables and its `borg.txt` configuration;
  - the formulas file;
  - the **trait vector** (`BI_*` indexes, `borg_trait.rs`). This is a flat
    array of a few hundred numbers (resistances, counts of cures, escapes,
    weights) recomputed from the inventory each think and read by every
    decision.
- **borg-reborn-rs**: `knowledge.rs` reads the engine's own tables (monster
  races, object kinds, spells, stores) once. Derived facts are methods on
  `World` (`world.rs`: `ammo_tval`, `melee_per_turn`, `shot`) and functions
  in `equipment.rs` and `readiness.rs`.

**Consequence:** the trait vector is borg-rs's real interface between
perception and decision. An errand or a moved subsystem reads and writes
it; it is the natural seam for the pilot.

## Assessment (L4): danger and paths

- **Danger.**
  - **borg-rs**: `borg_danger.rs:2528 borg_danger(y, x, c, average, full)`
    sums each known monster's expected damage over `c` turns into one
    integer, from `borg_danger_one_kill` (`:1993`). It covers melee and
    spells, with the C Borg's per-blow tables. `borg_caution.rs` compares
    that integer with `avoidance` (about the current HP) to flee, rest,
    phase or teleport.
  - **borg-reborn-rs**: `danger.rs assess` returns an `Assessment`:
    - expected damage and worst turn;
    - `p_death`;
    - `disabling`, `summoning` and `economic` threat;
    - a per-monster breakdown.

    `tactics.rs composite` weighs it with one risk aversion. Its first
    version lifted the C Borg's danger sum (M3).
- **Paths.**
  - **borg-rs**: `borg_flow.rs` keeps one cost array (`FlowState`), seeded
    by a goal kind (`GOAL_KILL`, `GOAL_TAKE`, `GOAL_DARK`, `GOAL_FLEE`,
    `GOAL_VAULT`) and committed (`borg_flow_commit`, `:722`).
    `borg_flow_old` (`:1307`) continues a committed goal across thinks.
    Danger enters as an avoidance test per grid.
  - **borg-reborn-rs**: `path.rs` builds per-think distance maps per goal
    (`Goal::Stairs`, `Frontier`, `Grids`), with threat costs and barred
    grids (#201). No map outlives its think.

## Strategy (L5)

- **borg-rs**: the priority list of `borg_think_dungeon`:
  - crucial goals (light, caution, recovery);
  - the return to town;
  - spells;
  - wearing;
  - attack (`borg_fight_attack.rs:4406 borg_attack`, which **simulates
    every attack type**, melee, missiles, spells, wands, and keeps the best
    damage);
  - objects;
  - inventory upkeep;
  - leaving the level;
  - exploration;
  - shops;
  - waiting for recall.

  Each check can commit a flow goal and return. Equipment and shopping
  maximise one scalar, `borg_power` (`borg_power.rs:1831`). `borg_think_store`
  buys or sells whatever raises power the most. Depth is decided by
  `borg_prepared` (kit and level gates per depth).
- **borg-reborn-rs**: `strategy.rs`:
  - modes: Recover, Engage, Town, Descend, Explore, Eat, Return, Stuck;
  - `choose_mode` picks one by priority rules, with hysteresis (a live goal
    keeps its mode), budgets in game turns and cooldowns for failed modes;
  - `mode_step` acts;
  - fights go to `tactics.rs responses`, which scores every response
    (attack, shoot, throw, cast, quaff, read, step away) on one scale:
    risk removed against cost;
  - the depth is the trip's target (#201).

**Consequence:** borg-rs has far more decision coverage. Its 96 checks
cover vaults, glyphs, anti-summon corridors, enchanting, recharging,
decursing and munchkin starts. borg-reborn-rs has a clearer frame (one
mode, one goal, one budget, one trace line per think), so a strategy can
be swapped one layer at a time. Errands need that frame. borg-rs's goal
flags (`borg.rs:157-185`) are the closest equivalent it has.

## Executor (L6)

- **borg-rs**: keys are pushed by each think function in place. Prompts
  are answered by `borg_inkey` checks and the message reactions
  (`borg_messages_react.rs`).
- **borg-reborn-rs**: `executor.rs` maps an `Action` to its key sequence.
  The prompt catcher answers prompts by rule. Each action's expected effect
  is checked by the next think, and a mismatch is a trace violation and a
  failed goal.

## Cross-cutting

| | borg-rs | borg-reborn-rs |
|---|---|---|
| parameters | `borg.txt` settings and the formulas file | `params.rs` table: unit, range, default, evidence per parameter; profiles hashed into every record |
| logging | `borg_log.rs` notes to a log, the C Borg's style | `trace.rs`: one JSON line per think (mode, goal, rule, candidates, threat, violations) |
| determinism | the Borg's own RNG; key-stream regression | the same, plus `harness/diff` references in CI and the exact-boundary contract |
| deaths | death workflow and replay tools | a dossier per death (RULES 12), reconciled in CI |
| measurement | its own runs and the zoo | the zoo contract (ZOO-INTERFACE.md) and `tools/progress.py` |

## What this means for each option

- **A (re-express here):** the decision layers map one to one onto ours.
  Perception and the trait vector do not, so the port needs a borg-rs-view
  adapter.
- **B (refactor in place):** the target structure is the right column of
  this document. The first steps would be:
  - the trait vector behind a typed struct;
  - `borg_think_dungeon`'s list behind a mode-and-goal frame.
- **C (shell, then migrate):** the seam is borg-rs's `generate_keypress`
  (keys out) and the trait vector plus goal flags (errands in). Our
  observation, trace and executor run beside it.

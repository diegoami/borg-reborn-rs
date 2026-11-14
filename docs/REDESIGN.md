# The repivot: borg-rs's play in this repository's domain

*Status: an evaluation for the owner's decision (2026-11-14, #205). No code
follows from it until the owner chooses. Figures are measured at the pins
of 2026-11-14: borg-rs `4ed3263` (main) and its carry branches; this
repository `e9a6a521`; upstream Angband `e473720`.*

## Why

The proof of concept is over. Starting from nothing, this repository's Borg
reached, by #201 (PR #202):

- a trip loop (town, target depth, explore, home);
- 13 of 45 alive on the full suite;
- a mean max depth of 1.67;
- a mean character level of 3.98;
- no run at DL5.

borg-rs, the faithful port of the C Borg and its continuation, plays far
better. A better Borg built from first principles is not in reach (owner,
2026-11-14).

What the POC did build is a **domain**:

- an engine boundary with a single `unsafe` module;
- a typed `Observation` read from the engine;
- a `World` and a `Knowledge` rebuilt from it;
- a threat model;
- per-think paths;
- a strategy of modes, goals and budgets;
- an executor with a prompt catcher;
- a decision trace;
- a parameter table;
- a harness whose suites diff runs key for key, and the zoo contract.

The question is whether borg-rs's *play* can be re-expressed in that domain:
so that strategies can be changed and tried one layer at a time, and so that
borg-rs can be driven by **errands** (shop, clear a level, dive to a depth,
play until scared). The second question is whether Angband itself should be
rebuilt in Rust.

## What borg-rs is

| | borg-rs | this repository |
|---|---|---|
| purpose | the C Borg ported to Rust, verified byte for byte against C; since v10, its own family of Borgs (borg-rs#25) | a ground-up Borg; POC finished |
| Rust lines (logic) | 68,887, plus 5,508 of c2rust headers | 15,000 (plus generated bindings) |
| `unsafe` | 279 occurrences; 21 files with `static mut` | 28, all in `engine.rs` |
| state | one `BorgState` (traits array, grids, items, goals) plus globals | `World`, `Knowledge`, `Params` passed by reference |
| perception | the screen (`borg_update.rs`), the message line, plus engine reads | the engine's own structures (`engine.rs`), checked against the engine (T2.5) |
| engine | upstream `e473720` plus documented fork corrections (FORK-CORRECTIONS.md) | upstream `e473720` byte for byte plus our harness patches |
| change | carries of better-borg-c milestones (c-m2 to c-m10, v11R), then its own family work since v10; 300 commits across branches since 2026-11-01 | — |

borg-rs's logic, by subsystem (lines):

| subsystem | lines | here it would be |
|---|---|---|
| items, power, town, home (`borg_item_*`, `borg_power`, `borg_home_*`, `borg_store_*`, `borg_junk`, `borg_prepared`) | 14,559 | `equipment`, `readiness`, the town plan |
| fight and magic (`borg_fight_*`, `borg_attack_munchkin`, `borg_magic*`) | 10,432 | `tactics`, `spells` |
| flow (`borg_flow*`) | 9,121 | `path`, the strategy's goals |
| knowledge (`borg_formulas*`, `borg_trait*`, `borg_init`, `borg_projection`) | 8,947 | `knowledge`, derived `World` facts |
| perception (`borg_update`, `borg_cave*`, `borg_messages*`, `borg_item_analyze`, `borg_store`, `borg_io`) | 7,581 | `observation`, `world` |
| engine boundary, state, glue | 7,430 | `engine`, `borg` |
| danger and caution (`borg_danger`, `borg_caution`, `borg_escape`, `borg_recover`) | 6,508 | `danger`, `tactics` |
| think (`borg_think*`) | 4,309 | `strategy` |

## The options

### A. Re-express borg-rs here, verified against borg-rs

Port subsystem by subsystem into the domain, with borg-rs at a pinned
commit as the **oracle**:

- the same seeds;
- the same engine (borg-rs's fork corrections become patches in our
  series, or the parity runs use borg-rs's engine);
- the same key stream.

The harness already diffs key streams, messages and state logs (`harness/diff`).

Key-stream parity alone is too coarse to debug 69,000 lines. The port needs
a **shadow check per think**: our `World`'s derived facts compared field by
field with borg-rs's `BorgState`, and our decision with borg-rs's, both
running in the same process on the same observation.

- **For:** a clean domain, `unsafe` confined to the boundary, every layer
  testable on its own.
- **Against:** the largest effort. It also has the hardest property to keep:
  **perception must produce borg-rs's values, not the engine's truth.** The
  C Borg reads the screen and its own memory. It misreads, forgets and
  guesses, and its play depends on those errors. Our observation is the
  engine's truth. Every place where borg-rs's view differs from the truth is
  a divergence to model on purpose (a "borg-rs view" adapter), or the
  parity is lost.
- **Carrying changes:** every later change in borg-rs is a line-level patch
  to code we no longer have in that shape. Each must be translated by hand,
  then re-verified. The cost grows with every subsystem moved.

### B. Refactor borg-rs in place

Move borg-rs's own code toward the domain inside borg-rs:

- `BorgState` split into `World`, `Knowledge` and `Params`;
- `static mut` removed;
- the think functions behind a `Strategy` trait;
- the key queue behind an executor;
- every step verified by borg-rs's own key-stream regression suites.

- **For:** parity at every commit, by construction. borg-rs's changes keep
  landing in the same codebase, so there is nothing to carry. No second
  perception to reconcile.
- **Against:** the work happens in borg-rs, not here. This repository's
  domain is used as a design, not as code. Refactoring a live codebase
  conflicts with its family work, so the two must be sequenced.

### C. Our shell around borg-rs's core, then migrate inward

This repository keeps what it does better than borg-rs:

- the engine boundary;
- the harness, the zoo contract and `tools/progress.py`;
- the decision trace;
- the executor and its prompt catcher;
- the town sessions (#191);
- **errands** (below).

borg-rs's decision core is vendored as a module behind one interface: an
observation in, an action (key sequence) out. Subsystems then move from the
core into the domain one at a time, each behind the shadow check of option A.
A subsystem moves only when a strategy experiment needs it.

- **For:** borg-rs's play from the first day. Errands and experiments start
  at once, and the port is paid only where it buys something. Until a
  subsystem moves, carrying borg-rs's changes is a plain patch.
- **Against:** two styles in one tree for as long as the migration runs. It
  also reverses SPEC §2 ("no part depends on borg-rs"), which the owner
  must change. borg-rs's globals must be made safe to run beside our state:
  one Borg per process, which is already the case.

### Recommendation

**C, starting with a pilot**, because it delivers the two things asked for
(borg-rs's strength and errands) before any port is paid. **B** is the
better choice if borg-rs's family work can pause for the refactor. **A** in
full is not recommended: it pays the whole port before any benefit, and its
perception requirement (borg-rs's view, not the truth) works against the
domain it is meant to use.

**The pilot (one milestone), to decide on numbers:**

1. Vendor borg-rs's core behind the observation-in, keys-out interface. The
   full suite must reproduce borg-rs's key streams at the pin, byte for byte.
2. Implement two errands on top (below), for example *dive to N* and *shop*.
3. Move one subsystem, **danger and caution** (6,508 lines), into the domain
   behind the shadow check. Measure:
   - the lines and the time it took;
   - the divergences found and their causes, perception above all;
   - the cost of carrying one real borg-rs change across the moved code.
4. Report the measurements. The owner decides whether to continue, switch to
   B, or stop.

## Errands

An errand is a **goal with a termination predicate and a budget**:

- the Borg plays under its normal rules;
- the errand changes what the top-level think aims for;
- it stops, and hands control back, when the predicate holds or the budget
  is spent.

| errand | aim | ends when |
|---|---|---|
| **shop** (a list, or the plan of #196) | town: buy and sell the given wares, or what the plan chooses | bought, out of gold, or nothing affordable |
| **clear level** | explore the whole level and kill what it can beat | no frontier, and no beatable monster known |
| **dive to N** | take the fastest staircases down, avoid fights, keep the escape reserve | at depth N, or the danger rule says leave |
| **play until scared** | normal play | danger above a threshold, HP below one, or an unknown unique in view |
| **go home** | recall or stairs to town | in town |
| **identify the pack** | use, sell or read unknown items in safety | no unknown item left |

**How they plug in.**

- *In borg-rs:* the C Borg already has the switches an errand needs. Its
  goal struct (borg-rs `borg.rs:157-185`) holds `rising`, `leaving`,
  `fleeing`, `fleeing_to_town`, `recalling`, `descending`, and the store
  target `shop`, `ware` and `item`. The store think and the
  `borg_think_dungeon` order of checks complete it.

  An errand sets these switches and adds its stop predicate before each
  think. It
  needs a small hook in the core: "before think" and "after think".
- *Here:* an errand is a `Mode` with its own goal and budget, as Return and
  Town already are.

**How they are called.**

- From the game: the Borg's `^Z` command menu, one key per errand plus a
  parameter prompt (*dive to* asks for N).
- From a program: the query surface's play mode (docs/QUERY-SURFACE.md,
  `borg-mcp`), as `errand(name, args) → outcome`.
- From the harness: `run.sh` flags, so the zoo can run an errand as an arm
  (*dive to 5, then play until scared*).

**What errands buy:**

- **Tests** that start from a state and an errand instead of a whole game.
- **Experiments** that change one decision family (shopping, diving) and
  keep the rest.
- **Human sessions** that hand the Borg a job instead of a game. #191's
  town handover is the first.
- **Diagnoses** by the query surface ("dive to 3 from this save: what
  happens?").

## Rebuilding Angband in Rust

**Size.**

- 193,506 lines of C in `src/`, 332 files, without the Borg's 64,147;
- 14,969 lines of headers;
- 53,527 lines of game data (`lib/gamedata/*.txt`), which a port would read
  unchanged.

That is about three times borg-rs.

**What it would buy:**

- the whole game in safe Rust, with no FFI boundary;
- a deterministic simulation in one process:
  - the game state can be **cloned**, for lookahead, counterfactual branches
    (QUERY-SURFACE Q3) and search-based strategies;
  - it runs without a terminal and in parallel, far faster than a full
    game with a frontend.

**What it would cost:**

- **Parity with the C engine.** Every formula, every random draw in the same
  order, every message. Without it no result compares with the zoo's
  baselines, the C Borg's or borg-rs's. The engine is the yardstick. A
  Rust engine that differs by one RNG call is a different game.
- **A second engine to keep current** with upstream Angband releases.
- **The UI.** The X11 frontend, the screen the C Borg reads, and the
  savefile format. All are needed by borg-rs's perception, which reads the
  screen.

**The cheaper routes to what it would buy:**

- **Cloning:** `fork()` the C engine process at a decision point. The child
  plays the branch, the parent continues. This works with the engine
  unchanged and is the natural backend for QUERY-SURFACE's `branch`.
  Savefiles are a slower alternative.
- **Speed:** the headless `borgh` frontend, which the harness already uses.
  Parallel runs are already the suite runner.

**Recommendation:** not now. Revisit only if lookahead or search becomes
the strategy, and `fork()`-based branching proves too slow. Even then, the
scope would be a simulation kernel (the dungeon turn: monsters, projections,
the player's commands) checked against the C engine, not the whole game.

## What it means for this repository

- **Kept:**
  - the engine integration;
  - the harness, the suites and the references;
  - the zoo contract and `tools/progress.py`;
  - the observation (for the shell and the trace);
  - the executor and its prompt catcher;
  - the decision trace;
  - the town sessions (#191).
- **Paused until the owner decides:** M5's remaining tasks. These are the
  town plan (#192–#198), the deaths (#203) and the last freezes (#204). Our
  own Borg stops being developed as a strategy of its own.
- **SPEC §2 and §6 change** under option C: borg-rs becomes a dependency
  (vendored at a pin), and the comparison arm is borg-rs at that pin.
- PLAN.md gets a new milestone for the pilot, after the owner's choice.

## Decisions for the owner

1. Option A, B or C (recommended: C, after the pilot).
2. The pilot's scope:
   - the two errands (proposed: *dive to N* and *shop*);
   - the subsystem to move (proposed: danger and caution).
3. Whether borg-rs's family work can pause, for option B, or for the
   pilot's parity runs at a fixed pin.
4. The SPEC §2 change (borg-rs vendored at a pin).
5. The Rust Angband: not now (recommended), or a scoped simulation-kernel
   study.

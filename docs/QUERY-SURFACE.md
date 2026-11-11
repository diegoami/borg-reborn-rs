# The query surface (draft)

*Architecture draft for the secondary goal Q of [GOALS.md](GOALS.md#secondary-goal-q-a-query-surface-for-language-models)
(owner, 2026-11-11, decision D12). Nothing here is built yet. The roadmap is
in [PLAN.md](../PLAN.md#secondary-track-q-the-query-surface). Where this
draft and GOALS differ, GOALS wins.*

## What it is

A server speaking the Model Context Protocol (MCP), `borg-mcp`, that lets a
language model ask questions about a game state using the Borg's own
judgement: how dangerous is this grid, what would the Borg do here and why,
what is the expected damage per turn of this weapon, what changes if the
character quaffs this potion. Each answer is computed by the same functions
the Borg plays with, so the model sees what the Borg sees, not a second
opinion written for the occasion.

Users: borg-agent and other agent players (knowledge, claims with
evidence), people reading a death dossier, and a model reviewing a strategy
change. **It is not a player.** No tool sends keys to the game.

## The decision

**Same repository, separate crate.** `borg-mcp` is a binary crate in this
Cargo workspace. It depends on `borg-reborn-rs` (as an `rlib`); the core
never depends on it.

```
 ┌───────────────────────────── workspace ─────────────────────────────┐
 │                                                                     │
 │  borg-reborn-rs (rlib + staticlib)       borg-mcp (bin)             │
 │  ┌───────────────────────────────┐       ┌────────────────────────┐ │
 │  │ L0..L6, trace, params         │       │ MCP transport (stdio)  │ │
 │  │                               │       │ tool registry, schemas │ │
 │  │ query  ◄── stable, read-only ─┼───────┤ sessions (one state    │ │
 │  │  (new)     facade over L2–L5  │       │   per loaded run)      │ │
 │  └───────────────────────────────┘       └────────────────────────┘ │
 │          ▲  linked into the game                  ▲                 │
 └──────────┼────────────────────────────────────────┼─────────────────┘
            │                                        │ JSON-RPC
      Angband + borgh                          a language model
```

Why this shape:

- **Same repository:** the predicates are the core's functions and change
  with every strategy PR. In the same workspace, `cargo test --workspace`
  fails as soon as a predicate changes shape, so the server cannot drift
  from the Borg it describes. A separate repository would answer with last
  month's Borg.
- **Separate crate:** the core is a `staticlib` linked into the C game and
  must stay small, synchronous and deterministic. The server needs a
  transport, JSON schemas and possibly an async runtime. None of that
  belongs in the game binary, and none of it may reach the zoo's build
  (docs/ZOO-INTERFACE.md).
- **Not in borg-agent:** borg-agent is a client, not the owner of the
  Borg's judgement. It connects to `borg-mcp` like any other model.

## The two modes

### Replay (Q1–Q3): no engine

A run already records what is needed, through the logs the core writes
when asked:

| Log | Variable | Content |
| --- | --- | --- |
| `observations.jsonl` | `BORG_OBSERVATION_LOG` | one `Observation` per think (T2.1) |
| `worlds.jsonl` | `BORG_WORLD_LOG` | the world's digest after each think (T2.2) |
| `knowledge.json` | `BORG_KNOWLEDGE_LOG` | the engine's static data (T2.3) |
| `trace.jsonl`, `params.json` | `BORG_RUN_DIR` | the decision trace and the parameter table (T2.4) |

The server loads a run directory (or a death dossier, or a decision
fixture) and rebuilds the state at any think by folding the observations
through `World::update`, as the M2 replay check already does. The world
digest checks the rebuild at every step. Replay mode is pure: no engine,
no C, no globals. One process can hold many runs at once, and CI can test
it without building the game.

**Strategy state.** `World` is rebuilt from observations alone, but the
strategy has its own state (mode, goal, budgets, `Memory`, the Borg's RNG).
Replay therefore also runs the strategy over every think, with the run's
seed and profile. The choice it makes must equal the trace's `choice` at
every think; a mismatch is a determinism bug and the session reports it,
never silenced. That turns every replay into a free determinism check
(RULES 10).

`harness/run.sh` gets an option to turn the logs on (`--record-state`).
Off by default, so measured runs and the zoo contract are unchanged.

### Live (Q4, optional): a game hosted by the server

`borg-mcp --live` links the engine like the harness build does and hosts
**one game per process** (the engine's globals allow no more). The Borg
plays; at a think the server may pause the game and answer queries on
the current state, then resume. Live mode exists for exploring situations
that no recording holds (start from a save, step to a turn). It never
lets the model choose an action, and a paused think resumes with the
decision the strategy made before the pause, so play is identical with
and without queries. A test checks that `keys.log` is the same with the
server attached and querying at every think as without it.

## The facade: `borg_reborn_rs::query`

A new module in the core, the only part of the core the server calls. It
is what makes the surface stable while the internals move.

```rust
/// One rebuilt moment of a game. Owns copies; borrows nothing from the engine.
pub struct State { world: World, knowledge: Arc<Knowledge>, params: Params,
                   strategy: StrategySnapshot, think: u64, turn: i64 }

impl State {
    pub fn danger_at(&self, x: i32, y: i32) -> DangerReport;
    pub fn assessment(&self) -> ThreatSummary;
    pub fn threats(&self) -> Vec<MonsterThreatReport>;
    pub fn can_attack(&self) -> bool;
    pub fn melee_per_turn(&self, item: Option<ItemRef>) -> Option<f64>;
    pub fn path(&self, to: (i32, i32)) -> Option<PathReport>;
    pub fn responses(&self) -> Vec<ResponseReport>;   // tactics, scored
    pub fn decide(&self) -> DecisionReport;            // on a clone
    pub fn what_if(&self, change: Hypothesis) -> DecisionReport;
    pub fn reserves(&self) -> ReserveReport;           // survival invariants
}
```

Rules for the facade:

1. **Read only.** Every method takes `&self`. Anything that runs the
   strategy runs it on a clone (GOALS principle 10). `State` is never the
   live Borg's state; it is a copy.
2. **Reports, not internals.** Methods return serialisable `*Report`
   types with units (hit points, game turns, probability) and the
   parameter values they used. Internal types can change; reports change
   only with a version bump.
3. **No new judgement.** A method may only call functions the Borg plays
   with. If a question needs a new computation, that computation goes into
   the core first, with its own task and tests, and is used by play or
   declared as query-only in its doc comment.
4. **Deterministic.** Same run, same think, same arguments, same answer.
   `decide` and `what_if` use a clone of the Borg's RNG at that think.
5. **Versioned.** `query::VERSION` is a semver string. Every report
   carries it, and the server advertises it in its MCP `serverInfo`.

`Hypothesis` is a closed enum, not free text: `Quaff(kind)`, `Read(kind)`,
`Wield(slot)`, `StepTo(x, y)`, `MonsterAt { race, x, y }`,
`Hp(i32)`, `Param { name, value }`. Each builds a modified `World` (or
`Params`) and calls `decide`. Values out of a parameter's declared range
are refused.

## The tools

MCP tools map one to one onto the facade, with a session handle:

| Tool | Arguments | Returns |
| --- | --- | --- |
| `load_run` | run directory, dossier or fixture path | `session`, thinks, turn range, profile hash, build identity |
| `seek` | `session`, `think` or `turn` | the state's summary (depth, HP, position, mode, goal) |
| `view` | `session`, radius | the remembered map around the character as text, with a legend |
| `danger_at` | `session`, x, y | expected and worst-case damage, monsters counted, the unit |
| `threats` | `session` | each monster: race, distance, speed, expected damage per player turn, confidence |
| `responses` | `session` | the tactical alternatives with their scores |
| `why` | `session` | the trace line at this think: candidates, choice, violations |
| `decide` | `session` | what the strategy decides now, re-run on a clone; equals `why` in replay |
| `what_if` | `session`, hypothesis | the decision under the hypothesis, beside the actual one |
| `reserves` | `session` | light, food, cure, escape, way home: held against the declared minimum |
| `params` | `session` | the parameter table: name, unit, range, value, evidence |
| `knowledge` | race, kind or class name | the static entry (spoiler data, as D3 allows) |

Resources: each loaded session exposes its `trace.jsonl` and `params.json`
as MCP resources, so a client can read them whole.

Every tool result carries `query_version`, `run` (build identity and
profile hash), `think` and `turn`, so a claim made from it can be cited and
checked later. That is the form borg-agent's claims need.

## What the server does not do

- No tool sends keys, and no tool changes the state a played game reads.
- No free-form code or expression evaluation.
- No network access by default: stdio transport. An HTTP transport, if
  added, binds to localhost.
- No sealed seeds: `load_run` refuses runs whose seed is in a sealed set
  (RULES 14).
- No statistics across runs. Comparisons are borg-zoo's (SPEC §6).

## Checks

- `cargo test --workspace` covers `query` and `borg-mcp`.
- **Replay determinism:** on the smoke references, `decide` at every think
  equals the trace's choice (a CI step from Q1, TQ1.3).
- **Facade purity:** a test runs every tool at every think of a recorded
  run and checks the rebuilt world digest is unchanged afterwards.
- **No effect on play:** the smoke, smoke-exact and full references
  reproduce key for key and trace for trace after each Q PR, and
  `cargo tree -p borg-reborn-rs` (the staticlib's dependencies) is the same
  with and without `borg-mcp` in the workspace; in Q4, `keys.log` is the
  same with and without live queries. The build's source hash
  (`harness/identity.sh`) is not the check: it covers all of
  `borg-reborn-rs/` and `Cargo.lock`, so adding `query` or the workspace
  member changes it. With play unchanged, that change does not start a new
  measurement baseline (RULES 22).
- **Schema:** the tool schemas are generated from the report types and
  checked in (`borg-mcp/schema/`); a change to them is a visible diff.

## Open questions

- Transport library: a maintained Rust MCP SDK if one fits the
  no-async-in-core rule, or a small hand-written JSON-RPC over stdio (the
  protocol subset we need is small). Settle in Q2.
- How much of `Memory` the trace should carry so that seeking to a think
  does not need a replay from the start. Snapshot every N thinks is the
  likely answer (Q3).
- Whether `what_if` should also report a short look-ahead (the tactical
  layer's one step) or only the decision. Follows M4's look-ahead decision.
- Live mode's pause point: inside `think` (simplest) or between keys
  (finer, but the executor's queue complicates it). Q4.

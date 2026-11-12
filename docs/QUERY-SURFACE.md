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

**Its purpose is diagnosis** (owner, 2026-11-11): explaining the
model-free Borg's deaths after the fact, finding the think where each was
decided and the alternative that would have saved it, and turning that
into fixtures and issues for the strategy. That is how it improves the
Borg. Q0–Q4 serve it.

**Query mode** answers questions and never sends keys. **Play mode** (Q5,
optional, decided after Q4) would let a model play a hosted game,
choosing among the Borg's own actions while the executor turns them into
keys; a model-played game is a separate arm, never the measured Borg,
which stays model-free (RULES 24).

Users: models explaining deaths and making claims with evidence, people
reading a death dossier, models reviewing a strategy change; model
players only if Q5 is built (see *Workflows*).

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
- **Not in borg-agent:** borg-agent was a failed experiment (owner,
  2026-11-11) and is retired as a source. Its roles, model-played games
  and claims with evidence, are done here, where the executor, the engine
  host and the predicates already are. Any MCP client (Claude Code, an
  API harness, another vendor's client) connects to `borg-mcp`.

## The modes

### Query mode: replay (Q1–Q4), no engine

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

### Query mode: branch (Q3), what an alternative would have done

`what_if` says what the Borg *would decide* under a changed state; it
cannot say whether the character would have lived. `branch` can. It
reruns the run's seed, build and profile through `harness/run.sh`, which
reproduces the recorded game key for key (RULES 10), and at the chosen
think replaces the strategy's action with the alternative (an executor
`Action`), then lets the Borg continue for N game turns or until death,
and reports what followed: survival, HP, depth, and the trace from the
fork. With no alternative it must reproduce the original `keys.log`; that
is its own test.

The server does not link the engine: `branch` spawns a harness process,
built with the core's `branch` cargo feature (the forced action at a
think, like `test-hallucinate`), which is never a zoo build. The harness
builds that feature only for `branch` runs and marks it in `BUILD_ID`
(`features=branch`, as `test_features` is marked today), because the
source hash of `harness/identity.sh` cannot tell a feature build from the
measured one, and results stay with the build they ran on (RULES 22). A
new zoo-contract assertion, added in Q3, refuses a build whose `BUILD_ID`
carries features; today's `harness/zoo-check.py` builds the specimen
itself and has no such guard. A branch is **one sample**:
the game is deterministic, so a different action leads to one outcome,
not a probability. A claim says so, and a fix is still judged on the
zoo's populations (RULES 14).

### Play mode: live hosting (Q5, optional)

`borg-mcp --live` links the engine like the harness build does and hosts
**one game per process** (the engine's globals allow no more). The Borg
plays; at a think the server may pause the game and answer queries on
the current state, then resume. Live mode exists for exploring situations
that no recording holds (start from a save, step to a turn). It never
lets the model choose an action, and a paused think resumes with the
decision the strategy made before the pause, so play is identical with
and without queries. A test checks that `keys.log` is the same with the
server attached and querying at every think as without it.

### Play mode: a model plays the hosted game (Q5, optional)

Play mode is live mode in which the model, not the strategy, chooses at
the moments it is asked. It never works key by key: keys, prompts and
menus are the executor's (L6), and the model chooses at the level the
Borg chooses.

| Tool | What it does |
| --- | --- |
| `observe` | The character's view: remembered map around it, HP and status, threats, reserves; the same `State` reports as query mode |
| `options` | The Borg's own scored alternatives now (`responses`, and the strategy's `decide`), so the model sees what the Borg would do and why |
| `act` | Carries out one executor `Action` (Attack, StepTo, Quaff, Read, Eat, Wield, stairs, …); returns the expected effect and whether the next think saw it |
| `autopilot` | Lets the Borg play until a hand-off condition: risk above a threshold, HP below a fraction, a new unique or out-of-depth monster, a level change, a failed effect, or N game turns |
| `why`, `what_if` | As in query mode, on the live state |

Three ways to run it, each its own arm:

- **model**: the model chooses at every think (slow and costly; a ceiling
  study, not a default);
- **hybrid**: `autopilot` with hand-offs, the model choosing only at the
  moments that matter. It is the useful one: it is cheap, and every
  hand-off where the model's choice beats the Borg's is a candidate claim
  for the model-free Borg;
- **advised** (later, optional): the Borg plays, the model is only asked
  for a second opinion that is logged and not acted on, which measures
  disagreement without changing play.

**Run record.** Each model game writes the usual run directory plus
`model.json` (client, model, effort, hand-off conditions) and
`calls.jsonl` (every tool call and result, with think and turn). The
game's `keys.log` replays it key for key without the model, so query mode
and the death tools read it like any run. Deaths get dossiers under the
arm's own name (RULES 12). The arm is never `4.2.6-reborn-rust`; if the
zoo measures it, it is a separate specimen, and the zoo decides that.

**Fairness.** The tools read only through the facade, so the model sees
what the Borg sees (D3). Excluded modes (D5, RULES 15) are not offered as
actions.

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
| `branch` | `session`, alternative action (or none), game turns | the rerun from this think with the alternative: survived or not, HP, depth, the trace from the fork; with no alternative, the original run reproduced |
| `reserves` | `session` | light, food, cure, escape, way home: held against the declared minimum |
| `params` | `session` | the parameter table: name, unit, range, value, evidence |
| `knowledge` | race, kind or class name | the static entry (spoiler data, as D3 allows) |

Resources: each loaded session exposes its `trace.jsonl` and `params.json`
as MCP resources, so a client can read them whole.

Every tool result carries `query_version`, `run` (build identity and
profile hash), `think` and `turn`, so a claim made from it can be cited and
checked later: a claim with evidence.

## What the server does not do

- In query mode, no tool sends keys or changes the state a played game
  reads. In play mode, only `act` and `autopilot` move the game, through
  the executor; no tool sends raw keys.
- No free-form code or expression evaluation.
- No network access by default: stdio transport. An HTTP transport, if
  added, binds to localhost.
- No sealed seeds: `load_run` refuses runs whose seed is in a sealed set
  (RULES 14).
- No statistics across runs. Comparisons are borg-zoo's (SPEC §6).

## Workflows

What models do with the server, once it exists. Every output is a claim
that tests and the zoo check; nothing a model writes goes into the code
directly.

1. **Explaining a death** (query mode). `load_run` on a dossier, `seek` to
   the last few hundred thinks, walk back with `why`, `threats`,
   `danger_at` and `reserves` to the think where the outcome was decided
   (often well before the death), try alternatives there with `what_if`,
   and write a claim with evidence (run, think, turn, `query_version`, tool
   outputs, and the `branch` result for the alternative). If an
   alternative holds, draft a decision fixture
   (docs/FIXTURES.md) and an issue. A person or session reviews it; the
   fixture becomes a test; a strategy task fixes the decision; the zoo
   measures.
2. **Triage** (query mode). After a zoo batch, a cheap pass sorts deaths
   into cause buckets (unused cure, wrong escape, readiness, oscillation,
   starvation) and flags the unusual ones for workflow 1. Feeds the
   calibration report and the death index.
3. **Reviewing a strategy change** (query mode). Same seeds on the old and
   new Borg; at each think where `decide` differs, a model judges whether
   the change is better, worse or a side effect. Supports the milestone
   review; the reviewer is still the owner's choice (RULES 6).
4. **Hybrid play** (play mode, only if Q5 is built). The Borg plays with hand-offs; at each
   hand-off the model chooses. Hand-offs where the model's choice is
   better, judged by what followed and replayed with `what_if`, become
   claims for the model-free Borg, as in workflow 1.

**Which model.** The repository stays model agnostic and any MCP client
works. As a starting point: a strong agentic model at high effort for
workflows 1 and 4 (long tool use, reading traces, Angband reasoning), a
small cheap model for workflow 2 (batched), and for workflow 3 a model of
a different family from the one that wrote the change. A model's use is
validated before it is trusted: on deaths with known explanations
(`reference/deaths/` bbc and rs, and the cases in `docs/FIXTURES.md` and
REDESIGN-LESSONS), it must find the same deciding turn and the same cause
and alternative. Those dossiers hold no think-level trace, so the turn is
the finest ground truth they give; our own dossiers keep the last 200
thinks, which often stops short of the deciding think (Q0 measures how
often).

## Checks

- `cargo test --workspace` covers `query` and `borg-mcp`.
- **Replay determinism:** on the smoke references, `decide` at every think
  equals the trace's choice (a CI step from Q1, TQ1.3).
- **Facade purity:** a test runs every tool at every think of a recorded
  run and checks the rebuilt world digest is unchanged afterwards.
- **No effect on play:** the smoke, smoke-exact and full references
  reproduce key for key and trace for trace after each Q PR, and
  `cargo tree -p borg-reborn-rs` (the staticlib's dependencies) is the same
  with and without `borg-mcp` in the workspace; in Q5, `keys.log` is the
  same with and without live queries. The build's source hash
  (`harness/identity.sh`) is not the check: it covers all of
  `borg-reborn-rs/` and `Cargo.lock`, so adding `query` or the workspace
  member changes it. With play unchanged, that change does not start a new
  measurement baseline (RULES 22).
- **Branch:** `branch` with no alternative reproduces the original
  `keys.log` on the smoke references; a `branch` build carries
  `features=branch` in `BUILD_ID`, and the new zoo-contract assertion
  (Q3) refuses it.
- **Play replay:** every model game in the Q5 tests replays key for key
  from its `keys.log` without the model, and its `calls.jsonl` accounts
  for every action the executor took.
- **Arm separation:** a model game's run record names its arm and model;
  the zoo-contract assertion refuses it as the reborn specimen.
- **Schema:** the tool schemas are generated from the report types and
  checked in (`borg-mcp/schema/`); a change to them is a visible diff.

## Open questions

- Transport library: a maintained Rust MCP SDK if one fits the
  no-async-in-core rule, or a small hand-written JSON-RPC over stdio (the
  protocol subset we need is small). Settle in Q2.
- How much of `Memory` the trace should carry so that seeking to a think
  does not need a replay from the start. Snapshot every N thinks is the
  likely answer (Q3).
- Play mode's hand-off conditions and their defaults, and whether the
  hybrid's thresholds are parameters of the arm (a profile) or of the
  client. Q5.
- Whether `what_if` should also report a short look-ahead (the tactical
  layer's one step) or only the decision. Follows M4's look-ahead decision.
- Live mode's pause point: inside `think` (simplest) or between keys
  (finer, but the executor's queue complicates it). Q5.

# Architecture (draft)

*The design direction that [GOALS.md](GOALS.md) implies. A draft: each
part is a hypothesis that a milestone in [PLAN.md](../PLAN.md) builds and
checks. Revise this file when a milestone shows a part was wrong, and say
so in its review PR.*

## The shape

```
             angband engine (C, unchanged except harness hooks)
                 │  inkey_hack(flush)            ▲ keys
                 ▼                               │
 ┌──────────────────────────── substrate ───────────────────────────────┐
 │ L0 engine boundary   FFI reads, key return, RNG swap  (from borg-rs) │
 │ L1 perception        Observation: one immutable snapshot per think   │
 │ L2 world model       map memory, monster tracks, objects, character  │
 │ L3 knowledge         monster/item/spell tables, citations to C Borg  │
 │ L4 assessment        danger & risk, reserves, escape options, paths  │
 │ L6 executor          actions → key sequences; prompts; ESC discipline│
 │ ×  trace, params     decision trace per think; parameter table       │
 └───────────────────────────────┬──────────────────────────────────────┘
                                 │ &World, &Assessment, &Params
                     L5 strategy (trait)   ── one per family member
                         tactical layer: compare responses to a threat
                         strategic layer: modes and goals with budgets
                                 │ Action
                                 ▼
                              executor
```

Only L5 differs between members of the family. Everything above the line
is built once.

## The control loop (kept from the C Borg)

The engine calls the Borg when it wants a key (spec §2.2). We keep the
mechanism and its observable rules exactly, because the harness, the
launchers and replay depend on them:

1. Read the player's status; handle deactivation and death.
2. Read the message line; answer prompts that need an immediate key.
3. If a key is queued, return it. **Think only when the queue is empty.**
4. If the game is resting or repeating, return `EVT_NONE` (spec §2.7).
5. Otherwise swap in the Borg's own RNG, think once, restore the game's RNG.
6. A think that queues nothing is a defect. Unlike the C Borg, the reborn
   Borg logs it with the trace and falls back to a safe action. It does not
   stop silently.

A think is: `Observation ← perceive()`; `world.update(&Observation)`;
`assessment ← assess(&world)`; `action ← strategy.decide(&world,
&assessment, &params)`; `executor.queue(action)`; `trace.write(...)`.

## The layers

### L0 Engine boundary

borg-rs's `engine.rs` is the only `unsafe` code there, and it is already
proven against the pinned engine. We import it, keep it the only `unsafe`
module, and narrow it to the reads the C Borg makes (decision D3). A test
lists every engine symbol it touches. That list is the audit of D3.

### L1 Perception

It produces an `Observation`: the character's state and gear (direct reads),
the visible panel through `map_info()`, the message line and its parsed
messages, monsters seen this turn with their real race and condition, and
objects seen with their real kind. That is the same set as spec §1.1. It
holds plain data with no references into the engine, so it can be
serialised. That makes recorded observations replayable into the decision
layers without the game, which unit tests and the situation suite need.

**Check:** for the same game state, the reborn Borg's `Observation` agrees
with what borg-rs's `BorgState` holds after its update, on recorded runs.
borg-rs is the oracle for perception only, never for decisions.

### L2 World model

Map memory per level (the C Borg's grid record without the panel
artefacts), monster tracks (position, last seen, awake, hit points,
speed), object memory, store memory, and a character model (stats,
resistances, abilities, spells, blows, all derived from the gear).

Rules (GOALS principle 10): updated once per think from the `Observation`.
The strategy gets `&World`, never `&mut World`. "Try this item on"
simulations run on a cloned character model. The borrow checker enforces
what the C Borg could only intend (spec §5.4, §11.1, §11.6).

### L3 Knowledge

Static facts about the game: monster races, blows, spells, item kinds,
egos, artifacts, spells by class, stores. Read from the engine's parsed
data (`r_info`, `k_info`, …) at start, plus derived tables. Formulas lifted
from the C Borg (for example per-blow damage adjustments in
`borg_danger_physical()`) carry a `file:function:lines` citation at the pin
and a unit test with the C values. Later, reviewed borg-agent claims feed
in here.

### L4 Assessment

The core of the redesign, computed once per think:

- **Threat.** For each grid of interest and a horizon of h player turns:
  expected damage, a high quantile of damage (worst plausible), and the
  probability of death. It accounts for monster speed against player speed,
  distance and reach, wakefulness, line of sight for spells and breaths,
  resistances, and the turn order. The C Borg's danger formulas are the
  starting point for the per-monster terms. The output is in hit points
  and probabilities, so it can be compared with healing and escapes.
  **Lethal risk and economic risk are separate outputs.** Theft, item
  destruction and drains cost something, but they are not a reason to
  spend the last Phase Door. In the C Borg, gold theft adds to the same
  danger number that triggers escapes (better-borg-c#158; the gallant death
  D0002).
- **Calibration.** Predicted damage is checked against damage actually
  taken in recorded runs. A calibration report is a milestone deliverable,
  not an afterthought.
- **Reserves.** Turns of light, turns of food, healing on hand (in hit
  points), escapes on hand by kind (phase, teleport, teleport level,
  recall, stairs nearby), each with its failure rate.
- **Paths.** Distance maps (flow) with costs that include threat, to
  stairs, to unexplored areas, to targets. They are built from the world
  model and cached per think, never across thinks without invalidation.
  That stale-cache reuse is the mold death in rs D0006.

### L5 Strategy

A trait:

```rust
pub trait Strategy {
    fn name(&self) -> &'static str;
    fn params(&self) -> &ParamTable;           // declared parameters, defaults, ranges, units
    fn decide(&mut self, w: &World, a: &Assessment, ctx: &mut DecisionCtx) -> Action;
}
```

The first strategy has two layers:

- **Tactical layer** (runs when a threat is present). Generate every
  available response: attack each adjacent or reachable target, shoot,
  throw, cast, quaff, read, use a device, step to each neighbour, take the
  stair underfoot, escape by each means. Score each one by the risk *after*
  it (one-step look-ahead with the threat model) and by what it gains (a
  kill, distance, healing). Choose under explicit constraints, for example
  "probability of death next turn below p_max". It is one scale, so a cure
  in the pack is always considered (GOALS principle 5).
- **Strategic layer** (runs when no immediate threat applies). A small set
  of modes: *recover* (rest, cure), *engage* (hunt a target worth its
  risk), *loot*, *explore*, *descend*, *ascend*, *return to town*,
  *town* (sell, buy reserves, upgrade). A mode is chosen by utility and
  kept with hysteresis. Each goal has a budget in **game turns** and a
  progress measure, and a failed goal is blocked for a cooldown
  (principle 6). *Descend* asks the **readiness policy**: may this
  character be at depth d, given class, level, resistances, reserves, speed
  and kit? Every route to depth goes through it: stairs, trapdoors, Deep
  Descent, recall depth (principle 8).
- **Invariants** (checked first, every think): reserves below their minimum
  for this depth turn into goals (*return to town*, *buy*) before anything
  discretionary (principle 7).

All thresholds live in the strategy's `ParamTable`.

### L6 Executor

An `Action` (for example `Quaff(item)`, `Cast(spell, target)`,
`Walk(dir)`, `TakeStairs`, `Buy(store, item, n)`) becomes the complete key
sequence, with the pre-answered prompts and the closing `ESCAPE` (spec
§2.6, §9.3). The prompt catcher handles the rest (§4.6). The C Borg's key
sequences and prompt answers are knowledge to reuse (D4), with citations.
Each action has an expected effect. The next think checks it ("did the
potion get drunk?") and a mismatch goes into the trace, so silent failure
loops become visible.

### Cross-cutting

- **Parameters.** One table per strategy: name, unit (engine turns, game
  turns, hit points, probability), range, default, and the principle or
  evidence behind the default. Profiles are files read at start (borg-rs's
  `borg.txt` syntax for the settings both share). The profile's hash goes
  into every run record.
- **Decision trace.** One structured line per think: game turn, mode, goal
  and its budget, top candidate actions with their scores, the choice,
  invariant violations, the profile hash. Off the key path, so it never
  changes play. Death dossiers read it.
- **Determinism.** The Borg's RNG is the only randomness in decisions, and
  it is seeded from the run. There are no hash-map iteration orders in
  decisions (use ordered maps) and no wall time.
- **Performance budget.** A think must stay well under the time the zoo's
  wall limits allow. The baseline is about 110 s for 400,000 engine turns
  for the C Borg. The tactical look-ahead is bounded by design.

## What comes from where

| Part | Source | How |
| --- | --- | --- |
| Engine tree, `borgh`, harness, launchers | borg-rs | Imported (subtree and file copies with history), same names |
| L0 | borg-rs `engine.rs` | Reused, narrowed |
| L1 | borg-rs perception (`borg_update*`, `borg_item_analyze`) | Reused where it survives review against spec §11; otherwise rewritten against the same reads |
| L2, L4, L5 | new | Designed here; C Borg formulas cited where lifted |
| L3 | engine data + C Borg knowledge | Lifted with citations |
| L6 | C Borg key sequences and prompt answers | Lifted with citations, new structure |

## Open questions (to settle in the milestones named)

- How much of borg-rs's perception survives review, and whether its global
  `BorgState` can be cut cleanly at the `Observation` boundary (M2).
- The threat model's form: analytic (distributions from blow and spell
  tables) or sampled (Monte Carlo over a short horizon, with the Borg's own
  RNG). Analytic first; sampling only if calibration shows the need (M4).
- The tactical layer's look-ahead depth: one step plus "what next" for
  escapes may be enough (M4).
- Where class strategies first need different structure, as opposed to
  different parameters (M6, from data).

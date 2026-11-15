# Structured advice for difficult fights

Status: owner-requested design proposal, 2026-11-15. No model connection,
controller, new observation read or default-policy change is implemented by
this document. Implementation and any amendments to SPEC/RULES/GOALS require
owner review. This proposal extends [QUERY-SURFACE.md](QUERY-SURFACE.md),
issues #153/#212, and the simple profile/errand-plan proposal in issue #305 (PR #308): a plain level-phase plan embedded directly in borg.txt. The owner rejected
both the full scripting language and YAML; Borg profiles remain ordinary
borg.txt configurations with their own level schedules.

## What the player can request

A player should be able to ask for help preparing for a unique, obtain a
second opinion during an encounter, or delegate a bounded difficult fight.
The external model receives enough structured information to explain a plan;
the Borg remains responsible for validating and executing every action through
its own legal, observable capabilities. A useful answer can recommend equipment,
healing/escape stock, spells, positioning, engagement conditions and aborts.
It may also say to avoid the fight or that evidence is insufficient.

Three explicit modes avoid conflating advice with control:

| Mode | Effect |
| --- | --- |
| `observe-only` | Log a proposed alternative and reasons; do not affect play. This corresponds to Q's existing log-only “advised” concept. |
| `hybrid-advice` | Borg validates a bounded proposal and may adopt it; otherwise continues its native policy. This is a proposed extension. |
| `delegated-encounter` | External model selects from currently offered capabilities for a bounded encounter, with native safety and an immediate hand-back. Corresponds to bounded Q5 hybrid play. |

Preparation advice can be reviewed by the player and compiled into a local
plan for later deterministic use. Automatic adoption is a distinct mode,
never inferred from the presence of an API key or a model response.

## Preserve the existing model-free contract

RULES 24–25, SPEC and GOALS D12 keep model calls out of the ordinary measured
Borg. Do not add network clients or dependencies to the core, engine, standard
harness binary or ordinary zoo specimen. A separate opt-in host/controller
performs calls, taking snapshots through the versioned Q facade and returning
typed proposals through a reviewed command boundary.

An active adviser run has a distinct arm identity, for example
`hybrid-advice/<policy-version>`, with provider/model/effort and adoption policy
recorded. The normal model-free build rejects active adviser configuration
instead of silently acting or claiming to measure the ordinary Borg. An
observe-only analysis attached to a record must prove noninterference before
it can be described as leaving its key stream unchanged. These are proposed
acceptance conditions, not claims that a host or such build checks exist today.
No silent amendment of the current meaning of Q's `advised` mode is intended.

## Dependency graph and staged scope

| Delivery | Required accepted capability |
| --- | --- |
| Offline advice evaluation | Q0 evidence work (#153), known-explanation cases and cemetery source-qualified identity |
| Rich inspection | Q1/Q2 versioned read-only facade and fair state, knowledge and provenance reports |
| Active tactical proposals | Accepted R2 danger/escape/recovery and R3 combat interfaces, action receipts and effect-once execution |
| Equipment/stock advice | Accepted R4 item/spell/stock interfaces and inventory work #293 |
| Bounded plan adoption | Simple level-phase plans #305: checked errands, safe phase transitions, progress and interruption receipts; later parameter APIs require their own review |
| Live delegation | Accepted Q5 host boundaries and R1/R5 control/goal ownership; Q3 for counterfactual verification where available |
| Track B | Owner reference unresolved: current main defines P/R/Q, not B. Clarify before assigning implementation work. |

A closed R task is not enough: use its accepted milestone/interface version.
This is not authorization to bypass track reviews, advance R4 early, or start
live model play just because design documents were merged. If track B means
calibration P, include P's accepted registry/profile provenance and gate. If B
is a new behaviour track, its charter and ownership are an owner decision.
The planning PRs can be reviewed now without waiting for these implementations.

## Handoff triggers and control ownership

Inputs include explicit player requests and registered events: a newly seen
unique, a threat assessment exceeding a configured bound, repeated ineffective
actions, no acceptable native option, or a known difficult encounter profile.
Thresholds, cooldowns, request/turn/token/cost limits and encounter horizons are
configuration under review, not new empirical recommendations in this design.
Trigger events carry reasons and uncertainty; a native `fleeing` flag alone is
not a calibrated death probability. The recorded shopping incident (#293)
illustrates how a distant monster can set a scary flag with immediate danger 0.

Only one controller may own an encounter. At a safe game-command boundary:

1. Finish the previous multi-key action and its receipt. Native emergency
   action/player cancellation takes priority over making a request.
2. Freeze a consistent snapshot, effective parameter epoch, plan/controller
   state and current legal candidates. Assign request/state/encounter IDs.
3. In the default synchronous mode, pause game advancement while the external
   host asks the model. Network time must not advance engine turns or RNG.
4. Validate the response against the live boundary state and current authority;
   adopt atomically or record a rejection. Execute at most one game action,
   then refresh and revalidate any remaining proposal.
5. Recede to native control on an expired horizon, encounter end, level/town
   transition, missing capability, player intervention or abort condition.

A player stop cancels the request and proposal. If an asynchronous mode is
later added, the Borg continues natively and stale advice is normally rejected;
reusing it requires an explicit revalidation policy, not blind rebasing.
Wall-time timeout, provider errors and late responses are host events recorded
in the run. An unavailable service, malformed answer or exhausted budget falls
back to the precomputed native action after revalidation (or player handover
where requested). Retry budgets are bounded; do not freeze forever or ask the
same unresolved question every think. A timeout is not a game death.

## Information packet: rich, fair and explicit

“As much information as possible” means all relevant, available knowledge
with provenance, not omniscient engine/debug output. The host must expose:

- **Identity:** source/build/engine and catalogue versions; run/character,
  snapshot/think/turn, encounter ID, plan hash and effective configuration epoch.
- **Character:** HP/SP and maxima, speed/energy knowledge, status effects,
  resistances/immunities/vulnerabilities, equipment and known modifiers,
  carrying/slot constraints, usable spells/abilities with costs/failure chances.
- **Resources:** identified inventory, known charges/timeouts, ammo, healing
  and escapes, protected reserves, relevant home/store knowledge with age and
  actual accessibility. Inventory, shop stock and home stock remain distinct.
- **Opponent and context:** observed unique/race identity and uncertainty,
  visible/remembered positions, known HP estimate/status/speed, observed
  abilities and provenance-qualified lore; other monsters, summoners, known
  terrain/LOS, doors, corridors, hazards, stairs and escape routes.
- **Borg judgement:** native choice and alternatives with stable candidate IDs,
  legality/precondition results, danger components, relevant readiness/reserve
  rules, predicted effects and limits of those estimates. A danger score is
  not labelled a death probability.
- **History:** bounded recent actions, effects, messages, positional changes,
  failed attempts, prior advice/rejections and why this request was triggered.
- **Authority:** available action/plan/parameter schemas, supported spell/item
  selectors, immutable restrictions, budgets, current lease and fallback.

Separate observed facts, estimates and engine rules. Unknown item properties,
unseen enemies, exact hidden monster HP, unexplored terrain and future RNG
must not be supplied just because a recorder/debugger can read them. Recorder
`details` or omniscient death diagnostics are not automatically fair facade
inputs. Every newly exposed field needs the engine-read/fairness audit.
A rule/lore statement carries engine pin/source provenance; spoilers the
player's accepted knowledge policy excludes remain excluded.

Packet size is bounded. Send a mandatory critical-state envelope first, then
an indexed context bundle the adviser can query through read-only tools.
Prioritize current threats, escape failures and applicable capabilities over
old messages. Mark truncation/omissions and stale observations; if a critical
field is missing, ask for a lawful detail or abstain. Preserve exact serialized
packet/tool results, not only the final prompt or a later reconstructed state.

## Response contract and execution

The transport returns a versioned structured document, not keys or prose to
be executed. Illustrative shape (IDs and values are examples, not live schema):

```json
{
  "schema": "borg-advice@1",
  "request_id": "request-17",
  "state_id": "state-204",
  "encounter_id": "encounter-3",
  "config_epoch": 7,
  "decision": "propose",
  "scope": {"max_actions": 3, "max_game_turns": 50},
  "preconditions": [{"predicate": "candidate_available", "id": "escape-2"}],
  "proposal": {"kind": "choose_candidate", "id": "escape-2"},
  "abort_when": [{"predicate": "encounter_changed"}],
  "rationale": "Use the offered escape before spending the remaining healing",
  "evidence_refs": ["threats/3", "reserves/escape", "candidates/escape-2"]
}
```

`decision` is `propose`, `abstain`, or `request_details`. Supported proposal
variants are: one offered candidate; a named, reviewed errand plan/profile for
later execution; or, once a
separate tactical API is accepted, a bounded named tactic with checked arguments.
Preparation/stock/loadout proposals use those named capabilities rather than
inventing a script or editing profile files. The first profile format has no
arbitrary parameter patches or model-generated rule blocks. Alternative variants
are mutually exclusive and schema-checked.
The response may explain uncertainty/confidence, but a confidence number is
not evidence of legality or expected survival. Unknown fields/IDs and executable
text are rejected. Models cannot expand their own leases or authority.

Validation order: schema/version/size; request/run/character/state/encounter
binding; capability versions and epoch; current preconditions/knowledge;
resource and target existence; action legality; invariant/reserve/abort checks;
then controller horizon. Rejected proposals never partially mutate inventory,
parameters, plan frames or the game. Configuration suggestions are reviewed for a later plan; the first simple
phase format has no live parameter/profile setter. The adviser cannot add one
by returning a parameter patch or writing borg.txt. Later setters need separate
validation, atomic application and cache invalidation. Richer tactical controls need their own accepted
API rather than extensions that recreate the rejected full scripting language.

A reviewed multi-step tactic is still reconsidered each think. A target can die,
move or become unseen; a spell can fail, a wand can run out, and an escape can
be blocked. The executor owns prompts and expected-effect receipts; advice
cannot insist the next action succeeded. No raw key sequences, arbitrary Rust,
Python/shell, new engine reads or downloaded tactic modules are accepted from
the model. Model-supplied lore is a claim until checked against the pinned
knowledge service and actual state. Only the host's offered capabilities can
be exercised; the server does not invent a second danger calculator (RULES 25).

## Unique preparation and encounter play

A preparation request is different from a fight already in progress. It may
produce a proposed kit (what to carry or keep at home), a legal loadout, buffs
and spells to consider, an engagement gate, positioning rules and escape/
abort conditions. Review it as a plan before travelling: do not assume an
item can be bought while fighting, or drain protected stock opportunistically.

For a live unique, expose the native candidate set and accept one action or a
bounded tactic lease. Re-check line of sight, costs, failure chances, known
resistances and retreat availability after each receipt. “Cast spell X” is
invalid if this character has not learned it; “use item Y” is invalid if it is
at home; an immune/unidentified target cannot be assigned invented weaknesses.
A town retreat hands back to the strategic plan and invalidates the fight
lease. Re-engagement needs a new snapshot and requirements check. Advice to
relax a survival constraint is not an ordinary preference change; absent a
separately reviewed authority, it is rejected and recorded.

## Records, replay, and ownership

Record model/provider/version/effort and decoding settings, tool catalogue,
exact request/response bytes and hashes, external request IDs, timestamps,
latency, retries/cost usage, adopted/rejected decisions with reason codes,
configuration epochs, action receipts and controller transitions. Do not store
credentials or secret headers. Declare what gameplay data is sent to an
external provider and use explicit opt-in configuration; never infer consent
from another application's credentials or publish private recordings.

Live model output is not assumed repeatable from a seed, even with nominally
deterministic sampling. Offline replay consumes recorded advice and host events
through the same validator/executor without network calls, preserving order
relative to game decisions. Verify game inputs, effects and plan/parameter
transitions as well as native streams. Missing/truncated model records or a
changed state/capability binding make replay unverified. A native save alone
cannot reconstruct Borg/plan/controller memory: use a qualified joint checkpoint
or replay the lead-in. Branching from a save is a new labelled experiment.

The normal replay/recording primitive belongs to angband-recorder: report and
fix recorder defects there via PR; its consumer adapter/configuration stays
here. Source-qualified dossiers, curation and written fight investigations
belong in borg-cemetery, with explicit temporary validation registries allowed.
The live controller and typed strategy/executor contracts belong here. Network
credentials/provider adapters belong to the separate opt-in host. A recorded
model action sequence must never be relabelled as an independent model-free
Borg victory or silently added to the standard specimen/reference baseline.

## Delivery, tests, and owner decisions

1. Offline packet and proposal schemas, replayed difficult cases and blind
   diagnosis against known explanations (#153); no action adoption.
2. Observe-only adviser through Q, with noninterference and bounded payload
   tests. Measure explanation/actionability and record abstentions and errors.
3. Simulated/recorded acceptance through the capability/profile validators, then isolated
   counterfactuals (#212/Q3) where exact reconstruction is qualified.
4. Explicitly accepted opt-in Q5 host and hybrid-advice mode; bounded live
   handoffs, failure injection and full recorded replay qualification.
5. Reusable reviewed unique tactic packs; evaluate generalization separately
   from the encounters used to author them.

Tests include stale/duplicate/out-of-order advice, malformed/oversize responses,
invalid spell/item/target IDs, hidden-information leakage, changing epochs,
player cancellation, unavailable provider, timeout while paused, failed effects,
interrupted stock preparation, emergency override and fallback. Disabled and
observe-only paths must preserve the model-free key stream. Active modes must
replay without a model or network and preserve native/plan action receipts.
No advice is trusted merely because its syntax is valid.

For later zoo work, predeclare difficult-fight selection, baselines (native,
script-only, observe-only, active advice), horizons, cost/latency and improvement
endpoints, sample size and held-out cases. Count all aborts, timeouts, crashes,
service failures and deaths. Counterfactual branches/repeated seeds are paired
evidence, not independent lives. The owner starts population experiments;
no accuracy or survival benefit is claimed by this design.

Owner review must settle track B, accepted R/Q dependencies, trigger/lease and
fallback policies, fair packet contents, provider/data configuration, limits,
and any contract amendment for active advice. No implementation or new Q
milestone is started by these planning PRs, and neither proposal blocks the
model-free Borg's primary development.

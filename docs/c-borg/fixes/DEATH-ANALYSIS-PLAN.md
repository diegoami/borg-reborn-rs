# Bug-first native death and replay investigation

Owner direction,2026-11-06: carry verified fixes and investigate bugs first,
whether already carried or not. Death analysis supplies evidence and issues
when no higher-priority bug/carry work is ready. It is not an automatic
new-strategy queue; owner-directed improvements are allowed (2026-11-09) and
bugs keep priority.
Plan ownership: consumer13; live next work stays on checkpoint1/source207.

## Work order

1. Check live canonical C proposals, consumer regressions and source ownership.
   Carry a verified independent correction with unchanged before/after/control
   proof, native gate, provenance and frozen comparison. Already-carried fixes
   still receive regression/upstream-replacement attention. Uncarried real bugs
   still receive investigation; absence of a ready patch does not demote them.
2. A newly reproduced crash, stall, bad command, resource waste or harmful
   decision pre-empts exploratory death batches and discretionary infrastructure.
   Coordinate with the source worker rather than starting a second writer.
3. If no ready carry or focused bug investigation can progress, run a bounded
   death/failure collection pilot and produce diagnoses/issues before changes.
4. Merge verified work, measure and continue. Accumulate external-review
   milestones; do not stop each PR waiting for an independent reviewer.

At this plan's queue check, source110/PR264 is newly verified and not yet carried:
its preparation scoring queues an extra regeneration cast. The native evidence
shows one extra20-mana/100-action-energy cast, including regeneration before a
chosen haste. That carry takes priority over implementing this fallback plan.
Source26 is under active independent investigation; do not race it. These are
queue observations, not a hand-maintained live backlog or permanent readiness.

## Mandos ideas being adapted

Public references: [death replay29](https://github.com/diegoami/mandos/issues/29),
[replay equivalence54](https://github.com/diegoami/mandos/issues/54),
[stall dossiers56](https://github.com/diegoami/mandos/issues/56) and
[the roadmap](https://github.com/diegoami/mandos/blob/main/docs/plans/getting-stronger.md).
Useful method: record a life, replay to a short pre-death window, inspect several
nearby moments, diagnose against code, attribute, file an issue, fix, measure.
Stalls also deserve an evidence dossier. Exact engine/content/configuration and
intervention identity matter. One death is not reliable fitness evidence.

Those are workflow references, not proof that their TS/lab/save interfaces exist
in native C. No Mandos/Neo changes, private lab/profile access, live learning,
parameter evolution or strategy implementation are part of this native plan.

## Phase A: capture deaths and preserve evidence

Reuse the existing native runner, actual game/Borg interfaces, ordinary rules,
explicit RNG setup, forked snapshot writer and separate failure outcomes. Do not
build a dashboard or replay platform before one real case is trustworthy.

For each attempted life preserve a machine-readable record containing:

- Exact source/build/tool/binary/data hashes, character/race/class, options and
  Borg settings, seed/starting-save hash, limits, commands and interventions.
- Terminal outcome and raw native died_from text; terminal depth/max-depth,
  level, HP/SP and timed effects, engine turns, spent energy, Borg decisions and
  wall time. A derived cause category cites the native messages/state and may
  stay unknown; it never overwrites the raw cause or invents the responsible foe.
- Recent native messages, Borg decisions/queued actions, actual terminal/map
  state, known inventory/equipment/resources and relevant movement/escape facts.
  Keep observed facts separate from diagnostic/internal-state reads; nothing
  from analysis is fed into baseline decision inputs.
- Hashed initial and recent living saves, save turn/phase, save success and
  actual load-validation status. Retain representative saves and the pre-death
  message/decision tail. Never claim that a dead-character save resurrects the
  encounter: native saving omits dungeon state after death.
- Exit/signal, timeout/stall details and runner errors. A stalled/crashed/failed
  life remains in the denominator and retains its evidence; it is not a normal
  death or survivor. Diagnostic retirement is not a natural combat death.

The current observer already records much of this. Inventory what is missing
before adding the smallest declared observation. Any new counters, action logs
or frame capture must pass non-interference controls for decisions/RNG/rules.
Fork snapshots are required: native saving compacts monsters, so direct saves
in the live measurement process were rejected by an actual full-length control.

## Phase B: attempt a few turns before a real death

1. Start with one naturally occurring death from a bounded disposable run. Freeze
   its exact identity/conditions, including inputs and any intervention. Do not
   kill a character artificially to claim mortality evidence.
2. Prefer replay from the original recorded seed/start over assuming a mid-run
   save restores Borg memory. Replay the full unchanged life, fast-forwarding
   observation to the last window rather than changing gameplay or RNG draws.
3. Owner's wider default profile is death engine-turn minus5/12/25/40/60/82/105.
   Ten is omitted per the owner's correction. Preserve older5/10/15/20 windows
   as historical evidence rather than overwriting them. Add labeled event
   anchors earlier than105 when needed to inspect entry, stock or retreat.
   Report the actual observable boundary, phase and actual offset. Also retain
   the recent Borg decision tail. World turns, commands, energy and Borg-think
   calls are different units; do not label them interchangeably or manufacture
   distinct snapshots where several offsets share a boundary.
4. Compare trajectory/messages/actions, RNG state where meaningful, terminal
   outcome/cause and death turn. Repeat before calling the case exact replay.
   A mismatch is a recorded replay-fidelity/harness investigation, not discarded
   evidence or a reason to loosen matching assertions.
5. Separately load a recent living checkpoint and observe several ordinary
   turns. Savefiles omit full Borg memory, and save-time compaction can change
   restored monster order. Label this a diagnostic branch until equivalence is
   demonstrated. Starting saves loaded at birth have a separate established
   proof; that does not establish faithful mid-run continuation.
6. Keep the original death and replay untouched. Any later alternative-action
   experiment is a labelled branch with recorded input/conditions; it is not
   counted as the original run or evidence of population survival improvement.

If native save replay is insufficient, investigate a minimal native key/action
record-and-replay seam only after duplicate/ownership checks and qualification.
Do not begin full Borg-memory serialization, retained live processes or engine
save-format changes just to promise an instant rewind. A reproducible rerun with
focused tail capture is a useful first result.

## Phase C: diagnose, attribute and file issues first

Each dossier explains: situation → actual wrong action/information → measured
consequence → proposed corrected behavior. Then Verification gives pins,
copyable commands, actual output, causal mechanism, controls and limitations.
Read the opening without implementation detail; it should explain the problem.

- Separate a reproducible C defect, a consumer harness failure, a plausible
  design/strategy weakness and ordinary bad luck. A death alone proves none of
  those mechanisms. Missing heals, poor RNG or a losing encounter are evidence
  to examine, not automatic proof of bad code.
- Search canonical open/closed reports and check source207 ownership before
  filing. Reusable original-C discoveries go first to neo-angband-tracker with
  native evidence and source attribution. Coordinate a single source writer.
  Do not post to official Angband under the standing pause.
- Establish the source fix/proposal and native fail-before/pass-after/controls
  there. Then create a separate linked consumer adoption, import the reviewed
  C-only correction, keep its patch/removal unit and run product verification
  plus the same frozen comparison. Do not quietly fix the original only here.
- Consumer-only recording/replay defects belong here. Duplicate deaths attach
  to their canonical mechanism with additional evidence, rather than creating
  one issue per victim. Keep cause frequency distinct from proved bug impact.
- Improvement suggestions go first to issues with supporting death window,
  alternative explanation, confidence and a proposed experiment. Owner decision
  2026-11-09: approved improvements are then implemented here as labelled
  improvement units with evidence, gates and before/after measurement; they
  never go upstream and never masquerade as bug corrections. No automatic
  tuning or live learning.

Non-carried findings always receive a review (owner, 2026-11-09): an
investigation that does not produce a carried source fix — a diagnostic
observer, classification, dossier write-up, disposition, source-tracker issue
or borg-rs proposal — is reviewed before its conclusions become actions. Post a
review package issue (milestone-review pattern) with the evidence and exact
claims; the owner launches any reviewer they choose; the disposition lands only
after the verdict. Carried patches are unchanged: work-order item 4 still
merges them without waiting for a per-PR reviewer.

## Bounded execution and acceptance

The existing three-seed/20000-turn cohort remains immutable as regression v1;
its shallow all-survivor result is not a death-analysis dataset. After the ready
bug queue permits, time one bounded prospective pilot before scheduling a batch.
Define and freeze a separate exploration cohort/configuration before looking at
comparative outcomes. Include caster coverage rather than assuming Warrior-only
behavior generalises; first pilot scope and limits are recorded on the work issue.
No cap/seed/class enlargement to obtain a favorable fix result. Report every
attempt, including cap survivors and all infrastructure failures, not only deaths.

First useful deliverable: one real natural-death record, its validated or
explicitly divergent pre-death replay windows, a load-tested recent save where
supported, and a diagnosis routed to a canonical issue or a documented no-bug
finding. If the bounded pilot has no deaths, report that and choose the next
prospective batch transparently; do not invent a death case. Bugs discovered
at any step pre-empt the discretionary next phase.

Generated runs/saves/builds stay ignored. Keep persistent evidence and per-file
hashes outside disposable worktrees, publish concise machine-readable results
and representative facts on issues, and keep reproduction scripts/pins in Git.
Use the shared validation lock, small watched jobs, numerical gates and unchanged
controls. Preserve failures; no private profiles/lab/data, publisher/release/tag,
cleanup, amendment, force-push or source-worktree mutation is authorised.

## Owner-requested improvement evaluation process (2026-11-07)

[Death improvement workflow](DEATH-IMPROVEMENT-WORKFLOW.md) separates diagnosis
and candidate filing from implementation experiments and confirmation. It adds
no automatic strategy queue. Enhancements live here with source proposal links;
original-C defects retain canonical source/fix/carry routing. Survival and
long-term progress require prospective paired evidence, not one rescued death.

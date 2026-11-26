# R4 combined exit checkpoint (#438)

The accepted integration is `dc1491b9d31972406c43e71e358842f9b411df97`
(#780). This reporting task changes no gameplay. It reuses the exact accepted
store campaign, which contains the prior power, item-use and wear slices.
Reused runs and copied constructed states are not additional independent games.
The #438 source task is merged. The frozen milestone review #1031 found no
parity blockers; owner acceptance remains pending. Review corrections are
tracked in #1038 and do not move its immutable reviewed snapshot.
The recorded local build commit is not served by the source remote; measured
source equivalence rests on its tree hash
`46a4cb56f2fd4f3bdf207e0c574468ce2706f57d`, equal to the accepted
integration tree. Original build identities are retained without relabelling.

All three smoke arms and all three full45 arms match the retained accepted
wear baseline in keys, messages, state, core trace, endpoints and all 2,091
parameters. Both shadow modes pass the combined strict complete-entry and immutable-root
gate with zero mismatches. Those counters pool natural runs and a separate
fixture-enabled run; this is not complete natural-play entry coverage. The cumulative constructed suite has 55 item-use,
12 power, 26 equipment and 40 store cases per mode. Canonical reconciliation
accounts for all 12 retained native deaths as existing games; 2,734 historical
and canonical packages verify. Original identities and failed attempts stay
with the builds that produced them. The immutable combined report package is pinned by
[R438-EVIDENCE-RECEIPT.json](R438-EVIDENCE-RECEIPT.json) in
[Cemetery PR #114](https://github.com/diegoami/borg-cemetery/pull/114).
No new evidence belongs in docs/deaths.

## Owned scope and limits

R4-CODE-METRICS.json measures eight owned source files: 172 lexical production
function definitions and 11,109 physical lines (10,213 function-definition
lines). These include helpers and formatting, not 172 public decisions.
Power owns 11 bodies; wear/identify owns 37; store/home owns 87. The item-use
file has 37 definitions around its public entry surface. The activation-ID
initializer remains shared lookup data. The earlier roughly 9,000-line estimate
was a primary-surface estimate, not a complete dependency inventory.

The typed boundary preserves mutable state, output scores, complete item
arguments, native ordering and transaction timing. Source audits compare
accepted/native/owned bodies and reject literal drift, including matching
native/owned mutants with honestly declared seams. Runtime fault rejection
and repaired controls supplement the lexical proof. The separate literal-mutant
run exits zero in check mode while recording two mismatches; detection rests
on those mismatch records, not a nonzero game exit. Its driver incorrectly
expected a nonzero exit; that failed expectation is preserved in the artifact.

Engine facts, stock observations, notices, reserve/identify family policy,
M13 town_live/town_shadow/usefulness/utility/solver/packing/record/trip and
configured dynamic power formulas remain shared. Replaying an opaque service
is not independent ownership of its choices. Adjacent enchant/decurse remains
shared in this receiving baseline. The P contract #680 is complete on
track-p via #773; adoption of an owner-accepted promotion and independent R
ownership remain separate work, tracked by #1020. #293's replacement shopping plan is not delivered or closed.

Constructed purchase and withdrawal controls queue commands on copied memory;
the engine does not consume them. Positive queued actions are not proof of
successful purchase, retention, rejection recovery or natural branch coverage.
The inherited fixture provides separately labelled nested callback and Borg-RNG
once/replay controls; none adds independent games. Missing positive at-home
best-stuff coverage and constructed-only inscription UI coverage remain limits.
Opt-in shared M13 planner contexts demonstrate their declared parity boundary,
not independently ported planning. Invocation counts cannot prove every branch.
No survival or speed improvement is claimed.

## Costs and R5 reforecast

Source counts are reproducible with `python3 scripts/r4-code-metrics.py`.
The reporting task preserves per-process timings and failure lineage in the
prepared canonical package. Concurrent elapsed sums are neither CPU time nor
exclusive implementation effort. Shadow validation has substantial memory,
serialization and storage costs: whole item arguments and full-state copies
are explicit contributors; source counts do not measure their individual cost.

R5 needs its own source/dependency inventory and accepted parameter handoff.
The R4 experience invalidates treating a rough subsystem line count as a
complete effort estimate: shared services, source foldback proofs, positive
controls and canonical custody must be budgeted separately. Reserve a complete
fresh off/check/switch matrix and mutation/repair cycle for each runtime slice.
A numerical calendar or R2-relative implementation estimate is not supported
by the retained measurements; refine it at the R5 inventory gate. Do not start
R5 before owner acceptance of the immutable R4 milestone review.

After green task CI and owner-authorized integration into feature/track-r,
freeze review/R4-SHA into review/track-r-baseline and stop. Main and global
reviewed remain untouched. Implementation family: Codex/GPT-6. The owner
selects and runs the milestone reviewer.

## Milestone review coverage correction (#1038)

The frozen review pin's strict full45 gate pools 45 natural runs and one
fixture-enabled run. Natural smoke has no calls to several unknown-item,
store/home and identification entries and no root calls for
power.borg_power_equipment or store.town_strategy_disabled. The gate run on
natural smoke alone is not a complete-entry gate; the earlier smoke output
did not use --require-all-entries.

For the retained full45, the entries observed only in the fixture-enabled
run are item_use.borg_eat_food_any, item_use.borg_activate_dragon,
equipment.borg_object_fully_id, store.take_selected_for_sale and
store.sell_one_selected. Read-only roots power.borg_power_equipment and
store.town_strategy_disabled likewise have no natural full45 observations.
This says nothing about branches inside entries that do have natural calls.
The production tree has no caller of the town_strategy_disabled public facade.

check-shadow.py now accepts explicit --constructed-root annotations and
reports natural-run and fixture-enabled-run counts separately, along with
entries/roots observed only in the latter. A fixture-enabled process also
contains live harness calls; this partition is by run, not a synthetic-call
counter. The combined strict coverage requirement remains unchanged. New JSON
gate reports record mode, surface and strict/provenance options; historical
outputs and raw counters are not rewritten. Focused stateful-entries seed
coverage must be read from its own new CI report, not inferred from smoke or
full45. No additional game or natural sample is created by rechecking logs.

The item-use audit now runs in CI and checks accepted pre-seam/native/owned
correspondence for all 37 bodies. An honestly refreshed seam cannot bless a
matching native/port threshold mutation. The power audit preserves separated
compound-operator punctuation and rejects unsupported static/impl/macro
definitions. These are bounded source correspondences, not a general Rust
semantic proof or independent ownership of imported services.

The store fixture also calls shared log_buy. Although taped as an immutable
BorgState query, that service pushes process-global pending_buys and may write
the object log. Only the driving arm executes it; replay does not repeat it.
A synthetic pending buy can suppress an acquisition line later in that
fixture-enabled process. The original stubbed-dependency labels mention UI
notes and anti-loop effects but do not enumerate this logging effect. The
fixtures are neither fully process-isolated nor natural-play object-log
witnesses. This reporting/audit correction leaves their original rows and
implementation unchanged.

engine_consumption:false is descriptive fixture metadata, not an independent
measurement. The actual no-consumption argument is structural: cloned memories
receive queued commands, and the fixture never drains those queues into the
engine. Checking the literal flag alone proves none of that structure.

All retained R4 measurements use engine 42cd156ac. Any adoption onto a newer
main engine requires a separately authorized carry and fresh parity baseline;
R4's evidence does not establish equivalence on a different engine.

# R856: R4.5 fixed-main catch-up

Owner accepted R4 (#1093) and named receiving main
24be98732748230062882fe2b7d9601dba5e2cc0. Task branch
`task/T856-r45-main-carry` starts at accepted R4 40cbaf27, targets
feature/track-r and does not consume unpromoted track-p commits.

R45-PLAN.md and R45-PREFLIGHT.json record the fixed scope, source counts,
changed engine/donor/settings and required staged checks. The read-only merge
preview has 35 conflicts; no runtime or receiving merge is implemented yet.
Resolve provenance/parameter/native-shadow/tooling/archive conflicts without
discarding either track. Historical R4 receivers and all original evidence
stay fixed. The source carry is not a generic upstream re-vendor operation.

The exact selected main now builds in an isolated clean checkout with its
required right-to-left C argument evaluation. Native smoke passes 3/3 under
the shipped settings; this is a receiving control, not R4.5 shadow parity.
An initial smoke invocation completed all three games but failed in harness
postprocessing: the selected Cemetery toolkit did not match main's pin and
`bc` was absent from the invocation PATH. All original outputs and failed
statuses are retained. The pinned toolkit was checked out separately,
postprocessing recovered, and the complete suite repeated into a fresh output
directory successfully. Neither invocation had a natural death. These repeats
are the same three starts, not six independent games. Runtime evidence stays
local; no new reports or dossiers have been published.

`scripts/r45-receiving-surface.py` verifies R45-RECEIVING-SURFACE.json against
the exact Git objects. It inventories 708 definitions in the 14 existing
primary native modules, plus the seven new modules. These include private
helpers, methods and conditional definitions, not 708 independent decisions.
The receiving source interleaves test modules with later production policy;
splitting at the first cfg(test) would silently omit that policy. The bounded
extractor blanks complete test items while preserving offsets and subsequent
production. Unsupported conditions mentioning test fail closed; other cfg arms
remain visible, not evaluated. Regression tests cover this boundary.

Eleven old native entry signatures need explicit reconciliation. Besides
retired unknown-item actions, main retires borg_caution_phase and
sell_one_selected, expands confirm_purchase with quantity and paid inputs,
and changes item/home/quantity mutability or power visibility. The inventory
does not approve replacement semantics or expand R ownership automatically.
Next implement the receiving merge and owned policies, then update source
proofs, dispatch, typed dependencies, fixtures and coverage for that epoch.
Do not re-record historical seam manifests to bless a changed native source.

R5 stays paused. This task and the eventual R4.5 milestone need their own
review/CI gates. Do not confuse R4 acceptance, future receiving parity and
review-baseline integration. The current R4 review checkpoint CI was still
running when acceptance was recorded; preserve that distinction in metadata.

## Source-audit follow-up from the published preflight

Draft #1095 records a later local carry checkpoint at `334fce3e`. That
commit is absent from this checkout and GitHub did not serve it. This follow-up
starts from the published preflight `c61c0704`; apply it to the carry checkout
and rerun the audits there once the checkpoint is available.

The attack and defence audits now exclude complete test items and retain
production definitions that follow them. Defence excludes its named,
feature-gated harness fixture without dropping later policy. Regression
controls verify that both audits accept matching late production and reject
an omitted or changed body. At the selected main pin the old attack cutoff
omitted seven production functions, including `attack_on_sight` and
`borg_fight_speed`. The preflight, receiving inventory, both source audits
and all 67 source-script tests passed on the published preflight checkout.
These checks do not validate the unpublished receiving source or its gameplay.

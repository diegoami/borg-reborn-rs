# R4 item-use implementation (#435), validation in progress

Current local checkpoint: normal and test native builds succeed. Native
off/check smoke has all3 gameplay streams equal; switch smoke is still
running. The controlled unknown-item flag fault is rejected with a full-state
mismatch, and the exact repaired file again passes constructed check. Source
fault patch is R435-LITERAL-FAULT.patch; repaired switch/nesting/full45 and
canonical evidence remain pending.

Normal and test native builds succeed. All53
constructed cases pass in check and switch with zero disagreements and all24
item entries observed. All37 policy function bodies pass source/literal
correspondence, with fresh named reads and audited seam changes only. These
synthetic controls explicitly stub the UI note effect and do not prove
engine consumption, live UI callbacks or full45 parity. Full normal gameplay, nesting and canonical evidence are still pending. Historical
attempt notes below preserve the initial state and failures.

Owner accepted R3 #427 and opened R4. Preflight reference #429/#432 and
ownership #431/#439 target feature/track-r and must integrate first, with
green exact-head CI. Runtime donor remains1443c2d1; no newer main/P carry.
#433 gates complete power/wear/store policy; this slice is item-use only.

All24 public item-use entries and private policy bodies are in an owned
module behind typed native/port facades. Native and port request the same
closed inventory/spell/IO/engine dependencies. Internal calls preserve the
ordered policy markers; external R2/R3 dependencies suspend the outer tape,
independently compare the item entry, then outer replay consumes recorded
results/state. No command/log/RNG effect is replayed live. New API delta is
three typed functions: kind-effect predicate, no-light fact and object log.
ActivationIds initialization remains shared engine-derived knowledge.

Runtime telemetry is shadow/shadow-stats@4 with surface r4-item-use and
item_use_calls. The checker requires --r4-item-use for this exact43-entry
surface. Historical default@3 and explicit R2@1/attack@2 readers remain;
they cannot accept new@4 evidence or satisfy its strict entry gate.

The test-only test-r4-item-use feature has53 constructed cases: all24 empty
entries,22 positive selections/queues, formula/recharge/dragon-denominator
cases and4 present-but-unavailable cases. These are hypothetical policy
memories with queued commands on copies, not actual engine consumptions or
natural trials. CI combines these test-build entries with the ordinary
unsealed stateful seed for strict entry coverage while retaining separate
artifacts/death registries. Full native/check/switch45 and literal-fault/
real-nesting/custody acceptance remain required; no completed runtime parity
or merge is claimed by this handover.

Normal compile and clippy pass; feature compile initially exposed BorgItem's
lack of Default (fixed using its existing blank constructor), and feature
clippy flagged the explicit fixture helper's argument count (a narrow helper
annotation retains readable witnesses). Source tests initially exposed one
stale expected file count12; the two newly audited native/port files require14
in that nested-module case (actual surface13). Both failures are retained in
local logs. The corrected46 source tests and shadow-reader regressions pass.
No assertions/seeds/caps/engine rules were relaxed and no measured game has
run yet. Source inventory and hook bytes were explicitly re-recorded only for
this reviewed delta; donor pin/inventory and other source bytes stay fixed.

Next: finish native feature/normal builds under the shared flock, execute
and retain bounded constructed/smoke checks, fault/re-port and nesting; full
exact-baseline suites and death reconciliation; publish new original gameplay
evidence in cemetery with immutable receipts before final feature merge.
Preserve private build metadata and every attempt, distinguish public source
identity from local build Git rev, and do not reuse old R3 evidence as new
runtime validation. Keep #435 open/in progress until all gates pass. This
handover supplements the unchanged global historical file.

The first feature-built constructed run (source f861d36a, private8c5c51f5,
binary90a9abf4) aborts with rc134 after24 empty cases: the first positive UI
note attempted to lend a copied state. This is a retained test-driver failure,
not a natural death or evidence of normal-policy agreement. Original binary,
BUILD_ID and raw run are preserved under r435-evidence. The guard is unchanged.
The constructed driver now explicitly stubs only decision_api's shared
borg_note_bytes UI notification within a feature-only scoped flag. Ordered
requests, complete state replay, object-log and queue operations stay checked;
real note-internal mutation, engine UI lending and nested callbacks are not
covered by those synthetic cases. Production has no stub capability. Source
gate rejects policy access to the driver. Normal smoke/full/nesting must still
exercise the real service. No claim that this stubbed fixture alone proves
real UI behavior or successful engine consumption is permitted.

The isolated constructed attempt reaches all22 positive selection cases,
then fails its formula witness because that case forgot item awareness and
therefore correctly returned the native unavailable sentinel100. The expected
formula result stays-20; mark the constructed item aware to reach that branch.
Retain the failed raw run and isolated binary/BUILD_ID. The first isolated
build also had a module-path compile error, fixed through the API parent.
These are setup/control failures, not unexplained natural-game mismatches.
Parent reference CI's15 environment probes failed after the pinned zoo began
scrubbing ambient variables. The dedicated #429 correction separates explicit
native-process injection from adapter-isolation checks; all14 contract tests
pass against the exact accepted R3 binary without runtime changes. This task
inherits that parent correction; original failed workflows are retained.

Fault-control reproduction uses the test-r4-item-use feature, whose synthetic
UI notification stub is explicitly declared above. Apply R435-LITERAL-FAULT.patch,
build the test feature, run the constructed check context (seed1001 Human Mage,
cap1000, menu), then require harness/check-shadow.py --r4-item-use --expect-mode
check to reject the reported item_use.borg_quaff_unknown final-state mismatch.
Apply the inverse patch and rebuild before further validation. The exact owned
file SHA256 is f34fd95c9b616aa0fe5928aafdfefdcbdefa221a2f3cba71a37b3b00a457333d.
The private controlled-fault commits are not part of the published source
ancestry; final publication uses the correct tree atop the public audit base.
Actual normal build records git_rev469ae3bc and inputs_dirty=no, with runtime
input2efde248c719cdcaa31b28078af461823aee47d06fa283642cee6f6f7d5d7ff8.
Keep that build identity unchanged even when the public containing source SHA
has different commit metadata. No full45 or milestone acceptance is implied.

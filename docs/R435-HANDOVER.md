# R4 item-use implementation (#435): gameplay validation complete

Current runtime input is 95fb364b150ad87a8330b45441d08f252096bedc0c766754e5eb3608b3e30293.
Actual measured local commit04520f0238cd609aa3108c5da42a3aa8c53d4a08 has
exactly the same tree as public c7b2dd33944ddc8f00305d307dece00b848fe014.
Private literal-fault ancestry is not published; BUILD_ID remains unchanged.
All45 full-suite cases match the exact accepted M13 donor in off/check/switch,
including keys/messages/state, endpoints and gameplay settings. Both shadow
modes have zero mismatches. Normal smoke and original-core smoke complete.
All55 constructed cases pass in check/switch, and two independent literal
faults reject at mutable full-state and immutable-return boundaries. Exact
repair passes. Actual nested callbacks match (four in each mode). Constructed
coverage supplements the strict entry gate; it is not independent natural
coverage, successful engine command consumption or live UI-note validation.

Canonical evidence: borg-cemetery PR51, pinned by R435-CEMETERY-RECEIPT.json.
It preserves16 qualified packages:12 current native deaths, one original-core
smoke death and CI613/614/616 originals. It includes all comparisons, original
build identities, fault controls and the prepublication archive-gate failure.
Canonical D-<hash> sidecars and aliases are assigned by the cemetery owner
implementation, preserving all original package bytes. The source verifier
checks those additive identities using that exact pinned implementation,
then imports only the unchanged qualified original into the disposable mirror.
Old R364/R365 manifests remain byte-identical; later owner sidecars are
validated separately. No historical inventory or reconciler is weakened.

Full canonical reconciliation and exact final-head CI must pass before merge.
Task targets feature/track-r only. Owner accepted R3, not all of R4. Complete
power/wear/store slices remain gated on accepted P handoff433. Final R4 owner
review is438; main/global reviewed promotion remains unauthorized.

The following text records earlier source epochs, setup failures and controls.
Statements that validation is pending below are historical, not current status.

# Historical implementation and validation notes

Current branch refinement: the availability predicate takes only &BorgState
in both programs. Its typed ReadDecision uses ordered read-only replay on the
same immutable memory, with no caller-state clone. Mutable entries retain the
original full-state fork/comparison. The surrounding predicate dependency is
now a query: complete input-state replay checking remains, and the unchanged
post-state copy is unnecessary. Only audited immutable facts are called; the
compiler rejects writes or mutable services through this interface.

The current protocol is shadow/shadow-stats@5, surface r4-item-use-readonly.
readonly_comparisons/readonly_points count immutable roots; root_points counts
mutable roots. The historical stateful_points field contains all policy-entry
markers, including immutable ones. Total comparisons include both partitions;
item_use_calls still counts every item marker. --r4-item-use requires @5;
--legacy-r4-item-use explicitly verifies unchanged @4 prototype evidence.
Neither historical reader can satisfy the current gate. No metric is silently
reinterpreted. The constructed list now has55 cases, adding charged-item and
ignore-charge checks at the immutable public boundary.

Six tape unit tests pass, including ordered read/query replay once, wrong
return/order rejection, and restoration of an opaque parent transaction with
complete mutable-state equality. Source correspondence still covers all37
bodies, and47 source tests plus12 shadow gate tests pass. Current native
@5 validation is pending. The measured @4 smoke below identified2.58million
immutable calls among2.78million item calls; that count motivated the narrower
type. This is not a controlled performance claim. Historical @4 smoke/fault/
constructed results below remain with source2efde248 and do not validate the
new runtime by themselves.

## Historical @4 prototype and retained attempts


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

Native @5 at private613bcaa5 (public5933ba43) passes55 constructed cases in
both modes, all3 smoke cases off/check/switch with zero disagreements and
identical keys/messages/state, and smoke reconciliation (zero natural deaths).
The readonly map is correctly separate from mutable roots. This source's
input is04fbdaaedfea75cc5458658b3035a551a002a11486f3738934066c0bc6475244;
original normal/feature binaries and BUILD_ID are preserved. No speed claim.
CI43895543704 retained two real gates: the grid diagnostic unit still expected
historical schema@3 (now requires@5 with its other assertions unchanged), and
core-trace-off's seed1011 natural death was absent from canonical custody.
Its original11-file registry artifact14239302694 is downloaded byte-for-byte
(SHA25698f9f0bcc3e09faa3ea2e4cafe63df06b5cb80fa04158dfa5d4cf1cab7b3321b).
Do not skip its archive gate or relabel the death. The previous prototype CI
artifact is also retained separately. The diagnostic expectation is test-only,
but the whole-file input hash changes; rebuild and name subsequent evidence
honestly rather than rewrite old BUILD_ID or reuse its hash as the new build.

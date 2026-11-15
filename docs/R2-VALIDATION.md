# R2: fight-or-flee port and measured handoff

> Historical R dossier numbers in this report are scoped to registry commit
> `fbd0f691de655c6db61565b1e3d2b5aea92184cd`, not the restored legacy slots.
> See [R334-RECOVERY.md](R334-RECOVERY.md) and R334-DOSSIER-MAP.json for
> full qualified paths. Raw measurements/build identities remain historical.


R2 exit evidence is complete. Task PR #291 remains subject to its final CI
merge gate, followed by the feature-only milestone review. R3 is not started.

R2 ports danger, caution, escape and recovery into borg-next. The runtime
core remains borg-rs v12 e6d13fc. The engine remains 42cd156ac. All work
integrates into feature/track-r; neither main nor global reviewed receives R.

## What now decides

Danger owns physical/spell estimates, per-monster combination and square
aggregation over the typed view. Caution, recovery and nine public escape
entry points own their policy branches and mutations. Every BI read uses a
named view field; goal/time/item records retain native memory semantics.
The only direct BI write is the named mana-simulation adapter.

Check drives native policy; switch drives the port. The alternate program
runs on owned Rust state and recorded dependency responses. All 68 dependency
functions are typed; replay validates each request and input state before
applying a post-state. Return values, request exhaustion and complete final
state must agree. Engine effects and RNG draws execute once. A mismatch in
switch is a validation failure; no engine rollback is claimed.

STATEFUL-SHADOW.md explains nested transactions, internal markers,
panic=abort handling, access-token limits and the closed source gate.
Existing projection, spell/item actions, preparation, attack/defence and
flow helpers remain shared dependencies; zero does not independently
validate those later-milestone implementations.

## Validation

- Final combined input hash: cd66d935d4b981d06904b3fcb03c20177ede6576fd0c21745f5e9ddd8acfb36e.
- Main baseline: reviewed P promotion 0ed0e5de; cemetery tools 33c37552.
- Off/check/switch smoke is 3/3 identical in keys, messages and states, with
  positive danger/caution/escape/recovery coverage and zero mismatches.
- Full check and switch each match independent pinned v12 in all 45 games
  across keys/messages/states. Each mode has zero mismatches across
  66,900,787 whole-danger comparisons, 111,865 standalone-monster comparisons,
  378,351 stateful transactions and 15,032,802 tape events: 14,331,822
  dependency requests plus 700,980 policy-entry markers. Internal
  entry counts are 154,821 caution, 322,741 escape and 223,418 recovery.
  Check versus switch also passes the standard 45/45 same-build diff.
- Game RNG and all other shared game fields agree with v12; the existing
  frontend telemetry differences and wall time remain. Standard cross-repo
  summary equality remains 0/45. No survival or performance gain is claimed.
- Pre-sync 2b59c452 evidence is immutable in R2-PRE-SYNC-COMPARISON.json.
  Owned port/tape code is unchanged by the main sync; parameter descriptions,
  safe ranges and core selection were carried, then final gates rerun.
- Unit/CI: 170 workspace tests, 143 core tests (one existing ignored), 24
  source-tool and 82 tool tests, formatting, Clippy and vendoring pass.
  Final engine/recorder CI remains the merge gate.

The immutable `shadow-stats@1` evidence names its tape-event total
`dependency_requests`; that legacy field includes policy markers. The count
clarification in R2-COUNT-CLARIFICATION.json derives the corrected split from
both preserved reports, with their exact hashes. This is a reporting correction,
not a runtime change or a new game campaign.

## Non-vacuity and real carry

The surrounded-assessment reversal produces a durable request divergence
in both modes at turn 1751/Borg turn 202. The engine exits 2 before an
unrecorded dependency runs; harness exit 1 correctly represents that
controlled failure. Re-porting restores zero mismatches and identical
changed-native play. Recovery has a separate completed-return control:
six check and eight switch mismatches; its re-port also restores zero and
changed-native play. Both the request and return paths are demonstrated.

The focused real donor is dc790dc46e13, recovery's patch 0134: remove four
HEAL1/HEAL2 stun attempts and two Waybread blindness attempts, keeping the
named parameter mechanism. It builds; porting the same six removals builds
and passes both modes on Human Ranger 1074 and High-Elf Rogue 1063 fixtures.
Neither fixture exposes an actual disagreement before re-porting: this is
a negative control, not proof that the changed branches execute or that
v13 policy is fully supported. #264 still gates production adoption.

All 20 stateful diagnostic attempts are retained: 18 completed games, two
intentional request-divergence exits, no diagnostic deaths. Controller
receipt interpretation was corrected without rerunning or replacing the
completed diagnostic. Test features, exact builds, first mismatches,
statistics, hashes, queue waits and active build/run timings are in
R2-STATEFUL-CONTROLS.json. All eight patches retain the diagnostic source
changes. Their runtime port/tape blobs are unchanged by the main sync; default
values are unchanged, and final full gates validate the combined tree separately.
The earlier danger controls and their identities remain R2-DANGER-*.

Forced native nested entry passes check and switch with seven actual
engine re-entries per mode, zero mismatches and identical logs. The opaque
nested-transaction effect-once path is tested separately by the unit fixture;
the forced engine test does not claim to force that exact opaque path.

## Deaths and integration

Pre-sync full-suite deaths are D2389–D2396 (eight cases; switch repeats
check). Its original-core CI controls are D2397/D2398. Final-source full
cases are D2439–D2446, with CI controls D2447/D2448. Each full campaign
reconciles all 16 natural deaths (eight cases and their switch repeats). No natural death is
omitted, reclassified as a configuration failure, or counted as independent
merely because a source/arm was rebuilt.

Incoming P review records keep their original bytes and file hashes, with
feature-local IDs D2399–D2438 mapped in R2-MAIN-SYNC.json. P review prose
uses its original main namespace. Canonical cemetery data was not written
or migrated. Historical validation explicitly selects docs/deaths; normal
exports keep the cemetery-owned default. The thin replay pointer preserves
recorded BORG_SHADOW restoration and inherited-setting cleanup before
canonical CLI dispatch.

## Effort and the next milestones

Physical port source: danger 2,532; caution 2,292; escape 1,424; recovery
558: **6,806 lines**, including comments/license. Port dispatch adds 36;
core snapshot/API/tape/request glue adds 702, including its unit fixtures.
The first R2 port issue was opened at 11:11:25 UTC; the complete stateful
implementation commit was 12:17:03 UTC: a 65m38s elapsed envelope including
measurement, CI and bookkeeping, not exclusive coding time or throughput.
Main synchronization and its repeated validation extend the overall work.

Measured carry cost: six native call removals and six matching port removals,
no conflict or new trait for this focused unit. Control JSON separates actual
build/run time from shared-lock waits; the initial constructed build waited
373 seconds. None of those timings is a performance experiment.

Provisional effort forecasts, relative to this R2 migration rather than a
calendar promise: R3 1.5–2×, R4 2–3×, R5 1.5–2.5×. R3 adds larger attack
code and owned item/spell facts with valid lifetimes; R4 adds inventory/store
side effects; R5 adds path/cache ownership and the actual modes/goals design.
Reordering pure queries also needs an explicit extension of the current
conservative ordered replay contract. One migration is insufficient for a
reliable lines-per-week rate. The owner sets the next milestone after review.

The watcher now names 60 port functions and warns about five changed ports
at moving reference 727ccff. Source watching is not dependency analysis;
shared helper/constant changes still need manual inspection. No v13 runtime
adoption, population experiment, sealed-suite opening or R3 work occurred.

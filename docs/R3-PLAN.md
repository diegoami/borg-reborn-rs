# R3 preparation after accepted R2

R2 review #302 (Claude Opus 5.5) accepted the verified migration with
non-blocking follow-ups. The owner accepted and delegated its merge and
continued Track R. Tag r2 points at the feature-only review ledger; the
reviewed feature snapshot cee4c94 remains immutable. All tasks still target
feature/track-r. This plan implements review follow-up #316; it does not
claim that attack/defence or an accessor optimization is implemented.

## What R2 establishes, and what remains

R2 owns copies of danger/caution/escape/recovery with named typed reads and
an effect-once comparison boundary. It preserves native function structure,
conditions and temporary native memory mutations. It is a mechanical policy
migration, not a new tactical policy, independent perception or completed
modes design. A policy carry currently needs changes in native and port
copies. The measured recovery carry required six removals in each copy;
its fixtures did not expose the changed branches. Keep that limitation.

R3 extends that verified boundary to attack and defence. R4 handles item and
store ownership; R5 replaces path/cache ownership and the dungeon priority
list with explicit modes/goals. Changing tactical policy while establishing
parity would lose the comparison oracle: separate each redesign step and
measure any gameplay claim through the zoo rules. R6 remains optional.

## Typed access without eager whole-view reconstruction

The reviewer identified 384 View construction sites in stateful policy.
Each existing View::new eagerly copies all trait groups. This is a source
observation, not a runtime profile or proof of a performance bottleneck.
Danger already constructs one view for an assessment; keep its immutable
snapshot semantics unless measurements justify a separate change.

For stateful policy, implement a borrowed accessor that stores only
&BorgState, with named borrowed groups (CharacterRead, StatusRead, SuppliesRead,
etc.) and named scalar methods. Constructing the accessor/group performs
no bulk copy. A scalar method reads its documented BI index and returns
the original i32. Internal trait-index access belongs to the projection
implementation, never to policy modules. Generate mapping/accessor definitions
from one declaration, retaining exact BI provenance and the existing values.
Keep config/parameters, engine tables and item/spell facts separate.

A borrow must end before a mutable dependency call or simulation write.
Reconstruct a cheap borrow after that mutation. Do not cache a character
snapshot, a group or a scalar across an effect unless native policy itself
stores that value. Preserve deliberate native temporaries and the named mana
write. No interior mutability, unsafe lifetime extension, root engine token,
shared engine snapshot or global cache is permitted. Policies remain independent
of the engine API, with fresh named access to the same native memory.

Checks: all named fields match native BI indices; successive borrows observe
changed values; the compiler rejects a retained borrow across mutation; old
and new access produce the same decisions/state in check and switch. Inspect
that accessor construction does no eager group/array traversal. Make no speed
claim without a separate measured experiment. Update the syntactic source
gate narrowly for the new accessor path and keep manual dependency review.

## Ordered R3 implementation slices

1. Refresh moving references in a dedicated PR. Read changed helpers and
   new evidence; keep the v12 runtime pin unless a separately authorized
   production carry satisfies #264. P2 parameter prerequisites are already
   in the reviewed main baseline carried by R2. Synchronize newer main work
   when required, through a tested task PR preserving concurrent work.
2. Introduce the cheap named accessor and migrate existing stateful reads
   without moving expressions across effects or changing policy. Retain
   accepted R2 evidence as historical; validate the exact new runtime inputs
   and record every natural death. Include scalar-mapping/freshness/lifetime
   checks plus native smoke and full parity before declaring migration done.
3. Port attack's complete native public surface and internal policy into
   borg-next. The pinned attack file is 4,583 physical lines. Audit every
   engine/RNG call, returned borrowed fact and mutable simulation state.
   Extend typed query/effect requests and scalar comparison hooks as needed;
   never pass shared native policy results off as independently ported logic.
4. Port defence (3,592 physical lines) including all public helpers and
   attack/escape cross-calls. Attack plus defence is 8,175 physical lines at
   this baseline. borg_fight_perm is a separate 775-line shared dependency,
   not silently included in that estimate; porting it requires an explicit
   scope and named-parameter audit. State item/spell lifetimes and shared
   dependencies in the report. Preserve native simulation restoration.
5. Validate the combined exact build: off/check/switch smoke; check full
   with zero disagreements; switch full 45/45 v12 keys/messages/states;
   per-entry and rare-path fixtures; durable non-vacuity controls and matching
   re-ports. Repeat native nesting where adapters change. Keep engine/RNG
   effect-once guarantees and explicit telemetry/branch-coverage limits.
   Preserve all failures/deaths and CI artifacts. Open a pinned feature-only
   R3 milestone review and stop for the owner's chosen reviewer.

The current tape contract is ordered: even reordered pure requests disagree.
If R3 needs a different contract, design and test it explicitly rather than
silently accepting reordered requests or weakening state comparisons.

## Effort and scope limits

The provisional R3 effort remains 1.5–2 times the R2 migration, not a delivery
date or coding rate. The now-counted 8,175 policy lines exclude the accessor,
wrappers, facts, validation and reporting. Re-estimate after dependency audit.
No sealed suite, population run, hybrid/model arm, perception replacement or
promotion to main/global reviewed is authorized by this plan.

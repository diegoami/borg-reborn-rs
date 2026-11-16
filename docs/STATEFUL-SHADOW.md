# Stateful Track R comparisons (#286, #364)

The dependency direction stays shell → next → core. The core owns a typed
request enum for caution, recovery, all nine public escape entry points,
and five mutating attack entries. Two additional attack entries compare
scalar results without a stateful transaction.
With shadow off, each wrapper immediately calls its original implementation;
no state is copied, no arguments are formatted and no tape/files are created.

In check, the native program drives the real state and engine. In switch,
the port drives them. A switch disagreement is a validation failure; no
engine rollback is claimed. The other program runs on an owned Rust policy snapshot
and receives recorded responses. **The engine is never cloned or rewound.**

The R2 closed `decision_api` surface had 68 dependency functions; the attack
extension is inventoried in R364-ADDED-DEPENDENCIES.json. A stateful
call records its name, explicit arguments, complete input state, result and
complete post-state. Replay validates the request and input state *before*
applying that post-state; an earlier port mutation cannot be overwritten and
hidden. Read-only state queries also validate their input state. Engine
queries and RNG draws return their recorded values during replay. Replay
never falls back to a real dependency. Both complete programs must return
the same value, consume every request and finish with equal native state.

The snapshot derives Clone/PartialEq for the native state's owned value
types. Field-wise equality for keypress and elemental item info lives in
our separate snapshot module; generated engine bindings remain untouched.
An engine access token copied into a snapshot does not grant access:
engine::check_access still requires the actual root state's field address.
Stateful BI reads use borrowed named Read accessors; danger uses an immutable
View. Temporary policy mutations stay
explicit on the driving state or replay snapshot. The only direct BI write
is the named mana adapter, matching the native temporary-mana simulation.

An opaque native dependency runs with recording paused. If it re-enters a
ported policy or a nested Borg think, that inner transaction saves the
suspended tape and pause depth, performs its own comparison, then restores
them. The parent records the dependency's complete result once. The recorder
mutex is never held while a policy or engine dependency runs.

Internal ported entry markers compare function identity and arguments even
when escape is called inside caution. Statistics distinguish root stateful
comparisons, internal policy entries, individual entry points and tape events. In `shadow-stats@1`, the legacy field
`dependency_requests` includes the `policy::` entry markers. Subtract the sum
of `stateful_points` (equivalently caution_calls + escape_calls + recovery_calls)
to obtain actual dependency requests. The R2 full suite has 15,032,802 events,
700,980 markers and 14,331,822 actual requests per mode. Historical raw JSON
keeps its original schema/values; future telemetry must retain these explicit
semantics or version a changed contract. Positive coverage of caution, escape and recovery is required;
wall-helper comparisons cannot stand in for stateful comparisons.

Use `harness/check-shadow.py --require-all-entries RUN_OR_SUITE` to require
positive counts for all 11 public entries across the given runs, and verify
that their sum agrees with subsystem counts. That agreement is a file-integrity
check: both totals come from the same visits loop, so it is not independent
proof of coverage. The `stateful-entries` suite
isolates High-Elf Rogue 1063 at the existing 100,000-turn cap. It exercises
rare `borg_caution_teleport` as well as all ten other entries; CI requires
the strict gate separately for check and switch. Entry coverage does not
prove every branch, successful action or real-carry change executes. The
full R2 counts and focused fixture evidence are in R2-ENTRY-COVERAGE.json.

If a native carry moves the fixture's single teleport entry, keep the strict
flag and fail the gate. Re-run the existing unsealed full suite on the exact
new build, inspect per-entry counts, and select a replacement (or a second
seed) that actually reaches the rare entry in both modes. Record the source,
profile, seed, cap, all entry counts, outcomes and play-log hashes; rerun the
focused check/switch gate and diff before updating the fixture/evidence in a
reviewable PR. If no seed reaches it, add a declared targeted state fixture
that exercises real policy, not invented counters. Never drop the flag,
weaken assertions/caps/timeouts or use sealed seeds to regain green.

R3 attack uses `shadow-stats@2` and `shadow-mismatch@2`, with
`surface: r3-attack`. `tape_events` counts all recorded events,
`policy_markers` counts the entry markers, and `dependency_requests` counts
only actual dependency calls. The invariant is tape_events = policy_markers
+ dependency_requests. The stateful point map sums to policy_markers;
subsystem call counts include attack; their sum is policy_markers plus
pure_attack_comparisons because attack_calls also includes standalone
scalar calls. Root
transactions remain separate from nested entries. `pure_attack_comparisons`
and `pure_points` count the two standalone scalar entries; they do not add
tape markers. When a scalar entry executes inside a stateful transaction,
its marker and recorded queries belong to that transaction instead.

The default consumer requires the R3 surface and all 18 public entries when
`--require-all-entries` is set, including both standalone scalar entries.
`--legacy-r2` explicitly reads historical @1 evidence, derives the split
from its legacy counters, and requires the original 11 entries. It never
rewrites raw evidence. @1 cannot satisfy the current default gate. The R2
coverage and seed instructions above describe the historical acceptance;
the extended strict gate must pass on the actual attack build.

The workspace intentionally uses panic=abort. An incompatible replay request,
input state or result type writes a durable mismatch to shadow.jsonl, then
stops with exit 2 before an unrecorded dependency can run. It does not invent
a result for the rest of a divergent replay. A completed comparison with a
different return/final state logs the mismatch and retains the driving
policy's result. Diagnostic request-divergence runs count as controlled
operational failures, never deaths or successful games.

Invalid danger-grid memory uses the same controlled-stop principle: append a
`danger.grid` diagnostic with coordinates and Borg clock, flush it, then exit
2. No scalar is invented. The engine game turn is explicitly null because
this path queries no engine state; it never takes the recorder mutex. Missing
run directories or write errors are reported on stderr before exit. A six-case
subprocess fixture exercises the actual grid failure path for both modes,
covering direct, nested and replay contexts with out-of-bounds or unallocated
memory. Assessment receives the provider's validated Mode; the diagnostic
does not reread BORG_SHADOW. The subprocess controls deliberately change that
environment value after parsing and require the original mode in the record.
It is an operational diagnostic,
not a natural game death or a successful shadow comparison.

The syntactic source gate in scripts/decision_surface.py checks the four
native policies and every Rust file recursively under stateful/, including
the dispatcher and newly added modules. It rejects known core/module aliases,
unsupported memory/local paths, raw trait indexing/reads, unsafe/extern and
direct dependency bypasses. Only the exact existing dispatcher type import
and named mana-write adapter are exempted, never the entire file. Regression
fixtures cover the #302 bypasses and #329's compiled parent-path probe,
trait methods/borrows, native delegation, external includes/path attributes
and non-exact decisions imports. Parent paths permit only the explicit local
escape import names and the named mana setter call. The raw trait_ identifier
is forbidden everywhere else, not merely its indexing spelling.

This is not Rust semantic analysis and does not prove arbitrary code cannot
bypass recording. Compilation verifies names/types; manual review must inspect
fields, methods, macros and every changed dependency as well as the watcher.
All new native state fields must satisfy snapshot Clone/PartialEq or
compilation fails.

Tests verify that effects/RNG execute once, input-state disagreement cannot
be hidden by post-state replacement, changed requests/returns/final states
fail, and nested transactions do not duplicate effects. Actual native
smoke/full and marked fault/re-port controls remain the behavior gates.
No survival or performance improvement follows from agreeing programs.

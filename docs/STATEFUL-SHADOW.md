# Stateful Track R comparisons (#286)

The dependency direction stays shell → next → core. The core owns a typed
request enum for caution, recovery and all nine public escape entry points.
With shadow off, each wrapper immediately calls its original implementation;
no state is copied, no arguments are formatted and no tape/files are created.

In check, the native program drives the real state and engine. In switch,
the port drives them. The other program runs on an owned Rust policy snapshot
and receives recorded responses. **The engine is never cloned or rewound.**

The closed `decision_api` surface has 68 dependency functions. A stateful
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
The port's BI reads use named View fields; temporary policy mutations stay
explicit on the driving state or replay snapshot. The only direct BI write
is the named mana adapter, matching the native temporary-mana simulation.

An opaque native dependency runs with recording paused. If it re-enters a
ported policy or a nested Borg think, that inner transaction saves the
suspended tape and pause depth, performs its own comparison, then restores
them. The parent records the dependency's complete result once. The recorder
mutex is never held while a policy or engine dependency runs.

Internal ported entry markers compare function identity and arguments even
when escape is called inside caution. Statistics distinguish root stateful
comparisons, internal policy entries, individual entry points and dependency
requests. Positive coverage of caution, escape and recovery is required;
wall-helper comparisons cannot stand in for stateful comparisons.

The workspace intentionally uses panic=abort. An incompatible replay request,
input state or result type writes a durable mismatch to shadow.jsonl, then
stops with exit 2 before an unrecorded dependency can run. It does not invent
a result for the rest of a divergent replay. A completed comparison with a
different return/final state logs the mismatch and retains the driving
policy's result. Diagnostic request-divergence runs count as controlled
operational failures, never deaths or successful games.

The source gate in scripts/decision_surface.py rejects direct core/engine
imports and paths outside the recorded boundary, unnamed trait reads in the
port, unsafe/extern bypasses and tape manipulation by policy code. It is a
closed import/path check, not Rust semantic analysis. Compilation verifies
API names/types. Future carries must audit the complete changed native files,
new fields and methods as well as the function-name watcher. All new native
state fields must satisfy snapshot Clone/PartialEq or compilation fails.

Tests verify that effects/RNG execute once, input-state disagreement cannot
be hidden by post-state replacement, changed requests/returns/final states
fail, and nested transactions do not duplicate effects. Actual native
smoke/full and marked fault/re-port controls remain the behavior gates.
No survival or performance improvement follows from agreeing programs.

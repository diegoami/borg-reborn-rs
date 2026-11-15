# Track R shadow checks

R1.2 registers a `borg-core::shadow::Shadow` object from `borg-next` at
startup. The core never depends on the port crate. With `BORG_SHADOW=off`
(the default), the registry stays empty, no shadow files are created, and
the native decision executes once.

| Mode | Ported points | Unported points |
| --- | --- | --- |
| `off` | Native result | Native result |
| `check` | Compare both; native result decides | Native result |
| `switch` | Compare both; port result decides | Native result |

The first and only port in R1 is `danger.wall_like`, from
`borg-core/src/borg_danger.rs:wall_like` at vendored borg-rs `e6d13fc`.
It reads the same feature ID and returns the same boolean. Permanent wall,
lava and passable rubble remain false, even though their names may suggest
a different passability rule. This port preserves an estimate, not engine
truth. Tests cover agreement and injected disagreement in check and switch.

Danger's boundary passes its `(y, x, turns, average, full)` request and
native scalar result. Caution has a before callback and a result callback
with `acted`, the queue before evaluation and the queue after evaluation.
Fresh typed views read the core's own memory at these boundaries. The
stateful decisions are **unported** until R2: observing these boundaries is
not a comparison of danger/caution implementations, and zero mismatches
here does not establish their parity. R2 must extend the adapters to carry
any ported state changes as well as keys; this frame never runs native
caution twice or assumes that cloning its Rust state clones the engine.

Enabled modes require `BORG_RUN_DIR`, set by `harness/run.sh` and native
borgh's summary-directory fallback. `shadow.jsonl` contains only mismatches:
schema, mode, engine turn, Borg turn, point, inputs, original and ported
results. Each mismatch is written directly, so engine exit preserves it.
`shadow-stats.jsonl` records cumulative coverage after each completed think,
including nested thinks, and explicitly lists ported and unported points.
File and mode errors fail loudly. There is no policy logging path in off.

`harness/check-shadow.py RUN_OR_SUITE...` requires nonempty statistics,
positive comparisons, danger/caution boundary coverage and zero mismatches.
CI runs the native core's smoke suite in all three modes, compares all
play logs, and applies this gate to check and switch. Full off-mode parity
with pinned v12 is recorded in R1.2-VALIDATION.md. Full danger/caution ports
and their check/switch full-suite gates belong to R2.

The harness records `BORG_SHADOW` in config.txt; death replay restores it
and removes an inherited mode when the recorded run did not set one.

## R2 danger slice (#281)

The port now owns complete whole-square and standalone-monster danger
assessment, including physical blows, spells, movement horizons, simulation
modifiers and native aggregation order. All BI reads use named View fields;
ThreatMemory borrows buffs, clocks, simulation flags, fear maps, glyphs and
monster slots. Both native danger entry points run once, then compare against
this read-only port. Their calculations mutate no Borg or engine state and
consume no RNG. Switch returns the ported scalar. Off never constructs a
View, calls a port or reads a shadow-only engine turn.

The immutable threat_facts service shares existing lower-level projection,
floor/protection, spell-legality, blow classification, parameters and engine
tables with the native implementation. These dependencies are deliberately
unported; a zero does not independently validate them. Engine object
pointers are reduced to value facts (including whether friends exist).
Only the projection query's borrow signature changes, not its logic.

Statistics distinguish danger_comparisons, monster_comparisons and
wall_comparisons. The gate requires positive coverage of all three across
a suite, exact danger call/comparison correspondence in each run, and
consistent total counts. Wall-helper traffic cannot stand in for a complete
danger comparison. All three appear under ported_points; caution, escape
and recovery remain explicitly unported. The R2 milestone's stateful ports
and final carry report remain #209; #281 alone does not complete R2.

## R2 stateful slice (#286)

Caution, recovery and nine public escape entry points now have independent
ported programs. Their check/switch adapter compares complete transactions,
not only acted/keys. STATEFUL-SHADOW.md specifies effect-once execution,
request/input-state checks, internal entry markers, full final-state equality,
nested transactions, controlled divergence exits and the closed source gate.
No R2 points remain deliberately unported. Off remains the native path.

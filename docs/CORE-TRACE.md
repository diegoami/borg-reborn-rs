# Vendored-core decision trace

With `BORG_CORE=borg-rs`, each completed core think writes one JSON line to
`BORG_RUN_DIR/trace-core.jsonl`. The harness sets that directory. Set
`BORG_CORE_TRACE=off` (also `0` or `false`) to disable it; without a run
directory it is disabled. This stream is separate from the original
reborn core's `trace.jsonl`.

Schema `borg-reborn-rs/core-trace@1` records:

- `think`: monotonically increasing output sequence, starting at 1;
- `turn`: engine turn; `borg_turn`: the core's clock;
- `context`: town, store or dungeon; depth, HP/max HP and SP/max SP;
- `danger`: the core's one-turn, average evaluation at its position;
  `avoidance`: its native avoidance value;
- `check`: the successful root priority check (for example `borg_caution`,
  `borg_recover`, `borg_attack` or `borg_flow_old`); `check_source` names
  its hook in this source revision;
- `goals`: leaving, fleeing, fleeing_to_town and rising;
- `errand`: its name and optional target, or null;
- `keys`: queued numeric keycodes in consumption order; `acted`: the
  think's original boolean return.

State is sampled after the decision and before queued keys are consumed.
Keys describe the whole pending queue, including any already queued by an
outer decision; they are not an engine action receipt. Store decisions use
`borg_think_store`. Composite root checks (munchkin/lunal/bravery helpers)
retain their root name, rather than claiming a deeper tactical reason.
A false return is `borg_think_no_action`; a true return without a hook is
`unclassified`, rejected by validation. Source line numbers refer to this
build, while parameter source locations refer to the pre-extraction file.

Each successful root return has a marked `choice!` hook that returns the
original `true` without evaluating policy again. Nested thinks keep their
own selection frames and are written in completion order. Reading errand
context uses a nonblocking lock; nested thinks inherit the outer context
if that lock is busy. No trace value feeds back into policy. Danger's extra
evaluation is read-only and consumes no RNG; full-suite parity tests guard
that boundary. Files are written directly per line, without a userspace
buffer that the engine's C exit could strand. Write failures exit with a
clear diagnostic and status 2.

A natural death dossier preserves the last 200 rows as
`trace-core-tail.jsonl`, hashes it, and records its line counts in
`record.json`'s `core_decision_trace`. It also preserves `params-core.json`.
The original core's `decision_trace` field and `trace-tail.jsonl` remain
separate. See P1.4-VALIDATION.md for measured examples and limitations.

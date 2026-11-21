# Vendored-core decision trace

With `BORG_CORE=borg-rs`, each completed core think writes one JSON line to
`BORG_RUN_DIR/trace-core.jsonl`. The harness sets that directory. Native borgh adapter calls use the parent
of `-summary FILE` when BORG_RUN_DIR is absent (P3). Set
`BORG_CORE_TRACE=off` (also `0` or `false`) to disable it; without a run
directory or frontend summary destination it is disabled. This stream is separate from the original
reborn core's `trace.jsonl`.

New output uses schema `borg-reborn-rs/core-trace@3`. Historical `@1` and
`@2` evidence remains valid; consumers retain support for it. The current
schema records:

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
  think's original boolean return;
- `attacks`: each attack-ranking invocation completed in this think, with
  ordered `candidates` and a `winner` BF ID (or null). Candidates record
  attack ID/name, invocation family, class ID/name, `raw_utility`,
  `adjusted_utility`, `parameter_name`, `effective_weight`,
  `parameter_origin`, `eligibility` and `incumbent_changed`. Eligibility is
  `positive_return`, `nonpositive_return` or `unknown_attack`, not an
  inferred immunity, mana or charge diagnosis. An empty array means no
  ranking invocation was recorded. See [ATTACK-PREFERENCES.md](ATTACK-PREFERENCES.md).

- `prepared`, `trip_id` and `previous_prepared_outcome`: optional live
  preparedness checks and the observed next outcome, as described in
  [ZOO-INTERFACE.md](ZOO-INTERFACE.md#preparedness-trace-795).
- `death_risk` (since `@3`): `f1_danger_hp`, `f2_time_to_die` and
  `f3_race_escapes` in 0–100, `stop_eligible`, and the observed `features`.
  The dungeon sample is taken at the existing stop gate. Town and early
  returns get a fallback sample with `stop_eligible=false`. The features
  include plain melee HP per action and native fallback danger separately.

Attack rankings capture original simulation returns without rerunning helpers.
Repeated attempts append separate rankings; nested thinks keep their own
rankings. Tracing does not affect selection or execution. The `attacks` field
is absent from historical `@1` rows; it must not be inferred from them.

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

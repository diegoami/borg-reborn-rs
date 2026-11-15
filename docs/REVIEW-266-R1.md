# Review #266: Track R milestone R1

Reviewer: an independent session, a different model family from the
implementation family (OpenAI GPT-6 / Codex).

* Pinned snapshot: `607d73c232fbc6ba3108f989a6c6233f40ac2f40`
* Compared against: `8351f0a394987f9dd8c2daa5e243e276b464c00d` (the main
  baseline that `feature/track-r` was created from)
* Scope: R1's feature ledger only — the typed native view (#223 / #248),
  the shadow harness (#224 / #261) and the measured source-carry
  procedure (#225 / #265), plus the moving-reference refresh (#262 /
  #263).

Merging this review PR does not promote the redesign into `main` or
`reviewed`, and no R2 work is implied by it.

## Verdict

**Accept R1.** The three deliverables are real, measured and honest about
their limits. The shadow harness is demonstrably non-vacuous — a
constructed fault produced 97,506 mismatches and a re-port drove them
back to zero on the same game — the typed view reads the native core's
memory and not our perception, and the carry procedure failed loudly
where it should. Four follow-ups are recorded below; none blocks the
milestone.

The most valuable property of this milestone is that it **says what it
does not establish**, in the documents themselves: `docs/CARRY.md` states
that "a function-name watcher is not a dependency or semantic analysis",
the R1.3 trial reports a *negative* control (25 changed native
functions, zero changed ported functions) as a valid result rather than
as a failure, and `docs/R1.2-VALIDATION.md` states that the standard
cross-repository summary diff remains 0/45 rather than papering over it.

## What was reproduced

| Check | Result |
| --- | --- |
| `cargo test --workspace --exclude borg-core` | 7 (borg-next) + 152 + 4 + 5 pass |
| `python3 -m unittest discover -s scripts/tests` | 20 pass (17 on main, +3 for the port watcher) |
| `python3 -m unittest discover -s tools/tests` | 72 pass |
| Dependency direction | `borg-next` depends on `borg-core`; only `borg-reborn-rs/src/engine.rs:256` calls `borg_next::register()`. `borg-core` does **not** depend on `borg-next` — no cycle |
| Typed view | `#![forbid(unsafe_code)]`; lifetime-bounded `View<'a>`; 266 trait mappings, exactly matching the 266 native `BI_*` constants |
| Off path | `Mode::Off` returns **before** `register_shadow` is called (`borg-next/src/shadow.rs:183-185`); `shadow::enabled()` is then false and every helper is `map_or(original, …)` — the native path is provably untouched |
| Invalid `BORG_SHADOW` | exits 2 with a specific message (`borg-next/src/shadow.rs:171-174`, `:29`) |
| Shadow check anti-vacuity | `harness/check-shadow.py` rejects an empty output directory, a missing statistics file, non-contiguous think counts, wrong schema or mode, unexpected port coverage, a non-empty mismatch log, and zero totals |
| Constructed-change control | 97,506 mismatches, first at `borg_turn 21`, recorded with the actual inputs and both results |
| Re-port control | the same game (same seed, RNG and key count) returns to **0 mismatches** after the native change is re-expressed in the port |
| Configuration refusals | 4 repeat invocations refused with exit 2, no game launched |
| Native v13 refresh | 25 changed native Rust functions, **0 changed ported functions** — the valid negative control |
| Full refresh trial | real build identity, 291,865 comparisons, 0 mismatches, 3 real conflicted files resolved by hand |
| Death preservation | D2360–D2372 present, each with its trace tail and parameters |

## Findings

### F1 — the ported function is watched by name, and the documents say so

`docs/ported-functions.json` names one port: `danger.wall_like`. The
watcher (`scripts/rust_ports.py`) is a **Rust function-name matcher**, not
a dependency or semantic analysis.

This is stated plainly in `docs/CARRY.md` step 3 — "a function-name
watcher is not a dependency or semantic analysis" — and step 4 adds the
two directions that matter: "a source change may affect an unported
function or may change comments only; neither implies a port mismatch.
Conversely, a changed constant or helper can affect a port whose own
function text did not change."

The tool's behaviour matches that description. It masks comments, string
and character literals, raw strings (including `r###"…"###`), lifetimes
and nested block comments before finding function boundaries
(`scripts/rust_ports.py:6-67`), balances delimiters rather than
treating a generic's brace as a body (`:72-88`), and refuses to
produce an empty watch list: a declared port missing from both
revisions raises `missing in both revisions` (`:126-129`), and an
unknown revision raises `unknown commit` (`:117-120`).

The tests cover the hard cases: a fake function inside a raw string, a
nested block comment, a brace inside a character literal, const generics,
and a trait declaration (`scripts/tests/test_rust_ports.py:14-27`), plus
added/removed/changed classification and both failure modes (`:30-60`).

**This is not a defect.** It is a correctly scoped mechanism with an
honest description and a fail-loud default. It is recorded here because
the milestone's exit evidence rests on it: when the native source changes
in a way the watcher cannot see, the report says nothing, and R2's
adoption must not read "no ported changes" as "no port risk". The
documents already warn against exactly this reading.

### F2 — the typed view reads native memory, verified structurally

`borg-next/src/lib.rs` is the deliverable and it satisfies the review's
first requirement:

* `#![forbid(unsafe_code)]` at the top of the file.
* Every trait is read from `BorgState.borg.tr(BI_*)` — the core's own
  trait array — through the `trait_group!` macro, which cites the exact
  source index in the field's documentation. There is **no engine read,
  no global, no cache and no coercion**: the file's own module comment
  says so, and the imports are only `borg_rs::memory::*`.
* The mapping is complete: the view re-exports 266 `BI_*` constants and
  the native header defines exactly 266 `pub const BI_*`. A set comparison
  of the two finds no constant present in one and missing from the other.
* Entity identity is preserved: `monsters()` and `objects()` skip slot
  zero and filter deleted records (`r_idx == 0`, `kidx == 0`), and keep
  the native slot id — `view.monsters()[0].0 == 3` in the test at
  `borg-next/src/lib.rs`, with the assertion on the slot index.
* Unallocated and out-of-bounds memory has no grid:
  `Cave::grid` bounds-checks both axes and returns `None`; the test
  asserts `None` for `(0,0)`, `(-1,0)`, `(0,-1)`, `(AUTO_MAX_X,0)` and
  `(0,AUTO_MAX_Y)`.
* In-bounds unknown grids keep their native zero/unknown fields rather
  than becoming floor — the documented behaviour in the `Cave::grid` doc
  comment.
* The read-only lifetime is real: `View<'a>` borrows the state, and the
  doc comment states that the lifetime "prevents retaining borrowed
  map/entity/goal memory across any mutation of the core state". `core_status`
  and `goals` are `&'a` references and the test asserts pointer identity
  with the native fields.

One point worth stating explicitly, because it is easy to get wrong:
**the view does not read the engine.** It reads the vendored core's own
memory. That is what "ported code reads borg-rs's memory through a typed
view, not our perception" (PLAN.md) requires, and it is what makes the
shadow comparison meaningful.

### F3 — the shadow harness measures only what it claims to measure

The `Shadow` trait (`borg-core/src/shadow.rs:27-35`) has six hooks:
`danger`, `before_caution`, `caution`, `wall_like`, `after_think`, and
one more. The default implementations all return the original value or
do nothing, so a partially-implemented shadow is safe by construction.

Only `danger.wall_like` is ported in R1. `danger` and `caution` are
**observed but unported** — the statistics record them under
`unported_points` — which is the correct and honest configuration for a
frame milestone: `docs/CARRY.md` says "R1's only port is the pure
`wall_like` helper; R2 supplies stateful decision adapters", and the
carry trial records 25 changed native functions of which none is a
ported one.

The harness does not run any native function twice: the comparison
receives the native result as an argument and the port is evaluated
against it. `docs/R1.2-VALIDATION.md` states this and the unit tests
inject a real disagreement to prove the two modes differ — check retains
the native result, switch returns the port's, and both record the turn,
the inputs and both results.

The caution boundary is instrumented without re-evaluating any policy
expression: `borg_caution` is wrapped so that when the shadow is
disabled it calls the original directly and returns
(`borg-core/src/borg_caution.rs:1508`), and when enabled it observes
before and after (`1509-1515`). The keys are compared around the call,
never consumed.

### F4 — the mismatch detection is proven non-vacuous

This is the load-bearing evidence and it is real.

The **constructed change** (`docs/R1.3-POSITIVE-CONTROL.patch`) injects a
fault into the native function — "ordinary floor becomes a wall" — and
the harness then reports **97,506 mismatches**, with the first recorded
at `borg_turn 21` together with the actual input (`feature: 1`), the
original result (`true`) and the ported result (`false`). The run's
statistics agree with the mismatch log.

The **re-port** (`docs/R1.3-RE-PORT-CONTROL.patch`) then expresses the
same change in the port, and the harness returns to **0 mismatches** on
the same game — the same seed, the same RNG stream and the same key
count (1,361 keys, `borg_rng` unchanged). That is the strongest form of
the evidence: the port is demonstrably the thing that decides, and
correcting it restores parity.

The **full refresh** used a real borg-rs v13R source (`dc790dc46e13`),
produced three genuine conflicts in `borg_caution.rs`,
`borg_fight_defend.rs` and `borg_danger.rs`, resolved them by hand
(retaining the named parameters, taking upstream's fixed 20/40/60 HP cure
estimates and its `BI_PSTUN` protection checks, and removing the branch
upstream removed), and then measured 291,865 comparisons with zero
mismatches on a 10,000-turn run.

The **repeat invocations** were refused: four attempts to re-run into an
existing output directory exited 2 with no game launched
(`docs/R1.3-CARRY-RESULTS.json`, `configuration_refusals`). Replay
isolation works and is recorded rather than quietly retried.

### F5 — the carry procedure's limitation is stated, not hidden

`docs/CARRY.md` is a procedure with six numbered steps, each with a
failure mode attached: record the pins and build identities; run the
watcher and keep the manifest in step with the ports; refresh in an
isolated checkout and resolve conflicts by hand rather than by taking a
side; update the view and the parameter defaults where upstream
behaviour changed; build under the shared lock and read the actual
mismatch inputs, repeating "until comparisons are positive and
mismatches are zero. Never treat an empty log without coverage as a
pass"; and run the parity gates, preserving every natural death and
every failure.

Two points are worth recording as **accepted limits, not defects**:

* **Full v13 policy parity was not established.** The trial re-vendored
  the whole core and proved the *ported* helper still matches; it did
  not re-derive the policy defaults for v13 in full. `docs/CARRY.md`
  says the trial's changed-function list contains "zero changed ported
  functions" and calls this "a valid negative control, not a claim that
  v13's danger/caution behavior is unchanged". Correct, and the
  distinction is the one that matters.
* **Five new traits in v13 are not yet in the view.** The carry trial
  does not require them yet, and future production adoption needs #264's
  view/provenance/baseline work, which is open and tracked
  (`track:R`).

The procedure also keeps `pins.json` (moving reference material) and
`borg-core/PIN` (the runtime core) separate, and the refresh report
measures the port watcher from the **core pin**, not the reference pin —
correct, since the two move independently
(`scripts/refresh-sources.py:206-220`).

### F6 — the v12 comparator is exact and the parity claim is correctly scoped

`docs/R1.2-V12-COMPARISON.json` retains the build identities of both
sides (ours at `2545f841`, borg-rs v12 at `e6d13fc9`, the latter from
`diegoami/borg-rs`), the 45 run names, the three compared log types
(`keys.log`, `messages.log`, `state.log`) and the eight donor deaths'
existing dossier identities.

The shared-summary difference is recorded as exactly four fields, each on
all 45 runs: `borg_decisions`, `borg_rng`, `effective_settings` and
`wall_seconds`. The first three are the known frontend telemetry
differences already documented for track P; the fourth is wall time.
`docs/R1.2-VALIDATION.md` states the consequence plainly: "The standard
cross-repo diff therefore reports 0/45 summary-equal runs; this is the
same documented frontend difference as Track P, not a new policy
divergence. The parity claim is 45/45 across all three play logs, not
byte-identical summaries."

That is the correct scoping. No survival or performance improvement is
claimed, and the eight donor deaths are correctly identified as repeats
of previously recorded v12 games rather than new independent cases.

### F7 — the feature isolation holds

`feature/track-r` is the branch; `main` and `reviewed` are untouched by
R1's task PRs. The R1 work adds `borg-next/` (a new crate), the
`Shadow` seam in `borg-core`, and the carry tooling; it does not change
the vendored core's *behaviour* — the shadow's off path is provably
inert, and the full suite parity is preserved by the fact that the
default `BORG_SHADOW` is `off` (`borg-next/src/shadow.rs:181`).

## Not checked

* The source refresh's **unassessed death evidence** was not
  independently audited: the new better-borg-c cases D0024–D0030 are
  recorded as upstream reference material with their own reports, not
  re-assessed by this review.
* The zoo contract for the R1 branch was not re-run by this review
  beyond the shared engine test suite; the claim in the issue that "the
  actual newly pinned zoo adapter passes its three native contract
  cells" was verified on the track-P tip, which shares the adapter.
* Full v13 policy parity is out of R1's scope, as stated above.
* No population experiment was launched and no sealed suite was opened.

## Follow-ups

1. **#264** (already open): the full v13 native-core carry plan, the
   view/provenance work and the baseline. This is the gate before any
   production adoption of the v13 source.
2. When R2 ports `danger` and `caution`, extend `docs/ported-functions.json`
   and re-run the R1.3 controls so the non-vacuity evidence covers the
   new ports rather than only `wall_like`.
3. When the view gains v13's five new traits, extend the view's
   completeness check to compare the re-exported `BI_*` set against the
   native header's constant set, so a new native trait cannot be added
   without a decision to include or exclude it.
4. Consider recording the "ported functions changed" line of the refresh
   report in the handover of each carry, so a zero is visibly a negative
   control rather than an absence of news.

# Review #259: Track P, P1–P3 retrospective

Reviewer: an independent session, a different model family from the
implementation family (OpenAI GPT-6 / Codex).

* Base (pre-P): `c2c4e61db73b44bedfe1f17be9890abe87f76cb5`
* Review tip: `f693e69ba48da35598ca7bc35ec5f13f5db9f564`
* Compared: the P1–P3 implementation diff against the base.
* Scope: `borg-core/src` (17 files, +4,318/−1,711), the P3 zoo contract,
  the sweep handoff, the validation reports, the policy change of #258.

CI success is not review approval. This document reports findings; it does
not accept the milestone.

## Verdict

**Accept P1, P2 and P3 with the follow-ups below.** No blocking defect was
found in the parameter mechanism, the extraction, the trace or the zoo
contract. The findings are one behavioural gap in the core selector
(F1), four documentation/classification corrections (F2–F5) and two
consistency notes (F6–F7). Every material claim in the review prompt was
reproduced independently.

## What was reproduced

| Check | Result |
| --- | --- |
| `cargo fmt --check -p borg-reborn-rs` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass, no warnings |
| `cargo test --workspace --exclude borg-core` | 152 + 4 + 5 pass |
| `cargo test --release -p borg-core` | 137 pass, 1 pre-existing ignored |
| `python3 -m unittest discover -s scripts/tests` | 17 pass |
| `python3 -m unittest discover -s tools/tests` | 36 pass |
| `python3 -m unittest discover -s engine/tests` | 36 pass (12 environment skips) |
| `scripts/borg-core-vendor.py check` | ok — `borg-core/` is borg-rs at `e6d13fc9` plus the declared files and hooks |
| **Full suite parity** | **45/45 runs identical** against `harness/reference/full-rust` (keys, messages, states), built at source hash `1f5eb966caeb02bd` |
| Core trace, trace on | 351 rows, `think` contiguous 1..351, schema `borg-reborn-rs/core-trace@1`, **0 `unclassified`**, 0 `borg_think_no_action` |
| Core trace, trace off | `keys.log`, `messages.log`, `state.log` **byte-identical** to the trace-on run; no `trace-core.jsonl` written |
| Invalid parameter | exit non-zero, stderr `borg-reborn-rs: core parameters: caution_clw_min_loss: 101 outside 0..=100 percent` |
| Zoo profile contract | 3/3 cells pass with `BORG_ZOO_CONTRACT_REQUIRED=1` (real pinned adapter, real `run_cell`) |
| Parameter counts | **1,734** exactly: caution 200, danger 257, escape 109, recover 37, attack 167, defend 218, item_use 19, power 727. No duplicate names. |
| Bounds | every default lies inside its own declared range; **all 112 `divisor` parameters have `min ≥ 1`**; no parameter used as an array index, storage index or narrow field |
| Source locations | 1,725 of 1,734 defaults appear verbatim on their cited pre-extraction line; the 9 exceptions are the documented ones (see F4) |

The `full` suite was run from the review tip and the source hash matches
the one `docs/CORE-SWEEP.md:18-19` pins, so the sweep's build identity is
the build measured here.

## Findings

### F1 — `borg_reborn_core` accepts any value and silently selects the wrong core (behavioural gap, follow-up, not a blocker for the milestones as merged)

`borg-reborn-rs/src/borg.rs:292-298`:

```rust
*CORE.get_or_init(|| {
    std::env::var("BORG_CORE")
        .ok()
        .or_else(|| borg_rs::shell::borg_txt_setting("borg_reborn_core"))
        .is_some_and(|v| matches!(v.as_str(), "borg-rs" | "1"))
})
```

Only `"1"` and `"borg-rs"` select the vendored core. **Every other value —
including `"0"`, `"true"`, `"2"`, `"7"`, `""` and `" 1"` — selects the
original reborn core, with no diagnostic.**

This matters for P3 because the whole zoo measurement rests on the
selector. A profile typo (`borg_reborn_core: true` instead of `1`) is
written to `borg.txt` correctly and passes the pinned adapter's
`check_config` (which verifies only that the file contains the written
line — `vendor/borg-zoo/zoo/borgrs.py:138-142`, and whose cell field is
literally `config_checked = 'written file (borgh reports no effective
settings)'`). The run then plays the *original* core and the cell looks
normal. The error surfaces only later, as a different `params-core` hash
in the encounter family (`tools/export-death.py:199-207`) or as a
population result that measures the wrong arm.

The parameter *values* next to it are strict: an out-of-range parameter
exits 2 with a specific message (`borg-core/src/params.rs:1840-1845`,
`1885-1888`), which this review reproduced. The asymmetry is that the
selector, which decides *which core runs*, is the one value that is not
validated.

*Recommended remedy* (a follow-up task, not a milestone blocker): accept
only `0` and `1` (plus the textual `borg-rs`/`reborn` for native
callers) and exit 2 on anything else, with the same fail-loud shape as
the parameter parser. `docs/ZOO-INTERFACE.md:64-69` currently documents
`1`/`0` without stating that other values are accepted, so the
documentation should also say what happens for other values.

### F2 — two parameters named and typed as turn counts are actually decimal digit strings

`borg-core/src/borg_caution.rs:1724-1727` and
`borg-core/src/borg_recover.rs:405-409`:

```rust
borg_keypress(st, key(b'R'));
for digit in param(Param::recover_rod_rest_turns).to_string().bytes() {
    borg_keypress(st, key(digit));
}
borg_keypress(st, KC_ENTER);
```

The parameters are declared with unit `turns` (`params.rs:234`,
`params.rs:639`), but their value is now consumed as the **decimal
digits of the typed key sequence**, not as a number of turns. The
original was the fixed literal `R100` (recall) and `R100` (rod rest);
at the defaults the emitted bytes are identical (`"100"` → `'1','0','0'`),
and `docs/P1.3-VALIDATION.md:8` records the substitution honestly.

The defect is one of naming and type, not of behaviour at the defaults:
`caution_recall_rest_turns = 12` types `R12`, which is a *different
command*, not "rest 12 turns". A sweep arm that sets these values would
produce an unintended key sequence while the parameter name suggests a
duration.

*Recommended remedy*: rename both to reflect the key sequence (for
example `..._rest_key_digits`) or narrow their bounds to the only
admissible values and state in `docs/CALIBRATION.md` that they are
literal key strings, not durations.

### F3 — `danger_melee_hit_power_scale` is declared a divisor but used as a multiplier

`borg-core/src/params.rs:408` declares the unit `divisor` with
`min: 1`; `borg-core/src/borg_danger.rs:659` uses it as

```rust
... / ((level + power) * param(Param::danger_melee_hit_power_scale))
```

The `divisor` unit is what the `params.rs:1919-1927` test uses to reject
a zero for every divisor-unit parameter, so the safety net applies — but
the unit label misdescribes the operation, and `docs/CALIBRATION.md`'s
unit table says `divisor` is "the denominator of `x / parameter`".

*Recommended remedy*: correct the unit to `multiplier`, or re-express the
expression so the label matches.

### F4 — five protected-summon parameters cite a source line that does not contain their literal

`params.rs:482-486` cite `borg_danger.rs:1685`, `:1703`, `:1759`,
`:1777`, `:1795`. At those lines the pre-extraction file reads
`p += spell_power;` — the multiplication the parameter replaced is three
lines further down (`:1688` for KIN). The **defaults are correct** (1,
matching `p += spell_power`) and the code is correct
(`borg_danger.rs:1687` multiplies by the parameter). Only the citation is
imprecise, and it is inconsistent with the 15 sibling parameters, whose
citations are exact.

This is the same set of 9 exceptions the catalogue audit reports
(1,725 of 1,734 verbatim on their cited line): the 7
`danger_rsf_*_protected_power_multiplier` entries plus the two digit-loop
parameters of F2. Both are already documented in
`docs/P1.2-VALIDATION.md:18-20` and `docs/P1.3-VALIDATION.md:8`.

*Recommended remedy*: correct the five line numbers, or state in the
catalogue that these five cite the branch start rather than the
multiplication.

### F5 — "the RNG number/order of calls stay fixed" is true only at the defaults

`docs/CALIBRATION.md:72-74` states that "the RNG domain and number/order
of RNG calls stay fixed". The domain claim is unconditionally true: every
`randint` call in the changed files uses the literal `100`, and
`borg_danger.rs` contains no RNG call at all.

The *number* claim holds only at the defaults. Two parameters control the
sampling budget of the escape landing search —
`escape_landing_samples` and `escape_landing_trials`
(`borg-core/src/borg_escape.rs:364`, `:370`, defaults 100/100) — and each
inner iteration draws two random values. Overriding either shifts the
entire random stream for the rest of the game.

`docs/CALIBRATION.md:321` already scopes the claim correctly ("when the
defaults are used"), so the document is internally consistent; the
summary sentence at `:72-74` is the one that overstates it.

*Recommended remedy*: qualify the summary sentence the way `:321`
already qualifies it, and note the two parameters as the mechanism.

### F6 — `docs/CALIBRATION.md:68-69` cites a retained literal that is not locatable

The "Constants retained" list claims "the exactly-one monster/one-blow
test" is retained. No `mon.unique == 1` or `blows == 1` test exists in
`borg_caution.rs`; the only `== 1`/`!= 1` comparisons are the direction
test `d == 0 || d == 5` (`:1049`), `p == 0` (`:216`) and the `BI_CDEPTH`
/`BI_CURSP`/`BI_LIGHT` zero tests. The claim is either stale or refers to
a different file.

*Recommended remedy*: correct or drop the line. Nine other claims in that
section were checked and all verify.

### F7 — the `borg.txt` name and the environment name use different case rules

`params.rs:1812` requires the **exact lowercase** name in `borg.txt`
(`s.name == name`); `params.rs:1820` requires the **uppercase** form in
the environment (`s.name.to_ascii_uppercase() == *name`). A mixed-case
environment variable such as `BORG_PARAM_caution_clw_min_loss=42` is
therefore rejected as an unknown name, with exit 2.

This is fail-loud, which is the right behaviour under "invalid input is
never silently ignored", and `docs/CALIBRATION.md:12` documents the
uppercase environment form. It is recorded here only because the
asymmetry is easy to trip over when writing a sweep arm.

*Recommended remedy*: none required; optionally accept either case in the
environment.

## Milestone acceptance

### P1 — parameter mechanism, caution/danger/escape/recovery, named trace

**Accept.**

* The mechanism is sound. One macro generates the enum and the metadata
  together, so a row cannot drift from its index (`params.rs:11-22`).
  Precedence is environment → `borg.txt` → default (`params.rs:1824-1833`),
  asserted by three independent tests (`params.rs:1969-1974`, `:1988-1991`,
  `:1996-2014`). Invalid numbers, unknown names, duplicate file settings
  and out-of-range values all stop initialization with a specific message
  and status 2 (`params.rs:1885-1888`); **nothing is clamped**
  (`params.rs:1840-1845`).
* The extraction is faithful. The `tenths` pattern is uniformly
  `x * p / 10`; no order-of-operation change was found. The one
  asymmetric parenthesisation, `borg_danger.rs:1808`
  `(p * p) / 100`, is present in the baseline at `30ab9d66` and was
  preserved rather than normalised.
* The threshold coverage is complete. The `CAUTION` catalogue section's
  603 rows and the P2 sections' 1,131 rows match the registry exactly in
  name, default, unit, range and source line — **0 mismatches** — and all
  1,734 parameters are documented.
* The retained-literal classification holds. Mechanics (dice means, slay
  and brand multipliers, the copied device-failure formula, the copied
  spell-damage expressions) are still literals; policy thresholds and
  weights are parameters. **No item index, monster race index, activation
  id, trait index, array index or storage index was turned into a
  parameter.**
* The trace is complete and safe. 117 `choice!` hooks in
  `borg_think_dungeon.rs` and 3 in `borg_think.rs` cover the successful
  root checks; the macro returns the original `true` without re-evaluating
  any policy expression (`core_trace.rs:114-120`). Nested thinks keep
  their own frames (`core_trace.rs:22-26`, tested at `:197-207`). The
  extra danger evaluation is read-only — `borg_danger` and
  `danger_one_kill` read `st.danger`, `st.kills` and engine tables and
  contain no RNG and no state write — and the parity check confirms it:
  trace-on and trace-off produce byte-identical keys, messages and states.
* The death tail is preserved and hashed. The dossiers carry the last 200
  rows as `trace-core-tail.jsonl` with the count in `record.json`'s
  `core_decision_trace` (`tools/export-death.py:106-111`, `:140-141`),
  and every file hash in `files_sha256` verifies.

### P2 — attack, defence, item-use, power

**Accept.**

167 + 218 + 19 + 727 = 1,131, and 603 + 1,131 = 1,734: every count
claim in the issue, in `docs/CALIBRATION.md:3-4` and in each P2
validation report is exact.

The power branch is the part most likely to hide a rounding change, and
it does not: `borg_power.rs:18-24` states the C `int`/`long` width model
and the helpers `add`/`sub`/`addi`/`subi`/`count_up`/`count_down`
(`:70-105`) keep the width of each expression. There is **no float**
anywhere in the file, so no percentage or tenth rounds differently now.
The static and dynamic valuation paths are cleanly separated: the existing
`BORG_USES_DYNAMIC_CALCS` switch (`borg_power.rs:33-36`, `:1836-1844`)
still selects the formula branch and was not rewritten. The one
`divisor`-unit parameter used as a divisor there
(`power_equipment_sp_warrior_divisor`) has `min: 1`, as do its sibling and
all 112 divisor-unit parameters.

### P3 — native zoo profiles and the sweep handoff

**Accept**, with F1 as a follow-up task.

The profile contract test is **not a simulation**: it reads the borg-zoo
pin from `pins.json`, verifies the checkout against it, imports
`zoo.borgrs` from the pinned tree, asserts the imported adapter file
lives under that archive, and calls the adapter's real `run_cell` and
`check_config` (`engine/tests/test_core_profile_contract.py:33-46`,
`:74-83`). CI requires it rather than skipping it
(`.github/workflows/ci.yml`, `BORG_ZOO_CONTRACT_REQUIRED=1`). The three
cells assert: an integer profile selects the core and resolves two
parameter values with `borg.txt` origins; an invalid value fails with
`harness_failure` and exit 2 and writes no `params-core.json`; and
`borg_reborn_core: 0` keeps the original core working without producing
`params-core.json`. All three pass here.

The diagnostic path precedence is as documented: an explicit
`BORG_RUN_DIR` wins; otherwise the frontend derives the directory from
the parent of `-summary` (`engine/frontend/main-borgh.c:876-891`);
otherwise no automatic directory is selected
(`docs/ZOO-INTERFACE.md:101-105`).

`docs/CORE-SWEEP.md` is a prespecified protocol, not a result. It pins the
commit, the build source hash, the zoo pin and the exact cap; separates
128-seed exploration from 900 fresh confirmation pairs with a prespecified
two-sided McNemar test at α = 0.05; fixes the candidate-selection rule
(rank 7 and 8 by paired mean difference, tie to the smaller departure,
stop if the winner's difference is not positive); and requires every
planned cell to count. No sealed seed set is opened and no sealed outcome
is referenced as known (`CORE-SWEEP.md:5-7`, `:52-65`). No population
experiment was launched by this review.

## Not checked

* No population experiment, and no sealed suite was opened.
* The cross-repository **summary**-equivalence remains 0/45, on the three
  known frontend telemetry fields plus wall time. The parity claim is
  45/45 across keys, messages and states, and no survival or performance
  improvement is claimed.
* The 31 natural deaths of this review's own `full` suite were **not**
  exported as dossiers: they were produced by a review build with a
  different source identity than the committed reference, and they are
  not the harness's measured runs. They are reported here as a count, not
  as recorded evidence.
* The zoo adapter's `config_checked` field still reports only the written
  file (documented in `docs/P3.1-VALIDATION.md:28`); F1 is the practical
  consequence.
* Upstream v13's behaviour is outside this milestone's scope; `docs/P1.2-VALIDATION.md` already records that the legacy melee hit powers differ from current `blow_effects.txt` and that PFE's factor 2 approximates a level-dependent engine probability. Those remain named legacy estimates, not corrected facts.

## Follow-ups to file as issues

1. Validate `borg_reborn_core` (and `borg_reborn_errand` on the same
   pattern): accept only the documented values, exit 2 otherwise, and
   state the behaviour in `docs/ZOO-INTERFACE.md`. (F1)
2. Rename or re-type `caution_recall_rest_turns` and
   `recover_rod_rest_turns`; they are key digit strings, not durations.
   (F2)
3. Correct the unit of `danger_melee_hit_power_scale`. (F3)
4. Correct the five `danger_rsf_*_protected_power_multiplier` source
   lines, or document the convention. (F4)
5. Qualify the RNG-stability sentence in `docs/CALIBRATION.md:72-74` and
   name the two escape sampling parameters as the mechanism. (F5)
6. Correct or remove the "exactly-one monster/one-blow test" claim in
   `docs/CALIBRATION.md:68-69`. (F6)

None of these blocks the milestones. F1 should be filed before the owner
runs the P3 sweep, because a mis-selected core would invalidate an arm
silently.

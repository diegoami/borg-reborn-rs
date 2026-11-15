# Review #292: strict configured-errand validation (track P)

Reviewer: an independent session, a different model family from the
implementation family.

* Branch: `track-p`, head `747caa648adfd4c68844145d942519b9e23071d5`
* Base: `main` at `6acbf170a9adcaf2d0a00a8f186402f9e7267d34`
* Task: #269 (task issue), PR #290 (the task PR inside `track-p`)
* Implementation family: OpenAI GPT-6/Codex (as stated in the PR body)

## Verdict

**Accept.** The defect is real, correctly diagnosed, and the fix is the
right shape: the configured errand is validated as a complete command
line *before* any prefix is applied, so an invalid command can no longer
leave the vendored core running unconstrained while the measurement
records a run that did not do what was asked. Valid play is byte-identical
to `main`; every invalid input now fails with exit status 2 and a
diagnostic naming the setting's source and value.

Two **follow-ups** (not blockers) remain, both in the same family of
setting and neither in the task's stated scope. One documentation
inconsistency is also noted.

## What was reproduced

Everything below was run locally on the branch head, and the "main"
rows on a separate worktree at the base commit.

### The defect the change fixes

| Check | Result |
| --- | --- |
| `main`, invalid `BORG_ERRAND=dvie 2` | **reproduced**: stderr `borg: unknown errand command 'dvie 2'`, and the run **continued unconstrained** — 401 turns, 156 keys, `dead: False`. The measurement recorded a run that never did the requested errand |
| branch, invalid `BORG_ERRAND=dvie 2` | **fixed**: exit status 2, stderr `borg-reborn-rs: BORG_ERRAND: invalid errand selector "dvie 2": unknown command "dvie 2"`, no unconstrained run |

### The new behavior, exercised end to end

| Input | Result |
| --- | --- |
| `BORG_ERRAND=dvie 2` | exit 2 — `BORG_ERRAND: invalid errand selector "dvie 2": unknown command "dvie 2"` |
| `BORG_ERRAND=true` | exit 2 — `... unknown command "true"` |
| `BORG_ERRAND=/town; typo` | exit 2 — `... unknown command "typo"` (the invalid **suffix** is named, after a valid prefix) |
| `BORG_ERRAND=; ;` | exit 2 — `... empty command line; use none to disable errands` |
| `BORG_ERRAND=' '` (whitespace only) | exit 2 — `... empty command line; use none to disable errands` |
| `BORG_ERRAND` = non-UTF-8 bytes `/town \xFF\xFE` | exit 2 — `BORG_ERRAND: invalid non-UTF-8 errand selector "/town \xFF\xFE"` (the raw bytes are shown) |
| `borg_reborn_errand = /typo` in borg.txt, no env | exit 2 — `borg.txt borg_reborn_errand: invalid errand selector "/typo": unknown command "/typo"` |
| `BORG_ERRAND=none` + invalid `borg_reborn_errand` in borg.txt | **no error** — the environment wins |
| `BORG_ERRAND=/dive 2` + invalid `borg_reborn_errand` in borg.txt | **no error** — the environment wins |
| `BORG_ERRAND=''` (empty) + valid `borg_reborn_errand = /dive 2` | **no error**, errands disabled — the empty environment value overrides the file, as before |
| `BORG_ERRAND=none` / `''` / `/dive 2` / `/town` / `/stopscared off; /dive 2` | exit 0, no diagnostic — all valid forms still work |
| `borg_reborn_errand = none` in borg.txt, no env | exit 0, no diagnostic — errands disabled |
| original core (`BORG_CORE=reborn`, or `borg_reborn_core = reborn` in borg.txt) + invalid `BORG_ERRAND` | **no error**, exit 0, `params.json` written and `params-core.json` absent — the vendored-core setting is not consulted, so the original core is unaffected |
| vendored core selected in borg.txt (`borg_reborn_core = borg-rs`) + invalid `BORG_ERRAND` | exit 2 — the file-selected arm validates too, not only the environment-selected one |

### Valid play is unchanged

| Check | Result |
| --- | --- |
| `main` vs branch, `BORG_ERRAND=none` / `''` / `/dive 2`, seed 1011 race 2 class 1 | `keys.log`, `messages.log`, `state.log` **byte-identical** in all three cases |
| Smoke suite on the branch | **3/3 runs identical** against `harness/reference/smoke-rust` (keys, messages, states) |
| Full suite on the branch (reborn core) | **45/45 runs identical** against `harness/reference/full-rust` |
| Full suite on the branch (vendored core, trace on) | **295,464** trace rows across 45 runs, **0** `unclassified` — matches the PR's figure exactly |
| Trace on/off invariance | `keys.log`, `messages.log`, `state.log` identical with and without `BORG_CORE_TRACE=off`; the trace file is absent when off |

### The validation is complete, not partial

The new check validates the **whole** line before applying any prefix.
I modelled `borg_rs::shell::parse_line` and `Command::parse` and compared
the new gate against `apply_line` over 35 inputs; **every** value the new
code accepts is also accepted by `apply_line`, and every value it rejects
is rejected there too. The `exit(2)` branch in `engine.rs` is therefore
not reachable for a value the new validator admitted, and no valid value
is newly rejected.

The `commands.is_empty()` guard is what rejects `" "` and `"; ;"`:
`parse_line` filters empty segments, so those parse to zero commands
rather than to an error.

### The empty-alias compatibility fix

| Check | Result |
| --- | --- |
| The rejected first candidate (`014e64a4`) | rejected the empty string; `engine/tests/test_shopping.py:72` uses `errand=''` as its "no errand" fixture and caught it |
| The corrected commit `c259f69c` | restores `None | Some("" | "none") => Ok(None)`; the shopping test is **unchanged** by both the PR and the rejected candidate (`git diff --stat` on `engine/tests/test_shopping.py` is empty for both) |
| `engine/tests/test_shopping.py` on the branch | all four tests pass |

### Suites and checks

| Check | Result |
| --- | --- |
| `cargo test --workspace --exclude borg-core` | 154 + 4 + 5 pass (the PR's figures) |
| `cargo test --release -p borg-core` | 138 pass, 1 ignored (the PR's figures) |
| `cargo fmt --check -p borg-reborn-rs` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `python3 -m unittest discover -s engine/tests` | 45 pass, 2 skips (the 2 need the pinned zoo) |
| `engine/tests/test_core_profile_contract.py` with the pinned zoo (`c5f1b042`) | **8/8 pass** — including the three new ones; without the zoo checkout they are silently skipped, so the count only means something with the checkout present |
| `python3 -m unittest discover -s scripts/tests` | 19 pass |
| `python3 -m unittest discover -s tools/tests` | 78 pass (needs the pinned cemetery at `death-tools-pin.txt`) |
| `scripts/borg-core-vendor.py check` | `borg-core/ against borg-rs e6d13fc9baff: ok` |
| CI on the PR | `rust`, `engine`, `recorder` all **green** |

### The death evidence

| Check | Result |
| --- | --- |
| New dossiers | 18 (`D2399`–`D2416`), every `record.json` carrying the **actual** source identity of the build that produced it: `D2399`–`D2406` at `77f040ff…` (the rejected candidate `014e64a4`), `D2407`–`D2416` at `cb48d943…` (the corrected `c259f69c`). No dossier is relabelled |
| The CI reconcile gate, reproduced locally | the three `ci.yml` runs (`core-params`, `core-trace-off`, `core-trace-death`) with `BORG_DEATH_RECORDS_DIR` to a temp registry → `1 natural deaths in the runs; 0 without a dossier` and `0 not in …/docs/deaths` — **the gate passes on this branch** |
| Local registry index | 2396 dossier directories, 2396 listed, `index.json` reports `cases: 2396, distinct: 2393, attempts: 2396` — consistent |
| The sweep pin | `2ec1f2507ec81f375a7967b2876597865458aa76` is an ancestor of `track-p` and **contains** `c259f69c`; the commit resolves on the remote |
| The catalogue fragments | merging all three `docs/examples/core-calibration-*.json` fragments into the pinned zoo's catalogue (`c5f1b042`) loads with no name collision and expands `core-mage-rest-explore-v1` to **384 cells** with the pinned specimen and control |

## Findings

### F1 — the diagnosis is right and the fix has the right shape (no action)

The old code validated lazily and partially: `apply_line` parses the line,
applies each non-errand command **as it goes**, and only discovers an
unreadable command when it reaches it — at which point the earlier
commands have already taken effect and the run continues with no errand
at all. The new code parses the complete line first
(`borg-reborn-rs/src/borg.rs:356`) and refuses to start if any part of it
is unreadable. That is the correct fix for a measured arm: a configuration
that cannot be honoured must not silently become a different arm.

The exit status 2 is also right. The `BORG_CORE` selector was made strict
the same way in the preceding task (#280), so the two settings that
choose the arm now agree: an invalid selector is an error, not a default.

### F2 — `BORG_STOPSCARED` keeps the old lenient behavior (follow-up)

`borg-reborn-rs/src/engine.rs:262` still uses the old `setting` closure
for `BORG_STOPSCARED`:

```rust
let setting = |env: &str, key: &str| {
    std::env::var(env)
        .ok()
        .or_else(|| borg_rs::shell::borg_txt_setting(key))
        .filter(|s| !s.is_empty() && s != "none")
};
if let Some(c) = setting("BORG_STOPSCARED", "borg_reborn_stopscared")
    .and_then(|v| borg_rs::shell::Command::parse(&format!("stopscared {v}")))
```

An invalid value is dropped without a word. Reproduced:

```
BORG_CORE=borg-rs BORG_STOPSCARED=bogus harness/run.sh … 1011 2 1 400 menu
→ rc=0, stderr has no diagnostic
```

The run then plays with the **default** stopscared threshold instead of
the requested one — the same class of silent misconfiguration this task
removes for errands, and in the same function, ten lines above the code
that was made strict. `docs/ERRANDS.md:16` lists the accepted values
(`off`, `flee`, `1`-`1000`), so an invalid one is a configuration error,
not an alias.

This is outside #269's stated scope (which is the errand selector), so
it does not block the merge. But leaving it means the repository has two
policies for the same kind of setting, and the next session to touch
`engine.rs` will reasonably assume the strict one is the house rule.

**Suggested follow-up:** a task that routes `BORG_STOPSCARED` through the
same validated path, with the same source/value diagnostic and exit 2.

### F3 — `BORG_ERRAND_BUDGET` is still silently defaulted (follow-up)

`borg-reborn-rs/src/engine.rs:274`:

```rust
let budget = std::env::var("BORG_ERRAND_BUDGET")
    .ok()
    .and_then(|b| b.parse().ok())
    .unwrap_or(0);
```

Reproduced:

```
BORG_CORE=borg-rs BORG_ERRAND=/dive 2 BORG_ERRAND_BUDGET=bogus harness/run.sh …
→ rc=0, stderr has no diagnostic
```

An unreadable budget becomes `0`, which `apply_line` reads as "no
budget" — an unbounded errand. The budget is the bound on the errand's
length, so an invalid one silently removes the bound rather than the
errand. Same family as F2, same argument; also outside #269's scope.

Note this one is a little more consequential than F2: F2's fallback is
the core's default threshold, whereas F3's fallback is "unbounded". Both
are the wrong answer for the same reason.

### F4 — the ERRANDS.md paragraph runs past the margin (documentation, trivial)

`docs/ERRANDS.md:22` — "including an invalid command after a valid
prefix, stop the process with exit status 2 …" — is a long unwrapped
line in a document whose other paragraphs are wrapped at ~78 columns.
Cosmetic; worth folding in when F2 or F3 is picked up.

### F5 — the catalogue example's profile lives in a third file (documentation, pre-existing)

Not introduced by this PR, but worth knowing before the sweep is
dispatched: `docs/examples/core-calibration-explore.json` names the
profile `core-calibration-default`, which is in **neither** the zoo's
catalogue nor `docs/examples/core-calibration-catalogue.json` — it is in
`docs/examples/core-calibration-profiles.json`, a third fragment. Loading
the catalogue fragment alone therefore fails with
`ValueError: unknown profile: core-calibration-default`.

Merging **all three** fragments works: the experiment loads and expands
to 384 cells. The same is true on `main` (the PR only re-pins the
specimen name), so this is pre-existing and not a regression. But
`docs/CORE-SWEEP.md:37` lists the three profile names without saying
which file defines them, and the catalogue fragment's own note says only
"Merge with existing catalogue", which does not cover the third file.
Since the sweep handoff is the thing this PR is preparing, one sentence
naming the three fragments would remove a trap for whoever dispatches it.

## Untested areas

* **No population experiment was launched** — correctly, since the owner
  gates dispatch (RULES 20). The sweep handoff's specimen pin, the
  catalogue fragments and the 384-cell expansion were validated
  statically; **no experiment was planned or run**, and no survival or
  improvement claim is checked by this review.
* **The zoo's own adapter path was not exercised.** The pinned zoo
  (`c5f1b042`) never writes `borg_reborn_errand`; the strictness is
  reached through the zoo only because `zoo/catalogue.py:256`
  (`setting_value`) accepts booleans and integers, both of which are
  invalid errand selectors. That path is covered by
  `test_invalid_errand_profile_is_not_an_unconstrained_core_run` against
  the pinned zoo, but the *native* adapter's errand handling at a newer
  zoo pin is not covered here.
* **The RULES 12 reconciliation for the rejected candidate's eight
  deaths** was checked by identity (each dossier carries the candidate's
  own `source_sha256` and `git_rev`), not by replaying all eight runs
  from scratch. The ten corrected-build deaths and the three `ci.yml`
  scenarios were reproduced.
* **Interactive `^Z /` parsing** was confirmed unchanged by reading
  `borg-core/src/shell.rs:500` (`borg_cmd_errand`) and by the fact that
  `borg-core/` is byte-identical to its pin (`borg-core-vendor.py
  check` ok), not by driving the interactive prompt.
* **The full-suite reference is the pinned v12**; the branch's 45/45 was
  checked against `harness/reference/full-rust` as committed, and that
  reference was not regenerated.

## Reproducers

The defect, on the base commit:

```sh
git worktree add /tmp/main 6acbf170
cd /tmp/main && BORG_IMPL=rust harness/build.sh
BORG_CORE=borg-rs BORG_ERRAND='dvie 2' harness/run.sh /tmp/out 1011 2 1 400 menu
grep 'unknown errand' /tmp/out/stderr.log   # the warning
python3 -c "import json;print(json.load(open('/tmp/out/summary.json'))['game_turns'])"
# 401 — the run continued unconstrained
```

The fix, on the branch head:

```sh
BORG_CORE=borg-rs BORG_ERRAND='dvie 2' harness/run.sh /tmp/out2 1011 2 1 400 menu
# run failed (rc=2)
grep 'borg-reborn-rs:' /tmp/out2/stderr.log
# borg-reborn-rs: BORG_ERRAND: invalid errand selector "dvie 2": unknown command "dvie 2"
```

F2, on the branch head:

```sh
BORG_CORE=borg-rs BORG_STOPSCARED=bogus harness/run.sh /tmp/out3 1011 2 1 400 menu
# rc=0, stderr silent — the requested threshold is dropped
```

F3, on the branch head:

```sh
BORG_CORE=borg-rs BORG_ERRAND='/dive 2' BORG_ERRAND_BUDGET=bogus \
  harness/run.sh /tmp/out4 1011 2 1 400 menu
# rc=0, stderr silent — the budget becomes 0, i.e. unbounded
```

## Recommendation

**Accept and merge into `main`** once the owner records the acceptance
this PR asks for. The task's own contract — validate the configured
errand completely, fail loudly, preserve valid and default behavior —
is met and independently verified. F2 and F3 are follow-up tasks in the
same area; they do not affect the correctness of this change, and F2 in
particular is a *pre-existing* condition of `BORG_STOPSCARED` rather
than something this PR introduced.

The one thing I would not do is let F2 and F3 sit unfiled: the next
change to `engine.rs` will read the strict errand path as the house
style and assume the neighbouring setting follows it.

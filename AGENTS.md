# Agent entry

This file is for any session that works on this repository: a cloud
session, a local one, any model, or a person. It is the only entry point;
`CLAUDE.md` just includes it. Read it first, then:

1. [RULES.md](RULES.md);
2. [docs/HANDOVER.md](docs/HANDOVER.md): where the work stands and what the
   last session left;
3. [PLAN.md](PLAN.md): milestones, tasks and their status;
4. [docs/GOALS.md](docs/GOALS.md): what we build and the owner's decisions;
5. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): the design direction;
6. [reference/c-manual/README.md](reference/c-manual/README.md): the Angband
   Strategy Manual (chapters 0–16, appendices A1–A10, *notes for the borg*
   in ch. 15) and, at `reference/c-manual/user-manual/`, the official user
   manual of the game version we play. The game knowledge every analysis
   starts from (S14);
7. [docs/SOURCES.md](docs/SOURCES.md): what every related project is for here;
8. [docs/research/KNOWLEDGE.md](docs/research/KNOWLEDGE.md): what has been
   read from the moving sources (borg-runner, better-borg-c), what it means
   for the tasks ahead, and what the refresh report says to re-read.

## What this repository is

The Angband Borg reborn in Rust: a ground-up redesign of how the Borg
**plays**. **[docs/SPEC.md](docs/SPEC.md) is the top-level contract.** The
game is upstream Angband `42cd156ac` byte for byte; the integration (patch
series, frontend, bindings, build, harness, tools) is ours. borg-rs is a
reference to copy from, except its decision core, vendored at a pin as
`borg-core/` with our errands around it (SPEC §2 amendment, the repivot of
2026-11-14; docs/ERRANDS.md). Our own core depends on nothing of borg-rs. Parity
with the C Borg is not a goal. [borg-zoo](https://diegoami.github.io/borg-zoo/)
measures our Borg and makes every comparison (SPEC §6), through the zoo
contract ([docs/ZOO-INTERFACE.md](docs/ZOO-INTERFACE.md)). Perception is
checked against the engine itself (T2.5).

## Game knowledge, and where analysis starts

The Borg's knowledge of the game has three inputs, in this order:

1. **The engine** (`engine/angband/`, at `42cd156ac`). It is the only
   authority: a formula, a table, a prompt or a key sequence is checked in
   `src/` or `lib/gamedata/` and cited there (RULES 16).
2. **The manuals**, copied in [`reference/c-manual/`](reference/c-manual/README.md)
   (S14): the **Angband Strategy Manual**, a source-grounded guide to how the
   game works underneath and how to survive it, with *notes for the borg* in
   chapter 15 (decision rules and candidate parameters) and experiments in
   chapter 16; and the **official user manual** for the version we play
   (`reference/c-manual/user-manual/`, also `engine/angband/docs/`) for
   commands, screens and options. Read the relevant chapter before designing
   a rule, then check every number against the engine.
3. **Evidence from recorded play**: the death dossiers
   (`reference/deaths/`, S4/S9/S11), borg-runner's accumulated knowledge
   (`reference/borg-runner/`, S15) and our own decision traces. Evidence is a
   lead, not a rule: nothing is a claim until borg-zoo measures it (RULES 14),
   and a rule or threshold no measurement supports is declared a hypothesis.

## Starting

```sh
git status --short
git log --oneline -5
gh issue list --label "in progress"   # who is working on what
bash scripts/setup.sh            # tools check; vendor/ checked out at the pins; safe to rerun
python3 scripts/refresh-sources.py   # what moved in every source since its pin (read only)
cargo test --workspace --exclude borg-core && cargo test --release -p borg-core
```

`borg-core/` (borg-rs's crate, vendored) keeps borg-rs's own formatting and
its unit tests link without the engine only in release, so the checks are
CI's (`.github/workflows/ci.yml`): `cargo fmt --check -p borg-reborn-rs`,
not `cargo fmt --check`; never run `rustfmt` on a vendored file.
`scripts/borg-core-vendor.py check` (with `vendor/borg-rs`) shows that
`borg-core/` is borg-rs at its pin plus our declared files and hooks.

Engine builds and suites (see [harness/README.md](harness/README.md)) run under the shared lock:

```sh
BORG_IMPL=rust flock /tmp/angband-validation.lock harness/build.sh
flock /tmp/angband-validation.lock harness/suite.sh smoke harness/out/smoke
harness/diff --suite smoke --expect-impl rust harness/out/smoke harness/reference/smoke-rust
```

## Track P integration and review gate (owner, 2026-11-15)

**Use `track-p`, not `p-track`.** Read this branch's current AGENTS.md and
RULES.md before P work. Start P task branches from `origin/track-p`, target
P task PRs at `track-p`, and merge them there only after required CI passes.
Close the task issue explicitly with its integration PR/commit. Never send
or merge a P task PR directly into `main` because CI passed.

At every milestone, stop and open a pinned **`track-p` → `main`** review
PR. The owner selects/runs the review; fix findings through `track-p` task
PRs. Keep promotion unmerged until the owner accepts the milestone review
and CI passes. The owner merges the promotion or explicitly delegates that
reviewed merge. Then the next milestone may start. Keep the reviewed head
fixed while the review is active; record a new exact pin after fixes.
Synchronize needed `main` changes through a CI-checked PR into `track-p`,
preserving the other track's work. Do not force-push the integration branch.
This P-specific rule overrides the generic merge instructions below; other
tracks keep their own authorized workflow.

Migration: P1–P3 were already on main at `8351f0a3` before this decision.
They need retrospective review; do not describe them as independently
reviewed or rewrite/revert main to simulate pre-merge review. The combined
review and handover identify that historical exception. All new P tasks
and review fixes follow the gate above.

Pre-rule dependency exception: P1–P3 were already on `main` at `8351f0a3`.
For those milestones, owner acceptance of the retrospective review #259
satisfies the P review dependency; a second runtime promotion is unnecessary.
A reviewer recommendation alone is not owner acceptance. New P work still
requires the reviewed `track-p` → `main` promotion. This clarification does
not undo other-track work already authorized before the gate.

## Choosing the work

**Work happens in tracks** (owner, 2026-11-14). PLAN.md lists them:

| track | label | what | epic |
|---|---|---|---|
| **P** | `track:P` | calibration of borg-rs's core: named parameters, a decision trace | #218 |
| **R** | `track:R` | the redesign: borg-rs's decisions re-expressed in our domain, behind a shadow check | #217 |
| Q | `track:Q` | the query surface (only when the owner opens it) | |

**A session works on one track**, the one the owner names in its first
message ("work on track P"). Without one, ask the owner; do not pick. Inside
the track, **work only in its milestone marked in progress** in PLAN.md.
Each milestone has a label of its own (`P1`, `R1`, ...):

```sh
gh issue list --label task --label "track:P" --label P1 --state open
```

Take the lowest-numbered open task issue of that milestone whose **Needs**
issues are closed (or in an open PR you can stack on, RULES 5) and that has
no `in progress` label. A Needs issue may belong to the other track: track R
ports a subsystem only after P has named its thresholds and the relevant
P milestone has been reviewed and promoted to `main`. Add the
`in progress` label and comment with your branch name before you start.

**Track R branch rule (owner, 2026-11-15).** Start R task branches from
`feature/track-r` and target every R task PR at `feature/track-r`.
Merge green task PRs there. Keep the unfinished redesign out of `main` and
`reviewed`; only the owner's explicit approval of a ready redesign permits
promotion. Milestone completion alone does not permit it. Synchronize main
into the integration branch to inherit Track P work, then update task branches.

Two sessions on different tracks share `borg-core/`. Keep each change to
the files its task names, synchronize the applicable integration branch
(`track-p` for P, otherwise the track’s authorized base) before the PR, and
never rewrite the other track's work to make yours fit; say so in the PR
and the handover instead.

At the end of a task:
1. `cargo fmt --check -p borg-reborn-rs`, `cargo clippy --workspace
   --all-targets -- -D warnings`, `cargo test --workspace --exclude
   borg-core` and `cargo test --release -p borg-core` pass (CI's commands),
   as well as the task's own checks;
2. docs/HANDOVER.md says what was done, what was found, what is left, and
   anything the owner has to decide; anything left becomes a new issue;
3. the PR says what changed, how it was checked and what was not checked,
   and `Closes #N` for its issue;
4. **you merge it yourself into the authorized integration branch** once
   CI is green (`track-p` for P; RULES 1). P promotion to `main` requires
   the owner-accepted milestone review, not just task CI.

When your track's milestone has its last task merged, **stop** and open the
review PR (PLAN.md). Track R uses an immutable feature snapshot into
review/track-r-baseline, never main or global reviewed. Do not start the track's next milestone, and do not
move to the other track.

## Models

This repository is **model agnostic**. Nothing in it depends on which model
or tool writes the code, and the Borg itself uses no model when it plays.
Which model works on a task is decided outside the repository by the
owner's general rules. **The reviewer is always the owner's choice**
(RULES 6). The session writes the review prompt and hands it over; it does
not pick or launch a reviewer. A milestone review is done by the owner or by
a model of a **different family** from the one that wrote most of the
milestone. Name both in the review PR.

## Where things are

| Path | What |
| --- | --- |
| `pins.json` | The pinned commits of every related repository |
| `reference/` | Copies of sources read often: C Borg spec and Tunable Borg Plan, **the Angband Strategy Manual and the official user manual (`c-manual/`, S14)**, what agent players learned (`borg-runner/`, S15), curated death dossiers, the zoo's catalogue (see docs/SOURCES.md) |
| `docs/research/` | Research digests (deaths and issues by subsystem) |
| `borg-reborn-rs/` | The Rust crate: `engine.rs` (the only `unsafe`, the engine boundary), `borg.rs` (the control loop), `strategy.rs` (L5: the first strategy's modes, goals and budgets), `danger.rs` (M3's danger placeholder, the C Borg's danger sum lifted), `executor.rs` (L6: actions to key sequences, the prompt catcher, expected effects), `path.rs` (L4's paths: per-think distance maps with threat-aware costs), `town.rs` (town sessions: the handover, the state records, the object log), `observation.rs`, `world.rs`, `knowledge.rs`, `params.rs`, `trace.rs`, `bindings.rs` (generated by `engine/bindings/generate.sh`) |
| `scripts/refresh-sources.py` | Reports what moved since each pin: commits, new death cases, new tags (new control arms), changed C Borg functions we cite, updated issues. `--update` refreshes pins, `reference/` copies and the death index in their own PR (RULES 21) |
| `reference/deaths/INDEX.md` | Generated index of every Borg death we know of: better-borg-c, borg-rs and ours |
| `scripts/setup.sh` | Tools check and pinned checkouts in `vendor/` (ignored by git). A checkout found at another revision is reported, not moved; `--checkout` moves clean ones, `--strict` fails on any mismatch. To read a pin regardless of the checkout: `git -C vendor/NAME show COMMIT:path` |
| `engine/` | The engine integration (docs/ENGINE.md): `angband/` (upstream `42cd156ac`, byte for byte, never edited, RULES 9), `patches/` (our harness hooks), `frontend/` (mode `borgh`), `bindings/` (bindgen), `CMakeLists.txt` and `sources.cmake` (our build), `tests/` |
| `harness/` | build (`build.sh`), run, suite, diff, identity, boundary and zoo checks, references; see [harness/README.md](harness/README.md) |
| `tools/export-death.py` | Thin pointer to the cemetery-owned exporter; normal dossiers go to borg-cemetery (RULES 12) |
| `watch-borg.sh`, `tools/rerun-death.py`, `tools/death_registry.py` | the launcher to watch the Borg (X11), the rerun of a recorded death with its logs, the registry's index (TE.6; the copied GUI, recording and replay tools are gone) |
| (removed) | borg-rs's GUI and headless launchers, recording, verified replay, human takeover and death workflow, copied in T1.6 with the adaptations listed at the top of [tools/README.md](tools/README.md) |
| `docs/ZOO-INTERFACE.md` | The contract borg-zoo's adapter calls: build, run, outputs (TE.7) |
| `docs/QUERY-SURFACE.md` | Secondary goal Q (D12): the draft of `borg-mcp`, the MCP server through which language models query the Borg's judgement (query mode) or play a hosted game as a separate arm (play mode) |
| `docs/deaths/` | Preserved historical validation archive; canonical new death reports/analysis live in borg-cemetery |

## Dossier IDs across branches (owner, 2026-11-15)

Follow [DEATH-ID-POLICY.md](docs/DEATH-ID-POLICY.md): final numeric IDs belong
to centralized cemetery acceptance. Local IDs are provisional; qualify
cross-branch references by exact containing registry commit. CI artifacts
also need run/registry namespacing before commit. Preserve old evidence and
origin mappings; run `python3 scripts/check-death-ids.py docs/deaths` before
integration. It detects numeric collisions that git's path merge can miss.
Canonical publication changes remain the cemetery owner's work.

## Local machine notes

Heavy runs take `flock /tmp/angband-validation.lock`, like the owner's other
Angband work (RULES 17). The C Borg repositories and borg-rs may have
checkouts in sibling directories. Read them, never write to them.

## Optional recorder (owner #210, 2026-11-14)

The standalone Rust recorder is an owner-authorized optional integration. Read
docs/ANGBAND-RECORDER.md and recorder-pin.json for its exact release and build
recipe. Its hooks apply to the generated tree only and the default harness
remains off. The old copied T1.6 recorder remains removed; this is a pinned
external crate through the existing Rust staticlib, not a borg-rs dependency.

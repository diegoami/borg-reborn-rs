# Agent entry

This file is for any session that works on this repository: a cloud
session, a local one, any model, or a person. It is the only entry point;
`CLAUDE.md` just includes it. Read it first, then:

1. [RULES.md](RULES.md);
2. [docs/handover/](docs/handover/README.md): where the work stands and what
   the last sessions left, one note per task, newest first
   (`ls -r docs/handover/`); older entries are in
   [docs/HANDOVER.md](docs/HANDOVER.md);
3. [PLAN.md](PLAN.md): milestones, tasks and their status;
4. [docs/GOALS.md](docs/GOALS.md): what we build and the owner's decisions;
5. [docs/architecture/](docs/architecture/README.md): how the original C Borg
   plays, the borg-core we ship, and the reborn plan (tracks P, R and Q);
6. [docs/strategy/README.md](docs/strategy/README.md): the Angband
   Strategy Manual (chapters 0–16, appendices A1–A10, *notes for the borg*
   in ch. 15) and, at `engine/angband/docs/`, the official user
   manual of the game version we play. The game knowledge every analysis
   starts from (S14);
7. [docs/SOURCES.md](docs/SOURCES.md): what every related project is for here;
8. [docs/research/KNOWLEDGE.md](docs/research/KNOWLEDGE.md): what has been
   read from the moving sources (borg-runner, better-borg-c), what it means
   for the tasks ahead, and what the refresh report says to re-read.

## What this repository is

The Angband Borg reborn in Rust: a ground-up redesign of how the Borg
**plays**. **[docs/SPEC.md](docs/SPEC.md) is the top-level contract.** The
game is upstream Angband `574b618c` byte for byte; the integration (patch
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

1. **The engine** (`engine/angband/`, at `574b618c`). It is the only
   authority: a formula, a table, a prompt or a key sequence is checked in
   `src/` or `lib/gamedata/` and cited there (RULES 16).
2. **The manuals** (S14): **our Angband Strategy Manual**, in
   [`docs/strategy/`](docs/strategy/README.md) (moved here 2026-11-21, ours to update), a source-grounded guide to how the
   game works underneath and how to survive it, with *notes for the borg* in
   chapter 15 (decision rules and candidate parameters) and experiments in
   chapter 16; and the **official user manual** for the version we play
   (the Angband developers', untouched in `engine/angband/docs/`) for
   commands, screens and options. Read the relevant chapter before designing
   a rule, then check every number against the engine.
3. **Evidence from recorded play**: the death dossiers
   (`reference/deaths/`, S4/S9/S11), borg-runner's accumulated knowledge
   (`reference/borg-runner/`, S15) and our own decision traces. Evidence is a
   lead, not a rule: nothing is a claim until borg-zoo measures it (RULES 14),
   and a rule or threshold no measurement supports is declared a hypothesis.

## Gameplay reports and scenarios (owner, 2026-11-16)

**Write gameplay evidence in borg-cemetery.** Incident/death investigations,
reusable saved or constructed scenarios, behavioral trial reports and
counterfactual comparisons belong there, including validation incidents.
This source repository keeps implementation, tests, immutable fixture references
and short validation summaries linking to the canonical case. Do not create
new gameplay reports under docs/ or add new validation dossiers to docs/deaths/
to satisfy CI. The existing historical archive is retained unchanged until an
explicitly reviewed migration; its existence is not a precedent for new reports.

Player-cemetery may retain original player/mixed recordings and working saves;
link those inputs from the Cemetery case. Case-specific alternative plays and
scenario trials are Cemetery evidence. Do not duplicate canonical analysis in
player-cemetery. Preserve published raw hashes/history while replacing misplaced
current report pages with pointers. All P/R tracks follow this ownership rule.

## Starting

Work in a worktree of your own, never in the shared main checkout
(RULES 5a): `git worktree add .worktrees/<name> -b task/Tn.m-short-name
origin/<base>` (`track-p` for a P task, otherwise `main`).

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
harness/diff --suite smoke --expect-impl rust harness/out/smoke harness/reference/smoke-borg-rs
```

**Which core a run plays** (owner, #606). With no `BORG_CORE`, the harness
plays the core `harness/borg.txt` names: **borg-rs** (`borg-core/`), the one
the zoo runs and the only one that reads `harness/borg.txt` and its
`borg_param_*` settings. Its reference is `harness/reference/smoke-borg-rs`.
`BORG_CORE=reborn` plays our own reborn core, which reads
`harness/profiles/reborn-default.txt` instead; its references are
`smoke-rust`, `smoke-exact-rust` and `full-rust`. A change to a borg-rs
setting or to `borg-core/` is checked on the default core; a review prompt
says which core each check ran. A PR that changes how the borg-rs core plays
at the shipped settings re-records `smoke-borg-rs` in the same PR, after
merging main (`harness/make-reference.sh` from a clean build), as reborn-core
changes do with `smoke-rust`: CI compares the PR merged with main.
To change a shipped value in `harness/borg.txt`: edit it, commit, then run
`scripts/rerecord-smoke-borg-rs.sh`, which builds, plays the smoke suite,
re-records the reference and commits it; push both commits together.

## Retained evidence: all tracks

For historical validation imports on any track, qualify provisional native
IDs using `tools/import-deaths.py` and run its `--check` before committing.
See [RULES.md](RULES.md) rule 12 and
[docs/DOSSIER-IMPORT.md](docs/DOSSIER-IMPORT.md). Preserve source records and
payloads from both branches when synchronizing; never rewrite the accepted
inventory or replace a historical death to hide an ID collision.

**An unchanged control game needs no new record** (owner, 2026-11-16, #464).
CI's `reconcile-deaths.py` (borg-cemetery, through `death-tools-pin.txt`)
accepts a death as KNOWN when the Cemetery retains the same start and the
same key stream from another build. Only a game that plays differently
(UNRECORDED) needs a new Cemetery package and a fixture pin move.

## Track P integration and review gate (owner, 2026-11-15)

**Use `track-p`, not `p-track`.** Read this branch's current AGENTS.md and
RULES.md before P work. Start P task branches from `origin/track-p`, target
P task PRs at `track-p`, and merge them there only after required CI passes
and the owner says the review is done (RULES 1a).
Close the task issue explicitly with its integration PR/commit. Never send
or merge a P task PR directly into `main` because CI passed.

At every milestone, stop and open a pinned **`track-p` → `main`** review
PR. The owner chooses the reviewer (RULES 6); the session may launch that
reviewer, which posts its review on the PR. Fix findings through `track-p` task
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
2. the task's handover note, a new file in `docs/handover/` (RULES 5b), says
   what was done, what was found, what is left, and
   anything the owner has to decide; anything left becomes a new issue;
3. the PR says what changed, how it was checked and what was not checked,
   and `Closes #N` for its issue;
4. **you merge it into the authorized integration branch** (`track-p` for
   P) once CI is green **and the owner has said its review is done**
   (RULES 1a, owner 2026-11-17). Green CI alone is never enough: report it
   and ask. P promotion to `main` requires the owner-accepted milestone
   review, not just task CI.

When your track's milestone has its last task merged, **stop** and open the
review PR (PLAN.md). Do not start the track's next milestone, and do not
move to the other track.

## Models

This repository is **model agnostic**. Nothing in it depends on which model
or tool writes the code, and the Borg itself uses no model when it plays.
Which model works on a task is decided outside the repository by the
owner's general rules. **The owner chooses the reviewer** (RULES 6): the
session does not pick one, and it does not post a review before the owner
has named the reviewer. The session writes the review prompt, pinned to an
exact commit, and posts it on the PR it reviews; the owner names the
reviewer; the session may then launch the reviewer the owner named, with
the pinned prompt and nothing else from its own context, or hand the
prompt to the owner to run. It never reviews its own work in its own
context. A milestone review is done by the owner or by a model of a
**different family** from the one that wrote most of the milestone. Name
both in the review PR.

**If you are the reviewer, post your review on the PR yourself** (owner,
2026-11-22; RULES 6). However you were launched (the owner, a model
the owner launched, a session the owner asked to review, a tool), post the finished
review on the PR it reviews (on the issue, when there is no PR) through
whatever write path you have: `gh pr comment <number> --repo
diegoami/borg-reborn-rs --body-file <file>` (`gh issue comment` for an
issue), a connected GitHub App or connector, or an MCP GitHub server.
Post a conversation comment, never a GitHub review body, *approve* or
*request changes*. Head it with the reviewer the owner named (model and
tool), where the owner named it, the pinned commit and the head at review
time (and which of the two the findings cover, when they differ), and
answer with the comment's URL. Findings returned only in chat are not a delivered review.
Only if you have no write path at all, say so in the first line of your
answer and give the full review; then, or when the owner ran the review
and has not posted it, the session posts it unedited, and
never posts a second copy of a review already on the PR. Posting is not
the owner's acceptance (RULES 1a). Every review prompt the session writes
says this in its first lines, with the exact command for its PR and the
other write paths, and repeats the command at its end.

## Where things are

| Path | What |
| --- | --- |
| `README.md` | For players: requirements, setup, building, watching the Borg, a headless run, which core plays, changing its settings |
| `docs/` | Current documents by audience, indexed in [docs/README.md](docs/README.md); `architecture/` (the C Borg, borg-core, the reborn plan, the layered design), `validation/` (per-PR receipts and dossier maps, with a map of their old `docs/` paths), `reviews/`, `archive/` (documents that no longer apply, each with when and why), `handover/` |
| `pins.json` | The pinned commits of every related repository |
| `docs/strategy/`, `docs/c-borg/` (with `fixes/`), `docs/design/`, `docs/research/borg-agent/` | **Our documents moved here from our other repositories** (owner, 2026-11-21): the Angband Strategy Manual; the C Borg's specification, plan and defect catalogue, and better-borg-c's verified fixes; the shipped core's design (town planner, trip policy, ...); borg-agent's knowledge. Ours to change; [docs/DOCUMENTATION-MAP.md](docs/DOCUMENTATION-MAP.md) separates our documents from other authors' (the game's manual and the original C Borg's, untouched in `engine/angband/`) |
| `reference/` | Copies of our documents that a live tool still produces or reads: the zoo's pages, borg-runner's manuals and run notes, the death dossiers, borg-rs's own records. Never edited here; `scripts/check-reference.py` checks they are their source's bytes at the pin (reference/README.md) |
| `docs/research/` | Research digests (deaths and issues by subsystem) |
| `borg-core/` | borg-rs's decision core, vendored at its pin with our declared files and hooks: the core that plays by default ([docs/architecture/borg-core.md](docs/architecture/borg-core.md)) |
| `harness/borg.txt`, `harness/borg_<class>.txt`, `harness/scary_monsters.txt`, `harness/plans/` | The shipped core's settings, per-class values, scary list and plans |
| `borg-reborn-rs/` | The Rust crate: the engine boundary and core selector, and our frozen reborn core: `engine.rs` (the only `unsafe`, the engine boundary), `borg.rs` (the control loop), `strategy.rs` (L5: the first strategy's modes, goals and budgets), `danger.rs` (M3's danger placeholder, the C Borg's danger sum lifted), `executor.rs` (L6: actions to key sequences, the prompt catcher, expected effects), `path.rs` (L4's paths: per-think distance maps with threat-aware costs), `town.rs` (town sessions: the handover, the state records, the object log), `observation.rs`, `world.rs`, `knowledge.rs`, `params.rs`, `trace.rs`, `bindings.rs` (generated by `engine/bindings/generate.sh`) |
| `scripts/refresh-sources.py` | Reports what moved since each pin: commits, new death cases, new tags (new control arms), changed C Borg functions we cite, updated issues. `--update` refreshes pins, `reference/` copies and the death index in their own PR (RULES 21) |
| `reference/deaths/INDEX.md` | Generated index of every Borg death we know of: better-borg-c, borg-rs and ours |
| `scripts/setup.sh` | Tools check and pinned checkouts in `vendor/` (ignored by git). A checkout found at another revision is reported, not moved; `--checkout` moves clean ones, `--strict` fails on any mismatch. To read a pin regardless of the checkout: `git -C vendor/NAME show COMMIT:path` |
| `engine/` | The engine integration (docs/ENGINE.md): `angband/` (upstream `574b618c`, byte for byte, never edited, RULES 9), `patches/` (our harness hooks), `frontend/` (mode `borgh`), `bindings/` (bindgen), `CMakeLists.txt` and `sources.cmake` (our build), `tests/` |
| `harness/` | build (`build.sh`), run, suite, diff, identity, boundary and zoo checks, references; see [harness/README.md](harness/README.md) |
| `tools/export-death.py` | Thin pointer to the cemetery-owned exporter; normal dossiers go to borg-cemetery (RULES 12) |
| `watch-borg.sh`, `tools/rerun-death.py`, `tools/death_registry.py` | the launcher to watch the Borg (X11), the rerun of a recorded death with its logs, the registry's index (TE.6; the copied GUI, recording and replay tools are gone) |
| (removed) | borg-rs's GUI and headless launchers, recording, verified replay, human takeover and death workflow, copied in T1.6 with the adaptations listed at the top of [tools/README.md](tools/README.md) |
| `docs/ZOO-INTERFACE.md` | The contract borg-zoo's adapter calls: build, run, outputs (TE.7) |
| `docs/QUERY-SURFACE.md` | Secondary goal Q (D12): the draft of `borg-mcp`, the MCP server through which language models query the Borg's judgement (query mode) or play a hosted game as a separate arm (play mode) |
| `docs/deaths/` | Preserved historical validation archive; canonical new death reports/analysis live in borg-cemetery |

## Local machine notes

Heavy runs take `flock /tmp/angband-validation.lock`, like the owner's other
Angband work (RULES 17). That lock is for builds, smoke and replays of named
cases; comparisons over many starts go to borg-zoo as a request issue, not
to local batches (RULES 20, owner 2026-11-20). The C Borg repositories and borg-rs may have
checkouts in sibling directories. Read them, never write to them.

## Optional recorder (owner #210, 2026-11-14)

The standalone Rust recorder is an owner-authorized optional integration. Read
docs/ANGBAND-RECORDER.md and recorder-pin.json for its exact release and build
recipe. Its hooks apply to the generated tree only and the default harness
remains off. The old copied T1.6 recorder remains removed; this is a pinned
external crate through the existing Rust staticlib, not a borg-rs dependency.

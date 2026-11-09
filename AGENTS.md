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
6. [docs/SOURCES.md](docs/SOURCES.md): what every related project is for here.

## What this repository is

The Angband Borg reborn in Rust: a ground-up redesign of how the Borg
**plays**, with the conventions, structure, interfaces, tooling and external
behaviour of [borg-rs](https://github.com/diegoami/borg-rs). borg-rs is a
faithful port of the C Borg. This repository is not: parity with the C
Borg is not a goal. Our results are compared with **control C Borgs**
(the original and the latest better-borg-c milestone) built here and run
under the same run contract (docs/GOALS.md D8), on the same seeds, through
[borg-zoo](https://diegoami.github.io/borg-zoo/). borg-rs is the source of
our engine tree and tooling, not a control; perception is checked against the
engine itself (T2.5).

## Starting

```sh
git status --short
git log --oneline -5
gh issue list --label "in progress"   # who is working on what
bash scripts/setup.sh            # tools check; vendor/ checked out at the pins; safe to rerun
python3 scripts/refresh-sources.py   # what moved in every source since its pin (read only)
cargo test --workspace
```

Engine builds and suites (see [harness/README.md](harness/README.md)) run under the shared lock:

```sh
BORG_IMPL=c flock /tmp/angband-validation.lock harness/build.sh
BORG_IMPL=c flock /tmp/angband-validation.lock harness/suite.sh smoke harness/out/smoke-c
harness/diff --suite smoke --expect-impl c harness/out/smoke-c harness/reference/smoke-c
```

## Choosing the work

PLAN.md's milestone table says which milestone is in progress. **Work only
inside it.** Tasks are GitHub issues labelled `task` in that GitHub
milestone:

```sh
gh issue list --milestone "M2. Perception and world model" --label task --state open
```

Take the lowest-numbered open task issue whose **Needs** issues are closed
(or in an open PR you can stack on, RULES 5) and that has no `in progress`
label. Add the `in progress` label and comment with your branch name before
you start.

At the end of a task:
1. `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` and
   `cargo test --workspace` pass, as well as the task's own checks;
2. docs/HANDOVER.md says what was done, what was found, what is left, and
   anything the owner has to decide; anything left becomes a new issue;
3. the PR says what changed, how it was checked and what was not checked,
   and `Closes #N` for its issue;
4. **you merge it yourself** once CI is green (RULES 1).

When the milestone's last task is merged, **stop** and open the review PR
(PLAN.md). Do not start the next milestone.

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
| `reference/` | Copies of sources read often: C Borg spec and Tunable Borg Plan, the Angband Strategy Manual (`c-manual/`, complete; `strategy/`, the earlier draft), curated death dossiers, the zoo's catalogue (see docs/SOURCES.md) |
| `docs/research/` | Research digests (deaths and issues by subsystem) |
| `borg-reborn-rs/` | The Rust crate: `engine.rs` (the only `unsafe`, the engine boundary), `borg.rs` (control loop and, until M3, a placeholder policy), `headers/` (borg-rs's generated engine bindings, copied unchanged) |
| `scripts/refresh-sources.py` | Reports what moved since each pin: commits, new death cases, new tags (new control arms), changed C Borg functions we cite, updated issues. `--update` refreshes pins, `reference/` copies and the death index in their own PR (RULES 21) |
| `reference/deaths/INDEX.md` | Generated index of every Borg death we know of: better-borg-c, borg-rs and ours |
| `scripts/setup.sh` | Tools check and pinned checkouts in `vendor/` (ignored by git). A checkout found at another revision is reported, not moved; `--checkout` moves clean ones, `--strict` fails on any mismatch. To read a pin regardless of the checkout: `git -C vendor/NAME show COMMIT:path` |
| `angband/` | The engine tree from borg-rs, with history; never edited here (RULES 9). What it contains and how to re-sync it: [docs/ENGINE-TREE.md](docs/ENGINE-TREE.md) |
| `harness/` | borg-rs's harness (build, run, suite, diff, identity) with the differences listed in [harness/README.md](harness/README.md) |
| `borg-rs/` | The seam package: the name the imported CMake builds; it only re-exports `borg-reborn-rs` |
| `tools/export-death.py` | Writes a basic death dossier at the end of every run that died (RULES 12) |
| `tools/endpoint.py` | The primary endpoint of GOALS.md per arm and class (all attempted runs, exact boundary, Wilson CI, paired McNemar) over local suite outputs and borg-zoo cells |
| launchers, the rest of `tools/` | borg-rs's GUI and headless launchers, recording, verified replay, human takeover and death workflow, copied in T1.6 with the adaptations listed at the top of [tools/README.md](tools/README.md) |
| `docs/deaths/` | From M1 (T1.2, before the first game): our death dossiers, borg-rs format |

## Local machine notes

Heavy runs take `flock /tmp/angband-validation.lock`, like the owner's other
Angband work (RULES 17). The C Borg repositories and borg-rs may have
checkouts in sibling directories. Read them, never write to them.

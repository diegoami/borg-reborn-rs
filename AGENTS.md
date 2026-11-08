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
our engine tree and tooling and the oracle for perception, not a control.

## Starting

```sh
git status --short
git log --oneline -5
bash scripts/setup.sh            # tools check; vendor/ checked out at the pins; safe to rerun
cargo test --workspace
```

From M1 on, also `harness/build.sh` (see PLAN.md T1.2) under the shared lock.

## Choosing the work

PLAN.md's milestone table says which milestone is in progress. **Work only
inside it.** Take the first task of it whose status is `todo` and whose
"Needs" are `done` (or in an open PR you can stack on, RULES 5). Set its
row to `in progress` with your branch name in your first commit.

At the end of a task:
1. `cargo fmt --check`, `cargo clippy --workspace -- -D warnings` and
   `cargo test --workspace` pass, as well as the task's own checks;
2. PLAN.md's row and docs/HANDOVER.md say what was done, what was found,
   what is left, and anything the owner has to decide;
3. the PR says what changed, how it was checked and what was not checked;
4. **you merge it yourself** once CI is green (RULES 1).

When the milestone's last task is merged, **stop** and open the review PR
(PLAN.md). Do not start the next milestone.

## Models

This repository is **model agnostic**. Nothing in it depends on which model
or tool writes the code, and the Borg itself uses no model when it plays.
Which model works on a task, and which reviews a milestone, is decided
outside the repository by the owner's general rules. The one requirement
here is that a milestone review is done by the owner or by a model of a
**different family** from the one that wrote most of the milestone. Name
both in the review PR.

## Where things are

| Path | What |
| --- | --- |
| `pins.json` | The pinned commits of every related repository |
| `reference/` | Copies of sources read often: C Borg spec and Tunable Borg Plan, curated death dossiers, the zoo's catalogue (see docs/SOURCES.md) |
| `docs/research/` | Research digests (deaths and issues by subsystem) |
| `borg-reborn-rs/` | The Rust crate (`staticlib` + `rlib`), linked into the engine from M1 |
| `scripts/setup.sh` | Tools check and pinned checkouts in `vendor/` (ignored by git). A checkout found at another revision is reported, not moved; `--checkout` moves clean ones, `--strict` fails on any mismatch. To read a pin regardless of the checkout: `git -C vendor/NAME show COMMIT:path` |
| `angband/`, `harness/`, `tools/`, launchers | From M1, with borg-rs's names and behaviour |
| `docs/deaths/` | From M1 (T1.2, before the first game): our death dossiers, borg-rs format |

## Local machine notes

Heavy runs take `flock /tmp/angband-validation.lock`, like the owner's other
Angband work (RULES 17). The C Borg repositories and borg-rs may have
checkouts in sibling directories. Read them, never write to them.

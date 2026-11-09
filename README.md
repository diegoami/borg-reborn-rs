# borg-reborn-rs

The Angband Borg reborn, in Rust: an automatic player for Angband, redesigned
from the ground up in how it **plays**, while keeping the conventions,
structure, interfaces, tooling and external behaviour of
[borg-rs](https://github.com/diegoami/borg-rs) (the faithful Rust port of the
C Borg, itself following [better-borg-c](https://github.com/diegoami/better-borg-c)).

It plays the same game the other Borgs play: Angband 4.2.6 at upstream
`e473720ed88a0f0c7ca66815cef3000eb4a38bc3`, behind the same `inkey_hack`
seam, launched the same way (**Ctrl-Z**, **z**), measured by the same
headless harness and by [borg-zoo](https://diegoami.github.io/borg-zoo/)
on the same seeds.

**Status (2026-11-09):** milestone 1, *the seam and the yardstick*, in
progress. M0 (research, goals, plan) is reviewed (tag `m0`). The engine and
borg-rs's harness, launchers, recording and replay are in place. The
original and better-borg-c C Borgs build as byte-checked controls, and every
run has an exact-boundary contract and a death record. The Rust Borg plays,
but only as a placeholder until M3. Tasks are GitHub issues
([milestones](https://github.com/diegoami/borg-reborn-rs/milestones)).

| Document | What it holds |
| --- | --- |
| [docs/SOURCES.md](docs/SOURCES.md) | Step 1: every source evaluated, what it teaches, what we take and what we leave |
| [docs/GOALS.md](docs/GOALS.md) | Step 2: what we are building and why, the objective, and the owner's decisions |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | The design direction those goals imply (a draft, to be proven milestone by milestone) |
| [PLAN.md](PLAN.md) | Step 3: milestones and tasks, and when we stop for a review |
| [docs/HANDOVER.md](docs/HANDOVER.md) | Where the work stands, for the next session |
| [reference/](reference/) | Copies of the sources we read most: the C Borg specification, the Tunable Borg Plan, curated death dossiers, the zoo's catalogue |

## Playing

```sh
./watch-borg.sh [NAME]               # build (X11) and open the game; create/load a character, then Ctrl-Z, z
BORG_IMPL=rust harness/build.sh      # the headless build (the zoo's entry point)
harness/run.sh OUT SEED RACE CLASS TURNS menu   # one deterministic headless run
tools/rerun-death.py docs/deaths/DXXXX-... OUT  # rerun a recorded death with the Borg's logs
```

Saves and settings of `watch-borg.sh` are in `.angband-reborn/`. Until M3
the Borg is a placeholder (it walks, rests and descends; see
[docs/HANDOVER.md](docs/HANDOVER.md)). [tools/README.md](tools/README.md)
documents the death records.

Working on this repository: start at [AGENTS.md](AGENTS.md), then
[RULES.md](RULES.md). It is model agnostic: any coding agent, or a person,
follows the same files.

## Related repositories

| Repository | Role here |
| --- | --- |
| [angband/angband](https://github.com/angband/angband) | The game and the original C Borg (`src/borg/`), at the pin above. Read only. |
| [neo-angband-tracker](https://github.com/diegoami/neo-angband-tracker) | The C Borg specification and the Tunable Borg Plan (`docs/c-borg/`); the issue board for C Borg defects. |
| [better-borg-c](https://github.com/diegoami/better-borg-c) | The C Borg with verified bug fixes; native measurement runner; death dossiers. |
| [borg-rs](https://github.com/diegoami/borg-rs) | The faithful Rust port. Our template for structure and tooling, and the source of the engine boundary we reuse. |
| [borg-zoo](https://github.com/diegoami/borg-zoo) | Measures Borg versions on frozen seed suites. Our yardstick. |
| [borg-agent](https://github.com/diegoami/borg-agent) | A language-model player that writes down game knowledge with evidence, meant as an input to this Borg. |

Licence: GPLv2 (see `LICENSE`), as borg-rs. Angband itself is dual-licensed
GPLv2 / Angband licence; files copied from other projects keep their
headers and are listed with their provenance.

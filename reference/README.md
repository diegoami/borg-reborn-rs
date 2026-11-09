# Reference sources

Copies of sources that sessions read often, so they are available offline
and at a fixed version. The original repositories stay canonical, and they
keep moving. Do not edit these files: replace them with a newer copy in a
PR that updates the pin and this table.

| Path | Copied from | Commit | What is included |
| --- | --- | --- | --- |
| `c-borg/BORG_SPEC.md` | diegoami/neo-angband-tracker `docs/c-borg/` | `eefc55000dd5d5f43d8f077f6543b4594807ce08` | The C Borg specification. Its relative links point into neo-angband-tracker |
| `c-borg/TUNABLE-BORG-PLAN.md` | same | same | Revision 3 plus *What a remake leaves out* (2026-11-08) |
| `c-borg/README.md` | same | same | Index of the C Borg documents |
| `strategy/` | diegoami/neo-angband-tracker `docs/strategy/` on branch **`c-borg-reference`** (pin `strategy-manual`, its own clone `vendor/neo-angband-tracker-c-borg-reference`) | `e67fd7dbce189a3e3a2f4d4910c8367fa93af605` | The Angband Strategy Manual: every file of the directory, chapters 1–2 and 4–8, appendices A1–A6 and their generator. Work in progress at this commit: chapter 3 (`03-magic.md`) is not written yet, so its four links do not resolve |
| `deaths/better-borg-c/` | diegoami/better-borg-c `docs/deaths/` | `b1588cc908c667a859db9a60e9d109766fe366c4` | Every top-level file (README, FEEDBACK, REVIEW-TEMPLATE, `index.json`, `owner-reviews.json`, `window-profile-v2.json`), and every `.md` and `.json` file at the root of each case: `report.md`, `assessment.json`, `record.json`, `build.json`, `overall-assessment.md`, owner feedback, review anchors and verification records |
| `deaths/borg-rs/` | diegoami/borg-rs `docs/deaths/` | `e5cb263eed8eee93860b5cfcdc18af99e6d134f1` | Same selection, including `discussion.json`, `owner-reviews.json`, the cohort notes and the seed reports |
| `borg-rs/FORK-CORRECTIONS.md`, `borg-rs/adoption/status.json` | diegoami/borg-rs `docs/` | see `pins.json` | What borg-rs changed in the engine tree we import (docs/ENGINE-TREE.md) |
| `zoo/DESIGN.md`, `zoo/catalogue/` | diegoami/borg-zoo | `3f6b76e27dba3ddd238b2104fff4d344165cb28f` | Specimens, profiles and suites as of that commit |

## Not copied: evidence that needs a read of the pinned repository

Each case's subdirectories (event logs, terminal maps, snapshots, frames,
saves and raw recordings) are left out because of their size. A few
relative links in the copied files point outside `docs/deaths/` and do not
resolve here. Read them at the pinned commit, for example
`git -C vendor/better-borg-c show 63fbc1f:docs/measurement/DEATH-IMPROVEMENT-WORKFLOW.md`:

| Registry | Link target (relative to the case or registry) | What it is |
| --- | --- | --- |
| better-borg-c | `docs/measurement/DEATH-IMPROVEMENT-WORKFLOW.md`, `DEATH-QUEUE.md`, `FLOW-DANGER-CACHE.md`, `NAMED-HEADLESS.md`, `flowcache-404/d0008-result.json` | Process documents and the D0008 flow-cache result |
| better-borg-c | `produce_death.py`, `replay-borg.py`, `replay_next_death.py`, `run-borg-headless.py` | Tools named in replay instructions |
| borg-rs | `docs/C-CONSUMER-ADOPTION.md` | The adoption contract |
| borg-rs | `docs/validation/2026-11-07-post-m4-investigation/` (including `death-checkpoints/*.save`) | Validation runs and living saves of the cohort |
| borg-rs | `produce_death.py`, `replay-borg.py`, `replay_next_death.py` | Tools named in replay instructions |

The strategy manual's links to the official manual (`../birth.rst`,
`../command.rst` and the other `docs/*.rst`) and its source references
(`src/...`, `lib/gamedata/...`) are relative to an Angband tree: here they are
`angband/docs/` and `angband/src/`, `angband/lib/gamedata/` in our engine tree.

`deaths/INDEX.md` is generated from the copies and from our own
`docs/deaths/` by `scripts/refresh-sources.py`. Refresh everything with
`scripts/refresh-sources.py --update` (RULES 21); the copy rules are the
`copies` section of `pins.json`.

A repository test (`borg-reborn-rs/tests/repository.rs`) checks that every
relative link between files inside a copied registry resolves.

Licences: the Angband material is under Angband's dual licence (GPLv2 /
Angband licence), as the source projects state.

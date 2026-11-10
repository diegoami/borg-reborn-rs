# Reference sources

Copies of sources that sessions read often, so they are available offline
and at a fixed version. The original repositories stay canonical, and they
keep moving. Do not edit these files: replace them with a newer copy in a
PR that updates the pin and this table.

| Path | Copied from | Commit | What is included |
| --- | --- | --- | --- |
| `c-borg/BORG_SPEC.md` | diegoami/neo-angband-tracker `docs/c-borg/` | `777eea1d9bd6c4094e87fff23af3984dd23f7edb` | The C Borg specification. Its relative links point into neo-angband-tracker |
| `c-borg/TUNABLE-BORG-PLAN.md` | same | same | Revision 3 plus *What a remake leaves out* (2026-11-08) |
| `c-borg/README.md` | same | same | Index of the C Borg documents |
| `c-manual/` | diegoami/neo-angband-tracker `docs/c-manual/` (default branch, pin `neo-angband-tracker`) | `777eea1d9bd6c4094e87fff23af3984dd23f7edb` | The complete Angband Strategy Manual (S14): every file of the directory: chapters 0–16, appendices A1–A10 and their generator, and `user-manual/`, a copy of the official manual and in-game help at `e473720` (without `thanks.rst`, as the source explains) |
| `deaths/better-borg-c/` | diegoami/better-borg-c `docs/deaths/` | `eb9a78edde6c1909b1430f6c93c3944998b774af` | Every top-level file (README, FEEDBACK, REVIEW-TEMPLATE, `index.json`, `owner-reviews.json`, `window-profile-v2.json`), and every `.md` and `.json` file at the root of each case: `report.md`, `assessment.json`, `record.json`, `build.json`, `overall-assessment.md`, owner feedback, review anchors and verification records |
| `deaths/borg-rs/` | diegoami/borg-rs `docs/deaths/` | `286215c5cbb0c37002447c530a66ad3b92014a36` | Same selection, including `discussion.json`, `owner-reviews.json`, the cohort notes and the seed reports |
| `borg-rs/FORK-CORRECTIONS.md`, `borg-rs/adoption/status.json` | diegoami/borg-rs `docs/` | see `pins.json` | What borg-rs changed in its engine tree; context only since milestone E (our tree is upstream's, docs/ENGINE.md) |
| `borg-runner/manuals/`, `borg-runner/knowledge/` | diegoami/borg-runner `manuals/`, `knowledge/` | `e829a0119617f6d0461d4458189db76adf130c4e` | What agent players learned (S15): `INSTRUCTIONS.md`, the page they read, every found line traced to `run#ask`; `INSTRUCTIONS_2.md` (depth 5–20, untested); `jev-1.md`; and each run's notebook |
| `borg-runner/records/` | diegoami/borg-runner `records/` | same | Every `.md` file: each run's `REPORT.md` and `notebook.md`, and `OVERVIEW.md`. Screens, keys, decisions and replay pages are left out: read them at the pin (`git -C vendor/borg-runner show e829a01:records/<run>/...`) |
| `zoo/DESIGN.md`, `zoo/catalogue/` | diegoami/borg-zoo | `e56d5faa8face1cecf03b3cc49f850e1b48f0d75` | Specimens, profiles and suites as of that commit |

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

`c-manual/` links its own copy of the official manual (`user-manual/`); its
source references (`src/...`, `lib/gamedata/...`) are relative to
an Angband tree: here `engine/angband/src/` and `engine/angband/lib/gamedata/`.

The retired draft strategy manual (S12) had the same convention; its copy
is in git history.

`deaths/INDEX.md` is generated from the copies and from our own
`docs/deaths/` by `scripts/refresh-sources.py`. Refresh everything with
`scripts/refresh-sources.py --update` (RULES 21); the copy rules are the
`copies` section of `pins.json`.

A repository test (`borg-reborn-rs/tests/repository.rs`) checks that every
relative link between files inside a copied registry resolves.

Licences: the Angband material is under Angband's dual licence (GPLv2 /
Angband licence), as the source projects state.

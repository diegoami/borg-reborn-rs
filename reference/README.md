# Reference: copies that follow a live tool

Copies of documents **we** wrote in other repositories whose tools still
produce or read them: the zoo's pages, borg-runner's manuals and run
notes, the death registries, and borg-rs's own records. They follow their
source at its pin (`copies` in `pins.json`, refreshed by
`scripts/refresh-sources.py --update` in its own PR, RULES 21). **Do not
edit them here.** `scripts/check-reference.py` checks that each one is
byte for byte its source's at the pin, and that nothing else is here.

Our own documents, including those moved here from our other
repositories on 2026-11-21 (the strategy manual, the C Borg documents and
fixes, the shipped core's design), are in [`docs/`](../docs/README.md);
other authors' documents (the game's manual, the original C Borg's) stay
untouched in [`engine/angband/`](../engine/angband/) and upstream. Which is
which: [docs/DOCUMENTATION-MAP.md](../docs/DOCUMENTATION-MAP.md).

## Copies

| Path | Copied from | Commit | What is included |
| --- | --- | --- | --- |
| `deaths/better-borg-c/` | diegoami/better-borg-c `docs/deaths/` | `cc4c3e43aa0a863ad2fc4adab8f930277d181900` | Every top-level file (README, FEEDBACK, REVIEW-TEMPLATE, `index.json`, `owner-reviews.json`, `window-profile-v2.json`), and every `.md` and `.json` file at the root of each case: `report.md`, `assessment.json`, `record.json`, `build.json`, `overall-assessment.md`, owner feedback, review anchors and verification records |
| `deaths/borg-rs/` | diegoami/borg-rs `docs/deaths/` | `d75e17a5ae743114c9b745934d36721800216f45` | Same selection, including `discussion.json`, `owner-reviews.json`, the cohort notes and the seed reports |
| `borg-rs/` | diegoami/borg-rs `docs/` | see `pins.json` | borg-rs's own records: what it changed in its engine tree (`FORK-CORRECTIONS.md`, `adoption/status.json`; context only since milestone E, docs/ENGINE.md) and its tooling (`harness.md`, `checkpoints.md`, `REPLAY-TOOLKIT.md`, `ENGINE-UPGRADE.md`, `C-CONSUMER-ADOPTION.md`) |
| `borg-runner/manuals/`, `borg-runner/knowledge/` | diegoami/borg-runner `manuals/`, `knowledge/` | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | What agent players learned (S15): `INSTRUCTIONS.md`, the page they read, every found line traced to `run#ask`; `INSTRUCTIONS_2.md` (depth 5–20, untested); `jev-1.md`; and each run's notebook |
| `borg-runner/records/` | diegoami/borg-runner `records/` | same | Every `.md` file: each run's `REPORT.md` and `notebook.md`, and `OVERVIEW.md`. Screens, keys, decisions and replay pages are left out: read them at the pin (`git -C vendor/borg-runner show e829a01:records/<run>/...`) |
| `borg-runner/ACCUMULATED_BEHAVIOUR.md`, `borg-runner/docs/DESIGN.md` | diegoami/borg-runner | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | How the agent players behaved, and how borg-runner works (#816) |
| `zoo/DESIGN.md`, `zoo/FINDINGS.md`, `zoo/PROFILES.md`, `zoo/catalogue/` | diegoami/borg-zoo | `1ba3a3cfc97bbde629ac187d9ccd3bf5d9eed177` | The zoo's design, what its completed experiments showed (`FINDINGS.md`, #816), what can be asked of it (`PROFILES.md`, #816), and its specimens, profiles and suites as of that commit |

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

`deaths/INDEX.md` is generated from the copies and from our own
historical registry (`docs/deaths/`, or explicit `BORG_DEATH_RECORDS_DIR`) by
`scripts/refresh-sources.py`. New reports and the shared tools belong in
[borg-cemetery](https://github.com/diegoami/borg-cemetery); historical cases
are not implicitly migrated. Refresh everything with
`scripts/refresh-sources.py --update` (RULES 21); the copy rules are the
`copies` section of `pins.json`.

A repository test (`borg-reborn-rs/tests/repository.rs`) checks that every
relative link between files inside a copied registry resolves.

Licences: the Angband material is under Angband's dual licence (GPLv2 /
Angband licence), as the source projects state.

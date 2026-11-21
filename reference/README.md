# Reference: game knowledge and the Borg's documentation

**This repository is the central place for the Borg's documentation**
(owner, 2026-11-21, #816). `reference/` holds two kinds of files:

- **originals**, kept and edited here: the strategy manual, the C Borg's
  specification and defect catalogue, the C Borg fix pages, the design
  documents of the core we ship, and borg-agent's knowledge. They were
  written in other repositories and moved here on 2026-11-21; the table
  below says from where. Edit them here, in an ordinary PR;
- **copies** of documents that another live tool generates or reads: the
  zoo's pages, borg-runner's manuals and run notes, the death registries,
  borg-rs's own records. They follow their source at its pin (`copies` in
  `pins.json`, refreshed by `scripts/refresh-sources.py --update` in its
  own PR, RULES 21). Do not edit them here.

What game knowledge is used, and by whom: [GAME-KNOWLEDGE.md](GAME-KNOWLEDGE.md).
Where every Borg repository's documentation is: [DOCUMENTATION-MAP.md](DOCUMENTATION-MAP.md).

## Originals

| Path | Moved from | At | What it is |
| --- | --- | --- | --- |
| `c-manual/` | diegoami/neo-angband-tracker `docs/c-manual/` | `dbde80762dd4` | **The Angband Strategy Manual** (S14): chapters 0–16 (with *notes for the borg*, ch. 15), appendices A1–A10 and their generator; and `user-manual/`, the official manual and in-game help at upstream `e473720` (without `thanks.rst`, as the source explains) |
| `c-borg/BORG_SPEC.md`, `c-borg/TUNABLE-BORG-PLAN.md`, `c-borg/README.md` | neo-angband-tracker `docs/c-borg/` | same | The C Borg specification; the Tunable Borg plan (revision 3 plus *What a remake leaves out*); their index |
| `c-borg/c-bugs/`, `c-borg/C-BUGS.md`, `c-borg/borg-rs-c-bugs.md` | neo-angband-tracker `docs/c-bugs/`, `docs/C-BUGS.md`, `docs/audit/c-reports/borg-rs-c-bugs.md` | same | The C Borg defect catalogue: every finding (`CBUG-0001`…) with its evidence, the catalogue index, the audit of borg-rs's C-bug list |
| `better-borg-c/` | diegoami/better-borg-c `docs/measurement/` (every `.md`), `docs/VERIFICATION.md` | `cc4c3e43aa0a` | One page per verified C Borg fix or measurement (the behaviour, why it was wrong, the fix, the evidence), the design lessons, the death-analysis workflow, the milestone write-ups, and how fixes are verified. Its raw cohorts and JSON results stay in better-borg-c (linked at that commit) |
| `borg-core/` | diegoami/borg-rs `docs/` | `8c5d000a58e9` (the vendored core's pin) | The design documents of the core we ship: the town planner (`TOWN-*.md`, the item catalogue), the trip policy, the class family, shop shortcuts, the joint-policy experiments, and the C bugs borg-rs found and fixed (`c-bugs.md`) |
| `borg-agent/` | diegoami/borg-agent (retired) `knowledge/`, `docs/POST-MORTEM.md`, `docs/KNOWLEDGE.md` | `57a480ceffdd` | Its knowledge pages (danger, dungeon, mechanics, monsters, readiness), the knowledge design and the post-mortem |

Their links to files that stayed in the source repositories were rewritten
to GitHub links at the commit above; links to files that are here are
relative. `borg-reborn-rs/tests/repository.rs` checks that they resolve.

## Copies

| Path | Copied from | Commit | What is included |
| --- | --- | --- | --- |
| `deaths/better-borg-c/` | diegoami/better-borg-c `docs/deaths/` | `46037038cb0ce3a248ea22c06f02454818dbe4b4` | Every top-level file (README, FEEDBACK, REVIEW-TEMPLATE, `index.json`, `owner-reviews.json`, `window-profile-v2.json`), and every `.md` and `.json` file at the root of each case: `report.md`, `assessment.json`, `record.json`, `build.json`, `overall-assessment.md`, owner feedback, review anchors and verification records |
| `deaths/borg-rs/` | diegoami/borg-rs `docs/deaths/` | `8c5d000a58e9ddca1f1954ab9552e505856b313f` | Same selection, including `discussion.json`, `owner-reviews.json`, the cohort notes and the seed reports |
| `borg-rs/` | diegoami/borg-rs `docs/` | see `pins.json` | borg-rs's own records: what it changed in its engine tree (`FORK-CORRECTIONS.md`, `adoption/status.json`; context only since milestone E, docs/ENGINE.md) and its tooling (`harness.md`, `checkpoints.md`, `REPLAY-TOOLKIT.md`, `ENGINE-UPGRADE.md`, `C-CONSUMER-ADOPTION.md`) |
| `borg-runner/manuals/`, `borg-runner/knowledge/` | diegoami/borg-runner `manuals/`, `knowledge/` | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | What agent players learned (S15): `INSTRUCTIONS.md`, the page they read, every found line traced to `run#ask`; `INSTRUCTIONS_2.md` (depth 5–20, untested); `jev-1.md`; and each run's notebook |
| `borg-runner/records/` | diegoami/borg-runner `records/` | same | Every `.md` file: each run's `REPORT.md` and `notebook.md`, and `OVERVIEW.md`. Screens, keys, decisions and replay pages are left out: read them at the pin (`git -C vendor/borg-runner show e829a01:records/<run>/...`) |
| `borg-runner/ACCUMULATED_BEHAVIOUR.md`, `borg-runner/docs/DESIGN.md` | diegoami/borg-runner | `e55ac6f384f2e19c0b148c2b1d2ac932fbec3d83` | How the agent players behaved, and how borg-runner works (#816) |
| `zoo/DESIGN.md`, `zoo/FINDINGS.md`, `zoo/PROFILES.md`, `zoo/catalogue/` | diegoami/borg-zoo | `7b373a702e7b9bb491ba468d97d34cf99ec3df2d` | The zoo's design, what its completed experiments showed (`FINDINGS.md`, #816), what can be asked of it (`PROFILES.md`, #816), and its specimens, profiles and suites as of that commit |

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
| borg-rs | [docs/ENGINE-UPGRADE.md](https://github.com/diegoami/borg-rs/blob/8c5d000a58e9ddca1f1954ab9552e505856b313f/docs/ENGINE-UPGRADE.md), linked from `borg-rs/FORK-CORRECTIONS.md` | The historical engine-upgrade procedure, outside the configured copy selection |
| borg-rs | `docs/validation/2026-11-07-post-m4-investigation/` (including `death-checkpoints/*.save`) | Validation runs and living saves of the cohort |
| borg-rs | `produce_death.py`, `replay-borg.py`, `replay_next_death.py` | Tools named in replay instructions |

`c-manual/` links its own copy of the official manual (`user-manual/`); its
source references (`src/...`, `lib/gamedata/...`) are relative to
an Angband tree: here `engine/angband/src/` and `engine/angband/lib/gamedata/`.

The retired draft strategy manual (S12) had the same convention; its copy
is in git history.

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

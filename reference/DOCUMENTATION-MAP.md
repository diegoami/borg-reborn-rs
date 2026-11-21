# Where the Borg documentation is

**This repository is the central place for the Borg's documentation**
(owner, 2026-11-21, #816). Documents written in the other Borg repositories
about the game, its strategy and how the Borgs work were moved here and are
**originals** now, edited here ([README.md](README.md) lists what came
from where). Documents that another live tool generates or reads stay
**copies** that follow their source at its pin (`copies` in
[`pins.json`](../pins.json), RULES 21). Documents that are only about how
another repository runs (its reviews, status logs, task lists, run
receipts) stay where they are. What the Borg itself reads, and in which
order analysis trusts the sources, is [GAME-KNOWLEDGE.md](GAME-KNOWLEDGE.md).

Inventory of 2026-11-21: every `diegoami` repository about Angband or the
Borg that this session could read.

## Brought here

| Repository | What it holds | Here | Kind |
| --- | --- | --- | --- |
| neo-angband-tracker | **The Angband Strategy Manual** and the official user manual; **the C Borg specification** and the Tunable Borg plan; **the C Borg defect catalogue** (152 findings) | [`c-manual/`](c-manual/README.md), [`c-borg/`](c-borg/README.md), [`c-borg/c-bugs/`](c-borg/c-bugs/README.md), [`c-borg/C-BUGS.md`](c-borg/C-BUGS.md), [`c-borg/borg-rs-c-bugs.md`](c-borg/borg-rs-c-bugs.md) | original |
| better-borg-c | One page per verified fix or measurement of the C Borg: the behaviour, why it was wrong, the fix and its evidence; design lessons; the death-analysis workflow; the milestone write-ups; how fixes are verified; its death dossiers | [`better-borg-c/measurement/`](better-borg-c/measurement/README.md), [`better-borg-c/VERIFICATION.md`](better-borg-c/VERIFICATION.md), [`deaths/better-borg-c/`](deaths/better-borg-c/) | original (deaths: copy) |
| borg-rs | The design documents of the core we ship: the town planner and item catalogue, the trip policy, the class family, shop shortcuts, joint-policy experiments, the C bugs it fixed; its own records (fork corrections, harness, checkpoints, replay toolkit, engine upgrade); its death dossiers | [`borg-core/`](borg-core/), [`borg-rs/`](borg-rs/), [`deaths/borg-rs/`](deaths/borg-rs/) | original (`borg-core/`); its own records and deaths: copy |
| borg-runner | What language-model players learned and how they behaved; each run's notebook and write-up; how borg-runner works | [`borg-runner/`](borg-runner/) | copy |
| borg-agent (retired) | Its knowledge pages (danger, dungeon, mechanics, monsters, readiness), the knowledge design, and the post-mortem of why it stopped | [`borg-agent/`](borg-agent/) | original |
| borg-zoo | The zoo's design, its findings, what can be asked of it, its catalogue | [`zoo/`](zoo/) | copy |
| angband/angband | The official manual, the in-game help and the C Borg's own documentation (`docs/hacking/borg.rst`, `src/borg/borg.txt`) | in-tree: [`engine/angband/docs/`](../engine/angband/docs/), and [`c-manual/user-manual/`](c-manual/user-manual/) | upstream, in-tree |

Written here (not copies): [the architecture](../docs/architecture/README.md)
of the C Borg, the shipped core and the redesign; the player's
[README](../README.md); the shipped core's settings record
[CALIBRATION.md](../docs/CALIBRATION.md); [ERRANDS.md](../docs/ERRANDS.md),
[GAME-PLAN-LANGUAGE.md](../docs/GAME-PLAN-LANGUAGE.md),
[PHASE-DOOR.md](../docs/PHASE-DOOR.md).

## Taken at

The originals were taken at the source heads of 2026-11-19/20 (neo-angband-tracker
`dbde80762dd4`, better-borg-c `cc4c3e43aa0a`, including its 30 fix pages
newer than the pin), except the core's design documents, taken at the
vendored core's pin `8c5d000a58e9` so that they describe the code we ship.
Copies follow their pins; borg-rs's pin is also the vendored core's and
moves only with a core refresh (`scripts/borg-core-vendor.py refresh`).

## Left where they are, and why

| Repository | What it holds | Why not here |
| --- | --- | --- |
| [borg-cemetery](https://github.com/diegoami/borg-cemetery) | Every recorded death, its investigation, replayable scenarios; its docs on death analysis (`death-analysis.md`, `analysis-pipeline.md`, `death-identity.md`), scenarios and the synthetic suite | The owner's rule (AGENTS.md *Gameplay reports and scenarios*): gameplay evidence and its process live in the Cemetery. Its tools are pinned by `death-tools-pin.txt` |
| [player-cemetery](https://github.com/diegoami/player-cemetery) | Recorded human and headless play | Raw recordings; the Cemetery links them |
| [angband-recorder](https://github.com/diegoami/angband-recorder) | The recorder's own spec and validation | Its own product; our integration is [ANGBAND-RECORDER.md](../docs/ANGBAND-RECORDER.md) |
| better-borg-c `docs/audit/`, borg-rs `docs/validation/` and `docs/adoption/`, zoo `reports/` | Run receipts, audits and adoption records (thousands of files) | Evidence, not documentation; read at the pin (`git -C vendor/NAME show COMMIT:path`) |
| neo-angband-tracker (the rest) | The Neo Angband engine and its mod platform: modding, packaging, parity of the TypeScript port | Another project; only its Borg and game-knowledge parts are copied |
| [neo-angband-borg-mod-tracker](https://github.com/diegoami/neo-angband-borg-mod-tracker), [neowarrior-borg](https://github.com/diegoami/neowarrior-borg) (archived) | Repairs to the TypeScript Borg port for Neo Angband; a warrior-only fork of it | A different Borg (TypeScript, Neo Angband); their docs are task and run records. neowarrior-borg names a successor, `better-borg` renamed Mandos, which this session could not see |
| angband-workspace (private) | Coordination between the repositories: session prompts, carry and review procedures | Private, and about process; not copied into this public repository |
| borg-rs, better-borg-c, borg-runner: reviews, status, progress, handovers, plans | How those projects were run | Project history, not knowledge |

## Still to do in the source repositories

The source repositories should say that these documents moved here, and
stop editing their own versions: a pointer in each moved directory (for
example neo-angband-tracker `docs/c-manual/README.md`, better-borg-c
`docs/measurement/README.md`, borg-rs's town documents, borg-agent's
`knowledge/`). That needs a change in each of those repositories; it is
not done from here.

borg-cemetery's guides (death analysis, the analysis pipeline, death
identity, scenarios, the synthetic suite) stay in the Cemetery: they
describe its own records, tools and identities, and `death-analysis.md` is
an analysis of specific deaths, which is gameplay evidence (AGENTS.md).

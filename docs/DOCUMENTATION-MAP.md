# Where the Borg documentation is: ours, and other authors'

**This repository is the central place for our Borg documentation**
(owner, 2026-11-21, #816). Documents are split by who wrote them:

- **Ours**: written in this project's repositories (`diegoami/*`) by the
  owner and the sessions working for him. They live in `docs/`, where we
  move, change and update them as we like. Those that came from another of
  our repositories were moved here on 2026-11-21 and are edited here from
  now on; their old copies elsewhere are not maintained.
- **Ours, but produced by a live tool**: the zoo's pages, borg-runner's
  manuals and run notes, the death registries and borg-rs's own records.
  They are generated or read by a tool that is still running in its own
  repository, so `reference/` keeps copies that follow it at a pin
  (`copies` in [`pins.json`](../pins.json), RULES 21) and never edits them.
  `scripts/check-reference.py` checks that every copy is its source's
  bytes.
- **Other authors'**: the game and its documentation, by the Angband
  developers, and the original C Borg and its documentation, by its
  authors. They are untouched and stay where they are: upstream, and
  in-tree byte for byte in [`engine/angband/`](../engine/angband/)
  (RULES 9). We cite them; we do not copy, move or edit them.

Inventory of 2026-11-21: every `diegoami` repository about Angband or the
Borg that this session could read.

## Ours, in `docs/`

| Here | What | Came from |
| --- | --- | --- |
| [strategy/](strategy/README.md) | **The Angband Strategy Manual**: how the game works underneath and how to survive it, every number cited to the engine; chapters 0–16 (*notes for the borg* in 15), appendices A1–A10 and their generator | neo-angband-tracker `docs/c-manual/` at `dbde80762dd4` |
| [c-borg/](c-borg/README.md) | **The C Borg**: its specification, the Tunable Borg plan and its review prompt, the defect catalogue ([c-bugs/](c-borg/c-bugs/README.md), 152 findings; [C-BUGS.md](c-borg/C-BUGS.md)), the audit of borg-rs's C-bug list | neo-angband-tracker `docs/c-borg/`, `docs/c-bugs/`, `docs/C-BUGS.md`, `docs/audit/c-reports/borg-rs-c-bugs.md` at `dbde80762dd4` |
| [c-borg/fixes/](c-borg/fixes/README.md) | **The C Borg's verified fixes**: one page per fix or measurement (the behaviour, why it was wrong, the fix, the evidence), design lessons, the death-analysis workflow, milestone write-ups, [how fixes are verified](c-borg/fixes/VERIFICATION.md) | better-borg-c `docs/measurement/` (pages) and `docs/VERIFICATION.md` at `cc4c3e43aa0a` |
| [design/](#design-the-shipped-cores-design-documents) | **The shipped core's design**: the town planner, the trip policy, the class family, shop shortcuts, joint-policy trials, the C bugs borg-rs kept or fixed | borg-rs `docs/` at the vendored core's pin `8c5d000a58e9` |
| [research/borg-agent/](research/borg-agent/knowledge/README.md) | borg-agent's knowledge pages (danger, dungeon, mechanics, monsters, readiness), its knowledge design and post-mortem | borg-agent (retired) at `57a480ceffdd` |
| [architecture/](architecture/README.md) | How the C Borg, the shipped core and the redesign work | written here |
| [README](../README.md), [CALIBRATION.md](CALIBRATION.md), [PHASE-DOOR.md](PHASE-DOOR.md), [GAME-KNOWLEDGE.md](GAME-KNOWLEDGE.md), ... | Playing the Borg, its settings, single decisions, what knowledge is used | written here ([docs/README.md](README.md)) |

When moved, the documents kept their text; only their links were changed
to the new places, and links to files that stayed behind (data, receipts,
other projects' pages) lead to that repository at the commit above. The
official manual the strategy manual cites is the game's own, in
[`engine/angband/docs/`](../engine/angband/docs/). The source of each move
is also recorded in [PROVENANCE.md](PROVENANCE.md).

### design/: the shipped core's design documents

| Document | What |
| --- | --- |
| [TOWN-INVENTORY.md](design/TOWN-INVENTORY.md) | The town inventory strategy: its architecture |
| [TOWN-INVENTORY-PLAN.md](design/TOWN-INVENTORY-PLAN.md), [TOWN-INVENTORY-B2.md](design/TOWN-INVENTORY-B2.md), [TOWN-INVENTORY-PHASE4.md](design/TOWN-INVENTORY-PHASE4.md) | Its implementation plan, the kind-only alternatives, the Stay rules and the plan record |
| [TOWN-UTILITY.md](design/TOWN-UTILITY.md) | Trip-relative town utility (model v5) |
| [TOWN-ITEM-CATALOGUE.md](design/TOWN-ITEM-CATALOGUE.md) | Every item kind the town planner knows, and how it is valued |
| [TOWN-BASELINE.md](design/TOWN-BASELINE.md), [TOWN-CAPABILITIES.md](design/TOWN-CAPABILITIES.md), [TOWN-CLASS-CALIBRATION.md](design/TOWN-CLASS-CALIBRATION.md) | The baseline of inventory choices, capability coverage, class profiles and their calibration |
| [TOWN-LIVE.md](design/TOWN-LIVE.md), [TOWN-SESSION.md](design/TOWN-SESSION.md), [TOWN-DUMP.md](design/TOWN-DUMP.md) | The live town executor, the interactive session, the town-visit dump |
| [TRIP-POLICY.md](design/TRIP-POLICY.md) | When a trip ends and the Borg goes home |
| [JOINT-POLICY-OPTIMISATION.md](design/JOINT-POLICY-OPTIMISATION.md), [JOINT-POLICY-TRIALS.md](design/JOINT-POLICY-TRIALS.md) | Trip and shopping settings chosen together; coming home and preparing the next trip |
| [BORG-SHOP-SHORTCUTS.md](design/BORG-SHOP-SHORTCUTS.md) | The shortcuts the Borg takes in shops |
| [FAMILY.md](design/FAMILY.md) | The family of Borg versions after impact v10 |
| [c-bugs.md](design/c-bugs.md) | The C Borg bugs borg-rs reproduces faithfully, and the ones it fixed |

## Ours, produced by live tools: copies in `reference/`

| Here | What | Source |
| --- | --- | --- |
| [`reference/zoo/`](../reference/zoo/) | The zoo's design, findings, request guide, catalogue | borg-zoo |
| [`reference/borg-runner/`](../reference/borg-runner/) | What agent players learned and how they behaved; run notes; how borg-runner works | borg-runner |
| [`reference/deaths/`](../reference/deaths/INDEX.md) | The C Borg's and borg-rs's death dossiers, and the index of every death | better-borg-c, borg-rs |
| [`reference/borg-rs/`](../reference/borg-rs/) | borg-rs's own records: its engine fork corrections and adoption status, its harness, checkpoints, replay toolkit, engine upgrade | borg-rs |

Pins and selections: [reference/README.md](../reference/README.md).

## Other authors': untouched, where they are

| What | Authors | Where |
| --- | --- | --- |
| Angband 4.2.6 itself, its data files | the Angband developers | upstream [angband/angband](https://github.com/angband/angband); in-tree at [`engine/angband/`](../engine/angband/), byte for byte (RULES 9) |
| The official user manual and in-game help, including *A Players' Guide to Angband* | the Angband developers | [`engine/angband/docs/`](../engine/angband/docs/), and [angband.readthedocs.io](https://angband.readthedocs.io/) |
| The original C Borg and its documentation (`src/borg/`, `docs/hacking/borg.rst`, the annotated `src/borg/borg.txt`; the old APW readme `borgread.txt` at tag 3.5.1) | its authors (Ben Harrison, Andrew P. White and later maintainers) | in-tree at [`engine/angband/src/borg/`](../engine/angband/src/borg/) and [`engine/angband/docs/hacking/borg.rst`](../engine/angband/docs/hacking/borg.rst); the old readme in upstream's history |
| Community guides (forum threads on the 4.2 classes, StrategyWiki, older FAQs) | their authors | on their sites; surveyed in [GAME-KNOWLEDGE.md](GAME-KNOWLEDGE.md) §3, not copied |

## Left in our other repositories, and why

| Repository | What it holds | Why not here |
| --- | --- | --- |
| [borg-cemetery](https://github.com/diegoami/borg-cemetery) | Recorded deaths, investigations, replayable scenarios; its guides on death analysis, death identity, scenarios and the synthetic suite | Gameplay evidence and its process live in the Cemetery (AGENTS.md); its guides describe its own records and tools, and `death-analysis.md` analyses specific deaths |
| [player-cemetery](https://github.com/diegoami/player-cemetery) | Recorded human and headless play | Raw recordings; the Cemetery links them |
| [angband-recorder](https://github.com/diegoami/angband-recorder) | The recorder's spec and validation | Its own product; our integration is [ANGBAND-RECORDER.md](ANGBAND-RECORDER.md) |
| better-borg-c, borg-rs, the zoo: audits, validation runs, adoption records, reports | Receipts and data (thousands of files) | Evidence, not documentation; read at the pin |
| neo-angband-tracker (the rest) | The Neo Angband engine and its mod platform | Another project |
| [neo-angband-borg-mod-tracker](https://github.com/diegoami/neo-angband-borg-mod-tracker), [neowarrior-borg](https://github.com/diegoami/neowarrior-borg) (archived) | Repairs to the TypeScript Borg port; a warrior-only fork of it | A different Borg; task and run records. neowarrior-borg names a successor, `better-borg` renamed Mandos, which this session could not see |
| angband-workspace (private) | Coordination between the repositories | Private, and about process |
| Reviews, status logs, handovers, plans of the other repositories | How those projects were run | Project history |

## Still to do in the other repositories

The repositories the documents came from should say they moved here (a
pointer in neo-angband-tracker `docs/c-manual/` and `docs/c-borg/`,
better-borg-c `docs/measurement/`, borg-rs's town documents, borg-agent's
`knowledge/`). That is a change in each of them, not made from here.

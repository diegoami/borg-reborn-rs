# docs/

The repository's documents, by audience. The [README](../README.md) is the
front door: how to get the game with the Borg, and where to read next.
Playing the Borg continues in the [play guide](user/GUIDE.md); helping with
the project in the [contributor guide](contributing/README.md); a session
working on the repository starts at [AGENTS.md](../AGENTS.md).
Everything here is **ours**: written in this project, here or in our other
repositories, and ours to change. Other authors' documents (the game's
manual, the original C Borg's) stay untouched in `engine/angband/`;
copies that a live tool still produces are in `reference/`. Which is which:
[DOCUMENTATION-MAP.md](DOCUMENTATION-MAP.md). Reorganised on 2026-11-21
(#816).

## The guides (#817)

| Document | What |
| --- | --- |
| [user/GUIDE.md](user/GUIDE.md) | **Layer 1, play**: install, watch, the Borg's commands, its settings, headless games, recording, deaths and measurement, the family of projects |
| [contributing/](contributing/README.md) | **Layer 2, contribute**: how we work, building and testing, changing how the Borg plays, investigating a game, measuring with the zoo, the related repositories |

## The game and the Borgs

| Document | What |
| --- | --- |
| [strategy/](strategy/README.md) | **The Angband Strategy Manual**: how the game works underneath and how to survive it, every number cited to the engine; *notes for the borg* in chapter 15 |
| [c-borg/](c-borg/README.md) | **The C Borg**: its specification, the Tunable Borg plan, the defect catalogue ([c-bugs/](c-borg/c-bugs/README.md)), and its verified fixes ([fixes/](c-borg/fixes/README.md)) |
| [design/](DOCUMENTATION-MAP.md#design-the-shipped-cores-design-documents) | **The shipped core's design**: the town planner, the trip policy, the class family, shop shortcuts |
| [GAME-KNOWLEDGE.md](GAME-KNOWLEDGE.md) | What game knowledge the Borg reads while playing, what analysis reads and in which order, and the survey of strategy guides |
| [research/](research/KNOWLEDGE.md) | Digests of the moving sources; [borg-agent's knowledge](research/borg-agent/knowledge/README.md) |

## Using the Borg

| Document | What |
| --- | --- |
| [CALIBRATION.md](CALIBRATION.md) | Every named setting (`borg_param_*`) of the shipped core: default, unit, range, the code it reaches, the evidence |
| [PHASE-DOOR.md](PHASE-DOOR.md) | Where the core decides to use Phase Door, and the settings it reads |
| [GUIDED-DEBUGGING.md](GUIDED-DEBUGGING.md) | The owner's workflow for a recorded game with a problem |
| [ANGBAND-RECORDER.md](ANGBAND-RECORDER.md) | The optional game recorder |

## Architecture

| Document | What |
| --- | --- |
| [architecture/](architecture/README.md) | The original C Borg, the borg-core we ship, the reborn plan (tracks P, R, Q), the layered design that is track R's target |
| [ENGINE.md](ENGINE.md), [ENGINE-READS.md](ENGINE-READS.md) | The engine integration (upstream byte for byte, our patches, frontend, bindings), and every engine symbol the reborn core reads |
| [CORE-TRACE.md](CORE-TRACE.md) | The shipped core's decision trace (`trace-core.jsonl`) |
| [DECISION-FIXTURES.md](DECISION-FIXTURES.md) | The recipe for a decision fixture: driving `borg_think_dungeon` on a fabricated state (the #1173/#1174 traps) |
| [ATTACK-PREFERENCES.md](ATTACK-PREFERENCES.md), [M13-INVENTORY-PARAMETERS.md](M13-INVENTORY-PARAMETERS.md) (and its [JSON inventory](M13-INVENTORY-PARAMETERS.json)) | Two parameter contracts of track P |
| [TOWN-PLAN.md](TOWN-PLAN.md) | The town plan's design and its record formats |
| [QUERY-SURFACE.md](QUERY-SURFACE.md) | Track Q's design (draft) |

## Contracts and process

| Document | What |
| --- | --- |
| [SPEC.md](SPEC.md) | The top-level contract |
| [GOALS.md](GOALS.md) | What we build and the owner's decisions |
| [SOURCES.md](SOURCES.md), [PROVENANCE.md](PROVENANCE.md) | What every related project is for here; what was copied from where |
| [ZOO-INTERFACE.md](ZOO-INTERFACE.md) | The contract borg-zoo's adapter calls |
| [VALIDATION-RECEIPTS.md](VALIDATION-RECEIPTS.md), [REFERENCE-RECEIPTS.md](REFERENCE-RECEIPTS.md), [DOSSIER-IMPORT.md](DOSSIER-IMPORT.md) | How validation evidence is recorded and imported |
| [CORE-SWEEP.md](CORE-SWEEP.md) | Track P's owner-run sweep prompt |
| [handover/](handover/README.md), [HANDOVER.md](HANDOVER.md) | Where the work stands, one note per task; the older archive |
| [examples/](examples/) | Example calibration catalogues and profiles |

## Records

| Directory | What |
| --- | --- |
| [validation/](validation/README.md) | Per-PR validation receipts and dossier maps, with a map of their old paths |
| [reviews/](reviews/) | Posted reviews |
| [deaths/](deaths/POINTER.md) | The historical death registry (new deaths go to borg-cemetery) |
| [archive/](archive/README.md) | Documents that no longer apply, each with when and why |
| [FIXTURES.md](FIXTURES.md), [M4-CALIBRATION.md](M4-CALIBRATION.md) | The frozen reborn core's decision fixtures (read by its tests) and its M4 calibration |

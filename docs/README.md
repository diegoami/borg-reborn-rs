# docs/

The repository's documents, by audience. Playing the Borg starts at the
[README](../README.md); working on the repository at [AGENTS.md](../AGENTS.md).
Game knowledge (the manuals, the C Borg's documentation, what is used by
whom) is in [reference/](../reference/GAME-KNOWLEDGE.md). Reorganised on
2026-11-21 (#816).

## Using the Borg

| Document | What |
| --- | --- |
| [ERRANDS.md](ERRANDS.md) | Errands: giving the core a goal (`/town`, `/dive N`, `/ladder`, `/stopscared`, ...) in the game, in `borg.txt` or in the environment |
| [GAME-PLAN-LANGUAGE.md](GAME-PLAN-LANGUAGE.md) | Plans: depth caps by class and character level |
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
| [research/](research/KNOWLEDGE.md) | Digests of the moving sources |
| [examples/](examples/) | Example calibration catalogues and profiles |

## Records

| Directory | What |
| --- | --- |
| [validation/](validation/README.md) | Per-PR validation receipts and dossier maps, with a map of their old paths |
| [reviews/](reviews/) | Posted reviews |
| [deaths/](deaths/POINTER.md) | The historical death registry (new deaths go to borg-cemetery) |
| [archive/](archive/README.md) | Documents that no longer apply, each with when and why |
| [FIXTURES.md](FIXTURES.md), [M4-CALIBRATION.md](M4-CALIBRATION.md) | The frozen reborn core's decision fixtures (read by its tests) and its M4 calibration |

# Architecture

How the Borg in this repository plays, where it comes from and where it is
going. Read in this order:

| Document | What it describes |
| --- | --- |
| [original-c-borg.md](original-c-borg.md) | **The original C Borg** (Angband's `src/borg/`, as fixed in better-borg-c): the think loop, danger, flows, caution and escapes, the attack choice, town shopping, `borg.txt`; its files and how they fit. Cites the better-borg-c pin and [the C Borg specification](../c-borg/BORG_SPEC.md) |
| [borg-core.md](borg-core.md) | **The borg-core we ship**: borg-rs's line-by-line Rust translation of the C Borg, vendored at its pin in `borg-core/`. The module map, the parameter registry (`params.rs`, [CALIBRATION.md](../CALIBRATION.md)), the decision trace (`trace-core.jsonl`), what we add around the core (the errands were retired, #1339), the hooks we declare, and how a setting reaches the code |
| [reborn-plan.md](reborn-plan.md) | **Borg reborn, the plan**: track P (calibrate the shipped core), track R (re-express its decisions in our own domain behind a shadow check), track Q (the query surface), and how the tracks hand over |
| [reborn-layers.md](reborn-layers.md) | The layered design (L0 engine boundary … L6 executor) of our own frozen reborn core, kept as the **target of track R** |

The documents they build on are kept in [reference/](../../reference/README.md):
the C Borg's specification, defect catalogue and fix pages, the shipped
core's design documents ([docs/design/](../design/)),
and the strategy manual. The contracts these documents answer to are [SPEC.md](../SPEC.md) (top
level) and [GOALS.md](../GOALS.md) (the owner's decisions). The engine
integration under all of it is [ENGINE.md](../ENGINE.md).

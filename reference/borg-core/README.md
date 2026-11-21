# The shipped core's design documents

The design documents of borg-rs's core, the one this repository vendors in
`borg-core/` and ships. They were written in borg-rs (`docs/`) and moved
here on 2026-11-21 (#816), taken at the vendored core's pin `8c5d000a58e9`,
so they describe the code we ship. Originals now: edit them here. How the
core fits together, and what we added to it, is
[docs/architecture/borg-core.md](../../docs/architecture/borg-core.md).

| Document | What |
| --- | --- |
| [TOWN-INVENTORY.md](TOWN-INVENTORY.md) | The town inventory strategy: its architecture |
| [TOWN-INVENTORY-PLAN.md](TOWN-INVENTORY-PLAN.md), [TOWN-INVENTORY-B2.md](TOWN-INVENTORY-B2.md), [TOWN-INVENTORY-PHASE4.md](TOWN-INVENTORY-PHASE4.md) | Its implementation plan, the kind-only alternatives, and the Stay rules with the plan record |
| [TOWN-UTILITY.md](TOWN-UTILITY.md) | Trip-relative town utility (model v5) |
| [TOWN-ITEM-CATALOGUE.md](TOWN-ITEM-CATALOGUE.md) | Every item kind the town planner knows, and how it is valued |
| [TOWN-BASELINE.md](TOWN-BASELINE.md), [TOWN-CAPABILITIES.md](TOWN-CAPABILITIES.md), [TOWN-CLASS-CALIBRATION.md](TOWN-CLASS-CALIBRATION.md) | The baseline of inventory choices, capability coverage, class profiles and their calibration |
| [TOWN-LIVE.md](TOWN-LIVE.md), [TOWN-SESSION.md](TOWN-SESSION.md), [TOWN-DUMP.md](TOWN-DUMP.md) | The live town executor, the interactive session (the Borg plays the dungeon, the player the town), the town-visit dump |
| [TRIP-POLICY.md](TRIP-POLICY.md) | When a trip ends and the Borg goes home |
| [JOINT-POLICY-OPTIMISATION.md](JOINT-POLICY-OPTIMISATION.md), [JOINT-POLICY-TRIALS.md](JOINT-POLICY-TRIALS.md) | Choosing trip and shopping settings together, and the trials of coming home and preparing the next trip |
| [BORG-SHOP-SHORTCUTS.md](BORG-SHOP-SHORTCUTS.md) | The shortcuts the Borg takes in shops |
| [FAMILY.md](FAMILY.md) | The family of Borg versions after impact v10 |
| [c-bugs.md](c-bugs.md) | The C Borg bugs borg-rs reproduced faithfully, and the ones it fixed |

Some documents name borg-rs's own files, branches or experiments; their
links lead to borg-rs at the commit above. Settings that track P has since
named are in [CALIBRATION.md](../../docs/CALIBRATION.md).

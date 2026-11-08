# Rust Borg death records

Every confirmed natural death gets an immutable source/build/input identity, original observations, an assessment and linked actions. Full raw runs/saves/builds stay outside Git. Death alone does not establish a bug.

Dossiers: 6; distinct starting conditions: 6; retained process attempts: 14 (including repeats). This registry is not a survival-rate estimate.

| Case | Discussion | Actual source | Seed | Raw cause | Death turn | Diagnosis |
| --- | --- | --- | --- | --- | --- | --- |
| [D0001](D0001-seed8400-warrior/report.md) | unassigned | `f772eb6dcb93` | 8400 | a pseudo-dragon | 37680 | C/Rust parity observed; defect not established |
| [D0002](D0002-seed8402-druid/report.md) | unassigned | `f772eb6dcb93` | 8402 | a cutpurse | 5590 | C/Rust parity observed; defect not established |
| [D0003](D0003-seed8406-rogue/report.md) | unassigned | `f772eb6dcb93` | 8406 | a giant yellow centipede | 5600 | C/Rust parity observed; defect not established |
| [D0004](D0004-seed8408-blackguard/report.md) | unassigned | `f772eb6dcb93` | 8408 | a wild cat | 6231 | C/Rust parity observed; defect not established |
| [D0005](D0005-recorded/report.md) | [Discuss](https://github.com/diegoami/borg-rs/issues/3) | `9881301ceaad` | 1532917772 | Fang, Farmer Maggot's Dog | 36289 | Fang death assessed; recovery/retreat gates and spell-confidence causality unresolved |
| [D0006](D0006-recorded/report.md) | [Discuss](https://github.com/diegoami/borg-rs/issues/3) | `aa4ad6cfeea2` | 2647001198 | a grey mold | 22841 | Grey-mold stair/exploration loop assessed; qualified private trace shows emergency movement and danger-cache reuse |

## Aggregated actions

- Prioritize escape-stock/restocking and the decision to descend. A controlled rerun from original birth is needed before attributing survival to another policy; loading the prior engine save starts fresh Borg memory. — recorded; needs causal investigation; cases D0001.
- Investigate avoiding adjacent lethal encounters with exhausted mana and minimal starting resources. No port-specific defect or guaranteed unused known escape is established. — recorded; needs causal investigation; cases D0002.
- Investigate emergency unknown-scroll policy and earlier avoidance. The actual hidden kind is diagnostic information, not knowledge available to the Borg. — recorded; needs causal investigation; cases D0003.
- Prioritize a controlled healing-versus-finishing-attack counterfactual. Last-checkpoint stock alone is not proof a particular alternative would survive; no tactical change is made from this observation. — recorded; needs causal investigation; cases D0004.
- [D0005: observe exact recovery, retreat/stairs and competing attack gates on original source before any change](https://github.com/diegoami/borg-rs/issues/4) — open; next discriminating evidence, no gameplay change; cases D0005.
- [D0005: compare separately a specified CLW and legal NW diagnostic branch after private-gate qualification](https://github.com/diegoami/borg-rs/issues/4) — provisional; not executed, depends on exact gate trace; cases D0005.
- [D0005: qualify simultaneous native/Borg spell confidence and coordinate existing ambient-light296 through C consumer](https://github.com/diegoami/borg-rs/issues/4) — open; existing source lead, no duplicate C report or immediate adoption; cases D0005.
- [D0006: construct legal native mode-transition cache scenario with emergency/safe/blocked controls; coordinate existing C ownership before any correction](https://github.com/diegoami/borg-rs/issues/5) — open; next discriminating evidence, no gameplay change; cases D0006.
- [D0006: separately test stair descent/hold/door-close and one adjacent mace attempt with declared source/RNG/continuation limits](https://github.com/diegoami/borg-rs/issues/5) — provisional; no alternate-action branch run, distinguish serialized human branch from uninterrupted private-state continuation; cases D0006.
- [D0006: explain actual CSW-sale and fuel/escape reserve choices from native inventory-power/shop score trace](https://github.com/diegoami/borg-rs/issues/5) — open; evidence before policy change; cases D0006.

[Earlier nine-class cohort and assessments](cohort-2026-11-07.md) · [Feedback process](FEEDBACK.md).

Export with tools/record-death.py, curate assessment.json/report.md, regenerate with tools/aggregate-deaths.py. Original-C defect leads go to neo-angband-tracker under the repository policy; no automatic issues or gameplay changes are made by the logger.

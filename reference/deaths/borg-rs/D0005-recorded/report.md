# D0005: Fang, Farmer Maggot's Dog at turn 36289

[Discuss this death and add owner feedback](https://github.com/diegoami/borg-rs/issues/3).

See record.json/build.json for immutable identity and original-frame windows. Full runs, living saves and builds remain in the linked recording bundle.

## Evaluation and diagnosis

[Full structured assessment](overall-assessment.md) · [decision timeline](decision-timeline.csv)
· [qualified supplemental evidence](analysis-evidence.json) · [analysis anchors](analysis-anchors/README.md).

[Final response to owner feedback](https://github.com/diegoami/borg-rs/issues/3#issuecomment-6713687354)
and [investigation update](https://github.com/diegoami/borg-rs/issues/4#issuecomment-6713684066).

The Elf Necromancer (CL2, HP15/MP4 maximum) met Fang at36094, spent its last
Phase Door **scroll** at36114, attempted walking/stair return, and died at36289.
One known Cure Light Wounds remained unused. At the final tile(191,28), NW to
(190,27) was legal and had actually been traversed earlier; a complete safe
route to upstairs is unproved. The tail has four Nether Bolt attempts (one
hit, three failures), retreats and five melees, not continuous spell spam.

The historical native observation replay matches all original channels and
native save boundaries, including an observer-disabled control. Supplemental
spell readiness shows Nether Bolt costs1MP with36% native failure while
affordable. The terminal Borg estimate is11%, consistent with existing
[source#296](https://github.com/diegoami/neo-angband-tracker/issues/296), but
predecision private gates/scores were not captured. Native readiness is not
Borg eligibility. Recovery/retreat admission and action priority are supported
source mechanisms to investigate, not proved rescue or a Rust-only defect.

## Action items

[Focused Rust investigation#4](https://github.com/diegoami/borg-rs/issues/4)
tracks exact private recovery/route/spell gate observations and isolated
diagnostic alternatives. Related C consumer healing/return investigations are
linked there; no duplicate canonical C finding or gameplay change is made.

The observational assessment answers the
[owner feedback](https://github.com/diegoami/borg-rs/issues/3#issuecomment-6713459074).
Causal attribution and safe alternatives remain provisional. Original
record/build/snapshots/raw-bundle identity and M5 remain unchanged; any future
shared fix follows the accepted C-consumer milestone policy.

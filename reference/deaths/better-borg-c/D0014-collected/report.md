# D0014: a blood falcon at turn 306771

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/341) · [Feedback process](../FEEDBACK.md)

See record.json and build.json for immutable source/build/seed/options.

## Evaluation and diagnosis

See the [overall assessment](overall-assessment.md). Three Deep Descent reads took this CL9
Rogue from DL2 to DL20. Digging there, it spent its last Phase Door at full HP and landed beside
an orc archer and a fire spirit. A speed +30 blood falcon then killed it, while it still carried
21 Cure Light Wounds and 5 Potions of Speed that its emergency rules would not use. These are
source-checked mechanisms with native evidence. The owner's verified manual branch from the Phase decision survived
with one Speed potion, five CLW and tactical Detect Monsters. No reusable original-C defect or
policy benefit is established.

## Action items

- [183: destination-aware Deep Descent](https://github.com/diegoami/better-borg-c/issues/183)
- [321: climb-to-restock failure](https://github.com/diegoami/better-borg-c/issues/321)
- [340: Speed potion rule](https://github.com/diegoami/better-borg-c/issues/340)
- [34: healing eligibility](https://github.com/diegoami/better-borg-c/issues/34)
- [159: walking retreat](https://github.com/diegoami/better-borg-c/issues/159)

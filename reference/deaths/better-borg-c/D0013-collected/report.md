# D0013: a small kobold at turn 3716

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/332) · [Feedback process](../FEEDBACK.md)

See record.json and build.json for immutable source/build/seed/options.

## Evaluation and diagnosis

See the [overall assessment](overall-assessment.md). A CL1 Druid sets off a siren trap and
steps into a doorway next to a floating eye, which paralyses it. It then spends its only Phase
Door where a single step back would have done. A hard-coded scary rule makes one small kobold
a reason to flee the level, and the kobold and an ant corner it. It had bought rings instead
of cures. These are source-checked mechanisms with native position evidence. The owner's verified manual
branch from 3651 survived by beating the kobold, testing the ant and fleeing to town. No
reusable original-C defect or policy benefit is established.

## Action items

- [159: walking retreat versus escape](https://github.com/diegoami/better-borg-c/issues/159)
- [327: scary-list triggers](https://github.com/diegoami/better-borg-c/issues/327)
- [41: survival stock](https://github.com/diegoami/better-borg-c/issues/41)

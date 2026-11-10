# D0018: a wild cat at turn 33471

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/401) · [Feedback process](../FEEDBACK.md)

See record.json and build.json for immutable source/build/seed/options.

## Evaluation and diagnosis

See the [overall assessment](overall-assessment.md). A CL2 Druid on source `5cd2258dd78d` spent
its gold in two town visits on armour, sandals, a Speed potion and unknown jewellery, bought one
Phase Door each time and no Cure Light Wounds, and left its Recall at Home. The bored/twitchy
fallback read its only Phase Door at low danger. On a level it was fleeing as "scary", it met a
speed +8 wild cat, hit it once, then walked away for eight steps while the cat clawed it to death.
It was carrying two Cure Light Wounds it had never identified, and the last-ditch unknown-potion
path excludes characters with mana. The owner's verified manual retry from 33363 also died. These
are source-checked mechanisms with native evidence. No reusable original-C defect or rescue is
established.

## Action items

- [41: survival stock and keeping the last escape](https://github.com/diegoami/better-borg-c/issues/41)
- [154: Phase Door and cure before early dives](https://github.com/diegoami/better-borg-c/issues/154)
- [159: walking retreat from a faster monster](https://github.com/diegoami/better-borg-c/issues/159)
- [180: identification and unknown-potion emergency use](https://github.com/diegoami/better-borg-c/issues/180)
- [183: Recall retention](https://github.com/diegoami/better-borg-c/issues/183)
- [340: Speed potion rule](https://github.com/diegoami/better-borg-c/issues/340)

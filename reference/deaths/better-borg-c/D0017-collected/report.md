# D0017: poison at turn 1094860

[Discuss this death](https://github.com/diegoami/better-borg-c/issues/403) · [Feedback process](../FEEDBACK.md)

See record.json and build.json for immutable source/build/seed/options.

## Evaluation and diagnosis

See the [overall assessment](overall-assessment.md). A CL25 Ranger on DL21, on source
`5cd2258dd78d`, carrying 9 Cure Critical Wounds, 20 Speed, 13 Phase Door, a Staff of Teleportation
and Herbal Curing, was found resting by Mirkwood spiders (speed +10, poison bites). It walked away
for ten turns with 3–4 of them adjacent, from 201 to 9 HP. Escape is evaluated only after the
retreat finds no step, and the CCW healing tier is a narrow 50–55% window that failed on an
integer tie. At 7 HP the staff got it out. With fleeing set, caution then skipped healing and it
rested for Recall while poisoned, dying at 1 HP per turn with 9 CCW in the pack. These are
source-checked mechanisms with native evidence. No reusable original-C defect or rescue is
established.

## Action items

- [34: healing gated off while fleeing](https://github.com/diegoami/better-borg-c/issues/34)
- [159: retreat pre-empting escape; walking retreat from faster monsters](https://github.com/diegoami/better-borg-c/issues/159)
- [319: healing tier windows](https://github.com/diegoami/better-borg-c/issues/319)
- [340: Speed potion rule](https://github.com/diegoami/better-borg-c/issues/340)

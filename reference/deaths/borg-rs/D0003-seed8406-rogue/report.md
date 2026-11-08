# Natural death: seed8406 Hobbit Rogue

Source: post-M4 #345 investigation, normal-live suite. Actual build identities
are in ../../validation/2026-11-07-post-m4-investigation/{c,rust}-BUILD_ID.
Fresh ordinary birth; no restored character, wizard kit, cheat death or
cheat-live. C twice and Rust have identical key/message/player-state traces.
This is one distinct input/death, not three deaths from the verification runs.

Result: a giant yellow centipede; game turn5600, clevel2,
final current depth1, deepest logged current depth1.
The final max_depth field is not treated as a lifetime peak.

## Observations

At1/11HP with two adjacent threats it read an unknown scroll. Diagnostic identity and subsequent messages identify Deep Descent; it gained level2 but was bitten by a rock lizard and giant yellow centipede before escape.

Last living engine checkpoint: turn5002,
SHA256 06e80ceaac56c4c31c3abfef06d08b052cf7266719f940e3a4f43f24b69fab5f. The curated save is
../../validation/2026-11-07-post-m4-investigation/death-checkpoints/live-rogue.save.
Actual inventory with awareness flags is in death-inventories.json in the
same receipt. Spoiler kind names were inspected after native load in a
separate process and were never fed to the playing Borg.

## Evaluation and next action

Investigate emergency unknown-scroll policy and earlier avoidance. The actual hidden kind is diagnostic information, not knowledge available to the Borg.

No causal fix or survival improvement is claimed. The full seeded history
can be rerun; an engine checkpoint does not restore prior private Borg state.
No new original-C defect is established solely by this mortality observation.

## Final events

```text
851 5596 0 # HIT_BY:The rock lizard (3)
851 5596 0 # MISS_BY:The giant yellow centipede (3)
851 5596 0 # HIT_BY:The giant yellow centipede (3)
851 5596 0 # Possibility of being surrounded (monsters/safegrids)(2/0)
851 5596 0 # Loc:26,154 Dep:1 Lev:1 HP:1/11 SP:0/0 Danger:p=14
851 5596 0 # Reading unknown scroll Scroll titled "citas ca".
853 5596 25 The air around you starts to swirl...
853 5596 7 Welcome to level 2.
853 5596 0 You have no more Scrolls of Deep Descent (c).
854 5599 39 The rock lizard bites you.
854 5600 39 The giant yellow centipede bites you.
855 5600 8 You die.
856 5600 0 Score not registered for borgs.
856 5600 0 # Aborting (death).
856 5600 0 # Aborting (player died).
```


Legacy dossier: original movies were not captured. See immutable record.json and the original cohort receipt; no new replay or life is claimed.

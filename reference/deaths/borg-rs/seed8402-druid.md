# Natural death: seed8402 Half-Elf Druid

Source: post-M4 #345 investigation, normal-live suite. Actual build identities
are in ../validation/2026-11-07-post-m4-investigation/{c,rust}-BUILD_ID.
Fresh ordinary birth; no restored character, wizard kit, cheat death or
cheat-live. C twice and Rust have identical key/message/player-state traces.
This is one distinct input/death, not three deaths from the verification runs.

Result: a cutpurse; game turn5590, clevel1,
final current depth1, deepest logged current depth1.
The final max_depth field is not treated as a lifetime peak.

## Observations

At the last logged decision it had0/12HP and0/2SP, estimated danger346, a scary-level/fleeing goal and an adjacent cutpurse. It attacked with a Whip and died to the next hit. The earlier snapshot has an unknown Speed potion, not known guaranteed escape stock.

Last living engine checkpoint: turn5002,
SHA256 9a05277fbb400aa736c2926fe3d4a2ee5b65e20a86d9e1db5e4d789e7fd7a653. The curated save is
../validation/2026-11-07-post-m4-investigation/death-checkpoints/live-druid.save.
Actual inventory with awareness flags is in death-inventories.json in the
same receipt. Spoiler kind names were inspected after native load in a
separate process and were never fed to the playing Borg.

## Evaluation and next action

Investigate avoiding adjacent lethal encounters with exhausted mana and minimal starting resources. No port-specific defect or guaranteed unused known escape is established.

No causal fix or survival improvement is claimed. The full seeded history
can be rerun; an engine checkpoint does not restore prior private Borg state.
No new original-C defect is established solely by this mortality observation.

## Final events

```text
852 5582 0 # HIT_BY:The cutpurse (3)
852 5582 0 # MISS_BY:The cutpurse (3)
852 5582 0 # Scary guy on level.
852 5582 0 # Loc:34,100 Dep:1 Lev:1 HP:0/12 SP:0/2 Danger:p=346
852 5582 0 # Fleeing and leaving the level. (scaryguy)
852 5582 0 # Adjacent to monster on Scaryguy Level.
852 5582 0 # Performing attack type 1 with value 469.
852 5582 0 # Facing cutpurse at (35,101) who has 13 Hit Points.
852 5582 0 # Attacking with weapon 'Whip (1d3) (+0,+0)'
854 5582 2 You hit the cutpurse.
854 5590 34 The cutpurse hits you.
854 5590 8 You die.
855 5590 0 Score not registered for borgs.
855 5590 0 # Aborting (death).
855 5590 0 # Aborting (player died).
```

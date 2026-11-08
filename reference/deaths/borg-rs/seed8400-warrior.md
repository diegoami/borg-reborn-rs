# Natural death: seed8400 Human Warrior

Source: post-M4 #345 investigation, normal-live suite. Actual build identities
are in ../validation/2026-11-07-post-m4-investigation/{c,rust}-BUILD_ID.
Fresh ordinary birth; no restored character, wizard kit, cheat death or
cheat-live. C twice and Rust have identical key/message/player-state traces.
This is one distinct input/death, not three deaths from the verification runs.

Result: a pseudo-dragon; game turn37680, clevel8,
final current depth13, deepest logged current depth14.
The final max_depth field is not treated as a lifetime peak.

## Observations

Known Deep Descent was read at35460; by37004 the Warrior was at depth13 without Phase Door stock. An unknown Darkness scroll at37064 blinded it. At the final decision it had25/113HP, estimated danger52 and attacked a pseudo-dragon with184 estimated HP; light breath killed it.

Last living engine checkpoint: turn37004,
SHA256 56ff9b687e360161ac82ec69e771397e2cb707d40ecdfac4a5ea2c74666aac66. The curated save is
../validation/2026-11-07-post-m4-investigation/death-checkpoints/live-warrior.save.
Actual inventory with awareness flags is in death-inventories.json in the
same receipt. Spoiler kind names were inspected after native load in a
separate process and were never fed to the playing Borg.

## Evaluation and next action

Prioritize escape-stock/restocking and the decision to descend. A controlled rerun from original birth is needed before attributing survival to another policy; loading the prior engine save starts fresh Borg memory.

No causal fix or survival improvement is claimed. The full seeded history
can be rerun; an engine checkpoint does not restore prior private Borg state.
No new original-C defect is established solely by this mortality observation.

## Final events

```text
4558 37678 0 # HIT:the pseudo-dragon (+)
4558 37678 0 # HIT:the pseudo-dragon (+)
4558 37678 0 # HIT:the pseudo-dragon (2)
4558 37678 0 # HIT:the pseudo-dragon (2)
4558 37678 0 # Loc:47,144 Dep:13 Lev:8 HP:25/113 SP:0/0 Danger:p=52
4558 37678 0 # Performing attack type 1 with value 209.
4558 37678 0 # Facing pseudo-dragon at (46,144) who has 184 Hit Points.
4558 37678 0 # Attacking with weapon 'Dagger (1d4) (+0,+0)'
4560 37678 2 You hit the pseudo-dragon.
4560 37678 2 You hit the pseudo-dragon.
4560 37680 102 The pseudo-dragon breathes light.
4561 37680 8 You die.
4562 37680 0 Score not registered for borgs.
4562 37680 0 # Aborting (death).
4562 37680 0 # Aborting (player died).
```

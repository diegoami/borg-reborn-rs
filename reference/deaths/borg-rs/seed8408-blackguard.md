# Natural death: seed8408 Gnome Blackguard

Source: post-M4 #345 investigation, normal-live suite. Actual build identities
are in ../validation/2026-11-07-post-m4-investigation/{c,rust}-BUILD_ID.
Fresh ordinary birth; no restored character, wizard kit, cheat death or
cheat-live. C twice and Rust have identical key/message/player-state traces.
This is one distinct input/death, not three deaths from the verification runs.

Result: a wild cat; game turn6231, clevel1,
final current depth1, deepest logged current depth1.
The final max_depth field is not treated as a lifetime peak.

## Observations

The6006 snapshot contains six known Cure Light Wounds potions and one known Phase Door scroll. Phase Door was used at6186 and depleted. At6225 it had4/16HP against a wild cat estimated at1HP, attacked, missed and was killed by repeated claws. No Cure Light Wounds quaff is logged in that interval.

Last living engine checkpoint: turn6006,
SHA256 d819f7b5d83eccc5db320d20c44370916e0189c9dd442a6d2f2a94507847ea76. The curated save is
../validation/2026-11-07-post-m4-investigation/death-checkpoints/live-blackguard.save.
Actual inventory with awareness flags is in death-inventories.json in the
same receipt. Spoiler kind names were inspected after native load in a
separate process and were never fed to the playing Borg.

## Evaluation and next action

Prioritize a controlled healing-versus-finishing-attack counterfactual. Last-checkpoint stock alone is not proof a particular alternative would survive; no tactical change is made from this observation.

No causal fix or survival improvement is claimed. The full seeded history
can be rerun; an engine checkpoint does not restore prior private Borg state.
No new original-C defect is established solely by this mortality observation.

## Final events

```text
916 6225 0 # Loc:44,64 Dep:1 Lev:1 HP:4/16 SP:2/2 Danger:p=18
916 6225 0 # Fleeing and leaving the level. (scaryguy)
916 6225 0 # Adjacent to monster on Scaryguy Level.
916 6225 0 # Performing attack type 1 with value 32.
916 6225 0 # Facing wild cat at (43,64) who has 1 Hit Points.
916 6225 0 # Attacking with weapon 'Tulwar (2d4) (+0,+0)'
918 6225 3 You miss the wild cat.
918 6227 38 The wild cat claws you.
918 6227 0 The wild cat misses you.
918 6231 38 The wild cat claws you.
919 6231 38 The wild cat claws you.
919 6231 8 You die.
920 6231 0 Score not registered for borgs.
920 6231 0 # Aborting (death).
920 6231 0 # Aborting (player died).
```

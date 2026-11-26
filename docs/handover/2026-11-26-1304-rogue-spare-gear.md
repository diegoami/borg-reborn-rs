# Release redundant Rogue spare gear

Issue #1304. Replaying the loaded save at turn 2,696,014 from
`bbe83d0b` with P-Borg and the recorded YAML drops all seven sandals
on the first safe decision. The boots plan can already select the
Slow Descent sandals at Home as the backup to Wormtongue. The ordinary
sandals are fully identified, have a combined Borg value of 28, and are
surplus under the ranked policy. The prior recording does show all
seven in the pack. The replay starts with fresh Borg decision state.

All four melee spares have unresolved runes and none is the active
swap. P-Borg has no usable identification at the checkpoint. The
baseline melee plan evaluates Forasgil, Narthanc, Nimthanc and
Dethanc, but the unresolved spares never reach the comparison. It
keeps all four through turn 2,705,000.

| Spare | Observed gear comparison |
| --- | --- |
| Short Sword (+4,+1) | The +1 Wisdom changes no saving throw. Thorongil already supplies acid resistance. |
| Trident (+2,+5) | Retained weapons improve its visible damage and accuracy. |
| Dagger (+6,+2) | The +3 Wisdom changes no saving throw. Wormtongue supplies Feather Fall. Narthanc and Nimthanc match its weight and throwable wielded-weapon flag, with higher damage, accuracy and armour. |
| Morning Star (+4,+3) | Retained weapons improve its visible damage and accuracy at lower weight. |

The native save contains unidentified egos on all four spares. Their
unseen effects are excluded from the decision. Narthanc has fire
damage and an activation, Colannon provides stealth, and the Shovel
of Digging is the active tunnelling swap. The replay starts in the
dungeon with selling disabled, so disposal uses safe drops. Town uses
the same gear plan with its existing sale and drop checks. The patched
run frees four weapon slots and the sandal slot by turn 2,696,034.
Through turn 2,705,000 it does not pick up any of those dropped stacks.
It later stores Narthanc at Home and withdraws and clears a separate
plain sandal that was already at Home.

When identification is unavailable, two distinct retained alternatives
can release an unresolved weapon if both cover the observed combat and
swap benefits. An unresolved armour stack can trim to one copy while
identification is still possible. The engine checks armour properties
before stacking. The native fixture verifies this for a valuable stack
and also verifies release of the last copy when two retained
alternatives cover it and identification is unavailable.

Replay with `python3 engine/tests/check_rogue_spare_gear.py /path/to/extracted-bundle`.
The script checks the source hashes, pack releases, and pickup messages.
The owner review gate applies before integration.
[Checkpoint and recorded YAML](https://github.com/diegoami/borg-reborn-rs/issues/1304)

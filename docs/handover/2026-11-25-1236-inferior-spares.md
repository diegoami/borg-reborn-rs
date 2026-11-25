## Inferior spare equipment (#1236)

Compare each eligible spare with the equipped item of the same kind. Keep
items that can improve a combat stat or contribute a property, lighter
gear, and designated swaps. In the dungeon, drop the inferior stack on a
safe floor before ordinary pack clearing. The existing `borg ignore`
inscription keeps it out of pickup. In town, prefer an available buyer
and preserve urgent purchases.

The unknown-rune case is limited to Leather Sandals with visible
`[1,+0] {??}` before max DL5. The shipped
`borg_param_junk_inferior_sandal_ac_margin = 7` requires the worn pair to
have at least seven more points of enchantment. Zero disables that case.
Normal ego generation adds positive armor enchantment. Disenchantment
can remove it, and a disenchanter eye can rarely appear out of depth.
See `obj-make.c:apply_magic`, `obj-desc.c:obj_desc_combat`, and the boot
egos in the pinned engine data.

The six-spare regression frees two slots with the rule enabled and one
with it disabled. Both retain the possible ego, the nexus and speed
swaps, and the +8 upgrade. A reconstructed native DL3 case dropped and
ignored the unknown +0 pair with the rule enabled and kept it with the
rule disabled through turn 700. The native town reconstruction sold the
pair at the Armoury with each chooser. The original Priest recording was
unavailable in this workspace. The broader unknown-wearable backlog is
tracked in #1169.

The shipped smoke change begins on Mage seed 102 at turn 12,064 with a
drop of fully identified inferior gloves. The other three cases match
the prior reference.

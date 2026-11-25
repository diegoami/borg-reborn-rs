## 2026-11-25 · Inferior spare equipment (#1236)

Compare spares with the equipped item of the same kind. Keep gear that
can improve a combat stat or contribute a property, lighter gear, and
designated swaps. Both town choosers sell a full stack to a store that
can accept it. In the dungeon the priority drop is limited to stacks
worth at most 15 gold and happens after urgent travel and recovery.
The `borg ignore` inscription keeps the stack out of pickup. The
historical stair replays set `borg_param_junk_inferior_spares = 0` to
keep their recorded routes.

The unknown-rune case is limited to Leather Sandals with visible
`[1,+0] {??}` before max DL5 and a seven-point enchantment advantage in
the worn pair. Ordinary ego generation adds positive armor enchantment
(`obj-make.c:apply_magic`). Elemental inventory damage can lower the
enchantment of pack armor (`project-obj.c:inven_damage`). Monster
disenchantment can lower it while the object is equipped
(`effect-handler-general.c:effect_handler_DISENCHANT`). The depth
limit confines this heuristic to the shallow game. A setting of zero
preserves every unknown pair.

The six-spare comparison frees two slots with the unknown-rune rule
enabled and one with it disabled. Both retain the possible ego, the
nexus and speed swaps, and the +8 upgrade. The checked-in native test
constructs the DL3 comparison and exercises the engine's description
and drop. The original Priest recording was unavailable in the workspace.
The earlier town reconstruction sold the pair at the Armoury with each
chooser.

### What remains

#1169 tracks the wider unidentified-wearable backlog. The sibling
known-cursed-spare PR (#1232) should integrate first if both are ready.
Its disposal hook can keep its earlier placement because this selector
excludes known cursed items.

### Owner decisions

The stack value limit, shallow depth and seven-point margin are
provisional. Revisit them with the Zoo results.

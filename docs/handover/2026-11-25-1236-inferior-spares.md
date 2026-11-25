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
enchantment of pack armor at any depth (`project-obj.c:inven_damage`).
The heuristic accepts that risk in the shallow game. The max-depth
cutoff uses the disenchanter eye's native DL5 (`monster.txt`).
Disenchantment could erode an earlier worn pair before it returned
to the pack (`effect-handler-general.c:effect_handler_DISENCHANT`).
A setting of zero preserves every unknown pair.

The six-spare comparison frees two slots with the unknown-rune rule
enabled and one with it disabled. Both retain the possible ego, the
nexus and speed swaps, and the +8 upgrade. The checked-in native test
constructs the DL3 comparison and exercises the engine's description
and drop. The original Priest recording was unavailable in the workspace.
The native town test equips a +7 pair and verifies that each chooser
sells all three +0 pairs in a single Angband transaction.

The four smoke games matched the previous reference in keys, messages,
and engine state. The reference now records the build and profile after
synchronizing with Track P at `aeb1544a`.

### What remains

[Zoo #795](https://github.com/diegoami/borg-zoo/issues/795) requests a
paired pilot of the identified and unknown spare rules.
#1169 tracks the wider unidentified-wearable backlog. The merged
known-cursed-spare rule (#1232) keeps its earlier priority. Both rules
use the same ignore-and-drop command sequence.

### Owner decisions

The stack value limit, shallow depth and seven-point margin are
provisional. Revisit them with the Zoo results.

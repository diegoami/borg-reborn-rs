# Wear upgrades and limit Home gear backups

The shipped profile enables `town_gear_home_backups` with the existing
gear cleanup enabled. Known equipment is judged with the character's
equipment power. Town acquires a better piece from Home or wears it
from the pack. It prefers a backup with a distinct known benefit, stores
it after urgent shopping and
disposes of lower-ranked stock to make room when necessary.

One additional backup can save at least one pound over both regular
choices while preserving the kit's essential capabilities. Ring
combinations use both fingers and allow two regular backups. Ties keep
worn gear first, then Home, then the earlier pack stack. Identical
single-slot gear adds no backup value. Gear protections still apply to
unknown runes, artifacts, activations, inscribed items and staged
loadouts. Fueled lights have their own reserve policy. Digging tools
still use the separate surplus cutoff.

Set `borg_param_town_gear_home_backups = 0` to select the previous
cutoff policy for historical replays. Native tests cover a full Home,
wearing the better pair of rings, and storing only the selected backups.

The Paladin checkpoint in [#1279](https://github.com/diegoami/borg-reborn-rs/issues/1279)
has Narthanc, a partially identified Nimthanc and two weaker ordinary
daggers. A named weapon with visible combat bonuses can count against an
ordinary spare of the same kind and weight even when some runes are
unknown. A partially identified fixed artifact can get a wear trial from
the pack in safety when its known power improves the kit. Run
`python3 engine/tests/check_paladin_daggers.py /path/to/extracted-evidence`
to check disposal with either gear policy.

The later Paladin checkpoint from [#1105](https://github.com/diegoami/borg-reborn-rs/issues/1105)
also covers the stronger stealth cloak, a weaker cloak with unseen
runes, and duplicate fueled lighting. The regular cloak backup favors
stealth when worn gear in that slot supplies none. Plain uninspected
armour can leave when two retained pieces cover its observed properties
and the character currently has no usable identification. This is a
policy hypothesis. The unseen runes can still make a discarded piece
useful, and the checkpoint does not measure how often that happens. The Phial
and permanent backup cover the radii and properties of both fueled
spares in this save. A brighter light or a light with a distinct known
benefit can still stay. Run
`python3 engine/tests/check_paladin_cloaks_lights.py /path/to/extracted-evidence`
to check this checkpoint.

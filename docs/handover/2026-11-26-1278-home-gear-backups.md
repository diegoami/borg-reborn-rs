# Wear upgrades and limit Home gear backups

The shipped profile enables `town_gear_home_backups` with the existing
gear cleanup enabled. Known equipment is judged with the character's
equipment power. Town acquires a better piece from Home or wears it
from the pack. It stores the runner-up after urgent shopping and
disposes of lower-ranked stock to make room when necessary.

One additional backup can save at least one pound over both regular
choices while preserving the kit's essential capabilities. Ring
combinations use both fingers and allow two regular backups. Ties keep
worn gear first, then Home, then the earlier pack stack. Identical
single-slot gear adds no backup value. Gear protections still apply to
unknown runes, artifacts, activations, inscribed items and staged
loadouts. Fueled lights and digging tools have separate roles.

Set `borg_param_town_gear_home_backups = 0` to select the previous
cutoff policy for historical replays. Native tests cover a full Home,
wearing the better pair of rings, and storing only the selected backups.

The Paladin checkpoint in [#1279](https://github.com/diegoami/borg-reborn-rs/issues/1279)
has Narthanc, a partially identified Nimthanc and two weaker ordinary
daggers. A named weapon with visible combat bonuses can count against an
ordinary spare of the same kind and weight even when some runes are
unknown. It still needs normal
identification before the new wear planner selects it. Run
`python3 engine/tests/check_paladin_daggers.py /path/to/extracted-evidence`
to check disposal with either gear policy.

# 2026-11-24 · Element exposure markers in item knowledge (#1145)

The historical Track P reproduction at `1c071812` matches the key hash in [Cemetery #146](https://github.com/diegoami/borg-cemetery/pull/146). At turn 21,887 after the first poison hit, the Elf Druid's Wicker Shield has a poison level of 1 in its known shadow and 0 on the real item. The race contribution, engine resistance, engine known resistance, and timed Opposition are all 0. The ordinary caution path sees `BI_RPOIS=1`.

Angband's `equip_learn_element` marks a partially identified item with a known level of 1 after exposure when the item has no resistance and the corresponding rune is unlearned. The Borg copied that marker into its equipment estimate. The engine boundary now strips that sentinel before item analysis. It retains real property values for fully known items and for learned runes. The normalization applies to every element because the engine uses the same exposure logic for each.

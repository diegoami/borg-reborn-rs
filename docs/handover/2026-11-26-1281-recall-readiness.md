## 2026-11-26: Check the Word of Recall destination from town (#1281)

The default Borg checks preparedness at 100% of the dungeon destination
before using Word of Recall from town. Angband resolves the stored recall
depth and applies forced descent. For a shallower probe, set
`borg_param_town_recall_prepared_depth_percent = 60`. The checked depth is
rounded down. The existing risky-play exception still applies.

The reported Paladin passed a DL9 check before recalling into DL16 with
no usable long-range teleports. The native regression checks the refusal
and the corresponding ready kit, and covers a saved depth distinct from
the maximum, forced descent and the 60% override. The Borg takes the
stairs under persistent levels when the engine requires a destination
choice.

The dungeon's separate decision to return to town when playing too
shallow still uses its existing probe. This task applies to the town
departure in [#1281](https://github.com/diegoami/borg-reborn-rs/issues/1281).

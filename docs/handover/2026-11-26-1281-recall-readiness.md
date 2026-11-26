## 2026-11-26: Check the Word of Recall destination from town (#1281)

The default Borg checks preparedness at 100% of the dungeon destination
before using Word of Recall from town. Angband resolves the stored recall
depth and applies forced descent. For a shallower probe, set
`borg_param_town_recall_prepared_depth_percent = 60`. The checked depth is
rounded down. Risky play can bypass preparedness. The special restock
guard for DL100 follows the destination, including if forced descent
advances there after the DL99 quest. It applies at every percentage.

The reported Paladin passed a DL9 check before recalling into DL16 with
no usable long-range teleports. The native regression checks the refusal
and the corresponding ready kit. It checks the refusal reason, the 60%
and 0% overrides, forced descent, and the DL100 guard when the destination
differs from the recorded maximum. The Borg takes the stairs when the
engine requires a persistent-level destination choice. Before the first
visit below town, the engine resolves Recall to DL1 without that prompt.

The dungeon's separate decision to return to town when playing too
shallow still uses its existing probe. This task applies to the town
departure in [#1281](https://github.com/diegoami/borg-reborn-rs/issues/1281).

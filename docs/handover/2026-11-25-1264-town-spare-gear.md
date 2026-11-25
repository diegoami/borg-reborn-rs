## Shared spare-gear comparison in town (#1264)

With the shipped exploration-first setting, town sales and arrival unloading
use the same known-loadout comparison as dungeon disposal, including the
cheap priority drop. It checks combat, protection, weight and speed, and
preserves special gear, viable loadouts, selected swaps, and equipment with
a rune the Borg can still identify. Both town planners can sell an inferior stack whole when a shop has
room for it. Arrival offers a marketable stack before using the existing
ignored-drop path. The separate learned-curse cleanup still takes priority.
After an arrival sale, the Borg resumes its wear and swap checks outside
the shop before ordinary town planning.

Required supplies still have their exploration reserves, including when a town
purchase must exceed the weight allowance. The ranked planner keeps its Home
priorities for useful future kit and scarce supplies. The compiled default of
`junk_exploration_first` remains zero for the paired control.

Native fixtures sell a two-Trident stack through each planner and sell
Tridents on arrival in the shipped mode. An unsaleable Chain Mail stack
is dropped and stays ignored. The arrival fixture keeps a boot
upgrade, a more accurate Rapier, and the two carried Recall scrolls. The
tests also cover the learned-curse path, the flag-off arrival comparison,
and the cheap dungeon drop with free pack slots. The existing Recall and
overweight restock fixtures also pass.

The four shipped borg-rs smoke games match the retained reference, including
messages and state. No reference update is needed.

The owner chooses the reviewer and approves this follow-up before it merges
into `track-p`. The paired evaluation remains [Zoo #804](https://github.com/diegoami/borg-zoo/issues/804)
and uses the separate [#1265](https://github.com/diegoami/borg-reborn-rs/issues/1265)
change for its control.

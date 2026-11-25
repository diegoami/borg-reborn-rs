## Stale kobold target at a doorway (#1229)

The Borg now checks the next map observation after a melee Alter aimed at
an apparently empty doorway. An open-to-closed or closed-to-open change
clears the disproved monster record before it can mark the door as floor.
An opening attributed to that Alter removes only that doorway from the
closed-door watch. Combat messages cancel the attribution, and the watch
still reports monster openings at other doors. [Issue and recording excerpts](https://github.com/diegoami/borg-reborn-rs/issues/1229)

The recorded loop ended at game turn 204,669. At Borg tick 25,000,
`clock_wrap_reset` forced a level-memory reset. The first action after the
reset window followed the up-stair flow west through the doorway. The
available excerpts already show stale monster memory at the first attempt.

The native regression drives a CL5 Dúnadan Rogue's dungeon decisions in a
constructed corridor and feeds both door changes through the engine map and
Borg observation code. It checks movement past the door, melee against a
visible kobold in the doorway, and the scare from another tracked door.

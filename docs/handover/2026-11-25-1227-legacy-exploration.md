## 2026-11-25: Legacy exploration vein loop (#1227)

`explore_stepable_goals` now filters legacy and stair-anchored exploration
in their shared candidate check. At the shipped value of `1`, the dark
flows can target unknown grids, rubble and closed doors. The separate
`borg_flow_vein` still digs tracked veins when its own checks pass.

The incident trace records five refused dig attempts at (189,34) on each
of 25 visits. Each visit then started a five-step spastic search. The
legacy `dark-4` flow could walk those five steps back to the vein's
approach. [Issue and evidence](https://github.com/diegoami/borg-reborn-rs/issues/1227)

`DarkVeinLoopTest` drives the production near and far legacy flows on a
crop of the recorded map, shifted west to fit the test dungeon. With the
filter off it sees the nearby flow commits and the five-step return to
the refused dig. With the filter on it follows a route to a reachable
unknown grid and checks the door, rubble and intentional vein commands.
The original save rebuilds Borg memory on load, so the check uses the
recorded geometry in place of a live replay.

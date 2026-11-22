## 2026-11-22: The light map's outer ring uses the light line of sight (#958)

**Done.** With `cave_light_engine_radius` on (#877, #884), the Borg's own
light map tested the radius-2 grids with `borg_los_light` but lit the
outer ring (radius 3+) on `BORG_VIEW` alone, so passable rubble the view
crosses could mark a grid lit that the engine's light does not reach
(`add_light` tests `los()` per grid, src/cave-view.c:625-640), and the Borg
would expect to see a monster there and drop its record. The outer ring
now also needs line of sight for light (`outer_ring_lit`,
borg-core/src/borg_cave_light.rs). C's proxy (the shipped 0) is unchanged.
New test: `outer_ring_needs_light_line_of_sight_with_the_engine_shape`.

**Found.** Codex's P7 review on #939 (finding P3). Shipped play is
unchanged: the setting ships 0.

**Left.** Nothing for this finding.

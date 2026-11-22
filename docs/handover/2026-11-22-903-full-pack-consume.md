## 2026-11-22: Drop inedible junk while shapechanged (#903)

The shared `borg_eat` command rejects pack food and mushrooms while
shapechanged, because Angband's eat command cannot return to normal
shape. The full-pack and overweight junk callers then use their existing
drop sequence. Potion dispatch still uses Quaff.

The [Cemetery validation](https://github.com/diegoami/borg-cemetery/pull/112)
reproduces both original Zoo #612 stops and verifies that the affected
Druid drops the selected food then reaches the 300,000-turn cap under
both profiles. The original pilot with just the patch is pinned at
`5c6d1c2ebee99b87b24140ac87b35a70c174aa7f` for the owner-started pilot
rerun. Default-core smoke is identical in all three cases on both the
pilot pin and this `track-p` task. The regression checks that the
eat command queues no input for a fox with a full pack containing both
Slime Molds and Slime Mold Juice.

**What is left**

The [four-arm Zoo rerun](https://github.com/diegoami/borg-zoo/issues/639)
awaits dispatch.

**Owner decision**

Accept the re-review for integration into `track-p` and dispatch the Zoo
pilot under RULES 20.

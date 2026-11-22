# #903 Drop inedible junk while shapechanged

The full-pack and overweight paths share `borg_consume`. It rejects pack
food and mushrooms while shapechanged, because Angband's eat command
cannot return to normal shape. Both callers then use their existing
drop sequence. Potion dispatch still uses the potion type and quaff
command. No parameter or shipped setting changes.

The [Cemetery validation](https://github.com/diegoami/borg-cemetery/pull/112)
reproduces both original Zoo #612 stops and verifies that the affected
Druid drops the selected food then reaches the 300,000-turn cap under
both profiles. The original pilot with just the patch is pinned at
`7f3d0d61b354ae911f131126781b4e4482e09290` for the owner-started pilot
rerun. Default-core smoke is identical in all three cases on both the
pilot pin and this `track-p` task. The unit regression checks the
consumption guard with full-pack Slime Mold and Slime Mold Juice stacks.

## 2026-11-25: Identify potions and scrolls through town sales (#1213)

Track P removes experimental quaffing, its recovery hold, and all seven
tasting parameters. Old file and environment overrides for those
parameters still load and are ignored. Ordinary quaffing and reading
require a known flavour.

The town planner already sells from an unknown stack to a shop that accepts
it. The sale reveals the flavour, then the planner evaluates the remaining
copies with that knowledge. The native regression covers an unknown potion
the old code tested, a sale from a four-potion stack, a scroll sale and
subsequent deliberate use of known consumables, including a cure for confusion.

The original watched incident is preserved in
[borg-cemetery](https://github.com/diegoami/borg-cemetery/tree/d7d90fbc40275da3d06af57bea22c6b3d1a16ca0/incidents/2026-11-25-watch-half-orc-ranger-poison-37450).
Restocking after running out of healing is tracked in [#1214](https://github.com/diegoami/borg-reborn-rs/issues/1214).

# #1214 Return to restock healing potions

Based on the stair-routing changes at `8bf96f01`. [#1212](https://github.com/diegoami/borg-reborn-rs/pull/1212)

The shipped minimum is two known healing potions. Below it, the Borg can return before depletion when the stock it last saw in town and its current gold cover the deficit. A failed or already short departure needs a new funding opportunity before another trip. The Borg still requests a return on first depleting its reserve. The funded low-stock check follows the flask resupply rule. [#1184](https://github.com/diegoami/borg-reborn-rs/issues/1184)

Caution still handles immediate threats. The return uses a reachable up stair first, then Word of Recall if the Borg can survive the delay. The objective survives dungeon level changes.

Either town shopper can fill the reserve from Home or affordable regular-store stock. The normal shopper can sell or free pack space if the Borg cannot buy yet. If a visit ends short, the Borg uses the last observed offers to assess whether new dungeon gold can fund another trip. Other supplies and preparedness keep their own checks.

The native regression drives the shallow dungeon and leave decisions with one potion remaining and at depletion. It covers an unaffordable refill, a blocked staircase, the next level's grace period and the failed-visit guard. A separate native town run verifies that each shopper buys up to the configured minimum. [Original Ranger case](https://github.com/diegoami/borg-cemetery/tree/d7d90fbc40275da3d06af57bea22c6b3d1a16ca0/incidents/2026-11-25-watch-half-orc-ranger-poison-37450)

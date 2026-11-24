# #1214 Return to restock healing potions

Based on the stair-routing changes at `8bf96f01`. [#1212](https://github.com/diegoami/borg-reborn-rs/pull/1212)

The shipped Borg requests a town return when an observed healing potion stock falls to zero. Caution still handles immediate threats. The return uses a reachable up stair first, then Word of Recall if the Borg can survive the delay. The objective survives dungeon level changes.

In town, either shopper can take a usable potion from Home or buy an affordable one. The regular shopper can sell or free pack space if the Borg cannot buy yet. If the visit ends without a potion, the Borg remembers the cheapest quote it saw that it could not pay. Crossing that price with dungeon gold can justify another trip. A visit with no usable stock can be reconsidered on a later town visit for another reason. Other supplies and preparedness still have their own checks.

The native regression drives the shallow dungeon and leave decisions, a blocked staircase, and a new level's grace period. It checks the failed-visit guard and its release when gold covers the observed quote. A separate native town run executes a purchase through each shopper. The original Ranger incident is retained in the [Cemetery case](https://github.com/diegoami/borg-cemetery/tree/d7d90fbc40275da3d06af57bea22c6b3d1a16ca0/incidents/2026-11-25-watch-half-orc-ranger-poison-37450).

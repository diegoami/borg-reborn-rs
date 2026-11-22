## 2026-11-22: Keep stair routes from undoing a tactical retreat (#932)

`caution_retreat_reentry_ticks` is opt-in and defaults to zero.
During a dungeon departure, the Borg remembers recent squares it
retreated from after confirming the retreat landed. Stair routes avoid
those squares while danger remains above the retreat threshold. A
blocked route can still fall through to ordinary flight Recall with its
existing gates. The tests cover a two-square and three-square pattern,
retreat confirmation and expiry, a usable detour and a blocked corridor.

In the [Cemetery validation](https://github.com/diegoami/borg-cemetery/pull/120),
the historical Ranger encounter matches the original keys to the first
changed move. The 128-think variant reduces jelly touches from 132 to 1
and reaches 100,000 at full HP. The live Mage's three-square geometry is
covered by the route tests. Smoke matches 3/3 at shipped settings.

The 128-think window is a Zoo pilot hypothesis. Owner acceptance is needed
to merge the reviewed version into `track-p` and start the Zoo pilot.

The review follow-up confirms landings before any new tactical action.
Stair reachability for the existing ranged and Recall choices and both
unweighted desperate-walk checks use the same exclusions as the route.

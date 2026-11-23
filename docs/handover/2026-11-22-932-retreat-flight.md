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
A guarded unweighted reachability probe also checks whether the spread's
path passes the actual flight's clean-path gate. A reachable detour with
an awake blocker now leaves Recall and ranged fallbacks available.

2026-11-23, review of `884ffa13`: the guarded unweighted probe now runs the
walk's own stair spread (`spread_unweighted_stairs`) under the walk's flight
danger rule (`with_flight_danger_rule`), so it cannot mark icky or call
unreachable a detour the walk would take. A held grid is rechecked over
`caution_danger_horizon`, the horizon strategic retreat reads. No unit test
walks the probe and then the flight on one map: the spread and the path walk
read the engine's cave, so that check needs an engine scenario. The branch
also merges track-p `09580159`.

The [two-arm Zoo pilot](https://github.com/diegoami/borg-zoo/issues/657) ran
on `e43b6d87`. It finished with no errors: one death per arm, the same start,
and 17 of 18 games changed. It did not compute the issue's own measure.

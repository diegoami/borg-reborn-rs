## 2026-11-21: Adjacent stair loop (#862)

An up-stair search that steps off a down stair can now take its next
step without ordinary or desperate flight taking over. The weighted
and unweighted routes let a viable search advance before any down-stair
fallback. This also prevents the three-square loop seen when an alternate
down stair briefly outranked the one it had just left. Progress renews
the deferral. A failed step, a level change or a marked increase in
danger restores normal stair routing.

The original recording, build and exact turn ranges are identified in
[#862](https://github.com/diegoami/borg-reborn-rs/issues/862).

The native Warrior smoke run reproduces the same transition at game
turn 7,456 in the old reference. With the fix it uses Recall instead
of walking straight back to the staircase. The Mage and Priest
smoke runs match their previous play.

The engine regression tests check the Warrior's desperate handoff and
an actual descent. They also exercise the Mage at seed 204, where the
ordinary fallback used to return to the same stair at turn 118,849.

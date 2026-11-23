## 2026-11-22 — Repeated hallway light (#995)

The defensive beam requires a set clock stamp at least 48 turns old,
as in the original delay. Exploration and defense share one history.
When the first eligible direction has the same observed terrain as the
last queued attempt from this square, the Borg yields. A simulation or
a command that fails to queue leaves the beam eligible. Rest and rod
recharge leave the record intact, and a level change clears it.

The ray runs through observed terrain to the beam's projection range
or the first known blocking grid. A newly revealed grid on that ray
can make another beam useful, including when the previous beam revealed
it. A beam that reveals nothing leaves the same record.

The decision regression exercises the shared simulation and queue path.
It covers a failed queue, an unchanged hallway, and a new observation
that permits another attempt. The original Priest's stationary sequences
are retained in [borg-cemetery #126](https://github.com/diegoami/borg-cemetery/pull/126).

The shipped profile enables the guard. The stair-handoff fixtures use
the original lighting policy so they can still reach the recorded
turns and grids with the original assertions.

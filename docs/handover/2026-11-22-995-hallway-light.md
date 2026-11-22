## 2026-11-22 — Repeated hallway light (#995)

The defensive beam now requires a recent unseen-attacker trigger. Both
exploration and defense consult the same attempt history before selecting
a direction. Queuing a spell, rod, or wand records that direction and the
observed ray terrain at the beam's origin. A rest or device recharge
leaves the record intact. Changes in the ray, another origin, or a new
level allow another beam.

The regression tests cover the unseen-attacker age gate, independent
directions, and retry after a change in observed terrain. The reported
Priest's stationary sequences are retained in
[borg-cemetery #126](https://github.com/diegoami/borg-cemetery/pull/126).

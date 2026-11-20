## Threat-weighted stair flights (#765)

Stacked on [#757](https://github.com/diegoami/borg-reborn-rs/pull/757)
and its measured mode 3 revision, `6951bc98`.
`caution_stairs_flight_threat_weight` is compiled and shipped at 0. A
positive value prices each entered square, including the stair, by the
Borg's danger estimate relative to current HP. The search blocks awake
monsters and keeps the capped down-stair flight's limit in actual moves.
The weight also covers the upward `GOAL_BORE` flight in 807683971.

The clean-path checks read the selected route. When they refuse it, the
search looks for a clean detour to the same stair set. Mode 3 evaluates
each possible destination's own definition of whether a monster is
ahead. Each movement decision uses a route from the current observation.

The 600-start comparison uses the [scratch manifest and runner](https://github.com/diegoami/borg-reborn-rs/tree/e4bcb70f/scratch-757),
shipped settings on `ad1dff48` for the control, and the measured mode 3
settings on `6951bc98`. The experimental arm adds weight 5 to those
mode 3 settings. Ranged-first (`50c23a5e`) is a separate
[borg-zoo arm](https://github.com/diegoami/borg-zoo/issues/501).

The experimental build `d8edf546` finished all 600 starts at the 100,000-turn
cap or on death. There were 10 deaths (5 Rangers, 3 Priests and 2
Necromancers), against 12 in the published shipped control and 16 in the
published mode 3 arm. Two initial wall timeouts, Rogue 23482467 and
Blackguard 149213750, reached the cap on retry under the same 300-second
limit. All three issue seeds reached the cap with weight 5. The comparison
uses the published control totals and local replays of the reported seeds.
See [PR #769](https://github.com/diegoami/borg-reborn-rs/pull/769).

Formatting, workspace Clippy and tests, release core tests, parameter
citations and the vendor check passed. All three shipped borg-rs smoke
runs matched the reference key streams, messages and state logs.

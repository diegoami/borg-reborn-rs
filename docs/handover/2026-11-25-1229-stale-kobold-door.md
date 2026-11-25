## Stale kobold target at a doorway (#1229)

### What changed

The Borg now checks the next map observation after a melee Alter aimed at
an apparently empty doorway. An open-to-closed or closed-to-open change
clears the disproved monster record before it can mark the door as floor.
An opening attributed to that Alter removes only that doorway from the
closed-door watch. A hit, miss, or kill message cancels the attribution.
The watch still reports monster openings at other doors.

### What the recording shows

At turns 162,109 and 162,129 the log both reports a monster opening the
door at `(90,37)` and queues an attack on a small kobold at that same
square. An active small-kobold record would mask the door as floor before
the watch. The native fixture reproduces the recorded combination with a
grid link to a slot outside the active kill range. The twitch reset and
the full-table allocator can both create such a link in source. The
recording excerpts lack the kill slot and active-range state needed to
identify the actual path. [Issue and recording excerpts](https://github.com/diegoami/borg-reborn-rs/issues/1229)

Removing the watch entry also matters when a record is active. Deleting
that record lets the watch see the open door in the same observation. The
fixture runs the terrain pass before the watch to exercise both cases.

The recorded loop ended at game turn 204,669. At Borg tick 25,000,
`clock_wrap_reset` forced a level-memory reset. The first action after the
reset window followed the up-stair flow west through the doorway. The
available excerpts already show stale monster memory at the first attempt.

### Verification

The native regression drives a CL5 Dúnadan Rogue's dungeon decisions in a
constructed corridor and feeds both door changes through the engine map and
Borg observation code. It checks active and inactive monster records, a
stable active count, movement past the door, hit-message cancellation, melee
against a visible kobold, and the scare from another tracked door.

### Remaining work

- Trace how a grid link survived outside the active range in the recorded
  game. Check the twitch reset and the allocation boundary, including on a
  non-door square. [#1229](https://github.com/diegoami/borg-reborn-rs/issues/1229)
- Measure the shipped recovery in a paired Zoo pilot against Track P
  `18a690350b1d1a239e1976db003fcb1ad6089b62`. The recovery is unconditional.
  Comparing those two pinned builds isolates the fix before subsequent
  changes land. Later builds have no switch to isolate it.

### Owner decision

Decide whether to wait for the Zoo pilot before accepting the PR. The
native fixture covers the reported failure and the shipped smoke suite
covers ordinary play.

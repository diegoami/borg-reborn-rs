## 2026-11-25: Stale kobold target at a doorway (#1229)

### What changed

The Borg now checks the next map observation after a melee Alter aimed at
an apparently empty doorway. A door toggle or failed lockpick disproves
the remembered target before it can mark the door as floor. A doorway
pointing to a reused slot loses its old grid link and preserves the other
monster. The count covers live records within `1..kills.nxt`. The allocator
reuses slots within that range and deletion decrements only for active slots.
An opening attributed to that Alter removes only that doorway from the
closed-door watch. A hit, miss, or kill message cancels the attribution.
The watch still reports monster openings at other doors and an opening
after the Borg's own lockpick failed. [Calibration](../CALIBRATION.md#a-stale-monster-at-a-doorway-1229)

### What the recording shows

At turns 162,109 and 162,129 the log both reports a monster opening the
door at `(90,37)` and queues an attack on a small kobold at that same
square. An active small-kobold record would mask the door as floor before
the watch. The native fixture reproduces the recorded combination with a
grid link to a slot outside the active kill range or a reused record
elsewhere. The twitch reset and the old full-table allocator could leave
links outside the scanned range. The
recording excerpts lack the kill slot and active-range state needed to
identify the actual path. [Issue and recording excerpts](https://github.com/diegoami/borg-reborn-rs/issues/1229)

At the pinned engine, Alter attacks an occupant first. Empty floor can
print `You spin around.`, which already triggers the Borg's nearby-memory
cleanup. A door takes the terrain branch instead. A failed pick prints
`You failed to pick the lock.` while leaving the door closed. The Borg
now uses that feedback when it had queued Alter as a melee attack.
[`cmd-cave.c:170–216`](../../engine/angband/src/cmd-cave.c#L170-L216) · [`cmd-cave.c:950–1003`](../../engine/angband/src/cmd-cave.c#L950-L1003)

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
stable active count, movement past the door, hit-message cancellation,
failed lockpicking, and genuine scares. The final control executes the
native Alter command against a visible kobold on the open door.

### Remaining work

- Trace how the recorded grid link became stale. The original monster slot
  and active range would distinguish an inactive record from a reused slot.
  General cleanup of reset links on non-door squares remains separate.
  [#1229](https://github.com/diegoami/borg-reborn-rs/issues/1229)
- Measure the shipped recovery in a paired Zoo pilot against Track P
  `18a690350b1d1a239e1976db003fcb1ad6089b62`. The recovery is unconditional.
  Comparing those two pinned builds isolates the fix before subsequent
  changes land. Later builds have no switch to isolate it.

### Owner decision

The owner requested reconciliation and merge. A paired Zoo pilot remains
proposed for population coverage.

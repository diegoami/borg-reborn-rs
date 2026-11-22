## 2026-11-22 – Emergency down-stair flight from floor (#1052)

When an emergency stair step reverses dark exploration, caution can
replan the short escape after assessing the next square. The handoff
expires if the move fails, danger rises, HP falls or the level changes.
Existing stair-route and descent checks still apply.

The regression starts on floor and walks past the two alternating squares
to a down staircase, with one and two moves remaining after the reversal.
A second check exercises a fresh threat and a failed move.

The uninterrupted recording and the native reload are linked from
[#1052](https://github.com/diegoami/borg-reborn-rs/issues/1052). The
reload starts with fresh Borg memory. The regression exercises the
movement sequence from the uninterrupted trace.

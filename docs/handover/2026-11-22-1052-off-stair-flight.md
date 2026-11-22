## 2026-11-22 – Emergency down-stair flight from floor (#1052)

When an emergency stair step reverses dark exploration, caution can
replan the short escape after assessing the next square. The handoff
expires if the move fails, danger rises, HP falls or the level changes.
Existing stair-route and descent checks still apply.

The native test runs production movement and caution on fabricated cave
observations and feeds each queued move back into the Borg. It starts on
floor and reaches a down staircase with one and two moves remaining after
the reversal, then checks the descent key. Both weighted and unweighted
routes are exercised. The unweighted continuation keeps the remaining
move limit, so a blocked nearby stair cannot send it toward a farther one.
A unit check exercises a fresh threat and a failed move.

The uninterrupted recording and the native reload are linked from
[#1052](https://github.com/diegoami/borg-reborn-rs/issues/1052). The
reload starts with fresh Borg memory. The regression exercises the
movement sequence from the uninterrupted trace.

All three shipped-core smoke runs matched the reference. The three
existing native stair-handoff cases passed. The checksum-verified
incident save also reached turn 365,200 alive with the shipped settings.
The recording's configuration includes parameters from another build.

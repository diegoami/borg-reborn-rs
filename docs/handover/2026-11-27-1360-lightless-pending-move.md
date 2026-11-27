## 2026-11-27: lightless recovery settles its last move before yielding (review of #1360)

Owner, 2026-11-27, from the Codex review of #1360 (P2). #1360 was merged
into `track-p` before that review was read; this is the follow-up.

**What was done:** `borg_lightless::update` now calls `Recovery::observe` with
the Borg's position on the first think after recovery queued a move, before it
can return false and give the turn to the ordinary think
(`lightless_fallback`, a monster adjacent). Test
`update_settles_the_pending_move_before_it_yields_to_combat` and
`update_settles_the_pending_move_with_the_switch_off`.

**Found:** `observe` ran only in `recover()`. With a monster adjacent,
`update()` yielded, the ordinary think could undo a step that had worked
(retreat to the start square in combat), and the next `recover()` saw
`at == from` for the still-pending move and counted it failed. One false
failure can close the only route up in a corridor. The reviewer reproduced it
with the routing code in a one-corridor case; the new test follows the same
sequence through `update()` and fails without the fix. The
switch still selects the same policy as before (0: #811 stops the Borg when
it gives up; 1: ordinary play); what changed is only when the pending move is
settled. That also holds with the switch at 0: the dungeon thinker can return
before `recover()` on a turn (`walk_away_step`, `paralyser_step_away`,
`fight_before_flight`), and those turns used to leave the pending move
unsettled; now `update()` settles it first, so bookkeeping no longer goes
stale across those actions either (review of this PR, P3).

**Checked:** CI's cargo commands (see the PR). Two unit tests, through
`update_with` (the switch given, since params are fixed per process): the
combat handoff (switch on, a monster next to the Borg: the turn yields, the
pending step is already settled, and after the retreat the step is still
usable) fails when the settlement is moved below the yield or removed; the
switch-off test pins the settlement on a turn that does not yield.

**Not checked:** a game that loses its light (none can be started, as in
#1359).

**Left:** nothing here. Whether `lightless_fallback` helps stays a zoo
comparison (RULES 14).

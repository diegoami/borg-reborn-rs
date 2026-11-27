## 2026-11-27: lightless recovery settles its last move before yielding (review of #1360)

Owner, 2026-11-27, from the Codex review of #1360 (P2). #1360 was merged
into `track-p` before that review was read; this is the follow-up.

**What was done:** `borg_lightless::update` now calls `Recovery::observe` with
the Borg's position on the first think after recovery queued a move, before it
can return false and give the turn to the ordinary think
(`lightless_fallback`, a monster adjacent). Test
`update_settles_the_pending_move_before_it_can_yield`.

**Found:** `observe` ran only in `recover()`. With a monster adjacent,
`update()` yielded, the ordinary think could undo a step that had worked
(retreat to the start square in combat), and the next `recover()` saw
`at == from` for the still-pending move and counted it failed. One false
failure can close the only route up in a corridor. The reviewer reproduced it
with the routing code in a one-corridor case; the new test follows the same
sequence through `update()` and fails without the fix. With
`lightless_fallback` at 0 nothing changes: `recover()` follows `update()` in
the same think and the pending move is taken once either way.

**Checked:** CI's cargo commands (see the PR); the unit test fails with the
`observe` call removed and passes with it. **Not checked:** a game that loses
its light (none can be started, as in #1359).

**Left:** nothing here. Whether `lightless_fallback` helps stays a zoo
comparison (RULES 14).

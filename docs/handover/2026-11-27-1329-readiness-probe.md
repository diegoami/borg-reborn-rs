## 2026-11-27: Side-effect-free readiness checks in the errand shell (#1329)

From an audit of how depth readiness is routed through the C Borg, borg-core
and the errand shell (issues #1326–#1333).

### What was done

- `borg_prepared.rs` gains `probe` and `prepared_probe`: a readiness check
  that C never makes, run without its side effects on the core.
  `borg_prepared` sets `ready_morgoth` (`borg_restock` resets an unknown
  -1 to 0; from DL98 it is set to -1 or 1) and, from DL82, re-notices the
  Home. Both are restored. The trip policy's private probes
  (`borg_trip.rs`) now call these; their effect is unchanged.
- The errand shell's four readiness checks (the ladder's skip in `target`,
  the `*`-trip check, `scared_home`, `prepared_below`) use `prepared_probe`.
  Before, an errand could change `ready_morgoth` or the Home counts between
  the core's own checks. The wrong comment ("reads the core's state only")
  is corrected.
- In town, `on_leave_level` no longer asks `prepared_below(st, 0)`: that is
  `borg_prepared(1)`, which always passes (C returns first for depth 1), so
  its `refuse_descent(st, 1)` could not run. A comment says why.

### Checks

CI's four cargo commands, and a unit test of `probe`. `borg_prepared`
itself reaches the engine, which borg-core's unit tests do not link, so
`prepared_probe` has no unit test of its own. No game was run. With no
errand every hook returns at once, so `smoke-borg-rs` is unaffected.

### What is left

The rest of the audit: #1326 (a restock trip is the core's own errand),
#1327 (judge a repeating trip on leaving town), #1328 (the errand's way
down obeys `more_refused`), #1330 (depth-cap wording), #1333 (stair walks
continue any `GOAL_FLEE` flow).

### Owner decisions

#1331 (should the kit descent gate apply from town and to Recall?) and
#1332 (should the plan and kit caps count in `borg_power`'s depth bonus?).

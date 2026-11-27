## 2026-11-27: Judge a repeating trip when the Borg leaves town (#1327)

Stacked on #1335 (#1326), itself on #1334 (#1329).

### What was found

A trip marked `*` repeats until the core is prepared for the deepest level
of the next trip. The errand shell asked that on the first think in town
(`arrive_home`), before the core shopped. `borg_prepared` checks
`borg_restock` first, so a Borg that came home short of cures, Phase Door
or food was judged not ready and repeated the trip even when shopping would
have made it ready.

### What was done

- `arrive_home` leaves the decision pending (`Running::repeat_pending`).
  `settle_repeat` makes it when the core is bored with town
  (`on_leave_level`, after shopping), before the next target is chosen; if
  the core leaves town another way (caution takes the stairs), on the next
  think. The gate itself is unchanged (`@N` from character level N, else
  `borg_prepared` for the next trip's deepest level), now through
  `prepared_probe` (#1329).
- `docs/ERRANDS.md` and the module doc say when the check runs.
- Unit test `a_repeating_trip_is_judged_after_town_not_on_arrival`.

### Checks

CI's four cargo commands. No game was run; with no errand every hook
returns at once, so `smoke-borg-rs` is unaffected.

### What is left

Nothing for this task. The rest of the audit: #1328, #1330, #1333; owner
decisions #1331, #1332.

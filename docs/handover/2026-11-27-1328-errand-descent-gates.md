## 2026-11-27: The errand's way down obeys the core's stair refusals (#1328)

### What was found

The errand shell's `stairs_down` checked readiness for the next level, then
pressed `>` when it stood on a down staircase, or continued a `GOAL_FLEE`
flow, before `borg_flow_stair_more`. The core's own descent through that
flow also refuses through `more_refused`: hungry, weak or short of food,
the sale hold, no light (these with an up staircase known, as C, unless
`stairs_descent_gates_without_upstair`), and a pending Recall
(`recall_restricts`). On the stairs the errand skipped them.

### What was done

`stairs_down` asks `more_refused(st, false)` first. Refused, it queues
nothing: in town the core goes on with town, in the dungeon `walk` returns
false and the core plays on (it eats, lights, waits for its Recall). In
`move_on` the errand then tries the stairs up for a fresh level, as it does
when no way down is known.

### Checks

CI's four cargo commands. `more_refused` reaches the engine, which
borg-core's unit tests do not link, so there is no unit test. No game was
run; with no errand every hook returns at once, so `smoke-borg-rs` is
unaffected.

### What is left

#1333: both stair walks continue any `GOAL_FLEE` flow, whatever its
direction (left out here). The rest of the audit: #1326, #1327, #1329,
#1330; owner decisions #1331, #1332.

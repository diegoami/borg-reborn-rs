# R5 flow and dungeon ownership audit

This audit covers `feature/track-r` at
`158868b126e428482e57ddd35971de942526df3a`. Its runtime comes from accepted
R4.5 `e898abb5`. The fixed native reference is `24be9873` on engine
`574b618c`. The nine-file, 13,984-line inventory in the preflight is the
search surface. This map follows the flow workspace and its invalidators,
the stair entry points and the order in which the dungeon driver tries
actions. [Preflight](handover/2026-11-26-r5-preflight.md) ·
[R4.5 acceptance](R45-ACCEPTANCE.md)

## Path state and ownership

A flow proposes a route to a goal. The driver decides which goal to try,
while the shared path computation produces costs and the native action layer
interprets the next grid. Several failed attempts still change state. A
port must compare their writes and downstream calls as well as the action
that eventually succeeds.

| State or operation | Current read and write boundary | R5 ownership |
| --- | --- | --- |
| `FlowState` workspace and queue | The ring's `head`, `tail`, and parallel coordinate arrays track work in discovery order. `data_hard` supplies the initial 255 costs, `data_cost` is the current search, and `data_flow` is the committed route. Clear resets the cost and queue. The enqueue and spread operations update costs and queue, and discard the queue after spread. State also holds temporary target coordinates and observed step and door tracks. [State and enqueue](../borg-core/src/borg_flow.rs#L96) · [Clear](../borg-core/src/borg_flow.rs#L320) · [Spread](../borg-core/src/borg_flow.rs#L763) | Shared search storage. R5 supplies goals and requests through typed operations. |
| Search policy and order | `borg_flow_spread` reads the route cap, optimization origin, avoidance, tunneling, sneak and mode flags. Low-level stair-relative spreads use the tracked stair as origin and turn off optimization. The ordered adjacency scan checks bounds and existing cost, sneak, terrain, occupancy and breeder restrictions, then cached danger. The direct dark flow has its own danger threshold and writes into the same grids. [Spread](../borg-core/src/borg_flow.rs#L763) · [Terrain and occupancy](../borg-core/src/borg_flow.rs#L410) · [Dark direct path](../borg-core/src/borg_flow_dark.rs#L281) | Own the caller's choices about routes. Keep traversal order, map access and cost updates in the shared computation for the first slice. |
| Commit and continuation | A normal commit refuses cost at or above 250, writes the note, copies costs to `data_flow`, resets weighted-step state, records whether explicit up-stair flight was selected, and sets `goal.type_0`. A subsequent `borg_flow_old` first tests the requested goal. A matching goal with no usable step clears the goal, and a digging flow at its target also updates the anti-summoning clock. A matching successful step can update the stair handoff. [Commit](../borg-core/src/borg_flow.rs#L962) · [Continuation](../borg-core/src/borg_flow.rs#L1595) | Own whether to try or continue a goal. Keep commit, route bookkeeping and execution as effectful boundaries. |
| Choosing a step | The unweighted selector starts at current cost × 10 minus 5 and walks the native eight-direction order. Equal choices preserve the first eligible dungeon direction. In town, each qualifying tie can draw from the Borg RNG. Near the digging goal, the cardinal check comes **after** tie handling. The same selector is called by the committed-flow step and the up-stair continuation check. [Callers and selector](../borg-core/src/borg_flow.rs#L1595) | First candidate for a small policy port. Preserve both call sites and the exact direction and RNG request sequence. |
| Weighted stair route | A move-capped A* search keeps cost/step trade-offs per grid and caches entry evaluation only within the query. It uses the native direction order and deterministic queue ties. Costs include the danger of entering the staircase. Commit writes remaining move counts to the regular cost grid, saves the explicit next grid, and starts a replan after the first attempt. [Weighted search](../borg-core/src/borg_flow.rs#L599) · [Queue ordering](../borg-core/src/borg_flow.rs#L659) · [Weighted commit](../borg-core/src/borg_flow.rs#L1004) | Shared route solver and commit. R5 can later choose direction, cap and weight through a seam. |
| Observing terrain and applying a step | The update layer owns observed terrain, stair tracks and route invalidation. A floor/wall change clears the cell's danger flags and turns a newly blocking cell's committed cost into 255. The action layer may close a door, take stairs, handle an obstacle, collect an item, enter a shop or queue a walk. A successful action can occur before a move, so handoffs verify the subsequently observed grid. [Map update](../borg-core/src/borg_update.rs#L543) · [Step execution](../borg-core/src/borg_flow.rs#L1316) · [Walk recording](../borg-core/src/borg_flow.rs#L1554) | Native observation and action execution. Leave these behind the current engine and command seams. |

## Flow danger cache

`data_know` records a classified square and `data_icky` records a veto.
They persist across ordinary flow clears. They are separate from the
regional and monster fear arrays read by `borg_danger`. In regular enqueue
and spread, an existing veto is consulted first. An unclassified accepted
square is marked known before danger is evaluated. If danger exceeds the
flow threshold, the square is marked icky and excluded. Desperate, digging
and configured stair flight skip new classifications; existing vetoes are
still checked on these unweighted paths. Weighted routes instead evaluate
per-entry danger and do not consult that cache. [Flow clear](../borg-core/src/borg_flow.rs#L320) ·
[Spread and enqueue](../borg-core/src/borg_flow.rs#L858) ·
[Weighted entry](../borg-core/src/borg_flow.rs#L599) ·
[Danger state](../borg-core/src/borg_danger.rs#L69)

| Invalidator or additional writer | Effect to preserve |
| --- | --- |
| Monster tracking | Delete, sleep, follow, create, observed movement (two branches), and locate/reidentify set `danger.wipe`. The sleep path also clears the committed goal. [Delete and sleep](../borg-core/src/borg_flow_kill.rs#L550) · [Follow](../borg-core/src/borg_flow_kill.rs#L850) · [Create](../borg-core/src/borg_flow_kill.rs#L950) · [Observe](../borg-core/src/borg_flow_kill.rs#L1165) · [Locate](../borg-core/src/borg_flow_kill.rs#L1613) |
| Level and map observation | A new level sets the wipe flag, resets avoidance and clears stairs, steps and doors. Forgetting the map immediately clears both cost grids and both cache arrays. Changed floor/wall status clears the per-cell flags even when the change opens rather than blocks the square. [New level](../borg-core/src/borg_update.rs#L1941) · [Track reset](../borg-core/src/borg_update.rs#L2065) · [Map reset](../borg-core/src/borg_update.rs#L294) · [Grid update](../borg-core/src/borg_update.rs#L543) |
| Dungeon policy | Normal thinking sets the wipe when it changes avoidance to current HP. A bravery attempt sets it before the brave pass and after restoring avoidance from HP, whether the attempt succeeded or failed. [HP refresh](../borg-core/src/borg_think_dungeon.rs#L683) · [Bravery](../borg-core/src/borg_think_dungeon.rs#L531) |
| Dark exploration | The dark-flow setup requests a wipe when ignoring monsters or on a scary level, then clears the workspace. The direct path writes classifications and cost; the local-region border sets and clears both classification arrays explicitly. [Direct path](../borg-core/src/borg_flow_dark.rs#L281) · [Border and clear](../borg-core/src/borg_flow_dark.rs#L424) |
| Next flow clear | Reset scratch cost and queue every time. If the flag is set, clear both classification arrays and consume the flag. Leave the committed cost grid in place. [Clear](../borg-core/src/borg_flow.rs#L320) |

The stair preview `with_stair_spread` saves the queue, scratch cost,
classification arrays and its scoped flags, temporarily enables
`desperate`, and restores those saved fields. The saved fields exclude
`danger.wipe`. If it was set, the clear inside the preview consumes it
while the old cache contents are restored at the end. Treat this as a
stateful observation in parity work. A change to the invalidation behavior
needs its own decision and evidence. Also retain the persistent scratch
effects of `borg_flow_cost_stair`, which calls clear even if its stair index
is absent. [Stair preview](../borg-core/src/borg_flow_stairs.rs#L606) ·
[Stair distance](../borg-core/src/borg_flow_stairs.rs#L507)

## Stair entry points and handoffs

| Entry | Selection and effects |
| --- | --- |
| `borg_flow_stair_both` | Clear the refusal latch. Try the combined rule, then on a dirty-path refusal try up and then down with that same rule. The combined rule handles hunger/light, rising, the weighted option and parked monsters. Its separated attempts still use the combined rule's gates. [Public entry](../borg-core/src/borg_flow_stairs.rs#L1200) · [Combined rule](../borg-core/src/borg_flow_stairs.rs#L544) |
| `borg_flow_stair_less` | Refuse forced-descend and absent up stairs. Choose weighting or the two unweighted spread options selected by level and light. [Up rule](../borg-core/src/borg_flow_stairs.rs#L816) |
| `borg_flow_stair_more` | Refuse absent stairs, flight down while rising, and `more_refused`. The last checks preparedness, food, sales and light when up stairs are known, then checks Recall. The brave flag bypasses the preparedness and food checks. [Refusal and down rule](../borg-core/src/borg_flow_stairs.rs#L858) |
| Shared route wrappers | Scope `flight_no_danger` around the attempt. A failed flee may retry with `hunting=true` through known breeders, then record `breeder_through` on success. Unweighted commit checks the already spread route when the clean-flight rule applies. Weighted routing can retry a blocked path as a clean detour. [Fight-through](../borg-core/src/borg_flow_stairs.rs#L1238) · [Unweighted check](../borg-core/src/borg_flow_stairs.rs#L306) · [Weighted alternatives](../borg-core/src/borg_flow_stairs.rs#L342) |
| Bounded desperate flight and observations | Caution can probe path blockers, monsters ahead and steps around the current grid. The bounded entry passes the requested cap to up/down routing. Weighted up and the down branches use the supplied cap. The unweighted up branch still spreads to 250. Weighted descent and weighted reachability have separate entries. Keep the distinction between a missing route, a refused route and an action that committed a route. [Probes](../borg-core/src/borg_flow_stairs.rs#L636) · [Bounded flight](../borg-core/src/borg_flow_stairs.rs#L483) · [Up spread](../borg-core/src/borg_flow_stairs.rs#L816) · [Caution caller](../borg-core/src/borg_caution.rs#L975) |
| Up-stair search handoff | `borg_flow_old` records the departure after a successful step action when searching for an up stair or following a known one. A record includes the original stair, expected next grid, level, turn, danger and known flag. An observed arrival in one to three turns can renew it if danger permits. Continuing exploration suppresses a down target that could undo the move; a known route replans before trying down again. [Departure write](../borg-core/src/borg_flow.rs#L1640) · [Validation and renewal](../borg-core/src/borg_flow_stairs.rs#L88) |
| Emergency down-stair handoff | The recorder recognizes reversal of an observed dark step or continued progress on an existing down route. It records remaining moves, expected danger and HP. Renewal requires arrival on the next turn at the expected grid and level, no extra danger, and no HP loss. Caution later checks confusion and rising, then descends at an allowed stair or replans within the remaining budget. Failure clears the record and releases the normal fallback. [Record](../borg-core/src/borg_flow_stairs.rs#L157) · [Resume](../borg-core/src/borg_caution.rs#L7141) |

The dungeon driver's normal and brave paths, the level-leave and early
stair helpers, and shell errands call the same public entries. Owned caution
already calls several through `decision_api`. Its stateful dependencies
record before/after Borg state and replay the result without re-running the
effect. Keep these dependencies in that interface as R5 adds narrower
decision entries. [Dungeon caller](../borg-core/src/borg_think_dungeon.rs#L1048) ·
[Level-leave caller](../borg-core/src/borg_think_dungeon_util.rs#L731) ·
[Errands](../borg-core/src/shell.rs#L1269) ·
[Decision API](../borg-core/src/decision_api.rs#L150) ·
[Tape effect](../borg-core/src/decision_tape.rs#L71)

## Dungeon decision order

The outer thinker refreshes traits and observations, handles store screens,
updates clocks, handles cancel, and calls `shell::before_dungeon_think`
before entering this driver. Store dispatch, observation and the outer
errand gate stay native. Inside the driver, `shell::steer` runs after
caution, town retreat and the town overflow check. Preserve that placement
when errands run on the modes frame. [Outer thinker](../borg-core/src/borg_think.rs#L225) ·
[Inner errand steering](../borg-core/src/borg_think_dungeon.rs#L811)

The table follows the lexical order. Each successful branch returns from
the driver, so later entries are tried only on fallthrough. Counts are
source return sites. They include guarded sites and say nothing about
coverage. Two outside-shop actions use a plain `return true` at lines 873
and 1441. The other 122 success sites use `choice!` to write a trace label
and return true. Preserve distinct positions even when labels repeat.
[Trace hook](../borg-core/src/core_trace.rs#L269)

| Driver range | Success sites | Order and state to retain |
| --- | ---: | --- |
| [583–653](../borg-core/src/borg_think_dungeon.rs#L583) | 3 | Update lightless state. Handle clock reset, floor disposal and Deep Descent discard. Delay/redraw and boredom flags precede nearby-monster classification. |
| [654–769](../borg-core/src/borg_think_dungeon.rs#L654) | 6 | Town-return stair handling, anti-bounce, breeder handling, HP avoidance and wipe, stair leash, inventory refresh, kit tick, curse removal, then town hunt or fight-before-flight. Lightless recovery immediately follows this block. Anti-bounce may change goals and forget tracks on its false return. |
| [770–843](../borg-core/src/borg_think_dungeon.rs#L770) | 9 | Lightless recovery, stair-leash action, dungeon lighting, retreat counter maintenance, committed digging, caution, town retreat, forced sale trip and errand steering. |
| [844–913](../borg-core/src/borg_think_dungeon.rs#L844) | 12 | Urgent light/status/food work and supplies, return-to-town check, magic, weapon repair, corridor attack, regular attack. An outside-shop action can return from supplies without a local trace label. |
| [914–983](../borg-core/src/borg_think_dungeon.rs#L914) | 13 | Wear and rings, continue nearby pickup, try visible loot, repeat ring handling, remove gear, dump quiver, check light, continue recovery flow, recover, new recovery flow, then permanent spells. Keep both rest-hold checks in front of each recovery-flow attempt. |
| [984–1141](../borg-core/src/borg_think_dungeon.rs#L984) | 13 | Low-SP stair rest and stair hugging, broke-town departure, up-stair leash, then fleeing. Active flight tries an existing route, scary-level combined route, up path and finally down when Recall and rising permit it. |
| [1142–1217](../borg-core/src/borg_think_dungeon.rs#L1142) | 13 | Morgoth glyph setup. Continue and start TAKE/KILL in the short-range order, with vein work between pickups. Continue vault and try aimed kill. Repeated TAKE/KILL checks occur at distinct points. |
| [1218–1268](../borg-core/src/borg_think_dungeon.rs#L1218) | 10 | Use/test/enchant/recharge, clear junk/space/weight, then continue TAKE and try visible long-range loot and vein routes. |
| [1269–1341](../borg-core/src/borg_think_dungeon.rs#L1269) | 5 | Level-leaving rules prepare up/down flags, continue flight, try up then down with sale hold. Powerdiving independently tries an old BORE route then a sneak down-stair flow after deeper preparedness. |
| [1342–1372](../borg-core/src/borg_think_dungeon.rs#L1342) | 6 | Continue MISC, DARK, XTRA, BORE and VAULT in that order, then stop exploration for qualifying Recall waits. |
| [1373–1451](../borg-core/src/borg_think_dungeon.rs#L1373) | 14 | Vault kill/take/vein/excavate/flow/dark; general kill/take/vein/near dark; town leave check; far dark; then shop choice, outside action or entry flow. |
| [1452–1530](../borg-core/src/borg_think_dungeon.rs#L1452) | 12 | Bored magic, door lighting, occasional search, direct pursuit, recharge wear, late town leaving, then a second lighting/search/pursuit sequence and safe Recall wait. |
| [1531–1728](../borg-core/src/borg_think_dungeon.rs#L1531) | 8 | Reset stalled state and suspend breeder marks. Try first bravery, scoped Phase, stronger bravery and jumpstart. Then try longer teleport and Recall. Forget old targets before final direct pursuit or twitch, and return false if no action succeeds. |

The brave pass has its own ordering. After the danger read it tries DL100
defence, attack and light beam, then a standing down stair. It tries leash
and flight routes before vault and general kill/take, committed utility
flows, nearby then far dark, and the late leave/search fallback. Its 35
plain success returns feed the outer bravery label. A bravery attempt raises
avoidance, sets the wipe, applies its preparation, runs the brave pass, then
restores avoidance from current HP and sets the wipe again. Jumpstart also
resets goal and selected trackers in its preparation. Preserve all of those
writes when a brave pass falls through. [Brave pass](../borg-core/src/borg_think_dungeon.rs#L222) ·
[Anti-bounce and breeders](../borg-core/src/borg_think_dungeon.rs#L455) ·
[Boost and restore](../borg-core/src/borg_think_dungeon.rs#L531) ·
[Jumpstart](../borg-core/src/borg_think_dungeon.rs#L1606)

Later slices can express the ordered goal checks as modes. A goal attempt
needs an effect result as well as its action result because refusal can
clear an old goal, spend a cache invalidation, update a clock or change
what a later attempt sees. Keep attempts in sequence until each has
separate source and shadow coverage.

## First port slice and controls

Start with direction selection for an already committed, unweighted flow.
Read the current position, committed costs, native direction order, depth
and digging target through a narrow interface. Record direction-table,
distance and Borg RNG requests in their existing order. Both callers can
use the same bounded input and return a direction index or `-1`. Retain the
native continuation driver, stair handoffs, A* solver, cache storage and
step executor in this slice. [First port #1298](https://github.com/diegoami/borg-reborn-rs/issues/1298)

| Control for the port | Expected comparison |
| --- | --- |
| No lower-cost grid and then a strictly better grid | Check failure and first winning direction, including lower-cost replacement. Leave goal clearing with the native continuation caller. |
| Equal dungeon and town candidates | Preserve dungeon's first eligible direction. In town replay the same RNG draws, including ties encountered before the digging filter. |
| Cardinal and diagonal choices within two grids of a digging goal | Preserve the special cardinal rejection and the same final candidate. Check the ordinary direction order just outside the cutoff. |
| Dark continuation just off a down stair | Exercise the second selector call site. Verify the return depends on leaving the original stair alone and the unseen-bump check. |
| Weighted next step and replan | Show that a weighted step bypasses this selector and retains one-attempt replan semantics. |
| Stair observation with a pending wipe | Verify saved scratch fields, the consumed flag and the next native flow's behavior before ownership expands to stair previews or cache clearing. |

For the subsequent stair slice use the five existing emergency handoff
cases, weighted detour and blocker controls, forced-descend and Recall
refusals, and the different up/down move-cap paths. For dungeon modes
require copied-state controls for
representative failed attempts, normal/urgent/errand precedence, repeated
flow calls, and bravery restore. Compare selected action and trace, queued
keys, return value, state and dependency/RNG order. Run small exact
off/check/switch smoke when a new entry is wired. Check each active arm with
`harness/check-shadow.py --r5 --expect-mode MODE ARM_ROOT`, using `check`
or `switch` for `MODE`. Compare each arm against off with
`harness/diff --suite smoke --expect-impl rust OFF_ROOT ARM_ROOT`. Use Zoo
for the full 45-start comparison, with accepted native and a newly pinned
candidate built under the shipped settings.
[Existing stair cases](../engine/tests/test_stair_handoff.py#L57) ·
[Weighted route controls](../borg-core/src/borg_flow.rs#L1968) ·
[R5 shadow reader](../harness/check-shadow.py) ·
[Exact gameplay diff](../harness/diff)

This task is a source audit. The control list describes validation for
future code ports. A later main carry needs its own receiving pin and a
separate review of #852's resistance-aware caution behavior.
[Track R carry #1297](https://github.com/diegoami/borg-reborn-rs/issues/1297) ·
[Originating change #852](https://github.com/diegoami/borg-reborn-rs/issues/852)

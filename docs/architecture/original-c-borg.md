# The original C Borg

This document describes the C Borg, the automatic player that ships with
Angband in `src/borg/`, as maintained in the better-borg-c fork. It is
the ancestor of borg-rs's core. `borg-core/` is a line-by-line Rust
translation of it, and this repository ships that translation vendored at a
pin. Read this first if you want to know how the Borg plays before you read
the Rust. What is ours is described elsewhere:
[borg-core.md](borg-core.md) covers the vendored core and the errands we
added around it, and [reborn-plan.md](reborn-plan.md) covers our own
redesign.

**Source and citations.** Every function and line number below was checked
in `vendor/better-borg-c/src/borg/` at better-borg-c
`46037038cb0ce3a248ea22c06f02454818dbe4b4` (`pins.json`; its baseline tag `4.2.6-impact-v7`
carries 76 verified fixes). Citations take the form
`borg-<file>.c:<function>` followed by line numbers at that pin. The full
specification,
[BORG_SPEC.md](../../reference/c-borg/BORG_SPEC.md), was written against
*upstream* Angband `e473720e`. Its section numbers are the ones to follow,
but its line numbers differ slightly from the ones given here. Where
better-borg-c differs in behaviour, this document says so.

## The shape of it

The Borg drives the game the way a player would, by sending keys. The
engine's key reader calls a hook, `inkey_hack`. The Borg answers that call
with keys from its own queue, and it thinks only when the queue is empty.
Every think starts from scratch: it reads the game, updates its memory,
scores itself, and then walks a long, fixed list of priorities until one of
them queues keys. Most of its "intelligence" lies in the order of that
list, plus three numeric judgements that the list keeps asking:

| Judgement | Function | Question it answers |
| --- | --- | --- |
| danger | `borg_danger()` | how much damage could I take on grid (y, x) in the next *c* turns? |
| power | `borg_power()` | how good is my kit, including how deep it lets me go? |
| preparation | `borg_prepared(depth)` | may I be on this depth? (`NULL` = yes, or a reason) |

## Perception: what it reads

There are two input channels and one output channel
([BORG_SPEC §1.1](../../reference/c-borg/BORG_SPEC.md#11-the-mechanism-two-input-channels-one-output),
[§4](../../reference/c-borg/BORG_SPEC.md#4-perception-borg_update)).

* **The screen.** `borg-io.c:borg_what_text` (line 55) reads the terminal's
  character buffer. The Borg uses it for the message line (prompts,
  `-more-`) and for the shop header that tells `borg_think()` it is in a
  store.
* **Direct reads of game state**, which is how most of its perception
  works.
  * `borg-item.c:borg_find_all_items` (line 117) reads equipment, quiver,
    pack and stores, and keeps a snapshot in `safe_items[]`.
  * `borg-trait.c:borg_notice_player` (line 2960) reads hit points, depth
    and timed effects from `player->`.
  * `borg-magic.c:borg_cheat_spells` (line 682) reads the spells.
  * `borg-update.c:borg_update_map` (line 257) calls the game's
    `map_info()` for every grid of the visible panel (line 281).
  * `borg-flow-kill.c:borg_update_kill_old` (line 243) reads a tracked
    monster's real hit points and timers.
  * `borg-flow-kill.c:borg_guess_race` (line 874) reads the real race of a
    newly seen monster.
  * Danger and attack scoring use the full race records (`r_info[]`), not
    the player's lore.
* **Memory.** `borg-update.c:borg_update` (lines 1550–3100) patches
  `borg_grids[][]`, the monster list (`borg_kills[]`), the object list
  (`borg_takes[]`) and the stair and feature tracks. It also builds two
  fear maps that danger adds in: `borg_fear_region[][]`, one value per
  11×11 region, from `borg_fear_regional` (line 729); and
  `borg_fear_monsters[][]`, one value per grid, from `borg_fear_grid`
  (line 641). On a level change it forgets everything.
* **Messages.** `borg-messages.c:borg_get_messages` (lines 183–298) reads
  the message line. `borg-messages-react.c:borg_react_prompted` (line 95)
  answers prompts at once, for example "Direction?", "Die?" or
  "Set recall depth". `borg-messages.c:borg_parse` (line 1332) turns the
  remaining messages into tagged reactions (`KILL:`, `DIED:`, `PAIN:`,
  ...), which `borg_update()` consumes on the next think.

Off-panel grids keep what the Borg last saw there. The Borg does not learn
of monsters the player cannot see, but it does read the exact race,
hit points and timers of the ones it has observed.

## The think loop

### One keypress

`borg.c:generate_keypress` (lines 268–394) runs every time the engine
wants a key
([BORG_SPEC §2.2](../../reference/c-borg/BORG_SPEC.md#22-one-call-of-generate_keypress)):

1. Read the player's state, and check for death or a stop condition.
2. Read the message line. A prompt that needs an answer returns its key
   immediately.
3. **If a key is already queued, return it.** A think that queues five
   keys is therefore followed by four calls that only drain the queue.
4. **If the player is resting or repeating a command, return `EVT_NONE`.**
   The game's own rest loop then runs without consulting the Borg. This is
   how resting works: the Borg queues `R` plus a count, and the game
   interrupts the rest itself when a monster appears.
5. Otherwise call `borg_think()` until it returns true, then return the
   first queued key. A think that queued nothing stops the Borg.

Every action is one complete key sequence: the command, the item letter,
any direction or count, and usually a trailing `ESCAPE`, so that an
unexpected prompt is dismissed rather than answered by the next action.
The queue is a ring of 8192 keys, managed by `borg-io.c:borg_keypress`
(line 315) and `borg-io.c:borg_inkey` (line 398).

### `borg_think`

`borg-think.c:borg_think` (lines 179–256) makes these calls, in order:

1. `borg_find_all_items()` and `borg_check_save()`;
2. `borg_cheat_spells()`;
3. a change back if the player is shapechanged;
4. `borg_notice(true)`, which rebuilds the trait vector `borg.trait[]`
   from the items (`borg-trait.c:borg_notice`, line 2826);
5. `borg_update()`;
6. `borg.power = borg_power()`.

If the screen shows a "Stor" or "Home" header, it returns
`borg_think_store()`. Otherwise it advances the Borg's clock
(`borg.time.now` counts thinks, not game turns) and the anti-bounce
counter (`borg.antibounce_count`, a "no progress" counter that reaches
thresholds at 300, 500 and 700), and then returns `borg_think_dungeon()`.

### `borg_think_dungeon`: the priority ladder

`borg-think-dungeon.c:borg_think_dungeon` (lines 1276–2324) is a fixed
list of rungs. Each rung either queues keys and returns true, or falls
through to the next. A goal that was committed on an earlier turn is
continued through `borg_flow_old(GOAL_*)` on its own rung, so a walk
persists across turns until a higher rung interrupts it. The order at the
pin, with line numbers:

| # | Rung | Line |
| --- | --- | --- |
| 1 | clock windows, destroy an ignored item underfoot, level boredom | 1317 |
| 2 | stair scum at depth 1 when fleeing to town | 1365 |
| 3 | `borg_check_bounciness()` (anti-bounce thresholds), `borg_check_breeders()` | 1390, 1394 |
| 4 | reset `borg.avoidance` to current HP (wipes the danger cache if it changed); stair leash for weak characters | 1398–1455 |
| 5 | lunal mode, munchkin mode; "back to the correct depth" if `borg_prepared(depth)` fails | 1475, 1487, 1497 |
| 6 | `borg_think_dungeon_light()`; continue a `GOAL_DIGGING` corridor | 1515, 1529 |
| 7 | **`borg_caution()`**: survive first | 1533 |
| 8 | no light, cut, poisoned or starving: light, wear, recover, crush, shop | 1536–1563 |
| 9 | `borg_must_return_to_town()` then `borg_leave_level()` | 1566 |
| 10 | `borg_play_magic(false)`; wear a weapon if bare-handed or digging | 1572–1581 |
| 11 | `borg_flow_kill_corridor()`, then **`borg_attack(false)`** | 1585, 1589 |
| 12 | wear, swap and wear rings; take a visible item within 5 | 1594–1606 |
| 13 | remove gear, dump the quiver, `borg_check_light()` (detections) | 1614–1624 |
| 14 | recover: `GOAL_RECOVER`, `borg_recover()`, `borg_flow_recover(50)` | 1628–1637 |
| 15 | `borg_perma_spell()` (keep buffs up); casters rest on a stair for mana | 1641–1690 |
| 16 | `goal.less` → up stair; `goal.fleeing` → any stair | 1730–1770 |
| 17 | glyphs against Morgoth | 1779–1783 |
| 18 | take within 5, kill within 20, take within 10, aim at a monster | 1790–1826 |
| 19 | use, test, enchant, recharge; drop junk | 1835–1859 |
| 20 | take anything reachable (250) | 1865–1871 |
| 21 | leave if bored or the pack is full of loot; dive if prepared for depth + 5 | 1879–1936 |
| 22 | continue `MISC`, `DARK`, `XTRA`, `BORE`, `VAULT`; vault block | 1946–1983 |
| 23 | explore near (`borg_flow_dark(true)`), kill and take far, leave level, explore far | 1987–2016 |
| 24 | visit a shop; study; detect; wander (`borg_flow_spastic`); `borg_flow_kill_direct` | 2022–2079 |
| 25 | waiting for recall and safe: `R 9 Enter` | 2086 |
| 26 | nothing worked: three "boosted bravery" passes, phase, teleport, recall, `borg_twitchy()` | 2128–2319 |

In the bravery passes, `borg.avoidance` is raised (to 2 × current HP, then
4 × max HP, then 30000) and `borg_think_dungeon_brave()` runs
(`borg-think-dungeon.c`, line 950). That reduced ladder starts with
`borg_defend()` at depth 100 and `borg_attack(true)`. A stuck Borg
therefore walks through danger it would otherwise route around
([BORG_SPEC §2.8](../../reference/c-borg/BORG_SPEC.md#28-the-dungeon-decision-ladder)).

Read the ladder as a policy: safety first, then fighting, then loot that
is close, then housekeeping, then loot that is far, then descending, then
exploring.

## Danger

`borg-danger.c:borg_danger` (lines 2823–2864) answers "how dangerous is
grid (y, x) over the next `c` turns?" It sums four contributions and caps
the result at 2000:

* the regional fear `borg_fear_region[y/11][x/11] × c` (not in vaults,
  only to depth 80; capped at 300 on depth 100);
* the per-grid fear `borg_fear_monsters[y][x] × c`, but only while the
  anti-bounce counter is at most 200;
* `borg_danger_one_kill()` for every tracked monster
  (`borg-danger.c:borg_danger_one_kill`, lines 2288–2800);
* nothing else. The caller's `full_damage` argument is overwritten with
  `true` (line 2848).

`borg_danger_one_kill()` turns the monster's speed into a number of moves
per player turn, then computes two terms:

* a melee term, `v1`, from `borg_danger_physical()` (line 64). It counts
  only if the monster can reach the grid this turn; the function simulates
  the monster's step.
* a spell term, `v2`, from `borg_danger_spell()` (line 556). It counts
  only with range and line of sight, again allowing for a fast monster
  that moves first.

Both are reduced for sleeping, confused, stunned or afraid monsters, and
"friends" are counted extra when the Borg is weak. The result is
**`max(v1, v2)`, not their sum** (line 2794).

Danger feeds almost every decision:

* **Flow.** A grid whose danger exceeds a fear threshold derived from
  `borg.avoidance` is marked "icky" and never entered (see Flows).
* **Caution.** `pos_danger = borg_danger(here, 1, true, false)`
  (`borg-caution.c`, line 960) is compared with current HP and with
  fractions of `borg.avoidance` to choose healing, retreat or escape.
* **Escape.** `borg_escape()` picks its band of escapes from
  `pos_danger / avoidance`.
* **Attack.** Melee adds the danger of the target to its damage, so that
  threats are hit first.
* **Rest.** The Borg rests only where the danger is 0, or below a small
  fraction of avoidance.

See [BORG_SPEC §8](../../reference/c-borg/BORG_SPEC.md#8-danger-estimation).

## Flows

The flow is the Borg's pathfinder: a breadth-first flood **from the goal
grids outward**, one cost byte per grid, followed by a greedy downhill step
from the Borg's own grid. It is not A*. Each goal builder follows the same
five steps ([BORG_SPEC §6](../../reference/c-borg/BORG_SPEC.md#6-flow-how-the-borg-decides-where-to-go)):

1. `borg-flow.c:borg_flow_clear` (lines 180–200) sets every grid to
   unreached. The know and icky danger caches are wiped only when
   `borg_danger_wipe` is set; otherwise they survive from flood to flood.
2. `borg-flow.c:borg_flow_enqueue_grid` (lines 519–592) puts each goal
   grid at cost 0, unless it is too dangerous.
3. `borg-flow.c:borg_flow_spread` (lines 269–514) gives each reachable
   neighbour cost + 1. It skips walls (unless tunnelling), unknown grids
   (if `avoid` is set), traps, other shops' doors, and grids whose danger
   exceeds the fear threshold. That threshold is `avoidance × 3/10`
   normally, ×2 with a scary monster on the level, ×25 after 5000 turns on
   the level, and ×100 when starving (lines 448–480).
4. `borg-flow.c:borg_flow_commit` (lines 597–620) fails if the Borg's own
   grid costs 250 or more. Otherwise it copies cost to flow and sets
   `borg.goal.type`. **It queues no key.**
5. `borg-flow.c:borg_flow_old` (lines 1134–1215) takes one step down the
   committed flow through `borg_play_step()` (lines 625–1109), which turns
   the step into walking, opening, tunnelling or disarming.

The goal builders, and the `GOAL_*` each commits (`borg-trait.h`, lines
32–41):

| Goal | Builder | Target |
| --- | --- | --- |
| `GOAL_KILL` | `borg-flow-kill.c:borg_flow_kill` (1633), `borg_flow_kill_aim` (1938) | a monster, or a grid with a shot at one |
| `GOAL_TAKE` | `borg-flow-take.c:borg_flow_take` (426), `borg-flow-misc.c:borg_flow_vein` (348) | an object, or a treasure vein |
| `GOAL_MISC` | `borg-flow-misc.c:borg_flow_shop_entry` (772), `borg-flow-glyph.c:borg_flow_glyph` (81) | a shop door, a glyph spot |
| `GOAL_DARK` | `borg-flow-dark.c:borg_flow_dark` (947) | unexplored grids: methods 1–3 "near" (torch-lit, straight line, short flood), methods 4–5 "far" |
| `GOAL_XTRA` | `borg-flow-misc.c:borg_flow_spastic` (467) | wander and search |
| `GOAL_BORE`, `GOAL_FLEE` | `borg-flow-stairs.c:borg_flow_stair_less` (137), `_more` (185), `_both` (76) | a stair: leave without fleeing, or flee |
| `GOAL_VAULT` | `borg-flow-misc.c:borg_flow_vault` (886) | a diggable vault wall |
| `GOAL_RECOVER` | `borg-flow-misc.c:borg_flow_recover` (231) | a safe grid to rest on |
| `GOAL_DIGGING` | `borg-flow-kill.c:borg_flow_kill_corridor` (2050), `borg_flow_kill_direct` (2544) | an anti-summon corridor |

There is a single flood buffer, so committing a new goal replaces the old
one. Raising `borg.avoidance` raises the fear threshold and opens routes
the Borg would otherwise refuse.

## Caution, escapes and rest

### `borg_caution`

`borg-caution.c:borg_caution` (lines 834–2078) runs before every routine
goal. It is a chain of "if the situation is X, do Y" checks
([BORG_SPEC §9.1](../../reference/c-borg/BORG_SPEC.md#91-borg_caution--the-reactive-layer)).
In source order:

1. Note nasty conditions: light running out, hunger, blindness,
   confusion.
2. At depth 100 and not ready for Morgoth, rise.
3. **Scary logic.** More than three escapes on this level, or a scary
   monster on the level (`borg.mon.scary`, set by
   `borg-flow-kill.c:borg_near_monster_type`, line 2657), sets
   `goal.leaving`, `goal.fleeing` and sometimes `fleeing_to_town`.
   `borg_surrounded()` is evaluated here.
4. **Defend and heal** (lines 1036–1064). A mage tries `borg_defend()`
   before `borg_heal()`; other classes heal first. better-borg-c adds one
   branch: a fleeing Borg still heals if this turn is survivable
   (`pos_danger < HP`, line 1057).
5. Swap to backup gear when danger exceeds avoidance/3 (line 1070).
6. Rest while waiting for recall. Not stocked to stay
   (`borg_restock()`, line 1106), or excessive danger: set leaving or
   fleeing.
7. Take the stair underfoot if leaving or fleeing, and handle critical
   needs (no light, starving, breeders at low level).
8. **Flee on foot** (lines 1385–1459). With `borg.status.desperate` set
   for one flood, dash for an up stair within 2–3 steps (or within 20 with
   a scary monster on the level and a low-level character), or a down
   stair within 2.
9. **Strategic retreat** (line 1472). Step to a safer neighbour when
   `pos_danger > avoidance/3` or the Borg may be surrounded. better-borg-c
   skips the retreat when a lone adjacent monster dies to one blow
   (`borg_adjacent_kill_in_one_blow`, line 807).
10. **`borg_escape(pos_danger)`** (line 1697).
11. Back away from a single adjacent monster; then cure confusion, fear,
    wounds and poison; `borg_recall()` when fleeing; buy time while recall
    is active (`borg_caution_phase(50, 1)`, line 2037); and as a last
    resort try unknown items.

### Escapes

`borg-escape.c:borg_escape` (lines 656–1302) works in bands of `b_q`
(the danger) against `borg.avoidance`. Every multiple is raised by 0.3
when `borg_plays_risky` is set:

* **Band 1, "about to die"** (line 732). This band applies above
  1.5 × avoidance with no unique near, above 1.7 × with a unique near
  (above depth 95), above 4.5 × in any case, or when heavily stunned. The
  Borg tries, in order, the stair underfoot, dimension door, teleport and
  teleport level, and finally a phase without checking where it lands.
* **Band 2** (line 851): teleport, then a phase if it helps.
* **Band 3, "not too bad"** (line 904). This band applies above
  1.0 × avoidance with no unique near. The Borg takes the stair
  underfoot, or a phase if `borg_caution_phase(25, 2)` says the landing
  area is safer.

The helpers are:

| Helper | Line | Purpose |
| --- | --- | --- |
| `borg_caution_phase` | 326 | samples 100 random landing grids; is a phase likely to help? |
| `borg_caution_teleport` | 419 | the same question for teleport |
| `borg_allow_teleport` | 544 | not in an arena, not on a no-teleport grid, no no-teleport curse |
| `borg_escape_stair` | 523 | take the up stair underfoot |
| `borg_dimension_door`, `borg_shadow_shift` | 571, 562 | short-range escapes |
| `borg_recall` | 55 | read or zap recall, pre-answering the recall-depth prompt |

### Rest

`borg-recover.c:borg_recover` (lines 59–469) cures and then rests with
`R & Enter` or `R` plus a count. It rests only where
`borg-flow-misc.c:borg_check_rest` (line 1221) approves the grid and the
danger is low. Otherwise `borg_flow_recover()` walks to a safer grid
first.

## Attack choice

`borg-fight-attack.c:borg_attack` (lines 5153–5324)
([BORG_SPEC §7](../../reference/c-borg/BORG_SPEC.md#7-fighting)) works in
four steps:

1. Collect the targets: tracked monsters seen in the last two thinks, on
   the panel, in view and within `max_range`, stored in
   `borg_temp_y/x[]`.
2. Set `borg_simulate = true`. Score every attack type
   `g = 0 .. BF_MAX` with `borg_calculate_attack_effectiveness(g)`
   (lines 3781–5129). Activations from `BF_ACT_STARLIGHT` on are scored
   only with random artifacts.
3. If the best score is positive, note `# Performing attack type %d with
   value %d`.
4. Clear `borg_simulate` and call the same scorer again. This time it
   queues the keys.

The same function both scores and acts. The `BF_*` order is: rest (wait for
the monster), thrust (melee), object (throw), launch (missile), then
spells, prayers, rods, staffs, wands, rings, dragon armour and
activations.

* **Melee.** `borg_attack_aux_thrust` (line 269) handles adjacent targets
  only. Damage comes from `borg_thrust_damage_one` (line 101): weapon dice
  × slay multiplier + bonuses, × blows, × hit chance, capped at twice the
  monster's HP for non-uniques. To that it adds
  `borg_danger_one_kill(here, 2, ...)`, the target's threat, cut to 1/10
  for a partial kill once max level exceeds 15. The Borg will not attack
  while afraid, and it avoids waking hard sleeping monsters.
* **Missiles and bolts.** `borg_attack_aux_launch` (line 2076) and the
  spell scorers go through `borg_launch_bolt` (line 1682). It sums the
  expected damage along the projection path, and the scorer picks the
  best target.
* **Spells.** `borg_attack_aux_spell_bolt` (line 2394) subtracts a mana
  penalty: a standard penalty, more if the cost far exceeds the damage,
  and more again for reserve mana. It refuses to spend the mana kept for
  a final teleport.
* **Defence.** `borg-fight-defend.c:borg_defend` (lines 3538–3605) is the
  same pattern for `BD_*` defences (speed, resistances, glyphs, teleport
  other, banishment), scored by `borg_calculate_defense_effectiveness`
  (line 3434). `borg-fight-perm.c:borg_perma_spell` (line 751) keeps buffs
  running.
* **Munchkin mode.** It uses its own attackers,
  `borg-attack-munchkin.c:borg_munchkin_mage` (line 47) and
  `borg_munchkin_melee` (line 193).

## Town and shopping

### Power

`borg-power.c:borg_power` (lines 1883–1914) adds four things
([BORG_SPEC §5](../../reference/c-borg/BORG_SPEC.md#5-scoring-itself-borg_power)):

* `borg_power_equipment()` (line 43): weapon damage, blows, slays, to-hit,
  speed, HP, mana, AC, resistances and abilities;
* `borg_power_inventory()` (line 1122): food, fuel, potions, scrolls and
  devices, with diminishing returns;
* **40,000 per depth `borg_prepared()` accepts**;
* the backup swap values.

The `borg_worships_*` options switch weight branches. With
`borg_uses_dynamic_calcs`, formulas from `borg.txt` replace the equipment
and inventory terms.

Candidates are valued by simulation. The Borg writes the candidate into
`borg_items[]`, re-runs `borg_notice()`, calls `borg_power()`, and restores
the slot from `safe_items[]`. Wearing, buying, selling and dropping all
use this compare-before-and-after pattern.

`borg-prepared.c:borg_prepared` (lines 538–686) gates vertical movement.
It checks the restock needs to *stay* (`borg_restock`, line 694),
"Clevel < depth", cumulative per-band requirements (light, cures, recall,
escapes, free action, see invisible, basic resists, ...), Healing potions
from depth 82, and unique gates near the bottom of the dungeon.

### The store loop

* **Choosing and walking.** `borg-think-store.c:borg_choose_shop`
  (lines 86–280) chooses the shop and the ware. The ladder then walks
  there with `borg_flow_shop_entry()`.
* **Inside.** `borg-think-store.c:borg_think_store` (lines 285–357) runs
  `borg_best_stuff()` (the best equipment combination, which can include
  the home's items), swaps out a digger, and then makes **one transaction
  per think** while `borg_choose_shop()` still wants one:
  `borg_think_shop_sell()` (`borg-store-sell.c`, line 1090) or
  `borg_think_shop_buy()` (`borg-store-buy.c`, line 1184). Otherwise it
  leaves with four `ESCAPE`s.
* **Order of choice.** `borg_choose_shop()` handles emergencies first: a
  cut or poison sends it to the temple; no food or no light to the general
  store, buying immediately. Then come the numbered steps
  (lines 168–269):
  * step 0: money scum;
  * step 1: sell to the home what is useful later;
  * step 2: sell useless items to the shops;
  * step 3: buy useful items from the shops;
  * step 4: buy useful items from the home;
  * step 5: take items from the home to sell;
  * step 6: buy interesting items for the home;
  * steps 7A and 7B: take backup weapons and armour from the home.
* **The home.** Each step compares `borg_power()` before and after for the
  Borg, or `borg-home-power.c:borg_power_home` (line 531) for the home. A
  small memory of recent purchases (`bought_item_*`) prevents buy/sell
  loops.

See [BORG_SPEC §12](../../reference/c-borg/BORG_SPEC.md#12-the-borg-in-town).

## Configuration: `borg.txt`

`borg-init.c:borg_init_txt_file` (lines 203–383) reads
`src/borg/borg.txt`. The defaults are in the `borg_settings[]` table
(`borg-init.c`, line 57). At this pin it has **37** settings: the 35 in
[BORG_SPEC §13](../../reference/c-borg/BORG_SPEC.md#13-configuration-borgtxt)
plus better-borg-c's `borg_munchkin_by_class` (true) and
`borg_munchkin_class_depth` (6). With those two, warriors, druids and
rangers get a munchkin start even when `borg_munchkin_start` is off.

The settings fall into these groups:

| Group | Settings |
| --- | --- |
| power weights | `borg_worships_{damage,speed,hp,mana,ac,gold}` |
| risk and diving | `borg_plays_risky`, `borg_kills_uniques`, `borg_no_deeper` |
| modes | `borg_munchkin_*`, `borg_lunal_mode`, `borg_self_lunal`, `borg_money_scum_amount`, `borg_self_scum` |
| items | `borg_uses_swaps` (true), `borg_enchant_limit`, `borg_chest_fail_tolerance` |
| stops and respawn | `borg_stop_dlevel`, `borg_stop_clevel`, `borg_stop_king` (true), `borg_respawn_*`, `borg_cheat_death` |
| logging and misc | `borg_verbose`, `borg_dump_level`, `borg_save_death`, `borg_autosave`, `borg_delay_factor`, `borg_stop_on_bell`, ... |
| formulas | `borg_uses_dynamic_calcs` plus the `[BEGIN FORMULA SECTION]` block (`borg.txt`, line 348 onwards): power, depth and restock formulas |

The modes (lunal, munchkin, stair scum, money scum, digging, vault,
desperate, boosted bravery) do not add rungs to the ladder. They replace
part of it for a while, or relax the flow's danger check
([BORG_SPEC §13.1](../../reference/c-borg/BORG_SPEC.md#131-modes-that-change-goal-selection)).

The player starts the Borg from the `^Z` menu,
`borg-commands.c:do_cmd_borg` (line 1557).

## Files and how they fit

All 59 `.c` files are listed. The borg-core column names the translated
Rust file in `borg-core/src/`.

| C file | Role | borg-core |
| --- | --- | --- |
| `borg.c` | entry hook, `generate_keypress`, stop and death checks | `borg.rs` |
| `borg-think.c` | `borg_think`: read, notice, update, power, dispatch | `borg_think.rs` |
| `borg-think-dungeon.c` | the dungeon ladder; lunal, munchkin and brave passes | `borg_think_dungeon.rs` |
| `borg-think-dungeon-util.c` | light, stair scum, leave level, money scum, vault digging | `borg_think_dungeon_util.rs` |
| `borg-think-store.c` | `borg_choose_shop`, `borg_think_store` | `borg_think_store.rs` |
| `borg-store.c`, `borg-store-buy.c`, `borg-store-sell.c` | shop memory and the buy and sell steps | `borg_store.rs`, `borg_store_buy.rs`, `borg_store_sell.rs` |
| `borg-home-notice.c`, `borg-home-power.c` | the home's contents and its power | `borg_home_notice.rs`, `borg_home_power.rs` |
| `borg-danger.c` | `borg_danger`, per-monster danger | `borg_danger.rs` |
| `borg-caution.c` | `borg_caution`, `borg_heal` | `borg_caution.rs` |
| `borg-escape.c` | `borg_escape`, phase and teleport checks, recall | `borg_escape.rs` |
| `borg-fight-attack.c` | `borg_attack` and its `BF_*` scorers | `borg_fight_attack.rs` |
| `borg-fight-defend.c` | `borg_defend` and its `BD_*` scorers, panel shift | `borg_fight_defend.rs` |
| `borg-fight-perm.c` | `borg_perma_spell` (buffs) | `borg_fight_perm.rs` |
| `borg-attack-munchkin.c` | munchkin-mode attackers | none (no Rust file of that name, and no `borg_munchkin_mage` in `borg-core/src`) |
| `borg-flow.c` | flood, commit, step (`borg_play_step`) | `borg_flow.rs` |
| `borg-flow-dark.c`, `-glyph.c`, `-kill.c`, `-misc.c`, `-stairs.c`, `-take.c` | goal builders; `-kill.c` also tracks monsters, `-take.c` objects | `borg_flow_dark.rs`, `borg_flow_glyph.rs`, `borg_flow_kill.rs`, `borg_flow_misc.rs`, `borg_flow_stairs.rs`, `borg_flow_take.rs` |
| `borg-update.c` | `borg_update`: map, monsters, objects, fear maps, level change | `borg_update.rs` |
| `borg-cave.c`, `borg-cave-light.c`, `borg-cave-util.c`, `borg-cave-view.c` | grid memory, lighting, view | `borg_cave.rs`, `borg_cave_light.rs`, `borg_cave_util.rs`, `borg_cave_view.rs` |
| `borg-projection.c` | line of sight, projectability, distance, targeting | `borg_projection.rs` |
| `borg-messages.c`, `borg-messages-react.c` | message line, prompts, reactions | `borg_messages.rs`, `borg_messages_react.rs` |
| `borg-io.c` | screen reads, key queue, notes | `borg_io.rs` |
| `borg-trait.c`, `borg-trait-swap.c` | `borg_notice`: the trait vector; swap items | `borg_trait.rs`, `borg_trait_swap.rs` |
| `borg-power.c`, `borg-prepared.c` | `borg_power`; `borg_prepared` and `borg_restock` | `borg_power.rs`, `borg_prepared.rs` |
| `borg-formulas.c`, `borg-formulas-calc.c` | the `borg.txt` formula language | `borg_formulas.rs`, `borg_formulas_calc.rs` |
| `borg-item*.c` (analyze, activation, decurse, enchant, id, use, val, wear) and `borg-item.c` | item reading, naming, use, wear and best kit | `borg_item.rs`, `borg_item_*.rs` |
| `borg-inventory.c`, `borg-junk.c`, `borg-light.c` | pack, crushing and dropping, light and detections | `borg_inventory.rs`, `borg_junk.rs`, `borg_light.rs` |
| `borg-magic.c`, `borg-magic-play.c` | spell tables, studying and trying spells | `borg_magic.rs`, `borg_magic_play.rs` |
| `borg-recover.c` | cure and rest | `borg_recover.rs` |
| `borg-init.c` | start-up, `borg.txt` | `borg_init.rs` |
| `borg-commands.c` | the `^Z` menu, debug displays | `borg_commands.rs` |
| `borg-log.c`, `borg-reincarnate.c`, `borg-util.c` | death logs, respawn, helpers | `borg_log.rs`, `borg_reincarnate.rs`, `borg_util.rs` |

borg-core also has files with no C counterpart: the `borg_town_*.rs`
files, `params.rs`, `core_trace.rs`, `errands_plan.rs` and others. Those
are not part of the C Borg; [borg-core.md](borg-core.md) describes them.

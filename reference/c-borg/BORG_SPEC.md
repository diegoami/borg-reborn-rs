# The C Borg: purpose and architecture

**Status:** specification, not a proposal. It explains what the original
Angband C Borg *is for* and how it is put together, so that a rebuild — or a
reader trying to understand a defect — does not have to rediscover it.

**Source of truth:** the C Borg in
[`angband/angband`](https://github.com/angband/angband) at

```
e473720ed88a0f0c7ca66815cef3000eb4a38bc3
```

directory `src/borg/` (59 `.c` files, 64,147 lines of C plus headers), read
at that commit. See §0.

**This document does not list the Borg's bugs.** They are on this fork's
issue board, and their state changes there (§18). What this document does
is explain **what the Borg is for and how it is built**, so that the bugs
make sense: a defect is only legible against the behaviour it disturbs.
§11 describes the *shapes* of defect the architecture produces, each with an
example read from the source, so that a rebuild can recognise them. The
examples are described by their code, not by finding number, and nothing in
this document records whether any finding is open, fixed or disputed.

**Scope:** the C Borg only. Nothing here describes the Neo TypeScript port
or the Mandos derivative.

**Reading rules.**

* "Must" means: the current C Borg does this, and a rebuild that does not
  will play differently. §16 says which of those are observable behaviour,
  which are decision-changing constants and which are configuration.
* **Citations** are written `file:function:lines` (for example
  `borg-flow.c:borg_flow_commit:585-608`). The file is under `src/borg/`
  unless another directory is named; the lines are at the commit above and
  lie inside the named function (or, for a header, contain the named
  identifier). Every one was checked mechanically; see
  `docs/audit/c-reports/spec-checks/RECHECK.md`.
* **Code blocks** are either transcribed from the source, with their
  citation in the line above the block, or labelled **pseudocode**.
  Transcriptions may omit comments, blank lines and unrelated statements;
  omissions are marked `...`. Pseudocode uses real identifiers wherever it
  names one.
* **Call graphs** were derived by grepping call sites at the commit, not
  from memory. Where a graph is partial, it says so.

---

## 0. The upstream commit this describes

| Field | Value |
| --- | --- |
| Repository | `angband/angband` |
| Commit | `e473720ed88a0f0c7ca66815cef3000eb4a38bc3` |
| Commit date | 2026-10-27 |
| Upstream `master` head when last checked | the same commit, on 2026-11-06 |
| Path read | `src/borg/` (plus the engine files named in citations) |
| Size read | 59 `.c` files, 64,147 lines (plus headers) |

The commit id is the pin. The statement "it is upstream's head" was true
when checked and is not a property of this document; when upstream moves,
the citations still hold *at the pin*, and re-reading them at a newer head
is a separate piece of work. The pin is also this session's recorded C
baseline (the C entry of the baselines in `session.json`) and the
catalogue's C source pin (the commit under sources → c in
`docs/c-bugs/catalogue.json`), and the local
`c-borg-reference` branch carries it unmodified:

```sh
gh api repos/angband/angband/commits/master --jq '.sha'     # head today
git rev-parse origin/c-borg-reference                         # the pin
```

---

## 1. What the Borg is

### 1.0 What it is for

The Borg exists to **play Angband well without a human**. It is the
original autoplayer: the thing you leave running overnight to push a
character deeper than anyone would sit and watch.

That purpose shapes everything about it. An autoplayer that has to be
watched is not an autoplayer, so it must be able to play for hours without
supervision: it decides, it acts, it recovers from its own mistakes, and it
keeps going. It plays to win rather than to survive — it dives when its
preparation checks say it may (§5.5), spends consumables when a fight is
worth it, and pushes towards Morgoth. And it acts on the game through the
same channel a player does: **keystrokes**.

The consequence is that the Borg is not a strategy and not a set of rules.
It is a **decision loop**: perceive, evaluate, act, repeat. Its
"intelligence" is a fixed priority ladder of goals (§2.8), most of whose
rungs are scoring functions over the possibilities it can perceive.

### 1.1 The mechanism: two input channels, one output

The Borg's **output** is a queue of keystrokes (§2.6). Every ordinary game
action goes through it. There is one exception, the cheat-death path, which
calls a game function directly (§1.2); the Borg also changes game *settings*
directly (§1.2).

Its **input** comes through two channels, and a rebuild needs both.

**Channel A — the screen.** Read with `borg_what_text()` (a wrapper over
the terminal's character/attribute buffer):

* the message line (row 0), including `-more-` and yes/no prompts
  (`borg-messages.c:borg_get_messages:183-294`, §4.6);
* the store/home header, which is how `borg_think()` knows it is in a shop
  (`borg-think.c:borg_think:222-224`).

**Channel B — direct reads of game state.** This is not an exception or a
cheat bolted on; it is how most perception works. On every call of
`borg_think()`:

* **its own gear.** The first call is `borg_find_all_items()`, which runs
  `borg_equipment()` (reads `player->body.slots[]`), then the quiver
  (`player->upkeep->quiver[]`), then `borg_inventory()` (reads
  `player->upkeep->inven[]`), then the stores, and snapshots the result
  into `safe_items[]` (`borg-item.c:borg_find_all_items:117-131`,
  `borg-inventory.c:borg_equipment:172-209`,
  `borg-inventory.c:borg_inventory:214-250`). Each object goes through
  `borg_item_analyze()`, which copies `tval`/`sval`, the *known* parts of
  the object (`obj->known`) and whether its runes are known
  (`borg-item-analyze.c:borg_item_analyze:450-498`).
* **its own status.** `borg_notice_player()` reads hit points, mana,
  depth, experience, gold, recall state and the timed effects straight
  from `player->` (`player->chp`, `player->timed[TMD_BLIND]`, …); it is
  also called at the top of every `generate_keypress()` (§2.2).
* **its spells.** `borg_cheat_spells()` reads
  `player->class->magic.books[]` and `player->spell_flags[]`
  (`borg-magic.c:borg_cheat_spells:677-718`).
* **the map panel.** `borg_update_map()` calls the game's own `map_info()`
  for every grid of the visible panel. `map_info()` returns what the game
  would draw there: the feature and the remembered objects from the
  *player's* memory (`player->cave`, so an object stays listed after it
  goes out of view), whether the grid is in view, and a monster only if the
  player can currently see it (`src/cave-map.c`). The Borg also asks `cave` directly
  whether a panel grid holds a visible trap, which shop it is, whether it
  is webbed, and it reads a visible monster's display attribute and
  mimic state through `cave_monster()`
  (`borg-update.c:borg_update_map:279-353`,
  `borg-update.c:borg_update_map:572-587`).
* **monster condition.** For a tracked monster on a known grid,
  `borg_update_kill_old()` reads the real `struct monster`: exact
  `hp`/`maxhp`, the sleep, fear, confusion and stun timers, `mspeed` and
  the game's monster index. The source says so itself: "this is a cheat.
  Borg does not look at the bar at the bottom"
  (`borg-flow-kill.c:borg_update_kill_old:243-330`).
* **identities of what it sees.** When a new monster sighting is matched,
  `borg_guess_race()` returns the real race of the monster on that grid
  (`square_monster(cave, …)->race->ridx`, with the comment "this is an real
  cheat"; `borg-flow-kill.c:borg_guess_race:892-907`). For a new floor
  object, `borg_guess_kind()` returns the real kind of the top unignored
  object on the grid, from `cave` rather than the player's memory
  (`borg-flow-take.c:borg_guess_kind:72-84`, through
  `borg-flow-take.c:borg_get_top_object:50-64`). Opening chests reads the
  real chest's `pval` (trap level) in `borg_play_step()`. The screen
  symbol is not how the Borg identifies anything.
* **race and kind data.** Danger and attack scoring use the full monster
  race record `r_info[]` (blows, spells, flags), not the player's lore,
  and items use `k_info[]`/`e_info[]`.
* **cave predicates.** A handful of decisions consult `cave` for facts
  about the Borg's surroundings: `square_isvault()` (danger, retreat),
  `square_isno_teleport()` and `player->upkeep->arena_level` (teleport),
  `square_in_bounds_fully()` (flow and neighbour scans), `square_shopnum()`
  (stores), and `cave->width`/`cave->height` (panel arithmetic).

What the Borg does **not** do, as far as this reading of the source found:
it does not read the player's map memory outside the current panel (it
remembers what earlier panels showed), it does not learn of monsters the
player cannot see (a tracked monster starts from a grid where `map_info()`
reported a visible monster, or from a monster named in a message), and it does not scan the level's
monster or object lists for things it has not observed. The reads above go
beyond what the screen shows *about* an observed thing — its exact race,
kind, hit points and timers — but not beyond *which* things are observed.

The upstream comment that explains the design is in
`borg-think.c:borg_think:179-256` (the comment block above it): the
"cheat" functions for inventory, equipment, panel and spellbooks "can be
replaced by (very expensive) parse functions, which cause an insane amount
of screen flashing". Upstream chose direct reads. A rebuild that parses the
inventory off the screen is a different, slower program.

### 1.2 What it is not

* **It does not consume the game's RNG while it thinks.** During
  `borg_think()` the RNG is switched to the quick generator seeded from
  the Borg's own `borg_rand_local`, and restored afterwards (§2.2,
  `borg.c:generate_keypress:355-374`). This covers the thinking pass only:
  code that runs before the swap — prompt handling in
  `borg_react_prompted()`, including the random race/class choice in
  `borg_reincarnate_start()` — uses the game's RNG.
* **It is not a pure keyboard player.** It sets game options when it
  starts (`borg-init.c:borg_reinit_options:403-444`: no `auto_more`,
  `pickup_always` on, original keyset, `easy_open`/`easy_alter` off, …),
  changes the delay factor, turns `cheat_live` on at start (with
  `borg_cheat_death`) and when it respawns, marks
  the character `NOSCORE_BORG`, and on the "Die?" prompt with
  `borg_cheat_death` set it calls `do_cmd_wiz_cure_all()` directly
  (`borg-messages-react.c:borg_react_prompted:105-139`). The option and
  score changes are settings; a rebuild must reproduce the option set or
  the screen it reads will look different. **The cure-all is not a
  setting: it is a game action** (it heals hit points and mana, restores
  stats and experience, removes curses and clears conditions) taken outside
  the keyboard. It is the one exception to the keystroke-only rule, and it
  happens only on the cheat-death path.
* **It does not run on a timer.** It runs when the game asks for a key.

---

## 2. The control loop

### 2.1 Taking over the keyboard

The engine exposes a function pointer `inkey_hack` (`src/ui-input.c`).
When it is set, the engine's key reader calls it instead of waiting for a
real keypress.

Transcribed from `borg.c:borg_entry_point:76-79` and
`borg.c:borg_update_entrypoint:84-92`:

```c
static struct keypress borg_entry_point(int flush_first)
{
    return borg_save_keypress(generate_keypress(flush_first));
}

void borg_update_entrypoint(bool start)
{
    if (start) {
        inkey_hack = borg_entry_point;
    }
    else {
        inkey_hack = NULL;
    }
}
```

The player starts the Borg from the `^Z` command menu (`do_cmd_borg()` in
`borg-commands.c`), which calls `borg_update_entrypoint(true)`.
`borg_save_keypress()` only records the key in a history ring for
diagnostics.

### 2.2 One call of `generate_keypress()`

Each time the engine wants a key it calls `generate_keypress()`
(`borg.c:generate_keypress:268-394`). In order:

1. `Term_locate()` — remember the cursor; `Term_fresh()`.
2. `borg_notice_player()` — read the player's stats and timed effects
   (Channel B), "before any check for deactivation that might use player
   stats".
3. `check_for_deactivate_or_death()` — if the Borg was stopped, the player
   died, the king retired, a stop depth/level was reached or the money-scum
   quota was met, tidy up and return a harmless key
   (`borg.c:check_for_deactivate_or_death:97-203`).
4. If the engine asked for a flush and keys are queued, `borg_flush()`.
5. `borg_get_messages()` — read the message line. Prompts that need an
   immediate answer (`-more-`, "Die?", "Direction?", …) **return a key
   straight away** (§4.6).
6. `borg_parse(NULL)` — flush the parsed messages into the reaction queue;
   then `borg.dont_react = false`.
7. **If a key is already queued, return it.** (`borg_inkey(true)`.)
8. `user_abort()` — a real key press from the user stops the Borg.
9. If `BI_CDEPTH >= 1`, clear `borg.in_shop`.
10. **If the player is resting or a command is repeating, return a key of
    type `EVT_NONE`** — let the game's own rest/repeat loop run (§2.7).
11. If respawning, finish the respawn (`borg_reincarnate_end()`).
12. `borg.targeting = false`.
13. Swap in the Borg's RNG, call `borg_think()` until it returns true, then
    `borg_display_status()`, save `borg_rand_local` and restore the game's
    RNG.
14. Count down the single-step counter `borg_step`, cancelling when it
    reaches zero.
15. Return the first queued key; if there is none, `borg_oops("normal
    abort")` and return `ESCAPE`.

Two consequences matter for a rebuild. **The Borg thinks only when its key
queue is empty** (step 7 comes before step 13), so a think that queues five
keys is followed by four calls of `generate_keypress()` that only drain the
queue. And **a think that queues nothing stops the Borg** (step 15).

### 2.3 The decision function

`borg_think()` (`borg-think.c:borg_think:179-256`), in order:

Outline of `borg-think.c:borg_think:179-256`; every line is a call or test,
in source order:

```pseudocode
borg_find_all_items()            Channel B: gear, quiver, pack, stores; snapshot safe_items[]
borg_check_save()                autosave if a save is pending (queues ^S); may return
borg_cheat_spells()              only if borg_do_spell is set
player_is_shapechanged()         if so, queue 'v' 'r' (the throw command, which changes back) and return
borg_notice(true)                rebuild borg.trait[] / borg.has[] from the items (§3.4)
borg_update()                    the map, monsters, objects, messages, timers (§4)
borg.power = borg_power()        score the current kit (§5)
store header "Stor"/"Home"?      if so: return borg_think_store()          (§12)
borg_do_spell = true
track BI_MAXCLEVEL / BI_MAXDEPTH
borg.time.now++ ; overflow check (§2.4)
borg.antibounce_count++          (§2.5)
borg.status.cancel?              return true
return borg_think_dungeon()      the decision ladder: §2.8
```

`generate_keypress()` calls it as `while (!borg_think());`.

### 2.4 The Borg's clock

`borg.time.now` is a `borg_time` (`int32_t`; `BORG_TIME_MAX` is
`INT32_MAX`). It counts calls of `borg_think()` outside the stores, not
game turns, and every `borg.time.*` timestamp and `borg_timer()` test is in
that unit.

`borg_think()` resets it to 1 with the warning
`"*****WARNING***** Borg Clock Overflow"` when it exceeds
`BORG_TIME_MAX - 100` (`borg-think.c:borg_think:242-245`). That reset is
**unreachable**: `borg_think_dungeon()` aborts with `borg_oops("clock
overflow")` once `borg.time.now >= BORG_TIME_MAX - 3000`
(`borg-think-dungeon.c:borg_think_dungeon:1282-1288`), which happens
first. The ladder also has two fixed windows, `borg.time.now` in
12000–12025 and 25000–25025, in which it queues four `ESCAPE`s and forces
the "new level" bookkeeping (`borg-think-dungeon.c:borg_think_dungeon:1263-1279`).

### 2.5 The anti-bounce counter

`borg.antibounce_count` (`int16_t`) measures how long the Borg has gone
without visible progress. It is not a turn counter:

* **incremented** by 1 on every `borg_think()` outside the stores, and
  also inside several loops (wear, junk, quiver, store buy/sell);
* **bumped by +10** whenever a note contains `"Best Combo"` or
  `"Taking off "` (`borg-io.c:borg_note_internal:165-170`). At the pin no
  note contains `"Best Combo"`; the ring-swap notes contain
  `"Taking off "`;
* **bumped by +100** on some messages (for example an `"Illegal "`
  command, `borg-messages.c`);
* **reset to 1** on progress: a new level, a kill, a panel change,
  tunnelling into granite, and a few other events (`borg-update.c`,
  `borg-messages.c`); `borg_recover()` resets it to 0 after some rests.

Thresholds read it to change behaviour — among them:

| Test | Where | Effect |
| --- | --- | --- |
| `<= 200` | `borg-danger.c:borg_danger:2846-2847` | the per-grid monster fear `borg_fear_monsters[][]` is added to danger only while the counter is at most 200 |
| `>= 300 && <= 303` | `borg-think-dungeon.c:borg_check_bounciness:1139-1142` | forget the current flow goal |
| `> 300` | `borg_backup_swap()` (`borg-item-wear.c`) | refuse an emergency equipment swap |
| `>= 500 && <= 503` | `borg-think-dungeon.c:borg_check_bounciness:1144-1158` | forget every tracked object and monster |
| `>= 700` | `borg-think-dungeon.c:borg_check_bounciness:1163-1173` | set `borg.goal.leaving` and `borg.goal.fleeing` |
| `> 1200` | `borg-danger.c` | change the danger estimate for long-stuck Borgs |

### 2.6 The key queue and the ESCAPE discipline

The queue is a ring of `KEY_SIZE` (8192) `keycode_t`s
(`borg-io.c:borg_keypress:311-336`, `borg-io.c:borg_inkey:394-418`,
`borg-io.c:borg_flush:423-429`):

* `borg_keypress(k)` appends one key; a zero key is refused with a note;
  overrunning the ring aborts the Borg (`borg_oops("overflow")`).
* `borg_keypresses(str)` appends each character of a string.
* `borg_inkey(false)` peeks at the next key; `borg_inkey(true)` takes it.
* `borg_flush()` empties the queue and forgets a queued direction.
* `borg_queue_direction()` stores a direction to be returned when the game
  asks "Direction?" (§4.6).

Every action is a **complete key sequence queued in one think**: command,
item letter, sometimes a direction or a count, and usually a closing
`ESCAPE`. For example the identify path queues the spell or scroll, then
the item selector, then `ESCAPE`
(`borg-item-wear.c:borg_test_stuff:212-238`); a recall queues the
use-item keys, the answer to the recall-depth prompt (`'y'` or `'n'`), and
`ESCAPE` (`borg-escape.c:borg_recall:58-124`). The trailing `ESCAPE` is
there so that an unexpected prompt or menu is dismissed rather than
answered by the next action's keys. Item letters come from
`all_letters_nohjkl[]`, never from the screen.

### 2.7 The rest/repeat pass-through

When the player is resting (`player->upkeep->resting`) or a command is
repeating (`cmd_get_nrepeats() > 0`), `generate_keypress()` returns a key
of type `EVT_NONE` and does not think (`borg.c:generate_keypress:334-337`).
That is how the Borg rests: it queues `R` and a count (`'R' '7' '5'
KC_ENTER`, or `'R' '9' KC_ENTER` while waiting for recall), and the game's
own rest loop then runs — and is interrupted by the game — without the
Borg being consulted. A rebuild that always returns a key interrupts its
own rest on the next turn and never recovers.

### 2.8 The dungeon decision ladder

`borg_think_dungeon()` (`borg-think-dungeon.c:borg_think_dungeon:1256-2302`)
is a fixed priority list. Each rung either queues keys and returns true, or
falls through. This ordering, more than any scoring weight, is what makes
the Borg play the way it does. The outline below lists every rung in source
order; the lines are within that function.

Pseudocode: the rungs of `borg-think-dungeon.c:borg_think_dungeon:1256-2302`
in source order.

```pseudocode
clock windows / clock abort                         (§2.4)
borg_destroy_floor()                                destroy an ignored item underfoot
delay; redraw if borg.status.redraw
level boredom: borg_timer(borg.time.level) >= 10000 -> goal.leaving, goal.fleeing
borg_near_monster_type()                            note scary/unique/breeder on level
depth 1 and goal.fleeing_to_town: borg_think_stair_scum()   (§13.1)
borg_check_bounciness()                             (§2.5)
borg_check_breeders()
avoidance follows BI_CURHP (borg_danger_wipe if it changed)
stair leash for weak characters (goal.less)
borg_notice(true)
self-lunal mode: borg_think_dungeon_lunal()         (§13.1)
munchkin start mode: borg_think_dungeon_munchkin()  (§13.1)
"get back on correct depth": borg_prepared() fails -> goal.less, take '<'
borg_think_dungeon_light()                          (§10.7)
borg.no_retreat countdown; borg.times_twitch > 20 -> borg.goal.fleeing = true
borg_flow_old(GOAL_DIGGING)                         continue an anti-summon corridor
borg_caution()                                      caution and escape (§9)
no light / cut / poisoned / starving: light, wear, borg_recover(), drop, shop
borg_must_return_to_town() -> borg_leave_level()
borg_play_magic(false)                              study / try spells (§10.6)
digger or bare hands: borg_wear_stuff()
borg_flow_kill_corridor()                           dig an anti-summon corridor
borg_attack(false)                                  routine combat (§7)
borg_wear_stuff(), borg_swap_rings(), borg_wear_rings()
borg_flow_old(GOAL_TAKE), borg_flow_take(true, 5)   pick up something visible and close
borg_wear_rings(), borg_remove_stuff(), borg_dump_quiver()
borg_check_light()                                  detections and lighting (§10.7)
borg_flow_old(GOAL_RECOVER), borg_recover(), borg_flow_recover(50)   (§10.9)
borg_perma_spell()                                  keep timed spells up (§7.5)
low-level casters: rest on a stair for mana, or flow to one
town and broke: leave via down stairs
goal.less: head for an up stair
goal.fleeing (not recalling): any stair
Morgoth on level and glyphs available: borg_flow_glyph()
GOAL_TAKE / borg_flow_take(false, 5) / borg_flow_vein(true, 5)
GOAL_KILL / borg_flow_kill(true, 20)                walk to a visible monster
GOAL_TAKE / borg_flow_take(false, 10) / borg_flow_vein(false, 10)
GOAL_KILL / GOAL_VAULT / borg_flow_kill_aim(true)  move to a firing position
borg_use_things(), borg_test_stuff(), borg_enchanting(), borg_recharging()
borg_drop_junk(), borg_drop_hole(false), borg_drop_slow()   (§10.8)
GOAL_TAKE / borg_flow_take(true, 250) / borg_flow_vein(true, 250)
goal.leaving, or low level with 13+ items to sell: take an up stair (or down if prepared)
prepared for BI_CDEPTH + 5: dive (GOAL_BORE, borg_flow_stair_more)
continue GOAL_MISC / GOAL_DARK / GOAL_XTRA / GOAL_BORE / GOAL_VAULT
vault on level: kill, take, borg_excavate_vault(), borg_flow_vault(), borg_flow_dark()
borg_flow_kill(false, 250), borg_flow_take(false, 250), borg_flow_vein(false, 250)
borg_flow_dark(true)                                explore (near)
money scum in town, else borg_leave_level(false)
borg_flow_dark(false)                               explore (far)
borg_choose_shop() -> borg_flow_shop_entry()
borg_play_magic(true)
detections, borg_flow_spastic(false), borg_flow_kill_direct(false), borg_wear_recharge()
money scum, else borg_leave_level(true)
detections, borg_flow_spastic(true), borg_flow_kill_direct(false)
recalling and safe: rest 'R' '9' KC_ENTER
                                                    --- nothing worked: "bizarre" level ---
borg.no_retreat = 5; boost bravery (1): avoidance = 2 * BI_CURHP, borg_think_dungeon_brave()
borg.times_twitch++; if < 3 and borg_allow_teleport(): phase -- PHASE_DOOR, act_tele_phase,
    sv_scroll_phase_door, borg_dimension_door(90), TELEPORT_SELF, PORTAL, borg_shadow_shift(90)
borg.no_retreat = 10
boost bravery (2): avoidance = 4 * BI_MAXHP, borg_think_dungeon_brave()
boost bravery (3): avoidance = 30000, forget stairs/glyphs/tracks/objects, borg_think_dungeon_brave()
borg.times_twitch++; if < 5 and borg_allow_teleport(): teleport -- borg_dimension_door(90), TELEPORT_SELF,
    PORTAL, borg_shadow_shift(90), sv_staff_teleportation, sv_scroll_teleport, sv_scroll_teleport_level,
    act_tele_level
in the dungeon: borg_recall()
forget objects and monsters; borg_flow_kill_direct(true); borg_twitchy()
return false
```

`borg.times_twitch` is reset on a level change (`borg-update.c`) and by
some escapes (`borg-escape.c`). It is incremented twice on every pass that
reaches the bottom of the ladder — once before the phase rung and once
before the teleport rung; a pass that falls through both adds two. With
the thresholds `< 3` and `< 5`, that allows one phase attempt per level (two
if the first phase succeeded and returned before the second increment) and
two teleport attempts; at more than 20 the early rung sets
`borg.goal.fleeing`.

`borg.avoidance` is the danger level the flow treats as impassable (§6.5).
The three "boosted bravery" passes raise it, re-run a reduced ladder
(`borg_think_dungeon_brave()`, which starts with `borg_defend()` at depth 100
and `borg_attack(true)`), and restore it — so a Borg that is stuck walks
through danger it would otherwise route around.

---

## 3. The Borg's memory

### 3.1 What it keeps

| Memory | Holds | Maintained by |
| --- | --- | --- |
| `borg_grids[][]` | what the Borg believes about every dungeon grid it has seen | `borg_update()` — patched in place; a grid keeps what the Borg last saw there |
| `borg_items[]`, `safe_items[]` | the gear, pack, quiver; the snapshot used to undo simulations | rebuilt from Channel B by `borg_find_all_items()` every think |
| `borg.trait[]`, `borg.has[]`, `borg.activation[]` | the model of itself | zeroed and rebuilt by `borg_notice()` |
| `borg_kills[]` | tracked monsters | `borg_update()` — created, refreshed, moved and retired across turns |
| `borg_takes[]` | tracked objects on the floor | `borg_update()`, the same way |
| `track_less`, `track_more`, `track_glyph`, `track_door`, `track_closed`, `track_step`, `track_vein` | stairs, glyphs, doors, steps, veins seen on this level | `borg_update()` and the message handlers |
| `borg_race_death[]`, `borg_race_count[]` | kills per race (all game), sightings per race (this level) | kills, `borg_update()` |
| `borg_shops[]` | the last seen contents of every store and the home | `borg_notice_store()` |
| `borg.time.*` | when things last happened (level start, detections, …) | everywhere |

### 3.2 The grid record

Transcribed from `borg-cave.h:borg_grid:74-86`:

```c
struct borg_grid {
    uint8_t  feat; /* Grid type */
    uint16_t info; /* Grid flags */
    bool     trap;
    bool     glyph;
    bool     web;
    uint8_t  store;
    uint8_t  take; /* Object index */
    uint8_t  kill; /* Monster index */
    uint8_t  xtra; /* Extra field (search count) */
};
```

`take` and `kill` are indexes into `borg_takes[]` and `borg_kills[]` (0 =
none), `store` is the shop number of a shop entrance, and `xtra` counts
searching. `borg_grids` is `borg_grid *borg_grids[AUTO_MAX_Y]`, one row per
dungeon row; `AUTO_MAX_Y`/`AUTO_MAX_X` are `DUNGEON_HGT` (66) and
`DUNGEON_WID` (198).

`glyph` is meant to mark a glyph of warding, but at the pin it is set only
inside the branch that handles a visible *player trap*
(`borg-update.c:borg_update_map:330-353`, guarded by
`square_isdisarmabletrap()`), and the glyph of warding is not a player trap
in `lib/gamedata/trap.txt` (its flags are `GLYPH | VISIBLE | FLOOR`). The
Borg tracks its glyphs through `track_glyph` instead (set from messages).

The `info` flags:

Transcribed from `borg-cave-view.h:BORG_MARK:67-75`:

```c
#define BORG_MARK  0x01 /* observed grid */
#define BORG_GLOW  0x02 /* probably perma-lit */
#define BORG_DARK  0x04 /* probably not perma-lit */
#define BORG_OKAY  0x08 /* on the current panel */
#define BORG_LIGHT 0x10 /* lit by the torch */
#define BORG_VIEW  0x20 /* in line of sight */
#define BORG_TEMP  0x40 /* temporary flag */
#define BORG_XTRA  0x80 /* extra flag */
#define BORG_IGNORE_MAP 0x100 /* ignore unseen grid from map_info() */
```

### 3.3 Item memory

`borg_items[]` is indexed by a **packed** index space (`borg-item.h`):

| Range | Meaning |
| --- | --- |
| `0 .. z_info->pack_size - 1` | the pack (`PACK_SLOTS` is the pack size less the quiver slots in use) |
| `INVEN_WIELD ..` | the equipment; `INVEN_WIELD` is `z_info->pack_size`, followed by `INVEN_BOW`, `INVEN_RIGHT`, `INVEN_LEFT`, …, `INVEN_LIGHT`, … up to `INVEN_TOTAL` |
| `QUIVER_START .. QUIVER_END - 1` | the quiver; `QUIVER_START` is `INVEN_TOTAL` |

A `borg_item` carries `tval`, `sval`, `iqty`, `weight` (from
`object_weight_one()`, so one item), `timeout`, the known to-hit/to-dam/AC,
dice, modifiers and flags, `ident` (`object_fully_known()`), `needs_ident`
(not `object_runes_known()`), `aware`, the description and inscription, and
a value. `safe_items[]` is the copy taken at the end of
`borg_find_all_items()`; the simulations in §5.4 restore from it.

### 3.4 The trait vector

`borg.trait[]` is a flat `int` array indexed by the `BI_*` enumerators of
`borg-trait.h`: its model of itself. A few of them: `BI_CURHP`, `BI_MAXHP`,
`BI_CURSP`, `BI_MAXSP`, `BI_CDEPTH`, `BI_MAXDEPTH`, `BI_CLEVEL`,
`BI_MAXCLEVEL`, `BI_GOLD`, `BI_SPEED`, `BI_LIGHT`, `BI_FOOD`, the conditions
`BI_ISCUT`, `BI_ISPOISONED`, `BI_ISWEAK`, `BI_ISHUNGRY`, `BI_ISPARALYZED`,
`BI_ISCONFUSED`, `BI_ISBLIND`, `BI_ISIMAGE`, `BI_ISAFRAID`, resistances
(`BI_RFIRE`, …, and `BI_SRFIRE` etc. which include swap items and spells),
and consumable counts such as `BI_ACSW` and `BI_AEZHEAL`.

`borg.has[]` is a different array: counts of items **by object kind**,
indexed by `kv_*` kind indexes (`borg.has[kv_potion_healing]`). The two
arrays are both `int *` and easy to confuse (§11.2).

`borg_notice(true)` **zeroes** `borg.has`, `borg.trait` and
`borg.activation` and rebuilds them: player stats, spells, equipment,
inventory, then the swap items (`borg-trait.c:borg_notice:2820-2949`). The
trait vector is therefore recomputed, not patched. `borg_think()` calls it
once and `borg_think_dungeon()` calls it again before the mode checks.

### 3.5 The monster and object lists

Transcribed from `borg-flow-kill.h:borg_kill:38-68`, with fields omitted where marked:

```c
struct borg_kill {
    uint16_t     r_idx; /* Race index */
    bool         known; /* Verified race */
    bool         awake; /* Probably awake */
    ...
    struct loc   pos; /* Location */
    ...
    uint8_t      speed; /* Estimated speed */
    uint8_t      moves; /* Estimates moves */
    ...
    int16_t      power; /* Estimated hit-points */
    int16_t      injury; /* Percent wounded */
    ...
    borg_time    when; /* When last seen */
    int16_t      m_idx; /* Game's index */
};
```

`borg_kills[]` is a global array (not a field of `borg`), with
`borg_kills_cnt` live entries and `borg_kills_nxt` the next unused slot;
slot 0 is unused, and a slot whose `r_idx` is 0 is dead. The list is
**patched and retired across turns**: `borg_locate_kill()` matches a
message or a screen sighting to an existing slot or makes a new one,
`borg_update_kill_old()` refreshes a slot in place, observed motion moves
it, and `borg_delete_kill()` retires it. It is emptied
(`borg_kills_cnt = 0; borg_kills_nxt = 1;`) on a level change
(`borg-update.c:borg_update:2232-2233`), by the anti-bounce check at 500
(§2.5), and by the last-resort rungs of the ladder (§2.8).

`borg_takes[]` (`borg_takes_cnt`, `borg_takes_nxt`) tracks floor objects
the same way.

---

## 4. Perception: `borg_update()`

`borg_update()` (`borg-update.c:borg_update:1539-3091`) brings the Borg's
memory up to date. Its main jobs:

### 4.1 What it reads

* The panel offset from the terminal (`w_y = Term->offset_y; w_x =
  Term->offset_x;`), and the number of panels from `cave->width`/
  `cave->height`.
* Every grid of the visible panel through `map_info()` (Channel B, §1.1),
  in `borg_update_map()`. A grid is marked `BORG_OKAY` (on the panel) and,
  if the game knows its feature, `BORG_MARK` with `feat` copied; shops,
  visible traps and webs are noted; visible monsters and objects become
  "wanks" (screen sightings) that are matched to the kill and take lists.
* The reaction queue filled by the message layer (§4.6). Each queued
  message is used at most once (`borg_msg_use[]`); kills, deaths, "you
  feel" messages, monster spells and pain messages update the lists and
  flags.
* `borg_notice_player()`'s view of the player's own state, already taken
  (§2.2).

### 4.2 What it writes

* `borg_grids[][]` — features, flags, monster/object indexes, traps, webs.
* `borg_kills[]`, `borg_takes[]`, the `track_*` lists.
* `borg_race_death[]`, `borg_race_count[]`.
* `borg.time.*` — level start, detections, morgoth sightings.
* `borg_fear_region[][]` (fear per 11×11 region) and
  `borg_fear_monsters[][]` (fear per grid), which `borg_danger()` adds in
  (§8).
* On a **level change** (`borg.status.old_depth != borg.trait[BI_CDEPTH]`):
  restart the level clock, reset the anti-bounce counter, forget the map
  and every list, and reset the level goals
  (`borg-update.c:borg_update:2055-2240`).

### 4.3 The panel

The map panel is a *window* on the dungeon: `SCREEN_HGT` rows by
`SCREEN_WID` columns (`src/ui-term.h`), starting at dungeon grid
(`w_y`, `w_x`). The Borg's map and position are in **dungeon** coordinates;
`borg.c` is the Borg's dungeon grid, found as the `is_player` grid of
`map_info()`. Grids off the panel keep whatever the Borg last saw there.

The Borg moves the panel itself when it wants to see off-screen monsters:
`borg_defend_panel_shift()` queues `L`, a direction and `ESCAPE`, driven by
the flag `borg.need_shift_panel`
(`borg-fight-defend.c:borg_defend_panel_shift:2819-2952`); it is the first
thing `borg_defend()` tries.

### 4.4 Monster bookkeeping

Monsters are *tracked*: a sighting or a message is matched to a slot by
`borg_locate_kill()` (which uses the remembered race and position), the
slot is moved when the monster is seen to move, and its condition is
refreshed from the real monster by `borg_update_kill_old()` (§1.1). The
race of a new slot comes from the real monster on the grid for a screen
sighting (`borg_guess_race()`, §1.1), or from the name in the message
(`borg_guess_race_name()`) for a message; the screen symbol and attribute
are what `observe_kill_diff()` and `observe_kill_move()` use to match
sightings to existing slots. Monster names beginning with "It" are treated as
invisible-monster messages (`my_strnicmp(who, "It", 2) == 0` in
`borg-flow-kill.c:borg_locate_kill:1286-1292`).

### 4.5 Detection timers

`borg.time.detect_traps`, `detect_doors`, `detect_walls`, `detect_evil` and
`detect_obj` record when each detection last succeeded, and per-grid
arrays (`borg_detect_trap`, …) record where. `borg_check_light()` re-casts
detections on a schedule rather than every turn (§10.7).

### 4.6 The message layer: prompts, parsing and reactions

Messages reach the Borg in three stages.

1. **Prompts that need an answer now.** `borg_get_messages()` reads the
   message line. If it looks like a prompt (`borg_prompt`), and the game is
   not waiting for a command, `borg_react_prompted()` gets the first look
   (`borg-messages-react.c:borg_react_prompted:95-324`). It answers, among
   others:

   | Prompt (prefix) | Answer |
   | --- | --- |
   | `"Die?"` (with `borg.status.cheat_death`) | log the death; with `borg_cheat_death` set, cure-all and `'n'`; otherwise start a respawn and `'y'` |
   | `"Atte"` (cast anyway?) | `'y'` |
   | `"(Equip: c-d,"` | `'c'` (left hand) |
   | `"Direction"` | the direction queued with `borg_queue_direction()`, or abort with a key history dump if none was expected |
   | `"You have no room"` | a key of code 0 (a no-op) |
   | `"Set recall depth"`, when nothing is queued | `'n'` |
   | `"Target out of range"` | `'y'` |
   | `"Word of Recall is already"` | `'n'`, and abort unless trying an unknown item |
   | equipment/inventory/browse sub-screens | `ESCAPE` |
   | `"Throw which item? (Throw"` | `'/'` (switch to the inventory) |
   | `"Which effect?"` | `'a'` |

   A `-more-` is answered with a space after the text before it has been
   parsed; a normal message while the game wants a command is parsed and
   answered with a space (`borg-messages.c:borg_get_messages:183-294`).

2. **Parsing.** `borg_parse()` splits the text into messages and hands
   each to the static `borg_parse_aux()`, which handles a long list of
   message prefixes. Some act immediately: an `"Illegal "` message queues
   two `ESCAPE`s and adds 100 to the anti-bounce counter; amnesia sets
   `BI_ISFORGET`; "You have been knocked out." and "You are paralyzed"
   set the global `borg_dont_react`. Others are rewritten into a tagged
   form (`"KILL:"`, `"DIED:"`, `"PAIN:"`, …) and passed to `borg_react()`.

3. **Reactions.** `borg_react()` appends the tagged message to the
   reaction queue (`borg_msg_buf`, `borg_msg_pos[]`, `borg_msg_num`),
   unless `borg_dont_react` is set or the Borg is paralysed, in which case
   it notes "Ignoring messages" and drops it
   (`borg-messages-react.c:borg_react:41-90`). `borg_update()` consumes the
   queue (§4.1); `borg_clear_reactions()` empties it.

So messages are not only information: some answer prompts, some change
state immediately, and the rest steer `borg_update()`.

The struct field `borg.dont_react` (reset to false on every
`generate_keypress()`) and the global `borg_dont_react` (the one
`borg_react()` tests, set by the knock-out and paralysis messages) are
different variables (§11.2).

---

## 5. Scoring itself: `borg_power()`

`borg_power()` (`borg-power.c:borg_power:1881-1912`) computes one
`int32_t` that says how good the Borg's current kit is. It is the utility
behind every "try this on" choice: wearing, swapping, buying, selling,
dropping and crushing all compare `borg_power()` before and after a
simulated change.

### 5.1 Shape

Transcribed from `borg-power.c:borg_power:1881-1912` (comments removed):

```c
int32_t borg_power(void)
{
    int     i = 1;
    int32_t value = 0L;

    if (borg_cfg[BORG_USES_DYNAMIC_CALCS]) {
        value += borg_power_dynamic();
    } else {
        value += borg_power_equipment();
        value += borg_power_inventory();
    }

    for (i = 1; i <= borg.trait[BI_MAXDEPTH] + 50; i++) {
        if ((char *)NULL != borg_prepared(i))
            break;
    }
    value += ((i - 1) * 40000L);

    value += weapon_swap_value;
    value += armour_swap_value;

    return (value);
}
```

* `borg_power_equipment()` (`borg-power.c:borg_power_equipment:43-1117`)
  scores the derived traits: weapon damage times blows and slays, to-hit,
  shooting, speed, hit points, mana, armour class, resistances, sustains,
  abilities, light, and penalties.
* `borg_power_inventory()` scores what is carried: food, light fuel,
  potions, scrolls, staffs, wands, rods, ammunition and books, mostly with
  diminishing returns per extra item.
* **Every depth the Borg is prepared for adds 40,000** (§5.5), so a change
  that makes it ready for one more level is worth a lot.
* The two swap items add their own values.

### 5.2 The worship options

Five of the `borg_worships_*` options select **different weight branches**
inside `borg_power_equipment()`; they do not multiply a component. For
example, with `borg_worships_damage` the to-hit bonus earns an extra 15 per
point and the slay damage is counted once more
(`borg-power.c:borg_power_equipment:87-110`); with `borg_worships_ac` the
armour-class reward uses the 2500/2000/1500-per-point schedule instead of
2000/500/…​ (`borg-power.c:borg_power_equipment:586-600`). The same holds
for `borg_worships_speed`, `borg_worships_hp` and `borg_worships_mana`.

`borg_worships_gold` is not read by `borg-power.c` at all. It changes
which items `borg-junk.c` keeps for sale and the depth thresholds in
`borg-think-dungeon-util.c`.

### 5.3 The formula section

With `borg_uses_dynamic_calcs` set, `borg.txt` may contain formula sections
(`borg-formulas.c`, `borg-formulas-calc.c`). Then `borg_power_dynamic()`
**replaces** the equipment and inventory scoring, and
`borg_prepared_dynamic()`/`borg_restock_dynamic()` replace the depth checks
in §5.5. If the option is set but no formulas (or old-style formulas) are
found, `borg_init_txt_file()` turns it off with a warning. Parse errors are
reported through `borg_formula_error()`.

### 5.4 Simulation: scoring a kit the Borg is not wearing

To value a candidate, the Borg **writes the candidate into
`borg_items[]`**, re-runs `borg_notice()`, calls `borg_power()`, and then
copies the original slot back from `safe_items[]`. The equipment search in
`borg_best_stuff_aux()` does exactly that
(`memcpy(&borg_items[slot], item, …)` … `memcpy(&borg_items[slot],
&safe_items[slot], …)` in `borg-item-wear.c`). The store and home code
does the same with shop wares. A simulation that returns without the
restore leaves the next pass reading a kit the Borg does not have (§11.1).

This is a different mechanism from the `borg_simulate` flag of §7.4,
which marks an attack- or defence-*scoring* pass.

### 5.5 Depth preparation: `borg_prepared()`

`borg_prepared(depth)` (`borg-prepared.c:borg_prepared:538-686`) answers
"may I be on this depth?" It returns `NULL` for yes, or a short reason
string for no ("Clevel < depth", "1 recall", "resist fire", "Must kill
…"). In order:

1. Depth 1 is always allowed.
2. With dynamic calcs: `borg_restock()`, then `borg_prepared_dynamic()`.
   Otherwise: `borg_restock()` (am I carrying enough food, light, cures,
   escapes to *stay*?), then "Clevel < depth" unless the maximum level is
   50, then `borg_prepared_aux(depth)` for depths up to 99, then
   `borg_no_deeper`.
3. A king may go anywhere; from town the answer stands.
4. From depth 82 down it requires a stock of *Healing*/Life potions,
   counting those in the home (`borg_notice_home()`).
5. Two unique gates, by `BI_MAXDEPTH` (`borg-prepared.c:borg_prepared:626-685`):
   * **`BI_MAXDEPTH <= 98`:** only with `borg_kills_uniques` set, and only
     while three or more uniques are alive, it refuses depths deeper than
     `borg_depth_hunted_unique` ("Must kill …").
   * **`BI_MAXDEPTH >= 99` and requested depth `>= 98`:** regardless of
     `borg_kills_uniques`, it sets `borg.ready_morgoth = -1` and refuses
     ("… still alive!") unless no unique is alive or the first living one
     is Morgoth — in which case it allows the depth and, for depth 99 or
     more, sets `borg.ready_morgoth = 1`. One exception: with
     `BI_MAXDEPTH == 99`, depth 98 is allowed if the Borg can cast Teleport
     Level (fail ≤ 20) or carries a Teleport Level scroll.
   * Otherwise (`BI_MAXDEPTH >= 99`, depth below 98) the answer is yes.

`borg_prepared_aux()` is cumulative by depth band: each band adds
requirements (light, food, fuel, hit points and level by class, cures,
recall, teleports, see invisible, free action, the basic resists, stat
minimums, …) and returns at the first one unmet.

The answer gates almost every vertical move: the dive rung of the ladder
(`borg_prepared(BI_CDEPTH + 5)`), the "get back on the correct depth" rung,
the leave-level choice of stairs, lunal-mode entry (`BI_CDEPTH + 15`), the
40,000-per-depth power bonus, and the town shopping list. Changing a
requirement here changes how deep and how fast the Borg dives.

---

## 6. Flow: how the Borg decides *where* to go

The flow is the Borg's pathfinder. It is a **breadth-first flood from the
goal grids outward**, over a one-byte cost per grid, followed by a greedy
**downhill step from the Borg's own grid**. It is not A*.

### 6.1 The data structures

Transcribed from `borg-flow.h:borg_data:58-60` and
`borg-flow.h:borg_data_flow:84-88`:

```c
struct borg_data {
    uint8_t data[AUTO_MAX_Y][AUTO_MAX_X];
};

extern borg_data *borg_data_flow; /* Current "flow" data */
extern borg_data *borg_data_cost; /* Current "cost" data */
extern borg_data *borg_data_hard; /* Constant "hard" data */
extern borg_data *borg_data_know; /* Current "know" flags */
extern borg_data *borg_data_icky; /* Current "icky" flags */
```

Each is one byte per grid. `borg_data_cost` is the flood being built,
`borg_data_flow` the committed copy the Borg walks, `borg_data_hard` the
"unreached" template (all 255), and `borg_data_know`/`borg_data_icky` cache
"danger already evaluated" / "too dangerous" per grid. The queue is the
circular `borg_flow_y[]`/`borg_flow_x[]` of `AUTO_FLOW_MAX` (1536) entries
with `flow_head`/`flow_tail`.

### 6.2 The contract

Transcribed from `borg-flow.h:borg_flow_clear:131-152` (comments removed):

```c
extern void borg_flow_clear(void);
extern void borg_flow_spread(int depth, bool optimize, bool avoid,
    bool tunneling, int stair_idx, bool sneak);
extern void borg_flow_enqueue_grid(int y, int x);
extern bool borg_flow_commit(const char *who, int why);
extern bool borg_flow_old(int why);
```

The standard goal builder uses five steps. Two exploration builders,
`borg_flow_dark_1()` and `borg_flow_dark_2()`, replace steps 2–3 with the
static `borg_flow_direct()`, which writes a straight-line path from the
Borg to one target into the cost array (checking walls and danger along
it), then commit and step as usual (`borg-flow-dark.c`).

1. **`borg_flow_clear()`** — copy `borg_data_hard` into `borg_data_cost`
   (every grid unreached), reset the queue, and, if `borg_danger_wipe` is
   set, also wipe the know/icky caches
   (`borg-flow.c:borg_flow_clear:166-186`). **The caches otherwise survive
   from flood to flood and from turn to turn**; `borg_danger_wipe` is set
   when `borg.avoidance` changes (including each bravery pass, §2.8), on a
   level change, and by some kill and exploration flows.
2. **`borg_flow_enqueue_grid(y, x)`** for each goal grid. It evaluates the
   grid's danger once (caching in know/icky), refuses a grid whose danger
   exceeds the current fear threshold (unless desperate, lunal, munchkin or
   digging), and otherwise **sets the grid's cost to 0** and enqueues it
   (`borg-flow.c:borg_flow_enqueue_grid:506-580`).
3. **`borg_flow_spread(depth, optimize, avoid, tunneling, stair_idx,
   sneak)`** — pop grids and give each reachable neighbour the cost
   `parent + 1`, so the cost of a grid is its step distance **from the
   nearest goal** (`borg-flow.c:borg_flow_spread:255-501`). A neighbour is
   skipped if it is out of bounds, already cheaper, a wall (unless
   `tunneling`), permanent rock, lava without fire immunity, unknown (when
   `avoid` is set or the Borg is desperate, unless twitchy), a monster grid
   in some modes, another shop's entrance, a trap the Borg is too weak to
   risk, icky, or too dangerous on first evaluation. `depth` caps the
   distance; `optimize` stops as soon as the Borg's own grid is reached;
   `stair_idx >= 0` measures from that up stair instead of the Borg (for
   characters below level 15); `sneak` refuses grids adjacent to a monster
   unless desperate or twitchy.
4. **`borg_flow_commit(who, why)`** — read the cost at the Borg's grid; if
   it is 250 or more, return false; otherwise note
   `"# Flowing toward <who> at cost <n>"`, copy cost→flow and set
   `borg.goal.type = why`. **It queues no key.**
5. **`borg_flow_old(why)`** — if `borg.goal.type == why`, step to the
   neighbour with the lowest flow value (never uphill, a −5 bias against
   loops; in town a random choice among equals, in the dungeon the first),
   by calling the static `borg_play_step()`; if no step is possible, clear
   `borg.goal.type`. **This is where the keys are queued**
   (`borg-flow.c:borg_flow_old:1122-1203`).

Transcribed from `borg-flow.c:borg_flow_commit:585-608` (comments removed):

```c
bool borg_flow_commit(const char *who, int why)
{
    int cost;

    cost = borg_data_cost->data[borg.c.y][borg.c.x];

    if (cost >= 250)
        return false;

    if (who)
        borg_note(format("# Flowing toward %s at cost %d", who, cost));

    memcpy(borg_data_flow, borg_data_cost, sizeof(borg_data));

    borg.goal.type = why;

    return true;
}
```

`borg_play_step()` (`borg-flow.c:borg_play_step:613-1097`) turns "go to
this neighbour" into keys: close doors on breeder levels, open or unlock
doors, tunnel, disarm or jump traps, alter, or walk.

A goal therefore persists across turns: each later think calls
`borg_flow_old(<goal>)` on its rung of the ladder (§2.8), takes one more
step down the committed flow, and only rebuilds a flood when that fails.

### 6.3 The goal types and who drives them

`borg.goal.type` takes one of the `GOAL_*` values
(`borg-trait.h:GOAL_KILL:32-41`). The table was derived by grepping every
`borg_flow_commit()` call site and its `why` argument.

| Goal | Value | Meaning | Committed by |
| --- | ---: | --- | --- |
| `GOAL_KILL` | 1 | reach a monster or a firing position | `borg_flow_kill()`, `borg_flow_kill_aim()`, the monster-path flows in `borg-flow-kill.c` |
| `GOAL_TAKE` | 2 | reach an object or a treasure vein | `borg_flow_take()`, `borg_flow_take_scum()`, `borg_flow_take_lunal()` (`borg-flow-take.c`), `borg_flow_vein()` (`borg-flow-misc.c`) |
| `GOAL_MISC` | 3 | reach a shop entrance or a glyph spot | `borg_flow_shop_entry()` (`borg-flow-misc.c`), `borg_flow_glyph()` (`borg-flow-glyph.c`) |
| `GOAL_DARK` | 4 | explore | `borg_flow_dark()` and its five static helpers (`borg-flow-dark.c`) |
| `GOAL_XTRA` | 5 | wander/search | `borg_flow_spastic()` (`borg-flow-misc.c`) |
| `GOAL_BORE` | 6 | leave by a stair without fleeing: dive down, or (bored, or as part of leaving) go up | `borg_flow_stair_more(GOAL_BORE, …)` from the ladder and from `borg_leave_level()`; `borg_flow_stair_less(GOAL_BORE, …)` from `borg_leave_level()` (`borg-flow-stairs.c`, `borg-think-dungeon-util.c`) |
| `GOAL_FLEE` | 7 | leave the level by any route | `borg_flow_stair_less/more/both(GOAL_FLEE, …)` (`borg-flow-stairs.c`) and `borg_flow_light(GOAL_FLEE)` |
| `GOAL_VAULT` | 8 | reach a diggable vault wall | `borg_flow_vault()` (`borg-flow-misc.c`) |
| `GOAL_RECOVER` | 9 | reach a safe grid to rest | `borg_flow_recover()` (`borg-flow-misc.c`) |
| `GOAL_DIGGING` | 10 | dig an anti-summon corridor | `borg_flow_kill_corridor()`, `borg_flow_kill_direct()` (`borg-flow-kill.c`) |

`borg_flow_light(why)` (`borg-flow-misc.c:borg_flow_light:824-874`) flows
to any grid marked `BORG_GLOW`; its only caller passes `GOAL_FLEE`
(`borg-think-dungeon-util.c`). Some builders call `borg_flow_old()`
straight after committing, so they step in the same think.

### 6.4 The invariants that matter

* **Clear before building.** A builder that enqueues without
  `borg_flow_clear()` floods on top of the last goal's costs.
* **One flood at a time.** `borg_data_cost` and `borg_data_flow` are
  single shared buffers; committing a new goal replaces the old one, and
  `borg.goal.type` names whose flow is in `borg_data_flow`.
* **Commit queues nothing; `borg_flow_old()` steps.** A builder returns
  true only after a successful step (or a successful commit when a later
  rung will step).
* **Costs are bytes.** Distances of 250 or more are "unreachable"
  (`borg_flow_commit()`), and `borg_flow_spread()` notes that the largest
  usable `depth` is 250.

### 6.5 Danger-aware flowing

Whether a grid may be entered is decided the first time the grid is
evaluated after the last cache wipe (§6.2 step 1) — not once per flood — by
comparing `borg_danger(y, x, 1, true, false)` with a **fear threshold**
derived from `borg.avoidance`. The threshold is a sequence of assignments,
so a later one overrides an earlier one
(`borg-flow.c:borg_flow_spread:442-474`, repeated in
`borg-flow.c:borg_flow_enqueue_grid:523-555`):

```pseudocode
if BI_MAXCLEVEL == 50:                     fear = avoidance * 5 / 10
if BI_MAXCLEVEL != 50:                     fear = avoidance * 3 / 10
if borg.mon.scary:                         fear = avoidance * 2
if borg.mon.unique and borg.status.vault and BI_MAXCLEVEL == 50:  fear = avoidance * 3
if borg.mon.scary and BI_CLEVEL <= 5:      fear = avoidance * 3
if borg.goal.ignoring:                     fear = avoidance * 5
if borg_timer(borg.time.level) > 5000:     fear = avoidance * 25
if BI_FOOD == 0:                           fear = avoidance * 100
if BI_CLEVEL == 0:                         fear = avoidance * 3 / 10
too dangerous if danger > fear   (not evaluated when desperate, lunal, munchkin or digging)
```
 Raising `borg.avoidance`
(the bravery boosts of §2.8) therefore opens routes the Borg would
otherwise refuse.

---

## 7. Fighting

`borg-fight-attack.c` (5322 lines) and `borg-fight-defend.c` (3605 lines)
are the two largest files.

### 7.1 Who calls the fight code

From call-site grep at the pin (all callers listed; pseudocode notation):

```pseudocode
borg_attack(bool boosted_bravery)          borg-fight-attack.c:borg_attack:5149-5320
  <- borg_think_dungeon()          borg_attack(false)  routine combat rung, §2.8
  <- borg_think_dungeon_brave()    borg_attack(true)   bravery-boost passes
  <- borg_think_dungeon_munchkin() borg_attack(false)

borg_defend(int p1)                        borg-fight-defend.c:borg_defend:3536-3603
  <- borg_caution()                two calls, before or after borg_heal()
  <- borg_think_dungeon_brave()    only at depth 100

borg_flow_kill(bool viewable, int nearness)   walk towards a monster (GOAL_KILL)
  <- borg_think_dungeon(), borg_think_dungeon_brave()
borg_flow_kill_aim(bool viewable)             move to a grid with a shot (GOAL_KILL)
  <- borg_think_dungeon()
```

So the Borg attacks from the ladder's `borg_attack(false)` rung whenever an
attack scores above zero; the flow functions only move it into range.
Defensive manoeuvres run inside `borg_caution()` (§9), before the attack
rung.

### 7.2 Attack selection

`borg_attack()` collects the monsters that are tracked, seen within the
last two Borg turns, on the panel (`BORG_OKAY`), in line of sight
(`BORG_VIEW`) and within `z_info->max_range` into `borg_temp_y[]`/
`borg_temp_x[]`, with special cases for scary monsters, multipliers and
low-level mages in town. It then sets `borg_simulate = true` and scores
every attack type `g` with `borg_calculate_attack_effectiveness(g)`. If the
best score is positive it notes `"# Performing attack type %d with value
%d."`, clears `borg_simulate`, and calls the same function again to queue
the keys (`borg-fight-attack.c:borg_attack:5277-5318`).

The attack types are the `BF_*` enum in `borg-fight-attack.h`, in this
order: `BF_REST`, `BF_THRUST` (melee), `BF_OBJECT` (throw), `BF_LAUNCH`
(missile weapon), the spells (`BF_SPELL_MAGIC_MISSILE`, …,
`BF_SPELL_WHIRLWIND_ATTACK`, …), the prayers (`BF_PRAYER_HOLY_ORB_BALL`,
…), rods (`BF_ROD_*`), staffs (`BF_STAFF_*`), wands (`BF_WAND_*`), rings
(`BF_RING_*`), dragon armour (`BF_DRAGON_*`) and activations
(`BF_ACT_*`), up to `BF_MAX`. Activations from `BF_ACT_STARLIGHT` on are
scored only with `birth_randarts`.

Each kind has a scorer — `borg_attack_aux_launch()`,
`borg_attack_aux_spell_bolt()`, `borg_attack_aux_dragon()`, … — that
estimates the damage against the targets, chooses the best target, and,
when `borg_simulate` is false, queues the keys.

### 7.3 Defence

`borg_defend(p1)` first tries `borg_defend_panel_shift()` (§4.3), then
sets `borg_simulate = true`, possibly refreshes Resistance, scores every
defence type `BD_*` (speed, resistances, glyphs, teleport-other, banish,
…; the enum is in `borg-fight-defend.c`) with
`borg_calculate_defense_effectiveness(g, p1)`, and executes the best if
its score is positive.

### 7.4 The simulation flag

`borg_simulate` (`borg-trait.h`) means "score, do not act". Every attack
and defence scorer checks it before queuing keys. `borg_attacking` means
"an attack is being considered"; `borg_danger()` boosts some monsters'
danger while it is set. Both are globals that the caller is expected to
reset; §11.1 shows where they are not.

### 7.5 Buffs and the munchkin attackers

`borg_perma_spell()` (`borg-fight-perm.c`) keeps bless, resistance,
speed, protection from evil, heroism, berserk, regeneration and similar
effects running, each scored by a `borg_perma_aux_*()` helper. Munchkin
mode (§13.1) has its own attackers, `borg_munchkin_mage()` and
`borg_munchkin_melee()` (`borg-attack-munchkin.c`), called from
`borg_think_dungeon_munchkin()`.

---

## 8. Danger estimation

`borg_danger(y, x, c, average, full_damage)`
(`borg-danger.c:borg_danger:2824-2865`) answers "how dangerous is grid
(y, x) over the next `c` turns?", on a scale capped at 2000:

Transcribed from `borg-danger.c:borg_danger:2824-2865` (comments removed):

```c
int borg_danger(int y, int x, int c, bool average, bool full_damage)
{
    int i, p = 0;

    struct loc l = loc(x, y);
    if (!square_in_bounds(cave, l))
        return 2000;

    if (!square_isvault(cave, l) && borg.trait[BI_CDEPTH] <= 80) {
        p += borg_fear_region[y / 11][x / 11] * c;
    }

    if (borg.trait[BI_CDEPTH] == 100 && p >= 300)
        p = 300;

    if (borg.antibounce_count <= 200 && !square_isvault(cave, loc(x, y)))
        p += borg_fear_monsters[y][x] * c;

    full_damage = true;

    for (i = 1; i < borg_kills_nxt; i++) {
        borg_kill *kill = &borg_kills[i];

        if (!kill->r_idx)
            continue;

        p += borg_danger_one_kill(y, x, c, i, average, full_damage);
    }

    return (p > 2000 ? 2000 : p);
}
```

Note the `full_damage = true;` line: whatever the caller passes, every
monster is scored at full damage from this entry point.

`borg_danger_one_kill()` (`borg-danger.c:borg_danger_one_kill:2289-2801`)
combines a physical term from the static `borg_danger_physical()` (the
race's blows, adjusted for resistances and the `MONBLOW_*` effects) and a
spell term from the static `borg_danger_spell()`, scaled by how many moves
the monster gets (speed and distance), whether it is awake, and whether it
can reach the grid. Monsters listed in the Teleport Other skip list
(`borg_tp_other_index[]`) score 0.

---

## 9. Caution and escape

### 9.1 `borg_caution()` — the reactive layer

`borg_caution()` (`borg-caution.c:borg_caution:799-2034`) runs on the
ladder after the light and digging rungs and before every routine goal
(§2.8). It is a long chain of "if the situation is X, do Y" checks; most
steps either queue keys and return true, or set a goal flag and fall
through. In source order:

1. **Nasty conditions** are noted for later steps: light about to run out,
   weak from hunger, blind, confused, hallucinating.
2. **Depth 100, not ready for Morgoth:** rise by Teleport Level, or set
   leaving and head for an up stair.
3. **Local danger.** `borg_surrounded()` is evaluated (used by step 12).
   More than three escapes on this level (unless a unique is near or the
   Borg is ready for Morgoth), or a scary monster on the level: set
   `borg.goal.leaving` / `borg.goal.fleeing` (the scary case can also set
   `fleeing_to_town`).
4. **Notes only:** ignoring monsters, ignoring messages, the danger summary,
   active protections, standing on a glyph or stair.
5. **Defend and heal**, only if not already fleeing
   (`borg-caution.c:borg_caution:1001-1022`). A mage that is not in a Morgoth
   position or an anti-summon corridor and is not blind, cut, poisoned or
   confused tries `borg_defend(pos_danger)` first and then the static
   `borg_heal(pos_danger)`; every other case heals first, then defends.
6. **Swap.** With `borg_uses_swaps()` and danger above a third of
   `borg.avoidance`, `borg_backup_swap(pos_danger)`
   (`borg-caution.c:borg_caution:1025-1031`).
7. **Waiting for recall:** rest if safe; in town, prepare the spells for the
   next level.
8. **Restock and danger flags:** not stocked to stay (`borg_restock()`,
   unless near a unique or just out of town), excessive danger, or potential
   danger in town: set leaving and/or fleeing.
9. **Take a stair now** if leaving, fleeing, a scary monster is on the
   level, or in lunal/munchkin flight — or if danger is high and the Borg is
   standing on an up stair; also take the stair named by `borg.stair_less`
   / `borg.stair_more`.
10. **Critical needs:** no light (leave), weak from hunger (eat, cure or
    leave), breeders at a low character level (flee).
11. **Flee on foot.** Dash for a nearby stair with `borg.status.desperate`
    set for that one flood (`borg_flow_stair_less()` /
    `borg_flow_stair_more()` with `GOAL_FLEE`), in this order: an up stair
    under three steps away when `pos_danger < BI_CURHP`; an up stair at most
    two steps away regardless; an up stair under twenty steps away when a
    scary monster is on the level, the character level is below 20, no
    adjacent monster is faster, and it is not a shallow level (≤ 5) with a
    unique near; a down stair at most two steps away
    (`borg-caution.c:borg_caution:1338-1418`).
12. **Strategic retreat.** Step to a less dangerous neighbour when danger
    exceeds a third of `borg.avoidance` (unless nasty or `borg.no_retreat`)
    or the Borg may be surrounded — unless confused, in a vault, at 500 or
    more hit points, or recently in an anti-summon spot.
13. **Escape.** `borg_escape(pos_danger)` — teleport or phase; counts
    `borg.escapes` and clears the goal
    (`borg-caution.c:borg_caution:1653-1663`).
14. **Back away** from a single adjacent monster.
15. **Cures** for confusion, fear, low hit points, bleeding and poison, the
    reserve *Healing*, and `borg_recall()` when fleeing; buying time while
    recall is active; finally trying unknown items.

### 9.2 `borg-escape.c` — the escape machinery

| Function | Purpose |
| --- | --- |
| `borg_escape(int b_q)` | choose and use an escape — teleport level, teleport, phase, dimension door, staffs, scrolls, spells, activations — by danger band and allowed failure rate (`borg-escape.c:borg_escape:645-1286`) |
| `borg_caution_phase(int emergency, int turns)` | is Phase Door a good idea? Tries 100 random landing grids with `rand_spread()` around the Borg and counts how many are safer (`borg-escape.c:borg_caution_phase:324-412`) |
| `borg_caution_teleport(int emergency, int turns)` | the same question for Teleport |
| `borg_escape_stair()` (static) | leave by the up stair the Borg is standing on |
| `borg_allow_teleport()` | not in an arena, not on a no-teleport grid, no no-teleport curse |
| `borg_recall()` | read/zap/activate/cast recall and answer the recall-depth prompt (§9.3) |
| `borg_surrounded()` | might the Borg be surrounded? (§11.9) |
| `borg_freedom(int y, int x)` | how "free" a grid is (near stairs) |
| `borg_shadow_shift()`, `borg_dimension_door()` | short-range escapes |

### 9.3 Answering the game's prompts

The Borg answers a prompt in one of two ways, and a rebuild needs both:

* **Pre-answered**: the action that will cause the prompt queues the
  answer with it. `borg_recall()` queues the recall command, then `'y'` or
  `'n'` for "Set recall depth to current depth?", then `ESCAPE`.
* **Caught**: `borg_react_prompted()` recognises the prompt on the message
  line and returns the answer (§4.6). It also answers "Set recall depth"
  with `'n'` when nothing is queued (an unknown recall item).

The recall-depth answer shows the shape of a prompt decision
(`borg-escape.c:borg_recall:64-117`). The prompt appears only when
`BI_CDEPTH` is below `BI_MAXDEPTH` and not 0. The answer depends on how far
the current and maximum depths are from `borg_depth_hunted_unique` (the
depth of the unique being hunted), and then on Sauron, munchkin flight,
depth 100 and the number of living uniques — state split across
`borg.trait[]`, `borg.goal.*`, `borg_race_death[]` and
`borg_numb_live_unique`. In one of its branches the comment says "Do reset
Depth" while the note and the key both say "not resetting" (`'n'`); §17
uses that branch as its worked example.

---

## 10. Items

### 10.1 The pipeline

Pseudocode: the item work done per think, in `borg_think()` and then on the
ladder.

```pseudocode
borg_find_all_items()      Channel B -> borg_items[] (borg_item_analyze() per object), safe_items[]
borg_notice(true)          borg_items[] -> borg.trait[], borg.has[], borg.activation[]
borg_power()               the kit as one number (§5)
ladder rungs:
  borg_wear_stuff(), borg_remove_stuff(), borg_swap_rings(), borg_wear_rings()   wear/remove by power
  borg_use_things(), borg_test_stuff(), borg_enchanting(), borg_recharging()     use, identify, improve
  borg_drop_junk(), borg_drop_hole(), borg_drop_slow(), borg_dump_quiver()       get rid of things
in the home:
  borg_best_stuff()          the best combination of pack + home + equipment
```

### 10.2 Name lookups: `borg_init_item_val()`

`borg-item-val.c` contains three functions: `borg_init_item_val()`,
`borg_lookup_kind()` and the static `borg_lookup_sval_fail()`. At start-up
`borg_init_item_val()` resolves every `sv_*` and `kv_*` the Borg uses
**by name** from the game data (`sv_wand_elec_bolt =
borg_lookup_sval_fail(tval, "Lightning Bolts")`, …) — the Borg's code never
uses raw sval numbers. A wrong name that happens to be a real item is
accepted silently (§11.5). There is no item-scoring function in this file;
item value comes from `borg_item_analyze()`'s value fields and, for every
keep/wear/buy/sell decision, from `borg_power()` (§5).

### 10.3 Identification

The Borg identifies by rune knowledge. `needs_ident` comes from the game
(not all runes known). `borg_test_stuff()` (§11.3) picks an item and uses
the Identify Rune spell or a scroll of identify; `borg_use_things()`,
`borg_quaff_unknown()`, `borg_read_unknown()`, `borg_use_unknown()` and
`borg_eat_unknown()` learn flavours by using them, and messages update the
item state (`borg-messages.c`).

### 10.4 Wearing, swapping, and the home combination

* `borg_wear_stuff()` and `borg_remove_stuff()` take one step towards the
  kit with the higher `borg_power()`.
* `borg_best_stuff()` runs only in the home (`shop_num == BORG_HOME`): it
  searches combinations of equipment, pack and home items with
  `borg_best_stuff_aux()` (a simulation per candidate, §5.4) and then
  wears the result one item per think through
  `borg_one_step_wearing_best()`.
* `borg_backup_swap()` swaps in a *backup* weapon or armour when the
  current one is inadequate against the present danger; the backups are
  chosen by `borg_notice_weapon_swap()` / `borg_notice_armour_swap()` and
  enabled by `borg_uses_swaps` (§13).
* `borg_wear_recharge()` swaps in a pack item with a running `timeout` so
  that it recharges (pack items, other than rods, do not recharge in the
  game), then rests (§11.12).

### 10.5 Blows per turn: `borg_calc_blows()`

`borg_calc_blows()` (`borg-trait.c:borg_calc_blows:1035-1071`) is a copy of
the engine's `calc_blows()` (`src/player-calcs.c`) with one difference in
the weight it uses and a final division to whole blows:

Transcribed from `borg-trait.c:borg_calc_blows:1042-1042` and from
`player-calcs.c:calc_blows:1711-1711` (in `src/`, not `src/borg/`):

```c
/* the Borg */
int weight     = item->weight * item->iqty;

/* the engine */
int weight = (obj == NULL) ? 0 : object_weight_one(obj);
```

`item->weight` is already one item's weight, so the Borg's copy weighs the
whole stack. For the wielded weapon `iqty` is 1 and the two agree. It is
also called on home items (`borg-home-notice.c`), where a stack of
weapons is weighed as one heavy weapon. The lesson for a rebuild is
general: a copy of an engine calculation must copy its semantics.

### 10.6 Spellbooks and spells

* **Book index.** `borg_cheat_spells()` maps each class book to its pack
  slot in `borg.book_idx[]` (−1 if not carried) and gives every spell a
  status from `player->spell_flags[]` and the level:
  `BORG_MAGIC_LOST`, `BORG_MAGIC_HIGH` (level too low),
  `BORG_MAGIC_OKAY` (learnable), `BORG_MAGIC_TEST` (learnt, never cast),
  `BORG_MAGIC_KNOW`. Spells are named by `enum borg_spells`
  (`MAGIC_MISSILE`, `PHASE_DOOR`, `IDENTIFY_RUNE`, …) in `borg-magic.h`.
* **Can I cast it?** `borg_spell_okay()` requires the spell to be legal
  (known and its book carried), the Borg not blind or confused, light
  available, and enough mana. From character level 35 a class-specific
  mana reserve is kept, waived for nourishment, teleport and (shallow)
  Magic Missile (`borg-magic.c:borg_spell_okay:419-494`).
  `borg_spell_fail(spell, allow_fail)` and `borg_spell_okay_fail()` add a
  failure-rate ceiling.
* **Casting** is `'m'`, the book's letter, the spell's letter
  (`borg-magic.c:borg_spell:596-630`).
* **Learning and practice.** `borg_play_magic(bored)` studies the
  best-rated learnable spell when the character can study (`'G'`, the
  book, the spell for choosing classes). Otherwise it casts untried spells
  to learn their effects only when **both** hold: it is in town or in
  munchkin mode, **and** `bored` is true
  (`borg-magic-play.c:borg_play_magic:165-170`); some spells are never "played" (`borg_can_play_spell()` in
  `borg-magic-play.c`).
* **Books as kit.** Books are scored in `borg_power_inventory()`, never
  crushed (`borg_drop_junk()` skips browsable kinds), bought by the store
  logic, and a missing dungeon book is one of the purchases that can set a
  money-scum target under `borg_self_scum` (§13.1).

### 10.7 Light and fuel

* `borg_maintain_light()` (`borg-light.c:borg_maintain_light:613-667`)
  returns `BORG_NO_NEED`, `BORG_MET_NEED` or `BORG_UNMET_NEED`. With no
  light it wields a lantern or a torch; a torch below 250 turns needs a
  spare torch; a lantern below 1000 turns is refuelled by the static
  `borg_refuel_lantern()` (§11.7). Necromancers and `OF_NO_FUEL` lights need
  nothing.
* `borg_think_dungeon_light()` (`borg-think-dungeon-util.c`) acts on an
  unmet light need early in the ladder: leave the level, recall, or flow to
  a lit area (`borg_flow_light(GOAL_FLEE)`).
* `borg_check_light()` casts the detections and the light spells on their
  timers (§4.5), and `borg_light_beam()` lights a corridor with the Light
  Beam spell.
* Light is also a precondition elsewhere: no spells without light, no
  spell practice without `BI_LIGHT`. In the depth checks, depth 1 is
  always allowed by `borg_prepared()`; for deeper levels
  `borg_prepared_aux()` requires some light (`BI_LIGHT >= 1`), and asks for
  five fuel only when `BI_LIGHT` is 0 — a condition the light check before
  it has already excluded.

### 10.8 Junk and crushing

* `borg_drop_junk()` (`borg-junk.c`) runs only when
  `borg_do_crush_junk` was set by a new pack read, not while recalling, and
  only when danger is at most a tenth of `BI_CURHP`. For each pack item
  that survives the exclusions (swap items, spellbooks, unaware scrolls,
  items whose value is above a per-depth limit, items still needing ID, and
  others), it removes one of the item (`iqty -= 1`), re-notices and scores
  `borg_power()`, and restores. The candidate whose removal gives the
  highest power **strictly above** the current power (`p > b_p`) is junk. It
  is inscribed `"borg ignore"` (`'{'`, item, text, Enter) and one of it is
  dropped (`'d'`, item, a count of 1 for a stack).
* `borg_destroy_floor()`, at the top of the ladder, then acts on a floor
  object the take list marks as junk (`value == -10`: inscribed
  `"borg ignore"`, or fully known with a negative bonus) by queuing the
  game's ignore command (`'k'`, `'-'`, `'a'`, `'a'`). Ignoring hides the
  object through the game's ignore settings; it is not an immediate
  deletion.
* `borg_drop_hole()` frees a slot when the pack is full (two passes, the
  second allowing unidentified items).
* `borg_drop_slow()` drops weight when slowed by it, valuing a tenth of a
  pound at one gold.
* `borg_dump_quiver()` shoots away cursed or negative ammunition.
* `borg_remove_stuff()` takes off equipment that reduces power.

### 10.9 Recovery and rest

`borg_recover()` (`borg-recover.c:borg_recover:59-469`) restores the Borg
after a fight: first "free" healing spells, then food, potions, scrolls,
staffs, rods and activations, then **rest** (`'R'` and a count, §2.7) when
`borg_check_rest()` says the grid is safe enough — to recharge a rod of
Healing or Recall, to regain hit points or mana, or to let a cut, poison,
fear or stun wear off. `borg_flow_recover()` walks to a grid where resting
is safe (`GOAL_RECOVER`). The ladder tries recovery twice: early when the
Borg has no light or is cut, poisoned or starving, and again after the
light check.

---

## 11. Defect shapes that a rebuild should not inherit

The individual bugs are on this fork's issue board (§18), with their
evidence and state. This section groups the *shapes* that the C Borg's
architecture produces and gives one example of each, read from the source
at the pin. A rebuild that recognises the shapes will avoid them whether or
not it has read the individual findings. Examples are described by their
code; whether a given one is tracked, fixed or disputed is the board's
business.

### 11.1 Stale state between calls

The Borg keeps a great deal of state in globals and expects every function
to leave it clean. The commonest form: a "we are only scoring" flag is set,
the function returns early, and the flag stays set for the next caller.

Example: `borg_attack()` sets `borg_simulate = true` before scoring and, when
no attack scores above zero, returns with `borg_attacking = false` but
without clearing `borg_simulate`
(`borg-fight-attack.c:borg_attack:5277-5305`). `borg_defend()` does the
same on its no-defence path.

**What to do differently:** a function that sets a global owes the caller a
restore on every exit path.

### 11.2 The wrong variable

Several parallel arrays and flags mean nearly the same thing and have the
same type, so mixing them compiles.

* `borg.has[]` is indexed by object kind, `borg.trait[]` by `BI_*`. A test
  written `borg.has[BI_CURHP] > 100` reads an item count against a
  hit-point threshold (`borg-fight-defend.c`, the Teleport Other / loskill
  defence). The intended expression is `borg.trait[BI_CURHP] > 100`.
* `generate_keypress()` resets the struct field `borg.dont_react`, while
  `borg_react()` tests the global `borg_dont_react` (§4.6).
* In `borg-home-notice.c` all five `OF_SUST_*` tests add to
  `num_sustain_str`.

**What to do differently:** name parallel things so the compiler can tell
them apart, and check an index against the array's documented meaning.

### 11.3 Dead code that matters

A branch that cannot be taken is harmful when it is the one that was
*supposed* to fire.

* `borg_test_stuff()` scores identify candidates with three successive
  **assignments** — artifact `value + 150000`, random-power ego
  `value + 100000`, then `value + 20000` if
  `borg_item_note_needs_id(item)` (`borg-item-wear.c:borg_test_stuff:101-134`).
  Both loops first skip items without `needs_ident`, and
  `borg_item_note_needs_id()` returns true for any item with
  `needs_ident` that is not `ident`
  (`borg-item-id.c:borg_item_note_needs_id:145-155`); since
  `object_fully_known()` implies `object_runes_known()`, that is every
  candidate except a cursed non-artifact weapon or bow, whose `ident` is
  forced true by `borg_notice_equipment()` (§11.6). So for ordinary
  candidates the last assignment always wins and the artifact and ego
  bonuses never survive; the same holds for the artifact bonus in the
  second loop. (An item that is both an artifact and an ego does not arise
  in ordinary generation: `apply_magic()` in `src/obj-make.c` returns once
  `make_artifact()` succeeds, before any ego is rolled.)
* `borg_spell_stat()` tests `spell != NULL` where `spell` is the address
  of an array element (`borg-magic.c:borg_spell_stat:239-249`).
* `if (randint1(100) < 1)` in `borg-flow-kill.c` is never true.
* The clock reset of §2.4 is behind a threshold that a stricter test
  reaches first.

**What to do differently:** a condition that looks unreachable is either a
typo or a missing case; write the test that would fail if it were fixed.

### 11.4 The wrong comparison

* In `borg_danger_physical()` the stun bonus for HURT blows is
  `if ((d_side < 3) && (z > d_dice * d_side))` directly after
  `z = (d_dice * d_side);`, so it is never true (`borg-danger.c`).
* The partial-damage hit chance is
  `150 - (((ac * 300) / 4) / 1 + ((r_ptr->level + power) * 3))`, which is
  at or below its floor of 5 for any armour class of 2 or more; and
  `borg_danger()` overwrites `full_damage` with `true` anyway (§8).

**What to do differently:** check a threshold at the extremes its inputs
can take.

### 11.5 The wrong member of a parallel set

* `borg_init_item_val()` looks up `sv_wand_confuse_monster` as
  `"Scare Monster"` (the fear wand) and `sv_wand_elec_ball` as
  `"Lightning Bolts"` (the bolt wand). Both names are real items, so both
  lookups succeed (`borg-item-val.c:borg_init_item_val:455-464`).
* `borg_light_beam()` starts the west scan at step 1 and the other three
  directions at step 0 (`borg-light.c:borg_light_beam:713-794`).
* `borg_flow_glyph()` skips "every non floor/glyph" grid with
  `ag->feat != FEAT_FLOOR && ag->glyph`, which skips only glyphed
  non-floor grids.

**What to do differently:** where a set is enumerated twice, derive one
from the other, and check that lookup names are the ones intended.

### 11.6 Mutation during a read-only pass

`borg_notice()` is meant to read the items and derive traits. But when the
bow or the weapon slot is empty **or holds a cursed non-artifact** (one
that is not marked uncursable, `cursed_nonartifact()`),
`borg_notice_equipment()` models "bare hands" by zeroing that item's
`ds`, `dd`, `to_d`, `to_h` and `weight` and setting `ident = true` **in
`borg_items[]`**, and for the bow also `sval = 0`
(`borg-trait.c:borg_notice_equipment:1868-1876` for the bow,
`borg-trait.c:borg_notice_equipment:1947-1955` for the weapon). For an
empty slot this is harmless. For a cursed bow or weapon, everything that
reads `borg_items[]` later in the same think sees an item with no dice, no
bonuses, no weight and `ident` set; a cursed bow no longer matches
`sv_sling` or `sv_long_bow`. `safe_items[]` keeps the real values.

**What to do differently:** a read pass must not write; model hypotheticals
on a copy.

### 11.7 Index and selection mistakes

* The Teleport Other skip list is read with
  `for (ii = 0; ii <= borg_tp_other_n; ii++)`, one entry past the count
  (`borg-danger.c:borg_danger_one_kill:2323-2330`).
* `borg_refuel_lantern()` has the right guards (`if (i >= 0)`) but the wrong
  search: with no flask of oil it starts at `borg_slot(TV_LIGHT,
  sv_light_lantern)` and walks forward until the first non-lantern
  (`borg-light.c:borg_refuel_lantern:552-608`). `borg_slot()` prefers the
  smallest pile and, among equal piles, the **last** match
  (`borg-inventory.c:borg_slot:88-127`), so for single lanterns the scan
  starts at the last lantern and an earlier lantern that still has fuel is
  never considered.

**What to do differently:** when a lookup has a tie-break, know which way
it breaks before using its result as the start of a scan; and check every
index that came from a count against the array it indexes.

### 11.8 Array capacity versus screen size

`borg_flow_light()` (`borg-flow-misc.c:borg_flow_light:824-874`) collects
every `BORG_GLOW` grid of the visible panel:

Transcribed from `borg-flow-misc.c:borg_flow_light:834-847` (comments removed):

```c
    for (y = w_y; y < w_y + SCREEN_HGT; y++) {
        for (x = w_x; x < w_x + SCREEN_WID; x++) {
            borg_grid *ag = &borg_grids[y][x];

            if (!(ag->info & BORG_GLOW))
                continue;

            borg_glow_y[borg_glow_n] = y;
            borg_glow_x[borg_glow_n] = x;
            borg_glow_n++;
        }
    }
```

`borg_glow_y[]` and `borg_glow_x[]` have `AUTO_LIGHT_MAX` (1536) entries
(`borg-cave-light.h:AUTO_LIGHT_MAX:30`) and there is **no bound check**. The
panel size comes from the terminal and the sidebar mode
(`src/ui-term.h`: `SCREEN_HGT`, `SCREEN_WID`; `src/ui-term.c`:
`row_top_map`, `row_bottom_map`, `col_map`). With one-cell tiles (the Borg
refuses scaled tiles unless `borg_allow_strange_opts` is set):

| Terminal | Sidebar mode | Map cells | vs. 1536 |
| --- | --- | ---: | --- |
| 80×24 | `SIDEBAR_LEFT` (0) | 22 × 66 = 1452 | fits |
| 80×24 | `SIDEBAR_TOP` (1) | 20 × 79 = 1580 | can overflow |
| 80×24 | `SIDEBAR_NONE` (2) | 23 × 79 = 1817 | can overflow |
| 80×28 | any | 1716 – 2133 | can overflow |
| 120×40 | any | 4028 – 4641 | can overflow |

So a write past the end is reachable whenever more than 1536 panel grids
are marked `BORG_GLOW` — possible in an ordinary 80×24 terminal with the
sidebar on top or off, and in any larger terminal.

**What the write corrupts is build-dependent.** It is undefined behaviour,
and the result depends on how the linker lays out the zero-initialised data that
follow the arrays and on whether the compiler keeps `borg_glow_n` in a
register during the loop. For one build — this tree at the pin, built with
`cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo -DSUPPORT_BORG=ON` (gcc 15.2.0,
`-O2 -g -DNDEBUG`) — `nm -S` lists `borg_glow_y` *before* `borg_glow_x`,
followed by `borg_glow_n` and then the `borg_light_*` arrays; the code
keeps `borg_glow_n` in a register and stores it after the loop. In that
build the overflowing `borg_glow_y[]` writes land in the head of
`borg_glow_x[]`, the overflowing `borg_glow_x[]` writes land on
`borg_glow_n`, padding and the head of `borg_light_y[]`, and the read-back
loop enqueues grids with correct `y` and wrong `x` near the start of the
list. Other builds will differ. The exact `nm` output, the disassembly
evidence and a simulation of that layout are in
`docs/audit/c-reports/spec-checks/glow_overflow_probe.c` and
`docs/audit/c-reports/spec-checks/RECHECK.md`; this specification relies only on the overflow.

**What to do differently:** size an array that holds per-screen data from
the screen, or bound the write.

### 11.9 Double counting

`borg_surrounded()` counts the non-safe neighbours with a four-branch
`if`/`else if` chain, followed by a **separate** `if` for traps
(`borg-escape.c:borg_surrounded:218-236`):

Transcribed from `borg-escape.c:borg_surrounded:219-236` (comments removed):

```c
        if (!borg_cave_floor_grid(ag))
            non_safe_grids++;
        else if (ag->feat == FEAT_NONE)
            non_safe_grids++;
        else if (ag->kill)
            non_safe_grids++;
        else if (feat_is_shop(ag->feat))
            non_safe_grids++;
        if (ag->trap && !ag->glyph)
            non_safe_grids++;
```

A grid that satisfies a chain condition and the trap condition — most
ordinarily a monster, awake or asleep, standing on a known trap — is counted
twice. The `monsters` count, by contrast, includes only awake, visible
monsters within three grids. `safe_grids`
starts at 8 and is reduced by the count, so double counts can drive it
below the number of monsters (even below zero), and `monsters >
safe_grids` then reports "surrounded".

**What to do differently:** write mutually exclusive tests as one chain; a
bare `if` beside an `else if` chain is a separate test and should say so.

### 11.10 A mode left by clearing another mode's flag

`borg_think_stair_scum()` notes `"# Leaving Scumming Mode. (Town or Weak)"`
and then clears `borg.lunal_mode`
(`borg-think-dungeon-util.c:borg_think_stair_scum:363-368`). There is no
scumming-mode flag; scumming is entered by its call site (depth 1 and
`borg.goal.fleeing_to_town`, `borg-think-dungeon.c:borg_think_dungeon:1340-1344`)
and it sets `borg.goal.fleeing`. The lunal mode it clears is a separate
mode with its own flags (`borg.lunal_mode`, `borg.goal.fleeing_lunal`). The
"Town" half of its guard (`BI_CDEPTH == 0`) cannot be true at that call
site.

**What to do differently:** the state that enters a mode and the state that
leaves it should be one structure, set and cleared together.

### 11.11 A shared running best across two scans

`borg_test_stuff()` runs two candidate loops — equipment and quiver, then
pack, equipment and quiver again — that share one `b_i`/`b_v` and the
`if (v <= b_v) continue;` test (`borg-item-wear.c:borg_test_stuff:101-205`).
A score left by the first loop is a floor for the second: the second loop's
"unaware item" bonus (by item type and `BI_MAXDEPTH`) is in competition
with first-loop scores computed by different rules. Combined with §11.3,
the first loop's scores at the pin are `value + 20000` for ordinary
candidates, and each of them is scored again by the second loop at least as
high, so the shared floor changes the second loop's choice only in two
cases: a **tie** (`v <= b_v` keeps the equipment or quiver item from the
first loop, where the second loop alone, scanning the pack first, would
keep an equal pack item), and the cursed-slot exception of §11.3, whose
first-loop score (`0 + 100000`, since a cursed item's value is 0) can
exceed a pack candidate the second loop would have chosen.

**What to do differently:** a scan that keeps a best candidate should own
its running best, or both scans should use one scoring rule.

### 11.12 A scan that keeps the last match

`borg_wear_recharge()` (`borg-item-wear.c:borg_wear_recharge:1451-1538`)
scans indexes `0 .. INVEN_TOTAL - 1` for an item with a running `timeout`
and records each qualifying one, so the **last** qualifying item wins.
There is no preference between a rod (rest), a worn item (rest) and a pack
item (swap in, then rest), and nothing rejects an item whose wield slot is
−1: if the winner is such an item and not a rod, the function returns
false without resting or swapping. At the pin, though, only rods carry a
`timeout` among unwieldable items (`tval_can_have_timeout()` in
`src/obj-tval.c` is `TV_ROD` only; activation timeouts are on wearable
items), so that last path needs an item state that ordinary play does not
produce. The last-match preference is real.

**What to do differently:** a search should prefer the best candidate
explicitly, and reject an unusable candidate when it is found.

---

## 12. The Borg in town

`borg-store.c`, `borg-store-buy.c`, `borg-store-sell.c`,
`borg-think-store.c`, `borg-home-notice.c`, `borg-home-power.c`.

* **Getting there.** `borg_choose_shop()` decides which shop to visit
  next (`borg.goal.shop`, with `borg.goal.ware`/`borg.goal.item`), and
  `borg_flow_shop_entry()` walks to its entrance (`GOAL_MISC`).
* **Inside.** `borg_think()` sees the store header and calls
  `borg_think_store()` (`borg-think-store.c:borg_think_store:285-357`):
  it notes the shop with `square_shopnum()`, clears the goal, sets
  `borg.in_shop`, runs `borg_best_stuff()` (home only), swaps a digger
  out, then — if `borg_choose_shop()` still wants something — sells
  (`borg_think_shop_sell()`) or buys (`borg_think_shop_buy()`) one thing,
  and otherwise leaves with four `ESCAPE`s. One transaction per think.
* **Choosing a transaction.** `borg_choose_shop()`
  (`borg-think-store.c:borg_choose_shop:86-280`) decides in this order:
  1. Only in town, not after 2000 Borg turns on the level, not with the
     anti-bounce counter above 1350.
  2. An existing transaction goal (`borg.goal.shop` and `borg.goal.ware`
     both set) stands.
  3. **Emergencies** set the shop: cut or poisoned → the temple (shop 3);
     no food, or no light at level 2+ → the general store (0); only a
     torch-level light and 100 gold → the general store. With no light, no
     food, a cut or poison (or the torch case below level 10) it buys at
     once, without visiting the other shops: `borg_think_shop_buy_useful()`,
     then `borg_think_home_buy_useful()`.
  4. Otherwise the goals are cleared; if `borg.goal.do_best` is set and
     neither the home nor the pack is full, go home (the best-kit search,
     §10.4).
  5. **Step 0**, money scumming for the player (target set, below it, in
     town, with light, and **not** `borg_self_scum`): only buy useful items
     from the shops, and nothing else.
  6. **Steps 1–5:** sell to the home what is useful later; sell useless
     items to the shops; buy useful items from the shops; buy useful items
     from the home; take items from the home to sell.
  7. If any money-scum target is set, stop here.
  8. **Steps 6–7B:** buy interesting items for the home; buy backup
     weapons, then armour, from the home when swaps are enabled.

  Each step simulates the transaction and compares `borg_power()` (for the
  Borg) or the home's own power (`borg_power_home()`, from
  `borg-home-power.c`).
* **Money scumming.** While a target (`borg_money_scum_amount`) is set and
  the Borg is in town with light and below it, the ladder calls
  `borg_money_scum()` instead of leaving the level: it rests in town,
  "waiting for towns people to breed", and the ordinary combat rungs kill
  them for their gold, and the store logic
  restricts itself as in steps 5 and 7. The target is either configured, or
  set by the buy logic under `borg_self_scum` (§13.1) and cleared to 0 when
  the wanted item is bought. `check_for_deactivate_or_death()` stops the
  Borg when gold is **greater than** the target at depth 0 and
  `borg_self_scum` is **off** (`borg.c:check_for_deactivate_or_death:190-199`).

---

## 13. Configuration (`borg.txt`)

`borg_init_txt_file()` (`borg-init.c`) reads `borg.txt`. The 35 settings
are the `BORG_*` enum in `borg.h` (before `BORG_MAX_SETTINGS`), in the same
order as the `borg_settings[]` table in `borg-init.c`, whose defaults are
below. The "Read by" column lists **every** file that reads the setting
(`grep -l 'borg_cfg\[BORG_…\]' src/borg/*.c` at the pin), including
logging (`borg-log.c`) and start-up checks and clamps (`borg-init.c`); the
"Effect" column describes the decision readers.

| Setting | Type | Default | Effect | Read by |
| --- | --- | --- | --- | --- |
| `borg_verbose` | bool | false | verbose notes | `borg-flow-kill.c`, `borg-flow-take.c`, `borg-io.c`, `borg-item-wear.c`, `borg-messages-react.c`, `borg-messages.c`, `borg-store-buy.c` |
| `borg_munchkin_start` | bool | false | munchkin mode while `BI_MAXCLEVEL < 12` (§13.1); munchkin power weights | `borg-flow-take.c`, `borg-power.c`, `borg-think-dungeon.c`, `borg-trait.c` |
| `borg_munchkin_level` | int | 12 | level used by the munchkin power weights (clamped 1–50) | `borg-init.c`, `borg-power.c`, `borg-trait.c` |
| `borg_munchkin_depth` | int | 16 | depth limit in munchkin mode | `borg-init.c`, `borg-think-dungeon.c` |
| `borg_worships_damage` | bool | false | §5.2 | `borg-log.c`, `borg-power.c` |
| `borg_worships_speed` | bool | false | §5.2 | `borg-log.c`, `borg-power.c` |
| `borg_worships_hp` | bool | false | §5.2 | `borg-log.c`, `borg-power.c` |
| `borg_worships_mana` | bool | false | §5.2 | `borg-log.c`, `borg-power.c` |
| `borg_worships_ac` | bool | false | §5.2 | `borg-log.c`, `borg-power.c` |
| `borg_worships_gold` | bool | false | keep and sell more items for gold (junk and shop-sell rules), dive later | `borg-junk.c`, `borg-log.c`, `borg-store-sell.c`, `borg-think-dungeon-util.c` |
| `borg_plays_risky` | bool | false | looser depth requirements; no self-lunal; no money-scum rest; less likely to heal mid-fight (`borg_heal()` chance +5) and more likely to stay in a fight (`borg_escape()` risk boost 3) | `borg-caution.c`, `borg-escape.c`, `borg-log.c`, `borg-prepared.c`, `borg-think-dungeon-util.c`, `borg-think-dungeon.c` |
| `borg_kills_uniques` | bool | false | do not dive past living uniques (§5.5) | `borg-prepared.c` |
| `borg_uses_swaps` | bool | **true** | backup weapon/armour swaps | `borg-log.c`, `borg-trait.c` |
| `borg_uses_dynamic_calcs` | bool | false | formula sections (§5.3) | `borg-formulas.c`, `borg-init.c`, `borg-power.c`, `borg-prepared.c` |
| `borg_stop_dlevel` | int | 128 | stop on reaching this depth | `borg.c` |
| `borg_stop_clevel` | int | 51 | stop on reaching this level | `borg.c` |
| `borg_no_deeper` | int | 127 | never prepared for this depth or deeper | `borg-prepared.c`, `borg-think-dungeon-util.c` |
| `borg_stop_king` | bool | **true** | stop after winning | `borg-init.c`, `borg.c` |
| `borg_cheat_death` | bool | false | turn on the game's `cheat_live` at start; on "Die?", cure and continue instead of respawning | `borg-init.c`, `borg-messages-react.c` |
| `borg_respawn_winners` | bool | false | respawn after winning | `borg-init.c`, `borg.c` |
| `borg_respawn_class` | int | -1 | class of a respawned character; −1 = random | `borg-init.c`, `borg-reincarnate.c` |
| `borg_respawn_race` | int | -1 | race of a respawned character; −1 = random | `borg-init.c`, `borg-reincarnate.c` |
| `borg_chest_fail_tolerance` | int | 7 | chest disarm failure tolerance | `borg-flow.c` |
| `borg_delay_factor` | int | 0 | extra delay per action | `borg-think-dungeon.c` |
| `borg_money_scum_amount` | int | 0 | money-scum target; 0 = off | `borg-fight-defend.c`, `borg-flow-kill.c`, `borg-flow-take.c`, `borg-junk.c`, `borg-log.c`, `borg-store-buy.c`, `borg-store-sell.c`, `borg-think-dungeon-util.c`, `borg-think-dungeon.c`, `borg-think-store.c`, `borg.c` |
| `borg_self_scum` | bool | **true** | let the buy logic set a money-scum target for a wanted item it cannot afford; do not stop when a target is reached; skips step 0 of the shop choice (§12, §13.1) | `borg-store-buy.c`, `borg-think-store.c`, `borg.c` |
| `borg_lunal_mode` | bool | false | initial value of the `borg.lunal_mode` flag only; it does not run the lunal routine (§13.1) | `borg-init.c` |
| `borg_self_lunal` | bool | false | enter lunal mode by itself (§13.1) | `borg-think-dungeon.c` |
| `borg_enchant_limit` | int | 12 | how far to enchant | `borg-init.c`, `borg-item-enchant.c`, `borg-trait.c` |
| `borg_dump_level` | int | 1 | write the map on death from this level | `borg-messages-react.c`, `borg-think.c`, `borg.c` |
| `borg_save_death` | int | 1 | write the death logs | `borg-log.c` |
| `borg_stop_on_bell` | bool | false | stop on the terminal bell | `borg-io.c` |
| `borg_allow_strange_opts` | bool | false | start even with scaled tiles or `birth_force_descend` (otherwise start-up fails) | `borg-init.c` |
| `borg_autosave` | bool | false | autosave periodically | `borg-update.c` |
| `borg_restore_ignore_settings` | bool | false | restore the player's ignore settings when stopping | `borg-init.c` |

A rebuild that changes a default changes the Borg from the first turn;
note the three that default to **true**.

### 13.1 Modes that change goal selection

Several "modes" do not add rungs to the ladder; they **replace** most of it
for a while, or change what the flow will accept.

| Mode | Entered when | What changes | Left when |
| --- | --- | --- | --- |
| **Lunal** (`borg.lunal_mode`, and the `borg_think_dungeon_lunal()` routine) | The **routine** runs only inside the self-lunal block: `borg_self_lunal`, not `borg_plays_risky`, prepared for `BI_CDEPTH + 15` and has been that deep, `BI_CDEPTH >= 1` and `BI_CDEPTH > BI_CLEVEL / 3`; the block sets the flag and calls the routine (`borg-think-dungeon.c:borg_think_dungeon:1438-1456`). The **flag** alone can also be true from start-up (`borg_lunal_mode`), without the routine | routine: stair-dive and stair-climb without fighting, pick up what is on the way, `borg.goal.fleeing_lunal`. Flag (with or without the routine): the flow avoids monster grids and skips danger evaluation (§6.2, §6.5) | routine exits clear the flag: town, weak, no stairs known, close to the depth it is prepared for, depth 50; also certain monster spells that move the Borg (`borg-messages.c`) and the stair-scum exit (§11.10) |
| **Munchkin** (`borg.munchkin_mode`) | `borg_munchkin_start` and `BI_MAXCLEVEL < 12`, in the dungeon (`borg-think-dungeon.c:borg_think_dungeon:1460-1471`) | `borg_think_dungeon_munchkin()` takes over: low-level stair diving to `borg_munchkin_depth`, munchkin attackers, munchkin item pickup; flow as for lunal | level 12, town, weak, no stairs |
| **Stair scum** | depth 1 and `borg.goal.fleeing_to_town` | `borg_think_stair_scum()`: take objects, then leave by any stair (§11.10) | weak or full pack |
| **Self scum** (`borg_self_scum`) | a wanted shop item the Borg cannot afford (e.g. a missing dungeon book, Teleport or Teleport Level scrolls), character level 10+, light, 100+ food, cost at most 85,000 and a good chance to avoid theft (`borg-store-buy.c`) | sets the **money-scum target** to the item's cost, which turns on money scumming (next row); with `borg_self_scum` set, reaching the target does not stop the Borg. It does not enable the depth-1 stair-scum routine | the target is cleared when the item is bought |
| **Money scum** | a target is set (configured or by self scum), in town, with light, gold below it | `borg_money_scum()` (rest in town) replaces leaving the level; store steps restricted (§12) | gold above the target: the Borg stops unless `borg_self_scum` |
| **Digging** (`borg.status.digging`) | anti-summon corridor builders (`GOAL_DIGGING`) | flow tunnels; danger is not evaluated | corridor finished |
| **Vault** (`borg.status.vault`) | a vault is guessed on the level | an extra block of rungs: kill, take, excavate, `borg_flow_vault()` | level change |
| **Boosted bravery** | the ladder ran out (§2.8) | `borg.avoidance` raised; `borg_think_dungeon_brave()` | the pass ends |
| **Desperate** (`borg.status.desperate`) | `borg_caution()` dashes for a nearby stair (§9.1, step 11) | that flood avoids unknown and monster grids and skips danger evaluation | cleared straight after the dash |

---

## 14. Death and respawning

`borg-reincarnate.c` and the death path of `check_for_deactivate_or_death()`.

* **Death.** On `player->is_dead` the Borg writes the map (from
  `borg_dump_level`), logs the death (`borg_log_death()`,
  `borg_log_death_data()`), flushes, clears the reactions and stops.
* **"Die?" with the game's `cheat_live`.** `borg_react_prompted()` either
  cures and continues (`borg_cheat_death`) or starts a respawn
  (`borg_reincarnate_start()`: random or configured race and class, a
  random name, birth keys queued).
* **Winning.** With `borg_stop_king` the Borg stops; with
  `borg_respawn_winners` it logs and calls `borg_force_reincarnate()`
  (queues `Q`, `y`, `@`).
* **After a respawn** `borg_reincarnate_end()` re-reads race/class data,
  notices the player, clears reactions, forces the level-change
  bookkeeping (`borg.status.old_depth = 128`), marks the character
  `NOSCORE_BORG`, re-applies the options and turns on `cheat_live`.

---

## 15. Logging and diagnostics

* `borg_note()` — a decision note, added to the message log
  (`message_add()`) and the Borg's log file; notes are where the
  anti-bounce `+10` is applied (§2.5).
* `borg_warning()` — a louder note; `borg_oops()` — note, stop, flush.
* `borg_log_death()` writes `borg-log.txt` and `borg_log_death_data()`
  writes `borg.dat`, both in `ANGBAND_DIR_ARCHIVE` and only with
  `borg_save_death` (`borg-log.c:borg_log_death:48-89`,
  `borg-log.c:borg_log_death_data:94-121`).
* `borg_write_map()` — dump the map; `borg_display_status()` — the status
  window; `borg_dump_recent_keys()` — the key history, used when an
  unexpected "Direction?" appears.
* `^Z` opens the Borg command menu (`borg-commands.c`): start, step, the
  debugging displays.

---

## 16. What a rebuild must preserve

If a rebuild is to *be* the C Borg, these must hold. They are sorted by
kind, because they are not equally fundamental.

### 16.1 Observable behaviour (the contract)

1. **Keystrokes are the action channel, and the queue discipline holds.**
   The one exception is the cheat-death cure-all (§1.2), which a rebuild
   reproduces as an explicit exception, not as a precedent: each think queues a complete key sequence; the Borg thinks only
   when the queue is empty; a think that queues nothing stops the Borg; an
   action closes with `ESCAPE` where the C Borg does (§2.2, §2.6).
2. **Rest and repeat are passed through** as `EVT_NONE`, not interrupted
   (§2.7).
3. **Perception is the two channels of §1.1** — the message line and store
   header from the screen, plus direct reads of the player's own state and
   gear, `map_info()` for the current panel (remembered features and
   objects, currently visible monsters), the named cave predicates, the
   **real identity** of each newly sighted monster and floor object
   (`borg_guess_race()`, `borg_guess_kind()`), the tracked monsters' real
   condition (`borg_update_kill_old()`), and full race/kind data. Nothing
   more was found: no reading of the map outside the panel beyond what the
   Borg remembers, no monsters the player cannot see, no scanning of the
   level's monster or object lists. A rebuild that uses
   less plays worse; one that uses more plays better; neither is this Borg.
4. **The Borg thinks with its own RNG** (§1.2), so its choices do not move
   the game's RNG during thinking.
5. **The decision ladder's order** (§2.8), including caution before
   routine combat, combat before pickup and exploration, and the three
   bravery passes at the end.
6. **Prompt answers** — the pre-answered keys and the caught prompts of
   §4.6 and §9.3.
7. **The option set** the Borg imposes (§1.2): the screen it reads depends
   on it.

### 16.2 Decision-changing constants and state machines

These are not visible on their own, but a rebuild that changes them makes
different decisions.

1. **Power is the utility** (§5) with the 40,000-per-prepared-depth bonus,
   and `borg_prepared()`'s requirements (§5.5) gate every dive.
2. **Danger** on the 0–2000 scale with the regional and per-grid fear terms
   (§8), and the **fear thresholds** that turn danger into "impassable"
   (§6.5).
3. **`borg.avoidance`** and its three boosts (§2.8).
4. **The flow's byte costs and the 250 limit** (§6.4).
5. **The anti-bounce counter** — its increments, resets and thresholds
   (§2.5). The `"Best Combo"` trigger does not fire at the pin; the clock
   reset at `BORG_TIME_MAX - 100` is unreachable (§2.4). Neither is
   behaviour a rebuild needs; the abort at `BORG_TIME_MAX - 3000` and the
   two fixed clock windows are.
6. **Simulations are meant to be undone** (§5.4): the discipline is that a
   "try this on" pass restores `borg_items[]` from `safe_items[]` and
   re-notices. The C Borg does not always keep it (§11.1, §11.6); where it
   does not, that is a §11 defect and falls under §17.2, not under this
   rule.
7. **The memory model:** the map and the monster/object lists are patched
   across turns and cleared on a level change; the trait vector and
   `borg_items[]` are rebuilt every think (§3).

### 16.3 Configuration

1. **The 35 settings and their defaults** (§13), including the three that
   default to true.
2. **The worship options as weight-branch switches** (§5.2), not
   multipliers.
3. **The mode switches** (§13.1) and the conditions that enter and leave
   them.

---

## 17. What a rebuild should *not* preserve — and which rule wins

### 17.1 Principles

These are the principles a rebuild applies when it finds itself
reproducing one of the §11 shapes.

1. **Where a comment states an intent and the code does something else,
   find out which one is the design** — and record the decision.
2. **Do not preserve an unbounded write.** A buffer filled from a quantity
   the environment controls (terminal size, map size, stack size) must be
   bounded by that quantity (§11.8).
3. **Do not preserve a count that can be double-counted or go negative**
   (§11.9).
4. **Do not preserve a mode flag that one routine sets and another clears**
   (§11.10).
5. **Do not preserve a copy of an engine function that has drifted from
   it**, or write down why it differs (§10.5).
6. **Do not preserve a read pass that writes** (§11.6), or a simulation
   that does not restore (§5.4, §11.1).
7. **Do not preserve state that leaks between branches** (§11.1, §11.11).

### 17.2 The conflict with §16, stated

§16 says "preserve the C Borg's behaviour". §17 says "fix these". They
conflict whenever a §11 defect changes a decision, and **a rebuild that
applies §17 is a different Borg**: it will make different choices in the
states the defect touches.

The recall-depth branch of §9.3 is the plain case. Its comment says "Do
reset Depth"; the code queues `'n'`; the note it writes says "Not Resetting
recall depth". Treating the comment as the specification would make the
Borg reset its recall depth in the endgame where the C Borg does not — a
real change in where it returns after recall.

The rule this document adopts:

* **By default, §16 wins.** The code at the pin is the specification of the
  C Borg's behaviour. A rebuild that claims to *be* the C Borg reproduces
  the behaviour, defects included, wherever the behaviour is defined.
* **§17 applies without conflict where the C behaviour is not defined** —
  undefined behaviour such as the glow-array overflow (§11.8), whose effect
  differs by build. There is no C behaviour to preserve; bounding the write
  is not a departure.
* **Every other §17 fix is a declared departure.** A rebuild may make it,
  but must name it, say which decisions it changes, and not claim parity
  for those decisions. The §11 examples fall into two groups:
  * **Fixing them changes decisions in reachable states** (shown at source
    level): the identify scoring (§11.3, §11.11), the wand name lookups and
    the light-beam range (§11.5), the cursed-item zeroing (§11.6), the
    lantern scan start (§11.7), the surrounded count (§11.9), the
    `borg.has[BI_CURHP]` test and the two reaction flags (§11.2), and the
    recall-depth answer (§9.3).
  * **Latent, masked or not shown to change a decision**: the partial-damage
    hit chance and the HURT stun test (§11.4) — `borg_danger()` forces
    `full_damage = true`, so the first is bypassed from that entry point;
    the stale `borg_simulate` (§11.1), whose effect depends on what the next
    caller does; the Teleport Other `<=` read (§11.7); the scum/lunal flag
    (§11.10), whose "Town" half is unreachable; and the recharge give-up
    path (§11.12), which needs an item state ordinary play does not
    produce. Fixing these is still a departure in principle; whether it
    changes play is unmeasured.

Where comment and code disagree (§17.1, rule 1), the code is the default
reading: the comment is evidence of intent, not the behaviour.

---

## 18. Companion documents

| Document | What it holds |
| --- | --- |
| [the issue board](https://github.com/diegoami/neo-angband-tracker/issues?q=label%3Aupstream-c) (labels upstream-c and finding) | the canonical, live state of every C Borg finding — the only place a finding's state is recorded |
| `docs/c-bugs/catalogue.json`, `docs/c-bugs/findings/CBUG-NNNN.json` | the machine-readable finding catalogue |
| `docs/c-bugs/watch-index.json` | the data-only watch index |
| `patches/upstream-c/` | stored proposal patches |
| `docs/audit/c-reports/spec-checks/` | the standalone check programs, their provenance and harness commands (`PROVENANCE.md`), and the mechanical re-check of this document (`RECHECK.md`) |

The board is where the bugs are. This document is the behavioural
reference: what the Borg is, how it is built, and what a rebuild has to
preserve. Neither substitutes for the other.

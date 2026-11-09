# Chapter 7 — The Town and the Dungeon

> Scope: how Angband 4.2.6 builds the town and every dungeon level, what is
> special (and specially dangerous) about each kind of level, rooms, vaults,
> pits and nests, terrain, doors and stairs, level feelings, the movement
> commands that change your depth (stairs, trapdoors, Deep Descent, Word of
> Recall, Teleport Level), the quest levels and the win condition, and a
> depth-by-depth map of where the game's well-known "danger zones" are.
>
> Every number in this chapter is taken from the game data in
> `lib/gamedata/*.txt` or from the C source in `src/`. File and function
> names are cited so you can check them. Where the official manual
> (`docs/dungeon.rst`) already says something, this chapter goes into the
> mechanics behind it.

---

## 7.1 Units, sizes and time

| Quantity | Value | Source |
|---|---|---|
| Dungeon level size (max) | 66 rows × 198 columns | `constants.txt` `world:dungeon-hgt:66`, `world:dungeon-wid:198` |
| Town size | 22 rows × 66 columns | `constants.txt` `world:town-hgt:22`, `world:town-wid:66` |
| Number of depths | 128 (town = 0, DL1 … DL127) | `constants.txt` `world:max-depth:128`; `world.txt` |
| Feet per level | 50 ft (DL10 = 500 ft) | `docs/dungeon.rst` |
| Levels moved per stair | 1 | `constants.txt` `world:stair-skip:1` |
| Max monsters per level | 1024 | `constants.txt` `level-max:monsters:1024` |
| Max pits/nests per level | 2 | `constants.txt` `dun-gen:pit-max:2` |
| Day length | 10 000 "days units" → a full day/night cycle is `10 × day_length` = 100 000 game turns | `constants.txt` `world:day-length:10000`; `src/game-world.c:is_daytime()` |
| Energy per player action | 100 | `constants.txt` `world:move-energy:100` |

### Game turns vs. player turns

The game clock (`turn`) advances in *game turns*. Each game turn every
creature gains energy from the table `extract_energy[]`
(`src/game-world.c`); acting costs 100 energy. At normal speed (+0) you gain
10 energy per game turn, so **one normal-speed player turn = 10 game turns**.
Selected values from the table:

| Speed | Energy / game turn | Game turns per action | Relative to +0 |
|---|---|---|---|
| −10 | 5 | 20 | ×0.5 |
| +0 | 10 | 10 | ×1 |
| +5 | 15 | 6.7 | ×1.5 |
| +10 | 20 | 5 | ×2 |
| +20 | 30 | 3.3 | ×3 |
| +30 | 38 | 2.6 | ×3.8 |
| +40 | 42 | 2.4 | ×4.2 |

Diminishing returns start at about +20. This matters for everything below
that is counted in "turns": the world-processing routine `process_world()`
runs once every 10 game turns (`src/game-world.c`, `if (!(turn % 10))`), so
**timers such as Word of Recall and Deep Descent count down in normal-speed
turns, not in your actions**. At +20 speed you get roughly three actions per
recall-timer tick.

---

## 7.2 The Town (depth 0)

### 7.2.1 Layout

The town is built by `town_gen()` / `town_gen_layout()` in
`src/gen-cave.c`:

* A town-sized area surrounded by permanent wall, with 3–5 lava streamers
  (`num_lava = 3 + randint0(3)`) worked into the boundary rock. Lava is real
  lava (see 7.6) — do not walk into it.
* The single down staircase is placed near the **north wall** (a random x
  near the middle, `rand_spread(town_wid/2, town_wid/6)`), with no lava next
  to it.
* The town is divided into lots around a crossroads; the eight shops are
  placed along the two streets; up to 80% of the remaining lots become
  "ruins" (rubble-filled lots, `build_ruin()`), which make good
  line-of-sight breakers.
* The layout is generated the first time and then kept; you always return
  to the same town (it is copied back from the chunk list).

### 7.2.2 Day and night

* `is_daytime()` is true for the first half of each 100 000-game-turn cycle
  (`src/game-world.c`). Dawn and dusk are announced ("The sun has risen.",
  "The sun has fallen.") and the whole town is lit or darkened by
  `cave_illuminate()`.
* In daylight you need no light source in town (`calc_light()` in
  `src/player-calcs.c` returns early in daytime town).
* **Residents**: on arrival the town is populated with
  `mon-gen:town-day:4` monsters by day and `mon-gen:town-night:8` by night
  (`constants.txt`; `town_gen()`), and more wander in over time. Night town
  is twice as crowded.

### 7.2.3 Townspeople

All depth-0 monsters (from `monster.txt`): filthy street urchin, scrawny
cat, scruffy little dog, Farmer Maggot (unique), blubbering idiot,
boil-covered wretch, village idiot, pitiful-looking beggar, mangy-looking
leper, squint-eyed rogue, singing happy drunk, aimless-looking merchant,
mean-looking mercenary, battle-scarred veteran, red-hatted elf and Father
Christmas. The last two are flagged `SEASONAL` and only appear between
24 and 26 December (`src/mon-make.c`, `get_mon_num_prep` check on
`date->tm_mon == 11`).

Practical notes:
* Townspeople give no experience (`docs/dungeon.rst`).
* The **squint-eyed rogue** steals gold, and the **mean-looking mercenary**
  and **battle-scarred veteran** can actually hurt a level-1 character. The
  APWBorg automatic player treats all three (plus urchins) as reasons to
  leave the level while it is clvl ≤ 8 (`src/borg/borg-flow-kill.c`, the
  "Scan for Scary Guys" block); mages and priests stay afraid of the rogue
  until clvl 6.
* Town monsters are never generated in the dungeon (`get_mon_num_prep`:
  "No town monsters in dungeon").

### 7.2.4 The stores

Store definitions are in `lib/gamedata/store.txt`; mechanics in
`src/store.c`.

| # | Store | Always stocked | Notable "normal" stock |
|---|---|---|---|
| 1 | General Store | Cloak, Ration of Food, Wooden Torch, Flask of Oil, Shovel, Pick, Iron Shot, Arrow, Bolt | Wine, Slime Mold, Whisky, Hard Biscuit, mushrooms of Stoneskin and Purging |
| 2 | Armoury | — | sandals/boots, caps/helms, robes through Metal Scale and Chain Mail, gloves/gauntlets, shields |
| 3 | Weaponsmith | — | shots/arrows/bolts, Dagger … Bastard Sword, Spear … Lucerne Hammer, Whip, Sling, Short/Long Bow, Light Crossbow |
| 4 | Bookseller | the town book of each realm (magic, prayer, nature, shadow) | — |
| 5 | Alchemist | **Word of Recall, Phase Door, Remove Curse, Cure Light Wounds** | Monster Confusion, Magic Mapping, Detect Invisible, Recharging, Remove Hunger, Blessing, Holy Chant, Boldness, Berserk Strength, Resist Heat/Cold, Restore Life Levels, Neutralize Poison, **Cure Serious Wounds, Cure Critical Wounds** |
| 6 | Magic shop | — | rings (Searching, Feather Falling, Protection, Open Wounds), amulets (Slow Digestion, Resist Acid), wands (Light, Slow/Confuse Monster, Magic Missile, Stinking Cloud, **Stone to Mud**), staves (**Teleportation**, Light, **Mapping**, Detect Invisible, **Detect Evil**) |
| 7 | Black market | anything except artifacts | price = value × 2 (sale markup) × 150% (`adjust = 150`) = **3× normal** (`src/store.c:price_item()`) |
| 8 | Home | your storage | — |

Restocking:
* Stores only change while you are in the dungeon. Every `10 ×
  store:turns` = 10 000 game turns spent in the dungeon increments a
  counter (`daycount`, `process_world()`); on your return to town
  `store_update()` runs `store_maint()` once per accumulated "day"
  (`src/store.c`). A long expedition therefore re-rolls the shops many
  times; a quick in-and-out (e.g. stair-scumming DL1 for a few hundred
  turns) usually re-rolls nothing.
* Each "day" there is a 1 in `store:shuffle` = 25 chance that one shopkeeper
  retires and is replaced (`store_shuffle()`).
* Stores hold at most `store:inven-max` = 24 item types.

Selling: by default `birth_no_selling` is **on** (`list-options.h`). Shops
then pay nothing (`price_item()` returns 0 when buying) but still accept and
identify items, and **gold found in the dungeon is multiplied by 5**
(`src/obj-make.c:make_gold()`, `value *= 5` when `birth_no_selling` and
`depth > 0`). With selling on, shops pay 2/3 of value (the black market
1/3), capped by the owner's purse.

Starting gold is `player:start-gold:600` (`constants.txt`); the start kit
(`birth_start_kit`, on by default) is deducted from that.

---

## 7.3 How a level is chosen: cave profiles

Every new level is built by one *profile* (`lib/gamedata/dungeon_profile.txt`,
builders in `src/gen-cave.c`). `choose_profile()` in `src/generate.c` picks
it in this exact order:

1. **Depth 0** → `town`.
2. **Quest level** (DL99 while Sauron lives, DL100 while Morgoth lives) →
   always `classic`, full size (`size_percent = 100`).
3. **Labyrinth check** (`labyrinth_check()`): never above DL13. Base chance
   2%, +1% for each of 3, 5, 7, 11, 13 that divides the depth.
4. **Moria check**: on DL10–39, a flat 1 in 40 (2.5%).
5. Otherwise a weighted draw among profiles with positive `alloc` whose
   `min-level` ≤ depth.

### 7.3.1 Weighted draw (step 5)

| Profile | alloc | min-level |
|---|---|---|
| classic | 90 | — |
| modified | 97 | — |
| cavern | 10 | 15 |
| lair | 1 | 20 |
| gauntlet | 1 | 20 |
| hard centre | 1 | 50 |
| town / labyrinth / moria | −1 (only via hard-coded checks) | — |

Resulting probabilities *given that steps 3–4 did not fire*:

| Depth | classic | modified | cavern | lair | gauntlet | hard centre |
|---|---|---|---|---|---|---|
| 1–14 | 48.1% | 51.9% | — | — | — | — |
| 15–19 | 45.7% | 49.2% | 5.1% | — | — | — |
| 20–49 | 45.2% | 48.7% | 5.0% | 0.5% | 0.5% | — |
| 50+ | 45.0% | 48.5% | 5.0% | 0.5% | 0.5% | 0.5% |

### 7.3.2 Labyrinth chance by depth

| Depth divisible by | Chance |
|---|---|
| none of 3,5,7,11,13 (e.g. DL17, 19, 23, 29, 31, 37, 41, 43, 47 …) | 2% |
| one of them (e.g. DL13, 20, 22, 25, 27 …) | 3% |
| two of them (e.g. DL15, 21, 30, 33, 35, 39, 42, 45, 55, 60, 63, 65, 66, 70, 77, 78, 84, 90, 91, 99*) | 4% |
| three of them (first is DL105) | 5% |

(*DL99 is a quest level while Sauron lives, so it is always classic.)

Labyrinths and gauntlets are refused when `birth_levels_persist` is on
(the builders return an error and generation retries); hard-centre levels
likewise.

---

## 7.4 The profiles in detail

### 7.4.1 Classic (`classic_gen`)

`params:11:50:200:2` — 11×11 blocks, aim for 50 rooms (the code notes the
real maximum is 36 blocks, so generation always stops when every block has
been tried), "unusual" 200, maximum room rarity 2.

* Size: always the full 66×198 grid. (The `size_percent` roll is computed
  but, per the comment, "currently does nothing" for classic.)
* At least 2 rooms or the level is regenerated.
* Tunnels: `tunnel:10:30:15:25:50` — 10% random direction, 30% change
  direction, 15% stop, 25% door at a room entrance, 50% door at a junction.
* Streamers: 3 magma (1 in 90 has treasure) and 2 quartz (1 in 40 has
  treasure); `streamer:5:2:3:90:2:40`.
* **Stairs**: 3–4 down, 1–2 up (`handle_level_stairs(... rand_range(3,4),
  rand_range(1,2))`).
* Rubble/traps/monsters scale with `k = clamp(depth/3, 2, 10)`:
  `randint1(k)` rubble in corridors, `randint1(k)/5` corridor traps.
* **Monsters**: `level-min (14) + 1d8 + k` monsters placed away from you
  (`pick_and_place_distant_monster`) — 17–24 at DL1, 25–32 from DL30 on.
* **Objects**: `Rand_normal(9,3)` in rooms (`dun-gen:amt-room:9`),
  `Rand_normal(3,3)` anywhere (`amt-item:3`), `Rand_normal(3,3)` gold piles
  (`amt-gold:3`).

### 7.4.2 Modified (`modified_gen`)

`params:1:50:300:2` — block size 1 (rooms find their own space), unusual
300 (rare rooms are rarer than in classic per attempt, but many more
attempts are made).

* Size varies: `i = 1d10 + depth/24`; i<2 → 75%, <3 → 80%, <4 → 85%,
  <5 → 90%, <6 → 95%, else 100%; then ±5% jitter per dimension. Deeper
  levels are more often full-size.
* Rooms are built until floor grids ≥ (area/7) and at least 2 rooms exist
  (gives up after 500 attempts). Roughly 22 rooms per level per the code
  comment.
* Unlocks the "new" vaults, **huge rooms** (DL40+), **rooms of chambers**
  (DL10+) and **Interesting rooms** (any depth).
* Same stairs (3–4 down, 1–2 up), monster count and item counts as classic.
* `mon_restrict(NULL, …)` — no monster theme.

### 7.4.3 Moria (`moria_gen`)

Only DL10–39, 2.5% of eligible levels.

* Same sizing rule as modified; rooms are almost all **moria rooms**
  (irregular "starburst" caves, `build_moria()`); no simple/large/crossed
  rooms. Moria rooms are lit with chance based on `depth <= randint1(35)`
  (vs. 25 for ordinary rooms) so they stay lit a little deeper.
* Monsters are generated under `mon_restrict("Moria dwellers")` — the
  `pit.txt` entry "Moria dwellers" is **orcs, ogres, trolls and giants**.
  Expect a level crowded with orc/troll packs. Good experience for a melee
  character; deadly for a weak caster who meets a troll pack in the open.
* Vaults, pits and nests can still appear (same rarity-2 list as modified).

### 7.4.4 Cavern (`cavern_gen`) — DL15+, 5%

* Size: ½ to ¾ of the full height and width.
* A cellular-automaton cave (density 25–40% floor, 3–6 smoothing passes);
  **no rooms, no lit grids** — the whole level is dark.
* Stairs: **1–3 down, 1–2 up** (fewer than normal levels).
* `k = max(4·k·area/fullarea, 6)`; monsters `1d8 + k` (note: *no* +14 base,
  so caverns are relatively sparse); objects `Rand_normal(k,2)` generated
  **5 levels deeper** than the level, gold `Rand_normal(k/2,2)`, plus
  `randint0(k/4)` guaranteed *good* objects.
* Danger: you fight in irregular open terrain with no corridors to retreat
  into, in the dark. Monsters with ranged attacks and fast packs (hounds,
  wolves) are at their best. On the plus side, the loot is above depth.

### 7.4.5 Labyrinth (`labyrinth_gen`) — DL13+, 2–4%

* Maze size: `h = 15 + 2·randint0(depth/10)`,
  `w = 51 + 2·randint0(depth/10)` — small (15×51 at DL13, up to 33×69 at
  DL90+).
* **Lit** if `randint0(depth) < 25` *or* a 50% roll. Always lit at DL≤25;
  75% lit at DL50; ~62% at DL100 (approximately 50% + 12.5%).
* **Known** (whole map revealed on arrival, via `light_level`) only if lit
  *and* a second independent `randint0(depth) < 25`. Always known at DL≤25,
  37.5% at DL50.
* **Soft walls** (granite you can tunnel/Stone-to-Mud) if `randint0(depth)
  < 35` or a 2/3 roll; otherwise walls are **permanent rock**.
* One closed (possibly locked) door per 100 squares.
* Rewards: **unlit** labyrinths get `Rand_normal(3,2)` extra *good* items;
  **permanent-walled** labyrinths get `Rand_normal(2,1)` *great* items.
  Plus `randint1(2)` good items always, and `Rand_normal(6k,2)` ordinary
  objects.
* Monsters: `14 + 1d8 + k'` where `k'` is k scaled by maze area × 3. Traps:
  `randint1(k')` (labyrinths are trap-heavy compared to classic levels,
  whose trap count is divided by 5).
* Stairs: the builder only guarantees one up and one down staircase.

Why labyrinths are dangerous: in an unknown, unlit maze you cannot see what
is around the next corner, you can be pinned in a one-wide corridor from
both ends, Phase Door rarely moves you far in "maze distance", and with
permanent walls you cannot dig an escape. Read Magic Mapping / Deep
Descent / Teleport Level early if the maze is unknown and the danger
feeling is bad.

### 7.4.6 Lair (`lair_gen`) — DL20+, 0.5%

* The level is split into a "normal" half built like a modified level and a
  **lair** half built as a cavern.
* The normal half gets only `1d4 + k` monsters; **the lair gets
  `14 + 1d20 + k`** monsters all drawn from one `pit.txt` theme (chosen by
  `set_pit_type()`), spread through the cavern.
* You always start in the normal half. Stairs: 3–4 down, 1–2 up.
* A lair of, say, "Undead" or "Fire dragons" or "Hydra" is a monster-nest
  the size of half a level. If the danger feeling jumps, find out what the
  theme is (detect monsters) before you go exploring the cavern side.

### 7.4.7 Gauntlet (`gauntlet_gen`) — DL20+, 0.5%

* Three parts: a left cavern, a thin **labyrinth** strip in the middle
  (`2·1d5+3` tall, `2·1d10+19` wide, built hard and unlit), and a right
  cavern.
* **Up staircases only in the left cavern; down staircases only in the right
  cavern** (2–3 down, 1–3 up).
* The **left cavern and the gauntlet strip are `NO_TELEPORT`**, and the
  strip is also `NO_MAP` (cannot be magic-mapped).
* If you arrived going **down**, you are put in the left cavern (where you
  cannot teleport); if you arrived by going **up** (`create_down_stair`) you
  are put in the right one.
* The labyrinth strip holds `14 + 1d6 + k/2` monsters of a single pit theme.
* Strategy: if you came down, you are standing on (or near) an up
  staircase with connected stairs on. Unless you specifically want to run
  the gauntlet, just take the stairs back up. Never rely on Teleport or
  Teleport Level as an escape while in the left cavern — both fail
  ("Teleportation forbidden!", `effect_handler_TELEPORT_LEVEL`).

### 7.4.8 Hard centre (`hard_centre_gen`) — DL50+, 0.5%

* A **greater vault in the middle of the level** (`vault_chunk()`, either
  "Greater vault" or "Greater vault (new)"), surrounded by four caverns.
* 1–3 down stairs, 1–2 up stairs placed in the caverns.
* Cavern-style monster/object counts plus the vault's own contents.
* This is the most reliable greater vault in the game (the vault is
  guaranteed); it is also a level where the vault's occupants are awake
  and wandering the caverns soon enough. Treat like any greater vault.

### 7.4.9 Arena (special)

`arena_gen()` builds a single-combat arena for the **Paladin** spell
*Single Combat* (clvl 40, `class.txt`). Word of Recall, Teleport Level and
the recall timer are suspended there (`player->upkeep->arena_level` checks
in `effect_handler_RECALL`, `effect_handler_TELEPORT_LEVEL` and
`process_world()`).

---

## 7.5 Rooms

### 7.5.1 How a room type is picked

For every room attempt (`classic_gen` / `modified_chunk`):

1. Roll `key = randint0(100)`.
2. Roll a rarity: start at 0; while below the profile's maximum (2), with
   probability `(50 + depth/2) / unusual` increase it and keep rolling.
   * Classic (`unusual` 200): P(rarity ≥ 1) = 25% at DL0, 35% at DL40,
     50% at DL100; P(rarity 2) = 6.3%, 12.3%, 25%.
   * Modified/moria/lair (`unusual` 300): P(≥1) = 16.7% at DL0, 23.3% at
     DL40, 33.3% at DL100; P(2) = 2.8%, 5.4%, 11.1%.
3. Walk the profile's room list in order and try the **first** room whose
   rarity ≤ the rolled rarity and whose `cutoff` > key and whose minimum
   level ≤ depth (`room_build()` enforces `profile.level`). If it fails to
   fit, try the next.

### 7.5.2 Room lists

Classic (`dungeon_profile.txt`):

| Room | min DL | rarity | cutoff |
|---|---|---|---|
| Greater vault | 35 | 0 | 100 (but see 7.5.4) |
| monster pit | 5 | 2 | 8 |
| monster nest | 5 | 2 | 16 |
| Medium vault | 30 | 2 | 38 |
| Lesser vault | 20 | 2 | 55 |
| large room | 3 | 1 | 15 |
| crossed room | 3 | 1 | 35 |
| circular room | 1 | 1 | 50 |
| overlap room | 1 | 1 | 70 |
| room template (rating 1) | 1 | 1 | 95 |
| room template (rating 2) | 10 | 1 | 99 |
| room template (rating 3) | 30 | 1 | 100 |
| simple room | 1 | 0 | 100 |

Modified and lair use: Greater vault (cutoff 50) and Greater vault (new)
(100) at DL35; **huge room** DL40; **room of chambers** DL10 (rarity 2,
cutoff 4); pit 12; nest 20; Medium vault 30 / (new) 40 at DL30; Lesser
vault 50 / (new) 60 at DL20; **Interesting room** (rarity 1, cutoff 10, any
depth); then large 25, crossed 40, circular 55, overlap 70, templates 95/99/100,
simple 100.

Moria uses the vault/pit/nest/chambers list, Interesting room at rarity 2
(cutoff 100), and **moria room** as its only common room.

So, on a classic level at a rarity-2 roll, keys 0–7 give a pit, 8–15 a
nest, 16–37 a medium vault (DL30+), 38–54 a lesser vault (DL20+). Above
those depths the vault slots fall through to the next eligible entry.

### 7.5.3 Lighting

Ordinary rooms (simple, circular, overlap, crossed, large, pits, nests) are
lit if `depth <= randint1(25)` (`src/gen-room.c`). That is 100% at DL1,
~50% at DL13 and **0% from DL26 down**. Moria rooms use 35 instead of 25
(dark from DL36), rooms of chambers are lit if `randint0(45) > depth`
(dark from DL45), and huge rooms are lit 2/3 of the time at any depth.
From the late 20s on, **assume every room is dark** — this is one reason a
light radius of 2+ and some form of detection matter (the borg refuses
DL10+ with light radius < 2, `borg-prepared.c`).

### 7.5.4 Special rooms

* **Huge room** (`build_huge`, DL40+, modified/lair only): 30–39 × 45–94
  starburst; only as the first room; flat 1 in 20 chance; lit 2/3; often
  broken up with rubble fields.
* **Room of chambers** (`build_room_of_chambers`, DL10+): a big block of
  small interconnected cells, densely packed with monsters — a "mini-nest"
  that you must clear cell by cell.
* **Room templates** (`room_template.txt`, 415 templates: 344 of rating 1,
  51 of rating 2, 20 of rating 3). Symbols include `8` (treasure 80% or
  stairs 20%, with guardian monsters) and `9` (monsters and treasure
  nearby), and random secret doors.
* **Interesting rooms** (`vault.txt` type "Interesting room", 66 of them,
  many with depth windows such as 2–10, 4–14, 20–45, 40–60, 60+).
  Thematic set-pieces; some carry a non-zero rating.

### 7.5.5 Vaults

Vault plans are in `lib/gamedata/vault.txt`:

| Type | Count | Notes |
|---|---|---|
| Lesser vault | 14 | rating 5–12, any depth (but the room profile needs DL20) |
| Lesser vault (new) | 12 | some depth-windowed (10–30, 25–40, 30–50) |
| Medium vault | 14 | rating 5–20; needs DL30 |
| Medium vault (new) | 15 | windows such as 20–60, 30–80, 55–75, 60+ |
| Greater vault | 27 | rating 15–45; needs DL35 |
| Greater vault (new) | 13 | windows 0–45, 35–50, 45–70; some only at 50+, 60+, 80+, 90+ |

A vault is chosen uniformly among the plans of that type whose
`min-depth ≤ depth ≤ max-depth` (`random_vault()`, `src/gen-room.c`;
`max-depth:0` means "no maximum", `parse_vault_max_depth()`).

**Greater-vault frequency.** `help_greater_vault()` allows a GV only as the
first non-staircase room of a level, then passes with probability 1/3 at
DL90+, multiplied by 2/3 for each 10 levels shallower; non-classic profiles
reject a further 2/3. The code's own table of resulting frequencies:

| DL | GV frequency per level |
|---|---|
| 100+ | 18.0% |
| 90–99 | 16.0–18.0% |
| 80–89 | 10.0–11.0% |
| 70–79 | 5.7–6.5% |
| 60–69 | 3.3–3.8% |
| 50–59 | 1.8–2.1% |
| 0–49 | 0.0–1.0% |

Plus the 0.5% hard-centre profile from DL50.

**Vault map symbols** (header of `vault.txt`), which tell you what was
generated where:

| Symbol | Meaning |
|---|---|
| `#` granite, `@` impenetrable (permanent) rock, `%` outer wall | |
| `*` treasure vein, `:` rubble (50% passable), `` ` `` lava | |
| `+` secret door, `^` trap (25%) | |
| `&` treasure (75%) or trap (6.25%) | |
| `1` monster (50%) / object (25%) / trap (6.25%) | |
| `2` monster up to 5 levels OOD | `3` object up to 3 OOD |
| `4` monster up to 3 OOD and/or treasure up to 7 OOD | `5` object up to 7 OOD |
| `6` monster up to 11 OOD | `7` object up to 15 OOD |
| `9` monster up to 9 OOD + treasure up to 7 OOD | `0` monster up to 20 OOD |
| `8` **monster up to 40 OOD and treasure up to 20 OOD** | `~` chest up to 5 OOD |
| `<` / `>` stairs (not on persistent / quest levels) | letters: a monster of that symbol |

Vault monsters are generated **asleep** (`pick_and_place_monster(..., true,
...)` in `build_vault()`), at depth + 0/3/5/9/11/20/40 as above. Vault
squares are `SQUARE_VAULT`; you can **never teleport into a vault**, and
any teleport from inside one lands you outside (`hints.txt`;
`sanitize_player_loc()` also avoids vault squares).

Practical vault rules:
* The good stuff is behind the `8`s, `0`s and `6`s — so are the worst
  monsters. Detect monsters before you dig in.
* Vault walls are often permanent rock: you must find the (secret) door or
  the designed entrance.
* Vault interiors are dark; bring light and detection.
* With `birth_lose_arts` off (default) an artifact you leave behind is not
  lost forever — but a vault is still the best artifact source in the game.

### 7.5.6 Monster pits and nests (`pit.txt`)

Both are built only on levels ≥ DL5 (room profile `level` 5), at most 2 per
level (`pit-max:2`).

* **Pit** (`build_pit`): an outer room with an inner walled room entered by
  **one door**. 16 monster types are drawn at **depth + 10**, sorted by
  power, every other one is kept (8 types), and they are placed in a fixed
  concentric pattern with the strongest in the centre.
* **Nest** (`build_nest`): inner room filled at random from 64 draws at
  **depth + 10**. Width 11 + 2·randint0(4).
* Pit and nest monsters are placed with `sleep = false` (they are **not
  generated asleep**), but they are shut inside the inner room until the
  door is opened.
* Some themes leave items on the floor: `obj-rarity` is the % chance per
  square, items generated at depth + 10, 1/3 of them good.

Theme is chosen by `set_pit_type()`: each theme gets a normal-distributed
"offset" around its average level (σ = 10) and passes a `1 in rarity` roll;
the theme closest to the current depth wins. Themes (room 1 = pit,
2 = nest, 3 = only used for lairs/gauntlets/moria):

| Theme | Room | Rarity | Avg DL | Notes |
|---|---|---|---|---|
| Kobolds | pit | 1 | 15 | |
| Ants | nest | 1 | 15 | |
| Jelly | nest | 1 | 20 | 5% items; many passive but some touch effects |
| Creepy crawlies | nest | 1 | 20 | |
| Minor demons | pit | 1 | 20 | |
| Spellcasters | pit | 1 | 20 | |
| Orc | pit | 1 | 25 | |
| Naga, Thieves, Warriors | pit | 1 | 25 | |
| Eyes, Ogres | pit | 1 | 30 | gaze effects (paralysis!) |
| Believers | pit | 1 | 35 | |
| Dark dwarves, Serpents, Lesser undead | pit/nest | 1 | 40 | |
| Troll | pit | 1 | 45 | great XP for melee |
| Wizards | pit | 1 | 45 | |
| Animals | nest | 1 | 50 | 10% items |
| Archers | pit | 1 | 50 | |
| Vampires | pit | 1 | 55 | |
| Hydra | nest | 1 | 60 | |
| Giant, Golems | pit | 1 | 65 | |
| Undead | nest | 1 | 80 | 20% items |
| Acid / Electric / Fire / Ice / Poison / Multi-hued / Gold dragons | pit | 6 | 85 | |
| Ainur | pit | 1 | 90 | |
| Demons | pit | 1 | 105 | |
| Insects (2/15), Reptiles (20), Cave dwellers (25), Moria dwellers (8/30), Creatures of Earth (8/35), Elemental creatures (40) | 3 | — | — | lair/gauntlet/moria only |

How to use pits: a pit is a fixed formation behind a single door — the
classic tactic is to open the door from a corridor and let the monsters
come to you one at a time, or to blast the room with ball spells/area
effects. Do **not** open a dragon, eye, vampire or undead pit you cannot
dominate.

---

## 7.6 Terrain (`terrain.txt`)

There are no trees or water in Vanilla 4.2 — `vault.txt` explicitly says
`/` (water) and `;` (tree) are "treated as plain floor by Vanilla".

| Feature | Passable | LOS | Notes |
|---|---|---|---|
| open floor | yes | yes | can hold objects and traps |
| closed door | no | no | may be locked (see 7.7); `digging:5` |
| open / broken door | yes | yes | broken doors cannot be closed |
| up / down staircase | yes | yes | permanent |
| shop entrances | yes | yes | town only |
| secret door | no | no | looks like granite |
| pile of rubble | no | no | `digging:1`; easy to clear |
| **pile of passable rubble** | yes | **no** | blocks line of sight but you can walk through (slowly) |
| magma vein / quartz vein | no | no | `digging:2` / `3`; "with treasure" variants hold gold |
| granite wall | no | no | `digging:4` |
| permanent wall | no | no | cannot be dug, Stone-to-Mud'd or destroyed |
| **lava** | yes | yes | `FIERY`, `BRIGHT` (lights itself) |

**Lava** (`player_check_terrain_damage()`, `src/player-util.c`): stepping
into or standing in lava deals **100 + 1d100 fire damage**, adjusted by
your fire resistance (and immunity, `resist-flag:IM_FIRE`), halved again by
Feather Falling, and it damages inventory as fire. The game asks for
confirmation ("The lava will scald you! Really step in?"). Pathing and the
monster AI avoid damaging squares; monsters without fire immunity won't
follow you in.

Digging chances per turn, out of 1600 (`calc_digging_chances()`): rubble
`skill·8`, magma `(skill−10)·4`, quartz `(skill−20)·2`, granite
`(skill−40)·1`. A shovel or pick (General Store) is the cheapest way to
dig out of a dead end, and rubble clearing is trivial for anyone.

---

## 7.7 Doors, locks, secret doors, traps

* Doors placed by generation are 30% open, 10% broken, 60% closed
  (`place_random_door()`); a closed door is **locked 1 time in 4** with a
  lock power of 1d7 (`place_closed_door()`).
* Unlocking chance per try = `max(2, disarm_phys_skill − 4·lock_power)`%
  (`calc_unlocking_chance()`); divided by 10 if you are blind or can't see
  the lock, and again by 10 if confused or hallucinating. A weak-skilled
  character facing a power-7 lock may be stuck at the 2% floor — tunnel
  through the door instead (doors are `digging:5`) or go round.
* **Secret doors** look like granite and are revealed simply by **walking
  next to them** (`docs/dungeon.rst`). Monsters use them freely.
* **Traps** (`trap.txt`, 40 kinds) are revealed when the square is seen
  (`square_reveal_trap()` in `update_one()`); 4.2 has no "search" command.
  Notable traps by minimum depth (`appear:rarity:min-depth:max`):
  * trap door DL2 (falls you one level; 2d8 damage, none with Feather Fall
    but you still fall),
  * teleport rune DL1, siren DL1, summon-foe rune DL3,
  * stat-drain darts (STR/DEX/CON) DL6,
  * mine trap DL15, mind-blasting trap DL20, knife trap DL20, rune of
    necromancy / dragonsong / hellhole DL20, blast trap DL25,
    earthquake trap DL30, **brain-smashing** and **area-blast** traps DL40.
* Classic/modified/moria/lair levels place only `randint1(k)/5` corridor
  traps; caverns, labyrinths and gauntlets place `randint1(k)` — five
  times as many.
* `W` deliberately walks into a trap; `D` disarms. Unknown runes on the
  floor ("strange rune") deserve respect: summoning runes in particular.

---

## 7.8 Stairs, connected stairs and trapdoors

* Normal levels have 3–4 down and 1–2 up staircases; caverns 1–3 down;
  labyrinths are only guaranteed one of each.
* **Connected stairs** (`birth_connect_stairs`, **on** by default): when
  you arrive by stairs, a staircase is created under you **leading back the
  way you came** (`new_player_spot()` in `src/gen-util.c`): going down puts
  you on an **up** staircase; going up puts you on a **down** staircase.
  * Consequence 1: you can always leave a level you just entered by
    stairs, immediately, at the cost of one turn. This is the single most
    important safety feature in the game.
  * Consequence 2 (stair-scumming): alternating `<` and `>` regenerates
    levels at DL d and d−1 endlessly, which is how you "shop" for a good
    level feeling, a particular unique, or a safe arrival.
  * You **always get the first move** on entering a level (`hints.txt`).
* With `birth_force_descend`, up staircases are never created and `<`
  does nothing; recall from a quest level is blocked; Deep Descent and
  Teleport Level always go down.
* Quest levels (DL99 with Sauron alive, DL100 with Morgoth alive) have **no
  down staircases** (`alloc_stairs()`), and a magical down staircase
  appears where the quest monster dies (`build_quest_stairs()` called
  from `quest_check()`, `src/player-quest.c`).
* Trap doors (DL2+) drop you to depth+1 at a random spot — **not** on a
  staircase. A trapdoor is a real hazard in the 30s–40s when your escape
  plan depended on standing on stairs.

### Autoexplore option

This tree includes an **`autoexplore_commands`** interface option
(`list-options.h`, `docs/option.rst`): `<`/`>` path to the nearest known
staircase if you are not on one, and `p` walks to the nearest unexplored
square (only when no monster is in view and you are not confused). Handy
for stair-scumming and for clearing quiet levels quickly; never leave it
running past an unknown corner in dangerous territory.

---

## 7.9 Level feelings — exactly how they work

Feelings are enabled by `birth_feelings` (on by default). The status line
shows `LF:d-o` (`prt_level_feeling()` in `src/ui-display.c`); `^F`
repeats the message (`do_cmd_feeling()`).

### 7.9.1 Danger (monster) feeling — known at once

`calc_mon_feeling()` (`src/generate.c`):

* While the level is generated every placed monster adds
  `level²` to `mon_rating`; an out-of-depth monster adds a further
  `(level − depth) × level²` (`place_new_monster_one()` in
  `src/mon-make.c`). Vaults add their `rating` (5–45) and pits/nests add
  `3 + ave/20` or `size + ave/20` — negligible next to the monsters
  themselves.
* `x = mon_rating / depth`, then:

| x > | message | `LF` first digit |
|---|---|---|
| 7000 | Omens of death haunt this place | 9 |
| 4500 | This place seems murderous | 8 |
| 2500 | This place seems terribly dangerous | 7 |
| 1500 | You feel anxious about this place | 6 |
| 800 | You feel nervous about this place | 5 |
| 400 | This place does not seem too risky | 4 |
| 150 | This place seems reasonably safe | 3 |
| 50 | This seems a tame, sheltered place | 2 |
| else | This seems a quiet, peaceful place | 1 |

Worked example (DL30): ~28 ordinary DL30 monsters give 28 × 900 = 25 200 →
x = 840 → LF:5 "nervous". Add one level-60 unique (out of depth by 30):
3600 + 30 × 3600 = 111 600 more → x ≈ 4560 → **LF:8 "murderous"**. The
danger feeling is dominated by *out-of-depth* monsters, because the OOD
term is cubic. **LF 7+ on a level you are already marginal for is a reason
to take the connected staircase straight back.**

The feeling is computed at generation and never changes as monsters die or
are summoned (`docs/dungeon.rst`).

### 7.9.2 Object (loot) feeling — needs exploration

* Every object placed during generation adds `(value/100)²` to
  `obj_rating` (`place_object()` in `src/gen-util.c`; value from
  `make_object()` including an out-of-depth bonus). Monster-carried items
  do not count; floor items, vault items and pit items do.
* `x = obj_rating / depth` (`calc_obj_feeling()`):

| Condition | feeling | `LF` second digit |
|---|---|---|
| artifact on level and `birth_lose_arts` on | special | `$` "you sense an item of wondrous power!" |
| x > 160 000 | superb treasures | 9 |
| x > 40 000 | excellent | 8 |
| x > 10 000 | very good | 7 |
| x > 2 500 | good | 6 |
| x > 640 (or any artifact, as a floor) | something worthwhile | 5 |
| x > 160 | not much interesting | 4 |
| x > 40 | not many treasures | 3 |
| x > 10 | scraps of junk | 2 |
| else | naught but cobwebs | 1 |

So **an artifact on the floor guarantees at least LF ?-5** (with
`birth_lose_arts` off, the default).

**How much exploring reveals it.** After generation `place_feeling()`
marks `feeling-total` = **100** random passable, non-damaging squares as
hidden "feeling squares". Each one that you **see** for the first time
increments `feeling_squares`; when it reaches `feeling-need` = **10**, the
object feeling is announced and the `?` becomes a digit (`update_one()` in
`src/cave-view.c`, constants in `constants.txt`). Because the 100 squares
are spread uniformly over the floor, you need to have seen roughly a tenth
of the level's open floor — a couple of lit rooms or a stretch of corridor
usually does it. Squares count when *seen* (in view and lit), not when
merely mapped by Magic Mapping.

### 7.9.3 Using feelings ("level-feeling scumming")

* LF danger digit 1–3 with object digit 7–9 or `$`: stay and loot.
* LF danger 7–9: unless you are hunting something specific, leave via the
  connected stairs before you move.
* Object digit 1–3 after the reveal: there is no floor loot worth the
  risk; continue diving.
* With connected stairs, scumming is cheap: `>` then `<` (or the other way)
  produces a fresh level each time. Remember that every level costs game
  time (shops restock) and every arrival is a fresh risk roll.

---

## 7.10 Changing depth: commands and effects

### 7.10.1 Word of Recall (`effect_handler_RECALL`, `process_world()`)

* Activation delay: `15 + randint0(20)` = **15–34 normal-speed turns**.
  Reading it again while active asks to cancel.
* From the dungeon it takes you to town. From town it takes you to your
  **recall depth**, which is normally your **max depth**
  (`player_set_recall_depth()`; at least DL1).
* **Resetting recall depth**: if you read Recall in the dungeon while
  shallower than your max depth (and not using persistent levels), you are
  asked "Set recall depth to current depth?". Answering yes sets **both
  `recall_depth` and `max_depth`** to the current level. This is how you
  retreat to a shallower working depth — and it also makes Deep Descent and
  `birth_force_descend` behave relative to the new max.
* `birth_no_recall` disables it (until you win); with `birth_force_descend`
  recall goes to max_depth + 1 and cannot be read on a quest level.
* Max depth only increases when you actually arrive deeper
  (`on_new_level()`, `src/game-world.c`).
* Persistent levels: recall from town lets you choose any already-generated
  level up to your max.

### 7.10.2 Deep Descent (`effect_handler_DEEP_DESCENT`)

* Scroll: level 10, cost 250, `alloc:20:1 to 100` (`object.txt`).
* Target = `dungeon_get_next_level(max_depth, 5)` — i.e. **five levels below
  your *maximum* depth, not your current depth**. The effect description in
  `src/list-effects.h` says exactly this: "teleports you up to five dungeon
  levels lower than the lowest point you have reached so far". Capped at
  DL127, and it **stops early at the first quest level** in the range
  (`dungeon_get_next_level()` checks every intermediate depth with
  `is_quest()`): reading it with max depth 96 sends you to **DL99, Sauron's
  level**.
* Delay: `3 + 1d4` = **4–7 normal-speed turns**, then "The floor opens
  beneath you!".
* If the target would not be deeper than where you are now (e.g. you are on
  a quest level), the read does nothing ("malevolent presence"); if that
  happens when the timer fires, you get a radius-5 **Destruction** centred
  on you instead.
* Strategic meaning: Deep Descent is a fast escape *only* when you are at or
  near your max depth and can handle max + 5. If you have retreated to
  shallower levels, Deep Descent throws you deeper than you have ever been.
  Use the recall-depth reset (above) to bring max depth back down first if
  you want Deep Descent as a safe "next level" escape.

### 7.10.3 Teleport Level (`effect_handler_TELEPORT_LEVEL`)

* Immediate. 50/50 up or down when both are possible.
* Up only: in town; with `birth_force_descend` it is down only.
* Not down: from a quest level, or at DL127, or (with forced descent) into
  a quest level.
* Fails on `NO_TELEPORT` squares (gauntlet left side) and with the
  `NO_TELEPORT` curse; monsters' teleport-level is resisted by Nexus
  resistance.
* The scroll is level 20, cost 800, `alloc:16:20 to 100` — deliberately
  rare and pricey. Mage (clvl 28) and Rogue (clvl 35) have it as a spell.

### 7.10.4 Stairs vs. recall vs. descent — which to use

| Want | Use |
|---|---|
| leave a level *now* while standing on stairs | stairs (1 turn) |
| leave a level *now*, no stairs, danger high | ?Teleport Level (instant, may go up or down) |
| get to town | ?Word of Recall (15–34 turns: read it early, then survive) |
| go deeper fast from your max depth | ?Deep Descent (4–7 turns, +5 levels) |
| go one level deeper | stairs `>` |

---

## 7.11 Persistent levels (`birth_levels_persist`, experimental, off)

* Levels are kept and reconnected: stairs join the same places
  (`get_join_info()`, staircase rooms with rarity 99 in every profile).
* Labyrinth, gauntlet and hard-centre profiles are disabled.
* Recall from town lets you choose any generated level ≤ max depth.
* No stair-scumming and no feeling-scumming; persistent play is a
  fundamentally slower, more careful game.

---

## 7.12 Quests and winning (`quest.txt`, `src/player-quest.c`)

| Quest | Level | Monster |
|---|---|---|
| Sauron | DL99 | Sauron, the Sorcerer (`number:1`) |
| Morgoth | DL100 | Morgoth, Lord of Darkness (`number:1`) |

* Quest levels are always `classic`, always full size.
* No down stairs are generated on a quest level; the magical staircase is
  built where the quest monster dies.
* While Sauron lives you cannot reach DL100 at all: no stairs, Teleport
  Level and Deep Descent refuse to go below a quest level, and
  `dungeon_get_next_level()` stops at DL99.
* Killing a quest monster sets `level = 0` for that quest; when no quests
  remain, `total_winner` is set: "*** CONGRATULATIONS *** You have won the
  game! You may retire (key is shift-q) when you are ready."
  (`quest_check()`). You can keep playing; you can still die until you
  retire with `Q`.
* Sauron has shape-changed forms in `monster.txt` (Serpent-, Vampire- and
  Wolf-Sauron, all depth 99, 8000 HP, Wolf-Sauron at +30 speed, the others
  +20). Sauron himself: 8000 HP, +20 speed.
* Morgoth: depth 100, **speed 140 (+30)**, **20 000 HP**, AC 180, never
  sleeps (`sleepiness:0`), blows of `HIT:SHATTER:20d10` (`monster.txt`).
  He wanders toward you across DL100 (`docs/dungeon.rst`).
* After winning, Word of Recall works even with `birth_no_recall`.

---

## 7.13 Monster generation pace, out-of-depth rolls and objects

* Every 10 game turns there is a **1 in 500** chance to spawn a new monster
  out of sight (`mon-gen:chance:500`, `process_world()`), at least
  `max_sight + 5` = 25 squares away. At normal speed that is roughly one new
  monster per 500 player turns; lingering on a level slowly refills it.
* **Out-of-depth monsters**: each monster draw has a **1 in 25** chance
  (`mon-gen:ood-chance:25`) of being generated at
  `depth + min(depth/4 + 2, 10)` (`get_mon_num()`, `ood-amount:10`). At DL40
  that is DL50 monsters, 4% of the time — per monster, ~25 times per level.
* **Object level boost**: 1 in 20 (`obj-make:great-obj:20`) object draws get
  a random boost (`get_obj_num()`), and 1 in 20 (`great-ego`) ego rolls are
  promoted. Objects are capped at level 100 (`obj-make:max-depth:100`).
* Unique monsters: one copy at a time; `FORCE_DEPTH` monsters never appear
  above their native depth.

---

## 7.14 Danger zones by depth

Speeds below are relative to normal (+0 = 110 in `monster.txt`). All
depths/HP/speeds are from `monster.txt`. Uniques are marked (U). Depth is
the monster's native level; with out-of-depth rolls (1 in 25, up to +10
levels) and vault/pit bonuses (+5 to +40), expect each threat a few levels
early.

### DL0–5: "the Farmer Maggot's dogs zone"
* **Grip** and **Fang, Farmer Maggot's Dogs** (U, DL2, +10 speed, 25 HP):
  the classic first-hour death for mages. They outrun you; fight them in a
  corridor or leave.
* **cave spider** (DL2, +10, swarm), **giant white louse** (DL3, +10,
  breeder), **poltergeist** (DL3, +20).
* **Sméagol** (U, DL3, +20, 400 HP, invisible thief) — annoying, not lethal.
* **Bullroarer the Hobbit** (U, DL5, +10, 60 HP): first "real" unique; the
  borg treats him as a reason to leave until clvl 15.
* The borg runs from Grip/Fang/small kobolds until clvl 5
  (`borg-flow-kill.c`).

### DL5–15: orc uniques and packs
* **Mughash** (U, DL7), **Lagduf** and **Wormtongue** (U, DL8 — casts frost
  bolts, slows, heals, creates traps), **Brodda** (U, DL9), **Grishnákh**
  and **Orfax** (U, DL10), **Golfimbul** (U, DL12), **Boldor** (U, DL13),
  **Ufthak** (U, DL14).
* **wolf** packs (DL10, +10) and **warg** packs (DL14, +10): fast packs that
  surround you in rooms. Fight in corridors.
* **orc archers** (DL13), **lurker** (DL14, invisible floor ambush).
* **disenchanter eye** (DL5), **moaning spirit** (DL12, invisible, scares),
  **lost soul** (DL7).
* Breeders: worm masses, lice, mice. The borg flees breeder levels before
  clvl 15 (`borg-caution.c`, "Fleeing (breeder level)").

### DL15–25: hounds, invisibility and paralysis
* **Zephyr hounds** start: clear/dark/light hounds DL15, cold/energy/fire
  hounds DL18, air/earth/water hounds DL20. Light and dark hounds **blind**
  you (the borg flees them without blindness or the matching resist).
* **Paralysis**: **carrion crawlers** (DL25, groups), **ghouls** (DL26),
  floating eyes / eye pits (avg DL30). **Free Action** is the gate — the
  borg will not go below DL20 without it (`borg-prepared.c`).
* **Invisible** threats: ghosts, **poltergeists**, **imps** (DL17,
  invisible, fire bolts), **homunculus**. See Invisible/ESP needed — borg
  requires it for DL10+.
* **Uniques**: Ulfast (DL16), Nár (DL17, 450 HP), **Old Man Willow** (DL19,
  1000 HP), Gorbag and Shagrat (DL19), **Bolg** (DL20, +10, 500 HP),
  Lugdush (21), Uglúk (22), **Azog** (DL23, +10, 900 HP),
  Angamaitë/Sangahyando/Ibun/Khîm (DL24).
* Vortices (DL21), wyverns (DL22), banshees (DL24), werewolves (DL20),
  hill giants (DL25), stone trolls (DL25), quylthulgs (summoners, DL20).

### DL25–35: "the 30s wall"
Everything that was optional becomes mandatory here:
* **Basic resists**. Young dragons (DL29–32), fire/frost giants (DL28–30),
  vortices, and the elemental hounds all breathe. The borg requires
  resist fire plus two other base resists from DL21–25 and all four by
  DL26 (`borg-prepared.c`).
* **Nexus hounds** and **vibration hounds** (DL27, stun), **chaos drake /
  ethereal drake / shadow drake** (DL33), **nexus quylthulg** (DL32).
* **Mind flayers** (DL28), **basilisk** (DL28, +10), **grave wights** (DL30),
  **barrow wights** (DL33), **lich** (DL34), **troll priests** (DL34).
* **Uniques**: Ulwarth (26), **Mîm** (DL27, +10, acid), Uldor (28),
  **Beorn** (DL28, both forms 1400 HP; the borg treats him as scary below
  clvl 45), **Draebor** (DL28, +10, summoner/teleporter), **Bert, Bill and
  Tom the Stone Trolls** (DL30, +10, 1100 HP each — a trio), Ulfang (34),
  Lokkak (35).
* **Gravity, impact and inertia hounds** (DL35): gravity blinks you around
  and slows; inertia slows. The borg considers gravity and inertia hounds
  "scary" (leave the level) below clvl 45. Combined with the borg's
  clvl-40 floor for DL34+, this is the "30s wall": until you have the
  level, speed and resists, DL30–40 is where careless characters die.

### DL35–45: mature dragons, dreads, wraiths
* **Mature dragons** (DL39–43, +5), **abyss spider** (DL38), **white wraith**
  (DL38), **time vortex** (DL38, +20), **shimmering vortex** (+30, blinds).
* **Beholder** (DL40), **xaren** (DL40, passes walls), **spirit troll**,
  **troll chieftain**, **master lich** (DL41), **hellhound** packs (DL42,
  +10, fire), **multi-hued hounds** (DL43).
* **Dread** packs (DL44, +10, pass walls, blind/confuse, nether bolts) —
  scary to the borg below clvl 45.
* **Greater basilisk** (DL45), **undead beholder**, **demonic / draconic /
  rotting quylthulgs** (DL45, summoners), **grey wraith** (DL45).
* Elemental uniques Quaker/Vargo/Waldern/Ariel (DL43), **The Queen Ant**
  (DL42), Kavlax (DL41, breathes several elements), Gorlim (DL41).
* Borg gate for DL40–45: resist poison, resist confusion, STR/DEX/CON and
  spell stat ≥ 16.

### DL45–60: speed or die
* **Ancient dragons** (DL50–53, +10). Scary to the borg below clvl 45.
* **Nether hounds**, **plasma hounds** (DL51, +10) and **time hounds**
  (DL51, **+20**): time breath drains stats and XP, and a +20 pack outruns
  anything without serious speed.
* **Dreadmaster** (DL54), **drolem** (DL54 — poison breath; borg flees
  without basic poison resist), **demilich** (DL54), **dracolich** and
  **dracolisk** (DL55 — nether/nexus; borg flees without nether resist).
* **The Balrog of Moria** (U, DL50, +20, 3000 HP), **Harowen** (DL51, +30),
  Itangast (52), Scatha (53), **The Phoenix** (54), **Shelob** (55, 3500 HP),
  **Ar-Pharazôn** (55, +20), **Baphomet** (58), **Fundin** (59),
  Gilim (60), **Saruman** (DL60, 5000 HP).
* Borg gate DL46+: **+5 speed**, a !Healing, 500 HP, Hold Life;
  DL55+: 2 healing potions, resist blindness, telepathy.

### DL60–80: the deep
* **Smaug** (DL62), **The Mouth of Sauron** (DL65, +20, 7000 HP),
  **Tevildo** (66), **Thuringwethil** (67), **Tselakus** (68), Nan (69),
  **Ossë** (72, nether), **Omarax** (73), **Ungoliant** (DL75, 13 000 HP),
  **Glaurung** (DL75), Makar (76), the **Nazgûl** begin (Uvatha DL77,
  Adunaphel 78, Akhorahil 79, Ren 80 …).
* **Chaos hounds** (DL64), **hounds of Tindalos** (DL69, +20),
  **aether hounds** (DL74, +20), **great wyrms** (DL73–82), **dreadlords**
  (DL62), **archlich** (64), **beholder hive-mother** (67), greater
  quylthulgs (DL71, summon greater demons/dragons/undead).
* Borg gate DL61+: **+10 speed**, resist chaos, resist disenchantment.

### DL80–98: endgame approach
* **Ancalagon** (DL85, 10 000 HP), **The Witch-King** (DL85), **Tarrasque**
  (86), **Lungorthin** (88), **Carcharoth** and **Huan** (DL90),
  **Vecna** (DL92), **Gothmog** (DL95, +20, 8000 HP), **greater Balrogs**
  (DL85, +20), **pit fiends** (DL77, +20).
* Greater vaults reach 16–18% per level from DL90.
* Borg gate DL81+: **+20 speed**. The borg also stops at DL82+ to stockpile
  potions of *Healing* until it has 10 (and 30 once it has seen DL97+).

### DL99–100
* Sauron (DL99) and then Morgoth (DL100); see 7.12 and Chapter 8.

---

## 7.15 Quick reference: what the generator tells you at a glance

| You see on arrival | It means | Typical response |
|---|---|---|
| whole map revealed, lit maze | known labyrinth (DL13+, always at ≤25) | fine; plan a route to `>` |
| dark maze, map unknown | unknown labyrinth; deeper ones may be permanent-walled | detect, map, or leave |
| huge irregular dark cave | cavern (DL15+, 5%) | fewer stairs; ranged enemies; loot is +5 levels |
| caverns, can't teleport | gauntlet left side (DL20+) | take the up stairs back |
| half normal, half cave full of one theme | lair (DL20+) | detect the theme first |
| level full of orcs/trolls in rough rooms | moria level (DL10–39) | melee heaven, caster hell |
| LF:7–9 | very dangerous OOD monster(s) | leave unless hunting them |
| LF:x-$ | artifact on the floor (`birth_lose_arts` on) | explore |
| LF:x-5..9 | good to superb floor loot | explore if danger allows |
| "Teleportation forbidden!" | `NO_TELEPORT` square | walk, don't read |

---

## 7.16 Source index for this chapter

* Profiles and selection: `lib/gamedata/dungeon_profile.txt`,
  `src/generate.c` (`choose_profile`, `labyrinth_check`, `cave_generate`,
  `place_feeling`, `calc_obj_feeling`, `calc_mon_feeling`)
* Builders: `src/gen-cave.c` (`classic_gen`, `labyrinth_gen`,
  `labyrinth_chunk`, `cavern_gen`, `cavern_chunk`, `town_gen`,
  `modified_gen`, `moria_gen`, `hard_centre_gen`, `lair_gen`,
  `gauntlet_gen`, `arena_gen`)
* Rooms, vaults, pits: `src/gen-room.c` (`random_vault`,
  `help_greater_vault`, `build_huge`, `build_pit`, `build_nest`,
  `set_pit_type`, `room_build`), `lib/gamedata/vault.txt`,
  `room_template.txt`, `pit.txt`
* Placement: `src/gen-util.c` (`place_object`, `place_closed_door`,
  `place_random_door`, `alloc_stairs`, `new_player_spot`)
* Feelings display: `src/cave-view.c` (`update_one`),
  `src/cmd-cave.c` (`display_feeling`), `src/ui-display.c`
  (`prt_level_feeling`)
* Depth changes: `src/effect-handler-general.c` (`effect_handler_RECALL`,
  `effect_handler_DEEP_DESCENT`, `effect_handler_TELEPORT_LEVEL`),
  `src/player-util.c` (`dungeon_get_next_level`,
  `player_set_recall_depth`, `player_get_recall_depth`,
  `dungeon_change_level`), `src/game-world.c` (`process_world`,
  `on_new_level`, `is_daytime`, `extract_energy`)
* Quests: `lib/gamedata/quest.txt`, `src/player-quest.c` (`quest_check`)
* Terrain/traps: `lib/gamedata/terrain.txt`, `trap.txt`,
  `src/player-util.c` (`player_check_terrain_damage`),
  `src/player-calcs.c` (`calc_unlocking_chance`, `calc_digging_chances`)
* Stores: `lib/gamedata/store.txt`, `src/store.c` (`price_item`,
  `store_update`)
* Constants: `lib/gamedata/constants.txt`
* Borg depth gates quoted here: `src/borg/borg-prepared.c`,
  `src/borg/borg-flow-kill.c`, `src/borg/borg-caution.c`
  (full table in Chapter 8)

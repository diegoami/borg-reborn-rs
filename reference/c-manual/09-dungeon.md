# 9. The Dungeon

How Angband builds the town and each dungeon level, what the different level types
("cave profiles") look like and how they play, how rooms, vaults, pits and nests are
chosen, what the level feeling really measures, and every way of moving between levels
(stairs, trapdoors, Word of Recall, Deep Descent, Teleport Level). All numbers are taken
from the 4.2-series master source (`lib/gamedata/*.txt`, `src/generate.c`,
`src/gen-cave.c`, `src/gen-room.c`, `src/gen-util.c`); where the official manual is
vaguer it is quoted first and then made precise.

**Contents**

1. [Overview and numbers at a glance](#91-overview-and-numbers-at-a-glance)
2. [Depth, level size and the monster/object budget](#92-depth-level-size-and-the-monsterobject-budget)
3. [Out-of-depth rules](#93-out-of-depth-rules-monsters-and-objects)
4. [Cave profiles: how a level type is chosen](#94-cave-profiles-how-a-level-type-is-chosen)
5. [The profiles one by one](#95-the-profiles-one-by-one)
6. [Rooms](#96-rooms)
7. [Vaults](#97-vaults)
8. [Monster pits and nests](#98-monster-pits-and-nests)
9. [Level feelings](#99-level-feelings)
10. [Terrain](#910-terrain)
11. [Doors](#911-doors)
12. [Light and darkness](#912-light-and-darkness)
13. [Traps and trapdoors (level-generation view)](#913-traps-and-trapdoors-level-generation-view)
14. [Stairs, connected stairs and stair-scumming](#914-stairs-connected-stairs-and-stair-scumming)
15. [Word of Recall and the recall depth](#915-word-of-recall-and-the-recall-depth)
16. [Deep Descent](#916-deep-descent)
17. [Teleport Level](#917-teleport-level)
18. [Persistent levels, forced descent and other birth options](#918-persistent-levels-forced-descent-and-other-birth-options)
19. [The town](#919-the-town)
20. [Quests and the bottom of the dungeon](#920-quests-and-the-bottom-of-the-dungeon)
21. [The single-combat arena](#921-the-single-combat-arena)
22. [Monsters that appear while you are on the level](#922-monsters-that-appear-while-you-are-on-the-level)
23. [Quick reference tables](#923-quick-reference-tables)
24. [Things to try (untested)](#things-to-try-untested)

See also: [Chapter 10 (exploration)](10-exploration.md),
[Chapter 13 (escapes)](13-escapes-and-emergencies.md),
[Chapter 8 (monsters)](08-monsters.md), [Chapter 5 (traps and status)](05-elements-and-status.md),
and the official manual's "Exploring the Dungeon" page
(<https://angband.readthedocs.io/en/latest/dungeon.html>, vendored as
[user-manual/dungeon.rst](user-manual/dungeon.rst)).

---

## 9.1 Overview and numbers at a glance

The official manual says each level is "divided into (large) rectangular regions (several
times larger than the screen) by permanent rock" and that "once you leave a level by a
staircase, you will never again find your way back to that region of that level"
([dungeon.rst](user-manual/dungeon.rst)). That is true unless the experimental
`birth_levels_persist` option is on (section 9.18).

FACT (`lib/gamedata/constants.txt`):

| Constant | Value | Meaning |
|---|---|---|
| `world:dungeon-hgt` × `dungeon-wid` | 66 × 198 | maximum level size (grids) |
| `world:town-hgt` × `town-wid` | 22 × 66 | town size |
| `world:max-depth` | 128 | levels 0 (town) to 127 exist; nothing is generated below 127 |
| `world:stair-skip` | 1 | each stair moves one level |
| `world:feeling-total` | 100 | hidden "feeling squares" per level |
| `world:feeling-need` | 10 | feeling squares you must *see* to get the object feeling |
| `mon-gen:level-min` | 14 | base number of monsters on most level types |
| `mon-gen:chance` | 500 | 1-in-500 chance per game "world tick" of a new wandering monster |
| `mon-gen:ood-chance` / `ood-amount` | 25 / 10 | out-of-depth monster boost (section 9.3) |
| `mon-gen:repro-max` | 100 | breeders stop multiplying when 100 breeders are alive on the level |
| `dun-gen:amt-room`, `amt-item`, `amt-gold` | 9, 3, 3 | average floor objects in rooms / anywhere / gold piles |
| `dun-gen:pit-max` | 2 | at most two pits/nests per level |
| `obj-make:great-obj` / `great-ego` | 20 / 20 | 1-in-20 object-level boosts (section 9.3) |
| `player:max-sight` | 20 | view distance; also the Mass Banishment radius |

Depth is shown in feet: dungeon level (DL) *n* = 50×*n* ft. DL99 holds Sauron and DL100
Morgoth (`lib/gamedata/quest.txt`).

---

## 9.2 Depth, level size and the monster/object budget

### Level size

FACT:

* **Classic** levels are always the full 66×198. `src/gen-cave.c:classic_gen()` computes a
  `size_percent` but the source itself notes "This code currently does nothing"; only
  the number of rooms is scaled and room placement is limited by the block grid anyway.
* **Modified, Moria and Lair** levels are scaled. `i = randint1(10) + depth/24`; then
  `i<2`→75%, `i<3`→80%, `i<4`→85%, `i<5`→90%, `i<6`→95%, otherwise 100%, and each
  dimension then gets a further `-5%..+4%` jitter (`modified_gen()`, `moria_gen()`,
  `lair_gen()`). Quest levels are always 100%. So shallow levels are often smaller; from
  DL24 the 75% case disappears, from DL48 the 80% case, and so on.
* **Caverns** are `rand_range(33..49)` × `rand_range(99..148)` (half to three-quarters
  of the maximum in each direction; `cavern_gen()`).
* **Labyrinths** are tiny: height `15 + 2*randint0(depth/10)`, width
  `51 + 2*randint0(depth/10)` (plus a permanent-rock border) — at DL50 at most 23×59
  (`labyrinth_gen()`).
* **Gauntlet** levels are two caverns plus a labyrinth strip; **Hard centre** levels are
  full size (sections 9.5.7–9.5.8).

### The basic monster and object budget

Almost every builder uses the same "general amount" `k = clamp(depth/3, 2, 10)`
(so k=2 on DL1–8, rises by one every three levels, and is 10 from DL30 down).

| Profile | Initial monsters (asleep) | Rubble | Random traps | Floor objects | Notes |
|---|---|---|---|---|---|
| classic / modified / moria | `14 + 1d8 + k` (17–32) | `1d k` in corridors | `1d k / 5` in corridors (0–2) | `N(9,3)` in rooms + `N(3,3)` anywhere + `N(3,3)` gold | `src/gen-cave.c:classic_gen()`, `modified_gen()`, `moria_gen()` |
| labyrinth | `14 + 1d8 + k'` | `1d k'` | `1d k'` | `N(6k',2)` objects, `N(3k',2)` gold, `1d2` good; +`N(3,2)` good if unlit; +`N(2,1)` great if walls are permanent | `k' = 3k × area / (66×198)` |
| cavern | `1d8 + k''` (no base 14) | `1d k''` | `1d k''` | `N(k'',2)` objects **at depth+5**, `N(k''/2,2)` gold, `randint0(k''/4)` good | `k'' = max(4k × area / (66×198), 6)` |
| lair | normal half: `1d4 + k/2`; lair half: `14 + 1d20 + k/2` of one theme | `1d(k/2)` | `1d(k/2)/5` | as classic | `lair_gen()` |
| gauntlet | each cavern `14 + 1d4 + k/2`; gauntlet strip `14 + 1d6 + k/2` of one theme | `1d(k/2)` | `1d(k/2)` | as classic | `gauntlet_gen()` |
| hard centre | `1d8 + k'` + the greater vault's own monsters | `1d k'` | `1d k'` | as cavern (depth+5) | `hard_centre_gen()` |

`N(a,b)` is `Rand_normal(a,b)`. Rooms and vaults add their own monsters, objects and
traps on top of this (sections 9.6–9.8). A level whose generation produces 1024 or more
monster slots is thrown away and regenerated (`src/generate.c:cave_generate()`).

ADVICE: caverns look sparse but are *denser* than classic levels. At DL30 a cavern
typically holds ~20 initial monsters in ~40% of the area of a classic level holding ~28,
i.e. roughly 1.5–2× the monster density, with no room walls to break line of sight and
no corridors to fight in. Their objects are generated five levels deeper, which is why
caverns are still worth a look for a strong character.

### Where monsters start

Initial monsters are placed with `pick_and_place_distant_monster(c, p->grid, 0, ...)` —
a minimum distance of **zero**, so a sleeping monster can be generated right next to
your arrival point. All initial monsters are generated asleep (`sleep = true`) except
pit and nest occupants, which are generated **awake** (section 9.8). Monsters arriving
later in play are placed at least `max_sight + 5 = 25` grids away (section 9.22).

---

## 9.3 Out-of-depth rules (monsters and objects)

### Monsters (`src/mon-make.c:get_mon_num()`)

1. **OOD boost:** with probability 1/25 (`ood-chance`) the generation level is raised by
   `min(level/4 + 2, 10)` (`ood-amount` caps the boost at 10). At DL8 that is +4, at DL20
   +7, at DL32 and deeper +10.
2. **"Pick the deeper of two":** a candidate is drawn from all races with native depth ≤
   the (possibly boosted) level; with 60% probability a second candidate is drawn and the
   deeper is kept, and with 10% a third. So the expected depth of a random monster is
   biased toward the current depth.
3. Races with `FORCE_DEPTH` never appear above their native depth; uniques already alive
   (or dead) are skipped; town monsters never appear in the dungeon (and vice versa);
   `SEASONAL` monsters only on 24–26 December.
4. Groups and escorts can add more monsters around the chosen one.

Rooms deliberately ask for deeper monsters: vault glyphs request depth +2, +3, +5, +9,
+11, +20 or +40; pits and nests use depth+10 (sections 9.7–9.8). Vault monsters still
go through `get_mon_num()` and so can get the 1/25 boost on top.

### Objects (`src/obj-make.c`)

* `get_obj_num()`: 1 in 20 (`great-obj`) the object level becomes
  `1 + level × 100 / randint1(100)` (capped at 100) — usually a modest boost, sometimes
  to the maximum.
* `make_ego_item()`: 1 in 20 (`great-ego`) the ego level is boosted the same way
  (capped at 127).
* "Good" drops use level +10; special artifacts are tried 1/1000 per normal object
  (1/10 for good ones).
* For the level feeling, an uncursed object whose kind's minimum allocation depth is
  deeper than the current level gets +20% of its value per level out of depth
  (`make_object()`).

---

## 9.4 Cave profiles: how a level type is chosen

FACT (`src/generate.c:choose_profile()`, `lib/gamedata/dungeon_profile.txt`), in order:

1. Depth 0 → **town**.
2. A quest level (DL99/DL100 while the quest is open) → **classic**, always.
3. **Labyrinth** if `labyrinth_check(depth)` passes: never above DL13; base chance 2%,
   +1% for each of 3, 5, 7, 11, 13 that divides the depth (DL15: 4%; DL30: 4%; DL20: 3%;
   DL17: 2%).
4. **Moria** with probability 1/40 if 10 ≤ depth < 40.
5. Otherwise a weighted draw among profiles with positive `alloc` whose `min-level` ≤
   depth: classic 90, modified 97, cavern 10 (DL15+), lair 1 (DL20+), gauntlet 1
   (DL20+), hard centre 1 (DL50+).

Resulting approximate probabilities (ignoring the small labyrinth/moria pre-checks):

| Depth | classic | modified | cavern | lair | gauntlet | hard centre |
|---|---|---|---|---|---|---|
| 1–14 | 48.1% | 51.9% | – | – | – | – |
| 15–19 | 45.7% | 49.2% | 5.1% | – | – | – |
| 20–49 | 45.2% | 48.7% | 5.0% | 0.5% | 0.5% | – |
| 50+ | 45.0% | 48.5% | 5.0% | 0.5% | 0.5% | 0.5% |

Then subtract 2–4% for labyrinths (DL13+) and ~2.5% for Moria levels (DL10–39) from
these figures.

With `birth_levels_persist` on, labyrinths, gauntlets and hard-centre levels refuse to
generate (their builders return failure and another profile is drawn).

---

## 9.5 The profiles one by one

### 9.5.1 Classic

`dungeon_profile.txt`: `params:11:50:200:2` — 11×11 blocks (a 6×18 block grid, so at
most ~36 rooms), aim for 50 rooms, "unusual" 200, max rarity 2. Tunnels
`10:30:15:25:50` (10% random turn, 30% change direction, 15% chance to just stop, 25% door
at room entrances, 50% door at junctions). 3 magma streamers (1/90 treasure) and 2 quartz
streamers (1/40 treasure). 3–4 down staircases and 1–2 up staircases, at least
`min(w,h)/4` apart.

Room choice (`classic_gen()`): for each block a random `key` (0–99) and a rarity are
rolled; rarity goes up by one with probability `(50 + depth/2)/200` per step, up to 2.
Then the first room type in the profile list with `rarity ≤ rolled rarity` and
`cutoff > key` whose builder succeeds is placed. In classic: Greater vault (rarity 0 but
its own depth gate, section 9.7), rarity 2 rooms (monster pit, nest, medium vault, lesser
vault), rarity 1 rooms (large, crossed, circular, overlap, room templates), rarity 0
simple rooms. Rooms are connected by classic random-walk tunnels; at least two rooms or
the level is regenerated.

How it plays: lots of rectangular rooms, long winding corridors with doors, a few dead
ends; the standard "corridor fighting" level.

### 9.5.2 Modified (the most common profile)

`params:1:50:300:2`, `alloc:97`. Block size 1 means rooms can be packed anywhere; rooms
are added until the floor area reaches `height×width/7` and at least two rooms exist
(`modified_chunk()`). The room list adds Greater vault (new), **huge room** (DL40+, only as
the first room, flat 1/20 chance, usually lit), **room of chambers** (DL10+), **Interesting
rooms** (from `vault.txt`), and both old and new lesser/medium vaults. Size 75–100%
(section 9.2). Monster restrictions are cleared before population.

How it plays: more varied, denser room layouts, more themed "interesting rooms"; similar
danger to classic for the same depth.

### 9.5.3 Moria (DL10–39, 1/40)

`params:1:50:250:2`; tunnels with only 30% junction doors. Rooms are mostly irregular
**moria rooms** (lit if `depth ≤ randint1(35)`), plus vaults, chambers, interesting rooms
and pits. The key feature (`moria_gen()`): initial monsters are restricted with
`mon_restrict("Moria dwellers", ...)` = monster bases **orc, ogre, troll, giant**
(`lib/gamedata/pit.txt`), uniques allowed.

How it plays: an orc/troll level. Great XP and drops for a melee character with slay
orc/troll, very dangerous for a low-HP caster (packs of cave orcs, uruks, trolls; at the
deeper end stone/ice/water trolls, ogre chieftains, giants). Rooms (vaults, pits,
chambers) still choose their own monsters, so not *everything* is an orc.

### 9.5.4 Labyrinth (DL13+, 2–4%)

`src/gen-cave.c:labyrinth_gen()`:

* A perfect maze of 1-wide corridors built with randomized Kruskal, one closed door per
  100 grids.
* **Lit** if `randint0(depth) < 25` or on a coin flip. **Known** (fully mapped on
  arrival, via `wiz_light`) if lit *and* a second `randint0(depth) < 25` passes.
  **Soft** (diggable granite) walls if `randint0(depth) < 35` or 2/3; otherwise the walls
  are *permanent*.
* Unlit labyrinths get `N(3,2)` extra *good* items; permanent-walled ones get `N(2,1)`
  extra *great* items (`labyrinth_chunk()`).
* Only one up and one down staircase (if the maze has none already).
* Never generated with persistent levels.

| Depth | P(lit) | P(known) | P(soft walls) |
|---|---|---|---|
| 15 | 100% | 100% | 100% |
| 30 | 92% | 76% | 98% |
| 50 | 75% | 38% | 80% |
| 80 | 66% | 21% | 63% |

How it plays: you arrive somewhere in a small maze with the full monster budget packed
into it. Unknown + unlit + permanent walls is the worst case: no digging, no line of sight
beyond a few grids, invisible corners. It is also where the free *great* items are.
ADVICE: in a known lit labyrinth read the map and decide immediately; in an unknown dark
one with permanent walls and a bad danger feeling, leave unless strong — there is
nowhere to retreat to and Stone to Mud does nothing to permanent rock.

### 9.5.5 Cavern (DL15+, ~5%)

`cavern_gen()`: cellular-automaton caves (25–40% initial floor density, 3–6 smoothing
passes; regenerated until floor ≥ area/13), small regions removed and the rest joined.
1–3 down and 1–2 up stairs. No rooms, no doors, **no lit squares** (nothing in the
cavern builder sets `SQUARE_GLOW`). Monster and object counts in section 9.2; objects are
generated 5 levels deeper.

How it plays: dark, irregular, wide-open, monster-dense; groups can surround you. Light
radius and detection matter a lot. Archers and breathers get long lines of fire across
open floor.

### 9.5.6 Lair (DL20+, 0.5%)

`lair_gen()`: the level is split into a *modified* half (with few monsters: `1d4 + k/2`)
and a *cavern* half (unlit) packed with `14 + 1d20 + k/2` sleeping monsters of a single
theme chosen by `set_pit_type(depth, 0)` — any entry in `pit.txt`, including the
"room 3" themes (insects, reptiles, cave dwellers, Moria dwellers, creatures of earth,
elementals). You always start in the normal half.

How it plays: a quiet normal half and a nest-sized themed horde in the dark cave half.
If the theme is wrong for you (e.g. dragons, undead or demons at their native depth),
finish the normal half and leave.

### 9.5.7 Gauntlet (DL20+, 0.5%)

`gauntlet_gen()`:

* Left cavern, a narrow labyrinth strip ("the gauntlet", unlit, **permanent walls**, 5–13
  rows × 21–39 columns), right cavern; the area between them is permanent rock.
* The **left cavern and the gauntlet are `NO_TELEPORT`**; the gauntlet is also
  `NO_MAP` (magic mapping does nothing there).
* **Up staircases only in the left cavern (1–3), down staircases only in the right
  cavern (2–3).**
* You arrive in the *right* cavern only if you came up a staircase (the game sets its
  "create down stair" marker whenever you go up, whatever the connected-stairs option);
  otherwise — going down, recall, trapdoor, Teleport Level,
  Deep Descent — you arrive in the **left** cavern.
* Each cavern gets `14 + 1d4 + k/2` monsters; the gauntlet strip `14 + 1d6 + k/2`
  monsters of one pit theme. More random traps than usual (`1d(k/2)`, not divided by 5).

How it plays: to go *down* you must walk through a themed monster-packed maze with no
teleport. To go *up* the stairs are next to you. NO_TELEPORT blocks Teleportation,
Teleport Level and any teleport longer than 10 grids, but **Phase Door (10) still works**
on a no-teleport square ([Chapter 13](13-escapes-and-emergencies.md#131-teleport-mechanics)), and
Word of Recall and Deep Descent do not check the flag.

### 9.5.8 Hard centre (DL50+, 0.5%)

`hard_centre_gen()`: a **greater vault** in the middle (old or new style, 50/50), rotated
to be wider than tall, with 1–4 entrances pierced, surrounded by four caverns joined
together. Stairs: 1–3 down, 1–2 up. Monsters `1d8 + k'` in the caverns plus everything
in the vault; cavern-style objects at depth+5. Never with persistent levels.

How it plays: a guaranteed greater vault at DL50+. Dangerous (greater vaults contain
+11/+20/+40 level monsters, section 9.7) but the richest level type in the game.

### 9.5.9 Town

See section 9.19.

---

## 9.6 Rooms

Room types come from `src/list-rooms.h` and are configured per profile in
`dungeon_profile.txt` as `room:name:rating:height:width:min-level:pit:rarity:cutoff`.
Builders live in `src/gen-room.c`.

| Room | Min DL | Where | Contents / notes |
|---|---|---|---|
| simple room | 1 | all room profiles | plain rectangle; 1/20 pillared, else 1/50 ragged-edged; lit if `depth ≤ randint1(25)` |
| large room | 3 | classic, modified, lair | outer room with an **inner room** of 5 kinds: (1) inner room with one monster (depth+2); (2) inner room with a locked inner-inner room holding 3–5 monsters (depth+2), an object (80%) or a staircase (20%) and 3–5 traps; (3) pillars, sometimes with two small secret-door rooms each holding a monster and an object; (4) a checkerboard maze with monsters, traps and 3 objects; (5) four compartments |
| crossed room | 3 | classic, modified, lair | plus-shaped; variants: plain, solid middle pillar, **inner treasure vault** (secret door, 1 object, 3–4 monsters at depth+2, 2–4 traps), or pinched/plus/pillar centre |
| circular room | 1 | classic, modified, lair | round; large ones (radius > 4) may get a middle chamber with a door, 0–1 objects and 0–2 monsters at depth+1 |
| overlap room | 1 | classic, modified, lair | two overlapping rectangles |
| room template | 1 / 10 / 30 | classic, modified, lair | one of 415 hand-drawn templates in `room_template.txt` (344 rating-1, 51 rating-2 at DL10+, 20 rating-3 at DL30+); 244 have `FEW_ENTRANCES`; doors from the template are **secret**; some templates place a treasure of a given tval |
| Interesting room | 0 | modified, moria, lair | 66 hand-drawn rooms in `vault.txt` with type "Interesting room" — e.g. Ant nest (DL3–50), Kobold den (4–14), Yeek Warren (4–14), Zoo (15–40), Spider Nest (10–20), Crevasse (20–70), Giant's Armoury (20–45), Mad Alchemist's Lab (30–55), Deep Crypt (60+), Little eruption (lava) |
| moria room | 1 | moria | irregular cave-like room; lit if `depth ≤ randint1(35)` |
| room of chambers | 10 | modified, moria, lair | a big block of small interconnected chambers filled with themed monsters (`get_chamber_monsters()`); lit if `randint0(45) > depth`; adds 10 to the monster rating |
| huge room | 40 | modified, lair | 30–39 × 45–94, only as the first room, 1/20 attempts; lit 2/3 of the time |
| monster pit / nest | 5 | all room profiles | section 9.8 |
| lesser / medium / greater vault (and "new") | 20 / 30 / 35 | all room profiles | section 9.7 |
| staircase room | – | persistent levels only | small room joining stairs between persistent levels |

Room lighting rule (`build_simple()`, `build_circular()`, `build_overlap()`,
`build_crossed()`, `build_large()`, `build_room_template()`): **lit iff
`depth ≤ randint1(25)`**, i.e. P(lit) = (26 − depth)/25: 100% on DL1, 64% on DL10, 24% on
DL20, 4% on DL25 and **0% from DL26 down**. Corridors are never lit.

---

## 9.7 Vaults

`lib/gamedata/vault.txt` holds hand-drawn vaults of types "Lesser vault" (14), "Lesser
vault (new)" (12), "Medium vault" (14), "Medium vault (new)" (15), "Greater vault" (27),
"Greater vault (new)" (13) plus the 66 "Interesting rooms". Each entry has a `rating`
(added to the monster rating), optional min/max depth, and a map.

Minimum depths from `dungeon_profile.txt`: lesser vault DL20, medium vault DL30, greater
vault DL35 (rarity-2 rooms; the "(new)" variants exist only in the modified, moria and
lair profiles).

**Greater vaults** (`src/gen-room.c:help_greater_vault()`) are only tried as the first
non-staircase room, and then only pass a depth check: chance 1/3 at DL90+, ×2/3 per 10
levels shallower (2/9 at DL80–89, 4/27 at DL70–79...). Non-classic profiles reject a
further 2/3. The source comment gives the resulting frequencies: ~18% of levels at DL100+,
16–18% at DL90–99, 10–11% at DL80–89, 5.7–6.5% at DL70–79, 3.3–3.8% at DL60–69,
1.8–2.1% at DL50–59, 0–1% above DL50. The hard-centre profile adds a guaranteed one at
DL50+ (0.5% of levels).

### Vault map glyphs (`src/gen-room.c:build_vault()`)

| Glyph | Meaning |
|---|---|
| `#` granite (inner wall) / `@` **permanent** wall / `%` outer wall | |
| `+` | **secret door** |
| `^` | trap (1 in 4) |
| `&` | object at depth (75%), otherwise a trap 1 in 4 |
| `*` | magma or quartz vein **with treasure** |
| `:` | rubble (passable or not, 50/50) |
| `<` `>` | staircases (none with persistent levels; `>` becomes `<` on quest levels and DL127; no `<` with forced descent) |
| `` ` `` | lava |
| `1` | monster at depth (and sometimes an object or trap) |
| `2` | monster at **depth+5** |
| `3` / `5` / `7` | object at depth+3 / +7 / +15 |
| `4` | monster at depth+3 and object at depth+7 |
| `6` | monster at **depth+11** |
| `9` | monster at depth+9 and **good** object at depth+7 |
| `0` | monster at **depth+20** |
| `8` | monster at **depth+40** and **great** object at depth+20 |
| `~` | object at depth+5 |
| `$` | gold |
| `]` `\|` `=` `"` `!` `?` `_` `-` `,` | armour / weapon / ring / amulet / potion / scroll / staff / wand-rod / food at depth+3 (armour and weapons are "good") |

All vault grids are flagged `SQUARE_VAULT`: random teleports avoid landing in them
unless there is no other choice (`effect_handler_TELEPORT()`), and the start location is
never in a vault. Vault monsters are generated asleep.

Danger by vault class (ADVICE, from the glyph rules):

* **Lesser vaults** (DL20+): mostly `1`–`4` glyphs; monsters up to depth+5; a solid
  source of early ego items. Usually safe for a character comfortable at the depth.
* **Medium vaults** (DL30+): more `6`/`9` glyphs; expect a few monsters 9–11 levels out
  of depth; mazes and secret doors.
* **Greater vaults** (DL35+, realistically DL50+): `0` and `8` glyphs place monsters 20
  and 40 levels deeper — at DL50 that means DL70 and DL90 monsters, i.e. great wyrms,
  greater Balrogs, Dreads packs. The `8` glyph is paired with a *great* object at
  depth+20. Approach only with detection, an escape that works inside (vault grids are
  avoided by teleport so you land *outside*), and a clear plan. Many are walled with
  permanent rock (`@`), so the entrances are fixed.

---

## 9.8 Monster pits and nests

`src/gen-room.c:build_pit()` / `build_nest()`, themes in `lib/gamedata/pit.txt`.

Both are an 11×(11..17) room with an inner room whose single entrance is a **closed
door**; the outer room is **never lit** (`light = false`). The theme is chosen by
`set_pit_type()`: for each eligible theme, `offset = Rand_normal(ave, 10)` and the theme
whose offset is closest to the depth wins, subject to a `one_in_(rarity)` test. Then 64
(nest) or 16 (pit) candidate monsters are drawn with `get_mon_num(depth + 10)` restricted
to the theme. Uniques are never placed in pits or nests.

* **Pit** (`room:1` themes): 16 candidates sorted by level, the even-numbered ones placed
  in a pattern with the **strongest at the centre** (the classic `#11000000011#` layout).
  No objects except where `obj-rarity` says so.
* **Nest** (`room:2` themes): the whole inner room filled with random picks from the 64
  candidates; each square has `obj-rarity`% chance of an object at depth+10 (good 1/3 of
  the time).

**All pit and nest monsters are generated awake** (`place_new_monster(..., sleep=false,
...)`). They are boxed in only by the inner door; anything that opens or bashes doors
comes out.

| Theme | Type | Most common around DL | Rarity |
|---|---|---|---|
| Kobolds, Ants, Insects | pit / nest | 15 | 1 (insects 2, lairs only) |
| Jelly, Creepy crawlies, Minor demons, Spellcasters, Reptiles | nest / nest / pit / pit / lair | 20 | 1 |
| Orc, Naga, Thieves, Warriors, Cave dwellers | pit | 25 | 1 |
| Eyes, Ogres, Moria dwellers | pit / pit / lair | 30 | 1 (Moria dwellers 8) |
| Believers, Creatures of Earth | pit / lair | 35 | 1 (8) |
| Dark dwarves, Serpents, Lesser undead, Elemental creatures | pit / nest / nest / lair | 40 | 1 |
| Troll, Wizards | pit | 45 | 1 |
| Animals | nest | 50 | 1 |
| Vampires | pit | 55 | 1 |
| Hydra | nest | 60 | 1 |
| Giant, Golems | pit | 65 | 1 |
| Undead | nest | 80 | 1 |
| Dragons (acid, electric, fire, ice, poison, multi-hued, gold) | pit | 85 | 6 each |
| Ainur | pit | 90 | 1 |
| Demons (major, no Horned Reaper) | pit | 105 | 1 |

("Room 3" themes — insects, reptiles, cave dwellers, Moria dwellers, creatures of earth,
elemental creatures — are used only for lairs, gauntlets and themed areas, never for pits
or nests.)

ADVICE: a pit or nest is an XP farm *if* you can fight it at the door one monster at a
time (orc and troll pits for warriors; jelly nests for anyone with a ranged attack and
patience). Undead, dragon, demon, hydra and vampire pits at their native depth are
level-leaving events for most characters. Because the occupants are awake, opening the
door is a commitment — check the theme first with detection (`Detect Monsters` /
`Reveal Monsters`, or monster ESP).

---

## 9.9 Level feelings

The official manual: "This feeling depends only on the monsters present in the dungeon
when you first enter it ... Once you have explored a certain amount of the dungeon you
will also get a feeling about how good are the objects" and "an artifact will guarantee a
feeling of 5 or better" ([dungeon.rst](user-manual/dungeon.rst)). Feelings require the
birth option `birth_feelings` (default on; `src/list-options.h`). With it off you get no
feeling at all ("cold-hearted characters", `src/cmd-cave.c:display_feeling()`).

### How it is computed (`src/generate.c`)

Stored as `feeling = obj_feeling + mon_feeling` once, at generation; never updated.

**Monster rating** (`mon_rating`): every monster placed during generation adds
`level²`; an out-of-depth monster adds a further `(level − depth) × level²`
(`src/mon-make.c`); vaults add their `rating`, pits/nests a few points, rooms of chambers
10. Then `x = mon_rating / depth`:

| x > | 7000 | 4500 | 2500 | 1500 | 800 | 400 | 150 | 50 | else |
|---|---|---|---|---|---|---|---|---|---|
| status bar `LF:` first digit | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 |
| message | Omens of death | murderous | terribly dangerous | anxious | nervous | not too risky | reasonably safe | tame, sheltered | quiet, peaceful |

**Object rating** (`obj_rating`): every floor object placed at generation adds
`(value/100)²` with value capped at 2,500,000 (`src/gen-util.c:place_object()`), where
value is `object_value_real()` plus the +20%-per-level out-of-depth bonus. Monster
inventories do not count. Then `x = obj_rating / depth`:

| x > | 160000 | 40000 | 10000 | 2500 | 640 | 160 | 40 | 10 | else |
|---|---|---|---|---|---|---|---|---|---|
| `LF:` second digit | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 |
| message | superb | excellent | very good | good | something worthwhile | not much interesting | not many treasures | scraps of junk | naught but cobwebs |

Special cases: with `birth_lose_arts` on, any artifact on the floor gives `$` ("you sense
an item of wondrous power!"); otherwise an artifact raises the feeling to at least 5.
Town has no feeling ("Looks like a typical town.").

### Worked examples

* DL30, 28 ordinary monsters of level ~28: `28 × 784 / 30 ≈ 730` → LF 4–5. Typical.
* Add one level-50 monster on DL30: `2500 + 20 × 2500 = 52,500`, `/30 = 1750` on its
  own → the feeling jumps to at least LF6 ("anxious"). **A single deep monster dominates
  the danger feeling.** Conversely a pack of 20 level-30 hounds adds `18,000/30 = 600`.
* A single floor item worth 10,000 gold on DL20: `(100)² / 20 = 500` → object LF4; worth
  40,000: `160,000/20 = 8000` → LF6.

### When you learn it

* The **danger** part is shown on arrival (`src/game-world.c:on_new_level()` →
  `display_feeling(false)`), always.
* The **loot** part: 100 hidden `SQUARE_FEEL` grids are scattered over passable,
  non-damaging floor (`place_feeling()`). Each time one of them goes from unseen to
  *seen* (in view and lit — `src/cave-view.c:update_one()`), the count increases; at 10
  the object feeling is announced and `LF:` shows the digit instead of `?`. Magic
  mapping does not count (it does not "see" floors); being blind does not count;
  **Enlightenment/Clairvoyance** helps indirectly because it perma-lights the whole level
  so every floor you look at counts.
* Rule of thumb: you get the loot feeling after seeing roughly 10% of the level's floor.
* `^F` repeats the feeling.

ADVICE: treat the danger digit relative to depth and your power: LF 1–3 = normal, 4–5 =
something notable (vault, pit, unique or OOD monster), 6+ = at least one monster far
beyond the depth or a big vault/pit. Treat the loot digit as "is it worth exploring":
LF loot 1–3 means leave as soon as convenient unless you want XP; 6+ means there is
something worth a detour; `$` means an artifact will be lost if you leave.

---

## 9.10 Terrain

All features in `lib/gamedata/terrain.txt` (there are **no trees or water** in this
version; vault `;` and `/` glyphs are commented out in `build_vault()`):

| Feature | Passable | Blocks LOS / projections | Dig difficulty (`digging`) | Notes |
|---|---|---|---|---|
| open floor | yes | no | – | can hold objects and traps |
| closed door | no | yes | 5 | may be locked (section 9.11) |
| open door / broken door | yes | no | – | broken doors cannot be closed |
| secret door | no | yes | 5 | looks like granite; found automatically when you stand next to it with light |
| up / down staircase | yes | no | permanent | `INTERESTING`, stops running |
| shop entrances 1–8 | yes | no | permanent | town only |
| pile of rubble | no | yes | 1 | tunnel through in a few turns |
| pile of **passable** rubble | yes | yes (and shrinks ball radius) | 1 | you and monsters walk through; still blocks sight and bolts |
| magma vein | no | yes | 2 | 1/90 of streamer grids contain treasure (classic profile) |
| quartz vein | no | yes | 3 | 1/40 contain treasure |
| magma / quartz **with treasure** | no | yes | 2 / 3 | `INTERESTING`; dig for gold |
| granite | no | yes | 4 | |
| permanent wall | no | yes | impossible | level edges, vault `@`, gauntlet, hard labyrinths |
| lava | yes | no | – | `FIERY`, `BRIGHT` (always lit); damages you each turn unless fire-immune, and burns inventory; never used by pathfinding |

Digging chances per turn (`src/player-calcs.c:calc_digging_chances()`,
`src/cmd-cave.c`): rubble `8×skill`, magma `4×(skill−10)`, quartz `2×(skill−20)`,
granite `skill−40`, all out of 1600. So with digging skill 40 rubble falls in ~5 turns on
average and granite is impossible; you need skill > 40 for granite. Rubble generated by
the level builders is 50/50 impassable or passable (`place_rubble()`).

---

## 9.11 Doors

FACT:

* Corridors and room entrances get doors (25% at room entrances, 50% at junctions in
  classic). `place_random_door()`: 30% open, 10% broken, 60% closed; any closed door is
  **locked 1/4 of the time** with lock power `1d7` (`place_closed_door()`).
* Room templates and vaults use **secret doors**. A secret door becomes a normal closed
  door (which may then be locked) when you are adjacent with light and not
  blind/confused/hallucinating (`src/player-util.c:search()`, called after each move and
  on arrival).
* Unlocking chance = `max(2, disarm_phys_skill − 4×power)`%, with the skill divided by
  10 when you can't see the lock and again when confused or hallucinating
  (`calc_unlocking_chance()`).
* You can lock a closed door yourself with the disarm command (`D`) on it: lock power
  `m_bonus(7, depth)` (`do_cmd_lock_door()`).
* Monsters: an unlocked closed door is always opened (`OPEN_DOOR`) or bashed
  (`BASH_DOOR`) in one turn. A locked door is passed only if `randint0(hp/10) > power`, so
  a monster with fewer than ~80 hp is badly delayed by a power-7 lock while a big monster
  goes through quickly (`src/mon-move.c`). Monsters without either flag cannot pass
  closed doors at all; `PASS_WALL`/`KILL_WALL` monsters ignore them.

ADVICE: doors are cheap defensive terrain — close the door behind you when fleeing slow
non-door-using monsters (many animals, jellies, hounds are blocked entirely), and lock
it against weak door-openers. See [Chapter 10](10-exploration.md#1013-doors-as-defensive-tools).

---

## 9.12 Light and darkness

FACT:

* Room lighting: P(lit) = (26 − depth)/25 for ordinary rooms, so **no ordinary room is
  lit below DL25** (section 9.6). Moria rooms use 35 instead of 25, chambers
  `randint0(45) > depth` (lit chambers until about DL44), huge rooms 2/3, labyrinths as in
  section 9.5.4. Caverns, pits, nests and corridors are never lit. Lava is always bright.
* The town is fully lit by day and only shop entrances/lava are lit at night
  (`cave_illuminate()`).
* Your own light: Wooden Torch intensity 2 (`LIGHT_2`, lights only the adjacent grids; burns out and is destroyed when fuel
  hits 0), Lantern intensity 3 (`LIGHT_3`, radius 2, refuelled with flasks) per `object.txt`. Grids at distance
  `< intensity` are lit (`src/cave-view.c:update_view_one()`); light
  modifiers on artifacts/egos add to this. A light with 0 fuel gives no light
  (`src/player-calcs.c:calc_light()`). Torches hold 5000 turns of fuel, lanterns 15000
  (`constants.txt`). Necromancers (`UNLIGHT`) carrying light ≤ 1 instead
  see grids at distance `< 2 + clvl/6 − light` (`src/cave-view.c`).
* Lighting a room (`LIGHT_AREA`: Light Room / Call Light / scroll, staff or rod of
  Light/Illumination) perma-lights the room you stand in; Clairvoyance and the Potion of
  Enlightenment perma-light the **whole level** and map it (`wiz_light()`).
* Without light you cannot read scrolls or cast spells, search for secret doors, or see
  floor traps (see [Chapter 13](13-escapes-and-emergencies.md#132-what-still-works-under-each-status-effect)).

---

## 9.13 Traps and trapdoors (level-generation view)

Full trap list in [Chapter 5](05-elements-and-status.md) and
[A10](appendices/A10-curses-traps-timed-effects.md). Generation facts:

* Random corridor traps: `1d k / 5` on classic, modified, moria and lair levels (0–2!),
  `1d k` on labyrinths, caverns, gauntlets and hard-centre levels. Rooms and vaults add
  their own (`vault_traps()`, vault `^`).
* Each trap has a `visibility` power; you notice it on sight only if your searching skill
  ≥ that power (`src/trap.c:square_reveal_trap()`); trap detection reveals all.
* **Trapdoors** (`trap door`, from DL2) drop you to the next level after 2d8 damage;
  Feather Fall avoids the damage but **you still fall** (`TRF_DOWN` is applied regardless
  of the save in `hit_trap()`). No trapdoors on quest levels, on DL127, with persistent
  levels or in the single-combat arena (`pick_trap()`, `place_trap()`).
* A trapdoor fall does not create a connected staircase under you.

---

## 9.14 Stairs, connected stairs and stair-scumming

### Stair counts

`handle_level_stairs()` / `alloc_stairs()`:

| Profile | Down | Up |
|---|---|---|
| classic, modified, moria, lair | 3–4 | 1–2 |
| cavern, hard centre | 1–3 | 1–2 |
| labyrinth | 1 (if none generated) | 1 |
| gauntlet | 2–3 (right cavern only) | 1–3 (left cavern only) |
| town | 1 | 0 |

Plus stairs from rooms (the large-room treasure chamber has a 20% staircase) and vaults.
Stairs are placed in enclosed spots ("next to walls"), at least `min(w,h)/4` apart from
stairs of the same kind. On a quest level and on DL127 no down stairs are made and the
down count is added to the up count; with forced descent no up stairs are made.

### Taking stairs

FACT (`src/cmd-cave.c:do_cmd_go_up()` / `do_cmd_go_down()`): one turn; you move one
level (`stair-skip` 1); a new level is generated. Monsters do not follow you. With the
`autoexplore_commands` option on, pressing `<`/`>` when not on a staircase pathfinds to
the nearest known one instead ([Chapter 10](10-exploration.md#104-autoexplore-and-pathfinding-commands)).

### Connected stairs (`birth_connect_stairs`, default on)

`src/gen-util.c:new_player_spot()`: when you arrive by **going down** a staircase, the
new level gets an **up** staircase under you; when you arrive by **going up**, a
**down** staircase. Arriving by any other means (recall, trapdoor, Teleport Level, Deep
Descent) gives no staircase. With forced descent the up staircase is not created.

Consequences:

* Going down always leaves an immediate retreat (`<` under your feet), never an
  immediate further descent.
* "Stair-scumming": alternate `<` and `>` on the arrival staircase. Each press makes a
  brand-new level; you bounce between depth *d* and *d*±1 and can re-roll levels until
  the danger feeling, layout or visible loot suits you. The cost is one game turn per
  level plus the risk of arriving next to something awake.

ADVICE on stair-scumming (expanded in [Chapter 10](10-exploration.md#1011-stair-scumming-and-its-tradeoffs)):
it is legitimate and safe-ish, and the single most effective way to skip bad levels
(labyrinths, gauntlets, LF 7+ danger). It is a poor way to *dive* (you need to find a
down staircase on each level to make net progress) and it forfeits the loot feeling and
the XP of exploring.

---

## 9.15 Word of Recall and the recall depth

FACT (`src/effect-handler-general.c:effect_handler_RECALL()`,
`src/game-world.c:process_world()`):

* Activation sets a timer of `15 + randint0(20)` = **15–34 world ticks**. A world tick is
  10 game turns, i.e. one action at normal speed; at +10 speed you get two actions per
  tick, at +20 three. When it reaches 0: from the dungeon you go to the town; from the
  town you go to the recall depth.
* Using recall again while it is active asks whether to cancel it.
* **Recall depth** = `max_depth` normally. Whenever you arrive on a level deeper than
  `max_depth`, both `max_depth` and `recall_depth` are set to that depth
  (`on_new_level()`).
* Reading recall in the dungeon at a depth *shallower* than `max_depth` (non-persistent
  game) asks **"Set recall depth to current depth?"** — answering yes sets
  `recall_depth = max_depth = current depth`. This permanently lowers your max depth,
  which also lowers where Deep Descent sends you (section 9.16).
* With persistent levels, recalling from town asks which previously visited level to go
  to.
* A character who has never left town recalls to DL1.
* Recall does nothing with `birth_no_recall` (unless you have won), in the arena, or on
  a quest level with forced descent. With forced descent, recall from town takes you to
  `max_depth + 1`.
* Recall is **not** blocked by no-teleport squares or the anti-teleportation curse
  (it never checks them).

Sources: Scroll of Word of Recall (always stocked in the Alchemist,
`lib/gamedata/store.txt`), Rod of Recall (DL30+), Priest/Paladin spell (clvl 25), some
artifact activations.

---

## 9.16 Deep Descent

FACT (`effect_handler_DEEP_DESCENT()` and the countdown in `process_world()`):

* Target = `dungeon_get_next_level(max_depth, 5)` — **five levels below your maximum
  depth**, not below your current depth, stopping early at a quest level. (With
  `stair-skip` 1 the increment is `4/1 + 1 = 5`.)
* If the target is deeper than your current level, the countdown is `3 + 1d4` = **4–7
  world ticks** ("The air around you starts to swirl..."), then "The floor opens beneath
  you!" and you arrive at the target with no connected staircase.
* If the target is not deeper than your current level (e.g. you are already at a quest
  level that blocks), you get "a malevolent presence blocking passage"; and if that
  becomes true during the countdown you are "thrown back in an explosion" — a radius-5
  **destruction** centred on you.
* It works from the town (taking you to `max_depth + 5`) and is not blocked by
  no-teleport squares.
* Source: Scroll of Deep Descent (level 10, allocated DL1–100, **not** sold in the
  Alchemist in this version), and the `DEEP_DESCENT` activation.

ADVICE: Deep Descent is a *dive* tool, not a safe escape. At `max_depth` 30, reading it on
DL10 drops you on DL35. Before using it as an emergency exit, check that
`max_depth + 5` is survivable.

---

## 9.17 Teleport Level

FACT (`effect_handler_TELEPORT_LEVEL()`):

* Immediate (no delay). Up or down with 50/50 when both are legal.
* **Up is impossible** in the town or with forced descent.
* **Down is impossible** from a quest level (DL99 with Sauron alive, DL100 with Morgoth
  alive) or from DL127, and impossible when up is also impossible *and* the level below
  is a quest level. So on DL99/DL100 before the quest is done, Teleport Level always
  takes you **up**.
* Blocked by a no-teleport square or the anti-teleportation curse ("Teleportation
  forbidden!"). Against monsters' *teleport level* spell, nexus resistance protects.
* Arriving by Teleport Level creates no connected staircase.

Sources: Scroll of Teleport Level (DL20+), Mage spell (clvl 28), Rogue spell (clvl 35),
`TELEPORT_LEVEL` activation. Monsters with the TELE_LEVEL spell can cast it on you.

---

## 9.18 Persistent levels, forced descent and other birth options

From `src/list-options.h` (see also <https://angband.readthedocs.io/en/latest/option.html>):

| Option | Default | Effect on the dungeon |
|---|---|---|
| `birth_connect_stairs` | yes | staircase under you after taking stairs (section 9.14) |
| `birth_force_descend` | no | no up staircases are generated, `<` does nothing, Teleport Level only goes down, recall from town goes to `max_depth+1`, no recall from quest levels |
| `birth_no_recall` | no | Word of Recall does nothing until you have won |
| `birth_levels_persist` | no | "experimental": every level is kept and revisited at the same depth; stairs join up between levels via staircase rooms; no labyrinths, gauntlets, hard-centre levels or trapdoors; recall asks which visited level to go to |
| `birth_lose_arts` | no | artifacts left on a level are lost; enables the `$` feeling |
| `birth_feelings` | yes | level feelings shown |
| `birth_no_selling` | yes | more gold in the dungeon, no selling (see [Chapter 11](11-town-and-shopping.md)) |

---

## 9.19 The town

FACT (`src/gen-cave.c:town_gen()`, `town_gen_layout()`):

* 22×66, generated once and then **kept** for the whole game (the "Town" chunk is
  stored). Layout: lava streamers (6–8), a starburst floor area, the eight shops on a lot
  grid, some ruins of (passable) rubble, and a single down staircase. You always arrive
  on that staircase when returning by stairs or recall.
* Residents: 4 by day, 8 by night (`town-day`/`town-night`), placed at least 3 grids from
  you; town monsters are weak but some steal or are aggressive (see
  [Chapter 11](11-town-and-shopping.md)).
* Day/night: a day is 10 × 10,000 game turns; the town is fully lit by day.
* Stores restock based on how long you were away (`store_update()` on return).
* Destruction does nothing in town ("The ground shakes for a moment",
  `effect_handler_DESTRUCTION()`); Banishment and Mass Banishment do work there (they
  only refuse in the arena). Town lava burns like dungeon lava.

---

## 9.20 Quests and the bottom of the dungeon

FACT (`lib/gamedata/quest.txt`, `src/player-quest.c`, `src/player-util.c`):

* Two quests: **Sauron, the Sorcerer at DL99** and **Morgoth, Lord of Darkness at DL100**.
* A quest level is always a **classic** level, full size, and the quest monster is placed
  on generation if not already present.
* **No down staircases** are generated on a quest level; extra up stairs are generated
  instead. No trapdoors on quest levels.
* `dungeon_get_next_level()` stops at the first quest level on the way down, so stairs,
  Deep Descent and trapdoors cannot skip past DL99 while Sauron lives (and DL100 while
  Morgoth lives). Teleport Level from a quest level always goes up.
* When the last required quest monster dies, a **magical down staircase** appears where
  it died (`build_quest_stairs()`) and the quest's level is cleared, so normal levels
  (with down stairs) are generated at that depth afterwards.
* Killing Morgoth (both quests done) wins the game. Levels DL101–127 then become
  reachable by the staircase he leaves; they are ordinary levels.
* The official manual warns that Morgoth "has a habit of coming at you across the
  dungeon" on DL100 — he is not asleep for long and is very fast.

ADVICE: you can recall to DL99 directly once your `max_depth` is 99; Sauron is placed
somewhere on every DL99 level you enter until he dies. Uniques that are still alive are
also more likely to show up as you go deeper simply because there are fewer shallow
candidates left. See [Chapter 12](12-game-plan-by-phase.md) for the endgame plan.

---

## 9.21 The single-combat arena

The Paladin spell **Single Combat** (clvl 40, `lib/gamedata/class.txt`,
`effect_handler_SINGLE_COMBAT()`) moves you and a targeted monster into a 6×6 permanent
cell (`arena_gen()`); a monster resists if `randint0(spell_power) > clvl`. In the arena:
no teleport of any kind, no Banishment/Mass Banishment/Destruction, no trapdoors, Word
of Recall is suspended. The level is not stored and you return to where you were when
one of you dies.

---

## 9.22 Monsters that appear while you are on the level

FACT: every world tick there is a **1 in 500** chance of a new monster
(`process_world()`), generated asleep at least 25 grids away with the normal
`get_mon_num()` rules (including the 1/25 OOD boost). That is about **one new monster per
500 normal-speed turns**; resting 1000 turns brings ~2 newcomers. Summons, breeders
(capped at 100 alive) and monsters following you are extra.

ADVICE: long stays are not free. Breeders and wandering arrivals slowly refill a level,
and every arrival is a new chance at a deep monster. This is the mechanical reason the
"dive, don't farm" advice in [Chapter 10](10-exploration.md) works.

---

## 9.23 Quick reference tables

### Ways off a level

| Method | Delay | Where to | Blocked by |
|---|---|---|---|
| Up staircase | 1 turn | depth−1 (connected: arrive on a `>`) | forced descent |
| Down staircase | 1 turn | depth+1 (connected: arrive on a `<`) | none on quest levels |
| Trapdoor | instant | depth+1 | none (not generated on quest levels) |
| Word of Recall | 15–34 ticks | town ↔ recall depth | `birth_no_recall`, arena |
| Deep Descent | 4–7 ticks | `max_depth + 5` (quest-capped) | target not deeper → explosion |
| Teleport Level | instant | ±1 (always up from quest levels) | no-teleport square, anti-tele curse, nexus resist (vs monster spell) |

### Profile cheat sheet

| Profile | Depth | Freq | Lit? | Stairs | Danger notes |
|---|---|---|---|---|---|
| classic | all | ~45% | rooms only, ≤DL25 | 3–4 / 1–2 | standard |
| modified | all | ~49% | rooms only, ≤DL25 | 3–4 / 1–2 | denser rooms, interesting rooms, chambers, huge rooms |
| moria | 10–39 | ~2.5% | moria rooms ≤DL35 | 3–4 / 1–2 | orc/ogre/troll/giant population |
| labyrinth | 13+ | 2–4% | often (see 9.5.4) | 1 / 1 | small, packed, maybe unmapped and undiggable; free good/great items |
| cavern | 15+ | ~5% | never | 1–3 / 1–2 | dark, open, dense; objects at depth+5 |
| lair | 20+ | 0.5% | normal half only | 3–4 / 1–2 | themed horde in a dark cave half |
| gauntlet | 20+ | 0.5% | no | down only on far side | no-teleport; themed maze between you and `>` |
| hard centre | 50+ | 0.5% | vault only | 1–3 / 1–2 | guaranteed greater vault |

---

## Things to try (untested)

1. **Feeling-guided stair-scum for vault hunting.** Bounce on connected stairs at DL30–40
   until the danger digit is 5–6 *and* the level is "modified"-looking (lots of rooms),
   then explore just enough for the loot digit. Hypothesis: a danger digit of 5–6 at those
   depths correlates with lesser/medium vaults more than with OOD wanderers. Test: log
   (danger digit, loot digit, vault seen?) over 200 levels with `cheat_room` in a test
   character.
2. **Deep Descent from town as a "recall +5".** For a strong diver, reading Deep Descent in
   town lands on `max_depth + 5` in 4–7 ticks, cheaper than recall + stairs. Test: compare
   turns-to-new-max-depth for recall+stairs versus Deep Descent from town.
3. **Lowering max depth on purpose.** Answer "yes" to "Set recall depth to current depth?"
   after an over-dive so that Deep Descent and recall target a safer depth. Test: verify
   Deep Descent then targets the new `max_depth + 5`.
4. **Gauntlet bypass by Deep Descent.** On a gauntlet level the down stairs are behind a
   no-teleport themed maze; Deep Descent ignores no-teleport. Test whether reading it in
   the left cavern reliably skips the gauntlet.
5. **Phase Door through the gauntlet strip.** Phase (10 grids) works on no-teleport
   squares; repeated phases inside the gauntlet might hop through walls of the maze toward
   the right cavern. Test landing statistics.
6. **Unlit permanent-walled labyrinths as loot farms.** They carry `N(3,2)` good plus
   `N(2,1)` great items in a tiny area. With Magic Mapping useless only on gauntlets
   (labyrinths are mappable), a mapped dark labyrinth might be a fast great-item source for
   a mid-level character with good detection. Test: count great items found per turn spent.
7. **Locking doors as a borg defensive primitive.** Lock power `m_bonus(7, depth)` stops
   monsters with <~80 hp for many turns. Test: in early levels, measure how often locking a
   door behind you ends a chase by orc packs.
8. **Cavern "objects at depth+5" for early ego items.** At DL15–20 caverns generate items
   as if at DL20–25. Test whether a careful cavern sweep yields more egos per hour than
   classic levels.
9. **Lava as a moat.** FACT: monsters without `IM_FIRE` treat lava as a grid they hate
   (`src/mon-move.c:monster_hates_grid()`). Standing behind a lava line (town, "Little
   eruption" rooms, vault `` ` `` glyphs) might neutralize melee packs that lack fire
   immunity. Untested: how reliably they path around versus wait; test with a fire-immune
   character and with a ranged character shooting across the lava.
10. **Hard-centre detection.** A hard-centre level has a vault in the middle and four
    caverns with no rooms; recognizing the profile early (magic map shows cave walls on all
    sides of a rectangular vault) could be used to decide "vault raid or leave" in a few
    turns. Test a recognizer on generated maps.

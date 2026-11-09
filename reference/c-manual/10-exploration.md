# 10. Strategies of Exploration

How to explore a level efficiently and safely: what to do in the first turns on a new
level, how detection works and what it covers, the autoexplore/pathfinding commands of
this master, running and disturbance, traps, light, stealth versus clearing, how fast to
dive, when to take the stairs, and a step-by-step procedure an automated player can
follow. Mechanics are cited from the game source; judgement calls are marked ADVICE.

**Contents**

1. [Arriving on a new level: the first turns](#101-arriving-on-a-new-level-the-first-turns)
2. [Reading the level feeling](#102-reading-the-level-feeling)
3. [Detection: what finds what, and how far](#103-detection-what-finds-what-and-how-far)
4. [Autoexplore and pathfinding commands](#104-autoexplore-and-pathfinding-commands)
5. [Running and the disturb rules](#105-running-and-the-disturb-rules)
6. [Exploration patterns](#106-exploration-patterns)
7. [Traps and trap-safe movement](#107-traps-and-trap-safe-movement)
8. [Light: carrying it, lighting rooms, dark levels](#108-light-carrying-it-lighting-rooms-dark-levels)
9. [Stealth play versus clearing](#109-stealth-play-versus-clearing)
10. [Diving versus farming: descent rates and clvl/dlvl heuristics](#1010-diving-versus-farming-descent-rates-and-clvldlvl-heuristics)
11. [Stair-scumming and its tradeoffs](#1011-stair-scumming-and-its-tradeoffs)
12. [When to leave a level](#1012-when-to-leave-a-level)
13. [Doors as defensive tools](#1013-doors-as-defensive-tools)
14. [Handling breeders](#1014-handling-breeders)
15. [Resting safely](#1015-resting-safely)
16. [Item pickup policy and time budget](#1016-item-pickup-policy-and-time-budget)
17. [Food and light on long levels](#1017-food-and-light-on-long-levels)
18. [Keeping an escape path; unknown-grid heuristics](#1018-keeping-an-escape-path-unknown-grid-heuristics)
19. [Step-by-step exploration procedure](#1019-step-by-step-exploration-procedure)
20. [Things to try (untested)](#things-to-try-untested)

Background on level generation, feelings and stairs is in [Chapter 9](09-dungeon.md);
escapes in [Chapter 13](13-escapes-and-emergencies.md); stealth, noise and monster
waking in [Chapter 1](01-core-mechanics.md); monster families in
[Chapter 8](08-monsters.md). Official manual pages: commands
(<https://angband.readthedocs.io/en/latest/command.html>, vendored
[user-manual/command.rst](user-manual/command.rst)), playing / disturb / pathfinding
(<https://angband.readthedocs.io/en/latest/playing.html>,
[user-manual/playing.rst](user-manual/playing.rst)), options
(<https://angband.readthedocs.io/en/latest/option.html>).

---

## 10.1 Arriving on a new level: the first turns

FACT (`src/game-world.c:on_new_level()`, `src/generate.c`):

* You are disturbed, the danger feeling is printed, adjacent secret doors are searched
  for, and you get at least one full turn of energy (100) before monsters act.
* If you came by stairs with connected stairs on, you are standing on a staircase back
  the way you came (down → `<` under you; up → `>` under you). By recall, trapdoor,
  Teleport Level or Deep Descent there is no staircase.
* Initial monsters were placed with minimum distance 0 from your start, so a sleeping
  monster can be adjacent. They are asleep unless they are pit/nest monsters (awake) or
  have woken from your noise.
* Your start position prefers enclosed grids (corridors and room edges), like stairs.

ADVICE — the arrival checklist (do it before moving):

1. **Read the danger feeling** (section 10.2). If it is 7+ ("terribly dangerous" or worse)
   and you are on a staircase, consider leaving at once.
2. **Look at the monster list** (`[`, "Display visible monster list", or a subwindow)
   for anything already in view or sensed. Anything awake and dangerous in view while
   standing on stairs → take the stairs.
3. **Detect.** Monsters first (the most important information), then the map/stairs if
   you have it, then traps/doors if you are a mage (only mages have trap detection, see
   10.3), then objects.
4. **Assess**: count dangerous monsters, note uniques, hounds (`Z` packs), breeders,
   invisible-monster hints, and where the stairs are. Decide: *explore*, *explore part
   and leave*, or *leave now*.
5. If you will explore, **note your escape route** (the staircase you are on is the best
   one you will ever have on this level).

---

## 10.2 Reading the level feeling

Full mechanics in [Chapter 9.9](09-dungeon.md#99-level-feelings). Short version:

* The status bar shows `LF:d-o`. **d (danger, 1–9)** is shown at once; **o (loot, 1–9 or
  `$`)** shows `?` until you have *seen* 10 of the level's 100 hidden feeling squares
  (about 10% of its floor). `^F` repeats the message.
* d is `monster rating / depth`, where each monster adds `level²` and an out-of-depth
  monster adds `(level − depth) × level²` more. A single monster 20 levels out of depth is
  enough to reach d=6 at DL30. Uniques count like any monster of their level.
* o is `Σ(value/100)² / depth` over floor items at generation; an artifact guarantees
  o ≥ 5 (or `$` with `birth_lose_arts`).

ADVICE — how to act on it:

| Danger d | Typical cause | Default action |
|---|---|---|
| 1–3 | ordinary population | explore normally |
| 4–5 | a vault, pit/nest, room of chambers, a unique, or a moderately OOD monster | explore, but detect monsters before entering big rooms; locate the cause |
| 6 | something much deeper than the level, or a big vault | find out what it is (detection, ESP) before committing; leave if it is a known killer for you |
| 7–9 | a very deep monster or several; greater vault; dangerous pit | leave unless you are overpowered for the depth and want the vault |

| Loot o | Action |
|---|---|
| `?` | keep exploring until it resolves if you would act on it |
| 1–3 | nothing worth a detour; leave when convenient (or explore only for XP) |
| 4–5 | normal; explore what is cheap to explore |
| 6–9 | something good; worth extra time and some risk |
| `$` | an artifact you lose by leaving (only with `birth_lose_arts`) |

A low danger feeling is not a guarantee: monsters arriving later (1 in 500 per world tick),
summons and wanderers are not in the rating.

---

## 10.3 Detection: what finds what, and how far

### Area

FACT: almost every detection effect uses `effect-yx:22:40` — a rectangle extending **22
rows up and down and 40 columns left and right** of you (about one 80×44 screen; the loops
stop one short of the bottom/right edge). Exceptions: Rogue *Detect Stairs* 44×60,
*Rod of Magic Mapping* 33×55, Druid/Ranger *Sense Surroundings* and Necromancer *Read
Minds* use level-scaled dice, and *Enlightenment*/*Clairvoyance* cover the whole level.
A full 198-wide level needs about three detections side by side to cover.

Detected monsters are shown **only until your next turn** (`MFLAG_SHOW`/`MFLAG_MARK`,
cleared in `src/game-world.c:process_player_cleanup()`), so read the screen/monster list
immediately after detecting.

### What each effect finds

| Effect | Finds | Sources (examples) |
|---|---|---|
| `DETECT_VISIBLE_MONSTERS` | all monsters that are not invisible | Mage *Detect Monsters* (clvl 3), Rogue (5), part of *Reveal Monsters*, *Detection*, Rod of Detection, *Enlightenment* potion (whole level) |
| `DETECT_INVISIBLE_MONSTERS` | invisible monsters | Scroll/Staff of Detect Invisible, *Reveal Monsters* (Mage 15, Rogue 25), *Detection* |
| `DETECT_LIVING_MONSTERS` | living (non-undead, non-nonliving) | Druid/Ranger *Detect Life* (1/5) |
| `DETECT_EVIL` | evil monsters | Priest/Paladin *Detect Evil* (1/3), Staff of Detect Evil |
| `DETECT_SOUL` + `READ_MINDS` | monsters with minds, and maps the area around them | Necromancer *Read Minds* (4) |
| `DETECT_FEARFUL_MONSTERS` | monsters that can be frightened | Blackguard *Seek Battle* (1) |
| `DETECT_TRAPS` / `DOORS` / `STAIRS` | traps (incl. chest traps), doors incl. secret, stairs | **only** Mage *Find Traps, Doors & Stairs* (clvl 1) and *Detection* (30); Rogue *Detect Stairs* (stairs only, 12) |
| `MAP_AREA` | walls, doors, stairs, rubble, veins — not floors, not traps, not objects | Scroll of Magic Mapping, Staff of Mapping, Rod of Magic Mapping, *Sense Surroundings* |
| `DETECT_ORE` / `SENSE_GOLD` / `DETECT_GOLD` | treasure veins / gold | Treasure Detection/Location, *Object Detection* |
| `SENSE_OBJECTS` / `DETECT_OBJECTS` | objects (sensed = location only / detected = identity) | *Treasure Detection* (Mage), *Object Detection* (Rogue), Rod of Treasure Location, Rod of Detection |
| `LIGHT_LEVEL` | perma-lights and maps the whole level; senses (or fully knows) all objects | Potion of Enlightenment, *Clairvoyance* (Priest/Paladin 37) |
| Telepathy (`TELEPATHY` timed or ESP item) | all monsters with minds in range, continuously | Mushroom of Second Sight (50+7d7 turns), ESP gear |

FACT: there is **no scroll, potion, staff, rod or activation that detects traps** in this
version (`grep DETECT_TRAPS lib/gamedata/*.txt` finds only the Mage and Rogue spells).
Non-mages find traps by seeing them (section 10.7).

FACT: magic mapping does not mark floors as known, so the pathfinder and the `p`
autoexplore command still treat mapped-but-unseen floors as unexplored
([playing.rst](user-manual/playing.rst), `effect_handler_MAP_AREA()` skips
`square_isfloor`).

ADVICE — detection order on arrival and periodically:

1. Monsters (all kinds you can detect; invisible too if anything hints at ghosts at
   DL20+).
2. Map/stairs (Magic Mapping or *Find ... Stairs*), so you know the shortest way out.
3. Traps (mages) for the area you will walk.
4. Objects, if you are going to spend time looting.

Re-detect monsters every ~30–50 squares of travel, before opening any door into an
unexplored room, before resting, and whenever something disturbs you without being
seen.

---

## 10.4 Autoexplore and pathfinding commands

This master adds an **interface option `autoexplore_commands`** ("Use autoexplore
commands", default **off**; `src/list-options.h`). With it on
([option.rst](user-manual/option.rst), [command.rst](user-manual/command.rst),
`src/cmd-cave.c`):

| Key | When not on the matching staircase | Code |
|---|---|---|
| `<` | path to the **nearest known up staircase** (by turns) and start walking | `do_cmd_navigate_up()` |
| `>` | path to the **nearest known down staircase** | `do_cmd_navigate_down()` |
| `p` (both keysets) | path to the **nearest known passable grid with an unknown neighbour**; failing that, to a known closed door or rubble that has an unknown neighbour | `do_cmd_explore()` → `src/player-path.c:path_nearest_unknown()` |

On a staircase `<`/`>` still take it. The context menu gains "Explore". Repeating the
command (`n`, or `^V` in the roguelike keys) recomputes the path from where you are.

### Preconditions (each command refuses otherwise)

* Not confused ("You cannot explore while confused.").
* **No visible monster in view** ("Something is here."; `player_has_monster_in_view()`).
* If you are in a web, the command clears the web instead (one turn).

### How the path is chosen (`src/player-path.c`)

* Cost is in expected turns: an unlocked closed door costs one extra turn; a locked door
  costs `100 / unlock_chance` turns assuming the worst lock (power 7); impassable rubble
  costs the expected digging time with your best digger (`compute_*_penalty()`).
* Passes are tried in order: known grids avoiding known traps; known grids allowing
  traps (skipped if you are trap-immune); then the same allowing **unknown grids, which
  are assumed passable**. Damaging terrain (lava) is never used.
* For `p`, the destination is the *closest* frontier grid, not the most promising one.

### What happens while following the path (`run_step()`)

* A known closed door on the path whose 3×3 neighbourhood is fully known is opened
  automatically (pushes `CMD_OPEN`, then re-paths). Known impassable rubble likewise is
  tunnelled (`CMD_TUNNEL`).
* It stops (disturb) when: the path ends; the next grid is known impassable terrain it
  cannot handle; a **visible monster** is in the next grid; a **visible, non-ignored
  object** is in the next grid; you would step from a trap-detected area into an
  undetected one; or any normal disturbance happens (a monster comes into view with
  `disturb_near`, you take damage, etc., section 10.5).
* If the path would end in a wall it degrades into an ordinary run in that direction.

ADVICE for using `p`:

* `p` is a safe *mechanical* explorer only because it refuses to start with a monster in
  view and stops for anything new. Its weakness is that it greedily picks the nearest
  frontier, so it zig-zags between half-explored rooms and happily walks into big dark
  rooms. Pair it with periodic monster detection.
* It ignores your danger map: it will path *through* a known nest door or a vault
  entrance if that is the nearest frontier. A borg should filter the target: if the
  chosen frontier is in a vault, pit or next to a known dangerous sleeper, mark it and pick
  another (e.g. by temporarily treating those grids as blocked in its own planner).
* Pressing `>` with autoexplore on is the fastest "go to the stairs now" command — once
  a down staircase is known (magic mapping marks stairs as known).
* It does not search for secret doors deliberately; it only finds them by walking next to
  them (`search()` runs after every step). Dead ends left over at the end of `p`
  exploration are where secret doors hide.

---

## 10.5 Running and the disturb rules

### Running (`Shift`+direction or `.`/`,` + direction)

FACT (`src/player-path.c:run_test()`): a run follows corridors around corners and stops
when a newly adjacent grid contains a **visible monster**, a **visible trap** (unless
trap-immune), a **visible non-ignored object**, an **"interesting" feature** (doors,
stairs, rubble, shop entrances, veins with treasure), or when the corridor branches
(two non-adjacent openings or three openings). In open areas it runs along walls and
stops at breaks. Confused characters cannot run. Running into a visible trap on the first
step is treated as an attempt to disarm it.

### Disturbance

FACT ([playing.rst "Disturb"](user-manual/playing.rst)): repeated commands (running,
resting, tunnelling, pathing, repeated counts) stop when: a monster hits you or casts at
you; a visible monster in view moves, enters or leaves view (**only with
`disturb_near`**, default on); you lose HP; a projection affects you; a timed effect
worsens; you find a secret door or chest trap; you hit a trap; you are about to leave a
trap-detected area while running; your light runs low; an item with `!!` recharges; you
step onto a visible pile; you enter a new level; and so on.

ADVICE: leave `disturb_near` **on** (a borg should too — it is the cheapest "something
moved" alarm). The only interface option relevant to exploration that a borg may want off
is `pickup_always` (section 10.16).

---

## 10.6 Exploration patterns

ADVICE. There is no single best pattern; choose by class, depth and lighting.

| Pattern | How | Good for | Risks |
|---|---|---|---|
| **Corridor-first** | follow corridors with runs, peek into rooms from the doorway, enter only lit/empty ones | low-HP casters, dark deep levels | slower to clear rooms, more doors |
| **Room clearing** | enter each room, light it if dark, kill what is inside, loot | melee characters at comfortable depth | walking into groups in the dark |
| **Map-guided sweep** | magic map first, visit rooms in a loop starting nearest, end at the stairs | anyone with mapping | mapping doesn't show monsters or objects |
| **Stair-to-stair dive** | detect/map, walk the shortest path to `>`, ignore the rest | diving characters, bad feelings | no XP/loot; arrival risks every level |
| **Objective sweep** | object-detect, visit only the interesting items, leave | late game (only big items matter) | ignores XP |
| **Autoexplore (`p`) with guards** | repeat `p`; detect monsters every few runs; stop and think when it refuses | borgs, tedious shallow levels | greedy frontier choice (10.4) |

General rules:

1. **Fight in corridors, not rooms.** In a corridor only one or two monsters reach you;
   in a room a group surrounds you. When a group wakes, back into the corridor you came
   from.
2. **Light dark rooms before entering** if you can (10.8); otherwise detect monsters
   first.
3. **Don't open closed doors with your face.** Detect before opening a door into
   unexplored territory; nests/pits open directly onto awake monsters.
4. **Leave rooms by the way you came** when you haven't explored beyond, so the known
   path back stays short.
5. Use the full map (`M`) and locate (`L`) to plan; the pathfinder's view of unknown
   grids is optimistic (assumes passable).
6. **Deep levels are dark** (no ordinary room is lit below DL25,
   [Chapter 9.12](09-dungeon.md#912-light-and-darkness)); from DL25 on, exploration is
   detection-driven, not sight-driven.

---

## 10.7 Traps and trap-safe movement

FACT (`src/trap.c`, `lib/gamedata/trap.txt`):

* A trap is noticed when its grid is in view and your **searching skill ≥ the trap's
  visibility power** (`square_reveal_trap()`). Runes (summon, teleport, necromancy...)
  have visibility 0 or low and are almost always seen; pits are 70–90, trapdoors
  `2d10 + m_bonus(50)`, slow darts `m_bonus(50)`, aggravation `m_bonus(50)`.
* Searching skill = class base + race + `increment × clvl / 10` (Warrior 10 + 12/10 levels,
  Rogue 20 + 16/10 levels, Ranger 15 + 15/10 levels; `lib/gamedata/class.txt`,
  `src/player-calcs.c`). So a low-level character misses most pits and trapdoors;
  a clvl-40 rogue sees nearly all.
* Detection (`DETECT_TRAPS`) reveals all traps in the area and marks it `DTrap`; the
  status bar shows **DTrap** (yellow when you stand on the edge of the detected area).
  Running/pathing stops before you step out of a detected area.
* Trap-immune characters (item rune `TRAP_IMMUNE` or a trap-safe timed effect) walk over
  traps without triggering them; the pathfinder and runs then ignore traps.
* Random corridor traps are rare on classic/modified levels (0–2 per level) but common in
  labyrinths, caverns, gauntlets and around room treasure (vaults, inner rooms).
* Trapdoors drop you one level (Feather Fall prevents only the damage); there are none on
  quest levels or with persistent levels.
* Disarming: `D` uses physical or magical disarm skill depending on the trap; walking
  onto a known trap asks for confirmation.

ADVICE:

* Mages: cast *Find Traps, Doors & Stairs* on arrival and keep inside the DTrap area when
  walking known-dangerous corridors (labyrinths, vault approaches).
* Everyone else: assume unseen pits/trapdoors in corridors you haven't walked; treat a
  trapdoor as a free (unplanned) descent — fine if you are ready for depth+1, otherwise
  prefer known paths.
* Never walk into an unknown rune in a vault: rune of summoning / dragonsong / hellhole
  summon monsters at the trap's depth.
* Wand/Rod of Disable Traps or the Mage spell *Disable Traps, Destroy Doors* clear a
  corridor of traps when one blocks a route.

---

## 10.8 Light: carrying it, lighting rooms, dark levels

FACT: torch intensity 2 = lit radius 1 (5000 turns, destroyed when empty), lantern intensity 3 = radius 2 (15000,
refuelled with flasks), artifacts/egos add `LIGHT` modifiers; a light with no fuel gives
no light. Without light (and not a Necromancer) you cannot read, cast, or see traps
and secret doors (`player_can_read()`, `player_can_cast()`, `search()`).

Room lighting drops off with depth: ordinary rooms are lit with probability
`(26 − depth)/25` (none below DL25). Lighting effects:

| Effect | Area | Sources |
|---|---|---|
| `LIGHT_AREA` | the room you are in (or a small radius in a corridor), plus a weak light ball that hurts light-sensitive monsters | Mage *Light Room* (1), Priest *Call Light* (1), Paladin (5), Scroll of Light, Staff of Light, Rod of Illumination |
| `LINE:LIGHT_WEAK` | a beam that lights a corridor line | Wand/Rod of Light, Priest *Spear of Light* |
| `LIGHT_LEVEL` | whole level | Potion of Enlightenment, *Clairvoyance* |

ADVICE:

* Light dark rooms from the doorway before entering: you see what is inside; the weak light ball only
  affects `HURT_LIGHT` monsters (orcs, some undead...), which take damage and wake —
  others are skipped by `project_monster_hurt_only()` and stay asleep; and the room's floor
  becomes "seen", which also counts toward the loot feeling.
* A Wand or Rod of Light down a long corridor shows what is coming and damages orcs and
  other light-sensitive monsters.
* Necromancers explore differently (they want darkness): see
  [Chapter 14](14-class-playbooks.md).

---

## 10.9 Stealth play versus clearing

FACT (`src/mon-move.c:monster_reduce_sleep()`): each monster turn a sleeping monster
rolls `notice = randint0(1024)`; if `notice³ ≤ 2^(30 − stealth)` it loses sleep, by 1 or
by `100/d` when it is within flow distance `d < 50` of you. Better stealth makes the roll
exponentially rarer; aggravation wakes everything. Details in
[Chapter 1](01-core-mechanics.md).

ADVICE:

* **Stealth play** (sneak past sleepers, take only what is near the path) suits rogues,
  hobbits/kobolds, mages and anyone diving. It gives up XP per level but keeps danger per
  turn low.
* **Clearing** (wake and kill everything reachable) suits warriors, blackguards and
  paladins at a depth where nothing on the level is a threat; it maximises XP per level
  and loot, at the cost of time and resources.
* Mixed: kill what blocks the path and what is worth the XP; leave sleeping dangers
  asleep (don't melee a sleeping unique you can't finish; don't throw things at sleeping
  dragons "to see").
* Time on a level increases risk: wandering monsters (1/500 per tick), breeders and the
  slow wake-up of everything within ~50 flow squares. Finish what you intend to do and go.

---

## 10.10 Diving versus farming: descent rates and clvl/dlvl heuristics

ADVICE (experience-based, cross-checked with the borg's depth gates in
`src/borg/borg-prepared.c`, which are ideas, not authority):

The deeper you are, the better the items and the faster the XP — the game rewards diving.
But depth must be backed by **detection, escapes, free action, see invisible, basic
resists and HP** more than by character level. A useful rule set:

| Depth band | Minimum before going there (ADVICE) |
|---|---|
| DL1–4 | light source, food, a few CLW, Phase Door; clvl 5+ for casters before DL3 |
| DL5–9 | Word of Recall scroll, 2+ cure potions, Flasks of Oil (to throw) or a launcher; clvl ≈ DL+3 for weak classes |
| DL10–19 | radius-2+ light, ≥2 Teleportation sources (staff or scrolls), CLW/CSW ×5+; see invisible or a plan for ghosts by DL15; clvl ≥ DL for most classes, clvl ≥ DL+5 for mages |
| DL20–29 | **Free Action** (carrion crawlers, ghouls, paralysers), resist fire and two of cold/elec/acid, 4+ teleports; clvl ≈ DL+5 |
| DL30–39 | all four base resists, Teleport Level or Deep Descent as an exit, 10 CCW; see invisible; clvl ≈ 30+ |
| DL40–49 | resist poison (hounds and drolems), resist confusion or a plan, +5 speed or better, Healing potions |
| DL50+ | speed +10 or more, resist nether/chaos/blindness/confusion/sound as available, many Healing / *Healing*, Mass Banishment or *Destruction* for emergencies |

Heuristics:

* **"clvl vs dlvl"**: early (DL1–20), keeping clvl ≥ dlvl (casters clvl ≥ dlvl+5) is a
  good default; after DL20, equipment and resists matter far more than clvl, and a
  well-equipped clvl-30 character can visit DL30–35.
* **Descent rate**: when a level feels easy (you are not using escapes and nothing on the
  danger list is threatening), take the first down staircase you find. When you used an
  escape on the last two levels, stop descending until you have a reason (new gear,
  levels gained).
* **Use recall to reset**: dive until supplies run low or a new resist is needed, recall,
  shop, recall back to `max_depth` ([Chapter 9.15](09-dungeon.md#915-word-of-recall-and-the-recall-depth)).
* **Farming** a depth (clearing many levels at the same DL) is only worth it when you
  need a specific thing found at that depth (e.g. Potions of CCW for a borg, books) or
  levels for a spell breakpoint.

---

## 10.11 Stair-scumming and its tradeoffs

FACT: with connected stairs, every stair trip puts you on a staircase of the opposite
kind, so you can alternate `<` and `>` and get a fresh level each time
([Chapter 9.14](09-dungeon.md#914-stairs-connected-stairs-and-stair-scumming)).

| Pros | Cons |
|---|---|
| Skips bad levels (labyrinths, gauntlets, LF7+ danger, unknown dark caverns) at 1 turn each | No net descent: you oscillate between d and d±1 |
| Lets you shop for a good feeling (high loot digit, vault-like danger digit) | The loot digit needs exploration, so scumming "for loot" still costs walking |
| Always standing on an escape | Each arrival rolls new monsters, some awake (pits/nests are awake; noise wakes others) |
| Fast way to get away from a unique that was hunting you | Uniques and OOD monsters keep appearing — scumming exposes you to many levels' worth of generation |

ADVICE:

* Scum to **leave** bad situations, not to farm. A borg should allow "take the stairs
  you're standing on" as an escape whenever the danger estimate exceeds its comfort
  threshold.
* To dive, use: stairs + autoexplore `>` once a `>` is known, Deep Descent (to
  `max_depth + 5`), or trapdoors. Scumming does not dive.
* Don't scum at a depth where arriving next to an awake monster can kill you in one
  monster turn (e.g. low HP at DL40+ with hound packs).

---

## 10.12 When to leave a level

ADVICE — leave (by the nearest safe staircase, recall, or Deep Descent) when any of these
holds:

1. **Danger feeling ≥ 7** at a depth you are only just ready for, or ≥ 6 and you can't
   identify the cause.
2. **A unique or OOD monster you can't beat is awake** and hunting you (it knows where you
   are; uniques are often fast).
3. **Hounds**: a `Z` pack whose breath you don't resist (e.g. gravity, inertia, nexus,
   time, plasma, nether hounds), or any hound pack when you are low on HP. Hounds come in
   packs, are often fast at depth, and breathe from range.
4. **Breeders out of control**: once worm masses, lice or flies fill corridors you are
   spending turns for no gain ([section 10.14](#1014-handling-breeders)).
5. **Invisible attackers** you can't see or detect (ghosts draining XP/stats).
6. **Supplies**: fewer escapes than your budget ([Chapter 13.5](13-escapes-and-emergencies.md#135-the-escape-budget)),
   no cure potions, light or food running out.
7. **The loot feeling is low and you have what you came for** (XP, an item, a level-up).
8. **Summoners** that keep calling help faster than you kill (e.g. at DL30+ demon/undead
   summoners) when you can't kill them in two turns.
9. **Level layout is against you**: unknown dark labyrinth with permanent walls, a
   gauntlet with you on the wrong side, a cavern full of archers.

Prefer stairs (instant) over recall (15–34 ticks) when danger is near. Remember Deep
Descent goes to `max_depth + 5` and Teleport Level is random up/down
([Chapter 9](09-dungeon.md#923-quick-reference-tables)).

---

## 10.13 Doors as defensive tools

FACT ([Chapter 9.11](09-dungeon.md#911-doors)): monsters without `OPEN_DOOR`/`BASH_DOOR`
cannot pass a closed door at all; an unlocked door costs a door-user one turn; a locked
door stops a monster unless `randint0(hp/10) > lock power` (power 1–7). You lock a closed
door with `D` (disarm) on it; lock power `m_bonus(7, depth)`.

ADVICE:

* Close doors behind you when retreating from animals, hounds, jellies, insects and
  other non-door-users; many packs simply stop.
* Lock doors against weak door-users (orc packs early on): a power-5+ lock holds monsters
  with < ~60 hp for many turns.
* Fight at a door: stand diagonally from the doorway or one step back in the corridor so
  only one monster can come through at a time.
* A closed door also blocks line of sight and projections (breaths, bolts) — close it to
  stop an archer or breather in the room from targeting you while you heal.
* Mage *Door Creation* (clvl 13) builds doors around you: a panic wall against non-door
  monsters.

---

## 10.14 Handling breeders

FACT (`src/mon-move.c`): a breeder (`MULTIPLY`) tries to reproduce when fewer than 4
monsters are adjacent: always if none are adjacent, otherwise 1 in `8 × adjacent count`
(`mult-rate 8`). New offspring are awake. Breeding stops while 100 breeders are alive on
the level (`repro-max`), and resumes as they die.

ADVICE:

* Kill breeders **early**, before there are many. A single worm mass is trivial; fifty
  are a time sink.
* If a breeder colony is already large, **leave the level** — the XP per turn is poor
  and they cut you off from parts of the map.
* Fight them in a corridor so adjacency (which throttles breeding: 4+ neighbours stops
  it) works for you; area attacks (balls, *Mass Sleep*, *Dispel*) clear clusters.
* Some breeders are dangerous to touch (yellow worms poison, some lice/fleas are fast);
  don't melee acidic/poisonous breeders with valuable gear unless resisted.

---

## 10.15 Resting safely

FACT (`src/player-util.c:player_regen_hp()`):

* HP regenerates every world tick by `mhp × 197/65536` (≈0.30% of max) plus a small
  constant, multiplied by up to 1.33 when well fed, **×2 while resting** (after 5
  consecutive rest turns, or immediately with `R&`/`R*`), ×2 with Regeneration, ×½ with
  impaired HP regeneration. Full HP from near zero takes about **125–165 ticks resting**
  (60–85 with Regeneration).
* **No HP regeneration at all while poisoned, stunned, cut or paralysed.** Cure those
  first (CLW reduces cuts; CSW/CCW cure cuts; CCW cures poison and stun).
* Hunger: Weak or worse halves or cuts regeneration.
* `R&` rests until HP/SP are full and no bad timed effect remains (and no recall or deep
  descent pending); `R*` until HP and SP are full; or a number of turns.
* Resting checks for keypress interrupts only every 128 turns, but any disturbance stops
  it (a monster appearing in view with `disturb_near`, damage, etc.).

ADVICE — where and when to rest:

* Rest in a **dead-end corridor or on a staircase**, out of line of sight of rooms, after
  detecting monsters. A staircase rest is ideal: anything nasty arrives and you leave.
* Don't rest where a summoner or breeder colony can reach you; don't rest in a lit room
  (monsters see you from afar).
* Before resting, ask: "What woke up because of the fight?" Detect again.
* Rest to full before opening a new room or taking a staircase down; arriving at 50% HP
  is how characters die to arrival ambushes.
* A borg should treat "rest interrupted by an unseen attacker" as an invisible-monster
  emergency ([Chapter 13.6](13-escapes-and-emergencies.md#136-emergency-decision-procedures)).

---

## 10.16 Item pickup policy and time budget

FACT ([command.rst](user-manual/command.rst)): picking up gold costs no time; each object
costs 1/10 of a turn (max one turn per pickup action). Options `pickup_always` (default
off) and `pickup_inven` (default on: pick up items matching inventory). Ignored items
(`k`/ignore menus) don't stop runs or pathing. Pack size 23 slots, quiver 10 slots
(`constants.txt`).

ADVICE:

* Set up ignoring early (bad flavours once known, average/good items you will never
  use) — it keeps runs and `p` from stopping on junk and keeps the pack clean.
* With `birth_no_selling` (default), unknown items still need to be learned for
  their runes and flavours, but money comes from gold drops: pick up gold always,
  potions/scrolls/wands/staffs/rods always (until known useless), and wearables only if
  they could be an upgrade or need identifying.
* Don't detour more than ~20 squares for an unknown item when the loot digit is ≤ 3.
* A full pack drops items when it overflows (and disturbs you) — keep 2–3 free slots on
  long trips.

---

## 10.17 Food and light on long levels

FACT: food is a timed value up to 10,000 (Full); Hungry below 1,500, Weak below 800,
Faint below 400 (`lib/gamedata/player_timed.txt` × `food-value` 100). Every 100 game
turns you digest `turn_energy(speed)` units (10 at normal speed, 20 at +10), doubled by
Regeneration and halved by Slow Digestion (`process_world()`). A Ration of Food
(`NOURISH:INC_BY:30`) adds 3,000 — about **3,000 normal-speed turns**. Torches last 5,000
turns, lanterns hold 15,000.

ADVICE:

* Carry 4–5 rations (or Scrolls of Remove Hunger / Elvish Waybread) and a spare light
  source (torches) or 3+ flasks (lantern; flasks double as thrown fire damage).
* Eat at "Hungry", not later — Weak halves regeneration.
* Very fast characters digest faster: +20 speed triples the food rate.
* Treat "light below 1000 turns and no fuel" as a reason to go home.

---

## 10.18 Keeping an escape path; unknown-grid heuristics

ADVICE:

* **Always know your exit**: the nearest known staircase and the path to it. Before
  entering any room or opening a door, check that you could reach a corridor or stair if
  everything inside woke up.
* **Teleport destinations** avoid vault squares and occupied squares but are otherwise
  random at ~60% of the level-span distance (Teleportation) or ≈10 squares (Phase Door);
  a teleport is an escape *from* a monster, not *to* safety
  ([Chapter 13.1](13-escapes-and-emergencies.md#131-teleport-mechanics)).
* **Unknown grids**: the pathfinder assumes they are passable, which is why "shortest
  path" through unexplored areas is optimistic. When planning a retreat, use only known
  grids.
* **Unknown-grid heuristics** for choosing where to explore next:
  * A corridor ending in unknown is usually a continuation; a room wall gap leads to a
    corridor.
  * Large unexplored rectangles between known rooms often hold another room or a vault;
    mapped walls with `#`-patterns and no visible entrance suggest a vault or
    room-template with secret doors.
  * Dead ends in classic levels are usually just dead ends (tunnels "terminate" 15% per
    step); dead ends next to rooms in templates/vaults often hide secret doors.
  * The edge of the level is permanent rock; don't plan escapes through it.
* **Keep the frontier small near danger**: explore away from known dangerous sleepers,
  not toward them.

---

## 10.19 Step-by-step exploration procedure

A procedure an automated player can follow. "Danger(g)" means the estimated maximum
damage the character could take at grid *g* before its next action
([Chapter 13.4](13-escapes-and-emergencies.md#134-damage-per-turn-and-hp-thresholds)).

```
ON_ARRIVAL:
  1. Record: depth, arrival method, staircase under me (yes/no), danger digit d.
  2. If d >= LEAVE_THRESHOLD(depth, power) and on stairs: take stairs. STOP.
  3. If any visible awake monster with max_damage >= 0.5*chp: escape/leave. STOP.
  4. Cast/use detections available, in order: monsters (+invisible), map/stairs,
     traps (mage), objects. Read results immediately (monsters show 1 turn).
  5. Build the threat list: uniques, OOD (level > depth+5), hound packs, breeders,
     summoners, invisible, anything that out-damages chp in one turn.
  6. Decide mode:
       LEAVE   if threat list has an unbeatable awake threat, or supplies < budget.
       DIVE    if diving and a down stair is known: path to it (autoexplore '>').
       EXPLORE otherwise.

EXPLORE_LOOP (repeat):
  a. If HP < 70% or a bad timed effect (poison/cut/stun) is present:
       cure; retreat to a quiet known spot (dead end / stair); detect; rest (R&).
  b. Every ~40 grids walked, before opening a door, and before resting:
       re-detect monsters.
  c. Choose target:
       - known valuable object (loot digit >= 4 or detected interesting item)
         reachable through known/low-danger grids, else
       - nearest frontier from path_nearest_unknown ('p'), rejecting frontiers in
         vaults / next to dangerous sleepers / pit or nest doors, else
       - nothing left: go to the stairs (DIVE or LEAVE).
  d. Before entering a dark room: light it from the doorway (if possible) or
     detect monsters; if a group is inside, fight from the corridor or skip it.
  e. Move with runs / pathfinding; on every disturbance: stop, look, classify:
       - visible monster: FIGHT if winnable at full HP with margin, else AVOID
         (back off out of LOS, close door) or ESCAPE (Chapter 13).
       - damage from unseen source: INVISIBLE procedure (Chapter 13.6).
       - trap found: disarm or route around.
  f. Pick up items per policy (gold always; consumables; possible upgrades).
  g. After the loot digit appears: if <= 3 and XP is not needed, switch to DIVE/LEAVE.
  h. Breeder count rising faster than kills: LEAVE.
  i. Food Hungry: eat. Light < 1000 turns and no spare: plan recall.

LEAVE / DIVE:
  - Prefer: stairs you stand on > nearest known stairs (autoexplore '<'/'>')
    > Deep Descent (only if max_depth+5 is acceptable) > Word of Recall
    (needs 15-34 safe ticks) > Teleport Level (random direction).
  - Before descending: HP and SP near full, no bad timed effects, budget met.
```

Suggested thresholds for a cautious borg (ADVICE, tune by testing):
`LEAVE_THRESHOLD`: d ≥ 7 always; d ≥ 6 if clvl < depth + 5 or no telepathy/detection;
explore HP floor 70%; descend only at ≥ 90% HP.

---

## Things to try (untested)

1. **Frontier filtering for `p`.** Wrap autoexplore with a filter that rejects frontier
   targets inside `SQUARE_VAULT`, within 5 grids of a known sleeping OOD monster, or behind
   a pit/nest door; measure deaths per 1000 levels against raw `p`.
2. **Loot-digit cutoff tuning.** Measure items-of-value found per turn after the loot
   feeling resolves, by loot digit. Hypothesis: below 4, exploring the remaining 90% of the
   level returns less than starting a new level.
3. **Danger-digit as a unique detector.** At DL20–40, log (danger digit, uniques
   detected). If d ≥ 5 on small levels reliably means "a unique or vault is here", a borg
   could use it to prioritise unique hunting or avoidance.
4. **Enlightenment for feelings.** Drinking Enlightenment right on arrival perma-lights
   the level, so walking a few corridors reveals 10 feeling squares quickly. Test turns to
   loot feeling with and without it.
5. **Staircase camping.** Rest and fight *only* while standing on a staircase; leave on
   any danger spike. Measure survival versus XP rate for fragile classes.
6. **Light-beam scouting.** Fire Wand/Rod of Light down every new corridor before walking
   it; test how many ambushes this prevents at DL25+ where nothing is lit.
7. **Door-lock kiting.** Against early orc packs, retreat through doors locking each one;
   measure how often the pack splits into singles you can kill.
8. **Trap-door diving.** For a diving character, deliberately step on known trapdoors
   (Feather Fall to avoid damage) as a free descent that skips needing `>`; compare dive
   speed.
9. **Scum-for-layout.** Stair-scum until the level is classic or modified with a known
   (lit, mapped) layout near the arrival point; test whether this reduces early-game
   deaths more than it costs in turns.
10. **Detect-before-door rule.** Count how many deaths in borg logs start within 3 turns of
    opening a door; if high, a mandatory "detect monsters before opening any unknown door"
    rule may be the single best safety change.
11. **Rest-on-stairs only below DL30.** Test whether restricting resting to staircases at
    DL30+ cuts deaths from wandering hounds and invisible ghosts.

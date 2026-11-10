# INSTRUCTIONS: Angband 4.2.6 for a player with one screen and ~150 decisions

What the game's manual and the strategy manual say, cut to what you use, in the order you use it. **What earlier runs found out is folded into this page**, and each of those lines names the runs that saw it as `run#ask`: the ask is in that run's own notebook under `knowledge/` and in its decisions under `records/`, so any line here can be traced to the screen it came from. What one model family found out about *itself* is in `agents/<family>/`, which is yours to read and to write back to.

**Depth 5 to 20 (breath, summoners, kit gates) is volume 2, `manuals/INSTRUCTIONS_2.md`; no run has tested it yet.**

Sources, so any line can be checked. `command`/`playing`/`attack`/`birth` = `vendor/angband/docs/<name>.rst`. `object`/`store`/`class`/`monster`/`trap` = `vendor/angband/lib/gamedata/<name>.txt`, and `race` = `vendor/angband/lib/gamedata/p_race.txt`. `src/x.c` = `vendor/angband/src/`. `c4`/`c14`/`c15` = strategy manual chapters at its pin (`git cat-file -p HEAD:docs/c-manual/<file>`). Words in double quotes are quoted from the source named beside them. Labels: **FACT** is in game data or source; **POLICY** is chapter 15's own "starting policy to be measured, not as truths", so following it and being hurt is a result worth reporting; **UNTESTED** means no run has tried it.

One caution about that chapter: it is *notes for the borg*, written for a program that reads the game's own variables. You read a screen, and the screen sometimes shows the reverse of what the program sees - the level feeling is the known case. A rule of its that mentions a number you cannot see, or can only see transformed, is not a rule you can follow; check against `src/` before you trust it.

Original keyset: digits walk (`1-9`, `5` stays), letters are commands. Keys you send are not interpreted: 80x24 screen, game turns advance in tens.

## The one thing to take from this page

**`lxg` explores and `l>g` descends. Use them instead of walking.**

- `lxg` — look, jump the cursor to the nearest square beside unexplored space, walk there. The game pathfinds; one ask covers what a dozen digit-presses would.
- `l>g` — the same to the nearest **known** down staircase. Then `>` takes it.
- `l<g` — the same to a known up staircase.
- Send each **alone in its ask**: any key after `g` interrupts the walk after one step, so `lxglxglxg` gives one-square hops, not three walks.
- With no staircase known yet, `>` in look mode does nothing (the game rings a bell) and the cursor stays put. Keep exploring with `lxg` until one is on the map.

OBSERVED, sonnet-2-again#156 to sonnet-2-again#230: this is how the only deliberate descent in any run happened — depth 6 to depth 7, after four runs and some 350 asks had never found a staircase at all. One ask walked 11 steps and 110 game turns onto the stairs.

Two runs went wrong here in ways worth not repeating. One spent ~100 asks walking a level blind and found nothing. One wrote itself a 143-line pathfinder, used it for 148 asks, reached depth 1, and never tried `lxg` — the game already had what it was building. **Check the game's own commands before you write code.**

## Keys as the game receives them

The keys of one ask are fed to the game without anything looking at the screen
in between, so every string is a bet that nothing interrupts it. This section
is what that bet has cost earlier runs. Each line says which runs saw it as
`run#ask`, and that ask is in the run's own notebook under `knowledge/` and in
its decisions under `records/`.

- **A `-more-` swallows the next key** (3 runs: sonnet-1#21, sonnet-2#16,
  sonnet-3#32). A command typed behind one loses its first key and the rest is
  misread. A space clears a `-more-` and is harmless otherwise (sonnet-2#20,
  sonnet-3#24), so end a string with spaces (`va'   `). A level change ends in
  one (sonnet-1#68, sonnet-2#26, sonnet-3#31): take stairs with `> `, not `>`
  - `<>` in one ask left a run standing in the town.
- **Menu letters are labels, not positions.** Store rows came up
  `a c f h j m n o q r u v y z` (3 runs: sonnet-1#25, sonnet-2#13,
  sonnet-3#22); the pack skips `h j k l`, so with eight items its fifth row is
  `e` and its eighth `i` (sonnet-1#57, sonnet-2#119); and the throw prompt has
  letters of its own, where a flask of oil was `a` and `b` or `c` in the pack
  (sonnet-2#40, sonnet-3#41). Read the letter off the screen, press it, and
  check the cursor moved to the row you meant - the header of every screen
  gives you `cursor at x,y` for exactly this.
- **A whole purchase fits one ask** (2 runs: sonnet-2#20, sonnet-3#16; the
  sequence itself in 3, sonnet-1#16): `p`, the row letter, the quantity as
  digits, Enter, then any
  key but Escape. `ph10\ny ` bought ten flasks of oil. "One purchase per ask"
  was an early wrong note.
- **Confused, blind, afraid or slowed: one or two keys per ask, never a string
  of waits** (sonnet-3#46, sonnet-3#47 - the run died there). It sent sixteen
  hold keys in one ask while confused; a wounded kobold it believed dead
  arrived about seven turns in and killed it from full health, with five Cure
  Light Wounds in the pack. The game flushes input on a disturbance, and what
  you sent is still in the buffer afterwards.
- **A digit string stops at the first wall and the rest is dropped**
  (sonnet-1#96, minimaxflash-1#14, which measured where the wall was). In
  caves `.` plus a digit stopped every two or three squares, where
  `4444444444` went further (sonnet-1#114) - but see the top of this page:
  neither is how to cross a level.
- **`l` with monsters on the screen answered only "You are on an open
  floor"** (sonnet-1#40, sonnet-2#58): the cursor starts on you. Both runs
  attacked or threw to find out what a letter was. Cycling with space, or `[`,
  is UNTESTED.
- **An unknown potion or scroll pays experience the first time it is used, and
  the payment shrinks fast.** FACT (`src/obj-knowledge.c`): the payment is
  `(kind level + your level / 2) / your level`, rounded down - at character
  level 1 that is the kind's own level, and by level 5 it is usually nothing.
  OBSERVED (sonnet-1#45, sonnet-1#57, sonnet-2#77, sonnet-2#110,
  sonnet-2#151): Blessing at CL1 moved `NXT` by one, Deep Descent at CL4 paid
  3, Cure Serious Wounds at CL5 paid 1, Blessing at CL6 nothing, and one
  unknown potion at CL1 was worth a whole character level. So try unknowns at
  CL1-3 and keep them for need after that: at CL5 one run drank an unknown at
  89/89 hit points and spent a Cure Serious Wounds on nothing (sonnet-2#109).
  Bought scrolls and potions are already named and pay nothing (3 runs:
  sonnet-1#57, sonnet-2#19, sonnet-3#25).
- **Try one at full hit points, with nothing in view, standing on a staircase
  you could take** - POLICY, from c15 ESC-01, which puts an unknown potion on
  the last rung of the escape ladder. Which kinds can hurt you is section 6b's
  `cost:0` rule; the complete list of potions and scrolls is appendix A6 of
  the strategy manual, to be looked up when you meet one, not read through.
- **A reroll of the level costs three asks** (sonnet-1#67, sonnet-2#25):
  `< ` then `> ` on connected stairs gives a freshly generated level of the
  same depth. It was no quicker to finish than the level it replaced
  (sonnet-1#69).

## 1. Be born (`birth` ; the birth screens)
- Race menu `*` (random item, Enter selects the highlight), class menu `*`. Stat screen: `a` (point-based), Enter to accept the allocation; Enter for the name, `y` for the history, any key to continue.
- Level-1 hit points = race hit die + class hit die (+CON) (`src/player-birth.c`: `hitdie = r_mhp + c_mhp`). FACT, `race` hitdie: Hobbit 7; Gnome, Kobold 8; Elf 9; Human, Half-Elf, Half-Orc, Dunadan, High-Elf 10; Dwarf 11; Half-Troll 12. Class die: Warrior 9, Blackguard 8, Paladin 6, Ranger 5, Rogue 4, Priest/Druid/Necromancer 2, Mage 0 (`class` hitdie).
- Level 2 needs `10 x XP-mod%` experience (`src/player.c`; mod = race exp, Human 100, Half-Troll 120, High-Elf 145). The sidebar `NXT` is experience still needed. A kill pays `monster exp x monster level / your level` (c15 15.3).
- Birth options in force (`birth` , `src/list-options.h`): start kit **on** (a few items, less gold), selling **off** (stores pay 0 gold), connected stairs **on**. Gold = 600 + unspent point-buy - kit value (`src/player-birth.c`), so it differs by class. Read `AU` on the left.
- Your class's block is in section 9; read it **after** the draw.

## 2. Look at what you have, then spend the gold
- `i` pack, `e` equipment, `|` quiver, `C` character sheet, `I` + item (damage per blow, blows per turn). Pack holds 23 kinds (`command`).
- Town stores are the digits on the map; walk onto one (`store` terrain order): **1** General, **2** Armoury, **3** Weaponsmith, **4** Bookseller, **5** Alchemist, **6** Magic shop, **7** Black market, **8** Home. In a store: `p` buys (then the row letter), `l` examines, Esc leaves.
- Prices. FACT (`object`, same as shop price for these): Flask of oil 3, Ration 3, Cure Light Wounds 20, Cure Serious Wounds 70, Phase Door 15, Magic Mapping 65, Word of Recall 125, Boldness 8, Monster Confusion 30, Blessing 15. **Gear (weapons, armour, torches, ammo) is priced by its power, not by the `cost:` line** (`src/obj-power.c: object_value_real`); read the price on the screen before buying. Torches burn out (the General Store lists 5000 turns).
- What each store always has (`store`): General: Ration, Torch, Flask of oil, Cloak, Shovel, Pick, shot/arrow/bolt. Alchemist: Cure Light Wounds, Phase Door, Word of Recall, Remove Curse; **sometimes** Magic Mapping, Cure Serious Wounds, Boldness, Remove Hunger. Magic shop **sometimes**: wands of Magic Missile and Stinking Cloud, staves of Teleportation, Detect Evil, Mapping (read the price). Bookseller: every town book. Not sold anywhere in town: Deep Descent, Teleportation scrolls, Healing potions (c15 TOWN-02).
- **POLICY** (c15 TOWN-01, c4 4.11): the town gives no experience; buy consumables first. Order there is escapes (Phase Door), a heal (Cure Light Wounds), then oil. Flasks of oil are the best early ranged attack for any class (FACT below, section 5); the strategy manual says 10-20 before the first descent.
- Armour above your class's weight allowance costs one spell point per pound over (Mage/Necromancer 30 lb, Priest/Druid/Rogue 35, Paladin/Ranger 40, Blackguard 60; c14 0.2).

## 3. Walk (`command`, `playing`)
- Step: a digit. Moving into a monster attacks it, into a door opens it, into a **visible trap tries to disarm it**; `W` + digit walks onto a trap on purpose. `o`/`c` open/close, `D` disarm or lock, `T` tunnel (rubble, veins), `g` pick up, `,` or `5` stay a turn.
- Run: `.` then a digit; it follows the corridor until you must choose or something disturbs you (`playing`, Disturb: a hit or spell on you, an object or monster in the next square, a known trap, a new level). A count repeats a command: `0`, number, command.
- `n` repeats the previous command. A keypress interrupts a run, rest or path (Space/Esc are safe).
- **Walk to a known place for one command** (see 'The one thing to take from this page' above; OBSERVED, sonnet-2-again#156 to #230, no longer untested) (`playing`, Targeting and Pathfinding, which lists `<`, `>`, `p` as up, down, unknown "respectively"; `src/ui-target.c` agrees): `*` or `l`, then `>` moves the cursor to the nearest *known* down staircase (`<` up), `x` to the nearest square next to unexplored space, then `g` walks you there along the shortest path, stopping when disturbed; `n` resumes. The `autoexplore_commands` option does the same on bare `>`, `<`, `p`, but options may not be changed here.
- Level feeling, bottom status line `LF:a-b`: **a is danger, higher = worse** (1 quiet ... 9 omens of death; `src/ui-display.c prt_level_feeling`), `?` until explored; b is loot. Chapter 15's STAIR-01 is not wrong, it is written for a different reader: internally 1 is the most dangerous (`src/generate.c calc_mon_feeling`, and the game's own comment in `src/ui-display.c`, "1 to 9 are feelings from omens of death to quiet"), and the screen prints `10 - feeling` to make it a danger number. A borg reads the struct; you read the reversal. So its `K_LF_LEAVE = 2` means 'leave the worst' on the internal scale and 'leave the two quietest' on yours - do not apply it to the screen. `^f` repeats the feeling text.
- Terrain on the status line names where you stand ("Up staircase", "Down staircase", "Open floor"). `>` and `<` work only on those.
- Stairs are connected (`birth`): after `>` you stand on an up staircase; after `<` on a down one. Leaving a level destroys it; taking a staircase always gives a new level.
- Light: your kit torch is wielded at birth and burns out; `F` refuels a lantern from flasks of oil. You cannot read scrolls in the dark, blind or confused (c15 CURE-02, `src/player-util.c`).
- Hunger: `Fed %` on the status line; `E` eats.

## 4. See (`command`, `playing`)
- `l` look. The list is sorted nearest first and includes your own square (`playing`; `src/target.c cmp_distance`), so it can open on it: Space or `+` for the next thing, `-` back, `r` toggles detail, Esc leaves (`playing`: "the list includes ... the player's location"). `[` lists visible monsters with asleep/awake and relative position. `]` lists items. `/` + symbol says what a glyph is. `M` whole-level map; `L` scroll the map by sectors (Esc leaves).
- Monster memory: look at a monster for its name and health; kills and hits teach you its attacks (`attack`). `~` knowledge menus.
- Health bar of the tracked monster is in the sidebar; `[----------]` or nothing means **not in view**, not dead.

## 5. Fight (`command`, `attack`, c4 4.11)
- Melee: walk into it. Blows per turn depend on STR, DEX and weapon weight (`I`nspect shows them). A hit that does no damage counts as a miss; armour class lowers both (`attack`).
- **Throw** `v`, the item, then a target. At the `Direction?` prompt: `'` = closest monster, `5` or `t` = the current target, `*` = choose (Space next, `t` select, Esc/`q` quit), a digit = that direction (`playing`, Targeting). `'` alone, as a command, targets the closest monster. Fire: `f`, ammo, target; **`h`** (or Tab) fires the first quiver ammo at the nearest visible monster.
- **Flask of oil** FACT (`object`, c4 4.11 from `src/player-attack.c`): 1d4 x3 (throwing weight multiplier) x3 (EXPLODE) = **9 to 36 per hit, about 22**, damage that fire-resistant monsters take too. Range up to 10 squares. OBSERVED, one run each: one flask killed each of the depth-6 `p` humans it met - apprentice, gallant, tamer (sonnet-2#124, sonnet-2#131, sonnet-2#145); and Fang died at character level 2 to two flasks in a corridor and then melee, which cost 20 of 31 hit points (sonnet-1#84, sonnet-1#90). Daggers and Spears are `THROWING` too (x3 and x6 on dice).
- Spells: `b` browse, `G` study (new spells show "Study" on the status line; **Priests and Paladins** are given a random one, others choose), `m` + book + spell casts. First cast of each spell pays experience (the last field of a `class` `spell:` line). Casting with too little mana is allowed and can paralyse you and drain CON (c14 0.2): do not.
- Early dangers, `monster`, hit points / worst melee per **your** turn (a +10 speed monster acts twice):
  - **floating eye `e`** (DL1, 11 hp, never moves): its gaze **paralyses**. Never melee it; oil from range or ignore. Same for grey mushroom patch `,` (confuses) and yellow mushroom patch `,` (terrifies): they never move, do not stand next to them.
  - small kobold `k` 8 hp, 1d5; kobold 12 hp, 1d8; large kobold (DL5) 65 hp, 1d10; kobold archer (DL4) 24 hp.
  - **Fang and Grip** (DL2 uniques, +10 speed, 25 hp, bite 1d4: up to 8 per turn); wild dogs `C` come in packs (3 hp each, DL1).
  - **cave spiders `S`** (DL2, +10 speed, 7 hp, 1d4, **swarm** of 2d8 that surrounds you): fight in a corridor or leave.
  - soldier `p` DL2, 23 hp, 2 x 1d7 (up to 14); apprentice (blinks, blinds, magic missile), acolyte, cutpurse (steals gold) `p`; kobold shaman DL3 casts; black naga DL3, 27 hp.
  - **Bullroarer** DL5, +10 speed, 60 hp, 2 x 1d8 (up to 32 per turn). Smeagol (DL3, +20, invisible) steals gold by touch.
  - Breeders (`w` worm masses, `r` mice and rats; flag MULTIPLY): do not fight them in the open; leave.
  - Stationary jellies `j` and molds `m` cannot follow you. White jelly (DL2, 36 hp, poison touch) is 10 exp x level 2 / your level: **UNTESTED** idea, oil from range. Yellow jelly drains mana.
- **POLICY** c15 FIGHT-02: do not stay in reach of a monster whose worst turn is at least your HP minus a reserve (max(20, 10% of max HP); under 20 max HP the reserve exceeds you, so the rule cannot guide the first levels). FIGHT-03: fight groups in a corridor or doorway, not in a room. Chapter 14 adds: do not fight breeders in the open.

## 6. Escape (c13, c15 ESC)
- Standing on a staircase: take it (one action). Phase Door (15 g): teleports about 10 squares; Cure Light Wounds: heals 20, cures blindness, cuts 20 and confusion 20 turns (`object`). Potions work blind or confused; scrolls do not.
- **POLICY** c15 ESC-01 ladder, in order: a staircase under you; Phase Door if one monster is the problem; Teleportation; heal; an unknown potion only as the last rung. HEAL-01: heal on predicted danger, the smallest heal that makes you safe; HEAL-04: if the monster out-damages your best heal per turn, leave instead.
- Do not treat Word of Recall (15-34 turns delay) or Deep Descent (3+1d4 turns) as an escape (c15 ESC-04). Those are *player* turns: FACT, `src/game-world.c` decrements the recall counter once every ten game turns, so the wait is 150-340 game turns, and a run that gave up on it after about forty had waited a quarter of the minimum. Read in town before you have ever left it, Recall takes you to DL1 rather than nowhere - see volume 2.
- Running from faster monsters does not work: +10 speed is double your moves (`attack`, "A note on speed").

## 6b. What to keep and what to leave

**Selling pays nothing** (`birth_no_selling` is on; `birth`, c15 ITEM-05). So an
object is worth carrying only if you will **use** it or **wear** it. Gold is
the only loot with a value. That removes most of item management: do not haul
anything to town.

**The rule that does the rest:** in `object.txt` every harmful consumable has
`cost:0`, and every useful one has a price. FACT, checked against the file:
cost 0 at `level:` 0-10 is Potion of Sleep, Blindness, Confusion, Salt Water,
Slowness, Poison, Surprise; Scroll of Summon Monster, Darkness, Aggravate
Monster, Trap Creation; Staff of Darkness, Summoning, Haste Monsters; Wand of
Heal Monster, Haste Monster. Once a kind is identified, its price in a shop
tells you which group it is in.

**Keep, in this order of usefulness to a shallow character** (in `object`, `cost:` is what the
Alchemist or General Store charges for potions, scrolls and food, and so what the
thing is worth to you; for the torch, lantern and wands it is only the base cost,
the shop prices them differently, so read the screen. `found from` is the first
depth the game can generate it at, the `alloc:` minimum in `object`; `level:` is
the object's own level and is not where you find it):

| Keep | `cost:` | `level:` | found from | why |
|---|---|---|---|---|
| Flask of oil | 3 | 1 | 1 | the ranged attack every class has; 10-20 of them |
| Potion of Cure Light Wounds | 20 | 0 | 1 | the heal you will actually reach for |
| Scroll of Phase Door | 15 | 1 | 1 | one monster is the problem; short hop |
| Ration of Food | 3 | 0 | 0 | two or three; starvation has killed a run |
| Wooden Torch / Lantern | 2 / 35 | 1 / 3 | 1 / 5 | you cannot read scrolls in the dark |
| Scroll of Magic Mapping | 65 | 5 | 5 | maps within 22 rows and 40 columns of you - but one run paid 65 and saw no staircase |
| Potion of Cure Serious Wounds / Cure Critical Wounds | 70 / 200 | 3 / 5 | 5 / 12 | when Light stops being enough |
| Scroll of Teleportation | 40 | 10 | 10 | the escape when Phase Door is not far enough |
| Scroll of Deep Descent | 250 | 10 | 1 | not sold; five levels below your deepest |
| Scroll of Word of Recall | 125 | 5 | 5 | useless until you have been deep; it does nothing from the town |
| Potion of Speed / Heroism / Berserk Strength | 75 / 25 / 80 | 1 / 1 / 3 | 1 / 1 / 3 | drink before a fight you have chosen |
| Wand of Magic Missile / Stinking Cloud | 200 / 400 | 3 / 5 | 3 / 5 | ranged damage for a class with no spell |

**Leave** everything with cost 0, and everything you will not wear: a second
weapon, armour too heavy for your class's spell allowance, a ring or amulet
whose rune you have not identified and cannot afford to test.

**An unknown potion or scroll is worth trying** rather than carrying, because
first use pays experience and selling pays nothing - what the first use pays,
and when to risk it, is under "Keys as the game receives them" above.

## 7. Recover
- `R` then `&` then Enter rests until healed or disturbed; `R` `*` Enter until HP and SP are full; or give a number. Resting doubles regeneration; poison, stun, cuts stop it (c15 CURE-01, REST-01).
- **POLICY** c15 REST-01/02: rest only when no awake monster can reach you in about 5 turns; rest on a staircase when one is within about 10 steps.
- Blackguards regenerate HP at half rate; Necromancers and spell-users rest for mana.

- **A trap door is not a way down, it is how you lose control of your descent.**
  `trap` gives it 2d8 damage (2-16, and a level-1 Mage has 7-12 hit points and a
  level-1 Druid has 9-14 hit points) and it drops you one level **on no staircase**. That second part is
  the expensive one: arriving off stairs forfeits the connected-stairs escape
  and the two-ask reroll (`< ` then `> `) that every run which survived has
  leaned on, and you land in an unexplored level with no known way back up.
  Recognise it - shown `^`, named by `l`ook - and walk around it. `W`+digit
  steps onto a trap deliberately; there is no reason to. If you fall through
  one anyway, find a staircase before you pick a fight.

## 8. Descend
- Every level has down staircases (`command`) but you must find one. Ways down, all FACT unless marked:
  0. **`l>g` then `>`** — the one to use; see the top of this page;
  1. a `>` you walk onto and press `>` on;
  2. Scroll of **Deep Descent** (`object`: found from DL1, **not sold**): after 3+1d4 turns you drop to **5 levels below your deepest level reached**, not below where you stand (`src/effect-handler-general.c`), landing on no staircase;
  3. Scroll of Magic Mapping (65 g, Alchemist when stocked; found from DL5) maps everything within 22 rows and 40 columns of you, a radius each side (`object` `effect-yx`; `src/effect-handler-general.c` MAP_AREA);
  5. Word of Recall (in your kit unless that option is on): read in the dungeon it takes you to town, read in town it returns you to your deepest depth after 15-34 turns (c15 TOWN-05).
- **POLICY** c15 DEPTH-02, minimum kit before going below: DL5 needs Word of Recall, 3+ cure potions and a ranged attack; DL15 needs 100 max HP and about 10 Phase Door. DEPTH-01 says gate depth on the worst single hit of the depth band, not on character level. Treat these as numbers to test.

## 9. Your class (`class`, `race`, c14; section numbers are c14's)
Level-1 facts are from `class.txt`; "buy" is POLICY from c4/c14/c15 with real prices from section 2.

**Warrior** (c14 s1): hit die 9, up to 6 blows, no spells, fear-proof only from CL30 (before that fear stops melee: Boldness 8 g, Berserk Strength). Kit: Dagger, Soft Leather Armour, Berserk Strength, food, torches. Buy: oil 15-20, Cure Light Wounds, Phase Door; a Sling or Short Bow if gold is left. Kills it: paralysis (floating eye), fear, open-room packs, running out of escapes.

**Mage** (s2): hit die 0 (HP = race die: **10** for a Human), INT, 30 lb armour. Learns **one** spell at CL1 (`G`, book, spell). CL1 spells: Magic Missile 1 sp (3d4, never resisted), Light Room 2 sp, Find Traps/Doors/Stairs 1 sp (detects within 22 rows and 40 columns). Phase Door spell at CL2, Detect Monsters CL3, Fire Ball CL6. Kit: Rapier, [First Spells], food, torch. Buy: oil (your real attack), Cure Light Wounds, Phase Door scrolls. Kills it: one round of melee, casting at 0 mana, anything adjacent at 10 HP.

**Druid** (s3): hit die 2, WIS, no teleport spell ever. CL1: Detect Life 1 sp, Fox Form 2 sp (double move, but a shape cannot read, quaff or cast: change back first); Remove Hunger CL2 (no food needed), Stinking Cloud CL3 (2 sp, poison ball, your gun). Kit: Whip, [Lesser Charms]. Buy: oil (nothing else hits hard until CL3), Cure Light Wounds, Phase Door. Kills it: no escape when surrounded, poison-immune targets.

**Priest** (s4): hit die 2, WIS, random spell learning. CL1 spells: Call Light, Detect Evil, Minor Healing (2 sp), Bless (2 sp, +5 AC, +10 to-hit). Kit: Mace (blessed +2/+2), **2 Cure Serious Wounds**, [Novice's Handbook]. Orb of Draining needs CL7 and [Cleansing Power] (kind cost 100, Bookseller). No escape spell until a DL30+ book. Buy: oil, Phase Door scrolls, Cure Light Wounds. Kills it: no escapes, non-evil animals, blindness or confusion stopping prayers.

**Necromancer** (s5): hit die 2, INT, **sees without light and must not carry one**: a lit grid adds 25% spell failure; at CL1-5 you see only adjacent squares in the dark. Nether Bolt 1 sp (3d4; undead immune, evil half), Sense Invisible CL2, Create Darkness and Bat Form CL3. Kit: Main Gauche, [Into the Shadows], food; no torch. Buy: oil, Cure Light Wounds, Phase Door; **no torch**. Kills it: fighting in a lit room, undead, 10-HP fragility.

**Paladin** (s6): hit die 6, 5 blows, random spell learning. CL1: Bless only (Detect Evil CL3, Call Light CL5, Minor Healing CL7, Heroism CL12, full effect CL15, for fear). Kit: Main Gauche, [Novice's Handbook], Protection from Evil scroll. Buy: oil, Cure Light Wounds, Phase Door. Kills it: as the Warrior, plus fear before CL12 and mana starvation.

**Rogue** (s7): hit die 4, best throwing skill (72) and stealth, first spell at **CL5** (none now; Phase Door spell CL7, 55% base failure: carry scrolls). `s` + digit steals from an adjacent monster, best while it sleeps. Kit: Dagger, Soft Leather Armour, [First Spells]. Buy: oil, Phase Door, Cure Light Wounds, a Sling; throw Daggers and Spears. Kills it: waking a pack, early melee at low HP.

**Ranger** (s8): hit die 5, best shooting skill (72), extra shots; first spell at **CL3** (Remove Hunger). Kit: Main Gauche, Short Bow, 15-20 Arrows, [Lesser Charms]. `h` fires at the nearest; arrows break. Buy: arrows (priced by power, believed about 1 g each: UNTESTED; 40+), oil, Cure Light Wounds, Phase Door. Kills it: out of arrows, fast melee in a corridor, archers.

**Blackguard** (s9): hit die 8, 5 blows, heavy weapons, 60 lb armour. Spell points are **earned by fighting and being hurt**, decay when idle, and casting heals; HP regenerates at **half** rate. CL1: Seek Battle 1 sp (little use); Berserk Strength CL3, Whirlwind CL5. Kit: Tulwar (2d4), Leather Shield, Cure Light Wounds, [Into the Shadows]; **starting gold is nearly gone**. Buy: whatever Cure Light Wounds and oil it covers. Kills it: starting a fight already hurt, ranged attackers.

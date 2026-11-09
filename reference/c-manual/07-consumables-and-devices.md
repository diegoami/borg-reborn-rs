# 7. Consumables and Devices

Every potion, scroll, food item, mushroom, flask, staff, wand and rod in the game, with numbers
resolved from the effect code; device skill and failure rates; charges, recharging and rod
timeouts; activations of artifacts, rings and dragon armour; pack and quiver limits; element
damage to carried items; and standard carry kits per phase with the stock source of each item.

FACT statements cite the game source (angband master `e473720ed`, paths relative to the source
root). ADVICE is marked and is judgement. Effect handlers live in
`src/effect-handler-general.c` and `src/effect-handler-attack.c`; item data in
`lib/gamedata/object.txt`; activations in `lib/gamedata/activation.txt`.

## Contents

1. [Reading the numbers](#1-reading-the-numbers)
2. [Healing and curing](#2-healing-and-curing)
3. [Escape and movement](#3-escape-and-movement)
4. [Detection and mapping](#4-detection-and-mapping)
5. [Attack items](#5-attack-items)
6. [Monster-status and area-control items](#6-monster-status-and-area-control-items)
7. [Buffs and temporary resistances](#7-buffs-and-temporary-resistances)
8. [Stat gain, experience and restoration](#8-stat-gain-experience-and-restoration)
9. [Enlightenment, identification and enchantment](#9-enlightenment-identification-and-enchantment)
10. [Utility](#10-utility)
11. [Food](#11-food)
12. [Mushrooms](#12-mushrooms)
13. [Bad items (do not use, ignore on sight)](#13-bad-items-do-not-use-ignore-on-sight)
14. [Device skill and failure rates](#14-device-skill-and-failure-rates)
15. [Charges, recharging, rod timeouts](#15-charges-recharging-rod-timeouts)
16. [Aiming: bolts, beams, balls, arcs](#16-aiming-bolts-beams-balls-arcs)
17. [Activations](#17-activations)
18. [Stacking, pack and quiver limits](#18-stacking-pack-and-quiver-limits)
19. [Element damage to carried items](#19-element-damage-to-carried-items)
20. [Standard carry kits per phase](#20-standard-carry-kits-per-phase)
21. [Things to try (untested)](#things-to-try-untested)

Related: [Chapter 1](01-core-mechanics.md) (device skill, food, light),
[Chapter 5](05-elements-and-status.md) (timed effects, cures), [Chapter 6](06-equipment.md)
(equipment activations), [Chapter 11](11-town-and-shopping.md) (prices),
[Chapter 13](13-escapes-and-emergencies.md) (emergency procedures). Full kind table:
[A6](appendices/A6-object-kinds.md). The official manual covers the commands to use items
(`q`uaff, `r`ead, `E`at, `u`se staff, `a`im wand, `z`ap rod, `A`ctivate, `F`uel, `v` throw) in
[user-manual/command.rst](user-manual/command.rst).

---

## 1. Reading the numbers

* **Depth** in tables is the item's allocation range (`alloc:` line): where it is generated in the
  dungeon. **Lvl** is its kind level, which sets device difficulty (14) and recharge difficulty.
* **Cost** is the kind's base cost (`cost:`), which is the normal-store price for consumables;
  the Black Market charges about 3x (see [Chapter 11](11-town-and-shopping.md)).
* **Store** says where it is sold: "Alch-A" = Alchemist always, "Alch" = Alchemist sometimes,
  "Gen-A" General Store always, "Gen" sometimes, "Magic" = Magic Shop sometimes, "BM" = only the
  Black Market (which can stock almost anything at depth), "-" = dungeon/BM only.
  Source: `lib/gamedata/store.txt`.
* Healing `300+m35` means "300 HP, or 35% of your *missing* HP if that is larger"
  (`effect_handler_HEAL_HP()`).
* Damage from devices is boosted by `max((device_skill - item_level)/2, 0)` percent
  (`src/cmd-obj.c:use_aux()`, `effect_calculate_value()`). A Mage with skill 100 using a level-30
  wand gets +35% damage.
* Timed durations are in player turns at normal speed. Food values are percent of the food meter
  (`lib/gamedata/player_timed.txt` scaled by `food-value`).

---

## 2. Healing and curing

### 2.1 Potions

| Potion | Effect | Lvl | Depth | Cost | Store |
|--------|--------|-----|-------|------|-------|
| Cure Light Wounds | heal 20, cure blindness, cuts -20, confusion -20, food +1% | 0 | 1-15 | 20 | Alch-A |
| Cure Serious Wounds | heal 40, cure blindness, cuts, confusion | 3 | 5-40 | 70 | Alch |
| Cure Critical Wounds | heal 60, cure blindness, cuts, poison, stun, amnesia, confusion | 5 | 12-100 | 200 | Alch |
| Healing | heal 300 or 35% of missing, all the CCW cures | 15 | 30-100 | 800 | BM |
| *Healing* | heal 1200, same cures | 40 | 40-100 | 1500 | BM |
| Life | heal 5000, restore experience and all stats, cure everything incl. hallucination | 60 | 60-100 | 5000 | BM |
| Neutralize Poison | cure poison | 5 | 1-35 | 75 | Alch |
| Restore Mana | restore all SP | 25 | 15-100 | 350 | BM |

Notes (FACT): CLW only *reduces* cuts and confusion by 20 (deep cuts and long confusion
survive it); CSW cures them fully. Only CCW and better cure **stun**, which blocks most actions
when heavy. Healing's 35% clause matters only when you are missing more than ~860 HP.

### 2.2 Staves, rods, others

| Item | Effect | Lvl | Depth | Charges / recharge | Cost | Store |
|------|--------|-----|-------|--------------------|------|-------|
| Staff of Cure Light Wounds | as the CLW potion | 5 | 5-100 | 6+d5 | 350 | - |
| Staff of Curing | cure blind, cut, poison, stun, confusion (no HP) | 25 | 10-100 | 4+d3 | 500 | - |
| Staff of Healing | heal 300 or 35% missing + CCW cures | 70 | 70-100 | 1+d2 | 5000 | - |
| Staff of Holiness | dispel evil 120, heal 50, prot. from evil, cures everything | 70 | 70-100 | 2d2 | 3500 | - |
| Staff of the Magi | restore INT and all SP | 70 | 70-100 | 2+1d2 | 4500 | - |
| Rod of Curing | as the staff | 40 | 10-80 | 100 turns | 1000 | - |
| Rod of Healing | heal 500, cure cut and stun | 80 | 80-100 | 1000 turns | 20000 | - |
| Rod of Restoration | restore all stats and experience | 80 | 80-100 | 1000 turns | 25000 | - |
| Mushroom of Fast Recovery | heal 30, cure stun, cut, poison, blind | 0 | 5-40 | - | 200 | - |
| Mushroom of Emergency | heal 200, but hallucination and temp rFire/rCold 25+5d5 | 0 | 5-100 | - | 60 | - |
| Piece of Elvish Waybread | food to 75%, cure poison, heal 4d8 | 5 | 30-100 | - | 10 | - |

ADVICE:

* CLW is the early-game heal and cure for blindness. Keep 5+ until about DL15, then CSW/CCW.
  CCW is the workhorse from DL10 to DL40: it fixes stun, poison, confusion and blindness in one
  quaff.
* Mushrooms of Emergency are strong: 200 HP at any depth. The downside is hallucination, which
  garbles the monster display and cuts device skill by 20% but does **not** stop reading
  (`src/player-util.c:player_can_read()` blocks only blindness, no light, confusion and amnesia).
  Carry a few deep as a second-tier heal.
* Healing potions should be bought from the BM whenever affordable after DL30.
* A Rod of Curing is a permanent cure for confusion/blindness/stun/poison. Scrolls cannot be read
  while blind, in the dark, confused or amnesiac (`player_can_read()`), but potions, staves, rods
  and wands have no such check: they work while blind or confused (confusion only randomises the
  direction of *aimed* effects, `player_confuse_dir()` in `use_aux()`). So the cure for "blind and
  confused" is a potion or a rod/staff, never a scroll.

---

## 3. Escape and movement

| Item | Effect | Lvl | Depth | Cost | Store |
|------|--------|-----|-------|------|-------|
| Scroll of Phase Door | teleport ~10 grids (dice 10, +-25%) | 1 | 1-100 | 15 | Alch-A |
| Scroll of Teleportation | teleport 60% of the farthest edge distance (M60) | 10 | 10-100 | 40 | - (dungeon/BM) |
| Scroll of Teleport Level | random up or down one level (only down from town; never up with force_descend) | 20 | 20-100 | 800 | - |
| Scroll of Deep Descent | after 3+1d4 turns, drop to **max depth + 5** (`dungeon_get_next_level(max_depth, 5)`) | 10 | 1-100 | 250 | - |
| Scroll of Word of Recall | after 15+1d20 turns, go to town (or from town to recall depth) | 5 | 5-100 | 125 | Alch-A |
| Staff of Teleportation | as the scroll | 20 | 20-100 | 2000 (5+d4 charges) | Magic |
| Rod of Recall | as Word of Recall | 30 | 30-100 | 4000 (60 turns) | - |
| Wand of Teleport Other | bolt: teleports a monster away (distance 100) | 20 | 20-100 | 3500 (6+d5) | - |
| Rod of Teleport Other | same | 35 | 45-100 | 5000 (25 turns) | - |
| Mushroom of Sprinting | haste (+10) for 100 turns, then slowness | 0 | 15-100 | 100 | - |
| Mushroom of Terror | TERROR 25+5d5: +10 speed, cannot attack | 0 | 5-40 | 60 | - |

FACT details:

* Teleport distance (`effect_handler_TELEPORT()`): with `M60`, distance = 60% of the larger of the
  vertical and horizontal distances from you to the far edge. On a full 198x66 level that is about
  59 grids from the centre and up to ~118 from an edge; then +-25% random. Phase Door uses a
  fixed 10 (+-25%). The landing grid is the one closest to the wanted distance; vault grids are
  avoided when possible.
* Teleport fails on a no-teleport grid (vault squares) for distances over 10, and always with
  the NO_TELEPORT curse ("Teleportation forbidden!").
* Recall from the dungeon: if you are above your max depth and levels don't persist, you are asked
  whether to reset recall depth to the current depth (`effect_handler_RECALL()`). Reading again
  while active cancels it. Recall does nothing with `birth_no_recall` or in single-combat arenas.
* **Deep Descent targets five levels below your *maximum* depth, not your current depth**
  (`src/effect-handler-general.c:effect_handler_DEEP_DESCENT()` and the delayed handler in
  `src/game-world.c`). At DL10 with max depth 30 it drops you to DL35. If no deeper level is
  possible (quest level or bottom) you instead take a Destruction-style blast.
* Teleport Level picks up or down at random (50/50) when both are allowed; down only from town or
  with `birth_force_descend`; you cannot go down from a quest level.

ADVICE: see [Chapter 13](13-escapes-and-emergencies.md) for decision rules. Short version: Phase
Door to break line of sight from a melee monster, Teleportation to leave a fight, Teleport Level
to leave the level now, Deep Descent only when you *want* to be deeper (never as a panic button
unless your max depth is near your current depth).

---

## 4. Detection and mapping

FACT: "local area" detections use `effect-yx:22:40`, i.e. 22 rows above and below and 40 columns
either side of you (about one screen). The Rod of Magic Mapping uses 33:55.

| Item | Effect | Lvl | Depth | Cost/charges | Store |
|------|--------|-----|-------|--------------|-------|
| Scroll of Magic Mapping | map local area (walls, doors, stairs) | 5 | 5-100 | 65 | Alch |
| Scroll of Treasure Detection | detect gold/ore, sense objects | 0 | 0-70 | 70 | - |
| Scroll of Detect Invisible | detect invisible monsters | 1 | 1-70 | 15 | Alch |
| Scroll of Light | light room + radius-2 weak light (2d8) | 0 | 0-60 | 15 | - |
| Staff of Detect Invisible | detect invisible monsters | 5 | 5-100 | 8+d15 | Magic |
| Staff of Detect Evil | detect evil monsters | 20 | 20-100 | 8+d15 | Magic |
| Staff of Mapping | map local area | 20 | 10-100 | 5+d5 | Magic |
| Staff of Light | as the scroll | 5 | 5-80 | 8+d20 | Magic |
| Rod of Treasure Location | detect gold/ore, sense objects | 10 | 8-75 | 50 turns | - |
| Rod of Detection | gold, objects, visible and invisible monsters | 30 | 50-100 | 100 turns | - |
| Rod of Magic Mapping | map a larger area (33x55 half-extents) | 35 | 35-100 | 100 turns | - |
| Rod of Probing | probe all visible monsters (reveals stats, resists) | 30 | 35-100 | 100 turns | - |
| Rod of Illumination | light area radius 2 | 20 | 10-100 | 30 turns | - |
| Rod of Light | lighting beam 6d8 | 10 | 5-100 | 9 turns | - |
| Potion of Enlightenment | map and light the whole level | 25 | 25-100 | 800 | - |
| Mushroom of Second Sight | cure blind, telepathy 50+7d7 | 0 | 5-100 | 200 | - |

ADVICE: Magic Mapping scrolls are cheap (65) and stocked by the Alchemist; map on arrival to find
stairs. The Staff of Detect Evil is the best early monster detection you can buy (most dangerous
monsters are evil). Rod of Detection plus Rod of Magic Mapping is the late-game detection kit.

---

## 5. Attack items

### 5.1 Wands (aim; 20% base beam chance for BOLT_OR_BEAM, `src/cmd-obj.c:beam_chance()`)

| Wand | Effect | Lvl | Depth | Charges | Cost | Store |
|------|--------|-----|-------|---------|------|-------|
| Magic Missile | bolt/beam missile 3d4 | 3 | 3-60 | 6+d10 | 200 | Magic |
| Stinking Cloud | poison ball 12, radius 3 | 5 | 5-60 | 6+d8 | 400 | Magic |
| Lightning Bolts | beam elec 6d6 | 15 | 15-60 | 6+d8 | 600 | - |
| Frost Bolts | bolt cold 12d8 | 20 | 20-60 | 6+d5 | 800 | - |
| Acid Bolts | bolt/beam acid 10d8 | 30 | 30-60 | 6+d8 | 950 | - |
| Fire Bolts | bolt/beam fire 12d8 | 30 | 30-60 | 6+d8 | 1000 | - |
| Lightning Balls | elec ball 64, r2 | 35 | 35-100 | 4+d8 | 1200 | - |
| Cold Balls | cold ball 100, r2 | 40 | 40-100 | 2+d6 | 1500 | - |
| Acid Balls | acid ball 120, r2 | 50 | 50-100 | 2+d5 | 1650 | - |
| Fire Balls | fire ball 144, r2 | 55 | 50-100 | 2+d4 | 1800 | - |
| Dragon's Frost | cold arc 160 | 50 | 50-100 | 1+d3 | 2400 | - |
| Dragon's Flame | fire arc 200 | 55 | 50-100 | 1+d3 | 2400 | - |
| Dragon's Breath | arc of one of acid/elec/fire/cold/poison 160 | 60 | 60-100 | 1+d3 | 2400 | - |
| Drain Life | 150 to living monsters | 50 | 50-100 | 3+d3 | 1200 | - |
| Annihilation | 250 to living monsters | 60 | 60-100 | 1+d2 | 3000 | - |
| Stone to Mud | destroy a wall; 20+1d30 to wall monsters | 10 | 10-100 | 6+d10 | 300 | Magic |
| Light | weak-light beam 6d8 (hurts light-sensitive monsters) | 3 | 3-60 | 6+d10 | 200 | Magic |
| Darkness | dark beam 6d8 | 3 | 3-60 | 6+d10 | 100 | - |
| Wonder | a random wand effect, scaled by level (`effect_handler_WONDER()`) | 3 | 3-60 | 8+d15 | 250 | - |

### 5.2 Rods (attack)

Rods beam BOLT_OR_BEAM effects 10% of the time.

| Rod | Effect | Lvl | Depth | Recharge | Cost |
|-----|--------|-----|-------|----------|------|
| Lightning Bolts | beam elec 6d6 | 20 | 20-80 | 11 | 2000 |
| Frost Bolts | bolt cold 12d8 | 25 | 25-80 | 13 | 2500 |
| Fire Bolts | bolt/beam fire 12d8 | 35 | 30-80 | 15 | 3000 |
| Acid Bolts | bolt/beam acid 12d8 | 40 | 40-80 | 12 | 3500 |
| Lightning Balls | elec ball 64 | 55 | 55-100 | 23 | 4000 |
| Cold Balls | cold ball 100 | 60 | 60-100 | 25 | 4500 |
| Acid Balls | acid ball 120 | 70 | 70-100 | 27 | 5500 |
| Fire Balls | fire ball 144 | 75 | 75-100 | 30 | 5000 |
| Drain Life | 150 to living | 75 | 75-100 | 23 | 3600 |

### 5.3 Staves and scrolls (area attacks)

| Item | Effect | Lvl | Depth | Charges | Cost |
|------|--------|-----|-------|---------|------|
| Staff of Dispel Evil | 60 to all evil in line of sight | 50 | 50-100 | 4+d3 | 1200 |
| Staff of Power | 120 to all monsters in LOS | 70 | 70-100 | 1+d3 | 4000 |
| Staff of Holiness | 120 to evil in LOS + heal + cures | 70 | 70-100 | 2d2 | 3500 |
| Staff of Starlight | light beams in all directions 6d8 | 20 | 20-80 | 6+d5 | 800 |
| Staff of Earthquakes | earthquake radius 10 | 40 | 40-100 | 3+d5 | 350 |
| Staff of *Destruction* | destroy radius 15 | 50 | 50-100 | 1+d3 | 6000 |
| Scroll of Dispel Undead | 60 to undead in LOS | 40 | 40-100 | - | 200 |
| Scroll of *Destruction* | destroy radius 15 (blinds you unless rLight) | 40 | 40-100 | - | 1000 |
| Potion of Dragon Breath | breathe fire or cold (your choice) 80, arc | 0 | 40-100 | - | 100 |
| Flask of oil (thrown) | 1d4 x3 "explosion" damage, not fire-typed | 1 | 1-100 | - | 3 |

FACT on *Destruction* (`effect_handler_DESTRUCTION()`): every grid within 15 except permanent walls
and stairs is turned to rubble/walls/floor; all monsters there are deleted (including uniques,
which are not killed and can reappear); objects are destroyed (artifacts lost if known or with
`birth_lose_arts`); you are blinded 10+1d10 unless you resist light; it does nothing in town.

FACT on flasks (`src/player-attack.c`): a thrown flask's damage is multiplied by 3 after criticals
(`OF_EXPLODE`), it always breaks, and it uses throwing skill. At CL1 this is roughly 3-12 damage a
throw at 3 gold each, the best early ranged attack for non-archers.

ADVICE: early, a Wand of Magic Missile or Stinking Cloud from the Magic Shop is a big upgrade for
classes with decent device skill (Mage, Necromancer, Rogue). Ball wands do not need to hit and
can be aimed at a grid to catch groups.

---

## 6. Monster-status and area-control items

| Item | Effect (power) | Lvl | Depth | Charges/recharge | Cost | Store |
|------|----------------|-----|-------|------------------|------|-------|
| Wand of Slow Monster | slow (10+1d20) | 15 | 5-100 | 2+d6 | 500 | Magic |
| Wand of Confuse Monster | confuse (5+1d5) | 5 | 5-100 | 2+d6 | 500 | Magic |
| Wand of Hold Monster | hold/paralyse (3+1d5) | 5 | 5-100 | 2+d6 | 500 | - |
| Wand of Stun Monster | stun (5+1d5) | 5 | 5-100 | 2+d6 | 500 | - |
| Wand of Scare Monster | frighten (10) | 10 | 10-100 | 3+d5 | 500 | - |
| Wand of Polymorph | polymorph (power = your level) | 20 | 20-60 | 6+d8 | 400 | - |
| Wand of Disable Traps | destroy traps in a line | 20 | 20-60 | 4+d5 | 700 | - |
| Rod of Slow Monster | slow | 40 | 30-100 | 40 turns | 1500 | - |
| Rod of Hold Monster | hold | 30 | 30-100 | 40 turns | 1500 | - |
| Rod of Polymorph | polymorph | 35 | 35-80 | 25 turns | 1200 | - |
| Rod of Disable Traps | destroy traps | 35 | 10-100 | 30 turns | 2100 | - |
| Staff of Sleep Monsters | sleep all in LOS (power 10*clvl+500) | 10 | 10-100 | 6+d5 | 700 | - |
| Staff of Slow Monsters | slow all in LOS | 20 | 10-100 | 4+d4 | 800 | - |
| Staff of Confuse Monsters | confuse all in LOS | 10 | 10-100 | 3+d3 | 800 | - |
| Scroll of Monster Confusion | your next melee hit confuses the target | 5 | 5-60 | - | 30 | Alch |
| Scroll of Banishment | remove all non-unique monsters of one symbol on the level; 1d4 damage to you per monster | 40 | 40-100 | - | 2500 | - |
| Scroll of Mass Banishment | remove all non-uniques within 20 grids; 1d3 per monster | 50 | 50-100 | - | 3000 | - |
| Staff of Banishment | as the scroll | 70 | 70-100 | 1+d2 | 5000 | - |
| Scroll of Rune of Protection | glyph of warding under you | 60 | 70-100 | - | 500 | - |

Monster status effects (what hold/stun/confuse/slow/scare do) are listed in the official manual,
[user-manual/attack.rst](user-manual/attack.rst) ("Monster status effects"), and in
[Chapter 4](04-combat.md). Uniques and many deep monsters resist.

ADVICE: Banishment is the strongest "solve this level" item: banish `Z` (hounds), `D` (great
dragons), `L` (liches) or `U` (major demons) before a fight. Count the strain damage (1d4 each)
before banishing a level full of the symbol at low HP. Mass Banishment is the late-game panic
button that also clears summons.

---

## 7. Buffs and temporary resistances

FACT: timed effect strengths from `src/player-calcs.c:calc_bonuses()`; a second dose of Speed
(and other `TIMED_INC:X:0:n` effects) only extends the duration by `n` (5 for haste) instead of
the full roll (`effect_handler_TIMED_INC()`).

| Item | Effect | Duration | Lvl | Depth | Cost | Store |
|------|--------|----------|-----|-------|------|-------|
| Potion of Speed | +10 speed | 20+2d10 (re-dose: +5) | 1 | 1-100 | 75 | BM |
| Staff of Speed | as potion | | 40 | 40-100 | 2000 (4+d3) | - |
| Rod of Speed | as potion | 100-turn recharge | 55 | 80-100 | 25000 | - |
| Potion of Heroism | heal 10, cure fear, hero: +12 to-hit, pFear, device +5% | 25+1d25 | 1 | 1-100 | 25 | BM |
| Potion of Berserk Strength | heal 30, cure fear, berserk: +75 melee skill, -10 AC, pFear, device -10% | 25+1d25 | 3 | 3-100 | 80 | Alch |
| Potion of Boldness | cure fear | - | 1 | 1-90 | 8 | Alch |
| Potion of Resist Heat / Resist Cold | temp rFire / rCold | 50+1d10 | 30 | 30-100 | 30 | Alch |
| Potion of Resist Poison | temp rPois, cure poison | 50+1d10 | 20 | 20-100 | 30 | - |
| Potion of True Seeing | cure blind, see invisible | 50+2d6 | 3 | 3-40 | 50 | BM |
| Potion of Infravision | infravision +5 | 100+4d25 | 3 | 3-40 | 20 | BM |
| Scroll of Blessing | bless: +5 AC, +10 to-hit, device +5% | 6+1d12 | 1 | 1-50 | 15 | Alch |
| Scroll of Holy Chant | bless | 12+1d24 | 10 | 10-70 | 40 | Alch |
| Scroll of Holy Prayer | bless | 24+1d48 | 25 | 50-100 | 80 | - |
| Scroll of Protection from Evil | prot. from evil | 3*clvl+1d25 | 30 | 30-100 | 50 | - (Paladin starts with one) |
| Mushroom of Stoneskin | +40 AC, -5 speed | 50+7d7 | 0 | 10-40 | 60 | Gen |
| Mushroom of Clear Mind | 20 SP, cure confusion/fear/hallucination, rConf | 50+7d7 | 0 | 5-40 | 200 | - |
| Mushroom of Shadows | stealth +10 | 10+d10 | 0 | 50-100 | 300 | - |

Temporary resistance stacks with permanent resistance for "double resistance" (damage divided by
9 instead of 3 for the base elements; see [Chapter 5](05-elements-and-status.md)). Temp resists
add one resist level, capped at 2 (`calc_bonuses()`).

ADVICE: Speed potions are the most valuable consumable in the game for unique fights; hoard them
from the dungeon and buy them in the BM when cheap (225 gold observed). Berserk Strength doubles
as a 30-HP heal and fear cure for melee characters. Resist Heat/Cold potions are cheap insurance
for fire/cold breathers in DL30-45 when you lack double resists.

---

## 8. Stat gain, experience and restoration

FACT (`effect_handler_GAIN_STAT()` -> `src/player.c:player_stat_inc()`): a gain increases a stat
by 1 below 18, by a random amount (larger when lower) between 18 and 18/90, and to 18/100 above
that; it also raises the stat's maximum. There is no cap below 18/100 per potion except that.

| Item | Effect | Lvl | Depth | Cost |
|------|--------|-----|-------|------|
| Potion of Strength/Intelligence/Wisdom/Dexterity/Constitution | restore and gain that stat | 30 | 30-100 | 8000 |
| Potion of Augmentation | restore and gain all five | 40 | 40-100 | 50000 |
| Potion of Brawn / Intellect / Contemplation / Nimbleness / Toughness | gain STR/INT/WIS/DEX/CON, **lose one other random stat** (permanent drain) | 3-20 | 20-40 | 1000 |
| Potion of Experience | gain 100000 experience (`amount/2` of 200000) | 65 | 65-100 | 25000 |
| Potion of *Enlightenment* | map/light level, detect everything, gain INT and WIS | 70 | 70-100 | 12000 |
| Draught of the Ents | food to 60%, restore and gain STR and CON | 0 | 60-100 | 2000 |
| Potion of Restore Life Levels | restore drained experience | 40 | 30-100 | 400 | 
| Sip of Miruvor | restore STR and CON | 0 | 10-50 | 100 |
| Mushroom of Vigor | restore all five stats | 0 | 20-100 | 200 |
| Mushroom of Purging | restore STR/CON, cure poison, but food set to 5% (Weak) | 0 | 15-100 | 60 |

Restore Life Levels is in the Alchemist's possible stock (`lib/gamedata/store.txt`).

FACT on the Brawn family (`effect_handler_LOSE_RANDOM_STAT()` -> `src/player.c:player_stat_dec()`
with `permanent = true`): the lost stat is chosen at random among the other four, the loss lowers
both current and **maximum** (10 points above 18/10, to 18 from 18/01-18/10, else 1 point), and
**sustains do not protect against it** (neither function checks sustain flags). A later Potion of
that stat or Augmentation restores and regains it.

ADVICE: drink stat potions immediately (they cannot be stolen or destroyed afterwards). Brawn-type
potions are a gamble: gain your important stat, lose a random other permanently; good for a
Warrior drinking Brawn (losing INT/WIS is cheap), bad for a caster drinking Toughness.

---

## 9. Enlightenment, identification and enchantment

| Item | Effect | Lvl | Depth | Cost | Store |
|------|--------|-----|-------|------|-------|
| Scroll of Identify Rune | learn one unknown rune on one item | 1 | 5-100 | 200 | - |
| Potion of Enlightenment | map and light the entire level | 25 | 25-100 | 800 | - |
| Scroll of Enchant Weapon To-Hit / To-Dam | +1 attempt to-hit / to-dam | 15 | 15-70 | 125 | BM |
| Scroll of Enchant Armour | +1 attempt to-AC | 15 | 15-70 | 125 | BM |
| Scroll of *Enchant Weapon* | 1d3 attempts each on to-hit and to-dam | 50 | 50-100 | 500 | - |
| Scroll of *Enchant Armour* | 2+1d3 attempts to-AC | 50 | 50-100 | 500 | - |
| Scroll of Remove Curse | strength 20+d20 | 10 | 10-100 | 100 | Alch-A |
| Scroll of *Remove Curse* | strength 50+d50 | 50 | 50-100 | 8000 | - |
| Staff of Remove Curse | strength 35+d30 | 40 | 40-100 | 1500 | - |
| Scroll of Acquirement | create 1 great item | 20 | 20-100 | 5000 | - |
| Scroll of *Acquirement* | create 1+1d2 great items | 60 | 60-100 | 10000 | - |

FACT on enchanting (`src/effect-handler-general.c:enchant()`, `enchant_score()`): each attempt on
a bonus currently at `+n` fails with probability `enchant_table[n]/1000`:

| Current bonus | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15+ |
|---------------|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|-----|
| Fail % | 0 | 1 | 2 | 4 | 8 | 16 | 28 | 40 | 55 | 70 | 80 | 90 | 95 | 97 | 99 | 100 |

Artifacts resist half the time. Stacks resist (a stack of n gets the attempt with chance 1/n);
ammunition stacks are 20x easier. So enchanting to +10 is practical, beyond +12 almost hopeless.
Negative bonuses are always raised. Curses are not removed by enchanting (see
[Chapter 6](06-equipment.md#15-curses) for curse removal mechanics).

ADVICE: Enchant To-Dam scrolls on your main weapon (+1 dam per blow is big early), and Enchant
Armour on light armour pieces; ammunition takes enchantments easily as a whole stack.

---

## 10. Utility

| Item | Effect | Lvl | Depth | Cost | Store |
|------|--------|-----|-------|------|-------|
| Scroll of Remove Hunger | food to 50% | 5 | 5-100 | 10 | Alch |
| Scroll of Recharging | recharge a wand/staff, strength 6 (15) | 40 | 40-100 | 200 | Alch |
| Scroll of Door Destruction | destroy adjacent doors | 10 | 10-60 | 50 | - |
| Scroll of Deep Descent | see 3 | | | | |
| Flask of oil | lantern fuel (+7500 turns, cap 15000) or thrown weapon | 1 | 1-100 | 3 | Gen-A |
| Wooden Torch | light intensity 2 (adjacent grids), 5000 turns | 1 | 1-100 | 2 | Gen-A |
| Wand of Stone to Mud | dig a wall instantly | 10 | 10-100 | 300 | Magic |
| Ring of Digging (activation) | stone to mud, every 15+2d10 turns | 0 | 10-50 | 800 | - |

---

## 11. Food

FACT: food values are percent of the food meter (Hungry below 15%, Weak below 8%, Faint below 4%,
Fed up to 90%, Full 90-100%; above 90% you are slowed proportionally,
`src/player-calcs.c:calc_bonuses()`).

| Food | Nourishment | Other | Depth | Cost | Store |
|------|-------------|-------|-------|------|-------|
| Ration of Food | +30% | | 0-50 | 3 | Gen-A |
| Hard Biscuit | +10% | | (store only) | 1 | Gen |
| Slime Mold | +15% | | 30-100 | 2 | Gen |
| Apple | +10% | | 0-20 | 3 | - |
| Handful of Dried Fruits | +15% | | 0-50 | 3 | - |
| Slice of Meat | +40% | | 10-50 | 3 | - |
| Honey-cake | to 60% | | 20-70 | 10 | - |
| Piece of Elvish Waybread | to 75% | cure poison, heal 4d8 | 30-100 | 10 | - |
| Scrap of Flesh | random: +20%/+20%/+10% or set to 1d4% or 5+1d4% | | 50-100 | 0 | - |
| Flask of Whisky | +3% | confusion d5-1 | (store only) | 1 | Gen |
| Pint of Fine Wine | +3% | bold 79+1d41 | (store only) | 2 | Gen |
| Swig of Orcish Liquor | +5% | stun and slow 2+1d3 | 20-100 | 1 | - |
| Sip of Miruvor | +3% | restore STR, CON | 10-50 | 100 | - |
| Draught of the Ents | to 60% | restore + gain STR, CON | 60-100 | 2000 | - |

ADVICE: 3-5 Rations early, Scrolls of Remove Hunger (10 gold, light) from mid-game. Never eat
Scraps of Flesh when low on food (they can *set* your food to 1-9%). Avoid overeating: "Full"
slows you.

---

## 12. Mushrooms

FACT: mushrooms are a flavoured food type (`lib/gamedata/object.txt`); Hobbits know them all
(`KNOW_MUSHROOM`, `lib/gamedata/p_race.txt`). All add +5% food except where noted.

| Mushroom | Effect | Depth | Cost | Verdict (ADVICE) |
|----------|--------|-------|------|------------------|
| Second Sight | cure blind, telepathy 50+7d7 | 5-100 | 200 | good: free ESP before a fight |
| Fast Recovery | heal 30, cure stun/cut/poison/blind | 5-40 | 200 | good early cure |
| Vigor | restore all stats | 20-100 | 200 | keep 1-2 |
| Clear Mind | 20 SP, cure confusion/fear/hallucination, temp rConf 50+7d7 | 5-40 | 200 | casters: excellent |
| Emergency | heal 200, hallucination, temp rFire+rCold 25+5d5 | 5-100 | 60 | strong emergency heal |
| Terror | TERROR 25+5d5 (+10 speed, cannot attack) | 5-40 | 60 | escape: run away fast |
| Stoneskin | +40 AC, -5 speed, 50+7d7 | 10-40 | 60 | melee tank buff; General Store sometimes |
| Debility | restore SP, then drain STR or CON | 10-40 | 20 | casters in a pinch only |
| Sprinting | +10 speed 100 turns, then slowness | 15-100 | 100 | escape |
| Purging | food to 5%, restore STR/CON, cure poison | 15-100 | 60 | eat food right after; General Store sometimes |
| Shadows | stealth +10, 10+d10 | 50-100 | 300 | sneaking |
| Turbulence | defined but has no allocation and no effect lines | never generated | 60 | - |

---

## 13. Bad items (do not use, ignore on sight)

FACT (`lib/gamedata/object.txt`, cost 0 kinds):

* Potions: Sleep (paralysis 5+1d5), Blindness (75+4d25), Confusion, Poison, Slowness (15+1d25),
  Salt Water (food to 2%, cure poison, paralysis 4 *ignoring free action*,
  `TIMED_INC_NO_RES`), Surprise (random: scramble stats, become a bat, drain INT or WIS, or
  stoneskin), Slime Mold Juice (harmless +10% food).
* Scrolls: Aggravate Monster (wakes and hastes monsters in LOS), Summon Monster, Summon Undead,
  Darkness, Trap Creation, Curse Weapon, Curse Armour (the last two have no allocation and are not
  generated normally).
* Wands: Clone Monster, Heal Monster, Haste Monster.
* Staves: Summoning, Haste Monsters, Darkness.

Several bad potions have `attack:3d4` and break 100% when thrown (base `potion`, break 100), so
they can be thrown for a little damage. Whether the potion's own effect is applied to a monster on
impact was not verified in this version; assume it is not.

---

## 14. Device skill and failure rates

### 14.1 The formula

FACT (`src/obj-util.c:get_use_device_chance()`, rewritten in 4.2.3 and retuned in 4.2.6 per
`changes.txt`):

```
level = artifact ? artifact.level : (activation ? activation.level : kind.level)
x     = 2 * (device_skill - level) + 1
fail  = 380 - 370 * x / (5 + |x|)          (out of 1000, integer arithmetic)
```

The use succeeds unless `randint1(1000) < fail` (`src/cmd-obj.c:check_devices()`). A failed use
costs a turn. **Since 4.2.6 a failed use is automatically retried until it succeeds or you are
disturbed** (changes.txt; `use_aux()` returns "repeat" when the check failed but the rate is under
100%). A wand or staff with no charges is refused before any roll ("That staff has no
charges", `do_cmd_use_staff()`/`do_cmd_aim_wand()`), costing no turn.

| skill - level | -30 | -20 | -10 | -5 | -3 | -1 | 0 | +1 | +3 | +5 | +10 | +15 | +20 | +30 | +40 | +60 |
|---------------|-----|-----|-----|----|----|----|---|----|----|----|-----|-----|-----|-----|-----|-----|
| Fail % | 72 | 71 | 67 | 62 | 57 | 44 | 32 | 24 | 17 | 13 | 8.2 | 6.2 | 5.1 | 3.9 | 3.2 | 2.5 |

The curve never goes below ~1% or above ~75%. The steep part is within 5 points either side of the
item level: one more point of skill matters a lot there.

### 14.2 Device skill

FACT: `skill = race + class base + class increment * clvl / 10 + adj_int_dev[INT]`
(`src/player-calcs.c:calc_bonuses()`, `lib/gamedata/class.txt` `skill-device:base:increment`,
`lib/gamedata/p_race.txt`).

| Class | base | per 10 levels | | Race | modifier |
|-------|------|---------------|-|------|----------|
| Warrior | 18 | 7 | | Human | 0 |
| Mage | 36 | 13 | | Half-Elf | 3 |
| Druid | 24 | 10 | | Elf | 6 |
| Priest | 30 | 10 | | Hobbit | 18 |
| Necromancer | 36 | 13 | | Gnome | 22 |
| Paladin | 24 | 10 | | Dwarf | 9 |
| Rogue | 32 | 10 | | Half-Orc | -3 |
| Ranger | 28 | 10 | | Half-Troll | -8 |
| Blackguard | 24 | 10 | | Dunadan | 5 |
| | | | | High-Elf | 20 |
| | | | | Kobold | 5 |

INT adds 0 (INT <8), 1 (8-14), 2 (15-17), 3 (18-18/49), 4-13 above (`adj_int_dev[]`).

Observed CL1 device skill for Humans with default stats (measured with a scratch build):
Warrior 19, Mage 40, Druid 26, Priest 31, Necromancer 40, Paladin 25, Rogue 34, Ranger 30,
Blackguard 26.

Modifiers (`calc_bonuses()`): Blessed +5%, Hero +5%, Berserk -10%, confusion -25%, amnesia -20%,
hallucination -20%, poison -5%, stun -10% (heavy -20%), item-borne fear -5%, hunger penalties when
Weak/Faint.

Gnomes know all wands and rods on sight (`KNOW_ZAPPER`).

Worked examples (ADVICE uses these):

* Warrior CL20, INT 10: skill 18+14+1 = 33. Staff of Teleportation (lvl 20): diff +13, fail ~7%.
  Staff of Speed (40): diff -7, fail ~64%. Rod of Healing (80): ~72%.
* Mage CL30, INT 18/50: 36+39+4 = 79. Wand of Fire Balls (55): fail ~5%, damage +12%.

### 14.3 Rods that must be aimed

Unknown rods always ask for a direction; known rods only if their effect needs one
(`obj_needs_aim()`).

---

## 15. Charges, recharging, rod timeouts

FACT:

* **Wands and staves** have charges (`pval`). Each use spends one. Stacks share charges
  (dropping part of a stack divides them, `src/obj-util.c:distribute_charges()`).
* **Rods** have a timeout. Zapping adds the rod's `time`; it recharges by 1 per game turn tick
  (every 10 game turns, i.e. once per normal-speed player turn), and a stack of N rods recharges
  N times faster while several are charging (`recharge_timeout()` subtracts `number_charging`).
  Speed does not make rods recharge faster in real terms: at +10 speed you act twice per recharge
  tick. Equipped activatable items recharge the same way.
* **Recharging** (`effect_handler_RECHARGE()`, `src/effects.c:recharge_failure_chance()`):

```
ease     = (100 - kind_level) / 10                  (9 for level 0-9 ... 3 for level 70)
chance i = strength + ease - 2 * (charges per item) (min 1)
backfire if i <= 1 or one_in_(i): one item of the stack is destroyed
otherwise charges += 2 + d(strength / (10 - ease) + 1)
```

Scroll of Recharging has strength 6; Cloak of Thingol's activation 10.

| Device (level) | ease | Backfire at 0 charges (scroll) | at 3 charges | charges gained |
|----------------|------|--------------------------------|--------------|----------------|
| Wand of Magic Missile (3) | 9 | 1/15 (6.7%) | 1/9 (11%) | 2+d7 |
| Wand of Stone to Mud (10) | 9 | 1/15 | 1/9 | 2+d7 |
| Staff of Teleportation (20) | 8 | 1/14 (7.1%) | 1/8 (12.5%) | 2+d4 |
| Wand of Teleport Other (20) | 8 | 1/14 | 1/8 | 2+d4 |
| Staff of Speed (40) | 6 | 1/12 (8.3%) | 1/6 (17%) | 2+d2 |
| Wand of Fire Balls (55) | 4 | 1/10 (10%) | 1/4 (25%) | 2+d2 |
| Staff of *Destruction* (50) | 5 | 1/11 | 1/5 | 2+d2 |

The recharge menu shows the failure chance (`SHOW_RECHARGE`).

ADVICE: recharge only when empty (each remaining charge raises backfire risk by 2 in the
denominator). Split a valuable stack before recharging so a backfire destroys only one.
**Store recharge**: selling or giving an empty wand/staff to a store that stocks that kind
(Magic Shop: Magic Missile, Stinking Cloud, Stone to Mud, Light, Slow/Confuse Monster wands;
Teleportation, Light, Mapping, Detect Invisible, Detect Evil staves) resets its charges to a
fresh roll if higher (`src/store.c:store_carry()`); you then buy it back at full price.

---

## 16. Aiming: bolts, beams, balls, arcs

FACT (`src/effect-handler-attack.c`, `src/project.c`):

* **Bolt**: stops at the first monster; misses nothing (no to-hit roll for magic).
* **Beam**: hits every monster on the line.
* **BOLT_OR_BEAM**: beams with chance `beam% + other`; wands 20%, rods 10% (`beam_chance()`).
* **Ball**: explodes at the first monster or at the target grid; damage falls with distance from
  the centre; radius 2-3 for devices. Can be aimed at a grid to hit an invisible or out-of-LOS
  group.
* **Arc / breath** (dragon wands, DSM, Potion of Dragon Breath): a cone from you; width set by the
  effect's `other` parameter.
* **LINE** effects (Light, Stone to Mud) affect grids along a line.
* **PROJECT_LOS** effects (Dispel Evil, Power, Sleep/Slow/Confuse Monsters staves) hit every
  monster in line of sight, no aiming.

ADVICE: target with `*`/`'` and fire at a grid behind a group with balls. Use beams down corridors.
A borg should prefer balls when the target is invisible but detected.

---

## 17. Activations

FACT: activatable equipment (artifacts, DSM, elemental rings, Ring of Open Wounds, Ring of Digging,
randarts) is used with `A`. Difficulty uses `get_use_device_chance()` with the artifact level for
standard artifacts, otherwise the activation's level (`lib/gamedata/activation.txt`) or, for
DSM and rings whose effect is on the kind, the kind level. Recharge time is the item's `time`.

| Item | Activation | Device level | Recharge |
|------|-----------|--------------|----------|
| Phial of Galadriel | ILLUMINATION: light area + radius-3 light 2d15 | 5 (artifact) | 10+d10 |
| Star of Elendil | MAPPING: map area | 30 | 50+d50 |
| Arkenstone of Thráin | CLAIRVOYANCE: light/map the level | 50 | 50+d50 |
| Ring of Open Wounds | heal 40, cure blind/cut/confusion | 5 (kind) | 10+5d8 |
| Ring of Digging | stone to mud | 0 (kind) | 15+2d10 |
| Rings of Flames/Acid/Ice/Lightning | ball 80/70/75/85 r2 + temp resist 20+d20 | 30/25/28/32 | 50+d50 |
| DSMs | breath (see [Chapter 6](06-equipment.md#63-dragon-scale-mail-no-hates-ignores-all-four-base-elements)) | 25-57 | 50 |
| 'Narthanc'/'Nimthanc'/'Dethanc' | fire bolt 9d8 / cold bolt 6d8 / elec beam 6d6 | 1 | 6-16 |
| 'Paurhach' etc. (gauntlets) | elemental bolts | 1 | 5-15 |
| 'Cammithrim' | magic missile 3d4 | 10 | 2 |
| 'Belegennon' | phase door | 20 | 2 |
| Boots of Wormtongue | phase door | 10 | 20 |
| 'Colannon' | teleport (M60) | 26 | 45 |
| 'Avavir' | recall | 20 | 200 |
| Cloak of Thingol | recharge (strength 10) | 62 | 70 |
| 'Colluin' | temp resist all 5 base, 20+1d20 | 12 | 111 |
| Crown of Gondor, 'Elessar' | heal 500 | 40/60 | 250/200 |
| 'Soulkeeper', Vilya | heal 1000 | 75/80 | 444/200+2d50 |
| Boots of Fëanor, 'Taratol' | haste 20+1d20 | 30/35 | 200 / 100+d100 |
| Ring of Tulkas | haste 75+1d75 | 60 | 150+d150 |
| Armour of Celeborn | banishment | 40 | 500 |
| Great Axe of Eönwë | mass banishment | 30 | 1000 |
| Lúthien, Evenstar, Nenya | restore life levels | 40/40/40 | 250/150/200+2d50 |
| 'Holhenneth' | detect all | 20 | 55+d55 |
| Bladeturner | heal 30, berserk, bless, resist all 50+1d50 | 55 | 50 |

ADVICE: activations are free, unlimited consumables. A borg should model each as "a potion that
recharges", using it before spending real consumables (e.g. Colluin before Resist Heat potions,
Open Wounds before CLW).

---

## 18. Stacking, pack and quiver limits

FACT (`lib/gamedata/constants.txt`, `src/obj-gear.c:pack_slots_used()`):

* Pack: **23 slots** (`pack-size`). Equipment does not count.
* Quiver: up to **10 quiver slots** (`quiver-size`) for ammunition (and THROWING weapons). Quiver
  contents take pack slots at **40 missiles per slot** (`quiver-slot-size`), rounded up; each
  throwing weapon counts as 5 missiles (`thrown-quiver-mult`). So 40 arrows cost 1 pack slot,
  41-80 cost 2.
* Maximum stack: 40 per stack (`default:max-stack:40` in `lib/gamedata/object_base.txt`).
* Wands/staves/rods of the same kind stack; their charges or timeouts are pooled.
* Floor: 23 objects per grid with `birth_stacking` (`floor-size`).
* Home: 24 stacks (`store:inven-max`), stacked like the pack (up to 40).

ADVICE: a typical mid-game pack: 1 slot food, 1 light fuel, 3-5 potion types, 4-6 scroll types,
2-4 devices, 1-2 swap items, books, 1-2 quiver slots: that already fills ~20 of 23. Plan the kit
(20) around slots, not just weight.

---

## 19. Element damage to carried items

FACT (`src/project-obj.c:inven_damage()`, called from `src/project-player.c` handlers and from
monster melee in `src/mon-blows.c`):

* When you take acid, electricity, fire or cold damage (including ice, and the acid side-effect of
  strong poison), each **unequipped, non-artifact** item stack that "hates" that element and does
  not "ignore" it is tested. The per-item chance is `min(damage * 5, 300) / 10000`, i.e. at most
  **3% per item**, reached at 60+ damage *after* resistance. Each item in a stack rolls
  separately. Rods use a quarter of the chance. Weapons and armour in the pack are *damaged*
  (-1 to-hit and to-dam, or -1 to-AC) instead of destroyed.
* Immunity protects completely; resistance protects indirectly by lowering the damage that feeds
  the formula (double resistance cuts it to 1/9).
* Standing in lava burns carried items too (`src/player-util.c`).
* Equipped items are never destroyed this way. Sound, shards and force damage do not call
  `inven_damage()` for carried items (they only affect floor objects that hate them).

What hates what (`lib/gamedata/object_base.txt`):

| Element | Carried item types destroyed/damaged |
|---------|--------------------------------------|
| Acid | scrolls, staves, arrows, bolts, all weapons except diggers, all armour except DSM, chests |
| Fire | scrolls, staves, books, arrows, bows, hafted, polearms, soft armour, cloaks, gloves, boots, lights |
| Cold | potions, flasks of oil |
| Electricity | rings, wands, rods |
| none | amulets, food, mushrooms, shots, diggers, dragon armour |

Items with matching IGNORE flags are safe (e.g. Wand of Lightning Balls, Rod of Recall and Rod of
Healing ignore elec; Wands of Wonder and the dragon wands ignore everything).

ADVICE:

* Get rFire early: it is the element that destroys scrolls *and* books *and* staves. rCold
  protects potions. rElec protects rings/wands/rods. rAcid protects scrolls/staves and armour.
* Spread critical functions across element classes: e.g. teleport via scrolls (acid/fire) *and*
  a potion-free backup such as an artifact activation (never destroyed while worn); recall via
  scroll *and* Rod of Recall (ignores elec). Staves share the acid/fire weakness of scrolls.
* Before fighting a fire breather without rFire, drop spare books and scrolls in a safe spot or
  leave them at home.

---

## 20. Standard carry kits per phase

Stock sources verified against `lib/gamedata/store.txt` (store columns as in 1). "Dungeon" means
found in the dungeon or the Black Market only. Prices are kind costs (normal-store price).

### 20.1 Early (CL1-15, DL1-10)

| Item | Qty | Source | Notes |
|------|-----|--------|-------|
| Potion of Cure Light Wounds | 5 | Alchemist, always (20) | heal + blindness cure |
| Scroll of Phase Door | 10 | Alchemist, always (15) | main escape |
| Flask of oil | 10-20 | General, always (3) | thrown weapon + lantern fuel |
| Scroll of Word of Recall | 1 (2 when affordable) | Alchemist, always (125); class kits start with 1 | |
| Wooden Torch | 3 (or a lantern) | General, always (2) | lantern from dungeon/BM only |
| Ration of Food | 3-5 | General, always (3) | |
| Potion of Cure Serious Wounds | 2-3 when affordable | Alchemist, sometimes (70) | Priest starts with 2 |
| Scroll of Magic Mapping | optional | Alchemist, sometimes (65) | stairs-finding |
| Shovel | optional | General, always | rubble |

Budget: Warrior starts with ~130-150 gold after the kit (see [Chapter 11](11-town-and-shopping.md));
5 CLW + 5 Phase + 10 flasks = 205 gold. Prioritise flasks and Phase Door, then CLW.

### 20.2 Mid (CL15-30, DL10-30)

| Item | Qty | Source |
|------|-----|--------|
| Potion of Cure Critical Wounds | 5-10 | Alchemist, sometimes (200) |
| Potion of Cure Serious Wounds | 5 | Alchemist, sometimes (70) |
| Scroll of Phase Door | 10-15 | Alchemist, always |
| Scroll of Word of Recall | 2 | Alchemist, always |
| Scroll of Teleportation | 3-5 | **dungeon only** (not stocked by the Alchemist in this version) |
| Staff of Teleportation | 1 | Magic Shop, sometimes (~2700-2900 with charges) |
| Scroll of Deep Descent | 1-2 (diving) | dungeon only |
| Scroll of Teleport Level | 1 | dungeon only |
| Potion of Speed | any found | dungeon or BM |
| Scroll of Magic Mapping / Staff of Detect Evil | 3-5 / 1 | Alchemist / Magic Shop |
| Flasks of oil | 10-15 | General |
| Scroll of Remove Hunger | 2-3 | Alchemist (10) |
| Wand of Stone to Mud | 1 | Magic Shop |
| Potion of Resist Poison / Neutralize Poison | a few | dungeon / Alchemist |
| Potion of Berserk Strength | 2-3 (melee) | Alchemist (80) |

Because Scrolls of Teleportation are not sold in normal stores, the **Staff of Teleportation from
the Magic Shop is the reliable teleport source**; buy one as soon as you can afford ~2800 gold,
and keep found teleport scrolls as backup. With `birth_no_selling` (default) dungeon gold is 5x
larger, which makes this affordable early.

### 20.3 Late (CL30-45, DL30-60)

| Item | Qty | Source |
|------|-----|--------|
| Potion of Cure Critical Wounds | 10-15 | Alchemist |
| Potion of Healing | 3-5 | dungeon / BM (2400 observed) |
| Potion of Restore Mana (casters) | 5+ | dungeon / BM |
| Potion of Speed | 5+ | dungeon / BM |
| Staff of Teleportation (or Rod/activation) | 1-2 | Magic Shop |
| Scroll of Teleportation | 5 | dungeon |
| Scroll of Teleport Level / Deep Descent | 2 / 1-2 | dungeon |
| Word of Recall or Rod of Recall | 2 / 1 | Alchemist / dungeon |
| Rod of Detection or Rod of Treasure Location + Staff of Detect Evil | 1 | dungeon / Magic Shop |
| Rod of Magic Mapping | 1 | dungeon |
| Scroll of *Destruction* or Staff of *Destruction* | 1-2 | dungeon |
| Scroll of Banishment | 1-2 | dungeon |
| Potion of Resist Poison / Heat / Cold | a few | dungeon / Alchemist |
| Mushrooms of Emergency / Clear Mind | a few | dungeon |

### 20.4 Endgame (CL45-50, DL60-100)

| Item | Qty | Source |
|------|-----|--------|
| Potion of *Healing* | 5-10 | dungeon / BM |
| Potion of Life | 2-5 | dungeon / BM |
| Potion of Healing | 5-10 | dungeon / BM |
| Potion of Restore Mana | 10+ (casters) | dungeon / BM |
| Potion of Speed | 10+ | dungeon / BM |
| Staff of Teleportation + Scrolls of Teleportation | 1 + 5 | Magic Shop + dungeon |
| Scroll of Teleport Level, Deep Descent | 2-3 | dungeon |
| Rod of Recall, Rod of Detection, Rod of Magic Mapping | 1 each | dungeon |
| Scroll of Mass Banishment / Staff of Banishment | 2-3 / 1 | dungeon |
| Staff/Scroll of *Destruction* | 1-2 | dungeon |
| Rod of Healing, Rod of Speed | if found | dungeon |

Compare with the borg's own preparation rules (`src/borg/borg-prepared.c:borg_prepared_aux()`):
2 cures by DL5, 1 recall by DL5-9, 2 teleports (scroll or staff) by DL10, 3 CCW by DL20, 6 teleports
and 10 CCW+CSW by DL26-33, 1 Healing by DL46, 2 Healing/*Healing* by DL56, and before Morgoth
5 Healing, 15 *Healing*/Life, 10 Speed and (casters) 10 Restore Mana. Those numbers are a
reasonable lower bound for a human player too.

---

## Things to try (untested)

1. **Store-recharge loop**: with lots of gold, give an empty Staff of Teleportation to the Magic
   Shop and buy it back recharged instead of reading Recharging (no backfire risk). Test: gold
   cost per charge vs Scroll of Recharging including backfire losses.
2. **Deep Descent as a controlled dive**: since DD goes to max depth +5, read it right after
   recalling to town-depth? It works from town too; test reading DD in town to skip stairs entirely
   (expected: town -> max+5). Measure turns saved per 5 levels vs stairs.
3. **Stack-split recharging**: split a stack of 3 wands into 1+2 before recharging the single one,
   compare charge-per-scroll and losses over 50 trials in a debug build.
4. **Flask barrage vs early uniques**: test killing Grip/Fang, Bullroarer and Wormtongue with
   flasks only (x3 explode damage) for a non-archer class; record flasks used.
5. **Mushroom of Emergency rotation**: treat Emergency mushrooms as the primary mid-game heal
   (60 gold for 200 HP if found) and measure deaths from hallucination side effects.
6. **Rod-stack detection**: carry 2 Rods of Treasure Location/Detection to halve effective
   recharge time; measure gold found per turn.
7. **Activation-first policy for a borg**: always prefer an available activation over a
   consumable with the same effect, and measure consumable savings over a full game.
8. **Device boost exploitation**: high-skill casters with low-level attack wands (Stinking Cloud,
   Magic Missile) get large % boosts; quantify damage per charge at CL40 vs spells.
9. **Fire-proof pack**: run a character that keeps all critical escapes in element-proof forms
   (rods, artifact activations, potions + rCold) and compare item losses to a scroll-based pack.
10. **Re-dose haste timing**: a second Speed potion only adds 5 turns; test whether quaffing the
    next potion exactly when haste ends (watch the status line) saves potions in long fights.

# 14. Class Playbooks

One deep strategy guide per class: what the class is good and bad at, which stats and races
suit it, where its damage comes from at each stage, which spells unlock when, what equipment it
wants, how it escapes, how it usually dies, and a phase-by-phase plan from CL1 to CL50.
Section 0 explains every class mechanic flag from the source first. All spell numbers are from
`lib/gamedata/class.txt` at the reference commit; the classes are exactly the nine defined there.

**Contents**

0. [Class mechanics reference](#0-class-mechanics-reference)
1. [Warrior](#1-warrior)
2. [Mage](#2-mage)
3. [Druid](#3-druid)
4. [Priest](#4-priest)
5. [Necromancer](#5-necromancer)
6. [Paladin](#6-paladin)
7. [Rogue](#7-rogue)
8. [Ranger](#8-ranger)
9. [Blackguard](#9-blackguard)
10. [Things to try (untested)](#things-to-try-untested)

Conventions: "CL" is character level, "DL" dungeon level (50 ft each). Spell rows are
`level / mana / base fail %` exactly as in `class.txt`. Damage figures are averages computed from
the `dice`/`expr` lines; "L" means character level. Phase depth suggestions are ADVICE; the
depth-readiness checklists live in [Chapter 12](12-game-plan-by-phase.md). Full spell
descriptions are in [Chapter 3](03-magic-and-spells.md) and [A7](appendices/A7-class-spells.md);
race/class numbers in [Chapter 2](02-character-creation.md) and
[A8](appendices/A8-races-and-classes.md). The official manual's class paragraphs are in
[user-manual/birth.rst](user-manual/birth.rst)
([online](https://angband.readthedocs.io/en/latest/birth.html)).

---

## 0. Class mechanics reference

### 0.1 Player flags (`src/list-player-flags.h`, `lib/gamedata/player_property.txt`)

Every flag below was traced to the code that implements it (FACT):

| Flag | Who | Exact mechanics | Code |
|---|---|---|---|
| `FAST_SHOT` | Ranger | Shots per turn (stored in tenths, base 10) gain `CL/3` tenths with **any** launcher (sling, bow or crossbow), unless the launcher is too heavy to hold. CL3: 1.1, CL15: 1.5, CL30: 2.0, CL45: 2.5, CL48-50: 2.6. Each shot costs `100×10/shots` energy | `player-calcs.c:calc_bonuses()`, `player-attack.c:ranged_helper()` |
| `BRAVERY_30` | Warrior | Immune to fear (`PROT_FEAR`) from CL30 | `player.c:player_flags()` |
| `BLESS_WEAPON` | Priest, Paladin | With a hafted weapon (`TV_HAFTED`) or any weapon with the `BLESSED` flag: +2 to-dam and +2 effective to-hit. There is **no penalty** for edged weapons in this version | `player-calcs.c`, `player-attack.c:chance_of_melee_hit_base()` |
| `ZERO_FAIL` | Mage, Druid, Priest, Necromancer | Spell failure may fall below 5% (down to the stat's minimum-fail value, 0% at 18/200+). Without it the floor is 5% | `player-spell.c:spell_chance()` |
| `BEAM` | Mage | `BOLT_OR_BEAM` spells beam with probability CL% instead of CL/2 % | `player-spell.c:beam_chance()` |
| `CHOOSE_SPELLS` | all casters except Priest, Paladin | You pick which spell to learn. Without it, studying a book teaches a **random** learnable spell from that book | `cmd-obj.c:do_cmd_study()` |
| `NO_MANA` | Warrior (and anyone whose max SP is 0) | Marks the character as having no mana; monsters that learn this stop using mana-drain spells | `player-calcs.c`, `mon-spell.c` |
| `CHARM` | Druid | The player's sleep, slow, confuse, hold, stun and polymorph effects get +50% power against `ANIMAL` monsters | `project-mon.c` |
| `UNLIGHT` | Necromancer | Resist darkness (in the dungeon); sees grids within `2 + CL/6 - light` (exclusive) when its light radius is 1 or less; +25% spell failure when standing on a lit grid; a +1 light modifier on gear counts as 0; remembers unlit grids; can unlock doors without a light without penalty | `player-calcs.c`, `cave-view.c`, `player-spell.c`, `cave-map.c`, `player-path.c` |
| `EVIL` | Necromancer | Resist nether; **vulnerable** to holy orb (resist level -1) | `player-calcs.c:calc_bonuses()` |
| `STEAL` | Rogue | The `s` command (original keys) on an adjacent monster attempts to steal a carried item instead of attacking | `cmd-cave.c:do_cmd_steal()`, `mon-util.c:steal_monster_item()` |
| `SHIELD_BASH` | Warrior, Paladin, Blackguard | Before normal blows, a chance to bash with a worn shield (details below) | `player-attack.c:attempt_shield_bash()` |
| `COMBAT_REGEN` | Blackguard | Mana comes from fighting and decays at rest; spending or losing mana heals (details in [section 9](#9-blackguard)) | `player-util.c`, `player-attack.c`, `player-spell.c` |
| `KNOW_MUSHROOM` | Hobbit (race) | Mushrooms identified on pickup | `obj-gear.c` |
| `KNOW_ZAPPER` | Gnome (race) | Wands and staffs identified on pickup | `obj-gear.c` |
| `SEE_ORE` | Dwarf (race) | Free ore/treasure detection, range 3, every turn while unimpaired | `game-world.c` |
| `ROCK` | Pukel-man shape | Cuts neither worsen nor heal | `game-world.c` |

The property text for `UNLIGHT` says "You gain stealth in ... the dark". **No stealth bonus for
darkness exists in the code at this commit** (stealth is only modified by race, class, level,
equipment, shapes and the timed Stealth effect in `calc_bonuses()`). Treat the stealth claim as
unimplemented.

**Shield bash in detail** (FACT, `attempt_shield_bash()`): only against visible monsters whose
level is at least half yours, and only with a shield worn.

- Chance: `melee skill/8 + DEX to-hit adj/2`, doubled if your weapon's `dice×dice-sides×blows`
  is less than 3× the shield's dice product, quadrupled if unarmed; succeeds if that exceeds
  `randint0(200 + monster level)`.
- Damage: shield dice × (`quality/40 + CL/14`) + STR to-dam bonus, capped at 125, where quality =
  `melee skill/4 + body weight/8 + carried weight/80 + shield weight/2` (the only use of your
  character's weight in the game).
- May stun and (rarely) confuse; then you may stumble and lose 26-75% of a turn: the chance of
  stumbling is `(25 - DEX to-hit adj)/60`, about 37% at DEX 18.
- ADVICE: bashes are a bonus, not a plan. They are most valuable for a Blackguard or Paladin
  carrying a heavy shield (big shield dice) and for an unarmed or weak-weapon character. Heavy
  shields cost spell weight for Paladins (40 lb allowance).

### 0.2 Spellcasting rules for every caster

FACT ([Chapter 2, section 6.2](02-character-creation.md#62-intelligence-and-wisdom-spell-stats)
has the stat tables; [Chapter 3](03-magic-and-spells.md) covers each spell):

- **Learning.** New spells per level depend on the spell stat (`adj_mag_study`). You can learn a
  spell only if its level is at most your CL and you have the book. Priests and Paladins get a
  random learnable spell from the book you choose to study; to control what they learn, study
  from the book whose only learnable spell is the one you want.
- **First cast bonus.** The first successful cast of each spell gives `exp × spell level`
  experience (the last field of the `spell:` line). Cast every new spell once, safely.
- **Mana.** `1 + adj_mag_mana × (CL - first spell level + 1)/100`; minus 1 per full pound of
  armour above the class allowance (Mage/Necro 30 lb, Priest/Druid 35, Rogue 35, Paladin/Ranger 40,
  Blackguard 60). **Gloves never matter** (no glove encumbrance code exists).
- **Failure.** `base - 3×(CL - spell level) - stat adj`; +5% per missing mana point; +25%
  Necromancer on a lit grid; +20% afraid; clamped to [min fail, 50]; then stun +15/+25, amnesia
  `50 + fail/2`, cap 95.
- **Casting with too little mana is allowed and dangerous**: your SP drops to 0, you faint
  (paralysis) for `1d(5×shortfall+1)` turns, and there is a 50% chance of CON damage
  (`player-spell.c:spell_cast()` calling `player_over_exert()`; the CON loss is permanent with
  some chance). Never do it with monsters near.
- **Shapechanged characters cannot cast, read, quaff, zap, aim, use, activate, eat, refill,
  wield/remove or study**: each such command asks whether to change back first
  (`player-util.c:player_get_resume_normal_shape()`). Changing back from the prompt is free of
  danger but you then need a turn for the action.

### 0.3 Shapechanges (`lib/gamedata/shape.txt`)

`combat` is to-hit / to-dam / AC. `MOVES[n]` reduces movement energy to `1/(n+1)` of normal
(`player-util.c:energy_per_move()`): MOVES[1] walks twice as fast, MOVES[3] four times as fast.
Melee blows use shape-specific verbs.

| Shape | Class / CL | Combat | Key values | Flags | Cost on change |
|---|---|---|---|---|---|
| fox | Druid 1 | -3/-3/+3 | STR -3, stealth +5, blows +1, moves +1 | Feather Fall, Free Action | none |
| bat | Necromancer 3 | 0/-10/+10 | speed +3, stealth +3, infra +5 | Feather Fall, See Invisible, blindness immunity | 5 HP |
| warg | Necromancer 28 | +5/+5/0 | blows +2 | - | `1d(CL)` HP; cures fear, berserk 5+1d20 |
| Pukel-man | Druid 30 | 0/+5/+20 | STR +4, CON +4, stealth -2, **speed -5**, poison immunity, rShards, damage reduction 10, save +20 | sustain STR/CON, stun immunity, Regen, Hold Life, `ROCK` | cures poison and stun |
| eagle | Druid 35 | +5/0/+10 | moves +3 | sustain WIS, immunity to blindness/confusion/fear, See Invisible, Free Action, trap immunity | none |
| bear | Druid 40 | +15/+15/+5 | STR +3, INT -2, CON +2, infra +1, stealth -3, blows +1, melee +10, disarm -5/-10 | fear immunity | none |
| vampire | Necromancer 37 | +3/+3/-5 | light -1, stealth +5, infra +5, speed +5 | See Invisible, Hold Life | HP (see Necromancer); vampiric bite 10+1d20 turns |
| werewolf | Blackguard 32 | +5/+5/0 | blows +1, moves +1 | Regen | scares all in LOS; berserk 5+1d20 |

### 0.4 Books: names, owners and sources

Town books are **always** in stock at the Bookseller (`lib/gamedata/store.txt`: `always:` lines;
`src/store.c` expands them to every non-dungeon book of that type). Dungeon books must be found
(or bought from the Black Market when it happens to stock one). The depth is the object's
allocation range from `book-properties` (generation outside the range is possible but rare).

| Realm | Book | Town/dungeon | Alloc depth | Used by |
|---|---|---|---|---|
| arcane (magic book) | [First Spells] | town | 1+ | Mage, Rogue |
| | [Attacks and Knowledge] | town | 10+ | Mage |
| | [Magical Defences] | town | 30+ | Mage |
| | [Arcane Control] | dungeon | 40+ | Mage, Rogue |
| | [Wizard's Tome of Power] | dungeon | 60+ | Mage |
| nature | [Lesser Charms] | town | 1+ | Druid, Ranger |
| | [Gifts of Nature] | town | 10+ | Druid |
| | [Creature Dominion] | dungeon | 30+ | Druid |
| | [Nature Craft] | dungeon | 40+ | Druid, Ranger |
| | [Wild Forces] | dungeon | 60+ | Druid |
| divine (prayer book) | [Novice's Handbook] | town | 1+ | Priest, Paladin |
| | [Cleansing Power] | town | 5+ | Priest |
| | [Healing and Sanctuary] | dungeon | 30+ | Priest, Paladin |
| | [Battle Blessings] | dungeon | 40+ | Priest, Paladin |
| | [Wrath of the Valar] | dungeon | 60+ | Priest |
| shadow | [Into the Shadows] | town | 1+ | Necromancer, Blackguard |
| | [Dark Rituals] | town | 10+ | Necromancer |
| | [Fear and Torment] | town | 30+ | Necromancer, Blackguard |
| | [Deadly Powers] | dungeon | 40+ | Necromancer, Blackguard |
| | [Corruption of Spirit] | dungeon | 60+ | Necromancer |

Hybrid classes see a *different subset* of the spells in a shared book (e.g. a Rogue's [First
Spells] contains Detect Monsters, Phase Door, Object Detection, Detect Stairs, Recharging,
Reveal Monsters, not the Mage's list). ADVICE: carry a spare copy of each book you rely on;
fire and acid destroy books in the pack ([Chapter 5](05-elements-and-status.md)).

A key consequence (FACT from the table): several classes' first spell escapes sit in dungeon
books. Priests get Portal (CL10) only from [Healing and Sanctuary] (DL30+); Rogues get Teleport
Self (CL17) only from [Arcane Control] (DL40+); Druids have **no teleport spell at all**. Plan
consumable escapes accordingly ([Chapter 13](13-escapes-and-emergencies.md)).

### 0.5 Class comparison at a glance (ADVICE)

| Class | Melee | Missiles | Attack magic | Escapes (own) | Detection (own) | Healing (own) | Difficulty |
|---|---|---|---|---|---|---|---|
| Warrior | best | good | none | none | none | none | low |
| Mage | poor | poor | best | best | very good | none | medium |
| Druid | poor (good in bear form) | fair | good | movement shapes only | good (map, life) | cures | high |
| Priest | fair | fair | good vs evil | Portal (CL10, dungeon book) | evil only early | best | low-medium |
| Necromancer | fair | poor | very good | Shadow Shift (CL15) | minds/invisible | Vampire Strike | high |
| Paladin | very good | fair | none | WoR prayer (CL25) | evil | good | low |
| Rogue | good late | very good (throwing) | none | Phase (CL7), Teleport (CL17, dungeon book) | objects, monsters, stairs | none | medium |
| Ranger | good | best | none | none (Haste CL25) | life, map | cures | low-medium |
| Blackguard | very good | weak | utility | none | fear-susceptible only | combat regen | high |

---

## 1. Warrior

### 1.1 Identity and numbers

FACT: STR +3, INT -2, WIS -2, DEX +2, CON +2; hit die 9 (the highest); max 6 blows, 3 lb weapon
floor, STR multiplier 5; melee 70 (+45/10 CL), shooting and throwing 55 (+45); device 18 (+7),
save 18 (+10), stealth 0. Flags `BRAVERY_30`, `NO_MANA`, `SHIELD_BASH`. Kit: Dagger, Soft
Leather Armour, Potion of Berserk Strength, Word of Recall, food and torches.

**Strengths:** most HP, most blows (6), best to-hit growth, excellent bows, fear immunity at
CL30, no spell or mana management. **Weaknesses:** no magic at all: detection, escapes,
healing, identification and light all come from devices and consumables, used with the worst
device skill in the game; worst saving throw growth among fighters; bad stealth.

### 1.2 Stats and races

STR and DEX for blows (see the blows tables in
[Chapter 2, section 6.5](02-character-creation.md#65-blows-what-stats-buy)), then CON. INT
matters a little (device skill via `adj_int_dev`), WIS a little (saving throw).

| Race | Why |
|---|---|
| Half-Troll | HD 21, STR +7 total, regeneration, sustain STR; worst devices (Teleportation staffs fail more) |
| Dunadan | HD 19, all stats up (+5 CON total), sustain CON, decent skills; the balanced choice |
| High-Elf | See Invisible, device/save +20, shooting +25; 145% exp |
| Dwarf | HD 20, blindness immunity (can always read scrolls), +15 melee |
| Half-Orc | cheap STR/CON and rDark |

### 1.3 Damage by phase

- **CL1-10:** melee with the best light weapon you can get multiple blows with; throw Flasks of
  Oil (2d6 fire, see [Chapter 4](04-combat.md)) at anything dangerous; a Sling early, a Long Bow
  as soon as affordable (55+ shooting skill).
- **CL10-30:** melee with an ego or high-dice weapon that keeps 2-4 blows; a launcher with good
  ammunition for things you do not want to touch (breeders across the room, archers, floating
  eyes).
- **CL30-50:** 4-6 blows with a slaying/branded weapon or a big artifact; launcher as backup.
  Warriors' damage per turn with a top weapon is the highest of any class in melee.

### 1.4 Equipment priorities (ADVICE)

1. A weapon you get at least 2 blows with (check the `I`nspect screen; [Chapter 4](04-combat.md)).
2. **Free Action** before DL20-ish (paralysis is the warrior killer: no spell can save you).
3. **See Invisible** (or High-Elf) before DL30.
4. **Fear protection before CL30**: afraid characters cannot melee. Carry Potions of Boldness /
   Heroism / Berserk Strength (Berserk also gives +to-hit and heals a little), or a fear-immunity
   item, until `BRAVERY_30`.
5. Base resists (fire, cold, acid, lightning), then poison, then confusion/blindness/sound.
6. Launcher (Long Bow → bow of extra might/shots), lots of arrows.
7. Detection devices: Rod of Treasure Location, Rod of Detection or Staff of Detect Evil, Staff
   of Mapping; Rod of Illumination; Rod of Recall to save scrolls.
8. Speed items as soon as found.

### 1.5 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-10 | Scrolls of Phase Door (always carry 5-10), Flasks of Oil, running to stairs, Potions of CLW |
| CL10-20 | + Scrolls of Teleportation or Staff of Teleportation, Scrolls of Deep Descent, Potions of CSW |
| CL20-30 | + Scrolls of Teleport Level, Wand of Teleport Other (device skill permitting), Potions of CCW |
| CL30-50 | + Staff/Rod of Teleportation as main escape, Potions of Healing / *Healing*, Scrolls of *Destruction*, Banishment |

Device skill: a Half-Troll Warrior's device skill at CL30 is roughly 10 + 21 (class) + INT adj,
so check the failure rate shown on each device's description before relying on it.

### 1.6 Typical deaths

Paralysis without Free Action; being afraid and unable to fight while a monster pins you;
invisible ghosts draining you; breath attacks from hounds/dragons before resistances; poison
and confusion breath; running out of escapes because a device failed. Late: big breathers
(Ancient Dragons, Great Wyrms, Dracolisks) and summoners when the warrior lacks resistances.

### 1.7 Phase plan

- **CL1-10 (DL1-5):** buy Flasks of Oil (10-20), Potions of CLW, Phase Door; a Sling or Short Bow
  if gold allows. Melee everything weak; throw oil at Fang, Grip, Bullroarer. Do not fight
  breeders in the open. Recall back when out of oil or CLW.
- **CL10-20 (DL5-15):** get a Long Bow; first ego weapon or a heavier weapon that keeps 2 blows;
  Free Action is the priority purchase/find. Carry Teleportation scrolls. Learn monster recall;
  avoid groups of archers and spellcasters in open rooms.
- **CL20-30 (DL15-30):** See Invisible, base resists, rPois target; Potions of CCW; a Rod of
  Treasure Location and Staff of Detect Evil/Mapping to compensate for no detection spells. Watch
  fear until CL30.
- **CL30-40 (DL30-45):** fear immunity arrives; build a swap kit (rConf/rBlind/rSound items);
  Potions of Healing; Staff of Teleportation as main escape; first speed items. Dive with stair
  scumming.
- **CL40-50 (DL45-99):** stack speed (+20 is a common target before DL60), rNether/rChaos/rDisen,
  *Healing* potions; kill uniques with melee; for Morgoth see [Chapter 12](12-game-plan-by-phase.md).

---

## 2. Mage

### 2.1 Identity and numbers

FACT: STR -3, INT +3, CON -2; **hit die 0** (race only); max 4 blows; melee 35 (+15), device 36
(+13, the best), save 30 (+9); `ZERO_FAIL`, `BEAM`, `CHOOSE_SPELLS`; INT; first spell CL1; 30 lb
armour allowance; 5 books, 30 spells. Kit: Rapier, [First Spells], Word of Recall.

**Strengths:** strongest attack spells and the best utility (detection, teleports, banishment,
Recharging, Identify Rune), best device skill, high beam chance. **Weaknesses:** the lowest HP in
the game, poor melee, no healing spells, needs good INT and light armour.

### 2.2 Stats and races

INT first (mana, spells known, fail), then CON, then DEX. Races: Gnome (Free Action, device
+22), High-Elf (See Invisible, INT +6 total), Dunadan (HP, all-round), Hobbit (Hold Life, very
low HP), Elf.

### 2.3 Spells

| Book | Spell | Lvl | Mana | Fail | Notes (L = CL) |
|---|---|---|---|---|---|
| [First Spells] (town) | Magic Missile | 1 | 1 | 22 | `((L-1)/5+3)d4` unresistable bolt or beam: 3d4 at CL1, 5d4 at CL11, 7d4 at CL21 |
| | Light Room | 1 | 2 | 26 | lights radius 2 + room; hurts light-sensitive |
| | Find Traps, Doors & Stairs | 1 | 1 | 20 | local detection (22×40 area) |
| | Phase Door | 2 | 2 | 22 | teleport 10 |
| | Electric Arc | 2 | 2 | 34 | short elec beam `((L-1)/5+3)d6` |
| | Detect Monsters | 3 | 2 | 24 | visible monsters only |
| | Fire Ball | 6 | 5 | 33 | radius-2 fire ball, damage 2L (12 at CL6, 40 at CL20) |
| [Attacks and Knowledge] (town) | Recharging | 5 | 5 | 28 | power L/8+6 |
| | Identify Rune | 8 | 7 | 25 | |
| | Treasure Detection | 10 | 3 | 60 | gold + sense objects |
| | Frost Bolt | 13 | 5 | 40 | `((L-5)/3+6)d8`: 8d8 at CL13, 14d8 at CL30 |
| | Reveal Monsters | 15 | 6 | 40 | detects invisible too |
| | Acid Spray | 20 | 5 | 30 | 60° arc, range 10, `(L/2)d8` |
| [Magical Defences] (town) | Disable Traps, Destroy Doors | 5 | 5 | 30 | adjacent |
| | Teleport Self | 7 | 6 | 35 | long-range teleport; distance scales with level (`M$M`, M = 9L/5) |
| | Teleport Other | 15 | 10 | 30 | bolt, distance 3L |
| | Resistance | 20 | 20 | 65 | temporary rFire/rCold/rAcid/rElec 20+1d20 |
| | Tap Magical Energy | 22 | 2 | 30 | drain a wand/staff for mana |
| | Mana Channel | 25 | 10 | 50 | faster casting 5+1d5 turns |
| [Arcane Control] (DL40+) | Door Creation | 13 | 9 | 40 | doors on all adjacent floor |
| | Mana Bolt | 25 | 8 | 60 | `(L-10)d8` unresistable: 15d8 at CL25, 40d8 at CL50 |
| | Teleport Level | 28 | 17 | 65 | |
| | Detection | 30 | 10 | 70 | everything, incl. invisible monsters and objects |
| | Dimension Door | 35 | 30 | 80 | teleport to a chosen grid |
| | Thrust Away | 40 | 12 | 90 | short force beam `Ld8` |
| [Wizard's Tome of Power] (DL60+) | Shock Wave | 20 | 5 | 40 | radius-2 sound ball 2L, stuns |
| | Explosion | 30 | 10 | 50 | radius-2 shards 2L + force L/5 |
| | Banishment | 35 | 45 | 95 | removes a monster symbol from the level, 1d4 HP per monster |
| | Mass Banishment | 40 | 75 | 90 | removes all non-uniques within 20, 1d3 HP each |
| | Mana Storm | 45 | 16 | 85 | radius-3 mana ball, 2L+300 (400 at CL50) |

Note how early the important escapes arrive: Phase Door at CL2 and Teleport Self at CL7 (both
from town books). [Magical Defences] costs more and its alloc depth is 30+, but it is a town book:
buy it as soon as you can afford it.

### 2.4 Damage by phase

- **CL1-5:** Magic Missile (3d4, unresistable, never misses). Melee only finishes weak things.
- **CL6-20:** **Fire Ball** from CL6 (2L, radius 2) handles groups; Magic Missile for
  fire-resistant targets; Frost Bolt from CL13 for single targets; Wand of Stinking Cloud / Magic
  Missile for backup. Beam chance (CL%) makes bolts hit lines of monsters.
- **CL20-30:** Acid Spray (cone, 10d8 at CL20) for packs at range; Frost Bolt for singles;
  Fire Ball for fire-vulnerable groups.
- **CL25-40 (needs [Arcane Control]):** Mana Bolt is the single-target workhorse (unresistable,
  20d8 at CL30, 30d8 at CL40).
- **CL40-50 (needs [Wizard's Tome of Power]):** Mana Storm (2L+300 radius 3) is the endgame nuke;
  Explosion and Shock Wave for crowds; Banishment and Mass Banishment remove whole threat classes.

### 2.5 Equipment priorities

INT items; armour totalling at most 30 lb (each extra pound = -1 SP); Free Action (unless Gnome);
See Invisible (Reveal Monsters at CL15 detects but does not let you see them); CON; base resists;
speed. Gloves are fine (no glove penalty). Carry Potions of CLW/CSW (no healing spells), Scrolls
of Teleportation as backup, and Potions of Restore Mana for emergencies.

### 2.6 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-7 | Phase Door spell (CL2), Scrolls of Phase Door, Flasks of Oil |
| CL7-15 | + Teleport Self (CL7, needs [Magical Defences]) |
| CL15-28 | + Teleport Other (CL15), Resistance (CL20), Scrolls of Deep Descent/Teleport Level |
| CL28-35 | + Teleport Level (CL28), Door Creation (CL13) to wall in fleeing routes |
| CL35-50 | + Dimension Door (CL35), Banishment (CL35), Mass Banishment (CL40) |

### 2.7 Typical deaths

One breath or one round of melee taking a low-HP mage from full to dead (check that your HP
exceeds the worst breath on the level: [Chapter 8](08-monsters.md)); casting while out of mana
(fainting); invisible monsters; being surrounded in a corridor junction; packs of hounds early;
failed teleports from low INT or stun.

### 2.8 Phase plan

- **CL1-10 (DL1-5):** buy [Magical Defences] early if you can (Teleport Self CL7) and Potions of
  CLW. Kill with Magic Missile from range; rest to full mana before each fight. At CL6 learn
  Fire Ball. Use Find Traps/Doors/Stairs on arrival and Detect Monsters often.
- **CL10-20 (DL5-15):** buy [Attacks and Knowledge] (Recharging, Identify Rune, Frost Bolt).
  Recharge wands of Magic Missile/Stinking Cloud. Get Free Action. Avoid melee completely.
- **CL20-30 (DL15-30):** Acid Spray, Resistance, Reveal Monsters. Look for [Arcane Control]
  (DL40+ normally, so often bought from the Black Market or found late). CON and INT potions.
- **CL30-40 (DL30-45):** Mana Bolt and Detection if you have [Arcane Control]; Teleport Level
  for fast escapes; Dimension Door at CL35. See Invisible and rConf/rBlind become critical.
- **CL40-50 (DL45-99):** Mass Banishment and Mana Storm if you found [Wizard's Tome of Power].
  Kill uniques at range; banish hounds (`Z`), dragons (`d`, `D`), or demons as the level demands.

---

## 3. Druid

### 3.1 Identity and numbers

FACT: STR -2, WIS +3, DEX -2; hit die 2; max 4 blows; melee 45 (+20), shooting 40 (+30), device
24 (+10), save 30 (+10), stealth 3; `ZERO_FAIL`, `CHOOSE_SPELLS`, `CHARM`; WIS; first spell CL1;
35 lb allowance; 5 books, 27 spells. Kit: Whip, [Lesser Charms], Word of Recall.

**Strengths:** good area damage early (Stinking Cloud at CL3), control (confuse, slow, sleep,
hold) boosted against animals by `CHARM`, mapping, fast-moving shapes, strong late nukes.
**Weaknesses:** no teleport spell at all; shapes block item use; low HP; weak melee in normal
form.

### 3.2 Stats and races

WIS, then CON, then DEX. Races: Dwarf (WIS +5 total, blindness immunity, HD 13), Hobbit (WIS +4,
Hold Life), Dunadan, Gnome (Free Action, WIS +3).

### 3.3 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Lesser Charms] (town) | Detect Life | 1 | 1 | 23 | living monsters |
| | Fox Form | 1 | 2 | 20 | double move speed, Free Action, stealth +5 |
| | Remove Hunger | 2 | 2 | 25 | food solved forever |
| | Stinking Cloud | 3 | 2 | 27 | radius-2 poison ball, L/2+10 |
| | Confuse Monster | 5 | 3 | 30 | power 5+d(L-5), ×1.5 vs animals |
| | Slow Monster | 6 | 4 | 30 | same power |
| [Gifts of Nature] (town) | Cure Poison | 4 | 4 | 30 | |
| | Resist Poison | 7 | 6 | 32 | 20+1d20 turns |
| | Turn Stone to Mud | 8 | 3 | 25 | |
| | Sense Surroundings | 9 | 4 | 35 | map, area grows at CL30 |
| | Lightning Strike | 12 | 6 | 35 | `(L/4)d4` elec + (L+5) sound at the target |
| | Earth Rising | 14 | 5 | 40 | shard line `(L/3+2)d6` |
| [Creature Dominion] (DL30+) | Trance | 20 | 10 | 45 | holds adjacent monsters 3+d(L/7) (longer for animals) |
| | Mass Sleep | 25 | 15 | 50 | sleep everything in LOS, power 10L+500 |
| | Become Pukel-man | 30 | 20 | 75 | tank form (speed -5) |
| | Eagle's Flight | 35 | 20 | 80 | 4× movement, many immunities |
| | Bear Form | 40 | 20 | 85 | melee form |
| [Nature Craft] (DL40+) | Tremor | 20 | 20 | 60 | targeted radius-4 earthquake |
| | Haste Self | 25 | 12 | 65 | +10 speed, L+d20 turns |
| | Revitalize | 35 | 70 | 90 | restore stats and exp |
| | Rapid Regeneration | 37 | 20 | 90 | |
| | Herbal Curing | 40 | 20 | 90 | cures cuts/poison/stun/Black Breath, feeds |
| [Wild Forces] (DL60+) | Meteor Swarm | 30 | 14 | 85 | 3-4 radius-1 meteors, each ~L/2+30 |
| | Rift | 35 | 20 | 60 | gravity beam 40+Ld7, may teleport targets |
| | Ice Storm | 37 | 25 | 75 | ice on every monster in LOS, 3d(3L) |
| | Volcanic Eruption | 40 | 30 | 75 | radius-5 fire 3L/2+d(3L) around you, then earthquake |
| | River of Lightning | 43 | 35 | 75 | plasma cone `(L+10)d8` |

### 3.4 Damage by phase

- **CL1-3:** Whip melee and thrown oil.
- **CL3-12:** Stinking Cloud (11-16 poison, radius 2) is your gun; watch for poison-resistant
  monsters (undead, golems, many `j` and `E`) and switch to oil or melee.
- **CL12-20:** Lightning Strike (sound component stuns) and Earth Rising (shards line, rarely
  resisted early). Confuse/Slow on dangerous single animals.
- **CL20-35:** Mass Sleep and Trance for control rather than damage; Earth Rising scaling;
  wands to fill gaps.
- **CL30-50 (needs [Wild Forces]):** Meteor Swarm, Rift, Ice Storm (LOS), River of Lightning are
  among the strongest attacks in the game. Bear Form melee is a viable alternative at CL40.

### 3.5 Equipment priorities

WIS, armour within 35 lb, Free Action, See Invisible, CON, resists. Plenty of Scrolls of Phase
Door, Teleportation and Deep Descent (no teleport spells). Note that you must leave a shape to
read them.

### 3.6 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-20 | Fox Form (2× movement: outrun normal-speed monsters if you have a path), Phase Door/Teleport scrolls, Stinking Cloud to kill pursuers |
| CL20-35 | + Mass Sleep (CL25), Trance (CL20), Slow/Confuse; Staff of Teleportation |
| CL35-50 | + Eagle's Flight (4× movement, immunities), Haste Self (CL25, [Nature Craft]), Rift (removes the target with a chance) |

ADVICE: casting a shape to flee and then needing a scroll costs an extra turn to change back.
Decide before transforming whether running is enough.

### 3.7 Typical deaths

No instant escape when surrounded; poison-immune monsters that ignore Stinking Cloud; low HP vs
breaths; being stuck in a shape when a scroll is needed; sleep/confuse-immune uniques.

### 3.8 Phase plan

- **CL1-10:** Stinking Cloud from CL3; Remove Hunger from CL2 means you never buy food. Fox Form
  for scouting and running. Buy [Gifts of Nature] at CL7-9 for Resist Poison and Stone to Mud.
- **CL10-20:** Sense Surroundings (CL9) to plan routes; Lightning Strike CL12; Earth Rising CL14.
  Free Action. Carry Teleportation scrolls.
- **CL20-30:** hunt for [Creature Dominion] (DL30+) and [Nature Craft] (DL40+): Mass Sleep and
  Haste Self change the class. Use Tremor carefully (it can hurt you).
- **CL30-40:** Pukel-man for tough melee fights (no speed though), Eagle's Flight for escape.
- **CL40-50:** with [Wild Forces], nuke from range: Ice Storm for LOS crowds, River of Lightning
  for lines, Rift for single big targets.

---

## 4. Priest

### 4.1 Identity and numbers

FACT: STR -1, INT -3, WIS +3, DEX -1, CON +1; hit die 2; max 4 blows; melee 45 (+20), device 30
(+10), save 32 (+12), stealth 2; `BLESS_WEAPON`, `ZERO_FAIL`; random spell learning (no
`CHOOSE_SPELLS`); WIS; first spell CL1; 35 lb; 5 books, 28 spells. Kit: Mace, 2 Potions of CSW,
[Novice's Handbook], Word of Recall.

**Strengths:** the best healing (Healing = 2000 HP at CL30), restoration of stats and
experience, strong anti-evil damage (Orb of Draining, Dispel/Holy Word, Spear of Oromë, Light of
Varda), Banish Evil, Word of Destruction, Glyph of Warding. **Weaknesses:** no escape spell
until Portal from the dungeon book [Healing and Sanctuary]; little detection beyond Detect Evil;
damage is weaker vs non-evil (animals, many hounds, vortices).

### 4.2 Stats and races

WIS, CON, then STR for melee and carrying. Races: Dwarf (blindness immunity, WIS +5), Dunadan,
High-Elf (See Invisible, device/save), Hobbit (Hold Life, WIS +4, low HP).

### 4.3 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Novice's Handbook] (town) | Call Light | 1 | 1 | 10 | |
| | Detect Evil | 1 | 1 | 10 | |
| | Minor Healing | 1 | 2 | 15 | heals L+10 + m_bonus, cures 20 cut/stun |
| | Bless | 1 | 2 | 20 | +5 AC, +10 to-hit, (L+10)+d(L+10) turns |
| | Sense Invisible | 3 | 4 | 25 | 24+1d24 turns |
| | Heroism | 5 | 2 | 30 | heals 10, cures fear; actual heroism only from CL20 |
| [Cleansing Power] (town) | Orb of Draining | 7 | 7 | 40 | ball 3L/2+3d6, ×2 vs evil, radius 3 from CL30 |
| | Spear of Light | 7 | 6 | 30 | 6d8 vs light-sensitive, lights a line |
| | Dispel Undead | 10 | 14 | 55 | d(5L) to undead in LOS |
| | Dispel Evil | 12 | 20 | 70 | d(5L) to evil in LOS |
| | Protection from Evil | 14 | 8 | 42 | 3L+d25 turns |
| | Remove Curse | 16 | 6 | 38 | |
| [Healing and Sanctuary] (DL30+) | Portal | 10 | 4 | 30 | teleport L/2+30 |
| | Remembrance | 20 | 30 | 90 | restore exp |
| | Word of Recall | 25 | 30 | 75 | |
| | Healing | 30 | 50 | 80 | heal 2000, cure cut/poison/stun/amnesia |
| | Restoration | 35 | 70 | 90 | restore all stats |
| | Clairvoyance | 37 | 50 | 80 | light and map whole level, sense objects |
| [Battle Blessings] (DL40+) | Enchant Weapon | 35 | 50 | 80 | |
| | Enchant Armour | 37 | 60 | 85 | |
| | Smite Evil | 25 | 20 | 70 | temporary slay evil |
| | Glyph of Warding | 20 | 40 | 90 | |
| | Demon Bane | 42 | 40 | 80 | temporary strong slay demon |
| [Wrath of the Valar] (DL60+) | Banish Evil | 25 | 25 | 80 | teleports away evil in LOS |
| | Word of Destruction | 35 | 35 | 80 | radius-15 destruction |
| | Holy Word | 39 | 32 | 95 | d(4L) to evil in LOS + heal 1000 + cures |
| | Spear of Oromë | 40 | 10 | 75 | holy beam `(L/2)d8`, ×2 vs evil |
| | Light of Varda | 42 | 40 | 85 | light ball 5L radius 4, then 100 to light-sensitive in LOS |

Random learning: when you study, a random spell among those you can learn in the chosen book is
taught. Early this rarely matters (one or two candidates per book).

### 4.4 Damage by phase

- **CL1-7:** Mace melee with `BLESS_WEAPON` (+2/+2) and Bless; thrown oil.
- **CL7-25:** **Orb of Draining** is the main weapon: 20 damage at CL7, 40 at CL20, doubled vs
  evil (most dangerous monsters are evil). It is unresisted except by the non-evil multiplier.
- **CL25-40:** Orb still (radius 3 from CL30, ~55/110 at CL30); Dispel Evil to hit everything in
  LOS; Smite Evil for blessed melee.
- **CL40-50:** Spear of Oromë (cheap holy beam), Holy Word, Light of Varda. Melee with a blessed
  weapon remains a valid backup.

### 4.5 Equipment priorities

WIS; a hafted or blessed weapon (bonus +2/+2); armour within 35 lb; Free Action; See Invisible
(or Sense Invisible prayers); base resists; CON. Since your heals are spells, carry Scrolls of
Phase Door / Teleportation / Deep Descent / Teleport Level as escapes until Portal.

### 4.6 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-10 | scrolls only (Phase Door, Teleportation), Flasks of Oil |
| CL10-30 | + Portal (CL10) **once you find [Healing and Sanctuary]** (DL30+, possibly Black Market) |
| CL20-35 | + Glyph of Warding (CL20, [Battle Blessings]); Protection from Evil |
| CL25-50 | + Banish Evil (CL25, [Wrath of the Valar]), Word of Destruction (CL35), Healing (CL30) as a "heal through it" option |

### 4.7 Typical deaths

Being caught without escapes in the long gap before Portal; non-evil monsters (hounds, animals,
vortices) that Orb hits for half value; mana exhaustion; blindness (Dwarves are immune) or
confusion stopping prayers; summoners.

### 4.8 Phase plan

- **CL1-10:** Bless + mace melee; buy [Cleansing Power] for Orb at CL7. Use Minor Healing
  between fights; keep the CSW potions for emergencies.
- **CL10-20:** Orb everything evil; Dispel Undead vs undead groups; Protection from Evil from
  CL14 makes evil melee monsters below your level miss often. Buy Phase/Teleport scrolls.
- **CL20-30:** hunt [Healing and Sanctuary] (Portal, Remembrance, Recall). Remove Curse handles
  sticky items.
- **CL30-40:** Healing at CL30 makes you very hard to kill by attrition; Restoration CL35. Look for
  [Battle Blessings] and [Wrath of the Valar].
- **CL40-50:** Spear of Oromë and Holy Word; Banish Evil to clear rooms; Word of Destruction as
  the panic button.

---

## 5. Necromancer

### 5.1 Identity and numbers

FACT: STR -3, INT +3, CON -2; hit die 2; max 4 blows; melee 35 (+25); device 36 (+13); save 30
(+9); `ZERO_FAIL`, `CHOOSE_SPELLS`, `UNLIGHT`, `EVIL`; INT; first spell CL1; 30 lb; 5 books, 26
spells. Kit: Main Gauche, [Into the Shadows], Word of Recall, food. **No light source.**

**Strengths:** strong attack rituals from CL1 (Nether Bolt), instant kills of weak monsters in
LOS (Crush), excellent late spells (Annihilate, Storm of Darkness, Unleash Chaos, Vampire
Form, Command), see in the dark, rDark and rNether. **Weaknesses:** **+25% spell failure on lit
squares**, unable to use normal light, vulnerable to holy orb, many spells hurt the caster, Nether
Bolt does nothing to undead and half to evil monsters, low HP.

### 5.2 How unlight works (FACT)

- With light radius ≤ 1 you "see" grids within distance `< 2 + CL/6 - light radius`
  (`cave-view.c`), i.e. out to distance `1 + CL/6` with no light: 1 (adjacent) at CL1-5, 2 at
  CL6, 3 at CL12, 4 at CL18, 5 at CL24, 6 at CL30, 7 at CL36, 8 at CL42, 9 at CL48-50. Monsters outside that radius in the dark are unseen unless lit, detected or
  infravision-visible.
- A torch or lantern (intensity 2 or more) turns this off and lights your own grid → +25% fail. A light
  source with only a +1 light modifier is treated as intensity 0.
- Lit rooms are lit grids: cast **Create Darkness** (CL3) in lit rooms before fighting.
- You resist darkness (blindness from dark attacks is prevented).
- `EVIL` gives rNether but makes holy orb (priest-type monster spells) hurt more.

### 5.3 Stats and races

INT, CON, DEX. Races: Gnome (Free Action, devices), High-Elf (See Invisible, INT), Dunadan,
Hobbit (Hold Life helps against the undead you fight), Elf. Infravision helps (see warm monsters
in the dark beyond the unlight radius): Gnome/Hobbit 40 ft, Dwarf/Kobold 50 ft.

### 5.4 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Into the Shadows] (town) | Nether Bolt | 1 | 1 | 22 | `(L/4+3)d4`; undead immune, evil take half |
| | Sense Invisible | 2 | 4 | 25 | |
| | Create Darkness | 3 | 3 | 10 | darkens area/room |
| | Bat Form | 3 | 4 | 20 | costs 5 HP |
| | Read Minds | 4 | 4 | 10 | detect monsters with minds and map around them |
| [Dark Rituals] (town) | Tap Unlife | 7 | 0 | 50 | hurts nearest undead `(L/4+2)d6`, gain 1/4 as mana |
| | Crush | 10 | 6 | 40 | kills every monster in LOS with HP < 4L; you take d(2L) |
| | Sleep Evil | 12 | 5 | 50 | sleeps evil in LOS |
| | Shadow Shift | 15 | 10 | 50 | teleport 30, costs 2d5 HP |
| | Disenchant | 16 | 12 | 50 | bolt 2d(2L+10) |
| [Fear and Torment] (town) | Frighten | 17 | 4 | 40 | |
| | Vampire Strike | 20 | 5 | 50 | jump to nearest living monster, drain up to 2L, heal |
| | Dispel Life | 22 | 20 | 50 | d(3L) to living in LOS |
| | Dark Spear | 25 | 10 | 60 | darkness beam 2d(2L) |
| | Warg Form | 28 | 20 | 60 | |
| [Deadly Powers] (DL40+) | Banish Spirits | 25 | 25 | 80 | teleports away monsters with spirits in LOS |
| | Annihilate | 30 | 20 | 75 | 4L to one non-undead, non-elemental, non-golem, non-vortex |
| | Grond's Blow | 35 | 35 | 80 | destruction radius 15 |
| | Unleash Chaos | 38 | 15 | 90 | random chaos beam/arc/ball `8dL`, wakes all, scares LOS |
| | Fume of Mordor | 40 | 50 | 80 | darkens and maps the level, detects objects |
| | Storm of Darkness | 43 | 16 | 80 | radius-4 dark ball 4d(2L) |
| [Corruption of Spirit] (DL60+) | Power Sacrifice | 27 | 0 | 20 | 50 HP for 50 mana |
| | Zone of Unmagic | 32 | 16 | 60 | radius-4 disenchant around you (hits you too) |
| | Vampire Form | 37 | 20 | 60 | vampire shape + jump-and-bite |
| | Curse | 40 | 50 | 60 | costs 100 HP; damage grows with the target's missing HP |
| | Command | 45 | 40 | 60 | control a monster |

Vampire Form's HP cost: the spell text says half your current HP; the shape file deals
`PLAYER_HP/4`. Which base `PLAYER_HP` uses was not verified; assume a large cost.

### 5.5 Damage by phase

- **CL1-10:** Nether Bolt (3d4 → 5d4). Against undead (immune) use melee or oil; against evil
  it does half. Read Minds to find targets.
- **CL10-20:** **Crush** is the defining spell: every monster in LOS with HP below 4×CL dies
  (40 HP at CL10, 80 at CL20), for 6 mana and d(2L) HP. Clear rooms of weak packs instantly.
  Tap Unlife farms mana from undead. Disenchant (CL16) for non-undead single targets.
- **CL20-30:** Dispel Life for LOS crowds of living monsters; Dark Spear beam; Vampire Strike
  for healing; Crush still kills anything under 4L.
- **CL30-50:** Annihilate (4L direct, 120-200), Storm of Darkness (big ball), Unleash Chaos; Curse
  for finishing big uniques; Command for tricks.

### 5.6 Equipment priorities

No light source (or one with only a +1 light modifier). INT; armour within 30 lb; Free Action;
See Invisible (bat form and vampire form give it temporarily); CON; rPois; resist light (light
attacks hurt; Elf/High-Elf race helps). Scrolls, wands and staffs of Darkness exist
(`lib/gamedata/object.txt`) and do the same job as Create Darkness; carry Phase Door and
Teleportation scrolls and CLW/CSW potions (no healing spell before Vampire Strike).

### 5.7 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-15 | scrolls; Bat Form (speed +3) for running; darkness to stay unseen while you retreat |
| CL15-35 | + Shadow Shift (CL15, teleport 30); Sleep Evil (CL12); Frighten (CL17) |
| CL25-50 | + Banish Spirits (CL25, [Deadly Powers]), Grond's Blow (CL35) |

### 5.8 Typical deaths

Fighting in a lit room (fail rates jump by 25%, capped at 50%); undead groups early (Nether Bolt
useless); self-damage stacking (Crush, Shadow Shift, Bat Form costs) at low HP; holy orb from
priest monsters; blindness from non-dark sources (rDark only prevents dark-based blindness),
which stops all rituals.

### 5.9 Phase plan

- **CL1-10:** do not buy a torch. Darken lit rooms. Kill with Nether Bolt; melee or oil undead.
  Buy [Dark Rituals] at CL7-10.
- **CL10-20:** Crush rooms clear; carry CLW to offset the HP cost; Shadow Shift at CL15.
- **CL20-30:** [Fear and Torment] (town but expensive): Vampire Strike, Dispel Life, Dark Spear.
- **CL30-40:** look for [Deadly Powers] (DL40+): Annihilate and Banish Spirits.
- **CL40-50:** Storm of Darkness, Fume of Mordor for map + darkness; [Corruption of Spirit] for
  Command and Curse.

---

## 6. Paladin

### 6.1 Identity and numbers

FACT: STR +1, INT -3, WIS +1, DEX -1, CON +2; hit die 6; max 5 blows, 3 lb floor, STR mult 5;
melee 65 (+40), shooting 50 (+30), device 24 (+10), save 25 (+11), stealth 0; `BLESS_WEAPON`,
`SHIELD_BASH`; random spell learning; WIS; first spell CL1; 40 lb allowance; 3 books, 16
spells; 5% minimum fail. Kit: Main Gauche, [Novice's Handbook], Scroll of Protection from Evil,
Word of Recall.

**Strengths:** near-warrior melee with Bless (+10 to-hit, +5 AC) from CL1, healing prayers,
Heroism from CL15, PfE, free Word of Recall and Healing (2000) from the dungeon book, late Smite
Evil, Demon Bane, Single Combat. **Weaknesses:** few spells, all buffs and utilities; no ranged
magic; poor stealth and devices.

### 6.2 Stats and races

STR and DEX for blows, CON, then WIS. Races: Dunadan, Dwarf (blindness immunity, WIS),
Half-Troll (HP, STR; WIS -1 total), High-Elf.

### 6.3 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Novice's Handbook] (town) | Bless | 1 | 2 | 20 | +5 AC, +10 to-hit |
| | Detect Evil | 3 | 2 | 10 | |
| | Call Light | 5 | 3 | 10 | |
| | Minor Healing | 7 | 3 | 15 | |
| | Sense Invisible | 8 | 4 | 25 | |
| | Heroism | 12 | 5 | 30 | real heroism (+12 to-hit, fear immunity) from CL15 |
| [Healing and Sanctuary] (DL30+) | Protection from Evil | 15 | 8 | 42 | |
| | Remove Curse | 20 | 12 | 38 | |
| | Word of Recall | 25 | 30 | 75 | |
| | Healing | 30 | 50 | 80 | 2000 HP |
| | Clairvoyance | 37 | 50 | 80 | |
| [Battle Blessings] (DL40+) | Smite Evil | 25 | 20 | 70 | slay evil |
| | Demon Bane | 30 | 40 | 80 | strong slay demon |
| | Enchant Weapon | 35 | 50 | 80 | |
| | Enchant Armour | 37 | 60 | 85 | |
| | Single Combat | 40 | 50 | 30 | arena with one monster (resisted if `randint0(spell power) > CL`) |

### 6.4 Damage by phase

Melee throughout. Bless (+10 to-hit) and Heroism (+12 to-hit) make even early weapons hit
reliably; hafted or blessed weapons add +2/+2. Shield bashes add occasional damage and stuns.
A launcher for things you should not melee. Late: Smite Evil turns any weapon into slay evil.

### 6.5 Equipment priorities

As Warrior (weapon with blows, Free Action, See Invisible, resists, speed) but watch the 40 lb
armour allowance (a heavy body armour plus shield exceeds it; each pound over costs 1 SP, and
Paladin SP are small). A hafted/blessed weapon is a mild bonus, not a requirement.

### 6.6 Escapes by phase

Scrolls and staves as a Warrior. From CL25 with [Healing and Sanctuary] the Word of Recall
prayer saves scrolls; Healing at CL30 lets you survive fights you could not escape.

### 6.7 Typical deaths

As Warrior, plus mana starvation when relying on Minor Healing; fear before Heroism (CL15).

### 6.8 Phase plan

- **CL1-10:** Bless before every fight; melee; oil and CLW. Detect Evil (CL3) on arrival.
- **CL10-20:** Heroism (CL12, real effect at CL15) solves fear; Free Action; launcher.
- **CL20-30:** find [Healing and Sanctuary]: PfE, Remove Curse, WoR.
- **CL30-40:** Healing; [Battle Blessings] for Smite Evil/Demon Bane.
- **CL40-50:** Single Combat vs dangerous uniques that you can out-melee (it prevents their
  escape and keeps summons away; the monster may resist). Enchant your own gear.

---

## 7. Rogue

### 7.1 Identity and numbers

FACT: INT +1, WIS -3, DEX +3, CON -1; hit die 4; max 5 blows, **2 lb** floor, STR mult 4; melee 35
(+45), shooting 66 (+30), **throwing 72 (+45)** (best), disarm 45/45 (+20, best), device 32
(+10), save 28 (+10), **stealth 3 (+1 per 10 levels, the only class whose stealth grows)**,
search 20 (+16, best); `CHOOSE_SPELLS`, `STEAL`; INT; first spell **CL5**; 35 lb; 2 books, 10
spells; 5% minimum fail. Kit: Dagger, Soft Leather Armour, [First Spells], Word of Recall.

**Strengths:** stealth (monsters stay asleep), detection of objects and monsters, the best
disarming and trap-spotting, throwing weapons, stealing items from monsters, Phase Door at CL7,
Recharging. **Weaknesses:** weak early melee; few spells; low HP; no attack magic.

### 7.2 Stealing (FACT, `mon-util.c:steal_monster_item()`)

`s` (original keyset) toward an adjacent monster that carries objects:

- Monster's guard = `monster level × 3/4` (×4/4 for uniques) + monster speed - your speed; halved
  if the monster is asleep.
- Your skill = stealth + DEX to-hit adjustment; quartered if blind, confused or hallucinating.
- Reaction = guard/2 + 1d(guard) + (item weight × number)/20.
- Success if reaction < skill: you take the item (gold goes straight to your purse); the monster's
  sleep is reduced by `35 - stealth`.
- Partial failure (reaction/2 < skill): the monster wakes and may notice you.
- Bad failure: the monster wakes, "cries out in anger" and wakes others nearby.
- With **Hit and Run** (CL23) active, after any steal attempt you teleport 20 squares away.
- If the monster has nothing, you are told so and it wakes 1 time in 3.

ADVICE: steal from *sleeping* monsters (guard halved) whose level is well below your
stealth+DEX; uniques guard more. Items monsters carry are their drops (you can see them with
Object Detection only once dropped, not while carried). Stealing is a way to get drops without a
fight and without waking the level; combine with Hit and Run to leave immediately.

### 7.3 Stats and races

DEX (blows with light weapons, to-hit, AC, theft, throwing accuracy), INT (spells, devices),
CON, STR for heavier throwing weapons and carrying. Races: Hobbit (stealth +4, DEX +6 total, Hold
Life), Kobold (rPois, stealth +3), High-Elf (See Invisible, skills), Gnome (Free Action),
Dunadan.

### 7.4 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [First Spells] (town) | Detect Monsters | 5 | 1 | 50 | |
| | Phase Door | 7 | 2 | 55 | |
| | Object Detection | 10 | 3 | 60 | gold, objects, ore |
| | Detect Stairs | 12 | 3 | 50 | wide area |
| | Recharging | 20 | 10 | 50 | power L/10+4 (weaker than the Mage's) |
| | Reveal Monsters | 25 | 3 | 40 | incl. invisible |
| [Arcane Control] (DL40+) | Teleport Self | 17 | 6 | 35 | |
| | Hit and Run | 23 | 20 | 40 | teleport 20 after next steal |
| | Teleport Other | 30 | 10 | 30 | |
| | Teleport Level | 35 | 17 | 65 | |

Base failure rates are high (50-60%) for the town spells; at CL10 Phase Door is `55 - 9 -
stat adj`, so with INT 18/20 (adj 7) about 39%. ADVICE: do not rely on the Phase Door spell
until its fail rate is under ~15%; carry scrolls.

### 7.5 Damage by phase

- **CL1-10:** dagger melee on sleeping monsters (stealth gives first strikes); thrown Flasks of
  Oil (2d6 fire) with the best throwing skill; a Sling.
- **CL10-25:** **throwing weapons** (daggers, spears and items with the `THROWING` flag are much
  better thrown: see [Chapter 4](04-combat.md)) and a launcher; melee improves quickly (+4.5
  melee skill per level, 2 lb floor = many blows with light weapons).
- **CL25-50:** melee with 4-5 blows from a light weapon at high DEX, plus thrown/launcher damage.
  Rogue melee skill at CL50 (260) is close to a Paladin's.

### 7.6 Equipment priorities

DEX, stealth items, Free Action, See Invisible, launcher (sling or bow), a stack of throwing
weapons, speed, CON. Armour within 35 lb.

### 7.7 Escapes by phase

| Phase | Escapes |
|---|---|
| CL1-7 | scrolls; stealth (just leave) |
| CL7-17 | + Phase Door spell (unreliable early) |
| CL17-35 | + Teleport Self (CL17, [Arcane Control] DL40+), Hit and Run (CL23) |
| CL30-50 | + Teleport Other (CL30), Teleport Level (CL35) |

### 7.8 Typical deaths

Waking a big pack by a bungled steal; low HP in early melee; relying on a 40%-fail Phase Door;
invisible monsters (Reveal Monsters only at CL25).

### 7.9 Phase plan

- **CL1-10:** sneak; kill sleeping weak monsters; throw oil; avoid awake groups. Detect Monsters
  at CL5.
- **CL10-20:** Object Detection (CL10) to grab good items and leave; Detect Stairs (CL12) for
  quick exits; collect throwing weapons; steal from sleeping monsters that carry items.
- **CL20-30:** Recharging; hunt [Arcane Control] for Teleport Self and Hit and Run.
- **CL30-40:** melee becomes strong; Teleport Other; deep diving on stealth.
- **CL40-50:** Teleport Level; end-game kit as for a fighter.

---

## 8. Ranger

### 8.1 Identity and numbers

FACT: WIS +2, DEX +1, CON -1; hit die 5; max 5 blows; melee 60 (+40); **shooting 72 (+45)** (best);
throwing 66 (+30); disarm 40/30; device 28 (+10); save 32 (+10); stealth 3; `FAST_SHOT`,
`CHOOSE_SPELLS`; WIS; first spell **CL3**; 40 lb; 2 books, 11 spells; 5% minimum fail. Kit: Main
Gauche, Short Bow, 15-20 Arrows, [Lesser Charms], Word of Recall.

**Strengths:** extra shots from level with any launcher (2.0 shots at CL30, 2.6 at CL48) on top of
the best shooting skill: the highest sustained ranged damage; good melee; Herbal Curing, Resist
Poison, mapping; Haste Self at CL25. **Weaknesses:** ammunition logistics; no teleport spells;
low HP compared to fighters.

### 8.2 Stats and races

DEX, STR (launcher weight and to-hit; heavy launchers lose extra shots and might when you cannot
hold them: `adj_str_hold`), CON, WIS. Races: High-Elf (shooting +25, See Invisible), Dunadan,
Elf, Kobold, Hobbit.

### 8.3 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Lesser Charms] (town) | Remove Hunger | 3 | 1 | 25 | |
| | Detect Life | 5 | 2 | 23 | |
| | Herbal Curing | 9 | 6 | 30 | cures cuts, poison, stun, Black Breath |
| | Resist Poison | 12 | 10 | 32 | |
| | Turn Stone to Mud | 15 | 5 | 25 | |
| | Sense Surroundings | 20 | 8 | 35 | |
| [Nature Craft] (DL40+) | Cover Tracks | 20 | 20 | 40 | no scent trail |
| | Create Arrows | 22 | 20 | 40 | turn a staff into arrows |
| | Haste Self | 25 | 12 | 65 | +10 speed, L+d20 turns |
| | Decoy | 30 | 30 | 40 | |
| | Brand Ammunition | 40 | 60 | 95 | random fire/cold/poison brand + enchant |

### 8.4 Damage by phase

- **CL1-10:** Short Bow, then Long Bow (×3 multiplier) as soon as possible; buy arrows in bulk
  (General Store and Weaponsmith stock them). Melee only weak things.
- **CL10-30:** shots grow by 0.1 every 3 levels; good arrows and an ego bow multiply damage.
  Fire from as far as possible; monsters approaching across a room take 2-4 volleys.
- **CL30-50:** 2-2.6 shots per turn with branded/slaying arrows and an extra-might bow is
  devastating. Brand Ammunition (CL40) and Create Arrows (CL22) solve supply.

### 8.5 Equipment priorities

The best launcher you can hold (Long Bow early; extra might / extra shots egos; artifacts),
**many** arrows (40-80+; they break), a quiver of special arrows for uniques, Free Action, See
Invisible, resists, speed. Armour within 40 lb.

### 8.6 Escapes by phase

Scrolls and staves throughout (no teleport spells). Haste Self (CL25, [Nature Craft]) is the
key defensive spell; Herbal Curing (CL9) handles poison/cuts/stun on the run.

### 8.7 Typical deaths

Running out of arrows mid-fight; fast melee monsters reaching you (speed matters: shooting does
not help in a corridor against 3 monsters adjacent); archers and breath attacks when your HP is
modest; invisible monsters you cannot target.

### 8.8 Phase plan

- **CL1-10:** buy a Long Bow and 40+ arrows; shoot everything; Remove Hunger at CL3.
- **CL10-20:** Herbal Curing (CL9), Resist Poison (CL12); upgrade arrows; Free Action.
- **CL20-30:** hunt [Nature Craft] (DL40+): Haste Self at CL25 is huge. Create Arrows from junk
  staves.
- **CL30-40:** ego/artifact launcher; stack speed; kill uniques at range.
- **CL40-50:** Brand Ammunition; endgame kit.

---

## 9. Blackguard

### 9.1 Identity and numbers

FACT: STR +2, WIS -3, CON +2; hit die 8; max 5 blows, **10 lb** weapon floor, STR mult 5; melee
65 (+40); shooting 35 (+15); throwing 40 (+30); device 24 (+10); save 18 (+10); stealth -1 (-1
per 10 levels: the only class whose stealth worsens); `CHOOSE_SPELLS`, `SHIELD_BASH`,
`COMBAT_REGEN`; object flag `IMPAIR_HP`; INT; first spell CL1; 60 lb allowance; 3 books, 15
spells; 5% minimum fail. Kit: Tulwar, Leather Shield, [Into the Shadows], Potion of CLW, Word of
Recall (starting gold is nearly gone).

### 9.2 Combat regeneration (FACT)

- **Mana is earned, not rested for.** Each melee attack command gives 5% of max SP (based on
  `max(msp,10)`, so at least 0.5 SP) (`player-attack.c:py_attack()`). Losing X% of your HP to
  damage gives X% of max SP (`player-util.c:take_hit()`; poison, bleeding and starvation do not
  count).
- **Mana decays** at half the rate other classes regenerate it (`player_regen_mana()`:
  `percent /= -2`), and decaying mana heals you at double the efficiency of casting.
- **Casting heals.** Spending SP on a spell heals a fraction of your missing HP
  (`convert_mana_to_hp()`: spending X% of max SP heals X/2 % of missing HP, at most 25% of missing
  HP per cast).
- **HP regeneration is halved** (`IMPAIR_HP`), and when you are above half HP, resting does not
  double mana changes.
- Resting until done does not wait for full SP (`player_resting_complete_special()`).

Implication (ADVICE): a Blackguard is strongest *in* a fight and weakest when entering one at low
HP after a long walk. Healing potions matter more than for other fighters. Cast spells mid-fight
both for their effect and for the heal.

### 9.3 Bloodlust (CL30 spell, FACT)

The Bloodlust spell adds 10 to the timed bloodlust counter; each kill while bloodlusted adds 10
more (`mon-util.c`). While active:

- +bloodlust/2 to-dam and +1 blow per 20 points (`calc_bonuses()`).
- If your HP would drop below 0, you survive while `HP + bloodlust + CL ≥ 0` (`take_hit()`).
- Each energy-using command has a `bloodlust/200` chance to be replaced by an attack on a random
  adjacent monster (`cmd-core.c`).
- Each kill risks confusion and hallucination; each miss 1/50 stat scrambling; each hit 1/50 CON
  damage.
- As it fades below 10, you may suffer HP loss, cuts and slowness each turn.

ADVICE: use it for big fights you intend to finish; never when you may need to read an escape
scroll (the forced attack can steal your turn).

### 9.4 Stats and races

STR (dominant: the 10 lb floor means blows come from STR vs weapon weight), CON, DEX, INT for
spells. Races: Half-Troll (STR/HP), Dunadan, Dwarf, Half-Orc. INT matters less than for other INT
casters since mana comes from combat, but spell fail rates still use INT.

### 9.5 Spells

| Book | Spell | Lvl | Mana | Fail | Notes |
|---|---|---|---|---|---|
| [Into the Shadows] (town) | Seek Battle | 1 | 1 | 15 | detects fear-susceptible monsters |
| | Berserk Strength | 3 | 2 | 20 | berserk (L+5)+d(L+5) turns: +75 melee skill, -10 AC, no healing over time |
| | Whirlwind Attack | 5 | 4 | 25 | blows on every adjacent monster: 1 at CL5, +1 per 15 levels |
| | Shatter Stone | 7 | 3 | 20 | |
| | Leap into Battle | 10 | 5 | 30 | move up to 4 and attack |
| | Grim Purpose | 13 | 8 | 30 | rConf + Free Action 12+d12 turns |
| [Fear and Torment] (town) | Maim Foe | 15 | 6 | 35 | L/15 blows, stun 6 on damage |
| | Howl of the Damned | 18 | 8 | 30 | scare LOS |
| | Relentless Taunting | 24 | 10 | 32 | monsters close in, fewer ranged attacks |
| | Venom | 28 | 18 | 35 | poison brand 18+d18 |
| | Werewolf Form | 32 | 30 | 40 | |
| [Deadly Powers] (DL40+) | Bloodlust | 30 | 25 | 37 | |
| | Unholy Reprieve | 34 | 45 | 60 | restore STR/INT/CON/exp for 66 HP |
| | Forceful Blow | 38 | 16 | 50 | one blow + force damage, knockback, stun |
| | Quake | 44 | 39 | 55 | radius-13 earthquake |

### 9.6 Damage by phase

Melee always. CL1-10: Tulwar (2d4) and Berserk Strength; whirlwind from CL5 against groups.
CL10-25: Leap into Battle to close on archers/casters; heavier weapons (no blow penalty up to 10
lb). CL25-50: Venom (poison brand), Werewolf Form (+1 blow, regen, faster movement), Bloodlust,
Forceful Blow.

### 9.7 Equipment priorities

Heavy weapon with good dice; a big shield (shield bash damage uses shield dice; 60 lb armour
allowance is generous); Free Action (Grim Purpose covers it temporarily from CL13); See
Invisible; resists; Potions of CLW/CSW/CCW (impaired regeneration); Regeneration items help offset
`IMPAIR_HP`.

### 9.8 Escapes by phase

Scrolls and staves only. Howl of the Damned (CL18) scares a room so you can walk away; Leap is
offensive only.

### 9.9 Typical deaths

Entering fights damaged (slow regeneration); bloodlust-forced attacks; ranged attackers that
avoid melee (Relentless Taunting at CL24 helps); bad stealth waking whole levels.

### 9.10 Phase plan

- **CL1-10:** buy CLW potions with the little gold you have; melee; Berserk Strength for hard
  fights; Whirlwind vs packs.
- **CL10-20:** Leap into Battle; Grim Purpose until you find Free Action; buy [Fear and Torment]
  when affordable.
- **CL20-30:** Maim Foe for stun-locking dangerous single monsters; Taunting vs archers; Venom.
- **CL30-40:** find [Deadly Powers]: Bloodlust, Unholy Reprieve; Werewolf Form.
- **CL40-50:** Forceful Blow; Quake to reshape fights; endgame kit as Warrior.

---

## Things to try (untested)

1. **Necromancer unlight vision as detection.** At CL30 the unlight radius (6) far exceeds a
   torch (2). Test whether a Necromancer exploring dark corridors spots sleeping monsters earlier than
   a lit character, by logging distance at first sighting.
2. **Druid Fox Form as a borg retreat move.** Fox Form doubles movement and grants Free Action
   for 2 SP at CL1. Test a borg rule "if no escape item is usable and the threat is slower than
   2× your speed, cast Fox Form and walk to the nearest stair", measuring survival vs reading
   Phase Door.
3. **Crush clearing as a level-feeling exploit.** At CL20 Crush kills everything in LOS under
   80 HP. Count how many rooms on DL15-25 a Necromancer can clear with one cast after Read Minds,
   and the HP cost per room.
4. **Priest Portal gap.** Measure how often a Priest finds [Healing and Sanctuary] before CL20
   (Black Market or drops). If rarely, recommend Priests buy Teleportation scrolls in bulk at
   CL10-20 and rate the class accordingly.
5. **Rogue steal economy under no-selling.** With `birth_no_selling`, gold is the only income.
   Track gold per hour from stealing gold from sleeping `p`/`h` monsters vs killing them.
6. **Ranger sling vs bow.** `FAST_SHOT` applies to any launcher. Compare a Ranger with an ego
   Sling of extra shots against a Long Bow at CL20-30 (damage per turn, ammo weight).
7. **Blackguard pre-fight heal ritual.** Since casting heals proportional to missing HP, test
   whether casting cheap spells (Seek Battle, 1 SP) repeatedly after a fight is an efficient
   heal compared with resting, given impaired regeneration.
8. **Paladin Single Combat vs dangerous summoners.** The arena isolates one monster. Test
   against summoning uniques that a Paladin can out-melee; record resist rate
   (`randint0(spell power) > CL`) and outcomes.
9. **Mage Door Creation + Phase Door "box".** Door Creation at CL13 surrounds you with doors;
   test whether it reliably buys time against non-door-opening packs (hounds, animals) compared
   with Phase Door alone.
10. **Warrior BRAVERY_30 race-to-30.** Measure how many Warrior deaths before CL30 involve fear.
    If many, a borg should treat "afraid and no cure" as an emergency equal to low HP.
11. **Druid Pukel-man as a tank against poison breathers.** Pukel-man grants poison immunity
    and damage reduction 10 at -5 speed. Test against drake and hound packs at DL30-40.
12. **Bloodlust survival floor.** With bloodlust 50 at CL40 you survive down to -90 HP. Test
    whether chaining kills to stack bloodlust before a unique fight improves win rate, and how
    often forced attacks cause deaths.

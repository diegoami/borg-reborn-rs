# Appendix A8. Races, classes and player tables

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/p_race.txt`, `lib/gamedata/class.txt`, `lib/gamedata/player_property.txt`, `src/player-calcs.c`, `src/player-spell.c`, `src/player.c` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Race and class statistics (stat modifiers, skills, hit dice, experience factor, infravision, intrinsic flags), class melee-blow parameters, class skill progressions, starting kits and titles, the player experience table and the stat-to-bonus tables from the C source. Choosing a combination: [Chapter 2](../02-character-creation.md); the formulas behind the tables: [Chapter 1](../01-core-mechanics.md).

## Contents

- [How the numbers combine](#how-the-numbers-combine)
- [Races](#races)
- [Classes](#classes)
- [Class skill progression](#class-skill-progression)
  - [Disarm (phys) by level](#disarm-phys-by-level)
  - [Disarm (magic) by level](#disarm-magic-by-level)
  - [Device by level](#device-by-level)
  - [Save by level](#save-by-level)
  - [Stealth by level](#stealth-by-level)
  - [Search by level](#search-by-level)
  - [Melee by level](#melee-by-level)
  - [Shoot by level](#shoot-by-level)
  - [Throw by level](#throw-by-level)
  - [Dig by level](#dig-by-level)
- [Starting equipment](#starting-equipment)
- [Titles](#titles)
- [Experience table](#experience-table)
- [Stat tables](#stat-tables)
  - [Blows table](#blows-table)

## How the numbers combine

- **Stats**: race + class modifier applied to the rolled/bought stat (`src/player-birth.c`, `src/player-calcs.c`).
- **Skills**: race skill + class base + `class increment * level / 10` (`calc_bonuses()` in `src/player-calcs.c`), plus stat bonuses (`adj_dex_dis`, `adj_int_dis`, `adj_int_dev`, `adj_wis_sav`, `adj_str_dig` below) and equipment. Stealth is clamped to 0..30, digging to at least 1.
- **Hit dice**: player hit die = race `hitdie` + class `hitdie`; HP per level is rolled from it, plus `adj_con_mhp / 100` per level.
- **Experience factor**: race `exp` + class `exp` percent; the experience needed for level L is `player_exp[L-2] * expfact / 100`.
- **Blows**: `calc_blows()` in `src/player-calcs.c`: `P = min(11, adj_str_blow[STR] * strength-multiplier / max(weapon weight, min-weight))`, `D = min(11, adj_dex_blow[DEX])`, energy per blow = `blows_table[P][D]`, blows = `min(100 / energy, max-attacks)` (as hundredths: `10000 / energy`), plus extra blows from equipment.

## Races

| Race | STR | INT | WIS | DEX | CON | Hit die | Exp | Infra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Human | +0 | +0 | +0 | +0 | +0 | 10 | 100% | 0 ft |
| Half-Elf | +0 | +1 | -1 | +1 | -1 | 10 | 120% | 20 ft |
| Elf | -1 | +2 | -1 | +1 | -1 | 9 | 120% | 30 ft |
| Hobbit | -2 | +2 | +1 | +3 | +2 | 7 | 120% | 40 ft |
| Gnome | -1 | +2 | +0 | +2 | +1 | 8 | 120% | 40 ft |
| Dwarf | +2 | -3 | +2 | -2 | +2 | 11 | 120% | 50 ft |
| Half-Orc | +2 | -1 | +0 | +0 | +1 | 10 | 120% | 30 ft |
| Half-Troll | +4 | -4 | -2 | -4 | +3 | 12 | 120% | 30 ft |
| Dunadan | +1 | +2 | +2 | +2 | +3 | 10 | 120% | 0 ft |
| High-Elf | +1 | +3 | -1 | +3 | +1 | 10 | 145% | 40 ft |
| Kobold | -1 | -1 | +0 | +2 | +2 | 8 | 120% | 50 ft |

| Race | Disarm (phys) | Disarm (magic) | Device | Save | Stealth | Search | Melee | Shoot | Throw | Dig |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Human | +0 | +0 | +0 | +0 | +0 | +0 | +0 | +0 | +0 | +0 |
| Half-Elf | +2 | +2 | +3 | +3 | +1 | +3 | -1 | +5 | +5 | +0 |
| Elf | +5 | +5 | +6 | +6 | +2 | +6 | -5 | +15 | +15 | +0 |
| Hobbit | +15 | +15 | +18 | +18 | +4 | +6 | -10 | +20 | +20 | +0 |
| Gnome | +10 | +10 | +22 | +12 | +3 | +4 | -8 | +12 | +12 | +0 |
| Dwarf | +2 | +2 | +9 | +9 | -1 | +2 | +15 | +0 | +0 | +40 |
| Half-Orc | -3 | -3 | -3 | -3 | -1 | -3 | +12 | -5 | -5 | +0 |
| Half-Troll | -5 | -5 | -8 | -8 | -2 | -9 | +20 | -10 | -10 | +0 |
| Dunadan | +4 | +4 | +5 | +5 | +1 | +3 | +15 | +10 | +10 | +0 |
| High-Elf | +4 | +4 | +20 | +20 | +2 | +10 | +10 | +25 | +25 | +0 |
| Kobold | +10 | +10 | +5 | +0 | +3 | +10 | -5 | +10 | +10 | +0 |

| Race | Intrinsic abilities | Age (base:rand) | Height | Weight |
|---|---|---|---|---|
| Human | - | 14:6 | 69:10 | 165:35 |
| Half-Elf | sustain dexterity (SUST\_DEX) | 24:16 | 71:8 | 115:25 |
| Elf | sustain dexterity (SUST\_DEX); resist light | 75:75 | 73:7 | 90:10 |
| Hobbit | hold life (HOLD\_LIFE); Mushroom Lore (KNOW\_MUSHROOM): You easily recognize mushrooms. | 21:12 | 34:4 | 55:5 |
| Gnome | free action (FREE\_ACT); Device Lore (KNOW\_ZAPPER): You easily recognize magic devices. | 50:40 | 40:5 | 80:5 |
| Dwarf | protection from blindness (PROT\_BLIND); Miner (SEE\_ORE): You can sense ore in the walls. | 35:15 | 47:4 | 135:20 |
| Half-Orc | resist dark | 11:4 | 64:2 | 135:15 |
| Half-Troll | sustain strength (SUST\_STR); regeneration (REGEN) | 20:10 | 90:16 | 240:60 |
| Dunadan | sustain constitution (SUST\_CON) | 50:20 | 80:6 | 190:20 |
| High-Elf | see invisible (SEE\_INVIS); resist light | 100:30 | 77:6 | 190:20 |
| Kobold | resist poison | 15:10 | 37:4 | 65:5 |

## Classes

| Class | STR | INT | WIS | DEX | CON | Hit die | Exp | Max blows | Min weight | STR mult |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | +3 | -2 | -2 | +2 | +2 | 9 | 0% | 6 | 30 (3.0 lb) | 5 |
| Mage | -3 | +3 | +0 | +0 | -2 | 0 | 0% | 4 | 40 (4.0 lb) | 2 |
| Druid | -2 | +0 | +3 | -2 | +0 | 2 | 0% | 4 | 35 (3.5 lb) | 3 |
| Priest | -1 | -3 | +3 | -1 | +1 | 2 | 0% | 4 | 35 (3.5 lb) | 3 |
| Necromancer | -3 | +3 | +0 | +0 | -2 | 2 | 0% | 4 | 35 (3.5 lb) | 3 |
| Paladin | +1 | -3 | +1 | -1 | +2 | 6 | 0% | 5 | 30 (3.0 lb) | 5 |
| Rogue | +0 | +1 | -3 | +3 | -1 | 4 | 0% | 5 | 20 (2.0 lb) | 4 |
| Ranger | +0 | +0 | +2 | +1 | -1 | 5 | 0% | 5 | 35 (3.5 lb) | 4 |
| Blackguard | +2 | +0 | -3 | +0 | +2 | 8 | 0% | 5 | 100 (10.0 lb) | 5 |

Class skills: `base / +increment per 10 levels`.

| Class | Disarm (phys) | Disarm (magic) | Device | Save | Stealth | Search | Melee | Shoot | Throw | Dig |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 25 / +15 | 20 / +10 | 18 / +7 | 18 / +10 | 0 / +0 | 10 / +12 | 70 / +45 | 55 / +45 | 55 / +45 | 0 / +0 |
| Mage | 30 / +10 | 35 / +12 | 36 / +13 | 30 / +9 | 2 / +0 | 10 / +12 | 35 / +15 | 20 / +15 | 20 / +15 | 0 / +0 |
| Druid | 30 / +10 | 30 / +10 | 24 / +10 | 30 / +10 | 3 / +0 | 12 / +12 | 45 / +20 | 40 / +30 | 40 / +30 | 0 / +0 |
| Priest | 25 / +12 | 25 / +12 | 30 / +10 | 32 / +12 | 2 / +0 | 10 / +14 | 45 / +20 | 35 / +20 | 35 / +20 | 0 / +0 |
| Necromancer | 30 / +10 | 35 / +12 | 36 / +13 | 30 / +9 | 2 / +0 | 10 / +12 | 35 / +25 | 20 / +15 | 20 / +15 | 0 / +0 |
| Paladin | 20 / +15 | 20 / +10 | 24 / +10 | 25 / +11 | 0 / +0 | 10 / +12 | 65 / +40 | 50 / +30 | 50 / +30 | 0 / +0 |
| Rogue | 45 / +20 | 45 / +20 | 32 / +10 | 28 / +10 | 3 / +1 | 20 / +16 | 35 / +45 | 66 / +30 | 72 / +45 | 0 / +0 |
| Ranger | 40 / +15 | 30 / +10 | 28 / +10 | 32 / +10 | 3 / +0 | 15 / +15 | 60 / +40 | 72 / +45 | 66 / +30 | 0 / +0 |
| Blackguard | 20 / +15 | 20 / +10 | 24 / +10 | 18 / +10 | -1 / -1 | 8 / +10 | 65 / +40 | 35 / +15 | 40 / +30 | 0 / +0 |

| Class | Class abilities | Magic |
|---|---|---|
| Warrior | Relentless \[30\] (BRAVERY\_30): You become immune to fear at level 30.; No Magic (NO\_MANA): You cannot cast spells.; Shield Bash (SHIELD\_BASH): You can bash monsters with a shield in melee. | - |
| Mage | Full Spellcaster (ZERO\_FAIL): You may obtain a perfect success rate with magic.; Extra Spell Beaming (BEAM): Your bolt spells frequently turn into beams.; Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study. | arcane, first spell level 1, armour weight limit 30.0 lb, 5 books |
| Druid | Full Spellcaster (ZERO\_FAIL): You may obtain a perfect success rate with magic.; Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study.; Charms (CHARM): You are extra persuasive to monsters. | nature, first spell level 1, armour weight limit 35.0 lb, 5 books |
| Priest | Blessed Fighter (BLESS\_WEAPON): You get improved melee with blessed or hafted weapons.; Full Spellcaster (ZERO\_FAIL): You may obtain a perfect success rate with magic. | divine, first spell level 1, armour weight limit 35.0 lb, 5 books |
| Necromancer | Full Spellcaster (ZERO\_FAIL): You may obtain a perfect success rate with magic.; Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study.; Unlight (UNLIGHT): You gain stealth in, can see in, and resist the dark.; Evil (EVIL): You resist nether, but are hurt by holy attacks. | shadow, first spell level 1, armour weight limit 30.0 lb, 5 books |
| Paladin | Blessed Fighter (BLESS\_WEAPON): You get improved melee with blessed or hafted weapons.; Shield Bash (SHIELD\_BASH): You can bash monsters with a shield in melee. | divine, first spell level 1, armour weight limit 40.0 lb, 3 books |
| Rogue | Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study.; Pickpocketing (STEAL): You can steal from monsters. | arcane, first spell level 5, armour weight limit 35.0 lb, 2 books |
| Ranger | Extra Shots (FAST\_SHOT): Your shooting speed improves every 3 levels.; Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study. | nature, first spell level 3, armour weight limit 40.0 lb, 2 books |
| Blackguard | Spell Choice (CHOOSE\_SPELLS): You may choose your own spells to study.; Shield Bash (SHIELD\_BASH): You can bash monsters with a shield in melee.; Combat Regeneration (COMBAT\_REGEN): You draw power from the thrill of combat, represented by Spell Points (SP). You gain SP when damaged by an enemy or when you attack in melee. As your blood cools your power fades; you lose SP at half the rate other classes gain them. When you spend SP by casting a spell you regain some health, and fading SP replenish your health more efficiently. The more damaged you are, the bigger these health gains will be.; impaired hitpoint recovery (IMPAIR\_HP) | shadow, first spell level 1, armour weight limit 60.0 lb, 3 books |

## Class skill progression

Class contribution only (`base + increment * level / 10`, integer division); add the race skill, stat bonus and equipment. Stealth is shown before the 0..30 clamp.

### Disarm (phys) by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 26 | 32 | 40 | 47 | 55 | 62 | 70 | 77 | 85 | 92 | 100 |
| Mage | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Druid | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Priest | 26 | 31 | 37 | 43 | 49 | 55 | 61 | 67 | 73 | 79 | 85 |
| Necromancer | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Paladin | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |
| Rogue | 47 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 | 145 |
| Ranger | 41 | 47 | 55 | 62 | 70 | 77 | 85 | 92 | 100 | 107 | 115 |
| Blackguard | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |

### Disarm (magic) by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 21 | 25 | 30 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 |
| Mage | 36 | 41 | 47 | 53 | 59 | 65 | 71 | 77 | 83 | 89 | 95 |
| Druid | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Priest | 26 | 31 | 37 | 43 | 49 | 55 | 61 | 67 | 73 | 79 | 85 |
| Necromancer | 36 | 41 | 47 | 53 | 59 | 65 | 71 | 77 | 83 | 89 | 95 |
| Paladin | 21 | 25 | 30 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 |
| Rogue | 47 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 | 145 |
| Ranger | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Blackguard | 21 | 25 | 30 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 |

### Device by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 18 | 21 | 25 | 28 | 32 | 35 | 39 | 42 | 46 | 49 | 53 |
| Mage | 37 | 42 | 49 | 55 | 62 | 68 | 75 | 81 | 88 | 94 | 101 |
| Druid | 25 | 29 | 34 | 39 | 44 | 49 | 54 | 59 | 64 | 69 | 74 |
| Priest | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Necromancer | 37 | 42 | 49 | 55 | 62 | 68 | 75 | 81 | 88 | 94 | 101 |
| Paladin | 25 | 29 | 34 | 39 | 44 | 49 | 54 | 59 | 64 | 69 | 74 |
| Rogue | 33 | 37 | 42 | 47 | 52 | 57 | 62 | 67 | 72 | 77 | 82 |
| Ranger | 29 | 33 | 38 | 43 | 48 | 53 | 58 | 63 | 68 | 73 | 78 |
| Blackguard | 25 | 29 | 34 | 39 | 44 | 49 | 54 | 59 | 64 | 69 | 74 |

### Save by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 19 | 23 | 28 | 33 | 38 | 43 | 48 | 53 | 58 | 63 | 68 |
| Mage | 30 | 34 | 39 | 43 | 48 | 52 | 57 | 61 | 66 | 70 | 75 |
| Druid | 31 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 |
| Priest | 33 | 38 | 44 | 50 | 56 | 62 | 68 | 74 | 80 | 86 | 92 |
| Necromancer | 30 | 34 | 39 | 43 | 48 | 52 | 57 | 61 | 66 | 70 | 75 |
| Paladin | 26 | 30 | 36 | 41 | 47 | 52 | 58 | 63 | 69 | 74 | 80 |
| Rogue | 29 | 33 | 38 | 43 | 48 | 53 | 58 | 63 | 68 | 73 | 78 |
| Ranger | 33 | 37 | 42 | 47 | 52 | 57 | 62 | 67 | 72 | 77 | 82 |
| Blackguard | 19 | 23 | 28 | 33 | 38 | 43 | 48 | 53 | 58 | 63 | 68 |

### Stealth by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Mage | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Druid | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| Priest | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Necromancer | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Paladin | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Rogue | 3 | 3 | 4 | 4 | 5 | 5 | 6 | 6 | 7 | 7 | 8 |
| Ranger | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| Blackguard | -1 | -1 | -2 | -2 | -3 | -3 | -4 | -4 | -5 | -5 | -6 |

### Search by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 11 | 16 | 22 | 28 | 34 | 40 | 46 | 52 | 58 | 64 | 70 |
| Mage | 11 | 16 | 22 | 28 | 34 | 40 | 46 | 52 | 58 | 64 | 70 |
| Druid | 13 | 18 | 24 | 30 | 36 | 42 | 48 | 54 | 60 | 66 | 72 |
| Priest | 11 | 17 | 24 | 31 | 38 | 45 | 52 | 59 | 66 | 73 | 80 |
| Necromancer | 11 | 16 | 22 | 28 | 34 | 40 | 46 | 52 | 58 | 64 | 70 |
| Paladin | 11 | 16 | 22 | 28 | 34 | 40 | 46 | 52 | 58 | 64 | 70 |
| Rogue | 21 | 28 | 36 | 44 | 52 | 60 | 68 | 76 | 84 | 92 | 100 |
| Ranger | 16 | 22 | 30 | 37 | 45 | 52 | 60 | 67 | 75 | 82 | 90 |
| Blackguard | 9 | 13 | 18 | 23 | 28 | 33 | 38 | 43 | 48 | 53 | 58 |

### Melee by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 74 | 92 | 115 | 137 | 160 | 182 | 205 | 227 | 250 | 272 | 295 |
| Mage | 36 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 | 102 | 110 |
| Druid | 47 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 | 145 |
| Priest | 47 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 | 145 |
| Necromancer | 37 | 47 | 60 | 72 | 85 | 97 | 110 | 122 | 135 | 147 | 160 |
| Paladin | 69 | 85 | 105 | 125 | 145 | 165 | 185 | 205 | 225 | 245 | 265 |
| Rogue | 39 | 57 | 80 | 102 | 125 | 147 | 170 | 192 | 215 | 237 | 260 |
| Ranger | 64 | 80 | 100 | 120 | 140 | 160 | 180 | 200 | 220 | 240 | 260 |
| Blackguard | 69 | 85 | 105 | 125 | 145 | 165 | 185 | 205 | 225 | 245 | 265 |

### Shoot by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 59 | 77 | 100 | 122 | 145 | 167 | 190 | 212 | 235 | 257 | 280 |
| Mage | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |
| Druid | 43 | 55 | 70 | 85 | 100 | 115 | 130 | 145 | 160 | 175 | 190 |
| Priest | 37 | 45 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 |
| Necromancer | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |
| Paladin | 53 | 65 | 80 | 95 | 110 | 125 | 140 | 155 | 170 | 185 | 200 |
| Rogue | 69 | 81 | 96 | 111 | 126 | 141 | 156 | 171 | 186 | 201 | 216 |
| Ranger | 76 | 94 | 117 | 139 | 162 | 184 | 207 | 229 | 252 | 274 | 297 |
| Blackguard | 36 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 | 102 | 110 |

### Throw by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 59 | 77 | 100 | 122 | 145 | 167 | 190 | 212 | 235 | 257 | 280 |
| Mage | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |
| Druid | 43 | 55 | 70 | 85 | 100 | 115 | 130 | 145 | 160 | 175 | 190 |
| Priest | 37 | 45 | 55 | 65 | 75 | 85 | 95 | 105 | 115 | 125 | 135 |
| Necromancer | 21 | 27 | 35 | 42 | 50 | 57 | 65 | 72 | 80 | 87 | 95 |
| Paladin | 53 | 65 | 80 | 95 | 110 | 125 | 140 | 155 | 170 | 185 | 200 |
| Rogue | 76 | 94 | 117 | 139 | 162 | 184 | 207 | 229 | 252 | 274 | 297 |
| Ranger | 69 | 81 | 96 | 111 | 126 | 141 | 156 | 171 | 186 | 201 | 216 |
| Blackguard | 43 | 55 | 70 | 85 | 100 | 115 | 130 | 145 | 160 | 175 | 190 |

### Dig by level

| Class | L1 | L5 | L10 | L15 | L20 | L25 | L30 | L35 | L40 | L45 | L50 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Warrior | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Mage | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Druid | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Priest | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Necromancer | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Paladin | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Rogue | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Ranger | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Blackguard | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## Starting equipment

`equip:` lines: item, quantity range, and birth options that remove it (e.g. `birth_no_recall` removes the Word of Recall scroll). Starting gold is `player:start-gold` (600) plus 50 per unspent point-buy point; the value of each kit item given is deducted from it, and with `birth_start_kit` off only one food item and one light are given (`player_outfit()` and `recalculate_stats()` in `src/player-birth.c`). See [Chapter 2](../02-character-creation.md).

| Class | Starting kit |
|---|---|
| Warrior | Ration of Food x1-3; Wooden Torch x1-3; Berserk Strength x1; Dagger x1; Soft Leather Armour x1; Word of Recall x1 (not with birth\_no\_recall) |
| Mage | Ration of Food x1-3; Wooden Torch x1-3; Rapier x1; Word of Recall x1 (not with birth\_no\_recall); \[First Spells\] x1 |
| Druid | Ration of Food x1-3; Whip x1; Wooden Torch x1-3; Word of Recall x1 (not with birth\_no\_recall); \[Lesser Charms\] x1 |
| Priest | Ration of Food x1-3; Wooden Torch x1-3; Mace x1; Cure Serious Wounds x2; Word of Recall x1 (not with birth\_no\_recall); \[Novice's Handbook\] x1 |
| Necromancer | Ration of Food x1-3; Main Gauche x1; Word of Recall x1 (not with birth\_no\_recall); \[Into the Shadows\] x1 |
| Paladin | Ration of Food x1-3; Wooden Torch x1-3; Main Gauche x1; Protection from Evil x1; Word of Recall x1 (not with birth\_no\_recall); \[Novice's Handbook\] x1 |
| Rogue | Ration of Food x1-3; Wooden Torch x1-3; Dagger x1; Soft Leather Armour x1; Word of Recall x1 (not with birth\_no\_recall); \[First Spells\] x1 |
| Ranger | Ration of Food x1-3; Wooden Torch x1-3; Main Gauche x1; Short Bow x1; Arrow x15-20; Word of Recall x1 (not with birth\_no\_recall); \[Lesser Charms\] x1 |
| Blackguard | \[Into the Shadows\] x1; Ration of Food x1-3; Wooden Torch x1-3; Tulwar x1; Leather Shield x1; Word of Recall x1 (not with birth\_no\_recall); Cure Light Wounds x1 |

## Titles

Title index = `(level - 1) / 5` (`src/ui-death.c`, character sheet).

| Class | L1-5 | L6-10 | L11-15 | L16-20 | L21-25 | L26-30 | L31-35 | L36-40 | L41-45 | L46-50 |
|---|---|---|---|---|---|---|---|---|---|---|
| Warrior | Rookie | Soldier | Swordsman | Swashbuckler | Veteran | Myrmidon | Commando | Champion | Hero | Lord |
| Mage | Novice | Apprentice | Trickster | Illusionist | Spellbinder | Evoker | Conjurer | Warlock | Sorcerer | Arch-Mage |
| Druid | Wanderer | Tamer | Nurturer | Gardener | Forester | Creator | Earth Warder | Windrider | Stormwielder | High Mystic |
| Priest | Believer | Acolyte | Devotee | Adept | Evangelist | Priest | Elder | Prophet | Patriarch | High Priest |
| Necromancer | Acolyte | Curser | Dark Student | Initiate | Slavemaster | Summoner | Controller | Commander | Dark Master | Night Lord |
| Paladin | Gallant | Keeper | Protector | Defender | Warder | Knight | Guardian | Chevalier | Paladin | Paladin Lord |
| Rogue | Vagabond | Cutpurse | Footpad | Robber | Burglar | Filcher | Sharper | Rogue | Thief | Master Thief |
| Ranger | Runner | Strider | Scout | Courser | Tracker | Guide | Explorer | Pathfinder | Ranger | Ranger Lord |
| Blackguard | Rat | Bully | Thug | Ruffian | Brigand | Raider | Tormentor | Marauder | Destroyer | Tyrant |

## Experience table

`player_exp[]` in `src/player.c`: base experience to reach each level; multiply by the race+class experience factor / 100. The maximum level is 50 (`PY_MAX_LEVEL`), so the array's last entry is never used. Values below are for factor 100% and for the other factors that occur.

| Level | exp at 100% | exp at 120% |
|---:|---:|---:|
| 2 | 10 | 12 |
| 3 | 25 | 30 |
| 4 | 45 | 54 |
| 5 | 70 | 84 |
| 6 | 100 | 120 |
| 7 | 140 | 168 |
| 8 | 200 | 240 |
| 9 | 280 | 336 |
| 10 | 380 | 456 |
| 11 | 500 | 600 |
| 12 | 650 | 780 |
| 13 | 850 | 1020 |
| 14 | 1100 | 1320 |
| 15 | 1400 | 1680 |
| 16 | 1800 | 2160 |
| 17 | 2300 | 2760 |
| 18 | 2900 | 3480 |
| 19 | 3600 | 4320 |
| 20 | 4400 | 5280 |
| 21 | 5400 | 6480 |
| 22 | 6800 | 8160 |
| 23 | 8400 | 10080 |
| 24 | 10200 | 12240 |
| 25 | 12500 | 15000 |
| 26 | 17500 | 21000 |
| 27 | 25000 | 30000 |
| 28 | 35000 | 42000 |
| 29 | 50000 | 60000 |
| 30 | 75000 | 90000 |
| 31 | 100000 | 120000 |
| 32 | 150000 | 180000 |
| 33 | 200000 | 240000 |
| 34 | 275000 | 330000 |
| 35 | 350000 | 420000 |
| 36 | 450000 | 540000 |
| 37 | 550000 | 660000 |
| 38 | 700000 | 840000 |
| 39 | 850000 | 1020000 |
| 40 | 1000000 | 1200000 |
| 41 | 1250000 | 1500000 |
| 42 | 1500000 | 1800000 |
| 43 | 1800000 | 2160000 |
| 44 | 2100000 | 2520000 |
| 45 | 2400000 | 2880000 |
| 46 | 2700000 | 3240000 |
| 47 | 3000000 | 3600000 |
| 48 | 3500000 | 4200000 |
| 49 | 4000000 | 4800000 |
| 50 | 4500000 | 5400000 |

Experience factors of all race/class combinations range from 100% to 145%.

## Stat tables

Transcribed by the generator from the C arrays named below. Index label is the stat value (`18/xx` = 18 plus xx percentile points). The comment above each array in the source is quoted as the meaning.

| Array | File | Meaning |
|---|---|---|
| `adj_str_td` | `src/player-calcs.c` | Stat Table (STR) -- bonus to dam |
| `adj_str_th` | `src/player-calcs.c` | Stat Table (STR) -- bonus to hit |
| `adj_dex_th` | `src/player-calcs.c` | Stat Table (DEX) -- bonus to hit |
| `adj_dex_ta` | `src/player-calcs.c` | Stat Table (DEX) -- bonus to ac |
| `adj_str_wgt` | `src/player-calcs.c` | Stat Table (STR) -- weight limit in deca-pounds |
| `adj_str_hold` | `src/player-calcs.c` | Stat Table (STR) -- weapon weight limit in pounds |
| `adj_str_dig` | `src/player-calcs.c` | Stat Table (STR) -- digging value |
| `adj_str_blow` | `src/player-calcs.c` | Stat Table (STR) -- help index into the "blow" table |
| `adj_dex_blow` | `src/player-calcs.c` | Stat Table (DEX) -- index into the "blow" table |
| `adj_dex_safe` | `src/player-calcs.c` | Stat Table (DEX) -- chance of avoiding "theft" and "falling" |
| `adj_con_fix` | `src/player-calcs.c` | Stat Table (CON) -- base regeneration rate |
| `adj_con_mhp` | `src/player-calcs.c` | Stat Table (CON) -- extra 1/100th hitpoints per level |
| `adj_dex_dis` | `src/player-calcs.c` | Stat Table (DEX) -- disarming |
| `adj_int_dis` | `src/player-calcs.c` | Stat Table (INT) -- disarming |
| `adj_int_dev` | `src/player-calcs.c` | Stat Table (INT) -- Magic devices |
| `adj_wis_sav` | `src/player-calcs.c` | Stat Table (WIS) -- Saving throw |
| `adj_mag_study` | `src/player-calcs.c` | (no comment in source) |
| `adj_mag_mana` | `src/player-calcs.c` | Stat Table (INT/WIS) -- extra 1/100 mana-points per level |
| `adj_mag_fail` | `src/player-spell.c` | Stat Table (INT/WIS) -- Minimum failure rate (percentage) |
| `adj_mag_stat` | `src/player-spell.c` | Stat Table (INT/WIS) -- failure rate adjustment |

| Stat | `str_td` | `str_th` | `dex_th` | `dex_ta` | `str_wgt` | `str_hold` | `str_dig` | `str_blow` | `dex_blow` | `dex_safe` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 3 | -2 | -3 | -3 | -4 | 5 | 4 | 0 | 3 | 0 | 0 |
| 4 | -2 | -2 | -2 | -3 | 6 | 5 | 0 | 4 | 0 | 1 |
| 5 | -1 | -1 | -2 | -2 | 7 | 6 | 1 | 5 | 0 | 2 |
| 6 | -1 | -1 | -1 | -1 | 8 | 7 | 2 | 6 | 0 | 3 |
| 7 | 0 | 0 | -1 | 0 | 9 | 8 | 3 | 7 | 0 | 4 |
| 8 | 0 | 0 | 0 | 0 | 10 | 10 | 4 | 8 | 0 | 5 |
| 9 | 0 | 0 | 0 | 0 | 11 | 12 | 4 | 9 | 0 | 5 |
| 10 | 0 | 0 | 0 | 0 | 12 | 14 | 5 | 10 | 1 | 6 |
| 11 | 0 | 0 | 0 | 0 | 13 | 16 | 5 | 11 | 1 | 6 |
| 12 | 0 | 0 | 0 | 0 | 14 | 18 | 6 | 12 | 1 | 7 |
| 13 | 0 | 0 | 0 | 0 | 15 | 20 | 6 | 13 | 1 | 7 |
| 14 | 0 | 0 | 0 | 0 | 16 | 22 | 7 | 14 | 1 | 8 |
| 15 | 0 | 0 | 0 | 1 | 17 | 24 | 7 | 15 | 1 | 8 |
| 16 | 1 | 0 | 1 | 1 | 18 | 26 | 8 | 16 | 1 | 9 |
| 17 | 2 | 0 | 2 | 1 | 19 | 28 | 8 | 17 | 2 | 9 |
| 18/00-18/09 | 2 | 1 | 3 | 2 | 20 | 30 | 9 | 20 | 2 | 10 |
| 18/10-18/19 | 2 | 1 | 3 | 2 | 22 | 30 | 10 | 30 | 2 | 10 |
| 18/20-18/29 | 3 | 1 | 3 | 2 | 24 | 35 | 12 | 40 | 3 | 15 |
| 18/30-18/39 | 3 | 1 | 3 | 2 | 26 | 40 | 15 | 50 | 3 | 15 |
| 18/40-18/49 | 3 | 1 | 3 | 2 | 28 | 45 | 20 | 60 | 4 | 20 |
| 18/50-18/59 | 3 | 1 | 4 | 3 | 30 | 50 | 25 | 70 | 4 | 25 |
| 18/60-18/69 | 3 | 1 | 4 | 3 | 30 | 55 | 30 | 80 | 5 | 30 |
| 18/70-18/79 | 4 | 2 | 4 | 3 | 30 | 60 | 35 | 90 | 5 | 35 |
| 18/80-18/89 | 5 | 3 | 4 | 4 | 30 | 65 | 40 | 100 | 6 | 40 |
| 18/90-18/99 | 5 | 4 | 5 | 5 | 30 | 70 | 45 | 110 | 6 | 45 |
| 18/100-18/109 | 6 | 5 | 6 | 6 | 30 | 80 | 50 | 120 | 7 | 50 |
| 18/110-18/119 | 7 | 6 | 7 | 7 | 30 | 80 | 55 | 130 | 7 | 60 |
| 18/120-18/129 | 8 | 7 | 8 | 8 | 30 | 80 | 60 | 140 | 8 | 70 |
| 18/130-18/139 | 9 | 8 | 9 | 9 | 30 | 80 | 65 | 150 | 8 | 80 |
| 18/140-18/149 | 10 | 9 | 9 | 9 | 30 | 80 | 70 | 160 | 8 | 90 |
| 18/150-18/159 | 11 | 10 | 10 | 10 | 30 | 90 | 75 | 170 | 9 | 100 |
| 18/160-18/169 | 12 | 11 | 11 | 11 | 30 | 90 | 80 | 180 | 9 | 100 |
| 18/170-18/179 | 13 | 12 | 12 | 12 | 30 | 90 | 85 | 190 | 9 | 100 |
| 18/180-18/189 | 14 | 13 | 13 | 13 | 30 | 90 | 90 | 200 | 10 | 100 |
| 18/190-18/199 | 15 | 14 | 14 | 14 | 30 | 90 | 95 | 210 | 10 | 100 |
| 18/200-18/209 | 16 | 15 | 15 | 15 | 30 | 100 | 100 | 220 | 11 | 100 |
| 18/210-18/219 | 18 | 15 | 15 | 15 | 30 | 100 | 100 | 230 | 11 | 100 |
| 18/220+ | 20 | 15 | 15 | 15 | 30 | 100 | 100 | 240 | 11 | 100 |

| Stat | `con_fix` | `con_mhp` | `dex_dis` | `int_dis` | `int_dev` | `wis_sav` | `mag_study` | `mag_mana` | `mag_fail` | `mag_stat` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 3 | 0 | -250 | 0 | 0 | 0 | 0 | 0 | 0 | 99 | -5 |
| 4 | 0 | -150 | 0 | 0 | 0 | 0 | 0 | 10 | 99 | -4 |
| 5 | 0 | -100 | 0 | 0 | 0 | 0 | 10 | 20 | 99 | -3 |
| 6 | 0 | -75 | 0 | 0 | 0 | 0 | 20 | 30 | 99 | -3 |
| 7 | 0 | -50 | 0 | 0 | 0 | 0 | 30 | 40 | 99 | -2 |
| 8 | 0 | -25 | 1 | 1 | 1 | 1 | 40 | 50 | 50 | -1 |
| 9 | 0 | -10 | 1 | 1 | 1 | 1 | 50 | 60 | 30 | 0 |
| 10 | 0 | -5 | 1 | 1 | 1 | 1 | 60 | 70 | 20 | 0 |
| 11 | 0 | 0 | 1 | 1 | 1 | 1 | 70 | 80 | 15 | 0 |
| 12 | 0 | 5 | 1 | 1 | 1 | 1 | 80 | 90 | 12 | 0 |
| 13 | 0 | 10 | 1 | 1 | 1 | 1 | 85 | 100 | 11 | 0 |
| 14 | 1 | 25 | 1 | 1 | 1 | 1 | 90 | 110 | 10 | 1 |
| 15 | 1 | 50 | 2 | 2 | 2 | 2 | 95 | 120 | 9 | 2 |
| 16 | 1 | 75 | 2 | 2 | 2 | 2 | 100 | 130 | 8 | 3 |
| 17 | 1 | 100 | 2 | 2 | 2 | 2 | 105 | 140 | 7 | 4 |
| 18/00-18/09 | 2 | 150 | 3 | 3 | 3 | 3 | 110 | 150 | 6 | 5 |
| 18/10-18/19 | 2 | 175 | 3 | 3 | 3 | 3 | 115 | 160 | 6 | 6 |
| 18/20-18/29 | 2 | 200 | 3 | 3 | 3 | 3 | 120 | 170 | 5 | 7 |
| 18/30-18/39 | 2 | 225 | 4 | 4 | 3 | 3 | 130 | 180 | 5 | 8 |
| 18/40-18/49 | 2 | 250 | 4 | 4 | 3 | 3 | 140 | 190 | 5 | 9 |
| 18/50-18/59 | 3 | 275 | 5 | 5 | 4 | 4 | 150 | 200 | 4 | 10 |
| 18/60-18/69 | 3 | 300 | 6 | 6 | 4 | 4 | 160 | 225 | 4 | 11 |
| 18/70-18/79 | 3 | 350 | 7 | 7 | 5 | 5 | 170 | 250 | 4 | 12 |
| 18/80-18/89 | 3 | 400 | 8 | 8 | 5 | 5 | 180 | 300 | 4 | 15 |
| 18/90-18/99 | 3 | 450 | 9 | 9 | 6 | 6 | 190 | 350 | 3 | 18 |
| 18/100-18/109 | 4 | 500 | 10 | 10 | 6 | 7 | 200 | 400 | 3 | 21 |
| 18/110-18/119 | 4 | 550 | 10 | 10 | 7 | 8 | 210 | 450 | 2 | 24 |
| 18/120-18/129 | 5 | 600 | 11 | 11 | 7 | 9 | 220 | 500 | 2 | 27 |
| 18/130-18/139 | 6 | 650 | 12 | 12 | 8 | 10 | 230 | 550 | 2 | 30 |
| 18/140-18/149 | 6 | 700 | 13 | 13 | 8 | 11 | 240 | 600 | 2 | 33 |
| 18/150-18/159 | 7 | 750 | 14 | 14 | 9 | 12 | 250 | 650 | 1 | 36 |
| 18/160-18/169 | 7 | 800 | 15 | 15 | 9 | 13 | 250 | 700 | 1 | 39 |
| 18/170-18/179 | 8 | 900 | 16 | 16 | 10 | 14 | 250 | 750 | 1 | 42 |
| 18/180-18/189 | 8 | 1000 | 17 | 17 | 10 | 15 | 250 | 800 | 1 | 45 |
| 18/190-18/199 | 8 | 1100 | 18 | 18 | 11 | 16 | 250 | 800 | 1 | 48 |
| 18/200-18/209 | 9 | 1250 | 19 | 19 | 11 | 17 | 250 | 800 | 0 | 51 |
| 18/210-18/219 | 9 | 1250 | 19 | 19 | 12 | 18 | 250 | 800 | 0 | 54 |
| 18/220+ | 9 | 1250 | 19 | 19 | 13 | 19 | 250 | 800 | 0 | 57 |

### Blows table

`blows_table[P][D]` in `src/player-calcs.c`: energy per blow (100 = one blow per turn). Rows P = STR index (`adj_str_blow * mult / weight`), columns D = DEX index (`adj_dex_blow`). Blows per round = `min(100 / energy, max blows)`.

| P \ D | D=0 | D=1 | D=2 | D=3 | D=4 | D=5 | D=6 | D=7 | D=8 | D=9 | D=10 | D=11 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| P=0 | 100 | 100 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 25 | 23 |
| P=1 | 100 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 25 | 23 | 21 |
| P=2 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 26 | 23 | 21 | 20 |
| P=3 | 85 | 75 | 60 | 50 | 42 | 36 | 32 | 28 | 25 | 22 | 20 | 19 |
| P=4 | 75 | 60 | 50 | 42 | 36 | 33 | 28 | 25 | 23 | 21 | 19 | 18 |
| P=5 | 60 | 50 | 42 | 36 | 33 | 30 | 27 | 24 | 22 | 21 | 19 | 17 |
| P=6 | 50 | 42 | 36 | 33 | 30 | 27 | 25 | 23 | 21 | 20 | 18 | 17 |
| P=7 | 42 | 36 | 33 | 30 | 28 | 26 | 24 | 22 | 20 | 19 | 18 | 17 |
| P=8 | 36 | 33 | 30 | 28 | 26 | 24 | 22 | 21 | 20 | 19 | 17 | 16 |
| P=9 | 35 | 32 | 29 | 26 | 24 | 22 | 21 | 20 | 19 | 18 | 17 | 16 |
| P=10 | 34 | 30 | 27 | 25 | 23 | 22 | 21 | 20 | 19 | 18 | 17 | 16 |
| P=11 | 33 | 29 | 26 | 24 | 22 | 21 | 20 | 19 | 18 | 17 | 16 | 15 |

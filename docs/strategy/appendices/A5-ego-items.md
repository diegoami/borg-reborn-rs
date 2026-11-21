# Appendix A5. Ego items

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/ego_item.txt`, `lib/gamedata/object_property.txt`, `lib/gamedata/slay.txt`, `lib/gamedata/brand.txt`, `src/obj-make.c` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

All 107 ego-item types, grouped by the kind of item they apply to: applicable item types, allocation, value, combat bonuses, modifiers, flags, slays/brands and random extra powers. Use with [Chapter 6](../06-equipment.md).

## Contents

- [Legend](#legend)
- [Melee weapons (28)](#melee-weapons-28)
- [Diggers (2)](#diggers-2)
- [Launchers (8)](#launchers-8)
- [Ammunition (14)](#ammunition-14)
- [Body armour (14)](#body-armour-14)
- [Shields (8)](#shields-8)
- [Cloaks (4)](#cloaks-4)
- [Gloves (6)](#gloves-6)
- [Boots (6)](#boots-6)
- [Helms and crowns (12)](#helms-and-crowns-12)
- [Light sources (4)](#light-sources-4)
- [Other (1)](#other-1)

## Legend

- **Alloc**: `min-max depth (xcommonness)`. An ego can still be rolled out of depth: below its minimum depth with chance 1 in `max(2, (min - level) / 3)` (`ego_find_random()` in `src/obj-make.c`).
- **Cost/rating**: `info:` value added to the item and level-feeling rating.
- **Combat**: extra `+hit : +dam : +AC`, each a random value; `d6` = 1d6, `M8` = level-scaled bonus 0..8 (see the `object.txt` header). **Min** gives `min-combat`/`min-values` floors (255 = no minimum).
- **Modifier units** (`calc_bonuses()` / `calc_blows()` in `src/player-calcs.c`): `BLOWS` = whole extra blows; `SHOTS` = tenths of a shot per turn (`SHOTS +10` = one extra shot); `MIGHT` adds to the launcher multiplier; `SPEED` in speed points (+10 = twice as fast); `INFRA` in 10-foot steps; `LIGHT` = light radius; `TUNNEL`, `STEALTH`, `SEARCH` and the five stats add directly. `M8` in a value = a level-dependent bonus of 0..8 (see the `object.txt` header).
- **Random power** (`ego_apply_magic()` in `src/obj-make.c`): `RAND_SUSTAIN` one random sustain (SUST_STR, SUST_INT, SUST_WIS, SUST_DEX, SUST_CON); `RAND_POWER` one random protection or ability (PROT_FEAR, PROT_BLIND, PROT_CONF, PROT_STUN, SLOW_DIGEST, FEATHER, REGEN, TELEPATHY, SEE_INVIS, FREE_ACT, HOLD_LIFE, TRAP_IMMUNE); `RAND_BASE_RES` one random base resist (acid, lightning, fire, cold); `RAND_HI_RES` one random high resist (poison, light, dark, sound, shards, nexus, nether, chaos, disenchantment); `RAND_RES_POWER` one of: random power (1/3) or random base resist (2/3). The pick avoids properties the item already has.

## Melee weapons (28)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| (Holy Avenger) | any sword, polearm, hafted | 15-127 (x10) | 20000 / 30 | d6:d6:d4 | WIS d4 | SEE\_INVIS, BLESSED, PROT\_FEAR | slay evil creatures x2, slay undead x3, slay demons x3 | RAND\_SUSTAIN | WIS\[0\] |
| (Defender) | any sword, polearm, hafted | 10-80 (x10) | 15000 / 25 | d4:d4:d8 | STEALTH d4, resist acid, resist lightning, resist fire, resist cold | FREE\_ACT, SEE\_INVIS, FEATHER, REGEN, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_SUSTAIN | STEALTH\[0\] |
| (Blessed) | any sword, polearm, hafted | 1-60 (x20) | 5000 / 20 | - | WIS d3 | BLESSED | - | RAND\_POWER | WIS\[0\] |
| of Gondolin | any sword, polearm, hafted | 20-127 (x10) | 25000 / 30 | d7:d7:0 | LIGHT +1, resist dark | SEE\_INVIS, FREE\_ACT, IGNORE\_ACID, IGNORE\_FIRE | slay demons x3, slay orcs x3, slay trolls x3, slay dragons x3 | RAND\_POWER | LIGHT\[1\] |
| of Westernesse | any sword, polearm, hafted | 10-70 (x10) | 20000 / 20 | d5:d5:0 | STR d2, DEX d2, CON d2 | FREE\_ACT, SEE\_INVIS | slay orcs x3, slay trolls x3, slay giants x3 | - | STR\[1\] DEX\[1\] CON\[1\] |
| of Extra Attacks | any sword, polearm, hafted | 10-127 (x10) | 10000 / 20 | - | BLOWS d2 | - | - | - | BLOWS\[0\] |
| of Acid | any sword, polearm, hafted | 1-60 (x30) | 5000 / 20 | - | resist acid | IGNORE\_ACID | brand acid x3 | - | - |
| of Lightning | any sword, polearm, hafted | 1-60 (x30) | 4500 / 20 | - | resist lightning | IGNORE\_ELEC | brand lightning x3 | - | - |
| of Flame | any sword, polearm, hafted | 1-50 (x40) | 3500 / 15 | - | resist fire | IGNORE\_FIRE | brand fire x3 | - | - |
| of Frost | any sword, polearm, hafted | 1-50 (x40) | 3000 / 15 | - | resist cold | IGNORE\_COLD | brand cold x3 | - | - |
| of Venom | any sword, polearm, hafted | 1-40 (x60) | 4000 / 15 | - | - | - | brand poison x3 | - | - |
| of Slay Animal | any sword, polearm, hafted | 1-30 (x100) | 3000 / 18 | - | - | - | slay animals x2 | - | - |
| of Slay Evil | any sword, polearm, hafted | 1-40 (x60) | 3000 / 18 | - | - | - | slay evil creatures x2 | - | - |
| of Slay Undead | any sword, polearm; hafted: Whip, Ball-and-Chain, Morning Star, Flail, Two-Handed Great Flail, Mace, Lead-Filled Mace, Quarterstaff, War Hammer, Maul, Great Hammer | 1-30 (x100) | 3500 / 18 | - | - | - | slay undead x3 | - | - |
| of Slay Demon | any sword, polearm, hafted | 1-30 (x100) | 3500 / 14 | - | - | - | slay demons x3 | - | - |
| of Slay Orc | any sword, polearm, hafted | 1-20 (x150) | 2500 / 10 | - | - | - | slay orcs x3 | - | - |
| of Slay Troll | any sword, polearm, hafted | 1-25 (x150) | 2500 / 10 | - | - | - | slay trolls x3 | - | - |
| of Slay Giant | any sword, polearm, hafted | 1-30 (x60) | 2500 / 14 | - | - | - | slay giants x3 | - | - |
| of Slay Dragon | any sword, polearm, hafted | 1-30 (x100) | 3500 / 18 | - | - | - | slay dragons x3 | - | - |
| of \*Slay Animal\* | any sword, polearm, hafted | 1-45 (x40) | 6000 / 20 | - | INT d2 | - | slay animals x2 | RAND\_RES\_POWER | INT\[0\] |
| of \*Slay Evil\* | any sword, polearm, hafted | 10-127 (x15) | 5000 / 20 | - | WIS d2 | BLESSED | slay evil creatures x2 | RAND\_RES\_POWER | WIS\[0\] |
| of \*Slay Undead\* | any sword, polearm; hafted: Whip, Ball-and-Chain, Morning Star, Flail, Two-Handed Great Flail, Mace, Lead-Filled Mace, Quarterstaff, War Hammer, Maul, Great Hammer | 10-60 (x30) | 8000 / 24 | - | WIS d2 | - | slay undead x5 | RAND\_RES\_POWER | WIS\[0\] |
| of \*Slay Demon\* | any sword, polearm, hafted | 10-60 (x30) | 8000 / 16 | - | INT d2 | - | slay demons x5 | RAND\_RES\_POWER | INT\[0\] |
| of \*Slay Orc\* | any sword, polearm, hafted | 1-40 (x60) | 4000 / 14 | - | DEX d2 | - | slay orcs x3 | RAND\_RES\_POWER | DEX\[0\] |
| of \*Slay Troll\* | any sword, polearm, hafted | 1-45 (x60) | 4000 / 14 | - | STR d2 | - | slay trolls x3 | RAND\_RES\_POWER | STR\[0\] |
| of \*Slay Giant\* | any sword, polearm, hafted | 1-45 (x20) | 4000 / 16 | - | STR d2 | - | slay giants x3 | RAND\_RES\_POWER | STR\[0\] |
| of \*Slay Dragon\* | any sword, polearm, hafted | 10-60 (x30) | 8000 / 24 | - | CON d2 | - | slay dragons x5 | RAND\_RES\_POWER | CON\[0\] |
| of Morgul | any sword, polearm, hafted | 10-80 (x10) | 1 / 0 | - | - | SEE\_INVIS, AGGRAVATE, HOLD\_LIFE, DRAIN\_EXP | slay undead x3, brand poison x3 | - | - |

## Diggers (2)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Digging | any digger | 1-40 (x100) | 500 / 4 | - | TUNNEL d4 | - | brand acid x3 | - | TUNNEL\[1\] |
| of Earthquakes | any digger | 10-127 (x10) | 3000 / 8 | d10:d10:0 | STR d4, TUNNEL d6 | IMPACT | brand acid x3 | - | STR\[1\] TUNNEL\[1\] |

## Launchers (8)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Accuracy | any bow | 5-70 (x100) | 1000 / 10 | d15:d5:0 | - | - | - | - | combat 15:255:0 |
| of Power | any bow | 5-70 (x100) | 1000 / 10 | d5:d15:0 | - | - | - | - | combat 255:15:0 |
| of Lothlórien | bow: Short Bow, Long Bow | 20-127 (x5) | 20000 / 30 | d10:d10:0 | DEX 1+d2M2, MIGHT d2 | FREE\_ACT, IGNORE\_ACID, IGNORE\_FIRE | - | RAND\_POWER | DEX\[2\] MIGHT\[1\] |
| of the Haradrim | bow: Light Crossbow | 20-127 (x5) | 20000 / 30 | d5:d5:0 | MIGHT +1, SHOTS 2d3 | IGNORE\_ACID, IGNORE\_FIRE | - | - | MIGHT\[1\] SHOTS\[2\] |
| of Extra Might | any bow | 15-100 (x20) | 10000 / 20 | d5:d10:0 | MIGHT +1 | - | - | - | MIGHT\[0\] |
| of Extra Shots | any bow | 15-100 (x10) | 10000 / 20 | d10:d5:0 | SHOTS 2d3 | - | - | - | SHOTS\[2\] |
| of Buckland | bow: Sling | 20-127 (x5) | 20000 / 25 | d8:d8:0 | DEX 1+d2M2, SHOTS 2d5, MIGHT d2 | IGNORE\_ACID, IGNORE\_FIRE | - | - | DEX\[2\] SHOTS\[2\] MIGHT\[1\] |
| of the Nazgûl | any bow | 10-80 (x10) | 0 / 0 | d10:d10:0 | - | DRAIN\_EXP, SEE\_INVIS | - | - | - |

## Ammunition (14)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Slay Animal | any shot, arrow, bolt | 1-40 (x80) | 20 / 10 | - | - | - | slay animals x2 | - | - |
| of Slay Evil | any shot, arrow, bolt | 10-100 (x10) | 20 / 10 | - | - | - | slay evil creatures x2 | - | - |
| of Slay Undead | any shot, arrow, bolt | 5-100 (x15) | 25 / 10 | - | - | - | slay undead x3 | - | - |
| of Slay Demon | any shot, arrow, bolt | 5-100 (x15) | 25 / 10 | - | - | - | slay demons x3 | - | - |
| of Acid | any shot, arrow, bolt | 10-100 (x5) | 50 / 10 | - | - | IGNORE\_ACID | brand acid x3 | - | - |
| of Lightning | any shot, arrow, bolt | 10-100 (x5) | 45 / 10 | - | - | IGNORE\_ELEC | brand lightning x3 | - | - |
| of Slay Giant | any shot, arrow, bolt | 5-60 (x5) | 25 / 10 | - | - | - | slay giants x3 | - | - |
| of Slay Dragon | any shot, arrow, bolt | 10-100 (x10) | 40 / 10 | - | - | - | slay dragons x3 | - | - |
| of Holy Might | shot: Mithril Shot; arrow: Seeker Arrow, Mithril Arrow; bolt: Seeker Bolt, Mithril Bolt | 30-127 (x2) | 60 / 20 | d10:d10:0 | - | IGNORE\_FIRE, IGNORE\_ACID | slay evil creatures x2, slay demons x3, slay undead x3 | - | - |
| of Venom | any shot, arrow, bolt | 10-60 (x20) | 40 / 10 | - | - | - | brand poison x3 | - | - |
| of Flame | any shot, arrow, bolt | 10-100 (x10) | 35 / 10 | - | - | IGNORE\_FIRE | brand fire x3 | - | - |
| of Frost | any shot, arrow, bolt | 10-100 (x10) | 30 / 10 | - | - | IGNORE\_COLD | brand cold x3 | - | - |
| of Wounding | any shot, arrow, bolt | 15-127 (x15) | 20 / 5 | 3+d5:3+d5:0 | - | - | - | - | - |
| of Backbiting | any shot, arrow, bolt | 10-80 (x0) | 0 / 0 | -26+d25:-26+d25:0 | - | - | - | - | combat 255:255:0 |

## Body armour (14)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Resist Acid | any soft armor; hard armor: Metal Scale Mail, Chain Mail, Augmented Chain Mail, Bar Chain Mail, Metal Brigandine Armour, Partial Plate Armour | 1-30 (x100) | 1000 / 16 | - | resist acid | IGNORE\_ACID | - | - | combat 255:0:0 |
| of Resist Lightning | any soft armor; hard armor: Metal Scale Mail, Chain Mail, Augmented Chain Mail, Bar Chain Mail, Metal Brigandine Armour, Partial Plate Armour | 1-30 (x100) | 400 / 10 | - | resist lightning | IGNORE\_ELEC | - | - | combat 255:0:0 |
| of Resist Fire | any soft armor; hard armor: Metal Scale Mail, Chain Mail, Augmented Chain Mail, Bar Chain Mail, Metal Brigandine Armour, Partial Plate Armour | 1-30 (x100) | 800 / 14 | - | resist fire | IGNORE\_FIRE | - | - | combat 255:0:0 |
| of Resist Cold | any soft armor; hard armor: Metal Scale Mail, Chain Mail, Augmented Chain Mail, Bar Chain Mail, Metal Brigandine Armour, Partial Plate Armour | 1-30 (x100) | 600 / 12 | - | resist cold | IGNORE\_COLD | - | - | combat 255:0:0 |
| of Resistance | any soft armor, hard armor | 10-100 (x50) | 12500 / 20 | 0:0:d10 | resist acid, resist lightning, resist fire, resist cold | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | - | combat 255:0:0 |
| of Elvenkind | any soft armor, hard armor | 30-127 (x10) | 15000 / 25 | 0:0:d10 | STEALTH d2, resist acid, resist lightning, resist fire, resist cold | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_HI\_RES | combat 255:0:0; STEALTH\[0\] |
| of Permanence | soft armor: Robe | 30-127 (x10) | 30000 / 30 | 0:0:d20 | resist acid, resist lightning, resist fire, resist cold | SUST\_STR, SUST\_DEX, SUST\_CON, SUST\_INT, SUST\_WIS, HOLD\_LIFE, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_HI\_RES | - |
| (Dwarven) | any hard armor | 20-127 (x20) | 5000 / 18 | 0:0:d15 | STR d2, CON d2, INFRA d2M3 | IGNORE\_ACID, IGNORE\_FIRE, FREE\_ACT | - | - | combat 255:0:0; STR\[1\] CON\[1\] INFRA\[1\] |
| of Craftsmanship | any dragon armor | 20-95 (x60) | 500 / 4 | 0:0:1d6M4 | - | - | - | - | combat 0:0:4 |
| of Stealth | any dragon armor | 20-100 (x10) | 500 / 16 | - | STEALTH d2 | - | - | - | combat 255:0:0; STEALTH\[0\] |
| of Resistance | dragon armor: Shining Dragon Scale Mail, Law Dragon Scale Mail, Gold Dragon Scale Mail, Chaos Dragon Scale Mail, Balance Dragon Scale Mail | 40-100 (x5) | 12500 / 10 | 0:0:d10 | resist acid, resist lightning, resist fire, resist cold | - | - | - | combat 255:0:0 |
| of Elvenkind | dragon armor: Shining Dragon Scale Mail, Law Dragon Scale Mail, Gold Dragon Scale Mail, Chaos Dragon Scale Mail, Balance Dragon Scale Mail | 50-127 (x3) | 15000 / 25 | 0:0:d10 | STEALTH d2, resist acid, resist lightning, resist fire, resist cold | - | - | RAND\_HI\_RES | STEALTH\[0\] |
| (Dwarven) | any dragon armor | 40-127 (x5) | 5000 / 18 | 0:0:d15 | STR d2, CON d2, INFRA d2M3 | FREE\_ACT | - | - | combat 255:0:5; STR\[1\] CON\[1\] INFRA\[1\] |
| of Speed | any dragon armor | 40-127 (x2) | 100000 / 25 | - | SPEED d5 | - | - | - | combat 255:0:0; SPEED\[0\] |

## Shields (8)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| (Dwarven) | shield: Small Metal Shield | 40-127 (x20) | 5000 / 20 | 1+M3:1+M3:d5 | STR d2, CON d2 | IGNORE\_ACID, IGNORE\_FIRE, FREE\_ACT | - | - | STR\[1\] CON\[1\] |
| of Resist Acid | any shield | 1-30 (x100) | 1000 / 16 | - | resist acid | IGNORE\_ACID | - | - | - |
| of Resist Lightning | any shield | 1-30 (x100) | 400 / 10 | - | resist lightning | IGNORE\_ELEC | - | - | - |
| of Resist Fire | any shield | 1-30 (x100) | 800 / 14 | - | resist fire | IGNORE\_FIRE | - | - | - |
| of Resist Cold | any shield | 1-30 (x100) | 600 / 12 | - | resist cold | IGNORE\_COLD | - | - | - |
| of Resistance | any shield | 10-100 (x50) | 12500 / 20 | 0:0:d10 | resist acid, resist lightning, resist fire, resist cold | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | - | - |
| of Elvenkind | any shield | 30-127 (x10) | 18000 / 25 | 0:0:d10 | resist fire, resist cold, resist acid, resist lightning, STEALTH d2 | IGNORE\_ACID, IGNORE\_FIRE, IGNORE\_COLD, IGNORE\_ELEC | - | RAND\_HI\_RES | STEALTH\[0\] |
| of Preservation | any shield | 40-127 (x5) | 24000 / 25 | 0:0:d20 | resist disenchantment | SUST\_STR, SUST\_CON, SUST\_DEX, HOLD\_LIFE, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_HI\_RES | - |

## Cloaks (4)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Protection | any cloak | 1-40 (x30) | 1500 / 10 | 0:0:d10 | resist shards | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | - | - |
| of Stealth | any cloak | 1-70 (x60) | 500 / 10 | - | STEALTH d3 | - | - | - | STEALTH\[0\] |
| of Aman | any cloak | 20-127 (x10) | 4000 / 20 | 0:0:d20 | STEALTH d3 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_HI\_RES | STEALTH\[0\] |
| of the Magi | any cloak | 30-127 (x10) | 2000 / 15 | 0:0:d4 | INT d2, STEALTH d3 | SUST\_INT, IGNORE\_ACID | - | RAND\_POWER | INT\[1\] STEALTH\[1\] |

## Gloves (6)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Free Action | any gloves | 1-60 (x100) | 1000 / 11 | - | - | FREE\_ACT | - | - | - |
| of Slaying | any gloves | 10-75 (x20) | 1500 / 17 | d5:d5:0 | - | - | - | - | - |
| of Agility | any gloves | 20-50 (x10) | 1000 / 14 | - | DEX d5 | - | - | - | DEX\[0\] |
| of Power | any gloves | 30-127 (x5) | 2500 / 22 | d5:d5:0 | STR d5 | - | - | - | STR\[0\] |
| of Thievery | gloves: Set of Leather Gloves, Set of Alchemist's Gloves | 40-127 (x5) | 5000 / 22 | d8:d3:0 | DEX 1+M4, SEARCH 1+d2M3 | FEATHER, FREE\_ACT | - | - | DEX\[1\] SEARCH\[2\] |
| of Combat | gloves: Set of Gauntlets, Set of Mithril Gauntlets, Set of Caestus | 50-127 (x5) | 7000 / 22 | d5:d8:0 | STR d2, CON d2 | AGGRAVATE | - | - | STR\[1\] CON\[1\] |

## Boots (6)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Slow Descent | any boots | 1-15 (x100) | 250 / 7 | - | - | FEATHER | - | - | - |
| of Stealth | boots: Pair of Leather Sandals, Pair of Leather Boots, Pair of Ethereal Slippers | 15-100 (x20) | 500 / 16 | - | STEALTH d3 | - | - | - | STEALTH\[0\] |
| of Free Action | any boots | 1-40 (x20) | 1000 / 15 | - | - | FREE\_ACT | - | - | - |
| of Speed | any boots | 20-127 (x3) | 100000 / 25 | - | SPEED 2+M8 | - | - | - | SPEED\[0\] |
| of Stability | any boots | 15-100 (x10) | 5000 / 20 | - | resist nexus | FEATHER | - | - | - |
| of Elvenkind | boots: Pair of Leather Boots, Pair of Iron Shod Boots | 60-127 (x3) | 200000 / 30 | - | STEALTH d4, SPEED 4+M5 | FEATHER, IGNORE\_ACID, IGNORE\_FIRE | - | - | STEALTH\[1\] SPEED\[1\] |

## Helms and crowns (12)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Intelligence | any helm | 1-40 (x100) | 500 / 13 | - | INT d2 | SUST\_INT | - | - | INT\[0\] |
| of Wisdom | any helm | 1-40 (x100) | 500 / 13 | - | WIS d2 | SUST\_WIS | - | - | WIS\[0\] |
| of the Magi | any crown | 20-127 (x20) | 7500 / 15 | - | INT d3, resist acid, resist lightning, resist fire, resist cold | SUST\_INT, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_POWER | INT\[0\] |
| of Might | any crown | 40-127 (x10) | 7500 / 19 | - | STR d3, DEX d3, CON d3 | SUST\_STR, SUST\_DEX, SUST\_CON, FREE\_ACT, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | - | STR\[1\] DEX\[1\] CON\[1\] |
| of Lordliness | any crown | 40-127 (x20) | 7500 / 17 | - | WIS d3 | SUST\_WIS, PROT\_FEAR, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - | RAND\_POWER | WIS\[1\] |
| of Seeing | any helm | 10-60 (x50) | 2000 / 8 | - | SEARCH d5 | PROT\_BLIND, SEE\_INVIS | - | - | SEARCH\[0\] |
| of Infravision | any helm | 1-20 (x100) | 500 / 11 | - | INFRA d5 | - | - | - | INFRA\[0\] |
| of Light | any crown | 5-30 (x50) | 1000 / 6 | - | LIGHT +1, resist light | - | - | - | LIGHT\[1\] |
| of Telepathy | any helm | 40-127 (x10) | 50000 / 20 | - | - | TELEPATHY | - | - | - |
| of Regeneration | any helm | 1-60 (x20) | 1500 / 10 | - | - | REGEN | - | - | - |
| of Serenity | any crown | 20-127 (x15) | 4000 / 20 | - | resist sound | PROT\_CONF, PROT\_FEAR, PROT\_STUN | - | - | - |
| of Night and Day | any crown | 10-80 (x15) | 4000 / 18 | - | LIGHT +1, resist light, resist dark | SEE\_INVIS, PROT\_BLIND, IGNORE\_ACID | - | - | LIGHT\[1\] |

## Light sources (4)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Shadows | light: Lantern | 10-100 (x40) | 1000 / 10 | - | STEALTH d2, resist light, LIGHT -4 | - | - | - | STEALTH\[0\] LIGHT\[-4\] |
| of Brightness | light: Wooden Torch, Lantern | 1-30 (x50) | 2000 / 10 | - | LIGHT +1 | - | - | - | LIGHT\[1\] |
| (Everburning) | light: Lantern | 1-30 (x100) | 2500 / 10 | - | - | NO\_FUEL, removes TAKES\_FUEL | - | - | - |
| of True Sight | light: Lantern | 20-127 (x10) | 6000 / 10 | - | - | PROT\_BLIND, SEE\_INVIS | - | - | - |

## Other (1)

| Ego | Applies to | Alloc | Cost / rating | Combat | Mods | Flags | Slays/brands | Random power | Min |
|---|---|---|---|---|---|---|---|---|---|
| of Fury | nothing: no `type:`/`item:` lines, so it is never generated | 50-127 (x2) | 20000 / 30 | d10:d10:0 | STR 1+d2M2, BLOWS d2 | AGGRAVATE, PROT\_FEAR, IGNORE\_ACID, IGNORE\_FIRE | - | - | STR\[2\] BLOWS\[1\] |

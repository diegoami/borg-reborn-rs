# Appendix A6. Object kinds

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/object.txt`, `lib/gamedata/object_base.txt`, `lib/gamedata/class.txt`, `lib/gamedata/projection.txt`, `lib/gamedata/player_timed.txt`, `src/list-effects.h` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Every base object kind (371 in `object.txt`, not counting the 4 `type:none` display placeholders, plus the 20 spell books defined in `class.txt`), grouped by item type: allocation, cost and base statistics; for consumables and devices the effect in words, charges and recharge time. See [Chapter 7](../07-consumables-and-devices.md) and [Chapter 6](../06-equipment.md).

## Contents

- [Legend](#legend)
- [Sword (14)](#sword-14)
- [Polearm (16)](#polearm-16)
- [Hafted (14)](#hafted-14)
- [Digger (3)](#digger-3)
- [Bow (5)](#bow-5)
- [Shot (3)](#shot-3)
- [Arrow (3)](#arrow-3)
- [Bolt (3)](#bolt-3)
- [Soft Armor (5)](#soft-armor-5)
- [Hard Armor (12)](#hard-armor-12)
- [Dragon Armor (12)](#dragon-armor-12)
- [Shield (6)](#shield-6)
- [Cloak (4)](#cloak-4)
- [Gloves (5)](#gloves-5)
- [Boots (6)](#boots-6)
- [Helm (4)](#helm-4)
- [Crown (4)](#crown-4)
- [Light (2)](#light-2)
- [Ring (30)](#ring-30)
- [Amulet (17)](#amulet-17)
- [Potion (41)](#potion-41)
- [Scroll (39)](#scroll-39)
- [Flask (1)](#flask-1)
- [Food (14)](#food-14)
- [Mushroom (12)](#mushroom-12)
- [Wand (30)](#wand-30)
- [Staff (24)](#staff-24)
- [Rod (25)](#rod-25)
- [Chest (6)](#chest-6)
- [Gold (11)](#gold-11)
- [Spell books (20)](#spell-books-20)

## Legend

- **Lvl** is the object level (device skill difficulty and activation level). **Alloc** is `min-max depth (xcommonness)`; `not random` items are only created by stores, birth kits or special code.
- **Cost** is the base value in gold (store sell price before the black market multiplier, see [A9](A9-stores.md)).
- Weapons: **Dice** base damage; ammunition is multiplied by the launcher. Armour: **AC** base armour class, **Hit** to-hit modifier. Launchers: **Mult** multiplier.
- **Effect**: what using the item does, generated from the `effect:` lines. Dice notation: `20+1d20`; `plev` = player level; `m35` in a healing roll = at least 35% of HP lost; `M60` in a teleport = 60% of the level's larger dimension; `x food-value` = times `player:food-value` (100).
- **Charges** for wands/staffs (random value); **Recharge** for rods and activations, in player turns.
- **Pile**: `chance% : number` for being generated in a stack.
- **Modifier units** (`calc_bonuses()` / `calc_blows()` in `src/player-calcs.c`): `BLOWS` = whole extra blows; `SHOTS` = tenths of a shot per turn (`SHOTS +10` = one extra shot); `MIGHT` adds to the launcher multiplier; `SPEED` in speed points (+10 = twice as fast); `INFRA` in 10-foot steps; `LIGHT` = light radius; `TUNNEL`, `STEALTH`, `SEARCH` and the five stats add directly. `M8` in a value = a level-dependent bonus of 0..8 (see the `object.txt` header).

## Sword (14)

*base flags: HATES\_ACID, SHOW\_DICE* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Dagger | 0 | 0-100 (x20) | 1d4 | 1.2 lb | 30 | THROWING |
| Main Gauche | 3 | 3-100 (x20) | 1d5 | 3.0 lb | 45 | - |
| Cutlass | 5 | 5-100 (x20) | 1d8 | 11.0 lb | 120 | - |
| Rapier | 5 | 5-100 (x20) | 1d6 | 4.0 lb | 70 | - |
| Short Sword | 5 | 5-100 (x20) | 1d7 | 8.0 lb | 90 | - |
| Tulwar | 5 | 5-100 (x20) | 2d4 | 10.0 lb | 170 | - |
| Broad Sword | 10 | 10-100 (x20) | 2d5 | 15.0 lb | 300 | - |
| Long Sword | 10 | 10-100 (x20) | 2d5 | 13.0 lb | 300 | - |
| Scimitar | 10 | 10-100 (x20) | 4d2 | 15.0 lb | 300 | - |
| Bastard Sword | 15 | 15-100 (x20) | 3d4 | 14.0 lb | 400 | - |
| Katana | 20 | 20-100 (x20) | 3d5 | 12.0 lb | 600 | - |
| Zweihander | 30 | 30-100 (x20) | 3d6 | 20.0 lb | 700 | - |
| Executioner's Sword | 40 | 40-100 (x20) | 4d5 | 26.0 lb | 850 | - |
| Blade of Chaos | 70 | 70-100 (x10) | 6d5 | 18.0 lb | 4000 | resist chaos |

## Polearm (16)

*base flags: HATES\_ACID, HATES\_FIRE, SHOW\_DICE* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Spear | 5 | 5-100 (x20) | 1d6 | 5.0 lb | 36 | THROWING |
| Trident | 5 | 5-100 (x20) | 1d10 | 7.0 lb | 120 | - |
| Awl-Pike | 10 | 10-100 (x20) | 1d8 | 16.0 lb | 340 | - |
| Lance | 10 | 10-100 (x20) | 2d8 | 30.0 lb | 230 | - |
| Lucerne Hammer | 10 | 10-100 (x20) | 2d5 | 12.0 lb | 376 | - |
| Battle Axe | 15 | 15-100 (x20) | 2d8 | 17.0 lb | 334 | - |
| Beaked Axe | 15 | 15-100 (x20) | 2d6 | 18.0 lb | 408 | - |
| Broad Axe | 15 | 15-100 (x20) | 2d6 | 16.0 lb | 304 | - |
| Pike | 15 | 15-100 (x20) | 2d5 | 16.0 lb | 358 | - |
| Glaive | 20 | 20-100 (x20) | 2d6 | 19.0 lb | 363 | - |
| Halberd | 25 | 25-100 (x20) | 3d5 | 19.0 lb | 430 | - |
| Throwing Axe | 25 | 25-100 (x10) | 2d3 | 4.0 lb | 150 | THROWING |
| Great Axe | 40 | 40-100 (x20) | 4d4 | 23.0 lb | 500 | - |
| Lochaber Axe | 45 | 45-100 (x20) | 3d8 | 25.0 lb | 750 | - |
| Scythe | 45 | 45-100 (x20) | 5d3 | 25.0 lb | 800 | - |
| Scythe of Slicing | 60 | 60-100 (x10) | 8d4 | 25.0 lb | 3500 | - |

## Hafted (14)

*base flags: HATES\_ACID, HATES\_FIRE, SHOW\_DICE* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Whip | 3 | 3-100 (x20) | 1d3 | 3.0 lb | 30 | - |
| Great Hammer | 5 | 20-100 (x20) | 8d1 | 18.0 lb | 500 | - |
| Mace | 5 | 5-100 (x20) | 2d4 | 12.0 lb | 130 | - |
| Maul | 5 | 5-100 (x20) | 4d4 | 20.0 lb | 130 | - |
| War Hammer | 5 | 5-100 (x20) | 3d3 | 12.0 lb | 225 | - |
| Flail | 10 | 10-100 (x20) | 2d6 | 15.0 lb | 353 | - |
| Morning Star | 10 | 10-100 (x20) | 2d6 | 15.0 lb | 396 | - |
| Quarterstaff | 10 | 10-100 (x20) | 1d9 | 15.0 lb | 200 | - |
| Lead-Filled Mace | 15 | 15-100 (x20) | 4d3 | 18.0 lb | 502 | - |
| Mighty Hammer | 15 | not random | 3d9 | 20.0 lb | 1000 | INSTA\_ART, QUEST\_ART |
| Ball-and-Chain | 20 | 20-100 (x20) | 2d4 | 15.0 lb | 200 | - |
| Throwing Hammer | 40 | 40-100 (x10) | 2d4 | 5.5 lb | 250 | THROWING |
| Two-Handed Great Flail | 45 | 45-100 (x20) | 3d6 | 28.0 lb | 590 | - |
| Mace of Disruption | 80 | 80-100 (x10) | 5d8 | 40.0 lb | 4300 | slay undead x3 |

## Digger (3)

*base flags: SHOW\_DICE* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Shovel | 1 | 5-100 (x20) | 1d2 | 6.0 lb | 10 | DIG\_1 |
| Pick | 30 | 30-100 (x20) | 1d3 | 18.0 lb | 300 | DIG\_2 |
| Mattock | 50 | 50-100 (x20) | 1d8 | 25.0 lb | 700 | DIG\_3 |

## Bow (5)

*base flags: HATES\_ACID, HATES\_FIRE, SHOW\_MULT* (`object_base.txt`).

| Name | Lvl | Alloc | Mult | Weight | Cost | Notes |
|---|---:|---|---:|---:|---:|---|
| Sling | 1 | 1-100 (x20) | x2 | 0.5 lb | 5 | SHOOTS\_SHOTS |
| Short Bow | 3 | 3-100 (x20) | x2 | 3.0 lb | 50 | SHOOTS\_ARROWS |
| Long Bow | 10 | 10-100 (x20) | x3 | 4.0 lb | 120 | SHOOTS\_ARROWS |
| Light Crossbow | 15 | 15-100 (x20) | x3 | 11.0 lb | 140 | SHOOTS\_BOLTS |
| Heavy Crossbow | 30 | 30-100 (x20) | x4 | 20.0 lb | 300 | SHOOTS\_BOLTS |

## Shot (3)

*base flags: SHOW\_DICE; break chance 0%* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Rounded Pebble | 0 | 0-100 (x30) | 1d2 | 0.4 lb | 1 | THROWING |
| Iron Shot | 3 | 3-100 (x25) | 1d4 | 0.5 lb | 2 | THROWING |
| Mithril Shot | 40 | 40-100 (x20) | 2d4 (5, 5) | 0.2 lb | 20 | THROWING, IGNORE\_ACID, IGNORE\_FIRE |

## Arrow (3)

*base flags: HATES\_ACID, HATES\_FIRE, SHOW\_DICE; break chance 35%* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Arrow | 3 | 3-100 (x30) | 1d4 | 0.2 lb | 1 | - |
| Mithril Arrow | 55 | 50-100 (x20) | 3d4 | 0.1 lb | 25 | IGNORE\_ACID, IGNORE\_FIRE |
| Seeker Arrow | 55 | 55-100 (x25) | 4d4 | 0.2 lb | 20 | - |

## Bolt (3)

*base flags: HATES\_ACID, SHOW\_DICE; break chance 20%* (`object_base.txt`).

| Name | Lvl | Alloc | Dice | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---|
| Bolt | 3 | 3-100 (x30) | 1d5 | 0.3 lb | 2 | - |
| Mithril Bolt | 50 | 60-100 (x20) | 3d5 | 0.2 lb | 30 | IGNORE\_ACID |
| Seeker Bolt | 65 | 65-100 (x25) | 4d5 | 0.3 lb | 25 | - |

## Soft Armor (5)

*base flags: HATES\_ACID, HATES\_FIRE* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Robe | 1 | 1-100 (x20) | 2 | 0 | 2.0 lb | 4 | - |
| Soft Leather Armour | 3 | 3-100 (x20) | 8 | 0 | 8.0 lb | 20 | - |
| Hard Leather Armour | 5 | 10-100 (x20) | 16 | -1 | 12.0 lb | 150 | - |
| Studded Leather Armour | 10 | 5-100 (x20) | 12 | -1 | 10.0 lb | 100 | - |
| Leather Scale Mail | 15 | 15-100 (x20) | 20 | -1 | 14.0 lb | 370 | - |

## Hard Armor (12)

*base flags: HATES\_ACID* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Chain Mail | 25 | 20-100 (x20) | 32 | -2 | 22.0 lb | 750 | - |
| Metal Scale Mail | 25 | 25-100 (x20) | 38 | -2 | 25.0 lb | 550 | - |
| Augmented Chain Mail | 30 | 35-100 (x20) | 42 | -2 | 27.0 lb | 900 | - |
| Bar Chain Mail | 35 | 40-100 (x20) | 45 | -2 | 28.0 lb | 950 | - |
| Metal Brigandine Armour | 35 | 45-100 (x20) | 48 | -3 | 29.0 lb | 1100 | - |
| Full Plate Armour | 45 | 55-100 (x15) | 62 | -3 | 36.0 lb | 1350 | - |
| Metal Lamellar Armour | 45 | 50-100 (x20) | 58 | -3 | 34.0 lb | 1250 | - |
| Partial Plate Armour | 45 | 30-100 (x20) | 40 | -3 | 26.0 lb | 1200 | - |
| Ribbed Plate Armour | 50 | 60-100 (x15) | 66 | -3 | 38.0 lb | 1500 | - |
| Mithril Chain Mail | 55 | 55-100 (x10) | 28 | -1 | 15.0 lb | 7000 | IGNORE\_ACID |
| Mithril Plate Mail | 60 | 65-100 (x10) | 60 | -3 | 30.0 lb | 15000 | IGNORE\_ACID |
| Adamantite Plate Mail | 75 | 75-100 (x10) | 80 | -4 | 42.0 lb | 20000 | IGNORE\_ACID |

## Dragon Armor (12)

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| White Dragon Scale Mail | 25 | 30-100 (x8) | 14 +10 | -2 | 11.0 lb | 40000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist cold, activation: breath of cold (20-degree cone) for 100, recharge 50 |
| Black Dragon Scale Mail | 27 | 35-100 (x8) | 16 +10 | -2 | 12.0 lb | 40000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist acid, activation: breath of acid (20-degree cone) for 120, recharge 50 |
| Blue Dragon Scale Mail | 30 | 25-100 (x8) | 12 +10 | -2 | 10.0 lb | 40000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist lightning, activation: breath of lightning (20-degree cone) for 150, recharge 50 |
| Gold Dragon Scale Mail | 30 | 70-100 (x8) | 28 +10 | -2 | 18.0 lb | 40000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist sound, activation: breath of sound (20-degree cone) for 150, recharge 50 |
| Green Dragon Scale Mail | 30 | 60-100 (x8) | 20 +10 | -2 | 14.0 lb | 60000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist poison, activation: breath of poison (20-degree cone) for 150, recharge 50 |
| Red Dragon Scale Mail | 32 | 40-100 (x8) | 24 +10 | -2 | 16.0 lb | 40000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist fire, activation: breath of fire (20-degree cone) for 200, recharge 50 |
| Shining Dragon Scale Mail | 37 | 75-100 (x6) | 32 +10 | -2 | 20.0 lb | 60000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist light, resist dark, activation: your choice of {breath of light (20-degree cone) for 200 / breath of dark (20-degree cone) for 200}, recharge 50 |
| Chaos Dragon Scale Mail | 45 | 85-100 (x5) | 40 +10 | -2 | 24.0 lb | 80000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist chaos, resist disenchantment, activation: your choice of {breath of chaos (20-degree cone) for 220 / breath of disenchantment (20-degree cone) for 220}, recharge 50 |
| Law Dragon Scale Mail | 45 | 80-100 (x6) | 36 +10 | -2 | 22.0 lb | 80000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist sound, resist shards, activation: your choice of {breath of sound (20-degree cone) for 230 / breath of shards (20-degree cone) for 230}, recharge 50 |
| Multi-Hued Dragon Scale Mail | 50 | 75-100 (x5) | 45 +10 | -2 | 26.0 lb | 150000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist acid, resist lightning, resist fire, resist cold, resist poison, activation: your choice of {breath of acid (20-degree cone) for 250 / breath of lightning (20-degree cone) for 250 / breath of fire (20-degree cone) for 250 / breath of cold (20-degree cone) for 250 / breath of poison (20-degree cone) for 250}, recharge 50 |
| Balance Dragon Scale Mail | 52 | 95-100 (x3) | 50 +10 | -2 | 28.0 lb | 100000 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist sound, resist shards, resist chaos, resist disenchantment, activation: your choice of {breath of sound (20-degree cone) for 250 / breath of shards (20-degree cone) for 250 / breath of chaos (20-degree cone) for 250 / breath of disenchantment (20-degree cone) for 250}, recharge 50 |
| Power Dragon Scale Mail | 57 | 100-100 (x1) | 60 +15 | -3 | 30.0 lb | 300000 | PROT\_CONF, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist acid, resist fire, resist cold, resist lightning, resist poison, resist disenchantment, resist sound, resist shards, resist light, resist dark, resist chaos, activation: breath of magic missiles (20-degree cone) for 300, recharge 50 |

## Shield (6)

*base flags: HATES\_ACID* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Wicker Shield | 1 | 1-100 (x20) | 2 | 0 | 3.0 lb | 20 | - |
| Small Metal Shield | 10 | 15-100 (x20) | 5 | 0 | 6.0 lb | 80 | - |
| Leather Shield | 15 | 10-100 (x20) | 8 | 0 | 9.0 lb | 200 | - |
| Large Metal Shield | 30 | 30-100 (x20) | 12 | 0 | 12.0 lb | 500 | - |
| Knight's Shield | 50 | 50-100 (x10) | 15 | 0 | 16.0 lb | 750 | - |
| Mithril Shield | 70 | 70-100 (x10) | 16 +10 | 0 | 10.0 lb | 10000 | IGNORE\_ACID |

## Cloak (4)

*base flags: HATES\_ACID, HATES\_FIRE* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Cloak | 1 | 1-100 (x20) | 1 | 0 | 1.0 lb | 3 | - |
| Fur Cloak | 1 | 20-100 (x20) | 3 | 0 | 2.5 lb | 600 | - |
| Ethereal Cloak | 50 | 70-100 (x10) | 0 +10 | 0 | 0.0 lb | 4500 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Elven Cloak | 60 | 40-100 (x8) | 6 | 0 | 0.5 lb | 2000 | STEALTH +1, SPEED +1 |

## Gloves (5)

*base flags: HATES\_ACID, HATES\_FIRE* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Set of Alchemist's Gloves | 0 | 50-100 (x10) | 0 +5+d5 | 0 | 0.5 lb | 500 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Set of Caestus | 0 | 20-100 (x10) | 5 | 0 | 4.0 lb | 100 | - |
| Set of Gauntlets | 0 | 10-100 (x20) | 3 | 0 | 2.5 lb | 45 | - |
| Set of Leather Gloves | 0 | 1-100 (x20) | 1 | 0 | 1.0 lb | 5 | - |
| Set of Mithril Gauntlets | 0 | 40-100 (x20) | 6 | 0 | 1.5 lb | 700 | IGNORE\_ACID, IGNORE\_FIRE |

## Boots (6)

*base flags: HATES\_ACID, HATES\_FIRE* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Pair of Ethereal Slippers | 0 | 30-100 (x10) | 0 | 0 | 0.0 lb | 4 | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Pair of Leather Sandals | 0 | 1-100 (x20) | 1 | 0 | 1.5 lb | 4 | - |
| Pair of Leather Boots | 3 | 3-100 (x20) | 2 | 0 | 2.0 lb | 7 | - |
| Pair of Iron Shod Boots | 5 | 5-100 (x20) | 4 | 0 | 3.5 lb | 12 | IGNORE\_FIRE |
| Pair of Steel Shod Boots | 20 | 20-100 (x20) | 7 | 0 | 6.0 lb | 50 | IGNORE\_FIRE |
| Pair of Mithril Shod Boots | 60 | 60-100 (x20) | 8 | 0 | 4.0 lb | 500 | IGNORE\_ACID, IGNORE\_FIRE |

## Helm (4)

*base flags: HATES\_ACID* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Hard Leather Cap | 3 | 3-100 (x20) | 2 | 0 | 2.0 lb | 12 | - |
| Metal Cap | 10 | 10-100 (x20) | 3 | 0 | 2.5 lb | 30 | - |
| Iron Helm | 20 | 20-100 (x20) | 7 | 0 | 5.0 lb | 75 | - |
| Steel Helm | 40 | 40-100 (x20) | 9 | 0 | 6.0 lb | 200 | - |

## Crown (4)

*base flags: HATES\_ACID* (`object_base.txt`).

| Name | Lvl | Alloc | AC | Hit | Weight | Cost | Notes |
|---|---:|---|---|---:|---:|---:|---|
| Massive Iron Crown | 44 | not random | 0 | 0 | 2.0 lb | 1000 | INSTA\_ART, QUEST\_ART |
| Golden Crown | 45 | 45-100 (x10) | 0 | 0 | 3.0 lb | 1000 | IGNORE\_ACID |
| Iron Crown | 45 | 45-100 (x20) | 0 | 0 | 2.0 lb | 500 | - |
| Jewel Encrusted Crown | 50 | 50-100 (x10) | 0 | 0 | 4.0 lb | 2000 | IGNORE\_ACID |

## Light (2)

*base flags: HATES\_FIRE; break chance 50%* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Weight | Mods / flags | Effect |
|---|---:|---|---:|---:|---|---|
| Wooden Torch | 1 | 1-100 (x70) | 2 | 2.2 lb | BURNS\_OUT, LIGHT\_2, EASY\_KNOW | - |
| Lantern | 3 | 5-100 (x70) | 35 | 5.0 lb | TAKES\_FUEL, EASY\_KNOW, LIGHT\_3, IGNORE\_FIRE | - |

## Ring (30)

*base flags: HATES\_ELEC* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Weight | Mods / flags | Effect |
|---|---:|---|---:|---:|---|---|
| Digging | 0 | 10-50 (x20) | 800 | 0.2 lb | TUNNEL 2+d3 | activation: line of stone to mud for 20+1d30 (recharge 15+2d10) |
| Light | 0 | 25-100 (x20) | 400 | 0.2 lb | SEARCH 1+M5, LIGHT +1, resist light | - |
| Reckless Attacks | 0 | 10-50 (x30) | 200 | 0.2 lb | - | - |
| the Dog | 0 | 10-50 (x50) | 100 | 0.2 lb | PROT\_FEAR, EASY\_KNOW, STEALTH -2 | - |
| Feather Falling | 5 | 10-50 (x20) | 200 | 0.2 lb | FEATHER, EASY\_KNOW | - |
| Open Wounds | 5 | 10-50 (x20) | 700 | 0.2 lb | IMPAIR\_HP, EASY\_KNOW | activation: heal 40 HP; cure blindness; cure wounds; cure confusion (recharge 10+5d8) |
| Searching | 5 | 5-90 (x20) | 250 | 0.2 lb | SEARCH d4M2 | - |
| Slow Digestion | 5 | 10-50 (x30) | 250 | 0.2 lb | SLOW\_DIGEST, EASY\_KNOW | - |
| Teleportation | 5 | 10-50 (x20) | 1000 | 0.2 lb | EASY\_KNOW, SPEED +2, curse teleportation power 100 | - |
| Protection | 10 | 10-40 (x50) | 500 | 0.2 lb | - | - |
| Resist Fire and Cold | 10 | 10-50 (x50) | 250 | 0.2 lb | IGNORE\_COLD, IGNORE\_FIRE, EASY\_KNOW, resist fire, resist cold | - |
| Accuracy | 20 | 20-100 (x30) | 500 | 0.2 lb | - | - |
| Damage | 20 | 20-100 (x30) | 500 | 0.2 lb | - | - |
| Free Action | 20 | 20-100 (x50) | 1500 | 0.2 lb | FREE\_ACT, EASY\_KNOW | - |
| Acid | 25 | 20-100 (x20) | 500 | 0.2 lb | IGNORE\_ACID, resist acid | activation: ball of acid (radius 2) for 70; +20+1d20 turns of acid resistance (recharge 50+d50) |
| Ice | 28 | 20-100 (x20) | 500 | 0.2 lb | IGNORE\_COLD, resist cold | activation: ball of cold (radius 2) for 75; +20+1d20 turns of cold resistance (recharge 50+d50) |
| Bodykeeping | 30 | 30-100 (x20) | 750 | 0.2 lb | SUST\_STR, SUST\_DEX, SUST\_CON, EASY\_KNOW | - |
| Constitution | 30 | 30-100 (x50) | 500 | 0.2 lb | SUST\_CON, CON 1+M5 | - |
| Dexterity | 30 | 30-100 (x50) | 500 | 0.2 lb | SUST\_DEX, DEX 1+M5 | - |
| Escaping | 30 | 15-80 (x50) | 4000 | 0.2 lb | AFRAID, EASY\_KNOW, SPEED +2 | - |
| Flames | 30 | 20-100 (x20) | 500 | 0.2 lb | IGNORE\_FIRE, resist fire | activation: ball of fire (radius 2) for 80; +20+1d20 turns of fire resistance (recharge 50+d50) |
| Intelligence | 30 | 30-100 (x50) | 500 | 0.2 lb | SUST\_INT, INT 1+M5 | - |
| See Invisible | 30 | 30-100 (x50) | 340 | 0.2 lb | SEE\_INVIS, EASY\_KNOW | - |
| Soulkeeping | 30 | 30-100 (x20) | 600 | 0.2 lb | SUST\_STR, SUST\_INT, SUST\_WIS, EASY\_KNOW | - |
| Strength | 30 | 30-100 (x50) | 500 | 0.2 lb | SUST\_STR, STR 1+M5 | - |
| the Mouse | 30 | 10-50 (x50) | 100 | 0.2 lb | DEX d4, STEALTH d4 | - |
| Lightning | 32 | 20-100 (x20) | 500 | 0.2 lb | IGNORE\_ELEC, resist lightning | activation: ball of lightning (radius 2) for 85; +20+1d20 turns of electricity resistance (recharge 50+d50) |
| Resist Poison | 40 | 40-100 (x50) | 16000 | 0.2 lb | EASY\_KNOW, resist poison | - |
| Slaying | 40 | 40-100 (x20) | 1000 | 0.2 lb | - | - |
| Speed | 75 | 75-100 (x40) | 100000 | 0.2 lb | GOOD, SPEED 4+M6 | - |

## Amulet (17)

| Name | Lvl | Alloc | Cost | Weight | Mods / flags | Effect |
|---|---:|---|---:|---:|---|---|
| Adornment | 10 | 10-40 (x20) | 0 | 0.3 lb | EASY\_KNOW | - |
| Infravision | 10 | 10-70 (x30) | 200 | 0.3 lb | INFRA 1+M5 | - |
| Resist Acid | 10 | 10-50 (x50) | 300 | 0.3 lb | IGNORE\_ACID, EASY\_KNOW, resist acid | - |
| Resist Lightning | 10 | 10-50 (x50) | 300 | 0.3 lb | IGNORE\_ELEC, EASY\_KNOW, resist lightning | - |
| Teleportation | 10 | 10-40 (x20) | 10 | 0.3 lb | EASY\_KNOW, curse teleportation power 100 | - |
| Inertia | 15 | 15-40 (x20) | 10 | 0.3 lb | FREE\_ACT, SPEED -3+1d2 | - |
| Searching | 15 | 10-40 (x20) | 600 | 0.3 lb | SEARCH d5M5 | - |
| Slow Digestion | 15 | 15-40 (x20) | 200 | 0.3 lb | SLOW\_DIGEST, EASY\_KNOW | - |
| Regeneration | 30 | 30-100 (x20) | 600 | 0.3 lb | REGEN, EASY\_KNOW | - |
| Wisdom | 30 | 20-90 (x50) | 500 | 0.3 lb | SUST\_WIS, WIS 1+M5 | - |
| ESP | 50 | 60-100 (x10) | 25000 | 0.3 lb | TELEPATHY, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - |
| Resistance | 60 | 60-100 (x10) | 20000 | 0.3 lb | EASY\_KNOW, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, resist fire, resist acid, resist lightning, resist cold | - |
| Sustenance | 60 | 60-100 (x10) | 20000 | 0.3 lb | SUST\_STR, SUST\_INT, SUST\_WIS, SUST\_DEX, SUST\_CON, HOLD\_LIFE, SLOW\_DIGEST, EASY\_KNOW, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD | - |
| Devotion | 70 | 70-100 (x10) | 30000 | 0.3 lb | HOLD\_LIFE, SUST\_WIS, GOOD, WIS 1+M3, LIGHT +1, resist dark, resist light, resist fire | - |
| the Magi | 70 | 70-100 (x10) | 30000 | 0.3 lb | FREE\_ACT, SEE\_INVIS, SUST\_INT, PROT\_BLIND, GOOD, IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD, SEARCH 1+M3, INT 1+M3 | - |
| Trickery | 70 | 70-100 (x10) | 30000 | 0.3 lb | SUST\_DEX, GOOD, DEX 1+M4, SPEED 1+M2, STEALTH 1+M2, SEARCH 1+d3M3, INFRA 1+d3M3, resist nexus, resist poison | - |
| Weaponmastery | 70 | 70-100 (x10) | 30000 | 0.3 lb | SUST\_STR, SUST\_CON, FREE\_ACT, PROT\_FEAR, GOOD, STR 1+M2, resist disenchantment | - |

## Potion (41)

*base flags: HATES\_COLD, HATES\_SOUND, HATES\_SHARD, HATES\_ICE, HATES\_FORCE, EASY\_KNOW; break chance 100%* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Pile | Notes |
|---|---:|---|---:|---|---|---|
| Blindness | 0 | 0-10 (x10) | 0 | +75+4d25 turns of blindness | 70:2d3 | - |
| Confusion | 0 | 0-10 (x10) | 0 | +10+4d5 turns of confusion; nourish +1 (x food-value) | 70:2d3 | - |
| Cure Light Wounds | 0 | 1-15 (x80) | 20 | heal 20 HP; cure blindness; reduce wounds by 20; reduce confusion by 20; nourish +1 (x food-value) | 90:2d3 | - |
| Dragon Breath | 0 | 40-100 (x20) | 100 | your choice of {breath of fire (20-degree cone) for 80 / breath of cold (20-degree cone) for 80} | 100:1d2 | - |
| Salt Water | 0 | 3-15 (x10) | 0 | set food to 2 (x food-value); cure poisoning; +4 turns of paralysis (unresistable) | 100:1d2 | - |
| Sleep | 0 | 0-10 (x10) | 0 | +5+1d5 turns of paralysis; nourish +1 (x food-value) | 70:2d3 | - |
| Slime Mold Juice | 0 | 1-10 (x10) | 2 | nourish +10 (x food-value) | 70:2d3 | - |
| Boldness | 1 | 1-90 (x40) | 8 | cure fear | 25:2 | - |
| Heroism | 1 | 1-100 (x40) | 25 | heal 10 HP; cure fear; +25+1d25 turns of heroism | 25:2 | - |
| Slowness | 1 | 0-10 (x10) | 0 | +15+1d25 turns of slowness; nourish +1 (x food-value) | 70:2d3 | - |
| Speed | 1 | 1-100 (x40) | 75 | +20+2d10 turns of haste | 25:2 | - |
| Berserk Strength | 3 | 3-100 (x40) | 80 | heal 30 HP; cure fear; +25+1d25 turns of berserk rage | 25:2 | - |
| Brawn | 3 | 20-40 (x30) | 1000 | restore STR; increase STR; lose a random stat other than STR | 10:2 | - |
| Cure Serious Wounds | 3 | 5-40 (x80) | 70 | heal 40 HP; cure blindness; cure wounds; cure confusion; nourish +1 (x food-value) | 80:2d3 | - |
| Infravision | 3 | 3-40 (x40) | 20 | +100+4d25 turns of enhanced infravision | 25:2 | - |
| Poison | 3 | 0-10 (x10) | 0 | +10+2d7 turns of poisoning | 70:2d3 | - |
| True Seeing | 3 | 3-40 (x40) | 50 | cure blindness; +50+2d6 turns of see invisible | 25:2 | - |
| Cure Critical Wounds | 5 | 12-100 (x80) | 200 | heal 60 HP; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion; nourish +1 (x food-value) | 70:2d3 | - |
| Neutralize Poison | 5 | 1-35 (x60) | 75 | cure poisoning | 25:2 | - |
| Nimbleness | 5 | 20-40 (x30) | 1000 | restore DEX; increase DEX; lose a random stat other than DEX | 10:2 | - |
| Surprise | 10 | 10-15 (x10) | 0 | one at random of {+20+1d20 turns of scrambled / change into bat shape / drain INT / drain WIS / +20+1d20 turns of stone skin} | 25:2 | - |
| Toughness | 10 | 20-40 (x30) | 1000 | restore CON; increase CON; lose a random stat other than CON | 10:2 | - |
| Healing | 15 | 30-100 (x64) | 800 | heal 300 HP or 35% of HP lost, whichever is more; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion; nourish +1 (x food-value) | 25:2 | - |
| Contemplation | 20 | 20-40 (x30) | 1000 | restore WIS; increase WIS; lose a random stat other than WIS | 10:2 | - |
| Intellect | 20 | 20-40 (x30) | 1000 | restore INT; increase INT; lose a random stat other than INT | 10:2 | - |
| Resist Poison | 20 | 20-100 (x40) | 30 | +50+1d10 turns of poison resistance; cure poisoning | 25:2 | - |
| Enlightenment | 25 | 25-100 (x40) | 800 | completely lights up and magically maps the level (1) | 25:2 | - |
| Restore Mana | 25 | 15-100 (x64) | 350 | restore all SP | 25:2 | - |
| Constitution | 30 | 30-100 (x30) | 8000 | restore CON; increase CON | 10:2 | - |
| Dexterity | 30 | 30-100 (x30) | 8000 | restore DEX; increase DEX | 10:2 | - |
| Intelligence | 30 | 30-100 (x30) | 8000 | restore INT; increase INT | 10:2 | - |
| Resist Cold | 30 | 30-100 (x40) | 30 | +50+1d10 turns of cold resistance | 25:2 | - |
| Resist Heat | 30 | 30-100 (x40) | 30 | +50+1d10 turns of fire resistance | 25:2 | - |
| Strength | 30 | 30-100 (x30) | 8000 | restore STR; increase STR | 10:2 | - |
| Wisdom | 30 | 30-100 (x30) | 8000 | restore WIS; increase WIS | 10:2 | - |
| \*Healing\* | 40 | 40-100 (x40) | 1500 | heal 1200 HP; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion | - | - |
| Augmentation | 40 | 40-100 (x20) | 50000 | restore STR; increase STR; restore INT; increase INT; restore WIS; increase WIS; restore DEX; increase DEX; restore CON; increase CON | - | - |
| Restore Life Levels | 40 | 30-100 (x40) | 400 | restores your experience | 25:2 | - |
| Life | 60 | 60-100 (x20) | 5000 | restores your experience; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion; cure hallucination; restore STR; restore INT; restore WIS; restore DEX; restore CON; heal 5000 HP | - | - |
| Experience | 65 | 65-100 (x50) | 25000 | gain 200000 experience | - | - |
| \*Enlightenment\* | 70 | 70-100 (x8) | 12000 | completely lights up and magically maps the level (1); restore INT; increase INT; restore WIS; increase WIS; detects veins nearby (area 22 x 40); detects gold nearby (area 22 x 40); detects objects nearby (area 22 x 40); detects invisible creatures nearby (area 500 x 500); detects visible creatures nearby (area 500 x 500) | - | - |

## Scroll (39)

*base flags: HATES\_ACID, HATES\_FIRE, EASY\_KNOW* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Pile | Notes |
|---|---:|---|---:|---|---|---|
| Light | 0 | 0-60 (x50) | 15 | lights up the surrounding area; sphere of weak light around you (radius 2) for 2d8 | 35:2d3 | - |
| Treasure Detection | 0 | 0-70 (x50) | 70 | detects veins nearby (area 22 x 40); senses gold nearby (area 22 x 40); senses objects nearby (area 22 x 40) | 40:1d5 | - |
| Blessing | 1 | 1-50 (x50) | 15 | +6+1d12 turns of your AC and to-hit bonus | 70:1d3 | - |
| Darkness | 1 | 1-70 (x10) | 0 | darken the area; sphere of weak darkness around you (radius 3) for 10 | 70:1d3 | - |
| Detect Invisible | 1 | 1-70 (x50) | 15 | detects invisible creatures nearby (area 22 x 40) | 70:1d3 | - |
| Identify Rune | 1 | 5-100 (x50) | 200 | identifies a single unknown rune on a selected item | 70:1d4 | - |
| Phase Door | 1 | 1-100 (x50) | 15 | teleport self up to 10 grids | 100:2d3 | - |
| Summon Monster | 1 | 1-100 (x10) | 0 | summon 1d3 x a monster | 70:1d3 | - |
| Aggravate Monster | 5 | 5-70 (x10) | 0 | wake up nearby monsters; hastes monsters in line of sight (power 25) | 70:1d3 | - |
| Magic Mapping | 5 | 5-100 (x50) | 65 | maps the area around you (area 22 x 40) | 70:1d3 | - |
| Monster Confusion | 5 | 5-60 (x16) | 30 | +10000 turns of temporary monster confusion | 25:2 | - |
| Remove Hunger | 5 | 5-100 (x60) | 10 | nourish up to 50 (x food-value) | 100:2d3 | - |
| Word of Recall | 5 | 5-100 (x50) | 125 | returns you from the dungeon or takes you to the dungeon after a short delay | 35:2d3 | - |
| Deep Descent | 10 | 1-100 (x20) | 250 | teleports you up to five dungeon levels lower than the lowest point you have reached so far | 20:2 | - |
| Door Destruction | 10 | 10-60 (x40) | 50 | destroys doors on adjacent grids (radius 1) | 25:2d3 | - |
| Holy Chant | 10 | 10-70 (x40) | 40 | +12+1d24 turns of your AC and to-hit bonus | 25:2 | - |
| Remove Curse | 10 | 10-100 (x16) | 100 | remove a curse (power 20+d20) | 25:2 | - |
| Teleportation | 10 | 10-100 (x40) | 40 | teleport self 60% of the level's larger dimension | 25:2 | - |
| Trap Creation | 10 | 10-70 (x10) | 0 | creates traps on adjacent grids (radius 3) | 70:1d3 | - |
| Enchant Armour | 15 | 15-70 (x40) | 125 | enchant armour class (power 1) | 25:2 | - |
| Enchant Weapon To-Dam | 15 | 15-70 (x40) | 125 | enchant to-dam (power 1) | 25:2 | - |
| Enchant Weapon To-Hit | 15 | 15-70 (x40) | 125 | enchant to-hit (power 1) | 25:2 | - |
| Summon Undead | 15 | 15-100 (x10) | 0 | summon 1d3 x undead | 70:1d3 | - |
| Acquirement | 20 | 20-100 (x10) | 5000 | creates good items nearby (1) | - | - |
| Teleport Level | 20 | 20-100 (x16) | 800 | teleport level (up or down) | 10:2 | - |
| Holy Prayer | 25 | 50-100 (x40) | 80 | +24+1d48 turns of your AC and to-hit bonus | 25:2 | - |
| Protection from Evil | 30 | 30-100 (x40) | 50 | +(plev\*3)+1d25 turns of protection from evil | 25:2 | - |
| \*Destruction\* | 40 | 40-100 (x30) | 1000 | destruction (radius 15) | - | - |
| Banishment | 40 | 40-100 (x25) | 2500 | removes all of a given creature type from the level | - | - |
| Dispel Undead | 40 | 40-100 (x40) | 200 | damages undead in line of sight (power 60) | 25:2 | - |
| Recharging | 40 | 40-100 (x40) | 200 | recharge a wand/staff (power 6) | 12:2d3 | - |
| \*Enchant Armour\* | 50 | 50-100 (x40) | 500 | enchant armour class (power 2+1d3) | 25:2 | - |
| \*Enchant Weapon\* | 50 | 50-100 (x40) | 500 | enchant to-hit and to-dam (power 1d3) | 25:2 | - |
| \*Remove Curse\* | 50 | 50-100 (x16) | 8000 | remove a curse (power 50+d50) | 25:2 | - |
| Curse Armour | 50 | not random | 0 | curses your worn armor | 70:1d3 | - |
| Curse Weapon | 50 | not random | 0 | curses your wielded melee weapon | 70:1d3 | - |
| Mass Banishment | 50 | 50-100 (x20) | 3000 | removes all nearby creatures | - | - |
| \*Acquirement\* | 60 | 60-100 (x5) | 10000 | creates good items nearby (1+1d2) | - | - |
| Rune of Protection | 60 | 70-100 (x30) | 500 | inscribe a glyph of warding | - | - |

## Flask (1)

*base flags: HATES\_COLD, HATES\_SOUND, HATES\_SHARD, HATES\_ICE, HATES\_FORCE, EASY\_KNOW; break chance 100%* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Pile | Notes |
|---|---:|---|---:|---|---|---|
| Flask of oil | 1 | 1-100 (x50) | 3 | - | - | EXPLODE, THROWING |

## Food (14)

*base flags: EASY\_KNOW; break chance 100%* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Pile | Notes |
|---|---:|---|---:|---|---|---|
| Apple | 0 | 0-20 (x40) | 3 | nourish +10 (x food-value) | 10:1d2 | - |
| Draught of the Ents | 0 | 60-100 (x10) | 2000 | nourish up to 60 (x food-value); restore STR; restore CON; increase STR; increase CON | - | - |
| Flask of Whisky | 0 | not random | 1 | nourish +3 (x food-value); +(-1+1d5) turns of confusion | - | - |
| Handful of Dried Fruits | 0 | 0-50 (x40) | 3 | nourish +15 (x food-value) | - | - |
| Hard Biscuit | 0 | not random | 1 | nourish +10 (x food-value); crunches | - | - |
| Pint of Fine Wine | 0 | not random | 2 | nourish +3 (x food-value); +79+1d41 turns of fearlessness | - | - |
| Ration of Food | 0 | 0-50 (x40) | 3 | nourish +30 (x food-value) | 100:1d5 | - |
| Scrap of Flesh | 0 | 50-100 (x30) | 0 | one at random of {nourish +20 (x food-value) / nourish +20 (x food-value) / nourish +10 (x food-value) / set food to 1d4 (x food-value) / set food to 5+1d4 (x food-value)} | 50:1d3 | - |
| Sip of Miruvor | 0 | 10-50 (x20) | 100 | nourish +3 (x food-value); restore STR; restore CON | - | - |
| Slice of Meat | 0 | 10-50 (x30) | 3 | nourish +40 (x food-value) | 50:1d3 | - |
| Swig of Orcish Liquor | 0 | 20-100 (x20) | 1 | nourish +5 (x food-value); set stunning to 2+1d3 turns; set slowness to 2+1d3 turns | 10:1d3 | - |
| Slime Mold | 1 | 30-100 (x40) | 2 | nourish +15 (x food-value) | 100:2d3 | - |
| Honey-cake | 5 | 20-70 (x30) | 10 | nourish up to 60 (x food-value) | 40:1d3 | - |
| Piece of Elvish Waybread | 5 | 30-100 (x20) | 10 | nourish up to 75 (x food-value); cure poisoning; heal 4d8 HP | 10:1d3 | - |

## Mushroom (12)

*base flags: EASY\_KNOW; break chance 100%* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Pile | Notes |
|---|---:|---|---:|---|---|---|
| Clear Mind | 0 | 5-40 (x10) | 200 | restore 20 SP; cure confusion; cure fear; cure hallucination; +50+7d7 turns of confusion resistance; nourish +5 (x food-value) | 70:1d4 | - |
| Debility | 0 | 10-40 (x10) | 20 | restore all SP; nourish +5 (x food-value); one at random of {drain STR / drain CON} | 70:1d4 | - |
| Emergency | 0 | 5-100 (x10) | 60 | +25+5d5 turns of hallucination; +25+5d5 turns of fire resistance; +25+5d5 turns of cold resistance; heal 200 HP; nourish +5 (x food-value) | 70:1d4 | - |
| Fast Recovery | 0 | 5-40 (x10) | 200 | heal 30 HP; cure stunning; cure wounds; cure poisoning; cure blindness; nourish +5 (x food-value) | 70:1d4 | - |
| Purging | 0 | 15-100 (x1) | 60 | set food to 5 (x food-value); restore STR; restore CON; cure poisoning | 70:1d4 | - |
| Second Sight | 0 | 5-100 (x10) | 200 | cure blindness; +50+7d7 turns of telepathy; nourish +5 (x food-value) | 70:1d4 | - |
| Shadows | 0 | 50-100 (x20) | 300 | nourish +5 (x food-value); +10+d10 turns of stealth | 70:1d4 | - |
| Sprinting | 0 | 15-100 (x1) | 100 | +100 turns of sprinting; nourish +5 (x food-value) | 70:1d4 | - |
| Stoneskin | 0 | 10-40 (x10) | 60 | set stone skin to 50+7d7 turns; nourish +5 (x food-value) | 70:1d4 | - |
| Terror | 0 | 5-40 (x10) | 60 | set terror to 25+5d5 turns; nourish +5 (x food-value) | 70:1d4 | - |
| Turbulence | 0 | not random | 60 | - | 70:1d4 | - |
| Vigor | 0 | 20-100 (x25) | 200 | restore STR; restore INT; restore WIS; restore DEX; restore CON; nourish +5 (x food-value) | 13:1d3 | - |

## Wand (30)

*base flags: HATES\_ELEC, EASY\_KNOW* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Charges | Recharge | Notes |
|---|---:|---|---:|---|---|---|---|
| Darkness | 3 | 3-60 (x20) | 100 | line of weak darkness for 6d8 | 6+d10 | - | - |
| Haste Monster | 3 | 3-40 (x10) | 0 | bolt that hastes monsters (power 100) | 8+d20 | - | - |
| Heal Monster | 3 | 3-40 (x10) | 0 | bolt that heals monsters (power 4d6) | 8+d20 | - | - |
| Light | 3 | 3-60 (x20) | 200 | line of weak light for 6d8 | 6+d10 | - | - |
| Magic Missile | 3 | 3-60 (x20) | 200 | bolt or beam of magic missiles for 3d4 | 6+d10 | - | - |
| Wonder | 3 | 3-60 (x20) | 250 | creates random and unpredictable effects ((plev/5)+1d100) | 8+d15 | - | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Confuse Monster | 5 | 5-100 (x20) | 500 | bolt that attempts to confuse monsters (power 5+1d5) | 2+d6 | - | - |
| Hold Monster | 5 | 5-100 (x20) | 500 | bolt that attempts to hold monsters still (power 3+1d5) | 2+d6 | - | - |
| Stinking Cloud | 5 | 5-60 (x20) | 400 | ball of poison (radius 3) for 12 | 6+d8 | - | - |
| Stun Monster | 5 | 5-100 (x20) | 500 | bolt that attempts to stun monsters (power 5+1d5) | 2+d6 | - | - |
| Scare Monster | 10 | 10-100 (x20) | 500 | bolt that causes monsters to flee (power 10) | 3+d5 | - | - |
| Stone to Mud | 10 | 10-100 (x20) | 300 | line of stone to mud for 20+1d30 | 6+d10 | - | - |
| Clone Monster | 15 | 15-100 (x20) | 0 | bolt that hastes, heals and magically duplicates monsters | 3+d5 | - | - |
| Lightning Bolts | 15 | 15-60 (x20) | 600 | beam of lightning for 6d6 | 6+d8 | - | - |
| Slow Monster | 15 | 5-100 (x20) | 500 | bolt that attempts to slow monsters (power 10+1d20) | 2+d6 | - | - |
| Disable Traps | 20 | 20-60 (x20) | 700 | line that disables traps | 4+d5 | - | - |
| Frost Bolts | 20 | 20-60 (x20) | 800 | bolt of cold for 12d8 | 6+d5 | - | - |
| Polymorph | 20 | 20-60 (x20) | 400 | bolt that polymorphs monsters into other kinds of creatures (power plev) | 6+d8 | - | - |
| Teleport Other | 20 | 20-100 (x20) | 3500 | bolt that teleports monsters away (power (sight\*5)) | 6+d5 | - | - |
| Acid Bolts | 30 | 30-60 (x20) | 950 | bolt or beam of acid for 10d8 | 6+d8 | - | - |
| Fire Bolts | 30 | 30-60 (x20) | 1000 | bolt or beam of fire for 12d8 | 6+d8 | - | - |
| Lightning Balls | 35 | 35-100 (x20) | 1200 | ball of lightning (radius 2) for 64 | 4+d8 | - | IGNORE\_ELEC |
| Cold Balls | 40 | 40-100 (x20) | 1500 | ball of cold (radius 2) for 100 | 2+d6 | - | IGNORE\_COLD |
| Acid Balls | 50 | 50-100 (x20) | 1650 | ball of acid (radius 2) for 120 | 2+d5 | - | IGNORE\_ACID |
| Dragon's Frost | 50 | 50-100 (x20) | 2400 | arc of cold (30-degree) for 160 | 1+d3 | - | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Drain Life | 50 | 50-100 (x50) | 1200 | bolt that damages living monsters for 150 | 3+d3 | - | - |
| Dragon's Flame | 55 | 50-100 (x20) | 2400 | arc of fire (30-degree) for 200 | 1+d3 | - | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Fire Balls | 55 | 50-100 (x20) | 1800 | ball of fire (radius 2) for 144 | 2+d4 | - | IGNORE\_FIRE |
| Annihilation | 60 | 60-100 (x20) | 3000 | bolt that damages living monsters for 250 | 1+d2 | - | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |
| Dragon's Breath | 60 | 60-100 (x20) | 2400 | your choice of {arc of acid (30-degree) for 160 / arc of lightning (30-degree) for 160 / arc of fire (30-degree) for 160 / arc of cold (30-degree) for 160 / arc of poison (30-degree) for 160} | 1+d3 | - | IGNORE\_ACID, IGNORE\_ELEC, IGNORE\_FIRE, IGNORE\_COLD |

## Staff (24)

*base flags: HATES\_ACID, HATES\_FIRE, EASY\_KNOW* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Charges | Recharge | Notes |
|---|---:|---|---:|---|---|---|---|
| Cure Light Wounds | 5 | 5-100 (x30) | 350 | heal 20 HP; cure blindness; reduce wounds by 20; reduce confusion by 20 | 6+d5 | - | - |
| Darkness | 5 | 5-40 (x10) | 0 | darken the area; sphere of weak darkness around you (radius 3) for 10 | 8+d8 | - | - |
| Detect Invisible | 5 | 5-100 (x30) | 200 | detects invisible creatures nearby (area 22 x 40) | 8+d15 | - | - |
| Light | 5 | 5-80 (x30) | 250 | lights up the surrounding area; sphere of weak light around you (radius 2) for 2d8 | 8+d20 | - | - |
| Confuse Monsters | 10 | 10-100 (x20) | 800 | attempts to confuse monsters in line of sight (power 5+1d5) | 3+d3 | - | - |
| Haste Monsters | 10 | 10-40 (x10) | 0 | hastes monsters in line of sight (power 50) | 8+d8 | - | - |
| Sleep Monsters | 10 | 10-100 (x20) | 700 | attempts to put all monsters to sleep in line of sight (power (plev\*10+500)) | 6+d5 | - | - |
| Summoning | 10 | 10-100 (x10) | 0 | summon 1d3 x a monster | 2d3 | - | - |
| Detect Evil | 20 | 20-100 (x30) | 350 | detects evil creatures nearby (area 22 x 40) | 8+d15 | - | - |
| Mapping | 20 | 10-100 (x40) | 600 | maps the area around you (area 22 x 40) | 5+d5 | - | - |
| Slow Monsters | 20 | 10-100 (x20) | 800 | attempts to slow monsters in line of sight (power 10+1d10) | 4+d4 | - | - |
| Starlight | 20 | 20-80 (x20) | 800 | beams of light in all 8 directions for 6d8 | 6+d5 | - | - |
| Teleportation | 20 | 20-100 (x50) | 2000 | teleport self 60% of the level's larger dimension | 5+d4 | - | - |
| Curing | 25 | 10-100 (x30) | 500 | cure blindness; cure wounds; cure poisoning; cure stunning; cure confusion | 4+d3 | - | - |
| Earthquakes | 40 | 40-100 (x20) | 350 | earthquake (radius 10) | 3+d5 | - | - |
| Remove Curse | 40 | 40-100 (x20) | 1500 | remove a curse (power 35+d30) | 4+d3 | - | - |
| Speed | 40 | 40-100 (x20) | 2000 | +20+2d10 turns of haste | 4+d3 | - | - |
| \*Destruction\* | 50 | 50-100 (x20) | 6000 | destruction (radius 15) | 1+d3 | - | - |
| Dispel Evil | 50 | 50-100 (x50) | 1200 | damages evil monsters in line of sight (power 60) | 4+d3 | - | - |
| Banishment | 70 | 70-100 (x10) | 5000 | removes all of a given creature type from the level | 1+d2 | - | - |
| Healing | 70 | 70-100 (x10) | 5000 | heal 300 HP or 35% of HP lost, whichever is more; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion | 1+d2 | - | - |
| Holiness | 70 | 70-100 (x20) | 3500 | damages evil monsters in line of sight (power 120); heal 50 HP; +(plev\*3)+1d25 turns of protection from evil; cure poisoning; cure terror; cure fear; cure stunning; cure wounds; cure slowness; cure blindness; cure confusion; cure hallucination; cure amnesia | 2d2 | - | - |
| Power | 70 | 70-100 (x20) | 4000 | damages all monsters in line of sight (power 120) | 1+d3 | - | - |
| the Magi | 70 | 70-100 (x10) | 4500 | restore INT; restore all SP | 2+1d2 | - | - |

## Rod (25)

*base flags: HATES\_ELEC, EASY\_KNOW* (`object_base.txt`).

| Name | Lvl | Alloc | Cost | Effect | Charges | Recharge | Notes |
|---|---:|---|---:|---|---|---|---|
| Light | 10 | 5-100 (x30) | 500 | line of weak light for 6d8 | - | 9 | - |
| Treasure Location | 10 | 8-75 (x30) | 1000 | detects veins nearby (area 22 x 40); senses gold nearby (area 22 x 40); senses objects nearby (area 22 x 40) | - | 50 | - |
| Illumination | 20 | 10-100 (x20) | 1000 | lights up the surrounding area; sphere of weak light around you (radius 2) for 2d8 | - | 30 | - |
| Lightning Bolts | 20 | 20-80 (x20) | 2000 | beam of lightning for 6d6 | - | 11 | - |
| Frost Bolts | 25 | 25-80 (x20) | 2500 | bolt of cold for 12d8 | - | 13 | - |
| Detection | 30 | 50-100 (x30) | 3000 | detects veins nearby (area 22 x 40); senses gold nearby (area 22 x 40); senses objects nearby (area 22 x 40); detects invisible creatures nearby (area 22 x 40); detects visible creatures nearby (area 22 x 40) | - | 100 | IGNORE\_ELEC |
| Hold Monster | 30 | 30-100 (x20) | 1500 | bolt that attempts to hold monsters still (power 3+1d5) | - | 40 | - |
| Probing | 30 | 35-100 (x20) | 1000 | gives you information on the health and abilities of monsters you can see | - | 100 | - |
| Recall | 30 | 30-100 (x10) | 4000 | returns you from the dungeon or takes you to the dungeon after a short delay | - | 60 | IGNORE\_ELEC |
| Disable Traps | 35 | 10-100 (x10) | 2100 | line that disables traps | - | 30 | - |
| Fire Bolts | 35 | 30-80 (x20) | 3000 | bolt or beam of fire for 12d8 | - | 15 | - |
| Magic Mapping | 35 | 35-100 (x30) | 2000 | maps the area around you (area 33 x 55) | - | 100 | - |
| Polymorph | 35 | 35-80 (x20) | 1200 | bolt that polymorphs monsters into other kinds of creatures (power plev) | - | 25 | - |
| Teleport Other | 35 | 45-100 (x20) | 5000 | bolt that teleports monsters away (power (sight\*5)) | - | 25 | - |
| Acid Bolts | 40 | 40-80 (x20) | 3500 | bolt or beam of acid for 12d8 | - | 12 | - |
| Curing | 40 | 10-80 (x20) | 1000 | cure blindness; cure wounds; cure poisoning; cure stunning; cure confusion | - | 100 | - |
| Slow Monster | 40 | 30-100 (x20) | 1500 | bolt that attempts to slow monsters (power 10+1d20) | - | 40 | - |
| Lightning Balls | 55 | 55-100 (x20) | 4000 | ball of lightning (radius 2) for 64 | - | 23 | IGNORE\_ELEC |
| Speed | 55 | 80-100 (x10) | 25000 | +20+2d10 turns of haste | - | 100 | IGNORE\_ELEC |
| Cold Balls | 60 | 60-100 (x20) | 4500 | ball of cold (radius 2) for 100 | - | 25 | - |
| Acid Balls | 70 | 70-100 (x20) | 5500 | ball of acid (radius 2) for 120 | - | 27 | - |
| Drain Life | 75 | 75-100 (x20) | 3600 | bolt that damages living monsters for 150 | - | 23 | - |
| Fire Balls | 75 | 75-100 (x20) | 5000 | ball of fire (radius 2) for 144 | - | 30 | - |
| Healing | 80 | 80-100 (x20) | 20000 | heal 500 HP; cure wounds; cure stunning | - | 1000 | IGNORE\_ELEC |
| Restoration | 80 | 80-100 (x10) | 25000 | restore STR; restore INT; restore WIS; restore DEX; restore CON; restores your experience | - | 1000 | IGNORE\_ELEC |

## Chest (6)

*base flags: HATES\_ACID, HATES\_FIRE* (`object_base.txt`).

Chest traps are listed in [A10](A10-curses-traps-timed-effects.md#chest-traps).

| Name | Lvl | Alloc | Cost | Weight | Mods / flags | Effect |
|---|---:|---|---:|---:|---|---|
| Small wooden chest | 5 | 5-100 (x15) | 20 | 25.0 lb | - | - |
| Large wooden chest | 15 | 15-100 (x10) | 60 | 50.0 lb | - | - |
| Small iron chest | 25 | 25-100 (x15) | 100 | 30.0 lb | - | - |
| Large iron chest | 35 | 35-100 (x10) | 150 | 100.0 lb | - | - |
| Small steel chest | 45 | 45-100 (x15) | 200 | 50.0 lb | - | - |
| Large steel chest | 55 | 55-100 (x10) | 250 | 100.0 lb | - | - |

## Gold (11)

Gold kinds only name the drop: its value is `rand_spread(16 * level / 10 + 16, level + 10)`, multiplied by 10 with chance 1/100 (repeatable), x5 with `birth_no_selling` in the dungeon, capped near 32767; the kind is picked from the value by `money_kind()` (`make_gold()` in `src/obj-make.c`).

| Name | Lvl | Alloc | Cost | Weight | Mods / flags | Effect |
|---|---:|---|---:|---:|---|---|
| adamantite | 0 | not random | 0 | 0.0 lb | - | - |
| copper | 0 | not random | 0 | 0.0 lb | - | - |
| diamonds | 0 | not random | 0 | 0.0 lb | - | - |
| emeralds | 0 | not random | 0 | 0.0 lb | - | - |
| garnets | 0 | not random | 0 | 0.0 lb | - | - |
| gold | 0 | not random | 0 | 0.0 lb | - | - |
| mithril | 0 | not random | 0 | 0.0 lb | - | - |
| opals | 0 | not random | 0 | 0.0 lb | - | - |
| rubies | 0 | not random | 0 | 0.0 lb | - | - |
| sapphires | 0 | not random | 0 | 0.0 lb | - | - |
| silver | 0 | not random | 0 | 0.0 lb | - | - |

## Spell books (20)

Books are defined in `class.txt` (`book:` and `book-properties:` lines), not in `object.txt`. *Town* books are stocked by the Bookseller ([A9](A9-stores.md)); *dungeon* books are only found in the dungeon or the black market. Spells are listed in [A7](A7-class-spells.md).

| Book | Type | Source | Realm | Cost | Alloc | Used by |
|---|---|---|---|---:|---|---|
| \[First Spells\] | magic book | town | arcane | 25 | 1-100 (x40) | Mage, Rogue |
| \[Attacks and Knowledge\] | magic book | town | arcane | 400 | 10-100 (x40) | Mage |
| \[Magical Defences\] | magic book | town | arcane | 1600 | 30-100 (x20) | Mage |
| \[Arcane Control\] | magic book | dungeon | arcane | 30000 | 40-100 (x15) | Mage, Rogue |
| \[Wizard's Tome of Power\] | magic book | dungeon | arcane | 50000 | 60-100 (x10) | Mage |
| \[Lesser Charms\] | nature book | town | nature | 25 | 1-100 (x40) | Druid, Ranger |
| \[Gifts of Nature\] | nature book | town | nature | 400 | 10-100 (x40) | Druid |
| \[Creature Dominion\] | nature book | dungeon | nature | 10000 | 30-100 (x20) | Druid |
| \[Nature Craft\] | nature book | dungeon | nature | 30000 | 40-100 (x15) | Druid, Ranger |
| \[Wild Forces\] | nature book | dungeon | nature | 50000 | 60-100 (x10) | Druid |
| \[Novice's Handbook\] | prayer book | town | divine | 25 | 1-100 (x40) | Priest, Paladin |
| \[Cleansing Power\] | prayer book | town | divine | 100 | 5-100 (x40) | Priest |
| \[Healing and Sanctuary\] | prayer book | dungeon | divine | 10000 | 30-100 (x20) | Priest, Paladin |
| \[Battle Blessings\] | prayer book | dungeon | divine | 30000 | 40-100 (x15) | Priest, Paladin |
| \[Wrath of the Valar\] | prayer book | dungeon | divine | 50000 | 60-100 (x10) | Priest |
| \[Into the Shadows\] | shadow book | town | shadow | 25 | 1-100 (x40) | Necromancer, Blackguard |
| \[Dark Rituals\] | shadow book | town | shadow | 400 | 10-100 (x40) | Necromancer |
| \[Fear and Torment\] | shadow book | town | shadow | 1600 | 30-100 (x20) | Necromancer, Blackguard |
| \[Deadly Powers\] | shadow book | dungeon | shadow | 30000 | 40-100 (x15) | Necromancer, Blackguard |
| \[Corruption of Spirit\] | shadow book | dungeon | shadow | 50000 | 60-100 (x10) | Necromancer |

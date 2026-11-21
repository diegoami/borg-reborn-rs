# Appendix A9. Stores

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/store.txt`, `lib/gamedata/terrain.txt`, `lib/gamedata/object.txt`, `lib/gamedata/constants.txt`, `src/store.c` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Each town store: owners and purse limits, stock slots and turnover, items always stocked, the list normal stock is drawn from (with base cost and effect), what the store buys; then the pricing and restocking rules from `src/store.c`. Shopping strategy: [Chapter 11](../11-town-and-shopping.md).

## Contents

- [Rules from the source](#rules-from-the-source)
- [General Store](#general-store)
  - [General Store: always in stock](#general-store-always-in-stock)
  - [General Store: normal stock list](#general-store-normal-stock-list)
- [Armoury](#armoury)
  - [Armoury: normal stock list](#armoury-normal-stock-list)
- [Weapon Smiths](#weapon-smiths)
  - [Weapon Smiths: normal stock list](#weapon-smiths-normal-stock-list)
- [Bookseller](#bookseller)
  - [Bookseller: always in stock](#bookseller-always-in-stock)
- [Alchemy Shop](#alchemy-shop)
  - [Alchemy Shop: always in stock](#alchemy-shop-always-in-stock)
  - [Alchemy Shop: normal stock list](#alchemy-shop-normal-stock-list)
- [Magic Shop](#magic-shop)
  - [Magic Shop: normal stock list](#magic-shop-normal-stock-list)
- [Black Market](#black-market)
- [Home](#home)

## Rules from the source

FACT, from `src/store.c` unless noted:

- **Buying from a store** (`price_item()`, store selling): price = the object's full value (`object_value_real()`), times the quantity; the **black market** doubles it and then applies an adjust factor of 150%, i.e. **3x value**.
- **Selling to a store** (store buying): price = min(real value, known value) `* 2 / 3`; the black market pays half of that and applies a 50% factor, i.e. **1/6 of value**. The result is capped at the owner's purse (`owner:` max cost) per item. With `birth_no_selling` stores pay 0 (gold drops in the dungeon are increased instead, see [Chapter 11](../11-town-and-shopping.md)).
- Stores refuse worthless items and anything not on their `buy:` list (`store_will_buy()`); the home accepts anything.
- **Normal stock** is generated at object level `rand_range(1, 5 + max(max_depth - 20, 0))` (capped at 70; `store:magic-level` = 5 in `constants.txt`) with `apply_magic()` but no artifacts; items with negative to-hit/to-dam/AC or curses are rejected (`store_create_random()`). Each pick is uniform over the `normal:` list (duplicates count twice).
- **Black market** stock is any object generated at level `max_depth + 5 .. max_depth + 20` (min capped at 55, max at 70); it only keeps egos, items with to-hit > 1, to-dam > 2 or AC bonus > 2, or items worth at least 10 gold that no other store stocks (`black_market_ok()`).
- **Always** items are kept at a full stack (`max-stack`, 40 by default) and buying them does not deplete the stock (`store_maint()`).
- **Turnover**: when the stores are updated, each store sells off 1d(turnover) items and then buys 1d(turnover) new ones, keeping the stock between `slots` min and max (+ always items) (`store_maint()`).
- **Restock timing**: while you are in the dungeon a day counter increases every `10 * store:turns` = 10000 game turns (`process_world()` in `src/game-world.c`); on return to town each store is maintained once per elapsed day (`store_update()`), and owners are replaced with chance 1 in `store:shuffle` (25) per update. Buying out a store's last item triggers 10 maintenance passes.
- **Pile sizes** of generated stock (`mass_produce()`): food, flasks, lights: +3d5-3 if value <= 5 and again if <= 20; potions/scrolls: +3d5-3 if <= 60, +1d5-1 if <= 240; books: +2d3-2 if <= 50, +1d3-1 if <= 500; non-ego armour/weapons: +3d5-3 if <= 10 and again if <= 100; ammo: 20-40 if value <= 5, 10-40 if <= 50, 5-20 if <= 500, else 1.
- Store inventory is capped at `store:inven-max` = 24 slots.

## General Store

- **Code**: `STORE_GENERAL`
- **Owners**: Bilbo the Friendly (Hobbit) (purse 5000); Rincewind the Chicken (Human) (purse 10000); Snafu the Midget (Gnome) (purse 20000); Lyar-el the Comely (Elf) (purse 30000)
- **Normal slots**: 0-4; **turnover**: 2
- **Buys**: light, food, mushroom, flask, digger, cloak, shot, bolt, arrow

### General Store: always in stock

| Item | Type | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---|
| Cloak | cloak | 1 | 3 | AC 1 |
| Ration of Food | food | 0 | 3 | nourish +30 (x food-value) |
| Wooden Torch | light | 1 | 2 | BURNS\_OUT, LIGHT\_2, EASY\_KNOW |
| Flask of Oil | flask | 1 | 3 | EXPLODE, THROWING |
| Shovel | digger | 1 | 10 | 1d2, DIG\_1 |
| Pick | digger | 30 | 300 | 1d3, DIG\_2 |
| Iron Shot | shot | 3 | 2 | 1d4, THROWING |
| Arrow | arrow | 3 | 1 | 1d4 |
| Bolt | bolt | 3 | 2 | 1d5 |

### General Store: normal stock list

*Entries* = how many times the item appears in the `normal:` list (each restock pick is uniform over the entries).

| Item | Type | Entries | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---:|---|
| Pint of Fine Wine | food | 1 | 0 | 2 | nourish +3 (x food-value); +79+1d41 turns of fearlessness |
| Slime Mold | food | 1 | 1 | 2 | nourish +15 (x food-value) |
| Flask of Whisky | food | 1 | 0 | 1 | nourish +3 (x food-value); +(-1+1d5) turns of confusion |
| Hard Biscuit | food | 1 | 0 | 1 | nourish +10 (x food-value); crunches |
| Stoneskin | mushroom | 1 | 0 | 60 | set stone skin to 50+7d7 turns; nourish +5 (x food-value) |
| Purging | mushroom | 1 | 0 | 60 | set food to 5 (x food-value); restore STR; restore CON; cure poisoning |

## Armoury

- **Code**: `STORE_ARMOR`
- **Owners**: Kon-Dar the Ugly (Half-Orc) (purse 5000); Darg-Low the Grim (Human) (purse 10000); Decado the Handsome (Dunadan) (purse 25000); Mauglin the Grumpy (Dwarf) (purse 30000)
- **Normal slots**: 6-18; **turnover**: 9
- **Buys**: boots, gloves, crown, helm, shield, cloak, soft armor, hard armor, dragon armor

### Armoury: normal stock list

*Entries* = how many times the item appears in the `normal:` list (each restock pick is uniform over the entries).

| Item | Type | Entries | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---:|---|
| Pair of Leather Sandals | boots | 1 | 0 | 4 | AC 1 |
| Pair of Leather Boots | boots | 1 | 3 | 7 | AC 2 |
| Pair of Iron Shod Boots | boots | 1 | 5 | 12 | AC 4, IGNORE\_FIRE |
| Hard Leather Cap | helm | 1 | 3 | 12 | AC 2 |
| Metal Cap | helm | 1 | 10 | 30 | AC 3 |
| Iron Helm | helm | 1 | 20 | 75 | AC 7 |
| Robe | soft armor | 1 | 1 | 4 | AC 2 |
| Soft Leather Armour | soft armor | 1 | 3 | 20 | AC 8 |
| Studded Leather Armour | soft armor | 1 | 10 | 100 | AC 12 |
| Hard Leather Armour | soft armor | 1 | 5 | 150 | AC 16 |
| Leather Scale Mail | soft armor | 1 | 15 | 370 | AC 20 |
| Metal Scale Mail | hard armor | 1 | 25 | 550 | AC 38 |
| Chain Mail | hard armor | 1 | 25 | 750 | AC 32 |
| Set of Leather Gloves | gloves | 1 | 0 | 5 | AC 1 |
| Set of Gauntlets | gloves | 1 | 0 | 45 | AC 3 |
| Wicker Shield | shield | 1 | 1 | 20 | AC 2 |
| Small Metal Shield | shield | 1 | 10 | 80 | AC 5 |
| Leather Shield | shield | 1 | 15 | 200 | AC 8 |

## Weapon Smiths

- **Code**: `STORE_WEAPON`
- **Owners**: Ithyl-Mak the Beastly (Half-Troll) (purse 5000); Arndal Beast-Slayer (Half-Elf) (purse 10000); Tarl Beast-Master (Hobbit) (purse 25000); Oglign Dragon-Slayer (Dwarf) (purse 30000)
- **Normal slots**: 3-14; **turnover**: 9
- **Buys**: shot, bolt, arrow, bow, digger, sword, polearm, hafted

### Weapon Smiths: normal stock list

*Entries* = how many times the item appears in the `normal:` list (each restock pick is uniform over the entries).

| Item | Type | Entries | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---:|---|
| Iron Shot | shot | 1 | 3 | 2 | 1d4, THROWING |
| Arrow | arrow | 1 | 3 | 1 | 1d4 |
| Bolt | bolt | 1 | 3 | 2 | 1d5 |
| Dagger | sword | 1 | 0 | 30 | 1d4, THROWING |
| Main Gauche | sword | 1 | 3 | 45 | 1d5 |
| Rapier | sword | 1 | 5 | 70 | 1d6 |
| Short Sword | sword | 1 | 5 | 90 | 1d7 |
| Cutlass | sword | 1 | 5 | 120 | 1d8 |
| Tulwar | sword | 1 | 5 | 170 | 2d4 |
| Broad Sword | sword | 1 | 10 | 300 | 2d5 |
| Long Sword | sword | 1 | 10 | 300 | 2d5 |
| Scimitar | sword | 1 | 10 | 300 | 4d2 |
| Katana | sword | 1 | 20 | 600 | 3d5 |
| Bastard Sword | sword | 1 | 15 | 400 | 3d4 |
| Spear | polearm | 1 | 5 | 36 | 1d6, THROWING |
| Awl-Pike | polearm | 1 | 10 | 340 | 1d8 |
| Trident | polearm | 1 | 5 | 120 | 1d10 |
| Pike | polearm | 1 | 15 | 358 | 2d5 |
| Beaked Axe | polearm | 1 | 15 | 408 | 2d6 |
| Broad Axe | polearm | 1 | 15 | 304 | 2d6 |
| Battle Axe | polearm | 1 | 15 | 334 | 2d8 |
| Lance | polearm | 1 | 10 | 230 | 2d8 |
| Lucerne Hammer | polearm | 1 | 10 | 376 | 2d5 |
| Whip | hafted | 1 | 3 | 30 | 1d3 |
| Sling | bow | 1 | 1 | 5 | x2, SHOOTS\_SHOTS |
| Short Bow | bow | 1 | 3 | 50 | x2, SHOOTS\_ARROWS |
| Long Bow | bow | 1 | 10 | 120 | x3, SHOOTS\_ARROWS |
| Light Crossbow | bow | 1 | 15 | 140 | x3, SHOOTS\_BOLTS |

## Bookseller

- **Code**: `STORE_BOOK`
- **Owners**: Ludwig the Humble (Human) (purse 15000); Gunnar the Paladin (Human) (purse 20000); Delilah the Pure (Elf) (purse 25000); Bosk the Wise (Dwarf) (purse 30000)
- **Normal slots**: 0-2; **turnover**: 0
- **Buys**: magic book, prayer book, nature book, shadow book

### Bookseller: always in stock

| Item | Type | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---|
| all town prayer books | prayer book | - | \[Novice's Handbook\] (25), \[Cleansing Power\] (100) | - |
| all town magic books | magic book | - | \[First Spells\] (25), \[Attacks and Knowledge\] (400), \[Magical Defences\] (1600) | - |
| all town nature books | nature book | - | \[Lesser Charms\] (25), \[Gifts of Nature\] (400) | - |
| all town shadow books | shadow book | - | \[Into the Shadows\] (25), \[Dark Rituals\] (400), \[Fear and Torment\] (1600) | - |

## Alchemy Shop

- **Code**: `STORE_ALCHEMY`
- **Owners**: Mauser the Chemist (Half-Elf) (purse 10000); Wizzle the Chaotic (Hobbit) (purse 10000); Ga-nat the Greedy (Gnome) (purse 15000); Vella the Slender (Human) (purse 15000)
- **Normal slots**: 6-10; **turnover**: 9
- **Buys**: scroll, potion

### Alchemy Shop: always in stock

| Item | Type | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---|
| Word of Recall | scroll | 5 | 125 | returns you from the dungeon or takes you to the dungeon after a short delay |
| Phase Door | scroll | 1 | 15 | teleport self up to 10 grids |
| Remove Curse | scroll | 10 | 100 | remove a curse (power 20+d20) |
| Cure Light Wounds | potion | 0 | 20 | heal 20 HP; cure blindness; reduce wounds by 20; reduce confusion by 20; nourish +1 (x food-value) |

### Alchemy Shop: normal stock list

*Entries* = how many times the item appears in the `normal:` list (each restock pick is uniform over the entries).

| Item | Type | Entries | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---:|---|
| Monster Confusion | scroll | 1 | 5 | 30 | +10000 turns of temporary monster confusion |
| Magic Mapping | scroll | 1 | 5 | 65 | maps the area around you (area 22 x 40) |
| Detect Invisible | scroll | 1 | 1 | 15 | detects invisible creatures nearby (area 22 x 40) |
| Recharging | scroll | 1 | 40 | 200 | recharge a wand/staff (power 6) |
| Remove Hunger | scroll | 1 | 5 | 10 | nourish up to 50 (x food-value) |
| Blessing | scroll | 1 | 1 | 15 | +6+1d12 turns of your AC and to-hit bonus |
| Holy Chant | scroll | 1 | 10 | 40 | +12+1d24 turns of your AC and to-hit bonus |
| Boldness | potion | 1 | 1 | 8 | cure fear |
| Berserk Strength | potion | 1 | 3 | 80 | heal 30 HP; cure fear; +25+1d25 turns of berserk rage |
| Resist Heat | potion | 1 | 30 | 30 | +50+1d10 turns of fire resistance |
| Resist Cold | potion | 1 | 30 | 30 | +50+1d10 turns of cold resistance |
| Restore Life Levels | potion | 1 | 40 | 400 | restores your experience |
| Neutralize Poison | potion | 1 | 5 | 75 | cure poisoning |
| Cure Serious Wounds | potion | 1 | 3 | 70 | heal 40 HP; cure blindness; cure wounds; cure confusion; nourish +1 (x food-value) |
| Cure Critical Wounds | potion | 1 | 5 | 200 | heal 60 HP; cure blindness; cure wounds; cure poisoning; cure stunning; cure amnesia; cure confusion; nourish +1 (x food-value) |

## Magic Shop

- **Code**: `STORE_MAGIC`
- **Owners**: Ariel the Sorceress (Half-Elf) (purse 15000); Buggerby the Great (Gnome) (purse 20000); Inglorian the Mage (Human) (purse 25000); Luthien Starshine (High-Elf) (purse 30000)
- **Normal slots**: 4-14; **turnover**: 9
- **Buys**: magic book, amulet, ring, staff, wand, rod

### Magic Shop: normal stock list

*Entries* = how many times the item appears in the `normal:` list (each restock pick is uniform over the entries).

| Item | Type | Entries | Lvl | Cost | Effect / stats |
|---|---|---:|---:|---:|---|
| Searching | ring | 1 | 5 | 250 | SEARCH d4M2 |
| Feather Falling | ring | 1 | 5 | 200 | FEATHER, EASY\_KNOW |
| Protection | ring | 1 | 10 | 500 | - |
| Open Wounds | ring | 1 | 5 | 700 | heal 40 HP; cure blindness; cure wounds; cure confusion |
| Slow Digestion | amulet | 1 | 15 | 200 | SLOW\_DIGEST, EASY\_KNOW |
| Resist Acid | amulet | 1 | 10 | 300 | IGNORE\_ACID, EASY\_KNOW, resist acid |
| Light | wand | 1 | 3 | 200 | line of weak light for 6d8 |
| Slow Monster | wand | 1 | 15 | 500 | bolt that attempts to slow monsters (power 10+1d20) |
| Confuse Monster | wand | 1 | 5 | 500 | bolt that attempts to confuse monsters (power 5+1d5) |
| Magic Missile | wand | 1 | 3 | 200 | bolt or beam of magic missiles for 3d4 |
| Stinking Cloud | wand | 1 | 5 | 400 | ball of poison (radius 3) for 12 |
| Stone to Mud | wand | 1 | 10 | 300 | line of stone to mud for 20+1d30 |
| Teleportation | staff | 1 | 20 | 2000 | teleport self 60% of the level's larger dimension |
| Light | staff | 1 | 5 | 250 | lights up the surrounding area; sphere of weak light around you (radius 2) for 2d8 |
| Mapping | staff | 1 | 20 | 600 | maps the area around you (area 22 x 40) |
| Detect Invisible | staff | 1 | 5 | 200 | detects invisible creatures nearby (area 22 x 40) |
| Detect Evil | staff | 1 | 20 | 350 | detects evil creatures nearby (area 22 x 40) |

## Black Market

- **Code**: `STORE_BLACK`
- **Owners**: Lo-Hak the Awful (Half-Troll) (purse 15000); Histor the Goblin (Kobold) (purse 20000); Durwin the Shifty (Half-Orc) (purse 25000); Drago the Fair (Elf) (purse 30000)
- **Normal slots**: 6-18; **turnover**: 9
- **Buys**: anything of value (no `buy:` list)

## Home

- **Code**: `HOME`
- **Owners**: Your home (purse 0); Your home (purse 0); Your home (purse 0); Your home (purse 0)

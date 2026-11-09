# 11. Town and Shopping

The town level, its residents and day/night cycle; the eight buildings, what each one always and
sometimes stocks, how stock turns over, the exact price formulas, selling and the default
`birth_no_selling` rule, the home; then shopping lists by class and phase (with measured starting
gold), money-making, and what not to buy.

FACT statements cite the game source (angband master `e473720ed`, paths relative to the source
root). Prices marked "observed" were measured by generating fresh games in a scratch build of this
exact source (store stock is random, so they are samples, not guarantees). ADVICE is judgement.

## Contents

1. [The town](#1-the-town)
2. [The stores at a glance](#2-the-stores-at-a-glance)
3. [Each store in detail](#3-each-store-in-detail)
4. [Stock turnover](#4-stock-turnover)
5. [Prices](#5-prices)
6. [Selling, giving, and birth_no_selling](#6-selling-giving-and-birth_no_selling)
7. [The home](#7-the-home)
8. [Starting gold and the first shopping trip, by class](#8-starting-gold-and-the-first-shopping-trip-by-class)
9. [Shopping lists by phase](#9-shopping-lists-by-phase)
10. [Money-making strategy](#10-money-making-strategy)
11. [What not to buy](#11-what-not-to-buy)
12. [Rules for a borg](#12-rules-for-a-borg)
13. [Things to try (untested)](#things-to-try-untested)

Related: [Chapter 6](06-equipment.md) (what equipment is worth), [Chapter 7](07-consumables-and-devices.md)
(what consumables do, carry kits), [Chapter 9](09-dungeon.md) (the town as a level),
[Chapter 12](12-game-plan-by-phase.md) (when to go back to town). Full store tables:
[A9 stores](appendices/A9-stores.md). The official manual's town and store description is in
[user-manual/dungeon.rst](user-manual/dungeon.rst) ("Townspeople", "Town Buildings").

---

## 1. The town

### 1.1 Layout

FACT (`src/gen-cave.c:town_gen_layout()`, `town_gen()`; sizes from `lib/gamedata/constants.txt`):

* The town is **66 x 22** grids (`town-wid`, `town-hgt`) inside a permanent wall.
* It is carved as a large "starburst" room out of granite, crossed by **6-8 lava streamers**
  (`3 + num_lava`, `num_lava = 3..5`). Lava burns (fire damage and destroys burnable items you
  carry, `src/player-util.c`); walk around it.
* The **down staircase** is placed near the north wall around the middle; lava is cleared next to
  it. You arrive in town on that staircase.
* A crossroads is laid out below the stairs; the eight store entrances (numbered `1`-`8`) are
  placed on lots 6x4 grids along the two streets, and the streets are cleared to floor.
* Other lots may become "ruins": granite blocks broken into rubble and passable rubble.
* The layout is generated once and then kept (the town chunk is saved and reloaded), so the stores
  never move during a game.

### 1.2 Day and night

FACT (`src/game-world.c:is_daytime()`, `src/cave-map.c:cave_illuminate()`):

* A day is `10 * day-length` = 100,000 game turns: the first half is day, the second night. At
  normal speed that is 5,000 player turns of daylight and 5,000 of night.
* By day the whole town is lit and remembered. At night only non-floor grids (walls, shop fronts)
  and the eight grids around each shop entrance glow; open floor is dark.
* Your light does not burn fuel in town during the day (`player_update_light()`), and the light
  radius is irrelevant then.

### 1.3 Town residents

FACT: `town_gen()` places 4 residents by day and 8 by night (`mon-gen:town-day` / `town-night`)
at least 3 grids from you. Town monsters are the depth-0 entries of `lib/gamedata/monster.txt`:

| Resident | Speed | HP | Attacks | Notes |
|----------|-------|----|---------|-------|
| filthy street urchin | +0 | 3 | beg, touch to steal gold | erratic; takes items |
| scrawny cat, scruffy little dog | +0 | 2 | 1d1 | harmless |
| blubbering idiot, boil-covered wretch, village idiot (+10 speed) | | 2-10 | drool | harmless |
| pitiful-looking beggar, mangy-looking leper | +0 | 1-3 | beg | harmless |
| singing, happy drunk / aimless-looking merchant | +0 | 4-6 | beg / 1d3 | drop gold (40%) |
| squint-eyed rogue | +0 | 9 | 1d6, **touch to steal an item**, insult | evil; can steal from your pack |
| mean-looking mercenary | +0 | 23 | 1d10 | evil; drops 40% |
| battle-scarred veteran | +0 | 32 | 2d6 | can kill a CL1 caster; drops 40% |
| Farmer Maggot (unique) | +0 | 350 | moans only | never hurts you; **drops one good item** (DROP_GOOD, ONLY_ITEM) |
| red-hatted elf, Father Christmas | | | | SEASONAL: only on 24-26 December (`src/mon-make.c`) |

Town monsters give no experience (manual; Maggot has `experience:0`).

ADVICE:

* At CL1, avoid the battle-scarred veteran and mean-looking mercenary (they hit hard for your HP);
  kill urchins and rogues on sight or keep away from them (theft).
* Farmer Maggot is worth killing at any level: he never fights back and his single drop is forced
  "good" (`src/mon-make.c:mon_create_drop()`: drop level = his level 0, and a good drop picks the
  kind at level +10 with to-hit/dam/AC bonuses). No artifacts can be made in town. It takes many
  turns at CL1 (350 HP).
* Shop in daytime when possible: you can see everything and fewer residents spawn.

---

## 2. The stores at a glance

FACT (`lib/gamedata/store.txt`, `lib/gamedata/terrain.txt`): eight buildings, entered by walking
onto the number.

| # | Store | Owners (max purse when buying from you) | Stock slots (normal min:max) | Turnover | Always stocks |
|---|-------|-----------------------------------------|------------------------------|----------|---------------|
| 1 | General Store | Bilbo the Friendly 5000, Rincewind the Chicken 10000, Snafu the Midget 20000, Lyar-el the Comely 30000 | 0:4 | 2 | Cloak, Ration of Food, Wooden Torch, Flask of Oil, Shovel, Pick, Iron Shot, Arrow, Bolt |
| 2 | Armoury | Kon-Dar the Ugly 5000, Darg-Low the Grim 10000, Decado the Handsome 25000, Mauglin the Grumpy 30000 | 6:18 | 9 | - |
| 3 | Weapon Smiths | Ithyl-Mak the Beastly 5000, Arndal Beast-Slayer 10000, Tarl Beast-Master 25000, Oglign Dragon-Slayer 30000 | 3:14 | 9 | - |
| 4 | Bookseller | Ludwig the Humble 15000, Gunnar the Paladin 20000, Delilah the Pure 25000, Bosk the Wise 30000 | 0:2 | 0 | every town book of all four realms |
| 5 | Alchemy Shop | Mauser the Chemist 10000, Wizzle the Chaotic 10000, Ga-nat the Greedy 15000, Vella the Slender 15000 | 6:10 | 9 | Word of Recall, Phase Door, Remove Curse scrolls; Cure Light Wounds potions |
| 6 | Magic Shop | Ariel the Sorceress 15000, Buggerby the Great 20000, Inglorian the Mage 25000, Luthien Starshine 30000 | 4:14 | 9 | - |
| 7 | Black Market | Lo-Hak the Awful 15000, Histor the Goblin 20000, Durwin the Shifty 25000, Drago the Fair 30000 | 6:18 | 9 | - |
| 8 | Home | you | 24 stacks | - | - |

Each store holds at most 24 stacks (`store:inven-max`). "Always" items are kept at a full stack
of 40 (`src/store.c:store_maint()`).

---

## 3. Each store in detail

Prices below are normal-store selling prices: kind cost for consumables
(`lib/gamedata/object.txt`), and for wearables the power-based value (5.1), observed in sample
games.

### 3.1 General Store (1)

FACT: always Cloak, Ration of Food, Wooden Torch, Flask of Oil, Shovel, Pick, Iron Shot, Arrow,
Bolt; sometimes Pint of Fine Wine, Slime Mold, Flask of Whisky, Hard Biscuit, Mushroom of
Stoneskin, Mushroom of Purging. Buys: lights, food, mushrooms, flasks, diggers, cloaks, shots,
bolts, arrows.

Observed prices: Ration 3, Flask of oil 3, Wooden Torch 6 (each, with 5000 turns of fuel),
Cloak 6, Arrow/Bolt 1, Iron Shot 3, Shovel 66, Pick 176, Stoneskin/Purging mushrooms 60.

ADVICE: Flasks of oil are the most important early purchase (thrown weapon and lantern fuel).
There is **no lantern** here; the General Store's only light source is the torch.

### 3.2 Armoury (2)

FACT: sometimes stocks Leather Sandals/Boots, Iron Shod Boots, Hard Leather Cap, Metal Cap, Iron
Helm, Robe, Soft/Studded/Hard Leather Armour, Leather Scale Mail, Metal Scale Mail, Chain Mail,
Leather Gloves, Gauntlets, Wicker/Small Metal/Leather Shield. Buys boots, gloves, crowns, helms,
shields, cloaks, soft/hard/dragon armour.

Items are generated with magic at a level `rand_range(1, 5 + max(max_depth - 20, 0))`
(`store_create_random()`, `store:magic-level:5`), so egos and higher enchantments become more
common once your max depth passes DL20. Damaged (negative) and cursed items are never stocked.

Observed prices: Leather Sandals 6, Leather Boots 14, Leather Gloves 6, Gauntlets 24, Hard
Leather Cap 14, Metal Cap 24, Iron Helm 104, Robe 14, Soft Leather Armour 84, Hard Leather 336,
Leather Scale Mail 546, Chain Mail 1184, Metal Scale Mail 1886, Leather Shield 66, Small Metal
Shield 36; enchanted versions cost more (Gauntlets [3,+7] 374); egos: Leather Gloves of Free Action
2750, Leather Shield of Resist Cold [8,+13] 1400, Iron Helm of Wisdom 1254.

### 3.3 Weapon Smiths (3)

FACT: sometimes stocks Iron Shot, Arrow, Bolt, Dagger, Main Gauche, Rapier, Short Sword, Cutlass,
Tulwar, Broad Sword, Long Sword, Scimitar, Katana, Bastard Sword, Spear, Awl-Pike, Trident,
Pike, Beaked Axe, Broad Axe, Battle Axe, Lance, Lucerne Hammer, Whip, Sling, Short Bow, Long Bow,
Light Crossbow. Buys shots, bolts, arrows, bows, diggers, swords, polearms, hafted.

Observed prices: Dagger 150, Main Gauche 84, Rapier 104, Cutlass 176, Tulwar 204, Spear 204,
Broad/Long Sword 300, Scimitar 300, Bastard Sword 414, Katana 594, Battle Axe 594, Sling 150,
Short Bow 204, Long Bow 414, Light Crossbow 546. Enchanted and ego versions: Long Bow (+6,+1) 924,
Sling of Power (+5,+17) 1254, Short Bow of Accuracy 1886, Broad Sword of *Slay Undead* 2750,
Arrows of Slay Animal ~40 each.

Warning (observed): the Weaponsmith can stock a **Short Bow of the Nazgûl** (an ego with
DRAIN_EXP; it is not a "curse", so the no-curse filter lets it through). Check `{??}` items' egos
before buying.

No hafted weapon except the Whip is a store item, and no Mace: priests buy blessed weapons from the
dungeon or use the starting Mace.

### 3.4 Bookseller (4)

FACT: always stocks every **town** book (`src/store.c:parse_always()` adds all non-dungeon books
for each listed book tval); turnover 0 but each update it may "sell" a random number of books
before restocking the staples. Buys magic, prayer, nature and shadow books. Book prices from
`book-properties` in `lib/gamedata/class.txt`:

| Realm | Book (town) | Price | Used by |
|-------|-------------|-------|---------|
| Arcane | [First Spells] | 25 | Mage, Rogue |
| Arcane | [Attacks and Knowledge] | 400 | Mage |
| Arcane | [Magical Defences] | 1600 | Mage |
| Nature | [Lesser Charms] | 25 | Druid, Ranger |
| Nature | [Gifts of Nature] | 400 | Druid |
| Divine | [Novice's Handbook] | 25 | Priest, Paladin |
| Divine | [Cleansing Power] | 100 | Priest |
| Shadow | [Into the Shadows] | 25 | Necromancer, Blackguard |
| Shadow | [Dark Rituals] | 400 | Necromancer |
| Shadow | [Fear and Torment] | 1600 | Necromancer, Blackguard |

Which classes can use which book is set per class in `lib/gamedata/class.txt`; see
[Chapter 3](03-magic-and-spells.md). Dungeon books cost 10,000-50,000 and are never stocked
except by the Black Market.

### 3.5 Alchemy Shop (5)

FACT: always Word of Recall (125), Phase Door (15), Remove Curse (100) scrolls and Cure Light
Wounds potions (20). Sometimes: scrolls of Monster Confusion (30), Magic Mapping (65), Detect
Invisible (15), Recharging (200), Remove Hunger (10), Blessing (15), Holy Chant (40); potions of
Boldness (8), Berserk Strength (80), Resist Heat (30), Resist Cold (30), Restore Life Levels (400),
Neutralize Poison (75), Cure Serious Wounds (70), Cure Critical Wounds (200). Buys any scroll or
potion.

**Not sold by the Alchemist in this version:** Scrolls of Teleportation, Teleport Level, Deep
Descent, Identify Rune, enchantment scrolls; Potions of Speed, Heroism, Healing, Restore Mana.
They come from the dungeon or the Black Market.

### 3.6 Magic Shop (6)

FACT: sometimes rings of Searching, Feather Falling, Protection, Open Wounds; amulets of Slow
Digestion, Resist Acid; wands of Light, Slow Monster, Confuse Monster, Magic Missile, Stinking
Cloud, Stone to Mud; staves of Teleportation, Light, Mapping, Detect Invisible, Detect Evil. Buys
magic books, amulets, rings, staves, wands, rods.

Observed prices (wands/staves vary with charges: `cost + cost * charges / 20`,
`src/obj-power.c:object_value_real()`): Staff of Teleportation ~2700-2900, Staff of Mapping
~810-900, Staff of Detect Evil ~630-750, Staff of Detect Invisible ~330-410, Staff of Light
~500-600, Wand of Magic Missile ~310-360, Wand of Stinking Cloud ~600-660, Wand of Stone to Mud
~435-540, Wand of Slow/Confuse Monster ~625-700, Wand of Light ~310-360, Ring of Protection [+10]
266 ([+12] 336), Ring of Open Wounds 546, Ring of Feather Falling 50, Ring of Searching 66-150,
Amulet of Resist Acid 204, Amulet of Slow Digestion 66.

The Magic Shop is the **only reliable source of a teleport device** (Staff of Teleportation).

### 3.7 Black Market (7)

FACT (`src/store.c:store_create_random()`, `black_market_ok()`):

* Items are any kind (`get_obj_num()`), generated at a level between `max_depth + 5` and
  `max_depth + 20` (min capped at 55, max at 70), never artifacts, never cursed or damaged.
* An item is kept only if it is an ego, or has to-AC > 2, to-hit > 1 or to-dam > 2, or (if
  plain) is worth at least 10 gold **and no other store currently stocks that kind**. Plain items
  that start being stocked elsewhere are thrown out at the next maintenance.
* Prices are about **3x** the normal price (5.2). Observed: Potion of Speed 225, Potion of Healing
  2400, Scroll of Teleportation 120, Lantern 1638, Lantern (Everburning) 2418, Rod of Curing 3000,
  Rod of Treasure Location 3000, Gauntlets of Free Action 7332.

ADVICE: the BM is where to buy Potions of Speed and Healing, Scrolls of Teleportation, a Lantern
before you find one, and the occasional great ego. It is stocked relative to your *max depth*, so
it gets better as you dive.

### 3.8 Home (8)

See 7.

---

## 4. Stock turnover

FACT (`src/game-world.c`, `src/store.c:store_update()`, `store_maint()`):

* While you are **in the dungeon**, every `10 * store:turns` = 10,000 game turns (1,000 player
  turns at normal speed) increments a day counter. Nothing changes while you are in town, even
  across save/load ([user-manual/dungeon.rst](user-manual/dungeon.rst)).
* On returning to town, each store (not the home) runs `store_maint()` once per counted day:
  1. Black Market: discard items that no longer pass `black_market_ok()`.
  2. If turnover > 0: delete random items down to `stock_num - randint1(turnover)` (bounded by the
     normal max). Deleting a stack removes all, half or one of it (ammo: whole stack or a
     multiple of 5).
  3. Re-create any missing "always" items and refill them to a stack of 40.
  4. If turnover > 0: add random items up to `stock_num + randint1(turnover)`, kept between
     `normal_min + always_num` and `normal_max + always_num`.
  5. Bookseller (turnover 0): sell a random number of books, then restock the staples.
* With chance 1 in 25 per counted day (`store:shuffle:25`) one random store changes owner
  (different purse limit).
* At game start each store runs `store_maint()` 10 times (`store_reset()`).

FACT, two more rules from `src/store.c:do_cmd_buy()`:

* **Staples are unlimited.** Buying a plain "always" item (Phase Door, CLW, Word of Recall,
  Remove Curse, flasks, torches, rations, cloaks, shovels, picks, basic ammo, town books) does not
  reduce the store's stock (`store_sale_should_reduce_stock()` is false for plain staples), so the
  Alchemist, General Store and Bookseller never run out of them.
* **Buying a store out restocks it at once.** When a purchase leaves a store with zero items, the
  owner "brings out some new stock": `store_maint()` runs 10 times immediately, and with chance
  1 in 25 the owner retires (new owner). Stores with staples can never be emptied; the Armoury,
  Weaponsmith, Magic Shop and Black Market can.

ADVICE: a quick dive (DL1-2 and back) of ~1,000 turns gives one restock. To fish for a specific
item (e.g. Potions of CCW, a Staff of Teleportation), go down, spend 1,000+ turns, recall up. If
the Magic Shop holds only a few cheap items, buying all of them forces a full 10-pass restock on
the spot, which can be cheaper than a dungeon trip.

---

## 5. Prices

### 5.1 The value of an item

FACT (`src/obj-power.c:object_value_real()`, `object_value()`):

* **Consumables and devices** (not wearable, not ammo): value = kind `cost`. Wands and staves add
  `cost * charges / 20` (charges rounded up per item). Worthless kinds (cost 0) are worth 0.
* **Wearables and ammunition**: value = `power * (power + 5)`, where `power = object_power()`
  sums the item's properties (dice, bonuses, modifiers, flags, resists, slays, brands,
  activation). Torches (BURNS_OUT, non-ego) and ammo are divided by a rescaler, and a value of 0 is
  rounded up to 1.
* The store uses the *real* value when selling to you (it knows everything); when buying from you
  it uses the smaller of real and apparent value (`price_item()`).

### 5.2 Store selling price (you buy)

FACT (`src/store.c:price_item()` with `store_buying = false`):

```
price = object_value_real(item, 1)          (whole stack value for wands/staves)
if Black Market: price = price * 2
price = (price * adjust + 50) / 100         adjust = 100 normally, 150 in the Black Market
total = price * quantity                     (wands/staves: already a stack price)
```

So normal stores sell at **1.0x value** and the Black Market at **3.0x value** (2 x 150%). The
source comment says "twice as much", but the code applies both factors; observed prices confirm 3x
(Potion of Speed 75 -> 225, Healing 800 -> 2400).

There is no haggling, no charisma, no race or owner price modifier in 4.2: all owners of a store
charge the same.

### 5.3 Store buying price (you sell; only with `birth_no_selling` off)

```
price = min(real value, apparent value)
price = price * 2 / 3
if Black Market: price = price / 2, then * 50 / 100     -> 1/6 of value
price = min(price, owner max_cost * quantity)            (the purse limit per item)
```

Normal stores pay **2/3 of value**, the Black Market **1/6**, both capped at the owner's purse
(5,000-30,000 gold per item, table in 2). With `birth_no_selling` on (the default) the store pays
0.

### 5.4 Quick price reference

FACT: normal-store prices of the consumables the town sells (kind `cost` in
`lib/gamedata/object.txt`); "BM" is the Black Market price (3x) for the same item, and also the
price of the many consumables that *only* the Black Market can stock.

| Item | Store | Normal price | BM price |
|------|-------|--------------|----------|
| Potion of Cure Light Wounds | Alchemist (always) | 20 | 60 |
| Potion of Cure Serious Wounds | Alchemist | 70 | 210 |
| Potion of Cure Critical Wounds | Alchemist | 200 | 600 |
| Potion of Neutralize Poison | Alchemist | 75 | 225 |
| Potion of Berserk Strength | Alchemist | 80 | 240 |
| Potion of Boldness | Alchemist | 8 | 24 |
| Potion of Resist Heat / Resist Cold | Alchemist | 30 | 90 |
| Potion of Restore Life Levels | Alchemist | 400 | 1200 |
| Scroll of Phase Door | Alchemist (always) | 15 | 45 |
| Scroll of Word of Recall | Alchemist (always) | 125 | 375 |
| Scroll of Remove Curse | Alchemist (always) | 100 | 300 |
| Scroll of Magic Mapping | Alchemist | 65 | 195 |
| Scroll of Remove Hunger | Alchemist | 10 | 30 |
| Scroll of Recharging | Alchemist | 200 | 600 |
| Scroll of Blessing / Holy Chant | Alchemist | 15 / 40 | 45 / 120 |
| Scroll of Monster Confusion | Alchemist | 30 | 90 |
| Scroll of Detect Invisible | Alchemist | 15 | 45 |
| Flask of oil | General (always) | 3 | - |
| Ration of Food | General (always) | 3 | - |
| Wooden Torch | General (always) | 6 (with fuel) | - |
| Potion of Speed | BM only | (75) | 225 |
| Potion of Heroism | BM only | (25) | 75 |
| Potion of True Seeing | BM only | (50) | 150 |
| Potion of Healing | BM only | (800) | 2400 |
| Potion of *Healing* | BM only | (1500) | 4500 |
| Potion of Life | BM only | (5000) | 15000 |
| Potion of Restore Mana | BM only | (350) | 1050 |
| Potion of Resist Poison | BM only | (30) | 90 |
| Scroll of Teleportation | BM only | (40) | 120 |
| Scroll of Teleport Level | BM only | (800) | 2400 |
| Scroll of Deep Descent | BM only | (250) | 750 |
| Scroll of Identify Rune | BM only | (200) | 600 |
| Scroll of Enchant Weapon To-Hit / To-Dam / Armour | BM only | (125) | 375 |
| Scroll of Banishment | BM only | (2500) | 7500 |
| Scroll of *Destruction* | BM only | (1000) | 3000 |

Black Market availability depends on the item's allocation depth versus the BM generation level
(`max_depth+5` to `max_depth+20`, capped 55/70) and on the kind not being stocked elsewhere, so a
Potion of Healing (alloc DL30+) cannot appear there until your max depth is roughly 10+, and
*Healing* (DL40+) roughly 20+ (generation level must reach the kind's minimum allocation depth,
`get_obj_num()`; out-of-depth boosts make earlier appearances possible but rare).

### 5.5 Is the Black Market price worth it?

ADVICE, as a rule of thumb for a character with income from 5x gold:

| Item | Buy at BM when |
|------|----------------|
| Potion of Speed (225) | always, if you have fewer than ~5 and can still afford the next trip's basics |
| Scroll of Teleportation (120) | until you own a Staff of Teleportation, keep 3-5 |
| Potion of Healing (2400) | from DL30 on, keep 3-5 |
| Restore Mana (1050) | casters from DL30, before unique fights |
| Lantern (~1600) | only if you will be in DL1-5 a long time; they are common on the floor |
| Ego armour with a resist you lack | yes, if it fills a "must have" from [Chapter 6](06-equipment.md#17-what-to-wear-at-each-stage) |
| Anything the Alchemist also sells | no (3x) |
| Enchant scrolls (375) | late, for your endgame weapon only |

---

## 6. Selling, giving, and birth_no_selling

FACT:

* `birth_no_selling` is **on by default** (`src/list-options.h`: `OP(birth_no_selling, ...,
  BIRTH, true)`). Stores pay nothing for items ("Give which item?"), but in exchange gold found in
  the dungeon is **multiplied by 5** (`src/obj-make.c:make_gold()`, only when `player->depth > 0`).
* Gold piles: average value `16 * level / 10 + 16` (16 at DL0, 80 at DL40, 176 at DL100), spread
  `level + 10`, with a 1% chance per step of x10 (`make_gold()`), then x5 with no_selling, capped
  at 32767.
* **Selling or giving identifies.** The store learns the flavour and all unknown runes of the item
  before taking it (`src/store.c:do_cmd_sell()`), and you learn them too (runes become known for
  all items). With no_selling the store even accepts unknown `{??}` items of variable power that
  look worthless, so you can learn their runes ([user-manual/option.rst](user-manual/option.rst)).
* A store only accepts kinds on its "buy" list; the home takes anything. Known-worthless items
  (value 0, e.g. cursed or bad potions) are refused.
* The store may delete what you sold at the next turnover; with no_selling you can only buy it back
  at full price.
* **Buying also identifies**: all unknown runes on a bought item are learned
  (`src/store.c:do_cmd_buy()`).
* Selling an empty wand/staff of a kind the store stocks re-rolls its charges (store recharge,
  [Chapter 7](07-consumables-and-devices.md#15-charges-recharging-rod-timeouts)).

ADVICE: with no_selling, *never carry loot back to sell*; carry back only items to identify (if
the rune knowledge is worth losing the item) or to store at home. Pick up gold always: it is 5x.
With selling enabled, sell egos and artifacts you don't need to the store whose owner has the
largest purse, and sell to normal stores, never to the BM.

---

## 7. The home

FACT: the Home (8) holds **24 stacks** (`store:inven-max`), stacking like your pack (up to 40);
it never loses items, never restocks, and is free. Use `d`/`g` (drop/pick up) or the store
commands inside. The home's contents count in the equipment comparison screen
([Chapter 6](06-equipment.md#19-comparing-two-items-and-a-scoring-function-for-a-borg)) and in the
character dump's home list.

ADVICE (what to keep at home):

* Swap items you don't carry: resist rings/amulets for specific uniques, a slay weapon, a digger,
  a Lantern of Shadows, spare light.
* Healing reserve: extra *Healing*/Life/Speed potions and spare Restore Mana, so a fire/cold
  breath can't destroy your whole stock.
* Spare escape and recall: 1-2 Word of Recall in case you return with none (you can always buy
  one in the Alchemist, but not if you are broke).
* Books: a spare copy of each spellbook you rely on (fire destroys books).
* Unknown items to test later in safety.
* Do not hoard junk: with no_selling it has no value; 24 stacks fill fast.

---

## 8. Starting gold and the first shopping trip, by class

### 8.1 Starting gold

FACT (`src/player-birth.c`): starting gold is `start-gold` (600, `lib/gamedata/constants.txt`)
`+ 50 * unspent point-buy points`, minus the value of the starting kit when `birth_start_kit` is
on (default). Without the kit you start with only 1 food and 1 light and keep ~595 gold.

Observed with the default point-buy (all points spent), Human characters, kit on (food and torch
counts are random, so +-10 gold):

| Class | Gold left | Kit (value) |
|-------|-----------|-------------|
| Warrior | ~134-146 | Word of Recall (125), Soft Leather Armour (84), Dagger (150), Potion of Berserk Strength (80), 1-3 torches (6 each), 1-3 rations (3 each) |
| Mage | ~322 | [First Spells] (25), Word of Recall (125), Rapier (104), torches, rations |
| Druid | ~373 | [Lesser Charms] (25), Word of Recall, Whip (50), torches, rations |
| Priest | ~91 | [Novice's Handbook] (25), Word of Recall, 2 Potions of CSW (140), Mace (204), torches, ration |
| Necromancer | ~357 | [Into the Shadows] (25), Word of Recall, Main Gauche (84), rations (no light: unlight) |
| Paladin | ~304 | [Novice's Handbook], Word of Recall, Scroll of Protection from Evil (50), Main Gauche (84), torch, rations |
| Rogue | ~201 | [First Spells], Word of Recall, Soft Leather Armour (84), Dagger (150), torches, ration |
| Ranger | ~137 | [Lesser Charms], Word of Recall, 15-20 Arrows (~1 each), Short Bow (204), Main Gauche (84), torch, ration |
| Blackguard | ~151 | [Into the Shadows], Word of Recall, Potion of CLW (20), Leather Shield (66), Tulwar (204), torch, ration |

Each unspent birth point is worth 50 gold; leaving 2 points unspent buys a full early consumable
kit. Race does not change gold.

### 8.2 First shopping trip (ADVICE)

Priorities for the very first trip, in order: escapes (Phase Door), damage (Flasks of oil for
non-archers, arrows/shots for archers), healing (CLW), then armour pieces.

| Class | Suggested CL1 purchases (approx. cost) |
|-------|----------------------------------------|
| Warrior (~140) | Flasks of oil x15 (45), Phase Door x3 (45), CLW x2 (40). Later: Sling (150) |
| Mage (~320) | Phase Door x5 (75), Flasks of oil x15 (45), CLW x3 (60), Soft Leather Armour (84) or Leather Boots/Cap/Gloves (~34), keep ~50 |
| Druid (~370) | Phase Door x5 (75), Flasks x15 (45), CLW x4 (80), Soft Leather Armour (84), Leather Boots + Cap (28) |
| Priest (~90) | Phase Door x3 (45), Flasks x10 (30), keep the 2 CSW; CLW x1 if possible |
| Necromancer (~355) | Phase Door x5 (75), CLW x4 (80), Flasks x15 (45), Soft Leather Armour (84), Leather Gloves/Boots (20); no light needed |
| Paladin (~300) | Phase Door x5 (75), CLW x4 (80), Flasks x10 (30), Soft Leather Armour (84) |
| Rogue (~200) | Phase Door x4 (60), Flasks x15 (45), CLW x3 (60), Sling (150) later |
| Ranger (~137) | Arrows x40 (40), Phase Door x3 (45), CLW x2 (40) |
| Blackguard (~150) | Phase Door x3 (45), Flasks x10 (30), CLW x3 (60), Leather Boots (14) |

Spellcasters: the first book is in the kit; the second town book costs 100-400, so plan the
second trip around it ([Chapter 3](03-magic-and-spells.md) says when its spells unlock).

---

## 9. Shopping lists by phase

ADVICE (prices from 3; what each item does is in [Chapter 7](07-consumables-and-devices.md)).

### 9.1 First trip back (CL5-15, after DL3-10)

With no_selling, expect roughly a few hundred to a few thousand gold from 5x gold drops.

1. Restock: Phase Door to 10, CLW to 5, Flasks of oil to 15-20, Word of Recall to 1 (2 if rich).
2. Potions of Cure Serious Wounds x3-5 (70 each) when the Alchemist has them.
3. Armour gaps: Leather Boots, Hard Leather Cap, Leather Gloves, Leather/Small Metal Shield,
   Soft/Hard Leather Armour (casters watch the weight limit, [Chapter 6](06-equipment.md#23-the-caster-armour-weight-limit-spell-weight)).
4. Casters: second town book (100-400).
5. Device users: Wand of Magic Missile (~340) or Stinking Cloud (~640).
6. A Lantern if the BM has one under ~1700 and you can spare it; otherwise wait for a dungeon one.

### 9.2 Mid game (CL15-30, DL10-30)

1. **Staff of Teleportation** (~2800, Magic Shop): the single most important purchase of the
   mid-game; check the Magic Shop every visit until you have one.
2. Potions of Cure Critical Wounds x5-10 (200 each) whenever stocked.
3. Free Action if you have none by DL20: Gloves of Free Action in the Armoury (observed 2750) or
   BM; See Invisible source (Helm of Seeing, BM).
4. Scrolls of Magic Mapping x5 (65), Staff of Detect Evil (~700) for detection.
5. Wand of Stone to Mud (~500): digging, and kills wall monsters.
6. Scroll of Recharging (200) for the teleport staff and attack wands.
7. Black Market: Potions of Speed (225) every time you can afford them; Scrolls of
   Teleportation (120) as backup.
8. Third town book (Mage/Necromancer: 1600).
9. Potions of Berserk Strength (80) for melee classes.

### 9.3 Late game (CL30-45, DL30-60)

1. CCW stock to 10-15 every trip.
2. Black Market: Potions of Healing (2400), Speed (225), Restore Mana (1050 observed), any ego with
   a resist you lack. Check it after every dive: its level follows your max depth.
3. Enchant scrolls when the BM has them, for weapon to-dam and armour.
4. Potions of Resist Heat/Cold (30) for fire/cold-heavy fights if you lack double resists.
5. Spare Staff of Teleportation (keep one at home in case of fire/acid loss).

### 9.4 Endgame

Stores matter little: the Alchemist still supplies CCW, Phase Door and Recall; the Black Market
is the only store with Healing/*Healing*/Life/Speed. Most supplies come from the dungeon. Keep a
reserve at home ([7](#7-the-home)).

---

## 10. Money-making strategy

FACT-based ADVICE:

* **Default game (no_selling):** income is dungeon gold only, at 5x. Gold piles average
  `1.6 * level + 16` before the multiplier, so deeper levels pay much more per pile, and DL20+
  piles average ~250 gold after x5. Treasure veins (magma/quartz with treasure) and monsters with
  ONLY_GOLD/DROP flags add more. Rods of Treasure Location and scrolls of Treasure Detection find
  gold and ore; Dwarves see ore (`SEE_ORE`).
* Diving is the best money strategy: gold scales with depth, and the BM's stock improves with max
  depth.
* Kill Farmer Maggot early for a free good item.
* With selling on: sell egos/artifacts to the normal store with the highest-purse owner (2/3 of
  value up to 30,000 per item). Never sell to the BM (1/6). Wands and staves sell per-charge.
* Don't buy what you will find anyway at your depth (CLW, Phase Door are common floor items).

---

## 11. What not to buy

ADVICE (with reasons from the item data):

| Item | Why not |
|------|---------|
| Potions of Boldness (8) | only cures fear; Berserk Strength (80) also heals 30 and grants immunity, CCW cures more |
| Potions of Resist Heat/Cold before DL25 | few early threats need them; buy when facing fire/cold breathers |
| Scroll of Detect Invisible, Staff of Detect Invisible | narrow; Detect Evil covers most invisible threats |
| Wand of Light, Staff of Light | situational (light-sensitive orcs); flasks or Magic Missile do more |
| Ring of Searching, Ring of Feather Falling, Amulet of Slow Digestion | weak slot use; buy only as cheap AC-less fillers |
| Ring of Open Wounds as a main ring | IMPAIR_HP; useful only as a swap |
| Heavy armour for casters | each pound over the class limit costs 1 SP ([Chapter 6](06-equipment.md)) |
| Weapons heavier than your STR can hold | 1 blow and a to-hit penalty |
| Expensive plain weapons/armour with small bonuses | a few hundred gold of CCW or a teleport staff saves more lives |
| Black Market consumables that the Alchemist sells | 3x price (e.g. Neutralize Poison 225 vs 75) |
| `{??}` launchers of the Nazgûl, weapons of Morgul | experience drain |
| Store weapons for the "average/bad" quality you already ignore | money wasted |
| Lanterns at BM 3x price when you are about to dive | DL5+ floors have lanterns as often as torches (alloc 70) |

---

## 12. Rules for a borg

FACT-based rules a borg can implement directly:

1. On entering town, visit in this order: Alchemist (5) -> General (1) -> Magic (6) -> Black
   Market (7) -> Armoury (2) -> Weaponsmith (3) -> Home (8); skip a store if nothing on its
   shopping list can be there (use the store's always/normal lists from `lib/gamedata/store.txt`).
2. Teleport source: if no teleport device and gold >= price of a Staff of Teleportation in store 6,
   buy it before anything else except one Word of Recall and 5 CLW/CSW.
3. Never "sell" (give) with no_selling unless the item is wanted only for its rune knowledge;
   learning a rune on a common item type (e.g. a resist) has value, learning on a one-off artifact
   little.
4. Treat BM prices as 3x value; buy there only Speed, Healing/*Healing*/Life, Restore Mana,
   Teleportation/Teleport Level/Deep Descent scrolls, a lantern, or egos that fill a missing
   protection.
5. To restock a missing item, spend >= 1,000 player turns in the dungeon (one day) per restock
   attempt, or, if the store (2, 3, 6 or 7) holds only cheap items, buy it out to trigger an
   immediate 10-pass restock.
6. Keep gold in reserve for the next trip's Word of Recall + 5 CLW (~225 gold) at all times.
7. Use buying to learn runes when an affordable store ego shows `{??}` with a rune that matters
   (e.g. FA, SI, resists): the knowledge applies to every later item.

---

## Things to try (untested)

1. **Point-buy for gold**: leave 2-4 birth points unspent (+100-200 gold) and buy a stronger first
   kit; compare CL10 survival with a fully spent point-buy.
2. **No-kit start**: turn `birth_start_kit` off (~595 gold) and buy a custom kit (e.g. a Mage
   buying Magic Missile wand + Phase Door instead of the Rapier); measure early death rate.
3. **Restock fishing**: compare two ways of rolling the Magic Shop for a Staff of Teleportation:
   dungeon days (1,000 turns each) vs buying the shop out when its total price is low (immediate
   10-pass restock). Measure gold and turns per staff found.
4. **Maggot first**: kill Farmer Maggot at CL1 with flasks of oil and track what the "good" drop
   is over many games (distribution of item types).
5. **Rune-learning by purchase**: each trip, buy the cheapest store ego with an unknown rune; count
   how early all base resist runes are known compared to normal play.
6. **BM after max-depth jumps**: since BM item level is `max_depth+5..+20`, a Deep Descent dive to
   a new max depth followed by an immediate recall might upgrade BM stock; test BM item quality
   before and after.
7. **Home as fire insurance**: keep half of all Healing/Speed potions at home and compare total
   potion losses to elements over a game.
8. **Owner purse targeting (selling games)**: track which owner a store has and sell high-value
   items only when the 30,000-purse owner is present; measure income vs selling immediately.
9. **Night shopping risk**: measure how often a CL1-3 character gets hurt by veterans/mercenaries
   when shopping at night (8 residents) vs day (4).

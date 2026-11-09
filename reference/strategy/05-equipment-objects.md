# Chapter 5 — Equipment and Objects

*Angband 4.2.6. Every number in this chapter is taken from `lib/gamedata/*.txt`
or from the C source in `src/`; the file or function is cited so you can check
it. The complete row-by-row listings are in the generated appendices:
[A4 Artifacts](A4-artifacts.md), [A5 Ego Items](A5-ego-items.md) and
[A6 Object Kinds](A6-objects.md). This chapter is about **what those rows
mean for your character** and **what to pick up, wear, carry and buy**.*

## Contents

1. [Equipment slots, weight and encumbrance](#1-equipment-slots-weight-and-encumbrance)
2. [Evaluating gear and the rune identification system](#2-evaluating-gear-and-the-rune-identification-system)
3. [Melee weapons](#3-melee-weapons)
4. [Launchers, ammunition and throwing](#4-launchers-ammunition-and-throwing)
5. [Armour, slot by slot](#5-armour-slot-by-slot)
6. [Rings and amulets](#6-rings-and-amulets)
7. [Light sources](#7-light-sources)
8. [Dragon scale mail, activations and device skill](#8-dragon-scale-mail-activations-and-device-skill)
9. [Consumables: potions, scrolls, mushrooms, food](#9-consumables-potions-scrolls-mushrooms-food)
10. [Magic devices: wands, staffs, rods](#10-magic-devices-wands-staffs-rods)
11. [The stores](#11-the-stores)
12. [Artifacts](#12-artifacts)
13. [How items are generated: good, great, ego and artifact rolls](#13-how-items-are-generated-good-great-ego-and-artifact-rolls)
14. [Swap gear and the ideal kit by stage](#14-swap-gear-and-the-ideal-kit-by-stage)

### Notation used in this chapter

* **Weights** are given in pounds. The data files store tenths of a pound
  (`weight:120` is 12 lb).
* **Dice** `XdY` average `X*(Y+1)/2`. A `+M` term (e.g. `4+M6`) is a
  level-dependent bonus: its mean is `M*level/128` and it never exceeds `M`
  (see the header comment of `lib/gamedata/object.txt`).
* **alloc `C: A to B`** is commonness `C` between dungeon levels `A` and `B`.
  An item can still appear shallower than `A` through the "level boost"
  described in [§13](#13-how-items-are-generated-good-great-ego-and-artifact-rolls).
* **Resistance levels**: `RES_X[1]` is a resistance, `RES_X[3]` is an
  **immunity**, `RES_X[-1]` a vulnerability (`src/project-player.c:adjust_dam()`).
  A resisted base element (acid, lightning, fire, cold, poison) does 1/3
  damage, and a second layer from a temporary resistance (potion, spell)
  makes it 1/9. High elements (light, dark, sound, shards, nexus, nether,
  chaos, disenchantment) are multiplied by `6/(8+1d4)` when resisted
  (`lib/gamedata/projection.txt`).
* **DL** = dungeon level (50 ft per level), **CL** = character level.

---

## 1. Equipment slots, weight and encumbrance

### 1.1 The twelve slots

The only body type in the game is defined in `lib/gamedata/body.txt`:

| # | Slot type | Slot name | What goes there |
|---:|---|---|---|
| 1 | WEAPON | weapon | swords, polearms, hafted weapons, diggers |
| 2 | BOW | shooting | slings, bows, crossbows |
| 3 | RING | right hand | any ring |
| 4 | RING | left hand | any ring |
| 5 | AMULET | neck | amulets |
| 6 | LIGHT | light | torches, lanterns, artifact lights |
| 7 | BODY_ARMOR | body | soft, hard and dragon armour |
| 8 | CLOAK | back | cloaks |
| 9 | SHIELD | arm | shields |
| 10 | HAT | head | helms and crowns |
| 11 | GLOVES | hands | gloves and gauntlets |
| 12 | BOOTS | feet | boots |

There is **no two-handed weapon rule**: a 28 lb Two-Handed Great Flail and a
shield can be used together. There is **no off-hand weapon** and no swap slot;
a "swap weapon" is simply carried in the pack and wielded when needed (it costs
a turn, see [§14](#14-swap-gear-and-the-ideal-kit-by-stage)). Diggers go in
the weapon slot, so digging with a shovel means unwielding your weapon (or
using a weapon with a tunnelling bonus).

Carrying capacity (`lib/gamedata/constants.txt`): the pack holds **23** item
stacks; the quiver has **10** slots of up to **40** missiles each, and every 40
missiles in the quiver use up one pack slot. Throwing weapons inscribed for the
quiver count **5×** against that capacity (`carry-cap:thrown-quiver-mult:5`).
Stacks are capped at 40 items (`object_base.txt: default:max-stack:40`).

### 1.2 Weight, speed and the weight limit

`src/player-calcs.c:calc_bonuses()` slows you down when you carry too much:

```
limit  = adj_str_wgt[STR] * 100            (in tenths of a pound)
if total_weight > limit/2:
    speed -= (total_weight - limit/2) / (limit/10)
```

| STR | `adj_str_wgt` | Slow-down starts above | Each further −1 speed per |
|---|---:|---:|---:|
| 8 | 10 | 50 lb | 10 lb |
| 12 | 14 | 70 lb | 14 lb |
| 16 | 18 | 90 lb | 18 lb |
| 18/00 | 20 | 100 lb | 20 lb |
| 18/20 | 24 | 120 lb | 24 lb |
| 18/40 | 28 | 140 lb | 28 lb |
| 18/50 and above | 30 | 150 lb | 30 lb |

The table caps at 30 from 18/50, so **strength above 18/50 does not raise
your carrying capacity**. Everything counts: armour, weapon, pack, quiver.
Typical weight sinks are heavy body armour (Full Plate 36 lb, Adamantite
Plate 42 lb), stacks of flasks of oil (2 lb each) and spare weapons. One point
of speed is worth roughly 10% more actions; losing 2–3 points of speed to a
bag of loot you will sell for 300 gold is a classic avoidable death.

### 1.3 Weapon weight: the "heavy wield" penalty

If a weapon weighs more than your strength can hold, you are *heavy wielding*
(`calc_bonuses()`, "It is hard to hold a heavy weapon"):

```
hold = adj_str_hold[STR]          (in pounds)
if hold < weapon_weight_in_lb:
    to_h += 2 * (hold - weapon_weight_in_lb)   (a penalty)
    and you get only ONE blow, and no digging bonus from the weapon
```

| STR | 10 | 14 | 16 | 18/00 | 18/30 | 18/50 | 18/70 | 18/90 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Max weapon (lb) | 14 | 22 | 26 | 30 | 40 | 50 | 60 | 70 | 80 | 90 | 100 |

The same rule applies to launchers (`heavy_shoot`): a too-heavy launcher loses
its extra shots and extra might. Only the Heavy Crossbow (20 lb) is ever an
issue.

### 1.4 Armour weight and spellcasting

Spellcasters lose mana when the total weight of their **armour** (body,
cloak, shield, helm, gloves, boots; weapon, bow, rings, amulet and light are
ignored) exceeds the class's `spell_weight` (`calc_mana()`; the
`magic:` line in `lib/gamedata/class.txt`): **one point of maximum SP is lost
per pound over the allowance**.

| Class | Armour allowance |
|---|---:|
| Mage, Necromancer | 30 lb |
| Druid, Priest, Rogue | 35 lb |
| Paladin, Ranger | 40 lb |
| Blackguard | 60 lb |

A Mage in Soft Leather Armour (8 lb) + Cloak (1 lb) + Leather Shield (9 lb) +
Hard Leather Cap (2 lb) + Leather Gloves (1 lb) + Leather Boots (2 lb) = 23 lb
is fine; switching to Metal Scale Mail (25 lb) puts him at 40 lb and costs 10 SP.
Watch for the message "The weight of your armor reduces your maximum SP".

**There is no glove penalty in 4.2.** Older versions of Angband reduced
arcane casters' mana for wearing gloves that lacked Free Action or DEX. In
4.2.6 `calc_mana()` only checks armour weight; no class has a glove
restriction. Mages can wear Gauntlets of Power or Gloves of Slaying freely.

---

## 2. Evaluating gear and the rune identification system

### 2.1 Where to look

* **Inspect** (`I`, or `x` in the roguelike keyset) an item: the description
  lists every *known* property, the blows/damage you would get with a weapon
  (`src/obj-info.c`), the shots and damage per shot of a launcher and ammo,
  the failure rate and effect of a device, and how many turns an activation
  takes to recharge.
* **Character sheet** (`C`), then **`h` to change mode**: the second page is
  the "flags grid" showing, per equipment slot, which resistances,
  abilities, sustains and modifiers each item supplies. This is the fastest
  way to see what you are missing (no `rPois`? no `FrAct`?). Unknown runes
  show as `?`.
* **Knowledge menu** (`~`): *Display rune knowledge* lists every rune you have
  learned (and lets you inscribe a rune, so every future item with it gets
  that inscription), *Display ego item knowledge* lists the ego types you have
  fully learned, and *Display equippable comparison* compares wearables.

### 2.2 How rune-based ID works

4.2 has no "unidentified sword" in the old sense. Everything about an object
is a set of **runes** (`src/obj-knowledge.c`, overview comment at the top):

* every object flag (Free Action, See Invisible, a sustain…),
* every modifier (STR, speed, stealth, blows, shots…),
* every element's resistance/immunity/vulnerability,
* every brand and slay,
* every curse,
* and three **combat runes**: to-hit, to-dam and to-AC.

The player has a single "knowledge object" (`player->obj_k`). **When you
learn a rune, you learn it for every object in the game at once** — every item
you own, every item on the floor you have walked over, every item in a shop.
Once you have learned the "to-hit" rune from one sword, every weapon you ever
see shows its to-hit bonus immediately.

An object whose runes are all known is *fully known*; an ego whose runes are
all known shows its ego name on sight (`player_knows_ego()`). A flavoured item
(potion, scroll, ring, amulet, device) becomes *aware* when you use it or buy
it.

### 2.3 How each kind of rune is learned

| What | Learned when | Code |
|---|---|---|
| All modifiers (stats, speed, stealth, infravision, blows, shots, might, tunnelling, light…) | You **wield** the item | `object_learn_on_wield()` |
| "On wield" flags: telepathy, see invisible, blessed, light intensity, digging, throwing, sticky, fragile, burns out / fuel | You **wield** the item | id-type `on wield` in `object_property.txt` |
| Sustains (for a stat the item also boosts) | Wield | `object_learn_on_wield()` |
| "On effect" flags: Free Action, protections from fear/blind/conf/stun, sustains, Hold Life, feather fall, no-teleport, trap immunity, earthquakes | The effect actually happens (a monster tries to paralyse you, you fall through a trapdoor, a stat drain is blocked…) | `equip_learn_flag()` |
| "Timed" flags: regeneration, slow digestion, impaired HP/SP recovery, aggravation, experience drain | After some time wearing it (checked every 100 game turns) | `equip_learn_after_time()` in `game-world.c` |
| Resistances / immunities / vulnerabilities | You are hit by that element | `equip_learn_element()` from `project-player.c:adjust_dam()` |
| Brands and slays | You hit (melee, fire or throw) a monster they affect | `learn_brand_slay_*` in `player-attack.c` |
| To-hit / to-dam | You attack in melee or fire | `equip_learn_on_melee_attack()`, `missile_learn_on_ranged_attack()` |
| To-AC | A monster attacks you | `equip_learn_on_defend()` |
| Curses | Wielding reveals a curse's obvious effects; others as they trigger | `object_curses_find_*()` |
| Device/potion/scroll effects | First use | `object_learn_on_use()` |

**Selling or giving an item to any shop** (even with `birth_no_selling`)
learns *all* of its runes at once ([§11.3](#113-prices-selling-and-birth_no_selling)).

**Scroll of Identify Rune** (level 1, 200 gold, alloc 50 from DL5, not sold in
town) teaches exactly one unknown rune of the item you choose
(`effect_handler_IDENTIFY` → `object_learn_unknown_rune()`). Spend it on the
rune you cannot easily learn by use: an unknown resistance on a piece of armour
you are deciding whether to keep, or a curse.

Practical rules:

1. **Wield everything you find** (unless it is obviously a known bad item).
   It costs one turn and immediately reveals all modifiers and most flags.
   In 4.2 almost nothing is sticky: only The One Ring and the Massive Iron
   Crown of Morgoth have the `STICKY` flag; ordinary cursed items can be
   taken off.
2. The first few levels teach the combat runes and the base resists quickly;
   by DL10–15 most players know to-hit, to-dam, to-AC and the four base
   resists, and new ego items are recognised on sight.
3. An unknown rune shows as `??` in the description. An item with an
   unknown rune that you have never seen before is worth investigating: it
   is the only way you will ever learn, e.g., the nexus resist rune.
4. The birth options `birth_know_runes` and `birth_know_flavors` remove the
   whole learning process; both are off by default
   (`src/list-options.h`).

Using an unknown flavour gives a little experience:
`(object_level + CL/2) / CL` points (`object_learn_on_use()`), which matters
only at CL1–5. It is still worth reading unknown scrolls and drinking unknown
potions in a safe place early: almost nothing harmful exists (see
[§9.6](#96-bad-and-dangerous-consumables)).

### 2.4 Curses

Curses are runes too (list in `lib/gamedata/curse.txt`: vulnerability,
teleportation, dullness, sickliness, enveloping, irritation, weakness,
clumsiness, slowness, annoyance, poison, siren, hallucination, paralysis,
dragon/demon/undead summon, impaired mana/HP recovery, cowardice, stone,
anti-teleportation, treacherous weapon, burning up, chilled to the bone,
steelskin, air swing). A wearable item has a 1 in 20 chance to receive
1–4 curses when generated (`apply_magic()`→`apply_curse()`), and items with
the BLESSED flag never get random curses.

Uncursing (`effect_handler_REMOVE_CURSE`, `uncurse_object()`):

* You choose **one** curse on one item; the scroll's strength is rolled
  (Remove Curse: `20+d20`; *Remove Curse*: `50+d50`; Staff of Remove Curse
  `35+d30`).
* If strength ≥ the curse's power, that curse is removed. Curses with power
  100 are permanent.
* If it fails, the item becomes **fragile**; failing again on a fragile item
  has a 1 in 4 chance to **destroy it** (5d5 damage to you).

Because the stats and abilities of a cursed item still work, many cursed items
are worth wearing as-is (a Ring of Teleportation gives +2 speed; see
[§6](#6-rings-and-amulets)). Uncurse only items you intend to keep for a long
time, and prefer *Remove Curse* for artifacts.

---
## 3. Melee weapons

### 3.1 How a weapon's damage is built

Per blow, in normal (non-`birth_percent_damage`) play
(`src/player-attack.c:melee_damage()` and `critical_melee()`):

```
damage = roll(dice) * best_multiplier + to_dam(weapon)
then, if a critical hit: damage = crit_mult * damage + crit_add
then your own to-dam (rings, gloves, STR, Heroism...) is added per blow
```

Key consequences:

* **Slays and brands multiply only the dice**, not the to-dam. A 2d5 weapon
  (avg 6) with Slay Evil (×2) adds 6 damage per blow against evil; the same
  slay on a 6d5 Blade of Chaos adds 18. Big dice reward slays; small dice
  reward plain +to-dam.
* Only the **single best** multiplier applies (`improve_attack_modifier()` in
  `src/obj-slays.c`); a sword with Slay Evil (×2) and a fire brand (×3) does
  ×3 against an evil, non-fire-immune orc, not ×6.
* **Blows multiply everything.** Going from 2 to 4 blows doubles damage,
  including all to-dam from rings and gloves. This is why light weapons with
  extra blows (and +to-dam rings) dominate the warrior early game.

#### Slays and brands (`lib/gamedata/slay.txt`, `brand.txt`)

| Code | Affects | Melee mult | Notes |
|---|---|---:|---|
| EVIL_2 | evil monsters | ×2 | the most broadly useful slay: most of the dungeon is evil |
| ANIMAL_2 | animals | ×2 | early hounds, spiders, wolves |
| ORC_3, TROLL_3, GIANT_3 | that race | ×3 | |
| DEMON_3, DRAGON_3, UNDEAD_3 | that race | ×3 | |
| DEMON_5, DRAGON_5, UNDEAD_5 | that race | ×5 | "*Slay*" egos and top artifacts |
| ACID_3, ELEC_3, FIRE_3, COLD_3, POIS_3 | non-immune monsters | ×3 | brands; fire/cold do ×6 vs `HURT_FIRE`/`HURT_COLD` monsters |
| ACID_2 … POIS_2 | non-immune monsters | ×2 | weaker brands (e.g. Paurnimmen's COLD_2) |

The brand doubling for vulnerable monsters is in
`get_monster_brand_multiplier()`; it makes Frost brands devastating against
fire creatures and vice-versa. Brands are useless against monsters with the
matching immunity (`IM_FIRE` etc.), which is common deep down. Poison is the
most-resisted element; acid and lightning the least.

#### Criticals (`lib/gamedata/constants.txt`, `melee-critical*`)

```
chance (out of 5000) = weight + 5*(to_hit of weapon + your to_hit) + melee_skill - 60
power = weight + 1d650
power < 400: x2 +5   "good"
      < 700: x2 +10  "great"
      < 900: x3 +15  "superb"
      <1300: x3 +20  "*GREAT*"
      else : x4 +20  "*SUPERB*"
```

(weight in tenth-pounds). Heavy weapons crit more often and harder, which is
partly what keeps a 26 lb Executioner's Sword competitive with fewer blows.

### 3.2 Blows

`src/player-calcs.c:calc_blows()`:

```
P = adj_str_blow[STR] * class.att_multiply / max(weapon_weight, class.min_weight)  (cap 11)
D = adj_dex_blow[DEX]                                                             (cap 11)
energy per blow = blows_table[P][D]
blows = min(100 / energy, class.max_attacks) + extra blows
```

Class parameters (`lib/gamedata/class.txt`):

| Class | Max blows | Min weight | STR multiplier |
|---|---:|---:|---:|
| Warrior | 6 | 3 lb | 5 |
| Blackguard | 5 | **10 lb** | 5 |
| Paladin | 5 | 3 lb | 5 |
| Rogue | 5 | 2 lb | 4 |
| Ranger | 5 | 3.5 lb | 4 |
| Priest, Druid, Necromancer | 4 | 3.5 lb | 3 |
| Mage | 4 | 4 lb | 2 |

`adj_str_blow` is 3 at STR 3 up to 17 at STR 17, then 20 at 18/00 and +10 for
every 10 points of percentile strength (120 at 18/100, 220 at 18/200).
`adj_dex_blow` goes 0 (DEX ≤9), 1 (10–16), 2 (17–18/19), 3 (18/20), 4
(18/40), 5 (18/60), 6 (18/80), 7 (18/100), 8 (18/120–149), 9 (18/150), 10
(18/180), 11 (18/200+).

Selected entries of `blows_table` (energy per blow; blows = 100/energy):

| P \ D | 0 | 2 | 4 | 6 | 7 | 9 | 11 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 100 (1.0) | 95 (1.05) | 75 (1.33) | 50 (2.0) | 42 (2.4) | 30 (3.3) | 23 (4.3) |
| 2 | 95 | 75 (1.33) | 50 (2.0) | 35 (2.9) | 30 (3.3) | 23 (4.3) | 20 (5.0) |
| 4 | 75 | 50 (2.0) | 36 (2.8) | 28 (3.6) | 25 (4.0) | 21 (4.8) | 18 (5.6) |
| 6 | 50 | 36 | 30 (3.3) | 25 (4.0) | 23 (4.3) | 20 (5.0) | 17 (5.9) |
| 8 | 36 | 30 | 26 (3.8) | 22 (4.5) | 21 (4.8) | 19 (5.3) | 16 (6.25) |
| 11 | 33 | 26 | 22 (4.5) | 20 (5.0) | 19 (5.3) | 17 (5.9) | 15 (6.7) |

Worked examples:

* **Warrior, STR 18/100, DEX 18/100** (P-index uses 120×5 = 600):
  Dagger (min weight 3 lb → 30): 600/30 = 20 → capped 11 → energy 19 → 5.3
  blows; Long Sword (13 lb): 600/130 = 4 → 25 → **4.0** blows; Zweihander
  (20 lb): 600/200 = 3 → 28 → 3.6 blows; Executioner's Sword (26 lb): 2 → 30
  → 3.3 blows.
* **Mage, STR 12, DEX 18/50** (12×2 = 24): any weapon up to 4 lb gives
  24/40 = 0 → D=4 → energy 75 → **1.3 blows**. Mages never get many blows;
  their weapon is a token. A light weapon with good brands or a Defender ego
  is worth more for the *flags* than the damage.
* **Blackguard**: the 10 lb minimum means a 4 lb rapier counts as 10 lb; you
  lose nothing by wielding something heavier (up to ~10 lb) with bigger dice.
  Blackguards' Bloodlust adds extra blows and to-dam on top
  (`TMD_BLOODLUST` in `calc_bonuses()`).

Blows are fractional: 3.6 blows means the attack command costs `100/3.6`
energy per blow — over time you deliver 3.6 blows per player turn.

### 3.3 Priests, paladins and blessed weapons

There is **no penalty for priests using edged weapons in 4.2.6**. The old
rule was replaced by a bonus: classes with the `BLESS_WEAPON` player flag
(Priest and Paladin in `class.txt`) get **+2 to-dam** and the message "You
feel attuned to your weapon" when wielding a **hafted** weapon or any weapon
with the **BLESSED** flag (`calc_bonuses()`, "Divine weapon bonus").
Blessed sources: the (Blessed) and (Holy Avenger) egos, *Slay Evil* egos, and
artifacts such as Ringil, Andúril, Glamdring, Aeglin, Orcrist, Aiglos,
Oromë, Nimloth, Wrath, Ulmo, Osondir, Eönwë and Turmil.

Since priests only get 4 blows at most (multiplier 3) the +2 per blow matters
less than slays; take a great sword over a mediocre mace.

### 3.4 Every weapon base

Averages are dice only. "Depth" is the `alloc` minimum. Data:
`lib/gamedata/object.txt`; the weaponsmith stocks those marked **W**
(`store.txt`).

#### Swords (bladed; `HATES_ACID`)

| Weapon | Dice | Avg | Weight | Depth | Notes |
|---|---|---:|---:|---:|---|
| Dagger | 1d4 | 2.5 | 1.2 lb | 0 | **W**, THROWING |
| Main Gauche | 1d5 | 3 | 3 lb | 3 | **W** |
| Rapier | 1d6 | 3.5 | 4 lb | 5 | **W** |
| Short Sword | 1d7 | 4 | 8 lb | 5 | **W** |
| Tulwar | 2d4 | 5 | 10 lb | 5 | **W** |
| Cutlass | 1d8 | 4.5 | 11 lb | 5 | **W** |
| Long Sword | 2d5 | 6 | 13 lb | 10 | **W** |
| Broad Sword | 2d5 | 6 | 15 lb | 10 | **W** |
| Scimitar | 4d2 | 6 | 15 lb | 10 | **W** |
| Bastard Sword | 3d4 | 7.5 | 14 lb | 15 | **W** |
| Katana | 3d5 | 9 | 12 lb | 20 | **W**; best dice/weight of the ordinary swords |
| Zweihander | 3d6 | 10.5 | 20 lb | 30 | |
| Executioner's Sword | 4d5 | 12 | 26 lb | 40 | |
| Blade of Chaos | 6d5 | 18 | 18 lb | 70 | resist chaos built in |

#### Hafted (blunt; `HATES_ACID | HATES_FIRE`; get the priest/paladin +2)

| Weapon | Dice | Avg | Weight | Depth | Notes |
|---|---|---:|---:|---:|---|
| Whip | 1d3 | 2 | 3 lb | 3 | **W** |
| Mace | 2d4 | 5 | 12 lb | 5 | priest start weapon |
| War Hammer | 3d3 | 6 | 12 lb | 5 | |
| Maul | 4d4 | 10 | 20 lb | 5 | huge dice for depth |
| Great Hammer | 8d1 | 8 | 18 lb | 20 | |
| Morning Star | 2d6 | 7 | 15 lb | 10 | |
| Flail | 2d6 | 7 | 15 lb | 10 | |
| Quarterstaff | 1d9 | 5 | 15 lb | 10 | |
| Lead-Filled Mace | 4d3 | 8 | 18 lb | 15 | |
| Ball-and-Chain | 2d4 | 5 | 15 lb | 20 | |
| Throwing Hammer | 2d4 | 5 | 5.5 lb | 40 | THROWING |
| Two-Handed Great Flail | 3d6 | 10.5 | 28 lb | 45 | |
| Mace of Disruption | 5d8 | 22.5 | 40 lb | 80 | built-in Slay Undead ×3; needs 18/100 STR to hold |

#### Polearms (`HATES_ACID | HATES_FIRE`)

| Weapon | Dice | Avg | Weight | Depth | Notes |
|---|---|---:|---:|---:|---|
| Spear | 1d6 | 3.5 | 5 lb | 5 | **W**, THROWING |
| Trident | 1d10 | 5.5 | 7 lb | 5 | **W**; light and strong early |
| Awl-Pike | 1d8 | 4.5 | 16 lb | 10 | **W** |
| Lance | 2d8 | 9 | 30 lb | 10 | **W**; very heavy |
| Lucerne Hammer | 2d5 | 6 | 12 lb | 10 | **W** |
| Pike | 2d5 | 6 | 16 lb | 15 | **W** |
| Beaked Axe | 2d6 | 7 | 18 lb | 15 | **W** |
| Broad Axe | 2d6 | 7 | 16 lb | 15 | **W** |
| Battle Axe | 2d8 | 9 | 17 lb | 15 | **W** |
| Glaive | 2d6 | 7 | 19 lb | 20 | |
| Halberd | 3d5 | 9 | 19 lb | 25 | |
| Throwing Axe | 2d3 | 4 | 4 lb | 25 | THROWING |
| Great Axe | 4d4 | 10 | 23 lb | 40 | |
| Lochaber Axe | 3d8 | 13.5 | 25 lb | 45 | |
| Scythe | 5d3 | 10 | 25 lb | 45 | |
| Scythe of Slicing | 8d4 | 20 | 25 lb | 60 | |

#### Diggers

| Digger | Dice | Weight | Depth | Digging | Store |
|---|---|---:|---:|---|---|
| Shovel | 1d2 | 6 lb | 5 | DIG_1 (+20) | General store, always |
| Pick | 1d3 | 18 lb | 30 | DIG_2 (+40) | General store, always |
| Mattock | 1d8 | 25 lb | 50 | DIG_3 (+60) | |

Great weapons can also have their dice "super-charged" when they are
generated as great items: `apply_magic_weapon()` repeatedly (1 in
`4×dd×ds`) adds dice or sides up to a product of 40. A 3d5 Katana that rolled
5d5 is a real find; always check the dice on great weapons.

### 3.5 Digging

`calc_digging_chances()` gives the chance (out of 1600) per turn:

| Terrain | Chance per turn |
|---|---|
| Rubble | `skill × 8` |
| Magma | `(skill − 10) × 4` |
| Quartz | `(skill − 20) × 2` |
| Granite | `(skill − 40) × 1` |

The digging skill is the race base (0 for most races, 40 for Dwarves in
`p_race.txt`) + `adj_str_dig[STR]` (0 at STR 3, 9
at 18/00, 50 at 18/100, 100 at 18/200) + **weapon weight in pounds** + **20
per digger level** (DIG_1/2/3) + **20 per point of tunnelling** (the
`TUNNEL` modifier on a digger, a Ring of Digging (+2+d3), or artifacts like
Aulë +8, Náin +6, Durin +5, Erebor +5, Deathwreaker +4). With STR 18/100 and a
Pick (+40, 18 lb) you have ~110 skill: granite in about 23 turns on average,
magma in ~4. In practice, a **Wand of Stone to Mud** (20+1d30 damage line,
6+d10 charges, sold in the Magic shop) or the spell is how most characters dig.
Diggers are for treasure veins (gold in magma/quartz with treasure) and for
Rangers/Warriors who want gold early. Ego diggers: *of Digging* (TUNNEL +d4,
acid brand, DL1–40) and *of Earthquakes* (STR d4, TUNNEL d6, IMPACT, acid
brand, DL10+).

### 3.6 Weapon egos

From `lib/gamedata/ego_item.txt`. "Common" is the ego's allocation weight
relative to others that fit the item; "Depth" is the ego's min–max level
(an ego can appear before its min only by an out-of-depth roll of
1 in `max(2,(min−level)/3)` per eligible ego, `ego_find_random()`).

| Ego | Common | Depth | Bonuses (to-hit/to-dam added on top of base magic) | Key powers |
|---|---:|---|---|---|
| (Holy Avenger) | 10 | 15+ | +d6/+d6, +d4 AC | SI, BLESSED, pFear, random sustain, WIS +d4; slay evil ×2, undead ×3, demon ×3 |
| (Defender) | 10 | 10–80 | +d4/+d4, +d8 AC | **FA, SI, feather fall, regen**, random sustain, stealth +d4, **rAcid rElec rFire rCold** |
| of Gondolin | 10 | 20+ | +d7/+d7 | SI, FA, random power, light +1, rDark; slay demon/orc/troll/dragon ×3 |
| of Westernesse | 10 | 10–70 | +d5/+d5 | **FA, SI**, STR/DEX/CON +1–2; slay orc/troll/giant ×3 |
| (Blessed) | 20 | 1–60 | — | BLESSED, random power, WIS +d3 |
| of Extra Attacks | 10 | 10+ | — | **+1 or +2 blows** |
| of Acid / Lightning | 30 | 1–60 | — | brand ×3 + matching resist |
| of Flame / Frost | 40 | 1–50 | — | brand ×3 + matching resist |
| of Venom | 60 | 1–40 | — | poison brand ×3 |
| of Slay Animal / Undead / Demon / Dragon | 100 | 1–30 | — | ×2 (animal) or ×3 |
| of Slay Orc / Troll | 150 | 1–20/25 | — | ×3 |
| of Slay Evil | 60 | 1–40 | — | ×2 evil |
| of Slay Giant | 60 | 1–30 | — | ×3 |
| of \*Slay Evil\* | 15 | 10+ | — | ×2 evil, BLESSED, WIS +d2, random resist **or** power |
| of \*Slay Undead/Demon/Dragon\* | 30 | 10–60 | — | **×5**, stat +d2, random resist or power |
| of \*Slay Animal/Orc/Troll/Giant\* | 20–60 | 1–45 | — | ×2/×3, stat +d2, random resist or power |
| of Morgul | 10 | 10–80 | — | SI, Hold Life, but **AGGRAVATE + EXP drain**; poison brand, slay undead |
| of Fury | — | — | — | defined but "not currently creatable" (its `type:` lines are commented out) |

"Random resist or power" (`RAND_RES_POWER`) is 1/3 a random ability
(protection, regen, SI, FA, feather fall…) and 2/3 a random base resist
(`ego_apply_magic()`). "Random power" picks from the protection/misc flag set;
"random sustain" picks a sustain.

**How to rank them.** For most melee characters the order is roughly:
*Holy Avenger* ≈ *Defender* ≈ *Westernesse* (early–mid) > *Gondolin* >
*\*Slay Evil\** with good dice > *Extra Attacks* on a light weapon >
elemental brands > plain slays. **Westernesse** and **Defender** are the
single most important early egos because they bring **Free Action and See
Invisible** — two of the three "must-have before DL30" flags
(see [Chapter 4](04-resistances-afflictions.md)) — in your weapon slot.
A Defender also covers all four base resists.

### 3.7 Good early weapons

* **Start**: whatever the kit gives (Warrior Dagger, Priest Mace, Rogue Dagger,
  Ranger Main Gauche + Short Bow…). For a fighter, buy a **Tulwar** (2d4,
  10 lb), **Trident** (1d10, 7 lb) or **Main Gauche/Rapier** depending on
  blows. Check the Inspect screen: it shows blows and damage per turn.
* **Artifact daggers** are common early and excellent: 'Narthanc', 'Nimthanc',
  'Dethanc' (1d4, +9,+12, fire/cold/lightning brand, a bolt activation, DL1–50,
  alloc 40), 'Forasgil' (rapier 1d12, +12,+19, cold brand, slay animal, alloc
  38, DL5–50), and 'Elvagil' (long sword, alloc 50, DL10–50).
* **'Sting'** (short sword, DL20+): +3 speed, **+2 blows**, FA, SI, pFear,
  slay evil/undead/orc/animal. A light, fast weapon that carries mages and
  rogues to the late game.
* Any **(Defender)** or **of Westernesse** you find.

### 3.8 Endgame weapons

The standard-set artifacts that stay in use to the end (full list in
[A4](A4-artifacts.md#sword)):

| Artifact | Base | Depth / rarity | Why |
|---|---|---|---|
| **'Ringil'** | Long Sword 4d5 (+22,+25) | DL20+, alloc 1 | **+10 speed**, cold brand, slay evil, undead ×3, troll, **demon ×5**, FA, SI, regen, blessed; cold ball 100 activation |
| **'Andúril'** | Long Sword 3d5 (+10,+15) | DL20+, alloc 3 | STR/DEX +4, fire brand, slay evil/undead/orc/troll, FA, SI, **rDisen**, blessed; fire ball 72 |
| **'Aranrúth'** | Broad Sword 3d5 (+20,+12) | DL20–100, alloc 2 | DEX +4, FA, slay demon/orc, rCold; frost bolt activation |
| **'Deathwreaker'** | Mace of Disruption 7d8 (+18,+18) | DL80+ | STR +6, fire brand, undead ×5, **rChaos, rDisen**, fire immunity — but **aggravates** |
| **'Doomcaller'** | Blade of Chaos 4d7 (+18,+28) | DL70+ | **telepathy**, FA, SI, base four + chaos resists, dragon ×5 — but aggravates and −50 AC |
| **of Eönwë** | Great Axe 5d4 (+15,+18) | DL40+, alloc 1 | all stats +2, **cold immunity**, demon ×5, evil, undead; activation **Mass Banishment** |
| **of Durin** | Great Axe 4d4 (+10,+20) | DL40+ | dragon ×5, acid+fire brands, rChaos/Light/Dark, FA |
| **of Aulë** | Great Hammer 18d1 (+19,+21) | DL40+ | dragon ×5, acid brand, FA, SI, base four + rNexus, tunnelling +8 |
| **of Ulmo** | Trident 4d10 (+15,+19) | DL30+ | acid immunity, FA, SI, Hold Life, rNether; **Teleport Other** activation |
| **'Zarcuthra'** | Zweihander 3d8 (+19,+21) | DL30+ | slays every race, dragon ×5, rChaos, but aggravates |
| **'Gurthang'** | Zweihander 3d6 (+13,+17) | DL30–100 | fire+poison brands, dragon ×5, FA, rPois |
| **'Grond'** | Mighty Hammer 9d9 (+5,+25) | Morgoth drop | quest item: telepathy, SI, slays everything relevant, IMPACT |

"Deathwing" does not exist in 4.2.6. Note that the 4.2 artifact set gives
the "one-stop" FA+SI to so many weapons that a weapon without them is usually
an upgrade only if you already have both elsewhere.

**Ego endgame picks**: a *Holy Avenger* or *\*Slay Evil\** on a weapon with big
super-charged dice, or *Extra Attacks* on a Katana/Zweihander, can outdamage
most artifacts for a warrior with 5–6 blows. Keep a ×5 *\*Slay Dragon\** /
*\*Slay Demon\** / *\*Slay Undead\** weapon as a swap if your main weapon lacks
that multiplier and you are hunting the corresponding uniques.

---

## 4. Launchers, ammunition and throwing

### 4.1 How missile damage works

`src/player-attack.c:ranged_damage()`:

```
mult    = launcher multiplier (+ extra might) + best slay/brand multiplier
damage  = (roll(ammo dice) + ammo to_dam + launcher to_dam) * mult
then criticals (critical_shot)
```

Two big differences from melee:

1. **Slays and brands add to the launcher multiplier** instead of
   multiplying the dice. A ×3 Long Bow firing Arrows of Slay Evil has a
   multiplier of **5** against evil; with Flame arrows against a
   fire-vulnerable monster, 3 + 6 = **9**.
2. **The multiplier also multiplies the to-dam** of both ammo and launcher.
   A Long Bow (+10,+15) ×3 with plain Arrows (1d4, +5) does
   `(2.5 + 5 + 15) × 3 = 67.5` per shot. To-dam on a launcher is worth three
   or four times what it is on a sword.

Brands and slays on the **launcher** apply to every missile it fires
(`make_ranged_shot()` calls `improve_attack_modifier()` on ammo and bow), so
'Cubragol' (fire brand) turns every bolt into a fire bolt.

Other rules (`calc_bonuses()`, `do_cmd_fire()`):

* **Shots** are counted in tenths: every launcher starts at 10 (= 1 shot per
  turn); a `SHOTS[x]` modifier adds `x` tenths. So *of Extra Shots* (2d3)
  adds 0.2–0.6 shots, and 'Belthronding' (SHOTS 10) adds a full shot.
  **Rangers** get `CL/3` extra tenths (`FAST_SHOT`): +1.6 shots at CL50.
* **Range** is `6 + 2 × multiplier`, capped at 20 squares.
* **Criticals** (`ranged-critical*` constants): chance out of 5000 =
  `weight + 4 × (to-hit) + 2 × CL`; power `weight + 1d500`:
  <500 ×2+5, <1000 ×2+10, else ×3+15.
* **Breakage** (`breakage_chance()`, `object_base.txt`): shots 0%, bolts
  20%, arrows 35% on a hit; on a miss the chance is squared (bolts 4%,
  arrows 12%). Artifacts never break. Pick your ammo up after a fight.

### 4.2 Launcher bases

| Launcher | Mult | Ammo | Weight | Depth | Store |
|---|---:|---|---:|---:|---|
| Sling | ×2 | shots, pebbles | 0.5 lb | 1 | Weaponsmith |
| Short Bow | ×2 | arrows | 3 lb | 3 | Weaponsmith |
| Long Bow | ×3 | arrows | 4 lb | 10 | Weaponsmith |
| Light Crossbow | ×3 | bolts | 11 lb | 15 | Weaponsmith |
| Heavy Crossbow | ×4 | bolts | 20 lb | 30 | — |

Which to use:

* **Early**: a Sling is cheap and its iron shots never break, but its ×2
  mult is weak. Buy a **Long Bow** (120 gold) as soon as you can afford one —
  ×3 is a 50% damage increase over a Short Bow for the same arrows.
* **Rangers** use bows (their extra shots apply to any launcher, but arrows
  are lighter and plentiful).
* **Warriors, Paladins, Blackguards**: a **Heavy Crossbow** (×4) is the
  strongest base; Light Crossbows of the Haradrim (+1 might, +0.2–0.6 shots)
  are excellent.
* **Casters**: any launcher is a free second attack that costs no mana. A
  Sling of Buckland or a ×3 bow is worth carrying all game.

### 4.3 Launcher egos

| Ego | On | Common | Depth | Bonuses | Notes |
|---|---|---:|---|---|---|
| of Accuracy | any | 100 | 5–70 | +d15 hit (min 15), +d5 dam | |
| of Power | any | 100 | 5–70 | +d5 hit, +d15 dam (min 15) | to-dam is multiplied; the common good find |
| of Extra Might | any | 20 | 15–100 | +d5/+d10; **+1 multiplier** | |
| of Extra Shots | any | 10 | 15–100 | +d10/+d5; **+0.2–0.6 shots** | |
| of Lothlórien | Short/Long Bow | 5 | 20+ | +d10/+d10, DEX +2–4, **might +1–2**, FA, random power | top bow ego |
| of the Haradrim | Light Crossbow | 5 | 20+ | +d5/+d5, **might +1, shots +0.2–0.6** | top crossbow ego |
| of Buckland | Sling | 5 | 20+ | +d8/+d8, DEX +2–4, **shots +0.2–1.0, might +1–2** | makes the sling competitive |
| of the Nazgûl | any | 10 | 10–80 | +d10/+d10, SI, **experience drain** | cursed-style ego |

Launcher artifacts ([A4 § Bow](A4-artifacts.md#bow)): 'Belthronding' (Long Bow,
+20/+22, **+1 shot**, DEX +3, +1 speed, rDisen, DL40+), of Bard (Long Bow,
might +2, FA, DL40+), of Amrod / of Amras (Short Bows, base resists, might or
shots, DL30+), 'Cubragol' (Light Crossbow, **+10 speed**, fire brand, DL50+),
of Umbar (Heavy Crossbow, might +2, a 150-damage arrow activation, but
aggravates).

### 4.4 Ammunition

| Ammo | Dice | Depth | Notes |
|---|---|---:|---|
| Rounded Pebble | 1d2 | 0 | THROWING |
| Iron Shot | 1d4 | 3 | THROWING; General store always, Weaponsmith |
| Mithril Shot | 2d4 (+5,+5) | 40 | THROWING; ignores acid/fire |
| Arrow | 1d4 | 3 | General store always, Weaponsmith |
| Mithril Arrow | 3d4 | 50 | ignores acid/fire |
| Seeker Arrow | 4d4 | 55 | |
| Bolt | 1d5 | 3 | General store always, Weaponsmith |
| Mithril Bolt | 3d5 | 60 | ignores acid |
| Seeker Bolt | 4d5 | 65 | |

Great ammo has a 1 in 6 chance of +1 die side, and of those 1 in 10 a second
(`apply_magic_weapon()`).

Ammo egos (`ego_item.txt`): *Slay Animal* (common 80, DL1–40), *Slay Evil*
(10, DL10+), *Slay Undead* / *Slay Demon* (15, DL5+), *Slay Dragon* (10,
DL10+), *Slay Giant* (5, DL5–60), *Venom* (20, DL10–60), *Flame* / *Frost*
(10, DL10+), *Acid* / *Lightning* (5, DL10+), *Wounding* (15, DL15+, +3+d5 to
hit and dam), *Holy Might* (2, DL30+, mithril/seeker only: +d10/+d10, evil ×2,
demon and undead ×3), and *Backbiting* (−26+d25 to hit and dam; junk).

Strategy: keep **one stack of plain ammo for trash** and **one or two
stacks of slaying/branded ammo** for uniques and dangerous monsters. Evil
covers most of the late game; Flame/Frost arrows on a ×4 crossbow give a ×7
multiplier against non-immune monsters. Mithril and Seeker ammo cost more to
lose but are the best sinks for *Enchant Weapon* scrolls.

### 4.5 Throwing (the `v` command)

`ranged_damage()` with no launcher:

```
damage = (roll(dice) + to_dam) * (2 + weight_in_tenth_lb / 12)   [THROWING items only]
         * (1 + best slay/brand multiplier)
flasks of oil (EXPLODE): damage * 3 more
range  = min((adj_str_blow[STR] + 20) * 10 / max(weight, 10), 10)
```

THROWING items also get all your equipment's to-hit, and use the throwing
skill; non-throwing objects use 1.5× the throwing skill but none of your
equipment's bonuses (`chance_of_missile_hit_base()`). Throwing weapons only
break 1% of the time (`breakage_chance()`).

| Missile | Avg dice | × weight factor | Typical damage (no to-dam) |
|---|---:|---:|---:|
| **Flask of oil** (2 lb) | 2.5 | ×3, then ×3 explode | **22.5** |
| Dagger (1.2 lb) | 2.5 | ×3 | 7.5 |
| Spear (5 lb) | 3.5 | ×6 | 21 |
| Throwing Axe (4 lb) | 4 | ×5 | 20 |
| Throwing Hammer (5.5 lb) | 5 | ×6 | 30 |
| Iron Shot (0.5 lb) | 2.5 | ×2 | 5 |

**Flasks of oil are the best early-game attack in Angband.** 3 gold each,
always stocked by the General Store, 22 damage on average at DL1 — more than
a starting Mage's Magic Missile (3d4 = 7.5 at CL1). Grip and Fang (25 HP each)
die to two hits, Bullroarer (60 HP) to about three. The flask's damage is not reduced by fire
resistance (it has no brand; the ×3 is unconditional), so it works on
everything. Buy 15–25 for any class before the first descent; keep a stack
until you have a better ranged attack (CL15–25 for most characters). They also
refuel lanterns ([§7](#7-light-sources)). They weigh 2 lb each, so watch your
weight limit.

Because to-dam is multiplied by the weight factor, **throwing weapons with
+to-dam are strong**: a Throwing Axe (+8,+10) does `(4+10)×5 = 70`, and an
artifact spear like 'Aiglos' or of Oromë would do far more if you were willing
to throw it. **Rogues** (throwing skill 72 + 45 per 10 levels, the best in the
game) can build around thrown weapons: inscribe them `@v1` and keep them in the
quiver.

---

## 5. Armour, slot by slot

### 5.1 What armour class does

* **Avoiding blows.** A monster blow has hit power `3 × monster level +
  blow power` (`chance_of_monster_hit_base()`); it hits if a roll from 0 to
  that power is at least `2/3 × your AC`, with a flat 12% always-hit and 5%
  always-miss (`hit_chance()` in `player-attack.c`).
* **Reducing physical damage.** For `HURT`-type blows (and some others)
  `adjust_dam_armor()` removes `AC/400` of the damage, up to 60% at AC 240.
* **Acid.** If acid damages a piece of your armour, the acid damage you take
  is halved (`adjust_dam()`); that is why armour "ignores acid" is a mixed
  blessing.
* AC does **nothing** against breaths and spells. Resistances do. A
  low-AC/high-resist set beats a high-AC/low-resist set from about DL30 on.

Heavy body armour also has a **to-hit penalty** (the `attack:` line, e.g.
−3 for plate) and costs casters mana (see [§1.4](#14-armour-weight-and-spellcasting)).

### 5.2 Body armour

| Armour | AC | Weight | To-hit | Depth | Store |
|---|---:|---:|---:|---:|---|
| Robe | 2 | 2 lb | 0 | 1 | Armoury |
| Soft Leather Armour | 8 | 8 lb | 0 | 3 | Armoury |
| Studded Leather Armour | 12 | 10 lb | −1 | 5 | Armoury |
| Hard Leather Armour | 16 | 12 lb | −1 | 10 | Armoury |
| Leather Scale Mail | 20 | 14 lb | −1 | 15 | Armoury |
| Chain Mail | 32 | 22 lb | −2 | 20 | Armoury |
| Metal Scale Mail | 38 | 25 lb | −2 | 25 | Armoury |
| Partial Plate Armour | 40 | 26 lb | −3 | 30 | |
| Augmented Chain Mail | 42 | 27 lb | −2 | 35 | |
| Bar Chain Mail | 45 | 28 lb | −2 | 40 | |
| Metal Brigandine Armour | 48 | 29 lb | −3 | 45 | |
| Metal Lamellar Armour | 58 | 34 lb | −3 | 50 | |
| Full Plate Armour | 62 | 36 lb | −3 | 55 | |
| Ribbed Plate Armour | 66 | 38 lb | −3 | 60 | |
| **Mithril Chain Mail** | 28 | **15 lb** | −1 | 55 | ignores acid |
| **Mithril Plate Mail** | 60 | 30 lb | −3 | 65 | ignores acid |
| Adamantite Plate Mail | 80 | 42 lb | −4 | 75 | ignores acid |
| Dragon scale mails | 12–60 (+10/+15) | 10–30 lb | −2/−3 | 25–100 | see [§8](#8-dragon-scale-mail-activations-and-device-skill) |

Body-armour egos (`ego_item.txt`):

| Ego | Applies to | Common | Depth | Gives |
|---|---|---:|---|---|
| of Resist Acid / Lightning / Fire / Cold | soft armour; Metal Scale to Partial Plate | 100 each | 1–30 | one base resist |
| **of Resistance** | soft or hard armour | 50 | 10–100 | **all four base resists**, +d10 AC |
| **of Elvenkind** | soft or hard armour | 10 | 30+ | four base resists, **one random high resist**, stealth +d2, +d10 AC |
| **of Permanence** | **Robe only** | 10 | 30+ | four base resists, random high resist, **all five sustains, Hold Life**, +d20 AC |
| (Dwarven) | hard armour | 20 | 20+ | **Free Action**, STR/CON +1–2, infravision, +d15 AC |

The best common upgrade for a mid-game character is **armour of Resistance**:
it frees two other slots that would otherwise carry base resists. A Robe of
Permanence is light enough for any caster. Notable artifact armours:
'Hithlomir' and of Arvedui (base four + extra), 'Thalkettoth' (+3 speed, FA),
'Belegennon' (base four + rPois, −5 damage reduction), of Isildur, of
Celeborn (Banishment activation), 'Soulkeeper' (Heal 1000 activation)
([A4 § Hard Armour](A4-artifacts.md#hard-armour)).

### 5.3 Cloaks

| Cloak | AC | Weight | Depth | Notes |
|---|---:|---:|---:|---|
| Cloak | 1 | 1 lb | 1 | General store always |
| Fur Cloak | 3 | 2.5 lb | 20 | |
| Elven Cloak | 6 | 0.5 lb | 40 | **+1 stealth, +1 speed** built in |
| Ethereal Cloak | 0 (+10) | 0 lb | 70 | ignores all elements |

| Ego | Common | Depth | Gives |
|---|---:|---|---|
| of Stealth | 60 | 1–70 | stealth +d3 |
| of Protection | 30 | 1–40 | +d10 AC, **rShards**, ignores elements |
| of the Magi | 10 | 30+ | INT +d2, stealth +d3, sustain INT, random power, +d4 AC |
| **of Aman** | 10 | 20+ | **random high resist**, stealth +d3, +d20 AC |

Cloak artifacts are some of the best early "gifts": **'Colluin'** (base four
+ rPois, *resist-all* activation, DL5+), **of Thorongil** (FA, SI, pFear, DL5–75,
alloc 10), 'Colannon' (+2 speed, rNexus, teleport activation), of Thingol
(FA, Recharge activation), of Lúthien (Elven Cloak; +2 speed, INT/WIS +4,
Restore Life Levels).

### 5.4 Shields

| Shield | AC | Weight | Depth | Store |
|---|---:|---:|---:|---|
| Wicker Shield | 2 | 3 lb | 1 | Armoury |
| Leather Shield | 8 | 9 lb | 10 | Armoury |
| Small Metal Shield | 5 | 6 lb | 15 | Armoury |
| Large Metal Shield | 12 | 12 lb | 30 | |
| Knight's Shield | 15 | 16 lb | 50 | |
| Mithril Shield | 16 (+10) | 10 lb | 70 | ignores acid |

| Ego | Common | Depth | Gives |
|---|---:|---|---|
| of Resist Acid / Lightning / Fire / Cold | 100 each | 1–30 | one base resist |
| **of Resistance** | 50 | 10–100 | four base resists, +d10 AC |
| of Elvenkind | 10 | 30+ | four base resists, random high resist, stealth +d2 |
| **of Preservation** | 5 | 40+ | sustain STR/DEX/CON, **Hold Life, rDisen**, random high resist, +d20 AC |
| (Dwarven) | 20 | 40+ | Small Metal Shield only: **FA**, STR/CON +1–2 |

Warriors and Paladins (and Blackguards, `SHIELD_BASH`) can shield-bash; every
class can wear any shield. A caster's shield choice is governed by weight
(§1.4). Artifact shields: of Celegorm (base four + rLight + rDark, pBlind),
of Anárion (all sustains, base four), of Thorin (acid immunity, rChaos, rSound,
FA), of Gil-galad (rDisen, rDark, WIS +5), of Elros (rShards, Hold Life,
pConf).

### 5.5 Helms and crowns

| Headgear | AC | Weight | Depth | Store |
|---|---:|---:|---:|---|
| Hard Leather Cap | 2 | 2 lb | 3 | Armoury |
| Metal Cap | 3 | 2.5 lb | 10 | Armoury |
| Iron Helm | 7 | 5 lb | 20 | Armoury |
| Steel Helm | 9 | 6 lb | 40 | |
| Iron Crown | 0 | 2 lb | 45 | crown |
| Golden Crown | 0 | 3 lb | 45 | crown, ignores acid |
| Jewel Encrusted Crown | 0 | 4 lb | 50 | crown, ignores acid |

Helm and crown egos are **separate lists** (`type:helm` vs `type:crown`):

| Ego | On | Common | Depth | Gives |
|---|---|---:|---|---|
| of Intelligence / of Wisdom | helm | 100 | 1–40 | stat +d2, sustain |
| of Infravision | helm | 100 | 1–20 | infravision +d5 |
| of Regeneration | helm | 20 | 1–60 | regeneration |
| **of Seeing** | helm | 50 | 10–60 | **See Invisible, pBlind**, searching +d5 |
| **of Telepathy** | helm | 10 | **40+** | **telepathy (ESP)** |
| of Light | crown | 50 | 5–30 | light +1, rLight |
| of Night and Day | crown | 15 | 10–80 | light +1, **rLight, rDark, SI, pBlind** |
| **of Serenity** | crown | 15 | 20+ | **pConf, pFear, pStun, rSound** |
| of the Magi | crown | 20 | 20+ | INT +d3, base four resists, sustain INT, random power |
| of Lordliness | crown | 20 | 40+ | WIS +d3, pFear, sustain WIS, random power |
| of Might | crown | 10 | 40+ | STR/DEX/CON +d3, sustains, **FA** |

Crowns appear from DL45 at the earliest (out-of-depth aside), so a crown of
Serenity or the Magi is a mid-to-late upgrade. The **Helm of Telepathy** is
the main non-artifact source of ESP alongside the Amulet of ESP. Artifact
helms: of Thranduil (ESP, pBlind), of Dor-Lómin (ESP, SI, base four, STR/CON
+4), 'Holhenneth' (SI, pBlind, pConf, detection activation), of Celebrimbor
(rDisen, rShards). Crowns: of Gondor (pConf/pBlind/pStun, Heal 500), of
Númenor, of Berúthiel (ESP, FA, SI but −5 CON).

### 5.6 Gloves

| Gloves | AC | Weight | Depth | Store | Notes |
|---|---:|---:|---:|---|---|
| Set of Leather Gloves | 1 | 1 lb | 1 | Armoury | |
| Set of Gauntlets | 3 | 2.5 lb | 10 | Armoury | |
| Set of Caestus | 5 | 4 lb | 20 | | **+3 to-dam** built in |
| Set of Mithril Gauntlets | 6 | 1.5 lb | 40 | | ignore acid/fire |
| Set of Alchemist's Gloves | 0 (+5+d5) | 0.5 lb | 50 | | ignore all elements |

| Ego | On | Common | Depth | Gives |
|---|---|---:|---|---|
| **of Free Action** | any | 100 | 1–60 | **Free Action** |
| of Slaying | any | 20 | 10–75 | +d5 hit, +d5 dam |
| of Agility | any | 10 | 20–50 | DEX +d5 |
| of Power | any | 5 | 30+ | STR +d5, +d5/+d5 |
| of Thievery | Leather / Alchemist's | 5 | 40+ | FA, feather fall, DEX +1+M4, searching, +d8 hit, +d3 dam |
| of Combat | Gauntlets / Mithril / Caestus | 5 | 50+ | STR/CON +d2, +d5 hit, +d8 dam, **aggravation** |

**Gloves of Free Action are the most common FA source in the game** (commonness
100 from DL1): if you have no FA by DL15, check the Black Market and every
pair of gloves you find. Because there is no glove spell penalty in 4.2
([§1.4](#14-armour-weight-and-spellcasting)), every class can wear Slaying
or Power gloves; their +to-dam applies to every blow. Brands on gloves (e.g.
'Paurnimmen', COLD_2) apply to your melee blows too, because melee checks the
slays and brands on all non-launcher equipment (`py_attack_real()`).

### 5.7 Boots

| Boots | AC | Weight | Depth | Store |
|---|---:|---:|---:|---|
| Pair of Leather Sandals | 1 | 1.5 lb | 1 | Armoury |
| Pair of Leather Boots | 2 | 2 lb | 3 | Armoury |
| Pair of Iron Shod Boots | 4 | 3.5 lb | 5 | Armoury |
| Pair of Steel Shod Boots | 7 | 6 lb | 20 | |
| Pair of Mithril Shod Boots | 8 | 4 lb | 60 | |
| Pair of Ethereal Slippers | 0 | 0 lb | 30 | ignore all elements |

| Ego | On | Common | Depth | Gives |
|---|---|---:|---|---|
| of Slow Descent | any | 100 | 1–15 | feather fall |
| **of Free Action** | any | 20 | 1–40 | Free Action |
| of Stealth | Sandals, Leather, Ethereal | 20 | 15+ | stealth +d3 |
| of Stability | any | 10 | 15+ | **rNexus**, feather fall |
| **of Speed** | any | **3** | 20+ | **speed +2+M8** (up to +10) |
| **of Elvenkind** | Leather, Iron Shod | 3 | 60+ | **speed +4+M5** (up to +9), stealth +d4, feather fall |

**Boots of Speed** are the most sought-after ego in the game. The `M8` term
averages `8×level/128`, so boots found at DL40 average about +4.5 and those at
DL90+ about +7–8. Artifact boots: **of Fëanor** (+15 speed, rNexus, haste
activation, DL40+, alloc 1), 'Dal-i-thalion' (+5 speed, FA, pConf, rNether),
'Stormwalker' (+6 speed, lightning immunity, rSound), of Thrór (+3 speed),
of Wormtongue (+1 move, trap immunity, phase activation), of Radagast (FA).

---

## 6. Rings and amulets

All jewellery is flavoured (you learn what a "Bloodstone Ring" is by wearing
it). Most kinds are `EASY_KNOW`: their single property is obvious on wield.
Depth is the `alloc` range; "Shop" means the Magic shop stocks it (`store.txt`).
Two rings can be worn, one amulet.

### 6.1 Rings

| Ring | Depth | Value | Effect | Verdict |
|---|---|---:|---|---|
| **Speed** | 75–100 | 100000 | speed **+4+M6**, then +1 more while a 1-in-2 roll succeeds (`apply_magic()`) | the best ring; a +10 or better ring is endgame |
| **Free Action** | 20–100 | 1500 | Free Action | essential early if no other FA |
| **See Invisible** | 30–100 | 340 | See Invisible | essential if no other SI |
| **Resist Poison** | 40–100 | 16000 | rPois | the standard rPois source before you find armour/artifacts with it |
| Resist Fire and Cold | 10–50 | 250 | rFire, rCold | early-game staple |
| Flames / Acid / Ice / Lightning | 20–100 | 500 | one base resist, +5+d5M10 AC; **activation**: a radius-2 ball (80/70/75/85 damage) **plus** 20+d20 turns of temporary resistance (`activation.txt` RING_*) | "double resist" before a big breather; decent attack for casters |
| Strength / Intelligence / Dexterity / Constitution | 30–100 | 500 | stat +1+M5, sustain | DEX/STR rings for blows; CON for HP; INT for mages |
| Damage | 20–100 | 500 | +5+d3M7 to-dam | melee staple: added to every blow |
| Accuracy | 20–100 | 500 | +10+d6M7 to-hit | |
| Slaying | 40–100 | 1000 | +d10M10 to-hit, +d5M5 to-dam | |
| Protection | 10–40 | 500 | +5+d5M10 AC | Shop |
| Bodykeeping | 30–100 | 750 | sustain STR, DEX, CON | |
| Soulkeeping | 30–100 | 600 | sustain STR, INT, WIS | |
| Light | 25–100 | 400 | light +1, rLight, searching | rLight early |
| Digging | 10–50 | 800 | tunnelling +2+d3; **activation**: stone to mud (15+2d10 turns) | |
| Feather Falling | 10–50 | 200 | feather fall | Shop |
| Slow Digestion | 10–50 | 250 | slow digestion | |
| Searching | 5–90 | 250 | searching skill (trap detection) | Shop |
| Open Wounds | 10–50 | 700 | **impairs HP regeneration**; activation heals 40 HP and cures blindness, cuts, confusion (10+5d8 turns) | Shop; a cheap reusable CLW for the early game |
| Escaping | 15–80 | 4000 | **+2 speed** but permanent fear (you cannot melee) | excellent for pure casters and archers |
| Teleportation | 10–50 | 1000 | +2 speed, **teleportation curse** (random teleports) | an early speed ring if you can live with random teleports; good for exploring, bad when you need to stay put |
| the Mouse | 10–50 | 100 | DEX +d4, stealth +d4, **−3d5 to-dam** | for casters and rogues |
| the Dog | 10–50 | 100 | pFear, −2 stealth | cures the fear that stops melee; warriors get pFear at CL30 anyway (`BRAVERY_30`) |
| Reckless Attacks | 10–50 | 200 | +2+d3 hit and dam, −8+4d3 AC | |

Artifact rings ([A4 § Ring](A4-artifacts.md#ring)): **'Narya'** (+5 speed,
all stats +1, FA, SI, ESP, regen, fire immunity; DL70+), **'Nenya'** (+5
speed, all stats +2, FA, SI, ESP, Hold Life, cold immunity; DL80+),
**'Vilya'** (+5 speed, all stats +3, FA, SI, ESP, lightning immunity, rPois,
rDark; Heal 1000 activation; DL90+), **The One Ring** (+5 speed, all stats
+5, immunity to all four base elements, rPois, rDark, rDisen, rNether, ESP,
SI — but **aggravation, experience drain, and STICKY**: once worn it cannot be
removed), of Barahir (all stats +1, rPois, rDark), of Tulkas (STR/DEX/CON +4,
Haste 75+d75 activation).

### 6.2 Amulets

| Amulet | Depth | Value | Effect | Verdict |
|---|---|---:|---|---|
| **ESP** | 60–100 (level 50) | 25000 | **telepathy** | the most important amulet for most characters |
| **Trickery** | 70–100 | 30000 | DEX +1+M4, **speed +1+M2**, stealth, searching, infra, **rPois, rNexus**, sustain DEX | superb for rogues/archers |
| **Weaponmastery** | 70–100 | 30000 | +1+M4 hit/dam, STR +1+M2, **rDisen, FA, pFear**, sustain STR/CON | the melee amulet |
| **the Magi** | 70–100 | 30000 | INT +1+M3, **FA, SI, pBlind**, sustain INT, searching, +d5M5 AC | the caster amulet |
| **Devotion** | 70–100 | 30000 | WIS +1+M3, light +1, **rLight, rDark, rFire**, Hold Life, sustain WIS | priests |
| Resistance | 60–100 | 20000 | **all four base resists** | |
| Sustenance | 60–100 | 20000 | all sustains, **Hold Life**, slow digestion | |
| Regeneration | 30–100 | 600 | regeneration | |
| Wisdom | 20–90 | 500 | WIS +1+M5, sustain | |
| Resist Lightning | 10–50 | 300 | rElec | |
| Resist Acid | 10–50 | 300 | rAcid | Shop |
| Infravision | 10–70 | 200 | infravision +1+M5 | |
| Slow Digestion | 15–40 | 200 | slow digestion | Shop |
| Searching | 10–40 | 600 | searching | |
| Inertia | 15–40 | 10 | **Free Action**, but speed −3+1d2 | an emergency FA source only |
| Teleportation | 10–40 | 10 | teleportation curse | junk (useful only to escape a level fast) |
| Adornment | 10–40 | 0 | nothing | junk |

Artifact amulets: of Carlammas (CON +3, Protection from Evil activation), of
Ingwë (FA, SI, pConf, base resists, Dispel Evil), of the Dwarves (FA, SI,
regen, STR/CON +3), 'Elessar' (+2 speed, rFire, rPois, Heal 500), 'Evenstar'
(Hold Life, rDark, rCold, rNether, Restore Life Levels).

### 6.3 What to wear when

| Stage | Ring 1 | Ring 2 | Amulet |
|---|---|---|---|
| DL1–10 | Protection / Open Wounds | Resist Fire and Cold | Resist Acid / Slow Digestion |
| DL10–25 | **Free Action** (if not elsewhere) | Resist Fire and Cold, or Damage/DEX for fighters | Resist Lightning / Regeneration |
| DL25–40 | See Invisible or Resist Poison | Damage / Accuracy / a stat ring; Escaping for a caster | ESP if found; else Resistance |
| DL40+ | **Speed** (as soon as one drops) | Resist Poison until covered elsewhere | ESP / Weaponmastery / Trickery / Magi / Devotion |
| Endgame | Speed | Speed, or a ring of Power (Vilya/Nenya/Narya) | whichever amulet fills your last hole |

The rule of thumb: **jewellery is the flexible part of your kit.** Body armour,
shield and helm are hard to replace; rings and amulets fill whatever resist
or ability you are missing *right now*, and you should carry a spare (e.g.
Resist Poison, See Invisible) to swap in.

---

## 7. Light sources

`src/player-calcs.c:calc_light()` sums the light of **every** equipped item:
the light source's `LIGHT_2`/`LIGHT_3` flag plus all `LIGHT` modifiers (crowns
of Light, Gondolin weapons, artifacts…). A light source without fuel gives 0.
Necromancers (`UNLIGHT`) get one less light from each `+LIGHT` item and
actually prefer darkness — see [Chapter 3](03-magic.md).

| Light | Radius | Fuel | Weight | Depth | Store |
|---|---:|---|---:|---:|---|
| Wooden Torch | 2 | 5000 turns, burns out | 2.2 lb | 1 | General store, always |
| Lantern | 3 | up to 15000 turns, refuel with Flasks of oil (+7500 each) | 5 lb | 5 | Black market/dungeon |
| Phial of Galadriel | 4 | none | — | 5–100 (alloc 40) | Illumination activation (radius-3 light ball, 2d15 to light-sensitive monsters), 10+d10 turns |
| Star of Elendil | 4 | none | — | 30–100 | SI; Magic Mapping activation |
| Arkenstone of Thráin | 4 | none | — | 50–127 | SI, Hold Life, rLight, rDark; **Clairvoyance** (light and map the whole level) |

Fuel constants are in `constants.txt` (`fuel-torch:5000`, `fuel-lamp:15000`,
`default-lamp:7500`). Lights do not burn fuel in town during the day
(`player_update_light()`). A torch that runs out is destroyed; you get
"Your light is growing faint" when less than 50 turns remain.

Light egos (`ego_item.txt`):

| Ego | On | Depth | Gives |
|---|---|---|---|
| of Brightness | Torch, Lantern | 1–30 | light +1 (a Lantern of Brightness is radius 4) |
| (Everburning) | Lantern | 1–30 | never needs fuel |
| **of True Sight** | Lantern | 20+ | **See Invisible, pBlind** |
| of Shadows | Lantern | 10+ | stealth +d2, rLight, **light −4** (gives no light) |

Strategy:

* Buy **3–5 torches** at the start; switch to a **Lantern** as soon as you
  find or can buy one (radius 3 vs 2 makes a real difference in seeing
  monsters coming). Keep 5+ flasks of oil to refuel it — they double as your
  thrown weapon.
* The **Phial** (alloc 40 from DL5) is the most common artifact in the game
  and usually turns up before DL20. Its activation lights rooms and damages
  light-sensitive monsters (orcs, some undead).
* A *Lantern of True Sight* is a good SI source.
* Light radius matters for **spotting** monsters, for **disarming** (unlit
  locks are harder: `calc_unlocking_chance()`), and for some spells.

---

## 8. Dragon scale mail, activations and device skill

### 8.1 Dragon scale mails

Dragon armour (`type:dragon armor`) is body armour that ignores all four
elements, has a modest to-hit penalty, and can be **activated to breathe**
(`object.txt`; activation recharge `time:50` for all). Base AC is listed plus
the built-in +10 (+15 for Power).

| DSM | AC | Weight | Depth (alloc) | Resists | Breath (activation level) |
|---|---:|---:|---|---|---|
| Blue | 12+10 | 10 lb | 25+ (8) | rElec | lightning 150 (30) |
| White | 14+10 | 11 lb | 30+ (8) | rCold | frost 100 (25) |
| Black | 16+10 | 12 lb | 35+ (8) | rAcid | acid 120 (27) |
| Red | 24+10 | 16 lb | 40+ (8) | rFire | fire 200 (32) |
| Green | 20+10 | 14 lb | 60+ (8) | rPois | poison 150 (30) |
| Gold | 28+10 | 18 lb | 70+ (8) | rSound | sound 150 (30) |
| Multi-Hued | 45+10 | 26 lb | 75+ (5) | base four + rPois | random of the five, 250 (50) |
| Shining | 32+10 | 20 lb | 75+ (6) | rLight, rDark | light or dark, 200 (37) |
| Law | 36+10 | 22 lb | 80+ (6) | rSound, rShards | sound or shards, 230 (45) |
| Chaos | 40+10 | 24 lb | 85+ (5) | rChaos, rDisen | chaos or disenchant, 220 (45) |
| Balance | 50+10 | 28 lb | 95+ (3) | rSound, rShards, rChaos, rDisen | one of the four, 250 (52) |
| **Power** | 60+15 | 30 lb | 100 (1) | **all base, rPois, rLight, rDark, rSound, rShards, rChaos, rDisen, pConf** | missile 300 (57) |

DSM egos (`ego_item.txt`): *of Craftsmanship* (+1d6M4 AC, common 60),
*of Stealth* (+d2), *of Resistance* (adds the base four; Shining/Law/Gold/Chaos/
Balance only), *of Elvenkind* (base four + random high resist + stealth;
same bases), *(Dwarven)* (FA, STR/CON), and the rare ***of Speed***
(**+d5 speed**, common 2, DL40+). A Chaos or Balance DSM *of Elvenkind* or
*of Resistance* is one of the strongest body armours in the game. Artifact
DSMs: **'Razorback'** (Multi-Hued; lightning immunity, rLight/rDark/rDisen,
FA, SI, but aggravates), **'Mediator'** (Balance + rNexus, FA, regen),
**'Bladeturner'** (Power DSM with every high resist incl. nether and nexus, pFear,
pBlind, pConf, Hold Life; activation heroism + blessing + 50+d50 turns of
resist-all).

DSM breath activations ignore your spell stat and are a good ranged attack
for warriors, but at 50 turns recharge they are a supplement, not a weapon.

### 8.2 Activations

Activatable objects (artifacts, some egos, DSMs, rings of the elements, Ring of
Open Wounds and Digging) are used with `A`. Each activation has a level
(`activation.txt`) which is the **difficulty** of using it (for artifacts
the artifact's level is used instead; `get_use_device_chance()`), and a
recharge time. Some activations worth knowing about (full list in
[A4](A4-artifacts.md)):

| Activation | Effect | Found on (examples) |
|---|---|---|
| RESIST_ALL | 20+1d20 turns of temporary rAcid/rElec/rFire/rCold/rPois | 'Colluin' |
| HASTE1 / HASTE2 | +10 speed for 20+1d20 / 75+1d75 turns | of Fëanor, 'Taratol' / of Tulkas |
| HEAL1 / HEAL2 | heal 500 / 1000 HP, cure cuts | of Gondor, 'Elessar' / 'Vilya', 'Soulkeeper' |
| LOSKILL | **Mass Banishment** | of Eönwë |
| BANISHMENT | Banishment | of Celeborn |
| TELE_OTHER | teleport a monster away | of Ulmo |
| RECALL | Word of Recall | 'Avavir' |
| RECHARGE | recharge strength 10 | of Thingol |
| CLAIRVOYANCE / MAPPING / DETECT_ALL | map/detect | Arkenstone / Star of Elendil / 'Holhenneth' |
| STAR_BALL | lightning balls 150 in all directions | 'Razorback', 'Mediator' |
| RAGE_BLESS_RESIST | berserk, bless and 50+1d50 turns resist-all | 'Bladeturner' |
| BIZARRE | random powerful effect | The One Ring |

### 8.3 Device skill and failure rates

Wands, staffs, rods and activations all use the same failure formula
(`src/obj-util.c:get_use_device_chance()`, out of 1000):

```
x    = 2 * (device_skill - item_level) + 1
fail = 380 - 370 * x / (5 + |x|)
```

| Skill − item level | −20 | −10 | −5 | −1 | 0 | +5 | +10 | +20 | +30 | +50 | +100 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Fail | 70.7% | 67.2% | 61.7% | 44.1% | 31.8% | 12.6% | 8.2% | 5.1% | 3.9% | 2.8% | 1.9% |

The curve is steep around zero: getting your device skill ten points above
an item's level takes it from ~32% to ~8% failure. The floor is about 1%.

Device skill = class base + class increment × CL/10 + race + INT bonus
(`adj_int_dev`: 0 at INT ≤7, 1 at 8–14, 2 at 15–17, 3 at 18/00–18/49, 4 at
18/50, 6 at 18/90, 9 at 18/150) — `calc_bonuses()`.

| Class | Base + per 10 CL | Skill at CL30 (no race/INT) | at CL50 |
|---|---|---:|---:|
| Mage, Necromancer | 36 + 13 | 75 | 101 |
| Rogue | 32 + 10 | 62 | 82 |
| Priest | 30 + 10 | 60 | 80 |
| Ranger | 28 + 10 | 58 | 78 |
| Druid, Paladin, Blackguard | 24 + 10 | 54 | 74 |
| Warrior | 18 + 7 | 39 | 53 |

Race modifiers (`p_race.txt`): Gnome +22, High-Elf +20, Hobbit +18, Dwarf +9,
Elf +6, Dunadan +5, Kobold +5, Half-Elf +3, Human 0, Half-Orc −3, Half-Troll
−8. Temporary modifiers: Blessing and Heroism raise it, Berserk, stun,
confusion, poison, amnesia and hallucination lower it (`calc_bonuses()`).

**A warrior's device skill is too low for high-level devices**: a Half-Troll
warrior at CL40 has about 18+28−8+INT ≈ 40 skill and will fail a Rod of
Healing (level 80) roughly 70% of the time, but a Staff of Teleportation
(level 20) only about 5% of the time. Plan escapes around low-level devices.

---

## 9. Consumables: potions, scrolls, mushrooms, food

Columns: **Depth** = `alloc` range in the dungeon; **Value** = base cost in
gold (normal shops sell at this value, the Black Market at 3×; see
[§11](#11-the-stores)); **Shop** = Alchemist (5) "always" or "normal" stock,
General store (1). Effects are from the `effect:`/`dice:` lines in
`lib/gamedata/object.txt`. `HEAL_HP` heals `max(base, M% of missing HP)`
(`effect_handler_HEAL_HP`).

### 9.1 Potions

**Healing and curing**

| Potion | Depth | Value | Effect | Shop |
|---|---|---:|---|---|
| Cure Light Wounds | 1–15 | 20 | heal 20, cure blindness, −20 cuts, −20 confusion | Alchemist **always** |
| Cure Serious Wounds | 5–40 | 70 | heal 40, cure blindness, cuts, confusion | Alchemist |
| Cure Critical Wounds | 12–100 | 200 | heal 60, cure blind, cuts, poison, **stun**, amnesia, confusion | Alchemist |
| Healing | 30–100 | 800 | heal **300 or 35% of missing HP** (whichever is more), cures all of the above | |
| \*Healing\* | 40–100 | 1500 | heal **1200**, cures as above | |
| Life | 60–100 | 5000 | heal **5000**, cure everything incl. hallucination, restore all stats and experience | |
| Neutralize Poison | 1–35 | 75 | cure poison | Alchemist |
| Restore Mana | 15–100 | 350 | restore all SP | |
| Restore Life Levels | 30–100 | 400 | restore experience | Alchemist |

**Combat buffs**

| Potion | Depth | Value | Effect | Shop |
|---|---|---:|---|---|
| **Speed** | 1–100 | 75 | **+10 speed** for 20+2d10 turns (only +5 turns if already hasted) | |
| Heroism | 1–100 | 25 | heal 10, cure fear, **hero** 25+1d25 turns (+12 to-hit, no fear) | |
| Berserk Strength | 3–100 | 80 | heal 30, cure fear, **berserk** 25+1d25 turns (+75 melee skill, −10 AC, no fear) | Alchemist |
| Boldness | 1–90 | 8 | cure fear | Alchemist |
| Resist Heat / Resist Cold | 30–100 | 30 | temporary rFire / rCold, 50+1d10 turns | Alchemist |
| Resist Poison | 20–100 | 30 | temporary rPois 50+1d10 turns, cure poison | |
| True Seeing | 3–40 | 50 | cure blindness, **See Invisible** 50+2d6 turns | |
| Infravision | 3–40 | 20 | +5 infravision for 100+4d25 turns | |
| Dragon Breath | 40–100 | 100 | breathe fire **or** frost (random) for 80 | |

Timed resists **stack** with permanent ones up to "double resist" (1/9 damage
for base elements; `calc_bonuses()` raises `res_level` by one up to 2).

**Stat and knowledge**

| Potion | Depth | Value | Effect |
|---|---|---:|---|
| Strength / Intelligence / Wisdom / Dexterity / Constitution | 30–100 | 8000 | restore and **gain** one point of the stat |
| Augmentation | 40–100 | 50000 | restore and gain **all five** stats |
| Brawn / Intellect / Contemplation / Nimbleness / Toughness | 20–40 | 1000 | gain a point of one stat, **lose** a point of a random other stat |
| Experience | 65–100 | 25000 | +200000 experience (or to the next level) |
| Enlightenment | 25–100 | 800 | map and light the entire level, show objects |
| \*Enlightenment\* | 70–100 | 12000 | as Enlightenment + gain INT and WIS + detect objects, gold and all monsters on the level |

Stat potions are always drunk immediately (the stat gain is permanent). The
"exchange" potions (Brawn etc.) are a gamble that moves points from a stat you
don't need into one you do; drink them only when your weak stats are already
at a level you don't care about.

### 9.2 Scrolls

**Movement and escape**

| Scroll | Depth | Value | Effect | Shop |
|---|---|---:|---|---|
| **Phase Door** | 1–100 | 15 | short teleport (distance 10) | Alchemist **always** |
| **Teleportation** | 10–100 | 40 | long teleport (distance up to 60% of the level's larger dimension) | |
| **Teleport Level** | 20–100 | 800 | instantly up or down one level | |
| **Word of Recall** | 5–100 | 125 | after 15+1d20 turns, to town or to your recall depth | Alchemist **always** |
| **Deep Descent** | 1–100 | 250 | after 3+1d4 turns, to **max depth + 5** (`effect_handler_DEEP_DESCENT`) | |

Notes from the code (`effect-handler-general.c`):

* Word of Recall sets your recall depth: read in the dungeon at a level
  shallower than your maximum and you are asked whether to reset the recall
  depth to the current level. Reading it again while active cancels it.
* **Deep Descent is based on your maximum depth, not your current depth**:
  from DL5 with a max depth of 30 it takes you to DL35. It is a fast way to
  return to the depths but a terrible "escape" from a shallow level.
* Teleport Level is the most reliable "get out now" scroll: it is immediate.
  It cannot be used when `birth_force_descend` would take you up.

**Detection and knowledge**

| Scroll | Depth | Value | Effect | Shop |
|---|---|---:|---|---|
| Magic Mapping | 5–100 | 65 | map the local area (radius 22×40) | Alchemist |
| Treasure Detection | 0–70 | 70 | detect gold/mineral veins with treasure, sense objects | |
| Detect Invisible | 1–70 | 15 | detect invisible monsters | Alchemist |
| Light | 0–60 | 15 | light the room, 2d8 weak-light ball | |
| Identify Rune | 5–100 | 200 | learn one rune on an item | |

**Combat and protection**

| Scroll | Depth | Value | Effect | Shop |
|---|---|---:|---|---|
| Blessing | 1–50 | 15 | **bless** 6+1d12 turns (+5 AC, +10 to-hit) | Alchemist |
| Holy Chant | 10–70 | 40 | bless 12+1d24 turns | Alchemist |
| Holy Prayer | 50–100 | 80 | bless 24+1d48 turns | |
| Protection from Evil | 30–100 | 50 | 3×CL+1d25 turns; evil monsters of level ≤ your CL are repelled when `1d100 + CL > 50` (`mon-attack.c`) | |
| Monster Confusion | 5–60 | 30 | your next melee hit confuses | Alchemist |
| Dispel Undead | 40–100 | 200 | 60 damage to all undead in line of sight | |
| Rune of Protection | 70–100 | 500 | glyph of warding under you | |
| Remove Hunger | 5–100 | 10 | nourish up to 50% ("Fed") | Alchemist |
| Door Destruction | 10–60 | 50 | destroy adjacent doors | |

**The "big three" level-clearing scrolls**

| Scroll | Depth | Value | Effect |
|---|---|---:|---|
| **\*Destruction\*** | 40–100 | 1000 | wipes a radius-15 circle: every monster in it is deleted (uniques included — they are not killed and can turn up again), plus objects and terrain; **blinds you for 10+1d10 turns unless you resist light** |
| **Banishment** | 40–100 | 2500 | removes every non-unique monster of the chosen symbol on the level; you take 1d4 damage per monster removed |
| **Mass Banishment** | 50–100 | 3000 | removes every non-unique monster within view distance (20); 1d3 damage per monster |

These are the classic "something terrible is next to me" answers. *Banishment*
is the anti-hound / anti-dragon / anti-demon tool (choose `Z`, `D`/`d`, `U`);
*Mass Banishment* clears your surroundings of everything except uniques;
*\*Destruction\** also removes uniques' escorts and the terrain, but costs you
the items in the area. Banishment and Mass Banishment skip uniques
(`monster_is_unique()` checks); only *\*Destruction\** removes a unique from
your vicinity, without killing it.

**Enchanting and curses**

| Scroll | Depth | Value | Effect |
|---|---|---:|---|
| Enchant Weapon To-Hit / To-Dam | 15–70 | 125 | +1 to-hit / to-dam |
| Enchant Armour | 15–70 | 125 | +1 AC |
| \*Enchant Weapon\* | 50–100 | 500 | +1d3 to-hit and to-dam (separately rolled) |
| \*Enchant Armour\* | 50–100 | 500 | +2+1d3 AC |
| Remove Curse | 10–100 | 100 | strength 20+d20 against one curse (Alchemist **always**) |
| \*Remove Curse\* | 50–100 | 8000 | strength 50+d50 |
| Recharging | 40–100 | 200 | recharge a wand/staff (strength 6; see [§10.4](#104-recharging)); Alchemist normal stock |
| Acquirement / \*Acquirement\* | 20+ / 60+ | 5000 / 10000 | create 1 / 1+1d2 **great** items with two extra artifact rolls |

Enchantment success (`enchant_score()`): the failure chance by current bonus
is 0%, 1%, 2%, 4%, 8%, 16%, 28%, 40%, 55%, 70%, 80%, 90%, 95%, 97%, 99%
for +0 … +14, and **100% at +15 or above**. Artifacts additionally resist
half of all attempts. Enchanting is therefore cheap up to about +8 and
pointless above +12. The best use is **launchers and ammo** (to-dam is
multiplied) and Mithril/Seeker missiles.

### 9.3 Mushrooms

| Mushroom | Depth | Value | Effect |
|---|---|---:|---|
| Second Sight | 5–100 | 200 | cure blindness, **telepathy** 50+7d7 turns |
| Fast Recovery | 5–40 | 200 | heal 30, cure stun, cuts, poison, blindness |
| Vigor | 20–100 | 200 | restore all five stats |
| Clear Mind | 5–40 | 200 | restore 20 SP, cure confusion, fear, hallucination; **rConf** 50+7d7 turns |
| Emergency | 5–100 | 60 | heal 200, temporary rFire and rCold 25+5d5 turns, but **hallucination** |
| Terror | 5–40 | 60 | **+10 speed** but terrified (cannot melee) 25+5d5 turns — an escape item |
| Stoneskin | 10–40 | 60 | +40 AC, −5 speed, 50+7d7 turns (General store) |
| Debility | 10–40 | 20 | restore mana, but drain STR or CON |
| Sprinting | 15–100 | 100 | +10 speed for 100 turns, then slowed |
| Purging | 15–100 | 60 | set food to 5% (very hungry), restore STR and CON, cure poison (General store) |
| Shadows | 50–100 | 300 | +10 stealth 10+d10 turns |

Mushrooms are cheap emergency tools. **Terror** (+10 speed, cannot attack) and
**Sprinting** are speed escapes; **Second Sight** is a temporary ESP that
lets you check a vault or a unique's position; **Clear Mind** both restores
mana and gives confusion protection — superb for casters.

### 9.4 Food

| Food | Depth | Value | Effect |
|---|---|---:|---|
| Ration of Food | 0–50 | 3 | +30% food (General store **always**) |
| Hard Biscuit | — | 1 | +10% (General store) |
| Apple | 0–20 | 3 | +10% |
| Handful of Dried Fruits | 0–50 | 3 | +15% |
| Slime Mold | 30–100 | 2 | +15% (General store) |
| Slice of Meat | 10–50 | 3 | +40% |
| Honey-cake | 20–70 | 10 | nourish to 60% |
| Piece of Elvish Waybread | 30–100 | 10 | nourish to 75%, cure poison, heal 4d8 |
| Flask of Whisky | — | 1 | +3%, confusion (General store) |
| Pint of Fine Wine | — | 2 | +3%, **boldness** 80–120 turns (General store) |
| Sip of Miruvor | 10–50 | 100 | restore STR and CON |
| Swig of Orcish Liquor | 20–100 | 1 | stun and slow you |
| Draught of the Ents | 60–100 | 2000 | nourish, restore and **gain** STR and CON |
| Scrap of Flesh | 50–100 | 0 | random nourishment |

Food levels (`player_timed.txt`): Hungry below 15%, Weak below 8%, Faint
below 4%, Fed up to 90%, Full above. Carry 5 rations or a stack of Scrolls of
Remove Hunger (cheaper per weight: 0.5 lb). Slow Digestion items make food
almost irrelevant.

### 9.5 What to carry, stage by stage

The quantities below are the minimums experienced players keep; they are
consistent with the escape and readiness rules in
[Chapter 8](08-phase-strategy.md).

| Item | DL1–10 | DL10–30 | DL30–60 | DL60+ |
|---|---|---|---|---|
| Flasks of oil | 15–25 | 10–20 (until a better ranged attack) | — | — |
| Cure Light Wounds | 5+ | 5 (they cure blindness) | 2–5 | — |
| Cure Serious/Critical Wounds | — | 5+ CSW/CCW | 10+ CCW | 10+ CCW (stun/confusion cure) |
| Healing / \*Healing\* / Life | — | 1–2 Healing if found | 3–5 Healing, 1–2 \*Healing\* | 5+ \*Healing\*, Life |
| Phase Door | 10+ | 15+ | 15+ | 15+ |
| Word of Recall | 1–2 | 2–3 | 2–3 (or Rod of Recall) | Rod of Recall + 2 scrolls |
| Teleportation (scroll or staff) | — | 3–5 scrolls / Staff | Staff + 5 scrolls | Staff + scrolls |
| Teleport Level | — | 1–2 | 3–5 | 5+ |
| Deep Descent | for diving | 1–2 for diving | — | — |
| Speed potions | any found | 2–5 | 5–10 | 10–20 |
| Magic Mapping / detection | a few scrolls | Rod of Treasure Location | Rod of Detection / Magic Mapping | Rod of Detection |
| Restore Mana (casters) | — | some | 5+ | 10+ |
| Banishment / Mass Banishment / \*Destruction\* | — | — | 1–2 each if found | several of each |
| Resist Poison potions | — | 2–3 until rPois | — | — |
| Heroism / Berserk | 2–3 | 3–5 (fear cure) | some | some |
| Food | 5 rations | 5 rations / Remove Hunger | Waybread or Slow Digestion | — |

### 9.6 Bad and dangerous consumables

These have no shop value (cost 0) and exist to punish blind use
(`object.txt`); none is fatal on its own at full HP, so test-using unknown
flavours in a safe place is fine:

* Potions of **Sleep** (paralysis 5+1d5, blocked by Free Action),
  **Blindness**, **Confusion**, **Poison**, **Slowness** (−10 speed 15+1d25
  turns), **Salt Water** (food to 2%, cure poison, and 4 turns of
  *unresistable* paralysis), **Surprise** (random: scramble stats, bat form,
  drain INT or WIS, or stoneskin).
* Scrolls of **Aggravate Monster** (wakes and hastes nearby monsters),
  **Summon Monster** / **Summon Undead** (1d3 monsters), **Curse Weapon** /
  **Curse Armour**, **Darkness**, **Trap Creation**.
* Wands of **Clone Monster**, **Haste Monster**, **Heal Monster**; Staffs of
  **Summoning**, **Haste Monsters**, **Darkness**.

Never read unknown scrolls next to a sleeping unique, and never drink unknown
potions while monsters are adjacent.

---

## 10. Magic devices: wands, staffs, rods

General rules:

* **Wands** (`a`im) and **rods** that need a target can be aimed; **staffs**
  (`u`se) affect you or everything in line of sight. Devices work while
  blind (scrolls do not), which is why the Staff of Teleportation is the
  premier escape.
* Every use rolls against the device's **level** with the formula in
  [§8.3](#83-device-skill-and-failure-rates). Depth (`alloc`) and level are
  different numbers: a Wand of Fire Balls is level 55 (difficulty) and appears
  from DL50.
* Bolt-or-beam effects become beams 20% of the time from wands and 10% from
  rods (`cmd-obj.c:beam_chance()`).
* Wands and staffs have **charges**; rods have a **recharge time** (in game
  turns at normal speed — being hasted gives you more actions per recharge).
  A stack of rods recharges in parallel: with three Rods of Fire Bolts you can
  zap three times in a row (`number_charging()`).
* Wands are destroyed by lightning (`HATES_ELEC`) unless they ignore it;
  staffs by acid and fire; rods by lightning. Ball wands of the matching
  element (Lightning Balls etc.) ignore their own element.

### 10.1 Wands

| Wand | Level | Depth | Value | Charges | Effect |
|---|---:|---|---:|---|---|
| Magic Missile | 3 | 3–60 | 200 | 6+d10 | bolt/beam 3d4 (unresistable) — **Magic shop** |
| Stinking Cloud | 5 | 5–60 | 400 | 6+d8 | poison ball 12, radius 3 — **Magic shop** |
| Light | 3 | 3–60 | 200 | 6+d10 | light beam 6d8 to light-sensitive monsters — **Magic shop** |
| Darkness | 3 | 3–60 | 100 | 6+d10 | darkness beam |
| Wonder | 3 | 3–60 | 250 | 8+d15 | random wand effect (power CL/5+1d100) |
| Confuse Monster | 5 | 5–100 | 500 | 2+d6 | confuse 5+1d5 — **Magic shop** |
| Hold Monster | 5 | 5–100 | 500 | 2+d6 | hold 3+1d5 |
| Stun Monster | 5 | 5–100 | 500 | 2+d6 | stun 5+1d5 |
| Scare Monster | 10 | 10–100 | 500 | 3+d5 | frighten |
| **Stone to Mud** | 10 | 10–100 | 300 | 6+d10 | dissolve a wall (20+1d30 to wall-monsters) — **Magic shop** |
| Slow Monster | 15 | 5–100 | 500 | 2+d6 | slow 10+1d20 — **Magic shop** |
| Lightning Bolts | 15 | 15–60 | 600 | 6+d8 | lightning **beam** 6d6 |
| Frost Bolts | 20 | 20–60 | 800 | 6+d5 | frost bolt 12d8 |
| Polymorph | 20 | 20–60 | 400 | 6+d8 | polymorph (power = CL) |
| **Teleport Other** | 20 | 20–100 | 3500 | 6+d5 | teleport a monster ~100 squares away |
| Disable Traps | 20 | 20–60 | 700 | 4+d5 | destroy traps in a line |
| Fire Bolts | 30 | 30–60 | 1000 | 6+d8 | fire bolt/beam 12d8 |
| Acid Bolts | 30 | 30–60 | 950 | 6+d8 | acid bolt/beam 10d8 |
| Lightning Balls | 35 | 35–100 | 1200 | 4+d8 | lightning ball 64, radius 2 |
| Cold Balls | 40 | 40–100 | 1500 | 2+d6 | frost ball 100 |
| **Drain Life** | 50 | 50–100 | 1200 | 3+d3 | 150 damage to living monsters (no effect on undead or `NONLIVING` monsters such as golems and vortices) |
| Acid Balls | 50 | 50–100 | 1650 | 2+d5 | acid ball 120 |
| Dragon's Frost | 50 | 50–100 | 2400 | 1+d3 | cold arc 160 |
| Fire Balls | 55 | 50–100 | 1800 | 2+d4 | fire ball 144 |
| Dragon's Flame | 55 | 50–100 | 2400 | 1+d3 | fire arc 200 |
| Dragon's Breath | 60 | 60–100 | 2400 | 1+d3 | random acid/elec/fire/cold/poison arc 160 |
| **Annihilation** | 60 | 60–100 | 3000 | 1+d2 | 250 drain-life damage |
| Clone / Haste / Heal Monster | 15 / 3 / 3 | | 0 | | junk; throw away |

**Wand of Teleport Other** is the single most important wand. Any monster it
hits — uniques included — is teleported about 100 squares away; there is no
saving throw (`project_monster_handler_AWAY_ALL()` in `project-mon.c`). It is a
bolt, so it stops at the first monster in its path. With it, a
character at DL30–50 can deal with an out-of-depth breather, a nexus hound
pack leader, or a unique too strong to fight. Carry two if you can.

**Stone to Mud** is the everyday tool: dig through walls to escape, to reach
treasure, or to create a corridor to fight in. It also kills or wounds
monsters made of rock.

**Drain Life** (and Annihilation) are the best non-elemental attack wands in
the game; nothing living resists them.

### 10.2 Staffs

| Staff | Level | Depth | Value | Charges | Effect |
|---|---:|---|---:|---|---|
| Detect Invisible | 5 | 5–100 | 200 | 8+d15 | detect invisible monsters — **Magic shop** |
| Light | 5 | 5–80 | 250 | 8+d20 | light room, 2d8 weak-light ball — **Magic shop** |
| Cure Light Wounds | 5 | 5–100 | 350 | 6+d5 | as the potion |
| Confuse Monsters | 10 | 10–100 | 800 | 3+d3 | confuse all in LOS |
| Sleep Monsters | 10 | 10–100 | 700 | 6+d5 | sleep all in LOS |
| Slow Monsters | 20 | 10–100 | 800 | 4+d4 | slow all in LOS |
| **Mapping** | 20 | 10–100 | 600 | 5+d5 | map the local area — **Magic shop** |
| **Detect Evil** | 20 | 20–100 | 350 | 8+d15 | detect evil monsters — **Magic shop** |
| **Teleportation** | 20 | 20–100 | 2000 | 5+d4 | long teleport (as the scroll) — **Magic shop** |
| Starlight | 20 | 20–80 | 800 | 6+d5 | light beams in all directions 6d8 |
| Curing | 25 | 10–100 | 500 | 4+d3 | cure blind, cuts, poison, stun, confusion |
| Earthquakes | 40 | 40–100 | 350 | 3+d5 | earthquake radius 10 |
| **Speed** | 40 | 40–100 | 2000 | 4+d3 | +10 speed 20+2d10 turns |
| Remove Curse | 40 | 40–100 | 1500 | 4+d3 | remove curse strength 35+d30 |
| \*Destruction\* | 50 | 50–100 | 6000 | 1+d3 | as the scroll |
| Dispel Evil | 50 | 50–100 | 1200 | 4+d3 | 60 damage to evil in LOS |
| **Healing** | 70 | 70–100 | 5000 | 1+d2 | heal 300 / 35%, cure as the potion |
| **Banishment** | 70 | 70–100 | 5000 | 1+d2 | as the scroll |
| **Power** | 70 | 70–100 | 4000 | 1+d3 | 120 damage to **everything** in LOS |
| **Holiness** | 70 | 70–100 | 3500 | 2d2 | 120 to evil in LOS, heal 50, Protection from Evil, cure everything incl. fear, stun, slow, hallucination |
| **the Magi** | 70 | 70–100 | 4500 | 2+1d2 | restore INT and **all SP** |
| Summoning / Haste Monsters / Darkness | — | | 0 | | junk |

### 10.3 Rods

| Rod | Level | Depth | Value | Recharge | Effect |
|---|---:|---|---:|---:|---|
| **Treasure Location** | 10 | 8–75 | 1000 | 50 | detect gold & mineral veins, sense objects (local area) |
| Light | 10 | 5–100 | 500 | 9 | light beam 6d8 |
| Illumination | 20 | 10–100 | 1000 | 30 | light the room |
| Lightning Bolts | 20 | 20–80 | 2000 | 11 | lightning beam 6d6 |
| Frost Bolts | 25 | 25–80 | 2500 | 13 | frost bolt 12d8 |
| **Detection** | 30 | 50–100 | 3000 | 100 | detect gold, objects, **all monsters** (visible and invisible) |
| Probing | 30 | 35–100 | 1000 | 100 | probe monsters in LOS (HP, resists) |
| Hold Monster | 30 | 30–100 | 1500 | 40 | hold 3+1d5 |
| **Recall** | 30 | 30–100 | 4000 | 60 | Word of Recall |
| Fire Bolts | 35 | 30–80 | 3000 | 15 | fire bolt/beam 12d8 |
| **Magic Mapping** | 35 | 35–100 | 2000 | 100 | map a larger area (33×55) |
| Polymorph | 35 | 35–80 | 1200 | 25 | polymorph |
| **Teleport Other** | 35 | 45–100 | 5000 | 25 | as the wand, never runs out |
| Disable Traps | 35 | 10–100 | 2100 | 30 | destroy traps in a line |
| Acid Bolts | 40 | 40–80 | 3500 | 12 | acid bolt/beam 12d8 |
| Slow Monster | 40 | 30–100 | 1500 | 40 | slow |
| Curing | 40 | 10–80 | 1000 | 100 | cure blind, cuts, poison, stun, confusion |
| Lightning Balls | 55 | 55–100 | 4000 | 23 | ball 64 |
| **Speed** | 55 | **80–100** | 25000 | 100 | +10 speed 20+2d10 turns |
| Cold Balls | 60 | 60–100 | 4500 | 25 | ball 100 |
| Acid Balls | 70 | 70–100 | 5500 | 27 | ball 120 |
| Fire Balls | 75 | 75–100 | 5000 | 30 | ball 144 |
| **Drain Life** | 75 | 75–100 | 3600 | 23 | 150 drain |
| **Healing** | 80 | 80–100 | 20000 | 1000 | heal 500, cure cuts and stun |
| **Restoration** | 80 | 80–100 | 25000 | 1000 | restore all stats and experience |

Rods are never used up and cannot be recharged by scrolls; they are the
backbone of a mature character's toolkit. The key ones, in the order you
should hope to find them:

1. **Rod of Treasure Location** (DL8+): gold detection pays for everything in
   [§11](#11-the-stores) and shows where objects are.
2. **Rod of Recall** (DL30+): frees you from carrying recall scrolls.
3. **Rod of Detection** (DL50+): all monsters, objects and gold in one zap —
   the most important information tool in the game. Until you find it, use a
   Rod of Treasure Location + Staff of Detect Evil + Detect Invisible, or ESP.
4. **Rod of Teleport Other** (DL45+), **Rod of Magic Mapping**,
   **Rod of Illumination**.
5. **Rod of Healing** / **Restoration** / **Speed** (DL80+), best stacked.

### 10.4 Recharging

`effect_handler_RECHARGE()` and `recharge_failure_chance()` (`effects.c`):

```
strength  = 6 (Scroll of Recharging), 10 (of Thingol), or the spell's power
ease      = (100 - item level) / 10                  (9 for level-10 items, 3 for level-70)
backfire  = 1 in (strength + ease - 2 * charges_per_item)   (always if that is <= 1)
on success: charges += 2 + 1d(strength / (10 - ease) + 1)
```

A backfire **destroys one item** of the stack. Examples with a Scroll of
Recharging:

| Item | Charges before | Backfire chance | Charges gained |
|---|---:|---:|---|
| Wand of Stone to Mud (lvl 10) | 0 | 1 in 15 (6.7%) | 2+1d7 |
| Staff of Teleportation (lvl 20) | 0 | 1 in 14 (7.1%) | 2+1d4 |
| Staff of Teleportation | 3 | 1 in 8 (12.5%) | 2+1d4 |
| Wand of Teleport Other (lvl 20) | 0 | 1 in 14 | 2+1d4 |
| Wand of Drain Life (lvl 50) | 0 | 1 in 11 (9.1%) | 2+1d2 |
| Staff of the Magi (lvl 70) | 0 | 1 in 9 (11%) | 2+1d1 (= 3) |

Rules: **recharge only empty (or nearly empty) devices**, because every
remaining charge makes failure much more likely; split a stack before
recharging if you can't afford to lose the whole thing; a Mage's recharge
spell is stronger than the scroll. The Inspect screen of a wand or staff shows
the backfire chance next to each item while you are choosing what to
recharge (`ui-object.c`).

### 10.5 Must-have devices

| Device | Why | When |
|---|---|---|
| **Wand of Stone to Mud** | escape, digging, corridor-making | from the Magic shop at once |
| **Staff of Teleportation** | the escape that works blind/confused | buy as soon as you can afford 2000 gold (Magic shop) |
| **Wand of Magic Missile / Stinking Cloud** | early ranged attack for non-casters | DL1–15 |
| **Staff of Detect Evil + Mapping** | situational awareness before ESP | DL5–30 |
| **Rod of Treasure Location** | gold → stores | DL10+ |
| **Wand of Teleport Other** | removes any problem monster | DL20+ |
| **Rod of Recall** | no recall scrolls needed | DL30+ |
| **Staff of Speed** / Potions of Speed | +10 speed on demand | DL40+ |
| **Wand/Rod of Drain Life** | top single-target device damage | DL50+ |
| **Rod of Detection** | full detection | DL50+ |
| **Staff of Banishment / Power / Holiness** | clear rooms, heal and cure | DL70+ |
| **Staff of the Magi** (casters) | full mana refill | DL70+ |
| **Rod of Healing** | renewable 500 HP heal | DL80+ |

---

## 11. The stores

### 11.1 The town

`lib/gamedata/store.txt` defines seven shops and your home. "Always" items
are staples that are restocked whenever they sell out (`store_is_staple()`);
"normal" items are drawn at random from the store's list.

| # | Store | Always stocks | Sometimes stocks | Buys (if selling is on) |
|---|---|---|---|---|
| 1 | **General Store** | Rations, Wooden Torches, **Flasks of oil**, Cloaks, Shovel, Pick, Iron Shots, Arrows, Bolts | Fine Wine, Slime Molds, Whisky, Hard Biscuits, mushrooms of Stoneskin and Purging | food, lights, flasks, diggers, cloaks, ammo |
| 2 | **Armoury** | — | sandals/leather/iron-shod boots, leather/metal caps, iron helm, robe to Leather Scale Mail, Metal Scale and Chain Mail, leather gloves, gauntlets, wicker/small metal/leather shields | all armour |
| 3 | **Weaponsmith** | — | daggers to bastard swords and katanas, spear to lucerne hammer, whip; sling, short bow, long bow, light crossbow; shots, arrows, bolts | weapons, launchers, ammo, diggers |
| 4 | **Bookseller** | every **town** book of all four realms | — | books |
| 5 | **Alchemist** | **Cure Light Wounds**, **Phase Door**, **Word of Recall**, Remove Curse | CSW, CCW, Neutralize Poison, Restore Life Levels, Boldness, Berserk, Resist Heat/Cold; Magic Mapping, Detect Invisible, Recharging, Remove Hunger, Blessing, Holy Chant, Monster Confusion | potions, scrolls |
| 6 | **Magic shop** | — | rings of Searching, Feather Falling, Protection, Open Wounds; amulets of Slow Digestion, Resist Acid; wands of Light, Slow/Confuse Monster, Magic Missile, Stinking Cloud, **Stone to Mud**; staffs of **Teleportation**, Light, **Mapping**, Detect Invisible, **Detect Evil** | jewellery, devices, magic books |
| 7 | **Black market** | — | random items of DL `max_depth+5` to `max_depth+20` (ego, enchanted, or kinds no other shop has) | — (pays half) |
| 8 | **Home** | your storage (24 item slots, `store:inven-max:24`) | | |

The town books (`class.txt`): Magic — *First Spells* (25 g), *Attacks and
Knowledge* (400), *Magical Defences* (1600); Nature — *Lesser Charms* (25),
*Gifts of Nature* (400); Divine — *Novice's Handbook* (25), *Cleansing Power*
(100); Shadow — *Into the Shadows* (25), *Dark Rituals* (400), *Fear and
Torment* (1600). Dungeon books are only found in the dungeon (see
[Chapter 3](03-magic.md)).

### 11.2 Stock quality improves with your depth

`store_create_random()` generates normal-store items at a level between 1 and
`5 + max(max_depth − 20, 0)` (capped at 70), and Black Market items at
`max_depth + 5` to `max_depth + 20` (min capped at 55, max at 70). Items with
negative bonuses or curses are rejected. So:

* Until your max depth passes DL20, shops only roll level-1–5 items.
* At max depth 40 the Weaponsmith and Armoury can roll level-25 items, which
  includes "good" (+hit/+dam/+AC) and occasionally ego weapons/armour
  (`apply_magic()` is called with that level). It is worth checking the
  shops after each trip deeper.
* The **Black Market** at max depth 30+ regularly offers Potions of Speed,
  Healing, egos and rods — at triple price.

Stores turn over part of their stock once per game "day" — every
10 × 1000 game turns (`store:turns:1000`, `game-world.c`) spent in the dungeon;
the changes are applied when you return. A shop's owner changes with a
1-in-25 chance per day (`store:shuffle:25`).

### 11.3 Prices, selling and `birth_no_selling`

`src/store.c:price_item()`:

* Normal shops **sell** at the item's value; the **Black Market sells at 3×**
  (`price × 2`, then the 150% adjustment).
* There is **no charisma** stat and no racial price adjustment in 4.2.
* Shops **buy** at 2/3 of value (the Black Market at 1/3), capped by the
  owner's purse (5000 to 30000 gold depending on owner; listed in
  `store.txt`).
* **`birth_no_selling` is ON by default** (`list-options.h`). With it, shops
  pay **0 gold** for anything you "sell" (you can still give items away), and **gold found in the dungeon is multiplied by
  5** (`obj-make.c:make_gold()`, applied when `player->depth > 0`).

With no-selling, your income is the gold you pick up. That makes treasure
detection (Rod of Treasure Location, Scrolls of Treasure Detection) and digging
gold veins much more valuable, and loot hauling pointless — don't carry heavy
items back to town for money.

**Selling (or giving) an item to a shop teaches you every rune on it**, and
buying one does too: `do_cmd_sell()` and `do_cmd_buy()` (`src/store.c`) call
`object_learn_unknown_rune()` until the item is fully known, and make its
flavour known. Since runes are global ([§2.2](#22-how-rune-based-id-works)),
handing a shop one unwanted ego item with an unknown rune (say, a cloak of
Aman with an unidentified high resist) teaches you that rune for good. This is
the cheapest identification in the game.

### 11.4 The essential town shopping list

| Phase | Buy |
|---|---|
| Character creation (with `birth_start_kit`) | a few Flasks of oil (General), 5 CLW (Alchemist), 5+ Phase Door, a Soft Leather Armour/cap/boots if gold remains |
| First return (CL5–10) | Flasks of oil to 20+, Lantern (if the Black Market has one), Long Bow or Sling + ammo, **Wand of Stone to Mud**, Wand of Magic Missile/Stinking Cloud (non-casters), CSW |
| CL15–25 | **Staff of Teleportation**, Staff of Detect Evil / Mapping, 2nd town spellbook, Word of Recall spares, Potions of CCW |
| CL25+ | Magical Defences / Fear and Torment etc.; Scrolls of Recharging; check the Black Market for Speed/Healing potions, egos and rods |

---

## 12. Artifacts

### 12.1 Standard and random artifacts

There are 138 standard artifacts (`artifact.txt`; full table in
[Appendix A4](A4-artifacts.md)). With `birth_randarts` on, the game builds a new
set with the same slots and similar total power (`src/obj-randart.c`, using
the power formulas of `src/obj-power.c`); everything in this section about
specific names then no longer applies, but the slot-by-slot principles do.
`birth_no_artifacts` removes artifacts entirely.

**Each artifact exists once.** When you leave a level (`generate.c:
forget_cave()`):

* an artifact you **had seen** (known) but left on the floor is **lost
  forever** ("history_lose_artifact");
* an artifact you **never saw** is returned to the pool and can be generated
  again later;
* with `birth_lose_arts` every artifact left behind is lost, and the level
  feeling gives a special warning ("you sense an item of wondrous power!").

So: never leave a level with an artifact you have seen lying on the floor.

### 12.2 Early "gift" artifacts

Several artifacts have high allocation weights and shallow depths, so most
characters find some of them before DL20:

| Artifact | Slot | Alloc, depth | What it does for you |
|---|---|---|---|
| **Phial of Galadriel** | light | 40, DL5+ | radius 4, no fuel, room-lighting activation |
| 'Narthanc' / 'Nimthanc' / 'Dethanc' | dagger | 40 each, DL1–50 | +9/+12, brand, rFire/rCold/rElec, bolt activation |
| 'Forasgil' | rapier | 38, DL5–50 | 1d12 +12/+19, cold brand, slay animal, rCold, rLight |
| 'Elvagil' | long sword | 50, DL10–50 | slay orc/troll, SI, feather fall, DEX/stealth +2 |
| 'Paurhach' / 'Pauraegen' / 'Paurnen' | gauntlets | 40/23/30, DL10–75 | a base resist and a bolt activation |
| 'Cammithrim' | gloves | 18, DL10–80 | FA, rLight, magic missile activation |
| of Thorongil | cloak | 10, DL5–75 | **FA, SI, pFear**, rAcid |
| of Thengel / 'Holhenneth' | helm | 18, DL10/20+ | pConf, WIS / SI, pBlind, pConf, detection |
| 'Hithlomir' | soft armour | 20, DL20–80 | base four + rDark + rNether, stealth +3 |
| of Arvedui | chain mail | 30, DL20–75 | base four + rShards + rNexus |
| of Himring | hard leather | 20, DL25–90 | rPois, rChaos, rNether; Protection from Evil |

### 12.3 The most important artifacts by slot

(Abbreviations: FA Free Action, SI See Invisible, ESP telepathy.)

| Slot | Top choices (standard set) |
|---|---|
| Weapon | 'Ringil', 'Andúril', of Eönwë, 'Deathwreaker', of Ulmo, of Durin, of Aulë, 'Aranrúth', 'Sting' (early-mid) — see [§3.8](#38-endgame-weapons) |
| Launcher | 'Belthronding', of Bard, 'Cubragol' (+10 speed), of Umbar |
| Ring | 'Vilya', 'Nenya', 'Narya'; of Tulkas; of Barahir |
| Amulet | 'Elessar' (+2 speed, Heal), of Ingwë, of the Dwarves, 'Evenstar' |
| Light | Arkenstone of Thráin, Star of Elendil, Phial |
| Body | 'Bladeturner' (every high resist), 'Mediator', 'Razorback', of Isildur, of Celeborn, 'Soulkeeper', 'Belegennon' |
| Cloak | 'Colluin', of Thingol, of Lúthien, of Tuor (acid immunity, FA, SI), 'Colannon' |
| Shield | of Thorin, of Celegorm, of Anárion, of Gil-galad, of Elros |
| Helm/crown | of Dor-Lómin, of Thranduil (ESP), of Gondor, of Númenor, 'Holhenneth' |
| Gloves | of Fingolfin (FA, DEX +4, +10/+10), 'Cambeleg', of Eöl (rPois, fire immunity, mana bolt — but aggravates) |
| Boots | **of Fëanor** (+15 speed), 'Stormwalker' (+6), 'Dal-i-thalion' (+5) |

### 12.4 The Rings of Power and the quest items

* **'Narya'** (Ring of Fire; DL70+): +5 speed, all stats +1, FA, SI, ESP,
  regeneration, slow digestion, **fire immunity**, pFear; Protection from
  Evil activation.
* **'Nenya'** (Ring of Adamant; DL80+): +5 speed, all stats +2, FA, SI, ESP,
  Hold Life, feather fall, regeneration, pBlind, **cold immunity**; Restore
  Life Levels activation.
* **'Vilya'** (Ring of Firmament; DL90+): +5 speed, all stats +3, FA, SI, ESP,
  Hold Life, **lightning immunity**, rPois, rDark; **Heal 1000** activation.
* **'The One Ring'** (Ring of Power; DL100, alloc 1): +5 speed, all stats +5,
  +15/+15, immunity to all four base elements, rPois, rDark, rDisen, rNether,
  ESP, SI, regeneration — and **aggravation, experience drain and STICKY**
  (cannot be removed once worn); activation BIZARRE (random strong effect,
  level 80). It is a gamble: it fixes four immunities and gives you a free
  ESP slot, but aggravation wakes the entire level.
* **'Grond'** (Mighty Hammer, 9d9 +5/+25) and **the Massive Iron Crown of
  Morgoth** (+50 to all stats, ESP, SI, nearly every resist, STICKY) are
  `QUEST_ART`: they are dropped only by Morgoth (`mon-make.c:mon_create_drop()`),
  i.e. after you have already won.

### 12.5 How artifacts are generated

* **Normal artifacts** are rolled when an item of the right base is generated
  as a *great* item (`make_artifact()`): 1 roll for a great item, 2 if the
  drop was forced great (`DROP_GREAT`), +2 more for uniques' drops and
  Acquirement. For each candidate artifact of that base: it must not exist
  yet; if the level is shallower than its minimum depth it passes only with
  probability `1/(2 × (min_depth − depth))`; it may never appear below its max
  depth; then it must pass its `alloc` percentage roll.
* **Special artifacts** (lights, the Rings of Power and other jewellery
  artifacts — their bases have `INSTA_ART`) are tried first in
  `make_object()`: 1 in 1000 for an ordinary object, **1 in 10 for a good
  drop**; if the roll is made but no artifact qualifies, the item is upgraded
  to *good*.
* No artifacts are generated in town or in stores.

---

## 13. How items are generated: good, great, ego and artifact rolls

### 13.1 Floor and monster objects

`src/obj-make.c:make_object()` and `apply_magic()`:

1. **Kind selection.** The kind is drawn from the allocation table at the
   object level (`get_obj_num()`). With probability 1 in 20 (`great-obj`) the
   level is boosted to `1 + level × 100 / 1d100` — an out-of-depth item. A
   *good* drop draws at `level + 10` from a table restricted to "good" kinds
   (weapons, armour, launchers, arrows/bolts and anything flagged `GOOD` such as
   Rings of Speed and the 70+ amulets; `kind_is_good()`).
2. **Quality roll.** An object is *good* with probability `33 + level` %
   (forced if the drop is good), and a good object is *great* with probability
   **30%** (forced if the drop is great).
3. **Artifact rolls** for great objects (above).
4. **Ego roll.** Every great object that isn't an artifact becomes an **ego
   item** if any ego fits the base and level (`make_ego_item()` →
   `ego_find_random()`). Egos have their own min/max depth with a small
   out-of-depth chance.
5. **Magic.** Good weapons get +1d5+m_bonus(5) to-hit and to-dam; great ones
   another m_bonus(10) each, and possibly super-charged dice
   (`apply_magic_weapon()`). Armour similarly gets +1d5+m_bonus(5) AC, +m_bonus(10)
   if great.
6. **Curses**: 1 in 20 wearables get curses (BLESSED items are immune).

Consequences: at DL10 an item is good 43% of the time and great 13%; at DL40
good 73%, great 22%; from DL67 every item is at least good and 30% are great.
This is why ego items become common in the 30s and why deep-diving is the
fastest way to gear up.

### 13.2 Monster drops

`src/mon-make.c:mon_create_drop()`:

* The drop level is `max((monster level + depth)/2, monster level)`, so killing
  out-of-depth monsters rewards you with deeper items.
* **Uniques** use `min(level + 15, level × 2)` as their level and get the
  two extra artifact rolls (`extra_roll`).
* `DROP_GOOD` monsters drop only good items; `DROP_GREAT` monsters drop only
  great items (so every non-artifact item they drop is an ego). `ONLY_ITEM` /
  `ONLY_GOLD` restrict the type; `DROP_40`, `DROP_60`, `DROP_1`… set counts;
  some monsters have specific drops (e.g. potions from alchemist-type
  monsters).
* Monster lore (`/` or `r` in the monster recall) tells you what a monster
  drops once you have seen enough kills; the [unique cards](A2-uniques.md) list
  every unique's drop flags.

### 13.3 Level feelings for objects

With `birth_feelings` on, after you have seen 10 squares of a level
(`feeling-need:10`) you get the object half of the level feeling
(`generate.c:calc_obj_feeling()`, text in `cmd-cave.c`). The feeling is the
sum of `(item value/100)²` of everything generated, divided by depth:

| Rating / depth | Feeling |
|---|---|
| artifact with `birth_lose_arts` | "you sense an item of wondrous power!" |
| > 160000 | "there are superb treasures here." |
| > 40000 | "there are excellent treasures here." |
| > 10000 | "there are very good treasures here." |
| > 2500 | "there are good treasures here." |
| > 640 (or any artifact) | "there may be something worthwhile here." |
| > 160 | "there may not be much interesting here." |
| > 40 | "there aren't many treasures here." |
| > 10 | "there are only scraps of junk here." |
| else | "there is naught but cobwebs here." |

An artifact anywhere on the level guarantees at least "something
worthwhile". "Superb" or "excellent" feelings usually mean a vault or an ego
with a very high value (a Ring of Speed is worth 100000) and are worth
exploring carefully. "Scraps of junk" means you can take the next stairs
without regret.

---


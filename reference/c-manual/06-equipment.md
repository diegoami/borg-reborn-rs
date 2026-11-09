# 6. Equipment

Everything you wear or wield: the twelve body slots, every base weapon and armour kind with its
numbers, lights, every ring and amulet, ego items and artifacts, how items are generated, the 4.2
rune-based identification system, curses, ignoring, a per-depth "what should I be wearing" table,
swap strategy, and a numeric scoring method a borg can use to compare two items.

FACT statements cite the game source (`lib/gamedata/*.txt`, `src/*.c`, relative to the game source
root, angband master at `e473720ed`). ADVICE is marked as such and is judgement, not source.

## Contents

1. [Equipment slots](#1-equipment-slots)
2. [Weight, burden, and the caster armour limit](#2-weight-burden-and-the-caster-armour-limit)
3. [Melee weapons](#3-melee-weapons)
4. [Digging tools](#4-digging-tools)
5. [Launchers and ammunition](#5-launchers-and-ammunition)
6. [Body armour](#6-body-armour)
7. [Shields, helms and crowns, gloves, boots, cloaks](#7-shields-helms-and-crowns-gloves-boots-cloaks)
8. [Lights](#8-lights)
9. [Rings](#9-rings)
10. [Amulets](#10-amulets)
11. [Ego items](#11-ego-items)
12. [Artifacts and randarts](#12-artifacts-and-randarts)
13. [How objects are generated](#13-how-objects-are-generated)
14. [Rune-based identification](#14-rune-based-identification)
15. [Curses](#15-curses)
16. [Ignoring and throwing out junk](#16-ignoring-and-throwing-out-junk)
17. [What to wear at each stage](#17-what-to-wear-at-each-stage)
18. [Swap items](#18-swap-items)
19. [Comparing two items, and a scoring function for a borg](#19-comparing-two-items-and-a-scoring-function-for-a-borg)
20. [Things to try (untested)](#things-to-try-untested)

Related chapters: [Chapter 1](01-core-mechanics.md) (to-hit, AC, blows, light),
[Chapter 4](04-combat.md) (slays, brands, criticals, blows tables),
[Chapter 5](05-elements-and-status.md) (what each resistance does),
[Chapter 7](07-consumables-and-devices.md) (activations, enchant scrolls),
[Chapter 11](11-town-and-shopping.md) (buying gear). Full tables:
[A4 artifacts](appendices/A4-artifacts.md), [A5 ego items](appendices/A5-ego-items.md),
[A6 object kinds](appendices/A6-object-kinds.md),
[A10 curses](appendices/A10-curses-traps-timed-effects.md).

---

## 1. Equipment slots

FACT: the player body is defined in `lib/gamedata/body.txt`. There is one body, "Humanoid", with
twelve slots in this order:

| # | Slot type | Label | What goes there |
|---|-----------|-------|-----------------|
| 1 | WEAPON | weapon | swords, hafted, polearms, diggers (a digger can be wielded as a weapon) |
| 2 | BOW | shooting | slings, bows, crossbows |
| 3 | RING | right hand | any ring |
| 4 | RING | left hand | any ring |
| 5 | AMULET | neck | amulets |
| 6 | LIGHT | light | torch, lantern, light artifacts |
| 7 | BODY_ARMOR | body | soft armour, hard armour, dragon scale mail |
| 8 | CLOAK | back | cloaks |
| 9 | SHIELD | arm | shields |
| 10 | HAT | head | helms and crowns |
| 11 | GLOVES | hands | gloves and gauntlets |
| 12 | BOOTS | feet | boots |

There is no two-handed weapon rule: a shield can always be worn with any weapon. There is no
off-hand weapon slot. The quiver is not an equipment slot; it is part of the pack (see
[Chapter 7](07-consumables-and-devices.md#18-stacking-pack-and-quiver-limits)).

Notes:

* Equipment bonuses from weapon and bow to-hit/to-dam do *not* add to your general to-hit/to-dam;
  they only apply to attacks made with that weapon or launcher. To-hit/to-dam on every *other*
  slot (rings, gloves, amulets, body armour penalties) adds to all melee and missile attacks
  (`src/player-calcs.c:calc_bonuses()`).
* AC from every slot adds up, including AC on weapons, rings and amulets.
* Light radius from every slot adds up (`src/player-calcs.c:calc_light()`), so a Ring of Light or
  a Crown of Light adds to your lantern.
* Sticky items (flag `STICKY`, only The One Ring and the Massive Iron Crown of Morgoth in the
  standard set) cannot be removed (`src/obj-util.c:obj_can_takeoff()`). Cursed items in 4.2 can
  otherwise be taken off freely; curses are runes with effects, not "can't remove" locks.

---

## 2. Weight, burden, and the caster armour limit

### 2.1 Carrying weight

FACT (`src/player-calcs.c:calc_bonuses()`, `weight_limit()`, table `adj_str_wgt[]`): your weight
limit is `adj_str_wgt[STR] * 100` tenth-pounds. You start losing speed when total carried weight
exceeds **half** the limit, at -1 speed per 10% of the limit above that.

| STR | adj_str_wgt | Limit (lb) | Slowdown starts above (lb) | -1 speed per extra (lb) |
|-----|-------------|-----------|----------------------------|-------------------------|
| 8 | 10 | 100 | 50 | 10 |
| 12 | 14 | 140 | 70 | 14 |
| 16 | 18 | 180 | 90 | 18 |
| 18 | 20 | 200 | 100 | 20 |
| 18/50 and up | 30 | 300 | 150 | 30 |

`weight_remaining()` (shown by the `C` screen / inventory header) uses `60 * adj_str_wgt` (i.e.
0.6 of the limit), so the displayed "remaining" figure is slightly conservative.

ADVICE: a low-STR caster in Metal Scale Mail (25 lb) plus a stack of flasks of oil (2 lb each) is
already slowed. Count weight before buying heavy armour; a -1 speed penalty is worse than 10 AC.

### 2.2 Heavy weapons and launchers

FACT: `hold = adj_str_hold[STR]` (in pounds). If `weapon weight / 10 > hold` you are "heavy
wielding": to-hit drops by `2 * (weight_lb - hold)` and you get no extra blows (blows stay at the
minimum) and no digging bonus from the weapon. The same rule applies to a launcher (no extra
shots or might while too heavy) (`src/player-calcs.c:calc_bonuses()`).

| STR | 3 | 8 | 10 | 12 | 14 | 16 | 17 | 18 | 18/20 | 18/50 | 18/100 | 18/150 | 18/200 |
|-----|---|---|----|----|----|----|----|----|-------|-------|--------|--------|--------|
| hold (lb) | 4 | 10 | 14 | 18 | 22 | 26 | 28 | 30 | 35 | 50 | 80 | 90 | 100 |

Blows depend on weapon weight versus STR and DEX through the blows table and each class's
`min-weight` and `max-attacks` (`lib/gamedata/class.txt`; `src/player-calcs.c:calc_blows()`). See
[Chapter 4](04-combat.md) for the blows table; the practical rule is that light weapons give more
blows until your STR is high.

### 2.3 The caster armour weight limit (spell weight)

FACT (`src/player-calcs.c:calc_mana()`): for a spellcasting class, the game sums the weight of
everything worn **except** weapon, bow, rings, amulet and light (so: body armour, cloak, shield,
helm, gloves, boots). If that total exceeds the class `spell_weight` (third field of `magic:` in
`lib/gamedata/class.txt`), maximum SP is reduced by **1 per full pound over**, and you get the
"encumbered" message.

| Class | spell_weight (tenth-lb) | Armour weight allowance |
|-------|------------------------|-------------------------|
| Mage | 300 | 30 lb |
| Necromancer | 300 | 30 lb |
| Druid | 350 | 35 lb |
| Priest | 350 | 35 lb |
| Rogue | 350 | 35 lb |
| Paladin | 400 | 40 lb |
| Ranger | 400 | 40 lb |
| Blackguard | 600 | 60 lb |
| Warrior | none (no spells) | - |

Example: a Mage wearing Leather Scale Mail (14 lb) + Leather Shield (9 lb) + Iron Helm (5 lb) +
Leather Boots (2 lb) + Cloak (1 lb) + Leather Gloves (1 lb) = 32 lb is 2 lb over and loses 2 SP.

**Glove penalty: none in this version.** Older versions (3.x, 4.1) reduced a spellcaster's mana for
wearing gloves without Free Action or DEX. FACT: there is no such code in 4.2/master: the only
armour-related mana rule is the weight limit above (`src/player-calcs.c:calc_mana()`; there is
no glove-related player flag in `src/list-player-flags.h`). Casters may wear any gloves. Gloves of
Free Action and Gloves of Slaying are good choices for everyone.

### 2.4 The body armour to-hit penalty

FACT: heavy body armour carries a negative to-hit in its `attack:` line (`lib/gamedata/object.txt`),
e.g. Metal Scale Mail `1d4:-2:0`, Full Plate `2d4:-3:0`, Adamantite Plate `2d4:-4:0`. Because body
armour is not the weapon slot, that to-hit penalty applies to all your melee and missile attacks
(`src/player-calcs.c:calc_bonuses()`). The official manual shows it as the `(-#)` before the
AC brackets ([user-manual/attack.rst](user-manual/attack.rst), "Your Armor Class").

---

## 3. Melee weapons

FACT: all base melee weapons from `lib/gamedata/object.txt` ("level" is the native depth used for
device difficulty and for "good" rolls; "alloc" is commonness and depth range; weight in lb).
Average damage per blow before bonuses is `dice * (sides+1) / 2`.

### 3.1 Swords (tval `sword`; base flag HATES_ACID only)

| Weapon | Dice | Avg | Weight | Alloc depth | Store? | Notes |
|--------|------|-----|--------|-------------|--------|-------|
| Dagger | 1d4 | 2.5 | 1.2 | 0+ | Weaponsmith | THROWING flag; good thrown weapon early |
| Main Gauche | 1d5 | 3 | 3.0 | 3+ | Weaponsmith | |
| Rapier | 1d6 | 3.5 | 4.0 | 5+ | Weaponsmith | Mage start weapon |
| Short Sword | 1d7 | 4 | 8.0 | 5+ | Weaponsmith | |
| Cutlass | 1d8 | 4.5 | 11.0 | 5+ | Weaponsmith | |
| Tulwar | 2d4 | 5 | 10.0 | 5+ | Weaponsmith | Blackguard start weapon |
| Broad Sword | 2d5 | 6 | 15.0 | 10+ | Weaponsmith | |
| Long Sword | 2d5 | 6 | 13.0 | 10+ | Weaponsmith | |
| Scimitar | 4d2 | 6 | 15.0 | 10+ | Weaponsmith | |
| Bastard Sword | 3d4 | 7.5 | 14.0 | 15+ | Weaponsmith | |
| Katana | 3d5 | 9 | 12.0 | 20+ | Weaponsmith | best weight/damage ratio of the common swords |
| Zweihander | 3d6 | 10.5 | 20.0 | 30+ | no | |
| Executioner's Sword | 4d5 | 12 | 26.0 | 40+ | no | |
| Blade of Chaos | 6d5 | 18 | 18.0 | 70+ | no | grants rChaos (RES_CHAOS on the base kind) |

### 3.2 Hafted weapons (tval `hafted`; HATES_ACID and HATES_FIRE)

Priests and Paladins get +2 to-dam with hafted weapons or BLESSED weapons (`BLESS_WEAPON` player
flag, `src/player-calcs.c:calc_bonuses()`).

| Weapon | Dice | Avg | Weight | Alloc depth | Store? | Notes |
|--------|------|-----|--------|-------------|--------|-------|
| Whip | 1d3 | 2 | 3.0 | 3+ | Weaponsmith | Druid start weapon |
| Mace | 2d4 | 5 | 12.0 | 5+ | no | Priest start weapon |
| War Hammer | 3d3 | 6 | 12.0 | 5+ | no | |
| Maul | 4d4 | 10 | 20.0 | 5+ | no | very heavy for its depth |
| Great Hammer | 8d1 | 8 | 18.0 | 20+ | no | fixed damage dice; Aulë's base |
| Quarterstaff | 1d9 | 5 | 15.0 | 10+ | no | |
| Morning Star | 2d6 | 7 | 15.0 | 10+ | no | |
| Flail | 2d6 | 7 | 15.0 | 10+ | no | |
| Lead-Filled Mace | 4d3 | 8 | 18.0 | 15+ | no | |
| Ball-and-Chain | 2d4 | 5 | 15.0 | 20+ | no | |
| Throwing Hammer | 2d4 | 5 | 5.5 | 40+ | no | THROWING |
| Two-Handed Great Flail | 3d6 | 10.5 | 28.0 | 45+ | no | |
| Mace of Disruption | 5d8 | 22.5 | 40.0 | 80+ | no | slay undead (x3) on the base kind |

### 3.3 Polearms (tval `polearm`; HATES_ACID and HATES_FIRE)

| Weapon | Dice | Avg | Weight | Alloc depth | Store? | Notes |
|--------|------|-----|--------|-------------|--------|-------|
| Spear | 1d6 | 3.5 | 5.0 | 5+ | Weaponsmith | THROWING |
| Trident | 1d10 | 5.5 | 7.0 | 5+ | Weaponsmith | light for its damage |
| Awl-Pike | 1d8 | 4.5 | 16.0 | 10+ | Weaponsmith | |
| Lance | 2d8 | 9 | 30.0 | 10+ | Weaponsmith | very heavy |
| Lucerne Hammer | 2d5 | 6 | 12.0 | 10+ | Weaponsmith | |
| Pike | 2d5 | 6 | 16.0 | 15+ | Weaponsmith | |
| Beaked Axe | 2d6 | 7 | 18.0 | 15+ | Weaponsmith | |
| Broad Axe | 2d6 | 7 | 16.0 | 15+ | Weaponsmith | |
| Battle Axe | 2d8 | 9 | 17.0 | 15+ | Weaponsmith | |
| Glaive | 2d6 | 7 | 19.0 | 20+ | no | |
| Halberd | 3d5 | 9 | 19.0 | 25+ | no | |
| Throwing Axe | 2d3 | 4 | 4.0 | 25+ | no | THROWING |
| Great Axe | 4d4 | 10 | 23.0 | 40+ | no | |
| Lochaber Axe | 3d8 | 13.5 | 25.0 | 45+ | no | |
| Scythe | 5d3 | 10 | 25.0 | 45+ | no | |
| Scythe of Slicing | 8d4 | 20 | 25.0 | 60+ | no | |

### 3.4 How to pick a melee weapon

FACT: damage per turn is roughly `blows * (dice_avg * slay_multiplier + to_dam)` modified by
criticals; blows come from weight vs STR/DEX (`src/player-calcs.c:calc_blows()`), and criticals
depend on weight and to-hit ([Chapter 4](04-combat.md)).

ADVICE:

* Early (CL1-15) the number of blows dominates. A light weapon that gives one more blow beats a
  heavier one with bigger dice. The Inspect (`I`) screen shows blows and average damage per turn
  for any weapon you are carrying or looking at, so compare there rather than by dice.
* A weapon with **(+to-dam)** matters more than its dice once you have 3+ blows, because to-dam is
  added per blow and is not multiplied by slays.
* Brands and slays multiply only the dice (`src/player-attack.c`), so they matter more on big-dice
  weapons. A Blade of Chaos or a Mace of Disruption with a brand is a late-game monster.
* Weight limit: stay under `hold` (2.2). A weapon you cannot hold gives 1 blow and a to-hit penalty.

---

## 4. Digging tools

FACT (`lib/gamedata/object.txt`, `src/player-calcs.c`): diggers are weapons (tval `digger`) that
add digging skill. Each `DIG_n` level is +20 digging skill, and the TUNNEL modifier adds +20 per
point. Wielded weapons also add `weight/10` to digging (when not heavy-wielded).

| Digger | Dice | Weight | Digging | Alloc depth | Store |
|--------|------|--------|---------|-------------|-------|
| Shovel | 1d2 | 6.0 | DIG_1 (+20) | 5+ | General Store (always) |
| Pick | 1d3 | 18.0 | DIG_2 (+40) | 30+ | General Store (always) |
| Mattock | 1d8 | 25.0 | DIG_3 (+60) | 50+ | no |

Digging chances per turn, out of 1600 (`src/player-calcs.c:calc_digging_chances()`): rubble
`skill*8`, magma `(skill-10)*4`, quartz `(skill-20)*2`, granite `(skill-40)*1`, doors
`(skill*4-119)/3`.

Egos: **of Digging** (TUNNEL +d4, acid brand), **of Earthquakes** (STR +d4, TUNNEL +d6, IMPACT,
acid brand, +d10/+d10). Artifacts: Pick of Erebor, Mattock of Náin (activation: stone to mud every
2 turns). Ring of Digging gives TUNNEL 2+d3 and activates for stone to mud.

ADVICE: you do not need to wield a digger. Swap one in (`w`), dig, swap back. A Shovel is cheap
enough to carry from CL1 for digging through rubble; magma with treasure is worth it only if
your skill is high. A Wand of Stone to Mud (Magic Shop) is far faster. The Pick costs about 176
gold in the General Store (observed price, see [Chapter 11](11-town-and-shopping.md)).

---

## 5. Launchers and ammunition

### 5.1 Launchers

FACT (`lib/gamedata/object.txt`; multiplier is the `pval`):

| Launcher | Ammo | Multiplier | Weight | Alloc depth | Store |
|----------|------|------------|--------|-------------|-------|
| Sling | shots/pebbles | x2 | 0.5 | 1+ | Weaponsmith |
| Short Bow | arrows | x2 | 3.0 | 3+ | Weaponsmith (Ranger start) |
| Long Bow | arrows | x3 | 4.0 | 10+ | Weaponsmith |
| Light Crossbow | bolts | x3 | 11.0 | 15+ | Weaponsmith |
| Heavy Crossbow | bolts | x4 | 20.0 | 30+ | no |

Extra shots are stored in tenths: `num_shots` starts at 10 (1 shot) and `SHOTS[n]` adds n tenths;
Rangers add `level/3` tenths (`FAST_SHOT`, `src/player-calcs.c:calc_bonuses()`). `MIGHT[n]` adds
to the multiplier.

### 5.2 Ammunition

| Ammo | Dice | Alloc depth | Notes |
|------|------|-------------|-------|
| Rounded Pebble | 1d2 | 0+ | THROWING, break 0% (base `shot`) |
| Iron Shot | 1d4 | 3+ | THROWING, break 0% |
| Mithril Shot | 2d4 (+5,+5) | 40+ | ignores acid/fire |
| Arrow | 1d4 | 3+ | break 35% |
| Seeker Arrow | 4d4 | 55+ | |
| Mithril Arrow | 3d4 | 50+ | ignores acid/fire |
| Bolt | 1d5 | 3+ | break 20% |
| Seeker Bolt | 4d5 | 65+ | |
| Mithril Bolt | 3d5 | 60+ | ignores acid |

Break chances come from `lib/gamedata/object_base.txt`; on a miss the chance is squared/100
(`src/player-attack.c:breakage_chance()`). Shots never break.

Launcher egos (`lib/gamedata/ego_item.txt`): **of Accuracy** (+d15 hit, min +15), **of Power**
(+d15 dam, min +15), **of Extra Might** (+1 multiplier), **of Extra Shots** (+2d3 tenths, min
+2), **of Lothlórien** (bows only: FA, DEX, +MIGHT d2), **of the Haradrim** (light crossbow:
+1 might, +2d3 shots), **of Buckland** (sling: DEX, +2d5 shots, +MIGHT d2), and the cursed-ish
**of the Nazgûl** (DRAIN_EXP, SI).

Ammo egos: Slay Animal/Evil/Undead/Demon/Giant/Dragon, Acid, Lightning, Flame, Frost, Venom,
Wounding (+3+d5 hit and dam), Holy Might (mithril/seeker only: slay evil x2, demon x3, undead x3),
and Backbiting (strongly negative).

ADVICE: every class should carry a launcher in the shooting slot because it costs nothing to
wear and lets you soften monsters as they approach. Warriors and Rangers rely on it; a Sling for
casters is fine. A x3 bow with (+10,+10) ammo out-damages most early melee.

---

## 6. Body armour

FACT (`lib/gamedata/object.txt`): base AC, to-hit penalty (applies to all attacks, 2.4), weight.

### 6.1 Soft armour (HATES_ACID and HATES_FIRE)

| Armour | AC | To-hit | Weight | Alloc depth | Store |
|--------|----|--------|--------|-------------|-------|
| Robe | 2 | 0 | 2.0 | 1+ | Armoury (only base for "of Permanence") |
| Soft Leather Armour | 8 | 0 | 8.0 | 3+ | Armoury (Warrior/Rogue start) |
| Studded Leather Armour | 12 | -1 | 10.0 | 5+ | Armoury |
| Hard Leather Armour | 16 | -1 | 12.0 | 10+ | Armoury |
| Leather Scale Mail | 20 | -1 | 14.0 | 15+ | Armoury |

### 6.2 Hard armour (HATES_ACID only)

| Armour | AC | To-hit | Weight | Alloc depth | Store |
|--------|----|--------|--------|-------------|-------|
| Mithril Chain Mail | 28 | -1 | 15.0 | 55+ | no (ignores acid) |
| Chain Mail | 32 | -2 | 22.0 | 20+ | Armoury |
| Metal Scale Mail | 38 | -2 | 25.0 | 25+ | Armoury |
| Partial Plate Armour | 40 | -3 | 26.0 | 30+ | no |
| Augmented Chain Mail | 42 | -2 | 27.0 | 35+ | no |
| Bar Chain Mail | 45 | -2 | 28.0 | 40+ | no |
| Metal Brigandine Armour | 48 | -3 | 29.0 | 45+ | no |
| Metal Lamellar Armour | 58 | -3 | 34.0 | 50+ | no |
| Mithril Plate Mail | 60 | -3 | 30.0 | 65+ | no (ignores acid) |
| Full Plate Armour | 62 | -3 | 36.0 | 55+ | no |
| Ribbed Plate Armour | 66 | -3 | 38.0 | 60+ | no |
| Adamantite Plate Mail | 80 | -4 | 42.0 | 75+ | no (ignores acid) |

### 6.3 Dragon scale mail (no "hates", ignores all four base elements)

FACT: every DSM has a base +10 to-AC (`armor:x:10`; Power +15), the resist of its dragon, and a
breath activation (time 50 turns). Activation difficulty uses the kind level, not the dragon's
depth (`src/obj-util.c:get_use_device_chance()`).

| DSM | AC | Resists | Breath (damage) | Kind level | Alloc depth |
|-----|----|---------|-----------------|-----------|-------------|
| White | 14 | rCold | cold 100 | 25 | 30+ |
| Black | 16 | rAcid | acid 120 | 27 | 35+ |
| Blue | 12 | rElec | elec 150 | 30 | 25+ |
| Green | 20 | rPois | poison 150 | 30 | 60+ |
| Gold | 28 | rSound | sound 150 | 30 | 70+ |
| Red | 24 | rFire | fire 200 | 32 | 40+ |
| Shining | 32 | rLight, rDark | light or dark 200 | 37 | 75+ |
| Law | 36 | rSound, rShards | sound or shards 230 | 45 | 80+ |
| Chaos | 40 | rChaos, rDisen | chaos or disen 220 | 45 | 85+ |
| Multi-Hued | 45 | 4 base + rPois | one of five elements 250 | 50 | 75+ |
| Balance | 50 | rSound, rShards, rChaos, rDisen | one of four 250 | 52 | 95+ |
| Power | 60 (+15) | 4 base, rPois, rLight, rDark, rSound, rShards, rChaos, rDisen, pConf | missile 300 | 57 | 100 only |

All DSMs weigh 10-30 lb, much lighter than plate for their AC, and have a -2 (Power -3) to-hit.
DSM egos: of Craftsmanship (+1d6M4 AC, min +4), of Stealth, of Resistance (base 4, high DSMs
only), of Elvenkind (high DSMs: base 4 + random high resist + stealth), (Dwarven) (FA, STR, CON,
infra), of Speed (+d5 speed, very rare).

ADVICE: Multi-Hued DSM is the classic body armour of the late game for melee characters: 4 base
resists plus poison, 45+10 AC, 26 lb. For casters, weight (2.3) usually limits you to MHDSM or
less; check the encumbrance number.

---

## 7. Shields, helms and crowns, gloves, boots, cloaks

### 7.1 Shields (HATES_ACID)

| Shield | AC | Weight | Alloc depth | Store |
|--------|----|--------|-------------|-------|
| Wicker Shield | 2 | 3.0 | 1+ | Armoury |
| Small Metal Shield | 5 | 6.0 | 15+ | Armoury |
| Leather Shield | 8 | 9.0 | 10+ | Armoury (Blackguard start) |
| Large Metal Shield | 12 | 12.0 | 30+ | no |
| Knight's Shield | 15 | 16.0 | 50+ | no |
| Mithril Shield | 16 (+10) | 10.0 | 70+ | no, ignores acid |

Shield egos: Resist Acid/Lightning/Fire/Cold, **of Resistance** (all 4 base, +d10 AC),
**of Elvenkind** (4 base + random high resist + stealth), **of Preservation** (sustain
STR/CON/DEX, hold life, rDisen, random high resist), **(Dwarven)** (Small Metal Shield only:
FA, STR, CON). Warriors, Paladins and Blackguards have `SHIELD_BASH` (shield used in melee, see
[Chapter 4](04-combat.md)).

### 7.2 Helms and crowns (HATES_ACID)

| Headgear | AC | Weight | Alloc depth | Store |
|----------|----|--------|-------------|-------|
| Hard Leather Cap | 2 | 2.0 | 3+ | Armoury |
| Metal Cap | 3 | 2.5 | 10+ | Armoury |
| Iron Helm | 7 | 5.0 | 20+ | Armoury |
| Steel Helm | 9 | 6.0 | 40+ | no |
| Iron Crown | 0 | 2.0 | 45+ | no |
| Golden Crown | 0 | 3.0 | 45+ | no, ignores acid |
| Jewel Encrusted Crown | 0 | 4.0 | 50+ | no, ignores acid |

Helm egos (any helm): of Intelligence, of Wisdom (+d2 and sustain), **of Seeing** (pBlind, SI,
search), of Infravision, **of Telepathy** (ESP; DL40+), **of Regeneration**.
Crown egos: **of the Magi** (INT +d3, sustain INT, 4 base resists, random power), **of Might**
(STR/DEX/CON +d3, their sustains, FA; DL40+), **of Lordliness** (WIS, pFear, random power),
**of Light** (+1 light, rLight), **of Serenity** (pConf, pFear, pStun, rSound), **of Night and Day**
(SI, pBlind, +1 light, rLight, rDark).

ADVICE: a Helm of Seeing is a cheap early answer to both blindness and invisible monsters. A Helm
of Telepathy is one of the best upgrades in the game for any character without another ESP source.

### 7.3 Gloves (HATES_ACID and HATES_FIRE)

| Gloves | AC | Notes | Alloc depth | Store |
|--------|----|-------|-------------|-------|
| Set of Leather Gloves | 1 | base for Thievery | 1+ | Armoury |
| Set of Gauntlets | 3 | | 10+ | Armoury |
| Set of Caestus | 5 | +3 to-dam on the base kind | 20+ | no |
| Set of Mithril Gauntlets | 6 | ignores acid and fire | 40+ | no |
| Set of Alchemist's Gloves | 0 (+5+d5) | ignores all 4 base elements; base for Thievery | 50+ | no |

Glove egos: **of Free Action** (DL1-60, alloc 100: the commonest early FA source),
**of Slaying** (+d5/+d5), **of Agility** (DEX +d5), **of Power** (STR +d5, +d5/+d5),
**of Thievery** (leather/alchemist: FA, feather fall, DEX, search), **of Combat** (gauntlet types:
STR, CON, +d5/+d8, but AGGRAVATE).

As noted in 2.3, gloves do not hurt spellcasting in this version.

### 7.4 Boots (HATES_ACID and HATES_FIRE)

| Boots | AC | Weight | Alloc depth | Store |
|-------|----|--------|-------------|-------|
| Pair of Leather Sandals | 1 | 1.5 | 1+ | Armoury |
| Pair of Leather Boots | 2 | 2.0 | 3+ | Armoury |
| Pair of Iron Shod Boots | 4 | 3.5 | 5+ | Armoury; ignores fire |
| Pair of Steel Shod Boots | 7 | 6.0 | 20+ | no; ignores fire |
| Pair of Mithril Shod Boots | 8 | 4.0 | 60+ | no; ignores acid/fire |
| Pair of Ethereal Slippers | 0 | 0 | 30+ | no; ignores all 4 |

Boot egos: of Slow Descent (feather fall), of Stealth (light boots only), **of Free Action**,
**of Stability** (feather fall, rNexus), **of Speed** (+2+M8, min 1; alloc 3, DL20+),
**of Elvenkind** (leather/iron shod: stealth, +4+M5 speed, feather; DL60+).

### 7.5 Cloaks (HATES_ACID and HATES_FIRE)

| Cloak | AC | Notes | Alloc depth | Store |
|-------|----|-------|-------------|-------|
| Cloak | 1 | | 1+ | General Store (always) |
| Fur Cloak | 3 | | 20+ | no |
| Elven Cloak | 6 | +1 stealth, +1 speed on the base kind | 40+ | no |
| Ethereal Cloak | 0 (+10) | ignores all 4 | 70+ | no |

Cloak egos: **of Protection** (rShards, ignores elements, +d10 AC), of Stealth, **of Aman**
(stealth, random high resist, +d20 AC), **of the Magi** (INT, stealth, sustain INT, random power).

---

## 8. Lights

### 8.1 Light kinds

FACT (`lib/gamedata/object.txt`, `lib/gamedata/artifact.txt`, `src/player-calcs.c:calc_light()`): the game
reports a light's *intensity* L; you see by your own light grids at distance `< L`
(`src/cave-view.c:update_view_one()`), so the lit radius is L − 1.

| Light | Intensity (lit radius) | Fuel | Notes |
|-------|------------|------|-------|
| Wooden Torch | 2 (`LIGHT_2`), lights adjacent grids only (radius 1) | 5000 turns, BURNS_OUT (destroyed when it runs out) | General Store always; cost 2 |
| Lantern | 3 (`LIGHT_3`), radius 2 | refillable with Flasks of oil: max 15000, store/new 7500 | DL5+; ignores fire |
| Phial of Galadriel | 4 (`LIGHT[4]`, no fuel), radius 3 | none | DL5+, alloc 40: the commonest artifact; activates ILLUMINATION (lights the room and a radius-3 2d15 weak-light ball), recharge 10+d10 |
| Star of Elendil | 4, no fuel, radius 3 | none | DL30+; See Invisible; activates MAPPING (map area), 50+d50 |
| Arkenstone of Thráin | 4, no fuel | none | DL50+; SI, Hold Life, rLight, rDark; activates CLAIRVOYANCE (light/map the level), 50+d50 |

Fuel constants are in `lib/gamedata/constants.txt` (`fuel-torch:5000`, `fuel-lamp:15000`,
`default-lamp:7500`); a Flask of oil has `pval:7500` and refuelling caps at 15000
(`src/cmd-obj.c:do_cmd_refill()`).

How the radius is computed (`src/player-calcs.c:calc_light()`): for every equipped item,
`LIGHT_2` gives 2, `LIGHT_3` gives 3, plus its LIGHT modifier; a fuelled light with zero fuel
gives 0. The amounts from **all** slots are summed. So Lantern (3) + Ring of Light (+1) = 4.
Characters with `UNLIGHT` (Necromancers) get one less from each +LIGHT item.

Burning (`src/player-util.c:player_update_light()`): fuel drops by 1 each player turn at normal
speed (each `process_world`, i.e. every 10 game turns), except in town in daytime and for NO_FUEL
lights. Warnings start below 100 turns. While blind the last turn of fuel is preserved. A torch
that reaches 0 is destroyed.

Light egos (`lib/gamedata/ego_item.txt`):

| Ego | On | Effect |
|-----|----|--------|
| of Brightness | torch or lantern | +1 light intensity, i.e. +1 radius (DL1-30, alloc 50) |
| (Everburning) | lantern | NO_FUEL: never needs oil (DL1-30, alloc 100) |
| of True Sight | lantern | pBlind and See Invisible (DL20+, alloc 10) |
| of Shadows | lantern | -4 light (so negative radius), +d2 stealth, rLight (DL10+) |

ADVICE:

* Get a Lantern as soon as you can. FACT: no normal store stocks lanterns (the General Store's
  only light is the Wooden Torch, `lib/gamedata/store.txt`); they come from the dungeon (DL5+,
  alloc 70, as common as torches) or the Black Market. Until then carry 3-5 torches. Flasks of oil are both lantern fuel and the best early thrown weapon
  (see [Chapter 7](07-consumables-and-devices.md)).
* A **Lantern of True Sight** solves See Invisible and blindness protection in one slot that
  nothing else competes for; keep it even over the Phial until you have SI elsewhere.
* An (Everburning) Lantern of intensity 3 vs the Phial's 4: the Phial wins (+1 radius and an
  activation), but Everburning removes the need to carry oil.
* Necromancers want *no* light (unlight); see [Chapter 14](14-class-playbooks.md).

---

## 9. Rings

FACT: all ring kinds from `lib/gamedata/object.txt`. Ring tval has HATES_ELEC (rings in the pack can
be destroyed by lightning, see [Chapter 7](07-consumables-and-devices.md#19-element-damage-to-carried-items)).
"Alloc" is depth range; values like `1+M5` grow with generation level (`M5` adds 0-5, more at depth).

| Ring | Effect (source values) | Alloc | Kind cost | Verdict (ADVICE) |
|------|------------------------|-------|-----------|------------------|
| Protection | +5+d5M10 to-AC | DL10-40 | 500 | early filler; Magic Shop |
| Searching | search +d4M2 (each point = +5 search skill) | DL5-90 | 250 | minor; you notice a trap only if your search skill is at least the trap's power (`src/trap.c`) |
| Feather Falling | FEATHER (no fall damage, float over trap doors) | DL10-50 | 200 | filler; Magic Shop |
| Open Wounds | IMPAIR_HP (slower HP regen); activation: heal 40, cure blind/cut/confusion, recharge 10+5d8 | DL10-50 | 700 | an early **swap**: an unlimited CLW-plus every ~30 turns; Magic Shop |
| Teleportation | +2 speed; curse "teleportation" power 100 (random teleport, d100-turn timer) | DL10-50 | 1000 | the curse is permanent (power 100 cannot be removed); a dive/escape tool for the reckless, see 15 |
| Escaping | +2 speed; AFRAID (permanent fear: no melee, -20 to-hit, +8 AC, device skill -5%, +20 raw spell fail) | DL15-80 | 4000 | for archers and device users who never melee; casters pay +20 fail on spells not far below their level; useless for melee |
| Reckless Attacks | +2+d3 hit and dam, AC -8+4d3 | DL10-50 | 200 | early melee filler |
| the Mouse | DEX +d4, stealth +d4, to-dam -3d5 | DL10-50 | 100 | casters and stealthy rogues; bad for melee |
| the Dog | pFear, stealth -2 | DL10-50 | 100 | cheap fear immunity for melee; stealth hit |
| Slow Digestion | SLOW_DIGEST | DL10-50 | 250 | filler |
| Digging | TUNNEL 2+d3; activation stone to mud (20+1d30 to wall monsters), recharge 15+2d10 | DL10-50 | 800 | utility swap |
| Resist Fire and Cold | rFire, rCold | DL10-50 | 250 | the standard early resist ring |
| Free Action | FREE_ACT | DL20+ | 1500 | essential by DL20-25 if you lack FA elsewhere |
| Damage | +5+d3M7 to-dam | DL20+ | 500 | strong melee ring |
| Accuracy | +10+d6M7 to-hit | DL20+ | 500 | |
| Flames / Acid / Ice / Lightning | resist that element, +5+d5M10 AC; activation: ball (80/70/75/85 dmg, radius 2) plus temporary resist 20+d20; recharge 50+d50 | DL20+ | 500 | good: a permanent resist plus a free temp resist for double resistance |
| Light | +1 light, rLight, search +1+M5 | DL25+ | 400 | blindness from light breath protection |
| Strength / Intelligence / Dexterity / Constitution | stat +1+M5 and its sustain | DL30+ | 500 | good stat rings; INT/WIS rings for casters |
| Bodykeeping | sustain STR, DEX, CON | DL30+ | 750 | |
| Soulkeeping | sustain STR, INT, WIS | DL30+ | 600 | |
| See Invisible | SEE_INVIS | DL30+ | 340 | essential if no other SI |
| Resist Poison | rPois | DL40+ | 16000 | the main mid-game rPois source |
| Slaying | +d10M10 hit, +d5M5 dam | DL40+ | 1000 | |
| Speed | +4+M6 speed, plus extra +1 per successful coin flip at generation (`src/obj-make.c:apply_magic()`) | DL75+ | 100000 | the best ring in the game |

Artifact rings: of Barahir (+1 all stats and stealth, rPois, rDark), of Tulkas (STR/DEX/CON +4,
haste activation), Narya, Nenya, Vilya, and The One Ring (+5 everything, +5 speed, many resists,
AGGRAVATE, DRAIN_EXP, STICKY). See [A4](appendices/A4-artifacts.md).

ADVICE on the ring tradeoffs:

* **Open Wounds**: IMPAIR_HP slows natural regeneration, which is worse for warriors who rest a lot.
  Use it as a *swap*: wear another ring normally, put on Open Wounds to activate when needed.
* **Escaping**: the item flag AFRAID blocks every melee attack ("You are too afraid to attack",
  `src/player-attack.c`), gives -20 to-hit and +8 AC and -5% device skill
  (`src/player-calcs.c:calc_bonuses()`), and adds **+20 to the raw spell failure chance**
  before the minimum-fail clamp (`src/player-spell.c:spell_chance()`), so well-mastered spells
  that are already at minimum fail may be unaffected while new or hard spells get much worse.
  Firing missiles and using devices still work. Protection from fear does not cancel the flag
  (BRAVERY_30 only grants PROT_FEAR, `src/player.c`). Best for an archer, or a high-level caster
  whose escape spells are deeply mastered; check the spell menu's fail column with it on. See [Chapter 5](05-elements-and-status.md).
* **Teleportation**: the curse power is 100, which `src/effect-handler-general.c:uncurse_object()`
  treats as permanent. The +2 speed is real. Random teleports every ~d100 turns make it a crude
  "keep moving" tool but they disrupt fights and can drop you next to something nasty. Not
  recommended except as an experiment ([Things to try](#things-to-try-untested)).
* Two resist rings (Flames + Ice) give rFire, rCold and two different temporary resists.

---

## 10. Amulets

FACT: amulet kinds from `lib/gamedata/object.txt`. Amulets have no "hates" flags; they are never
destroyed by elements in the pack.

| Amulet | Effect | Alloc | Kind cost | Verdict (ADVICE) |
|--------|--------|-------|-----------|------------------|
| Adornment | nothing | DL10-40 | 0 | junk; ignore |
| Teleportation | curse "teleportation" power 100 | DL10-40 | 10 | junk |
| Inertia | FREE_ACT, speed -3+1d2 (-2 or -1) | DL15-40 | 10 | emergency FA only; usually junk |
| Infravision | infra +1+M5 | DL10-70 | 200 | junk |
| Resist Lightning | rElec | DL10-50 | 300 | early filler; Magic Shop |
| Resist Acid | rAcid | DL10-50 | 300 | early filler; Magic Shop |
| Slow Digestion | SLOW_DIGEST | DL15-40 | 200 | filler; Magic Shop |
| Searching | search +d5M5 | DL10-40 | 600 | minor |
| Wisdom | WIS +1+M5, sustain WIS | DL20-90 | 500 | priests, paladins |
| Regeneration | REGEN | DL30+ | 600 | good for warriors; exactly cancels a Blackguard's intrinsic IMPAIR_HP |
| ESP | TELEPATHY | DL60+ | 25000 | excellent |
| Resistance | rFire, rCold, rAcid, rElec | DL60+ | 20000 | excellent filler of base resists |
| Sustenance | sustain all five stats, hold life, slow digestion | DL60+ | 20000 | |
| the Magi | FA, SI, sustain INT, pBlind, INT +1+M3, search, +d5M5 AC | DL70+ | 30000 | best caster amulet |
| Devotion | hold life, sustain WIS, WIS +1+M3, +1 light, rDark, rLight, rFire | DL70+ | 30000 | priests |
| Weaponmastery | +1+M4 hit and dam, STR +1+M2, sustain STR/CON, FA, pFear, rDisen | DL70+ | 30000 | best melee amulet |
| Trickery | DEX +1+M4, speed +1+M2, stealth +1+M2, search, infra, rNexus, rPois, sustain DEX | DL70+ | 30000 | rogues, archers; rPois and speed are valuable for anyone |

Note on Regeneration vs IMPAIR_HP (`src/player-util.c:player_regen_hp()`): REGEN doubles the
HP regeneration rate and IMPAIR_HP halves it, so one of each cancels. Regeneration is zero while
paralysed, poisoned, stunned or cut. The Blackguard class has intrinsic IMPAIR_HP
(`obj-flags:IMPAIR_HP` in `lib/gamedata/class.txt`), so a Blackguard with an Amulet of Regeneration
regenerates at the normal rate. See [Chapter 1](01-core-mechanics.md).

Artifact amulets: of Carlammas (CON +3, STR +1, rFire, prot-from-evil activation), of Ingwë
(INT/WIS +3, SI, FA, pConf, 3 base resists, dispel evil), of the Dwarves (STR/CON +3, SI, FA,
regen, pFear, light), Elessar (STR/WIS/speed +2, rFire, rPois, heal 500 activation),
Evenstar (hold life, regen, rDark, rCold, rNether, restore life levels).

---

## 11. Ego items

FACT: egos are defined in `lib/gamedata/ego_item.txt`; the full table is in
[A5](appendices/A5-ego-items.md). An object becomes an ego only if it rolled "great" (power 2)
in `src/obj-make.c:apply_magic()` and `ego_find_random()` finds an ego whose item list includes
the object's kind and whose depth range fits (an out-of-depth ego is allowed with chance
`1 in max(2, (alloc_min - level)/3)`). See 13 for the probabilities.

Random extra powers (`src/obj-make.c:ego_apply_magic()`):

* `RAND_SUSTAIN`: one random sustain.
* `RAND_POWER`: one random "protection" or "misc" flag (e.g. FA, SI, pBlind, pConf, feather, regen,
  slow digestion...).
* `RAND_BASE_RES`: one random base resist.
* `RAND_HI_RES`: one random high resist (pois, light, dark, sound, shards, nexus, nether, chaos,
  disen).
* `RAND_RES_POWER`: 1/3 a random power, 2/3 a random base resist.

### 11.1 The important egos per slot

| Slot | Ego | Why it matters |
|------|-----|----------------|
| Weapon | (Holy Avenger) | +d6/+d6, slay evil x2, undead x3, demon x3, SI, BLESSED, pFear, WIS, random sustain |
| Weapon | (Defender) | FA, SI, feather, regen, 4 base resists, stealth, random sustain, +d8 AC (DL10-80) |
| Weapon | of Westernesse | FA, SI, STR/DEX/CON +d2, slay orc/troll/giant x3 (DL10-70) |
| Weapon | of Gondolin | SI, FA, rDark, +1 light, slay demon/orc/troll/dragon x3, random power |
| Weapon | of Extra Attacks | +d2 blows: often the best damage ego |
| Weapon | of Fury | STR, +d2 blows, pFear but AGGRAVATE (DL50+, rare) |
| Weapon | (Blessed) | BLESSED, WIS, random power: priests |
| Weapon | brands (Acid, Lightning, Flame, Frost, Venom) | x3 dice vs non-resistant; elemental ones also give the resist |
| Weapon | *Slay X* | upgraded slays with a random resist/power and a stat |
| Launcher | of Extra Might / Extra Shots / Power | damage |
| Body | of Resistance | all four base resists (DL10+, alloc 50) |
| Body | of Elvenkind | 4 base + random high resist + stealth |
| Body | of Permanence | Robe only: 4 base, all sustains, hold life, random high resist |
| Shield | of Resistance / Elvenkind / Preservation | resists |
| Helm | of Seeing, of Telepathy, of Regeneration | SI+pBlind; ESP; regen |
| Crown | of the Magi, of Might, of Serenity, of Night and Day | see 7.2 |
| Cloak | of Aman, of Protection, of the Magi | high resist; rShards; INT |
| Gloves | of Free Action, of Slaying, of Power, of Thievery | FA; damage |
| Boots | of Free Action, of Stability, of Speed, of Elvenkind | FA; rNexus; speed |
| Light | of True Sight, (Everburning), of Brightness | SI+pBlind; no fuel; +1 radius |

Egos that are traps: **of Morgul** (weapon: SI, hold life, poison brand, slay undead but
AGGRAVATE and DRAIN_EXP), **of the Nazgûl** (launcher: DRAIN_EXP), **of Fury** and gloves
**of Combat** (AGGRAVATE), ammo **of Backbiting** (to-hit/dam -26+d25 each).

---

## 12. Artifacts and randarts

### 12.1 How artifacts appear

FACT (`src/obj-make.c`):

* **Special artifacts** (lights, rings and amulets whose kind has `INSTA_ART`: the Phial, Star,
  Arkenstone, all artifact rings and amulets) are tried first: `make_object()` calls
  `make_artifact_special()` with chance **1 in 10 for a "good" drop, 1 in 1000 otherwise**. Each
  candidate must pass a depth check (if its min depth is above the current depth it survives only
  `1 in (2 * levels_short)`), a max-depth check (strict), and a rarity roll `d100 <= alloc_prob`.
  If the special roll fails the object is upgraded to "good".
* **Normal artifacts** are rolled in `apply_magic()` only for an object of the *right base kind*
  (e.g. 'Ringil' can only come from a Long Sword). The object gets **1 artifact roll if it is
  "great"** (power 2), **2 rolls if forced great** (a DROP_GREAT monster or *Acquirement*), and
  **+2 rolls with `extra_roll`** (uniques' drops and acquirement scrolls). Each roll walks the
  artifact list and applies the same depth and rarity checks.
* No artifacts in town; none with `birth_no_artifacts`. An artifact can exist only once
  (`is_artifact_created()`); with `birth_lose_arts` an artifact left on a level is gone.
* Artifacts are never sold by stores (store items are generated with `allow_artifacts=false`,
  `src/store.c:store_create_random()`).

### 12.2 Notable standard artifacts to hope for

FACT values from `lib/gamedata/artifact.txt` (alloc = probability:depth range).

| Artifact | Why | Alloc |
|----------|-----|-------|
| Phial of Galadriel | light 4, no fuel, Illumination activation | 40: DL5-100 |
| Dagger 'Narthanc' / 'Nimthanc' / 'Dethanc' | early brand + resist + bolt activation | 40: DL1-50 |
| Rapier 'Forasgil' | 1d12 (+12,+19), cold brand, slay animal, rCold, rLight | 38: DL5-50 |
| Long Sword 'Elvagil' | (+12,+12), SI, feather, DEX, stealth, slay troll/orc | 50: DL10-50 |
| Short Sword 'Sting' | +2 blows, +3 speed, FA, SI, pFear, slay evil/undead/orc/animal | 6: DL20+ |
| Cutlass 'Gondricam' | 4 base resists, FA, SI, regen, +50 AC | 12: DL20-65 |
| Gauntlets 'Paurhach' etc. | resist + bolt activation; common | 40/23/30: DL10-75 |
| Leather Gloves 'Cammithrim' | FA, magic missile activation every 2 turns | 18: DL10-80 |
| Cloak of Thorongil | FA, SI, pFear, rAcid | 10: DL5-75 |
| Soft Leather 'Hithlomir' | 4 base, rDark, rNether, stealth +3 | 20: DL20-80 |
| Chain Mail of Arvedui | 4 base, rShards, rNexus, STR +2 | 30: DL20-75 |
| Helm 'Holhenneth' | pBlind, pConf, SI, INT/WIS, detect-all activation | 18: DL20-100 |
| Boots of Wormtongue | +1 moves (MOVES), feather, trap immunity, phase activation | 8: DL15-80 |
| Boots 'Dal-i-thalion' | +5 speed, FA, pConf, rNether | 4: DL50-100 |
| Boots of Fëanor | +15 speed, rNexus, haste activation | 1: DL40+ |
| Long Sword 'Ringil' | +10 speed, slay demon x5, cold brand, cold ball | 1: DL20+ |
| Long Sword 'Andúril' | STR/DEX +4, fire brand, rFire, rDisen, fire ball | 3: DL20+ |
| Mithril Chain 'Belegennon' | 4 base + rPois, DAM_RED 5, phase door every 2 turns | 30: DL55+ |
| Power DSM 'Bladeturner' | all resists, regen, hold life, pFear/pBlind/pConf | 6: DL100+ |
| Amulet 'Elessar' / 'Evenstar' | heal / restore life | 2 |

### 12.3 Randarts (`birth_randarts`)

FACT: `src/obj-randart.c` regenerates every artifact except The One Ring and the quest artifacts
(Grond and the Massive Iron Crown, flag `QUEST_ART`) with new names, bases and powers. Power per
tval is sampled around the standard set's average/min/max for that tval, and allocation
probability and depth are recomputed from power, so the count, findability and power "match the
standard set approximately" ([user-manual/option.rst](user-manual/option.rst)). Randart
activations use the activation's own level for device difficulty
(`src/obj-util.c:get_use_device_chance()`: artifact level first, then activation level).

ADVICE: with randarts, knowledge of specific artifacts (a borg's lookup tables) is useless;
evaluate every artifact by its properties (19). A randart Phial-equivalent may not exist, so
carry fuel until you find a NO_FUEL light.

---

## 13. How objects are generated

### 13.1 Kind selection

FACT (`src/obj-make.c:make_object()`, `get_obj_num()`):

* Each kind has `alloc:commonness:min to max`. The choice is weighted by commonness among kinds
  legal at the generation level.
* **Level boost**: with chance 1 in `great-obj` (20, `lib/gamedata/constants.txt`) the level is
  replaced by `1 + level * 100 / d100`, which can be far out of depth.
* A "good" request uses level+10 and only kinds for which `kind_is_good()` is true: all armour
  and weapons/launchers/diggers without negative base bonuses, arrows and bolts, and anything
  flagged GOOD (Ring of Speed, Amulets of the Magi/Devotion/Weaponmastery/Trickery).
* Books the player can't read are rerolled (up to 3 tries, 1/5 kept anyway).
* Monster drops use level `max((monster_level + depth)/2, monster_level)`; uniques use monster
  level `min(level+15, level*2)` and get the extra artifact rolls
  (`src/mon-make.c:mon_create_drop()`).

### 13.2 Quality rolls in `apply_magic()`

FACT (`src/obj-make.c:apply_magic()`):

* `good_chance = 33 + level` (percent), `great_chance = 30`. Power 1 ("good") if
  `d100 < good_chance`; then power 2 ("great") if `d100 < 30`. Forced good/great drops skip the
  rolls.
* Power 2 is required for an ego (and gives the artifact roll).
* Weapons: power >= 1 adds `d5 + m_bonus(5, level)` to hit and to dam; power 2 adds a further
  `m_bonus(10, level)` to each and may supercharge the damage dice (each loop chance
  `1 in 4*dd*ds`, cap dd*ds <= 40). Ammo at power 2 may gain +1 or +2 sides.
* Armour: power >= 1 adds `d5 + m_bonus(5, level)` AC; power 2 adds `m_bonus(10, level)`.
* `m_bonus(max, level)` is a normal roll with mean `max * level / 128` and s.d. `max/4`, clamped
  to 0..max (`src/z-rand.c:m_bonus()`).
* Every wearable has a **1 in 20** chance to get curses (15).
* There are no "negative power" broken items in 4.2: bad items come from curses.

Probability that a non-forced wearable is "great" (ego/artifact eligible):

| Generation level | P(good) | P(great) |
|------------------|---------|----------|
| 1 | 34% | 10.2% |
| 10 | 43% | 12.9% |
| 20 | 53% | 15.9% |
| 30 | 63% | 18.9% |
| 40 | 73% | 21.9% |
| 50 | 83% | 24.9% |
| 67+ | 100% | 30% |

Then an ego is applied only if one fits the kind; also with chance 1 in `great-ego` (20) the ego
level is boosted (`make_ego_item()`).

### 13.3 Good and great drops

* DROP_GOOD monsters: every item is generated "good" (level+10 kind choice, power >= 1).
* DROP_GREAT: forced great, so the item is ego-eligible and gets 2 artifact rolls (4 with the
  unique extra roll).
* Scroll of Acquirement: 1 great item; *Acquirement*: 1+1d2 great items, all with extra artifact
  rolls (`src/effect-handler-general.c:effect_handler_ACQUIRE()`, `src/obj-make.c:acquirement()`).
* Vaults and level feelings are covered in [Chapter 9](09-dungeon.md).

---

## 14. Rune-based identification

### 14.1 The system

FACT (overview comment at the top of `src/obj-knowledge.c`): every object property is a **rune**
(a flag, a modifier such as +STR, a resist, a brand, a slay, a curse, or one of the three combat
runes: to-AC, to-hit, to-dam). The player has a knowledge object (`p->obj_k`); when you learn a
rune it becomes known **on every object**, past and future. An item is fully known when all its
runes are known. An item with runes you have not learned is shown with `{??}`.

Flavours (potions, scrolls, rings, amulets, wands, staves, rods, mushrooms) are a separate system:
you become "aware" of a flavour by using it with a noticeable effect, buying it, or selling it.

### 14.2 How each kind of rune is learned

| Rune type | Learned when | Source |
|-----------|--------------|--------|
| Any modifier (+STR, +speed, +stealth, +blows, +light...) | immediately on wielding the item | `object_learn_on_wield()` |
| "Obvious on wield" flags: telepathy, see invisible, blessed, burns out/takes fuel/no fuel, constant fear (AFRAID), sticky, light and dig powers, throwing | on wield | `lib/gamedata/object_property.txt` (`id:on wield`) |
| Sustains for a stat the item modifies | on wield | `object_learn_on_wield()` |
| Timed flags: slow digestion, regeneration, impaired HP/mana, aggravation, experience drain | within a short time of wearing (checked every 100 game turns) | `equip_learn_after_time()`, `id:timed` |
| On-effect flags: free action, hold life, feather fall, protection from fear/blind/conf/stun, sustains, teleport ban, trap immunity | when the effect would happen (e.g. a paralysis attack) | `equip_learn_flag()`, `id:on effect` |
| Resistances | when you are hit by that element ("Your X glows") | `equip_learn_element()` |
| To-AC | when a monster attacks you | `equip_learn_on_defend()` |
| To-hit, to-dam (non-weapon slots) | when you make a melee or missile attack | `equip_learn_on_melee_attack()`, `..._ranged_attack()` |
| Slays and brands | when they apply against a monster | `src/player-attack.c` |
| Weapon/launcher to-hit, to-dam | when you attack with it | as above |
| Curses | when their effect is noticed, or some on wield (stat/AC/flag curses) | `object_curses_find_*()` |

Once all runes of an ego are known, you recognise that ego on sight
(`player_knows_ego()`). Walking onto an object (or picking it up) tells you whether it is an
artifact (`object_touch()` marks it "assessed").

### 14.3 Shortcuts

* **Scroll of Identify Rune** (cost 200, DL5+): learns one unknown rune on one item
  (`effect_handler_IDENTIFY()` -> `object_learn_unknown_rune()`).
* **Buying from a store** learns all unknown runes on the bought item (`src/store.c:do_cmd_buy()`
  loops `object_learn_unknown_rune()` until fully known). A cheap ego in the Armoury with `{??}`
  teaches you its runes for every future item.
* **Selling (or, with `birth_no_selling`, giving) to a store** also learns every rune on the item
  and makes its flavour known (`src/store.c:do_cmd_sell()`). The store keeps the item, so this is
  "ID by sacrifice"; the official manual mentions it ([user-manual/dungeon.rst](user-manual/dungeon.rst)).
* `birth_know_runes` / `birth_know_flavors` make everything known from the start.
* Wearing an unknown item is mostly safe in 4.2 because curses can be removed by taking the item
  off (except STICKY). The risk is a curse that acts while worn (paralysis, summon, teleport).

ADVICE for a borg: maintain the rune knowledge set and treat `{??}` items as "possibly better than
displayed". Wear-test unknown jewellery and armour in a safe place (town or a cleared level), then
take it off if a bad curse appears. Learn combat runes early by simply fighting.

---

## 15. Curses

### 15.1 What a curse is

FACT (`lib/gamedata/curse.txt`, `src/obj-curse.c`, `src/obj-make.c:apply_curse()`): curses are
runes attached to a wearable with a **power** (1-99 removable, 100 permanent). A cursed item has
1-4 curse attempts (`randint1(4)`), each picking a random curse legal for the item type, with
power `randint1(9) + 10 * m_bonus(9, level)` (so roughly 1-19 at shallow depth, up to 99 deep).
Each successful curse also raises the item's effective generation level, so cursed items tend to
have better other properties. BLESSED items are never cursed. Conflicting curses are rejected.

Curse effects with timers fire when their timeout runs out and then reset
(`src/game-world.c`, `src/obj-curse.c:do_curse_effect()`).

### 15.2 The curse list

| Curse | Item types | Effect |
|-------|-----------|--------|
| vulnerability | body/shield/cloak | to-AC -50, AGGRAVATE |
| teleportation | helm/crown/amulet/ring | random teleport (range 40) every d100 turns |
| dullness | helm/crown | INT -5, WIS -5 |
| sickliness | helm/crown | STR, DEX, CON -5 |
| enveloping | cloak | to-hit -5, to-dam -5, AC +20 |
| irritation | cloak | to-hit -15, to-dam -15, AGGRAVATE |
| weakness | gloves | STR -10 |
| clumsiness | gloves | DEX -10 |
| slowness | boots | speed -5 |
| annoyance | boots | speed -10, stealth -10, AGGRAVATE |
| poison | any wearable | poisons you (10+d10) every 1d500 turns |
| siren | any | wakes monsters every 100+1d50 turns |
| hallucination | any | hallucination 10+d10 every 1d500 turns |
| paralysis | any | paralysis 5+1d5 every 1d1000 turns ("Your equipment grabs you!") |
| dragon summon | dragon armour | summons a dragon every 1d500 turns |
| demon summon | weapons, diggers | summons a demon every 1d500 |
| undead summon | weapons, diggers | summons an undead every 1d500 |
| impair mana recovery | helm/crown | IMPAIR_MANA |
| impair hitpoint recovery | armour slots | IMPAIR_HP |
| cowardice | amulet/ring | permanent fear (AFRAID) |
| stone | amulet/ring | stoneskin 20+1d20 every 1d1000 turns (+40 AC, -5 speed) |
| anti-teleportation | boots/cloak/body/amulet/ring | NO_TELEPORT: you cannot teleport |
| treacherous weapon | weapons | the weapon hits you for its damage every 1d200 turns |
| burning up | armour slots | fire vulnerability, cold resistance |
| chilled to the bone | armour slots | cold vulnerability, fire resistance |
| steelskin | many | AC +20, electricity vulnerability, shard resistance |
| air swing | weapons | to-hit -20 |

The paralysis curse is blocked on items that already give Free Action; curses whose timed
effect is resisted by the item's own flags or resists are rejected (`append_object_curse()`).

### 15.3 Removing curses

FACT (`src/effect-handler-general.c:uncurse_object()`): you choose one curse on one item. If
`strength >= curse power` the curse is removed. If not: the first failure makes the item
**FRAGILE**; a failure on an already fragile item destroys it with chance 1/4 (5d5 damage to you,
artifacts are lost); otherwise "The removal fails". Power 100 curses cannot be removed at all.

| Source | Strength |
|--------|----------|
| Scroll of Remove Curse (Alchemist, always stocked, 100 gold) | 20+d20 (21-40) |
| Staff of Remove Curse (DL40+) | 35+d30 (36-65) |
| Scroll of *Remove Curse* (DL50+, 8000) | 50+d50 (51-100) |
| Activations REMOVE_CURSE / REMOVE_CURSE2 | 20+d20 / 50+d50 |

Enchantment scrolls do not remove curses in this version: the header comment of
`src/effect-handler-general.c:enchant()` still mentions a "parallel attempt to uncurse", but the
function body only raises to-hit/to-dam/to-AC.

ADVICE:

* Most curses are just a tradeoff to evaluate. A body armour with "steelskin" (+20 AC, elec
  vulnerability) is fine if you have rElec. "Enveloping" is +20 AC for -5/-5.
* Remove only curses that matter: paralysis, teleportation (for melee), anti-teleportation
  (always, it disables your escapes), summons, treacherous weapon, siren, vulnerability.
* Do not gamble a valuable item on a weak scroll against a high-power curse: the item turns
  fragile on the first miss, and then each further miss is a 1/4 destruction chance. The curse
  power is shown when the curse is known; compare it to the scroll's maximum.

---

## 16. Ignoring and throwing out junk

FACT (`src/obj-ignore.c`): the `k` (or `^D`) command ignores an item; ignored items are hidden and
skipped by pickup. Ignoring works by:

* **Kind/flavour**: ignore a kind once aware (or a flavour while unaware).
* **Quality levels** per ignore type (swords, blunt, bows, body armour, cloaks, shields, headgear,
  handgear, feet, diggers, rings, amulets, lights, each DSM class, Elven cloaks...): "bad",
  "average", "good", "non-artifact". An item is rated by `ignore_level_of()`:
  * jewellery: "average" if any modifier or combat bonus is positive, "bad" if any is negative;
  * other items when fully known: good/average/bad by comparing to-dam (x4), to-hit (x2), to-AC (x1)
    against the base kind; egos rate as "non-artifact" level; artifacts never qualify;
  * not fully known but assessed and not an artifact: counts as "non-artifact".
* **Ego type**: ignore a specific ego on a specific ignore type (e.g. "of Slay Animal" swords).
* Inscribing `!k` or `!*` protects an item from ignoring; artifacts are never quality-ignored.

The store-side equivalent is the ignore option in the knowledge menu; see
[user-manual/command.rst](user-manual/command.rst) and [user-manual/customize.rst](user-manual/customize.rst).

ADVICE (a sane ignore setup by phase):

| Phase | Set quality ignore to | Also ignore |
|-------|-----------------------|-------------|
| DL1-10 | bad on all types | Potions of Salt Water, Sleep, Blindness, Confusion, Poison, Slowness; Scrolls of Darkness, Aggravate, Curse, Summon Undead, Treasure Detection once aware |
| DL10-30 | average on weapons/armour you won't use; bad on jewellery | Amulets of Adornment, Inertia, Teleportation; Rings of Teleportation (unless testing), Searching? (keep if traps bite) |
| DL30+ | "good" (non-ego) on armour and weapons | low egos you no longer want (Slay Animal, Resist X single resists) |
| DL50+ | "non-artifact" on many types except body armour/DSM/boots | everything but healing/escape consumables |

Never ignore by quality before the relevant runes are known (otherwise "average" items might hide
egos only shown after learning). With `birth_no_selling` (default, see [Chapter 11](11-town-and-shopping.md))
there is no money in junk, so ignore aggressively.

---

## 17. What to wear at each stage

ADVICE (combines source facts above with common play experience; requirements for resists by
depth are discussed in [Chapter 5](05-elements-and-status.md) and readiness in
[Chapter 12](12-game-plan-by-phase.md)). "Must" means "do not go deeper without it".

| Slot | DL1-5 | DL5-15 | DL15-30 | DL30-40 | DL40-60 | DL60-98 | Endgame (DL99-100) |
|------|-------|--------|---------|---------|---------|---------|---------------------|
| Weapon | start weapon or best store weapon giving 2+ blows | any ego/brand; +to-dam; check blows | ego with slays/brand; FA/SI ego a bonus | high-damage ego (Westernesse, Gondolin, Extra Attacks, *Slay*) | best available; slay evil/brands | artifact or great ego; dice x slay | slay evil + high blows; Morgoth is evil (no undead/demon slay) |
| Bow | Sling/Short Bow | Long Bow, good ammo | x3 + egos | Extra Might/Shots | Heavy Crossbow or ego long bow | artifact/ego | as available |
| Rings | anything +AC | Resist Fire and Cold; Protection; Open Wounds as swap | **Free Action** (must by DL20-25 if no other FA); **See Invisible** (if no SI) | rPois ring if no rPois | stat rings, Damage/Slaying | **Speed**, Speed | 2x Speed or Speed + utility |
| Amulet | none | Resist Acid/Lightning, Slow Digestion | Regeneration, Wisdom | rAcid/rElec filler until better | ESP or Resistance | Magi/Weaponmastery/Trickery/ESP | best of these |
| Light | torch x3-5 | Lantern (+ oil) | Lantern of True Sight or Phial | Phial / Star | any no-fuel artifact | Arkenstone (HL, rLight, rDark) ideal | same |
| Body | Soft Leather | Leather Scale / Chain (casters: weight!) | Resistance or Elvenkind ego, or low DSM | **4 base resists** across the kit (must by DL30-35) | DSM / high-resist ego | MHDSM, Power DSM, artifact | max resists |
| Cloak | Cloak | Cloak | of Protection/Stealth | of Aman | of Aman/artifact | artifact | |
| Shield | Wicker / Leather | Leather | of Resist X | Resistance | Elvenkind/Preservation | best | |
| Helm | Hard Leather Cap | Metal Cap / Iron Helm | **Seeing** (pBlind + SI) | Telepathy if possible | Telepathy / Serenity (pConf) | Telepathy or crown with pConf/pBlind | |
| Gloves | Leather | **Free Action** gloves (cheap FA source) | FA / Slaying | Slaying/Power | | | |
| Boots | Sandals/Leather | Iron Shod | Free Action / Stability (rNexus) | Stability / Speed | Speed | Speed/Elvenkind | |

Milestones the table implies (FACT about what each protection blocks, in [Chapter 5](05-elements-and-status.md)):

* **Free Action** before DL20-25: paralysis from carrion crawlers, ghouls and spells is lethal
  without it. Sources in order of availability: Gloves of FA (alloc 100, DL1-60), Boots of FA,
  Ring of FA (DL20+), weapon egos (Defender, Westernesse, Gondolin), artifacts.
* **See Invisible** by DL20-30: ghosts and invisible "G"/"p". Sources: Helm of Seeing, Lantern of
  True Sight, Ring of SI (DL30+), Potion of True Seeing (temporary), weapon egos, ESP.
* **Four base resists** by DL30-40 (dragons and hounds); **rPois** by DL40 (drolems, poison
  hounds, AMHD); **pConf/pBlind**, **rNether/rChaos/rDisen** for DL60+.
* **Speed**: +10 by DL60, +20 for the bottom.

---

## 18. Swap items

FACT: swapping an item costs one turn (wield takes a player turn, `src/cmd-obj.c:do_cmd_wield()`).
Inventory slots are limited to 23 (`pack-size`), so each swap item costs a pack slot.

ADVICE:

| Swap | Use |
|------|-----|
| Ring of Open Wounds | activate for 40 HP + cure blind/conf/cuts every 10+5d8 turns, then swap back |
| A second resist ring/amulet (e.g. rPois ring) | put on before fighting a known poison breather |
| Free Action item | if your FA is on a slot you'd rather use otherwise, swap it in for paralyzers (ghouls, carrion crawlers, basilisks) |
| See Invisible item | swap in on ghost-heavy levels |
| Digger | swap in to tunnel |
| Slay weapon (e.g. *Slay Dragon*, *Slay Undead*) | swap in for one big fight (a dragon unique, a lich) |
| Lantern of Shadows | stealth for sneaking past sleeping things (light -4!) |
| Ring of Escaping | +2 speed while running away or shooting; blocks melee and adds +20% spell fail |
| Ring of Digging | stone to mud activation as an emergency wall breaker |
| Amulet of Regeneration | wear while resting to recover faster |

A borg should value a swap item by the improvement it gives in the specific situation times how
often that situation occurs, minus the pack-slot cost (19.3).

---

## 19. Comparing two items, and a scoring function for a borg

### 19.1 In-game tools

FACT:

* `I` (Inspect) shows the full known description built by `src/obj-info.c`: blows with this
  weapon, average damage per blow and per turn against normal and slain monster types, shots and
  missile damage, light radius and fuel turns, AC, all known runes, and device/activation fail
  rate. It reflects your *current* stats.
* The knowledge menu (`~`) entry **"Display equippable comparison"**
  (`src/ui-knowledge.c:do_cmd_knowledge_equip_cmp()` -> `src/ui-equip-cmp.c:equip_cmp_display()`)
  shows a resistance/ability grid of equippable items from several sources: worn, pack, floor,
  home and store (`c` cycles sources, `v` cycles attribute views, `q`/`!` quick filters, `I`/`x`
  to select one or two items for details, `d` dumps to a file, `?` help).
* The character screen `C` shows the resistance grid of your current kit.

### 19.2 Practical comparison rules (ADVICE)

1. Never trade away a *needed* protection (FA, SI, a base resist you would lose entirely, rPois at
   depth, pConf/pBlind deep) for raw damage or AC unless another slot covers it.
2. Speed beats almost everything: +1 speed is about +10% actions.
3. For weapons compare damage per turn on the Inspect screen, then slays/brands vs what you fight.
4. For casters, compare max SP: an item that pushes you over the armour weight limit costs 1 SP per
   pound.
5. Unknown runes (`{??}`) can only add value; an `{??}` ego is usually worth trying on.

### 19.3 A numeric scoring approach (ADVICE for a borg)

The existing borg scores a whole kit, not an item (`borg_power()` in `src/borg/borg-power.c`,
summarised in [../c-borg/BORG_SPEC.md](../c-borg/BORG_SPEC.md) section 5): it simulates wearing a
candidate, recomputes derived traits, and adds a large bonus for every depth it becomes "prepared"
for. That idea is sound; here is a compact, explicit version to implement or tune.

Score a *whole equipment set* S (with the candidate swapped in), then compare:

```
score(S) = W_speed * speed_bonus
         + W_dmg   * melee_dpt(S)          # avg damage per player turn vs a normal monster (obj-info calc)
         + W_shoot * missile_dpt(S)
         + W_ac    * ac_value(total_ac)    # diminishing: e.g. 3*min(AC,100) + 1*max(AC-100,0)
         + W_hp    * max_hp(S)
         + W_sp    * max_sp(S)             # casters only; includes weight-limit loss
         + sum over needed protections p of  NEED(p, depth) * has(p, S)
         + W_stat  * sum of stat gains weighted by class (spell stat x2, STR/DEX for melee)
         + W_stealth * stealth + W_light * min(light_radius, 4)
         - penalties (AGGRAVATE, DRAIN_EXP, IMPAIR_*, known curses, NO_TELEPORT huge)
```

`NEED(p, depth)` is the key: a step function that makes a protection worth very little before the
depth where it starts to matter and very much after. Example values (score units comparable to
1 point of damage per turn = 10):

| Protection | DL<15 | DL15-30 | DL30-40 | DL40-60 | DL60+ |
|------------|-------|---------|---------|---------|-------|
| Free Action | 50 | 2000 | 5000 | 5000 | 5000 |
| See Invisible | 20 | 800 | 1500 | 1500 | 2000 (unless ESP) |
| each base resist | 30 | 300 | 1500 | 2500 | 3000 |
| rPois | 0 | 100 | 1000 | 4000 | 5000 |
| pConf, pBlind | 10 | 100 | 500 | 1500 | 3000 |
| rNether, rChaos, rDisen, rNexus, rShards, rSound, rLight, rDark | 0 | 50 | 200 | 800 | 2000 |
| Hold Life | 0 | 100 | 500 | 1500 | 2000 |
| ESP | 50 | 500 | 1500 | 2500 | 3000 |
| Speed (per point) | 300 | 500 | 800 | 1200 | 1500 |

Implementation notes:

* Compute derived values with the game's own formulas (blows from `calc_blows()`, light from
  `calc_light()`, mana from `calc_mana()`), not by summing item fields; many properties are
  non-additive (resists do not stack, light does, speed does, blows are capped by class).
* Count each protection once across the set: the value is in `has(p, S)`, not per item.
* For unknown runes, add an expected value bonus (e.g. 20% of the ego's typical power).
* For swap items, score `max(score(S), score(S with swap))` per situation, weighted by frequency,
  minus a pack-slot cost (e.g. the score of the consumable you'd have to leave behind).
* Re-score after every rune learned; an item's value can jump when a rune becomes known.

An alternative single-item number already in the game is `object_power()` in `src/obj-power.c`,
used for pricing (value = power*(power+5) for wearables, `object_value_real()`). It is a decent
"how strong is this item in general" proxy but ignores *your* needs, so use it only as a
tie-breaker or for ignoring decisions.

---

## Things to try (untested)

1. **Ring of Teleportation dive**: wear it while diving to keep moving through levels with +2
   speed and free random teleports. Why: the curse is permanent but the ring can be removed. Test:
   compare 20 dives DL10-25 with and without it, counting deaths, turns per level and stairs found.
2. **ID-by-buying**: buy the cheapest `{??}` ego in the Armoury/Weaponsmith each town visit purely
   to learn its runes. Why: buying learns all runes (`do_cmd_buy()`). Test: count runes known at
   CL20 with and without this habit.
3. **Two elemental rings for double resists**: Flames + Ice give permanent rFire/rCold and
   activate for temp resist, so you reach "double resist" without potions. Test: measure damage
   taken from fire/cold breathers in DL25-40 with this vs Resist Fire and Cold.
4. **Lantern of Shadows sneaking**: swap to a Shadows lantern (light -4) to reduce your light radius
   while passing sleeping monsters. Unknown whether negative light radius changes monster
   detection; test with a stealth-heavy rogue on DL20-30 counting wake-ups.
5. **Ring of Escaping for archers**: wear +2 speed with constant fear on a Ranger who never melees
   (accepting +20% spell fail). Test: kill rate and survival vs a Ring of Damage over DL20-40.
6. **Ring of Open Wounds as a permanent healer**: for casters who rest anyway, check whether the
   IMPAIR_HP regen loss is outweighed by a free 40 HP heal every ~30 turns.
7. **Curse-for-power hunting**: cursed items get raised generation level (`apply_curse()`); try
   keeping cursed armour with harmless curses (steelskin, enveloping, burning up when you have
   rFire) and compare AC/resists to uncursed finds at the same depth.
8. **Equip-cmp screen as borg oracle**: dump the equippable comparison (`d`) and parse it instead of
   inspecting items one by one. Test feasibility against the borg's own item model.
9. **Glove-free myth**: verify in play that a Mage's SP is unaffected by plain gauntlets (it should
   be, per `calc_mana()`), and correct any borg rule that avoids gloves for casters.
10. **Score-function tuning**: run the 19.3 scoring with different `NEED` tables in an automated
    player and measure average max depth reached; tune the step points to the depth where deaths
    from each missing protection actually start.

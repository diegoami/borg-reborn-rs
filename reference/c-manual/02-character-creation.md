# 2. Character Creation

What you choose at birth, what every number on the birth screen means in play, and how to pick
a race, a class, a stat allocation and a set of birth options for a given goal (learning,
winning, or running an automated player). All figures come from `lib/gamedata/p_race.txt`,
`lib/gamedata/class.txt`, `src/player-birth.c` and `src/player-calcs.c` at the reference commit.

**Contents**

1. [What birth fixes forever](#1-what-birth-fixes-forever)
2. [How race and class combine](#2-how-race-and-class-combine)
3. [Races](#3-races)
4. [Classes](#4-classes)
5. [Hit points and the hit-die matrix](#5-hit-points-and-the-hit-die-matrix)
6. [Stats and their breakpoints](#6-stats-and-their-breakpoints)
7. [Point-based stats](#7-point-based-stats)
8. [The standard roller](#8-the-standard-roller)
9. [Birth options](#9-birth-options)
10. [Choosing a combination](#10-choosing-a-combination)
11. [Things to try (untested)](#things-to-try-untested)

Related: per-class strategy is in [Chapter 14](14-class-playbooks.md), every spell in
[Chapter 3](03-magic-and-spells.md) and [A7](appendices/A7-class-spells.md), the full race/class
tables in [A8](appendices/A8-races-and-classes.md), and the underlying formulas (to-hit, blows,
skills, regeneration) in [Chapter 1](01-core-mechanics.md). The official page is
[Creating a Character](https://angband.readthedocs.io/en/latest/birth.html)
(vendored: [user-manual/birth.rst](user-manual/birth.rst)); the birth options are documented in
[user-manual/option.rst](user-manual/option.rst).

---

## 1. What birth fixes forever

The official page says you pick a race and class, may change the birth options, allocate stats,
and name the character; race, class and birth options cannot be changed afterwards. That is
correct. In more detail (FACT, `src/player-birth.c`):

| Choice | Permanent? | Matters in play? |
|---|---|---|
| Race | yes | yes: stats, skills, hit die, exp factor, infravision, intrinsic flags |
| Class | yes | yes: stats, skills and their growth, hit die, blows, magic, flags, starting kit |
| Base stats (point-buy or roll) | the *base* is permanent, but stats can be raised later by potions and drained/restored | yes |
| Birth options | yes | yes (see [section 9](#9-birth-options)) |
| Hit-point rolls | rolled once in `do_cmd_accept_character()` by `roll_hp()`, after you accept; you cannot reroll them on the birth screen | yes |
| Name | yes | only for the savefile name and score list. A trailing Roman numeral ("Fred I") is incremented automatically for the next character |
| History text | editable at birth | no effect at all |
| Age, height | random by race | no effect at all |
| Weight | random by race (`get_ahw()`) | **one effect**: shield-bash quality adds `weight/8` (`src/player-attack.c:attempt_shield_bash()`). Nothing else reads it |
| Sex | does not exist in this version | n/a |

So: the history, age and height are pure flavour; weight matters only marginally for the three
shield-bashing classes; there is no sex choice to agonise over.

Useful birth-screen keys (official manual and option page): in the birth option menu, `s` saves
the current selections as the default for future characters, `r` resets to defaults, `x` resets
to the maintainer's defaults. On the stat screen, the point-buy allocation is pre-filled by an
algorithm ([section 7.3](#73-the-default-allocation-algorithm)); the roller uses `r` to reroll
and can step back to the previous roll.

---

## 2. How race and class combine

FACT (`src/player-birth.c:player_generate()`, `src/player-calcs.c:calc_bonuses()`):

| Quantity | Formula |
|---|---|
| Stats | base stat, then race modifier + class modifier applied point by point by `modify_stat_value()`: below 18 each +1 adds 1; at 18 and above each +1 adds 10 percentile points (17 +3 = 18/20). Negative points remove 10 percentile above 18/10, else 1 |
| Skills | race base + class base + (class increment × level / 10) + stat adjustment + equipment. Increments are applied continuously: a skill with "+45 per 10 levels" gains 4.5 per level |
| Hit die | race hit die + class hit die |
| Experience factor | race `exp` + class `exp`. No class sets `exp`, so the factor is the race's alone: Human 100%, High-Elf 145%, every other race 120% |
| Infravision | race only (×10 feet), plus equipment and timed infravision |
| Intrinsic flags | union of race flags and class flags |
| Stealth | clamped to 0..30 after everything is added |

The class increment is what makes classes diverge with level: a Warrior's melee skill goes from
70 to 295 by CL50 (+45 per 10 levels), a Mage's from 35 to 110. A race bonus of +15 melee
(Dwarf, Dunadan) is a flat head start that never grows.

---

## 3. Races

### 3.1 The numbers

Stats and core values (FACT, `lib/gamedata/p_race.txt`):

| Race | STR | INT | WIS | DEX | CON | Sum | Hit die | Exp | Infra |
|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 10 | 100% | 0 |
| Half-Elf | 0 | +1 | -1 | +1 | -1 | 0 | 10 | 120% | 20 ft |
| Elf | -1 | +2 | -1 | +1 | -1 | 0 | 9 | 120% | 30 ft |
| Hobbit | -2 | +2 | +1 | +3 | +2 | +6 | 7 | 120% | 40 ft |
| Gnome | -1 | +2 | 0 | +2 | +1 | +4 | 8 | 120% | 40 ft |
| Dwarf | +2 | -3 | +2 | -2 | +2 | +1 | 11 | 120% | 50 ft |
| Half-Orc | +2 | -1 | 0 | 0 | +1 | +2 | 10 | 120% | 30 ft |
| Half-Troll | +4 | -4 | -2 | -4 | +3 | -3 | 12 | 120% | 30 ft |
| Dunadan | +1 | +2 | +2 | +2 | +3 | +10 | 10 | 120% | 0 |
| High-Elf | +1 | +3 | -1 | +3 | +1 | +7 | 10 | 145% | 40 ft |
| Kobold | -1 | -1 | 0 | +2 | +2 | +2 | 8 | 120% | 50 ft |

Skill bonuses (FACT; "Disarm" is physical/magical, which are equal for every race):

| Race | Disarm | Device | Save | Stealth | Search | Melee | Shoot | Throw | Dig |
|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Half-Elf | 2 | 3 | 3 | 1 | 3 | -1 | 5 | 5 | 0 |
| Elf | 5 | 6 | 6 | 2 | 6 | -5 | 15 | 15 | 0 |
| Hobbit | 15 | 18 | 18 | 4 | 6 | -10 | 20 | 20 | 0 |
| Gnome | 10 | 22 | 12 | 3 | 4 | -8 | 12 | 12 | 0 |
| Dwarf | 2 | 9 | 9 | -1 | 2 | 15 | 0 | 0 | 40 |
| Half-Orc | -3 | -3 | -3 | -1 | -3 | 12 | -5 | -5 | 0 |
| Half-Troll | -5 | -8 | -8 | -2 | -9 | 20 | -10 | -10 | 0 |
| Dunadan | 4 | 5 | 5 | 1 | 3 | 15 | 10 | 10 | 0 |
| High-Elf | 4 | 20 | 20 | 2 | 10 | 10 | 25 | 25 | 0 |
| Kobold | 10 | 5 | 0 | 3 | 10 | -5 | 10 | 10 | 0 |

Intrinsic abilities (FACT, `obj-flags`, `player-flags`, `values` in `p_race.txt`):

| Race | Intrinsics |
|---|---|
| Human | none |
| Half-Elf | Sustain DEX |
| Elf | Sustain DEX, resist light |
| Hobbit | Hold Life, Mushroom Lore (`KNOW_MUSHROOM`) |
| Gnome | Free Action, Device Lore (`KNOW_ZAPPER`) |
| Dwarf | Blindness immunity (`PROT_BLIND`), Miner (`SEE_ORE`), +40 digging |
| Half-Orc | resist darkness |
| Half-Troll | Sustain STR, Regeneration |
| Dunadan | Sustain CON |
| High-Elf | See Invisible, resist light |
| Kobold | resist poison |

All races use the same humanoid body (same equipment slots).

### 3.2 What the intrinsics mean in play

FACT, with the code that implements each:

- **Hold Life (Hobbit).** Object flag `HOLD_LIFE`: protects against experience drain (details
  in [Chapter 5](05-elements-and-status.md)). Experience drain is common from undead touches
  and nether, so this saves many Restore Life Levels potions in the midgame.
- **Mushroom Lore (Hobbit).** Mushrooms become flavour-aware when picked up
  (`src/obj-gear.c:inven_carry()`, message "Mushrooms for breakfast!"). Saves eat-testing
  dangerous mushrooms.
- **Free Action (Gnome).** `FREE_ACT` from birth. Paralysis is the classic early-to-midgame
  killer (paralysing touches, gazes and hold spells): a Gnome
  never needs to hunt for Free Action gear. ADVICE: the single most valuable early intrinsic
  (see [Chapter 5](05-elements-and-status.md) and [Chapter 8](08-monsters.md) for which monsters
  paralyse).
- **Device Lore (Gnome).** Wands and staffs (not rods: `tval_is_zapper()` is wand or staff only)
  become flavour-aware on pickup. You know what a wand is before you aim it.
- **Blindness immunity (Dwarf).** `PROT_BLIND`. Blindness stops reading scrolls and casting
  spells, so this is especially good for a Dwarf Priest; it also removes one way dark/light
  breath and certain traps can cripple you.
- **Miner (Dwarf).** Each player turn, while not hallucinating, confused, amnesiac, stunned,
  paralysed or afraid, the game runs a free ore/treasure detection in a 3-grid range
  (`src/game-world.c:process_player()` calling `EF_DETECT_ORE`). Cosmetic for most play; useful
  for gold early when `birth_no_selling` makes gold the only income.
- **Resist light (Elf, High-Elf), resist darkness (Half-Orc), resist poison (Kobold).** A
  permanent first-level resistance (damage divided, side effects such as blindness from
  light/dark prevented; see [Chapter 5](05-elements-and-status.md)). Kobold poison resistance is
  the most practically valuable of the three: poison breath (drakes, hounds, Basilisks) and
  poison attacks are everywhere from DL10 to DL40, and rPois is otherwise a midgame item hunt.
- **Sustains (Half-Elf/Elf DEX, Half-Troll STR, Dunadan CON).** Stat drain cannot lower the
  sustained stat. CON is the stat you least want drained late (it is your HP), so the Dunadan
  sustain is the best of these.
- **Regeneration (Half-Troll).** HP regeneration doubled (`src/player-util.c:player_regen_hp()`)
  and also mana regeneration (`player_regen_mana()`), but digestion is doubled too
  (`src/game-world.c`, food consumption `i *= 2` with `OF_REGEN`). Carry more food.
- **See Invisible (High-Elf).** Ghosts, invisible demons and "clear" monsters are visible from
  birth. Invisible monsters are a constant source of unexplained deaths for characters without
  See Invisible (ghosts in particular, which also pass through walls). The High-Elf trades 25%
  more experience for this. Which monsters are invisible: [Chapter 8](08-monsters.md).
- **Infravision.** Shows warm-blooded monsters within the radius even without light. Useful for
  spotting sleeping monsters in dark rooms before you walk into them; not a substitute for See
  Invisible (invisible monsters are not shown).

### 3.3 Race profiles

**Human.** Exp 100%: levels fastest, everything else average, no intrinsics, no infravision.
Faster levelling matters most in the first few hours; by CL30 the 20% difference is a few
dungeon levels' worth of experience. ADVICE: a fine choice for any class if you want the
fastest early spell/blow progression; objectively weaker than Dunadan for a winner.

**Half-Elf.** A slightly elfier Human with Sustain DEX and 20 ft infravision for 120% exp. ADVICE:
rarely the best choice for anything; harmless.

**Elf.** INT +2, good shooting (+15), sustain DEX, resist light, hit die 9, weak melee (-5).
ADVICE: decent Mage, Rogue or Ranger; strictly worse than High-Elf except for the exp factor.

**Hobbit.** DEX +3, CON +2, superb device/save (+18/+18) and stealth (+4), Hold Life, but STR -2
and hit die 7 (lowest). Hobbit is the best Rogue race (stealth and throwing/shooting) and a good
Mage/Necromancer if you accept low HP. ADVICE: the low hit die is very noticeable on a Mage
(1d7 per level); on a Rogue (1d11) it is fine.

**Gnome.** Free Action from birth, the best device skill (+22), INT +2, DEX +2, CON +1, hit die 8.
ADVICE: the standard Mage race. Free Action plus device skill makes a Gnome Mage's early game
far safer than any other race's.

**Dwarf.** STR/WIS/CON +2, INT -3, DEX -2, blindness immunity, +15 melee, +40 digging, hit die 11,
terrible stealth (-1). ADVICE: best Priest race (WIS +2, can always read and pray); good Paladin
and Warrior; very good Druid (WIS). Unusable for INT casters.

**Half-Orc.** STR +2, CON +1, +12 melee, resist dark, poor skills. ADVICE: a cheap Warrior or
Blackguard race; outclassed by Dunadan and Half-Troll.

**Half-Troll.** STR +4, CON +3, INT -4, WIS -2, DEX -4; +20 melee; hit die 12 (highest);
regeneration and sustain STR; worst device skill and saving throw (-8/-8). ADVICE: the
strongest early melee race; a Half-Troll Warrior is the classic "easy" character. The bad device
skill makes staffs of Teleportation and wands fail more often, and the bad save makes
paralysis/confusion/blindness from monster spells more frequent: buy Free Action early.

**Dunadan.** +1/+2/+2/+2/+3 (the best total, +10), +15 melee, +10 shooting, sustain CON, hit die
10, no infravision, 120% exp (same as most races). ADVICE: the best all-round race; good at
every class. It is the default recommendation for a first winner.

**High-Elf.** INT +3, DEX +3, STR/CON +1, excellent device/save (+20/+20) and shooting (+25),
See Invisible, resist light, hit die 10, but 145% exp. ADVICE: strongest race for Mage, Ranger
and Rogue once levels are not the bottleneck; See Invisible is worth a lot to every class.
The extra exp mostly costs time early (spell unlocks arrive later).

**Kobold.** DEX/CON +2, resist poison, stealth +3, hit die 8. ADVICE: an underrated Rogue or
Ranger race; rPois covers a big midgame hole. Weak for INT/WIS casters (no bonus to either).

---

## 4. Classes

### 4.1 The numbers

Stats, hit die and blows parameters (FACT, `lib/gamedata/class.txt`):

| Class | STR | INT | WIS | DEX | CON | HD | Max blows | Min weapon wt | STR mult |
|---|---|---|---|---|---|---|---|---|---|
| Warrior | +3 | -2 | -2 | +2 | +2 | 9 | 6 | 3.0 lb | 5 |
| Mage | -3 | +3 | 0 | 0 | -2 | 0 | 4 | 4.0 lb | 2 |
| Druid | -2 | 0 | +3 | -2 | 0 | 2 | 4 | 3.5 lb | 3 |
| Priest | -1 | -3 | +3 | -1 | +1 | 2 | 4 | 3.5 lb | 3 |
| Necromancer | -3 | +3 | 0 | 0 | -2 | 2 | 4 | 3.5 lb | 3 |
| Paladin | +1 | -3 | +1 | -1 | +2 | 6 | 5 | 3.0 lb | 5 |
| Rogue | 0 | +1 | -3 | +3 | -1 | 4 | 5 | 2.0 lb | 4 |
| Ranger | 0 | 0 | +2 | +1 | -1 | 5 | 5 | 3.5 lb | 4 |
| Blackguard | +2 | 0 | -3 | 0 | +2 | 8 | 5 | 10.0 lb | 5 |

"Min weapon wt" is the weight floor used in the blows formula: weapons lighter than this count as
this heavy. "STR mult" multiplies the STR term (higher is more blows). Blackguards' 10 lb floor
means a dagger gives them no more blows than a 10 lb weapon, so they should use heavy weapons.
See [Chapter 4](04-combat.md) for the full blows formula.

Skills: base at CL1 and increment per 10 levels (FACT):

| Class | Disarm phys | Disarm mag | Device | Save | Stealth | Search | Melee | Shoot | Throw |
|---|---|---|---|---|---|---|---|---|---|
| Warrior | 25 (+15) | 20 (+10) | 18 (+7) | 18 (+10) | 0 (+0) | 10 (+12) | 70 (+45) | 55 (+45) | 55 (+45) |
| Mage | 30 (+10) | 35 (+12) | 36 (+13) | 30 (+9) | 2 (+0) | 10 (+12) | 35 (+15) | 20 (+15) | 20 (+15) |
| Druid | 30 (+10) | 30 (+10) | 24 (+10) | 30 (+10) | 3 (+0) | 12 (+12) | 45 (+20) | 40 (+30) | 40 (+30) |
| Priest | 25 (+12) | 25 (+12) | 30 (+10) | 32 (+12) | 2 (+0) | 10 (+14) | 45 (+20) | 35 (+20) | 35 (+20) |
| Necromancer | 30 (+10) | 35 (+12) | 36 (+13) | 30 (+9) | 2 (+0) | 10 (+12) | 35 (+25) | 20 (+15) | 20 (+15) |
| Paladin | 20 (+15) | 20 (+10) | 24 (+10) | 25 (+11) | 0 (+0) | 10 (+12) | 65 (+40) | 50 (+30) | 50 (+30) |
| Rogue | 45 (+20) | 45 (+20) | 32 (+10) | 28 (+10) | 3 (+1) | 20 (+16) | 35 (+45) | 66 (+30) | 72 (+45) |
| Ranger | 40 (+15) | 30 (+10) | 28 (+10) | 32 (+10) | 3 (+0) | 15 (+15) | 60 (+40) | 72 (+45) | 66 (+30) |
| Blackguard | 20 (+15) | 20 (+10) | 24 (+10) | 18 (+10) | -1 (-1) | 8 (+10) | 65 (+40) | 35 (+15) | 40 (+30) |

Note the Rogue's melee *increment* (+45) equals the Warrior's: Rogues start poor at melee but
end at 260 (base 35 + 225), close to a Paladin (265).

Magic (FACT, `magic:` lines; realm stat from `lib/gamedata/realm.txt`):

| Class | Realm (stat) | First spell at CL | Armour weight before mana penalty | Books used | Spells | Chooses spells? | Zero-fail possible? |
|---|---|---|---|---|---|---|---|
| Warrior | none | - | - | - | 0 | - | - |
| Mage | arcane (INT) | 1 | 30 lb | 5 | 30 | yes | yes |
| Druid | nature (WIS) | 1 | 35 lb | 5 | 27 | yes | yes |
| Priest | divine (WIS) | 1 | 35 lb | 5 | 28 | no (random from chosen book) | yes |
| Necromancer | shadow (INT) | 1 | 30 lb | 5 | 26 | yes | yes |
| Paladin | divine (WIS) | 1 | 40 lb | 3 | 16 | no | no (5% floor) |
| Rogue | arcane (INT) | 5 | 35 lb | 2 | 10 | yes | no |
| Ranger | nature (WIS) | 3 | 40 lb | 2 | 11 | yes | no |
| Blackguard | shadow (INT) | 1 | 60 lb | 3 | 15 | yes | no |

The armour weight counts all worn armour (body, cloak, shield, helm, gloves, boots); each full
pound over the allowance removes 1 point of maximum mana (`src/player-calcs.c:calc_mana()`).
**There is no glove penalty and no "encumbering gloves" rule in this version** (no such code
exists in `calc_mana()`): casters can wear any gloves, including Gauntlets of Power. Older
guides that tell Mages to avoid gloves are wrong for 4.2.

Class flags (FACT; full mechanics in [Chapter 14, section 0](14-class-playbooks.md#0-class-mechanics-reference)):

| Class | Flags |
|---|---|
| Warrior | `BRAVERY_30` (fear immunity at CL30), `NO_MANA`, `SHIELD_BASH` |
| Mage | `ZERO_FAIL`, `BEAM` (bolt-or-beam chance = CL% instead of CL/2 %), `CHOOSE_SPELLS` |
| Druid | `ZERO_FAIL`, `CHOOSE_SPELLS`, `CHARM` (+50% power of sleep/slow/confuse/hold/stun/polymorph on animals) |
| Priest | `BLESS_WEAPON` (+2 to-hit and +2 to-dam with hafted or blessed weapons), `ZERO_FAIL` |
| Necromancer | `ZERO_FAIL`, `CHOOSE_SPELLS`, `UNLIGHT`, `EVIL` |
| Paladin | `BLESS_WEAPON`, `SHIELD_BASH` |
| Rogue | `CHOOSE_SPELLS`, `STEAL` |
| Ranger | `FAST_SHOT` (+0.1 shots per 3 levels), `CHOOSE_SPELLS` |
| Blackguard | `CHOOSE_SPELLS`, `SHIELD_BASH`, `COMBAT_REGEN`; object flag `IMPAIR_HP` (halved HP regeneration) |

### 4.2 Starting kit and starting gold

FACT: every character starts with `player:start-gold` = 600 gold
(`lib/gamedata/constants.txt`, `src/player-birth.c:get_money()`), and with `birth_start_kit` on
(default) the value of each kit item is deducted from that 600 (`player_outfit()`). With the
option off you get only one ration and one light source (if your class has one) and keep the
rest of the gold. The Word of Recall scroll is dropped when `birth_no_recall` is set.

| Class | Kit (quantities from `equip:` lines) | Kit base cost* | Gold left* |
|---|---|---|---|
| Warrior | 1-3 Rations, 1-3 Wooden Torches, Potion of Berserk Strength, Dagger, Soft Leather Armour, Word of Recall | ~265 | ~335 |
| Mage | Rations, Torches, Rapier, Word of Recall, [First Spells] | ~230 | ~370 |
| Druid | Rations, Whip, Torches, Word of Recall, [Lesser Charms] | ~190 | ~410 |
| Priest | Rations, Torches, Mace, 2 Potions of Cure Serious Wounds, Word of Recall, [Novice's Handbook] | ~430 | ~170 |
| Necromancer | Rations, Main Gauche, Word of Recall, [Into the Shadows] (**no light source**) | ~205 | ~395 |
| Paladin | Rations, Torches, Main Gauche, Scroll of Protection from Evil, Word of Recall, [Novice's Handbook] | ~255 | ~345 |
| Rogue | Rations, Torches, Dagger, Soft Leather Armour, Word of Recall, [First Spells] | ~210 | ~390 |
| Ranger | Rations, Torches, Main Gauche, Short Bow, 15-20 Arrows, Word of Recall, [Lesser Charms] | ~270 | ~330 |
| Blackguard | [Into the Shadows], Rations, Torches, Tulwar, Leather Shield, Word of Recall, Potion of Cure Light Wounds | ~550 | ~50 |

\*Approximate, from base `cost:` values in `lib/gamedata/object.txt` (Word of Recall 125, Berserk
80, CSW 70, CLW 20, Rapier 70, Mace 130, Tulwar 170, Leather Shield 200, Short Bow 50, Main Gauche
45, Dagger 30, Whip 30, Soft Leather 20, town books 25). The game deducts `object_value_real()`,
which can differ slightly; the birth screen shows the true figure. ADVICE: a Blackguard starts
nearly broke; a Priest has little spare. Plan the first shopping trip accordingly
([Chapter 11](11-town-and-shopping.md)).

Every class except the Warrior starts with its first book. Paladins start with [Novice's
Handbook]; Rogues with [First Spells] even though they cannot cast until CL5; Rangers with
[Lesser Charms] even though they cannot cast until CL3.

---

## 5. Hit points and the hit-die matrix

FACT (`src/player-birth.c:player_generate()`, `roll_hp()`; `src/player-calcs.c:calc_hitpoints()`):

- Level 1 HP = the full hit die. Each later level adds `1d(hit die)`.
- All 50 rolls are made when you accept the character, and rerolled until the CL50 total lies
  between `50 + 50×(HD-1)×3/8` and `50 + 50×(HD-1)×5/8`. Variance is therefore bounded; the
  average per level is about `(HD+1)/2`.
- CON adds `adj_con_mhp/100` HP per level, recomputed from CL1 (so raising CON later raises
  past levels too). Table excerpt (CON after modifiers → HP per level): 3: -2.5, 7: -0.5, 10:
  -0.05, 11: 0, 14: +0.25, 15: +0.5, 16: +0.75, 17: +1, 18/00: +1.5, 18/50: +2.75, 18/70: +3.5,
  18/100: +5, 18/150: +7.5, 18/200+: +12.5.
- Minimum maximum HP is `level + 1`.

Hit die by race (rows) and class (columns):

| Race \ Class | War | Mage | Druid | Priest | Necro | Pal | Rogue | Ranger | BG |
|---|---|---|---|---|---|---|---|---|---|
| Human (10) | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Half-Elf (10) | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Elf (9) | 18 | 9 | 11 | 11 | 11 | 15 | 13 | 14 | 17 |
| Hobbit (7) | 16 | 7 | 9 | 9 | 9 | 13 | 11 | 12 | 15 |
| Gnome (8) | 17 | 8 | 10 | 10 | 10 | 14 | 12 | 13 | 16 |
| Dwarf (11) | 20 | 11 | 13 | 13 | 13 | 17 | 15 | 16 | 19 |
| Half-Orc (10) | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Half-Troll (12) | 21 | 12 | 14 | 14 | 14 | 18 | 16 | 17 | 20 |
| Dunadan (10) | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| High-Elf (10) | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Kobold (8) | 17 | 8 | 10 | 10 | 10 | 14 | 12 | 13 | 16 |

Approximate CL50 HP before CON (derived from the formula, average roll):

| Hit die | 7 | 9 | 10 | 12 | 14 | 16 | 18 | 19 | 21 |
|---|---|---|---|---|---|---|---|---|---|
| Avg CL50 HP (no CON) | ~203 | ~254 | ~280 | ~330 | ~382 | ~433 | ~484 | ~509 | ~560 |
| Allowed range | 162-237 | 200-300 | 218-331 | 256-393 | 293-456 | 331-518 | 368-581 | 387-612 | 425-675 |

Add `CON bonus × 50`: 18/100 CON adds +250, 18/150 adds +375, 18/200 adds +625. ADVICE: CON is
the most important late-game stat for every class: a Mage with 18/150 CON roughly doubles a Mage
with 16 CON. See [Chapter 1](01-core-mechanics.md) for regeneration.

---

## 6. Stats and their breakpoints

The official page describes each stat qualitatively. The table values below (FACT,
`src/player-calcs.c` and `src/player-spell.c`) are indexed by the *displayed* stat after
race/class/equipment modifiers. Column "18/x" means 18/x0-18/x9.

### 6.1 Strength

- **Blows** (index into `adj_str_blow`: 3-17 give 3..17; 18/00: 20; then +10 per 10 percentile to
  240 at 18/220). The STR term is `adj_str_blow × class STR mult / max(weapon weight, class min
  weight)`, capped at 11.
- **Damage**: `adj_str_td` to-dam: 16: +1, 17: +2, 18/20: +3, 18/70: +4, 18/80: +5, 18/100: +6,
  then +1 per 10 percentile up to +20.
- **To-hit**: `adj_str_th` (small).
- **Weapon/bow handling**: `adj_str_hold` (10: 14 lb, 16: 26 lb, 18/00: 30 lb, 18/50: 50 lb,
  18/100: 80 lb). A weapon or launcher heavier than this gives -2 to-hit per pound over and
  **no extra blows / no extra shots** ("heavy wield").
- **Carrying**: weight limit `adj_str_wgt × 10 lb` (STR 7: 90 lb; 10: 120; 16: 180; 18/00: 200;
  18/50: 300). You are slowed by -1 speed for each 10% of the limit carried above half the limit
  (`calc_bonuses()`). A STR-7 Mage starts losing speed above 45 lb.
- **Digging**: `adj_str_dig`.

### 6.2 Intelligence and Wisdom (spell stats)

INT is the spell stat for arcane (Mage, Rogue) and shadow (Necromancer, Blackguard) casters; WIS
for divine (Priest, Paladin) and nature (Druid, Ranger). FACT:

| Spell stat | 8 | 10 | 13 | 15 | 16 | 17 | 18/00 | 18/20 | 18/50 | 18/70 | 18/90 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mana per level ×100 (`adj_mag_mana`) | 50 | 70 | 100 | 120 | 130 | 140 | 150 | 170 | 200 | 250 | 350 | 400 | 650 | 800 |
| Spells per level ×100 (`adj_mag_study`) | 40 | 60 | 85 | 95 | 100 | 105 | 110 | 120 | 150 | 170 | 190 | 200 | 250 | 250 |
| Fail reduction (`adj_mag_stat`) | -1 | 0 | 0 | 2 | 3 | 4 | 5 | 7 | 10 | 12 | 18 | 21 | 36 | 51 |
| Minimum fail % (`adj_mag_fail`) | 50 | 20 | 11 | 9 | 8 | 7 | 6 | 5 | 4 | 4 | 3 | 3 | 1 | 0 |

- Max mana = `1 + adj_mag_mana × (CL - first spell level + 1) / 100`, minus armour-weight penalty.
- Spells you may know = `(adj_mag_study × levels + 50) / 100`.
- Spell fail % = `base fail - 3×(CL - spell level) - adj_mag_stat`, plus 5% per missing mana
  point when you lack the mana, plus 25% for an `UNLIGHT` caster (Necromancer) standing on a lit
  grid, plus 20% if afraid; the result is then clamped between the minimum-fail value (never below
  5% for classes without `ZERO_FAIL`) and 50%. After that clamp, stun adds 15% (25% when heavy)
  and amnesia makes it `50 + fail/2`; the final value is capped at 95%
  (`src/player-spell.c:spell_chance()`). See [Chapter 3](03-magic-and-spells.md).
- INT also adds to device skill (`adj_int_dev`: 15: +2, 18/00: +3, 18/50: +4, 18/70: +5, 18/90:
  +6, ...) and magical disarming. WIS adds to saving throw (`adj_wis_sav`, same shape).

ADVICE: the mana curve accelerates sharply above 18/50 (200 → 400 at 18/100 → 800 at 18/180).
For a pure caster, every potion of the spell stat in the midgame is worth more than nearly any
other stat gain.

### 6.3 Dexterity

- **Blows** (`adj_dex_blow`, the column index): 3-9: 0, 10-16: 1, 17-18/19: 2, 18/20-39: 3,
  18/40-59: 4, 18/60-79: 5, 18/80-99: 6, 18/100-119: 7, 18/120-149: 8, 18/150-179: 9,
  18/180-199: 10, 18/200+: 11. The jumps at 17, 18/20, 18/40, ... matter much more than the
  values between them.
- To-hit (`adj_dex_th`), AC (`adj_dex_ta`), physical disarming, and resistance to monster
  theft (see [Chapter 8](08-monsters.md)); a Rogue's steal skill adds `adj_dex_th`.

### 6.4 Constitution

HP per level ([section 5](#5-hit-points-and-the-hit-die-matrix)) and faster resting
regeneration only through having more HP (regeneration is a percentage of max HP).

### 6.5 Blows: what stats buy

Worked examples from the blows formula (`src/player-calcs.c:calc_blows()`, table
`blows_table`); values are blows per turn for the *displayed* STR (rows) and DEX (columns).
Warrior with a light weapon (5 lb):

| STR \ DEX | 16 | 17 | 18/20 | 18/50 | 18/80 | 18/100 |
|---|---|---|---|---|---|---|
| 17 | 1.05 | 1.17 | 1.33 | 1.66 | 2.38 | 2.85 |
| 18/00 | 1.17 | 1.33 | 1.66 | 2.00 | 2.85 | 3.33 |
| 18/20 | 1.66 | 2.00 | 2.38 | 2.77 | 3.57 | 4.00 |
| 18/50 | 2.77 | 3.03 | 3.33 | 3.57 | 4.16 | 4.54 |
| 18/100 | 3.44 | 3.84 | 4.16 | 4.54 | 5.00 | 5.26 |

Warrior with a 12 lb weapon:

| STR \ DEX | 16 | 17 | 18/20 | 18/50 | 18/80 | 18/100 |
|---|---|---|---|---|---|---|
| 18/00 | 1.00 | 1.05 | 1.17 | 1.33 | 2.00 | 2.38 |
| 18/50 | 1.17 | 1.33 | 1.66 | 2.00 | 2.85 | 3.33 |
| 18/100 | 2.00 | 2.38 | 2.77 | 3.03 | 3.70 | 4.16 |

A Mage (STR mult 2, 4 lb floor) with a 5 lb weapon at STR 18/00 gets 1.05-1.33 blows for DEX
17-18/50: melee is never the Mage's plan. Fractional blows are real: blows are spent as energy
(`100 × move_energy / blows` per blow), so 1.33 blows means four blows every three turns.
Full tables for all classes are generated by the formula in [Chapter 4](04-combat.md).

---

## 7. Point-based stats

The official page: you have a fixed pool, costs rise for higher values, unspent points become
gold, the pre-filled allocation is an algorithm's suggestion, and this is the recommended
method. The facts below refine (and in one place contradict) that.

### 7.1 The rules

FACT (`src/player-birth.c`):

- Every stat starts at base **10**; you have **20 points** (`MAX_BIRTH_POINTS`).
- Cost of raising a stat *to* a value (`birth_stat_costs`): 11-16 cost 1 point each, 17 costs 2,
  18 costs 4. Cumulative cost from 10: 16 = 6 points, 17 = 8 points, 18 = 12 points.
- Base 18 is the maximum.
- **Unspent points:** the code computes `au_birth = 600 + 50 × points left` while you allocate
  (`recalculate_stats()`), but when you accept the character `get_money()` resets gold to the
  flat 600 (`p->au = p->au_birth = z_info->start_gold`). On this commit, unspent points therefore
  appear to give **no** gold, despite the official manual. ADVICE: spend all 20 points.

Every legal full spend (ignoring order) is one of these shapes:

| Shape | Points | Typical use |
|---|---|---|
| 18 / 17 / 10 / 10 / 10 | 12+8 | two-stat builds: STR+DEX fighter, or spell stat + CON |
| 18 / 16 / 12 / 10 / 10 | 12+6+2 | spell stat 18, CON 16, a little STR for carrying |
| 18 / 14 / 14 / 10 / 10 | 12+4+4 | spread |
| 17 / 17 / 14 / 10 / 10 | 8+8+4 | three-stat fighter (STR/DEX/CON) |
| 17 / 16 / 16 / 10 / 10 | 8+6+6 | three-stat hybrid |
| 16 / 16 / 16 / 12 / 10 | 6+6+6+2 | four-stat spread (rarely right) |

### 7.2 Why 17 and 18 are the important numbers

- A base of 17 plus a +1 modifier reaches 18, and every further modifier point adds 10
  percentile. Base 18 plus modifiers reaches one band higher than base 17 (e.g. a Gnome Mage's
  INT: base 17 → 18/40; base 18 → 18/50, which moves minimum fail from 5% to 4% and mana from
  190 to 200 per 100 levels).
- Base 17 costs 8 and base 18 costs 12: the 18th point costs as much as four points elsewhere.
  It is worth it for the one stat that drives the class (spell stat for a pure caster; STR for a
  heavy-weapon fighter) and rarely otherwise.
- Gaining a level never raises a stat: the level-up loop in `src/player.c` only *restores*
  drained stats (it calls `EF_RESTORE_STAT` for all five). Permanent gains come from potions
  (Strength, Intelligence, ..., Augmentation) from the midgame onward
  ([Chapter 7](07-consumables-and-devices.md)). Your birth allocation is therefore what you
  live with for the first 20-30 character levels.

### 7.3 The default allocation algorithm

FACT (`src/player-birth.c:generate_stats()`): the pre-filled allocation is computed as

1. Buy base STR up to 17.
2. Unless the class is a "caster" (max blows below 5: Mage, Druid, Priest, Necromancer), buy DEX
   up to 17 while tracking the DEX at which blows last improved with the starting weapon, then
   sell back DEX that did not add a blow.
3. Spend up to half the remaining points on the spell stat (Warriors skip this and may use all
   remaining points on CON), then up to half on CON (CON capped at base 16). The source comment
   says the spell stat is capped at 16 for hybrids, but the code's cap is 18 for everyone.
4. Spend any leftover on DEX, then the non-spell mental stat.

Worked results (derived by hand from the code): a pure caster gets STR 17 (8 points), then half
of the remaining 12 on the spell stat (16) and the other 6 on CON (16): **STR 17, spell stat 16,
CON 16**. A Warrior gets STR 17, DEX 17 (unless the top DEX points add no blow with the Dagger
and are sold back), and the remaining points on CON: typically **17/17/14**.

ADVICE: the algorithm buys STR 17 for *every* class, including Mages and Necromancers. That is a
defensible choice (carrying capacity, a usable melee fallback), but most experienced players
prefer the spell stat at 18 for a pure caster. Treat the default as a reasonable fallback, not
an optimum.

### 7.4 Recommended allocations (ADVICE)

Base values (before race/class modifiers). "Rest" stays at 10.

| Class | Recommended | Alternative | Reasoning |
|---|---|---|---|
| Warrior | STR 17, DEX 17, CON 14 | STR 18, DEX 17 | Blows need both STR and DEX; the 17 DEX column is the first DEX blow breakpoint. CON 14 gives +0.25 HP/level. With a heavy-weapon plan (Half-Troll), STR 18 |
| Paladin | STR 17, DEX 17, CON 14 | STR 17, DEX 16, WIS 16 | Melee first; the early prayers (Bless, Call Light, Minor Healing) are cheap, so WIS can wait for potions |
| Blackguard | STR 18, CON 16, DEX 12 | STR 17, DEX 17, CON 14 | 10 lb minimum weapon weight makes STR the dominant blows stat; INT matters for spells but their mana comes from combat, not resting |
| Mage | INT 18, CON 16, STR 12 | INT 18, DEX 17 | Spell stat first (mana, fail, number of spells), then HP; STR 12 keeps you under the weight limit with books and potions |
| Necromancer | INT 18, CON 16, STR 12 | INT 18, DEX 17 | as Mage |
| Priest | WIS 18, CON 16, STR 12 | WIS 18, STR 17 | WIS for prayers; Priests melee better than Mages (blessed hafted weapons), so STR 17 is a defensible alternative |
| Druid | WIS 18, CON 16, STR 12 | WIS 18, DEX 17 | Druid's early damage is Stinking Cloud, not melee |
| Rogue | DEX 17, INT 17, CON 14 | DEX 18, STR 17 | Rogue spells start at CL5 and stay cheap; DEX drives blows, to-hit, AC, theft, and throwing accuracy. Consider STR 17 if you plan to melee or throw heavy weapons |
| Ranger | DEX 17, STR 17, CON 14 | DEX 18, WIS 17 | Shots come from level, not DEX; DEX/STR help to-hit and the bow's weight limit, CON keeps you alive. WIS can stay low early (first spell CL3, few spells) |

Check the birth screen afterwards: the displayed blows ("Blows x.x/turn" with the starting
weapon) and mana are recomputed live, so you can adjust a point and watch the effect.

---

## 8. The standard roller

The official page says the standard roller "rolls three six-sided dice for each stat". **The
code does something else** (FACT, `src/player-birth.c:get_stats()`):

- Each stat is `5 + 1d3 + 1d4 + 1d5` (range 8-17, mean 12.5).
- The 15 dice are rerolled until their total is strictly between 35 and 45, so the five base
  stats sum to 61-69.
- Race and class modifiers are then applied as usual.
- After rolling, buying and selling stats is locked out (`rolled_stats = true`). `r` rolls again;
  the previous roll can be restored.

Compare point-buy: base sums of 65 (18/17/10/10/10) to 70 (16/16/16/12/10), with full control.
A roll can give 17 in a stat for free alongside decent other stats (e.g. 17/15/14/12/11), which
point-buy cannot, but it can never give 18, and you must reroll many times to get a 17 where you
need it.

ADVICE: use point-buy. The roller is only better if you want several middling stats rather than
one or two high ones (e.g. a Ranger who wants STR, DEX, WIS and CON all around 14-15), and it costs
rerolling time. An automated player should always use point-buy (deterministic).

---

## 9. Birth options

All birth options live in `src/list-options.h`; they can only be set at birth (or on the quick
restart of a dead character). Table of defaults and effects (FACT, behaviour verified in the
listed code):

| Option | Default | What it actually does | Code |
|---|---|---|---|
| `birth_randarts` | off | Generates a random artifact set at birth (`do_randart()`); standard artifacts are replaced | `player-birth.c:do_cmd_accept_character()` |
| `birth_connect_stairs` | **on** | After taking stairs you arrive on a staircase: a down staircase if you went up, an up staircase if you went down (no up staircase is created under force-descend) | `gen-util.c:new_player_spot()` |
| `birth_force_descend` | off | Up stairs do nothing; down stairs and recall-from-town take you to max depth + 1; no recall out of an unfinished quest level; teleport level always goes down | `cmd-cave.c`, `player-util.c`, `effect-handler-general.c` |
| `birth_no_recall` | off | Word of Recall has no effect (until you win); the starting Recall scroll is not given | `effect-handler-general.c:effect_handler_RECALL()` |
| `birth_no_artifacts` | off | No artifacts are ever generated | `obj-make.c` |
| `birth_stacking` | **on** | Several objects may share a floor grid; off: dropped items spread out and may vanish | `obj-pile.c` |
| `birth_lose_arts` | off | Any artifact still on the level when you leave is lost forever, and a level with an artifact gets the top object feeling (`calc_obj_feeling()` returns 10). With the option off, only artifacts you have *seen and recognised* are lost on leaving; unseen ones may be generated again | `generate.c` |
| `birth_feelings` | **on** | Level feelings are shown | `cmd-cave.c:display_feeling()` |
| `birth_no_selling` | **on** | Shops pay 0 for items (they still take and identify them); gold generated in the dungeon is ×5 | `store.c`, `obj-make.c:make_gold()` |
| `birth_start_kit` | **on** | Start with the class kit (its cost deducted from 600 gold); off: only one food and one light, keep the gold | `player-birth.c:player_outfit()` |
| `birth_ai_learn` | **on** | Monsters that observe your resistances/abilities remember them (smart monsters always, others half the time, occasionally forgetting) and avoid spells you resist | `mon-util.c:update_smart_learn()`, `mon-attack.c` |
| `birth_know_runes` | off | All runes are known at birth: worn items are fully identified on walk-over | `player-birth.c` |
| `birth_know_flavors` | off | All consumable flavours are known at birth | `player-birth.c` |
| `birth_levels_persist` | off | Experimental: every level is kept and restored when revisited; you arrive on the matching staircase; recall lets you choose a visited depth | `generate.c`, `gen-util.c`, `effect-handler-general.c` |
| `birth_percent_damage` | off | Experimental "O-combat": to-dam adds a percentage to dice instead of flat damage; criticals add dice; minimum 2 blows | `player-attack.c:o_melee_damage()`, `player-calcs.c:calc_blows()` |

### 9.1 How each option changes strategy (ADVICE unless marked)

**`birth_connect_stairs` (keep on).** This is what makes "stair-scumming" possible: you arrive
on a staircase, look around, and if the level is dangerous or uninteresting you immediately take
the stairs again. It is the single biggest safety tool in the game for a careful player and for
a borg: every new level starts with an escape under your feet. With it off you must find stairs
(or read Deep Descent/Teleport Level) to leave a bad level. See [Chapter 10](10-exploration.md).

**`birth_force_descend`.** You can never go up, every staircase down puts you one level below
your *maximum* depth, and recall from town returns to max depth + 1 (FACT). Consequences: you
cannot retreat to an easier level to farm, so your power must keep up with the dive; stair-scum
is still possible (down stairs only). Combined with `birth_no_recall` it is the old "ironman"
mode: no town after the first descent, so you live on what you find. Not recommended for a
first win or for a new borg.

**`birth_no_recall`.** Removes the town as a resupply point (unless you walk up). Healing
potions, Phase Door and flasks of oil become finite resources. Pair only with deliberate
challenge play.

**`birth_no_selling` (on by default).** With it on, gold comes only from the floor, from monster
drops and from digging (×5 normal). Selling is replaced by "giving" (useful for free
identification by the shopkeeper). Strategy: pick up gold, dig treasure veins (Dwarves see them),
and do not haul loot to town. With it off, selling found items (especially at the start) is a big
early income source and you will want to carry valuable drops. A borg is simpler with
`no_selling` on: it never needs to evaluate resale value.

**`birth_start_kit` (keep on).** The kit is priced at shop value, so turning it off gives the
same total value in gold with more freedom but also more early shopping decisions. Turn it off
only if you have a specific shopping plan (e.g. a Warrior who wants a Sling and Flasks of Oil
instead of armour).

**`birth_feelings` (keep on).** Level feelings tell you how dangerous a level is on arrival and,
after some exploration, how good its loot is (see [Chapter 9](09-dungeon.md)). They are free
information. Turn off only for challenge.

**`birth_lose_arts`.** Makes every artifact feeling ("superb feeling") a must-explore level and
punishes stair-scumming past good levels. Leaving it off lets you skip a level without losing
unseen artifacts.

**`birth_randarts`.** Randarts are generated to match the standard set's power and frequency
roughly (official manual). You lose all knowledge of "where is Ringil" but the game remains
balanced. Some randarts are much better than standard ones for a specific build; some classic
lifesavers (the Phial, specific artifacts with Teleport activations) may not exist. A borg that
evaluates items by their properties rather than by name handles randarts fine.

**`birth_no_artifacts`.** Removes artifacts; makes resistances and speed much harder to collect.
A pure challenge setting.

**`birth_ai_learn` (on by default).** The official manual says it "makes the game very
difficult and is not recommended", yet it is on by default. With learning on, a monster that has
seen you resist fire stops breathing fire at you in favour of something you do not resist; a
monster that knows you have no mana stops casting Drain Mana. The practical effect is that
resistances are less of a "free" defence against the specific monsters that have learned them,
and that spell-casting monsters choose their best remaining attack. ADVICE: for a human player
it is a modest difficulty increase; for a borg's danger model it means "a monster that has
observed resistance X will prefer non-X attacks", so danger should be computed over the
spells it would still choose. Turning it off makes monster spell choice simpler to predict.

**`birth_know_runes` / `birth_know_flavors`.** These remove the identify-by-use game
([Chapter 6](06-equipment.md), [Chapter 7](07-consumables-and-devices.md)). They do not make
the character stronger, only better informed: you never drink an unknown potion of Death, never
wear an unknown cursed item without seeing its curse runes. They are strongly recommended for a
borg (removes a whole class of risky experiments and knowledge tracking) and harmless for a
learning player who does not enjoy ID.

**`birth_stacking` (keep on).** Off makes floor management tedious and lets items vanish.

**`birth_levels_persist` (experimental).** Levels persist; you arrive on the matching
staircase. Monsters you left alive are still there; uniques do not reset. Strategically this
removes "new level = new chances" and makes stair-scumming impossible in the usual sense
(the level on the other side of a staircase is always the same). Not recommended unless you want
that style.

**`birth_percent_damage` (experimental).** Changes melee and missile damage to "deadliness"
scaling (each +1 to-dam adds a percentage to the dice average, capped at 150 deadliness; slays
and brands multiply dice; criticals add dice) and guarantees at least 2 blows
(FACT, `o_melee_damage()`, `calc_blows()`). The official manual warns it is "not very balanced".
Damage estimates elsewhere in this manual assume it is **off**.

### 9.2 Recommended option sets (ADVICE)

| Goal | Changes from default |
|---|---|
| First character / learning | defaults; optionally `birth_know_flavors` on |
| First win | defaults (connected stairs, feelings, start kit, no selling) |
| Automated player (borg) | defaults plus `birth_know_runes` and `birth_know_flavors` on; consider `birth_ai_learn` off for simpler danger modelling; never `force_descend`, `no_recall`, `levels_persist` or `percent_damage` until the basics work |
| Variety for veterans | `birth_randarts` on |
| Ironman challenge | `birth_force_descend` + `birth_no_recall` (+ `birth_connect_stairs` off for the hard version) |

---

## 10. Choosing a combination

### 10.1 For a beginner (ADVICE)

| Combination | Why |
|---|---|
| **Half-Troll Warrior** | Highest HP (hit die 21), STR 18/xx at birth, regeneration, fear immunity at CL30; nothing to manage except melee and devices. Weakness: poor devices and saves, so buy Free Action and use Flasks of Oil / Potions of Cure Light Wounds generously |
| **Dunadan Warrior** | Slightly less HP, much better skills, sustain CON; a smoother all-round warrior |
| **Dwarf Priest** | Never blinded (can always read and pray), WIS +5 total, good HP for a caster; healing spells and Orb of Draining carry the midgame |
| **Dunadan Paladin** | Fights like a slightly weaker Warrior, gets Bless/Heroism/healing and later a free Word of Recall prayer |
| **Gnome Mage** | If you want to learn a caster: Free Action from birth, best device skill, Magic Missile/Phase Door/Fire Ball by CL6 |

Avoid as a first character: Necromancer (no light source, light penalties, unusual vision),
Blackguard (mana from combat, impaired regeneration, bloodlust side-effects), Hobbit Mage
(1d7 hit die).

### 10.2 For winning (ADVICE)

Classes that win most reliably have a combination of high HP or strong escapes, and a damage
source that scales to Morgoth's 20,000 HP band ([Chapter 12](12-game-plan-by-phase.md)):

- **Warrior** (Dunadan, High-Elf or Half-Troll): best melee, 6 blows cap, fear immunity; relies on
  devices for escapes and detection (Rods of Treasure Location/Detection, Staves of Teleportation,
  Potions of Healing).
- **Priest** (Dwarf, Dunadan, High-Elf): Healing (2000 HP) at CL30, Holy Word, Banish Evil;
  Restoration/Remembrance replace potions; Orb of Draining and Spear of Oromë for damage.
- **Mage** (High-Elf, Gnome, Dunadan): the strongest escapes and detection (Teleport Self CL7,
  Teleport Other CL15, Teleport Level CL28, Dimension Door CL35, Banishment/Mass Banishment CL35/40),
  Mana Storm at CL45.
- **Ranger** (High-Elf, Dunadan): extra shots from level and the best bow skill give the highest
  sustained ranged damage; Haste Self at CL25 from [Nature Craft].
- **Paladin**, **Rogue**, **Necromancer**, **Blackguard**, **Druid** all win in skilled hands;
  each has a sharper learning curve ([Chapter 14](14-class-playbooks.md)).

Race choice for a winner: Dunadan (no drawbacks), High-Elf (See Invisible + skills, but 145%
exp), Half-Troll (melee HP), Dwarf (Priest/Druid), Gnome (Mage, Free Action).

### 10.3 For an automated player (borg) (ADVICE)

A borg benefits from characters whose survival depends on few, reliable decisions:

1. **Dunadan or High-Elf Warrior.** No spell bookkeeping; damage = melee + missiles; all escapes
   are consumables whose fail rates the borg can compute from device skill. High-Elf See
   Invisible removes the "unseen attacker" problem; its device skill (+20) makes Staves of
   Teleportation and wands reliable. Dunadan has better HP growth via CON +5 total and sustain CON.
2. **High-Elf or Dunadan Ranger.** Ranged damage that scales with level automatically (extra
   shots), fire-at-nearest targeting is simple, and the class has Haste Self by spell.
3. **Dwarf/Dunadan Priest or Gnome/High-Elf Mage.** Spells give unlimited Phase Door (Mage CL2),
   detection, and healing (Priest), reducing consumable dependence, at the cost of mana
   management and spell-choice logic. Priests learn spells randomly within a book
   (`do_cmd_study_book()`), which a borg must handle.

Avoid for a borg until the basics work: Necromancer (light/dark vision rules, +25% fail on lit
grids, many self-damaging spells), Blackguard (mana decays, regeneration impaired, bloodlust
can force attacks), Druid (shapechanges block item use).

### 10.4 A decision procedure

1. Pick the class by play style: melee (Warrior, Paladin, Blackguard), missiles (Ranger), stealth
   and utility (Rogue), arcane spells (Mage), divine spells and healing (Priest), nature control
   (Druid), dark spells (Necromancer).
2. Pick the race by the class's prime stats and weaknesses:
   - INT casters: Gnome (Free Action), High-Elf (See Invisible), Dunadan, Hobbit (Hold Life, low HP).
   - WIS casters: Dwarf (blindness immunity), Dunadan, Hobbit (Druid), High-Elf for Rangers.
   - Melee: Half-Troll, Dunadan, Dwarf, High-Elf, Half-Orc.
   - Stealth/missiles: Hobbit, Kobold, High-Elf, Elf.
3. Allocate points per [section 7.4](#74-recommended-allocations-advice); spend all 20.
4. Keep the default birth options unless you have a reason ([section 9.2](#92-recommended-option-sets-advice)).
5. Name, accept, and read [Chapter 14](14-class-playbooks.md) for your class and
   [Chapter 12](12-game-plan-by-phase.md) for the first trip.

---

## Things to try (untested)

1. **Roll for a 17 in the "third" stat.** For a Ranger or Paladin who wants four stats around
   14-16, reroll the standard roller until it offers 17 in the prime stat and 14+ in two others,
   and compare the CL20 character against a point-buy twin. Test: two characters with the same
   seed of play decisions (or a borg with fixed config), compare HP, blows, mana at CL20.
2. **Unspent points as a check of the gold bug.** Create a character, leave 4 points unspent,
   and verify the gold shown after accepting. If it is 600 minus the kit (not +200), the
   "unspent points become gold" statement in the official manual is wrong for this commit;
   report it upstream.
3. **STR 17 default vs INT 18 for Mages.** The default algorithm gives Mages STR 17. Play the
   first 15 levels of a Gnome Mage both ways and record deaths, time to CL15, and how often the
   STR-17 Mage meleed successfully. Hypothesis: STR 17 is better before CL10 (melee fallback,
   carrying), INT 18 better after.
4. **Heavy Blackguard opening.** Spend STR 18 and buy the heaviest weapon you can wield with full
   blows in town; Blackguards' 10 lb floor means a 15-18 lb weapon costs no blows over a light one
   at most STR values. Test: compare displayed blows and damage per turn for Tulwar vs Lance vs
   Lucerne Hammer at birth.
5. **Kobold Rogue for poison-heavy midgame.** rPois from birth plus Rogue stealth may let you
   dive through DL15-35 (drakes, hounds, Basilisks) faster than a Hobbit Rogue. Test: borg or
   human runs, record depth at CL25 and causes of near-deaths.
6. **No-start-kit Warrior with a sling and oil.** Turn off the kit; buy a Sling, Iron Shots and
   Flasks of Oil instead of armour. Hypothesis: much faster first 5 levels. Test: time and HP
   lost to reach DL5.
7. **`birth_ai_learn` off for a borg.** Run the same borg configuration with learning on and off
   for N characters and compare depth reached and the share of deaths from spells of an element
   the character resisted elsewhere. If the difference is large, the borg's danger model should
   model learned avoidance.
8. **Know-runes on, then measure ID value.** Play the same class with `birth_know_runes` on and
   off; record how many unknown-curse or unknown-rune incidents happen by CL30. This quantifies
   how much an ID-aware borg needs.
9. **High-Elf vs Dunadan Mage race test.** The 145% vs 120% exp costs the High-Elf several
   levels at a given depth early; See Invisible saves it later. Compare CL at DL20 and DL40 and
   deaths to invisible monsters.
10. **Dwarf Miner gold run under no-selling.** Dig every visible treasure vein on DL1-10 with a
    Dwarf (SEE_ORE shows them) and compare gold at DL10 with a non-Dwarf. Hypothesis: enough
    extra gold for early Potions of Cure Light Wounds and a Lantern.

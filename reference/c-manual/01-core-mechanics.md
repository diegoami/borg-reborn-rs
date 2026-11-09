# 1. Core Mechanics

How the game engine actually works: time and energy, speed, hitting and being hit, armour,
criticals, blows, the five stats and every table derived from them, skills, stealth and
monster awareness, hit points and regeneration, experience, food, light, spell and device
failure, saving throws, burden, digging, disarming and searching. Every formula below was read
from the game source (angband master `e473720ed`, i.e. 4.2.6 plus later master changes); where a
number differs from older versions or wikis, the source wins.

**Contents**

1. [Time: game turns, player turns and energy](#11-time-game-turns-player-turns-and-energy)
2. [Speed](#12-speed)
3. [To-hit: the universal hit roll](#13-to-hit-the-universal-hit-roll)
4. [Armour class and damage reduction](#14-armour-class-and-damage-reduction)
5. [Monster melee against the player](#15-monster-melee-against-the-player)
6. [Critical hits](#16-critical-hits)
7. [Blows per round](#17-blows-per-round)
8. [Shots, throws and ranges](#18-shots-throws-and-ranges)
9. [The five stats](#19-the-five-stats)
10. [Stat tables (complete)](#110-stat-tables-complete)
11. [Skills](#111-skills)
12. [Stealth, noise, scent and monster awareness](#112-stealth-noise-scent-and-monster-awareness)
13. [Hit points](#113-hit-points)
14. [Regeneration and resting](#114-regeneration-and-resting)
15. [Experience and levels](#115-experience-and-levels)
16. [Food and digestion](#116-food-and-digestion)
17. [Light, fuel and unlight](#117-light-fuel-and-unlight)
18. [Spell failure and mana](#118-spell-failure-and-mana)
19. [Device failure](#119-device-failure)
20. [Saving throw](#120-saving-throw)
21. [Carrying capacity and burden](#121-carrying-capacity-and-burden)
22. [Digging](#122-digging)
23. [Disarming, lock picking, searching and secret doors](#123-disarming-lock-picking-searching-and-secret-doors)
24. [Status effects that change combat numbers](#124-status-effects-that-change-combat-numbers)
25. [Quick reference for the borg](#125-quick-reference-for-the-borg)
26. [Things to try (untested)](#things-to-try-untested)

Conventions: "FACT" statements are taken from the cited source; "ADVICE" is judgement. Weights
in the source are in tenths of a pound (a 12 lb sword has weight 120). Stat values above 18 are
written 18/xx and stored internally as 18+xx (18/50 = 68). See
[Chapter 0](00-sources-and-conventions.md) for general conventions.

---

## 1.1 Time: game turns, player turns and energy

The official manual ([a-quick-demo](https://angband.readthedocs.io/en/latest/a-quick-demo.html))
only says that monsters move when you use a command that takes energy. The real model:

**FACT** (`src/game-world.c:run_game_loop()`, `turn_energy()`, `lib/gamedata/constants.txt`):

- The world advances in **game turns** (the global `turn` counter). Every game turn, every
  monster and the player gain energy equal to `extract_energy[speed]` (the table below; at
  normal speed this is 10). `world:move-energy` is 100.
- Anyone with at least 100 energy may act. A normal action (move, attack round, quaff, read,
  cast, fire one volley, rest one turn) costs 100 energy (`z_info->move_energy`). So at normal
  speed you act once every 10 game turns.
- "Free" commands (inventory, look, knowledge menus) cost 0.
- Fractional costs exist: melee spends `blow_energy` per blow (section 1.7), so a 3.5-blow
  character uses 84-100 energy per attack command and keeps the remainder; firing costs
  `100 * 10 / shots` (section 1.8); a shield-bash stumble adds 26-75 energy.
- Before the player acts, monsters with **more** energy than the player act first
  (`process_monsters(player->energy + 1)`); then the player acts; then the rest of the monsters.
- **Every 10 game turns** `process_world()` runs. It handles, in this order: random monster
  generation (1 in `mon-gen:chance` = 1 in 500 per call), poison and cut damage, bloodlust,
  timed healing, Black Breath, digestion, fainting/starvation, HP and mana regeneration, all timed
  effect countdowns (`decrease_timeouts()`), light fuel, noise and scent, `DRAIN_EXP` items,
  rod/activation recharge, trap timeouts, Word of Recall and Deep Descent countdowns.
- Monster HP regeneration happens once every 100 game turns (`process_monsters()`).

Consequences you should know:

| Thing | Measured in | At normal speed | At +10 speed |
|---|---|---|---|
| Your actions | energy | 1 per 10 game turns | 1 per 5 game turns |
| Timed effects (haste, resist, stun, poison, confusion...) | 10-game-turn ticks | 1 per action | 1 per 2 actions |
| HP/SP regeneration | 10-game-turn ticks | per action | per 2 actions |
| Light fuel | 10-game-turn ticks | 1 per action | 1 per 2 actions |
| Recall / Deep Descent delay | 10-game-turn ticks | per action | per 2 actions |
| Digestion | every 100 game turns, scaled by your energy | see 1.16 | same food per action |

**ADVICE.** Being fast makes every timed buff effectively last longer *in actions*: a 20-turn
potion of Speed gives 20 ticks, which is 40 actions at +10. Conversely, recall takes the same
game time however fast you are, so a fast character gets more actions while waiting for recall.

The character sheet shows `Game` turns, `Standard` turns (`total_energy / 100`, i.e. actions
normalised to normal speed) and `Resting` turns (`src/ui-player.c`).

## 1.2 Speed

**FACT.** Normal speed is 110 internally and is displayed as "Normal"; 120 is "Fast (+10)".
Energy per game turn is `extract_energy[speed]` (`src/game-world.c`):

| Displayed speed | Energy / game turn | Relative to normal |
|---|---|---|
| -20 (Slow) | 3 (index 90) | 0.3x |
| -10 | 5 | 0.5x |
| -5 | 6 (index 105) | 0.6x |
| Normal | 10 | 1.0x |
| +5 | 15 | 1.5x |
| +10 | 20 | 2.0x |
| +15 | 25 | 2.5x |
| +20 | 30 | 3.0x |
| +25 | 35 | 3.5x |
| +30 | 38 | 3.8x |
| +35 | 40 | 4.0x |
| +40 | 42 | 4.2x |
| +50 | 45 | 4.5x |
| +60 | 47 | 4.7x |
| +70 and up | 49 | 4.9x (cap) |

Full row values: from Normal to +20 each point of speed is worth one energy point (+1 = +10%
relative to normal); below Normal it drops unevenly (-1 = 9, -2 = 8, -4 = 7, -6 = 6, -10 = 5);
from +20 to +30 the curve flattens (36, 36, 37, 37, 38...); above +30 each
energy point costs about 3 points of speed. Speed is clamped to 0..199.

The official manual ([attack](https://angband.readthedocs.io/en/latest/attack.html), "A note on
speed") says +10 is exactly double and +20 about triple; the table confirms this.

Monster speed: non-unique monsters get a random spread of `rand_spread(0, energy/10)` speed
points at birth (`src/mon-make.c`), so a normal-speed race varies by +-1, a "+10" race by +-2
(+8 to +12) and a "+20" race by +-3. Hasted monsters
get +10; monster slow subtracts 2 per slow level.

Sources of player speed (FACT, `src/player-calcs.c:calc_bonuses()`): `SPEED` modifiers on gear,
`TMD_FAST` or `TMD_SPRINT` +10 (not cumulative with each other), `TMD_TERROR` +10, `TMD_SLOW`
-10, `TMD_STONESKIN` -5, being gorged (up to -10, section 1.16), burden (section 1.21). There is
also a `MOVES` modifier: `energy_per_move = 100 * (1 + |n| - n) / (1 + |n|)`, so +1 moves makes
*walking* cost 50 energy, +2 makes it 33 (`src/player-util.c:energy_per_move()`).

**ADVICE.** Relative speed is what kills. A +10 monster gets two turns per normal-speed turn of
yours; a +20 monster (most late hounds, dragons, uniques at +20/+30) gets three. Plan to reach
+10 by the mid-game and +20 or more by DL 60+, see [Chapter 12](12-game-plan-by-phase.md).

## 1.3 To-hit: the universal hit roll

The same function decides player melee, player missiles, monster melee, traps that roll against
your AC, and monster spells that "can miss".

**FACT** (`src/player-attack.c:test_hit()`, `hit_chance()`):

```
chance = max(9, to_hit)
P(hit) = 12% + 83% * max(0, chance - (2*AC)/3) / chance
```

So there is always a 12% chance to hit and always a 5% chance to miss; the maximum is 95%.

Your `to_hit` value ("skill + bonuses"):

| Attack | Formula (FACT) | Source |
|---|---|---|
| Melee | `skill_thn + 3 * (to_h_gear + to_h_weapon + 2 if blessed-wield)`; halved if the monster is not visible | `chance_of_melee_hit_base()` |
| Launcher | `skill_thb + 3 * (to_h_gear + to_h_bow + to_h_ammo) - distance`; halved if the monster is not "obvious" | `chance_of_missile_hit()` |
| Throwing weapon (has THROWING flag) | `skill_tht + 3 * (to_h_gear + to_h_object) - distance` | same |
| Any other thrown object | `1.5 * skill_tht + 3 * to_h_object - distance` (gear to-hit not added) | same |

`to_h_gear` is the to-hit from everything except the weapon and launcher slots, plus stat bonuses
(`adj_dex_th + adj_str_th`), plus timed effects (Bless +10, Hero +12, fear -20, stun -5/-20,
hunger penalties). The weapon's own to-hit is kept separately so that it does not leak into
missile attacks; the launcher's to-hit only counts for shots.

Hit chance table (rounded %, chance vs monster AC):

| to-hit chance | AC 10 | AC 20 | AC 40 | AC 60 | AC 80 | AC 100 | AC 120 | AC 150 | AC 200 |
|---|---|---|---|---|---|---|---|---|---|
| 50 | 85 | 73 | 52 | 29 | 12 | 12 | 12 | 12 | 12 |
| 75 | 88 | 81 | 66 | 51 | 36 | 22 | 12 | 12 | 12 |
| 100 | 90 | 84 | 73 | 62 | 51 | 40 | 29 | 12 | 12 |
| 150 | 92 | 88 | 81 | 73 | 66 | 58 | 51 | 40 | 21 |
| 200 | 93 | 90 | 84 | 78 | 73 | 68 | 62 | 54 | 40 |
| 250 | 93 | 91 | 86 | 82 | 77 | 73 | 68 | 62 | 51 |
| 300 | 93 | 91 | 88 | 84 | 80 | 77 | 73 | 67 | 58 |
| 400 | 94 | 92 | 90 | 87 | 84 | 81 | 78 | 74 | 67 |

Examples: a level-1 human warrior has `skill_thn` 70 + 45 x 1/10 = 74 and, with +3 to-hit from
stats and weapon, `chance` 83: vs a cave orc (AC 48) that is 12 + 83 x (83-32)/83 = 63%. A
level-30 warrior (skill 70 + 45 x 3 = 205) with +25 total to-hit has chance 280: vs an AC 100
monster, 75%.

**ADVICE.** Against high-AC targets (golems, dragons with AC 100-170) to-hit matters more than
damage. Each +1 to-hit is worth 3 points of skill; each class level is worth `x_skill/10` (4.5 for
warriors).

## 1.4 Armour class and damage reduction

Your AC is `ac + to_a` (base armour plus magical and stat bonuses; `adj_dex_ta`, Bless +5, Shield
+50, Stoneskin +40, Invulnerability +100, fear +8, berserk -10).

**FACT.** AC does two things:

1. **Avoiding hits.** Monster blows, the four monster missile spells that can miss (SHOT and
   ARROW and BOLT with `hit:50`, BOULDER with `hit:60`, to-hit `3 x level + hit`) and dart traps
   (`SAVE_ARMOR`, to-hit 125) roll `test_hit()` against your AC. Every other monster spell
   (all bolts, balls, breaths and status spells) has `hit:100` in
   `lib/gamedata/monster_spell.txt` and cannot be dodged by AC.
2. **Reducing damage**, but *only* for some blows (`src/mon-attack.c:adjust_dam_armor()`):

```
reduced = dam - dam * min(AC, 240) / 400        (at most 60% reduction)
```

| Your AC | HURT/SHATTER blow damage kept | Physical part of an elemental blow kept (uses AC+50) |
|---|---|---|
| 0 | 100% | 88% |
| 50 | 88% | 75% |
| 100 | 75% | 62% |
| 150 | 62% | 50% |
| 200 | 50% | 40% |
| 240+ | 40% | 40% |

Which blow effects are reduced by AC (FACT, `src/mon-blows.c`):

- `HURT` and `SHATTER`: reduced by `adjust_dam_armor(dam, AC)`.
- `ACID`, `ELEC`, `FIRE`, `COLD`, `POISON`: damage is `max(physical, elemental)`, where physical =
  `adjust_dam_armor(dam, AC+50)` if the blow method is physical (HIT, PUNCH, KICK, CLAW, BITE,
  STING, BUTT, CRUSH; not TOUCH, ENGULF, SPIT, GAZE...) and elemental = damage after your
  resistance. So resisting fire does not save you from a fire *bite*'s physical component.
- **Every other effect** (EXP_xx, LOSE_STR..., BLIND, CONFUSE, TERRIFY, PARALYZE, DISENCHANT,
  EAT_GOLD/ITEM/FOOD/LIGHT, DRAIN_CHARGES, HALLU, BLACK_BREATH) does its **full rolled damage**,
  ignoring AC. Only `DAM_RED` modifiers reduce it.

There is also flat damage reduction: `player_apply_damage_reduction()` subtracts `dam_red` (from
`DAM_RED` modifiers) and then a percentage (`perc_dam_red`, from shapes), and Invulnerability
makes any hit below 9000 do 0. This applies to essentially all damage (melee, spells, poison
ticks, cuts, starvation, traps).

The official manual says AC "reduces damage"; this is only true for the cases above.

**ADVICE.** High AC is very good against physical-melee monsters (trolls, giants, orcs, hounds'
bites) and useless against breathers and spellcasters. Past AC ~150 the benefit to damage
reduction is shrinking toward the 240 cap; past that only hit avoidance improves.

## 1.5 Monster melee against the player

**FACT** (`src/mon-attack.c:make_attack_normal()`, `chance_of_monster_hit_base()`):

```
monster to_hit = 3 * max(level, 1) + effect power       (x 0.75 if the monster is stunned)
```

Effect powers (`lib/gamedata/blow_effects.txt`): HURT 40, FIRE/ELEC/COLD 40, ACID 20, POISON 20,
CONFUSE 20, EXP_xx 20, DISENCHANT 10, DRAIN_CHARGES 10, SHATTER 60, and 0 for BLIND, TERRIFY,
PARALYZE, LOSE_xx, EAT_xx, HALLU, BLACK_BREATH. Effect `NONE` (e.g. beg/insult) always "hits".

Monster hit chance vs your AC (HURT blows, rounded %):

| Monster level (to-hit) | AC 20 | AC 50 | AC 80 | AC 100 | AC 130 | AC 160 | AC 200 | AC 250 |
|---|---|---|---|---|---|---|---|---|
| 5 (55) | 75 | 45 | 15 | 12 | 12 | 12 | 12 | 12 |
| 10 (70) | 80 | 56 | 32 | 17 | 12 | 12 | 12 | 12 |
| 20 (100) | 84 | 68 | 51 | 40 | 24 | 12 | 12 | 12 |
| 30 (130) | 87 | 74 | 61 | 53 | 40 | 27 | 12 | 12 |
| 40 (160) | 88 | 78 | 68 | 61 | 50 | 40 | 26 | 12 |
| 50 (190) | 89 | 81 | 72 | 66 | 57 | 49 | 37 | 22 |
| 60 (220) | 90 | 83 | 75 | 70 | 63 | 55 | 45 | 32 |
| 80 (280) | 91 | 85 | 79 | 75 | 70 | 64 | 56 | 46 |
| 100 (340) | 92 | 87 | 82 | 79 | 74 | 69 | 63 | 54 |

For effects with power 0 (paralyze, blind, drain stats, steal) subtract 40 from the to-hit:
a level-20 paralyser has to-hit 60, so AC 80 holds it to 22% and AC 90+ to the 12% floor.

Other facts:

- Damage is `randcalc(blow dice)`; stunned monsters do 25% less damage and hit 25% less often,
  and miss their turn 1 time in 10 (`src/mon-timed.h`).
- **Cuts and stuns from melee** (`monster_critical()`): blows whose method can cut (HIT, CLAW,
  BITE) or stun (HIT, PUNCH, KICK, BUTT, CRUSH) cause a cut/stun only when the damage roll is at
  least 95% of the blow's maximum; if both are possible one is chosen 50/50. Weak blows (<20
  damage) succeed only with probability dam%. Stun amounts by tier: 1d5, 10+1d10, 20+1d20,
  30+1d30, 40+1d40, 100, 200. Cut amounts: 1d5, 5+1d5, 20+1d20, 50+1d50, 100+1d100, 300, 500.
- Protection from Evil (`TMD_PROTEVIL`) repels a blow from an evil monster if
  `player level >= monster level` and `randint0(100) + player level > 50`.
- If you are moved during the monster's blows (e.g. teleported by a blow), the remaining blows
  are skipped.

Blow effects and their protections are detailed in
[Chapter 5](05-elements-and-status.md#58-monster-melee-blow-effects).

## 1.6 Critical hits

The standard combat model has data-driven criticals (`lib/gamedata/constants.txt`,
`src/player-attack.c:critical_melee()`, `critical_shot()`). These numbers are new in 4.2.x
master and differ from 3.x/4.1.

### Melee criticals (standard combat)

**FACT.**

```
chance (out of 5000) = weight + 5 * (to_h_gear + to_h_weapon [+10 if the monster is confused,
                       held, afraid or stunned]) + skill_thn - 60
power = weight + randint1(650)
```

| Power | Name | Damage becomes |
|---|---|---|
| < 400 | good hit | 2 x dmg + 5 |
| 400-699 | great hit | 2 x dmg + 10 |
| 700-899 | superb hit | 3 x dmg + 15 |
| 900-1299 | \*GREAT\* hit | 3 x dmg + 20 |
| 1300+ | \*SUPERB\* hit | 4 x dmg + 20 |

`dmg` here is dice x slay/brand multiplier + the **weapon's** to-dam. To-dam from rings, gloves,
stats and timed effects is added *after* the critical and is never multiplied. Unarmed attacks
never crit (no weapon). Player level has scale 0 in the current constants.

Worked example: level-30 human warrior, `skill_thn` = 70 + 45*30/10 = 205, total to-hit +25,
15 lb weapon (150): chance = 150 + 125 + 205 - 60 = 420 -> 8.4% per hit. Power = 151..800, so
38% good, 46% great, 15% superb. With a 40 lb weapon (400): chance 670 -> 13.4%, power
401..1050: 46% great, 31% superb, 23% \*GREAT\*.

**ADVICE.** Criticals are rare (under ~15% even for heavy weapons) in this version; weapon choice
should be dominated by blows x (dice + to-dam) x slays, see [Chapter 4](04-combat.md).

### Missile criticals (standard combat)

**FACT.**

```
chance (out of 5000) = missile weight + 4 * (to_h_gear + to_h_missile [+10 vs debuffed])
                       + 2 * player level
power = missile weight + randint1(500)
< 500 good (2x+5), 500-999 great (2x+10), 1000+ superb (3x+15)
```

The launcher's to-hit is not included. Arrows weigh 2 (0.2 lb), so a level-30 archer with +20
to-hit has chance 2 + 80 + 60 = 142 -> 2.8%; virtually all ranged crits are "good".

### O-combat (birth option `birth_percent_damage`)

With this birth option the game uses `o_critical_melee()` / `o_critical_shot()`: power is the
to-hit value scaled (melee x1/3, launched x1, thrown x3/2), crit chance is
`power / (power + 240)` for melee (`+360` for missiles), and a crit adds extra damage *dice*
(melee: +5 dice with chance 1/40, +4 1/12, +3 1/3, +2 1/2, else +1; missiles +3 1/50, +2 1/10,
else +1). To-dam becomes "deadliness", a percentage multiplier on dice sides
(`deadliness_conversion[]`, e.g. +10 = +39%, +20 = +69%, +30 = +99%). Minimum two blows. This
manual otherwise assumes the default (standard) combat.

## 1.7 Blows per round

The official manual ([attack](https://angband.readthedocs.io/en/latest/attack.html)) says warriors
get up to 6 blows, pure casters 4, others 5, and that fractions carry over as energy. The exact
formula:

**FACT** (`src/player-calcs.c:calc_blows()`, `blows_table[][]`, `lib/gamedata/class.txt`):

```
div        = max(weapon weight, class min-weight)          (tenths of a pound)
P          = min(11, adj_str_blow[STR] * class strength-multiplier / div)
D          = min(11, adj_dex_blow[DEX])
blow_energy = blows_table[P][D]
blows      = min(10000 / blow_energy, 100 * class max-attacks) / 100   (+ extra blows)
```

| Class | max-attacks | min-weight | strength-multiplier |
|---|---|---|---|
| Warrior | 6 | 30 (3 lb) | 5 |
| Paladin | 5 | 30 | 5 |
| Blackguard | 5 | 100 (10 lb) | 5 |
| Rogue | 5 | 20 | 4 |
| Ranger | 5 | 35 | 4 |
| Priest | 4 | 35 | 3 |
| Druid | 4 | 35 | 3 |
| Necromancer | 4 | 35 | 3 |
| Mage | 4 | 40 | 2 |

`blows_table` (energy per blow; rows P = 0..11, columns D = 0..11):

| P \ D | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 100 | 100 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 25 | 23 |
| 1 | 100 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 25 | 23 | 21 |
| 2 | 95 | 85 | 75 | 60 | 50 | 42 | 35 | 30 | 26 | 23 | 21 | 20 |
| 3 | 85 | 75 | 60 | 50 | 42 | 36 | 32 | 28 | 25 | 22 | 20 | 19 |
| 4 | 75 | 60 | 50 | 42 | 36 | 33 | 28 | 25 | 23 | 21 | 19 | 18 |
| 5 | 60 | 50 | 42 | 36 | 33 | 30 | 27 | 24 | 22 | 21 | 19 | 17 |
| 6 | 50 | 42 | 36 | 33 | 30 | 27 | 25 | 23 | 21 | 20 | 18 | 17 |
| 7 | 42 | 36 | 33 | 30 | 28 | 26 | 24 | 22 | 20 | 19 | 18 | 17 |
| 8 | 36 | 33 | 30 | 28 | 26 | 24 | 22 | 21 | 20 | 19 | 17 | 16 |
| 9 | 35 | 32 | 29 | 26 | 24 | 22 | 21 | 20 | 19 | 18 | 17 | 16 |
| 10 | 34 | 30 | 27 | 25 | 23 | 22 | 21 | 20 | 19 | 18 | 17 | 16 |
| 11 | 33 | 29 | 26 | 24 | 22 | 21 | 20 | 19 | 18 | 17 | 16 | 15 |

`D` by DEX: 3-9 -> 0, 10-16 -> 1, 17-18/19 -> 2, 18/20-18/39 -> 3, 18/40-18/59 -> 4,
18/60-18/79 -> 5, 18/80-18/99 -> 6, 18/100-18/119 -> 7, 18/120-18/149 -> 8,
18/150-18/179 -> 9, 18/180-18/199 -> 10, 18/200+ -> 11.

Other rules:

- **Too heavy**: if `adj_str_hold[STR] < weapon weight in lb`, you get `to_h -= 2 * (excess lb)`
  and exactly **1 blow** (`heavy_wield`), and the weapon's weight is not added to digging.
- Extra blows (`BLOWS` modifier on gear, e.g. weapons "of Extra Attacks") add whole blows
  *after* the class cap. Bloodlust adds `bloodlust/20` blows.
- With O-combat the minimum is 2 blows.
- In combat, `py_attack()` gives you blows while `energy_used + blow_energy <= min(energy, 100)`
  and charges only what you used; the remainder stays with you. So 3.84 blows really averages
  3.84 blows per 100 energy.

**Worked example** (the one asked for in the brief): a warrior with STR 18/50 and DEX 18/10
wielding a 12 lb sword. `adj_str_blow[18/50]` = 70, so P = 70 x 5 / 120 = 2 (integer); DEX 18/10
gives D = 2; `blows_table[2][2]` = 75, so blows = 10000/75 = 133 -> **1.33 blows**. The same
warrior with a 3 lb dagger (div = min-weight 30) has P = 350/30 = 11, table[11][2] = 26 ->
**3.84 blows**. With DEX raised to 18/50 (D = 4) the sword gives 2.00 blows and the dagger 4.54.

Warrior blows by STR (rows) and DEX (columns), 3 lb or lighter weapon:

| STR \ DEX | 10 | 17 | 18/20 | 18/50 | 18/70 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|---|---|---|
| 16 | 1.17 | 1.33 | 1.66 | 2.00 | 2.38 | 3.33 | 4.34 | 5.00 |
| 18/00 | 1.33 | 1.66 | 2.00 | 2.38 | 2.77 | 3.57 | 4.54 | 5.26 |
| 18/30 | 3.03 | 3.33 | 3.57 | 3.84 | 4.16 | 4.76 | 5.26 | 6.00 |
| 18/50 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/70 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/90 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/100 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/120 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/150 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |
| 18/200 | 3.44 | 3.84 | 4.16 | 4.54 | 4.76 | 5.26 | 5.88 | 6.00 |

Warrior, 12 lb weapon:

| STR \ DEX | 10 | 17 | 18/20 | 18/50 | 18/70 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|---|---|---|
| 16 | 1.00 | 1.05 | 1.17 | 1.33 | 1.66 | 2.38 | 3.33 | 4.34 |
| 18/00 | 1.00 | 1.05 | 1.17 | 1.33 | 1.66 | 2.38 | 3.33 | 4.34 |
| 18/30 | 1.17 | 1.33 | 1.66 | 2.00 | 2.38 | 3.33 | 4.34 | 5.00 |
| 18/50 | 1.17 | 1.33 | 1.66 | 2.00 | 2.38 | 3.33 | 4.34 | 5.00 |
| 18/70 | 1.33 | 1.66 | 2.00 | 2.38 | 2.77 | 3.57 | 4.54 | 5.26 |
| 18/90 | 1.66 | 2.00 | 2.38 | 2.77 | 3.03 | 4.00 | 4.76 | 5.55 |
| 18/100 | 2.00 | 2.38 | 2.77 | 3.03 | 3.33 | 4.16 | 4.76 | 5.88 |
| 18/120 | 2.00 | 2.38 | 2.77 | 3.03 | 3.33 | 4.16 | 4.76 | 5.88 |
| 18/150 | 2.77 | 3.03 | 3.33 | 3.57 | 3.84 | 4.54 | 5.26 | 5.88 |
| 18/200 | 3.12 | 3.44 | 3.84 | 4.16 | 4.54 | 5.00 | 5.55 | 6.00 |

Warrior, 20 lb weapon:

| STR \ DEX | 10 | 17 | 18/20 | 18/50 | 18/70 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|---|---|---|
| 16 | 1.00 | 1.05 | 1.17 | 1.33 | 1.66 | 2.38 | 3.33 | 4.34 |
| 18/00 | 1.00 | 1.05 | 1.17 | 1.33 | 1.66 | 2.38 | 3.33 | 4.34 |
| 18/30 | 1.05 | 1.17 | 1.33 | 1.66 | 2.00 | 2.85 | 4.00 | 4.76 |
| 18/50 | 1.05 | 1.17 | 1.33 | 1.66 | 2.00 | 2.85 | 4.00 | 4.76 |
| 18/70 | 1.17 | 1.33 | 1.66 | 2.00 | 2.38 | 3.33 | 4.34 | 5.00 |
| 18/90 | 1.17 | 1.33 | 1.66 | 2.00 | 2.38 | 3.33 | 4.34 | 5.00 |
| 18/100 | 1.33 | 1.66 | 2.00 | 2.38 | 2.77 | 3.57 | 4.54 | 5.26 |
| 18/120 | 1.33 | 1.66 | 2.00 | 2.38 | 2.77 | 3.57 | 4.54 | 5.26 |
| 18/150 | 1.66 | 2.00 | 2.38 | 2.77 | 3.03 | 4.00 | 4.76 | 5.55 |
| 18/200 | 2.00 | 2.38 | 2.77 | 3.03 | 3.33 | 4.16 | 4.76 | 5.88 |

All classes at selected STR/DEX (columns are weapon weights):



**ADVICE.**
- DEX is the main driver of blows once STR is enough to max P for your weapon; for light
  weapons STR 18/30+ already gives P = 11 for a warrior. Blackguards are the exception: their
  10 lb minimum weight means light weapons never help them.
- Casters (priest, mage, necromancer, druid) cap at 4 blows and are rarely worth building for
  melee; see [Chapter 4](04-combat.md) for blows x damage comparisons.

## 1.8 Shots, throws and ranges

**FACT** (`src/player-calcs.c`, `src/player-attack.c`):

- `num_shots` is stored in tenths: base 10 (1 shot), plus `SHOTS` modifiers, plus for rangers
  (`FAST_SHOT`) `level / 3` tenths (+1 shot at level 30). A shot costs `100 * 10 / num_shots`
  energy. If the launcher is too heavy (`adj_str_hold < launcher lb`) you lose
  `2 x excess` to-hit and get no extra shots or might.
- Launcher multiplier = launcher `pval` (sling 2, short bow 2, long bow 3, light crossbow 3,
  heavy crossbow 4) + `MIGHT` modifiers. Damage =
  `(ammo dice + ammo to-dam + launcher to-dam) x (multiplier + best slay/brand multiplier)` -
  slays add to the launcher multiplier (a x3 bow with a x3 slay arrow multiplies by 6). Gear
  to-dam does **not** apply to shots in standard combat (it does, as deadliness, in O-combat).
- Range for firing: `min(6 + 2 x multiplier, 20)` grids (x3 bow -> 12).
- Throwing: range `min((adj_str_blow[STR] + 20) x 10 / max(weight, 10), 10)` (weight in tenths
  of a pound); energy 100 per throw. Objects with the THROWING flag do
  `(dice + to-dam) x (2 + weight/12) x (1 + slay multiplier)` damage (weight in tenths of a pound,
  so a 1.2 lb dagger gets x3), add gear to-hit and (in O-combat) gear to-dam, and break only 1% of
  the time. Other thrown objects do `(dice + to-dam) x (1 + slay)`.
- Breakage on a miss is the square of the hit breakage chance (arrows 35% on hit -> 12% on miss;
  potions/flasks 100%).

## 1.9 The five stats

Stats run 3, 4, ..., 18, 18/10, ..., 18/100, ... Equipment and race/class bonuses can push the
*used* value up to 18/220 and beyond (index capped at "18/220+"). The official manual
([birth](https://angband.readthedocs.io/en/latest/birth.html)) describes each stat in words;
here is what each one does in the code.

**FACT** (`src/player-calcs.c:calc_bonuses()`, `src/player-spell.c`, `src/player-util.c`):

| Stat | Feeds |
|---|---|
| STR | to-dam (`adj_str_td`), to-hit (`adj_str_th`), carrying capacity (`adj_str_wgt`), max weapon/launcher weight (`adj_str_hold`), digging (`adj_str_dig`), blows (`adj_str_blow`), throwing range, shield-bash damage |
| INT | magic devices (`adj_int_dev`), magical disarm (`adj_int_dis`); for arcane/shadow casters (mage, rogue, necromancer, blackguard): spells known (`adj_mag_study`), mana (`adj_mag_mana`), min fail (`adj_mag_fail`), fail adjust (`adj_mag_stat`) |
| WIS | saving throw (`adj_wis_sav`); for divine/nature casters (priest, paladin, druid, ranger) the same four spell tables |
| DEX | to-hit (`adj_dex_th`), AC (`adj_dex_ta`), blows (`adj_dex_blow`), theft protection (`adj_dex_safe`), physical disarm and lock picking (`adj_dex_dis`), shield-bash chance and stumble |
| CON | HP per level (`adj_con_mhp`), recovery rate of stun/cut/poison (`adj_con_fix`) |

Note: CON does **not** change the HP regeneration rate in this version; regeneration is a
percentage of max HP (section 1.14). `adj_con_fix` only speeds the countdown of cuts, poison and
stun.

### How stats change

**Stat gain** (`player_stat_inc()`, used by potions of Strength, Intelligence, ... and
Augmentation, and by the stat-gain mushrooms/foods):

| Current (natural) value | Gain |
|---|---|
| 3-17 | +1 |
| 18 to 18/89 | `gain = ((18/100 - v)/2 + 3)/2` (in stat points), add `randint1(gain) + gain/2`, capped at 18/99. From 18/00: +14..+39; from 18/50: +8..+21; from 18/80: +4..+9 |
| 18/90-18/99 | becomes 18/100 |
| 18/100 | no further natural gain |

So the natural maximum is 18/100; anything above that comes from race/class/equipment
modifiers (`modify_stat_value()` adds 10 per point above 18). Potions of Brawn, Intellect,
Contemplation, Nimbleness and Toughness raise one stat and **permanently** lower a random other
one (`LOSE_RANDOM_STAT`).

**Stat drain** (`player_stat_dec()`): above 18/10 -> -10; 18/01..18/10 -> 18; 4..18 -> -1. Normal
drains (monster blows, traps, potions) lower only the *current* value; the maximum is kept, and
the stat shows in lower case/yellow. Sustains (`SUST_xxx`) block every `DRAIN_STAT` effect (stat-drain blows, darts, the
heat/cold/stench side effects of big fire, cold and poison attacks). Two sources call
`player_stat_dec()` directly and therefore **ignore sustains**: the time element and Black
Breath. See
[Chapter 5](05-elements-and-status.md#511-stat-drain-sustains-and-restoration).

**Restoration** (FACT): every time you gain a character level, **all five stats are restored**
to their maximum (`src/player.c:adjust_level()` calls `EF_RESTORE_STAT` for each stat). This
also happens when you regain a level lost to experience drain. Other restorers: mushroom of
Vigor (all), Purging (STR, CON), Sip of Miruvor and Draught of the Ents (STR, CON), any stat-gain
potion (restores that stat first), potion of Life, rod of Restoration, staff of the Magi (INT),
\*Enlightenment\* (INT, WIS), priest Restoration, druid Revitalize, blackguard Unholy Reprieve.
There are no plain "Restore STR" potions in this version.

## 1.10 Stat tables (complete)

Index column = the stat value actually in use (after race, class and equipment). All tables are
from `src/player-calcs.c` except `adj_mag_fail` and `adj_mag_stat` (`src/player-spell.c`).

**Strength**: to-dam bonus, to-hit bonus, weight limit (x10 lb), max weapon weight (lb),
digging, blow index.

| Stat | To-dam | To-hit | Weight limit (x10 lb) | Max weapon lb | Digging | Blow index |
|---|---|---|---|---|---|---|
| 3 | -2 | -3 | 5 | 4 | 0 | 3 |
| 4 | -2 | -2 | 6 | 5 | 0 | 4 |
| 5 | -1 | -1 | 7 | 6 | 1 | 5 |
| 6 | -1 | -1 | 8 | 7 | 2 | 6 |
| 7 | 0 | 0 | 9 | 8 | 3 | 7 |
| 8 | 0 | 0 | 10 | 10 | 4 | 8 |
| 9 | 0 | 0 | 11 | 12 | 4 | 9 |
| 10 | 0 | 0 | 12 | 14 | 5 | 10 |
| 11 | 0 | 0 | 13 | 16 | 5 | 11 |
| 12 | 0 | 0 | 14 | 18 | 6 | 12 |
| 13 | 0 | 0 | 15 | 20 | 6 | 13 |
| 14 | 0 | 0 | 16 | 22 | 7 | 14 |
| 15 | 0 | 0 | 17 | 24 | 7 | 15 |
| 16 | 1 | 0 | 18 | 26 | 8 | 16 |
| 17 | 2 | 0 | 19 | 28 | 8 | 17 |
| 18/00 | 2 | 1 | 20 | 30 | 9 | 20 |
| 18/10 | 2 | 1 | 22 | 30 | 10 | 30 |
| 18/20 | 3 | 1 | 24 | 35 | 12 | 40 |
| 18/30 | 3 | 1 | 26 | 40 | 15 | 50 |
| 18/40 | 3 | 1 | 28 | 45 | 20 | 60 |
| 18/50 | 3 | 1 | 30 | 50 | 25 | 70 |
| 18/60 | 3 | 1 | 30 | 55 | 30 | 80 |
| 18/70 | 4 | 2 | 30 | 60 | 35 | 90 |
| 18/80 | 5 | 3 | 30 | 65 | 40 | 100 |
| 18/90 | 5 | 4 | 30 | 70 | 45 | 110 |
| 18/100 | 6 | 5 | 30 | 80 | 50 | 120 |
| 18/110 | 7 | 6 | 30 | 80 | 55 | 130 |
| 18/120 | 8 | 7 | 30 | 80 | 60 | 140 |
| 18/130 | 9 | 8 | 30 | 80 | 65 | 150 |
| 18/140 | 10 | 9 | 30 | 80 | 70 | 160 |
| 18/150 | 11 | 10 | 30 | 90 | 75 | 170 |
| 18/160 | 12 | 11 | 30 | 90 | 80 | 180 |
| 18/170 | 13 | 12 | 30 | 90 | 85 | 190 |
| 18/180 | 14 | 13 | 30 | 90 | 90 | 200 |
| 18/190 | 15 | 14 | 30 | 90 | 95 | 210 |
| 18/200 | 16 | 15 | 30 | 100 | 100 | 220 |
| 18/210 | 18 | 15 | 30 | 100 | 100 | 230 |
| 18/220+ | 20 | 15 | 30 | 100 | 100 | 240 |

**Dexterity**: to-hit, AC, blow column D, theft/fall safety, physical disarm bonus.

| Stat | To-hit | AC | Blow column D | Theft safety | Disarm (phys) |
|---|---|---|---|---|---|
| 3 | -3 | -4 | 0 | 0 | 0 |
| 4 | -2 | -3 | 0 | 1 | 0 |
| 5 | -2 | -2 | 0 | 2 | 0 |
| 6 | -1 | -1 | 0 | 3 | 0 |
| 7 | -1 | 0 | 0 | 4 | 0 |
| 8 | 0 | 0 | 0 | 5 | 1 |
| 9 | 0 | 0 | 0 | 5 | 1 |
| 10 | 0 | 0 | 1 | 6 | 1 |
| 11 | 0 | 0 | 1 | 6 | 1 |
| 12 | 0 | 0 | 1 | 7 | 1 |
| 13 | 0 | 0 | 1 | 7 | 1 |
| 14 | 0 | 0 | 1 | 8 | 1 |
| 15 | 0 | 1 | 1 | 8 | 2 |
| 16 | 1 | 1 | 1 | 9 | 2 |
| 17 | 2 | 1 | 2 | 9 | 2 |
| 18/00 | 3 | 2 | 2 | 10 | 3 |
| 18/10 | 3 | 2 | 2 | 10 | 3 |
| 18/20 | 3 | 2 | 3 | 15 | 3 |
| 18/30 | 3 | 2 | 3 | 15 | 4 |
| 18/40 | 3 | 2 | 4 | 20 | 4 |
| 18/50 | 4 | 3 | 4 | 25 | 5 |
| 18/60 | 4 | 3 | 5 | 30 | 6 |
| 18/70 | 4 | 3 | 5 | 35 | 7 |
| 18/80 | 4 | 4 | 6 | 40 | 8 |
| 18/90 | 5 | 5 | 6 | 45 | 9 |
| 18/100 | 6 | 6 | 7 | 50 | 10 |
| 18/110 | 7 | 7 | 7 | 60 | 10 |
| 18/120 | 8 | 8 | 8 | 70 | 11 |
| 18/130 | 9 | 9 | 8 | 80 | 12 |
| 18/140 | 9 | 9 | 8 | 90 | 13 |
| 18/150 | 10 | 10 | 9 | 100 | 14 |
| 18/160 | 11 | 11 | 9 | 100 | 15 |
| 18/170 | 12 | 12 | 9 | 100 | 16 |
| 18/180 | 13 | 13 | 10 | 100 | 17 |
| 18/190 | 14 | 14 | 10 | 100 | 18 |
| 18/200 | 15 | 15 | 11 | 100 | 19 |
| 18/210 | 15 | 15 | 11 | 100 | 19 |
| 18/220+ | 15 | 15 | 11 | 100 | 19 |

**Intelligence / Wisdom / Constitution**: device bonus (INT), magic disarm bonus (INT), saving
throw (WIS), spells studied (% per level), mana (1/100 SP per level), minimum spell fail %,
spell fail adjustment, cut/poison/stun recovery (CON), extra HP (1/100 per level, CON).

| Stat | Device (INT) | Disarm magic (INT) | Save (WIS) | Study % (spell stat) | Mana /100 per lvl (spell stat) | Min fail % (spell stat) | Fail adj (spell stat) | Recovery (CON) | HP /100 per lvl (CON) |
|---|---|---|---|---|---|---|---|---|---|
| 3 | 0 | 0 | 0 | 0 | 0 | 99 | -5 | 0 | -250 |
| 4 | 0 | 0 | 0 | 0 | 10 | 99 | -4 | 0 | -150 |
| 5 | 0 | 0 | 0 | 10 | 20 | 99 | -3 | 0 | -100 |
| 6 | 0 | 0 | 0 | 20 | 30 | 99 | -3 | 0 | -75 |
| 7 | 0 | 0 | 0 | 30 | 40 | 99 | -2 | 0 | -50 |
| 8 | 1 | 1 | 1 | 40 | 50 | 50 | -1 | 0 | -25 |
| 9 | 1 | 1 | 1 | 50 | 60 | 30 | 0 | 0 | -10 |
| 10 | 1 | 1 | 1 | 60 | 70 | 20 | 0 | 0 | -5 |
| 11 | 1 | 1 | 1 | 70 | 80 | 15 | 0 | 0 | 0 |
| 12 | 1 | 1 | 1 | 80 | 90 | 12 | 0 | 0 | 5 |
| 13 | 1 | 1 | 1 | 85 | 100 | 11 | 0 | 0 | 10 |
| 14 | 1 | 1 | 1 | 90 | 110 | 10 | 1 | 1 | 25 |
| 15 | 2 | 2 | 2 | 95 | 120 | 9 | 2 | 1 | 50 |
| 16 | 2 | 2 | 2 | 100 | 130 | 8 | 3 | 1 | 75 |
| 17 | 2 | 2 | 2 | 105 | 140 | 7 | 4 | 1 | 100 |
| 18/00 | 3 | 3 | 3 | 110 | 150 | 6 | 5 | 2 | 150 |
| 18/10 | 3 | 3 | 3 | 115 | 160 | 6 | 6 | 2 | 175 |
| 18/20 | 3 | 3 | 3 | 120 | 170 | 5 | 7 | 2 | 200 |
| 18/30 | 3 | 4 | 3 | 130 | 180 | 5 | 8 | 2 | 225 |
| 18/40 | 3 | 4 | 3 | 140 | 190 | 5 | 9 | 2 | 250 |
| 18/50 | 4 | 5 | 4 | 150 | 200 | 4 | 10 | 3 | 275 |
| 18/60 | 4 | 6 | 4 | 160 | 225 | 4 | 11 | 3 | 300 |
| 18/70 | 5 | 7 | 5 | 170 | 250 | 4 | 12 | 3 | 350 |
| 18/80 | 5 | 8 | 5 | 180 | 300 | 4 | 15 | 3 | 400 |
| 18/90 | 6 | 9 | 6 | 190 | 350 | 3 | 18 | 3 | 450 |
| 18/100 | 6 | 10 | 7 | 200 | 400 | 3 | 21 | 4 | 500 |
| 18/110 | 7 | 10 | 8 | 210 | 450 | 2 | 24 | 4 | 550 |
| 18/120 | 7 | 11 | 9 | 220 | 500 | 2 | 27 | 5 | 600 |
| 18/130 | 8 | 12 | 10 | 230 | 550 | 2 | 30 | 6 | 650 |
| 18/140 | 8 | 13 | 11 | 240 | 600 | 2 | 33 | 6 | 700 |
| 18/150 | 9 | 14 | 12 | 250 | 650 | 1 | 36 | 7 | 750 |
| 18/160 | 9 | 15 | 13 | 250 | 700 | 1 | 39 | 7 | 800 |
| 18/170 | 10 | 16 | 14 | 250 | 750 | 1 | 42 | 8 | 900 |
| 18/180 | 10 | 17 | 15 | 250 | 800 | 1 | 45 | 8 | 1000 |
| 18/190 | 11 | 18 | 16 | 250 | 800 | 1 | 48 | 8 | 1100 |
| 18/200 | 11 | 19 | 17 | 250 | 800 | 0 | 51 | 9 | 1250 |
| 18/210 | 12 | 19 | 18 | 250 | 800 | 0 | 54 | 9 | 1250 |
| 18/220+ | 13 | 19 | 19 | 250 | 800 | 0 | 57 | 9 | 1250 |

Reading the tables:

- **CON**: HP bonus = `adj_con_mhp x level / 100`; at level 50 with CON 18/100 that is
  500 x 50 / 100 = +250 HP; with CON 18/200 it is +625; with CON 8 it is -12.
- **Spell stat**: mana = `1 + adj_mag_mana x (level - first spell level + 1) / 100`; spells known
  = `(adj_mag_study x levels + 50) / 100`.
- **Theft**: a thief's touch fails if `randint0(100) < adj_dex_safe + player level` (and you are
  not paralysed), so DEX 18/150+ (100) makes you immune to EAT_GOLD/EAT_ITEM.

## 1.11 Skills

**FACT** (`calc_bonuses()`): each skill is

```
skill = race skill + class base + stat adjustment + class increment x level / 10
        + equipment/timed modifiers
```

Class base / increment per 10 levels (`lib/gamedata/class.txt`):

| Class | Disarm phys | Disarm magic | Device | Save | Stealth | Search | Melee | Bows | Throw | Dig |
|---|---|---|---|---|---|---|---|---|---|---|
| Warrior | 25/15 | 20/10 | 18/7 | 18/10 | 0/0 | 10/12 | 70/45 | 55/45 | 55/45 | 0/0 |
| Mage | 30/10 | 35/12 | 36/13 | 30/9 | 2/0 | 10/12 | 35/15 | 20/15 | 20/15 | 0/0 |
| Druid | 30/10 | 30/10 | 24/10 | 30/10 | 3/0 | 12/12 | 45/20 | 40/30 | 40/30 | 0/0 |
| Priest | 25/12 | 25/12 | 30/10 | 32/12 | 2/0 | 10/14 | 45/20 | 35/20 | 35/20 | 0/0 |
| Necromancer | 30/10 | 35/12 | 36/13 | 30/9 | 2/0 | 10/12 | 35/25 | 20/15 | 20/15 | 0/0 |
| Paladin | 20/15 | 20/10 | 24/10 | 25/11 | 0/0 | 10/12 | 65/40 | 50/30 | 50/30 | 0/0 |
| Rogue | 45/20 | 45/20 | 32/10 | 28/10 | 3/1 | 20/16 | 35/45 | 66/30 | 72/45 | 0/0 |
| Ranger | 40/15 | 30/10 | 28/10 | 32/10 | 3/0 | 15/15 | 60/40 | 72/45 | 66/30 | 0/0 |
| Blackguard | 20/15 | 20/10 | 24/10 | 18/10 | -1/-1 | 8/10 | 65/40 | 35/15 | 40/30 | 0/0 |

Race skills, hit die, experience factor and infravision (`lib/gamedata/p_race.txt`):

| Race | D.phys | D.mag | Device | Save | Stealth | Search | Melee | Bows | Throw | Dig | Hit die | Exp % | Infra |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 10 | 100 | 0 |
| Half-Elf | 2 | 2 | 3 | 3 | 1 | 3 | -1 | 5 | 5 | 0 | 10 | 120 | 2 |
| Elf | 5 | 5 | 6 | 6 | 2 | 6 | -5 | 15 | 15 | 0 | 9 | 120 | 3 |
| Hobbit | 15 | 15 | 18 | 18 | 4 | 6 | -10 | 20 | 20 | 0 | 7 | 120 | 4 |
| Gnome | 10 | 10 | 22 | 12 | 3 | 4 | -8 | 12 | 12 | 0 | 8 | 120 | 4 |
| Dwarf | 2 | 2 | 9 | 9 | -1 | 2 | 15 | 0 | 0 | 40 | 11 | 120 | 5 |
| Half-Orc | -3 | -3 | -3 | -3 | -1 | -3 | 12 | -5 | -5 | 0 | 10 | 120 | 3 |
| Half-Troll | -5 | -5 | -8 | -8 | -2 | -9 | 20 | -10 | -10 | 0 | 12 | 120 | 3 |
| Dunadan | 4 | 4 | 5 | 5 | 1 | 3 | 15 | 10 | 10 | 0 | 10 | 120 | 0 |
| High-Elf | 4 | 4 | 20 | 20 | 2 | 10 | 10 | 25 | 25 | 0 | 10 | 145 | 4 |
| Kobold | 10 | 10 | 5 | 0 | 3 | 10 | -5 | 10 | 10 | 0 | 8 | 120 | 5 |

The player's hit die is race + class (`hitdie` in class.txt: warrior 9, blackguard 8, paladin 6,
ranger 5, rogue 4, priest/druid/necromancer 2, mage 0); the experience factor is race + class
(all classes add 0 in this version).

Stat adjustments added to skills: physical disarm `adj_dex_dis`, magical disarm `adj_int_dis`,
device `adj_int_dev`, save `adj_wis_sav`, digging `adj_str_dig`. Equipment: `STEALTH` modifier
+1 each, `SEARCH` modifier +5 each, `TUNNEL` modifier +20 each, digger DIG_1/2/3 = +20/40/60,
wielded weapon weight/10 lb to digging. Stealth is clamped to 0..30; digging to at least 1.

What each skill does (sections below give the formulas):

| Skill | Used for |
|---|---|
| Disarm (physical / magical) | disarming traps (by trap type), picking locks (physical) |
| Device | failure rate of wands, staves, rods and activations (1.19) |
| Save | resisting monster status spells, nexus side effects, some traps (1.20) |
| Stealth | how often sleeping monsters notice you; how far monsters can hear you (1.12) |
| Search | whether you notice a trap when it comes into view (1.23) |
| Melee / Bows / Throw | the `skill` part of to-hit (1.3) and melee criticals |
| Digging | tunnelling (1.22) |

The character screen describes stealth with words: `likert(stealth, 1)` gives 0-1 Bad, 2 Poor,
3-4 Fair, 5 Good, 6 Very Good, 7-8 Excellent, 9-13 Superb, 14-17 Heroic, 18+ Legendary.

Timed and status modifiers to skills (FACT, `calc_bonuses()`): Bless and Hero +5% device;
Berserk +75 melee skill and -10% device; stun -10% (Heavy Stun -20%) device; confusion -25%
device; amnesia -20%; hallucination -20%; poison -5%; fear -5%; `TMD_STEALTH` +10 stealth;
hunger penalties (section 1.16).

## 1.12 Stealth, noise, scent and monster awareness

The official manual only says stealth makes monsters less likely to wake. The mechanics
(**FACT**, `src/mon-move.c`, `src/game-world.c:make_noise()`, `update_scent()`):

### Noise map

Every 10 game turns (except while resting) the game floods a "noise" map outward from you:
your grid is 0, each step through passable or door grids adds 1 (adds 4 while
`TMD_COVERTRACKS` is active). Walls, rubble and other `NO_FLOW` terrain block it; **closed and
secret doors do not**. The value at a monster's grid is roughly its walking distance to you.

### Hearing

```
monster can hear you  <=>  noise at its grid > 0  and  hearing - stealth/3 > noise
```

`hearing` is per race (`hearing:` in monster.txt; most monsters 20, dragons and some hunters
30-100). With stealth 9 a hearing-20 monster hears you within 16 steps; with stealth 30, within
9. Monsters that hear you track you through corridors by moving to lower-noise grids.

### Scent

Each turn you lay scent in a 5x5 square (centre 0, inner ring 1, outer ring 2, not through
walls); all existing scent ages by 1 each 10 game turns. A monster with `smell` S can follow
scent younger than S (168 races have smell 20, 30 or 50; most have none). Cover Tracks stops
you laying scent.

### Who is "active"

A monster only does anything (including waking up) when it is active
(`monster_check_active()`): it is in your line of sight, or it can hear you, or it can smell
you, or it is hurt, or it passes walls and is within `hearing` grids, or terrain is hurting
it. A sleeping monster that is none of those **never wakes up on its own**.

### Waking up

Monsters are generated with sleep `2 x sleepiness + randint1(10 x sleepiness)` (sleepiness from
monster.txt; 0 = always awake). Each game turn an active sleeping monster has energy for, it
rolls:

```
notice = randint0(1024);  succeeds if notice^3 <= 2^(30 - stealth)
=> P(notice) ~= 2^(-stealth/3)
```

| Stealth | P per monster turn |
|---|---|
| 0 | 100% |
| 3 | 50% |
| 6 | 25% |
| 9 | 12.5% |
| 12 | 6.3% |
| 15 | 3.1% |
| 20 | 1.0% |
| 25 | 0.3% |
| 30 | 0.1% |

On success the sleep counter drops by `100 / noise` if the noise value at the monster is
between 1 and 49, otherwise by 1. So a monster adjacent to you (noise 1) loses 100 sleep per
success, at 5 steps 20, at 10 steps 10, beyond 50 steps (or unreachable) only 1.

Worked example: a cave orc (sleepiness 70, so sleep 141..840, average ~490) five steps away from
a stealth-6 character: 25% x 20 = 5 sleep per orc turn, ~100 orc turns to wake on average. At
one step: 25 sleep per turn, ~20 turns. With stealth 1 at one step: ~80/turn, wakes in ~6.

Other wake-ups (FACT): any melee attack on it (`monster_wake()`), damage from projections that
have `wake`, the `AGGRAVATE` flag (wakes every monster that becomes active, immediately), the
WAKE effect (shrieks, siren curse, aggravation/siren traps). Awake active monsters become
"aware" of you with chance 1/10 per turn.

**ADVICE.**
- Stealth matters mostly at distance: a monster right next to you wakes quickly even at good
  stealth. Fight sleeping monsters by sneaking up and killing them in one or two rounds, or shoot
  them from far away (missile damage wakes only the target).
- Doors do not stop noise; rubble and walls do. Closing a door does not hide you from hearing.
- Resting does not update noise or scent (`process_world()` skips them), so monsters that could
  not hear you when you started resting will not start hearing you while you rest unless they
  move.
- Aggravation (cursed items, `AGGRAVATE`) is lethal for deep diving: every monster that becomes
  active wakes instantly.

## 1.13 Hit points

**FACT** (`src/player-birth.c:roll_hp()`, `src/player-calcs.c:calc_hitpoints()`):

- Hit die `hd` = race hit die + class hit die (Human 10 + Warrior 9 = 19; Hobbit 7 + Mage 0 = 7).
- Level 1: `hd` HP. Each later level adds `randint1(hd)`. The whole 50-level sequence is rolled
  at birth and re-rolled until the level-50 total lies between `50 x (hd-1) x 3/8 + 50` and
  `50 x (hd-1) x 5/8 + 50` (so no extreme life rating).
- Max HP = `player_hp[level-1] + adj_con_mhp[CON] x level / 100`, minimum `level + 1`.

| Race+class hit die | Expected base HP at level 50 | Allowed range |
|---|---|---|
| 7 (Hobbit mage) | 7 + 49 x 4 = 203 | 162-237 |
| 12 (Human priest) | 12 + 49 x 6.5 = 330 | 256-393 |
| 16 (Human paladin) | 16 + 49 x 8.5 = 432 | 331-518 |
| 19 (Human warrior) | 19 + 49 x 10 = 509 | 387-612 |
| 21 (Half-troll warrior) | 21 + 49 x 11 = 560 | 425-675 |

Add the CON bonus: +250 at CON 18/100, +375 at 18/150, +625 at 18/200+ (level 50).

## 1.14 Regeneration and resting

**FACT** (`src/player-util.c:player_regen_hp()`, `player_regen_mana()`): once per 10 game
turns,

```
HP gain (1/65536 HP) = max_HP x percent + 1442
percent = 197 (food >= Weak threshold 800), 98 (400-799), 33 (100-399), 0 (starving)
          x (100 + (food/100)/3) / 100        (up to +33% when well fed)
          x 2 if REGEN, x 2 if resting (after 5 turns of rest, or immediately with R& / R*)
          / 2 if IMPAIR_HP
          = 0 if paralysed, poisoned, stunned or cut
SP gain = max_SP x percent' + 524, percent' = 197 x 2 (REGEN) x 2 (resting) / 2 (IMPAIR_MANA)
```

So HP regen is a fixed *percentage* of max HP (about 0.30-0.39% per tick awake, double resting)
plus a trickle of 0.022 HP; CON only matters through max HP. Mana does not depend on food.

| Situation (food) | % of max HP per tick | Ticks from 0 to full (300 HP) |
|---|---|---|
| Awake, Fed 90% | 0.39% | 252 |
| Awake, 50% | 0.35% | 282 |
| Resting, 90% | 0.78% | 127 |
| Resting, 50% | 0.70% | 143 |
| Resting + REGEN, 90% | 1.56% | 64 |
| Resting, Weak (food 700) | 0.30% | 324 |
| Resting, Faint (food 300) | 0.10% | 926 |

A tick is 10 game turns, i.e. one action at normal speed. Resting command counts are in *your*
turns, so a +10 character resting "100 turns" rests 50 ticks.

Blackguards (`COMBAT_REGEN`, intrinsic `IMPAIR_HP`): HP regen is halved; their mana *decays*
(percent becomes negative) and the lost mana heals HP; they gain mana from taking damage and
from each melee round (see [Chapter 14](14-class-playbooks.md)).

Resting modes (`src/player-util.c`): `R` with a count, `R&` "as needed" (stops when HP and SP
are full and no blindness, confusion, poison, fear, terror, stun, cut, slow, paralysis,
hallucination, recall or deep descent remain), `R*` "HP or SP" (stops when either is full).
Special modes get the resting bonus from the first turn; a counted rest only after 5 turns.

**ADVICE.** Always rest with `R&` rather than a number. Cure cuts, poison and stun before resting:
any of them stops HP regeneration completely.

## 1.15 Experience and levels

**FACT** (`src/player.c`):

- Level L+1 requires `player_exp[L-1] x expfact / 100` experience; `expfact` = race exp % +
  class exp % (class adds 0). Max level 50; max experience 99,999,999.
- Experience for a kill (`src/mon-util.c`): `monster exp x monster level / player level`, with
  the fraction kept. Example: a cave orc (exp 20, level 7) gives 140/L: 14 exp at level 10.
- Other sources: first successful cast of a spell gives `spell exp x spell level`; first use
  of an unknown item kind gives `(item level + player level/2) / player level`; disarming a
  trap gives `1 + depth/5`; potion of Experience; unlocking doors gives none.
- **Drained experience**: `player_exp_lose()` lowers current exp but not `max_exp` (unless the
  drain is permanent); you lose levels if exp falls below the threshold. While `exp < max_exp`,
  every experience gain also raises `max_exp` by 10% of the gain. Restore Life Levels, Life,
  rod of Restoration, priest Remembrance, druid Revitalize, blackguard Unholy Reprieve set exp
  back to max.
- Regaining a level (including a drained one) restores all stats (section 1.9).

Experience required (base table x factor):

| Level | Human (100%) | Most races (120%) | High-Elf (145%) |
|---|---|---|---|
| 2 | 10 | 12 | 14 |
| 3 | 25 | 30 | 36 |
| 4 | 45 | 54 | 65 |
| 5 | 70 | 84 | 101 |
| 6 | 100 | 120 | 145 |
| 7 | 140 | 168 | 203 |
| 8 | 200 | 240 | 290 |
| 9 | 280 | 336 | 406 |
| 10 | 380 | 456 | 551 |
| 11 | 500 | 600 | 725 |
| 12 | 650 | 780 | 942 |
| 13 | 850 | 1,020 | 1,232 |
| 14 | 1,100 | 1,320 | 1,595 |
| 15 | 1,400 | 1,680 | 2,030 |
| 16 | 1,800 | 2,160 | 2,610 |
| 17 | 2,300 | 2,760 | 3,335 |
| 18 | 2,900 | 3,480 | 4,205 |
| 19 | 3,600 | 4,320 | 5,220 |
| 20 | 4,400 | 5,280 | 6,380 |
| 21 | 5,400 | 6,480 | 7,830 |
| 22 | 6,800 | 8,160 | 9,860 |
| 23 | 8,400 | 10,080 | 12,180 |
| 24 | 10,200 | 12,240 | 14,790 |
| 25 | 12,500 | 15,000 | 18,125 |
| 26 | 17,500 | 21,000 | 25,375 |
| 27 | 25,000 | 30,000 | 36,250 |
| 28 | 35,000 | 42,000 | 50,750 |
| 29 | 50,000 | 60,000 | 72,500 |
| 30 | 75,000 | 90,000 | 108,750 |
| 31 | 100,000 | 120,000 | 145,000 |
| 32 | 150,000 | 180,000 | 217,500 |
| 33 | 200,000 | 240,000 | 290,000 |
| 34 | 275,000 | 330,000 | 398,750 |
| 35 | 350,000 | 420,000 | 507,500 |
| 36 | 450,000 | 540,000 | 652,500 |
| 37 | 550,000 | 660,000 | 797,500 |
| 38 | 700,000 | 840,000 | 1,015,000 |
| 39 | 850,000 | 1,020,000 | 1,232,500 |
| 40 | 1,000,000 | 1,200,000 | 1,450,000 |
| 41 | 1,250,000 | 1,500,000 | 1,812,500 |
| 42 | 1,500,000 | 1,800,000 | 2,175,000 |
| 43 | 1,800,000 | 2,160,000 | 2,610,000 |
| 44 | 2,100,000 | 2,520,000 | 3,045,000 |
| 45 | 2,400,000 | 2,880,000 | 3,480,000 |
| 46 | 2,700,000 | 3,240,000 | 3,915,000 |
| 47 | 3,000,000 | 3,600,000 | 4,350,000 |
| 48 | 3,500,000 | 4,200,000 | 5,075,000 |
| 49 | 4,000,000 | 4,800,000 | 5,800,000 |
| 50 | 4,500,000 | 5,400,000 | 6,525,000 |

**ADVICE.** Kill experience is divided by your level, so deep monsters are the only efficient
source late in the game; diving is the way to level, see [Chapter 10](10-exploration.md).

## 1.16 Food and digestion

**FACT** (`lib/gamedata/player_timed.txt`, `src/game-world.c:process_world()`,
`calc_bonuses()`). Food is the timed effect `TMD_FOOD`, stored as percent x 100
(`player:food-value` = 100):

| Grade | Food value | Effects |
|---|---|---|
| Starving | 1-100 | damage `(100 - food)/10` per tick; regen 0 |
| Faint | 101-400 | 1 in 10 per tick: paralysed 1-5 turns (ignores Free Action); regen 33 |
| Weak | 401-800 | to-hit/to-dam penalty (below); regen 98 (normal regen at exactly 800) |
| Hungry | 801-1500 | to-hit/to-dam penalty begins |
| Fed | 1501-9000 | no effect; regen bonus up to +30% |
| Full (gorged) | 9001-10000 | speed `-(food - 9000)/100` (up to -10); digests 50 per tick; normal digestion stops |

Hunger penalty below 1500: `lack = (1500 - food) x 20 / 1500` (0..20): to-hit and to-dam -lack;
lack 11-15 (food ~675-375): device -10%; 16-18: device -20%, disarm -10%; 19+: device -30%,
disarm -20%, save -10%, search -10%.

Digestion: every 100 game turns you lose `energy per game turn x 100 / 100` food = 10 at normal
speed (x2 with REGEN, /2 with SLOW_DIGEST, minimum 1). That is 1 food value per tick at normal
speed, independent of speed **per action** (a fast character digests faster per game turn but
the same per action). `TMD_HEAL` burns 800 food per tick.

Food values (`lib/gamedata/object.txt`, `NOURISH` x 100): Ration 3000 (30%), Slice of Meat 4000,
Honey-cake and Elvish Waybread "increase to" 60% / 75%, Apple/Biscuit 1000, Dried Fruits and
Slime Mold 1500, potions add 100, mushrooms 500. A ration therefore lasts ~3000 actions at
normal speed; going from Full (9000) to Hungry (1500) takes ~7500 actions.

**ADVICE.** Eat when "Hungry" appears; never let yourself reach Weak (lost to-hit/damage and
halved regen). Avoid gorging before a fight (speed penalty). Two or three rations or a few
Waybread cover a long dive; SLOW_DIGEST makes food a non-issue.

## 1.17 Light, fuel and unlight

**FACT** (`src/player-calcs.c:calc_light()`, `src/cave-view.c:calc_lighting()`, `add_light()`,
`lib/gamedata/object.txt`):

- Your light *intensity* is the sum over all equipped items of `2 (LIGHT_2) or 3 (LIGHT_3) +
  LIGHT modifier`. Light sources with no fuel give 0.
- A source of intensity L lights grids within distance `L - 1` (brightness `L - distance`), and
  you "see" (`close`) grids at distance `< L`. So a Wooden Torch (LIGHT_2) lights your square
  and the 8 adjacent squares; a Lantern (LIGHT_3) lights radius 2; an artifact light with
  LIGHT_3 and +1 light lights radius 3.
- In town during the day you need no light.
- Fuel: torches have 5000 turns (`obj-make:fuel-torch`) and burn out (are destroyed) at 0;
  lanterns hold up to 15000 and come with 7500; a Flask of oil adds 7500. One unit burns per
  tick. Artifact lights and NO_FUEL items do not burn. When blind, fuel never runs out
  completely. Warnings at < 50 turns.
- Light affects: seeing monsters/objects (without infravision or ESP), reading scrolls and
  casting (`no_light()` blocks both), disarming (skill /10 without light), lock picking
  (/10 if you cannot see the lock), searching (disabled in the dark), trap detection by sight.
- Infravision (`see_infra`, units of 10 ft) shows warm-blooded monsters in the dark.

**Unlight** (necromancers, `PF_UNLIGHT`; FACT): resist darkness while in the dungeon; when your
light intensity is 0 or 1 you see adjacent-ish grids anyway (`close` if distance
`< 2 + level/6 - light`); spell failure +25% when standing on a lit grid; LIGHT +1 gear counts
one less. Details in [Chapter 14](14-class-playbooks.md) and [Chapter 3](03-magic-and-spells.md).

**ADVICE.** Buy a Lantern as soon as affordable (radius 2 vs 1 is a large safety gain), carry
two flasks of oil, and swap to an artifact light when found. Flasks double as thrown weapons.

## 1.18 Spell failure and mana

**FACT** (`src/player-spell.c:spell_chance()`):

```
fail = spell base fail
     - 3 x (player level - spell level)
     - adj_mag_stat[casting stat]
     + 5 x (spell mana - current SP)          if you lack the mana
     + 25 if UNLIGHT and standing on a lit grid
     + 20 if afraid
min fail = adj_mag_fail[casting stat]; if the class lacks ZERO_FAIL, min fail is at least 5
fail = clamp(fail, min fail, 50)
fail += 25 if stun > 50 (Heavy Stun / Knocked Out), else +15 if stunned
if amnesia: fail = 50 + fail/2
fail = min(fail, 95)
```

ZERO_FAIL classes: Mage, Druid, Priest, Necromancer (min fail can reach 0% at 18/200+). Paladin,
Rogue, Ranger, Blackguard never go below 5%. Casting stat by realm (`lib/gamedata/realm.txt`,
books in `class.txt`): arcane = INT (mage, rogue), divine = WIS (priest, paladin), nature = WIS
(druid, ranger), shadow = INT (necromancer, blackguard).

Example: level-1 mage, INT 18/20 (`adj_mag_stat` 7, min fail 5) casting Magic Missile (level 1,
base fail 22): 22 - 0 - 7 = 15%. At level 5: 22 - 12 - 7 = 3 -> raised to min fail 5%.

You cannot cast while blind, without light or while confused (`player_can_cast()`); amnesia does
not stop casting but makes it unreliable. Casting with insufficient mana is allowed: you lose all
SP, are paralysed for `1d(5 x shortfall + 1)` turns even with Free Action (`PY_EXERT_FAINT`,
100% chance), and lose a point of CON 50% of the time - permanently in half of those cases
(`src/player-util.c:player_over_exert()`).

**Mana** (`calc_mana()`): `msp = 1 + adj_mag_mana[stat] x (level - first spell level + 1) / 100`,
minus 1 SP per pound of armour weight above the class limit (`magic:` line in class.txt:
mage 30 lb, necromancer 30, priest 35, druid 35, rogue 35, paladin 40, ranger 40, blackguard
60). Armour weight counts every slot except weapon, bow, rings, amulet and light.

**Gloves**: in this version there is **no glove penalty** for spellcasters (no glove
encumbrance code exists in `src/`). Older versions' "mages must not wear gloves" rule does not
apply.

Spells known: `(adj_mag_study x levels + 50) / 100`, where levels = level - first spell level + 1.

## 1.19 Device failure

**FACT** (`src/obj-util.c:get_use_device_chance()`, `src/cmd-obj.c`):

```
lev  = artifact level if artifact, else activation level if the item has an activation,
       else the object kind level
x    = 2 x (device skill - lev) + 1
fail = 380 - 370 x x / (5 + |x|)        (per mille; the item fails if randint1(1000) < fail)
```

| skill - item level | Fail % |
|---|---|
| -30 | 72 |
| -20 | 71 |
| -10 | 67 |
| -5 | 62 |
| 0 | 32 |
| +5 | 13 |
| +10 | 8 |
| +15 | 6 |
| +20 | 5 |
| +30 | 4 |
| +40 | 3 |
| +60 | 2.5 |
| +100 | 1.9 |

The curve is a steep sigmoid: being even 5 points of skill below the item level is
catastrophic, 10+ above is comfortable. The minimum is about 1%, the maximum 75%.

**The 4.2.6 change** (commit `7838cdc27`, April 2025): the level used for a non-artifact item
that has an activation is now the *activation's* level from `lib/gamedata/activation.txt`
rather than the object kind's level. In the current data no ego item has an `act:` line and
special light activations are attached to artifact-only kinds, so for ordinary play this
changes nothing: wands/staves/rods use kind level, artifacts use artifact level. (Verified by
grepping `act:` lines in `ego_item.txt` and `object.txt`.)

Device skill penalties: confusion -25%, stun -10% / heavy stun -20%, amnesia -20%,
hallucination -20%, berserk -10%, poison -5%, fear -5%, hunger up to -30%; Bless/Hero +5%.

Example: a level-25 human mage has device skill 36 + 13 x 25/10 = 68, plus `adj_int_dev` 5 at
INT 18/70 = 73: a Wand of Stone to Mud (kind level 10) fails ~2.5%. A level-30 human warrior
with INT 10 has 18 + 21 + 1 = 40: a Rod of Healing (level 80) fails ~73% of the time, a Staff of
Teleportation (level 20) ~5%. Item levels are listed in
[Appendix A6](appendices/A6-object-kinds.md).

## 1.20 Saving throw

**FACT**: `skill_save = race + class base + adj_wis_sav + class increment x level / 10`. A save
succeeds if `randint0(100) < skill_save`; values >= 100 always save. Saves apply to:

- Monster spells that have a `message-save` (`src/mon-spell.c:do_mon_spell()`): SCARE, BLIND,
  CONF, SLOW, HOLD, FORGET (amnesia), TELE_LEVEL, MIND_BLAST, BRAIN_SMASH, WOUND (cause wounds).
  Bolts, balls, breaths, teleport-to and teleport-away have **no** save.
- Melee blows TERRIFY and PARALYZE (a save before the protection check).
- Nexus side effects (stat scramble and teleport-level each get a save).
- Traps with `SAVE_THROW` (mind-blast and brain-smash traps).

A failed save is still blocked by the matching protection (FREE_ACT, PROT_CONF, PROT_BLIND,
PROT_FEAR), see [Chapter 5](05-elements-and-status.md). Example: level-50 human priest with WIS
18/100: 32 + 7 + 60 = 99%; level-50 human warrior with WIS 10: 18 + 1 + 50 = 69%.

## 1.21 Carrying capacity and burden

**FACT** (`calc_bonuses()`, `weight_limit()`):

```
limit = adj_str_wgt[STR] x 100      (tenths of a pound; = adj_str_wgt x 10 lb)
speed penalty = (total weight - limit/2) / (limit/10)   (integer division, only if positive)
```

Because of the integer division the first -1 appears at **60% of the limit**, then -1 for each
further 10%. The character screen's "Overweight" figure is weight minus 60% of the limit, so
any positive "Overweight" means at least -1 speed.

| STR | Limit | No penalty up to | Each -1 speed per |
|---|---|---|---|
| 10 | 120 lb | 72 lb | 12 lb |
| 15 | 170 lb | 102 lb | 17 lb |
| 18 | 200 lb | 120 lb | 20 lb |
| 18/30 | 260 lb | 156 lb | 26 lb |
| 18/50+ | 300 lb | 180 lb | 30 lb |

`adj_str_wgt` stops growing at 18/50: higher STR never increases capacity.

## 1.22 Digging

**FACT** (`src/player-calcs.c:calc_digging_chances()`, `src/cmd-cave.c:do_cmd_tunnel_aux()`):
digging skill = race (dwarf 40) + `adj_str_dig` + 20 x digger power (Shovel DIG_1 = 1,
Pick DIG_2 = 2, Mattock DIG_3 = 3) + 20 x TUNNEL modifier + wielded weapon weight in lb. The game
automatically uses your best digger from the pack for the attempt ("swap digger"). Chance per
turn, out of 1600:

| Terrain | Chance /1600 |
|---|---|
| Rubble | 8 x skill |
| Magma (incl. with treasure) | 4 x (skill - 10) |
| Quartz | 2 x (skill - 20) |
| Granite | skill - 40 |
| (Doors, if diggable) | (4 x skill - 119) / 3 |

Example: a dwarf with STR 18/50 (`adj_str_dig` 25) and a Pick (weight 18 lb): 40 + 25 + 40 +
18 = 123 -> rubble 61%, magma 28%, quartz 13%, granite 5.2% per turn (~19 turns). A human mage
with STR 10 (5) and no digger: rubble 2.5%, granite impossible. Rubble yields an object 10% of
the time (in the dungeon).

## 1.23 Disarming, lock picking, searching and secret doors

**FACT.**

- **Disarming traps** (`src/cmd-cave.c:do_cmd_disarm_aux()`): skill = magical disarm for
  MAGICAL traps (runes), physical otherwise; divided by 10 if blind, without light, confused or
  hallucinating. `chance = max(2, skill - depth/5)`. Roll once: success (gain `1 + depth/5`
  exp). Otherwise roll again: below chance = harmless failure; above = you set the trap off.
  Walking into a known trap tries to disarm it (use `W` to walk onto it deliberately).
- **Locked doors** (`calc_unlocking_chance()`): `max(2, physical disarm - 4 x lock power)`;
  skill /10 when blind or without light, and /10 again when confused or hallucinating.
- **Searching / noticing traps**: 4.2 has no search command and no searching *roll*. A trap
  becomes visible as soon as its grid is in view and `skill_search >= trap power`
  (`src/trap.c:square_reveal_trap()`, called from `src/cave-view.c`). Trap power is rolled at
  creation from the trap's `visibility:` line (`lib/gamedata/trap.txt`): 0 for the summoning
  runes and the acid trap (seen by anyone), 20-25 for gas, fire and most mechanical traps,
  30-60 for darts, teleport runes and mind-blast traps, 70/80/90 for poison, spiked and plain
  pits, and `2d10M50` (grows with depth) for trap doors. So the Search skill is a deterministic
  threshold: below it you walk into the trap blind. Detection magic reveals all traps regardless
  of skill.
- **Secret doors**: found automatically, 100%, whenever you stand next to one and are not blind,
  confused, hallucinating or in darkness (`src/player-util.c:search()`, called after every move
  and when holding still). No skill involved.
- **Chest traps** are revealed the same way when adjacent.

## 1.24 Status effects that change combat numbers

Summary of `calc_bonuses()` and `spell_chance()` (FACT). Full descriptions, protections and cures
are in [Chapter 5](05-elements-and-status.md#55-timed-effects).

| Effect | To-hit | To-dam | AC | Speed | Other |
|---|---|---|---|---|---|
| Stun (1-50) | -5 | -5 | | | device -10%, spell fail +15, no regen |
| Heavy Stun (51-150) | -20 | -20 | | | device -20%, spell fail +25, no regen |
| Knocked Out (151+) | | | | | you lose all turns |
| Afraid / Terror | -20 | | +8 | Terror +10 | cannot melee, spell fail +20, device -5% |
| Cut | | | | | damage 1-3 per tick, no regen |
| Poisoned | | | | | 1 damage per tick, device -5%, no regen |
| Bless | +10 | | +5 | | device +5% |
| Hero | +12 | | | | immune to fear, device +5% |
| Berserk | +75 melee skill | | -10 | | immune to fear, device -10% |
| Shield | | | +50 | | |
| Stoneskin | | | +40 | -5 | |
| Hungry/Weak | -0..-20 | -0..-20 | | | see 1.16 |
| Bloodlust | | +bloodlust/2 | | | +bloodlust/20 blows |
| Slow | | | | -10 | |
| Haste / Sprint | | | | +10 | |

## 1.25 Quick reference for the borg

Machine-usable summary (all FACT unless marked):

1. `energy/turn = extract_energy[speed]`; normal 10, +10 = 20, +20 = 30, +30 = 38.
2. `P(hit) = 0.12 + 0.83 x max(0, c - 2AC/3)/c`, `c = max(9, skill + 3 x to_h)`; cap 0.95.
3. Monster melee to-hit `3 x level + effect power`; only HURT/SHATTER (and the physical half
   of elemental blows) are reduced by AC: `dam x (1 - min(AC,240)/400)`.
4. Blows: `min(10000 / blows_table[min(11, str_blow x mult / max(w, minw))][min(11, dex_blow)],
   100 x max) / 100`; heavy weapon (`hold < weight lb`) -> 1 blow.
5. Wake chance per monster turn `~ 2^(-stealth/3)`; sleep -= `100/noise` (noise 1..49).
6. Monster hears you iff `hearing - stealth/3 > path distance`.
7. Regen per 10 game turns: `mhp x 197/65536` (x food bonus, x2 resting, x2 REGEN); 0 if
   poisoned, cut, stunned or paralysed.
8. Food: eat below 1500 (Hungry); never above 9000 before combat.
9. Spell fail: `base - 3 x (clvl - slvl) - adj_mag_stat`, clamp [minfail, 50], then stun
   +15/+25, amnesia `50 + f/2`, max 95.
10. Device fail per mille: `380 - 370x/(5+|x|)`, `x = 2 x (skill - level) + 1`. Avoid using a
    device with `skill < level + 5` in an emergency.
11. Burden penalty starts at 60% of `adj_str_wgt x 10 lb`.
12. Level-up restores all drained stats (ADVICE: a drained character close to a level should
    just keep killing).

---

## Things to try (untested)

These are ideas, not established facts. Each needs testing in play or in a debug build.

1. **Level-drain stat restore trick.** Since regaining a drained level restores all stats, a
   character with several drained stats and a Potion of Restore Life Levels might deliberately
   accept a small exp drain (e.g. from a nether worm) just below a level threshold, then drink
   the potion to restore everything. Test: drain STR with a known source, record stats, get
   level-drained, quaff Restore Life Levels, compare. Expected per code: all stats restored.
2. **Fast-rest economics.** Regeneration is per game time, not per action, so speed does not
   speed up healing. A fast character should prefer potions over resting when a monster is
   approaching, because each potion costs fewer game turns. Test by timing (game turns)
   rest-to-full at +0 and +10.
3. **Noise wall.** Rubble blocks noise flow but doors do not. Tunnelling rubble into a corridor
   behind you (or using a Stone to Mud on purpose elsewhere) might cut monsters off from hearing
   you. Test: place rubble between a hearing-20 sleeping monster and the player, watch whether
   it stays asleep longer.
4. **Stealth vs distance sweet spot.** Waking rate = P(notice) x 100/noise. For archers, shooting
   from 10+ grids at sleeping targets gives them only 10 sleep loss per notice. Measure the number
   of free shots per kill at stealth 5 vs 12.
5. **Weight-limited blows.** For warriors with STR 18/30+ any weapon under 3 lb gives the same P;
   test whether a heavy-dice light weapon (e.g. a 3 lb weapon with good dice and slays) beats
   a 15 lb one once DEX reaches 18/150 (table says 5.88 vs 4.76 blows).
6. **Critical-weight threshold.** Melee crit power crosses 900 (x3+20) only for weapons over
   25 lb; test whether a heavy weapon with fewer blows actually out-damages a lighter one for a
   high-skill warrior against high-HP uniques (crits scale with weapon to-dam).
7. **Device skill buff stacking.** Bless and Hero each add +5% device skill (multiplicative on
   current skill). Before a high-level rod/staff use in an emergency, quaffing Heroism might
   pull the fail rate off the steep part of the sigmoid. Test with a rod 3-5 levels above
   skill.
8. **Gorged as a weapon?** Food above 9000 costs speed, but `TMD_ATT_VAMP` exempts the speed
   loss. Investigate whether vampiric-strike users can deliberately stay gorged for faster regen
   (food bonus +33%) without the speed penalty.
9. **Infra-only exploration.** For characters with infravision 5 (dwarf, kobold), test whether
   exploring with *no* light (no fuel) gives better stealth outcomes; in code light does not
   affect stealth, so the expected answer is "no gain" - worth confirming because monsters'
   sight checks are `square_isview` from the player's side.
10. **Secret-door sweep.** Secret doors are found automatically when adjacent; a borg can find
    them by walking corridors' dead ends rather than "searching". Measure how often dead ends
    hide doors per level.

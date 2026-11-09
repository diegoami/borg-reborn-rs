# 4. Combat

How melee, shooting and throwing work in Angband 4.2 (upstream master), with the formulas
from the source, worked weapon comparisons, how monsters hit you back, and the tactics
that win fights: corridors, doors, speed, line of sight, fear, invisible monsters, resting.
It ends with a damage-race procedure a borg can compute before committing to a fight.

**Contents**

- [4.1 What the official manual says](#41-what-the-official-manual-says)
- [4.2 Melee: the sequence of one attack command](#42-melee-the-sequence-of-one-attack-command)
- [4.3 To-hit](#43-to-hit)
- [4.4 Blows and energy per blow](#44-blows-and-energy-per-blow)
- [4.5 Damage, criticals and the order of bonuses](#45-damage-criticals-and-the-order-of-bonuses)
- [4.6 Slays and brands](#46-slays-and-brands)
- [4.7 Special melee: shield bashes, unarmed, shapes, temporary effects](#47-special-melee-shield-bashes-unarmed-shapes-temporary-effects)
- [4.8 No sneak attacks: sleeping, unaware and debuffed monsters](#48-no-sneak-attacks-sleeping-unaware-and-debuffed-monsters)
- [4.9 Choosing a melee weapon](#49-choosing-a-melee-weapon)
- [4.10 Missiles: launchers, ammo, shots, might](#410-missiles-launchers-ammo-shots-might)
- [4.11 Throwing](#411-throwing)
- [4.12 Stealing (Rogues)](#412-stealing-rogues)
- [4.13 Monster melee against you](#413-monster-melee-against-you)
- [4.14 Tactics](#414-tactics)
- [4.15 Is this fight winnable? A damage-race procedure](#415-is-this-fight-winnable-a-damage-race-procedure)
- [4.16 O-combat (birth_percent_damage)](#416-o-combat-birth_percent_damage)
- [Things to try (untested)](#things-to-try-untested)

Related chapters: energy, speed, AC and the stat tables are in
[Chapter 1](01-core-mechanics.md); elements and status effects in
[Chapter 5](05-elements-and-status.md); weapon egos and artifacts in
[Chapter 6](06-equipment.md); monster AI in [Chapter 8](08-monsters.md); escapes in
[Chapter 13](13-escapes-and-emergencies.md).

---

## 4.1 What the official manual says

The [Attacking and Being Attacked](https://angband.readthedocs.io/en/latest/attack.html)
page ([local](user-manual/attack.rst)) says: you attack by moving into a monster; hits are
decided by your skill against the monster's armour class; damage is the weapon dice plus
the to-damage bonus; heavy weapons give fewer blows but more criticals; fractional blows
carry over as unused energy; only to-hit (not to-dam) bonuses on other equipment help
missiles; inspect a weapon (`I`) to see its damage. All of that is correct for this version.
This chapter adds the exact numbers, the order in which bonuses are applied, and what to
do with them.

> **Read the game's own calculator first.** Inspecting a weapon, launcher or missile
> (`I`) prints blows per round, "Average damage/round" against each slay or brand
> (including expected criticals), launcher range, and breakage chance
> (`src/obj-info.c`). A player or borg choosing between two weapons should compare
> those lines. The formulas below explain them.

---

## 4.2 Melee: the sequence of one attack command

FACT (`src/player-attack.c:py_attack()`):

1. The player is disturbed. Blackguards (`COMBAT_REGEN`) gain mana here (5% of max SP,
   minimum half a point) every time they attack.
2. If you have the `SHIELD_BASH` ability (Warrior, Paladin, Blackguard: `class.txt`
   `player-flags`) and can see the monster, you may get a free shield bash first
   ([4.7](#47-special-melee-shield-bashes-unarmed-shapes-temporary-effects)).
3. Available energy is `min(your energy, 100)`. Each blow costs
   `100 × 100 / num_blows` energy (`num_blows` is stored ×100). Blows are struck one at a
   time until the next would cost more energy than remains, or the monster dies. You
   never spend more than one normal turn on one attack command, so a slow monster
   never gets a double move from your attack.
4. Unused energy stays with you: with 2.5 blows you strike 2 blows for 80 energy and act
   again sooner. Fractional blows are real.
5. Each blow is `py_attack_real()`:
   - If you are **afraid** (`OF_AFRAID`, from the AFRAID or TERROR timed effects or a
     cursed item) you cannot melee at all: "You are too afraid to attack".
   - The monster wakes, and any **hold** on it is cleared (`mon_clear_timed(...HOLD)`).
     Hitting a held (paralysed) monster in melee releases it.
   - Roll to hit ([4.3](#43-to-hit)). A miss ends this blow.
   - Find the best slay or brand ([4.6](#46-slays-and-brands)).
   - Roll damage and criticals ([4.5](#45-damage-criticals-and-the-order-of-bonuses)).
   - Apply damage, check fear and death (`src/mon-util.c:mon_take_hit()`).
   - Earthquake if a weapon with `IMPACT` dealt more than 50 damage.

---

## 4.3 To-hit

### Melee skill

FACT (`src/player-attack.c:chance_of_melee_hit_base()`):

```
chance = skill_melee + 3 × (to_h + weapon_to_h + (blessed-wield ? 2 : 0))
chance = chance / 2          if the monster is not visible
```

- `skill_melee` = class base + race modifier + class increment × CL / 10
  (`src/player-calcs.c`, end of `calc_bonuses()`; values in `class.txt`/`p_race.txt`).
- `to_h` is the total of every **non-weapon, non-launcher** item's to-hit, plus the DEX
  and STR to-hit tables, plus timed effects. The weapon's own to-hit is added separately
  (the code keeps them apart so it can show both).
- Blessed-wield: Priests and Paladins (`BLESS_WEAPON`) wielding a hafted weapon or a
  `BLESSED` weapon get +2 to-hit here and +2 to-damage (`calc_bonuses()`).

| Class | Melee base / per 10 CL | Shoot base / per 10 CL | Throw base / per 10 CL | Max blows | Min weight (lb) | STR multiplier |
|---|---|---|---|---|---|---|
| Warrior | 70 / 45 | 55 / 45 | 55 / 45 | 6 | 3.0 | 5 |
| Mage | 35 / 15 | 20 / 15 | 20 / 15 | 4 | 4.0 | 2 |
| Druid | 45 / 20 | 40 / 30 | 40 / 30 | 4 | 3.5 | 3 |
| Priest | 45 / 20 | 35 / 20 | 35 / 20 | 4 | 3.5 | 3 |
| Necromancer | 35 / 25 | 20 / 15 | 20 / 15 | 4 | 3.5 | 3 |
| Paladin | 65 / 40 | 50 / 30 | 50 / 30 | 5 | 3.0 | 5 |
| Rogue | 35 / 45 | 66 / 30 | 72 / 45 | 5 | 2.0 | 4 |
| Ranger | 60 / 40 | 72 / 45 | 66 / 30 | 5 | 3.5 | 4 |
| Blackguard | 65 / 40 | 35 / 15 | 40 / 30 | 5 | 10.0 | 5 |

Source: `lib/gamedata/class.txt` (`skill-melee`, `skill-shoot`, `skill-throw`,
`max-attacks`, `min-weight`, `strength-multiplier`). Race melee modifiers range from
Hobbit −10 to Half-Troll +20 (`p_race.txt`); see [A8](appendices/A8-races-and-classes.md).

### The hit roll

FACT (`src/player-attack.c:hit_chance()`, used by players and monsters alike):

```
to_hit  = max(9, chance)
P(hit)  = 12% + 83% × max(0, to_hit − ⌊2·AC/3⌋) / to_hit
```

So there is always a 12% hit chance and a 5% miss chance. The rest scales with how far
your chance exceeds two thirds of the monster's AC.

| Your chance | AC 20 | AC 60 | AC 100 | AC 150 | AC 200 |
|---|---|---|---|---|---|
| 60 | 77% | 40% | 12% | 12% | 12% |
| 100 | 84% | 62% | 40% | 12% | 12% |
| 150 | 88% | 73% | 58% | 40% | 21% |
| 200 | 90% | 78% | 68% | 54% | 40% |
| 300 | 91% | 84% | 77% | 67% | 58% |

(computed from the formula; the game shows your exact percentage against any monster
whose armour you know in its recall: "You have a N% chance to hit such a creature in
melee", `src/mon-lore.c:lore_append_toughness()`.)

### What changes your to-hit

FACT (`src/player-calcs.c:calc_bonuses()`):

| Source | Effect |
|---|---|
| Bless (timed) | +10 to-hit, +5 AC |
| Heroism | +12 to-hit, immune to fear |
| Berserk strength (SHERO) | +75 **skill** (not ×3), −10 AC |
| Afraid | −20 to-hit, and **no melee at all** |
| Stun / Heavy stun | −5 / −20 to-hit and to-dam |
| Hunger (Weak and worse) | to-hit and to-dam fall progressively |
| Weapon too heavy (`adj_str_hold[STR]` < weight in lb) | −2 to-hit per lb over, and see [4.4](#44-blows-and-energy-per-blow) |
| Monster not visible | chance halved |

**ADVICE.** Heroism (+36 effective chance) and Berserk (+75) are the cheapest large
to-hit boosts in the game; drink one before any hard melee fight. Berserk also cures
fear, which otherwise forbids melee entirely.

---

## 4.4 Blows and energy per blow

FACT (`src/player-calcs.c:calc_blows()` and `blows_table[12][12]`):

```
weight_used = max(weapon weight, class min-weight)          (tenths of a pound)
P = min(11, adj_str_blow[STR] × class STR-multiplier / weight_used)
D = min(11, adj_dex_blow[DEX])
energy per blow = blows_table[P][D]
blows = min(10000 / energy, 100 × class max-attacks) + 100 × extra_blows   (×100)
```

- `adj_str_blow` is 3..17 for STR 3..17, then 20, 30, 40 … 240 for 18/00 … 18/220.
- `adj_dex_blow` is 0 (DEX ≤ 9), 1 (10–16), 2 (17–18/19), 3 (18/20–39), 4 (18/40–59),
  5 (18/60–79), 6 (18/80–99), 7 (18/100–119), 8 (18/120–149), 9 (18/150–179),
  10 (18/180–199), 11 (18/200+).
- The table entry is energy per blow (out of 100). Row P=11 runs 33, 29, 26, 24, 22, 21,
  20, 19, 18, 17, 16, 15 across DEX index 0..11, i.e. 3.0 to 6.6 blows before the class
  cap. Row P=0 runs 100 … 23.
- **Heavy weapons:** if `adj_str_hold[STR]` (lb) is below the weapon's weight in lb, you
  are "too weak", get −2 to-hit per pound over, and `calc_blows()` is **not called**:
  you are left at exactly **1 blow, and extra blows are ignored**. Hold values: 18 at
  STR 12, 28 at 17, 30 at 18/00–18/19, 50 at 18/50, 80 at 18/100–18/149, 100 at 18/200+.
- Extra blows from items (`BLOWS` modifier: the "of Extra Attacks" and "of Fury" egos and
  some artifacts, `ego_item.txt`/`artifact.txt`) are whole blows added after the cap.
- Unarmed (no weapon): weight is 0 so the class minimum weight is used.

### Reading the table

**FACT, computed.** A Warrior with STR 18/50 (adj 70) and DEX 18/50 (index 4):

| Weapon | Weight (lb) | P = 70×5 / weight | Energy/blow | Blows |
|---|---|---|---|---|
| Dagger, Main Gauche (≤ 3 lb → min 3.0) | 1.2 / 3.0 | 11 | 22 | 4.54 |
| Rapier | 4.0 | 8 | 26 | 3.84 |
| Tulwar / Cutlass | 10 / 11 | 3 | 42 | 2.38 |
| Katana, Mace | 12 | 2 | 50 | 2.00 |
| Blade of Chaos, Maul | 18 / 20 | 1 | 60 | 1.66 |
| Lance, Executioner's Sword | 30 / 26 | 1 | 60 | 1.66 |

The same Warrior at STR 18/100, DEX 18/100 (P index uses 120, D index 7): Dagger 5.26,
Rapier 5.26, Katana 4.16, Blade of Chaos 3.57, Lance 3.33.

**ADVICE.**

- Below about 18/50 STR, every extra pound of weapon costs blows quickly. Early on, a
  light weapon (Dagger, Main Gauche, Rapier) usually beats a heavier one of the same
  quality.
- DEX moves you along a row; STR moves you down the column (and is divided by weight).
  For light weapons the P index is already capped at 11, so **only DEX adds blows**.
  For heavy weapons STR matters most.
- Check the "blows" line when inspecting (`I`) a weapon: it is exact for your current
  stats. The inspect screen also shows how many blows you would get with more STR/DEX
  (`src/obj-info.c:obj_known_blows()`).

---

## 4.5 Damage, criticals and the order of bonuses

### Standard damage (the default; O-combat is in [4.16](#416-o-combat-birth_percent_damage))

FACT (`src/player-attack.c:melee_damage()`, `critical_melee()`, `py_attack_real()`):

```
d    = roll(weapon dice)                       (unarmed: 1)
d    = d × slay_multiplier   or   d × brand_multiplier   (best one only)
d    = d + weapon_to_d
d    = critical(d)        (weapons only; never unarmed)
d    = d + player_to_d    (state.to_d: rings, gloves, amulet, STR table, blessed +2, ...)
if d ≤ 0: "You fail to harm"
```

Consequences:

- The slay/brand multiplies **only the dice**. The weapon's to-dam is not multiplied by
  slays, but **is** multiplied by criticals.
- Your off-weapon to-dam (rings of Damage, gloves, STR) is added once per blow and is
  multiplied by nothing. With many blows it dominates: +10 to-dam on a 5-blow character
  is +50 per turn whatever the weapon.
- Splash (`weight × damage / 100`) is computed but only used for the IMPACT earthquake
  test, which fires when damage after criticals exceeds 50.

### Critical hits in melee

FACT (`src/player-attack.c:critical_melee()`, `lib/gamedata/constants.txt`
`melee-critical`):

```
chance  = weight + 5 × (to_h + weapon_to_h [+10 if monster debuffed]) + skill_melee − 60
P(crit) = chance / 5000
power   = weight + 1d650
```

| Power | Message | Damage becomes |
|---|---|---|
| < 400 | "It was a good hit!" | 2 × d + 5 |
| 400–699 | "It was a great hit!" | 2 × d + 10 |
| 700–899 | "It was a superb hit!" | 3 × d + 15 |
| 900–1299 | "It was a \*GREAT\* hit!" | 3 × d + 20 |
| ≥ 1300 | "It was a \*SUPERB\* hit!" | 4 × d + 20 |

(`d` is the dice-plus-weapon-to-dam value at that point.) Weight is in tenths of a pound,
so a 30 lb Lance adds 300 to both the chance and the power, while a Dagger adds 12. Only a
weapon over 65 lb could ever reach the ×4 tier, so in practice it is unreachable.

FACT, computed examples: Warrior CL20 with total to-hit +5: Dagger 2.7% crit chance
(always "good" or "great"); Lance 8.5% (mostly "great" or "superb"). Warrior CL35 with
to-hit +21: Dagger 5.7%, Lance 11.4%.

**ADVICE.** Criticals are a modest bonus (typically +5–15% expected damage for heavy
weapons, +2–5% for light ones). Do not choose a weapon for its criticals; choose it for
blows × dice and slays, then let criticals break ties.

---

## 4.6 Slays and brands

### The multipliers

FACT (`lib/gamedata/slay.txt`, `lib/gamedata/brand.txt`):

| Slay | Monsters (race flag) | Multiplier | O-combat |
|---|---|---|---|
| EVIL_2 | `EVIL` | ×2 | 1.8 |
| ANIMAL_2 | `ANIMAL` | ×2 | 2.0 |
| ORC_3, TROLL_3, GIANT_3 | `ORC`, `TROLL`, `GIANT` | ×3 | 2.5 |
| DEMON_3, DRAGON_3, UNDEAD_3 | `DEMON`, `DRAGON`, `UNDEAD` | ×3 | 2.5 |
| DEMON_5, DRAGON_5, UNDEAD_5 ("\*slay\*"/kill) | same | ×5 | 3.5 |

| Brand | Ignored by | Doubled against | Multiplier (weak / strong) |
|---|---|---|---|
| Acid | `IM_ACID` | — | ×2 / ×3 |
| Lightning | `IM_ELEC` | — | ×2 / ×3 |
| Fire | `IM_FIRE` | `HURT_FIRE` | ×2 / ×3 (×4 / ×6 vs HURT_FIRE) |
| Cold | `IM_COLD` | `HURT_COLD` | ×2 / ×3 (×4 / ×6 vs HURT_COLD) |
| Poison | `IM_POIS` | — | ×2 / ×3 |

The vulnerability doubling is in `src/obj-slays.c:get_monster_brand_multiplier()`.
`HURT_FIRE` covers molds, mushrooms, trees, zombies, vampires and many others;
`HURT_COLD` covers reptiles, snakes, nagas, hydras and fire creatures
([Chapter 8](08-monsters.md) lists them by family).

### How the one multiplier is chosen

FACT (`src/obj-slays.c:improve_attack_modifier()`, called from `py_attack_real()`):

1. Every equipped item **except the weapon and launcher** is checked first (body slots
   from index 2 on), then the weapon, then your **temporary** brands/slays (timed
   effects such as `ATT_FIRE`, `ATT_POIS` from the Blackguard spell *Venom*, `ATT_EVIL`
   from *Smite Evil*, `ATT_DEMON` from *Demon Bane*; `player_timed.txt`).
2. A brand counts only if the monster lacks its immunity flag; a slay counts only if the
   monster has its race flag.
3. The **single highest multiplier wins**. Brands and slays do not stack. Ties keep the
   first found.

So a ×3 fire brand and ×2 slay evil on the same weapon give ×3 against an evil,
non-fire-immune monster, ×2 against an evil fire-immune one, and ×3 against a non-evil
non-immune one.

Missiles and throws use the same choice, but only over the **missile and launcher** (no
other equipment, no temporary brands), and the result is **added** to the launcher
multiplier ([4.10](#410-missiles-launchers-ammo-shots-might)).

### Learning

Using a slay or brand against a monster that reacts teaches you the rune
(`learn_brand_slay_from_melee()`), so unknown weapons identify themselves in use.

**ADVICE.**

- Slays multiply only dice: on a light many-blow weapon with small dice they add little
  (Dagger ×3 vs orc: 2.5 → 7.5 per blow); on a big-dice weapon they are huge (Blade of
  Chaos 17.5 → 52.5).
- The best universal multipliers are the strong brands (×3 against everything not
  immune). Slay evil (×2) covers most dangerous monsters but is the weakest multiplier.
- Kill (×5) dragon/demon/undead matters a lot in the late game: Sauron is not undead or
  demon (he is `ainu` base, `EVIL`), Morgoth is `EVIL` only, so for the final fights
  slay evil and non-immune brands are what count. Morgoth is immune to all five base
  elements (`monster.txt`), so **brands do nothing to him**: only slay evil applies.
  Sauron is immune to cold, fire, poison and lightning (`IM_ELEC` plus the `ainu` base
  flags), so of the brands only acid works on him.

---

## 4.7 Special melee: shield bashes, unarmed, shapes, temporary effects

### Shield bash

FACT (`src/player-attack.c:attempt_shield_bash()`), for classes with `SHIELD_BASH`
(Warrior, Paladin, Blackguard), before your normal blows, against a visible monster:

- Needs a shield, and the monster's level must be at least CL/2.
- Chance: `skill_melee/8 + adj_dex_th[DEX]/2`, ×4 if unarmed, ×2 if
  `weapon dice × blows < shield dice × 3`; succeeds if that beats `1d(200 + monster
  level)` (rolled as `randint0`).
- Damage: `shield dice × (quality/40 + CL/14) + adj_str_td[STR]`, max 125, where
  quality = `skill_melee/4 + body weight/8 + total carried weight/80 + shield weight/2`.
- May stun (`quality + CL > 1d(200 + 8 × level)`) or confuse
  (`quality + CL > 1d(300 + 12 × level)`) the monster for 4 + 0..CL/5 turns.
- Afterwards you may stumble and lose 26–75% of a turn if
  `35 + adj_dex_th[DEX] < 1d60`.

**ADVICE.** Bashes are a small free bonus with a real cost (stumbling). They are most
useful to classes with weak weapons; a heavy shield (Knight's Shield 1d6, 16 lb) makes
them hit hardest. Stunned monsters hit 25% less often, deal 25% less damage, and lose a
turn now and then (`src/mon-attack.c`, `src/mon-move.c:process_monster_timed()`), so a
bash that stuns is worth more than its damage.

### Unarmed

Damage per blow is 1 + your to-dam, no criticals, blows computed with the class minimum
weight. Only worth anything with huge to-dam (shapechanges, Bloodlust).

### Shapechanges

Druid/Necromancer/Blackguard shapes (`lib/gamedata/shape.txt`) change your blow verbs,
and add the shape's skills, to-hit and to-dam (`calc_shapechange()`); the damage
calculation is otherwise the same and still uses your wielded weapon. See
[Chapter 3](03-magic-and-spells.md).

### Temporary melee effects

| Effect | Source (examples) | What it does in melee |
|---|---|---|
| `ATT_CONF` | Scroll of Monster Confusion; an activation (`object.txt`, `activation.txt`) | Next hit confuses the monster for 10 + 0..CL/10 turns, then ends |
| `ATT_VAMP` | Vampire shape (`shape.txt`) | Heals you by the damage dealt to living monsters |
| `ATT_FIRE` … `ATT_POIS`, `ATT_EVIL`, `ATT_DEMON` | Spells (*Venom*, *Smite Evil*, *Demon Bane*) | Temporary brand/slay, chosen as in [4.6](#46-slays-and-brands) |
| `BLOODLUST` | Blackguard *Bloodlust* | +to-dam = timer/2, +1 blow per 20 timer; misses and hits have a 1-in-50 chance of side effects |
| `ATT_RUN` | Rogue *Hit and Run* | After a steal attempt, teleports you 20 grids |

(`src/player-attack.c:blow_side_effects()`, `src/player-calcs.c`,
`lib/gamedata/class.txt`.)

---

## 4.8 No sneak attacks: sleeping, unaware and debuffed monsters

FACT: in this version there is **no bonus to hit or to damage for attacking a sleeping
or unaware monster**. `test_hit()` uses only your chance and the monster's AC, and the
critical formula only knows about a *debuffed* monster: if the monster is **confused,
held, afraid or stunned**, +10 is added to your to-hit for the critical-chance roll (that
is +50 on a 5000 scale, +1 percentage point), in melee, shooting and throwing
(`src/player-attack.c:is_debuffed()`, `constants.txt` `debuff-toh:10`).

What attacking a sleeper does give you:

- The first attack command is free: the monster does nothing until you hit it, and any
  damage wakes it (`mon_take_hit()` calls `monster_wake()`).
- At range, every shot you get before it reaches you is free.
- Rogues stealing from a sleeping monster face half the guard value
  ([4.12](#412-stealing-rogues)).

And a trap: any damage also clears **hold** (paralysis from *Hold Monster* style effects or
from fear). Do not "finish off" a held monster with weak blows expecting it to stay
helpless.

---

## 4.9 Choosing a melee weapon

### The formula

Expected damage per player turn against one monster:

```
dpt = blows × P(hit) × [ (1 − Pc) × base + Pc × E[crit(base)] + player_to_d ]
base = dice_average × multiplier + weapon_to_d
```

where `multiplier` is the best applicable slay/brand (1 if none) and `Pc` the critical
chance. For a quick estimate ignore criticals and use
`blows × P(hit) × (dice_avg × mult + weapon_to_d + player_to_d)`.

### Worked examples (FACT, computed with the formulas above)

**Case A.** Warrior CL20, STR 18/50, DEX 18/50, no to-hit/to-dam items (player to-dam +3
from STR, to-hit +5 from stats), weapons at (+0,+0), monster AC 60 (hit chance 76%).

| Weapon | Dice | Wt (lb) | Blows | Dmg/blow | Damage/turn | vs orc (×3 slay) |
|---|---|---|---|---|---|---|
| Dagger | 1d4 | 1.2 | 4.54 | 5.8 | 19.9 | 37.6 |
| Main Gauche | 1d5 | 3.0 | 4.54 | 6.3 | 21.8 | 43.1 |
| Rapier | 1d6 | 4.0 | 3.84 | 6.9 | 20.0 | 41.1 |
| Tulwar | 2d4 | 10 | 2.38 | 8.6 | 15.6 | 34.5 |
| Cutlass | 1d8 | 11 | 2.38 | 8.1 | 14.7 | 31.8 |
| Mace | 2d4 | 12 | 2.00 | 8.7 | 13.2 | 29.2 |
| Katana | 3d5 | 12 | 2.00 | 12.9 | 19.6 | 48.5 |
| Maul | 4d4 | 20 | 1.66 | 14.4 | 18.2 | 45.5 |
| Executioner's Sword | 4d5 | 26 | 1.66 | 17.0 | 21.5 | 54.9 |
| Lance | 2d8 | 30 | 1.66 | 14.0 | 17.7 | 43.1 |
| Blade of Chaos | 6d5 | 18 | 1.66 | 22.9 | 28.9 | 77.6 |

**Case B.** Warrior CL35, STR 18/100, DEX 18/100, +10 to-hit and +10 to-dam from rings
and gloves (so player to-dam +16), monster AC 100.

| Weapon | Blows | Damage/turn | vs ×3 slay |
|---|---|---|---|
| Dagger | 5.26 | 76 | 97 |
| Main Gauche | 5.26 | 79 | 104 |
| Rapier | 5.26 | 81 | 111 |
| Tulwar | 4.34 | 73 | 108 |
| Katana | 4.16 | 84 | 146 |
| Maul | 3.57 | 76 | 137 |
| Executioner's Sword | 3.33 | 78 | 148 |
| Lance | 3.33 | 70 | 123 |
| Blade of Chaos | 3.57 | 100 | 208 |

What the numbers say:

- **Dagger vs Main Gauche:** same blows (both use the 3 lb class minimum), Main Gauche
  has bigger dice, so it always wins for Warriors, Paladins and Rangers. For a Rogue
  (minimum 2.0 lb) the Dagger's lower weight can buy an extra fraction of a blow.
- **Light vs heavy with no slay:** roughly equal at mid stats; once off-weapon to-dam
  grows (Case B), many-blow light weapons catch up because the to-dam is added per blow.
- **With a slay or brand:** big-dice weapons win clearly (Katana, Executioner's Sword,
  Blade of Chaos), because only dice are multiplied.
- **Lance** is never a good choice for its weight: 2d8 on 30 lb. Weight helps only the
  crit chance, which is small.
- Weak-STR classes (Mage, Necromancer) get 1 blow with almost anything; for them the
  biggest dice they can hold without the heavy penalty is best, but melee is not their
  plan anyway.

### Rules of thumb (ADVICE)

1. Compare the inspect screen's "Average damage/round" lines for the monsters you
   actually fight (inspect shows one line per slay/brand plus "vs. others").
2. An ego or artifact with a brand beats a plain weapon with +5 more to-dam once your
   dice are 2d5 or bigger.
3. Extra blows (`BLOWS`) are worth more than anything else on light weapons: each one
   adds a full `dice×mult + all to-dam`.
4. Keep the weapon under your hold limit (`adj_str_hold`); one pound over makes you
   "too weak" and costs all blows but one.
5. Carry a second weapon only if it has a brand or slay your main weapon lacks and you
   fight that monster type often (e.g. a fire brand for trolls if your main weapon has
   none, or a \*slay dragon\* for a dragon hunt). See swap rules in
   [Chapter 6](06-equipment.md).
6. Against monsters immune to your brand, the brand does nothing; check the monster
   recall for `IM_*` before relying on it.

---

## 4.10 Missiles: launchers, ammo, shots, might

### Launchers

FACT (`lib/gamedata/object.txt`, `src/player-calcs.c:calc_bonuses()`):

| Launcher | Multiplier (pval) | Ammo | Weight (lb) | Range |
|---|---|---|---|---|
| Sling | ×2 | shots, pebbles | 0.5 | 10 |
| Short Bow | ×2 | arrows | 3 | 10 |
| Long Bow | ×3 | arrows | 4 | 12 |
| Light Crossbow | ×3 | bolts | 11 | 12 |
| Heavy Crossbow | ×4 | bolts | 20 | 14 |

- Range is `min(6 + 2 × multiplier, 20)` grids (`do_cmd_fire()`); firing beyond range
  asks for confirmation and the missile stops at range.
- `MIGHT` adds to the multiplier, `SHOTS` adds tenths of a shot (`ego_item.txt`: "of
  Extra Might", "of Extra Shots" (+0.2..0.6 shots), "of Lothlórien", "of the Haradrim",
  "of Buckland"; see [A5](appendices/A5-ego-items.md) for exact values).
- A launcher heavier than your hold limit gives −2 to-hit per lb over and **cancels
  extra shots and might**.
- Shots are stored ×10. Base 10 (one shot). Rangers (`FAST_SHOT`) get `+CL/3` tenths:
  +1.0 shot at CL30, +1.6 at CL50, with **any** launcher. Energy per shot is
  `100 × 10 / num_shots`: 2.0 shots = 50 energy each.

### Ammo

| Ammo | Dice | Weight | Breakage on hit (miss = square/100) |
|---|---|---|---|
| Rounded pebble / Iron shot | 1d2 / 1d4 | 0.4 / 0.5 | 0% (shots never break) |
| Mithril shot | 2d4 | 0.2 | 0% |
| Arrow / Mithril / Seeker arrow | 1d4 / 3d4 / 4d4 | 0.2 / 0.1 / 0.2 | 35% (12% on a miss) |
| Bolt / Mithril / Seeker bolt | 1d5 / 3d5 / 4d5 | 0.3 / 0.2 / 0.3 | 20% (4% on a miss) |

(`object_base.txt` `break`; `src/player-attack.c:breakage_chance()`; artifacts never
break.)

### Shot damage and to-hit

FACT (`src/player-attack.c:ranged_damage()`, `chance_of_missile_hit()`,
`critical_shot()`):

```
mult    = launcher multiplier (+MIGHT) + best slay/brand multiplier of ammo or launcher
damage  = (roll(ammo dice) + ammo_to_d + launcher_to_d) × mult      then critical
chance  = skill_shoot + 3 × (to_h + ammo_to_h + launcher_to_h) − distance
          halved if the monster is not "obvious" (unseen or camouflaged)
crit    : chance = ammo weight + 4 × (to_h + ammo_to_h [+10 debuffed]) + 2 × CL,  /5000
          power  = ammo weight + 1d500:  <500 ×2+5,  500–999 ×2+10,  ≥1000 ×3+15
```

Key differences from melee:

- The slay/brand is **added** to the launcher multiplier: a ×3 Long Bow firing ×3
  flame arrows deals ×6, firing slay-evil (×2) arrows deals ×5.
- **Off-weapon to-dam (rings, STR) does not apply to missiles** in standard combat;
  off-weapon to-hit does. The ammo and launcher to-dam are multiplied.
- The distance penalty is only −1 chance per grid: small.
- A missile stops at the first monster in its path whether it hits or not.
- Your temporary brands do not apply to missiles.

### Worked example (FACT, computed)

Ranger CL30 (2.0 shots) with a Long Bow (+5,+5) and Arrows (+3,+3):
`(2.5 + 3 + 5) × 3 = 31.5` per hit, 63 per turn before hit chance. With flame arrows
(×3 brand): `10.5 × 6 = 63` per hit, 126 per turn. Compare a Warrior CL30 with the same
bow and arrows (1.0 shot): 31.5 per turn.

A Sling with plain iron shots (+0,+0): `2.5 × 2 = 5` per hit. Slings are mainly for
pebbles and for classes with nothing better early.

**ADVICE.**

- For Rangers the bow is the main weapon from CL1; for everyone else it is the opening
  shot, the way to kill things you must not touch (floating eyes, jellies with nasty
  touches, molds), and the way to wake and pull one monster from a group.
- Keep 40+ arrows/bolts for the main ammo slot, and a stack of branded or slaying ammo
  for hard targets (uniques, dragons).
- Pick up ammo after a fight (`breakage` means about two-thirds of arrows survive a hit).
- Shots do not break, so slings are cheap to use; bolts break least of the sharp ammo.

---

## 4.11 Throwing

FACT (`src/player-attack.c:do_cmd_throw()`, `make_ranged_throw()`, `ranged_damage()`):

- One throw costs a full turn (`shots = 10`), no class gets extra throws.
- Range: `min((adj_str_blow[STR] + 20) × 10 / max(weight, 10), 10)` grids. Heavy things
  go a short distance with low STR.
- To-hit: for items flagged **`THROWING`**: `skill_throw + 3 × (to_h + item to_h)`.
  For anything else: `1.5 × skill_throw + 3 × item to_h` (easier to hit, but no
  damage bonus).
- Damage for `THROWING` items: `(dice + item to_d) × (2 + ⌊weight/12⌋)` with weight in
  tenths of a pound (a 5 lb Spear, weight 50, gives ×6), then × (1 + best brand/slay of
  the item), then critical, then **×3 if the item `EXPLODE`s** (flasks of oil).
- Non-`THROWING` items do just dice + to-dam (× 1 + slay).
- `THROWING` weapons (not ammo, not exploding) break only 1% of the time on a hit.

`THROWING` items in `object.txt`: **Flask of oil** (also `EXPLODE`), **Dagger**, **Spear**,
**Throwing Axe**, **Throwing Hammer**, and slings' shots (Rounded Pebble, Iron Shot,
Mithril Shot). Throwing weapons can sit in the quiver; each takes the room of 5 missiles
(`constants.txt` `thrown-quiver-mult:5`).

| Item | Dice | Weight | Throw multiplier | Average per hit (+0) |
|---|---|---|---|---|
| Flask of oil | 1d4 | 2 lb | ×3, then ×3 explode | 22.5 |
| Dagger | 1d4 | 1.2 lb | ×3 | 7.5 |
| Spear | 1d6 | 5 lb | ×6 | 21 |
| Throwing Axe | 2d3 | 4 lb | ×5 | 20 |
| Throwing Hammer | 2d4 | 5.5 lb | ×6 | 30 |

The item's own to-dam is inside the multiplier: a Throwing Hammer (+5,+5) averages
`(5 + 5) × 6 = 60` per hit.

**ADVICE.**

- **Flasks of oil are the best early ranged attack for every class.** At 3 gold each
  (`object.txt` cost) and 22.5 average damage they kill most DL1–10 threats in one or two
  hits, including Grip, Fang and Bullroarer's escorts. The object description says the
  damage applies even to fire-resistant monsters (the ×3 is a flat multiplier, not a fire
  brand). Carry 10–20 early (they also refuel lanterns).
- Rogues have the best throwing skill (72 + 4.5/CL) and a good ego/artifact throwing
  weapon makes them strong ranged attackers. A stack of plain Spears or Throwing Axes is
  a solid mid-game Rogue weapon.
- Throwing weapons are flagged so that they take your to-hit from other equipment; they
  do not take your to-dam.

---

## 4.12 Stealing (Rogues)

FACT (`src/cmd-cave.c:do_cmd_steal()`, `src/mon-util.c:steal_monster_item()`): Rogues
(`STEAL` ability) have a *steal* command (`s` in the original keyset, see
[command list](https://angband.readthedocs.io/en/latest/command.html)). It costs one turn
and targets an adjacent monster.

```
guard        = level × (unique ? 4 : 3) / 4 + monster speed − your speed
guard        = guard / 2                          if the monster is asleep
steal_skill  = stealth skill + adj_dex_th[DEX]    (÷4 if blind, confused or hallucinating)
reaction     = guard/2 + 1d(max(guard,1)) + (item weight × number)/20
reaction < steal_skill          → success: you take one random item (or gold)
reaction/2 < steal_skill        → fail: you see the item, the monster wakes and may notice
otherwise                       → "cries out in anger": the monster wakes and shrieks (wakes the level)
```

- The monster must be carrying something (monsters with drops carry them from birth;
  "You can find nothing to steal" otherwise, and a 1-in-3 chance it wakes).
- On success the monster's sleep drops by `35 − stealth`.
- With *Hit and Run* (`ATT_RUN`) active you teleport 20 grids after the attempt,
  whatever the result.

**ADVICE.** Steal from sleeping, slow, low-level monsters with good drops (uniques'
`DROP_GOOD` items are carried). Speed matters a lot: each +1 speed over the monster
reduces guard by 1. Combine with *Hit and Run* so a bungled attempt costs nothing but a
turn. Do not steal from anything you cannot escape when it wakes and shrieks.

---

## 4.13 Monster melee against you

### Hit chance

FACT (`src/mon-attack.c:chance_of_monster_hit_base()`, `check_hit()`):

```
to_hit = 3 × monster level + effect power      (×0.75 if the monster is stunned)
P(hit) = same formula as yours, against your total AC (base AC + to_a)
```

Effect powers (`lib/gamedata/blow_effects.txt`): HURT 40, SHATTER 60, ELEC/FIRE/COLD 40,
POISON 20, ACID 20, CONFUSE 20, EXP_* 20, DISENCHANT 10, DRAIN_CHARGES 10; all
others (BLIND, PARALYZE, TERRIFY, stat drains, EAT_*, HALLU, BLACK_BREATH) 0. A blow
with effect `NONE` always hits.

| Monster level, HURT blow | Your AC 30 | AC 60 | AC 100 | AC 150 | AC 200 |
|---|---|---|---|---|---|
| 10 (to-hit 70) | 71% | 48% | 17% | 12% | 12% |
| 30 (to-hit 130) | 82% | 69% | 53% | 31% | 12% |
| 50 (to-hit 190) | 86% | 78% | 66% | 51% | 37% |
| 80 (to-hit 280) | 89% | 83% | 75% | 65% | 56% |

The monster recall shows the exact hit chance for each known blow against your
current AC, and the average damage per monster turn ("averaging N damage on each of its
turns", `src/mon-lore.c:lore_append_attack()`).

### Armour reduces physical damage

FACT (`src/mon-attack.c:adjust_dam_armor()`, `src/mon-blows.c`):

- `HURT` and `SHATTER` blows: damage × (1 − min(AC, 240)/400). At AC 100 you take 75%,
  at AC 200 50%, at AC 240+ 40%.
- Elemental blows (ACID/ELEC/FIRE/COLD): you take the **larger** of the
  armour-reduced physical part (computed with AC + 50, and zero for non-physical methods
  like TOUCH and ENGULF) and the resisted elemental part. Resistances matter more than
  AC for these. Elemental blows also damage inventory.
- Every other effect does its rolled damage regardless of AC (AC still matters for
  whether it lands).
- **Protection from evil**: an evil monster's blow is repelled if your CL ≥ monster level
  and `1d100 + CL > 50` (`make_attack_normal()`).

### Cuts and stuns from blows

Methods HIT, CLAW, BITE can cut; HIT, PUNCH, KICK, BUTT, CRUSH can stun
(`blow_methods.txt`). A cut/stun happens only on a "critical" monster blow: one that
rolls at least 95% of its maximum damage (`monster_critical()`); harder blows give bigger
cuts and stuns. A Heavy Stun (−20 to-hit/to-dam) and knock-out are real dangers from
big crushing monsters; protection from stun is in [Chapter 5](05-elements-and-status.md).

### Blow effects that matter (ADVICE, with FACT from `src/mon-blows.c`)

| Effect | Why it matters | Counter |
|---|---|---|
| PARALYZE | 3 + 1d(monster level) turns helpless; a saving throw applies; does not stack while paralysed | **Free Action** before DL20. Never melee floating eyes, carrion crawlers, ghouls without it |
| EXP_10..80 | Experience drain | Hold Life; avoid melee with W, V, L |
| LOSE_* | Stat drain | Sustains. There are no Restore potions: drained stats come back on each level gain (`src/player.c:adjust_level()`) and from a few foods/mushrooms ([Ch. 7](07-consumables-and-devices.md)) |
| DISENCHANT | Lowers to-hit/to-dam/to-a of your gear | rDisen, or kill at range |
| DRAIN_CHARGES | Empties wands/staves, heals the monster | Keep devices in the home or kill at range |
| EAT_GOLD / EAT_ITEM | Steals and the thief teleports away | Kill at range; DEX and CL give a save |
| BLIND / CONFUSE | Cannot read scrolls; confused moves and aims | rBlind, rConf (pConf) |
| TERRIFY | You cannot melee | pFear, Heroism, Berserk |
| SHATTER | Earthquake around the monster | Do not fight it near your stash |
| BLACK_BREATH | Nazgûl only | see [Chapter 5](05-elements-and-status.md) |

---

## 4.14 Tactics

Each item is ADVICE grounded in the cited mechanics.

### Fighting in corridors

- In a corridor at most two monsters (often one) can reach you; in a room up to eight.
- **But pack monsters know this.** Monsters with `GROUP_AI` (hounds `Z`, wolves and dogs
  `C`, many spiders, ants) that cannot pass walls check how many open grids surround you.
  If fewer than 5 and you have more than half your HP, they **retreat out of sight to
  ambush** instead of entering the corridor (`src/mon-move.c:get_move()`,
  `get_move_find_hiding()`). Standing in a corridor mouth leaves them waiting in the
  room.
- Breathers still breathe down a corridor if they can see you; a corridor lines them up
  for your bolt and beam spells and for missiles.
- Practical: stand one or two squares inside the corridor from a room entrance. Hounds
  that peek in get hit one at a time; if they hide, shoot or let them come back, or leave.

### Doors

- Closing a door breaks line of sight: casters and breathers cannot target you, archers
  cannot shoot (`src/mon-attack.c:monster_can_cast()` needs a projectable path).
- Monsters with neither `OPEN_DOOR` nor `BASH_DOOR` (insects, worms, jellies, many
  reptiles and other animals; check recall) cannot pass a closed door at all. Note that
  dogs and nearly all hounds **do** have `BASH_DOOR` (`monster.txt`).
- An unlocked closed door costs an opener or basher one turn (bashers may then fall into
  the doorway). **Locked** doors are far better: each try succeeds only if
  `randint0(monster HP/10)` exceeds the lock power, and each success lowers the power by
  one (`src/mon-move.c:monster_turn_can_move()`). You can lock a closed door yourself with
  the Disarm/lock command (`D`), with power `m_bonus(7, depth)` (0–7, higher when deeper;
  `src/cmd-cave.c:do_cmd_lock_door()`). A monster whose `HP/10` is at most `power + 1` can never
  get through: a power-2 lock stops 35 HP hounds and wolves for good.
- Fighting from a doorway: you in the corridor, the monster in the door square, means
  one attacker at a time and no breath angle from the rest of the room.

### Pillar dancing and hit-and-run with speed

- With a speed advantage you get `energy(you)/energy(monster)` actions per monster action
  (+10 vs +0 is 2:1; [Chapter 1](01-core-mechanics.md) has the energy table). Shoot,
  step back, shoot: a slower melee-only monster never reaches you.
- Pillar dancing: keep a pillar or wall corner between you and a melee monster of equal
  speed, attacking when it steps adjacent. It works against non-casters; casters just
  cast when they see you.
- Monsters with `RAND_25`/`RAND_50` waste 25/50% of their moves wandering
  (`monster_turn_should_stagger()`); they are easy to outpace.
- Monsters never get a double move from your melee (energy capped at one turn per
  attack), but **they do from your speed deficit**: at −10 relative speed, a monster
  gets two turns for each of yours.

### Luring and pulling

- Wake one monster with a missile from max range, then retreat into a corridor; it
  follows by sight, then by sound (your noise flow), then by scent
  (`get_move_advance()`). Group members are roused only slowly unless they see the
  awake one and are near it (`src/mon-group.c:monster_group_rouse()`: 1 in
  (distance × 20) per turn).
- Monsters track you by sound: `hearing − stealth/3` must exceed your path distance.
  Good stealth means fewer followers.

### Stairs as safe spots

- With the default `birth_connect_stairs` you arrive on a staircase
  (`list-options.h`). Standing on a stair, any fight you are losing can be ended in one
  turn by taking it (a new level is generated; nothing follows).
- ADVICE: fight dangerous single monsters, and uniques you want to test, from a stair
  square. A borg should treat "standing on stairs" as its most reliable escape (it cannot
  fail, unlike scrolls when blind/confused or devices when failing). See
  [Chapter 13](13-escapes-and-emergencies.md).

### Groups versus singletons

- Group monsters (packs of hounds, orc gangs) are dangerous by numbers: their total
  damage per turn is the sum. Fight them one at a time (corridor), or with area attacks
  (balls, breath-like spells), or skip them.
- A singleton of the same level is usually easier, unless it is a unique.
- Summoners make a singleton into a group: summons appear next to the **summoner**, and
  only if there is free floor within 2 grids of it (`summon_possible()`). A summoner
  in a corridor with you blocking one side has little room.

### Archers and casters: break line of sight

- Every monster spell, breath and arrow needs a projectable path from the monster to you
  (`monster_can_cast()`); bolt spells additionally need a path with no other monster in
  the way (`PROJECT_STOP`), so non-stupid monsters will not bolt through their friends.
- Step around a corner and wait: the caster has to come to you (and casters with high
  spell frequency like to keep 3 grids away: `get_move_find_range()`), where you can
  melee it. This beats trading spells with a priest or mage you cannot outdamage.
- A monster's spell chance **doubles when it is at its preferred range**
  (`monster_can_cast()`): archers prefer `min_range + 3`, frequent casters likewise; so
  closing to melee often reduces how often it casts.
- **Every non-stupid monster fails non-innate spells about 24% of the time** in this
  version (`src/mon-attack.c:monster_spell_failrate()` computes
  `power = MIN(spell_power, 1)`, so the failure is `25 − 1 = 24%`, plus 20% if afraid,
  50% if confused or disenchanted). Innate attacks (breaths, arrows, spits, boulders)
  never fail. See [Chapter 8](08-monsters.md).

### Monster fear and fleeing monsters

FACT (`src/mon-util.c:monster_scared_by_damage()`, `src/mon-move.c`):

- A monster becomes afraid when hit for at least its remaining HP (80%), or randomly
  when at ≤10% HP; never if `NO_FEAR`, rarely in big groups, never as a bodyguard.
- Afraid monsters run to a grid you cannot see (`get_move_find_safety()`), then hover
  out of sight. They **still cast spells and breathe** while fleeing (ranged attacks are
  tried before movement in `monster_turn()`), with +20% spell failure.
- An afraid monster that cannot move away is **frozen**: its fear is converted into
  hold (`monster_turn()` "paralyzed by fear"). Hitting it clears the hold.
- Fear wears off by `1d(level/10 + 1)` per monster turn, and damage can cure it
  (a hit for more than the remaining fear timer ends it).
- Monsters at low HP compared with you also keep their distance even when not afraid
  (`get_move_find_range()`: a strong player "scares strong monsters").

ADVICE: when a monster flees, either finish it with missiles/bolts immediately, or let it
go and expect it back healed. Do not chase a fleeing caster into the dark; it shoots
you on the way. Corner it if you can: a cornered afraid monster is helpless until hit.

### Invisible monsters

- Unseen monsters halve your melee and missile chance (`chance_of_melee_hit()`,
  `chance_of_missile_hit()`).
- Signs: "It hits you", "You feel something", attacks from an empty square, items
  being stolen.
- Answers: See Invisible, telepathy (except `EMPTY_MIND` and most `WEIRD_MIND`
  monsters), **infravision for warm-blooded invisible monsters** (infravision is checked
  before invisibility in `src/mon-util.c:update_mon()`, so a warm-blooded invisible
  monster within infravision range is seen; ghosts are cold-blooded),
  detect invisible, area attacks, or simply attacking the square it hit you from.
  Details per family in [Chapter 8](08-monsters.md).

### Monster HP recovery and when to rest

- Monsters regenerate `max(1, maxHP/100)` HP every 100 game turns (10 normal-speed
  player turns), doubled for `REGENERATE` monsters (trolls, vampires, some uniques)
  (`src/mon-move.c:regen_monster()`). A 1000 HP unique regains 10 HP per 10 turns, 20
  if it regenerates. Breaking off a fight to rest therefore costs little against
  big-HP monsters but lets small ones recover fully.
- Resting near an awake monster that knows where you are is pointless: it comes to you.
  Rest behind a closed door, after teleporting, or on a fresh level.
- ADVICE for a borg: rest when HP < 70% (or SP needed) and no monster is in view or
  known to be tracking; never rest with a breeder colony on the level.

---

## 4.15 Is this fight winnable? A damage-race procedure

ADVICE (a procedure, built on the facts above). It works for a single monster; for a group
sum the monsters' damage and use the toughest one's HP, or don't fight.

### Step 1: your damage per turn against it (Dp)

```
P_hit   = 0.12 + 0.83 × max(0, C − ⌊2·AC_m/3⌋) / max(9, C)        (C halved if unseen)
Dp_melee = blows × P_hit × (dice_avg × best_mult + weapon_to_d + player_to_d) × 1.05
Dp_shoot = shots × P_hit_shoot × (ammo_avg + ammo_to_d + bow_to_d) × (bow_mult + slay_mult)
Dp_spell = average spell damage after the monster's resistances / energy cost
Dp = max(Dp_melee, Dp_shoot, Dp_spell)
```

(the 1.05 is a rough allowance for criticals.)

### Step 2: its damage per monster turn against you (Dm)

```
Dm_melee = Σ over blows  P_hit_m(blow) × avg_dice × armour_factor     (if adjacent)
           armour_factor = 1 − min(AC,240)/400 for HURT/SHATTER, 1 otherwise (minus resists)
Dm_spell = Σ over its spells  (freq/100) × 0.76 × (1/n_spells) × avg_damage_after_resist
           (0.76 = 1 − 24% failure for non-innate spells; 1.0 for innate/breaths)
Dm = (1 − f) × Dm_melee + Dm_spell      where f = its total chance to use a ranged attack
```

Breath damage is `current HP / divisor` capped per element (fire/cold/acid/elec/mana /3
cap 1600, poison /3 cap 800, the rest mostly /6 cap 150–600; see
[A3](appendices/A3-monster-spells.md) and [Chapter 5](05-elements-and-status.md)). The
monster recall lists its known spells and averages; use them.

### Step 3: race it

```
T_kill  = monster_HP / Dp                         (your turns to kill it)
R       = energy_per_turn(monster speed) / energy_per_turn(your speed)
expected_damage_taken = T_kill × R × Dm
worst_case_burst      = max single action (biggest breath/spell/full blow round), × 2 if R ≥ 2
```

Fight if all hold:

1. `expected_damage_taken < 0.5 × current HP` (leave room for variance), and
2. `worst_case_burst < 0.5 × current HP`, and
3. you have a working escape (stairs under you, or a reliable teleport not blocked by
   blindness/confusion/amnesia), and
4. it has no effect you are unprotected against that ends fights by itself:
   paralysis (no FA), heavy stat/exp drain you care about, confusion+blindness with no
   cure, `TELE_LEVEL`/`TELE_TO` into worse situations.

Otherwise: avoid, lure into a better position, buff (Heroism, Berserk, Haste, resist
potions) and recompute, or leave.

### Example

Bolg, Son of Azog (DL20, +10 speed, 500 HP, AC 60, four HIT 3d6 blows, no spells;
`monster.txt`). Your Warrior from Case A: Dp ≈ 20 per turn with a Main Gauche →
`T_kill = 25` turns. Bolg's HURT blows at level 20 have to-hit 100: against your AC 40
P_hit ≈ 0.12 + 0.83 × (100 − 26)/100 = 0.73; damage `4 × 10.5 × 0.73 × 0.9 = 27.6` per
Bolg turn; R = 2 (he is +10, you +0). Expected damage `25 × 2 × 27.6 ≈ 1380`. Not
winnable at CL20 without speed: get Haste (R → 1, damage ≈ 690, still too much), or a
×3 orc-slaying weapon (Dp 43, T_kill 12, damage ≈ 330 hasted), and his escort of
uruks first. This matches experience: Bolg kills many characters at DL20.

**Borg note.** The procedure needs: your blows/shots and hit chances (computable from
state), the monster's recall (blows, AC, HP, speed, spells, frequency), and the breath
divisors. Use the recall's own "averaging N damage" and "N% chance to hit" figures when
the blows are known; when they are not, assume the worst blow set for the monster's
glyph and depth ([Chapter 8](08-monsters.md)).

---

## 4.16 O-combat (birth_percent_damage)

FACT: the birth option `birth_percent_damage` ("To-damage is a percentage of dice
(experimental)", default **off**, `src/list-options.h`) switches to the O-combat model
(`o_melee_damage()`, `o_ranged_damage()`, `o_critical_melee()`):

- To-damage (weapon + player, capped at 150) becomes a percentage increase of the dice
  via `deadliness_conversion[]` (e.g. +10 → +39%, +20 → +69%, +30 → +99%).
- Slays/brands multiply the die average with the O-multipliers (slay evil 1.8, ×3
  slays 2.5, ×5 slays 3.5, strong brands 2.5, weak 1.5) and add a small flat bonus.
- Criticals add whole extra dice; chance `power/(power + 240)` with
  `power = melee chance / 3`.
- Every character gets at least 2 blows.
- Throwing weapons and launchers use the same scheme.

This manual assumes the default (off). If you turn it on, the weapon-choice rules change:
dice size matters much more, and many small blows matter less.

---

## Things to try (untested)

1. **Flask-of-oil opening for every class.** Buy 15–20 flasks before the first dive and
   throw them at every unique and pack leader in DL1–12. *Why:* 22.5 average damage
   regardless of fire resistance kills Grip/Fang/Bullroarer in 1–3 hits. *Test:* run 20
   borg starts with and without flasks; compare deaths before CL10 and turns to CL15.
2. **Corridor-mouth hound farming.** Exploit the `GROUP_AI` ambush rule: stand in a
   corridor at >50% HP so hounds hide, and shoot any that peek. *Why:* they refuse to
   enter, so you take only breath from those that see you. *Test:* log damage taken per
   hound killed with the player in a corridor versus in a room.
3. **Deliberately drop below 50% HP to pull a pack in.** The hide rule only applies when
   `chp > mhp/2`. *Why:* below half HP the pack charges into your corridor one at a time.
   Risky. *Test:* in wizard mode, compare kill rate and deaths for the two thresholds.
4. **Cornering frightened casters.** Since an afraid monster with no escape gets
   "held", chase fleeing casters into dead ends and then use a big single hit. *Test:*
   count spells cast by fleeing monsters before death, cornered versus uncornered.
5. **Stun-lock with shield bashes.** A Warrior with a Knight's Shield and a light weapon
   gets doubled bash chance (puny-weapon rule) and stuns lower the monster's hit chance
   and damage by 25%. *Test:* measure damage taken per kill with a dagger + Knight's
   Shield versus a heavier weapon + small shield at equal damage output.
6. **Throwing-hammer Rogue.** Keep a quiver of Throwing Hammers (×6 multiplier) instead of
   a sling. *Why:* `(5 + to_d) × 6` per hit beats slings by a factor of 5+.
   *Test:* compare kills per 1000 game turns for Rogues CL15–30.
7. **Ranger branded-ammo budget.** Since slays add to the launcher multiplier, a ×3 bow
   with ×3 ammo doubles damage. Save all branded ammo for uniques. *Test:* track unique
   kill times with and without the reserve.
8. **Slow-monster steal loop.** Rogue with high stealth and *Hit and Run*: steal from
   sleeping `DROP_GOOD` uniques (guard halved when asleep) and teleport out.
   *Test:* success rate and items obtained per attempt by unique level.
9. **Spell-failure exploitation.** Because non-innate monster spells fail 24% of the time
   (see [4.14](#414-tactics)), pure-caster monsters are weaker than their frequency
   suggests. A borg danger model that ignores this overestimates casters. *Test:* compare
   predicted versus observed damage from caster monsters over many fights.
10. **Off-weapon to-dam stacking for many-blow characters.** Two rings of Damage on a
    5-blow character add +50+ per turn regardless of the weapon. *Test:* compare
    inspect-screen damage for "best weapon + no rings" versus "average weapon + 2 rings
    of Damage" across CL20–40.

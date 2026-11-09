# Core Mechanics: How the Game Really Works

> **Scope.** This chapter documents the *engine* of Angband 4.2.6 as it is
> actually implemented in the C source shipped in this repository. Every
> formula below was read from the code; file and function names are given so
> you can verify them yourself (e.g. `src/player-attack.c:hit_chance()`).
> Game data (monster lists, object kinds, class tables) lives in
> `lib/gamedata/*.txt`; tunable constants live in `lib/gamedata/constants.txt`.
>
> The official manual (`docs/*.rst`, in particular `docs/attack.rst`,
> `docs/playing.rst`, `docs/dungeon.rst` and `docs/birth.rst`) explains *what*
> the commands do. This chapter explains *how the numbers work*, so that you
> can make good decisions about speed, gear, stats and tactics.

---

## Table of contents

1. [Conventions used in the code](#1-conventions-used-in-the-code)
2. [Time, energy and speed](#2-time-energy-and-speed)
3. [Melee combat](#3-melee-combat)
4. [Missile and throwing combat](#4-missile-and-throwing-combat)
5. [Armour class and monster melee](#5-armour-class-and-monster-melee)
6. [Hit points, levels and experience](#6-hit-points-levels-and-experience)
7. [Stats](#7-stats)
8. [Skills and the saving throw](#8-skills-and-the-saving-throw)
9. [Stealth, noise, scent and monster waking](#9-stealth-noise-scent-and-monster-waking)
10. [Light, vision, invisibility, telepathy and detection](#10-light-vision-invisibility-telepathy-and-detection)
11. [Monster AI basics](#11-monster-ai-basics)
12. [Spellcasting: mana, spell count and failure](#12-spellcasting-mana-spell-count-and-failure)
13. [Magic devices: failure rates](#13-magic-devices-failure-rates)
14. [Regeneration, food and digging](#14-regeneration-food-and-digging)
15. [Resistances and damage reduction (formulas)](#15-resistances-and-damage-reduction-formulas)
16. [Traps, doors and locks](#16-traps-doors-and-locks)
17. [Practical takeaways](#17-practical-takeaways)

---

## 1. Conventions used in the code

A few internal conventions explain many "odd" numbers you will see:

| Convention | Meaning | Where |
|---|---|---|
| Weights are in **tenths of a pound** | A Dagger has `weight:12` = 1.2 lb; a Blade of Chaos `weight:180` = 18 lb | `lib/gamedata/object.txt`, `object_weight_one()` |
| `num_blows` is stored **×100** | 526 = 5.26 blows per round | `src/player-calcs.c:calc_blows()` |
| `num_shots` is stored **×10** | 20 = 2.0 shots per round; `SHOTS[x]` modifiers are in tenths | `src/player-calcs.c:calc_bonuses()` |
| Speed is stored **+110** | `speed 110` = "Normal"; +10 speed = 120 | `calc_bonuses()` sets `state->speed = 110` |
| Energy per full action | `world:move-energy:100` | `constants.txt` |
| HP/SP fractions | 16.16 fixed point (`chp_frac`, units of 1/65536) | `src/player-util.c:player_adjust_hp_precise()` |
| `randint0(n)` | uniform 0..n-1 | `src/z-rand.c` |
| `randint1(n)` | uniform 1..n | |
| `one_in_(n)` | probability 1/n | |
| Monster `spell-freq:N` | casts with probability 100/N percent per turn ("1_IN_N") | `src/mon-init.c:parse_monster_spell_freq()` |
| "Game turn" | the global `turn` counter; 10 game turns = 1 player turn at normal speed | `src/game-world.c:run_game_loop()` |

Stats are stored as 3..18, then 18/10 is stored as 28, 18/100 as 118 and so
on; tables are indexed by a **stat index** 0..37 (see §7).

---

## 2. Time, energy and speed

### 2.1 The game loop

`src/game-world.c:run_game_loop()` is the heart of the game. Each pass of the
inner loop is one **game turn**:

1. All monsters get processed (`process_monsters()`): each one gains
   `turn_energy(its speed)` energy, and any monster that had at least 100
   energy *before* the gain spends 100 and acts.
2. Every **10 game turns**, `process_world()` runs: regeneration, digestion,
   poison/cut damage, timed-effect countdowns, light fuel, the chance of a new
   monster appearing (`one_in_(500)`, `mon-gen:chance:500`), etc.
3. The player gains `turn_energy(player speed)`.
4. While the player has ≥ 100 energy, the player acts (monsters that have
   *more* energy than the player act first — `process_monsters(player->energy + 1)`).

`turn_energy(speed) = extract_energy[speed] * move_energy / 100`, and with the
default `move_energy` of 100 this is just the table value
(`src/game-world.c:turn_energy()`).

Important consequences:

* **Regeneration, digestion, poison and timed effects tick on game time**,
  i.e. once per 10 game turns, *not* once per player action. A fast character
  gets more actions per regeneration tick (more actions per point of poison,
  more actions per turn of a Potion of Speed's duration, etc.).
* Leftover energy is kept. If an action costs 95 energy, the remaining 5 carry
  over. This is how fractional blows and shots average out (see §3.2).
* New monsters spawn at a rate of 1 in 500 per 10 game turns — on average one
  wandering monster per 500 player turns at normal speed.

### 2.2 The energy table

From `src/game-world.c:extract_energy[]` (index = speed + 110):

| Speed | Energy / game turn | Game turns per action | Actions relative to normal |
|---|---|---|---|
| −20 | 3 | 33.3 | 0.30× |
| −15 | 4 | 25.0 | 0.40× |
| −10 | 5 | 20.0 | 0.50× |
| −5 | 6 | 16.7 | 0.60× |
| −3 | 7 | 14.3 | 0.70× |
| −2 | 8 | 12.5 | 0.80× |
| −1 | 9 | 11.1 | 0.90× |
| **0 (Normal)** | **10** | **10.0** | **1.00×** |
| +1 | 11 | 9.1 | 1.10× |
| +2 | 12 | 8.3 | 1.20× |
| +3 | 13 | 7.7 | 1.30× |
| +5 | 15 | 6.7 | 1.50× |
| +7 | 17 | 5.9 | 1.70× |
| **+10** | **20** | **5.0** | **2.00×** |
| +13 | 23 | 4.3 | 2.30× |
| +15 | 25 | 4.0 | 2.50× |
| +17 | 27 | 3.7 | 2.70× |
| **+20** | **30** | **3.3** | **3.00×** |
| +23 | 33 | 3.0 | 3.30× |
| +25 | 35 | 2.9 | 3.50× |
| +27 | 36 | 2.8 | 3.60× |
| **+30** | **38** | **2.6** | **3.80×** |
| +33 | 39 | 2.6 | 3.90× |
| +35 | 40 | 2.5 | 4.00× |
| +40 | 42 | 2.4 | 4.20× |
| +45 | 43 | 2.3 | 4.30× |
| +50 | 45 | 2.2 | 4.50× |
| +60 | 47 | 2.1 | 4.70× |
| +70 and above | 49 | 2.0 | 4.90× |

The full table is linear (+1 energy per +1 speed) from +0 to +26, then
flattens: +26 and +27 both give 36, +28 and +29 give 37, and from +30 onward each
extra point is worth less and less. Below normal, speed −1..−10 loses ground
fast (−10 halves your actions) and anything below −40 is a flat 1 energy.
Speed is clamped to 0..199 internally (`calc_bonuses()`).

### 2.3 What "+10 speed" really means

At +10 you get **twice as many actions** as a normal-speed monster. Every
monster at normal speed now gets one move for every two of yours: you can
fire twice before it closes one square, quaff a potion and still attack, or
walk away from it indefinitely.

Relative action ratio, player : monster (= energy ratio):

| Player \ Monster | +0 (10) | +10 (20) | +20 (30) | +30 (38) | +40 (42) |
|---|---|---|---|---|---|
| +0 | 1.00 | 0.50 | 0.33 | 0.26 | 0.24 |
| +10 | 2.00 | 1.00 | 0.67 | 0.53 | 0.48 |
| +20 | 3.00 | 1.50 | 1.00 | 0.79 | 0.71 |
| +30 | 3.80 | 1.90 | 1.27 | 1.00 | 0.90 |
| +40 | 4.20 | 2.10 | 1.43 | 1.11 | 1.00 |

Why speed is the single most important attribute:

* It multiplies **everything** — damage output, healing per unit of danger,
  escapes, detection, item use. No other single bonus scales all of your
  actions at once.
* It divides the enemy's damage *per action of yours*: a breath that would
  arrive every turn at +0 arrives every second turn at +10.
* Because of the table's curve, going from +0 to +10 is worth +100% actions,
  +10→+20 is another +50%, +20→+30 only +27%. Early speed is enormously
  valuable; deep in the game the battle is about matching fast monsters
  (e.g. in `monster.txt`: most hound packs are +0 to +10, time hounds +20;
Sauron, Gothmog, Ancalagon and Carcharoth are +20; Morgoth is +30).

Sources of speed seen by `calc_bonuses()`: the `SPEED` modifier on equipment,
`TMD_FAST`/`TMD_SPRINT` (+10), `TMD_TERROR` (+10), `TMD_SLOW` (−10),
`TMD_STONESKIN` (−5), overeating (see §14.2), and **burden** (§7.4).

Monster speed: each non-unique monster's speed is randomised at birth by
`rand_spread(0, turn_energy(race speed) / 10)` (`src/mon-make.c`), i.e. ±1 for
normal-speed monsters, ±2 for +10 monsters, ±3 for +20 monsters. Monsters also
start with random energy 0..49, so who moves first at the moment of contact is
partly luck. A hasted monster (`MON_TMD_FAST`) gets +10; a slowed monster gets
−2 per "level" of slow (`process_monsters()`).

### 2.4 Action costs

Most actions cost exactly 100 energy (`z_info->move_energy`). Exceptions found
in the source:

| Action | Energy | Source |
|---|---|---|
| Walk one step | `energy_per_move()`: 100 × (1+\|n\|−n)/(1+\|n\|), n = `MOVES` modifier | `src/player-util.c` |
| … with Moves +1 / +2 / −1 | 50 / 33 / 150 | |
| Melee round | `blow_energy` per blow, never more than 100 total | `src/player-attack.c:py_attack()` |
| Fire a missile | 100 × 10 / `num_shots` | `ranged_helper()` |
| Throw an object | 100 (throws use shots = 10) | `do_cmd_throw()` |
| Cast a spell | 100; **75** with `TMD_FASTCAST` | `src/cmd-obj.c:do_cmd_cast()` |
| Take off an item | 50 | `do_cmd_takeoff()` |
| Wield/drop/etc. | 50 or 100 depending on command | `src/cmd-obj.c` |
| Pick up | 10 per object picked up, capped at 100 | `src/cmd-pickup.c:do_cmd_pickup()` |
| Shield-bash stumble | +26..75 extra energy | `attempt_shield_bash()` |

The `MOVES` modifier (e.g. from some boots and artifacts) reduces only the
cost of *walking*; it is superb for escape and for "pillar dancing" but does
nothing for attacking.

---

## 3. Melee combat

Everything here is in `src/player-attack.c` unless noted. See also
`docs/attack.rst` for the player-facing description.

### 3.1 To-hit

```
chance_of_melee_hit_base = skill_thn + (to_h + weapon_to_h + (bless_wield ? 2 : 0)) * BTH_PLUS_ADJ
BTH_PLUS_ADJ = 3                                   (src/player.h)
chance = visible ? base : base / 2                 (chance_of_melee_hit)
```

* `skill_thn` is the melee skill (`SKILL_TO_HIT_MELEE`, §8).
* `to_h` (`p->state.to_h`) is the sum of to-hit on all *non-weapon,
  non-launcher* equipment plus `adj_dex_th` + `adj_str_th` plus temporary
  effects (Bless +10, Heroism +12, Fear −20, Stun −5/−20, hunger penalties).
* The weapon's own to-hit is added separately (`object_to_hit(weapon)`).
* Each point of to-hit is worth **3 points of skill**.
* **Unseen monsters halve your to-hit.**

The roll itself is `hit_chance()` / `test_hit()`, shared by the player, by
monster melee and by monster bolt spells:

```
to_hit = max(9, to_hit)
P(hit) = 12% + 83% * max(0, to_hit - AC*2/3) / to_hit
```

i.e. **12% always hit, 5% always miss**, and in between you need to beat
two-thirds of the defender's AC. Selected values:

| to_hit \ AC | 10 | 30 | 50 | 75 | 100 | 150 | 200 |
|---|---|---|---|---|---|---|---|
| 50 | 85% | 62% | 40% | 12% | 12% | 12% | 12% |
| 100 | 90% | 78% | 68% | 54% | 40% | 12% | 12% |
| 150 | 92% | 84% | 77% | 67% | 58% | 40% | 21% |
| 200 | 93% | 87% | 81% | 74% | 68% | 54% | 40% |
| 250 | 93% | 88% | 84% | 78% | 73% | 62% | 51% |
| 300 | 93% | 89% | 86% | 81% | 77% | 67% | 58% |

Worked example: a level-30 Human Warrior has melee skill 70 + 45×30/10 = 205.
With DEX 18/100 (+6) and STR 18/100 (+5) and a (+5,+5) weapon, total to-hit is
16, worth 48: `to_hit = 253`. Against an AC 60 monster (needs 40):
12 + 83 × 213/253 = **81.9%**.

### 3.2 Blows per round

`src/player-calcs.c:calc_blows()`:

```
div       = max(weapon_weight, class min-weight)            (tenths of lb)
str_index = min(11, adj_str_blow[STR] * class strength-multiplier / div)
dex_index = min(11, adj_dex_blow[DEX])
energy    = blows_table[str_index][dex_index]
blows     = min(10000 / energy, 100 * class max-attacks) + 100 * extra_blows
blows     = max(blows, 100)            (200 minimum with birth_percent_damage)
```

Class parameters (`lib/gamedata/class.txt`):

| Class | max-attacks | min-weight | strength-multiplier |
|---|---|---|---|
| Warrior | 6 | 30 (3 lb) | 5 |
| Paladin | 5 | 30 | 5 |
| Blackguard | 5 | **100** (10 lb) | 5 |
| Rogue | 5 | 20 | 4 |
| Ranger | 5 | 35 | 4 |
| Priest | 4 | 35 | 3 |
| Druid | 4 | 35 | 3 |
| Necromancer | 4 | 35 | 3 |
| Mage | 4 | 40 | 2 |

The blows table (energy per blow; blows = 100 / entry):

| P \ D | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11+ |
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
| 11+ | 33 | 29 | 26 | 24 | 22 | 21 | 20 | 19 | 18 | 17 | 16 | 15 |

`adj_str_blow` and `adj_dex_blow` are listed in §7. The DEX index
`adj_dex_blow` is 0 below DEX 10, 1 for 10–16, 2 for 17–18/19, 3 for
18/20–18/39, 4 for 18/40–18/59, 5 for 18/60–18/79, 6 for 18/80–18/99, 7 for
18/100–18/119, 8 for 18/120–18/149, 9 for 18/150–18/179, 10 for
18/180–18/199 and 11 for 18/200+.

**How the round is spent** (`py_attack()`): `blow_energy = 10000 / num_blows`;
the player keeps swinging while the remaining part of a 100-energy budget
covers another blow, and pays only for the blows made. With 5.26 blows
(19 energy each) you swing 5 times for 95 energy and keep 5 energy for later,
so over many rounds you really do get 5.26 blows per 100 energy. The round
stops early if the monster dies — and you only pay for the blows used, so
killing a weak monster with your first blow costs only one blow's energy.

Blows computed for common cases (STR across, DEX in sub-columns):

**Warrior** (blows; rows = weapon weight in lb)

| Weight | S 18/50: D 18/50 | D 18/100 | D 18/150 | S 18/100: D 18/50 | D 18/100 | D 18/150 | S 18/150: D 18/50 | D 18/100 | D 18/150 | S 18/200: D 18/50 | D 18/100 | D 18/150 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ≤3 | 4.54 | 5.26 | 5.88 | 4.54 | 5.26 | 5.88 | 4.54 | 5.26 | 5.88 | 4.54 | 5.26 | 5.88 |
| 12 | 2.00 | 3.33 | 4.34 | 3.03 | 4.16 | 4.76 | 3.57 | 4.54 | 5.26 | 4.16 | 5.00 | 5.55 |
| 15 | 2.00 | 3.33 | 4.34 | 2.77 | 4.00 | 4.76 | 3.03 | 4.16 | 4.76 | 3.57 | 4.54 | 5.26 |
| 20 | 1.66 | 2.85 | 4.00 | 2.38 | 3.57 | 4.54 | 2.77 | 4.00 | 4.76 | 3.03 | 4.16 | 4.76 |
| 26 | 1.66 | 2.85 | 4.00 | 2.00 | 3.33 | 4.34 | 2.38 | 3.57 | 4.54 | 2.77 | 4.00 | 4.76 |

**Mage** (same layout)

| Weight | S 18/50: D 18/50 | D 18/100 | D 18/150 | S 18/100: D 18/50 | D 18/100 | D 18/150 | S 18/150: D 18/50 | D 18/100 | D 18/150 | S 18/200: D 18/50 | D 18/100 | D 18/150 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ≤4 | 2.38 | 3.57 | 4.00 | 3.33 | 4.00 | 4.00 | 3.84 | 4.00 | 4.00 | 4.00 | 4.00 | 4.00 |
| 12 | 1.66 | 2.85 | 4.00 | 2.00 | 3.33 | 4.00 | 2.00 | 3.33 | 4.00 | 2.38 | 3.57 | 4.00 |
| 20 | 1.33 | 2.38 | 3.33 | 1.66 | 2.85 | 4.00 | 1.66 | 2.85 | 4.00 | 2.00 | 3.33 | 4.00 |

Observations:

* With a light weapon the STR index saturates at 11 almost immediately, and
  **DEX alone decides your blows** (5.26 at DEX 18/100, 5.88 at 18/150 for a
  warrior).
* With a heavy weapon both matter; each extra STR step is worth about as much
  as a DEX step.
* Blackguards' `min-weight` of 10 lb means they gain nothing from weapons
  lighter than 10 lb — they are designed around heavy weapons.
* `BLOWS` modifiers on equipment add **whole** blows (`extra_blows` is ×100 in
  `calc_blows()`), even beyond the class maximum.
* Blackguard *Bloodlust* (`TMD_BLOODLUST`) adds `timer/20` extra blows and
  `timer/2` to-dam (`calc_bonuses()`).

**Heavy wield.** If `adj_str_hold[STR] < weapon weight in lb`, you suffer
`2 × (hold − weight_lb)` to-hit and `calc_blows()` is *not called*: you get
**exactly one blow** (`state->num_blows` stays 100). `adj_str_hold` is 4 lb at
STR 3, 14 lb at STR 10, 30 lb at STR 18, 80 lb at 18/100 and 100 lb at
18/200 (§7). The same `hold` value limits launchers (§4).

### 3.3 Damage per blow (standard combat)

`melee_damage()` + `critical_melee()` + `py_attack_real()` give this
pipeline:

```
dmg  = damroll(dd, ds)                     (1 if unarmed)
dmg *= best slay or brand multiplier       (dice only!)
dmg += weapon to_d
dmg  = critical_melee(dmg)                 (weapon only; no crits unarmed)
dmg += p->state.to_d                       (rings, gloves, STR, Bloodlust...)
```

Key facts:

* **Slays and brands multiply only the dice**, not any plus.
* **Criticals multiply dice and the weapon's to-dam**, but not off-weapon
  to-dam.
* **Off-weapon to-dam (rings of Damage, STR bonus, gauntlets, etc.) is added
  flat to every blow** — so it is worth `blows × to_d` per round. This is why
  many-blow light weapons love +damage gear.
* Only the single best multiplier applies (`improve_attack_modifier()` scans
  the weapon, every non-launcher equipment slot, and temporary brands).
* `IMPACT` weapons cause an earthquake when one blow does more than 50 damage.
* If a monster is afraid, confused, held or stunned it is "debuffed": your
  to-hit for crit purposes gets +10 (`melee-critical:debuff-toh:10`).

### 3.4 Melee criticals

From `critical_melee()` with the constants in `constants.txt`:

```
chance = weight + 5 * (to_h + weapon_to_h [+10 if debuffed]) + skill_thn - 60
crit if randint1(5000) <= chance
power  = weight + randint1(650)
```

| power | Result | Damage (applied to dice×slay + weapon to_d) | Message |
|---|---|---|---|
| < 400 | Good hit | ×2 + 5 | "It was a good hit!" |
| 400–699 | Great hit | ×2 + 10 | "It was a great hit!" |
| 700–899 | Superb hit | ×3 + 15 | "It was a superb hit!" |
| 900–1299 | \*GREAT\* hit | ×3 + 20 | "It was a \*GREAT\* hit!" |
| ≥ 1300 | \*SUPERB\* hit | ×4 + 20 | "It was a \*SUPERB\* hit!" |

Because `power` ≤ weight + 650, \*GREAT\* needs a weapon of at least 25 lb
(power ≥ 900) and \*SUPERB\* at least 65 lb — effectively only giant weapons.
Crit chance at our level-30 warrior (skill 205, total to-hit 16):

| Weapon | weight | crit chance | crit type distribution |
|---|---|---|---|
| Dagger | 1.2 lb | (12+80+205−60)/5000 = 4.7% | 60% Good, 40% Great |
| Blade of Chaos | 18 lb | (180+80+205−60)/5000 = 8.1% | 34% Good, 46% Great, 20% Superb |
| Mace of Disruption | 40 lb | (400+80+205−60)/5000 = 12.5% | Great 46%, Superb 31%, \*GREAT\* 23% |

Criticals are a modest bonus in standard combat; heavy weapons benefit most.

### 3.5 Slays and brands

From `lib/gamedata/slay.txt` and `lib/gamedata/brand.txt` (standard
multiplier / O-combat multiplier ×10):

| Slay | vs | Mult | O-mult |
|---|---|---|---|
| EVIL_2 | evil | ×2 | 18 |
| ANIMAL_2 | animals | ×2 | 20 |
| ORC_3, TROLL_3, GIANT_3 | orcs, trolls, giants | ×3 | 25 |
| DEMON_3, DRAGON_3, UNDEAD_3 | demons, dragons, undead | ×3 | 25 |
| DEMON_5, DRAGON_5, UNDEAD_5 ("\*slay\*"/"kill") | demons, dragons, undead | ×5 | 35 |

| Brand | Mult | O-mult | Ignored by | Doubled vs |
|---|---|---|---|---|
| ACID_3 / ELEC_3 / FIRE_3 / COLD_3 / POIS_3 | ×3 | 25 | IM_ACID / IM_ELEC / IM_FIRE / IM_COLD / IM_POIS | HURT_FIRE (fire), HURT_COLD (cold) |
| ACID_2 / ELEC_2 / FIRE_2 / COLD_2 / POIS_2 ("weak" brands) | ×2 | 15 | same | same |

Vulnerability (`src/obj-slays.c:get_monster_brand_multiplier()`): a monster
with `HURT_FIRE` takes **double the multiplier** from a fire brand (×6 instead
of ×3); in O-combat the *extra* part doubles (25 → 40).

### 3.6 Worked example: Dagger vs Blade of Chaos

Level-30 Human Warrior, STR 18/100 (to-dam +6 from `adj_str_td`, to-hit +5),
DEX 18/100 (to-hit +6), no other gear bonuses. Both weapons are (+5,+5).
Target: AC 60 (82% to hit, from §3.1). Expected values computed from the
formulas above, including crits:

| Weapon | Blows | Per blow (avg) | Per round, no slay | Per round, ×2 slay | Per round, ×3 slay |
|---|---|---|---|---|---|
| Dagger 1d4 (1.2 lb) | 5.26 | 7.5 dice+plus → 14.2 with crits and +6 | **61** | 72 | 84 |
| Blade of Chaos 6d5 (18 lb) | 3.57 | 23 → 32.0 | **94** | 151 | 209 |

Lessons:

* Dice matter more than blows once slays are involved: the ×3 slay is worth
  +115 per round on the Blade, +23 on the Dagger.
* The Dagger catches up when you add flat to-dam: each +1 to-dam on a ring is
  +4.3 damage per round for the Dagger user, +2.9 for the Blade user.
* Run the numbers for your own gear with the in-game weapon description
  (`I`nspect), which uses exactly these functions (`src/obj-info.c`).

### 3.7 O-combat (`birth_percent_damage`)

The birth option *"To-damage is a percentage of dice (experimental)"*
(`birth_percent_damage`, `src/list-options.h`) replaces the damage model:

* `o_melee_damage()`: deadliness = player to-dam + weapon to-dam is converted
  through `deadliness_conversion[]` to a percentage (1 → 5%, 10 → 39%,
  20 → 69%, 30 → 99%, 40 → 124%, 60 → 174%, 100 → 253%, capped at 255%) and **added to
  the size of each die** (with the slay/brand O-multiplier also scaling the
  die). Flat to-dam is *not* added afterward.
* Criticals add **extra dice**: chance = power / (power + 240) where power =
  `chance_of_melee_hit_base / 3`; then 1/40 → +5 dice (\*SUPERB\*), else 1/12
  → +4, else 1/3 → +3, else 1/2 → +2, else +1.
* Minimum of 2 blows per round (`calc_blows()`).
* Ranged: launcher multiplier and deadliness also scale die size; crit
  chance = power / (power + 360).

Example (same warrior, Dagger, deadliness 11 → +42%): the d4 becomes a "d6.1"
(rolled as d6, or d7 10% of the time); crit chance = 84/(84+240) ≈ 26%,
adding on average ~2.2 extra dice when it triggers. The rest of this manual
assumes the standard (default) model.

### 3.8 Shield bashes

Warriors, Paladins and Blackguards (`SHIELD_BASH` player flag) may bash
before their blows (`attempt_shield_bash()`):

* Only with a shield, only vs a visible monster whose level ≥ player level/2.
* Chance: `skill_thn/8 + adj_dex_th/2` vs `randint0(200 + monster level)`;
  ×4 when unarmed, ×2 if `weapon dice × blows < 3 × shield dice`.
* Damage: shield dice × (quality/40 + level/14) + `adj_str_td`, where quality
  = skill/4 + body weight/8 + carried weight/80 + shield weight/2. Can stun
  and confuse.
* You stumble (lose 26–75% of a turn) when `35 + adj_dex_th < randint1(60)`.

### 3.9 Other melee details

* **Fear** (`OF_AFRAID`) prevents melee entirely ("You are too afraid to
  attack"), gives −20 to-hit and, curiously, +8 AC (`calc_bonuses()`).
  Warriors become immune at level 30 (`BRAVERY_30`); Heroism/Berserk cure it.
* **Berserk** (`TMD_SHERO`) gives +75 *melee skill* (= +25 to-hit) and −10 AC.
* **Blessed weapons**: Priests and Paladins (`BLESS_WEAPON`) wielding a hafted
  weapon or a `BLESSED` weapon get +2 to-dam and +2 to-hit
  (`state->bless_wield`).
* **Vampiric strike** (`TMD_ATT_VAMP`) heals you by damage dealt to living
  monsters; **Confusing touch** (`TMD_ATT_CONF`) confuses on the next hit.
* Every melee attack wakes the target (`monster_wake(mon, false, 100)`) and
  breaks monster *hold*.

---

## 4. Missile and throwing combat

### 4.1 Launchers and ammunition

Base launchers (`lib/gamedata/object.txt`; multiplier is the launcher `pval`):

| Launcher | Weight | Multiplier | Ammo |
|---|---|---|---|
| Sling | 0.5 lb | ×2 | shots, pebbles |
| Short Bow | 3 lb | ×2 | arrows |
| Long Bow | 4 lb | ×3 | arrows |
| Light Crossbow | 11 lb | ×3 | bolts |
| Heavy Crossbow | 20 lb | ×4 | bolts |

| Ammo | Dice | Weight | Break % on hit (on miss) |
|---|---|---|---|
| Rounded Pebble | 1d2 | 0.4 lb | 0 (0) |
| Iron Shot | 1d4 | 0.5 lb | 0 (0) |
| Mithril Shot | 2d4 | 0.2 lb | 0 (0) |
| Arrow | 1d4 | 0.2 lb | 35 (12) |
| Mithril Arrow | 3d4 | 0.1 lb | 35 (12) |
| Seeker Arrow | 4d4 | 0.2 lb | 35 (12) |
| Bolt | 1d5 | 0.3 lb | 20 (4) |
| Mithril Bolt | 3d5 | 0.2 lb | 20 (4) |
| Seeker Bolt | 4d5 | 0.3 lb | 20 (4) |

Breakage (`breakage_chance()`): the base `break` percentage from
`object_base.txt` applies on a hit, its square/100 on a miss. **Artifacts never
break.** Non-ammo throwing weapons (`THROWING` flag, not `EXPLODE`) break only
1% of the time. Potions, flasks and food always break.

### 4.2 Ranged to-hit

`chance_of_missile_hit_base()` and `chance_of_missile_hit()`:

```
fired:   skill_thb + (to_h + launcher_to_h + ammo_to_h) * 3
thrown THROWING weapon: skill_tht + (to_h + object_to_h) * 3
thrown other object:    skill_tht * 3/2 + object_to_h * 3
then:    chance -= distance to target           (1 per grid)
         chance /= 2 if the monster is not "obvious" (unseen)
```

The distance penalty is only 1 point of skill per grid, i.e. at most 20 — tiny
compared with a skill of 200+. The same 12%/5% `hit_chance()` roll vs
AC×2/3 applies.

### 4.3 Shots, range and might

`calc_bonuses()`:

* `num_shots` starts at 10 (one shot) and adds the launcher's `SHOTS` modifier
  in **tenths** (Extra Shots ego: `SHOTS[2d3]` = +0.2..+0.6 shots).
* Rangers (`FAST_SHOT`) get `+level/3` tenths: +1 full shot at level 30,
  +1.6 at level 50.
* `ammo_mult = launcher pval + MIGHT` modifiers.
* **Heavy shooting**: if `adj_str_hold < launcher weight in lb`, to-hit
  suffers `2 × (hold − weight)` and you lose all extra shots and extra might.
* Range: `min(6 + 2 × multiplier, 20)` grids (`do_cmd_fire()`), so a ×3 bow
  reaches 12 grids, ×4 reaches 14, ×7 reaches the cap of 20.
* Each shot costs `100 × 10 / num_shots` energy.

### 4.4 Ranged damage (standard combat)

`ranged_damage()` is **different** from melee in an important way:

```
mult = launcher multiplier + slay/brand multiplier          (ADDITIVE)
dmg  = (damroll(ammo dice) + ammo to_d + launcher to_d) * mult
```

* Slays/brands **add** to the launcher multiplier: ×3 bow + ×2 slay evil = ×5,
  not ×6.
* Both ammo and launcher to-dam *are* multiplied.
* Player to-dam from rings etc. does **not** apply to missiles at all.

Ranged criticals (`critical_shot()`):

```
chance = weight + 4 * (to_h + ammo_to_h [+10 if debuffed]) + 2 * level     out of 5000
power  = weight + randint1(500)
< 500: ×2+5 ("good")   500–999: ×2+10 ("great")   ≥1000: ×3+15 ("superb")
```

Since ammo weighs almost nothing, ranged crits are rare (a few percent) and
almost always the ×2+5 kind.

**Worked example.** Level-30 Human Ranger: shooting skill 72 + 45×3 = 207.
Long Bow ×3 (+5,+5), Arrows 1d4 (+5,+5), player to-hit 11.

* To-hit: 207 + (11+5+5)×3 = 270, −5 at 5 grids = 265 → vs AC 60: 82.5%.
* Damage per arrow: (2.5 + 5 + 5) × 3 = **37.5**; vs evil with Slay Evil
  arrows ×5 = 62.5.
* Shots: 10 + 30/3 = 20 → **2 shots per 100 energy**.
* Crit chance: (2 + 4×16 + 60)/5000 = 2.5%.
* Expected: 2 × 37.5 × 0.825 ≈ **62 per turn**, ≈ 103 vs evil — at range, with
  no retaliation. Compare with the warrior's 94 in melee (§3.6).

### 4.5 Throwing

`do_cmd_throw()` and `ranged_damage()`:

* A throw always costs 100 energy.
* Range = `min((adj_str_blow[STR] + 20) × 10 / max(weight, 10), 10)` grids
  (weight in tenths of a pound).
* **Throwing weapons** (`THROWING` flag) multiply damage by
  `2 + weight/12` (weight in tenths of a pound, integer division), then by any
  brand/slay multiplier (+1 base, so a ×3 brand gives ×4).
* Flasks of oil (`EXPLODE`) get a further ×3 (`make_ranged_throw()`).
* Throwing weapons in the quiver take 5 quiver "slots" each
  (`carry-cap:thrown-quiver-mult:5`).

| Thrown object | Dice | Weight | Throw multiplier | Avg damage (no plusses) |
|---|---|---|---|---|
| Dagger | 1d4 | 1.2 lb | ×3 | 7.5 |
| Spear | 1d6 | 5 lb | ×6 | 21 |
| Throwing Axe | 2d3 | 4 lb | ×5 | 20 |
| Throwing Hammer | 2d4 | 5.5 lb | ×6 | 30 |
| Flask of oil | 1d4 | 2 lb | ×3, ×3 explode | 22.5 |

A (+5,+5) Throwing Axe: (4 + 5) × 5 = 45 per throw. Rogues (throwing skill
72 + 45 per 10 levels) are the natural throwers; flasks of oil are the
best early-game ranged attack for everyone.

---

## 5. Armour class and monster melee

### 5.1 Your AC

Total AC = Σ base armour (`obj->ac`) + Σ to-AC (`to_a`) + `adj_dex_ta[DEX]`
+ temporary effects (`calc_bonuses()`):

| Effect | AC |
|---|---|
| Blessing | +5 (and +10 to-hit) |
| Stone Skin | +40 (and −5 speed) |
| Shield (spell/potion) | +50 |
| Invulnerability | +100 |
| Berserk | −10 |
| Fear | +8 |

### 5.2 Will the monster hit you?

`src/mon-attack.c:make_attack_normal()` / `check_hit()`:

```
to_hit = max(monster level, 1) * 3 + blow_effect power
         (×0.75 if the monster is stunned: STUN_HIT_REDUCTION 25)
P(hit) = hit_chance(to_hit, your AC)     -- same 12%/5% formula as §3.1
```

Blow effect powers (`lib/gamedata/blow_effects.txt`):

| Power | Effects |
|---|---|
| 60 | SHATTER (earthquake blows) |
| 40 | HURT, ELEC, FIRE, COLD |
| 20 | POISON, ACID, CONFUSE, EXP_10/20/40/80 |
| 10 | DISENCHANT, DRAIN_CHARGES |
| 0 | EAT_GOLD, EAT_ITEM, EAT_FOOD, EAT_LIGHT, BLIND, TERRIFY, PARALYZE, LOSE_STR/INT/WIS/DEX/CON/ALL, HALLU, BLACK_BREATH |

Monster hit chance (%), by monster level and blow power, vs your AC:

| Mon. level / power | AC 20 | AC 50 | AC 100 | AC 150 | AC 200 | AC 250 |
|---|---|---|---|---|---|---|
| 5 / 0 | 23 | 12 | 12 | 12 | 12 | 12 |
| 5 / 40 | 75 | 45 | 12 | 12 | 12 | 12 |
| 15 / 0 | 71 | 34 | 12 | 12 | 12 | 12 |
| 15 / 40 | 82 | 63 | 31 | 12 | 12 | 12 |
| 30 / 0 | 83 | 65 | 34 | 12 | 12 | 12 |
| 30 / 40 | 87 | 74 | 53 | 31 | 12 | 12 |
| 50 / 0 | 88 | 77 | 58 | 40 | 21 | 12 |
| 50 / 40 | 89 | 81 | 66 | 51 | 37 | 22 |
| 70 / 40 | 91 | 84 | 73 | 62 | 51 | 40 |
| 90 / 60 | 92 | 87 | 78 | 70 | 62 | 53 |

**AC is excellent protection against power-0 "touch" effects** (paralysis,
stat drain, theft): a level-30 monster's paralysing touch hits AC 100 only
34% of the time and AC 150 only the minimum 12%. Against high-level HURT
blows AC is much less decisive.

### 5.3 How much damage AC removes

`adjust_dam_armor()`:

```
damage -= damage * min(AC, 240) / 400
```

| AC | 20 | 50 | 100 | 150 | 200 | 240+ |
|---|---|---|---|---|---|---|
| Damage taken | 95% | 87.5% | 75% | 62.5% | 50% | 40% |

This reduction is applied only to:

* **HURT** blows (`melee_effect_handler_HURT()`) and **SHATTER** blows;
* the *physical* half of **elemental** blows (ACID/ELEC/FIRE/COLD), using
  AC + 50. The blow then does `max(physical, elemental-after-resistance)`
  (`melee_effect_elemental()`), so resistance only helps once it brings the
  elemental part below the armour-reduced physical part.

All other blow effects (poison, stat drain, paralysis, …) do their full dice
damage. Finally every hit passes through `player_apply_damage_reduction()`:
flat `dam_red` (from `DAM_RED` modifiers) then percentage reduction; and
Invulnerability negates any hit below 9000.

Stunned monsters also do 25% less damage (`STUN_DAM_REDUCTION`).

### 5.4 Cuts and stuns from monster blows

Blow *methods* (HIT, BITE, CLAW, CRUSH…) carry `cut` / `stun` flags in
`blow_methods.txt`. When both apply, one is chosen 50/50. The severity comes
from `monster_critical()`: the hit must do at least 95% of maximum damage;
blows under 20 damage only qualify with probability `dam`%; then the level is
1 (≤11), 2 (12–18), 3 (19–25), 4 (26–33), 5 (34–45), 6 (>45), +1 for a
perfect roll and possibly more. Cut amounts range from 1d5 up to 300/500;
stun from 1d5 up to 100/200. A "Heavy Stun" (>50) or "Knocked Out" (>150)
is extremely dangerous (§12, §14).

### 5.5 Protection from evil

`TMD_PROTEVIL` repels a melee blow from an **evil** monster when
`player level ≥ monster level` and `randint0(100) + player level > 50`,
i.e. with probability (49 + level)% — 69% at level 20, 89% at level 40. It does nothing vs
monsters out-levelling you.

---

## 6. Hit points, levels and experience

### 6.1 Hit dice and HP rolls

`src/player-birth.c`: `hitdie = race hitdie + class hitdie`. Level 1 HP is
the full hitdie; each later level adds `randint1(hitdie)`. The whole
50-level sequence is re-rolled at birth until the level-50 total lies between
`50 + 50×(hitdie−1)×3/8` and `50 + 50×(hitdie−1)×5/8` (`roll_hp()`), so the
"bad roll" tail is cut off.

Race hitdie: Human 10, Half-Elf 10, Elf 9, Hobbit 7, Gnome 8, Dwarf 11,
Half-Orc 10, Half-Troll 12, Dúnadan 10, High-Elf 10, Kobold 8.
Class hitdie: Warrior 9, Mage 0, Druid 2, Priest 2, Necromancer 2, Paladin 6,
Rogue 4, Ranger 5, Blackguard 8.

Combined hitdie (expected base HP at level 50 before CON):

| Race | Warrior | Mage | Druid | Priest | Necro | Paladin | Rogue | Ranger | Blackguard |
|---|---|---|---|---|---|---|---|---|---|
| Human | 19 (509) | 10 (279) | 12 (330) | 12 (330) | 12 (330) | 16 (432) | 14 (381) | 15 (407) | 18 (483) |
| Half-Elf | 19 (509) | 10 (279) | 12 (330) | 12 (330) | 12 (330) | 16 (432) | 14 (381) | 15 (407) | 18 (483) |
| Elf | 18 (483) | 9 (254) | 11 (305) | 11 (305) | 11 (305) | 15 (407) | 13 (356) | 14 (381) | 17 (458) |
| Hobbit | 16 (432) | 7 (203) | 9 (254) | 9 (254) | 9 (254) | 13 (356) | 11 (305) | 12 (330) | 15 (407) |
| Gnome | 17 (458) | 8 (228) | 10 (279) | 10 (279) | 10 (279) | 14 (381) | 12 (330) | 13 (356) | 16 (432) |
| Dwarf | 20 (534) | 11 (305) | 13 (356) | 13 (356) | 13 (356) | 17 (458) | 15 (407) | 16 (432) | 19 (509) |
| Half-Orc | 19 (509) | 10 (279) | 12 (330) | 12 (330) | 12 (330) | 16 (432) | 14 (381) | 15 (407) | 18 (483) |
| Half-Troll | 21 (560) | 12 (330) | 14 (381) | 14 (381) | 14 (381) | 18 (483) | 16 (432) | 17 (458) | 20 (534) |
| Dúnadan | 19 (509) | 10 (279) | 12 (330) | 12 (330) | 12 (330) | 16 (432) | 14 (381) | 15 (407) | 18 (483) |
| High-Elf | 19 (509) | 10 (279) | 12 (330) | 12 (330) | 12 (330) | 16 (432) | 14 (381) | 15 (407) | 18 (483) |
| Kobold | 17 (458) | 8 (228) | 10 (279) | 10 (279) | 10 (279) | 14 (381) | 12 (330) | 13 (356) | 16 (432) |

(Expected value = hitdie + 49 × (hitdie+1)/2.)

### 6.2 CON bonus

`calc_hitpoints()`:

```
max HP = player_hp[level-1] + adj_con_mhp[CON] * level / 100     (minimum level+1)
```

`adj_con_mhp` is in hundredths of an HP per level:

| CON | 3 | 7 | 10 | 12 | 14 | 16 | 17 | 18 | 18/50 | 18/80 | 18/100 | 18/150 | 18/170 | 18/200+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| HP / level | −2.5 | −0.5 | −0.05 | +0.05 | +0.25 | +0.75 | +1 | +1.5 | +2.75 | +4 | +5 | +7.5 | +9 | +12.5 |
| at level 50 | −125 | −25 | −2 | +2 | +12 | +37 | +50 | +75 | +137 | +200 | +250 | +375 | +450 | +625 |

So a level-50 Human Warrior averages ~509 base + 250 at CON 18/100 = ~760 HP;
with CON 18/200 it is ~1130. **CON is the HP stat**, and its effect is
retroactive (all levels), so CON boosts are worth more the higher your level.

### 6.3 Experience table

`src/player.c:player_exp[]`. To reach level L you need
`player_exp[L−2] × expfact / 100` experience, where `expfact` is the race's
`exp` (classes add 0 in 4.2.6): Human 100%, High-Elf 145%, all other races
120%.

| Level | Base exp | Level | Base exp | Level | Base exp |
|---|---|---|---|---|---|
| 2 | 10 | 19 | 3,600 | 36 | 450,000 |
| 3 | 25 | 20 | 4,400 | 37 | 550,000 |
| 4 | 45 | 21 | 5,400 | 38 | 700,000 |
| 5 | 70 | 22 | 6,800 | 39 | 850,000 |
| 6 | 100 | 23 | 8,400 | 40 | 1,000,000 |
| 7 | 140 | 24 | 10,200 | 41 | 1,250,000 |
| 8 | 200 | 25 | 12,500 | 42 | 1,500,000 |
| 9 | 280 | 26 | 17,500 | 43 | 1,800,000 |
| 10 | 380 | 27 | 25,000 | 44 | 2,100,000 |
| 11 | 500 | 28 | 35,000 | 45 | 2,400,000 |
| 12 | 650 | 29 | 50,000 | 46 | 2,700,000 |
| 13 | 850 | 30 | 75,000 | 47 | 3,000,000 |
| 14 | 1,100 | 31 | 100,000 | 48 | 3,500,000 |
| 15 | 1,400 | 32 | 150,000 | 49 | 4,000,000 |
| 16 | 1,800 | 33 | 200,000 | 50 | 4,500,000 |
| 17 | 2,300 | 34 | 275,000 | | |
| 18 | 2,900 | 35 | 350,000 | | |

(Multiply by 1.2 for most races, 1.45 for High-Elves.)

### 6.4 Experience from kills

`src/mon-util.c:player_kill_monster()`:

```
exp = monster mexp * monster level / player level      (fractions carried in exp_frac)
```

Experience per kill falls as 1/your level, so "level-appropriate" monsters
keep you levelling while shallow monsters stop being worth anything.

Other sources:

* first successful cast of each spell: `spell exp × spell level`
  (`src/player-spell.c`);
* learning an object flavour by use: `(object level + player level/2) /
  player level` (`src/obj-knowledge.c:object_learn_on_use()`);
* disarming traps: `1 + depth/5`; disarming/opening chests;
* Potions of Experience (the `GAIN_EXP` handler halves its dice, so
  `dice:200000` grants 100,000 exp).

### 6.5 Experience drain and level restoration

Drained experience lowers `exp` but max exp is kept; `player_exp_gain()` gives
back 10% of any gain toward `max_exp` while drained. Hold Life prevents most
drains. **Gaining (or regaining) a level restores all drained stats**
(`adjust_level()` calls `EF_RESTORE_STAT` for all five stats on every level
up).

---

## 7. Stats

### 7.1 How a stat value becomes a bonus

`calc_bonuses()` computes, for each stat:

```
stat_top = modify_stat_value(stat_max, race adj + class adj + equipment)
stat_use = modify_stat_value(stat_cur, race adj + class adj + equipment)
index    = 0           if use <= 3
           use - 3     if use <= 18          (3..18  -> 0..15)
           15 + (use-18)/10  up to 18/219    (18/00 -> 15, 18/10 -> 16, ...)
           37          if use >= 18/220
```

`modify_stat_value()` moves in single points below 18 and in steps of 10
("18/10, 18/20, …") above. Every `adj_*` table is indexed by that 0..37 index.
Stats displayed in lower case / with a `!` are drained (`stat_cur <
stat_max`).

Race and class modifiers (`p_race.txt`, `class.txt`; STR/INT/WIS/DEX/CON):

| Race | STR | INT | WIS | DEX | CON | | Class | STR | INT | WIS | DEX | CON |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | | Warrior | +3 | −2 | −2 | +2 | +2 |
| Half-Elf | 0 | +1 | −1 | +1 | −1 | | Mage | −3 | +3 | 0 | 0 | −2 |
| Elf | −1 | +2 | −1 | +1 | −1 | | Druid | −2 | 0 | +3 | −2 | 0 |
| Hobbit | −2 | +2 | +1 | +3 | +2 | | Priest | −1 | −3 | +3 | −1 | +1 |
| Gnome | −1 | +2 | 0 | +2 | +1 | | Necromancer | −3 | +3 | 0 | 0 | −2 |
| Dwarf | +2 | −3 | +2 | −2 | +2 | | Paladin | +1 | −3 | +1 | −1 | +2 |
| Half-Orc | +2 | −1 | 0 | 0 | +1 | | Rogue | 0 | +1 | −3 | +3 | −1 |
| Half-Troll | +4 | −4 | −2 | −4 | +3 | | Ranger | 0 | 0 | +2 | +1 | −1 |
| Dúnadan | +1 | +2 | +2 | +2 | +3 | | Blackguard | +2 | 0 | −3 | 0 | +2 |
| High-Elf | +1 | +3 | −1 | +3 | +1 | | | | | | | |
| Kobold | −1 | −1 | 0 | +2 | +2 | | | | | | | |

Spell stats (`lib/gamedata/realm.txt`): **INT** for arcane (Mage, Rogue) and
shadow (Necromancer, Blackguard); **WIS** for divine (Priest, Paladin) and
nature (Druid, Ranger).

### 7.2 The physical stat tables

All values from `src/player-calcs.c`. "Burden unit" is `adj_str_wgt`: your
carrying limit is 10× this in pounds, slowdown begins at **5×** and you lose 1
speed per additional **1×** (see §7.4). "Hold" is `adj_str_hold`, the
heaviest weapon/launcher in pounds you can use without penalty. "CON
recovery" is how many points of cut, stun and poison you shed per game tick
(`adj_con_fix + 1`, `src/game-world.c:decrease_timeouts()`). "Theft save" is
added to your level as the percent chance to foil EAT_GOLD/EAT_ITEM touches
(`src/mon-blows.c`).

| Stat | Index | STR to-dam | STR to-hit | Blow (STR) | Hold (lb) | Burden unit (lb) | Dig | DEX to-hit | DEX AC | Blow idx (DEX) | Disarm phys | Theft save | CON HP/lvl ×100 | CON recovery |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 3 | 0 | -2 | -3 | 3 | 4 | 5 | 0 | -3 | -4 | 0 | 0 | 0 | -250 | 1 |
| 4 | 1 | -2 | -2 | 4 | 5 | 6 | 0 | -2 | -3 | 0 | 0 | 1 | -150 | 1 |
| 5 | 2 | -1 | -1 | 5 | 6 | 7 | 1 | -2 | -2 | 0 | 0 | 2 | -100 | 1 |
| 6 | 3 | -1 | -1 | 6 | 7 | 8 | 2 | -1 | -1 | 0 | 0 | 3 | -75 | 1 |
| 7 | 4 | 0 | 0 | 7 | 8 | 9 | 3 | -1 | 0 | 0 | 0 | 4 | -50 | 1 |
| 8 | 5 | 0 | 0 | 8 | 10 | 10 | 4 | 0 | 0 | 0 | 1 | 5 | -25 | 1 |
| 9 | 6 | 0 | 0 | 9 | 12 | 11 | 4 | 0 | 0 | 0 | 1 | 5 | -10 | 1 |
| 10 | 7 | 0 | 0 | 10 | 14 | 12 | 5 | 0 | 0 | 1 | 1 | 6 | -5 | 1 |
| 11 | 8 | 0 | 0 | 11 | 16 | 13 | 5 | 0 | 0 | 1 | 1 | 6 | 0 | 1 |
| 12 | 9 | 0 | 0 | 12 | 18 | 14 | 6 | 0 | 0 | 1 | 1 | 7 | 5 | 1 |
| 13 | 10 | 0 | 0 | 13 | 20 | 15 | 6 | 0 | 0 | 1 | 1 | 7 | 10 | 1 |
| 14 | 11 | 0 | 0 | 14 | 22 | 16 | 7 | 0 | 0 | 1 | 1 | 8 | 25 | 2 |
| 15 | 12 | 0 | 0 | 15 | 24 | 17 | 7 | 0 | +1 | 1 | 2 | 8 | 50 | 2 |
| 16 | 13 | +1 | 0 | 16 | 26 | 18 | 8 | +1 | +1 | 1 | 2 | 9 | 75 | 2 |
| 17 | 14 | +2 | 0 | 17 | 28 | 19 | 8 | +2 | +1 | 2 | 2 | 9 | 100 | 2 |
| 18/00-18/09 | 15 | +2 | +1 | 20 | 30 | 20 | 9 | +3 | +2 | 2 | 3 | 10 | 150 | 3 |
| 18/10-18/19 | 16 | +2 | +1 | 30 | 30 | 22 | 10 | +3 | +2 | 2 | 3 | 10 | 175 | 3 |
| 18/20-18/29 | 17 | +3 | +1 | 40 | 35 | 24 | 12 | +3 | +2 | 3 | 3 | 15 | 200 | 3 |
| 18/30-18/39 | 18 | +3 | +1 | 50 | 40 | 26 | 15 | +3 | +2 | 3 | 4 | 15 | 225 | 3 |
| 18/40-18/49 | 19 | +3 | +1 | 60 | 45 | 28 | 20 | +3 | +2 | 4 | 4 | 20 | 250 | 3 |
| 18/50-18/59 | 20 | +3 | +1 | 70 | 50 | 30 | 25 | +4 | +3 | 4 | 5 | 25 | 275 | 4 |
| 18/60-18/69 | 21 | +3 | +1 | 80 | 55 | 30 | 30 | +4 | +3 | 5 | 6 | 30 | 300 | 4 |
| 18/70-18/79 | 22 | +4 | +2 | 90 | 60 | 30 | 35 | +4 | +3 | 5 | 7 | 35 | 350 | 4 |
| 18/80-18/89 | 23 | +5 | +3 | 100 | 65 | 30 | 40 | +4 | +4 | 6 | 8 | 40 | 400 | 4 |
| 18/90-18/99 | 24 | +5 | +4 | 110 | 70 | 30 | 45 | +5 | +5 | 6 | 9 | 45 | 450 | 4 |
| 18/100-18/109 | 25 | +6 | +5 | 120 | 80 | 30 | 50 | +6 | +6 | 7 | 10 | 50 | 500 | 5 |
| 18/110-18/119 | 26 | +7 | +6 | 130 | 80 | 30 | 55 | +7 | +7 | 7 | 10 | 60 | 550 | 5 |
| 18/120-18/129 | 27 | +8 | +7 | 140 | 80 | 30 | 60 | +8 | +8 | 8 | 11 | 70 | 600 | 6 |
| 18/130-18/139 | 28 | +9 | +8 | 150 | 80 | 30 | 65 | +9 | +9 | 8 | 12 | 80 | 650 | 7 |
| 18/140-18/149 | 29 | +10 | +9 | 160 | 80 | 30 | 70 | +9 | +9 | 8 | 13 | 90 | 700 | 7 |
| 18/150-18/159 | 30 | +11 | +10 | 170 | 90 | 30 | 75 | +10 | +10 | 9 | 14 | 100 | 750 | 8 |
| 18/160-18/169 | 31 | +12 | +11 | 180 | 90 | 30 | 80 | +11 | +11 | 9 | 15 | 100 | 800 | 8 |
| 18/170-18/179 | 32 | +13 | +12 | 190 | 90 | 30 | 85 | +12 | +12 | 9 | 16 | 100 | 900 | 9 |
| 18/180-18/189 | 33 | +14 | +13 | 200 | 90 | 30 | 90 | +13 | +13 | 10 | 17 | 100 | 1000 | 9 |
| 18/190-18/199 | 34 | +15 | +14 | 210 | 90 | 30 | 95 | +14 | +14 | 10 | 18 | 100 | 1100 | 9 |
| 18/200-18/209 | 35 | +16 | +15 | 220 | 100 | 30 | 100 | +15 | +15 | 11 | 19 | 100 | 1250 | 10 |
| 18/210-18/219 | 36 | +18 | +15 | 230 | 100 | 30 | 100 | +15 | +15 | 11 | 19 | 100 | 1250 | 10 |
| 18/220+ | 37 | +20 | +15 | 240 | 100 | 30 | 100 | +15 | +15 | 11 | 19 | 100 | 1250 | 10 |


### 7.3 The mental stat tables

"Spells /100 lv" is `adj_mag_study`, "Mana /100 lv" is `adj_mag_mana`
(§12.1); "Min fail" and "Fail reduction" are `adj_mag_fail` and
`adj_mag_stat` from `src/player-spell.c` (§12.3).

| Stat | Index | INT device | INT disarm magic | WIS save | Spells /100 lv (study) | Mana /100 lv | Min fail % | Fail reduction |
|---|---|---|---|---|---|---|---|---|
| 3 | 0 | 0 | 0 | 0 | 0 | 0 | 99 | -5 |
| 4 | 1 | 0 | 0 | 0 | 0 | 10 | 99 | -4 |
| 5 | 2 | 0 | 0 | 0 | 10 | 20 | 99 | -3 |
| 6 | 3 | 0 | 0 | 0 | 20 | 30 | 99 | -3 |
| 7 | 4 | 0 | 0 | 0 | 30 | 40 | 99 | -2 |
| 8 | 5 | +1 | +1 | +1 | 40 | 50 | 50 | -1 |
| 9 | 6 | +1 | +1 | +1 | 50 | 60 | 30 | 0 |
| 10 | 7 | +1 | +1 | +1 | 60 | 70 | 20 | 0 |
| 11 | 8 | +1 | +1 | +1 | 70 | 80 | 15 | 0 |
| 12 | 9 | +1 | +1 | +1 | 80 | 90 | 12 | 0 |
| 13 | 10 | +1 | +1 | +1 | 85 | 100 | 11 | 0 |
| 14 | 11 | +1 | +1 | +1 | 90 | 110 | 10 | 1 |
| 15 | 12 | +2 | +2 | +2 | 95 | 120 | 9 | 2 |
| 16 | 13 | +2 | +2 | +2 | 100 | 130 | 8 | 3 |
| 17 | 14 | +2 | +2 | +2 | 105 | 140 | 7 | 4 |
| 18/00-18/09 | 15 | +3 | +3 | +3 | 110 | 150 | 6 | 5 |
| 18/10-18/19 | 16 | +3 | +3 | +3 | 115 | 160 | 6 | 6 |
| 18/20-18/29 | 17 | +3 | +3 | +3 | 120 | 170 | 5 | 7 |
| 18/30-18/39 | 18 | +3 | +4 | +3 | 130 | 180 | 5 | 8 |
| 18/40-18/49 | 19 | +3 | +4 | +3 | 140 | 190 | 5 | 9 |
| 18/50-18/59 | 20 | +4 | +5 | +4 | 150 | 200 | 4 | 10 |
| 18/60-18/69 | 21 | +4 | +6 | +4 | 160 | 225 | 4 | 11 |
| 18/70-18/79 | 22 | +5 | +7 | +5 | 170 | 250 | 4 | 12 |
| 18/80-18/89 | 23 | +5 | +8 | +5 | 180 | 300 | 4 | 15 |
| 18/90-18/99 | 24 | +6 | +9 | +6 | 190 | 350 | 3 | 18 |
| 18/100-18/109 | 25 | +6 | +10 | +7 | 200 | 400 | 3 | 21 |
| 18/110-18/119 | 26 | +7 | +10 | +8 | 210 | 450 | 2 | 24 |
| 18/120-18/129 | 27 | +7 | +11 | +9 | 220 | 500 | 2 | 27 |
| 18/130-18/139 | 28 | +8 | +12 | +10 | 230 | 550 | 2 | 30 |
| 18/140-18/149 | 29 | +8 | +13 | +11 | 240 | 600 | 2 | 33 |
| 18/150-18/159 | 30 | +9 | +14 | +12 | 250 | 650 | 1 | 36 |
| 18/160-18/169 | 31 | +9 | +15 | +13 | 250 | 700 | 1 | 39 |
| 18/170-18/179 | 32 | +10 | +16 | +14 | 250 | 750 | 1 | 42 |
| 18/180-18/189 | 33 | +10 | +17 | +15 | 250 | 800 | 1 | 45 |
| 18/190-18/199 | 34 | +11 | +18 | +16 | 250 | 800 | 1 | 48 |
| 18/200-18/209 | 35 | +11 | +19 | +17 | 250 | 800 | 0 | 51 |
| 18/210-18/219 | 36 | +12 | +19 | +18 | 250 | 800 | 0 | 54 |
| 18/220+ | 37 | +13 | +19 | +19 | 250 | 800 | 0 | 57 |

### 7.4 What each stat does, in practice

**Strength**
* To-dam on every melee blow (`adj_str_td`, +6 at 18/100, +20 at 18/220) and a
  little to-hit (`adj_str_th`).
* Blows with heavier weapons (`adj_str_blow` × class multiplier / weight).
* Weapon and launcher "hold" limit (`adj_str_hold`).
* Throwing range.
* Digging (`adj_str_dig`).
* **Carrying capacity.** `calc_bonuses()`: with weight limit
  `i = adj_str_wgt × 100` (tenths of lb), if total carried weight `j > i/2`
  then `speed −= (j − i/2) / (i/10)`. Examples:

  | STR | adj_str_wgt | Free weight | Speed loss |
  |---|---|---|---|
  | 10 | 12 | 60 lb | −1 per 12 lb beyond |
  | 16 | 18 | 90 lb | −1 per 18 lb |
  | 18 | 20 | 100 lb | −1 per 20 lb |
  | 18/50 and above | 30 | 150 lb | −1 per 30 lb |

  (The "Burden" figure on the character sheet comes from
  `weight_remaining()`, which uses a 60% threshold for display; the actual
  speed penalty starts at 50%.) Being even −1 speed from burden is a real
  cost (§2): drop the junk.

**Dexterity**
* Blows (the DEX index of the blows table — usually the binding constraint).
* To-hit (`adj_dex_th`) and AC (`adj_dex_ta`).
* Physical disarming (`adj_dex_dis`).
* Protection against theft (`adj_dex_safe` + level).
* Shield-bash stumbling.

**Constitution**
* Hit points (`adj_con_mhp`, retroactive across all levels).
* Faster recovery from cuts, stun and poison (`adj_con_fix`).

**Intelligence / Wisdom**
* The casting stat sets spells known, mana, minimum fail and fail reduction
  (§12).
* INT also adds to device skill (`adj_int_dev`) and magical disarming
  (`adj_int_dis`) for *every* class.
* WIS adds to the saving throw (`adj_wis_sav`) for every class.

### 7.5 Raising stats

The only permanent stat increases are from potions and similar consumables
(`GAIN_STAT` effect → `src/player.c:player_stat_inc()`):

```
if cur >= 18/100: no effect (natural cap 18/100)
if cur <  18:     +1
if cur <  18/90:  gain = ((18/100 - cur)/2 + 3)/2 ;  cur += randint1(gain) + gain/2  (max 18/99)
else:             cur = 18/100
```

So from 18 a potion gives +27..+39 points (i.e. 18/27–18/39); from 18/50 it
gives +8..+21; from 18/80, +4..+9; at 18/90 or more it sets 18/100. Each such
potion also restores the stat first. The natural cap is **18/100**; race,
class and equipment bonuses push the displayed value beyond that, up to the
table maximum at 18/220.

| Item | Effect (`object.txt`) |
|---|---|
| Potion of Strength / Intelligence / Wisdom / Dexterity / Constitution | restore + gain that stat |
| Potion of Brawn / Intellect / Contemplation / Nimbleness / Toughness | restore + gain that stat, **permanently lose** a random *other* stat (`LOSE_RANDOM_STAT`) |
| Potion of Augmentation | restore + gain all five |
| Draught of the Ents | restore + gain STR and CON (among other effects) |
| Potion of Experience | +100,000 exp (dice 200000, halved by `GAIN_EXP`) |

### 7.6 Losing stats: drain, sustain and nexus scramble

* **Drain** (`DRAIN_STAT`, monster LOSE_xxx blows, traps, Black Breath):
  `player_stat_dec(…, false)` lowers only the *current* value — by 10 points
  above 18/10, to 18 between 18/01 and 18/10, by 1 below 18. Drained stats
  come back with a restore effect (Potion of Vigor, Life, Miruvor/Purging for
  STR+CON, the matching gain potion), **and automatically whenever you gain a
  level** (`adjust_level()`). A **sustain** (`SUST_STR` etc.) blocks the
  drain entirely.
* **Permanent loss** (`LOSE_RANDOM_STAT`, e.g. the side effect of a Potion of
  Brawn) lowers the maximum too, and is *not* blocked by sustains.
* **Nexus scramble** (`project_player_handler_NEXUS`): unless you resist nexus,
  a saving throw is rolled; on failure you get `TMD_SCRAMBLE` for 20–39
  turns, which shuffles your stats and **un-shuffles them when it expires**
  (`on-end-effect:UNSCRAMBLE_STATS` in `player_timed.txt`). In 4.2 nexus does
  not permanently swap stats. Nexus also teleports you (1/3 to the monster,
  1/4 of the rest a level teleport that the saving throw can avoid, otherwise
  a 200-grid teleport).

---

## 8. Skills and the saving throw

### 8.1 How skills are computed

`calc_bonuses()`:

```
skill = race skill + class base
      + equipment (STEALTH modifier; SEARCH modifier ×5; TUNNEL/DIG ×20; weapon weight/10 to digging)
      + stat adjustments (DEX→disarm phys, INT→disarm magic and device, WIS→save, STR→digging)
      + class increment * level / 10
      + temporary effects
stealth is clamped to 0..30; digging is at least 1
```

The second number on each `skill-*` line of `class.txt` is the gain **per 10
levels** (exactly `x_skills × lev / 10`).

### 8.2 Class skill table

Base at level 1 / increment per 10 levels / class value at level 50:

| Class | Disarm phys | Disarm magic | Device | Save | Stealth | Search | Melee | Bows | Throw |
|---|---|---|---|---|---|---|---|---|---|
| Warrior | 25/15/100 | 20/10/70 | 18/7/53 | 18/10/68 | 0/0/0 | 10/12/70 | 70/45/295 | 55/45/280 | 55/45/280 |
| Mage | 30/10/80 | 35/12/95 | 36/13/101 | 30/9/75 | 2/0/2 | 10/12/70 | 35/15/110 | 20/15/95 | 20/15/95 |
| Druid | 30/10/80 | 30/10/80 | 24/10/74 | 30/10/80 | 3/0/3 | 12/12/72 | 45/20/145 | 40/30/190 | 40/30/190 |
| Priest | 25/12/85 | 25/12/85 | 30/10/80 | 32/12/92 | 2/0/2 | 10/14/80 | 45/20/145 | 35/20/135 | 35/20/135 |
| Necromancer | 30/10/80 | 35/12/95 | 36/13/101 | 30/9/75 | 2/0/2 | 10/12/70 | 35/25/160 | 20/15/95 | 20/15/95 |
| Paladin | 20/15/95 | 20/10/70 | 24/10/74 | 25/11/80 | 0/0/0 | 10/12/70 | 65/40/265 | 50/30/200 | 50/30/200 |
| Rogue | 45/20/145 | 45/20/145 | 32/10/82 | 28/10/78 | 3/1/8 | 20/16/100 | 35/45/260 | 66/30/216 | 72/45/297 |
| Ranger | 40/15/115 | 30/10/80 | 28/10/78 | 32/10/82 | 3/0/3 | 15/15/90 | 60/40/260 | 72/45/297 | 66/30/216 |
| Blackguard | 20/15/95 | 20/10/70 | 24/10/74 | 18/10/68 | −1/−1/−6 | 8/10/58 | 65/40/265 | 35/15/110 | 40/30/190 |

No class has a digging skill; digging comes from race (Dwarf +40), STR and
tools.

### 8.3 Race skill table

| Race | Dis-P | Dis-M | Device | Save | Stealth | Search | Melee | Bows | Throw | Dig | Infra | Exp % |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 100 |
| Half-Elf | 2 | 2 | 3 | 3 | 1 | 3 | −1 | 5 | 5 | 0 | 2 | 120 |
| Elf | 5 | 5 | 6 | 6 | 2 | 6 | −5 | 15 | 15 | 0 | 3 | 120 |
| Hobbit | 15 | 15 | 18 | 18 | 4 | 6 | −10 | 20 | 20 | 0 | 4 | 120 |
| Gnome | 10 | 10 | 22 | 12 | 3 | 4 | −8 | 12 | 12 | 0 | 4 | 120 |
| Dwarf | 2 | 2 | 9 | 9 | −1 | 2 | 15 | 0 | 0 | 40 | 5 | 120 |
| Half-Orc | −3 | −3 | −3 | −3 | −1 | −3 | 12 | −5 | −5 | 0 | 3 | 120 |
| Half-Troll | −5 | −5 | −8 | −8 | −2 | −9 | 20 | −10 | −10 | 0 | 3 | 120 |
| Dúnadan | 4 | 4 | 5 | 5 | 1 | 3 | 15 | 10 | 10 | 0 | 0 | 120 |
| High-Elf | 4 | 4 | 20 | 20 | 2 | 10 | 10 | 25 | 25 | 0 | 4 | 145 |
| Kobold | 10 | 10 | 5 | 0 | 3 | 10 | −5 | 10 | 10 | 0 | 5 | 120 |

Example: a level-30 Hobbit Rogue has stealth 4 + 3 + 1×30/10 = **10** before
equipment; a level-30 Half-Troll Warrior has stealth 0 (−2 clamped to 0).

### 8.4 Temporary skill modifiers

From `calc_bonuses()` (`adjust_skill_scale()` scales by a fraction):

| Condition | Effect |
|---|---|
| Hunger (food below 1500) | `lack = (1500 − food) × 20 / 1500`: to-hit and to-dam −lack (up to −20); device −10% once food ≤ 675, −20% and disarm ×0.9 at ≤ 300, −30%, disarm ×0.8, save and search ×0.9 at ≤ 75 |
| Stun | to-hit/to-dam −5, device −10% |
| Heavy Stun | to-hit/to-dam −20, device −20% |
| Confused | device −25% |
| Amnesia, Hallucination | device −20% each |
| Poisoned | device −5% |
| Fear | to-hit −20, device −5% |
| Berserk | melee skill +75, device −10% |
| Bless / Heroism | device +5% each |
| `TMD_STEALTH` | stealth +10 |

### 8.5 The saving throw

The save is a single roll, `randint0(100) < skill_save`. A skill of 100 or
more saves every time. It is used in these places (grep for `SKILL_SAVE`):

| Threat | Code |
|---|---|
| Monster spells with a `message-save`: MIND_BLAST, BRAIN_SMASH, WOUND (cause wounds), SCARE, BLIND, CONF, SLOW, HOLD, TELE_LEVEL, FORGET | `src/mon-spell.c:do_mon_spell()` |
| Monster TERRIFY and PARALYZE melee blows (but **not** BLIND or CONFUSE blows) | `src/mon-blows.c:melee_effect_timed()` |
| Nexus stat scramble and nexus level-teleport | `src/project-player.c` |
| Traps flagged `SAVE_THROW` | `src/trap.c` |

Breaths, bolts and balls get no save: only resistances help. Many saved-against
effects are also stopped by a protection flag (`pConf`, `pBlind`, `pFear`,
Free Action), which is absolute where the save is probabilistic — e.g. the
paralysis touch is blocked by `FREE_ACT` regardless of the save.

Typical saves: a level-50 Priest has 92 (class) + race + WIS bonus (18/100 →
+7) = 99+; a level-50 Half-Troll Warrior has 68 − 8 + ~1 ≈ 61.

---

## 9. Stealth, noise, scent and monster waking

### 9.1 Sleep and waking

A monster is generated asleep with `sleep counter = 2 × race sleep +
randint1(10 × race sleep)` (`src/mon-make.c`). Each turn a sleeping monster
gets, `src/mon-move.c:monster_reduce_sleep()` rolls:

```
player_noise = 2^(30 - stealth)
notice       = randint0(1024)
if notice^3 <= player_noise:
    reduction = 1, or 100 / local_noise if 0 < local_noise < 50
    sleep counter -= reduction            (monster wakes when it reaches 0)
```

`local_noise` is the monster's flow distance to you (§9.2). So the chance per
monster turn that your noise "registers" is about `2^(−stealth/3)`:

| Stealth | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 12 | 14 | 16 | 18 | 20 | 25 | 30 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Chance % | 100 | 79 | 63 | 50 | 40 | 32 | 25 | 20 | 16 | 13 | 10 | 6.3 | 4.0 | 2.5 | 1.7 | 1.1 | 0.4 | 0.2 |

and each registered noise costs the monster `100/distance` points of sleep:
100 when adjacent, 20 at 5 grids, 10 at 10 grids, 2 at 50+ grids (or when
unreachable).

Worked example: a monster with `sleep:30` starts with 60 + 1d300 ≈ 210
sleep. You are 10 grids away. With stealth 6 (25%), you knock off 10 points
per success → ~21 successes → ~84 monster turns. With stealth 12 (6.3%) it
takes ~330 monster turns. Adjacent, a single success removes 100 points —
**stealth matters most at a distance; walking up to a sleeping monster wakes
it fast regardless.**

* `AGGRAVATE` wakes every monster that gets a turn, instantly.
* Damaging, or even attacking, a monster wakes it.
* Group members: a monster that is aware of you rouses sleeping members of its
  group that it can see, each with chance `1 / (20 × distance)` per turn
  (`src/mon-group.c:monster_group_rouse()`).
* Awake monsters that are "active" become fully aware with 1/10 chance per turn.

### 9.2 Noise, hearing and how monsters find you

Every player turn (except while resting) `src/game-world.c:make_noise()`
floods outward from you through every grid that is not `NO_FLOW` (granite,
magma, quartz, rubble and permanent walls are `NO_FLOW`; **closed doors are
not**), assigning each grid its walking distance. Monsters use this map to
path toward you around corners.

A monster is **active** (`monster_check_active()`) if any of:

1. it can pass/kill walls and is within its `hearing` range,
2. it is hurt,
3. it is in your view,
4. it can *hear* you: `race hearing − stealth/3 > noise at its grid`,
5. it can *smell* you: `race smell > scent age at its grid`,
6. it is taking terrain damage.

Otherwise it is passive and does nothing. Most monsters have hearing 20; some
have much more (dragons and hounds often 30–100). Your stealth reduces their
effective hearing by stealth/3 grids — at stealth 15, a hearing-20 monster
must be within 14 walking steps to notice you.

**Scent** (`update_scent()`): you lay down scent in a 5×5 square around you
each turn; old scent ages by 1 per turn. Monsters with `smell` follow scent
trails younger than their smell value — this is how hounds and other
smellers home in on where you *were*, even after you teleport.

`TMD_COVERTRACKS` ("cover tracks") quadruples noise falloff, removes your
scent, and hides you from monsters more than `max_sight/4` = 5 grids away.

### 9.3 Practical stealth

* Stealth determines *how long sleeping monsters stay asleep* and *how far
  awake monsters can hear you*. It does nothing once a monster has seen you.
* Low-stealth races/classes (Half-Troll, Blackguard, Warrior) should expect
  every level to wake up; plan to fight, or to dive quickly.
* Resting does not update the noise map, so you do not "broadcast" while
  resting — but already-hunting monsters keep coming.

---

## 10. Light, vision, invisibility, telepathy and detection

### 10.1 Light radius

`calc_light()` sums the light of every wielded item: `LIGHT_2` gives
intensity 2, `LIGHT_3` gives 3, plus any `LIGHT` modifier; a fuelled light
with zero fuel gives 0. `calc_lighting()` then lights every grid within
**radius = intensity − 1** with brightness `intensity − distance`.

| Source (`object.txt`) | Intensity | Lit radius |
|---|---|---|
| Wooden Torch (`LIGHT_2`, burns out) | 2 | 1 (adjacent grids) |
| Lantern (`LIGHT_3`, refuel with oil) | 3 | 2 |
| Light-boosting egos/artifacts | +1 each `LIGHT` | +1 |

Torches burn 5000 turns (`fuel-torch`), lanterns hold 15000 and come with
7500 (`fuel-lamp`, `default-lamp`); a flask of oil adds 7500.

Unlit grids are visible only if they are in a lit room or you have light on
them; monsters in unlit grids beyond your light are seen only by
infravision, telepathy, or detection.

### 10.2 UNLIGHT (Necromancers)

Necromancers (`UNLIGHT`) are designed to work in the dark
(`src/cave-view.c`, `src/player-spell.c`, `src/player-calcs.c`):

* With light intensity ≤ 1 they see nearby grids within
  `2 + level/6 − light` grids as if lit.
* +25 to spell failure when standing on a lit square (before the 50% cap,
  §12.3).
* Light gear with `+1 LIGHT` gives them 1 less, so a +1 item is neutral.
* Resist darkness once in the dungeon.

### 10.3 Infravision

`update_mon()` (`src/mon-util.c`): a monster in line of sight within
`see_infra` grids (race infravision + `INFRA` modifiers, +5 from
`TMD_SINFRA`) is seen even in the dark **unless it is `COLD_BLOOD`**.
The infravision check does not look at invisibility at all, so it also
reveals *warm-blooded invisible* monsters within range (most invisible
monsters — ghosts and other undead — are cold-blooded, though). Ranges: Dwarf 5,
Kobold 5, Hobbit/Gnome/High-Elf 4, Elf/Half-Orc/Half-Troll 3, Half-Elf 2,
Human/Dúnadan 0.

### 10.4 Invisibility

An `INVISIBLE` monster in a lit, visible grid is seen only with `SEE_INVIS`
(High-Elves have it innately). Without it you fight at **half to-hit** (§3.1,
§4.2) and cannot target it normally.

### 10.5 Telepathy

`TELEPATHY` shows every monster within **`max_sight` = 20 grids** (the
approximate distance `dy + dx/2`), through walls, except:

* `EMPTY_MIND` monsters — never;
* `WEIRD_MIND` monsters — only one individual in ten (those with
  `midx % 10 == 5`);
* grids flagged `NO_ESP` (vault interiors etc.).

So ESP is a 20-grid radar, not a level-wide one.

### 10.6 Detection

Detection effects use a rectangle `effect-yx:Y:X` centred on you: almost all
detections (58 entries across `class.txt`, `object.txt`, `activation.txt`)
use **22 × 40**, i.e. y ± 22 and x ± 40 — roughly a screen. The dungeon is
66 × 198, so one detection covers about 40% of a level's width. A few use
11×11, 33×55, 44×60, or the whole level (500×500).

---

## 11. Monster AI basics

### 11.1 Turn structure

`src/mon-move.c:monster_turn()` does, in order:

1. Deal with webs.
2. Rouse group members (`monster_group_rouse()`).
3. Try to multiply (breeders).
4. **Try a ranged attack** (`make_ranged_attack()`) — *before* moving, even
   when adjacent.
5. Otherwise pick a move (`get_move()`): advance by sight, sound or scent,
   follow a group-mate who is tracking you, flee, hide, surround, or wander.
6. Moving into you = melee (`make_attack_normal()`); into another monster =
   push past or kill (if `MOVE_BODY`/`KILL_BODY`).
7. If it could not move and is afraid, the fear turns into **hold** (the
   monster cowers, paralysed).

### 11.2 Spell frequency

`spell-freq:N` in `monster.txt` → `freq_spell = 100/N` (similarly
`innate-freq` for breaths, arrows, etc.). The stored value is a percentage;
if a monster has spells but no frequency line, `mon-init.c` sets it to 4%. Each turn (`monster_can_cast()`):

```
chance = freq (innate or spell)
chance /= 2   if you are Taunting (TMD_TAUNT)
chance *= 2   if the monster is at its preferred range (best_range)
cast if randint0(100) < chance, target within 20 grids and projectable
```

* A `spell-freq:3` monster casts on ~33% of its turns, ~66% when at its
  preferred distance.
* Breathers (`innate freq > 24`) like point-blank range while above half HP;
  other casters with `spell freq > 24` prefer to stand 3 grids further back;
  archers +3 grids.
* Non-stupid monsters filter their spell list (`remove_bad_spells()`): no
  heal at full HP, no haste if already hasted, no teleport-to when adjacent,
  no bolts if another monster is in the way, no summons if there is no room.
  With `birth_ai_learn` they also learn your resistances and protections
  when they see an attack fail (`update_smart_learn()`: smart monsters
  always, others 50%), and avoid useless spells.
* Smart monsters below 10% HP: 50% chance to use only non-damaging
  (escape/heal/summon) spells.

Spell failure (`monster_spell_failrate()`): innate attacks never fail. For
other spells, non-stupid monsters fail
`25 − (power + 3)/4` percent, where `power = MIN(spell_power, 1)` — as
written in 4.2.6 this caps power at 1, so **non-innate spells fail 24% of the
time** (25% for spell power 0) for all non-stupid monsters, +20 if afraid,
+50 if confused or disenchanted. Stupid monsters never fail.

Monster bolt-type spells (ARROW, SHOT, BOLT `hit:50`, BOULDER `hit:60`) must
pass `check_hit(level × 3 + hit)` against your AC — another reason AC matters.

### 11.3 Fear and morale

`get_move_find_range()` decides whether a monster wants to fight:

* Afraid or `FRIGHTENED` monsters flee (up to `max_sight + flee-range` = 25
  grids).
* A non-afraid monster flees anyway if
  `monster level + 25 (+8 for some individuals) + 3 < player level`, i.e. only
  trivial monsters run from high-level characters; and in close cases it
  compares `level × maxHP + 4 × curHP` ratios.
* Monsters within `turn-range` = 5 grids do not run away by this logic.

Becoming afraid from damage (`src/mon-util.c:monster_scared_by_damage()`):

* If already afraid, damage `dam` cures `randint1(dam)` points of fear (a big
  hit can snap it out of fear completely).
* Otherwise, if it *can* be scared (not `NO_FEAR`; bodyguards never; servants
  2/3 of the time; group members resist with 1/20 per other member), it
  panics if `randint1(10) ≥ remaining HP%` (only below 10% HP) or with 80%
  chance when a single hit took at least half its remaining HP. Duration is
  `1d10 + 20` or `1d10 + (11 − HP%) × 5`.
* Recovery: each monster turn fear drops by `randint1(level/10 + 1)`.

### 11.4 Groups and pack AI

Monsters generated together share a group with roles (leader, servant,
bodyguard, member, summon — `src/mon-group.c`). Bodyguards never flee.
Monsters with `GROUP_AI` (hounds, wolves, many packs):

* **will not follow you into corridors** while you are healthy (fewer than 5
  open grids around you and your HP > 50%): they find a hiding place out of
  sight and wait in ambush;
* in the open, try to **surround** you by moving to empty grids adjacent to you.

### 11.5 Monster HP and regeneration

* Uniques have exactly their listed `hit-points`. Others roll a normal
  distribution with mean `avg_hp` and standard deviation ≈ `avg_hp/8 + 1`
  (`src/mon-make.c:mon_hp()`).
* Every 100 game turns (10 normal-speed player turns) each monster regains
  `maxhp/100` (min 1), doubled for `REGENERATE` (`regen_monster()`).

### 11.6 Monster status effects

Monsters resist your stun/confuse/slow/sleep/fear/hold effects with
`resist_chance = min(90, monster level + max(0, 25 − timer/2))`, and
**uniques roll twice** (`src/mon-timed.c`). A stunned monster misses 1 turn in
10 and has −25% to-hit and damage; a held monster skips its turns; a
confused monster moves erratically 30% of the time per level of confusion
(compounding: 30%, 51%, 66%…, `monster_turn_should_stagger()`), has +50%
spell failure and −20% spell accuracy (`CONF_HIT_REDUCTION`). `RAND_25` /
`RAND_50` monsters move erratically 25%/50% of the time anyway.

---

## 12. Spellcasting: mana, spell count and failure

### 12.1 Mana

`src/player-calcs.c:calc_mana()`:

```
levels = player level - first spell level + 1
msp    = 1 + adj_mag_mana[spell stat] * levels / 100
msp   -= (armour weight - class spell_weight) / 10      (if positive; tenths of lb)
```

* `spell_weight` (`magic:` line in `class.txt`): Mage/Necromancer 30 lb,
  Priest/Druid/Rogue 35 lb, Paladin/Ranger 40 lb, Blackguard 60 lb.
  Armour weight counts everything except weapon, bow, rings, amulet and light.
  **Each pound over the limit costs 1 SP.** There is no glove penalty in 4.2.
* Example: a level-30 Mage with INT 18/100 (`adj_mag_mana` 400):
  1 + 400 × 30/100 = **121 SP**; wearing 40 lb of armour drops it to 111.
* First spell level: 1 for most casters, 3 for Rangers, 5 for Rogues.

### 12.2 Number of spells

`calc_spells()`: `allowed = (adj_mag_study[stat] × levels + 50) / 100`,
limited by the spells your level allows. At INT/WIS 18/100 that is 2 spells
per level, far more than needed; at 14 it is 0.9 per level, which can leave
a low-stat caster short of spells.

### 12.3 Spell failure

`src/player-spell.c:spell_chance()`:

```
chance  = spell base fail (spell:name:level:mana:FAIL:exp)
chance -= 3 * (player level - spell level)
chance -= adj_mag_stat[spell stat]                     (−5 .. +57, table in §7.3)
chance += 5 * (mana cost - current SP)                 (if casting without enough mana)
minfail = adj_mag_fail[spell stat]                      (99 .. 0, table in §7.3)
minfail = max(minfail, 5)  unless class has ZERO_FAIL   (Mage, Druid, Priest, Necromancer)
chance += 25  if UNLIGHT and standing on a lit square
chance += 20  if afraid
chance  = clamp(chance, minfail, 50)
chance += 15 if stunned, 25 if heavily stunned (stun > 50)     (after the clamp)
chance  = 50 + chance/2  if amnesiac
chance  = min(chance, 95)
```

Key points:

* **The 50% cap**: a spell can never be worse than 50% fail (before stun and
  amnesia), however far above your level it is.
* **The minimum fail** depends only on your casting stat: 7% at 17, 6% at 18,
  5% at 18/20, 4% at 18/50, 3% at 18/90, 2% at 18/110, 1% at 18/150, **0% at
  18/200**. Paladins, Rogues, Rangers and Blackguards (no `ZERO_FAIL`) can
  never go below 5%.
* Each level above the spell's level is −3%.
* Stun is applied *after* the clamp, so even 0%-fail spells become 15%/25%
  when stunned — cure stun before relying on spells.

Worked example (Mage, Magic Missile: level 1, base fail 22):

| Mage level | INT | adj_mag_stat | minfail | Fail |
|---|---|---|---|---|
| 1 | 18/10 | 6 | 6 | 22 − 0 − 6 = **16%** |
| 5 | 18/10 | 6 | 6 | 22 − 12 − 6 = 4 → **6%** (min) |
| 20 | 18/100 | 21 | 3 | **3%** (min) |
| 40 | 18/200 | 51 | 0 | **0%** |

Frost Bolt (level 13, base 40) at level 13 with INT 18/50: 40 − 0 − 10 =
30%; at level 20: 40 − 21 − 10 = 9%.

Casting takes 100 energy (75 with `TMD_FASTCAST`, §2.4). Casting without
enough mana is allowed after a prompt and can damage your CON and paralyse
you — avoid it.

---

## 13. Magic devices: failure rates

`src/obj-util.c:get_use_device_chance()` (rewritten in 4.2.3; constant
changed in 4.2.6):

```
lev  = artifact level, or activation level, or object kind level
x    = 2 * (device skill - lev) + 1
fail = 380 - 370 * x / (5 + |x|)          (out of 1000; C integer division)
use fails if randint1(1000) < fail
```

It is a sigmoid in `skill − level`, ranging from 75% (hopeless) to 1%
(trivial):

| skill − level | −20 | −10 | −5 | −3 | −2 | −1 | 0 | +1 | +2 | +3 | +5 | +7 | +10 | +15 | +20 | +30 | +40 | +60 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Fail % | 70.7 | 67.2 | 61.7 | 56.5 | 51.8 | 44.1 | 31.9 | 24.2 | 19.5 | 16.5 | 12.6 | 10.3 | 8.2 | 6.2 | 5.1 | 3.9 | 3.2 | 2.5 |

The changes note for 4.2.6 (`changes.txt`) says the constant was changed "to
tighten the transition from high to low failure rates: effects that change a
player's device skill will have more of an impact if the skill is close to
the activation difficulty." In practice: **the steep part of the curve is
±5 around the device level**. A few points of device skill (INT, Bless,
Heroism) matter enormously for devices near your skill, and almost nothing
for devices far above or below it.

Also new in 4.2.6: a failed activation **automatically retries** (like
tunnelling) until it succeeds or you are disturbed, so a moderate fail rate
mostly costs time, not a wasted keypress — but each attempt still costs a
turn in danger.

Device skill = race + class base + class increment × level/10 + `adj_int_dev`
(+0 at INT 7, +1 at 8–14, +2 at 15–17, +3 at 18–18/49, +6 at 18/100, +13 at
18/220) ± temporary modifiers (§8.4: confusion −25%, stun −10/−20%, etc.).

Worked examples (device levels from `object.txt`):

| Character (level 30) | Device skill | Staff of Teleportation (20) | Rod of Recall (30) | Wand of Teleport Other (20) | Rod of Healing (80) |
|---|---|---|---|---|---|
| Human Warrior, INT 10 | 18+21+1 = 40 | 5.1% | 8.2% | 5.1% | 72.7% |
| Human Mage, INT 18/100 | 36+39+6 = 81 | 2.5% | 2.8% | 2.5% | 24.2% |
| High-Elf Mage, INT 18/100 | 81+20 = 101 | 2.2% | 2.3% | 2.2% | 4.9% |

The ×10 difference between warrior and mage on
endgame devices is the real reason "devices are a caster's thing".

---

## 14. Regeneration, food and digging

### 14.1 Hit point and mana regeneration

`src/player-util.c:player_regen_hp()` runs once per 10 game turns (§2.1):

```
percent = 197    if food >= Weak threshold        (PY_REGEN_NORMAL, units of 1/65536)
          98     if Weak                           (PY_REGEN_WEAK)
          33     if Faint                          (PY_REGEN_FAINT)
          0      if Starving
percent = percent * (100 + (food/100)/3) / 100     (well fed: up to +33%)
percent *= 2     with REGEN
percent *= 2     after resting 5+ turns (REST_REQUIRED_FOR_REGEN)
percent /= 2     with IMPAIR_HP
percent  = 0     if paralysed, poisoned, stunned or cut
gain     = mhp * percent + 1442                    (in 1/65536 HP)
```

So a normally-fed character regains about **0.3% of max HP per tick**
(0.35% when very well fed), ×2 resting, ×2 more with Regeneration. Example:
500 max HP, food 5000 → percent 228 → 1.74 HP/tick; resting 3.5/tick;
resting with Regen 7/tick (full heal from 0 in ~70 normal-speed turns).

**Poison, stun and cuts stop regeneration entirely** — cure them first
(CON speeds their natural recovery, §7.2).

Mana (`player_regen_mana()`) uses the same 197 base percent of max SP plus a
small constant, doubled by Regen and by resting. Blackguards
(`COMBAT_REGEN`) are the exception: their SP slowly *drains* over time, each
point lost being converted into HP at double rate, and they gain SP whenever
they make a melee attack (5% of max SP, `py_attack()`).

### 14.2 Food

`TMD_FOOD` grades (`player_timed.txt`, percentages ×`player:food-value:100`):

| Grade | Food value | Effect |
|---|---|---|
| Starving | 0–100 | damage each tick `(100 − food)/10`; no regeneration |
| Faint | 101–400 | 1-in-10 per tick to faint (paralysed 1–5 turns, bypassing Free Action); regen 33/65536 |
| Weak | 401–800 | regen halved (98) |
| Hungry | 801–1500 | to-hit/to-dam penalties begin (§8.4) |
| Fed | 1501–9000 | normal |
| Full | 9001–10000 | **slow**: speed −(food − 9000) × 10 / 1000 (up to −10); digests 50 per tick |

Digestion (`process_world()`): every 100 game turns you lose
`turn_energy(speed) × 100 / food_value` food, ×2 with Regeneration, ÷2 with
Slow Digestion, minimum 1. At normal speed this is 10 per 10 player turns =
**1 per player turn**, and since faster characters burn more per game turn,
it is ~1 per player action at any speed. Going from Fed (9000) to Hungry
(1500) therefore takes ~7500 player turns.

Practical: don't eat to "Full" before a fight — the overeating penalty is
real speed loss.

### 14.3 Digging

`calc_digging_chances()` gives chances out of 1600 per turn; `src/cmd-cave.c`
rolls `chance > randint0(1600)`:

| Terrain | Chance per turn (/1600) |
|---|---|
| Rubble | 8 × skill |
| Magma vein | 4 × (skill − 10) |
| Quartz vein | 2 × (skill − 20) |
| Granite | 1 × (skill − 40) |
| (Door, via tunnelling) | (4 × skill − 119)/3 |
| Permanent wall | impossible |

Digging skill = race (Dwarf 40) + `adj_str_dig` (0 at STR 3–4 up to 100 at
18/200) + 20 per `DIG_n` level of a digger + 20 per `TUNNEL` modifier + the
wielded weapon's weight in lb. The game automatically swaps in the best
digger from your pack when you tunnel (`player_best_digger()`), counting its
own weight instead of your weapon's.

Diggers (`object.txt`): Shovel `DIG_1` (6 lb), Pick `DIG_2` (18 lb), Mattock
`DIG_3` (25 lb).

| Character | Skill | Rubble | Magma | Quartz | Granite |
|---|---|---|---|---|---|
| Human Mage, STR 10, Shovel | 5 + 20 + 6 = 31 | 15.5% (6.5 turns) | 5.3% (19) | 1.4% (73) | never |
| Human Warrior, STR 18/100, Pick | 50 + 40 + 18 = 108 | 54% (1.9) | 24.5% (4.1) | 11% (9.1) | 4.3% (24) |
| Dwarf Warrior, STR 18/100, Pick | 148 | 74% (1.4) | 34.5% (2.9) | 16% (6.3) | 6.8% (15) |
| Dwarf, STR 18/100, Mattock | 40 + 50 + 60 + 25 = 175 | 87.5% | 41% (2.4) | 19% (5.2) | 8.4% (12) |

Granite needs skill > 40 to be possible at all. Like device use, tunnelling
auto-repeats until success or disturbance.

---

## 15. Resistances and damage reduction (formulas)

The detailed per-element tables belong to the "Resistances and damage"
chapter; here are the mechanics.

### 15.1 Resistance levels

Each element has a `res_level` (`calc_bonuses()`):

| res_level | Meaning |
|---|---|
| −1 | vulnerable (race or item) |
| 0 | none |
| 1 | resist (any item/race/temporary) |
| 2 | **double resist**: permanent + temporary (a timed resist adds +1 only if the level is < 2) |
| 3 | immune |

The best item value is taken (resistances don't stack across items); a
vulnerability lowers the level by one unless you are immune, so vulnerability
+ resist = 0 (normal damage). Temporary resists exist for the five base
elements (acid, electricity, fire, cold, poison) via `TMD_OPP_*`.

### 15.2 Damage formula

`src/project-player.c:adjust_dam()`:

```
if res_level == 3: damage = 0
if element is ACID and minus_ac() picks an armour piece with AC > 0: damage = (damage+1)/2
if res_level == -1: damage = damage * 4/3
else repeat res_level times: damage = damage * numerator / denominator
```

`numerator/denominator` from `lib/gamedata/projection.txt`:

| Elements | Resist factor | Double resist | Breath divisor (HP/x) | Breath cap |
|---|---|---|---|---|
| Acid, Lightning, Fire, Cold | 1/3 | 1/9 | 3 | 1600 |
| Poison | 1/3 | 1/9 | 3 | 800 |
| Light, Dark, Sound, Shards, Disenchant | 6/(8+1d4) ≈ 0.50–0.67 | — | 6 | 500 |
| Nexus | 6/(8+1d4) | — | 6 | 400 |
| Nether, Chaos | 6/(8+1d4) | — | 6 | 600 |
| Gravity, Inertia, Force | no resist | — | 3/6/6 | 200 |
| Time | no resist | — | 3 | 150 |
| Plasma | no resist | — | 6 | 150 |
| Mana | no resist | — | 3 | 1600 |

Breath damage is `monster current HP / divisor`, capped
(`src/mon-spell.c:breath_dam()`): **wounded breathers breathe weaker**.

Notes:

* Ice uses the cold resistance.
* Acid's halving needs only that the randomly chosen armour slot holds armour
  with AC > 0 — even acid-proof armour halves the damage (it just isn't
  damaged).
* Elemental damage also destroys inventory items (`inven_damage()`, strength
  `min(dam × 5, 300)`).
* After resistances, `player_apply_damage_reduction()` subtracts flat
  `DAM_RED` and percentage reduction (§5.3).

---

## 16. Traps, doors and locks

### 16.1 Noticing traps

In 4.2 there is no "search" command: a trap is revealed when its grid comes
into view if your **searching skill ≥ the trap's power** (`visibility:` in
`trap.txt`, rolled per trap; `src/trap.c:square_reveal_trap()`). Some
examples: pits 70–90, trap door `2d10M50`, dart traps 30, gas traps 20+d5,
siren 60+d10, most runes and the acid trap 0 (always visible; the teleport
rune is 30+d30). High searching (Rogues
reach 100) sees nearly everything; low-skill characters walk into the
better-hidden traps. Detection and `TRAPSAFE` (`TMD_TRAPSAFE`) are the
alternatives.

Saves when you trigger one (`hit_trap()`): `SAVE_ARMOR` traps miss if
`check_hit(125)` fails against your AC; `SAVE_THROW` traps are avoided with a
successful saving throw.

### 16.2 Disarming

`src/cmd-cave.c:do_cmd_disarm_aux()`:

```
skill  = disarm-magic for magical (rune) traps, disarm-phys otherwise
skill /= 10 if blind, without light, confused or hallucinating
chance = max(2, skill - depth/5)
roll 1: success with chance%         -> trap removed, exp = 1 + depth/5
roll 2: with chance%                 -> harmless failure, may retry
else                                 -> you set it off
```

So P(setting it off) = (1 − chance)². With 60% skill margin you trigger it
16% of attempts; with 90%, 1%.

### 16.3 Doors and locks

* Closed doors are locked 1 time in 4, with lock power 1–7
  (`src/gen-util.c:place_closed_door()`); random doors are 30% open, 10%
  broken, 60% closed.
* Picking a lock: `chance = max(2, disarm-phys − 4 × lock power)`, with skill
  ÷10 when blind/without light and ÷10 when confused/hallucinating
  (`calc_unlocking_chance()`). It auto-repeats.
* Locking a door yourself: lock power `m_bonus(7, depth)` on success.
* Closed doors still carry sound (they are not `NO_FLOW`), so they don't hide
  you from hearing monsters, but they do block sight and most monsters must
  open or bash them, buying turns.

---

## 17. Practical takeaways

Speed and time
- [ ] **Speed first.** +10 speed doubles your actions against normal monsters;
      +20 triples them. Value a Potion of Speed or a speed item above almost
      any other bonus (§2).
- [ ] Never be burdened: each `adj_str_wgt` pounds over half your limit is −1
      speed (§7.4). Don't eat to "Full" (§14.2).
- [ ] Remember that poison, regeneration and timed buffs tick on *game*
      time: when fast, you get more actions per buff turn.

Melee
- [ ] To-hit is `skill + 3 × to-hit`, rolled against 2/3 of monster AC with
      12% auto-hit and 5% auto-miss (§3.1).
- [ ] For light weapons, **DEX determines blows**; for heavy weapons, STR and
      DEX both do (§3.2). Check the in-game blows calculator when changing
      weapons.
- [ ] Slays/brands multiply only dice; ring and STR to-dam is added flat to
      *every* blow (§3.3). Many blows → stack +dam; big dice → stack slays.
- [ ] Never wield a weapon heavier than `adj_str_hold`: you drop to one blow
      (§3.2).
- [ ] Unseen monsters halve your to-hit — get See Invisible (§10.4).

Missiles
- [ ] Launcher multiplier and slay **add**, and multiply ammo *and* launcher
      to-dam; ring to-dam does nothing for missiles (§4.4).
- [ ] Rangers get +1 shot at level 30; heavy launchers lose extra shots and
      might (§4.3).
- [ ] Flasks of oil and good throwing weapons are excellent early (§4.5).

Defence
- [ ] AC reduces HURT damage by up to 60% (AC 240) and makes power-0 touch
      attacks (paralysis, drains, theft) miss most of the time (§5).
- [ ] Free Action, protection from confusion/blindness/fear are absolute;
      the saving throw is only a percentage (§8.5).
- [ ] Double resist (permanent + potion) cuts base-element damage to 1/9;
      wounded breathers do less damage (§15).
- [ ] CON adds HP retroactively to every level and speeds cut/poison/stun
      recovery (§6.2, §7.2).

Stealth and AI
- [ ] Stealth matters for sleeping monsters at a distance: each 3 points of
      stealth halves the chance your noise registers (§9.1).
- [ ] Monsters track you by sound (walking distance) and by scent (where you
      were); teleporting away leaves a scent trail they will follow (§9.2).
- [ ] Pack animals won't follow you into corridors while you are healthy —
      fight them in rooms on your terms, or ignore them (§11.4).
- [ ] Monsters try ranged attacks before moving, even when adjacent;
      breathers like close range, casters like to stand back (§11.2).
- [ ] Non-innate monster spells fail ~24% of the time; breaths never fail.

Magic
- [ ] Spell fail is capped at 50% and floored by your casting stat (0% only
      at 18/200 for pure casters, 5% for hybrids). Stun adds +15/+25 *after*
      the floor (§12.3).
- [ ] Device failure is a sigmoid around `skill − device level`; within ±5 of
      the level, every point of device skill matters (§13).
- [ ] Each pound of armour over the class limit costs 1 SP (§12.1).

Survival
- [ ] Regeneration stops entirely while poisoned, cut or stunned (§14.1).
- [ ] Resting doubles regeneration after 5 turns; Regeneration doubles it
      again but doubles food use.
- [ ] Gaining a level restores drained stats; nexus stat scrambles wear off
      (§6.5, §7.6).
- [ ] Rogue-level searching spots most traps; otherwise detect traps or
      accept that well-hidden ones will be walked into (§16.1).

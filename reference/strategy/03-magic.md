# Chapter 3 — Spells and Prayers

> Angband 4.2.6 (C). Every spell, number and formula below comes from
> `lib/gamedata/class.txt` (the `magic:`, `book:`, `book-properties:`, `spell:`,
> `effect:`, `dice:` and `expr:` lines), `lib/gamedata/realm.txt`,
> `lib/gamedata/shape.txt`, and the C sources — mainly `src/player-spell.c`
> (learning, failure, casting), `src/player-calcs.c` (mana, spells known),
> `src/effects.c`, `src/effect-handler-attack.c` and `src/effect-handler-general.c`
> (what each effect does), and `src/project-mon.c` (how monsters resist).
> `lib/gamedata/old_class.txt` is not loaded by the game and is ignored.
>
> Race/class basics and play guides are in [Chapter 2](02-races-classes.md).

---

## Contents

1. [Realms, books and commands](#1-realms-books-and-commands)
2. [Casting mechanics](#2-casting-mechanics)
   - 2.1 Learning spells · 2.2 First-cast experience · 2.3 Mana ·
     2.4 Failure rates · 2.5 Mana regeneration · 2.6 Casting without enough mana ·
     2.7 Casting restrictions · 2.8 Beams, ranges and how dice scale
3. [Where the books come from](#3-where-the-books-come-from)
4. [Reading the spell tables](#4-reading-the-spell-tables)
5. [Mage](#5-mage) — arcane
6. [Druid](#6-druid) — nature, and all shapechanges
7. [Priest](#7-priest) — divine
8. [Necromancer](#8-necromancer) — shadow
9. [Paladin](#9-paladin) — divine
10. [Rogue](#10-rogue) — arcane
11. [Ranger](#11-ranger) — nature
12. [Blackguard](#12-blackguard) — shadow
13. [Shapechange reference](#13-shapechange-reference)
14. [Comparative quick reference](#14-comparative-quick-reference)

---

## 1. Realms, books and commands

`lib/gamedata/realm.txt` defines four realms. The realm decides the casting stat and
the words the game uses:

| Realm | Stat | Verb | Spell noun | Book noun | Classes |
|---|---|---|---|---|---|
| arcane | INT | cast | spell | magic book | Mage, Rogue |
| divine | WIS | recite | prayer | prayer book | Priest, Paladin |
| nature | WIS | chant | verse | nature book | Druid, Ranger |
| shadow | INT | perform | ritual | shadow book | Necromancer, Blackguard |

Each class's `magic:` line gives *level of first spell : armour weight limit (tenths of
a pound) : number of books*:

| Class | First spell | Armour limit | Books | Total spells |
|---|---|---|---|---|
| Mage | CL1 | 30.0 lb | 5 | 30 |
| Druid | CL1 | 35.0 lb | 5 | 27 |
| Priest | CL1 | 35.0 lb | 5 | 28 |
| Necromancer | CL1 | 30.0 lb | 5 | 26 |
| Paladin | CL1 | 40.0 lb | 3 | 16 |
| Rogue | CL5 | 35.0 lb | 2 | 10 |
| Ranger | CL3 | 40.0 lb | 2 | 11 |
| Blackguard | CL1 | 60.0 lb | 3 | 15 |
| Warrior | — | — | — | 0 |

Books are shared between classes of the same realm, but **each class has its own
spell list for a shared book**: a Rogue reading [First Spells] sees six rogue
spells (Detect Monsters at CL5, Phase Door at CL7, …), not the Mage's seven, and a
Paladin's [Novice's Handbook] has Bless at CL1 but Call Light at CL5. A book is only
usable by your class if it appears in your class's `book:` list.

Commands (`src/ui-game.c`, original keyset): `b`rowse (`P` in the roguelike keys),
`G`ain new spells (study), `m` cast. Browsing shows level, mana, fail % and a
summary of the effect computed for your current level (`get_spell_info()`).

---

## 2. Casting mechanics

### 2.1 Learning spells

`calc_spells()` (`src/player-calcs.c`):

```
levels      = CL − first_spell_level + 1                      (≥ 0)
num_allowed = (adj_mag_study[spell stat] × levels + 50) / 100
new_spells  = num_allowed − spells already known
```

capped by the number of not-yet-learned spells whose level is ≤ your CL. Each new
spell costs one game turn to learn.

* Classes with `CHOOSE_SPELLS` (Mage, Druid, Necromancer, Rogue, Ranger, Blackguard)
  pick the spell (`do_cmd_study_spell()`).
* **Priests and Paladins learn a random spell** from the book you choose to study
  (`do_cmd_study_book()` picks uniformly among learnable spells in that book). You
  still control *which* book.
* If your spell stat drops (drained), or you lose levels to experience drain, you
  **forget** spells — most recently learned first, plus any whose level is now above
  your CL — and remember them automatically when the stat/level returns.

Spells allowed per level by stat (`adj_mag_study`, ×1/100):

| Spell stat | 10 | 14 | 16 | 17 | 18/00 | 18/50 | 18/70 | 18/100 | 18/120 | 18/150+ |
|---|---|---|---|---|---|---|---|---|---|---|
| Spells/level | 0.60 | 0.90 | 1.00 | 1.05 | 1.10 | 1.50 | 1.70 | 2.00 | 2.20 | 2.50 |
| Spells known at CL10 (first spell CL1) | 6 | 9 | 10 | 11 | 11 | 15 | 17 | 20 | 22 | 25 |
| … at CL20 | 12 | 18 | 20 | 21 | 22 | 30 | 34 | 40 | 44 | 50 |

Because spell lists are 10–30 spells long, the stat only limits learning in the early
game; with 18/50+ a Mage has more slots than available spells by about CL20.

### 2.2 First-cast experience

The first time a spell *succeeds*, you gain `spell exp × spell level` experience
(`spell_cast()`, `player_exp_gain(player, e * spell->slevel)`). The tables below
show this product. Some are large: the Mage's Mana Storm gives 200 × 45 = 9,000 exp,
the Priest's Demon Bane 300 × 42 = 12,600, the Necromancer's Command 250 × 45 = 11,250,
the Paladin's Single Combat 300 × 40 = 12,000. **Always cast every new spell once in a
safe place** — at low level the first-cast bonus of a freshly-learned spell can be
worth a large fraction of a level. A failed cast gives nothing (but still costs mana).

### 2.3 Mana

`calc_mana()` (`src/player-calcs.c`):

```
levels = CL − first_spell_level + 1
msp    = 1 + adj_mag_mana[spell stat] × levels / 100        (0 if levels ≤ 0)
msp   −= (total armour weight − class armour limit) / 10    (if positive)
```

"Armour weight" is everything worn *except* weapon, bow, rings, amulet and light
source. Each full pound over the limit costs 1 SP; the game reports "The weight of
your armor reduces your maximum SP." when this starts (`cumber_armor` in
`update_bonuses()`).

**Gloves do not affect spellcasting in 4.2.6.** Older versions (3.x) and many
variants penalised arcane casters for wearing gloves without Free Action or DEX; a
search of the 4.2.6 sources finds no glove check anywhere in `calc_mana()`,
`spell_chance()` or elsewhere — mages may wear any gloves (e.g. Gauntlets of
Slaying) freely. Only total armour weight matters.

Maximum SP (before armour penalty) by spell stat and level, for a class whose first
spell is CL1 (Mage, Druid, Priest, Necromancer, Paladin, Blackguard). For Rangers
(first spell CL3) subtract 2 levels, for Rogues (CL5) subtract 4.

| Spell stat | mana/level | CL1 | CL5 | CL10 | CL20 | CL30 | CL40 | CL50 |
|---|---|---|---|---|---|---|---|---|
| 10 | 0.70 | 1 | 4 | 8 | 15 | 22 | 29 | 36 |
| 16 | 1.30 | 2 | 7 | 14 | 27 | 40 | 53 | 66 |
| 18/00 | 1.50 | 2 | 8 | 16 | 31 | 46 | 61 | 76 |
| 18/50 | 2.00 | 3 | 11 | 21 | 41 | 61 | 81 | 101 |
| 18/70 | 2.50 | 3 | 13 | 26 | 51 | 76 | 101 | 126 |
| 18/100 | 4.00 | 5 | 21 | 41 | 81 | 121 | 161 | 201 |
| 18/120 | 5.00 | 6 | 26 | 51 | 101 | 151 | 201 | 251 |
| 18/150 | 6.50 | 7 | 33 | 66 | 131 | 196 | 261 | 326 |
| 18/180+ | 8.00 | 9 | 41 | 81 | 161 | 241 | 321 | 401 |

Note the jump above 18/70: each ten points of spell stat from 18/80 to 18/180 adds
0.5 SP per level — +25 SP at CL50 per step. Spell-stat items and potions are the
biggest mana upgrade in the game. Mana is recalculated from level 1 whenever the
stat changes.

### 2.4 Failure rates

`spell_chance()` in `src/player-spell.c`, in order:

1. Start with the spell's base fail (`spell:` line).
2. Subtract **3 per character level above the spell's level**.
3. Subtract `adj_mag_stat[spell stat]` (table below).
4. If the spell costs more mana than you have: add 5 per missing point.
5. Minimum = `adj_mag_fail[spell stat]`; if your class lacks `ZERO_FAIL` the minimum
   is raised to **5%**.
6. **Necromancer (UNLIGHT) standing on a lit grid: +25.**
7. **Afraid: +20.**
8. Clamp to [minimum, **50%**].
9. Stunned: +15 (+25 if heavy stun / knocked out).
10. Amnesia: `fail = 50 + fail/2`.
11. Final cap 95%.

Two consequences that are easy to miss:

* **No spell you know ever has more than 50% fail** unless you are stunned or
  amnesiac. A freshly-learned Banishment (base 95%) shows 50%, not 95%.
* Fear and the Necromancer's light penalty are added *before* the 50% clamp, so they
  matter for spells you've mastered and not at all for spells already at 50%.
  Stun and amnesia are added *after* the clamp and always hurt.

Stat adjustments (`adj_mag_stat`) and minimum fail (`adj_mag_fail`):

| Spell stat | 8 | 10 | 14 | 16 | 17 | 18/00 | 18/20 | 18/50 | 18/80 | 18/100 | 18/120 | 18/150 | 18/180 | 18/200 | 18/220 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Fail reduction | −1 | 0 | 1 | 3 | 4 | 5 | 7 | 10 | 15 | 21 | 27 | 36 | 45 | 51 | 57 |
| Minimum fail | 50% | 20% | 10% | 8% | 7% | 6% | 5% | 4% | 4% | 3% | 2% | 1% | 1% | 0% | 0% |

Worked examples:

* Human Mage, base INT 18 → 18/30 (reduction 8, minimum 5%): Magic Missile at CL1 =
  22 − 0 − 8 = **14%**; at CL5 = 22 − 12 − 8 = 2 → **5%** (minimum). Frost Bolt at CL13
  = 40 − 8 = **32%**; at CL20 = 40 − 21 − 8 = **11%**.
* Same mage at CL45 with INT 18/150 (reduction 36, minimum 1%): Mana Storm =
  85 − 0 − 36 = **49%**; at CL50 = 85 − 15 − 36 = **34%**. With 18/200 INT
  (reduction 51, minimum 0%) at CL50: **19%**.
* Rogue with INT 15 (reduction 2): Phase Door at CL7 = 55 − 0 − 2 = 53 → **50%**;
  at CL20 = 55 − 39 − 2 = **14%**; floor 5% from CL22.
* Necromancer, Nether Bolt at CL1 with INT 18/30, standing in a lit room:
  14 + 25 = **39%** (14% in the dark).

### 2.5 Mana regeneration

`player_regen_mana()` (`src/player-util.c`) runs once per 10 game turns
(`process_world()`), i.e. once per normal-speed player turn regardless of your own
speed:

```
gain (in 1/65536 SP) = max SP × percent + 524
percent = 197 (PY_REGEN_NORMAL)
        × 2 with Regeneration
        × 2 while resting (after 5 consecutive turns of rest)
        ÷ 2 with Impaired Mana Recovery
```

So without bonuses you regain about 0.3% of max SP (plus ~0.008 SP) per turn —
roughly 330 turns from empty to full; resting halves that, Regeneration halves it
again. Speed does not increase regeneration per game time, but a fast character gets
more actions per regeneration tick. Blackguards are completely different (§12).

Other mana sources: Potions of Restore Mana, the Mage's **Tap Magical Energy**
(drain a wand/staff: `(5 + item level) × 3 × charges / 2 / 6` SP, at least 36 energy
needed, briefly stuns you — `effect_handler_TAP_DEVICE()`), the Necromancer's **Tap
Unlife** and **Power Sacrifice**.

### 2.6 Casting without enough mana

If a spell costs more SP than you have, the game asks "Attempt it anyway?". If you
do (`spell_cast()`): your SP drops to 0, you are **paralysed for 1d(5 × shortfall + 1)
turns** (this bypasses Free Action), and you have a 50% chance of **CON damage** —
permanent CON loss happens when `randint0(100) < 25` (`player_over_exert()`). The
fail chance is also +5% per missing point. This is a last-ditch option only.

### 2.7 Casting restrictions

`player_can_cast()` (`src/player-util.c`): you cannot cast while **blind**, while
your grid is unseen (no light — except Necromancers, who always see their own grid),
or while **confused**. You *can* cast while afraid (+20% fail), stunned
(+15/+25%), poisoned, hallucinating or cut. Shapechanged characters must revert
first; answering `y` to the prompt reverts and casts in the same action
(`player_get_resume_normal_shape()`).

Casting takes one normal turn of energy; with **Mana Channel** active (`TMD_FASTCAST`)
it takes 3/4 of a turn (`do_cmd_cast()` in `src/cmd-obj.c`). Being stunned cancels
Mana Channel.

### 2.8 Beams, ranges and how dice scale

* `expr:` lines evaluate **left to right with integer division** (no operator
  precedence: `PLAYER_LEVEL:- 1 / 5 + 3` means ((CL − 1) / 5) + 3) —
  `src/z-expression.c`. All values in the tables below were computed that way.
* **Beam chance** for `BOLT_OR_BEAM` = character level (Mage, `BEAM` flag) or half
  the level (others), plus the effect's own modifier (Magic Missile has −10)
  (`beam_chance()`, `effect_handler_BOLT_OR_BEAM()`). Only Mages have
  bolt-or-beam spells.
* `BALL:elem:r:n` has radius r + CL/n (Orb of Draining: 2, becoming 3 at CL30).
* `SHORT_BEAM:elem:r:n` has length r + CL/n (Electric Arc: length = CL; Earth Rising:
  4 + CL/5), capped by the 20-square maximum range.
* `ARC:elem:r:deg` is a cone of the given width (minimum 20°) and range r (0 = the full
  range of 20, `z_info->max_range` from `constants.txt`).
* `PROJECT_LOS` hits every monster in line of sight; `PROJECT_LOS_AWARE` every
  monster in view.
* `TIMED_INC:effect:0:n` — when the effect is already active, recasting adds only n
  turns instead of a fresh roll (Haste Self, Smite Evil, Venom, Cover Tracks, Berserk
  Strength: +5; Bloodlust: −1). `effect_handler_TIMED_INC()`.
* `HEAL_HP` with `m` dice heals the larger of the fixed amount and that percentage of
  your *missing* HP (`effect_handler_HEAL_HP()`).
* `TELEPORT` with `M` dice teleports that percentage of the distance to the farthest
  level edge (`effect_handler_TELEPORT()`), with ±25% randomisation.
* Player spell damage is **not** boosted by anything (spells pass a device boost of 0);
  it depends only on character level.

How some projection types interact with monsters (`src/project-mon.c`):

| Projection | Special rule |
|---|---|
| missile, mana, meteor | No resistance at all ("unresistable") |
| nether | Undead immune; nether-resisters take ×3/(7..12); **evil monsters take half** |
| holy orb | ×2 against evil monsters |
| light_weak / dark_weak | Light_weak only hurts light-sensitive monsters; dark_weak does no damage |
| light | ×2 vs light-sensitive; light breathers resist |
| disenchant | Also makes spellcasting monsters fail more (`MON_TMD_DISEN`) |
| sound, ice, force | Chance to stun; force also knocks the target back |
| gravity | Chance to teleport the target 10 squares (high-level monsters resist) |
| chaos | Confuses, may polymorph |
| mon drain (Dispel Life, Annihilate) | No effect on non-living monsters |
| mon crush (Crush) | Only kills monsters with HP below the damage; others unaffected |

---

## 3. Where the books come from

In 4.2 spellbooks have **no entries in `object.txt`** (only their object bases in
`object_base.txt`); book allocation is set by the
`book-properties:cost:commonness:min to max` lines in `class.txt` (parsed by `parse_class_book_properties()` in `src/init.c` into the
object's cost, allocation probability and depth range). **Town books** are also
*always* stocked by the Bookseller (store 5 — `always:magic book` etc. in
`store.txt` adds every non-dungeon book of that type), so you only ever need to find
**dungeon books** in the dungeon.

| Realm | Book | Type | Cost | Commonness | Depth | Used by (spells) |
|---|---|---|---|---|---|---|
| arcane | [First Spells] | town | 25 | 40 | 1–100 | Mage (7), Rogue (6) |
| arcane | [Attacks and Knowledge] | town | 400 | 40 | 10–100 | Mage (6) |
| arcane | [Magical Defences] | town | 1600 | 20 | 30–100 | Mage (6) |
| arcane | [Arcane Control] | **dungeon** | 30000 | 15 | 40–100 | Mage (6), Rogue (4) |
| arcane | [Wizard's Tome of Power] | **dungeon** | 50000 | 10 | 60–100 | Mage (5) |
| nature | [Lesser Charms] | town | 25 | 40 | 1–100 | Druid (6), Ranger (6) |
| nature | [Gifts of Nature] | town | 400 | 40 | 10–100 | Druid (6) |
| nature | [Creature Dominion] | **dungeon** | 10000 | 20 | 30–100 | Druid (5) |
| nature | [Nature Craft] | **dungeon** | 30000 | 15 | 40–100 | Druid (5), Ranger (5) |
| nature | [Wild Forces] | **dungeon** | 50000 | 10 | 60–100 | Druid (5) |
| divine | [Novice's Handbook] | town | 25 | 40 | 1–100 | Priest (6), Paladin (6) |
| divine | [Cleansing Power] | town | 100 | 40 | 5–100 | Priest (6) |
| divine | [Healing and Sanctuary] | **dungeon** | 10000 | 20 | 30–100 | Priest (6), Paladin (5) |
| divine | [Battle Blessings] | **dungeon** | 30000 | 15 | 40–100 | Priest (5), Paladin (5) |
| divine | [Wrath of the Valar] | **dungeon** | 50000 | 10 | 60–100 | Priest (5) |
| shadow | [Into the Shadows] | town | 25 | 40 | 1–100 | Necromancer (5), Blackguard (6) |
| shadow | [Dark Rituals] | town | 400 | 40 | 10–100 | Necromancer (5) |
| shadow | [Fear and Torment] | town | 1600 | 20 | 30–100 | Necromancer (5), Blackguard (5) |
| shadow | [Deadly Powers] | **dungeon** | 30000 | 15 | 40–100 | Necromancer (6), Blackguard (4) |
| shadow | [Corruption of Spirit] | **dungeon** | 50000 | 10 | 60–100 | Necromancer (5) |

How dungeon books are generated (`make_object()` in `src/obj-make.c`):

* The *depth* column is the allocation range: a book cannot be generated as a normal
  item shallower than its minimum depth (except via the usual out-of-depth boosts of
  item level — drops from deep monsters, "good" level feelings, etc.).
* **Books you can't read are mostly rejected**: when the random kind is a book your
  class can't browse, the game re-rolls (up to 3 tries), keeping the unreadable book
  only 1 time in 5. So a Mage sees far more magic books than prayer books.
* Books are **not "good" items** (`kind_is_good()` covers only armour, weapons,
  ammunition and `GOOD`-flagged kinds), so "good"/"great" drops (`DROP_GOOD` monsters,
  *Acquirement*) never produce books — dungeon books come from ordinary floor items
  and ordinary drops.
* Books `HATES_FIRE` (`object_base.txt`): fire attacks can burn them in your pack.
  Keep a spare of every dungeon book in your home once you have one, and consider
  carrying two.

Practical expectations: the first dungeon book of each caster ([Creature Dominion],
[Healing and Sanctuary] at depth 30; [Arcane Control], [Nature Craft],
[Deadly Powers], [Battle Blessings] at 40) typically turns up somewhere in the DL30–45
range; the deepest ones (60+) are a late-game find. Plan your character around *not*
having the dungeon book until you find it.

---

## 4. Reading the spell tables

* **Lvl / Mana / Base fail / First-cast exp**: straight from `spell:name:level:mana:fail:exp`;
  first-cast exp is `exp × level` (§2.2).
* **Effect(s)**: the `effect:` lines in order, translated. Projection names are the
  game's (`missile`, `holy orb`, `light weak`, …).
* **Dice / value by character level**: the effect's `dice:` with every `$X` expression
  evaluated at the spell's own level and at CL20/30/40/50; averages are given for
  dice. When an effect has several dice lines they are separated by "—" in effect
  order. Where a value is a power rather than damage (sleep, confuse, teleport-away
  distance, recharge strength, uncurse strength) the tactical notes explain it.

---

## 5. Mage

*Realm arcane (INT) · first spell CL1 · armour limit 30 lb · books 3 town + 2 dungeon ·
flags `ZERO_FAIL`, `BEAM`, `CHOOSE_SPELLS`.*


**[First Spells]** — town book, realm arcane, 7 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Magic Missile | 1 | 1 | 22% | 4 | bolt (sometimes beam) of missile, beam chance −10 points | L1: 3d4 (avg 7.5); L20: 6d4 (avg 15); L30: 8d4 (avg 20); L40: 10d4 (avg 25); L50: 12d4 (avg 30) |
| Light Room | 1 | 2 | 26% | 4 | light area; sphere of light weak centred on you, radius 2 | 1 |
| Find Traps, Doors & Stairs | 1 | 1 | 20% | 2 | detect traps; detect doors; detect stairs |  |
| Phase Door | 2 | 2 | 22% | 8 | teleport self | 10 |
| Electric Arc | 2 | 2 | 34% | 8 | short beam of elec, length CL (max 20) | L2: 3d6 (avg 10.5); L20: 6d6 (avg 21); L30: 8d6 (avg 28); L40: 10d6 (avg 35); L50: 12d6 (avg 42) |
| Detect Monsters | 3 | 2 | 24% | 12 | detect visible monsters |  |
| Fire Ball | 6 | 5 | 33% | 30 | ball of fire, radius 2 | L6: 12; L20: 40; L30: 60; L40: 80; L50: 100 |

**[Attacks and Knowledge]** — town book, realm arcane, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Recharging | 5 | 5 | 28% | 30 | recharge | L5: 6; L20: 8; L30: 9; L40: 11; L50: 12 |
| Identify Rune | 8 | 7 | 25% | 64 | identify |  |
| Treasure Detection | 10 | 3 | 60% | 50 | detect ore; sense gold; sense objects |  |
| Frost Bolt | 13 | 5 | 40% | 78 | bolt (sometimes beam) of cold | L13: 8d8 (avg 36); L20: 11d8 (avg 49.5); L30: 14d8 (avg 63); L40: 17d8 (avg 76.5); L50: 21d8 (avg 94.5) |
| Reveal Monsters | 15 | 6 | 40% | 120 | detect invisible monsters; detect visible monsters |  |
| Acid Spray | 20 | 5 | 30% | 240 | 60° arc of acid, range 10 | L20: 10d8 (avg 45); L30: 15d8 (avg 67.5); L40: 20d8 (avg 90); L50: 25d8 (avg 112.5) |

**[Magical Defences]** — town book, realm arcane, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Disable Traps, Destroy Doors | 5 | 5 | 30% | 30 | adjacent: kill door; adjacent: kill trap |  |
| Teleport Self | 7 | 6 | 35% | 35 | teleport self | L7: 12% of distance to level edge; L20: 36% of distance to level edge; L30: 54% of distance to level edge; L40: 72% of distance to level edge; L50: 90% of distance to level edge |
| Teleport Other | 15 | 10 | 30% | 180 | bolt: away all | L15: 45; L20: 60; L30: 90; L40: 120; L50: 150 |
| Resistance | 20 | 20 | 65% | 400 | timed opp acid; timed opp elec; timed opp cold; timed opp fire | shared duration 20+d20 |
| Tap Magical Energy | 22 | 2 | 30% | 264 | tap device |  |
| Mana Channel | 25 | 10 | 50% | 500 | timed fastcast | 5+1d5 |

**[Arcane Control]** — dungeon book, realm arcane, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Door Creation | 13 | 9 | 40% | 156 | adjacent: make door |  |
| Mana Bolt | 25 | 8 | 60% | 750 | bolt (sometimes beam) of missile | L25: 15d8 (avg 67.5); L30: 20d8 (avg 90); L40: 30d8 (avg 135); L50: 40d8 (avg 180) |
| Teleport Level | 28 | 17 | 65% | 560 | teleport level |  |
| Detection | 30 | 10 | 70% | 900 | detect traps; detect doors; detect stairs; detect ore; sense gold; sense objects; detect invisible monsters; detect visible monsters |  |
| Dimension Door | 35 | 30 | 80% | 1400 | teleport to chosen square |  |
| Thrust Away | 40 | 12 | 90% | 1600 | short beam of force, length 1+CL/10 (max 20) | L40: 40d8 (avg 180); L50: 50d8 (avg 225) |

**[Wizard's Tome of Power]** — dungeon book, realm arcane, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Shock Wave | 20 | 5 | 40% | 320 | ball of sound, radius 2 | L20: 40; L30: 60; L40: 80; L50: 100 |
| Explosion | 30 | 10 | 50% | 600 | ball of shard, radius 2; ball of force, radius 2 | shards L30: 60; L40: 80; L50: 100 — then force L30: 6; L40: 8; L50: 10 |
| Banishment | 35 | 45 | 95% | 875 | banish |  |
| Mass Banishment | 40 | 75 | 90% | 4000 | mass banish |  |
| Mana Storm | 45 | 16 | 85% | 9000 | ball of mana, radius 3 | L45: 390; L50: 400 |

### 5.1 Tactical notes — Mage

**The attack ladder.** Approximate average damage and mana cost at a few levels
(from the table; beams/balls hit more than one target):

| Spell | Mana | CL10 | CL20 | CL30 | CL40 | CL50 | Shape |
|---|---|---|---|---|---|---|---|
| Magic Missile | 1 | 4d4 = 10 | 6d4 = 15 | 8d4 = 20 | 10d4 = 25 | 12d4 = 30 | bolt/beam, unresistable |
| Electric Arc | 2 | 4d6 = 14 | 6d6 = 21 | 8d6 = 28 | 10d6 = 35 | 12d6 = 42 | short beam, length CL (max 20) |
| Fire Ball | 5 | 20 | 40 | 60 | 80 | 100 | ball r2 |
| Frost Bolt | 5 | — | 11d8 = 49.5 | 14d8 = 63 | 17d8 = 76.5 | 21d8 = 94.5 | bolt/beam |
| Acid Spray | 5 | — | 10d8 = 45 | 15d8 = 67.5 | 20d8 = 90 | 25d8 = 112.5 | 60° cone, range 10 |
| Mana Bolt | 8 | — | — | 20d8 = 90 | 30d8 = 135 | 40d8 = 180 | bolt/beam, unresistable |
| Shock Wave | 5 | — | 40 | 60 | 80 | 100 | ball r2, may stun |
| Explosion | 10 | — | — | 60 + 6 | 80 + 8 | 100 + 10 | shard ball r2 + force ball r2 |
| Thrust Away | 12 | — | — | — | 40d8 = 180 | 50d8 = 225 | force short beam, length 1 + CL/10 |
| Mana Storm | 16 | — | — | — | — | 400 (390 at CL45) | ball r3, unresistable |

* **CL1–12: Magic Missile.** 1 mana, unresistable, never misses. Use Electric Arc
  (CL2) when monsters line up in a corridor — its length equals your level, so it is
  a short-range beam early on. Flasks of oil are still worth carrying.
* **CL6: Fire Ball** — the first area spell; 2×CL fire damage in radius 2. Its damage
  is flat (no dice), so it is reliable for clearing groups of weak monsters (jackals,
  cave spiders, kobolds). Fire-resistant and fire-immune monsters shrug it off.
* **CL13: Frost Bolt** — the main single-target spell from CL13 until Mana Bolt; beams
  CL% of the time. Cold is resisted by many deeper monsters; fall back on Magic
  Missile/Mana Bolt for those.
* **CL20: Acid Spray** — a 60° cone reaching 10 squares: the best "room clearer" in the
  town books. Aim it at the middle of a group.
* **CL25: Mana Bolt** ([Arcane Control], DL40+) — unresistable (CL−10)d8, the
  long-term workhorse.
* **CL45: Mana Storm** ([Wizard's Tome of Power], DL60+) — 2×CL+300 unresistable
  damage in radius 3 for 16 mana: the strongest damage spell in the game per mana.

**Escapes and defence.**
* **Phase Door** (CL2, 2 mana, 10 squares) — your primary "get out of melee" tool for
  the whole game.
* **Teleport Self** (CL7, [Magical Defences]): distance = (CL × 9/5)% of your largest
  distance to a level edge (`effect_handler_TELEPORT()`), randomised ±25%. On a full
  198×66 level you are always at least 99 columns from one edge, so at CL7 (12%) it
  only moves you ~12 squares — little more than Phase Door. It reaches ~36% at CL20,
  54% at CL30, 90% at CL50. Carry Teleportation scrolls until about CL20.
* **Teleport Other** (CL15): a bolt that teleports the first monster hit 3×CL squares
  away (45 at CL15, 150 at CL50). The best answer to a single dangerous melee monster.
  Does not work on monsters that resist teleportation.
* **Resistance** (CL20): 20+1d20 turns of temporary acid, electricity, cold and fire
  resistance (one shared roll). Stacks with permanent resistance (double resistance).
* **Mana Channel** (CL25): 5+1d5 turns where casting costs 3/4 of a turn — effectively
  +33% casting speed. Stun cancels it.
* **Dimension Door** (CL35): teleport to a chosen visible location — escape *and*
  repositioning. 80% base fail, so expect high fail at first.
* **Teleport Level** (CL28): up or down one level at random (only down with forced
  descent). The emergency exit when the whole level is hostile.
* **Banishment** (CL35, 45 mana) and **Mass Banishment** (CL40, 75 mana): remove a
  whole monster symbol from the level / every monster within 20 squares (uniques
  excepted); you take 1d4 / 1d3 damage per monster removed.

**Utility.**
* **Find Traps, Doors & Stairs** (CL1, 1 mana): detects in a 22-row × 40-column window
  around you (`effect-yx:22:40`). Cheap stair-finding makes Mages excellent divers.
* **Detect Monsters** (CL3) shows non-invisible monsters; **Reveal Monsters** (CL15) adds
  invisible ones. Use one of them every time you arrive on a new level or open a door.
* **Recharging** (CL5): strength CL/8+6. Failure chance is 1 in
  `strength + (100 − item level)/10 − 2 × charges per item` (`recharge_failure_chance()`
  in `src/effects.c`); on failure one item of the stack is destroyed. On success the
  item gains `2 + 1d(strength / (10 − ease) + 1)` charges, where ease = (100 − item
  level)/10. Recharge wands when they are empty and singly, not in big stacks.
* **Identify Rune** (CL8) — identifies one unknown rune on an object.
* **Treasure Detection** (CL10) — finds gold veins and *senses* objects (shows that an
  object is there, not what it is).
* **Detection** (CL30) — every detection in one cast.
* **Tap Magical Energy** (CL22) — drain a wand/staff into mana, stunning yourself
  briefly (§2.5).
* **Door Creation** (CL13) — closes you in with doors on every adjacent floor grid; a
  classic way to buy turns against non-door-opening monsters.

**Learning order suggestions.** CL1: Magic Missile, Find Traps/Doors/Stairs (or Detect
Monsters at CL3), Phase Door at CL2. Then Fire Ball (CL6), Teleport Self (CL7, if you
bought Magical Defences — even short teleports help), Identify Rune/Recharging,
Frost Bolt (CL13), Teleport Other (CL15), Reveal Monsters (CL15), Acid Spray and
Resistance (CL20). With INT 18/50+ you will be able to learn everything anyway (§2.1).

**Biggest first-cast exp** (worth casting the moment you learn them): Mana Storm 9,000;
Mass Banishment 4,000; Thrust Away 1,600; Dimension Door 1,400; Detection 900;
Banishment 875; Mana Bolt 750.

---

## 6. Druid

*Realm nature (WIS) · first spell CL1 · armour limit 35 lb · books 2 town + 3 dungeon ·
flags `ZERO_FAIL`, `CHOOSE_SPELLS`, `CHARM`.*


**[Lesser Charms]** — town book, realm nature, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Detect Life | 1 | 1 | 23% | 2 | detect living monsters |  |
| Fox Form | 1 | 2 | 20% | 10 | become fox |  |
| Remove Hunger | 2 | 2 | 25% | 4 | nourish inc to | 50 |
| Stinking Cloud | 3 | 2 | 27% | 12 | ball of pois, radius 2 | L3: 11; L20: 20; L30: 25; L40: 30; L50: 35 |
| Confuse Monster | 5 | 3 | 30% | 25 | bolt: mon conf | L5: 5; L20: 5+d15 (avg 13); L30: 5+d25 (avg 18); L40: 5+d35 (avg 23); L50: 5+d45 (avg 28) |
| Slow Monster | 6 | 4 | 30% | 30 | bolt: mon slow | L6: 5+d1 (avg 6); L20: 5+d15 (avg 13); L30: 5+d25 (avg 18); L40: 5+d35 (avg 23); L50: 5+d45 (avg 28) |

**[Gifts of Nature]** — town book, realm nature, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Cure Poison | 4 | 4 | 30% | 16 | cure poisoned |  |
| Resist Poison | 7 | 6 | 32% | 35 | timed opp pois | 20+d20 |
| Turn Stone to Mud | 8 | 3 | 25% | 40 | line (beam) of kill wall | 20+1d30 |
| Sense Surroundings | 9 | 4 | 35% | 54 | map area | L9: ±22 rows, ±44 cols; L20: ±22 rows, ±44 cols; L30: ±44 rows, ±88 cols; L40: ±44 rows, ±88 cols; L50: ±44 rows, ±88 cols |
| Lightning Strike | 12 | 6 | 35% | 96 | strike of elec at target, radius 0; strike of sound at target, radius 3 | L12: 3d4 (avg 7.5); L20: 5d4 (avg 12.5); L30: 7d4 (avg 17.5); L40: 10d4 (avg 25); L50: 12d4 (avg 30) — L12: 17; L20: 25; L30: 35; L40: 45; L50: 55 |
| Earth Rising | 14 | 5 | 40% | 168 | short beam of shard, length 4+CL/5 (max 20) | L14: 6d6 (avg 21); L20: 8d6 (avg 28); L30: 12d6 (avg 42); L40: 15d6 (avg 52.5); L50: 18d6 (avg 63) |

**[Creature Dominion]** — dungeon book, realm nature, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Trance | 20 | 10 | 45% | 300 | adjacent monsters: mon hold | L20: 3+d2 (avg 4.5); L30: 3+d4 (avg 5.5); L40: 3+d5 (avg 6); L50: 3+d7 (avg 7) |
| Mass Sleep | 25 | 15 | 50% | 375 | sleep all on every monster in line of sight | L25: 750; L30: 800; L40: 900; L50: 1000 |
| Become Pukel-man | 30 | 20 | 75% | 1500 | become Pukel-man |  |
| Eagle's Flight | 35 | 20 | 80% | 2800 | become eagle |  |
| Bear Form | 40 | 20 | 85% | 4000 | become bear |  |

**[Nature Craft]** — dungeon book, realm nature, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Tremor | 20 | 20 | 60% | 320 | earthquake radius 4 at target |  |
| Haste Self | 25 | 12 | 65% | 375 | timed fast (adds 5 if already active) | L25: 25+d20 (avg 35.5); L30: 30+d20 (avg 40.5); L40: 40+d20 (avg 50.5); L50: 50+d20 (avg 60.5) |
| Revitalize | 35 | 70 | 90% | 3150 | restore str; restore int; restore wis; restore dex; restore con; restore exp |  |
| Rapid Regeneration | 37 | 20 | 90% | 3700 | timed heal | 5+1d3 |
| Herbal Curing | 40 | 20 | 90% | 4000 | cure cut; cure poisoned; cure stun; cure blackbreath; nourish inc to | 80 |

**[Wild Forces]** — dungeon book, realm nature, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Meteor Swarm | 30 | 14 | 85% | 600 | swarm of meteor projectiles, radius 1 | L30: 45 dmg × 3 meteors; L40: 50 dmg × 4 meteors; L50: 55 dmg × 4 meteors |
| Rift | 35 | 20 | 60% | 875 | beam of gravity | L35: 40+35d7 (avg 180); L40: 40+40d7 (avg 200); L50: 40+50d7 (avg 240) |
| Ice Storm | 37 | 25 | 75% | 888 | ice on every monster in line of sight | L37: 3d111 (avg 168); L40: 3d120 (avg 181.5); L50: 3d150 (avg 226.5) |
| Volcanic Eruption | 40 | 30 | 75% | 2000 | sphere of fire centred on you, radius 5; earthquake radius 5 around you | L40: 60+d120 (avg 120.5); L50: 75+d150 (avg 150.5) |
| River of Lightning | 43 | 35 | 75% | 4300 | 20° arc of plasma, max range | L43: 53d8 (avg 238.5); L50: 60d8 (avg 270) |

### 6.1 Tactical notes — Druid

**Offence progression.**

| Spell | Mana | CL10 | CL20 | CL30 | CL40 | CL50 | Shape |
|---|---|---|---|---|---|---|---|
| Stinking Cloud | 2 | 15 | 20 | 25 | 30 | 35 | poison ball r2 |
| Lightning Strike | 6 | — | 5d4 (12.5) + 25 | 7d4 (17.5) + 35 | 10d4 (25) + 45 | 12d4 (30) + 55 | elec strike r0 + sound strike r3 at target |
| Earth Rising | 5 | — | 8d6 = 28 | 12d6 = 42 | 15d6 = 52.5 | 18d6 = 63 | shard short beam, length 4 + CL/5 |
| Meteor Swarm | 14 | — | — | 45 × 3 | 50 × 4 | 55 × 4 | 3–4 meteors, each a radius-1 unresistable ball |
| Rift | 20 | — | — | — | 40+40d7 = 200 | 40+50d7 = 240 | gravity beam (may teleport target) |
| Ice Storm | 25 | — | — | — | 3d120 = 181.5 | 3d150 = 226.5 | ice on everything in line of sight |
| Volcanic Eruption | 30 | — | — | — | 60+d120 = 120.5 | 75+d150 = 150.5 | fire sphere r5 around you + earthquake r5 |
| River of Lightning | 35 | — | — | — | — | 60d8 = 270 (53d8 at CL43) | 20° plasma arc, full range |

* **Stinking Cloud** (CL3) carries the first 12 levels. Its damage barely grows
  (CL/2+10) — treat it as an early-game spell; poison-resistant monsters ignore it.
* **Lightning Strike** (CL12) strikes the *target square from above*: no bolt path is
  needed beyond line of sight. It is a small electric hit plus a radius-3 sound burst
  that can stun. If you cast it at a direction instead of a target, or the target grid
  is not in line of fire, **it lands on you** (`effect_handler_STRIKE()` — "the player
  will be the target"). Always target a monster.
* **Earth Rising** (CL14) is the Druid's corridor beam: shards in a line of length
  4 + CL/5.
* **Meteor Swarm** (CL30, [Wild Forces] DL60+): CL/2+30 damage per meteor, CL/20+2
  meteors; each meteor stops at the first monster or wall and explodes for
  unresistable damage. If one kills its target, the next can continue.
* **Ice Storm** and **River of Lightning** are the late-game screen-clearers.
  **Volcanic Eruption** centres on *you* — the earthquake can damage you and destroy
  items, so use it when surrounded, not as a ranged attack.

**Control spells (benefit from `CHARM` vs animals: +50% power).**
* **Confuse Monster** (CL5) and **Slow Monster** (CL6): power 5+1d(CL−5) (5 + 1d45
  at CL50).
* **Trance** (CL20, [Creature Dominion]): holds every adjacent monster for 3+1d(CL/7)
  turns (longer for animals).
* **Mass Sleep** (CL25): sleep power CL×10+500 on everything in view; monsters that
  resist sleep are unaffected. The single best "skip this room" spell any class has.
* **Tremor** (CL20, [Nature Craft]): targeted radius-4 earthquake — can wall off a
  corridor or bury monsters; destroys items in the area (not artifacts).

**Defence and support.** Remove Hunger (CL2) means you never carry food. Cure Poison
(CL4), Resist Poison (CL7, 20+1d20 turns of temporary poison resistance), Turn Stone to
Mud (CL8; 20+1d30 damage to rock-vulnerable monsters), Sense Surroundings (CL9; maps
22 rows × 44 columns each way, doubling at CL30), **Haste Self** (CL25; CL+1d20 turns,
+5 turns per recast while active), Revitalize (CL35: restore all stats and exp),
**Rapid Regeneration** (CL37: `TMD_HEAL` for 5+1d3 turns — heals 30 HP per turn but
burns food very fast and ends when you get hungry; `process_world()`), **Herbal Curing**
(CL40: cures cuts, poison, stun and Black Breath, and feeds you).

**No teleport spell.** The nature list has no Phase Door or Teleport. Druids rely on
scrolls (and Eagle form's fast movement late). Plan accordingly.

**Shapechanges** — see §13 for full stats. Summary:
* **Fox Form** (CL1, 2 mana): +1 blow, moves cost half a turn, +5 stealth, Free Action,
  Feather Fall; −3 STR, −3 to-hit/to-dam, +3 AC. A scouting/escape form from level 1.
* **Become Pukel-man** (CL30): **poison immunity**, resist shards, 10 damage reduction
  per hit, +20 AC, +5 to-dam, +20 save, STR/CON +4 (sustained), stun-proof, Regeneration,
  Hold Life; −5 speed, −2 stealth. Cures poison and stun on assuming. A tank form for
  poison breathers.
* **Eagle's Flight** (CL35): moves cost 1/4 turn, trap immunity, Free Action, See
  Invisible, immune to blindness, confusion and fear, +5 to-hit, +10 AC. Fastest way
  to travel and to flee.
* **Bear Form** (CL40): +15 to-hit, +15 to-dam, +5 AC, +1 blow, +10 melee skill,
  fear-proof, STR +3, CON +2; INT −2, −3 stealth, worse disarming.

Remember: forms *add* to your equipment, keep your weapon's damage, and block item use
and spellcasting (revert with `y` at the prompt for a free change-back).

---

## 7. Priest

*Realm divine (WIS) · first spell CL1 · armour limit 35 lb · books 2 town + 3 dungeon ·
flags `BLESS_WEAPON`, `ZERO_FAIL` · prayers are learned **at random** from the chosen
book.*


**[Novice's Handbook]** — town book, realm divine, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Call Light | 1 | 1 | 10% | 2 | light area; sphere of light weak centred on you, radius 1 | L1: 0; L20: 2d10 (avg 11); L30: 2d15 (avg 16); L40: 2d20 (avg 21); L50: 2d25 (avg 26) |
| Detect Evil | 1 | 1 | 10% | 4 | detect evil |  |
| Minor Healing | 1 | 2 | 15% | 4 | heal hp; reduce cut; reduce stun | L1: 11 HP or 0% of wounds; L20: 30 HP or 5% of wounds; L30: 40 HP or 7% of wounds; L40: 50 HP or 10% of wounds; L50: 60 HP or 12% of wounds — 20 — 20 |
| Bless | 1 | 2 | 20% | 4 | timed blessed | L1: 11+d11 (avg 17); L20: 30+d30 (avg 45.5); L30: 40+d40 (avg 60.5); L40: 50+d50 (avg 75.5); L50: 60+d60 (avg 90.5) |
| Sense Invisible | 3 | 4 | 25% | 12 | timed sinvis | 24+d24 |
| Heroism | 5 | 2 | 30% | 25 | heal hp; cure afraid; timed hero | 10 — L5: none; L20: 1+d1 (avg 2); L30: 11+d11 (avg 17); L40: 21+d21 (avg 32); L50: 31+d31 (avg 47) |

**[Cleansing Power]** — town book, realm divine, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Orb of Draining | 7 | 7 | 40% | 28 | ball of holy orb, radius 2 (+1 per 30 CL) | L7: 10+3d6 (avg 20.5); L20: 30+3d6 (avg 40.5); L30: 45+3d6 (avg 55.5); L40: 60+3d6 (avg 70.5); L50: 75+3d6 (avg 85.5) |
| Spear of Light | 7 | 6 | 30% | 35 | line (beam) of light weak | 6d8 |
| Dispel Undead | 10 | 14 | 55% | 60 | disp undead on every monster in line of sight | L10: d50 (avg 25.5); L20: d100 (avg 50.5); L30: d150 (avg 75.5); L40: d200 (avg 100.5); L50: d250 (avg 125.5) |
| Dispel Evil | 12 | 20 | 70% | 120 | disp evil on every monster in line of sight | L12: d60 (avg 30.5); L20: d100 (avg 50.5); L30: d150 (avg 75.5); L40: d200 (avg 100.5); L50: d250 (avg 125.5) |
| Protection from Evil | 14 | 8 | 42% | 70 | timed protevil | L14: 42+d25 (avg 55); L20: 60+d25 (avg 73); L30: 90+d25 (avg 103); L40: 120+d25 (avg 133); L50: 150+d25 (avg 163) |
| Remove Curse | 16 | 6 | 38% | 128 | remove curse | L16: 16+d16 (avg 24.5); L20: 20+d20 (avg 30.5); L30: 30+d30 (avg 45.5); L40: 40+d40 (avg 60.5); L50: 50+d50 (avg 75.5) |

**[Healing and Sanctuary]** — dungeon book, realm divine, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Portal | 10 | 4 | 30% | 80 | teleport self | L10: 35; L20: 40; L30: 45; L40: 50; L50: 55 |
| Remembrance | 20 | 30 | 90% | 1000 | restore exp |  |
| Word of Recall | 25 | 30 | 75% | 250 | recall |  |
| Healing | 30 | 50 | 80% | 3000 | heal hp; cure cut; cure poisoned; cure stun; cure amnesia | 2000 |
| Restoration | 35 | 70 | 90% | 4550 | restore str; restore int; restore wis; restore dex; restore con |  |
| Clairvoyance | 37 | 50 | 80% | 5550 | light level |  |

**[Battle Blessings]** — dungeon book, realm divine, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Enchant Weapon | 35 | 50 | 80% | 8050 | enchant toboth | 1d4 |
| Enchant Armour | 37 | 60 | 85% | 9250 | enchant toac | 1+1d3 |
| Smite Evil | 25 | 20 | 70% | 1000 | timed att evil (adds 5 if already active) | 20+d20 |
| Glyph of Warding | 20 | 40 | 90% | 700 | glyph warding |  |
| Demon Bane | 42 | 40 | 80% | 12600 | timed att demon (adds 5 if already active) | 20+d20 |

**[Wrath of the Valar]** — dungeon book, realm divine, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Banish Evil | 25 | 25 | 80% | 6250 | away evil on every monster in line of sight | 100 |
| Word of Destruction | 35 | 35 | 80% | 4025 | destruction radius 15 |  |
| Holy Word | 39 | 32 | 95% | 780 | disp evil on every monster in line of sight; heal hp; cure afraid; cure poisoned; cure stun; cure cut | L39: d156 (avg 78.5); L40: d160 (avg 80.5); L50: d200 (avg 100.5) — 1000 |
| Spear of Oromë | 40 | 10 | 75% | 5200 | beam of holy orb | L40: 20d8 (avg 90); L50: 25d8 (avg 112.5) |
| Light of Varda | 42 | 40 | 85% | 8400 | ball of light, radius 4; light weak on every monster in line of sight | L42: 210; L50: 250 — 100 |

Note: in [Battle Blessings] the spells are listed in book order, which is not level
order (Enchant Weapon CL35, Enchant Armour CL37, Smite Evil CL25, Glyph of Warding
CL20, Demon Bane CL42).

### 7.1 Tactical notes — Priest

**Offence.**

| Prayer | Mana | CL10 | CL20 | CL30 | CL40 | CL50 | Notes |
|---|---|---|---|---|---|---|---|
| Orb of Draining | 7 | 15+3d6 = 25.5 | 30+3d6 = 40.5 | 45+3d6 = 55.5 | 60+3d6 = 70.5 | 75+3d6 = 85.5 | ball r2 (r3 from CL30); ×2 vs evil; nothing resists holy orb otherwise |
| Spear of Light | 6 | 6d8 = 27 | same | same | same | same | beam; only hurts light-sensitive monsters, lights the line |
| Dispel Undead | 14 | d50 | d100 | d150 | d200 | d250 | every undead in view |
| Dispel Evil | 20 | — | d100 | d150 | d200 | d250 | every evil monster in view |
| Holy Word | 32 | — | — | — | d160 + heal 1000 | d200 + heal 1000 | dispel evil + full cure |
| Spear of Oromë | 10 | — | — | — | 20d8 = 90 | 25d8 = 112.5 | holy orb beam (×2 vs evil) |
| Light of Varda | 40 | — | — | — | — | 250 (210 at CL42) | light ball r4 + 100 to light-sensitive in view |

* Before CL7 the Priest fights with the starting Mace (blessed-fighter +2/+2) and thrown
  flasks of oil. **Buy [Cleansing Power]** (cost 100, always in the Bookseller) at once;
  Orb of Draining at CL7 is your main attack for the next 30 levels. Because you learn
  at random, study [Cleansing Power] only when Orb is learnable (CL7) — earlier slots
  are better spent on [Novice's Handbook].
* Orb of Draining is a *ball* — target the monster (or a square next to a group) and it
  explodes there. Against evil monsters (most orcs, undead, demons) its damage doubles.
* Dispel Undead/Evil hit every matching monster in your field of view (no bolt path
  needed) — use them when a group of undead/evil monsters is visible and closing.

**Healing and cures.**
* **Minor Healing** (CL1): heals CL+10 HP, or CL/4 % of your missing HP if larger; also
  reduces cuts and stun by 20.
* **Healing** (CL30, [Healing and Sanctuary]): 2000 HP and cures cuts, poison, stun and
  amnesia — effectively a full heal for 50 mana.
* **Holy Word** (CL39): 1000 HP heal, cures fear, poison, stun and cuts, *and* dispels
  evil — attack and heal in one action.
* **Remembrance** (CL20) restores experience; **Restoration** (CL35) restores all stats.

**Buffs.**
* **Bless** (CL1): +5 AC and +10 to-hit (and a little device skill) for
  CL+10+1d(CL+10) turns — cast before every melee fight.
* **Heroism** (CL5): always heals 10 and cures fear; grants Hero (+12 to-hit, fear
  immunity) only from CL20 (duration (CL−19)+1d(CL−19)).
* **Sense Invisible** (CL3): 24+1d24 turns of See Invisible.
* **Protection from Evil** (CL14): 3×CL+1d25 turns. An evil monster's melee blow is
  repelled if the monster's level ≤ your level and `randint0(100) + CL > 50`
  (`src/mon-attack.c`) — e.g. ~79% at CL30 against evil monsters of level ≤ 30.
* **Smite Evil** (CL25): weapon gains *slay evil* (×2, `EVIL_2`) for 20+1d20 turns;
  **Demon Bane** (CL42): *slay demon* ×5.
* **Glyph of Warding** (CL20): a glyph monsters must break to step on or melee you
  through; summoned monsters can't appear on it.

**Movement and utility.**
* **Portal** (CL10, [Healing and Sanctuary]): teleport CL/2+30 squares — the only priest
  teleport spell. Until you find that dungeon book, carry Phase Door and Teleportation.
* **Word of Recall** (CL25): saves buying scrolls; activates after 15–34 turns, cast
  again to cancel.
* **Clairvoyance** (CL37): lights and maps the whole level and senses all objects.
* **Remove Curse** (CL16): strength CL+1d(CL) vs. the curse's power; a failure makes the
  item fragile, and a fragile item may be destroyed on a further failure
  (`uncurse_object()`).
* **Banish Evil** (CL25, [Wrath of the Valar]): teleports away every evil monster in
  view. **Word of Destruction** (CL35): destroys everything within 15 squares (not
  stairs or permanent walls); you are blinded 10+1d10 turns unless you resist light or
  blindness.
* **Enchant Weapon / Armour** (CL35/37): free enchanting late in the game.

**Biggest first-cast exp:** Demon Bane 12,600; Enchant Armour 9,250; Light of Varda
8,400; Enchant Weapon 8,050; Banish Evil 6,250; Clairvoyance 5,550; Spear of Oromë
5,200; Restoration 4,550.

---

## 8. Necromancer

*Realm shadow (INT) · first spell CL1 · armour limit 30 lb · books 3 town + 2 dungeon ·
flags `ZERO_FAIL`, `CHOOSE_SPELLS`, `UNLIGHT`, `EVIL`.*


**[Into the Shadows]** — town book, realm shadow, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Nether Bolt | 1 | 1 | 22% | 2 | bolt of nether | L1: 3d4 (avg 7.5); L20: 8d4 (avg 20); L30: 10d4 (avg 25); L40: 13d4 (avg 32.5); L50: 15d4 (avg 37.5) |
| Sense Invisible | 2 | 4 | 25% | 6 | timed sinvis | 24+d24 |
| Create Darkness | 3 | 3 | 10% | 12 | darken area; sphere of dark weak centred on you, radius 1 | 1 |
| Bat Form | 3 | 4 | 20% | 30 | become bat |  |
| Read Minds | 4 | 4 | 10% | 24 | detect soul; read minds | L4: ±0 rows, ±1 cols around each mind; L20: ±2 rows, ±3 cols around each mind; L30: ±3 rows, ±4 cols around each mind; L40: ±4 rows, ±5 cols around each mind; L50: ±5 rows, ±6 cols around each mind |

**[Dark Rituals]** — town book, realm shadow, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Tap Unlife | 7 | 0 | 50% | 21 | tap unlife | L7: 3d6 (avg 10.5); L20: 7d6 (avg 24.5); L30: 9d6 (avg 31.5); L40: 12d6 (avg 42); L50: 14d6 (avg 49) |
| Crush | 10 | 6 | 40% | 50 | mon crush on every monster in line of sight; damages you | L10: 40; L20: 80; L30: 120; L40: 160; L50: 200 — L10: d20 (avg 10.5); L20: d40 (avg 20.5); L30: d60 (avg 30.5); L40: d80 (avg 40.5); L50: d100 (avg 50.5) |
| Sleep Evil | 12 | 5 | 50% | 60 | sleep evil on every monster in line of sight | L12: 620; L20: 700; L30: 800; L40: 900; L50: 1000 |
| Shadow Shift | 15 | 10 | 50% | 150 | teleport self; damages you | 30 — 2d5 |
| Disenchant | 16 | 12 | 50% | 240 | bolt of disen | L16: 2d42 (avg 43); L20: 2d50 (avg 51); L30: 2d70 (avg 71); L40: 2d90 (avg 91); L50: 2d110 (avg 111) |

**[Fear and Torment]** — town book, realm shadow, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Frighten | 17 | 4 | 40% | 204 | bolt: turn all | L17: 17; L20: 20; L30: 30; L40: 40; L50: 50 |
| Vampire Strike | 20 | 5 | 50% | 200 | jump and bite | L20: 40; L30: 60; L40: 80; L50: 100 |
| Dispel Life | 22 | 20 | 50% | 220 | mon drain on every monster in line of sight | L22: d66 (avg 33.5); L30: d90 (avg 45.5); L40: d120 (avg 60.5); L50: d150 (avg 75.5) |
| Dark Spear | 25 | 10 | 60% | 300 | line (beam) of dark | L25: 2d50 (avg 51); L30: 2d60 (avg 61); L40: 2d80 (avg 81); L50: 2d100 (avg 101) |
| Warg Form | 28 | 20 | 60% | 2240 | become warg |  |

**[Deadly Powers]** — dungeon book, realm shadow, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Banish Spirits | 25 | 25 | 80% | 625 | away spirit on every monster in line of sight | 100 |
| Annihilate | 30 | 20 | 75% | 750 | bolt of mon drain | L30: 120; L40: 160; L50: 200 |
| Grond's Blow | 35 | 35 | 80% | 4025 | destruction radius 15 |  |
| Unleash Chaos | 38 | 15 | 90% | 3800 | **one of** (random): beam of chaos / 20° arc of chaos, full range / ball of chaos radius 3; then wake all nearby monsters; then turn all (fear) on every monster in line of sight | chaos: L38: 8d38 (avg 156); L40: 8d40 (avg 164); L50: 8d50 (avg 204) — fear power = CL |
| Fume of Mordor | 40 | 50 | 80% | 6000 | darken level | 1 |
| Storm of Darkness | 43 | 16 | 80% | 8600 | ball of dark, radius 4 | L43: 4d86 (avg 174); L50: 4d100 (avg 202) |

**[Corruption of Spirit]** — dungeon book, realm shadow, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Power Sacrifice | 27 | 0 | 20% | 540 | damages you; restore mana | 50 — 50 |
| Zone of Unmagic | 32 | 16 | 60% | 3520 | disen on everything within radius 4 of you (including you) | L32: 96; L40: 120; L50: 150 |
| Vampire Form | 37 | 20 | 60% | 5180 | become vampire; jump and bite | L37: 37; L40: 40; L50: 50 |
| Curse | 40 | 50 | 60% | 7200 | damages you (100 HP); curse (direct damage to the targeted monster) | (CL/12+1) d (percentage of the target's HP already lost + 50): L40: 4d(50..150); L50: 5d(50..150) |
| Command | 45 | 40 | 60% | 11250 | command | 5+1d10 |

### 8.1 Unlight — the Necromancer's operating rules

(Details and sources in Chapter 2 §5.5.)

* You see grids within distance < 2 + CL/6 − light radius when your light radius is
  ≤ 1 — no torch needed. Your own grid is always visible, so you can read and cast in
  darkness.
* Standing on a **lit** grid adds **+25% spell failure** (before the 50% cap).
  Carrying a lit torch lights your own grid.
* You resist darkness; dark-based blinding does not affect you.
* Items with a +light modifier give you one less light.

Spell consequences:
* **Create Darkness** (CL3): darkens the area (and the whole room if you are in one) —
  cast it on entering a lit room to remove the fail penalty.
* **Fume of Mordor** (CL40): darkens, maps and detects objects on the whole level.
* **Storm of Darkness**, **Dark Spear**: darkness damage; you are immune to their
  blinding side-effects.
* **Grond's Blow** (destruction): blinds you only if you lack dark/blindness resistance
  — Unlight gives you dark resistance, so you are never blinded by your own Grond's Blow.

### 8.2 Tactical notes — Necromancer

**Offence.**

| Ritual | Mana | CL10 | CL20 | CL30 | CL40 | CL50 | Notes |
|---|---|---|---|---|---|---|---|
| Nether Bolt | 1 | 5d4 = 12.5 | 8d4 = 20 | 10d4 = 25 | 13d4 = 32.5 | 15d4 = 37.5 | bolt; undead immune, evil ½ |
| Tap Unlife | 0 | 4d6 = 14 | 7d6 = 24.5 | 9d6 = 31.5 | 12d6 = 42 | 14d6 = 49 | nearest undead only; gain mana |
| Crush | 6 | kills < 40 HP | < 80 | < 120 | < 160 | < 200 | every monster in LOS; costs you d(2×CL) HP |
| Disenchant | 12 | — | 2d50 = 51 | 2d70 = 71 | 2d90 = 91 | 2d110 = 111 | bolt; spell-casters fail more |
| Dispel Life | 20 | — | — | d90 | d120 | d150 | every *living* monster in view |
| Dark Spear | 10 | — | — | 2d60 = 61 | 2d80 = 81 | 2d100 = 101 | darkness beam |
| Annihilate | 20 | — | — | 120 | 160 | 200 | single living target, not elementals/golems/vortices/undead |
| Unleash Chaos | 15 | — | — | — | 8d40 = 164 | 8d50 = 204 | random beam/arc/ball of chaos + wakes all + scares LOS |
| Storm of Darkness | 16 | — | — | — | — | 4d100 = 202 (4d86 at CL43) | darkness ball r4 |
| Curse | 50 | — | — | — | 4d(%gone+50) | 5d(%gone+50) | single target, costs you 100 HP |

* **Nether Bolt** (CL1) is cheap and strong early, but read its resistance rule
  (`project_monster_handler_NETHER()`): undead take **nothing**, monsters that resist
  nether take ×3/(7–12), and **evil monsters take half**. Many early humanoids (orcs,
  kobold chieftains, cultists) are evil. Use it on animals and non-evil monsters; for
  evil ones expect half damage; never on undead.
* **Tap Unlife** (CL7, 0 mana): automatically targets the nearest undead, deals
  (CL/4+2)d6, and gives you mana equal to ¼ of min(damage, monster's HP)
  (`effect_handler_TAP_UNLIFE()`). It is your answer to undead *and* a mana battery.
* **Crush** (CL10): every monster in line of sight with fewer than 4×CL hit points
  dies outright; others are untouched. It *always* costs you 1d(2×CL) HP. Superb against
  packs of weak monsters at any depth.
* **Annihilate** (CL30) is the reliable single-target damage spell (4×CL, no roll).
  **Curse** (CL40) does (CL/12+1)d(%HP-missing + 50) — best as a finisher on an already
  wounded unique.

**Escape and control.**
* **Shadow Shift** (CL15): teleport 30 squares, costs 2d5 HP. The shadow list has **no
  Teleport Self**, so carry Teleportation scrolls.
* **Sleep Evil** (CL12, power CL×10+500) and **Frighten** (CL17, power CL) for control;
  **Banish Spirits** (CL25) teleports away every monster with a spirit in view.
* **Vampire Strike** (CL20): jump next to the nearest *living* monster and bite for
  min(2×CL, its HP+1), healing yourself that much and feeding you. Doubles as a gap-closer
  — be careful where it takes you.
* **Warg Form** (CL28), **Vampire Form** (CL37) — see §13. Vampire Form costs a quarter
  of your *current* HP (`shape.txt`: `PLAYER_HP / 4`; the spell description says half),
  then jumps and bites for CL; while in the form your melee drains life (`ATT_VAMP`
  10+1d20 turns).
* **Zone of Unmagic** (CL32): disenchantment of power 3×CL on everything within radius 4
  — *including you*.
* **Command** (CL45): take control of a monster for 5+1d10 turns (resisted if
  `1d(your level) < 1d(monster level)`).

**HP-for-power trades.** Crush (−d(2×CL)), Shadow Shift (−2d5), Power Sacrifice
(−50 HP → +50 SP), Curse (−100 HP), Vampire Form (−¼ of current HP). Never cast these
when low.

**Detection.** Read Minds (CL4) detects monsters with minds and maps a small area
around each (±CL/10 rows, ±CL/10+1 columns). Necromancers have no stair or door
detection; use the map from Read Minds, Fume of Mordor (CL40), or Magic Mapping.

**Biggest first-cast exp:** Command 11,250; Storm of Darkness 8,600; Curse 7,200;
Fume of Mordor 6,000; Vampire Form 5,180; Grond's Blow 4,025; Unleash Chaos 3,800;
Zone of Unmagic 3,520; Warg Form 2,240.

---

## 9. Paladin

*Realm divine (WIS) · first spell CL1 · armour limit 40 lb · books 1 town + 2 dungeon ·
flags `BLESS_WEAPON`, `SHIELD_BASH` · **not** zero-fail (5% floor) · prayers learned at
random from the chosen book.*


**[Novice's Handbook]** — town book, realm divine, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Bless | 1 | 2 | 20% | 4 | timed blessed | L1: 11+d11 (avg 17); L20: 30+d30 (avg 45.5); L30: 40+d40 (avg 60.5); L40: 50+d50 (avg 75.5); L50: 60+d60 (avg 90.5) |
| Detect Evil | 3 | 2 | 10% | 12 | detect evil |  |
| Call Light | 5 | 3 | 10% | 10 | light area; sphere of light weak centred on you, radius 1 | L5: 2d2 (avg 3); L20: 2d10 (avg 11); L30: 2d15 (avg 16); L40: 2d20 (avg 21); L50: 2d25 (avg 26) |
| Minor Healing | 7 | 3 | 15% | 28 | heal hp; reduce cut; reduce stun | L7: 17 HP or 1% of wounds; L20: 30 HP or 5% of wounds; L30: 40 HP or 7% of wounds; L40: 50 HP or 10% of wounds; L50: 60 HP or 12% of wounds — 20 — 20 |
| Sense Invisible | 8 | 4 | 25% | 32 | timed sinvis | 24+d24 |
| Heroism | 12 | 5 | 30% | 60 | heal hp; cure afraid; timed hero | 10 — L12: none; L20: 6+d6 (avg 9.5); L30: 16+d16 (avg 24.5); L40: 26+d26 (avg 39.5); L50: 36+d36 (avg 54.5) |

**[Healing and Sanctuary]** — dungeon book, realm divine, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Protection from Evil | 15 | 8 | 42% | 75 | timed protevil | L15: 45+d25 (avg 58); L20: 60+d25 (avg 73); L30: 90+d25 (avg 103); L40: 120+d25 (avg 133); L50: 150+d25 (avg 163) |
| Remove Curse | 20 | 12 | 38% | 160 | remove curse | L20: 20+d20 (avg 30.5); L30: 30+d30 (avg 45.5); L40: 40+d40 (avg 60.5); L50: 50+d50 (avg 75.5) |
| Word of Recall | 25 | 30 | 75% | 250 | recall |  |
| Healing | 30 | 50 | 80% | 3000 | heal hp; cure cut; cure poisoned; cure stun; cure amnesia | 2000 |
| Clairvoyance | 37 | 50 | 80% | 5550 | light level |  |

**[Battle Blessings]** — dungeon book, realm divine, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Smite Evil | 25 | 20 | 70% | 1000 | timed att evil (adds 5 if already active) | 20+d20 |
| Demon Bane | 30 | 40 | 80% | 4500 | timed att demon (adds 5 if already active) | 20+d20 |
| Enchant Weapon | 35 | 50 | 80% | 8050 | enchant toboth | 1d4 |
| Enchant Armour | 37 | 60 | 85% | 9250 | enchant toac | 1+1d3 |
| Single Combat | 40 | 50 | 30% | 12000 | single combat |  |

### 9.1 Tactical notes — Paladin

Paladin prayers are a subset of the Priest's, usually at higher levels and sometimes
higher cost (compare: Minor Healing CL7/3 mana vs Priest CL1/2; Remove Curse CL20/12
mana vs CL16/6; Demon Bane CL30 vs CL42 — the one prayer Paladins get *earlier*). The
Paladin has no attack prayers at all: every prayer supports melee.

* **Bless** (CL1) before every serious fight: +10 to-hit and +5 AC for
  CL+10+1d(CL+10) turns. At CL1 that is 11+1d11 turns for 2 mana.
* **Heroism** (CL12): heal 10, cure fear always; Hero status (+12 to-hit, fear
  immunity) from CL15, lasting (CL−14)+1d(CL−14). This is the Paladin's answer to the
  Warrior's Bravery-30.
* **Minor Healing** (CL7) between fights saves potions; it is too small to matter
  mid-fight after CL20.
* **Protection from Evil** (CL15, [Healing and Sanctuary]) — strong against evil melee
  monsters up to your level (§7.1 for the formula).
* **Healing** (CL30): 2000 HP for 50 mana with 80% base fail — at CL30 with good WIS you
  will see roughly 50% − (stat reduction) fail at first; it improves 3% per level.
  Combined with your HP, it makes the Paladin extremely durable from CL35 on.
* **Smite Evil** (CL25) and **Demon Bane** (CL30): temporary slay evil (×2) and slay demon
  (×5) on your weapon for 20+1d20 turns (+5 if recast while active).
* **Single Combat** (CL40, 50 mana, base fail only 30%): pulls you and the target monster
  into a sealed arena level (`effect_handler_SINGLE_COMBAT()`); the monster resists if
  `randint0(its spell power) > your level`. In the arena teleport, recall and destruction
  don't work — only one of you leaves. Use it on uniques you can out-melee, never on
  one that could kill you.
* **Word of Recall** (CL25) and **Clairvoyance** (CL37) are conveniences.

WIS only needs to be good enough to keep Bless/Heroism/Healing fail low; STR, DEX and
CON matter more. Paladins never drop below 5% fail.

---

## 10. Rogue

*Realm arcane (INT) · first spell **CL5** · armour limit 35 lb · books 1 town + 1
dungeon · flags `CHOOSE_SPELLS`, `STEAL` · **not** zero-fail (5% floor).*


**[First Spells]** — town book, realm arcane, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Detect Monsters | 5 | 1 | 50% | 20 | detect visible monsters |  |
| Phase Door | 7 | 2 | 55% | 28 | teleport self | 10 |
| Object Detection | 10 | 3 | 60% | 50 | detect ore; detect gold; detect objects |  |
| Detect Stairs | 12 | 3 | 50% | 60 | detect stairs |  |
| Recharging | 20 | 10 | 50% | 120 | recharge | L20: 6; L30: 7; L40: 8; L50: 9 |
| Reveal Monsters | 25 | 3 | 40% | 200 | detect invisible monsters; detect visible monsters |  |

**[Arcane Control]** — dungeon book, realm arcane, 4 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Teleport Self | 17 | 6 | 35% | 85 | teleport self | L17: 30% of distance to level edge; L20: 36% of distance to level edge; L30: 54% of distance to level edge; L40: 72% of distance to level edge; L50: 90% of distance to level edge |
| Hit and Run | 23 | 20 | 40% | 460 | timed att run | 10 |
| Teleport Other | 30 | 10 | 30% | 1500 | bolt: away all | L30: 90; L40: 120; L50: 150 |
| Teleport Level | 35 | 17 | 65% | 2800 | teleport level |  |

### 10.1 Tactical notes — Rogue

Rogues read the Mage's books but see only their own spells, at higher levels and
higher base fail (Phase Door: Mage CL2/22%, Rogue CL7/55%). Expect every new rogue spell
to start at or near the 50% cap (§2.4) and improve by 3% per level.

* **Detect Monsters** (CL5, 1 mana) — cheap and the most-cast rogue spell.
* **Phase Door** (CL7) — still worth learning immediately; carry scrolls until its fail
  drops (at CL20 with INT 15 it is about 14%).
* **Object Detection** (CL10): real detection of objects and gold (`DETECT_GOLD`,
  `DETECT_OBJECTS`, plus ore) in a 22×40 window — better than the Mage's
  Treasure Detection, which only *senses* objects. The rogue's loot radar.
* **Detect Stairs** (CL12): stairs in a **44-row × 60-column** window
  (`effect-yx:44:60`) — wider than any other stair detection. Combine with high
  stealth to dive quickly.
* **Recharging** (CL20): strength CL/10+4 (weaker than the Mage's CL/8+6).
* **Reveal Monsters** (CL25): visible and invisible monsters.
* [Arcane Control] (DL40+): **Teleport Self** (CL17 — but you can't cast it until you
  find this dungeon book), **Hit and Run** (CL23: for 10 turns, your next steal attempt
  is followed by a 20-square teleport, `steal_monster_item()`), **Teleport Other**
  (CL30, distance 3×CL), **Teleport Level** (CL35).

**Stealing and spells together.** The intended pattern: Detect Monsters → sneak up to a
sleeping monster that carries items → cast Hit and Run → `s`teal → teleport away
whatever the result. The steal formula is in Chapter 2 §5.7: sleeping monsters halve
their guard, and your speed advantage over the monster lowers it further.

---

## 11. Ranger

*Realm nature (WIS) · first spell **CL3** · armour limit 40 lb · books 1 town + 1
dungeon · flags `FAST_SHOT`, `CHOOSE_SPELLS` · **not** zero-fail (5% floor).*


**[Lesser Charms]** — town book, realm nature, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Remove Hunger | 3 | 1 | 25% | 6 | nourish inc to | 50 |
| Detect Life | 5 | 2 | 23% | 10 | detect living monsters |  |
| Herbal Curing | 9 | 6 | 30% | 36 | cure cut; cure poisoned; cure stun; cure blackbreath |  |
| Resist Poison | 12 | 10 | 32% | 60 | timed opp pois | 20+d20 |
| Turn Stone to Mud | 15 | 5 | 25% | 75 | line (beam) of kill wall | 20+1d30 |
| Sense Surroundings | 20 | 8 | 35% | 160 | map area | L20: ±22 rows, ±44 cols; L30: ±44 rows, ±88 cols; L40: ±44 rows, ±88 cols; L50: ±44 rows, ±88 cols |

**[Nature Craft]** — dungeon book, realm nature, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Cover Tracks | 20 | 20 | 40% | 800 | timed covertracks (adds 5 if already active) | L20: 40+d20 (avg 50.5); L30: 60+d20 (avg 70.5); L40: 80+d20 (avg 90.5); L50: 100+d20 (avg 110.5) |
| Create Arrows | 22 | 20 | 40% | 880 | create arrows |  |
| Haste Self | 25 | 12 | 65% | 375 | timed fast (adds 5 if already active) | L25: 25+d20 (avg 35.5); L30: 30+d20 (avg 40.5); L40: 40+d20 (avg 50.5); L50: 50+d20 (avg 60.5) |
| Decoy | 30 | 30 | 40% | 1800 | glyph decoy |  |
| Brand Ammunition | 40 | 60 | 95% | 4800 | brand ammo |  |

### 11.1 Tactical notes — Ranger

The Ranger's spells are survival tools; the bow does the killing.

* **Remove Hunger** (CL3, 1 mana): no food needed.
* **Detect Life** (CL5): living monsters only — undead, golems and other non-living
  monsters do not show.
* **Herbal Curing** (CL9): cures cuts, poison, stun and Black Breath (the Druid version
  at CL40 also feeds you). Very good value early: stun and poison stop HP regeneration.
* **Resist Poison** (CL12), **Turn Stone to Mud** (CL15), **Sense Surroundings** (CL20).
* [Nature Craft] (DL40+):
  * **Cover Tracks** (CL20): 2×CL+1d20 turns during which you leave no scent and monsters
    more than 5 squares away (max sight 20 / 4) cannot see you; it ends as soon as you
    damage a monster (`mon_take_hit()` clears it).
  * **Create Arrows** (CL22): destroys a staff (from pack or floor) and creates a stack
    of arrows generated at your level; the arrows are "good" if `1d(staff level) > 25`
    and "great" if additionally `1d(staff level) > 50` (`effect_handler_CREATE_ARROWS()`).
    Feed it high-level junk staffs.
  * **Haste Self** (CL25): +10 speed for CL+1d20 turns (+5 per recast while active).
  * **Decoy** (CL30): a glyph that looks like you; monsters target it instead of you
    until it is destroyed.
  * **Brand Ammunition** (CL40): one stack gets a random brand — flame, frost or venom,
    1/3 each (`effect_handler_BRAND_AMMO()`) — plus an enchantment attempt; no effect on
    ammo that is already branded/slaying, broken or cursed.

**Extra shots** (`FAST_SHOT`): +0.1 shots per 3 levels → 1.5 shots at CL15, 2.0 at
CL30, 2.6 at CL48. Haste Self and extra shots together make a midgame Ranger one of the
fastest killers in the game.

---

## 12. Blackguard

*Realm shadow (INT) · first spell CL1 · armour limit **60 lb** · books 2 town + 1
dungeon · flags `CHOOSE_SPELLS`, `SHIELD_BASH`, `COMBAT_REGEN`; `IMPAIR_HP` · **not**
zero-fail (5% floor).*


**[Into the Shadows]** — town book, realm shadow, 6 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Seek Battle | 1 | 1 | 15% | 2 | detect fearful monsters |  |
| Berserk Strength | 3 | 2 | 20% | 12 | cure afraid; timed shero (adds 5 if already active) | L3: 8+d8 (avg 12.5); L20: 25+d25 (avg 38); L30: 35+d35 (avg 53); L40: 45+d45 (avg 68); L50: 55+d55 (avg 83) |
| Whirlwind Attack | 5 | 4 | 25% | 30 | sweep | L5: 1; L20: 2; L30: 2; L40: 3; L50: 4 |
| Shatter Stone | 7 | 3 | 20% | 56 | short beam of kill wall, length 1 | 20+1d30 |
| Leap into Battle | 10 | 5 | 30% | 100 | move attack | L10: 1; L20: 1; L30: 2; L40: 3; L50: 3 |
| Grim Purpose | 13 | 8 | 30% | 156 | timed opp conf; timed free act | shared duration 12+d12 |

**[Fear and Torment]** — town book, realm shadow, 5 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Maim Foe | 15 | 6 | 35% | 210 | melee blows + mon stun side-effect | L15: 1; L20: 1; L30: 2; L40: 2; L50: 3 |
| Howl of the Damned | 18 | 8 | 30% | 288 | turn all on every monster in line of sight | L18: 18; L20: 20; L30: 30; L40: 40; L50: 50 |
| Relentless Taunting | 24 | 10 | 32% | 480 | timed taunt | 12+d12 |
| Venom | 28 | 18 | 35% | 840 | timed att pois (adds 5 if already active) | 18+d18 |
| Werewolf Form | 32 | 30 | 40% | 3200 | become werewolf |  |

**[Deadly Powers]** — dungeon book, realm shadow, 4 spells for this class

| Spell | Lvl | Mana | Base fail | First-cast exp | Effect(s) | Dice / value by character level |
|---|---|---|---|---|---|---|
| Bloodlust | 30 | 25 | 37% | 2400 | timed bloodlust (adds -1 if already active) | 10 |
| Unholy Reprieve | 34 | 45 | 60% | 4080 | restore str; restore int; restore con; restore exp; damages you | 66 |
| Forceful Blow | 38 | 16 | 50% | 5700 | melee blows + force side-effect | 1 |
| Quake | 44 | 39 | 55% | 7700 | earthquake radius 13 around you |  |

### 12.1 Combat regeneration (how Blackguards get mana)

Blackguards do not regenerate mana by resting. From `src/player-util.c` and
`src/player-attack.c`:

* **Taking damage** gives SP: losing X% of your max HP gives X% of your max SP
  (max SP treated as at least 10). Damage from poison, bleeding ("a fatal wound") and
  starvation doesn't count; traps and lava do.
* **Each melee attack command** gives 5% of max SP (min. base 10 → at least 0.5 SP).
* **SP decays** over time at half the normal regeneration rate, and decaying SP heals
  you at twice the casting rate.
* **Casting** a spell heals you: spending X% of max SP restores X/2 % of your *missing*
  HP, at most 25% of missing HP per cast (`convert_mana_to_hp()`).
* For a Blackguard the regeneration rate is made **negative** (`percent /= -2` in
  `player_regen_mana()`), so SP always drains toward 0. While your HP is at or below
  half, Regeneration and resting each double that drain — and since drained SP is
  converted into healing, resting while badly hurt and holding SP heals you faster.
  Above half HP they have no effect.
* `IMPAIR_HP`: normal HP regeneration is halved.

### 12.2 Tactical notes — Blackguard

* **Berserk Strength** (CL3, 2 mana): cures fear and gives berserk rage for
  (CL+5)+1d(CL+5) turns (+5 per recast while active): +75 melee skill, −10 AC, fear
  immunity, device skill −10% (`calc_bonuses()`, `TMD_SHERO`). The spell description
  also says it "prevents healing over time", but no such check exists in 4.2.6's
  `player_regen_hp()`. Your bread-and-butter buff: cast it as soon as the first hit gives
  you 2 SP.
* **Whirlwind Attack** (CL5): (CL+10)/15 rounds of one blow against *every* adjacent
  monster — 1 at CL5, 2 at CL20, 3 at CL35, 4 at CL50. Fight in the open against groups
  only when you can afford it; otherwise in a room corner.
* **Shatter Stone** (CL7): destroys the adjacent wall/rubble section; 20+1d30 to
  rock-vulnerable monsters. Digging without a digger.
* **Leap into Battle** (CL10): run up to 4 squares toward the target and attack with
  (CL+5)/15 blows (1 at CL10, 2 at CL25, 3 at CL40), reduced to
  `(blows × moves_left + 2)/4` — i.e. 25% fewer per square moved. Stops early if a
  monster is in the way. A gap-closer for archers and casters that keep their distance.
* **Grim Purpose** (CL13): 12+1d12 turns of temporary Free Action *and* confusion
  protection (one shared roll) — the Blackguard's paralysis insurance before permanent
  Free Action.
* **Maim Foe** (CL15): CL/15 blows (1, 2 at CL30, 3 at CL45); any blow that damages also
  stuns the target (power 6).
* **Howl of the Damned** (CL18): fear on everything in line of sight (power CL).
  Frightened monsters flee and don't melee — use it to break up a crowd, or as an escape.
* **Relentless Taunting** (CL24): 12+1d12 turns; monsters are drawn toward you and their
  spell-casting chance is halved (`src/mon-attack.c`). Excellent against packs of
  archers/casters.
* **Venom** (CL28): poison brand (×3, `POIS_3`) on your weapon for 18+1d18 turns.
* **Werewolf Form** (CL32): +1 blow, faster moves, Regeneration (cancels `IMPAIR_HP`),
  +5 to-hit/to-dam; on assuming, scares everything in view (power CL), cures fear and
  gives 5+1d20 turns of berserk.
* [Deadly Powers] (DL40+):
  * **Bloodlust** (CL30): see Chapter 2 §5.9 — up to +25 to-dam and +2 blows, death
    protection, with serious side-effects. Casting while active reduces it by 1; kills
    are what build it.
  * **Unholy Reprieve** (CL34): restores STR, INT, CON and experience for 66 HP.
  * **Forceful Blow** (CL38): one melee blow; if it damages, a *force* projection of
    power 200 hits the target (`effect_handler_MELEE_BLOWS()` uses the effect's radius,
    200, as damage) — knockback and a stun chance.
  * **Quake** (CL44): earthquake of radius 13 around you.

---

## 13. Shapechange reference

All data from `lib/gamedata/shape.txt`; the `combat:` line is to-hit : to-dam : AC.
Shapes are applied by `calc_shapechange()` in `src/player-calcs.c` **in addition to**
everything your equipment gives. `MOVES[n]` makes walking cost
`100/(1 + n)` energy for positive n (`energy_per_move()`): ½ turn for +1, ¼ turn for +3.

| Shape | Spell (class, CL, mana) | To-hit / to-dam / AC | Stats | Speed, blows, moves | Skills | Flags | Resists / other | On assuming |
|---|---|---|---|---|---|---|---|---|
| fox | Fox Form (Druid 1, 2) | −3 / −3 / +3 | STR −3 | +1 blow, +1 move | stealth +5 | Feather Fall, Free Action | — | — |
| bat | Bat Form (Necro 3, 4) | 0 / −10 / +10 | — | speed +3 | stealth +3, infra +5 | Feather Fall, See Invisible, blind-proof | — | you take 5 damage |
| warg | Warg Form (Necro 28, 20) | +5 / +5 / 0 | — | +2 blows | — | — | — | you take 1d(CL) damage; cure fear; berserk 5+1d20 turns |
| Pukel-man | Become Pukel-man (Druid 30, 20) | 0 / +5 / +20 | STR +4, CON +4 | speed −5 | save +20, stealth −2 | sustain STR & CON, stun-proof, Regeneration, Hold Life, `ROCK` | **poison immunity**, resist shards, 10 damage reduction | cure poison and stun |
| werewolf | Werewolf Form (Blackguard 32, 30) | +5 / +5 / 0 | — | +1 blow, +1 move | — | Regeneration | — | scare every monster in view (power CL); cure fear; berserk 5+1d20 turns |
| eagle | Eagle's Flight (Druid 35, 20) | +5 / 0 / +10 | — | +3 moves | — | sustain WIS, blind-, confusion- and fear-proof, See Invisible, Free Action, trap immunity | — | — |
| vampire | Vampire Form (Necro 37, 20) | +3 / +3 / −5 | — | speed +5 | stealth +5, infra +5, light −1 | See Invisible, Hold Life | — | you lose ¼ of current HP; life-draining melee (`ATT_VAMP`) 10+1d20 turns; the spell then jump-bites the nearest living monster for CL |
| bear | Bear Form (Druid 40, 20) | +15 / +15 / +5 | STR +3, INT −2, CON +2 | +1 blow | melee +10, disarm phys −5, disarm magic −10, stealth −3, infra +1 | fear-proof | — | — |

`ROCK` (Pukel-man): cuts neither hurt nor heal (`process_world()` / timed-effect decrease
in `src/game-world.c`: "Rock players just maintain").

**Rules while shapechanged** (`src/player-util.c`, `src/cmd-core.c`, `src/cmd-obj.c`):
* Melee still uses your wielded weapon's dice and bonuses; only the attack verbs change
  (`py_attack_real()`), and shape blows/to-hit/to-dam are added.
* You cannot read, quaff, aim, use, zap, activate, fire, throw, wield, cast or study;
  item prompts only see the floor. Trying any of these asks "Change back and … (y/n) or
  (r)eturn to normal?" — `y` reverts **and** performs the action in one turn; `r` only
  reverts.
* Shops slam their doors on you (except your home).
* Vampire Form's life-drain timer ends when you revert.

**When to use them.**
* *Fox*: early scouting and running away (half-cost moves). Its Free Action is real.
* *Bat*: Necromancer scouting at low level; speed +3 and See Invisible, but −10 to-dam
  makes it useless for fighting.
* *Warg*: Necromancer melee burst (+2 blows, berserk) — pay 1d(CL) HP.
* *Pukel-man*: walk through poison breathers and shard storms; −5 speed is the cost.
* *Eagle*: crossing levels at quadruple walking speed, trap immunity, and an escape
  that works without reading anything.
* *Bear*: the Druid's late-game melee form.
* *Werewolf*: the Blackguard's fight-opening form — the free mass scare plus berserk
  plus Regeneration.
* *Vampire*: Necromancer melee with life drain; budget the ¼-HP cost.

---

## 14. Comparative quick reference

### 14.1 Escape spells

| Class | Short hop | Teleport | Other / level | Notes |
|---|---|---|---|---|
| Mage | Phase Door (CL2, 10 sq.) | Teleport Self (CL7, CL×9/5 % of edge distance) | Teleport Other (CL15), Teleport Level (CL28*), Dimension Door (CL35*) | * dungeon book |
| Rogue | Phase Door (CL7) | Teleport Self (CL17*) | Hit and Run (CL23*), Teleport Other (CL30*), Teleport Level (CL35*) | * [Arcane Control] |
| Priest | — | Portal (CL10*, CL/2+30 sq.) | Banish Evil (CL25*) | * dungeon books |
| Necromancer | — | Shadow Shift (CL15, 30 sq., −2d5 HP) | Banish Spirits (CL25*) | Bat/Vampire forms are fast |
| Druid | — | — | Eagle's Flight (CL35*), Mass Sleep (CL25*) | no teleport spell at all |
| Paladin, Ranger, Blackguard | — | — | Howl of the Damned (BG CL18), Decoy (Ranger CL30*) | rely on scrolls/staves |

### 14.2 Healing and cures by spell

| Class | Spell | Heals | Cures |
|---|---|---|---|
| Priest | Minor Healing (CL1) | CL+10 or CL/4 % of wounds | −20 cuts, −20 stun |
| Priest | Healing (CL30*) | 2000 | cuts, poison, stun, amnesia |
| Priest | Holy Word (CL39*) | 1000 | fear, poison, stun, cuts |
| Priest | Heroism (CL5) | 10 | fear |
| Paladin | Minor Healing (CL7), Heroism (CL12), Healing (CL30*) | as Priest | as Priest |
| Druid | Rapid Regeneration (CL37*) | 30/turn for 5+1d3 turns | — |
| Druid | Herbal Curing (CL40*) / Cure Poison (CL4) | — | cuts, poison, stun, Black Breath / poison |
| Ranger | Herbal Curing (CL9) | — | cuts, poison, stun, Black Breath |
| Necromancer | Vampire Strike (CL20) | min(2×CL, target HP+1) | — (feeds you) |
| Blackguard | any spell | X/2 % of missing HP per X% of SP spent (≤25%) | Berserk Strength, Grim Purpose (fear/confusion/paralysis protection) |

### 14.3 Detection by spell

| Class | Monsters | Objects/treasure | Map/stairs/doors/traps |
|---|---|---|---|
| Mage | Detect Monsters (CL3), Reveal Monsters (CL15, + invisible), Detection (CL30*) | Treasure Detection (CL10, senses objects), Detection | Find Traps/Doors/Stairs (CL1), Detection |
| Rogue | Detect Monsters (CL5), Reveal Monsters (CL25) | Object Detection (CL10, full detection) | Detect Stairs (CL12, 44×60 area) |
| Priest | Detect Evil (CL1) | Clairvoyance (CL37*) | Clairvoyance (whole level) |
| Paladin | Detect Evil (CL3) | Clairvoyance (CL37*) | Clairvoyance |
| Druid | Detect Life (CL1) | — | Sense Surroundings (CL9) |
| Ranger | Detect Life (CL5) | — | Sense Surroundings (CL20) |
| Necromancer | Read Minds (CL4, monsters with minds) | Fume of Mordor (CL40*) | Read Minds (local), Fume of Mordor (whole level) |
| Blackguard | Seek Battle (CL1, fear-susceptible monsters) | — | — |

Detection windows: most detections use `effect-yx:22:40` — 22 rows above/below and
40 columns either side of you; Rogue's Detect Stairs uses 44:60.

### 14.4 Best single-cast damage at CL50 (averages)

| Class | Spell | Average damage | Mana | Type |
|---|---|---|---|---|
| Mage | Mana Storm | 400 | 16 | ball r3, unresistable |
| Mage | Thrust Away | 225 | 12 | short beam, force |
| Mage | Mana Bolt | 180 | 8 | bolt/beam, unresistable |
| Druid | River of Lightning | 270 | 35 | 20° arc, plasma |
| Druid | Rift | 240 | 20 | beam, gravity |
| Druid | Ice Storm | 226.5 per monster in LOS | 25 | ice |
| Druid | Meteor Swarm | 55 × 4 | 14 | meteors, unresistable |
| Priest | Light of Varda | 250 (+100 to light-sensitive in LOS) | 40 | light ball r4 |
| Priest | Spear of Oromë | 112.5 (×2 vs evil) | 10 | holy beam |
| Priest | Orb of Draining | 85.5 (×2 vs evil) | 7 | holy ball r3 |
| Necromancer | Unleash Chaos | 204 | 15 | random chaos beam/arc/ball |
| Necromancer | Storm of Darkness | 202 | 16 | darkness ball r4 |
| Necromancer | Annihilate | 200 (living only) | 20 | bolt |
| Necromancer | Crush | kills anything in LOS under 200 HP | 6 | — |
| Blackguard | Forceful Blow | 1 blow + 200 force | 16 | melee |

### 14.5 Spells worth learning first, by class

| Class | First picks |
|---|---|
| Mage | Magic Missile, Find Traps/Doors/Stairs, Phase Door, Detect Monsters, Fire Ball, then Teleport Self / Frost Bolt / Teleport Other |
| Druid | Stinking Cloud, Fox Form, Detect Life, Remove Hunger, Confuse/Slow Monster, then Stone to Mud, Lightning Strike |
| Priest (random) | Study [Novice's Handbook] until CL7, then [Cleansing Power] for Orb of Draining |
| Necromancer | Nether Bolt, Create Darkness, Read Minds, Bat Form, then Tap Unlife, Crush, Shadow Shift |
| Paladin (random) | Only one town book: you will learn all of [Novice's Handbook] by CL12 |
| Rogue | Detect Monsters, Phase Door, Object Detection, Detect Stairs |
| Ranger | Remove Hunger, Detect Life, Herbal Curing, Resist Poison |
| Blackguard | Berserk Strength, Whirlwind Attack, Seek Battle, Leap into Battle, Grim Purpose |


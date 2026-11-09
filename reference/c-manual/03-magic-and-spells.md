# 3. Magic and Spells

Everything about spellcasting in Angband 4.2.x (upstream master `e473720ed`): the four realms, how
spells are learned, the exact failure and mana formulas, every book and where it comes from, and
every spell of every casting class with its resolved numbers, alternatives and advice. It ends with
a cross-class "I need X" lookup and a casting decision procedure for an automated player.

**Contents**

1. [Magic in one screen](#1-magic-in-one-screen)
2. [Realms, casting stats and class magic parameters](#2-realms-casting-stats-and-class-magic-parameters)
3. [Learning spells](#3-learning-spells)
4. [Casting: requirements, energy, over-exertion](#4-casting-requirements-energy-over-exertion)
5. [Spell failure](#5-spell-failure)
6. [Mana: maximum, encumbrance, regeneration](#6-mana-maximum-encumbrance-regeneration)
7. [How spell effects work (geometry, dice, monster saves)](#7-how-spell-effects-work)
8. [Books: town vs dungeon, prices, depths, who uses which](#8-books)
9. [Mage](#9-mage)
10. [Druid](#10-druid)
11. [Priest](#11-priest)
12. [Necromancer](#12-necromancer)
13. [Paladin](#13-paladin)
14. [Rogue](#14-rogue)
15. [Ranger](#15-ranger)
16. [Blackguard](#16-blackguard)
17. [Shapechanges](#17-shapechanges)
18. [Cross-class lookup: "I need X"](#18-cross-class-lookup-i-need-x)
19. [Casting decision procedure for a borg](#19-casting-decision-procedure-for-a-borg)
20. [Things to try (untested)](#things-to-try-untested)

Conventions: **L** = character level, **SP** = spell points (mana). "Fail" in spell tables is the
*base* fail from `lib/gamedata/class.txt`; your real chance is computed by the formula in
[section 5](#5-spell-failure). "XP" is the experience for the *first* successful cast
(= `exp` field × spell level, see [3.4](#34-first-cast-experience)). Dice expressions such as
`$Dd4` with `expr:D:PLAYER_LEVEL:- 1 / 5 + 3` are evaluated **strictly left to right with integer
division** (`src/z-expression.c:expression_evaluate()`), so `- 1 / 5 + 3` means `(L-1)/5 + 3`.
Full raw spell tables are generated in [Appendix A7](appendices/A7-class-spells.md); this chapter
interprets them. Warriors have no spells (`NO_MANA`).

The official manual covers the commands (`b`rowse, `G`ain a spell, `m` cast) in
[command](https://angband.readthedocs.io/en/latest/command.html) (vendored:
[user-manual/command.rst](user-manual/command.rst)) and the class descriptions in
[birth](https://angband.readthedocs.io/en/latest/birth.html) ([user-manual/birth.rst](user-manual/birth.rst)).
It says only that failure "starts out fairly large but decreases as you gain levels", that casting
without enough mana risks paralysis and CON damage, and that you need light (except necromancers)
and must not be blind or confused. Everything below goes further, with the numbers.

---

## 1. Magic in one screen

FACT (all from `src/player-spell.c`, `src/player-calcs.c`, `src/cmd-obj.c`):

- Each casting class uses one **realm** (arcane/INT, divine/WIS, nature/WIS, shadow/INT). The realm
  stat drives number of spells, max mana and failure.
- **Spells allowed** = `(adj_mag_study[stat] × (L − first_spell_level + 1) + 50) / 100`. You pick the
  spell (`CHOOSE_SPELLS` classes) or get a random one from the chosen book (Priest, Paladin).
- **Max SP** = `1 + adj_mag_mana[stat] × (L − first_spell_level + 1) / 100`, minus 1 SP per pound of
  armour above the class weight limit. **Gloves do not matter in this version.**
- **Fail** = base − 3×(L − spell level) − adj_mag_stat[stat], (+5 per missing SP, +25 on a lit square
  for necromancers, +20 if afraid), floored at the stat minimum (5% minimum for classes without
  `ZERO_FAIL`), **capped at 50%**, then +15/+25 if stunned, then amnesia `50 + chance/2`, final cap 95%.
- Casting costs one full turn (100 energy; 75 under *Mana Channel*). You cannot cast while blind,
  confused, or (non-necromancers) without light on your square, and you cannot cast while
  shapechanged without first reverting.
- **First successful cast** of each spell gives `exp × spell_level` experience — cast every new spell
  once as soon as it is safe.
- Mana regenerates per 10 game turns (not per player turn): ~0.30% of max SP each normal-speed turn,
  doubled by resting and again by *Regeneration*.

---

## 2. Realms, casting stats and class magic parameters

FACT: `lib/gamedata/realm.txt` defines four realms:

| Realm | Stat | Verb | Spell noun | Book noun |
|---|---|---|---|---|
| arcane | INT | cast | spell | magic book |
| divine | WIS | recite | prayer | prayer book |
| nature | WIS | chant | verse | nature book |
| shadow | INT | perform | ritual | shadow book |

FACT: class magic parameters (`magic:first:weight:books` and `player-flags` in `lib/gamedata/class.txt`):

| Class | Realm (stat) | First spell level | Armour weight limit | Books | Spells | Relevant flags |
|---|---|---|---|---|---|---|
| Mage | arcane (INT) | 1 | 30 lb | 5 | 30 | `ZERO_FAIL`, `BEAM`, `CHOOSE_SPELLS` |
| Druid | nature (WIS) | 1 | 35 lb | 5 | 27 | `ZERO_FAIL`, `CHOOSE_SPELLS`, `CHARM` |
| Priest | divine (WIS) | 1 | 35 lb | 5 | 28 | `ZERO_FAIL`, `BLESS_WEAPON` (random learning) |
| Necromancer | shadow (INT) | 1 | 30 lb | 5 | 26 | `ZERO_FAIL`, `CHOOSE_SPELLS`, `UNLIGHT`, `EVIL` |
| Paladin | divine (WIS) | 1 | 40 lb | 3 | 16 | `BLESS_WEAPON`, `SHIELD_BASH` (random learning) |
| Rogue | arcane (INT) | 5 | 35 lb | 2 | 10 | `CHOOSE_SPELLS`, `STEAL` |
| Ranger | nature (WIS) | 3 | 40 lb | 2 | 11 | `CHOOSE_SPELLS`, `FAST_SHOT` |
| Blackguard | shadow (INT) | 1 | 60 lb | 3 | 15 | `CHOOSE_SPELLS`, `COMBAT_REGEN`, `SHIELD_BASH`; `IMPAIR_HP` |
| Warrior | — | — | — | 0 | 0 | `NO_MANA` |

What the magic flags do (FACT, with source):

- `ZERO_FAIL` — your minimum fail can drop below 5% down to the stat table minimum (0% at 18/200+).
  Without it the floor is 5% (`spell_chance()`). Paladins, Rogues, Rangers and Blackguards always
  keep at least 5% failure.
- `BEAM` — beam chance for `BOLT_OR_BEAM` spells is L% instead of L/2% (`beam_chance()`). Only Mage
  spells use `BOLT_OR_BEAM`, so in practice this is a Mage feature.
- `CHOOSE_SPELLS` — `G` lets you pick the spell; without it `G` asks for a book and learns a random
  eligible spell from it (`do_cmd_study_book()`).
- `CHARM` (Druid) — conf/slow/hold/sleep/polymorph *durations* against `ANIMAL` monsters are ×1.5
  (`src/project-mon.c`).
- `UNLIGHT` (Necromancer) — +25% fail when *your* square is lit; see in the dark without light;
  resist dark in the dungeon; can cast without light ([section 12](#12-necromancer)).
- `EVIL` (Necromancer) — resist nether, vulnerable to holy orb (`calc_bonuses()`).
- `COMBAT_REGEN` (Blackguard) — mana comes from fighting and drains away at rest
  ([section 16](#16-blackguard)).

The same physical book can carry **different spells for different classes**: e.g. Mage, Rogue
both read [First Spells] but see different lists; Necromancer and Blackguard share the shadow books;
Priest and Paladin share prayer books; Druid and Ranger share nature books. A book is only usable by
a class if `class.txt` lists it for that class.

---

## 3. Learning spells

### 3.1 How many spells you may know

FACT (`src/player-calcs.c:calc_spells()`):

```
levels       = L - first_spell_level + 1          (0 if negative)
num_allowed  = (adj_mag_study[stat_index] * levels + 50) / 100
new_spells   = num_allowed - spells_known          (capped at the number of spells
                                                     of level <= L that you don't know)
```

For classes with two realms the stat is averaged (none exist in 4.2: every class has one realm).
`adj_mag_study` (spells per 100 levels) by casting-stat value:

| Stat | 8 | 10 | 12 | 14 | 16 | 17 | 18/00 | 18/20 | 18/50 | 18/70 | 18/80 | 18/90 | 18/100 | 18/120 | 18/150+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| spells/level ×100 | 40 | 60 | 80 | 90 | 100 | 105 | 110 | 120 | 150 | 170 | 180 | 190 | 200 | 220 | 250 |

Consequences:

- With a casting stat of 16 you get exactly one new spell per level; at 18/50, 1.5; at 18/150+, 2.5.
- Spells are only "short" at low level with a low stat. A Mage with INT 13 (85) at L5 may know
  `(85×5+50)/100 = 4` spells while 8 are available — choose carefully. By mid-game every caster with a
  decent stat can know every spell of its level; at L50 a stat of 16 already allows 50 spells, more
  than any class has (max 30, Mage).
- **Stat drain makes you forget spells.** If `num_allowed` drops below what you know, the most
  recently learned spells are marked *forgotten* (they cannot be cast). Restoring the stat restores
  them automatically, in learning order ("You have remembered..."). Spells above your level are
  also forgotten (only possible after experience drain lowers your level).
- The "Study" indicator on the status bar means `new_spells > 0` and a book in your pack has a
  learnable spell.

### 3.2 Choosing (`CHOOSE_SPELLS`) vs random (Priest, Paladin)

FACT (`src/cmd-obj.c:do_cmd_study_spell()`, `do_cmd_study_book()`):

- Choosers press `G` (or browse and select) and pick any spell with level ≤ L that they do not know,
  from any carried (or floor) book. Learning takes one turn.
- Priests and Paladins pick only the **book**; the game picks uniformly among that book's learnable
  spells (reservoir sampling over eligible spells). The trick (ADVICE): if you want a particular
  prayer, study from the book in which it is the *only* eligible prayer, or delay studying until the
  book you want has few eligible prayers. Because study slots are never lost, delaying a study costs
  nothing except the time the spell is unavailable.
- You must be able to cast (not blind/confused, have light) to study.

### 3.3 Which spell to learn first (general rules)

ADVICE, valid for all choosers:

1. An attack spell you can afford to spam (Magic Missile, Nether Bolt, Stinking Cloud).
2. An escape (Phase Door; Shadow Shift; Fox Form) and a detection (Find Traps/Doors/Stairs, Detect
   Monsters, Detect Life, Read Minds, Seek Battle).
3. Anything that saves consumables you'd otherwise buy every trip (Remove Hunger, Light Room,
   Identify Rune, Recharging).
4. Skip spells you will never cast (see each class's "traps" list); with a good stat you'll get them
   later anyway for the first-cast experience.

### 3.4 First-cast experience

FACT (`src/player-spell.c:spell_cast()`): the first time a spell *succeeds* you gain
`sexp × slevel` experience (raw points; your race/class experience factor applies to the level
thresholds, not to the gain). Notable values:

| Spell (class) | Level | XP for first cast |
|---|---|---|
| Fox Form (Druid) | 1 | 10 (= the whole of level 2 for a 100% expfact character) |
| Magic Missile + Light Room + Find Traps (Mage, L1) | 1 | 4 + 4 + 2 = 10 |
| Identify Rune (Mage) | 8 | 64 |
| Resistance (Mage) | 20 | 400 |
| Banish Evil (Priest) | 25 | 6,250 |
| Mana Storm (Mage) | 45 | 9,000 |
| Enchant Armour (Priest/Paladin) | 37 | 9,250 |
| Command (Necromancer) | 45 | 11,250 |
| Single Combat (Paladin) | 40 | 12,000 |
| Demon Bane (Priest) | 42 | 12,600 |

ADVICE: cast each newly learned spell once at the first safe moment (in town or a cleared corridor);
for utility spells with no valid target (Recharging, Identify Rune, Enchant) you need a valid target
item for the effect to complete — the experience is only given when the effect actually runs
(`effect_do()` returning false, e.g. on cancelling the item prompt, aborts before the reward — and
before the SP cost, so a cancelled cast costs neither SP nor a turn).

---

## 4. Casting: requirements, energy, over-exertion

FACT:

- **Book**: the book must be in your pack or on the floor under you (`src/ui-spell.c`, `USE_INVEN | USE_FLOOR`).
- **Can cast** (`src/player-util.c:player_can_cast()`): not blind, not confused, and your own square
  must be "seen" (`no_light()`); an `UNLIGHT` character always sees its own square, so necromancers
  can cast in total darkness. There is **no spell that cures blindness or confusion** usable while
  blind or confused — carry potions (CLW cures blindness; CSW/CCW cure confusion; see
  [Chapter 7](07-consumables-and-devices.md)).
- **Shapechanged**: casting, studying, reading, quaffing, using devices, wielding, firing and
  throwing all prompt "Change back and ... (y/n) or (r)eturn to normal?". Answering `y` reverts
  (free) and continues the command (`player_get_resume_normal_shape()`).
- **Energy**: a cast costs `move_energy` (100); with `FASTCAST` (Mage *Mana Channel*) 75
  (`do_cmd_cast()`). A failed cast still costs the turn **and the mana**.
- **Not enough mana**: you are asked to confirm. Fail chance rises by 5% per missing SP (before the
  50% cap). Whether it succeeds or not, your SP become 0 and you are **paralysed for 1d(5×missing+1)
  turns — this bypasses Free Action** — and have a 50% chance of CON damage, permanent with 25%
  probability (`player_over_exert()`). Never do this with monsters near; it is only a deliberate
  "last gamble" when the alternative is certain death.
- **Blackguards** heal a little whenever they cast (`convert_mana_to_hp()`).
- A cast does **not** disturb or wake monsters by itself beyond normal noise; some spells explicitly
  wake (*Unleash Chaos* `WAKE`; *Command*), and damage wakes the target.

---

## 5. Spell failure

FACT (`src/player-spell.c:spell_chance()`), in order:

```
chance  = base_fail                                  (class.txt)
chance -= 3 * (L - spell_level)
chance -= adj_mag_stat[casting stat]
if spell_mana > current SP:  chance += 5 * (spell_mana - current SP)
minfail = adj_mag_fail[casting stat]
if class lacks ZERO_FAIL and minfail < 5:  minfail = 5
if UNLIGHT and own square is lit:  chance += 25
if afraid:  chance += 20
chance = clamp(chance, minfail, 50)                  <-- the 50% cap
if stun > 50 (heavy stun): chance += 25  else if stunned: chance += 15
if amnesia: chance = 50 + chance/2
chance = min(chance, 95)
```

Casting-stat tables (selected rows; `adj_mag_stat` reduces fail, `adj_mag_fail` is the floor):

| Stat | 8 | 10 | 12 | 14 | 16 | 17 | 18/00 | 18/20 | 18/50 | 18/70 | 18/80 | 18/90 | 18/100 | 18/120 | 18/150 | 18/170 | 18/200+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fail reduction | −1 | 0 | 0 | 1 | 3 | 4 | 5 | 7 | 10 | 12 | 15 | 18 | 21 | 27 | 36 | 42 | 51 (57 at 18/220) |
| minimum fail % | 50 | 20 | 12 | 10 | 8 | 7 | 6 | 5 | 4 | 4 | 4 | 3 | 3 | 2 | 1 | 1 | 0 |

Key consequences:

- **The 50% cap is the most important number.** Any spell whose computed fail is above 50% shows and
  behaves as exactly 50% (unless stunned/amnesiac). A L35 Mage learning *Banishment* (base 95) is at
  50%, not 95%; *Holy Word* (95) is 50% for a fresh L39 Priest. High-base spells are usable
  emergency tools from the moment you learn them, at coin-flip odds. Don't rely on them when a
  failure kills you; do use them when a 50% chance is better than any alternative.
- Each character level above the spell level is −3%: a spell learned at its level with base 40 and
  stat 18/50 starts at 30% and reaches the floor (4%) about 9 levels later.
- Fear adds 20% *before* the cap. The anti-fear spells have low base fail so they still work while
  afraid (Heroism 30, Berserk Strength 20).
- Stun adds 15% (25% heavy) *after* the cap: a stunned caster may be at 65–75%. Cure stun first
  (Minor Healing, CLW/CSW potions) before relying on spells.
- Amnesia: `50 + chance/2` — at least 50%. Healing cures amnesia; nothing else castable does.
- **Armour weight does not change failure** in this version; it only reduces mana (section 6).
- **Gloves have no spellcasting penalty in this version.** There is no glove/encumbrance code in
  `src/` at all (the 3.x/4.1 rule that gloves without Free Action/DEX hurt arcane casters is gone).
  Wear the best gloves you find.
- Worked example: Mage, INT 18/50, L20, *Frost Bolt* (L13, base 40): 40 − 21 − 10 = 9% (floor 4).
  Same mage at L13: 40 − 0 − 10 = 30%. Same mage casting *Mana Bolt* (L25, base 60) at L25 with
  18/100: 60 − 0 − 21 = 39%.
- Necromancer, Nether Bolt at L10 with INT 18/20: 22 − 27 − 7 = −12 → floor 5% in the dark. On a lit
  square the +25 is added *before* the floor: −12 + 25 = 13%. For a spell nearer its level the
  penalty is the full +25 (e.g. Disenchant at L16, INT 18/50: 50 − 10 = 40% dark, 50% (capped) lit).
  Darkness matters enormously for necromancers.

Device skill is unrelated to spell failure (see [Chapter 7](07-consumables-and-devices.md) and
[Chapter 1](01-core-mechanics.md)).

---

## 6. Mana: maximum, encumbrance, regeneration

### 6.1 Maximum mana

FACT (`src/player-calcs.c:calc_mana()`):

```
levels = L - first_spell_level + 1
msp    = 1 + adj_mag_mana[stat] * levels / 100        (0 if levels <= 0)
msp   -= max(0, (armour_weight - class_weight_limit) / 10)    (weights in 1/10 lb)
```

Armour weight counts every equipped item except weapon, bow, rings, amulet and light. Recalculated
continuously, so a stat gain is retroactive to level 1 (as the official manual says).

`adj_mag_mana` (extra SP per level ×100):

| Stat | 8 | 10 | 12 | 14 | 16 | 17 | 18/00 | 18/20 | 18/50 | 18/70 | 18/80 | 18/90 | 18/100 | 18/120 | 18/150 | 18/170 | 18/180+ |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SP/level ×100 | 50 | 70 | 90 | 110 | 130 | 140 | 150 | 170 | 200 | 250 | 300 | 350 | 400 | 500 | 650 | 750 | 800 |

Example max SP (first spell level 1):

| L | stat 16 | 18/50 | 18/100 | 18/150 | 18/200 |
|---|---|---|---|---|---|
| 10 | 14 | 21 | 41 | 66 | 81 |
| 20 | 27 | 41 | 81 | 131 | 161 |
| 30 | 40 | 61 | 121 | 196 | 241 |
| 40 | 53 | 81 | 161 | 261 | 321 |
| 50 | 66 | 101 | 201 | 326 | 401 |

Rogue (first level 5) and Ranger (3) use `L−4` and `L−2` levels. Mana grows much faster with the
stat above 18/70 than below it; for a pure caster every +10 in the casting stat past 18/70 is worth
more mana than several character levels.

**Encumbrance**: 1 SP lost per full pound over the limit (Mage/Necro 30 lb, Druid/Priest/Rogue
35 lb, Paladin/Ranger 40 lb, Blackguard 60 lb). Reference weights (`lib/gamedata/object.txt`): Robe 2,
Soft Leather 8, Hard Leather 12, Leather Scale 14, Metal Scale 25, Chain 22, Mithril Chain 15 lb; caps
2–6 lb; boots 1.5–6; gloves 0.5–4; shields: Wicker 3, Small Metal 6, Leather 9, Large Metal 12 lb;
dragon armour 10–30 lb. A Mage in Soft Leather + leather boots + leather cap + cloak + leather gloves
+ wicker shield carries ~17 lb — no penalty. ADVICE: a few SP lost for much better AC is often fine
at high mana; at low level (10–20 SP total) stay under the limit.

### 6.2 Regeneration

FACT (`src/player-util.c:player_regen_mana()`, called from `process_world()` every 10 game turns,
i.e. once per normal-speed player turn regardless of your speed):

```
percent = 197                     (PY_REGEN_NORMAL, units of 1/65536)
if REGEN flag:      percent *= 2
if resting (>=5 turns, or rest '&' / '*' / HP-or-SP modes immediately): percent *= 2
if IMPAIR_MANA:     percent /= 2
gain per tick = msp * percent / 65536  + 524/65536
```

- About **0.30% of max SP per tick** walking, 0.60% resting, 1.2% resting with *Regeneration*. From
  empty: ~330 ticks walking, ~170 resting, ~85 resting with Regen.
- Speed does not speed up regeneration: a hasted character gets fewer SP per action. Resting while
  hasted wastes haste duration.
- Mana also comes from: Potions of Restore Mana; Mage *Tap Magical Energy*; Necromancer *Tap Unlife*
  and *Power Sacrifice*; Blackguard combat ([Chapter 7](07-consumables-and-devices.md) for items).

### 6.3 Blackguard mana (`COMBAT_REGEN`)

FACT (`src/player-util.c`, `src/player-attack.c`, `src/player-spell.c`):

- Taking damage gives SP: losing X% of max HP gives X% of max(SP,10) (not from poison, starvation,
  "fatal wound").
- Each melee attack command gives 5% of max(SP,10).
- SP *decay* over time at half the normal regeneration rate; the decay is converted to HP at double
  the efficiency of casting. Regen/resting bonuses to the decay apply only when HP ≤ 50%.
- Every spell cast converts its cost to healing: spending X% of max SP restores X/2% of *missing* HP
  (capped at 25% of missing HP per cast).
- So a Blackguard starts fights with little mana and builds it by hitting and being hit. Plan
  spells as finishers and mid-fight buffs, not openers.

---

## 7. How spell effects work

### 7.1 Geometry and targeting

FACT (`src/effect-handler-attack.c`, `src/project.c`):

| Effect | Shape | Stops at first monster? | Hurts floor items? | Notes |
|---|---|---|---|---|
| `BOLT` | single target | yes (even with a target set: it hits whatever is first in the path) | no | |
| `BEAM` | line to range | no, hits all in line | no | |
| `BOLT_OR_BEAM` | bolt, beam with chance `beam% + other` | as above | no | Mage only; beam% = L |
| `LINE` | beam that also affects grids | no | no | Spear of Light lights; Stone to Mud digs |
| `BALL` | explodes at target | **targeted:** flies over monsters to the target grid; **direction:** stops at first monster | yes | damage at distance d from centre = `(dam+d)/(d+1)`: full, 1/2, 1/3... |
| `SPHERE` | ball centred on you | — | yes | does not hit you; source "diameter" sets fall-off (Volcanic Eruption full damage to radius 4) |
| `ARC` | cone | no | yes | full damage out to `4×60/degrees` grids (Acid Spray 60°: 3 grids; River of Lightning 20°: 11 grids), then falls off |
| `SHORT_BEAM` | short cone/beam of fixed length | no | yes | full damage over its whole length |
| `STRIKE` | ball at the target, no path ("from above") | — | yes | target must be in LOS and passable or it lands on **you** (centre on you, monsters around you hit) |
| `SWARM` | N balls at one target point | each stops at first monster | yes | Meteor Swarm |
| `PROJECT_LOS(_AWARE)` | every monster in line of sight | — | no | damage rolled **once** and applied to all |
| `TOUCH` | radius around you | — | yes | |
| `SPOT` | radius around you **including you** | — | yes | Zone of Unmagic hits the caster |

ADVICE: use bolts and beams near loot (they never destroy floor objects); use a target (not a
direction) for balls so they reach the group behind a blocker.

### 7.2 Dice, `$` expressions and `m_bonus`

FACT: `dice:` strings may contain `$X` placeholders resolved by `expr:X:BASE:ops` (bases:
`PLAYER_LEVEL`, `PLAYER_HP` = current HP, `MONSTER_PERCENT_HP_GONE`, ...; `src/effects.c`). `m$M` sets
the *m_bonus* field, which different effects use differently: for `HEAL_HP` it is the percentage of
*missing* HP healed (with the base+dice as minimum); for `TELEPORT` it makes the distance a
percentage of the distance to the level edge; for `SWARM` it is the number of projectiles. Spell
damage gets no device "boost" (`spell_cast()` passes boost 0).

### 7.3 Monster saving throws against status spells

FACT (`src/list-mon-timed.h`, `src/mon-timed.c`):

| Status | Saving throw? | Blocked by flag | Stacking | Max | Notes |
|---|---|---|---|---|---|
| Sleep | yes | `NO_SLEEP` | no | 10000 | |
| Fear | yes | `NO_FEAR` | adds | 10000 | decays by 1d(level/10+1) per monster turn, so deep monsters recover fast |
| Confusion | **no** | `NO_CONF` | max | 50 | |
| Slow | **no** | `NO_SLOW` | adds | 50 | −2 speed per 10 turns of timer (max −10) |
| Hold | **no** | `NO_HOLD` | max | 50 | broken by any damage from you |
| Stun | **no** | `NO_STUN` | max | 50 | 1 in 10 turns lost, −25% to-hit/damage |
| Disenchant (spell disruption) | no | `IM_DISEN` | max | 50 | only monsters with non-innate spells |

Saving throw (sleep, fear): resist chance = `min(90, monster_level + max(0, 25 − timer/2))`;
uniques roll twice. Huge timers (Mass Sleep 10L+500) reduce this to just the monster's level.
Teleport-away (`AWAY_*`, Teleport Other's `AWAY_ALL`) has **no** resistance beyond the type check; teleport
works on uniques. Polymorph (Chaos) never affects uniques.

Consequence (ADVICE): *Slow Monster*, *Confuse Monster* and *Trance* are far more reliable than their
reputation suggests: only 7 monsters in the game have `NO_SLOW` (see the table in
[19.2](#192-monster-flag-prevalence-by-depth)), and there is no level-based save.

---

## 8. Books

FACT (`book:` and `book-properties:cost:commonness:min to max` in `lib/gamedata/class.txt`;
`src/init.c` gives dungeon books `IGNORE` for acid/elec/fire/cold and the `GOOD` kind flag; `src/store.c`:
`always:<book tval>` stocks every **town** book of that realm):

| Book | Realm | Type | Base cost | Commonness | Depth | Classes (spells) |
|---|---|---|---|---|---|---|
| [First Spells] | arcane | town | 25 | 40 | 1–100 | Mage (7), Rogue (6) |
| [Attacks and Knowledge] | arcane | town | 400 | 40 | 10–100 | Mage (6) |
| [Magical Defences] | arcane | town | 1600 | 20 | 30–100 | Mage (6) |
| [Arcane Control] | arcane | dungeon | 30000 | 15 | 40–100 | Mage (6), Rogue (4) |
| [Wizard's Tome of Power] | arcane | dungeon | 50000 | 10 | 60–100 | Mage (5) |
| [Novice's Handbook] | divine | town | 25 | 40 | 1–100 | Priest (6), Paladin (6) |
| [Cleansing Power] | divine | town | 100 | 40 | 5–100 | Priest (6) |
| [Healing and Sanctuary] | divine | dungeon | 10000 | 20 | 30–100 | Priest (6), Paladin (5) |
| [Battle Blessings] | divine | dungeon | 30000 | 15 | 40–100 | Priest (5), Paladin (5) |
| [Wrath of the Valar] | divine | dungeon | 50000 | 10 | 60–100 | Priest (5) |
| [Lesser Charms] | nature | town | 25 | 40 | 1–100 | Druid (6), Ranger (6) |
| [Gifts of Nature] | nature | town | 400 | 40 | 10–100 | Druid (6) |
| [Creature Dominion] | nature | dungeon | 10000 | 20 | 30–100 | Druid (5) |
| [Nature Craft] | nature | dungeon | 30000 | 15 | 40–100 | Druid (5), Ranger (5) |
| [Wild Forces] | nature | dungeon | 50000 | 10 | 60–100 | Druid (5) |
| [Into the Shadows] | shadow | town | 25 | 40 | 1–100 | Necromancer (5), Blackguard (6) |
| [Dark Rituals] | shadow | town | 400 | 40 | 10–100 | Necromancer (5) |
| [Fear and Torment] | shadow | town | 1600 | 20 | 30–100 | Necromancer (5), Blackguard (5) |
| [Deadly Powers] | shadow | dungeon | 30000 | 15 | 40–100 | Necromancer (6), Blackguard (4) |
| [Corruption of Spirit] | shadow | dungeon | 50000 | 10 | 60–100 | Necromancer (5) |

Where to get them (FACT, `lib/gamedata/store.txt`):

- The **Bookseller (store 4)** always stocks every town book of all four realms and buys all books.
  Prices shown in the store include the owner's markup on the base cost (see
  [Chapter 11](11-town-and-shopping.md)).
- The **Magic shop (6)** buys magic books (does not normally stock them). The **Black market** can
  sell anything, including dungeon books, at a large markup.
- **Dungeon books** are only found in the dungeon (or the black market, or a store you sold one to).
  They are `GOOD` objects, so they can come from "good" drops, *Acquirement* and vaults, and they
  cannot burn or dissolve. Town books **can** be destroyed by fire (and the like) in your pack —
  carry a spare of your key town book once you can afford it ([Chapter 5](05-elements-and-status.md)).
- Starting kit: every caster starts with its first town book (Mage/Rogue [First Spells], Druid/Ranger
  [Lesser Charms], Priest/Paladin [Novice's Handbook], Necromancer/Blackguard [Into the Shadows]).

Which book gives what (ADVICE summary; details in each class section):

| Class | Must-buy town books, in order | Dungeon books that change the game |
|---|---|---|
| Mage | First Spells (start) → Attacks and Knowledge (400: Recharging, Identify Rune, Frost Bolt) → Magical Defences (1600: **Teleport Self L7**, Teleport Other, Resistance) | Arcane Control (Mana Bolt, Teleport Level, Detection, Dimension Door), Wizard's Tome (Banishment, Mass Banishment, Mana Storm) |
| Druid | Lesser Charms (start) → Gifts of Nature (400: Stone to Mud, Sense Surroundings, Lightning Strike, Earth Rising) | Creature Dominion (Trance, Mass Sleep, shapes), Nature Craft (**Haste Self**, Herbal Curing), Wild Forces (Meteor Swarm, Rift, Ice Storm) |
| Priest | Novice's Handbook (start) → Cleansing Power (100: **Orb of Draining**, PfE, Remove Curse) | Healing and Sanctuary (**Portal**, **WoR**, **Healing**, Clairvoyance), Battle Blessings, Wrath of the Valar (Banish Evil, Holy Word) |
| Necromancer | Into the Shadows (start) → Dark Rituals (400: Crush, Shadow Shift, Disenchant) → Fear and Torment (1600: Vampire Strike, Dispel Life, Dark Spear) | Deadly Powers (Annihilate, Banish Spirits, Unleash Chaos, Storm of Darkness), Corruption of Spirit (Vampire Form, Curse) |
| Paladin | Novice's Handbook (start) | Healing and Sanctuary (PfE, WoR, Healing), Battle Blessings (Smite Evil, Single Combat) |
| Rogue | First Spells (start) | Arcane Control (**Teleport Self L17**, Teleport Other, Teleport Level) |
| Ranger | Lesser Charms (start) | Nature Craft (**Haste Self L25**, Decoy, Brand Ammunition) |
| Blackguard | Into the Shadows (start) → Fear and Torment (1600: Maim Foe, Howl, Taunting, Venom, Werewolf) | Deadly Powers (Bloodlust, Unholy Reprieve, Forceful Blow, Quake) |

Important implication: **no hybrid class and no Priest gets a teleport/escape spell from a town
book**. Priest *Portal* is in a dungeon book (DL30+), Rogue *Teleport Self* in another (DL40+).
Rogues, Priests, Paladins, Rangers, Druids and Blackguards rely on scrolls for escapes early.

---
## 9. Mage

INT, arcane, `ZERO_FAIL | BEAM | CHOOSE_SPELLS`, first spell L1, armour limit 30 lb, 30 spells in 5
books. The most complete toolbox: unresistable bolts, the best detection, teleport-other, and the
only identify/recharge/banishment caster. Weak HP. Official manual: [birth](https://angband.readthedocs.io/en/latest/birth.html#mage).

### 9.1 Mage spells, resolved

Beam chance for `BOLT_OR_BEAM` spells: L% (Magic Missile: L−10%).

**[First Spells]** (town, 25 gold, starting book)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Magic Missile | 1 | 1 | 22 | 4 | Bolt/beam of `MISSILE` (nothing resists), `((L−1)/5+3)d4`: 3d4 (7.5) L1–5, 4d4 L6–10, 6d4 (15) L16–20, 8d4 (20) L26–30, 10d4 (25) L36–40, 12d4 (30) L46–50. |
| Light Room | 1 | 2 | 26 | 4 | Lights the room you are in plus radius 2; 1 point of `LIGHT_WEAK` (hurts only `HURT_LIGHT` monsters) in radius 2. |
| Find Traps, Doors & Stairs | 1 | 1 | 20 | 2 | Detects traps, doors, stairs within ±22 rows, ±40 columns. |
| Phase Door | 2 | 2 | 22 | 4 | Teleport ~10 squares (the game randomises every teleport distance by up to ±25%, here 9–11). |
| Electric Arc | 2 | 2 | 34 | 4 | Short beam of `ELEC`, length L grids (max range 20), `((L−1)/5+3)d6` (same dice count as Magic Missile, d6): 3d6 at L2–5, 6d6 at L16–20. Hits every monster along it; destroys elec-vulnerable floor items. |
| Detect Monsters | 3 | 2 | 24 | 4 | Shows non-invisible monsters in ±22/±40 for one turn. |
| Fire Ball | 6 | 5 | 33 | 5 | Radius-2 `FIRE` ball, 2L at the centre (12 at L6, 20 at L10, 40 at L20, 100 at L50), half at radius 1, a third at radius 2. |

**[Attacks and Knowledge]** (town, 400 gold, DL10+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Recharging | 5 | 5 | 28 | 6 | Recharge strength `L/8+6` (6 at L5, 8 at L20, 12 at L50; the Scroll of Recharging is 6). Backfire (destroys one item of the stack) with chance `1/(strength + (100−item level)/10 − 2×charges_per_item)`; on success adds `2 + 1d(strength/(10−ease)+1)` charges. |
| Identify Rune | 8 | 7 | 25 | 8 | Reveals one unknown rune on a chosen item. |
| Treasure Detection | 10 | 3 | 60 | 5 | Detects mineral veins with treasure; *senses* gold and objects (shows that something is there, not what) in ±22/±40. |
| Frost Bolt | 13 | 5 | 40 | 6 | Bolt/beam of `COLD`, `((L−5)/3+6)d8`: 8d8 (36) at L13, 11d8 (49.5) at L20, 14d8 (63) at L30, 17d8 (76.5) at L40, 21d8 (94.5) at L50. |
| Reveal Monsters | 15 | 6 | 40 | 8 | Shows all monsters, including invisible, in ±22/±40 for one turn. |
| Acid Spray | 20 | 5 | 30 | 12 | 60° cone of `ACID`, range 10, `(L/2)d8`: 10d8 (45) at L20, 15d8 (67.5) at L30, 25d8 (112.5) at L50; full damage out to 3 grids, then 4/(d+1). |

**[Magical Defences]** (town, 1600 gold, DL30+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Disable Traps, Destroy Doors | 5 | 5 | 30 | 6 | Destroys doors and disables traps in radius 1 around you. |
| Teleport Self | 7 | 6 | 35 | 5 | Teleport a distance of `9L/5`% of the distance to the farthest level edge (±25%): 18% at L10, 36% at L20, 54% at L30, 90% at L50. The Scroll of Teleportation is 60% — **the spell only matches the scroll from L34.** |
| Teleport Other | 15 | 10 | 30 | 12 | Bolt that teleports the first monster hit 3L squares away. No saving throw; works on uniques (not in Single Combat arenas). |
| Resistance | 20 | 20 | 65 | 20 | Temporary resist acid, lightning, cold and fire for 20+1d20 turns (one shared roll). |
| Tap Magical Energy | 22 | 2 | 30 | 12 | Drains *all* charges of a wand/staff stack: gain `(5 + item level) × charges / 4` SP (needs at least 6 SP worth), and you are stunned 1–2 turns. |
| Mana Channel | 25 | 10 | 50 | 20 | `FASTCAST` for 5+1d5 turns: each cast costs 75 energy (33% more casts per game time). Cancelled by stun. |

**[Arcane Control]** (dungeon, DL40+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Door Creation | 13 | 9 | 40 | 12 | Closed (unlocked) door on every empty floor grid adjacent to you. |
| Mana Bolt | 25 | 8 | 60 | 30 | Bolt/beam of `MISSILE` (unresistable), `(L−10)d8`: 15d8 (67.5) at L25, 20d8 (90) at L30, 30d8 (135) at L40, 40d8 (180) at L50. |
| Teleport Level | 28 | 17 | 65 | 20 | One level up or down at random (may do nothing with forced descent). |
| Detection | 30 | 10 | 70 | 30 | Traps, doors, stairs, treasure veins, senses gold and objects, all monsters including invisible, ±22/±40. |
| Dimension Door | 35 | 30 | 80 | 40 | Teleport to (near) a grid you target; a vault target is randomised by up to 10. |
| Thrust Away | 40 | 12 | 90 | 40 | Short beam of `FORCE`, length 1+L/10 (5 at L40), `Ld8` (180 at L40, 225 at L50); 1/3 chance to stun 5–14 turns; knocks monsters back `3 + damage/20` squares. Only force breathers resist. |

**[Wizard's Tome of Power]** (dungeon, DL60+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Shock Wave | 20 | 5 | 40 | 16 | Radius-2 `SOUND` ball, 2L; 1/3 chance to stun each monster (unless `NO_STUN`). |
| Explosion | 30 | 10 | 50 | 20 | Radius-2 `SHARD` ball 2L, then radius-2 `FORCE` ball L/5 (knockback, stun chance). |
| Banishment | 35 | 45 | 95 | 25 | Removes every non-unique monster of a chosen symbol from the level; you take 1d4 per monster removed. |
| Mass Banishment | 40 | 75 | 90 | 100 | Removes every non-unique monster within 20 squares; 1d3 damage per monster removed. |
| Mana Storm | 45 | 16 | 85 | 200 | Radius-3 `MANA` ball (unresistable), 2L+300 (390 at L45, 400 at L50). |

### 9.2 Mage spell-by-spell advice

- **Magic Missile** — FACT: cheapest damage per SP in the game for single targets until about L20,
  and still ~25–30 damage per SP late; never resisted; beams at L−10%. ADVICE: learn first; the
  default attack against anything weak, anything with unknown resists, and to finish wounded
  monsters. Bolts stop at the first monster: line up.
- **Light Room** — ADVICE: light dark rooms before entering to see what is inside. It does not wake
  monsters by itself, but a lit room is lit for everyone: what you can see can also target you. The
  1-point damage is irrelevant.
- **Find Traps, Doors & Stairs** — ADVICE: cast on arrival on every level and whenever you leave the
  detected area. Essential for stair-scumming and diving ([Chapter 10](10-exploration.md)).
- **Phase Door** — ADVICE: the bread-and-butter escape and positioning tool (break line of sight,
  get out of melee to cast). Keep scrolls anyway: the spell can fail (22% base) and needs you not
  blind/confused.
- **Electric Arc** — ADVICE: for corridors full of weak monsters (e.g. a line of jackals) it beats
  Magic Missile per cast; per SP it's worse (2 SP). Destroys floor items (wands/rings) — don't use it
  over loot.
- **Detect Monsters** — ADVICE: cast before opening doors, on arrival, before resting. Invisible
  monsters need Reveal Monsters (L15).
- **Fire Ball** — ADVICE: worth it against groups of 3+ or `HURT_FIRE` monsters. Against a single
  monster at L6–20 Magic Missile does more per SP. Target the group (not a direction) so the ball
  passes over the front monster.
- **Recharging** — ADVICE: recharge only *empty* wands (charges count against you, `−2×charges`),
  low-level wands (Magic Missile L3, Stinking Cloud) are safe-ish; deep wands (Teleport Other, Drain
  Life) have a real backfire chance — never recharge your only one. A mage with Recharging and
  a Wand of Magic Missile/Stinking Cloud has a second "mana pool".
- **Identify Rune** — ADVICE: big quality-of-life spell; identify runes on wearables you are about to
  rely on (especially to learn curses before wielding, see [Chapter 6](06-equipment.md)).
- **Treasure Detection** — TRAP: 60% base fail and it only *senses* objects. Useful only for gold-diving;
  Detection (L30) supersedes it. Learn for the XP when study slots are plentiful.
- **Frost Bolt** — ADVICE: the main single-target spell from L13 to the mid-20s (~50 damage for 5 SP at
  L20). Swap to Magic Missile against `IM_COLD` monsters (cold immunity: 16% at DL11–20, 34% at
  DL31–40). Doubles against `HURT_COLD`.
- **Reveal Monsters** — ADVICE: from L15 this is your monster detection; 6 SP is cheap insurance
  against ghosts and invisible casters.
- **Acid Spray** — ADVICE: the best crowd spell a mage has until Mana Storm: a 60° cone with full
  damage to 3 grids, 45 damage at L20 for 5 SP. Use it on packs in rooms and against breeders. Acid
  immunity is rare at DL21–40 (5–15%). Destroys acid-vulnerable floor items.
- **Disable Traps, Destroy Doors** — ADVICE: situational; useful when a trap blocks a corridor, or to
  remove doors so you can't be ambushed from them. Mostly XP fodder.
- **Teleport Self** — FACT: short at low level (see table). ADVICE: learn it as soon as you can buy
  Magical Defences (1600 gold) — it's still a free escape that does not use inventory — but until
  ~L34 keep Scrolls of Teleportation as the "get away for real" option.
- **Teleport Other** — ADVICE: the mage's best defensive spell. Removes any single monster, including
  uniques, with no save. Bolt: it hits the *first* monster in the path — clear line needed. The main
  answer to dangerous melee uniques, invisible-but-detected ghosts, and breathers in corridors.
- **Resistance** — ADVICE: cast before fighting dragons, hounds or casters of the four base elements
  when you lack permanent resists; temporary resistance stacks with item resistance (double resist
  cuts damage to 1/9). 20 SP and 65% base fail make it a pre-buff, not an emergency spell.
- **Tap Magical Energy** — TRAP in most situations: it destroys all charges and stuns you (1–2
  turns). Use only on junk wands/staves when you need mana *now* in a safe place (deeper items with many
  charges give the most: SP = (5 + item level) × charges / 4).
- **Mana Channel** — ADVICE: before a long boss fight; 10 SP buys 5–10 turns of 33% faster casting.
  Stun cancels it: cure stun first.
- **Door Creation** — ADVICE: niche tactical spell: box yourself in to rest, or put a door between you
  and a non-door-opening breather pack. Many monsters open or bash doors.
- **Mana Bolt** — ADVICE: the unique-killer from L25: unresistable, 90 damage at L30 for 8 SP. Starts at
  60% base fail (capped to 50%) — it becomes reliable around L30–35 with a good INT.
- **Teleport Level** — ADVICE: the strongest escape that also leaves the level's threats behind.
  17 SP, 65% base: practise it until reliable before depending on it. Takes effect immediately (unlike
  Deep Descent / Word of Recall scrolls).
- **Detection** — ADVICE: one cast does everything; becomes the standard arrival spell once its fail
  drops (70% base).
- **Dimension Door** — ADVICE: precise escape/positioning: jump across chasms of monsters to the stairs,
  or behind a wall. 30 SP, 80% base.
- **Thrust Away** — ADVICE: a strong short-range spell late (180+ force damage, stun, knockback); its
  90% base fail keeps it at 50% for a while. Knockback also breaks melee contact. Short range (5–6).
- **Shock Wave / Explosion** — ADVICE: arrive only when the Wizard's Tome is found (DL60+). By then
  Acid Spray and Mana Bolt are better per SP for most targets; use Shock Wave against
  acid-immune groups (stun is a nice bonus), Explosion is rarely worth 10 SP.
- **Banishment** — ADVICE: removes a whole symbol (e.g. all `Z` hounds, all `d`/`D`, all `U` that are not
  unique) at the cost of 45 SP and 1d4 HP per removed monster. Superb against hound packs, nests and
  summoners' summons; check HP first: banishing a 40-monster nest costs up to 160 HP.
- **Mass Banishment** — ADVICE: the panic button: clears everything non-unique within 20 squares. 75 SP,
  1d3 HP per monster. Leaves uniques — great for isolating a unique from its escort.
- **Mana Storm** — ADVICE: endgame damage: 400 unresistable damage in radius 3 for 16 SP. The best
  spell for Morgoth/Sauron fights together with Mana Bolt.

### 9.3 Mage priorities

**Workhorse attacks by level**

| Level | Single target | Groups |
|---|---|---|
| 1–5 | Magic Missile | Electric Arc (lines) |
| 6–12 | Magic Missile | Fire Ball (3+ targets) |
| 13–19 | Frost Bolt (Magic Missile vs cold-immune) | Fire Ball |
| 20–24 | Frost Bolt | Acid Spray |
| 25–44 | Mana Bolt (tough/unique), Frost Bolt (routine), Magic Missile (trivial) | Acid Spray |
| 45–50 | Mana Bolt, Mana Storm | Mana Storm, Acid Spray |

**Utility**: Find Traps/Doors/Stairs (L1) → Detect Monsters (L3) → Identify Rune (L8) → Recharging (L5)
→ Reveal Monsters (L15) → Detection (L30). Light Room (L1) for lighting.

**Emergency**: Phase Door (L2), Teleport Self (L7; scroll better before L34), Teleport Other (L15),
Teleport Level (L28), Dimension Door (L35), Mass Banishment (L40), Banishment (L35).

**Traps / low value**: Treasure Detection, Tap Magical Energy (except as deliberate mana top-up),
Door Creation, Explosion; Thrust Away until its fail drops.

Note what the Mage does **not** have: no healing, no cure, no haste, no Word of Recall, no Deep
Descent, no remove curse. Buy potions/scrolls/rods for these ([Chapter 7](07-consumables-and-devices.md)).

---

## 10. Druid

WIS, nature, `ZERO_FAIL | CHOOSE_SPELLS | CHARM`, first spell L1, armour limit 35 lb, 27 spells.
A caster of balls, cones and line-of-sight storms with shapechanges and crowd control. Only two town
books — the middle game depends on finding dungeon books. Official manual:
[birth](https://angband.readthedocs.io/en/latest/birth.html#druid).

### 10.1 Druid spells, resolved

**[Lesser Charms]** (town, 25 gold, starting book)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Detect Life | 1 | 1 | 23 | 2 | Shows all *living* monsters (not undead/nonliving), even invisible ones, ±22/±40, one turn. |
| Fox Form | 1 | 2 | 20 | 10 | Shapechange to fox ([section 17](#17-shapechanges)): +1 blow, moves cost half energy, stealth +5, STR −3, to-hit/dam −3, AC +3, Feather Fall, Free Action. |
| Remove Hunger | 2 | 2 | 25 | 2 | Sets food to at least 50% ("Fed", not full). |
| Stinking Cloud | 3 | 2 | 27 | 4 | Radius-2 `POIS` ball, `L/2+10` (11 at L3, 15 at L10, 20 at L20, 35 at L50). |
| Confuse Monster | 5 | 3 | 30 | 5 | Bolt: confusion for `5+1d(L−5)` turns (5 at L5, 5–20 at L20); ×1.5 vs animals; only `NO_CONF` resists. |
| Slow Monster | 6 | 4 | 30 | 5 | Bolt: slow for `5+1d(L−5)` turns, stacking; −2 speed per 10 turns of timer; ×1.5 vs animals; only `NO_SLOW` (7 monsters) resists. |

**[Gifts of Nature]** (town, 400 gold, DL10+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Cure Poison | 4 | 4 | 30 | 4 | Cures poison. |
| Resist Poison | 7 | 6 | 32 | 5 | Temporary resist poison 20+1d20 turns. |
| Turn Stone to Mud | 8 | 3 | 25 | 5 | Beam that dissolves the first wall section (not permanent walls); 20+1d30 to `HURT_ROCK` monsters in the beam. |
| Sense Surroundings | 9 | 4 | 35 | 6 | Maps the area ±22 rows × ±44 columns; from L30 ±44 × ±88. |
| Lightning Strike | 12 | 6 | 35 | 8 | `ELEC` strike `(L/4)d4` on the target grid, then a radius-3 `SOUND` blast `L+5` centred there (falls off with distance; stuns 1/3). Needs a visible passable target in LOS, else it centres on you. |
| Earth Rising | 14 | 5 | 40 | 12 | Short beam of `SHARD`, length `4+L/5`, `(L/3+2)d6` (6d6=21 at L14, 8d6=28 at L20, 12d6=42 at L30, 18d6=63 at L50). |

**[Creature Dominion]** (dungeon, DL30+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Trance | 20 | 10 | 45 | 15 | Holds every adjacent monster for `3+1d(L/7)` turns (×1.5 animals). Only `NO_HOLD` resists; **any damage from you breaks hold**. |
| Mass Sleep | 25 | 15 | 50 | 15 | Sleep (power 10L+500) on every monster in LOS. Monsters without `NO_SLEEP` resist with probability = their level % (uniques roll twice). |
| Become Pukel-man | 30 | 20 | 75 | 50 | Shapechange: speed −5, +4 STR/CON, damage reduction 10, **poison immunity** (`RES_POIS[3]`), rShards, protection from stun, regen, hold life, +20 save, +20 AC; cures poison and stun on change. |
| Eagle's Flight | 35 | 20 | 80 | 80 | Shapechange: moves cost 1/4 energy, see invisible, protection from blind/conf/fear, Free Action, trap immunity. |
| Bear Form | 40 | 20 | 85 | 100 | Shapechange: +15 to-hit, +15 to-dam, +5 AC, +1 blow, STR+3 CON+2 INT−2, protection from fear, melee skill +10, stealth −3. |

**[Nature Craft]** (dungeon, DL40+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Tremor | 20 | 20 | 60 | 16 | Earthquake radius 4 at a targeted grid (no effect in town; can hurt you if you are inside it). |
| Haste Self | 25 | 12 | 65 | 15 | +10 speed for `L+1d20` turns; recasting while hasted adds only 5 turns. |
| Revitalize | 35 | 70 | 90 | 90 | Restores all five stats and experience. |
| Rapid Regeneration | 37 | 20 | 90 | 100 | `HEAL` timed effect 5+1d3 turns: +30 HP every game tick, burns 8× food per tick, ends when hungry. |
| Herbal Curing | 40 | 20 | 90 | 100 | Cures cuts, poison, stun, Black Breath; feeds to 80%. |

**[Wild Forces]** (dungeon, DL60+)

| Spell | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Meteor Swarm | 30 | 14 | 85 | 20 | `L/20+2` meteors (3 below L40, 4 from L40), each a radius-1 `METEOR` ball (unresistable) of `L/2+30` (45 at L30, 50 at L40, 55 at L50), all aimed at the same point; each stops at the first monster in its path. Up to 135/200/220 total on one target. |
| Rift | 35 | 20 | 60 | 25 | Beam of `GRAVITY`, `40+Ld7` (180 at L35, 240 at L50); each hit monster may be teleported 10 squares (if 1d127 > its level). Only gravity breathers resist. |
| Ice Storm | 37 | 25 | 75 | 24 | `ICE` on every monster in LOS, `3d(3L)` rolled once (avg ~168 at L37, 226 at L50); cold immunity divides by 9, `HURT_COLD` doubles; 1/3 stun. |
| Volcanic Eruption | 40 | 30 | 75 | 50 | `FIRE` sphere radius 5 around you, `3L/2 + 1d(3L)` (avg 120 at L40, 150 at L50), full strength to radius 4; then an earthquake radius 5 centred on you. |
| River of Lightning | 43 | 35 | 75 | 100 | 20° cone of `PLASMA`, range 20, `(L+10)d8` (238 at L43, 270 at L50), full damage to 11 grids. Only `IM_PLASMA` (8 monsters) resists. |

### 10.2 Druid spell-by-spell advice

- **Detect Life** — ADVICE: cheap; misses undead and golems/elementals/vortices, which is exactly what
  you must not forget. Combine with Sense Surroundings for exploring.
- **Fox Form** — ADVICE: an L1 movement tool: two moves per turn means you outrun almost everything
  early. Use it to cross explored levels, flee, and scout. You cannot read or quaff while a fox (must
  revert, which is free, then act next turn) — so transform only when the danger is "out-runnable".
  The 10 XP first cast is a free level 2.
- **Remove Hunger** — ADVICE: never buy food again.
- **Stinking Cloud** — ADVICE: workhorse from L3 to the mid-20s; radius 2 means it is always an area
  spell. Poison immunity is common (29% at DL1–20, ~40% at DL21–40: jellies, molds, undead, golems,
  many demons/dragons) — check monster recall and switch to Lightning Strike / melee.
- **Confuse Monster / Slow Monster** — ADVICE: underrated because there is **no saving throw** (only
  the flag). Slow stacks: two casts of Slow at L20 average −4 to −6 speed. Excellent against single
  dangerous melee monsters and early uniques without `NO_CONF` (Bullroarer, Wormtongue, Grishnákh,
  Golfimbul, Shagrat, Gorbag, Bolg, Azog — but not Grip/Fang, Nár, Uglúk or Mîm, which have it). At
  depth, `NO_CONF` becomes near-universal (68% at DL31–40) but `NO_SLOW` stays near zero.
- **Cure Poison / Resist Poison** — ADVICE: Cure Poison saves CLW potions early; Resist Poison before
  fighting poison breathers and poison-biting swarms when you lack permanent rPois.
- **Turn Stone to Mud** — ADVICE: digging (shortcuts, escape tunnels, reaching vault items) and
  damaging `HURT_ROCK` monsters: clay/stone golems, umber hulks, xorns/xarens, earth spirits and
  elementals, stone trolls and **Bert, Bill and Tom** (DL30) — 20+1d30 per cast for 3 SP. Cheap, low fail.
- **Sense Surroundings** — ADVICE: core exploration spell (the druid's magic mapping). Doubles in area at
  L30.
- **Lightning Strike** — ADVICE: the mid-game single-target-plus-splash spell: ~38 at L20 for 6 SP. Two
  elements (elec + sound) means it is almost never fully resisted. Requires LOS to a passable target
  grid; if you aim at a direction or an unseen grid it explodes **on your position** (hitting
  monsters around you, not you — the projection lacks the player flag; unverified in play).
- **Earth Rising** — ADVICE: shard short beam, good in corridors (full damage over its length).
  Shards are resisted only by shard breathers. Comparable to Lightning Strike; cheaper (5 SP).
- **Trance** — ADVICE: an escape, not an attack: hold all adjacent monsters, then walk away or phase.
  Any damage you deal breaks it. Animals held longer.
- **Mass Sleep** — ADVICE: put a room to sleep and leave or pick targets. Most deep monsters have
  `NO_SLEEP` (67%+ beyond DL30), so it is mainly a DL20–40 tool.
- **Become Pukel-man** — ADVICE: a defensive melee form: DR 10, poison immunity, rShards, stun protection, regen,
  +20 AC — but **speed −5** is a large price. Useful for tanking a slow melee unique when you cannot
  escape. Cuts neither heal nor bleed while in this form (`ROCK` flag, see [17](#17-shapechanges)).
- **Eagle's Flight** — ADVICE: fastest overland travel in the game (4 moves per turn), plus
  blind/conf/fear protection and trap immunity. Use for crossing known levels to stairs and for
  escaping slow pursuers. No item use.
- **Bear Form** — ADVICE: turns a druid into a credible melee character (+15/+15, +1 blow). By L40,
  though, your spells usually out-damage your claws; use when out of mana or against magic-resistant
  monsters.
- **Tremor** — ADVICE: targeted earthquake: wall off a corridor, drop rubble between you and a pack, or
  destroy a summoner's surroundings. Can bury items. 60% base.
- **Haste Self** — ADVICE: the druid's most important utility spell (+10 speed, 26–45 turns at L25). Cast
  before every serious fight once its fail is acceptable; until then carry Potions of Speed.
- **Revitalize** — ADVICE: full restore after nexus/drain damage, 70 SP; cast in safety.
- **Rapid Regeneration** — ADVICE: 150–240 HP over a few game ticks at the cost of food; out-of-combat
  healing. 90% base fail (capped 50% until high level).
- **Herbal Curing** — ADVICE: the druid's answer to cuts/poison/stun/Black Breath; 90% base so long
  practice needed. Not healing.
- **Meteor Swarm** — ADVICE: the druid's single-target nuke from DL60 books: 200 unresistable damage on
  one target for 14 SP at L40 (four meteors). Each meteor stops at the first monster in its path; with
  monsters in front, they hit the front monster.
- **Rift** — ADVICE: big gravity beam (200+), resisted only by gravity hounds/dragons; may blink targets
  away (useful defensively). Lower fail (60) than the other Wild Forces spells.
- **Ice Storm** — ADVICE: line-of-sight room clearer: one roll applied to every visible monster. Bad vs
  cold-immune crowds (53%+ of DL41+ monsters are `IM_COLD`). Great vs mixed animal/orc/troll packs.
- **Volcanic Eruption** — ADVICE: point-blank fire sphere + earthquake around yourself. The earthquake
  can wall you in and destroy floor items; fire immunity is common deep. Situational.
- **River of Lightning** — ADVICE: the druid's best damage spell: 238–270 plasma in a long narrow cone,
  practically unresisted. Use on lines of monsters and big single targets.

### 10.3 Druid priorities

**Workhorse attacks by level**

| Level | Single target | Groups |
|---|---|---|
| 1–2 | melee / thrown, Fox Form to escape | — |
| 3–11 | Stinking Cloud | Stinking Cloud |
| 12–29 | Lightning Strike, Earth Rising (corridors); Stinking Cloud vs non-poison-immune | Stinking Cloud, Earth Rising |
| 30–36 (with Wild Forces) | Meteor Swarm, Rift | Lightning Strike, Rift (lines) |
| 37–42 | Meteor Swarm, Rift | Ice Storm |
| 43–50 | River of Lightning, Meteor Swarm | River of Lightning, Ice Storm |

Without dungeon books the druid stays on Lightning Strike/Earth Rising/Stinking Cloud. **Find
Creature Dominion (DL30+) and Nature Craft (DL40+) early** — their arrival is the druid's real power
curve.

**Utility**: Detect Life (L1), Remove Hunger (L2), Sense Surroundings (L9), Stone to Mud (L8), Cure
Poison (L4), Resist Poison (L7), Haste Self (L25), Revitalize (L35).

**Emergency**: Fox Form (L1), Trance (L20), Mass Sleep (L25), Eagle's Flight (L35), Rift (L35, teleport
side effect), Herbal Curing (L40). The druid has **no teleport spell, no phase door, no healing spell
(except Rapid Regeneration)** — carry Phase Door, Teleportation, CLW/CSW/CCW.

**Traps / low value**: Volcanic Eruption (self-centred quake), Become Pukel-man (speed −5),
Rapid Regeneration in combat, Tremor when you are inside the radius.

---
## 11. Priest

WIS, divine, `ZERO_FAIL | BLESS_WEAPON`, **random learning** (you choose the book, the god chooses the
prayer), first spell L1, armour limit 35 lb, 28 prayers. The best healer and anti-evil caster; no
attack spell until L7, no escape spell until a dungeon book. Official manual:
[birth](https://angband.readthedocs.io/en/latest/birth.html#priest).

### 11.1 Priest prayers, resolved

**[Novice's Handbook]** (town, 25 gold, starting book)

| Prayer | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Call Light | 1 | 1 | 10 | 2 | Lights the room you are in; radius-1 `LIGHT_WEAK` of `2d(L/2)` (0 at L1, 2d5 at L10, 2d10 at L20) to adjacent light-sensitive monsters. |
| Detect Evil | 1 | 1 | 10 | 4 | Shows all evil monsters (including invisible) in ±22/±40 for one turn. |
| Minor Healing | 1 | 2 | 15 | 4 | Heals `max(L+10, (L/4)% of missing HP)` (11 at L1, 20 at L10, 30 at L20; the percentage rarely matters), reduces cuts and stun by 20. |
| Bless | 1 | 2 | 20 | 4 | Blessed for `(L+10)+1d(L+10)` turns: +5 AC, +10 to-hit, slightly better device skill. |
| Sense Invisible | 3 | 4 | 25 | 4 | See invisible for 24+1d24 turns. |
| Heroism | 5 | 2 | 30 | 5 | Heals 10, cures fear; heroism (+12 to-hit, immune fear) for `(L−19)+1d(L−19)` turns — **nothing below L20** (2 turns at L20, ~16 at L30). |

**[Cleansing Power]** (town, 100 gold, DL5+)

| Prayer | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Orb of Draining | 7 | 7 | 40 | 4 | `HOLY_ORB` ball, radius 2 (3 from L30), `3L/2 + 3d6` (20.5 at L7, 40.5 at L20, 55.5 at L30, 85.5 at L50); **×2 against evil**, nothing resists it. |
| Spear of Light | 7 | 6 | 30 | 5 | Beam that lights every grid along it; `6d8` (27) to light-sensitive monsters only. |
| Dispel Undead | 10 | 14 | 55 | 6 | `1d(5L)` (rolled once) to every undead in view. |
| Dispel Evil | 12 | 20 | 70 | 10 | `1d(5L)` to every evil monster in view. |
| Protection from Evil | 14 | 8 | 42 | 5 | `3L+1d25` turns; each melee hit from an evil monster of level ≤ L is repelled with probability ≈ (49+L)% (69% at L20, 89% at L40). |
| Remove Curse | 16 | 6 | 38 | 8 | Attacks one curse with strength `L+1d(L)`; a curse with power ≥ strength is not removed and the item becomes fragile; a fragile item that fails again is destroyed 1 time in 4 (5d5 damage to you). Power ≥100 curses are permanent. |

**[Healing and Sanctuary]** (dungeon, DL30+)

| Prayer | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Portal | 10 | 4 | 30 | 8 | Teleport `L/2+30` squares (35 at L10, 45 at L30, 55 at L50; ±25%). |
| Remembrance | 20 | 30 | 90 | 50 | Restores lost experience. |
| Word of Recall | 25 | 30 | 75 | 10 | Recall activates after 15–34 turns; cast again to cancel. |
| Healing | 30 | 50 | 80 | 100 | Heals 2000 HP; cures cuts, poison, stun, amnesia. |
| Restoration | 35 | 70 | 90 | 130 | Restores all drained stats. |
| Clairvoyance | 37 | 50 | 80 | 150 | Lights and maps the entire level and *senses* every object. |

**[Battle Blessings]** (dungeon, DL40+) — note the book order is not level order.

| Prayer | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Enchant Weapon | 35 | 50 | 80 | 230 | 1d4 enchantment attempts on both to-hit and to-dam (normal enchant success table; artifacts resist half the time; +15 is the practical cap). |
| Enchant Armour | 37 | 60 | 85 | 250 | 2–4 enchantment attempts on armour AC. |
| Smite Evil | 25 | 20 | 70 | 40 | Melee gets "slay evil" (×2) for 20+1d20 turns (recast while active adds 5). |
| Glyph of Warding | 20 | 40 | 90 | 35 | Glyph under you: summons can't appear on it; a monster must break it to step on it or melee you while you stand on it (chance `(level−1)/550` per try). |
| Demon Bane | 42 | 40 | 80 | 300 | Melee gets "slay demon" (×5) for 20+1d20 turns. |

**[Wrath of the Valar]** (dungeon, DL60+)

| Prayer | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Banish Evil | 25 | 25 | 80 | 250 | Teleports every evil monster in view 100 squares away (no save; uniques too). |
| Word of Destruction | 35 | 35 | 80 | 115 | *Destruction* radius 15 around you (stairs and permanent walls survive; monsters including uniques are removed, objects destroyed); blinds you 10+1d10 turns unless you resist blindness or light. |
| Holy Word | 39 | 32 | 95 | 20 | `1d(4L)` to every evil monster in view; heals 1000 HP; cures fear, poison, stun, cuts. |
| Spear of Oromë | 40 | 10 | 75 | 130 | `HOLY_ORB` beam `(L/2)d8` (90 at L40, 112.5 at L50; ×2 vs evil). |
| Light of Varda | 42 | 40 | 85 | 200 | `LIGHT` ball radius 4, `5L` (210 at L42, 250 at L50; ×2 vs `HURT_LIGHT`, light breathers resist), then 100 `LIGHT_WEAK` to every light-sensitive monster in view. |

### 11.2 Priest prayer-by-prayer advice

- **Call Light** — ADVICE: the priest's light source supplement; light rooms on entry. Lighting a
  corridor? Use Spear of Light.
- **Detect Evil** — ADVICE: very cheap and catches invisible evil (ghosts, many casters). Blind spots:
  animals, golems, vortices, hounds, insects, jellies and other non-evil monsters (orcs, kobolds,
  dragons, undead and demons are evil by monster base).
- **Minor Healing** — ADVICE: early-game healing that saves CLW potions (and stops bleeding/stun). It
  is never a combat heal later (30–60 HP).
- **Bless** — ADVICE: cast before melee for +10 to-hit/+5 AC; it is the priest's early "attack spell".
- **Sense Invisible** — ADVICE: for ghost levels; cheaper than potions.
- **Heroism** — FACT: useless as a buff before L20; even at L20 it lasts 2–3 turns. ADVICE: before L20
  it is a 2-SP "cure fear + heal 10", which is still worth having for fear (low 30% base fail so it works
  while afraid).
- **Orb of Draining** — ADVICE: *the* priest attack from L7 to the late game: unresistable, ball, double
  vs evil (30% of DL1–10 monsters, ~60% at DL31–50). 41 damage on an evil monster at L7 for 7 SP. Buy
  Cleansing Power (100 gold) as soon as you can; since learning is random, at L7 study from Cleansing
  Power expecting Orb or Spear of Light (50/50) — study the other one at L8 regardless.
- **Spear of Light** — ADVICE: lights corridors (great for seeing what's coming); damages only
  `HURT_LIGHT` monsters (snagas, cave orcs, orc trackers/shamans/archers, Sméagol, jellies, worm
  masses, floating eyes, forest/stone/cave/snow trolls incl. Bert/Bill/Tom, wights, ghouls, vampires,
  liches — check recall).
- **Dispel Undead / Dispel Evil** — ADVICE: line-of-sight damage, one roll for all; 14/20 SP is expensive
  early. Dispel Evil shines against groups of evil monsters in a room once your SP pool is large; for
  single targets Orb is better per SP.
- **Protection from Evil** — ADVICE: one of the strongest defensive prayers: most melee threats are evil,
  and at L30 79% of their hits are repelled — but **only from monsters of level ≤ your level**. Keep it
  up whenever you fight evil melee monsters at or below your level.
- **Remove Curse** — ADVICE: try it on curses you want gone; failure makes the item fragile and a second
  failure can destroy it. For artifacts you care about, cast only when your strength `L+1d(L)` comfortably
  exceeds the curse power ([Chapter 6](06-equipment.md) for curse powers).
- **Portal** — ADVICE: the priest's only teleport, a mid-range jump; found in a DL30+ book. Until then,
  Phase Door and Teleportation scrolls are mandatory.
- **Remembrance / Restoration** — ADVICE: cast in town or safety; they replace Restore Life Levels and
  stat-restore potions. Both 90% base (50% cap).
- **Word of Recall** — ADVICE: saves buying WoR scrolls. Keep one scroll anyway (fail, blindness,
  confusion, no SP).
- **Healing** — ADVICE: 2000 HP is a full heal for everyone. 50 SP and 80% base: a big pool and some
  levels above 30 make it reliable. This is the priest's emergency button; it also cures amnesia (the
  only castable cure).
- **Clairvoyance** — ADVICE: the whole map plus object positions for 50 SP — superb for deciding
  whether a level is worth exploring.
- **Enchant Weapon / Enchant Armour** — ADVICE: cast repeatedly in town (rest between casts) to push
  gear to +10..+15; replaces expensive enchant scrolls.
- **Smite Evil / Demon Bane** — ADVICE: for melee priests; ×2 vs evil is weaker than most ego slays but
  stacks with nothing (best multiplier applies). Demon Bane ×5 is excellent against demons if you melee.
- **Glyph of Warding** — ADVICE: fight from a glyph in a corridor: melee monsters must break it to hit
  you (deep monsters break it more often: level 50 → ~9% per try). Summons can't land on it.
- **Banish Evil** — ADVICE: line-of-sight evil removal (uniques included) for 25 SP; the priest's
  crowd escape. Note the monsters are not killed — they come back.
- **Word of Destruction** — ADVICE: the classic last resort; deletes uniques too (they are *not* killed
  and will reappear later). Resist blindness/light to avoid being blinded.
- **Holy Word** — ADVICE: heal 1000 + cure + damage evil in view, all in one turn. At 95% base it stays
  at 50% for many levels — use Healing as the reliable heal until Holy Word's fail drops.
- **Spear of Oromë** — ADVICE: the priest's best single-target damage per SP late (180+ vs evil for 10 SP).
- **Light of Varda** — ADVICE: big area light damage; especially strong vs light-sensitive undead/orc
  crowds.

### 11.3 Priest priorities

**Workhorse attacks by level**

| Level | Single target | Groups |
|---|---|---|
| 1–6 | melee (blessed hafted weapon) + Bless | — |
| 7–39 | Orb of Draining | Orb of Draining; Dispel Evil/Undead when SP allow |
| 40–50 | Spear of Oromë (lines), Orb | Light of Varda, Holy Word, Orb |

**Utility**: Call Light, Detect Evil (L1); Bless; Sense Invisible (L3); Remove Curse (L16); WoR (L25);
Clairvoyance (L37); Remembrance/Restoration; Enchant spells.

**Emergency**: Minor Healing (early), Portal (L10, needs dungeon book), Healing (L30), Banish Evil (L25,
DL60 book), Holy Word (L39), Word of Destruction (L35), Glyph of Warding (L20).

**Traps / low value**: Heroism before L20 (cure-fear only), Spear of Light as an attack against
non-light-sensitive monsters (0 damage), Dispel spells against single targets, Glyph against
high-level breathers (they don't need to step on it).

---

## 12. Necromancer

INT, shadow, `ZERO_FAIL | CHOOSE_SPELLS | UNLIGHT | EVIL`, first spell L1, armour limit 30 lb, 26
rituals. A dark-adapted caster with an excellent cheap bolt, powerful but self-damaging high magic,
and the strongest single-target spells against living monsters. Official manual:
[birth](https://angband.readthedocs.io/en/latest/birth.html#necromancer).

### 12.1 Unlight: the necromancer's rules

FACT:

- **+25% spell failure when your own square is lit** (`spell_chance()`), added before the 50% cap.
- **Sight without light**: if your light radius is ≤ 1, you see grids at distance `< 2 + L/6 − light`
  (i.e. radius 1 at L1–5, 3 at L12, 6 at L30, 9 at L48) (`src/cave-view.c`). A light source of radius
  2+ turns this off. Items with a `+1 LIGHT` modifier are reduced by 1 for UNLIGHT characters
  (`calc_light()`), so +1-light gear does not spoil it.
- **Can cast and read without light** (own square always "seen").
- **Resist darkness** in the dungeon (`calc_bonuses()`), so Create Darkness and dark attacks don't blind you.
- **EVIL**: resist nether; *vulnerable* to holy orb.
- The class description says Unlight also gives stealth in the dark; this source has no stealth code
  tied to `UNLIGHT` (`player_property.txt` claims it) — treat that part as unverified.
- Starting kit has **no torch**. ADVICE: never carry a light source; fight from dark corridors; cast
  Create Darkness on lit rooms before fighting in them; stay off lit squares when casting anything
  important.

### 12.2 Necromancer rituals, resolved

**[Into the Shadows]** (town, 25 gold, starting book)

| Ritual | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Nether Bolt | 1 | 1 | 22 | 2 | `NETHER` bolt (never beams), `(L/4+3)d4`: 3d4 L1–3, 4d4 L4–7, 5d4 L8–11, 8d4 (20) L20–23, 10d4 (25) L28–31, 13d4 (32.5) L40–43, 15d4 (37.5) L48–50. **Undead: immune (0). Evil: half. Nether-immune non-undead: ×3/(7..12).** |
| Sense Invisible | 2 | 4 | 25 | 3 | See invisible 24+1d24 turns. |
| Create Darkness | 3 | 3 | 10 | 4 | Darkens the room you are in (or the area around you); harmless to monsters. Would blind a non-resistant caster; you resist dark. |
| Bat Form | 3 | 4 | 20 | 10 | Shapechange to bat (costs 5 HP): speed +3, stealth +3, infra +5, see invisible, feather fall, protection from blindness, to-dam −10. |
| Read Minds | 4 | 4 | 10 | 6 | Detects all monsters with `SPIRIT` (incl. invisible) in ±22/±40, then maps ±(L/10) rows × ±(L/10+1) columns around each. |

**[Dark Rituals]** (town, 400 gold, DL10+)

| Ritual | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Tap Unlife | 7 | **0** | 50 | 3 | Auto-targets the nearest visible undead: `(L/4+2)d6` damage (4d6 at L10, 7d6 at L20), and you gain `min(its HP, damage)/4` SP. Castable at 0 SP. |
| Crush | 10 | 6 | 40 | 5 | Every monster in line of sight whose **current** HP is below `4L` dies; others are untouched. You take `1d(2L)` damage per cast. |
| Sleep Evil | 12 | 5 | 50 | 5 | Sleep (power 10L+500) on every evil monster in LOS; resisted with probability = monster level (and by `NO_SLEEP`). |
| Shadow Shift | 15 | 10 | 50 | 10 | Teleport ~30 squares (±25%), then take 2d5 damage. |
| Disenchant | 16 | 12 | 50 | 15 | `DISEN` bolt `2d(2L+10)` (43 at L16, 71 at L30, 111 at L50); disenchant-immune monsters take ×3/(7..12). Non-resistant monsters with real (non-innate) spells get "disenchanted" for 5–14 turns: **+50% spell failure**. |

**[Fear and Torment]** (town, 1600 gold, DL30+)

| Ritual | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Frighten | 17 | 4 | 40 | 12 | Bolt: fear for L turns; save `min(90, level + max(0, 25 − L/2))`; `NO_FEAR` immune. Afraid monsters flee and fail spells 20% more. |
| Vampire Strike | 20 | 5 | 50 | 10 | Teleports you next to the **nearest visible living monster** and bites for `2L`; heals and feeds you by `min(2L, its HP+1)`. |
| Dispel Life | 22 | 20 | 50 | 10 | `1d(3L)` (one roll) to every living monster in view. |
| Dark Spear | 25 | 10 | 60 | 12 | `DARK` line (beam that also affects grids) `2d(2L)` (51 at L25, 61 at L30, 101 at L50); only dark breathers resist. |
| Warg Form | 28 | 20 | 60 | 80 | Shapechange to warg: +2 blows, +5 to-hit/dam; costs `1d(L)` HP; berserk 5+1d20 turns, cures fear. |

**[Deadly Powers]** (dungeon, DL40+)

| Ritual | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Banish Spirits | 25 | 25 | 80 | 25 | Teleports every `SPIRIT` monster in view 100 squares away (no save). |
| Annihilate | 30 | 20 | 75 | 25 | Bolt: exactly `4L` damage (120 at L30, 200 at L50) to a **living** monster; undead and `NONLIVING` (golems, elementals, vortices...) take 0. |
| Grond's Blow | 35 | 35 | 80 | 115 | *Destruction* radius 15; blinds unless you resist blindness or dark (you resist dark: no blindness). |
| Unleash Chaos | 38 | 15 | 90 | 100 | Randomly one of: chaos beam, chaos arc (20°, range 20) or chaos ball radius 3, each `8dL` (156 at L38, 204 at L50); chaos confuses and can polymorph non-uniques. Then **wakes every sleeping monster within 40 squares** and frightens (power L) everything in view. |
| Fume of Mordor | 40 | 50 | 80 | 150 | Darkens the whole level, maps it and **fully detects** every object (better than Clairvoyance). |
| Storm of Darkness | 43 | 16 | 80 | 200 | `DARK` ball radius 4, `4d(2L)` (174 at L43, 202 at L50). |

**[Corruption of Spirit]** (dungeon, DL60+)

| Ritual | Lv | SP | Fail | XP | What it does (FACT) |
|---|---|---|---|---|---|
| Power Sacrifice | 27 | 0 | 20 | 20 | Take 50 damage, restore 50 SP. |
| Zone of Unmagic | 32 | 16 | 60 | 110 | `DISEN` radius 4 centred on you, `3L`, **including you** (disenchants your equipment unless you resist disenchantment). |
| Vampire Form | 37 | 20 | 60 | 140 | Shapechange to vampire (+5 speed, +5 stealth, see invisible, hold life, light −1); costs **current HP / 4** (the in-game text says half — the data says `PLAYER_HP / 4`); vampiric melee for 10+1d20 turns (each hit on a living monster heals you by the damage dealt); then jumps to the nearest living monster and bites for L. |
| Curse | 40 | 50 | 60 | 180 | You take 100 damage; the targeted monster takes `(L/12+1)d(50 + % of its HP already lost)`: 4d50 (102) on an unhurt monster at L40, 4d100 (202) on a half-dead one, 5d150 at L48 on a nearly dead one. |
| Command | 45 | 40 | 60 | 250 | Take control of the target for 5+1d10 turns (resisted if `1d(L) < 1d(monster level)`). |

### 12.3 Necromancer ritual-by-ritual advice

- **Nether Bolt** — FACT/ADVICE: cheaper and stronger than Magic Missile (5d4 vs 4d4 at L10, 15d4 vs
  12d4 at L50), but **zero against undead** (4% of DL1–20 monsters, 13–27% deeper) and **half against
  evil non-undead** (orcs, kobolds, dragons, demons and most humanoid casters: 26–53% of monsters).
  Against an evil target Nether Bolt (half) does less than Magic Missile would at the same cost.
  Workhorse to the mid-30s.
- **Sense Invisible** — ADVICE: cheap; ghosts are a necromancer's problem (immune to nether).
- **Create Darkness** — ADVICE: cast before fighting in a lit room — it removes your +25% fail. It
  does not hurt monsters; it is purely for your fail rate (and makes you harder to see from a
  distance in an unlit room).
- **Bat Form** — ADVICE: weak (−10 damage) but +3 speed and see invisible; an early travel/escape form.
  You cannot read or quaff without reverting.
- **Read Minds** — ADVICE: the necromancer's early detection. Spirits are 34–74% of monsters; it misses
  mindless animals and golems. The mapping around each spirit is tiny at low level.
- **Tap Unlife** — ADVICE: free damage *and* mana against undead, which is exactly what Nether Bolt
  cannot hurt. It always picks the nearest undead. The necromancer's anti-undead tool from L7.
- **Crush** — ADVICE: a room-clearer against weak crowds (all monsters below 40 HP at L10, 120 at L30,
  200 at L50) and a finisher: anything you have wounded below 4L dies. Costs up to 2L HP per cast —
  watch your HP.
- **Sleep Evil** — ADVICE: escape/skip tool against evil packs; `NO_SLEEP` is rare at DL1–20 (23–25%).
- **Shadow Shift** — ADVICE: the necromancer's medium escape from L15 (Dark Rituals is a town book):
  30-square teleport for 10 SP and 2d5 HP. Keep Teleportation scrolls for long escapes.
- **Disenchant** — ADVICE: the best mid-level single-target bolt against undead and evil (it ignores
  both); also makes casters fail half their spells. Use it on dangerous spellcasters first.
- **Frighten** — ADVICE: a frightened melee monster can't hurt you; casters still cast (at +20% fail).
- **Vampire Strike** — ADVICE: damage + heal + food, but it teleports you *next to the nearest living
  monster*, which may not be the one you want. Use only when the nearest living visible monster is the
  one you want to bite and its neighbours are safe.
- **Dispel Life** — ADVICE: room damage against living crowds (animals, orcs, trolls), useless vs undead
  and golems.
- **Dark Spear** — ADVICE: a beam almost nothing resists; also darkens the grids it passes (good for
  your Unlight). Mid-game line attack.
- **Warg Form** — ADVICE: a melee form (+2 blows, berserk). Necromancers are weak in melee; mainly for
  when out of SP.
- **Banish Spirits** — ADVICE: clears most intelligent monsters from view; hounds/animals without
  SPIRIT stay.
- **Annihilate** — ADVICE: the necromancer's unique-killer: a flat 4L to any living target, no other
  resistance. Check `UNDEAD`/`NONLIVING` first (21–33% of DL31+ monsters are immune).
- **Grond's Blow** — ADVICE: Destruction without the blindness (you resist dark).
- **Unleash Chaos** — ADVICE: powerful but random shape, and it **wakes the level around you** — never
  use it while sneaking. Good when everything is already awake.
- **Fume of Mordor** — ADVICE: full map + full object knowledge; also darkens every room on the level,
  which removes the lit-square fail penalty everywhere.
- **Storm of Darkness** — ADVICE: big dark ball almost nothing resists; the late-game area attack.
- **Power Sacrifice** — ADVICE: 50 HP → 50 SP, zero SP cost, 20% base fail: a strong mana battery when
  you have HP to spare and a heal available.
- **Zone of Unmagic** — TRAP without resist disenchantment: you disenchant your own gear.
- **Vampire Form** — ADVICE: strong late melee/regain form; remember you can't use items while a vampire.
- **Curse** — ADVICE: a finisher: weak on healthy targets (100 HP cost for ~100 damage), strong on
  badly wounded ones (200+). Never cast below ~150 HP.
- **Command** — ADVICE: novelty/utility (use a monster as a pawn, move it out of the way, make it
  drop items). Level-vs-level resist makes it unreliable on deep monsters.

### 12.4 Necromancer priorities

**Workhorse attacks by level**

| Level | Non-undead | Undead |
|---|---|---|
| 1–6 | Nether Bolt | melee, avoid |
| 7–15 | Nether Bolt; Crush for weak crowds (L10) | Tap Unlife |
| 16–29 | Nether Bolt (non-evil), Disenchant (evil/casters) | Disenchant, Tap Unlife |
| 25–29 | + Dark Spear (lines), Dispel Life (living crowds) | Dark Spear, Disenchant |
| 30–42 | Annihilate (tough/unique), Nether Bolt | Disenchant, Dark Spear |
| 43–50 | Annihilate, Storm of Darkness, Unleash Chaos | Storm of Darkness, Dark Spear |

**Utility**: Read Minds (L4), Sense Invisible (L2), Create Darkness (L3), Fume of Mordor (L40).

**Emergency**: Shadow Shift (L15), Sleep Evil (L12), Frighten (L17), Banish Spirits (L25), Grond's Blow
(L35), Power Sacrifice (L27, mana). No healing beyond Vampire Strike/Form; carry potions.

**Traps / low value**: Zone of Unmagic without rDisen; Curse at low HP; Unleash Chaos when stealth
matters; Vampire Strike next to a dangerous crowd; Crush when low on HP; casting anything on a lit
square.

---

## 13. Paladin

WIS, divine, **random learning**, no `ZERO_FAIL` (5% minimum fail), first spell L1, armour limit
40 lb, 16 prayers in 3 books (one town). Mana is modest; prayers are buffs and self-heals for a melee
character. Official manual: [birth](https://angband.readthedocs.io/en/latest/birth.html#paladin).

**[Novice's Handbook]** (town, starting book)

| Prayer | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Bless | 1 | 2 | 20 | 4 | +5 AC, +10 to-hit for `(L+10)+1d(L+10)` turns. ADVICE: cast before every melee fight early. |
| Detect Evil | 3 | 2 | 10 | 4 | Evil monsters in ±22/±40. ADVICE: the paladin's only detection; use it constantly. |
| Call Light | 5 | 3 | 10 | 2 | Light room, `2d(L/2)` to adjacent light-sensitive monsters. |
| Minor Healing | 7 | 3 | 15 | 4 | Heals `max(L+10, L/4 % of missing)`, −20 cuts/stun. ADVICE: early heal; cut/stun cure. |
| Sense Invisible | 8 | 4 | 25 | 4 | See invisible 24+1d24. |
| Heroism | 12 | 5 | 30 | 5 | Heal 10, cure fear; heroism for `(L−14)+1d(L−14)` turns from L15 (1 at L15, 6+1d6 at L20). ADVICE: the paladin gets real heroism 5 levels earlier than the priest. |

**[Healing and Sanctuary]** (dungeon, DL30+)

| Prayer | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Protection from Evil | 15 | 8 | 42 | 5 | As priest. ADVICE: superb for a melee class — keep it up vs evil melee of level ≤ yours. |
| Remove Curse | 20 | 12 | 38 | 8 | As priest (twice the SP). |
| Word of Recall | 25 | 30 | 75 | 10 | As priest. |
| Healing | 30 | 50 | 80 | 100 | 2000 HP + cures. **Needs 50 SP**: a paladin with WIS 16 has only 40 SP at L30, 53 at L40 — raise WIS or you can't cast it safely. |
| Clairvoyance | 37 | 50 | 80 | 150 | As priest. |

**[Battle Blessings]** (dungeon, DL40+)

| Prayer | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Smite Evil | 25 | 20 | 70 | 40 | Slay evil ×2 for 20+1d20 turns. ADVICE: only useful if your weapon lacks a better slay vs the target. |
| Demon Bane | 30 | 40 | 80 | 150 | Slay demon ×5 (the paladin gets it 12 levels before the priest). |
| Enchant Weapon | 35 | 50 | 80 | 230 | As priest. |
| Enchant Armour | 37 | 60 | 85 | 250 | As priest. |
| Single Combat | 40 | 50 | 30 | 300 | Moves you and the target to a closed arena level; only the winner leaves. Resisted if `1d(spell power) > L`. No teleport, banishment or recall works inside. ADVICE: a duel with a unique you can out-melee, away from its escort; a death trap against one you can't. |

**Paladin priorities.** Workhorse: melee (no attack prayers). Utility: Bless (L1), Detect Evil (L3), Call
Light (L5), Heroism (L12/15), PfE (L15), Remove Curse (L20), WoR (L25). Emergency: Minor Healing (early),
Healing (L30, if SP ≥ 50). Traps: Single Combat against a stronger unique; Healing with insufficient SP
(over-exertion paralysis). No escapes: carry Phase Door, Teleportation, Deep Descent/Teleport Level.

---

## 14. Rogue

INT, arcane, `CHOOSE_SPELLS`, no `ZERO_FAIL` (5% floor), **first spell L5**, armour limit 35 lb, 10
spells in 2 books. Detection and escapes for a stealthy fighter/thief; no attack spells. Official
manual: [birth](https://angband.readthedocs.io/en/latest/birth.html#rogue).

**[First Spells]** (town, starting book — rogue version)

| Spell | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Detect Monsters | 5 | 1 | 50 | 4 | Non-invisible monsters ±22/±40. ADVICE: learn first; it's 50% at L5 (capped), quickly improves. |
| Phase Door | 7 | 2 | 55 | 4 | ~10-square teleport. ADVICE: carry scrolls until its fail is low (~L20 with decent INT). |
| Object Detection | 10 | 3 | 60 | 5 | **Fully detects** objects, gold and treasure veins in ±22/±40 (better than the Mage's Treasure Detection, which only senses). ADVICE: superb for a thief-diver — find the good floor items and leave. |
| Detect Stairs | 12 | 3 | 50 | 5 | Stairs in a *wide* area (±44 rows, ±60 columns). ADVICE: the rogue's diving engine. |
| Recharging | 20 | 10 | 50 | 6 | Strength `L/10+4` (6 at L20, 9 at L50 — weaker than the Mage's). |
| Reveal Monsters | 25 | 3 | 40 | 8 | All monsters including invisible ±22/±40. |

**[Arcane Control]** (dungeon, DL40+ — rogue version)

| Spell | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Teleport Self | 17 | 6 | 35 | 5 | Same formula as the Mage (`9L/5`% of edge distance): 30% at L17, 54% at L30. |
| Hit and Run | 23 | 20 | 40 | 20 | For 10 turns, your next successful steal teleports you ~20 squares. ADVICE: the safe way to steal from dangerous monsters. |
| Teleport Other | 30 | 10 | 30 | 50 | As Mage (3L distance, no save). |
| Teleport Level | 35 | 17 | 65 | 80 | As Mage. |

**Rogue priorities.** Detection: Detect Monsters (L5), Object Detection (L10), Detect Stairs (L12), Reveal
Monsters (L25). Emergency: Phase Door (L7), Teleport Self (L17, needs a DL40+ book), Teleport Other (L30),
Teleport Level (L35). Utility: Recharging (L20) for the rogue's good device skill. Traps: relying on any
rogue spell at 50% (fail is capped, never below 5%).

---

## 15. Ranger

WIS, nature, `CHOOSE_SPELLS | FAST_SHOT`, no `ZERO_FAIL`, **first spell L3**, armour limit 40 lb, 11
verses in 2 books. Survival magic for an archer. Official manual:
[birth](https://angband.readthedocs.io/en/latest/birth.html#ranger).

**[Lesser Charms]** (town, starting book — ranger version)

| Verse | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Remove Hunger | 3 | 1 | 25 | 2 | Food to 50%. ADVICE: learn first; no more rations. |
| Detect Life | 5 | 2 | 23 | 2 | Living monsters ±22/±40 (misses undead/nonliving). |
| Herbal Curing | 9 | 6 | 30 | 4 | Cures cuts, poison, stun and Black Breath (no food). ADVICE: the ranger gets this 31 levels before the druid, at 30% base instead of 90%. Huge. |
| Resist Poison | 12 | 10 | 32 | 5 | rPois 20+1d20 turns. |
| Turn Stone to Mud | 15 | 5 | 25 | 5 | As druid (dig; 20+1d30 to `HURT_ROCK`). |
| Sense Surroundings | 20 | 8 | 35 | 8 | Map ±22 × ±44 (doubles at L30). |

**[Nature Craft]** (dungeon, DL40+ — ranger version)

| Verse | Lv | SP | Fail | XP | What it does / advice |
|---|---|---|---|---|---|
| Cover Tracks | 20 | 20 | 40 | 40 | For `2L+1d20` turns: you leave no scent, your noise falls off 4× faster, monsters more than 5 squares away cannot "see" you, and short-range monster projections are cut to 1/4 range. ADVICE: a strong stealth/sniping buff. |
| Create Arrows | 22 | 20 | 40 | 40 | Destroys a staff, makes a stack of arrows of your level; "good" if 1d(staff level) > 25, "great" if additionally 1d(staff level) > 50. ADVICE: turn junk deep staves into ammo. |
| Haste Self | 25 | 12 | 65 | 15 | +10 speed, `L+1d20` turns (+5 if recast while hasted). ADVICE: the reason to find this book. |
| Decoy | 30 | 30 | 40 | 60 | Places a decoy (one at a time) that monsters target instead of you; destroyed when attacked. ADVICE: shoot from safety while they chase the decoy. |
| Brand Ammunition | 40 | 60 | 95 | 120 | Brands one ammo stack with fire, cold or poison at random (1/3 flame, 1/3 frost, 1/3 venom) and tries to enchant it; fails on already branded/slain/cursed/broken ammo. |

**Ranger priorities.** Workhorse: the bow (no attack spells). Utility: Remove Hunger (L3), Detect Life
(L5), Stone to Mud (L15), Sense Surroundings (L20), Create Arrows (L22). Emergency/cure: Herbal Curing
(L9), Resist Poison (L12), Haste Self (L25), Decoy (L30), Cover Tracks (L20). No escape spells: carry
Phase/Teleport scrolls.

---

## 16. Blackguard

INT, shadow, `CHOOSE_SPELLS | COMBAT_REGEN | SHIELD_BASH`, intrinsic `IMPAIR_HP` (slower HP regen),
no `ZERO_FAIL`, first spell L1, armour limit **60 lb**, 15 spells in 3 books. Melee spells powered by
mana gained in combat ([6.3](#63-blackguard-mana-combat_regen)). Official manual:
[birth](https://angband.readthedocs.io/en/latest/birth.html#blackguard).

**[Into the Shadows]** (town, starting book — blackguard version)

| Spell | Lv | SP | Fail | XP | What it does (FACT) and advice |
|---|---|---|---|---|---|
| Seek Battle | 1 | 1 | 15 | 2 | Detects monsters that can be frightened (no `NO_FEAR`), incl. invisible, ±22/±40. ADVICE: the BG's monster detection; misses fearless monsters (golems, many undead — 20–30% of monsters). |
| Berserk Strength | 3 | 2 | 20 | 4 | Cures fear; berserk for `(L+5)+1d(L+5)` turns: +75 melee skill, −10 AC, immune to fear. (The in-game text says it stops HP regeneration; no such code was found — unverified.) ADVICE: cast before every serious melee. |
| Whirlwind Attack | 5 | 4 | 25 | 6 | One blow against every adjacent monster, repeated `(L+10)/15` times (1 at L5, 2 at L20, 3 at L35, 4 at L50). ADVICE: when surrounded. |
| Shatter Stone | 7 | 3 | 20 | 8 | Destroys one adjacent wall/rubble grid (not permanent); 20+1d30 to `HURT_ROCK`. ADVICE: digging tool. |
| Leap into Battle | 10 | 5 | 30 | 10 | Moves up to 4 squares toward a visible target and makes `(L+5)/15` blows (1, 2 at L25, 3 at L40), reduced by 25% per square moved (rounded). Stops if blocked; attacks anything it bumps into. ADVICE: closes the gap with archers/casters; adjacent use gives full blows. |
| Grim Purpose | 13 | 8 | 30 | 12 | Protection from confusion + Free Action for 12+1d12 turns. ADVICE: before fighting paralysers/confusers if you lack FA or pConf. |

**[Fear and Torment]** (town, 1600 gold, DL30+ — blackguard version)

| Spell | Lv | SP | Fail | XP | What it does (FACT) and advice |
|---|---|---|---|---|---|
| Maim Foe | 15 | 6 | 35 | 14 | `L/15` blows (1, 2 at L30, 3 at L45) against an adjacent target; each blow that damages also stuns it for 6 turns (stunned monsters lose 1 turn in 10, −25% to-hit/damage; `NO_STUN` immune). |
| Howl of the Damned | 18 | 8 | 30 | 16 | Fear (power L) on every monster in view (save as Frighten). ADVICE: scatter a melee crowd; frightened monsters still use ranged attacks. |
| Relentless Taunting | 24 | 10 | 32 | 20 | 12+1d12 turns: monsters want to close to melee and cast half as often. ADVICE: perfect for a melee character facing casters/archers. |
| Venom | 28 | 18 | 35 | 30 | Poison brand ×3 on melee for 18+1d18 turns (`IM_POIS` monsters immune). |
| Werewolf Form | 32 | 30 | 40 | 100 | Shapechange: +1 blow, double move speed, +5 to-hit/dam, regeneration; frightens everything in view (power L), cures fear, berserk 5+1d20. |

**[Deadly Powers]** (dungeon, DL40+ — blackguard version)

| Spell | Lv | SP | Fail | XP | What it does (FACT) and advice |
|---|---|---|---|---|---|
| Bloodlust | 30 | 25 | 37 | 80 | Bloodlust timer 10 (cap 50); each kill +10. Gives `+timer/2` to-dam and `+timer/20` blows. Each command has `timer/200` chance to be replaced by an attack on a random adjacent monster (up to 25%, even when you wanted to quaff or read). Kills may confuse/hallucinate you (5% each); misses may scramble stats (1/50), hits may drain CON (1/50); as it wanes below 10 you risk HP loss, cuts, slowness. **While active you survive HP down to −(timer + L)**. Recasting while active *reduces* the timer by 1 (the `-1` extend parameter). |
| Unholy Reprieve | 34 | 45 | 60 | 120 | Restores STR, INT, CON and experience; costs 66 HP. |
| Forceful Blow | 38 | 16 | 50 | 150 | One melee blow; if it damages, the target also takes a **200-damage `FORCE` projection** (1/3 stun, knockback `3+dam/20`; only force breathers resist). ADVICE: the BG's big hitter. |
| Quake | 44 | 39 | 55 | 175 | Earthquake radius 13 centred on you. ADVICE: area denial/escape; destroys floor items. |

**Blackguard priorities.** Workhorse: melee + Berserk Strength (L3), Leap into Battle (L10), Maim Foe
(L15), Venom (L28), Forceful Blow (L38). Crowds: Whirlwind (L5), Howl (L18). Utility: Seek Battle (L1),
Shatter Stone (L7), Grim Purpose (L13), Relentless Taunting (L24). Emergency: Howl, Werewolf Form, Quake;
otherwise scrolls. Traps: opening a fight with spells (you have little SP until you fight); Bloodlust
when you need to drink potions reliably; recasting Bloodlust; Unholy Reprieve at low HP.

---
## 17. Shapechanges

FACT (`lib/gamedata/shape.txt`, `src/player-calcs.c:calc_shapechange()`, `src/player-util.c`,
`src/cmd-core.c:cmd_get_item()`; official manual: [playing — Shape Changes](https://angband.readthedocs.io/en/latest/playing.html),
vendored [user-manual/playing.rst](user-manual/playing.rst)).

**What you can't do while shapechanged:** cast or study, read scrolls, quaff potions, aim wands, use
staves, zap rods, activate, wield/take off, inscribe, refuel, fire missiles, throw, or enter a shop
(the shop door slams shut; your home is allowed). Commands that need an item can only see items **on
the floor** — so you can still eat food lying on the floor and pick things up. Each blocked command
prompts: `y` = revert (free, no energy) and do it; `r` = just revert; `n` = cancel. Reverting also ends
vampiric melee (`ATT_VAMP`). There is no time limit: a shape lasts until you revert.

**What carries over:** your equipment stays worn and keeps working (stats, resists, blows, weapon
damage). Shape blows are only flavour verbs — damage still comes from your weapon. Shape modifiers
below are *added* to your normal values.

| Shape | Source (level) | to-hit/to-dam/AC | Modifiers and flags | Cost on change |
|---|---|---|---|---|
| fox | Druid Fox Form (1) | −3/−3/+3 | STR −3, stealth +5, **+1 blow, +1 move** (moves cost 1/2 energy), Feather Fall, Free Action | — |
| bat | Necro Bat Form (3) | 0/−10/+10 | speed +3, stealth +3, infravision +5, Feather Fall, See Invisible, prot. blindness | 5 HP |
| warg | Necro Warg Form (28) | +5/+5/0 | **+2 blows** | 1d(L) HP; cures fear; berserk 5+1d20 |
| Pukel-man | Druid (30) | 0/+5/+20 | STR +4, CON +4, **speed −5**, stealth −2, save +20, poison **immunity**, rShards, damage reduction 10, sustain STR/CON, prot. stun, Regeneration, Hold Life, `ROCK` (cuts neither bleed nor heal) | cures poison and stun |
| werewolf | Blackguard (32) | +5/+5/0 | +1 blow, +1 move, Regeneration | frightens all in view (power L); cures fear; berserk 5+1d20 |
| eagle | Druid Eagle's Flight (35) | +5/0/+10 | **+3 moves** (1/4 energy per move), sustain WIS, prot. blind/conf/fear, Free Action, See Invisible, trap immunity | — |
| vampire | Necro Vampire Form (37) | +3/+3/−5 | speed +5, stealth +5, infravision +5, light −1, See Invisible, Hold Life | current HP/4; vampiric melee 10+1d20 turns; jump-and-bite L |
| bear | Druid Bear Form (40) | +15/+15/+5 | STR +3, CON +2, INT −2, infravision +1, stealth −3, **+1 blow**, melee skill +10, disarm −5/−10, prot. fear | — |

ADVICE (tradeoffs):

- **Movement shapes (fox, werewolf, eagle)** are the only "speed for movement" a druid or blackguard
  gets early. Extra moves only make *walking* cheaper; attacks and everything else cost normal energy.
  They are excellent for crossing explored ground and for running from slow melee monsters; useless
  against fast or ranged threats. Before shifting, make sure you won't need a potion or scroll in the
  next few turns: reverting is free but the item use then costs your turn as usual.
- **Combat shapes (bear, warg, werewolf, vampire, Pukel-man)** turn casters into melee characters for
  a while. Worth it when out of mana or against a monster that resists your spells. Pukel-man's −5
  speed usually makes it a bad choice unless you are cornered by something slow.
- **Bat** is mainly +3 speed and see invisible for a low-level necromancer; −10 to-dam ruins melee.
- **For a borg:** treat "shapechanged" as "no consumables available". Never shift when HP < 50% or
  when an unknown/fast monster is in view; revert proactively (via a harmless command answered `r`)
  before the situation might call for items.

---

## 18. Cross-class lookup: "I need X"

Spell level in brackets; **(D)** = from a dungeon book (not buyable in town); "—" = no spell, use items.

| Need | Mage | Druid | Priest | Necromancer | Paladin | Rogue | Ranger | Blackguard |
|---|---|---|---|---|---|---|---|---|
| Short escape (blink) | Phase Door (2) | Fox Form (1, run) | Portal (10, D) | Shadow Shift (15) | — | Phase Door (7) | — | Leap into Battle is the opposite |
| Long escape (teleport) | Teleport Self (7; ≥ scroll from L34) | — | Portal (10, D; 35–55 sq) | Shadow Shift (15; 30 sq) | — | Teleport Self (17, D) | — | — |
| Leave the level now | Teleport Level (28, D) | — | — | — | — | Teleport Level (35, D) | — | — |
| Word of Recall | — | — | WoR (25, D) | — | WoR (25, D) | — | — | — |
| Deep Descent | — | — | — | — | — | — | — | — |
| Remove one monster | Teleport Other (15) | Rift (35, D, chance) | Banish Evil (25, D, all evil) | Banish Spirits (25, D) | Single Combat (40, D, duel) | Teleport Other (30, D) | — | — |
| Remove many monsters | Banishment (35, D), Mass Banishment (40, D) | — | Banish Evil (25, D), Word of Destruction (35, D) | Banish Spirits (25, D), Grond's Blow (35, D) | — | — | — | Quake (44, D) |
| Detect monsters | Detect Monsters (3), Reveal (15), Detection (30, D) | Detect Life (1) | Detect Evil (1) | Read Minds (4) | Detect Evil (3) | Detect Monsters (5), Reveal (25) | Detect Life (5) | Seek Battle (1) |
| Detect invisible monsters | Reveal Monsters (15) | Detect Life (1, living only) | Detect Evil (1, evil only) | Read Minds (4, spirits) | Detect Evil (3) | Reveal Monsters (25) | Detect Life (5) | Seek Battle (1, fearful) |
| See invisible | — | Eagle form (35, D) | Sense Invisible (3) | Sense Invisible (2), bat/vampire form | Sense Invisible (8) | — | — | — |
| Detect objects | Treasure Detection (10, sense), Detection (30, D) | — | Clairvoyance (37, D, sense whole level) | Fume of Mordor (40, D, full, whole level) | Clairvoyance (37, D) | Object Detection (10, full) | — | — |
| Detect traps | Find Traps/Doors/Stairs (1), Detection (30, D) | — | — | — | — | — | — | — |
| Detect doors/stairs | Find Traps/Doors/Stairs (1) | Sense Surroundings (9, map) | Clairvoyance (37, D) | Read Minds (4, partial map), Fume (40, D) | Clairvoyance (37, D) | Detect Stairs (12, wide) | Sense Surroundings (20) | — |
| Map the area | — | Sense Surroundings (9) | Clairvoyance (37, D) | Fume of Mordor (40, D) | Clairvoyance (37, D) | — | Sense Surroundings (20) | — |
| Light a room | Light Room (1) | — | Call Light (1) | (darkens instead) | Call Light (5) | — | — | — |
| Heal HP | — | Rapid Regeneration (37, D) | Minor Healing (1), Healing (30, D), Holy Word (39, D) | Vampire Strike (20), Vampire Form (37, D) | Minor Healing (7), Healing (30, D) | — | — | casting heals (COMBAT_REGEN) |
| Cure poison | — | Cure Poison (4), Herbal Curing (40, D) | Healing (30, D), Holy Word (39, D) | — | Healing (30, D) | — | Herbal Curing (9) | — |
| Cure stun / cuts | — | Herbal Curing (40, D); Pukel-man form (30, D) | Minor Healing (1, −20), Healing, Holy Word | — | Minor Healing (7), Healing | — | Herbal Curing (9) | — |
| Cure confusion / blindness | impossible (can't cast while confused/blind) — use potions; prevention below | | | | | | | |
| Prevent confusion/blindness | — | Eagle form (35, D) | — | Bat form (blind only) | — | — | — | Grim Purpose (13, conf + FA) |
| Cure fear | — | Bear form (40, D) | Heroism (5), Holy Word (39, D) | Warg Form (28) | Heroism (12) | — | — | Berserk Strength (3), Werewolf (32) |
| Cure Black Breath | — | Herbal Curing (40, D) | — | — | — | — | Herbal Curing (9) | — |
| Food | — | Remove Hunger (2) | — | Vampire Strike (20) | — | — | Remove Hunger (3) | — |
| Restore stats/exp | — | Revitalize (35, D) | Remembrance (20, D), Restoration (35, D) | — | — | — | — | Unholy Reprieve (34, D; STR/INT/CON+exp) |
| Speed | — | Haste Self (25, D) | — | Bat (+3), Vampire Form (+5, D) | — | — | Haste Self (25, D) | — |
| Faster casting | Mana Channel (25) | — | — | — | — | — | — | — |
| Temporary resists | Resistance (20; acid/elec/fire/cold) | Resist Poison (7); Pukel-man (30, D; immune poison, rShards) | — | (innate rDark, rNether) | — | — | Resist Poison (12) | Grim Purpose (13; pConf+FA) |
| Protection from evil | — | — | PfE (14) | — | PfE (15, D) | — | — | — |
| To-hit / AC buff | — | — | Bless (1), Heroism (L20+) | — | Bless (1), Heroism (L15+) | — | — | Berserk Strength (3) |
| Melee slay/brand | — | — | Smite Evil (25, D), Demon Bane (42, D) | — | Smite Evil (25, D), Demon Bane (30, D) | — | Brand Ammunition (40, D; ammo) | Venom (28) |
| Identify | Identify Rune (8) | — | — | — | — | — | — | — |
| Recharge | Recharging (5) | — | — | — | — | Recharging (20) | — | — |
| Remove curse | — | — | Remove Curse (16) | — | Remove Curse (20, D) | — | — | — |
| Enchant gear | — | — | Enchant Weapon (35, D), Armour (37, D) | — | same (35/37, D) | — | Brand Ammunition (40, D) | — |
| Dig | Disable Traps/Destroy Doors (doors only) | Stone to Mud (8), Tremor | — | — | — | — | Stone to Mud (15) | Shatter Stone (7) |
| Kill a crowd | Fire Ball (6), Acid Spray (20), Mana Storm (45, D) | Stinking Cloud (3), Ice Storm (37, D), River of Lightning (43, D) | Orb (7), Dispel Evil (12), Holy Word (39, D), Light of Varda (42, D) | Crush (10), Dispel Life (22), Storm of Darkness (43, D) | — | — | — | Whirlwind (5), Howl (18, fear) |
| Kill a unique | Frost Bolt (13), Mana Bolt (25, D) | Lightning Strike (12), Meteor Swarm (30, D), Rift (35, D) | Orb (7; ×2 evil), Spear of Oromë (40, D) | Disenchant (16), Annihilate (30, D), Curse (40, D) | melee + Bless/Heroism | — | — | Maim Foe (15), Forceful Blow (38, D), Bloodlust (30, D) |
| Disable a monster | — | Confuse (5), Slow (6), Trance (20, D), Mass Sleep (25, D) | — | Sleep Evil (12), Frighten (17), Disenchant (16, spells) | — | — | — | Maim Foe (15, stun), Howl (18) |
| Make monsters come to you | — | — | — | — | — | — | Decoy (30, D) diverts them | Relentless Taunting (24) |
| Restore mana | Tap Magical Energy (22) | — | — | Tap Unlife (7), Power Sacrifice (27, D) | — | — | — | melee/being hit |

Gaps worth noting: **nobody has a Deep Descent spell**; only the
Mage detects traps; only Priest/Paladin recall; only Druid/Ranger haste; nobody can cure confusion or
blindness with a spell.

---

## 19. Casting decision procedure for a borg

### 19.1 Projection vs monster resistance

FACT (`src/project-mon.c`). "×r" below means damage × factor / (1d6+6), i.e. ×(factor/7 .. factor/12),
average ≈ factor/9.3.

| Projection | Player spells using it | Reduced/immune | Increased | Side effects |
|---|---|---|---|---|
| `MISSILE` | Magic Missile, Mana Bolt | nothing | — | — |
| `MANA` | Mana Storm | nothing | — | — |
| `METEOR` | Meteor Swarm | nothing | — | — |
| `HOLY_ORB` | Orb of Draining, Spear of Oromë | nothing | ×2 `EVIL` | — |
| `FIRE` | Fire Ball, Volcanic Eruption | `IM_FIRE` ÷9 | ×2 `HURT_FIRE` | burns floor items |
| `COLD` | Frost Bolt | `IM_COLD` ÷9 | ×2 `HURT_COLD` | — |
| `ICE` | Ice Storm | `IM_COLD` ÷9 | ×2 `HURT_COLD` | 1/3 stun |
| `ELEC` | Electric Arc, Lightning Strike (part) | `IM_ELEC` ÷9 | — | destroys floor items |
| `ACID` | Acid Spray | `IM_ACID` ÷9 | — | destroys floor items |
| `POIS` | Stinking Cloud | `IM_POIS` ÷9 | — | — |
| `NETHER` | Nether Bolt | `UNDEAD` **0**; `IM_NETHER` ×3r; `EVIL` ÷2 | — | — |
| `LIGHT` | Light of Varda | light breathers ×2r | ×2 `HURT_LIGHT` | — |
| `LIGHT_WEAK` | Light Room, Call Light, Spear of Light, Light of Varda (part) | **everything not `HURT_LIGHT` takes 0** | — | lights grids |
| `DARK` | Dark Spear, Storm of Darkness | dark breathers ×2r | — | darkens grids |
| `SOUND` | Shock Wave, Lightning Strike (part) | sound breathers ×2r | — | 1/3 stun 5–14 |
| `SHARD` | Earth Rising, Explosion | shard breathers ×3r | — | — |
| `FORCE` | Thrust Away, Explosion, Forceful Blow | force breathers ×3r | — | 1/3 stun, knockback |
| `GRAVITY` | Rift | gravity breathers ×3r (and no blink) | — | blink 10 if 1d127 > level |
| `CHAOS` | Unleash Chaos | chaos breathers ×3r | — | confusion 10–20; polymorph non-uniques |
| `DISEN` | Disenchant, Zone of Unmagic | `IM_DISEN` ×3r | — | +50% monster spell failure 5–14 turns |
| `PLASMA` | River of Lightning | `IM_PLASMA` ×3r | — | — |
| `MON_DRAIN` | Annihilate, Dispel Life | `UNDEAD` or `NONLIVING`: 0 | — | — |
| `DISP_UNDEAD` / `DISP_EVIL` | Dispel Undead/Evil, Holy Word | non-matching: 0 | — | — |
| `MON_CRUSH` | Crush | current HP ≥ damage: 0 | — | kills outright otherwise |
| `KILL_WALL` | Stone to Mud, Shatter Stone | non-`HURT_ROCK`: 0 | — | digs |
| sleep/conf/slow/hold/fear | see [7.3](#73-monster-saving-throws-against-status-spells) | flags; saves for sleep/fear only | ×1.5 duration vs animals (Druid) | — |

Monster spell failure (FACT, `src/mon-attack.c:monster_spell_failrate()`): a non-stupid monster fails
24% of its spells, +20% if afraid, +50% if confused **or disenchanted**. (The code computes
`power = MIN(spell_power, 1)`, which makes the base 24% for every monster — probably intended as `MAX`;
treat the exact base as version-specific.) Confuse Monster, Disenchant, Unleash Chaos and Frighten
therefore also cripple casters.

### 19.2 Monster flag prevalence by depth

FACT, counted from `lib/gamedata/monster.txt` + `monster_base.txt` (flags inherited from the base; by
native depth band; uniques included):

| Property | DL1–10 | 11–20 | 21–30 | 31–40 | 41–50 | 51–70 | 71–100 |
|---|---|---|---|---|---|---|---|
| monsters in band | 140 | 95 | 93 | 101 | 53 | 65 | 60 |
| `UNDEAD` (0 from nether, drain) | 4% | 4% | 11% | 14% | 13% | 22% | 27% |
| `UNDEAD` or `NONLIVING` (0 from drain) | 4% | 12% | 18% | 33% | 21% | 29% | 32% |
| `EVIL` | 30% | 39% | 51% | 58% | 66% | 63% | 72% |
| evil non-undead (nether halved) | 26% | 35% | 40% | 45% | 53% | 43% | 45% |
| `IM_FIRE` | 5% | 15% | 20% | 26% | 43% | 43% | 60% |
| `IM_COLD` | 9% | 16% | 27% | 34% | 53% | 51% | 65% |
| `IM_ELEC` | 3% | 11% | 13% | 23% | 25% | 26% | 35% |
| `IM_ACID` | 9% | 11% | 5% | 15% | 26% | 28% | 32% |
| `IM_POIS` | 29% | 29% | 39% | 46% | 49% | 58% | 70% |
| `HURT_LIGHT` | 16% | 6% | 13% | 10% | 9% | 11% | 20% |
| `HURT_FIRE` | 9% | 8% | 9% | 6% | 6% | 5% | 2% |
| `HURT_COLD` | 16% | 11% | 6% | 5% | 8% | 3% | 5% |
| `SPIRIT` | 34% | 35% | 46% | 54% | 70% | 74% | 72% |
| `ANIMAL` | 42% | 40% | 35% | 15% | 15% | 22% | 15% |
| `NO_FEAR` | 26% | 27% | 20% | 30% | 19% | 18% | 27% |
| `NO_SLEEP` | 23% | 25% | 37% | 67% | 85% | 89% | 97% |
| `NO_CONF` | 23% | 27% | 37% | 68% | 85% | 91% | 97% |
| `NO_SLOW` | 0% | 1% | 0% | 2% | 2% | 3% | 2% |
| `NO_HOLD` | 2% | 3% | 16% | 24% | 57% | 72% | 87% |
| `NO_STUN` | 6% | 5% | 12% | 19% | 19% | 28% | 38% |
| `IM_DISEN` | 1% | 0% | 1% | 3% | 0% | 2% | 10% |
| `IM_PLASMA` | 0% | 0% | 0% | 1% | 0% | 5% | 7% |
| unique | 7% | 9% | 17% | 4% | 19% | 25% | 57% |

`NO_SLOW` monsters (all): old forest tree, huorn, inertia hound, black-hearted huorn, hasty ent, aether
vortex, aether hound. `IM_PLASMA`: plasma vortex, plasma hound, the Phoenix, aether vortex/hound, greater
Balrog, Lungorthin, Gothmog.

Reading the table: elemental bolts get steadily worse with depth (fire and cold immunity pass 50% beyond
DL40), poison is the worst element at every depth, while `MISSILE`/`MANA`/`METEOR`/`HOLY_ORB`/`PLASMA`/
`GRAVITY`/`FORCE`/`DARK`/`SHARD` stay almost always at full damage.

### 19.3 Attack-spell selection algorithm

A practical procedure (ADVICE, built from the facts above):

```
for each known attack spell s castable now (SP >= cost, not blind/confused, not shapechanged
        or willing to revert):
    fail   = spell_chance(s)                     # section 5 (+25 if necro on lit square)
    if fail >= 50 and not desperate: skip         # coin flips only in emergencies
    targets = monsters s would hit:
        bolt: first monster on the path to the chosen target (not the target itself if blocked)
        beam/line/short beam/arc: every monster on/in the shape
        ball: targeted -> all within radius of target, damage /(d+1) at distance d
        LOS spells: every monster in view (one damage roll)
        auto-target spells (Tap Unlife, Vampire Strike): nearest valid monster only
    for each target t:
        mult(t) = resistance factor from 19.1 using known lore
                  (unknown lore: assume the flag prevalence for the depth from 19.2,
                   or prefer unresistable projections)
        dmg(t)  = min(avg_damage(s, L) * mult(t), t.hp)    # overkill is wasted
    value = (1 - fail/100) * sum_t weight(t) * dmg(t)
            # weight: danger of t (uniques, casters, breathers high; harmless low)
    efficiency = value / cost(s)
choose:
    - if one cast kills the most dangerous adjacent/visible threat with >= 80% probability: that spell
    - else if SP are plentiful relative to remaining threats: highest value
    - else: highest efficiency
    - never: area spells that would destroy wanted floor items; Zone of Unmagic without rDisen;
      Unleash Chaos when monsters asleep matter; Crush/Curse/Power Sacrifice/Shadow Shift when HP low
```

Class-specific rules:

- **Mage**: default Magic Missile (unresistable). Frost Bolt unless `IM_COLD`. Mana Bolt for tough or
  unknown-resist targets from L25. Acid Spray for 3+ targets in a cone. Teleport Other a monster whose
  expected damage to you per turn exceeds what you can kill in 2 turns.
- **Necromancer**: before any cast check `square_islit(player)`; if lit and the fight isn't urgent, step
  to a dark grid or cast Create Darkness. Nether Bolt only on non-undead (and preferably non-evil);
  undead → Tap Unlife (free) / Disenchant / Dark Spear. Annihilate only on living targets.
- **Priest**: Orb of Draining on evil targets (×2) — check `EVIL` in lore. Dispel spells when ≥3 matching
  monsters are in view and SP > 3× cost.
- **Druid**: Stinking Cloud unless `IM_POIS`; then Lightning Strike (two elements) or Earth Rising.
  Ice Storm only if most visible monsters lack `IM_COLD`. Slow Monster on any dangerous single
  melee monster (no save, stacks).
- **Blackguard**: spells are for when SP ≥ cost after a few rounds of melee; Berserk first, Maim Foe on
  dangerous melee monsters without `NO_STUN`, Forceful Blow on big targets.

### 19.4 Mana budgeting

ADVICE:

- Keep an **escape reserve**: never spend below the cost of your escape spell + 1 (Phase Door 2,
  Teleport Self 6, Portal 4, Shadow Shift 10) unless you hold the equivalent scroll — and since spells
  can fail, always hold the scroll too.
- Keep a **healing reserve** for Priests/Paladins: Healing needs 50 SP. A Priest with < 50 SP left in a
  dangerous fight should treat Healing as unavailable and quaff instead.
- Estimate SP needed to clear: `sum over threats of hp(t) / (efficiency of best spell vs t)`. If that
  exceeds current SP, plan to retreat and rest (resting regenerates ~0.6% of max SP per turn, ~170
  turns from empty; with *Regeneration* half that) or use mana potions only for real emergencies.
- **Don't rest while hasted**; don't count on regeneration mid-fight (0.3% per turn is nothing).
- Rest to full SP before opening doors to vaults, pits and nests.
- Blackguards: the budget is inverted — SP appear during combat. Cast buffs when the fight has gone on
  long enough to have SP; expect 0 SP when you arrive somewhere new.

### 19.5 Cost-effectiveness tables

FACT-based computation: average damage from the dice, fail from section 5 with an assumed casting stat
of 18/20 at L10, 18/50 at L20, 18/100 at L30, 18/150 at L40, 18/200 at L50, single target at the
centre, no resistances. Cells are **average damage / fail % / expected damage per SP**
(`damage × (1 − fail) / SP`). Area spells multiply their value by the number of targets hit; LOS spells
hit everything in view. Necromancer values assume a dark square (+25% fail on a lit square). Priest
Orb doubles against evil; Spear of Oromë too.

| Spell | Class | Lvl | SP | L10 | L20 | L30 | L40 | L50 | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Magic Missile | Mage | 1 | 1 | 10 / 5 / 9.5 | 15 / 4 / 14.4 | 20 / 3 / 19.4 | 25 / 1 / 24.8 | 30 / 0 / 30.0 | bolt/beam, unresistable |
| Electric Arc | Mage | 2 | 2 | 14 / 5 / 6.7 | 21 / 4 / 10.1 | 28 / 3 / 13.6 | 35 / 1 / 17.3 | 42 / 0 / 21.0 | short beam, length L |
| Fire Ball | Mage | 6 | 5 | 20 / 14 / 3.4 | 40 / 4 / 7.7 | 60 / 3 / 11.6 | 80 / 1 / 15.8 | 100 / 0 / 20.0 | ball r2 (centre) |
| Frost Bolt | Mage | 13 | 5 | - | 50 / 9 / 9.0 | 63 / 3 / 12.2 | 76 / 1 / 15.1 | 94 / 0 / 18.9 | bolt/beam cold |
| Acid Spray | Mage | 20 | 5 | - | 45 / 20 / 7.2 | 68 / 3 / 13.1 | 90 / 1 / 17.8 | 112 / 0 / 22.5 | 60° cone |
| Shock Wave | Mage | 20 | 5 | - | 40 / 30 / 5.6 | 60 / 3 / 11.6 | 80 / 1 / 15.8 | 100 / 0 / 20.0 | ball r2 sound (DL60 book) |
| Mana Bolt | Mage | 25 | 8 | - | - | 90 / 24 / 8.6 | 135 / 1 / 16.7 | 180 / 0 / 22.5 | bolt/beam, unresistable |
| Explosion | Mage | 30 | 10 | - | - | 66 / 29 / 4.7 | 88 / 1 / 8.7 | 110 / 0 / 11.0 | shards + force |
| Thrust Away | Mage | 40 | 12 | - | - | - | 180 / 50 / 7.5 | 225 / 9 / 17.1 | short beam force |
| Mana Storm | Mage | 45 | 16 | - | - | - | - | 400 / 19 / 20.2 | ball r3 mana |
| Stinking Cloud | Druid | 3 | 2 | 15 / 5 / 7.1 | 20 / 4 / 9.6 | 25 / 3 / 12.1 | 30 / 1 / 14.8 | 35 / 0 / 17.5 | ball r2 poison |
| Lightning Strike | Druid | 12 | 6 | - | 38 / 4 / 6.0 | 52 / 3 / 8.5 | 70 / 1 / 11.5 | 85 / 0 / 14.2 | elec + sound r3 |
| Earth Rising | Druid | 14 | 5 | - | 28 / 12 / 4.9 | 42 / 3 / 8.1 | 52 / 1 / 10.4 | 63 / 0 / 12.6 | short beam shards |
| Meteor Swarm | Druid | 30 | 14 | - | - | 135 / 50 / 4.8 | 200 / 19 / 11.6 | 220 / 0 / 15.7 | all meteors on one target |
| Rift | Druid | 35 | 20 | - | - | - | 200 / 9 / 9.1 | 240 / 0 / 12.0 | beam gravity |
| Ice Storm | Druid | 37 | 25 | - | - | - | 182 / 30 / 5.1 | 226 / 0 / 9.1 | every monster in view |
| Volcanic Eruption | Druid | 40 | 30 | - | - | - | 120 / 39 / 2.5 | 150 / 0 / 5.0 | r5 around you |
| River of Lightning | Druid | 43 | 35 | - | - | - | - | 270 / 3 / 7.5 | 20° cone plasma |
| Orb of Draining | Priest | 7 | 7 | 26 / 24 / 2.8 | 40 / 4 / 5.6 | 56 / 3 / 7.7 | 70 / 1 / 10.0 | 86 / 0 / 12.2 | ×2 vs evil |
| Dispel Undead | Priest | 10 | 14 | 26 / 48 / 0.9 | 50 / 15 / 3.1 | 76 / 3 / 5.2 | 100 / 1 / 7.1 | 126 / 0 / 9.0 | per undead in view |
| Dispel Evil | Priest | 12 | 20 | - | 50 / 36 / 1.6 | 76 / 3 / 3.7 | 100 / 1 / 5.0 | 126 / 0 / 6.3 | per evil in view |
| Holy Word | Priest | 39 | 32 | - | - | - | 80 / 50 / 1.3 | 100 / 11 / 2.8 | + heal 1000 |
| Spear of Oromë | Priest | 40 | 10 | - | - | - | 90 / 39 / 5.5 | 112 / 0 / 11.2 | beam, ×2 vs evil |
| Light of Varda | Priest | 42 | 40 | - | - | - | - | 250 / 10 / 5.6 | r4 (+100 light-sensitive) |
| Nether Bolt | Necro | 1 | 1 | 12 / 5 / 11.9 | 20 / 4 / 19.2 | 25 / 3 / 24.2 | 32 / 1 / 32.2 | 38 / 0 / 37.5 | 0 undead, ½ evil |
| Tap Unlife | Necro | 7 | 0 | 14 / 34 / free | 24 / 4 / free | 32 / 3 / free | 42 / 1 / free | 49 / 0 / free | undead only, gives SP |
| Disenchant | Necro | 16 | 12 | - | 51 / 28 / 3.1 | 71 / 3 / 5.7 | 91 / 1 / 7.5 | 111 / 0 / 9.2 | bolt |
| Vampire Strike | Necro | 20 | 5 | - | 40 / 40 / 4.8 | 60 / 3 / 11.6 | 80 / 1 / 15.8 | 100 / 0 / 20.0 | nearest living; heals |
| Dispel Life | Necro | 22 | 20 | - | - | 46 / 5 / 2.2 | 60 / 1 / 3.0 | 76 / 0 / 3.8 | per living in view |
| Dark Spear | Necro | 25 | 10 | - | - | 61 / 24 / 4.6 | 81 / 1 / 8.0 | 101 / 0 / 10.1 | beam dark |
| Annihilate | Necro | 30 | 20 | - | - | 120 / 50 / 3.0 | 160 / 9 / 7.3 | 200 / 0 / 10.0 | living only |
| Unleash Chaos | Necro | 38 | 15 | - | - | - | 164 / 48 / 5.7 | 204 / 3 / 13.2 | random shape; wakes |
| Storm of Darkness | Necro | 43 | 16 | - | - | - | - | 202 / 8 / 11.6 | ball r4 dark |

Takeaways:

- The 1-SP bolts (Magic Missile, Nether Bolt) are the most SP-efficient attacks in the game at every
  level; what the expensive spells buy is **damage per turn** (and area). When SP are scarce relative to
  the fight, spam the cheap bolt; when time (monster turns) is scarce, cast the biggest spell.
- Mana Bolt and Mana Storm at L50 are ~20+ damage per SP *and* huge per turn — the Mage's endgame.
- Priest single-target efficiency is low except vs evil (Orb ×2 → 24 per SP at L50); the Priest's
  strength is healing and line-of-sight anti-evil, not raw damage.
- Damage per turn ranking at L40 (single target, no resist): Thrust Away 180 (short range) ≈ Rift 200
  ≈ Meteor Swarm 200 > Annihilate 160 = Unleash Chaos ~164 > Mana Bolt 135 > Spear of Oromë 90
  (180 vs evil) > Frost Bolt 76. Multiply by `(1 − fail)`.
- Number of casts per full pool (assumed stats above): Mage L30 has 121 SP = 24 Frost Bolts or
  15 Mana Bolts; Mage L50 401 SP = 25 Mana Storms.

---

## Things to try (untested)

1. **Slow-stacking as a primary Druid defence.** FACT: Slow Monster has no saving throw and only 7
   monsters have `NO_SLOW`; slow stacks to −10 speed. TRY: against a dangerous single melee unique at
   DL20–40, cast Slow 3–4 times before attacking. TEST: log monster energy/turns per player turn
   before/after in a series of fights; compare HP lost per kill with and without the slows.
2. **Confuse/Disenchant to shut down casters.** FACT: confused or disenchanted monsters fail +50% of
   spells. TRY: open every fight against a summoner/caster with Confuse Monster (druid) or Disenchant
   (necromancer). TEST: count monster spell attempts vs successes (message log) with and without.
3. **Necromancer "darkness discipline".** TRY: a borg rule "never cast on a lit square unless HP <
   30%", stepping into corridors or casting Create Darkness first. TEST: compare effective fail rates
   and deaths over many games.
4. **Teleport Self vs scroll at low level.** FACT: the spell covers 9L/5 % of the edge distance vs 60% for
   the scroll. TRY: measure actual landing distances at L10/20/30 and decide at which level a borg
   should prefer the spell over the scroll for "flee" (not "reposition"). TEST: 100 casts per level in
   a wizard-mode empty level.
5. **Priest study ordering.** TRY: always study from the book with the fewest eligible prayers to steer
   the random choice toward the prayer you want (e.g. get Orb before Spear of Light by waiting for L8 and
   studying twice). TEST: simulate learning order; compare time-to-Orb.
6. **Crush as a finisher combo.** TRY: wound a group with an area spell to below 4L HP each, then Crush
   kills them all for 6 SP. TEST: SP per kill on orc packs at L15–25 vs pure Nether Bolt.
7. **Trance + walk away.** FACT: held monsters skip turns until damaged. TRY: Druid at L20+ surrounded
   by melee monsters: Trance, then walk/phase away rather than fight. TEST: escape success rate vs
   Phase Door alone.
8. **Glyph corridor fighting for Priests.** TRY: cast Glyph of Warding in a corridor and fight with Orb
   from it; monsters must break it (chance (level−1)/550) to melee you. TEST: damage taken per kill vs
   open fighting against DL30–50 melee packs.
9. **Recharging as a mana battery.** TRY: Mage keeps 2–3 Wands of Magic Missile/Stinking Cloud, recharges
   only when empty, and uses them when SP < 25%. TEST: total damage per trip and backfire losses.
10. **Forceful Blow first.** FACT: the 200 force damage only needs one damaging blow. TRY: Blackguard
    opens with Forceful Blow on big targets after building SP. TEST: kills per SP vs Maim Foe.
11. **Cover Tracks sniping.** TRY: Ranger casts Cover Tracks, then shoots from > 5 squares: monsters
    can't "see" the player beyond max_sight/4. TEST: count monster ranged attacks received per shot fired
    with vs without.
12. **Power Sacrifice + Healing potion loop for burst damage.** TRY: necromancer converts HP to SP
    before a unique fight (50 HP → 50 SP repeatedly) when it carries surplus CCW/Healing potions. TEST:
    damage dealt per potion used vs straight casting.
13. **Lightning Strike on yourself as a point-blank AoE.** FACT: if the target is not a visible passable
    LOS grid, the strike centres on the player, and the projection has no player flag. TRY: verify that
    casting it without a target hurts adjacent monsters (radius-3 sound) but not you. TEST: wizard-mode
    with monsters around the player; check HP.
14. **Light Room as orc control.** TRY: Light Room/Call Light on orc and jelly-heavy early levels
    (`HURT_LIGHT` 16% of DL1–10) — check whether the 1 / 2d(L/2) damage plus light makes orc packs
    flee or is negligible. TEST: message log of "cringes from the light".

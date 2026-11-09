# Chapter 2 — Races and Classes

> Angband 4.2.6 (C). Every number in this chapter is taken from the game data in
> `lib/gamedata/` (`p_race.txt`, `class.txt`, `shape.txt`, `player_property.txt`,
> `constants.txt`, `object.txt`) or from the C sources in `src/` (cited by file and
> function). Where the game's own help text (`docs/birth.rst`, `docs/option.rst`)
> and the code disagree, the code wins and the disagreement is pointed out.
> `lib/gamedata/old_class.txt` is a leftover file that the game never loads and is
> ignored here.
>
> Spells, prayers and shapechanges are covered in detail in
> [Chapter 3 — Spells and Prayers](03-magic.md). This chapter covers what your race and class
> *are*, how the numbers combine, and how to play each class.

---

## Contents

1. [How race and class combine](#1-how-race-and-class-combine)
2. [The eleven races](#2-the-eleven-races)
3. [Race × class combination tables](#3-race--class-combination-tables)
4. [The nine classes — overview and mechanics](#4-the-nine-classes--overview-and-mechanics)
5. [Class guides](#5-class-guides)
   - [Warrior](#51-warrior) · [Mage](#52-mage) · [Druid](#53-druid) ·
     [Priest](#54-priest) · [Necromancer](#55-necromancer) ·
     [Paladin](#56-paladin) · [Rogue](#57-rogue) · [Ranger](#58-ranger) ·
     [Blackguard](#59-blackguard)
6. [Character birth: stats, rollers and birth options](#6-character-birth-stats-rollers-and-birth-options)
7. [Recommended combinations](#7-recommended-combinations)
8. [Quick reference card](#8-quick-reference-card)

---

## 1. How race and class combine

### 1.1 Stats

Each character has five stats: STR, INT, WIS, DEX, CON. At birth you get a
*base* value (10–18 by point-buy, or 3d-ish by the roller — see §6). Race and
class each add a modifier (`stats:` lines in `p_race.txt` and `class.txt`), and the
sum is applied with `modify_stat_value()` (`src/player-util.c`):

* below 18, each +1 adds one point (15 → 16 → 17 → 18);
* at 18 or above, each +1 adds **ten** percentile points (18 → 18/10 → 18/20 …);
* penalties work in reverse (18/20 → 18/10 → 18 → 17 …).

So a base 18 INT with a +3 race/class total becomes 18/30, while a base 17 with
+3 becomes 18/20 (17 → 18, then two steps of ten). **Buying the last point to 18
before modifiers is therefore worth an extra ten percentile points of the final
stat** — this is why the point-buy cost of 17→18 is high (see §6.1).

Stats are converted to an *index* (`calc_bonuses()` in `src/player-calcs.c`):
3–18 map to indexes 0–15, 18/00–18/09 is index 15, every further ten percentile
points is +1 index, capped at 18/220 (index 37). All stat tables in the game use
this index.

A few of those tables that matter a lot when choosing race and class:

| Stat value | Blows STR factor `adj_str_blow` | Blows DEX index `adj_dex_blow` | Extra HP per level (1/100) `adj_con_mhp` | Mana per level (1/100) `adj_mag_mana` | Spells learned per level (1/100) `adj_mag_study` | Minimum spell fail `adj_mag_fail` |
|---|---|---|---|---|---|---|
| 8 | 8 | 0 | −25 | 50 | 40 | 50% |
| 10 | 10 | 1 | −5 | 70 | 60 | 20% |
| 12 | 12 | 1 | 5 | 90 | 80 | 12% |
| 14 | 14 | 1 | 25 | 110 | 90 | 10% |
| 16 | 16 | 1 | 75 | 130 | 100 | 8% |
| 17 | 17 | 2 | 100 | 140 | 105 | 7% |
| 18/00 | 20 | 2 | 150 | 150 | 110 | 6% |
| 18/20 | 40 | 3 | 200 | 170 | 120 | 5% |
| 18/50 | 70 | 4 | 275 | 200 | 150 | 4% |
| 18/70 | 90 | 5 | 350 | 250 | 170 | 4% |
| 18/100 | 120 | 7 | 500 | 400 | 200 | 3% |
| 18/120 | 140 | 8 | 600 | 500 | 220 | 2% |
| 18/150 | 170 | 9 | 750 | 650 | 250 | 1% |
| 18/180 | 200 | 10 | 1000 | 800 | 250 | 1% |
| 18/200 | 220 | 11 | 1250 | 800 | 250 | 0% |
| 18/220 | 240 | 11 | 1250 | 800 | 250 | 0% |

(Tables from `src/player-calcs.c` and `src/player-spell.c`. The 0% minimum fail is
only reachable by classes with the `ZERO_FAIL` flag; everyone else is floored at
5% — see Chapter 3.)

### 1.2 Skills

Every skill is computed in `calc_bonuses()` as:

```
skill = race_skill + class_base_skill            (birth value)
      + class_increment * character_level / 10   (integer maths)
      + stat adjustments                          (DEX→disarm-phys, INT→disarm-magic
                                                   and device, WIS→save, STR→dig)
      + equipment, shape and temporary effects
```

Race skills (`p_race.txt`) are flat; class skills are `base:increment-per-10-levels`
(`class.txt`). Stealth is clamped to 0–30 and digging to a minimum of 1.
Melee/shooting/throwing skill turn into to-hit chance via
`chance_of_melee_hit_base()` / `chance_of_missile_hit_base()` in
`src/player-attack.c`: *skill + (to-hit bonus × 3)* (`BTH_PLUS_ADJ` = 3 in
`src/player.h`). Non-throwing objects that are thrown use 3/2 of the throwing skill.

### 1.3 Hit points

`player_generate()` (`src/player-birth.c`) sets your **hit die** to
`race hitdie + class hitdie`. At level 1 you get the full hit die; every later level
adds `1d(hit die)` — but `roll_hp()` re-rolls the whole 49-level sequence until the
level-50 total falls between `50 + 50·(HD−1)·3/8` and `50 + 50·(HD−1)·5/8`, so your
final HP is never far from average. The rolls are made when you *accept* the
character (`do_cmd_accept_character()`), so the birth screen HP is only a preview.

`calc_hitpoints()` (`src/player-calcs.c`) then adds `adj_con_mhp[CON] × level / 100`.
That bonus is recalculated from level 1, so raising CON from 18/50 to 18/100 at
character level 40 instantly adds (500 − 275) × 40 / 100 = 90 HP.

### 1.4 Experience factor

Your experience penalty is `race exp + class exp` (`player_generate()`); no class sets
an `exp:` line, so in practice it is just the race value (100% Human, 145% High-Elf,
120% everyone else). The experience needed to reach level *L* is
`player_exp[L−2] × expfact / 100` (`player_exp[]` in `src/player.c`):

| Level | Human (100%) | Most races (120%) | High-Elf (145%) |
|---|---|---|---|
| 5 | 70 | 84 | 101 |
| 10 | 380 | 456 | 551 |
| 15 | 1,400 | 1,680 | 2,030 |
| 20 | 4,400 | 5,280 | 6,380 |
| 25 | 12,500 | 15,000 | 18,125 |
| 30 | 50,000 | 60,000 | 72,500 |
| 35 | 150,000 | 180,000 | 217,500 |
| 40 | 450,000 | 540,000 | 652,500 |
| 45 | 1,250,000 | 1,500,000 | 1,812,500 |
| 50 | 4,500,000 | 5,400,000 | 6,525,000 |

The penalty matters most in the early game. By the midgame, experience per kill
grows so quickly (monster level × monster exp / your level) that a 20–25% slower
curve is rarely noticeable; High-Elves reach the same depth as everyone else, just a
few levels "behind" for a while.

### 1.5 Infravision

`infravision:` (in squares) lets you see warm-blooded monsters within that distance
even when they are unlit (`src/mon-util.c`, `see_infra`); the character sheet
displays it ×10 ft (`src/ui-player.c`). It never increases with level. Humans and
Dúnedain have none. It does **not** show cold-blooded or invisible monsters.

### 1.6 Intrinsic flags

Races and classes can carry *object flags* (`obj-flags:` — the same properties an
item could give: sustains, Free Action, See Invisible, Hold Life, Regeneration…),
*element values* (`values:` — `RES_LIGHT[1]` means resist light) and *player flags*
(`player-flags:` — special abilities defined in `player_property.txt`). These are
permanent and cannot be disenchanted or lost.

---

## 2. The eleven races

### 2.1 Comparison tables

**Stats, hit die, experience, infravision** (`p_race.txt`)

| Race | STR | INT | WIS | DEX | CON | Sum | Hit die | Exp % | Infra |
|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 10 | 100 | 0 |
| Half-Elf | 0 | +1 | −1 | +1 | −1 | 0 | 10 | 120 | 2 |
| Elf | −1 | +2 | −1 | +1 | −1 | 0 | 9 | 120 | 3 |
| Hobbit | −2 | +2 | +1 | +3 | +2 | +6 | 7 | 120 | 4 |
| Gnome | −1 | +2 | 0 | +2 | +1 | +4 | 8 | 120 | 4 |
| Dwarf | +2 | −3 | +2 | −2 | +2 | +1 | 11 | 120 | 5 |
| Half-Orc | +2 | −1 | 0 | 0 | +1 | +2 | 10 | 120 | 3 |
| Half-Troll | +4 | −4 | −2 | −4 | +3 | −3 | 12 | 120 | 3 |
| Dúnadan | +1 | +2 | +2 | +2 | +3 | +10 | 10 | 120 | 0 |
| High-Elf | +1 | +3 | −1 | +3 | +1 | +7 | 10 | 145 | 4 |
| Kobold | −1 | −1 | 0 | +2 | +2 | +2 | 8 | 120 | 5 |

**Skills** (`p_race.txt`; these are added to the class values)

| Race | Disarm (phys) | Disarm (magic) | Device | Save | Stealth | Search | Melee | Shoot | Throw | Dig |
|---|---|---|---|---|---|---|---|---|---|---|
| Human | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Half-Elf | 2 | 2 | 3 | 3 | 1 | 3 | −1 | 5 | 5 | 0 |
| Elf | 5 | 5 | 6 | 6 | 2 | 6 | −5 | 15 | 15 | 0 |
| Hobbit | 15 | 15 | 18 | 18 | 4 | 6 | −10 | 20 | 20 | 0 |
| Gnome | 10 | 10 | 22 | 12 | 3 | 4 | −8 | 12 | 12 | 0 |
| Dwarf | 2 | 2 | 9 | 9 | −1 | 2 | 15 | 0 | 0 | 40 |
| Half-Orc | −3 | −3 | −3 | −3 | −1 | −3 | 12 | −5 | −5 | 0 |
| Half-Troll | −5 | −5 | −8 | −8 | −2 | −9 | 20 | −10 | −10 | 0 |
| Dúnadan | 4 | 4 | 5 | 5 | 1 | 3 | 15 | 10 | 10 | 0 |
| High-Elf | 4 | 4 | 20 | 20 | 2 | 10 | 10 | 25 | 25 | 0 |
| Kobold | 10 | 10 | 5 | 0 | 3 | 10 | −5 | 10 | 10 | 0 |

**Intrinsic abilities** (`obj-flags`, `values`, `player-flags` in `p_race.txt`;
meanings from `player_property.txt`)

| Race | Sustain | Protection / ability | Resist | Special |
|---|---|---|---|---|
| Human | — | — | — | — (fastest levelling) |
| Half-Elf | DEX | — | — | — |
| Elf | DEX | — | Light | — |
| Hobbit | — | Hold Life | — | Mushroom Lore (`KNOW_MUSHROOM`: recognises mushrooms) |
| Gnome | — | **Free Action** | — | Device Lore (`KNOW_ZAPPER`: recognises wands, staves, rods) |
| Dwarf | — | Blindness immunity (`PROT_BLIND`) | — | Miner (`SEE_ORE`: senses ore/treasure in walls); digging +40 |
| Half-Orc | — | — | Dark | — |
| Half-Troll | STR | Regeneration | — | — |
| Dúnadan | CON | — | — | — |
| High-Elf | — | **See Invisible** | Light | — |
| Kobold | — | — | **Poison** | — |

The object-knowledge flags work through `obj-gear.c` (`player_has(p, PF_KNOW_MUSHROOM)`
/ `PF_KNOW_ZAPPER` make those item kinds known on pickup); `SEE_ORE` is applied in
`process_world()` (`src/game-world.c`) to reveal mineral veins with treasure.

### 2.2 Race-by-race notes

#### Human
*Stats 0/0/0/0/0 · HD 10 · Exp 100% · Infra 0 · no intrinsics.*

The baseline. Humans have nothing except the cheapest experience curve in the game:
a Human reaches character level 30 with 50,000 exp, where everyone else needs 60,000
and a High-Elf 72,500. In the first hour of play that means a level or two ahead of
the other races, which converts directly into hit points and spells. After that,
the lack of any intrinsic (no Free Action, no See Invisible, no infravision) is a
real, permanent cost that you must cover with equipment.

*Best with:* any class, but there is almost always a strictly better race for each
role. Human Warrior or Human Blackguard is a reasonable "learn the game" choice
because the fast levels smooth the early game.

#### Half-Elf
*Stats 0/+1/−1/+1/−1 · HD 10 · Exp 120% · Infra 2 · Sustain DEX.*

Small, even bonuses to device, save, shooting (+5) and stealth (+1) for a −1 melee.
Sustain DEX is minor. Half-Elves are a "mild generalist" — a Half-Elf Rogue or Ranger
is fine, but an Elf, Hobbit or High-Elf usually does each job better.

#### Elf
*Stats −1/+2/−1/+1/−1 · HD 9 · Exp 120% · Infra 3 · Sustain DEX, resist Light.*

+15 shooting and throwing, +6 device and save, +2 stealth, at the cost of −5 melee
and a lower hit die. **Resist Light** is uncommon early and blocks the blinding
side-effect of light attacks. Elves make good Mages, Rangers and Rogues; INT +2 helps
arcane casters (Mage INT total +5).

#### Hobbit
*Stats −2/+2/+1/+3/+2 · HD 7 · Exp 120% · Infra 4 · Hold Life, Mushroom Lore.*

The best stealth (+4), excellent device (+18), save (+18), disarm (+15/+15) and
missile skills (+20) of any race, plus **Hold Life**, which protects experience from
draining. The price is the lowest hit die in the game (7: a Hobbit Mage has a
total hit die of 7, i.e. 162–237 base HP at level 50 before CON) and −10 melee.
DEX +3 makes Hobbits strong slingers and throwers.

*Best with:* Rogue (DEX +6 total, stealth 7 at birth before level growth), Ranger,
Mage (INT +5 total). Avoid on melee classes: STR −2 hurts blows a lot.

#### Gnome
*Stats −1/+2/0/+2/+1 · HD 8 · Exp 120% · Infra 4 · Free Action, Device Lore.*

**Intrinsic Free Action** from turn one is the reason to pick a Gnome: paralysis
(floating eyes at dungeon level 1, carrion crawlers and ghouls in the 20s, basilisks)
is the classic killer of characters who have not found a Free Action item, and the
Gnome simply ignores it. +22 device skill is the best in the game, and wands, staves
and rods are identified on pickup.

*Best with:* Mage (INT +5, devices), Necromancer, Rogue, Ranger. A Gnome Mage is one of
the safest pure casters to learn the game with.

#### Dwarf
*Stats +2/−3/+2/−2/+2 · HD 11 · Exp 120% · Infra 5 · Blindness immunity, Miner, Dig +40.*

Strong, tough, wise and blind-proof. **Blindness immunity** (`PROT_BLIND`) is more
valuable than it looks: the game refuses to let a blind character read scrolls or
cast spells (`player_can_cast()` / `player_can_read()` in `src/player-util.c`), so a
Dwarf can always read Phase Door, Teleport, Word of Recall and cast spells while
blinding monsters are around. +15 melee and +40 digging (tunnel through rubble and
magma quickly with any weapon) are bonuses; INT −3 rules out arcane magic.

*Best with:* Priest (WIS +5, CON +3 total, blind-proof prayers), Druid (WIS +5),
Paladin (STR +3, WIS +3, CON +4), Warrior, Blackguard (STR +4, CON +4; INT −3 is
painful for Blackguard mana though).

#### Half-Orc
*Stats +2/−1/0/0/+1 · HD 10 · Exp 120% · Infra 3 · resist Dark.*

A cheaper, less extreme fighter race: +12 melee, small skill penalties everywhere
else, and **resist Dark** (which also prevents darkness-based blinding). Solid for
Warrior, Blackguard and Paladin; usually out-classed by Dúnadan or Half-Troll.

#### Half-Troll
*Stats +4/−4/−2/−4/+3 · HD 12 · Exp 120% · Infra 3 · Sustain STR, Regeneration.*

The biggest hit die (12; a Half-Troll Warrior has 21, giving 425–675 base HP at level
50 before CON), STR +4, CON +3, +20 melee and permanent **Regeneration** (doubles HP
and mana regeneration in `player_regen_hp()` / `player_regen_mana()`,
`src/player-util.c`). Every non-combat skill is bad (device −8, save −8, search −9,
stealth −2) and DEX −4 costs blows with light weapons — Half-Trolls want heavy
weapons. Regeneration also makes you digest food faster.

*Best with:* Warrior, Blackguard (STR +6, CON +5 total). Viable Paladin. Avoid for
casters other than as a challenge.

#### Dúnadan
*Stats +1/+2/+2/+2/+3 · HD 10 · Exp 120% · Infra 0 · Sustain CON.*

The best all-round stat line (+10 total, nothing negative) with +15 melee and +10
shooting. No infravision and no resistances, but Sustain CON protects your hit
points. A Dúnadan is good at *every* class; it is the default answer when nothing else
stands out.

*Best with:* Paladin, Ranger, Blackguard, Warrior, Priest (WIS +5 total).

#### High-Elf
*Stats +1/+3/−1/+3/+1 · HD 10 · Exp 145% · Infra 4 · See Invisible, resist Light.*

Big INT and DEX, good STR/CON, +20 device, +20 save, +25 shooting, +10 melee and
search, and **permanent See Invisible** — ghosts, poltergeists, moaning spirits and
other invisible monsters are visible from the first step. The 145% experience
penalty is the only cost.

*Best with:* Mage (INT +6 total), Necromancer, Ranger (shooting 72+25 at birth), Rogue.
Very strong Warrior as well (STR +4, DEX +5 totals). Many experienced players
consider a High-Elf the strongest race overall.

#### Kobold
*Stats −1/−1/0/+2/+2 · HD 8 · Exp 120% · Infra 5 · resist Poison.*

Intrinsic **resist Poison** is the attraction: poison is the most common damaging
breath and bite in the early and middle game, and without resistance poison-breathing
monsters (e.g. the basilisk's `BR_POIS`) deal full damage. Kobolds also have +3
stealth and +10 search/disarm. Stats are mediocre (INT −1 hurts arcane casters).

*Best with:* Rogue, Ranger, Priest/Druid (WIS 0 is neutral), Warrior as a stealthy
alternative.

### 2.3 Intrinsics ranked by usefulness

Rough strategic ranking (opinion, based on how often the missing property kills
characters in the early/mid game):

1. **Free Action** (Gnome) — paralysis is a frequent cause of death before you find
   gear with it. Note that Free Action stops paralysis only (`FREE_ACT` = "You are
   immune to paralysis" in `player_property.txt`); it does not stop slowing.
2. **See Invisible** (High-Elf) — invisible monsters can't be targeted or avoided
   otherwise; priests/necromancers have a spell for it, others need Detect
   Invisible scrolls/staves or gear.
3. **Resist Poison** (Kobold) — normally a mid-to-late-game find.
4. **Hold Life** (Hobbit) — experience drain is annoying more than deadly.
5. **Blindness immunity** (Dwarf) — keeps scrolls and spells working.
6. **Regeneration** (Half-Troll) — saves resting time, makes long fights easier.
7. **Resist Light / Dark** (Elf, High-Elf / Half-Orc) — mostly prevents blinding.
8. Sustains (Half-Elf, Elf DEX; Half-Troll STR; Dúnadan CON).

---

## 3. Race × class combination tables

### 3.1 Total stat modifiers (race + class), as STR/INT/WIS/DEX/CON

Class modifiers come from `class.txt` (`stats:` lines). Apply the total to your base
stat with the 18 → 18/10 rule from §1.1.

| Race | Warrior | Mage | Druid | Priest | Necromancer | Paladin | Rogue | Ranger | Blackguard |
|---|---|---|---|---|---|---|---|---|---|
| Human | +3/−2/−2/+2/+2 | −3/+3/+0/+0/−2 | −2/+0/+3/−2/+0 | −1/−3/+3/−1/+1 | −3/+3/+0/+0/−2 | +1/−3/+1/−1/+2 | +0/+1/−3/+3/−1 | +0/+0/+2/+1/−1 | +2/+0/−3/+0/+2 |
| Half-Elf | +3/−1/−3/+3/+1 | −3/+4/−1/+1/−3 | −2/+1/+2/−1/−1 | −1/−2/+2/+0/+0 | −3/+4/−1/+1/−3 | +1/−2/+0/+0/+1 | +0/+2/−4/+4/−2 | +0/+1/+1/+2/−2 | +2/+1/−4/+1/+1 |
| Elf | +2/+0/−3/+3/+1 | −4/+5/−1/+1/−3 | −3/+2/+2/−1/−1 | −2/−1/+2/+0/+0 | −4/+5/−1/+1/−3 | +0/−1/+0/+0/+1 | −1/+3/−4/+4/−2 | −1/+2/+1/+2/−2 | +1/+2/−4/+1/+1 |
| Hobbit | +1/+0/−1/+5/+4 | −5/+5/+1/+3/+0 | −4/+2/+4/+1/+2 | −3/−1/+4/+2/+3 | −5/+5/+1/+3/+0 | −1/−1/+2/+2/+4 | −2/+3/−2/+6/+1 | −2/+2/+3/+4/+1 | +0/+2/−2/+3/+4 |
| Gnome | +2/+0/−2/+4/+3 | −4/+5/+0/+2/−1 | −3/+2/+3/+0/+1 | −2/−1/+3/+1/+2 | −4/+5/+0/+2/−1 | +0/−1/+1/+1/+3 | −1/+3/−3/+5/+0 | −1/+2/+2/+3/+0 | +1/+2/−3/+2/+3 |
| Dwarf | +5/−5/+0/+0/+4 | −1/+0/+2/−2/+0 | +0/−3/+5/−4/+2 | +1/−6/+5/−3/+3 | −1/+0/+2/−2/+0 | +3/−6/+3/−3/+4 | +2/−2/−1/+1/+1 | +2/−3/+4/−1/+1 | +4/−3/−1/−2/+4 |
| Half-Orc | +5/−3/−2/+2/+3 | −1/+2/+0/+0/−1 | +0/−1/+3/−2/+1 | +1/−4/+3/−1/+2 | −1/+2/+0/+0/−1 | +3/−4/+1/−1/+3 | +2/+0/−3/+3/+0 | +2/−1/+2/+1/+0 | +4/−1/−3/+0/+3 |
| Half-Troll | +7/−6/−4/−2/+5 | +1/−1/−2/−4/+1 | +2/−4/+1/−6/+3 | +3/−7/+1/−5/+4 | +1/−1/−2/−4/+1 | +5/−7/−1/−5/+5 | +4/−3/−5/−1/+2 | +4/−4/+0/−3/+2 | +6/−4/−5/−4/+5 |
| Dúnadan | +4/+0/+0/+4/+5 | −2/+5/+2/+2/+1 | −1/+2/+5/+0/+3 | +0/−1/+5/+1/+4 | −2/+5/+2/+2/+1 | +2/−1/+3/+1/+5 | +1/+3/−1/+5/+2 | +1/+2/+4/+3/+2 | +3/+2/−1/+2/+5 |
| High-Elf | +4/+1/−3/+5/+3 | −2/+6/−1/+3/−1 | −1/+3/+2/+1/+1 | +0/+0/+2/+2/+2 | −2/+6/−1/+3/−1 | +2/+0/+0/+2/+3 | +1/+4/−4/+6/+0 | +1/+3/+1/+4/+0 | +3/+3/−4/+3/+3 |
| Kobold | +2/−3/−2/+4/+4 | −4/+2/+0/+2/+0 | −3/−1/+3/+0/+2 | −2/−4/+3/+1/+3 | −4/+2/+0/+2/+0 | +0/−4/+1/+1/+4 | −1/+0/−3/+5/+1 | −1/−1/+2/+3/+1 | +1/−1/−3/+2/+4 |

Highlights: the best spell-stat totals are **High-Elf Mage/Necromancer (INT +6)**,
**Dwarf/Dúnadan Priest and Druid (WIS +5)**; the best melee lines are
**Half-Troll Warrior (STR +7, CON +5)** and **Dúnadan/High-Elf Warrior (DEX +4/+5)**.

### 3.2 Total hit die (race + class)

| Race | Warrior | Mage | Druid | Priest | Necro | Paladin | Rogue | Ranger | Blackguard |
|---|---|---|---|---|---|---|---|---|---|
| Human | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Half-Elf | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Elf | 18 | 9 | 11 | 11 | 11 | 15 | 13 | 14 | 17 |
| Hobbit | 16 | 7 | 9 | 9 | 9 | 13 | 11 | 12 | 15 |
| Gnome | 17 | 8 | 10 | 10 | 10 | 14 | 12 | 13 | 16 |
| Dwarf | 20 | 11 | 13 | 13 | 13 | 17 | 15 | 16 | 19 |
| Half-Orc | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Half-Troll | 21 | 12 | 14 | 14 | 14 | 18 | 16 | 17 | 20 |
| Dúnadan | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| High-Elf | 19 | 10 | 12 | 12 | 12 | 16 | 14 | 15 | 18 |
| Kobold | 17 | 8 | 10 | 10 | 10 | 14 | 12 | 13 | 16 |

### 3.3 Level-50 base HP range (before the CON bonus)

From `roll_hp()`: minimum `50 + 50·(HD−1)·3/8`, maximum `50 + 50·(HD−1)·5/8`.
Add `adj_con_mhp × 50 / 100` for CON (e.g. +250 at 18/100, +375 at 18/150, +625 at
18/200+).

| Race | Warrior | Mage | Druid/Priest/Necro | Paladin | Rogue | Ranger | Blackguard |
|---|---|---|---|---|---|---|---|
| Human / Half-Elf / Half-Orc / Dúnadan / High-Elf | 387–612 | 218–331 | 256–393 | 331–518 | 293–456 | 312–487 | 368–581 |
| Elf | 368–581 | 200–300 | 237–362 | 312–487 | 275–425 | 293–456 | 350–550 |
| Hobbit | 331–518 | 162–237 | 200–300 | 275–425 | 237–362 | 256–393 | 312–487 |
| Gnome / Kobold | 350–550 | 181–268 | 218–331 | 293–456 | 256–393 | 275–425 | 331–518 |
| Dwarf | 406–643 | 237–362 | 275–425 | 350–550 | 312–487 | 331–518 | 387–612 |
| Half-Troll | 425–675 | 256–393 | 293–456 | 368–581 | 331–518 | 350–550 | 406–643 |

A Hobbit Mage with 18/100 CON tops out around 162–237 + 250 ≈ 410–490 HP. A Half-Troll
Warrior with 18/200 CON: 425–675 + 625 ≈ 1050–1300 HP. That gap is why the casters
must avoid damage rather than absorb it.

---

## 4. The nine classes — overview and mechanics

### 4.1 Class comparison

From `class.txt`. "Magic" is `magic:first-spell-level:armour-weight-limit:books`
(the weight limit is in tenths of a pound). The realm determines the casting stat
(`realm.txt`: arcane and shadow use INT; divine and nature use WIS).

| Class | STR | INT | WIS | DEX | CON | HD | Max blows | Min weapon wt | STR mult | Realm (stat) | 1st spell lvl | Armour limit | Books | Flags |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Warrior | +3 | −2 | −2 | +2 | +2 | 9 | 6 | 3.0 lb | 5 | — | — | — | — | BRAVERY_30, NO_MANA, SHIELD_BASH |
| Mage | −3 | +3 | 0 | 0 | −2 | 0 | 4 | 4.0 lb | 2 | arcane (INT) | 1 | 30 lb | 5 | ZERO_FAIL, BEAM, CHOOSE_SPELLS |
| Druid | −2 | 0 | +3 | −2 | 0 | 2 | 4 | 3.5 lb | 3 | nature (WIS) | 1 | 35 lb | 5 | ZERO_FAIL, CHOOSE_SPELLS, CHARM |
| Priest | −1 | −3 | +3 | −1 | +1 | 2 | 4 | 3.5 lb | 3 | divine (WIS) | 1 | 35 lb | 5 | BLESS_WEAPON, ZERO_FAIL |
| Necromancer | −3 | +3 | 0 | 0 | −2 | 2 | 4 | 3.5 lb | 3 | shadow (INT) | 1 | 30 lb | 5 | ZERO_FAIL, CHOOSE_SPELLS, UNLIGHT, EVIL |
| Paladin | +1 | −3 | +1 | −1 | +2 | 6 | 5 | 3.0 lb | 5 | divine (WIS) | 1 | 40 lb | 3 | BLESS_WEAPON, SHIELD_BASH |
| Rogue | 0 | +1 | −3 | +3 | −1 | 4 | 5 | 2.0 lb | 4 | arcane (INT) | 5 | 35 lb | 2 | CHOOSE_SPELLS, STEAL |
| Ranger | 0 | 0 | +2 | +1 | −1 | 5 | 5 | 3.5 lb | 4 | nature (WIS) | 3 | 40 lb | 2 | FAST_SHOT, CHOOSE_SPELLS |
| Blackguard | +2 | 0 | −3 | 0 | +2 | 8 | 5 | 10.0 lb | 5 | shadow (INT) | 1 | 60 lb | 3 | CHOOSE_SPELLS, SHIELD_BASH, COMBAT_REGEN; *IMPAIR_HP* |

No class has an experience penalty. All classes except the Warrior can cast; the
Blackguard additionally carries the object flag `IMPAIR_HP` ("Slow Regeneration":
HP regeneration halved in `player_regen_hp()`).

### 4.2 Class skills (base, +increment per 10 levels → value at level 50 for a Human)

| Class | Disarm phys | Disarm magic | Device | Save | Stealth | Search | Melee | Shoot | Throw |
|---|---|---|---|---|---|---|---|---|---|
| Warrior | 25 (+15) → 100 | 20 (+10) → 70 | 18 (+7) → 53 | 18 (+10) → 68 | 0 (+0) → 0 | 10 (+12) → 70 | 70 (+45) → 295 | 55 (+45) → 280 | 55 (+45) → 280 |
| Mage | 30 (+10) → 80 | 35 (+12) → 95 | 36 (+13) → 101 | 30 (+9) → 75 | 2 (+0) → 2 | 10 (+12) → 70 | 35 (+15) → 110 | 20 (+15) → 95 | 20 (+15) → 95 |
| Druid | 30 (+10) → 80 | 30 (+10) → 80 | 24 (+10) → 74 | 30 (+10) → 80 | 3 (+0) → 3 | 12 (+12) → 72 | 45 (+20) → 145 | 40 (+30) → 190 | 40 (+30) → 190 |
| Priest | 25 (+12) → 85 | 25 (+12) → 85 | 30 (+10) → 80 | 32 (+12) → 92 | 2 (+0) → 2 | 10 (+14) → 80 | 45 (+20) → 145 | 35 (+20) → 135 | 35 (+20) → 135 |
| Necromancer | 30 (+10) → 80 | 35 (+12) → 95 | 36 (+13) → 101 | 30 (+9) → 75 | 2 (+0) → 2 | 10 (+12) → 70 | 35 (+25) → 160 | 20 (+15) → 95 | 20 (+15) → 95 |
| Paladin | 20 (+15) → 95 | 20 (+10) → 70 | 24 (+10) → 74 | 25 (+11) → 80 | 0 (+0) → 0 | 10 (+12) → 70 | 65 (+40) → 265 | 50 (+30) → 200 | 50 (+30) → 200 |
| Rogue | 45 (+20) → 145 | 45 (+20) → 145 | 32 (+10) → 82 | 28 (+10) → 78 | 3 (+1) → 8 | 20 (+16) → 100 | 35 (+45) → 260 | 66 (+30) → 216 | 72 (+45) → 297 |
| Ranger | 40 (+15) → 115 | 30 (+10) → 80 | 28 (+10) → 78 | 32 (+10) → 82 | 3 (+0) → 3 | 15 (+15) → 90 | 60 (+40) → 260 | 72 (+45) → 297 | 66 (+30) → 216 |
| Blackguard | 20 (+15) → 95 | 20 (+10) → 70 | 24 (+10) → 74 | 18 (+10) → 68 | −1 (−1) → −6 | 8 (+10) → 58 | 65 (+40) → 265 | 35 (+15) → 110 | 40 (+30) → 190 |

All classes have `skill-dig:0:0`; digging comes from race (Dwarf +40), STR
(`adj_str_dig`), the weapon's weight (+weight/10) and diggers. Add the race column
from §2.1 and stat adjustments to get your actual values. Note that only Rogues gain
stealth with level (+1 per 10) and Blackguards *lose* it (−1 per 10, floored at 0).

### 4.3 How blows are calculated

`calc_blows()` (`src/player-calcs.c`) gives fractional blows (stored ×100):

1. `div = max(weapon weight, class min-weight)` (tenths of a pound);
2. `P = min(11, adj_str_blow[STR] × class STR-multiplier / div)`;
3. `D = min(11, adj_dex_blow[DEX])`;
4. energy per blow = `blows_table[P][D]` (a 12×12 table, 100 = one whole turn);
5. blows = `min(10000 / energy, 100 × max-attacks)`, plus 100 per extra blow from
   items/shapes/bloodlust; minimum 1 blow (2 with `birth_percent_damage`).

If the weapon is too heavy to hold (`adj_str_hold[STR] < weight/10`) you get the
"heavy wield" to-hit penalty and **only one blow**. Blessed-fighter classes
(Priest, Paladin) also get +2 to-hit and +2 to-dam when wielding a hafted or
`BLESSED` weapon (`PF_BLESS_WEAPON` in `calc_bonuses()`).

Blows per round with STR = DEX at the value shown (final, after race/class
modifiers), computed from the formulas above:

| Class | Stats | 3 lb | 6 lb | 12 lb | 15 lb | 20 lb | 30 lb |
|---|---|---|---|---|---|---|---|
| Warrior | 17/17 | 1.33 | 1.17 | 1.05 | 1.05 | 1.05 | 1.05 |
| Warrior | 18/50 | 4.54 | 3.03 | 2.00 | 2.00 | 1.66 | 1.66 |
| Warrior | 18/100 | 5.26 | 5.00 | 4.16 | 4.00 | 3.57 | 3.33 |
| Warrior | 18/150 | 5.88 | 5.88 | 5.26 | 4.76 | 4.76 | 4.34 |
| Warrior | 18/200 | 6.00 | 6.00 | 6.00 | 5.88 | 5.88 | 5.26 |
| Paladin | 18/50 | 4.54 | 3.03 | 2.00 | 2.00 | 1.66 | 1.66 |
| Paladin | 18/100 | 5.00 | 5.00 | 4.16 | 4.00 | 3.57 | 3.33 |
| Paladin | 18/150 | 5.00 | 5.00 | 5.00 | 4.76 | 4.76 | 4.34 |
| Blackguard | 18/50 | 2.38 | 2.38 | 2.00 | 2.00 | 1.66 | 1.66 |
| Blackguard | 18/100 | 4.34 | 4.34 | 4.16 | 4.00 | 3.57 | 3.33 |
| Blackguard | 18/150 | 5.00 | 5.00 | 5.00 | 4.76 | 4.76 | 4.34 |
| Rogue | 18/50 | 4.16 | 2.77 | 2.00 | 1.66 | 1.66 | 1.33 |
| Rogue | 18/100 | 5.00 | 4.76 | 4.00 | 3.57 | 3.33 | 2.85 |
| Ranger | 18/50 | 3.84 | 2.77 | 2.00 | 1.66 | 1.66 | 1.33 |
| Ranger | 18/100 | 5.00 | 4.76 | 4.00 | 3.57 | 3.33 | 2.85 |
| Priest / Druid / Necro | 18/50 | 3.33 | 2.38 | 1.66 | 1.66 | 1.66 | 1.33 |
| Priest / Druid / Necro | 18/100 | 4.00 | 4.00 | 3.57 | 3.33 | 2.85 | 2.85 |
| Mage | 18/50 | 2.38 | 2.00 | 1.66 | 1.33 | 1.33 | 1.33 |
| Mage | 18/100 | 4.00 | 4.00 | 3.33 | 2.85 | 2.85 | 2.38 |

Observations:

* **Blackguards gain nothing from light weapons**: their min-weight is 10 lb, so a
  3 lb dagger counts as 10 lb. Use the heaviest weapon that keeps your blows.
* Warriors are the only class that can reach 6 blows; at 18/100 STR+DEX a 6 lb weapon
  already gives 5.
* With low stats (17/17) *everyone* gets about one blow — early damage comes from
  weapon dice and to-dam, not blows. STR and DEX into the 18/xx range are the main
  melee upgrades of the early midgame (stat potions and items with +STR/+DEX).
* The in-game weapon inspection (`I`nspect a weapon) shows exactly this calculation,
  including what +STR/+DEX would give you (the "hypothetical blows" hack in
  `calc_bonuses()`).

### 4.4 Class special abilities (player flags)

Definitions are in `lib/gamedata/player_property.txt`; the code that implements each:

| Flag | Classes | What it does (code) |
|---|---|---|
| `BRAVERY_30` | Warrior | Immune to fear from character level 30 (`player_flags()` in `src/player.c` turns on `PROT_FEAR`). Fear otherwise blocks melee entirely. |
| `NO_MANA` | Warrior | Cannot cast; also set dynamically for any class whose max SP is 0 (`calc_bonuses()`). |
| `SHIELD_BASH` | Warrior, Paladin, Blackguard | Before a melee round, may bash with a worn shield (`attempt_shield_bash()`, `src/player-attack.c`). Chance = melee skill/8 + DEX to-hit/2 (×4 unarmed, ×2 if weapon dice×blows < shield dice×3) vs `randint0(200 + monster level)`; only against monsters of level ≥ your level/2. Damage = shield dice × (quality/40 + level/14) + STR to-dam, max 125; may stun or confuse; you may stumble and lose 26–75% of a turn. |
| `BLESS_WEAPON` | Priest, Paladin | +2 to-hit and +2 to-dam with hafted or `BLESSED` weapons. In 4.2 there is no penalty for edged weapons. |
| `ZERO_FAIL` | Mage, Druid, Priest, Necromancer | Spell failure can drop below 5% (to the stat-based minimum, as low as 0%). Others never go below 5% (`spell_chance()`). |
| `BEAM` | Mage | Bolt spells become beams with probability = character level % (other classes: level/2 %) (`beam_chance()` in `src/player-spell.c`). |
| `CHOOSE_SPELLS` | Mage, Druid, Necromancer, Rogue, Ranger, Blackguard | You pick which spell to learn. Priests and Paladins learn a **random** spell from the book you study (`do_cmd_study_book()`, `src/cmd-obj.c`). |
| `CHARM` | Druid | Sleep, slow, confuse, hold, stun and polymorph effects get +50% power against animals (`project_monster_*` handlers in `src/project-mon.c`). |
| `UNLIGHT` | Necromancer | Resist Dark; see nearby grids without light; spells fail +25% when standing on a lit grid. Details in §5.5. |
| `EVIL` | Necromancer | Resist Nether, but **vulnerable** to holy orb (`calc_bonuses()` sets NETHER res 1, HOLY_ORB res −1). |
| `STEAL` | Rogue | `s`teal command takes items from monsters (`steal_monster_item()`, `src/mon-util.c`). Details in §5.7. |
| `FAST_SHOT` | Ranger | +0.1 shots per 3 character levels with any launcher (`num_shots += level/3` tenths in `calc_bonuses()`). |
| `COMBAT_REGEN` | Blackguard | Mana comes from fighting and fades over time; spending mana heals. Details in §5.9. |

### 4.5 Starting equipment

With `birth_start_kit` on (the default), each class gets (`equip:` lines in
`class.txt`; quantities are random in the range shown):

| Class | Weapon | Armour | Consumables | Book |
|---|---|---|---|---|
| Warrior | Dagger (1d4, 1.2 lb) | Soft Leather Armour [8] | 1–3 Rations, 1–3 Wooden Torches, 1 Potion of Berserk Strength, 1 Word of Recall | — |
| Mage | Rapier (1d6, 4.0 lb) | — | 1–3 Rations, 1–3 Torches, 1 WoR | [First Spells] |
| Druid | Whip (1d3, 3.0 lb) | — | 1–3 Rations, 1–3 Torches, 1 WoR | [Lesser Charms] |
| Priest | Mace (2d4, 12.0 lb) | — | 1–3 Rations, 1–3 Torches, 2 Potions of Cure Serious Wounds, 1 WoR | [Novice's Handbook] |
| Necromancer | Main Gauche (1d5, 3.0 lb) | — | 1–3 Rations, 1 WoR (**no torch**) | [Into the Shadows] |
| Paladin | Main Gauche (1d5, 3.0 lb) | — | 1–3 Rations, 1–3 Torches, 1 Scroll of Protection from Evil, 1 WoR | [Novice's Handbook] |
| Rogue | Dagger (1d4, 1.2 lb) | Soft Leather Armour [8] | 1–3 Rations, 1–3 Torches, 1 WoR | [First Spells] (magic book) |
| Ranger | Main Gauche (1d5, 3.0 lb), Short Bow (×2) + 15–20 Arrows | — | 1–3 Rations, 1–3 Torches, 1 WoR | [Lesser Charms] |
| Blackguard | Tulwar (2d4, 10.0 lb) | Leather Shield [8] | 1–3 Rations, 1–3 Torches, 1 Potion of Cure Light Wounds, 1 WoR | [Into the Shadows] |

The Word of Recall scroll is omitted if `birth_no_recall` is set. With
`birth_start_kit` off you get only 1 ration and 1 light source (Necromancers: only the
ration) and keep the gold instead (`player_outfit()`, `src/player-birth.c`).

---

## 5. Class guides

Each guide gives the class data, its special mechanics, a phase-by-phase plan and
the usual ways it dies. Dungeon levels are written "DL" (DL10 = 500 ft); character
levels "CL".

General early-game shopping list for everyone (stores from `store.txt`): the
General Store always stocks Flasks of Oil (a `THROWING` item that explodes on
impact), Cure Light Wounds potions and Phase Door / Word of Recall scrolls are always
in the Alchemist. Flasks of oil are the strongest attack most level-1 characters own —
buy 10–20 unless you are a caster whose first spell already out-damages them.

### 5.1 Warrior

**Data.** STR +3, INT −2, WIS −2, DEX +2, CON +2 · hit die 9 · up to **6 blows**,
min weapon weight 3 lb, STR multiplier 5 · melee 70 (+45/10 levels), shooting and
throwing 55 (+45) — the best melee *and* equal-best overall missile progression ·
device 18 (+7), save 18 (+10), stealth 0 · flags `BRAVERY_30`, `NO_MANA`,
`SHIELD_BASH` · titles Rookie → Lord.

**Special abilities.**

* *Most blows in the game.* `max-attacks:6` and multiplier 5 (shared with Paladin and
  Blackguard) — see the blows table in §4.3.
* *Relentless (Bravery 30).* From CL30 the Warrior is permanently immune to fear.
  Before that, fear (from spells, scary monsters, some melee blows) prevents melee
  entirely; carry Potions of Boldness, Heroism (heal 10, cure fear, 25+1d25 turns of
  Hero) or Berserk Strength (heal 30, cure fear, 25+1d25 turns of berserk) — data in
  `object.txt`. The Warrior's starting kit includes one Berserk Strength potion for
  exactly this reason.
* *Shield bash.* Free extra attack chance when wearing a shield against monsters at
  least half your level (§4.4).
* *No spells.* All detection, escape and healing come from consumables and devices,
  and device skill is the worst of all classes (53 at CL50 before race/INT).

**Stat priorities.** STR = DEX > CON. INT/WIS are dump stats (WIS does still feed
the saving throw via `adj_wis_sav`). Aim for 18/xx STR and DEX as early as possible:
blows jump from ~1 to 2–3 between 18/00 and 18/50.

**Early game (CL1–15, DL1–10).**
* Buy Flasks of Oil, Potions of Cure Light Wounds, Scrolls of Phase Door; upgrade
  armour (the Armoury sells Leather Shields, Hard Leather Caps and Leather Boots).
  Buy a Sling (or a bow) — warriors have the same shooting skill progression as their
  throwing and a launcher lets you soften monsters before melee.
* Swap the dagger for whatever gives the best `blows × (dice + to-dam)` — use the
  `I`nspect command, which prints blows and average damage per round.
* Fang and Grip (Farmer Maggot's dogs, native depth DL2) and Bullroarer (DL5) are
  good early experience for a warrior; leave Wormtongue (DL8, a spellcaster) until you
  have more HP and an escape.

**Midgame (CL15–35, DL10–40).**
* Priorities: **Free Action** (before DL20–25: carrion crawlers at DL25 and ghouls at
  DL26 paralyse with every claw), **See Invisible** (ghosts from DL31), the four base
  resists, **resist Poison**, then speed.
* Detection: Rods of Treasure Location are cheap, but what you need are Rods of
  Detection/Magic Mapping, Staffs of Detect Evil/Mapping and scrolls of Magic Mapping
  and Deep Descent. A warrior without detection walks into hound packs and vaults
  blind.
* Escapes: always carry Phase Door (15+), Teleportation (5+ scrolls or a staff),
  Word of Recall, and Potions of Cure Serious/Critical Wounds. Teleport Other (wand)
  is a superb tool for a warrior.
* From CL30, fear is no longer a problem — you can drop the Boldness potions.

**Late game (CL35+, DL40+).**
* Collect resistances (Nether, Chaos, Confusion, Blindness, Sound, Shards, Nexus,
  Disenchantment), Telepathy and +10 speed or more. Warriors are the best damage
  dealers in melee but must rely on Potions of Healing/*Healing*/Life and on devices
  for everything else.
* Throwing weapons: with throw skill 280 at CL50 (before race), warriors can make good
  use of throwing weapons found in the dungeon.

**Typical deaths and how to avoid them.**
1. *Paralysis* without Free Action — never melee a floating eye (paralysing gaze, DL1)
   and treat carrion crawlers/ghouls/basilisks as lethal until you have FA.
2. *Invisible monsters* (poltergeists early, ghosts later) — buy Detect Invisible
   scrolls or find See Invisible.
3. *Breath attacks from packs* (hounds, young dragons around DL29) — carry the base
   resists and use corridors.
4. *Running out of escapes* — never descend with fewer than ~5 Teleportation and
   ~10 Phase Door.
5. *Fear before CL30* — keep Boldness/Heroism/Berserk potions.

### 5.2 Mage

**Data.** STR −3, INT +3, WIS 0, DEX 0, CON −2 · hit die **0** (only the race's) ·
max 4 blows, min weight 4 lb, multiplier 2 · device 36 (+13) — best class device skill
· save 30 (+9) · disarm magic 35 (+12) · melee 35 (+15) · realm **arcane (INT)**,
first spell CL1, armour limit 30 lb, 5 books · flags `ZERO_FAIL`, `BEAM`,
`CHOOSE_SPELLS` · titles Novice → Arch-Mage.

**Special abilities.**
* *Beaming bolts.* Magic Missile, Frost Bolt and Mana Bolt are `BOLT_OR_BEAM`; the
  Mage's beam chance is equal to character level (Magic Missile has a −10 modifier),
  so a CL30 Frost Bolt beams 30% of the time (`beam_chance()`, `effect_handler_BOLT_OR_BEAM()`).
* *Zero-fail casting.* With 18/200+ INT the minimum fail of every spell can reach 0%.
* *Spell choice.* You choose which spell to learn with each new slot.
* *Best device skill* — wands and rods supplement the spell list from the start.

**Stat priorities.** INT >> DEX ≥ CON > STR. INT gives spells per level, mana and fail
rate (Chapter 3). CON is the life insurance a 0-hit-die class badly needs.

**Spellbooks.** [First Spells] (town), [Attacks and Knowledge] (town),
[Magical Defences] (town), [Arcane Control] (dungeon, DL40+), [Wizard's Tome of Power]
(dungeon, DL60+). Full spell data in Chapter 3 §5.

**Early game (CL1–15).**
* Magic Missile (3d4 at CL1, `(CL−1)/5+3` d4, 1 mana) is the workhorse; at CL2 you
  get Phase Door as a *spell* (2 mana) and Electric Arc; at CL6, **Fire Ball** (2×CL
  damage, radius 2) for groups. Find Traps, Doors & Stairs (1 mana, CL1) is your
  stair-finder for diving; Detect Monsters (CL3) tells you what is waiting.
* Buy [Attacks and Knowledge] as soon as you can afford it: Identify Rune (CL8),
  Recharging (CL5) and **Frost Bolt** at CL13 (8d8 at CL13).
* Wear the lightest armour that adds AC; you have a 30 lb budget before losing mana
  (Chapter 3 §2.3) — e.g. Soft Leather Armour (8 lb) + Cloak (1 lb) + Leather Boots
  (2 lb) + Hard Leather Cap (2 lb) + Leather Gloves (1 lb) + Wicker Shield (3 lb) is
  only 17 lb. **There is no glove penalty for spellcasters in 4.2.6** (see Chapter 3).
* You have very few hit points: a Hobbit/Gnome Mage has only 7–8 hit die. Never fight
  in the open; use corridors so only one monster reaches you, and Phase Door away from
  anything that gets adjacent.

**Midgame (CL15–35).**
* [Magical Defences] (town, expensive) gives **Teleport Self** (CL7 — learnable early
  if you buy the book), **Teleport Other** (CL15), Resistance (CL20), Tap Magical
  Energy (CL22) and Mana Channel (CL25, faster casting). Teleport Other is the
  single most important defensive spell in the arcane list.
* Acid Spray (CL20) is a 60° cone of (CL/2)d8 — the main group-clearing attack until
  the dungeon books.
* Free Action and See Invisible are as vital as for anyone; Mages lack HP so speed and
  resistances matter even more.

**Late game (CL35+).**
* [Arcane Control] (DL40+): **Mana Bolt** (CL25; (CL−10)d8 unresistable),
  **Teleport Level**, Detection (everything in one cast), **Dimension Door** (CL35,
  teleport to a chosen square) and Thrust Away.
* [Wizard's Tome of Power] (DL60+): Shock Wave, Explosion, **Banishment** (CL35),
  **Mass Banishment** (CL40) and **Mana Storm** (CL45; 2×CL+300 mana ball, radius 3).

**Typical deaths.**
1. *Being caught at low HP* — one breath or a double move by a fast melee monster;
   keep HP topped up and never let monsters reach you.
2. *Blindness/confusion* — you can't cast (`player_can_cast()`); carry Potions of
   Cure Light Wounds (cure blindness, reduce confusion by 20) and Cure Serious Wounds
   (cure blindness and confusion) — `object.txt` — or get protection from
   blindness/confusion.
3. *Running out of mana* in a fight — rest to full before exploring; casting with
   insufficient mana can paralyse you and damage CON (Chapter 3 §2.6).
4. *Paralysis* — Free Action is mandatory.

### 5.3 Druid

**Data.** STR −2, INT 0, WIS +3, DEX −2, CON 0 · hit die 2 · max 4 blows, min weight
3.5 lb, multiplier 3 · melee 45 (+20), shooting/throwing 40 (+30) · save 30 (+10) ·
realm **nature (WIS)**, first spell CL1, armour limit 35 lb, 5 books · flags
`ZERO_FAIL`, `CHOOSE_SPELLS`, `CHARM` · titles Wanderer → High Mystic. Druids start
with a whip and no armour.

**Special abilities.**
* *Shapechanging.* Fox Form (CL1), Become Pukel-man (CL30), Eagle's Flight (CL35),
  Bear Form (CL40). A shape adds its bonuses **on top of your equipment** (gear
  "merges into your body" but `calc_bonuses()` still counts it; you still hit with your
  wielded weapon, only the attack verb changes — `py_attack_real()`), but you cannot
  use items, read, quaff, cast or enter shops other than your home while changed.
  Any such command asks whether to change back; answering `y` reverts and performs
  the command in the same action. Full shape stats in Chapter 3 §13.
* *Charm.* +50% power for sleep, slow, confusion, hold and stun effects against
  animals (`project-mon.c`): Confuse Monster, Slow Monster, Trance and Mass Sleep are
  noticeably better versus animal packs.
* *Stinking Cloud at CL3* — a radius-2 poison ball doing CL/2+10.

**Stat priorities.** WIS >> CON > DEX > STR. A Dwarf (WIS +5 total) or Dúnadan
(WIS +5) Druid has strong mana and fail rates.

**Early game.** Your offence is Stinking Cloud (CL3) and, when outnumbered, Confuse/
Slow Monster. Fox Form (CL1, 2 mana) gives +1 blow, +1 move (moving costs half
energy), +5 stealth, Free Action and Feather Falling, at −3 STR and −3 to-hit/to-dam —
useful for scouting and running, and the Free Action is a real bonus at DL1–20.
Remove Hunger (CL2) means you never need to carry food. Buy [Gifts of Nature] for
Stone to Mud (CL8), Sense Surroundings (CL9, maps 22 rows × 44 columns each way), and
**Lightning Strike** (CL12) — your main single-target attack for a long time.

**Midgame.** Earth Rising (CL14) is a shard short-beam (CL/3+2)d6 reaching 4+CL/5
squares. [Creature Dominion] (DL30+) gives Trance (CL20, holds adjacent monsters) and
**Mass Sleep** (CL25) — Druids are the best class at simply putting a room to sleep
and walking past. [Nature Craft] (DL40+) gives Tremor, **Haste Self** (CL25),
Revitalize, Rapid Regeneration and Herbal Curing.

**Late game.** [Wild Forces] (DL60+): **Meteor Swarm** (CL30), **Rift** (CL35;
40+CL d7 gravity beam), **Ice Storm** (CL37; 3d(3×CL) to everything in line of sight),
**Volcanic Eruption** (CL40) and **River of Lightning** (CL43; (CL+10)d8 plasma arc).
Pukel-man form (immune to poison, +20 AC, 10 points of damage reduction, −5 speed) and
Bear form (+15 to-hit/to-dam, +1 blow) turn a high-level druid into a credible
melee character.

**Typical deaths.** Low HP plus no early escape spell: Druids have **no teleport spell
at all** — carry Phase Door and Teleportation scrolls and remember you must leave
animal form to read them (the change-back is free when you answer `y`). Poor DEX and
STR means melee is weak until a shape helps.

### 5.4 Priest

**Data.** STR −1, INT −3, WIS +3, DEX −1, CON +1 · hit die 2 · max 4 blows, min weight
3.5 lb, multiplier 3 · melee 45 (+20) · save 32 (+12) — best class save progression ·
realm **divine (WIS)**, first spell CL1, armour limit 35 lb, 5 books · flags
`BLESS_WEAPON`, `ZERO_FAIL` (no `CHOOSE_SPELLS`) · titles Believer → High Priest.
Starts with a Mace (2d4, a hafted weapon, so the blessed-fighter bonus applies) and
two Potions of Cure Serious Wounds.

**Special abilities.**
* *Random prayer learning.* Priests do not choose: studying picks a random learnable
  prayer from the book you study (`do_cmd_study_book()`). You can still control
  *which book* you study from.
* *Blessed fighter.* +2 to-hit/+2 to-dam with hafted or blessed weapons.
* *Healing.* Shares Healing (2000 HP, CL30) only with the Paladin, and alone has
  Holy Word (heal 1000 + dispel evil), Restoration and Remembrance.

**Stat priorities.** WIS >> CON > STR = DEX. INT is irrelevant.

**Early game.** Call Light, Detect Evil, Minor Healing and **Bless** at CL1 (Bless:
+5 AC, +10 to-hit for CL+10+d(CL+10) turns). Your attack is the mace — or flasks of
oil — until **Orb of Draining** (CL7, [Cleansing Power], a cheap town book: radius-2
ball of 3/2×CL+3d6, double vs evil). Heroism (CL5) cures fear immediately but only
grants heroism from CL20.

**Midgame.** Orb of Draining carries you to the dungeon books; it gains radius 3 at
CL30. Dispel Undead (CL10) and Dispel Evil (CL12) hit everything in view.
Protection from Evil (CL14) repels melee from evil monsters whose level ≤ yours
(success when `randint0(100) + CL > 50`, `src/mon-attack.c`). [Healing and Sanctuary]
(DL30+) gives **Portal** (CL10, teleport CL/2+30), Remembrance, **Word of Recall**
(CL25), **Healing** (CL30), Restoration and Clairvoyance — finding it changes the game.

**Late game.** [Battle Blessings] (DL40+) and [Wrath of the Valar] (DL60+):
Glyph of Warding, Smite Evil, Banish Evil, Word of Destruction, **Holy Word**
(heal 1000 + dispel evil d(4×CL)), Spear of Oromë, Light of Varda.

**Typical deaths.** Slow early kill speed leading to being swarmed; no escape spell
before Portal (dungeon book) — carry scrolls; confusion/blindness preventing prayers
(Dwarf priests are blind-proof).

### 5.5 Necromancer

**Data.** STR −3, INT +3, WIS 0, DEX 0, CON −2 · hit die 2 · max 4 blows, min weight
3.5 lb, multiplier 3 · melee 35 (+25) · device 36 (+13) · realm **shadow (INT)**,
first spell CL1, armour limit 30 lb, 5 books · flags `ZERO_FAIL`, `CHOOSE_SPELLS`,
`UNLIGHT`, `EVIL` · titles Acolyte → Night Lord. **No torch** in the starting kit.

**The Unlight mechanics** (all from source):

| Effect | Rule | Source |
|---|---|---|
| See without light | With current light radius ≤ 1, you see grids at distance < 2 + CL/6 − light | `update_view_one()` in `cave-view.c` |
| Always see your own grid | So you can read scrolls and cast in total darkness (`no_light()` is false) | `update_view()` in `cave-view.c` |
| Resist Dark | `res_level = 1` for darkness; not blinded by DARK_WEAK | `calc_bonuses()`, `project-player.c` |
| Light penalty for spells | **+25% fail** when your grid is lit (applied before the 50% cap) | `spell_chance()` in `player-spell.c` |
| Light gear adjustment | Items with a positive light *modifier* give one less light radius | `calc_light()` in `player-calcs.c` |
| Darkened grids | When you darken an area, you still remember its layout | `cave_unlight()` in `cave-map.c` |

`EVIL`: resist Nether, but **vulnerable to holy orb** (`res_level = −1`).

**What this means.** A Necromancer plays in the dark. Carrying a normal torch
(radius 2) lights your square, costs 25% on every spell and switches off your
unlight vision. Walk through unlit corridors, cast Create Darkness (CL3) to darken
lit rooms, and fight from shadow. Your unlight sight radius grows to 7 at CL30
(2 + 30/6).

**Stat priorities.** INT >> CON ≥ DEX > STR.

**Early game.** Nether Bolt (CL1, (CL/4+3)d4, 1 mana) is your only attack — and
**undead are immune to nether, evil monsters take half**, non-resisting non-evil
monsters full (`project_monster_handler_NETHER()`). Many early orcs are evil, so
expect reduced damage; animals and most early non-evil monsters take full damage.
Bat Form (CL3) is a scouting shape (+3 speed, see invisible, feather fall, but −10
to-dam and costs 5 HP to assume). Read Minds (CL4) maps around minds you detect.
[Dark Rituals] (town) brings **Tap Unlife** (CL7, damage undead and gain mana),
**Crush** (CL10; instantly kills every monster in view with fewer than 4×CL HP, at the
cost of d(2×CL) damage to you), Sleep Evil, **Shadow Shift** (CL15, short teleport)
and Disenchant.

**Midgame.** [Fear and Torment] (town): Frighten, **Vampire Strike** (CL20: jump next
to the nearest living monster and drain up to 2×CL HP into your own HP and food),
Dispel Life, **Dark Spear** (CL25) and Warg Form (CL28). [Deadly Powers] (DL40+):
Banish Spirits, **Annihilate** (4×CL to one living target), Grond's Blow (destruction
— you resist dark so you are not blinded), Unleash Chaos, Fume of Mordor, Storm of
Darkness.

**Late game.** [Corruption of Spirit] (DL60+): Power Sacrifice (50 HP → 50 mana),
Zone of Unmagic, **Vampire Form**, Curse and **Command** (take control of a monster).

**Typical deaths.** Fighting in lit rooms with high fail rates; using HP-costing
spells (Crush, Shadow Shift, Power Sacrifice, Curse costs 100 HP) when already low;
running into undead with only Nether Bolt; holy-orb casting priests (vulnerability).
Teleport Self is not in the shadow list — Shadow Shift (30 squares) is your only
teleport spell, so carry Teleportation scrolls/staves.

### 5.6 Paladin

**Data.** STR +1, INT −3, WIS +1, DEX −1, CON +2 · hit die 6 · max 5 blows, min weight
3 lb, multiplier 5 · melee 65 (+40), shooting/throwing 50 (+30) · save 25 (+11) ·
realm **divine (WIS)**, first spell CL1, armour limit **40 lb**, 3 books (Novice's
Handbook; dungeon: Healing and Sanctuary, Battle Blessings) · flags `BLESS_WEAPON`,
`SHIELD_BASH` (random prayer learning) · titles Gallant → Paladin Lord. Starts with
a Scroll of Protection from Evil.

**Playstyle.** A Warrior with prayers instead of the Warrior's 6th blow and
bravery. Bless at CL1 (+10 to-hit, +5 AC) makes the early game easy; Minor Healing
(CL7) and **Heroism** (CL12, heroism from CL15) cover fear. The dungeon books give
Protection from Evil (CL15), Remove Curse, **Word of Recall** (CL25), **Healing**
(CL30, 2000 HP), Clairvoyance, Smite Evil (CL25), Demon Bane, Enchant Weapon/Armour
and **Single Combat** (CL40: duel a monster in an arena).

**Stat priorities.** STR = DEX > CON > WIS. WIS only needs to be "enough" for low
fail rates on the few prayers you use (Paladins are not zero-fail: 5% floor).

**Early/mid/late.** Play exactly as a Warrior (§5.1), using Bless before fights,
Heroism against fear, and Minor Healing between fights to save potions. Prefer hafted
weapons (blessed-fighter +2/+2) unless an edged weapon is clearly better. Finding
[Healing and Sanctuary] at DL30+ gives a self-heal of 2000 HP at CL30 — a defining
power spike.

**Typical deaths.** Same as the Warrior, except fear is handled by Heroism instead of
CL30 bravery; weaker devices than a priest (device 24+10).

### 5.7 Rogue

**Data.** STR 0, INT +1, WIS −3, DEX +3, CON −1 · hit die 4 · max 5 blows, min weight
**2 lb**, multiplier 4 · disarm 45/45 (+20) — best in the game · stealth 3 (+1 per 10
levels, the only class that improves) · search 20 (+16) · shooting 66 (+30) ·
**throwing 72 (+45)** — best in the game · melee 35 (+45) · realm **arcane (INT)**,
first spell **CL5**, armour limit 35 lb, 2 books ([First Spells] town, [Arcane
Control] dungeon) · flags `CHOOSE_SPELLS`, `STEAL` · titles Vagabond → Master Thief.

**Stealing** (`s` command → `steal_monster_item()` in `src/mon-util.c`):

* Your *steal skill* = stealth skill + DEX to-hit adjustment (quartered if you are
  blind, confused or hallucinating).
* The monster's *guard* = monster level × 3/4 (× 4/4 for uniques) + monster speed −
  your speed; halved if the monster is asleep.
* *Reaction* = guard/2 + 1d(guard) + (item weight × quantity)/20.
* Reaction < skill → **success**: you take a random item (or its gold) from the
  monster's inventory; it wakes a little (sleep reduced by 35 − stealth).
* Reaction/2 < skill → failure, the monster wakes (and may notice you).
* Otherwise → bungled: the monster wakes, cries out and **wakes everything nearby**.
* With *Hit and Run* active (spell, CL23), you teleport 20 squares after the attempt.

Steal from **sleeping**, slow, low-level monsters carrying items (the monster recall
tells you which races carry objects). Fast monsters and uniques are much harder.

**Spells.** Rogues are 5%-floor casters with a short utility list: Detect Monsters
(CL5), Phase Door (CL7), **Object Detection** (CL10 — real detection of objects and
gold, better than the Mage's "sense"), Detect Stairs (CL12, wide area), Recharging
(CL20), Reveal Monsters (CL25); [Arcane Control]: Teleport Self (CL17), Hit and Run
(CL23), Teleport Other (CL30), Teleport Level (CL35).

**Stat priorities.** DEX > STR > INT > CON. DEX helps blows, stealing and disarming;
INT gives spell access (spells start at CL5).

**Playstyle.** Rogues are stealthy skirmishers: sling or throwing weapons first,
melee to finish. A Hobbit or Kobold Rogue moves almost silently, picks fights and
loot, and avoids the rest. Disarm skill makes vaults and trapped chests safe.
Early, buy a Sling and Iron Shots (General Store always stocks them) plus Flasks of
Oil — your throwing skill makes oil flasks unusually effective.

**Typical deaths.** Bungled steals waking a whole room; low HP in melee; having to
rely on 50–55% base-fail Phase Door early (cast it a lot when safe to bring the fail
down via levels).

### 5.8 Ranger

**Data.** STR 0, INT 0, WIS +2, DEX +1, CON −1 · hit die 5 · max 5 blows, min weight
3.5 lb, multiplier 4 · **shooting 72 (+45)** — best in the game · melee 60 (+40) ·
disarm phys 40 (+15) · realm **nature (WIS)**, first spell **CL3**, armour limit
**40 lb**, 2 books ([Lesser Charms] town, [Nature Craft] dungeon) · flags `FAST_SHOT`,
`CHOOSE_SPELLS` · titles Runner → Ranger Lord. Starts with a Short Bow (×2) and
15–20 arrows.

**Extra shots.** `FAST_SHOT` adds CL/3 tenths of a shot to any launcher
(`calc_bonuses()`): 1.0 shots at CL1–2, 1.5 at CL15, 2.0 at CL30, 2.5 at CL45,
2.6 at CL48–50 — on top of any `SHOTS` modifiers on your launcher. Not applied if the
launcher is too heavy for your STR.

**Spells.** Remove Hunger (CL3), Detect Life (CL5), **Herbal Curing** (CL9 — cures
cuts, poison, stun, black breath), Resist Poison (CL12), Stone to Mud (CL15), Sense
Surroundings (CL20); [Nature Craft]: Cover Tracks (CL20), **Create Arrows** (CL22,
turns a staff into arrows), **Haste Self** (CL25), Decoy (CL30), Brand Ammunition
(CL40).

**Stat priorities.** DEX > STR > CON > WIS. STR matters for heavier bows (holding a
launcher whose weight/10 exceeds `adj_str_hold` gives the heavy-shoot penalty and
cancels extra shots).

**Playstyle.** Shoot everything. Buy as many arrows as you can carry (the General
Store and Weaponsmith always/normally stock them), upgrade to a Long Bow when your
STR allows, and keep the quiver full. In corridors a Ranger kills most things before
they arrive. Haste Self (dungeon book) and Create Arrows make the midgame smooth.

**Typical deaths.** Running out of ammo far from town; being caught in melee by
something fast; no teleport spell — carry scrolls.

### 5.9 Blackguard

**Data.** STR +2, INT 0, WIS −3, DEX 0, CON +2 · hit die 8 · max 5 blows, min weight
**10 lb**, multiplier 5 · melee 65 (+40) · save 18 (+10) · **stealth −1 (−1 per 10
levels)** · realm **shadow (INT)**, first spell CL1, armour limit **60 lb**, 3 books
([Into the Shadows], [Fear and Torment] town; [Deadly Powers] dungeon) · flags
`CHOOSE_SPELLS`, `SHIELD_BASH`, `COMBAT_REGEN`; object flag `IMPAIR_HP` · titles
Rat → Tyrant. Starts with a Tulwar (2d4, 10 lb) and a Leather Shield.

**Combat regeneration** (all in `src/player-util.c` / `src/player-attack.c`):

| Event | Effect |
|---|---|
| Taking damage | Gain SP in proportion: losing X% of max HP gives X% of max SP (max SP treated as at least 10). Poison, bleeding ("a fatal wound") and starvation damage don't count. |
| Making a melee attack (`py_attack`) | Gain 5% of max SP (treated as ≥10) per attack command. |
| Over time | SP **decays** at half the normal regeneration rate (`percent /= -2`), and decaying SP heals HP at double the casting efficiency. |
| Casting a spell | Heals HP: spending X% of max SP restores X/2 % of *missing* HP, capped at 25% of missing HP per cast (`convert_mana_to_hp()`). |
| Regeneration bonuses | Resting and Regeneration items only boost SP regen while HP ≤ 50%. |
| `IMPAIR_HP` | HP regeneration halved. |

So a Blackguard starts every fight with little or no mana, *earns* it by hitting and
being hit, then spends it on spells that also heal. Resting does not fill your mana.

**Bloodlust** (spell at CL30, [Deadly Powers]; `TMD_BLOODLUST`, max 50):

* Casting gives 10 points of bloodlust; **casting again while it is active reduces
  it by 1** (`TIMED_INC:BLOODLUST:0:-1` — the "other" parameter is the increment when
  already active).
* Every kill adds 10 (`mon-util.c`), with a 5% chance each of 1d3 turns confusion and
  of 1d10 turns hallucination.
* While active: +bloodlust/2 to-dam and +bloodlust/20 extra blows (`calc_bonuses()`)
  — up to +25 to-dam and +2 blows at 50.
* Each command that uses energy has a bloodlust/200 chance to be replaced by an attack
  on a random adjacent monster (`cmd-core.c`).
* While bloodlust is below 10 each game turn has a (10 − bloodlust)% chance of HP loss,
  cuts or slowing (`process_world()`).
* Each miss: 1-in-50 chance of scrambled stats; each hit: 1-in-50 chance of CON
  damage (`player-attack.c`).
* **Death protection**: if you drop below 0 HP but `HP + bloodlust + CL ≥ 0`, you
  survive ("Your lust for blood keeps you alive!", `take_hit()`).

**Other spells.** Seek Battle (CL1), **Berserk Strength** (CL3: cure fear and
berserk rage — +75 melee skill, −10 AC, immunity to fear, device skill −10%), Whirlwind Attack (CL5: attack all adjacent), Shatter Stone,
**Leap into Battle** (CL10), Grim Purpose (CL13: temporary Free Action and confusion
protection), Maim Foe (CL15, stunning blows), Howl of the Damned (CL18, scare all in
view), Relentless Taunting (CL24), Venom (CL28, poison brand), **Werewolf Form**
(CL32), Unholy Reprieve (CL34), Forceful Blow (CL38), Quake (CL44).

**Stat priorities.** STR > CON > DEX > INT. INT raises the max SP pool that
combat regeneration fills (and lowers fail), but STR/CON win fights.

**Playstyle.** Wade in with a heavy weapon and a shield (shield bash + 60 lb armour
allowance). Open fights with Berserk Strength once you have mana (from the first hit
you take), use Whirlwind Attack against groups, and let spellcasting double as
healing. Blackguards are noisy (stealth falls with level) — expect to wake things.

**Typical deaths.** Slow out-of-combat healing (`IMPAIR_HP`) leads to starting fights
hurt; no mana when you need an emergency spell (combat regen only); poor saving throw
(18 base, WIS −3) vs. paralysis/confusion; bloodlust side-effects at the wrong time.

---

## 6. Character birth: stats, rollers and birth options

### 6.1 Point-based stats (recommended)

`src/player-birth.c`: every stat starts at a base of **10** and you have
**20 points** (`MAX_BIRTH_POINTS`). The cost of each increase is
`birth_stat_costs[]`:

| Raise base to | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 |
|---|---|---|---|---|---|---|---|---|
| Cost of this step | 1 | 1 | 1 | 1 | 1 | 1 | 2 | 4 |
| Total cost from 10 | 1 | 2 | 3 | 4 | 5 | 6 | 8 | 12 |

So 20 points buys, for example: **18 + 17** (12 + 8); **18 + 16 + 12** (12 + 6 + 2);
**17 + 17 + 14** (8 + 8 + 4); **16 + 16 + 16 + 12** (6 + 6 + 6 + 2).

Why 18 is worth 4 points: race/class bonuses beyond 18 add ten percentile points
each (§1.1). A Human Mage (INT +3) gets 18/30 INT from a base 18 but only 18/20 from
a base 17, and a High-Elf Mage (INT +6) gets 18/60 vs. 18/50. For a caster that is
one index step on every spell table.

**Unspent points are wasted.** The birth screen shows "+50 gold per unspent point"
(`recalculate_stats()` sets `au_birth = 600 + 50 × points left`), but when you accept
the character `do_cmd_accept_character()` calls `get_money()`, which resets your gold
to `start-gold` (600 in `constants.txt`). Then the starting kit's value is deducted
(`player_outfit()`). Spend all 20 points.

**The automatic allocation** (`generate_stats()`), used when you let the game pick:
1. buy STR up to base 17;
2. non-casters: buy DEX up to 17 but sell back any points that don't reach a new
   blows breakpoint with your starting weapon;
3. spend up to half of the remainder on the spell stat (casters may go to 18) and on
   CON (up to 16); warriors put everything left into CON;
4. dump leftovers into DEX, then the non-spell mental stat.

It is a reasonable default for fighters but a poor one for pure casters (step 1
spends 8 points on STR 17 before the spell stat). Allocate manually instead.

**Suggested allocations** (base values; adjust for race):

| Class | Suggested bases (STR/INT/WIS/DEX/CON) | Reasoning |
|---|---|---|
| Warrior | 17 / 10 / 10 / 17 / 14 or 18 / 10 / 10 / 17 / 10 | Blows need both STR and DEX; check blows with your weapon on the birth screen |
| Mage | 10 / 18 / 10 / 16 / 12 or 10 / 18 / 10 / 10 / 17 | INT 18 first; then DEX (AC, a bit of blows) or CON (HP) |
| Druid | 10 / 10 / 18 / 10 / 17 | WIS 18, then CON |
| Priest | 12 / 10 / 18 / 10 / 16 | WIS 18, some STR for the mace, CON |
| Necromancer | 10 / 18 / 10 / 10 / 17 | INT 18, then CON |
| Paladin | 18 / 10 / 10 / 17 / 10 or 17 / 10 / 14 / 17 / 10 | Melee first; a little WIS helps fail rates |
| Rogue | 17 / 14 / 10 / 17 / 10 | DEX/STR for blows and throwing, INT for spells at CL5 |
| Ranger | 17 / 10 / 10 / 18 / 10 | DEX first (to-hit), STR for bows and blows |
| Blackguard | 18 / 10 / 10 / 10 / 17 or 17 / 14 / 10 / 10 / 16 | STR and CON; some INT enlarges the SP pool |

### 6.2 The stat roller

`get_stats()` rolls each stat as 5 + 1d3 + 1d4 + 1d5 (base 8–17) and rerolls the whole
set until the fifteen dice total more than 35 and less than 45 — so the five base
stats always sum to between 61 and 69. **The roller can never produce a base 18**,
and you cannot adjust rolled stats (buy/sell is locked once you roll). Point-buy is
therefore strictly better for any character that wants one stat as high as possible,
which is every class. Use the roller only for fun or variety.

### 6.3 Hit points at birth

Level-1 HP is your full hit die plus the CON bonus; the 49 later level rolls are made
when you accept the character (§1.3). There is no reason to re-roll for HP.

### 6.4 Birth options

From `src/list-options.h` (defaults) and their implementation:

| Option | Default | What it really does (code) | Advice |
|---|---|---|---|
| `birth_randarts` | off | Generates a new random artifact set (`do_randart()` at birth). | Turn on once you know the standard artifacts. |
| `birth_connect_stairs` | **on** | Taking stairs down puts you on an up staircase; taking stairs up puts you on a down staircase (`gen-util.c`, `create_up_stair`/`create_down_stair`). | Keep on. It makes "stair-scumming" possible: if a level looks dangerous, leave at once. Going up then immediately down generates a fresh level. |
| `birth_force_descend` | off | Up stairs don't work; down stairs and recall from town take you to **one level below your max depth**; Teleport Level only goes down; can't recall off a quest level (`cmd-cave.c`, `effect_handler_RECALL()`, `effect_handler_TELEPORT_LEVEL()`). | Challenge option. Makes Teleport Level an emergency "dive" and removes stair-scumming upward. |
| `birth_no_recall` | off | Word of Recall does nothing (until Morgoth is dead); the starting WoR scroll is not given. | Challenge option; with force_descend it recreates "ironman". |
| `birth_no_artifacts` | off | No artifacts are generated. | Masochism. |
| `birth_stacking` | **on** | Several objects may share a floor grid. | Keep on. |
| `birth_lose_arts` | off | Artifacts left on a level are lost forever (also affects level feelings — `generate.c`). | Neutral for most players (you rarely return to old levels). |
| `birth_feelings` | **on** | Shows level feelings (danger and, after exploring, loot quality). | Keep on; the danger feeling is an important survival tool. |
| `birth_no_selling` | **on** | Stores pay nothing; gold found in the dungeon is **×5** (`obj-make.c`). | Keep on — the standard way to play 4.2. Stores still identify items you "give" them. |
| `birth_start_kit` | **on** | Start with the class kit (§4.5), its value deducted from 600 gold. Off: only 1 ration + 1 light, keep 600 gold. | On for most players. Off lets you buy a different weapon or more flasks of oil. |
| `birth_ai_learn` | **on** | Monsters learn which elements you resist and avoid wasting those spells. | `docs/option.rst` calls it "not recommended" but the code default is on; it makes casters smarter. Turning it off is a mild easing. |
| `birth_know_runes` | off | All object properties ("runes") are known from the start. | Convenience; removes identify-by-use. |
| `birth_know_flavors` | off | All potion/scroll/etc. flavours known from the start. | Convenience. |
| `birth_levels_persist` | off | Levels are saved and re-entered as left (experimental). When on, recall from town lets you choose any depth up to your max. | Experimental — not recommended for a first game. |
| `birth_percent_damage` | off | To-dam bonuses add 5% of dice damage per +1 instead of flat damage; minimum 2 blows (`calc_blows()`). | Experimental and unbalanced (per `docs/option.rst`). |

**Recall depth.** When you activate Word of Recall from a level shallower than your
max depth (and levels don't persist) the game asks "Set recall depth to current
depth?" — answering yes *lowers your max depth*, which is useful after a dive that
went too deep. Recall activates after 15–34 player turns (`randint0(20) + 15`).

---

## 7. Recommended combinations

### 7.1 For beginners

| Combination | Why it is forgiving |
|---|---|
| **Dúnadan Warrior** | Best stat line (+4/0/0/+4/+5 total), 19 hit die, no spells to manage. Learn the monsters and the dungeon. |
| **Half-Troll Warrior** | 21 hit die, regeneration, STR +7. Very hard to kill early; devices are poor. |
| **Dwarf Priest** | WIS +5, blind-proof prayers, 13 hit die; Orb of Draining at CL7 and Healing later. |
| **Dúnadan Paladin** | Warrior-like melee plus Bless, healing and Heroism; 16 hit die. |
| **High-Elf Ranger** | Shooting 97 at birth (72 + 25), See Invisible, extra shots. Kills things before they arrive. |
| **Gnome Mage** | Free Action from turn one, best device skill, INT +5. The safest way to learn a pure caster (still fragile: 8 hit die). |
| **Dúnadan Blackguard** | Big HP and STR with simple, powerful melee spells; teaches the combat-regeneration mechanic. |

### 7.2 Strong combinations by class

| Class | Strong races | Notes |
|---|---|---|
| Warrior | Half-Troll, Dúnadan, High-Elf, Dwarf | High-Elf: STR +4 DEX +5 and See Invisible; slower levelling. |
| Mage | High-Elf, Gnome, Elf, Hobbit | INT +6/+5/+5/+5. Hobbit has the lowest HP. |
| Druid | Dwarf, Dúnadan, Hobbit, Gnome | WIS +5/+5/+4/+3. Dwarf blind-proof. |
| Priest | Dwarf, Dúnadan, Hobbit | WIS +5/+5/+4. |
| Necromancer | High-Elf, Gnome, Elf, Dúnadan | Half-Orc's resist dark is redundant (Unlight already resists dark). |
| Paladin | Dúnadan, Half-Troll, Dwarf, High-Elf | |
| Rogue | Hobbit, Kobold, High-Elf, Gnome | Stealth and DEX; Hobbit DEX +6 total. |
| Ranger | High-Elf, Elf, Hobbit, Dúnadan | Shooting +25/+15/+20/+10. |
| Blackguard | Half-Troll, Dúnadan, Dwarf, Half-Orc | Half-Troll Regeneration (×2) exactly cancels the class's `IMPAIR_HP` (×½) for HP regeneration (`player_regen_hp()`). |

### 7.3 Combinations to avoid while learning

* **Half-Troll Mage / Necromancer** — INT −1 total, device −8, save −8.
* **Hobbit / Kobold Warrior or Blackguard** — STR −2/−1 hurts blows; a lot of HP lost
  to the small hit die.
* **Dwarf Mage / Rogue** — INT −3 on an INT class.
* **High-Elf anything** if you are impatient — 145% experience is felt most in the
  first 15 levels.

---

## 8. Quick reference card

| | War | Mag | Dru | Pri | Nec | Pal | Rog | Ran | BG |
|---|---|---|---|---|---|---|---|---|---|
| Hit die (class part) | 9 | 0 | 2 | 2 | 2 | 6 | 4 | 5 | 8 |
| Max blows | 6 | 4 | 4 | 4 | 4 | 5 | 5 | 5 | 5 |
| Min weapon weight | 3 lb | 4 lb | 3.5 lb | 3.5 lb | 3.5 lb | 3 lb | 2 lb | 3.5 lb | 10 lb |
| Casting stat | — | INT | WIS | WIS | INT | WIS | INT | WIS | INT |
| First spell | — | 1 | 1 | 1 | 1 | 1 | 5 | 3 | 1 |
| Armour weight before SP loss | — | 30 lb | 35 lb | 35 lb | 30 lb | 40 lb | 35 lb | 40 lb | 60 lb |
| Choose spells | — | yes | yes | no | yes | no | yes | yes | yes |
| Min fail can go below 5% | — | yes | yes | yes | yes | no | no | no | no |
| Teleport-type spells | — | Phase, Teleport Self, Tele Other, Tele Level, Dimension Door | — | Portal (dungeon book) | Shadow Shift | — | Phase, Teleport Self, Tele Other, Tele Level | — | — |
| Self-heal spells | — | — | Rapid Regeneration (dungeon) | Minor Healing, Healing, Holy Word | Vampire Strike/Form | Minor Healing, Healing | — | — | casting heals (combat regen) |
| Special | Bravery 30, shield bash | Beams | Shapes, charm | Blessed weapons | Unlight, evil | Bless. weapons, shield bash | Steal, best throw | Extra shots | Combat regen, bloodlust, shield bash |

# 5. Elements and Status Effects

Everything that hurts you other than plain physical damage: the 25 elemental projections and
how resistance, immunity and vulnerability change them; breath damage and its caps; every timed
effect, what prevents it and what cures it; experience and stat drain; monster melee effects;
curses; traps; and a resistance checklist by dungeon level derived from `lib/gamedata/monster.txt`.
All numbers were checked against the game source (angband master `e473720ed`, 4.2.6+).

**Contents**

1. [How an elemental hit is resolved](#51-how-an-elemental-hit-is-resolved)
2. [Element summary table](#52-element-summary-table)
3. [Element by element](#53-element-by-element)
4. [Breath damage, caps and range](#54-breath-damage-caps-and-range)
5. [Timed effects](#55-timed-effects)
6. [Poison, cuts and stun in detail](#56-poison-cuts-and-stun-in-detail)
7. [Experience drain and hold life](#57-experience-drain-and-hold-life)
8. [Monster melee blow effects](#58-monster-melee-blow-effects)
9. [Monster status spells and saving throws](#59-monster-status-spells-and-saving-throws)
10. [Inventory and equipment damage](#510-inventory-and-equipment-damage)
11. [Stat drain, sustains and restoration](#511-stat-drain-sustains-and-restoration)
12. [Curses](#512-curses)
13. [Traps](#513-traps)
14. [Resistance checklist by depth](#514-resistance-checklist-by-depth)
15. [Quick rules for the borg](#515-quick-rules-for-the-borg)
16. [Things to try (untested)](#things-to-try-untested)

Full generated tables of curses, traps and timed effects are in
[Appendix A10](appendices/A10-curses-traps-timed-effects.md); monster spells in
[Appendix A3](appendices/A3-monster-spells.md). Where resistances come from is covered in
[Chapter 6](06-equipment.md) and [Appendix A5](appendices/A5-ego-items.md).

---

## 5.1 How an elemental hit is resolved

The official manual ([attack](https://angband.readthedocs.io/en/latest/attack.html)) lists the
resistances; the engine does the following (**FACT**, `src/project-player.c:project_p()`,
`adjust_dam()`):

1. **Damage at your grid.** Balls and breaths lose strength with distance from their centre or
   source (section 5.4). Bolts do full damage.
2. **Resistance level.** Each element has a resistance level `res_level`:
   - -1 = vulnerable, 0 = none, 1 = resist, 2 = resist twice, 3 = immune.
   - Equipment and race give at most the best single value (resist 1 or immune 3), never more.
   - A **temporary resist** (`TMD_OPP_ACID/ELEC/FIRE/COLD/POIS`, potions, spells, mushrooms)
     adds +1 if the level is below 2. So permanent resist + temporary resist = level 2.
   - Vulnerability (a curse such as "burning up", or a race flag) subtracts 1 unless you are
     immune. Vulnerable + permanent resist = 0; vulnerable + temporary resist = 0;
     vulnerable + both = 1.
   - Ice uses your cold resistance.
3. **Adjust damage** (`adjust_dam()`):
   - immune (3): 0 damage;
   - acid only: a random armour slot is chosen; if it holds armour whose `ac + to_a > 0`, the
     damage is **halved** (and that armour loses 1 to_a unless it ignores acid). This happens
     before resistance;
   - vulnerable (-1): x 4/3;
   - otherwise multiply by `numerator / denominator` once per resist level. Base elements
     (acid, lightning, fire, cold, poison, and ice) use 1/3. High elements (light, dark, sound,
     shards, nexus, nether, chaos, disenchantment) use `6 / (8 + 1d4)`, a random fraction between
     1/2 and 2/3, re-rolled each hit (average 0.58).
   - Water, gravity, inertia, force, time, plasma, meteor, missile, mana, holy orb and arrows have
     no resistance at all.
4. **Flat damage reduction** (`DAM_RED` modifiers, Invulnerability) is applied to the HP loss
   only.
5. **Side effects** (blindness, stun, drains, teleport, inventory destruction...) are then run
   by the element's handler, using the damage **after** resistance (step 3) but before flat
   reduction. Resisting therefore also shrinks the chance and size of side effects, and most
   handlers skip their side effects entirely if you resist.

Resulting fractions:

| Resist level | Base element (1/3 per level) | High element (6/(8+1d4)) |
|---|---|---|
| vulnerable | 133% | 133% |
| none | 100% | 100% |
| resist | 33.3% | 50-66.7% (avg 58%) |
| resist x2 (perm + temp) | 11.1% | only possible for base elements |
| immune | 0 | (no high-element immunity for players) |

Self-inflicted damage (your own ball hitting you, when allowed) is divided by 10.

## 5.2 Element summary table

Breath divisor and cap from `lib/gamedata/projection.txt` (breath damage = current monster HP /
divisor, capped). "Worst breath" is the largest breath any monster in `monster.txt` can produce
(at average HP for uniques, ~98th-percentile HP for others), before resistance.

| Element | Resist / effect of resist | Breath = HP / | Cap | Worst breath (who) | Unresisted side effects |
|---|---|---|---|---|---|
| Acid | rAcid 1/3, temp x2 1/9, immunity 0; armour halves | 3 | 1600 | 1600 (great bile wyrm, Ancalagon) | destroys acid-hating pack items, damages armour |
| Lightning | rElec 1/3, 1/9, 0 | 3 | 1600 | 1600 (great storm wyrm) | destroys rings, wands, rods |
| Fire | rFire 1/3, 1/9, 0 | 3 | 1600 | 1600 (great hell wyrm, Glaurung) | destroys scrolls, books, arrows, staffs...; power 80+: STR drain, blind, poison |
| Cold | rCold 1/3, 1/9, 0 | 3 | 1600 | 1600 (great ice wyrm) | destroys potions and flasks; power 80+: DEX drain, exp drain |
| Poison | rPois 1/3, 1/9 (no immunity source in data) | 3 | 800 | 800 (AMHD, Shelob, Glaurung) | poisoned 10+1d(dam); power 60+: extra acid damage, CON drain |
| Light | rLight 1/2-2/3 | 6 | 500 | 500 (The Phoenix) | blind 2+1d5; dam > 300: confused |
| Dark | rDark 1/2-2/3 | 6 | 500 | 500 (Shelob) | blind 2+1d5; power 70+: exp drain, slow, amnesia |
| Sound | rSound 1/2-2/3 | 6 | 500 | 500 (great wyrm of thunder) | stun 5+1d(dam/3) max 35; dam > 300: confused |
| Shards | rShards 1/2-2/3 | 6 | 500 | 500 (gelugon) | cut 1d(dam) |
| Nexus | rNexus 1/2-2/3 | 6 | 400 | 400 (dracolisk) | stat scramble (save), teleport-to / teleport level (save) / teleport |
| Nether | rNether 1/2-2/3 | 6 | 600 | 600 (dracolich) | exp drain 200 + 2%; power 80+: mana loss, lose a turn |
| Chaos | rChaos 1/2-2/3 | 6 | 600 | 600 (great wyrm of chaos) | hallucinate, confuse 10-29, exp drain 3% |
| Disenchantment | rDisen 1/2-2/3 | 6 | 500 | 500 (Ossë) | -1 to-hit/to-dam/AC on a random equipped item |
| Water | none | 6 | (no breathers) | BA_WATE up to 325 | confuse 5+1d5, stun 1d40 |
| Ice | uses rCold (1/3) | 6 | (no breathers) | bolts/storms only | cold inventory damage, cut 5d8 (rShards blocks), stun 1d15 |
| Gravity | none | 3 | 200 | 200 (Kavlax, aether hound, sky dragon) | short teleport, slow 4-7, stun 5+1d(dam/3) |
| Inertia | none | 6 | 200 | 200 (aether hound) | slow 4-7 (Free Action does not help) |
| Force | none | 6 | 200 | 200 (Baphomet, aether hound, sky dragon) | stun 1d20, knocked back 3+dam/20 grids |
| Time | none | 3 | 150 | 150 (hound of Tindalos, aether hound, great wyrm of annihilation) | exp drain or stat drains, ignoring sustains and hold life |
| Plasma | none | 6 | 150 | 150 (The Phoenix, aether hound, sky dragon) | stun 5+1d(3dam/4) max 35 |
| Meteor, Missile, Arrow | none | 6 | n/a | spells only | damage only |
| Mana | none | 3 | 1600 | no breathers; BA_MANA up to 600 (Morgoth) | damage only |
| Holy orb | none (evil players are vulnerable) | 6 | n/a | BA_HOLY up to 70 | damage only |

Protection flags that block side effects even without the resist: PROT_STUN blocks the stun
from sound, gravity, plasma, water, ice and force; PROT_CONF blocks all confusion; PROT_BLIND
blocks all blindness; HOLD_LIFE blocks the exp drains of nether (and its other side effects),
chaos, dark and cold, but not time or Black Breath; sustains block the stat drains of fire, cold
and poison, but not time.

## 5.3 Element by element

All FACT from `src/project-player.c` unless stated. "dam" below means damage after resistance.

### Acid

- Damage: halved first if the randomly chosen armour slot has armour with positive total AC
  (`src/obj-gear.c:minus_ac()`), which also costs that item 1 point of to_a unless it has
  IGNORE_ACID. Then resistance.
- Unless immune: each non-equipped, non-artifact pack item that hates acid is destroyed with
  chance `min(5 x dam, 300)` per 10000 per item (weapons and armour in the pack are damaged
  -1 instead of destroyed). Acid-hating bases (`lib/gamedata/object_base.txt`): chests, arrows,
  bolts, bows, hafted, polearms, swords, boots, gloves, helms, crowns, shields, cloaks, soft and
  hard armour, staffs, scrolls.
- **ADVICE**: rAcid also protects your armour's to_a in the long run; acid breathers (black
  dragons, AMHD) chew through enchantments.

### Lightning

- Destroys rings, wands and rods in the pack (rods at 1/4 the rate), same `min(5 x dam, 300)`
  rate. Immunity prevents it.

### Fire

- Destroys arrows, bows, hafted and polearm weapons (damaged), boots, gloves, cloaks, soft armour
  (damaged), lights, staffs, scrolls, and all four kinds of spell book.
- From a monster with spell power 80+ (or any POWERFUL breather), each of three rolls succeeds
  if `randint0(dam) > 500`: STR drain (sustain blocks), blindness `1d(dam/100)` (PROT_BLIND
  blocks), poison `1d(dam/10)` (rPois blocks). An unresisted 1600 fire breath succeeds each roll
  69% of the time; resisted (533) only 6%; double-resisted never.
- Lava terrain deals fire damage each turn and also burns inventory
  (`src/player-util.c:player_take_terrain_damage()`).

### Cold

- Destroys potions and flasks of oil.
- Power 80+: if `randint0(dam) > 500`: DEX drain (sustain blocks); exp drain equal to `dam`
  (HOLD_LIFE blocks).

### Poison

- Adds `10 + 1d(dam)` turns of poison unless you resist poison or have `TMD_OPP_POIS`.
- Power 60+: if `randint0(dam) > 200`: an extra acid hit of `dam/5` (through acid resistance, and
  acid inventory damage at `dam/5` per 10000) and, separately, CON drain (sustain blocks).
- No item or race in the data gives poison *immunity*; best is resist + temporary resist (1/9).

### Light

- Unless rLight: blind `2 + 1d5` (PROT_BLIND blocks); if dam > 300 also confused
  `2 + 1d(dam/100)` (PROT_CONF blocks).

### Dark

- Unless rDark (necromancers have it intrinsically in the dungeon): blind `2 + 1d5`.
- Power 70+, each separately: exp drain = dam if `randint0(dam) > 100` (HOLD_LIFE blocks); slow
  `dam/100` turns if `> 200` (**no protection**, not even Free Action, because it bypasses the
  check); amnesia `dam/100` if `> 300` (no protection).
- Weak darkness (`DARK_WEAK`, from the DARKNESS spell and some traps) just blinds `3 + 1d5`
  unless rDark.

### Sound

- Unless rSound: stun `5 + 1d(dam/3)`, at most 35 per hit (PROT_STUN blocks); dam > 300:
  confused `2 + 1d(dam/100)` (PROT_CONF blocks).
- Potions in the pack hate sound but this handler does not call inventory damage; floor items
  can still shatter.

### Shards

- Unless rShards: cut `1d(dam)` (no protection flag blocks this cut).

### Nexus

- Unless rNexus: saving throw or `TMD_SCRAMBLE` for 20-39 turns (stats randomly swapped, swapped
  back when it ends); then, if a monster is the source, 1/3 teleport to that monster; otherwise
  1/4 teleport level (separate saving throw) or else a 200-grid teleport. From a monster:
  1/3 teleport-to, 1/6 teleport level, 1/2 long teleport.
- **ADVICE**: nexus hounds (DL 27) and nexus vortices can drop you into a pack or yank you up a
  level; carry Word of Recall/Deep Descent knowledge and do not fight them near dangerous
  uniques without rNexus.

### Nether

- Unless rNether **or HOLD_LIFE**: exp drain `200 + 2% of exp`; power 80+: SP loss of `dam/10`
  if `randint0(dam) > 100`, and your energy set to 0 (you lose your next action) if
  `randint0(dam) > 200`.
- Necromancers resist nether (EVIL player flag) but are vulnerable to holy orb.

### Chaos

- Unless rChaos: hallucination `1d10` turns (no other protection), confusion `10 + 0..19`
  (PROT_CONF blocks), exp drain `3% of exp` (HOLD_LIFE blocks).

### Disenchantment

- Unless rDisen: one random equipment slot (not rings, amulet or light) loses 1 to-hit and
  to-dam (weapons/launchers) or 1 to_a (armour), plus another point 20% of the time if the value
  is above 5. Artifacts resist 60% of the time.

### Water, ice, gravity, inertia, force, plasma

- Water: confusion `5 + 1d5` (PROT_CONF), stun `1d40` (PROT_STUN). No resistance.
- Ice: damage reduced by cold resistance; cold inventory damage unless cold-immune; cut `5d8`
  unless rShards; stun `1d15` (PROT_STUN).
- Gravity: if `randint1(127) > player level`, short teleport (range 5); slow `4 + 0..3` turns
  (no protection); stun `5 + 1d(dam/3)` max 35 (PROT_STUN).
- Inertia: slow `4 + 0..3` (no protection).
- Force: stun `1d20` (PROT_STUN), thrown back `3 + dam/20` grids.
- Plasma: stun `5 + 1d(3 x dam/4)` max 35 (PROT_STUN).

### Time

- No resistance. Each hit: 50% exp drain `100 + 2% of exp`; else 80% drain two random stats; else
  drain all five stats. These call `player_stat_dec()` / `player_exp_lose()` directly, so
  **sustains and HOLD_LIFE do not help**. The drains are temporary (restored by level gain or
  restore effects).

### Mana, meteor, missile, arrow, holy orb

- Pure damage, no resistance (holy orb is x 4/3 against necromancers).

## 5.4 Breath damage, caps and range

**FACT** (`src/mon-spell.c:breath_dam()`, `src/effect-handler-attack.c:effect_handler_BREATH()`):

```
breath damage = min(cap, CURRENT monster HP / divisor)
```

- Current HP: a wounded dragon breathes for less. Uniques have fixed HP; other monsters roll HP
  as a normal distribution around the listed average with standard deviation ~avg/8
  (`src/mon-make.c:mon_hp()`), so individual breathers can be 25% above average.
- All `BR_xxx` spells are 30-degree arcs. Damage stays at full strength up to 7 grids from the
  breather (11 grids if the monster is POWERFUL), then falls as `8/(d+1)` of full
  (`12/(d+1)` for POWERFUL): at 9 grids a normal breath does 80%, at 15 grids 50%.
- Ball spells (radius 2) do full damage at the centre, 1/2 at distance 1, 1/3 at distance 2.
- Breaths, balls and bolts never miss and have no saving throw.
- Confused monsters breathe in a random direction 40% of the time per confusion level.

Damage after resistance, worst case:

| Element | Max breath | Resist | Resist + temporary | Immune |
|---|---|---|---|---|
| Acid / lightning / fire / cold | 1600 | 533 (acid with armour 266) | 177 | 0 |
| Poison | 800 | 266 | 88 | n/a |
| Nether, chaos | 600 | 300-400 | n/a | n/a |
| Light, dark, sound, shards, disenchantment | 500 | 250-333 | n/a | n/a |
| Nexus | 400 | 200-266 | n/a | n/a |
| Gravity, inertia, force | 200 | 200 | - | - |
| Time, plasma | 150 | 150 | - | - |

**ADVICE.** Against the great wyrms (DL 70+) a single base-element breath is 1400-1600: you
must have the resist *and* the matching temporary resist active (177 damage) or immunity.
Temporary resists last `50 + 1d10` turns from potions; drink them before engaging, not after
the first breath.

## 5.5 Timed effects

The full list is in `lib/gamedata/player_timed.txt`. Timed effects count down by 1 per 10 game
turns (CUT, POISONED and STUN by `adj_con_fix + 1`, FOOD separately); see
[Chapter 1](01-core-mechanics.md#11-time-game-turns-player-turns-and-energy). An effect "fails"
(is not applied) if the listed protection is present **and** the source checks protection;
direct game code sometimes bypasses the check (noted).

### Harmful effects

| Effect | Grades (value) | What it does (FACT, `calc_bonuses()` etc.) | Prevented by | Cured by |
|---|---|---|---|---|
| Slow (`SLOW`) | 1+ | -10 speed | FREE_ACT (monster SLOW spell, traps that check); **not** for inertia, gravity, dark side effect, slow darts (`TIMED_INC_NO_RES`), Sprint's after-effect | staff of Holiness only; otherwise wait |
| Blind (`BLIND`) | 1+ | cannot read, cast, see monsters/map; disarm and lock picking /10; no searching | PROT_BLIND (Dwarves have it) | CLW, CSW, CCW, Healing, \*Healing\*, Life, True Seeing, Second Sight mushroom, Fast Recovery mushroom, rod/staff of Curing, Holiness |
| Paralyzed (`PARALYZED`) | 1+ | you lose all turns; non-stacking: cannot be extended while active | FREE_ACT (Gnomes have it); saving throw first for melee and HOLD spells | wait it out (no cure item) |
| Confused (`CONFUSED`) | 1+ | random direction 75% when moving/aiming; cannot read or cast; device -25%; disarm /10 | PROT_CONF, `TMD_OPP_CONF` (Clear Mind mushroom, blackguard Grim Purpose) | CLW (-20 turns), CSW, CCW, Healing, \*Healing\*, Life, Clear Mind, rod/staff of Curing, Holiness |
| Afraid (`AFRAID`) | 1+ | cannot melee; -20 to-hit; +8 AC; spell fail +20; device -5% | PROT_FEAR, Hero, Berserk, Bold; warriors from level 30 (BRAVERY_30); saving throw for SCARE and TERRIFY | Boldness, Heroism, Berserk Strength, Clear Mind, Holiness, Heroism/Berserk spells |
| Terror (`TERROR`) | 1+ | afraid + 10 speed (from mushroom of Terror) | - | Holiness |
| Hallucination (`IMAGE`) | 1+ | monsters/objects shown wrongly; no searching; disarm /10; device -20% | rChaos (resist, not a flag) | Clear Mind, Life, Holiness |
| Poisoned (`POISONED`) | 1+ | 1 damage per tick (after damage reduction); no HP regen; device -5% | rPois or `TMD_OPP_POIS` | CCW, Neutralize Poison, Resist Poison, Healing, \*Healing\*, Life, Waybread, Purging, Salt Water, Fast Recovery, rod/staff of Curing, Holiness, druid Cure Poison, Herbal Curing |
| Cut (`CUT`) | Graze 1-10, Light 11-25, Bad 26-50, Nasty 51-100, Severe 101-200, Deep Gash 201-1000, Mortal Wound 1001+ | damage per tick: 1 up to Nasty, 2 Severe, 3 Deep Gash/Mortal; no regen; Mortal Wound never heals by itself | (ROCK shapechange only) | CLW (-20), CSW, CCW, Healing, \*Healing\*, Life, Fast Recovery, rod of Healing, rod/staff of Curing, Holiness, priest/paladin Minor Healing (reduces) |
| Stun (`STUN`) | Stun 1-50, Heavy Stun 51-150, Knocked Out 151+ | Stun: -5 hit/dam, device -10%, spell fail +15; Heavy: -20/-20, device -20%, fail +25; Knocked Out: no turns at all; any stun stops regen | PROT_STUN (except rock-fall trap) | CCW, Healing, \*Healing\*, Life, Fast Recovery, rod of Healing, rod/staff of Curing, Holiness, Minor Healing (reduces) |
| Amnesia (`AMNESIA`) | 1+ | cannot read; spell fail = 50 + fail/2; device -20% | saving throw vs FORGET only | CCW, Healing, \*Healing\*, Life, Holiness, priest/paladin Healing |
| Scrambled (`SCRAMBLE`) | 1+ | stats randomly swapped until it ends | rNexus; saving throw | wait (unscrambles on expiry) |
| Black Breath (`BLACKBREATH`) | 1-10 | each tick: 50% CON drain, 50% STR drain, 50% exp drain (100 + 2%); all ignore sustains/hold life | none | Herbal Curing (druid L40, ranger L9); otherwise lasts at most 10 ticks |
| Hunger (`FOOD`) | see [1.16](01-core-mechanics.md#116-food-and-digestion) | penalties, fainting, starvation | SLOW_DIGEST slows | food |
| Bloodlust (`BLOODLUST`) | 1-50 | +to-dam, +blows; side effects as it fades | - | wait |

### Beneficial effects (for reference)

| Effect | Does |
|---|---|
| FAST | +10 speed (not cumulative with Sprint) |
| SPRINT | +10 speed; when it ends you are slowed for 100 turns, unresistable |
| HERO | +12 to-hit, immune to fear, device +5% |
| SHERO (Berserk) | +75 melee skill, -10 AC, immune to fear, device -10% |
| BLESSED | +5 AC, +10 to-hit, device +5% |
| SHIELD | +50 AC |
| STONESKIN | +40 AC, -5 speed |
| INVULN | +100 AC; any damage below 9000 is reduced to 0 |
| PROTEVIL | repels melee of evil monsters whose level <= yours (chance `randint0(100) + clvl > 50`) |
| OPP_ACID/ELEC/FIRE/COLD/POIS | +1 resist level (up to 2): double resist with a permanent one |
| OPP_CONF | protection from confusion |
| BOLD | protection from fear |
| FREE_ACT | free action |
| SINVIS, TELEPATHY, SINFRA | see invisible, ESP, +5 infravision |
| STEALTH | +10 stealth |
| COVERTRACKS | noise spreads 4x slower, no scent, monsters see you only within 5 grids |
| TRAPSAFE | immune to traps |
| FASTCAST | spells cost 75 energy (cancelled by any stun) |
| HEAL | 30 HP per tick, burns 800 food per tick (ends when hungry) |
| ATT_ACID/ELEC/FIRE/COLD/POIS | temporary x3 brand on melee |
| ATT_EVIL, ATT_DEMON | temporary slay evil x2, slay demon x5 |
| ATT_CONF | next melee hit confuses |
| ATT_VAMP | melee heals you by damage dealt to living monsters; exempts gorged speed loss |
| ATT_RUN | after a kill, teleport 20 grids |
| POWERSHOT | next sharp missile pierces |
| TAUNT | monsters close in and cast less (spell chance halved) |
| COMMAND | you control a monster |

## 5.6 Poison, cuts and stun in detail

**FACT** (`src/game-world.c:process_world()`, `decrease_timeouts()`):

- Per tick (10 game turns): poison deals 1 damage; cuts deal 1 (Graze to Nasty), 2 (Severe) or
  3 (Deep Gash, Mortal Wound); each goes through flat damage reduction.
- Countdown per tick: `adj_con_fix[CON] + 1` (1 at CON <= 13, 2 at 14-17, 3 at 18-18/49, 4 at
  18/50-18/99, 5 at 18/100+...). Mortal Wounds do not count down at all.
- While poisoned, cut, stunned or paralysed you regenerate **no HP**.
- Stun accumulates: a 35-turn sound stun on top of a 20-turn stun gives 55 = Heavy Stun. Above
  150 you are Knocked Out and lose turns until it drops below 151. With CON 18/100 that is
  5 per tick of recovery.
- Monster melee cuts and stuns occur only on near-maximum damage rolls
  ([1.5](01-core-mechanics.md#15-monster-melee-against-the-player)); shards breath cuts
  `1d(dam)` so an unresisted 500 shard breath can inflict a Deep Gash (up to 500).

**ADVICE.** Stun is the most dangerous "minor" status: Heavy Stun takes 20 off to-hit and
to-dam and 25% off spell success, and sound/plasma/gravity hounds stack it fast. Get PROT_STUN
(crown of Serenity) or rSound for the mid game, and always carry CCW or Curing.

## 5.7 Experience drain and hold life

**FACT.** `life_drain_percent` = 2 (`lib/gamedata/constants.txt`). Drains reduce current
experience only (max exp is kept) and are restored by Restore Life Levels, Life, rod of
Restoration, priest Remembrance, druid Revitalize and blackguard Unholy Reprieve, or simply by
earning experience (each gain while drained also raises max exp by 10% of the gain).

| Source | Amount | HOLD_LIFE effect |
|---|---|---|
| Blow EXP_10 | 10d6 + 2% of exp | 95% no drain, else 1/10 of the amount |
| Blow EXP_20 | 20d6 + 2% | 90% no drain, else 1/10 |
| Blow EXP_40 | 40d6 + 2% | 75% no drain, else 1/10 |
| Blow EXP_80 | 80d6 + 2% | 50% no drain, else 1/10 |
| Nether (unresisted) | 200 + 2% | fully blocked (also blocks nether's mana/turn loss) |
| Chaos (unresisted) | 3% | blocked |
| Dark (power 70+, randint0(dam) > 100) | = dam | blocked |
| Cold (power 80+, randint0(dam) > 500) | = dam | blocked |
| Time | 100 + 2% (50% of hits) | **not** blocked |
| Black Breath | 100 + 2% (50% per tick) | **not** blocked |
| DRAIN_EXP item flag | (10d6 + 2%)/10, 1 tick in 10 | not blocked |

Hold Life sources (`lib/gamedata/`): Hobbit race; amulets of Sustenance and Devotion; egos of
Permanence and Preservation; weapons of Morgul (which also aggravate and drain exp); some
artifacts. See [Appendix A4](appendices/A4-artifacts.md).

## 5.8 Monster melee blow effects

**FACT** (`lib/gamedata/blow_effects.txt`, `src/mon-blows.c`). "Power" is added to the
monster's to-hit (`3 x level + power`). Damage of non-HURT effects is **not** reduced by AC
([1.4](01-core-mechanics.md#14-armour-class-and-damage-reduction)).

| Effect | Power | Effect on you | Protection | Earliest monsters (DL) |
|---|---|---|---|---|
| HURT | 40 | damage reduced by AC | AC | everywhere |
| POISON | 20 | max(physical, poison-resisted) damage; poison 5+1d(level) | rPois, OPP_POIS | white worm mass 1, white jelly 2 |
| ACID / ELEC / FIRE / COLD | 20 / 40 / 40 / 40 | max(physical at AC+50, resisted elemental) damage; inventory damage `min(5 x elemental dam, 300)` | resist/immunity (physical part still applies for biting/clawing methods) | centipedes 1-3, salamander 2 |
| BLIND | 0 | blind 10+1d(level) | PROT_BLIND | bloodshot eye 7, king cobra 9 |
| CONFUSE | 20 | confused 3+1d(level) | PROT_CONF | grey mushroom patch 1, umber hulk 16 |
| TERRIFY | 0 | afraid 3+1d(level) | saving throw, then PROT_FEAR | yellow mushroom patch 2, poltergeist 3 |
| PARALYZE | 0 | paralysed 3+1d(level) (min 1 damage if already paralysed) | saving throw, then FREE_ACT | floating eye 1, homunculus 15, carrion crawler 25, ghoul 26, ghast 30 |
| LOSE_STR/INT/WIS/DEX/CON | 0 | drain the stat | matching sustain | radiation eye 3 (STR), yellow worm 3 (DEX), rot jelly 5 (WIS), purple mushroom 6 (CON), mind flayer 28 (INT) |
| LOSE_ALL | 0 | drains all five | each sustain | storm of Unmagic 95, Morgoth 100 |
| EXP_10/20/40/80 | 20 | experience drain (5.7) | HOLD_LIFE (partial) | grape jelly 12, banshee 24, vampire bat 24, emperor wight 37 |
| DISENCHANT | 10 | disenchant an equipped item | rDisen | disenchanter eye 5, disenchanter mold 10 |
| DRAIN_CHARGES | 10 | drains charges of a random wand or staff in the pack (rods are immune), heals the monster | none | lich 34, gauth 36 |
| EAT_GOLD | 0 | steals 10% of gold + 1d25 (more if rich), monster teleports | `randint0(100) < adj_dex_safe + clvl` and not paralysed | street urchin 0, cutpurse 2, Sméagol 3 |
| EAT_ITEM | 0 | steals a non-artifact pack item | same DEX/level check | squint-eyed rogue 0, master thief 34 |
| EAT_FOOD | 0 | eats a food item | none | blubbering icky thing 2 |
| EAT_LIGHT | 0 | drains 250+1d250 turns of fuel | artifact/no-fuel light | silver jelly 3, silver mouse 4 |
| HALLU | 0 | hallucinate 3+1d(level/2) | rChaos | magic mushroom patch 15 |
| SHATTER | 60 | AC-reduced damage; earthquake radius dam/12 if dam > 23; may throw you back | AC | Quaker 43 |
| BLACK_BREATH | 0 | damage; 1 in 5 adds Black Breath (dam/10 ticks) | none | the Nazgûl 77-90 |

A blow that would both cut and stun picks one at random.

## 5.9 Monster status spells and saving throws

**FACT** (`lib/gamedata/monster_spell.txt`, `src/mon-spell.c:do_mon_spell()`): spells with a
`message-save` give you a saving throw (`randint0(100) < skill_save`); if the save fails, the
effect still has to pass the protection check of the timed effect.

| Spell | Effect | Save? | Then blocked by | First casters (DL) |
|---|---|---|---|---|
| SCARE | fear 3+1d4 | yes | PROT_FEAR | acolyte 2 |
| BLIND | blind 11+1d4 | yes | PROT_BLIND | apprentice 2 |
| CONF | confused 3+1d4 | yes | PROT_CONF | apprentice 2, kobold shaman 3 |
| SLOW | slow 3+1d4 | yes | FREE_ACT | tamer 4, Wormtongue 8 |
| HOLD | paralysed 3+1d4 | yes | FREE_ACT | illusionist 13, druid 13, ghoul 26 |
| FORGET | amnesia 3 | yes | - | mind flayer 28 |
| TELE_LEVEL | teleport level | yes | rNexus; NO_TELEPORT curse | quasit 16 |
| MIND_BLAST | 8d8 + confusion | yes | PROT_CONF (confusion part) | Nár 17 |
| BRAIN_SMASH | 12d15 + slow, confusion, paralysis, blindness | yes | FA, PROT_CONF, PROT_BLIND each | mind flayer 28 |
| WOUND | (2 x power/3)d5 damage, cut at higher power | yes | - | kobold shaman 3 |
| TELE_TO | pulls you next to the monster | no | NO_TELEPORT curse (and no-teleport vault grids) | Orfax 10, tengu 10 |
| TELE_AWAY | teleports you away (range 100) | no | NO_TELEPORT curse | mage 28, Draebor 28 |
| DRAIN_MANA | lose 1d(power/2+1) SP | no | - | silver jelly 3 |
| DARKNESS | darken room, weak-dark blind 3+1d5 | no | rDark, PROT_BLIND | eastern dwarf 7 |
| All bolts, balls, breaths | damage + element side effects | no | resistances | see 5.14 |

## 5.10 Inventory and equipment damage

**FACT** (`src/project-obj.c:inven_damage()`):

- Only acid, lightning, fire and cold (and ice -> cold, poison's venom -> acid, lava -> fire)
  call inventory damage. Equipped items and artifacts are never destroyed this way.
- For each pack stack of a kind that hates the element and does not ignore it: each item in the
  stack is destroyed with probability `cperc / 10000`, `cperc = min(5 x dam, 300)` where dam is
  the damage after resistance (rods: cperc/4). Weapons and armour in the pack instead lose
  1 to-hit/to-dam or 1 to_a with that probability.
- So any hit of 60+ post-resist damage destroys each vulnerable item with 3% probability. A
  single resist does not lower this for big breaths (533 post-resist is still capped at 3%);
  double resist (177) still gives 3%; only immunity (0) or the element missing you saves the
  pack.

| Element | Hated by |
|---|---|
| Acid | chests, arrows, bolts, bows, hafted, polearms, swords, boots, gloves, helms, crowns, shields, cloaks, soft/hard armour, staffs, scrolls |
| Fire | chests, arrows, bows, hafted, polearms, boots, gloves, cloaks, soft armour, lights, staffs, scrolls, all spell books |
| Cold | potions, flasks |
| Lightning | rings, wands, rods |
| (Sound, shards, ice, force) | potions and flasks on the *floor* |

**ADVICE.** Spare spell books are worth carrying for casters from DL 30 on (fire breathers),
and potion stacks should be split between pack and home before fighting cold breathers.

## 5.11 Stat drain, sustains and restoration

**FACT** (`src/player.c:player_stat_dec()`, `src/effect-handler-general.c`):

- A drain lowers the current value: above 18/10 by 10 points, 18/01-18/10 to 18, otherwise by 1
  (minimum 3). The maximum is untouched.
- `EF_DRAIN_STAT` (blows, darts, heat/cold/stench side effects, potions/mushrooms) is blocked by
  the matching `SUST_xxx` and also does the effect's damage. Time and Black Breath bypass
  sustains.
- Permanent loss happens only through `LOSE_RANDOM_STAT` (side effect of the stat-swap potions
  Brawn, Intellect, Contemplation, Nimbleness, Toughness) and through over-exertion when casting
  without enough mana (CON).
- **Restoration (verified):** gaining a character level restores all five stats
  (`src/player.c:adjust_level()`), including re-gaining a drained level. Items: mushroom of Vigor
  (all five), Purging and Draught of the Ents and Sip of Miruvor (STR, CON), every stat potion
  (that stat), Augmentation (all), potion of Life (all), rod of Restoration (all + exp), staff
  of the Magi (INT), \*Enlightenment\* (INT, WIS). Spells: priest Restoration (L35), druid
  Revitalize (L35), blackguard Unholy Reprieve (STR, INT, CON; L34).
- Sustain sources: rings of Strength/Intelligence/Dexterity/Constitution, Bodykeeping (STR, DEX,
  CON), Soulkeeping (STR, INT, WIS), amulets of Wisdom, Sustenance (all + hold life + slow
  digestion), Weaponmastery, Trickery, the Magi; egos (Might, Preservation, Holy Avenger and
  Defender random sustain...); race sustains (Half-Troll STR, Elf/Half-Elf DEX, Dunadan CON).

**ADVICE.** Do not spend money on stat restoration early: the next level-up restores
everything. Late in the game levels come slowly, so carry a Vigor mushroom or rod of
Restoration.

## 5.12 Curses

The official manual ([playing](https://angband.readthedocs.io/en/latest/playing.html)) says that
curses are runes with a power. **FACT** (`lib/gamedata/curse.txt`, `src/obj-curse.c`,
`src/obj-make.c:apply_curse()`, `src/effect-handler-general.c:uncurse_object()`):

- A cursed item carries one or more curses, each with a **power** (generated as
  `randint1(9) + 10 x m_bonus(9, level)`, roughly 1-99, higher deeper). Power 100 means
  permanent (only the rings and amulets of Teleportation have a power-100 teleportation curse).
- Cursed items can be taken off freely (no sticky curses on normal items).
- Some curses are periodic: every `time` turns (while worn) the effect fires.
- **Removing a curse**: Scroll of Remove Curse (strength 20+1d20), Scroll of \*Remove Curse\*
  (50+1d50), Staff of Remove Curse (35+1d30), priest spell Remove Curse (L16; strength
  `clvl + 1d clvl`), paladin Remove Curse (L20; same). You pick one curse on one item. If
  strength >= power, the curse is removed. If not: the first failure makes the item FRAGILE;
  on a FRAGILE item each further failure has a 1 in 4 chance to destroy it ("bang and a flash",
  5d5 damage, artifacts lost), otherwise nothing happens. Power-100 curses cannot be removed.

| Curse | Item types | Effect |
|---|---|---|
| vulnerability | body armour, cloak, shield | AC -50 and aggravation |
| teleportation | helm, crown, amulet, ring | random teleport (range 40) about every 1d100 turns |
| dullness | helm, crown | INT -5, WIS -5 |
| sickliness | helm, crown | STR, DEX, CON -5 |
| enveloping | cloak | -5 to-hit, -5 to-dam, +20 AC |
| irritation | cloak | -15 to-hit, -15 to-dam, aggravation |
| weakness | gloves | STR -10 |
| clumsiness | gloves | DEX -10 |
| slowness | boots | speed -5 |
| annoyance | boots | speed -10, stealth -10, aggravation |
| poison | most items | poisoned 10+1d10 about every 1d500 turns |
| siren | most items | wakes nearby monsters every 100+1d50 turns |
| hallucination | most items | hallucinate 10+1d10 every 1d500 turns |
| paralysis | most items | paralysed 5+1d5 every 1d1000 turns (Free Action blocks) |
| dragon summon | dragon armour | summons a dragon every 1d500 turns |
| demon summon / undead summon | melee weapons, diggers | summons every 1d500 turns |
| impair mana recovery | helm, crown | half mana regeneration |
| impair hitpoint recovery | boots, gloves, shield, cloak, body armour | half HP regeneration |
| cowardice | amulet, ring | permanent fear (cannot melee) |
| stone | amulet, ring | Stoneskin (+40 AC, -5 speed) every 1d1000 turns |
| anti-teleportation | boots, cloak, body armour, amulet, ring | you cannot teleport (escapes fail) |
| treacherous weapon | melee weapons, diggers | the weapon hits you for its damage every 1d200 turns |
| burning up | armour pieces | vulnerable to fire, resist cold |
| chilled to the bone | armour pieces | vulnerable to cold, resist fire |
| steelskin | weapons, helm, shield, armour, light, jewellery | +20 AC, vulnerable to lightning, resist shards |
| air swing | melee weapons, diggers | -20 to-hit |

**ADVICE.** Many cursed items are still worth wearing: steelskin can be an asset for a
character with rElec, enveloping on a caster's cloak, burning up/chilled if you have the other
resist. The deadly ones for a diver are anti-teleportation (no escapes), cowardice (no melee),
annoyance/irritation/vulnerability/siren (aggravation), paralysis without FA, and teleportation
during fights. Never try to uncurse an artifact you cannot afford to lose with a weak scroll:
check strength against the curse power (shown once the rune is known).

## 5.13 Traps

**FACT** (`lib/gamedata/trap.txt`, `src/trap.c:hit_trap()`). A trap triggers when you step on it
(the block-fall trap when you step off). Before effects:

1. Trap immunity (`TRAP_IMMUNE` gear or `TMD_TRAPSAFE`) skips it entirely.
2. `save:` flags (FEATHER for trap doors and pits) negate it.
3. `SAVE_ARMOR` traps (darts) miss if a to-hit 125 roll fails against your AC (AC 50: 27%
   miss; AC 100: 49% miss; AC 188+: 88% miss, the maximum).
4. `SAVE_THROW` traps (mind blast, brain smash) are negated by a saving throw.
Otherwise the effect fires, plus the `-xtra` effect 50% of the time. After triggering, ONETIME
traps vanish and every trap has a 1 in 3 chance to vanish.

Seeing traps: a trap is noticed when its square is in view and your Search skill is at least
its power (`visibility`); see [1.23](01-core-mechanics.md#123-disarming-lock-picking-searching-and-secret-doors).
Disarming uses physical or magical (runes) skill: `chance = skill - depth/5`.

| Trap | Min DL | Visibility | Effect | Avoid with |
|---|---|---|---|---|
| trap door | 2 | 2d10M50 | 2d8 damage, fall to next level | Feather Fall |
| pit | 2 | 90 | 2d6 | Feather Fall |
| spiked pit | 2 | 80 | 2d6; 50%: +2d6 and cut 4d6 | Feather Fall |
| poison pit | 2 | 70 | 2d6; 50%: cut 4d6, poison 8d6 | Feather Fall |
| rune of summon foe | 3 | 0 | summons 1 monster | (magic disarm) |
| rune of summoning | 3 | 0 | summons 2+1d3 monsters | |
| rune of necromancy | 20 | 0 | summons undead | |
| rune of dragonsong | 20 | 0 | summons 1d3 dragons | |
| hellhole | 20 | 0 | summons 1+1d3 demons | |
| teleport rune | 1 | 30+1d30 | teleport | |
| fire trap | 2 | 20 | fire, 4d(DL/2) | rFire |
| acid trap | 2 | 0 | acid, 4d(DL/2) | rAcid |
| slow dart | 2 | M50 | 1d4, slow 20+1d20 (unresistable) | AC |
| STR/DEX/CON loss dart | 6 | 30 | 1d4 + stat drain | AC, sustain |
| blinding gas trap | 2 | 20+1d5 | blind 25+1d50 | PROT_BLIND |
| confusion gas trap | 1 | 20+1d5 | confused 10+1d20 | PROT_CONF |
| poison gas trap | 2 | 20+1d5 | poison 10+1d20 | rPois |
| sleep gas trap | 2 | 20+1d5 | paralysed 5+1d10 | FREE_ACT |
| aggravation trap | 5 | M50 | wakes every monster on the level and hastes those in line of sight | |
| siren | 1 | 60+1d10 | wakes monsters | |
| mine trap | 15 | 10+1d5 | shards, 2 x DL damage, radius 2 | rShards (cut) |
| blast trap | 25 | 0 | light 2d6, sound DL/2, fire DL, force DL (radius 1-2) | rFire, rSound, PROT_STUN |
| mind blasting trap | 20 | 30+1d10 | 8d(DL/10), confusion | save, PROT_CONF |
| brain smashing trap | 40 | 30+1d10 | 10d(DL/5), slow, confusion, paralysis, blindness | save, FA, PROT_CONF, PROT_BLIND |
| rock fall trap | 4 | 40+1d5 | (DL/10+1)d5, stun 2d20 (unresistable), rubble | |
| earthquake trap | 30 | 30+1d5 | (DL/10+1)d5, earthquake radius 5 | |
| block fall trap | 4 | 40+1d5 | granite falls behind you (delayed) | |
| area blast trap | 40 | 20+1d5 | clears walls, force DL radius 2 | PROT_STUN |
| blinding flash trap | 5 | 30+1d5 | light ball (DL/10+2)d8 radius 4 | rLight, PROT_BLIND |
| blinding trap | 5 | 20+1d5 | dark ball (DL/10+2)d8 radius 4, drains 100+1d100 light | rDark |
| mana drain trap | 5 | 50+1d10 | lose 1d(DL/2+1) SP | |
| knife trap | 20 | 20+1d5 | cut 150 (Deep Gash), DL/2 damage | |
| petrifying trap | 5 | 20+1d5 | Stoneskin 20+1d20, stun 20+1d20 | PROT_STUN (stun part) |

("M50" means an m_bonus value up to 50 scaled by trap level, so deeper traps are harder to
see. "Min DL" is the minimum generation depth from the `appear:` line.)

**ADVICE.** Traps are rarely lethal on their own; the dangerous combinations are sleep gas
without FA, trap doors when you need to stay on a level, summoning runes deep, and teleport
runes in a vault. Cast or read trap detection when entering a vault or long corridor; walk
around unknown floor in vaults.

## 5.14 Resistance checklist by depth

**Method.** For every monster in `lib/gamedata/monster.txt` with native depth <= DL, compute the
breath damage `min(cap, HP/divisor)` (uniques at fixed HP, others at average + 2 standard
deviations, ~98th percentile) and the maximum of every damaging spell from its spell power
(`lib/gamedata/monster_spell.txt`). Monsters can appear out of depth (1 in 25 generations is
boosted by up to 10 levels, `lib/gamedata/constants.txt`), so read each row as "what you must
survive a few levels earlier than this".

### Worst single breath by element, monsters native to DL <= N (before resistance)

| DL <= | Acid | Elec | Fire | Cold | Pois | Light | Dark | Sound | Shards | Nexus | Nether | Chaos | Disen | Time | Plasma | Force | Inertia | Gravity |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 5 | - | - | - | 6 | 6 | - | - | - | - | - | - | - | - | - | - | - | - | - |
| 10 | 37 | 37 | 37 | 37 | 37 | 37 | 37 | 18 | - | - | - | - | - | - | - | - | - | - |
| 15 | 48 | 48 | 48 | 48 | 48 | 37 | 37 | 18 | - | - | - | - | - | - | - | - | - | - |
| 20 | 48 | 48 | 67 | 48 | 48 | 37 | 37 | 18 | 14 | - | - | - | - | - | - | - | - | - |
| 30 | 48 | 99 | 110 | 99 | 130 | 37 | 37 | 29 | 14 | 29 | - | - | - | - | - | - | - | - |
| 35 | 117 | 117 | 117 | 117 | 130 | 73 | 73 | 110 | 110 | 29 | 55 | 110 | 110 | - | - | 40 | 40 | 81 |
| 40 | 117 | 292 | 271 | 292 | 292 | 73 | 73 | 110 | 110 | 37 | 55 | 110 | 110 | 74 | 37 | 40 | 40 | 81 |
| 45 | 433 | 433 | 433 | 433 | 421 | 73 | 210 | 216 | 216 | 216 | 55 | 216 | 110 | 74 | 37 | 40 | 40 | 200 |
| 50 | 433 | 626 | 1000 | 626 | 626 | 406 | 406 | 216 | 406 | 216 | 406 | 216 | 110 | 74 | 37 | 40 | 40 | 200 |
| 60 | 1000 | 1000 | 1284 | 1284 | 800 | 500 | 500 | 325 | 406 | 400 | 600 | 216 | 110 | 150 | 150 | 200 | 88 | 200 |
| 70 | 1027 | 1600 | 1400 | 1600 | 800 | 500 | 500 | 325 | 500 | 400 | 600 | 587 | 110 | 150 | 150 | 200 | 88 | 200 |
| 80+ | 1600 | 1600 | 1600 | 1600 | 800 | 500 | 500 | 500 | 500 | 400 | 600 | 600 | 500 | 150 | 150 | 200 | 200 | 200 |

Who sets the numbers: DL 9-11 baby dragons (29-38 average) and baby multi-hued (all five
bases); DL 20 chimaera (fire 53); DL 27-29 young dragons, gorgimaera (fire 87), basilisk
(poison 103 average); DL 27 vibration and nexus hounds (packs); DL 33 the drakes (ethereal:
light/dark, law: sound/shards, chaos: chaos/disen, balance: sound/shards/chaos/disen, shadow:
nether, crystal: shards); DL 35 gravity, inertia and impact hounds (packs); DL 38-39 time
vortex, mature dragons (233 average); DL 41 Kavlax (acid 433, sound/nexus/chaos 216,
gravity 200); DL 45 greater basilisk (dark/nexus 168); DL 50 ancient dragons, death drake
(nether), ethereal dragon (light/dark), great crystal drake (shards), the Balrog of Moria (fire
1000); DL 51 time/nether/plasma hounds; DL 53-55 AMHD (800 each base), dracolisk, dracolich,
Shelob, The Phoenix; DL 62 Smaug (fire 1400); DL 64 chaos hounds; DL 68-75 jabberwock, gelugon,
great wyrms, aether hounds, Ossë, great wyrm of chaos; DL 85 Ancalagon.

**Hound packs multiply this.** Zephyr hounds come in packs of 3-15 (`friends:100:2d7:Same`) and
breathe 1 turn in 5 each. Six gravity hounds (64 average) is up to ~380 damage, slow and
blinks in one turn of bad luck; six time hounds (110 each, +20 speed) can drain levels and stats
repeatedly.

### Worst damaging spell by depth (max roll, before resistance)

| DL <= | Strongest spells (caster, DL) |
|---|---|
| 10 | frost bolt 50 (Wormtongue 8); stinking cloud 32 (blacklock mage 10); wound 30 |
| 20 | fire bolt 78 (dark dwarven lord 20); poison bolt 76 (druid 13); mind blast 64 (Nár 17) |
| 30 | brain smash 180 (mind flayer 28); lightning beam 113 (energy vortex 29); acid ball 96 (Mîm 27); nether bolt 92 (necromancer 25) |
| 40 | fire ball 150 (sorcerer 40); wound 130; acid ball 123; nether bolt 112; plasma bolt 103 |
| 50 | nether ball 296, fire ball 181, mana bolt 172 (Eöl 49); storms (storm giant 49: water 30+16d5, lightning and ice 20+16d5); water ball 157 (Waldern 43) |
| 60 | fire ball 220, water ball 200, wound 200 (Saruman 60); mana bolt 200 |
| 70 | mana storm 425 (Mouth of Sauron 65); nether storm 376 (nightcrawler 69); darkness storm 372 (Tselakus 68); fire 307 (Nan) |
| 80 | mana storm 485, nether storm 408 (Feagwath 77); darkness storm 400 (Ungoliant 75); storms 30+24d5 water (Ossë 72) |
| 99-100 | darkness storm 620, nether storm 620, fire ball 465, wound 430 (Sauron 99); mana storm 600 (Morgoth 100) |

### The checklist

FACT columns state the threat; the "need" column is ADVICE derived from it (a borg should
treat "must" as a hard gate for diving past that level).

| By DL | Threats appearing (FACT) | Need (ADVICE) |
|---|---|---|
| **10** | baby dragon breaths <= 38; floating eyes paralyse on gaze if you melee them (DL 1); invisible ghosts (poltergeist DL 3); thieves; confusion and blindness spells from DL 2 casters; frost bolt 50 (Wormtongue) | light radius 2, CLW potions (cure blind/confusion), Phase Door; never melee floating eyes without Free Action; no resists required |
| **20** | baby multi-hued (5 elements, 48); homunculus/Old Man Willow paralysis (15-19); HOLD from illusionists and druids (13); umber hulk confusion gaze (16); first fire 67 (chimaera); mind blast | **Free Action must** (by DL 15-20); see invisible desirable; rFire/rCold nice; CCW/CSW and !Speed for emergencies |
| **30** | carrion crawlers and ghouls (paralysis packs, 25-26); young dragons ~100; basilisk poison 130 (28); mind flayers (brain smash 180, amnesia, INT/WIS drain; 28); nexus and vibration hound packs (27); vampires and wights (exp drain 24+); invisible ghosts (31) | **FA must; See Invisible must**; all four base resists (rAcid, rElec, rFire, rCold) strongly recommended; rPois or !Resist Poison for basilisks; PROT_STUN or rSound helpful vs vibration hounds; Hold Life useful |
| **40** | mature dragons 233-292 (39); 7-headed hydra fire 271 (40); drakes with sound/shards/chaos/disen/nether ~110 (33); gravity/inertia/impact hound packs (35); time vortex 74 unresistable; emperor wights EXP_80 (37); liches drain charges and DEX (34) | **all four base resists must; rPois must by DL 40** (292 poison); PROT_CONF strongly recommended (chaos breaths, hydras); PROT_BLIND or rLight/rDark; Hold Life recommended; speed +5..+10; 400+ HP |
| **50** | Kavlax (41), ancient dragons 500-626 (50), Balrog of Moria fire 1000 (50), death drake nether 406, ethereal dragon light/dark 406, great crystal drake shards 406; Eöl nether storm 296 (49); greater basilisk (45); undead beholders (45) | base resists + rPois + **double resist via potions for big dragons/Balrog**; at least two of rNether, rChaos, rDark, rShards, rSound; PROT_CONF, PROT_BLIND, FA, SI, HL; +10 speed; 600+ HP |
| **70** | AMHD 800 each base (53), dracolich nether 600, dracolisk nexus 400 (55), Shelob dark 500, Phoenix light 500 + plasma 150, time/plasma/nether hounds (51), chaos hounds (64, 155 avg), jabberwock chaos 587 (68), gelugon shards 500 + cold 1026 (69); mana storm 425 | rNether, rChaos (also protects vs hallucination), rDisen (gear protection), rNexus, rSound, rShards, rLight, rDark: aim for all; PROT_STUN or rSound; +20 speed; 1000+ HP to survive one unresisted high breath plus a follow-up |
| **90** | great wyrms 1400-1600 base breaths (70-75), Ossë disenchantment 500, aether hounds (74) with 200-damage force/inertia/gravity and time/plasma 150 each, great wyrm of chaos 600 chaos + 500 disen, Ancalagon (85), Feagwath mana storm 485 | everything above **plus** immunity or perm+temp resist for the element of any great wyrm you fight; high HP for unresistable damage (aether hound pack: several 150-200 breaths per turn); +20..+30 speed |
| **99** | Sauron: darkness and nether storms 620, fire 465, wound 430, mana; Morgoth (100): mana storm 600, melee; Vampire-Sauron poison 272 | full high-resist set; rDark and rNether drop the 620 storms to 310-413; HP comfortably above 800 for unresistable mana storms; see [Chapter 12](12-game-plan-by-phase.md) |

Where to find resists: see [Chapter 6](06-equipment.md). Quick pointers from
`lib/gamedata/object.txt` / `ego_item.txt`: Free Action (ring L20, gloves and boots "of Free
Action", Gnome race), See Invisible (ring L30, helm of Seeing, High-Elf race), rPois (ring L40,
green/multi-hued/power dragon armour, amulet of Trickery, Kobold race), PROT_CONF (crown of
Serenity, power dragon armour, temporary via Clear Mind mushroom), PROT_BLIND (Dwarf race, helm
of Seeing, crown of Night and Day, amulet of the Magi), PROT_STUN (crown of Serenity), Hold Life
(Hobbit race, amulets of Sustenance and Devotion, Permanence and Preservation egos), base resists
(armour/shields of Resistance and Elvenkind, amulet of Resistance, ring of Resist Fire and Cold,
dragon armours), rNexus (boots of Stability, amulet of Trickery), rShards (cloak of Protection,
law/balance/power dragon armour), rDisen (shield of Preservation, amulet of Weaponmastery,
chaos/balance/power dragon armour), rSound (crown of Serenity, gold/law/balance/power dragon
armour), rChaos (chaos/balance/power dragon armour), rLight/rDark (shining and power dragon
armour, crown of Night and Day), rNether (artifacts and random-resist egos only).

## 5.15 Quick rules for the borg

1. `dam_after = dam x (1/3)^res` for acid/elec/fire/cold/poison (res 0..2, immune = 0);
   high elements: `x 6/(8+1d4)` if resisted (use 2/3 for worst case); vulnerable `x 4/3`.
2. Breath = `min(cap, monster current HP / divisor)`; divisor 3 for acid, elec, fire, cold,
   poison, time, gravity; 6 for the rest. Caps in 5.2. Full strength to 7 grids (11 if POWERFUL),
   then `x 8/(d+1)` (or 12/(d+1)).
3. Danger estimate per breather = worst breath after resist x (number of breaths it can make
   before you act, from relative speed). Add every hound in the pack (1/5 breath chance each).
4. Paralysis without FREE_ACT is a death sentence near any meleeing monster: treat missing FA as
   "do not go below DL 20" and "do not melee floating eyes".
5. Confusion and blindness block scrolls and spells: when confused or blind and in danger, use
   potions (CLW/CSW/CCW cure both) - never plan escapes on scrolls without PROT_CONF/PROT_BLIND.
6. Stun > 50 (Heavy) or > 150 (Knocked Out) -> quaff CCW/Curing immediately.
7. Time and Black Breath ignore sustains and hold life; nexus can scramble or teleport-level
   you; disenchantment permanently lowers gear.
8. Before engaging a big base-element breather, drink the matching temporary resist.
9. Curse removal: only attempt if scroll strength >= curse power, otherwise the item becomes
   fragile and may be destroyed later.
10. Disarm traps with `skill - DL/5 > ~50%`; otherwise walk around or use magic.

---

## Things to try (untested)

1. **Double resist as a "free" immunity substitute.** Permanent rFire + Potion of Resist Heat
   gives 1/9 damage; versus immunity the difference on a 1600 breath is 177. Test whether a
   borg that always pre-drinks resist potions before engaging great wyrms survives as well as
   one hunting for immunity artifacts.
2. **Range-dancing breathers.** Breath damage falls off beyond 7 grids (11 for POWERFUL). An
   archer that keeps dragons at 12-15 grids takes 50-60% breath damage. Test how often dragons
   approach versus breathe from range, and measure average damage taken.
3. **Heal a breather less.** Breath depends on *current* HP: wounding a dragon first (e.g. with a
   big bolt or a wand of Drain Life) directly reduces its breath. Test killing order: hurt the
   biggest breather first, versus killing the weakest.
4. **Vulnerability curses with matching temporary resist.** A "burning up" item (vulnerable to
   fire, resists cold) plus Potion of Resist Heat nets fire 0 and keeps the cold resist. Test
   whether such cursed items with good stats are usable in practice.
5. **Hold Life vs nether.** Since HOLD_LIFE fully blocks nether's side effects (exp drain, mana
   loss, lost turn) even without rNether, test whether a caster can treat HL as "half of
   rNether" in the DL 40-55 band.
6. **Stun management by CON.** Stun recovery is `adj_con_fix + 1` per tick: CON 18/100 recovers
   5 per tick versus 1 at CON 13. Measure time spent stunned per hound fight at different CON.
7. **Amnesia and spellcasting.** Amnesia makes casting 50 + fail/2 but does not forbid it. For a
   low-fail mage, casting through amnesia is ~52% success - test whether it beats quaffing CCW
   first in emergencies.
8. **Trap-door escapes.** A known trap door (without Feather Fall) is an instant level change
   that does not need a scroll. Test using known trap doors as emergency exits.
9. **Uncurse expected value.** With Remove Curse strength 21-40 against a power-35 curse, the
   success chance is ~30%; the first failure only makes the item fragile. Simulate when it is
   better to accept the curse than risk destruction.
10. **Disenchanter bait.** Disenchantment never touches rings, amulets or the light source.
    Characters relying on jewellery for to-hit/to-dam lose less to disenchanters: test whether
    swapping enchanted weapons out before fighting disenchanter monsters is worth the turns.
11. **Drain-charges protection.** DRAIN_CHARGES only hits wands and staves in the pack (rods
    use timeouts and are immune, `src/obj-tval.c:tval_can_have_charges()`). Test whether a
    rod-based kit (Curing, Recall, Detection, Teleport Other) removes the lich/beholder risk
    enough to justify the higher device levels.

# 13. Escapes and Emergencies

Every escape and defensive tool in the game with its real mechanics (distances, delays,
what blocks it), what still works under each bad status effect, how to compute the damage
you can take before your next action and turn it into HP thresholds, how many escapes to
carry at each stage, and decision procedures for the emergencies that kill characters.

**Contents**

1. [Teleport mechanics](#131-teleport-mechanics)
2. [What still works under each status effect](#132-what-still-works-under-each-status-effect)
3. [The toolkit, item by item](#133-the-toolkit-item-by-item)
4. [Damage per turn and HP thresholds](#134-damage-per-turn-and-hp-thresholds)
5. [The escape budget](#135-the-escape-budget)
6. [Emergency decision procedures](#136-emergency-decision-procedures)
7. [Things to try (untested)](#things-to-try-untested)

Related: [Chapter 9](09-dungeon.md) (stairs, recall, Deep Descent, Teleport Level,
no-teleport areas), [Chapter 10](10-exploration.md) (when to leave),
[Chapter 5](05-elements-and-status.md) (status effects, resistances),
[Chapter 7](07-consumables-and-devices.md) (device failure, carry kits),
[Chapter 8](08-monsters.md) and [A3](appendices/A3-monster-spells.md) (monster
attacks). The official manual covers the commands only
(<https://angband.readthedocs.io/en/latest/command.html>,
[user-manual/command.rst](user-manual/command.rst)).

---

## 13.1 Teleport mechanics

### `TELEPORT` (Phase Door, Teleportation, Teleport Self, Portal, Shadow Shift...)

FACT (`src/effect-handler-general.c:effect_handler_TELEPORT()`):

1. **Blocks.** The player cannot teleport if standing on a `NO_TELEPORT` square **and the
   distance is greater than 10 (or 0)**, or if wearing an item with the
   **anti-teleportation curse** (`NO_TELEPORT`, blocks every distance). No teleporting
   at all in the single-combat arena. Phase Door (distance 10) therefore **works on
   no-teleport squares** (gauntlet levels), but not with the curse.
2. **Distance.** `dis = base + dice`. For "M" effects (`dice:M60`) the distance is instead
   `perc% × max(distance from you to the farther vertical edge, to the farther horizontal
   edge)`. Then ±25%: half the time `dis -= randint0(dis/4)`, else `dis += randint0(dis/4)`.
3. **Destination.** Every grid of the level is scored by `|distance − dis|`; only the
   best-scoring grids are kept and one is chosen at random. So you land **at almost
   exactly the rolled distance**, in any direction, possibly through walls.
4. **Legal destinations** (`has_teleport_destination_prereqs()`): passable, **no trap**
   (any player trap, known or not), no monster, not damaging terrain (lava), not webbed,
   not a shop entrance. **Vault squares are avoided** unless nothing else qualifies.
5. Your target is cleared if it is no longer visible.

Distances of the player teleports:

| Source | Formula | Typical distance |
|---|---|---|
| Phase Door (scroll; Mage spell clvl 2; Rogue 7; activation `TELE_PHASE`) | 10 | **9–11** |
| Necromancer *Shadow Shift* (15) | 30, and you take 2d5 damage | 23–37 |
| Priest *Portal* (10) | `clvl/2 + 30` | 35 at clvl 10, 55 at clvl 50 (±25%) |
| Mage *Teleport Self* (7), Rogue *Teleport Self* (17) | `M` with perc = `clvl × 9/5` | clvl 10: 18% of span (≈18 grids mid-level); clvl 30: 54%; clvl 50: 90% |
| Scroll / Staff of Teleportation, activation `TELE_LONG` | `M60` (60% of span) | middle of a 198-wide level: span 99 → 45–74 grids; near an edge: span ~190 → 85–140 |
| Curse "teleportation" | 40, at random times | 30–50 |
| Monster *blink* (on itself) | 10 | |
| Monster *teleport* (TPORT, on itself) | `2 × max_sight + 5` = 45 | |
| Monster *teleport away* (on you) | 100 | 75–125 |
| Thief "touch to steal" escape | 45 | |
| Teleport Other on a monster | wand/rod `5 × max_sight` = 100; Mage/Rogue spell `3 × clvl` | |

ADVICE: Phase Door moves you ~10 squares — usually still in line of sight of a breather
or caster in a big room, but out of melee range for a few turns. Use it against melee and
to break line of sight around corners, not against ranged attackers in open areas.
Teleportation moves you far; you may land next to something worse (one more reason to
detect after every teleport). The early Mage *Teleport Self* is short (18% of the span at
clvl 10) — not a substitute for a Staff/Scroll of Teleportation until clvl ~30.

### `TELEPORT_TO` (Dimension Door; monster *teleport to*)

FACT (`effect_handler_TELEPORT_TO()`): moves the player (or monster) to a legal grid near
the aim point, widening the search radius as needed. **Can land in vaults** (randomised
by 10 grids if you aim into one). Blocked by a no-teleport square under you or the
anti-teleport curse. Mage *Dimension Door* (clvl 35): you choose the target.

Monster *teleport to* brings **you** next to the monster; *teleport toward* (TELE_SELF_TO)
brings the monster next to you.

### `TELEPORT_LEVEL`

FACT (`effect_handler_TELEPORT_LEVEL()`; full rules in
[Chapter 9.17](09-dungeon.md#917-teleport-level)): instant; up or down at random when
both are legal; always up from DL99/DL100 while their quest is open; never up in town or
with forced descent; blocked by no-teleport squares and the curse; **nexus resistance
blocks a monster's teleport-level spell**.

Gotcha: if a Ranger **decoy** is deployed anywhere on the level, *any* Teleport Level
effect — including your own scroll or spell — destroys the decoy instead of moving you
(the decoy check comes before the source check).

### Monster teleport-away and decoys

A monster's *teleport away* on you is absorbed by your Ranger decoy if one exists
(`context->subtype` path); otherwise it moves you 75–125 grids. Nexus breath and nexus
melee have their own teleport side effects, prevented by nexus resistance
([Chapter 5](05-elements-and-status.md)).

---

## 13.2 What still works under each status effect

FACT (`src/player-util.c:player_can_read()`, `player_can_cast()`, `player_can_fire()`,
`player_confuse_dir()`, `src/cmd-obj.c`, `src/player-calcs.c`,
`src/player-spell.c:spell_chance()`, `src/target.c:target_able()`):

* **Reading** requires: not blind, **your own grid lit or you carry light** (`no_light()` =
  your grid is not "seen"), not confused, not amnesiac.
* **Casting** requires: not blind, light, not confused. Amnesia doesn't block casting but
  sets fail to `50 + fail/2`.
* **Devices** (quaff, eat, aim wands, use staffs, zap rods, activate) have **no status
  checks at all**; they only use device skill. Aimed devices while confused go in a random
  direction 75% of the time.
* **Melee** is blocked by fear (`AFRAID`, including Terror).
* **Missiles/throwing** are allowed while afraid.
* **Targeting a monster** is impossible while hallucinating; aim by direction instead.

| Condition | Read scroll | Cast spell | Quaff/eat | Aim wand / zap rod / use staff / activate | Melee | Shoot / throw | Walk | Take stairs | Run / pathfind / `p` |
|---|---|---|---|---|---|---|---|---|---|
| **Blind** | no | no | yes | yes (aim by direction; can't see targets) | yes | yes (no target) | yes | yes | run yes; you can't see what stops it |
| **No light** (unlit grid, no light source) | no | no | yes | yes | yes | yes | yes | yes | yes |
| **Confused** | no | no | yes | yes, but aimed effects go random 75% | yes | aimed random 75% | 75% random direction | **yes** (no direction needed) | **no** |
| **Amnesia** | no | yes, fail = 50 + fail/2 | yes | device skill −20% | yes | yes | yes | yes | yes |
| **Afraid** (fear) | yes | yes, fail +20 (before the 50% cap) | yes | device skill −5% | **no** | yes | yes | yes | yes |
| **Terror** (mushroom) | yes | as afraid | yes | as afraid | **no** | yes | yes, **+10 speed** | yes | yes |
| **Stun** | yes | fail +15 after cap (heavy stun +25) | yes | skill −10% (heavy −20%) | −5 to-hit/dam (heavy −20) | penalised | yes | yes | yes |
| **Knocked out** (stun > 150) | – | – | – | – | – | – | **no turns at all** | – | – |
| **Paralysed** | – | – | – | – | – | – | **no turns at all** (non-stacking: can't be extended while paralysed) | – | – |
| **Hallucinating** | yes | yes | yes | skill −20%; no monster targeting | yes | no targeting | yes | yes | yes |
| **Poisoned** | yes | yes | yes | skill −5% | yes | yes | yes | yes | yes |
| **Slowed** | all actions at −10 speed | | | | | | | | |
| **Silence** | *does not exist in this game*; amnesia and blindness are the "can't cast" effects | | | | | | | | |

Other consequences:

* **No HP regeneration** while poisoned, cut, stunned or paralysed
  (`player_regen_hp()`); poison costs 1 HP per tick, cuts 1–3 per tick by severity.
* Warriors gain permanent **protection from fear at clvl 30** (`BRAVERY_30`,
  `src/player.c:player_flags()`). Heroism and Berserk Strength also grant fear protection
  while active (`flag-synonym:PROT_FEAR` in `player_timed.txt`) and cure fear.
* Free Action prevents paralysis; a saving throw (`SKILL_SAVE`) also protects against the
  *hold person* spell and paralysing melee.
* Blindness also stops secret-door searching and the loot feeling count.

ADVICE — "what can I still do?" in one line each:

* Blind or confused → **potions, devices, stairs**. Keep a CLW/CSW/CCW (cures both) and
  your escape staff handy; don't plan on scrolls or spells.
* Afraid → shoot, throw, cast, use devices; drink Boldness/Heroism/Berserk/CCW... (CCW
  does not cure fear; Boldness, Heroism, Berserk, Holy Word, Staff of Holiness and the
  Mushroom of Clear Mind do).
* Amnesia → devices and potions; spells are a coin flip.
* Paralysed/knocked out → nothing; prevention only (Free Action, protection from stun).

---

## 13.3 The toolkit, item by item

Detailed object data in [Chapter 7](07-consumables-and-devices.md) and
[A6](appendices/A6-object-kinds.md); spells in [Chapter 3](03-magic-and-spells.md) and
[A7](appendices/A7-class-spells.md). Store availability from `lib/gamedata/store.txt`:
the **Alchemist always stocks Phase Door, Word of Recall, Remove Curse and CLW**, and
sometimes CSW, CCW, Magic Mapping, Monster Confusion, Blessing/Holy Chant, Boldness,
Berserk Strength, Restore Life Levels, Neutralize Poison, Resist Heat/Cold; the **Magic
shop** sometimes has the **Staff of Teleportation**, Wands of Stone to Mud / Slow /
Confuse Monster, Staves of Mapping/Detect Evil/Detect Invisible. **Scrolls of
Teleportation, Teleport Level and Deep Descent are not in any regular store** (Black
Market or dungeon only).

### A. Moving yourself away

| Tool | Effect / numbers | Fails when | Notes |
|---|---|---|---|
| **Scroll of Phase Door** (DL1, 15 gp) | teleport 9–11 | can't read; anti-tele curse | the basic escape; works on no-teleport squares |
| **Scroll of Teleportation** (DL10+) | 60% of span | can't read; no-teleport square; curse | not sold in stores |
| **Staff of Teleportation** (DL20+, Magic shop) | as the scroll, 5+1d4 charges | device failure; no-tele; curse | works blind/confused — the escape of choice for most classes |
| **Teleport Self** (Mage 7, Rogue 17) | `clvl×9/5`% of span | spell failure, blind/confused | short at low level |
| **Portal** (Priest 10) | 35–55 grids | spell failure | |
| **Shadow Shift** (Necro 15) | 30 grids, 2d5 damage to you | | |
| **Dimension Door** (Mage 35) | to a chosen grid | no-tele under you | can enter vaults |
| **Hit and Run** (Rogue 23) | after your next *steal* attempt (`s`) you teleport 20 grids (`src/mon-util.c`) | | steal-and-vanish |
| **Mushroom of Terror** | `TERROR` 25+5d5 turns: +10 speed, cannot melee | | run away at double speed; works with no light |
| **Mushroom of Sprinting** | +10 speed for 100 turns, then **slowed 100 turns** | | the slow afterwards can kill |
| **Potion of Speed** | +10 speed, 20+2d10 turns | | doubles your actions relative to monsters |
| **Staff/Rod of Speed**, Druid/Ranger *Haste Self* (25) | +10 speed | device/spell failure | |

### B. Leaving the level

| Tool | Delay | Destination | Fails when |
|---|---|---|---|
| Stairs | 1 turn | ±1 | – |
| **Scroll of Teleport Level** (DL20+), Mage spell (28), Rogue spell (35) | instant | ±1 random (up only on quest levels) | no-tele square, curse, decoy present, can't read |
| **Scroll of Deep Descent** (DL1–100, level 10) | **4–7 ticks** | `max_depth + 5` | target not deeper than you → explosion |
| **Word of Recall** (scroll, Rod DL30+, Priest/Paladin 25) | **15–34 ticks** | town / recall depth | arena; `birth_no_recall` |
| Trapdoor | instant | +1 | – |

### C. Healing (HP)

| Tool | Heals | Also cures |
|---|---|---|
| Potion of Cure Light Wounds | 20 | blindness; reduces cut and confusion by 20 |
| Potion of Cure Serious Wounds | 40 | blindness, cut, confusion |
| Potion of Cure Critical Wounds | 60 | blindness, cut, poison, stun, amnesia, confusion |
| Potion of Healing | **max(300, 35% of missing HP)** | as CCW |
| Potion of \*Healing\* | 1200 | as CCW |
| Potion of Life | 5000 | everything incl. hallucination; restores stats and exp |
| Staff of Cure Light Wounds / Healing; Rod of Healing (500, cut, stun) | as listed in `object.txt` | |
| Mushroom of Emergency | 200, then hallucination + temp rFire/rCold | |
| Mushroom of Fast Recovery | 30 | stun, cut, poison, blindness |
| Priest/Paladin *Minor Healing* | `clvl + 10` (with a % component) | reduces cut/stun |
| Priest/Paladin *Healing* (clvl 30) | **2000** | cut, poison, stun, amnesia |
| Priest *Holy Word* (39) | 1000 + dispel evil | fear, poison, stun, cut |
| Druid *Rapid Regeneration* (37) | 30 HP per tick (timed) | |
| Elvish Waybread | 4d8, cures poison | |

### D. Controlling monsters

| Tool | Effect | Resisted by |
|---|---|---|
| **Teleport Other** (Wand DL20+, Rod found DL45+; Mage 15, Rogue 30) | bolt; the first monster hit is teleported 100 (spell: `3×clvl`) away | **nothing** except being a bolt (an intervening monster takes it) |
| *Banish Evil* (Priest 25) | teleports all evil in LOS away | non-evil |
| **Banishment** (scroll DL40, staff DL70, Mage 35) | removes **every non-unique monster of one symbol on the level**; you take 1d4 per monster removed | uniques |
| **Mass Banishment** (scroll DL50, Mage 40, activation) | removes every non-unique monster within **20 grids**; 1d3 damage each | uniques |
| ***Destruction*** (scroll DL40, staff DL50, Priest *Word of Destruction* 35, Necro *Grond's Blow* 35) | radius 15: deletes monsters (**including uniques — they are removed, not killed, and can come back**), objects and terrain except permanent walls and stairs; blinds you 10+1d10 unless you resist light (or dark for Grond's Blow) | nothing; useless in town and arena |
| Staff of Earthquakes (DL40), Druid *Tremor* (20, targeted, radius 4), *Volcanic Eruption* (40, radius 5), Blackguard *Quake* (44, radius 13) | earthquake: walls and floors shuffle, monsters caught in rubble are crushed or buried | |
| **Hold Monster** (wand DL5, rod DL30) | the monster loses all its turns for 3+1d5 turns; **any damage ends it** | `NO_HOLD` only — no level-based save in this code path |
| Slow Monster (wand DL5/15, rod DL30, staff DL10/20 "Slow Monsters" in LOS, Druid 6) | −2 speed per 10 timer turns, up to −10 (wand 10+1d20 → −4 to −6) | `NO_SLOW` only |
| Confuse Monster(s) (wand, staff in LOS, Druid 5, scroll of Monster Confusion for melee) | confused monsters move randomly and cast inaccurately | `NO_CONF` only |
| Stun Monster (wand) | a stunned monster loses each turn with chance 1/10 (`STUN_MISS_CHANCE`) | `NO_STUN` |
| Scare Monster (wand DL10, Necro *Frighten*, Blackguard *Howl*, Necro *Unleash Chaos*) | fear: monster flees and doesn't melee | `NO_FEAR`; save `min(90, level + max(0, 25 − timer/2))` (wand timer 10 → level+20%) |
| Sleep Monsters (staff, Druid *Mass Sleep* 25, Necro *Sleep Evil* 12) | puts monsters to sleep; any damage wakes them | `NO_SLEEP`; save ≈ `min(90, level)`%, uniques roll twice |
| *Protection from Evil* (scroll DL30, Priest 14, Paladin 15, Staff of Holiness) | each melee blow of an **evil monster of level ≤ yours** is repelled with chance `(clvl + 49)%` approx | non-evil or higher-level monsters |
| **Glyph of Warding** (Priest 20), **Scroll of Rune of Protection** (DL70+), activation | monsters can't step onto the glyph; each attempt breaks it with chance `(level−1)/550` | ranged attacks ignore it; one decoy/glyph per square |
| Ranger *Decoy* (30) | monsters attack/target the decoy; it absorbs monster teleport-away / teleport-level | one at a time; also eats your own Teleport Level |
| Blackguard *Relentless Taunting* | draws monsters to you | (an anti-escape) |

FACT sources: `effect_handler_BANISH()`, `MASS_BANISH()`, `DESTRUCTION()`, `GLYPH()`,
`src/mon-move.c:monster_turn_attack_glyph()`, `src/mon-attack.c` (protection from evil),
`src/mon-timed.c:does_resist()` (only sleep and fear get level saves), `src/mon-util.c:
mon_take_hit()` (damage clears hold).

### E. Terrain tools

| Tool | Use |
|---|---|
| Wand of **Stone to Mud** (DL10, Magic shop), Druid *Turn Stone to Mud* (8), Ranger (15) | dig a 1-wide tunnel through granite/veins/rubble (not permanent rock) — out of dead ends, into a corridor to fight in, or to cut a vault wall; damages `HURT_ROCK` monsters 20+1d30 |
| Blackguard *Shatter Stone* (7) | short-range wall removal |
| Closing and locking doors (`c`, `D`) | blocks non-door monsters; locks delay weak ones ([Chapter 9.11](09-dungeon.md#911-doors)) |
| Mage *Door Creation* (13) | surrounds you with doors |
| Scroll of Door Destruction, Mage *Disable Traps, Destroy Doors* | opens a path |
| Tunnelling rubble (`T`) | rubble falls at `8 × digging skill / 1600` per turn |

### F. Curing and status

CCW is the universal emergency potion (blind, confused, poison, stun, cut, amnesia) but
not fear. Boldness/Heroism/Berserk cure fear (Hero/Berserk also prevent it). CLW cures
blindness and reduces confusion. Mushroom of Clear Mind: +20 SP, cures confusion, fear,
hallucination, grants temporary confusion protection. Staff/Rod of Curing: blindness,
cut, poison, stun, confusion (no HP). Potion of Neutralize Poison / Resist Poison.
Mushroom of Second Sight: cures blindness and gives telepathy 50+7d7 turns.

### G. Monster abilities that defeat escapes

FACT (`lib/gamedata/monster_spell.txt`; save = `randint0(100) < SKILL_SAVE` where the
spell has a `message-save` line, plus the relevant protection flag):

| Monster ability | Effect on you | Counter |
|---|---|---|
| *teleport to* (`TELE_TO`) | pulls you next to the caster — undoes a phase/teleport | kill or Teleport Other the caster; leave the level |
| *teleport toward* (`TELE_SELF_TO`) | the caster appears next to you | same |
| *blink* / *teleport* (on itself) | the monster jumps 10 / 45 grids — it may come back | detect before resting |
| *teleport away* (`TELE_AWAY`) | throws you 75–125 grids (absorbed by a Ranger decoy) | usually helps you |
| *teleport level* (`TELE_LEVEL`) | moves you ±1 level | nexus resistance; decoy |
| nexus breath/attacks | random teleport, teleport-to, teleport level or stat swap | nexus resistance |
| *forget* (`FORGET`) | amnesia 3 turns → **no scrolls**, spells at ≥50% fail | save only (no protection flag); CCW cures amnesia; use devices |
| *blind* (`BLIND`) | 11+1d4 turns blind → no scrolls/spells | save; protection from blindness; CLW/CCW |
| *confuse* (`CONF`) | 3+1d4 turns confused → no scrolls/spells, random aim | save; protection from confusion; CSW/CCW |
| *scare* (`SCARE`) | 3+1d4 turns afraid → no melee | save; protection from fear; Boldness/Hero/Berserk |
| *hold* (`HOLD`) | 3+1d4 turns paralysed | save; **Free Action** |
| *slow* (`SLOW`) | 3+1d4 turns at −10 speed | save; **Free Action** also blocks it (`fail:1:FREE_ACT` on `SLOW` in `player_timed.txt`) |
| *drain mana* | removes `1d(power/2+1)` SP | keep scrolls/devices as backup escapes |
| *shriek* | wakes everything nearby and hastes monsters in LOS (+25 timer) | kill shriekers first; leave if the level wakes |
| summons (`S_*`) | more monsters next to you; deep `S_HI_UNDEAD`/`S_HI_DRAGON`/`S_HI_DEMON`/`S_UNIQUE` are deadly | break LOS; Teleport Other the summoner; leave |
| `PASS_WALL` / `KILL_WALL` movement | ghosts and wall-eaters follow you through rock after a phase | teleport (long) instead of phase |
| speed | a +10/+20 monster re-closes a 10-grid phase in 1 turn | long teleport, then leave |

ADVICE: the dangerous combinations are *blind/confuse/forget + damage* (it turns your
scroll escapes off right when you need them) and *teleport-to + breath*. Carry at least
one device escape and one potion cure for exactly these.

---

## 13.4 Damage per turn and HP thresholds

### The quantity to watch

For a borg (and a careful human) the number that matters is **D = the most damage you can
take before your next action**:

```
D = Σ over awake monsters that can act before your next turn of
      moves(m) × max(worst melee round if it can reach you,
                     worst ranged attack (breath/spell/arrows) if it has LOS)
moves(m) = turn_energy(monster speed) / turn_energy(your speed)
```

FACT inputs:

* Energy per game turn (`src/game-world.c:extract_energy`): normal speed 10, +10 → 20,
  +20 → 30, +30 → 38, −10 → 5. So a +10 monster acts twice per action of a normal-speed
  player; a +20 monster three times; against a +10 player a +20 monster acts 1.5 times.
* **Breath** damage = monster's **current** HP / divisor, capped
  (`src/mon-spell.c:breath_dam()`, `lib/gamedata/projection.txt`): acid, lightning, fire,
  cold `/3` cap 1600; poison `/3` cap 800; light, dark, sound, shards `/6` cap 500; nexus
  `/6` cap 400; nether, chaos `/6` cap 600; disenchantment `/6` cap 500; gravity `/3` cap
  200; inertia and force `/6` cap 200; time `/3` cap 150; plasma `/6` cap 150; mana `/3`
  cap 1600. A wounded breather breathes less.
* **Resistance** (`src/project-player.c:adjust_dam()`): base elements and poison ×1/3 per
  resist level (single resist 1/3, resist + temporary resist 1/9), immunity 0,
  vulnerability ×4/3. High elements (light, dark, sound, shards, nexus, nether, chaos,
  disenchantment) ×6/(8+1d4) ≈ 0.5–0.67 with resistance. Acid is halved first if armour
  absorbs it.
* **Spells**: e.g. `BA_MANA` = 5×spell power + 10d10 (unresistable), `BA_DARK`/`BA_NETH` =
  4×power + 10d10, `BO_MANA` = 50 + 1d(2.5×power), `WOUND` (cause wounds) = (2×power/3)d5
  plus cuts; spell power defaults to the monster's level (`monster.txt` header,
  `monster_spell.txt`). Spells are cast with probability 1/`spell-freq` per monster turn,
  but the worst case is what kills.
* **Melee** worst case = Σ of max dice of all blows (each blow must hit first; AC
  matters).

### Worked worst cases (unresisted maxima from `monster.txt`)

| Depth | Example threat | Worst single action | Per your turn at normal speed |
|---|---|---|---|
| DL5–10 | Grishnákh (melee), Brodda (melee) | 44–48 | 44–48 |
| DL10–15 | baby dragons' breath | 29–38 | 29–38 |
| DL15–20 | Bolg, Son of Azog (+10) | 72 melee | **144** |
| DL20 | chimaera fire breath | 53 | 53 |
| DL25–30 | stone troll uniques Bert/Bill/Tom (+10) | 102 melee | **204** each |
| DL28 | basilisk poison breath (+10) | 103 (34 with rPois) | 206 |
| DL35 | gravity hounds (pack of ~6) | 64 each, unresistable | **~380** if all breathe |
| DL39–43 | mature dragons / mature multi-hued | 233–333 breath | 233–333 (78–111 resisted) |
| DL50 | ancient dragons; The Balrog of Moria (+20) | 500; 1000 fire | 500; **3000** fire (1000 resisted) |
| DL55 | dracolich/dracolisk | 1026 | 1026 (342 resisted, 114 double) |
| DL70–80 | great wyrms | 1400–1500 | (467–500 resisted, 156–167 double) |
| DL85+ | Ancalagon, greater Balrogs, many uniques | breath cap 1600 | 533 resisted, 178 double resisted |
| DL99 | Sauron (+20, spell power 130) | BA_DARK/BA_NETH ≤ 620, BO_MANA ≤ 375 | ×3 against a normal-speed player |
| DL100 | Morgoth (+30, power 100) | BA_MANA ≤ 600; melee 520 | ~1.27 rounds per action against a +20 player |

ADVICE — the thresholds (and why):

1. **Escape now if `chp ≤ D_worst`** — you could die before acting again. Healing doesn't
   help if the next monster round out-damages the potion.
2. **Heal if `chp − D_worst` is small but positive and the heal amount > D_expected** —
   e.g. a Healing potion (≥300) against a single monster doing ~150 per turn.
3. **Otherwise escape if healing can't keep up**: if `D_expected ≥ heal per turn`, every
   heal just postpones the escape; escape while you still have the HP to survive a failed
   attempt.
4. **Allow for failure**: if your escape has failure rate *f* (device/spell), require
   `chp > D_worst × (1 + f/(1 − f))` or use a 0%-failure option (scroll, stairs).
   Scrolls never fail once readable.
5. Rule-of-thumb percentages that follow from typical character HP versus the table:

| Phase | Typical max HP (rough) | Act (heal/back off) at | Escape at |
|---|---|---|---|
| DL1–10 | 30–120 | 60% | 40% or when an unknown unique appears |
| DL10–30 | 100–300 | 60% | 40%; immediately vs. +10-speed uniques |
| DL30–50 | 250–600 | 60% | **50%** (hound packs and dragon breath take 200–400 at once) |
| DL50–80 | 500–1000 | 70% | 50%, or any time an unresisted big breather is in LOS |
| DL80–100 | 800–1500+ | 75% | 50%; never stand in LOS of an unresisted 1600-cap breather |

The deeper you go the *earlier* you must act, because single actions remove a larger
fraction of your HP. "Act at 50%, escape at 30%" is fine to DL30 and suicidal at DL50+.

---

## 13.5 The escape budget

ADVICE (minimum stock to carry before descending; "teleport" = Scroll or Staff of
Teleportation charge, or a reliable ≤10%-fail long-range spell):

| Phase | Short escapes | Long escapes | Level exits | Healing | Other |
|---|---|---|---|---|---|
| **DL1–5** | 5+ Phase Door | – | 1 WoR (from DL5) | 3–5 CLW | Flasks of Oil to throw |
| **DL5–15** | 10 Phase | Staff of Teleportation if affordable, else 2–3 scrolls when found | 1–2 WoR | 5 CLW + 3 CSW | Wand of Stone to Mud nice |
| **DL15–30** | 10 Phase | **5+ teleports** (staff + scrolls) | 2 WoR, 1–2 Deep Descent | 5 CSW + 5–10 CCW | Free Action by DL20–25; Teleport Other wand when found |
| **DL30–50** | 10–15 Phase | **8+ teleports**, two independent sources | 2 WoR; Teleport Level ×2–3; Deep Descent | 10+ CCW, 2–5 Healing | Speed potions ×3+; Teleport Other |
| **DL50–80** | 15 Phase | 10+ teleports | WoR ×2 (or Rod of Recall), Teleport Level ×3+ | 5+ Healing, 2+ \*Healing\* | Speed ×5+, Mass Banishment/\*Destruction\*, Teleport Other rod |
| **DL80–100** | 15 Phase | Staff + scrolls, 15+ | as above | 10+ Healing, 5+ \*Healing\*, Life if found | Speed ×10, Mass Banishment ×3+, *Destruction* ×3+ |

Principles:

* **Two independent escape types** (e.g. scroll + staff): blindness/confusion kills the
  scroll, device failure or an antimagic situation kills the staff.
* **Count charges, not items**: a staff with 2 charges left is 2 escapes.
* **Escapes are consumed on the level**: if you used more than half your long escapes
  on one level, go home and restock rather than continuing.
* Keep at least **one escape you can use while blind and confused** (staff, or stairs
  underfoot).
* **Rod of Recall** and Teleport Level are your "end the level" buttons; Deep Descent is
  conditional (only if `max_depth + 5` is fine).

---

## 13.6 Emergency decision procedures

### Choosing *which* escape (general algorithm)

```
CHOOSE_ESCAPE(threats):
  usable = escapes not blocked by: blind/no-light/confused/amnesia (scrolls, spells),
           no-teleport square (anything > 10 grids, Teleport Level),
           anti-teleport curse (all teleports), arena (all), decoy (Teleport Level)
  if standing on stairs:                         return STAIRS        # instant, 0% fail
  if every threat is melee-only and none is faster than me:
                                                 return PHASE         # 9-11 grids is enough
  if a threat has teleport-to, or is PASS_WALL/KILL_WALL, or is faster by >= +10:
                                                 prefer LONG TELEPORT, then plan to leave
  if the level itself is lost (many awake threats, no safe spot):
      if TELEPORT_LEVEL usable and not on a quest level going down: return TELEPORT_LEVEL
      if DEEP_DESCENT usable and max_depth+5 acceptable and I can survive 4-7 ticks:
                                                 return DEEP_DESCENT
      if RECALL usable and I can survive 15-34 ticks (after a teleport): TELEPORT then RECALL
  default:                                       return LONG TELEPORT
  tie-breaks: 0%-fail (scroll, stairs) over device over spell when chp <= D_worst;
              cheaper/renewable (rod, spell) over consumable when chp is comfortable.
After any teleport: detect monsters, cure, heal to a safe margin, then decide to stay or leave.
```

Each procedure lists checks in order; take the first action that applies. "Escape" means
the best available long escape (Teleportation staff/scroll/spell); "phase" means Phase
Door; "stairs" means a staircase you are standing on. Speed matters: if you are slower
than the threat, prefer instant exits (stairs, Teleport Level, teleport) to delayed ones.

### Low HP

1. If on stairs and `chp ≤ D_worst` → **take the stairs**.
2. If `chp ≤ D_worst` → **escape** (0%-fail source first: scroll if you can read).
3. If a heal brings you above `D_worst` with margin and `D_expected < heal` → **heal**
   (CCW at low level, Healing at mid/late).
4. If the threat is melee-only and adjacent → **phase**, then shoot/heal/leave.
5. After escaping: detect, cure, quaff to ~70%, rest in a safe spot or leave the level.

### Poisoned or badly cut

1. Remember: **no HP regeneration** while poisoned or cut; poison does 1/tick, cuts up to
   3/tick ("Deep Gash"/"Mortal Wound").
2. Cure: CCW (both), CSW (cuts), Neutralize Poison/Resist Poison (poison), Staff/Rod of
   Curing, Herbal Curing, Fast Recovery mushroom.
3. Mortal wounds don't heal by themselves — cure immediately.
4. If no cure: leave the fight; rest won't help until the effect expires.

### Paralysis threat without Free Action

(Carrion crawlers, ghouls and ghasts, floating eyes (gaze), basilisks, homunculi;
monsters with the *hold* spell.)

1. **Never melee a paralyser without Free Action.** Paralysis is non-stacking, but a
   group can chain it.
2. Kill from range, or avoid: close a door, phase away, leave.
3. If several paralysers are awake and approaching → **leave the level now** (stairs,
   teleport, Teleport Level).
4. Get Free Action before DL20–25 ([Chapter 6](06-equipment.md)).

### Summoner in view

1. Kill it in the next turn or two if you can (summoners are usually frail).
2. Otherwise break line of sight immediately (most summon spells need LOS) — step around a
   corner, close a door, or teleport it away (**Teleport Other** is ideal: no resist).
3. If summons are already arriving and outclass you → leave the level.
4. Don't fight summoners in open rooms; corridors limit what can reach you.

### A unique you can't beat

1. Is it awake and aware of you? If not, leave it asleep — go elsewhere or leave.
2. **Teleport Other** it (works on uniques), or leave by stairs.
3. If it is faster than you: phase/teleport, then leave the level before it finds you
   again; don't try to outrun a +10 unique on foot.
4. Banishment/Mass Banishment **don't** affect uniques; *Destruction* removes them
   temporarily.

### Surrounded

1. Count adjacent damage: if `D_worst ≥ chp` → escape (teleport, not phase — phase may
   land within reach of the same pack).
2. If the pack is weak individually → move so that only 1–2 can reach you (corridor,
   corner), then fight.
3. Area tools: *Mass Sleep*, Staff of Sleep/Confuse/Slow Monsters, Scare (wand) on the
   strongest, ball spells.
4. Last resort: *Destruction* (blinds you unless rLight) or Mass Banishment.

### Out of light

1. You can't read or cast. Use devices (Staff of Light, Rod of Illumination light the
   room around you) or stand in a lit room — "no light" means *your own grid* is not seen
   (`src/cave-view.c:no_light()`), so a lit grid counts as light.
2. Swap to a spare light (always carry one), refuel a lantern (`F`) with a flask.
3. Recall cannot be read without light — **use a Rod of Recall** or walk to stairs.
4. Mushroom of Terror/Sprinting, potions and staves still work.

### Out of food

1. Eat anything: rations, waybread, slime molds, mushrooms (+5), potions give 1 each,
   CLW/CSW/CCW give +1.
2. Scroll of Remove Hunger, Druid/Ranger *Remove Hunger*, Herbal Curing.
3. Weak → regeneration halved; Faint → you randomly pass out. Head to town (WoR) while
   you still can act.

### Blinded

1. You can't read or cast. Potions and devices work.
2. Cure: CLW/CSW/CCW, Staff/Rod of Curing, Mushroom of Second Sight or Fast Recovery.
3. If enemies are adjacent and you can't cure: **staff of teleport** or stairs.
4. Blindness from *Destruction* (light) is avoided with rLight; get rBlind/rLight before
   using it as an emergency button.

### Confused

1. You can't read or cast; walking goes in a random direction 75% of the time; you cannot
   run or autoexplore.
2. Cure: CLW (reduces), CSW, CCW, Staff/Rod of Curing, Mushroom of Clear Mind.
3. Stairs work while confused (no direction needed) — if on stairs and threatened, take
   them.
4. Staves work (no direction); wands/rods aimed at a monster usually miss.
5. Prevention: resist confusion (or Clear Mind's temporary protection) by DL40.

### Afraid

1. You can't melee; you can shoot, throw, cast (fail +20) and use devices.
2. Cure: Boldness, Heroism, Berserk Strength, Holy Word, Staff of Holiness, Clear Mind.
3. Warriors are immune from clvl 30; Heroism/Berserk prevent it while active.
4. A melee character afraid next to a monster: cure fear or phase away; don't stand there.

### Stat drained

1. Not an emergency in itself. Stats are **restored automatically on every level gain**
   (`src/player.c` restores all five stats when the level goes up).
2. Restorers: Mushroom of Vigor (all), Potions of the specific stat (also restore), Sip
   of Miruvor (STR/CON), Purging (STR/CON), Revitalize/Restoration/Unholy Reprieve spells,
   Rod of Restoration, Potion of Life.
3. Prevention: sustain items; avoid the monster that drains.

### Mana gone

1. Treat a caster with 0 SP as a weak fighter: escape with devices/scrolls.
2. Refill: Potion of Restore Mana, Staff of the Magi, Mushroom of Clear Mind (+20),
   Necromancer *Tap Unlife* (from monsters) and *Power Sacrifice*, Mage *Tap Magical
   Energy* (drains a device).
3. Rest in a safe place; mana regenerates like HP (resting doubles it; Blackguards
   regenerate differently, see [Chapter 1](01-core-mechanics.md)).

### Inventory being destroyed

1. Elemental attacks destroy unprotected pack items: chance per item roughly
   `min(5 × damage taken, 300)/10000` (≤3%) per hit for acid/fire/cold/lightning
   (`src/project-obj.c:inven_damage()`), applied to the post-resistance damage — so
   resisting also protects the pack. Equipped items and artifacts are never destroyed
   this way.
2. Which pack items are at risk (`HATES_*` in `lib/gamedata/object_base.txt`): **fire**
   — scrolls, all magic books, staves, arrows, bows, hafted/polearm weapons, soft armour,
   cloaks, boots, gloves, light sources, chests; **cold** — potions and flasks (also hurt
   by sound, shards, ice, force); **lightning** — rings, wands, rods; **acid** — scrolls,
   staves, missiles and nearly all weapons and armour. Non-ammo weapons in the pack lose
   to-hit/to-dam instead of being destroyed.
3. Response: get the resistance (temporary resist potions/spells work), kill or avoid the
   source; spread critical consumables across types (scroll + staff + potion).
4. Disenchantment/drain charges melee (Sauron, Morgoth) attack equipment and devices —
   keep distance.

### Hounds pack arriving

1. Hounds (`Z`) come in packs and breathe from range; several breathe per turn.
2. Identify the type (detect, look). If you resist the element, fight in a corridor where
   one or two can see you.
3. If you don't resist it (gravity, inertia, impact, nexus, nether, plasma, time, chaos,
   or base elements before you have them) and the pack is large → **leave**: stairs if
   under you, otherwise teleport then leave.
4. Gravity breath also blinks you 5 grids (chance `(127 − clvl)/127`), slows you 4–7
   turns and stuns you unless you have protection from stun
   (`project_player_handler_GRAVITY()`); inertia slows; time drains stats or experience.
   Don't stay.

### Invisible attacker

1. Signs: "It hits you", damage with no visible monster, being drained, doors opening.
2. Detect: Reveal Monsters / Detect Invisible scroll or staff / Rod of Detection;
   see invisible (True Seeing potion, Sense Invisible spell, gear) or telepathy.
3. Ghosts pass walls and drain XP/stats; poltergeists are fast but weak.
4. If you can't detect it and it out-damages your regeneration → leave the level.
5. Area attacks (balls, *Dispel Undead/Evil*, *Holy Word*) hit unseen monsters.

### Stuck in a dead end

1. Check for a secret door: stand next to each dead-end wall (searching is automatic
   with light).
2. Dig out: Wand of Stone to Mud / *Turn Stone to Mud* / tunnel (granite needs digging
   skill > 40).
3. Phase Door goes through walls (≈10 grids); Teleportation further.
4. A dead end is also a good **fighting** spot against packs: only one can reach you.

### Trapped behind rubble

1. Tunnel it (`T`): rubble falls at `8 × digging skill / 1600` per turn — a few turns
   with a digger, many without. Passable rubble can be walked through.
2. Stone to Mud removes it in one zap.
3. If monsters are coming and the rubble won't fall in time → phase.

### Drained of experience

1. Not urgent. While below your max, kills add full XP to current exp and 10% to max
   (`player_exp_gain()`), so you climb back by playing.
2. Restore: Potion of Restore Life Levels (Alchemist), *Remembrance* (Priest 20),
   *Revitalize* (Druid 35), *Unholy Reprieve* (Blackguard 34), Rod of Restoration, Potion
   of Life.
3. Prevention: hold life; avoid nether/time breathers and ghosts without it.

### Being stolen from

1. FACT (`src/mon-blows.c`): `EAT_GOLD` takes `au/10 + 1d25` (more on big purses) unless
   you pass a save of `adj_dex_safe + clvl` % (impossible while paralysed); `EAT_ITEM`
   takes an item (never an artifact; one of a stack). The thief then **teleports 45
   grids away** carrying the loot (it drops it when killed).
2. Prevention: kill thieves at range; don't let them touch you; high DEX and level help.
3. After a theft: the thief is 45 grids away with your item — detect and hunt it if the
   item matters, otherwise move on.

---

## Things to try (untested)

1. **Phase-door threat scoring.** Before phasing, enumerate all legal grids at distance
   9–11 (that's where Phase lands) and estimate danger at each, like the borg's
   `borg_caution_phase()` but with the exact distance band; phase only if the expected
   danger drops by half.
2. **Teleport landing prediction.** For Teleportation, landing distance is 45–74 grids
   from the middle of a full level and 85–140 from an edge. Teleporting from a corner
   throws you across the map; from the centre, somewhere mid-range. Test whether
   "move toward a corner, then teleport" lands you farther from a pursuing pack.
3. **Hold Monster as a universal escape.** If Hold Monster really has no level save
   (only `NO_HOLD`), a Rod of Hold Monster is a reusable "skip the next 3–8 turns of that
   monster". Test against uniques without `NO_HOLD` and log success rates.
4. **Slow Monster stacking.** Slow is `−2 × ceil(timer/10)` up to −10; repeated wand
   zaps add timer (stacking `INCR`). Test whether two zaps reliably take a +10 monster to
   speed parity.
5. **Decoy Teleport Level trap.** Rangers: remember that your decoy eats your own Teleport
   Level. Test whether keeping a decoy out on DL40+ protects more (from monster
   teleport-level/teleport-away) than it costs.
6. **Terror mushroom as an emergency exit for melee classes.** +10 speed for 25+5d5
   turns with no scroll needed; test as the "blind and confused" escape compared to a
   teleport staff with nonzero fail.
7. **Destruction as unique removal.** *Destruction* deletes uniques from the level (they
   can return later). Test whether it's a reliable way to "skip" a dangerous unique fight
   when no Teleport Other is available.
8. **Glyph-and-shoot.** Priests at clvl 20+ can stand on a glyph in a corridor and
   shoot/cast at melee-only monsters; a level-30 monster breaks it only ~5% per attempt.
   Measure XP/hour and risk.
9. **Door-wall panic button.** Mage *Door Creation* around you, then rest or heal while
   non-door monsters are stuck; test against hounds and animals.
10. **Deep Descent as an "escape forward".** When the current level is lost but you are
    well-prepared for `max_depth + 5`, Deep Descent's 4–7 ticks may be safer than recall's
    15–34. Compare survival in borg logs.
11. **HP threshold auto-tuning.** Log D_worst estimates and actual damage taken per
    monster turn; fit per-depth thresholds instead of fixed percentages.
12. **Two-source rule enforcement.** Test whether forbidding descent without two
    independent long-escape sources measurably reduces deaths at DL30–50.

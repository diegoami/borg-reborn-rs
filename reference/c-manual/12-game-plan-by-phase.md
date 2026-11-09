# 12. Game plan by phase

The whole game, from the town at character level 1 to the kill of Morgoth on dungeon level 100, cut into
eight phases. For each phase: what you are trying to achieve, the kit you need, the monsters that end
characters there (with numbers taken from `lib/gamedata/monster.txt`), a "go deeper" readiness test, what
to buy, what to farm, the common deaths, and the recall/home routine. The chapter ends with the
endgame rules (quests on DL99/100), fight plans for Sauron and Morgoth, and a critique of the C borg's
depth-readiness rules.

Contents

- [12.1 How to read this chapter](#121-how-to-read-this-chapter)
- [12.2 The shape of a winning game](#122-the-shape-of-a-winning-game)
- [12.3 Diving versus slow play](#123-diving-versus-slow-play)
- [12.4 Phase 0: Town and the first descent (CL1, DL0-1)](#124-phase-0-town-and-the-first-descent-cl1-dl0-1)
- [12.5 Phase 1: The first levels (DL1-5)](#125-phase-1-the-first-levels-dl1-5)
- [12.6 Phase 2: Early game (DL5-15)](#126-phase-2-early-game-dl5-15)
- [12.7 Phase 3: Early-mid game (DL15-30)](#127-phase-3-early-mid-game-dl15-30)
- [12.8 Phase 4: Mid game (DL30-40)](#128-phase-4-mid-game-dl30-40)
- [12.9 Phase 5: Late-mid game (DL40-60)](#129-phase-5-late-mid-game-dl40-60)
- [12.10 Phase 6: Deep game (DL60-98)](#1210-phase-6-deep-game-dl60-98)
- [12.11 Phase 7: Endgame — Sauron (DL99) and Morgoth (DL100)](#1211-phase-7-endgame--sauron-dl99-and-morgoth-dl100)
- [12.12 The recall and home routine](#1212-the-recall-and-home-routine)
- [12.13 Summary tables: kit by depth](#1213-summary-tables-kit-by-depth)
- [12.14 The C borg's readiness rules, and where the data disagrees](#1214-the-c-borgs-readiness-rules-and-where-the-data-disagrees)
- [Things to try (untested)](#things-to-try-untested)

---

## 12.1 How to read this chapter

Each phase section has the same parts:

| Part | Meaning |
|---|---|
| **Goals** | what this phase is for; when it is "done" |
| **Typical CL** | a rough character level band; ADVICE, not a rule |
| **Kit** | required (a missing item is a common cause of death) and desired |
| **Threats** | monsters that appear here and kill, with depth, speed and maximum damage computed from `monster.txt` |
| **Go-deeper test** | a checklist; pass it before moving to the next phase's depths |
| **Buy / farm** | what to spend gold on, what is worth staying for |
| **Common deaths** | how characters die here, and the counter |
| **Recall routine** | how to use town in this phase |

Number conventions used below (all FACT, see [Chapter 1](01-core-mechanics.md) and
[Chapter 5](05-elements-and-status.md) for derivations):

- **Speed.** Energy per game turn comes from `extract_energy[]` in `src/game-world.c`: normal speed (110)
  gives 10, +10 gives 20, +20 gives 30, +30 gives 38, +40 gives 42. A monster's actions per player
  action is `energy(monster) / energy(player)`. A +10 monster against a +0 player acts twice per player
  turn; Morgoth (+30) against a +20 player acts 38/30 = 1.27 times.
- **Breath.** Monster breath damage is `current hp / divisor`, capped (`src/mon-spell.c:breath_dam()`,
  divisors and caps in `lib/gamedata/projection.txt`): acid/elec/fire/cold divide by 3 cap 1600; poison
  by 3 cap 800; light/dark/sound/shards/disenchantment by 6 cap 500; nexus by 6 cap 400; nether/chaos
  by 6 cap 600; gravity 3/200, inertia 6/200, force 6/200, time 3/150, plasma 6/150.
- **Resists.** One level of resistance to a base element (acid, elec, fire, cold, poison) multiplies
  damage by 1/3; a temporary resist stacks with an item resist to 1/9
  (`src/player-calcs.c` raises `res_level` by one for a timed resist, up to 2;
  `src/project-player.c:adjust_dam()` applies `numerator/denominator` once per level). For the
  "high" elements the factor is 6/(8+1d4), so at worst 6/9 (two-thirds) and at best 6/12. Immunity is 0.
  Vulnerability is ×4/3. Below, a breath is written "unresisted / resisted / double-resisted".
- **Breath falls off with distance.** Breath is an arc centred on the breather with "source diameter" 4,
  or 6 for `POWERFUL` monsters (`src/effect-handler-attack.c:effect_handler_BREATH()`); damage at
  distance *d* is `dam × diameter / (d+1)`, capped at full (`src/project.c:project()`). So a non-powerful
  breath is full strength up to 3 grids away and half strength at 7. Ball spells, by contrast, are
  centred on you.
- **Spell frequency.** `spell-freq:N` means a 100/N percent chance per monster turn
  (`src/mon-init.c`, `src/mon-attack.c:monster_can_cast()`), **doubled** when the monster is at its
  preferred range (`best_range`), halved when you are taunting. Spellcasters with more than 24% frequency
  prefer to sit 3 grids further back than their minimum range; frequent breathers prefer point blank
  (`src/mon-move.c:get_move_find_range()`).
- **Melee.** Physical damage after armour is `dam − dam × min(AC,240) / 400`
  (`src/mon-attack.c:adjust_dam_armor()`), so AC 100 removes 25% and AC 240 removes 60% of the damage of
  `HURT` and `SHATTER` blows. Many elemental and status blows are not reduced by AC.

A monster's maximum "per player turn" damage is approximately
`max(melee total, best spell or breath) × energy(monster)/energy(you)`. That is the number to compare with
your current hit points. [Chapter 15](15-notes-for-the-borg.md) turns it into a formal rule.

---

## 12.2 The shape of a winning game

### What the game requires (FACT)

The official manual is blunt: "Your one and only mission is to slay Morgoth on dungeon level 100. In
order to get there, you need to go down a lot of stairs and kill Sauron on dungeon level 99"
([guide](https://angband.readthedocs.io/en/latest/guide.html), vendored at
[user-manual/guide.rst](user-manual/guide.rst)). In code terms the game has exactly two quests,
`lib/gamedata/quest.txt`:

```
name:Sauron   level:99   race:Sauron, the Sorcerer      number:1
name:Morgoth  level:100  race:Morgoth, Lord of Darkness  number:1
```

When the last incomplete quest is completed, `src/player-quest.c:quest_check()` sets
`p->total_winner` and prints "You have won the game!". Nothing else is required: no other unique, no
character level, no artifact.

### Why you must still get strong

The two questors are, by the data, the strongest monsters a character must face (Section 12.11): Sauron
has 8000 hit points, speed +20 and casts on half his turns at spell power 130; Morgoth has 20000 hit
points, speed +30, and an unresistable mana ball of up to 600 damage. Getting to the point of fighting
them requires character level, hit points, speed and a full set of resistances, and those come from:

- **Experience.** A kill gives `mexp × monster level / your level` experience
  (`src/mon-util.c`, `new_exp = ((long)mon->race->mexp * mon->race->level) / div`). Deeper monsters have
  both higher `experience:` and higher `level`, so experience per kill grows roughly with the square of
  depth. The level table (`src/player.c:player_exp[]`, multiplied by the race+class experience factor)
  needs 5,000,000 × expfact/100 experience for CL50: 5.4 million base points from CL1.
- **Items.** Floor items are generated at the level depth. Monster drops use
  `MAX((monster level + depth)/2, monster level)` (`src/mon-make.c`, drop code): the drop is at least as
  good as the monster's own level and better when the monster is out of depth for the level. Speed,
  high resists and stat potions are simply not generated at shallow depth (for example *Healing* potions
  are allocated from DL40, Life from DL60, Rings of Speed from DL75, Potions of Strength from DL30: see
  `lib/gamedata/object.txt` and [Appendix A6](appendices/A6-object-kinds.md)).

### The depth curve (ADVICE)

A healthy winning game has three visible slopes:

1. **Steep early (DL1 to ~DL30).** The shallow dungeon has nothing you need for the endgame. A
   character that is careful about specific killers (Section 12.5-12.7) can descend about one level per
   one or two character levels. The official guide's "Try a Different Strategy" section recommends
   playing "as though the iron-man stairs setting is in effect" to learn how fast levelling can be.
2. **A shoulder (~DL30 to ~DL45).** Here the character collects the "mandatory" defensive kit: free
   action, see invisible, the four base resists, then poison resistance, plus escapes that are
   reliable. The kit is usually assembled from the dungeon and the Black Market, so time spent here is
   time spent gathering. Do not skip the shoulder by stair-diving: depth 40+ monsters (Section 12.9) do
   500-800 damage per breath to an unresisted character.
3. **A long climb to the bottom (~DL45 to DL98)** gated by speed and high resists. Dive as fast as the
   kit allows; most of the game time is spent between DL40 and DL80 hunting speed items, `*Healing*`
   potions and resistances.

A character is "on curve" when the go-deeper tests of its current phase pass. A character that passes
the test of the *next* phase too should dive faster.

### How long does it take?

The C borg's measurements give the scale of automatic play: on the tracker's harness a borg run of
400,000 engine (game) turns typically reaches only the low dungeon levels, and the plan's objective is
"alive at turn T having reached DL D (for example dlvl 10 at 400,000 engine turns)"
([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)). One player turn at normal speed is 10 game turns
(`src/game-world.c`: the world is processed every 10 game turns; a +0 player gets 10 energy per game
turn and needs 100). Human game lengths are not recorded in the source; this chapter makes no claim
about them.

---

## 12.3 Diving versus slow play

### The mechanics that favour diving (FACT)

| Mechanic | Source | Consequence |
|---|---|---|
| Experience per kill ∝ `mexp × mlevel / plevel` | `src/mon-util.c` | the same number of kills gives far more experience deeper |
| Drop level ≥ monster level, floor objects at depth | `src/mon-make.c`, `src/obj-make.c` | endgame items exist only deep |
| Connected stairs (default on) | `birth_connect_stairs` in `src/list-options.h`, `src/cmd-cave.c:do_cmd_go_down()` sets `create_up_stair` | you arrive on a staircase back the way you came, a guaranteed first escape |
| Level feelings (default on) | `birth_feelings`, `src/cmd-cave.c:display_feeling()` | the danger digit is known on arrival; you can leave at once |
| Deep Descent | `src/effect-handler-general.c:effect_handler_DEEP_DESCENT()` | after 4-7 world ticks (`3 + 1d4`), teleports you to **max depth + 5**, not current depth + 5 (see warning below) |
| Word of Recall | `effect_handler_RECALL()` | 15-34 world ticks later, to your recall depth (normally your max depth) |

**Deep Descent warning (FACT).** The target is computed from `player->max_depth`, both when the scroll is
read and when it fires (`src/game-world.c`), and the effect text is "teleports you up to five dungeon
levels lower than the lowest point you have reached so far" (`src/list-effects.h`). Reading it on DL5
with a max depth of 30 sends you to DL35. It never takes you past a quest level
(`src/player-util.c:dungeon_get_next_level()` stops at the first quest level), so it cannot skip DL99.

### The mechanics that punish diving (FACT)

- Monster damage scales with depth much faster than character hit points (compare the threat tables in
  each phase below).
- Out-of-depth monsters exist: monster generation can boost the level used to pick a monster
  (`mon-gen:ood-chance:25` and `ood-amount:10` in `lib/gamedata/constants.txt`).
- Many uniques come with escorts (`friends:` lines in `monster.txt`), and a few (`FORCE_DEPTH`, such as
  the questors) are never generated shallower than their native depth (`src/mon-make.c`).

### The guide's argument, and the risk-per-move model (ADVICE)

The official guide puts it well: "If you take enough low-probability chances of death, you'll never
survive to fight Sauron... It's the trick to extremely fast dives: the fewer moves you make, the less
chance any one of them will be fatal, even if on average, your individual moves are riskier than in
slower play." ([guide.rst](user-manual/guide.rst), "On Bad Luck".)

Model: if each player turn at a depth carries a small death probability *p(depth, kit)*, the total risk
of a phase is the sum over turns. Slow play reduces *p* but multiplies the number of turns; diving
raises *p* but cuts turns. The optimum is not "as deep as possible" but "the deepest depth at which
*p* stays below a threshold you accept, given your kit". The go-deeper tests in this chapter are a
hand-made estimate of that threshold. A borg can estimate *p* directly (see
[Chapter 15](15-notes-for-the-borg.md), rules DANGER-*).

### A recommended policy: "measured diving"

1. On arrival, read the level feeling. The danger digit (the first number of the `LF:` indicator) runs
   from 1 "Omens of death haunt this place" to 9 "This seems a quiet, peaceful place"; **lower is more
   dangerous** (`src/generate.c:calc_mon_feeling()` thresholds the level's monster rating divided by
   depth; texts in `src/cmd-cave.c`). If it is 1-3 and you are near the edge of your kit, take the
   stairs you arrived on.
2. Detect (monsters, then map/stairs). If a monster on the level fails the "one-turn kill" test
   (Section 12.1: its per-turn damage exceeds about half your hit points, or it has an attack you do
   not resist that can disable you), leave.
3. Otherwise, fight what is profitable, pick up what is valuable, and take the first down staircase
   once the level is no longer profitable. Do not "clear" levels.
4. Stop diving at the first depth where a go-deeper test fails; farm there until it passes.

This is close to what the official guide's later sections describe ("Just enter the level, immediately
do your detection routine... assess the immediate threat, and use the stairs immediately if you want
to").

---

## 12.4 Phase 0: Town and the first descent (CL1, DL0-1)

### Goals

Leave town in a few minutes of play with a light, food, a few flasks of oil, some Cure Light Wounds,
Phase Door, and Word of Recall if gold allows. Starting gold is `player:start-gold:600`
(`constants.txt`) plus 50 per unspent point-buy point, minus the value of the start kit when
`birth_start_kit` is on (`src/player-birth.c`). With `birth_no_selling` on (the default) gold drops in
the dungeon are larger but stores pay nothing for your items, so buy deliberately. See [Chapter 2](02-character-creation.md) and [Chapter 11](11-town-and-shopping.md).

### Store facts relevant to the first trip (FACT, `lib/gamedata/store.txt`)

| Store | Always stocked | Often stocked (normal) |
|---|---|---|
| General Store (1) | Ration of Food, Wooden Torch, Flask of Oil, Cloak, Shovel, Pick, iron shots, arrows, bolts | wine, slime molds, Hard Biscuits, mushrooms of Stoneskin and Purging |
| Alchemist (5) | **Word of Recall, Phase Door, Remove Curse, Cure Light Wounds** | Cure Serious, Cure Critical, Magic Mapping, Detect Invisible, Blessing, Holy Chant, Resist Heat, Resist Cold, Neutralize Poison, Berserk Strength, Boldness, Recharging, Remove Hunger, Monster Confusion, Restore Life Levels |
| Magic shop (6) | — | Wands of Magic Missile, Stinking Cloud, Stone to Mud, Light, Slow/Confuse Monster; **Staff of Teleportation**, Mapping, Detect Evil, Detect Invisible, Light; Rings of Protection, Open Wounds, Feather Falling, Searching; Amulets of Slow Digestion, Resist Acid |
| Bookseller | the town books of each realm | — |

Note what is **not** in the town stores: Scrolls of Teleportation, Teleport Level, Deep Descent,
Potions of Healing and better, Free Action, See Invisible. Those come from the dungeon or the Black
Market (random stock). The Staff of Teleportation is the only reliable town source of a long-range
escape, and it is "normal" stock (not always present).

### Shopping list for CL1 (ADVICE)

| Item | Why | Quantity |
|---|---|---|
| Flasks of Oil | thrown, they explode for fire damage "even against creatures resistant to fire" (`object.txt` description; `EXPLODE \| THROWING`); the best early ranged attack for every class | 10-20 |
| Potion of Cure Light Wounds | heals 20 hp, cures blindness, reduces cuts and confusion (`object.txt`: `HEAL_HP` 20) | 3-5 |
| Scroll of Phase Door | short teleport | 5-10 |
| Wooden Torches | light intensity 2 (`LIGHT_2`, `BURNS_OUT`): lights adjacent grids only | 2-3 spare (or buy a Lantern later: `LIGHT_3`, `TAKES_FUEL`) |
| Food | rations | 3-5 |
| Soft armour, a shield, a sling or bow if your class can use one | AC and ranged damage | as gold allows |
| Word of Recall | lets you come back up from depth and return to max depth | 1 if affordable (not essential on DL1-4) |

Spellcasters: buy the first town book of your realm if the start kit did not give it. Mages want *Magic
Missile*; priests *Bless* and *Call Light*; necromancers *Nether Bolt* (and they want darkness, see
[Chapter 14](14-class-playbooks.md)).

### Go-deeper test (to DL1)

- [ ] a light source with fuel
- [ ] food for a few days
- [ ] at least 2 healing potions and 3 Phase Door (or a mage with Phase Door spell at CL2+)
- [ ] a ranged option (thrown oil, sling/bow, or Magic Missile)

---

## 12.5 Phase 1: The first levels (DL1-5)

### Goals

Reach CL5-10, collect gold for Cure potions and Word of Recall, learn your class's first spells or
fighting rhythm, and learn which early monsters to avoid.

### Typical CL

1-10. Many players take one level per character level here; warriors can go faster.

### Kit

| Required | Desired |
|---|---|
| light, food, CLW ×3+, Phase ×5+ | a bow or sling and ammunition; a second weapon or a better one |
| some ranged attack | Word of Recall ×1 by DL4-5 |

### Threats (FACT from `monster.txt`)

| Monster | Depth | Speed | HP | Attack | Why it kills |
|---|---|---|---|---|---|
| Grip / Fang, Farmer Maggot's Dog | 2 | +20 | 25 | BITE 1d4 | two moves per player turn; deadly for a CL1 mage with 10 hp |
| cave spider (groups) | 2 | +20 | 7 | BITE 1d4 | swarms; fight in a corridor |
| floating eye | 1 | +0, never moves | 11 | GAZE:PARALYZE | melee it without Free Action and you can be paralysed repeatedly. Never melee; shoot it or walk away |
| poltergeist | 3 | +30, invisible, pass-wall | 6 | TOUCH:TERRIFY | harmless but disturbing; ignore |
| Sméagol | 3 | +30, invisible | 400 | BITE 1d4, steals gold | you cannot catch him; protect gold by spending it |
| kobold archer | 4 | +0 | 24 | ARROW at 1-in-2 | ranged damage while you approach |
| kobold shaman | 3 | +0 | 11 | WOUND at 1-in-5 (power 3 → 2d5) | early caster |
| apprentice | 2 | +0 | 15 | BLIND, CONF, BLINK, MISSILE at 1-in-12 | confusion and blindness early |
| Bullroarer the Hobbit | 5 | +20 | 60 | 2× HIT 1d8 (16 per turn, 32 per player turn) | the first unique that can kill a CL5 character; good drop |
| breeders: giant white mouse (1), worm masses (1-5), giant white louse (3, +20) | 1-5 | | | | waste time and food; leave the level if they spread |

### Go-deeper test (to DL5)

- [ ] CL ≥ 5 (casters 8+)
- [ ] maximum hit points ≥ ~40 (Bullroarer can deal 32 per player turn)
- [ ] CLW ×5, Phase ×5, a ranged attack, Word of Recall ×1

### Buy / farm

Buy CLW, Phase, oil flasks. Farm nothing: DL1-4 drops are poor (Section 12.2). Kill packs you can
handle in corridors; take the first stairs down when the level feels empty.

### Common deaths and counters

| Death | Counter |
|---|---|
| Fang/Grip on a CL1-3 caster | stand in a corridor, throw oil, use Phase Door; or wait to CL4+ before DL2 |
| melee with a floating eye | never melee it without Free Action |
| a pack in the open | retreat to a corridor so only one can attack |
| Bullroarer at CL<5 | leave the level when he is detected; return at CL8+ with oil flasks |

### Recall routine

Walk up by stairs until you can afford Word of Recall. Once you have one, read it on the dungeon level
when your pack is full or you are out of potions, buy a replacement first thing in town, then read it
again in town to go back to your max depth.

---

## 12.6 Phase 2: Early game (DL5-15)

### Goals

Reach CL15-22, a stack of Cure Light/Serious potions, Word of Recall always in pack, a reliable ranged
attack, and the first piece of the defensive kit (Free Action is the first important one; see Phase 3).

### Typical CL

8-22.

### Kit

| Required | Desired |
|---|---|
| Word of Recall ×1 (always), CLW/CSW ×5+, Phase ×10 | Cure Critical Wounds (alchemist "normal" stock; heals 60, cures poison, stun, cuts, confusion, amnesia, blindness) |
| a launcher with 40+ missiles, or class attack spells | Staff of Teleportation or Scrolls of Teleportation |
| light radius 2 | Lantern (intensity 3 = radius 2: `LIGHT_3`) |
| | Scroll of Magic Mapping, Detect Invisible |
| | Potion of Resist Poison or Neutralize Poison |

### Threats (FACT)

| Monster | Depth | Speed | HP | Damage | Notes |
|---|---|---|---|---|---|
| Wormtongue, Agent of Saruman | 8 | +0 | 250 | BO_COLD, BA_POIS, SLOW, TRAPS, HEAL at 1-in-5 | creates traps and slows; good drop |
| Lagduf, the Snaga | 8 | +0 | 190 | 4 blows, 38 max | orc pack leader |
| Brodda, the Easterling | 9 | +0 | 210 | 4× 1d12 (48) | |
| Grishnákh, the Hill Orc | 10 | +0 | 230 | 4 blows (44) | steals |
| Orfax, Son of Boldor | 10 | +10 | 120 | TELE_TO, CONF, SLOW, S_MONSTER at 1-in-4 | summons |
| Boldor, King of the Yeeks | 13 | +10 | 180 | BLIND, SLOW, TPORT, summons at 1-in-3 | |
| Golfimbul, the Hill Orc Chief | 12 | +0 | 240 | 44 | escorted |
| Ufthak of Cirith Ungol | 14 | +0 | 320 | 48 | |
| Mughash the Kobold Lord | 7 | +0 | 150 | 3× 1d10 | kobold escort |
| blacklock mage | 10 | +10 | 39 | BA_POIS (max 32) at 1-in-5, BLIND, CONF | |
| ironfist priest | 12 | +10 | 39 | WOUND (power 12 → 8d5 = 40) at 1-in-5 | per-turn 80 at +0 |
| druid | 13 | +0 | 78 | BO_FIRE (power 13: 4+9d8 → max 76) at 1-in-3, HOLD (paralyse) | one of the first paralysers |
| illusionist | 13 | +10 | 54 | HOLD, BLIND, CONF, SLOW at 1-in-3 | paralysis without Free Action |
| homunculus | 15 | +0 | | PARALYZE blow | |
| spotted / ochre jelly | 12-13 | +10 | 59 | touches 34 | ochre jelly moves |
| baby dragons (blue 9, multi-hued 11) | 9-11 | +0 | 88-114 | breath 29-38 unresisted | low danger, good experience |
| wolf (10), warg (14) packs | | +10 | 21-36 | bites | packs of fast animals: fight in corridors |
| orc archer | 13 | +0 | 66 | ARROW at 1-in-3 | in orc groups |
| wererat | 15 | +0 | 90 | WOUND (50), BO_COLD, BA_POIS | |
| moaning spirit | 12 | +10, invisible | 23 | SCARE, TPORT; LOSE_DEX touch | first invisible ghost that drains stats |
| giant fruit fly (10, +10), giant black louse (9, +10) | | | | breeders | leave the level early |

### Go-deeper test (to DL15)

- [ ] CL ≥ 15 for fighters, ≥ 18-20 for pure casters
- [ ] maximum hit points ≥ 100
- [ ] Word of Recall in pack, plus money for a spare
- [ ] CLW/CSW ×5+ (the tougher early uniques do 40-50 per turn)
- [ ] at least one way to deal with paralysis: Free Action, or kill paralysers at range, or a high
      saving throw plus caution. **Free Action becomes required by DL20** (Phase 3)
- [ ] a way to see or detect invisible monsters is desirable (moaning spirit at DL12, ghosts later)

### Buy / farm

- Buy: CCW when available, Phase Door, Magic Mapping, flasks of oil, arrows/shots, a Lantern, armour
  pieces for AC. Mages: book 2 (*Attacks and Knowledge*) and book 3 (*Magical Defences*, which has
  *Teleport Self* at CL7).
- Farm: orc uniques (Grishnákh, Golfimbul, Ufthak, Lagduf) for drops; orc packs for experience if you
  can fight them in a corridor.

### Common deaths and counters

| Death | Counter |
|---|---|
| paralysed by a druid/illusionist/homunculus or a floating eye | kill casters at range, never melee eyes; buy/find Free Action |
| a pack of wolves or wargs in a room | corridor fighting; Phase Door if surrounded |
| a summoner (Orfax/Boldor) filling the room | kill quickly or leave |
| poison and no cure | carry CCW or Neutralize Poison |
| exhausted ammunition and potions in a long fight | recall earlier; the cost of a recall is one scroll |

### Recall routine

Recall when: out of CLW/CSW; fewer than 3 Phase; pack full of things worth keeping; an item needs
identifying by use in safety. In town: restock potions/scrolls first, then gear. Keep a spare Word of
Recall at home.

---

## 12.7 Phase 3: Early-mid game (DL15-30)

### Goals

Reach CL25-30, acquire **Free Action** and **See Invisible**, the first base resists (fire and cold
first), a reliable long-range escape (Staff of Teleportation or Scrolls of Teleportation, or the mage
*Teleport Self* / rogue *Teleport Self* spells), and a damage output that kills orc uniques and trolls.

### Typical CL

18-32.

### Kit

| Required (by the end of the phase) | Desired |
|---|---|
| **Free Action** by DL20-25 | resist poison (temporary, from potions) |
| See Invisible or telepathy by DL25-30 | Scrolls of Teleport Level (dungeon only) |
| Teleportation (staff/scrolls/spell) ×5 uses | the four base resists |
| CCW ×5+, Phase ×10+ | +10 to-dam on a melee weapon or a good launcher |
| resist fire and resist cold (item or potions) | |

### Threats (FACT)

| Monster | Depth | Speed | HP | Damage | Notes |
|---|---|---|---|---|---|
| Old Man Willow | 19 | +0 | 1000 | TOUCH:PARALYZE ×2, TELE_TO, HOLD | paralysis plus teleport-to |
| Shagrat / Gorbag, the Orc Captains | 19 | +0 | 400 | 4 blows, 78 | orc escorts |
| Bolg, Son of Azog | 20 | +10 | 500 | 4× 3d6 (72), 144 per player turn at +0 | a top killer in this band: fast, tough |
| blood falcon | 20 | +30 | 6 | 64 melee per monster turn, about 240 per player turn at +0 | a glass cannon: kill it first, at range if possible |
| dark dwarven lord | 20 | +10 | 144 | BO_FIRE 78 at 1-in-5 | |
| Uglúk / Lugdush, the Uruks | 21-22 | +0 | 640-720 | 72-78 | |
| Azog, Enemy of the Dwarves | 23 | +10 | 900 | 3× 5d5 (75), 150 per player turn | |
| brigand | 23 | +10 | 75 | 64 + SHOT | steals |
| Sangahyando / Angamaitë of Umbar | 24 | +0 | 800 | 4× 4d6 (96), CONF, SLOW | come together |
| banshee | 24 | +10, invisible | 27 | EXP_20 touch, DRAIN_MANA | see invisible |
| vampire bat | 24 | +10 | 50 | EXP_40 bites | experience drain |
| carrion crawler | 25 | +0 | 130 | 2× STING:PARALYZE 2d6 | in groups; **Free Action required** |
| nruling | 25 | +10 | 50 | BO_FIRE 75 at 1-in-2 | |
| ghoul | 26 | +0 | 83 | PARALYZE claws, HOLD | in packs; **Free Action required** |
| Mîm, Betrayer of Turin | 27 | +10 | 1100 | BA_ACID (max 96) at 1-in-6 | |
| vibration / nexus hounds | 27 | +0 | 138 | sound (stun) / nexus breath | packs |
| basilisk | 28 | +10 | 310 | BR_POIS 103/34/11, GAZE:PARALYZE | a top killer in this band |
| mind flayer | 28 | +0 | 132 | BRAIN_SMASH (12d15 = 180 plus slow/confuse/paralyse/blind) at 1-in-8 | saving throw negates the whole spell (`src/mon-spell.c`: save → no effect) |
| black knight | 28 | +10 | 165 | WOUND 90 | |
| Draebor, the Imp | 28 | +10, invisible | 520 | TELE_TO, TELE_LEVEL, TELE_AWAY, BLIND, CONF | |
| Beorn, the Shape-Changer | 28 | +0 / +10 as bear | 1400 | bear form 90 melee | |
| Bert, Bill and Tom, the Stone Trolls | 30 | +10 | 1100 each | 102 each, BOULDER | appear together; 600 per player turn combined at +0 if all reach you |
| young dragons (blue 29, red/black/gold 31, multi-hued 32) | | +0 | 237-281 | breath 79-93 unresisted | young MHD breathes five elements |
| giant army ant (30), killer slicer beetle (30) | | | | 48-112 melee | |

### Go-deeper test (to DL30)

- [ ] CL ≥ 25 (casters 28+)
- [ ] maximum hit points ≥ 200
- [ ] **Free Action**
- [ ] **See Invisible** (or telepathy)
- [ ] resist fire and resist cold, at least from potions
- [ ] Teleportation ×5 uses, Phase ×10, CCW ×5
- [ ] a way to kill or avoid Bolg and Azog (they are fast; they are the most common unique killers in
      this band by the per-turn numbers above)

### Buy / farm

- Buy: CCW, Staff of Teleportation (magic shop), Magic Mapping; potions of Resist Heat/Cold; Black
  Market Free Action or See Invisible if it appears and you can afford it.
- Farm: orc/troll packs for experience; uniques with good drops (Shagrat, Gorbag, Bolg, Azog, Uglúk,
  Lugdush). This band is also where most characters get their first ego weapon or launcher.

### Common deaths and counters

| Death | Counter |
|---|---|
| paralysed by carrion crawlers or ghouls | Free Action, period. Without it, leave the level on sight |
| Bolg or Azog catching you | +10 speed monsters: never run in the open; Phase/Teleport early; fight in corridors |
| blood falcon in melee range | it dies to almost any hit; kill it first |
| basilisk breath and gaze | Free Action, poison resist potion, kill at range |
| the stone troll trio | fight them in a corridor one at a time; or leave |
| experience drain from vampire bats and banshees | not fatal; carry Restore Life Levels, find Hold Life later |

### Recall routine

From here on, recall every trip rather than walking. Keep two Word of Recall (one in pack, one at home).
Use the home to store the swap items you are collecting: a resist-fire item and a resist-cold item can be
swapped in when a red or white dragon is detected.

---

## 12.8 Phase 4: Mid game (DL30-40)

### Goals

Reach CL30-35, complete the four base resists (acid, electricity, fire, cold), keep Free Action and
See Invisible, and start assembling poison resistance, confusion protection and blindness protection.
Find your first +speed if possible.

### Typical CL

28-38.

### Kit

| Required (by DL40) | Desired |
|---|---|
| rAcid, rElec, rFire, rCold (permanent) | **resist poison** (required by DL40-45) |
| Free Action, See Invisible | protection from confusion, protection from blindness |
| Teleportation ×10 uses or a staff, Teleport Level ×2 | +5 to +10 speed |
| CCW ×10, some Potions of Healing (DL30+ drops: `HEAL_HP 300+m35`) | Restore Mana (casters) |
| | Hold Life |

### Threats (FACT)

| Monster | Depth | Speed | HP | Damage | Notes |
|---|---|---|---|---|---|
| ghost | 31 | +10, invisible, pass-wall | 59 | HOLD, BLIND, DRAIN_MANA, EXP_20, INT/WIS drain | |
| young multi-hued dragon | 32 | +0 | 281 | five breaths, 93/31/10 | needs all base resists |
| nexus quylthulg | 32 | +0, invisible | 65 | TELE_AWAY at 1-in-1 | splits you from your escape |
| lich | 34 | +0 | 264 | BRAIN_SMASH 180 at 1-in-4 | high saving throw helps |
| master thief | 34 | +20 | 99 | 64 melee, steals | |
| Ulfang the Black | 34 | +10 | 1000 | 4× 5d5 (100), summons kin | |
| Lokkak, the Ogre Chieftain | 35 | +10 | 1500 | 3× 6d6 (108) | |
| gravity / impact / inertia hounds | 35 | +0 | 193 | gravity 64 (unresistable: blinks and slows you), force, inertia (slows) | packs |
| Lorgan, Chief of the Easterlings | 36 | +10 | 1800 | 120 melee, TELE_TO | |
| dúnadan of Angmar | 36 | +10 | 246 | WOUND 120 at 1-in-3, HOLD | |
| trapper | 36 | +10, invisible | 528 | 2× PARALYZE (15d1) blows | |
| Castamir the Usurper | 38 | +10 | 880 | bolts at 1-in-2 | |
| death knight | 38 | +10 | 528 | WOUND 120 | |
| will o' the wisp | 38 | +20, invisible | 176 | WOUND 120 at 1-in-2 | 360 per player turn at +0 |
| 6- and 7-headed hydras | 39-40 | +10 | 550-650 | BR_POIS 183-216 (7-headed also fire) | poison resist |
| mature white/blue/green dragons | 39 | +5 | 700 | breath 233 unresisted | |
| sorcerer | 40 | +20 | 457 | BA_FIRE max 150 at 1-in-2, summons | 450 per player turn at +0 |
| death quasit | 40 | +20, invisible | 387 | WOUND 130 | |
| giant roc | 40 | +0 | 560 | 336 melee | |
| minotaur | 40 | +20 | 550 | 72 melee, ~216 per player turn | |

### Go-deeper test (to DL40)

- [ ] CL ≥ 30
- [ ] maximum hit points ≥ 300
- [ ] all four base resists permanent
- [ ] Free Action, See Invisible
- [ ] Teleportation ×10 uses (staff counts), Teleport Level ×2, CCW ×10
- [ ] at least one of: resist poison, +5 speed, protection from confusion. Without rPois, treat
      hydras and green/multi-hued dragons as "leave the level" monsters

### Buy / farm

- Buy: CCW in bulk, Black Market stat potions when affordable, Staff of Teleportation spares.
- Farm: experience on trolls and giants (high `mexp`, melee only); drops from Lorgan, Ulfang,
  Castamir, Lokkak. Potions of Strength/Intelligence/etc. begin to be generated at DL30
  (`object.txt` alloc `30 to 100`).

### Common deaths and counters

| Death | Counter |
|---|---|
| a breath of an element you do not resist | complete the base four before DL35; carry Resist Heat/Cold potions |
| gravity hounds blinking you into a room | fight hounds at a corridor mouth; their breath is unresistable but capped at 200 |
| will o' the wisp / death quasit (fast invisible) | See Invisible, Teleport early |
| sorcerer summons | kill fast at range or Teleport away; do not stay in line of sight |
| nexus quylthulg teleporting you away from your escape route | kill it (65 hp) when seen |

### Recall routine

At DL30+ a typical trip is: recall in, detect, take stairs down when the level is finished, recall
out when restock is needed. Keep a "swap" kit at home or in the pack: resist-poison item, an amulet or
ring of the resist you lack.

---

## 12.9 Phase 5: Late-mid game (DL40-60)

### Goals

Reach CL35-45, get **poison resistance**, **+10 speed**, protection from blindness and confusion,
resist nether or Hold Life, and a stock of Potions of Healing; start collecting `*Healing*` potions
(allocated from DL40).

### Typical CL

33-45.

### Kit

| Required (by DL50) | Desired |
|---|---|
| resist poison | **+10 speed** (required by DL55-60) |
| protection from blindness and confusion | resist nether, Hold Life |
| Potions of Healing ×5+ | resist chaos, resist disenchantment |
| Teleport ×10, Teleport Level ×3 | telepathy |
| | Scrolls of Banishment (DL40+) and *Destruction* (DL40+) |

### Threats (FACT)

| Monster | Depth | Speed | HP | Max breath (unres/res/double) | Notes |
|---|---|---|---|---|---|
| Kavlax the Many-Headed | 41 | +10 | 1300 | fire/cold/acid/elec 433/144/48; chaos, nexus, shards, sound 216/144; gravity 200 | breathes at 1-in-4 |
| mature red dragon | 41 | +5 | 740 | fire 246/82/27 | |
| hellhound (packs) | 42 | +10 | 352 | fire 117/39/13; FIRE bites 3×3d12 | packs of fast fire breathers |
| multi-hued hound | 43 | +0 | 220 | five base breaths ~73 each | |
| mature multi-hued dragon | 43 | +5 | 1000 | 333/111/37 ×5 | |
| dread | 44 | +10, invisible, pass-wall | 263 | BO_NETH, HOLD, BLIND, CONF | groups |
| greater basilisk | 45 | +10 | 1010 | poison, dark, nexus; GAZE:PARALYZE ×2 | |
| lesser Balrog | 49 | +10 | 1760 | fire 586 | |
| Eöl, the Dark Elf | 49 | +20 | 2400 | BA_NETH (max 296) at 1-in-2 | |
| The Balrog of Moria | 50 | +20 | 3000 | fire 1000/333/111, POWERFUL | |
| ancient dragons (white, green, blue 50; red, black 51) | 50-51 | +10 | 1500-1560 | 500-520/~170/~57 | |
| death drake / great crystal drake | 50 | +10 | 1950 | nether 325/216; shards 325/216 | |
| time hound | 51 | +20 | 330 | time 110 (no resist; drains stats/experience) | |
| nether / plasma hounds | 51 | +10 | 330 | nether 55/36; plasma 55 | |
| Itangast the Fire Drake | 52 | +10 | 2200 | fire 733/244/81 | |
| ancient multi-hued dragon | 53 | +10 | 2400 | **poison 800/266/88**, four others 800/266/88 | the classic rPois check |
| Scatha the Worm | 53 | +10 | 2500 | cold 833/277/92 | |
| drolem | 54 | +10 | 2200 | **poison 733/244/81** | |
| dreadmaster, nether wraith | 54 | +10, invisible | 1000-1056 | nether bolts, HOLD, TELE_LEVEL | |
| The Phoenix | 54 | +10 | 3600 | fire 1200/400/133 | |
| dracolich / dracolisk | 55 | +10 | 3080 | cold or fire 1026/342/114; nether 513/342 or nexus 400/266 | |
| Shelob, Spider of Darkness | 55 | +10 | 3500 | poison 800/266/88 | |
| Ar-Pharazôn the Golden | 55 | +20 | 4000 | 256 melee, TELE_AWAY | |
| 9-headed hydra | 57 | +10 | 1200 | poison/fire 400 | |
| Baphomet the Minotaur Lord | 58 | +20 | 4700 | 512 melee per turn (×2 per player turn at +0) | melee monster |
| Fundin Bluecloak | 59 | +20 | 5000 | WOUND 190, 256 melee | |
| Saruman of Many Colours | 60 | +10 | 5000 | balls, summons, TELE_AWAY, at 1-in-2 | |

Read the poison column: an unresisted ancient multi-hued dragon or drolem breath (733-800) kills almost
any character below CL40. With rPois the same breath is 244-266, and 81-88 with a temporary resist on
top. This is why rPois is the gate between Phase 4 and Phase 5.

### Go-deeper test (to DL50, then DL60)

- [ ] CL ≥ 35 (DL50) / ≥ 40 (DL60)
- [ ] maximum hit points ≥ 400 (DL50) / ≥ 500 (DL60)
- [ ] resist poison, protection from blindness and confusion
- [ ] +5 speed (DL50) / +10 speed (DL60)
- [ ] Potions of Healing ×5 (DL50) / ×10 and some `*Healing*` (DL60)
- [ ] resist nether or Hold Life by DL55 (dracolich, death drakes, nether hounds, dreads)
- [ ] an answer to summoners: Teleport Other, Banishment, or "leave the level"

### Buy / farm

- Buy: Potions of Healing and Restore Mana from the Black Market when you can; CCW in bulk; stat
  potions.
- Farm: the ancient dragons (huge experience, good drops) *once* you resist their element twice
  over (item + potion); `*Healing*` potions start dropping here.

### Common deaths and counters

| Death | Counter |
|---|---|
| poison breath without rPois | rPois before DL45, no exceptions |
| fast invisible nether casters | see invisible + telepathy; Teleport early |
| ancient dragon breath at full hit points but no double resist | temporary resist potions before engaging; fight from range (breath halves at distance 7 for non-powerful breathers) |
| being slowed (gravity, inertia, SLOW spells) | +10 speed buffer; Potions of Speed |
| Saruman / summoners | Teleport Other, or leave |

### Recall routine

Recall trips now include a Black Market check every visit (stat potions, Healing, speed items). Store
unused swap items at home: a rPois amulet, a resist-nether item, a Free Action backup.

---

## 12.10 Phase 6: Deep game (DL60-98)

### Goals

Reach CL45-50, **+20 speed** (permanent), all high resists that matter (nether, chaos, disenchantment,
sound/shards, dark/light, nexus), telepathy, Hold Life, and a large potion stock for the endgame:
Healing, `*Healing*`, Life, Speed, Restore Mana. Kill or avoid the late uniques.

### Typical CL

42-50.

### Kit

| Required (by DL80) | Desired |
|---|---|
| +15 to +20 speed | +25 or more |
| rPois, rNether, rChaos, rDisen, rBlind, pConf | rSound, rShards, rNexus, rDark, rLight |
| FA, SI, ESP, Hold Life | |
| Healing ×10, `*Healing*` ×5+ | Life potions |
| Teleport and Teleport Level, Teleport Other | Banishment, Mass Banishment |

### Threats (FACT)

| Monster | Depth | Speed | HP | Max breath (unres/res/double) |
|---|---|---|---|---|
| Smaug the Golden | 62 | +10 | 4200 | fire 1400/466/155 |
| dreadlord | 62 | +10, invisible | 2640 | BA_NETH, BRAIN_SMASH, HOLD |
| werewolf of Sauron | 63 | +10 | 2200 | nether 366/244, dark 366/244 |
| chaos hound | 64 | +10 | 930 | chaos 155/103 |
| The Mouth of Sauron | 65 | +20, invisible | 7000 | BA_MANA (power 65 → max 425), BA_NETH, BO_PLAS, at 1-in-2 |
| Thuringwethil | 67 | +20 | 5000 | BRAIN_SMASH, BA_NETH |
| Tselakus, the Dreadlord | 68 | +20, invisible | 6500 | BA_NETH, BA_DARK, HOLD |
| gelugon | 69 | +20 | 3080 | cold 1026/342/114, shards 500/333 |
| hound of Tindalos | 69 | +20 | 480 | time 150, nether, disenchant |
| great storm/ice wyrms | 70 | +10 | 4200 | 1400/466/155 |
| great hell/bile wyrms | 73 | +10 | 4500 | 1500/500/166 |
| Omarax, the Eye Tyrant | 73 | +20 | 6500 | BA_MANA etc. at 1-in-2 |
| aether hound | 74 | +20 | 1230 | 18 breath types, base ones 410/136/45 |
| black reaver | 74 | +10 | 3960 | BA_MANA, BA_NETH, BRAIN_SMASH |
| Glaurung, Father of the Dragons | 75 | +20 | 7500 | fire 1600/533/177, poison 800/266/88 |
| Ungoliant, the Unlight | 75 | +20 | 13000 | dark 500/333, poison 800/266/88 |
| great wyrms of chaos/law | 75 | +10 | 3960 | chaos 600/400, disen/shards/sound 500/333 |
| pit fiend | 77 | +20 | 3520 | fire 1173/391/130, chaos 586/390, poison 800/266/88 |
| the Nazgûl (Uvatha 77 … Khamûl 84) | 77-84 | +10 | 4200-4900 | BLACK_BREATH and EXP_80 blows, HOLD, nether |
| Wiruin, the Maelstrom | 80 | +30 | 3500 | cold 1166 |
| sky dragon | 80 | +10 | 5000 | elec 1600/533/177 |
| great wyrm of many colours, of balance | 82 | +10 | 5200 | 1600/533/177 base; chaos 600/400 |
| Pazuzu, Lord of Air | 82 | +30, invisible | 5500 | BO_MANA, 576 melee |
| Cantoras, the Skeletal Lord | 84 | +30 | 7500 | BA_NETH (max 540) at 1-in-2 |
| The Witch-King of Angmar | 85 | +20 | 5000 | BRAIN_SMASH, BO_MANA, BA_NETH, TELE_AWAY, many summons, at 1-in-2 |
| Ancalagon the Black | 85 | +20 | 10000 | 1600/533/177 for acid/elec/fire/cold |
| greater Balrog | 85 | +20 | 4500 | fire 1500/500/166 |
| The Tarrasque | 86 | +20 | 8500 | fire/cold 1600/533/177, disen 500/333, breathes at 1-in-2 |
| Lungorthin | 88 | +20 | 7000 | fire 1600/533/177, plasma 150 |
| Carcharoth, Huan | 90 | +20 | 8000 | fire/cold 1600/533/177, nether 600/400 … |
| Vecna, the Emperor Lich | 92 | +20 | 6000 | BA_MANA (power 92 → max 560), BRAIN_SMASH, at 1-in-2 |
| Gothmog | 95 | +20 | 8000 | fire 1600/533/177 |

Two numbers dominate this phase. **533** is the largest resisted base-element breath (fire, cold,
acid or electricity at the 1600 cap ÷ 3): a character without about 600 hit points, or without a
temporary resist on top, can die to one breath of a great wyrm. **400** is the largest resisted
high-element breath (nether or chaos at the 600 cap × 6/9). An unresisted chaos or nether breath can be
600; an unresisted fire breath 1600.

### Go-deeper test (to DL80, then DL98)

- [ ] CL ≥ 45
- [ ] maximum hit points ≥ 600 (one resisted max fire breath is 533)
- [ ] speed +15 (DL80) / +20 (DL90+)
- [ ] rNether, rChaos, rDisen, rPois, base four, pBlind, pConf, FA, SI, Hold Life
- [ ] Healing ×10, `*Healing*` ×5+
- [ ] Teleport Other or Banishment for uniques you will not fight

### Buy / farm

- Farm `*Healing*` and Life potions (DL40+/DL60+ allocation) and Potions of Speed. The C borg
  deliberately waits on DL80-81 until it holds 10, and then 30, `*Healing*`/Life potions
  (`src/borg/borg-prepared.c:borg_prepared()`, "Scumming *Heal* potions").
- Farm experience on great wyrms only when doubly resisted to their breath.

### Common deaths and counters

| Death | Counter |
|---|---|
| a 1600-cap breath while only singly resisted at low hp | keep hit points above 600; use temporary resists before engaging dragons |
| mana balls (unresistable) from the Mouth of Sauron, Omarax, Vecna, black reavers | keep a hit-point buffer above their maximum; Teleport out |
| summoned crowds (Witch-King, Vecna, Nazgûl) | Banishment/Mass Banishment, or leave |
| being slowed at depth | Speed potions; free action stays essential |
| stat and experience drain | Hold Life, sustains; Life potions restore everything |

### Recall routine

Recall only when necessary; each trip to town is short and focused (Black Market, potions). From DL80
the "home stock" of endgame potions is the most important part of your character.

---

## 12.11 Phase 7: Endgame — Sauron (DL99) and Morgoth (DL100)

### 12.11.1 The rules of DL99 and DL100 (FACT, from source)

| Rule | Source |
|---|---|
| DL99 (Sauron) and DL100 (Morgoth) are quest levels while their quest is incomplete | `lib/gamedata/quest.txt`; `src/player-quest.c:is_quest()` |
| Quest levels always use the "classic" cave profile at full size | `src/generate.c` (`profile = find_cave_profile("classic")`, `if (dun->quest) size_percent = 100`) |
| The questor is placed when the level is generated (unless that unique is already alive elsewhere) | `src/generate.c`, "Ensure quest monsters" |
| No down staircases are generated on a quest level | `src/gen-cave.c` / `src/gen-util.c:alloc_stairs()` (quest levels get only up stairs); `docs/command.rst`: quest levels "have none until the quest monster is killed" |
| Killing the questor creates a magical down staircase at its grid | `src/player-quest.c:build_quest_stairs()` ("A magical staircase appears...") |
| Teleport Level cannot take you down from a quest level, and a trapdoor cannot exist there | `src/effect-handler-general.c:effect_handler_TELEPORT_LEVEL()` (`if (is_quest(...)) down = false`); `src/trap.c` skips trap doors on quest levels |
| Deep Descent and stairs stop at the first quest level | `src/player-util.c:dungeon_get_next_level()` |
| Word of Recall can take you to DL99 (your max depth) but not past it | the recall depth is your max depth, which cannot exceed 99 before Sauron dies |
| Morgoth's death completes the last quest: you are the winner | `quest_check()` sets `total_winner` when no incomplete quests remain |
| Morgoth drops every artifact flagged as a quest artifact (Grond and the Massive Iron Crown of Morgoth) | `src/mon-make.c` ("Morgoth currently drops all artifacts with the QUEST_ART flag"); `lib/gamedata/artifact.txt`, "Quest artifacts" |
| Questors (and other `FORCE_SLEEP` monsters) start "nice": they cannot cast spells until you have taken your first energy-using action after they appear | `RF_FORCE_SLEEP` → `MFLAG_NICE` (`src/mon-make.c`); `monster_can_cast()` refuses while nice; the flag is cleared for all monsters in `src/game-world.c:process_player_cleanup()` |
| With `birth_force_descend`, you cannot recall away from a quest level | `effect_handler_RECALL()` |

So: once on DL99 you can always leave upward (stairs up, Teleport Level up, Word of Recall), but the only
way down is through Sauron. After Sauron dies, DL100 behaves the same way with Morgoth. The official
manual adds: "Try to avoid wandering around on level 100 unless you are ready for him, since he has a
habit of coming at you across the dungeon" ([dungeon.rst](user-manual/dungeon.rst), "Winning The Game").
Morgoth has `hearing:100` and `sleepiness:0`.

### 12.11.2 Sauron, the Sorcerer (FACT)

`monster.txt`: depth 99, speed +20 (130), 8000 hit points, AC 192, `spell-freq:2` (50% per turn,
**100% at his preferred range**), `spell-power:130`, `REGENERATE`, `EVIL`, `UNIQUE`, `QUESTOR`.
Immunities: electricity (own flag) plus cold, fire, poison from the `ainu` base; `NO_CONF`, `NO_SLEEP`,
`NO_HOLD`, `NO_FEAR`. He arrives with servants if they are alive:
`friends:100:1d1:Khamûl, the Black Easterling:servant` and
`friends:100:1d1:The Witch-King of Angmar:servant`.

Melee: 2× HIT:DISENCHANT 10d12, 2× HIT:DRAIN_CHARGES 8d12 (max 432 before AC).

Spells, with maximum damage at power 130 (formulas in `lib/gamedata/monster_spell.txt`):

| Spell | Formula | Max | Resisted max | Saving throw? |
|---|---|---|---|---|
| BA_DARK | 4×130 + 10d10 | 620 | 413 (rDark) | no |
| BA_NETH | 4×130 + 10d10 | 620 | 413 (rNether) | no |
| BA_FIRE | 10 + 1d(130×7/2) | 465 | 155 / 51 | no |
| BO_MANA | 50 + 1d(130×5/2) | 375 | **no resist** | no |
| BO_PLAS | 140 + 8d7 | 196 | no resist | no |
| BO_ICE | 130 + 6d6 | 166 | 55 (cold resist applies to ice) | no |
| WOUND | 86d5 | 430 | — | **yes** (whole spell negated on a save) |
| BRAIN_SMASH | 12d15 + slow, confuse, paralyse, blind | 180 | — | yes |
| TELE_LEVEL | teleport level (up only on DL99) | — | resisted by nexus resistance (`spell_check_for_fail_rune`) | yes |
| TPORT | Sauron teleports himself | | | |
| SHAPECHANGE | into Wolf-, Serpent- or Vampire-Sauron | | | |
| S_HI_DEMON, S_HI_DRAGON, S_HI_UNDEAD, S_UNIQUE, S_WRAITH | summons | | | |

Shapes (they keep Sauron's hit points):

| Shape | Speed | Notable | Max breath at 8000 hp |
|---|---|---|---|
| Wolf-Sauron | **+30** | CLAW:PARALYZE, CLAW:CONFUSE, POISON bites (256 melee); BR_FIRE, BR_NETH; immune to nether, fire, cold, elec, poison | fire 1600/533/177, nether 600/400 |
| Serpent-Sauron | +20 | BRAIN_SMASH, FORGET; BR_POIS, BR_CHAO, BR_ACID | acid 1600/533/177, chaos 600/400, poison 800/266/88 |
| Vampire-Sauron | +20 | EXP_80 bite, TERRIFY; STORM, BA_NETH, BE_NETH, BA_POIS, BA_ACID, BO_PLAS, BO_ICE at 1-in-2 | — |

Breath uses the shape's *current* hit points, so a Sauron at full health in wolf form breathes fire at
the 1600 cap. Damage him first in his base form, where his attacks are spells with fixed formulas.

**Sauron readiness checklist (ADVICE, with the numbers above).**

- [ ] maximum hit points ≥ 700 (BA_NETH/BA_DARK 620 unresisted; 413 resisted; plus a bolt in the same
      player turn if he is faster than you)
- [ ] speed ≥ +20 permanent, +30 with a Potion of Speed (`TIMED_INC:FAST`, `20+2d10` turns; the
      haste gives +10). At +20 you trade turns 1:1; Wolf-Sauron at +30 still outpaces you
- [ ] rFire, rCold, rAcid, rElec, rPois, rNether, rDark, rChaos, rNexus (stops his teleport level),
      FA (wolf form paralyses), pConf, pBlind, Hold Life (vampire form), SI
- [ ] a high saving throw (negates WOUND and BRAIN_SMASH entirely)
- [ ] `*Healing*`/Healing potions in quantity (15+), Speed potions (5+)
- [ ] a way to deal with Khamûl and the Witch-King if they are alive: kill them first in earlier
      trips (Khamûl DL84, Witch-King DL85), or carry Teleport Other
- [ ] Banishment / Mass Banishment for summoned high demons and dragons

**Sauron fight plan (ADVICE).**

1. Kill his servants beforehand if possible; they are uniques at DL84-85 that can be met earlier.
2. Arrive on DL99 by stairs or recall, buffed (speed potion, heroism/berserk, temporary resists).
   He cannot cast until you have taken one action (`MFLAG_NICE`), so your first action on the level is
   safe from his spells (not from melee if he is adjacent).
3. Fight adjacent or at any distance other than his preferred range: his cast chance doubles at
   `best_range`, which for a frequent caster is three grids beyond his minimum range
   (`get_move_find_range()`, `monster_can_cast()`). In melee range his cast chance is 50%.
4. Heal when hit points fall below ~650 (one unresisted 620 ball plus margin), not lower.
5. When he shapechanges into Wolf-Sauron (+30), expect breath at full current hit points; if he is
   still above 4800 hp, fire breath is at the 1600 cap (533 resisted): heal above 600 before each of
   his turns, or Phase away to range ≥ 7 where a breath falls to half.
6. Use summon control: Banishment of the summoned symbol, or move on after killing Sauron.
7. When he dies a magical staircase appears at his grid. Do not take it until you are ready for
   Morgoth: you can recall to town first (the quest is complete; your max depth stays 99 until you
   step on DL100).

### 12.11.3 Morgoth, Lord of Darkness (FACT)

`monster.txt`: depth 100, speed +30 (140), 20000 hit points, AC 180, `spell-freq:3` (33% per turn,
66% at preferred range), spell power 100 (his depth), `REGENERATE`, `SMASH_WALL`, `MOVE_BODY`,
`EVIL`, `SPIRIT`, `SMART`, `UNIQUE`, `QUESTOR`, `FORCE_DEPTH`, `FORCE_SLEEP`.

Immunities: **acid, cold, electricity, fire, poison**; `NO_CONF`, `NO_SLEEP`, `NO_HOLD`, `NO_FEAR`. He
is *not* listed as resisting nether, light, dark, mana, plasma, meteors, holy power or disenchantment,
and he is `EVIL` (slay evil works).

Melee (per Morgoth turn):

| Blow | Damage | Effect |
|---|---|---|
| HIT:SHATTER | 20d10 (max 200) | reduced by AC; if damage after AC > 23, an earthquake of radius damage/12 centred on Morgoth; > 100 can knock you back (`src/mon-blows.c:melee_effect_handler_SHATTER()`) |
| HIT:SHATTER | 20d10 (max 200) | as above |
| HIT:LOSE_ALL | 10d12 (max 120) | **not** reduced by AC; drains STR, DEX, CON, INT, WIS unless sustained (`melee_effect_handler_LOSE_ALL()`) |
| TOUCH:DRAIN_CHARGES | — | drains a wand/staff/rod charge and heals him |

At AC 200 the SHATTER blows drop to 100 each; worst-case melee is about 320 per Morgoth turn and he gets
1.27 turns per player turn at +20 (more at lower speed).

Spells at power 100:

| Spell | Formula | Max | Resisted |
|---|---|---|---|
| BA_MANA | 5×100 + 10d10 | **600** | **unresistable** (mana has no resist) |
| BA_NETH | 4×100 + 10d10 | 500 | 333 with rNether |
| BO_MANA | 50 + 1d250 | 300 | unresistable |
| BRAIN_SMASH | 12d15 + slow/confuse/paralyse/blind | 180 | saving throw negates |
| S_HI_DEMON, S_HI_DRAGON, S_HI_UNDEAD, S_MONSTERS, S_UNIQUE | summons | | |

Regeneration: `REGENERATE` doubles the base 1% per regeneration tick, so Morgoth recovers 2% = 400 hit
points every 100 game turns (`src/mon-move.c:regen_monster()`, called when `turn % 100 == 0`). At +20
speed you get about 30 player turns per 100 game turns, so his regeneration is about 13 hp per player
turn: irrelevant compared with endgame damage output, but long breaks to heal do give some back.

A Glyph of Warding stops him only (100−1)/550 ≈ 18% of the time per attempt
(`mon-play:break-glyph:550`; `src/mon-move.c`, `randint1(glyph_hardness) < level`).

**Morgoth readiness checklist (ADVICE, from the numbers above).**

- [ ] maximum hit points ≥ 800 (two consecutive unresistable 600 mana balls is the worst case; with
      +20 speed he gets two actions in a row often enough that 600 is not a safe floor)
- [ ] speed +20 permanent and Speed potions (10+) to reach +30 parity
- [ ] all sustains (LOSE_ALL), Free Action, pConf, pBlind (BRAIN_SMASH side effects if you fail the
      save), rNether, See Invisible
- [ ] 15-30 `*Healing*` / Life potions (*Healing* heals 1200 and cures; Life heals 5000, restores
      experience and all stats: `object.txt`)
- [ ] Restore Mana in quantity for casters (the C borg asks for 15)
- [ ] summon control (Banishment, Mass Banishment, or Teleport Other on summoned uniques)
- [ ] a damage plan that does not rely on acid/cold/elec/fire/poison (he is immune): melee with a
      slay-evil weapon, nether/mana/holy/light attacks, or wands of drain life / annihilation
      (check [Chapter 7](07-consumables-and-devices.md) for what each does to an `EVIL` non-living
      target)

**Morgoth fight plan (ADVICE).**

1. Prepare in town with a full stock; recall to DL99, take the magical staircase (or stairs on a new
   DL99 if Sauron is dead) to DL100.
2. Buff before he arrives: Speed potion, heroism/berserk, protection from evil if a priest/paladin.
3. Fight in a large open area or a long corridor; earthquakes from SHATTER rearrange the walls around
   him, so corridor tactics degrade.
4. Heal at ≤ 700 hit points; do not wait for a lower threshold. A `*Healing*` (1200) is the normal heal;
   Life when stats are drained.
5. Summons: Banish the summoned type, or kill quickly what blocks you. If too many uniques appear,
   Teleport Level up (DL100 → DL99 is allowed) and come back.
6. After the kill: pick up Grond and the Crown (the Crown is `STICKY`: once worn it cannot be taken off,
   see `artifact.txt`), then retire with `Q` when ready ([dungeon.rst](user-manual/dungeon.rst)).

### 12.11.4 Endgame preparation summary (ADVICE)

| Item | Sauron | Morgoth | Why (FACT) |
|---|---|---|---|
| Max HP | 700+ | 800+ | 620 ball (Sauron); 600 unresistable mana ball (Morgoth) |
| Permanent speed | +20 | +20 | Sauron +20, Morgoth +30 |
| Temporary speed | +10 (potions) | +10 (potions) | parity with Morgoth needs +30 |
| `*Healing*`/Life | 10-15 | 15-30 | each heals 1200/5000 |
| Healing | 5-10 | 5-10 | 300+35% of missing hp |
| Resists | base 4, pois, nether, dark, chaos, nexus | nether (others matter for summons) | his immunities don't matter to you; his summons breathe |
| Status | FA, pConf, pBlind, SI, Hold Life | FA, pConf, pBlind, SI, sustains | paralyse/confuse from shapes and BRAIN_SMASH; LOSE_ALL |
| Saving throw | high | high | negates WOUND, BRAIN_SMASH, TELE_LEVEL |

---

## 12.12 The recall and home routine

### Facts

- Word of Recall activates after `15 + randint0(20)` world ticks (15-34; one tick per 10 game turns,
  i.e. one normal-speed player turn) (`effect_handler_RECALL()`; decremented in
  `src/game-world.c:process_world()`).
- Reading it below your max depth asks "Set recall depth to current depth?" (only without persistent
  levels); yes lowers your max depth to the current level (`player->recall_depth = player->max_depth =
  player->depth`). This is how you deliberately *reset* a too-deep max depth.
- Reading it again while active asks to cancel it.
- Deep Descent fires after 4-7 ticks and targets max depth + 5 (Section 12.3).
- Stores turn over stock with game time (`store:turns:1000`, `store:shuffle:25` in `constants.txt`).

### A routine that works in every phase (ADVICE)

1. **Leave when** any of: healing potions below the phase's minimum; escapes below minimum; pack full
   of valuable items; a curse or drain that town can fix; a level with nothing more to gain and no
   down stairs found.
2. **Read recall early**, in a safe spot, then keep exploring or resting until it fires. Never rely on
   recall as an emergency escape: 15-34 ticks is a long time.
3. **In town**, in this order: Alchemist (healing, Phase, Word of Recall ×2), General store (food,
   oil, light), Magic shop (staff of teleport recharge or replacement, wands), Black Market (scan for
   speed, Free Action, See Invisible, stat potions, Healing), Home (drop/swap), armour and weapon
   stores last.
4. **Home** holds: a spare Word of Recall, swap items (resist-X jewellery, a second launcher), the
   endgame potion stock from DL60 on, and spellbook spares.
5. **Return** with recall from town; you arrive on a random grid (not a staircase) at your recall depth.
   Detect immediately.

The official guide suggests an alternative for the DL20s: recall up, then take stairs down from DL1
"to the level you left" so that you always arrive on stairs. That costs time but guarantees an escape on
arrival; with connected stairs every level arrived at by stairs starts on a staircase.

---

## 12.13 Summary tables: kit by depth

### Defensive kit (ADVICE; the "first threat" column is FACT from `monster.txt`)

| Property | Get it by | First serious threat that needs it |
|---|---|---|
| Free Action | DL20-25 | floating eye (DL1, melee only), druid/illusionist HOLD (DL13), carrion crawler (DL25), ghoul (DL26), basilisk (DL28) |
| See Invisible / ESP | DL25-30 | moaning spirit (DL12), banshee (DL24), ghost (DL31), will o' the wisp (DL38), dread (DL44) |
| rFire, rCold | DL25-30 | young red/white dragons (DL29-31), fire giant (DL30), hellhounds (DL42) |
| rAcid, rElec | DL30-35 | young black/blue dragons (DL29-31), young MHD (DL32) |
| rPois | DL40-45 | basilisk (DL28), 6/7-headed hydras (DL39-40), AMHD (DL53, 800 cap), drolem (DL54) |
| pConf | DL35-45 | umber hulk gaze (DL16), Sangahyando (DL24), many casters |
| pBlind | DL40-50 | dark breath (ethereal drake DL33, greater basilisk DL45), BLIND casters |
| Hold Life | DL45-55 | vampire bat (DL24), wights (DL24-37), nether wraith (DL54), Nazgûl (DL77-84) |
| rNether | DL55-65 | dracolich (DL55), death drake (DL50), werewolf of Sauron (DL63), Sauron (DL99) |
| rChaos | DL60-75 | Kavlax (DL41), chaos hound (DL64), great wyrm of chaos (DL75), pit fiend (DL77) |
| rDisen | DL65-80 | great wyrm of chaos (DL75), Tarrasque (DL86) |
| rSound/rShards | DL60-80 | vibration hound (DL27, stun), great crystal drake (DL50), great wyrm of law (DL75) |
| rNexus | DL60-99 | nexus hounds (DL27), dracolisk (DL55); stops Sauron's TELE_LEVEL |
| Speed +10 | DL50-60 | Balrog of Moria (+20, DL50), time hounds (+20, DL51) |
| Speed +20 | DL80-90 | Ancalagon, Tarrasque, greater Balrogs (+20, DL85+); Sauron (+20) |
| Speed +30 (with potion) | DL100 | Morgoth (+30) |

### Consumables carried (ADVICE)

| Phase | Heal | Escape | Other |
|---|---|---|---|
| DL1-5 | CLW ×5 | Phase ×5 | oil ×15, WoR ×1 |
| DL5-15 | CLW/CSW ×5-10 | Phase ×10 | WoR ×1, oil, Magic Mapping |
| DL15-30 | CCW ×5-10 | Phase ×10, Teleport ×5 | WoR, resist potions |
| DL30-40 | CCW ×10, Healing ×2 | Teleport ×10, Teleport Level ×2 | WoR, Speed ×2 |
| DL40-60 | CCW ×10, Healing ×5 | Teleport ×10, Teleport Level ×3 | Speed ×5, Restore Mana |
| DL60-98 | Healing ×10, `*Healing*` ×5 | as above + Teleport Other | Speed ×10, Banishment |
| Endgame | `*Healing*`/Life ×15-30 | Teleport Level | Speed ×10-20 |

Healing amounts (FACT, `object.txt`): CLW 20, CSW 40, CCW 60, Healing 300 or 35% of missing hp
(whichever is more), `*Healing*` 1200, Life 5000 (plus restores).

---

## 12.14 The C borg's readiness rules, and where the data disagrees

The C borg asks "may I be on this depth?" through `borg_prepared(depth)` in
`src/borg/borg-prepared.c` (see [BORG_SPEC §5.5](../c-borg/BORG_SPEC.md)). It is one source of
ideas, transcribed here and then critiqued. The borg is not an authority on good play: its death rates
range from 19% (Paladin) to 70% (Druid) on the tracker's harness
([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md)).

### What the borg requires (transcribed, default mode, not "plays risky")

Global rule in `borg_prepared()`: **character level ≥ depth until CL50** ("Clevel < depth"), plus
`borg_restock()` (carry enough to *stay*), plus `borg_prepared_aux()`:

| Depth | Requirement (cumulative) |
|---|---|
| 1 | light radius ≥ 1, food ≥ 5 |
| 2 | fuel ≥ 5 if no permanent light; 30 hp |
| 3-4 | class gates: warrior/blackguard 50 hp & CL4; rogue 50 hp & CL8; priest/druid 40 hp & CL9; paladin, ranger 50 hp & CL4; mage/necromancer 60 hp & CL11. Two cure potions (CLW+CSW+CCW) if CL<30 |
| 5-9 | (only while in the dungeon) warrior 60 hp & CL6; rogue CL10; priest/druid CL15; paladin/ranger CL6; mage/necro 80 hp & CL15. Word of Recall ×1 |
| 10-19 | light radius ≥ 2 (not necromancers); Teleport+teleport-staff charges ≥ 2; class: warrior CL ≥ depth−4, rogue/priest/druid/paladin/ranger CL ≥ depth, mage/necro CL ≥ depth+5 (until CL28); CCW ≥ 3 if CL<30; See Invisible or detect invisible or ESP |
| 20 | Free Action |
| 21-25 | resist fire and two of acid/cold/elec; STR, DEX, CON and spell stat ≥ 7; class: warrior CL ≥ depth+5 (until CL38), rogue +10 (until 43), priest/druid +13 (until 46), paladin +7 (until 40), ranger +8 (between CL29 and 41), mage/necro +8 (until 38) |
| 26-33 | all four base resists; teleports ≥ 6; CCW+CSW ≥ 10 if CL<30 |
| 34-39 | CL40 (unless playing risky) |
| 40-45 | rPois, pConf; STR, DEX, CON, spell stat ≥ 16 |
| 46-55 | speed +5; one Healing or `*Healing*`; 500 hp; STR ≥ 18/40, DEX and CON ≥ 18/60, spell stat 18/100; Hold Life (unless CL50) |
| 56-59 | two Healing or one `*Healing*`; pBlind; telepathy |
| 61-80 | speed +10; rChaos; rDisen |
| 81-85 | speed +20 |
| 86-99 | nothing new |
| 100 | (unless winner) 15 Restore Mana if max SP > 100; 5 Healing; 15 `*Healing*`+Life; 10 Speed potions |

Extra gates in `borg_prepared()`: from DL82, at least 10 `*Healing*`/Life (counting the home) or it
"scums" DL80-81; once max depth ≥ 97, at least 30 before going below DL81. With `borg_kills_uniques`,
it will not go below the depth of a living unique while three or more uniques are alive. At max depth
≥ 99 it refuses DL98+ unless **every unique except Morgoth is dead** (with one exception: at max depth
exactly 99 it may visit DL98 if it can Teleport Level).

`borg_restock()` (carry-to-stay) asks for, cumulatively: light; fuel; food ≥ 3 (DL2+); Phase ≥ 1 and a
cure potion (DL6+); light radius 2, two cures, two teleports (DL10+); four CSW/CCW and four teleports
(DL20+); Teleport+Teleport Level scrolls ≥ 2 (DL36+); a Healing (DL65+); fifteen `*Healing*` on
arrival at DL100.

### Where the borg and the data agree

- **Free Action at DL20.** The data agree: druid and illusionist (DL13) cast HOLD; carrion crawlers
  (DL25) and ghouls (DL26) paralyse in melee. DL20 is a good gate, perhaps slightly late for casters
  who meet illusionists at DL13.
- **See invisible by the DL10-19 band.** Moaning spirits (DL12) and banshees (DL24) justify it,
  although an invisible monster in this band is rarely lethal; the gate is conservative but harmless.
- **rPois at DL40.** The hydras (DL39-40) and AMHD/drolem (DL53-54, 733-800 unresisted) justify this
  gate exactly.
- **+10 speed by DL61 and +20 by DL81.** The deep uniques are +20 (Section 12.10); the gates match.
- **The *Healing* stock and 10 Speed potions for Morgoth** match the arithmetic of 12.11.3.

### Where the data suggests otherwise

| Borg rule | Critique from the data | Suggested change (untested) |
|---|---|---|
| **CL ≥ depth everywhere until CL50** | The single biggest brake on the borg ([TUNABLE-BORG-PLAN](../c-borg/TUNABLE-BORG-PLAN.md) calls depth preparation "probably the biggest single lever"). Experience per kill grows with monster level and falls with your level, so a character kept shallow by this rule levels slowly. Most DL1-15 threats are beaten by kit (escapes, ranged attacks, FA), not by CL. | Gate on kit and hit points first, with CL ≥ depth/2 early (DL ≤ 20) and CL ≥ depth − 10 to DL40, then CL ≥ 40 deeper |
| Mage/necromancer CL ≥ depth + 5 (DL10-19) and + 8 (DL21+) | Makes casters the slowest divers, yet casters have the best escapes (*Phase Door* CL2, *Teleport Self* CL7, *Teleport Level* CL28 for mages) and the best detection | Gate casters on spell availability (Teleport Self at fail ≤ 10%, Detect Monsters) and hit points instead of CL+5 |
| Light radius 2 at DL10 | The borg's "light radius" is the intensity value (`cur_light`): a Wooden Torch (`LIGHT_2`) already meets it, though it lights only adjacent grids; a Lantern (`LIGHT_3`) lights radius 2 | Require intensity 3 (a Lantern) for non-necromancers by DL10 |
| Stat minimums of 7 at DL21 and 16 at DL40 | Low stats are a symptom, not a danger; a STR 15 caster with all resists is safer at DL40 than a STR 18 warrior without rPois | Replace with derived quantities: blows, hit points, spell failure rates |
| 18/40 STR, 18/60 DEX/CON and max spell stat for DL46 | Strong demand that delays the borg in the DL40s, a phase where damage taken is dominated by breath (which stats do not affect except via hit points) | Use max hit points (already gated at 500) and spell failure |
| Teleport counts mix scrolls and staff charges ("tele + teleport staffs") | Correct in spirit; but no requirement for *Teleport Level*, the best escape from a level with an unbeatable unique, until DL36 (restock) | Require Teleport Level ×1 from DL25 (the scroll is allocated from DL20) |
| No rNether until never (commented out) | Nether is the dominant high element deep: dracolich (DL55), werewolf of Sauron (DL63), Sauron (BA_NETH 620), Morgoth (BA_NETH 500) | Require rNether or Hold Life by DL60, rNether by DL80 |
| No rNexus, rSound, rShards requirement | Sound stuns, shards cut; nexus scrambles stats and teleports; Sauron's TELE_LEVEL is resisted by nexus resistance | Add rNexus as a DL99 desired item; rSound/rShards desirable by DL70 |
| "All uniques except Morgoth dead" before DL98+ | Not a game rule: only Sauron must die. It forces the borg to hunt every deep unique (Ancalagon, Tarrasque, Gothmog...), each a large risk. Some, like Khamûl and the Witch-King, are worth killing because they escort Sauron | Require: Sauron's two servants dead (or Teleport Other ready) plus the Morgoth kit |
| `borg_kills_uniques` gate "three living uniques above you" | Encourages fighting uniques that give nothing toward the goal | Keep optional; prefer the risk estimate of [Chapter 15](15-notes-for-the-borg.md) |
| Max hit points not required between DL55 and DL100 | A resisted 1600-cap breath is 533; an unresistable mana ball is up to 600 at DL100 and 560 from Vecna (DL92) | Require HP ≥ 600 from DL75, ≥ 800 for DL100 |
| No check of saving throw | WOUND and BRAIN_SMASH are negated entirely by a save; saving throw matters against liches, mind flayers, Sauron | Track `skill_sav`; prefer gear that raises it |
| `borg_heal()` uses random thresholds and hoards `*Healing*`/Life for Morgoth | A potion unused at death is wasted; random thresholds add noise | Heal by expected incoming damage, not random draws ([Chapter 15](15-notes-for-the-borg.md), HEAL-*) |

The pattern is that the borg's gates are strongest where kit matters least (CL and stats early) and
weakest where kit matters most (high resists and hit-point buffers deep). A rebuilt automatic player
should gate depth on **the maximum damage the level's likely monsters can do in one player turn**
against **current resists and hit points**, which is computable from the monster data and the
character sheet.

---

## Things to try (untested)

None of these has been measured. Each needs a controlled trial ([Chapter 16](16-things-to-try.md)
describes the method).

1. **Kit-gated diving instead of CL-gated diving.** Replace "CL ≥ depth" with "max hit points ≥
   2 × the largest single max-damage attack of monsters native to depth+5, given current resists".
   *Why:* experience per kill scales with monster level. *Test:* borg runs on fixed seeds, both rules,
   metric: alive at turn T having reached DL D, deaths by cause.
2. **Early Deep Descent as a "level skip".** Read Deep Descent only when current depth equals max
   depth (so it is exactly +5) and the go-deeper test of the target band passes. *Why:* skips five
   levels of risk-carrying moves. *Risk:* you arrive off-stairs, 5 levels deeper. *Test:* compare
   turns-to-DL30 and death rate with and without.
3. **"Stairs-only" return after recall in the DL15-30 band** (the official guide's suggestion).
   *Why:* arrival on a staircase gives a free escape. *Test:* count deaths within 50 turns of arrival,
   recall-arrival vs stair-arrival.
4. **Resistance swap kit at home.** Keep one item per base element and swap them in on detection of
   a dragon of that colour before engaging. *Test:* breath damage taken per 1000 turns at DL30-50.
5. **Fight Sauron adjacent.** His spell chance is 50% adjacent and 100% at preferred range. *Test:*
   in debug mode, repeated Sauron fights at distance 1 versus distance 4; count spells per 100 of his
   turns.
6. **Kill Khamûl and the Witch-King early** (DL84/85) before attempting Sauron, versus carrying
   Teleport Other. *Test:* Sauron fight outcomes with and without servants present.
7. **Long-range dragon fights.** Non-powerful breath falls to half at distance 7 and a third at 11.
   *Why:* `project()` falloff with source diameter 4. *Test:* average breath damage taken versus
   distance in controlled encounters with mature dragons.
8. **Single Combat (Paladin CL40) against a questor.** It isolates you with one monster; Morgoth
   resists with probability about 49% at CL50 (`randint0(spell_power) > player level`), Sauron about
   61%. *Risk:* there is no escape from an arena (no recall, no level change). *Test:* debug-mode trials
   for whether summons work inside and whether the quest completes.
9. **A deliberate "shoulder" at DL30-40.** Stop diving until rPois and +5 speed are found, versus
   diving to DL50 with temporary resist potions. *Test:* death rates on the DL40-55 band.
10. **Early Free Action purchase.** Spend the first big money on Black Market Free Action, even at the
    cost of other gear. *Test:* paralysis-caused deaths before DL30.

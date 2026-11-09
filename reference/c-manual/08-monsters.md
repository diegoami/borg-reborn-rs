# 8. Monsters

What the game tells you about monsters, how they think (movement, groups, fear, spells,
summons, breeding), how to see the ones you cannot see, a tour of every monster family by
glyph, the monsters that most often end characters at each depth, and a tactical guide to
the uniques, finishing with Sauron and Morgoth.

**Contents**

- [8.1 Monster recall: what you know and how you learn it](#81-monster-recall-what-you-know-and-how-you-learn-it)
- [8.2 Basic monster numbers](#82-basic-monster-numbers)
- [8.3 How a monster takes its turn](#83-how-a-monster-takes-its-turn)
- [8.4 Movement, tracking and preferred range](#84-movement-tracking-and-preferred-range)
- [8.5 Group AI](#85-group-ai)
- [8.6 Fear and morale](#86-fear-and-morale)
- [8.7 Spellcasting](#87-spellcasting)
- [8.8 Summoning](#88-summoning)
- [8.9 Regeneration, doors, walls, items, webs, wards](#89-regeneration-doors-walls-items-webs-wards)
- [8.10 Breeders](#810-breeders)
- [8.11 Seeing monsters: invisibility, ESP, infravision, detection, light](#811-seeing-monsters-invisibility-esp-infravision-detection-light)
- [8.12 Monster spells that matter and their counters](#812-monster-spells-that-matter-and-their-counters)
- [8.13 Monster families by glyph](#813-monster-families-by-glyph)
- [8.14 Killers by depth](#814-killers-by-depth)
- [8.15 Uniques: a tactical guide](#815-uniques-a-tactical-guide)
- [8.16 The final quests: Sauron and Morgoth](#816-the-final-quests-sauron-and-morgoth)
- [Things to try (untested)](#things-to-try-untested)

Full data tables are generated in the appendices: every monster by depth in
[A1](appendices/A1-monsters-by-depth.md), every unique in [A2](appendices/A2-uniques.md),
every monster spell with its damage formula in [A3](appendices/A3-monster-spells.md).
This chapter interprets them. Numbers here are from `lib/gamedata/monster.txt`,
`monster_base.txt`, `monster_spell.txt` and the code cited.

Notation in this chapter: **L** = native depth (`depth:`), speed as +N relative to
normal, **HP** = average HP (uniques have exactly this), "melee ≈ N" = sum of the average
dice of all blows per monster turn (before AC and hit chance), breath values are at full
HP. Spell damage is the average at the monster's spell power (= its level unless
`spell-power` is given).

---

## 8.1 Monster recall: what you know and how you learn it

### What the official manual says

The [Attacking](https://angband.readthedocs.io/en/latest/attack.html) page
([local](user-manual/attack.rst), "Monster Memories") explains that the game remembers what
you have observed about each monster race, that killing enough of a race or suffering
enough of its attacks reveals armour, hit dice, spells and frequencies, that attacks are
colour-coded by whether you resist them, and that the memory is shared by all your
characters (`lore.txt` in your user directory). Look at a monster with `l` (or `x` in the
roguelike keys) and press `r` for recall; the full list is in the knowledge menu (`~`).

### What you learn, and when

FACT (`src/mon-lore.c:lore_update()` and its callers):

| Knowledge | How you get it |
|---|---|
| "Obvious" flags (glyph, colour, unique, gender etc.) | Always |
| A specific melee blow (method, effect, dice) | The first time that blow is used on you (`times_seen`) |
| Hit chance of each known blow against your current AC, and "averaging N damage on each of its turns" | As soon as the blow is known (`lore_append_attack()`) |
| HP ("life rating"), armour, **your % chance to hit it in melee** | After you have **killed one** (`tkills > 0`) |
| What it drops, race adjectives, generation flags | After killing one |
| Flags such as `INVISIBLE`, `COLD_BLOOD`, `PASS_WALL`, `KILL_WALL`, `OPEN_DOOR`, `BASH_DOOR`, `MULTIPLY`, `RAND_25/50`, `TAKE_ITEM`, `NEVER_MOVE` | When you see it do (or fail to do) the thing in question |
| `EMPTY_MIND`, `WEIRD_MIND`, `SMART`, `STUPID` | When you sense it with telepathy |
| A spell | The first time you see it cast |
| Spell damage figures | Once its HP is known (after one kill) |
| Spell frequency ("1 time in N") | After seeing it cast more than 50 times (innate and non-innate counted separately) |
| How deeply it sleeps | After enough observations of it waking or ignoring you |
| Resistances | When you hit it with that element |
| **Everything** | **Probing** (Rod of Probing, native DL30+; the `PROBING` activation; `lore_do_probe()`) |

There is **no birth option to know all monster lore** in this version (`list-options.h`
has `birth_know_runes` and `birth_know_flavors`, not monsters). Lore accumulates across
characters in `lore.txt`, so later characters know more.

### Reading recall (ADVICE)

- The two most useful lines for a fight decision are "You have an N% chance to hit such
  a creature" and "averaging N damage on each of its turns"; both are recomputed from your
  current state. They feed directly into the damage race in
  [Chapter 4](04-combat.md#415-is-this-fight-winnable-a-damage-race-procedure).
- Blows you have not seen are not counted in the average ("at least N").
- Colour: red/orange attacks are unresisted, yellow partially, green resisted.
- **Borg note:** a borg should not rely on lore it has not earned. Use the data files
  (as this manual does) for planning, and the recall for live numbers.

---

## 8.2 Basic monster numbers

FACT (`src/mon-make.c:place_new_monster_one()`, `mon_hp()`):

- **HP:** uniques always have exactly their listed HP. Others roll a normal distribution
  around the average with standard deviation about 1/8 of it (minimum 1).
- **Speed:** uniques have exactly their speed. Others get a small random variation of
  up to ±(energy per turn/10) speed points: about ±1 at normal speed, ±2 at +10, ±3 at
  +20.
- **Sleep:** a monster with sleepiness `S` starts with a sleep counter of
  `2S + 1d(10S)`. Uniques and many deep monsters have `S = 0` (always awake). How your
  stealth wakes them is in [Chapter 1](01-core-mechanics.md).
- **Energy:** speed uses the same table as yours ([Chapter 1](01-core-mechanics.md)):
  normal = 10 energy per game turn, +10 = 20, +20 = 30, +30 = 38, −10 = 5.
- **Spell frequency:** `spell-freq:N` and `innate-freq:N` in `monster.txt` mean a
  `100/N` percent chance per turn (`src/mon-init.c`): 1_IN_2 = 50%, 1_IN_5 = 20%.
- **Groups:** `friends:` lines make escorts and packs; roles are *servant*, *bodyguard*
  (stays near the leader, never flees) or ordinary members (`src/mon-group.c`). Group
  size is capped at 25 (`constants.txt` `group-max`).

---

## 8.3 How a monster takes its turn

FACT (`src/mon-move.c:process_monsters()`, `monster_check_active()`,
`process_monster_timed()`, `monster_turn()`):

1. Every game turn each monster gains energy by its speed; at 100 it acts.
2. Every 100 game turns, monsters with energy regenerate HP ([8.9](#89-regeneration-doors-walls-items-webs-wards)).
3. **Mimics** (unrevealed `?`, `!`, `=`, `$`, `~`) do nothing until disturbed.
4. A monster is **active** only if it is hurt, or in your line of sight, or can hear you,
   or can smell you, or is a wall-passer within its hearing range of you, or is standing
   in damaging terrain. Inactive monsters skip their turn entirely.
5. Timed effects tick: a **sleeping** monster only tries to wake (and acts on a later
   turn); fear decreases by `1d(level/10 + 1)`; hold (paralysis) and command make it
   skip the turn; a stunned monster skips 1 turn in 10 (`STUN_MISS_CHANCE`).
6. `monster_turn()`:
   1. Rouse sleeping group members it can see (1 in distance×20 each).
   2. Try to **breed** ([8.10](#810-breeders)).
   3. Try a **ranged attack** (spell, breath, arrow) ([8.7](#87-spellcasting)).
   4. Otherwise move: randomly if confused or `RAND_25/RAND_50` triggers, else by AI
      ([8.4](#84-movement-tracking-and-preferred-range)). Moving into you is a melee
      attack ([Chapter 4](04-combat.md#413-monster-melee-against-you)).
   5. An afraid monster that found no move becomes **held** ("paralyzed by fear").

Important consequences:

- **Ranged attacks come before movement**, so a fleeing monster still casts and breathes.
- A monster that cannot see, hear or smell you and is unhurt does nothing at all, so a
  well-hidden player is ignored.

---

## 8.4 Movement, tracking and preferred range

### How monsters find you

FACT (`src/mon-move.c:get_move_advance()`, `monster_can_hear()`, `monster_can_smell()`,
`src/game-world.c:update_scent()`):

1. **Wall-passers and wall-eaters** (`PASS_WALL`, `KILL_WALL`) go straight for you,
   unless a permanent wall is in the way (then they use the flow like everyone else;
   5% of the time they flow anyway).
2. **If it can see you** (you are in its line of view), it heads straight for you.
3. **Sound:** the game keeps a "noise" map of walking distance from you. A monster can
   hear you if `hearing − stealth/3 > noise at its grid`; it then steps to the
   neighbouring grid with less noise. Most monsters have hearing 20 (in grids); stealth
   30 subtracts 10.
4. **Scent:** grids you walked over keep a scent that ages by 1 each player turn
   (only 2 grids around you are scented, not through walls). A monster with `smell` can
   follow scent no older than its smell value. About three quarters of all races have no
   smell at all (`monster.txt`).
5. Otherwise follow a group member who is tracking, keep going to the last target, or
   wander.

The `COVERTRACKS` timed effect (the Ranger spell *Cover Tracks*, `class.txt`)
lays no scent and hides you from sight beyond a quarter of max sight.

ADVICE: after you teleport, monsters that were chasing you by sound/scent converge on
where you were, then lose you if your noise no longer reaches them. Good stealth plus
distance makes most monsters give up.

### Preferred range

FACT (`src/mon-move.c:get_move_find_range()`):

- Every monster has a minimum range (how close it wants to be) and a best range.
- **Afraid or `FRIGHTENED`** monsters: minimum range = 25 (flee out of sight, up to 5
  grids beyond max sight).
- **Morale:** a monster compares its "power" (`level + 25`, +8 for half of individuals,
  scaled by HP fraction) with yours (`CL`, scaled by HP fraction). A monster much weaker
  than you (by level) or at a big disadvantage keeps away. Monsters within 5 grids stop
  running anyway. Bodyguards never flee.
- `NEVER_MOVE` and `NEVER_BLOW` (pure caster) monsters add 3 to minimum range.
- **Archers** (arrow/shot/bolt spells with innate frequency above 25%) add 3 to their
  best range: they like to stand back.
- **Frequent breathers** (innate frequency over 24%, more than half HP) like point-blank
  range; **frequent casters** (spell frequency over 24%) add 3 to best range.
- A monster at exactly its best range has **double** spell/breath chance
  (`src/mon-attack.c:monster_can_cast()`).
- If you have the `TAUNT` effect (Blackguard *Relentless Taunting*), monsters ignore
  morale and come straight at you, and cast half as often.

### Terrain

Monsters avoid damaging terrain (lava etc.) unless they resist it
(`monster_hates_grid()`). If they are in it, they seek safety first.

---

## 8.5 Group AI

FACT (`src/mon-move.c:get_move()`, `get_move_find_hiding()`):

Monsters with `GROUP_AI` (all `Z` hounds via their base; wild dogs, wolves, white wolves,
wargs, blink dogs, hellhounds, cave/wood/Mirkwood/phase spiders, craban, several ants,
the Queen Ant) that cannot pass walls:

1. **Ambush:** if fewer than 5 of the 8 grids around you are open (you are in a corridor
   or doorway) **and you have more than half your HP**, each member looks for a hidden
   grid at least `3/4 × its distance + 2` from you, out of your sight, and goes there. They
   wait for you to come out.
2. **Surround:** in the open, members that can see you pick an empty grid adjacent to you
   and fill it, so the pack surrounds you.

Other group behaviour:

- Bodyguards stay next to their leader and attack you when they can
  (`get_move_bodyguard()`).
- Group members follow a member who is tracking you.
- Summoned monsters join the summoner's group.

ADVICE: you cannot pull a hound pack into a corridor while healthy; either kill them at
the room edge as they show themselves (good with balls and breath-like spells), dive past
them, or leave. A pack coming *into* your corridor is a sign that your HP is below half.

---

## 8.6 Fear and morale

FACT (`src/mon-util.c:monster_scared_by_damage()`, `src/mon-predicate.c:monster_can_be_scared()`,
`src/mon-move.c`):

- **Becoming afraid:** when hit for at least its remaining HP (80% chance), or with
  `1d10 ≥ HP%` (so only near 10% HP). Duration 1d10 + 20, or 1d10 + 5 × (11 − HP%) at
  very low HP.
- **Immune:** `NO_FEAR` monsters (all `E`, `g`, `j`, `m`, `v`, `,`, `Q`, `x`, `s`
  skeletons, many uniques including Sauron and Morgoth), bodyguards; servants resist 2 in
  3; each other group member makes it 5% less likely (1 in 20 per member).
- **While afraid:** runs out of sight, still casts with +20% spell failure, freezes if
  cornered.
- **Recovery:** fear drops by `1d(level/10 + 1)` each turn; damage can reduce or cure it.
- Scare effects you cast (and the AFRAID blow effect used by monsters on monsters) add to
  the same timer.

The player side: you cannot melee while afraid ([Chapter 4](04-combat.md#42-melee-the-sequence-of-one-attack-command)).

---

## 8.7 Spellcasting

### Will it cast this turn?

FACT (`src/mon-attack.c:monster_can_cast()`, `make_ranged_attack()`):

1. The non-innate spell chance (`freq_spell`) is tried first, then the innate one
   (breaths, arrows, boulders, spit, whip, shriek). So the innate chance is effectively
   `(1 − spell%) × innate%`.
2. Chance doubles at the monster's best range, halves if you are taunting.
3. You must be within 20 grids (`max-range`) and there must be a projectable path
   (`PROJECT_SHORT`) from the monster to you. **No line of fire, no spell.**
4. A monster cannot cast on the turn it is created (`MFLAG_NICE`) until you have had a
   move.

### Which spell?

FACT (`make_ranged_attack()`, `remove_bad_spells()`, `choose_attack_spell()`):

- Start with all its spells. **Smart** monsters below 10% HP, half the time, drop all
  damaging spells (they heal, blink, teleport or summon instead).
- Non-stupid monsters remove: HEAL at full HP, HEAL_KIN with no hurt kin near, HASTE
  while already hasted (>10 turns), TELE_TO when adjacent, WHIP beyond 2 grids and SPIT
  beyond 3; **bolts if a monster is in the way**; **summons if no empty floor within 2
  grids of the summoner**.
- With `birth_ai_learn` (on by default), monsters remember what they have seen you
  resist ([below](#monsters-learn-your-resistances)) and drop spells that would fail.
- Then **one spell is chosen uniformly at random** from what is left (innate or
  non-innate, whichever roll succeeded). There is no further weighting.

### Failure and saving throws

FACT:

- **Non-innate spells fail about 24% of the time for every non-stupid monster**
  ("tries to cast a spell, but fails"). The code is
  `power = MIN(spell_power, 1); failrate = 25 − (power + 3)/4` in
  `src/mon-attack.c:monster_spell_failrate()`, which gives 24% for any monster with
  spell power ≥ 1. This looks like it was meant to be `MAX` (failure falling with spell
  power) but as written it is a flat 24%. Afraid +20%, confused or disenchanted +50%.
  Stupid monsters never fail. **Innate attacks never fail.**
- Spells with a saving throw: MIND_BLAST, BRAIN_SMASH, WOUND, SCARE, BLIND, CONF, SLOW,
  HOLD, TELE_LEVEL, FORGET (`monster_spell.txt` `message-save`). The save is
  `1d100 < your saving-throw skill` (`src/mon-spell.c:do_mon_spell()`). A save of 100%
  (high-level Priests with high WIS, items) blocks them all, though the protection
  flags (FA, pConf, rBlind, pFear, rNexus for TELE_LEVEL) are checked separately and are
  more reliable.
- ARROW/SHOT/BOLT (hit 50) and BOULDER (hit 60) roll to hit against your AC like a melee
  blow with power = that number.

### Monsters learn your resistances

FACT (`src/mon-util.c:update_smart_learn()`, `src/mon-spell.c:unset_spells()`):

- When you resist something in a monster's presence, the monster may learn it (always
  if `SMART`, half the time otherwise; never if `STUPID`; 1% failure).
- Before casting, for each bolt/ball/breath of an element you resist, the spell is
  dropped with probability `resist_level × 50%` (smart) or `× 25%` (not smart); an
  immunity (level 3) is always dropped by smart monsters. Status spells you are
  protected from are dropped (always for smart monsters, 2 times in 3 otherwise).
- Each turn there is a 1 in 20 chance the monster forgets everything it learned.

ADVICE: resistances help twice: less damage, and smart casters stop wasting turns on
them and switch to something else, which can be worse (a smart caster that learns you
resist fire may switch to its nether ball).

### Spell power and damage

Damage scales with spell power (= level by default). The most important formulas
(`monster_spell.txt`; average values in [A3](appendices/A3-monster-spells.md)):

| Spell | Damage | At power 20 | 50 | 80 | 100 |
|---|---|---|---|---|---|
| WOUND (save) | `(⌊P/3⌋×2)d5` | 36 | 96 | 156 | 198 |
| BO_FIRE | `P/3 + 9d8` | 46 | 56 | 66 | 73 |
| BO_COLD | `P/3 + 6d8` | 33 | 43 | 53 | 60 |
| BO_NETH | `1.5P + 30 + 5d5` | 75 | 120 | 165 | 195 |
| BO_MANA | `50 + 1d(2.5P)` | 75 | 113 | 150 | 175 |
| BO_PLAS | `P + 10 + 8d7` | 62 | 92 | 122 | 142 |
| BO_ICE | `P + 6d6` | 41 | 71 | 101 | 121 |
| BA_FIRE | `10 + 1d(3.5P)` | 45 | 98 | 150 | 185 |
| BA_ACID | `15 + 1d(3P)` | 45 | 90 | 135 | 165 |
| BA_NETH / BA_DARK | `4P + 10d10` | 135 | 255 | 375 | 455 |
| BA_MANA | `5P + 10d10` | 155 | 305 | 455 | 555 |
| BA_WATE | `50 + 1d(2.5P)` | 75 | 113 | 150 | 175 |
| STORM (3 balls at once) | water `30+(P/3)d5`, elec and ice `20+(P/3)d5` each | ~124 | ~214 | ~304 | ~367 |
| MIND_BLAST (save) | `8d8` + confusion | 36 | 36 | 36 | 36 |
| BRAIN_SMASH (save) | `12d15` + slow, conf, paralysis, blind | 96 | 96 | 96 | 96 |
| DRAIN_MANA | `1d(P/2+1)` SP, heals monster 6×drained | ~6 | ~13 | ~21 | ~26 |
| HEAL | `6P` HP | 120 | 300 | 480 | 600 |

Breaths: `current HP / divisor`, capped (fire, cold, acid, lightning, mana: /3 cap
1600; poison /3 cap 800; gravity /3 cap 200; time /3 cap 150; light, dark, sound,
shards, disenchantment /6 cap 500; nether, chaos /6 cap 600; nexus /6 cap 400; inertia,
force /6 cap 200; plasma /6 cap 150; `lib/gamedata/projection.txt`,
`src/mon-spell.c:breath_dam()`). Breaths are 30-degree cones at full strength out to 7
grids (11 for `POWERFUL` monsters), then fall off as `8/(distance+1)`
(`src/effect-handler-attack.c:effect_handler_BREATH()`, `src/project.c`). **Damaging a
breather reduces its breath.** Resistances are in [Chapter 5](05-elements-and-status.md).

---

## 8.8 Summoning

FACT (`src/effect-handler-general.c:effect_handler_SUMMON()`,
`src/mon-summon.c:summon_specific()`, `lib/gamedata/summon.txt`):

- Summoned monsters appear **next to the summoner** (up to 4 grids away), not next to
  you, and only on grids that allow it (not on glyphs of warding).
- The summoner keeps summoning, up to the spell's maximum count (1 for S_MONSTER,
  S_DEMON, S_UNDEAD, S_DRAGON, S_AINU; 6 for S_HYDRA; 8 for S_KIN, S_MONSTERS, S_HI_*,
  S_WRAITH, S_UNIQUE; 12 for S_ANIMAL, S_SPIDER, S_HOUND), **until the sum of the squares
  of the summoned monsters' levels reaches `your depth × summoner level`**. So deep
  summoners call more, and each summon of a high-level monster counts for a lot.
- Each summoned monster is drawn at level `(depth + summoner level)/2 + 5`, from the
  allowed set:

| Summon | Allowed | Uniques? |
|---|---|---|
| S_MONSTER, S_MONSTERS | anything | no |
| S_KIN | same base as the summoner | no |
| S_ANIMAL | `ANIMAL` flag | no |
| S_SPIDER | `S` | no |
| S_HOUND | `Z` and `C` | no |
| S_HYDRA | `M` | no |
| S_AINU | `A` | no |
| S_DEMON / S_UNDEAD / S_DRAGON | flag | no |
| S_HI_DEMON | `U` (major demons) | **yes** |
| S_HI_UNDEAD | `V`, `W`, `L` | **yes** |
| S_HI_DRAGON | `D` (ancient dragons) | **yes** |
| S_WRAITH | unique `W` (the Nazgûl), else S_HI_UNDEAD | yes |
| S_UNIQUE | any unique, else S_HI_UNDEAD | yes |

- No summoning on arena levels.
- Summoned monsters are awake and join the summoner's group.

ADVICE:

- Fight summoners where they have no empty floor within 2 grids: in a corridor with you
  next to them, the "summon possible" check fails and they lose that option.
- Kill summoners first, from range if possible, and never stand in a room with a
  quylthulg (`Q`): they are invisible, never move, and summon every other turn.
- S_UNIQUE and S_HI_* at DL80+ can bring Nazgûl, Great Wyrms and Balrogs. Leave rather
  than fight a summoning unique if you are not ready for its summons too.

---

## 8.9 Regeneration, doors, walls, items, webs, wards

### Regeneration

FACT (`src/mon-move.c:regen_monster()`): every 100 game turns (10 player turns at normal
speed) a hurt monster regains `max(1, maxHP/100)` HP, ×2 with `REGENERATE` (trees and huorns,
vampires and vampire bats, skeleton trolls/ettens, the spirit troll, Rogrog, the
9-headed hydra, gelugons, horned reapers, pit fiends, Old Man Willow, Thuringwethil,
Sauron and Morgoth). Spells
(HEAL `6 × power`, DRAIN_MANA) heal much faster. Morgoth regenerates 400 HP per 10 normal
turns, Sauron 160.

### Doors

FACT (`src/mon-move.c:monster_turn_can_move()`):

- `OPEN_DOOR` monsters open closed or secret doors; `BASH_DOOR` monsters smash them
  ("You hear a door burst open!") and may fall into the doorway. Monsters with both
  sometimes bash anyway.
- An unlocked door costs them one turn. Locked doors: each try succeeds only if
  `randint0(HP/10)` exceeds the lock power, reducing the lock by one each time: big
  monsters break locks fast, monsters with fewer than `10 × (power + 2)` HP never do. You can
  lock doors yourself with `D` (power `m_bonus(7, depth)`,
  `src/cmd-cave.c:do_cmd_lock_door()`).
- No door ability (insects, worms, jellies, molds, many reptiles and birds): a closed door
  stops it. **Dogs and almost all hounds have `BASH_DOOR`**; only wall-passers like hounds
  of Tindalos ignore doors entirely.
- `PASS_WALL` monsters go through doors and walls; `KILL_WALL` monsters destroy them;
  `SMASH_WALL` (Morgoth) destroys the wall and much around it.

### Walls

`PASS_WALL`: all `G` ghosts, Xorns/Xarens, earth spirits/elementals, some drakes and
dragons, shadows, dreads, elder vampires, nether wraiths, the archlich, hounds of
Tindalos, death quasits, furies, Maia of Aulë, Tselakus. `KILL_WALL`: umber hulks, abyss
worm masses, black reaver, hasty ents, Makar, Maeglin, storm of Unmagic. A monster
inside a wall cannot be hit by bolts or balls aimed at its grid (the projection stops at
the wall; [official manual](https://angband.readthedocs.io/en/latest/attack.html)), but
can be hit in melee by tunnelling toward it or when it steps out.

### Items

FACT (`monster_turn_grab_objects()`): `TAKE_ITEM` monsters pick up floor items,
`KILL_ITEM` monsters destroy them. Artifacts, gold, mimicked items and items that would
slay the monster are left alone. Do not leave your stash where orcs, trolls, giants and
uniques walk.

### Pushing and trampling

`MOVE_BODY` monsters swap places with weaker monsters (lower experience value);
`KILL_BODY` monsters kill and replace them. Neither works on uniques.

### Webs and wards

- Spiders' WEAVE spell makes webs; `PASS_WEB` monsters ignore them, `CLEAR_WEB`
  monsters spend a turn clearing, others are stuck (`monster_turn()`).
- A **glyph of warding** stops a monster from entering; each attempt breaks it with
  chance `level/550` (`constants.txt` `break-glyph:550`), and it can try up to five
  directions per turn. Morgoth breaks one 18% of the time per attempt; a DL40 monster
  7%. No summons appear on glyphs.

---

## 8.10 Breeders

FACT (`src/mon-move.c:monster_turn_multiply()`, `multiply_monster()`,
`constants.txt`):

- Each turn, a `MULTIPLY` monster counts monsters in the 3×3 block around it
  **including itself** (`k`). If `k < 4`, it breeds with chance `1/(8k)`: 1 in 8 alone,
  1 in 16 with one neighbour, 1 in 24 with two, never with three or more neighbours.
- The child appears on an adjacent empty grid, awake, not in any group.
- A level holds at most **100** bred monsters (`repro-max:100`); after that breeding
  stops. Uniques never breed. No breeding in single-combat arenas.
- Breeding takes the monster's turn.

Breeders (`MULTIPLY`): worm masses `w` (white, green, yellow, clear, blue, red, nether,
abyss, disenchanter), lice and insects `I` (giant white/black louse, giant fruit fly, giant
flea, hummerhorn, neekerbreeker, giant firefly), rodents `r` (giant white mouse, giant
white rat, silver mouse, giant grey rat), the clear mushroom patch, the blue icky thing,
the black ooze.

ADVICE:

- Kill the first breeder you see at once, or leave the level. A colony doubles fast in
  open rooms; in a corridor each breeder has fewer free grids, so a colony grows slowly
  and can be "farmed" for experience only if it is weak (white worm masses).
- Dangerous colonies: **hummerhorns** (L18, confusing bites, `MOVE_BODY`), **giant
  fireflies** (blinding), **neekerbreekers** (shriek, wakes the level), **disenchanter
  worm masses** (disenchant your gear), **nether/abyss worm masses** (exp drain; abyss
  ones are invisible and tunnel), black oozes (drain mana, acid, `KILL_BODY`).
- A borg should never rest while a breeder colony is growing on its level.

---

## 8.11 Seeing monsters: invisibility, ESP, infravision, detection, light

FACT (`src/mon-util.c:update_mon()`, `src/mon-predicate.c:monster_is_esp_detectable()`):
a monster within max sight (20) is visible if any of these hold:

1. **Telepathy** (`TELEPATHY` flag) and the monster is ESP-detectable, unless you or it
   stand on a no-ESP grid. **`EMPTY_MIND` monsters are never detected** (golems,
   elementals, vortices, jellies, molds, mushrooms, quylthulgs, lurkers, mimics, creeping
   coins, skeletons, zombies, icky things and many more). **`WEIRD_MIND` monsters are
   detected only for one individual in ten** (those whose monster index ends in 5): ants,
   centipedes, insects, dragon flies, beetles, spiders, worm masses, some trees.
2. **Infravision**: in line of sight, not blind, within your infravision range, and the
   monster is **not** `COLD_BLOOD`. This check comes **before** invisibility, so a
   warm-blooded invisible monster inside your infravision range is seen. Ghosts,
   undead, golems, air spirits, imps and lurkers are cold-blooded, but many invisible
   monsters are not: **all quylthulgs**, quasits and death quasits, Sméagol, Draebor,
   clear hounds/worms/centipedes/icky things, displacer beasts, spirit nagas, mystics,
   shadow/ethereal drakes, death drakes, ethereal dragons, will o' the wisps, osyluths,
   hounds of Tindalos, Tevildo, Pazuzu, the Mouth of Sauron (`monster.txt`). A few grids
   of infravision is a partial See Invisible against these.
3. **Light**: in line of sight on a lit grid, and either not `INVISIBLE` or you have
   **See Invisible**.
4. **Detected** this turn (detection spells mark monsters until your next move).

Not visible means: half melee and missile to-hit
([Chapter 4](04-combat.md#43-to-hit)), no targeting, "it" in messages.

### Detection effects

`src/list-effects.h`: DETECT_VISIBLE_MONSTERS (not invisible ones), DETECT_INVISIBLE_MONSTERS,
DETECT_LIVING_MONSTERS (not undead or `NONLIVING`), DETECT_EVIL, DETECT_FEARFUL_MONSTERS,
DETECT_SOUL (monsters with `SPIRIT`). Which class spells and devices give them is in
[Chapter 3](03-magic-and-spells.md) and [Chapter 7](07-consumables-and-devices.md).
ADVICE: no single detection sees everything; "visible" plus "invisible" covers all
monsters, ESP covers all minds; pair ESP with a detection to catch empty minds.

### Camouflage

Mimics (`UNAWARE` + `MIMIC_INV`, e.g. potion/scroll/ring/chest mimics, creeping coins)
look like items, lurkers and trappers (`x`) look like floor. They become known when they
act, are hit, or are detected. A borg should treat an unexpected item in a vault or a
lone "coin pile" at depth with suspicion: walking onto a trapper (L36) means 2 paralyzing
hits per turn.

### Monster light and darkness

Some monsters carry light (`light:` 1–3: many humans, fire creatures, light hounds) and
light the grids around them, so you see them coming in dark corridors; their attacks are
always "visible" (`mon-attack.c`). Negative light (dark hounds, abyss worm masses, some
undead and Ungoliant-type monsters) darkens the area around them. `HURT_LIGHT` monsters
(orcs, trolls, many undead, `e` eyes, worm masses) take extra damage from light
attacks.

---

## 8.12 Monster spells that matter and their counters

ADVICE built on FACT from `monster_spell.txt` and the effect handlers.

| Spell | What it does | Counter |
|---|---|---|
| **TELE_TO** | Teleports you next to the monster | Not blocked by rNexus. Be ready to escape again; fight casters with it only if you can beat them adjacent |
| **TELE_LEVEL** | Moves you up or down a level (save allowed) | **rNexus blocks it** (`effect_handler_TELEPORT_LEVEL`); on a quest level (DL99/100) it can only send you **up** |
| **TELE_AWAY** | Teleports you 100 grids away | Annoying, rarely deadly; can separate you from your loot or stairs |
| **BLIND** | 11–15 turns blind (save) | rBlind; carry CLW/CSW (cure blindness); you cannot read scrolls while blind |
| **CONF** | 3–7 turns confused (save) | pConf (rConf); CLW/CSW cure; confused you cannot read scrolls |
| **SCARE** | 3–7 turns afraid (save) | pFear, Heroism, Berserk; afraid you cannot melee |
| **SLOW** | −10 speed, 3–7 turns (save) | **Free Action blocks it** (`player_timed.txt` SLOW `fail:FREE_ACT`); Haste offsets |
| **HOLD** | 3–7 turns paralysed (save) | **Free Action**; essential by DL20 |
| **FORGET** | Amnesia (save): **you cannot read scrolls** (`src/player-util.c:player_can_read()`), spell failure becomes `50% + half the normal`, device skill −20% | High saving throw; keep a non-scroll escape (staff/rod of teleport, potions) for when amnesiac |
| **MIND_BLAST / BRAIN_SMASH** | 36 / 96 damage plus confusion (and slow, paralysis, blindness for Brain Smash) (save) | Saving throw, pConf, FA, rBlind |
| **DRAIN_MANA** | Drains SP and heals the monster 6× the amount | Casters: kill fast or leave; non-casters immune ("the draining fails") |
| **WOUND** | Up to ~200 damage late game, plus cuts (save) | High saving throw; often the biggest non-breath spell of mid/late casters |
| **Summons** | See [8.8](#88-summoning) | Corridors, kill summoner first, leave |
| **HEAL / HEAL_KIN** | 6 × power HP | Burst it down; do not trade slow damage with healers |
| **BLINK / TPORT** | Monster teleports | Expect it back; it does not leave the level |
| **TRAPS** | Creates traps around you | Do not walk blindly; Disarm or walk around |
| **DARKNESS** | Unlights the area, weak dark ball | rDark or just relight; blocks some attacks only by losing sight |
| **SHRIEK** | Wakes the level and hastes (+10 for 25 turns) every monster in its line of sight | Kill shriekers (`,` shrieker, neekerbreekers, craban, terrified yeeks) from range quickly or leave |
| **HASTE** | Monster +10 speed for 50 turns | Slow Monster, or avoid |
| **WEAVE** | Webs around the spider | Burn them (fire) or clear/tunnel; stops your escape by walking |
| **SHAPECHANGE** | Monster becomes another race for 5–15 turns, keeping HP | Recall the new form (Sauron, Beorn, druids) |
| **ARROW / BOLT / SHOT / BOULDER** | Physical missiles, roll vs your AC | AC; corners |

---

## 8.13 Monster families by glyph

For each family: what it is, depth range (from `monster.txt`), dangerous members, how to
deal with it. Base flags come from `monster_base.txt`. Full lists in
[A1](appendices/A1-monsters-by-depth.md).

### `a` Ants (L1–42)

Animals with `WEIRD_MIND` (mostly invisible to ESP). Early ants are trivial. Giant silver
ants (L23) spit acid; giant grey ants (L26, AC 200) trample weaker monsters; giant army
ants (L30, +10, groups, `KILL_BODY`) and giant fire ants (L35, groups, 2×3d12 fire bites)
swarm. **The Queen Ant** (L42, 1500 HP, summons kin 50% of turns) sits in an ant nest:
kill her fast or leave. Slay animal works on all.

### `A` Ainur / Maiar (L34–99)

Powerful spirits: immune to fire, cold, poison (base), no confusion/sleep/hold, take
items. Maia (L34) heal, haste, blind, confuse. The named Maiar (Nienna, Mandos, Oromë,
Yavanna, Aulë, Ulmo, Manwë, Varda; L43–71) have 1250–5000 HP, shapechange, and summon
other Ainur; Maia of Ulmo/Varda have paralyzing touches. Uniques: **Saruman** (L60),
**Ossë** (L72), **Makar** (L76), **Meássë** (L87), **Sauron** (L99). Not evil (except
Saruman and Sauron), so slay evil does nothing to most Maiar. ADVICE: they are rarely
worth fighting; avoid before you have FA, rConf and big damage.

### `b` Bats (L1–32)

Fast, erratic (`RAND_25/50`), annoying rather than deadly. Vampire bats (L24, `REGEN`,
EXP_40 bites), disenchanter bats (L26, +20, disenchant touch), bat of Gorgoroth (L28,
poison/dark breath), doombats (L32, +20, 3×5d4 fire bites). Erratic movement makes them
hard to hit with bolts; melee them when they come adjacent.

### `B` Birds (L2–54)

Crows (early), crebain (L16, shriek), blood falcons (L20, +30, 2d10/2d10/3d8 but 6 HP),
giant rocs (L40, 182 melee per turn, elec), winged horrors (L48, nether/dark/poison
breath, EXP_40). **The Phoenix** (L54) breathes fire 1200, light 500, plasma 150. Slay
animal applies.

### `c` Centipedes (L1–25)

Weird minds, early fodder. **Carrion crawler** (L25, two PARALYZE stings 2d6, groups of
2d4): one of the classic killers without Free Action.

### `C` Dogs (L0–99)

Packs (`GROUP_AI` on the pack members), fast. Grip and Fang (L2) are the first uniques;
jackal-like wild dogs (L1), wolves (L10, +10, packs 2d10), white wolves, wargs (L14),
blink dogs (L18, teleport-to), werewolves (L20, 230 HP), wolf chieftain (L26, 422 HP, with
wolf escort), hellhounds (L42, +10, packs of 2d5, 3×3d12 fire bites, fire breath 117
each): a hellhound pack without rFire is a death sentence. Werewolf of Sauron (L63).
Uniques: **Draugluin** (L83), **Huan** (L90, not evil), **Carcharoth** (L90), and
**Wolf-Sauron** (Sauron's +30 shape). Dogs bash doors (`BASH_DOOR`); the bigger ones
also open them. A *locked* door of power 2–3 stops wolves and wargs (21–36 HP) for good.

### `d` Dragons, young and mature (L9–43)

`DRAGON`, `EVIL`, wake rarely (`FORCE_SLEEP`). Baby dragons (L9–11, 88–114 HP, breath ~29–38)
are a real threat at DL5–10. Young dragons (L29–32, ~240–280 HP, breath 79–93) and
mature dragons (L39–43, 700–1000 HP, breath 233–333) breathe their element. Multi-hued
ones breathe all five base elements. Drakes (L33): chaos, law (shards/sound), balance,
ethereal and shadow (invisible, pass walls), crystal (shards). Pseudo-dragon (L10, light
and dark breath). **Kavlax** (L41) breathes nine elements. Slay dragon ×3/×5; damage
them first to weaken the breath; rBase by DL30.

### `D` Ancient dragons and wyrms (L50–85)

1500–5200 HP, `POWERFUL` (breath full strength to 11 grids), smart, summon kin (wyrms).
Ancient dragons (L50–53) breathe ~500 base element (1/3 of HP); death drakes (L50,
invisible, pass walls, nether ~325, EXP_80 bite), great crystal drakes (shards),
ethereal dragons (light/dark, invisible, pass walls), dracolich (L55, cold 1026,
nether 513), dracolisk (L55, fire 1026, nexus 400, **paralyzing gaze**, HOLD). Great
wyrms (L70–82): storm/ice/swamp/hell/bile/thunder (one element, 1400–1500 for base
elements), chaos, law, balance, many colours, annihilation (poison, dark, nether, chaos,
**time**), sky dragon (lightning, light, **gravity**, plasma, force). Uniques:
**Itangast**, **Scatha**, **Smaug**, **Glaurung**, **Ancalagon**. ADVICE: full base
resists plus rPois are mandatory for any `D`; rNether, rChaos, rDisen, rSound, rShards
for the special ones. Weaken with damage before letting them breathe (a full-HP great
hell wyrm breathes 1500 fire, ~500 resisted, ~167 double resisted).

### `e` Floating eyes (L1–73)

Most never move (`NEVER_MOVE`; spectators, gauths and the beholders do move), gaze attacks. **Floating eye**
(L1): paralyzing gaze; meleeing it without Free Action can paralyse you until death
(each paralysis is non-stacking but a fresh one follows when it ends). Shoot it or walk
away. Radiation eye (STR drain gaze), disenchanter eye, bloodshot eye (blind), evil eye
(L18, HOLD and TELE_TO), spectator (L28, paralyze/confuse gaze, FORGET, HOLD), gauth
(disenchant, drain charges), **beholder** (L40, +10, 1400 HP, 50% spells: forget, mind
blast, bolts, darkness, slow), undead beholder (L45, brain smash, mana bolts),
beholder hive-mother (L67), **Omarax** (L73). `HURT_LIGHT` (most): light attacks.

### `E` Elementals and spirits (L12–65)

`EMPTY_MIND` (no ESP), `NONLIVING`, poison-immune, no fear. Air spirits (L12, +20,
invisible, erratic), earth spirits (pass walls), fire spirits; elementals (L33–37) bolt
their element and destroy items (`KILL_ITEM`); invisible stalkers (L35, +20, invisible,
erratic); will o' the wisps (L38, +20, invisible, pass walls, teleport); magma and ice
elementals. Uniques: **Vargo, Waldern, Quaker, Ariel** (L43). Great earth and water elementals
(L60–65) have rarity 0: they are never generated normally, only as shapechange forms
(`shape:` lines in `monster.txt`, e.g. for Quaker and the Maiar). ADVICE: carry
See Invisible by DL30; elementals' bolts are their main danger.

### `f` Cats (L0–66)

Fast melee animals: panthers (L10), tigers (L12), sabre-tooth tigers (L20, 4 blows),
displacer beasts (L26, invisible, AC 150). **Tevildo** (L66, +20, invisible, AC 240,
paralyzing bite 15d1 + confusion and blinding claws). Slay animal.

### `F` Dragon flies (L5–6)

Giant dragon flies breathe weak elements (~5 damage), erratic. Harmless except as a
distraction; good early experience.

### `g` Golems (L14–75)

`EMPTY_MIND`, never afraid, mostly slow and never chase far. Flesh/clay/stone/iron golems
are experience bags (iron golem L22: 520 HP, SLOW spell). Pukelman (L25: acid bolt,
confusion, slow), mithril golem (L30), Eog golem (L34, 1050 HP, 4 heavy blows, −10 speed),
colossus (L36, 2640 HP, 2×10d10, −10 speed), silent watcher (L35, never moves:
paralyzing gaze, summons, shrieks), colbran (L36, lightning), **drolem** (L54, +10, 2200
HP, poison breath **733**, arrows, blind/conf/slow): a classic killer of characters
without rPois. Bone golem (L68: brain smash, nether ball), bronze golem (L75). ADVICE:
slow golems can be fought with hit-and-run; never fight a drolem without rPois.

### `G` Ghosts (L3–68)

`UNDEAD`, `INVISIBLE`, `COLD_BLOOD` (infravision useless), `PASS_WALL`; immune to cold and
poison. Early: poltergeist (L3, +20, fast and erratic), green glutton ghost (eats food),
lost soul (drain mana, WIS drain), moaning spirit (scare, DEX drain). Mid: banshee (L24,
EXP_20), ghost (L31: EXP_20 touch, INT/WIS drain claws, blind, HOLD, drain mana),
shade/spectre (L34/37, EXP_40, stat drains, HOLD), phantom (L40, EXP_80, FORGET), **dread**
(L44, groups of 2d5, 263 HP each, blind/conf/hold, nether bolt 111, 6d6 blows):
a dread pack is a top mid-game killer. Shadows (L50, EXP_80, TELE_TO, nether bolt),
dreadmaster (L54, **TELE_LEVEL**, summon undead), dreadlord (L62, nether ball 303, brain
smash), **Tselakus** (L68). Spirit troll (L40, 880 HP, invisible, pass walls, regenerates).
ADVICE: See Invisible (or ESP; ghosts are not empty-minded) and Hold Life; they attack
out of walls where you cannot hit them back with bolts. Light-based attacks hurt the
`HURT_LIGHT` ones (poltergeist).

### `h` Dwarves, elves, hobbits (L0–81)

Spellcasting humanoids (open doors). **Sméagol** (L3), **Bullroarer** (L5). Eastern
dwarves (L7, confusion, darkness), **blacklock mages** (L10, +10: blind, confuse,
magic missile, stinking cloud), stonefoot warriors, **ironfist priests** (L12, +10:
blind, confuse, wound, heal), **dark dwarven lords** (L20, +10: blind, confuse, haste,
fire/cold bolts ~46), **Nár** (L17), **Ibun, Khîm, Mîm** (L24–27), **mind flayers**
(L28: brain smash 96, hold, forget, INT/WIS draining gaze), green elf archers (L29, AC
112, arrows 1-in-3), stiffbeard sorcerers (L41, +20, 50% spells, summons demons/undead),
**Eöl** (L49), **Fundin** (L59), Fëanorian raiders (L65), **Maeglin** (L81). ADVICE: the
dwarf casters at DL10–20 are the main source of early blindness and confusion; carry
CLW potions and kill them first. There are **no dark elven priests/mages in this
version**; the blacklock, ironfist and dark dwarven casters fill that role.

### `H` Hybrid monsters (L2–68)

Harpies (early, erratic), hippogriffs, griffons, chimaeras (L20, fire breath ~53),
gorgimaeras (L27, fire breath 87, **paralyzing gaze**), manticores (L30, arrows),
minotaurs (L40, +20, 550 HP, 4 butts), **Baphomet** (L58), jabberwocks (L68, chaos breath).

### `i` Icky things (L1–14)

Weak, erratic, slow. Bloodshot icky thing (drain mana), blue icky thing (L14, breeds,
blinds/confuses/scares). Free experience early.

### `I` Insects (L3–26)

All breeders, erratic, weird minds: lice, fruit flies, fleas, **hummerhorns** (confusion),
**neekerbreekers** (shriek), **giant fireflies** (blindness). See
[8.10](#810-breeders).

### `j` Jellies and oozes (L2–23)

Mostly never move, `STUPID`, `EMPTY_MIND`. They are experience bags with nasty touches:
white jelly (poison, 36 HP), yellow jelly (poison, drain mana), green jelly (acid),
silver jelly (eats light), red jelly (STR drain), grape jelly (L12: 234 HP, EXP_10 touch,
drain mana: shoot it). Moving ones: green/blue ooze, ochre jelly (L13, acid, picks up
items), gelatinous cube (L16, 316 HP, 3 acid touches, takes items), black ooze (L23,
breeds, drain mana, kills weaker monsters). ADVICE: kill stationary jellies with
missiles or spells, not melee, unless their touch is harmless to you (check recall).

### `J` Snakes (L1–99)

Slow early snakes with high AC (large white snake L1, AC 36; large grey snake AC 61) are
time-wasters. Cobras/mambas spit blindness, poison. Serpent of the brownlands (L32: HOLD,
blind, conf). Nightcrawler (L69: brain smash, nether breath), serpent of chaos (L78,
+20, chaos breath, TELE_TO). **Serpent-Sauron** is one of Sauron's shapes.

### `k` Kobolds (L1–7)

Poison-immune small humanoids, early fodder. Kobold shamans (magic missile), kobold
archers (arrows half the time), large kobolds (65 HP). **Mughash** (L7) leads a kobold
army.

### `K` Killer beetles (L13–45)

Armoured animals (AC 64–106), weird minds, no spells except spit. Killer slicer beetle
(L30: 2×7d8 bites), killer iridescent beetle (L37: paralyzing gaze), killer white beetle
(L45: 800 HP). Steady melee; mainly a test of your damage output.

### `l` Trees (L11–52)

Regenerate, `HURT_FIRE` (fire brands do ×4/×6), immune to water, mostly never move or are
slow. Old forest tree (L11, 800 HP, never moves, confuses). **Old Man Willow** (L19).
Huorns (L31, 1000 HP, move, haste, TELE_TO, darkness), black-hearted huorns (L42), hasty
ents (L52, +10, 3000 HP, `KILL_WALL`). ADVICE: fire.

### `L` Liches (L34–92)

`UNDEAD`, `SMART`, cold-blooded, immune to cold/poison, no conf/sleep/stun/hold.
Spellcasters with EXP drain and DRAIN_CHARGES touches. Lich (L34: brain smash, drain
mana, hold, tele_to/away, slow), master lich (L41, 1584 HP: + confusion, summon undead),
demilich (L54: + forget, heal, teleport, summon demons), archlich (L64, pass walls, nether
ball 311, summon high undead), black reaver (L74, `KILL_WALL`, mana/nether balls),
**Feagwath** (L77), **Vecna** (L92). ADVICE: FA, rBlind, rConf, high saving throw,
Hold Life, keep wands at home (drain charges); kill fast because they heal and
teleport.

### `m` Molds (L1–47)

Never move, `HURT_FIRE`. Spore attacks only when you touch or stand next to them.
Disenchanter mold (L10), red mold (L19, 4d4 fire), shimmering mold (L27, 2×5d4
lightning), **death mold** (L47, +30 speed, 1050 HP, 3×7d7 disenchant + EXP_80 spores).
ADVICE: just walk around molds; never melee a death mold.

### `M` Hydras (L17–57)

`ANIMAL`, `HURT_COLD` (cold brands ×4/×6), many bite blows, SCARE spells; 6–9-headed
hydras add poison/fire/acid breath and blinding/confusing bites; the 9-headed hydra
(L57) is smart, regenerates and summons hydras. Mostly a melee slugfest: cold brand and
slay animal are ideal.

### `n` Nagas (L3–48)

Slow, tough, `HURT_COLD`, crushing blows. Spirit naga (L28, invisible: blind, darkness,
mind blast, heal). Gorgon (L48: EXP_80 and paralyzing gazes, acid ball, plasma bolt,
summons hydras). Early nagas are good targets for slow characters.

### `o` Orcs (L6–23)

`ORC`, `EVIL`, `HURT_LIGHT` (most), groups. Snagas (L6), cave orcs (L7), orc trackers
and orc archers (arrows), half-orcs, uruks (L16, no light weakness, bring wargs and
archers), orc captains. Uniques: **Lagduf, Grishnákh, Golfimbul, Ufthak, Shagrat, Gorbag,
Bolg, Lugdush, Uglúk, Azog**. ADVICE: orc gangs are the main danger of DL5–25; fight them
in corridors, carry slay orc if you find it, and use light (`HURT_LIGHT`) area attacks.

### `O` Ogres (L13–35)

Giants (`GIANT` flag): slay giant works. Ogre (L13), black ogre, cave ogre (L26),
ogre shaman (L27: fire bolt, traps, HOLD, summons), ogre mage (L32: cold ball, heal,
HOLD, summons), ogre chieftain (L30). **Lokkak** (L35) with an ogre gang.

### `p` Humans (L2–65)

The biggest family (54 races): every kind of caster. Early: apprentices (blink, blind,
confuse), acolytes, cutpurses (steal gold), soldiers, scouts and kobold-level archers.
DL10–20: **Wormtongue** (L8), **Brodda** (L9), witches, drúadan mages, priests (L12:
summon kin, holy/light balls), warriors, **illusionists** (L13: blind, confuse, HOLD,
slow, haste self), druids (L13: stinking cloud and fire bolts ~44, HOLD, shapechange,
summon animals), rogues (steal items), **Ulfast**. DL20–40: blackguards, rangers
(arrows), paladins, Easterling champions, brigands (steal), **Sangahyando and Angamaitë**,
necromancers (L25: nether bolt 82, wound, summon undead), **Ulwarth, Uldor**, black
knights, **mages** (L28: blind, confuse, **TELE_AWAY**, bolts ~49, summon), **Beorn**,
southron assassins (STR drain), drúadan druids, mystics (invisible kickers), master
thieves (+20), **Ulfang**, dúnadan of Angmar, **Lorgan**, demonologists (summon demons),
death knights (nether bolt 102), **Castamir**, southron archers. DL40+: enchantresses and
sorcerers (L40, +20, 50% spells: summon dragons/undead, balls), patriarchs, **Gorlim**,
berserkers, lords of Carn Dûm, master mystics (paralyzing hits), **Harowen**,
**Ar-Pharazôn**, grand master mystics, **the Mouth of Sauron**. ADVICE: casters with
BLIND/CONF/HOLD/TELE_AWAY are dangerous out of proportion to their HP: rBlind, rConf
and FA neutralise most of this family, and line-of-sight tactics do the rest.

### `P` Giants (L25–69) and Morgoth (L100)

`GIANT`, slay giant. Hill/frost/fire/stone/cloud giants (L25–36) throw boulders
(`BOULDER`, `(P/7+1)d12`) and hit hard (up to 2×8d8 elemental). Cyclops (L45, 1050 HP,
boulders), storm giants (L49, **STORM** ~214, TELE_TO, lightning balls), lesser/greater
titans (L56/66, 4×confusing 9d9/12d12 hits, heal, summons). **Gilim** (L60), **Nan**
(L69). **Morgoth** uses the `P` glyph (his own base in `monster_base.txt`).

### `q` Quadrupeds (L9–39)

Bears (cave bear L9, grizzly L16, werebear L24), catoblepas (L29, terrify/blind gaze),
mûmakil (L35, 495 HP, escort for Easterlings), night mares (L39, EXP_80 bites,
confusion). **Beorn** in bear form is a `q`.

### `Q` Quylthulgs (L20–79)

Invisible, never move, never melee, `EMPTY_MIND` (no ESP), AC 1; they **only
summon** (or teleport you), 50–100% of turns: quylthulg (S_MONSTER), nexus quylthulg
(L32, TELE_AWAY every turn), rotting/demonic/draconic (L45, undead/demons/dragons),
greater versions (L71, high undead/demons/dragons), master quylthulg (L76: everything),
**Qlzqqlzuup** (L79: summons uniques, Ainur, everything, every turn). ADVICE: detect
invisible; a quylthulg in view must die this turn (any damage spell kills the small ones)
or you must break line of sight.

### `r` Rodents (L1–15)

Breeding mice and rats (poison bites, silver mice eat light). Wererats (L15: blink,
wound, stinking cloud, cold bolt). Kill early or leave.

### `R` Reptiles and amphibians (L1–86)

Rock lizards, frogs, salamanders (fire bite), giant salamanders (fire breath). **Basilisk**
(L28, +10, 310 HP, **paralyzing gaze**, 3×2d12 bites, poison breath 103) and **greater
basilisk** (L45, +10, 1010 HP, two paralyzing gazes 3d12, poison breath 336, nexus and
dark breath 168): both killers without FA. **The Tarrasque** (L86). `HURT_COLD`.

### `s` Skeletons (L5–84)

`UNDEAD`, `EMPTY_MIND` (most), never afraid, cold/poison immune. Skeleton kobolds/orcs/
humans are fodder; skeleton trolls and ettens regenerate. Druj (hand L57, eye L66, skull
L74): never move or melee, cast every turn (`1_IN_1`): bolts of all elements, brain
smash, nether balls, TELE_AWAY. **Cantoras** (L84). ADVICE: a druj in view casts every
turn; break line of sight or kill it with one burst.

### `S` Spiders (L2–75)

Webs (`PASS_WEB`), poison, weird minds, many `GROUP_AI` packs. Cave spiders (L2, +10,
swarms of 2d8): a swarm kills many CL1–3 characters. Giant white ticks (AC 150),
Mirkwood spiders (L20, +10, packs, poison, webs), **phase spiders** (L23, +10, packs:
blink and TELE_TO you into the pack), giant tarantulas, scorpions (STR drain), abyss
spiders (L38: poison/dark breath, confusion, slow, blindness), ancient spiders (L48:
summons spiders, hold, drain mana), spiders of Gorgoroth (L63, paralyzing bite), **Shelob**
(L55), **Ungoliant** (L75). ADVICE: rPois by DL20 makes most spiders easy; FA for the
deep ones; area attacks for packs.

### `t` Townspeople (L0)

Harmless or thieves (urchins and squint-eyed rogues steal). Kill or avoid; no danger.

### `T` Trolls (L17–42)

`TROLL`, `EVIL`, most `HURT_LIGHT`, strong melee, groups. Ordinary trolls do **not**
regenerate in this version (only Rogrog, the spirit troll `G` and skeleton trolls have
`REGENERATE`). Forest trolls (L17), stone trolls (L25), snow
trolls, cave trolls (L33), troll priests (blink, wound, darkness), olog (L36, smart, no
light weakness), mountain trolls, ettens (L38, 1320 HP), troll chieftains (L40, +10).
Uniques: **Bert, Bill and Tom** (L30), **Rogrog** (L42). ADVICE: slay troll, light area
attacks, corridors.

### `u` Minor demons (L8–40)

`DEMON`, `EVIL`, fire-immune. Lemures (L8, slow groups), tengus (L10, +10, TELE_TO),
homunculi (L15, paralyzing hit), **quasits and imps** (L16–17, invisible, **TELE_LEVEL**,
TELE_TO, blink, blind, conf), nrulings (L25, +10, fire bolts, blink to you), bodaks (L36,
fire balls, summon demons), death quasits (L40, +20, invisible, pass walls). **Draebor**
(L28). ADVICE: See Invisible and rNexus (stops their teleport-level) by DL20.

### `U` Major demons (L40–95)

`DEMON`, `EVIL`, no conf/sleep/hold. Vrocks (L40, 2×8d12 crush), glabrezu, nalfeshnee
(fire breath), mariliths (L47, 4×4d6), **lesser Balrogs** (L49, +10, fire breath 586,
DRAIN_CHARGES), **the Balrog of Moria** (L50), hezrous, barbazu, bile demons (L61, acid breath 821,
poison 800), osyluths (L65, +20, invisible), gelugons (L69, +20, cold breath 1026,
shards 500, paralyzing hit), horned reapers (L72, +20, 4×11d11), pit fiends (L77, +20,
chaos/fire/poison breath), furies (L80, pass walls, brain smash, blink to you), greater
Balrogs (L85, fire 1500, plasma), **Pazuzu** (L82), **Lungorthin** (L88), **Gothmog**
(L95). ADVICE: slay/kill demon; rFire; rChaos for pit fiends; FA for gelugons.

### `v` Vortices (L21–95)

`EMPTY_MIND`, erratic, never afraid; engulf and breathe their element (breath from ~45
HP: small at first). Fire/cold/water/energy vortices (L21–29) destroy inventory more than
they hurt. Nexus (L37, stat swap/teleport), plasma, shardstorm, **time vortices** (L38,
+20, EXP_20 engulf and time breath 58: stat/exp drain), shimmering vortices (L38, +30,
blinding light breath, shriek), chaos vortices (L53), aether vortices (L60, +20, all 18
breaths), **Wiruin** (L80), storm of Unmagic (L95, +30: exp drain, drain charges,
disenchant, all-stat drain, chaos/disen/time breath). ADVICE: kill at range. They
cannot open doors but can bash them (`BASH_DOOR` in the base).

### `V` Vampires (L27–99)

`UNDEAD`, `REGENERATE`, `HURT_LIGHT`, `HURT_FIRE`, cold-blooded. EXP_20–80 bites, WOUND,
DARKNESS, MIND_BLAST, HOLD, TELE_TO. Vampire (L27), master vampire (L36, nether bolt),
vampire lord (L46, 1400 HP, brain smash, drain mana), elder vampire (L59, invisible, pass
walls, summon undead), **Thuringwethil** (L67), **Vampire-Sauron**. ADVICE: Hold Life, FA;
fire brands do ×4/×6 (`HURT_FIRE`); light attacks hurt them.

### `w` Worms (L1–40)

Mostly breeders ([8.10](#810-breeders)). Purple worms (L29, acid/poison, not breeders),
wereworms (L35, EXP_20 gaze).

### `W` Wights and wraiths (L24–85)

`UNDEAD`, `HURT_LIGHT`, EXP drain touches, WOUND/DARKNESS/SCARE/HOLD spells, nether bolts.
Forest wight (L24), grave wight (L30), barrow wight (L33, EXP_40, HOLD), emperor wight
(L37, 2×EXP_80, nether bolt), white/grey/black wraiths (L38–49), nether wraiths (L54,
invisible, pass walls, nether beam), nightwings (L61), nightwalkers (L73, 4 disenchant
hits), and the **nine Nazgûl** (L77–85: Uvatha, Adunaphel, Akhorahil, Ren, Ji Indur, Dwar,
Hoarmurath, Khamûl, the Witch-King) with **BLACK_BREATH** blows. ADVICE: Hold Life is
the key item; FA; light attacks.

### `x` Lurkers (L14–36)

Invisible, never move, look like floor (`UNAWARE`): lurkers (L14, 176 HP), **trappers**
(L36: 2×3d8 plus 2×paralyzing 15d1 hits). With FA they are free experience.

### `X` Xorns (L16–40)

Umber hulks (L16, `KILL_WALL`, **confusing gaze**, tunnels to you), xorns and xarens (L36/40,
pass walls, 4 blows, eat items). ADVICE: rConf by DL20 makes umber hulks harmless.

### `y` Yeeks (L2–13)

Harmless (the "terrified yeek" only shrieks). **Orfax** and **Boldor** lead yeek
groups. Boldor's and Orfax's drops are good for their depth.

### `Y` Yetis (L9–20)

Cold-immune melee animals: yeti (L9), sasquatch (L20, +10).

### `z` Zombies (L7–38)

`UNDEAD`, slow-witted (`EMPTY_MIND` for the mindless ones), `HURT_FIRE`. Zombified
kobolds/orcs/humans, mummified orcs/humans/trolls are fodder. **Ghouls** (L26: 2×
paralyzing claws, HOLD spell, packs of 4d4) and **ghasts** (L30, +10, paralyzing claws,
CON drain, escorted by 4d4 ghouls and assorted zombies) are FA tests. Mummified chieftain (L38: many
spells, summon undead).

### `Z` Zephyr hounds (L15–74)

Packs (`GROUP_AI`, friends 2d7) that **breathe** on sight: light/dark/clear (L15),
fire/cold/energy (L18), earth (shards), air (poison), water (acid) (L20), vibration
(sound, stuns) and nexus (L27), gravity/inertia/impact (L35), multi-hued (L43, all five
base elements 73 each), nether/time/plasma (L51), chaos (L64), hounds of Tindalos (L69,
+20, invisible, pass walls: nether, disenchantment, time), aether hounds (L74, +20, all
18 breaths). Each hound breathes for `HP/3` or `HP/6`, so a pack of 8 can deal 8× that in
one turn. ADVICE: they ambush corridors ([8.5](#85-group-ai)); balls and area spells kill
packs; the deadly ones are **gravity** (35: damage, slow, stun, blinks you around),
**inertia** (−10 speed), **nexus** (teleports, stat swaps), **time** (stat and exp
drain, +20 speed), **plasma** (stun) and **chaos**. Hounds bash doors (nether, time and
plasma hounds also open them), so an unlocked door only delays them a turn; a locked door
stops a hound for good once the lock power is at least `HP/10 − 1` (power 1 stops the
21 HP light/dark/clear hounds, power 2 the 35 HP fire/cold/energy hounds, power 5 the
68 HP earth/air/water hounds).

### `$` Creeping coins, `,` mushrooms, mimics (`! ? = ~`)

Coins (L4–27) look like gold, poison touches, drop gold; adamantite coins (L27) +10 speed.
Mushrooms: shriekers wake and haste the level; yellow (terrify), spotted (poison), purple
(CON drain), magic mushroom patches (blink, darkness, scare, slow), memory moss (L32,
FORGET). Mimics: potion (L18), scroll (L21), ring (L29), chest (L35) mimics cast bolts,
blindness, confusion, and summon.

---

## 8.14 Killers by depth

FACT for each monster's numbers (`monster.txt`, computed averages); ADVICE for "why it
kills" and the counter. Grouped by native depth band; out-of-depth monsters appear
earlier.

### DL1–5

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Grip / Fang, Farmer Maggot's Dogs | 2 | +10 speed: 2 bites per turn while you swing once, 25 HP | Flasks of oil, fight with a full HP bar |
| Cave spider swarm | 2 | +10, swarms of 2d8, surround you | Corridor, flasks, leave |
| Floating eye | 1 | Paralyzing gaze when you melee it | **Never melee it without FA**; shoot or ignore |
| Bullroarer | 5 | +10, 60 HP, 2×1d8, with hobbit thieves | Flasks, corridor |
| Kobold archers, apprentices | 2–4 | Arrows; blink/blind/confuse | Close fast; CLW |
| Shrieker patch | 2 | Wakes and hastes everything in sight | Kill it from range first |

### DL5–10

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Wormtongue | 8 | Frost bolt ~29, poison ball ~17, SLOW, TRAPS, heals 48, 250 HP | Close and melee; potions of CLW; bring 30+ HP margin |
| Mughash and kobold army | 7 | Numbers: archers, shamans, large kobold bodyguards | Corridor |
| Lagduf, Grishnákh + wolves + orcs | 8–10 | 190–230 HP uniques with 4 blows (~21–24/turn) and gangs | Corridor, flasks |
| Brodda | 9 | 4×1d12 (~26/turn), 210 HP | Melee with full HP, flasks |
| Baby dragons | 9–11 | 88–114 HP, breath ~29–38 | Missiles; rBase helps |
| Blacklock mages | 10 | Blind, confuse, stinking cloud | CLW cures; kill first |
| Orfax | 10 | +10, TELE_TO, confusion, heals, summons | Kill quickly; low HP (120) |

### DL10–20

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Golfimbul's war party (with Bullroarer, Grishnákh, wargs, orcs) | 12 | Three uniques at once | Separate them in corridors; leave if you can't |
| Ironfist priests, illusionists, dark dwarven lords | 12–20 | Blind/confuse/hold, fire bolts ~46 | FA, CLW/CSW, kill on sight |
| Nár | 17 | 450 HP, blind, confuse, mind blast 36, wound 30, heals 102 | rBlind/rConf ideal; burst him down |
| Umber hulk | 16 | Confusing gaze, tunnels straight to you | rConf; melee it |
| Hummerhorns | 18 | Breeding, confusing bites | Kill first one or leave |
| Homunculus, ghouls (deeper) | 15+ | Paralysis | FA |
| Quasits, imps | 16–17 | Invisible, TELE_LEVEL, TELE_TO | SI, rNexus |
| Fire/cold/energy hounds | 18 | Pack breath ~11 each × many | Corridor ambush rules, rBase |
| Shagrat, Gorbag (+ uruk gangs) | 19 | 400 HP, ~45/turn | Corridor, slay orc |
| Old Man Willow | 19 | TELE_TO + HOLD + paralyzing touches, 1000 HP | **FA**; or avoid line of sight entirely |

### DL20–30

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Bolg, Azog | 20/23 | +10 speed, 500/900 HP, ~42–45/turn, uruk bodyguards | Haste, slay orc, or skip |
| Mirkwood spiders | 20 | +10 packs, poison | rPois, area attacks |
| Phase spiders | 23 | Blink/TELE_TO you into the pack | Kill at range; don't fight in the open |
| Sangahyando + Angamaitë | 24 | Pair, 800 HP each, ~56/turn each, confuse/slow | rConf, FA, fight one at a time |
| Carrion crawlers | 25 | 2 paralyzing stings each, groups of 2d4 | **FA** |
| Ghouls, ghasts | 26/30 | Paralyzing claws + HOLD, packs of 4d4 | FA |
| Nexus hounds | 27 | Teleport, stat swap | rNexus |
| Gorgimaera, basilisk | 27/28 | Paralyzing gaze; basilisk +10 and poison breath 103 | FA, rPois |
| Mind flayer | 28 | Brain smash 96, hold, forget | FA, saving throw |
| Mages | 28 | Blind, confuse, TELE_AWAY, bolts ~49 | rBlind, rConf |
| Draebor | 28 | Invisible, TELE_LEVEL/TO/AWAY, summons imps | SI, rNexus |
| Young dragons | 29–32 | Breath 79–93 | rBase |
| Bert, Bill, Tom | 30 | Three +10 trolls, 1100 HP each, ~58/turn each | Corridor, slay troll, light attacks; or leave |

### DL30–40

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Lich | 34 | Brain smash, drain mana, hold, tele_to/away | FA, rBlind, rConf, saving throw |
| Lokkak and ogres | 35 | +10, 1500 HP, ~63/turn, ogre mages | Corridor, slay giant |
| Gravity hounds | 35 | Gravity breath 64 each + blink + slow + stun, packs | Kill fast at range, or leave |
| Inertia, impact hounds | 35 | Inertia breath slows (−10), force breath stuns | FA blocks the slow (inertia applies TMD_SLOW, which FREE_ACT prevents); pStun vs impact |
| Lorgan | 36 | +10, 1800 HP, TELE_TO, summons | Strong melee or skip |
| Time vortex | 38 | +20, time breath (stat/exp drain) | Kill at range |
| Dunadan of Angmar, demonologists, death knights | 36–38 | Nether bolts ~100, summons, hold | FA, rNether later |
| Castamir | 38 | 50% spells: bolts up to ~59, heals 228, traps | rBase, burst |

### DL40–50

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Master lich | 41 | 1584 HP, brain smash, hold, conf, summon undead | FA, rConf, rBlind, high save |
| Kavlax | 41 | 9 breath types, 433 for base elements | All base resists + rSound/rShards/rChaos/rNexus |
| Hellhound packs | 42 | +10, fire breath 117 each, fire bites | rFire (double resist), leave otherwise |
| Multi-hued hounds | 43 | 5 base breaths 73 each, packs | rBase + rPois |
| Dreads | 44 | Packs, invisible, pass walls, blind/conf/hold, nether bolt 111 | SI/ESP, FA, rConf, rBlind |
| Greater basilisk | 45 | Two paralyzing gazes, poison breath 336 | FA, rPois |
| Vampire lord | 46 | Brain smash, EXP_80, drain mana | Hold Life, FA |
| Death mold | 47 | +30, 3×7d7 disenchant spores + EXP_80 (only adjacent) | Never stand next to it |
| Lesser Balrog, Balrog of Moria | 49/50 | Fire breath 586/1000 | rFire, ideally double; slay demon |

### DL50–60

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Ancient dragons | 50–53 | ~500 base-element breath (multi-hued: all five) | rBase, damage them first |
| Death drake | 50 | Invisible, pass walls, nether 325, EXP_80 | SI, rNether, Hold Life |
| Time hounds | 51 | +20 packs, time breath 110 | Leave, or kill with one area blast |
| Nether / plasma hounds | 51 | Packs, nether 55 / plasma 55 + stun | rNether; pStun |
| Drolem | 54 | Poison breath **733** | **rPois** (still ~244) |
| Dreadmaster | 54 | TELE_LEVEL, hold, nether bolts, summon undead | rNexus, FA |
| Dracolich, dracolisk | 55 | Cold 1026 / fire 1026 and nexus 400; dracolisk paralyzing gaze | rBase, rNexus, FA |
| Shelob | 55 | Paralyzing bite, poison 800, dark 500 | FA, rPois, rDark |

### DL60–75

| Monster | L | Why it kills | Counter |
|---|---|---|---|
| Dreadlords | 62 | Invisible, pass walls, nether ball 303, brain smash | SI, rNether, FA |
| Chaos hounds | 64 | Packs, chaos 155 each (confusion, hallucination, exp drain) | rChaos |
| The Mouth of Sauron | 65 | +20, invisible, mana ball 380 | SI, big HP, kill fast |
| Saruman | 60 | TELE_AWAY, FORGET, balls ~110–125, summons | Saving throw; leave if out-classed |
| Hounds of Tindalos | 69 | +20, invisible, pass walls; time 150 | SI, leave |
| Gelugon | 69 | +20, cold 1026, shards 500, paralyzing hit | rCold, rShards, FA |
| Great wyrms (from 70) | 70–82 | 1400–1600 breaths, or 500–600 of high elements | Full resists, damage first, or avoid |

### DL75 and deeper

| Monster | Why it kills | Counter |
|---|---|---|
| Aether hounds (L74, +20) | 18 breath types, up to 410 each, packs | Everything resisted; better to leave |
| The Nazgûl (L77–85) | WOUND ~150–170, nether ~160 bolts / ~390 balls, BLACK_BREATH, summon Nazgûl | Hold Life, rNether, high save, kill one at a time |
| Feagwath, Vecna (L77, L92) | Mana balls 440–515, nether balls ~360–420, brain smash | Big HP, rNether, FA |
| Greater Balrogs (L85), Lungorthin, Gothmog | Fire 1500–1600, plasma | Double rFire, slay demon |
| Ancalagon (L85) | Four 1600 breaths, comes with five dragon uniques | Usually avoid |
| Qlzqqlzuup (L79) | Summons uniques every turn | Kill or leave instantly |
| Pazuzu (L82, +30, invisible) | 4×12d12 lightning hits | rElec, SI |
| Storm of Unmagic (L95, +30) | Exp/charges/disenchant/all-stat drains | Kill at range or leave |

---

## 8.15 Uniques: a tactical guide

Every unique is listed in [A2](appendices/A2-uniques.md). This section covers the
notable ones. All numbers FACT from `monster.txt` (averages computed); tactics ADVICE.
Uniques never breed, are never trampled, and keep their full HP between meetings (a
unique you hurt and fled from will be at full HP next time, since it regenerates and
the level is regenerated).

### Town (DL0)

- **Farmer Maggot** (350 HP): harmless (two "moans"), drops mushrooms. Free.
- **Grip and Fang, Farmer Maggot's Dogs** (DL2 but can appear early; 25 HP, +10, erratic,
  1d4 bite): your first real test. At +10 they bite twice per turn. A CL1 mage dies to
  them easily. Throw flasks of oil (22.5 average each) or fight at full HP; Magic Missile
  works.

### DL1–10

- **Sméagol** (L3, 400 HP, +20, invisible, erratic, `HURT_LIGHT`): touches to steal gold,
  then teleports. Almost impossible to kill early (invisible, 400 HP, fast). Ignore him,
  carry little gold, or hit him with light. His drop is good (`DROP_GREAT`).
- **Bullroarer the Hobbit** (L5, 60 HP, +10, 2×1d8) with 1d7 scruffy hobbit thieves:
  kill with flasks or melee at full HP. Good drop.
- **Mughash the Kobold Lord** (L7, 150 HP, 3×1d10) with small kobolds, kobolds, shamans,
  archers and large kobold bodyguards: fight from a corridor; kill the shamans and archers
  as they come.
- **Wormtongue, Agent of Saruman** (L8, 250 HP, speed +0, AC 45): weak melee (2×1d5,
  steal gold) but casts 1 turn in 5: frost bolt ~29, poison ball ~17, SLOW, create
  traps, heal 48. At range he wears you down; adjacent he is easy. Close the distance,
  keep HP above 40, and finish him before he heals. `DROP_GREAT`: often an excellent
  early item.
- **Lagduf, the Snaga** (L8, 190 HP, ~21/turn) with wolves and snagas; **Grishnákh, the
  Hill Orc** (L10, 230 HP, ~24/turn) with wolves and cave orcs: fight in corridors; the
  wolves are faster than the orcs, so they arrive first: kill them, then the orcs.
- **Brodda, the Easterling** (L9, 210 HP, 4×1d12 ≈ 26/turn): pure melee. Flasks, then
  melee.
- **Orfax, Son of Boldor** (L10, 120 HP, +10): blink, TELE_TO, confusion, slow, heal 60,
  summons, with yeeks. Low HP: burst him.

### DL10–20

- **Golfimbul, the Hill Orc Chief** (L12, 240 HP, AC 90, resists base elements and
  poison) comes with Bullroarer, Grishnákh, wargs and orcs. Three uniques together: lure
  them into a corridor and fight one at a time, or leave.
- **Boldor, King of the Yeeks** (L13, 180 HP, +10): blind, slow, heal, teleport, summons
  yeeks and monsters, with Orfax. Easy if you can catch him.
- **Ufthak of Cirith Ungol** (L14, 320 HP, AC 75, 4×3d4 ≈ 30/turn) with orc archers.
- **Ulfast, Son of Ulfang** (L16, 340 HP, 4×3d5 ≈ 36/turn) with warriors.
- **Nár, the Dwarf** (L17, 450 HP, AC 84, rFire/rCold/rPois): blind, confuse, mind
  blast 36, wound 30, heal 102, 4×3d5 melee. The first unique that demands cures: carry
  CLW/CSW. Burst damage beats his heals.
- **Shagrat** and **Gorbag, the Orc Captains** (L19, 400 HP, ~45/turn each) with uruks,
  archers and trackers. Corridor and slay orc.
- **Old Man Willow** (L19, 1000 HP, AC 100, never moves, regenerates): TELE_TO and HOLD
  10% of turns, two paralyzing touches 1d14 and a 2d12 crush. Without Free Action, stay
  out of his line of sight entirely. With FA and fire (trees are `HURT_FIRE`: a fire brand
  is ×6), he is a big experience bag.

### DL20–30

- **Bolg, Son of Azog** (L20, 500 HP, +10, 4×3d6 ≈ 42/turn) with uruk bodyguards,
  wargs, archers. A frequent killer ([worked example](04-combat.md#415-is-this-fight-winnable-a-damage-race-procedure)):
  needs speed or a big slay-orc weapon.
- **Lugdush, the Uruk** (L21, 640 HP, AC 108, ~42) and **Uglúk, the Uruk** (L22, 720 HP,
  AC 114, resists fire/cold/poison, ~45) often together with uruks.
- **Azog, Enemy of the Dwarves** (L23, 900 HP, +10, 3×5d5 ≈ 45) with Bolg and an uruk
  army: the hardest orc fight. Do not fight both at once without speed.
- **Ibun** and **Khîm, Sons of Mîm** (L24, 820 HP, rFire/rCold): disenchanting hits, fire
  bolt 48, slow, heal 144. **Mîm, Betrayer of Turin** (L27, 1100 HP, +10, immune to all
  base elements and poison): acid ball ~56, acid bolt, scare, heal 162, disenchant. The
  family together is a long fight; disenchantment ruins your weapon's pluses: use a
  backup weapon or rDisen.
- **Sangahyando** and **Angamaitë of Umbar** (L24, 800 HP each, 4×4d6 ≈ 56/turn each,
  resist fire/elec): always together; confuse and slow. rConf and FA; separate them.
- **Ulwarth, Son of Ulfang** (L26, 850 HP, ~42) with Easterling champions and Ulfast.
- **Uldor the Accursed** (L28, 1000 HP, ~56, summons kin).
- **Draebor, the Imp** (L28, 520 HP, +10, invisible): teleports you away, to him, or off
  the level, blinds and confuses, summons imps. More annoying than deadly. SI and rNexus.
- **Beorn, the Shape-Changer** (L28, 1400 HP): as a man weak melee, but he often
  (spell frequency 1 in 2) shapechanges into **the Mountain Bear** (+10, ~50/turn melee), with a bear escort.
- **Bert, Bill and Tom, the Stone Trolls** (L30, 1100 HP each, +10, ~58/turn each,
  boulders, rCold, `HURT_LIGHT`): always together. 3300 HP of fast trolls. Fight in a
  corridor with slay troll and light attacks, or come back later.

### DL30–40

- **Ulfang the Black** (L34, 1000 HP, +10, 4×5d5) with his sons, Easterling bodyguards,
  mûmakil.
- **Lokkak, the Ogre Chieftain** (L35, 1500 HP, +10, 3×6d6) with ogres, ogre mages and
  shamans (who HOLD and summon). Slay giant.
- **Lorgan, Chief of the Easterlings** (L36, 1800 HP, +10, ~69/turn, immune to base +
  poison): TELE_TO and summon monsters 1 turn in 4. A test of raw melee and HP.
- **Castamir the Usurper** (L38, 880 HP, +10): casts half his turns: fire/cold/elec/ice
  bolts ~30–59, heal 228, traps. Low HP for his depth: burst him.

### DL40–50

- **Gorlim, Betrayer of Barahir** (L41, 1600 HP, +10, AC 144): 50% spells: mana bolt
  ~101, water bolt ~96, wound 78; disenchanting hits.
- **Kavlax the Many-Headed** (L41, 1300 HP, +10): breathes 1 turn in 4: acid, cold,
  elec, fire (433), sound, shards, chaos, nexus (216), gravity (200). Damage him first;
  his breaths shrink with his HP.
- **The Queen Ant** (L42, 1500 HP): summons ants half her turns. Kill her fast.
- **Rogrog the Black Troll** (L42, 2000 HP, +10, regenerates, ~88/turn, acid spit,
  boulders) with an olog army.
- **Vargo** (fire), **Waldern** (water), **Quaker** (earth, pass walls, shattering hit),
  **Ariel** (+20 air, confusing hits) (L43, 2400–2800 HP): elemental balls and bolts
  ~40–104. No ESP (empty minds).
- **Eöl, the Dark Elf** (L49, 2400 HP, +20, AC 150): 50% spells: nether ball ~251, mana
  bolt ~111, fire/acid/cold/elec balls, wound 96, heal 294, TELE_TO, blind, confuse,
  summons demons/dragons/undead/monsters. A dangerous caster: rNether, rBlind, rConf,
  FA, and burst.
- **The Balrog of Moria** (L50, 3000 HP, +20): fire breath **1000** at full HP,
  fire hits 2×6d12, DRAIN_CHARGES, confusion, scare, summons demons and undead, with orc
  and troll servants. Needs rFire (better: double resist) and slay demon.

### DL50–60

- **Harowen the Black Hand** (L51, 2500 HP, **+30**): a master thief: steals gold and
  items, blinding and poison hits, TELE_TO, TELE_SELF_TO, traps, HOLD. Keep your
  valuables at home; kill or avoid.
- **Itangast the Fire Drake** (L52, 2200 HP, fire 733) and **Scatha the Worm** (L53, 2500
  HP, cold 833): wound 102, confusion.
- **The Phoenix** (L54, 3600 HP): fire 1200, light 500, plasma 150; fire bolts and balls.
  Immune to fire/acid/elec/poison/plasma: use cold or plain damage. Not evil.
- **Ar-Pharazôn the Golden** (L55, 4000 HP, +20, AC 54): 4×8d8 ≈ 144/turn melee, heal 330,
  TELE_AWAY, summons. Low AC; hit hard.
- **Shelob, Spider of Darkness** (L55, 3500 HP): paralyzing bite 5d10, poison breath
  800, dark 500, heals 330, webs, with uruks and Mirkwood spiders. FA, rPois, rDark.
- **Baphomet the Minotaur Lord** (L58, 4700 HP, +20): **melee ≈ 278/turn**, force breath
  200, mana bolt ~123, plasma bolt ~100, slow. Do not let him reach you.
- **Fundin Bluecloak** (L59, 5000 HP, AC 234): brain smash, wound ~114, heal 354,
  summons. Very hard to hit.

### DL60–75

- **Saruman of Many Colours** (L60, 5000 HP, +10): half his turns: TELE_AWAY, FORGET,
  teleport, traps, heal 360, haste, acid/cold/fire/water balls ~55–125, ice bolt ~81,
  summons animals, demons, high dragons, monsters, undead; Wormtongue as servant.
  Requires FA, high saving throw, and patience.
- **Gilim, the Giant of Eruman** (L60, 4000 HP): STORM ~250, ice bolt, boulders, melee
  ~199.
- **Smaug the Golden** (L62, 4200 HP, AC 195): fire breath **1400**, melee ~183, wound 120.
- **The Mouth of Sauron** (L65, 7000 HP, +20, invisible): half his turns: mana ball
  ~380, dark/nether balls ~315, fire/water balls, plasma bolt, HOLD, TELE_TO, traps,
  summons high demons/undead. SI essential.
- **Tevildo, Prince of Cats** (L66, 4800 HP, +20, invisible, AC 240): paralyzing bite,
  confusion and blindness claws, TELE_TO, summons cats. FA, SI.
- **Thuringwethil, the Vampire Messenger** (L67, 5000 HP, +20): EXP_80 bites, brain
  smash, nether ball ~323, wound, drain mana, summons high undead.
- **Tselakus, the Dreadlord** (L68, 6500 HP, +20, invisible, pass walls): dark/nether
  balls ~327, HOLD, confusion, blindness, summons undead.
- **Nan, the Giant** (L69, 3344 HP): melee ≈ 312/turn (4×12d12, two confusing), plasma
  bolt ~127, fire ball, boulders.
- **Ossë, Herald of Ulmo** (L72, 6000 HP, +20): STORM ~286, disenchantment breath 500,
  mana bolt, TELE_TO, summons Ainur. Not evil.
- **Omarax, the Eye Tyrant** (L73, 6500 HP, +20): dark/nether balls ~347, brain smash,
  FORGET, drain mana, mana bolt, summons kin.

### DL75–98

- **Glaurung, Father of the Dragons** (L75, 7500 HP, +20): fire 1600, poison 800, brain
  smash, FORGET, summons high dragons.
- **Ungoliant, the Unlight** (L75, 13000 HP, +20): dark ball ~355, dark/poison breath,
  heal 450, paralyzing bite.
- **Makar, the Warrior** (L76, 6000 HP, +10): no spells but **melee ≈ 364/turn** (2
  shattering 13d13 and 2 confusing 13d13), tunnels. Never melee him; shoot or leave.
- **Feagwath, the Undead Sorcerer** (L77, 5000 HP, +20): mana ball ~440, nether ball
  ~363, brain smash, wound 150, traps, summons, with dreads and liches.
- **The Nazgûl** (Uvatha L77, Adunaphel L78, Akhorahil L79, Ren L80 (invisible), Ji
  Indur L81 (invisible), Dwar L82, Hoarmurath L83, Khamûl L84, the Witch-King L85): 4200–
  5000 HP, +10 (Witch-King +20), a **BLACK_BREATH** hit, EXP_40/80 touches, WOUND ~150–170,
  nether bolts ~160 or nether balls ~380–395, mana bolts ~155, HOLD, and S_WRAITH (they
  summon each other). Khamûl has TELE_LEVEL; the Witch-King TELE_AWAY, brain smash and
  every high summon. Hold Life, rNether, FA, high saving throw.
- **Qlzqqlzuup, the Emperor Quylthulg** (L79, 5000 HP, +20, invisible, never moves): only
  summons, on every turn its spell does not fail, including uniques. Leave on sight unless you can kill 5000 HP
  instantly.
- **Wiruin, the Maelstrom** (L80, 3500 HP, +30, never moves): cold breath 1166, sound 500.
- **Maeglin, the Traitor of Gondolin** (L81, 6000 HP, +20, tunnels): 4×8d8 melee and
  summons (high demons, dragons, undead, uniques).
- **Pazuzu, Lord of Air** (L82, 5500 HP, +30, invisible): 4×12d12 lightning hits (~312),
  mana bolt ~153, lightning ball/bolt, summons high demons, with giant rocs.
- **Draugluin, Sire of All Werewolves** (L83, 7000 HP, +20): summons hounds and
  monsters, with werewolves and wolf chieftains.
- **Cantoras, the Skeletal Lord** (L84, 7500 HP, **+30**): nether ball ~495, wound 216,
  water ball, ice and mana bolts, brain smash, summons high undead.
- **Ancalagon the Black** (L85, 10000 HP, +20): acid, cold, elec, fire 1600, poison 800;
  appears **with Kavlax, Scatha, Itangast, Smaug and Glaurung** as friends
  (`monster.txt` `friends:`). One of the most dangerous encounters in the game.
- **The Tarrasque** (L86, 8500 HP, +20): breathes half its turns: fire, cold 1600,
  disenchantment 500.
- **Meássë, the Bloody** (L87, 7000 HP): melee ≈ 312 (4×confusing 12d12), sound breath 500,
  heal 522, with Maiar, titans and Makar.
- **Lungorthin, the Balrog of White Fire** (L88, 7000 HP, +20): fire 1600, plasma 150,
  with lesser/greater Balrogs and the Balrog of Moria.
- **Huan, Wolfhound of the Valar** (L90, 8000 HP, +20, not evil): cold 1600, light,
  shards, sound 500. **Carcharoth, the Jaws of Thirst** (L90, 8000 HP, +20): fire 1600,
  nether 600, poison 800, dark 500, summons hounds.
- **Vecna, the Emperor Lich** (L92, 6000 HP, +20): mana ball ~515, nether ball ~423,
  fire ball, brain smash, wound 180, summons, with a large undead court.
- **Gothmog, the High Captain of Balrogs** (L95, 8000 HP, +20): fire 1600, plasma 150,
  blind/conf/scare, summons; brings the Balrog of Moria, Lungorthin, greater and lesser
  Balrogs and greater demons. Probably the hardest non-quest fight.

---

## 8.16 The final quests: Sauron and Morgoth

### The quest rules

FACT (`lib/gamedata/quest.txt`, `src/player-quest.c`, `src/player-util.c:dungeon_get_next_level()`,
`src/gen-cave.c`, `src/mon-make.c`):

- Two quests: **Sauron on DL99** and **Morgoth on DL100**. Both are `QUESTOR` and
  `FORCE_DEPTH` (never generated shallower than their level).
- **You cannot go below a quest level until its monster is dead**: stairs, deep descent
  and level teleport stop at it (`dungeon_get_next_level()` returns the first quest level
  in the way), quest levels are generated with no down staircases, and TELE_LEVEL
  cannot send you down from one (only up).
- Quest levels always use the classic profile at full size.
- Killing the quest monster creates a magical down staircase where it died. Killing
  Morgoth (both quests done) wins the game.
- Sauron and Morgoth are placed by normal monster generation (rarity 1, `FORCE_DEPTH`),
  not forced onto the level; if he is not on your DL99 level, take a staircase and try
  another.
- Morgoth drops every quest artifact: **'Grond'** (Mighty Hammer, 9d9, (+5,+25),
  `IMPACT`, SI, ESP, `AGGRAVATE`, slays) and the **Massive Iron Crown of Morgoth**
  (+50 to all stats, many resists, `STICKY`) (`artifact.txt`, `mon-make.c`).

### Sauron, the Sorcerer (DL99)

FACT (`monster.txt`):

| | |
|---|---|
| HP / speed / AC | 8000 / **+20** / 192 |
| Flags | Unique, evil, smart, regenerates (160 HP per 10 turns), never afraid, `MOVE_BODY`; immune to fire, cold, poison, lightning; no conf/sleep/hold (`ainu` base); spell power **130** |
| Melee | 2×HIT DISENCHANT 10d12, 2×HIT DRAIN_CHARGES 8d12 (≈ 234 per turn before AC) |
| Spells | **1 in 2** turns: BRAIN_SMASH 96, WOUND ~258, TELE_LEVEL, teleport, dark ball ~575, fire ball ~238, nether ball ~575, ice bolt ~151, mana bolt ~213, plasma bolt ~172, **SHAPECHANGE**, summon high demons, high dragons, high undead, uniques, wraiths |
| Escort | **Khamûl and the Witch-King** (`friends:100:1d1` each, servants) |
| Shapes | **Wolf-Sauron** (+30, breathes fire/nether, dark ball, confusing and paralyzing claws, summons hounds/dragons/demons/undead); **Serpent-Sauron** (+20, breathes poison/chaos/acid, brain smash, FORGET, wound); **Vampire-Sauron** (+20, EXP_80 bite, storm, nether beam/ball, dark/poison/acid balls, brain smash, FORGET). Each lasts 5–15 turns and keeps his current HP; breaths use that HP (1600 cap for fire/acid, 800 poison, 600 nether/chaos) |

ADVICE (what is needed):

- **Resists:** all base, poison, nether, dark, chaos, disenchantment, nexus (his
  TELE_LEVEL would throw you up to DL98, away from the fight; rNexus blocks it), confusion protection,
  blindness protection, Free Action, See Invisible, Hold Life (Vampire-Sauron, the
  Nazgûl escort).
- **HP and speed:** a dark or nether ball of ~575 is roughly 1/3–1/2 of that resisted
  (see [Chapter 5](05-elements-and-status.md) for the resist divisors); plan for repeated
  300+ damage turns, so 600+ HP and at least +20 speed (+30 to match Wolf-Sauron).
- **Saving throw:** BRAIN_SMASH and WOUND allow saves; a near-100% save removes ~250
  damage spells from his list.
- **Damage:** only slay evil (×2) and acid brands work (he is not undead or demon; immune
  to the other four base elements). Plan 8000 HP of damage at ~300–500 per turn while he
  regenerates and you heal; that is 20–40 of your turns.
- **Devices:** his DRAIN_CHARGES hits empty wands and staves; rely on rods, spells and
  melee.
- **Escort first:** Khamûl and the Witch-King start next to him. Separate them if
  possible; fighting three level-84+ uniques at once is how most attempts fail.
- **Summons:** S_UNIQUE can bring anyone. Fight in a corridor so he has no space
  (summons need empty floor within 2 grids of him).

### Morgoth, Lord of Darkness (DL100)

FACT (`monster.txt`):

| | |
|---|---|
| HP / speed / AC | **20000** / **+30** / 180 |
| Flags | Unique, evil, smart, regenerates (400 HP per 10 turns), never afraid; immune to acid, cold, elec, fire, poison; no conf/sleep/hold; `SMASH_WALL` (walls are no protection), `MOVE_BODY`; spell power 100 |
| Melee | 2×HIT SHATTER 20d10 (earthquakes around him), HIT LOSE_ALL 10d12 (drains all stats), TOUCH DRAIN_CHARGES (≈ 285 per turn before AC) |
| Spells | **1 in 3** turns: BRAIN_SMASH 96, **mana ball ~555** (mana is not resistible), nether ball ~455, mana bolt ~175, summon high demons, high dragons, high undead, monsters, uniques |

ADVICE:

- **Speed is the core problem:** at +30 he has 38 energy per game turn; you need +30 or
  better to avoid giving him two actions for each of yours.
- **HP:** a mana ball averaging 555 cannot be resisted. Survive two in a row: 1100+ HP
  effective, or keep HP high at all times with *Healing*/*Life* potions.
- **Never melee next to your escape stairs or stash:** his SHATTER hits cause earthquakes
  around him (radius damage/12; knockback above 100 damage).
- Sustain all stats (LOSE_ALL), rNether, FA, SI, pConf, rBlind, high saving throw
  (BRAIN_SMASH).
- **Damage:** only slay evil works (×2); no brand, no slay dragon/demon/undead. 20000 HP
  at 400 regeneration per 10 turns means you need sustained damage well above 40 per
  turn just to make progress; realistically 500+ per turn for a fight of ~40 turns.
- **Glyphs of warding** slow him (18% break chance per try), and he must break them to
  reach you, but he still casts across them.
- Summons can include any unique still alive: kill the dangerous uniques before
  attempting Morgoth.

---

## Things to try (untested)

1. **Infravision vs warm-blooded invisibles.** Since infravision is checked before
   invisibility ([8.11](#811-seeing-monsters-invisibility-esp-infravision-detection-light)),
   a race with high infravision (or extra infravision from items) should see quylthulgs,
   quasits, Draebor and the other warm-blooded invisible monsters inside that range
   without See Invisible. *Test:* in wizard mode, spawn each `INVISIBLE` race lacking
   `COLD_BLOOD` and check visibility at distances inside and outside infravision range.
2. **Starve summoners of floor.** Since summons need empty floor within 2 grids of the
   summoner, fight summoners in 1-wide corridors with you adjacent. *Test:* count summon
   attempts that produce "nothing comes" versus summons in rooms.
3. **Exploit the 24% spell failure.** A borg danger model should multiply non-innate
   spell damage by 0.76. *Test:* log predicted and actual caster damage over 1000 fights.
4. **Make smart casters waste turns.** Wear a resist the caster has a spell for, let it
   learn, then remove it before a fight? Monsters forget 1 in 20 turns, so this decays
   fast. *Test:* measure spell choice frequencies after resist learning.
5. **Lock out hound packs.** Low-HP hounds and wolves cannot beat a locked door
   (`randint0(HP/10)` must exceed the lock power). Lock (`D`) the doors between you and a
   hound room and explore elsewhere. *Test:* compare hound damage taken per level with and
   without door-locking behaviour in a borg; check how often the lock fails to set.
6. **Breeder corridor farming.** Breeding needs fewer than 3 neighbours: a colony of weak
   worms confined to a corridor grows linearly. Farm white worm masses for early levels.
   *Test:* experience per 1000 turns vs normal exploration at CL5–10.
7. **Fear-freezing.** Scaring a monster that has no escape route converts its fear into
   hold. Try Scare Monster effects on monsters in dead ends, then burst them (the first hit
   wakes them). *Test:* turns monster spent held vs. scared in open rooms.
8. **Weaken breathers before they breathe.** Open with your biggest single attack on a
   dragon at long range (breath falls off after 7/11 grids) and only close in once its HP
   is halved. *Test:* compare breath damage taken per dragon kill.
9. **Rouse control.** Group members wake only slowly from an awake comrade (1 in 20×dist
   per turn). Kill pack members one at a time while the others sleep. *Test:* fraction of
   orc gangs where the whole gang woke, stealth-matched.
10. **Sauron escort split.** Teleport-to (or lure with noise) Khamûl and the Witch-King
    away from Sauron before engaging him. *Test:* wizard-mode replays of DL99 with and
    without the split.
11. **Glyph fortress for Morgoth.** Stand on a glyph-ringed spot so he must break a glyph
    each time he approaches (18%/try, up to 5 tries per turn) while you shoot. *Test:*
    in wizard mode, measure his melee turns per 100 game turns with and without glyphs.

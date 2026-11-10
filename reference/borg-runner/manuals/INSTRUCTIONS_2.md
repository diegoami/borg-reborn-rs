# INSTRUCTIONS 2: Angband 4.2.6, depth 5 to 20, for a character that survived volume 1

**No run has reached depth 8. Volume 1 carries OBSERVED lines because earlier runs tested it; this volume has none and cannot have any yet.** Every line below is extraction from the game's data and source, or a strategy-manual rule: a hypothesis, not knowledge. A run that gets here should expect to correct it, and what a run has actually measured - in volume 1, or here - wins over this page. Character level 8 to 25 is the manual's rough figure (c12 12.6, 12.7: typical CL 8-22 and 18-32), not a game rule.

Sources, so any line can be checked. Volume 1's shorthand holds, plus `monster_spell`/`monster_base`/`projection`/`curse`/`ego_item`/`constants`/`player_timed`/`dungeon_profile`/`object_property` = `vendor/angband/lib/gamedata/<name>.txt` and `c8`/`c12`/`c13` for strategy chapters. Labels as in volume 1: **FACT** (game data or source), **POLICY** (a chapter 15 rule: a starting policy to be measured, not truths), **UNTESTED** (untried).

## 1. What changes at depth 5 (`monster`, `monster_spell`)
FACT, counted from `monster` (the `<player>` entry is not a monster): kinds per depth band, and how many breathe (a `BR_` spell) or summon (an `S_` spell).

| Depth | 0-4 | 5-9 | 10-14 | 15-19 | 20-24 |
|---|---|---|---|---|---|
| kinds | 80 | 60 | 51 | 45 | 45 |
| breathe | 0 | 11 | 2 | 5 | 9 |
| summon | 0 | 0 | 6 | 0 | 3 |

- **Breath** (from DL5): damage is a fraction of the breather's *current* hit points, armour does not reduce it (`hit:100`, `monster_spell`; acid is the one exception) and there is no saving throw (no `message-save` line on any `BR_` spell). Resist it or leave its line of sight. Section 4.
- **Summoning** (from DL10): the fight you are winning gets more monsters in it. Section 5.
- **HOLD** (from DL13): paralysis, which ends runs the same way. Section 2.

## 2. The kit gates (c15 DEPTH-02, c12 12.5-12.6)
**POLICY**: c15 DEPTH-02 gives a minimum kit per depth, each gate anchored to monsters. Rows in this volume's range, checked against `monster`:

| Depth | Required | Anchor monsters |
|---|---|---|
| 5 | Word of Recall, 3+ cure potions, ranged attack | Bullroarer (DL5) |
| 15 | 100 max hp, Phase x10 | Wormtongue (DL8), Boldor (DL13), druid (DL13) |
| 20 | Free Action | illusionist (DL13), Old Man Willow (DL19), carrion crawler (DL25), ghoul (DL26) |

- **DL5.** **Bullroarer** (DL5, +10, 60 hp, 2 x 1d8: up to 32 per turn) is the anchor; the chapter's "+20" for him is +10 in `monster`. Cure Light Wounds heals 20 (`object`), under his 32 (c15 HEAL-04): carry 3+ and leave rather than trade.
- **DL15, 100 hp.** Worst spells at the native depth: druid BO_FIRE up to 76 (power 13); Wormtongue BO_COLD up to 50 and BA_POIS up to 28; dark dwarven lord BO_FIRE up to 78. `BO_FIRE` is `$B+9d8` with B = power / 3, and power is the monster's depth unless `spell-power` says otherwise (`monster_spell`, `monster`). 76 + the 20-point reserve (c15 `K_HP_RESERVE`, max(20, 10%)) = 96, so 100 is the round number above the druid's bolt, and Phase x10 has no figure in `monster` behind it: UNTESTED as gates. Average maximum hp at CL n is hit die + (n-1) x (hit die + 1)/2 plus CON (`src/player-birth.c roll_hp`: each level rolls 1 to the hit die): hit die 10 (Human Mage) has 87 at CL15 and needs CL18 for 100; hit die 21 (Half-Troll Warrior) has 175 at CL15.
- **DL20, Free Action.** HOLD paralyses for 3+1d4 turns (`monster_spell`); Free Action blocks it (`fail:1:FREE_ACT`, `player_timed`), and so does your saving throw (`message-save`). **The gate sits seven levels after its own first anchor:** the **druid** (DL13, 78 hp, 1 in 3) and the **illusionist** (DL13, +10, 54 hp, 1 in 3) already cast it, and c15 DEPTH-01 looks at monsters native to d-2 to d+5, so DL13 is inside that window from DL8. Which depth is right is UNTESTED. **No store sells it** (`store`): it is a ring (found from DL20, 1500, `object`) or an ego on gloves (found from DL1 to 60) or boots (1 to 40) (`ego_item`). Without it, kill HOLD casters at range or avoid them. Others paralyse: **homunculus** (DL15, PARALYZE), **evil eye** (DL18, never moves, HOLD), **Old Man Willow** (DL19, never moves, PARALYZE).
- The next gate, DL25, wants See Invisible or ESP and Teleport x5 (c15 DEPTH-02). Invisible things come first: **moaning spirit** (DL12, invisible, passes walls), **quasit** (DL16, invisible).

## 3. Escapes that still work (c13 13.1, c15 ESC)
FACT (`src/effect-handler-general.c TELEPORT`, `object`):

| Escape | `cost:` | `level:` | found from | where | distance |
|---|---|---|---|---|---|
| Scroll of Phase Door | 15 | 1 | 1 | Alchemist, always | 10 |
| Scroll of Teleportation | 40 | 10 | 10 | no store | 60% of the span |
| Scroll of Teleport Level | 800 | 20 | 20 | no store | up or down one level |
| Staff of Teleportation | 2000 | 20 | 20 | Magic shop, sometimes | 60% of the span |

- **Phase Door stops being enough**, three ways. (1) It lands you 9 to 11 squares away; a +10 monster has energy 20 to your 10 (`src/game-world.c extract_energy`) and so closes it in 5 of your turns, a +20 monster (energy 30) in 4. c13 13.3 says "in 1 turn": the arithmetic does not. (2) Breath, bolts and arrows need only line of sight, which a short hop in a room does not break. (3) TELE_TO casters pull you back next to them: Orfax and tengu (DL10), quasit (DL16), imp (DL17), blink dog and evil eye (DL18), Old Man Willow (DL19).
- **Teleportation** distance is 60% of the longer of your farthest vertical and farthest horizontal edge distance. From the middle of a full 198 x 66 level (`constants`) that is 59, then a random quarter either way: 46 to 72 squares; near an edge, more. A scroll cannot be read blind, confused or in the dark; the staff (5+d4 charges) is a device and needs none of that (c13 13.2).
- **Teleport Level** goes up or down at random, even odds; it cannot go up from the town (`src/effect-handler-general.c TELEPORT_LEVEL`). TELE_LEVEL is cast on you by: quasit (DL16), imp (DL17); a nexus resist saves you.
- **Never works** with the anti-teleportation curse (boots, cloak, armour, amulet or ring, `curse`): both effects print "Teleportation forbidden!" (`src/effect-handler-general.c`) and do nothing, Phase Door included; the game learns the curse only when it takes effect (`id-type:on effect`, `object_property`), so an unknown ring can show it just when you need the staff. On a no-teleport square only a teleport of 10 or less works; the gauntlet level (found from DL20, `dungeon_profile`) has them.
- **Deep Descent is not an escape** (c15 ESC-04): it drops you to 5 levels below your deepest: deepest DL15, you land on DL20.
- Class spells: town books hold Mage Teleport Self and Necromancer Shadow Shift; dungeon-only books hold Rogue Teleport Self and Priest Portal. Druid, Ranger, Paladin and Blackguard have none (`class`).

## 4. Breath attacks (`src/mon-spell.c breath_dam`, `projection`)
FACT: damage = the breather's current hit points / divisor, capped. `hit-points` in `monster` is the average; damaging the breather weakens its breath. Resistance divides what is left.

| Element | divisor | cap | besides damage |
|---|---|---|---|
| acid, lightning, fire, cold | 3 | 1600 | destroys pack items; acid is halved if the worn piece it picks has AC |
| poison | 3 | 800 | poisons |
| light, dark | 6 | 500 | blinds |
| sound | 6 | 500 | stuns |
| shards | 6 | 500 | cuts |

- **Resist** (`src/project-player.c adjust_dam`): acid, lightning, fire, cold and poison are 1/3 per resist level; light, dark, sound and shards 6/(8+1d4) per level. A temporary resist adds one level and no more than 2 count (`src/player-calcs.c`), so the potion alone is 1/3 and a permanent resist plus the potion is 1/9. Sources (section 7): potions for fire, cold and poison only; the Magic shop's amulet of Resist Acid; Mage Resistance CL20.
- **Distance does not help the way c15 says.** FACT (`src/effect-handler-attack.c BREATH`, `src/project.c`): every breath here is a 30-degree cone (`BREATH:...:0:30`), full strength out to 7 squares and then 8/(distance + 1) of it: 73% at 10 squares, 50% at 15. c8 says this; c15 15.3's `D = 4` and FIGHT-04's `halves at 7` do not. Break line of sight instead: a monster needs a projectable path to cast (`src/mon-attack.c monster_can_cast`), and a corner or a closed door gives none.

Every breather of DL5-20, breath at full hit points. 1 in N is `innate-freq`, per monster turn, doubled at the monster's preferred range (FACT, `monster`):

| Monster | DL | hp | element | breath | 1 in N |
|---|---|---|---|---|---|
| giant white dragon fly | 5 | 14 | cold | 4 | 10 |
| giant green dragon fly | 5 | 14 | poison | 4 | 10 |
| giant black dragon fly | 6 | 14 | acid | 4 | 10 |
| giant gold dragon fly | 6 | 14 | sound | 2 | 10 |
| giant salamander | 8 | 24 | fire | 8 | 9 |
| baby blue dragon | 9 | 88 | lightning | 29 | 12 |
| baby white dragon | 9 | 88 | cold | 29 | 12 |
| baby green dragon | 9 | 88 | poison | 29 | 12 |
| baby black dragon | 9 | 88 | acid | 29 | 12 |
| baby red dragon | 9 | 88 | fire | 29 | 12 |
| baby gold dragon | 9 | 88 | sound | 14 | 12 |
| pseudo-dragon | 10 | 176 | dark, light | 29 | 15 |
| baby multi-hued dragon | 11 | 114 | acid, cold, lightning, fire, poison | 38 | 12 |
| light hound | 15 | 21 | light | 3 | 5 |
| dark hound | 15 | 21 | dark | 3 | 5 |
| fire hound | 18 | 35 | fire | 11 | 10 |
| cold hound | 18 | 35 | cold | 11 | 10 |
| energy hound | 18 | 35 | lightning | 11 | 10 |
| earth hound | 20 | 68 | shards | 11 | 10 |
| air hound | 20 | 68 | poison | 22 | 10 |
| water hound | 20 | 68 | acid | 22 | 10 |
| chimaera | 20 | 160 | fire | 53 | 10 |

- The largest breath in the range is the chimaera's 53; the baby dragons' 29 to 38 are the first that matter. **Hounds come in packs**: each has 2d7 companions of its kind (`friends`, `monster`), all breathing.

## 5. Summoners (`monster`, `monster_spell`, `src/effect-handler-general.c SUMMON`)
FACT, every summoner of DL5-20 (`spell-freq` is 1 in N monster turns):

| Monster | DL | speed | hp | 1 in N | calls |
|---|---|---|---|---|---|
| Orfax, Son of Boldor | 10 | +10 | 120 | 4 | `S_MONSTER` |
| druadan mage | 11 | +0 | 52 | 4 | `S_MONSTER` |
| master yeek | 12 | +0 | 60 | 4 | `S_MONSTER` |
| priest | 12 | +0 | 54 | 3 | `S_KIN` |
| Boldor, King of the Yeeks | 13 | +10 | 180 | 3 | `S_KIN`, `S_MONSTER` |
| druid | 13 | +0 | 78 | 3 | `S_ANIMAL` |
| quylthulg | 20 | +0 | 27 | 4 | `S_MONSTER` |
| ranger | 20 | +0 | 90 | 5 | `S_ANIMAL` |

- **How many, how strong.** The `dice:` on each spell is the number of attempts: `S_MONSTER` 1, `S_KIN` 8, `S_ANIMAL` 12. The loop adds the square of each summoned monster's level and stops when the sum reaches your depth x the summoner's own level, or the attempts run out. Each monster is drawn up to level (depth + summoner level)/2 + 5 and appears within 4 squares of the summoner (`src/mon-summon.c summon_specific`). So a lone `S_MONSTER` brings one. `S_KIN` calls monsters of the summoner's own base, `S_ANIMAL` any animal.
- **Why leaving is the answer** (**POLICY**, c12 12.6: "kill quickly or leave"; c13 13.3 G: break line of sight, then leave). Each cast adds monsters to a fight that was about one, and six of the eight (all but the priest and the druid) have BLINK, so you cannot pin them. Orfax comes with 2d5 blue yeeks and 1d3 master yeeks; Boldor comes with 2d4 master yeeks and 2d6 blue yeeks (`friends`); master yeeks summon too. Against that, summoners have 27 to 180 hp and an oil flask averages about 22 (volume 1, section 5). Which is cheaper is UNTESTED; after the second summons, take the stairs.

## 6. Spells and books (`class`)
FACT, `class`: town books are sold by the Bookseller (kind cost shown; read the price); dungeon books are not.

**Mage**: [First Spells]: Fire Ball CL6 (5 sp). [Attacks and Knowledge] (kind cost 400): Frost Bolt CL13 (5 sp, 40% base failure), Acid Spray CL20 (5 sp, 30% base failure). [Magical Defences] (kind cost 1600): Teleport Self CL7 (6 sp, 35% base failure), Teleport Other CL15 (10 sp), Resistance CL20 (20 sp, 65% base failure).
**Druid**: [Gifts of Nature] (kind cost 400): Resist Poison CL7 (6 sp), Sense Surroundings CL9 (4 sp), Lightning Strike CL12 (6 sp).
**Priest**: [Cleansing Power] (kind cost 100): Orb of Draining CL7 (7 sp), Protection from Evil CL14 (8 sp). [Healing and Sanctuary] (dungeon only, kind cost 10000): Portal CL10, Word of Recall CL25.
**Necromancer**: [Dark Rituals] (kind cost 400): Crush CL10 (6 sp), Shadow Shift CL15 (10 sp). [Fear and Torment] (kind cost 1600): Vampire Strike CL20, Dark Spear CL25.
**Paladin**: [Novice's Handbook] is the only town book; its last spell is Heroism CL12 (5 sp). [Healing and Sanctuary] (dungeon only): Protection from Evil CL15.
**Rogue**: [First Spells]: Object Detection CL10, Detect Stairs CL12 (3 sp, 50% base failure). [Arcane Control] (dungeon only): Teleport Self CL17.
**Ranger**: [Lesser Charms]: Resist Poison CL12 (10 sp), Sense Surroundings CL20 (8 sp).
**Blackguard**: [Into the Shadows]: Leap into Battle CL10, Grim Purpose CL13 (8 sp).

- **Failure** (`src/player-spell.c spell_chance`): the base falls 3 per level above the spell's level, never below 5% (the floor can be higher for a low casting stat) and never above 50%. A Mage's Teleport Self (35%) is down to the 5% floor at CL17 before stat adjustment.
- **Teleport Self** moves you (CL x 9/5)% of the span (`class`): CL7 is 12%, about 11 squares from the middle of a level, no better than Phase Door; CL20 is 36%, about 35; CL25 is 45%, about 44. The staff is the better escape until about CL30 (c13 13.1).
- **First cast pays experience x spell level** (`src/player-spell.c`): Mage Resistance 20 x 20 = 400, Mage Acid Spray 12 x 20 = 240, Mage Teleport Self 5 x 7 = 35, Druid Lightning Strike 8 x 12 = 96. CL15 needs 1400 x race exp / 100, CL20 4400 and CL25 12500 (`src/player.c player_exp`). UNTESTED as a way to level.

## 7. Back to town (c15 TOWN, c12 12.6)
- **Word of Recall** (FACT, `src/effect-handler-general.c RECALL`, `src/game-world.c`): 15-34 turns after reading, in the dungeon it takes you up, in town down to your recall depth. **Read in town before you have ever left it, it takes you to DL1** (`src/player-util.c player_set_recall_depth` makes it at least 1): 125 gold wasted. Read in the dungeon on a level shallower than your deepest, it asks "Set recall depth to current depth?": **y also sets your deepest depth to this one** (and Deep Descent's target with it); n keeps both. c15 TOWN-05 calls y "upward-safe".
- **POLICY** c12 12.6: recall when out of Cure potions or down to under 3 Phase Door; buy the replacement first.
- **Gold** (FACT, `src/obj-make.c make_gold`): a pile averages (16 x depth / 10 + 16) x 5 with selling off: 120 at DL5, 160 at DL10, 200 at DL15, 240 at DL20. What a run will pick up is UNTESTED; a 2000 staff is 2000 / 240 = 8.3 piles at DL20.

FACT, `object` (shops add a markup; read the price). Found from is the `alloc:` minimum:

| Item | `cost:` | `level:` | found from | where |
|---|---|---|---|---|
| Potion of Cure Critical Wounds | 200 | 5 | 12 | Alchemist, sometimes |
| Potion of Resist Heat / Resist Cold | 30 / 30 | 30 / 30 | 30 / 30 | Alchemist, sometimes |
| Potion of Resist Poison | 30 | 20 | 20 | no store |
| Potion of Restore Life Levels | 400 | 40 | 30 | Alchemist, sometimes |
| Amulet of Resist Acid | 300 | 10 | 10 | Magic shop, sometimes |

- CCW heals 60 and cures poison, stun, amnesia, confusion and cuts besides blindness (`object`); Cure Light Wounds stops dropping in the dungeon at DL15. Restore Life Levels is for experience drain (section 8); drained stats come back at your next level (`src/player.c adjust_level`), so they are not an emergency (c15 CURE-03). No store means in no `store` list; the Black market stocks at random.
- **Unknown scrolls and potions** (volume 1 says try them): here Deep Descent (section 3), Summon Undead (found from DL15), Aggravate Monster (found from DL5) and the cost-0 potions of Sleep and Salt Water, which paralyse (`object`), can kill. Read them on a staircase (c15 ITEM-07): UNTESTED.

## 8. The monsters of 5-20 worth knowing by name (`monster`)
FACT, every unique in range. Per turn is the worst melee round (every blow at its maximum) times its turns per yours, as in volume 1:

| Unique | DL | speed | hp | per turn | note |
|---|---|---|---|---|---|
| Bullroarer the Hobbit | 5 | +10 | 60 | 32 | |
| Mughash the Kobold Lord | 7 | +0 | 150 | 30 | |
| Lagduf, the Snaga | 8 | +0 | 190 | 38 | |
| Wormtongue, Agent of Saruman | 8 | +0 | 250 | 10 | casts; steals gold |
| Brodda, the Easterling | 9 | +0 | 210 | 48 | |
| Grishnakh, the Hill Orc | 10 | +0 | 230 | 44 | |
| Orfax, Son of Boldor | 10 | +10 | 120 | 34 | summons; TELE_TO |
| Golfimbul, the Hill Orc Chief | 12 | +0 | 240 | 44 | |
| Boldor, King of the Yeeks | 13 | +10 | 180 | 52 | summons |
| Ufthak of Cirith Ungol | 14 | +0 | 320 | 48 | |
| Ulfast, Son of Ulfang | 16 | +0 | 340 | 60 | |
| Nar, the Dwarf | 17 | +0 | 450 | 60 | casts; MIND_BLAST |
| Shagrat, the Orc Captain | 19 | +0 | 400 | 78 | |
| Gorbag, the Orc Captain | 19 | +0 | 400 | 78 | |
| Old Man Willow | 19 | +0 | 1000 | 52 | never moves; paralyses |
| Bolg, Son of Azog | 20 | +10 | 500 | 144 | |

- **Out-speed you** (volume 1, section 5): **wolf** (DL10, +10, 21 hp, up to 12 per turn, pack of up to 2d10 more), **sabre-tooth tiger** (DL20, +10, 150 hp, up to 80 per turn), **blood falcon** (DL20, +30, 6 hp, up to 64 per its turn: kill it first, from range).
- **Drain experience** (EXP_10): **grape jelly** (DL12, never moves), **nether worm mass** (DL12, breeds), **evil eye** (DL18, never moves). Drain STR (LOSE_STR): red jelly, giant red frog, red naga (DL7), giant red ant (DL9). Drain DEX (LOSE_DEX): moaning spirit (DL12), quasit (DL16). Breed (MULTIPLY): giant fruit fly (DL10), giant flea (DL14): leave the level.

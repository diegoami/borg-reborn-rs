# Appendix A7. Class spells

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/class.txt`, `lib/gamedata/realm.txt`, `lib/gamedata/store.txt`, `lib/gamedata/projection.txt`, `lib/gamedata/player_timed.txt`, `src/list-effects.h` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Every spell of every class, book by book: level, mana, base failure rate, first-cast experience, the effect in words and the in-game description. Which spells to learn and when: [Chapter 3](../03-magic-and-spells.md) and the class playbooks in [Chapter 14](../14-class-playbooks.md).

## Contents

- [Legend](#legend)
- [Realms](#realms)
- [Warrior](#warrior)
- [Mage](#mage)
  - [Mage: First Spells](#mage-first-spells)
  - [Mage: Attacks and Knowledge](#mage-attacks-and-knowledge)
  - [Mage: Magical Defences](#mage-magical-defences)
  - [Mage: Arcane Control](#mage-arcane-control)
  - [Mage: Wizard's Tome of Power](#mage-wizards-tome-of-power)
- [Druid](#druid)
  - [Druid: Lesser Charms](#druid-lesser-charms)
  - [Druid: Gifts of Nature](#druid-gifts-of-nature)
  - [Druid: Creature Dominion](#druid-creature-dominion)
  - [Druid: Nature Craft](#druid-nature-craft)
  - [Druid: Wild Forces](#druid-wild-forces)
- [Priest](#priest)
  - [Priest: Novice's Handbook](#priest-novices-handbook)
  - [Priest: Cleansing Power](#priest-cleansing-power)
  - [Priest: Healing and Sanctuary](#priest-healing-and-sanctuary)
  - [Priest: Battle Blessings](#priest-battle-blessings)
  - [Priest: Wrath of the Valar](#priest-wrath-of-the-valar)
- [Necromancer](#necromancer)
  - [Necromancer: Into the Shadows](#necromancer-into-the-shadows)
  - [Necromancer: Dark Rituals](#necromancer-dark-rituals)
  - [Necromancer: Fear and Torment](#necromancer-fear-and-torment)
  - [Necromancer: Deadly Powers](#necromancer-deadly-powers)
  - [Necromancer: Corruption of Spirit](#necromancer-corruption-of-spirit)
- [Paladin](#paladin)
  - [Paladin: Novice's Handbook](#paladin-novices-handbook)
  - [Paladin: Healing and Sanctuary](#paladin-healing-and-sanctuary)
  - [Paladin: Battle Blessings](#paladin-battle-blessings)
- [Rogue](#rogue)
  - [Rogue: First Spells](#rogue-first-spells)
  - [Rogue: Arcane Control](#rogue-arcane-control)
- [Ranger](#ranger)
  - [Ranger: Lesser Charms](#ranger-lesser-charms)
  - [Ranger: Nature Craft](#ranger-nature-craft)
- [Blackguard](#blackguard)
  - [Blackguard: Into the Shadows](#blackguard-into-the-shadows)
  - [Blackguard: Fear and Torment](#blackguard-fear-and-torment)
  - [Blackguard: Deadly Powers](#blackguard-deadly-powers)

## Legend

- **Lvl**: character level needed to learn. **Mana**: SP cost. **Fail**: base failure rate before level and stat adjustments (see [Chapter 1](../01-core-mechanics.md) and the stat tables in [A8](A8-races-and-classes.md)). **Exp**: the first successful cast gives `Exp * Lvl` experience (`spell_cast()` in `src/player-spell.c`).
- **Effect** is generated from the `effect:` lines: `plev` = character level, dice as in the data (`$` variables substituted with their expressions); `+1 per N plev` = radius/length grows every N levels.
- **Source** of a book: *town* books are always stocked by the Bookseller (`always:` lines for every book type in `store.txt`); *dungeon* books are found at the listed depths (`book-properties:` `cost:commonness:min to max`).

## Realms

| Realm | Stat | Verb | Spell noun | Book noun | Classes |
|---|---|---|---|---|---|
| arcane | INT | cast | spell | magic book | Mage, Rogue |
| divine | WIS | recite | prayer | prayer book | Priest, Paladin |
| nature | WIS | chant | verse | nature book | Druid, Ranger |
| shadow | INT | perform | ritual | shadow book | Necromancer, Blackguard |

## Warrior

No spells (BRAVERY\_30, NO\_MANA, SHIELD\_BASH).

## Mage

Realm **arcane** (spell stat INT). First spell at level 1; total armour weight above 300 (30.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 5 books. Class flags: ZERO\_FAIL, BEAM, CHOOSE\_SPELLS.

### Mage: First Spells

*town book: Bookseller, cost 25, 7 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Magic Missile | 1 | 1 | 22 | 4 | bolt or beam of magic missiles, beam chance -10 for ((plev-1)/5+3)d4 | Fires a magic missile that always hits its target and does unresistable damage. Sometimes a beam is fired instead that hurts each monster in its path. The chance to get a beam goes up with your character level. |
| Light Room | 1 | 2 | 26 | 4 | lights up the surrounding area; sphere of weak light around you (radius 2) for 1 | Lights up all squares within 2 grids of the player, and hurts all light-sensitive monsters in the area of effect. If you are in a room, the entire room will be lit up as well. |
| Find Traps, Doors & Stairs | 1 | 1 | 20 | 2 | detects traps nearby (area 22 x 40); detects doors nearby (area 22 x 40); detects stairs nearby (area 22 x 40) | Detects all traps, doors, and stairs in the immediate area. |
| Phase Door | 2 | 2 | 22 | 4 | teleport self up to 10 grids | Teleports you randomly up to 10 squares away. |
| Electric Arc | 2 | 2 | 34 | 4 | short beam of lightning (length plev) for ((plev-1)/5+3)d6 | Creates an electrical discharge that hurts monsters and destroys vulnerable items in its path. Length of the effect increases with player level. |
| Detect Monsters | 3 | 2 | 24 | 4 | detects visible creatures nearby (area 22 x 40) | Detects all non-invisible monsters in the immediate area, for one turn only. |
| Fire Ball | 6 | 5 | 33 | 5 | ball of fire (radius 2) for (plev\*2) | Shoots a radius-2 fire ball. |

### Mage: Attacks and Knowledge

*town book: Bookseller, cost 400, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Recharging | 5 | 5 | 28 | 6 | recharge a wand/staff (power (plev/8+6)) | Adds charges to a stack of wands or staves. Chance of success and number of charges gained increase with your level and decrease with level of wand or staff and number of charges. A failed attempt to recharge destroys one wand or staff from the stack. |
| Identify Rune | 8 | 7 | 25 | 8 | identifies a single unknown rune on a selected item | Reveals an unknown rune on an object. |
| Treasure Detection | 10 | 3 | 60 | 5 | detects veins nearby (area 22 x 40); senses gold nearby (area 22 x 40); senses objects nearby (area 22 x 40) | Detects gold and senses objects in the immediate area. |
| Frost Bolt | 13 | 5 | 40 | 6 | bolt or beam of cold for ((plev-5)/3+6)d8 | Shoots a bolt of frost that always hits its target. Sometimes a beam is fired instead that hurts each monster in its path. The chance to get a beam goes up with your character level. |
| Reveal Monsters | 15 | 6 | 40 | 8 | detects invisible creatures nearby (area 22 x 40); detects visible creatures nearby (area 22 x 40) | Detects all monsters in the immediate area, for one turn only. |
| Acid Spray | 20 | 5 | 30 | 12 | arc of acid (60-degree, range 10) for (plev/2)d8 | Sprays acid in a 60-degree cone, hitting all monsters up to 10 grids away. |

### Mage: Magical Defences

*town book: Bookseller, cost 1600, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Disable Traps, Destroy Doors | 5 | 5 | 30 | 6 | destroys doors on adjacent grids (radius 1); disables traps on adjacent grids (radius 1) | Disables traps and destroys doors within a 1-square radius of you. |
| Teleport Self | 7 | 6 | 35 | 5 | teleport self $M% of the level's larger dimension | Teleports you randomly within the current level. |
| Teleport Other | 15 | 10 | 30 | 12 | bolt that teleports monsters away (power (plev\*3)) | Produces a bolt that teleports away the first monster in its path. Distance teleported increases with player level. |
| Resistance | 20 | 20 | 65 | 20 | +20+d20 turns of acid resistance; +20+d20 turns of electricity resistance; +20+d20 turns of cold resistance; +20+d20 turns of fire resistance | Grants you resistance to acid, cold, fire and lightning for 20+1d20 turns. |
| Tap Magical Energy | 22 | 2 | 30 | 12 | drains magical energy from a staff or wand | Turns wand or staff energy into mana; tapping stuns the player briefly. |
| Mana Channel | 25 | 10 | 50 | 20 | +5+1d5 turns of mana channelling | Increases spellcasting speed by 33% for 5+1d5 turns. |

### Mage: Arcane Control

*dungeon book: depth 40-100 (x15), cost 30000, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Door Creation | 13 | 9 | 40 | 12 | creates doors on adjacent grids (radius 1) | A door is created on each empty floor space that is directly adjacent to you. These doors are closed, but not locked. |
| Mana Bolt | 25 | 8 | 60 | 30 | bolt or beam of magic missiles for (plev-10)d8 | Fires a magic missile that always hits its target and does unresistable damage. Sometimes a beam is fired instead that hurts each monster in its path. The chance to get a beam goes up with your character level. |
| Teleport Level | 28 | 17 | 65 | 20 | teleport level (up or down) | Teleports you 1 level up or 1 level down (chosen at random). This spell may have no effect when the forced descent option is set. |
| Detection | 30 | 10 | 70 | 30 | detects traps nearby (area 22 x 40); detects doors nearby (area 22 x 40); detects stairs nearby (area 22 x 40); detects veins nearby (area 22 x 40); senses gold nearby (area 22 x 40); senses objects nearby (area 22 x 40); detects invisible creatures nearby (area 22 x 40); detects visible creatures nearby (area 22 x 40) | Detects all traps, doors, stairs, treasure, objects and monsters in the immediate area. |
| Dimension Door | 35 | 30 | 80 | 40 | teleport the target next to the caster | Teleports you to (or near to) a square of your choice. |
| Thrust Away | 40 | 12 | 90 | 40 | short beam of force (length 1+plev/10) for plevd8 | Fires a short range beam of force. |

### Mage: Wizard's Tome of Power

*dungeon book: depth 60-100 (x10), cost 50000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Shock Wave | 20 | 5 | 40 | 16 | ball of sound (radius 2) for (plev\*2) | Shoots a radius-2 sound ball. Apart from doing sound damage, this also has a chance to stun each monster in the area of effect. |
| Explosion | 30 | 10 | 50 | 20 | ball of shards (radius 2) for (plev\*2); ball of force (radius 2) for (plev/5) | Shoots a radius-2 shards ball and throws around anything in the area of effect. |
| Banishment | 35 | 45 | 95 | 25 | removes all of a given creature type from the level | Removes all monsters represented by a chosen symbol from the level. Uniques are not affected. You take 1d4 points of damage for every monster removed. |
| Mass Banishment | 40 | 75 | 90 | 100 | removes all nearby creatures | Removes all monsters within 20 squares of you. Uniques are not affected. You take 1d3 points of damage for every monster removed. |
| Mana Storm | 45 | 16 | 85 | 200 | ball of mana (radius 3) for (plev\*2+300) | Shoots a radius-3 mana ball that does unresistable damage to each monster in its area of effect. |

## Druid

Realm **nature** (spell stat WIS). First spell at level 1; total armour weight above 350 (35.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 5 books. Class flags: ZERO\_FAIL, CHOOSE\_SPELLS, CHARM.

### Druid: Lesser Charms

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Detect Life | 1 | 1 | 23 | 2 | detects living creatures nearby (area 22 x 40) | Detects all living monsters in the immediate area, for one turn only. |
| Fox Form | 1 | 2 | 20 | 10 | change into fox shape | Changes you into the shape of a quick and stealthy fox. While shapechanged you cannot use items. |
| Remove Hunger | 2 | 2 | 25 | 2 | nourish up to 50 (x food-value) | Magically renders you comfortably fed (but not satiated). |
| Stinking Cloud | 3 | 2 | 27 | 4 | ball of poison (radius 2) for (plev/2+10) | Shoots a radius-2 poison ball. |
| Confuse Monster | 5 | 3 | 30 | 5 | bolt that attempts to confuse monsters (power 5+d(plev-5)) | Confuses a single monster; more effective against animals. |
| Slow Monster | 6 | 4 | 30 | 5 | bolt that attempts to slow monsters (power 5+d(plev-5)) | Slows down a monster; more effective against animals. |

### Druid: Gifts of Nature

*town book: Bookseller, cost 400, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Cure Poison | 4 | 4 | 30 | 4 | cure poisoning | Neutralizes poison. |
| Resist Poison | 7 | 6 | 32 | 5 | +20+d20 turns of poison resistance | Grants you resistance to poison for 20+1d20 turns. |
| Turn Stone to Mud | 8 | 3 | 25 | 5 | line of stone to mud for 20+1d30 | Produces a beam that destroys the first section of wall it hits, unless it is a permanent wall. Each monster caught in the beam that is susceptible to rock remover takes 20+1d30 points of damage. |
| Sense Surroundings | 9 | 4 | 35 | 6 | maps the area around you (((plev/30+1)\*22)d((plev/30+1)\*44)) | Maps the nearby area. |
| Lightning Strike | 12 | 6 | 35 | 8 | strike of lightning at target (radius 0) for (plev/4)d4; strike of sound at target (radius 3) for (plev+5) | Strikes a target from above with a lightning bolt, followed by a clap of thunder. If targeting a direction or the target is not a passable grid in the line of sight, the player will be the target. |
| Earth Rising | 14 | 5 | 40 | 12 | short beam of shards (length 4+plev/5) for (plev/3+2)d6 | Causes shards to explode in a line out of the earth, striking objects and monsters. Length of the line increases with player level. |

### Druid: Creature Dominion

*dungeon book: depth 30-100 (x20), cost 10000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Trance | 20 | 10 | 45 | 15 | attempts to hold monsters still on adjacent grids (radius 1) (power 3+d(plev/7)) | Holds every adjacent, non-resistant monster for a level-dependent number of turns; animals are entranced for longer. |
| Mass Sleep | 25 | 15 | 50 | 15 | attempts to put all monsters to sleep in line of sight (power (plev\*10+500)) | Attempts to put to sleep each monster within line of sight. Monsters that resist sleep are not affected. |
| Become Pukel-man | 30 | 20 | 75 | 50 | change into Pukel-man shape | Gives you the form of a statue of the Drúedain, slow but hard to hurt. |
| Eagle's Flight | 35 | 20 | 80 | 80 | change into eagle shape | Gives you the form of one of the great eagles of Manwë. |
| Bear Form | 40 | 20 | 85 | 100 | change into bear shape | Allows you to take the form of a great bear of the mountains, sweeping aside your enemies with your mighty paws. |

### Druid: Nature Craft

*dungeon book: depth 40-100 (x15), cost 30000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Tremor | 20 | 20 | 60 | 16 | earthquake (radius 4) | Causes a targeted 4-square radius earthquake. Earthquakes may hurt the player, damage monsters and destroy items in some squares in the area of effect. Artifacts will not be destroyed by this spell. This spell has no effect when used in the town. |
| Haste Self | 25 | 12 | 65 | 15 | +plev+d20 turns of haste | Hastes you (+10 to speed) for a level-dependent duration. |
| Revitalize | 35 | 70 | 90 | 90 | restore STR; restore INT; restore WIS; restore DEX; restore CON; restores your experience | Restores all stats and experience points to maximum. |
| Rapid Regeneration | 37 | 20 | 90 | 100 | +5+1d3 turns of turn-by-turn healing through increased metabolism | Speeds up digestion to increase regeneration of hitpoints for the next 5+1d3 turns (or until hungry). |
| Herbal Curing | 40 | 20 | 90 | 100 | cure wounds; cure poisoning; cure stunning; cure black breath; nourish up to 80 (x food-value) | Cures all cuts, poison, stunning and black breath, and leaves you well-nourished. |

### Druid: Wild Forces

*dungeon book: depth 60-100 (x10), cost 50000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Meteor Swarm | 30 | 14 | 85 | 20 | swarm of (plev/20+2) balls of meteors (radius 1) for (plev/2+30) each | Shoots a swarm of 3 or 4 meteors. Meteors will travel until hitting a wall, a monster, or reaching the target location. Meteors will explode as a radius-1 ball that does unresistable damage to each monster in its area of effect. If this kills the monster that blocked the path to the target location, the next meteor may continue to find another target. |
| Rift | 35 | 20 | 60 | 25 | beam of gravity for 40+plevd7 | Fires a gravity beam that hurts each monster in its path and has a chance of teleporting it away. |
| Ice Storm | 37 | 25 | 75 | 24 | ice in line of sight (power 3d(plev\*3)) | Strikes every monster in line of sight with sheets of ice from above. |
| Volcanic Eruption | 40 | 30 | 75 | 50 | sphere of fire around you (radius 5) for (plev\*3/2)+d(plev\*3); earthquake (radius 5) | Creates a radius 5 ball of fire, followed by a radius 5 earthquake, around the player. |
| River of Lightning | 43 | 35 | 75 | 100 | arc of plasma (20-degree) for (plev+10)d8 | Spews plasma in a long narrow cone from the player. |

## Priest

Realm **divine** (spell stat WIS). First spell at level 1; total armour weight above 350 (35.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 5 books. Class flags: BLESS\_WEAPON, ZERO\_FAIL.

### Priest: Novice's Handbook

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Call Light | 1 | 1 | 10 | 2 | lights up the surrounding area; sphere of weak light around you (radius 1) for 2d(plev/2) | Lights up all squares in a level-dependent area, and hurts all light-sensitive monsters in the area of effect. If you are in a room, the entire room will be lit up as well. |
| Detect Evil | 1 | 1 | 10 | 4 | detects evil creatures nearby (area 22 x 40) | Detects all evil monsters in the immediate area, for one turn only. |
| Minor Healing | 1 | 2 | 15 | 4 | heal (plev+10) HP or $M% of HP lost, whichever is more; reduce wounds by 20; reduce stunning by 20 | Heals a level-dependent amount or percentage of hitpoints, and cures 20 points worth of cuts and of stunning. |
| Bless | 1 | 2 | 20 | 4 | +(plev+10)+d(plev+10) turns of your AC and to-hit bonus | Blesses you, giving a bonus of +5 to AC and +10 to-hit, for a level-dependent number of turns. |
| Sense Invisible | 3 | 4 | 25 | 4 | +24+d24 turns of see invisible | Enables you to see invisible monsters for 24+1d24 turns. |
| Heroism | 5 | 2 | 30 | 5 | heal 10 HP; cure fear; +(plev-19)+d(plev-19) turns of heroism | Cures 10 points of damage, removes fear and, from character level 20, grants you heroism (resistance to fear, and a bonus of +12 to-hit) for a level-dependent duration. |

### Priest: Cleansing Power

*town book: Bookseller, cost 100, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Orb of Draining | 7 | 7 | 40 | 4 | ball of holy power (radius 2+plev/30) for (plev\*3/2)+3d6 | Shoots a radius-2 or radius-3 ball that does unresistable damage to each monster in its area of effect. Evil monsters take double damage. The bigger area of effect is attained at character level 30. |
| Spear of Light | 7 | 6 | 30 | 5 | line of weak light for 6d8 | Fires a beam that lights up each square and hurts each light-sensitive monster in its path. |
| Dispel Undead | 10 | 14 | 55 | 6 | damages undead in line of sight (power d(plev\*5)) | Inflicts unresistable damage on each undead monster within line of sight. |
| Dispel Evil | 12 | 20 | 70 | 10 | damages evil monsters in line of sight (power d(plev\*5)) | Inflicts unresistable damage on each evil monster within line of sight. |
| Protection from Evil | 14 | 8 | 42 | 5 | +(plev\*3)+d25 turns of protection from evil | Protects you from evil for a level-dependent duration: all melee attacks by evil monsters have a chance to be repelled, unless the monster's level is higher than your character level. |
| Remove Curse | 16 | 6 | 38 | 8 | remove a curse (power plev+dplev) | Attempts to remove a curse from an item, more effective at higher character levels. |

### Priest: Healing and Sanctuary

*dungeon book: depth 30-100 (x20), cost 10000, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Portal | 10 | 4 | 30 | 8 | teleport self up to (plev/2+30) grids | Teleports you randomly over a short distance. |
| Remembrance | 20 | 30 | 90 | 50 | restores your experience | Restores experience points to maximum if it is currently below maximum. |
| Word of Recall | 25 | 30 | 75 | 10 | returns you from the dungeon or takes you to the dungeon after a short delay | Teleports you from the dungeon to the town or from the town to the deepest level you have visited in the dungeon. The recall effect is not immediate; it is delayed by 14+1d20 turns. During that delay, the prayer can be canceled by invoking the prayer of recall again. This prayer has no effect when the option to restrict the use of stairs and recall is set, unless Morgoth is dead. |
| Healing | 30 | 50 | 80 | 100 | heal 2000 HP; cure wounds; cure poisoning; cure stunning; cure amnesia | Heals 2000 points of damage and cures all stunning, cuts, poison and amnesia. |
| Restoration | 35 | 70 | 90 | 130 | restore STR; restore INT; restore WIS; restore DEX; restore CON | Restores to maximum all stats that are currently below maximum. |
| Clairvoyance | 37 | 50 | 80 | 150 | completely lights up and magically maps the level | Lights up, maps out, and senses all objects on the complete current dungeon level. |

### Priest: Battle Blessings

*dungeon book: depth 40-100 (x15), cost 30000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Enchant Weapon | 35 | 50 | 80 | 230 | enchant to-hit and to-dam (power 1d4) | Attempts to improve the to-hit bonus and the to-dam bonus of a weapon. |
| Enchant Armour | 37 | 60 | 85 | 250 | enchant armour class (power 1+1d3) | Attempts to improve the armour class bonus of a piece of armour. |
| Smite Evil | 25 | 20 | 70 | 40 | +20+d20 turns of temporary slay evil | Temporarily makes your melee attacks slay evil creatures. |
| Glyph of Warding | 20 | 40 | 90 | 35 | inscribe a glyph of warding | Inscribes a glyph of warding beneath you. Summoned monsters can't appear on the glyph. If a monster attempts to move onto the glyph or melee you while you are standing on the glyph, it must first succeed in breaking the glyph. Higher level monsters are more likely to break the glyph. The glyph will remain where it is, until a monster succeeds in breaking it, or until you leave the level. |
| Demon Bane | 42 | 40 | 80 | 300 | +20+d20 turns of temporary slay demon | Temporarily makes your melee attacks powerfully slay demons. |

### Priest: Wrath of the Valar

*dungeon book: depth 60-100 (x10), cost 50000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Banish Evil | 25 | 25 | 80 | 250 | teleports evil monsters away in line of sight (power 100) | Teleports away each evil monster within line of sight. |
| Word of Destruction | 35 | 35 | 80 | 115 | destruction (radius 15) | Destroys everything in a 15-square radius circle around you. All monsters, objects, and terrain features in the area of effect are destroyed, except stairs and permanent walls. You will also be blinded for 10+1d10 turns, unless you have resistance to blindness or light. This prayer has no effect when used in the town. |
| Holy Word | 39 | 32 | 95 | 20 | damages evil monsters in line of sight (power d(plev\*4)); heal 1000 HP; cure fear; cure poisoning; cure stunning; cure wounds | Inflicts unresistable damage on each evil monster within line of sight, cures 1000 points of damage, cures all stunning, heals all cut damage, neutralizes poison, and removes fear. |
| Spear of Oromë | 40 | 10 | 75 | 130 | beam of holy power for (plev/2)d8 | Fires a beam of holy magic that does extra damage to evil creatures. |
| Light of Varda | 42 | 40 | 85 | 200 | ball of light (radius 4) for (plev\*5); light in line of sight (power 100) | Casts a radius 4 ball of strong light, and then bathes the entire area in light doing 100 damage to every light-sensitive creature. |

## Necromancer

Realm **shadow** (spell stat INT). First spell at level 1; total armour weight above 300 (30.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 5 books. Class flags: ZERO\_FAIL, CHOOSE\_SPELLS, UNLIGHT, EVIL.

### Necromancer: Into the Shadows

*town book: Bookseller, cost 25, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Nether Bolt | 1 | 1 | 22 | 2 | bolt of nether for (plev/4+3)d4 | Fires a bolt of nether magic. |
| Sense Invisible | 2 | 4 | 25 | 3 | +24+d24 turns of see invisible | Enables you to see invisible monsters for 24+1d24 turns. |
| Create Darkness | 3 | 3 | 10 | 4 | darken the area; sphere of weak darkness around you (radius 1) for 1 | Darkens all squares in a level-dependent area. If you are in a room, the entire room will be darkened as well. |
| Bat Form | 3 | 4 | 20 | 10 | change into bat shape | Changes you into the shape of a bat, weak but with excellent senses. While shapechanged you cannot use items. |
| Read Minds | 4 | 4 | 10 | 6 | detects creatures with a spirit nearby (area 22 x 40); maps the area around recently detected monsters ((plev/10)d(plev/10+1)) | Detects all monsters which have a spirit in the immediate area, and then maps in a region around them, increasing in size with character level. |

### Necromancer: Dark Rituals

*town book: Bookseller, cost 400, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Tap Unlife | 7 | 0 | 50 | 3 | drain (plev/4+2)d6 mana from nearest undead, damaging it | Targets the nearest undead monster and damages it to gain player mana. |
| Crush | 10 | 6 | 40 | 5 | kills monsters below a hitpoint threshold in line of sight (power (plev\*4)); d(plev\*2) damage to you | Instantly kills any monster in line of sight with less than four times the player's level in hitpoints, also hurting the player. Other monsters are unharmed. |
| Sleep Evil | 12 | 5 | 50 | 5 | attempts to put evil monsters to sleep in line of sight (power (plev\*10+500)) | Attempts to put to sleep each evil monster within line of sight. Monsters that resist sleep are not affected. |
| Shadow Shift | 15 | 10 | 50 | 10 | teleport self up to 30 grids; 2d5 damage to you | Displaces your spirit a short distance, hurting your body as it is dragged along. |
| Disenchant | 16 | 12 | 50 | 15 | bolt of disenchantment for 2d(plev\*2+10) | Casts a bolt which wounds, and makes spellcasting monsters less successful in casting spells. |

### Necromancer: Fear and Torment

*town book: Bookseller, cost 1600, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Frighten | 17 | 4 | 40 | 12 | bolt that causes monsters to flee (power plev) | Attempts to scare a single monster for a level-dependent duration. Monsters that resist fear are not affected. |
| Vampire Strike | 20 | 5 | 50 | 10 | jumps the player to the closest living monster and bites it ((plev\*2)) | Teleports you to the nearest living monster and drains its hitpoints to heal and nourish you. The number of hitpoints drained is twice your level or one more than the monster's current hitpoints, whichever is smaller. |
| Dispel Life | 22 | 20 | 50 | 10 | damages living monsters in line of sight (power d(plev\*3)) | Inflicts unresistable damage on each living monster within line of sight. |
| Dark Spear | 25 | 10 | 60 | 12 | line of dark for 2d(plev\*2) | Fires a beam of darkness. |
| Warg Form | 28 | 20 | 60 | 80 | change into warg shape | Changes you into the form of a warg, and gives you temporary berserk strength, at the cost of some hitpoints. |

### Necromancer: Deadly Powers

*dungeon book: depth 40-100 (x15), cost 30000, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Banish Spirits | 25 | 25 | 80 | 25 | teleports monsters with a spirit away in line of sight (power 100) | Teleports away each monster with a spirit within line of sight. |
| Annihilate | 30 | 20 | 75 | 25 | bolt that damages living monsters for (plev\*4) | Inflicts damage equal to four times player level on a single monster that is not undead, an elemental, a golem, or a vortex. |
| Grond's Blow | 35 | 35 | 80 | 115 | destruction (radius 15) | Destroys everything in a 15-square radius circle around you. All monsters, objects, and terrain features in the area of effect are destroyed, except stairs and permanent walls. You will also be blinded for 10+1d10 turns, unless you have resistance to blindness or dark. This ritual has no effect when used in the town. |
| Unleash Chaos | 38 | 15 | 90 | 100 | one at random of {beam of chaos for 8dplev / arc of chaos (20-degree) for 8dplev / ball of chaos (radius 3) for 8dplev}; wake up nearby monsters; causes monsters to flee in line of sight (power plev) | Fires a chaos beam, arc or ball in the chosen direction, wakes everything in the area, and attempts to frighten everything in line of sight. |
| Fume of Mordor | 40 | 50 | 80 | 150 | completely darkens up and magically maps the level (1) | Darkens, maps out, and detects all objects on the complete current dungeon level. |
| Storm of Darkness | 43 | 16 | 80 | 200 | ball of dark (radius 4) for 4d(plev\*2) | Invokes a radius-4 darkness ball. |

### Necromancer: Corruption of Spirit

*dungeon book: depth 60-100 (x10), cost 50000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Power Sacrifice | 27 | 0 | 20 | 20 | 50 damage to you; restore 50 SP | Exchanges 50 hitpoints for 50 mana. |
| Zone of Unmagic | 32 | 16 | 60 | 110 | ball of disenchantment on yourself (radius 4) for (plev\*3) | Produces a zone of disenchantment affecting the player and monsters within radius 4. |
| Vampire Form | 37 | 20 | 60 | 140 | change into vampire shape; jumps the player to the closest living monster and bites it (plev) | Allows you to assume the form of a vampire at the cost of half your current hitpoints, then teleports you to the nearest living monster and drains a level-dependent number of hitpoints, healing and nourishing you. When first transformed you will temporarily be able to take hitpoints from living monsters with your bite attack. |
| Curse | 40 | 50 | 60 | 180 | 100 damage to you; damage a monster directly for (plev/12+1)d(target\_%HP\_lost+50) | Deals direct damage to a single monster, more the more damaged the monster is already, at the cost of 100 hitpoints. |
| Command | 45 | 40 | 60 | 250 | takes control of a monster (5+1d10) | Allows you to take control of a monster, and move it instead of yourself. Possible actions are move, stand still, 'd'rop, 'm'agic, or 'r'elease. |

## Paladin

Realm **divine** (spell stat WIS). First spell at level 1; total armour weight above 400 (40.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 3 books. Class flags: BLESS\_WEAPON, SHIELD\_BASH.

### Paladin: Novice's Handbook

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Bless | 1 | 2 | 20 | 4 | +(plev+10)+d(plev+10) turns of your AC and to-hit bonus | Blesses you, giving a bonus of +5 to AC and +10 to-hit, for a level-dependent number of turns. |
| Detect Evil | 3 | 2 | 10 | 4 | detects evil creatures nearby (area 22 x 40) | Detects all evil monsters in the immediate area, for one turn only. |
| Call Light | 5 | 3 | 10 | 2 | lights up the surrounding area; sphere of weak light around you (radius 1) for 2d(plev/2) | Lights up all squares in a level-dependent area, and hurts all light-sensitive monsters in the area of effect. If you are in a room, the entire room will be lit up as well. |
| Minor Healing | 7 | 3 | 15 | 4 | heal (plev+10) HP or $M% of HP lost, whichever is more; reduce wounds by 20; reduce stunning by 20 | Heals a level-dependent amount or percentage of hitpoints, and cures 20 points worth of cuts and of stunning. |
| Sense Invisible | 8 | 4 | 25 | 4 | +24+d24 turns of see invisible | Enables you to see invisible monsters for 24+1d24 turns. |
| Heroism | 12 | 5 | 30 | 5 | heal 10 HP; cure fear; +(plev-14)+d(plev-14) turns of heroism | Cures 10 points of damage, removes fear and, from character level 15, grants you heroism (resistance to fear, and a bonus of +12 to-hit) for a level-dependent duration. |

### Paladin: Healing and Sanctuary

*dungeon book: depth 30-100 (x20), cost 10000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Protection from Evil | 15 | 8 | 42 | 5 | +(plev\*3)+d25 turns of protection from evil | Protects you from evil for a level-dependent duration: all melee attacks by evil monsters have a chance to be repelled, unless the monster's level is higher than your character level. |
| Remove Curse | 20 | 12 | 38 | 8 | remove a curse (power plev+dplev) | Attempts to remove a curse from an item, more effective at higher character levels. |
| Word of Recall | 25 | 30 | 75 | 10 | returns you from the dungeon or takes you to the dungeon after a short delay | Teleports you from the dungeon to the town or from the town to the deepest level you have visited in the dungeon. The recall effect is not immediate; it is delayed by 14+1d20 turns. During that delay, the prayer can be canceled by invoking the prayer of recall again. This prayer has no effect when the option to restrict the use of stairs and recall is set, unless Morgoth is dead. |
| Healing | 30 | 50 | 80 | 100 | heal 2000 HP; cure wounds; cure poisoning; cure stunning; cure amnesia | Heals 2000 points of damage and cures all stunning, cuts, poison and amnesia. |
| Clairvoyance | 37 | 50 | 80 | 150 | completely lights up and magically maps the level | Lights up, maps out, and senses all objects on the complete current dungeon level. |

### Paladin: Battle Blessings

*dungeon book: depth 40-100 (x15), cost 30000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Smite Evil | 25 | 20 | 70 | 40 | +20+d20 turns of temporary slay evil | Temporarily makes your melee attacks slay evil creatures. |
| Demon Bane | 30 | 40 | 80 | 150 | +20+d20 turns of temporary slay demon | Temporarily makes your melee attacks powerfully slay demons. |
| Enchant Weapon | 35 | 50 | 80 | 230 | enchant to-hit and to-dam (power 1d4) | Attempts to improve the to-hit bonus and the to-dam bonus of a weapon. |
| Enchant Armour | 37 | 60 | 85 | 250 | enchant armour class (power 1+1d3) | Attempts to improve the armour class bonus of a piece of armour. |
| Single Combat | 40 | 50 | 30 | 300 | engages a monster in single combat | Encases you and the targeted monster in a stone cell from which only the winner can emerge. |

## Rogue

Realm **arcane** (spell stat INT). First spell at level 5; total armour weight above 350 (35.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 2 books. Class flags: CHOOSE\_SPELLS, STEAL.

### Rogue: First Spells

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Detect Monsters | 5 | 1 | 50 | 4 | detects visible creatures nearby (area 22 x 40) | Detects all non-invisible monsters in the immediate area, for one turn only. |
| Phase Door | 7 | 2 | 55 | 4 | teleport self up to 10 grids | Teleports you randomly up to 10 squares away. |
| Object Detection | 10 | 3 | 60 | 5 | detects veins nearby (area 22 x 40); detects gold nearby (area 22 x 40); detects objects nearby (area 22 x 40) | Detects all objects in the immediate area. |
| Detect Stairs | 12 | 3 | 50 | 5 | detects stairs nearby (area 44 x 60) | Detects all stairs in a wide area. |
| Recharging | 20 | 10 | 50 | 6 | recharge a wand/staff (power (plev/10+4)) | Adds charges to a stack of wands or staves. Chance of success and number of charges gained increase with your level and decrease with level of wand or staff and number of charges. A failed attempt to recharge destroys one wand or staff from the stack. |
| Reveal Monsters | 25 | 3 | 40 | 8 | detects invisible creatures nearby (area 22 x 40); detects visible creatures nearby (area 22 x 40) | Detects all monsters in the immediate area, for one turn only. |

### Rogue: Arcane Control

*dungeon book: depth 40-100 (x15), cost 30000, 4 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Teleport Self | 17 | 6 | 35 | 5 | teleport self $M% of the level's larger dimension | Teleports you randomly within the current level. |
| Hit and Run | 23 | 20 | 40 | 20 | +10 turns of escape on attack | Prepares you to teleport a short distance after stealing from a monster. |
| Teleport Other | 30 | 10 | 30 | 50 | bolt that teleports monsters away (power (plev\*3)) | Produces a bolt that teleports away the first monster in its path. Distance teleported increases with player level. |
| Teleport Level | 35 | 17 | 65 | 80 | teleport level (up or down) | Teleports you 1 level up or 1 level down (chosen at random). This spell may have no effect when the forced descent option is set. |

## Ranger

Realm **nature** (spell stat WIS). First spell at level 3; total armour weight above 400 (40.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 2 books. Class flags: FAST\_SHOT, CHOOSE\_SPELLS.

### Ranger: Lesser Charms

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Remove Hunger | 3 | 1 | 25 | 2 | nourish up to 50 (x food-value) | Magically renders you comfortably fed (but not satiated). |
| Detect Life | 5 | 2 | 23 | 2 | detects living creatures nearby (area 22 x 40) | Detects all living monsters in the immediate area, for one turn only. |
| Herbal Curing | 9 | 6 | 30 | 4 | cure wounds; cure poisoning; cure stunning; cure black breath | Cures all cuts, poison, stunning and black breath. |
| Resist Poison | 12 | 10 | 32 | 5 | +20+d20 turns of poison resistance | Grants you resistance to poison for 20+1d20 turns. |
| Turn Stone to Mud | 15 | 5 | 25 | 5 | line of stone to mud for 20+1d30 | Produces a beam that destroys the first section of wall it hits, unless it is a permanent wall. Each monster caught in the beam that is susceptible to rock remover takes 20+1d30 points of damage. |
| Sense Surroundings | 20 | 8 | 35 | 8 | maps the area around you (((plev/30+1)\*22)d((plev/30+1)\*44)) | Maps the nearby area. |

### Ranger: Nature Craft

*dungeon book: depth 40-100 (x15), cost 30000, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Cover Tracks | 20 | 20 | 40 | 40 | +(plev\*2)+d20 turns of leave no scent trail, reduce visibility | Prevents you from leaving scent for monsters to track you by for a level-dependent duration. |
| Create Arrows | 22 | 20 | 40 | 40 | uses a staff to create a stack of arrows | Allows the player to turn a staff into a stack of arrows, with quality depending on staff level and player level. |
| Haste Self | 25 | 12 | 65 | 15 | +plev+d20 turns of haste | Hastes you (+10 to speed) for a level-dependent duration. |
| Decoy | 30 | 30 | 40 | 60 | inscribe a decoy | Leaves a magical decoy that looks and sounds like the player. |
| Brand Ammunition | 40 | 60 | 95 | 120 | brands a stack of ammunition | Brands one stack of ammunition with fire, cold, or poison (selected at random) and at the same time attempts to improve the to-hit bonus and the to-dam bonus of the same ammunition. The spell has no effect if the ammunition is already branded, has a slay, is broken, or is cursed. |

## Blackguard

Realm **shadow** (spell stat INT). First spell at level 1; total armour weight above 600 (60.0 lb) costs 1 SP per pound over (`calc_mana()` in `src/player-calcs.c`). 3 books. Class flags: CHOOSE\_SPELLS, SHIELD\_BASH, COMBAT\_REGEN.

### Blackguard: Into the Shadows

*town book: Bookseller, cost 25, 6 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Seek Battle | 1 | 1 | 15 | 2 | detects creatures nearby which are susceptible to fear (area 22 x 40) | Detects all monsters susceptible to fear in the immediate area, for one turn only. |
| Berserk Strength | 3 | 2 | 20 | 4 | cure fear; +(plev+5)+d(plev+5) turns of berserk rage | Removes fear and grants you berserk strength (resistance to fear, +75 melee skill, -10 to AC, and prevents healing over time) for a level-dependent duration. |
| Whirlwind Attack | 5 | 4 | 25 | 6 | ((plev+10)/15) blows against all adjacent monsters | Deals a level-dependent number of melee blows to each adjacent enemy. At level 5 you do one blow and gain an additional blow every 15 levels (2 at 20, 3 at 35, and 4 at 50). |
| Shatter Stone | 7 | 3 | 20 | 8 | short beam of stone to mud (length 1) for 20+1d30 | Destroys an adjacent section of wall or rubble, unless it is a permanent wall. Deals 20+1d30 damage to monsters vulnerable to rock remover. |
| Leap into Battle | 10 | 5 | 30 | 10 | move up to 4 grids and make up to ((plev+5)/15) blows | You run up to 4 spaces towards the targeted enemy, then perform a melee blow. You gain an additional blow at levels 25 and 40. If you moved, the number of blows is reduced by 25% for each space, then rounded. |
| Grim Purpose | 13 | 8 | 30 | 12 | +12+d12 turns of confusion resistance; +12+d12 turns of free action | Your mind becomes too focused to be slowed, paralyzed or confused for 12+1d12 turns. |

### Blackguard: Fear and Torment

*town book: Bookseller, cost 1600, 5 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Maim Foe | 15 | 6 | 35 | 14 | (plev/15) blows on an adjacent monster; each damaging blow also attempts to stun monsters (power 6) | Attempts a level-dependent number of melee blows against the targeted enemy. You get 1 blow for every 15 levels. If any of these blows damage the enemy, it will be stunned for 6 turns. |
| Howl of the Damned | 18 | 8 | 30 | 16 | causes monsters to flee in line of sight (power plev) | Attempts to scare every monster in line of sight for a duration dependent on player level and monster depth. Monsters that resist fear are not affected. |
| Relentless Taunting | 24 | 10 | 32 | 20 | +12+d12 turns of taunting | Makes any monster that can move more inclined to run toward you, and less likely to use a ranged attack (including spells and innate abilities). |
| Venom | 28 | 18 | 35 | 30 | +18+d18 turns of temporary poison brand | Coats all your melee weapons with poison for 18+1d18 turns. |
| Werewolf Form | 32 | 30 | 40 | 100 | change into werewolf shape | With a blood-curdling cry you change into a half-man half-wolf! In this form you move, attack, and heal faster. |

### Blackguard: Deadly Powers

*dungeon book: depth 40-100 (x15), cost 30000, 4 spells for this class.*

| Spell | Lvl | Mana | Fail | Exp | Effect | Description |
|---|---:|---:|---:|---:|---|---|
| Bloodlust | 30 | 25 | 37 | 80 | +10 turns of desire to kill | Gives you a powerful desire to kill, increased every time you satisfy it. Gives you extra melee damage and extra blows, but comes with dangerous side-effects. |
| Unholy Reprieve | 34 | 45 | 60 | 120 | restore STR; restore INT; restore CON; restores your experience; 66 damage to you | Restores STR, INT, CON, and experience at the cost of 66 HP. |
| Forceful Blow | 38 | 16 | 50 | 150 | 1 blows on an adjacent monster; each damaging blow also force (power 200) | Attempts a melee blow that deals additional force damage when it hits, knocking the target back with a chance to stun it. |
| Quake | 44 | 39 | 55 | 175 | earthquake (radius 13) | Causes a radius 13 earthquake centered around the player. |

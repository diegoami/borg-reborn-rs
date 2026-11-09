# Appendix A10. Curses, traps, timed effects and properties

> **Generated file - do not edit by hand.** Produced by `docs/c-manual/tools/gen_appendices.py` from `lib/gamedata/curse.txt`, `lib/gamedata/trap.txt`, `lib/gamedata/chest_trap.txt`, `lib/gamedata/player_timed.txt`, `lib/gamedata/player_property.txt`, `lib/gamedata/object_property.txt`, `lib/gamedata/object.txt`, `lib/gamedata/activation.txt`, `lib/gamedata/class.txt`, `lib/gamedata/slay.txt`, `lib/gamedata/brand.txt`, `src/trap.c` of the Angband source tree at commit `e473720ed`. Regenerate with `python3 docs/c-manual/tools/gen_appendices.py --src <angband-source-root>` (run from the repository root).

Every object curse and the item types it can appear on; every trap and rune with depth, effect and save; chest traps; every timed effect with its grades, what protects against it and every item, activation or spell that cures or reduces it; player and object properties; and the rune list used by identification. Advice: [Chapter 5](../05-elements-and-status.md), [Chapter 6](../06-equipment.md), [Chapter 9](../09-dungeon.md).

## Contents

- [Curses](#curses)
- [Traps and runes](#traps-and-runes)
- [Chest traps](#chest-traps)
- [Timed effects](#timed-effects)
- [Player properties](#player-properties)
- [Object properties and runes](#object-properties-and-runes)
  - [Slays](#slays)
  - [Brands](#brands)

## Curses

From `curse.txt`. A curse can only be put on the listed object bases. *Combat* is `to-hit : to-dam : AC` adjustment; *Random effect* fires on average once every *time* turns. *Power* (on the item) decides how hard the curse is to remove. Items that carry fixed curses are listed after the table.

| Curse | Description | Item types | Combat | Mods | Flags | Random effect | Conflicts with |
|---|---|---|---|---|---|---|---|
| vulnerability | attracts opponents and weakens your defences | cloak, soft armor, hard armor, dragon armor, shield | 0:0:-50 | - | AGGRAVATE | - | - |
| teleportation | randomly makes you teleport | helm, crown, amulet, ring | - | - | - | teleport self up to 40 grids (every d100 turns) | anti-teleportation, NO\_TELEPORT |
| dullness | makes you mentally slow | helm, crown | - | INT -5, WIS -5 | - | - | - |
| sickliness | makes you frail | helm, crown | - | STR -5, DEX -5, CON -5 | - | - | - |
| enveloping | protects you at the expense of free movement | cloak | -5:-5:20 | - | - | - | - |
| irritation | annoys monsters and makes it hard for you to hit them | cloak | -15:-15:0 | - | AGGRAVATE | - | - |
| weakness | weakens your grasp | gloves | - | STR -10 | - | - | - |
| clumsiness | gives you butterfingers | gloves | - | DEX -10 | - | - | - |
| slowness | drags at your feet | boots | - | SPEED -5 | - | - | - |
| annoyance | wakes up and annoys nearby monsters, and delays your escape | boots | - | SPEED -10, STEALTH -10 | AGGRAVATE | - | - |
| poison | poisons you from time to time | bow, digger, hafted, polearm, sword, boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor, light, amulet, ring | - | - | - | +10+d10 turns of poisoning (every 1d500 turns) | - |
| siren | occasionally makes a loud noise | bow, digger, hafted, polearm, sword, boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor, light, amulet, ring | - | - | - | wake up nearby monsters (every 100+1d50 turns) | - |
| hallucination | makes you hallucinate sometimes | bow, digger, hafted, polearm, sword, boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor, light, amulet, ring | - | - | - | +10+d10 turns of hallucination (every 1d500 turns) | - |
| paralysis | paralyses you every now and then | bow, digger, hafted, polearm, sword, boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor, light, amulet, ring | - | - | - | +5+1d5 turns of paralysis (every 1d1000 turns) | - |
| dragon summon | sometimes summons a dragon | dragon armor | - | - | - | summon 1 x dragons (every 1d500 turns) | - |
| demon summon | occasionally summons a demon | digger, hafted, polearm, sword | - | - | - | summon 1 x demons (every 1d500 turns) | - |
| undead summon | occasionally summons an undead | digger, hafted, polearm, sword | - | - | - | summon 1 x undead (every 1d500 turns) | - |
| impair mana recovery | makes you slow to recover mana | helm, crown | - | - | IMPAIR\_MANA | - | IMPAIR\_MANA |
| impair hitpoint recovery | makes you slow to recover hitpoints | boots, gloves, shield, cloak, soft armor, hard armor, dragon armor | - | - | IMPAIR\_HP | - | IMPAIR\_HP |
| cowardice | makes you too scared to fight | amulet, ring | - | - | AFRAID | - | AFRAID, PROT\_FEAR |
| stone | periodically turns your skin to stone | amulet, ring | - | - | - | +20+1d20 turns of stone skin (every 1d1000 turns) | - |
| anti-teleportation | prevents you from teleporting | boots, cloak, soft armor, hard armor, dragon armor, amulet, ring | - | - | NO\_TELEPORT | - | teleportation, NO\_TELEPORT |
| treacherous weapon | makes your weapon attack you | digger, hafted, polearm, sword | - | - | - | weapon\_roll damage to you (every 1d200 turns) | - |
| burning up | makes you vulnerable to fire attacks, but resistant to cold | boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor | - | vulnerable fire, resist cold | - | - | chilled to the bone |
| chilled to the bone | makes you vulnerable to cold attacks, but resistant to fire | boots, gloves, helm, crown, shield, cloak, soft armor, hard armor, dragon armor | - | resist fire, vulnerable cold | - | - | burning up |
| steelskin | makes your skin harder to damage, but it conducts electricity | digger, hafted, polearm, sword, helm, crown, shield, hard armor, dragon armor, light, amulet, ring | 0:0:20 | vulnerable lightning, resist shards | - | - | - |
| air swing | makes it hard to hit anything | digger, hafted, polearm, sword | -20:0:0 | - | - | - | - |

Fixed curses: ring Teleportation: teleportation power 100; amulet Teleportation: teleportation power 100; artifact Bastard Sword 'Calris': air swing power 30.

## Traps and runes

From `trap.txt`; trap mechanics in `hit_trap()` (`src/trap.c`). *Appear* is `rarity : minimum depth : max number per level`. *Save*: an object flag listed on `save:` avoids the trap; `SAVE_ARMOR` = avoided if a monster-style hit roll of power 125 against your AC misses (`check_hit()`); `SAVE_THROW` = avoided if `randint0(100) < skill_save`. *Extra* effects happen 50% of the time when the trap fires. `trap immunity` (TRAP_IMMUNE) avoids all traps. *Disarm*: `MAGICAL` traps use the magical disarm skill, others the physical one. *Visibility* is the trap's hidden-ness value.

| Trap | Shown as | Appear | Visibility | Flags | Effect | Extra (50%) | Save | Description |
|---|---|---|---|---|---|---|---|---|
| no trap | no trap | - | - | - | - | - | - | - |
| glyph of warding | glyph of warding | 0:0:0 | - | GLYPH, VISIBLE, FLOOR | - | - | - | A magical rune which monsters cannot pass without difficulty. |
| decoy | decoy | 0:0:0 | - | GLYPH, VISIBLE, FLOOR | - | - | - | A magical decoy which sounds and looks like the player. |
| door lock | door lock | 0:0:0 | - | LOCK, INVISIBLE | - | - | - | A device which makes a closed door harder to open. |
| web | web | 0:0:0 | - | WEB, VISIBLE | - | - | - | A tangle of sticky spider webs. |
| trap door | trap door | 1:2:0 | 2d10M50 | TRAP, FLOOR, DOWN | 2d8 damage to you | - | FEATHER | It looks like floor, but step incautiously and you will fall through. |
| pit | pit | 1:2:0 | 90 | TRAP, FLOOR, PIT | 2d6 damage to you | - | FEATHER | A hole dug to snare the unwary. |
| spiked pit | pit | 1:2:0 | 80 | TRAP, FLOOR, PIT | 2d6 damage to you | 2d6 damage to you; +4d6 turns of wounds | FEATHER | A hole dug to snare the unwary, with spikes. |
| poison pit | pit | 1:2:0 | 70 | TRAP, FLOOR, PIT | 2d6 damage to you | +4d6 turns of wounds; +8d6 turns of poisoning | FEATHER | A hole dug to snare the unwary, with poisoned spikes. |
| rune of summon foe | strange rune | 1:3:0 | 0 | TRAP, FLOOR, ONETIME, MAGICAL | summon 1 x a monster | - | - | A magical rune which calls a monster to attack you. |
| rune of summoning | strange rune | 2:3:0 | 0 | TRAP, FLOOR, ONETIME, MAGICAL | summon 2+1d3 x a monster | - | - | A magical rune which calls up monsters to attack you. |
| rune of necromancy | strange rune | 1:20:0 | 0 | TRAP, FLOOR, ONETIME, MAGICAL | summon 1d3M2 x undead | - | - | A magical rune which calls the undead to attack you. |
| rune of dragonsong | strange rune | 1:20:0 | 0 | TRAP, FLOOR, ONETIME, MAGICAL | summon 1d3 x dragons | - | - | A magical rune which calls dragons to attack you. |
| hellhole | strange rune | 1:20:0 | 0 | TRAP, FLOOR, ONETIME, MAGICAL | summon 1+1d3 x demons | - | - | A magical rune which calls demons to attack you. |
| teleport rune | strange rune | 1:1:0 | 30+d30 | TRAP, FLOOR, MAGICAL | teleport self 80% of the level's larger dimension | - | - | A magical rune which teleports you to a random location. |
| fire trap | discolored spot | 1:2:0 | 20 | TRAP, FLOOR, MAGICAL | ball of fire on yourself (radius 0) for 4d(dlev/2) | - | - | A strange coloured spot on the ground - beware its fiery attack. |
| acid trap | discolored spot | 1:2:0 | 0 | TRAP, FLOOR, MAGICAL | ball of acid on yourself (radius 0) for 4d(dlev/2) | - | - | A strange coloured spot on the ground - beware its acidic attack. |
| slow dart | dart trap | 1:2:0 | M50 | TRAP, FLOOR | 1d4 damage to you; +20+1d20 turns of slowness (unresistable) | - | SAVE\_ARMOR | A trap which shoots slowing darts. |
| strength loss dart | dart trap | 2:6:0 | 30 | TRAP, FLOOR | 1d4 damage to you; drain STR | - | SAVE\_ARMOR | A trap which shoots strength-draining darts. |
| dexterity loss dart | dart trap | 2:6:0 | 30 | TRAP, FLOOR | 1d4 damage to you; drain DEX | - | SAVE\_ARMOR | A trap which shoots dexterity-draining darts. |
| constitution loss dart | dart trap | 2:6:0 | 30 | TRAP, FLOOR | 1d4 damage to you; drain CON | - | SAVE\_ARMOR | A trap which shoots constitution-draining darts. |
| blinding gas trap | gas trap | 1:2:0 | 20+d5 | TRAP, FLOOR | +25+1d50 turns of blindness | - | - | A trap that releases gas to blind you. |
| confusion gas trap | gas trap | 1:1:0 | 20+d5 | TRAP, FLOOR | +10+1d20 turns of confusion | - | - | A trap that releases gas to confuse you. |
| poison gas trap | gas trap | 1:2:0 | 20+d5 | TRAP, FLOOR | +10+1d20 turns of poisoning | - | - | A trap that releases gas to poison you. |
| sleep gas trap | gas trap | 1:2:0 | 20+d5 | TRAP, FLOOR | +5+1d10 turns of paralysis | - | - | A trap that releases gas to put you to sleep. |
| aggravation trap | alarm | 1:5:0 | M50 | TRAP, FLOOR, MAGICAL | wake up nearby monsters; hastes monsters in line of sight (power 25) | - | - | A trap that wakes nearby monsters and impels them toward you. |
| siren | alarm | 1:1:0 | 60+d10 | TRAP, FLOOR, MAGICAL | wake up nearby monsters | - | - | A trap that wakes nearby monsters. |
| mine trap | explosion | 1:15:0 | 10+d5 | TRAP, FLOOR, ONETIME | ball of shards on yourself (radius 2) for (dlev\*2) | - | - | A trap that blows up the floor. |
| blast trap | explosion | 2:25:0 | 0 | TRAP, FLOOR, ONETIME | ball of light on yourself (radius 1) for 2d6; ball of sound on yourself (radius 2) for (dlev/2); ball of fire on yourself (radius 2) for dlev; ball of force on yourself (radius 2) for dlev | - | - | A trap that creates an explosion. |
| mind blasting trap | mind blast trap | 1:20:0 | 30+d10 | TRAP, FLOOR, MAGICAL | 8d(dlev/10) damage to you; +3+1d4 turns of confusion | - | SAVE\_THROW | A trap that blasts your senses. |
| brain smashing trap | mind blast trap | 2:40:0 | 30+d10 | TRAP, FLOOR, MAGICAL | 10d(dlev/5) damage to you; +3+1d4 turns of slowness; +3+1d4 turns of confusion; +3+1d4 turns of paralysis; +7+1d8 turns of blindness | - | SAVE\_THROW | A trap that blasts your senses. |
| rock fall trap | ancient mechanism | 1:4:0 | 40+d5 | TRAP, FLOOR, ONETIME | (dlev/10+1)d5 damage to you; +2d20 turns of stunning (unresistable); causes rubble to fall around you | - | - | A trap that drops a boulder on you. |
| earthquake trap | ancient mechanism | 1:30:0 | 30+d5 | TRAP, FLOOR | (dlev/10+1)d5 damage to you; earthquake (radius 5) | - | - | A trap that causes an earthquake. |
| block fall trap | ancient mechanism | 1:4:0 | 40+d5 | TRAP, FLOOR, ONETIME, DELAY | causes a granite wall to fall behind you | - | - | A pressure plate that drops a stone block behind you. |
| area blast trap | ancient mechanism | 2:40:0 | 20+d5 | TRAP, FLOOR, ONETIME | ball of stone to mud (radius 2) for 20; ball of force (radius 2) for dlev; causes rubble to fall around you | - | - | A trap that clears the surrounding area. |
| blinding flash trap | trap | 1:5:0 | 30+d5 | TRAP, FLOOR, MAGICAL, ONETIME | ball of light (radius 4) for (dlev/10+2)d8 | - | - | A trap that creates a ball of light. |
| blinding trap | trap | 1:5:0 | 20+d5 | TRAP, FLOOR, MAGICAL, ONETIME | ball of dark (radius 4) for (dlev/10+2)d8; drains your light source (100+1d100) | - | - | A trap that envelops you in darkness. |
| mana drain trap | trap | 1:5:0 | 50+d10 | TRAP, FLOOR, MAGICAL | drain 1d(dlev/2+1) mana | - | - | A trap that drains your mana. |
| knife trap | trap | 1:20:0 | 20+d5 | TRAP, FLOOR | +150 turns of wounds; (dlev/2) damage to you | - | - | A trap designed to bleed you to death. |
| petrifying trap | trap | 1:5:0 | 20+d5 | TRAP, FLOOR, MAGICAL | +20+1d20 turns of stone skin; +20+1d20 turns of stunning | - | - | A trap that stuns you and gives you stoneskin. |

## Chest traps

From `chest_trap.txt`. A chest's trap is chosen from those with *level* <= the chest's level (`src/obj-chest.c`).

| Trap | Code | Level | Effect | Destroys contents | Disarm skill | Message |
|---|---|---:|---|:-:|---|---|
| locked | `NO_TRAP` | 1 | - | - | physical | - |
| gas trap | `POISON` | 1 | +10+d20 turns of poisoning | - | physical | A puff of green gas surrounds you! |
| poison needle | `LOSE_STR` | 2 | 1d4 damage to you; drain STR | - | physical | A small needle has pricked you! |
| poison needle | `LOSE_CON` | 3 | 1d4 damage to you; drain CON | - | physical | A small needle has pricked you! |
| summoning runes | `SUMMON` | 15 | summon 2+1d3 x a monster | - | magical | You are enveloped in a cloud of smoke! |
| gas trap | `PARALYZE` | 19 | +10+d20 turns of paralysis | - | physical | A puff of yellow gas surrounds you! |
| explosion device | `EXPLODE` | 25 | 5d8 damage to you | yes | physical | There is a sudden explosion!  Everything inside the chest is destroyed! |

## Timed effects

From `player_timed.txt`. *Grades* are `max duration: name` steps (for cuts and stuns the grade decides the damage/penalty). *Protection* lists the `fail:` conditions that prevent the effect (object flag, resistance, player flag or another timed effect). *Cured / reduced by* is found by scanning every object kind, activation and class spell for `CURE:` and `TIMED_DEC:` effects (artifact activations appear under their activation name, see [A4](A4-artifacts.md)).

| Effect | Description | Grades | Protection | Notes | Cured / reduced by |
|---|---|---|---|---|---|
| `FAST` | haste | 10000: Haste | - | - | none |
| `SLOW` | slowness | 10000: Slow | FREE\_ACT | - | **activation**: STAFF\_HOLY; **staff**: Holiness |
| `BLIND` | blindness | 10000: Blind | PROT\_BLIND | - | **activation**: CURE\_BLINDNESS, CURE\_BODY, CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_LIGHT, CURE\_NONORLYBIG, CURE\_SERIOUS, CURE\_TEMP, STAFF\_HOLY, TMD\_ESP, TMD\_SINVIS; **mushroom**: Fast Recovery, Second Sight; **potion**: \*Healing\*, Cure Critical Wounds, Cure Light Wounds, Cure Serious Wounds, Healing, Life, True Seeing; **ring**: Open Wounds; **rod**: Curing; **staff**: Cure Light Wounds, Curing, Healing, Holiness |
| `PARALYZED` | paralysis | 10000: Paralyzed! | FREE\_ACT | NONSTACKING: cannot be extended while active | none |
| `CONFUSED` | confusion | 10000: Confused | PROT\_CONF | - | **activation**: CURE\_CONFUSION, CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_LIGHT (-20), CURE\_MIND, CURE\_NONORLYBIG, CURE\_SERIOUS, CURE\_TEMP, STAFF\_HOLY; **mushroom**: Clear Mind; **potion**: \*Healing\*, Cure Critical Wounds, Cure Light Wounds (-20), Cure Serious Wounds, Healing, Life; **ring**: Open Wounds; **rod**: Curing; **staff**: Cure Light Wounds (-20), Curing, Healing, Holiness |
| `AFRAID` | fear | 10000: Afraid | PROT\_FEAR | same as the AFRAID flag | **Blackguard spell**: Berserk Strength; **Paladin spell**: Heroism; **Priest spell**: Heroism, Holy Word; **activation**: CURE\_MIND, CURE\_PARANOIA, HERO, RAGE\_BLESS\_RESIST, REM\_FEAR\_POIS, SHERO, STAFF\_HOLY; **mushroom**: Clear Mind; **potion**: Berserk Strength, Boldness, Heroism; **shape**: warg, werewolf; **staff**: Holiness |
| `IMAGE` | hallucination | 10000: Halluc | resist chaos | - | **activation**: CURE\_MIND, CURE\_NONORLYBIG, STAFF\_HOLY; **mushroom**: Clear Mind; **potion**: Life; **staff**: Holiness |
| `POISONED` | poisoning | 10000: Poisoned | resist poison, timed OPP\_POIS | - | **Druid spell**: Cure Poison, Herbal Curing; **Paladin spell**: Healing; **Priest spell**: Healing, Holy Word; **Ranger spell**: Herbal Curing; **activation**: CURE\_BODY, CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_NONORLYBIG, CURE\_POISON, CURE\_TEMP, FOOD\_WAYBREAD, REM\_FEAR\_POIS, SHROOM\_PURGING, STAFF\_HOLY; **food**: Piece of Elvish Waybread; **mushroom**: Fast Recovery, Purging; **potion**: \*Healing\*, Cure Critical Wounds, Healing, Life, Neutralize Poison, Resist Poison, Salt Water; **rod**: Curing; **shape**: Pukel-man; **staff**: Curing, Healing, Holiness |
| `CUT` | wounds | 10: Graze; 25: Light Cut; 50: Bad Cut; 100: Nasty Cut; 200: Severe Cut; 1000: Deep Gash; 10000: Mortal Wound | player flag ROCK | - | **Druid spell**: Herbal Curing; **Paladin spell**: Healing, Minor Healing (-20); **Priest spell**: Healing, Holy Word, Minor Healing (-20); **Ranger spell**: Herbal Curing; **activation**: CURE\_BODY, CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_LIGHT (-20), CURE\_NONORLYBIG, CURE\_SERIOUS, CURE\_TEMP, HEAL1, HEAL2, HEAL3, STAFF\_HOLY; **mushroom**: Fast Recovery; **potion**: \*Healing\*, Cure Critical Wounds, Cure Light Wounds (-20), Cure Serious Wounds, Healing, Life; **ring**: Open Wounds; **rod**: Curing, Healing; **staff**: Cure Light Wounds (-20), Curing, Healing, Holiness |
| `STUN` | stunning | 50: Stun; 150: Heavy Stun; 10000: Knocked Out | PROT\_STUN | - | **Druid spell**: Herbal Curing; **Paladin spell**: Healing, Minor Healing (-20); **Priest spell**: Healing, Holy Word, Minor Healing (-20); **Ranger spell**: Herbal Curing; **activation**: CURE\_BODY, CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_NONORLYBIG, CURE\_TEMP, HEAL3, STAFF\_HOLY; **mushroom**: Fast Recovery; **potion**: \*Healing\*, Cure Critical Wounds, Healing, Life; **rod**: Curing, Healing; **shape**: Pukel-man; **staff**: Curing, Healing, Holiness |
| `FOOD` | nourishment | 1: Starving; 4: Faint; 8: Weak; 15: Hungry; 90: Fed; 100: Full | - | lower bound 1 | none |
| `PROTEVIL` | protection from evil | 10000: ProtEvil | - | - | none |
| `INVULN` | invulnerability | 10000: Invuln | - | - | none |
| `HERO` | heroism | 10000: Hero | - | gives the PROT\_FEAR flag (plus other effects) | none |
| `SHERO` | berserk rage | 10000: Berserk | - | gives the PROT\_FEAR flag (plus other effects) | none |
| `SHIELD` | mystic shield | 10000: Shield | - | - | none |
| `BLESSED` | your AC and to-hit bonus | 10000: Blssd | - | - | none |
| `SINVIS` | see invisible | 10000: SInvis | - | same as the SEE\_INVIS flag | none |
| `SINFRA` | enhanced infravision | 10000: Infra | - | - | none |
| `OPP_ACID` | acid resistance | 10000: RAcid | vulnerable acid | grants resist acid | none |
| `OPP_ELEC` | electricity resistance | 10000: RElec | vulnerable lightning | grants resist lightning | none |
| `OPP_FIRE` | fire resistance | 10000: RFire | vulnerable fire | grants resist fire | none |
| `OPP_COLD` | cold resistance | 10000: RCold | vulnerable cold | grants resist cold | none |
| `OPP_POIS` | poison resistance | 10000: RPois | vulnerable poison | grants resist poison | none |
| `OPP_CONF` | confusion resistance | 10000: RConf | - | same as the PROT\_CONF flag | none |
| `AMNESIA` | amnesia | 10000: Amnesiac | - | - | **Paladin spell**: Healing; **Priest spell**: Healing; **activation**: CURE\_CRITICAL, CURE\_FULL, CURE\_FULL2, CURE\_NONORLYBIG, STAFF\_HOLY; **potion**: \*Healing\*, Cure Critical Wounds, Healing, Life; **staff**: Healing, Holiness |
| `TELEPATHY` | telepathy | 10000: ESP | - | same as the TELEPATHY flag | none |
| `STONESKIN` | stone skin | 10000: Stone | - | - | none |
| `TERROR` | terror | 10000: Terror | - | gives the AFRAID flag (plus other effects) | **activation**: STAFF\_HOLY; **staff**: Holiness |
| `SPRINT` | sprinting | 10000: Sprint | - | on end: +100 turns of slowness (unresistable) | none |
| `BOLD` | fearlessness | 10000: Bold | - | same as the PROT\_FEAR flag | none |
| `SCRAMBLE` | scrambled | 10000: Scrambled | resist nexus | on start: scramble stats; on end: unscramble stats | none |
| `TRAPSAFE` | safety from traps | 10000: TrapSafe | - | same as the TRAP\_IMMUNE flag | none |
| `FASTCAST` | mana channelling | 10000: FastCast | - | - | none |
| `ATT_ACID` | temporary acid brand | 10000: AttAcid | - | brand ACID\_3 | none |
| `ATT_ELEC` | temporary lightning brand | 10000: AttElec | - | brand ELEC\_3 | none |
| `ATT_FIRE` | temporary fire brand | 10000: AttFire | - | brand FIRE\_3 | none |
| `ATT_COLD` | temporary cold brand | 10000: AttCold | - | brand COLD\_3 | none |
| `ATT_POIS` | temporary poison brand | 10000: AttPois | - | brand POIS\_3 | none |
| `ATT_CONF` | temporary monster confusion | 10000: AttConf | - | - | none |
| `ATT_EVIL` | temporary slay evil | 10000: AttEvil | - | slay EVIL\_2 | none |
| `ATT_DEMON` | temporary slay demon | 10000: AttDemon | - | slay DEMON\_5 | none |
| `ATT_VAMP` | temporary hitpoint siphoning | 10000: AttVamp | - | - | none |
| `HEAL` | turn-by-turn healing through increased metabolism | 10000: Heal | - | - | none |
| `COMMAND` | commanding a monster | 10000: Cmd | - | - | none |
| `ATT_RUN` | escape on attack | 10000: HitRun | - | - | none |
| `COVERTRACKS` | leave no scent trail, reduce visibility | 10000: NoTrack | - | - | none |
| `POWERSHOT` | piercing shot | 10000: PShot | - | - | none |
| `TAUNT` | taunting | 10000: Taunt | - | - | none |
| `BLOODLUST` | desire to kill | 10: Bloodlust; 18: Bloodlust; 26: Bloodlust; 34: Bloodlust; 45: Bloodlust; 50: Bloodlust | - | - | none |
| `BLACKBREATH` | black breath | 10: BlkBrth | - | - | **Druid spell**: Herbal Curing; **Ranger spell**: Herbal Curing |
| `STEALTH` | stealth | 10000: Stealth | - | - | none |
| `FREE_ACT` | free action | 10000: FrAct | - | same as the FREE\_ACT flag | none |

## Player properties

From `player_property.txt`: race/class abilities (`player`), object flags as they appear on the character sheet (`object`) and element templates (`element`).

| Type | Code | Name | Description |
|---|---|---|---|
| player | `ROCK` | Rock | You are made of rock. |
| player | `KNOW_MUSHROOM` | Mushroom Lore | You easily recognize mushrooms. |
| player | `KNOW_ZAPPER` | Device Lore | You easily recognize magic devices. |
| player | `SEE_ORE` | Miner | You can sense ore in the walls. |
| player | `FAST_SHOT` | Extra Shots | Your shooting speed improves every 3 levels. |
| player | `BRAVERY_30` | Relentless \[30\] | You become immune to fear at level 30. |
| player | `BLESS_WEAPON` | Blessed Fighter | You get improved melee with blessed or hafted weapons. |
| player | `ZERO_FAIL` | Full Spellcaster | You may obtain a perfect success rate with magic. |
| player | `BEAM` | Extra Spell Beaming | Your bolt spells frequently turn into beams. |
| player | `CHOOSE_SPELLS` | Spell Choice | You may choose your own spells to study. |
| player | `NO_MANA` | No Magic | You cannot cast spells. |
| player | `CHARM` | Charms | You are extra persuasive to monsters. |
| player | `UNLIGHT` | Unlight | You gain stealth in, can see in, and resist the dark. |
| player | `STEAL` | Pickpocketing | You can steal from monsters. |
| player | `SHIELD_BASH` | Shield Bash | You can bash monsters with a shield in melee. |
| player | `EVIL` | Evil | You resist nether, but are hurt by holy attacks. |
| player | `COMBAT_REGEN` | Combat Regeneration | You draw power from the thrill of combat, represented by Spell Points (SP). You gain SP when damaged by an enemy or when you attack in melee. As your blood cools your power fades; you lose SP at half the rate other classes gain them. When you spend SP by casting a spell you regain some health, and fading SP replenish your health more efficiently. The more damaged you are, the bigger these health gains will be. |
| object | `SUST_STR` | Sustain Strength | Your strength is sustained. |
| object | `SUST_INT` | Sustain Intelligence | Your intelligence is sustained. |
| object | `SUST_WIS` | Sustain Wisdom | Your wisdom is sustained. |
| object | `SUST_DEX` | Sustain Dexterity | Your dexterity is sustained. |
| object | `SUST_CON` | Sustain Constitution | Your constitution is sustained. |
| object | `PROT_FEAR` | Fear Immunity | You are immune to fear. |
| object | `PROT_BLIND` | Blindness Immunity | You cannot be blinded. |
| object | `PROT_CONF` | Confusion Immunity | You cannot be confused. |
| object | `PROT_STUN` | Stun Immunity | You cannot be stunned. |
| object | `SLOW_DIGEST` | Slow Digestion | You have a slow metabolism. |
| object | `FEATHER` | Feather Falling | You fall lightly. |
| object | `REGEN` | Regeneration | You regenerate quickly. |
| object | `TELEPATHY` | Telepathy | You have telepathy. |
| object | `SEE_INVIS` | See Invisible | You can see invisible creatures. |
| object | `FREE_ACT` | Free Action | You are immune to paralysis. |
| object | `HOLD_LIFE` | Hold Life | Your life force is sustained. |
| object | `IMPACT` | Earthquakes | You sometimes create earthquakes on impact. |
| object | `IMPAIR_HP` | Slow Regeneration | Your hitpoint recovery is impaired. |
| object | `IMPAIR_MANA` | Slow Mana Regeneration | Your mana recovery is impaired. |
| object | `AFRAID` | Constant Fear | You are afraid of melee, and bad at shooting and casting spells. |
| object | `NO_TELEPORT` | Teleport Ban | You cannot teleport. |
| object | `AGGRAVATE` | Aggravate | You aggravate creatures nearby. |
| object | `DRAIN_EXP` | Experience Drain | Your experience constantly drains away. |
| object | `TRAP_IMMUNE` | Trap Immune | You are immune to traps. |
| element |  | Resistance | You resist |
| element |  | Immunity | You are immune to |
| element |  | Vulnerability | You are vulnerable to |

## Object properties and runes

From `object_property.txt`. In 4.2 every property is a **rune**: once you learn a rune (by noticing it in use, by an Identify effect, or as listed under *Learned*), it is recognised on every object. *Learned* (`id-type`): `on wield` = known as soon as the item is wielded, `on effect` = when it does something, `timed` = after wearing it for a while. *Power* is the weight in object power/value calculations.

| Type | Code | Name | Subtype | Learned | Power | Description / message |
|---|---|---|---|---|---:|---|
| stat | `STR` | strength | - | - | 9 | - |
| stat | `INT` | intelligence | - | - | 5 | - |
| stat | `WIS` | wisdom | - | - | 5 | - |
| stat | `DEX` | dexterity | - | - | 8 | - |
| stat | `CON` | constitution | - | - | 12 | - |
| mod | `STEALTH` | stealth | - | - | 8 | - |
| mod | `SEARCH` | searching skill | - | - | 2 | - |
| mod | `INFRA` | infravision | - | - | 4 | - |
| mod | `TUNNEL` | tunneling | - | - | 3 | - |
| mod | `SPEED` | speed | - | - | 20 | - |
| mod | `DAM_RED` | damage reduction | - | - | 5 | - |
| mod | `BLOWS` | extra blows | - | - | 0 | - |
| mod | `SHOTS` | shooting speed | - | - | 0 | - |
| mod | `MIGHT` | shooting power | - | - | 0 | - |
| mod | `MOVES` | extra moves | - | - | 0 | - |
| mod | `LIGHT` | light | - | - | 3 | - |
| flag | `SUST_STR` | sustain strength | sustain | on effect | 9 | Your {name} glows. |
| flag | `SUST_INT` | sustain intelligence | sustain | on effect | 4 | Your {name} glows. |
| flag | `SUST_WIS` | sustain wisdom | sustain | on effect | 4 | Your {name} glows. |
| flag | `SUST_DEX` | sustain dexterity | sustain | on effect | 7 | Your {name} glows. |
| flag | `SUST_CON` | sustain constitution | sustain | on effect | 8 | Your {name} glows. |
| flag | `PROT_FEAR` | protection from fear | protection | on effect | 6 | fear |
| flag | `PROT_BLIND` | protection from blindness | protection | on effect | 16 | blindness |
| flag | `PROT_CONF` | protection from confusion | protection | on effect | 24 | confusion |
| flag | `PROT_STUN` | protection from stunning | protection | on effect | 12 | stunning |
| flag | `SLOW_DIGEST` | slow digestion | misc ability | timed | 2 | Slows your metabolism |
| flag | `FEATHER` | feather falling | misc ability | on effect | 1 | Feather Falling |
| flag | `REGEN` | regeneration | misc ability | timed | 5 | Speeds regeneration |
| flag | `TELEPATHY` | telepathy | misc ability | on wield | 35 | Grants telepathy |
| flag | `SEE_INVIS` | see invisible | misc ability | on wield | 6 | Grants the ability to see invisible things |
| flag | `FREE_ACT` | free action | misc ability | on effect | 8 | Prevents paralysis |
| flag | `HOLD_LIFE` | hold life | misc ability | on effect | 5 | Sustains your life force |
| flag | `IMPACT` | earthquakes | melee | on effect | 10 | Sometimes creates earthquakes on impact |
| flag | `BLESSED` | blessed melee | melee | on wield | 1 | Blessed by the Valar (combat bonuses for holy casters) |
| flag | `BURNS_OUT` | burns out | light | on wield | 0 | - |
| flag | `TAKES_FUEL` | takes fuel | light | on wield | 0 | - |
| flag | `NO_FUEL` | no fuel | light | on wield | 5 | - |
| flag | `IMPAIR_HP` | impaired hitpoint recovery | bad | timed | -8 | Impairs hitpoint recovery |
| flag | `IMPAIR_MANA` | impaired mana recovery | bad | timed | -8 | Impairs mana recovery |
| flag | `AFRAID` | constant fear | bad | on wield | -20 | Makes you afraid of melee, and worse at shooting and casting spells |
| flag | `NO_TELEPORT` | teleportation ban | bad | on effect | -20 | Prevents teleportation |
| flag | `AGGRAVATE` | aggravation | bad | timed | -20 | Aggravates creatures nearby |
| flag | `DRAIN_EXP` | experience drain | bad | timed | -5 | Drains experience |
| flag | `STICKY` | stuck on | bad | on wield | -5 | Can't be removed |
| flag | `FRAGILE` | fragile | bad | on wield | -1 | Can be destroyed if you attempt to remove its curses |
| flag | `LIGHT_2` | intensity 2 light | light | on wield | 9 | - |
| flag | `LIGHT_3` | intensity 3 light | light | on wield | 18 | - |
| flag | `DIG_1` | power 1 digging | dig | on wield | 3 | - |
| flag | `DIG_2` | power 2 digging | dig | on wield | 6 | - |
| flag | `DIG_3` | power 3 digging | dig | on wield | 9 | - |
| flag | `EXPLODE` | explode | light | on wield | 0 | - |
| flag | `TRAP_IMMUNE` | trap immunity | misc ability | on effect | 5 | Gives immunity to traps |
| flag | `THROWING` | throwing | throw | on wield | 4 | - |
| flag | `MULTIPLY_WEIGHT` | multiply weight | curse-only | on wield | 0 | - |
| ignore | `ACID` | ignore acid | - | - | 3 | - |
| ignore | `ELEC` | ignore electricity | - | - | 1 | - |
| ignore | `FIRE` | ignore fire | - | - | 3 | - |
| ignore | `COLD` | ignore cold | - | - | 1 | - |
| resistance | `ACID` | acid resistance | - | - | 5 | - |
| resistance | `ELEC` | electricity resistance | - | - | 6 | - |
| resistance | `FIRE` | fire resistance | - | - | 6 | - |
| resistance | `COLD` | cold resistance | - | - | 6 | - |
| resistance | `POIS` | poison resistance | - | - | 28 | - |
| resistance | `LIGHT` | light resistance | - | - | 6 | - |
| resistance | `DARK` | dark resistance | - | - | 16 | - |
| resistance | `SOUND` | sound resistance | - | - | 14 | - |
| resistance | `SHARD` | shards resistance | - | - | 8 | - |
| resistance | `NEXUS` | nexus resistance | - | - | 15 | - |
| resistance | `NETHER` | nether resistance | - | - | 20 | - |
| resistance | `CHAOS` | chaos resistance | - | - | 20 | - |
| resistance | `DISEN` | disenchantment resistance | - | - | 20 | - |
| vulnerability | `ACID` | acid vulnerability | - | - | -6 | - |
| vulnerability | `ELEC` | lightning vulnerability | - | - | -6 | - |
| vulnerability | `FIRE` | fire vulnerability | - | - | -6 | - |
| vulnerability | `COLD` | cold vulnerability | - | - | -6 | - |
| immunity | `ACID` | acid immunity | - | - | 43 | - |
| immunity | `ELEC` | lightning immunity | - | - | 41 | - |
| immunity | `FIRE` | fire immunity | - | - | 46 | - |
| immunity | `COLD` | cold immunity | - | - | 43 | - |

### Slays

| Code | Slays | Monster flag/base | Multiplier |
|---|---|---|---:|
| `EVIL_2` | evil creatures | EVIL | x2 |
| `ANIMAL_2` | animals | ANIMAL | x2 |
| `ORC_3` | orcs | ORC | x3 |
| `TROLL_3` | trolls | TROLL | x3 |
| `GIANT_3` | giants | GIANT | x3 |
| `DEMON_3` | demons | DEMON | x3 |
| `DRAGON_3` | dragons | DRAGON | x3 |
| `UNDEAD_3` | undead | UNDEAD | x3 |
| `DEMON_5` | demons | DEMON | x5 |
| `DRAGON_5` | dragons | DRAGON | x5 |
| `UNDEAD_5` | undead | UNDEAD | x5 |

### Brands

| Code | Brand | Verb | Multiplier | Resisted by | Extra vs |
|---|---|---|---:|---|---|
| `ACID_3` | acid | dissolve | x3 | IM\_ACID | - |
| `ELEC_3` | lightning | shock | x3 | IM\_ELEC | - |
| `FIRE_3` | fire | burn | x3 | IM\_FIRE | HURT\_FIRE |
| `COLD_3` | cold | freeze | x3 | IM\_COLD | HURT\_COLD |
| `POIS_3` | poison | poison | x3 | IM\_POIS | - |
| `ACID_2` | acid | corrode | x2 | IM\_ACID | - |
| `ELEC_2` | lightning | zap | x2 | IM\_ELEC | - |
| `FIRE_2` | fire | singe | x2 | IM\_FIRE | HURT\_FIRE |
| `COLD_2` | cold | chill | x2 | IM\_COLD | HURT\_COLD |
| `POIS_2` | poison | sicken | x2 | IM\_POIS | - |

Rune categories: combat runes (to-hit, to-dam, AC bonuses), the modifiers and stats above, each resistance/immunity/vulnerability, each flag, each slay and brand, and each curse (curses are runes too, learned when the curse is noticed). See [Chapter 6](../06-equipment.md) for identification by use.

# The Angband Strategy Manual

*A strategy guide to upstream Angband (the C game, 4.2.6 plus master at `e473720ed`),
grounded in the game's source code. It is for players and for automatic players (borgs).*

The [official user manual](https://angband.readthedocs.io/en/latest/) explains the
commands and screens. This manual explains how the game works underneath and how to
survive and win. Use it to look things up:

- which spell to learn or cast;
- what a monster will do to you;
- what equipment to buy, and what to have at each stage;
- how to explore a level, and when to leave it;
- what to do when things go wrong.

Every number comes from the game's data (`lib/gamedata/`) or code (`src/`), with a
citation so you can check it. [Chapter 0](00-sources-and-conventions.md) gives the
version, the sources (including the official manual, a copy of which is in
[`user-manual/`](user-manual/)) and the conventions.

## How to use this manual

| You want to… | Read |
|---|---|
| Start a first character | [02 Character creation](02-character-creation.md), then the first phase of [12 Game plan](12-game-plan-by-phase.md) and your class in [14 Class playbooks](14-class-playbooks.md) |
| Decide which spell to learn or cast | [03 Magic and spells](03-magic-and-spells.md), and [A7 Class spells](appendices/A7-class-spells.md) for the full tables |
| Know what a monster can do | [08 Monsters](08-monsters.md), then [A1](appendices/A1-monsters-by-depth.md), [A2](appendices/A2-uniques.md), [A3](appendices/A3-monster-spells.md) |
| Decide whether you are ready to go deeper | [12 Game plan](12-game-plan-by-phase.md) and the resistance checklist in [05](05-elements-and-status.md) |
| Choose equipment, or decide what to buy | [06 Equipment](06-equipment.md), [07 Consumables and devices](07-consumables-and-devices.md), [11 Town and shopping](11-town-and-shopping.md) |
| Explore a level well | [10 Exploration](10-exploration.md) and [09 Dungeon](09-dungeon.md) |
| Survive an emergency | [13 Escapes and emergencies](13-escapes-and-emergencies.md) |
| Understand a formula | [01 Core mechanics](01-core-mechanics.md) and [04 Combat](04-combat.md) |
| Build an automatic player | [15 Notes for the borg](15-notes-for-the-borg.md) |
| Experiment | [16 Things to try](16-things-to-try.md) |

## Contents

### Chapters

| # | Chapter | Contents |
|---|---|---|
| 0 | [Sources and conventions](00-sources-and-conventions.md) | Version, official manual map, source citations, conventions |
| 1 | [Core mechanics](01-core-mechanics.md) | Energy and speed, to-hit, AC, criticals, blows, stats, skills, stealth, HP, regeneration, experience, food, light, failure rates, saving throw |
| 2 | [Character creation](02-character-creation.md) | Races, classes, stats, point-buy, birth options, recommended combinations |
| 3 | [Magic and spells](03-magic-and-spells.md) | Realms, learning, failure, mana; every spell of every class with advice on when to learn and cast it; a cross-class spell lookup |
| 4 | [Combat](04-combat.md) | Melee, missiles, throwing, slays and brands, weapon choice, tactics, deciding whether a fight can be won |
| 5 | [Elements and status effects](05-elements-and-status.md) | Elements and resistances, breath damage, timed effects and their cures, drains, curses, traps, resistance checklist by depth |
| 6 | [Equipment](06-equipment.md) | Every slot, egos, artifacts, runes and identification, curses, the kit to have at each stage, swap items |
| 7 | [Consumables and devices](07-consumables-and-devices.md) | Potions, scrolls, food, staves, wands, rods, activations, device skill, standard carry kits |
| 8 | [Monsters](08-monsters.md) | Monster recall, AI, families by symbol, killers by depth, uniques up to Sauron and Morgoth |
| 9 | [The dungeon](09-dungeon.md) | Level generation, profiles, rooms, vaults, pits and nests, feelings, stairs, recall, terrain, quests |
| 10 | [Exploration](10-exploration.md) | Procedures for a new level, detection, autoexplore, diving rate, stairs, when to leave |
| 11 | [Town and shopping](11-town-and-shopping.md) | Stores and stock, prices, selling, the home, shopping lists by class and phase |
| 12 | [Game plan by phase](12-game-plan-by-phase.md) | From town to Morgoth: goals, kit, threats and readiness tests for each phase |
| 13 | [Escapes and emergencies](13-escapes-and-emergencies.md) | Every escape and how it really works, emergency procedures, escape budgets |
| 14 | [Class playbooks](14-class-playbooks.md) | A deep guide to each class |
| 15 | [Notes for the borg](15-notes-for-the-borg.md) | Decision rules and thresholds for an automatic player |
| 16 | [Things to try](16-things-to-try.md) | Untested strategies, with test protocols |

### Appendices (generated from `lib/gamedata` by [`tools/gen_appendices.py`](tools/gen_appendices.py))

| | Appendix | Contents |
|---|---|---|
| A1 | [Monsters by depth](appendices/A1-monsters-by-depth.md) | Every non-unique monster: speed, HP, AC, blows, spells with maximum damage, immunities, behaviour |
| A2 | [Uniques](appendices/A2-uniques.md) | One card per unique |
| A3 | [Monster spells](appendices/A3-monster-spells.md) | Every monster spell, the breath table, the deadliest breathers, the depths where threats first appear |
| A4 | [Artifacts](appendices/A4-artifacts.md) | Every standard artifact |
| A5 | [Ego items](appendices/A5-ego-items.md) | Every ego type |
| A6 | [Object kinds](appendices/A6-object-kinds.md) | Every base object and its effect |
| A7 | [Class spells](appendices/A7-class-spells.md) | Every spell of every class: level, mana, fail, effect |
| A8 | [Races and classes](appendices/A8-races-and-classes.md) | Stats, skills, abilities, the experience table |
| A9 | [Stores](appendices/A9-stores.md) | Store stock and price rules |
| A10 | [Curses, traps and timed effects](appendices/A10-curses-traps-timed-effects.md) | Curses, traps, timed effects and their cures, properties, runes |

### Official user manual (copy)

[`user-manual/`](user-manual/) holds the reStructuredText source of
<https://angband.readthedocs.io/en/latest/> and the in-game help files, both from the
same commit. They are included so that the game version and its official manual can be
read together. The text and its licence belong to the Angband developers (see
[`user-manual/copying.rst`](user-manual/copying.rst)).

## Rules that keep characters alive

Each rule is explained, with the evidence for it, in the chapter linked.

1. **Speed matters more than anything else.** At +10 you act twice for every turn a
   normal-speed monster takes, and a +10 monster acts twice for each of yours.
   ([01](01-core-mechanics.md))
2. **Get Free Action before the 20s, then See Invisible.** Paralysis without Free
   Action kills characters of every level. ([05](05-elements-and-status.md))
3. **Carry escapes that work when you are blind or confused.** You cannot read scrolls
   then, but you can quaff potions and use staffs and rods.
   ([13](13-escapes-and-emergencies.md))
4. **Breath damage comes from the monster's current HP.** Hurt a breather before it
   breathes, or break line of sight. ([05](05-elements-and-status.md),
   [A3](appendices/A3-monster-spells.md))
5. **Look up a unique before you fight it.** If its worst turn does more than half your
   HP, leave. ([08](08-monsters.md), [A2](appendices/A2-uniques.md))
6. **Fight in corridors, not in the open.** A corridor limits how many monsters can
   reach you and how many can breathe on you. ([04](04-combat.md))
7. **Leaving a level is free.** Take the stairs for a bad feeling, an out-of-depth
   unique, a hound pack or a breeder explosion. ([10](10-exploration.md))
8. **Decide your escape thresholds in advance, then act on them.** Choose the HP at
   which you heal, phase, teleport or leave, and do it when you reach it.
   ([13](13-escapes-and-emergencies.md))
9. **Get resistances in order.** The base four, then poison, then confusion and
   blindness, then the high elements, each by the depth that needs it.
   ([05](05-elements-and-status.md))
10. **Use and wear unknown items to learn their runes.** Identification in 4.2 is
    knowledge you build up by playing. ([06](06-equipment.md))

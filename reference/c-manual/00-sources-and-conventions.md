# 0. Sources and conventions

This manual is a strategy guide to Angband. It sits on top of the official user manual and
the game's source code. This chapter says which version it describes, what it was built
from, and how to read its citations, numbers and advice.

- [The game version](#the-game-version)
- [The official user manual (required companion)](#the-official-user-manual-required-companion)
- [The game source](#the-game-source)
- [Other sources and how they were used](#other-sources-and-how-they-were-used)
- [Conventions](#conventions)
- [Keeping this manual current](#keeping-this-manual-current)

## The game version

The manual describes **upstream Angband (the C game)** at
[angband/angband `e473720ed`](https://github.com/angband/angband/commit/e473720ed88a0f0c7ca66815cef3000eb4a38bc3).
That commit is Angband 4.2.6 plus the later master changes. This repository mirrors it
on the branch
[`c-borg-reference`](https://github.com/diegoami/neo-angband-tracker/tree/c-borg-reference).
The [C Borg documents](../c-borg/README.md) read the same commit.

The 4.2.6 release tree in this repository's `reference/` directory is **not** the same
game. The master commit changes 18 of the files in `lib/gamedata` and adds or removes
none:

- 16 change the rules: `activation.txt`, `artifact.txt`, `chest_trap.txt`, `class.txt`,
  `curse.txt`, `ego_item.txt`, `monster.txt`, `monster_base.txt`, `object.txt`,
  `object_property.txt`, `player_property.txt`, `player_timed.txt`,
  `room_template.txt`, `terrain.txt`, `trap.txt` and `vault.txt`.
- `ui_knowledge.txt` adds a "Not Fully Known" monster category to the knowledge menus.
- `history.txt` changes only the wording of character histories.

The master commit also adds features: the autoexplore commands
(`autoexplore_commands` in `src/list-options.h`) and doors and rubble that are cleared
automatically when you follow a path. When a number here disagrees with a 4.2.x
release, check which version you are playing.

Older guides, wikis and spoilers describe 3.x or 4.1, and many of their numbers are wrong
for 4.2. Examples:

- In 4.2 there is no searching skill.
- Identification works by learning runes.
- The classes are different: Necromancer and Blackguard were added in 4.2.

Be suspicious of any figure that is not cited to the source.

## The official user manual (required companion)

The official manual is at **<https://angband.readthedocs.io/en/latest/>**. It covers how to
play: the commands, the screens and the options. This strategy manual assumes you have
it, or at least know where it is. A copy of its source, from the same commit, is
included in [`user-manual/`](user-manual/) so you can read it offline. The in-game help
files from `lib/help/` are copied there too.

| Official page | What it covers | Where this manual extends it |
|---|---|---|
| [A Quick Demo](https://angband.readthedocs.io/en/latest/a-quick-demo.html) ([local](user-manual/a-quick-demo.rst)) | First character and first steps | [12 Game plan](12-game-plan-by-phase.md), [10 Exploration](10-exploration.md) |
| [A Players' Guide](https://angband.readthedocs.io/en/latest/guide.html) ([local](user-manual/guide.rst)) | Hints and advice, written for 3.5 and partly out of date (the page says so itself) | Everything here, re-checked against 4.2 source |
| [FAQ](https://angband.readthedocs.io/en/latest/faq.html) ([local](user-manual/faq.rst)) | Common questions | — |
| [Creating a Character](https://angband.readthedocs.io/en/latest/birth.html) ([local](user-manual/birth.rst)) | Races, classes, stats, birth options | [02 Character creation](02-character-creation.md), [14 Class playbooks](14-class-playbooks.md), [A8](appendices/A8-races-and-classes.md) |
| [Exploring the Dungeon](https://angband.readthedocs.io/en/latest/dungeon.html) ([local](user-manual/dungeon.rst)) | Map symbols, town, stores, dungeon features | [09 Dungeon](09-dungeon.md), [11 Town and shopping](11-town-and-shopping.md), [A9](appendices/A9-stores.md) |
| [Attacking and Being Attacked](https://angband.readthedocs.io/en/latest/attack.html) ([local](user-manual/attack.rst)) | Combat, monster attacks, resistances, status effects | [01 Mechanics](01-core-mechanics.md), [04 Combat](04-combat.md), [05 Elements and status](05-elements-and-status.md) |
| [Playing the Game](https://angband.readthedocs.io/en/latest/playing.html) ([local](user-manual/playing.rst)) | The screen, objects, identification, inscriptions | [06 Equipment](06-equipment.md), [07 Consumables](07-consumables-and-devices.md) |
| [List of Commands](https://angband.readthedocs.io/en/latest/command.html) ([local](user-manual/command.rst)) | Every command in both keysets | Used throughout. Exploration commands are in [10](10-exploration.md) |
| [Options](https://angband.readthedocs.io/en/latest/option.html) ([local](user-manual/option.rst)) | Interface and birth options | Birth options in [02](02-character-creation.md) |
| [Customising the Game](https://angband.readthedocs.io/en/latest/customize.html) ([local](user-manual/customize.rst)) | Keymaps, visuals, pref files | Not covered (an interface topic, not strategy) |

The official manual tells you **what** you can do. This manual tells you **how the
game works underneath** and **what to do**. Where the two disagree, the source code
decides. Each chapter cites the code it relied on, so you can check.

## The game source

All references are to paths relative to the root of the game source:

- `lib/gamedata/*.txt`: the data. Monsters, objects, artifacts, egos, classes, races,
  spells, stores, traps, curses, timed effects, dungeon profiles, vaults. The appendices
  are generated from these files.
- `src/*.c`: the rules. Combat, spells, effects, monster AI, level generation, stores.
- `src/borg/`: the automatic player. It is treated as a source of ideas only.

A citation such as `src/player-calcs.c:calc_blows()` means: read that function to check
the claim. A citation such as `lib/gamedata/monster.txt` means: the value comes from that
data file.

## Other sources and how they were used

- **The earlier strategy draft on `c-borg-reference`**,
  [`docs/strategy/`](https://github.com/diegoami/neo-angband-tracker/tree/c-borg-reference/docs/strategy).
  An earlier session wrote it. It contains an index and six generated appendices (monsters,
  uniques, monster spells, artifacts, egos, objects), produced by its own
  `gen_appendices.py`. The planned chapters were never written. The generator here,
  [`tools/gen_appendices.py`](tools/gen_appendices.py), started from that script. It
  was extended (class spells, races and classes, stores, curses, traps, timed effects)
  and every parse was checked again against the data. The branch also has a single-file
  `docs/user-manual.md`. It is a consolidation of the official manual and is not used
  here: the official pages, cited above, are the reference.
- **The C Borg specification**, [`docs/c-borg/BORG_SPEC.md`](../c-borg/BORG_SPEC.md),
  and the [Tunable Borg Plan](../c-borg/TUNABLE-BORG-PLAN.md). They describe what the
  existing automatic player does: its depth-readiness rules, danger estimate and escape
  logic. These were used **as ideas, not directives**. The borg is not an authority on
  good play, and this manual says so where it departs from the borg.
- **Player community knowledge.** It was used only where the source confirmed it.

## Conventions

- **FACT and ADVICE.** A *fact* comes from the source and is cited. *Advice* is
  judgement: a recommendation, a threshold, a priority. Chapters keep the two apart. The
  sections headed **Things to try (untested)** go further: they are hypotheses that
  nobody has checked in play. [Chapter 16](16-things-to-try.md) collects the larger ideas
  and explains how to test them.
- **Abbreviations.**
  - DL: dungeon level. One DL is 50 feet, so DL10 is 500 ft.
  - CL: character level.
  - HP / SP: hit points / spell points (mana).
  - AC: armour class.
  - FA: Free Action.
  - SI: See Invisible.
  - rBase: resist fire, cold, acid and lightning.
  - rPois, rConf, rBlind, rNether and so on: resist poison, confusion, blindness,
    nether and so on.
  - ESP: telepathy.
  - CLW, CSW, CCW: Cure Light, Serious and Critical Wounds.
  - `@`: the player.
- **Speed.** It is written relative to normal: +10 means about twice as many actions as
  a normal-speed creature. The game's internal speed is 110 for normal; this manual
  writes +0.
- **Dice.** `XdY` is the sum of X dice with Y sides. `m_bonus` is the game's
  depth-scaled random bonus.
- **Monster damage.** "Max damage" is the most a single blow or spell can do. Breath
  damage depends on the monster's *current* HP (see
  [A3](appendices/A3-monster-spells.md)).
- **Borg notes.** Sections addressed to an automatic player say so.
  [Chapter 15](15-notes-for-the-borg.md) collects them as rules.

## Where the source contradicts common belief

Writing this manual turned up several places where the code differs from the official
manual, from older guides, or from its own comments. Each item is explained in the chapter
cited.

| Finding | Source | Chapter |
|---|---|---|
| Unspent point-buy points show as extra gold on the birth screen, but accepting the character resets gold to the flat starting amount. The official manual says unspent points become gold. | `src/player-birth.c:do_cmd_accept_character()` calls `get_money()` after `recalculate_stats()` | [02](02-character-creation.md) |
| The stat roller is `5 + 1d3 + 1d4 + 1d5` per stat, with the totals constrained, not 3d6. | `src/player-birth.c:get_stats()` | [02](02-character-creation.md) |
| Gloves do not hurt spellcasting. Only armour weight above the class limit reduces mana. | no glove code in `src/player-calcs.c:calc_mana()` | [03](03-magic-and-spells.md) |
| Player spell failure is capped at 50% before stun and amnesia are added. | `src/player-spell.c:spell_chance()` | [03](03-magic-and-spells.md) |
| Every monster that is not stupid fails its non-innate spells at a flat 24% (more if afraid or confused). The code takes `MIN(spell_power, 1)`, which looks like a typo for `MAX`. Breaths never fail. | `src/mon-attack.c:monster_spell_failrate()` | [08](08-monsters.md) |
| A light's number is its *intensity*. You see grids at distance less than the intensity, so a Wooden Torch (2) lights only the adjacent grids and a Lantern (3) lights radius 2. | `src/cave-view.c:update_view_one()` | [01](01-core-mechanics.md), [06](06-equipment.md) |
| Deep Descent goes five levels below your **maximum** depth, not your current depth. | `src/effect-handler-general.c:effect_handler_DEEP_DESCENT()` | [09](09-dungeon.md), [13](13-escapes-and-emergencies.md) |
| There is no search command. Secret doors are found by walking next to them. A trap is noticed when it comes into view if your searching skill is at least the trap's power. | `lib/gamedata/trap.txt`, `src/trap.c` | [01](01-core-mechanics.md), [10](10-exploration.md) |
| Only sleep and fear give monsters a saving throw. Slow, confuse, hold and stun are stopped only by the matching `NO_*` flag. | `src/mon-timed.c:does_resist()` | [03](03-magic-and-spells.md), [13](13-escapes-and-emergencies.md) |
| Black Market prices are three times normal: ×2, then ×150%. The source comment says "twice". | `src/store.c:price_item()` | [11](11-town-and-shopping.md) |
| The Necromancer's "stealth in the dark" from Unlight is described but not implemented. | `lib/gamedata/player_property.txt` and no matching code | [14](14-class-playbooks.md) |

These were found by reading the code. Where a finding may be a defect in the game rather
than a design choice, it should go through this repository's triage rules
([RULES](../../RULES.md)) before anyone calls it a bug.

## Keeping this manual current

The appendices are generated. After any change to `lib/gamedata/`, run this against an
up-to-date game checkout:

```bash
python3 docs/c-manual/tools/gen_appendices.py --src /path/to/angband
```

The chapters are written by hand. When the upstream source changes:

1. Grep the manual for the functions and data files it cites.
2. Re-check the cited claims.
3. Update the pinned commit at the top of this chapter.

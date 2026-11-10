# INSTRUCTIONS: Angband 4.2.6 for a player with one screen and ~150 decisions

What the game's manual and the strategy manual say, cut to what you use, in the order you use it. **What earlier runs found out is folded into this page**, and each of those lines names the runs that saw it as `run#ask`: the ask is in that run's own notebook under `knowledge/` and in its decisions under `records/`, so any line here can be traced to the screen it came from. What one model family found out about *itself* is in `agents/<family>/`, which is yours to read and to write back to.

**Depth 5 to 20 (breath, summoners, kit gates) is volume 2, `manuals/INSTRUCTIONS_2.md`; no run has tested it yet.**

Sources, so any line can be checked. `command`/`playing`/`attack`/`birth` = `vendor/angband/docs/<name>.rst`. `object`/`store`/`class`/`monster`/`trap` = `vendor/angband/lib/gamedata/<name>.txt`, and `race` = `vendor/angband/lib/gamedata/p_race.txt`. `src/x.c` = `vendor/angband/src/`. `c4`/`c14`/`c15` = strategy manual chapters at its pin (`git cat-file -p HEAD:docs/c-manual/<file>`). Words in double quotes are quoted from the source named beside them. Labels: **FACT** is in game data or source; **POLICY** is chapter 15's own "starting policy to be measured, not as truths", so following it and being hurt is a result worth reporting; **UNTESTED** means no run has tried it.

One caution about that chapter: it is *notes for the borg*, written for a program that reads the game's own variables. You read a screen, and the screen sometimes shows the reverse of what the program sees - the level feeling is the known case. A rule of its that mentions a number you cannot see, or can only see transformed, is not a rule you can follow; check against `src/` before you trust it.

Original keyset: digits walk (`1-9`, `5` stays), letters are commands. Keys you send are not interpreted: 80x24 screen, game turns advance in tens.

## The one thing to take from this page

**`lxg` explores and `l>g` descends. Use them instead of walking.**

- `lxg` — look, jump the cursor to the nearest square beside unexplored space, walk there. The game pathfinds; one ask covers what a dozen digit-presses would.
- `l>g` — the same to the nearest **known** down staircase. Then `>` takes it.
- `l<g` — the same to a known up staircase.
- Send each **alone in its ask**: any key after `g` interrupts the walk after one step, so `lxglxglxg` gives one-square hops, not three walks.
- With no staircase known yet, `>` in look mode does nothing (the game rings a bell) and the cursor stays put. Keep exploring with `lxg` until one is on the map.

OBSERVED, sonnet-2.2#156 to sonnet-2.2#230: this is how the only deliberate descent in any run happened — depth 6 to depth 7, after four runs and some 350 asks had never found a staircase at all. One ask walked 11 steps and 110 game turns onto the stairs.

Two runs went wrong here in ways worth not repeating. One spent ~100 asks walking a level blind and found nothing. One wrote itself a 143-line pathfinder, used it for 148 asks, reached depth 1, and never tried `lxg` — the game already had what it was building. **Check the game's own commands before you write code.**

## Keys as the game receives them

The keys of one ask are fed to the game without anything looking at the screen
in between, so every string is a bet that nothing interrupts it. This section
is what that bet has cost earlier runs. Each line says which runs saw it as
`run#ask`, and that ask is in the run's own notebook under `knowledge/` and in
its decisions under `records/`.

- **A `-more-` swallows the next key** (3 runs: sonnet-1#21, sonnet-2#16,
  sonnet-3#32). A command typed behind one loses its first key and the rest is
  misread. A space clears a `-more-` and is harmless otherwise (sonnet-2#20,
  sonnet-3#24), so end a string with spaces (`va'   `). A level change ends in
  one (sonnet-1#68, sonnet-2#26, sonnet-3#31): take stairs with `> `, not `>`
  - `<>` in one ask left a run standing in the town.
- **Menu letters are labels, not positions.** Store rows came up
  `a c f h j m n o q r u v y z` (3 runs: sonnet-1#25, sonnet-2#13,
  sonnet-3#22); the pack skips `h j k l`, so with eight items its fifth row is
  `e` and its eighth `i` (sonnet-1#57, sonnet-2#119); and the throw prompt has
  letters of its own, where a flask of oil was `a` and `b` or `c` in the pack
  (sonnet-2#40, sonnet-3#41). Read the letter off the screen, press it, and
  check the cursor moved to the row you meant - the header of every screen
  gives you `cursor at x,y` for exactly this.
- **A whole purchase fits one ask** (2 runs: sonnet-2#20, sonnet-3#16; the
  sequence itself in 3, sonnet-1#16): `p`, the row letter, the quantity as
  digits, Enter, then any
  key but Escape. `ph10\ny ` bought ten flasks of oil. "One purchase per ask"
  was an early wrong note.
- **Confused, blind, afraid or slowed: one or two keys per ask, never a string
  of waits** (sonnet-3#46, sonnet-3#47 - the run died there). It sent sixteen
  hold keys in one ask while confused; a wounded kobold it believed dead
  arrived about seven turns in and killed it from full health, with five Cure
  Light Wounds in the pack. The game flushes input on a disturbance, and what
  you sent is still in the buffer afterwards.
- **A digit string stops at the first wall and the rest is dropped**
  (sonnet-1#96, minimaxflash-1#14, which measured where the wall was). In
  caves `.` plus a digit stopped every two or three squares, where
  `4444444444` went further (sonnet-1#114) - but see the top of this page:
  neither is how to cross a level.
- **`l` with monsters on the screen answered only "You are on an open
  floor"** (sonnet-1#40, sonnet-2#58): the cursor starts on you. Both runs
  attacked or threw to find out what a letter was. Cycling with space, or `[`,
  is UNTESTED.
- **An unknown potion or scroll pays experience the first time it is used, and
  the payment shrinks fast.** FACT (`src/obj-knowledge.c`): the payment is
  `(kind level + your level / 2) / your level`, rounded down - at character
  level 1 that is the kind's own level, and by level 5 it is usually nothing.
  OBSERVED (sonnet-1#45, sonnet-1#57, sonnet-2#77, sonnet-2#110,
  sonnet-2#151): Blessing at CL1 moved `NXT` by one, Deep Descent at CL4 paid
  3, Cure Serious Wounds at CL5 paid 1, Blessing at CL6 nothing, and one
  unknown potion at CL1 was worth a whole character level. So try unknowns at
  CL1-3 and keep them for need after that: at CL5 one run drank an unknown at
  89/89 hit points and spent a Cure Serious Wounds on nothing (sonnet-2#109).
  Bought scrolls and potions are already named and pay nothing (3 runs:
  sonnet-1#57, sonnet-2#19, sonnet-3#25).
- **Try one at full hit points, with nothing in view, standing on a staircase
  you could take** - POLICY, from c15 ESC-01, which puts an unknown potion on
  the last rung of the escape ladder. Which kinds can hurt you is section 6b's
  `cost:0` rule; the complete list of potions and scrolls is appendix A6 of
  the strategy manual, to be looked up when you meet one, not read through.
- **Never send a real newline as a key.** Send the two-character escape
  `\n`; the frontend drops a *trailing* newline byte on purpose, because that
  is the one a text editor adds. From a shell that means `"\n"` in double
  quotes, not `$'\n'`. With the Enter lost, the prompt stays open and
  everything after it is eaten (minimaxflash-2#72, five asks; the tool now
  translates it for you and says so).
- **Look or targeting mode can be left standing**, and then it eats the first
  keys of the next ask. The tell is `Press '?' for help.` on the bottom line,
  or a look description in the message line with a turn counter that has not
  moved. Press `\e` first (minimaxflash-2#72, minimaxflash-1.2#221).
- **A key after a movement command is swallowed by whatever it opened.** A
  walk, a run or an aimed `g` leaves the game ready for the next command, and
  the first key of the next string is read as one: a stray `a` became "Aim a
  wand" and ate the rest five times in one run (minimaxflash-2#91). The rule
  that covers every case of this: **never send two commands in one ask after a
  movement command.**
- **The store list renumbers when an item sells out**, so every letter below
  it changes - read the letter off the screen after each purchase, not before
  the first (minimaxflash-2#91). Only the letters *after* the removed row
  move, so **buy out the last row first** and every earlier letter stays
  valid (minimaxflash-2#24). The pack skips `h j k l`, so ten items are
  `a b c d e f g i m n`.
- **The rest prompt is pre-filled.** `R*` types `*` into a field that already
  holds `& `, so the game rests "as needed" rather than until spell points are
  full; one backspace clears the whole field (`src/ui-input.c`: the first
  backspace deletes all), which makes the one-ask full rest
  `R`+backspace+`*`+Enter (minimaxflash-2#91).
- **A reroll of the level costs three asks** (sonnet-1#67, sonnet-2#25):
  `< ` then `> ` on connected stairs gives a freshly generated level of the
  same depth. It was no quicker to finish than the level it replaced
  (sonnet-1#69).

## Depth is not the objective

Every run so far has treated the down staircase as the only thing worth
finding, and two of them died at character level 1 doing it - one on depth 1,
one in the town. The one that reached depth 7 arrived there at **character
level 8**, after an earlier life of the same character had spent its asks
levelling and shopping. Nothing here scores depth: the record keeps the depth,
the character level and the game's own valuation of everything you carry, and
a shallow level played well is worth more than a deep one survived by luck.

So **exploring the level you are on and going back to town is a plan, not a
failure.** It is cheap and it pays:

- you arrive on an **up** staircase, so `< ` is one key home-ward and `l<g`
  walks to one you have seen; Word of Recall brings you back from deeper and
  heals you on arrival;
- the town's shops **restock daily** and the shopkeepers rotate, so the Magic
  Mapping scroll that was not there on day one may be there on day two
  (`store.c store_maint()`, OBSERVED minimaxflash-1.2#399);
- experience and gold both come from the level you are on, and armour bought
  with that gold is what decides whether the next fight can touch you at all
  (see section 5 on hit chance);
- a level whose feeling says it is empty is worth leaving, and leaving means
  **either** staircase.

**The town-and-depth-1 shuttle**, which is the cheapest form of that plan and
which no run has tried: in the town you stand on the down staircase, and on
depth 1 you arrive on an up staircase, so `> ` and `< ` are one key each and a
whole cycle is two or three asks (FACT, connected stairs; sonnet-1#67,
sonnet-2#25). Every `>` builds a *new* depth 1, so the monsters beside the
staircase are new as well, and the town between cycles is where resting is
safe, where the shops restock and where nothing can reach you. What the
shuttle does not do is pay for itself: **stairs give no experience**, so a
cycle is worth exactly the kills you made before taking it, and section 1's
arithmetic says CL5 is about forty depth-1 kills for a Human. UNTESTED as a
route to a character level - what would settle it is one run that shuttles,
counts its asks per level, and says where the kills stopped paying.

## Constitution sets how fast you may dive

Hit points are a rate, not a class fact, and constitution is the rate.
`src/player-calcs.c calc_hitpoints()` is `player_hp[level-1] + adj_con_mhp x
level / 100`, with a floor of `level + 1`: the rolled part comes from the hit
die and a level adds a roll of `randint1(hit die)` (FACT,
`src/player-birth.c roll_hp()`), and **constitution adds or subtracts a fixed
amount on top, every level** (FACT, `src/player-calcs.c adj_con_mhp[]`) - in hit points per level:

| CON | 3 | 5 | 7 | 9 | 11 | 13 | 15 | 17 | 18/00 | 18/50 |
|---|---|---|---|---|---|---|---|---|---|---|
| hit points per level | -2.5 | -1 | -0.5 | -0.1 | 0 | +0.1 | +0.5 | +1 | +1.5 | +2.75 |

Your own modifier is the race's plus the class's (`race` and `class` `stats`,
which read STR:INT:WIS:DEX:CON): a **Gnome Mage is -1**, a Human Mage -2, a
Rogue or a Ranger -1 whatever the race, a High-Elf Druid +1, a **Half-Troll
Warrior +5**. Read `CON` off `C` and look it up. Two characters of the same
class and level can differ by ten hit points, and the gap widens with every
level, which is why a rule written in hit points fits nobody.

**So the rule is a pace, not a prohibition.** Gate each descent on the worst
round of the depth you are entering, and let a low constitution mean **more
levels between descents**, not a ban (POLICY, c15 DEPTH-01 put in a form you
can check on the screen; nobody has tested it as a rule):

1. **The worst single round at depth 2 or shallower is a soldier, 2 x 1d7 =
   14 damage** (FACT, `monster`). **5 of the 52 monsters at depth 2 or
   shallower can take 8 hit points in one round, and 3 of them can take 10.**
   Below the band's worst round, one bad corridor is a death; above it, a
   mistake costs a potion. Volume two has the worst monsters of each band
   below depth 5.
2. **Descend one depth at a time**, and look at `HP` on the left and that
   number before each staircase. A character at -1 or worse on constitution
   reaches any given hit-point total several levels later than one at +3, so
   it spends those levels where they are cheap - depth 1, depth 2 and the
   town - and the shuttle above is how.
3. **Before depth 1 at all**: 10 to 20 flasks of oil, **one Cure Light
   Wounds** (20 g) and **one Phase Door** (15 g). The pair is 35 gold out of
   the 300-odd a caster is born with, and nothing else buys a second chance.
4. **Read the escape at half your hit points**, not at two.
5. **One monster unaccounted for means one key an ask.** Send a batch of
   waits only when the screen is empty.

Three more things the shallow depths do that no hit-point total answers, all
FACT from `monster`: a **cave spider** comes in a group of **2d8** of them,
each 1d4, at speed 120 - two actions to your one; a **floating eye** (depth 1)
does no damage at all and its gaze paralyses, which is how a character at full
health dies; and **the town is not a lobby** - a battle-scarred veteran hits
for 2d6 = 12 and a mean-looking mercenary for 1d10 = 10, both at depth 0,
where there is no staircase to flee down and no experience to be had.

And what actually killed the three characters that died here - not one of
them to a big hit, and each line names the run that saw it:

- a **scruffy little dog**, 2 hit points, biting for 1d1, in the **town**: a
  CL1 Druid was clawed from 13 to 1 by scrawny cats while it hunted for shop
  doors over sixty asks, and the dog took the last point (qwen-flash-1#73);
- a **small kobold** that was not confirmed dead: a 10-hit-point Mage, blind
  and confused from a gas trap, sent sixteen `,` holds **in one ask**, and the
  kobold walked up during them. A batch of keys cannot be interrupted by what
  the screen would have shown (sonnet-3#47);
- a **metallic green centipede**, 1d1 and 1d1 at speed 120 - two damage a
  round: a CL2 Mage stood at 4 of 16 hit points with three monsters adjacent,
  throwing oil, with a Scroll of Phase Door in its pack that it had carried
  the whole game and did not read until it was at 2 (minimaxflash-2.2#350).

And the fourth death, which was none of those things: a **Human Warrior** at
**43 maximum hit points**, constitution +5, character level 4, killed on depth
2 by a **cave spider** at 26 of 43 - because it read a Scroll of Word of
Recall on arriving at a fresh level and then **stood still for forty turns**
waiting for it (minimaxflash-3#149). Hit points buy you mistakes; they do not
buy you standing still. See section 6 for what the Recall's timer actually
is.

None of this makes depth 1 safe, and none of it says stay in the town. It
makes a death a thing you chose rather than a thing that happened while you
were reading the map.

Descend when you can pay for the depth, and say in your report what you were
buying with the asks you spent. "I cleared depth 1, reached level 5, bought
armour and went down at ask 120" is a result. So is "I spent 150 asks looking
for a staircase and never found one" - but only one of those two was a choice.

## Where you are, and what you are standing on

Four reports in a row lost asks to the same three facts, none of which is in
either manual.

- **You begin the game standing on the town's down staircase.** FACT,
  `src/gen-cave.c town_gen_layout()` ends by setting `FEAT_MORE` on the
  player's own square and then placing the player on it. So the first useful
  key of a run is `>`, and no staircase has to be found to reach depth 1
  (OBSERVED, minimaxflash-2#7, which read the source after the status line had
  already said `Down staircase`).
- **The staircase under you is never drawn.** The map draws `@`, not the
  terrain beneath it; only the status line's last field names it (OBSERVED,
  minimaxflash-1.2#163). After going down you arrive on an **up**
  staircase, so `< ` then `> ` is a two-key reroll of the level you have just
  landed in, and in the town the `<` puts you back on its `>` (2 runs:
  sonnet-1#67, minimaxflash-1.2#163). A player scanning the map for a
  glyph will never see the one it is on.
- **The level is 198x66 and you see 66x22 of it.** FACT,
  `constants.txt` (`world:dungeon-wid:198`, `world:dungeon-hgt:66`) against the map panel,
  which is the screen's 80 columns less the 13-column sidebar and one; there
  are **three or four down staircases and one or two up** per level (FACT,
  `src/gen-cave.c classic_gen()` and `modified_gen()` both call
  `handle_level_stairs()` for `rand_range(3, 4)` down and `rand_range(1, 2)`
  up; section 8 has the profiles), each placed on a square with three
  adjacent walls - a corridor end - and kept at least MIN(width, height) / 4
  = 16 tiles apart (`src/gen-util.c alloc_stairs()`). **Three screens wide,
  three tall, several staircases at corridor ends**: that is the arithmetic
  behind every "I explored for ever" report here (OBSERVED,
  minimaxflash-1#104, minimaxflash-2#7).

**Aim the cursor at something you have seen.** In targeting mode the digit
keys move the cursor one tile each and `g` walks the shortest path to it, and
which path you get depends on whether the destination is *known*: aimed at a
square you have seen, the game's find_path gives a known-only path and a long walk -
17 to 44 tiles in one ask, measured (OBSERVED, minimaxflash-1.3#437). Aimed
into unexplored ground it assumes the unknown squares are passable, and the
path it reconstructs can be nonsense: one run measured a 20-tile aim that
walked **0 steps** and a 3-tile aim that walked 3 (OBSERVED,
minimaxflash-2#90). So aim at a landmark - a staircase, a door, the far
corner of a room you have lit - and use `lxg` for the frontier. The two runs
do not disagree about the game: they disagree about what was known when they
aimed, and that is the thing to check before you spend an ask on it.

**You cannot read your absolute position - and you do not need to.** No
screen names your coordinates, the panel scrolls in jumps and `M` draws only
what you have remembered. Two runs reported that as the thing that beat them
(minimaxflash-2#91 spent 158 asks not knowing whether it stood at x=20 or
x=120). It is not. **Nothing in this game asks you to go back to a place.**

- **A level you leave is gone.** Leave and return and the game generates a
  new one: different corridors, the up staircase elsewhere, and the down
  staircase you had found no longer remembered (OBSERVED,
  minimaxflash-1.3#437). So a map of the level has no second use, and the
  work of building one buys nothing.
- **Stairs are connected, and both directions are exits.** You arrive on an
  up staircase; `< ` and then `> ` is a fresh level of the same depth for two
  or three asks (2 runs: sonnet-1#67, sonnet-2.2#231). Going **up** is as
  good a way out of a bad level as going down.
- **This is not that other game.** Levels do not persist, there is nothing to
  come back for, and a coordinate is worth only what the next few steps need:
  what is adjacent, and which direction still has dark in it.

So the answer to "this level will not give me a staircase" is not a better
map. It is: take the staircase you arrived on and get another level. With
three or four staircases per level, a fresh one is as likely to put one near
you as this one was - at two asks instead of a hundred.

## 1. Be born (`birth` ; the birth screens)
- Race menu `*` (random item, Enter selects the highlight), class menu `*`. Stat screen: `a` (point-based), Enter to accept the allocation; Enter for the name, `y` for the history, any key to continue.
- Level-1 hit points = race hit die + class hit die (+CON) (`src/player-birth.c`: `hitdie = r_mhp + c_mhp`). FACT, `race` hitdie: Hobbit 7; Gnome, Kobold 8; Elf 9; Human, Half-Elf, Half-Orc, Dunadan, High-Elf 10; Dwarf 11; Half-Troll 12. Class die: Warrior 9, Blackguard 8, Paladin 6, Ranger 5, Rogue 4, Priest/Druid/Necromancer 2, Mage 0 (`class` hitdie).
- Level 2 needs `10 x XP-mod%` experience (`src/player.c`; mod = race exp, Human 100, Half-Troll 120, High-Elf 145). The sidebar `NXT` is experience still needed. A kill pays `monster exp x monster level / your level` (c15 15.3).
- The whole early ladder, cumulative and before that modifier (FACT, player_exp in `src/player.c`, applied in `src/player.c adjust_level()` as experience >= `player_exp[lev-1] x expfact / 100`): CL2 10, CL3 25, CL4 45, CL5 70, CL6 100, CL7 140, CL8 200, CL9 280. CL5 is therefore 70 experience for a Human, 84 for a Gnome or a Half-Troll, 101 for a High-Elf.
- **A kill pays less the higher your level, and that is what makes the middle levels slow.** The award is `mexp x monster level / your level` and the remainder is carried over, so nothing is lost to rounding (FACT, `src/mon-util.c monster_death()`). A small kobold (`monster` experience 5, depth 1) pays 5 at CL1 but 1.25 at CL4, so climbing CL1 to CL5 on depth-1 kobolds is about 2 kills, then 6, then 12, then 20: **forty kills, half of them for the last level alone**. The monster's own level multiplies, which is why a soldier (`monster` experience 6, depth 2) pays 12 at CL1, and why depth pays better than patience.
- **Nothing else pays at all**: stairs, walking, resting, shopping, detection and reading the map are worth zero experience. What pays besides kills is the first use of an unknown potion, scroll or wand (section 6b) and the **first cast of each spell** - the last field of its `class` `spell:` line. A mage's five CL1 and CL2 spells pay 4, 4, 2, 4 and 4, so studying them and casting each once is 18 experience: CL2 and most of the way to CL3 for a Human, for the price of the mana.
- Birth options in force (`birth` , `src/list-options.h`): start kit **on** (a few items, less gold), selling **off** (stores pay 0 gold), connected stairs **on**. Gold = 600 + unspent point-buy - kit value (`src/player-birth.c`), so it differs by class. Read `AU` on the left.
- Your class's block is in section 9; read it **after** the draw.

## 2. Look at what you have, then spend the gold
- `i` pack, `e` equipment, `|` quiver, `C` character sheet, `I` + item (damage per blow, blows per turn). Pack holds 23 kinds (`command`).
- Town stores are the digits on the map; walk onto one (`store` terrain order): **1** General, **2** Armoury, **3** Weaponsmith, **4** Bookseller, **5** Alchemist, **6** Magic shop, **7** Black market, **8** Home. In a store: `p` buys (then the row letter), `l` examines, Esc leaves.
- Prices. FACT (`object`, same as shop price for these): Flask of oil 3, Ration 3, Cure Light Wounds 20, Cure Serious Wounds 70, Phase Door 15, Magic Mapping 65, Word of Recall 125, Boldness 8, Monster Confusion 30, Blessing 15. **Gear (weapons, armour, torches, ammo) is priced by its power, not by the `cost:` line** (`src/obj-power.c: object_value_real`); read the price on the screen before buying. Torches burn out (the General Store lists 5000 turns).
- What each store always has (`store`): General: Ration, Torch, Flask of oil, Cloak, Shovel, Pick, shot/arrow/bolt. Alchemist: Cure Light Wounds, Phase Door, Word of Recall, Remove Curse; **sometimes** Magic Mapping, Cure Serious Wounds, Boldness, Remove Hunger. Magic shop **sometimes**: wands of Magic Missile and Stinking Cloud, staves of Teleportation, Detect Evil, Mapping (read the price). Bookseller: every town book. Not sold anywhere in town: Deep Descent, Teleportation scrolls, Healing potions (c15 TOWN-02).
- **POLICY** (c15 TOWN-01, c4 4.11): the town gives no experience; buy consumables first. Order there is escapes (Phase Door), a heal (Cure Light Wounds), then oil. Flasks of oil are the best early ranged attack for any class (FACT below, section 5); the strategy manual says 10-20 before the first descent.
- Armour above your class's weight allowance costs one spell point per pound over (Mage/Necromancer 30 lb, Priest/Druid/Rogue 35, Paladin/Ranger 40, Blackguard 60; c14 0.2).
- **Your pack slows you above half your strength's allowance.** The limit is `src/player-calcs.c adj_str_wgt` x 100 tenths of a pound, and past half of it you lose **1 speed per tenth of the limit over** (FACT, `src/player-calcs.c` weight analysis). The character sheet's Burden line shows the margin to exactly that threshold: a High-Elf Druid's sheet put it at 108 lb, and 20 flasks of oil are 40 of them - three runs sized their oil against this number without knowing what it was for (OBSERVED, minimaxflash-1#23, minimaxflash-1.2#155, mmf-mage.2#94). Slow is moves: at -1 a normal-speed monster takes an extra turn on you every ten.

## 3. Walk (`command`, `playing`)
- Step: a digit. Moving into a monster attacks it, into a door opens it, into a **visible trap tries to disarm it**; `W` + digit walks onto a trap on purpose. `o`/`c` open/close, `D` disarm or lock, `T` tunnel (rubble, veins), `g` pick up, `,` or `5` stay a turn. Stepping onto a square picks up **gold, always and for free**, but only *notices* anything else - the message is "You see %s." (`src/obj-pile.c`) - because the `pickup_always` option is off (FACT, `src/cmd-pickup.c do_autopickup()`, "Always pickup gold, effortlessly", and options may not be changed here); an object takes the `g` (OBSERVED, mmf-mage.2#43, an ask spent learning it).
- Run: `.` then a digit; it follows the corridor until you must choose or something disturbs you (`playing`, Disturb: a hit or spell on you, an object or monster in the next square, a known trap, a new level). A count repeats a command: `0`, number, command.
- `n` repeats the previous command. A keypress interrupts a run, rest or path (Space/Esc are safe).
- **Walk to a known place for one command** (see 'The one thing to take from this page' above; OBSERVED, sonnet-2.2#156 to #230, no longer untested) (`playing`, Targeting and Pathfinding, which lists `<`, `>`, `p` as up, down, unknown "respectively"; `src/ui-target.c` agrees): `*` or `l`, then `>` moves the cursor to the nearest *known* down staircase (`<` up), `x` to the nearest square next to unexplored space, then `g` walks you there along the shortest path, stopping when disturbed; `n` resumes. The `autoexplore_commands` option does the same on bare `>`, `<`, `p`, but options may not be changed here.
- Level feeling, bottom status line `LF:a-b`: **a is danger, higher = worse** (1 quiet ... 9 omens of death; `src/ui-display.c prt_level_feeling`), `?` until explored; b is loot. Chapter 15's STAIR-01 is not wrong, it is written for a different reader: internally 1 is the most dangerous (`src/generate.c calc_mon_feeling`, and the game's own comment in `src/ui-display.c`, "1 to 9 are feelings from omens of death to quiet"), and the screen prints `10 - feeling` to make it a danger number. A borg reads the struct; you read the reversal. So its `K_LF_LEAVE = 2` means 'leave the worst' on the internal scale and 'leave the two quietest' on yours - do not apply it to the screen. `^f` repeats the feeling text.
- Terrain on the status line names where you stand ("Up staircase", "Down staircase", "Open floor"). `>` and `<` work only on those.
- Stairs are connected (`birth`): after `>` you stand on an up staircase; after `<` on a down one. **So you never arrive without a way back: the staircase under your feet the moment you arrive is the exit in the other direction, one action away, and monsters do not follow you through it.** Taking it does not restore the level you left - leaving a level destroys it, and taking a staircase always gives a new one - but it always leaves the one you are in.
- Light: your kit torch is wielded at birth and burns out; `F` refuels a lantern from flasks of oil. You cannot read scrolls in the dark, blind or confused (c15 CURE-02, `src/player-util.c`).
- Hunger: `Fed %` on the status line; `E` eats.
- **A closed door may be locked, and a lock is a roll, not a wall.** One closed door in four gets a lock of strength `randint1(7)` (FACT, `src/gen-util.c`), and walking into it picks it: the chance is `MAX(2, disarm-phys - 4 x lock strength)` per cent, with the skill divided by ten when your own square is dark (FACT, `src/player-calcs.c calc_unlocking_chance()`, and `src/cmd-cave.c do_cmd_open_aux()` passes no_light). A class with disarm-phys 40 opens a depth-1 door about one attempt in four. Two costs the manuals do not mention, both measured: **a failed pick prints `Cancelled` and eats the rest of the keys in that ask**, so a string of repeats is one attempt, not five (OBSERVED, minimaxflash-1.4#616); and **a picked lock does not move you** - the door opens and you are still outside it, so every locked door is two decisions (OBSERVED, minimaxflash-2.2#162).

## 4. See (`command`, `playing`)
- `l` look. The list is sorted nearest first and includes your own square (`playing`; `src/target.c cmp_distance`), so it can open on it: Space or `+` for the next thing, `-` back, `r` toggles detail, Esc leaves (`playing`: "the list includes ... the player's location"). `[` lists visible monsters with asleep/awake and relative position. `]` lists items. `/` + symbol says what a glyph is. `M` whole-level map; `L` scroll the map by sectors (Esc leaves).
- Monster memory: look at a monster for its name and health; kills and hits teach you its attacks (`attack`). `~` knowledge menus.
- **A letter is a monster; an item is punctuation.** Items are drawn `! ? ( = " _ | \ , $ -` and the like, monsters with letters of either case (`monster_base.txt`: `i` is the icky thing, not an item). On the floor of these depths almost everything is a monster (OBSERVED, minimaxflash-2#91).
- Health bar of the tracked monster is in the sidebar; `[----------]` or nothing means **not in view**, not dead.
- **In look mode a direction key does not move the cursor one tile: it jumps to the next *interesting* square** in that direction - a door, an item, a monster, a staircase (`src/ui-target.c square_isinteresting()`). That is why `l` + direction + `g` walks a corridor to the next door in one ask, six to twelve tiles at a time (OBSERVED, minimaxflash-1.4#698). `p` switches to free mode, where a digit moves the cursor one tile; without it a digit picks the next thing and `g` walks you to that instead.
- **`l` then `>` is the only exact distance the game will tell you**: it parks the cursor on the nearest *known* down staircase and the message line prints the offset from you to it, `18 S, 30 W` (`src/ui-target.c`, `src/target.c coords_desc()`). The panel's own columns cannot measure anything because the panel scrolls; this survives it (OBSERVED, minimaxflash-2.2#178).
- **`!` is both a flask of oil and a potion, and only the colour differs** - `graphics:!:y` for oil, `graphics:!:d` for the potions (FACT, `object`). A text screen has no colour, so a `!` on the floor must be looked at before you assume: one run picked up what it was sure was oil and had bought itself four Potions of Cure Light Wounds (OBSERVED, minimaxflash-1.4#638).

## 5. Fight (`command`, `attack`, c4 4.11)
- Melee: walk into it. Blows per turn depend on STR, DEX and weapon weight (`I`nspect shows them). A hit that does no damage counts as a miss; armour class lowers both (`attack`).
- **Armour class mostly buys misses, not padding.** When a blow does land, armour removes `damage x min(AC, 240) / 400` of it (FACT, `src/mon-attack.c adjust_dam_armor()`) - **2.5% at AC 10**. What AC really buys is the monster missing more often (it is the defence value in the hit roll, `src/mon-attack.c`), which is the formula below. So a 100-gold armour for +4 AC is a poor trade for a class that can fight, and a **Potion of Berserk Strength** - which heals 30, gives **+75 to-hit** and fear immunity and takes **10 off your armour class** (FACT, `src/player-calcs.c`, the berserk timer) - is a bargain at that level, because the 10 AC it costs is worth 2.5% of one blow (measured by minimaxflash-3, which drank one and watched `Cur AC` fall from 10 to 0 while winning the fight; its report has the numbers).
- **Throw** `v`, the item, then a target. At the `Direction?` prompt: `'` = closest monster, `5` or `t` = the current target, `*` = choose (Space next, `t` select, Esc/`q` quit), a digit = that direction (`playing`, Targeting). `'` alone, as a command, targets the closest monster. Fire: `f`, ammo, target; **`h`** (or Tab) fires the first quiver ammo at the nearest visible monster.
- **Armour class decides whether you can hit at all.** FACT,
`src/player-attack.c:196 hit_chance()`: the chance is `12% + 83% x (to_hit - 2 x AC / 3) / to_hit`, so against AC `A` a to-hit below `2A/3` leaves you on the flat **12%** floor and nothing else. At depth 1 a giant black ant and a small kobold are AC 24, so they need to-hit 16: a character with to-hit 9-11 (a Mage's rapier, or its thrown oil) cannot fight them at all and must pick low-armour targets - worm mass, mould, mushroom patch, icky thing, mouse are AC 1-8 (OBSERVED, minimaxflash-1.2 on Grip at AC 36, minimaxflash-2#91 with to-hit 11). The same formula is what your own AC does for you: 200 gold of armour took one Mage from AC 1 to **AC 21**, which puts most of depth 1 on its 12% floor against eight hit points.
- **Flask of oil** FACT (`object`, c4 4.11 from `src/player-attack.c`): 1d4 x3 (throwing weight multiplier) x3 (EXPLODE) = **9 to 36 per hit, about 22**, damage that fire-resistant monsters take too. Range up to 10 squares. OBSERVED, one run each: one flask killed each of the depth-6 `p` humans it met - apprentice, gallant, tamer (sonnet-2#124, sonnet-2#131, sonnet-2#145); and Fang died at character level 2 to two flasks in a corridor and then melee, which cost 20 of 31 hit points (sonnet-1#84, sonnet-1#90). Daggers and Spears are `THROWING` too (x3 and x6 on dice).
- Spells: `b` browse, `G` study (new spells show "Study" on the status line; **Priests and Paladins** are given a random one, others choose), `m` + book + spell casts. First cast of each spell pays experience (the last field of a `class` `spell:` line). Casting with too little mana is allowed and can paralyse you and drain CON (c14 0.2): do not.
- Early dangers, `monster`, hit points / worst melee per **your** turn (a +10 speed monster acts twice):
  - **floating eye `e`** (DL1, 11 hp, never moves): its gaze **paralyses**. Never melee it; oil from range or ignore. Same for grey mushroom patch `,` (confuses) and yellow mushroom patch `,` (terrifies): they never move, do not stand next to them.
  - small kobold `k` 8 hp, 1d5; kobold 12 hp, 1d8; large kobold (DL5) 65 hp, 1d10; kobold archer (DL4) 24 hp.
  - **Fang and Grip** (DL2 uniques, +10 speed, 25 hp, bite 1d4: up to 8 per turn); wild dogs `C` come in packs (3 hp each, DL1).
  - **cave spiders `S`** (DL2, +10 speed, 7 hp, 1d4, **swarm** of 2d8 that surrounds you): fight in a corridor or leave.
  - soldier `p` DL2, 23 hp, 2 x 1d7 (up to 14); apprentice (blinks, blinds, magic missile), acolyte, cutpurse (steals gold) `p`; kobold shaman DL3 casts; black naga DL3, 27 hp.
  - **Bullroarer** DL5, +10 speed, 60 hp, 2 x 1d8 (up to 32 per turn). Smeagol (DL3, +20, invisible) steals gold by touch.
  - Breeders (`w` worm masses, `r` mice and rats; flag MULTIPLY): do not fight them in the open; leave.
  - Stationary jellies `j` and molds `m` cannot follow you. White jelly (DL2, 36 hp, poison touch) is 10 exp x level 2 / your level: **UNTESTED** idea, oil from range. Yellow jelly drains mana.
- **POLICY** c15 FIGHT-02: do not stay in reach of a monster whose worst turn is at least your HP minus a reserve (max(20, 10% of max HP); under 20 max HP the reserve exceeds you, so the rule cannot guide the first levels). FIGHT-03: fight groups in a corridor or doorway, not in a room. Chapter 14 adds: do not fight breeders in the open.

## 6. Escape (c13, c15 ESC)
- Standing on a staircase: take it (one action). Phase Door (15 g): teleports about 10 squares; Cure Light Wounds: heals 20, cures blindness, cuts 20 and confusion 20 turns (`object`). Potions work blind or confused; scrolls do not.
- **POLICY** c15 ESC-01 ladder, in order: a staircase under you; Phase Door if one monster is the problem; Teleportation; heal; an unknown potion only as the last rung. HEAL-01: heal on predicted danger, the smallest heal that makes you safe; HEAL-04: if the monster out-damages your best heal per turn, leave instead.
- Do not treat Word of Recall (15-34 turns delay) or Deep Descent (3+1d4 turns) as an escape (c15 ESC-04). Those are *player* turns: FACT, `src/game-world.c` decrements the recall counter once every ten game turns, so the wait is 150-340 game turns, and a run that gave up on it after about forty had waited a quarter of the minimum. **One run died of exactly this** (OBSERVED, minimaxflash-3#149): it read the Recall on arriving at a fresh depth 2 and stood still for forty turns, and a cave spider killed it at 26 of 43 hit points - 110 game turns after reading, 40 before the earliest the scroll could have fired, **while standing on a down staircase it never weighed**: it had come up from depth 3, so the staircase under it went the wrong way for the town it wanted, but it was still a one-action escape from the spider, and monsters do not follow through stairs. Read it and keep walking, or do not read it. **Or rest the timer out** (OBSERVED, mmf-mage.2#91): the wait is 15 to 34 player actions and one full rest is about 61 for a single keypress, so one `R` backspace `*` Enter covers it with margin - on a safe square, because a monster reaching you interrupts the rest. Read in town before you have ever left it, Recall takes you to DL1 rather than nowhere - see volume 2.
- Running from faster monsters does not work: +10 speed is double your moves (`attack`, "A note on speed").

## 6b. What to keep and what to leave

**Selling pays nothing** (`birth_no_selling` is on; `birth`, c15 ITEM-05). So an
object is worth carrying only if you will **use** it, **wear** it, or **learn
what it is** - see the next block, which is the one exception to hauling. Gold
is the only loot with a value, so do not carry gear to town: it pays 0.

### An unknown item is a pool of kinds, not a hope

Nothing you pick up is a particular thing until you learn it, and **what it
can be is fixed by your depth, not by what you want**: an unknown consumable
is one kind of its type whose `alloc:` band covers the depth you found it at,
weighted by the first number of that band (FACT, `object`). At **depth 2**
the whole pool is:

| unknown | the kinds it can be, with their weights | chances |
|---|---|---|
| **scroll**, 8 kinds | Treasure Detection, Phase Door, Light, Detect Invisible, Blessing 50 each; Deep Descent 20; Summon Monster 10; Darkness 10 | Phase Door 17%, Deep Descent 7%, **7% against you** |
| **potion**, 11 kinds | Cure Light Wounds 80; Neutralize Poison 60; Speed, Heroism, Boldness 40 each; Slowness, Sleep, Poison, Confusion, Blindness, Slime Mold Juice 10 each | 84% harmless or good, **16% against you** |

**Magic Mapping is not in that pool.** Its band starts at **DL5** (`object`
`alloc`), so an unknown scroll on depth 1 or 2 cannot be it, however much the
run would like a staircase detector. Nor can anything of a type that does not
exist yet: the shallowest **wand** is DL3, the shallowest **staff**, **rod**,
**mushroom** and **ring** DL5, the shallowest **amulet** DL10. On depth 1 and
2 the only unknown kinds in the game are scrolls and potions.

One exception, and it is small: one object in twenty is generated as if from a
deeper level (`src/obj-make.c`, `one_in_(great_obj)` with
`obj-make:great-obj:20` in `constants.txt`, which recomputes the level as `1 +
level x 100 / randint1(100)`). So a wand on depth 2 is possible and rare, and
when you find one the pool to think about is the deeper one.

**So do not plan around one kind.** A run on depth 2 wrote that its standing
question was what Magic Mapping shows and walked to two unknown scrolls to
find out; the pool says it had a 17% chance of a Phase Door and no chance at
all of the thing it was hoping for. The useful question is never "is this the
one I want", it is "what does this pool make likely, and can I afford the
tail".

**Three ways a kind becomes known**, all FACT:

1. **Use it.** Reading or quaffing identifies the kind at once and the first
   use pays experience (section 1). For a depth-2 potion that is an 84% good
   bet; for a scroll, 7% of the pool is Summon Monster or Darkness.
2. **Buy one.** Everything in a shop is identified, so every kind the
   Alchemist stocks is a kind you can rule out by sight for the rest of the
   game - and the stock changes daily.
3. **Give it to the store that buys that type.** `src/store.c do_cmd_sell()`
   calls `object_flavor_aware()` and learns the object's runes **before** the
   item changes hands, so with selling off you get **0 gold, lose the item,
   and know the kind for the rest of the game**. The prompt reads `Give which
   item?` and the message afterwards is `You had ...` (`src/ui-store.c`).
   **This is the only way to identify something you dare not use**, and it is
   worth a town trip of its own.

Who takes what (`store` `buy:`): **Alchemist (5)** scrolls and potions;
**Magic shop (6)** wands, staves, rods, rings, amulets and books; **General
(1)** mushrooms, flasks, food, lights, diggers, cloaks and ammunition;
**Armoury (2)** armour; **Weaponsmith (3)** weapons, bows and ammunition;
**Bookseller (4)** books. The **Black market and your Home buy nothing**.

So the workflow: pick unknown consumables up, use the ones whose pool is
mostly harmless **while you are safe and at full hit points**, keep the rest,
and give them away on your next trip to town - one trip identifies every
unknown you are carrying, and a trip to town costs two or three asks from
depth 1 (see the shuttle above).

**The rule that does the rest:** in `object.txt` every harmful consumable has
`cost:0`, and every useful one has a price. FACT, checked against the file:
cost 0 at `level:` 0-10 is Potion of Sleep, Blindness, Confusion, Salt Water,
Slowness, Poison, Surprise; Scroll of Summon Monster, Darkness, Aggravate
Monster, Trap Creation; Staff of Darkness, Summoning, Haste Monsters; Wand of
Heal Monster, Haste Monster. Once a kind is identified, its price in a shop
tells you which group it is in.

**Keep, in this order of usefulness to a shallow character** (in `object`, `cost:` is what the
Alchemist or General Store charges for potions, scrolls and food, and so what the
thing is worth to you; for the torch, lantern and wands it is only the base cost,
the shop prices them differently, so read the screen. `found from` is the first
depth the game can generate it at, the `alloc:` minimum in `object`; `level:` is
the object's own level and is not where you find it):

| Keep | `cost:` | `level:` | found from | why |
|---|---|---|---|---|
| Flask of oil | 3 | 1 | 1 | the ranged attack every class has; 10-20 of them |
| Potion of Cure Light Wounds | 20 | 0 | 1 | the heal you will actually reach for |
| Scroll of Phase Door | 15 | 1 | 1 | one monster is the problem; short hop |
| Ration of Food | 3 | 0 | 0 | two or three; starvation has killed a run |
| Wooden Torch / Lantern | 2 / 35 | 1 / 3 | 1 / 5 | you cannot read scrolls in the dark |
| Scroll of Magic Mapping | 65 | 5 | 5 | maps within 22 rows and 40 columns of you - but one run paid 65 and saw no staircase |
| Potion of Cure Serious Wounds / Cure Critical Wounds | 70 / 200 | 3 / 5 | 5 / 12 | when Light stops being enough |
| Scroll of Teleportation | 40 | 10 | 10 | the escape when Phase Door is not far enough |
| Scroll of Deep Descent | 250 | 10 | 1 | not sold; five levels below your deepest |
| Scroll of Word of Recall | 125 | 5 | 5 | read in the dungeon it takes you to town; read in town before you have left it, it drops you to depth 1 - 125 gold wasted |
| Potion of Speed / Heroism / Berserk Strength | 75 / 25 / 80 | 1 / 1 / 3 | 1 / 1 / 3 | drink before a fight you have chosen |
| Wand of Magic Missile / Stinking Cloud | 200 / 400 | 3 / 5 | 3 / 5 | ranged damage for a class with no spell |

**Leave** everything with cost 0, and everything you will not wear: a second
weapon, armour too heavy for your class's spell allowance, a ring or amulet
whose rune you have not identified and cannot afford to test.

**An unknown potion or scroll is worth trying** rather than carrying, because
first use pays experience and selling pays nothing - what the first use pays,
and when to risk it, is under "Keys as the game receives them" above.

## 7. Recover
- `R` then `&` then Enter rests until healed or disturbed; `R` backspace `*` Enter until HP and SP are full (the prompt is pre-filled with `& `, so `R` `*` rests only as needed - see section 1); or give a number. Resting doubles regeneration; poison, stun, cuts stop it (c15 CURE-01, REST-01).
- **POLICY** c15 REST-01/02: rest only when no awake monster can reach you in about 5 turns; rest on a staircase when one is within about 10 steps.
- Blackguards regenerate HP at half rate; Necromancers and spell-users rest for mana.

- **A trap door is not a way down, it is how you lose control of your descent.**
  `trap` gives it 2d8 damage (2-16, and a level-1 Mage has 7-12 hit points and a
  level-1 Druid has 9-14 hit points) and it drops you one level **on no staircase**. That second part is
  the expensive one: arriving off stairs forfeits the connected-stairs escape
  and the two-ask reroll (`< ` then `> `) that every run which survived has
  leaned on, and you land in an unexplored level with no known way back up.
  Recognise it - shown `^`, named by `l`ook - and walk around it. `W`+digit
  steps onto a trap deliberately; there is no reason to. If you fall through
  one anyway, find a staircase before you pick a fight.

## 8. Descend
**When you can pay for it** - see "Depth is not the objective" above; the
choice of whether to go down now is yours to make and to report, and this
section is only how.
- Every level has down staircases (`command`) but you must find one. Ways down, all FACT unless marked:
- **How many there are, and where they can be.** A depth-1 level is built by `src/gen-cave.c classic_gen()` or `src/gen-cave.c modified_gen()` - the only two profiles with an `alloc` at that depth (`dungeon_profile.txt`: modified 97, classic 90, cavern and the rest deeper or special) - and both ask for **3 or 4 down staircases and 1 or 2 up** (`src/gen-cave.c handle_level_stairs()`). Each one is placed on an empty square with **three adjacent walls** - three of its four orthogonal neighbours, diagonals not counted - and the requirement drops by one wall at a time only when no such square is left: a corridor end or a dead end, never the middle of a floor. They are kept **at least `min(width, height) / 4` = 16 squares apart** (`src/gen-util.c alloc_stairs()`). So a level with no staircase in it does not exist, and a level you cannot find one on is a level you have not covered.
- **A sweep is not how you find them** (OBSERVED, minimaxflash-1.4#753). One life spent about 140 of its decisions sweeping a depth-1 level: fifteen locked doors opened, four chests found empty, two up staircases found besides the one it arrived on, the level mapped - by its own estimate - to two thirds of its squares, and **no down staircase in any of it**. With three or four of them on that level the sweep plainly covered less than it looked, which is the trap: `M` draws the squares you *remember*, and a band of columns you have crossed is not the squares inside it. The same run wrote at its ask 734 that "the down staircase is at the end of one of them" - one, singular, which is what everybody assumes. `< ` then `> ` is two or three asks and a level with three or four fresh ones.
- **The level is sparse by design, and everything on it is asleep.** A level at depths 1-6 gets 14 + `randint1(8)` + 2 monster placements (FACT: `constants.txt` `mon-gen:level-min` is 14, and `src/gen-cave.c classic_gen(), z_info->level_monster_min + randint1(8) + k` with k = MAX(MIN(depth/3, 10), 2)), each placed asleep (`src/mon-make.h pick_and_place_distant_monster()`'s sleep argument, set true at generation) and each possibly a group (`monster.txt` `friends`). Seventeen to twenty-four placements over 198x66 squares is one per several hundred squares, so an empty corridor is the level, not your luck (OBSERVED, mmf-mage.2#72 spent twenty asks on an empty depth 2 before reading the source).
  0. **The town's own staircase, which you start on** — `>` and you are on depth 1 (see "Where you are" above);
  0b. **`l>g` then `>`** — the one to use once a `>` is known; see the top of this page;
  1. a `>` you walk onto and press `>` on;
  2. Scroll of **Deep Descent** (`object`: found from DL1, **not sold**): after 3+1d4 turns you drop to **5 levels below your deepest level reached**, not below where you stand (`src/effect-handler-general.c`), landing on no staircase;
  3. Scroll of Magic Mapping (65 g, Alchemist when stocked; found from DL5) maps everything within 22 rows and 40 columns of you, a radius each side (`object` `effect-yx`; `src/effect-handler-general.c` MAP_AREA);
  5. Word of Recall (in your kit unless that option is on): read in the dungeon it takes you to town, read in town it returns you to your deepest depth after 15-34 turns - or to depth 1 if you have never left the town (c15 TOWN-05).
- **Detection is the only thing that finds a staircase you have not seen**, and only a Mage or a Rogue has it: `Find Traps, Doors & Stairs` is `DETECT_STAIRS` with `effect-yx:22:40`, which is `+-22` rows and `+-40` columns - an **81x45 window, about 28% of a level** - and it reads the *real* grid and memorises what it finds (`src/effect-handler-general.c:1467`), so it reveals a staircase never seen. Three things measured with it (OBSERVED, minimaxflash-2#91, eleven casts): it **only reports ground it has not already scanned**, so two casts from the same place give "You sense no traps. You sense no doors." and nothing else; when there is no staircase in the window it says **nothing at all** about stairs, which is what a negative looks like; and covering a level takes **three positions across and two down, six casts**. Do
not try to compute those positions - you cannot read your coordinates (see
"Where you are" above). Cast, walk a long way with one aimed `g`, cast again,
and if two casts find nothing, **reroll the level**: a fresh one is as likely
to have its staircase in your first window as this one was. **But read the
next line before you cast twice on one level.**
- **The detection spell breaks the game's own walking, and that is the most
expensive thing any run here has found.** `DETECT_TRAPS` sets the trap-detected flag
on **every** square of its window, trap or no trap (FACT,
`src/effect-handler-general.c:1376`), and `src/cmd-cave.c run_step()` stops a run - and the
`g` pathfinder with it - the moment it steps **off** a detected square, every
step but the first (FACT, `src/cmd-cave.c run_step()`, which disturbs the
player when the square left was detected and the one entered is not). So on
a level you have cast on twice from different places, the seams between the
windows cut every walk: `l>g` to a staircase 36 tiles away moved **one or two
tiles an ask for about a hundred asks** (OBSERVED, minimaxflash-2.2#165,
#256). Three things follow, and they are the whole technique:
  1. **Plain digit walking is immune** - `src/cmd-cave.c do_cmd_walk()` has no such test, so
     `44` always moves you two tiles west. If a walk is being cut short, send
     digits.
  2. **A fresh level has no detected squares at all**, so the first cast's own
     window walks clean: one cast on a new depth 2 and the staircase inside it
     was reached in a single ask (OBSERVED, minimaxflash-2.2#178).
  3. **On a level you have already broken, cast again from a spot whose window
     covers the path** - one spell point re-marks every square, seams
     included, even though the spell then reports nothing new. A cast that
     says "You sense no traps" has still done this work. At CL1 one spell point is about 71 player actions of regeneration (`player_regen_mana()`, `msp x 197 + 524` in 65536ths, doubled while resting), so those six casts plus the travel between them are **the whole of a 150-decision budget**.
- **POLICY** c15 DEPTH-02, minimum kit before going below: DL5 needs Word of Recall, 3+ cure potions and a ranged attack; DL15 needs 100 max HP and about 10 Phase Door. DEPTH-01 says gate depth on the worst single hit of the depth band, not on character level. Treat these as numbers to test.

## 9. Your class (`class`, `race`, c14; section numbers are c14's)
Level-1 facts are from `class.txt`; "buy" is POLICY from c4/c14/c15 with real prices from section 2.

**Every class is born with a Scroll of Word of Recall** (all nine `class.txt` entries have an equip line for it, gated on the `birth_no_recall` option, which is off here), so the kit lists below omit it: it is the way home in section 6, and the scroll a Warrior read and then stood still beside for forty turns (minimaxflash-3#149).

**Warrior** (c14 s1): hit die 9, up to 6 blows, no spells, fear-proof only from CL30 (before that fear stops melee: Boldness 8 g, Berserk Strength). Kit: Dagger, Soft Leather Armour, Berserk Strength, food, torches. Buy: oil 15-20, Cure Light Wounds, Phase Door; a Sling or Short Bow if gold is left. Kills it: paralysis (floating eye), fear, open-room packs, running out of escapes. **It can never dig**: `skill-dig:0:0` in `class` and no points a level, while `src/player-calcs.c calc_digging_chances()` needs digging skill over 10 for magma and over 20 for quartz, so every vein on every level is a permanent wall to this class - and a `*` is a vein **with treasure** in it (`%` is a plain one), which means every `*` you see is money a Warrior cannot have. A Shovel does not change it (measured by minimaxflash-3; its report has the veins it walked past).

**Mage** (s2): hit die 0 (HP = race die: **10** for a Human, 8 for a Gnome), INT, 30 lb armour. Learns **one** spell at CL1 (`G`, book, spell); studying costs one action and nothing else - no mana, no experience, no gold (`src/cmd-obj.c do_cmd_study_spell()`) - and the status line says `Study` while a slot is free, so take every slot the level gives you as soon as it appears. CL1 spells: Magic Missile 1 sp (dice `$Dd4` with the count `(CL-1)/5+3`, so **3d4, 3 to 12 damage**, all the way from CL1 to CL5 and 4d4 at CL6; never resisted. The 3 is the dice count, not a bonus: there is no `+3`, and a page that reads it as 6-15 is reading the expression wrong), Light Room 2 sp, Find Traps/Doors/Stairs 1 sp (detects within 22 rows and 40 columns). That last one is the staircase detector, and what its window costs in asks is in section 8. **Its melee is not a fallback**: to-hit 9 with the kit rapier is below `2 x AC / 3` for most of depth 1, so melee lands on the 12% floor only (OBSERVED, minimaxflash-2#91). Spell points at CL1 regenerate at about 0.014 a player action, one point per 71 actions. Phase Door spell at CL2, Detect Monsters CL3, Fire Ball CL6. Kit: Rapier, [First Spells], food, torch. Buy: oil (your real attack), Cure Light Wounds, Phase Door scrolls. Kills it: one round of melee, casting at 0 mana, anything adjacent at 10 HP. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Druid** (s3): hit die 2, WIS, no teleport spell ever. CL1: Detect Life 1 sp, Fox Form 2 sp (double move, but a shape cannot read, quaff or cast: change back first); Remove Hunger CL2 (no food needed), Stinking Cloud CL3 (2 sp, poison ball, your gun). Kit: Whip, [Lesser Charms]. Buy: oil (nothing else hits hard until CL3), Cure Light Wounds, Phase Door. Kills it: no escape when surrounded, poison-immune targets. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Priest** (s4): hit die 2, WIS, random spell learning. CL1 spells: Call Light, Detect Evil, Minor Healing (2 sp), Bless (2 sp, +5 AC, +10 to-hit). Kit: Mace (blessed +2/+2), **2 Cure Serious Wounds**, [Novice's Handbook]. Orb of Draining needs CL7 and [Cleansing Power] (kind cost 100, Bookseller). No escape spell until a DL30+ book. Buy: oil, Phase Door scrolls, Cure Light Wounds. Kills it: no escapes, non-evil animals, blindness or confusion stopping prayers. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Necromancer** (s5): hit die 2, INT, **sees without light and must not carry one**: a lit grid adds 25% spell failure; at CL1-5 you see only adjacent squares in the dark. Nether Bolt 1 sp (3d4; undead immune, evil half), Sense Invisible CL2, Create Darkness and Bat Form CL3. Kit: Main Gauche, [Into the Shadows], food; no torch. Buy: oil, Cure Light Wounds, Phase Door; **no torch**. Kills it: fighting in a lit room, undead, 10-HP fragility. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Paladin** (s6): hit die 6, 5 blows, random spell learning. CL1: Bless only (Detect Evil CL3, Call Light CL5, Minor Healing CL7, Heroism CL12, full effect CL15, for fear). Kit: Main Gauche, [Novice's Handbook], Protection from Evil scroll. Buy: oil, Cure Light Wounds, Phase Door. Kills it: as the Warrior, plus fear before CL12 and mana starvation.

**Rogue** (s7): hit die 4, best throwing skill (72) and stealth, first spell at **CL5** (none now; Phase Door spell CL7, 55% base failure: carry scrolls). `s` + digit steals from an adjacent monster, best while it sleeps. Kit: Dagger, Soft Leather Armour, [First Spells]. Buy: oil, Phase Door, Cure Light Wounds, a Sling; throw Daggers and Spears. Kills it: waking a pack, early melee at low HP. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Ranger** (s8): hit die 5, best shooting skill (72), extra shots; first spell at **CL3** (Remove Hunger). Kit: Main Gauche, Short Bow, 15-20 Arrows, [Lesser Charms]. `h` fires at the nearest; arrows break. Buy: arrows (priced by power, believed about 1 g each: UNTESTED; 40+), oil, Cure Light Wounds, Phase Door. Kills it: out of arrows, fast melee in a corridor, archers. **Your hit points grow slowly**: this class is -1 or worse on constitution, so see **Constitution sets how fast you may dive** above - it is a pace for descending, and three of the four characters that died here were this fragile (the fourth was a Warrior).

**Blackguard** (s9): hit die 8, 5 blows, heavy weapons, 60 lb armour. Spell points are **earned by fighting and being hurt**, decay when idle, and casting heals; HP regenerates at **half** rate. CL1: Seek Battle 1 sp (little use); Berserk Strength CL3, Whirlwind CL5. Kit: Tulwar (2d4), Leather Shield, Cure Light Wounds, [Into the Shadows]; **starting gold is nearly gone**. Buy: whatever Cure Light Wounds and oil it covers. Kills it: starting a fight already hurt, ranged attackers.

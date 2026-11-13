# ACCUMULATED KNOWLEDGE — what the runs found that no manual says

Edition 7, 2026-11-10. *(Edition 1, the same day, repeated things the manuals
already say; see "What is deliberately not here". Edition 2 added the eleventh
run and the caveat in section 1 that the runs never questioned their own
objective. Edition 3 added three runs and corrected the single most
load-bearing claim of edition 2: a level has three or four down staircases,
not one. Edition 4 added two runs - a Gnome Mage played to the page's plan
and a fifth Druid life that found the staircase and died past it - the fifth
death, the fear mechanic, and the map facts that make "no staircase on the M
map" real evidence. Edition 5 added the first Priest and Dwarf, and the sixth
death. Edition 6 added the mage carry-on's death - the seventh - and the
locked-door and never-sleeping corrections. **Edition 7 adds the Priest's
second life and the first Ranger (the first Hobbit), and three measurements
from play: what actually cuts a key string (`disturb()` and its list), what
each movement command is worth per ask, and that the status line's `Light N`
is the square's light, not the torch's.**)*

**Who this is for.** [borg-reborn-rs](https://github.com/diegoami/borg-rs),
the Rust Borg being rebuilt, and anyone else outside this repository. The
process that produces it is [docs/EXPORT.md](docs/EXPORT.md); it is rewritten
from time to time, not after every run.

**What it contains, and why it is short.** The consumer can read the game's
own manual, the strategy manual and `lib/gamedata` itself, at leisure and in
full — a program has time a player in a 150-decision budget does not. So
nothing here is in those documents. What is here is the residue: **what
twenty-one runs measured, what they corrected, and what had to be read out of
the source because no manual states it.**

**Not for the players.** A run here is denied exactly one thing: another run's
account of the seed it is playing. This file aggregates every report, so
handing it to a player would hand them a solution key for their own dungeon.
The prompts do not mention it.

**How to read a claim.** Each names its evidence:

| | |
|---|---|
| `sonnet-2#124` | run and ask: screens in `records/sonnet-2/`, the player's words in `knowledge/sonnet-2.md` |
| GAME SOURCE | read out of `vendor/angband` at the pin (4.2.6, `e473720ed88a`); true of the game, and **not in the manuals** |
| ONE RUN | one player, once. A lead |
| N RUNS | N different runs, independently |

## The corpus

Twenty-one runs, 5,637 asks, 42,684 keypresses, deepest **depth 7**, seven
deaths. One session was played by a person (`diego-1`), which is the control.
Three characters were asked for (two Gnome Mages and the Warrior that
followed them); the rest were drawn. By class: six Warrior runs, six Druid
runs (five of them lives of one character), five Mages, three Priests (one
the person, two lives of one Dwarf) and one Ranger - **Blackguard, Paladin,
Rogue and Necromancer have never been played here.**

| run | model | character | depth | level | asks | turns | ended |
|---|---|---|---|---|---|---|---|
| [sonnet-1](records/sonnet-1/REPORT.md) | anthropic/claude-sonnet-5-5 | Half-Troll Warrior | 1 | 3 | 125 | 4,659 | alive, budget |
| [sonnet-2](records/sonnet-2/REPORT.md) | anthropic/claude-sonnet-5-5 | Half-Troll Warrior | 6 | 6 | 153 | 6,056 | alive, budget |
| [sonnet-2.2](records/sonnet-2.2/REPORT.md) | anthropic/claude-sonnet-5-5 | Half-Troll Warrior | **7** | 8 | 291 | 10,416 | alive, budget |
| [sonnet-3](records/sonnet-3/REPORT.md) | anthropic/claude-sonnet-5-5 | Human Mage | 1 | 1 | 47 | 1,348 | **died**, small kobold |
| [codex-1-tty](records/codex-1-tty/REPORT.md) | openai/gpt-5.6-luna | Dunadan Warrior | 1 | 1 | 66 | 1,860 | alive, in town |
| [qwen-flash-1](records/qwen-flash-1/REPORT.md) | alibaba-token-plan/qwen3.8-flash | High-Elf Druid | 0 | 1 | 73 | 570 | **died** in the town, scruffy little dog |
| [minimaxflash-1](records/minimaxflash-1/REPORT.md) | minimax/MiniMax-M3.1-Flash | High-Elf Druid | 1 | 1 | 148 | 7,433 | alive, budget |
| [minimaxflash-1.2](records/minimaxflash-1.2/REPORT.md) | minimax/MiniMax-M3 | High-Elf Druid | 2 | 4 | 401 | 10,898 | alive, budget |
| [minimaxflash-1.3](records/minimaxflash-1.3/REPORT.md) | minimax/MiniMax-M3 | High-Elf Druid | 1 | 4 | 610 | 17,281 | alive, budget |
| [minimaxflash-2](records/minimaxflash-2/REPORT.md) | minimax/MiniMax-M3.1-Flash | Gnome Mage (asked for) | 1 | 1 | 158 | 8,001 | alive, budget |
| [minimaxflash-2.2](records/minimaxflash-2.2/REPORT.md) | minimax/MiniMax-M3 | Gnome Mage | 2 | 2 | 354 | 10,811 | **died**, metallic green centipede |
| [minimaxflash-1.4](records/minimaxflash-1.4/REPORT.md) | minimax/MiniMax-M3.1-Flash | High-Elf Druid | 2 | 4 | 759 | 24,105 | alive, budget |
| [minimaxflash-3](records/minimaxflash-3/REPORT.md) | minimax/MiniMax-M3.1-Flash | Human Warrior (asked for) | 3 | 4 | 152 | 8,394 | **died**, cave spider |
| [mmf-mage.2](records/mmf-mage.2/REPORT.md) | minimax/MiniMax-M3.1-Flash | Gnome Mage (asked for) | 3 | 1 | 146 | 4,872 | alive, stopped |
| [mmf-mage.3](records/mmf-mage.3/REPORT.md) | minimax-coding-plan/MiniMax-M3.1-Flash | Gnome Mage (asked for) | 3 | 1 | 199 | 8,019 | **died**, crow |
| [mmx-pri-1.2](records/mmx-pri-1.2/REPORT.md) | minimax-coding-plan/MiniMax-M3.1-Flash | Dwarf Priest (asked for) | 1 | 3 | 206 | 13,883 | alive, stopped |
| [mmx-ran-1](records/mmx-ran-1/REPORT.md) | minimax-coding-plan/MiniMax-M3.1-Flash | Hobbit Ranger (asked for) | 1 | 1 | 151 | 5,219 | alive, stopped |
| [minimaxflash-1.5](records/minimaxflash-1.5/REPORT.md) | minimax/MiniMax-M3.1-Flash | High-Elf Druid (life 5) | 2 | 4 | 893 | 29,966 | **died**, an acolyte |
| [diego-1](records/diego-1/REPORT.md) | a person | Kobold Priest | 1 | 2 | 418 | 4,741 | alive, saved |
| [mmx-pri-1](records/mmx-pri-1/REPORT.md) | minimax-coding-plan/MiniMax-M3.1-Flash | Dwarf Priest (asked for) | 1 | 2 | 149 | 9,952 | alive, stopped |
| [mmx-war-1](records/mmx-war-1/REPORT.md) | minimax-coding-plan/MiniMax-M3.1-Flash | Half-Elf Warrior (asked for) | 2 | 4 | 141 | 10,923 | **died**, silver mouse |

Every row links a report, and every report links a replay that steps through
screen by screen: <https://diegoami.github.io/borg-runner/>.

---

## 1. Experience is the objective, not depth

Sixteen of twenty-one runs ended at depth 0, 1 or 2, and the seven deaths are
all in that band too. The fourteen that stopped were alive, and most of them
had spent their asks looking for a way down. **That was the wrong job.** The
first character levels are on depth 1: a depth-1 kill pays 1 to 5 experience,
and CL1 to CL3 is a handful of kills. A run that covers the level, kills what
it meets and shuttles to town has done the job without a `>`. The priest's
second life is the proof: **CL3, 625 carried, no staircase found on either of
two depth-1 levels** (ONE RUN, `mmx-pri-1.2#205`). Read this section as where the
decisions went, not as proof that descent is the goal.

- **Walking a depth-1 level costs on the order of 100 decisions and finds no
  staircase - but it is where the first kills are.** 100 of `sonnet-1`'s 125
  asks; 148 of `minimaxflash-1`'s; 174
  screens of depth 1 in `minimaxflash-1.3`; 4,290 of `diego-1`'s 4,741 game
  turns on one level (5 RUNS, one of them a person).
- **The person and the first model run are the same session.** 971 keypresses
  against 644, 4,741 turns against 4,659, both depth 1, neither found a way
  down. Three in five of the person's keypresses moved one square; the run
  command was used 32 times in 418 (`diego-1`).
- **The one run that changed this changed one thing**: it used the game's own
  routing to walk to a staircase it had seen (`sonnet-2.2#228`–`#230`, 11
  steps and 110 game turns in a single decision), and reached depth 7. The
  corpus' only other descent was a Deep Descent scroll found by luck
  (`sonnet-2#77`).
- **A player that writes its own router measures its own router.**
  `minimaxflash-1` spent 148 asks driving a 143-line breadth-first search it
  wrote, reached depth 1, and never tried the game's commands. Its next life
  used the game's and got further on fewer asks.

**The caveat that matters as much as the finding: no run ever questioned the
objective.** All twenty-one treated the down staircase as the only thing worth
finding, and the seven that died were at character level 4 or lower while
doing it. The one run that planned to level instead (`mmf-mage.2`, asked to
play a Gnome Mage by the page's own arithmetic) found the plan's real
bottleneck: not the fighting but the *finding* - a CL1 kill on depth 1 pays
1 to 5 experience and the monsters are fifty placements over 198x66
squares, asleep. The gate to the experience is the staircase, not the
killing: a depth-2 native pays about twice its depth-1 worth at the same
character level (`minimaxflash-1.5`, which earned more in one depth-2
chamber than its first four lives earned in total). The
alternative was never tested: explore the level you are on, take the
experience and the gold, go back up - you arrive on an *up* staircase, and the
town's shops restock daily - and descend when the character can pay for the
depth. The one run that reached depth 7 is in fact evidence for that plan
rather than against it, because it arrived at **character level 8**, after an
earlier life of the same character had spent its asks levelling and shopping.
So read section 1 as *where the decisions went*, not as proof that descent is
the only way to spend them.

**For the consumer:** the manuals describe descent as the objective and say
nothing about its cost. Measure decisions-per-descent, but also
decisions-per-character-level: a policy that spends 80% of its decisions on
floor coverage has 20% left for everything the manuals are about. The seven
deaths are their own finding (section 6): none was a big hit, and none was
at a depth the character had no business being at.

## 2. What the generator does, which decides whether searching can work

All GAME SOURCE, found because play could not answer it. None of it is in the
manuals.

- **THREE OR FOUR down staircases per level, and one or two up** - corrected
  in edition 3, and edition 2 had it wrong. `classic_gen` and `modified_gen`
  are the only two profiles with an `alloc` at depth 1 (90 and 97 in
  `dungeon_profile.txt`), and both call
  `handle_level_stairs(..., rand_range(3, 4), rand_range(1, 2))`
  (`gen-cave.c:1291`). The "exactly one" of edition 2 came from
  `gen-cave.c:1583`, which is the **labyrinth** generator, chosen only by
  hard-coded tests. Each staircase sits on an empty square with **exactly
  three adjacent walls** - a corridor end - relaxed one wall at a time only if
  no such square is free, and at least `MIN(width, height) / 4` = **16 tiles**
  from the others (`gen-util.c alloc_stairs()`). There is still no "near the
  up staircase" or "in the largest room" prior: what there is instead is a
  **shape** (a dead end) and a **count** (three or four), both of which a
  searcher can use. Via `minimaxflash-3`, which found three on three levels;
  `minimaxflash-1.4` had already swept two thirds of a level for the one
  staircase it believed in and found none of them.
- **A level is 198×66 with up to 50 rooms** (`constants.txt`,
  `dungeon_profile.txt` "classic" `params:11:50:200:2`), against a 66×22 view.
  Via `minimaxflash-1`. With the uniform placement above, this is the whole
  explanation of section 1.
- **You start the game standing on the town's own down staircase.**
  `town_gen_layout()` (`src/gen-cave.c`) sets `FEAT_MORE` on the player's
  square and then places the player on it, so the first descent needs no
  search at all - and no run before the eleventh knew it (GAME SOURCE, via
  `minimaxflash-2`).
- **The down staircase is a dead end, at least a quarter of the level away
  from the up staircase.** `alloc_stairs()` (`src/gen-util.c`) wants a floor
  square with **exactly three adjacent walls**, relaxing only if the level has
  none, and `handle_level_stairs()` sets the minimum separation to
  `MIN(width, height) / 4` = 16 tiles. So it is not in the staircase room -
  the up staircase is (GAME SOURCE, via `minimaxflash-2`).
- **Detection marks its whole window as trap-detected, and that stops the
  game's own pathfinder.** `DETECT_TRAPS` sets `SQUARE_DTRAP` on **every**
  square of its 81x45 window, trap or not
  (`effect-handler-general.c:1376`), and `run_step` (`cmd-cave.c`) disturbs
  the player - ending the run or the `g` walk - whenever a step leaves a
  detected square for an undetected one, every step but the first. Two casts
  from different places leave a seam, and every path across a seam dies on it:
  `l>g` to a staircase 36 tiles away made **one or two tiles an ask for about
  a hundred asks** (`minimaxflash-2.2#165`, `#256`). `do_cmd_walk` - plain
  digit movement - has no such test and is immune. **Any program that casts
  detection and then uses the game's travel command inherits this**, and the
  fix is one more cast covering the path, or digits.
- **Only a Mage or a Rogue can find a staircase it has not seen, and the
  arithmetic is brutal.** `DETECT_STAIRS` with `effect-yx:22:40` reads the
  *real* grid and memorises what it finds (`effect-handler-general.c:1467`),
  so it reveals an unseen staircase - in an **81x45 window, about 28% of a
  level**, which takes three positions across and two down, six casts. It
  reports only ground it has not already scanned (two casts from one place
  give nothing), and when no staircase is in the window it says **nothing at
  all** rather than "no stairs". At character level 1 one spell point is about
  **71 player actions** of regeneration (`player_regen_mana`:
  `msp x 197 + 524` in 65536ths, doubled while resting), so six casts and
  their travel are the whole of a 150-decision budget - which is what the one
  Mage run spent, with eleven casts from inside a single 40-tile band (GAME
  SOURCE + ONE RUN, `minimaxflash-2`).
- **A level is regenerated when you leave it.** The depth 1 a character
  returned to after a Word of Recall was demonstrably not the level it had
  left — different corridors, the up staircase elsewhere, and the game's own
  jump to a *known* down staircase found nothing, so the staircase it had used
  was not merely unseen but not remembered (ONE RUN, `minimaxflash-1.3#437`).
  A staircase found is not a staircase an heir inherits — and, the other way
  round, **a map of a level has no second use and position is not
  information.** Nothing in this game asks a player to return to a place:
  stairs are connected, both directions are exits, and `<` then `>` is a
  fresh level of the same depth for two or three decisions. Two runs reported
  "I could not tell where on the level I was" as the thing that beat them;
  the answer to a level that will not yield is another level, not a better
  map.
- **The game's own long walk is worth 17–44 tiles a decision — aimed at
  something already seen.** `find_path` (`player-path.c:1069`) tries a
  known-only path and falls back to one through unknown grids, and the
  targeting cursor is clamped to the level rather than the view, so any
  square can be aimed at. Measured at 170–440 game turns and 17–44 tiles per
  decision (ONE RUN, `minimaxflash-1.3`) — but a later run measured the
  fallback misbehaving: aimed into unexplored ground it assumes unknown
  squares are passable and the reconstructed path can be nonsense, a 20-tile
  aim walking **zero** steps while a 3-tile aim walked three (ONE RUN,
  `minimaxflash-2#90`). The two agree about the game and disagree about what
  was known when they aimed: **aim at a landmark you have seen**, and use the
  nearest-frontier command for the dark.
- **A key string ends at `disturb()`, not at a wall.** `disturb()`
  (`src/player-util.c:1674`) signals `EVENT_INPUT_FLUSH`, and the frontend
  drops the rest of the buffer. What disturbs, measured over one run: a
  **visible monster attacking, hit or miss** (the miss branch calls
  `disturb()` too), picking up gold or an item, opening a door, a failed lock
  pick or disarm. A 24-key chain gave one step while two giant white mice
  were awake, six asks running, with an empty message line because the
  `-more-` erases it (ONE RUN, `mmx-ran-1`). The page's "a digit string stops
  at the first wall" is one way a string ends; this is the other, and it is
  the one that costs asks.
- **What each movement command is worth per ask, measured over 150 asks:**
  `lxg` 1 action; `l` + direction + `g` 3-6 in a room; `.` + direction 1-2 in
  a room but 20+ in a corridor; a digit chain 1-9 and cut by any disturbance;
  the aimed walk `*` `p` digits `g` **9-23**, including through the dark. A
  run stops in an open room (`run_test`), and a diagonal step is refused when
  either orthogonal neighbour is a wall (ONE RUN, `mmx-ran-1`).
- **A reroll of the level is two or three decisions** (`<` then `>`; stairs
  are connected: 3 RUNS). Against a uniform placement, a fresh level is
  exactly as likely to be hard — which makes the comparison *decisions spent
  here* against *decisions spent anywhere*, and nobody has measured it.
- **Magic Mapping is `effect-yx:22:40`**, a window and not a level, 65 gold,
  and the one reading of one on depth 1 showed no staircase (GAME SOURCE +
  ONE RUN, `sonnet-2#47`). **It is nice to have at best, not the instrument
  the corpus treated it as**: 16% of a level, and every staircase found in
  this corpus was found by walking.
- **A level's monsters are few, asleep, and nowhere near you on purpose.**
  `classic_gen` places `z_info->level_monster_min` (14) + `randint1(8)` +
  `k`, with `k = MAX(MIN(depth/3, 10), 2)` = 2 at depths 1-6, each with
  `pick_and_place_distant_monster(c, p->grid, 0, true, depth)` - asleep, at
  any distance. Seventeen to twenty-four placements (before groups) over
  198x66 squares: **an empty corridor is the level, not bad luck** (GAME
  SOURCE; ONE RUN spent twenty asks on an empty depth 2 before reading the
  source, `mmf-mage.2#72`). It is also why a detection spell is worth more
  for its *doors* than its stairs: the doors are the map of where the rooms
  are, and a room is a hundred squares of concentrated search where a
  corridor is ten (`mmf-mage.2`, ONE RUN).

## 3. Where the manuals are wrong, or silent

The corpus' direct corrections. Each cost a run something.

| the manuals say | what the runs found |
|---|---|
| the level feeling is a number to compare against a danger table (strategy manual c15) | the game prints `10 - mon_feeling`, so **the digit on screen is inverted** against that table: higher is worse (GAME SOURCE). A session was told *"naught but cobwebs"* and stayed 4,290 turns (`diego-1`) |
| `M` shows the whole-level map | it shows the **whole level, scaled to 78×22** (`display_map` computes row and column for every square of the 198×66 dungeon), but only the **remembered** squares are drawn in it (2 RUNS, `minimaxflash-1#104`, `minimaxflash-1.2#181`). Two things the manuals do not say, both read out of the source (`minimaxflash-1.5`): a staircase can never be hidden by the scaling - up and down staircases carry the highest terrain **priority** (25; `terrain.txt`), so a remembered `>` always wins its cell and "no `>` on the M map" is real evidence of absence; and positions are recoverable from it, because the panel scrolls in jumps of 33 columns and 11 rows (`verify_panel_int`), so panel columns are not level columns and the M map is the only honest grid |
| the Armoury's prices (c11 tables) | gear is priced by **power**, not by its `cost:` line (`src/obj-power.c: object_value_real`): Leather Shield 66, Studded Leather 150, far above the tables (2 RUNS, `sonnet-3#27`, `minimaxflash-1`). A Soft Leather Armour has `cost:20` and the Armoury asks **84** - and a Priest with 22 gold after its oil cannot have it, so the shopping order decides whether the AC is bought at all (`mmx-pri-1#17`, `#24`, `#67`) |
| Deep Descent is found from depth 1 (our own earlier distillate) | `object.txt` gives it `level:10` with `alloc:20:1 to 100`, and one *was* found and read on depth 1 (`sonnet-2#76`). Which field governs the depth is still unread (`obj-make.c`) |
| a shop's stock is the seed's stock (inferred by a run) | `store_maint()` runs once per game **day** and the shopkeeper shuffles (`store.c:1425`): "the Alchemist had no Magic Mapping this seed" was true of one day only, and a later visit found three at 65 gold under a different shopkeeper (ONE RUN, `minimaxflash-1.2#399`) |
| Word of Recall is not an escape (c15 ESC-04) | right, and the number behind it: the counter ticks in `process_world`, once per ten game turns, so 15+1d20 ticks is **150–340 game turns** — one run gave up after about forty (`qwen-flash-1`). Read in town before ever leaving it, it goes to depth 1, and arriving in town heals (GAME SOURCE + 2 RUNS). **The way to wait it out is one rest**: a full rest is about 61 player actions for a single key, comfortably over the 15-34 maximum, interrupted only if a monster reaches you (ONE RUN, `mmf-mage.2#91`) |
| our own earlier distillate: "one spell point per TEN pounds of armour over the allowance" (a run's correction of the manual) | the manual was right and the run was wrong: `cur_wgt` is in tenths of a pound as well, so `(cur_wgt - spell_weight) / 10` is **pounds** over, one spell point each (GAME SOURCE; the claim is corrected in `agents/minimax/` where it stood) |
| nothing in any manual mentions locked doors | one closed door in four is locked with strength `randint1(7)` (`gen-util.c`), walking into it picks it at `MAX(2, disarm-phys - 4 x strength)` per cent, and the skill is divided by ten if the lock is unseen or you are blind (`player-calcs.c calc_unlocking_chance()`). Three costs no manual states: **a failed pick prints `Cancelled` and eats the rest of that decision's keys** (`minimaxflash-1.4#616`); **a picked lock leaves you outside the door** (`minimaxflash-2.2#162`); and **the game's aimed walk ends on a door and turns into lock-picking**, so a walk aimed at a locked door stops there and you stand at it - a CL1 mage aimed at a door the detection spell had marked on depth 2, failed the pick, and was killed at it (`mmf-mage.3#191`, GAME SOURCE + ONE RUN) |
| a level's monsters are placed asleep (the strategy manual) | only a race with a sleep value is: `place_new_monster_one()` (`src/mon-make.c`) sleeps a monster **`if (sleep && race->sleep)`**, so a race with **`sleepiness:0` is placed awake**. The depth-2 **crow** and **wild cat** are the examples (both speed 120, 9 hit points, AC 14, two blows of 1d3, 8 experience). **Read the race's row before you descend; the placement being asleep is not the race's sleepiness** (`mmf-mage.3#192`, GAME SOURCE + ONE RUN) |
| an unknown item is identified by use, or by a Scroll of Identify (the manuals' model) | **giving it to the store that buys its type identifies the kind for nothing**: `do_cmd_sell()` calls `object_flavor_aware()` and learns the runes *before* the item changes hands (`store.c`), so with `birth_no_selling` on you get 0 gold, lose the item and know every future copy. With selling off this is the only way to identify something you dare not use (GAME SOURCE) |
| the manuals list the mushroom patches | they do not say that the **grey mould is mobile** — no `NEVER_MOVE`, and it chases (GAME SOURCE + `minimaxflash-1.2#174`, `codex-1-tty`, which lost 5 of 21 hit points to spores without ever meleeing it) |
| nothing in any manual describes fear | **fear is -20 to-hit, +8 armour class, and no attacks** (`player-calcs.c`: `OF_AFRAID` adjusts all three; the attack refusal is `do_cmd_walk_test`'s "You are too afraid to attack"). The +8 AC is visible on the sidebar and the -20 is not, and nothing cures it but the timer. Two crows held a druid at 3 hit points for about eight decisions while it could do nothing at all - the fifth death is mostly this row (`minimaxflash-1.5`, GAME SOURCE + ONE RUN) |
| nothing in any manual describes the shrieker mushroom patch | it shrieks when you come near, and the shriek **wakes everything in line of sight** (`monster_spell.txt`: `effect:WAKE`, "shriek for help" - it wakes monsters up, it does not summon new ones). Lethal to stand next to at 3 hit points, and part of the fifth death (`minimaxflash-1.5`, GAME SOURCE + ONE RUN) |
| the birth sheet's `Melee` line is the character's damage | it is not. A Dwarf Priest's birth sheet said `Melee 1d1,+2` while the equipment screen said `a Mace (2d4)`; the birth line is a class default, not the equipped weapon (ONE RUN, `mmx-pri-1#1`, `#23`). Read damage off the equipment screen or `I` |
| a Priest's `ZERO_FAIL` means its prayers never fail | it means the class floor is removed, not the stat floor: the spell list's Fail column reads **5%** for the level-1 prayers, because `adj_mag_fail` for WIS 18 is 5, and only a stat can take it below (GAME SOURCE + ONE RUN, `mmx-pri-1#104`) |
| the depth-1 monster table is the depth-2 table too | it is not, and the difference killed a run: the **silver mouse** has `depth:4` in `monster.txt` and still spawns on depth 2 **in a pack of eleven**, with `MULTIPLY` and an `EAT_LIGHT` blow. Killing them one at a time is how the pack grows, and it eats the torch as it goes (ONE RUN, `mmx-war-1#137`, `#140`) |
| `v` + item + target throws the item | the recipe is right; one run failed it without ever reaching the prompt. It answered the `[` **monster list** with `v` `a` `'`: the list takes one key to close and **eats it**, so `v` (throw) vanished; `a` then ran its own binding, **Aim a wand**, and printed "You have no wands to aim"; `'` ran **Target closest monster** and printed "The apprentice is targeted", which the run read as a ready throw. `g`, Enter and a direction were then ordinary map keys, and it walked into melee (ONE RUN, `mmx-war-1#128`-`#134`; the frames show the message order) |

## 4. Numbers the manuals do not give

- **Armour class decides whether anything connects at all**, in both
  directions. `hit_chance()` (`src/player-attack.c:196`) is
  `12% + 83% × (to_hit − 2 × AC / 3) / to_hit`, so a to-hit below `2AC/3`
  leaves you on the flat 12% floor: at depth 1 a giant black ant and a small
  kobold are AC 24 and need to-hit 16, which a Mage's rapier (9) and its
  thrown oil (11) do not have - that character could not fight most of the
  level and starved for experience instead (ONE RUN, `minimaxflash-2`). Read
  the other way, 200 gold of armour took it from AC 1 to **AC 21**, which puts
  most of depth 1 on its own 12% floor. It needs so
  Grip (AC 36) could not be hurt at range 2 and died at range 1 at about 77%,
  while AC 1–4 targets (worm mass, mould, patch, mouse) are worth throwing at
  from anywhere (GAME SOURCE + ONE RUN, `minimaxflash-1.2`). One flask
  killed each depth-6 `p` human it met (`sonnet-2#124`, `#131`, `#145`).
- **A monster's hit dice are not its damage against you**: Grip's 1d4 bite did
  2–4 against AC 12 (ONE RUN). Armour eats most of a small die.
- **Spell points regenerate at about 0.023 per turn at character level 3**
  (`player_regen_mana`, `player-util.c:487`: `msp × 197 + 524` in 65536ths,
  doubled while resting). One spell point is roughly 22 turns of rest, and the
  Druid's only attack spell costs two: 20 turns of resting bought **nothing**
  (ONE RUN, `minimaxflash-1.2#363`). A class whose weapon arrives at CL3
  cannot pay for it at CL3.
- **A player-cast ball never hits its caster** (`PROJECT_PLAY` is not set:
  `effect-handler-attack.c:594`, `project.c:519`), so radius-2 poison is safe
  at point-blank — and **most depth-1 monsters have `IM_POIS`** and shrug it
  off anyway (GAME SOURCE + ONE RUN).
- **Confusion randomises every direction with 75% probability**
  (`player_confuse_dir`, `player-util.c:1353`) and blocks casting and running
  (`player-util.c:1103`, `cmd-cave.c:1503`). **Resting beside a spore monster
  never clears it** — it re-spores every turn (ONE RUN, `minimaxflash-1.3`). A
  Cure Light Wounds cuts it by 20 turns, and no run has ever drunk one for
  that.
- **Identifying by use pays `(kind level + your level / 2) / your level`,
  rounded down** (`src/obj-knowledge.c`): a whole character level at CL1,
  nothing by CL5 (GAME SOURCE + 2 RUNS). A bought item is already named and
  pays nothing (3 RUNS).
- **Every harmful consumable has `cost:0` in `object.txt` and every useful one
  has a price** (GAME SOURCE), so once a kind is identified its shop price is
  its triage. Selling pays **0 gold** for everything (`birth_no_selling`: 2
  RUNS), which makes the starting Word of Recall scroll not money.
- **The experience ladder, and why the middle levels are slow.** `player_exp[]`
  (`src/player.c`) is cumulative and pre-race-factor: CL2 10, CL3 25, CL4 45,
  CL5 70, CL6 100, CL7 140, CL8 200, CL9 280, applied as
  `exp >= player_exp[lev-1] x expfact / 100`. A kill pays
  `mexp x monster level / your level` with the remainder carried in
  `exp_frac`, so nothing is lost to rounding but **the same monster is worth a
  quarter as much at CL4 as at CL1** (`mon-util.c monster_death()`). A small
  kobold (experience 5, depth 1) pays 5 at CL1 and 1.25 at CL4, so CL1 to CL5
  on depth-1 trash is about 2, then 6, then 12, then 20 kills - **forty kills,
  half of them for the last level**. Stairs, resting, shopping and detection
  pay nothing; the **first cast of each spell** pays its `class` `spell:`
  experience field (a mage's five CL1-CL2 spells pay 18 between them), and the
  first use of an unknown object pays the formula above (GAME SOURCE).
- **Constitution is a rate, not a bonus.** `calc_hitpoints()` is
  `player_hp[level-1] + adj_con_mhp x level / 100`, floor `level + 1`, and a
  level adds `randint1(hit die)` (`player-birth.c roll_hp()`). `adj_con_mhp`
  runs from **-250 at CON 3** through 0 at 11 to **+1250 at 18/200**, i.e.
  -2.5 to +12.5 hit points per level, so two characters of one class and level
  differ by ten hit points and the gap widens with every level (GAME SOURCE).
- **Armour buys misses, not padding.** A blow that lands loses
  `damage x min(AC, 240) / 400` (`mon-attack.c adjust_dam_armor()`) - 2.5% at
  AC 10 - while AC is the defence value in the hit roll, which is where its
  value is. A Potion of Berserk Strength heals 30, gives +75 to-hit and fear
  immunity and costs 10 AC, which at these depths is a bargain (GAME SOURCE +
  `minimaxflash-3`).
- **What an unknown item can be is fixed by the depth it was found at**, and
  the pool is small: at depth 2 an unknown scroll is one of **eight** kinds
  (Treasure Detection, Phase Door, Light, Detect Invisible, Blessing at weight
  50; Deep Descent 20; Summon Monster and Darkness 10), so Phase Door 17%,
  Deep Descent 7%, and 7% actively harmful; an unknown potion is one of
  **eleven**, 84% harmless or good. **Magic Mapping cannot appear before DL5**,
  the shallowest wand is DL3 and the shallowest staff, rod, mushroom and ring
  DL5, so on depth 1 and 2 the only unknown kinds in the game are scrolls and
  potions - except for the **one object in twenty** generated as if from a
  deeper level (`obj-make.c`, `one_in_(great_obj)` with `great-obj:20`, which
  recomputes the level as `1 + level x 100 / randint1(100)`) (GAME SOURCE).
- **A Warrior can never dig**: `skill-dig:0:0` and no points a level, against
  `calc_digging_chances()` needing over 10 for magma and over 20 for quartz.
  So every vein is a permanent wall for that class, and a `*` - a vein **with
  treasure** - is money it cannot have (GAME SOURCE + `minimaxflash-3`).
- **The status line's `Light N` is the light of the square, not the torch's.**
  `ui-display.c prt_light()` prints `square_light(cave, player->grid)`: 1 is a
  dim corridor, 2 a lit one, 3 an open room, 0 darkness, and the town reads 1.
  It changes as you move and without the torch burning (ONE RUN,
  `mmx-pri-1.2`). It also sets the view: Light 2 is radius 1 (3x3) and Light 3
  radius 2 (5x5) (`cave-view.c calc_lighting()`), so a torch in a corridor
  shows only the squares around you (ONE RUN, `mmx-ran-1`).
- **A Priest gets exactly one blow a turn, whatever it holds.**
  `calc_blows()` divides `adj_str_blow[STR]` by `max(weapon weight, class
  min-weight)`, and the Priest's min-weight is 35 with strength-multiplier 3,
  so the index is 0 for every weapon and the table gives 100 energy a blow.
  Melee is not a fallback for this class, it is a rounding error (GAME
  SOURCE + ONE RUN, `mmx-pri-1`).
- **Thrown oil's to-hit is `35 - distance`**, so the distance is the thing to
  manage: at one tile a Priest hits AC 24 about 56% of the time and AC 36
  about 36%, and every tile further is worse. The class's throw skill (35 for
  a Priest) is the ceiling, not the roll (GAME SOURCE + ONE RUN,
  `mmx-pri-1#82`).
- **Digging repeats until the wall is gone, for one decision.** `T` plus a
  direction keeps attempting while the game is not disturbed, and a Dwarf's
  `SKILL_DIGGING` is race 40 + `adj_str_dig[STR]` + `weapon_weight / 10` = 61
  at STR 18, so quartz falls at 6.8% an attempt and one ask is a hundred
  attempts. A vein gave **90 gold in three decisions** (ONE RUN,
  `mmx-pri-1#52`, `#55`, `#56`). The same command tunnelled a corridor through
  four veins and a granite wall on the next level (`mmx-pri-1#130`-`#145`).
- **Resting is not a plan inside a decision budget.** `R` then Enter rests
  until healed in one decision, but it healed **3 hit points in 100 turns**
  (ONE RUN, `mmx-war-1#105`); the rate, not the command, is what makes it
  useless mid-run.
- **A breeding monster is a tax of one to two hit points on every decision.**
  Silver mice `MULTIPLY`, and a 38-tile walk with the nest at your back cost
  **22 hit points in one decision** (ONE RUN, `mmx-pri-1#81`, `#82`). A
  character with 24 hit points and fifty decisions left cannot pay it: kill
  the nest in a corridor where only one can reach you, or leave the level.

## 5. What the players did with all this, which is the behavioural finding

- **Escapes bought are escapes unused.** Every run that bought Phase Door and
  Cure Light Wounds still held them at the end, and the run that died was
  holding six and five of them (3 RUNS). The purchase was right; reaching for
  it never happened.
- **The one escape that was used, worked.** Phase Door broke away from a
  +10-speed unique twice in one run, for one decision each
  (`minimaxflash-1.2#332`, `#345`).
- **The town is not free.** A street urchin took 8 of 26 gold and a cutpurse
  lifted a flask (`minimaxflash-1.2#394`); the next life of that character
  arrived with 3 gold and could not reach a shop at all, which made the
  previous life's own advice — buy the 65-gold Magic Mapping — unactionable
  (`minimaxflash-1.3`). The corpus' second death was in the town
  (`qwen-flash-1`).
- **A model can hold the knowledge and not play.** `qwen-flash-1` had the same
  pages and notes as the run that reached depth 7, and spent 29 of 73
  decisions on birth and shopping and the rest walking around the town until a
  dog killed it.
- **Players fixate on the item they want rather than the pool they have.** A
  run on depth 2 made "what does Magic Mapping show" its standing question and
  walked to two unknown scrolls to answer it. Magic Mapping cannot be an
  unknown scroll on depth 2 at all (section 4): the pool it was drawing from
  was 17% Phase Door and 7% harmful. The same shape appears in the deaths:
  **the plan was written around the best case of an unknown, not its
  distribution.**
- **A warning already written down is not a warning read.** The page these
  runs read says, with the source behind it, that Word of Recall is not an
  escape and that its wait is 150 to 340 game turns. The fourth death is a run
  that read the Recall and stood still for forty turns. A line in a document
  the player has open is not the same as a line the player applies - which is
  the strongest argument in this corpus for *checked* knowledge over more
  knowledge.
- **A long string is a bet on a screen that has not changed.** The two runs
  that sent sixteen and sixty keys died inside them (section 6), and both had
  seen what they were walking into. Since 2026-11-10 the game's own flush ends
  the string at a disturbance (D17), so the harness no longer forces the bet -
  but the run that sent the keys still spent them, and a player that has not
  learned to stop has only had the decision taken away from it, not made.

## 6. How the seven deaths happened

Not one of them was a big hit, none was at a depth the character had no
business being at, and **none is the kind of death a hit-point threshold
prevents** — which is the part worth carrying into a death analyser. Four
were at character level 1, one at 2 and three at 4; the deepest was depth 2.

- **`sonnet-3`, Human Mage, 10 hit points, depth 1, turn 1,348, small
  kobold.** A wounded kobold was believed dead because its health bar went
  blank (it means *not in view*); then a gas trap confused the character; then
  it sent sixteen hold keys in one decision and died inside them. Five Cure
  Light Wounds and six Phase Door in the pack, unused. It had also chosen
  detection over Magic Missile as its single CL1 spell, so it had no attack
  but thrown oil.
  *The failure is state-tracking plus an uncancellable batch, not target
  selection.*
- **`qwen-flash-1`, High-Elf Druid, 13 hit points, depth 0, turn 570, scruffy
  little dog** (2 hit points, biting for 1d1). It never left the town: it spent
  sixty decisions hunting for shop doors while the town's animals clawed it
  from 13 to 1.
  *The failure is that it never played.*
- **`minimaxflash-2.2`, Gnome Mage, 16 hit points, depth 2, turn 10,811,
  metallic green centipede** (1d1 and 1d1 at speed 120 - two damage a round).
  It stood at 4 of 16 hit points with three monsters adjacent, throwing oil at
  a unique with AC 36 - where its to-hit of 11 is on the game's flat 12% floor
  - with a Scroll of Phase Door in the pack it had carried all game and did
  not read until 2 hit points (`minimaxflash-2.2#350`).
  *The failure is that the escape was never spent, against a fight the
  arithmetic said it could not win.*
- **`minimaxflash-3`, Human Warrior, 43 hit points, constitution +5,
  character level 4, depth 2, turn 8,394, cave spider.** The fattest character
  in the corpus, killed by a 7-hit-point spider doing 1d4: it read a Scroll of
  Word of Recall on arriving at a fresh level and then **stood still for forty
  turns** waiting for it, not knowing the timer ticks once per ten game turns
  (`minimaxflash-3#149`).
  *The failure is an action with a 150-turn cost taken as though it were
  instant. Hit points buy mistakes; they do not buy standing still.*
- **`minimaxflash-1.5`, High-Elf Druid, 33 hit points, character level 4,
  depth 2, turn 29,966, an acolyte.** The richest death in the corpus: it
  arrived with no Phase Door and no Word of Recall in the pack, crows
  frightened it - **fear is -20 to-hit and no attacks at all**, so for about
  eight decisions it could do nothing - a shrieker mushroom patch woke the
  corridor, and at 3 hit points with 118 gold its last three decisions
  walked toward a second gold pile instead of the up staircase
  (`minimaxflash-1.5#892`). Its own verdict: the escape was spent late or
  not at all, and "the right moment is when the fight is unwinnable, not
  when the character is dying".
  *The failure is the condition nobody had read: fear takes the attacks
  away, and a pack with no escape in it is a decision made in the town that
  kills you on depth 2.*
- **`mmx-war-1`, Half-Elf Warrior, 35 hit points, character level 4, depth 2,
  turn 10,923, a silver mouse.** The sixth death and the second of its shape:
  it saw **eleven** silver mice on the panel at ask 140 and sent a 60-key
  beam chain into them anyway (`mmx-war-1#140`). It killed about thirty-five
  of them - reaching character level 4 mid-slaughter - and the pack replaced
  its dead; the recorded frames show `*** LOW HITPOINT WARNING! ***` at frame
  33 and a soldier ant joining at 34, and the string still had keys left. It
  died at 35/35 to dead, with a Word of Recall in the pack that needed 150
  turns it did not have, having spent its only Phase Door surviving a
  59-key chain into a grey mould at 4 hit points twelve decisions earlier
  (`mmx-war-1#104`, `#109`). No Cure Light Wounds was bought at all, against
  its own family's rule of "one heal, one escape, then oil".
  *The failure is an uncancellable batch aimed at a breeder: killing a
  `MULTIPLY` monster one at a time is how the pack grows, and the string was
  spent before the warning could be read. It is the death that decided T11.*
- **`mmf-mage.3`, Gnome Mage, 8 hit points, character level 1, depth 2, turn
  8,019, a crow.** The seventh death, and the one with the clearest order of
  mistakes. **It should not have been on depth 2 at all**: it descended at
  CL1 for the doubled experience, with 8 hit points and 1 of 2 spell points,
  onto a depth whose crow does two bites of 1d3 at speed 120 - the table
  says two rounds is 10 or more against 8 hit points. At the junction
  it met - the crow in the north doorway, a person in the south one, the
  **up staircase five tiles west** - the right move was to retreat to the
  staircase it had just come down; instead it aimed the game's walk at a
  **locked door** east, failed the pick, and stood there while the crow bit
  twice a turn. Five bites landed and it died at -2/8 with **11 experience,
  one short of CL2**; it had 2 Phase Door scrolls, 4 Cure Light Wounds and
  23 flasks of oil and used none (`mmf-mage.3#191`, `#192`).
  *The failure is in this order: a CL1 dive it could not pay for, no retreat
  to the up staircase when awake monsters appeared, and a walk aimed at a
  door rather than open ground. The door is a better escape than a wall - it
  is passable - but a clear path is what the walk should target.*

## 7. The classes, where play said something data does not

- **Warrior** (6 runs, the deepest): the only class past depth 2 here. Oil is
  the whole ranged game and that was enough to depth 7. It **can never dig**
  (section 4), so veins and the treasure in them are scenery for it, and
  armour is a poor buy for a class that can fight (2.5% of a blow at AC 10)
  while a Potion of Berserk Strength is a good one. Two died: one of standing
  still at 43 hit points, one inside a 60-key chain at 35 - neither of being
  outmatched.
- **Druid** (6 runs, five of them one character, all seed 23): 13 hit points
  and a 1d1+1 whip at CL1 — five blows to kill a 10-hit-point worm — and the
  mana economy above makes its CL3 spell unpayable. The fifth life reached
  depth 2 and put the number on the alternative: **Stinking Cloud killed an
  acolyte for 2 spell points and no hit points, where the same kill with the
  whip costs about thirty** - damage per hit point spent is what decides a
  druid's fight, and the spell is about five times better (`minimaxflash-1.5`,
  ONE RUN). It also learned what kills druids: crows. Section 6 has the death.
- **Mage** (5 runs, three died): one spell at CL1, and the choice of which
  decided whether it had an attack at all. The second run settled the choice:
  **Magic Missile first** - 1 spell point, `$Dd4` with the dice count
  `(CL-1)/5+3`, so **3d4 (3 to 12) from CL1 to CL5**, never resisted, which is
  the only damage in the game that ignores armour class. Studying costs one
  action and nothing else, so take every slot the level offers. Its melee is
  not a fallback: to-hit 9 with the kit rapier is on the 12% floor for most of
  depth 1. The fourth run chose the other way on purpose - **Find Traps first,
  because the staircase is the gate to the experience** - and it worked
  exactly as far as the spell did: the cast found two down staircases per
  level and the walks were four asks. What it could not do was fight: its
  bottleneck was finding anything it could kill at 8 hit points
  (`mmf-mage.2`, ONE RUN; it left a room of three sleeping cave spiders
  unfought, correctly). The two choices are not a disagreement: take Magic
  Missile when you can win fights, detection when you cannot. Three costs the
  third life paid for: **the cast menu's rows are the spells you have
  studied** while the screen prints the whole book, and a letter moves the
  cursor rather than selecting (a second press or Enter selects), so
  "casting" an unstudied spell casts the studied one instead (`mmf-mage.3`);
  **rest to full spell points whenever the ground is safe** - a mage's bar is
  its attack, and 1 of 2 points is not a bar; **do not dive to depth 2 at
  CL1**, because the doubled experience is not worth a round you cannot
  survive; and the depth-2 **crow and wild cat act twice a turn** (speed
  120, two blows of 1d3) - read their table rows before you descend (section
  6 has the death).
- **Priest** (3 runs: the person, and two lives of one Dwarf): **no prayer
  before CL5 detects anything**, and prayers are given at random rather than
  chosen (`class.txt`), so a Priest has no magical route to a staircase —
  which, given section 1, is the class's real problem rather than its combat.
  The Dwarf is the best-bodied and worst-armed Priest the game makes: 24 hit
  points at CL2, +1.75 a level from CON 19, **one blow a turn whatever it
  holds**, and thrown oil at `35 - distance` as the whole attack (section 4).
  Its AC is what it buys (Robes to Soft Leather is AC 3 to 9), and its one
  real income is **digging**: a Dwarf's skill turns a treasure vein into 90
  gold in three decisions. Life 1 reached CL2 and depth 1 with 410; life 2
  reached **CL3 and 625**, bought back the escape ladder life 1 had spent,
  and found no staircase on two different depth-1 levels (15% then 1% of the
  map). **That did not matter: CL3 on depth 1 is where the first levels
  are** - the staircase is for later, not the objective. Its own report's
  lesson is the mouse tax - a 38-tile walk with a breeding nest at its back
  cost 22 of its 24 hit points in one decision (`mmx-pri-1#81`, `#82`) - and
  its measurement is the `Light N` row in section 4.
- **Ranger** (1 run, the first Hobbit): 12 hit points, AC 2, melee 1d5 at
  to-hit 18 with 1.3 blows, **shoot to-hit 32 with 1.0 shot a turn**, disarm
  59%, devices 48%, search 22%, stealth Excellent. At CL1 the Short Bow is
  **worse than the dagger in damage per turn** (1d4 x2 = 2-8 against 1d5 x1.3)
  and the FAST_SHOT flag only adds shots at CL3, so **thrown oil is the
  Ranger's real weapon** (9-36 a flask at range ten). It reached CL1 and
  depth 1 and stopped alive with 724 carried, and its report is the corpus's
  movement measurement (section 2): the aimed walk 9-23 tiles an ask, `lxg`
  1, the run 20+ in a corridor but 1-2 in a room, and a digit chain cut by
  any disturbance.
- **Blackguard, Paladin, Ranger, Rogue and Necromancer have never been played.**

## 8. What this corpus cannot answer

Read these as open. A consumer's own evidence would be the first.

1. **Is descending even the right plan at character level 1-4?** Nobody has
   tested the alternative: clear a level, take the experience and the gold, go
   back up, buy armour, and descend when the character can pay. The one run
   built to level (`mmf-mage.2`) found the plan's hidden flaw: a CL1 kill on
   depth 1 pays 1-5 and the monsters are two dozen placements over a level
   the size of four football pitches, so the experience is gated by the
   staircase, not the killing. What would settle it: two runs on one seed,
   one diving, one levelling first, compared on where each stood at ask 150.
2. **Can a staircase be found by coverage?** Two runs have tried. One spent
   140 decisions sweeping a depth-1 level, opened fifteen locked doors and
   mapped two thirds of the squares *by its own estimate* - and **found no
   down staircase in any of it** (`minimaxflash-1.4#753`), on a level the
   generator gave three or four. The other measured the M map's coverage as
   the instrument: **16% after a Magic Mapping scroll, 26% when the `>` at
   last appeared**, 99 decisions from arriving to descending
   (`mmx-war-1#22`, `#92`). The honest reading is that the estimate was
   wrong: `M` draws the squares you remember, and a band of columns you
   crossed is not the squares inside it. What is still open is the
   measurement nobody has made: the share of a level's squares actually
   remembered after N decisions, which the replay of either run could be made
   to count.
3. **Explore or reroll?** Two or three decisions against perhaps a hundred,
   with a uniform prior either way. Unmeasured, and it is the cheapest
   experiment left: the same seed, one run that rerolls on arrival and one
   that explores.
4. **Does Magic Mapping ever show a staircase?** Three readings, nothing in
   radius: `minimaxflash-1.4`, `sonnet-2#47` and `mmx-war-1#22`, whose window
   was 44 rows x 80 columns - 16% of the level - with no `>` in it. **It is a
   convenience, not a plan**: the runs that found staircases found them by
   covering the level.
5. **Can a Mage's detection be aimed at all** without knowing its own
   coordinates? Six casts cover a level; eleven from one band cover one
   window.
6. **What governs Deep Descent's depth**, `level:` or `alloc:`? One was found
   on depth 1 against `level:10`.
7. **Is depth 5–20 survivable by a player of this kind?** Nobody has been
   below depth 7; our own volume 2 is written entirely from game data.
8. **Do the strategy manual's numeric policies hold anywhere?** They are
   explicitly "a starting policy to be measured", and this corpus measured
   them at depth 0–2.

## What is deliberately not here

The consumer reads the manuals and the data itself, so this file leaves out:

- **anything the game's manual or the strategy manual says** — the command
  set, the spell lists, prices, kits, monster tables, the escape ladder;
- **anything in `lib/gamedata` that a reader can look up** — we quote a data
  file only where a run's conclusion depends on it;
- **what is only true of reading a text screen**: where a keypress is lost to
  a `-more-`, that menu letters are labels rather than positions, that the
  glyph under `@` is not drawn, that a blank health bar means *unseen*. Those
  are real and they are in `manuals/INSTRUCTIONS.md` for the players here;
  they are not knowledge about Angband, and a program with the game's own
  state does not have those problems. Where one of them caused a death it is
  in section 6, because the death is the point.

## What this repository is, in one paragraph

A language model plays Angband 4.2.6 from an 80×24 screen, one keypress at a
time, with no state extraction and no primitives: the screen is the state and
the keyboard is the action. A run is a seed plus its keys, so `replay`
reproduces it byte for byte and `carry-on` continues a stopped character;
every run's keys, decisions, notebook and report are in `records/`. The game
is built with `-DSUPPORT_BORG=OFF`, so nothing in the binary can play for the
model. What the runs establish is therefore about a *reading* player, which is
the one thing a Borg's own games cannot produce.

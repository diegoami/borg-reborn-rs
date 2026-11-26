# ACCUMULATED BEHAVIOUR — the characters worth imitating, and what they did

Edition 4, 2026-11-10. *(Edition 1 was written the same day. Edition 2 added
the first Priest and the sixth death. Edition 3 added the mage lineage's
second archived life and its death at a locked door. Edition 4 adds the
Priest's second life - CL3, the best Priest so far - and the first Ranger,
whose report is the corpus's movement measurement.)*

**Who this is for.** [borg-reborn-rs](https://github.com/diegoami/borg-rs),
the Rust Borg being rebuilt, and anyone else outside this repository. It is
the companion to [ACCUMULATED_KNOWLEDGE.md](ACCUMULATED_KNOWLEDGE.md): that
file is what the corpus *knows*; this one is *who played well enough to copy*.
No player reads either (D16 - the export aggregates what a player is denied).

**The criteria, stated because they are a decision, not a default:**
**character level and net worth** (gold plus the game's own valuation of what
the character carries). Maximum dungeon depth is recorded and is **not** a
success criterion: twenty-one runs chased the down staircase and the one thing
they never questioned was the chasing. Depth appears below as context only.

**The unit is the character, not the run.** A character can live several
lives (`carry-on` replays the earlier keys into the next generation, with
the notebook inherited), and a lineage is named after it: `minimaxflash-1`,
`minimaxflash-1.2`, `minimaxflash-1.3`, ... is one character's first, second
and third life. Every entry below gives the whole lineage, because what a
life did is only readable against what it inherited.

**The caveats, stated because they are the difference between this and
astrology:** the corpus is twenty-one runs. Every shape below is one character,
on one seed, by one model - a lead, not a result. Nothing here is true of
any seed but its own, and the seed-specific parts are left out. Where a run
was lucky, the luck is named; what you imitate is what it did with it.

Each entry: the lineage, the numbers, what it did right, what it did wrong,
and the shape to copy, with `run#ask` citations into `records/`.

---

## sonnet-2's character — two lives: `sonnet-2`, `sonnet-2.2` (Half-Troll Warrior)

**Character level 8, worth 2,025, alive.** The most successful character in
the corpus on both criteria, and the only one built openly across two lives:
the second life inherited the first's notebook and took the character from
level 6 to 8.

**What it did right.** The first life treated an unknown scroll as a pool
with odds rather than a risk to avoid, and one read - a Deep Descent found
on DL1 (`sonnet-2#76`-`#79`) - opened the depth-6 economy the second life
lived on. The second life moved by the game's own routing and nothing else:
`lxg` as a free autoexplore hop, `l>g` to a known down staircase, `l<g` back
up, each alone in its ask because a key after `g` cuts the walk to one step
(`sonnet-2.2#156`-`#164`, measured both ways). Its descent to depth 7 was
three asks (`sonnet-2.2#228`-`#230`). It rerolled bad levels for two asks
instead of exploring them for a hundred (`sonnet-2.2#231`-`#232`), and it
read the terrain after every dead end, which is where its DL6 staircase was
hiding.

**What it did wrong.** The second life aimed an unknown wand at a sleeping
tamer and it was Haste Monster: about 15 asks and 50 hit points to survive
its own test. And neither life wrote a report on time - `sonnet-2.2`'s
summary is reconstructed from its decisions and notebook.

**The shape to copy:** inherit notes; move by the game's routing only;
reroll instead of grind; unknown consumables are a priced bet - know the
pool and pay it when the downside is survivable; take the escalator when
the dungeon offers one.

[life 1: sonnet-2](records/sonnet-2/REPORT.md) ·
[replay](https://diegoami.github.io/borg-runner/records/sonnet-2/report.html) ·
[life 2: sonnet-2.2](records/sonnet-2.2/REPORT.md) ·
[replay](https://diegoami.github.io/borg-runner/records/sonnet-2.2/report.html)

## sonnet-1 — one life (Half-Troll Warrior)

**Character level 3, worth 723, alive.** The first run of all, with nothing
inherited. It never found a down staircase - and it is on this list because
it did everything else right.

**What it did right.** Shopped with a plan (15 oil, 3 Phase Door, 2 cures,
asks 14-22). Killed Fang at CL2 in a corridor with two thrown flasks and
melee, never below 20 HP (asks 81-90). Rerolled a twisty level by the
connected stairs (ask 67). Tested an unknown potion at full health and
levelled off it. **Surviving is the floor every dead run fell through.**

**What it did wrong.** It rerolled thirty asks late (its own verdict: the
cave was twisty at ask 35). It walked into unlit rooms without looking and
paid 13 HP to a wild cat.

**The shape to copy:** the opening. A planned shopping list, oil as the
ranged attack, escapes before depth, and no fight you do not have to take.

[report](records/sonnet-1/REPORT.md) ·
[replay](https://diegoami.github.io/borg-runner/records/sonnet-1/report.html)

## minimaxflash-1's character — five lives: `minimaxflash-1`, `1.2`, `1.3`, `1.4`, `1.5` (High-Elf Druid)

**Best life: character level 4, worth 382.** The corpus' longest
apprenticeship: one character, five lives, all on seed 23, each one
inheriting the last one's notes. Read it as one learning curve rather than
five runs.

- **Life 1** (`minimaxflash-1`) spent 148 asks driving a 143-line
  breadth-first search it wrote, reached depth 1, and never tried the
  game's own commands. The corpus' cautionary tale about writing tools.
- **Life 2** (`minimaxflash-1.2`) found the staircase life 1 missed - it
  was on the map the whole time, one step from where life 1 stopped. One
  `l` on the right square (*"You see a down staircase, 1 S, 1 W"*,
  `minimaxflash-1.2#161`) is the cleanest demonstration in the corpus that
  the game's own commands answer what custom tools do not.
- **Lives 3 and 4** (`minimaxflash-1.3`, `minimaxflash-1.4`) reached CL4
  and stayed poor, and life 4 spent about 140 decisions sweeping two thirds
  of a level for a staircase it believed was unique - the measurement that
  forced the three-or-four-staircases correction.
- **Life 5** (`minimaxflash-1.5`) was the best player of the five and the
  one that died: it found the staircase deliberately, reached depth 2,
  measured the M map's geometry (the whole level scaled; a staircase can
  never be hidden on it), and then met crows with no Phase Door and no
  Recall in the pack - fear took its attacks away and it died collecting
  gold at 3 hit points (`minimaxflash-1.5#892`). Its report is the richest
  single document of the lineage.

**What it did right, as a lineage:** every life wrote down what the next
one needed, and the next one measurably played better. **What it did
wrong:** it never solved what it is - a Druid with a whip and 13 hit
points, whose real weapon (Stinking Cloud, CL3) it kept not having in the
fight that mattered.

**The shape to copy:** the inheritance discipline, and life 5's measurement
habits. Not the ending.

[life 2](records/minimaxflash-1.2/REPORT.md) ·
[life 3](records/minimaxflash-1.3/REPORT.md) ·
[life 4](records/minimaxflash-1.4/REPORT.md) ·
[life 5](records/minimaxflash-1.5/REPORT.md) ·
[replay, life 5](https://diegoami.github.io/borg-runner/records/minimaxflash-1.5/report.html)

## minimaxflash-3 — one life (Human Warrior)

**Character level 4, worth 903, died (cave spider).** The best wealth of
any CL4 run, included because it did more right than wrong - and because
its one wrong is the most instructive decision in the corpus.

**What it did right.** The whole middle of the run: gold found and taken,
a kobold shaman killed for the level, locked doors treated as rolls not
walls, and a measured correction of the corpus' own Magic Mapping
arithmetic while it played (`minimaxflash-3#112`).

**What it did wrong.** Ask 149: it read a Scroll of Word of Recall on a
fresh depth 2 and sent forty stand-still keys in one decision. The scroll's
minimum wait is 150 game turns; it died at turn 8,394, **110 turns after
reading, 40 before the scroll could have fired**, while standing on a down
staircase - a one-action escape from the spider it never weighed, because
it wanted the town and the way up was unknown (`minimaxflash-3#149`).

**The shape to copy:** everything except the ending - and the ending is
the lesson that a scroll with a timer is not an escape, and that the
staircase under your feet is always an exit in one direction.

[report](records/minimaxflash-3/REPORT.md) ·
[replay](https://diegoami.github.io/borg-runner/records/minimaxflash-3/report.html)

---

## Not on the list, and why

- **`mmf-mage`'s character** (two archived lives: `.2` CL1, 576, alive;
  `.3` CL1, 645, died to a crow) - the cleanest process and the emptiest
  outcome. `.2` bought perfect safety and reached character level 1: a CL1
  mage's problem is *finding* kills, not fighting them. `.3` did the
  arithmetic right - turned a Yellow Potion into 5 experience, found the
  staircase with a detection cast - and then acted on the wrong conclusion:
  it dived to depth 2 at CL1 for the doubled experience, with 8 hit points
  and 1 of 2 spell points, met a crow (two bites of 1d3 at speed 120) at a
  junction, and spent the turn on a locked door instead of the up staircase
  five tiles west (`mmf-mage.3#191`, `#192`). Read the pair as one lesson: the plan can
  be right everywhere except the gate - **do not enter a depth whose worst
  round you cannot survive, retreat to the staircase you came down when
  awake monsters appear, and rest to full spell points whenever the ground
  is safe.** A door is a better escape than a wall, but a clear path is what
  the walk should target.
- **The other three deaths** (`sonnet-3`, `qwen-flash-1`,
  `minimaxflash-2.2`) - each did more wrong than right by the only vote
  that counts. Their deaths are section 6 of ACCUMULATED_KNOWLEDGE.md;
  that is where they teach.
- **`mmx-war-1`** (CL4, 637, died to a silver mouse) - **right for 139
  decisions and wrong once.** It spent every coin on oil and a Magic Mapping
  scroll, read the scroll to map **16% of the level** - a convenience, not the
  answer: the staircase appeared seventy asks later, by walking - wrote
  its own beam-search planner, found and fixed a real padding bug in it, and
  pushed the M map's coverage from 16% to 26% until the staircase appeared
  (`mmx-war-1#22`, `#92`). Then it saw **eleven** silver mice and sent a
  60-key chain into them: it killed about thirty-five, the pack replaced
  them, and it died inside a low-hit-point warning it could not answer
  (`mmx-war-1#140`). It is the corpus' clearest case of a character that had
  learned the game and not yet learned to stop - and the death that made the
  harness let the game end a key string (D17). Read it as a warning, not a
  model: the run is worth copying up to ask 139 and not one key past it.
- **`mmx-pri-1`'s character** (two lives: `.1` CL2, 410; `.2` CL3, 625, alive)
  - no shape yet, but the most useful *class* document in the corpus: the
  first Dwarf and the first model Priest, one blow a turn whatever it holds,
  thrown oil at `35 - distance`, and an income in digging (90 gold from a
  vein in three decisions). The second life is the better one: it bought back
  the escape ladder the first had spent and reached **CL3 on depth 1 without
  a staircase** - which is the plan, not a failure: the first levels are on
  depth 1, and the staircase is for when the character can pay for the depth.
  The play is not yet worth copying, but the inheritance is: **a life that
  spends its escapes leaves the next one poorer, and the next one must buy
  them back before anything else.**
- **`mmx-ran-1`** (CL1, 724, alive, depth 1) - no shape yet: it reached
  character level 1 and found no staircase. It is here for its *measurement*,
  not its play: the aimed walk 9-23 tiles an ask, `lxg` 1, the run 20+ in a
  corridor but 1-2 in a room, and a digit chain cut by any disturbance - a
  visible monster's attack, a pickup, a door. Read it before you plan a
  movement budget; do not read it for how to fight (at CL1 its bow is worse
  than its dagger, and thrown oil is the Ranger's real weapon).
- **The CL1 survivors** (`codex-1-tty`, `minimaxflash-2`) - alive, level 1:
  no shape worth copying yet.
- **`diego-1`** (CL2, the person) - the control, not a candidate: it saved
  early and sold nothing, so its numbers say little about play. Its value
  is that a person and a model made the same decisions on the same ground.

## How to read this against the export

The knowledge file says what is *true*; this file says who *used it well*.
A consumer that can hold both should notice the correlation the corpus
runs on: the characters that levelled and got rich are the ones that spent
their decisions through the game's own commands, and the ones that died or
stagnated spent theirs fighting the interface or standing still.

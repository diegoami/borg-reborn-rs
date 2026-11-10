# minimaxflash-1 — High-Elf Druid, seed 23, depth 1, level 1

*Model: `minimax-coding-plan/MiniMax-M3.1-Flash`, played by opencode. Seed 23
(chosen because `records/OVERVIEW.md` only held seeds 7 and 11).*
*Files I opened before playing: `manuals/INSTRUCTIONS.md` (all of it),
`knowledge/NOTES.md` (all of it), `records/OVERVIEW.md`, `PLAYGAME.txt`,
`CONTINUEGAME.txt` (to confirm it did not apply), and the game's own data
files under `vendor/angband/lib/gamedata/` and sources under
`vendor/angband/src/` — the latter only to answer questions the manual left
open, never another run's decisions.*

## What happened

The character was drawn, not chosen: `*` at the race menu and `*` at the class
menu produced a **High-Elf Druid** (hit die 9 + 2 + CON = 13 HP at CL1, SP 2/2,
melee `1d1,+1` with the starting whip, 1.0 blow/turn). That is a poor
first-level fighter: the whip needs five blows to kill a 10 HP worm, so oil had
to be the whole attack plan. (The draw is seed-determined: the other seed-23
run in the overview, `qwen-flash-1`, also drew a High-Elf Druid.)

**Shop.** In town I bought 39 flasks of oil (90 gold), a cloak (+1 AC, 6), a
Soft Leather Armour (+8 AC, 84) and a Metal Cap (+3 AC, 24) — **AC 12** — plus
4 Cure Light Wounds, 3 Phase Door scrolls and the kit's Word of Recall, for 161
gold spent of 382. Armour and the cap had to be *worn* by hand (`w` + the pack
letter); bought gear is not auto-worn and the sidebar still said `Cur AC 0`
until I did it. The Druid's 35 lb spell-weight allowance is not an issue at 8 lb
of armour, and the pack stayed under the ~108 lb burden limit (39 flasks = 78 lb).

**On depth 1 (7,433 turns, 148 asks, 1,081 keys).** I never found the down
staircase. I mapped a large part of the level's eastern third, killed six
monsters for 13 of the 14 experience units CL2 needs, and finished **alive, at
full 13/13 HP, on depth 1, one experience unit short of level 2** when the
150-decision budget ran out.

Kills: two white worm masses (in the worm room), a giant white centipede, a
fruit bat, a floating eye and a giant white mouse. The first Detect Life cast
paid the other 2 units. The worm room cost me about 9 HP and ten asks.

## What I learned that a next run should use

These are the things that changed how I played, with the evidence.

1. **Targeting mode is a pathfinder: `*`, `x`, `g`.** In the targeting screen,
   `x` jumps the cursor to the nearest *known* square that touches unexplored
   space and `g` walks the shortest path there (source:
   `vendor/angband/src/ui-target.c`, `event_is_key(press, 'x')` and
   `event_is_key(press, 'g')`). The manual called this "UNTESTED by any run";
   it works, and `>`/`<` in the same mode jump to the nearest **known** down/up
   staircase (`path_nearest_known(..., square_isdownstairs, ...)`).
2. **The level is much bigger than the screen.** `world:dungeon-wid:198`,
   `world:dungeon-hgt:66` (`constants.txt`), and the main map window is only
   `SCREEN_WID = 80-13-1 = 66` columns by 22 rows (`ui-term.h`). A level is
   about three screens wide and three screens tall — which is why "no run has
   ever found the stairs on depth 1" is a level-size problem, not a
   search-technique problem. My own exploration covered the eastern third
   in 148 asks.
3. **`M` shows the *remembered* map, not the level.** The full-level map draws
   the whole level scaled down, but unexplored squares are blank, so it is a
   way to check whether a staircase has scrolled out of my window (I used it
   twice: asks 104 and 146) — it is not a shortcut. The manual's "M whole-level
   map" reads as if it revealed the level; it does not.
4. **The remembered map can be wrong about walls.** A square drawn as `.` was
   a wall: the walk stopped with "There is a wall in the way!" (ask 43-44), and
   the square was then correctly drawn as a wall. An object was also drawn on a
   square I could not walk onto (the `*` at ask 64). So a planned path is a
   hint, not a promise — one digit string wastes one ask, not a fight.
5. **Runs and digit strings.** `.`+direction follows a corridor through the
   dark, which is worth more than a digit string when the corridor is not yet
   drawn; but **any keypress during a run cancels it** ("Cancelled."), so a
   trailing space after `.8` throws the run away. A blocked move costs no turn.
6. **The quaff prompt uses pack letters, the throw prompt does not.** Quaff:
   `q`, a space (the prompt prints a message with a `-more-`), then the pack
   letter (`d` = Cure Light Wounds). Throw: `v`, the *throw prompt's* own letter
   (`a` = oil), then a target. The throw target prompt's `'` (closest) only
   works for a monster **in view**; my light radius is 2, so for anything
   further away I aim with a direction key (`va4` = throw oil west), which
   costs the throw its hit chance but not its aim.
7. **`o` (open) asks for a direction, and `5` means your own square** — but
   only if the chest is *openable* (`obj-chest.c`: openable means `pval != 0`).
   The chest I found was already open (its contents were a cloak lying on the
   floor), so `5` was refused and the prompt stayed up.
8. **The floating eye's gaze only bites when you stand next to it, and it
   never moves** (`monster_base.txt`: "Motionless creatures which cause bad
   effects if you stand next to them"; `NEVER_MOVE`). Oil from three tiles
   killed it for 1 experience unit. The white worm mass is the opposite: 10 HP,
   `1d2` crawling poison, and `MULTIPLY` means it *breeds while alive* — three
   of them became eight in about thirty turns, and it gave 2 units each.
9. **Detect Life is the one spell worth its point**: it detects in a 22x40
   area (`effect-yx:22:40`), the first cast pays 2 experience units, and it
   found both the eye and the mouse. Spell points come back very slowly: 16
   turns of rest bought 1 SP at CL1.

## The decisions I would take back

* **I spent the first twenty asks on shopping and the worm room.** The worm
  room cost about 9 HP and ten asks for 4 experience units; the same asks spent
  exploring would have mapped more level. The manual's advice to leave breeders
  alone is right, and I broke it once.
* **I should have gone west early.** The frontier logic always sent me to the
  nearest unexplored square, which kept me circling the eastern blob. The
  down staircase is one room in fifty, and the level's western two thirds were
  never touched. A deliberate westward push — or a Magic Mapping scroll to
  scan 40x22 at a time — was the play I did not make.
* **I did not press Detect Life for what it is worth.** I used it twice, both
  times to find one specific monster. Detecting life in an unfamiliar room
  before entering it is the cheapest safety there is for a 13-HP character, and
  I spent my two points on targets instead.
* **Two of my four Cure Light Wounds went to the worm room**, and I still ended
  with three in the pack; the balance was right, the timing was not.
* **I left the run at CL1 with one experience unit to spare.** A single mouse
  (1 unit) stood between me and level 2; I spent the last asks on map
  coverage instead. That was a defensible trade, but it is the trade I would
  reverse first.

## Where the reading was wrong, or not enough

* `manuals/INSTRUCTIONS.md` §8 says the Deep Descent scroll is "found from
  DL1"; `object.txt` gives it `level:10` with `alloc:20:1 to 100`. One of the
  two is wrong, and `sonnet-2` found one on depth 1, so the data's `level:`
  field is probably not the allocation depth. That matters, because a Deep
  Descent is the only quick way down: it drops you five levels below your
  deepest level, and `sonnet-2` went from depth 1 to depth 6 that way.
* The manual's "`M` whole-level map" and its "walk to a known place" both
  needed a correction (see 2 and 3 above). The "walk to a known place" recipe
  is the best thing in the manual and it is marked untested; it is the first
  thing a next run should use.
* `knowledge/NOTES.md` said the throw prompt's letters differ from the pack's.
  True, and the same is *not* true of the quaff prompt, which uses the pack
  letters — worth adding, since I guessed wrong once.
* The manual never says how big a dungeon level is. 198x66 with up to 50
  rooms is the single fact that explains every "I explored forever and found
  no stairs" report in `NOTES.md`, including mine.

## What I could not see or could not do

* I never saw a down staircase, so I could not test the targeting mode's `>`
  (nearest known down staircase), and I could not test the `g` walk to it.
* I could not test a Deep Descent scroll: I found five floor items in 148 asks
  (two flasks or potions, a cloak from an open chest, one unknown `*`) and no
  scroll. The Deep Descent route stays unproven by me.
* I could not buy a Scroll of Magic Mapping: it cost 65 gold and I had 26,
  the Alchemist did not stock one this seed, and the Black Market (which
  stocks randomly) was never visited — it is the only other possible source.
* I never went to the Magic Shop (store 6) or the Black Market (store 7); the
  town's own doors were never used, and I never identified anything.
* The level's unexplored two thirds are simply unknown to me: I am reporting
  a negative result from the eastern third only.

## Leads, with what would settle each

1. **Is the down staircase findable at all on depth 1, and where?** The level
   is 198x66 with up to 50 rooms (`dungeon_profile.txt`, "classic":
   `params:11:50:200:2` = block 11, 50 rooms). I mapped the eastern third in
   148 asks and found two up staircases and no down one. What would settle it:
   a run that buys a Magic Mapping scroll (65 gold) or a run that pushes
   westward for 60 asks with the `*xg` trick, and reports where the
   staircase was when it finally appeared.
2. **Does `*` then `>` really jump to the nearest known down staircase?**
   The source says so (`ui-target.c`, `path_nearest_known` with
   `square_isdownstairs`), but it is untested because I never saw one.
   What would settle it: one descent to depth 2, then `*`, `>`, and see whether
   the cursor lands on the `>` you just used, then `g` to walk back to it.
3. **How common is a Deep Descent scroll on depth 1?** `object.txt` says
   `level:10` but `alloc:20:1 to 100`; `sonnet-2` read one on depth 1, and the
   notes say 1 of 4 unknown scrolls in all runs was one. What would settle it:
   read the allocation code in `obj-make.c` (whether `alloc_min/alloc_max` or
   `level` governs the depth), or find one on depth 1 and read it.
4. **Why can the remembered map show floor where there is a wall?** One case
   (ask 43-44) and one object drawn on an unwalkable square (ask 64). What
   would settle it: compare `cave->memory` with `cave->grid` for the square in
   question — probably a `mimic` feature (terrain.txt has granite walls that
   mimic other features) or a stale redraw.
5. **What is the experience economy on depth 1?** 13 of 14 units came from six
   kills and one spell cast: worm masses and the centipede 2 each, the bat,
   the eye and the mouse 1 each, Detect Life's first cast 2. Level 2 for a
   High-Elf is 14 units (145 experience). What would settle it: a table of
   `experience:` values per depth-1 monster against the number of kills a
   careful level-1 player can manage in 100 turns.
6. **Is a druid viable at all in this game?** Hit die 2 plus CON, 13 HP at CL1
   and a `1d1` whip; the class's own manual block says oil is the attack until
   CL3. What would settle it: a druid run that reaches CL3 (where Stinking
   Cloud arrives) — with the Stinking Cloud spell and 18 HP, the melee
   problem may not matter at all.

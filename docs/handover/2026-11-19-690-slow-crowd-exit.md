## 2026-11-19: walk out of a crowd of slower monsters (#690)

Owner, 2026-11-19, on #675: "yes file that"; then "do 690 next".

**The case** is borg-cemetery#78. A CL6 Blackguard (seed 1472012981) on
DL1 had spent its Phase Door and Word of Recall. It stood in a yellow worm
mass colony and meleed on one grid until it died. The worms were slower
than it (speed 100, energy 5, against its 110, energy 10), and a grey
mushroom patch was next to it. Because the patch is not a breeder, #544's
breeder exit did not apply.

**What was done:** `caution_slow_crowd_exit`, compiled 0 and shipped 0. It
is tried in `borg_caution` right after #544's breeder exit.
- **When:** all of these hold:
  - the Borg is surrounded (`borg_surrounded`);
  - the danger here is at or above the HP;
  - a monster is adjacent;
  - every awake monster within `danger_monster_max_distance` either never
    moves or is slower than the Borg. Slower means energy per game turn
    (`extract_energy`), with a margin of
    `caution_slow_crowd_exit_margin_percent` (25).
  - A sleeping monster does not count. An awake monster as fast as the Borg
    or faster holds the rule.
- **Where to:** it walks with the desperate flow, as #544 does.
  - First choice: the nearest known chokepoint (corridor or doorway) or
    staircase, at least `caution_slow_crowd_exit_distance` (6) grids from
    every monster in reach.
  - If none can be reached: the nearest other known open grid that far.
  - If neither: the distance comes down by one, to 2.
- **Tests:**
  - `a_crowd_is_slow_when_every_awake_mover_is_slower_by_the_margin`;
  - `crowd_exits_are_corridors_and_staircases_beyond_the_crowd`.

**Found on the case replay:** the first version only took chokepoints and
staircases. It found 6 exits, but never walked:
- the Borg was cornered at the east edge of the known map, at (178,20);
- the nearest corridor was 115 grids west, behind the worms (flow cost
  255, no path);
- by the end, 19 worm masses left no grid 6 grids from all of them.

That is why the open grids and the descending distance were added.

**Checked:**
- CI's commands; `check-param-citations.py` (2202, after merging main `18d2094f`).
- Switch off: the 18 classes-18 starts play keys identical to main
  `a87b4c96` (18/18).
- Switch on: the same 18 are unchanged too. The rule never fires in them.
- **The case**, replayed on its own build (`6e64ddb9`, #659's branch, with
  this change applied), shipped settings plus
  `caution_ranged_vs_immobile = 1`. These are local leads (RULES 14):

| arm | result |
|---|---|
| switch off | dies to a yellow worm mass, turn 4,578, CL6 (the dossier) |
| on (distance 6, margin 25) | lives: 100,000 turns, CL8, DL3; 33 walks |
| on, margin 0 | the same game as "on" |
| on, distance 3 | dies, turn 4,883, CL6; 39 walks |

  The distance-3 death shows the game is chaotic here. With the first
  version (no open grids, no descending distance), distance 3 lived and
  distance 6 died.

**How often (analysis):**
- Of 8,481 distinct deaths in the local zoo results (`zoo-results/results/reborn-*`):
  - 251 (3%) were killed by a monster slower than normal speed: white worm
    masses 135, large white snakes 56, green 34, blue 11, red 5, yellow 4,
    clear 3, creeping coins 3;
  - 479 more were killed by monsters that never move;
  - 556 were killed by "poison", with no monster named (worm bites among
    them).
- The zoo keeps no message logs, so the "surrounded on one grid" sign
  cannot be counted there.
- Among the Cemetery dossiers in the local checkout (37), no other slow-killer death carries
  the "surrounded" note in its trace tail.
- Worm masses are breeders, and #556's `caution_breeder_stoppable` also
  covers them. The large white snakes (not breeders) are this rule's own
  share.

**Left / for the owner:**
- A zoo pilot. The 18 starts never trigger the rule, so only a big cohort
  can show an effect.
- The distance (6) and the margin (25%).
- How it interacts with #556's stoppable switch: kill first, or walk? This
  rule only acts at danger at or above the HP while surrounded.

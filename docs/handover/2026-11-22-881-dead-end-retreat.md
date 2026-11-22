## 2026-11-22: Retreat and back-up refuse a dead end near breeders (#881, part 2)

Owner, 2026-11-22: "start on part 2, the dead-end retreat". The branch is
`task/T881-dead-end-retreat`, from `origin/track-p` `e2d8919e`. P7 closed for
review the same day (PR #939, promote/P7), and #881 moved to **P8**, which
waits for the P7 promotion. This PR therefore cannot merge into `track-p`
before P7 is promoted (RULES 1). Part 1 is PR #897 on
`claude/focused-goldberg-2m52ql`; the two changes are independent and touch
different functions.

**What changed.** `retreat` takes the nearest viewable "happy" grid of lower
danger, and `back_away` the adjacent grid of least danger
(borg-core/src/borg_caution.rs). Neither knows where the way out is. In a
one-exit lattice room every square is happy. Priest 399497898 made 47
retreats and 27 back-ups inside the room after turn 62900, reached the
dead-end tip 25 times, and never left. The study is in borg-cemetery,
`studies/worm-pocket-399497898` (branch `claude/modest-knuth-w7k1hg`).

`caution_retreat_dead_end` (mode, compiled and shipped **0**) and
`caution_retreat_dead_end_distance` (grids, 6) are listed in
`harness/borg.txt` and documented in docs/CALIBRATION.md. The rule applies on
a dungeon level when a known breeder is within `danger_monster_max_distance`
of the Borg (`RetreatExits::for_think`).
- **The way out** is every known floor grid at least the distance from
  every known breeder, plus every known staircase. Two breadth-first
  searches run over known floor (unknown grids count as walls; monsters are
  ignored): one gives each grid's distance to the way out, the other runs
  with the Borg's own grid closed.
- **Mode 1** refuses a grid reached only through the Borg's grid: the tip,
  or the whole branch behind the Borg.
- **Mode 2** also refuses a grid farther from the way out than the Borg's
  own grid.
- The rule checks a retreat's target and its first step, and each back-up
  grid. It refuses nothing when the Borg has no known path out or already
  stands on the way out.

A unit test (`retreat_refuses_the_pocket_tip_and_mode_2_the_grids_deeper_in`)
uses a small lattice pocket under a corridor.

**Checks.** fmt, clippy, workspace tests, `cargo test --release -p
borg-core` (654 pass, 4 ignored), script and tool tests, citations
2378/2378, vendor contract ok. Smoke at the shipped settings: 3/3
identical to `smoke-borg-rs`, so no re-record.

**Not checked.**
- The named case cannot test this. On track-p, Priest 399497898 plays
  differently from key 29 on and never reaches the worm room (part 1's
  handover).
- I did not run a local batch in a breeder world to see whether the rule
  fires. Comparisons over many starts belong to the zoo (RULES 20); the
  session's local batch was refused for that reason.
- The rule writes no note when it refuses a grid, so the zoo will see it act
  only as games that play differently from `ctl`.

**Owner decisions.**
- A zoo request for modes 1 and 2, like borg-zoo#642: in breeder worlds
  and/or normal play, and combined with part 1's arms or not. Not filed yet.
- Whether to extend the rule to slow crowds without breeders (#690's
  `slow_crowd_exit` case), which it does not cover now.
- Whether to add a note on each refusal (useful for the zoo's activation
  counts; it changes nothing at mode 0).

Part 3 of #881 (the perception gap at 28,147) is not started.

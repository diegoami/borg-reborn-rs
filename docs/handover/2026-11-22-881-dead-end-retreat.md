## 2026-11-22: Retreat and back-up refuse a dead end near breeders (#881, part 2)

Owner, 2026-11-22: "start on part 2, the dead-end retreat". The branch is
`task/T881-dead-end-retreat`. It started from `origin/track-p` `e2d8919e`
and has `track-p` `4b2c2fcf` merged in (the PR's base). #881 is in **P8**.
The P7 review runs from the fixed branch `promote/P7` (#939), so `track-p`
is open (owner, on #897). Part 1 is PR #897 on
`claude/focused-goldberg-2m52ql`. The two parts are independent and touch
different functions.

**What changed.** `retreat` takes the nearest viewable "happy" grid of lower
danger, and `back_away` the adjacent grid of least danger
(borg-core/src/borg_caution.rs). Neither knows where the way out is. In a
one-exit lattice room every square is happy. Priest 399497898 made 47
retreats and 27 back-ups inside the room after turn 62900, reached the
room's north corner 25 times, and never left. The study is in borg-cemetery,
`studies/worm-pocket-399497898` (branch `claude/modest-knuth-w7k1hg`).

`caution_retreat_dead_end` (mode, compiled and shipped **0**) and
`caution_retreat_dead_end_distance` (grids, 6) are listed in
`harness/borg.txt` and documented in docs/CALIBRATION.md. The rule applies on
a dungeon level when a known breeder is within `danger_monster_max_distance`
of the Borg (`RetreatExits::for_think`, with the gates in
`RetreatExits::build`).
- **The way out** is every known floor grid at least the distance from
  every known breeder, plus every known staircase. Two breadth-first
  searches run over known floor (unknown grids count as walls; monsters are
  ignored): one gives each grid's distance to the way out, the other runs
  with the Borg's own grid closed.
- **Mode 1** refuses a grid reached only through the Borg's grid. That
  happens only where the Borg's grid is the only link: a corridor's end, or
  a pocket behind a single square.
- **Mode 2** also refuses a grid farther from the way out than the Borg's
  own grid.
- Both refuse a grid the Borg sees but has not mapped (`FEAT_NONE`).
- The rule checks a retreat's target and its first step, and each back-up
  grid. It refuses nothing when the Borg has no known path out or already
  stands on the way out.

**Review (Mavis, MiniMax M3.1-Flash, on `b157f9dd`: pass with findings).**
- **F1:** the review found that in the priest's room **mode 1 refuses
  nothing and mode 2 is the mode that acts**. The north corner keeps a
  second known link, so it is no dead end in the rule's sense; the room is
  a mesh. Replaying the 74 recorded decisions against the rule: mode 1
  refuses 0/47 retreats and 0/27 back-ups, mode 2 refuses 25/47 and 11/27.
  The study's truth map stood in for the Borg's map, so #881 part 3 could
  change this. I checked the corner's two links on the study's map at
  63192: (30,147) and (30,149).
  - The new test `in_a_lattice_room_mode_1_refuses_nothing_and_mode_2_the_corner`
    shows this on a small lattice.
  - The first test (`retreat_refuses_the_pocket_tip_and_mode_2_the_grids_deeper_in`)
    is a single-link pocket: it tests the helpers, not the priest's room.
  - The expectation was added to borg-zoo#659: `d1` ≈ `ctl` in lattice
    rooms; `d2` is the arm that changes play.
- **F2, F3:** the setting's text, the doc comment and CALIBRATION say when
  mode 1 refuses (only where the Borg's square is the only link), and that
  both modes refuse a visible but unmapped grid.
- **F4:** I dropped the unreachable match arm in `exit_refuses`.
- **F11:** the gates moved into `RetreatExits::build`, a pure function;
  `the_dead_end_rule_is_inert_outside_its_gates` covers mode 0, town, no
  breeder, a breeder out of reach, standing on the way out, no known way
  out, and a staircase as the way out. The settings load once and cannot be
  set in a unit test, so `for_think` itself (the settings read) and the
  wiring into `retreat` / `back_away` are not unit-tested.
- **F5, not changed:** `known_breeders` keeps the race bound
  (`r_idx >= r_max - 1`) of `breeder_exit` and `slow_crowd_exit`. A shared
  helper, or a fix to the bound, would change those two functions too, so
  it belongs in its own PR if wanted.
- **F6, not changed:** with a mode on, the map is built in both `retreat`
  and `back_away` in one think. That costs nothing at mode 0; a per-think
  cache is for later, if a mode ships.
- **F7:** the PR says `Closes #881` for part 2, and #881 stays open for
  part 3.
- **F8:** RULES 4 forbids model trailers in commits and PR bodies. Two
  commits on this branch (`4f89626a`, `4bd55cd7`) carry one. They are not
  rewritten: RULES 5 forbids amending published commits, and borg-zoo#659
  builds `b157f9dd`. Later commits and the PR body carry none.
- **F9, F10:** this note and the PR body now point at borg-zoo#659 and the
  current base.

**Checks.** fmt, clippy, workspace tests, `cargo test --release -p
borg-core` (656 pass, 4 ignored), script and tool tests, citations
2378/2378, vendor contract ok. Smoke at the shipped settings: 3/3
identical to `smoke-borg-rs`, so no re-record. CI is green on `b157f9dd`.

**Not checked.**
- The named case cannot test this. On track-p, Priest 399497898 plays
  differently from key 29 on and never reaches the worm room (part 1's
  handover).
- I did not run a local batch in a breeder world. Comparisons over many
  starts belong to the zoo (RULES 20), and the session's local batch was
  refused for that reason. The test is **borg-zoo#659** (owner, 2026-11-22:
  ctl, d1, d2 in breeders-x4 and breeders-x8, pinned at `b157f9dd`; the
  fixes after that pin change no play).
- The rule writes no note when it refuses a grid, so the zoo sees it act
  only as games that play differently from `ctl`.

**Owner decisions.**
- Whether to extend the rule to slow crowds without breeders (#690's
  `slow_crowd_exit` case), which it does not cover now.
- Whether to add a note on each refusal (useful for the zoo's activation
  counts; it changes nothing at mode 0).
- F5 and F6 as follow-ups, if wanted.

Part 3 of #881 (the perception gap at 28,147) is not started. The F1 result
depends on it: if the Borg's map of the room is smaller than the truth map,
mode 1 can fire more often.

## 2026-11-22: Stair flights price or refuse unknown grids (#881, part 1)

Owner, 2026-11-22: #881 goes to track P, P7; part 1 first (the flight's
route on known grids), as a named setting shipped off. Branch
`claude/focused-goldberg-2m52ql` from `origin/track-p` (`4c686250`).

**What changed.** `weighted_stair_path` (borg-core/src/borg_flow.rs) is the
route of every weighted stair flight; the flight weight ships at 5
(`caution_stairs_flight_threat_weight`). It priced a grid the Borg's map
holds as unknown (`FEAT_NONE`) at one step, like floor, because
`spread_blocked` refuses unknown grids only with `avoid` or when desperate,
and the down flights pass `avoid: false`. Two new parameters
(docs/CALIBRATION.md, "Stair flights through unknown grids"):

- `caution_stairs_flight_unknown` (mode, 0): 0 = one step (today's play);
  1 = one step plus `caution_stairs_flight_unknown_steps`; 2 = refused.
- `caution_stairs_flight_unknown_steps` (steps, 5; the issue's range 3–10).

`harness/borg.txt` lists both at their compiled values, so a zoo arm can
set them with `BORG_TXT_SET`. The same search answers the flee rules'
reachability test (`stair_flow_reaches` → `weighted_stair_reachable`). Under
mode 2, a staircase reachable only through unknown grids is not reachable for
that test either. That is deliberate: the test and the flight agree. A unit
test covers the price and the refusal on a toy grid.

**Checks.** fmt, clippy, workspace tests, `cargo test --release -p
borg-core` (637 pass, 4 ignored), citations 2375/2375, vendor contract ok.
Smoke at the shipped settings: 3/3 identical to `smoke-borg-rs` (keys,
messages, states), so no re-record.

**Named case, Priest 399497898** (one case, not a measurement, RULES 14/20).
I built da097a76, the zoo specimen, with only this commit's borg-core code
applied. I played the `reborn-confusion-clw_walk` arm from birth (seed
399497898, race 8, class 3, menu, exact, 100,000 turns):

| mode | result | keys.log |
|---|---|---|
| 0 | dies, white worm mass, turn 66588 | `dde95ceb…`, the zoo's death, byte for byte |
| 1 (5 steps) | alive at the cap, DL1 | `d4fb2425…` |
| 2 | alive at the cap, DL1 | `d67f29d4…` |

Modes 1 and 2 first press a different key at key 2113, game turn 27807. That
is a flight from an apprentice ("Scary guy on level") on another level,
35,000 turns before the worm room. These games never reach the room, so
their survival is no evidence on the room. It only shows that the setting
changes ordinary flights early in a game.

I also tried to isolate the room: stop at key 5314 (turn 62000), save, and
continue from the save under each mode. That failed as a control: after the
load even mode 0 lives, with or without `keep-load-rng`. The Borg's map
memory does not survive a save. A real counterfactual needs the study's
in-process handover (`BORGH_HANDOVER` in the Cemetery study's
`tools/diagnostic.patch`), and I did not run it.

**The same case on track-p** (owner: "repeat on track-p"). The build is
this branch's head `ea14528a` (track-p `4c686250` plus this change, clean);
seed 399497898 from birth, 100,000 turns, arms ctl and clw_walk, each in
modes 0, 1 and 2. All six games live. Every one plays differently from the
zoo's death from key 29 on, in town, because of the settings shipped since
da097a76. The worm room never comes. Mode 1 first plays differently from
mode 0 at turn 11177 and mode 2 at turn 13120. So on track-p the case is
gone at the shipped settings, and it cannot test the change either way. The
zoo request (borg-zoo#642) is the test.

**Left.**
- A borg-zoo comparison of modes 1 and 2 against the shipped 0 (RULES 20).
  The endpoint is deaths by 100,000 turns (P7's tactical endpoint).
  Requested as borg-zoo#642 (owner, 2026-11-22: in the breeders-x4 and
  breeders-x8 worlds); the owner dispatches it.
- #881 part 2 (retreat and back-up refuse a dead end) is started on its
  own branch (owner, 2026-11-22). Part 3 (the
  perception gap at 28,147, granite in `player->cave`, unknown to the Borg)
  is not started. It can explain how much of the "unknown" in the
  lattice should have been known.
- Not done: the issue's option of allowing unknown grids only beyond the
  first K steps.

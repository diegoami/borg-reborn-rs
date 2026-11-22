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

**Review (MCode, MiniMax-M3.1-Flash, on `24496992`): no code defect.**
- **F1:** mode 2 also changes three flee decisions. `stair_reachable` asks
  the same search, so with no staircase reachable through known grids,
  `stairs_before_recall`, `scary_ranged_no_stairs` and
  `ranged_before_recall` (switches shipped 1) let the Borg read Recall or
  shoot. Mode 1 only re-ranks routes. Documented (option a) in
  CALIBRATION, in the setting's text and on borg-zoo#642, which measured
  mode 2 with this coupling.
- **F2 / F5:** #881 is split, so `Closes #881` closes only part 1. The
  leftovers are issues:
  - #1045: part 2, the dead-end retreat (PR #953);
  - #1043: part 3, the perception gap;
  - #1044: the K-steps option.
- **F3:** the branch's commits up to `f5659f22` carry a model trailer
  (RULES 4). They are not rewritten, following the owner's ruling recorded
  in #880's handover: earlier published commits keep theirs, later ones
  carry none. RULES 5 forbids amending published commits, and borg-zoo#642
  builds `ea14528a`. Commits from this one on carry none.
- **F4:** the branch name `claude/focused-goldberg-2m52ql` is not RULES 5's
  `task/...` form. The session environment allows pushing only to that
  branch; part 2 got a `task/` branch with the owner's leave.
- **F6, F7:** CALIBRATION now says where the rule acts (the down flight)
  and that 5 steps equals the cost of a grid at full-HP danger while the
  threat weight is 5.
- **F8:** the two new `harness/borg.txt` keys change each smoke run's
  borg.txt hash recorded in `harness/reference/smoke-borg-rs/REFERENCE`.
  That hash is provenance and `harness/diff` does not compare it, so the
  comparison passes with no re-record. A later re-record will show the new
  hash; that is not a behaviour change.
- **F9:** citations are now 2391/2391 after the merge with track-p
  `cd1e81e0`.
- **F10:** mode 0 adds exactly 0 to every grid's cost, so the shipped play
  cannot change by construction; smoke only checks the wiring. No test pins
  "a grid the Borg has seen is not unknown": that needs the engine.

**Shipped off (owner, P9 rule).** Every new behaviour is behind
`caution_stairs_flight_unknown` at 0 (`..._steps` 5 is a sub-setting).
Smoke-borg-rs is 3/3 identical to track-p's reference, with no re-record.

**Zoo result, borg-zoo#642** (pilot, small and medium, 360 starts per world,
`ea14528a`). Deaths in 360:
- breeders-x4: ctl 17, m1 16, m2 13 (13 rescued / 9 newly killed, p 0.52);
- breeders-x8: ctl 31, m1 34, m2 31; m2 x8 has 5 unfinished games against
  ctl's 2.

No arm moves deaths. The zoo saw no case for a normal-world tier.

**Left (issues).** #1043 (part 3), #1044 (K-steps option); part 2 is #1045
and #953.

**Owner decisions.**
- Whether to try mode 2 narrowed to the route only (the reachability test
  kept on today's search: review F1 option b). There is no evidence for
  either mode yet.
- The trailers (F3) and the branch name (F4), if the rulings above do not
  stand.

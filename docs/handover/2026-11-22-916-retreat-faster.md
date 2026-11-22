# 2026-11-22 — #916: no strategic retreat from a faster monster in contact

**Issue:** diegoami/borg-reborn-rs#916. **Branch:** `task/T916-retreat-faster` (from `track-p`
`86843a79`, `track-p` `b7acbdec` merged in). Track P, P8: this PR merges into `track-p` once the
owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md)
(seed 2008390085, borg-zoo#596); zoo request borg-zoo#652.

## What was done

New parameters `caution_retreat_from_faster` (compiled and **shipped 0**) and
`caution_retreat_faster_margin` (5). With the switch on, `retreat` in `borg_caution` is refused:
- while an awake monster next to the Borg is faster by the margin (observed haste counted):
  `# No retreat: <race> in contact is faster by N (#916)`;
- toward a destination whose danger is still at or above the HP.

"Faster" compares the engine's own speeds: `player->state.speed` (`player_combat_speed`, which
includes the timed haste, `player-calcs.c:2176-2187`) against the monster's `mspeed` plus 10 when
its `MON_TMD_FAST` timer runs. It reads neither `BI_SPEED` nor `temp.fast`, so #941/PR #947 does
not change what it reads; #947 changes only how often the Borg is hasted.

`track-p` `b7acbdec` merged in: both sides kept in `borg_caution.rs` (#881's `RetreatExits`),
`params.rs` and `CALIBRATION.md`, then `check-param-citations.py --fix` (0 errors).

## Why it stays 0

- Zoo game (`7f8c6b80` + `6113b3a8`, stats-ban borg.txt): off plays the zoo game (keys
  `1ef12466…`, death at 64,483); on lives to the cap (CL7, max DL3): at 64,444 the refused
  retreat leaves the turn to the emergency Phase Door.
- borg-zoo#652 small + medium (`67a0c69d`, 360 paired starts): the `retreat` arm rescued 0 and
  newly killed 3; mean CL and depth level with the control. Pilot: 8 of 18 games changed, 0/0.
- The three new deaths replay key for key on `67a0c69d`. Two follow refusals from Fang (+10)
  where the Phase rule says the fight is not lost, so the Borg attacks instead of stepping; a +10
  monster acts twice per Borg action, so the step did cost it a move. The third dies 15,000 turns
  after its only changed decision (divergence). Details in `docs/CALIBRATION.md` and the PR.
- On the merged head the switch changes no named case's outcome: 2008390085 (shipped and
  stats-ban arm) and 224 play the same keys off and on. With #947 merged in (a local scratch
  merge, never pushed) 2008390085 plays differently but lives either way (CL6, max DL3 off, DL2
  on); 224 plays the same keys.

## What is left / owner

- **Owner:** whether to ask the zoo for a narrower arm. Leads, from four replays only: margin 15
  (the three new deaths then replay the control, the zoo game keeps its rescue), or refusing the
  retreat only when an escape will take the turn (the Phase rule's "fight looks lost").
- Whether the back-away step (`back_away`) needs the same test is open.
- Thresholds are hypotheses until borg-zoo measures them (RULES 14, 20).

## 2026-11-20: flight tuning after the Ranger deaths (#743, #747, #749, #750, #751, #753)

Owner, 2026-11-20, after the analysis of borg-zoo#488's 12 Ranger deaths:
"we must fix the flight. My idea is to stop flight when other monsters are
IN BETWEEN the stairs, not close"; "the problem is the flight algorithm we
implemented, it needs tuning".

**What was done:** five switches, compiled and shipped 0 = today
(docs/CALIBRATION.md, "Flight tuning after the Ranger deaths"):
- `caution_stairs_clean_path` 2 (#751): only a monster on the path, the
  staircase included, blocks a walk to the stairs; read in `path_blocker`,
  so it also applies to #743's check of every flight.
- `caution_desperate_fast_danger_percent` (#747): a harmless faster
  neighbour no longer cancels "Desperate for Stairs (three)".
- `caution_recall_cure_min_loss_percent` (#749): no Cure Light Wounds at
  full HP while waiting for Recall.
- `caution_stairs_flight_ignore_danger` (#750): a flight's stair flow
  refuses no grid for its danger; the path test decides.
- `caution_rising_hold_min_hp_percent` (#753): #557's rising hold no longer
  keeps a nearly dead fleeing Borg off a down staircase.

The other session's #743 (PR #752, `caution_stairs_clean_path_flee`) is
merged into this branch, at the owner's request; PR #752 is closed in
favour of this one. #755 (C's leash as parameters) stays its own PR.

**What was found:** on the 12 Ranger deaths each of #747, #749 and #750
saves two games, all four switches together save 4. On 600 local starts
(200 Rangers, 50 of every other class) the switches together are neutral on
the Rangers (9 → 10 deaths, 1 saved, 2 new) and lean harmful elsewhere
(3 → 8, 1 saved, 6 new). No single switch reproduces the new deaths. These
are too few deaths for a claim either way.

**Left:**
- borg-zoo#501 measures the switches (with #755 on the scratch branch
  `scratch/T751-with-755`); per-switch arms are the owner's call. Every
  switch stays at 0 until it reports.
- #754 (the Borg stops after Recall) is not started.
- Smoke at the shipped values is unchanged (3/3 identical), so
  `smoke-borg-rs` is not re-recorded.

**Later the same day: mode 3 and the review (a comment on PR #757).**
- The owner, on the death page of the "on" arm (93034700: a Necromancer at
  11/11 HP walked past an adjacent apprentice to stairs 8 steps beyond it and
  died carrying 9 CLW, 6 Phase Door and 27 Flasks of oil): "he has to
  actually walk past his opponent"; "phase door would work in this case,
  change that setting". `caution_stairs_clean_path` 3: on the path, or next
  to it and nearer the staircase than the Borg. A first version compared flow
  costs; the flow skips monster grids, so it compares grid distances to the
  path's last grid. On the 600 starts mode 3 has 16 deaths (mode 2 18, off 12);
  CALIBRATION.md has the table.
- Review findings fixed: #750's scope is every `GOAL_BORE` stair flow while
  fleeing or leaving (the docs now say so; the runs measured that code);
  #743's fallback tries each kind under `stair_both`'s rules (`stair_kinds`),
  not `stair_more`'s `more_refused`; `path_blocker` is split into
  `walk_path` and `first_blocker`, with tests on a constructed corridor (on
  the path, on the staircase, beside it ahead, next to the Borg level with it,
  behind, asleep); #743's CALIBRATION row cites `borg_caution.rs`.
- Filed from the same deaths: #758 (Recall read as a flight next to the
  killer), #759 (never flee a winning fight), #760 (monster categories:
  attack on sight), #761 (Phase Door, Rangers first; a Sonnet agent of this
  session is measuring arrows 20, heal reserve 100 and oil 25 for Rangers).

# 2026-11-22 — #916: no strategic retreat from a faster monster in contact

**Issue:** diegoami/borg-reborn-rs#916. **Branch:** `task/T916-retreat-faster` (from `track-p`
`86843a79`; `track-p` merged in at `b7acbdec`, `b784409e`, `f8725bc0`, `182ca8b2` and `802ba380`). Track P, P8: this PR merges into `track-p` once the
owner says its review is done.

Evidence: [borg-cemetery study](https://github.com/diegoami/borg-cemetery/blob/claude/happy-brahmagupta-2jcsa3/studies/fastfrag-cave-spiders-2008390085/README.md)
(seed 2008390085, borg-zoo#596); zoo request borg-zoo#652.

## What was done

New parameters `caution_retreat_from_faster` (compiled and **shipped 0**) and
`caution_retreat_faster_margin` (5). With the switch on, `retreat` in `borg_caution` is refused:
- while an awake monster record next to the Borg has a haste forecast at least the margin above
  the Borg's speed: `# No retreat: <race> in contact is faster by N (#916)`;
- when the selected destination's danger is at or above HP: that vetoes the whole retreat.

The contact veto compares a **haste forecast**, not the engine's net speed: `player->state.speed`
(`player_combat_speed`, timed haste included, `player-calcs.c:2176-2187`) against the observed
`mspeed` plus 10 for an observed `MON_TMD_FAST` (`forecast_monster_speed`). It omits the engine's
SLOW adjustment, which subtracts twice the SLOW level (`mon-move.c:1928-1938`): with margin 5, a
monster of base speed 116 with SLOW level 2 against a speed-110 Borg vetoes at +6, though the
engine's net difference is +2. The contact scan accepts any awake adjacent record, with no
freshness (`kill.when`) or mobility (`RF_NEVER_MOVE`) check. It reads neither `BI_SPEED` nor
`temp.fast`, so #941/PR #947 does not change what it reads; #947 changes only how often the Borg
is hasted.

The danger veto runs after the happy-grid destination is selected, on that destination's
predicted danger `b_p`, and returns from `retreat`: it filters no candidate (another destination
below HP is not tried) and does not check the danger of the actual first step.

`track-p` merged in five times: at `b7acbdec` both sides kept in `borg_caution.rs` (#881's
`RetreatExits`), `params.rs` and `CALIBRATION.md`; at `b784409e` (#915, #1030, #1046) only
`CALIBRATION.md` conflicted (both sections kept); at `f8725bc0` (#879's `unseen_*` rows)
citation-only conflicts, `track-p`'s side taken, then re-pointed; at `182ca8b2` (to #1146, merge
`86d5a82b`) `params.rs` and `CALIBRATION.md` taken from `track-p` with this PR's two rows and
section re-added, both test modules kept in `borg_caution.rs`, both blocks in `borg.txt`; at
`802ba380` (to #1158, merge `9b1f293a`) citation-only conflicts in `params.rs` (15 rows),
`track-p`'s side taken, then re-pointed. Each time `check-param-citations.py --fix` (0 errors),
each re-pointed citation checked to read its param.

## Why it stays 0

- Zoo game (`7f8c6b80` + `6113b3a8`, stats-ban borg.txt): off plays the zoo game (keys
  `1ef12466…`, death at 64,483); on lives to the cap (CL7, max DL3): at 64,444 the refused
  retreat leaves the turn to the emergency Phase Door.
- borg-zoo#652 small + medium (`67a0c69d`, 360 paired starts): the `retreat` arm rescued 0 and
  newly killed 3; mean CL and depth level with the control. Pilot: 8 of 18 games changed, 0/0.
- The three new deaths replay key for key on `67a0c69d`. Only the Hobbit Ranger 1248006956's is
  direct evidence: refusals from Fang (+10) where the Phase rule says the fight is not lost, so
  the Borg attacks instead of stepping, and Fang kills it at 5,395. The Elf Paladin 262549225
  (refusals from Fang at 56,352-56,359, killed by an apprentice at 56,728) and the Half-Elf Priest
  584077430 (one changed decision at 8,941, killed by a wild cat at 24,156) died later. "Two
  actions" is the rate at 120 against 110 (`game-world.c:68-89`); the actual gain from a step
  depends on pursuit and energy. Details in `docs/CALIBRATION.md` and the PR.
- On the merged head the switch changes no named case's outcome. At `b7acbdec`: 2008390085
  (shipped and stats-ban arm) and 224 play the same keys off and on. At `b784409e`: 2008390085
  the same keys off and on (both arms); 224 plays differently and lives either way (CL11, max DL6). With #947 merged in (a local scratch
  merge, never pushed) 2008390085 plays differently but lives either way (CL6, max DL3 off, DL2
  on); 224 plays the same keys.

## What is left / owner

- **Owner:** whether to ask the zoo for a narrower arm. Leads, from four replays only: margin 15
  (the three new deaths then replay the control, the zoo game keeps its rescue), or refusing the
  retreat only when an escape will take the turn (the Phase rule's "fight looks lost").
- Whether the back-away step (`back_away`) needs the same test is open.
- Margin 5, the HP cutoff and margin 15 are hypotheses until borg-zoo measures them (RULES 14,
  20); the four margin-15 replays were chosen after the result.
- Not done, for the owner (code changes, outside a wording-only fix): whether the contact veto
  should use the engine's net speed (subtract the SLOW adjustment) and require a fresh, mobile
  record; whether the danger veto should filter candidates during the search and check the
  first step's danger instead of vetoing the whole retreat on the selected destination.

## Review (GPT-6 in Codex, on `86d5a82b`)

Two P3 findings, both addressed in wording only (no behaviour or shipped value changed): the
contact veto is described as a haste forecast that omits SLOW, with an unchecked freshness and
mobility; the danger veto as "the selected destination's danger at or above HP vetoes the whole
retreat", in the param descriptions, `docs/CALIBRATION.md`, the `borg.txt` comment, the
`borg_caution.rs` comments and this note. The zoo deaths now separate the direct (Fang) death
from the two later ones.

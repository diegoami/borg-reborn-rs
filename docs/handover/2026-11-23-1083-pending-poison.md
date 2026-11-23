## 2026-11-23: The Phase rule counts the poison still to come (#1083)

**Done.** `escape_phase_lost_poison_pending`, compiled and shipped 0, so play
is unchanged. With it on, the Phase rule judges the adjacent fight on the HP
left after the poison still to come. The estimate of the timer comes from the
poisoning messages, the poisoner's level and the CON decay, all checked in
the engine.
- `docs/CALIBRATION.md` has the section and the row; `harness/borg.txt` has
  the shipped line.
- Unit tests cover the estimate. A source pin checks that nothing is read or
  written with the switch off, and that the Phase rule judges on the HP after
  the poison.

**Checked.**
- CI's commands and the citation check (2445 parameters, 0 wrong).
- Smoke at the shipped 0: 4/4 identical to `smoke-borg-rs`.
- **Seed 829474344, replayed from birth** on the zoo's `7f8c6b80` plus #968's
  code (`e6eadfd1`) and this change, on a local branch never pushed:

  | settings | result |
  |---|---|
  | both off | the zoo game key for key (`798335ae…`), dead at 96,820 |
  | #968's `caution_poisoned_no_cure_retreat_hp_percent = 75` alone | dead at 96,950, as before |
  | this switch alone | the zoo game key for key: the escapes are not asked during the loop |
  | both on | **lives to the 100,000-turn cap** (CL8); 5 notes, the first at 95,380 |

  With both on, the fight is judged lost at 95,540 (HP 71, about 37 HP of
  poison to come), and the Borg reads Phase Door. #968 alone allowed it only
  from HP 33, at 96,660.
- The estimate against the true timer (`state.log`, `t7`) at the five notes:
  16/19, 40/46, 74/85, 62/75, 104/123, about 15% low. Likely causes: the integer
  mean (10 instead of 10.5 for the white jelly) and touches that came between
  two thinks.

**Left.**
- The other places #1083 names, the outmatched cure and the flee decisions,
  still ignore the poison; only the Phase rule counts it.
- Poison from spells and breaths (Stinking Cloud) adds an amount that depends
  on the damage. Here it is counted as a melee poisoning.
- A zoo measure: with #968, in the jelly worlds of borg-zoo#712. That needs a
  new arm (`poison` together with this switch).

**Owner.** Merge into `track-p` after the review (RULES 1a). Thresholds are
hypotheses (RULES 14).

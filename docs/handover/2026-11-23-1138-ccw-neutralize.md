## 2026-11-23: more Cure Critical Wounds and Neutralize Poison (#1138)

Owner: "We need to get more CCW and cure poison potion, in a similar mechanism as
Detect Invisible scrolls." Stacked on #1112 (`task/P9.1103-black-market`), which
adds the row before these in `NAMES`.

Done, on `task/P9.ccw-poison` from `ffbad634`:

- `PREP_CCW` and `PREP_NEUTRALIZE`, two requirement rows of their own, fed by one
  kind each; `town_prepared_ccw` 3 and `_percent` 24, `town_prepared_neutralize` 3
  and `_percent` 16 (compiled 0). From `town_variety_min_depth`. `NAMES` 85.
- `engine/tests/test_stair_handoff.py` pins the two counts off.
- Unit test `the_ccw_and_neutralize_rows_are_requirements_of_their_own` (the
  first copies, the target, the next, one kind not filling the other).
- Local 36-game comparison off, shipped, three times (table in CALIBRATION,
  "#1138"): CCW bought 0.11, 0.47, 0.72 a game, Neutralize Poison 0.17, 0.39, 0.72;
  the cost shows in Cure Serious Wounds and Word of Recall at three times, and
  slightly in Recall at the shipped percents (1.78, 1.47). Directional only.

Found: at the shipped percents the extra CCW are the Warrior's; the other classes
rarely have 200 gold left after the staples. A higher percent buys more and
displaces Recall and Cure Serious Wounds.

Left, for the owner: the percents (24 and 16 are guesses; the zoo should measure
potions bought and held by class and what they displace); whether Recall and Cure
Serious Wounds should be protected from the new rows. The Neutralize Poison row does
not count Resist Poison; the shared prepared-poison row still does.

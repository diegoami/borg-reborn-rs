## 2026-11-23: Neutralize Poison's own row, and the dear cures wait for Recall (#1137)

**Owner, 2026-11-23**, after #1135 shipped `town_prepared_poison_percent 40`
and a parallel session's local runs (36 games per arm, directional) showed the
dear rows displacing Cure Serious Wounds (1.72 → 0.64 per game) and Word of
Recall (1.78 → 1.28): "40 just Neutralize Poison"; "use the gold floor"; then
"try to use the existing model and push it in a way that when he is rich
enough to get a WoR he starts buying antipoison and CCW".

**Done** (`borg-core/src/borg_town_capability.rs`, `params.rs`, `harness/borg.txt`):
- **`town_prepared_neutralize`, `town_prepared_neutralize_percent`**: a
  prepared row of its own, "prepared: Neutralize Poison", one copy per potion
  (it still counts on the shared poison row, as a CCW does). It replaces
  `town_prepared_poison_percent`, which raised the whole shared poison row
  (Resist Poison, Mushroom of Purging, CCW too) and is shipped back to 0.
  Shipped 2 copies at 40.
- **`town_prepared_dear_after_recall`** (flag, shipped 1), in place of a fixed
  gold floor: the two dear rows (CCW, Neutralize Poison) count only when the
  purse after the urgent needs reaches `buy_recall_from_gold` (the Recall
  reserve's own floor, 200) **and** the Recall reserve (`town_recall_reserve`,
  2; at least one) is in the pack (`BI_RECALL`). Before that they are worth
  nothing and an own percent falls back to the shared one. Reason: a CCW at
  112 percent of the trip weight outbids a missing reserve scroll (100
  percent), so without the gate the dear rows can take Recall's gold. The
  fixed 450 floor (`town_prepared_dear_min_gold`) of #1137's first ask was
  replaced by this on the owner's later word; it never shipped.
- **Row index.** `PREP_NEUTRALIZE = PREP_CCW + 1` (83), `NAMES` 84. Both dear
  rows are held only while one of them is on, so the model keeps its shape
  with both off. #1112's `RESTORE_STAT_RESERVE` also sits after `DEV_CONTROL`:
  whichever merges second renumbers.
- Tests: `a_remedy_of_its_own_percent_buys_dear_cures_with_spare_gold`
  (extended to the new row) and `the_dear_rows_wait_for_recall`.

**Checked.** CI's commands locally (fmt, clippy, the two test runs, the
scripts' tests, `check-param-citations.py`). The smoke result is in the PR.

**Not checked / left.**
- Not measured. The zoo request (borg-zoo) compares the shipped arm with
  0/0/0 and with the gate off, and reports Recall, CLW, CSW, Phase Door,
  light and food per class, not only the dear cures.
- Whether one decision can buy Recall and then the cures in the same town
  visit depends on the planner re-observing after each purchase; the zoo's
  purchase counts will show it.

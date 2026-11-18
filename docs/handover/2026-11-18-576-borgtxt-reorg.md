## 2026-11-18: harness/borg.txt regrouped, 25 parameters written out (#576)

Owner: reorganize `harness/borg.txt`, expose what matters, list what can be
retired.

- **`scripts/borgtxt-inventory.py`:** the inventory, re-runnable
  (`--zoo <borg-zoo checkout>` adds the last zoo definition per key). For
  every core setting, shell key and registry parameter it gives where the
  code reads it, the compiled default, the shipped value and the last zoo
  arm. The report is in the PR.
- **Regrouped:** the core and stops, run control, Town, Word of Recall,
  descending and the trip policy, Combat, Healing, Fleeing, Cornered,
  Breeders, Scary monsters, then the C Borg's own settings; the class block
  stays last. Every setting keeps its value. The 6 measurement starts and
  the six borg-play deaths play byte-identical keys with the old and the new
  file.
- **Written out at their compiled defaults (25):** the Recall waits
  (`caution_recall_*`, #582, #583), Phase Door (`escape_phase_*`, #584),
  Bless (`defend_bless_*`, #560), `caution_heal_outmatched_min_loss`
  (borg-zoo#270, #275), `caution_rising_descend_danger_tenths` (#483),
  `caution_cornered_spell_fail` (#504).
- **Measured reachability:** each of 56 core settings changed in turn, 6
  starts x 50,000 turns. With `borg_town_plan = 1` the live planner handles
  every store visit (`borg_think_store` returns after `execute_store`), so
  the Buying Strategy Family never runs: 0/6 games change even with every
  class bit set.
- **Owner to decide:** the retirement list in the PR. Nothing is retired yet.

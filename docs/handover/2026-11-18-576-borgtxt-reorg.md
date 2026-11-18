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
- **Commented (owner: "comment them well, group them that should be
  changed together, mention how much impact they have and what increasing
  and decreasing do"):** every setting says what it does and what raising or
  lowering does, the sets to change together share one comment, and an
  "Impact" line gives how many of 6 test games (50,000 turns) play
  differently with the value changed. All 77 written `borg_param_` lines and
  56 core settings were measured.
- **Found by the measurement:** `town_heal_reserve_hp_percent` and
  `town_heal_curve_scale_percent` change no game, because every class sets
  its own value in the class files; the common lines are unused now. Many
  others change nothing in 50,000 turns (Phase Door thresholds, Recall waits,
  Bless): situations these short games rarely reach.
- **Retired from the file (owner: "yes retire them"):** seven settings, all at
  their compiled defaults, so play is unchanged (6/6 starts identical):
  the four Buying Strategy settings (only reachable with `borg_town_plan =
  0`), `borg_uses_dynamic_calcs`, `borg_restore_ignore_settings` and
  `borg_allow_strange_opts`. The last two are read in `borg_init.rs` (the
  first inventory missed that file's own readers) and do nothing at FALSE.
- **Review fixes:** `borg_town_light_depth` and `borg_town_food_trip` are
  live (the live planner's light and food needs), not v4 leftovers;
  `borg_town_w_knowledge` is forced to 0 with `borg_trip_policy = 1`; the
  header says the values are shipped, not compiled, and gives the
  `BORG_PARAM_<NAME>__<CLASS>` form. `borg_uses_dynamic_calcs = TRUE`
  changed 6 of 6 games only through its start-up warning (one extra key
  acknowledges it), not through formulas.
- **How the impact was measured** (the driver is not committed): one cell
  per start as the zoo adapter runs it (`harness/borg.txt` with one
  `key = value` rewritten, `scary_monsters.txt` beside it, `-turns 50000
  -exact-turns`), the 6 classes-18 starts 1085222502 (Warrior),
  801785300 (Mage), 102023799 (Druid), 1569794094 (Priest), 1739161339
  (Ranger), 1472012981 (Blackguard), compared by `keys.log`.
- **Left:** #599, retiring them in the core with `retired(...)`. It renumbers
  the settings table and touches several vendored files, the formula module
  among them.

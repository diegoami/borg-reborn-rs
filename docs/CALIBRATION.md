# Calibrating borg-rs's core (track P)

P1.1 (#219) gives caution 200 parameters; P1.2 (#220) adds 366 in danger
and escape, for 566 total. P1.3 (#221) adds 37 recovery settings, bringing the table to
603 entries. This changes calibration access, not the default
policy. No value here is claimed
to improve play; the defaults are the vendored v12 values. The configured
ranges are input bounds for experiments, not measured recommendations.

## Configuration and records

At core initialization, values are loaded once for the process, including
reincarnations. Precedence is `BORG_PARAM_<UPPERCASE_NAME>`, then
`borg_param_<name>` in the run's `ANGBAND_DIR_USER/borg.txt`, then the default.
Other `borg.txt` settings are unchanged. For example:

```text
borg_param_caution_clw_min_loss = 20
```

```sh
BORG_CORE=borg-rs BORG_PARAM_CAUTION_CLW_MIN_LOSS=20 \
  harness/run.sh harness/out/calibration 1010 0 0 100000 menu
```

Values must be integers in the inclusive range. Invalid numbers, unknown
parameter names and duplicate file settings stop initialization with a
specific error on stderr and exit status 2; they are never clamped. An
environment override replaces the file value before numeric validation.

`params-core.json` in `BORG_RUN_DIR` records every name, default, effective
value, origin (`default`, `borg.txt`, `environment`), unit, bounds, source
location and description. A failed write aborts initialization. The harness
hashes this file into `config.txt` as `params_core_sha256`. Death dossiers
keep the file and hash; encounter families include the hash so environment
arms cannot silently merge into one family. Runs of our original core do
not load these settings. Zoo profile wiring is P3.

The parameter table and all marked hooks are ours. Source locations below
refer to **borg_caution.rs at reborn commit `726e04fa`**, before extraction,
not to the file's shifting line numbers after the hooks.

## Units and arithmetic

- `hp`, `sp`, `gold`, `level`, `depth`, `count`, `grids` and `votes` use
  their literal units. `danger` uses borg-rs's existing danger score.
- `percent` is on the original 0–100 scale; `permille` is the device
  failure scale, 0–1000. Loss means missing HP, not remaining HP.
- `tenths` is the numerator of `x * parameter / 10`. `multiplier` is the
  factor of `x * parameter`; `divisor` is the denominator of `x / parameter`
  and is always positive. The integer operations keep their original order.
- `turns` uses the existing caller's clock: Borg-clock ticks for memory
  and delays, danger horizons in Borg turns, engine rest count for
  `caution_recall_rest_turns`, and the original fuel timeout unit for
  `caution_nasty_light_timeout`. There is no clock conversion.
- Healing band endpoints remain separate parameters. Changing them can
  intentionally create gaps or overlaps; tune the bands as a policy.

## Constants retained, and why

The following numbers in caution remain literals because they describe
representation or game structure rather than an adjustable decision:

- Class IDs 0–5; `GOAL_FLEE = 7`; inventory light offset 5; key encoding
  (`b'0'`, direction 5); raw engine enums and item/spell IDs.
- Zero tests for a boolean/status, town depth, empty inventory, no danger,
  dead/missing monsters or no goal; -1 sentinels; index zero, loop
  increments and one-based monster-table boundaries.
- Eight neighbor directions, adjacency at one grid, the exactly-one
  monster/one-blow test, the first retreat step and the 2/3 history offsets
  required to recognize the same bounce pattern. Fast-monster safety's
  two-grid reach is tunable separately.
- 100 for percent conversion and random-number domains, 10 for tenths,
  11 for the regional fear-grid indexing. The RNG domain and number/order
  of RNG calls stay fixed; the chance cutoffs are parameters.
- `near.unique >= 10` is the encoded questor category. Depth 100 is
  Morgoth's level, the `< 100` questor exclusion is that boundary, and
  `CDEPTH + 1` is the next dungeon level. The strategy's 98/85/80 depth
  gates are parameters.
- The first point per drained stat counts distinct drains; class-specific
  extra votes and HP/SP-dependent weights are parameters. Checks against
  actual current/max HP and SP or computed danger retain their relationships.
- The diagnostic-only danger cutoff `avoidance / 10` controls a printed
  note, not an action. Comment examples (healing dice and an unused C
  estimate) are documentation, not executed decisions.

## Parameter catalogue

The description and original expression are also available in the registry
and at each source location. The prefix `caution_` is part of every name.

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `caution_bounce_healthy_tenths` | 7 | tenths | 0–100 | `borg_caution.rs:102` |
| `caution_bounce_limit` | 300 | turns | 0–100000 | `borg_caution.rs:108` |
| `caution_caster_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:157` |
| `caution_priest_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:160` |
| `caution_druid_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:163` |
| `caution_necromancer_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:166` |
| `caution_warrior_stat_vote` | 1 | votes | 0–100 | `borg_caution.rs:169` |
| `caution_stat_fix_hp_high` | 850 | hp | 0–10000 | `borg_caution.rs:171` |
| `caution_stat_fix_vote_high` | 1 | votes | 0–100 | `borg_caution.rs:172` |
| `caution_stat_fix_hp_low` | 700 | hp | 0–10000 | `borg_caution.rs:174` |
| `caution_stat_fix_vote_low` | 3 | votes | 0–100 | `borg_caution.rs:175` |
| `caution_priest_stat_fix_sp` | 100 | sp | 0–10000 | `borg_caution.rs:177` |
| `caution_priest_stat_fix_votes` | 5 | votes | 0–100 | `borg_caution.rs:178` |
| `caution_mage_stat_fix_sp` | 100 | sp | 0–10000 | `borg_caution.rs:180` |
| `caution_mage_stat_fix_votes` | 5 | votes | 0–100 | `borg_caution.rs:181` |
| `caution_heal_spell_fail` | 15 | percent | 0–100 | `borg_caution.rs:196` |
| `caution_clw_estimate_percent` | 15 | percent | 0–100 | `borg_caution.rs:201` |
| `caution_clw_estimate_min` | 15 | hp | 0–10000 | `borg_caution.rs:201` |
| `caution_csw_estimate_percent` | 20 | percent | 0–100 | `borg_caution.rs:202` |
| `caution_csw_estimate_min` | 25 | hp | 0–10000 | `borg_caution.rs:202` |
| `caution_ccw_estimate_percent` | 25 | percent | 0–100 | `borg_caution.rs:203` |
| `caution_ccw_estimate_min` | 30 | hp | 0–10000 | `borg_caution.rs:203` |
| `caution_heal_estimate_percent` | 35 | percent | 0–100 | `borg_caution.rs:205` |
| `caution_heal_estimate_min` | 300 | hp | 0–10000 | `borg_caution.rs:205` |
| `caution_healing_rod_fail` | 500 | permille | 0–1000 | `borg_caution.rs:211` |
| `caution_confused_heal_loss` | 80 | percent | 0–100 | `borg_caution.rs:225` |
| `caution_confused_star_loss` | 85 | percent | 0–100 | `borg_caution.rs:232` |
| `caution_confused_star_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:233` |
| `caution_confused_staff_fail` | 650 | permille | 0–1000 | `borg_caution.rs:259` |
| `caution_confused_cure_danger_tenths` | 15 | tenths | 0–100 | `borg_caution.rs:260` |
| `caution_confused_escape_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:270` |
| `caution_blind_cure_chance` | 85 | percent | 0–100 | `borg_caution.rs:287` |
| `caution_blind_escape_danger_tenths` | 25 | tenths | 0–100 | `borg_caution.rs:291` |
| `caution_blind_heal_loss` | 300 | hp | 0–10000 | `borg_caution.rs:296` |
| `caution_blind_warrior_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:301` |
| `caution_blind_confused_star_loss` | 400 | hp | 0–10000 | `borg_caution.rs:318` |
| `caution_blind_confused_danger_multiplier` | 5 | multiplier | 0–20 | `borg_caution.rs:318` |
| `caution_blind_confused_min_loss` | 100 | hp | 0–10000 | `borg_caution.rs:318` |
| `caution_questor_heal_hp` | 700 | hp | 0–10000 | `borg_caution.rs:327` |
| `caution_questor_holy_word_hp` | 250 | hp | 0–10000 | `borg_caution.rs:328` |
| `caution_questor_holy_word_fail` | 14 | percent | 0–100 | `borg_caution.rs:328` |
| `caution_questor_life_stat_votes` | 5 | votes | 0–100 | `borg_caution.rs:330` |
| `caution_questor_life_loss` | 500 | hp | 0–10000 | `borg_caution.rs:332` |
| `caution_questor_emergency_holy_hp` | 250 | hp | 0–10000 | `borg_caution.rs:344` |
| `caution_questor_emergency_holy_fail` | 5 | percent | 0–100 | `borg_caution.rs:344` |
| `caution_questor_safe_holy_hp` | 550 | hp | 0–10000 | `borg_caution.rs:345` |
| `caution_questor_safe_holy_fail` | 15 | percent | 0–100 | `borg_caution.rs:345` |
| `caution_questor_healing_fail` | 15 | percent | 0–100 | `borg_caution.rs:346` |
| `caution_magi_sp_divisor` | 5 | divisor | 1–100 | `borg_caution.rs:356` |
| `caution_magi_chance` | 50 | percent | 0–100 | `borg_caution.rs:357` |
| `caution_restore_mana_sp_divisor` | 10 | divisor | 1–100 | `borg_caution.rs:366` |
| `caution_restore_mana_sp` | 70 | sp | 0–10000 | `borg_caution.rs:367` |
| `caution_restore_mana_maxsp` | 200 | sp | 0–10000 | `borg_caution.rs:367` |
| `caution_restore_mana_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:370` |
| `caution_restore_stat_votes` | 5 | votes | 0–100 | `borg_caution.rs:393` |
| `caution_restore_stat_hp` | 650 | hp | 0–10000 | `borg_caution.rs:395` |
| `caution_heal_unique_chance_bonus` | 10 | percent | 0–100 | `borg_caution.rs:414` |
| `caution_heal_lethal_chance_bonus` | 75 | percent | 0–100 | `borg_caution.rs:419` |
| `caution_heal_nonpriest_chance_penalty` | 25 | percent | 0–100 | `borg_caution.rs:421` |
| `caution_heal_risky_chance_penalty` | 5 | percent | 0–100 | `borg_caution.rs:426` |
| `caution_heal_band1_max_loss` | 15 | percent | 0–100 | `borg_caution.rs:429` |
| `caution_heal_band1_skip_chance` | 98 | percent | 0–100 | `borg_caution.rs:429` |
| `caution_heal_band2_min_loss` | 16 | percent | 0–100 | `borg_caution.rs:430` |
| `caution_heal_band2_max_loss` | 25 | percent | 0–100 | `borg_caution.rs:430` |
| `caution_heal_band2_skip_chance` | 95 | percent | 0–100 | `borg_caution.rs:430` |
| `caution_heal_band3_min_loss` | 26 | percent | 0–100 | `borg_caution.rs:431` |
| `caution_heal_band3_max_loss` | 50 | percent | 0–100 | `borg_caution.rs:431` |
| `caution_heal_band3_skip_chance` | 80 | percent | 0–100 | `borg_caution.rs:431` |
| `caution_heal_band4_min_loss` | 51 | percent | 0–100 | `borg_caution.rs:432` |
| `caution_heal_band4_max_loss` | 65 | percent | 0–100 | `borg_caution.rs:432` |
| `caution_heal_band4_skip_chance` | 50 | percent | 0–100 | `borg_caution.rs:432` |
| `caution_heal_band5_min_loss` | 66 | percent | 0–100 | `borg_caution.rs:433` |
| `caution_heal_band5_max_loss` | 74 | percent | 0–100 | `borg_caution.rs:433` |
| `caution_heal_band5_skip_chance` | 25 | percent | 0–100 | `borg_caution.rs:433` |
| `caution_heal_band6_min_loss` | 75 | percent | 0–100 | `borg_caution.rs:434` |
| `caution_heal_band6_skip_chance` | 1 | percent | 0–100 | `borg_caution.rs:434` |
| `caution_clw_min_loss` | 30 | percent | 0–100 | `borg_caution.rs:444` |
| `caution_clw_max_loss` | 40 | percent | 0–100 | `borg_caution.rs:445` |
| `caution_clw_max_level` | 10 | level | 1–51 | `borg_caution.rs:445` |
| `caution_clw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:447` |
| `caution_csw_min_loss` | 40 | percent | 0–100 | `borg_caution.rs:456` |
| `caution_csw_max_loss` | 50 | percent | 0–100 | `borg_caution.rs:457` |
| `caution_csw_max_level` | 20 | level | 1–51 | `borg_caution.rs:457` |
| `caution_csw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:459` |
| `caution_ccw_min_loss` | 50 | percent | 0–100 | `borg_caution.rs:468` |
| `caution_ccw_max_loss` | 55 | percent | 0–100 | `borg_caution.rs:468` |
| `caution_ccw_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:470` |
| `caution_emergency_ccw_hp` | 50 | hp | 0–10000 | `borg_caution.rs:480` |
| `caution_deep_ccw_depth` | 80 | depth | 0–127 | `borg_caution.rs:490` |
| `caution_deep_ccw_danger` | 50 | danger | 0–10000 | `borg_caution.rs:491` |
| `caution_deep_ccw_loss` | 20 | percent | 0–100 | `borg_caution.rs:492` |
| `caution_heal_device_min_loss` | 55 | percent | 0–100 | `borg_caution.rs:500` |
| `caution_conserve_heal_depth` | 98 | depth | 0–127 | `borg_caution.rs:520` |
| `caution_heal_potion_min_loss` | 50 | percent | 0–100 | `borg_caution.rs:530` |
| `caution_heal_all_min_loss` | 60 | percent | 0–100 | `borg_caution.rs:544` |
| `caution_heal_unique_reserve_min_loss` | 65 | percent | 0–100 | `borg_caution.rs:563` |
| `caution_heal_last_resort_loss` | 75 | percent | 0–100 | `borg_caution.rs:589` |
| `caution_cure_safe_danger_tenths` | 2 | tenths | 0–100 | `borg_caution.rs:603` |
| `caution_poison_cure_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:609` |
| `caution_poison_cure_fail` | 60 | percent | 0–100 | `borg_caution.rs:610` |
| `caution_poison_herbal_fail` | 60 | percent | 0–100 | `borg_caution.rs:611` |
| `caution_poison_healing_fail` | 60 | percent | 0–100 | `borg_caution.rs:626` |
| `caution_poison_holy_fail` | 60 | percent | 0–100 | `borg_caution.rs:627` |
| `caution_poison_critical_hp` | 2 | hp | 0–10000 | `borg_caution.rs:646` |
| `caution_poison_critical_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:646` |
| `caution_poison_reserve_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:684` |
| `caution_cut_cure_hp_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:702` |
| `caution_cut_cure_chance` | 20 | percent | 0–100 | `borg_caution.rs:702` |
| `caution_cut_critical_ccw_hp` | 10 | hp | 0–10000 | `borg_caution.rs:705` |
| `caution_cut_critical_hp` | 2 | hp | 0–10000 | `borg_caution.rs:713` |
| `caution_cut_critical_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:713` |
| `caution_cut_reserve_hp_divisor` | 20 | divisor | 1–100 | `borg_caution.rs:736` |
| `caution_desperate_stair_distance` | 3 | grids | 1–100 | `borg_caution.rs:872` |
| `caution_desperate_adjacent_up_distance` | 2 | grids | 1–100 | `borg_caution.rs:881` |
| `caution_desperate_scary_distance` | 20 | grids | 1–100 | `borg_caution.rs:886` |
| `caution_desperate_scary_level` | 20 | level | 1–51 | `borg_caution.rs:886` |
| `caution_desperate_unique_depth` | 5 | depth | 0–127 | `borg_caution.rs:912` |
| `caution_desperate_adjacent_down_distance` | 2 | grids | 1–100 | `borg_caution.rs:922` |
| `caution_retreat_max_distance` | 10 | grids | 1–100 | `borg_caution.rs:1009` |
| `caution_retreat_step_hp_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:1024` |
| `caution_retreat_step_danger_tenths` | 5 | tenths | 0–100 | `borg_caution.rs:1029` |
| `caution_retreat_surrounded_level` | 20 | level | 1–51 | `borg_caution.rs:1034` |
| `caution_retreat_surrounded_distance` | 5 | grids | 1–100 | `borg_caution.rs:1035` |
| `caution_retreat_surrounded_hp_tenths` | 15 | tenths | 0–100 | `borg_caution.rs:1036` |
| `caution_retreat_low_level_hp_multiplier` | 4 | multiplier | 0–20 | `borg_caution.rs:1043` |
| `caution_retreat_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1060` |
| `caution_back_surrounded_danger_tenths` | 12 | tenths | 0–100 | `borg_caution.rs:1143` |
| `caution_back_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1199` |
| `caution_back_fast_monster_distance` | 2 | grids | 1–100 | `borg_caution.rs:1209` |
| `caution_back_high_level` | 35 | level | 1–51 | `borg_caution.rs:1235` |
| `caution_back_high_level_danger_tenths` | 6 | tenths | 0–100 | `borg_caution.rs:1235` |
| `caution_back_low_level` | 35 | level | 1–51 | `borg_caution.rs:1243` |
| `caution_back_low_level_danger_tenths` | 8 | tenths | 0–100 | `borg_caution.rs:1243` |
| `caution_back_injured_level` | 10 | level | 1–51 | `borg_caution.rs:1264` |
| `caution_back_freedom_depth` | 85 | depth | 0–127 | `borg_caution.rs:1273` |
| `caution_move_antisummon_delay` | 50 | turns | 0–100000 | `borg_caution.rs:1330` |
| `caution_move_max_hp` | 500 | hp | 0–10000 | `borg_caution.rs:1333` |
| `caution_one_blow_monster_age` | 2 | turns | 0–100000 | `borg_caution.rs:1365` |
| `caution_nasty_light_timeout` | 250 | turns | 0–100000 | `borg_caution.rs:1425` |
| `caution_questor_arrival_turns` | 10 | turns | 0–100000 | `borg_caution.rs:1438` |
| `caution_escape_limit` | 3 | count | 0–1000 | `borg_caution.rs:1472` |
| `caution_escape_absolute_limit` | 55 | count | 0–1000 | `borg_caution.rs:1473` |
| `caution_escape_limit_max_depth` | 98 | depth | 0–127 | `borg_caution.rs:1475` |
| `caution_escape_flee_limit` | 3 | count | 0–1000 | `borg_caution.rs:1481` |
| `caution_swap_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:1615` |
| `caution_recall_prep_turns` | 2 | turns | 0–100000 | `borg_caution.rs:1643` |
| `caution_restock_town_delay` | 200 | turns | 0–100000 | `borg_caution.rs:1657` |
| `caution_restock_ccw_count` | 2 | count | 0–1000 | `borg_caution.rs:1671` |
| `caution_restock_food_count` | 3 | count | 0–1000 | `borg_caution.rs:1672` |
| `caution_restock_fuel_count` | 2 | count | 0–1000 | `borg_caution.rs:1673` |
| `caution_restock_level_delay` | 400 | turns | 0–100000 | `borg_caution.rs:1674` |
| `caution_flee_danger_hp_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:1685` |
| `caution_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:1690` |
| `caution_town_flee_max_level` | 50 | level | 1–51 | `borg_caution.rs:1704` |
| `caution_stair_low_level` | 5 | level | 1–51 | `borg_caution.rs:1719` |
| `caution_stair_danger_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:1719` |
| `caution_breeder_flee_max_level` | 15 | level | 1–51 | `borg_caution.rs:1799` |
| `caution_desperate_max_level` | 35 | level | 1–51 | `borg_caution.rs:1809` |
| `caution_retreat_danger_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:1822` |
| `caution_back_danger_tenths` | 4 | tenths | 0–100 | `borg_caution.rs:1851` |
| `caution_confused_emergency_loss` | 300 | hp | 0–10000 | `borg_caution.rs:1863` |
| `caution_fear_cure_chance` | 70 | percent | 0–100 | `borg_caution.rs:1885` |
| `caution_fear_berserk_fail` | 25 | percent | 0–100 | `borg_caution.rs:1891` |
| `caution_fear_hero_fail` | 25 | percent | 0–100 | `borg_caution.rs:1892` |
| `caution_fear_holy_fail` | 25 | percent | 0–100 | `borg_caution.rs:1899` |
| `caution_flee_low_hp_divisor` | 3 | divisor | 1–100 | `borg_caution.rs:1907` |
| `caution_flee_moderate_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:1908` |
| `caution_flee_hp_per_level` | 3 | hp_per_level | 0–100 | `borg_caution.rs:1909` |
| `caution_flee_ccw_count` | 3 | count | 0–1000 | `borg_caution.rs:1910` |
| `caution_flee_heal_count` | 1 | count | 0–1000 | `borg_caution.rs:1911` |
| `caution_flee_low_hp_chance` | 25 | percent | 0–100 | `borg_caution.rs:1914` |
| `caution_flee_status_hp_divisor` | 2 | divisor | 1–100 | `borg_caution.rs:1922` |
| `caution_flee_status_chance` | 25 | percent | 0–100 | `borg_caution.rs:1925` |
| `caution_reserve_critical_hp_divisor` | 10 | divisor | 1–100 | `borg_caution.rs:1934` |
| `caution_reserve_escape_count` | 2 | count | 0–1000 | `borg_caution.rs:1936` |
| `caution_reserve_danger_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:1937` |
| `caution_reserve_heal_count` | 5 | count | 0–1000 | `borg_caution.rs:1939` |
| `caution_reserve_surplus_hp_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:1939` |
| `caution_reserve_unique_loss` | 600 | hp | 0–10000 | `borg_caution.rs:1941` |
| `caution_reserve_unique_depth` | 85 | depth | 0–127 | `borg_caution.rs:1943` |
| `caution_recall_danger_multiplier` | 2 | multiplier | 0–20 | `borg_caution.rs:1970` |
| `caution_unknown_danger_multiplier` | 4 | multiplier | 0–20 | `borg_caution.rs:1978` |
| `caution_unknown_max_level` | 20 | level | 1–51 | `borg_caution.rs:1979` |
| `caution_stair_danger_max_level` | 49 | level | 1–51 | `borg_caution.rs:2019` |
| `caution_stair_min_food` | 2 | count | 0–1000 | `borg_caution.rs:2044` |
| `caution_sell_max_level` | 25 | level | 1–51 | `borg_caution.rs:2052` |
| `caution_sell_max_gold` | 25000 | gold | 0–1000000 | `borg_caution.rs:2053` |
| `caution_sell_min_count` | 13 | count | 0–1000 | `borg_caution.rs:2054` |
| `caution_recall_mana_maxsp` | 60 | sp | 0–10000 | `borg_caution.rs:2077` |
| `caution_recall_mana_divisor` | 4 | divisor | 1–100 | `borg_caution.rs:2078` |
| `caution_recall_phase_tolerance` | 50 | percent | 0–100 | `borg_caution.rs:2086` |
| `caution_recall_phase_horizon` | 1 | turns | 1–100000 | `borg_caution.rs:2086` |
| `caution_recall_phase_fail` | 30 | percent | 0–100 | `borg_caution.rs:2088` |
| `caution_recall_portal_fail` | 30 | percent | 0–100 | `borg_caution.rs:2089` |
| `caution_recall_cure_loss` | 100 | hp | 0–10000 | `borg_caution.rs:2096` |
| `caution_recall_heal_loss` | 150 | hp | 0–10000 | `borg_caution.rs:2105` |
| `caution_retreat_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1014` |
| `caution_back_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1223` |
| `caution_danger_horizon` | 1 | turns | 1–10 | `borg_caution.rs:1514` |
| `caution_recall_rest_turns` | 500 | turns | 1–9999 | `borg_caution.rs:1630` |


## P1.2: danger and escape (#220)

The source locations in this section refer to **reborn `30ab9d66`**, before
these hooks. They are separate from caution's original locations above.
The registry's single generated enum/table keeps indices, names and defaults
in step. Existing configuration precedence, recording and dossier hashes
apply unchanged to the added names.

The new controls cover per-blow hit-power estimates and fear supplements;
per-spell status/inventory/summon risk weights; danger caps; low-level,
sleep/confusion and stuck-monster adjustments; regional fear; the escape
priority ladder; spell-failure tolerances; HP/SP/depth/count cutoffs; and
Phase Door/teleport sampling budgets and danger horizons. These are the
vendored policy's existing values, not recommended tuning settings.

`hit_power`, `save_points`, `sleep_rating`, `energy_tenths`, `ac`,
`coefficient` and `freedom_score` are native model units. The stair-freedom
penalty multiplies doubled distance (the existing `double_distance`), not
ordinary grids. The hit-luck bias is in percentage points and can exceed
100 before the existing probability cap; its default is 150. In contrast,
ordinary probability thresholds stay in 0–100. Level offsets use `levels`.

Sampling tolerances count scary samples, not a newly normalized percentage.
The default sample budget is 100. If that budget changes, choose the
landing tolerances on the same count scale. Sampling budgets and danger
horizons must be positive; divisions have positive denominators. No RNG
call or search order changes when the defaults are used.

The fast-player threshold is bounded at its original 135 or below. Assumed
player speeds are 110–189, leaving room for the unchanged +10 haste in the
200-entry energy table. The low-HP monster-speed bonus is 0–3, so an
override cannot increase it beyond the original adjustment. These bounds
protect table indexing; they are not a measurement of gameplay benefit.
The non-questor unique-count cutoffs stop at 9, below the questor encoding
of 10 (`borg_flow_kill.rs`, the nearby-monster scan).

### Retained constants and mechanics-model data

These remain literals. Extracting policy values does not correct or retune
the vendored mechanics model:

- Engine IDs (classes, spells, blow effects, terrain, curses), feature
  masks, table boundaries, array dimensions, the regional `/ 11` geometry,
  inventory offset 5, the eight adjacent grids, unit adjacency and loop
  counters/sentinels. Zero indicates absent/dead/no damage, or initializes
  an accumulator. A zero protected-summon fear branch remains a categorical
  exemption, not a knob that enables a new risk term.
- Actual blow dice, HP-based breath damage/caps, elemental resistance and
  immunity reductions, and spell-power damage expressions stay with the
  copied attack model. See upstream `lib/gamedata/monster.txt`,
  `monster_spell.txt`, `src/list-elements.h` and the corresponding handlers
  at engine `42cd156ac`. The model is legacy data, **not a claim that every
  expression matches today's engine**. Supplementary fear and status
  penalties are parameters; the direct damage formulas are not.
- A shield's +50 AC is engine data (`src/player-calcs.c:2160–2162`). The
  protection-from-evil divisor is now a parameter: the legacy factor 2 is
  an approximation, while `src/mon-attack.c:597–609` uses the player's level
  in `randint0(100) + level > 50`. Haste/slow's 10 speed,
  the percentage denominator 100, the game's 100 energy per action and
  the danger model's tenth-action conversion/ceiling remain units and
  arithmetic, not tunable thresholds.
- The 10/20 energy branch boundaries delimit one action and move-then-cast
  in the one-step movement simulator. Its cap of 20 belongs to that
  algorithm. `danger_minimum_monster_energy` is the separate policy floor.
- Stat 3 is the minimum drainable stat; saving/theft escape checks at 100
  represent a guaranteed roll. Crush's strict `HP < 4 * clevel` condition
  is game damage (`lib/gamedata/class.txt:1038–1044`,
  `src/project-mon.c:1004–1013`), not a safety margin. The hit-power values
  are deliberately named: for example, the legacy HURT 60 and POISON 5
  differ from `blow_effects.txt`'s current 40 and 20. Defaults preserve the
  old model rather than silently carrying a mechanics correction.
- Questor encoding 10 and game levels 99/100 remain categorical game data.
  Town depth 0 and the starvation check on the first dungeon level remain
  locations, not policy cutoffs. The 80/95/97 depth gates are parameters.
- Phase's radius 10, the legacy teleport radius 100 and half-radius sampling
  interval remain with the copied landing model, as do lash/spit reach
  checks at 3/4. The fixed lash/spit damage estimates themselves are named.
  The landing model is not today's exact teleport distribution: the manual
  ch. 13 and engine `effect_handler_TELEPORT` describe span-based long
  teleports and ±25% distance. Source carry #595 is already flagged by the
  refresh report; that correction would change default play and is not
  part of this extraction.
- Raw keys, direction 5, recall reply characters, local-map boundaries and
  panel offsets are engine interface data. The freedom score's distance
  subtraction is the definition of that score; its base values, close
  range and distance penalty are parameters.

### Additional parameter catalogue

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `danger_monblow_hurt_fear` | 400 | danger | 0–5000 | `borg_danger.rs:208` |
| `danger_monblow_hurt_hit_power` | 60 | hit_power | 0–1000 | `borg_danger.rs:210` |
| `danger_monblow_poison_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:218` |
| `danger_monblow_poison_fear` | 10 | danger | 0–5000 | `borg_danger.rs:226` |
| `danger_monblow_disenchant_hit_power` | 20 | hit_power | 0–1000 | `borg_danger.rs:234` |
| `danger_monblow_disenchant_fear` | 500 | danger | 0–5000 | `borg_danger.rs:239` |
| `danger_monblow_drain_charges_fear` | 20 | danger | 0–5000 | `borg_danger.rs:248` |
| `danger_monblow_drain_charges_hit_power` | 15 | hit_power | 0–1000 | `borg_danger.rs:249` |
| `danger_monblow_eat_gold_fear` | 50 | danger | 0–5000 | `borg_danger.rs:259` |
| `danger_monblow_eat_gold_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:261` |
| `danger_monblow_eat_gold_fear_2` | 5 | danger | 0–5000 | `borg_danger.rs:274` |
| `danger_monblow_eat_item_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:282` |
| `danger_monblow_eat_item_fear` | 5 | danger | 0–5000 | `borg_danger.rs:289` |
| `danger_monblow_eat_food_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:297` |
| `danger_monblow_eat_food_fear` | 5 | danger | 0–5000 | `borg_danger.rs:302` |
| `danger_monblow_eat_light_hit_power` | 5 | hit_power | 0–1000 | `borg_danger.rs:310` |
| `danger_monblow_eat_light_fear` | 5 | danger | 0–5000 | `borg_danger.rs:319` |
| `danger_monblow_acid_fear` | 200 | danger | 0–5000 | `borg_danger.rs:337` |
| `danger_monblow_elec_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:348` |
| `danger_monblow_elec_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:356` |
| `danger_monblow_fire_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:367` |
| `danger_monblow_fire_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:375` |
| `danger_monblow_cold_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:386` |
| `danger_monblow_cold_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:394` |
| `danger_monblow_blind_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:402` |
| `danger_monblow_blind_fear` | 10 | danger | 0–5000 | `borg_danger.rs:407` |
| `danger_monblow_blind_fear_2` | 75 | danger | 0–5000 | `borg_danger.rs:409` |
| `danger_monblow_confuse_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:418` |
| `danger_monblow_confuse_fear` | 200 | danger | 0–5000 | `borg_danger.rs:423` |
| `danger_monblow_confuse_fear_2` | 200 | danger | 0–5000 | `borg_danger.rs:425` |
| `danger_monblow_terrify_hit_power` | 10 | hit_power | 0–1000 | `borg_danger.rs:434` |
| `danger_monblow_terrify_damage_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:439` |
| `danger_monblow_paralyze_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:447` |
| `danger_monblow_paralyze_fear` | 200 | danger | 0–5000 | `borg_danger.rs:451` |
| `danger_monblow_lose_str_fear` | 150 | danger | 0–5000 | `borg_danger.rs:474` |
| `danger_monblow_lose_str_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:477` |
| `danger_monblow_lose_dex_fear` | 150 | danger | 0–5000 | `borg_danger.rs:498` |
| `danger_monblow_lose_dex_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:501` |
| `danger_monblow_lose_con_fear` | 150 | danger | 0–5000 | `borg_danger.rs:526` |
| `danger_monblow_lose_con_fear_2` | 100 | danger | 0–5000 | `borg_danger.rs:529` |
| `danger_monblow_lose_int_fear` | 150 | danger | 0–5000 | `borg_danger.rs:553` |
| `danger_monblow_lose_int_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:556` |
| `danger_monblow_lose_wis_fear` | 150 | danger | 0–5000 | `borg_danger.rs:577` |
| `danger_monblow_lose_wis_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:580` |
| `danger_monblow_lose_all_hit_power` | 2 | hit_power | 0–1000 | `borg_danger.rs:589` |
| `danger_monblow_shatter_hit_power` | 60 | hit_power | 0–1000 | `borg_danger.rs:596` |
| `danger_monblow_shatter_fear` | 150 | danger | 0–5000 | `borg_danger.rs:598` |
| `danger_monblow_hallu_fear` | 250 | danger | 0–5000 | `borg_danger.rs:636` |
| `danger_rsf_shriek_fear` | 5 | danger | 0–5000 | `borg_danger.rs:795` |
| `danger_rsf_br_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:838` |
| `danger_rsf_br_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:857` |
| `danger_rsf_br_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:876` |
| `danger_rsf_br_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:895` |
| `danger_rsf_br_pois_fear` | 20 | danger | 0–5000 | `borg_danger.rs:917` |
| `danger_rsf_br_neth_fear` | 125 | danger | 0–5000 | `borg_danger.rs:931` |
| `danger_rsf_br_light_fear` | 20 | danger | 0–5000 | `borg_danger.rs:947` |
| `danger_rsf_br_light_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:949` |
| `danger_rsf_br_dark_fear` | 20 | danger | 0–5000 | `borg_danger.rs:968` |
| `danger_rsf_br_dark_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:970` |
| `danger_rsf_br_soun_fear` | 500 | danger | 0–5000 | `borg_danger.rs:988` |
| `danger_rsf_br_soun_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:991` |
| `danger_rsf_br_soun_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:994` |
| `danger_rsf_br_chao_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1007` |
| `danger_rsf_br_chao_fear_2` | 200 | danger | 0–5000 | `borg_danger.rs:1011` |
| `danger_rsf_br_dise_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1026` |
| `danger_rsf_br_nexu_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1042` |
| `danger_rsf_br_time_fear` | 250 | danger | 0–5000 | `borg_danger.rs:1052` |
| `danger_rsf_br_iner_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1062` |
| `danger_rsf_br_grav_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1072` |
| `danger_rsf_br_grav_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1078` |
| `danger_rsf_br_grav_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1081` |
| `danger_rsf_br_shar_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1098` |
| `danger_rsf_br_plas_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1112` |
| `danger_rsf_br_plas_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1115` |
| `danger_rsf_br_plas_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1118` |
| `danger_rsf_br_wall_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1133` |
| `danger_rsf_br_wall_fear_2` | 500 | danger | 0–5000 | `borg_danger.rs:1136` |
| `danger_rsf_br_wall_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:1139` |
| `danger_rsf_ba_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1172` |
| `danger_rsf_ba_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1187` |
| `danger_rsf_ba_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1202` |
| `danger_rsf_ba_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1217` |
| `danger_rsf_ba_pois_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1235` |
| `danger_rsf_ba_shar_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1247` |
| `danger_rsf_ba_neth_fear` | 250 | danger | 0–5000 | `borg_danger.rs:1259` |
| `danger_rsf_ba_wate_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1269` |
| `danger_rsf_ba_wate_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1272` |
| `danger_rsf_ba_wate_fear_3` | 50 | danger | 0–5000 | `borg_danger.rs:1278` |
| `danger_rsf_ba_wate_fear_4` | 20 | danger | 0–5000 | `borg_danger.rs:1280` |
| `danger_rsf_ba_mana_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1287` |
| `danger_rsf_ba_holy_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1293` |
| `danger_rsf_ba_dark_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1308` |
| `danger_rsf_ba_dark_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:1310` |
| `danger_rsf_ba_light_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1326` |
| `danger_rsf_ba_light_fear_2` | 20 | danger | 0–5000 | `borg_danger.rs:1328` |
| `danger_rsf_storm_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1339` |
| `danger_rsf_storm_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1342` |
| `danger_rsf_drain_mana_fear` | 100 | danger | 0–5000 | `borg_danger.rs:1350` |
| `danger_rsf_brain_smash_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1362` |
| `danger_rsf_bo_acid_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1390` |
| `danger_rsf_bo_elec_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1406` |
| `danger_rsf_bo_fire_fear` | 40 | danger | 0–5000 | `borg_danger.rs:1422` |
| `danger_rsf_bo_cold_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1438` |
| `danger_rsf_bo_neth_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1465` |
| `danger_rsf_bo_wate_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1476` |
| `danger_rsf_bo_wate_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1479` |
| `danger_rsf_bo_wate_fear_3` | 20 | danger | 0–5000 | `borg_danger.rs:1485` |
| `danger_rsf_bo_wate_fear_4` | 20 | danger | 0–5000 | `borg_danger.rs:1487` |
| `danger_rsf_bo_mana_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1495` |
| `danger_rsf_bo_plas_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1506` |
| `danger_rsf_bo_plas_fear_2` | 1000 | danger | 0–5000 | `borg_danger.rs:1509` |
| `danger_rsf_bo_ice_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1517` |
| `danger_rsf_bo_ice_fear_2` | 50 | danger | 0–5000 | `borg_danger.rs:1523` |
| `danger_rsf_bo_ice_fear_3` | 1000 | danger | 0–5000 | `borg_danger.rs:1526` |
| `danger_rsf_scare_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1560` |
| `danger_rsf_blind_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1572` |
| `danger_rsf_conf_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1583` |
| `danger_rsf_slow_fear` | 5 | danger | 0–5000 | `borg_danger.rs:1594` |
| `danger_rsf_hold_fear` | 150 | danger | 0–5000 | `borg_danger.rs:1604` |
| `danger_rsf_haste_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1609` |
| `danger_rsf_heal_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1614` |
| `danger_rsf_tport_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1623` |
| `danger_rsf_tele_to_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1628` |
| `danger_rsf_tele_self_to_fear` | 20 | danger | 0–5000 | `borg_danger.rs:1633` |
| `danger_rsf_tele_away_fear` | 10 | danger | 0–5000 | `borg_danger.rs:1638` |
| `danger_rsf_tele_level_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1644` |
| `danger_rsf_darkness_fear` | 5 | danger | 0–5000 | `borg_danger.rs:1650` |
| `danger_rsf_traps_fear` | 50 | danger | 0–5000 | `borg_danger.rs:1655` |
| `danger_rsf_forget_fear` | 500 | danger | 0–5000 | `borg_danger.rs:1662` |
| `danger_rsf_forget_fear_2` | 30 | danger | 0–5000 | `borg_danger.rs:1664` |
| `danger_rsf_shapechange_fear` | 200 | danger | 0–5000 | `borg_danger.rs:1671` |
| `danger_rsf_s_kin_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1688` |
| `danger_rsf_s_kin_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1691` |
| `danger_summon_unique_percent` | 75 | percent | 0–100 | `borg_danger.rs:1696` |
| `danger_rsf_s_hi_demon_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1706` |
| `danger_rsf_s_hi_demon_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1709` |
| `danger_rsf_s_monster_power_multiplier` | 5 | multiplier | 0–50 | `borg_danger.rs:1723` |
| `danger_rsf_s_monsters_power_multiplier` | 7 | multiplier | 0–50 | `borg_danger.rs:1733` |
| `danger_rsf_s_animal_power_multiplier` | 5 | multiplier | 0–50 | `borg_danger.rs:1747` |
| `danger_rsf_s_hydra_power_multiplier` | 2 | multiplier | 0–50 | `borg_danger.rs:1762` |
| `danger_rsf_s_hydra_power_multiplier_2` | 5 | multiplier | 0–50 | `borg_danger.rs:1765` |
| `danger_rsf_s_ainu_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1780` |
| `danger_rsf_s_ainu_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1783` |
| `danger_rsf_s_demon_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1798` |
| `danger_rsf_s_demon_power_multiplier_2` | 7 | multiplier | 0–50 | `borg_danger.rs:1801` |
| `danger_rsf_s_hi_undead_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1816` |
| `danger_rsf_s_hi_undead_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1819` |
| `danger_rsf_s_hi_dragon_power_multiplier` | 6 | multiplier | 0–50 | `borg_danger.rs:1834` |
| `danger_rsf_s_hi_dragon_power_multiplier_2` | 12 | multiplier | 0–50 | `borg_danger.rs:1837` |
| `danger_rsf_s_unique_power_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:1852` |
| `danger_rsf_s_unique_power_multiplier_2` | 6 | multiplier | 0–50 | `borg_danger.rs:1855` |
| `danger_unknown_physical` | 1000 | danger | 0–5000 | `borg_danger.rs:175` |
| `danger_knockout_max_sides` | 3 | count | 0–1000 | `borg_danger.rs:207` |
| `danger_knockout_min_dice` | 5 | count | 0–1000 | `borg_danger.rs:207` |
| `danger_gold_theft_low_level` | 5 | level | 1–51 | `borg_danger.rs:258` |
| `danger_gold_theft_min_gold` | 100 | gold | 0–1000000 | `borg_danger.rs:267` |
| `danger_gold_theft_max_gold` | 100000 | gold | 0–1000000 | `borg_danger.rs:270` |
| `danger_food_theft_reserve` | 5 | count | 0–1000 | `borg_danger.rs:298` |
| `danger_fuel_theft_reserve` | 5 | count | 0–1000 | `borg_danger.rs:315` |
| `danger_str_drain_low_stat` | 10 | stat | 3–238 | `borg_danger.rs:476` |
| `danger_dex_drain_low_stat` | 10 | stat | 3–238 | `borg_danger.rs:500` |
| `danger_con_drain_low_stat` | 8 | stat | 3–238 | `borg_danger.rs:528` |
| `danger_shatter_ac_cap` | 150 | ac | 0–1000 | `borg_danger.rs:595` |
| `danger_shatter_ac_divisor` | 250 | divisor | 1–1000 | `borg_danger.rs:595` |
| `danger_exp10_fear` | 100 | danger | 0–5000 | `borg_danger.rs:608` |
| `danger_exp_ignore_level` | 50 | level | 1–51 | `borg_danger.rs:608` |
| `danger_exp20_fear` | 150 | danger | 0–5000 | `borg_danger.rs:609` |
| `danger_exp40_fear` | 200 | danger | 0–5000 | `borg_danger.rs:610` |
| `danger_exp80_fear` | 250 | danger | 0–5000 | `borg_danger.rs:611` |
| `danger_melee_hit_luck_percent` | 150 | percent | 100–300 | `borg_danger.rs:657` |
| `danger_melee_hit_ac_scale` | 300 | coefficient | 0–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_ac_divisor` | 4 | divisor | 1–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_power_scale` | 3 | divisor | 1–1000 | `borg_danger.rs:657` |
| `danger_melee_hit_floor_percent` | 5 | percent | 0–100 | `borg_danger.rs:663` |
| `danger_summon_protected_divisor` | 100 | divisor | 1–1000 | `borg_danger.rs:707` |
| `danger_summon_corridor_divisor` | 1000 | divisor | 1–10000 | `borg_danger.rs:710` |
| `danger_unknown_spell` | 1000 | danger | 0–5000 | `borg_danger.rs:766` |
| `danger_brain_smash_save_weight` | 2 | multiplier | 0–50 | `borg_danger.rs:1362` |
| `danger_cause4_save_bias` | 120 | save_points | 0–1000 | `borg_danger.rs:1374` |
| `danger_forget_low_sp` | 15 | sp | 0–10000 | `borg_danger.rs:1661` |
| `danger_ignore_side_effects_depth` | 75 | depth | 0–127 | `borg_danger.rs:1876` |
| `danger_corridor_spell_tenths` | 7 | tenths | 0–30 | `borg_danger.rs:1901` |
| `danger_worst_spell_average_tenths` | 15 | tenths | 0–30 | `borg_danger.rs:1948` |
| `danger_worst_spell_hp_tenths` | 8 | tenths | 0–30 | `borg_danger.rs:1948` |
| `danger_sleep_level_margin` | 15 | level | 1–51 | `borg_danger.rs:1972` |
| `danger_ghost_monster` | 100 | danger | 0–5000 | `borg_danger.rs:2035` |
| `danger_monster_max_distance` | 20 | grids | 1–100 | `borg_danger.rs:2067` |
| `danger_fast_player_threshold` | 135 | speed | 110–135 | `borg_danger.rs:2072` |
| `danger_fast_player_unique_speed` | 120 | speed | 110–189 | `borg_danger.rs:2073` |
| `danger_fast_player_normal_speed` | 125 | speed | 110–189 | `borg_danger.rs:2073` |
| `danger_low_hp_speed_limit` | 20 | hp | 0–10000 | `borg_danger.rs:2085` |
| `danger_low_hp_monster_speed_bonus` | 3 | speed | 0–3 | `borg_danger.rs:2086` |
| `danger_minimum_monster_energy` | 10 | energy_tenths | 1–100 | `borg_danger.rs:2122` |
| `danger_stuck_bounce_limit` | 1200 | count | 0–10000 | `borg_danger.rs:2135` |
| `danger_stuck_level_turns` | 25000 | turns | 0–100000 | `borg_danger.rs:2135` |
| `danger_stuck_melee_divisor` | 5 | divisor | 1–1000 | `borg_danger.rs:2136` |
| `danger_breeder_melee_max_level` | 20 | level | 1–51 | `borg_danger.rs:2150` |
| `danger_breeder_melee_extra_tenths` | 15 | tenths | 0–30 | `borg_danger.rs:2152` |
| `danger_friends_melee_max_level` | 20 | level | 1–51 | `borg_danger.rs:2158` |
| `danger_friends_melee_low_level` | 15 | level | 1–51 | `borg_danger.rs:2159` |
| `danger_friends_melee_low_extra_tenths` | 18 | tenths | 0–30 | `borg_danger.rs:2161` |
| `danger_friends_melee_extra_tenths` | 13 | tenths | 0–30 | `borg_danger.rs:2164` |
| `danger_sleeping_melee_alertness` | 5 | sleep_rating | 0–1000 | `borg_danger.rs:2170` |
| `danger_sleeping_melee_min_level` | 25 | level | 1–51 | `borg_danger.rs:2172` |
| `danger_sleeping_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2173` |
| `danger_sleep2_melee_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2183` |
| `danger_sleep2_melee_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2183` |
| `danger_sleep2_melee_divisor` | 3 | divisor | 1–1000 | `borg_danger.rs:2186` |
| `danger_sleep_melee_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2192` |
| `danger_sleep_melee_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2192` |
| `danger_sleep_melee_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2195` |
| `danger_confused_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2204` |
| `danger_stunned_melee_numerator` | 10 | coefficient | 0–1000 | `borg_danger.rs:2207` |
| `danger_stunned_melee_divisor` | 13 | divisor | 1–1000 | `borg_danger.rs:2207` |
| `danger_confuse_spell_weak_level` | 20 | level | 1–51 | `borg_danger.rs:2211` |
| `danger_confuse_spell_weak_hp_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2211` |
| `danger_confuse_spell_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2214` |
| `danger_stuck_spell_divisor` | 5 | divisor | 1–1000 | `borg_danger.rs:2401` |
| `danger_breeder_spell_max_level` | 20 | level | 1–51 | `borg_danger.rs:2405` |
| `danger_breeder_spell_extra_tenths` | 12 | tenths | 0–30 | `borg_danger.rs:2406` |
| `danger_friends_spell_max_level` | 20 | level | 1–51 | `borg_danger.rs:2410` |
| `danger_friends_spell_extra_tenths` | 12 | tenths | 0–30 | `borg_danger.rs:2411` |
| `danger_sleeping_spell_alertness` | 5 | sleep_rating | 0–1000 | `borg_danger.rs:2416` |
| `danger_sleeping_spell_min_level` | 25 | level | 1–51 | `borg_danger.rs:2418` |
| `danger_sleeping_spell_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2419` |
| `danger_sleep2_spell_linear_level` | 15 | level | 1–51 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_subtract` | 10 | levels | 0–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_divisor` | 4 | divisor | 1–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_multiplier` | 3 | multiplier | 0–50 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_level_add` | 10 | levels | 0–1000 | `borg_danger.rs:2430` |
| `danger_sleep2_spell_divisor` | 3 | divisor | 1–1000 | `borg_danger.rs:2437` |
| `danger_sleep_spell_distance_offset` | 2 | count | 0–1000 | `borg_danger.rs:2448` |
| `danger_confused_spell_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:2452` |
| `danger_stunned_spell_numerator` | 10 | coefficient | 0–1000 | `borg_danger.rs:2456` |
| `danger_stunned_spell_divisor` | 13 | divisor | 1–1000 | `borg_danger.rs:2456` |
| `danger_confuse_spell_divisor` | 6 | divisor | 1–1000 | `borg_danger.rs:2459` |
| `danger_frequency_low_percent` | 11 | percent | 0–100 | `borg_danger.rs:2467` |
| `danger_frequency_low_tenths` | 4 | tenths | 0–30 | `borg_danger.rs:2468` |
| `danger_frequency_mid_percent` | 26 | percent | 0–100 | `borg_danger.rs:2469` |
| `danger_frequency_mid_tenths` | 6 | tenths | 0–30 | `borg_danger.rs:2470` |
| `danger_frequency_high_percent` | 51 | percent | 0–100 | `borg_danger.rs:2471` |
| `danger_frequency_high_tenths` | 8 | tenths | 0–30 | `borg_danger.rs:2472` |
| `danger_maximum` | 2000 | danger | 0–5000 | `borg_danger.rs:2487` |
| `danger_regional_fear_max_depth` | 80 | depth | 0–127 | `borg_danger.rs:2544` |
| `danger_morgoth_regional_fear_cap` | 300 | danger | 0–5000 | `borg_danger.rs:2549` |
| `danger_monster_fear_bounce_limit` | 200 | count | 0–1000 | `borg_danger.rs:2556` |
| `danger_rsf_s_kin_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1685` |
| `danger_rsf_s_hi_demon_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1703` |
| `danger_rsf_s_hydra_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1759` |
| `danger_rsf_s_ainu_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1777` |
| `danger_rsf_s_demon_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1795` |
| `danger_rsf_s_hi_undead_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1813` |
| `danger_rsf_s_unique_protected_power_multiplier` | 1 | multiplier | 0–50 | `borg_danger.rs:1849` |
| `escape_antisummon_reset_turns` | 50 | turns | 0–100000 | `borg_escape.rs:77` |
| `escape_recall_fail` | 60 | percent | 0–100 | `borg_escape.rs:95` |
| `escape_recall_depth_window` | 5 | depth | 0–127 | `borg_escape.rs:107` |
| `escape_recall_reset_min_depth` | 80 | depth | 0–127 | `borg_escape.rs:111` |
| `escape_recall_reset_unique_count` | 5 | count | 0–1000 | `borg_escape.rs:127` |
| `escape_surround_distance` | 3 | grids | 1–100 | `borg_escape.rs:197` |
| `escape_surround_single_exit_count` | 1 | count | 1–8 | `borg_escape.rs:265` |
| `escape_surround_single_monster_count` | 1 | count | 1–8 | `borg_escape.rs:265` |
| `escape_town_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:316` |
| `escape_town_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:319` |
| `escape_town_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:320` |
| `escape_town_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:320` |
| `escape_dungeon_stair_value` | 1000 | freedom_score | 0–5000 | `borg_escape.rs:330` |
| `escape_dungeon_stair_close_distance` | 4 | grids | 1–100 | `borg_escape.rs:333` |
| `escape_dungeon_stair_close_value` | 2000 | freedom_score | 0–5000 | `borg_escape.rs:334` |
| `escape_dungeon_stair_distance_penalty` | 500 | freedom_score | 0–5000 | `borg_escape.rs:334` |
| `escape_landing_samples` | 100 | count | 1–1000 | `borg_escape.rs:362` |
| `escape_landing_trials` | 100 | count | 1–1000 | `borg_escape.rs:368` |
| `escape_unknown_landing_max_hp` | 30 | hp | 0–10000 | `borg_escape.rs:418` |
| `escape_unknown_teleport_level_turns` | 2000 | turns | 0–100000 | `borg_escape.rs:487` |
| `escape_shadow_shift_min_hp` | 12 | hp | 0–10000 | `borg_escape.rs:549` |
| `escape_dimension_door_range` | 50 | grids | 1–100 | `borg_escape.rs:558` |
| `escape_dimension_door_horizon` | 2 | turns | 1–10 | `borg_escape.rs:601` |
| `escape_near_unique_max_count` | 8 | count | 1–9 | `borg_escape.rs:650` |
| `escape_normal_fail` | 25 | percent | 0–100 | `borg_escape.rs:659` |
| `escape_healthy_hp_percent` | 70 | percent | 0–100 | `borg_escape.rs:662` |
| `escape_healthy_fail` | 10 | percent | 0–100 | `borg_escape.rs:663` |
| `escape_stunned_fail` | 35 | percent | 0–100 | `borg_escape.rs:668` |
| `escape_morgoth_hold_hp_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:689` |
| `escape_morgoth_hold_glyph_count` | 3 | count | 0–1000 | `borg_escape.rs:707` |
| `escape_risky_danger_extra_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:725` |
| `escape_emergency_danger_tenths` | 45 | tenths | 0–100 | `borg_escape.rs:733` |
| `escape_morgoth_danger_tenths` | 40 | tenths | 0–100 | `borg_escape.rs:734` |
| `escape_morgoth_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:737` |
| `escape_sauron_danger_tenths` | 30 | tenths | 0–100 | `borg_escape.rs:738` |
| `escape_sauron_max_hp` | 600 | hp | 0–10000 | `borg_escape.rs:741` |
| `escape_deep_unique_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:742` |
| `escape_deep_unique_depth` | 95 | depth | 0–127 | `borg_escape.rs:744` |
| `escape_deep_unique_max_hp` | 550 | hp | 0–10000 | `borg_escape.rs:745` |
| `escape_unique_danger_tenths` | 17 | tenths | 0–100 | `borg_escape.rs:746` |
| `escape_ordinary_danger_tenths` | 15 | tenths | 0–100 | `borg_escape.rs:747` |
| `escape_emergency_fail` | 15 | percent | 0–100 | `borg_escape.rs:749` |
| `escape_initial_fail_discount` | 10 | percent | 0–100 | `borg_escape.rs:753` |
| `escape_emergency_fail_extra` | 9 | percent | 0–100 | `borg_escape.rs:762` |
| `escape_emergency_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:772` |
| `escape_emergency_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:772` |
| `escape_last_ditch_max_level` | 10 | level | 1–51 | `borg_escape.rs:790` |
| `escape_last_ditch_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:791` |
| `escape_last_ditch_dimension_fail` | 90 | percent | 0–100 | `borg_escape.rs:793` |
| `escape_critical_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:805` |
| `escape_last_ditch_phase_max_level` | 10 | level | 1–51 | `borg_escape.rs:818` |
| `escape_last_ditch_phase_hp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:819` |
| `escape_last_ditch_phase_fail` | 15 | percent | 0–100 | `borg_escape.rs:820` |
| `escape_endgame_stay_danger_tenths` | 25 | tenths | 0–100 | `borg_escape.rs:834` |
| `escape_endgame_stay_unique_count` | 3 | count | 1–9 | `borg_escape.rs:836` |
| `escape_endgame_stay_min_depth` | 97 | depth | 0–127 | `borg_escape.rs:837` |
| `escape_mage_danger_tenths` | 3 | tenths | 0–100 | `borg_escape.rs:846` |
| `escape_mage_max_sp` | 20 | sp | 0–10000 | `borg_escape.rs:848` |
| `escape_mage_min_level` | 45 | level | 1–51 | `borg_escape.rs:849` |
| `escape_teleport_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:850` |
| `escape_teleport_ordinary_danger_tenths` | 11 | tenths | 0–100 | `borg_escape.rs:851` |
| `escape_teleport_phase_tolerance` | 50 | count | 0–1000 | `borg_escape.rs:877` |
| `escape_teleport_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:877` |
| `escape_phase_antisummon_delay` | 50 | turns | 0–100000 | `borg_escape.rs:878` |
| `escape_phase_unique_danger_tenths` | 13 | tenths | 0–100 | `borg_escape.rs:895` |
| `escape_phase_unique_min_count` | 2 | count | 1–9 | `borg_escape.rs:895` |
| `escape_phase_ordinary_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:896` |
| `escape_afraid_warrior_danger_tenths` | 10 | tenths | 0–100 | `borg_escape.rs:897` |
| `escape_phase_tolerance` | 25 | count | 0–1000 | `borg_escape.rs:903` |
| `escape_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:903` |
| `escape_fallback_phase_tolerance` | 75 | count | 0–1000 | `borg_escape.rs:938` |
| `escape_fallback_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:938` |
| `escape_failed_teleport_flee_max_level` | 35 | level | 1–51 | `borg_escape.rs:968` |
| `escape_failed_teleport_leave_max_level` | 35 | level | 1–51 | `borg_escape.rs:980` |
| `escape_compromised_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:992` |
| `escape_compromised_max_level` | 35 | level | 1–51 | `borg_escape.rs:993` |
| `escape_compromised_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:993` |
| `escape_compromised_unique_danger_tenths` | 9 | tenths | 0–100 | `borg_escape.rs:994` |
| `escape_compromised_unique_max_level` | 35 | level | 1–51 | `borg_escape.rs:996` |
| `escape_compromised_unique_hp_divisor` | 3 | divisor | 1–1000 | `borg_escape.rs:996` |
| `escape_compromised_ordinary_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:997` |
| `escape_compromised_ordinary_max_level` | 20 | level | 1–51 | `borg_escape.rs:997` |
| `escape_compromised_general_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:998` |
| `escape_compromised_general_max_level` | 35 | level | 1–51 | `borg_escape.rs:998` |
| `escape_compromised_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1002` |
| `escape_compromised_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1002` |
| `escape_failed_compromised_flee_max_level` | 25 | level | 1–51 | `borg_escape.rs:1041` |
| `escape_weak_caster_max_level` | 35 | level | 1–51 | `borg_escape.rs:1061` |
| `escape_weak_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:1062` |
| `escape_weak_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1062` |
| `escape_low_level_max_level` | 10 | level | 1–51 | `borg_escape.rs:1080` |
| `escape_low_level_danger_tenths` | 5 | tenths | 0–100 | `borg_escape.rs:1081` |
| `escape_low_level_unique_danger_tenths` | 7 | tenths | 0–100 | `borg_escape.rs:1081` |
| `escape_low_level_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1085` |
| `escape_low_level_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1085` |
| `escape_low_caster_max_level` | 8 | level | 1–51 | `borg_escape.rs:1138` |
| `escape_low_caster_phase_tolerance` | 65 | count | 0–1000 | `borg_escape.rs:1139` |
| `escape_low_caster_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1139` |
| `escape_low_mana_danger_tenths` | 6 | tenths | 0–100 | `borg_escape.rs:1159` |
| `escape_low_mana_unique_danger_tenths` | 8 | tenths | 0–100 | `borg_escape.rs:1159` |
| `escape_low_mana_sp_divisor` | 10 | divisor | 1–1000 | `borg_escape.rs:1160` |
| `escape_low_mana_min_maxsp` | 100 | sp | 0–10000 | `borg_escape.rs:1160` |
| `escape_low_mana_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1164` |
| `escape_low_mana_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1164` |
| `escape_scoot_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1200` |
| `escape_scoot_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1200` |
| `escape_twitch_limit` | 50 | count | 0–1000 | `borg_escape.rs:1215` |
| `escape_twitch_phase_tolerance` | 20 | count | 0–1000 | `borg_escape.rs:1218` |
| `escape_twitch_phase_horizon` | 2 | turns | 1–10 | `borg_escape.rs:1218` |
| `danger_default_hit_power` | 0 | hit_power | 0–1000 | `borg_danger.rs:197` |
| `danger_rsf_lash_damage` | 100 | danger | 0–5000 | `borg_danger.rs:800` |
| `danger_rsf_spit_damage` | 100 | danger | 0–5000 | `borg_danger.rs:806` |
| `danger_pfe_melee_divisor` | 2 | divisor | 1–1000 | `borg_danger.rs:212` |


## P1.3: recovery (#221)

Recovery's source locations refer to **`30ab9d66`**, before extraction.
The controls cover its danger horizon and avoidance fraction; the injured
and critical-HP chance bonuses; each cheap/expensive cure's chance;
HP/SP gates; Unholy Reprieve's HP cutoff; rod-rest duration; and SP/depth,
food and danger gates for ordinary and munchkin rest.

Fractions retain their original multiply/divide order. Chance thresholds
stay in 0–100; SP tenths in 0–10; denominators/horizons and rod-rest counts
must be positive. The rod rest defaults to the same `R100<Enter>` keys.
The zero-danger requirement for cheap healing remains a categorical
no-danger guard. Status booleans, full-health/full-SP comparisons, rod
presence/charging state, goal and terrain enums, regional fear geometry,
RNG domain 100, tenths denominator 10, anti-bounce resets and the `R&`/`R*`
rest protocol remain representation or game/interface data. No rest order,
item preference, random roll or effect was changed.

| Name | Default | Unit | Range | Original location |
| --- | ---: | --- | --- | --- |
| `recover_danger_horizon` | 1 | turns | 1–10 | `borg_recover.rs:98` |
| `recover_danger_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:101` |
| `recover_hurt_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:111` |
| `recover_hurt_chance_bonus` | 10 | percent | 0–100 | `borg_recover.rs:112` |
| `recover_critical_hp_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:116` |
| `recover_critical_chance_bonus` | 10 | percent | 0–100 | `borg_recover.rs:117` |
| `recover_cheap_stun_chance` | 75 | percent | 0–100 | `borg_recover.rs:124` |
| `recover_cheap_cut_chance` | 75 | percent | 0–100 | `borg_recover.rs:160` |
| `recover_cheap_poison_chance` | 75 | percent | 0–100 | `borg_recover.rs:173` |
| `recover_cheap_fear_chance` | 75 | percent | 0–100 | `borg_recover.rs:186` |
| `recover_cheap_hunger_chance` | 75 | percent | 0–100 | `borg_recover.rs:199` |
| `recover_cheap_image_chance` | 75 | percent | 0–100 | `borg_recover.rs:207` |
| `recover_cheap_heal_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:214` |
| `recover_cheap_heal_chance` | 75 | percent | 0–100 | `borg_recover.rs:215` |
| `recover_cheap_heal_sp_divisor` | 4 | divisor | 1–1000 | `borg_recover.rs:217` |
| `recover_unholy_min_hp` | 90 | hp | 0–10000 | `borg_recover.rs:237` |
| `recover_costly_stun_chance` | 25 | percent | 0–100 | `borg_recover.rs:266` |
| `recover_costly_heavy_stun_chance` | 95 | percent | 0–100 | `borg_recover.rs:279` |
| `recover_costly_cut_chance` | 25 | percent | 0–100 | `borg_recover.rs:292` |
| `recover_critical_ccw_hp` | 10 | hp | 0–10000 | `borg_recover.rs:298` |
| `recover_costly_poison_chance` | 25 | percent | 0–100 | `borg_recover.rs:305` |
| `recover_costly_blind_chance` | 25 | percent | 0–100 | `borg_recover.rs:321` |
| `recover_costly_confusion_chance` | 25 | percent | 0–100 | `borg_recover.rs:336` |
| `recover_costly_fear_chance` | 25 | percent | 0–100 | `borg_recover.rs:350` |
| `recover_costly_hunger_chance` | 25 | percent | 0–100 | `borg_recover.rs:362` |
| `recover_costly_heal_hp_divisor` | 2 | divisor | 1–1000 | `borg_recover.rs:370` |
| `recover_costly_heal_chance` | 25 | percent | 0–100 | `borg_recover.rs:371` |
| `recover_rest_high_depth` | 85 | depth | 0–127 | `borg_recover.rs:418` |
| `recover_rest_high_sp_tenths` | 7 | tenths | 0–10 | `borg_recover.rs:418` |
| `recover_rest_sp_tenths` | 6 | tenths | 0–10 | `borg_recover.rs:418` |
| `recover_mana_rest_max_level` | 40 | level | 1–51 | `borg_recover.rs:461` |
| `recover_mana_rest_deep_depth` | 85 | depth | 0–127 | `borg_recover.rs:461` |
| `recover_mana_rest_sp_tenths` | 8 | tenths | 0–10 | `borg_recover.rs:462` |
| `recover_mana_rest_danger_divisor` | 10 | divisor | 1–1000 | `borg_recover.rs:463` |
| `recover_mana_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:469` |
| `recover_munchkin_rest_min_food` | 2 | count | 0–1000 | `borg_recover.rs:492` |
| `recover_rod_rest_turns` | 100 | turns | 1–9999 | `borg_recover.rs:404` |

## Named decision trace (P1.4)

The vendored core writes `trace-core.jsonl` and death dossiers retain its
last 200 rows. See [CORE-TRACE.md](CORE-TRACE.md) for the schema, sampling
point and opt-out, and [P1.4-VALIDATION.md](P1.4-VALIDATION.md) for default
parity and the recovery override's named divergence.

## Attack (P2.1, #240)

167 attack parameters bring the registry to 770 entries. Source lines below
refer to `borg_fight_attack.rs` at `5d9abb8f`. The existing policy and damage
model are retained; these are not corrections to that model. Reviewed the
combat manual (chapter 4) and Borg notes §§15.5/15.11 before extraction.

Retained data: class/attack/effect identifiers; keycodes/directions; array
sizes, loop/index increments and missing/invalid sentinels (-1, -2, -12,
-500, -999, -9999); zero/status guards; adjacency and projection geometry
(including bolt/beam/dispel radius sentinels, panel/fear-region sizes, angle
table bounds); percentage/decimal units and dice means; slay/brand and
resistance multipliers; copied spell/device/activation damage and range
formulas, including crush self-cost and special strike blow counts.
The item-destruction penalties, mana reserves/failure allowances, paranoia
chances and strategic scoring weights are configurable.

Engine authority: `src/player-attack.c` (melee/ranged damage and hit tests),
`src/project.c` (projection geometry), `lib/gamedata/class.txt`,
`object.txt`, `activation.txt` (spell/item effects). Copied estimates may
differ from these current mechanics; no claim of exact engine modelling is
made. In particular the legacy spell-damage dispatch remains frozen.
Bounds require positive divisors/horizons, restrict leap distance to the
spell's existing four-grid geometry, and keep the multiple-unique cutoff
below questor rank 10. Numeric bounds are exploratory input limits, not
recommended values.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `attack_melee_min_hit_tenths` | 8 | tenths | 380 |
| `attack_melee_bonus_min_level` | 15 | level | 388 |
| `attack_melee_hit_bonus` | 10 | percent | 389 |
| `attack_mage_melee_min_sp` | 1 | mana | 394 |
| `attack_mage_hit_penalty` | 10 | percent | 396 |
| `attack_melee_overkill_multiplier` | 2 | multiplier | 408 |
| `attack_mage_melee_max_level` | 40 | level | 416 |
| `attack_mage_melee_damage_tenths` | 8 | tenths | 419 |
| `attack_melee_unique_bonus` | 5 | multiplier | 430 |
| `attack_melee_town_unique_numerator` | 2 | multiplier | 438 |
| `attack_melee_town_unique_divisor` | 3 | divisor | 438 |
| `attack_town_unique_min_level` | 5 | level | 441 |
| `attack_melee_breeder_numerator` | 3 | multiplier | 448 |
| `attack_melee_breeder_divisor` | 2 | divisor | 448 |
| `attack_melee_summoner_numerator` | 3 | multiplier | 455 |
| `attack_melee_summoner_divisor` | 2 | divisor | 455 |
| `attack_melee_questor_bonus` | 5 | multiplier | 463 |
| `attack_melee_sleep_danger_horizon` | 1 | turns | 478 |
| `attack_melee_sleep_danger_multiplier` | 2 | multiplier | 480 |
| `attack_melee_danger_horizon` | 2 | turns | 491 |
| `attack_melee_partial_min_level` | 15 | level | 494 |
| `attack_melee_partial_danger_divisor` | 10 | divisor | 495 |
| `attack_status_low_level_cutoff` | 13 | level | 660 |
| `attack_status_low_level_limit` | 10 | level | 660 |
| `attack_status_level_offset` | 10 | level | 660 |
| `attack_status_level_divisor` | 4 | divisor | 660 |
| `attack_status_level_numerator` | 3 | multiplier | 660 |
| `attack_status_level_base` | 10 | level | 660 |
| `attack_status_danger_horizon` | 1 | turns | 691 |
| `attack_fear_bonus_max_level` | 10 | level | 694 |
| `attack_fear_danger_bonus` | 20 | danger | 695 |
| `attack_arrow_point_blank_divisor` | 5 | divisor | 766 |
| `attack_arrow_min_hit_tenths` | 8 | tenths | 769 |
| `attack_mana_unique_min_potions` | 3 | count | 777 |
| `attack_mana_unique_multiplier` | 2 | multiplier | 779 |
| `attack_nether_point_blank_divisor` | 5 | divisor | 887 |
| `attack_teleport_unique_injury` | 60 | percent | 995 |
| `attack_teleport_unique_danger_tenths` | 13 | tenths | 1001 |
| `attack_teleport_unique_max_depth` | 98 | depth | 1001 |
| `attack_teleport_unique_min_count` | 2 | count | 1004 |
| `attack_teleport_unique_max_count` | 8 | count | 1004 |
| `attack_teleport_mage_unique_distance` | 2 | grids | 1007 |
| `attack_teleport_vault_min_walls` | 2 | count | 1032 |
| `attack_teleport_glyph_value` | 300 | score | 1062 |
| `attack_teleport_morgoth_value` | 100 | score | 1070 |
| `attack_teleport_low_sp` | 35 | mana | 1076 |
| `attack_teleport_low_sp_value` | 150 | score | 1079 |
| `attack_turn_danger_horizon` | 1 | turns | 1090 |
| `attack_status_slow_speed_margin` | 5 | speed | 1098 |
| `attack_fear_danger_horizon` | 2 | turns | 1160 |
| `attack_fear_danger_multiplier` | 2 | multiplier | 1162 |
| `attack_crush_monster_level_margin` | 5 | level | 1175 |
| `attack_sleep_danger_horizon` | 1 | turns | 1198 |
| `attack_missile_danger_horizon` | 1 | turns | 1236 |
| `attack_missile_danger_tenths` | 2 | tenths | 1236 |
| `attack_missile_level_margin` | 5 | level | 1238 |
| `attack_missile_weak_max_level` | 20 | level | 1244 |
| `attack_ranged_overkill_multiplier` | 2 | multiplier | 1258 |
| `attack_ranged_unique_multiplier` | 3 | multiplier | 1265 |
| `attack_ranged_town_unique_numerator` | 2 | multiplier | 1273 |
| `attack_ranged_town_unique_divisor` | 3 | divisor | 1273 |
| `attack_ranged_breeder_numerator` | 3 | multiplier | 1283 |
| `attack_ranged_breeder_divisor` | 2 | divisor | 1283 |
| `attack_ranged_summoner_numerator` | 3 | multiplier | 1290 |
| `attack_ranged_summoner_divisor` | 2 | divisor | 1290 |
| `attack_ranged_questor_bonus` | 9 | multiplier | 1298 |
| `attack_bolt_max_target_age` | 2 | turns | 1347 |
| `attack_bolt_unknown_min_walls` | 2 | count | 1388 |
| `attack_bolt_unknown_min_grids` | 1 | count | 1388 |
| `attack_bolt_sleep_danger_horizon` | 1 | turns | 1415 |
| `attack_bolt_sleep_danger_divisor` | 2 | divisor | 1418 |
| `attack_bolt_danger_horizon` | 1 | turns | 1428 |
| `attack_bolt_kill_multiplier` | 2 | multiplier | 1432 |
| `attack_destroy_unknown_boots_value` | 20 | score | 1450 |
| `attack_destroy_unknown_ring_value` | 20 | score | 1456 |
| `attack_destroy_speed_ring_value` | 2000 | score | 1459 |
| `attack_destroy_valuable_potion_value` | 2000 | score | 1482 |
| `attack_destroy_unknown_potion_value` | 20 | score | 1485 |
| `attack_unmapped_fear_divisor` | 20 | divisor | 1531 |
| `attack_ego_ammo_multiplier` | 99 | multiplier | 2193 |
| `attack_throw_max_target_age` | 10 | turns | 2307 |
| `attack_throw_fast_monster_margin` | 5 | speed | 2317 |
| `attack_throw_min_item_value` | 100 | gold | 2408 |
| `attack_throw_low_damage` | 5 | damage | 2408 |
| `attack_reserve_mage_sp` | 6 | mana | 2503 |
| `attack_reserve_ranger_sp` | 22 | mana | 2504 |
| `attack_reserve_rogue_sp` | 20 | mana | 2505 |
| `attack_reserve_priest_sp` | 8 | mana | 2506 |
| `attack_reserve_paladin_sp` | 20 | mana | 2507 |
| `attack_reserve_necromancer_sp` | 10 | mana | 2508 |
| `attack_spell_paranoia_max_level` | 2 | level | 2534 |
| `attack_spell_paranoia_chance` | 1 | percent | 2535 |
| `attack_spell_unique_fail` | 40 | percent | 2546 |
| `attack_spell_fail` | 25 | percent | 2546 |
| `attack_primary_teleport_fail` | 15 | percent | 2562 |
| `attack_primary_max_level` | 30 | level | 2562 |
| `attack_spell_low_max_sp` | 50 | mana | 2575 |
| `attack_spell_reserve_divisor` | 2 | divisor | 2580 |
| `attack_spell_reserve_penalty` | 3 | multiplier | 2581 |
| `attack_spell_deep_reserve_divisor` | 3 | divisor | 2585 |
| `attack_spell_deep_reserve_penalty` | 5 | multiplier | 2586 |
| `attack_escape_reserve_min_max_sp` | 30 | mana | 2592 |
| `attack_escape_reserve_penalty` | 750 | multiplier | 2593 |
| `attack_reserve_spell_max_level` | 15 | level | 2631 |
| `attack_reserve_spell_fail` | 25 | percent | 2646 |
| `attack_reserve_spell_danger_horizon` | 1 | turns | 2651 |
| `attack_reserve_spell_danger_multiplier` | 2 | multiplier | 2651 |
| `attack_reserve_spell_near_distance` | 7 | grids | 2679 |
| `attack_reserve_spell_damage_margin` | 4 | damage | 2686 |
| `attack_dispel_paranoia_chance` | 2 | percent | 2757 |
| `attack_dispel_fail` | 25 | percent | 2762 |
| `attack_dispel_reserve_divisor` | 2 | divisor | 2775 |
| `attack_dispel_reserve_penalty` | 3 | multiplier | 2776 |
| `attack_dispel_deep_reserve_divisor` | 3 | divisor | 2780 |
| `attack_dispel_deep_reserve_penalty` | 5 | multiplier | 2781 |
| `attack_dispel_escape_min_max_sp` | 30 | mana | 2786 |
| `attack_dispel_escape_penalty` | 750 | multiplier | 2787 |
| `attack_wand_paranoia_chance` | 2 | percent | 2818 |
| `attack_wand_charge_penalty` | 50 | score | 2831 |
| `attack_rod_paranoia_chance` | 2 | percent | 2854 |
| `attack_rod_max_failure` | 500 | permille | 2859 |
| `attack_unknown_device_paranoia_chance` | 2 | percent | 2911 |
| `attack_unknown_wand_max_failure` | 500 | permille | 2929 |
| `attack_unknown_wand_penalty_min_level` | 5 | level | 2937 |
| `attack_unknown_wand_penalty` | 5 | score | 2938 |
| `attack_unknown_wand_danger_horizon` | 1 | turns | 2949 |
| `attack_unknown_wand_danger_tenths` | 7 | tenths | 2949 |
| `attack_unknown_wand_emergency_value` | 999 | score | 2951 |
| `attack_staff_paranoia_chance` | 5 | percent | 2994 |
| `attack_activation_paranoia_chance` | 2 | percent | 3088 |
| `attack_ring_paranoia_chance` | 2 | percent | 3132 |
| `attack_dragon_paranoia_chance` | 2 | percent | 3171 |
| `attack_leap_min_target_score` | 10 | score | 3284 |
| `attack_leap_max_distance` | 4 | grids | 3296 |
| `attack_vampire_max_distance` | 20 | grids | 3513 |
| `attack_vampire_sleep_danger_horizon` | 1 | turns | 3577 |
| `attack_vampire_sleep_danger_multiplier` | 2 | multiplier | 3579 |
| `attack_vampire_danger_horizon` | 2 | turns | 3585 |
| `attack_vampire_partial_min_level` | 15 | level | 3588 |
| `attack_vampire_partial_danger_divisor` | 10 | divisor | 3589 |
| `attack_flag_danger_horizon` | 4 | turns | 3618 |
| `attack_crush_hp_margin` | 10 | hp | 3636 |
| `attack_crush_hp_level_multiplier` | 4 | multiplier | 3636 |
| `attack_crush_post_danger_multiplier` | 2 | multiplier | 3648 |
| `attack_crush_min_post_hp` | 50 | hp | 3648 |
| `attack_crush_reserve_divisor` | 2 | divisor | 3658 |
| `attack_crush_reserve_penalty` | 10 | multiplier | 3659 |
| `attack_trance_reserve_divisor` | 2 | divisor | 3704 |
| `attack_trance_reserve_penalty` | 10 | multiplier | 3705 |
| `attack_holy_word_min_loss` | 300 | hp | 3917 |
| `attack_curse_min_hp` | 120 | hp | 4030 |
| `attack_unknown_rod_damage` | 75 | damage | 4085 |
| `attack_unknown_wand_damage` | 75 | damage | 4088 |
| `attack_holy_staff_hp_divisor` | 2 | divisor | 4164 |
| `attack_holy_staff_hurt_value` | 500 | score | 4164 |
| `attack_holy_staff_value` | 120 | score | 4164 |
| `attack_holy_activation_hp_divisor` | 2 | divisor | 4283 |
| `attack_holy_activation_hurt_value` | 500 | score | 4283 |
| `attack_holy_activation_value` | 120 | score | 4283 |
| `attack_max_target_age` | 2 | turns | 4444 |
| `attack_town_mage_max_level` | 10 | level | 4457 |
| `attack_fast_monster_distance` | 2 | grids | 4466 |
| `attack_scary_breeder_max_depth` | 5 | depth | 4476 |
| `attack_scary_max_level_time` | 2000 | turns | 4478 |
| `attack_scary_max_town_time` | 3000 | turns | 4478 |
| `attack_scary_depth_multiplier` | 4 | multiplier | 4487 |
| `attack_scary_min_level` | 10 | level | 4487 |

## Defence (P2.2) (#241)

218 named parameters. Source locations and validated input ranges
are also recorded in `params-core.json` for each run. Defaults retain the
existing policy; no tuning recommendation is implied.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `defend_bless_fail` | 25 | percent | 291 |
| `defend_bless_max_target_age` | 5 | turns | 332 |
| `defend_bless_no_rest_duration` | 11000 | duration | 357 |
| `defend_speed_fail_base` | 25 | percent | 393 |
| `defend_speed_fail_mid_adjustment` | 10 | percent | 393 |
| `defend_speed_fail_low_adjustment` | 10 | percent | 393 |
| `defend_grim_purpose_fail_base` | 25 | percent | 535 |
| `defend_grim_purpose_fail_mid_adjustment` | 10 | percent | 535 |
| `defend_grim_purpose_fail_low_adjustment` | 10 | percent | 535 |
| `defend_grim_purpose_improvement_divisor` | 7 | divisor | 554 |
| `defend_grim_purpose_score_bonus` | 2 | score | 557 |
| `defend_grim_purpose_no_rest_duration` | 13000 | duration | 565 |
| `defend_resist_fecap_improvement_divisor` | 7 | divisor | 630 |
| `defend_resist_fecap_score_bonus` | 2 | score | 633 |
| `defend_resist_fecap_no_rest_duration` | 21000 | duration | 643 |
| `defend_resist_f_fail_base` | 25 | percent | 668 |
| `defend_resist_f_fail_mid_adjustment` | 10 | percent | 668 |
| `defend_resist_f_fail_low_adjustment` | 10 | percent | 668 |
| `defend_resist_f_improvement_divisor` | 7 | divisor | 698 |
| `defend_resist_f_no_rest_duration` | 21000 | duration | 721 |
| `defend_resist_c_fail_base` | 25 | percent | 744 |
| `defend_resist_c_fail_mid_adjustment` | 10 | percent | 744 |
| `defend_resist_c_fail_low_adjustment` | 10 | percent | 744 |
| `defend_resist_c_improvement_divisor` | 7 | divisor | 775 |
| `defend_resist_c_no_rest_duration` | 21000 | duration | 798 |
| `defend_resist_a_fail_base` | 25 | percent | 821 |
| `defend_resist_a_fail_mid_adjustment` | 10 | percent | 821 |
| `defend_resist_a_fail_low_adjustment` | 10 | percent | 821 |
| `defend_resist_a_improvement_divisor` | 7 | divisor | 844 |
| `defend_resist_a_no_rest_duration` | 21000 | duration | 871 |
| `defend_resist_e_fail_base` | 25 | percent | 893 |
| `defend_resist_e_fail_mid_adjustment` | 10 | percent | 893 |
| `defend_resist_e_fail_low_adjustment` | 10 | percent | 893 |
| `defend_resist_e_improvement_divisor` | 7 | divisor | 917 |
| `defend_resist_e_no_rest_duration` | 21000 | duration | 945 |
| `defend_resist_p_fail_base` | 25 | percent | 967 |
| `defend_resist_p_fail_mid_adjustment` | 10 | percent | 967 |
| `defend_resist_p_fail_low_adjustment` | 10 | percent | 967 |
| `defend_resist_p_improvement_divisor` | 7 | divisor | 990 |
| `defend_resist_p_no_rest_duration` | 21000 | duration | 1006 |
| `defend_prot_evil_fail_base` | 25 | percent | 1032 |
| `defend_prot_evil_fail_mid_adjustment` | 5 | percent | 1032 |
| `defend_prot_evil_fail_low_adjustment` | 10 | percent | 1032 |
| `defend_prot_evil_improvement_divisor` | 7 | divisor | 1069 |
| `defend_shield_improvement_divisor` | 7 | divisor | 1126 |
| `defend_shield_no_rest_duration` | 2000 | duration | 1139 |
| `defend_tele_away_fail` | 50 | percent | 1152 |
| `defend_tele_away_max_target_age` | 2 | turns | 1235 |
| `defend_hero_fail` | 15 | percent | 1336 |
| `defend_hero_no_rest_duration` | 10000 | duration | 1374 |
| `defend_regen_fail` | 15 | percent | 1384 |
| `defend_regen_no_rest_duration` | 10000 | duration | 1416 |
| `defend_berserk_fail` | 15 | percent | 1427 |
| `defend_near_evil_max_target_age` | 2 | turns | 1488 |
| `defend_smite_evil_fail` | 15 | percent | 1516 |
| `defend_glyph_fail_base` | 25 | percent | 1587 |
| `defend_glyph_fail_mid_adjustment` | 5 | percent | 1587 |
| `defend_glyph_fail_low_adjustment` | 20 | percent | 1587 |
| `defend_glyph_improvement_divisor` | 7 | divisor | 1622 |
| `defend_create_door_fail_base` | 30 | percent | 1672 |
| `defend_create_door_fail_mid_adjustment` | 5 | percent | 1672 |
| `defend_create_door_fail_low_adjustment` | 20 | percent | 1672 |
| `defend_create_door_improvement_divisor` | 7 | divisor | 1725 |
| `defend_genocide_fail_base` | 25 | percent | 1901 |
| `defend_genocide_fail_mid_adjustment` | 10 | percent | 1901 |
| `defend_genocide_fail_low_adjustment` | 10 | percent | 1901 |
| `defend_earthquake_improvement_divisor` | 5 | divisor | 2359 |
| `defend_banishment_fail` | 15 | percent | 2557 |
| `defend_inviso_fail` | 25 | percent | 2723 |
| `defend_inviso_no_rest_duration` | 18000 | duration | 2789 |
| `defend_inviso_potion_no_rest_duration` | 16000 | duration | 2795 |
| `defend_tele_away_morgoth_fail` | 40 | percent | 3097 |
| `defend_tele_away_morgoth_max_target_age` | 2 | turns | 3151 |
| `defend_banishment_morgoth_fail` | 50 | percent | 3247 |
| `defend_banishment_morgoth_max_target_age` | 2 | turns | 3291 |
| `defend_light_morgoth_fail` | 50 | percent | 3368 |
| `defend_light_morgoth_max_target_age` | 2 | turns | 3418 |
| `defend_scaled_fail_high_adjustment` | 19 | percent | 170 |
| `defend_scaled_fail_mid_numerator` | 2 | multiplier | 171 |
| `defend_scaled_fail_mid_divisor` | 3 | divisor | 171 |
| `defend_scaled_fail_low_divisor` | 3 | divisor | 173 |
| `defend_limit_unique_numerator` | 2 | multiplier | 183 |
| `defend_limit_unique_divisor` | 3 | divisor | 183 |
| `defend_limit_divisor` | 2 | divisor | 185 |
| `defend_some_danger_divisor` | 10 | divisor | 199 |
| `defend_some_danger_unique_tenths` | 7 | tenths | 199 |
| `defend_some_danger_tenths` | 5 | tenths | 199 |
| `defend_danger_horizon` | 1 | turns | 222 |
| `defend_bless_near_distance` | 3 | grids | 337 |
| `defend_bless_danger_divisor` | 12 | divisor | 347 |
| `defend_bless_max_level` | 15 | level | 347 |
| `defend_bless_max_danger_divisor` | 2 | divisor | 347 |
| `defend_speed_morgoth_tenths` | 3 | tenths | 443 |
| `defend_speed_breeder_tenths` | 7 | tenths | 448 |
| `defend_speed_questor_tenths` | 7 | tenths | 452 |
| `defend_speed_sauron_tenths` | 6 | tenths | 456 |
| `defend_speed_unique_tenths` | 6 | tenths | 468 |
| `defend_speed_morgoth_fight_tenths` | 5 | tenths | 473 |
| `defend_speed_no_unique_min_depth` | 97 | depth | 477 |
| `defend_speed_good_unique_numerator` | 2 | multiplier | 485 |
| `defend_speed_good_unique_divisor` | 3 | divisor | 485 |
| `defend_speed_good_divisor` | 2 | divisor | 485 |
| `defend_speed_good_min_danger_divisor` | 5 | divisor | 485 |
| `defend_speed_unique_numerator` | 2 | multiplier | 486 |
| `defend_speed_unique_divisor` | 3 | divisor | 486 |
| `defend_speed_divisor` | 3 | divisor | 486 |
| `defend_speed_min_danger_divisor` | 7 | divisor | 486 |
| `defend_speed_no_rest_per_level` | 1000 | turns | 496 |
| `defend_prayer_unique_tenths` | 8 | tenths | 619 |
| `defend_prayer_bonus_min_level` | 45 | level | 624 |
| `defend_prayer_level_tenths` | 8 | tenths | 625 |
| `defend_resist_fire_unique_tenths` | 8 | tenths | 693 |
| `defend_resist_cold_unique_tenths` | 8 | tenths | 770 |
| `defend_resist_all_no_rest_per_level` | 1000 | turns | 1085 |
| `defend_protection_evil_unique_tenths` | 7 | tenths | 1121 |
| `defend_shield_saving_danger_tenths` | 7 | tenths | 1163 |
| `defend_shield_saving_min_sp` | 30 | mana | 1163 |
| `defend_shield_low_danger_tenths` | 5 | tenths | 1166 |
| `defend_shield_min_danger_tenths` | 4 | tenths | 1171 |
| `defend_shield_high_danger_multiplier` | 3 | multiplier | 1176 |
| `defend_shield_high_fail_adjustment` | 10 | percent | 1177 |
| `defend_shield_mid_danger_multiplier` | 2 | multiplier | 1178 |
| `defend_shield_mid_fail_adjustment` | 5 | percent | 1180 |
| `defend_shield_low_danger_numerator` | 5 | multiplier | 1181 |
| `defend_shield_low_danger_divisor` | 2 | divisor | 1181 |
| `defend_shield_low_fail_adjustment` | 5 | percent | 1183 |
| `defend_shield_remnant_danger_divisor` | 2 | divisor | 1291 |
| `defend_berserk_min_max_hp` | 100 | hp | 1397 |
| `defend_hero_value` | 5 | score | 1453 |
| `defend_regeneration_near_distance` | 3 | grids | 1501 |
| `defend_regeneration_value` | 5 | score | 1542 |
| `defend_create_doors_max_bad_grids` | 6 | count | 1713 |
| `defend_mass_banish_fail` | 40 | percent | 1760 |
| `defend_mass_banish_min_danger_tenths` | 12 | tenths | 1769 |
| `defend_mass_banish_distance` | 20 | grids | 1789 |
| `defend_mass_banish_self_harm_tenths` | 12 | tenths | 1815 |
| `defend_mass_banish_morgoth_min_count` | 8 | count | 1823 |
| `defend_mass_banish_morgoth_tenths` | 6 | tenths | 1824 |
| `defend_banish_min_hp` | 75 | hp | 1933 |
| `defend_banish_threat_min_hp` | 375 | hp | 2045 |
| `defend_banish_threat_hp_multiplier` | 3 | multiplier | 2050 |
| `defend_banish_threat_self_harm_tenths` | 12 | tenths | 2056 |
| `defend_banish_threat_min_count` | 75 | count | 2062 |
| `defend_banish_threat_danger_divisor` | 5 | divisor | 2067 |
| `defend_banish_danger_self_harm_tenths` | 12 | tenths | 2094 |
| `defend_banish_min_danger_tenths` | 7 | tenths | 2102 |
| `defend_banish_deep_depth` | 75 | depth | 2102 |
| `defend_banish_deep_danger_tenths` | 6 | tenths | 2102 |
| `defend_banish_remnant_divisor` | 2 | divisor | 2107 |
| `defend_banish_nasties_min_hp_tenths` | 7 | tenths | 2210 |
| `defend_banish_nasties_min_hp` | 250 | hp | 2210 |
| `defend_banish_nasties_min_depth` | 25 | depth | 2215 |
| `defend_banish_nasties_danger_divisor` | 4 | divisor | 2225 |
| `defend_banish_nasties_fail` | 35 | percent | 2229 |
| `defend_banish_nasties_value` | 10 | score | 2250 |
| `defend_quake_fail` | 35 | percent | 2323 |
| `defend_quake_min_danger_tenths` | 6 | tenths | 2332 |
| `defend_quake_min_monster_distance` | 2 | grids | 2343 |
| `defend_quake_high_threat_count` | 4 | count | 2352 |
| `defend_quake_high_danger_tenths` | 7 | tenths | 2352 |
| `defend_quake_high_reduction_divisor` | 3 | divisor | 2353 |
| `defend_quake_mid_threat_count` | 3 | count | 2355 |
| `defend_quake_mid_danger_tenths` | 7 | tenths | 2355 |
| `defend_quake_mid_reduction_tenths` | 6 | tenths | 2356 |
| `defend_destruction_value` | 500 | score | 2393 |
| `defend_destruction_min_danger_tenths` | 8 | tenths | 2406 |
| `defend_destruction_low_hp_min_depth` | 90 | depth | 2406 |
| `defend_destruction_deep_low_hp` | 300 | hp | 2406 |
| `defend_destruction_unique_max_count` | 4 | count | 2427 |
| `defend_destruction_teleport_min_hp` | 275 | hp | 2428 |
| `defend_destruction_teleport_chance` | 75 | percent | 2429 |
| `defend_destruction_teleport_horizon` | 2 | turns | 2429 |
| `defend_destruction_teleport_min_escapes` | 2 | count | 2435 |
| `defend_destruction_fail` | 55 | percent | 2440 |
| `defend_destruction_emergency_danger_multiplier` | 4 | multiplier | 2448 |
| `defend_destruction_emergency_hp` | 150 | hp | 2448 |
| `defend_destruction_few_unique_count` | 2 | count | 2465 |
| `defend_destruction_few_unique_danger_multiplier` | 2 | multiplier | 2465 |
| `defend_teleport_level_danger_multiplier` | 2 | multiplier | 2496 |
| `defend_teleport_level_safe_chance` | 65 | percent | 2513 |
| `defend_teleport_level_safe_horizon` | 2 | turns | 2513 |
| `defend_teleport_level_min_escapes` | 2 | count | 2519 |
| `defend_teleport_level_fail` | 55 | percent | 2524 |
| `defend_banish_evil_min_danger_divisor` | 10 | divisor | 2563 |
| `defend_banish_evil_high_danger_multiplier` | 4 | multiplier | 2568 |
| `defend_banish_evil_fail_adjustment` | 10 | percent | 2569 |
| `defend_banish_evil_artifact_min_hp` | 100 | hp | 2577 |
| `defend_banish_evil_unique_injury` | 60 | percent | 2649 |
| `defend_banish_evil_sauron_hp` | 250 | hp | 2703 |
| `defend_banish_evil_morgoth_hp` | 350 | hp | 2706 |
| `defend_see_invis_max_age` | 45 | turns | 2735 |
| `defend_see_invis_max_danger_multiplier` | 2 | multiplier | 2740 |
| `defend_see_invis_value` | 10 | score | 2767 |
| `defend_see_invis_short_duration` | 3000 | duration | 2784 |
| `defend_see_invis_spell_duration` | 18000 | duration | 2788 |
| `defend_see_invis_potion_duration` | 30000 | duration | 2794 |
| `defend_light_beam_max_invis_age` | 48 | turns | 2817 |
| `defend_light_beam_danger_numerator` | 3 | multiplier | 2870 |
| `defend_light_beam_danger_divisor` | 4 | divisor | 2870 |
| `defend_light_beam_value` | 10 | score | 2881 |
| `defend_panel_shift_min_depth` | 70 | depth | 2917 |
| `defend_panel_shift_edge_distance` | 5 | grids | 2940 |
| `defend_panel_shift_max_age` | 10 | turns | 2977 |
| `defend_panel_shift_morgoth_max_age` | 10 | turns | 2978 |
| `defend_panel_shift_antisummon_min_age` | 50 | turns | 3020 |
| `defend_panel_shift_teleport_fail` | 30 | percent | 3038 |
| `defend_panel_shift_value` | 200 | score | 3082 |
| `defend_teleport_morgoth_min_glyphs` | 10 | count | 3121 |
| `defend_teleport_morgoth_value_cap` | 1000 | score | 3199 |
| `defend_teleport_distance` | 7 | grids | 3313 |
| `defend_teleport_min_count` | 7 | count | 3322 |
| `defend_teleport_simulated_value` | 1500 | score | 3328 |
| `defend_teleport_value` | 1000 | score | 3358 |
| `defend_smite_max_distance` | 5 | grids | 3430 |
| `defend_smite_simulated_value` | 500 | score | 3452 |
| `defend_smite_value` | 200 | score | 3469 |
| `defend_refresh_resistance_horizon` | 2 | turns | 3532 |
| `defend_refresh_resistance_duration` | 25000 | duration | 3553 |

Defence source lines refer to `5d9abb8f`; total 988 parameters. Retained
data: BD/class/effect identifiers, status and zero guards, unit success
scores (1), invalid score 9999, direction/keycodes, adjacency and panel
geometry, fear-region width 11, percentage units, questor rank 10 and
Sauron/Morgoth floors 99/100, banishment's self-damage model (3 for mass,
4 for selected race), single-monster danger's one-turn primitive.
Timers written to i16 have inclusive range 0..32767 and narrow only after
input validation; divisors and horizons are positive; unique-count
cutoffs stay below questor rank 10. Chapter 7 and Borg notes on defence,
healing and escapes distinguish mechanics from policy. Engine authority:
`src/effect-handler-general.c:effect_handler_BANISH/MASS_BANISH`
(self damage), `src/effect-handler-attack.c:effect_handler_DESTRUCTION`,
and `lib/gamedata/class.txt`/`activation.txt` (effects and durations).
Legacy predicted effects/durations and redundant branch order are retained,
not asserted to be accurate engine models.

## Item use (P2.3) (#242)

19 named parameters. Source locations and validated input ranges
are also recorded in `params-core.json` for each run. Defaults retain the
existing policy; no tuning recommendation is implied.

| Name | Default | Unit | Source line |
| --- | ---: | --- | ---: |
| `item_use_ccw_recent_window` | 4 | turns | 148 |
| `item_use_ccw_recent_skip_chance` | 75 | percent | 148 |
| `item_use_ccw_min_stock` | 2 | count | 153 |
| `item_use_food_heal_critical_hp` | 4 | hp | 423 |
| `item_use_rod_equip_max_failure` | 500 | permille | 461 |
| `item_use_rod_use_max_failure` | 500 | permille | 494 |
| `item_use_staff_use_max_failure` | 500 | permille | 586 |
| `item_use_staff_equip_max_failure` | 500 | permille | 641 |
| `item_use_staff_teleport_max_failure` | 650 | permille | 650 |
| `item_use_ring_equip_max_failure` | 500 | permille | 748 |
| `item_use_dragon_equip_max_failure` | 500 | permille | 867 |
| `item_use_unholy_min_hp` | 90 | hp | 1010 |
| `item_use_recharge_wand_max_charges` | 1 | count | 1179 |
| `item_use_recharge_staff_max_charges` | 1 | count | 1196 |
| `item_use_recharge_stacked_min_charges` | 1 | count | 1208 |
| `item_use_recharge_low_charges` | 2 | count | 1188 |
| `item_use_recharge_fail` | 96 | percent | 1189 |
| `item_use_teleport_staff_min_charges` | 3 | count | 1201 |
| `item_use_recharge_min_total_stock` | 4 | count | 1208 |

Item-use source lines refer to `5d9abb8f`; total 1007 entries. Retained
mechanics/interface data: 75% confusion skill modifier, the copied device
failure formula (including its documented zero-divisor case), percentage
and permille scales, EF_NOURISH subtype identifiers 0/2/3, inventory slot
offsets, keys/effect IDs, zero charge/readiness and status guards. Recharge
stock thresholds of one are policy and have names. Rod recall bypass,
activation ordering and unknown-item scans retain their existing behavior.
Read manual chapter7 and its device/recharge sections; engine authority is
`src/player-util.c:get_use_device_chance`, `src/effect-handler-general.c`
(recharge/nourishment), and `lib/gamedata/object.txt`/`activation.txt`. The
legacy failure estimate is retained, not presented as an exact modern
formula or silently corrected.

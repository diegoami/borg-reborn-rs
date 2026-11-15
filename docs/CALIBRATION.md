# Calibrating borg-rs's core (track P)

P1.1 (#219) gives `borg-core/src/borg_caution.rs` 200 parameters. This
changes calibration access, not the default policy. No value here is claimed
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

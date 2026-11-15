//! borg-reborn-rs (#219): calibration of the vendored core.
//!
//! Source locations: caution at 726e04fa; danger/escape/recovery at 30ab9d66 (before hooks).
//! Defaults retain the original integer operations and RNG calls.
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

// One row generates both the lookup index and its metadata: additions cannot
// accidentally shift the values away from their names.
macro_rules! parameters {
    ($($name:ident => ($default:expr, $unit:expr, $min:expr, $max:expr, $source:expr, $description:expr)),* $(,)?) => {
        #[derive(Clone, Copy)]
        #[repr(usize)]
        pub(crate) enum Param { $($name),* }
        pub(crate) const SPECS: &[Spec] = &[
            $(Spec { name: stringify!($name), default: $default, unit: $unit,
                min: $min, max: $max, source: $source, description: $description }),*
        ];
    };
}

#[derive(Serialize)]
pub(crate) struct Spec {
    pub name: &'static str,
    pub default: i32,
    pub unit: &'static str,
    pub min: i32,
    pub max: i32,
    pub source: &'static str,
    pub description: &'static str,
}

parameters! {
    caution_bounce_healthy_tenths => (7, "tenths", 0, 100, "borg_caution.rs:102", "Controls bounce healthy tenths."),
    caution_bounce_limit => (300, "turns", 0, 100000, "borg_caution.rs:108", "Controls bounce limit."),
    caution_caster_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:157", "Controls caster stat vote."),
    caution_priest_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:160", "Controls priest stat vote."),
    caution_druid_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:163", "Controls druid stat vote."),
    caution_necromancer_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:166", "Controls necromancer stat vote."),
    caution_warrior_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:169", "Controls warrior stat vote."),
    caution_stat_fix_hp_high => (850, "hp", 0, 10000, "borg_caution.rs:171", "Controls stat fix hp high."),
    caution_stat_fix_vote_high => (1, "votes", 0, 100, "borg_caution.rs:172", "Controls stat fix vote high."),
    caution_stat_fix_hp_low => (700, "hp", 0, 10000, "borg_caution.rs:174", "Controls stat fix hp low."),
    caution_stat_fix_vote_low => (3, "votes", 0, 100, "borg_caution.rs:175", "Controls stat fix vote low."),
    caution_priest_stat_fix_sp => (100, "sp", 0, 10000, "borg_caution.rs:177", "Controls priest stat fix sp."),
    caution_priest_stat_fix_votes => (5, "votes", 0, 100, "borg_caution.rs:178", "Controls priest stat fix votes."),
    caution_mage_stat_fix_sp => (100, "sp", 0, 10000, "borg_caution.rs:180", "Controls mage stat fix sp."),
    caution_mage_stat_fix_votes => (5, "votes", 0, 100, "borg_caution.rs:181", "Controls mage stat fix votes."),
    caution_heal_spell_fail => (15, "percent", 0, 100, "borg_caution.rs:196", "Controls heal spell fail."),
    caution_clw_estimate_percent => (15, "percent", 0, 100, "borg_caution.rs:201", "Controls clw estimate percent."),
    caution_clw_estimate_min => (15, "hp", 0, 10000, "borg_caution.rs:201", "Controls clw estimate min."),
    caution_csw_estimate_percent => (20, "percent", 0, 100, "borg_caution.rs:202", "Controls csw estimate percent."),
    caution_csw_estimate_min => (25, "hp", 0, 10000, "borg_caution.rs:202", "Controls csw estimate min."),
    caution_ccw_estimate_percent => (25, "percent", 0, 100, "borg_caution.rs:203", "Controls ccw estimate percent."),
    caution_ccw_estimate_min => (30, "hp", 0, 10000, "borg_caution.rs:203", "Controls ccw estimate min."),
    caution_heal_estimate_percent => (35, "percent", 0, 100, "borg_caution.rs:205", "Controls heal estimate percent."),
    caution_heal_estimate_min => (300, "hp", 0, 10000, "borg_caution.rs:205", "Controls heal estimate min."),
    caution_healing_rod_fail => (500, "permille", 0, 1000, "borg_caution.rs:211", "Controls healing rod fail."),
    caution_confused_heal_loss => (80, "percent", 0, 100, "borg_caution.rs:225", "Controls confused heal loss."),
    caution_confused_star_loss => (85, "percent", 0, 100, "borg_caution.rs:232", "Controls confused star loss."),
    caution_confused_star_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:233", "Controls confused star danger multiplier."),
    caution_confused_staff_fail => (650, "permille", 0, 1000, "borg_caution.rs:259", "Controls confused staff fail."),
    caution_confused_cure_danger_tenths => (15, "tenths", 0, 100, "borg_caution.rs:260", "Controls confused cure danger tenths."),
    caution_confused_escape_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:270", "Controls confused escape danger multiplier."),
    caution_blind_cure_chance => (85, "percent", 0, 100, "borg_caution.rs:287", "Controls blind cure chance."),
    caution_blind_escape_danger_tenths => (25, "tenths", 0, 100, "borg_caution.rs:291", "Controls blind escape danger tenths."),
    caution_blind_heal_loss => (300, "hp", 0, 10000, "borg_caution.rs:296", "Controls blind heal loss."),
    caution_blind_warrior_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:301", "Controls blind warrior hp divisor."),
    caution_blind_confused_star_loss => (400, "hp", 0, 10000, "borg_caution.rs:318", "Controls blind confused star loss."),
    caution_blind_confused_danger_multiplier => (5, "multiplier", 0, 20, "borg_caution.rs:318", "Controls blind confused danger multiplier."),
    caution_blind_confused_min_loss => (100, "hp", 0, 10000, "borg_caution.rs:318", "Controls blind confused min loss."),
    caution_questor_heal_hp => (700, "hp", 0, 10000, "borg_caution.rs:327", "Controls questor heal hp."),
    caution_questor_holy_word_hp => (250, "hp", 0, 10000, "borg_caution.rs:328", "Controls questor holy word hp."),
    caution_questor_holy_word_fail => (14, "percent", 0, 100, "borg_caution.rs:328", "Controls questor holy word fail."),
    caution_questor_life_stat_votes => (5, "votes", 0, 100, "borg_caution.rs:330", "Controls questor life stat votes."),
    caution_questor_life_loss => (500, "hp", 0, 10000, "borg_caution.rs:332", "Controls questor life loss."),
    caution_questor_emergency_holy_hp => (250, "hp", 0, 10000, "borg_caution.rs:344", "Controls questor emergency holy hp."),
    caution_questor_emergency_holy_fail => (5, "percent", 0, 100, "borg_caution.rs:344", "Controls questor emergency holy fail."),
    caution_questor_safe_holy_hp => (550, "hp", 0, 10000, "borg_caution.rs:345", "Controls questor safe holy hp."),
    caution_questor_safe_holy_fail => (15, "percent", 0, 100, "borg_caution.rs:345", "Controls questor safe holy fail."),
    caution_questor_healing_fail => (15, "percent", 0, 100, "borg_caution.rs:346", "Controls questor healing fail."),
    caution_magi_sp_divisor => (5, "divisor", 1, 100, "borg_caution.rs:356", "Controls magi sp divisor."),
    caution_magi_chance => (50, "percent", 0, 100, "borg_caution.rs:357", "Controls magi chance."),
    caution_restore_mana_sp_divisor => (10, "divisor", 1, 100, "borg_caution.rs:366", "Controls restore mana sp divisor."),
    caution_restore_mana_sp => (70, "sp", 0, 10000, "borg_caution.rs:367", "Controls restore mana sp."),
    caution_restore_mana_maxsp => (200, "sp", 0, 10000, "borg_caution.rs:367", "Controls restore mana maxsp."),
    caution_restore_mana_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:370", "Controls restore mana danger multiplier."),
    caution_restore_stat_votes => (5, "votes", 0, 100, "borg_caution.rs:393", "Controls restore stat votes."),
    caution_restore_stat_hp => (650, "hp", 0, 10000, "borg_caution.rs:395", "Controls restore stat hp."),
    caution_heal_unique_chance_bonus => (10, "percent", 0, 100, "borg_caution.rs:414", "Controls heal unique chance bonus."),
    caution_heal_lethal_chance_bonus => (75, "percent", 0, 100, "borg_caution.rs:419", "Controls heal lethal chance bonus."),
    caution_heal_nonpriest_chance_penalty => (25, "percent", 0, 100, "borg_caution.rs:421", "Controls heal nonpriest chance penalty."),
    caution_heal_risky_chance_penalty => (5, "percent", 0, 100, "borg_caution.rs:426", "Controls heal risky chance penalty."),
    caution_heal_band1_max_loss => (15, "percent", 0, 100, "borg_caution.rs:429", "Controls heal band1 max loss."),
    caution_heal_band1_skip_chance => (98, "percent", 0, 100, "borg_caution.rs:429", "Controls heal band1 skip chance."),
    caution_heal_band2_min_loss => (16, "percent", 0, 100, "borg_caution.rs:430", "Controls heal band2 min loss."),
    caution_heal_band2_max_loss => (25, "percent", 0, 100, "borg_caution.rs:430", "Controls heal band2 max loss."),
    caution_heal_band2_skip_chance => (95, "percent", 0, 100, "borg_caution.rs:430", "Controls heal band2 skip chance."),
    caution_heal_band3_min_loss => (26, "percent", 0, 100, "borg_caution.rs:431", "Controls heal band3 min loss."),
    caution_heal_band3_max_loss => (50, "percent", 0, 100, "borg_caution.rs:431", "Controls heal band3 max loss."),
    caution_heal_band3_skip_chance => (80, "percent", 0, 100, "borg_caution.rs:431", "Controls heal band3 skip chance."),
    caution_heal_band4_min_loss => (51, "percent", 0, 100, "borg_caution.rs:432", "Controls heal band4 min loss."),
    caution_heal_band4_max_loss => (65, "percent", 0, 100, "borg_caution.rs:432", "Controls heal band4 max loss."),
    caution_heal_band4_skip_chance => (50, "percent", 0, 100, "borg_caution.rs:432", "Controls heal band4 skip chance."),
    caution_heal_band5_min_loss => (66, "percent", 0, 100, "borg_caution.rs:433", "Controls heal band5 min loss."),
    caution_heal_band5_max_loss => (74, "percent", 0, 100, "borg_caution.rs:433", "Controls heal band5 max loss."),
    caution_heal_band5_skip_chance => (25, "percent", 0, 100, "borg_caution.rs:433", "Controls heal band5 skip chance."),
    caution_heal_band6_min_loss => (75, "percent", 0, 100, "borg_caution.rs:434", "Controls heal band6 min loss."),
    caution_heal_band6_skip_chance => (1, "percent", 0, 100, "borg_caution.rs:434", "Controls heal band6 skip chance."),
    caution_clw_min_loss => (30, "percent", 0, 100, "borg_caution.rs:444", "Controls clw min loss."),
    caution_clw_max_loss => (40, "percent", 0, 100, "borg_caution.rs:445", "Controls clw max loss."),
    caution_clw_max_level => (10, "level", 1, 51, "borg_caution.rs:445", "Controls clw max level."),
    caution_clw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:447", "Controls clw danger divisor."),
    caution_csw_min_loss => (40, "percent", 0, 100, "borg_caution.rs:456", "Controls csw min loss."),
    caution_csw_max_loss => (50, "percent", 0, 100, "borg_caution.rs:457", "Controls csw max loss."),
    caution_csw_max_level => (20, "level", 1, 51, "borg_caution.rs:457", "Controls csw max level."),
    caution_csw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:459", "Controls csw danger divisor."),
    caution_ccw_min_loss => (50, "percent", 0, 100, "borg_caution.rs:468", "Controls ccw min loss."),
    caution_ccw_max_loss => (55, "percent", 0, 100, "borg_caution.rs:468", "Controls ccw max loss."),
    caution_ccw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:470", "Controls ccw danger divisor."),
    caution_emergency_ccw_hp => (50, "hp", 0, 10000, "borg_caution.rs:480", "Controls emergency ccw hp."),
    caution_deep_ccw_depth => (80, "depth", 0, 127, "borg_caution.rs:490", "Controls deep ccw depth."),
    caution_deep_ccw_danger => (50, "danger", 0, 10000, "borg_caution.rs:491", "Controls deep ccw danger."),
    caution_deep_ccw_loss => (20, "percent", 0, 100, "borg_caution.rs:492", "Controls deep ccw loss."),
    caution_heal_device_min_loss => (55, "percent", 0, 100, "borg_caution.rs:500", "Controls heal device min loss."),
    caution_conserve_heal_depth => (98, "depth", 0, 127, "borg_caution.rs:520", "Controls conserve heal depth."),
    caution_heal_potion_min_loss => (50, "percent", 0, 100, "borg_caution.rs:530", "Controls heal potion min loss."),
    caution_heal_all_min_loss => (60, "percent", 0, 100, "borg_caution.rs:544", "Controls heal all min loss."),
    caution_heal_unique_reserve_min_loss => (65, "percent", 0, 100, "borg_caution.rs:563", "Controls heal unique reserve min loss."),
    caution_heal_last_resort_loss => (75, "percent", 0, 100, "borg_caution.rs:589", "Controls heal last resort loss."),
    caution_cure_safe_danger_tenths => (2, "tenths", 0, 100, "borg_caution.rs:603", "Controls cure safe danger tenths."),
    caution_poison_cure_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:609", "Controls poison cure hp divisor."),
    caution_poison_cure_fail => (60, "percent", 0, 100, "borg_caution.rs:610", "Controls poison cure fail."),
    caution_poison_herbal_fail => (60, "percent", 0, 100, "borg_caution.rs:611", "Controls poison herbal fail."),
    caution_poison_healing_fail => (60, "percent", 0, 100, "borg_caution.rs:626", "Controls poison healing fail."),
    caution_poison_holy_fail => (60, "percent", 0, 100, "borg_caution.rs:627", "Controls poison holy fail."),
    caution_poison_critical_hp => (2, "hp", 0, 10000, "borg_caution.rs:646", "Controls poison critical hp."),
    caution_poison_critical_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:646", "Controls poison critical hp divisor."),
    caution_poison_reserve_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:684", "Controls poison reserve hp divisor."),
    caution_cut_cure_hp_divisor => (3, "divisor", 1, 100, "borg_caution.rs:702", "Controls cut cure hp divisor."),
    caution_cut_cure_chance => (20, "percent", 0, 100, "borg_caution.rs:702", "Controls cut cure chance."),
    caution_cut_critical_ccw_hp => (10, "hp", 0, 10000, "borg_caution.rs:705", "Controls cut critical ccw hp."),
    caution_cut_critical_hp => (2, "hp", 0, 10000, "borg_caution.rs:713", "Controls cut critical hp."),
    caution_cut_critical_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:713", "Controls cut critical hp divisor."),
    caution_cut_reserve_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:736", "Controls cut reserve hp divisor."),
    caution_desperate_stair_distance => (3, "grids", 1, 100, "borg_caution.rs:872", "Controls desperate stair distance."),
    caution_desperate_adjacent_up_distance => (2, "grids", 1, 100, "borg_caution.rs:881", "Controls desperate adjacent up distance."),
    caution_desperate_scary_distance => (20, "grids", 1, 100, "borg_caution.rs:886", "Controls desperate scary distance."),
    caution_desperate_scary_level => (20, "level", 1, 51, "borg_caution.rs:886", "Controls desperate scary level."),
    caution_desperate_unique_depth => (5, "depth", 0, 127, "borg_caution.rs:912", "Controls desperate unique depth."),
    caution_desperate_adjacent_down_distance => (2, "grids", 1, 100, "borg_caution.rs:922", "Controls desperate adjacent down distance."),
    caution_retreat_max_distance => (10, "grids", 1, 100, "borg_caution.rs:1009", "Controls retreat max distance."),
    caution_retreat_step_hp_tenths => (4, "tenths", 0, 100, "borg_caution.rs:1024", "Controls retreat step hp tenths."),
    caution_retreat_step_danger_tenths => (5, "tenths", 0, 100, "borg_caution.rs:1029", "Controls retreat step danger tenths."),
    caution_retreat_surrounded_level => (20, "level", 1, 51, "borg_caution.rs:1034", "Controls retreat surrounded level."),
    caution_retreat_surrounded_distance => (5, "grids", 1, 100, "borg_caution.rs:1035", "Controls retreat surrounded distance."),
    caution_retreat_surrounded_hp_tenths => (15, "tenths", 0, 100, "borg_caution.rs:1036", "Controls retreat surrounded hp tenths."),
    caution_retreat_low_level_hp_multiplier => (4, "multiplier", 0, 20, "borg_caution.rs:1043", "Controls retreat low level hp multiplier."),
    caution_retreat_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:1060", "Controls retreat monster age."),
    caution_back_surrounded_danger_tenths => (12, "tenths", 0, 100, "borg_caution.rs:1143", "Controls back surrounded danger tenths."),
    caution_back_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:1199", "Controls back monster age."),
    caution_back_fast_monster_distance => (2, "grids", 1, 100, "borg_caution.rs:1209", "Controls back fast monster distance."),
    caution_back_high_level => (35, "level", 1, 51, "borg_caution.rs:1235", "Controls back high level."),
    caution_back_high_level_danger_tenths => (6, "tenths", 0, 100, "borg_caution.rs:1235", "Controls back high level danger tenths."),
    caution_back_low_level => (35, "level", 1, 51, "borg_caution.rs:1243", "Controls back low level."),
    caution_back_low_level_danger_tenths => (8, "tenths", 0, 100, "borg_caution.rs:1243", "Controls back low level danger tenths."),
    caution_back_injured_level => (10, "level", 1, 51, "borg_caution.rs:1264", "Controls back injured level."),
    caution_back_freedom_depth => (85, "depth", 0, 127, "borg_caution.rs:1273", "Controls back freedom depth."),
    caution_move_antisummon_delay => (50, "turns", 0, 100000, "borg_caution.rs:1330", "Controls move antisummon delay."),
    caution_move_max_hp => (500, "hp", 0, 10000, "borg_caution.rs:1333", "Controls move max hp."),
    caution_one_blow_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:1365", "Controls one blow monster age."),
    caution_nasty_light_timeout => (250, "turns", 0, 100000, "borg_caution.rs:1425", "Controls nasty light timeout."),
    caution_questor_arrival_turns => (10, "turns", 0, 100000, "borg_caution.rs:1438", "Controls questor arrival turns."),
    caution_escape_limit => (3, "count", 0, 1000, "borg_caution.rs:1472", "Controls escape limit."),
    caution_escape_absolute_limit => (55, "count", 0, 1000, "borg_caution.rs:1473", "Controls escape absolute limit."),
    caution_escape_limit_max_depth => (98, "depth", 0, 127, "borg_caution.rs:1475", "Controls escape limit max depth."),
    caution_escape_flee_limit => (3, "count", 0, 1000, "borg_caution.rs:1481", "Controls escape flee limit."),
    caution_swap_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:1615", "Controls swap danger divisor."),
    caution_recall_prep_turns => (2, "turns", 0, 100000, "borg_caution.rs:1643", "Controls recall prep turns."),
    caution_restock_town_delay => (200, "turns", 0, 100000, "borg_caution.rs:1657", "Controls restock town delay."),
    caution_restock_ccw_count => (2, "count", 0, 1000, "borg_caution.rs:1671", "Controls restock ccw count."),
    caution_restock_food_count => (3, "count", 0, 1000, "borg_caution.rs:1672", "Controls restock food count."),
    caution_restock_fuel_count => (2, "count", 0, 1000, "borg_caution.rs:1673", "Controls restock fuel count."),
    caution_restock_level_delay => (400, "turns", 0, 100000, "borg_caution.rs:1674", "Controls restock level delay."),
    caution_flee_danger_hp_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:1685", "Controls flee danger hp multiplier."),
    caution_flee_max_level => (50, "level", 1, 51, "borg_caution.rs:1690", "Controls flee max level."),
    caution_town_flee_max_level => (50, "level", 1, 51, "borg_caution.rs:1704", "Controls town flee max level."),
    caution_stair_low_level => (5, "level", 1, 51, "borg_caution.rs:1719", "Controls stair low level."),
    caution_stair_danger_divisor => (2, "divisor", 1, 100, "borg_caution.rs:1719", "Controls stair danger divisor."),
    caution_breeder_flee_max_level => (15, "level", 1, 51, "borg_caution.rs:1799", "Controls breeder flee max level."),
    caution_desperate_max_level => (35, "level", 1, 51, "borg_caution.rs:1809", "Controls desperate max level."),
    caution_retreat_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:1822", "Controls retreat danger divisor."),
    caution_back_danger_tenths => (4, "tenths", 0, 100, "borg_caution.rs:1851", "Controls back danger tenths."),
    caution_confused_emergency_loss => (300, "hp", 0, 10000, "borg_caution.rs:1863", "Controls confused emergency loss."),
    caution_fear_cure_chance => (70, "percent", 0, 100, "borg_caution.rs:1885", "Controls fear cure chance."),
    caution_fear_berserk_fail => (25, "percent", 0, 100, "borg_caution.rs:1891", "Controls fear berserk fail."),
    caution_fear_hero_fail => (25, "percent", 0, 100, "borg_caution.rs:1892", "Controls fear hero fail."),
    caution_fear_holy_fail => (25, "percent", 0, 100, "borg_caution.rs:1899", "Controls fear holy fail."),
    caution_flee_low_hp_divisor => (3, "divisor", 1, 100, "borg_caution.rs:1907", "Controls flee low hp divisor."),
    caution_flee_moderate_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:1908", "Controls flee moderate hp divisor."),
    caution_flee_hp_per_level => (3, "hp_per_level", 0, 100, "borg_caution.rs:1909", "Controls flee hp per level."),
    caution_flee_ccw_count => (3, "count", 0, 1000, "borg_caution.rs:1910", "Controls flee ccw count."),
    caution_flee_heal_count => (1, "count", 0, 1000, "borg_caution.rs:1911", "Controls flee heal count."),
    caution_flee_low_hp_chance => (25, "percent", 0, 100, "borg_caution.rs:1914", "Controls flee low hp chance."),
    caution_flee_status_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:1922", "Controls flee status hp divisor."),
    caution_flee_status_chance => (25, "percent", 0, 100, "borg_caution.rs:1925", "Controls flee status chance."),
    caution_reserve_critical_hp_divisor => (10, "divisor", 1, 100, "borg_caution.rs:1934", "Controls reserve critical hp divisor."),
    caution_reserve_escape_count => (2, "count", 0, 1000, "borg_caution.rs:1936", "Controls reserve escape count."),
    caution_reserve_danger_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:1937", "Controls reserve danger hp divisor."),
    caution_reserve_heal_count => (5, "count", 0, 1000, "borg_caution.rs:1939", "Controls reserve heal count."),
    caution_reserve_surplus_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:1939", "Controls reserve surplus hp divisor."),
    caution_reserve_unique_loss => (600, "hp", 0, 10000, "borg_caution.rs:1941", "Controls reserve unique loss."),
    caution_reserve_unique_depth => (85, "depth", 0, 127, "borg_caution.rs:1943", "Controls reserve unique depth."),
    caution_recall_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:1970", "Controls recall danger multiplier."),
    caution_unknown_danger_multiplier => (4, "multiplier", 0, 20, "borg_caution.rs:1978", "Controls unknown danger multiplier."),
    caution_unknown_max_level => (20, "level", 1, 51, "borg_caution.rs:1979", "Controls unknown max level."),
    caution_stair_danger_max_level => (49, "level", 1, 51, "borg_caution.rs:2019", "Controls stair danger max level."),
    caution_stair_min_food => (2, "count", 0, 1000, "borg_caution.rs:2044", "Controls stair min food."),
    caution_sell_max_level => (25, "level", 1, 51, "borg_caution.rs:2052", "Controls sell max level."),
    caution_sell_max_gold => (25000, "gold", 0, 1000000, "borg_caution.rs:2053", "Controls sell max gold."),
    caution_sell_min_count => (13, "count", 0, 1000, "borg_caution.rs:2054", "Controls sell min count."),
    caution_recall_mana_maxsp => (60, "sp", 0, 10000, "borg_caution.rs:2077", "Controls recall mana maxsp."),
    caution_recall_mana_divisor => (4, "divisor", 1, 100, "borg_caution.rs:2078", "Controls recall mana divisor."),
    caution_recall_phase_tolerance => (50, "percent", 0, 100, "borg_caution.rs:2086", "Controls recall phase tolerance."),
    caution_recall_phase_horizon => (1, "turns", 1, 100000, "borg_caution.rs:2086", "Controls recall phase horizon."),
    caution_recall_phase_fail => (30, "percent", 0, 100, "borg_caution.rs:2088", "Controls recall phase fail."),
    caution_recall_portal_fail => (30, "percent", 0, 100, "borg_caution.rs:2089", "Controls recall portal fail."),
    caution_recall_cure_loss => (100, "hp", 0, 10000, "borg_caution.rs:2096", "Controls recall cure loss."),
    caution_recall_heal_loss => (150, "hp", 0, 10000, "borg_caution.rs:2105", "Controls recall heal loss."),
    caution_retreat_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:1014", "Number of Borg turns of danger evaluated along a retreat."),
    caution_back_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:1223", "Number of Borg turns of danger evaluated for a back-away grid."),
    caution_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:1514", "Number of Borg turns of danger evaluated at the current grid."),
    caution_recall_rest_turns => (500, "turns", 1, 9999, "borg_caution.rs:1630", "Number of rest turns queued while waiting safely for recall."),
    // borg-reborn-rs (#220): source lines in danger/escape/recovery at 30ab9d66.
    danger_monblow_hurt_fear => (400, "danger", 0, 5000, "borg_danger.rs:208", "Additional melee danger for MONBLOW_HURT (n += 400;)."),
    danger_monblow_hurt_hit_power => (60, "hit_power", 0, 1000, "borg_danger.rs:210", "Legacy hit-power estimate used for MONBLOW_HURT in partial melee danger."),
    danger_monblow_poison_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:218", "Legacy hit-power estimate used for MONBLOW_POISON in partial melee danger."),
    danger_monblow_poison_fear => (10, "danger", 0, 5000, "borg_danger.rs:226", "Additional melee danger for MONBLOW_POISON (z += 10;)."),
    danger_monblow_disenchant_hit_power => (20, "hit_power", 0, 1000, "borg_danger.rs:234", "Legacy hit-power estimate used for MONBLOW_DISENCHANT in partial melee danger."),
    danger_monblow_disenchant_fear => (500, "danger", 0, 5000, "borg_danger.rs:239", "Additional melee danger for MONBLOW_DISENCHANT (z += 500;)."),
    danger_monblow_drain_charges_fear => (20, "danger", 0, 5000, "borg_danger.rs:248", "Additional melee danger for MONBLOW_DRAIN_CHARGES (z += 20;)."),
    danger_monblow_drain_charges_hit_power => (15, "hit_power", 0, 1000, "borg_danger.rs:249", "Legacy hit-power estimate used for MONBLOW_DRAIN_CHARGES in partial melee danger."),
    danger_monblow_eat_gold_fear => (50, "danger", 0, 5000, "borg_danger.rs:259", "Additional melee danger for MONBLOW_EAT_GOLD (z += 50;)."),
    danger_monblow_eat_gold_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:261", "Legacy hit-power estimate used for MONBLOW_EAT_GOLD in partial melee danger."),
    danger_monblow_eat_gold_fear_2 => (5, "danger", 0, 5000, "borg_danger.rs:274", "Additional melee danger for MONBLOW_EAT_GOLD (z += 5;)."),
    danger_monblow_eat_item_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:282", "Legacy hit-power estimate used for MONBLOW_EAT_ITEM in partial melee danger."),
    danger_monblow_eat_item_fear => (5, "danger", 0, 5000, "borg_danger.rs:289", "Additional melee danger for MONBLOW_EAT_ITEM (z += 5;)."),
    danger_monblow_eat_food_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:297", "Legacy hit-power estimate used for MONBLOW_EAT_FOOD in partial melee danger."),
    danger_monblow_eat_food_fear => (5, "danger", 0, 5000, "borg_danger.rs:302", "Additional melee danger for MONBLOW_EAT_FOOD (z += 5;)."),
    danger_monblow_eat_light_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:310", "Legacy hit-power estimate used for MONBLOW_EAT_LIGHT in partial melee danger."),
    danger_monblow_eat_light_fear => (5, "danger", 0, 5000, "borg_danger.rs:319", "Additional melee danger for MONBLOW_EAT_LIGHT (z += 5;)."),
    danger_monblow_acid_fear => (200, "danger", 0, 5000, "borg_danger.rs:337", "Additional melee danger for MONBLOW_ACID (z += 200; // We don't want our armour corroded.)."),
    danger_monblow_elec_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:348", "Legacy hit-power estimate used for MONBLOW_ELEC in partial melee danger."),
    danger_monblow_elec_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:356", "Multiplies damage to account for MONBLOW_ELEC side effects."),
    danger_monblow_fire_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:367", "Legacy hit-power estimate used for MONBLOW_FIRE in partial melee danger."),
    danger_monblow_fire_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:375", "Multiplies damage to account for MONBLOW_FIRE side effects."),
    danger_monblow_cold_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:386", "Legacy hit-power estimate used for MONBLOW_COLD in partial melee danger."),
    danger_monblow_cold_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:394", "Multiplies damage to account for MONBLOW_COLD side effects."),
    danger_monblow_blind_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:402", "Legacy hit-power estimate used for MONBLOW_BLIND in partial melee danger."),
    danger_monblow_blind_fear => (10, "danger", 0, 5000, "borg_danger.rs:407", "Additional melee danger for MONBLOW_BLIND (z += 10;)."),
    danger_monblow_blind_fear_2 => (75, "danger", 0, 5000, "borg_danger.rs:409", "Additional melee danger for MONBLOW_BLIND (z += 75;)."),
    danger_monblow_confuse_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:418", "Legacy hit-power estimate used for MONBLOW_CONFUSE in partial melee danger."),
    danger_monblow_confuse_fear => (200, "danger", 0, 5000, "borg_danger.rs:423", "Additional melee danger for MONBLOW_CONFUSE (z += 200;)."),
    danger_monblow_confuse_fear_2 => (200, "danger", 0, 5000, "borg_danger.rs:425", "Additional melee danger for MONBLOW_CONFUSE (z += 200;)."),
    danger_monblow_terrify_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:434", "Legacy hit-power estimate used for MONBLOW_TERRIFY in partial melee danger."),
    danger_monblow_terrify_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:439", "Multiplies damage to account for MONBLOW_TERRIFY side effects."),
    danger_monblow_paralyze_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:447", "Legacy hit-power estimate used for MONBLOW_PARALYZE in partial melee danger."),
    danger_monblow_paralyze_fear => (200, "danger", 0, 5000, "borg_danger.rs:451", "Additional melee danger for MONBLOW_PARALYZE (z += 200;)."),
    danger_monblow_lose_str_fear => (150, "danger", 0, 5000, "borg_danger.rs:474", "Additional melee danger for MONBLOW_LOSE_STR (z += 150;)."),
    danger_monblow_lose_str_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:477", "Additional melee danger for MONBLOW_LOSE_STR (z += 100;)."),
    danger_monblow_lose_dex_fear => (150, "danger", 0, 5000, "borg_danger.rs:498", "Additional melee danger for MONBLOW_LOSE_DEX (z += 150;)."),
    danger_monblow_lose_dex_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:501", "Additional melee danger for MONBLOW_LOSE_DEX (z += 100;)."),
    danger_monblow_lose_con_fear => (150, "danger", 0, 5000, "borg_danger.rs:526", "Additional melee danger for MONBLOW_LOSE_CON (z += 150;)."),
    danger_monblow_lose_con_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:529", "Additional melee danger for MONBLOW_LOSE_CON (z += 100;)."),
    danger_monblow_lose_int_fear => (150, "danger", 0, 5000, "borg_danger.rs:553", "Additional melee danger for MONBLOW_LOSE_INT (z += 150;)."),
    danger_monblow_lose_int_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:556", "Additional melee danger for MONBLOW_LOSE_INT (z += 50;)."),
    danger_monblow_lose_wis_fear => (150, "danger", 0, 5000, "borg_danger.rs:577", "Additional melee danger for MONBLOW_LOSE_WIS (z += 150;)."),
    danger_monblow_lose_wis_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:580", "Additional melee danger for MONBLOW_LOSE_WIS (z += 50;)."),
    danger_monblow_lose_all_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:589", "Legacy hit-power estimate used for MONBLOW_LOSE_ALL in partial melee danger."),
    danger_monblow_shatter_hit_power => (60, "hit_power", 0, 1000, "borg_danger.rs:596", "Legacy hit-power estimate used for MONBLOW_SHATTER in partial melee danger."),
    danger_monblow_shatter_fear => (150, "danger", 0, 5000, "borg_danger.rs:598", "Additional melee danger for MONBLOW_SHATTER (z += 150;)."),
    danger_monblow_hallu_fear => (250, "danger", 0, 5000, "borg_danger.rs:636", "Additional melee danger for MONBLOW_HALLU (z += 250;)."),
    danger_rsf_shriek_fear => (5, "danger", 0, 5000, "borg_danger.rs:795", "Additional spell danger for RSF_SHRIEK (p += 5;)."),
    danger_rsf_br_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:838", "Additional spell danger for RSF_BR_ACID (p += 40;)."),
    danger_rsf_br_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:857", "Additional spell danger for RSF_BR_ELEC (p += 20;)."),
    danger_rsf_br_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:876", "Additional spell danger for RSF_BR_FIRE (p += 40;)."),
    danger_rsf_br_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:895", "Additional spell danger for RSF_BR_COLD (p += 20;)."),
    danger_rsf_br_pois_fear => (20, "danger", 0, 5000, "borg_danger.rs:917", "Additional spell danger for RSF_BR_POIS (p += 20;)."),
    danger_rsf_br_neth_fear => (125, "danger", 0, 5000, "borg_danger.rs:931", "Additional spell danger for RSF_BR_NETH (p += 125;)."),
    danger_rsf_br_light_fear => (20, "danger", 0, 5000, "borg_danger.rs:947", "Additional spell danger for RSF_BR_LIGHT (p += 20;)."),
    danger_rsf_br_light_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:949", "Additional spell danger for RSF_BR_LIGHT (p += 20;)."),
    danger_rsf_br_dark_fear => (20, "danger", 0, 5000, "borg_danger.rs:968", "Additional spell danger for RSF_BR_DARK (p += 20;)."),
    danger_rsf_br_dark_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:970", "Additional spell danger for RSF_BR_DARK (p += 20;)."),
    danger_rsf_br_soun_fear => (500, "danger", 0, 5000, "borg_danger.rs:988", "Additional spell danger for RSF_BR_SOUN (z += 500;)."),
    danger_rsf_br_soun_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:991", "Additional spell danger for RSF_BR_SOUN (z += 1000;)."),
    danger_rsf_br_soun_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:994", "Additional spell danger for RSF_BR_SOUN (p += 50;)."),
    danger_rsf_br_chao_fear => (100, "danger", 0, 5000, "borg_danger.rs:1007", "Additional spell danger for RSF_BR_CHAO (p += 100;)."),
    danger_rsf_br_chao_fear_2 => (200, "danger", 0, 5000, "borg_danger.rs:1011", "Additional spell danger for RSF_BR_CHAO (p += 200;)."),
    danger_rsf_br_dise_fear => (500, "danger", 0, 5000, "borg_danger.rs:1026", "Additional spell danger for RSF_BR_DISE (p += 500;)."),
    danger_rsf_br_nexu_fear => (100, "danger", 0, 5000, "borg_danger.rs:1042", "Additional spell danger for RSF_BR_NEXU (p += 100;)."),
    danger_rsf_br_time_fear => (250, "danger", 0, 5000, "borg_danger.rs:1052", "Additional spell danger for RSF_BR_TIME (p += 250;)."),
    danger_rsf_br_iner_fear => (100, "danger", 0, 5000, "borg_danger.rs:1062", "Additional spell danger for RSF_BR_INER (p += 100;)."),
    danger_rsf_br_grav_fear => (100, "danger", 0, 5000, "borg_danger.rs:1072", "Additional spell danger for RSF_BR_GRAV (p += 100;)."),
    danger_rsf_br_grav_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1078", "Additional spell danger for RSF_BR_GRAV (z += 500;)."),
    danger_rsf_br_grav_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1081", "Additional spell danger for RSF_BR_GRAV (z += 1000;)."),
    danger_rsf_br_shar_fear => (50, "danger", 0, 5000, "borg_danger.rs:1098", "Additional spell danger for RSF_BR_SHAR (p += 50;)."),
    danger_rsf_br_plas_fear => (100, "danger", 0, 5000, "borg_danger.rs:1112", "Additional spell danger for RSF_BR_PLAS (p += 100;)."),
    danger_rsf_br_plas_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1115", "Additional spell danger for RSF_BR_PLAS (z += 500;)."),
    danger_rsf_br_plas_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1118", "Additional spell danger for RSF_BR_PLAS (z += 1000;)."),
    danger_rsf_br_wall_fear => (100, "danger", 0, 5000, "borg_danger.rs:1133", "Additional spell danger for RSF_BR_WALL (z += 100;)."),
    danger_rsf_br_wall_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1136", "Additional spell danger for RSF_BR_WALL (z += 500;)."),
    danger_rsf_br_wall_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:1139", "Additional spell danger for RSF_BR_WALL (p += 50;)."),
    danger_rsf_ba_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:1172", "Additional spell danger for RSF_BA_ACID (p += 40;)."),
    danger_rsf_ba_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:1187", "Additional spell danger for RSF_BA_ELEC (p += 20;)."),
    danger_rsf_ba_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:1202", "Additional spell danger for RSF_BA_FIRE (p += 40;)."),
    danger_rsf_ba_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:1217", "Additional spell danger for RSF_BA_COLD (p += 20;)."),
    danger_rsf_ba_pois_fear => (20, "danger", 0, 5000, "borg_danger.rs:1235", "Additional spell danger for RSF_BA_POIS (p += 20;)."),
    danger_rsf_ba_shar_fear => (20, "danger", 0, 5000, "borg_danger.rs:1247", "Additional spell danger for RSF_BA_SHAR (p += 20;)."),
    danger_rsf_ba_neth_fear => (250, "danger", 0, 5000, "borg_danger.rs:1259", "Additional spell danger for RSF_BA_NETH (p += 250;)."),
    danger_rsf_ba_wate_fear => (500, "danger", 0, 5000, "borg_danger.rs:1269", "Additional spell danger for RSF_BA_WATE (p += 500;)."),
    danger_rsf_ba_wate_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1272", "Additional spell danger for RSF_BA_WATE (p += 1000;)."),
    danger_rsf_ba_wate_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:1278", "Additional spell danger for RSF_BA_WATE (p += 50;)."),
    danger_rsf_ba_wate_fear_4 => (20, "danger", 0, 5000, "borg_danger.rs:1280", "Additional spell danger for RSF_BA_WATE (p += 20;)."),
    danger_rsf_ba_mana_fear => (50, "danger", 0, 5000, "borg_danger.rs:1287", "Additional spell danger for RSF_BA_MANA (p += 50;)."),
    danger_rsf_ba_holy_fear => (50, "danger", 0, 5000, "borg_danger.rs:1293", "Additional spell danger for RSF_BA_HOLY (p += 50;)."),
    danger_rsf_ba_dark_fear => (20, "danger", 0, 5000, "borg_danger.rs:1308", "Additional spell danger for RSF_BA_DARK (p += 20;)."),
    danger_rsf_ba_dark_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:1310", "Additional spell danger for RSF_BA_DARK (p += 20;)."),
    danger_rsf_ba_light_fear => (20, "danger", 0, 5000, "borg_danger.rs:1326", "Additional spell danger for RSF_BA_LIGHT (p += 20;)."),
    danger_rsf_ba_light_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:1328", "Additional spell danger for RSF_BA_LIGHT (p += 20;)."),
    danger_rsf_storm_fear => (500, "danger", 0, 5000, "borg_danger.rs:1339", "Additional spell danger for RSF_STORM (p += 500;)."),
    danger_rsf_storm_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1342", "Additional spell danger for RSF_STORM (p += 1000;)."),
    danger_rsf_drain_mana_fear => (100, "danger", 0, 5000, "borg_danger.rs:1350", "Additional spell danger for RSF_DRAIN_MANA (p += 100;)."),
    danger_rsf_brain_smash_fear => (200, "danger", 0, 5000, "borg_danger.rs:1362", "Additional spell danger for RSF_BRAIN_SMASH (p += 200 - 2 * st.borg.tr(BI_SAV);)."),
    danger_rsf_bo_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:1390", "Additional spell danger for RSF_BO_ACID (p += 40;)."),
    danger_rsf_bo_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:1406", "Additional spell danger for RSF_BO_ELEC (p += 20;)."),
    danger_rsf_bo_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:1422", "Additional spell danger for RSF_BO_FIRE (p += 40;)."),
    danger_rsf_bo_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:1438", "Additional spell danger for RSF_BO_COLD (p += 20;)."),
    danger_rsf_bo_neth_fear => (200, "danger", 0, 5000, "borg_danger.rs:1465", "Additional spell danger for RSF_BO_NETH (p += 200;)."),
    danger_rsf_bo_wate_fear => (500, "danger", 0, 5000, "borg_danger.rs:1476", "Additional spell danger for RSF_BO_WATE (p += 500;)."),
    danger_rsf_bo_wate_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1479", "Additional spell danger for RSF_BO_WATE (p += 1000;)."),
    danger_rsf_bo_wate_fear_3 => (20, "danger", 0, 5000, "borg_danger.rs:1485", "Additional spell danger for RSF_BO_WATE (p += 20;)."),
    danger_rsf_bo_wate_fear_4 => (20, "danger", 0, 5000, "borg_danger.rs:1487", "Additional spell danger for RSF_BO_WATE (p += 20;)."),
    danger_rsf_bo_mana_fear => (50, "danger", 0, 5000, "borg_danger.rs:1495", "Additional spell danger for RSF_BO_MANA (p += 50;)."),
    danger_rsf_bo_plas_fear => (500, "danger", 0, 5000, "borg_danger.rs:1506", "Additional spell danger for RSF_BO_PLAS (z += 500;)."),
    danger_rsf_bo_plas_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1509", "Additional spell danger for RSF_BO_PLAS (z += 1000;)."),
    danger_rsf_bo_ice_fear => (20, "danger", 0, 5000, "borg_danger.rs:1517", "Additional spell danger for RSF_BO_ICE (p += 20;)."),
    danger_rsf_bo_ice_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:1523", "Additional spell danger for RSF_BO_ICE (z += 50;)."),
    danger_rsf_bo_ice_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1526", "Additional spell danger for RSF_BO_ICE (z += 1000;)."),
    danger_rsf_scare_fear => (10, "danger", 0, 5000, "borg_danger.rs:1560", "Additional spell danger for RSF_SCARE (p += 10;)."),
    danger_rsf_blind_fear => (10, "danger", 0, 5000, "borg_danger.rs:1572", "Additional spell danger for RSF_BLIND (p += 10;)."),
    danger_rsf_conf_fear => (10, "danger", 0, 5000, "borg_danger.rs:1583", "Additional spell danger for RSF_CONF (p += 10;)."),
    danger_rsf_slow_fear => (5, "danger", 0, 5000, "borg_danger.rs:1594", "Additional spell danger for RSF_SLOW (p += 5;)."),
    danger_rsf_hold_fear => (150, "danger", 0, 5000, "borg_danger.rs:1604", "Additional spell danger for RSF_HOLD (p += 150;)."),
    danger_rsf_haste_fear => (10, "danger", 0, 5000, "borg_danger.rs:1609", "Additional spell danger for RSF_HASTE (p += 10;)."),
    danger_rsf_heal_fear => (10, "danger", 0, 5000, "borg_danger.rs:1614", "Additional spell danger for RSF_HEAL (p += 10;)."),
    danger_rsf_tport_fear => (10, "danger", 0, 5000, "borg_danger.rs:1623", "Additional spell danger for RSF_TPORT (p += 10;)."),
    danger_rsf_tele_to_fear => (20, "danger", 0, 5000, "borg_danger.rs:1628", "Additional spell danger for RSF_TELE_TO (p += 20;)."),
    danger_rsf_tele_self_to_fear => (20, "danger", 0, 5000, "borg_danger.rs:1633", "Additional spell danger for RSF_TELE_SELF_TO (p += 20;)."),
    danger_rsf_tele_away_fear => (10, "danger", 0, 5000, "borg_danger.rs:1638", "Additional spell danger for RSF_TELE_AWAY (p += 10;)."),
    danger_rsf_tele_level_fear => (50, "danger", 0, 5000, "borg_danger.rs:1644", "Additional spell danger for RSF_TELE_LEVEL (p += 50;)."),
    danger_rsf_darkness_fear => (5, "danger", 0, 5000, "borg_danger.rs:1650", "Additional spell danger for RSF_DARKNESS (p += 5;)."),
    danger_rsf_traps_fear => (50, "danger", 0, 5000, "borg_danger.rs:1655", "Additional spell danger for RSF_TRAPS (p += 50;)."),
    danger_rsf_forget_fear => (500, "danger", 0, 5000, "borg_danger.rs:1662", "Additional spell danger for RSF_FORGET (p += 500;)."),
    danger_rsf_forget_fear_2 => (30, "danger", 0, 5000, "borg_danger.rs:1664", "Additional spell danger for RSF_FORGET (p += 30;)."),
    danger_rsf_shapechange_fear => (200, "danger", 0, 5000, "borg_danger.rs:1671", "Additional spell danger for RSF_SHAPECHANGE (p += 200;)."),
    danger_rsf_s_kin_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1688", "Spell-power risk multiplier in RSF_S_KIN, branch 1."),
    danger_rsf_s_kin_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1691", "Spell-power risk multiplier in RSF_S_KIN, branch 2."),
    danger_summon_unique_percent => (75, "percent", 0, 100, "borg_danger.rs:1696", "Percentage of summon fear retained when the summoner is a unique."),
    danger_rsf_s_hi_demon_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1706", "Spell-power risk multiplier in RSF_S_HI_DEMON, branch 1."),
    danger_rsf_s_hi_demon_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1709", "Spell-power risk multiplier in RSF_S_HI_DEMON, branch 2."),
    danger_rsf_s_monster_power_multiplier => (5, "multiplier", 0, 50, "borg_danger.rs:1723", "Spell-power risk multiplier in RSF_S_MONSTER, branch 1."),
    danger_rsf_s_monsters_power_multiplier => (7, "multiplier", 0, 50, "borg_danger.rs:1733", "Spell-power risk multiplier in RSF_S_MONSTERS, branch 1."),
    danger_rsf_s_animal_power_multiplier => (5, "multiplier", 0, 50, "borg_danger.rs:1747", "Spell-power risk multiplier in RSF_S_ANIMAL, branch 1."),
    danger_rsf_s_hydra_power_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:1762", "Spell-power risk multiplier in RSF_S_HYDRA, branch 1."),
    danger_rsf_s_hydra_power_multiplier_2 => (5, "multiplier", 0, 50, "borg_danger.rs:1765", "Spell-power risk multiplier in RSF_S_HYDRA, branch 2."),
    danger_rsf_s_ainu_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1780", "Spell-power risk multiplier in RSF_S_AINU, branch 1."),
    danger_rsf_s_ainu_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1783", "Spell-power risk multiplier in RSF_S_AINU, branch 2."),
    danger_rsf_s_demon_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1798", "Spell-power risk multiplier in RSF_S_DEMON, branch 1."),
    danger_rsf_s_demon_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1801", "Spell-power risk multiplier in RSF_S_DEMON, branch 2."),
    danger_rsf_s_hi_undead_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1816", "Spell-power risk multiplier in RSF_S_HI_UNDEAD, branch 1."),
    danger_rsf_s_hi_undead_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1819", "Spell-power risk multiplier in RSF_S_HI_UNDEAD, branch 2."),
    danger_rsf_s_hi_dragon_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1834", "Spell-power risk multiplier in RSF_S_HI_DRAGON, branch 1."),
    danger_rsf_s_hi_dragon_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1837", "Spell-power risk multiplier in RSF_S_HI_DRAGON, branch 2."),
    danger_rsf_s_unique_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1852", "Spell-power risk multiplier in RSF_S_UNIQUE, branch 1."),
    danger_rsf_s_unique_power_multiplier_2 => (6, "multiplier", 0, 50, "borg_danger.rs:1855", "Spell-power risk multiplier in RSF_S_UNIQUE, branch 2."),
    danger_unknown_physical => (1000, "danger", 0, 5000, "borg_danger.rs:175", "Sets unknown physical, measured in danger, in the existing danger rule."),
    danger_knockout_max_sides => (3, "count", 0, 1000, "borg_danger.rs:207", "Sets knockout max sides, measured in count, in the existing danger rule."),
    danger_knockout_min_dice => (5, "count", 0, 1000, "borg_danger.rs:207", "Sets knockout min dice, measured in count, in the existing danger rule."),
    danger_gold_theft_low_level => (5, "level", 1, 51, "borg_danger.rs:258", "Sets gold theft low level, measured in level, in the existing danger rule."),
    danger_gold_theft_min_gold => (100, "gold", 0, 1000000, "borg_danger.rs:267", "Sets gold theft min gold, measured in gold, in the existing danger rule."),
    danger_gold_theft_max_gold => (100000, "gold", 0, 1000000, "borg_danger.rs:270", "Sets gold theft max gold, measured in gold, in the existing danger rule."),
    danger_food_theft_reserve => (5, "count", 0, 1000, "borg_danger.rs:298", "Sets food theft reserve, measured in count, in the existing danger rule."),
    danger_fuel_theft_reserve => (5, "count", 0, 1000, "borg_danger.rs:315", "Sets fuel theft reserve, measured in count, in the existing danger rule."),
    danger_str_drain_low_stat => (10, "stat", 3, 238, "borg_danger.rs:476", "Sets str drain low stat, measured in stat, in the existing danger rule."),
    danger_dex_drain_low_stat => (10, "stat", 3, 238, "borg_danger.rs:500", "Sets dex drain low stat, measured in stat, in the existing danger rule."),
    danger_con_drain_low_stat => (8, "stat", 3, 238, "borg_danger.rs:528", "Sets con drain low stat, measured in stat, in the existing danger rule."),
    danger_shatter_ac_cap => (150, "ac", 0, 1000, "borg_danger.rs:595", "Sets shatter ac cap, measured in ac, in the existing danger rule."),
    danger_shatter_ac_divisor => (250, "divisor", 1, 1000, "borg_danger.rs:595", "Sets shatter ac divisor, measured in divisor, in the existing danger rule."),
    danger_exp10_fear => (100, "danger", 0, 5000, "borg_danger.rs:608", "Sets exp10 fear, measured in danger, in the existing danger rule."),
    danger_exp_ignore_level => (50, "level", 1, 51, "borg_danger.rs:608", "Sets exp ignore level, measured in level, in the existing danger rule."),
    danger_exp20_fear => (150, "danger", 0, 5000, "borg_danger.rs:609", "Sets exp20 fear, measured in danger, in the existing danger rule."),
    danger_exp40_fear => (200, "danger", 0, 5000, "borg_danger.rs:610", "Sets exp40 fear, measured in danger, in the existing danger rule."),
    danger_exp80_fear => (250, "danger", 0, 5000, "borg_danger.rs:611", "Sets exp80 fear, measured in danger, in the existing danger rule."),
    danger_melee_hit_luck_percent => (150, "percent", 100, 300, "borg_danger.rs:657", "Sets melee hit luck percent, measured in percent, in the existing danger rule."),
    danger_melee_hit_ac_scale => (300, "coefficient", 0, 1000, "borg_danger.rs:657", "Sets melee hit ac scale, measured in coefficient, in the existing danger rule."),
    danger_melee_hit_ac_divisor => (4, "divisor", 1, 1000, "borg_danger.rs:657", "Sets melee hit ac divisor, measured in divisor, in the existing danger rule."),
    danger_melee_hit_power_scale => (3, "divisor", 1, 1000, "borg_danger.rs:657", "Sets melee hit power scale, measured in divisor, in the existing danger rule."),
    danger_melee_hit_floor_percent => (5, "percent", 0, 100, "borg_danger.rs:663", "Sets melee hit floor percent, measured in percent, in the existing danger rule."),
    danger_summon_protected_divisor => (100, "divisor", 1, 1000, "borg_danger.rs:707", "Sets summon protected divisor, measured in divisor, in the existing danger rule."),
    danger_summon_corridor_divisor => (1000, "divisor", 1, 10000, "borg_danger.rs:710", "Sets summon corridor divisor, measured in divisor, in the existing danger rule."),
    danger_unknown_spell => (1000, "danger", 0, 5000, "borg_danger.rs:766", "Sets unknown spell, measured in danger, in the existing danger rule."),
    danger_brain_smash_save_weight => (2, "multiplier", 0, 50, "borg_danger.rs:1362", "Sets brain smash save weight, measured in multiplier, in the existing danger rule."),
    danger_cause4_save_bias => (120, "save_points", 0, 1000, "borg_danger.rs:1374", "Sets cause4 save bias, measured in save_points, in the existing danger rule."),
    danger_forget_low_sp => (15, "sp", 0, 10000, "borg_danger.rs:1661", "Sets forget low sp, measured in sp, in the existing danger rule."),
    danger_ignore_side_effects_depth => (75, "depth", 0, 127, "borg_danger.rs:1876", "Sets ignore side effects depth, measured in depth, in the existing danger rule."),
    danger_corridor_spell_tenths => (7, "tenths", 0, 30, "borg_danger.rs:1901", "Sets corridor spell tenths, measured in tenths, in the existing danger rule."),
    danger_worst_spell_average_tenths => (15, "tenths", 0, 30, "borg_danger.rs:1948", "Sets worst spell average tenths, measured in tenths, in the existing danger rule."),
    danger_worst_spell_hp_tenths => (8, "tenths", 0, 30, "borg_danger.rs:1948", "Sets worst spell hp tenths, measured in tenths, in the existing danger rule."),
    danger_sleep_level_margin => (15, "level", 1, 51, "borg_danger.rs:1972", "Sets sleep level margin, measured in level, in the existing danger rule."),
    danger_ghost_monster => (100, "danger", 0, 5000, "borg_danger.rs:2035", "Sets ghost monster, measured in danger, in the existing danger rule."),
    danger_monster_max_distance => (20, "grids", 1, 100, "borg_danger.rs:2067", "Sets monster max distance, measured in grids, in the existing danger rule."),
    danger_fast_player_threshold => (135, "speed", 110, 135, "borg_danger.rs:2072", "Sets fast player threshold, measured in speed, in the existing danger rule."),
    danger_fast_player_unique_speed => (120, "speed", 110, 189, "borg_danger.rs:2073", "Sets fast player unique speed, measured in speed, in the existing danger rule."),
    danger_fast_player_normal_speed => (125, "speed", 110, 189, "borg_danger.rs:2073", "Sets fast player normal speed, measured in speed, in the existing danger rule."),
    danger_low_hp_speed_limit => (20, "hp", 0, 10000, "borg_danger.rs:2085", "Sets low hp speed limit, measured in hp, in the existing danger rule."),
    danger_low_hp_monster_speed_bonus => (3, "speed", 0, 3, "borg_danger.rs:2086", "Sets low hp monster speed bonus, measured in speed, in the existing danger rule."),
    danger_minimum_monster_energy => (10, "energy_tenths", 1, 100, "borg_danger.rs:2122", "Sets minimum monster energy, measured in energy_tenths, in the existing danger rule."),
    danger_stuck_bounce_limit => (1200, "count", 0, 10000, "borg_danger.rs:2135", "Sets stuck bounce limit, measured in count, in the existing danger rule."),
    danger_stuck_level_turns => (25000, "turns", 0, 100000, "borg_danger.rs:2135", "Sets stuck level turns, measured in turns, in the existing danger rule."),
    danger_stuck_melee_divisor => (5, "divisor", 1, 1000, "borg_danger.rs:2136", "Sets stuck melee divisor, measured in divisor, in the existing danger rule."),
    danger_breeder_melee_max_level => (20, "level", 1, 51, "borg_danger.rs:2150", "Sets breeder melee max level, measured in level, in the existing danger rule."),
    danger_breeder_melee_extra_tenths => (15, "tenths", 0, 30, "borg_danger.rs:2152", "Sets breeder melee extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_melee_max_level => (20, "level", 1, 51, "borg_danger.rs:2158", "Sets friends melee max level, measured in level, in the existing danger rule."),
    danger_friends_melee_low_level => (15, "level", 1, 51, "borg_danger.rs:2159", "Sets friends melee low level, measured in level, in the existing danger rule."),
    danger_friends_melee_low_extra_tenths => (18, "tenths", 0, 30, "borg_danger.rs:2161", "Sets friends melee low extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_melee_extra_tenths => (13, "tenths", 0, 30, "borg_danger.rs:2164", "Sets friends melee extra tenths, measured in tenths, in the existing danger rule."),
    danger_sleeping_melee_alertness => (5, "sleep_rating", 0, 1000, "borg_danger.rs:2170", "Sets sleeping melee alertness, measured in sleep_rating, in the existing danger rule."),
    danger_sleeping_melee_min_level => (25, "level", 1, 51, "borg_danger.rs:2172", "Sets sleeping melee min level, measured in level, in the existing danger rule."),
    danger_sleeping_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2173", "Sets sleeping melee divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_melee_weak_level => (20, "level", 1, 51, "borg_danger.rs:2183", "Sets sleep2 melee weak level, measured in level, in the existing danger rule."),
    danger_sleep2_melee_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2183", "Sets sleep2 melee weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_melee_divisor => (3, "divisor", 1, 1000, "borg_danger.rs:2186", "Sets sleep2 melee divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_melee_weak_level => (20, "level", 1, 51, "borg_danger.rs:2192", "Sets sleep melee weak level, measured in level, in the existing danger rule."),
    danger_sleep_melee_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2192", "Sets sleep melee weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_melee_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2195", "Sets sleep melee distance offset, measured in count, in the existing danger rule."),
    danger_confused_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2204", "Sets confused melee divisor, measured in divisor, in the existing danger rule."),
    danger_stunned_melee_numerator => (10, "coefficient", 0, 1000, "borg_danger.rs:2207", "Sets stunned melee numerator, measured in coefficient, in the existing danger rule."),
    danger_stunned_melee_divisor => (13, "divisor", 1, 1000, "borg_danger.rs:2207", "Sets stunned melee divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_weak_level => (20, "level", 1, 51, "borg_danger.rs:2211", "Sets confuse spell weak level, measured in level, in the existing danger rule."),
    danger_confuse_spell_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2211", "Sets confuse spell weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2214", "Sets confuse spell distance offset, measured in count, in the existing danger rule."),
    danger_stuck_spell_divisor => (5, "divisor", 1, 1000, "borg_danger.rs:2401", "Sets stuck spell divisor, measured in divisor, in the existing danger rule."),
    danger_breeder_spell_max_level => (20, "level", 1, 51, "borg_danger.rs:2405", "Sets breeder spell max level, measured in level, in the existing danger rule."),
    danger_breeder_spell_extra_tenths => (12, "tenths", 0, 30, "borg_danger.rs:2406", "Sets breeder spell extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_spell_max_level => (20, "level", 1, 51, "borg_danger.rs:2410", "Sets friends spell max level, measured in level, in the existing danger rule."),
    danger_friends_spell_extra_tenths => (12, "tenths", 0, 30, "borg_danger.rs:2411", "Sets friends spell extra tenths, measured in tenths, in the existing danger rule."),
    danger_sleeping_spell_alertness => (5, "sleep_rating", 0, 1000, "borg_danger.rs:2416", "Sets sleeping spell alertness, measured in sleep_rating, in the existing danger rule."),
    danger_sleeping_spell_min_level => (25, "level", 1, 51, "borg_danger.rs:2418", "Sets sleeping spell min level, measured in level, in the existing danger rule."),
    danger_sleeping_spell_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2419", "Sets sleeping spell divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_spell_linear_level => (15, "level", 1, 51, "borg_danger.rs:2430", "Sets sleep2 spell linear level, measured in level, in the existing danger rule."),
    danger_sleep2_spell_level_subtract => (10, "levels", 0, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level subtract, measured in levels, in the existing danger rule."),
    danger_sleep2_spell_level_divisor => (4, "divisor", 1, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_spell_level_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:2430", "Sets sleep2 spell level multiplier, measured in multiplier, in the existing danger rule."),
    danger_sleep2_spell_level_add => (10, "levels", 0, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level add, measured in levels, in the existing danger rule."),
    danger_sleep2_spell_divisor => (3, "divisor", 1, 1000, "borg_danger.rs:2437", "Sets sleep2 spell divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_spell_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2448", "Sets sleep spell distance offset, measured in count, in the existing danger rule."),
    danger_confused_spell_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2452", "Sets confused spell divisor, measured in divisor, in the existing danger rule."),
    danger_stunned_spell_numerator => (10, "coefficient", 0, 1000, "borg_danger.rs:2456", "Sets stunned spell numerator, measured in coefficient, in the existing danger rule."),
    danger_stunned_spell_divisor => (13, "divisor", 1, 1000, "borg_danger.rs:2456", "Sets stunned spell divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_divisor => (6, "divisor", 1, 1000, "borg_danger.rs:2459", "Sets confuse spell divisor, measured in divisor, in the existing danger rule."),
    danger_frequency_low_percent => (11, "percent", 0, 100, "borg_danger.rs:2467", "Sets frequency low percent, measured in percent, in the existing danger rule."),
    danger_frequency_low_tenths => (4, "tenths", 0, 30, "borg_danger.rs:2468", "Sets frequency low tenths, measured in tenths, in the existing danger rule."),
    danger_frequency_mid_percent => (26, "percent", 0, 100, "borg_danger.rs:2469", "Sets frequency mid percent, measured in percent, in the existing danger rule."),
    danger_frequency_mid_tenths => (6, "tenths", 0, 30, "borg_danger.rs:2470", "Sets frequency mid tenths, measured in tenths, in the existing danger rule."),
    danger_frequency_high_percent => (51, "percent", 0, 100, "borg_danger.rs:2471", "Sets frequency high percent, measured in percent, in the existing danger rule."),
    danger_frequency_high_tenths => (8, "tenths", 0, 30, "borg_danger.rs:2472", "Sets frequency high tenths, measured in tenths, in the existing danger rule."),
    danger_maximum => (2000, "danger", 0, 5000, "borg_danger.rs:2487", "Sets maximum, measured in danger, in the existing danger rule."),
    danger_regional_fear_max_depth => (80, "depth", 0, 127, "borg_danger.rs:2544", "Sets regional fear max depth, measured in depth, in the existing danger rule."),
    danger_morgoth_regional_fear_cap => (300, "danger", 0, 5000, "borg_danger.rs:2549", "Sets morgoth regional fear cap, measured in danger, in the existing danger rule."),
    danger_monster_fear_bounce_limit => (200, "count", 0, 1000, "borg_danger.rs:2556", "Sets monster fear bounce limit, measured in count, in the existing danger rule."),
    danger_rsf_s_kin_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1685", "Protected-summon spell-power risk multiplier for RSF_S_KIN."),
    danger_rsf_s_hi_demon_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1703", "Protected-summon spell-power risk multiplier for RSF_S_HI_DEMON."),
    danger_rsf_s_hydra_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1759", "Protected-summon spell-power risk multiplier for RSF_S_HYDRA."),
    danger_rsf_s_ainu_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1777", "Protected-summon spell-power risk multiplier for RSF_S_AINU."),
    danger_rsf_s_demon_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1795", "Protected-summon spell-power risk multiplier for RSF_S_DEMON."),
    danger_rsf_s_hi_undead_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1813", "Protected-summon spell-power risk multiplier for RSF_S_HI_UNDEAD."),
    danger_rsf_s_unique_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1849", "Protected-summon spell-power risk multiplier for RSF_S_UNIQUE."),
    escape_antisummon_reset_turns => (50, "turns", 0, 100000, "borg_escape.rs:77", "Sets antisummon reset turns, measured in turns, in the existing escape rule."),
    escape_recall_fail => (60, "percent", 0, 100, "borg_escape.rs:95", "Sets recall fail, measured in percent, in the existing escape rule."),
    escape_recall_depth_window => (5, "depth", 0, 127, "borg_escape.rs:107", "Sets recall depth window, measured in depth, in the existing escape rule."),
    escape_recall_reset_min_depth => (80, "depth", 0, 127, "borg_escape.rs:111", "Sets recall reset min depth, measured in depth, in the existing escape rule."),
    escape_recall_reset_unique_count => (5, "count", 0, 1000, "borg_escape.rs:127", "Sets recall reset unique count, measured in count, in the existing escape rule."),
    escape_surround_distance => (3, "grids", 1, 100, "borg_escape.rs:197", "Sets surround distance, measured in grids, in the existing escape rule."),
    escape_surround_single_exit_count => (1, "count", 1, 8, "borg_escape.rs:265", "Sets surround single exit count, measured in count, in the existing escape rule."),
    escape_surround_single_monster_count => (1, "count", 1, 8, "borg_escape.rs:265", "Sets surround single monster count, measured in count, in the existing escape rule."),
    escape_town_stair_value => (1000, "freedom_score", 0, 5000, "borg_escape.rs:316", "Sets town stair value, measured in freedom_score, in the existing escape rule."),
    escape_town_stair_close_distance => (4, "grids", 1, 100, "borg_escape.rs:319", "Sets town stair close distance, measured in grids, in the existing escape rule."),
    escape_town_stair_close_value => (2000, "freedom_score", 0, 5000, "borg_escape.rs:320", "Sets town stair close value, measured in freedom_score, in the existing escape rule."),
    escape_town_stair_distance_penalty => (500, "freedom_score", 0, 5000, "borg_escape.rs:320", "Sets town stair distance penalty, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_value => (1000, "freedom_score", 0, 5000, "borg_escape.rs:330", "Sets dungeon stair value, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_close_distance => (4, "grids", 1, 100, "borg_escape.rs:333", "Sets dungeon stair close distance, measured in grids, in the existing escape rule."),
    escape_dungeon_stair_close_value => (2000, "freedom_score", 0, 5000, "borg_escape.rs:334", "Sets dungeon stair close value, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_distance_penalty => (500, "freedom_score", 0, 5000, "borg_escape.rs:334", "Sets dungeon stair distance penalty, measured in freedom_score, in the existing escape rule."),
    escape_landing_samples => (100, "count", 1, 1000, "borg_escape.rs:362", "Sets landing samples, measured in count, in the existing escape rule."),
    escape_landing_trials => (100, "count", 1, 1000, "borg_escape.rs:368", "Sets landing trials, measured in count, in the existing escape rule."),
    escape_unknown_landing_max_hp => (30, "hp", 0, 10000, "borg_escape.rs:418", "Sets unknown landing max hp, measured in hp, in the existing escape rule."),
    escape_unknown_teleport_level_turns => (2000, "turns", 0, 100000, "borg_escape.rs:487", "Sets unknown teleport level turns, measured in turns, in the existing escape rule."),
    escape_shadow_shift_min_hp => (12, "hp", 0, 10000, "borg_escape.rs:549", "Sets shadow shift min hp, measured in hp, in the existing escape rule."),
    escape_dimension_door_range => (50, "grids", 1, 100, "borg_escape.rs:558", "Sets dimension door range, measured in grids, in the existing escape rule."),
    escape_dimension_door_horizon => (2, "turns", 1, 10, "borg_escape.rs:601", "Sets dimension door horizon, measured in turns, in the existing escape rule."),
    escape_near_unique_max_count => (8, "count", 1, 9, "borg_escape.rs:650", "Sets near unique max count, measured in count, in the existing escape rule."),
    escape_normal_fail => (25, "percent", 0, 100, "borg_escape.rs:659", "Sets normal fail, measured in percent, in the existing escape rule."),
    escape_healthy_hp_percent => (70, "percent", 0, 100, "borg_escape.rs:662", "Sets healthy hp percent, measured in percent, in the existing escape rule."),
    escape_healthy_fail => (10, "percent", 0, 100, "borg_escape.rs:663", "Sets healthy fail, measured in percent, in the existing escape rule."),
    escape_stunned_fail => (35, "percent", 0, 100, "borg_escape.rs:668", "Sets stunned fail, measured in percent, in the existing escape rule."),
    escape_morgoth_hold_hp_tenths => (5, "tenths", 0, 100, "borg_escape.rs:689", "Sets morgoth hold hp tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_hold_glyph_count => (3, "count", 0, 1000, "borg_escape.rs:707", "Sets morgoth hold glyph count, measured in count, in the existing escape rule."),
    escape_risky_danger_extra_tenths => (3, "tenths", 0, 100, "borg_escape.rs:725", "Sets risky danger extra tenths, measured in tenths, in the existing escape rule."),
    escape_emergency_danger_tenths => (45, "tenths", 0, 100, "borg_escape.rs:733", "Sets emergency danger tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_danger_tenths => (40, "tenths", 0, 100, "borg_escape.rs:734", "Sets morgoth danger tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_max_hp => (600, "hp", 0, 10000, "borg_escape.rs:737", "Sets morgoth max hp, measured in hp, in the existing escape rule."),
    escape_sauron_danger_tenths => (30, "tenths", 0, 100, "borg_escape.rs:738", "Sets sauron danger tenths, measured in tenths, in the existing escape rule."),
    escape_sauron_max_hp => (600, "hp", 0, 10000, "borg_escape.rs:741", "Sets sauron max hp, measured in hp, in the existing escape rule."),
    escape_deep_unique_danger_tenths => (25, "tenths", 0, 100, "borg_escape.rs:742", "Sets deep unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_deep_unique_depth => (95, "depth", 0, 127, "borg_escape.rs:744", "Sets deep unique depth, measured in depth, in the existing escape rule."),
    escape_deep_unique_max_hp => (550, "hp", 0, 10000, "borg_escape.rs:745", "Sets deep unique max hp, measured in hp, in the existing escape rule."),
    escape_unique_danger_tenths => (17, "tenths", 0, 100, "borg_escape.rs:746", "Sets unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_ordinary_danger_tenths => (15, "tenths", 0, 100, "borg_escape.rs:747", "Sets ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_emergency_fail => (15, "percent", 0, 100, "borg_escape.rs:749", "Sets emergency fail, measured in percent, in the existing escape rule."),
    escape_initial_fail_discount => (10, "percent", 0, 100, "borg_escape.rs:753", "Sets initial fail discount, measured in percent, in the existing escape rule."),
    escape_emergency_fail_extra => (9, "percent", 0, 100, "borg_escape.rs:762", "Sets emergency fail extra, measured in percent, in the existing escape rule."),
    escape_emergency_phase_tolerance => (75, "count", 0, 1000, "borg_escape.rs:772", "Sets emergency phase tolerance, measured in count, in the existing escape rule."),
    escape_emergency_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:772", "Sets emergency phase horizon, measured in turns, in the existing escape rule."),
    escape_last_ditch_max_level => (10, "level", 1, 51, "borg_escape.rs:790", "Sets last ditch max level, measured in level, in the existing escape rule."),
    escape_last_ditch_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:791", "Sets last ditch hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_dimension_fail => (90, "percent", 0, 100, "borg_escape.rs:793", "Sets last ditch dimension fail, measured in percent, in the existing escape rule."),
    escape_critical_phase_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:805", "Sets critical phase hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_phase_max_level => (10, "level", 1, 51, "borg_escape.rs:818", "Sets last ditch phase max level, measured in level, in the existing escape rule."),
    escape_last_ditch_phase_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:819", "Sets last ditch phase hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_phase_fail => (15, "percent", 0, 100, "borg_escape.rs:820", "Sets last ditch phase fail, measured in percent, in the existing escape rule."),
    escape_endgame_stay_danger_tenths => (25, "tenths", 0, 100, "borg_escape.rs:834", "Sets endgame stay danger tenths, measured in tenths, in the existing escape rule."),
    escape_endgame_stay_unique_count => (3, "count", 1, 9, "borg_escape.rs:836", "Sets endgame stay unique count, measured in count, in the existing escape rule."),
    escape_endgame_stay_min_depth => (97, "depth", 0, 127, "borg_escape.rs:837", "Sets endgame stay min depth, measured in depth, in the existing escape rule."),
    escape_mage_danger_tenths => (3, "tenths", 0, 100, "borg_escape.rs:846", "Sets mage danger tenths, measured in tenths, in the existing escape rule."),
    escape_mage_max_sp => (20, "sp", 0, 10000, "borg_escape.rs:848", "Sets mage max sp, measured in sp, in the existing escape rule."),
    escape_mage_min_level => (45, "level", 1, 51, "borg_escape.rs:849", "Sets mage min level, measured in level, in the existing escape rule."),
    escape_teleport_unique_danger_tenths => (13, "tenths", 0, 100, "borg_escape.rs:850", "Sets teleport unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_teleport_ordinary_danger_tenths => (11, "tenths", 0, 100, "borg_escape.rs:851", "Sets teleport ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_teleport_phase_tolerance => (50, "count", 0, 1000, "borg_escape.rs:877", "Sets teleport phase tolerance, measured in count, in the existing escape rule."),
    escape_teleport_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:877", "Sets teleport phase horizon, measured in turns, in the existing escape rule."),
    escape_phase_antisummon_delay => (50, "turns", 0, 100000, "borg_escape.rs:878", "Sets phase antisummon delay, measured in turns, in the existing escape rule."),
    escape_phase_unique_danger_tenths => (13, "tenths", 0, 100, "borg_escape.rs:895", "Sets phase unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_phase_unique_min_count => (2, "count", 1, 9, "borg_escape.rs:895", "Sets phase unique min count, measured in count, in the existing escape rule."),
    escape_phase_ordinary_danger_tenths => (10, "tenths", 0, 100, "borg_escape.rs:896", "Sets phase ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_afraid_warrior_danger_tenths => (10, "tenths", 0, 100, "borg_escape.rs:897", "Sets afraid warrior danger tenths, measured in tenths, in the existing escape rule."),
    escape_phase_tolerance => (25, "count", 0, 1000, "borg_escape.rs:903", "Sets phase tolerance, measured in count, in the existing escape rule."),
    escape_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:903", "Sets phase horizon, measured in turns, in the existing escape rule."),
    escape_fallback_phase_tolerance => (75, "count", 0, 1000, "borg_escape.rs:938", "Sets fallback phase tolerance, measured in count, in the existing escape rule."),
    escape_fallback_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:938", "Sets fallback phase horizon, measured in turns, in the existing escape rule."),
    escape_failed_teleport_flee_max_level => (35, "level", 1, 51, "borg_escape.rs:968", "Sets failed teleport flee max level, measured in level, in the existing escape rule."),
    escape_failed_teleport_leave_max_level => (35, "level", 1, 51, "borg_escape.rs:980", "Sets failed teleport leave max level, measured in level, in the existing escape rule."),
    escape_compromised_danger_tenths => (8, "tenths", 0, 100, "borg_escape.rs:992", "Sets compromised danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_max_level => (35, "level", 1, 51, "borg_escape.rs:993", "Sets compromised max level, measured in level, in the existing escape rule."),
    escape_compromised_hp_divisor => (3, "divisor", 1, 1000, "borg_escape.rs:993", "Sets compromised hp divisor, measured in divisor, in the existing escape rule."),
    escape_compromised_unique_danger_tenths => (9, "tenths", 0, 100, "borg_escape.rs:994", "Sets compromised unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_unique_max_level => (35, "level", 1, 51, "borg_escape.rs:996", "Sets compromised unique max level, measured in level, in the existing escape rule."),
    escape_compromised_unique_hp_divisor => (3, "divisor", 1, 1000, "borg_escape.rs:996", "Sets compromised unique hp divisor, measured in divisor, in the existing escape rule."),
    escape_compromised_ordinary_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:997", "Sets compromised ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_ordinary_max_level => (20, "level", 1, 51, "borg_escape.rs:997", "Sets compromised ordinary max level, measured in level, in the existing escape rule."),
    escape_compromised_general_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:998", "Sets compromised general danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_general_max_level => (35, "level", 1, 51, "borg_escape.rs:998", "Sets compromised general max level, measured in level, in the existing escape rule."),
    escape_compromised_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1002", "Sets compromised phase tolerance, measured in count, in the existing escape rule."),
    escape_compromised_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1002", "Sets compromised phase horizon, measured in turns, in the existing escape rule."),
    escape_failed_compromised_flee_max_level => (25, "level", 1, 51, "borg_escape.rs:1041", "Sets failed compromised flee max level, measured in level, in the existing escape rule."),
    escape_weak_caster_max_level => (35, "level", 1, 51, "borg_escape.rs:1061", "Sets weak caster max level, measured in level, in the existing escape rule."),
    escape_weak_caster_phase_tolerance => (65, "count", 0, 1000, "borg_escape.rs:1062", "Sets weak caster phase tolerance, measured in count, in the existing escape rule."),
    escape_weak_caster_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1062", "Sets weak caster phase horizon, measured in turns, in the existing escape rule."),
    escape_low_level_max_level => (10, "level", 1, 51, "borg_escape.rs:1080", "Sets low level max level, measured in level, in the existing escape rule."),
    escape_low_level_danger_tenths => (5, "tenths", 0, 100, "borg_escape.rs:1081", "Sets low level danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_level_unique_danger_tenths => (7, "tenths", 0, 100, "borg_escape.rs:1081", "Sets low level unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_level_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1085", "Sets low level phase tolerance, measured in count, in the existing escape rule."),
    escape_low_level_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1085", "Sets low level phase horizon, measured in turns, in the existing escape rule."),
    escape_low_caster_max_level => (8, "level", 1, 51, "borg_escape.rs:1138", "Sets low caster max level, measured in level, in the existing escape rule."),
    escape_low_caster_phase_tolerance => (65, "count", 0, 1000, "borg_escape.rs:1139", "Sets low caster phase tolerance, measured in count, in the existing escape rule."),
    escape_low_caster_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1139", "Sets low caster phase horizon, measured in turns, in the existing escape rule."),
    escape_low_mana_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:1159", "Sets low mana danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_mana_unique_danger_tenths => (8, "tenths", 0, 100, "borg_escape.rs:1159", "Sets low mana unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_mana_sp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:1160", "Sets low mana sp divisor, measured in divisor, in the existing escape rule."),
    escape_low_mana_min_maxsp => (100, "sp", 0, 10000, "borg_escape.rs:1160", "Sets low mana min maxsp, measured in sp, in the existing escape rule."),
    escape_low_mana_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1164", "Sets low mana phase tolerance, measured in count, in the existing escape rule."),
    escape_low_mana_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1164", "Sets low mana phase horizon, measured in turns, in the existing escape rule."),
    escape_scoot_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1200", "Sets scoot tolerance, measured in count, in the existing escape rule."),
    escape_scoot_horizon => (2, "turns", 1, 10, "borg_escape.rs:1200", "Sets scoot horizon, measured in turns, in the existing escape rule."),
    escape_twitch_limit => (50, "count", 0, 1000, "borg_escape.rs:1215", "Sets twitch limit, measured in count, in the existing escape rule."),
    escape_twitch_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1218", "Sets twitch phase tolerance, measured in count, in the existing escape rule."),
    escape_twitch_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1218", "Sets twitch phase horizon, measured in turns, in the existing escape rule."),
    danger_default_hit_power => (0, "hit_power", 0, 1000, "borg_danger.rs:197", "Fallback hit-power estimate for blows without a specific legacy hit-power value."),
    danger_rsf_lash_damage => (100, "danger", 0, 5000, "borg_danger.rs:800", "Fixed danger estimate for a lash in reach."),
    danger_rsf_spit_damage => (100, "danger", 0, 5000, "borg_danger.rs:806", "Fixed danger estimate for spit in reach."),
    danger_pfe_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:212", "Divides melee damage plus fear when the legacy protection-from-evil approximation applies."),
    // borg-reborn-rs (#221): recovery source lines at 30ab9d66.
    recover_danger_horizon => (1, "turns", 1, 10, "borg_recover.rs:98", "Sets danger horizon, in turns, in the existing recovery rule."),
    recover_danger_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:101", "Sets danger divisor, in divisor, in the existing recovery rule."),
    recover_hurt_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:111", "Sets hurt hp divisor, in divisor, in the existing recovery rule."),
    recover_hurt_chance_bonus => (10, "percent", 0, 100, "borg_recover.rs:112", "Sets hurt chance bonus, in percent, in the existing recovery rule."),
    recover_critical_hp_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:116", "Sets critical hp divisor, in divisor, in the existing recovery rule."),
    recover_critical_chance_bonus => (10, "percent", 0, 100, "borg_recover.rs:117", "Sets critical chance bonus, in percent, in the existing recovery rule."),
    recover_cheap_stun_chance => (75, "percent", 0, 100, "borg_recover.rs:124", "Sets cheap stun chance, in percent, in the existing recovery rule."),
    recover_cheap_cut_chance => (75, "percent", 0, 100, "borg_recover.rs:160", "Sets cheap cut chance, in percent, in the existing recovery rule."),
    recover_cheap_poison_chance => (75, "percent", 0, 100, "borg_recover.rs:173", "Sets cheap poison chance, in percent, in the existing recovery rule."),
    recover_cheap_fear_chance => (75, "percent", 0, 100, "borg_recover.rs:186", "Sets cheap fear chance, in percent, in the existing recovery rule."),
    recover_cheap_hunger_chance => (75, "percent", 0, 100, "borg_recover.rs:199", "Sets cheap hunger chance, in percent, in the existing recovery rule."),
    recover_cheap_image_chance => (75, "percent", 0, 100, "borg_recover.rs:207", "Sets cheap image chance, in percent, in the existing recovery rule."),
    recover_cheap_heal_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:214", "Sets cheap heal hp divisor, in divisor, in the existing recovery rule."),
    recover_cheap_heal_chance => (75, "percent", 0, 100, "borg_recover.rs:215", "Sets cheap heal chance, in percent, in the existing recovery rule."),
    recover_cheap_heal_sp_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:217", "Sets cheap heal sp divisor, in divisor, in the existing recovery rule."),
    recover_unholy_min_hp => (90, "hp", 0, 10000, "borg_recover.rs:237", "Sets unholy min hp, in hp, in the existing recovery rule."),
    recover_costly_stun_chance => (25, "percent", 0, 100, "borg_recover.rs:266", "Sets costly stun chance, in percent, in the existing recovery rule."),
    recover_costly_heavy_stun_chance => (95, "percent", 0, 100, "borg_recover.rs:279", "Sets costly heavy stun chance, in percent, in the existing recovery rule."),
    recover_costly_cut_chance => (25, "percent", 0, 100, "borg_recover.rs:292", "Sets costly cut chance, in percent, in the existing recovery rule."),
    recover_critical_ccw_hp => (10, "hp", 0, 10000, "borg_recover.rs:298", "Sets critical ccw hp, in hp, in the existing recovery rule."),
    recover_costly_poison_chance => (25, "percent", 0, 100, "borg_recover.rs:305", "Sets costly poison chance, in percent, in the existing recovery rule."),
    recover_costly_blind_chance => (25, "percent", 0, 100, "borg_recover.rs:321", "Sets costly blind chance, in percent, in the existing recovery rule."),
    recover_costly_confusion_chance => (25, "percent", 0, 100, "borg_recover.rs:336", "Sets costly confusion chance, in percent, in the existing recovery rule."),
    recover_costly_fear_chance => (25, "percent", 0, 100, "borg_recover.rs:350", "Sets costly fear chance, in percent, in the existing recovery rule."),
    recover_costly_hunger_chance => (25, "percent", 0, 100, "borg_recover.rs:362", "Sets costly hunger chance, in percent, in the existing recovery rule."),
    recover_costly_heal_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:370", "Sets costly heal hp divisor, in divisor, in the existing recovery rule."),
    recover_costly_heal_chance => (25, "percent", 0, 100, "borg_recover.rs:371", "Sets costly heal chance, in percent, in the existing recovery rule."),
    recover_rest_high_depth => (85, "depth", 0, 127, "borg_recover.rs:418", "Sets rest high depth, in depth, in the existing recovery rule."),
    recover_rest_high_sp_tenths => (7, "tenths", 0, 10, "borg_recover.rs:418", "Sets rest high sp tenths, in tenths, in the existing recovery rule."),
    recover_rest_sp_tenths => (6, "tenths", 0, 10, "borg_recover.rs:418", "Sets rest sp tenths, in tenths, in the existing recovery rule."),
    recover_mana_rest_max_level => (40, "level", 1, 51, "borg_recover.rs:461", "Sets mana rest max level, in level, in the existing recovery rule."),
    recover_mana_rest_deep_depth => (85, "depth", 0, 127, "borg_recover.rs:461", "Sets mana rest deep depth, in depth, in the existing recovery rule."),
    recover_mana_rest_sp_tenths => (8, "tenths", 0, 10, "borg_recover.rs:462", "Sets mana rest sp tenths, in tenths, in the existing recovery rule."),
    recover_mana_rest_danger_divisor => (10, "divisor", 1, 1000, "borg_recover.rs:463", "Sets mana rest danger divisor, in divisor, in the existing recovery rule."),
    recover_mana_rest_min_food => (2, "count", 0, 1000, "borg_recover.rs:469", "Sets mana rest min food, in count, in the existing recovery rule."),
    recover_munchkin_rest_min_food => (2, "count", 0, 1000, "borg_recover.rs:492", "Sets munchkin rest min food, in count, in the existing recovery rule."),
    recover_rod_rest_turns => (100, "turns", 1, 9999, "borg_recover.rs:404", "Number of rest turns queued while waiting for a healing/recall rod to recharge."),
}

#[derive(Serialize)]
struct Setting<'a> {
    #[serde(flatten)]
    spec: &'a Spec,
    value: i32,
    origin: &'static str,
}

static VALUES: OnceLock<Vec<i32>> = OnceLock::new();

/// Pure lookup after init; isolated decision tests use the same defaults.
pub(crate) fn value(param: Param) -> i32 {
    let i = param as usize;
    VALUES.get().map_or(SPECS[i].default, |values| values[i])
}

fn parse<'a>(
    text: &str,
    env: &BTreeMap<String, String>,
    specs: &'a [Spec],
) -> Result<Vec<Setting<'a>>, String> {
    let mut file = BTreeMap::new();
    for (line, raw) in text.lines().enumerate() {
        let raw = raw.split('#').next().unwrap_or("").trim();
        if !raw.starts_with("borg_param_") {
            continue;
        }
        let (name, value) = raw
            .split_once('=')
            .ok_or_else(|| format!("borg.txt:{}: expected name = integer", line + 1))?;
        let name = name.trim().strip_prefix("borg_param_").unwrap();
        if !specs.iter().any(|s| s.name == name) {
            return Err(format!("unknown borg_param_{name}"));
        }
        if file.insert(name, value.trim()).is_some() {
            return Err(format!("duplicate borg_param_{name}"));
        }
    }
    for name in env.keys() {
        if !specs.iter().any(|s| s.name.to_ascii_uppercase() == *name) {
            return Err(format!("unknown BORG_PARAM_{name}"));
        }
    }
    specs
        .iter()
        .map(|spec| {
            let (raw, origin) = if let Some(raw) = env.get(&spec.name.to_ascii_uppercase()) {
                (Some(raw.as_str()), "environment")
            } else if let Some(raw) = file.get(spec.name) {
                (Some(*raw), "borg.txt")
            } else {
                (None, "default")
            };
            let value = match raw {
                Some(raw) => raw
                    .parse::<i32>()
                    .map_err(|_| format!("{}: expected integer, got {raw:?}", spec.name))?,
                None => spec.default,
            };
            if !(spec.min..=spec.max).contains(&value) {
                return Err(format!(
                    "{}: {value} outside {}..={} {}",
                    spec.name, spec.min, spec.max, spec.unit
                ));
            }
            Ok(Setting {
                spec,
                value,
                origin,
            })
        })
        .collect()
}

/// Called before initialization consumes RNG or makes decisions. Configuration
/// is immutable for the process, including reincarnations and core switches.
pub(crate) fn initialize() {
    VALUES.get_or_init(|| {
        let result = (|| -> Result<Vec<i32>, String> {
            let path = crate::engine::path_build_user(1024, b"borg.txt");
            let path = String::from_utf8_lossy(&path);
            let text = match std::fs::read_to_string(path.as_ref()) {
                Ok(text) => text,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(e) => return Err(format!("{path}: {e}")),
            };
            let mut env = BTreeMap::new();
            for (name, value) in std::env::vars_os() {
                if let Some(name) = name.to_str().and_then(|n| n.strip_prefix("BORG_PARAM_")) {
                    let value = value
                        .into_string()
                        .map_err(|_| format!("BORG_PARAM_{name}: invalid UTF-8"))?;
                    env.insert(name.to_string(), value);
                }
            }
            let settings = parse(&text, &env, SPECS)?;
            if let Some(dir) = std::env::var_os("BORG_RUN_DIR") {
                let path = std::path::PathBuf::from(dir).join("params-core.json");
                let mut json = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
                json.push(b'\n');
                std::fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            Ok(settings.iter().map(|s| s.value).collect())
        })();
        result.unwrap_or_else(|e| {
            eprintln!("borg-reborn-rs: core parameters: {e}");
            std::process::exit(2)
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_registry_are_valid() {
        let settings = parse("", &BTreeMap::new(), SPECS).unwrap();
        let mut names = std::collections::BTreeSet::new();
        for s in settings {
            assert!(names.insert(s.spec.name));
            assert_eq!(s.value, s.spec.default);
            assert_eq!(s.origin, "default");
            assert!(!s.spec.description.is_empty());
            let (file, line) = s.spec.source.split_once(':').unwrap();
            let module = s.spec.name.split('_').next().unwrap();
            assert_eq!(file, format!("borg_{module}.rs"));
            assert!(line.parse::<usize>().unwrap() > 0);
        }
    }

    #[test]
    fn rejects_bad_values_and_names() {
        for text in [
            "borg_param_caution_clw_min_loss = 101",
            "borg_param_caution_clw_min_loss = -1",
            "borg_param_caution_clw_min_loss = foo",
            "borg_param_caution_clw_min_loss = 2147483648",
            "borg_param_caution_clw_danger_divisor = 0",
            "borg_param_typo = 1",
            "borg_param_caution_clw_min_loss",
            "borg_param_caution_clw_min_loss = 1\nborg_param_caution_clw_min_loss = 2",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        assert!(parse("", &BTreeMap::from([("TYPO".into(), "1".into())]), SPECS).is_err());
    }

    #[test]
    fn danger_escape_configuration_bounds() {
        for text in [
            "borg_param_danger_fast_player_normal_speed = 190",
            "borg_param_danger_fast_player_threshold = 136",
            "borg_param_danger_summon_protected_divisor = 0",
            "borg_param_escape_landing_samples = 0",
            "borg_param_escape_landing_trials = 0",
            "borg_param_escape_phase_horizon = 0",
            "borg_param_escape_near_unique_max_count = 10",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        let env = BTreeMap::from([("DANGER_MAXIMUM".into(), "1".into())]);
        let settings = parse("borg_param_escape_phase_tolerance = 12", &env, SPECS).unwrap();
        let cap = &settings[Param::danger_maximum as usize];
        assert_eq!((cap.value, cap.origin), (1, "environment"));
        let phase = &settings[Param::escape_phase_tolerance as usize];
        assert_eq!((phase.value, phase.origin), (12, "borg.txt"));
    }

    #[test]
    fn recovery_configuration_bounds_and_precedence() {
        for text in [
            "borg_param_recover_danger_divisor = 0",
            "borg_param_recover_danger_horizon = 0",
            "borg_param_recover_rod_rest_turns = 0",
            "borg_param_recover_cheap_poison_chance = 101",
            "borg_param_recover_rest_sp_tenths = 11",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        let env = BTreeMap::from([("RECOVER_DANGER_DIVISOR".into(), "8".into())]);
        let values = parse("borg_param_recover_danger_divisor = 2", &env, SPECS).unwrap();
        let setting = &values[Param::recover_danger_divisor as usize];
        assert_eq!((setting.value, setting.origin), (8, "environment"));
    }

    #[test]
    fn environment_over_file_over_defaults_and_json_roundtrip() {
        let env = BTreeMap::from([("CAUTION_CLW_MIN_LOSS".into(), "20".into())]);
        let text = "# ignored\nborg_verbose = TRUE\nborg_param_caution_clw_min_loss = invalid\nborg_param_caution_csw_min_loss = 35 # comment";
        let settings = parse(text, &env, SPECS).unwrap();
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&settings).unwrap())
                .unwrap()
        );
        let clw = settings
            .iter()
            .find(|s| s.spec.name == "caution_clw_min_loss")
            .unwrap();
        assert_eq!((clw.value, clw.origin), (20, "environment"));
        let csw = settings
            .iter()
            .find(|s| s.spec.name == "caution_csw_min_loss")
            .unwrap();
        assert_eq!((csw.value, csw.origin), (35, "borg.txt"));
    }
}

//! borg-reborn-rs (#219): calibration of the vendored core.
//!
//! Source locations refer to borg_caution.rs at 726e04fa (before the hooks).
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
            assert!(s.spec.source.starts_with("borg_caution.rs:"));
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

//! Port of `borg-power.c`: the calculations to determine how powerful the
//! Borg thinks it is given the value arrays (`borg.trait`/`has`/`activation`).
//!
//! Copyright (c) 1997 Ben Harrison, James E. Wilson, Robert A. Koeneke
//! Copyright (c) 2007-9 Andi Sidwell, Chris Carr, Ed Graham, Erik Osheim
//!
//! This work is free software; you can redistribute it and/or modify it
//! under the terms of either:
//!
//! a) the GNU General Public License as published by the Free Software
//!    Foundation, version 2, or
//!
//! b) the "Angband License":
//!    This software may be copied and distributed for educational, research,
//!    and not for profit purposes provided that this copyright and statement
//!    are included in all such copies.  Other copyrights may also apply.
//!
//! Integer semantics: `value` is an `int32_t`. A C `value += x` with `x` a
//! `long` expression is done in i64 and truncated back (`add`/`sub`); with
//! `x` an `int` expression it is a wrapping i32 add (`addi`/`subi`).
//! Products are formed in the width C forms them (`int` products wrap,
//! `... * 100L` products are i64), and divisions are done after them in
//! that same width. The callers compare the results for ties (what-if
//! loops in wear/junk/store code), so the order of additions is kept too.

// borg-reborn-rs (#243): named power policy; default operations unchanged.
use crate::params::{value as param, Param};
use crate::borg_formulas::borg_power_dynamic;
use crate::borg_item::borg_item_weight;
use crate::borg_item_activation::ActivationIds;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_magic::{borg_get_spell_entry, borg_spell_stat};
use crate::borg_headers::borg_h::{
    BORG_MUNCHKIN_LEVEL, BORG_MUNCHKIN_START, BORG_USES_DYNAMIC_CALCS, BORG_WORSHIPS_AC,
    BORG_WORSHIPS_DAMAGE, BORG_WORSHIPS_HP, BORG_WORSHIPS_MANA, BORG_WORSHIPS_SPEED,
};
use crate::borg_headers::borg_trait_h::*;
use crate::headers::list_player_flags_h::PF_ZERO_FAIL;
use crate::headers::list_stats_h::{STAT_CON, STAT_DEX, STAT_INT, STAT_STR, STAT_WIS};
use crate::headers::list_tvals_h::{TV_ARROW, TV_DIGGING, TV_POTION, TV_SCROLL};
use crate::headers::player_h::STAT_MAX;
use crate::state::BorgState;
use crate::engine;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;
const CLASS_BLACKGUARD: i32 = 8;

/// `INVEN_WIELD + off` (`z_info->pack_size + off`): an equipment slot of
/// `borg_items`.
fn inven(off: usize) -> usize {
    usize::from(engine::z_info().pack_size) + off
}

const WIELD: usize = 0;
const BOW: usize = 1;
const BODY: usize = 6;
const OUTER: usize = 7;
const ARM: usize = 8;
const HEAD: usize = 9;
const HANDS: usize = 10;
const FEET: usize = 11;

/// `value += x` with `x` a C `long` expression.
fn add(value: i32, x: i64) -> i32 {
    (i64::from(value) + x) as i32
}

/// `value -= x` with `x` a C `long` expression.
fn sub(value: i32, x: i64) -> i32 {
    (i64::from(value) - x) as i32
}

/// `value += x` with `x` a C `int` expression.
fn addi(value: i32, x: i32) -> i32 {
    value.wrapping_add(x)
}

/// `value -= x` with `x` a C `int` expression.
fn subi(value: i32, x: i32) -> i32 {
    value.wrapping_sub(x)
}

/// `for (; k < lim && k < n; k++) value += each;` (`each` a C `long`
/// constant). `k` carries over, as C reuses it between loops.
fn count_up(value: &mut i32, k: &mut i32, lim: i32, n: i32, each: i64) {
    while *k < lim && *k < n {
        *value = add(*value, each);
        *k += 1;
    }
}

/// `for (; k < lim && k < n; k++) value -= each;`
fn count_down(value: &mut i32, k: &mut i32, lim: i32, n: i32, each: i64) {
    while *k < lim && *k < n {
        *value = sub(*value, each);
        *k += 1;
    }
}

/// The bonus of one activation, the `else if` chain of
/// `borg_power_equipment()` in C order (the first matching index wins; an
/// index no entry names gets 0).
fn activation_bonus(a: &ActivationIds, class: i32, clevel: i32, act: i32) -> i32 {
    let tele_other = if class == CLASS_MAGE { param(Param::power_activation_tele_other_mage_base) } else { param(Param::power_activation_tele_other_base) + param(Param::power_activation_tele_other_nonmage_bonus) };
    let rem_fear_pois = if class == CLASS_MAGE || class == CLASS_PRIEST || class == CLASS_DRUID {
        param(Param::power_activation_remove_fear_caster_base)
    } else {
        param(Param::power_activation_remove_fear_base) + param(Param::power_activation_remove_fear_noncaster_bonus)
    };
    let resist_all = (param(Param::power_activation_resist_all_base) + param(Param::power_activation_resist_all_extra)) + if class != CLASS_MAGE { param(Param::power_activation_resist_all_nonmage_bonus) } else { 0 };
    let sleepii = param(Param::power_activation_sleep_base)
        + if class != CLASS_DRUID && class != CLASS_NECROMANCER {
            param(Param::power_activation_sleep_class_bonus)
        } else {
            0
        };
    let recharge = param(Param::power_activation_recharge_base)
        + if class != CLASS_MAGE && class != CLASS_ROGUE {
            param(Param::power_activation_recharge_class_bonus)
        } else {
            0
        };
    let rage_bless_resist = (param(Param::power_activation_rage_bless_resist_base) + param(Param::power_activation_rage_bless_resist_extra)) + if class != CLASS_MAGE { param(Param::power_activation_rage_bless_resist_nonmage_bonus) } else { 0 };
    let dispel_evil = param(Param::power_activation_dispel_evil_base) + (10 + clevel.wrapping_mul(5) / 2);

    let chain: [(i32, i32); 160] = [
        (a.illumination, param(Param::power_activation_illumination_base)),
        (a.mapping, param(Param::power_activation_mapping_base)),
        (a.clairvoyance, param(Param::power_activation_clairvoyance_base)),
        (a.fire_bolt, param(Param::power_activation_fire_bolt_base) + (9 * (8 + 1) / 2)),
        (a.cold_bolt, param(Param::power_activation_cold_bolt_base) + (6 * (8 + 1) / 2)),
        (a.elec_bolt, param(Param::power_activation_elec_bolt_base) + (4 * (8 + 1) / 2)),
        (a.acid_bolt, param(Param::power_activation_acid_bolt_base) + (5 * (8 + 1) / 2)),
        (a.mana_bolt, param(Param::power_activation_mana_bolt_base) + (12 * (8 + 1) / 2)),
        (a.stinking_cloud, param(Param::power_activation_stinking_cloud_base) + 24),
        (a.cold_ball50, param(Param::power_activation_cold_ball50_base) + 96),
        (a.cold_ball100, param(Param::power_activation_cold_ball100_base) + 200),
        (a.fire_ball72, param(Param::power_activation_fire_ball72_base) + 72),
        (a.cold_bolt2, param(Param::power_activation_cold_bolt2_base) + (12 * (8 + 1) / 2)),
        (a.fire_ball, param(Param::power_activation_fire_ball_base) + 144),
        (a.dispel_evil, dispel_evil),
        (a.confuse2, param(Param::power_activation_confuse2_base)),
        (a.haste, param(Param::power_activation_haste_base)),
        (a.haste1, param(Param::power_activation_haste1_base)),
        (a.haste2, param(Param::power_activation_haste2_base)),
        (a.detect_objects, param(Param::power_activation_detect_objects_base)),
        (a.probing, param(Param::power_activation_probing_base)),
        (a.stone_to_mud, param(Param::power_activation_stone_to_mud_base)),
        (a.tele_other, tele_other),
        (a.drain_life1, param(Param::power_activation_drain_life1_base) + 90),
        (a.drain_life2, param(Param::power_activation_drain_life2_base) + 120),
        (a.berserker, param(Param::power_activation_berserker_base)),
        (a.cure_light, param(Param::power_activation_cure_light_base)),
        (a.cure_serious, param(Param::power_activation_cure_serious_base)),
        (a.cure_critical, param(Param::power_activation_cure_critical_base)),
        (a.cure_full2, param(Param::power_activation_cure_full2_base)),
        (a.loskill, param(Param::power_activation_loskill_base) + 200),
        (a.recall, param(Param::power_activation_recall_base)),
        (a.arrow, param(Param::power_activation_arrow_base) + 150),
        (a.rem_fear_pois, rem_fear_pois),
        (a.tele_phase, param(Param::power_activation_tele_phase_base)),
        (a.detect_all, param(Param::power_activation_detect_all_base)),
        (a.cure_full, param(Param::power_activation_cure_full_base)),
        (a.heal1, param(Param::power_activation_heal1_base)),
        (a.heal2, param(Param::power_activation_heal2_base)),
        (a.heal3, param(Param::power_activation_heal3_base)),
        (a.cure_nonorlybig, param(Param::power_activation_cure_nonorlybig_base)),
        (a.protevil, param(Param::power_activation_protevil_base)),
        (a.destroy_doors, param(Param::power_activation_destroy_doors_base)),
        (a.banishment, param(Param::power_activation_banishment_base)),
        (a.resist_all, resist_all),
        (a.sleepii, sleepii),
        (a.recharge, recharge),
        (a.tele_long, param(Param::power_activation_tele_long_base)),
        (a.missile, param(Param::power_activation_missile_base) + (2 * (6 + 1) / 2)),
        (a.cure_temp, param(Param::power_activation_cure_temp_base)),
        (a.starlight2, param(Param::power_activation_starlight2_base) + (10 * (8 + 1)) / 2),
        (a.bizarre, param(Param::power_activation_bizarre_base)),
        (a.star_ball, param(Param::power_activation_star_ball_base) + 300),
        (a.rage_bless_resist, rage_bless_resist),
        (a.polymorph, param(Param::power_activation_polymorph_base)),
        (a.starlight, param(Param::power_activation_starlight_base) + (6 * (8 + 1)) / 2),
        (a.light, param(Param::power_activation_light_base)),
        (a.firebrand, param(Param::power_activation_firebrand_base)),
        (a.restore_life, param(Param::power_activation_restore_life_base)),
        (a.restore_exp, param(Param::power_activation_restore_exp_base)),
        (a.restore_st_lev, param(Param::power_activation_restore_st_lev_base)),
        (a.enlightenment, param(Param::power_activation_enlightenment_base)),
        (a.hero, param(Param::power_activation_hero_base)),
        (a.shero, param(Param::power_activation_shero_base)),
        (a.cure_paranoia, param(Param::power_activation_cure_paranoia_base)),
        (a.cure_mind, param(Param::power_activation_cure_mind_base)),
        (a.cure_body, param(Param::power_activation_cure_body_base)),
        (a.mon_slow, param(Param::power_activation_mon_slow_base)),
        (a.mon_confuse, param(Param::power_activation_mon_confuse_base)),
        (a.sleep_all, param(Param::power_activation_sleep_all_base)),
        (a.mon_scare, param(Param::power_activation_mon_scare_base)),
        (a.light_line, param(Param::power_activation_light_line_base)),
        (a.disable_traps, param(Param::power_activation_disable_traps_base)),
        (a.drain_life3, param(Param::power_activation_drain_life3_base) + 150),
        (a.drain_life4, param(Param::power_activation_drain_life4_base) + 250),
        (a.elec_ball, param(Param::power_activation_elec_ball_base) + 64),
        (a.elec_ball2, param(Param::power_activation_elec_ball2_base) + 250),
        (a.acid_bolt2, param(Param::power_activation_acid_bolt2_base) + (10 * (8 + 1) / 2)),
        (a.acid_bolt3, param(Param::power_activation_acid_bolt3_base) + (12 * (8 + 1) / 2)),
        (a.acid_ball, param(Param::power_activation_acid_ball_base) + 120),
        (a.cold_ball160, param(Param::power_activation_cold_ball160_base) + 160),
        (a.cold_ball2, param(Param::power_activation_cold_ball2_base) + 200),
        (a.fire_ball2, param(Param::power_activation_fire_ball2_base) + 120),
        (a.fire_ball200, param(Param::power_activation_fire_ball200_base) + 200),
        (a.fire_bolt2, param(Param::power_activation_fire_bolt2_base) + (12 * (8 + 1) / 2)),
        (a.fire_bolt3, param(Param::power_activation_fire_bolt3_base) + (16 * (8 + 1) / 2)),
        (a.dispel_evil60, param(Param::power_activation_dispel_evil60_base) + 60),
        (a.dispel_undead, param(Param::power_activation_dispel_undead_base) + 60),
        (a.dispel_all, param(Param::power_activation_dispel_all_base) + 60 * 2),
        (a.deep_descent, param(Param::power_activation_deep_descent_base)),
        (a.earthquakes, param(Param::power_activation_earthquakes_base)),
        (a.destruction2, param(Param::power_activation_destruction2_base)),
        (a.losslow, param(Param::power_activation_losslow_base)),
        (a.lossleep, param(Param::power_activation_lossleep_base)),
        (a.losconf, param(Param::power_activation_losconf_base)),
        (a.satisfy, param(Param::power_activation_satisfy_base)),
        (a.blessing, param(Param::power_activation_blessing_base)),
        (a.blessing2, param(Param::power_activation_blessing2_base)),
        (a.blessing3, param(Param::power_activation_blessing3_base)),
        (a.glyph, param(Param::power_activation_glyph_base)),
        (a.tele_level, param(Param::power_activation_tele_level_base)),
        (a.confusing, param(Param::power_activation_confusing_base)),
        (a.enchant_tohit, param(Param::power_activation_enchant_tohit_base)),
        (a.enchant_todam, param(Param::power_activation_enchant_todam_base)),
        (a.enchant_weapon, param(Param::power_activation_enchant_weapon_base)),
        (a.enchant_armor, param(Param::power_activation_enchant_armor_base)),
        (a.enchant_armor2, param(Param::power_activation_enchant_armor2_base)),
        (a.remove_curse, param(Param::power_activation_remove_curse_base)),
        (a.remove_curse2, param(Param::power_activation_remove_curse2_base)),
        (a.detect_treasure, param(Param::power_activation_detect_treasure_base)),
        (a.detect_invis, param(Param::power_activation_detect_invis_base)),
        (a.detect_evil, param(Param::power_activation_detect_evil_base)),
        (a.restore_mana, param(Param::power_activation_restore_mana_base)),
        (a.brawn, param(Param::power_activation_brawn_base)),
        (a.intellect, param(Param::power_activation_intellect_base)),
        (a.contemplation, param(Param::power_activation_contemplation_base)),
        (a.toughness, param(Param::power_activation_toughness_base)),
        (a.nimbleness, param(Param::power_activation_nimbleness_base)),
        (a.restore_str, param(Param::power_activation_restore_str_base)),
        (a.restore_int, param(Param::power_activation_restore_int_base)),
        (a.restore_wis, param(Param::power_activation_restore_wis_base)),
        (a.restore_dex, param(Param::power_activation_restore_dex_base)),
        (a.restore_con, param(Param::power_activation_restore_con_base)),
        (a.restore_all, param(Param::power_activation_restore_all_base)),
        (a.tmd_free_act, param(Param::power_activation_tmd_free_act_base)),
        (a.tmd_infra, param(Param::power_activation_tmd_infra_base)),
        (a.tmd_sinvis, param(Param::power_activation_tmd_sinvis_base)),
        (a.tmd_esp, param(Param::power_activation_tmd_esp_base)),
        (a.resist_acid, param(Param::power_activation_resist_acid_base)),
        (a.resist_elec, param(Param::power_activation_resist_elec_base)),
        (a.resist_fire, param(Param::power_activation_resist_fire_base)),
        (a.resist_cold, param(Param::power_activation_resist_cold_base)),
        (a.resist_pois, param(Param::power_activation_resist_pois_base)),
        (a.cure_confusion, param(Param::power_activation_cure_confusion_base)),
        (a.wonder, param(Param::power_activation_wonder_base)),
        (a.wand_breath, param(Param::power_activation_wand_breath_base)),
        (a.staff_magi, param(Param::power_activation_staff_magi_base)),
        (a.staff_holy, param(Param::power_activation_staff_holy_base)),
        (a.drink_breath, param(Param::power_activation_drink_breath_base)),
        (a.food_waybread, param(Param::power_activation_food_waybread_base)),
        (a.shroom_emergency, param(Param::power_activation_shroom_emergency_base)),
        (a.shroom_terror, param(Param::power_activation_shroom_terror_base)),
        (a.shroom_stone, param(Param::power_activation_shroom_stone_base)),
        (a.shroom_debility, param(Param::power_activation_shroom_debility_base)),
        (a.shroom_sprinting, param(Param::power_activation_shroom_sprinting_base)),
        (a.shroom_purging, param(Param::power_activation_shroom_purging_base)),
        (a.ring_acid, param(Param::power_activation_ring_acid_base)),
        (a.ring_flames, param(Param::power_activation_ring_flames_base)),
        (a.ring_ice, param(Param::power_activation_ring_ice_base)),
        (a.ring_lightning, param(Param::power_activation_ring_lightning_base)),
        (a.dragon_blue, param(Param::power_activation_dragon_blue_base)),
        (a.dragon_green, param(Param::power_activation_dragon_green_base)),
        (a.dragon_red, param(Param::power_activation_dragon_red_base)),
        (a.dragon_multihued, param(Param::power_activation_dragon_multihued_base)),
        (a.dragon_gold, param(Param::power_activation_dragon_gold_base)),
        (a.dragon_chaos, param(Param::power_activation_dragon_chaos_base)),
        (a.dragon_law, param(Param::power_activation_dragon_law_base)),
        (a.dragon_balance, param(Param::power_activation_dragon_balance_base)),
        (a.dragon_shining, param(Param::power_activation_dragon_shining_base)),
        (a.dragon_power, param(Param::power_activation_dragon_power_base)),
    ];

    chain
        .iter()
        .find(|&&(id, _)| id == act)
        .map_or(0, |&(_, bonus)| bonus)
}

/// Helper function -- calculate "power" of equipment
fn borg_power_equipment(st: &BorgState) -> i32 {
    let tr = |i: u32| st.borg.tr(i);
    let cfg = |i: u32| st.cfg(i) != 0;
    let items = &st.items.items;

    let mut value: i32 = 0;

    // Obtain the "hold" value (weight limit for weapons)
    let hold = engine::adj_str_hold(tr(BI_STR_INDEX) as usize);

    // *** Analyze weapon ***

    // Examine current weapon
    let item = &items[inven(WIELD)];

    // We give a bonus to wearing an unID'D sword in order to use it and
    // garner a pseudoID from it.  We do not do this late in the game though
    // because our weapon often has traits that we need in order to be deep
    // (FA, SeeInvis)
    if tr(BI_CDEPTH) < param(Param::power_equipment_unidentified_weapon_max_depth) && tr(BI_MAXCLEVEL) < param(Param::power_equipment_unidentified_weapon_max_level) && item.iqty != 0 && !item.ident {
        value = addi(value, param(Param::power_equipment_unidentified_weapon_bonus));
    }

    // Calculate "average" damage per "normal" blow
    // and assume we can enchant up to +8 if borg.trait[BI_CLEVEL] > 25
    let dd_ds = i32::from(item.dd) * i32::from(item.ds);
    let damage = (i64::from(dd_ds) * i64::from(param(Param::power_equipment_melee_dice_weight))) as i32;

    // Reward "damage" and increased blows per round
    value = addi(value, damage.wrapping_mul(tr(BI_BLOWS) + param(Param::power_equipment_blows_weight)));

    // Reward "bonus to hit"
    value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * i64::from(param(Param::power_equipment_tohit_weight)));

    // Reward "bonus to dam"
    value = add(value, i64::from(tr(BI_TODAM) + i32::from(item.to_d)) * i64::from(param(Param::power_equipment_todam_weight)));

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * i64::from(param(Param::power_equipment_tohit_weight_2)));
    }

    // extra boost for deep dungeon
    if tr(BI_MAXDEPTH) >= param(Param::power_equipment_maxdepth_threshold) {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * i64::from(param(Param::power_equipment_tohit_weight_3)));

        value = add(value, i64::from(dd_ds) * i64::from(param(Param::power_equipment_blows_weight_2)) * i64::from(param(Param::power_equipment_blows_weight_3)) * i64::from(tr(BI_BLOWS)));
    }

    // assume 2x base damage for x% of creatures
    let dam = damage.wrapping_mul(2).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WS_ANIMAL) != 0 {
        value = addi(value, part(param(Param::power_equipment_ws_animal_weight)));
    }
    if tr(BI_WS_EVIL) != 0 {
        value = addi(value, part(param(Param::power_equipment_ws_evil_weight)));
    }

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // assume 3x base damage for x% of creatures
    let dam = damage.wrapping_mul(3).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WS_UNDEAD) != 0 && tr(BI_WK_UNDEAD) == 0 {
        value = addi(value, part(param(Param::power_equipment_wk_undead_weight)));
    }
    if tr(BI_WS_DEMON) != 0 && tr(BI_WK_DEMON) == 0 {
        value = addi(value, part(param(Param::power_equipment_wk_demon_weight)));
    }
    if tr(BI_WS_DRAGON) != 0 && tr(BI_WK_DRAGON) == 0 {
        value = addi(value, part(param(Param::power_equipment_wk_dragon_weight)));
    }
    if tr(BI_WS_GIANT) != 0 {
        value = addi(value, part(param(Param::power_equipment_ws_giant_weight)));
    }
    if tr(BI_WB_ACID) != 0 {
        value = addi(value, part(param(Param::power_equipment_wb_acid_weight)));
    }
    if tr(BI_WB_ELEC) != 0 {
        value = addi(value, part(param(Param::power_equipment_wb_elec_weight)));
    }
    if tr(BI_WB_FIRE) != 0 {
        value = addi(value, part(param(Param::power_equipment_wb_fire_weight)));
    }
    if tr(BI_WB_COLD) != 0 {
        value = addi(value, part(param(Param::power_equipment_wb_cold_weight)));
    }
    // SOrc and STroll get 1/2 of reward now
    if tr(BI_WS_ORC) != 0 {
        value = addi(value, part(param(Param::power_equipment_ws_orc_weight)));
    }
    if tr(BI_WS_TROLL) != 0 {
        value = addi(value, part(param(Param::power_equipment_ws_troll_weight)));
    }
    // and the other 2/2 if SEvil not possessed
    if tr(BI_WS_ORC) != 0 && tr(BI_WS_EVIL) == 0 {
        value = addi(value, part(param(Param::power_equipment_ws_evil_weight_2)));
    }
    if tr(BI_WS_TROLL) != 0 && tr(BI_WS_EVIL) == 0 {
        value = addi(value, part(param(Param::power_equipment_ws_evil_weight_3)));
    }

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // assume 5x base damage for x% of creatures
    let dam = damage.wrapping_mul(5).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WK_UNDEAD) != 0 {
        value = addi(value, part(param(Param::power_equipment_wk_undead_weight_2)));
    }
    if tr(BI_WK_DEMON) != 0 {
        value = addi(value, part(param(Param::power_equipment_wk_demon_weight_2)));
    }
    if tr(BI_WK_DRAGON) != 0 {
        value = addi(value, part(param(Param::power_equipment_wk_dragon_weight_2)));
    }
    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // It used to be only on Grond
    if tr(BI_W_IMPACT) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_w_impact_weight)));
    }

    // It is hard to hold a heavy weapon
    if tr(BI_HEAVYWEPON) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_heavywepon_weight)));
    }

    // We want low level borgs to have high blows (dagger, whips)
    if tr(BI_CLEVEL) <= param(Param::power_equipment_clevel_threshold) {
        value = add(value, i64::from(tr(BI_BLOWS)) * i64::from(param(Param::power_equipment_blows_weight_4)));
    }

    // *** Analyze bow ***

    // Examine current bow
    let item = &items[inven(BOW)];

    // We give a bonus to wearing an unID'D bow in order to use it and
    // garner a pseudoID from it.  We do not do this late in the game though
    // because our weapon often has traits that we need in order to be deep
    // (FA, SeeInvis)
    if tr(BI_CDEPTH) < param(Param::power_equipment_unidentified_bow_max_depth) && tr(BI_MAXCLEVEL) < param(Param::power_equipment_unidentified_bow_max_level) && item.iqty != 0 && !item.ident {
        value = addi(value, param(Param::power_equipment_unidentified_bow_bonus));
    }

    // Calculate "average" damage per "normal" shot (times 2)
    let damage = if i32::from(item.to_d) > param(Param::power_equipment_bow_enchant_threshold) || tr(BI_CLEVEL) < param(Param::power_equipment_bow_enchant_min_level) {
        (tr(BI_AMMO_SIDES) + i32::from(item.to_d)).wrapping_mul(tr(BI_AMMO_POWER))
    } else {
        (tr(BI_AMMO_SIDES) + param(Param::power_equipment_bow_enchant_floor)).wrapping_mul(tr(BI_AMMO_POWER))
    };

    // Reward "damage"
    let shots_damage = tr(BI_SHOTS).wrapping_mul(damage);
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(shots_damage) * i64::from(param(Param::power_equipment_reward_damage_weight)));
    } else {
        value = add(value, i64::from(shots_damage) * i64::from(param(Param::power_equipment_reward_damage_weight_2)));
    }

    // Extra bonus for low levels, they need a ranged weapon
    if tr(BI_CLEVEL) < param(Param::power_equipment_clevel_threshold_2) {
        value = add(value, i64::from(shots_damage) * i64::from(param(Param::power_equipment_clevel_weight)));
    }

    // slings force you to carry heavy ammo.  Penalty for that unless you
    // have lots of str
    if i32::from(item.sval) == st.item_val.sv_sling && item.art_idx == 0 && tr(BI_STR) < param(Param::power_equipment_str_threshold) {
        value = sub(value, i64::from(param(Param::power_equipment_str_weight)));
    }

    // Bonus if level 1 to buy a sling, they are cheap ranged weapons
    if i32::from(item.sval) == st.item_val.sv_sling && tr(BI_CLEVEL) == param(Param::power_equipment_clevel_threshold_3) && tr(BI_STR) >= param(Param::power_equipment_str_threshold_2) {
        value = addi(value, param(Param::power_equipment_str_weight_2));
    }

    // Reward "bonus to hit"
    value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * i64::from(param(Param::power_equipment_tohit_weight_4)));

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * i64::from(param(Param::power_equipment_tohit_weight_5)));
    }

    // Prefer bows
    if tr(BI_FAST_SHOTS) != 0 && tr(BI_AMMO_TVAL) == TV_ARROW as i32 {
        value = add(value, i64::from(param(Param::power_equipment_ammo_tval_weight)));
    }

    // It is hard to hold a heavy weapon
    if hold < i32::from(item.weight) / 10 {
        value = sub(value, i64::from(param(Param::power_equipment_it_is_hard_to_hold_a_heavy_weapon_weight)));
    }

    // *** Reward various things ***

    // Reward light radius
    // necromancers like the dark
    if tr(BI_CLASS) == CLASS_NECROMANCER {
        value = sub(value, i64::from(tr(BI_LIGHT) - 1) * i64::from(param(Param::power_equipment_light_penalty)));
    } else if tr(BI_LIGHT) <= param(Param::power_equipment_light_threshold) {
        value = add(value, i64::from(tr(BI_LIGHT)) * i64::from(param(Param::power_equipment_light_weight)));
    } else if tr(BI_LIGHT) > param(Param::power_equipment_light_threshold_2) {
        value = add(value, i64::from(param(Param::power_equipment_light_weight_2)) + i64::from(tr(BI_LIGHT).wrapping_mul(param(Param::power_equipment_light_weight_3))));
    }

    value = add(value, i64::from(tr(BI_MOD_MOVES)) * i64::from(param(Param::power_equipment_mod_moves_weight)));
    value = add(value, i64::from(tr(BI_DAM_RED)) * i64::from(param(Param::power_equipment_dam_red_weight)));

    // Reward speed
    // see if speed can be a bonus if good speed; not +3.
    // reward higher for +10 than +50 speed (decreased return).
    let speed = tr(BI_SPEED);
    // `((speed - base) * per) + extra`, all `long`
    let tier = |base: i32, per: i64, extra: i64| i64::from(speed - base) * per + extra;
    if cfg(BORG_WORSHIPS_SPEED) {
        if speed >= param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base), i64::from(param(Param::power_equipment_speed_tier_slope)), i64::from(param(Param::power_equipment_speed_tier_offset))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_2)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_3)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_2), i64::from(param(Param::power_equipment_speed_tier_slope_2)), i64::from(param(Param::power_equipment_speed_tier_offset_2))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_4)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_5)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_3), i64::from(param(Param::power_equipment_speed_tier_slope_3)), i64::from(param(Param::power_equipment_speed_tier_offset_3))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_6)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_7)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_4), i64::from(param(Param::power_equipment_speed_tier_slope_4)), i64::from(param(Param::power_equipment_speed_tier_offset_4))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_8)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_9)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_5), i64::from(param(Param::power_equipment_speed_tier_slope_5)), i64::from(param(Param::power_equipment_speed_tier_offset_5))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_10)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_11)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_6), i64::from(param(Param::power_equipment_speed_tier_slope_6)), i64::from(param(Param::power_equipment_speed_tier_offset_6))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_12)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_13)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_7), i64::from(param(Param::power_equipment_speed_tier_slope_7)), i64::from(param(Param::power_equipment_speed_tier_offset_7))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_14)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_15)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_8), i64::from(param(Param::power_equipment_speed_tier_slope_8)), i64::from(param(Param::power_equipment_speed_tier_offset_8))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_16)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_17)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_9), i64::from(param(Param::power_equipment_speed_tier_slope_9)), i64::from(param(Param::power_equipment_speed_tier_offset_9))));
        }
        if speed < param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_18) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_10), i64::from(param(Param::power_equipment_speed_tier_slope_10)), i64::from(param(Param::power_equipment_speed_tier_offset_10))));
        }
    } else {
        if speed >= param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_19) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_11), i64::from(param(Param::power_equipment_speed_tier_slope_11)), i64::from(param(Param::power_equipment_speed_tier_offset_11))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_20)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_21)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_12), i64::from(param(Param::power_equipment_speed_tier_slope_12)), i64::from(param(Param::power_equipment_speed_tier_offset_12))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_22)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_23)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_13), i64::from(param(Param::power_equipment_speed_tier_slope_13)), i64::from(param(Param::power_equipment_speed_tier_offset_13))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_24)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_25)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_14), i64::from(param(Param::power_equipment_speed_tier_slope_14)), i64::from(param(Param::power_equipment_speed_tier_offset_14))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_26)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_27)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_15), i64::from(param(Param::power_equipment_speed_tier_slope_15)), i64::from(param(Param::power_equipment_speed_tier_offset_15))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_28)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_29)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_16), i64::from(param(Param::power_equipment_speed_tier_slope_16)), i64::from(param(Param::power_equipment_speed_tier_offset_16))));
        }
        if (param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_30)..=param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_31)).contains(&speed) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_17), i64::from(param(Param::power_equipment_speed_tier_slope_17)), i64::from(param(Param::power_equipment_speed_tier_offset_17))));
        }
        if speed < param(Param::power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_32) {
            value = add(value, tier(param(Param::power_equipment_speed_tier_base_18), i64::from(param(Param::power_equipment_speed_tier_slope_18)), i64::from(param(Param::power_equipment_speed_tier_offset_18))));
        }
    }

    // Reward strength bonus
    value = add(value, i64::from(tr(BI_STR_INDEX)) * i64::from(param(Param::power_equipment_str_index_weight)));

    // Reward spell stat bonus
    let spell_stat = borg_spell_stat();
    if spell_stat >= 0 {
        value = add(value, i64::from(tr(BI_STR_INDEX + spell_stat as u32)) * i64::from(param(Param::power_equipment_str_index_weight_2)));

        // Bonus for sp.
        if cfg(BORG_WORSHIPS_MANA) {
            value = add(value, i64::from(tr(BI_SP_ADJ) / param(Param::power_equipment_sp_warrior_divisor)) * i64::from(param(Param::power_equipment_sp_warrior_weight)));
        } else {
            value = add(value, i64::from(tr(BI_SP_ADJ) / param(Param::power_equipment_sp_caster_divisor)) * i64::from(param(Param::power_equipment_sp_caster_weight)));
        }

        // bonus for low fail rate
        value = addi(
            value,
            (100 - i32::from(engine::spell_chance(0))).wrapping_mul(param(Param::power_equipment_spell_reliability_weight)),
        );

        // should try to get min fail to 0
        if engine::player_has(PF_ZERO_FAIL) {
            // other fail rates
            if i32::from(engine::spell_chance(0)) < param(Param::power_equipment_other_fail_rates_threshold) {
                value = add(value, i64::from(param(Param::power_equipment_other_fail_rates_weight)));
            }
        }
    }

    // Dexterity Bonus --good for attacking and ac
    if tr(BI_DEX_INDEX) <= 37 {
        // Reward bonus
        value = add(value, i64::from(tr(BI_DEX_INDEX)) * i64::from(param(Param::power_equipment_dex_index_weight)));
    }

    // Constitution Bonus
    if tr(BI_CON_INDEX) <= 37 {
        if cfg(BORG_WORSHIPS_HP) {
            value = add(value, i64::from(tr(BI_CON_INDEX)) * i64::from(param(Param::power_equipment_con_index_weight)));
            // Reward hp bonus
            // This is a bit weird because we are not really giving
            // a bonus for what hp you have, but the 'bonus'
            // hp you get getting over 800hp is very important.
            if tr(BI_HP_ADJ) < param(Param::power_equipment_hp_adj_threshold) {
                value = add(value, i64::from(tr(BI_HP_ADJ)) * i64::from(param(Param::power_equipment_hp_adj_weight)));
            } else {
                value = add(value, i64::from(tr(BI_HP_ADJ) - param(Param::power_equipment_hp_adj_weight_2)) * i64::from(param(Param::power_equipment_hp_adj_weight_3)) + i64::from(param(Param::power_equipment_hp_adj_weight_4)) * i64::from(param(Param::power_equipment_hp_adj_weight_5)));
            }
        } else {
            // does not worship hp
            value = add(value, i64::from(tr(BI_CON_INDEX)) * i64::from(param(Param::power_equipment_con_index_weight_2)));
            // Reward hp bonus
            // This is a bit weird because we are not really giving
            // a bonus for what hp you have, but the 'bonus'
            // hp you get getting over 500hp is very important.
            if tr(BI_HP_ADJ) < param(Param::power_equipment_hp_adj_threshold_2) {
                value = add(value, i64::from(tr(BI_HP_ADJ)) * i64::from(param(Param::power_equipment_hp_adj_weight_6)));
            } else {
                value = add(value, i64::from(tr(BI_HP_ADJ) - param(Param::power_equipment_hp_adj_weight_7)) * i64::from(param(Param::power_equipment_hp_adj_weight_8)) + i64::from(param(Param::power_equipment_hp_adj_weight_9)) * i64::from(param(Param::power_equipment_hp_adj_weight_10)));
            }
        }
    }

    // HACK - a small bonus for adding to stats even above max.
    //        This will allow us to swap a ring of int +6 for
    //        our ring of int +2 even though we are at max int because
    //        we are wielding a weapon that has +4 int
    //        later it might be nice to swap to a weapon that does not
    //        have an int bonus
    for i in 0..STAT_MAX {
        value = addi(value, tr(BI_ASTR + i));
    }

    // Tiny rewards
    value = add(value, i64::from(tr(BI_DISP)) * i64::from(param(Param::power_equipment_disp_weight)));
    value = add(value, i64::from(tr(BI_DISM)) * i64::from(param(Param::power_equipment_dism_weight)));
    value = add(value, i64::from(tr(BI_DEV)) * i64::from(param(Param::power_equipment_dev_weight)));
    value = add(value, i64::from(tr(BI_SAV)) * i64::from(param(Param::power_equipment_sav_weight)));
    // perfect saves are very nice
    if tr(BI_SAV) > param(Param::power_equipment_sav_threshold) {
        value = addi(value, param(Param::power_equipment_sav_weight_2));
    }
    value = add(value, i64::from(tr(BI_STL)) * i64::from(param(Param::power_equipment_stl_weight)));
    value = add(value, i64::from(tr(BI_SRCH)));
    value = add(value, i64::from(tr(BI_THN)) * i64::from(param(Param::power_equipment_thn_weight)));
    value = add(value, i64::from(tr(BI_THB)) * i64::from(param(Param::power_equipment_thb_weight)));
    value = add(value, i64::from(tr(BI_THT)) * i64::from(param(Param::power_equipment_tht_weight)));
    value = add(value, i64::from(tr(BI_DIG)) * i64::from(param(Param::power_equipment_dig_weight)));

    // *** Reward current flags ***

    // Various flags
    if tr(BI_SDIG) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sdig_weight)));
    }
    if tr(BI_SDIG) != 0 && tr(BI_ISHUNGRY) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_ishungry_weight)));
    }
    if tr(BI_SDIG) != 0 && tr(BI_ISWEAK) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_isweak_weight)));
    }

    // Feather Fall if low level is nice
    if tr(BI_MAXDEPTH) < param(Param::power_equipment_maxdepth_threshold_2) {
        if tr(BI_FEATH) != 0 {
            value = add(value, i64::from(param(Param::power_equipment_feath_weight)));
        }
    } else if tr(BI_FEATH) != 0 {
        value = addi(value, param(Param::power_equipment_feath_weight_2));
    }

    if tr(BI_LIGHT) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_light_weight_4)));
    }

    if tr(BI_ESP) != 0 && tr(BI_SINV) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sinv_weight)));
    }

    if tr(BI_DINV) == 0 && tr(BI_SINV) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sinv_weight_2)));
    }

    if tr(BI_FRACT) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_fract_weight)));
    }

    // after you max out you are pretty safe from drainers.
    if tr(BI_MAXCLEVEL) < 50 {
        if tr(BI_HLIFE) != 0 {
            value = add(value, i64::from(param(Param::power_equipment_hlife_weight)));
        }
    } else if tr(BI_HLIFE) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_hlife_weight_2)));
    }
    if tr(BI_REG) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_reg_weight)));
    }
    if tr(BI_ESP) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_esp_weight)));
    }

    // Immunity flags
    if tr(BI_ICOLD) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_icold_weight)));
    }
    if tr(BI_IELEC) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_ielec_weight)));
    }
    if tr(BI_IFIRE) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_ifire_weight)));
    }
    if tr(BI_IACID) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_iacid_weight)));
    }

    // Resistance flags
    if tr(BI_RCOLD) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rcold_weight)));
    }
    if tr(BI_RELEC) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_relec_weight)));
    }
    if tr(BI_RACID) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_racid_weight)));
    }
    if tr(BI_RFIRE) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rfire_weight)));
    }
    // extra bonus for getting all basic resist
    if tr(BI_RFIRE) != 0 && tr(BI_RACID) != 0 && tr(BI_RELEC) != 0 && tr(BI_RCOLD) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rcold_weight_2)));
    }
    if tr(BI_RPOIS) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rpois_weight)));
    }
    if tr(BI_RSND) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rsnd_weight)));
    }
    if tr(BI_RLITE) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rlite_weight)));
    }
    if tr(BI_RDARK) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rdark_weight)));
    }
    if tr(BI_RKAOS) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rkaos_weight)));
    }

    // this is way boosted to avoid carrying stuff you don't need
    if tr(BI_RCONF) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rconf_weight)));
    }

    // mages need a slight boost for this
    if tr(BI_CLASS) == CLASS_MAGE && tr(BI_RCONF) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rconf_weight_2)));
    }
    if tr(BI_RDIS) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rdis_weight)));
    }
    if tr(BI_RSHRD) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rshrd_weight)));
    }
    if tr(BI_RNXUS) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rnxus_weight)));
    }
    if tr(BI_RBLIND) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rblind_weight)));
    }
    if tr(BI_RNTHR) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rnthr_weight)));
    }
    if tr(BI_RFEAR) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_rfear_weight)));
    }

    // Sustain flags
    if tr(BI_SSTR) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sstr_weight)));
    }
    if tr(BI_SINT) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sint_weight)));
    }
    if tr(BI_SWIS) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_swis_weight)));
    }
    if tr(BI_SCON) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_scon_weight)));
    }
    if tr(BI_SDEX) != 0 {
        value = add(value, i64::from(param(Param::power_equipment_sdex_weight)));
    }
    // boost for getting them all
    if tr(BI_SSTR) != 0
        && tr(BI_SINT) != 0
        && tr(BI_SWIS) != 0
        && tr(BI_SDEX) != 0
        && tr(BI_SCON) != 0
    {
        value = add(value, i64::from(param(Param::power_equipment_sstr_weight_2)));
    }

    // *** XXX XXX XXX Reward "necessary" flags ***

    // `borg.trait[BI_MAXDEPTH] + 1 >= d`
    let deep = |d: i32| tr(BI_MAXDEPTH) + 1 >= d;

    // Mega-Hack -- See invisible (level 10)
    if (tr(BI_SINV) != 0 || tr(BI_ESP) != 0) && deep(param(Param::power_equipment_esp_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_esp_weight_2)));
    }

    // Mega-Hack -- Free action (level 20)
    if tr(BI_FRACT) != 0 && deep(param(Param::power_equipment_fract_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_fract_weight_2)));
    }

    //  Mega-Hack -- resists (level 25)
    if tr(BI_RFIRE) != 0 && deep(param(Param::power_equipment_rfire_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rfire_weight_2)));
    }

    //  Mega-Hack -- resists (level 40)
    if tr(BI_RPOIS) != 0 && deep(param(Param::power_equipment_rpois_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rpois_weight_2)));
    }
    if tr(BI_RELEC) != 0 && deep(param(Param::power_equipment_relec_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_relec_weight_2)));
    }
    if tr(BI_RACID) != 0 && deep(param(Param::power_equipment_racid_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_racid_weight_2)));
    }
    if tr(BI_RCOLD) != 0 && deep(param(Param::power_equipment_rcold_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rcold_weight_3)));
    }

    //  Mega-Hack -- Speed / Hold Life (level 46) and maxed out
    if tr(BI_HLIFE) != 0 && deep(param(Param::power_equipment_hold_life_depth)) && tr(BI_MAXCLEVEL) < 50 {
        value = add(value, i64::from(param(Param::power_equipment_maxclevel_weight)));
    }
    if tr(BI_SPEED) >= param(Param::power_equipment_speed_threshold) && deep(param(Param::power_equipment_speed_threshold_2)) {
        value = add(value, i64::from(param(Param::power_equipment_speed_weight)));
    }
    if tr(BI_RCONF) != 0 && deep(param(Param::power_equipment_rconf_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rconf_weight_3)));
    }

    //  Mega-Hack -- resist Nether is -very- nice to have at level 50
    if tr(BI_RNTHR) != 0 && deep(param(Param::power_equipment_rnthr_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rnthr_weight_2)));
    }

    //  Mega-Hack -- resist Sound to avoid being KO'd
    if tr(BI_RSND) != 0 && deep(param(Param::power_equipment_rsnd_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rsnd_weight_2)));
    }

    //  Mega-Hack -- resists & Telepathy (level 55)
    if tr(BI_RBLIND) != 0 && deep(param(Param::power_equipment_rblind_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rblind_weight_2)));
    }
    if tr(BI_ESP) != 0 && deep(param(Param::power_equipment_esp_threshold_2)) {
        value = add(value, i64::from(param(Param::power_equipment_esp_weight_3)));
    }
    if tr(BI_RNTHR) != 0 && deep(param(Param::power_equipment_rnthr_threshold_2)) {
        value = add(value, i64::from(param(Param::power_equipment_rnthr_weight_3)));
    }

    //  Mega-Hack -- resists & +10 speed (level 60)
    if tr(BI_RKAOS) != 0 && deep(param(Param::power_equipment_rkaos_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rkaos_weight_2)));
    }
    if tr(BI_RDIS) != 0 && deep(param(Param::power_equipment_rdis_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rdis_weight_2)));
    }
    if tr(BI_SPEED) >= param(Param::power_equipment_speed_threshold_3) && deep(param(Param::power_equipment_speed_threshold_4)) {
        value = add(value, i64::from(param(Param::power_equipment_speed_weight_2)));
    }

    //  Must have +20 speed (level 80)
    if tr(BI_SPEED) >= param(Param::power_equipment_speed_threshold_5) && deep(param(Param::power_equipment_speed_threshold_6)) {
        value = add(value, i64::from(param(Param::power_equipment_speed_weight_3)));
    }

    // Not Req, but a good idea:
    // Extra boost to Nether deeper down
    // RDark for deeper uniques
    // Good to have +30 speed
    if tr(BI_RNTHR) != 0 && deep(param(Param::power_equipment_rnthr_threshold_3)) {
        value = add(value, i64::from(param(Param::power_equipment_rnthr_weight_4)));
    }
    if tr(BI_RDARK) != 0 && deep(param(Param::power_equipment_rdark_threshold)) {
        value = add(value, i64::from(param(Param::power_equipment_rdark_weight_2)));
    }
    if tr(BI_SPEED) >= param(Param::power_equipment_speed_threshold_7) && deep(param(Param::power_equipment_speed_threshold_8)) && tr(BI_CLASS) == CLASS_WARRIOR {
        value = add(value, i64::from(param(Param::power_equipment_class_weight)));
    }

    // *** Reward powerful armor ***

    // Reward armor
    let armor = tr(BI_ARMOR);
    if cfg(BORG_WORSHIPS_AC) {
        if armor < param(Param::power_equipment_reward_armor_threshold) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight)));
        }
        if (param(Param::power_equipment_reward_armor_threshold_2)..param(Param::power_equipment_reward_armor_threshold_3)).contains(&armor) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight_2)) + i64::from(param(Param::power_equipment_reward_armor_weight_3)));
        }
        if armor >= param(Param::power_equipment_reward_armor_threshold_4) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight_4)) + i64::from(param(Param::power_equipment_reward_armor_weight_5)));
        }
    } else {
        if armor < param(Param::power_equipment_reward_armor_threshold_5) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight_6)));
        }
        if (param(Param::power_equipment_reward_armor_threshold_6)..param(Param::power_equipment_reward_armor_threshold_7)).contains(&armor) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight_7)) + i64::from(param(Param::power_equipment_reward_armor_weight_8)));
        }
        if armor >= param(Param::power_equipment_reward_armor_threshold_8) {
            value = add(value, i64::from(armor) * i64::from(param(Param::power_equipment_reward_armor_weight_9)) + i64::from(param(Param::power_equipment_reward_armor_weight_10)));
        }
    }

    // *** Penalize various things ***

    // Penalize various flags
    if tr(BI_CRSAGRV) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsagrv_weight)));
    }
    if tr(BI_CRSHPIMP) != 0 {
        value = subi(value, param(Param::power_equipment_crshpimp_weight));
    }
    if tr(BI_CLASS) != CLASS_WARRIOR && tr(BI_CRSMPIMP) != 0 {
        value = subi(value, param(Param::power_equipment_crsmpimp_weight));
    }
    if (tr(BI_CLASS) == CLASS_MAGE
        || tr(BI_CLASS) == CLASS_PRIEST
        || tr(BI_CLASS) == CLASS_DRUID
        || tr(BI_CLASS) == CLASS_NECROMANCER)
        && tr(BI_CRSMPIMP) != 0
    {
        value = subi(value, param(Param::power_equipment_class_weight_2));
    }
    if tr(BI_CRSFEAR) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsfear_weight)));
    }
    if tr(BI_CRSFEAR) != 0 && tr(BI_CLASS) != CLASS_MAGE {
        value = sub(value, i64::from(param(Param::power_equipment_class_weight_3)));
    }
    if tr(BI_CRSDRAIN_XP) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsdrain_xp_weight)));
    }
    if tr(BI_CRSFVULN) != 0 {
        value = subi(value, param(Param::power_equipment_crsfvuln_weight));
    }
    if tr(BI_CRSEVULN) != 0 {
        value = subi(value, param(Param::power_equipment_crsevuln_weight));
    }
    if tr(BI_CRSCVULN) != 0 {
        value = subi(value, param(Param::power_equipment_crscvuln_weight));
    }
    if tr(BI_CRSAVULN) != 0 {
        value = subi(value, param(Param::power_equipment_crsavuln_weight));
    }

    if tr(BI_CRSTELE) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crstele_weight)));
    }
    if tr(BI_CRSENVELOPING) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsenveloping_weight)));
    }
    if tr(BI_CRSIRRITATION) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsirritation_weight)));
    }
    if tr(BI_CRSPOIS) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crspois_weight)));
    }
    if tr(BI_CRSSIREN) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crssiren_weight)));
    }
    if tr(BI_CRSHALU) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crshalu_weight)));
    }
    if tr(BI_CRSPARA) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crspara_weight)));
    }
    if tr(BI_CRSSDEM) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crssdem_weight)));
    }
    if tr(BI_CRSSDRA) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crssdra_weight)));
    }
    if tr(BI_CRSSUND) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crssund_weight)));
    }
    if tr(BI_CRSSTONE) != 0 && tr(BI_SPEED) < param(Param::power_equipment_speed_threshold_9) {
        value = sub(value, i64::from(param(Param::power_equipment_speed_weight_4)));
    }
    if tr(BI_CRSSTEELSKIN) != 0 && tr(BI_SPEED) < param(Param::power_equipment_speed_threshold_10) {
        value = sub(value, i64::from(param(Param::power_equipment_speed_weight_5)));
    }
    if tr(BI_CRSNOTEL) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsnotel_weight)));
    }
    if tr(BI_CRSTWEP) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crstwep_weight)));
    }
    if tr(BI_CRSAIRSWING) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsairswing_weight)));
    }
    if tr(BI_CRSUNKNO) != 0 {
        value = sub(value, i64::from(param(Param::power_equipment_crsunkno_weight)));
    }

    // (C has a disabled `#if 0` Morgoth block here.)

    // bonus for something with multiple useful bonuses
    value = add(value, i64::from(param(Param::power_equipment_multiple_bonuses_weight)) * i64::from(tr(BI_MULTIPLE_BONUSES)));

    // bonus for wearing something that needs an ID
    value = add(value, i64::from(param(Param::power_equipment_worn_need_id_weight)) * i64::from(tr(BI_WORN_NEED_ID)));

    let act_max = i32::from(engine::z_info().act_max);
    for act in 1..act_max {
        let n = st.borg.activation[act as usize];
        if n == 0 {
            continue;
        }
        // an extra bonus for activations; the bonus is per item that has
        // this activation
        let bonus = activation_bonus(&st.act, tr(BI_CLASS), tr(BI_CLEVEL), act);
        value = addi(value, bonus.wrapping_mul(n));
    }

    // Hack-- Reward the borg for carrying a NON-ID items that have random
    // powers
    if items[inven(OUTER)].iqty != 0 {
        let item = &items[inven(OUTER)];
        if borg_ego_has_random_power(usize::from(item.ego_idx)) && !item.ident {
            value = add(value, i64::from(param(Param::power_equipment_powers_weight)));
        }
    }

    // `borg_item_weight(&borg_items[slot])`
    let weight = |off: usize| i32::from(borg_item_weight(&items[inven(off)]));

    // *** Penalize armor weight ***
    if tr(BI_STR_INDEX) < param(Param::power_equipment_str_index_threshold) {
        if weight(BODY) > param(Param::power_equipment_powers_threshold) {
            value = subi(value, (weight(BODY) - param(Param::power_equipment_str_index_weight_3)).wrapping_mul(param(Param::power_equipment_str_index_weight_4)));
        }
        if weight(HEAD) > param(Param::power_equipment_powers_threshold_2) {
            value = subi(value, param(Param::power_equipment_str_index_weight_5));
        }
        if weight(ARM) > param(Param::power_equipment_powers_threshold_3) {
            value = subi(value, param(Param::power_equipment_str_index_weight_6));
        }
        if weight(FEET) > param(Param::power_equipment_powers_threshold_4) {
            value = subi(value, param(Param::power_equipment_str_index_weight_7));
        }
    }

    // Compute the total armor weight
    let mut cur_wgt: i32 = 0;
    cur_wgt += weight(BODY);
    cur_wgt += weight(HEAD);
    cur_wgt += weight(ARM);
    cur_wgt += weight(OUTER);
    cur_wgt += weight(HANDS);
    cur_wgt += weight(FEET);

    // Determine the weight allowance
    let max_wgt = engine::class_spell_weight();

    // Heavy armor hurts magic
    if engine::class_total_spells() != 0 && (cur_wgt - max_wgt) / 10 > 0 {
        let over = (cur_wgt - max_wgt) / 10;
        // max sp must be calculated in case it changed with the armor
        let mut max_sp = tr(BI_SP_ADJ) / 100 + 1;
        max_sp -= over;
        // Mega-Hack -- Penalize heavy armor which hurts mana
        if (param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold)..=param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_2)).contains(&max_sp) {
            value = sub(value, i64::from(over) * i64::from(param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_weight)));
        }
        if (param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_3)..=param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_4)).contains(&max_sp) {
            value = sub(value, i64::from(over) * i64::from(param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_2)));
        }
        if (param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_5)..=param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_6)).contains(&max_sp) {
            value = sub(value, i64::from(over) * i64::from(param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_3)));
        }
        if max_sp <= param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_7) {
            value = sub(value, i64::from(over) * i64::from(param(Param::power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_4)));
        }
    }

    // Result
    value
}

/// Helper function -- calculate power of inventory
fn borg_power_inventory(st: &BorgState) -> i32 {
    let tr = |i: u32| st.borg.tr(i);
    let has = |kv: i32| st.borg.has[kv as usize];
    let cfg = |i: u32| st.cfg(i);
    let iv = &st.item_val;

    // `borg_cfg[BORG_MUNCHKIN_START] && borg.trait[BI_MAXCLEVEL] <
    // borg_cfg[BORG_MUNCHKIN_LEVEL]`: "Do not carry these until later"
    let munchkin_wait =
        || cfg(BORG_MUNCHKIN_START) != 0 && tr(BI_MAXCLEVEL) < cfg(BORG_MUNCHKIN_LEVEL);

    let mut value: i32 = 0;
    let v = &mut value;

    // *** Basic abilities ***

    // Reward fuel
    let mut k = 0;
    count_up(v, &mut k, param(Param::power_inventory_afuel_limit), tr(BI_AFUEL), i64::from(param(Param::power_inventory_afuel_weight)));
    if tr(BI_STR) >= param(Param::power_inventory_str_threshold) {
        while k < param(Param::power_inventory_reward_fuel_threshold) && k < tr(BI_AFUEL) {
            *v = add(*v, 6000 - i64::from(k * 100));
            k += 1;
        }
    }

    // Reward Food
    // if hungry, food is THE top priority
    if (tr(BI_ISHUNGRY) != 0 || tr(BI_ISWEAK) != 0) && tr(BI_FOOD) != 0 {
        *v = addi(*v, 100000);
    }
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_food_limit), tr(BI_FOOD), i64::from(param(Param::power_inventory_food_weight)));
    if tr(BI_STR) >= param(Param::power_inventory_str_threshold_2) {
        count_up(v, &mut k, param(Param::power_inventory_food_limit_2), tr(BI_FOOD), i64::from(param(Param::power_inventory_food_weight_2)));
    }
    if tr(BI_REG) != 0 && tr(BI_CLEVEL) <= param(Param::power_inventory_clevel_threshold) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_food_limit_3), tr(BI_FOOD), i64::from(param(Param::power_inventory_food_weight_3)));
    }

    // Prefer to buy HiCalorie foods over LowCalorie
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_food_hi_limit), tr(BI_FOOD_HI), i64::from(param(Param::power_inventory_food_hi_weight)));
    k = 0;
    count_down(v, &mut k, param(Param::power_inventory_food_lo_limit), tr(BI_FOOD_LO), i64::from(param(Param::power_inventory_food_lo_weight)));

    // Reward Cure Poison and Cuts
    let hurt = tr(BI_ISCUT) != 0 || tr(BI_ISPOISONED) != 0;
    if hurt && tr(BI_ACCW) != 0 {
        *v = addi(*v, 100000);
    }
    if hurt && tr(BI_AHEAL) != 0 {
        *v = addi(*v, 50000);
    }
    if hurt && tr(BI_ACSW) != 0 {
        // usually takes more than one
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_acsw_limit), tr(BI_ACSW), i64::from(param(Param::power_inventory_acsw_weight)));
    }
    if tr(BI_ISPOISONED) != 0 && tr(BI_ACUREPOIS) != 0 {
        *v = addi(*v, 15000);
    }

    // collect Resistance pots if not immune -- All Classes
    if tr(BI_IPOIS) == 0 && tr(BI_ACUREPOIS) <= param(Param::power_inventory_acurepois_threshold) && !munchkin_wait() {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_arespois_limit), tr(BI_ARESPOIS), i64::from(param(Param::power_inventory_arespois_weight)));
    }

    // Reward Resistance Potions for Warriors
    if tr(BI_CLASS) == CLASS_WARRIOR && tr(BI_MAXDEPTH) > param(Param::power_inventory_maxdepth_threshold) {
        k = 0;
        if !munchkin_wait() {
            // collect pots if not immune
            if tr(BI_IFIRE) == 0 {
                count_up(v, &mut k, param(Param::power_inventory_aresheat_limit), tr(BI_ARESHEAT), i64::from(param(Param::power_inventory_aresheat_weight)));
            }
            k = 0;
            // collect pots if not immune
            if tr(BI_ICOLD) == 0 {
                count_up(v, &mut k, param(Param::power_inventory_arescold_limit), tr(BI_ARESCOLD), i64::from(param(Param::power_inventory_arescold_weight)));
            }
            k = 0;
            // collect pots if not immune
            if tr(BI_IPOIS) == 0 {
                count_up(v, &mut k, param(Param::power_inventory_arespois_limit_2), tr(BI_ARESPOIS), i64::from(param(Param::power_inventory_arespois_weight_2)));
            }
        }
    }

    // Reward ident
    k = 0;
    if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_2) {
        count_up(v, &mut k, param(Param::power_inventory_aid_limit), tr(BI_AID), i64::from(param(Param::power_inventory_aid_weight)));
        if tr(BI_STR) >= param(Param::power_inventory_str_threshold_3) {
            count_up(v, &mut k, param(Param::power_inventory_aid_limit_2), tr(BI_AID), i64::from(param(Param::power_inventory_aid_weight_2)));
        }
    }
    // Reward ID if I am carrying a {magical} or {excellent} item
    if tr(BI_ALL_NEED_ID) != 0 {
        k = 0;
        while k < tr(BI_ALL_NEED_ID) && k < tr(BI_AID) {
            *v = add(*v, 6000);
            k += 1;
        }
    }

    //  Reward PFE carry lots of these
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, param(Param::power_inventory_apfe_limit), tr(BI_APFE), i64::from(param(Param::power_inventory_apfe_weight)));
        count_up(v, &mut k, param(Param::power_inventory_apfe_limit_2), tr(BI_APFE), i64::from(param(Param::power_inventory_apfe_weight_2)));
    }
    //  Reward Glyph- Rune of Protection-  carry lots of these
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_aglyph_limit), tr(BI_AGLYPH), i64::from(param(Param::power_inventory_aglyph_weight)));
    count_up(v, &mut k, param(Param::power_inventory_aglyph_limit_2), tr(BI_AGLYPH), i64::from(param(Param::power_inventory_aglyph_weight_2)));
    if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_2) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aglyph_limit_3), tr(BI_AGLYPH), i64::from(param(Param::power_inventory_aglyph_weight_3)));
        count_up(v, &mut k, param(Param::power_inventory_aglyph_limit_4), tr(BI_AGLYPH), i64::from(param(Param::power_inventory_aglyph_weight_4)));
    }

    // Reward Scroll of Mass Genocide, only when fighting Morgoth
    if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_3) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_amassban_limit), tr(BI_AMASSBAN), i64::from(param(Param::power_inventory_amassban_weight)));
    }

    // Reward recall
    k = 0;
    if tr(BI_CLEVEL) > param(Param::power_inventory_clevel_threshold_3) && !munchkin_wait() {
        count_up(v, &mut k, param(Param::power_inventory_recall_limit), tr(BI_RECALL), i64::from(param(Param::power_inventory_recall_weight)));
        if tr(BI_STR) >= param(Param::power_inventory_str_threshold_4) {
            count_up(v, &mut k, param(Param::power_inventory_recall_limit_2), tr(BI_RECALL), i64::from(param(Param::power_inventory_recall_weight_2)));
        }
        // Deep borgs need the rod to avoid low mana traps
        if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_4) && has(iv.kv_rod_recall) != 0 {
            *v = addi(*v, 12000);
        }
    }

    // Reward phase
    k = 1;
    // first phase door is very important
    if tr(BI_APHASE) != 0 {
        *v = addi(*v, 50000);
    }
    if !munchkin_wait() {
        count_up(v, &mut k, param(Param::power_inventory_aphase_limit), tr(BI_APHASE), i64::from(param(Param::power_inventory_aphase_weight)));
        if tr(BI_STR) >= param(Param::power_inventory_str_threshold_5) {
            count_up(v, &mut k, param(Param::power_inventory_aphase_limit_2), tr(BI_APHASE), i64::from(param(Param::power_inventory_aphase_weight_2)));
        }
    }

    // Reward escape (staff of teleport or artifact)
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, param(Param::power_inventory_aescape_limit), tr(BI_AESCAPE), i64::from(param(Param::power_inventory_aescape_weight)));
        if tr(BI_MAXDEPTH) > param(Param::power_inventory_maxdepth_threshold_5) {
            k = 0;
            count_up(v, &mut k, param(Param::power_inventory_aescape_limit_2), tr(BI_AESCAPE), i64::from(param(Param::power_inventory_aescape_weight_2)));
        }
    }

    // Reward teleport scroll
    k = 0;
    if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_4) && tr(BI_ATELEPORT) != 0 {
        *v = add(*v, 10000);
    }
    if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_5) {
        count_up(v, &mut k, param(Param::power_inventory_ateleport_limit), tr(BI_ATELEPORT), i64::from(param(Param::power_inventory_ateleport_weight)));
    }
    if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_6) {
        count_up(v, &mut k, param(Param::power_inventory_ateleport_limit_2), tr(BI_ATELEPORT), i64::from(param(Param::power_inventory_ateleport_weight_2)));
    }

    // Reward Teleport Level scrolls
    k = 0;
    if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_7) {
        count_up(v, &mut k, param(Param::power_inventory_ateleportlvl_limit), tr(BI_ATELEPORTLVL), i64::from(param(Param::power_inventory_ateleportlvl_weight)));
    }

    // carry a couple of *Healing* for emergency, store the rest
    let ezheal = |v: &mut i32| {
        let mut k = 0;
        if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_6) {
            if tr(BI_PREP_BIG_FIGHT) != 0 {
                count_up(v, &mut k, param(Param::power_inventory_aezheal_limit), tr(BI_AEZHEAL), i64::from(param(Param::power_inventory_aezheal_weight)));
            } else {
                count_up(v, &mut k, param(Param::power_inventory_aezheal_limit_2), tr(BI_AEZHEAL), i64::from(param(Param::power_inventory_aezheal_weight_2)));
            }
        }
    };

    // *** Healing ***
    // !TODO !FIX make sure these numbers make sense for the new classes
    let class = tr(BI_CLASS);
    if class == CLASS_WARRIOR || class == CLASS_ROGUE || class == CLASS_BLACKGUARD {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aheal_limit), tr(BI_AHEAL), i64::from(param(Param::power_inventory_aheal_weight)));

        ezheal(v);

        // These guys need to carry the rods more, town runs low on supply.
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_rod_healing_limit), has(iv.kv_rod_healing), i64::from(param(Param::power_inventory_rod_healing_weight)));
    } else if class == CLASS_RANGER
        || class == CLASS_PALADIN
        || class == CLASS_NECROMANCER
        || class == CLASS_MAGE
    {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aheal_limit_2), tr(BI_AHEAL), i64::from(param(Param::power_inventory_aheal_weight_2)));

        ezheal(v);

        if tr(BI_CLASS) == CLASS_PALADIN {
            // Reward heal potions
            k = 0;
            count_up(v, &mut k, param(Param::power_inventory_potion_healing_limit), has(iv.kv_potion_healing), i64::from(param(Param::power_inventory_potion_healing_weight)));
        }

        // These guys need to carry the rods more, town runs low on supply.
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_rod_healing_limit_2), has(iv.kv_rod_healing), i64::from(param(Param::power_inventory_rod_healing_weight_2)));
    } else if class == CLASS_PRIEST || class == CLASS_DRUID {
        // Level 1 priests are given a Potion of Healing.  It is better
        // for them to sell that potion and buy equipment or several
        // Cure Crits with it.
        if tr(BI_CLEVEL) == param(Param::power_inventory_clevel_threshold_8) {
            k = 0;
            count_down(v, &mut k, param(Param::power_inventory_potion_healing_limit_2), has(iv.kv_potion_healing), i64::from(param(Param::power_inventory_potion_healing_weight_2)));
        }
        // Reward heal potions
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_potion_healing_limit_3), has(iv.kv_potion_healing), i64::from(param(Param::power_inventory_potion_healing_weight_3)));

        ezheal(v);
    }

    // Collecting Potions, prepping for Morgoth/Sauron fight
    // only carry them if we aren't prepping, otherwise they store at
    // home
    if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_7) && tr(BI_PREP_BIG_FIGHT) == 0 {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_potion_healing_limit_4), has(iv.kv_potion_healing), i64::from(param(Param::power_inventory_potion_healing_weight_4)));
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aezheal_limit_3), tr(BI_AEZHEAL), i64::from(param(Param::power_inventory_aezheal_weight_3)));
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aspeed_limit), tr(BI_ASPEED), i64::from(param(Param::power_inventory_aspeed_weight)));
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_alife_limit), tr(BI_ALIFE), i64::from(param(Param::power_inventory_alife_weight)));
        k = 0;
        if tr(BI_CLASS) != CLASS_WARRIOR {
            count_up(v, &mut k, param(Param::power_inventory_potion_restore_mana_limit), has(iv.kv_potion_restore_mana), i64::from(param(Param::power_inventory_potion_restore_mana_weight)));
        }
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_mush_stoneskin_limit), has(iv.kv_mush_stoneskin), i64::from(param(Param::power_inventory_mush_stoneskin_weight)));

        // Sauron is dead -- store them unless I have enough
        if tr(BI_SAURON_DEAD) != 0 {
            // Reward Scroll of Mass Genocide, only when fighting Morgoth
            k = 0;
            count_up(v, &mut k, param(Param::power_inventory_amassban_limit_2), tr(BI_AMASSBAN), i64::from(param(Param::power_inventory_amassban_weight_2)));
        }
    }

    // Restore Mana
    if tr(BI_MAXSP) > param(Param::power_inventory_maxsp_threshold) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_potion_restore_mana_limit_2), has(iv.kv_potion_restore_mana), i64::from(param(Param::power_inventory_potion_restore_mana_weight_2)));
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_astfmagi_limit), tr(BI_ASTFMAGI), i64::from(param(Param::power_inventory_astfmagi_weight)));
    }

    // Reward Cure Critical.  Heavy reward on first 5
    if tr(BI_CLEVEL) < param(Param::power_inventory_clevel_threshold_9) && tr(BI_CLEVEL) > param(Param::power_inventory_clevel_threshold_10) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_accw_limit), tr(BI_ACCW), i64::from(param(Param::power_inventory_accw_weight)));
    } else if tr(BI_CLEVEL) >= param(Param::power_inventory_clevel_threshold_11) {
        // Reward cure critical.  Later on in game.
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_accw_limit_2), tr(BI_ACCW), i64::from(param(Param::power_inventory_accw_weight_2)));
        if tr(BI_STR) > param(Param::power_inventory_str_threshold_6) {
            count_up(v, &mut k, param(Param::power_inventory_accw_limit_3), tr(BI_ACCW), i64::from(param(Param::power_inventory_accw_weight_3)));
        }
    }

    // Reward cure serious -- only reward serious if low on crits
    if tr(BI_ACCW) < param(Param::power_inventory_accw_threshold) && tr(BI_MAXCLEVEL) > param(Param::power_inventory_maxclevel_threshold) && (tr(BI_CLEVEL) < param(Param::power_inventory_clevel_threshold_12) || tr(BI_RCONF) == 0) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_acsw_limit_2), tr(BI_ACSW), i64::from(param(Param::power_inventory_acsw_weight_2)));
        if tr(BI_STR) > param(Param::power_inventory_str_threshold_7) {
            count_up(v, &mut k, param(Param::power_inventory_acsw_limit_3), tr(BI_ACSW), i64::from(param(Param::power_inventory_acsw_weight_3)));
        }
    }

    // Reward cure light -- Low Level Characters
    if tr(BI_ACCW) + tr(BI_ACSW) < param(Param::power_inventory_acsw_threshold) && tr(BI_CLEVEL) < param(Param::power_inventory_clevel_threshold_13) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_aclw_limit), tr(BI_ACLW), i64::from(param(Param::power_inventory_aclw_weight)));
    }

    // Reward Cures
    if tr(BI_RCONF) == 0 {
        if cfg(BORG_MUNCHKIN_START) != 0 && tr(BI_MAXCLEVEL) < param(Param::power_inventory_maxclevel_threshold_2) {
            // Do not carry these until later
        } else {
            k = 0;
            count_up(v, &mut k, param(Param::power_inventory_food_cure_conf_limit), tr(BI_FOOD_CURE_CONF), i64::from(param(Param::power_inventory_food_cure_conf_weight)));
        }
    }
    if tr(BI_RBLIND) == 0 {
        k = 0;
        if !munchkin_wait() {
            count_up(v, &mut k, param(Param::power_inventory_food_cure_blind_limit), tr(BI_FOOD_CURE_BLIND), i64::from(param(Param::power_inventory_food_cure_blind_weight)));
        }
    }
    if tr(BI_RPOIS) == 0 {
        k = 0;
        if !munchkin_wait() {
            count_up(v, &mut k, param(Param::power_inventory_acurepois_limit), tr(BI_ACUREPOIS), i64::from(param(Param::power_inventory_acurepois_weight)));
        }
    }

    // *** Detection ***

    // Reward detect trap
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_adettrap_limit), tr(BI_ADETTRAP), i64::from(param(Param::power_inventory_adettrap_weight)));

    // Reward detect door
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_adetdoor_limit), tr(BI_ADETDOOR), i64::from(param(Param::power_inventory_adetdoor_weight)));

    // Reward detect evil
    if tr(BI_ESP) == 0 {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_adetevil_limit), tr(BI_ADETEVIL), i64::from(param(Param::power_inventory_adetevil_weight)));
    }

    // Reward magic mapping
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_amagicmap_limit), tr(BI_AMAGICMAP), i64::from(param(Param::power_inventory_amagicmap_weight)));

    // Reward call lite
    if tr(BI_CLASS) != CLASS_NECROMANCER {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_alite_limit), tr(BI_ALITE), i64::from(param(Param::power_inventory_alite_weight)));
    }

    // Genocide scrolls. Just scrolls, mainly used for Morgoth
    if tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_8) {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_scroll_mass_banishment_limit), has(iv.kv_scroll_mass_banishment), i64::from(param(Param::power_inventory_scroll_mass_banishment_weight)));
        count_up(v, &mut k, param(Param::power_inventory_scroll_mass_banishment_limit_2), has(iv.kv_scroll_mass_banishment), i64::from(param(Param::power_inventory_scroll_mass_banishment_weight_2)));
    }

    // Reward speed potions/rods/staves (but no staves deeper than depth 95)
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, param(Param::power_inventory_aspeed_limit_2), tr(BI_ASPEED), i64::from(param(Param::power_inventory_aspeed_weight_2)));
    }

    // Reward Recharge ability
    if tr(BI_ARECHARGE) != 0 && tr(BI_MAXDEPTH) < param(Param::power_inventory_maxdepth_threshold_9) {
        *v = add(*v, 5000);
    }

    // *** Missiles ***

    // Reward missiles
    if tr(BI_CLASS) == CLASS_RANGER || tr(BI_CLASS) == CLASS_WARRIOR {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_amissiles_limit), tr(BI_AMISSILES), i64::from(param(Param::power_inventory_amissiles_weight)));
        if tr(BI_STR) > param(Param::power_inventory_str_threshold_8) && tr(BI_STR) <= param(Param::power_inventory_str_threshold_9) {
            count_up(v, &mut k, param(Param::power_inventory_amissiles_limit_2), tr(BI_AMISSILES), i64::from(param(Param::power_inventory_amissiles_weight_2)));
        }
        if tr(BI_STR) > param(Param::power_inventory_str_threshold_10) {
            count_up(v, &mut k, param(Param::power_inventory_amissiles_limit_3), tr(BI_AMISSILES), i64::from(param(Param::power_inventory_amissiles_weight_3)));
        }

        // penalize use of too many quiver slots
        k = param(Param::power_inventory_penalize_use_of_too_many_quiver_slots_threshold);
        while k < tr(BI_QUIVER_SLOTS) {
            *v = sub(*v, 10000);
            k += 1;
        }
    } else {
        k = 0;
        count_up(v, &mut k, param(Param::power_inventory_amissiles_limit_4), tr(BI_AMISSILES), i64::from(param(Param::power_inventory_amissiles_weight_4)));
        if tr(BI_STR) > param(Param::power_inventory_str_threshold_11) {
            count_up(v, &mut k, param(Param::power_inventory_amissiles_limit_5), tr(BI_AMISSILES), i64::from(param(Param::power_inventory_amissiles_weight_5)));
        }
        // Don't carry too many
        if tr(BI_STR) <= param(Param::power_inventory_str_threshold_12) && tr(BI_AMISSILES) > param(Param::power_inventory_amissiles_threshold) {
            *v = sub(*v, 1000);
        }
        // penalize use of too many quiver slots
        k = param(Param::power_inventory_penalize_use_of_too_many_quiver_slots_threshold_2);
        while k < tr(BI_QUIVER_SLOTS) {
            *v = sub(*v, 10000);
            k += 1;
        }
    }

    // cursed arrows are "bad"
    *v = sub(*v, 1000 * i64::from(tr(BI_AMISSILES_CURSED)));

    // ego arrows are worth a bonus
    *v = add(*v, 100 * i64::from(tr(BI_AMISSILES_SPECIAL)));

    // *** Various ***

    //  -- Reward carrying a staff of destruction.
    if tr(BI_ASTFDEST) != 0 {
        *v = add(*v, 5000);
    }
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_astfdest_limit), tr(BI_ASTFDEST), i64::from(param(Param::power_inventory_astfdest_weight)));

    //  -- Reward carrying a wand of Teleport Other.
    if tr(BI_ATPORTOTHER) != 0 {
        *v = add(*v, 5000);
    }
    // Much greater reward for Warrior
    if tr(BI_CLASS) == CLASS_WARRIOR && tr(BI_ATPORTOTHER) != 0 {
        *v = add(*v, 50000);
    }
    // reward per charge
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_atportother_limit), tr(BI_ATPORTOTHER), i64::from(param(Param::power_inventory_atportother_weight)));

    //  -- Reward carrying an attack wand.
    if (has(iv.kv_wand_magic_missile) != 0 || has(iv.kv_wand_stinking_cloud) != 0)
        && tr(BI_MAXDEPTH) < param(Param::power_inventory_maxdepth_threshold_10)
    {
        *v = add(*v, 5000);
    }
    if has(iv.kv_wand_annihilation) != 0 && tr(BI_CDEPTH) < param(Param::power_inventory_cdepth_threshold) {
        *v = add(*v, 5000);
    }
    // Much greater reward for Warrior or lower level
    if (tr(BI_CLASS) == CLASS_WARRIOR || tr(BI_CLEVEL) <= param(Param::power_inventory_clevel_threshold_14))
        && (has(iv.kv_wand_magic_missile) != 0
            || has(iv.kv_wand_annihilation) != 0
            || has(iv.kv_wand_stinking_cloud) != 0)
    {
        *v = add(*v, 10000);
    }
    // reward per charge
    *v = add(*v, i64::from(tr(BI_GOOD_W_CHG)) * 50);

    // These staves are great but do not clutter inven with them
    //  -- Reward carrying a staff of holiness/power
    if tr(BI_GOOD_S_CHG) != 0 {
        *v = add(*v, 2500);
    }
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_good_s_chg_limit), tr(BI_GOOD_S_CHG), i64::from(param(Param::power_inventory_good_s_chg_weight)));

    // Rods of attacking are good too
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_arod1_limit), tr(BI_AROD1), i64::from(param(Param::power_inventory_arod1_weight)));
    k = 0;
    count_up(v, &mut k, param(Param::power_inventory_arod2_limit), tr(BI_AROD2), i64::from(param(Param::power_inventory_arod2_weight)));

    // Reward being at max stat
    let need = &st.borg.need_statgain;
    if !need[STAT_STR as usize] {
        *v = addi(*v, 50000);
    }
    if !need[STAT_INT as usize] {
        *v = addi(*v, 20000);
    }
    if !need[STAT_WIS as usize] {
        *v = addi(*v, 20000);
    }
    if !need[STAT_DEX as usize] {
        *v = addi(*v, 50000);
    }
    if !need[STAT_CON as usize] {
        *v = addi(*v, 50000);
    }

    let spell_stat = borg_spell_stat();
    if spell_stat >= 0 && !need[spell_stat as usize] {
        *v = addi(*v, 50000);
    }

    // Reward stat potions
    let amt = &st.borg.amt_statgain;
    if amt[STAT_STR as usize] != 0 && tr(BI_CSTR) < 18 + 100 {
        *v = addi(*v, 550000);
    }
    if amt[STAT_INT as usize] != 0 && tr(BI_CINT) < 18 + 100 {
        *v = addi(*v, 520000);
    }
    if spell_stat >= 0
        && amt[spell_stat as usize] != 0
        && tr(BI_CSTR + spell_stat as u32) < 18 + 100
    {
        *v = addi(*v, 575000);
    }
    if amt[STAT_WIS as usize] != 0 && tr(BI_CWIS) < 18 + 100 {
        *v = addi(*v, 520000);
    }
    if amt[STAT_DEX as usize] != 0 && tr(BI_CDEX) < 18 + 100 {
        *v = addi(*v, 550000);
    }
    if amt[STAT_CON as usize] != 0 && tr(BI_CCON) < 18 + 100 {
        *v = addi(*v, 550000);
    }

    // Reward Remove Curse
    if tr(BI_FIRST_CURSED) != 0 {
        if has(iv.kv_scroll_star_remove_curse) != 0 {
            *v = addi(*v, 90000);
        }
        if has(iv.kv_scroll_remove_curse) != 0 {
            *v = addi(*v, 90000);
        }
    }

    // Restore experience
    if tr(BI_HASFIXEXP) != 0 {
        *v = addi(*v, 50000);
    }

    // *** Enchantment ***

    // Reward enchant armor
    if tr(BI_AENCH_ARM) < param(Param::power_inventory_aench_arm_threshold) && tr(BI_NEED_ENCHANT_TO_A) != 0 {
        *v = add(*v, 540);
    }

    // Reward enchant weapon to hit
    if tr(BI_AENCH_TOH) < param(Param::power_inventory_aench_toh_threshold) && tr(BI_NEED_ENCHANT_TO_H) != 0 {
        *v = add(*v, 540);
    }

    // Reward enchant weapon to damage
    if tr(BI_AENCH_TOD) < param(Param::power_inventory_aench_tod_threshold) && tr(BI_NEED_ENCHANT_TO_D) != 0 {
        *v = add(*v, 500);
    }

    // Reward *enchant weapon* to damage
    if tr(BI_AENCH_SWEP) != 0 {
        *v = add(*v, 5000);
    }

    // Reward *enchant armour*
    if tr(BI_AENCH_SARM) != 0 {
        *v = add(*v, 5000);
    }

    // Reward empty slots (up to 6)
    k = 1;
    count_up(v, &mut k, param(Param::power_inventory_empty_limit), tr(BI_EMPTY), i64::from(param(Param::power_inventory_empty_weight)));

    // first empty slot is highly rewarded
    if tr(BI_EMPTY) != 0 {
        *v = add(*v, 4000);
    }

    // Reward carrying a shovel if low level
    if tr(BI_MAXDEPTH) <= param(Param::power_inventory_maxdepth_threshold_11)
        && tr(BI_MAXDEPTH) >= param(Param::power_inventory_maxdepth_threshold_12)
        && tr(BI_GOLD) < param(Param::power_inventory_gold_threshold)
        && u32::from(st.items.items[inven(WIELD)].tval) != TV_DIGGING
        && tr(BI_ADIGGER) == 1
    {
        *v = add(*v, 5000);
    }

    // *** Books ***
    //   Reward books
    for book in 0..9 {
        let copies = i32::from(st.borg.amt_book[book as usize]);

        // No copies
        if copies == 0 {
            continue;
        }

        // The "hard" books
        if engine::class_book_dungeon(book) {
            // Scan the spells
            for what in 0..9 {
                let Some(si) = borg_get_spell_entry(st, book, what) else {
                    break;
                };
                let spell = &st.magic.magics[si];

                // Track minimum level
                if i32::from(spell.level) > tr(BI_MAXCLEVEL) {
                    continue;
                }

                // Track Mana req.
                if i32::from(spell.power) > tr(BI_MAXSP) {
                    continue;
                }

                // Reward the book based on the spells I can cast
                *v = add(*v, 15000);
            }
        }
        // The "easy" books
        else {
            let mut when = 99;

            // Scan the spells
            for what in 0..9 {
                let Some(si) = borg_get_spell_entry(st, book, what) else {
                    break;
                };

                // Track minimum level
                let level = i32::from(st.magic.magics[si].level);
                if level < when {
                    when = level;
                }

                // Track Mana req.
                // if (as->power < mana) mana = as->power;
            }

            // Ignore "difficult" normal books
            if when > param(Param::power_inventory_ignore_difficult_normal_books_threshold) && when >= tr(BI_MAXCLEVEL) + param(Param::power_inventory_maxclevel_threshold_3) {
                continue;
            }
            // if (mana > borg.trait[BI_MAXSP]) continue;

            // Reward the book
            k = 0;
            count_up(v, &mut k, param(Param::power_inventory_reward_the_book_limit), copies, i64::from(param(Param::power_inventory_reward_the_book_weight)));
            if tr(BI_STR) > param(Param::power_inventory_str_threshold_13) {
                count_up(v, &mut k, param(Param::power_inventory_reward_the_book_limit_2), copies, i64::from(param(Param::power_inventory_reward_the_book_weight_2)));
            }
        }
    }

    // Apply "encumbrance" from weight

    // XXX XXX XXX Apply "encumbrance" from weight
    if tr(BI_WEIGHT) > tr(BI_CARRY) / 2 {
        // *HACK*  when testing items, the borg puts them in the last empty
        // slot so this is POSSIBLY just a test item
        // (C starts at `PACK_SLOTS`, one past the last usable pack slot)
        let pack_slots = i32::from(engine::z_info().pack_size) - tr(BI_QUIVER_SLOTS);
        let mut found = None;
        let mut i = pack_slots;
        while i >= 0 {
            if st.items.items[i as usize].iqty != 0 {
                found = Some(&st.items.items[i as usize]);
                break;
            }
            i -= 1;
        }

        // Some items will be used immediately and should not contribute to
        // encumbrance
        let used_at_once = found.is_some_and(|item| {
            let sval = i32::from(item.sval);
            let tval = u32::from(item.tval);
            item.iqty != 0
                && item.aware
                && ((tval == TV_SCROLL
                    && ((sval == iv.sv_scroll_enchant_armor
                        && tr(BI_AENCH_ARM) < param(Param::power_inventory_aench_arm_threshold_2)
                        && tr(BI_NEED_ENCHANT_TO_A) != 0)
                        || (sval == iv.sv_scroll_enchant_weapon_to_hit
                            && tr(BI_AENCH_TOH) < param(Param::power_inventory_aench_toh_threshold_2)
                            && tr(BI_NEED_ENCHANT_TO_H) != 0)
                        || (sval == iv.sv_scroll_enchant_weapon_to_dam
                            && tr(BI_AENCH_TOD) < param(Param::power_inventory_aench_tod_threshold_2)
                            && tr(BI_NEED_ENCHANT_TO_D) != 0)
                        || sval == iv.sv_scroll_star_enchant_weapon
                        || sval == iv.sv_scroll_star_enchant_armor))
                    || (tval == TV_POTION
                        && (sval == iv.sv_potion_inc_str
                            || sval == iv.sv_potion_inc_int
                            || sval == iv.sv_potion_inc_wis
                            || sval == iv.sv_potion_inc_dex
                            || sval == iv.sv_potion_inc_con
                            || sval == iv.sv_potion_inc_all)))
        });
        if !used_at_once {
            // No encumbrance penalty for purchasing the items above;
            // C divides by `BI_CARRY / 10` unchecked (0 traps, Rust panics)
            let over = (tr(BI_WEIGHT) - tr(BI_CARRY) / 2) / (tr(BI_CARRY) / 10);
            *v = sub(*v, i64::from(over) * 1000);
        }
    }

    // Return the value
    value
}

/// Calculate the "power" of the Borg
pub fn borg_power(st: &mut BorgState) -> i32 {
    let mut value: i32 = 0;

    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        value = value.wrapping_add(borg_power_dynamic(st));
    } else {
        // Process the equipment
        value = value.wrapping_add(borg_power_equipment(st));

        // Process the inventory
        value = value.wrapping_add(borg_power_inventory(st));
    }

    // Add a bonus for deep level prep
    // Dump prep codes
    // Scan from surface to deep , stop when not preped
    // (`borg_prepared_c`, so `borg_prepared_buffer` is written as in C)
    let mut i = 1;
    while i <= st.borg.tr(BI_MAXDEPTH) + param(Param::power_prepared_depth_lookahead) {
        // Dump fear code
        if crate::borg_prepared::borg_prepared_c(st, i).is_some() {
            break;
        }
        i += 1;
    }
    value = add(value, i64::from(i - 1) * i64::from(param(Param::power_prepared_depth_weight)));

    // Add the value for the swap items
    value = value.wrapping_add(st.trait_swap.weapon_swap_value);
    value = value.wrapping_add(st.trait_swap.armour_swap_value);

    // Return the value
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_and_int_adds() {
        assert_eq!(add(i32::MAX, 1), i32::MIN);
        assert_eq!(sub(0, 9999999), -9999999);
        assert_eq!(addi(i32::MAX, 1), i32::MIN);
        assert_eq!(subi(i32::MIN, 1), i32::MAX);
    }

    #[test]
    fn count_loops_carry_k() {
        let mut v = 0;
        let mut k = 0;
        count_up(&mut v, &mut k, 3, 10, 5);
        assert_eq!((v, k), (15, 3));
        count_up(&mut v, &mut k, 5, 4, 1);
        assert_eq!((v, k), (16, 4));
        count_down(&mut v, &mut k, 10, 6, 2);
        assert_eq!((v, k), (12, 6));
    }

    #[test]
    fn activation_chain_first_match() {
        let mut a = ActivationIds::default();
        // Unset ids are 0 and never match `act >= 1`.
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 0);
        a.mapping = 5;
        a.dragon_power = 5;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 550);
        a.mapping = 0;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 5150);
        a.tele_other = 7;
        assert_eq!(activation_bonus(&a, CLASS_MAGE, 1, 7), 500);
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 7), 1000);
        a.dispel_evil = 8;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 21, 8), 500 + 10 + 52);
    }
}

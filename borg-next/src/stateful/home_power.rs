//! Port of `borg-home-power.c`: determine the power value of the home.
//! This is used to decide what items are best to keep at home.
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
//! Integer semantics: `value` is an `int32_t`; every C `value += x * 5L`
//! is done in `long` (i64) and truncated back to i32 (`add`). The
//! counters are the `int16_t` `st.home` fields (borg_home_notice), promoted
//! to `int`.

// borg-reborn-rs (#433): neutral M13 inventory policy parameters.
use borg_rs::decision_api::engine;
use borg_rs::decision_api::headers::list_stats_h::{STAT_CON, STAT_DEX, STAT_STR};
use borg_rs::decision_api::headers::list_tvals_h::{TV_FOOD, TV_POTION, TV_SCROLL, TV_STAFF};
use borg_rs::memory::BorgState;
use borg_rs::threat_facts::{param, Param};

/// `CLASS_WARRIOR` (borg-trait.h).
const CLASS_WARRIOR: i32 = 0;

/// `value += x` with `x` a C `long` expression: computed in i64, then
/// truncated to `int32_t`.
fn add(value: i32, x: i64) -> i32 {
    (i64::from(value) + x) as i32
}

/// `value -= x` with `x` a C `long` expression.
fn sub(value: i32, x: i64) -> i32 {
    (i64::from(value) - x) as i32
}

/// The "spare item" tiers used for most flags in `borg_power_home_aux1`:
/// ```c
/// if (n == 1) value += one;
/// else if (n == 2) value += two;
/// else if (n > 2) value += two + (n - 2) * each;
/// ```
fn spare(value: i32, n: i32, one: i64, two: i64, each: i64) -> i32 {
    if n == 1 {
        add(value, one)
    } else if n == 2 {
        add(value, two)
    } else if n > 2 {
        add(value, two + i64::from(n - 2) * each)
    } else {
        value
    }
}

/// `if (n > limit) value -= (n - limit) * each;`
fn too_many(value: i32, n: i32, limit: i32, each: i64) -> i32 {
    if n > limit {
        sub(value, i64::from(n - limit) * each)
    } else {
        value
    }
}

/// The stat-gain tiers:
/// ```c
/// if (s < lo) value += s * a;
/// else if (s < hi) value += base2 + (s - lo) * b;
/// else value += base3 + (s - hi) * c;
/// ```
#[allow(clippy::too_many_arguments)]
fn stat_tiers(
    value: i32,
    s: i32,
    lo: i32,
    hi: i32,
    a: i64,
    base2: i64,
    b: i64,
    base3: i64,
    c: i64,
) -> i32 {
    if s < lo {
        add(value, i64::from(s) * a)
    } else if s < hi {
        add(value, base2 + i64::from(s - lo) * b)
    } else {
        add(value, base3 + i64::from(s - hi) * c)
    }
}

/// `kb_info[tval].max_stack`
fn max_stack(tval: u32) -> i32 {
    engine::base_max_stack(tval)
}

/// Helper function -- calculate power of equipment in the home
fn borg_power_home_aux1(st: &BorgState) -> i32 {
    let mut value: i32 = 0;

    // This would be better separated by item type (so 1 bonus for resist
    // cold armor, 1 bonus for resist cold shield... but that would take a
    // bunch more code.

    // try to collect at least 2 of each resist/power (for swapping)
    // This can be used to get rid of extra artifacts...

    // spare lite sources.  Artifacts only
    value = spare(
        value,
        i32::from(st.home.num_LIGHT),
        i64::from(param(Param::home_light_one_weight)),
        i64::from(param(Param::home_light_two_weight)),
        i64::from(param(Param::home_light_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_slow_digest),
        i64::from(param(Param::home_slow_digest_one_weight)),
        i64::from(param(Param::home_slow_digest_two_weight)),
        i64::from(param(Param::home_slow_digest_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_regenerate),
        i64::from(param(Param::home_regenerate_one_weight)),
        i64::from(param(Param::home_regenerate_two_weight)),
        i64::from(param(Param::home_regenerate_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_telepathy),
        i64::from(param(Param::home_telepathy_one_weight)),
        i64::from(param(Param::home_telepathy_two_weight)),
        i64::from(param(Param::home_telepathy_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_see_inv),
        i64::from(param(Param::home_see_inv_one_weight)),
        i64::from(param(Param::home_see_inv_two_weight)),
        i64::from(param(Param::home_see_inv_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_ffall),
        i64::from(param(Param::home_ffall_one_weight)),
        i64::from(param(Param::home_ffall_two_weight)),
        i64::from(param(Param::home_ffall_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_free_act),
        i64::from(param(Param::home_free_act_one_weight)),
        i64::from(param(Param::home_free_act_two_weight)),
        i64::from(param(Param::home_free_act_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_hold_life),
        i64::from(param(Param::home_hold_life_one_weight)),
        i64::from(param(Param::home_hold_life_two_weight)),
        i64::from(param(Param::home_hold_life_each_weight)),
    );

    value = spare(
        value,
        i32::from(st.home.num_resist_acid),
        i64::from(param(Param::home_resist_acid_one_weight)),
        i64::from(param(Param::home_resist_acid_two_weight)),
        i64::from(param(Param::home_resist_acid_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_immune_acid),
        i64::from(param(Param::home_immune_acid_one_weight)),
        i64::from(param(Param::home_immune_acid_two_weight)),
        i64::from(param(Param::home_immune_acid_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_elec),
        i64::from(param(Param::home_resist_elec_one_weight)),
        i64::from(param(Param::home_resist_elec_two_weight)),
        i64::from(param(Param::home_resist_elec_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_immune_elec),
        i64::from(param(Param::home_immune_elec_one_weight)),
        i64::from(param(Param::home_immune_elec_two_weight)),
        i64::from(param(Param::home_immune_elec_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_fire),
        i64::from(param(Param::home_resist_fire_one_weight)),
        i64::from(param(Param::home_resist_fire_two_weight)),
        i64::from(param(Param::home_resist_fire_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_immune_fire),
        i64::from(param(Param::home_immune_fire_one_weight)),
        i64::from(param(Param::home_immune_fire_two_weight)),
        i64::from(param(Param::home_immune_fire_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_cold),
        i64::from(param(Param::home_resist_cold_one_weight)),
        i64::from(param(Param::home_resist_cold_two_weight)),
        i64::from(param(Param::home_resist_cold_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_immune_cold),
        i64::from(param(Param::home_immune_cold_one_weight)),
        i64::from(param(Param::home_immune_cold_two_weight)),
        i64::from(param(Param::home_immune_cold_each_weight)),
    );

    value = spare(
        value,
        i32::from(st.home.num_resist_pois),
        i64::from(param(Param::home_resist_pois_one_weight)),
        i64::from(param(Param::home_resist_pois_two_weight)),
        i64::from(param(Param::home_resist_pois_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_conf),
        i64::from(param(Param::home_resist_conf_one_weight)),
        i64::from(param(Param::home_resist_conf_two_weight)),
        i64::from(param(Param::home_resist_conf_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_sound),
        i64::from(param(Param::home_resist_sound_one_weight)),
        i64::from(param(Param::home_resist_sound_two_weight)),
        i64::from(param(Param::home_resist_sound_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_LIGHT),
        i64::from(param(Param::home_resist_light_one_weight)),
        i64::from(param(Param::home_resist_light_two_weight)),
        i64::from(param(Param::home_resist_light_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_dark),
        i64::from(param(Param::home_resist_dark_one_weight)),
        i64::from(param(Param::home_resist_dark_two_weight)),
        i64::from(param(Param::home_resist_dark_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_chaos),
        i64::from(param(Param::home_resist_chaos_one_weight)),
        i64::from(param(Param::home_resist_chaos_two_weight)),
        i64::from(param(Param::home_resist_chaos_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_disen),
        i64::from(param(Param::home_resist_disen_one_weight)),
        i64::from(param(Param::home_resist_disen_two_weight)),
        i64::from(param(Param::home_resist_disen_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_shard),
        i64::from(param(Param::home_resist_shard_one_weight)),
        i64::from(param(Param::home_resist_shard_two_weight)),
        i64::from(param(Param::home_resist_shard_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_nexus),
        i64::from(param(Param::home_resist_nexus_one_weight)),
        i64::from(param(Param::home_resist_nexus_two_weight)),
        i64::from(param(Param::home_resist_nexus_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_blind),
        i64::from(param(Param::home_resist_blind_one_weight)),
        i64::from(param(Param::home_resist_blind_two_weight)),
        i64::from(param(Param::home_resist_blind_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_resist_neth),
        i64::from(param(Param::home_resist_neth_one_weight)),
        i64::from(param(Param::home_resist_neth_two_weight)),
        i64::from(param(Param::home_resist_neth_each_weight)),
    );

    // stat gain items as well...(good to carry ring of dex +6 in
    //                            house even if I don't need it right now)
    let s = i32::from(st.home.home_stat_add[STAT_STR as usize]);
    value = stat_tiers(
        value,
        s,
        param(Param::home_stat_str_low_limit),
        param(Param::home_stat_str_high_limit),
        i64::from(param(Param::home_stat_str_first_weight)),
        i64::from(param(Param::home_stat_str_middle_base)),
        i64::from(param(Param::home_stat_str_middle_weight)),
        i64::from(param(Param::home_stat_str_last_base)),
        i64::from(param(Param::home_stat_str_last_weight)),
    );
    let s = i32::from(st.home.home_stat_add[STAT_DEX as usize]);
    value = stat_tiers(
        value,
        s,
        param(Param::home_stat_dex_low_limit),
        param(Param::home_stat_dex_high_limit),
        i64::from(param(Param::home_stat_dex_first_weight)),
        i64::from(param(Param::home_stat_dex_middle_base)),
        i64::from(param(Param::home_stat_dex_middle_weight)),
        i64::from(param(Param::home_stat_dex_last_base)),
        i64::from(param(Param::home_stat_dex_last_weight)),
    );

    // HACK extra con for thorin and other such things
    let s = i32::from(st.home.home_stat_add[STAT_CON as usize]);
    value = stat_tiers(
        value,
        s,
        param(Param::home_stat_con_low_limit),
        param(Param::home_stat_con_high_limit),
        i64::from(param(Param::home_stat_con_first_weight)),
        i64::from(param(Param::home_stat_con_middle_base)),
        i64::from(param(Param::home_stat_con_middle_weight)),
        i64::from(param(Param::home_stat_con_last_base)),
        i64::from(param(Param::home_stat_con_last_weight)),
    );

    // spell stat is only bonused for spell casters.
    let spell_stat = borg_rs::decision_api::borg_magic::borg_spell_stat();
    if spell_stat >= 0 {
        let s = i32::from(st.home.home_stat_add[spell_stat as usize]);
        // C bug 8: the last tier starts from 20 * 100, not 20 * 400.
        value = stat_tiers(
            value,
            s,
            param(Param::home_stat_caster_low_limit),
            param(Param::home_stat_caster_high_limit),
            i64::from(param(Param::home_stat_caster_first_weight)),
            i64::from(param(Param::home_stat_caster_middle_base)),
            i64::from(param(Param::home_stat_caster_middle_weight)),
            i64::from(param(Param::home_stat_caster_last_base)),
            i64::from(param(Param::home_stat_caster_last_weight)),
        );
    }

    // Sustains
    value = spare(
        value,
        i32::from(st.home.num_sustain_str),
        i64::from(param(Param::home_sustain_str_one_weight)),
        i64::from(param(Param::home_sustain_str_two_weight)),
        i64::from(param(Param::home_sustain_str_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_sustain_int),
        i64::from(param(Param::home_sustain_int_one_weight)),
        i64::from(param(Param::home_sustain_int_two_weight)),
        i64::from(param(Param::home_sustain_int_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_sustain_wis),
        i64::from(param(Param::home_sustain_wis_one_weight)),
        i64::from(param(Param::home_sustain_wis_two_weight)),
        i64::from(param(Param::home_sustain_wis_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_sustain_con),
        i64::from(param(Param::home_sustain_con_one_weight)),
        i64::from(param(Param::home_sustain_con_two_weight)),
        i64::from(param(Param::home_sustain_con_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_sustain_dex),
        i64::from(param(Param::home_sustain_dex_one_weight)),
        i64::from(param(Param::home_sustain_dex_two_weight)),
        i64::from(param(Param::home_sustain_dex_each_weight)),
    );
    value = spare(
        value,
        i32::from(st.home.num_sustain_all),
        i64::from(param(Param::home_sustain_all_one_weight)),
        i64::from(param(Param::home_sustain_all_two_weight)),
        i64::from(param(Param::home_sustain_all_each_weight)),
    );

    // do a minus for too many duplicates.  This way we do not store
    // useless items and spread out types of items.
    let n = i32::from(st.home.num_weapons);
    if n > param(Param::home_weapons_soft_limit) {
        value = too_many(
            value,
            n,
            param(Param::home_weapons_soft_limit),
            i64::from(param(Param::home_weapons_excess_weight)),
        );
    } else {
        value = too_many(
            value,
            n,
            param(Param::home_weapons_spare_limit),
            i64::from(param(Param::home_weapons_spare_penalty)),
        );
    }
    value = too_many(
        value,
        i32::from(st.home.num_bow),
        param(Param::home_bows_limit),
        i64::from(param(Param::home_bows_penalty)),
    );
    let n = i32::from(st.home.num_rings);
    if n > param(Param::home_rings_soft_limit) {
        value = too_many(
            value,
            n,
            param(Param::home_rings_soft_limit),
            i64::from(param(Param::home_rings_excess_weight)),
        );
    } else {
        value = too_many(
            value,
            n,
            param(Param::home_rings_spare_limit),
            i64::from(param(Param::home_rings_spare_penalty)),
        );
    }
    value = too_many(
        value,
        i32::from(st.home.num_neck),
        param(Param::home_amulets_limit),
        i64::from(param(Param::home_amulets_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_armor),
        param(Param::home_armor_limit),
        i64::from(param(Param::home_armor_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_cloaks),
        param(Param::home_cloaks_limit),
        i64::from(param(Param::home_cloaks_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_shields),
        param(Param::home_shields_limit),
        i64::from(param(Param::home_shields_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_hats),
        param(Param::home_hats_limit),
        i64::from(param(Param::home_hats_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_gloves),
        param(Param::home_gloves_limit),
        i64::from(param(Param::home_gloves_penalty)),
    );
    value = too_many(
        value,
        i32::from(st.home.num_boots),
        param(Param::home_boots_limit),
        i64::from(param(Param::home_boots_penalty)),
    );

    // `int` addition (no `long` here)
    value = value.wrapping_add(i32::from(st.home.home_damage));

    // if edged and priest, small penalty
    value = sub(
        value,
        i64::from(i32::from(st.home.num_edged_weapon))
            * i64::from(param(Param::home_edged_weapon_penalty)),
    );

    // if gloves and mage or ranger and not FA/Dex, dump it.
    value = sub(
        value,
        i64::from(i32::from(st.home.num_bad_gloves))
            * i64::from(param(Param::home_bad_gloves_penalty)),
    );

    // do not allow duplication of items.
    value = sub(
        value,
        i64::from(i32::from(st.home.num_duplicate_items))
            * i64::from(param(Param::home_duplicate_item_penalty)),
    );

    value
}

/// `for (k = 0; k < lim && k < n; k++) value += base - k * step;`
fn stack(value: i32, lim: i32, n: i32, base: i64, step: i64) -> i32 {
    let mut value = value;
    let mut k = 0;
    while k < lim && k < n {
        value = add(value, base - i64::from(k) * step);
        k += 1;
    }
    value
}

/// Helper function -- calculate power of items in the home
///
/// The weird calculations help spread out the purchase order
fn borg_power_home_aux2(st: &BorgState) -> i32 {
    let mut value: i32 = 0;

    // *** Basic abilities ***

    // Collect food
    if crate::Read::new(st).character().maxclevel() < param(Param::home_food_stock_max_level) {
        value = stack(
            value,
            max_stack(TV_FOOD),
            i32::from(st.home.num_food),
            i64::from(param(Param::home_stock_food_base)),
            i64::from(param(Param::home_stock_food_decay)),
        );
    }

    // Collect ident
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_ident),
        i64::from(param(Param::home_stock_ident_base)),
        i64::from(param(Param::home_stock_ident_decay)),
    );

    // Collect enchantments armour
    if crate::Read::new(st).character().clevel() < param(Param::home_enchant_armor_max_level) {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_a),
            i64::from(param(Param::home_stock_enchant_to_a_base)),
            i64::from(param(Param::home_stock_enchant_to_a_decay)),
        );
    }
    // Collect enchantments to hit
    if crate::Read::new(st).character().clevel() < param(Param::home_enchant_hit_max_level) {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_h),
            i64::from(param(Param::home_stock_enchant_to_h_base)),
            i64::from(param(Param::home_stock_enchant_to_h_decay)),
        );
    }
    // Collect enchantments to dam
    if crate::Read::new(st).character().clevel() < param(Param::home_enchant_damage_max_level) {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_d),
            i64::from(param(Param::home_stock_enchant_to_d_base)),
            i64::from(param(Param::home_stock_enchant_to_d_decay)),
        );
    }

    // Collect pfe
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_pfe),
        i64::from(param(Param::home_stock_pfe_base)),
        i64::from(param(Param::home_stock_pfe_decay)),
    );

    // Collect glyphs
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_glyph),
        i64::from(param(Param::home_stock_glyph_base)),
        i64::from(param(Param::home_stock_glyph_decay)),
    );

    // Reward Genocide scrolls. Just scrolls, mainly used for Morgoth
    value = stack(
        value,
        max_stack(TV_SCROLL) * param(Param::home_banishment_stack_multiplier),
        i32::from(st.home.num_genocide),
        i64::from(param(Param::home_stock_genocide_base)),
        i64::from(param(Param::home_stock_genocide_decay)),
    );

    // Reward Mass Genocide scrolls. Just scrolls, mainly used for Morgoth
    value = stack(
        value,
        max_stack(TV_SCROLL) * param(Param::home_mass_banishment_stack_multiplier),
        i32::from(st.home.num_mass_genocide),
        i64::from(param(Param::home_stock_mass_genocide_base)),
        i64::from(param(Param::home_stock_mass_genocide_decay)),
    );

    // Collect Recharge ability
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_recharge),
        i64::from(param(Param::home_stock_recharge_base)),
        i64::from(param(Param::home_stock_recharge_decay)),
    );

    // Reward Resistance Potions for Warriors
    if crate::Read::new(st).character().class() == CLASS_WARRIOR
        && crate::Read::new(st).character().maxdepth() > param(Param::home_resistance_min_depth)
        && crate::Read::new(st).character().maxdepth() < param(Param::home_resistance_max_depth)
    {
        // `k` carries over from the first loop into the second.
        let mut k = 0;
        while k < max_stack(TV_POTION) && k < i32::from(st.home.num_pot_rheat) {
            value = add(
                value,
                i64::from(param(Param::home_heat_resist_base))
                    - i64::from(k) * i64::from(param(Param::home_heat_resist_decay)),
            );
            k += 1;
        }
        while k < max_stack(TV_POTION) && k < i32::from(st.home.num_pot_rcold) {
            value = add(
                value,
                i64::from(param(Param::home_cold_resist_base))
                    - i64::from(k) * i64::from(param(Param::home_cold_resist_decay)),
            );
            k += 1;
        }
    }

    // Collect recall - stick to 5 spare, for if you run out of money
    value = stack(
        value,
        param(Param::home_stock_recall_limit),
        i32::from(st.home.num_recall),
        i64::from(param(Param::home_stock_recall_base)),
        i64::from(param(Param::home_stock_recall_decay)),
    );

    // Collect escape  (staff of teleport)
    value = stack(
        value,
        param(Param::home_stock_escape_limit),
        i32::from(st.home.num_escape),
        i64::from(param(Param::home_stock_escape_base)),
        i64::from(param(Param::home_stock_escape_decay)),
    );

    // Collect only one stack worth of teleport staffs at home
    let mut k = max_stack(TV_STAFF);
    while k < i32::from(st.home.num_tele_staves) {
        value = sub(
            value,
            i64::from(param(Param::home_teleport_staff_excess_penalty)),
        );
        k += 1;
    }

    // Collect teleport
    value = stack(
        value,
        param(Param::home_stock_teleport_limit),
        i32::from(st.home.num_teleport),
        i64::from(param(Param::home_stock_teleport_base)),
        i64::from(param(Param::home_stock_teleport_decay)),
    );

    // Collect phase
    if crate::Read::new(st).character().maxclevel() < param(Param::home_phase_stock_max_level) {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_phase),
            i64::from(param(Param::home_stock_phase_base)),
            i64::from(param(Param::home_stock_phase_decay)),
        );
    }

    // Collect Speed
    // for (k = 0; k < 85 && k < num_speed; k++) value += 5000L - k*10L;

    // collect mana/
    if crate::Read::new(st).character().maxsp() > param(Param::home_mana_stock_min_sp) {
        value = stack(
            value,
            max_stack(TV_POTION),
            i32::from(st.home.num_mana),
            i64::from(param(Param::home_stock_mana_base)),
            i64::from(param(Param::home_stock_mana_decay)),
        );
    }

    // Level 1 priests are given a Potion of Healing.  It is better
    // for them to sell that potion and buy equipment or several
    // Cure Crits with it.
    if crate::Read::new(st).character().clevel() == param(Param::home_initial_heal_sell_level) {
        value = stack(
            value,
            param(Param::home_stock_initial_heal_limit),
            i32::from(st.home.num_heal),
            i64::from(param(Param::home_stock_initial_heal_base)),
            i64::from(param(Param::home_stock_initial_heal_decay)),
        );
    }

    // *** Healing ***

    // Collect cure critical
    value = stack(
        value,
        max_stack(TV_POTION),
        i32::from(st.home.num_cure_critical),
        i64::from(param(Param::home_stock_cure_critical_base)),
        i64::from(param(Param::home_stock_cure_critical_decay)),
    );

    // Collect heal, *Heal*, Life
    value = stack(
        value,
        param(Param::home_stock_heal_limit),
        i32::from(st.home.num_heal),
        i64::from(param(Param::home_stock_heal_base)),
        i64::from(param(Param::home_stock_heal_decay)),
    );
    value = stack(
        value,
        param(Param::home_stock_ezheal_limit),
        i32::from(st.home.num_ezheal),
        i64::from(param(Param::home_stock_ezheal_base)),
        i64::from(param(Param::home_stock_ezheal_decay)),
    );
    value = stack(
        value,
        param(Param::home_stock_life_limit),
        i32::from(st.home.num_life),
        i64::from(param(Param::home_stock_life_base)),
        i64::from(param(Param::home_stock_life_decay)),
    );

    // junk cure serious if we have some in the home
    // don't bother keeping them if high level
    if crate::Read::new(st).character().clevel() > param(Param::home_cure_serious_penalty_min_level)
    {
        // value -= 1500L - k * 10L
        value = stack(
            value,
            param(Param::home_stock_cure_serious_limit),
            i32::from(st.home.num_cure_serious),
            i64::from(param(Param::home_stock_cure_serious_base)),
            i64::from(param(Param::home_stock_cure_serious_decay)),
        );
    }

    // *** Various ***

    // Fixing Stats
    if crate::Read::new(st).character().clevel() == 50 && i32::from(st.home.num_fix_exp) != 0 {
        value = sub(value, i64::from(param(Param::home_max_level_exp_penalty)));
    }
    if crate::Read::new(st).character().clevel() > param(Param::home_exp_stock_late_min_level)
        && crate::Read::new(st).character().clevel() <= 49
    {
        value = stack(
            value,
            param(Param::home_stock_late_exp_limit),
            i32::from(st.home.num_fix_exp),
            i64::from(param(Param::home_stock_late_exp_base)),
            i64::from(param(Param::home_stock_late_exp_decay)),
        );
    } else if crate::Read::new(st).character().clevel()
        <= param(Param::home_exp_stock_early_max_level)
    {
        value = stack(
            value,
            param(Param::home_stock_early_exp_limit),
            i32::from(st.home.num_fix_exp),
            i64::from(param(Param::home_stock_early_exp_base)),
            i64::from(param(Param::home_stock_early_exp_decay)),
        );
    }

    // *** books ***

    // Reward books
    for book in 0..9 {
        // only collect books up to level 14.
        // After that, just buy them, they are always in stock
        if crate::Read::new(st).character().clevel() < param(Param::home_book_stock_max_level) {
            // Collect up to 5 copies of each normal book
            let mut k = 0;
            while k < param(Param::home_book_stock_limit) && k < i32::from(st.home.num_book[book]) {
                // Only stockpile useful books
                if i32::from(st.home.num_book[book]) != 0 {
                    value = add(
                        value,
                        i64::from(param(Param::home_book_stock_base))
                            - i64::from(k) * i64::from(param(Param::home_book_stock_decay)),
                    );
                }
                k += 1;
            }
        }
    }

    // Reward artifacts in the home
    value = add(
        value,
        i64::from(i32::from(st.home.num_artifact)) * i64::from(param(Param::home_artifact_weight)),
    );

    // Reward certain types of egos in the home
    value = add(
        value,
        i64::from(i32::from(st.home.num_ego)) * i64::from(param(Param::home_ego_weight)),
    );

    // Only allow unid'd stuff if we can't id them
    let un_id = i32::from(st.home.home_un_id);
    if un_id != 0 {
        value = add(
            value,
            i64::from(un_id - crate::Read::new(st).supplies().aid())
                * i64::from(param(Param::home_unidentified_weight)),
        );
    }

    value
}

/// Calculate the "power" of the home
pub fn borg_power_home(st: &BorgState) -> i32 {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark("policy::store.borg_power_home", format!("{:?}", ()));
    }

    let mut value: i32 = 0;

    // Process the home equipment
    value = value.wrapping_add(borg_power_home_aux1(st));

    // Process the home inventory
    value = value.wrapping_add(borg_power_home_aux2(st));

    value
}

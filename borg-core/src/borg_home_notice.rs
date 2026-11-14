//! Port of `borg-home-notice.c`: extract the bonuses for items in the home.
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
//! Integer semantics: the counters are `int16_t` globals. Every C
//! `n += x` is done in `int` and truncated back to `int16_t` (`add`).

use crate::borg_magic::{borg_spell_legal, borg_spell_legal_fail};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{
    ENCHANT_WEAPON, GLYPH_OF_WARDING, HERBAL_CURING, IDENTIFY_RUNE, PROTECTION_FROM_EVIL,
    RECHARGING, REMOVE_HUNGER, TELEPORT_LEVEL, WORD_OF_RECALL,
};
use crate::borg_headers::borg_trait_h::{
    BI_AMMO_TVAL, BI_CLEVEL, BI_SCON, BI_SDEX, BI_SINT, BI_SSTR, BI_SWIS, BI_TODAM,
};
use crate::headers::list_elements_h::{
    ELEM_ACID, ELEM_CHAOS, ELEM_COLD, ELEM_DARK, ELEM_DISEN, ELEM_ELEC, ELEM_FIRE, ELEM_LIGHT,
    ELEM_NETHER, ELEM_NEXUS, ELEM_POIS, ELEM_SHARD, ELEM_SOUND,
};
use crate::headers::list_kind_flags_h::KF_GOOD;
use crate::headers::list_object_flags_h::{
    OF_BLESSED, OF_FEATHER, OF_FREE_ACT, OF_HOLD_LIFE, OF_LIGHT_2, OF_LIGHT_3, OF_PROT_BLIND,
    OF_PROT_CONF, OF_REGEN, OF_SEE_INVIS, OF_SLOW_DIGEST, OF_SUST_CON, OF_SUST_DEX, OF_SUST_INT,
    OF_SUST_STR, OF_SUST_WIS, OF_TELEPATHY,
};
use crate::headers::list_object_modifiers_h::OBJ_MOD_SPEED;
use crate::headers::list_player_flags_h::PF_BLESS_WEAPON;
use crate::headers::list_stats_h::{
    OBJ_MOD_CON, OBJ_MOD_DEX, OBJ_MOD_INT, OBJ_MOD_STR, OBJ_MOD_WIS, STAT_CON, STAT_DEX, STAT_INT,
    STAT_STR, STAT_WIS,
};
use crate::headers::list_tvals_h::{
    TV_AMULET, TV_ARROW, TV_BOLT, TV_BOOTS, TV_BOW, TV_CLOAK, TV_CROWN, TV_DRAG_ARMOR, TV_FLASK,
    TV_FOOD, TV_GLOVES, TV_HAFTED, TV_HARD_ARMOR, TV_HELM, TV_LIGHT, TV_MAGIC_BOOK, TV_NATURE_BOOK,
    TV_OTHER_BOOK, TV_POLEARM, TV_POTION, TV_PRAYER_BOOK, TV_RING, TV_ROD, TV_SCROLL,
    TV_SHADOW_BOOK, TV_SHIELD, TV_SHOT, TV_SOFT_ARMOR, TV_STAFF, TV_SWORD,
};
use crate::state::BorgState;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_item_use::borg_equips_item;
use crate::engine;

/// `BORG_HOME` (borg-store.h): the home's index in `borg_shops`.
pub const BORG_HOME: usize = 7;

/// `STAT_MAX`
const STAT_MAX: usize = 5;

/// Various "amounts" (for the home). Field names are the C globals'
/// (hence `num_LIGHT` and `num_resist_LIGHT`).
#[allow(non_snake_case)]
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct HomeState {
    pub num_food: i16,
    pub num_fuel: i16,
    pub num_mold: i16,
    pub num_ident: i16,
    pub num_recall: i16,
    pub num_phase: i16,
    pub num_escape: i16,
    pub num_tele_staves: i16,
    pub num_teleport: i16,
    pub num_berserk: i16,
    pub num_teleport_level: i16,
    pub num_recharge: i16,

    pub num_cure_critical: i16,
    pub num_cure_serious: i16,

    pub num_pot_rheat: i16,
    pub num_pot_rcold: i16,

    pub num_missile: i16,

    pub num_book: [i16; 9],

    pub num_fix_stat: [i16; STAT_MAX],
    pub home_stat_add: [i16; STAT_MAX],

    pub num_fix_exp: i16,
    pub num_mana: i16,
    pub num_heal: i16,
    pub num_heal_true: i16,
    pub num_ezheal: i16,
    pub num_ezheal_true: i16,
    pub num_life: i16,
    pub num_life_true: i16,
    pub num_pfe: i16,
    pub num_glyph: i16,
    pub num_mass_genocide: i16,
    pub num_speed: i16,

    pub num_enchant_to_a: i16,
    pub num_enchant_to_d: i16,
    pub num_enchant_to_h: i16,
    /// brand bolts (never set in C)
    pub num_brand_weapon: i16,
    /// never counted in C (C bug 12)
    pub num_genocide: i16,

    pub num_artifact: i16,
    pub num_ego: i16,

    pub home_slot_free: i16,
    pub home_un_id: i16,
    pub home_damage: i16,
    pub num_duplicate_items: i16,
    pub num_slow_digest: i16,
    pub num_regenerate: i16,
    pub num_telepathy: i16,
    pub num_LIGHT: i16,
    pub num_see_inv: i16,
    pub num_invisible: i16,

    pub num_ffall: i16,
    pub num_free_act: i16,
    pub num_hold_life: i16,
    pub num_immune_acid: i16,
    pub num_immune_elec: i16,
    pub num_immune_fire: i16,
    pub num_immune_cold: i16,
    pub num_resist_acid: i16,
    pub num_resist_elec: i16,
    pub num_resist_fire: i16,
    pub num_resist_cold: i16,
    pub num_resist_pois: i16,
    pub num_resist_conf: i16,
    pub num_resist_sound: i16,
    pub num_resist_LIGHT: i16,
    pub num_resist_dark: i16,
    pub num_resist_chaos: i16,
    pub num_resist_disen: i16,
    pub num_resist_shard: i16,
    pub num_resist_nexus: i16,
    pub num_resist_blind: i16,
    pub num_resist_neth: i16,
    pub num_sustain_str: i16,
    pub num_sustain_int: i16,
    pub num_sustain_wis: i16,
    pub num_sustain_dex: i16,
    pub num_sustain_con: i16,
    pub num_sustain_all: i16,

    pub num_edged_weapon: i16,
    pub num_bad_gloves: i16,
    pub num_weapons: i16,
    pub num_bow: i16,
    pub num_rings: i16,
    pub num_neck: i16,
    pub num_armor: i16,
    pub num_cloaks: i16,
    pub num_shields: i16,
    pub num_hats: i16,
    pub num_gloves: i16,
    pub num_boots: i16,
}

/// C's `n += x` on an `int16_t` counter: `int` addition, truncated.
fn add(n: &mut i16, x: i32) {
    *n = i32::from(*n).wrapping_add(x) as i16;
}

/// `of_has(item->flags, flag)`
fn of_has(item: &borg_item, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

/// `item->el_info[elem].res_level`
fn res_level(item: &borg_item, elem: u32) -> i16 {
    item.el_info[elem as usize].res_level
}

/// `z_info->store_inven_max`
fn store_inven_max() -> i32 {
    i32::from(engine::z_info().store_inven_max)
}

/// `INVEN_WIELD` (`z_info->pack_size`)
fn inven_wield() -> i32 {
    i32::from(engine::z_info().pack_size)
}

/// `INVEN_TOTAL - INVEN_WIELD`
const INVEN_EQUIP: i32 = 12;

/// `INVEN_LIGHT` (`z_info->pack_size + 5`)
fn inven_light() -> usize {
    usize::from(engine::z_info().pack_size) + 5
}

/// Item `i` of the home scan: the home's wares, then the equipment.
fn home_item(st: &mut BorgState, i: i32) -> borg_item {
    if i < store_inven_max() {
        st.store.shops[BORG_HOME].ware[i as usize]
    } else {
        st.items.items[(i - store_inven_max() + inven_wield()) as usize]
    }
}

/// Helper function -- clear counters for home equipment
///
/// !FIX This needs to change into an array like borg.trait
fn borg_notice_home_clear(h: &mut HomeState, in_item: bool, no_items: bool) {
    // *** Reset counters ***

    // Reset basic
    h.num_food = 0;
    h.num_fuel = 0;
    h.num_mold = 0;
    h.num_ident = 0;
    h.num_recall = 0;
    h.num_phase = 0;
    h.num_escape = 0;
    h.num_tele_staves = 0;
    h.num_teleport = 0;
    h.num_teleport_level = 0;
    h.num_recharge = 0;

    h.num_artifact = 0;
    h.num_ego = 0;

    h.num_invisible = 0;
    h.num_pfe = 0;
    h.num_glyph = 0;
    h.num_genocide = 0;
    h.num_mass_genocide = 0;
    h.num_berserk = 0;
    h.num_pot_rheat = 0;
    h.num_pot_rcold = 0;
    h.num_speed = 0;

    h.num_slow_digest = 0;
    h.num_regenerate = 0;
    h.num_telepathy = 0;
    h.num_see_inv = 0;
    h.num_ffall = 0;
    h.num_free_act = 0;
    h.num_hold_life = 0;
    h.num_immune_acid = 0;
    h.num_immune_elec = 0;
    h.num_immune_fire = 0;
    h.num_immune_cold = 0;
    h.num_resist_acid = 0;
    h.num_resist_elec = 0;
    h.num_resist_fire = 0;
    h.num_resist_cold = 0;
    h.num_resist_pois = 0;
    h.num_resist_conf = 0;
    h.num_resist_sound = 0;
    h.num_resist_LIGHT = 0;
    h.num_resist_dark = 0;
    h.num_resist_chaos = 0;
    h.num_resist_disen = 0;
    h.num_resist_shard = 0;
    h.num_resist_nexus = 0;
    h.num_resist_blind = 0;
    h.num_resist_neth = 0;
    h.num_sustain_str = 0;
    h.num_sustain_int = 0;
    h.num_sustain_wis = 0;
    h.num_sustain_dex = 0;
    h.num_sustain_con = 0;
    h.num_sustain_all = 0;

    h.home_stat_add[STAT_STR as usize] = 0;
    h.home_stat_add[STAT_INT as usize] = 0;
    h.home_stat_add[STAT_WIS as usize] = 0;
    h.home_stat_add[STAT_DEX as usize] = 0;
    h.home_stat_add[STAT_CON as usize] = 0;

    h.num_weapons = 0;

    h.num_bow = 0;
    h.num_rings = 0;
    h.num_neck = 0;
    h.num_armor = 0;
    h.num_cloaks = 0;
    h.num_shields = 0;
    h.num_hats = 0;
    h.num_gloves = 0;
    h.num_boots = 0;
    h.num_LIGHT = 0;
    h.num_speed = 0;
    h.num_edged_weapon = 0;
    h.num_bad_gloves = 0;

    // Reset healing
    h.num_cure_critical = 0;
    h.num_cure_serious = 0;
    h.num_fix_exp = 0;
    h.num_mana = 0;
    h.num_heal = 0;
    h.num_ezheal = 0;
    h.num_life = 0;
    if !in_item && !no_items {
        h.num_ezheal_true = 0;
    }
    if !in_item && !no_items {
        h.num_heal_true = 0;
    }
    if !in_item && !no_items {
        h.num_life_true = 0;
    }

    // Reset missiles
    h.num_missile = 0;

    // Reset books
    h.num_book = [0; 9];

    // Reset various
    h.num_fix_stat[STAT_STR as usize] = 0;
    h.num_fix_stat[STAT_INT as usize] = 0;
    h.num_fix_stat[STAT_WIS as usize] = 0;
    h.num_fix_stat[STAT_DEX as usize] = 0;
    h.num_fix_stat[STAT_CON as usize] = 0;

    // Reset enchantment
    h.num_enchant_to_a = 0;
    h.num_enchant_to_d = 0;
    h.num_enchant_to_h = 0;

    h.home_slot_free = 0;
    h.home_damage = 0;
    h.home_un_id = 0;

    h.num_duplicate_items = 0;
}

/// This checks for duplicate items in the home.
///
/// `scan` holds the items of the home scan before `i` (the C code reads
/// `borg_shops[BORG_HOME].ware[x]` / `borg_items[...]` for `x < i`; nothing
/// changes them during `borg_notice_home`).
fn borg_notice_home_dupe(
    st: &mut BorgState,
    scan: &[borg_item],
    item: &borg_item,
    check_sval: bool,
    i: i32,
) {
    // eventually check for power overlap... armor of resistance is same as
    // weak elvenkind.
    //  two armors of elvenkind that resist poison is a dupe.

    // check for a duplicate.
    // be careful about extra powers (elvenkind/magi)
    if borg_ego_has_random_power(usize::from(item.ego_idx)) {
        return;
    }

    // if it isn't identified, it isn't duplicate
    if item.needs_ident {
        return;
    }

    // if this is a stack of items then all after the first are a
    // duplicate
    let mut dupe_count: i32 = i32::from(item.iqty) - 1;

    // Look for other items before this one that are the same
    for x in 0..i {
        // home wares, then what the borg has on as well
        let item2 = &scan[x as usize];

        // skip zero quantity or unknown items
        if item2.iqty == 0 || !item2.aware {
            continue;
        }

        // if everything matches it is a duplicate item
        // Note that we only check sval on certain items.  This
        // is because, for example, two pairs of dragon armor
        // are not the same unless their subtype (color) matches
        // but a defender is a defender even if one is a dagger and
        // one is a mace
        if item.tval == item2.tval
            && (!check_sval || item.sval == item2.sval)
            && item.art_idx == item2.art_idx
            && item.ego_idx == item2.ego_idx
        {
            dupe_count += 1;
        }
    }

    // there can be one dupe of rings because there are two ring slots.
    if u32::from(item.tval) == TV_RING && dupe_count != 0 {
        dupe_count -= 1;
    }

    // Add this items count to the total duplicate count
    add(&mut st.home.num_duplicate_items, dupe_count);
}

/// Helper function -- notice the home inventory
fn borg_notice_home_aux(st: &mut BorgState, in_item: Option<&borg_item>, no_items: bool) {
    let n = store_inven_max() + INVEN_EQUIP;

    // The scanned items, copied once (see borg_notice_home_dupe). Only the
    // full scan reads them.
    let scan: Vec<borg_item> = if in_item.is_none() && !no_items {
        (0..n).map(|i| home_item(st, i)).collect()
    } else {
        Vec::new()
    };

    // *** Process the inventory ***

    // Scan the home
    for i in 0..n {
        if no_items {
            break;
        }

        let item: &borg_item = match in_item {
            None => &scan[i as usize],
            Some(it) => it,
        };
        let iqty = i32::from(item.iqty);

        // Skip empty items
        if item.iqty == 0 {
            if i < store_inven_max() {
                add(&mut st.home.home_slot_free, 1);
            }
            continue;
        }

        // Skip un-aware items
        if !item.aware {
            continue;
        }

        let h = &mut st.home;
        if of_has(item, OF_SLOW_DIGEST) {
            add(&mut h.num_slow_digest, iqty);
        }
        if of_has(item, OF_REGEN) {
            add(&mut h.num_regenerate, iqty);
        }
        if of_has(item, OF_TELEPATHY) {
            add(&mut h.num_telepathy, iqty);
        }
        if of_has(item, OF_SEE_INVIS) {
            add(&mut h.num_see_inv, iqty);
        }
        if of_has(item, OF_FEATHER) {
            add(&mut h.num_ffall, iqty);
        }
        if of_has(item, OF_FREE_ACT) {
            add(&mut h.num_free_act, iqty);
        }
        if of_has(item, OF_HOLD_LIFE) {
            add(&mut h.num_hold_life, iqty);
        }
        if of_has(item, OF_PROT_CONF) {
            add(&mut h.num_resist_conf, iqty);
        }
        if of_has(item, OF_PROT_BLIND) {
            add(&mut h.num_resist_blind, iqty);
        }
        if res_level(item, ELEM_FIRE) == 3 {
            add(&mut h.num_immune_fire, iqty);
            add(&mut h.num_resist_fire, iqty);
        }
        if res_level(item, ELEM_ACID) == 3 {
            add(&mut h.num_immune_acid, iqty);
            add(&mut h.num_resist_acid, iqty);
        }
        if res_level(item, ELEM_COLD) == 3 {
            add(&mut h.num_immune_cold, iqty);
            add(&mut h.num_resist_cold, iqty);
        }
        if res_level(item, ELEM_ELEC) == 3 {
            add(&mut h.num_immune_elec, iqty);
            add(&mut h.num_resist_elec, iqty);
        }
        if res_level(item, ELEM_ACID) == 1 {
            add(&mut h.num_resist_acid, iqty);
        }
        if res_level(item, ELEM_ELEC) == 1 {
            add(&mut h.num_resist_elec, iqty);
        }
        if res_level(item, ELEM_FIRE) == 1 {
            add(&mut h.num_resist_fire, iqty);
        }
        if res_level(item, ELEM_COLD) == 1 {
            add(&mut h.num_resist_cold, iqty);
        }
        if res_level(item, ELEM_POIS) == 1 {
            add(&mut h.num_resist_pois, iqty);
        }
        if res_level(item, ELEM_SOUND) == 1 {
            add(&mut h.num_resist_sound, iqty);
        }
        if res_level(item, ELEM_LIGHT) == 1 {
            add(&mut h.num_resist_LIGHT, iqty);
        }
        if res_level(item, ELEM_DARK) == 1 {
            add(&mut h.num_resist_dark, iqty);
        }
        if res_level(item, ELEM_CHAOS) == 1 {
            add(&mut h.num_resist_chaos, iqty);
        }
        if res_level(item, ELEM_DISEN) == 1 {
            add(&mut h.num_resist_disen, iqty);
        }
        if res_level(item, ELEM_SHARD) == 1 {
            add(&mut h.num_resist_shard, iqty);
        }
        if res_level(item, ELEM_NEXUS) == 1 {
            add(&mut h.num_resist_nexus, iqty);
        }
        if res_level(item, ELEM_NETHER) == 1 {
            add(&mut h.num_resist_neth, iqty);
        }

        // Count Sustains
        // C bug 12: every item sustain is counted in num_sustain_str.
        if of_has(item, OF_SUST_STR) {
            add(&mut h.num_sustain_str, iqty);
        }
        if of_has(item, OF_SUST_INT) {
            add(&mut h.num_sustain_str, iqty);
        }
        if of_has(item, OF_SUST_WIS) {
            add(&mut h.num_sustain_str, iqty);
        }
        if of_has(item, OF_SUST_DEX) {
            add(&mut h.num_sustain_str, iqty);
        }
        if of_has(item, OF_SUST_CON) {
            add(&mut h.num_sustain_str, iqty);
        }
        if of_has(item, OF_SUST_STR)
            && of_has(item, OF_SUST_INT)
            && of_has(item, OF_SUST_WIS)
            && of_has(item, OF_SUST_DEX)
            && of_has(item, OF_SUST_CON)
        {
            add(&mut h.num_sustain_all, iqty);
        }

        // count up bonus to stats
        // HACK only collect stat rings above +3
        for (m, s) in [
            (OBJ_MOD_STR, STAT_STR),
            (OBJ_MOD_INT, STAT_INT),
            (OBJ_MOD_WIS, STAT_WIS),
            (OBJ_MOD_DEX, STAT_DEX),
            (OBJ_MOD_CON, STAT_CON),
        ] {
            let modifier = i32::from(item.modifiers[m as usize]);
            if modifier != 0 && (u32::from(item.tval) != TV_RING || modifier > 3) {
                add(&mut h.home_stat_add[s as usize], modifier * iqty);
            }
        }

        // count up bonus to speed
        add(
            &mut h.num_speed,
            i32::from(item.modifiers[OBJ_MOD_SPEED as usize]) * iqty,
        );

        // count artifacts
        if item.art_idx != 0 {
            add(&mut h.num_artifact, iqty);
        }
        // count egos that need *ID*
        if borg_ego_has_random_power(usize::from(item.ego_idx)) && item.needs_ident {
            add(&mut st.home.num_ego, iqty);
        }

        // count up unidentified stuff
        if item.needs_ident && i < store_inven_max() {
            add(&mut st.home.home_un_id, 1);
        }

        // Analyze the item
        match u32::from(item.tval) {
            TV_SOFT_ARMOR | TV_HARD_ARMOR => {
                add(&mut st.home.num_armor, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_DRAG_ARMOR => {
                add(&mut st.home.num_armor, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, true, i);
            }

            TV_CLOAK => {
                add(&mut st.home.num_cloaks, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_SHIELD => {
                add(&mut st.home.num_shields, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_HELM | TV_CROWN => {
                add(&mut st.home.num_hats, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_GLOVES => {
                add(&mut st.home.num_gloves, iqty);

                // gloves of slaying give a damage bonus
                add(&mut st.home.home_damage, i32::from(item.to_d) * 3);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_FLASK => {
                // Use as fuel if we equip a lantern
                if i32::from(st.items.items[inven_light()].sval) == st.item_val.sv_light_lantern {
                    add(&mut st.home.num_fuel, iqty);
                }
            }

            TV_LIGHT => {
                // Fuel
                if i32::from(st.items.items[inven_light()].sval) == st.item_val.sv_light_torch {
                    add(&mut st.home.num_fuel, iqty);
                }

                // Artifacts
                if item.art_idx != 0 {
                    add(&mut st.home.num_LIGHT, iqty);
                }
            }

            TV_BOOTS => {
                add(&mut st.home.num_boots, iqty);

                // see if this item is duplicated
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_SWORD | TV_POLEARM | TV_HAFTED => {
                // case TV_DIGGING:
                add(&mut st.home.num_weapons, iqty);
                //  most edged weapons hurt magic for priests
                // Penalize non-blessed edged weapons
                if engine::player_has(PF_BLESS_WEAPON)
                    && !(u32::from(item.tval) == TV_HAFTED || of_has(item, OF_BLESSED))
                {
                    add(&mut st.home.num_edged_weapon, iqty);
                }

                let num_blow = crate::borg_trait::borg_calc_blows(st, item) as i16;
                let dice = i32::from(item.dd) * i32::from(item.ds);
                if item.to_d > 8 || st.borg.tr(BI_CLEVEL) < 15 {
                    let x = i32::from(num_blow)
                        * (dice + (st.borg.tr(BI_TODAM) + i32::from(item.to_d)));
                    add(&mut st.home.home_damage, x);
                } else {
                    let x = i32::from(num_blow) * (dice + (st.borg.tr(BI_TODAM) + 8));
                    add(&mut st.home.home_damage, x);
                }

                // see if this item is a duplicate
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_BOW => {
                add(&mut st.home.num_bow, iqty);

                // see if this item is a duplicate
                borg_notice_home_dupe(st, &scan, item, false, i);
            }

            TV_RING => {
                add(&mut st.home.num_rings, iqty);

                // see if this item is a duplicate
                borg_notice_home_dupe(st, &scan, item, true, i);
            }

            TV_AMULET => {
                add(&mut st.home.num_neck, iqty);

                // see if this item is a duplicate
                borg_notice_home_dupe(st, &scan, item, true, i);
            }

            // Books
            TV_MAGIC_BOOK | TV_PRAYER_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK => {
                let kind = item.kind as usize;

                // Skip incorrect books (if we can browse this book, it is
                // good), only ever store non-dungeon books
                if engine::obj_kind_can_browse(kind)
                    && !engine::flag_has(&engine::kind(kind).kind_flags, KF_GOOD as i32)
                {
                    // Count the books
                    add(&mut st.home.num_book[usize::from(item.sval)], iqty);
                }
            }

            // Food
            TV_FOOD => {
                let sval = i32::from(item.sval);
                let v = &st.item_val;
                let h = &mut st.home;
                if sval == v.sv_food_ration {
                    add(&mut h.num_food, iqty);
                } else if sval == v.sv_food_slime_mold {
                    add(&mut h.num_mold, iqty);
                } else if sval == v.sv_mush_purging {
                    add(&mut h.num_fix_stat[STAT_CON as usize], iqty);
                    add(&mut h.num_fix_stat[STAT_STR as usize], iqty);
                } else if sval == v.sv_mush_restoring {
                    add(&mut h.num_fix_stat[STAT_STR as usize], iqty);
                    add(&mut h.num_fix_stat[STAT_INT as usize], iqty);
                    add(&mut h.num_fix_stat[STAT_WIS as usize], iqty);
                    add(&mut h.num_fix_stat[STAT_DEX as usize], iqty);
                    add(&mut h.num_fix_stat[STAT_CON as usize], iqty);
                }
            }

            // Potions
            TV_POTION => {
                let sval = i32::from(item.sval);
                let v = &st.item_val;
                let h = &mut st.home;
                let real = in_item.is_none() && !no_items;

                // Analyze
                if sval == v.sv_potion_cure_critical {
                    add(&mut h.num_cure_critical, iqty);
                } else if sval == v.sv_potion_cure_serious {
                    add(&mut h.num_cure_serious, iqty);
                } else if sval == v.sv_potion_resist_heat {
                    add(&mut h.num_pot_rheat, iqty);
                } else if sval == v.sv_potion_resist_cold {
                    add(&mut h.num_pot_rcold, iqty);
                } else if sval == v.sv_potion_restore_life {
                    add(&mut h.num_fix_exp, iqty);
                } else if sval == v.sv_potion_restore_mana {
                    add(&mut h.num_mana, iqty);
                } else if sval == v.sv_potion_healing {
                    add(&mut h.num_heal, iqty);
                    if real {
                        add(&mut h.num_heal_true, iqty);
                    }
                } else if sval == v.sv_potion_star_healing {
                    add(&mut h.num_ezheal, iqty);
                    if real {
                        add(&mut h.num_ezheal_true, iqty);
                    }
                } else if sval == v.sv_potion_life {
                    add(&mut h.num_life, iqty);
                    if real {
                        add(&mut h.num_life_true, iqty);
                    }
                } else if sval == v.sv_potion_berserk {
                    add(&mut h.num_berserk, iqty);
                } else if sval == v.sv_potion_speed {
                    add(&mut h.num_speed, iqty);
                }
            }

            // Scrolls
            TV_SCROLL => {
                let sval = i32::from(item.sval);
                let v = &st.item_val;
                let h = &mut st.home;

                // Analyze the scroll
                if sval == v.sv_scroll_identify {
                    add(&mut h.num_ident, iqty);
                } else if sval == v.sv_scroll_phase_door {
                    add(&mut h.num_phase, iqty);
                } else if sval == v.sv_scroll_teleport {
                    add(&mut h.num_teleport, iqty);
                } else if sval == v.sv_scroll_word_of_recall {
                    add(&mut h.num_recall, iqty);
                } else if sval == v.sv_scroll_enchant_armor {
                    add(&mut h.num_enchant_to_a, iqty);
                } else if sval == v.sv_scroll_enchant_weapon_to_hit {
                    add(&mut h.num_enchant_to_h, iqty);
                } else if sval == v.sv_scroll_enchant_weapon_to_dam {
                    add(&mut h.num_enchant_to_d, iqty);
                } else if sval == v.sv_scroll_protection_from_evil {
                    add(&mut h.num_pfe, iqty);
                } else if sval == v.sv_scroll_rune_of_protection {
                    add(&mut h.num_glyph, iqty);
                } else if sval == v.sv_scroll_teleport_level {
                    add(&mut h.num_teleport_level, iqty);
                } else if sval == v.sv_scroll_recharging {
                    add(&mut h.num_recharge, iqty);
                } else if sval == v.sv_scroll_banishment {
                    add(&mut h.num_genocide, iqty);
                } else if sval == v.sv_scroll_mass_banishment {
                    add(&mut h.num_mass_genocide, iqty);
                }
            }

            // Rods
            TV_ROD => {
                // Analyze
                if i32::from(item.sval) == st.item_val.sv_rod_recall {
                    add(&mut st.home.num_recall, iqty * 100);
                }
            }

            // Staffs
            TV_STAFF => {
                // only collect staves with more than 3 charges at high level
                let skip = item.pval <= 3 && st.borg.tr(BI_CLEVEL) > 30;

                // Analyze
                if !skip && i32::from(item.sval) == st.item_val.sv_staff_teleportation {
                    add(&mut st.home.num_escape, i32::from(item.pval) * iqty);
                    add(&mut st.home.num_tele_staves, 1);
                }
            }

            // Missiles
            // Ignore invalid missiles
            // !FIX !TODO: be willing to switch to a better weapon.
            // Ignore worthless missiles
            TV_SHOT | TV_ARROW | TV_BOLT
                if i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL) && item.value > 0 =>
            {
                // Count them
                add(&mut st.home.num_missile, iqty);
            }

            _ => {}
        }

        // if only doing one item, break.
        if in_item.is_some() {
            break;
        }
    }

    // *** Process the Spells and Prayers ***

    // Handle "satisfy hunger" -> infinite food
    if borg_spell_legal(st, REMOVE_HUNGER) || borg_spell_legal(st, HERBAL_CURING) {
        add(&mut st.home.num_food, 1000);
    }

    // Handle "identify" -> infinite identifies
    if borg_spell_legal(st, IDENTIFY_RUNE) {
        add(&mut st.home.num_ident, 1000);
    }

    // Handle ENCHANT_WEAPON
    if borg_spell_legal_fail(st, ENCHANT_WEAPON, 65) {
        add(&mut st.home.num_enchant_to_h, 1000);
        add(&mut st.home.num_enchant_to_d, 1000);
    }

    //  Handle PROTECTION_FROM_EVIL
    if borg_spell_legal(st, PROTECTION_FROM_EVIL) {
        add(&mut st.home.num_pfe, 1000);
    }

    //  Handle "rune of protection" glyph
    let act_glyph = st.act.glyph;
    if borg_spell_legal(st, GLYPH_OF_WARDING) || borg_equips_item(st, act_glyph, false) {
        add(&mut st.home.num_glyph, 1000);
    }

    // handle restore

    // Handle recall
    if borg_spell_legal(st, WORD_OF_RECALL) {
        add(&mut st.home.num_recall, 1000);
    }

    // Handle teleport_level
    if borg_spell_legal(st, TELEPORT_LEVEL) {
        add(&mut st.home.num_teleport_level, 1000);
    }

    // Handle recharge
    if borg_spell_legal(st, RECHARGING) {
        add(&mut st.home.num_recharge, 1000);
    }

    // *** Process the Needs ***

    // HACK: No need for stat repair
    for (bi, s) in [
        (BI_SSTR, STAT_STR),
        (BI_SINT, STAT_INT),
        (BI_SWIS, STAT_WIS),
        (BI_SDEX, STAT_DEX),
        (BI_SCON, STAT_CON),
    ] {
        if st.borg.tr(bi) != 0 {
            add(&mut st.home.num_fix_stat[s as usize], 1000);
        }
    }

    // Extract the player flags
    let f = engine::player_flags();
    let has = |flag: u32| engine::flag_has(&f, flag as i32);
    let h = &mut st.home;

    // Good flags
    if has(OF_SLOW_DIGEST) {
        add(&mut h.num_slow_digest, 1);
    }
    if has(OF_FEATHER) {
        add(&mut h.num_ffall, 1);
    }
    // (C uses rf_has for OF_LIGHT_3; the same bit test.)
    if has(OF_LIGHT_2) || has(OF_LIGHT_3) {
        add(&mut h.num_LIGHT, 1);
    }
    if has(OF_REGEN) {
        add(&mut h.num_regenerate, 1);
    }
    if has(OF_TELEPATHY) {
        add(&mut h.num_telepathy, 1);
    }
    if has(OF_SEE_INVIS) {
        add(&mut h.num_see_inv, 1);
    }
    if has(OF_FREE_ACT) {
        add(&mut h.num_free_act, 1);
    }
    if has(OF_HOLD_LIFE) {
        add(&mut h.num_hold_life, 1);
    }
    if has(OF_PROT_CONF) {
        add(&mut h.num_resist_conf, 1);
    }
    if has(OF_PROT_BLIND) {
        add(&mut h.num_resist_blind, 1);
    }

    // Weird flags

    // Bad flags

    // Immunity flags
    let race = engine::player_race_res_level;
    if race(ELEM_FIRE) == 3 {
        add(&mut h.num_immune_fire, 1);
    }
    if race(ELEM_ACID) == 3 {
        add(&mut h.num_immune_acid, 1);
    }
    if race(ELEM_COLD) == 3 {
        add(&mut h.num_immune_cold, 1);
    }
    if race(ELEM_ELEC) == 3 {
        add(&mut h.num_immune_elec, 1);
    }

    // Resistance flags
    if race(ELEM_ACID) > 0 {
        add(&mut h.num_resist_acid, 1);
    }
    if race(ELEM_ELEC) > 0 {
        add(&mut h.num_resist_elec, 1);
    }
    if race(ELEM_FIRE) > 0 {
        add(&mut h.num_resist_fire, 1);
    }
    if race(ELEM_COLD) > 0 {
        add(&mut h.num_resist_cold, 1);
    }
    if race(ELEM_POIS) > 0 {
        add(&mut h.num_resist_pois, 1);
    }
    if race(ELEM_LIGHT) > 0 {
        add(&mut h.num_resist_LIGHT, 1);
    }
    if race(ELEM_DARK) > 0 {
        add(&mut h.num_resist_dark, 1);
    }
    if race(ELEM_SOUND) > 0 {
        add(&mut h.num_resist_sound, 1);
    }
    if race(ELEM_SHARD) > 0 {
        add(&mut h.num_resist_shard, 1);
    }
    if race(ELEM_NEXUS) > 0 {
        add(&mut h.num_resist_nexus, 1);
    }
    if race(ELEM_NETHER) > 0 {
        add(&mut h.num_resist_neth, 1);
    }
    if race(ELEM_CHAOS) > 0 {
        add(&mut h.num_resist_chaos, 1);
    }
    if race(ELEM_DISEN) > 0 {
        add(&mut h.num_resist_disen, 1);
    }

    // Sustain flags (C uses rf_has; the same bit test)
    if has(OF_SUST_STR) {
        add(&mut h.num_sustain_str, 1);
    }
    if has(OF_SUST_INT) {
        add(&mut h.num_sustain_int, 1);
    }
    if has(OF_SUST_WIS) {
        add(&mut h.num_sustain_wis, 1);
    }
    if has(OF_SUST_DEX) {
        add(&mut h.num_sustain_dex, 1);
    }
    if has(OF_SUST_CON) {
        add(&mut h.num_sustain_con, 1);
    }
}

/// Extract the bonuses for items in the home.
///
/// `in_item` is passed in if you want to pretend that `in_item` is the
/// only item in the home. `no_items` is passed in as true if you want to
/// pretend that the home is empty.
pub fn borg_notice_home(st: &mut BorgState, in_item: Option<&borg_item>, no_items: bool) {
    // Notice the home equipment
    borg_notice_home_clear(&mut st.home, in_item.is_some(), no_items);

    // Notice the home inventory
    borg_notice_home_aux(st, in_item, no_items);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_truncates_like_int16() {
        let mut n: i16 = i16::MAX;
        add(&mut n, 1);
        assert_eq!(n, i16::MIN);
        let mut n: i16 = 30000;
        add(&mut n, 1000 * 100);
        assert_eq!(n, (30000i32 + 100_000) as i16);
    }

    #[test]
    fn clear_keeps_true_counts_for_pretend_calls() {
        let mut h = HomeState {
            num_heal_true: 3,
            num_ezheal_true: 4,
            num_life_true: 5,
            num_food: 7,
            num_brand_weapon: 2,
            ..HomeState::default()
        };
        borg_notice_home_clear(&mut h, true, false);
        assert_eq!(
            (h.num_heal_true, h.num_ezheal_true, h.num_life_true),
            (3, 4, 5)
        );
        assert_eq!(h.num_food, 0);
        borg_notice_home_clear(&mut h, false, true);
        assert_eq!(
            (h.num_heal_true, h.num_ezheal_true, h.num_life_true),
            (3, 4, 5)
        );
        borg_notice_home_clear(&mut h, false, false);
        assert_eq!(
            (h.num_heal_true, h.num_ezheal_true, h.num_life_true),
            (0, 0, 0)
        );
        // never touched by C
        assert_eq!(h.num_brand_weapon, 2);
    }
}

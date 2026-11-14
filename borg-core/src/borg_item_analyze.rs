//! Port of `borg-item-analyze.c`: evaluate items to determine what powers
//! the borg knows about.
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

use crate::borg_item_activation::ActivationIds;
use crate::borg_item_val::ItemValState;
use crate::engine::{self, GameObject};
use crate::borg_headers::borg_item_h::*;
use crate::headers::list_kind_flags_h::{
    KF_RAND_BASE_RES, KF_RAND_HI_RES, KF_RAND_POWER, KF_RAND_RES_POWER, KF_RAND_SUSTAIN,
};
use crate::headers::list_object_modifiers_h::*;
use crate::headers::list_stats_h::{OBJ_MOD_CON, OBJ_MOD_DEX, OBJ_MOD_INT, OBJ_MOD_STR, OBJ_MOD_WIS};
use crate::headers::list_tvals_h::*;
use crate::headers::obj_properties_h::OBJ_MOD_MAX;
use crate::headers::object_h::ELEM_MAX;

/// `MAX_BLOWS` (player-calcs.h)
const MAX_BLOWS: i32 = 5;

/// `FLAG_START` / `FLAG_END` (z-bitflag.h)
const FLAG_START: i32 = 1;
const FLAG_END: i32 = 0;

/// `sizeof(item->desc)`
const DESC_SIZE: usize = 80;

/// Determine the "base price" of a known item (`borg_object_value_known`,
/// adapted from `object_value_known()`): a guess at the value of an item
/// if it was sold to a store, with perfect "charisma" modifiers.
///
/// The `long` sums are done in `i64` and truncated to `int32_t` as in C;
/// the stat bonus test reads the positive `OBJ_MOD_*` modifiers
/// (better-borg-c milestone8 patch 0082).
fn borg_object_value_known(iv: &ItemValState, item: &borg_item) -> i32 {
    let k_ptr = engine::kind(item.kind as usize);

    /* Worthless items */
    if k_ptr.cost == 0 {
        return 0;
    }

    /* Extract the base value */
    let mut value: i32 = k_ptr.cost;

    /* Use artifact base costs Cheat */
    if item.art_idx != 0 {
        let a_ptr = engine::artifact(usize::from(item.art_idx));

        /* Worthless artifacts */
        if a_ptr.cost == 0 {
            return 0;
        }

        /* Use the artifact cost Cheat */
        value = a_ptr.cost;
    }

    /* Add in ego-item bonus cost Cheat */
    if item.ego_idx != 0 {
        let e_ptr = engine::ego(usize::from(item.ego_idx));

        /* Worthless ego-items */
        if e_ptr.cost == 0 {
            return 0;
        }

        /* Reward the ego-item cost Cheat */
        value = value.wrapping_add(e_ptr.cost);
    }

    /* `value += x * nL` (computed as long, stored as int32_t) */
    let add_long = |value: i32, x: i64| (i64::from(value) + x) as i32;
    let m = |i: u32| i64::from(item.modifiers[i as usize]);

    /* Analyze pval bonus */
    match u32::from(item.tval) {
        /* Wands/Staffs */
        TV_WAND | TV_STAFF => {
            /* Pay extra for charges */
            value = value.wrapping_add((value / 20).wrapping_mul(i32::from(item.pval)));
        }

        /* Wearable items */
        TV_SHOT | TV_ARROW | TV_BOLT | TV_BOW | TV_DIGGING | TV_HAFTED | TV_POLEARM
        | TV_SWORD | TV_BOOTS | TV_GLOVES | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK
        | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR | TV_LIGHT | TV_AMULET | TV_RING => {
            /* Negative "pval" is always bad */
            if item.pval < 0 {
                return 0;
            }

            /* Give credit for stat bonuses (the stat modifiers themselves:
             * better-borg-c milestone8 patch 0082) */
            for stat in [OBJ_MOD_STR, OBJ_MOD_INT, OBJ_MOD_WIS, OBJ_MOD_DEX, OBJ_MOD_CON] {
                if m(stat) > 0 {
                    value = add_long(value, m(stat) * 200);
                }
            }

            /* Give credit for stealth and searching */
            value = add_long(value, m(OBJ_MOD_STEALTH) * 100);
            value = add_long(value, m(OBJ_MOD_SEARCH) * 100);

            /* Give credit for infra-vision and tunneling */
            value = add_long(value, m(OBJ_MOD_INFRA) * 50);
            value = add_long(value, m(OBJ_MOD_TUNNEL) * 50);

            /* Give credit for extra attacks */
            if m(OBJ_MOD_BLOWS) != 0 {
                if m(OBJ_MOD_BLOWS) > i64::from(MAX_BLOWS) {
                    value = add_long(value, i64::from(MAX_BLOWS) * 2000);
                } else {
                    value = add_long(value, m(OBJ_MOD_BLOWS) * 2000);
                }
            }
            value = add_long(value, m(OBJ_MOD_SHOTS) * 2000);

            /* Give credit for speed bonus */
            value = add_long(value, m(OBJ_MOD_SPEED) * 30000);

            /* Give credit for glowing bonus */
            value = add_long(value, m(OBJ_MOD_LIGHT) * 100);

            /* Give credit for might */
            value = add_long(value, m(OBJ_MOD_MIGHT) * 100);

            /* Give credit for moves */
            value = add_long(value, m(OBJ_MOD_MOVES) * 100);

            /* Give credit for damage reduction */
            value = add_long(value, m(OBJ_MOD_DAM_RED) * 200);
        }
        _ => {}
    }

    let (to_h, to_d, to_a) = (i32::from(item.to_h), i32::from(item.to_d), i32::from(item.to_a));
    let (dd, ds) = (i32::from(item.dd), i32::from(item.ds));

    /* Analyze the item */
    match u32::from(item.tval) {
        /* Rings/Amulets */
        TV_RING | TV_AMULET => {
            /* HACK special case (rings only, then fall through) */
            if u32::from(item.tval) == TV_RING && i32::from(item.sval) == iv.sv_ring_dog {
                return 0;
            }

            /* Negative bonuses are bad */
            if to_a < 0 {
                return 0;
            }
            if to_h < 0 {
                return 0;
            }
            if to_d < 0 {
                return 0;
            }

            /* Give credit for bonuses */
            value = add_long(value, i64::from(to_h + to_d + to_a) * 100);
        }

        /* Armor */
        TV_BOOTS | TV_GLOVES | TV_CLOAK | TV_CROWN | TV_HELM | TV_SHIELD | TV_SOFT_ARMOR
        | TV_HARD_ARMOR | TV_DRAG_ARMOR => {
            /* Negative armor bonus */
            if to_a < 0 {
                return 0;
            }

            /* Give credit for bonuses */
            /* ignore low to_hit on armor for now since the base armor is marked */
            /* and that should be built into the value */
            if to_h < 0 && to_h > -5 {
                value = add_long(value, i64::from(to_d + to_a) * 100);
            } else {
                value = add_long(value, i64::from(to_h + to_d + to_a) * 100);
            }
        }

        /* Bows/Weapons */
        TV_BOW | TV_DIGGING | TV_HAFTED | TV_SWORD | TV_POLEARM => {
            /* Negative hit/damage bonuses */
            if to_h + to_d < 0 {
                return 0;
            }

            /* Factor in the bonuses */
            value = add_long(value, i64::from(to_h + to_d + to_a) * 100);

            /* Factor in extra damage dice */
            if dd > k_ptr.dd && ds == k_ptr.ds {
                value = add_long(value, i64::from((dd - k_ptr.dd).wrapping_mul(ds)) * 200);
            }
        }

        /* Ammo */
        TV_SHOT | TV_ARROW | TV_BOLT => {
            /* Negative hit/damage bonuses */
            if to_h + to_d < 0 {
                return 0;
            }

            /* Factor in the bonuses */
            value = add_long(value, i64::from(to_h + to_d) * 5);

            /* Factor in extra damage dice */
            if dd > k_ptr.dd && ds == k_ptr.ds {
                value = add_long(value, i64::from((dd - k_ptr.dd).wrapping_mul(ds)) * 5);
            }
        }
        _ => {}
    }

    /* Return the value */
    value
}

/// Guess the value of un-id'd items (`borg_object_value_guess`).
fn borg_object_value_guess(item: &borg_item) -> i32 {
    let mut value: i32;

    /* Guess at value */
    match u32::from(item.tval) {
        TV_FOOD => value = 5,
        TV_POTION => value = 20,
        TV_SCROLL => value = 20,
        TV_STAFF => value = 70,
        TV_WAND => value = 50,
        TV_ROD => value = 90,
        TV_RING | TV_AMULET => {
            value = 45;

            /* Negative bonuses are bad */
            if item.to_a < 0 {
                value = 0;
            }
            if item.to_h < 0 {
                value = 0;
            }
            if item.to_d < 0 {
                value = 0;
            }

            let mut i = 0;
            while i < OBJ_MOD_MAX as usize && value != 0 {
                if item.modifiers[i] < 0 {
                    value = 0;
                }
                i += 1;
            }
        }
        _ => {
            value = 20;

            /* Negative bonuses are bad */
            if item.to_a < 0 {
                value = 0;
            }
            if item.to_h < 0 {
                value = 0;
            }
            if item.to_d < 0 {
                value = 0;
            }
        }
    }

    value
}

/// Convert from the object slays structure to a basic multiplier per race
/// (`borg_set_slays`); `o_slays` is the object's non-NULL `slays` array.
fn borg_set_slays(item: &mut borg_item, o_slays: &[bool]) {
    for (i, &has) in o_slays.iter().enumerate() {
        if has {
            let (race_flag, multiplier) = engine::slay_race_flag_multiplier(i);
            item.slays[race_flag as usize] = multiplier;
        }
    }
}

/// The Borg's curse index for a curse name (`BORG_CURSE_UNKNOWN` for one
/// it does not know).
fn borg_curse_index(name: &[u8]) -> C2RustUnnamedH_44 {
    match name {
        b"vulnerability" => BORG_CURSE_VULNERABILITY,
        b"teleportation" => BORG_CURSE_TELEPORTATION,
        b"dullness" => BORG_CURSE_DULLNESS,
        b"sickliness" => BORG_CURSE_SICKLINESS,
        b"enveloping" => BORG_CURSE_ENVELOPING,
        b"irritation" => BORG_CURSE_IRRITATION,
        b"weakness" => BORG_CURSE_WEAKNESS,
        b"clumsiness" => BORG_CURSE_CLUMSINESS,
        b"slowness" => BORG_CURSE_SLOWNESS,
        b"annoyance" => BORG_CURSE_ANNOYANCE,
        b"poison" => BORG_CURSE_POISON,
        b"siren" => BORG_CURSE_SIREN,
        b"hallucination" => BORG_CURSE_HALLUCINATION,
        b"paralysis" => BORG_CURSE_PARALYSIS,
        b"dragon summon" => BORG_CURSE_DRAGON_SUMMON,
        b"demon summon" => BORG_CURSE_DEMON_SUMMON,
        b"undead summon" => BORG_CURSE_UNDEAD_SUMMON,
        b"impair mana recovery" => BORG_CURSE_IMPAIR_MANA_RECOVERY,
        b"impair hitpoint recovery" => BORG_CURSE_IMPAIR_HITPOINT_RECOVERY,
        b"cowardice" => BORG_CURSE_COWARDICE,
        b"stone" => BORG_CURSE_STONE,
        b"anti-teleportation" => BORG_CURSE_ANTI_TELEPORTATION,
        b"treacherous weapon" => BORG_CURSE_TREACHEROUS_WEAPON,
        b"burning up" => BORG_CURSE_BURNING_UP,
        b"chilled to the bone" => BORG_CURSE_CHILLED_TO_THE_BONE,
        b"steelskin" => BORG_CURSE_STEELSKIN,
        b"air swing" => BORG_CURSE_AIR_SWING,
        _ => BORG_CURSE_UNKNOWN,
    }
}

/// Convert from the dynamic curses to the set the borg knows
/// (`borg_set_curses`); `powers` are the powers of the object's non-NULL
/// `curses` array.
fn borg_set_curses(item: &mut borg_item, powers: &[i32]) {
    item.uncursable = false;
    for (i, &power) in powers.iter().enumerate() {
        if power > 0 {
            item.cursed = true;
            if power < 100 {
                item.uncursable = true;
            }
            item.curses[borg_curse_index(engine::curse_name(i)) as usize] = true;
        }
    }
}

/// Look for a negative flag on the object (`borg_item_has_negative_flag`).
fn borg_item_has_negative_flag(item: &borg_item) -> bool {
    let mut i = engine::flag_next(&item.flags, FLAG_START);
    while i != FLAG_END {
        /* Get the flag details */
        if let Some(power) = engine::obj_flag_power(i) {
            if power < 0 {
                return true;
            }
        }
        i = engine::flag_next(&item.flags, i + 1);
    }
    false
}

/// Analyze an item, also given its name (`borg_item_analyze`).
///
/// This cheats all the information, and maybe is getting information
/// that the player doesn't always get. The best way to fix this is to
/// refactor the main game code to get it to make a 'fake' object that
/// contains only known info and copy from that.
///
/// `desc` is the description (C string semantics: it ends at the first
/// NUL, if any); at most 79 bytes of it are kept. The item's `note` is
/// pointed at its inscription (or the end) within the item's own `desc`;
/// `origin` names the live element `item` is (C's pointer target).
pub fn borg_item_analyze(
    item: &mut borg_item,
    origin: crate::borg_item::NoteOrigin,
    real_item: GameObject,
    desc: &[u8],
    in_store: bool,
    act: &ActivationIds,
    iv: &ItemValState,
) {
    /* Wipe the item */
    *item = crate::borg_item::blank_item();

    /* Non-item */
    if !real_item.has_kind() || real_item.number() == 0 {
        return;
    }

    /* see if the object is fully identified.  If it is, use the base object */
    item.ident = real_item.fully_known();
    let o = if item.ident {
        real_item
    } else {
        /* this needs to be a good pointer or object_flags_know will crash */
        real_item.known().unwrap_or(real_item)
    };
    item.needs_ident = !real_item.runes_known();

    /* Extract data from the game */
    item.flags = real_item.flags_known();

    /* Save the item description */
    let len = desc.iter().position(|&b| b == 0).unwrap_or(desc.len()).min(DESC_SIZE - 1);
    for (d, &s) in item.desc.iter_mut().zip(&desc[..len]) {
        *d = s as core::ffi::c_char;
    }
    item.desc[len] = 0;

    /* Advance to the "inscription" or end of string and save */
    let scan = item.desc.iter().position(|&c| c == 0 || c as u8 == b'{').expect("NUL-terminated");
    crate::borg_item::set_item_note(item, origin, scan);

    /* Get various info */
    item.tval = real_item.tval();
    item.sval = real_item.sval();
    item.iqty = real_item.number();
    item.weight = real_item.weight_one();
    item.timeout = real_item.timeout();
    item.level = real_item.kind_level() as u8;

    /* always aware of items in the store */
    item.aware = in_store || real_item.flavor_is_aware();

    /* get info from the known part of the object */
    item.ac = o.ac();
    item.dd = o.dd();
    item.ds = o.ds();
    item.to_h = o.to_h();
    item.to_d = o.to_d();
    item.to_a = o.to_a();

    /* copy modifiers that are known */
    let mods = o.modifiers();
    item.modifiers[..OBJ_MOD_MAX as usize].copy_from_slice(&mods[..OBJ_MOD_MAX as usize]);

    let el = o.el_info();
    for (d, s) in item.el_info.iter_mut().zip(&el).take(ELEM_MAX as usize) {
        d.res_level = s.res_level;
        d.flags = s.flags;
    }

    if let Some(powers) = o.curse_powers() {
        borg_set_curses(item, &powers);
    }

    /* If any of the flags are negative, count the object as cursed */
    if borg_item_has_negative_flag(item) {
        item.cursed = true;
    }

    if let Some(slays) = o.slays() {
        borg_set_slays(item, &slays);
    }

    if let Some(brands) = o.brands() {
        item.brands[..brands.len()].copy_from_slice(&brands);
    }

    /* check if we know this is the one ring */
    /* HACK we assume The One Ring is the only artifact that does BIZARRE */
    if let Some(index) = o.activation_index() {
        if index == act.bizarre {
            item.one_ring = true;
        }
        item.activ_idx = index;
    } else {
        let sval = i32::from(item.sval);
        /* assign special activations that are now effects */
        if u32::from(item.tval) == TV_RING {
            if sval == iv.sv_ring_flames {
                item.activ_idx = act.ring_flames;
            }
            if sval == iv.sv_ring_acid {
                item.activ_idx = act.ring_acid;
            }
            if sval == iv.sv_ring_ice {
                item.activ_idx = act.ring_ice;
            }
            if sval == iv.sv_ring_lightning {
                item.activ_idx = act.ring_lightning;
            }
        }
        /* NOTE two activations are missed (don't have activation indexes) */
        /* white and black dragon */
        if u32::from(item.tval) == TV_DRAG_ARMOR {
            if sval == iv.sv_dragon_blue {
                item.activ_idx = act.dragon_blue;
            }
            if sval == iv.sv_dragon_red {
                item.activ_idx = act.dragon_red;
            }
            if sval == iv.sv_dragon_green {
                item.activ_idx = act.dragon_green;
            }
            if sval == iv.sv_dragon_multihued {
                item.activ_idx = act.dragon_multihued;
            }
            if sval == iv.sv_dragon_shining {
                item.activ_idx = act.dragon_shining;
            }
            if sval == iv.sv_dragon_law {
                item.activ_idx = act.dragon_law;
            }
            if sval == iv.sv_dragon_gold {
                item.activ_idx = act.dragon_gold;
            }
            if sval == iv.sv_dragon_chaos {
                item.activ_idx = act.dragon_chaos;
            }
            if sval == iv.sv_dragon_balance {
                item.activ_idx = act.dragon_balance;
            }
            if sval == iv.sv_dragon_power {
                item.activ_idx = act.dragon_power;
            }
        }
    }

    /* default the pval */
    if item.ident {
        item.pval = o.pval();
    }

    /* Rods are considered pval 1 if charged */
    if u32::from(item.tval) == TV_ROD {
        /* XXX There should be an obj_rod_charging() function for this logic */
        /* This was ripped from object/obj-desc.c */
        if item.iqty == 1 {
            item.pval = if real_item.timeout() != 0 { 0 } else { 1 };
        } else {
            let mut time_base = real_item.kind_time_min();
            if time_base == 0 {
                time_base = 1;
            }

            /*
             * Find out how many rods are charging, by dividing
             * current timeout by each rod's maximum timeout.
             * Ensure that any remainder is rounded up.  Display
             * very discharged stacks as merely fully discharged.
             */
            let power = (i32::from(real_item.timeout()).wrapping_add(time_base.wrapping_sub(1)))
                .wrapping_div(time_base);
            item.pval = if power < i32::from(item.iqty) { 1 } else { 0 };
        }
    } else if u32::from(item.tval) == TV_STAFF || u32::from(item.tval) == TV_WAND {
        /* Staffs & wands considered charged unless they are known empty */

        /* Assume good */
        item.pval = 1;

        /* if Known, get correct pval */
        if item.ident {
            item.pval = real_item.pval();
        }

        /* if seen {empty} assume pval 0 */
        if !item.aware && o.pval() == 0 {
            item.pval = 0;
        }
    }

    /* Kind index -- Only if we are aware of its kind */
    if item.aware {
        item.kind = o.kind_kidx();
    }

    if let Some(aidx) = o.artifact_aidx() {
        item.art_idx = aidx as u8;
    }

    if let Some(eidx) = o.ego_eidx() {
        item.ego_idx = eidx as u8;
    }

    /* Notice values */
    if item.ident {
        item.value = borg_object_value_known(iv, item);
    } else if item.aware {
        item.value = o.kind_cost();
    } else {
        item.value = borg_object_value_guess(item);
    }

    /* If it's not The One Ring, then it's worthless if cursed */
    if item.cursed && !item.one_ring {
        item.value = 0;
    }
}

/// Helper to see if an object kind does a certain thing
/// (`borg_obj_has_effect`): `index` is the effect index, `subtype` its
/// subtype (-1: any).
pub fn borg_obj_has_effect(kind: u32, index: i32, subtype: i32) -> bool {
    engine::kind_effects(kind as usize)
        .into_iter()
        .any(|(i, s)| i32::from(i) == index && (s == subtype || subtype == -1))
}

/// Does an ego with these kind flags have a random power or sustain? (the
/// test of `borg_ego_has_random_power`)
pub fn ego_kind_flags_have_random_power(kind_flags: &[u8]) -> bool {
    [KF_RAND_POWER, KF_RAND_SUSTAIN, KF_RAND_BASE_RES, KF_RAND_HI_RES, KF_RAND_RES_POWER]
        .into_iter()
        .any(|f| engine::flag_has(kind_flags, f as i32))
}

/// This checks if an item has a random power or sustain
/// (`borg_ego_has_random_power(&e_info[ego_idx])`).
pub fn borg_ego_has_random_power(ego_idx: usize) -> bool {
    ego_kind_flags_have_random_power(&engine::ego(ego_idx).kind_flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curse_names() {
        assert_eq!(borg_curse_index(b"vulnerability"), BORG_CURSE_VULNERABILITY);
        assert_eq!(borg_curse_index(b"air swing"), BORG_CURSE_AIR_SWING);
        assert_eq!(
            borg_curse_index(b"impair hitpoint recovery"),
            BORG_CURSE_IMPAIR_HITPOINT_RECOVERY
        );
        assert_eq!(borg_curse_index(b"Vulnerability"), BORG_CURSE_UNKNOWN);
        assert_eq!(borg_curse_index(b""), BORG_CURSE_UNKNOWN);
    }

    #[test]
    fn value_guess() {
        let mut it = crate::borg_item::blank_item();
        it.tval = TV_ROD as u8;
        assert_eq!(borg_object_value_guess(&it), 90);
        it.tval = TV_RING as u8;
        assert_eq!(borg_object_value_guess(&it), 45);
        it.modifiers[OBJ_MOD_SPEED as usize] = -1;
        assert_eq!(borg_object_value_guess(&it), 0);
        it.tval = TV_SWORD as u8;
        assert_eq!(borg_object_value_guess(&it), 20);
        it.to_d = -1;
        assert_eq!(borg_object_value_guess(&it), 0);
    }
}

//! Port of `borg-store-sell.c`: think about what you want to sell as well
//! as store at home.
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
//! The "what if" evaluations change `st.items.items` and the home wares in
//! place, call `borg_notice` / `borg_power` / `borg_notice_home` /
//! `borg_power_home`, and restore exactly what C restores, in C's order
//! (struct copies: a copied item's `note` keeps pointing into the
//! original's `desc`, as with `memcpy`).

use crate::borg_buy_strategy::{reserve410_unmet, retain_reserve410_item};
use crate::borg_home_notice::borg_notice_home;
use crate::borg_home_power::borg_power_home;
use crate::borg_inventory::borg_item_worth_id;
use crate::borg_io::{borg_keypress, borg_note_bytes, ESCAPE, KC_ENTER};
use crate::borg_item::inven_wield;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_power::borg_power;
use crate::borg_store::BORG_HOME;
use crate::engine;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_WORSHIPS_GOLD};
use crate::borg_headers::borg_item_h::{borg_item, BORG_CURSE_MAX};
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_AMMO_TVAL, BI_CLASS, BI_CLEVEL, BI_GOLD, BI_MAXCLEVEL, BI_MAXDEPTH,
    BI_MAXSP, BI_QUIVER_SLOTS,
};
use crate::headers::list_options_h::{OPT_birth_no_selling, OPT_birth_randarts};
use crate::headers::list_tvals_h::*;
use crate::headers::obj_properties_h::OBJ_MOD_MAX;
use crate::headers::object_h::{ELEM_MAX, EL_INFO_HATES, EL_INFO_IGNORE};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `CLASS_WARRIOR` (borg-trait.h).
const CLASS_WARRIOR: i32 = 0;

/// `sizeof(z_info->store_inven_max * sizeof(uint8_t))`: the size of a
/// `size_t`, which is all the two `memset`s clear (C bug 33).
const MEMSET_SIZE: usize = 8;

/// borg-store-sell.c globals.
pub struct StoreSellState {
    /// `sold_item_tval`
    pub sold_item_tval: [i32; 10],
    /// `sold_item_sval`
    pub sold_item_sval: [i32; 10],
    /// `sold_item_pval`
    pub sold_item_pval: [i32; 10],
    /// `sold_item_store`
    pub sold_item_store: [i32; 10],
    /// `sold_item_num`: the number of entries in use, newest included
    /// (better-borg-c milestone9 patch 0094)
    pub sold_item_num: i32,
    /// `sold_item_nxt`
    pub sold_item_nxt: i32,
    /// `test_item`: `z_info->store_inven_max` entries (empty = NULL).
    pub test_item: Vec<u8>,
    /// `best_item`: `z_info->store_inven_max` entries (empty = NULL).
    pub best_item: Vec<u8>,
}

/// `(sold_item_tval[p], sold_item_sval[p], sold_item_store[p])`; panics
/// past the 10 entries.
pub fn sold_item(st: &BorgState, p: usize) -> (i32, i32, i32) {
    let s = &st.store_sell;
    (s.sold_item_tval[p], s.sold_item_sval[p], s.sold_item_store[p])
}

impl Default for StoreSellState {
    fn default() -> Self {
        StoreSellState {
            sold_item_tval: [0; 10],
            sold_item_sval: [0; 10],
            sold_item_pval: [0; 10],
            sold_item_store: [0; 10],
            sold_item_num: -1,
            sold_item_nxt: 0,
            test_item: Vec::new(),
            best_item: Vec::new(),
        }
    }
}

/// `z_info->store_inven_max`
fn store_inven_max() -> i32 {
    i32::from(engine::z_info().store_inven_max)
}

/// `z_info->pack_size`
fn pack_size() -> i32 {
    i32::from(engine::z_info().pack_size)
}

/// `INVEN_TOTAL`
fn inven_total() -> i32 {
    inven_wield() + 12
}

/// `PACK_SLOTS`: the pack slots not taken by the quiver.
fn pack_slots(st: &BorgState) -> i32 {
    pack_size() - st.borg.tr(BI_QUIVER_SLOTS)
}

/// `OPT(player, birth_randarts)`
fn randarts() -> bool {
    engine::player_opt(OPT_birth_randarts)
}

/// `item->tval == TV_STAFF || item->tval == TV_WAND`
fn is_charged(item: &borg_item) -> bool {
    let tval = u32::from(item.tval);
    tval == TV_STAFF || tval == TV_WAND
}

/// `MIN(item->value, 30000)` (as `long`, stored in an `int32_t`).
fn capped_value(item: &borg_item) -> i32 {
    if item.value < 30000 {
        item.value
    } else {
        30000
    }
}

/// `borg_items[i]` (a mutable reference into the pack).
fn pack_item(st: &mut BorgState, i: i32) -> &mut borg_item {
    &mut st.items.items[i as usize]
}

/// `borg_shops[BORG_HOME].ware[n]` (a mutable reference).
fn home_ware(st: &mut BorgState, n: i32) -> &mut borg_item {
    &mut st.store.shops[BORG_HOME].ware[n as usize]
}

/// `weapon_swap && i == weapon_swap - 1` or the same for `armour_swap`:
/// slot `i` holds a swap item.
fn is_swap(st: &mut BorgState, i: i32) -> bool {
    let weapon_swap = st.trait_swap.weapon_swap;
    if weapon_swap != 0 && i == weapon_swap - 1 {
        return true;
    }
    let armour_swap = st.trait_swap.armour_swap;
    armour_swap != 0 && i == armour_swap - 1
}

/// Determine if an item can "absorb" a second item
///
/// See "object_absorb()" for the actual "absorption" code.
///
/// If permitted, we allow wands/staffs (if they are known to have equal
/// charges) and rods (if fully charged) to combine.
///
/// Note that rods/staffs/wands are then unstacked when they are used.
///
/// If permitted, we allow weapons/armor to stack, if they both known.
///
/// Food, potions, scrolls, and "easy know" items always stack.
///
/// Chests never stack (for various reasons).
///
/// We do NOT allow activatable items (artifacts or dragon scale mail)
/// to stack, to keep the "activation" code clean.  Artifacts may stack,
/// but only with another identical artifact (which does not exist).
///
/// Ego items may stack as long as they have the same ego-item type.
/// This is primarily to allow ego-missiles to stack.
fn borg_object_similar(st: &BorgState, o_ptr: &borg_item, j_ptr: &borg_item) -> bool {
    // NOTE: This assumes the giving of one item at a time
    let total = i32::from(o_ptr.iqty) + 1;

    // Require identical object types
    if o_ptr.kind != j_ptr.kind {
        return false;
    }

    // Different flags don't stack (of_is_equal: the whole flag set)
    if o_ptr.flags != j_ptr.flags {
        return false;
    }

    // Different elements don't stack
    let mask = (EL_INFO_HATES | EL_INFO_IGNORE) as u8;
    for i in 0..ELEM_MAX as usize {
        if o_ptr.el_info[i].res_level != j_ptr.el_info[i].res_level {
            return false;
        }
        if (o_ptr.el_info[i].flags & mask) != (j_ptr.el_info[i].flags & mask) {
            return false;
        }
    }

    // Analyze the items: does the "identical bonuses" block apply?
    let aware = o_ptr.aware && j_ptr.aware;
    let bonuses = match u32::from(o_ptr.tval) {
        // Chests: never okay
        TV_CHEST => return false,

        // Food and Potions and Scrolls: assume okay
        TV_FOOD | TV_POTION | TV_SCROLL => false,

        // Staffs and Wands: require knowledge (then as rods)
        TV_STAFF | TV_WAND => {
            if !aware {
                return false;
            }
            false
        }

        // Staffs and Wands and Rods: probably okay
        TV_ROD => false,

        // Weapons and Armor (fall through), Rings, Amulets, Lites:
        // require full knowledge of both items (fall through)
        TV_BOW | TV_DIGGING | TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOOTS | TV_GLOVES
        | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK | TV_SOFT_ARMOR | TV_HARD_ARMOR
        | TV_DRAG_ARMOR | TV_RING | TV_AMULET | TV_LIGHT => {
            if !aware {
                return false;
            }
            true
        }

        // Missiles
        TV_BOLT | TV_ARROW | TV_SHOT => true,

        // Various: require knowledge
        _ => {
            if !aware {
                return false;
            }
            false
        }
    };

    if bonuses {
        // Require identical "bonuses"
        if o_ptr.to_h != j_ptr.to_h {
            return false;
        }
        if o_ptr.to_d != j_ptr.to_d {
            return false;
        }
        if o_ptr.to_a != j_ptr.to_a {
            return false;
        }

        // Require identical modifiers
        for i in 0..OBJ_MOD_MAX as usize {
            if o_ptr.modifiers[i] != j_ptr.modifiers[i] {
                return false;
            }
        }

        // Require identical curses; note that curse powers are not
        // available so this is not the same as what object_similar() does
        if o_ptr.cursed != j_ptr.cursed || o_ptr.uncursable != j_ptr.uncursable {
            return false;
        }
        if o_ptr.cursed {
            for i in 0..BORG_CURSE_MAX as usize {
                if o_ptr.curses[i] != j_ptr.curses[i] {
                    return false;
                }
            }
        }

        // Require identical "artifact" names
        if o_ptr.art_idx != j_ptr.art_idx {
            return false;
        }

        // Require identical "ego-item" names
        if o_ptr.ego_idx != j_ptr.ego_idx {
            return false;
        }

        // HACK: Never stack "powerful" items (of_is_empty)
        if o_ptr.flags.iter().any(|&b| b != 0) || j_ptr.flags.iter().any(|&b| b != 0) {
            return false;
        }

        // Never stack recharging items
        if o_ptr.timeout != 0 || j_ptr.timeout != 0 {
            return false;
        }

        // Require identical "values"
        if o_ptr.ac != j_ptr.ac {
            return false;
        }
        if o_ptr.dd != j_ptr.dd {
            return false;
        }
        if o_ptr.ds != j_ptr.ds {
            return false;
        }
    }

    // Require identical "broken" status
    if o_ptr.ident != j_ptr.ident {
        return false;
    }

    // The stuff with 'note' is not right but it is close.  I think it
    // has him assuming that he can't stack sometimes when he can.  This
    // is alright, it just causes him to take a bit more time to do
    // some exchanges.
    // Require semi-matching "inscriptions"
    let o_note = o_ptr.note.is_some();
    let j_note = j_ptr.note.is_some();
    if (o_note && !j_note) || (!o_note && j_note) {
        return false;
    }

    if o_note && j_note {
        let a = crate::borg_item::item_note(st, o_ptr);
        let b = crate::borg_item::item_note(st, j_ptr);
        if !a.is_empty() && !b.is_empty() && a != b {
            return false;
        }

        // Normally require matching "inscriptions"
        if a != b {
            return false;
        }
    }

    // Maximal "stacking" limit
    if total >= engine::kind_max_stack(o_ptr.kind as usize) {
        return false;
    }

    // They match, so they must be similar
    true
}

/// Find the minimum amount of some item to buy/sell. For most
/// items this is 1, but for certain items (such as ammunition)
/// it may be higher.  -- RML
pub fn borg_min_item_quantity(st: &mut BorgState, item: &borg_item) -> i32 {
    // Only trade in bunches if sufficient cash
    if st.borg.tr(BI_GOLD) < 250 {
        return 1;
    }

    // Don't trade expensive items in bunches
    if item.value > 5 {
        return 1;
    }

    // Don't trade non-known items in bunches
    if !item.aware {
        return 1;
    }

    // Only allow some types
    match u32::from(item.tval) {
        TV_SHOT | TV_ARROW | TV_BOLT => {
            // Maximum number of items
            if item.iqty < 5 {
                return i32::from(item.iqty);
            }
            5
        }
        TV_FOOD => {
            if item.iqty < 3 {
                return i32::from(item.iqty);
            }
            3
        }
        _ => 1,
    }
}

fn borg_think_home_sell_bad(st: &mut BorgState, i: i32, borg_empty_home_power: i32) -> bool {
    let mut charge_each: i32 = 0;

    // Skip empty or unknown items
    {
        let item = pack_item(st, i);
        if item.iqty == 0 || !item.aware {
            return true;
        }
    }

    // Skip swap items
    if is_swap(st, i) {
        return true;
    }

    // Do not dump stuff at home that is not fully id'd and should be
    // this is good with random artifacts.
    {
        let item = pack_item(st, i);
        if randarts() && item.art_idx != 0 && !item.ident {
            return true;
        }

        // Ignore "worthless" items
        if item.value == 0 {
            return true;
        }
    }

    // If this item was just bought a the house, don't tell it back to
    // the house
    let bought_num = st.buy.bought_item_num;
    for p in 0..bought_num {
        let b = crate::borg_store_buy::bought_item(st, p as usize);
        let item = pack_item(st, i);
        if b.tval == i32::from(item.tval)
            && b.sval == i32::from(item.sval)
            && b.pval == i32::from(item.pval)
            && b.store == BORG_HOME as i32
        {
            return true;
        }
    }

    let copy = *pack_item(st, i);
    borg_notice_home(st, Some(&copy), false);
    if borg_power_home(st) <= borg_empty_home_power {
        return true;
    }

    // for wands and staffs adjust charges
    {
        let item = pack_item(st, i);
        if is_charged(item) {
            charge_each = i32::from(item.pval) / i32::from(item.iqty);
        }

        // assume one item sold.
        item.iqty = item.iqty.wrapping_sub(1);

        // for wands and staffs adjust charges
        if is_charged(item) {
            item.pval = (i32::from(item.pval) - charge_each) as i16;
        }
    }

    // Examine borg
    crate::borg_trait::borg_notice(st, true);

    // if this reduces the borgs power, it is a bad transaction
    if borg_power(st) < st.borg.power {
        let item = pack_item(st, i);
        item.iqty = item.iqty.wrapping_add(1);

        // for wands and staffs adjust charges
        if is_charged(item) {
            item.pval = (i32::from(item.pval) + charge_each) as i16;
        }

        return true;
    }
    let item = pack_item(st, i);
    item.iqty = item.iqty.wrapping_add(1);

    // for wands and staffs adjust charges
    if is_charged(item) {
        item.pval = (i32::from(item.pval) + charge_each) as i16;
    }

    false
}

/// this will see what single addition/substitution is best for the home.
fn borg_think_home_sell_best(st: &mut BorgState, best_home_power: &mut i32) {
    let sim = store_inven_max();
    let mut charge_each: i32 = 0;

    // get what an empty home would have for power
    borg_notice_home(st, None, true);
    let borg_empty_home_power = borg_power_home(st);

    // get the starting best (current)
    // Examine the home
    borg_notice_home(st, None, false);

    // Evaluate the home
    *best_home_power = borg_power_home(st);

    let mut first_empty = 0;
    while first_empty < sim {
        if home_ware(st, first_empty).iqty == 0 {
            break;
        }
        first_empty += 1;
    }
    if first_empty < sim {
        first_empty += 1;
    }

    // try individual substitutions/additions.
    for n in 0..first_empty {
        for i in 0..pack_size() {
            if borg_think_home_sell_bad(st, i, borg_empty_home_power) {
                continue;
            }

            // for wands and staffs adjust charges
            {
                let item = pack_item(st, i);
                if is_charged(item) {
                    charge_each = i32::from(item.pval) / i32::from(item.iqty);
                }
            }

            // stacking?
            let item = st.items.items[i as usize];
            if borg_object_similar(st, &st.store.shops[BORG_HOME].ware[n as usize], &item) {
                // if this stacks with what was previously here
                let item2 = home_ware(st, n);
                item2.iqty = item2.iqty.wrapping_add(1);
                if is_charged(&item) {
                    item2.pval = (i32::from(item2.pval) + charge_each) as i16;
                }
            } else {
                // eliminate items that would stack else where in the list.
                let mut found_match = false;
                for k in 0..sim {
                    if borg_object_similar(st, &st.store.safe_shops[BORG_HOME].ware[k as usize], &item) {
                        found_match = true;
                        break;
                    }
                }
                if found_match {
                    continue;
                }

                // replace current item with this item
                let item2 = home_ware(st, n);
                *item2 = item;

                // only move one into a non-stack slot
                item2.iqty = 1;
                if is_charged(&item) {
                    item2.pval = charge_each as i16;
                }
            }

            // remove item from pack
            {
                let item = pack_item(st, i);
                item.iqty = item.iqty.wrapping_sub(1);

                if is_charged(item) {
                    item.pval = (i32::from(item.pval) - charge_each) as i16;
                }
            }

            // Note the attempt
            st.store_sell.test_item[n as usize] = (i + sim) as u8;

            // Test to see if this is a good substitution.
            // Examine the home
            borg_notice_home(st, None, false);

            // Evaluate the home
            let home_power = borg_power_home(st);

            // Track best
            if home_power > *best_home_power {
                // Save the results
                for k in 0..sim as usize {
                    st.store_sell.best_item[k] = st.store_sell.test_item[k];
                }

                // Use it
                *best_home_power = home_power;
            }

            // restore stuff
            st.store.shops[BORG_HOME].ware[n as usize] =
                st.store.safe_shops[BORG_HOME].ware[n as usize];

            // put item back into pack
            let item = pack_item(st, i);
            item.iqty = item.iqty.wrapping_add(1);

            // for wands and staffs put the charges back too
            if is_charged(item) {
                item.pval = (i32::from(item.pval) + charge_each) as i16;
            }

            // put the item back in the test array
            st.store_sell.test_item[n as usize] = n as u8;
        }
    }
}

/// Step 1 -- sell "useful" things to the home (for later)
pub fn borg_think_home_sell_useful(st: &mut BorgState, best_home_power: &mut i32) -> bool {
    let sim = store_inven_max();

    // clear out our initial best/test objects
    st.store_sell.test_item[..MEMSET_SIZE].fill(0);
    st.store_sell.best_item[..MEMSET_SIZE].fill(0);

    // The home is full
    // and pack is full
    if home_ware(st, sim - 1).iqty != 0 && {
        let last = pack_slots(st) - 1;
        pack_item(st, last).iqty != 0
    } {
        return false;
    }

    // #410's reserve pass must not buy items that the home-best pass stores
    // immediately afterward (the rejected upstream v1 buy/store loop). Keep
    // carried reserve items in the pack until the deficit closes (#25).
    if reserve410_unmet(st) {
        for i in 0..pack_slots(st) {
            if retain_reserve410_item(st, &st.items.items[i as usize]) {
                return false;
            }
        }
    }

    // clear test arrays (test[i] == i is no change)
    for i in 0..sim as usize {
        st.store_sell.test_item[i] = i as u8;
        st.store_sell.best_item[i] = st.store_sell.test_item[i];
    }

    *best_home_power = -1;

    // find best combo for home.
    borg_think_home_sell_best(st, best_home_power);

    // restore bonuses and such
    for i in 0..sim as usize {
        st.store.shops[BORG_HOME].ware[i] = st.store.safe_shops[BORG_HOME].ware[i];
    }

    for i in 0..inven_total() as usize {
        st.items.items[i] = st.items.safe_items[i];
    }

    crate::borg_trait::borg_notice(st, true);

    // Drop stuff that will stack in the home
    for i in 0..sim {
        // if this is not the item that was there,
        // drop off the item that replaces it.
        let best = i32::from(st.store_sell.best_item[i as usize]);
        if best != i && best != 255 {
            let item = st.items.items[(best - sim) as usize];
            let item2 = st.store.shops[BORG_HOME].ware[i as usize];

            // if this item is not the same as what was
            // there before take it.
            if !borg_object_similar(st, &item2, &item) {
                continue;
            }

            // There are limted quantity per slot
            if item2.iqty > 90 {
                continue;
            }

            st.borg.goal.shop = BORG_HOME as i16;
            st.borg.goal.item = (best - sim) as i16;

            return true;
        }
    }

    // Get rid of stuff in house but not in 'best' house if
    // pack is not full
    let last = pack_slots(st) - 1;
    if pack_item(st, last).iqty == 0 {
        for i in 0..sim {
            // if this is not the item that was there,
            // get rid of the item that was there
            let best = i32::from(st.store_sell.best_item[i as usize]);
            if best != i && home_ware(st, i).iqty != 0 {
                let item = st.items.items[(best - sim) as usize];
                let item2 = st.store.shops[BORG_HOME].ware[i as usize];

                // if this item is not the same as what was
                // there before take it.
                if borg_object_similar(st, &item, &item2) {
                    continue;
                }

                // skip stuff if we sold bought it
                // skip stuff if we sold/bought it
                let s = &st.store_sell;
                for p in 0..s.sold_item_num {
                    let p = p as usize;
                    if s.sold_item_tval[p] == i32::from(item2.tval)
                        && s.sold_item_sval[p] == i32::from(item2.sval)
                        && s.sold_item_store[p] == BORG_HOME as i32
                    {
                        return false;
                    }
                }

                st.borg.goal.shop = BORG_HOME as i16;
                st.borg.goal.ware = i as i16;

                return true;
            }
        }
    }

    // Drop stuff that is in best house but currently in inventory
    for i in 0..sim {
        // if this is not the item that was there,
        // drop off the item that replaces it.
        let best = i32::from(st.store_sell.best_item[i as usize]);
        if best != i && best != 255 {
            // hack don't sell
            if pack_item(st, best - sim).iqty == 0 {
                return false;
            }

            st.borg.goal.shop = BORG_HOME as i16;
            st.borg.goal.item = (best - sim) as i16;

            return true;
        }
    }

    // Return our num_ counts to normal
    borg_notice_home(st, None, false);

    // Assume not
    false
}

/// scan inventory and equipment to see if we have
/// more than one of this (`borg_has_mutiple(&borg_items[idx])`)
fn borg_has_mutiple(st: &mut BorgState, idx: i32) -> bool {
    let in_item = st.items.items[idx as usize];
    if in_item.iqty > 1 {
        return true;
    }

    // Some types of items we only want to count as a dupe in a stack
    // for example two unIDd swords may look the same (both rapiers)
    // but have different powers
    if !in_item.ident {
        match u32::from(in_item.tval) {
            TV_BOOTS | TV_GLOVES | TV_HELM | TV_CROWN | TV_SHIELD | TV_SOFT_ARMOR
            | TV_HARD_ARMOR | TV_SHOT | TV_BOLT | TV_ARROW | TV_BOW | TV_DIGGING | TV_HAFTED
            | TV_POLEARM | TV_SWORD => return false,
            _ => {}
        }
    }

    // check the pack
    for i in 0..pack_size() {
        if i == idx {
            continue;
        }
        let item = &st.items.items[i as usize];
        if item.tval == in_item.tval && item.sval == in_item.sval && item.iqty != 0 {
            return true;
        }
    }

    // check equipment
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];
        if item.tval == in_item.tval && item.sval == in_item.sval && item.iqty != 0 {
            return true;
        }
    }
    false
}

fn borg_store_buys(item: &borg_item, who: i32) -> bool {
    let tval = u32::from(item.tval);
    match who + 1 {
        // General Store
        1 => matches!(
            tval,
            TV_FOOD | TV_MUSHROOM | TV_FLASK | TV_SHOT | TV_BOLT | TV_ARROW | TV_DIGGING
        ),

        // Armory
        2 => matches!(
            tval,
            TV_BOOTS
                | TV_GLOVES
                | TV_HELM
                | TV_CROWN
                | TV_SHIELD
                | TV_SOFT_ARMOR
                | TV_HARD_ARMOR
                | TV_DRAG_ARMOR
        ),

        // Weapon Shop
        3 => matches!(
            tval,
            TV_SHOT | TV_BOLT | TV_ARROW | TV_BOW | TV_DIGGING | TV_HAFTED | TV_POLEARM | TV_SWORD
        ),

        // Bookstore
        4 => matches!(
            tval,
            TV_PRAYER_BOOK | TV_MAGIC_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK
        ),

        // book store --Alchemist
        5 => matches!(tval, TV_SCROLL | TV_POTION),

        // Magic Shop
        6 => matches!(
            tval,
            TV_AMULET | TV_RING | TV_STAFF | TV_WAND | TV_ROD | TV_MAGIC_BOOK
        ),

        // Black Market
        7 => {
            // if we don't get money on sales, allow anything to be
            // sold to the black market
            if engine::player_opt(OPT_birth_no_selling) {
                return true;
            }

            matches!(tval, TV_LIGHT | TV_CLOAK | TV_FOOD)
        }
        _ => false,
    }
}

/// Determine if an item should be sold (`borg_good_sell(&borg_items[i], who)`)
fn borg_good_sell(st: &mut BorgState, i: i32, who: i32) -> bool {
    let item = st.items.items[i as usize];
    // do I have more than one, even if not stacked
    // only used for non-IDd items
    let mut multiple = false;
    let tval = u32::from(item.tval);

    // Never attempt to sell worthless items, shops won't buy them
    if item.value <= 0 {
        // except unidentified potions and scrolls.  Since these can't be IDd,
        // best to sell them
        if !((tval == TV_POTION || tval == TV_SCROLL) && !item.ident) {
            return false;
        }
    }

    // make sure we are in the shop that buys this
    if !borg_store_buys(&item, who) {
        return false;
    }

    // Never sell valuable non-id'd items
    // unless you have a stack, in which case, sell one to ID them
    if borg_item_note_needs_id(st, &item) {
        // note if we have more than one
        multiple = borg_has_mutiple(st, i);
        if !multiple {
            return false;
        }
    }

    // Worshipping gold or scumming will allow the sale
    let scum = st.cfg(BORG_MONEY_SCUM_AMOUNT);
    if item.value > 0
        && ((st.cfg(BORG_WORSHIPS_GOLD) != 0 || st.borg.tr(BI_MAXCLEVEL) < 10)
            || (scum < st.borg.tr(BI_GOLD) && scum != 0))
    {
        // Borg is allowed to continue in this routine to sell
    } else {
        // Some items must be ID, or at least 'known'
        // Analyze the type
        match tval {
            TV_POTION | TV_SCROLL => {
                // Always sell potions and scrolls, it is the only way to ID other
                // than using
                if !item.ident {
                    return true;
                }

                // Spell casters should not sell ResMana to shop unless
                // they have tons in the house
                if tval == TV_POTION
                    && i32::from(item.sval) == st.item_val.sv_potion_restore_mana
                    && st.borg.tr(BI_MAXSP) > 100
                    && st.borg.has[st.item_val.kv_potion_restore_mana as usize]
                        + i32::from(st.home.num_mana)
                        > 99
                {
                    return false;
                }
            }

            // Never sell if not "known"
            // unless we have more than one or are deep
            TV_FOOD | TV_ROD | TV_WAND | TV_STAFF | TV_RING | TV_AMULET | TV_LIGHT
                if borg_item_worth_id(st, &item) && st.borg.tr(BI_MAXDEPTH) < 35 && !multiple =>
            {
                return false;
            }

            // Only sell "known" items (unless we have more than one)
            TV_BOW | TV_DIGGING | TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOOTS | TV_GLOVES
            | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK | TV_SOFT_ARMOR | TV_HARD_ARMOR
            | TV_DRAG_ARMOR
                if borg_item_worth_id(st, &item) && !multiple =>
            {
                return false;
            }
            _ => {}
        }
    }

    // Do not sell stuff that is not fully id'd and should be
    if randarts() && item.art_idx != 0 && !item.ident {
        // CHECK THE ARTIFACTS
        // For now check all artifacts
        return false;
    }

    // Do not sell stuff that is not fully id'd and should be
    if !item.ident
        && item.ego_idx != 0
        && item.iqty < 2
        && borg_ego_has_random_power(usize::from(item.ego_idx))
    {
        return false;
    }

    // Do not sell it if I just bought one
    // do not buy the item if I just sold it.
    let bought_num = st.buy.bought_item_num;
    for p in 0..bought_num {
        let b = crate::borg_store_buy::bought_item(st, p as usize);
        if b.tval == i32::from(item.tval)
            && b.sval == i32::from(item.sval)
            && (b.store == who || who != BORG_HOME as i32)
        {
            return false;
        }
    }

    true
}

/// Step 2 -- sell "useless" items to a shop (for cash)
pub fn borg_think_shop_sell_useless(st: &mut BorgState) -> bool {
    let icky = (store_inven_max() - 1) as usize;

    let mut b_k: i32 = -1;
    let mut b_i: i32 = -1;

    let mut fix = false;

    // Evaluate
    let mut b_p: i32 = st.borg.power;
    let mut b_c: i32 = 30001;

    // Check each shop
    for k in 0..(i32::from(engine::z_info().store_max) - 1) {
        // Skip "full" shops
        if st.store.shops[k as usize].ware[icky].iqty != 0 {
            continue;
        }

        // Sell stuff
        for i in 0..pack_size() {
            let item = *pack_item(st, i);
            let tval = u32::from(item.tval);
            let sval = i32::from(item.sval);

            // Skip empty items
            if item.iqty == 0 {
                continue;
            }

            // Skip some important type items
            if i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL) && st.borg.tr(BI_AMISSILES) < 45 {
                continue;
            }
            if tval == TV_ROD
                && sval == st.item_val.sv_rod_healing
                && st.borg.has[st.item_val.kv_rod_healing as usize] <= 3
            {
                continue;
            }

            if st.borg.tr(BI_CLASS) == CLASS_WARRIOR
                && tval == TV_ROD
                && sval == st.item_val.sv_rod_mapping
                && item.iqty <= 2
            {
                continue;
            }

            // Do not sell our attack wands if they still have charges
            if tval == TV_WAND
                && st.borg.tr(BI_CLEVEL) < 35
                && (sval == st.item_val.sv_wand_magic_missile
                    || sval == st.item_val.sv_wand_stinking_cloud
                    || sval == st.item_val.sv_wand_annihilation)
                && item.pval != 0
            {
                continue;
            }

            // don't sell our swap items
            if is_swap(st, i) {
                continue;
            }

            // Skip "bad" sales
            if !borg_good_sell(st, i, k) {
                continue;
            }

            let (ku, iu) = (k as usize, i as usize);

            // Give the item to the shop
            st.store.shops[ku].ware[icky] = st.items.safe_items[iu];

            // get the quantity
            let copy = st.items.items[iu];
            let qty = borg_min_item_quantity(st, &copy);

            // Give "qty" items
            st.store.shops[ku].ware[icky].iqty = qty as u8;

            // Lose a single item
            {
                let it = &mut st.items.items[iu];
                it.iqty = (i32::from(it.iqty) - qty) as u8;
            }

            // for wands and staffs adjust charges
            if is_charged(&st.items.items[iu]) {
                let safe = &st.items.safe_items[iu];
                let charge_loss = i32::from(safe.pval) / i32::from(safe.iqty);
                st.store.shops[ku].ware[icky].pval = (qty * charge_loss) as i16;
                let it = &mut st.items.items[iu];
                it.pval = (i32::from(it.pval) - qty * charge_loss) as i16;
            }

            // Fix later
            fix = true;

            // Examine the inventory
            crate::borg_trait::borg_notice(st, true);

            // Evaluate the inventory
            let p = borg_power(st);

            // Restore the item
            st.items.items[iu] = st.items.safe_items[iu];

            // Restore the store item
            st.store.shops[ku].ware[icky] = st.store.safe_shops[ku].ware[icky];

            // Ignore "bad" sales
            if p < b_p {
                continue;
            }

            // Extract the "price"
            let c = capped_value(&st.items.items[iu]);

            // sell cheap items first.  This is done because we may have to
            // buy the item back in some very strange circumstances.
            if p == b_p && c >= b_c {
                continue;
            }

            // Maintain the "best"
            b_k = k;
            b_i = i;
            b_p = p;
            b_c = c;
        }
    }

    // Examine the inventory
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    // Sell something (if useless)
    if b_k >= 0 && b_i >= 0 {
        // Visit that shop
        st.borg.goal.shop = b_k as i16;

        // Sell that item
        st.borg.goal.item = b_i as i16;

        // Success
        return true;
    }

    // Assume not
    false
}

/// Sell items to the current shop, if desired
pub fn borg_think_shop_sell(st: &mut BorgState) -> bool {
    // Sell something if requested
    if st.borg.goal.shop == st.think.shop_num && st.borg.goal.item >= 0 {
        let idx = st.borg.goal.item as usize;
        let item = st.items.items[idx];

        let qty = borg_min_item_quantity(st, &item);

        // Log
        let mut note = b"# Selling ".to_vec();
        note.extend(item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8));
        borg_note_bytes(st, &note);

        // Sell an item
        borg_keypress(st, keycode_t::from(b's'));

        // Sell the desired item
        let letter = engine::all_letters_nohjkl(i32::from(st.borg.goal.item)) as i8 as keycode_t;
        borg_keypress(st, letter);

        // Sell a single item
        if st.items.items[idx].iqty > 1 || qty >= 2 {
            if qty == 5 {
                borg_keypress(st, keycode_t::from(b'5'));
            }
            if qty == 4 {
                borg_keypress(st, keycode_t::from(b'4'));
            }
            if qty == 3 {
                borg_keypress(st, keycode_t::from(b'3'));
            }
            if qty == 2 {
                borg_keypress(st, keycode_t::from(b'2'));
            }
            borg_keypress(st, KC_ENTER);
        }

        // Mega-Hack -- Accept the price
        if st.borg.goal.shop != BORG_HOME as i16 {
            borg_keypress(st, KC_ENTER);
            borg_keypress(st, ESCAPE);
            borg_keypress(st, ESCAPE);
            borg_keypress(st, ESCAPE);
        }

        // Mark our last item sold
        let item = st.items.items[idx];
        let shop = i32::from(st.borg.goal.shop);
        let s = &mut st.store_sell;
        if s.sold_item_nxt >= 9 {
            s.sold_item_nxt = 0;
        }
        let nxt = s.sold_item_nxt as usize;
        s.sold_item_pval[nxt] = i32::from(item.pval);
        s.sold_item_tval[nxt] = i32::from(item.tval);
        s.sold_item_sval[nxt] = i32::from(item.sval);
        s.sold_item_store[nxt] = shop;
        s.sold_item_nxt += 1;
        if s.sold_item_num < s.sold_item_nxt {
            s.sold_item_num = s.sold_item_nxt;
        }

        // The purchase is complete
        st.borg.goal.item = -1;
        st.borg.goal.ware = st.borg.goal.item;
        st.borg.goal.shop = st.borg.goal.ware;

        // Increment our anti-bounce count to avoid loops
        st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(1);

        // I'm not in a store
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        st.borg.in_shop = false;
        // Success
        return true;
    }

    // Nope
    false
}

/// `(borg.trait[BI_GOLD] / 100L) + add`, stored in an `int32_t`.
fn gold_greed(st: &BorgState, add: i64) -> i32 {
    (i64::from(st.borg.tr(BI_GOLD)) / 100 + add) as i32
}

/// Count the number of items worth "selling"
///
/// This determines the choice of stairs.
pub fn borg_count_sell(st: &mut BorgState) -> i32 {
    let mut k: i32 = 0;

    // Calculate "greed" factor
    let mut greed = gold_greed(st, 100);

    // Minimal greed
    greed = greed.clamp(1000, 25000);
    if st.borg.tr(BI_MAXDEPTH) >= 50 {
        greed = 75000;
    }
    if st.borg.tr(BI_CLEVEL) < 25 {
        greed = gold_greed(st, 50);
    }
    if st.borg.tr(BI_CLEVEL) < 20 {
        greed = gold_greed(st, 35);
    }
    if st.borg.tr(BI_CLEVEL) < 15 {
        greed = gold_greed(st, 20);
    }
    if st.borg.tr(BI_CLEVEL) < 13 {
        greed = gold_greed(st, 10);
    }
    if st.borg.tr(BI_CLEVEL) < 10 {
        greed = gold_greed(st, 5);
    }
    if st.borg.tr(BI_CLEVEL) < 5 {
        greed = gold_greed(st, 0);
    }

    // Count "sellable" items
    for i in 0..pack_size() {
        let item = *pack_item(st, i);
        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);

        // Skip empty items
        if item.iqty == 0 {
            continue;
        }

        // Skip "crappy" items
        if item.value <= 0 {
            continue;
        }

        // skip our swap weapon
        if is_swap(st, i) {
            continue;
        }

        // Don't sell my ammo
        if i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL) {
            continue;
        }

        // Don't sell my books
        if engine::obj_kind_can_browse(item.kind as usize) {
            continue;
        }

        // Never sell valuable non-id'd items
        // unless you have a stack, in which case, sell one to ID them
        if borg_item_note_needs_id(st, &item) {
            // note if we have more than one
            if !borg_has_mutiple(st, i) {
                continue;
            }
        }

        // Don't sell my needed potion/wands/staff/scroll collection
        let iv = &st.item_val;
        if (tval == TV_POTION && sval == iv.sv_potion_cure_serious)
            || (tval == TV_POTION && sval == iv.sv_potion_cure_critical)
            || (tval == TV_POTION && sval == iv.sv_potion_healing)
            || (tval == TV_POTION && sval == iv.sv_potion_star_healing)
            || (tval == TV_POTION && sval == iv.sv_potion_life)
            || (tval == TV_POTION && sval == iv.sv_potion_speed)
            || (tval == TV_STAFF && sval == iv.sv_staff_teleportation)
            || (tval == TV_WAND && sval == iv.sv_wand_drain_life)
            || (tval == TV_WAND && sval == iv.sv_wand_annihilation)
            || (tval == TV_SCROLL && sval == iv.sv_scroll_teleport)
        {
            continue;
        }
        // Obtain the base price
        let price = capped_value(&item);

        // Skip cheap "known" (or "average") items
        if price * i32::from(item.iqty) < greed && !borg_item_note_needs_id(st, &item) {
            continue;
        }

        // only mark things as sellable if getting rid of them doesn't reduce
        // our power much
        let sv_qty = item.iqty;
        pack_item(st, i).iqty = 0;
        crate::borg_trait::borg_notice(st, true);
        let p = borg_power(st);
        pack_item(st, i).iqty = sv_qty;

        if p.wrapping_add(50) < st.borg.power {
            continue;
        }

        // Count remaining items
        k += 1;
    }
    // reset the results
    crate::borg_trait::borg_notice(st, true);

    // Result
    k
}

/// Allocate `test_item` / `best_item` (`borg_init_store_sell`).
pub fn borg_init_store_sell(st: &mut BorgState) {
    borg_free_store_sell(st);

    let sim = usize::from(engine::z_info().store_inven_max);
    st.store_sell.test_item = vec![0; sim];
    st.store_sell.best_item = vec![0; sim];
}

/// Free `test_item` / `best_item` (`borg_free_store_sell`).
pub fn borg_free_store_sell(st: &mut BorgState) {
    st.store_sell.test_item = Vec::new();
    st.store_sell.best_item = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> borg_item {
        crate::borg_item::blank_item()
    }

    #[test]
    fn store_buys_tables() {
        let mut it = blank();
        it.tval = TV_FLASK as u8;
        assert!(borg_store_buys(&it, 0));
        assert!(!borg_store_buys(&it, 1));
        it.tval = TV_MAGIC_BOOK as u8;
        assert!(borg_store_buys(&it, 3));
        assert!(borg_store_buys(&it, 5));
        assert!(!borg_store_buys(&it, 7));
        assert!(!borg_store_buys(&it, -1));
        it.tval = TV_DRAG_ARMOR as u8;
        assert!(borg_store_buys(&it, 1));
        assert!(!borg_store_buys(&it, 2));
    }

    #[test]
    fn default_state_matches_c_initialisers() {
        let s = StoreSellState::default();
        assert_eq!(s.sold_item_num, -1);
        assert_eq!(s.sold_item_nxt, 0);
        assert!(s.test_item.is_empty() && s.best_item.is_empty());
    }
}

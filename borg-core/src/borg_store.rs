//! Port of `borg-store.c`: definitions of stores (shops) and store items.
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

use crate::engine::{self, StoreObject};
use crate::borg_headers::borg_item_h::borg_item;
use crate::headers::list_options_h::OPT_birth_no_selling;
use crate::headers::list_terrain_h::{FEAT_HOME, FEAT_STORE_BLACK};
use crate::headers::obj_desc_h::{ODESC_FULL, ODESC_STORE};
use crate::state::BorgState;
use crate::borg_item::blank_item;

/// `BORG_HOME` (borg-store.h): the home's index in the shops.
pub const BORG_HOME: usize = 7;

/// `SHOP_MENU_ITEMS`: the menu letter of each store slot (with C's
/// terminating NUL).
pub const SHOP_MENU_ITEMS: &[u8] = b"acfhjmnoqruvyzABDFGHJKLMNOPQRSTUVWXYZ\0";

/// One store as the Borg sees it (`borg_shop`).
///
/// The `Vec` is only (re)allocated by `borg_init_store` / `borg_free_store`.
#[derive(Default)]
pub struct BorgShop {
    /// `z_info->store_inven_max` wares.
    pub ware: Vec<borg_item>,
}

/// The stores (`borg_shops`) and the copy taken after each read
/// (`borg_safe_shops`). Empty = NULL (not allocated).
#[derive(Default)]
pub struct StoreState {
    pub shops: Vec<BorgShop>,
    pub safe_shops: Vec<BorgShop>,
}

/// check for the a full home
pub fn borg_home_full(st: &mut BorgState) -> bool {
    // must have a free inventory slot and a free home slot
    let last = usize::from(engine::z_info().store_inven_max) - 1;
    st.store.shops[BORG_HOME].ware[last].iqty != 0
}

/// check for the a full inventory
pub fn borg_inventory_full(st: &mut BorgState) -> bool {
    crate::borg_inventory::borg_first_empty_inventory_slot(st) == -1
}

/// `f_info[feat].shopnum - 1`: the store index of a shop feature.
fn feat_store(feat: u32) -> i32 {
    i32::from(engine::feat_shopnum(feat as usize)) - 1
}

/// `MIN(object_value_real(obj, qty), object_value(obj, qty))`, evaluated
/// as the C macro does (`(a > b) ? b : a`: both, then the smaller again).
fn min_value(obj: StoreObject, qty: i32) -> i32 {
    if obj.value_real(qty) > obj.value(qty) {
        obj.value(qty)
    } else {
        obj.value_real(qty)
    }
}

fn borg_price_item(obj: StoreObject, store_buying: bool, qty: i32, this_store: i32) -> i32 {
    let mut adjust: i32 = 100;
    let mut price: i32;

    if this_store == feat_store(FEAT_HOME) {
        return 0;
    }

    // Get the value of the stack of wands, or a single item
    if obj.can_have_charges() {
        price = min_value(obj, qty);
    } else {
        price = min_value(obj, 1);
    }

    // Worthless items
    if price <= 0 {
        return 0;
    }

    // The black market is always a worse deal
    if this_store == feat_store(FEAT_STORE_BLACK) {
        adjust = 150;
    }

    // Shop is buying
    if store_buying {
        // Set the factor
        adjust = 100 + (100 - adjust);
        if adjust > 100 {
            adjust = 100;
        }

        // Shops now pay 2/3 of true value
        price = price * 2 / 3;

        // Black market sucks
        if this_store == feat_store(FEAT_STORE_BLACK) {
            price /= 2;
        }

        // Check for no_selling option
        if engine::player_opt(OPT_birth_no_selling) {
            return 0;
        }
    } else {
        // Re-evaluate if we're selling
        if obj.can_have_charges() {
            price = obj.value_real(qty);
        } else {
            price = obj.value_real(1);
        }

        // Black market sucks
        if this_store == feat_store(FEAT_STORE_BLACK) {
            price *= 2;
        }
    }

    // Compute the final price (with rounding): `int * int`, then `long`
    price = ((i64::from(price * adjust) + 50) / 100) as i32;

    // Now convert price to total price for non-wands
    if !obj.can_have_charges() {
        price *= qty;
    }

    // Now limit the price to the purse limit
    // (`proprietor = stores[this_store].owner`)
    if store_buying && price > engine::store_owner_max_cost(this_store as usize) * qty {
        price = engine::store_owner_max_cost(this_store as usize) * qty;
    }

    // Note -- Never become "free"
    if price <= 0 {
        return qty;
    }

    // Return the price
    price
}

/// Cheat the "Store" screen
pub fn borg_notice_store(st: &mut BorgState) {
    // Scan each store
    for store_num in 0..usize::from(engine::z_info().store_max) {
        // The store's wares and the two tables the analyzer reads are
        // borrowed as separate fields of the state, next to the access
        // token the store objects borrow (docs/SELF-REVIEW.md L4).
        let BorgState { access, store, act, item_val, .. } = &mut *st;

        // Clear the Inventory from memory
        store.shops[store_num].ware.fill(blank_item());

        let list = engine::store_stock(access, store_num);

        // Check each existing object in this store
        for (slot, o_ptr) in list.into_iter().enumerate() {
            // Describe the item
            let buf = o_ptr.desc120(ODESC_FULL | ODESC_STORE);
            if buf == b"(nothing)" {
                break;
            }

            // Analyze the item
            // (`borg_item_analyze(&borg_shops[shop].ware[slot], obj, desc, in_store)`)
            crate::borg_item_analyze::borg_item_analyze(
                &mut store.shops[store_num].ware[slot],
                crate::borg_item::NoteOrigin::Ware { shop: store_num as u8, slot: slot as u8 },
                o_ptr.into(),
                &buf,
                store_num != BORG_HOME,
                act,
                item_val,
            );

            // Save the declared cost
            store.shops[store_num].ware[slot].cost = borg_price_item(o_ptr, false, 1, store_num as i32);
        }

        // save a backup copy of the store.  This list should never change
        let StoreState { shops, safe_shops } = &mut st.store;
        safe_shops[store_num].ware.copy_from_slice(&shops[store_num].ware);
    }
}

/// Allocate the shops (`borg_init_store`).
pub fn borg_init_store(st: &mut BorgState) {
    borg_free_store(st);

    let home = feat_store(FEAT_HOME);
    if home != BORG_HOME as i32 {
        let text = format!(
            "borg thinks home is {}, game thinks home is {}.  aborting. ",
            home, BORG_HOME
        );
        engine::msg(st, text.as_bytes());
        st.init.init_failure = true;
    }

    let z = engine::z_info();
    let shops = || -> Vec<BorgShop> {
        (0..z.store_max)
            .map(|_| BorgShop {
                ware: vec![blank_item(); usize::from(z.store_inven_max)],
            })
            .collect()
    };

    // Make the stores in the town
    st.store.shops = shops();

    // Make the "safe" stores in the town
    st.store.safe_shops = shops();

    crate::borg_store_sell::borg_init_store_sell(st);
}

/// Free the shops (`borg_free_store`).
pub fn borg_free_store(st: &mut BorgState) {
    crate::borg_store_sell::borg_free_store_sell(st);

    st.store.safe_shops = Vec::new();
    st.store.shops = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shop_menu_items_match_c() {
        assert_eq!(SHOP_MENU_ITEMS.len(), 38);
        assert_eq!(SHOP_MENU_ITEMS[37], 0);
        assert_eq!(&SHOP_MENU_ITEMS[..5], b"acfhj");
    }
}

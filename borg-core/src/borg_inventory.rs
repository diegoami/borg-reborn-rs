//! Port of `borg-inventory.c`: reading the pack, the equipment and the
//! quiver, and simple slot lookups.
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

use crate::borg_item::{blank_item, borg_deinscribe, inven_wield, quiver_start};
use crate::borg_item_analyze::borg_item_analyze;
use crate::engine::{self, CarriedObject};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::BI_QUIVER_SLOTS;
use crate::headers::list_tvals_h::*;
use crate::state::BorgState;

/// borg-inventory.c globals.
#[derive(Default)]
pub struct InventoryState {
    /// `borg_do_crush_junk`: the pack changed, look for junk to crush.
    pub do_crush_junk: bool,
}

/// Return the slot that items of the given type are wielded into, or -1
/// if the item cannot be wielded (`borg_wield_slot`).
pub fn borg_wield_slot(item: &borg_item) -> i32 {
    let w = inven_wield();
    match u32::from(item.tval) {
        TV_SWORD | TV_POLEARM | TV_HAFTED | TV_DIGGING => w,
        TV_DRAG_ARMOR | TV_HARD_ARMOR | TV_SOFT_ARMOR => w + 6, // INVEN_BODY
        TV_SHIELD => w + 8,                                     // INVEN_ARM
        TV_CROWN | TV_HELM => w + 9,                            // INVEN_HEAD
        TV_BOW => w + 1,                                        // INVEN_BOW
        TV_RING => w + 3,                                       // INVEN_LEFT
        TV_AMULET => w + 4,                                     // INVEN_NECK
        TV_LIGHT => w + 5,                                      // INVEN_LIGHT
        TV_CLOAK => w + 7,                                      // INVEN_OUTER
        TV_GLOVES => w + 10,                                    // INVEN_HANDS
        TV_BOOTS => w + 11,                                     // INVEN_FEET
        /* No slot available */
        _ => -1,
    }
}

/// Find the pack slot of an aware item with the given tval/sval, or -1
/// (`borg_slot`). Prefers the smallest pile; the "largest pval" test is
/// dead code in C (see docs/c-bugs.md) and kept.
pub fn borg_slot(st: &mut BorgState, tval: i32, sval: i32) -> i32 {
    let mut n: i32 = -1;
    let items = &st.items.items;

    /* Scan the pack */
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = &items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip un-aware items */
        if !item.aware {
            continue;
        }

        /* Require correct tval */
        if i32::from(item.tval) != tval {
            continue;
        }

        /* Require correct sval */
        if i32::from(item.sval) != sval {
            continue;
        }

        /* Prefer smallest pile */
        if n >= 0 && item.iqty > items[n as usize].iqty {
            continue;
        }

        /* Prefer largest "pval" (even if smaller pile) */
        if n >= 0 && item.pval < items[n as usize].pval && item.iqty > items[n as usize].iqty {
            continue;
        }

        /* Save this item */
        n = i;
    }

    n
}

/// Analyze one carried object into slot `i` (no price), then remove a `!`
/// or `borg` inscription (the equipment and quiver readers).
/// The engine object comes from `get(&st.access)` (its handle borrows the
/// state's access token, so it is made here, next to the split borrows of
/// the analyzed slot: docs/SELF-REVIEW.md L4).
fn analyze_and_deinscribe(
    st: &mut BorgState,
    i: i32,
    get: impl for<'a> Fn(&'a engine::Access) -> Option<CarriedObject<'a>>,
) {
    let BorgState { access, items, act, item_val, .. } = st;
    /* skip non items */
    let Some(obj) = get(access) else { return };
    /* Describe it */
    let desc = engine::object_desc_full(obj);
    /* Analyze the item (no price) */
    borg_item_analyze(
        &mut items.items[i as usize],
        crate::borg_item::NoteOrigin::Item(i as u16),
        obj.into(),
        &desc,
        false,
        act,
        item_val,
    );
    deinscribe_if_marked(st, i);
}

/// `if (borg_items[i].note && (strstr(note, "!") || strstr(note, "borg")))
/// borg_deinscribe(i);` (a NULL note reads as "", which matches neither).
fn deinscribe_if_marked(st: &mut BorgState, i: i32) {
    let note = crate::borg_item::borg_get_note(st, i as usize);
    if note.contains(&b'!') || note.windows(4).any(|w| w == b"borg") {
        borg_deinscribe(st, i);
    }
}

/// Cheat the quiver part of the equipment (`borg_cheat_quiver`).
fn borg_cheat_quiver(st: &mut BorgState) {
    let quiver_size = i32::from(engine::z_info().quiver_size);
    for j in 0..quiver_size {
        let i = quiver_start() + j;
        st.items.items[i as usize] = blank_item();

        analyze_and_deinscribe(st, i, |a| engine::quiver_object(a, j as usize));
    }
}

/// Cheat the "equip" screen (`borg_equipment`), then the quiver.
pub fn borg_equipment(st: &mut BorgState) {
    let pack_size = i32::from(engine::z_info().pack_size);
    let count = i32::from(engine::body_count()) + pack_size;
    for (j, i) in (pack_size..count).enumerate() {
        st.items.items[i as usize] = blank_item();

        analyze_and_deinscribe(st, i, |a| engine::equip_object(a, j));
    }
    borg_cheat_quiver(st);
}

/// Cheat the "inven" screen (`borg_inventory`).
pub fn borg_inventory(st: &mut BorgState) {
    for i in 0..i32::from(engine::z_info().pack_size) {
        let BorgState { access, items, act, item_val, .. } = &mut *st;
        let obj = engine::inven_object(access, i as usize);
        items.items[i as usize] = blank_item();

        /* Skip non-items */
        let Some(obj) = obj else { continue };

        /* Describe it */
        let desc = engine::object_desc_full(obj);

        /* Skip Empty slots */
        if desc == b"(nothing)" {
            continue;
        }

        /* Analyze the item (no price) */
        borg_item_analyze(
        &mut items.items[i as usize],
        crate::borg_item::NoteOrigin::Item(i as u16),
        obj.into(),
        &desc,
        false,
        act,
        item_val,
    );

        /* Note changed inventory */
        st.inventory.do_crush_junk = true;

        /* Uninscribe items with ! or borg inscriptions */
        deinscribe_if_marked(st, i);
    }
}

/// The first empty pack slot after the last used one, or -1
/// (`borg_first_empty_inventory_slot`). An empty pack gives slot 0.
pub fn borg_first_empty_inventory_slot(st: &mut BorgState) -> i32 {
    /* PACK_SLOTS */
    let pack_slots = i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS);

    /* Find the last used slot (-1 when the pack is empty) */
    let mut i = pack_slots - 1;
    while i >= 0 && st.items.items[i as usize].iqty == 0 {
        i -= 1;
    }

    /* The slot after it is the first empty one, if the pack has room */
    if (i + 1) < pack_slots {
        i + 1
    } else {
        -1
    }
}

/// Is it worth using ID on an item (`borg_item_worth_id`)? Also a general
/// test for whether an unidentified item is worth keeping.
pub fn borg_item_worth_id(st: &mut BorgState, item: &borg_item) -> bool {
    if !crate::borg_item_id::borg_item_note_needs_id(st, item) {
        return false;
    }

    /* Things that can't be wielded can't be ID'd (potions, scrolls etc) */
    borg_wield_slot(item) >= 0
}

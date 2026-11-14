//! Port of `borg-item.c`: the Borg's view of its items.
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

use crate::borg_io::borg_keypress;
use crate::borg_headers::borg_item_h::borg_item;
use crate::headers::list_tvals_h::TV_FOOD;
use crate::state::BorgState;
use crate::engine;

/// One item as the Borg sees it (`borg_item`), with the C field names.
#[derive(Copy, Clone)]
pub struct BorgItem {
    pub desc: [core::ffi::c_char; 80],
    /// `note` (C: `char *note`, NULL = `None`): where the inscription
    /// starts, as the live element whose `desc` C's pointer points into
    /// plus the offset in it. See `borg_item::item_note`.
    pub note: Option<NoteRef>,
    pub kind: u32,
    pub ident: bool,
    pub needs_ident: bool,
    pub aware: bool,
    /// `xxxx` (never read, in C either)
    #[allow(dead_code)]
    pub xxxx: bool,
    pub tval: u8,
    pub sval: u8,
    pub pval: i16,
    pub iqty: u8,
    pub weight: i16,
    pub art_idx: u8,
    pub ego_idx: u8,
    pub activ_idx: i32,
    pub one_ring: bool,
    pub timeout: i16,
    pub to_h: i16,
    pub to_d: i16,
    pub to_a: i16,
    pub ac: i16,
    pub dd: u8,
    pub ds: u8,
    pub level: u8,
    pub cost: i32,
    pub value: i32,
    pub cursed: bool,
    pub uncursable: bool,
    pub curses: [bool; 28],
    pub flags: [u8; 5],
    pub modifiers: [i16; 16],
    pub el_info: [crate::headers::object_h::element_info; 25],
    pub brands: [bool; 254],
    pub slays: [i32; 85],
}

/// The items (`borg_items`) and the snapshot taken after each full read
/// (`safe_items`, the "safety inventory").
///
/// Slots: `0..pack_size` the pack, `pack_size..pack_size + 12` the
/// equipment (`INVEN_WIELD` = `pack_size`), then the quiver.
///
/// An item's `note` is a `NoteRef`: the live origin element (pack/equipment
/// slot or shop ware) plus an offset into *that* element's `desc`, read at
/// each use, as C's pointer is (see `NoteRef`, `item_note`).
#[derive(Default)]
pub struct ItemState {
    pub items: Vec<borg_item>,
    pub safe_items: Vec<borg_item>,
}

/// `INVEN_WIELD`: the first equipment slot.
pub fn inven_wield() -> i32 {
    i32::from(engine::z_info().pack_size)
}

/// `INVEN_FEET`: the last equipment slot that has a letter.
pub fn inven_feet() -> i32 {
    inven_wield() + 11
}

/// `QUIVER_START`
pub fn quiver_start() -> i32 {
    inven_wield() + 12
}

/// `QUIVER_END`: the number of item slots.
pub fn quiver_end() -> i32 {
    let z = engine::z_info();
    i32::from(z.pack_size) + 12 + i32::from(z.quiver_size)
}

/// A zeroed item (as `mem_zalloc` makes them): every field 0, `note`
/// NULL.
pub fn blank_item() -> borg_item {
    use crate::headers::object_h::element_info;
    borg_item {
        desc: [0; 80],
        note: None,
        kind: 0,
        ident: false,
        needs_ident: false,
        aware: false,
        xxxx: false,
        tval: 0,
        sval: 0,
        pval: 0,
        iqty: 0,
        weight: 0,
        art_idx: 0,
        ego_idx: 0,
        activ_idx: 0,
        one_ring: false,
        timeout: 0,
        to_h: 0,
        to_d: 0,
        to_a: 0,
        ac: 0,
        dd: 0,
        ds: 0,
        level: 0,
        cost: 0,
        value: 0,
        cursed: false,
        uncursable: false,
        curses: [false; 28],
        flags: [0; 5],
        modifiers: [0; 16],
        el_info: [element_info { res_level: 0, flags: 0 }; 25],
        brands: [false; 254],
        slays: [0; 85],
    }
}

/// The live element a note points into. C sets `note` only in
/// `borg_item_analyze`, and only on `borg_items[i]` (borg-inventory.c) or
/// `borg_shops[shop].ware[slot]` (borg-store.c), as a pointer into that
/// element's own `desc`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NoteOrigin {
    /// `borg_items[i]`
    Item(u16),
    /// `borg_shops[shop].ware[slot]`
    Ware { shop: u8, slot: u8 },
}

/// C's `note` pointer: `origin->desc + offset`.
///
/// A struct copy of an item (the `safe_items`/`safe_shops` snapshots, the
/// what-if trials that copy items between slots) copies the pointer, so a
/// copy's note still reads the *origin's* `desc` as it is when the note is
/// read, not the copy's own bytes. Example (a wear trial): the old
/// equipment is copied from `safe_items[slot]` into the pack hole, then the
/// candidate is copied over `borg_items[slot]`; the hole's note now reads
/// the candidate's `desc` at the old offset. Resolving the note against the
/// live origin at every read reproduces this exactly (see
/// `tests::wear_trial_hole_reads_live_origin`).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NoteRef {
    pub origin: NoteOrigin,
    pub offset: u8,
}

/// `item->note = item->desc + offset` (borg-item-analyze.c), where `item`
/// is the live element `origin`.
pub fn set_item_note(item: &mut borg_item, origin: NoteOrigin, offset: usize) {
    assert!(offset < item.desc.len(), "note offset outside desc");
    item.note = Some(NoteRef { origin, offset: offset as u8 });
}

/// The `desc` buffer a note points into, as it is now.
fn note_origin_desc(st: &BorgState, origin: NoteOrigin) -> &[core::ffi::c_char; 80] {
    match origin {
        NoteOrigin::Item(i) => &st.items.items[usize::from(i)].desc,
        NoteOrigin::Ware { shop, slot } => &st.store.shops[usize::from(shop)].ware[usize::from(slot)].desc,
    }
}

/// The bytes of `borg_get_note(item)` for any item (a live one or a copy):
/// the bytes from the note pointer up to the next NUL, read from the
/// origin's current `desc`; "" when `note` is NULL. Panics when the buffer
/// has no NUL after the offset (C would read past `desc`).
pub fn item_note(st: &BorgState, item: &borg_item) -> Vec<u8> {
    match item.note {
        None => Vec::new(),
        Some(n) => {
            let desc = note_origin_desc(st, n.origin);
            let tail = &desc[usize::from(n.offset)..];
            let len = tail.iter().position(|&c| c == 0).expect("note runs past desc without a NUL");
            tail[..len].iter().map(|&c| c as u8).collect()
        }
    }
}

/// `borg_get_note(&borg_items[i])`, as bytes.
pub fn borg_get_note(st: &BorgState, i: usize) -> Vec<u8> {
    item_note(st, &st.items.items[i])
}

/// Does the note of slot `j` start with `{` (an inscription)? C reads
/// `borg_items[j].note[0]` without a check; a NULL note would crash C, so
/// it panics here.
fn note_starts_with_brace(st: &mut BorgState, j: i32) -> bool {
    let n = st.items.items[j as usize].note.unwrap_or_else(|| panic!("borg_items[{j}].note is NULL"));
    note_origin_desc(st, n.origin)[usize::from(n.offset)] as u8 == b'{'
}

/// Remove the inscription of slot `i` (`borg_deinscribe`).
pub fn borg_deinscribe(st: &mut BorgState, i: i32) {
    let it = &st.items.items[i as usize];
    if u32::from(it.tval) == TV_FOOD
        && i32::from(it.sval) == st.item_val.sv_food_slime_mold
    {
        return;
    }

    borg_keypress(st, u32::from(b'}'));

    let letter = |n: i32| engine::all_letters_nohjkl(n) as i8 as i32 as u32;
    if i < inven_wield() {
        borg_keypress(st, letter(i));
    } else if i < inven_feet() {
        for j in 0..inven_wield() {
            if st.items.items[j as usize].iqty != 0 && note_starts_with_brace(st, j) {
                borg_keypress(st, u32::from(b'/'));
                break;
            }
        }
        borg_keypress(st, letter(i - inven_wield()));
    } else {
        for j in 0..=inven_feet() {
            if st.items.items[j as usize].iqty != 0 && note_starts_with_brace(st, j) {
                borg_keypress(st, u32::from(b'|'));
                break;
            }
        }
        borg_keypress(st, (i32::from(b'0') + (i - quiver_start())) as u32);
    }
    borg_keypress(st, u32::from(b'y'));
    borg_keypress(st, u32::from(b'y'));
}

/// Weight of the whole stack, as C's `int16_t` (`borg_item_weight`).
pub fn borg_item_weight(item: &borg_item) -> i16 {
    (i32::from(item.iqty) * i32::from(item.weight)) as i16
}

/// Read the equipment, the pack and the store, then snapshot the items
/// (`borg_find_all_items`).
pub fn borg_find_all_items(st: &mut BorgState) {
    crate::borg_inventory::borg_equipment(st);
    crate::borg_inventory::borg_inventory(st);
    crate::borg_store::borg_notice_store(st);
    let n = quiver_end() as usize;
    let snapshot = st.items.items[..n].to_vec();
    st.items.safe_items[..n].copy_from_slice(&snapshot);
}

/// Allocate the item arrays (`borg_init_item`).
pub fn borg_init_item(st: &mut BorgState) {
    let n = quiver_end() as usize;
    st.items.items = vec![blank_item(); n];
    st.items.safe_items = vec![blank_item(); n];
}

/// Free the item arrays (`borg_free_item`).
pub fn borg_free_item(st: &mut BorgState) {
    st.items.safe_items = Vec::new();
    st.items.items = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(it: &mut borg_item, desc: &str, origin: NoteOrigin, off: usize) {
        *it = blank_item();
        it.iqty = 1;
        for (d, c) in it.desc.iter_mut().zip(desc.bytes()) {
            *d = c as core::ffi::c_char;
        }
        set_item_note(it, origin, off);
    }

    /// Review of 4c06cec08, finding 1: in a wear trial the old equipment is
    /// copied from the snapshot into the pack hole and the candidate over the
    /// equipment slot; C's hole note then reads the candidate's `desc` at the
    /// old offset (expected values from the C side of
    /// harness/regress/note-alias, linked against the original C code).
    #[test]
    fn wear_trial_hole_reads_live_origin() {
        let mut st = BorgState::default();
        st.items.items = vec![blank_item(); 3];
        put(&mut st.items.items[0], "Old", NoteOrigin::Item(0), 3);
        put(&mut st.items.items[1], "New longer {??}", NoteOrigin::Item(1), 11);
        st.items.safe_items = st.items.items.clone();
        st.items.items[2] = st.items.safe_items[0];
        st.items.items[0] = st.items.safe_items[1];
        let hole = st.items.items[2];
        assert_eq!(item_note(&st, &hole), b" longer {??}");
        assert!(crate::borg_item_id::borg_item_note_needs_id(&st, &hole));
        // restored: the slot's own note again
        st.items.items[0] = st.items.safe_items[0];
        assert_eq!(item_note(&st, &st.items.items[0]), b"");
    }

    /// A ware copied into the pack keeps reading the live ware.
    #[test]
    fn ware_copy_reads_live_ware() {
        let mut st = BorgState::default();
        st.items.items = vec![blank_item(); 1];
        st.store.shops = vec![crate::borg_store::BorgShop { ware: vec![blank_item(); 1] }];
        let w = NoteOrigin::Ware { shop: 0, slot: 0 };
        put(&mut st.store.shops[0].ware[0], "A ware {??}", w, 7);
        st.items.items[0] = st.store.shops[0].ware[0];
        put(&mut st.store.shops[0].ware[0], "Changed {magical}", w, 8);
        assert_eq!(item_note(&st, &st.items.items[0]), b" {magical}");
    }
}

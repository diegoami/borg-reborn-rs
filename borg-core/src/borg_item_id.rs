//! Port of `borg-item-id.c`: code around identification of items (objects).
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

use crate::borg_io::{borg_keypress, borg_keypresses, KC_ENTER};
use crate::borg_item::{inven_wield, quiver_end};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::BI_CDEPTH;
use crate::headers::list_elements_h::{
    ELEM_ACID, ELEM_CHAOS, ELEM_COLD, ELEM_DARK, ELEM_DISEN, ELEM_ELEC, ELEM_FIRE, ELEM_LIGHT,
    ELEM_NETHER, ELEM_NEXUS, ELEM_POIS, ELEM_SHARD, ELEM_SOUND,
};
use crate::headers::list_object_flags_h::{
    OF_FREE_ACT, OF_HOLD_LIFE, OF_PROT_BLIND, OF_PROT_CONF, OF_SEE_INVIS, OF_TELEPATHY,
};
use crate::headers::list_object_modifiers_h::OBJ_MOD_SPEED;
use crate::headers::list_tvals_h::{TV_BOW, TV_RING};
use crate::state::BorgState;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::engine;

/// `strstr(hay, needle) != NULL` on the bytes of a C string.
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// `of_has(item->flags, flag)`
fn of_has(item: &borg_item, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

/// `item->el_info[elem].res_level`
fn res_level(item: &borg_item, elem: u32) -> i16 {
    item.el_info[elem as usize].res_level
}

/// Look for an item that needs to be analyzed because it has been IDd
/// (`borg_object_fully_id`).
///
/// This goes through inventory and equipment and inscribes items with
/// random powers (egos with a random power, artifacts) with their powers.
pub fn borg_object_fully_id(st: &mut BorgState) -> bool {
    // look in inventory and equipment for something to *id*
    // (or INVEN_TOTAL)
    for i in 0..quiver_end() {
        // A copy: the keypresses below do not change the items.
        let item = st.items.items[i as usize];

        // inscribe certain objects
        let note = crate::borg_item::item_note(st, &item);
        if st.borg.tr(BI_CDEPTH) != 0
            && u32::from(item.tval) >= TV_BOW
            && u32::from(item.tval) <= TV_RING
            && (borg_ego_has_random_power(usize::from(item.ego_idx)) || item.art_idx != 0)
            && (note == b"{ }" || note.is_empty() || contains(&note, b"uncursed"))
        {
            // make the inscription
            borg_keypress(st, u32::from(b'{'));

            let letter = |n: i32| engine::all_letters_nohjkl(n) as i8 as i32 as u32;
            if i >= inven_wield() {
                borg_keypress(st, u32::from(b'/'));
                borg_keypress(st, letter(i - inven_wield()));
            } else {
                borg_keypress(st, letter(i));
            }

            // make the inscription
            if item.modifiers[OBJ_MOD_SPEED as usize] != 0 {
                borg_keypresses(st, b"Spd");
            }
            // slays and immunities
            if res_level(&item, ELEM_POIS) > 0 {
                borg_keypresses(st, b"Poisn");
            }
            if res_level(&item, ELEM_FIRE) == 3 {
                borg_keypresses(st, b"IFir");
            }
            if res_level(&item, ELEM_COLD) == 3 {
                borg_keypresses(st, b"ICld");
            }
            if res_level(&item, ELEM_ACID) == 3 {
                borg_keypresses(st, b"IAcd");
            }
            if res_level(&item, ELEM_ELEC) == 3 {
                borg_keypresses(st, b"IElc");
            }
            if res_level(&item, ELEM_LIGHT) > 0 {
                borg_keypresses(st, b"Lite");
            }
            if res_level(&item, ELEM_DARK) > 0 {
                borg_keypresses(st, b"Dark");
            }
            if of_has(&item, OF_PROT_BLIND) {
                borg_keypresses(st, b"Blnd");
            }
            if of_has(&item, OF_PROT_CONF) {
                borg_keypresses(st, b"Conf");
            }
            if res_level(&item, ELEM_SOUND) > 0 {
                borg_keypresses(st, b"Sound");
            }
            if res_level(&item, ELEM_SHARD) > 0 {
                borg_keypresses(st, b"Shrd");
            }
            if res_level(&item, ELEM_NETHER) > 0 {
                borg_keypresses(st, b"Nthr");
            }
            if res_level(&item, ELEM_NEXUS) > 0 {
                borg_keypresses(st, b"Nxs");
            }
            if res_level(&item, ELEM_CHAOS) > 0 {
                borg_keypresses(st, b"Chaos");
            }
            if res_level(&item, ELEM_DISEN) > 0 {
                borg_keypresses(st, b"Disn");
            }
            // TR2_activate was removed
            if item.activ_idx != 0 {
                borg_keypresses(st, b"Actv");
            }
            if of_has(&item, OF_TELEPATHY) {
                borg_keypresses(st, b"ESP");
            }
            if of_has(&item, OF_HOLD_LIFE) {
                borg_keypresses(st, b"HL");
            }
            if of_has(&item, OF_FREE_ACT) {
                borg_keypresses(st, b"FA");
            }
            if of_has(&item, OF_SEE_INVIS) {
                borg_keypresses(st, b"SInv");
            }

            // end the inscription
            borg_keypress(st, KC_ENTER);
        }
    }
    true
}

/// The code currently inscribes items with {??} if they have unknown
/// powers. This helper keeps the check isolated in case this ever changes
/// (`borg_item_note_needs_id`).
pub fn borg_item_note_needs_id(st: &BorgState, item: &borg_item) -> bool {
    if item.ident {
        return false;
    }

    // save a string check
    if item.needs_ident {
        return true;
    }

    contains(&crate::borg_item::item_note(st, item), b"{??}")
}

#[cfg(test)]
mod tests {
    use super::contains;

    #[test]
    fn contains_is_strstr() {
        assert!(contains(b"{??}", b"{??}"));
        assert!(contains(b"magic {??} x", b"{??}"));
        assert!(!contains(b"{?}", b"{??}"));
        assert!(!contains(b"", b"uncursed"));
        assert!(contains(b"{uncursed}", b"uncursed"));
    }
}

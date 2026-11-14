//! Port of `borg-item-decurse.c`: handle removing curses.
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
//! The three functions share the same "can we remove a curse" test and
//! the same "start removing it" chain (scroll, staff, spell, *scroll*,
//! activations, in C's order); they only differ in the item they pick.

use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, KC_ENTER};
use crate::borg_item::{inven_wield, quiver_start};
use crate::borg_item_use::{
    borg_activate_item, borg_equips_item, borg_equips_staff_fail, borg_read_scroll, borg_use_staff,
};
use crate::borg_magic::{borg_spell, borg_spell_okay_fail};
use crate::engine;
use crate::borg_headers::borg_magic_h::REMOVE_CURSE;
use crate::borg_headers::borg_trait_h::{BI_FIRST_CURSED, BI_WHERE_CURSED, BORG_EQUIP, BORG_INVEN};
use crate::headers::list_tvals_h::TV_SCROLL;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `all_letters_nohjkl[i]` as a key (a C `char`, sign-extended to
/// `keycode_t`). `i` is -1 only for a swap index of 0 (a decurse flag
/// without a swap, which `borg_notice_*_swap` never leaves behind). C then
/// reads the byte before the array, which is alignment padding (0) in
/// both harness binaries (see `borg_item_wear::swap_letter` and
/// docs/c-bugs.md sections 21 and 22), so the key is 0.
fn letter(i: i32) -> keycode_t {
    if i == -1 {
        return 0;
    }
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// A key for a character literal.
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// The "Ability for curses" test: false when the Borg has no scroll,
/// staff, spell or activation that removes a curse.
fn can_decurse(st: &mut BorgState) -> bool {
    let sv_scroll = st.item_val.sv_scroll_remove_curse;
    let sv_staff = st.item_val.sv_staff_remove_curse;
    let sv_star = st.item_val.sv_scroll_star_remove_curse;
    let (act1, act2) = (st.act.remove_curse, st.act.remove_curse2);

    !(-1 == borg_slot(st, TV_SCROLL as i32, sv_scroll)
        && !borg_equips_staff_fail(st, sv_staff)
        && !borg_spell_okay_fail(st, REMOVE_CURSE, 40)
        && -1 == borg_slot(st, TV_SCROLL as i32, sv_star)
        && !borg_equips_item(st, act1, true)
        && !borg_equips_item(st, act2, true))
}

/// "remove the curse": read, use, cast or activate the first thing that
/// works (C's `||` chain, same order).
fn start_decurse(st: &mut BorgState) -> bool {
    let sv_scroll = st.item_val.sv_scroll_remove_curse;
    let sv_staff = st.item_val.sv_staff_remove_curse;
    let sv_star = st.item_val.sv_scroll_star_remove_curse;
    let (act1, act2) = (st.act.remove_curse, st.act.remove_curse2);

    borg_read_scroll(st, sv_scroll)
        || borg_use_staff(st, sv_staff)
        || borg_spell(st, REMOVE_CURSE)
        || borg_read_scroll(st, sv_star)
        || borg_activate_item(st, act1)
        || borg_activate_item(st, act2)
}

/// Remove the curse of a swap item in pack slot `swap - 1`.
fn decurse_swap(st: &mut BorgState, swap: fn(&BorgState) -> i32) -> bool {
    /* Ability for curses */
    if !can_decurse(st) {
        return false;
    }

    /* remove the curse */
    if start_decurse(st) {
        /* pick the item */
        let i = swap(st) - 1;
        borg_keypress(st, letter(i));
        /* pick first curse */
        borg_keypress(st, KC_ENTER);

        /* Shekockazol! */
        return true;
    }

    /* Nothing to do */
    false
}

/// Remove Curse swap armour
pub fn borg_decurse_armour(st: &mut BorgState) -> bool {
    /* Nothing to decurse */
    if !crate::borg_trait::borg_uses_swaps(st) || !st.trait_swap.armour.decurse {
        return false;
    }

    decurse_swap(st, |st| st.trait_swap.armour_swap)
}

/// Remove Curse swap weapon
pub fn borg_decurse_weapon(st: &mut BorgState) -> bool {
    /* Nothing to decurse */
    if !crate::borg_trait::borg_uses_swaps(st) || !st.trait_swap.weapon.decurse {
        return false;
    }

    decurse_swap(st, |st| st.trait_swap.weapon_swap)
}

/// Remove Curse any
pub fn borg_decurse_any(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_FIRST_CURSED) != 0 {
        if !can_decurse(st) {
            return false;
        }

        /* remove the curse */
        if start_decurse(st) {
            /* pick the item */
            let first = st.borg.tr(BI_FIRST_CURSED);
            let where_ = st.borg.tr(BI_WHERE_CURSED);
            if first <= inven_wield() {
                borg_keypress(st, letter(first - 1));
            } else if first <= quiver_start() {
                if where_ & BORG_INVEN as i32 != 0 {
                    borg_keypress(st, key(b'/'));
                }

                borg_keypress(st, letter(first - inven_wield() - 1));
            } else {
                if where_ & 1 != 0 || where_ & BORG_EQUIP as i32 != 0 {
                    borg_keypress(st, key(b'|'));
                }
                borg_keypress(
                    st,
                    (i32::from(b'0') + (first - 1 - quiver_start())) as keycode_t,
                );
            }
            /* pick first curse */
            borg_keypress(st, KC_ENTER);

            /* Shekockazol! */
            return true;
        }
    }

    /* Nothing to do */
    false
}

//! Port of `borg-item-wear.c`: wearing, swapping and removing equipment.
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
//! The "what if" evaluations change `st.items.items` in place, call
//! `borg_notice` / `borg_power` / `borg_danger`, and restore the slots
//! from `safe_items` exactly as C does. Struct copies copy the `note`
//! reference (`borg_item::NoteRef`), so a copied item's note reads the
//! *live origin's* current `desc`, as C's copied pointer does: e.g. the
//! pack hole holding the old equipment reads the candidate's `desc` while
//! the candidate is worn (review of 4c06cec08, finding 1).

use crate::borg::BorgBest;
use crate::borg_danger::borg_danger;
use crate::borg_inventory::{borg_first_empty_inventory_slot, borg_item_worth_id, borg_wield_slot};
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, ESCAPE, KC_ENTER};
use crate::borg_item::{inven_wield, quiver_end, quiver_start};
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_magic::{borg_spell, borg_spell_legal};
use crate::borg_power::borg_power;
use crate::borg_store::{borg_home_full, borg_inventory_full, BORG_HOME, SHOP_MENU_ITEMS};
use crate::borg_flow_misc;
use crate::engine;
use crate::borg_headers::borg_h::BORG_VERBOSE;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::IDENTIFY_RUNE;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CURSP, BI_FRACT, BI_ISENCUMB, BI_ISHUNGRY, BI_ISWEAK, BI_MAXDEPTH,
    BI_QUIVER_SLOTS, BI_RBLIND, BI_RCONF, BI_RDARK, BI_RLITE, BI_SAV,
};
use crate::headers::list_options_h::OPT_birth_randarts;
use crate::headers::list_stats_h::OBJ_MOD_STR;
use crate::headers::list_tvals_h::*;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// borg-item-wear.c globals: the artifacts worn recently, to avoid
/// wear/take-off loops.
#[derive(Default)]
pub struct WearState {
    /// `track_worn_num`
    pub track_worn_num: i16,
    /// `track_worn_size`
    pub track_worn_size: i16,
    /// `track_worn_name1`: `track_worn_size` artifact indexes (empty = NULL).
    pub track_worn_name1: Vec<u8>,
    /// `track_worn_time` (an `int16_t`: `borg.time.now` is truncated).
    pub track_worn_time: i16,
}

/// `INVEN_RIGHT`
fn inven_right() -> i32 {
    inven_wield() + 2
}

/// `INVEN_LEFT`
fn inven_left() -> i32 {
    inven_wield() + 3
}

/// `PACK_SLOTS`: the pack slots not taken by the quiver.
fn pack_slots(st: &BorgState) -> i32 {
    i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS)
}

/// `borg_timer(t)`: the turns since `t`.
fn borg_timer(st: &BorgState, t: i32) -> i32 {
    st.borg.time.now - t
}

/// `all_letters_nohjkl[i]` as a key (a C `char` widened to `keycode_t`).
fn letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// `all_letters_nohjkl[swap]` in `borg_backup_swap`, where `swap` can be
/// -1 (an armour swap with no weapon swap, see docs/c-bugs.md). C then
/// reads the byte before the array: in both the C and the Rust build that
/// is alignment padding (0; checked with `objdump` on the harness
/// binaries), so the key is 0 and `borg_keypress` refuses it with its
/// "BAD KEY" note.
fn swap_letter(swap: i32) -> keycode_t {
    if swap == -1 {
        return 0;
    }
    letter(swap)
}

/// The bytes of a C `desc` buffer, up to its NUL (`%s`).
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `borg_note(format(fmt, desc))` with one `%s` (written as `{}`).
fn note_desc(st: &mut BorgState, before: &str, item: &borg_item, after: &str) {
    let mut s = before.as_bytes().to_vec();
    s.extend_from_slice(&desc(item));
    s.extend_from_slice(after.as_bytes());
    borg_note_bytes(st, &s);
}

/// `OPT(player, birth_randarts)`
fn randarts() -> bool {
    engine::player_opt(OPT_birth_randarts)
}

/// `borg_danger(borg.c.y, borg.c.x, 1, true, false)`: the danger here.
fn danger_here(st: &mut BorgState) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger(st, y, x, 1, true, false)
}

/// `memcpy(&borg_items[to], &safe_items[from], sizeof(borg_item))`
fn from_safe(st: &mut BorgState, to: i32, from: i32) {
    st.items.items[to as usize] = st.items.safe_items[from as usize];
}

/// `borg_items[i].iqty--` / `++` (an `uint8_t`).
fn add_iqty(st: &mut BorgState, i: i32, d: i8) {
    let q = &mut st.items.items[i as usize].iqty;
    *q = q.wrapping_add_signed(d);
}

/// `borg.antibounce_count++` (an `int16_t`).
fn antibounce(st: &mut BorgState) {
    st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(1);
}

/// `long` arithmetic stored back into an `int`: `(int)(a + b)`.
fn add_long(a: i32, b: i64) -> i32 {
    (i64::from(a) + b) as i32
}

/// Identify items if possible (`borg_test_stuff`).
///
/// Note that "borg_parse()" will "cancel" the identification if it
/// detects a "You failed..." message.  This is VERY important!!!
/// Otherwise the "identify" might induce bizarre actions by sending
/// the "index" of an item as a command.
///
/// We instantly identify items known to be "good" (or "terrible"), and
/// most un-aware and un-known items as soon as possible.
pub fn borg_test_stuff(st: &mut BorgState) -> bool {
    let mut b_i: i32 = -1;
    let mut b_v: i32 = -1;
    let mut inv_item_needs_id = false;
    let free_id = borg_spell_legal(st, IDENTIFY_RUNE);

    /* don't ID stuff when you can't recover spent spell point immediately */
    if st.borg.tr(BI_CURSP) < 50 && borg_spell_legal(st, IDENTIFY_RUNE) && {
        let (y, x) = (st.borg.c.y, st.borg.c.x);
        !borg_flow_misc::borg_check_rest(st, y, x)
    } {
        return false;
    }

    /* No ID if in danger */
    if danger_here(st) > 1 {
        return false;
    }

    /* Look for an item to identify (equipment) */
    for i in inven_wield()..quiver_end() {
        let mut v: i32 = 0;
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }
        if !item.needs_ident {
            continue;
        }

        /* Preferentially ID egos and artifacts */
        if item.art_idx != 0 {
            v = add_long(item.value, 150000);
        }

        if borg_ego_has_random_power(usize::from(item.ego_idx)) {
            v = add_long(item.value, 100000);
        }

        /* Prioritize the ID */
        if borg_item_note_needs_id(st, item) {
            v = add_long(item.value, 20000);
        }

        /* Ignore */
        if v == 0 {
            continue;
        }

        /* Track the best */
        if v <= b_v {
            continue;
        }

        /* Track it */
        b_i = i;
        b_v = v;
    }

    /* Look for an item to identify  */
    for i in 0..quiver_end() {
        let mut v: i32 = 0;
        let item = st.items.items[i as usize];

        /* Skip empty and ID'd items */
        if item.iqty == 0 {
            continue;
        }
        if !item.needs_ident {
            continue;
        }

        if i < inven_wield() {
            inv_item_needs_id = true;
        }

        /* Preferentially ID artifacts */
        if item.art_idx != 0 {
            v = add_long(item.value, 150000);
        }

        /* Identify "good" items */
        if borg_item_note_needs_id(st, &item) {
            v = add_long(item.value, 20000);
        } else if free_id || borg_item_worth_id(st, &item) {
            v = item.value;
        }

        /* Reward "unaware" items */
        if !item.aware {
            let maxdepth = i64::from(st.borg.tr(BI_MAXDEPTH));
            /* Analyze the type */
            match u32::from(item.tval) {
                TV_RING | TV_AMULET => v = add_long(v, maxdepth * 5000),
                TV_ROD => v = add_long(v, maxdepth * 3000),
                TV_WAND | TV_STAFF => v = add_long(v, maxdepth * 2000),
                TV_POTION | TV_SCROLL => {
                    /* Boring levels */
                    if maxdepth >= 5 {
                        /* Reward depth */
                        v = add_long(v, maxdepth * 500);
                    }
                }
                TV_FOOD => v = add_long(v, maxdepth * 10),
                _ => {}
            }
        }

        /* Ignore */
        if v == 0 {
            continue;
        }

        /* Track the best */
        if v <= b_v {
            continue;
        }

        /* Track it */
        b_i = i;
        b_v = v;
    }

    /* Found something */
    if b_i >= 0 {
        /* Use an item to identify */
        let sv = st.item_val.sv_scroll_identify;
        if borg_spell(st, IDENTIFY_RUNE) || crate::borg_item_use::borg_read_scroll(st, sv) {
            /* Log -- may be cancelled */
            let item = st.items.items[b_i as usize];
            note_desc(st, "# Identifying ", &item, ".");

            if b_i >= inven_wield() && b_i < quiver_start() {
                /* Equipment */
                if inv_item_needs_id {
                    borg_keypress(st, keycode_t::from(b'/'));
                }

                /* Select the item */
                borg_keypress(st, letter(b_i - inven_wield()));
            } else if b_i >= quiver_start() {
                /* Select quiver */
                borg_keypress(st, keycode_t::from(b'|'));

                /* Select the item (I2D) */
                borg_keypress(st, (b_i - quiver_start() + i32::from(b'0')) as keycode_t);
            } else {
                /* Inventory: select the item */
                borg_keypress(st, letter(b_i));
            }

            borg_keypress(st, ESCAPE);

            return true;
        }
    }

    /* Nothing to do */
    false
}

/// Make sure that, if possible, the "best" ring is on the "right" (tight)
/// finger (`borg_swap_rings`), so that the other functions can assume the
/// right ring is "good for us" and should never be removed.
pub fn borg_swap_rings(st: &mut BorgState) -> bool {
    let hole = borg_first_empty_inventory_slot(st);
    let left = inven_left();
    let right = inven_right();

    /*** Check conditions ***/

    /* Require two empty slots */
    if hole == -1 {
        return false;
    }

    if (hole + 1) >= pack_slots(st) {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 1000 {
        return false;
    }
    if st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }

    /*** Remove naked "loose" rings ***/

    /* Remove any naked loose ring */
    {
        let items = &st.items.items;
        if items[left as usize].iqty != 0
            && items[right as usize].iqty == 0
            && !items[left as usize].one_ring
        {
            /* Log */
            borg_note(st, "# Taking off naked loose ring.");

            /* Remove it */
            borg_keypress(st, keycode_t::from(b't'));
            borg_keypress(st, letter(left - inven_wield()));

            /* Success */
            return true;
        }
    }

    /*** Check conditions ***/

    /* Require "tight" ring */
    if st.items.items[right as usize].iqty == 0 {
        return false;
    }

    /* Cannot remove the One Ring */
    if st.items.items[right as usize].one_ring {
        return false;
    }

    /*** Remove nasty "tight" rings ***/

    let v1: i32 = if st.items.items[left as usize].iqty == 0 {
        /* Take off the ring */
        st.items.items[hole as usize] = st.items.items[left as usize];

        /* Erase left ring */
        st.items.items[left as usize].iqty = 0;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let v1 = borg_power(st);

        /* Restore the ring */
        from_safe(st, left, left);

        /* Restore the hole */
        st.items.items[hole as usize].iqty = 0;
        v1
    } else {
        st.borg.power
    };

    /*** Consider taking off the "right" ring ***/

    /* Take off the ring */
    st.items.items[hole as usize] = st.items.items[right as usize];

    /* Erase the ring */
    st.items.items[right as usize].iqty = 0;

    /* Examine the inventory */
    crate::borg_trait::borg_notice(st, false);

    /* Evaluate the inventory */
    let v2 = borg_power(st);

    /* Restore the ring */
    from_safe(st, right, right);

    /* Restore the hole */
    st.items.items[hole as usize].iqty = 0;

    /*** Swap rings if necessary ***/

    /* Remove "useless" ring */
    if v2 > v1 {
        /* Log */
        borg_note(st, "# Taking off less valuable right ring.");

        /* Take it off */
        if st.items.items[right as usize].iqty != 0 {
            borg_keypress(st, keycode_t::from(b't'));
            borg_keypress(st, letter(right - inven_wield()));
        }

        /* make sure one is on the left */
        if st.items.items[left as usize].iqty != 0 {
            borg_note(st, "# Taking off more valuable left ring.");
            borg_keypress(st, keycode_t::from(b't'));
            borg_keypress(st, letter(left - inven_wield()));
        }

        /* Success */
        return true;
    }

    /* fix the inventory */
    crate::borg_trait::borg_notice(st, true);

    /* Nope */
    false
}

/// Place our "best" ring on the "tight" finger if needed
/// (`borg_wear_rings`): each ring is tried on the left hand and the one
/// that gives the most "power" is worn. Only used when both ring fingers
/// are empty.
pub fn borg_wear_rings(st: &mut BorgState) -> bool {
    let hole = borg_first_empty_inventory_slot(st);
    let left = inven_left() as usize;

    let mut b_p: i32 = 0;
    let mut b_i: i32 = -1;

    let mut fix = false;

    if hole == -1 {
        return false;
    }

    /* Require no rings */
    if st.items.items[left].iqty != 0 {
        return false;
    }
    if st.items.items[inven_right() as usize].iqty != 0 {
        return false;
    }

    /* Require two empty slots */
    if hole + 1 >= pack_slots(st) {
        return false;
    }
    if st.items.items[(hole + 1) as usize].iqty != 0 {
        return false;
    }

    /* hack prevent the swap till you drop loop */
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 2000 {
        return false;
    }

    /* Scan inventory */
    for i in 0..inven_wield() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Require aware */
        if !item.aware {
            continue;
        }

        /* Ignore "worthless" items */
        if item.value == 0 {
            continue;
        }

        /* skip artifact rings not star id'd  */
        if randarts() && !item.ident && item.art_idx != 0 {
            continue;
        }

        /* Only process "rings" */
        if u32::from(item.tval) != TV_RING {
            continue;
        }

        /* Wear new item */
        st.items.items[left] = st.items.items[i as usize];

        /* Only a single item */
        st.items.items[left].iqty = 1;

        /* Reduce the inventory quantity by one */
        add_iqty(st, i, -1);

        /* Fix later */
        fix = true;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let mut p = borg_power(st);

        /* the One Ring would be awesome */
        if st.items.items[i as usize].one_ring {
            p = st.borg.power.wrapping_mul(2);
        }

        /* Restore the old item (empty) */
        st.items.items[left].iqty = 0;

        /* Restore the item in inventory */
        add_iqty(st, i, 1);

        /* Ignore "bad" swaps */
        if (b_i >= 0) && (p < b_p) {
            continue;
        }

        /* Maintain the "best" */
        b_i = i;
        b_p = p;
    }

    /* Restore bonuses */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* No item */
    if (b_i >= 0) && (b_p > st.borg.power) {
        /* Get the item */
        let item = st.items.items[b_i as usize];

        /* Log */
        borg_note(st, "# Putting on best tight ring.");

        /* Log */
        note_desc(st, "# Wearing ", &item, ".");

        /* Wear it */
        borg_keypress(st, keycode_t::from(b'w'));
        borg_keypress(st, letter(b_i));

        /* Did something */
        antibounce(st);
        return true;
    }

    /* Nope */
    false
}

/// Try one backup item in its slot (`borg_backup_swap`'s per-item block,
/// up to the evaluation): wear `safe_items[i]` there, examine with
/// `borg_notice(false)` and return `(slot, danger)`, or `None` when the
/// item cannot be worn. The caller restores the slots.
fn backup_try(st: &mut BorgState, i: i32) -> Option<(i32, i32)> {
    /* Where does it go */
    let slot = borg_wield_slot(&st.items.items[i as usize]);

    /* safety check in case slot = -1 */
    if slot < 0 {
        return None;
    }

    /* Wear new item */
    from_safe(st, slot, i);

    /* Only a single item */
    st.items.items[slot as usize].iqty = 1;

    /* Reduce the inventory quantity by one */
    add_iqty(st, i, -1);

    /* Examine the benefits of the swap item */
    crate::borg_trait::borg_notice(st, false);

    /* Evaluate the power with the new item worn */
    Some((slot, danger_here(st)))
}

/// The resists that must not be swapped out
/// (`borg_backup_swap`'s "critical skills" tests).
fn lost_critical(st: &BorgState, save_rconf: i32, save_rblind: i32, save_fract: i32) -> [bool; 3] {
    let t = |i| st.borg.tr(i);
    [
        save_rconf != 0 && t(BI_RCONF) == 0,
        save_rblind != 0
            && (t(BI_RBLIND) == 0 && t(BI_RLITE) == 0 && t(BI_RDARK) == 0 && t(BI_SAV) < 100),
        save_fract != 0 && (t(BI_FRACT) == 0 && t(BI_SAV) < 100),
    ]
}

/// The swap item may not be used: an un-*ID*'d random artifact or a
/// curse that cannot be removed.
fn backup_forbidden(st: &BorgState, i: i32) -> [bool; 2] {
    let item = &st.items.items[i as usize];
    [randarts() && item.art_idx != 0 && !item.ident, item.cursed && !item.uncursable]
}

/// The emergency-swap choice (`borg_backup_swap`).
///
/// The candidate with the lower danger is worn; the weapon wins a tie. A
/// candidate that does not exist keeps the 9999 sentinel, so it can only be
/// chosen in a 9999 tie, where the acceptance threshold rejects the swap
/// anyway.
fn backup_swap_choice(armour_swap: i32, weapon_swap: i32, b_p1: i32, b_p2: i32) -> (i32, i32) {
    if b_p2 <= b_p1 {
        (b_p2, weapon_swap - 1)
    } else {
        (b_p1, armour_swap - 1)
    }
}

/// Place our "swap" if needed (`borg_backup_swap`). Both the armour and
/// the weapon swap are evaluated and the one with the lower danger (but
/// see docs/c-bugs.md) is worn when it lowers the danger `p` enough.
/// The borg is forbidden to swap out certain resistances.
pub fn borg_backup_swap(st: &mut BorgState, p: i32) -> bool {
    let mut b_p1: i32 = 9999;
    let mut b_p2: i32 = 9999;

    let mut save_rconf: i32 = 0;
    let mut save_rblind: i32 = 0;
    let mut save_fract: i32 = 0;

    let mut fix = false;

    /* hack prevent the swap till you drop loop */
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if st.borg.antibounce_count > 300 {
        return false;
    }

    /* (`armour_swap` / `weapon_swap` are read where C reads them:
     * `borg_notice(true)` recomputes them) */

    if st.trait_swap.armour_swap != 0 {
        /* Save our normal condition */
        save_rconf = st.borg.tr(BI_RCONF);
        save_rblind = st.borg.tr(BI_RBLIND);
        save_fract = st.borg.tr(BI_FRACT);

        /* Check the items, first armour then weapon */
        let mut i = st.trait_swap.armour_swap - 1;

        /* make sure it is not a -1 */
        if i == -1 {
            i = 0;
        }

        let Some((slot, d)) = backup_try(st, i) else {
            return false;
        };
        fix = true;
        b_p1 = d;

        /* Restore the old item */
        from_safe(st, slot, slot);

        /* Restore the new item */
        from_safe(st, i, i);

        /* Examine the critical skills */
        let lost = lost_critical(st, save_rconf, save_rblind, save_fract);
        for l in lost {
            if l {
                b_p1 = 9999;
            }
        }

        /* Restore bonuses */
        if fix {
            crate::borg_trait::borg_notice(st, true);
        }

        /*  skip random artifact not star id'd  */
        /* skip it if it has not been decursed */
        for f in backup_forbidden(st, i) {
            if f {
                b_p1 = 9999;
            }
        }
    }

    /* Now we check the weapon */
    if st.trait_swap.weapon_swap != 0 {
        /* get the item */
        let mut i = st.trait_swap.weapon_swap - 1;

        /* make sure it is not a -1 */
        if i == -1 {
            i = 0;
        }

        let Some((slot, d)) = backup_try(st, i) else {
            return false;
        };
        fix = true;
        b_p2 = d;

        /* Examine the critical skills */
        let lost = lost_critical(st, save_rconf, save_rblind, save_fract);
        for l in lost {
            if l {
                b_p2 = 9999;
            }
        }

        /* Restore the old item (empty) */
        from_safe(st, slot, slot);

        /* Restore the new item */
        from_safe(st, i, i);

        /* Restore bonuses */
        if fix {
            crate::borg_trait::borg_notice(st, true);
        }

        /*  skip random artifact not star id'd  */
        /* skip it if it has not been decursed */
        for f in backup_forbidden(st, i) {
            if f {
                b_p2 = 9999;
            }
        }
    }

    /* Pass on the swap which yields the best result */
    let (b_p, swap) = backup_swap_choice(
        st.trait_swap.armour_swap,
        st.trait_swap.weapon_swap,
        b_p1,
        b_p2,
    );

    /* good swap.  Make sure it helps a significant amount */
    let avoidance = i32::from(st.borg.avoidance);
    let limit = if st.borg.near.unique != 0 { (avoidance * 2) / 3 } else { avoidance / 2 };
    if p > b_p && b_p <= limit {
        /* Log */
        borg_note(st, &format!("# Swapping backup.  ({} < {}).", b_p, p));

        /* Wear it */
        borg_keypress(st, keycode_t::from(b'w'));
        borg_keypress(st, swap_letter(swap));

        /* Did something */
        return true;
    }

    /* Nope */
    false
}

/// Wear `safe_items[i]` in `slot` (the old item goes to `hole`), evaluate
/// power and danger, then restore (`borg_wear_stuff`'s trial block).
fn wear_trial(st: &mut BorgState, hole: i32, slot: i32, i: i32) -> (i32, i32) {
    /* Take off old item */
    from_safe(st, hole, slot);

    /* Wear new item */
    from_safe(st, slot, i);

    /* Only a single item */
    st.items.items[slot as usize].iqty = 1;

    /* Reduce the inventory quantity by one */
    add_iqty(st, i, -1);

    /* Examine the inventory */
    crate::borg_trait::borg_notice(st, true);

    /* Evaluate the inventory */
    let p = borg_power(st);

    /* Evaluate local danger */
    let d = danger_here(st);

    /* Restore the old item */
    from_safe(st, slot, slot);

    /* Restore the new item */
    from_safe(st, i, i);

    /* Restore the hole */
    st.items.items[hole as usize].iqty = 0;

    (p, d)
}

/// Wear useful equipment (`borg_wear_stuff`).
///
/// Look through the inventory for equipment that is better than the
/// current equipment, and take one step towards the world with the most
/// "power". With both hands full, a ring is now and then (or always, the
/// One Ring) tried on each hand.
pub fn borg_wear_stuff(st: &mut BorgState) -> bool {
    let left = inven_left();
    let right = inven_right();

    let mut b_i: i32 = -1;
    let mut b_ii: i32 = -1;

    let mut fix = false;

    /* Start with current power */
    let mut b_p: i32 = st.borg.power;

    /*  hack to prevent the swap till you drop loop */
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* We need an empty slot to simulate pushing equipment */
    let hole = borg_first_empty_inventory_slot(st);
    if hole == -1 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 2000 {
        return false;
    }
    if st.borg.antibounce_count > 1300 {
        return false;
    }

    /* Scan inventory */
    for i in 0..inven_wield() {
        let item = &st.items.items[i as usize];

        /* reset this item marker */
        let mut recently_worn = false;

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Require aware */
        if !item.aware {
            continue;
        }

        /* Ignore "worthless" items */
        if item.value == 0 {
            continue;
        }

        /* do not wear not *idd* artifacts */
        if randarts() && item.art_idx != 0 && !item.ident {
            continue;
        }

        /* Do not consider wearing this item if I worn it already this level,
         * I might be stuck in a loop.
         */
        let wear = &st.wear;
        for o in 0..i32::from(wear.track_worn_num) {
            /* Examine the worn list */
            if wear.track_worn_num >= 1
                && wear.track_worn_name1[o as usize] == item.art_idx
                && borg_timer(st, i32::from(wear.track_worn_time)) > 10
            {
                /* Recently worn item */
                recently_worn = true;
            }
        }

        /* Note and fail out */
        if recently_worn {
            let item = *item;
            note_desc(st, "# Not considering a ", &item, "; it was recently worn.");
            continue;
        }

        /* Where does it go */
        let slot = borg_wield_slot(item);

        /* Cannot wear this item */
        if slot < 0 {
            continue;
        }

        /* Do not wear certain items if I am over weight limit.  It induces
         * loops */
        if st.borg.tr(BI_ISENCUMB) != 0 {
            /* Compare Str bonuses */
            let m = OBJ_MOD_STR as usize;
            if st.items.items[slot as usize].modifiers[m] > item.modifiers[m] {
                continue;
            }
        }

        /* Obtain danger */
        let danger = danger_here(st);

        /* If this is a ring and both hands are full, then check each hand
         * and compare the two.  If needed the tight ring can be removed then
         * the better ring placed there on.
         */

        /*** Process regular items and non full rings ***/

        /* Non ring, non full hands */
        if slot != left
            || (st.items.items[left as usize].iqty == 0 || st.items.items[right as usize].iqty == 0)
        {
            /* Fix later */
            fix = true;
            let (mut p, d) = wear_trial(st, hole, slot, i);

            /* Need to be careful not to put the One Ring onto
             * the Left Hand
             */
            if st.items.items[i as usize].one_ring && st.items.items[left as usize].tval == 0 {
                p = -99999;
            }

            /* Ignore if more dangerous */
            if danger < d {
                continue;
            }

            /* XXX XXX XXX Consider if slot is empty */

            /* Ignore "essentially equal" swaps */
            if p <= b_p + 50 {
                continue;
            }

            /* Maintain the "best" */
            b_i = i;
            b_p = p;
        } /* non-rings, non full */

        if engine::randint0(100) == 10 || st.items.items[i as usize].one_ring {
            /* ring, full hands */
            if slot == left
                && st.items.items[left as usize].iqty != 0
                && st.items.items[right as usize].iqty != 0
            {
                for ii in right..=left {
                    /* Does One Ring need to be handled here? */
                    fix = true;
                    let (mut p, d) = wear_trial(st, hole, ii, i);

                    /* Need to be careful not to put the One Ring onto
                     * the Left Hand
                     */
                    if ii == left && st.items.items[i as usize].one_ring {
                        p = -99999;
                    }

                    /* Ignore "bad" swaps */
                    if p < b_p {
                        continue;
                    }

                    /* no swapping into more danger */
                    if danger <= d && danger != 0 {
                        continue;
                    }

                    /* Maintain the "best" */
                    b_i = i;
                    b_p = p;
                    b_ii = ii;
                }
            } /* ring, looking at replacing each ring */
        } /* Random ring check */
    } /* end scanning inventory */

    /* Restore bonuses */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* item */
    if (b_i >= 0) && (b_p > st.borg.power) {
        /* Get the item */
        let item = st.items.items[b_i as usize];

        /* (C copies the desc into an unused `target_ring_desc` here) */

        /* Remove old ring to make room for good one */
        if b_ii >= right && u32::from(item.tval) == TV_RING {
            /* Log */
            let mut s = b"# Removing ".to_vec();
            s.extend_from_slice(&desc(&st.items.items[b_ii as usize]));
            s.extend_from_slice(b" to make room for ");
            s.extend_from_slice(&desc(&item));
            s.push(b'.');
            borg_note_bytes(st, &s);

            /* Make room */
            borg_keypress(st, keycode_t::from(b't'));
            borg_keypress(st, letter(b_ii - inven_wield()));

            /* Did something */
            antibounce(st);
            return true;
        }

        /* Log */
        note_desc(st, "# Wearing ", &item, ".");

        /* Wear it */
        borg_keypress(st, keycode_t::from(b'w'));
        borg_keypress(st, letter(b_i));
        antibounce(st);

        /* Track the newly worn artifact item to avoid loops */
        if item.art_idx != 0 && (st.wear.track_worn_num < st.wear.track_worn_size) {
            borg_note(st, "# Noting the wearing of artifact.");
            let w = &mut st.wear;
            w.track_worn_name1[w.track_worn_num as usize] = item.art_idx;
            w.track_worn_time = st.borg.time.now as i16;
            w.track_worn_num += 1;
        }
        return true;
    }

    /* Nope */
    false
}

/// `UINT16_MAX`: "no slot" in `borg_best_stuff_order` and the test lists.
const NO_SLOT: u16 = u16::MAX;

/// Order of the slots (`borg_best_stuff_order`).
fn borg_best_stuff_order(n: i32) -> u16 {
    let w = inven_wield();
    let slot = match n {
        0 => w + 1,  // INVEN_BOW
        1 => w,      // INVEN_WIELD
        2 => w + 6,  // INVEN_BODY
        3 => w + 7,  // INVEN_OUTER
        4 => w + 8,  // INVEN_ARM
        5 => w + 9,  // INVEN_HEAD
        6 => w + 10, // INVEN_HANDS
        7 => w + 11, // INVEN_FEET
        8 => w + 3,  // INVEN_LEFT
        9 => w + 2,  // INVEN_RIGHT
        10 => w + 5, // INVEN_LIGHT
        11 => w + 4, // INVEN_NECK
        _ => return NO_SLOT,
    };
    slot as u16
}

/// `z_info->equip_slots_max`
fn equip_slots_max() -> usize {
    usize::from(engine::z_info().equip_slots_max)
}

/// Take one step in wearing the optimized equipment
/// (`borg_one_step_wearing_best`). The `skipping` counter is set once,
/// before the loop (better-borg-c milestone7 patch 0075).
fn borg_one_step_wearing_best(st: &mut BorgState, skip: i32) -> bool {
    let mut item_num: usize = 0;

    if st.borg.goal.best_item.is_empty() {
        return false;
    }

    /* skip some if there are issues with full inventory */
    let mut skipping = skip;

    /* find a change */
    loop {
        let b = st.borg.goal.best_item[item_num];
        if b.tval != 0 || b.pval != 0 || b.sval != 0 {
            if skipping != 0 {
                item_num += 1;
                skipping -= 1;

                /* skipping the last item means we need to make room */
                /* before continuing */
                if item_num >= equip_slots_max() {
                    return false;
                }

                continue;
            }

            break;
        }

        item_num += 1;
        if item_num >= equip_slots_max() {
            borg_clear_best(st);

            /* If we aren't skipping we have done our best optimization.  */
            /* if we are skipping and got to the end, we probably need */
            /*  to start over */
            if skip == 0 {
                st.borg.goal.do_best = false;
            }

            return false;
        }
    }

    /* Extract the slot */
    let slot = i32::from(borg_best_stuff_order(item_num as i32));
    let want = st.borg.goal.best_item[item_num];
    let matches = |item: &borg_item| {
        item.pval == want.pval && item.tval == want.tval && item.sval == want.sval
    };

    /* if the item is a home item to be worn, */
    /*  * drop any item you already have in the slot */
    /*  * if the slot is empty, pick up the item and mark */
    /*    as not in the home */
    /* if the item is not a home item to be worn, */
    /*      it is an inventory item.  */
    /*  * take off any item you already have in the slot */
    /*  * if the slot is empty, wear the item */
    /* NOTE that items need to be found again because they could have */
    /*   moved slots since other objects are being taken off and dropped */
    /*   into inventory and the home*/
    if want.home {
        /* if wearing an item in that slot, drop it */
        if st.items.items[slot as usize].iqty != 0 {
            /* if we can't drop an item into the home try the next change */
            if borg_home_full(st) {
                return borg_one_step_wearing_best(st, skip + 1);
            }

            borg_keypress(st, keycode_t::from(b'd'));
            borg_keypress(st, keycode_t::from(b'/'));
            borg_keypress(st, letter(slot - inven_wield()));
            return true;
        }

        /* if we can't get an item from the home, try the next change */
        if borg_inventory_full(st) {
            return borg_one_step_wearing_best(st, skip + 1);
        }

        let n = usize::from(engine::z_info().store_inven_max);
        for (i, &menu) in SHOP_MENU_ITEMS.iter().enumerate().take(n) {
            let item = &st.store.shops[BORG_HOME].ware[i];

            if !matches(item) {
                continue;
            }
            let qty = item.iqty;

            /* "Buy" the desired item from the home */
            borg_keypress(st, menu as i8 as keycode_t);
            borg_keypress(st, keycode_t::from(b'p'));
            if qty > 1 {
                borg_keypress(st, KC_ENTER);
            }

            st.borg.goal.best_item[item_num].home = false;
            return true;
        }
        crate::borg_think::borg_oops(st, "unable to find item in home");
    } else {
        /* if wearing an item in that slot, take it off */
        if st.items.items[slot as usize].iqty != 0 {
            /* take off an item because there is no room, try next */
            if borg_inventory_full(st) {
                return borg_one_step_wearing_best(st, skip + 1);
            }

            borg_keypress(st, keycode_t::from(b't'));
            borg_keypress(st, letter(slot - inven_wield()));
            return true;
        }
        /* (INVEN_TOTAL: the pack and the equipment) */
        for i in 0..quiver_start() {
            if !matches(&st.items.items[i as usize]) {
                continue;
            }
            borg_keypress(st, keycode_t::from(b'w'));
            borg_keypress(st, letter(i));

            st.borg.goal.best_item[item_num] = BorgBest::default();
            return true;
        }
        crate::borg_think::borg_oops(st, "unable to find item in inventory");
    }

    false
}

/// Helper of `borg_best_stuff` (`borg_best_stuff_aux`): try every usable
/// pack and home item in slot `n` of the order, then recurse.
fn borg_best_stuff_aux(st: &mut BorgState, n: i32, test: &mut [u16], best: &mut [u16], vp: &mut i32) {
    /* Extract the slot */
    let order = borg_best_stuff_order(n);

    /* All done */
    if order == NO_SLOT {
        /* Examine */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate */
        let p = borg_power(st);

        /* Track best */
        if p > *vp {
            /* Save the results */
            let k = equip_slots_max();
            best[..k].copy_from_slice(&test[..k]);

            /* Use it */
            *vp = p;
        }

        /* Success */
        return;
    }
    let slot = i32::from(order);
    let nu = n as usize;

    /* Note the attempt */
    test[nu] = order;

    /* Evaluate the default item */
    borg_best_stuff_aux(st, n + 1, test, best, vp);

    let pack_size = i32::from(engine::z_info().pack_size);
    let total = pack_size + i32::from(engine::z_info().store_inven_max);

    /* Try other possible object combination */
    for i in 0..total {
        let in_pack = i < pack_size;
        /* the test list entry: pack index, or home index + 100 */
        let mark = if in_pack { i } else { (i - pack_size) + 100 };
        {
            let item = if in_pack {
                &st.items.items[i as usize]
            } else {
                &st.store.shops[BORG_HOME].ware[(i - pack_size) as usize]
            };

            /* Skip empty items */
            if item.iqty == 0 {
                continue;
            }

            /* Require aware */
            if !item.aware {
                continue;
            }

            /* Ignore "worthless" items */
            if item.value == 0 {
                continue;
            }

            /* Skip it if it is not decursable */
            if item.cursed && !item.uncursable {
                continue;
            }

            /* Do not wear not *idd* artifacts */
            if randarts() && item.art_idx != 0 && !item.ident {
                continue;
            }

            /* Make sure it goes in this slot, special consideration for checking
             * rings */
            let ws = borg_wield_slot(item);
            if !((slot == ws) || (slot == inven_right() && ws == inven_left())) {
                continue;
            }
        }

        /* don't test the same item in both ring slots; the left ring is
         * chosen first, so only the right ring needs the check */
        if slot == inven_right() && i32::from(test[nu - 1]) == mark {
            continue;
        }

        /* Make sure that slot does not have an item that can't be removed */
        if st.items.items[slot as usize].one_ring {
            continue;
        }

        /* Wear the new item */
        st.items.items[slot as usize] = if in_pack {
            st.items.items[i as usize]
        } else {
            st.store.shops[BORG_HOME].ware[(i - pack_size) as usize]
        };

        /* Note the attempt */
        /* if in home, note by adding 100 to item number. */
        test[nu] = mark as u16;

        /* Use recursion to test other slot changes */
        borg_best_stuff_aux(st, n + 1, test, best, vp);

        /* Restore equipment */
        from_safe(st, slot, slot);
    }
}

/// Attempt to instantiate the *best* possible equipment
/// (`borg_best_stuff`): out of everything worn, in the pack and in the
/// home. Only at home, once per trip to town; the changes are then made
/// one step at a time.
pub fn borg_best_stuff(st: &mut BorgState) -> bool {
    /* only do this at home */
    if i32::from(st.think.shop_num) != BORG_HOME as i32 {
        return false;
    }

    /* if we don't need to do "best", check if we need to process the list */
    if !st.borg.goal.do_best {
        return borg_one_step_wearing_best(st, 0);
    }

    /* must have a free inventory slot and a free home slot */
    if borg_home_full(st) || borg_inventory_full(st) {
        return false;
    }

    /* only do this once per trip to town */
    if !st.borg.goal.do_best {
        return false;
    }

    let esm = equip_slots_max();
    let mut best = vec![NO_SLOT; esm];
    let mut test = vec![NO_SLOT; esm];

    /* Evaluate the inventory */
    let mut value = st.borg.power;

    /* Determine the best possible equipment */
    borg_best_stuff_aux(st, 0, &mut test, &mut best, &mut value);

    /* Restore bonuses */
    crate::borg_trait::borg_notice(st, true);

    /* convert the list into a list of items we can find even if they */
    /* change locations due to picking up and dropping items */
    /* Make first change. */
    st.borg.goal.best_item = vec![BorgBest::default(); esm];
    let mut found = false;
    for (k, &i) in best.iter().enumerate() {
        /* Get choice: i */

        /* Ignore non-changes */
        if i == borg_best_stuff_order(k as i32) || NO_SLOT == i {
            continue;
        }

        found = true;
        let item = if i < 100 {
            &st.items.items[usize::from(i)]
        } else {
            st.borg.goal.best_item[k].home = true;
            &st.store.shops[BORG_HOME].ware[usize::from(i - 100)]
        };

        /* hopefully this is enough to get a unique item */
        let (tval, pval, sval) = (item.tval, item.pval, item.sval);
        let b = &mut st.borg.goal.best_item[k];
        b.tval = tval;
        b.pval = pval;
        b.sval = sval;
    }
    /* if we didn't change anything, clear out the changes */
    if !found {
        borg_clear_best(st);
    }

    /* we have done our best optimization.  */
    st.borg.goal.do_best = false;

    /* if we are being super chatty, spit out the new list of best */
    if st.cfg(BORG_VERBOSE) != 0 {
        if found {
            for &b in &best {
                let item = if b >= 100 {
                    st.store.shops[BORG_HOME].ware[usize::from(b - 100)]
                } else {
                    st.items.items[usize::from(b)]
                };
                note_desc(st, "new best: ", &item, "");
            }
        } else {
            borg_note(st, "new best: no changes");
        }
    }

    borg_one_step_wearing_best(st, 0)
}

/// Clear out the "best" swap list and trigger to do another
/// (`borg_clear_best`).
pub fn borg_clear_best(st: &mut BorgState) {
    st.borg.goal.best_item = Vec::new();
    st.borg.goal.do_best = true;
}

/// Scan the item list and recharge items before leaving the level
/// (`borg_wear_recharge`). Rods are not recharged from this, except rods
/// of recall.
pub fn borg_wear_recharge(st: &mut BorgState) -> bool {
    let mut b_i: i32 = -1;
    let mut b_slot: i32 = -1;

    /* No resting in danger */
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    if !borg_flow_misc::borg_check_rest(st, y, x) {
        return false;
    }

    /* Not if hungry */
    if st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Look for an (wearable- non rod) item to recharge */
    /* (INVEN_TOTAL: the pack and the equipment) */
    for i in 0..quiver_start() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* skip items that are charged */
        if item.timeout == 0 {
            continue;
        }

        /* Where can it be worn? */
        let slot = borg_wield_slot(item);

        /* if this is a rod, only count it if it is a rod of recall */
        /* and we are in town.  This is to prevent walking down when */
        /* the borg has a perfectly serviceable rod */
        if u32::from(item.tval) == TV_ROD
            && i32::from(item.sval) != st.item_val.sv_rod_recall
            && st.borg.tr(BI_CDEPTH) == 0
        {
            continue;
        }

        /* skip non-ego lights, No need to rest to recharge a torch, which uses
         * fuels turns in o_ptr->timeout */
        if u32::from(item.tval) == TV_LIGHT
            && (i32::from(item.sval) == st.item_val.sv_light_torch
                || i32::from(item.sval) == st.item_val.sv_light_lantern)
        {
            continue;
        }

        /* note this one */
        b_i = i;
        b_slot = slot;
    }

    /* nothing found */
    if b_i == -1 {
        return false;
    }

    let rest = |st: &mut BorgState| {
        borg_keypress(st, keycode_t::from(b'R'));
        borg_keypress(st, keycode_t::from(b'7'));
        borg_keypress(st, keycode_t::from(b'5'));
        borg_keypress(st, KC_ENTER);
    };

    if b_i >= inven_wield() || u32::from(st.items.items[b_i as usize].tval) == TV_ROD {
        /* Item is worn, no swap is nec. */
        let item = st.items.items[b_i as usize];
        note_desc(st, "# Waiting for '", &item, "' to Recharge.");

        /* Rest for a while */
        rest(st);

        /* done */
        return true;
    }
    /* Item must be worn to be recharged
     */
    if b_slot >= inven_wield() {
        /* wear the item */
        borg_note(st, "# Swapping Item for Recharge.");
        borg_keypress(st, ESCAPE);
        borg_keypress(st, keycode_t::from(b'w'));
        borg_keypress(st, letter(b_i));

        /* rest for a while */
        rest(st);

        /* done */
        return true;
    }

    /* nothing to recharge */
    false
}

/// Check an item for being ammo (`borg_is_ammo`).
pub fn borg_is_ammo(tval: i32) -> bool {
    matches!(tval as u32, TV_SHOT | TV_ARROW | TV_BOLT)
}

/// `borg_init_item_wear`
pub fn borg_init_item_wear(st: &mut BorgState) {
    borg_free_item_wear(st);

    /* Track the worn items to avoid loops */
    let w = &mut st.wear;
    w.track_worn_num = 0;
    w.track_worn_size = 10;
    w.track_worn_time = 0;
    w.track_worn_name1 = vec![0; w.track_worn_size as usize];

    let z = engine::z_info();
    if i32::from(z.equip_slots_max) + i32::from(z.pack_size) >= 100 {
        engine::msg(st, b"pack size plus equipment slots exceeds 100.  Need a rewrite of best_item code");
        st.init.init_failure = true;
    }
}

/// `borg_free_item_wear`
pub fn borg_free_item_wear(st: &mut BorgState) {
    st.wear.track_worn_name1 = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ammo_tvals() {
        for tval in -2..300 {
            let want = tval == TV_SHOT as i32 || tval == TV_ARROW as i32 || tval == TV_BOLT as i32;
            assert_eq!(borg_is_ammo(tval), want, "tval {}", tval);
        }
    }

    /// v12R: the upstream emergency-swap rule. The lower danger wins and the
    /// weapon wins a tie; an absent candidate keeps the 9999 sentinel.
    #[test]
    fn emergency_swap_chooses_the_lower_danger() {
        // armour only: the armour is worn
        assert_eq!(backup_swap_choice(3, 0, 40, 9999), (40, 2));
        // weapon only: the weapon is worn
        assert_eq!(backup_swap_choice(0, 5, 9999, 30), (30, 4));
        // both: the lower danger wins
        assert_eq!(backup_swap_choice(3, 5, 40, 30), (30, 4));
        assert_eq!(backup_swap_choice(3, 5, 30, 40), (30, 2));
        // a tie goes to the weapon
        assert_eq!(backup_swap_choice(3, 5, 40, 40), (40, 4));
        // neither candidate: the sentinel is returned, and the acceptance
        // threshold rejects it, so no key is queued from it
        assert_eq!(backup_swap_choice(0, 0, 9999, 9999), (9999, -1));
    }
}

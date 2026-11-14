//! Port of `borg-item-use.c`: use potions, scrolls, rods, wands, staffs
//! and rings.
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

use crate::borg::BorgTime;
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note_bytes, ESCAPE};
use crate::borg_item::inven_wield;
use crate::borg_item_analyze::borg_obj_has_effect;
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_okay_fail};
use crate::engine;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{
    HERBAL_CURING, RECHARGING, REMEMBRANCE, REMOVE_HUNGER, REVITALIZE, UNHOLY_REPRIEVE,
};
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_CDEPTH, BI_CURHP, BI_DEV, BI_FRACT, BI_ISBLIND, BI_ISCONFUSED, BI_ISFIXCON,
    BI_ISFIXDEX, BI_ISFIXEXP, BI_ISFIXINT, BI_ISFIXSTR, BI_ISFIXWIS, BI_ISFORGET, BI_ISHUNGRY,
    BI_MAXHP, BI_RBLIND, BI_RCONF,
};
use crate::headers::list_effects_h::EF_NOURISH;
use crate::headers::list_tvals_h::{
    TV_DRAG_ARMOR, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_RING, TV_ROD, TV_SCROLL, TV_STAFF, TV_WAND,
};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// borg-item-use.c function statics.
#[derive(Default)]
pub struct ItemUseState {
    /// `when_last_quaff` (static in `borg_quaff_crit`).
    pub when_last_quaff: BorgTime,
}

/// `all_letters_nohjkl[i]` as a key (a C `char`, sign-extended to
/// `keycode_t`).
fn letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// A key for a character literal.
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// The item description up to its NUL (`%s` of `item->desc`).
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `borg_note(format("<pre>%s<post>", item->desc))`.
fn note_desc(st: &mut BorgState, pre: &[u8], item_idx: usize, post: &[u8]) {
    let mut note = pre.to_vec();
    note.extend_from_slice(&desc(&st.items.items[item_idx]));
    note.extend_from_slice(post);
    borg_note_bytes(st, &note);
}

/// `borg.goal.shop = borg.goal.ware = borg.goal.item = -1;`
fn clear_shop_goals(st: &mut BorgState) {
    st.borg.goal.item = -1;
    st.borg.goal.ware = st.borg.goal.item;
    st.borg.goal.shop = st.borg.goal.ware;
}

/// `skill = BI_DEV`, cut by confusion: the device skill used for the
/// fail checks.
fn device_skill(st: &BorgState) -> i32 {
    /* Base chance of success */
    let mut skill = st.borg.tr(BI_DEV);

    /* Confusion hurts skill */
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        skill = skill * 75 / 100;
    }
    skill
}

/// `100 * ((skill - lev) - (141 - 1)) / ((lev - skill) - (100 - 10))`.
/// The divisor is 0 when `lev - skill == 90` (C bug 20: a division by
/// zero, a crash in C and a panic here).
fn device_fail(skill: i32, lev: i32) -> i32 {
    100 * ((skill - lev) - (141 - 1)) / ((lev - skill) - (100 - 10))
}

/// Index of the last item in the pack that is not empty, has the given
/// tval test true and is not aware (the `*_unknown` scans), or -1.
fn last_unaware(st: &BorgState, tval_ok: impl Fn(u32) -> bool) -> i32 {
    let mut n = -1;

    /* Scan the pack */
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Require correct tval */
        if !tval_ok(u32::from(item.tval)) {
            continue;
        }

        /* Skip aware items */
        if item.aware {
            continue;
        }

        /* Save this item */
        n = i;
    }
    n
}

/// Quaff a potion of cure critical wounds. This is a special case
/// for several reasons.
///   1) it is usually the only healing potion we have on us
///   2) we should try to conserve a couple for when we really need them
///   3) if we are burning through them fast we should probably teleport out
///      of the fight.
///   4) When it is the only/best way out of danger, drink away
pub fn borg_quaff_crit(st: &mut BorgState, no_check: bool) -> bool {
    if no_check {
        if borg_quaff_potion(st, st.item_val.sv_potion_cure_critical) {
            st.item_use.when_last_quaff = st.borg.time.now;
            return true;
        }
        return false;
    }

    /* Avoid drinking CCW twice in a row. */
    let last = st.item_use.when_last_quaff;
    let elapsed = st.borg.time.now - last;
    if last != 0 && elapsed <= 4 && elapsed > 0 && (engine::randint0(100) + 1 < 75) {
        return false;
    }

    /* Save the last two for when we really need them */
    if st.borg.tr(BI_ACCW) < 2 {
        return false;
    }

    if borg_quaff_potion(st, st.item_val.sv_potion_cure_critical) {
        st.item_use.when_last_quaff = st.borg.time.now;
        return true;
    }
    false
}

/// Attempt to quaff the given potion (by sval).
pub fn borg_quaff_potion(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that potion */
    let i = borg_slot(st, TV_POTION as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Quaffing ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'q'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals. Inventory changed so goals need to be redone. */
    clear_shop_goals(st);

    /* Success */
    true
}

/// Attempt to quaff an unknown potion.
pub fn borg_quaff_unknown(st: &mut BorgState) -> bool {
    let n = last_unaware(st, |t| t == TV_POTION);

    /* None available */
    if n < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Quaffing unknown potion ", n as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'q'));
    borg_keypress(st, letter(n));

    /* Clear "shop" goals. Inventory changed so goals need to be redone. */
    clear_shop_goals(st);

    st.borg.trying_unknown = true;

    /* Success */
    true
}

/// Attempt to read the given scroll (by sval).
pub fn borg_read_scroll(st: &mut BorgState, sval: i32) -> bool {
    /* Dark */
    if engine::no_light() {
        return false;
    }

    /* Blind or Confused or Amnesia*/
    if st.borg.tr(BI_ISBLIND) != 0
        || st.borg.tr(BI_ISCONFUSED) != 0
        || st.borg.tr(BI_ISFORGET) != 0
    {
        return false;
    }

    /* Look for that scroll */
    let i = borg_slot(st, TV_SCROLL as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Reading ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);
    borg_keypress(st, key(b'r'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals */
    clear_shop_goals(st);

    /* Success */
    true
}

/// Attempt to read an unknown scroll.
pub fn borg_read_unknown(st: &mut BorgState) -> bool {
    let n = last_unaware(st, |t| t == TV_SCROLL);

    /* None available */
    if n < 0 {
        return false;
    }

    /* Dark */
    if engine::no_light() {
        return false;
    }

    /* Blind or Confused */
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Reading unknown scroll ", n as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'r'));
    borg_keypress(st, letter(n));

    /* In case it is ID scroll, ESCAPE out. */
    borg_keypress(st, ESCAPE);

    /* Clear "shop" goals */
    clear_shop_goals(st);

    st.borg.trying_unknown = true;

    /* Success */
    true
}

/// Attempt to eat the given food or mushroom.
pub fn borg_eat(st: &mut BorgState, tval: i32, sval: i32) -> bool {
    /* Look for that food */
    let i = borg_slot(st, tval, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Eating ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'E'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals */
    clear_shop_goals(st);

    /* Success */
    true
}

/// Attempt to eat an unknown food/mushroom. This is done in emergencies.
pub fn borg_eat_unknown(st: &mut BorgState) -> bool {
    let n = last_unaware(st, |t| t == TV_FOOD || t == TV_MUSHROOM);

    /* None available */
    if n < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Eating unknown mushroom ", n as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'E'));
    borg_keypress(st, letter(n));

    /* Clear "shop" goals */
    clear_shop_goals(st);

    st.borg.trying_unknown = true;

    /* Success */
    true
}

/// Prevent starvation by any means possible.
pub fn borg_eat_food_any(st: &mut BorgState) -> bool {
    let pack_size = usize::from(engine::z_info().pack_size);

    /* Scan the inventory for "normal" food */
    for i in 0..pack_size {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip unknown food */
        if !item.aware {
            continue;
        }

        /* Skip non-food */
        if u32::from(item.tval) != TV_FOOD {
            continue;
        }

        /* Eat something of that type */
        let (tval, sval) = (i32::from(item.tval), i32::from(item.sval));
        if borg_eat(st, tval, sval) {
            return true;
        }
    }

    /* Scan the inventory for "okay" food */
    for i in 0..pack_size {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip unknown food */
        if !item.aware {
            continue;
        }

        /* Skip non-food */
        if u32::from(item.tval) != TV_FOOD && u32::from(item.tval) != TV_MUSHROOM {
            continue;
        }

        /* Skip non-food */
        if !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 0)
            && !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 2)
            && !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 3)
        {
            continue;
        }

        /* Eat something of that type */
        let (tval, sval) = (i32::from(item.tval), i32::from(item.sval));
        if borg_eat(st, tval, sval) {
            return true;
        }
    }

    /*
     * Try potions that can provide nutrition.  First try ones that are
     * pure nutrition without additional effects.
     */
    if borg_quaff_potion(st, st.item_val.sv_potion_slime_mold) {
        return true;
    }
    /*
     * Then try those that, besides the nourishment, only have negative
     * effects.  But only try if there's protection against the negative effect.
     */
    if (st.borg.tr(BI_FRACT) != 0
        && (borg_quaff_potion(st, st.item_val.sv_potion_sleep)
            || borg_quaff_potion(st, st.item_val.sv_potion_slowness)))
        || (st.borg.tr(BI_RBLIND) != 0 && borg_quaff_potion(st, st.item_val.sv_potion_blindness))
        || (st.borg.tr(BI_RCONF) != 0 && borg_quaff_potion(st, st.item_val.sv_potion_confusion))
    {
        return true;
    }
    /* Consume in order, when hurting */
    if (st.borg.tr(BI_CURHP) < 4 || st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP))
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing))
    {
        return true;
    }

    /* Nothing */
    false
}

/// Checks rod (by sval) and make a fail check on it.
pub fn borg_equips_rod(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_ROD as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage (at least 1/2 chance of success. */
    if fail > 500 {
        return false;
    }

    /* Yep we got one */
    true
}

/// Attempt to zap the given (charged) rod (by sval).
pub fn borg_zap_rod(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that rod */
    let i = borg_slot(st, TV_ROD as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Still charging */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage */
    if sval != st.item_val.sv_rod_recall && fail > 500 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Zapping ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'z'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Attempt to use the given (charged) staff (by sval).
pub fn borg_use_staff(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Using ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'u'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Attempt to use an unknown staff. This is done in emergencies.
pub fn borg_use_unknown(st: &mut BorgState) -> bool {
    let n = last_unaware(st, |t| t == TV_STAFF);

    /* None available */
    if n < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Using unknown Staff ", n as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'u'));
    borg_keypress(st, letter(n));

    /* In case it is ID staff, ESCAPE out. */
    borg_keypress(st, ESCAPE);

    st.borg.trying_unknown = true;

    /* Success */
    true
}

/// Attempt to use the given (charged) staff (by sval) and make a fail
/// check on it.
pub fn borg_use_staff_fail(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > 500 {
        if sval != st.item_val.sv_staff_teleportation {
            return false;
        }

        /* We need to give some "desperation attempt to teleport staff" */
        if st.borg.tr(BI_ISCONFUSED) == 0 && st.borg.tr(BI_ISBLIND) == 0 {
            /* We really have no chance, return false, attempt the scroll
             * (C tests `fail > 500` again here; it is true) */
            return false;
        }
        /* We might have a slight chance, or we cannot not read */
    }

    /* Log the message */
    note_desc(st, b"# Using ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'u'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Checks staff (by sval) and make a fail check on it.
pub fn borg_equips_staff_fail(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* If its a Destruction, we only use it in emergencies, attempt it */
    if sval == st.item_val.sv_staff_destruction {
        return true;
    }

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > 500 {
        /* No real chance of success on other types of staffs */
        if sval != st.item_val.sv_staff_teleportation {
            return false;
        }

        /* We need to give some "desperation attempt to teleport staff" */
        if sval == st.item_val.sv_staff_teleportation && st.borg.tr(BI_ISCONFUSED) == 0 {
            /* We really have no chance, return false, attempt the scroll */
            if fail < 650 {
                return false;
            }
        }

        /* We might have a slight chance (or its a Destruction), continue on */
    }

    /* Yep we got one */
    true
}

/// Attempt to aim the given (charged) wand (by sval).
pub fn borg_aim_wand(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that wand */
    let i = borg_slot(st, TV_WAND as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Aiming ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'a'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// `INVEN_RIGHT`
fn inven_right() -> i32 {
    inven_wield() + 2
}

/// `INVEN_LEFT`
fn inven_left() -> i32 {
    inven_wield() + 3
}

/// `INVEN_BODY`
fn inven_body() -> i32 {
    inven_wield() + 6
}

/// `INVEN_TOTAL`: `INVEN_WIELD + 12` (the c2rust constant).
fn inven_total() -> i32 {
    inven_wield() + 12
}

/// Check and see if borg is wielding a ring and if he will pass a fail
/// check.
pub fn borg_equips_ring(st: &mut BorgState, ring_sval: i32) -> bool {
    for i in inven_right()..=inven_left() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 || !item.aware {
            continue;
        }

        /* Skip incorrect armors */
        if u32::from(item.tval) != TV_RING {
            continue;
        }
        if i32::from(item.sval) != ring_sval {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /*  Make Sure is IDed */
        if !item.ident {
            continue;
        }

        /* check on fail rate */

        /* Extract the item level */
        let lev = i32::from(item.level);

        /* Base chance of success, confusion hurts skill */
        let skill = device_skill(st);

        /* High level objects are harder */
        let fail = device_fail(skill, lev);

        /* Roll for usage, but if its a Teleport be generous. */
        if fail > 500 {
            continue;
        }

        /* Success */
        return true;
    }

    false
}

/// Attempt to use the given ring.
pub fn borg_activate_ring(st: &mut BorgState, ring_sval: i32) -> bool {
    /* Check the equipment */
    for i in inven_right()..=inven_left() {
        let item = &st.items.items[i as usize];

        /*  Make Sure item is IDed */
        if !item.aware {
            continue;
        }

        /* Skip incorrect mails */
        if u32::from(item.tval) != TV_RING {
            continue;
        }
        if i32::from(item.sval) != ring_sval {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /*  Make Sure item is IDed */
        if !item.ident {
            continue;
        }

        /* Log the message */
        note_desc(st, b"# Activating ring ", i as usize, b".");

        /* Perform the action */
        borg_keypress(st, key(b'A'));
        borg_keypress(st, letter(i - inven_wield()));

        /* Success */
        return true;
    }

    false
}

/// The checks shared by `borg_equips_dragon` and `borg_activate_dragon`:
/// is the body armour an aware, identified, charged dragon armour of
/// this sval?
fn dragon_ready(item: &borg_item, drag_sval: i32) -> bool {
    if item.iqty == 0 || !item.aware {
        return false;
    }

    /* Skip incorrect armours */
    if u32::from(item.tval) != TV_DRAG_ARMOR {
        return false;
    }
    if i32::from(item.sval) != drag_sval {
        return false;
    }

    /* Check charge */
    if item.timeout != 0 {
        return false;
    }

    /*  Make Sure Mail is IDed */
    if !item.ident {
        return false;
    }
    true
}

/// Check and see if borg is wielding a dragon armor and if he will pass a
/// fail check.
pub fn borg_equips_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
    /* Check the equipment */
    let item = &st.items.items[inven_body() as usize];

    if !dragon_ready(item, drag_sval) {
        return false;
    }

    /* check on fail rate
     * The fail check is automatic for dragon armor.  It is an attack
     * item.  He should not sit around failing 5 or 6 times in a row.
     * he should attempt to activate it, and if he is likely to fail, then
     * eh should look at a different attack option.  We are assuming
     * that the fail rate is about 50%.  So He may still try to activate it
     * and fail.  But he will not even try if he has negative chance or
     * less than twice the USE_DEVICE variable
     */
    /* Extract the item level */
    let lev = i32::from(item.level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let numerator = (skill - lev) - (141 - 1);
    let mut denominator = (lev - skill) - (100 - 10);

    /* Make sure that we don't divide by zero */
    if denominator == 0 {
        denominator = if numerator > 0 { 1 } else { -1 };
    }

    let fail = (100 * numerator) / denominator;

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > 500 {
        return false;
    }

    /* Success */
    true
}

/// Attempt to use the given dragon armour.
pub fn borg_activate_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
    /* Check the equipment */
    let body = inven_body();
    if !dragon_ready(&st.items.items[body as usize], drag_sval) {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Activating dragon scale ", body as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'A'));
    borg_keypress(st, letter(body - inven_wield()));

    /* Success */
    true
}

/// Attempt to use the given artifact.
pub fn borg_activate_item(st: &mut BorgState, activation: i32) -> bool {
    /* a quick check of the array */
    if st.borg.activation[activation as usize] == 0 {
        return false;
    }

    /* Check the equipment */
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 {
            continue;
        }

        /* Skip wrong activation*/
        if item.activ_idx != activation {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /* Log the message */
        note_desc(st, b"# Activating item ", i as usize, b".");

        /* Perform the action */
        borg_keypress(st, key(b'A'));
        borg_keypress(st, letter(i - inven_wield()));

        /* Success */
        return true;
    }

    /* Oops */
    false
}

/// Check and see if borg is wielding an item with this activation.
pub fn borg_equips_item(st: &mut BorgState, activation: i32, check_charge: bool) -> bool {
    /* a quick check of the array */
    if st.borg.activation[activation as usize] == 0 {
        return false;
    } else if !check_charge {
        return true;
    }

    /* Check the equipment */
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 {
            continue;
        }

        /* Skip wrong activation */
        if item.activ_idx != activation {
            continue;
        }

        /* Check charge.  */
        if check_charge && item.timeout >= 1 {
            continue;
        }

        /* Success */
        return true;
    }

    /* I do not have it or it is not charged */
    false
}

/// Return the relative chance for failure to activate an item.
/// The lower the number, the better the chance of success
/// 200 is 80% of success
/// 600 is 40% chance of success
pub fn borg_activate_failure(st: &mut BorgState, tval: i32, sval: i32) -> i32 {
    /* Look for that item */
    let i = borg_slot(st, tval, sval);

    /* None available */
    if i < 0 {
        return 100;
    }

    let item = &st.items.items[i as usize];

    /* No charges */
    if item.pval == 0 {
        return 100;
    }

    /* no known activation */
    if item.activ_idx == 0 {
        return 100;
    }

    /* Extract the item level */
    let lev = i32::from(item.level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    device_fail(skill, lev)
}

/// Use things in a useful, but non-essential, manner.
pub fn borg_use_things(st: &mut BorgState) -> bool {
    /* Quaff experience restoration potion */
    if st.borg.tr(BI_ISFIXEXP) != 0
        && (borg_spell(st, REVITALIZE)
            || borg_spell(st, REMEMBRANCE)
            || (st.borg.tr(BI_CURHP) > 90 && borg_spell(st, UNHOLY_REPRIEVE))
            || borg_activate_item(st, st.act.restore_exp)
            || borg_activate_item(st, st.act.restore_st_lev)
            || borg_activate_item(st, st.act.restore_life)
            || borg_quaff_potion(st, st.item_val.sv_potion_restore_life))
    {
        return true;
    }

    /* just drink the stat gains, at this dlevel we won't need cash */
    if borg_quaff_potion(st, st.item_val.sv_potion_inc_str)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_int)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_wis)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_dex)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_con)
    {
        return true;
    }

    let mush = TV_MUSHROOM as i32;

    /* Quaff potions of "restore" stat if needed */
    if (st.borg.tr(BI_ISFIXSTR) != 0
        && (borg_quaff_potion(st, st.item_val.sv_potion_inc_str)
            || borg_eat(st, mush, st.item_val.sv_mush_purging)
            || borg_activate_item(st, st.act.shroom_purging)
            || borg_activate_item(st, st.act.restore_str)
            || borg_activate_item(st, st.act.restore_all)
            || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXINT) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_int)
                || borg_activate_item(st, st.act.restore_int)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXWIS) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_wis)
                || borg_activate_item(st, st.act.restore_wis)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXDEX) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_dex)
                || borg_activate_item(st, st.act.restore_dex)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXCON) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_con)
                || borg_activate_item(st, st.act.restore_con)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_purging)
                || borg_activate_item(st, st.act.shroom_purging)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
    {
        return true;
    }

    /* Use some items right away */
    for i in 0..usize::from(engine::z_info().pack_size) {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 || !item.aware {
            continue;
        }

        let sval = i32::from(item.sval);

        /* Process "force" items */
        match u32::from(item.tval) {
            TV_POTION => {
                /* Check the potion */
                if sval == st.item_val.sv_potion_enlightenment {
                    /* Never quaff these in town */
                    if st.borg.tr(BI_CDEPTH) != 0 && borg_quaff_potion(st, sval) {
                        return true;
                    }
                } else if sval == st.item_val.sv_potion_inc_all {
                    /* Try quaffing the potion */
                    if borg_quaff_potion(st, sval) {
                        return true;
                    }
                }
            }
            TV_SCROLL => {
                /* Check Blind/Confused */
                if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
                    continue;
                }

                /* XXX XXX XXX Dark */

                /* Check the scroll */
                if sval == st.item_val.sv_scroll_mapping
                    || sval == st.item_val.sv_scroll_acquirement
                    || sval == st.item_val.sv_scroll_star_acquirement
                {
                    /* Never read these in town */
                    if st.borg.tr(BI_CDEPTH) == 0 {
                        continue;
                    }

                    /* Try reading the scroll */
                    if borg_read_scroll(st, sval) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    /* Eat food */
    if st.borg.tr(BI_ISHUNGRY) != 0 {
        let food = TV_FOOD as i32;
        /* Attempt to satisfy hunger */
        if borg_spell(st, REMOVE_HUNGER)
            || borg_spell(st, HERBAL_CURING)
            || borg_quaff_potion(st, st.item_val.sv_potion_slime_mold)
            || borg_eat(st, food, st.item_val.sv_food_slime_mold)
            || borg_eat(st, food, st.item_val.sv_food_slice)
            || borg_eat(st, food, st.item_val.sv_food_apple)
            || borg_eat(st, food, st.item_val.sv_food_pint)
            || borg_eat(st, food, st.item_val.sv_food_handful)
            || borg_eat(st, food, st.item_val.sv_food_honey_cake)
            || borg_eat(st, food, st.item_val.sv_food_ration)
            || borg_eat(st, food, st.item_val.sv_food_waybread)
            || borg_eat(st, food, st.item_val.sv_food_draught)
            || borg_activate_item(st, st.act.food_waybread)
        {
            return true;
        }
    }

    /* Nothing to do */
    false
}

/// Recharge things.
///
/// XXX XXX XXX Prioritize available items
pub fn borg_recharging(st: &mut BorgState) -> bool {
    /* Forbid blind/confused */
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* XXX XXX XXX Dark */

    /* Look for an item to recharge */
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip non-identified items */
        if !item.ident || !item.aware {
            continue;
        }

        /* assume we can't charge it. */
        let mut charge = false;

        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);
        let pval = i32::from(item.pval);

        /* Wands with no charges can be charged */
        if tval == TV_WAND && pval <= 1 {
            charge = true;
        }

        /* recharge staves sometimes */
        if tval == TV_STAFF {
            /* Allow staves to be recharged at 2 charges if
             * the borg has the big recharge spell. And its not a *Dest*
             */
            if pval < 2
                && borg_spell_okay_fail(st, RECHARGING, 96)
                && sval < st.item_val.sv_staff_power
            {
                charge = true;
            }

            /* recharge any staff at 0 charges */
            if pval <= 1 {
                charge = true;
            }

            /* Staves of teleport get recharged at 2 charges in town */
            if sval == st.item_val.sv_staff_teleportation && pval < 3 && st.borg.tr(BI_CDEPTH) == 0
            {
                charge = true;
            }

            /* They stack.  If quantity is 4 and pval is 1, then there are 4
             * charges. */
            if i32::from(item.iqty) + pval >= 4 && pval >= 1 {
                charge = false;
            }
        }

        /* get the next item if we are not to charge this one */
        if !charge {
            continue;
        }

        /* Attempt to recharge */
        if borg_read_scroll(st, st.item_val.sv_scroll_recharging)
            || borg_spell_fail(st, RECHARGING, 96)
            || borg_activate_item(st, st.act.recharge)
        {
            /* Message (the item is unchanged by the calls above) */
            let mut note = b"Recharging ".to_vec();
            note.extend_from_slice(&desc(&st.items.items[i as usize]));
            note.extend_from_slice(
                format!(" with current charge of {}", st.items.items[i as usize].pval).as_bytes(),
            );
            borg_note_bytes(st, &note);

            /* Recharge the item */
            borg_keypress(st, letter(i));

            /* Success */
            return true;
        } else {
            /* if we fail once, no need to try again. */
            break;
        }
    }

    /* Nope */
    false
}

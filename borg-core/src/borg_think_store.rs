//! Port of `borg-think-store.c`: prepare to perform an action while in a
//! store.
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
//! This processing handles the highest level goals, and store interaction.
//!
//! Store interaction strategy
//!
//!   (1) Sell items to the home (for later use)
//!       optimize the stuff in the home... this involves buying and selling
//!       stuff not in the 'best' list.
//!       We sell anything we may need later (see step 4)
//!
//!   (2) Sell items to the shops (for money)
//!       We sell anything we do not actually need
//!
//!   (3) Buy items from the shops (for the player)
//!       We buy things that we actually need
//!
//!   (4) Buy items from the home (for the player)
//!       We buy things that we actually need (see step 1)
//!
//!   (5) Buy items from the shops (for the home)
//!       We buy things we may need later (see step 1)
//!
//!   (6) Buy items from the home (for the stores)
//!       We buy things we no longer need (see step 2)
//!
//!   The basic principle is that we should always act to improve our
//!   "status", and we should sometimes act to "maintain" our status,
//!   especially if there is a monetary reward.  But first we should
//!   attempt to use the home as a "stockpile", even though that is
//!   not worth any money, since it may save us money eventually.

use crate::borg::BorgTime;
use crate::borg_io::{borg_keypress, borg_note_bytes, ESCAPE};
use crate::borg_item::inven_wield;
use crate::borg_item_wear::{borg_best_stuff, borg_wear_stuff};
use crate::borg_store::{borg_home_full, borg_inventory_full, BORG_HOME, SHOP_MENU_ITEMS};
use crate::borg_store_buy::{
    borg_think_home_buy_swap_armour, borg_think_home_buy_swap_weapon, borg_think_home_buy_useful,
    borg_think_home_grab_useless, borg_think_shop_buy, borg_think_shop_buy_useful,
    borg_think_shop_grab_interesting,
};
use crate::borg_store_sell::{
    borg_think_home_sell_useful, borg_think_shop_sell, borg_think_shop_sell_useless,
};
use crate::engine;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_SELF_SCUM};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_FOOD, BI_GOLD, BI_ISCUT, BI_ISPOISONED, BI_LIGHT,
};
use crate::headers::list_tvals_h::TV_DIGGING;
use crate::state::BorgState;

/// `borg_timer(t)` (borg-think-dungeon-util.h): game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// A C string up to its NUL (`%s` of a `char` array).
fn cstr(buf: &[core::ffi::c_char]) -> impl Iterator<Item = u8> + '_ {
    buf.iter().take_while(|&&c| c != 0).map(|&c| c as u8)
}

/// `borg_shops[borg.goal.shop].ware[borg.goal.ware].desc`
fn goal_ware_desc(st: &BorgState) -> Vec<u8> {
    let shop = st.borg.goal.shop as usize;
    let ware = st.borg.goal.ware as usize;
    cstr(&st.store.shops[shop].ware[ware].desc).collect()
}

/// `borg_items[borg.goal.item].desc`
fn goal_item_desc(st: &BorgState) -> Vec<u8> {
    cstr(&st.items.items[st.borg.goal.item as usize].desc).collect()
}

/// `f_info[stores[borg.goal.shop].feat].name`
fn goal_shop_name(st: &BorgState) -> Vec<u8> {
    engine::store_feat_name(st.borg.goal.shop as usize)
}

/// `borg_note(format(...))` from byte pieces.
fn note(st: &mut BorgState, parts: &[&[u8]]) {
    let s = parts.concat();
    borg_note_bytes(st, &s);
}

/// `"# <what> '<ware>' at '<shop>'<after>"` for the goal ware.
fn note_ware_at(st: &mut BorgState, what: &str, after: &str) {
    let desc = goal_ware_desc(st);
    let name = goal_shop_name(st);
    note(st, &[what.as_bytes(), b" '", &desc, b"' at '", &name, b"'", after.as_bytes()]);
}

/// `"# <what> '<ware>'<after>"` for the goal ware.
fn note_ware(st: &mut BorgState, what: &str, after: &str) {
    let desc = goal_ware_desc(st);
    note(st, &[what.as_bytes(), b" '", &desc, b"'", after.as_bytes()]);
}

/// `borg.goal.shop = borg.goal.ware = borg.goal.item = -1`
fn clear_shop_goal(st: &mut BorgState) {
    st.borg.goal.item = -1;
    st.borg.goal.ware = st.borg.goal.item;
    st.borg.goal.shop = st.borg.goal.ware;
}

/// Four `ESCAPE`s: clear possible errors and leave the store.
fn leave(st: &mut BorgState) {
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);
}

/// Choose a shop to visit (`borg_choose_shop`).
pub fn borg_choose_shop(st: &mut BorgState) -> bool {
    let mut best_home_power: i32 = 0;

    /* Must be in town */
    if st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 2000 {
        return false;
    }
    if i32::from(st.borg.antibounce_count) > 1350 {
        return false;
    }

    /* Already flowing to a store to sell something */
    if st.borg.goal.shop != -1 && st.borg.goal.ware != -1 {
        return true;
    }

    /* If poisoned or bleeding -- flow to the Alchemy shop */
    if st.borg.tr(BI_ISCUT) != 0 || st.borg.tr(BI_ISPOISONED) != 0 {
        st.borg.goal.shop = 4;
    }

    /* If Starving  -- flow to general store */
    if st.borg.tr(BI_FOOD) == 0 || (st.borg.tr(BI_LIGHT) == 0 && st.borg.tr(BI_CLEVEL) >= 2) {
        /* G Store first */
        st.borg.goal.shop = 0;
    }

    /* if No Lantern -- flow to general store */
    if st.borg.tr(BI_LIGHT) == 1 && st.borg.tr(BI_GOLD) >= 100 {
        st.borg.goal.shop = 0;
    }

    /* If poisoned, bleeding, or needing to shop instantly
     * Buy items straight away, without having to see each shop
     */
    if (st.borg.tr(BI_LIGHT) == 0
        || st.borg.tr(BI_FOOD) == 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISPOISONED) != 0)
        || (st.borg.tr(BI_LIGHT) == 1 && st.borg.tr(BI_GOLD) >= 100 && st.borg.tr(BI_CLEVEL) < 10)
    {
        if borg_think_shop_buy_useful(st) {
            /* Message */
            note_ware_at(st, "# Buying", " immediately");

            /* Success */
            return true;
        }

        /* if temple is out of healing stuff, try the house */
        if borg_think_home_buy_useful(st) {
            /* Message */
            note_ware(st, "# Buying", " from the home immediately.");

            /* Success */
            return true;
        }
    }

    /* if we are already flowing toward a shop do not check again... */
    if st.borg.goal.shop != -1 && st.borg.goal.ware != -1 {
        return true;
    }

    /* Assume no important shop */
    clear_shop_goal(st);

    /* If it is time to try to put on our best stuff, and we have free slots, go
     * home */
    if st.borg.goal.do_best && !borg_home_full(st) && !borg_inventory_full(st) {
        st.borg.goal.shop = BORG_HOME as i16;
        return true;
    }

    /* if the borg is scumming for cash for the human player and not himself,
     * we don't want him messing with the home inventory
     */
    let scum = st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize];
    if st.borg.tr(BI_GOLD) < scum
        && scum != 0
        && st.borg.tr(BI_CDEPTH) == 0
        && st.borg.tr(BI_LIGHT) != 0
        && st.core.cfg[BORG_SELF_SCUM as usize] == 0
    {
        /* Step 0 -- Buy items from the shops (for the player while scumming) */
        if borg_think_shop_buy_useful(st) {
            /* Message */
            note_ware_at(st, "# Buying", " (money scumming)");

            /* Success */
            return true;
        } else {
            return false;
        }
    }

    /* Step 1 -- Sell items to the home */
    if borg_think_home_sell_useful(st, &mut best_home_power) {
        /* Message */
        if st.borg.goal.item != -1 {
            let desc = goal_item_desc(st);
            note(st, &[b"# Selling '", &desc, b"' to the home"]);
        } else {
            note_ware(st, "# Buying", " from the home (step 1)");
        }

        /* Success */
        return true;
    }

    /* Step 2 -- Sell items to the shops */
    if borg_think_shop_sell_useless(st) {
        /* Message */
        let desc = goal_item_desc(st);
        let name = goal_shop_name(st);
        note(st, &[b"# Selling '", &desc, b"' at '", &name, b"'"]);

        /* Success */
        return true;
    }

    /* Step 3 -- Buy items from the shops (for the player) */
    if borg_think_shop_buy_useful(st) {
        /* Message */
        let desc = goal_ware_desc(st);
        // `%c` of a C `char`: the byte itself
        let letter = SHOP_MENU_ITEMS[st.borg.goal.ware as usize];
        let name = goal_shop_name(st);
        note(
            st,
            &[b"# Buying '", &desc, b"'(", &[letter], b") at '", &name, b"' (for player 'b')"],
        );

        /* Success */
        return true;
    }

    /* Step 4 -- Buy items from the home (for the player) */
    if borg_think_home_buy_useful(st) {
        /* Message */
        note_ware(st, "# Buying", " from the home (step 4)");

        /* Success */
        return true;
    }

    /* get rid of junk from home first.  That way the home is 'uncluttered' */
    /* before you buy stuff for it.  This will prevent the problem where an */
    /* item has become a negative value and swapping in a '0' gain item */
    /* (like pottery) is better. */

    /* Step 5 -- Grab items from the home (for the shops) */
    if borg_think_home_grab_useless(st) {
        /* Message */
        note_ware(st, "# Grabbing (to sell)", " from the home");

        /* Success */
        return true;
    }

    /* Do not Stock Up the home while money scumming */
    if st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize] != 0 {
        return false;
    }

    /* Step 6 -- Buy items from the shops (for the home) */
    if borg_think_shop_grab_interesting(st) {
        /* Message */
        note_ware_at(st, "# Grabbing (for home)", "");

        /* Success */
        return true;
    }

    /* Step 7A -- Buy weapons from the home (as a backup item) */
    if crate::borg_trait::borg_uses_swaps(st) && borg_think_home_buy_swap_weapon(st) {
        /* Message */
        note_ware(st, "# Buying", " from the home as a backup");

        /* Success */
        return true;
    }
    /* Step 7B -- Buy armour from the home (as a backup item) */
    if crate::borg_trait::borg_uses_swaps(st) && borg_think_home_buy_swap_armour(st) {
        /* Message */
        note_ware(st, "# Buying", " from the home as a backup");

        /* Success */
        return true;
    }

    /* Failure */
    false
}

/// Deal with being in a store (`borg_think_store`).
pub fn borg_think_store(st: &mut BorgState) -> bool {
    /* Cheat the store number */
    st.think.shop_num = engine::player_square_shopnum() as i16;

    /* Clear the goal (the goal was probably going to a shop number) */
    st.borg.goal.type_0 = 0;

    /* Recheck spells */
    st.think.do_spell = true;

    /* Allow user abort */
    if st.borg.status.cancel {
        return true;
    }

    /* Do not allow a user key to interrupt the borg while in a store */
    st.borg.in_shop = true;

    /* HACK: Prevent clock wrapping */
    if borg_timer(st, st.borg.time.level) >= 20000 && borg_timer(st, st.borg.time.level) <= 20010
    {
        /* Clear Possible errors and leave the store */
        leave(st);
        return true;
    }

    /* Wear "optimal" equipment */
    if borg_best_stuff(st) {
        return true;
    }

    /* If using a digger, Wear "useful" equipment.
     * unless that digger is an artifact, then treat
     * it as a normal weapon
     */
    let wield = &st.items.items[inven_wield() as usize];
    if i32::from(wield.tval) == TV_DIGGING as i32 && wield.art_idx == 0 && borg_wear_stuff(st) {
        return true;
    }

    /* Choose a shop to visit.  Goal_shop indicates he is trying to sell
     * something somewhere. */
    if borg_choose_shop(st) {
        /* Note Pref. */
        let shop_num = st.think.shop_num;
        if shop_num != st.borg.goal.shop {
            let s = format!(
                "# Currently in store '{}' would prefer '{}'.",
                i32::from(shop_num) + 1,
                i32::from(st.borg.goal.shop) + 1
            );
            borg_note_bytes(st, s.as_bytes());
        } else {
            let s = format!("# Currently in preferred store '{}'.", i32::from(st.borg.goal.shop) + 1);
            borg_note_bytes(st, s.as_bytes());
        }

        /* Try to sell stuff */
        if borg_think_shop_sell(st) {
            return true;
        }

        /* Try to buy stuff */
        if borg_think_shop_buy(st) {
            return true;
        }
    }

    /* No shop */
    clear_shop_goal(st);

    /* Leave the store */
    leave(st);

    /* Done */
    true
}

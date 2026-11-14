//! Port of `borg-store-buy.c`: shopping and buying items or getting them
//! from the home.
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
//! The "what if" evaluations change `st.items.items` and the shop wares in
//! place, call `borg_notice` / `borg_power` / `borg_notice_home` /
//! `borg_power_home`, and restore the slots exactly as C does (struct
//! copies copy the `note` reference (`borg_item::NoteRef`), so a copied
//! item's note reads the live origin's current `desc`, as C's copied
//! pointer does). C's pointer `item` into a ware is read again
//! at each use (`ware`), never cached across a call.

use crate::borg_buy_strategy::{
    buy_identify_enabled, is_learning_candidate, reserve410_unmet, RESERVE410_CURE,
    RESERVE410_PHASE,
};
use crate::borg_home_notice::borg_notice_home;
use crate::borg_home_power::borg_power_home;
use crate::borg_inventory::{borg_first_empty_inventory_slot, borg_slot, borg_wield_slot};
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, ESCAPE, KC_ENTER};
use crate::borg_item::inven_wield;
use crate::borg_magic::{borg_get_book_num, borg_is_dungeon_book, borg_primarily_caster};
use crate::borg_power::borg_power;
use crate::borg_store::{BORG_HOME, SHOP_MENU_ITEMS};
use crate::engine;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_SELF_SCUM, BORG_VERBOSE};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_ACLW, BI_ACSW, BI_ALITE, BI_APHASE, BI_CCON, BI_CDEX, BI_CINT, BI_CLASS,
    BI_CLEVEL, BI_CSTR, BI_CWIS, BI_DEX_INDEX, BI_FIRST_CURSED, BI_FOOD, BI_GOLD,
    BI_GOOD_W_CHG, BI_ISCUT, BI_ISPOISONED, BI_LIGHT, BI_MAXCLEVEL, BI_QUIVER_SLOTS,
    BI_SAURON_DEAD,
};
use crate::headers::list_object_flags_h::{OF_BURNS_OUT, OF_TAKES_FUEL};
use crate::headers::list_tvals_h::*;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;

/// borg-store-buy.c globals: the money-scum target (file statics) and the
/// last items bought (to avoid buy/sell loops).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BuyState {
    /// `borg_money_scum_who` (static): the shop of the scum target.
    pub money_scum_who: i32,
    /// `borg_money_scum_ware` (static): the ware of the scum target.
    pub money_scum_ware: i32,
    /// `bought_item_tval`
    pub bought_item_tval: [i32; 10],
    /// `bought_item_sval`
    pub bought_item_sval: [i32; 10],
    /// `bought_item_pval`
    pub bought_item_pval: [i32; 10],
    /// `bought_item_store`
    pub bought_item_store: [i32; 10],
    /// `bought_item_num`: the number of entries in use, newest included
    /// (-1 at start; better-borg-c milestone9 patch 0094).
    pub bought_item_num: i32,
    /// `bought_item_nxt`: the next entry to write.
    pub bought_item_nxt: i32,
}

/// One entry of the "just bought" list: `bought_item_tval/sval/pval/store[i]`.
#[derive(Clone, Copy, Debug)]
pub struct BoughtItem {
    pub tval: i32,
    pub sval: i32,
    pub pval: i32,
    pub store: i32,
}

/// `bought_item_*[i]`; panics past the 10 entries.
pub fn bought_item(st: &BorgState, i: usize) -> BoughtItem {
    let b = &st.buy;
    BoughtItem {
        tval: b.bought_item_tval[i],
        sval: b.bought_item_sval[i],
        pval: b.bought_item_pval[i],
        store: b.bought_item_store[i],
    }
}

impl Default for BuyState {
    fn default() -> Self {
        BuyState {
            money_scum_who: 0,
            money_scum_ware: 0,
            bought_item_tval: [0; 10],
            bought_item_sval: [0; 10],
            bought_item_pval: [0; 10],
            bought_item_store: [0; 10],
            bought_item_num: -1,
            bought_item_nxt: 0,
        }
    }
}

/// `INVEN_RIGHT`
fn inven_right() -> i32 {
    inven_wield() + 2
}

/// `INVEN_LEFT`
fn inven_left() -> i32 {
    inven_wield() + 3
}

/// `INVEN_LIGHT`
fn inven_light() -> i32 {
    inven_wield() + 5
}

/// `PACK_SLOTS`: the pack slots not taken by the quiver.
fn pack_slots(st: &BorgState) -> i32 {
    i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS)
}

/// `z_info->store_max`
fn store_max() -> i32 {
    i32::from(engine::z_info().store_max)
}

/// `z_info->store_inven_max`
fn store_inven_max() -> i32 {
    i32::from(engine::z_info().store_inven_max)
}

/// `&borg_shops[k].ware[n]`
fn ware(st: &BorgState, k: i32, n: i32) -> &borg_item {
    &st.store.shops[k as usize].ware[n as usize]
}

/// `&mut borg_shops[k].ware[n]`
fn ware_mut(st: &mut BorgState, k: i32, n: i32) -> &mut borg_item {
    &mut st.store.shops[k as usize].ware[n as usize]
}

/// `memcpy(&borg_shops[k].ware[n], &borg_safe_shops[k].ware[n], ...)`
fn restore_ware(st: &mut BorgState, k: i32, n: i32) {
    st.store.shops[k as usize].ware[n as usize] = st.store.safe_shops[k as usize].ware[n as usize];
}

/// `memcpy(&borg_items[to], &borg_safe_shops[k].ware[n], ...)`
fn item_from_safe_ware(st: &mut BorgState, to: i32, k: i32, n: i32) {
    st.items.items[to as usize] = st.store.safe_shops[k as usize].ware[n as usize];
}

/// `memcpy(&borg_items[to], &safe_items[from], ...)`
fn item_from_safe(st: &mut BorgState, to: i32, from: i32) {
    st.items.items[to as usize] = st.items.safe_items[from as usize];
}

/// `borg_items[i]`
fn item_mut(st: &mut BorgState, i: i32) -> &mut borg_item {
    &mut st.items.items[i as usize]
}

/// `tval == TV_STAFF || tval == TV_WAND`
fn is_device(tval: u8) -> bool {
    let t = u32::from(tval);
    t == TV_STAFF || t == TV_WAND
}

/// `obj_kind_can_browse(&k_info[item->kind])`
fn can_browse(item: &borg_item) -> bool {
    engine::obj_kind_can_browse(item.kind as usize)
}

/// `iqty -= qty` on an `uint8_t`.
fn sub_iqty(q: &mut u8, qty: i32) {
    *q = (i32::from(*q) - qty) as u8;
}

/// `borg_note(format(before "%s" after, item->desc))`
fn note_desc(st: &mut BorgState, before: &str, item: &borg_item, after: &str) {
    let mut s = before.as_bytes().to_vec();
    s.extend(item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8));
    s.extend_from_slice(after.as_bytes());
    borg_note_bytes(st, &s);
}

/// `borg_min_item_quantity(&borg_shops[k].ware[n])` (borg-store-sell.c),
/// on the ware as it is now.
fn min_quantity(st: &mut BorgState, k: i32, n: i32) -> i32 {
    let w = *ware(st, k, n);
    crate::borg_store_sell::borg_min_item_quantity(st, &w)
}

/// Help decide if an item should be bought from a real store
/// (`borg_good_buy`).
///
/// We prevent the purchase of enchanted (or expensive) ammo,
/// so we do not spend all our money on temporary power.
///
/// if level 35, who needs cash?  buy the expensive ammo!
///
/// We prevent the purchase of low level discounted books,
/// so we will not waste slots on cheap books.
///
/// We prevent the purchase of items from the black market
/// which are often available at normal stores, currently,
/// this includes low level books, and all wands and staffs.
fn borg_good_buy(st: &mut BorgState, who: i32, ware_n: i32) -> bool {
    // the item does not change in here
    let item = *ware(st, who, ware_n);
    let tval = u32::from(item.tval);
    let sval = i32::from(item.sval);

    /* Check the object */
    match tval {
        TV_SHOT | TV_ARROW | TV_BOLT => {
            if st.borg.tr(BI_CLEVEL) < 35 {
                if item.to_h != 0 {
                    return false;
                }
                if item.to_d != 0 {
                    return false;
                }
            }
        }
        /* not our book */
        TV_PRAYER_BOOK | TV_MAGIC_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK
            if !can_browse(&item) =>
        {
            return false;
        }
        _ => {}
    }

    /* Don't buy from the BM until we are rich */
    if who == 6 {
        let iv = &st.item_val;
        let b = &st.borg;

        /* buying Remove Curse scroll is acceptable */
        if tval == TV_SCROLL && sval == iv.sv_scroll_remove_curse && b.tr(BI_FIRST_CURSED) != 0 {
            return true;
        }

        /* Buying certain special items are acceptable */
        let class = b.tr(BI_CLASS);
        if (tval == TV_POTION
            && (sval == iv.sv_potion_star_healing
                || sval == iv.sv_potion_life
                || sval == iv.sv_potion_healing
                || (sval == iv.sv_potion_inc_str && b.tr(BI_CSTR) < (18 + 100))
                || (sval == iv.sv_potion_inc_int && b.tr(BI_CINT) < (18 + 100))
                || (sval == iv.sv_potion_inc_wis && b.tr(BI_CWIS) < (18 + 100))
                || (sval == iv.sv_potion_inc_dex && b.tr(BI_CDEX) < (18 + 100))
                || (sval == iv.sv_potion_inc_con && b.tr(BI_CCON) < (18 + 100))))
            || (tval == TV_ROD
                && (sval == iv.sv_rod_healing
                    /* priests and paladins can cast recall */
                    || (sval == iv.sv_rod_recall
                        && class != CLASS_PRIEST
                        && class != CLASS_PALADIN)
                    /* druid and ranger can cast haste*/
                    || (sval == iv.sv_rod_speed && class != CLASS_DRUID && class != CLASS_RANGER)
                    /* mage and rogue can cast teleport away */
                    || (sval == iv.sv_rod_teleport_other
                        && class != CLASS_MAGE
                        && class != CLASS_ROGUE)
                    || (sval == iv.sv_rod_illumination && b.tr(BI_ALITE) == 0)))
            || (can_browse(&item)
                && b.amt_book[borg_get_book_num(sval) as usize] == 0
                && borg_is_dungeon_book(tval as i32, sval))
            || (tval == TV_SCROLL
                && (sval == iv.sv_scroll_teleport_level || sval == iv.sv_scroll_teleport))
        {
            /* Hack-- Allow the borg to scum for this Item */
            if st.core.cfg[BORG_SELF_SCUM as usize] != 0 /* borg is allowed to scum */
                && b.tr(BI_CLEVEL) >= 10 /* Be of sufficient level */
                && b.tr(BI_LIGHT) != 0 /* Have some Perma lite source */
                && b.tr(BI_FOOD) + i32::from(st.home.num_food) >= 100 /* Have plenty of food */
                && item.cost <= 85000
            /* Its not too expensive */
            {
                /* Good chance to thwart mugging */
                if engine::adj_dex_safe(b.tr(BI_DEX_INDEX) as usize) + b.tr(BI_CLEVEL) > 90 {
                    /* Record the amount that I need to make purchase */
                    st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize] = item.cost;
                    st.buy.money_scum_who = who;
                    st.buy.money_scum_ware = ware_n;
                }
            }

            /* Ok to buy this */
            return true;
        }

        if b.tr(BI_CLEVEL) < 15 && b.tr(BI_GOLD) < 20000 {
            return false;
        }
        if b.tr(BI_CLEVEL) < 35 && b.tr(BI_GOLD) < 15000 {
            return false;
        }
        if b.tr(BI_GOLD) < 10000 {
            return false;
        }
    }

    /* do not buy the item if I just sold it. */
    let mut p = 0;
    while p < st.store_sell.sold_item_num {
        let (s_tval, s_sval, s_store) = crate::borg_store_sell::sold_item(st, p as usize);
        if s_tval == tval as i32 && s_sval == sval && s_store == who {
            if st.core.cfg[BORG_VERBOSE as usize] != 0 {
                note_desc(st, "# Choosing not to buy back ", &item, "");
            }
            return false;
        }
        p += 1;
    }

    /* Do not buy a second digger */
    if tval == TV_DIGGING {
        /* scan for an existing digger */
        for ii in 0..i32::from(engine::z_info().pack_size) {
            /* skip non diggers */
            if u32::from(st.items.items[ii as usize].tval) == TV_DIGGING {
                return false;
            }
        }
    }

    /* Low level borgs should not waste the money on certain things */
    if st.borg.tr(BI_MAXCLEVEL) < 5 {
        /* next book, cant read it */
        if can_browse(&item) && sval >= 1 {
            return false;
        }
    }

    /* Not direct spell casters and the extra books */
    /* classes that are direct spell casters get more than 3 books */
    if !borg_primarily_caster() && st.borg.tr(BI_MAXCLEVEL) <= 8 && can_browse(&item) && sval >= 1 {
        return false;
    }

    /* Okay */
    true
}

/// Point the Borg at one shop ware for purchase. A pending pack-sale goal
/// (`goal.item`) is cleared first: `borg_think_shop_sell` would otherwise sell
/// it on entering the chosen shop instead of buying the ware.
fn set_buy_goal(st: &mut BorgState, k: i32, n: i32) {
    st.borg.goal.item = -1;
    st.borg.goal.shop = k as i16;
    st.borg.goal.ware = n as i16;
}

/// Buy the cheapest affordable deficit toward the experimental #410 reserve.
/// While that reserve is unmet, every other purchase in this function is
/// suppressed. This is deliberately opt-in: upstream rejected the strict
/// policy after its declared survival/progress margins failed (borg-rs#25).
fn borg_buy_reserve410(st: &mut BorgState) -> bool {
    let need_phase = RESERVE410_PHASE - st.borg.tr(BI_APHASE);
    let held_cure = st.borg.tr(BI_ACLW) + st.borg.tr(BI_ACSW) + st.borg.tr(BI_ACCW);
    let need_cure = RESERVE410_CURE - held_cure;
    let mut b_kr = -1;
    let mut b_nr = -1;
    let mut b_cr = 0;

    for k in 0..(store_max() - 1) {
        for n in 0..store_inven_max() {
            let item = *ware(st, k, n);
            if item.iqty == 0 || !borg_good_buy(st, k, n) {
                continue;
            }

            let tval = u32::from(item.tval);
            let sval = i32::from(item.sval);
            let phase_ware = need_phase > 0
                && tval == TV_SCROLL
                && sval == st.item_val.sv_scroll_phase_door;
            let cure_ware = need_cure > 0
                && tval == TV_POTION
                && (sval == st.item_val.sv_potion_cure_light
                    || sval == st.item_val.sv_potion_cure_serious
                    || sval == st.item_val.sv_potion_cure_critical);
            if (!phase_ware && !cure_ware) || st.borg.tr(BI_GOLD) < item.cost {
                continue;
            }
            if b_kr >= 0 && item.cost >= b_cr {
                continue;
            }
            b_kr = k;
            b_nr = n;
            b_cr = item.cost;
        }
    }

    if b_kr >= 0 {
        set_buy_goal(st, b_kr, b_nr);
        return true;
    }

    // Unaffordable or out of stock: retain the gold rather than spending it
    // on the old valuation's gear. This strictness is part of the experiment,
    // including the trade-off recorded by the rejected upstream cohorts.
    false
}

/// Buy one affordable still-unaware kind from the small #27 learning list.
/// Failure is deliberately non-blocking: the old valuation runs next.
fn borg_buy_identify(st: &mut BorgState) -> bool {
    let mut b_k = -1;
    let mut b_n = -1;
    let mut b_cost = 0;

    for k in 0..(store_max() - 1) {
        for n in 0..store_inven_max() {
            let item = *ware(st, k, n);
            if item.iqty == 0 || !borg_good_buy(st, k, n) || !is_learning_candidate(st, &item) {
                continue;
            }
            if st.borg.tr(BI_GOLD) < item.cost || (b_k >= 0 && item.cost >= b_cost) {
                continue;
            }
            b_k = k;
            b_n = n;
            b_cost = item.cost;
        }
    }

    if b_k >= 0 {
        let item = *ware(st, b_k, b_n);
        note_desc(st, "# Learning '", &item, "'");
        set_buy_goal(st, b_k, b_n);
        return true;
    }
    false
}

/// Step 3 -- buy "useful" things from a shop (to be used)
/// (`borg_think_shop_buy_useful`).
pub fn borg_think_shop_buy_useful(st: &mut BorgState) -> bool {
    let hole = borg_first_empty_inventory_slot(st);

    let mut b_k: i32 = -1;
    let mut b_n: i32 = -1;
    let mut b_c: i32 = 0;

    let mut fix = false;

    /* Require one empty slot */
    if hole == -1 {
        return false;
    }

    /* Already have a target 9-4-05*/
    if st.borg.goal.ware != -1 {
        return false;
    }

    // Post-M10 family variant: only an explicitly selected reserve410 changes
    // this decision. `current` goes straight to the v10 valuation below.
    if reserve410_unmet(st) {
        return borg_buy_reserve410(st);
    }
    if buy_identify_enabled(st) && borg_buy_identify(st) {
        return true;
    }

    /* Extract the "power" */
    let mut b_p: i32 = st.borg.power;

    /* Check the shops */
    for k in 0..(store_max() - 1) {
        /* If I am bad shape up, only see certain stores */
        if (st.borg.tr(BI_LIGHT) == 0 || st.borg.tr(BI_FOOD) == 0)
            && k != 0
            && k != BORG_HOME as i32
            && !(st.borg.tr(BI_FOOD) == 0 && k == 4)
        {
            continue;
        }
        /* Cures for poison and cuts are sold in the Alchemy shop */
        if (st.borg.tr(BI_ISCUT) != 0 || st.borg.tr(BI_ISPOISONED) != 0) && k != 4 {
            continue;
        }

        /* Scan the wares */
        for n in 0..store_inven_max() {
            /* Skip empty items */
            if ware(st, k, n).iqty == 0 {
                continue;
            }

            /* Skip "bad" buys */
            if !borg_good_buy(st, k, n) {
                continue;
            }

            /* Attempting to scum money, don't buy other stuff unless it is our
             * home or food-store */
            if st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize] != 0
                && (k != st.buy.money_scum_who || n != st.buy.money_scum_ware)
            {
                continue;
            }

            /* Require "sufficient" cash */
            if st.borg.tr(BI_GOLD) < ware(st, k, n).cost {
                continue;
            }

            let tval = u32::from(ware(st, k, n).tval);
            let sval = i32::from(ware(st, k, n).sval);

            /* Special check for 'immediate shopping' */
            if st.borg.tr(BI_FOOD) == 0
                && tval != TV_FOOD
                && !(tval == TV_SCROLL && sval == st.item_val.sv_scroll_satisfy_hunger)
            {
                continue;
            }

            /* Don't fill up on attack wands, its ok to buy a few */
            if tval == TV_WAND
                && (sval == st.item_val.sv_wand_magic_missile
                    || sval == st.item_val.sv_wand_stinking_cloud)
                && st.borg.tr(BI_GOOD_W_CHG) > 40
            {
                continue;
            }

            /* These wands are not useful later on, we need beefier attacks */
            if tval == TV_WAND
                && (sval == st.item_val.sv_wand_magic_missile
                    || sval == st.item_val.sv_wand_stinking_cloud)
                && st.borg.tr(BI_MAXCLEVEL) > 30
            {
                continue;
            }

            /* Save the number to trade */
            let qty = min_quantity(st, k, n);

            /* Remove one item from shop (sometimes) */
            sub_iqty(&mut ware_mut(st, k, n).iqty, qty);

            let mut slot: i32 = -1;

            /* require two empty slots to buy things to wield */
            if (hole + 1) < pack_slots(st) {
                /* Obtain "slot" */
                slot = borg_wield_slot(ware(st, k, n));

                /* XXX what if the item is a ring?  we have 2 ring slots --- copy it
                 * from the Home code */

                /* special cases for buying torches */
                if tval == TV_LIGHT && sval == st.item_val.sv_light_torch {
                    let light = &st.items.items[inven_light() as usize];

                    /* the borg will not replace his Brightness Torch with a plain one,
                     * so it ends up not buying any torches.  Force plain torches for
                     * purchase to be seen as fuel only
                     */
                    if engine::flag_has(&light.flags, OF_BURNS_OUT as i32) {
                        slot = -1;
                    }

                    /* the borg will buy torches to replace a lantern when he should
                     *  just refuel the lantern.  Again, make the torches just go to
                     *  inventory rather than equipment.
                     */
                    if i32::from(light.sval) == st.item_val.sv_light_lantern
                        && engine::flag_has(&light.flags, OF_TAKES_FUEL as i32)
                    {
                        slot = -1;
                    }
                }

                /* Hack, we keep diggers as a back-up, not to
                 * replace our current weapon
                 */
                if tval == TV_DIGGING {
                    slot = -1;
                }
            }

            let p: i32;

            /* Consider new equipment */
            if slot >= 0 {
                /* Move equipment into inventory */
                item_from_safe(st, hole, slot);

                /* Move new item into equipment */
                item_from_safe_ware(st, slot, k, n);

                /* Only a single item */
                item_mut(st, slot).iqty = qty as u8;

                /* Fix later */
                fix = true;

                /* Examine the inventory */
                crate::borg_trait::borg_notice(st, true);

                /* Evaluate the inventory */
                p = borg_power(st);

                /* Restore old item */
                item_from_safe(st, slot, slot);

                /* restore the hole */
                item_mut(st, hole).iqty = 0;
            }
            /* Consider new inventory */
            else {
                /* Move new item into inventory */
                item_from_safe_ware(st, hole, k, n);

                /* Only a single item */
                item_mut(st, hole).iqty = qty as u8;

                /* Fix later */
                fix = true;

                /* Examine the inventory */
                crate::borg_trait::borg_notice(st, true);

                /* Evaluate the equipment */
                p = borg_power(st);
            }

            /* Restore hole */
            item_mut(st, hole).iqty = 0;

            /* Restore shop item */
            restore_ware(st, k, n);

            /* Obtain the "cost" of the item */
            let c = ware(st, k, n).cost.wrapping_mul(qty);

            /* Ignore "bad" purchases */
            if p <= b_p {
                continue;
            }

            /* Ignore "expensive" purchases */
            if (p == b_p) && (c >= b_c) {
                continue;
            }

            /* Save the item and cost */
            b_k = k;
            b_n = n;
            b_p = p;
            b_c = c;
        }
    }

    /* Examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* Buy something */
    if (b_k >= 0) && (b_n >= 0) {
        /* Visit that shop */
        st.borg.goal.shop = b_k as i16;

        /* Buy that item */
        st.borg.goal.ware = b_n as i16;

        /* Success */
        return true;
    }

    /* Nope */
    false
}

/// Step 4 -- buy "useful" things from the home (to be used)
/// (`borg_think_home_buy_useful`).
pub fn borg_think_home_buy_useful(st: &mut BorgState) -> bool {
    let home = BORG_HOME as i32;
    let mut b_n: i32 = -1;
    // these keep their values from one ware to the next, as in C
    let mut charge_each: i32 = 0;
    let mut p_left: i32 = 0;
    let mut p_right: i32 = 0;

    let mut fix = false;

    /* Extract the "power" */
    let mut b_p: i32 = st.borg.power;

    /* Scan the home */
    for n in 0..store_inven_max() {
        /* Skip empty items */
        if ware(st, home, n).iqty == 0 {
            continue;
        }

        let tval = ware(st, home, n).tval;
        let sval = ware(st, home, n).sval;

        /* Skip it if I just sold it */
        let mut skip_it = false;
        let mut i = 0;
        while i < st.store_sell.sold_item_num {
            let (s_tval, s_sval, _) = crate::borg_store_sell::sold_item(st, i as usize);
            if s_tval == i32::from(tval) && s_sval == i32::from(sval) {
                if st.core.cfg[BORG_VERBOSE as usize] != 0 {
                    let w = *ware(st, home, n);
                    note_desc(st, "# Choosing not to buy back '", &w, "' from home.");
                }
                skip_it = true;
            }
            i += 1;
        }
        if skip_it {
            continue;
        }

        /* Reset the 'hole' in case it was changed by the last stacked item.*/
        let mut hole = borg_first_empty_inventory_slot(st);
        if hole == -1 {
            continue;
        }

        /* for wands and staffs adjust charges */
        if is_device(tval) {
            let w = ware(st, home, n);
            charge_each = i32::from(w.pval) / i32::from(w.iqty);
        }

        /* Save the number */
        let qty = min_quantity(st, home, n);

        /* Remove one item from shop (sometimes) */
        sub_iqty(&mut ware_mut(st, home, n).iqty, qty);

        /* for wands and staffs adjust charges */
        if is_device(tval) {
            let w = ware_mut(st, home, n);
            w.pval = (i32::from(w.pval) - qty * charge_each) as i16;
        }

        /* Obtain "slot" */
        let slot = borg_wield_slot(ware(st, home, n));
        let stack = borg_slot(st, i32::from(tval), i32::from(sval));

        /* Consider new equipment-- Must check both ring slots */
        let mut p: i32 = 0;
        if slot >= 0 {
            /* Require two empty slots */
            if hole == -1 || (hole + 1) >= pack_slots(st) {
                /* Restore shop item */
                restore_ware(st, home, n);
                continue;
            }

            /* Check Rings */
            if slot == inven_left() {
                /* First Check Left Hand */

                /* special curse check for left ring */
                if !st.items.items[inven_left() as usize].one_ring {
                    /* Move equipment into inventory */
                    item_from_safe(st, hole, slot);

                    /* Move new item into equipment */
                    item_from_safe_ware(st, slot, home, n);

                    /* Only a single item */
                    item_mut(st, slot).iqty = 1;

                    /* Fix later */
                    fix = true;

                    /* Examine the inventory */
                    crate::borg_trait::borg_notice(st, true);

                    /* Evaluate the inventory */
                    p_left = borg_power(st);

                    /* Restore old item */
                    item_from_safe(st, slot, slot);
                }

                /* Second Check Right Hand */
                /* special curse check for right ring */
                if !st.items.items[inven_right() as usize].one_ring {
                    /* Move equipment into inventory */
                    item_from_safe(st, hole, inven_right());

                    /* Move new item into equipment */
                    item_from_safe_ware(st, inven_right(), home, n);

                    /* Only a single item */
                    item_mut(st, inven_right()).iqty = qty as u8;

                    /* Fix later */
                    fix = true;

                    /* Examine the inventory */
                    crate::borg_trait::borg_notice(st, true);

                    /* Evaluate the inventory */
                    p_right = borg_power(st);

                    /* Restore old item */
                    item_from_safe(st, inven_right(), inven_right());
                }

                /* Is this ring better than one of mine? */
                p = if p_right < p_left { p_left } else { p_right };

                /* Restore hole */
                item_from_safe(st, hole, hole);
            } else {
                /* non rings */

                /* Move equipment into inventory */
                item_from_safe(st, hole, slot);

                /* Move new item into equipment */
                item_from_safe_ware(st, slot, home, n);

                /* Only a single item */
                item_mut(st, slot).iqty = qty as u8;

                /* Fix later */
                fix = true;

                /* Examine the inventory */
                crate::borg_trait::borg_notice(st, true);

                /* Evaluate the inventory */
                p = borg_power(st);

                /* Restore old item */
                item_from_safe(st, slot, slot);
            }
        }

        /* Consider new inventory.*/
        /* note, we may grab an equip able if, for example, we want to ID it */
        if p <= b_p {
            /* Restore hole if we are trying an item in inventory that didn't
             * work equipped */
            if slot >= 0 {
                item_mut(st, hole).iqty = 0;
            }

            if stack != -1 {
                hole = stack;
            }

            /* Require two empty slots */
            if stack == -1 && (hole == -1 || (hole + 1) >= pack_slots(st)) {
                /* Restore shop item */
                restore_ware(st, home, n);
                continue;
            }

            /* Move new item into inventory */
            item_from_safe_ware(st, hole, home, n);

            /* Is this new item merging into an existing stack? */
            if stack != -1 {
                /* Add a quantity to the stack */
                let safe_qty = i32::from(st.items.safe_items[hole as usize].iqty);
                item_mut(st, hole).iqty = (safe_qty + qty) as u8;

                /* for wands and staffs adjust charges */
                if is_device(ware(st, home, n).tval) {
                    let it = item_mut(st, hole);
                    it.pval = (i32::from(it.pval) + qty * charge_each) as i16;
                }
            } else {
                /* Only a single item */
                item_mut(st, hole).iqty = qty as u8;

                /* for wands and staffs adjust charges */
                if is_device(ware(st, home, n).tval) {
                    item_mut(st, hole).pval = (qty * charge_each) as i16;
                }
            }

            /* Fix later */
            fix = true;

            /* Examine the inventory */
            crate::borg_trait::borg_notice(st, true);

            /* Evaluate the equipment */
            p = borg_power(st);
        }

        /* Restore hole */
        item_from_safe(st, hole, hole);

        /* Restore shop item */
        restore_ware(st, home, n);

        /* Ignore "silly" purchases */
        if p <= b_p {
            continue;
        }

        /* Save the item and cost */
        b_n = n;
        b_p = p;
    }

    /* Examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* Buy something */
    if (b_n >= 0) && (b_p > st.borg.power) {
        /* Go to the home */
        st.borg.goal.shop = BORG_HOME as i16;

        /* Buy that item */
        st.borg.goal.ware = b_n as i16;

        /* Success */
        return true;
    }

    /* Nope */
    false
}

/// Step 5 -- buy "interesting" things from a shop (to be used later)
/// (`borg_think_shop_grab_interesting`).
pub fn borg_think_shop_grab_interesting(st: &mut BorgState) -> bool {
    let mut b_k: i32 = -1;
    let mut b_n: i32 = -1;
    // keeps its value from one ware to the next, as in C
    let mut charge_each: i32 = 0;

    let mut s: i32 = 0;
    let mut b_c: i32 = 0;

    /* Don't do this if Sauron is dead */
    if st.borg.tr(BI_SAURON_DEAD) != 0 {
        return false;
    }

    /* not until later-- use that money for better equipment */
    if st.borg.tr(BI_CLEVEL) < 15 {
        return false;
    }

    /* get what an empty home would have for power */
    borg_notice_home(st, None, true);
    let borg_empty_home_power = borg_power_home(st);

    let hole = borg_first_empty_inventory_slot(st);

    /* Require two empty slots */
    if hole == -1 {
        return false;
    }
    if hole + 1 >= pack_slots(st) {
        return false;
    }

    /* Examine the home */
    borg_notice_home(st, None, false);

    /* Evaluate the home */
    let mut b_s: i32 = borg_power_home(st);

    /* Check the shops (max -1 to skip home) */
    for k in 0..(store_max() - 1) {
        /* Scan the wares */
        for n in 0..store_inven_max() {
            /* Skip empty items */
            if ware(st, k, n).iqty == 0 {
                continue;
            }

            /* Skip "bad" buys */
            if !borg_good_buy(st, k, n) {
                continue;
            }

            /* Don't buy easy spell books late in the game */
            /* Require some "extra" cash */
            // `1000L + cost * 5`: the product is an int32_t, the sum a long
            if i64::from(st.borg.tr(BI_GOLD))
                < 1000 + i64::from(ware(st, k, n).cost.wrapping_mul(5))
            {
                continue;
            }

            /* for wands and staffs adjust charges */
            if is_device(ware(st, k, n).tval) {
                let w = ware(st, k, n);
                charge_each = i32::from(w.pval) / i32::from(w.iqty);
            }

            /* make this the next to last item that the player has */
            /* (can't make it the last or it thinks that both player and */
            /*  home are full) */
            st.items.items[hole as usize] = *ware(st, k, n);

            /* Save the number */
            let qty = min_quantity(st, k, n);

            /* Give a single item */
            item_mut(st, hole).iqty = qty as u8;

            /* for wands and staffs adjust charges */
            if is_device(ware(st, k, n).tval) {
                item_mut(st, hole).pval = (charge_each * qty) as i16;
            }

            /* make sure this item would help an empty home */
            let w = *ware(st, k, n);
            borg_notice_home(st, Some(&w), false);
            if borg_empty_home_power >= borg_power_home(st) {
                continue;
            }

            /* optimize the home inventory */
            if !crate::borg_store_sell::borg_think_home_sell_useful(st, &mut s) {
                continue;
            }

            /* Obtain the "cost" of the item */
            let c = ware(st, k, n).cost.wrapping_mul(qty);

            /* Penalize expensive items */
            if c > st.borg.tr(BI_GOLD) / 10 {
                s = s.wrapping_sub(c);
            }

            /* Ignore "bad" sales */
            if s < b_s {
                continue;
            }

            /* Ignore "expensive" purchases */
            if (s == b_s) && (c >= b_c) {
                continue;
            }

            /* Save the item and cost */
            b_k = k;
            b_n = n;
            b_s = s;
            b_c = c;
        }
    }

    /* restore inventory hole (just make sure the last slot goes back to */
    /* empty) */
    item_mut(st, hole).iqty = 0;

    /* Examine the real home */
    borg_notice_home(st, None, false);

    /* Evaluate the home */
    borg_power_home(st);

    /* remove the target that optimizing the home gave */
    st.borg.goal.item = -1;
    st.borg.goal.ware = -1;
    st.borg.goal.shop = -1;

    /* Buy something */
    if (b_k >= 0) && (b_n >= 0) {
        /* Visit that shop */
        st.borg.goal.shop = b_k as i16;

        /* Buy that item */
        st.borg.goal.ware = b_n as i16;

        /* Success */
        return true;
    }

    /* Nope */
    false
}

/// Step 6 -- take "useless" things from the home (to be sold)
/// (`borg_think_home_grab_useless`).
pub fn borg_think_home_grab_useless(st: &mut BorgState) -> bool {
    let home = BORG_HOME as i32;
    let mut b_n: i32 = -1;
    // this keeps its value from one ware to the next, as in C
    let mut charge_each: i32 = 0;
    let hole = borg_first_empty_inventory_slot(st);

    /* Require two empty slots */
    if hole == -1 {
        return false;
    }
    if hole + 1 >= pack_slots(st) {
        return false;
    }

    /* Examine the home */
    borg_notice_home(st, None, false);

    /* Evaluate the home */
    let mut b_s: i32 = borg_power_home(st);

    /* Scan the home */
    for n in 0..store_inven_max() {
        /* Skip empty items */
        if ware(st, home, n).iqty == 0 {
            continue;
        }

        /* skip stuff if we sold bought it */
        let mut skip_it = false;
        let mut p = 0;
        while p < st.store_sell.sold_item_num {
            let (s_tval, s_sval, s_store) = crate::borg_store_sell::sold_item(st, p as usize);
            let w = ware(st, home, n);
            if s_tval == i32::from(w.tval) && s_sval == i32::from(w.sval) && s_store == home {
                skip_it = true;
            }
            p += 1;
        }
        if skip_it {
            continue;
        }

        /* for wands and staffs adjust charges */
        if is_device(ware(st, home, n).tval) {
            let w = ware(st, home, n);
            charge_each = i32::from(w.pval) / i32::from(w.iqty);
        }

        /* Save the number */
        let qty = min_quantity(st, home, n);

        /* Remove one item from shop */
        sub_iqty(&mut ware_mut(st, home, n).iqty, qty);

        /* for wands and staffs adjust charges */
        if is_device(ware(st, home, n).tval) {
            let w = ware_mut(st, home, n);
            w.pval = (i32::from(w.pval) - charge_each * qty) as i16;
        }

        /* Examine the home */
        borg_notice_home(st, None, false);

        /* Evaluate the home */
        let s = borg_power_home(st);

        /* Restore shop item */
        restore_ware(st, home, n);

        /* Ignore "bad" sales */
        if s < b_s {
            continue;
        }

        /* Maintain the "best" */
        b_n = n;
        b_s = s;
    }

    /* Examine the home */
    borg_notice_home(st, None, false);

    /* Evaluate the home */
    borg_power_home(st);

    /* Stockpile */
    if b_n >= 0 {
        /* Visit the home */
        st.borg.goal.shop = BORG_HOME as i16;

        /* Grab that item */
        st.borg.goal.ware = b_n as i16;

        /* Success */
        return true;
    }

    /* Assume not */
    false
}

/// The four swap globals of borg-trait-swap.c:
/// `weapon_swap`, `weapon_swap_value`, `armour_swap`, `armour_swap_value`.
type Swaps = (i32, i32, i32, i32);

/// Read the four `Swaps` globals.
fn swaps(st: &BorgState) -> Swaps {
    let t = &st.trait_swap;
    (t.weapon_swap, t.weapon_swap_value, t.armour_swap, t.armour_swap_value)
}

/// Set the four `Swaps` globals (in C's order).
fn set_swaps(st: &mut BorgState, v: Swaps) {
    let t = &mut st.trait_swap;
    t.weapon_swap = v.0;
    t.weapon_swap_value = v.1;
    t.armour_swap = v.2;
    t.armour_swap_value = v.3;
}

/// Which swap `home_buy_swap` looks for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Swap {
    Weapon,
    Armour,
}

/// `weapon_swap_value` or `armour_swap_value`, read now.
fn swap_value(st: &mut BorgState, which: Swap) -> i32 {
    match which {
        Swap::Weapon => st.trait_swap.weapon_swap_value,
        Swap::Armour => st.trait_swap.armour_swap_value,
    }
}

/// Steps 7A and 7B: `borg_think_home_buy_swap_weapon` and
/// `borg_think_home_buy_swap_armour`, two C copies that differ only in
/// the swap they read and in the weapon test (same order of steps).
fn home_buy_swap(st: &mut BorgState, which: Swap) -> bool {
    let home = BORG_HOME as i32;
    let mut b_n: i32 = -1;
    // keeps its value from one ware to the next, as in C
    let mut p: i32 = 0;

    let mut fix = false;

    /* save the current values */
    let old: Swaps = swaps(st);
    let (swap, value) = match which {
        Swap::Weapon => (old.0, old.1),
        Swap::Armour => (old.2, old.3),
    };

    let hole = if swap <= 0 || value <= 0 {
        let h = borg_first_empty_inventory_slot(st);
        match which {
            Swap::Weapon => st.trait_swap.weapon_swap_value = -1,
            Swap::Armour => st.trait_swap.armour_swap_value = -1,
        }
        h
    } else {
        swap - 1
    };
    if hole == -1 {
        // C returns without restoring the value (docs/c-bugs.md)
        return false;
    }

    /* Extract the "power" */
    let mut b_p: i32 = swap_value(st, which);

    /* Scan the home */
    for n in 0..store_inven_max() {
        /* Skip empty items */
        if ware(st, home, n).iqty == 0 {
            continue;
        }

        if which == Swap::Weapon {
            /* Obtain "slot" make sure its a weapon */
            let slot = borg_wield_slot(ware(st, home, n));
            if slot != inven_wield() {
                continue;
            }
        }

        /* Remove one item from shop */
        let w = ware_mut(st, home, n);
        w.iqty = w.iqty.wrapping_sub(1);

        /* Consider new equipment (the weapon copy tests
         * `slot == INVEN_WIELD` again: always true here) */

        /* Move new item into inventory */
        item_from_safe_ware(st, hole, home, n);

        /* Only a single item */
        item_mut(st, hole).iqty = 1;

        /* Fix later */
        fix = true;

        /* Examine the inventory and swap value*/
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the new equipment */
        p = swap_value(st, which);

        /* Restore hole (the current swap, if the trial used its slot) */
        item_from_safe(st, hole, hole);

        /* Restore shop item */
        restore_ware(st, home, n);

        /* Ignore "silly" purchases */
        if p <= b_p {
            continue;
        }

        /* Save the item and value */
        b_n = n;
        b_p = p;
    }

    /* Examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* Buy something */
    if (b_n >= 0) && (b_p > swap_value(st, which)) {
        /* Go to the home */
        st.borg.goal.shop = BORG_HOME as i16;

        /* Buy that item */
        st.borg.goal.ware = b_n as i16;

        /* Restore the values */
        set_swaps(st, old);

        /* Success */
        return true;
    }

    /* Restore the values */
    set_swaps(st, old);

    /* Nope */
    false
}

/// Step 7A -- buy "useful" weapons from the home (to be used as a swap)
/// (`borg_think_home_buy_swap_weapon`).
pub fn borg_think_home_buy_swap_weapon(st: &mut BorgState) -> bool {
    home_buy_swap(st, Swap::Weapon)
}

/// Step 7B -- buy "useful" armour from the home (to be used as a swap)
/// (`borg_think_home_buy_swap_armour`).
pub fn borg_think_home_buy_swap_armour(st: &mut BorgState) -> bool {
    home_buy_swap(st, Swap::Armour)
}

/// `borg.goal.shop = borg.goal.ware = borg.goal.item = -1` and
/// `borg.antibounce_count++` (an `int16_t`).
fn purchase_done(st: &mut BorgState) {
    /* The purchase is complete */
    st.borg.goal.item = -1;
    st.borg.goal.ware = -1;
    st.borg.goal.shop = -1;

    /* Increment our anti-bounce count to avoid loops */
    st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(1);
}

/// Buy items from the current shop, if desired (`borg_think_shop_buy`).
pub fn borg_think_shop_buy(st: &mut BorgState) -> bool {
    /* Buy something if requested */
    if i32::from(st.borg.goal.shop) == i32::from(st.think.shop_num) && st.borg.goal.ware >= 0 {
        let shop = st.borg.goal.shop as i32;
        let w = st.borg.goal.ware as i32;
        // the ware does not change in here
        let item = *ware(st, shop, w);

        // a C `char` widened to `keycode_t`
        let purchase_target = SHOP_MENU_ITEMS[w as usize] as i8 as keycode_t;

        /* Paranoid */
        if item.tval == 0 {
            purchase_done(st);
            return false;
        }

        /* Log */
        note_desc(st, "# Buying ", &item, ".");

        /* Buy the desired item */
        borg_keypress(st, purchase_target);
        borg_keypress(st, keycode_t::from(b'p'));

        /* Mega-Hack -- Accept the purchase */
        borg_keypress(st, KC_ENTER);
        borg_keypress(st, KC_ENTER);

        /* if the borg is scumming and bought it.,
         * reset the scum amount.
         */
        let scum = st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize];
        if scum != 0 && item.cost >= scum.wrapping_mul(9) / 10 {
            st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize] = 0;

            /* Log */
            let msg = format!(
                "# Setting Money Scum to {}.",
                st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize]
            );
            borg_note(st, &msg);
        }

        /* Remember what we bought to avoid buy/sell loops */
        let b = &mut st.buy;
        if b.bought_item_nxt >= 9 {
            b.bought_item_nxt = 0;
        }
        let i = b.bought_item_nxt as usize;
        b.bought_item_pval[i] = i32::from(item.pval);
        b.bought_item_tval[i] = i32::from(item.tval);
        b.bought_item_sval[i] = i32::from(item.sval);
        b.bought_item_store[i] = i32::from(st.borg.goal.shop);
        b.bought_item_nxt += 1;
        if b.bought_item_num < b.bought_item_nxt {
            b.bought_item_num = b.bought_item_nxt;
        }

        purchase_done(st);

        /* leave the store */
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);

        /* I'm not in a store */
        st.borg.in_shop = false;

        /* Success */
        return true;
    }

    /* Nothing to buy */
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_state_starts_like_c() {
        let b = BuyState::default();
        assert_eq!(b.bought_item_num, -1);
        assert_eq!(b.bought_item_nxt, 0);
    }

    #[test]
    fn buy_goal_clears_a_pending_pack_sale() {
        let mut st = BorgState::default();
        st.borg.goal.item = 5;
        set_buy_goal(&mut st, 4, 2);
        assert_eq!(st.borg.goal.item, -1);
        assert_eq!(st.borg.goal.shop, 4);
        assert_eq!(st.borg.goal.ware, 2);
    }

    #[test]
    fn iqty_subtraction_wraps_like_uint8() {
        let mut q: u8 = 1;
        sub_iqty(&mut q, 5);
        assert_eq!(q, 252);
    }
}

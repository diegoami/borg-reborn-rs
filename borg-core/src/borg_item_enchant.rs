//! Port of `borg-item-enchant.c`: enchant weapons and armor.
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
//! `borg_enchant_to_h` and `borg_enchant_to_d` are copies of each other in
//! C (bonus, traits and scroll differ); here they are one function,
//! `enchant_weapon`, with the same tests in the same order.

use crate::borg_io::borg_keypress;
use crate::borg_item::{inven_wield, quiver_end, quiver_start};
use crate::borg_item_decurse::{borg_decurse_any, borg_decurse_armour, borg_decurse_weapon};
use crate::borg_item_use::{borg_activate_item, borg_read_scroll};
use crate::borg_magic::{borg_spell_fail, borg_spell_okay_fail};
use crate::engine;
use crate::borg_headers::borg_h::BORG_ENCHANT_LIMIT;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{
    borg_spells, BRAND_AMMUNITION, ENCHANT_ARMOUR, ENCHANT_WEAPON,
};
use crate::borg_headers::borg_trait_h::{
    BI_ABRAND, BI_AENCH_ARM, BI_AENCH_SARM, BI_AENCH_SWEP, BI_AENCH_TOD, BI_AENCH_TOH,
    BI_AMMO_POWER, BI_AMMO_TVAL, BI_CDEPTH, BI_ISBLIND, BI_ISCONFUSED, BI_NEED_BRAND_WEAPON,
    BI_NEED_ENCHANT_TO_A, BI_NEED_ENCHANT_TO_D, BI_NEED_ENCHANT_TO_H,
};
use crate::headers::list_tvals_h::{
    TV_BOOTS, TV_BOW, TV_CLOAK, TV_CROWN, TV_DIGGING, TV_DRAG_ARMOR, TV_GLOVES, TV_HAFTED,
    TV_HARD_ARMOR, TV_HELM, TV_POLEARM, TV_SHIELD, TV_SOFT_ARMOR, TV_SWORD,
};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `all_letters_nohjkl[i]` as a key (a C `char`, sign-extended to
/// `keycode_t`). `i` is never negative here.
fn letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// A key for a character literal.
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `INVEN_BOW`
fn inven_bow() -> i32 {
    inven_wield() + 1
}

/// `INVEN_BODY`
fn inven_body() -> i32 {
    inven_wield() + 6
}

/// `INVEN_TOTAL`
fn inven_total() -> i32 {
    inven_wield() + 12
}

/// `borg_timer(t)`: the turns since `t`.
fn borg_timer(st: &BorgState, t: i32) -> i32 {
    st.borg.time.now - t
}

/// Is there a pack item (`iqty` set) with one of these tvals? (The
/// `found` scans of `borg_pick_armor` / `borg_pick_weapon`.)
fn pack_has(st: &BorgState, tvals: &[u32]) -> bool {
    let pack = i32::from(engine::z_info().pack_size);
    (0..pack).any(|e| {
        let it = &st.items.items[e as usize];
        it.iqty != 0 && tvals.contains(&u32::from(it.tval))
    })
}

/// The "Skip boring items" test: with the spell (65% fail allowed) or a
/// "*enchant*" scroll (`star_trait`) an item is done at `limit`, else at 8.
/// `borg_spell_okay_fail` is tested first, as in C (it has no side
/// effects).
fn enchanted(st: &BorgState, spell: borg_spells, star_trait: u32, a: i32, limit: i32) -> bool {
    if borg_spell_okay_fail(st, spell, 65) || st.borg.tr(star_trait) >= 1 {
        a >= limit
    } else {
        a >= 8
    }
}

/// select the given armor
fn borg_pick_armor(st: &mut BorgState, i: i32) {
    /* Choose from equipment */
    if i >= inven_wield() {
        /* if there is armor in inventory, you have to press */
        /* '/' to get to equipment */
        /* (C tests TV_HELM twice) */
        let found = pack_has(
            st,
            &[
                TV_BOOTS,
                TV_GLOVES,
                TV_CLOAK,
                TV_CROWN,
                TV_HELM,
                TV_SHIELD,
                TV_HELM,
                TV_SOFT_ARMOR,
                TV_HARD_ARMOR,
                TV_DRAG_ARMOR,
            ],
        );
        if found {
            borg_keypress(st, key(b'/'));
        }

        /* Choose that item */
        borg_keypress(st, letter(i - inven_wield()));
    } else {
        /* Choose that item */
        borg_keypress(st, letter(i));
    }
}

/// select the given weapon
fn borg_pick_weapon(st: &mut BorgState, i: i32) {
    /* Choose from equipment */
    if i < inven_wield() {
        borg_keypress(st, letter(i));
    } else {
        /* if there is a weapon in inventory, you have to press */
        /* '/' to get to equipment or '|' to go to quiver */
        let found = pack_has(st, &[TV_BOW, TV_DIGGING, TV_HAFTED, TV_POLEARM, TV_SWORD]);

        if i < quiver_start() {
            if found {
                borg_keypress(st, key(b'/'));
            }

            borg_keypress(st, letter(i - inven_wield()));
        } else {
            /* Quiver Slot */
            if found
                || st.items.items[inven_wield() as usize].iqty != 0
                || st.items.items[inven_bow() as usize].iqty != 0
            {
                borg_keypress(st, key(b'|'));
            }
            borg_keypress(st, (i32::from(b'0') + (i - quiver_start())) as keycode_t);
        }
    }
}

/// Enchant armor, not including my swap armour
fn borg_enchant_to_a(st: &mut BorgState) -> bool {
    let mut b_i = -1;
    let mut b_a = 99;

    /* Nothing to enchant */
    if st.borg.tr(BI_NEED_ENCHANT_TO_A) == 0 {
        return false;
    }

    /* Need "enchantment" ability */
    if st.borg.tr(BI_AENCH_ARM) == 0 && st.borg.tr(BI_AENCH_SARM) == 0 {
        return false;
    }

    let limit = st.cfg(BORG_ENCHANT_LIMIT);

    /* Look for armor that needs enchanting */
    for i in inven_body()..inven_total() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip non-identified items */
        if !item.ident {
            continue;
        }

        /* Obtain the bonus */
        let a = i32::from(item.to_a);

        /* Skip "boring" items */
        if enchanted(st, ENCHANT_ARMOUR, BI_AENCH_SARM, a, limit) {
            continue;
        }

        /* Find the least enchanted item */
        if b_i >= 0 && b_a < a {
            continue;
        }

        /* Save the info */
        b_i = i;
        b_a = a;
    }

    /* Nothing */
    if b_i < 0 {
        return false;
    }

    /* Enchant it */
    let sv_star = st.item_val.sv_scroll_star_enchant_armor;
    let sv_plain = st.item_val.sv_scroll_enchant_armor;
    if borg_read_scroll(st, sv_star)
        || borg_read_scroll(st, sv_plain)
        || borg_spell_fail(st, ENCHANT_ARMOUR, 65)
    {
        borg_pick_armor(st, b_i);

        /* Success */
        return true;
    }

    /* Nothing to do */
    false
}

/// Which weapon bonus `enchant_weapon` works on.
#[derive(Clone, Copy)]
enum Bonus {
    /// `borg_enchant_to_h`
    ToHit,
    /// `borg_enchant_to_d`
    ToDam,
}

impl Bonus {
    fn of(self, item: &borg_item) -> i32 {
        match self {
            Bonus::ToHit => i32::from(item.to_h),
            Bonus::ToDam => i32::from(item.to_d),
        }
    }
}

/// `borg_enchant_to_h` / `borg_enchant_to_d`: enchant weapons to hit or
/// to dam (the swap weapon and, failing that, the ammo too).
fn enchant_weapon(st: &mut BorgState, bonus: Bonus) -> bool {
    let mut b_i = -1;
    let mut b_a = 99;

    let (need, swap_needs, ability, sv_scroll) = match bonus {
        Bonus::ToHit => (
            BI_NEED_ENCHANT_TO_H,
            st.trait_swap.enchant_weapon_swap_to_h,
            BI_AENCH_TOH,
            st.item_val.sv_scroll_enchant_weapon_to_hit,
        ),
        Bonus::ToDam => (
            BI_NEED_ENCHANT_TO_D,
            st.trait_swap.enchant_weapon_swap_to_d,
            BI_AENCH_TOD,
            st.item_val.sv_scroll_enchant_weapon_to_dam,
        ),
    };

    /* Nothing to enchant */
    if st.borg.tr(need) == 0 && swap_needs == 0 {
        return false;
    }

    /* Need "enchantment" ability */
    if st.borg.tr(ability) == 0 && st.borg.tr(BI_AENCH_SWEP) == 0 {
        return false;
    }

    let limit = st.cfg(BORG_ENCHANT_LIMIT);

    /* Look for a weapon that needs enchanting */
    for i in inven_wield()..=inven_bow() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip non-identified items */
        if !item.ident {
            continue;
        }

        /* Skip my swap digger */
        if u32::from(item.tval) == TV_DIGGING {
            continue;
        }

        /* Obtain the bonus */
        let a = bonus.of(item);

        /* Skip "boring" items */
        if enchanted(st, ENCHANT_WEAPON, BI_AENCH_SWEP, a, limit) {
            continue;
        }

        /* Most classes store the enchants until they get a 3x shooter
         * (like a long bow); skip a plain x2 shooter (see the C comment). */
        if i == inven_bow() /* bow */
            && st.borg.tr(BI_AMMO_POWER) < 3 /* 3x shooter */
            && (item.art_idx == 0 && item.ego_idx == 0)
        /* Not artifact or ego */
        {
            continue;
        }

        /* Find the least enchanted item */
        if b_i >= 0 && b_a < a {
            continue;
        }

        /* Save the info */
        b_i = i;
        b_a = a;
    }
    let weapon_swap = st.trait_swap.weapon_swap;
    if weapon_swap != 0 {
        let mut skip = false;
        let item = &st.items.items[(weapon_swap - 1) as usize];

        /* Skip my swap digger and anything unid'd */
        if item.ident && u32::from(item.tval) != TV_DIGGING {
            /* Obtain the bonus */
            let s_a = bonus.of(item);

            /* Skip items that are already enchanted */
            if enchanted(st, ENCHANT_WEAPON, BI_AENCH_SWEP, s_a, limit) {
                skip = true;
            }

            /* Find the least enchanted item */
            if b_a > s_a && !skip {
                /* Save the info */
                b_i = weapon_swap - 1;
                b_a = s_a;
            }
        }
    }
    /* Nothing, check ammo */
    if b_i < 0 {
        /* look through inventory for ammo */
        for i in quiver_start()..quiver_end() {
            /* Only enchant ammo if we have a good shooter,
             * otherwise, store the enchants in the home.
             * (borg_enchant_to_d tests BI_AMMO_POWER < 3 a second time:
             * no effect.) */
            let bow = &st.items.items[inven_bow() as usize];
            if st.borg.tr(BI_AMMO_POWER) < 3 || (bow.art_idx == 0 && bow.ego_idx == 0) {
                continue;
            }

            let item = &st.items.items[i as usize];

            /* Only enchant if qty >= 5 */
            if item.iqty < 5 {
                continue;
            }

            /* Skip non-identified items  */
            if !item.ident {
                continue;
            }

            /* Make sure it is the right type if missile */
            if i32::from(item.tval) != st.borg.tr(BI_AMMO_TVAL) {
                continue;
            }

            /* Obtain the bonus  */
            let a = bonus.of(item);

            /* Skip items that are already enchanted */
            if enchanted(st, ENCHANT_WEAPON, BI_AENCH_SWEP, a, 10) {
                continue;
            }

            /* Find the least enchanted item */
            if b_i >= 0 && b_a < a {
                continue;
            }

            /* Save the info  */
            b_i = i;
            b_a = a;
        }
    }

    /* Nothing */
    if b_i < 0 {
        return false;
    }

    /* Enchant it */
    let sv_star = st.item_val.sv_scroll_star_enchant_weapon;
    if borg_read_scroll(st, sv_star)
        || borg_read_scroll(st, sv_scroll)
        || borg_spell_fail(st, ENCHANT_WEAPON, 65)
    {
        borg_pick_weapon(st, b_i);

        /* Success */
        return true;
    }

    /* Nothing to do */
    false
}

/// Brand Bolts
fn borg_brand_weapon(st: &mut BorgState) -> bool {
    let mut b_i = -1;
    let mut b_a = 0;

    /* Nothing to brand */
    if st.borg.tr(BI_NEED_BRAND_WEAPON) == 0 {
        return false;
    }

    /* Need "brand" ability */
    if st.borg.tr(BI_ABRAND) == 0 {
        return false;
    }

    /* look through inventory for ammo */
    for i in quiver_start()..quiver_end() {
        let item = &st.items.items[i as usize];

        /* Only enchant if qty >= 20 */
        if item.iqty < 20 {
            continue;
        }

        /* Skip non-identified items  */
        if !item.ident {
            continue;
        }

        /* Make sure it is the right type if missile */
        if i32::from(item.tval) != st.borg.tr(BI_AMMO_TVAL) {
            continue;
        }

        /* Obtain the bonus  */
        let a = i32::from(item.to_h);

        /* Skip branded items */
        if item.ego_idx != 0 {
            continue;
        }

        /* Find the most enchanted item */
        if b_i >= 0 && b_a > a {
            continue;
        }

        /* Save the info  */
        b_i = i;
        b_a = a;
    }

    /* Nothing to Brand */
    if b_i == -1 {
        return false;
    }

    /* Enchant it */
    let act = st.act.firebrand;
    if borg_activate_item(st, act) || borg_spell_fail(st, BRAND_AMMUNITION, 65) {
        borg_pick_weapon(st, b_i);

        /* Success */
        return true;
    }

    /* Nothing to do */
    false
}

/// Enchant things
pub fn borg_enchanting(st: &mut BorgState) -> bool {
    /* Forbid blind/confused */
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and finish the job later */
    let level = st.borg.time.level;
    if (borg_timer(st, level) > 5500 && st.borg.tr(BI_CDEPTH) >= 1)
        || (borg_timer(st, level) > 3501 && st.borg.tr(BI_CDEPTH) == 0)
    {
        return false;
    }

    /* Remove Curses */
    if borg_decurse_armour(st) {
        return true;
    }
    if borg_decurse_weapon(st) {
        return true;
    }
    if borg_decurse_any(st) {
        return true;
    }

    /* Only in town */
    if st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }

    /* Enchant things */
    if borg_brand_weapon(st) {
        return true;
    }
    if enchant_weapon(st, Bonus::ToHit) {
        return true;
    }
    if enchant_weapon(st, Bonus::ToDam) {
        return true;
    }
    if borg_enchant_to_a(st) {
        return true;
    }

    /* Nope */
    false
}

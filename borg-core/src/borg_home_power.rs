//! Port of `borg-home-power.c`: determine the power value of the home.
//! This is used to decide what items are best to keep at home.
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
//! Integer semantics: `value` is an `int32_t`; every C `value += x * 5L`
//! is done in `long` (i64) and truncated back to i32 (`add`). The
//! counters are the `int16_t` `st.home` fields (borg_home_notice), promoted
//! to `int`.

use crate::borg_headers::borg_trait_h::{
    BI_AID, BI_CLASS, BI_CLEVEL, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXSP,
};
use crate::headers::list_stats_h::{STAT_CON, STAT_DEX, STAT_STR};
use crate::headers::list_tvals_h::{TV_FOOD, TV_POTION, TV_SCROLL, TV_STAFF};
use crate::state::BorgState;
use crate::engine;

/// `CLASS_WARRIOR` (borg-trait.h).
const CLASS_WARRIOR: i32 = 0;

/// `value += x` with `x` a C `long` expression: computed in i64, then
/// truncated to `int32_t`.
fn add(value: i32, x: i64) -> i32 {
    (i64::from(value) + x) as i32
}

/// `value -= x` with `x` a C `long` expression.
fn sub(value: i32, x: i64) -> i32 {
    (i64::from(value) - x) as i32
}

/// The "spare item" tiers used for most flags in `borg_power_home_aux1`:
/// ```c
/// if (n == 1) value += one;
/// else if (n == 2) value += two;
/// else if (n > 2) value += two + (n - 2) * each;
/// ```
fn spare(value: i32, n: i32, one: i64, two: i64, each: i64) -> i32 {
    if n == 1 {
        add(value, one)
    } else if n == 2 {
        add(value, two)
    } else if n > 2 {
        add(value, two + i64::from(n - 2) * each)
    } else {
        value
    }
}

/// `if (n > limit) value -= (n - limit) * each;`
fn too_many(value: i32, n: i32, limit: i32, each: i64) -> i32 {
    if n > limit {
        sub(value, i64::from(n - limit) * each)
    } else {
        value
    }
}

/// The stat-gain tiers:
/// ```c
/// if (s < lo) value += s * a;
/// else if (s < hi) value += base2 + (s - lo) * b;
/// else value += base3 + (s - hi) * c;
/// ```
#[allow(clippy::too_many_arguments)]
fn stat_tiers(
    value: i32,
    s: i32,
    lo: i32,
    hi: i32,
    a: i64,
    base2: i64,
    b: i64,
    base3: i64,
    c: i64,
) -> i32 {
    if s < lo {
        add(value, i64::from(s) * a)
    } else if s < hi {
        add(value, base2 + i64::from(s - lo) * b)
    } else {
        add(value, base3 + i64::from(s - hi) * c)
    }
}

/// `kb_info[tval].max_stack`
fn max_stack(tval: u32) -> i32 {
    engine::kb_info()[tval as usize].max_stack
}

/// Helper function -- calculate power of equipment in the home
fn borg_power_home_aux1(st: &mut BorgState) -> i32 {
    let mut value: i32 = 0;

    // This would be better separated by item type (so 1 bonus for resist
    // cold armor, 1 bonus for resist cold shield... but that would take a
    // bunch more code.

    // try to collect at least 2 of each resist/power (for swapping)
    // This can be used to get rid of extra artifacts...

    // spare lite sources.  Artifacts only
    value = spare(value, i32::from(st.home.num_LIGHT), 150, 170, 5);
    value = spare(value, i32::from(st.home.num_slow_digest), 50, 70, 5);
    value = spare(value, i32::from(st.home.num_regenerate), 75, 100, 10);
    value = spare(value, i32::from(st.home.num_telepathy), 1000, 1500, 10);
    value = spare(value, i32::from(st.home.num_see_inv), 800, 1200, 10);
    value = spare(value, i32::from(st.home.num_ffall), 10, 15, 1);
    value = spare(value, i32::from(st.home.num_free_act), 1000, 1500, 10);
    value = spare(value, i32::from(st.home.num_hold_life), 1000, 1500, 10);

    value = spare(value, i32::from(st.home.num_resist_acid), 1000, 1500, 1);
    value = spare(value, i32::from(st.home.num_immune_acid), 3000, 5000, 30);
    value = spare(value, i32::from(st.home.num_resist_elec), 1000, 1500, 1);
    value = spare(value, i32::from(st.home.num_immune_elec), 3000, 5000, 30);
    value = spare(value, i32::from(st.home.num_resist_fire), 1000, 1500, 1);
    value = spare(value, i32::from(st.home.num_immune_fire), 3000, 5000, 30);
    value = spare(value, i32::from(st.home.num_resist_cold), 1000, 1500, 1);
    value = spare(value, i32::from(st.home.num_immune_cold), 3000, 5000, 30);

    value = spare(value, i32::from(st.home.num_resist_pois), 5000, 9000, 40);
    value = spare(value, i32::from(st.home.num_resist_conf), 2000, 8000, 45);
    value = spare(value, i32::from(st.home.num_resist_sound), 500, 700, 30);
    value = spare(value, i32::from(st.home.num_resist_LIGHT), 100, 150, 1);
    value = spare(value, i32::from(st.home.num_resist_dark), 100, 150, 1);
    value = spare(value, i32::from(st.home.num_resist_chaos), 1000, 1500, 10);
    value = spare(value, i32::from(st.home.num_resist_disen), 5000, 7000, 35);
    value = spare(value, i32::from(st.home.num_resist_shard), 100, 150, 1);
    value = spare(value, i32::from(st.home.num_resist_nexus), 200, 300, 2);
    value = spare(value, i32::from(st.home.num_resist_blind), 500, 1000, 5);
    value = spare(value, i32::from(st.home.num_resist_neth), 3000, 4000, 45);

    // stat gain items as well...(good to carry ring of dex +6 in
    //                            house even if I don't need it right now)
    let s = i32::from(st.home.home_stat_add[STAT_STR as usize]);
    value = stat_tiers(value, s, 9, 15, 300, 9 * 300, 200, 9 * 300 + 6 * 200, 1);
    let s = i32::from(st.home.home_stat_add[STAT_DEX as usize]);
    value = stat_tiers(value, s, 9, 15, 300, 9 * 300, 200, 9 * 300 + 6 * 200, 1);

    // HACK extra con for thorin and other such things
    let s = i32::from(st.home.home_stat_add[STAT_CON as usize]);
    value = stat_tiers(value, s, 15, 21, 300, 15 * 300, 200, 15 * 300 + 6 * 200, 1);

    // spell stat is only bonused for spell casters.
    let spell_stat = crate::borg_magic::borg_spell_stat();
    if spell_stat >= 0 {
        let s = i32::from(st.home.home_stat_add[spell_stat as usize]);
        // C bug 8: the last tier starts from 20 * 100, not 20 * 400.
        value = stat_tiers(value, s, 20, 26, 400, 20 * 400, 300, 20 * 100 + 6 * 300, 5);
    }

    // Sustains
    value = spare(value, i32::from(st.home.num_sustain_str), 200, 250, 1);
    value = spare(value, i32::from(st.home.num_sustain_int), 200, 250, 1);
    value = spare(value, i32::from(st.home.num_sustain_wis), 200, 250, 1);
    value = spare(value, i32::from(st.home.num_sustain_con), 200, 250, 1);
    value = spare(value, i32::from(st.home.num_sustain_dex), 200, 250, 1);
    value = spare(value, i32::from(st.home.num_sustain_all), 1000, 1500, 1);

    // do a minus for too many duplicates.  This way we do not store
    // useless items and spread out types of items.
    let n = i32::from(st.home.num_weapons);
    if n > 5 {
        value = too_many(value, n, 5, 2000);
    } else {
        value = too_many(value, n, 1, 100);
    }
    value = too_many(value, i32::from(st.home.num_bow), 2, 1000);
    let n = i32::from(st.home.num_rings);
    if n > 6 {
        value = too_many(value, n, 6, 4000);
    } else {
        value = too_many(value, n, 4, 2000);
    }
    value = too_many(value, i32::from(st.home.num_neck), 3, 1500);
    value = too_many(value, i32::from(st.home.num_armor), 6, 1000);
    value = too_many(value, i32::from(st.home.num_cloaks), 3, 1000);
    value = too_many(value, i32::from(st.home.num_shields), 3, 1000);
    value = too_many(value, i32::from(st.home.num_hats), 4, 1000);
    value = too_many(value, i32::from(st.home.num_gloves), 3, 1000);
    value = too_many(value, i32::from(st.home.num_boots), 3, 1000);

    // `int` addition (no `long` here)
    value = value.wrapping_add(i32::from(st.home.home_damage));

    // if edged and priest, small penalty
    value = sub(value, i64::from(i32::from(st.home.num_edged_weapon)) * 50);

    // if gloves and mage or ranger and not FA/Dex, dump it.
    value = sub(value, i64::from(i32::from(st.home.num_bad_gloves)) * 3000);

    // do not allow duplication of items.
    value = sub(
        value,
        i64::from(i32::from(st.home.num_duplicate_items)) * 50000,
    );

    value
}

/// `for (k = 0; k < lim && k < n; k++) value += base - k * step;`
fn stack(value: i32, lim: i32, n: i32, base: i64, step: i64) -> i32 {
    let mut value = value;
    let mut k = 0;
    while k < lim && k < n {
        value = add(value, base - i64::from(k) * step);
        k += 1;
    }
    value
}

/// Helper function -- calculate power of items in the home
///
/// The weird calculations help spread out the purchase order
fn borg_power_home_aux2(st: &mut BorgState) -> i32 {
    let mut value: i32 = 0;

    // *** Basic abilities ***

    // Collect food
    if st.borg.tr(BI_MAXCLEVEL) < 10 {
        value = stack(
            value,
            max_stack(TV_FOOD),
            i32::from(st.home.num_food),
            8000,
            10,
        );
    }

    // Collect ident
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_ident),
        2000,
        10,
    );

    // Collect enchantments armour
    if st.borg.tr(BI_CLEVEL) < 45 {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_a),
            500,
            10,
        );
    }
    // Collect enchantments to hit
    if st.borg.tr(BI_CLEVEL) < 45 {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_h),
            500,
            10,
        );
    }
    // Collect enchantments to dam
    if st.borg.tr(BI_CLEVEL) < 45 {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_enchant_to_d),
            500,
            10,
        );
    }

    // Collect pfe
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_pfe),
        500,
        10,
    );

    // Collect glyphs
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_glyph),
        500,
        10,
    );

    // Reward Genocide scrolls. Just scrolls, mainly used for Morgoth
    value = stack(
        value,
        max_stack(TV_SCROLL) * 2,
        i32::from(st.home.num_genocide),
        500,
        10,
    );

    // Reward Mass Genocide scrolls. Just scrolls, mainly used for Morgoth
    value = stack(
        value,
        max_stack(TV_SCROLL) * 2,
        i32::from(st.home.num_mass_genocide),
        500,
        0,
    );

    // Collect Recharge ability
    value = stack(
        value,
        max_stack(TV_SCROLL),
        i32::from(st.home.num_recharge),
        500,
        10,
    );

    // Reward Resistance Potions for Warriors
    if st.borg.tr(BI_CLASS) == CLASS_WARRIOR
        && st.borg.tr(BI_MAXDEPTH) > 20
        && st.borg.tr(BI_MAXDEPTH) < 80
    {
        // `k` carries over from the first loop into the second.
        let mut k = 0;
        while k < max_stack(TV_POTION) && k < i32::from(st.home.num_pot_rheat) {
            value = add(value, 100 - i64::from(k) * 10);
            k += 1;
        }
        while k < max_stack(TV_POTION) && k < i32::from(st.home.num_pot_rcold) {
            value = add(value, 100 - i64::from(k) * 10);
            k += 1;
        }
    }

    // Collect recall - stick to 5 spare, for if you run out of money
    value = stack(value, 5, i32::from(st.home.num_recall), 100, 0);

    // Collect escape  (staff of teleport)
    value = stack(value, 85, i32::from(st.home.num_escape), 2000, 10);

    // Collect only one stack worth of teleport staffs at home
    let mut k = max_stack(TV_STAFF);
    while k < i32::from(st.home.num_tele_staves) {
        value = sub(value, 50000);
        k += 1;
    }

    // Collect teleport
    value = stack(value, 85, i32::from(st.home.num_teleport), 5000, 0);

    // Collect phase
    if st.borg.tr(BI_MAXCLEVEL) < 10 {
        value = stack(
            value,
            max_stack(TV_SCROLL),
            i32::from(st.home.num_phase),
            5000,
            0,
        );
    }

    // Collect Speed
    // for (k = 0; k < 85 && k < num_speed; k++) value += 5000L - k*10L;

    // collect mana/
    if st.borg.tr(BI_MAXSP) > 1 {
        value = stack(
            value,
            max_stack(TV_POTION),
            i32::from(st.home.num_mana),
            6000,
            8,
        );
    }

    // Level 1 priests are given a Potion of Healing.  It is better
    // for them to sell that potion and buy equipment or several
    // Cure Crits with it.
    if st.borg.tr(BI_CLEVEL) == 1 {
        value = stack(value, 10, i32::from(st.home.num_heal), -5000, 0);
    }

    // *** Healing ***

    // Collect cure critical
    value = stack(
        value,
        max_stack(TV_POTION),
        i32::from(st.home.num_cure_critical),
        1500,
        10,
    );

    // Collect heal, *Heal*, Life
    value = stack(value, 90, i32::from(st.home.num_heal), 3000, 0);
    value = stack(value, 198, i32::from(st.home.num_ezheal), 8000, 0);
    value = stack(value, 198, i32::from(st.home.num_life), 9000, 0);

    // junk cure serious if we have some in the home
    // don't bother keeping them if high level
    if st.borg.tr(BI_CLEVEL) > 35 {
        // value -= 1500L - k * 10L
        value = stack(value, 90, i32::from(st.home.num_cure_serious), -1500, -10);
    }

    // *** Various ***

    // Fixing Stats
    if st.borg.tr(BI_CLEVEL) == 50 && i32::from(st.home.num_fix_exp) != 0 {
        value = sub(value, 7500);
    }
    if st.borg.tr(BI_CLEVEL) > 35 && st.borg.tr(BI_CLEVEL) <= 49 {
        value = stack(value, 70, i32::from(st.home.num_fix_exp), 1000, 10);
    } else if st.borg.tr(BI_CLEVEL) <= 35 {
        value = stack(value, 5, i32::from(st.home.num_fix_exp), 1000, 10);
    }

    // *** books ***

    // Reward books
    for book in 0..9 {
        // only collect books up to level 14.
        // After that, just buy them, they are always in stock
        if st.borg.tr(BI_CLEVEL) < 15 {
            // Collect up to 5 copies of each normal book
            let mut k = 0;
            while k < 5 && k < i32::from(st.home.num_book[book]) {
                // Only stockpile useful books
                if i32::from(st.home.num_book[book]) != 0 {
                    value = add(value, 5000 - i64::from(k) * 10);
                }
                k += 1;
            }
        }
    }

    // Reward artifacts in the home
    value = add(value, i64::from(i32::from(st.home.num_artifact)) * 500);

    // Reward certain types of egos in the home
    value = add(value, i64::from(i32::from(st.home.num_ego)) * 5000);

    // Only allow unid'd stuff if we can't id them
    let un_id = i32::from(st.home.home_un_id);
    if un_id != 0 {
        value = add(value, i64::from(un_id - st.borg.tr(BI_AID)) * 1005);
    }

    value
}

/// Calculate the "power" of the home
pub fn borg_power_home(st: &mut BorgState) -> i32 {
    let mut value: i32 = 0;

    // Process the home equipment
    value = value.wrapping_add(borg_power_home_aux1(st));

    // Process the home inventory
    value = value.wrapping_add(borg_power_home_aux2(st));

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spare_tiers() {
        assert_eq!(spare(0, 0, 150, 170, 5), 0);
        assert_eq!(spare(0, -1, 150, 170, 5), 0);
        assert_eq!(spare(0, 1, 150, 170, 5), 150);
        assert_eq!(spare(0, 2, 150, 170, 5), 170);
        assert_eq!(spare(0, 4, 150, 170, 5), 180);
    }

    #[test]
    fn spell_stat_tiers_keep_c_bug() {
        let f = |s| stat_tiers(0, s, 20, 26, 400, 20 * 400, 300, 20 * 100 + 6 * 300, 5);
        assert_eq!(f(19), 7600);
        assert_eq!(f(25), 8000 + 1500);
        // C bug 8: drops at 26.
        assert_eq!(f(26), 3800);
        assert_eq!(f(30), 3820);
    }

    #[test]
    fn stack_loops() {
        // value -= 1500L - k * 10L, three items
        assert_eq!(stack(0, 90, 3, -1500, -10), -(1500 + 1490 + 1480));
        assert_eq!(stack(0, 2, 5, 500, 10), 990);
        assert_eq!(stack(7, 5, 0, 500, 10), 7);
    }

    #[test]
    fn long_arithmetic_truncates() {
        assert_eq!(add(i32::MAX, 1), i32::MIN);
        assert_eq!(sub(i32::MIN, 1), i32::MAX);
    }
}

//! Port of `borg-power.c`: the calculations to determine how powerful the
//! Borg thinks it is given the value arrays (`borg.trait`/`has`/`activation`).
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
//! Integer semantics: `value` is an `int32_t`. A C `value += x` with `x` a
//! `long` expression is done in i64 and truncated back (`add`/`sub`); with
//! `x` an `int` expression it is a wrapping i32 add (`addi`/`subi`).
//! Products are formed in the width C forms them (`int` products wrap,
//! `... * 100L` products are i64), and divisions are done after them in
//! that same width. The callers compare the results for ties (what-if
//! loops in wear/junk/store code), so the order of additions is kept too.

use crate::borg_formulas::borg_power_dynamic;
use crate::borg_item::borg_item_weight;
use crate::borg_item_activation::ActivationIds;
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_magic::{borg_get_spell_entry, borg_spell_stat};
use crate::borg_headers::borg_h::{
    BORG_MUNCHKIN_LEVEL, BORG_MUNCHKIN_START, BORG_USES_DYNAMIC_CALCS, BORG_WORSHIPS_AC,
    BORG_WORSHIPS_DAMAGE, BORG_WORSHIPS_HP, BORG_WORSHIPS_MANA, BORG_WORSHIPS_SPEED,
};
use crate::borg_headers::borg_trait_h::*;
use crate::headers::list_player_flags_h::PF_ZERO_FAIL;
use crate::headers::list_stats_h::{STAT_CON, STAT_DEX, STAT_INT, STAT_STR, STAT_WIS};
use crate::headers::list_tvals_h::{TV_ARROW, TV_DIGGING, TV_POTION, TV_SCROLL};
use crate::headers::player_h::STAT_MAX;
use crate::state::BorgState;
use crate::engine;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;
const CLASS_BLACKGUARD: i32 = 8;

/// `INVEN_WIELD + off` (`z_info->pack_size + off`): an equipment slot of
/// `borg_items`.
fn inven(off: usize) -> usize {
    usize::from(engine::z_info().pack_size) + off
}

const WIELD: usize = 0;
const BOW: usize = 1;
const BODY: usize = 6;
const OUTER: usize = 7;
const ARM: usize = 8;
const HEAD: usize = 9;
const HANDS: usize = 10;
const FEET: usize = 11;

/// `value += x` with `x` a C `long` expression.
fn add(value: i32, x: i64) -> i32 {
    (i64::from(value) + x) as i32
}

/// `value -= x` with `x` a C `long` expression.
fn sub(value: i32, x: i64) -> i32 {
    (i64::from(value) - x) as i32
}

/// `value += x` with `x` a C `int` expression.
fn addi(value: i32, x: i32) -> i32 {
    value.wrapping_add(x)
}

/// `value -= x` with `x` a C `int` expression.
fn subi(value: i32, x: i32) -> i32 {
    value.wrapping_sub(x)
}

/// `for (; k < lim && k < n; k++) value += each;` (`each` a C `long`
/// constant). `k` carries over, as C reuses it between loops.
fn count_up(value: &mut i32, k: &mut i32, lim: i32, n: i32, each: i64) {
    while *k < lim && *k < n {
        *value = add(*value, each);
        *k += 1;
    }
}

/// `for (; k < lim && k < n; k++) value -= each;`
fn count_down(value: &mut i32, k: &mut i32, lim: i32, n: i32, each: i64) {
    while *k < lim && *k < n {
        *value = sub(*value, each);
        *k += 1;
    }
}

/// The bonus of one activation, the `else if` chain of
/// `borg_power_equipment()` in C order (the first matching index wins; an
/// index no entry names gets 0).
fn activation_bonus(a: &ActivationIds, class: i32, clevel: i32, act: i32) -> i32 {
    let tele_other = if class == CLASS_MAGE { 500 } else { 500 + 500 };
    let rem_fear_pois = if class == CLASS_MAGE || class == CLASS_PRIEST || class == CLASS_DRUID {
        500
    } else {
        500 + 200
    };
    let resist_all = (500 + 150) + if class != CLASS_MAGE { 25000 } else { 0 };
    let sleepii = 500
        + if class != CLASS_DRUID && class != CLASS_NECROMANCER {
            200
        } else {
            0
        };
    let recharge = 500
        + if class != CLASS_MAGE && class != CLASS_ROGUE {
            100
        } else {
            0
        };
    let rage_bless_resist = (500 + 150) + if class != CLASS_MAGE { 25000 } else { 0 };
    let dispel_evil = 500 + (10 + clevel.wrapping_mul(5) / 2);

    let chain: [(i32, i32); 160] = [
        (a.illumination, 500),
        (a.mapping, 550),
        (a.clairvoyance, 600),
        (a.fire_bolt, 500 + (9 * (8 + 1) / 2)),
        (a.cold_bolt, 500 + (6 * (8 + 1) / 2)),
        (a.elec_bolt, 500 + (4 * (8 + 1) / 2)),
        (a.acid_bolt, 500 + (5 * (8 + 1) / 2)),
        (a.mana_bolt, 500 + (12 * (8 + 1) / 2)),
        (a.stinking_cloud, 500 + 24),
        (a.cold_ball50, 500 + 96),
        (a.cold_ball100, 500 + 200),
        (a.fire_ball72, 500 + 72),
        (a.cold_bolt2, 500 + (12 * (8 + 1) / 2)),
        (a.fire_ball, 500 + 144),
        (a.dispel_evil, dispel_evil),
        (a.confuse2, 0),
        (a.haste, 0),
        (a.haste1, 0),
        (a.haste2, 0),
        (a.detect_objects, 10),
        (a.probing, 0),
        (a.stone_to_mud, 0),
        (a.tele_other, tele_other),
        (a.drain_life1, 500 + 90),
        (a.drain_life2, 500 + 120),
        (a.berserker, 500),
        (a.cure_light, 0),
        (a.cure_serious, 0),
        (a.cure_critical, 0),
        (a.cure_full2, 0),
        (a.loskill, 500 + 200),
        (a.recall, 0),
        (a.arrow, 500 + 150),
        (a.rem_fear_pois, rem_fear_pois),
        (a.tele_phase, 500),
        (a.detect_all, 0),
        (a.cure_full, 0),
        (a.heal1, 0),
        (a.heal2, 0),
        (a.heal3, 0),
        (a.cure_nonorlybig, 0),
        (a.protevil, 0),
        (a.destroy_doors, 50),
        (a.banishment, 1000),
        (a.resist_all, resist_all),
        (a.sleepii, sleepii),
        (a.recharge, recharge),
        (a.tele_long, 300),
        (a.missile, 500 + (2 * (6 + 1) / 2)),
        (a.cure_temp, 500),
        (a.starlight2, 100 + (10 * (8 + 1)) / 2),
        (a.bizarre, 999999),
        (a.star_ball, 500 + 300),
        (a.rage_bless_resist, rage_bless_resist),
        (a.polymorph, 0),
        (a.starlight, 100 + (6 * (8 + 1)) / 2),
        (a.light, 0),
        (a.firebrand, 500),
        (a.restore_life, 0),
        (a.restore_exp, 0),
        (a.restore_st_lev, 0),
        (a.enlightenment, 500),
        (a.hero, 500),
        (a.shero, 500),
        (a.cure_paranoia, 100),
        (a.cure_mind, 1000),
        (a.cure_body, 1000),
        (a.mon_slow, 500),
        (a.mon_confuse, 500),
        (a.sleep_all, 500),
        (a.mon_scare, 500),
        (a.light_line, 50),
        (a.disable_traps, 50),
        (a.drain_life3, 500 + 150),
        (a.drain_life4, 500 + 250),
        (a.elec_ball, 500 + 64),
        (a.elec_ball2, 500 + 250),
        (a.acid_bolt2, 500 + (10 * (8 + 1) / 2)),
        (a.acid_bolt3, 500 + (12 * (8 + 1) / 2)),
        (a.acid_ball, 500 + 120),
        (a.cold_ball160, 500 + 160),
        (a.cold_ball2, 500 + 200),
        (a.fire_ball2, 500 + 120),
        (a.fire_ball200, 500 + 200),
        (a.fire_bolt2, 500 + (12 * (8 + 1) / 2)),
        (a.fire_bolt3, 500 + (16 * (8 + 1) / 2)),
        (a.dispel_evil60, 500 + 60),
        (a.dispel_undead, 500 + 60),
        (a.dispel_all, 500 + 60 * 2),
        (a.deep_descent, 0),
        (a.earthquakes, 500),
        (a.destruction2, 500),
        (a.losslow, 50),
        (a.lossleep, 100),
        (a.losconf, 100),
        (a.satisfy, 50),
        (a.blessing, 50),
        (a.blessing2, 50),
        (a.blessing3, 50),
        (a.glyph, 0),
        (a.tele_level, 5000),
        (a.confusing, 0),
        (a.enchant_tohit, 0),
        (a.enchant_todam, 0),
        (a.enchant_weapon, 0),
        (a.enchant_armor, 0),
        (a.enchant_armor2, 0),
        (a.remove_curse, 9000),
        (a.remove_curse2, 10000),
        (a.detect_treasure, 0),
        (a.detect_invis, 50),
        (a.detect_evil, 50),
        (a.restore_mana, 5000),
        (a.brawn, 0),
        (a.intellect, 0),
        (a.contemplation, 0),
        (a.toughness, 0),
        (a.nimbleness, 0),
        (a.restore_str, 50),
        (a.restore_int, 50),
        (a.restore_wis, 50),
        (a.restore_dex, 50),
        (a.restore_con, 50),
        (a.restore_all, 150),
        (a.tmd_free_act, 0),
        (a.tmd_infra, 0),
        (a.tmd_sinvis, 50),
        (a.tmd_esp, 50),
        (a.resist_acid, 100),
        (a.resist_elec, 100),
        (a.resist_fire, 100),
        (a.resist_cold, 100),
        (a.resist_pois, 150),
        (a.cure_confusion, 50),
        (a.wonder, 300),
        (a.wand_breath, 0),
        (a.staff_magi, 0),
        (a.staff_holy, 1000),
        (a.drink_breath, 0),
        (a.food_waybread, 50),
        (a.shroom_emergency, 0),
        (a.shroom_terror, 0),
        (a.shroom_stone, 50),
        (a.shroom_debility, 0),
        (a.shroom_sprinting, 0),
        (a.shroom_purging, 50),
        (a.ring_acid, 10000),
        (a.ring_flames, 25000),
        (a.ring_ice, 15000),
        (a.ring_lightning, 10000),
        (a.dragon_blue, 1100),
        (a.dragon_green, 2750),
        (a.dragon_red, 1100),
        (a.dragon_multihued, 3250),
        (a.dragon_gold, 5150),
        (a.dragon_chaos, 5150),
        (a.dragon_law, 5150),
        (a.dragon_balance, 5150),
        (a.dragon_shining, 5150),
        (a.dragon_power, 5150),
    ];

    chain
        .iter()
        .find(|&&(id, _)| id == act)
        .map_or(0, |&(_, bonus)| bonus)
}

/// Helper function -- calculate "power" of equipment
fn borg_power_equipment(st: &BorgState) -> i32 {
    let tr = |i: u32| st.borg.tr(i);
    let cfg = |i: u32| st.cfg(i) != 0;
    let items = &st.items.items;

    let mut value: i32 = 0;

    // Obtain the "hold" value (weight limit for weapons)
    let hold = engine::adj_str_hold(tr(BI_STR_INDEX) as usize);

    // *** Analyze weapon ***

    // Examine current weapon
    let item = &items[inven(WIELD)];

    // We give a bonus to wearing an unID'D sword in order to use it and
    // garner a pseudoID from it.  We do not do this late in the game though
    // because our weapon often has traits that we need in order to be deep
    // (FA, SeeInvis)
    if tr(BI_CDEPTH) < 10 && tr(BI_MAXCLEVEL) < 15 && item.iqty != 0 && !item.ident {
        value = addi(value, 1000000);
    }

    // Calculate "average" damage per "normal" blow
    // and assume we can enchant up to +8 if borg.trait[BI_CLEVEL] > 25
    let dd_ds = i32::from(item.dd) * i32::from(item.ds);
    let damage = (i64::from(dd_ds) * 20) as i32;

    // Reward "damage" and increased blows per round
    value = addi(value, damage.wrapping_mul(tr(BI_BLOWS) + 1));

    // Reward "bonus to hit"
    value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * 100);

    // Reward "bonus to dam"
    value = add(value, i64::from(tr(BI_TODAM) + i32::from(item.to_d)) * 30);

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * 15);
    }

    // extra boost for deep dungeon
    if tr(BI_MAXDEPTH) >= 75 {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * 15);

        value = add(value, i64::from(dd_ds) * 20 * 2 * i64::from(tr(BI_BLOWS)));
    }

    // assume 2x base damage for x% of creatures
    let dam = damage.wrapping_mul(2).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WS_ANIMAL) != 0 {
        value = addi(value, part(2));
    }
    if tr(BI_WS_EVIL) != 0 {
        value = addi(value, part(7));
    }

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // assume 3x base damage for x% of creatures
    let dam = damage.wrapping_mul(3).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WS_UNDEAD) != 0 && tr(BI_WK_UNDEAD) == 0 {
        value = addi(value, part(5));
    }
    if tr(BI_WS_DEMON) != 0 && tr(BI_WK_DEMON) == 0 {
        value = addi(value, part(3));
    }
    if tr(BI_WS_DRAGON) != 0 && tr(BI_WK_DRAGON) == 0 {
        value = addi(value, part(6));
    }
    if tr(BI_WS_GIANT) != 0 {
        value = addi(value, part(4));
    }
    if tr(BI_WB_ACID) != 0 {
        value = addi(value, part(4));
    }
    if tr(BI_WB_ELEC) != 0 {
        value = addi(value, part(5));
    }
    if tr(BI_WB_FIRE) != 0 {
        value = addi(value, part(3));
    }
    if tr(BI_WB_COLD) != 0 {
        value = addi(value, part(3));
    }
    // SOrc and STroll get 1/2 of reward now
    if tr(BI_WS_ORC) != 0 {
        value = addi(value, part(1));
    }
    if tr(BI_WS_TROLL) != 0 {
        value = addi(value, part(2));
    }
    // and the other 2/2 if SEvil not possessed
    if tr(BI_WS_ORC) != 0 && tr(BI_WS_EVIL) == 0 {
        value = addi(value, part(1));
    }
    if tr(BI_WS_TROLL) != 0 && tr(BI_WS_EVIL) == 0 {
        value = addi(value, part(1));
    }

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // assume 5x base damage for x% of creatures
    let dam = damage.wrapping_mul(5).wrapping_mul(tr(BI_BLOWS));
    let part = |n: i32| dam.wrapping_mul(n) / 2;
    if tr(BI_WK_UNDEAD) != 0 {
        value = addi(value, part(5));
    }
    if tr(BI_WK_DEMON) != 0 {
        value = addi(value, part(5));
    }
    if tr(BI_WK_DRAGON) != 0 {
        value = addi(value, part(5));
    }
    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = addi(value, dam);
    }

    // It used to be only on Grond
    if tr(BI_W_IMPACT) != 0 {
        value = add(value, 50);
    }

    // It is hard to hold a heavy weapon
    if tr(BI_HEAVYWEPON) != 0 {
        value = sub(value, 500000);
    }

    // We want low level borgs to have high blows (dagger, whips)
    if tr(BI_CLEVEL) <= 10 {
        value = add(value, i64::from(tr(BI_BLOWS)) * 45000);
    }

    // *** Analyze bow ***

    // Examine current bow
    let item = &items[inven(BOW)];

    // We give a bonus to wearing an unID'D bow in order to use it and
    // garner a pseudoID from it.  We do not do this late in the game though
    // because our weapon often has traits that we need in order to be deep
    // (FA, SeeInvis)
    if tr(BI_CDEPTH) < 10 && tr(BI_MAXCLEVEL) < 15 && item.iqty != 0 && !item.ident {
        value = addi(value, 6000000);
    }

    // Calculate "average" damage per "normal" shot (times 2)
    let damage = if i32::from(item.to_d) > 8 || tr(BI_CLEVEL) < 25 {
        (tr(BI_AMMO_SIDES) + i32::from(item.to_d)).wrapping_mul(tr(BI_AMMO_POWER))
    } else {
        (tr(BI_AMMO_SIDES) + 8).wrapping_mul(tr(BI_AMMO_POWER))
    };

    // Reward "damage"
    let shots_damage = tr(BI_SHOTS).wrapping_mul(damage);
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(shots_damage) * 11);
    } else {
        value = add(value, i64::from(shots_damage) * 9);
    }

    // Extra bonus for low levels, they need a ranged weapon
    if tr(BI_CLEVEL) < 15 {
        value = add(value, i64::from(shots_damage) * 200);
    }

    // slings force you to carry heavy ammo.  Penalty for that unless you
    // have lots of str
    if i32::from(item.sval) == st.item_val.sv_sling && item.art_idx == 0 && tr(BI_STR) < 9 {
        value = sub(value, 5000);
    }

    // Bonus if level 1 to buy a sling, they are cheap ranged weapons
    if i32::from(item.sval) == st.item_val.sv_sling && tr(BI_CLEVEL) == 1 && tr(BI_STR) >= 9 {
        value = addi(value, 8000);
    }

    // Reward "bonus to hit"
    value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * 100);

    // extra damage for some
    if cfg(BORG_WORSHIPS_DAMAGE) {
        value = add(value, i64::from(tr(BI_TOHIT) + i32::from(item.to_h)) * 25);
    }

    // Prefer bows
    if tr(BI_FAST_SHOTS) != 0 && tr(BI_AMMO_TVAL) == TV_ARROW as i32 {
        value = add(value, 30000);
    }

    // It is hard to hold a heavy weapon
    if hold < i32::from(item.weight) / 10 {
        value = sub(value, 500000);
    }

    // *** Reward various things ***

    // Reward light radius
    // necromancers like the dark
    if tr(BI_CLASS) == CLASS_NECROMANCER {
        value = sub(value, i64::from(tr(BI_LIGHT) - 1) * 10000);
    } else if tr(BI_LIGHT) <= 3 {
        value = add(value, i64::from(tr(BI_LIGHT)) * 10000);
    } else if tr(BI_LIGHT) > 3 {
        value = add(value, 30000 + i64::from(tr(BI_LIGHT).wrapping_mul(1000)));
    }

    value = add(value, i64::from(tr(BI_MOD_MOVES)) * 3000);
    value = add(value, i64::from(tr(BI_DAM_RED)) * 10000);

    // Reward speed
    // see if speed can be a bonus if good speed; not +3.
    // reward higher for +10 than +50 speed (decreased return).
    let speed = tr(BI_SPEED);
    // `((speed - base) * per) + extra`, all `long`
    let tier = |base: i32, per: i64, extra: i64| i64::from(speed - base) * per + extra;
    if cfg(BORG_WORSHIPS_SPEED) {
        if speed >= 150 {
            value = add(value, tier(120, 1500, 185000));
        }
        if (145..=149).contains(&speed) {
            value = add(value, tier(120, 1500, 180000));
        }
        if (140..=144).contains(&speed) {
            value = add(value, tier(120, 1500, 175000));
        }
        if (135..=139).contains(&speed) {
            value = add(value, tier(120, 1500, 175000));
        }
        if (130..=134).contains(&speed) {
            value = add(value, tier(120, 1500, 160000));
        }
        if (125..=129).contains(&speed) {
            value = add(value, tier(110, 1500, 135000));
        }
        if (120..=124).contains(&speed) {
            value = add(value, tier(110, 1500, 110000));
        }
        if (115..=119).contains(&speed) {
            value = add(value, tier(110, 1500, 85000));
        }
        if (110..=114).contains(&speed) {
            value = add(value, tier(110, 1500, 65000));
        }
        if speed < 110 {
            value = add(value, tier(110, 2500, 0));
        }
    } else {
        if speed >= 140 {
            value = add(value, tier(120, 1000, 175000));
        }
        if (135..=139).contains(&speed) {
            value = add(value, tier(120, 1000, 165000));
        }
        if (130..=134).contains(&speed) {
            value = add(value, tier(120, 1000, 150000));
        }
        if (125..=129).contains(&speed) {
            value = add(value, tier(110, 1000, 125000));
        }
        if (120..=124).contains(&speed) {
            value = add(value, tier(110, 1000, 100000));
        }
        if (115..=119).contains(&speed) {
            value = add(value, tier(110, 1000, 75000));
        }
        if (110..=114).contains(&speed) {
            value = add(value, tier(110, 1000, 55000));
        }
        if speed < 110 {
            value = add(value, tier(110, 2500, 0));
        }
    }

    // Reward strength bonus
    value = add(value, i64::from(tr(BI_STR_INDEX)) * 100);

    // Reward spell stat bonus
    let spell_stat = borg_spell_stat();
    if spell_stat >= 0 {
        value = add(value, i64::from(tr(BI_STR_INDEX + spell_stat as u32)) * 500);

        // Bonus for sp.
        if cfg(BORG_WORSHIPS_MANA) {
            value = add(value, i64::from(tr(BI_SP_ADJ) / 2) * 255);
        } else {
            value = add(value, i64::from(tr(BI_SP_ADJ) / 2) * 155);
        }

        // bonus for low fail rate
        value = addi(
            value,
            (100 - i32::from(engine::spell_chance(0))).wrapping_mul(100),
        );

        // should try to get min fail to 0
        if engine::player_has(PF_ZERO_FAIL) {
            // other fail rates
            if i32::from(engine::spell_chance(0)) < 1 {
                value = add(value, 30000);
            }
        }
    }

    // Dexterity Bonus --good for attacking and ac
    if tr(BI_DEX_INDEX) <= 37 {
        // Reward bonus
        value = add(value, i64::from(tr(BI_DEX_INDEX)) * 120);
    }

    // Constitution Bonus
    if tr(BI_CON_INDEX) <= 37 {
        if cfg(BORG_WORSHIPS_HP) {
            value = add(value, i64::from(tr(BI_CON_INDEX)) * 250);
            // Reward hp bonus
            // This is a bit weird because we are not really giving
            // a bonus for what hp you have, but the 'bonus'
            // hp you get getting over 800hp is very important.
            if tr(BI_HP_ADJ) < 800 {
                value = add(value, i64::from(tr(BI_HP_ADJ)) * 450);
            } else {
                value = add(value, i64::from(tr(BI_HP_ADJ) - 800) * 100 + 350 * 500);
            }
        } else {
            // does not worship hp
            value = add(value, i64::from(tr(BI_CON_INDEX)) * 150);
            // Reward hp bonus
            // This is a bit weird because we are not really giving
            // a bonus for what hp you have, but the 'bonus'
            // hp you get getting over 500hp is very important.
            if tr(BI_HP_ADJ) < 500 {
                value = add(value, i64::from(tr(BI_HP_ADJ)) * 350);
            } else {
                value = add(value, i64::from(tr(BI_HP_ADJ) - 500) * 100 + 350 * 500);
            }
        }
    }

    // HACK - a small bonus for adding to stats even above max.
    //        This will allow us to swap a ring of int +6 for
    //        our ring of int +2 even though we are at max int because
    //        we are wielding a weapon that has +4 int
    //        later it might be nice to swap to a weapon that does not
    //        have an int bonus
    for i in 0..STAT_MAX {
        value = addi(value, tr(BI_ASTR + i));
    }

    // Tiny rewards
    value = add(value, i64::from(tr(BI_DISP)) * 2);
    value = add(value, i64::from(tr(BI_DISM)) * 2);
    value = add(value, i64::from(tr(BI_DEV)) * 25);
    value = add(value, i64::from(tr(BI_SAV)) * 25);
    // perfect saves are very nice
    if tr(BI_SAV) > 99 {
        value = addi(value, 10000);
    }
    value = add(value, i64::from(tr(BI_STL)) * 2);
    value = add(value, i64::from(tr(BI_SRCH)));
    value = add(value, i64::from(tr(BI_THN)) * 5);
    value = add(value, i64::from(tr(BI_THB)) * 35);
    value = add(value, i64::from(tr(BI_THT)) * 2);
    value = add(value, i64::from(tr(BI_DIG)) * 2);

    // *** Reward current flags ***

    // Various flags
    if tr(BI_SDIG) != 0 {
        value = add(value, 750);
    }
    if tr(BI_SDIG) != 0 && tr(BI_ISHUNGRY) != 0 {
        value = add(value, 7500);
    }
    if tr(BI_SDIG) != 0 && tr(BI_ISWEAK) != 0 {
        value = add(value, 7500);
    }

    // Feather Fall if low level is nice
    if tr(BI_MAXDEPTH) < 20 {
        if tr(BI_FEATH) != 0 {
            value = add(value, 500);
        }
    } else if tr(BI_FEATH) != 0 {
        value = addi(value, 50);
    }

    if tr(BI_LIGHT) != 0 {
        value = add(value, 2000);
    }

    if tr(BI_ESP) != 0 && tr(BI_SINV) != 0 {
        value = add(value, 500);
    }

    if tr(BI_DINV) == 0 && tr(BI_SINV) != 0 {
        value = add(value, 5000);
    }

    if tr(BI_FRACT) != 0 {
        value = add(value, 10000);
    }

    // after you max out you are pretty safe from drainers.
    if tr(BI_MAXCLEVEL) < 50 {
        if tr(BI_HLIFE) != 0 {
            value = add(value, 2000);
        }
    } else if tr(BI_HLIFE) != 0 {
        value = add(value, 200);
    }
    if tr(BI_REG) != 0 {
        value = add(value, 2000);
    }
    if tr(BI_ESP) != 0 {
        value = add(value, 80000);
    }

    // Immunity flags
    if tr(BI_ICOLD) != 0 {
        value = add(value, 65000);
    }
    if tr(BI_IELEC) != 0 {
        value = add(value, 40000);
    }
    if tr(BI_IFIRE) != 0 {
        value = add(value, 80000);
    }
    if tr(BI_IACID) != 0 {
        value = add(value, 50000);
    }

    // Resistance flags
    if tr(BI_RCOLD) != 0 {
        value = add(value, 3000);
    }
    if tr(BI_RELEC) != 0 {
        value = add(value, 4000);
    }
    if tr(BI_RACID) != 0 {
        value = add(value, 6000);
    }
    if tr(BI_RFIRE) != 0 {
        value = add(value, 8000);
    }
    // extra bonus for getting all basic resist
    if tr(BI_RFIRE) != 0 && tr(BI_RACID) != 0 && tr(BI_RELEC) != 0 && tr(BI_RCOLD) != 0 {
        value = add(value, 10000);
    }
    if tr(BI_RPOIS) != 0 {
        value = add(value, 20000);
    }
    if tr(BI_RSND) != 0 {
        value = add(value, 3500);
    }
    if tr(BI_RLITE) != 0 {
        value = add(value, 800);
    }
    if tr(BI_RDARK) != 0 {
        value = add(value, 800);
    }
    if tr(BI_RKAOS) != 0 {
        value = add(value, 5000);
    }

    // this is way boosted to avoid carrying stuff you don't need
    if tr(BI_RCONF) != 0 {
        value = add(value, 80000);
    }

    // mages need a slight boost for this
    if tr(BI_CLASS) == CLASS_MAGE && tr(BI_RCONF) != 0 {
        value = add(value, 2000);
    }
    if tr(BI_RDIS) != 0 {
        value = add(value, 5000);
    }
    if tr(BI_RSHRD) != 0 {
        value = add(value, 100);
    }
    if tr(BI_RNXUS) != 0 {
        value = add(value, 100);
    }
    if tr(BI_RBLIND) != 0 {
        value = add(value, 5000);
    }
    if tr(BI_RNTHR) != 0 {
        value = add(value, 5500);
    }
    if tr(BI_RFEAR) != 0 {
        value = add(value, 2000);
    }

    // Sustain flags
    if tr(BI_SSTR) != 0 {
        value = add(value, 50);
    }
    if tr(BI_SINT) != 0 {
        value = add(value, 50);
    }
    if tr(BI_SWIS) != 0 {
        value = add(value, 50);
    }
    if tr(BI_SCON) != 0 {
        value = add(value, 50);
    }
    if tr(BI_SDEX) != 0 {
        value = add(value, 50);
    }
    // boost for getting them all
    if tr(BI_SSTR) != 0
        && tr(BI_SINT) != 0
        && tr(BI_SWIS) != 0
        && tr(BI_SDEX) != 0
        && tr(BI_SCON) != 0
    {
        value = add(value, 1000);
    }

    // *** XXX XXX XXX Reward "necessary" flags ***

    // `borg.trait[BI_MAXDEPTH] + 1 >= d`
    let deep = |d: i32| tr(BI_MAXDEPTH) + 1 >= d;

    // Mega-Hack -- See invisible (level 10)
    if (tr(BI_SINV) != 0 || tr(BI_ESP) != 0) && deep(10) {
        value = add(value, 100000);
    }

    // Mega-Hack -- Free action (level 20)
    if tr(BI_FRACT) != 0 && deep(20) {
        value = add(value, 100000);
    }

    //  Mega-Hack -- resists (level 25)
    if tr(BI_RFIRE) != 0 && deep(25) {
        value = add(value, 100000);
    }

    //  Mega-Hack -- resists (level 40)
    if tr(BI_RPOIS) != 0 && deep(40) {
        value = add(value, 100000);
    }
    if tr(BI_RELEC) != 0 && deep(40) {
        value = add(value, 100000);
    }
    if tr(BI_RACID) != 0 && deep(40) {
        value = add(value, 100000);
    }
    if tr(BI_RCOLD) != 0 && deep(40) {
        value = add(value, 100000);
    }

    //  Mega-Hack -- Speed / Hold Life (level 46) and maxed out
    if tr(BI_HLIFE) != 0 && deep(46) && tr(BI_MAXCLEVEL) < 50 {
        value = add(value, 100000);
    }
    if tr(BI_SPEED) >= 115 && deep(46) {
        value = add(value, 100000);
    }
    if tr(BI_RCONF) != 0 && deep(46) {
        value = add(value, 100000);
    }

    //  Mega-Hack -- resist Nether is -very- nice to have at level 50
    if tr(BI_RNTHR) != 0 && deep(50) {
        value = add(value, 55000);
    }

    //  Mega-Hack -- resist Sound to avoid being KO'd
    if tr(BI_RSND) != 0 && deep(50) {
        value = add(value, 100000);
    }

    //  Mega-Hack -- resists & Telepathy (level 55)
    if tr(BI_RBLIND) != 0 && deep(55) {
        value = add(value, 100000);
    }
    if tr(BI_ESP) != 0 && deep(55) {
        value = add(value, 100000);
    }
    if tr(BI_RNTHR) != 0 && deep(60) {
        value = add(value, 55000);
    }

    //  Mega-Hack -- resists & +10 speed (level 60)
    if tr(BI_RKAOS) != 0 && deep(60) {
        value = add(value, 104000);
    }
    if tr(BI_RDIS) != 0 && deep(60) {
        value = add(value, 90000);
    }
    if tr(BI_SPEED) >= 120 && deep(60) {
        value = add(value, 100000);
    }

    //  Must have +20 speed (level 80)
    if tr(BI_SPEED) >= 130 && deep(80) {
        value = add(value, 100000);
    }

    // Not Req, but a good idea:
    // Extra boost to Nether deeper down
    // RDark for deeper uniques
    // Good to have +30 speed
    if tr(BI_RNTHR) != 0 && deep(80) {
        value = add(value, 15000);
    }
    if tr(BI_RDARK) != 0 && deep(80) {
        value = add(value, 25000);
    }
    if tr(BI_SPEED) >= 140 && deep(80) && tr(BI_CLASS) == CLASS_WARRIOR {
        value = add(value, 100000);
    }

    // *** Reward powerful armor ***

    // Reward armor
    let armor = tr(BI_ARMOR);
    if cfg(BORG_WORSHIPS_AC) {
        if armor < 15 {
            value = add(value, i64::from(armor) * 2500);
        }
        if (15..75).contains(&armor) {
            value = add(value, i64::from(armor) * 2000 + 28250);
        }
        if armor >= 75 {
            value = add(value, i64::from(armor) * 1500 + 73750);
        }
    } else {
        if armor < 15 {
            value = add(value, i64::from(armor) * 2000);
        }
        if (15..75).contains(&armor) {
            value = add(value, i64::from(armor) * 500 + 28350);
        }
        if armor >= 75 {
            value = add(value, i64::from(armor) * 100 + 73750);
        }
    }

    // *** Penalize various things ***

    // Penalize various flags
    if tr(BI_CRSAGRV) != 0 {
        value = sub(value, 800000);
    }
    if tr(BI_CRSHPIMP) != 0 {
        value = subi(value, 35000);
    }
    if tr(BI_CLASS) != CLASS_WARRIOR && tr(BI_CRSMPIMP) != 0 {
        value = subi(value, 15000);
    }
    if (tr(BI_CLASS) == CLASS_MAGE
        || tr(BI_CLASS) == CLASS_PRIEST
        || tr(BI_CLASS) == CLASS_DRUID
        || tr(BI_CLASS) == CLASS_NECROMANCER)
        && tr(BI_CRSMPIMP) != 0
    {
        value = subi(value, 15000);
    }
    if tr(BI_CRSFEAR) != 0 {
        value = sub(value, 400000);
    }
    if tr(BI_CRSFEAR) != 0 && tr(BI_CLASS) != CLASS_MAGE {
        value = sub(value, 200000);
    }
    if tr(BI_CRSDRAIN_XP) != 0 {
        value = sub(value, 400000);
    }
    if tr(BI_CRSFVULN) != 0 {
        value = subi(value, 30000);
    }
    if tr(BI_CRSEVULN) != 0 {
        value = subi(value, 30000);
    }
    if tr(BI_CRSCVULN) != 0 {
        value = subi(value, 30000);
    }
    if tr(BI_CRSAVULN) != 0 {
        value = subi(value, 30000);
    }

    if tr(BI_CRSTELE) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSENVELOPING) != 0 {
        value = sub(value, 50000);
    }
    if tr(BI_CRSIRRITATION) != 0 {
        value = sub(value, 20000);
    }
    if tr(BI_CRSPOIS) != 0 {
        value = sub(value, 10000);
    }
    if tr(BI_CRSSIREN) != 0 {
        value = sub(value, 800000);
    }
    if tr(BI_CRSHALU) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSPARA) != 0 {
        value = sub(value, 800000);
    }
    if tr(BI_CRSSDEM) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSSDRA) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSSUND) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSSTONE) != 0 && tr(BI_SPEED) < 140 {
        value = sub(value, 10000);
    }
    if tr(BI_CRSSTEELSKIN) != 0 && tr(BI_SPEED) < 140 {
        value = sub(value, 10000);
    }
    if tr(BI_CRSNOTEL) != 0 {
        value = sub(value, 700000);
    }
    if tr(BI_CRSTWEP) != 0 {
        value = sub(value, 100000);
    }
    if tr(BI_CRSAIRSWING) != 0 {
        value = sub(value, 10000);
    }
    if tr(BI_CRSUNKNO) != 0 {
        value = sub(value, 9999999);
    }

    // (C has a disabled `#if 0` Morgoth block here.)

    // bonus for something with multiple useful bonuses
    value = add(value, 3000 * i64::from(tr(BI_MULTIPLE_BONUSES)));

    // bonus for wearing something that needs an ID
    value = add(value, 10000 * i64::from(tr(BI_WORN_NEED_ID)));

    let act_max = i32::from(engine::z_info().act_max);
    for act in 1..act_max {
        let n = st.borg.activation[act as usize];
        if n == 0 {
            continue;
        }
        // an extra bonus for activations; the bonus is per item that has
        // this activation
        let bonus = activation_bonus(&st.act, tr(BI_CLASS), tr(BI_CLEVEL), act);
        value = addi(value, bonus.wrapping_mul(n));
    }

    // Hack-- Reward the borg for carrying a NON-ID items that have random
    // powers
    if items[inven(OUTER)].iqty != 0 {
        let item = &items[inven(OUTER)];
        if borg_ego_has_random_power(usize::from(item.ego_idx)) && !item.ident {
            value = add(value, 999999);
        }
    }

    // `borg_item_weight(&borg_items[slot])`
    let weight = |off: usize| i32::from(borg_item_weight(&items[inven(off)]));

    // *** Penalize armor weight ***
    if tr(BI_STR_INDEX) < 15 {
        if weight(BODY) > 200 {
            value = subi(value, (weight(BODY) - 200).wrapping_mul(15));
        }
        if weight(HEAD) > 30 {
            value = subi(value, 250);
        }
        if weight(ARM) > 10 {
            value = subi(value, 250);
        }
        if weight(FEET) > 50 {
            value = subi(value, 250);
        }
    }

    // Compute the total armor weight
    let mut cur_wgt: i32 = 0;
    cur_wgt += weight(BODY);
    cur_wgt += weight(HEAD);
    cur_wgt += weight(ARM);
    cur_wgt += weight(OUTER);
    cur_wgt += weight(HANDS);
    cur_wgt += weight(FEET);

    // Determine the weight allowance
    let max_wgt = engine::class_spell_weight();

    // Heavy armor hurts magic
    if engine::class_total_spells() != 0 && (cur_wgt - max_wgt) / 10 > 0 {
        let over = (cur_wgt - max_wgt) / 10;
        // max sp must be calculated in case it changed with the armor
        let mut max_sp = tr(BI_SP_ADJ) / 100 + 1;
        max_sp -= over;
        // Mega-Hack -- Penalize heavy armor which hurts mana
        if (300..=350).contains(&max_sp) {
            value = sub(value, i64::from(over) * 400);
        }
        if (200..=299).contains(&max_sp) {
            value = sub(value, i64::from(over) * 800);
        }
        if (100..=199).contains(&max_sp) {
            value = sub(value, i64::from(over) * 1600);
        }
        if max_sp <= 99 {
            value = sub(value, i64::from(over) * 3200);
        }
    }

    // Result
    value
}

/// Helper function -- calculate power of inventory
fn borg_power_inventory(st: &BorgState) -> i32 {
    let tr = |i: u32| st.borg.tr(i);
    let has = |kv: i32| st.borg.has[kv as usize];
    let cfg = |i: u32| st.cfg(i);
    let iv = &st.item_val;

    // `borg_cfg[BORG_MUNCHKIN_START] && borg.trait[BI_MAXCLEVEL] <
    // borg_cfg[BORG_MUNCHKIN_LEVEL]`: "Do not carry these until later"
    let munchkin_wait =
        || cfg(BORG_MUNCHKIN_START) != 0 && tr(BI_MAXCLEVEL) < cfg(BORG_MUNCHKIN_LEVEL);

    let mut value: i32 = 0;
    let v = &mut value;

    // *** Basic abilities ***

    // Reward fuel
    let mut k = 0;
    count_up(v, &mut k, 6, tr(BI_AFUEL), 60000);
    if tr(BI_STR) >= 15 {
        while k < 10 && k < tr(BI_AFUEL) {
            *v = add(*v, 6000 - i64::from(k * 100));
            k += 1;
        }
    }

    // Reward Food
    // if hungry, food is THE top priority
    if (tr(BI_ISHUNGRY) != 0 || tr(BI_ISWEAK) != 0) && tr(BI_FOOD) != 0 {
        *v = addi(*v, 100000);
    }
    k = 0;
    count_up(v, &mut k, 7, tr(BI_FOOD), 50000);
    if tr(BI_STR) >= 15 {
        count_up(v, &mut k, 10, tr(BI_FOOD), 200);
    }
    if tr(BI_REG) != 0 && tr(BI_CLEVEL) <= 15 {
        k = 0;
        count_up(v, &mut k, 15, tr(BI_FOOD), 700);
    }

    // Prefer to buy HiCalorie foods over LowCalorie
    k = 0;
    count_up(v, &mut k, 7, tr(BI_FOOD_HI), 52);
    k = 0;
    count_down(v, &mut k, 15, tr(BI_FOOD_LO), 2);

    // Reward Cure Poison and Cuts
    let hurt = tr(BI_ISCUT) != 0 || tr(BI_ISPOISONED) != 0;
    if hurt && tr(BI_ACCW) != 0 {
        *v = addi(*v, 100000);
    }
    if hurt && tr(BI_AHEAL) != 0 {
        *v = addi(*v, 50000);
    }
    if hurt && tr(BI_ACSW) != 0 {
        // usually takes more than one
        k = 0;
        count_up(v, &mut k, 5, tr(BI_ACSW), 25000);
    }
    if tr(BI_ISPOISONED) != 0 && tr(BI_ACUREPOIS) != 0 {
        *v = addi(*v, 15000);
    }

    // collect Resistance pots if not immune -- All Classes
    if tr(BI_IPOIS) == 0 && tr(BI_ACUREPOIS) <= 20 && !munchkin_wait() {
        k = 0;
        count_up(v, &mut k, 4, tr(BI_ARESPOIS), 300);
    }

    // Reward Resistance Potions for Warriors
    if tr(BI_CLASS) == CLASS_WARRIOR && tr(BI_MAXDEPTH) > 20 {
        k = 0;
        if !munchkin_wait() {
            // collect pots if not immune
            if tr(BI_IFIRE) == 0 {
                count_up(v, &mut k, 4, tr(BI_ARESHEAT), 500);
            }
            k = 0;
            // collect pots if not immune
            if tr(BI_ICOLD) == 0 {
                count_up(v, &mut k, 4, tr(BI_ARESCOLD), 500);
            }
            k = 0;
            // collect pots if not immune
            if tr(BI_IPOIS) == 0 {
                count_up(v, &mut k, 4, tr(BI_ARESPOIS), 500);
            }
        }
    }

    // Reward ident
    k = 0;
    if tr(BI_CLEVEL) >= 10 {
        count_up(v, &mut k, 5, tr(BI_AID), 6000);
        if tr(BI_STR) >= 15 {
            count_up(v, &mut k, 15, tr(BI_AID), 600);
        }
    }
    // Reward ID if I am carrying a {magical} or {excellent} item
    if tr(BI_ALL_NEED_ID) != 0 {
        k = 0;
        while k < tr(BI_ALL_NEED_ID) && k < tr(BI_AID) {
            *v = add(*v, 6000);
            k += 1;
        }
    }

    //  Reward PFE carry lots of these
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, 10, tr(BI_APFE), 10000);
        count_up(v, &mut k, 25, tr(BI_APFE), 2000);
    }
    //  Reward Glyph- Rune of Protection-  carry lots of these
    k = 0;
    count_up(v, &mut k, 10, tr(BI_AGLYPH), 10000);
    count_up(v, &mut k, 25, tr(BI_AGLYPH), 2000);
    if tr(BI_MAXDEPTH) >= 100 {
        k = 0;
        count_up(v, &mut k, 10, tr(BI_AGLYPH), 2500);
        count_up(v, &mut k, 75, tr(BI_AGLYPH), 2500);
    }

    // Reward Scroll of Mass Genocide, only when fighting Morgoth
    if tr(BI_MAXDEPTH) >= 100 {
        k = 0;
        count_up(v, &mut k, 99, tr(BI_AMASSBAN), 2500);
    }

    // Reward recall
    k = 0;
    if tr(BI_CLEVEL) > 7 && !munchkin_wait() {
        count_up(v, &mut k, 3, tr(BI_RECALL), 50000);
        if tr(BI_STR) >= 15 {
            count_up(v, &mut k, 7, tr(BI_RECALL), 5000);
        }
        // Deep borgs need the rod to avoid low mana traps
        if tr(BI_MAXDEPTH) >= 50 && has(iv.kv_rod_recall) != 0 {
            *v = addi(*v, 12000);
        }
    }

    // Reward phase
    k = 1;
    // first phase door is very important
    if tr(BI_APHASE) != 0 {
        *v = addi(*v, 50000);
    }
    if !munchkin_wait() {
        count_up(v, &mut k, 8, tr(BI_APHASE), 500);
        if tr(BI_STR) >= 15 {
            count_up(v, &mut k, 15, tr(BI_APHASE), 500);
        }
    }

    // Reward escape (staff of teleport or artifact)
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, 2, tr(BI_AESCAPE), 10000);
        if tr(BI_MAXDEPTH) > 70 {
            k = 0;
            count_up(v, &mut k, 3, tr(BI_AESCAPE), 10000);
        }
    }

    // Reward teleport scroll
    k = 0;
    if tr(BI_CLEVEL) >= 3 && tr(BI_ATELEPORT) != 0 {
        *v = add(*v, 10000);
    }
    if tr(BI_CLEVEL) >= 7 {
        count_up(v, &mut k, 3, tr(BI_ATELEPORT), 10000);
    }
    if tr(BI_CLEVEL) >= 30 {
        count_up(v, &mut k, 10, tr(BI_ATELEPORT), 10000);
    }

    // Reward Teleport Level scrolls
    k = 0;
    if tr(BI_CLEVEL) >= 15 {
        count_up(v, &mut k, 5, tr(BI_ATELEPORTLVL), 5000);
    }

    // carry a couple of *Healing* for emergency, store the rest
    let ezheal = |v: &mut i32| {
        let mut k = 0;
        if tr(BI_MAXDEPTH) >= 46 {
            if tr(BI_PREP_BIG_FIGHT) != 0 {
                count_up(v, &mut k, 1, tr(BI_AEZHEAL), 10000);
            } else {
                count_up(v, &mut k, 2, tr(BI_AEZHEAL), 10000);
            }
        }
    };

    // *** Healing ***
    // !TODO !FIX make sure these numbers make sense for the new classes
    let class = tr(BI_CLASS);
    if class == CLASS_WARRIOR || class == CLASS_ROGUE || class == CLASS_BLACKGUARD {
        k = 0;
        count_up(v, &mut k, 15, tr(BI_AHEAL), 8000);

        ezheal(v);

        // These guys need to carry the rods more, town runs low on supply.
        k = 0;
        count_up(v, &mut k, 6, has(iv.kv_rod_healing), 20000);
    } else if class == CLASS_RANGER
        || class == CLASS_PALADIN
        || class == CLASS_NECROMANCER
        || class == CLASS_MAGE
    {
        k = 0;
        count_up(v, &mut k, 10, tr(BI_AHEAL), 4000);

        ezheal(v);

        if tr(BI_CLASS) == CLASS_PALADIN {
            // Reward heal potions
            k = 0;
            count_up(v, &mut k, 3, has(iv.kv_potion_healing), 5000);
        }

        // These guys need to carry the rods more, town runs low on supply.
        k = 0;
        count_up(v, &mut k, 4, has(iv.kv_rod_healing), 20000);
    } else if class == CLASS_PRIEST || class == CLASS_DRUID {
        // Level 1 priests are given a Potion of Healing.  It is better
        // for them to sell that potion and buy equipment or several
        // Cure Crits with it.
        if tr(BI_CLEVEL) == 1 {
            k = 0;
            count_down(v, &mut k, 10, has(iv.kv_potion_healing), 2000);
        }
        // Reward heal potions
        k = 0;
        count_up(v, &mut k, 5, has(iv.kv_potion_healing), 2000);

        ezheal(v);
    }

    // Collecting Potions, prepping for Morgoth/Sauron fight
    // only carry them if we aren't prepping, otherwise they store at
    // home
    if tr(BI_MAXDEPTH) >= 99 && tr(BI_PREP_BIG_FIGHT) == 0 {
        k = 0;
        count_up(v, &mut k, 99, has(iv.kv_potion_healing), 8000);
        k = 0;
        count_up(v, &mut k, 99, tr(BI_AEZHEAL), 10000);
        k = 0;
        count_up(v, &mut k, 99, tr(BI_ASPEED), 8000);
        k = 0;
        count_up(v, &mut k, 99, tr(BI_ALIFE), 10000);
        k = 0;
        if tr(BI_CLASS) != CLASS_WARRIOR {
            count_up(v, &mut k, 99, has(iv.kv_potion_restore_mana), 5000);
        }
        k = 0;
        count_up(v, &mut k, 40, has(iv.kv_mush_stoneskin), 5000);

        // Sauron is dead -- store them unless I have enough
        if tr(BI_SAURON_DEAD) != 0 {
            // Reward Scroll of Mass Genocide, only when fighting Morgoth
            k = 0;
            count_up(v, &mut k, 99, tr(BI_AMASSBAN), 2500);
        }
    }

    // Restore Mana
    if tr(BI_MAXSP) > 100 {
        k = 0;
        count_up(v, &mut k, 10, has(iv.kv_potion_restore_mana), 4000);
        k = 0;
        count_up(v, &mut k, 100, tr(BI_ASTFMAGI), 4000);
    }

    // Reward Cure Critical.  Heavy reward on first 5
    if tr(BI_CLEVEL) < 35 && tr(BI_CLEVEL) > 10 {
        k = 0;
        count_up(v, &mut k, 10, tr(BI_ACCW), 5000);
    } else if tr(BI_CLEVEL) >= 35 {
        // Reward cure critical.  Later on in game.
        k = 0;
        count_up(v, &mut k, 10, tr(BI_ACCW), 5000);
        if tr(BI_STR) > 15 {
            count_up(v, &mut k, 15, tr(BI_ACCW), 500);
        }
    }

    // Reward cure serious -- only reward serious if low on crits
    if tr(BI_ACCW) < 5 && tr(BI_MAXCLEVEL) > 10 && (tr(BI_CLEVEL) < 35 || tr(BI_RCONF) == 0) {
        k = 0;
        count_up(v, &mut k, 7, tr(BI_ACSW), 50);
        if tr(BI_STR) > 15 {
            count_up(v, &mut k, 10, tr(BI_ACSW), 5);
        }
    }

    // Reward cure light -- Low Level Characters
    if tr(BI_ACCW) + tr(BI_ACSW) < 5 && tr(BI_CLEVEL) < 8 {
        k = 0;
        count_up(v, &mut k, 5, tr(BI_ACLW), 550);
    }

    // Reward Cures
    if tr(BI_RCONF) == 0 {
        if cfg(BORG_MUNCHKIN_START) != 0 && tr(BI_MAXCLEVEL) < 10 {
            // Do not carry these until later
        } else {
            k = 0;
            count_up(v, &mut k, 10, tr(BI_FOOD_CURE_CONF), 400);
        }
    }
    if tr(BI_RBLIND) == 0 {
        k = 0;
        if !munchkin_wait() {
            count_up(v, &mut k, 5, tr(BI_FOOD_CURE_BLIND), 300);
        }
    }
    if tr(BI_RPOIS) == 0 {
        k = 0;
        if !munchkin_wait() {
            count_up(v, &mut k, 5, tr(BI_ACUREPOIS), 250);
        }
    }

    // *** Detection ***

    // Reward detect trap
    k = 0;
    count_up(v, &mut k, 1, tr(BI_ADETTRAP), 4000);

    // Reward detect door
    k = 0;
    count_up(v, &mut k, 1, tr(BI_ADETDOOR), 2000);

    // Reward detect evil
    if tr(BI_ESP) == 0 {
        k = 0;
        count_up(v, &mut k, 1, tr(BI_ADETEVIL), 1000);
    }

    // Reward magic mapping
    k = 0;
    count_up(v, &mut k, 1, tr(BI_AMAGICMAP), 4000);

    // Reward call lite
    if tr(BI_CLASS) != CLASS_NECROMANCER {
        k = 0;
        count_up(v, &mut k, 1, tr(BI_ALITE), 1000);
    }

    // Genocide scrolls. Just scrolls, mainly used for Morgoth
    if tr(BI_MAXDEPTH) >= 100 {
        k = 0;
        count_up(v, &mut k, 10, has(iv.kv_scroll_mass_banishment), 10000);
        count_up(v, &mut k, 25, has(iv.kv_scroll_mass_banishment), 2000);
    }

    // Reward speed potions/rods/staves (but no staves deeper than depth 95)
    k = 0;
    if !munchkin_wait() {
        count_up(v, &mut k, 20, tr(BI_ASPEED), 5000);
    }

    // Reward Recharge ability
    if tr(BI_ARECHARGE) != 0 && tr(BI_MAXDEPTH) < 99 {
        *v = add(*v, 5000);
    }

    // *** Missiles ***

    // Reward missiles
    if tr(BI_CLASS) == CLASS_RANGER || tr(BI_CLASS) == CLASS_WARRIOR {
        k = 0;
        count_up(v, &mut k, 40, tr(BI_AMISSILES), 100);
        if tr(BI_STR) > 15 && tr(BI_STR) <= 18 {
            count_up(v, &mut k, 80, tr(BI_AMISSILES), 10);
        }
        if tr(BI_STR) > 18 {
            count_up(v, &mut k, 180, tr(BI_AMISSILES), 8);
        }

        // penalize use of too many quiver slots
        k = 4;
        while k < tr(BI_QUIVER_SLOTS) {
            *v = sub(*v, 10000);
            k += 1;
        }
    } else {
        k = 0;
        count_up(v, &mut k, 20, tr(BI_AMISSILES), 100);
        if tr(BI_STR) > 15 {
            count_up(v, &mut k, 50, tr(BI_AMISSILES), 10);
        }
        // Don't carry too many
        if tr(BI_STR) <= 15 && tr(BI_AMISSILES) > 20 {
            *v = sub(*v, 1000);
        }
        // penalize use of too many quiver slots
        k = 2;
        while k < tr(BI_QUIVER_SLOTS) {
            *v = sub(*v, 10000);
            k += 1;
        }
    }

    // cursed arrows are "bad"
    *v = sub(*v, 1000 * i64::from(tr(BI_AMISSILES_CURSED)));

    // ego arrows are worth a bonus
    *v = add(*v, 100 * i64::from(tr(BI_AMISSILES_SPECIAL)));

    // *** Various ***

    //  -- Reward carrying a staff of destruction.
    if tr(BI_ASTFDEST) != 0 {
        *v = add(*v, 5000);
    }
    k = 0;
    count_up(v, &mut k, 9, tr(BI_ASTFDEST), 200);

    //  -- Reward carrying a wand of Teleport Other.
    if tr(BI_ATPORTOTHER) != 0 {
        *v = add(*v, 5000);
    }
    // Much greater reward for Warrior
    if tr(BI_CLASS) == CLASS_WARRIOR && tr(BI_ATPORTOTHER) != 0 {
        *v = add(*v, 50000);
    }
    // reward per charge
    k = 0;
    count_up(v, &mut k, 15, tr(BI_ATPORTOTHER), 5000);

    //  -- Reward carrying an attack wand.
    if (has(iv.kv_wand_magic_missile) != 0 || has(iv.kv_wand_stinking_cloud) != 0)
        && tr(BI_MAXDEPTH) < 30
    {
        *v = add(*v, 5000);
    }
    if has(iv.kv_wand_annihilation) != 0 && tr(BI_CDEPTH) < 30 {
        *v = add(*v, 5000);
    }
    // Much greater reward for Warrior or lower level
    if (tr(BI_CLASS) == CLASS_WARRIOR || tr(BI_CLEVEL) <= 20)
        && (has(iv.kv_wand_magic_missile) != 0
            || has(iv.kv_wand_annihilation) != 0
            || has(iv.kv_wand_stinking_cloud) != 0)
    {
        *v = add(*v, 10000);
    }
    // reward per charge
    *v = add(*v, i64::from(tr(BI_GOOD_W_CHG)) * 50);

    // These staves are great but do not clutter inven with them
    //  -- Reward carrying a staff of holiness/power
    if tr(BI_GOOD_S_CHG) != 0 {
        *v = add(*v, 2500);
    }
    k = 0;
    count_up(v, &mut k, 3, tr(BI_GOOD_S_CHG), 500);

    // Rods of attacking are good too
    k = 0;
    count_up(v, &mut k, 6, tr(BI_AROD1), 8000);
    k = 0;
    count_up(v, &mut k, 6, tr(BI_AROD2), 12000);

    // Reward being at max stat
    let need = &st.borg.need_statgain;
    if !need[STAT_STR as usize] {
        *v = addi(*v, 50000);
    }
    if !need[STAT_INT as usize] {
        *v = addi(*v, 20000);
    }
    if !need[STAT_WIS as usize] {
        *v = addi(*v, 20000);
    }
    if !need[STAT_DEX as usize] {
        *v = addi(*v, 50000);
    }
    if !need[STAT_CON as usize] {
        *v = addi(*v, 50000);
    }

    let spell_stat = borg_spell_stat();
    if spell_stat >= 0 && !need[spell_stat as usize] {
        *v = addi(*v, 50000);
    }

    // Reward stat potions
    let amt = &st.borg.amt_statgain;
    if amt[STAT_STR as usize] != 0 && tr(BI_CSTR) < 18 + 100 {
        *v = addi(*v, 550000);
    }
    if amt[STAT_INT as usize] != 0 && tr(BI_CINT) < 18 + 100 {
        *v = addi(*v, 520000);
    }
    if spell_stat >= 0
        && amt[spell_stat as usize] != 0
        && tr(BI_CSTR + spell_stat as u32) < 18 + 100
    {
        *v = addi(*v, 575000);
    }
    if amt[STAT_WIS as usize] != 0 && tr(BI_CWIS) < 18 + 100 {
        *v = addi(*v, 520000);
    }
    if amt[STAT_DEX as usize] != 0 && tr(BI_CDEX) < 18 + 100 {
        *v = addi(*v, 550000);
    }
    if amt[STAT_CON as usize] != 0 && tr(BI_CCON) < 18 + 100 {
        *v = addi(*v, 550000);
    }

    // Reward Remove Curse
    if tr(BI_FIRST_CURSED) != 0 {
        if has(iv.kv_scroll_star_remove_curse) != 0 {
            *v = addi(*v, 90000);
        }
        if has(iv.kv_scroll_remove_curse) != 0 {
            *v = addi(*v, 90000);
        }
    }

    // Restore experience
    if tr(BI_HASFIXEXP) != 0 {
        *v = addi(*v, 50000);
    }

    // *** Enchantment ***

    // Reward enchant armor
    if tr(BI_AENCH_ARM) < 1000 && tr(BI_NEED_ENCHANT_TO_A) != 0 {
        *v = add(*v, 540);
    }

    // Reward enchant weapon to hit
    if tr(BI_AENCH_TOH) < 1000 && tr(BI_NEED_ENCHANT_TO_H) != 0 {
        *v = add(*v, 540);
    }

    // Reward enchant weapon to damage
    if tr(BI_AENCH_TOD) < 1000 && tr(BI_NEED_ENCHANT_TO_D) != 0 {
        *v = add(*v, 500);
    }

    // Reward *enchant weapon* to damage
    if tr(BI_AENCH_SWEP) != 0 {
        *v = add(*v, 5000);
    }

    // Reward *enchant armour*
    if tr(BI_AENCH_SARM) != 0 {
        *v = add(*v, 5000);
    }

    // Reward empty slots (up to 6)
    k = 1;
    count_up(v, &mut k, 6, tr(BI_EMPTY), 40);

    // first empty slot is highly rewarded
    if tr(BI_EMPTY) != 0 {
        *v = add(*v, 4000);
    }

    // Reward carrying a shovel if low level
    if tr(BI_MAXDEPTH) <= 40
        && tr(BI_MAXDEPTH) >= 25
        && tr(BI_GOLD) < 100000
        && u32::from(st.items.items[inven(WIELD)].tval) != TV_DIGGING
        && tr(BI_ADIGGER) == 1
    {
        *v = add(*v, 5000);
    }

    // *** Books ***
    //   Reward books
    for book in 0..9 {
        let copies = i32::from(st.borg.amt_book[book as usize]);

        // No copies
        if copies == 0 {
            continue;
        }

        // The "hard" books
        if engine::class_book_dungeon(book) {
            // Scan the spells
            for what in 0..9 {
                let Some(si) = borg_get_spell_entry(st, book, what) else {
                    break;
                };
                let spell = &st.magic.magics[si];

                // Track minimum level
                if i32::from(spell.level) > tr(BI_MAXCLEVEL) {
                    continue;
                }

                // Track Mana req.
                if i32::from(spell.power) > tr(BI_MAXSP) {
                    continue;
                }

                // Reward the book based on the spells I can cast
                *v = add(*v, 15000);
            }
        }
        // The "easy" books
        else {
            let mut when = 99;

            // Scan the spells
            for what in 0..9 {
                let Some(si) = borg_get_spell_entry(st, book, what) else {
                    break;
                };

                // Track minimum level
                let level = i32::from(st.magic.magics[si].level);
                if level < when {
                    when = level;
                }

                // Track Mana req.
                // if (as->power < mana) mana = as->power;
            }

            // Ignore "difficult" normal books
            if when > 5 && when >= tr(BI_MAXCLEVEL) + 2 {
                continue;
            }
            // if (mana > borg.trait[BI_MAXSP]) continue;

            // Reward the book
            k = 0;
            count_up(v, &mut k, 1, copies, 500000);
            if tr(BI_STR) > 5 {
                count_up(v, &mut k, 2, copies, 10000);
            }
        }
    }

    // Apply "encumbrance" from weight

    // XXX XXX XXX Apply "encumbrance" from weight
    if tr(BI_WEIGHT) > tr(BI_CARRY) / 2 {
        // *HACK*  when testing items, the borg puts them in the last empty
        // slot so this is POSSIBLY just a test item
        // (C starts at `PACK_SLOTS`, one past the last usable pack slot)
        let pack_slots = i32::from(engine::z_info().pack_size) - tr(BI_QUIVER_SLOTS);
        let mut found = None;
        let mut i = pack_slots;
        while i >= 0 {
            if st.items.items[i as usize].iqty != 0 {
                found = Some(&st.items.items[i as usize]);
                break;
            }
            i -= 1;
        }

        // Some items will be used immediately and should not contribute to
        // encumbrance
        let used_at_once = found.is_some_and(|item| {
            let sval = i32::from(item.sval);
            let tval = u32::from(item.tval);
            item.iqty != 0
                && item.aware
                && ((tval == TV_SCROLL
                    && ((sval == iv.sv_scroll_enchant_armor
                        && tr(BI_AENCH_ARM) < 1000
                        && tr(BI_NEED_ENCHANT_TO_A) != 0)
                        || (sval == iv.sv_scroll_enchant_weapon_to_hit
                            && tr(BI_AENCH_TOH) < 1000
                            && tr(BI_NEED_ENCHANT_TO_H) != 0)
                        || (sval == iv.sv_scroll_enchant_weapon_to_dam
                            && tr(BI_AENCH_TOD) < 1000
                            && tr(BI_NEED_ENCHANT_TO_D) != 0)
                        || sval == iv.sv_scroll_star_enchant_weapon
                        || sval == iv.sv_scroll_star_enchant_armor))
                    || (tval == TV_POTION
                        && (sval == iv.sv_potion_inc_str
                            || sval == iv.sv_potion_inc_int
                            || sval == iv.sv_potion_inc_wis
                            || sval == iv.sv_potion_inc_dex
                            || sval == iv.sv_potion_inc_con
                            || sval == iv.sv_potion_inc_all)))
        });
        if !used_at_once {
            // No encumbrance penalty for purchasing the items above;
            // C divides by `BI_CARRY / 10` unchecked (0 traps, Rust panics)
            let over = (tr(BI_WEIGHT) - tr(BI_CARRY) / 2) / (tr(BI_CARRY) / 10);
            *v = sub(*v, i64::from(over) * 1000);
        }
    }

    // Return the value
    value
}

/// Calculate the "power" of the Borg
pub fn borg_power(st: &mut BorgState) -> i32 {
    let mut value: i32 = 0;

    if st.cfg(BORG_USES_DYNAMIC_CALCS) != 0 {
        value = value.wrapping_add(borg_power_dynamic(st));
    } else {
        // Process the equipment
        value = value.wrapping_add(borg_power_equipment(st));

        // Process the inventory
        value = value.wrapping_add(borg_power_inventory(st));
    }

    // Add a bonus for deep level prep
    // Dump prep codes
    // Scan from surface to deep , stop when not preped
    // (`borg_prepared_c`, so `borg_prepared_buffer` is written as in C)
    let mut i = 1;
    while i <= st.borg.tr(BI_MAXDEPTH) + 50 {
        // Dump fear code
        if crate::borg_prepared::borg_prepared_c(st, i).is_some() {
            break;
        }
        i += 1;
    }
    value = add(value, i64::from(i - 1) * 40000);

    // Add the value for the swap items
    value = value.wrapping_add(st.trait_swap.weapon_swap_value);
    value = value.wrapping_add(st.trait_swap.armour_swap_value);

    // Return the value
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_and_int_adds() {
        assert_eq!(add(i32::MAX, 1), i32::MIN);
        assert_eq!(sub(0, 9999999), -9999999);
        assert_eq!(addi(i32::MAX, 1), i32::MIN);
        assert_eq!(subi(i32::MIN, 1), i32::MAX);
    }

    #[test]
    fn count_loops_carry_k() {
        let mut v = 0;
        let mut k = 0;
        count_up(&mut v, &mut k, 3, 10, 5);
        assert_eq!((v, k), (15, 3));
        count_up(&mut v, &mut k, 5, 4, 1);
        assert_eq!((v, k), (16, 4));
        count_down(&mut v, &mut k, 10, 6, 2);
        assert_eq!((v, k), (12, 6));
    }

    #[test]
    fn activation_chain_first_match() {
        let mut a = ActivationIds::default();
        // Unset ids are 0 and never match `act >= 1`.
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 0);
        a.mapping = 5;
        a.dragon_power = 5;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 550);
        a.mapping = 0;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 5), 5150);
        a.tele_other = 7;
        assert_eq!(activation_bonus(&a, CLASS_MAGE, 1, 7), 500);
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 1, 7), 1000);
        a.dispel_evil = 8;
        assert_eq!(activation_bonus(&a, CLASS_WARRIOR, 21, 8), 500 + 10 + 52);
    }
}

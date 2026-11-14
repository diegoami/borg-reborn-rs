//! Port of `borg-trait.c`: the calculations that determine what items and
//! abilities the Borg has (`borg.trait`, `borg.has`, `borg.activation`).
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
//! `borg_notice` is called in what-if loops all over the Borg (store, wear,
//! junk, power), so the order of its steps and C's `int` arithmetic are
//! kept exactly. It also writes the items it looks at, as C does: the
//! "net-zero armour" hack sets `to_a = -20`, and an empty or cursed bow or
//! weapon slot is zeroed ("bare hands").
//!
//! Several C bugs are kept (see `docs/c-bugs.md`): the racial fire
//! resistance sets `BI_RACID`, a light's radius counts twice, the slay
//! traits are overwritten by every later equipment slot, and the ammo
//! "brand" need is counted twice.

use core::ops::{Index, IndexMut};

use crate::borg_home_notice::borg_notice_home;
use crate::borg_item::{borg_item_weight, inven_wield, quiver_end, quiver_start};
use crate::borg_item_analyze::borg_obj_has_effect;
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_item_use::{borg_activate_failure, borg_equips_item, borg_equips_ring};
use crate::borg_item_wear::borg_is_ammo;
use crate::borg_magic::{
    borg_cheat_spells, borg_get_book_num, borg_spell_legal, borg_spell_legal_fail, borg_spell_stat,
};
use crate::borg_trait_swap::{borg_notice_armour_swap, borg_notice_weapon_swap};
use crate::borg_headers::borg_h::{
    BORG_ENCHANT_LIMIT, BORG_MUNCHKIN_LEVEL, BORG_MUNCHKIN_START, BORG_USES_SWAPS,
};
use crate::borg_headers::borg_item_h::*;
use crate::borg_headers::borg_magic_h::*;
use crate::borg_headers::borg_trait_h::*;
use crate::headers::list_effects_h::{EF_CRUNCH, EF_CURE, EF_NOURISH, EF_TIMED_INC};
use crate::headers::list_elements_h::{
    ELEM_ACID, ELEM_CHAOS, ELEM_COLD, ELEM_DARK, ELEM_DISEN, ELEM_ELEC, ELEM_FIRE, ELEM_LIGHT,
    ELEM_NETHER, ELEM_NEXUS, ELEM_POIS, ELEM_SHARD, ELEM_SOUND,
};
use crate::headers::list_mon_race_flags_h::{
    RF_ANIMAL, RF_DEMON, RF_DRAGON, RF_EVIL, RF_GIANT, RF_ORC, RF_TROLL, RF_UNDEAD,
};
use crate::headers::list_object_flags_h::*;
use crate::headers::list_object_modifiers_h::{
    OBJ_MOD_BLOWS, OBJ_MOD_DAM_RED, OBJ_MOD_INFRA, OBJ_MOD_LIGHT, OBJ_MOD_MIGHT, OBJ_MOD_MOVES,
    OBJ_MOD_SEARCH, OBJ_MOD_SHOTS, OBJ_MOD_SPEED, OBJ_MOD_STEALTH, OBJ_MOD_TUNNEL,
};
use crate::headers::list_options_h::OPT_birth_percent_damage;
use crate::headers::list_player_flags_h::{PF_BLESS_WEAPON, PF_BRAVERY_30, PF_FAST_SHOT};
use crate::headers::list_player_timed_h::*;
use crate::headers::list_stats_h::{
    OBJ_MOD_CON, OBJ_MOD_DEX, OBJ_MOD_INT, OBJ_MOD_STR, OBJ_MOD_WIS, STAT_CON, STAT_DEX,
    STAT_INT, STAT_STR, STAT_WIS,
};
use crate::headers::list_tvals_h::*;
use crate::headers::player_h::{
    SKILL_DEVICE, SKILL_DIGGING, SKILL_DISARM_MAGIC, SKILL_DISARM_PHYS, SKILL_SAVE,
    SKILL_SEARCH, SKILL_STEALTH, SKILL_TO_HIT_BOW, SKILL_TO_HIT_MELEE, SKILL_TO_HIT_THROW,
    STAT_MAX,
};
use crate::engine;
use crate::state::BorgState;

const CLASS_NECROMANCER: i32 = 4;

/// `BORG_DIG` (borg-flow.h): the digging skill below which a digger is
/// not worth carrying.
const BORG_DIG: i32 = 10;

/// The globals of borg-trait.c (`st.traits`).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TraitState {
    /// `borg_simulate`: simulation flag (attacks are only rated)
    pub simulate: bool,
    /// `borg_attacking`: simulation flag
    pub attacking: bool,
    /// `borg_on_glyph`: the Borg is standing on a glyph of warding
    pub on_glyph: bool,
    /// `borg_create_door`: the Borg is going to create doors
    pub create_door: bool,
    /// `borg_sleep_spell`
    pub sleep_spell: bool,
    /// `borg_sleep_spell_ii`
    pub sleep_spell_ii: bool,
    /// `borg_crush_spell`
    pub crush_spell: bool,
    /// `borg_slow_spell`: the Borg is about to cast the spell
    pub slow_spell: bool,
    /// `borg_confuse_spell`
    pub confuse_spell: bool,
    /// `borg_fear_mon_spell`
    pub fear_mon_spell: bool,
    /// `borg_game_ratio`: the ratio of Borg time to game time
    pub game_ratio: i16,
}

/// MAJOR HACK copied in because it is static in the main code
/// (`borg_adj_mag_mana`).
const BORG_ADJ_MAG_MANA: [i32; 38] = [
    0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200,
    225, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 800, 800, 800, 800,
];

/// `borg_adj_dex_ta`
const BORG_ADJ_DEX_TA: [i32; 38] = [
    -4, -3, -2, -1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 4, 5, 6, 7, 8, 9, 9,
    10, 11, 12, 13, 14, 15, 15, 15,
];

/// Stat Table (STR) -- bonus to dam (`borg_adj_str_td`)
pub const BORG_ADJ_STR_TD: [i32; 38] = [
    -2, -2, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 3, 3, 3, 3, 3, 4, 5, 5, 6, 7, 8, 9, 10,
    11, 12, 13, 14, 15, 16, 18, 20,
];

/// Stat Table (DEX) -- bonus to hit (`borg_adj_dex_th`)
pub const BORG_ADJ_DEX_TH: [i32; 38] = [
    -3, -2, -2, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 6, 7, 8, 9, 9,
    10, 11, 12, 13, 14, 15, 15, 15,
];

/// Stat Table (STR) -- bonus to hit (`borg_adj_str_th`)
const BORG_ADJ_STR_TH: [i32; 38] = [
    -3, -2, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 15, 15,
];

/// `borg_adj_dex_blow`
const BORG_ADJ_DEX_BLOW: [i32; 38] = [
    0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 9, 9,
    9, 10, 10, 11, 11, 11,
];

/// `borg_blows_table[P][D]`: energy per blow.
const BORG_BLOWS_TABLE: [[i32; 12]; 12] = [
    /* 0  */ [100, 100, 95, 85, 75, 60, 50, 42, 35, 30, 25, 23],
    /* 1  */ [100, 95, 85, 75, 60, 50, 42, 35, 30, 25, 23, 21],
    /* 2  */ [95, 85, 75, 60, 50, 42, 35, 30, 26, 23, 21, 20],
    /* 3  */ [85, 75, 60, 50, 42, 36, 32, 28, 25, 22, 20, 19],
    /* 4  */ [75, 60, 50, 42, 36, 33, 28, 25, 23, 21, 19, 18],
    /* 5  */ [60, 50, 42, 36, 33, 30, 27, 24, 22, 21, 19, 17],
    /* 6  */ [50, 42, 36, 33, 30, 27, 25, 23, 21, 20, 18, 17],
    /* 7  */ [42, 36, 33, 30, 28, 26, 24, 22, 20, 19, 18, 17],
    /* 8  */ [36, 33, 30, 28, 26, 24, 22, 21, 20, 19, 17, 16],
    /* 9  */ [35, 32, 29, 26, 24, 22, 21, 20, 19, 18, 17, 16],
    /* 10 */ [34, 30, 27, 25, 23, 22, 21, 20, 19, 18, 17, 16],
    /* 11+ */ [33, 29, 26, 24, 22, 21, 20, 19, 18, 17, 16, 15],
];

/// `borg_adj_dex_dis`
const BORG_ADJ_DEX_DIS: [i32; 38] = [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13,
    14, 15, 16, 17, 18, 19, 19, 19,
];

/// `borg_adj_int_dis`
const BORG_ADJ_INT_DIS: [i32; 38] = [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13,
    14, 15, 16, 17, 18, 19, 19, 19,
];

/// `borg_adj_int_dev`
const BORG_ADJ_INT_DEV: [i32; 38] = [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9,
    10, 10, 11, 11, 12, 13,
];

/// `borg_adj_str_dig`
const BORG_ADJ_STR_DIG: [i32; 38] = [
    0, 0, 1, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10, 12, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60,
    65, 70, 75, 80, 85, 90, 95, 100, 100, 100,
];

/// `borg_adj_wis_sav`
const BORG_ADJ_WIS_SAV: [i32; 38] = [
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15, 16, 17, 18, 19,
];

/// `borg_adj_str_wgt`
const BORG_ADJ_STR_WGT: [i32; 38] = [
    5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 30, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];

/// `borg_adj_con_mhp`
const BORG_ADJ_CON_MHP: [i32; 38] = [
    -250, -150, -100, -75, -50, -25, -10, -5, 0, 5, 10, 25, 50, 75, 100, 150, 175, 200, 225, 250,
    275, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 900, 1000, 1100, 1250, 1250, 1250,
];

/// Stat Table (INT/WIS) -- Minimum failure rate (percentage)
/// (`borg_adj_mag_fail`)
const BORG_ADJ_MAG_FAIL: [i32; 38] = [
    99, 99, 99, 99, 99, 50, 30, 20, 15, 12, 11, 10, 9, 8, 7, 6, 6, 5, 5, 5, 4, 4, 4, 4, 3, 3, 2, 2,
    2, 2, 1, 1, 1, 1, 1, 0, 0, 0,
];

/// Stat Table (INT/WIS) -- failure rate adjustment (`borg_adj_mag_stat`)
const BORG_ADJ_MAG_STAT: [i32; 38] = [
    -5, -4, -3, -3, -2, -1, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15, 18, 21, 24,
    27, 30, 33, 36, 39, 42, 45, 48, 51, 54, 57,
];

/// The names of the `BI_*` traits (`prefix_pref`, without C's NULL end
/// entry). NOTE: This must exactly match the enum in borg-trait.h.
pub const PREFIX_PREF: [&[u8]; BI_MAX as usize] = [
    // personal attributes
    b"str",
    b"int",
    b"wis",
    b"dex",
    b"con",
    b"str adj",
    b"int adj",
    b"wis adj",
    b"dex adj",
    b"con adj",
    b"cur str",
    b"cur int",
    b"cur wis",
    b"cur dex",
    b"cur con",
    b"str index",
    b"int index",
    b"wis index",
    b"dex index",
    b"con index",
    b"sust str",
    b"sust int",
    b"sust wis",
    b"sust dex",
    b"sust con",
    b"class",
    b"light",
    b"cur hp",
    b"max hp",
    b"hp adj",
    b"cur sp",
    b"max sp",
    b"sp adj",
    b"SFAIL1",
    b"SFAIL2",
    b"clevel",
    b"max clevel",
    b"esp",
    b"recall",
    b"food",
    b"food high",
    b"food low",
    b"food cure conf",
    b"food cure blind",
    b"speed",
    b"gold",
    b"extra moves",
    b"damage reduction",
    b"slow dig",
    b"feather fall",
    b"regen",
    b"see inv",
    b"infravision",
    b"fast shots",
    b"disarm ph",
    b"disarm mag",
    b"use device",
    b"save",
    b"stealth",
    b"search",
    b"to hit normal",
    b"to hit bow",
    b"to hit throw",
    b"dig",
    b"immune fire",
    b"immune acid",
    b"immune cold",
    b"immune elec",
    b"immune poison",
    b"resist fire",
    b"resist cold",
    b"resist elec",
    b"resist acid",
    b"resist poison",
    b"resist fear",
    b"resist lite",
    b"resist dark",
    b"resist blind",
    b"resist conf",
    b"resist sound",
    b"resist shards",
    b"resist nexus",
    b"resist nether",
    b"resist chaos",
    b"resist dis",
    b"hold life",
    b"free action",
    b"resist fire with swap",
    b"resist cold with swap",
    b"resist elec with swap",
    b"resist acid with swap",
    b"resist poison with swap",
    b"resist fear with swap",
    b"resist lite with swap",
    b"resist dark with swap",
    b"resist blind with swap",
    b"resist conf with swap",
    b"resist sound with swap",
    b"resist shards with swap",
    b"resist nexus with swap",
    b"resist nether with swap",
    b"resist chaos with swap",
    b"resist dis with swap",
    b"hold life with swap",
    b"free action with swap",
    // random extra variable
    b"depth",
    b"max depth",
    b"king",
    // player state things
    b"is weak",
    b"is hungry",
    b"is full",
    b"is gorged",
    b"is blind",
    b"is afraid",
    b"is confused",
    b"is poisoned",
    b"is cut",
    b"is stun",
    b"is heavystun",
    b"is paralyzed",
    b"is image",
    b"is forget",
    b"is encumb",
    b"is study",
    b"is fixlev",
    b"is fixexp",
    b"has fixexp",
    b"is fixstr",
    b"is fixint",
    b"is fixwis",
    b"is fixdex",
    b"is fixcon",
    b"is fixall",
    // some combat stuff
    b"armor",
    b"to hit",
    b"to damage",
    b"wep to hit",
    b"wep to damage",
    b"wep id",
    b"wep damage dice",
    b"wep damage sides",
    b"bow id",
    b"bow to hit",
    b"bow to damage",
    b"bow is sling",
    b"bow artifact",
    b"blows",
    b"EXTRA_BLOWS",
    b"shots",
    b"heavy weapon",
    b"heavy bow",
    b"ammo count",
    b"ammo tval",
    b"ammo sides",
    b"ammo power",
    b"amt missiles",
    b"amt ego missiles",
    b"amt cursed missiles",
    b"quiver slots",
    b"first cursed",
    b"where cursed",
    // curses
    b"enveloping",
    b"irritation",
    b"teleport",
    b"curse poison",
    b"siren",
    b"hallucinate",
    b"paralysis",
    b"summon demon",
    b"summon dragon",
    b"summon undead",
    b"curse stone",
    b"no teleport",
    b"treach wep",
    b"aggravate",
    b"vulnerable",
    b"dullness",
    b"sickness",
    b"weakness",
    b"clumsiness",
    b"slowness",
    b"annoyance",
    b"impair hp",
    b"CRSMPIMP",
    b"curse steel",
    b"air swing",
    b"fear",
    b"drain xp",
    b"vuln fire",
    b"vuln elec",
    b"vuln cold",
    b"vuln acid",
    b"unknown curse",
    // weapon attributes
    b"wep slay animal",
    b"wep slay evil",
    b"wep slay undead",
    b"wep slay demon",
    b"wep slay orc",
    b"wep slay troll",
    b"wep slay giant",
    b"wep slay dragon",
    b"wep kill undead",
    b"wep kill demon",
    b"wep kill dragon",
    b"wep impact",
    b"wep brand acid",
    b"wep brand elec",
    b"wep brand fire",
    b"wep brand cold",
    b"wep brand poison",
    // amounts
    b"amt phase",
    b"amt teleport",
    b"amt escape",
    b"fuel",
    b"amt heal",
    b"amt ezheal",
    b"amt life",
    b"amt id",
    b"amt speed",
    b"amt staff magi",
    b"amt staff destruction",
    b"amt teleport other",
    b"amt cure poison",
    b"amt detect traps",
    b"amt detect door",
    b"amt detect evil",
    b"amt magic map",
    b"amt recharge",
    b"amt call lite",
    b"amt prot evil",
    b"amt glyph",
    b"amt potion ccw",
    b"amt potion csw",
    b"amt potion clw",
    b"amt ench to hit",
    b"amt ench to dam",
    b"amt *ench to wep*",
    b"amt ench to armor",
    b"amt *ench to armor*",
    b"amt brand",
    b"need ench to armor",
    b"need ench to hit",
    b"need ench to dam",
    b"need brand",
    b"amt resist heat",
    b"amt resist cold",
    b"amt resist poison",
    b"amt teleport level",
    b"holy word",
    b"mass banishment",
    b"amt cool shroom",
    b"amt attack rods1",
    b"amt attack rods2",
    b"worn need id",
    b"amt need id",
    b"amt diggers",
    b"amt good staff chg",
    b"amt good wand chg",
    b"multi bonus",
    b"detect inv",
    b"weight",
    b"carry",
    b"empty slots",
    b"sauron dead",
    b"prep big fight",
];

/// The name of trait `i` (`prefix_pref[i]`); panics for `i >= BI_MAX`
/// (C's NULL end entry).
pub fn trait_name(i: usize) -> &'static [u8] {
    PREFIX_PREF[i]
}

/// `borg.trait` indexed by the `BI_*` constants.
struct Traits<'a>(&'a mut [i32]);

impl Index<u32> for Traits<'_> {
    type Output = i32;
    fn index(&self, i: u32) -> &i32 {
        &self.0[i as usize]
    }
}

impl IndexMut<u32> for Traits<'_> {
    fn index_mut(&mut self, i: u32) -> &mut i32 {
        &mut self.0[i as usize]
    }
}

fn tr(st: &mut BorgState) -> Traits<'_> {
    Traits(&mut st.borg.trait_)
}

/// `borg.trait[i]` (read).
fn t(st: &BorgState, i: u32) -> i32 {
    st.borg.trait_[i as usize]
}

/// `of_has(flags, flag)`
fn of_has(flags: &[u8], flag: u32) -> bool {
    engine::flag_has(flags, flag as i32)
}

/// `borg_equips_item(act_<name>, false)`
macro_rules! equips {
    ($st:expr, $act:ident) => {{
        let a = $st.act.$act;
        borg_equips_item($st, a, false)
    }};
}

/// Helper to turn off swap items when deeper than depth 90
/// (`borg_uses_swaps`).
pub fn borg_uses_swaps(st: &BorgState) -> bool {
    st.core.cfg[BORG_USES_SWAPS as usize] != 0 && t(st, BI_MAXDEPTH) < 90
}

/// Calculate the blows a player would get (`borg_calc_blows`).
///
/// copied and adjusted from player-calcs.c
pub fn borg_calc_blows(st: &BorgState, item: &borg_item) -> i32 {
    let class = engine::player_class_info();

    /* The engine weighs one weapon (object_weight_one): blows per
     * weapon must not depend on how many weapons are stacked. */
    let weight = i32::from(item.weight);
    let min_weight = class.min_weight;

    /* Enforce a minimum "weight" (tenth pounds) */
    let div = if weight < min_weight { min_weight } else { weight };

    /* Get the strength vs weight */
    let mut str_index =
        engine::adj_str_blow(t(st, BI_STR_INDEX) as usize) * class.att_multiply / div;

    /* Maximal value */
    if str_index > 11 {
        str_index = 11;
    }

    /* Index by dexterity */
    let dex_index = BORG_ADJ_DEX_BLOW[t(st, BI_DEX_INDEX) as usize].min(11);

    /* Use the blows table to get energy per blow */
    let blow_energy = BORG_BLOWS_TABLE[str_index as usize][dex_index as usize];

    let blows = (10000 / blow_energy).min(100 * class.max_attacks);

    /* Require at least one blow, two for O-combat */
    let min = if engine::player_opt(OPT_birth_percent_damage) { 200 } else { 100 };
    (blows + 100 * (i32::from(item.modifiers[OBJ_MOD_BLOWS as usize]) + t(st, BI_EXTRA_BLOWS)))
        .max(min)
        / 100
}

/*
 * Note that we assume that any item with quantity zero does not exist,
 * thus, when simulating possible worlds, we do not actually have to
 * "optimize" empty slots.
 *
 * The "notice" functions examine various aspects of the player inventory,
 * the player equipment, or the home contents, and extract various numerical
 * quantities based on those aspects, adjusting them for various "abilities",
 * such as the ability to cast certain spells, etc.
 */

/// Helper function -- notice one slot of ammo (`borg_notice_ammo`).
fn borg_notice_ammo(st: &mut BorgState, slot: i32) {
    let item = st.items.items[slot as usize];

    /* Skip empty items */
    if item.iqty == 0 {
        return;
    }

    let iqty = i32::from(item.iqty);
    {
        let mut t = tr(st);

        /* number of inventory slots the quiver used  */
        if slot >= quiver_start() {
            t[BI_QUIVER_SLOTS] += 1;
        }

        /* total up the weight of the items */
        t[BI_WEIGHT] += i32::from(borg_item_weight(&item));

        /* Count all ammo */
        t[BI_AMMO_COUNT] += iqty;

        if i32::from(item.tval) != t[BI_AMMO_TVAL] {
            return;
        }

        /* Count missiles that fit your bow */
        t[BI_AMISSILES] += iqty;

        /* track first uncursable item */
        if item.uncursable {
            t[BI_WHERE_CURSED] |= BORG_QUILL as i32;
            if t[BI_FIRST_CURSED] == 0 {
                t[BI_FIRST_CURSED] = slot + 1;
            }

            t[BI_AMISSILES_CURSED] += iqty;
            return;
        }

        if item.ego_idx != 0 {
            t[BI_AMISSILES_SPECIAL] += iqty;
        }

        /* Ignore worthless missiles */
        if item.value <= 0 {
            return;
        }
    }

    /* Only enchant ammo if we have a good shooter,
     * otherwise, store the enchants in the home.
     */
    if t(st, BI_AMMO_POWER) >= 3 {
        if (equips!(st, firebrand) || borg_spell_legal_fail(st, BRAND_AMMUNITION, 65))
            && item.iqty >= 5
            /* Skip artifacts and ego-items */
            && item.ego_idx == 0
            && item.art_idx == 0
            && item.ident
            && i32::from(item.tval) == t(st, BI_AMMO_TVAL)
        {
            tr(st)[BI_NEED_BRAND_WEAPON] += 10;
        }

        /* if we have loads of cash (as we will at level 35),  */
        /* enchant missiles */
        if t(st, BI_CLEVEL) > 35 {
            let limit = if borg_spell_legal_fail(st, ENCHANT_WEAPON, 65) && item.iqty >= 5 {
                10
            } else {
                8
            };
            let mut t = tr(st);
            if i32::from(item.to_h) < limit {
                t[BI_NEED_ENCHANT_TO_H] += limit - i32::from(item.to_h);
            }

            if i32::from(item.to_d) < limit {
                t[BI_NEED_ENCHANT_TO_D] += limit - i32::from(item.to_d);
            }
        }
    } /* Ammo Power > 3 */
}

/// don't give credit for perm-cursed non-artifacts (`cursed_nonartifact`)
fn cursed_nonartifact(item: &borg_item) -> bool {
    if item.iqty == 0 {
        return false;
    }

    if item.art_idx != 0 {
        return false;
    }

    if item.uncursable {
        return false;
    }

    item.cursed
}

/// The skill of the race plus the class (`rb_ptr->r_skills[s] +
/// cb_ptr->c_skills[s]`).
fn base_skill(s: u32) -> i32 {
    engine::player_race_info().r_skills[s as usize] + engine::player_class_info().c_skills[s as usize]
}

/// `cb_ptr->x_skills[s] * borg.trait[BI_MAXCLEVEL] / 10`
fn level_skill(st: &BorgState, s: u32) -> i32 {
    engine::player_class_info().x_skills[s as usize] * t(st, BI_MAXCLEVEL) / 10
}

/// Helper function -- notice the player equipment (`borg_notice_equipment`).
fn borg_notice_equipment(st: &mut BorgState) {
    let rb = engine::player_race_info();
    let cb = engine::player_class_info();
    let p = engine::player_notice();

    let mut extra_shots = 0i32;
    let mut extra_might = 0i32;

    /* Start with a single shot per turn */
    let mut my_num_fire = 1i32;

    let res = |e: u32| i32::from(rb.el_info[e as usize].res_level);

    {
        let mut t = tr(st);

        /* Base infravision (purely racial) */
        t[BI_INFRA] = rb.infra;

        /* Base skill -- disarming */
        t[BI_DISP] = base_skill(SKILL_DISARM_PHYS);
        t[BI_DISM] = base_skill(SKILL_DISARM_MAGIC);

        /* Base skill -- magic devices */
        t[BI_DEV] = base_skill(SKILL_DEVICE);

        /* Base skill -- saving throw */
        t[BI_SAV] = base_skill(SKILL_SAVE);

        /* Base skill -- stealth */
        t[BI_STL] = base_skill(SKILL_STEALTH);

        /* Base skill -- searching ability */
        t[BI_SRCH] = base_skill(SKILL_SEARCH);

        /* Base skill -- combat (normal) */
        t[BI_THN] = base_skill(SKILL_TO_HIT_MELEE);

        /* Base skill -- combat (shooting) */
        t[BI_THB] = base_skill(SKILL_TO_HIT_BOW);

        /* Base skill -- combat (throwing) */
        t[BI_THT] = base_skill(SKILL_TO_HIT_THROW);

        /* Affect Skill -- digging (STR) */
        t[BI_DIG] = base_skill(SKILL_DIGGING);

        /* Racial Skills */

        /* Extract the player flags */
        let f = engine::player_flags();

        /* Good flags */
        for (flag, bi) in [
            (OF_SLOW_DIGEST, BI_SDIG),
            (OF_FEATHER, BI_FEATH),
            (OF_REGEN, BI_REG),
            (OF_TELEPATHY, BI_ESP),
            (OF_SEE_INVIS, BI_SINV),
            (OF_FREE_ACT, BI_FRACT),
            (OF_HOLD_LIFE, BI_HLIFE),
            /* Bad flags */
            (OF_IMPACT, BI_W_IMPACT),
            (OF_AGGRAVATE, BI_CRSAGRV),
            (OF_AFRAID, BI_CRSFEAR),
            (OF_DRAIN_EXP, BI_CRSDRAIN_XP),
        ] {
            if of_has(&f, flag) {
                t[bi] = 1;
            }
        }

        if res(ELEM_FIRE) == -1 {
            t[BI_CRSFVULN] = 1;
        }
        if res(ELEM_ACID) == -1 {
            t[BI_CRSAVULN] = 1;
        }
        if res(ELEM_COLD) == -1 {
            t[BI_CRSCVULN] = 1;
        }
        if res(ELEM_ELEC) == -1 {
            t[BI_CRSEVULN] = 1;
        }

        /* Immunity flags */
        if res(ELEM_FIRE) == 3 {
            t[BI_IFIRE] = 1;
        }
        if res(ELEM_ACID) == 3 {
            t[BI_IACID] = 1;
        }
        if res(ELEM_COLD) == 3 {
            t[BI_ICOLD] = 1;
        }
        if res(ELEM_ELEC) == 3 {
            t[BI_IELEC] = 1;
        }

        /* Resistance flags (the racial fire resistance sets BI_RACID,
         * C bug 48) */
        for (elem, bi) in [
            (ELEM_FIRE, BI_RACID),
            (ELEM_ELEC, BI_RELEC),
            (ELEM_FIRE, BI_RFIRE),
            (ELEM_COLD, BI_RCOLD),
            (ELEM_POIS, BI_RPOIS),
            (ELEM_LIGHT, BI_RLITE),
            (ELEM_DARK, BI_RDARK),
            (ELEM_SOUND, BI_RSND),
            (ELEM_SHARD, BI_RSHRD),
            (ELEM_NEXUS, BI_RNXUS),
            (ELEM_NETHER, BI_RNTHR),
            (ELEM_CHAOS, BI_RKAOS),
            (ELEM_DISEN, BI_RDIS),
        ] {
            if res(elem) > 0 {
                t[bi] = 1;
            }
        }
        /* C tests these with `rf_has` (size RF_SIZE) on the OF_SIZE
         * array; the object flags are below 8 * OF_SIZE, so it reads the
         * same byte as `of_has`. */
        for (flag, bi) in [
            (OF_PROT_FEAR, BI_RFEAR),
            (OF_PROT_BLIND, BI_RBLIND),
            (OF_PROT_CONF, BI_RCONF),
            /* Sustain flags */
            (OF_SUST_STR, BI_SSTR),
            (OF_SUST_INT, BI_SINT),
            (OF_SUST_WIS, BI_SWIS),
            (OF_SUST_DEX, BI_SDEX),
            (OF_SUST_CON, BI_SCON),
        ] {
            if of_has(&f, flag) {
                t[bi] = 1;
            }
        }

        /* if hasting */
        if engine::player_timed(TMD_FAST) != 0 || engine::player_timed(TMD_SPRINT) != 0 {
            t[BI_SPEED] += 10;
        } else if engine::player_timed(TMD_TERROR) != 0 {
            t[BI_SPEED] += 5;
        }
    }

    /* track activations */
    /* note this is done first so that it we can use this */
    /* array in borg_equips_item */
    let wield = inven_wield();
    let total = wield + 12;
    for i in wield..total {
        let a = st.items.items[i as usize].activ_idx;
        if a != 0 {
            st.borg.activation[a as usize] += 1;
        }
    }

    if st.borg.activation[st.act.staff_magi as usize] != 0 {
        tr(st)[BI_ASTFMAGI] += 10;
    }

    /* Scan the usable inventory */
    let class = t(st, BI_CLASS);
    for i in wield..total {
        // A pure read of the note (it may point into another slot's
        // `desc`, as C's pointer does), taken before the field borrows;
        // nothing below changes an item before C's test.
        let needs_id = borg_item_note_needs_id(st, &st.items.items[i as usize]);
        let BorgState { borg, items, .. } = &mut *st;
        let item = &mut items.items[i as usize];
        let mut t = Traits(&mut borg.trait_);

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* track first uncursable item */
        if item.uncursable {
            t[BI_WHERE_CURSED] |= BORG_EQUIP as i32;
            if t[BI_FIRST_CURSED] == 0 {
                t[BI_FIRST_CURSED] = i + 1;
            }
        }

        /* skip cursed non-artifacts */
        if cursed_nonartifact(item) {
            continue;
        }

        /* total up the weight of the items */
        t[BI_WEIGHT] += i32::from(borg_item_weight(item));

        if needs_id {
            t[BI_ALL_NEED_ID] += 1;
            t[BI_WORN_NEED_ID] += 1;
        }

        let m = |k: u32| i32::from(item.modifiers[k as usize]);
        let known = |k: u32| i32::from(p.obj_k_modifiers[k as usize]);

        /* Affect stats */
        t[BI_ASTR] += m(OBJ_MOD_STR) * known(OBJ_MOD_STR);
        t[BI_AINT] += m(OBJ_MOD_INT) * known(OBJ_MOD_INT);
        t[BI_AWIS] += m(OBJ_MOD_WIS) * known(OBJ_MOD_WIS);
        t[BI_ADEX] += m(OBJ_MOD_DEX) * known(OBJ_MOD_DEX);
        t[BI_ACON] += m(OBJ_MOD_CON) * known(OBJ_MOD_CON);

        /* various slays: accumulate across slots.  Assigning them per
         * slot made the last occupied slot (usually the light or boots,
         * with no slays) erase the wielded weapon's slays. */
        t[BI_WS_ANIMAL] |= item.slays[RF_ANIMAL as usize];
        t[BI_WS_EVIL] |= item.slays[RF_EVIL as usize];
        t[BI_WS_UNDEAD] |= item.slays[RF_UNDEAD as usize];
        t[BI_WS_DEMON] |= item.slays[RF_DEMON as usize];
        t[BI_WS_ORC] |= item.slays[RF_ORC as usize];
        t[BI_WS_TROLL] |= item.slays[RF_TROLL as usize];
        t[BI_WS_GIANT] |= item.slays[RF_GIANT as usize];
        t[BI_WS_DRAGON] |= item.slays[RF_DRAGON as usize];

        /* the greater, race-destroying slays are stored under the same
         * race flag with the largest multiplier */
        if item.slays[RF_UNDEAD as usize] >= 5 {
            t[BI_WK_UNDEAD] = 1;
        }
        if item.slays[RF_DEMON as usize] >= 5 {
            t[BI_WK_DEMON] = 1;
        }
        if item.slays[RF_DRAGON as usize] >= 5 {
            t[BI_WK_DRAGON] = 1;
        }

        /* various brands */
        for (elem, bi) in [
            (ELEM_ACID, BI_WB_ACID),
            (ELEM_ELEC, BI_WB_ELEC),
            (ELEM_FIRE, BI_WB_FIRE),
            (ELEM_COLD, BI_WB_COLD),
            (ELEM_POIS, BI_WB_POIS),
        ] {
            if item.brands[elem as usize] {
                t[bi] = 1;
            }
        }
        if of_has(&item.flags, OF_IMPACT) {
            t[BI_W_IMPACT] = 1;
        }

        /* Affect infravision */
        t[BI_INFRA] += m(OBJ_MOD_INFRA);

        /* Affect stealth */
        t[BI_STL] += m(OBJ_MOD_STEALTH);

        /* Affect searching ability (factor of five) */
        t[BI_SRCH] += m(OBJ_MOD_SEARCH) * 5;

        /* weapons of digging type get a special bonus */
        let mut dig = 0;
        if u32::from(item.tval) == TV_DIGGING {
            if of_has(&item.flags, OF_DIG_1) {
                dig = 1;
            } else if of_has(&item.flags, OF_DIG_2) {
                dig = 2;
            } else if of_has(&item.flags, OF_DIG_3) {
                dig = 3;
            }
        }
        dig += m(OBJ_MOD_TUNNEL);

        /* Affect digging (factor of 20) */
        t[BI_DIG] += dig * 20;

        /* Affect speed */
        t[BI_SPEED] += m(OBJ_MOD_SPEED);

        /* Affect blows (not from primary weapon) */
        if i != wield {
            t[BI_EXTRA_BLOWS] += m(OBJ_MOD_BLOWS);
        }

        /* Boost shots */
        extra_shots += m(OBJ_MOD_SHOTS);

        /* Boost might */
        extra_might += m(OBJ_MOD_MIGHT);

        if i != wield + 5 || of_has(&item.flags, OF_NO_FUEL) || item.timeout != 0 {
            /* Item makes player glow or has a light radius  */
            t[BI_LIGHT] += m(OBJ_MOD_LIGHT);

            /* LIGHT_2 and LIGHT_3 */
            if of_has(&item.flags, OF_LIGHT_2) {
                t[BI_LIGHT] += 2;
            } else if of_has(&item.flags, OF_LIGHT_3) {
                t[BI_LIGHT] += 3;
            }

            /* people with "unlight" can use radius 1 light artifacts */
            if m(OBJ_MOD_LIGHT) > 0 && class == CLASS_NECROMANCER {
                t[BI_LIGHT] -= 1;
            }
        }

        /* Boost mod moves */
        t[BI_MOD_MOVES] += m(OBJ_MOD_MOVES);

        /* Boost damage reduction */
        t[BI_DAM_RED] += m(OBJ_MOD_DAM_RED);

        /* Various flags */
        for (flag, bi) in [
            (OF_SLOW_DIGEST, BI_SDIG),
            (OF_AGGRAVATE, BI_CRSAGRV),
            (OF_IMPAIR_HP, BI_CRSHPIMP),
            (OF_IMPAIR_MANA, BI_CRSMPIMP),
            (OF_AFRAID, BI_CRSFEAR),
            (OF_DRAIN_EXP, BI_CRSDRAIN_XP),
        ] {
            if of_has(&item.flags, flag) {
                t[bi] = 1;
            }
        }

        /* curses that don't have flags or stat changes that are tracked
         * elsewhere */
        let c = |k: u32| item.curses[k as usize];
        if c(BORG_CURSE_VULNERABILITY) {
            t[BI_CRSVULN] = 1;
        }
        if c(BORG_CURSE_TELEPORTATION) {
            t[BI_CRSTELE] = 1;
        }
        if c(BORG_CURSE_DULLNESS) {
            t[BI_CRSDULL] = 1;
        }
        if c(BORG_CURSE_SICKLINESS) {
            t[BI_CRSSICK] = 1;
        }
        if c(BORG_CURSE_ENVELOPING) {
            t[BI_CRSENVELOPING] = 1;
        }
        if c(BORG_CURSE_IRRITATION) {
            t[BI_CRSAGRV] = 1;
            t[BI_CRSIRRITATION] = 1;
        }
        if c(BORG_CURSE_WEAKNESS) {
            t[BI_CRSWEAK] = 1;
        }
        if c(BORG_CURSE_CLUMSINESS) {
            t[BI_CRSCLUM] = 1;
        }
        if c(BORG_CURSE_SLOWNESS) {
            t[BI_CRSSLOW] = 1;
        }
        if c(BORG_CURSE_ANNOYANCE) {
            t[BI_CRSANNOY] = 1;
        }
        if c(BORG_CURSE_POISON) {
            t[BI_CRSPOIS] = 1;
        }
        if c(BORG_CURSE_SIREN) {
            t[BI_CRSSIREN] = 1;
        }
        if c(BORG_CURSE_HALLUCINATION) {
            t[BI_CRSHALU] = 1;
        }
        if c(BORG_CURSE_PARALYSIS) {
            t[BI_CRSPARA] = 1;
        }
        if c(BORG_CURSE_DEMON_SUMMON) {
            t[BI_CRSSDEM] = 1;
        }
        if c(BORG_CURSE_DRAGON_SUMMON) {
            t[BI_CRSSDRA] = 1;
        }
        if c(BORG_CURSE_UNDEAD_SUMMON) {
            t[BI_CRSSUND] = 1;
        }
        if c(BORG_CURSE_IMPAIR_MANA_RECOVERY) {
            t[BI_CRSMPIMP] = 1;
        }
        if c(BORG_CURSE_IMPAIR_HITPOINT_RECOVERY) {
            t[BI_CRSHPIMP] = 1;
        }
        if c(BORG_CURSE_COWARDICE) {
            t[BI_CRSFEAR] = 1;
        }
        if c(BORG_CURSE_STONE) {
            t[BI_CRSSTONE] = 1;
        }
        if c(BORG_CURSE_ANTI_TELEPORTATION) {
            t[BI_CRSNOTEL] = 1;
        }
        if c(BORG_CURSE_TREACHEROUS_WEAPON) {
            t[BI_CRSTWEP] = 1;
        }
        if c(BORG_CURSE_BURNING_UP) {
            t[BI_CRSFVULN] = 1;
            t[BI_RCOLD] = 1;
        }
        if c(BORG_CURSE_CHILLED_TO_THE_BONE) {
            t[BI_CRSCVULN] = 1;
            t[BI_RFIRE] = 1;
        }
        if c(BORG_CURSE_STEELSKIN) {
            t[BI_CRSSTEELSKIN] = 1;
        }
        if c(BORG_CURSE_AIR_SWING) {
            t[BI_CRSAIRSWING] = 1;
        }
        if c(BORG_CURSE_UNKNOWN) {
            t[BI_CRSUNKNO] = 1;
        }

        let r = |e: u32| i32::from(item.el_info[e as usize].res_level);
        if r(ELEM_FIRE) == -1 {
            t[BI_CRSFVULN] = 1;
        }
        if r(ELEM_ACID) == -1 {
            t[BI_CRSAVULN] = 1;
        }
        if r(ELEM_COLD) == -1 {
            t[BI_CRSCVULN] = 1;
        }
        if r(ELEM_ELEC) == -1 {
            t[BI_CRSEVULN] = 1;
        }

        for (flag, bi) in [
            (OF_REGEN, BI_REG),
            (OF_TELEPATHY, BI_ESP),
            (OF_SEE_INVIS, BI_SINV),
            (OF_FEATHER, BI_FEATH),
            (OF_FREE_ACT, BI_FRACT),
            (OF_HOLD_LIFE, BI_HLIFE),
            (OF_PROT_CONF, BI_RCONF),
            (OF_PROT_BLIND, BI_RBLIND),
        ] {
            if of_has(&item.flags, flag) {
                t[bi] = 1;
            }
        }

        /* Immunity flags */
        /* if you are immune you automatically resist */
        if r(ELEM_FIRE) == 3 {
            t[BI_IFIRE] = 1;
            t[BI_RFIRE] = 1;
            borg.temp.res_fire = true;
        }
        if r(ELEM_ACID) == 3 {
            t[BI_IACID] = 1;
            t[BI_RACID] = 1;
            borg.temp.res_acid = true;
        }
        if r(ELEM_COLD) == 3 {
            t[BI_ICOLD] = 1;
            t[BI_RCOLD] = 1;
            borg.temp.res_cold = true;
        }
        if r(ELEM_ELEC) == 3 {
            t[BI_IELEC] = 1;
            t[BI_RELEC] = 1;
            borg.temp.res_elec = true;
        }

        /* Resistance flags */
        for (elem, bi) in [
            (ELEM_ACID, BI_RACID),
            (ELEM_ELEC, BI_RELEC),
            (ELEM_FIRE, BI_RFIRE),
            (ELEM_COLD, BI_RCOLD),
            (ELEM_POIS, BI_RPOIS),
            (ELEM_SOUND, BI_RSND),
            (ELEM_LIGHT, BI_RLITE),
            (ELEM_DARK, BI_RDARK),
            (ELEM_CHAOS, BI_RKAOS),
            (ELEM_DISEN, BI_RDIS),
            (ELEM_SHARD, BI_RSHRD),
            (ELEM_NEXUS, BI_RNXUS),
            (ELEM_NETHER, BI_RNTHR),
        ] {
            if r(elem) > 0 {
                t[bi] = 1;
            }
        }

        /* Sustain flags */
        for (flag, bi) in [
            (OF_SUST_STR, BI_SSTR),
            (OF_SUST_INT, BI_SINT),
            (OF_SUST_WIS, BI_SWIS),
            (OF_SUST_DEX, BI_SDEX),
            (OF_SUST_CON, BI_SCON),
        ] {
            if of_has(&item.flags, flag) {
                t[bi] = 1;
            }
        }

        /* Good to have one item with multiple high resists */
        let high = [ELEM_POIS, ELEM_SOUND, ELEM_SHARD, ELEM_NEXUS, ELEM_NETHER, ELEM_CHAOS, ELEM_DISEN]
            .into_iter()
            .filter(|&e| r(e) > 0)
            .count() as i32;
        /* resist base 4 */
        let base4 = r(ELEM_FIRE) > 0 && r(ELEM_COLD) > 0 && r(ELEM_ELEC) > 0 && r(ELEM_ACID) > 0;
        /* sustains all stats  */
        let sust_all = [OF_SUST_STR, OF_SUST_INT, OF_SUST_WIS, OF_SUST_DEX, OF_SUST_CON]
            .into_iter()
            .all(|f| of_has(&item.flags, f));
        let bonuses = high + i32::from(base4) + i32::from(sust_all);

        if bonuses > 2 {
            t[BI_MULTIPLE_BONUSES] += bonuses;
        }

        /* HACK: Net-zero The borg will miss read acid damaged items such as
         * Leather Gloves [2,-2] and falsely assume they help his power.
         * this hack rewrites the bonus to an extremely negative value
         * thus encouraging him to remove the non-helpful-non-harmful but
         * heavy-none-the-less item.
         */
        if (item.art_idx == 0 && item.ego_idx == 0)
            && item.ac >= 1
            && i32::from(item.to_a) + i32::from(item.ac) <= 0
        {
            item.to_a = -20;
        }

        /* Modify the base armor class */
        t[BI_ARMOR] += i32::from(item.ac);

        /* Apply the bonuses to armor class */
        t[BI_ARMOR] += i32::from(item.to_a);

        /* Do not apply "weapon" bonuses */
        if i == wield {
            continue;
        }

        /* Do not apply "bow" bonuses */
        if i == wield + 1 {
            continue;
        }

        /* Apply the bonuses to hit/damage */
        t[BI_TOHIT] += i32::from(item.to_h);
        t[BI_TODAM] += i32::from(item.to_d);
    }

    {
        let mut t = tr(st);

        /* if the player has unlight count them as having light if they have none */
        if t[BI_CLASS] == CLASS_NECROMANCER && t[BI_LIGHT] <= 0 {
            t[BI_LIGHT] = 1;
        }

        if t[BI_CRSVULN] != 0 {
            t[BI_CRSAGRV] = 1;
            t[BI_ARMOR] -= 50;
        }
        if t[BI_CRSDULL] != 0 {
            t[BI_CINT] -= 5;
            t[BI_CWIS] -= 5;
        }
        if t[BI_CRSSICK] != 0 {
            t[BI_CSTR] -= 5;
            t[BI_CDEX] -= 5;
            t[BI_CCON] -= 5;
        }
        if t[BI_CRSWEAK] != 0 {
            t[BI_CSTR] -= 10;
        }
        if t[BI_CRSCLUM] != 0 {
            t[BI_CDEX] -= 10;
        }
        if t[BI_CRSSLOW] != 0 {
            t[BI_SPEED] -= 5;
        }
        if t[BI_CRSANNOY] != 0 {
            t[BI_SPEED] -= 10;
            t[BI_STL] -= 10;
            t[BI_CRSAGRV] = 1;
        }

        /* The borg needs to update his base stat points */
        /* Cheat the exact number from the game.  This number is available to
         * the player on the extra term window.
         */
        let stat_cur = p.stat_cur;

        /* Update "stats" */
        for i in 0..STAT_MAX {
            let mut add = t[BI_ASTR + i];

            /* Modify the stats for race/class */
            add += rb.r_adj[i as usize] + cb.c_adj[i as usize];

            /* Extract the new "use_stat" value for the stat */
            let use_ = i32::from(engine::modify_stat_value(i32::from(stat_cur[i as usize]), add));

            let ind = if use_ <= 18 {
                /* Values: 3, ..., 17 */
                use_ - 3
            } else if use_ <= 18 + 219 {
                /* Ranges: 18/00-18/09, ..., 18/210-18/219 */
                15 + (use_ - 18) / 10
            } else {
                /* Range: 18/220+ */
                37
            };

            /* Save the index */
            t[BI_STR_INDEX + i] = if ind > 37 { 37 } else { ind };
            t[BI_STR + i] = use_;
            t[BI_CSTR + i] = i32::from(stat_cur[i as usize]);
        }

        t[BI_HP_ADJ] = i32::from(p.player_hp[(i32::from(p.lev) - 1) as usize])
            + BORG_ADJ_CON_MHP[t[BI_CON_INDEX] as usize] * t[BI_CLEVEL] / 100;
    }

    /* 'Mana' is actually the 'mana adjustment' */
    let spell_stat = borg_spell_stat();
    if spell_stat >= 0 {
        let mut t = tr(st);
        let ind = t[BI_STR_INDEX + spell_stat as u32] as usize;
        t[BI_SP_ADJ] =
            (BORG_ADJ_MAG_MANA[ind] * (t[BI_CLEVEL] - cb.magic.spell_first + 1)) / 2;
        t[BI_FAIL1] = BORG_ADJ_MAG_STAT[ind];
        t[BI_FAIL2] = BORG_ADJ_MAG_FAIL[ind];
    }

    let hold;
    {
        let mut t = tr(st);

        /* Bloating slows the player down (a little) */
        if t[BI_ISGORGED] != 0 {
            t[BI_SPEED] -= 10;
        }

        /* Actual Modifier Bonuses */
        t[BI_ARMOR] += BORG_ADJ_DEX_TA[t[BI_DEX_INDEX] as usize];
        t[BI_TODAM] += BORG_ADJ_STR_TD[t[BI_STR_INDEX] as usize];
        t[BI_TOHIT] += BORG_ADJ_DEX_TH[t[BI_DEX_INDEX] as usize];
        t[BI_TOHIT] += BORG_ADJ_STR_TH[t[BI_STR_INDEX] as usize];

        /* Obtain the "hold" value */
        hold = engine::adj_str_hold(t[BI_STR_INDEX] as usize);

        /* digging */
        t[BI_DIG] += BORG_ADJ_STR_DIG[t[BI_STR_INDEX] as usize];
    }

    /* Examine the "current bow" */
    {
        let BorgState { borg, items, item_val, .. } = &mut *st;
        let item = &mut items.items[(wield + 1) as usize];
        let mut t = Traits(&mut borg.trait_);

        /* attacking with bare hands */
        if item.iqty == 0 || cursed_nonartifact(item) {
            item.ds = 0;
            item.dd = 0;
            item.to_d = 0;
            item.to_h = 0;
            item.weight = 0;
            item.ident = true;
            item.sval = 0;
        }

        /* Real bonuses */
        t[BI_BTOHIT] = i32::from(item.to_h);
        t[BI_BTODAM] = i32::from(item.to_d);
        t[BI_BID] = i32::from(item.ident);
        t[BI_SLING] = i32::from(i32::from(item.sval) == item_val.sv_sling);
        t[BI_BART] = i32::from(item.art_idx);

        let w10 = i32::from(item.weight) / 10;

        /* It is hard to carholdry a heavy bow */
        if hold < w10 {
            t[BI_HEAVYBOW] = 1;
            /* Hard to wield a heavy bow */
            t[BI_TOHIT] += 2 * (hold - w10);
        }

        /* Compute "extra shots" if needed */
        if item.iqty != 0 && hold >= w10 {
            let sval = i32::from(item.sval);
            /* Take note of required "tval" for missiles */
            let ammo = if sval == item_val.sv_sling {
                Some((TV_SHOT, 3, 2))
            } else if sval == item_val.sv_short_bow {
                Some((TV_ARROW, 4, 2))
            } else if sval == item_val.sv_long_bow {
                Some((TV_ARROW, 4, 3))
            } else if sval == item_val.sv_light_xbow {
                Some((TV_BOLT, 5, 3))
            } else if sval == item_val.sv_heavy_xbow {
                Some((TV_BOLT, 5, 4))
            } else {
                None
            };
            if let Some((tval, sides, power)) = ammo {
                t[BI_AMMO_TVAL] = tval as i32;
                t[BI_AMMO_SIDES] = sides;
                t[BI_AMMO_POWER] = power;
            }

            /* Add in extra power */
            t[BI_AMMO_POWER] += extra_might;

            /* Reward High Level Rangers using Bows */
            if engine::player_has(PF_FAST_SHOT) {
                if t[BI_AMMO_TVAL] == TV_ARROW as i32 {
                    /* Extra shot at level 20 */
                    if t[BI_CLEVEL] >= 20 {
                        my_num_fire += 1;
                    }
                }

                /* Extra shot at level 40 */
                if t[BI_CLEVEL] >= 40 {
                    my_num_fire += 1;
                }

                t[BI_FAST_SHOTS] = 1;
            }

            /* Add in the "bonus shots" */
            my_num_fire += extra_shots;

            /* Require at least one shot */
            if my_num_fire < 1 {
                my_num_fire = 1;
            }
        }
        t[BI_SHOTS] = my_num_fire;
    }

    /* Examine the "main weapon" */
    let weapon = {
        let BorgState { borg, items, .. } = &mut *st;
        let item = &mut items.items[wield as usize];
        let mut t = Traits(&mut borg.trait_);

        /* attacking with bare hands */
        if item.iqty == 0 || cursed_nonartifact(item) {
            item.ds = 0;
            item.dd = 0;
            item.to_d = 0;
            item.to_h = 0;
            item.weight = 0;
            item.ident = true;
        }

        /* Real values */
        t[BI_WTOHIT] = i32::from(item.to_h);
        t[BI_WTODAM] = i32::from(item.to_d);
        t[BI_WID] = i32::from(item.ident);
        t[BI_WDD] = i32::from(item.dd);
        t[BI_WDS] = i32::from(item.ds);

        let w10 = i32::from(item.weight) / 10;

        /* It is hard to hold a heavy weapon */
        if hold < w10 {
            t[BI_HEAVYWEPON] = 1;

            /* Hard to wield a heavy weapon */
            t[BI_TOHIT] += 2 * (hold - w10);
        }
        *item
    };

    /* Normal weapons */
    if weapon.iqty != 0 && hold >= i32::from(weapon.weight) / 10 {
        /* calculate the number of blows */
        let blows = borg_calc_blows(st, &weapon);
        let mut t = tr(st);
        t[BI_BLOWS] = blows;

        /* Boost digging skill by weapon weight */
        t[BI_DIG] += i32::from(weapon.weight) / 10;
    }

    {
        /* Reward High Level Warriors with Res Fear */
        if engine::player_has(PF_BRAVERY_30) {
            /* Resist fear at level 30 */
            if t(st, BI_CLEVEL) >= 30 {
                tr(st)[BI_RFEAR] = 1;
            }
        }

        let disp = level_skill(st, SKILL_DISARM_PHYS);
        let dism = level_skill(st, SKILL_DISARM_MAGIC);
        let dev = level_skill(st, SKILL_DEVICE);
        let sav = level_skill(st, SKILL_SAVE);
        let stl = level_skill(st, SKILL_STEALTH);
        let srch = level_skill(st, SKILL_SEARCH);
        let thn = level_skill(st, SKILL_TO_HIT_MELEE);
        let thb = level_skill(st, SKILL_TO_HIT_BOW);
        let tht = level_skill(st, SKILL_TO_HIT_THROW);

        let mut t = tr(st);

        /* Affect Skill -- stealth (bonus one) */
        t[BI_STL] += 1;

        /* Affect Skill -- disarming (DEX and INT) */
        t[BI_DISP] += BORG_ADJ_DEX_DIS[t[BI_DEX_INDEX] as usize];
        t[BI_DISM] += BORG_ADJ_INT_DIS[t[BI_INT_INDEX] as usize];

        /* Affect Skill -- magic devices (INT) */
        t[BI_DEV] += BORG_ADJ_INT_DEV[t[BI_INT_INDEX] as usize];

        /* Affect Skill -- saving throw (WIS) */
        t[BI_SAV] += BORG_ADJ_WIS_SAV[t[BI_WIS_INDEX] as usize];

        /* Affect Skill -- disarming (Level, by Class) */
        t[BI_DISP] += disp;
        t[BI_DISM] += dism;

        /* Affect Skill -- magic devices (Level, by Class) */
        t[BI_DEV] += dev;

        /* Affect Skill -- saving throw (Level, by Class) */
        t[BI_SAV] += sav;

        /* Affect Skill -- stealth (Level, by Class) */
        t[BI_STL] += stl;

        /* Affect Skill -- search ability (Level, by Class) */
        t[BI_SRCH] += srch;

        /* Affect Skill -- combat (normal) (Level, by Class) */
        t[BI_THN] += thn;

        /* Affect Skill -- combat (shooting) (Level, by Class) */
        t[BI_THB] += thb;

        /* Affect Skill -- combat (throwing) (Level, by Class) */
        t[BI_THT] += tht;

        /* Limit Skill -- stealth from 0 to 30 */
        t[BI_STL] = t[BI_STL].clamp(0, 30);

        /* Limit Skill -- digging from 1 up */
        if t[BI_DIG] < 1 {
            t[BI_DIG] = 1;
        }

        /*** Some penalties to consider ***/

        /* Fear from spell or effect or flag */
        if t[BI_ISAFRAID] != 0 || t[BI_CRSFEAR] != 0 {
            t[BI_TOHIT] -= 20;
            t[BI_ARMOR] += 8;
            t[BI_DEV] = t[BI_DEV] * 95 / 100;
        }

        /* priest weapon penalty for non-blessed edged weapons */
        if engine::player_has(PF_BLESS_WEAPON)
            && (u32::from(weapon.tval) == TV_HAFTED || of_has(&weapon.flags, OF_BLESSED))
        {
            /* Reduce the real bonuses */
            t[BI_TOHIT] += 2;
            t[BI_TODAM] += 2;
        }
    }

    /*** Count needed enchantment ***/
    let limit = st.core.cfg[BORG_ENCHANT_LIMIT as usize];

    /* Enchant all the equipment (weapons) */
    for i in wield..=wield + 1 {
        let item = st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip "unknown" items */
        if !item.ident {
            continue;
        }

        /* skip cursed non-artifacts */
        if cursed_nonartifact(&item) {
            continue;
        }

        /* Most classes store the enchants until they get
         * a 3x shooter (like a long bow).
         * --Important: Also look in borg7.c for the enchanting.
         * --We do not want the bow enchanted by mistake.
         */
        if i == wield + 1 /* bow */
            && t(st, BI_AMMO_POWER) < 3 /* 3x shooter */
            && (item.art_idx == 0 && item.ego_idx == 0)
        /* Not Ego or Artifact */
        {
            continue;
        }

        let (to_h, to_d) = (i32::from(item.to_h), i32::from(item.to_d));

        /* Enchant all weapons (to hit) */
        if borg_spell_legal_fail(st, ENCHANT_WEAPON, 65) || t(st, BI_AENCH_SWEP) >= 1 {
            let mut t = tr(st);
            if to_h < limit {
                t[BI_NEED_ENCHANT_TO_H] += limit - to_h;
            }

            /* Enchant all weapons (to damage) */
            if to_d < limit {
                t[BI_NEED_ENCHANT_TO_D] += limit - to_d;
            }
        } else
        /* I don't have the spell or *enchant* */
        {
            let mut t = tr(st);
            if to_h < 8 {
                t[BI_NEED_ENCHANT_TO_H] += 8 - to_h;
            }

            /* Enchant all weapons (to damage) */
            if to_d < 8 {
                t[BI_NEED_ENCHANT_TO_D] += 8 - to_d;
            }
        }
    }

    /* Enchant all the equipment (armor) */
    for i in wield + 6..=wield + 11 {
        let item = st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip "unknown" items */
        if !item.ident {
            continue;
        }

        /* skip cursed non-artifacts */
        if cursed_nonartifact(&item) {
            continue;
        }

        let to_a = i32::from(item.to_a);

        /* Note need for enchantment */
        if borg_spell_legal_fail(st, ENCHANT_ARMOUR, 65) || t(st, BI_AENCH_SARM) >= 1 {
            if to_a < limit {
                tr(st)[BI_NEED_ENCHANT_TO_A] += limit - to_a;
            }
        } else if to_a < 8 {
            tr(st)[BI_NEED_ENCHANT_TO_A] += 8 - to_a;
        }
    }

    /* Special way to handle See Inv */
    if st.borg.see_inv >= 1 {
        tr(st)[BI_SINV] = 1;
    }
    if t(st, BI_CDEPTH) == 0 /* only in town.  Allow him to recall down */
        && borg_spell_legal(st, SENSE_INVISIBLE)
    {
        tr(st)[BI_SINV] = 1;
    }

    {
        let mut t = tr(st);

        /* Very special handling of Free Action.
         * If the person has perfect Savings throw, he can be
         * considered ok on Free Action.  This can free up an
         * equipment slot.
         */
        if t[BI_SAV] >= 100 {
            t[BI_FRACT] = 1;
        }

        /* Special case for RBlindness.  Perfect saves and the
         * resistances for light and dark are good enough for RBlind
         */
        if t[BI_SAV] >= 100 && t[BI_RDARK] != 0 && t[BI_RLITE] != 0 {
            t[BI_RBLIND] = 1;
        }
    }

    /*** Quiver needs to be evaluated ***/

    /* Ignore invalid missiles */
    for i in quiver_start()..quiver_end() {
        borg_notice_ammo(st, i);
    }
}

/// The food part of the inventory scan (C's `case TV_FOOD:`, also reached
/// by falling through from `case TV_MUSHROOM:`).
fn notice_food(st: &mut BorgState, item: &borg_item) {
    let iqty = i32::from(item.iqty);

    /* check for food that hurts us */
    if borg_obj_has_effect(item.kind, EF_CRUNCH as i32, -1)
        || borg_obj_has_effect(item.kind, EF_TIMED_INC as i32, TMD_CONFUSED as i32)
    {
        return;
    }

    let sval = i32::from(item.sval);
    let v = &st.item_val;
    let mut t = Traits(&mut st.borg.trait_);

    /* check for food that gives nutrition */
    if u32::from(item.tval) == TV_MUSHROOM {
        /* mushrooms that increase nutrition are low effect */
        if borg_obj_has_effect(item.kind, EF_NOURISH as i32, 0) {
            t[BI_FOOD_LO] += iqty;
        }
    } else
    /* TV_FOOD */
    {
        if sval == v.sv_food_apple
            || sval == v.sv_food_handful
            || sval == v.sv_food_slime_mold
            || sval == v.sv_food_pint
            || sval == v.sv_food_sip
        {
            t[BI_FOOD_LO] += iqty;
        } else if sval == v.sv_food_ration
            || sval == v.sv_food_slice
            || sval == v.sv_food_honey_cake
            || sval == v.sv_food_waybread
            || sval == v.sv_food_draught
        {
            t[BI_FOOD_HI] += iqty;
        }
    }

    /* check for food that does stuff */
    if borg_obj_has_effect(item.kind, EF_CURE as i32, TMD_POISONED as i32) {
        t[BI_ACUREPOIS] += iqty;
    }
    if borg_obj_has_effect(item.kind, EF_CURE as i32, TMD_CONFUSED as i32) {
        t[BI_FOOD_CURE_CONF] += iqty;
    }
    if borg_obj_has_effect(item.kind, EF_CURE as i32, TMD_BLIND as i32) {
        t[BI_FOOD_CURE_BLIND] += iqty;
    }
}

/// `int16_t += int` (truncating).
fn add_i16(v: &mut i16, x: i32) {
    *v = (i32::from(*v) + x) as i16;
}

/// One pack item of `borg_notice_inventory` (the `switch (item->tval)`).
/// `remove_hunger` is C's local count of Remove Hunger scrolls.
fn notice_pack_item(st: &mut BorgState, item: &borg_item, remove_hunger: &mut i32) {
    let iqty = i32::from(item.iqty);
    let sval = i32::from(item.sval);
    let pval = i32::from(item.pval);
    let tval = u32::from(item.tval);
    let light = inven_wield() as usize + 5;

    /* Analyze the item */
    match tval {
        /* Books */
        TV_MAGIC_BOOK | TV_PRAYER_BOOK | TV_NATURE_BOOK | TV_SHADOW_BOOK | TV_OTHER_BOOK => {
            /* Skip incorrect books (if we can browse this book, it is good) */
            if !engine::obj_kind_can_browse(item.kind as usize) {
                return;
            }
            /* Count the books (a book number of -1 panics where C writes
             * out of bounds) */
            let n = borg_get_book_num(sval);
            add_i16(&mut st.borg.amt_book[n as usize], iqty);
        }

        /* Food */
        TV_MUSHROOM => {
            let v = &st.item_val;
            let cfg = &st.core.cfg;
            let munchkin = cfg[BORG_MUNCHKIN_START as usize] != 0;
            let maxclevel = t(st, BI_MAXCLEVEL);
            if (sval == v.sv_mush_purging
                || sval == v.sv_mush_restoring
                || sval == v.sv_mush_cure_mind)
                && munchkin
                && maxclevel < cfg[BORG_MUNCHKIN_LEVEL as usize]
            {
                return;
            }
            if (sval == v.sv_mush_second_sight
                || sval == v.sv_mush_emergency
                || sval == v.sv_mush_terror
                || sval == v.sv_mush_stoneskin
                || sval == v.sv_mush_debility
                || sval == v.sv_mush_sprinting)
                && munchkin
                && maxclevel >= cfg[BORG_MUNCHKIN_LEVEL as usize]
            {
                tr(st)[BI_ASHROOM] += iqty;
            }
            /* fall through */
            notice_food(st, item);
        }
        TV_FOOD => notice_food(st, item),

        /* Potions */
        TV_POTION => {
            let v = &st.item_val;
            let b = &mut st.borg;
            let mut t = Traits(&mut b.trait_);
            let g = &mut b.amt_statgain;
            /* Analyze */
            if sval == v.sv_potion_healing {
                t[BI_AHEAL] += iqty;
            } else if sval == v.sv_potion_star_healing {
                t[BI_AEZHEAL] += iqty;
            } else if sval == v.sv_potion_life {
                t[BI_ALIFE] += iqty;
            } else if sval == v.sv_potion_cure_critical {
                t[BI_ACCW] += iqty;
            } else if sval == v.sv_potion_cure_serious {
                t[BI_ACSW] += iqty;
            } else if sval == v.sv_potion_cure_light {
                t[BI_ACLW] += iqty;
            } else if sval == v.sv_potion_cure_poison {
                t[BI_ACUREPOIS] += iqty;
            } else if sval == v.sv_potion_resist_heat {
                t[BI_ARESHEAT] += iqty;
            } else if sval == v.sv_potion_resist_cold {
                t[BI_ARESCOLD] += iqty;
            } else if sval == v.sv_potion_resist_pois {
                t[BI_ARESPOIS] += iqty;
            } else if sval == v.sv_potion_inc_str {
                add_i16(&mut g[STAT_STR as usize], iqty);
            } else if sval == v.sv_potion_inc_int {
                add_i16(&mut g[STAT_INT as usize], iqty);
            } else if sval == v.sv_potion_inc_wis {
                add_i16(&mut g[STAT_WIS as usize], iqty);
            } else if sval == v.sv_potion_inc_dex {
                add_i16(&mut g[STAT_DEX as usize], iqty);
            } else if sval == v.sv_potion_inc_con {
                add_i16(&mut g[STAT_CON as usize], iqty);
            } else if sval == v.sv_potion_inc_all {
                for s in [STAT_STR, STAT_INT, STAT_WIS, STAT_DEX, STAT_CON] {
                    add_i16(&mut g[s as usize], iqty);
                }
            } else if sval == v.sv_potion_restore_life {
                t[BI_HASFIXEXP] = 1;
            } else if sval == v.sv_potion_speed {
                t[BI_ASPEED] += iqty;
            }
        }

        /* Scrolls */
        TV_SCROLL => {
            let v = &st.item_val;
            let mut t = Traits(&mut st.borg.trait_);
            if sval == v.sv_scroll_identify {
                t[BI_AID] += iqty;
            } else if sval == v.sv_scroll_satisfy_hunger {
                *remove_hunger += iqty;
            } else if sval == v.sv_scroll_recharging {
                t[BI_ARECHARGE] += iqty;
            } else if sval == v.sv_scroll_phase_door {
                t[BI_APHASE] += iqty;
            } else if sval == v.sv_scroll_teleport {
                t[BI_ATELEPORT] += iqty;
            } else if sval == v.sv_scroll_word_of_recall {
                t[BI_RECALL] += iqty;
            } else if sval == v.sv_scroll_enchant_armor {
                t[BI_AENCH_ARM] += iqty;
            } else if sval == v.sv_scroll_star_enchant_armor {
                t[BI_AENCH_SARM] += iqty;
            } else if sval == v.sv_scroll_enchant_weapon_to_hit {
                t[BI_AENCH_TOH] += iqty;
            } else if sval == v.sv_scroll_enchant_weapon_to_dam {
                t[BI_AENCH_TOD] += iqty;
            } else if sval == v.sv_scroll_star_enchant_weapon {
                t[BI_AENCH_SWEP] += iqty;
            } else if sval == v.sv_scroll_protection_from_evil {
                t[BI_APFE] += iqty;
            } else if sval == v.sv_scroll_rune_of_protection {
                t[BI_AGLYPH] += iqty;
            } else if sval == v.sv_scroll_teleport_level {
                t[BI_ATELEPORTLVL] += iqty;
                t[BI_ATELEPORT] += 1;
            } else if sval == v.sv_scroll_mass_banishment {
                t[BI_AMASSBAN] += iqty;
            }
        }

        /* Rods */
        TV_ROD => {
            let v = rod_svals(&st.item_val);
            /* Analyze */
            if sval == v.recall {
                /* Don't count on it if I suck at activations */
                let n = if borg_activate_failure(st, tval as i32, sval) < 500 {
                    iqty * 100
                } else {
                    iqty
                };
                tr(st)[BI_RECALL] += n;
            } else if sval == v.detection {
                let mut t = tr(st);
                t[BI_ADETTRAP] += iqty * 100;
                t[BI_ADETDOOR] += iqty * 100;
                t[BI_ADETEVIL] += iqty * 100;
            } else if sval == v.illumination {
                tr(st)[BI_ALITE] += iqty * 100;
            } else if sval == v.speed {
                /* Don't count on it if I suck at activations */
                let n = if borg_activate_failure(st, tval as i32, sval) < 500 {
                    iqty * 100
                } else {
                    iqty
                };
                tr(st)[BI_ASPEED] += n;
            } else if sval == v.tele_other {
                /* Counted like a wand of teleport other */
                tr(st)[BI_ATPORTOTHER] += iqty;
            } else if sval == v.mapping {
                tr(st)[BI_AMAGICMAP] += iqty * 100;
            } else if sval == v.healing {
                /* only +2 per rod because of long charge time. */
                /* Don't count on it if I suck at activations */
                let n = if borg_activate_failure(st, tval as i32, sval) < 500 {
                    iqty * 3
                } else {
                    iqty + 1
                };
                tr(st)[BI_AHEAL] += n;
            } else if sval == v.light
                || sval == v.fire_bolt
                || sval == v.elec_bolt
                || sval == v.cold_bolt
                || sval == v.acid_bolt
            {
                tr(st)[BI_AROD1] += iqty;
            } else if sval == v.drain_life
                || sval == v.fire_ball
                || sval == v.elec_ball
                || sval == v.cold_ball
                || sval == v.acid_ball
            {
                tr(st)[BI_AROD2] += iqty;
            }
        }

        /* Wands */
        TV_WAND => {
            let v = &st.item_val;
            let mut t = Traits(&mut st.borg.trait_);
            /* Analyze each */
            if sval == v.sv_wand_teleport_away {
                t[BI_ATPORTOTHER] += pval;
            }

            if sval == v.sv_wand_stinking_cloud && t[BI_MAXDEPTH] < 30 {
                t[BI_GOOD_W_CHG] += pval;
            }

            if sval == v.sv_wand_magic_missile && t[BI_MAXDEPTH] < 30 {
                t[BI_GOOD_W_CHG] += pval;
            }

            if sval == v.sv_wand_annihilation {
                t[BI_GOOD_W_CHG] += pval;
            }
        }

        /* Staffs */
        TV_STAFF => {
            let v = &st.item_val;
            /* Analyze */
            if sval == v.sv_staff_teleportation {
                tr(st)[BI_AESCAPE] += iqty;
                if borg_activate_failure(st, tval as i32, sval) < 500 {
                    tr(st)[BI_AESCAPE] += pval;
                }
            } else {
                let mut t = Traits(&mut st.borg.trait_);
                if sval == v.sv_staff_speed {
                    t[BI_ASPEED] += pval;
                } else if sval == v.sv_staff_healing {
                    t[BI_AHEAL] += pval;
                } else if sval == v.sv_staff_the_magi {
                    t[BI_ASTFMAGI] += pval;
                } else if sval == v.sv_staff_destruction {
                    t[BI_ASTFDEST] += pval;
                } else if sval == v.sv_staff_power {
                    t[BI_GOOD_S_CHG] += iqty;
                } else if sval == v.sv_staff_holiness {
                    t[BI_GOOD_S_CHG] += iqty;
                    t[BI_AHEAL] += pval;
                }
            }
        }

        /* Flasks */
        TV_FLASK => {
            /* Use as fuel if we equip a lantern */
            if i32::from(st.items.items[light].sval) == st.item_val.sv_light_lantern {
                tr(st)[BI_AFUEL] += iqty;
            }
        }

        /* Torches */
        TV_LIGHT => {
            let torch = st.item_val.sv_light_torch;
            let l = &st.items.items[light];
            /* Use as fuel if it is a torch and we carry a torch */
            if (sval == torch && item.timeout >= 1) && i32::from(l.sval) == torch && l.iqty != 0 {
                tr(st)[BI_AFUEL] += iqty;
            }
        }

        /* Weapons */
        TV_HAFTED | TV_POLEARM | TV_SWORD => {
            /* These items are checked a bit later in a sub routine
             * to notice the flags.  It is done outside this switch.
             */
        }

        /* Shovels and such */
        TV_DIGGING => {
            /* Ignore worthless ones (including cursed) */
            if item.value <= 0 {
                return;
            }
            if item.cursed {
                return;
            }

            /* Do not carry if weak, won't be able to dig anyway */
            if t(st, BI_DIG) < BORG_DIG {
                return;
            }

            tr(st)[BI_ADIGGER] += iqty;
        }

        _ => {}
    }
}

/// The rod svals `borg_notice_inventory` compares (copied, so the state
/// can be borrowed mutably for `borg_activate_failure` in between).
struct RodSvals {
    recall: i32,
    detection: i32,
    illumination: i32,
    speed: i32,
    tele_other: i32,
    mapping: i32,
    healing: i32,
    light: i32,
    fire_bolt: i32,
    elec_bolt: i32,
    cold_bolt: i32,
    acid_bolt: i32,
    drain_life: i32,
    fire_ball: i32,
    elec_ball: i32,
    cold_ball: i32,
    acid_ball: i32,
}

fn rod_svals(v: &crate::borg_item_val::ItemValState) -> RodSvals {
    RodSvals {
        recall: v.sv_rod_recall,
        detection: v.sv_rod_detection,
        illumination: v.sv_rod_illumination,
        speed: v.sv_rod_speed,
        tele_other: v.sv_rod_teleport_other,
        mapping: v.sv_rod_mapping,
        healing: v.sv_rod_healing,
        light: v.sv_rod_light,
        fire_bolt: v.sv_rod_fire_bolt,
        elec_bolt: v.sv_rod_elec_bolt,
        cold_bolt: v.sv_rod_cold_bolt,
        acid_bolt: v.sv_rod_acid_bolt,
        drain_life: v.sv_rod_drain_life,
        fire_ball: v.sv_rod_fire_ball,
        elec_ball: v.sv_rod_elec_ball,
        cold_ball: v.sv_rod_cold_ball,
        acid_ball: v.sv_rod_acid_ball,
    }
}

/// Helper function -- notice the player inventory (`borg_notice_inventory`).
fn borg_notice_inventory(st: &mut BorgState) {
    let mut remove_hunger = 0;

    /*** Reset counters ***/

    /* Reset stat potions */
    for i in 0..STAT_MAX as usize {
        st.borg.need_statgain[i] = false;
        st.borg.amt_statgain[i] = 0;
    }

    /* Reset books */
    for i in 0..9 {
        st.borg.amt_book[i] = 0;
    }

    /*** Process the inventory ***/

    /* Scan the inventory (PACK_SLOTS is re-read at each step, as C's
     * macro) */
    let mut i = 0i32;
    while i < i32::from(engine::z_info().pack_size) - t(st, BI_QUIVER_SLOTS) {
        let item = st.items.items[i as usize];
        i += 1;
        let slot = i - 1;

        /* Skip empty items */
        if item.iqty == 0 {
            tr(st)[BI_EMPTY] += 1;
            continue;
        }

        /* special case for ammo outside the quiver. */
        /* this happens when we are deciding what to buy so items */
        /* are put in empty slots */
        if borg_is_ammo(i32::from(item.tval)) {
            borg_notice_ammo(st, slot);
            continue;
        }

        {
            let needs_id = borg_item_note_needs_id(st, &item);
            let mut t = tr(st);

            /* total up the weight of the items */
            t[BI_WEIGHT] += i32::from(borg_item_weight(&item));

            /* Does the borg need to get an ID for it? */
            if needs_id {
                t[BI_ALL_NEED_ID] += 1;
            }

            /* track first uncursable item */
            if item.uncursable {
                t[BI_WHERE_CURSED] |= BORG_INVEN as i32;
                if t[BI_FIRST_CURSED] == 0 {
                    t[BI_FIRST_CURSED] = slot + 1;
                }
            }
        }

        /* Skip un-aware items */
        if !item.aware {
            continue;
        }

        /* count up the items on the borg (do not count artifacts  */
        /* that are not being wielded) */
        st.borg.has[item.kind as usize] += i32::from(item.iqty);

        /* skip cursed non-artifacts */
        if cursed_nonartifact(&item) {
            continue;
        }

        notice_pack_item(st, &item, &mut remove_hunger);
    }

    /* flasks of oil are ammo at low levels */
    let oil = st.borg.has[st.item_val.kv_flask_oil as usize];
    if oil != 0 && t(st, BI_CLEVEL) < 15 {
        /* only count the first 15 */
        tr(st)[BI_AMISSILES] += if oil < 15 { oil } else { 15 };
    }

    /*** Process the Spells and Prayers ***/
    /*    artifact activations are accounted here
     *  But some artifacts are not counted for two reasons .
     *  1.  Some spells-powers are needed instantly and are considered in
     *  the borg preparation code.  An artifact maybe non-charged at the
     *  moment he needs it.  Then he would need the spell and not be able
     *  to cast it. (ie. teleport, phase)
     *  2.  An artifact may grant a power then he assumes he has infinite
     *  amounts.  He then sells off his scrolls with the duplicate power.
     *  When it comes time to upgrade and swap out the artifact, he won't
     *  because his power drops since he does not have the scrolls anymore.
     *  and he does not buy items first.
     *
     *  A possible solution would be to have him keep a few scrolls as a
     *  back-up, or to remove the bonus for level preparation from borg_power.
     *  Right now I think it is better that he not consider the artifacts
     *  Whose powers are considered in borg_prep.
     */

    /* Handle "satisfy hunger" -> infinite food */
    if borg_spell_legal_fail(st, REMOVE_HUNGER, 80) || borg_spell_legal_fail(st, HERBAL_CURING, 80)
    /* VAMPIRE_STRIKE? */
    {
        tr(st)[BI_FOOD] += 1000;
    }

    /* Handle "identify" -> infinite identifies */
    if borg_spell_legal(st, IDENTIFY_RUNE) {
        tr(st)[BI_AID] += 1000;
    }

    /* Handle "detect traps" */
    if borg_spell_legal(st, FIND_TRAPS_DOORS_STAIRS) || borg_spell_legal(st, DETECTION) {
        tr(st)[BI_ADETTRAP] = 1000;
    }

    /* Handle "detect evil & monsters" */
    if borg_spell_legal(st, REVEAL_MONSTERS)
        || borg_spell_legal(st, DETECT_LIFE)
        || borg_spell_legal(st, DETECT_EVIL)
        || borg_spell_legal(st, READ_MINDS)
        || borg_spell_legal(st, DETECT_MONSTERS)
        || borg_spell_legal(st, SEEK_BATTLE)
    {
        tr(st)[BI_ADETEVIL] = 1000;
    }

    /* Handle DETECTION */
    if borg_spell_legal(st, DETECTION) || equips!(st, enlightenment) || equips!(st, clairvoyance) {
        let mut t = tr(st);
        t[BI_ADETDOOR] = 1000;
        t[BI_ADETTRAP] = 1000;
        t[BI_ADETEVIL] = 1000;
    }

    /* Handle "See Invisible" in a special way. */
    if borg_spell_legal(st, SENSE_INVISIBLE) {
        tr(st)[BI_DINV] = 1;
    }

    /* Handle "magic mapping" */
    if borg_spell_legal(st, SENSE_SURROUNDINGS)
        || equips!(st, detect_all)
        || equips!(st, mapping)
    {
        let mut t = tr(st);
        t[BI_ADETDOOR] = 1000;
        t[BI_ADETTRAP] = 1000;
        t[BI_AMAGICMAP] = 1000;
    }

    /* Handle "call lite" */
    if borg_spell_legal(st, LIGHT_ROOM)
        || equips!(st, light)
        || equips!(st, illumination)
        || borg_spell_legal(st, CALL_LIGHT)
    {
        tr(st)[BI_ALITE] += 1000;
    }

    /* Handle PROTECTION_FROM_EVIL */
    if borg_spell_legal(st, PROTECTION_FROM_EVIL)
        || equips!(st, protevil)
        || st.borg.has[st.item_val.kv_staff_holiness as usize] != 0
        || equips!(st, staff_holy)
    {
        tr(st)[BI_APFE] += 1000;
    }

    /* Handle "rune of protection" glyph" */
    if borg_spell_legal(st, GLYPH_OF_WARDING) || equips!(st, glyph) {
        tr(st)[BI_AGLYPH] += 1000;
    }

    /* Handle "detect traps/doors" */
    if borg_spell_legal(st, FIND_TRAPS_DOORS_STAIRS) {
        let mut t = tr(st);
        t[BI_ADETDOOR] = 1000;
        t[BI_ADETTRAP] = 1000;
    }

    /* Handle ENCHANT_WEAPON */
    if borg_spell_legal_fail(st, ENCHANT_WEAPON, 65) || equips!(st, enchant_weapon) {
        let mut t = tr(st);
        t[BI_AENCH_TOH] += 1000;
        t[BI_AENCH_TOD] += 1000;
        t[BI_AENCH_SWEP] += 1000;
    }
    if equips!(st, enchant_tohit) {
        tr(st)[BI_AENCH_TOH] += 1000;
    }
    if equips!(st, enchant_todam) {
        tr(st)[BI_AENCH_TOD] += 1000;
    }

    /* Handle "Brand Weapon (bolts)" */
    if equips!(st, firebrand) || borg_spell_legal_fail(st, BRAND_AMMUNITION, 65) {
        tr(st)[BI_ABRAND] += 1000;
    }

    /* Handle "enchant armor" */
    if borg_spell_legal_fail(st, ENCHANT_ARMOUR, 65)
        || equips!(st, enchant_armor)
        || equips!(st, enchant_armor2)
    {
        let mut t = tr(st);
        t[BI_AENCH_ARM] += 1000;
        t[BI_AENCH_SARM] += 1000;
    }

    /* Handle Diggers (stone to mud) */
    if borg_spell_legal_fail(st, TURN_STONE_TO_MUD, 40) || equips!(st, stone_to_mud) || {
        let s = st.item_val.sv_ring_digging;
        borg_equips_ring(st, s)
    } {
        tr(st)[BI_ADIGGER] += 1;
    }

    /* Handle recall */
    if borg_spell_legal_fail(st, WORD_OF_RECALL, 40)
        || (t(st, BI_CDEPTH) == 100 && borg_spell_legal(st, WORD_OF_RECALL))
    {
        tr(st)[BI_RECALL] += 1000;
    }
    if equips!(st, recall) {
        tr(st)[BI_RECALL] += 1;
    }

    /* Handle teleport_level */
    if borg_spell_legal_fail(st, TELEPORT_LEVEL, 20) {
        tr(st)[BI_ATELEPORTLVL] += 1000;
    }

    /* Handle PhaseDoor spell carefully */
    if borg_spell_legal_fail(st, PHASE_DOOR, 3) {
        tr(st)[BI_APHASE] += 1000;
    }
    if equips!(st, tele_phase) {
        tr(st)[BI_APHASE] += 1;
    }

    /* Handle teleport spell carefully */
    if borg_spell_legal_fail(st, TELEPORT_SELF, 1)
        || borg_spell_legal_fail(st, PORTAL, 1)
        || borg_spell_legal_fail(st, SHADOW_SHIFT, 1)
        || borg_spell_legal_fail(st, DIMENSION_DOOR, 1)
    {
        tr(st)[BI_ATELEPORT] += 1000;
    }
    if equips!(st, tele_long) {
        let mut t = tr(st);
        t[BI_AESCAPE] += 1;
        t[BI_ATELEPORT] += 1;
    }

    /* Handle teleport away */
    if borg_spell_legal_fail(st, TELEPORT_OTHER, 40) {
        tr(st)[BI_ATPORTOTHER] += 1000;
    }

    /* Handle Holy Word prayer just to see if legal */
    if borg_spell_legal(st, HOLY_WORD) {
        tr(st)[BI_AHWORD] += 1000;
    }

    /* speed spells HASTE*/
    if borg_spell_legal(st, HASTE_SELF)
        || equips!(st, haste)
        || equips!(st, haste1)
        || equips!(st, haste2)
    {
        tr(st)[BI_ASPEED] += 1000;
    }

    /* Handle "cure light wounds" */
    if equips!(st, cure_light) {
        tr(st)[BI_ACLW] += 1000;
    }

    /* Handle "cure serious wounds" */
    if equips!(st, cure_serious) {
        tr(st)[BI_ACSW] += 1000;
    }

    /* Handle "cure critical wounds" */
    if equips!(st, cure_critical) {
        tr(st)[BI_ACCW] += 1000;
    }

    /* Handle "heal" */
    if equips!(st, cure_full)
        || equips!(st, cure_full2)
        || equips!(st, cure_nonorlybig)
        || equips!(st, heal1)
        || equips!(st, heal2)
        || equips!(st, heal3)
        || borg_spell_legal(st, HEALING)
    {
        tr(st)[BI_AHEAL] += 1000;
    }

    /* Handle "fix exp" */
    if equips!(st, cure_nonorlybig)
        || equips!(st, restore_exp)
        || equips!(st, restore_st_lev)
        || equips!(st, restore_life)
    {
        tr(st)[BI_HASFIXEXP] = 1;
    }

    /* Handle REMEMBRANCE -- is just as good as Hold Life */
    if borg_spell_legal(st, REMEMBRANCE)
        || equips!(st, cure_nonorlybig)
        || equips!(st, restore_exp)
        || equips!(st, restore_st_lev)
        || equips!(st, restore_life)
    {
        tr(st)[BI_HLIFE] = 1;
    }

    /* Handle "recharge" */
    if equips!(st, recharge) || borg_spell_legal(st, RECHARGING) {
        tr(st)[BI_ARECHARGE] += 1000;
    }

    /*** Process the Needs ***/

    /* No need for fuel if we know it doesn't need it */
    let light = inven_wield() as usize + 5;
    if of_has(&st.items.items[light].flags, OF_NO_FUEL) || t(st, BI_CLASS) == CLASS_NECROMANCER {
        tr(st)[BI_AFUEL] += 1000;
    }

    /* No need to *buy* stat increase potions */
    for (bi, s) in [
        (BI_CSTR, STAT_STR),
        (BI_CINT, STAT_INT),
        (BI_CWIS, STAT_WIS),
        (BI_CDEX, STAT_DEX),
        (BI_CCON, STAT_CON),
    ] {
        if t(st, bi) < 18 + 100 {
            st.borg.need_statgain[s as usize] = true;
        }
    }

    let mut t = tr(st);

    /* No need for experience repair */
    if t[BI_ISFIXEXP] == 0 {
        t[BI_HASFIXEXP] = 1;
    }

    /* Correct the high and low calorie foods */
    t[BI_FOOD] += t[BI_FOOD_HI];
    t[BI_FOOD] += t[BI_FOOD_LO];

    /* With no other food, Remove Hunger scrolls are the food supply */
    if t[BI_FOOD] == 0 {
        t[BI_FOOD] = remove_hunger;
    }

    /* If weak, do not count food spells */
    if t[BI_ISWEAK] != 0 && t[BI_FOOD] >= 1000 {
        t[BI_FOOD] -= 1000;
    }
}

/// Analyze the equipment and inventory (`borg_notice`).
pub fn borg_notice(st: &mut BorgState, notice_swap: bool) {
    /* Clear out trait arrays (each is allocated with its full C size) */
    st.borg.has.fill(0);
    st.borg.trait_[..BI_MAX as usize].fill(0);
    st.borg.activation.fill(0);

    {
        let mut t = tr(st);

        /* Start with a single blow per turn */
        t[BI_BLOWS] = 1;

        /* speed starts at 110 */
        t[BI_SPEED] = 110;

        /* Reset the "ammo" attributes */
        t[BI_AMMO_TVAL] = -1;
        t[BI_AMMO_SIDES] = 4;
    }

    /* Many of our variables are tied to borg.trait[], which is erased at the
     * the start of borg_notice().  So we must update the frame the cheat in
     * all the non inventory skills.
     */
    borg_notice_player(st);

    /*** Process books/spells ***/
    if st.think.do_spell {
        borg_cheat_spells(st);
        st.think.do_spell = false;
    }

    /* Notice the equipment */
    borg_notice_equipment(st);

    /* Notice the inventory */
    borg_notice_inventory(st);

    /* Notice and locate my swap weapon */
    if notice_swap {
        borg_notice_weapon_swap(st);
        borg_notice_armour_swap(st);
    }

    let resistance = |st: &BorgState| borg_spell_legal_fail(st, RESISTANCE, 15); /* Res FECAP */
    let (w, a) = (st.trait_swap.weapon, st.trait_swap.armour);
    let v = t(st, BI_RACID) != 0 || a.resist_acid || w.resist_acid || resistance(st);
    tr(st)[BI_SRACID] = i32::from(v);
    let v = t(st, BI_RELEC) != 0 || a.resist_elec || w.resist_elec || resistance(st);
    tr(st)[BI_SRELEC] = i32::from(v);
    let v = t(st, BI_RFIRE) != 0 || a.resist_fire || w.resist_fire || resistance(st);
    tr(st)[BI_SRFIRE] = i32::from(v);
    let v = t(st, BI_RCOLD) != 0 || a.resist_cold || w.resist_cold || resistance(st);
    tr(st)[BI_SRCOLD] = i32::from(v);
    let v = t(st, BI_RPOIS) != 0
        || a.resist_pois
        || w.resist_pois
        || borg_spell_legal_fail(st, RESIST_POISON, 15); /* Res P */
    tr(st)[BI_SRPOIS] = i32::from(v);

    {
        let mut t = tr(st);
        let mut swap = |bi: u32, base: u32, a: bool, w: bool| {
            t[bi] = i32::from(t[base] != 0 || a || w);
        };
        swap(BI_SRFEAR, BI_RFEAR, a.resist_fear, w.resist_fear);
        swap(BI_SRLITE, BI_RLITE, a.resist_light, w.resist_light);
        swap(BI_SRDARK, BI_RDARK, a.resist_dark, w.resist_dark);
        swap(BI_SRBLIND, BI_RBLIND, a.resist_blind, w.resist_blind);
        swap(BI_SRCONF, BI_RCONF, a.resist_conf, w.resist_conf);
        swap(BI_SRSND, BI_RSND, a.resist_sound, w.resist_sound);
        swap(BI_SRSHRD, BI_RSHRD, a.resist_shard, w.resist_shard);
        swap(BI_SRNXUS, BI_RNXUS, a.resist_nexus, w.resist_nexus);
        swap(BI_SRNTHR, BI_RNTHR, a.resist_neth, w.resist_neth);
        swap(BI_SRKAOS, BI_RKAOS, a.resist_chaos, w.resist_chaos);
        swap(BI_SRDIS, BI_RDIS, a.resist_disen, w.resist_disen);
        swap(BI_SHLIFE, BI_HLIFE, a.hold_life, w.hold_life);
        swap(BI_SFRACT, BI_FRACT, a.free_act, w.free_act);

        /* Apply "encumbrance" from weight */
        /* Extract the "weight limit" (in tenth pounds) */
        t[BI_CARRY] = BORG_ADJ_STR_WGT[t[BI_STR_INDEX] as usize] * 100;

        /* Apply "encumbrance" from weight */
        if t[BI_WEIGHT] > t[BI_CARRY] / 2 {
            t[BI_SPEED] -= (t[BI_WEIGHT] - (t[BI_CARRY] / 2)) / (t[BI_CARRY] / 10);
        }

        /* top speed */
        if t[BI_SPEED] > 199 {
            t[BI_SPEED] = 199;
        }
    }

    /* Check my ratio for decrementing variables */
    let speed = t(st, BI_SPEED);
    st.traits.game_ratio = if speed > 110 {
        (100000 / (((speed - 110) * 10) + 100)) as i16
    } else {
        1000
    };

    /* set if we are preparing for fighting morgoth or sauron */
    tr(st)[BI_PREP_BIG_FIGHT] = 0;
    if t(st, BI_MAXDEPTH) >= 99 {
        /* Examine the home */
        borg_notice_home(st, None, false);

        /* pot of healing + *healing* + life */
        let mut total_big_heal = st.borg.has[st.item_val.kv_potion_healing as usize];
        total_big_heal += t(st, BI_AEZHEAL);
        total_big_heal += t(st, BI_ALIFE);

        /* plus the same at home */
        total_big_heal += i32::from(st.home.num_heal_true);
        total_big_heal += i32::from(st.home.num_ezheal_true);
        total_big_heal += i32::from(st.home.num_life_true);

        /* want bunches of heal and speed to feel prepped for the fight */
        if total_big_heal < 30 || (i32::from(st.home.num_speed) + t(st, BI_ASPEED)) < 15 {
            tr(st)[BI_PREP_BIG_FIGHT] = 1;
        }
    }
}

/// Update the Borg based on the current player values
/// (`borg_notice_player`).
pub fn borg_notice_player(st: &mut BorgState) {
    let p = engine::player_notice();
    let timed = |i: u32| i32::from(engine::player_timed(i));
    let (food_weak, food_hungry, food_full, food_max) = engine::py_food_levels();

    {
        let mut t = tr(st);

        /*** Extract class Cheat ***/
        t[BI_CLASS] = engine::class_cidx() as i32;

        /* Assume level is fine */
        t[BI_ISFIXLEV] = 0;

        /* Note "Lev" vs "LEV" */
        if p.lev < p.max_lev {
            t[BI_ISFIXLEV] = 1;
        }

        /* Extract "LEVEL xxxxxx" */
        t[BI_CLEVEL] = i32::from(p.lev);

        /* cheat the max clevel */
        t[BI_MAXCLEVEL] = i32::from(p.max_lev);

        /* Note "Winner" */
        t[BI_KING] = i32::from(p.total_winner);

        /* Assume experience is fine */
        t[BI_ISFIXEXP] = 0;

        /* Access depth Cheat */
        t[BI_CDEPTH] = i32::from(p.depth);

        /* Access max depth Cheat */
        t[BI_MAXDEPTH] = i32::from(p.max_depth);

        /* Note "Exp" vs "EXP" and am I lower than level 50*/
        if p.exp < p.max_exp {
            /* fix it if in town */
            if t[BI_CLEVEL] == 50 && t[BI_CDEPTH] == 0 {
                t[BI_ISFIXEXP] = 1;
            }

            /* don't worry about fixing it in the dungeon */
            if t[BI_CLEVEL] == 50 && t[BI_CDEPTH] >= 1 {
                t[BI_ISFIXEXP] = 0;
            }

            /* Not at Max Level */
            if t[BI_CLEVEL] != 50 {
                t[BI_ISFIXEXP] = 1;
            }
        }

        /* Extract "AU xxxxxxxxx" */
        t[BI_GOLD] = p.au;
    }

    /* A quick cheat to see if I missed a message about my status on some */
    /* timed spells */
    let b = &mut st.borg;
    if b.goal.recalling == 0 && p.word_recall != 0 {
        b.goal.recalling = i32::from(p.word_recall) * 1000;
    }
    if b.goal.recalling != 0 && p.word_recall == 0 {
        b.goal.recalling = 0;
    }
    if !b.temp.prot_from_evil && timed(TMD_PROTEVIL) != 0 {
        b.temp.prot_from_evil = timed(TMD_PROTEVIL) != 0;
    }
    let fast = || timed(TMD_FAST) != 0 || timed(TMD_SPRINT) != 0 || timed(TMD_TERROR) != 0;
    if !b.temp.fast && fast() {
        b.temp.fast = fast();
    }
    b.temp.res_acid = timed(TMD_OPP_ACID) != 0;
    b.temp.res_elec = timed(TMD_OPP_ELEC) != 0;
    b.temp.res_fire = timed(TMD_OPP_FIRE) != 0;
    b.temp.res_cold = timed(TMD_OPP_COLD) != 0;
    b.temp.res_pois = timed(TMD_OPP_POIS) != 0;
    b.temp.bless = timed(TMD_BLESSED) != 0;
    b.temp.shield = timed(TMD_SHIELD) != 0 || timed(TMD_STONESKIN) != 0;
    b.temp.fastcast = timed(TMD_FASTCAST) != 0;
    b.temp.hero = timed(TMD_HERO) != 0;
    b.temp.berserk = timed(TMD_SHERO) != 0;
    b.temp.regen = timed(TMD_HEAL) != 0;
    b.temp.venom = timed(TMD_ATT_POIS) != 0;
    b.temp.smite_evil = timed(TMD_ATT_EVIL) != 0;
    if b.see_inv == 0 && timed(TMD_SINVIS) != 0 {
        b.see_inv = 1000;
    }

    let sauron_dead = i32::from(st.kills.race_death[st.borg.id.sauron as usize]);
    let mut t = tr(st);

    /* Extract "Cur HP xxxxx" */
    t[BI_CURHP] = i32::from(p.chp);

    /* Extract "Max HP xxxxx" */
    t[BI_MAXHP] = i32::from(p.mhp);

    /* Extract "Cur SP xxxxx" (or zero) */
    t[BI_CURSP] = i32::from(p.csp);

    /* Extract "Max SP xxxxx" (or zero) */
    t[BI_MAXSP] = i32::from(p.msp);

    /* Clear all the "state flags" */
    for bi in [
        BI_ISWEAK,
        BI_ISHUNGRY,
        BI_ISFULL,
        BI_ISGORGED,
        BI_ISBLIND,
        BI_ISCONFUSED,
        BI_ISAFRAID,
        BI_ISPOISONED,
        BI_ISCUT,
        BI_ISSTUN,
        BI_ISHEAVYSTUN,
        BI_ISIMAGE,
        BI_ISSTUDY,
        BI_ISPARALYZED,
        BI_ISFORGET,
    ] {
        t[bi] = 0;
    }

    let food = timed(TMD_FOOD);
    if food < food_weak {
        /* Check for "Weak" */
        t[BI_ISWEAK] = 1;
        t[BI_ISHUNGRY] = 1;
    } else if food < food_hungry {
        /* Check for "Hungry" */
        t[BI_ISHUNGRY] = 1;
    } else if food < food_full {
        /* Check for "Normal" */
        /* Nothing */
    } else if food < food_max {
        /* Check for "Full" */
        t[BI_ISFULL] = 1;
    } else {
        /* Check for "Gorged" */
        t[BI_ISGORGED] = 1;
        t[BI_ISFULL] = 1;
    }

    /* Check for "Blind" */
    if timed(TMD_BLIND) != 0 {
        t[BI_ISBLIND] = 1;
    }

    /* Check for "Confused" */
    if timed(TMD_CONFUSED) != 0 {
        t[BI_ISCONFUSED] = 1;
    }

    /* Check for "Afraid" */
    if timed(TMD_AFRAID) != 0 {
        t[BI_ISAFRAID] = 1;
    }

    /* Check for "Poisoned" */
    if timed(TMD_POISONED) != 0 {
        t[BI_ISPOISONED] = 1;
    }

    /* Check for any text */
    if timed(TMD_CUT) != 0 {
        t[BI_ISCUT] = 1;
    }

    /* Check for Stun */
    if timed(TMD_STUN) != 0 && timed(TMD_STUN) <= 50 {
        t[BI_ISSTUN] = 1;
    }

    /* Check for Heavy Stun */
    if timed(TMD_STUN) > 50 {
        t[BI_ISHEAVYSTUN] = 1;
    }

    /* Check for Paralyze */
    if timed(TMD_PARALYZED) > 50 {
        t[BI_ISPARALYZED] = 1;
    }

    /* Check for "Hallucinating" */
    if timed(TMD_IMAGE) != 0 {
        t[BI_ISIMAGE] = 1;
    }

    /* Check for "Amnesia" */
    if timed(TMD_AMNESIA) != 0 {
        t[BI_ISFORGET] = 1;
    }

    /* Check for "Study" */
    if p.new_spells != 0 {
        t[BI_ISSTUDY] = 1;
    }

    /* Parse stats */
    for i in 0..5u32 {
        let s = (STAT_STR + i) as usize;
        t[BI_ISFIXSTR + i] = i32::from(p.stat_cur[s] < p.stat_max[s]);
        t[BI_CSTR + i] = i32::from(p.stat_cur[s]);
    }

    /* Track if Sauron is dead Cheat */
    t[BI_SAURON_DEAD] = sauron_dead;
}

/// Allocate the trait arrays (`borg_trait_init`).
pub fn borg_trait_init(st: &mut BorgState) {
    borg_trait_free(st);

    let z = engine::z_info();
    st.borg.has = vec![0; usize::from(z.k_max)];
    st.borg.trait_ = vec![0; BI_MAX as usize];
    st.borg.activation = vec![0; usize::from(z.act_max)];
}

/// Free the trait arrays (`borg_trait_free`; empty = NULL).
pub fn borg_trait_free(st: &mut BorgState) {
    st.borg.has = Vec::new();
    st.borg.trait_ = Vec::new();
    st.borg.activation = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_pref_matches_the_enum() {
        assert_eq!(PREFIX_PREF[BI_STR as usize], b"str");
        assert_eq!(PREFIX_PREF[BI_CON_INDEX as usize], b"con index");
        assert_eq!(PREFIX_PREF[BI_SFRACT as usize], b"free action with swap");
        assert_eq!(PREFIX_PREF[BI_ISFIXALL as usize], b"is fixall");
        assert_eq!(PREFIX_PREF[BI_WHERE_CURSED as usize], b"where cursed");
        assert_eq!(PREFIX_PREF[BI_CRSUNKNO as usize], b"unknown curse");
        assert_eq!(PREFIX_PREF[BI_WB_POIS as usize], b"wep brand poison");
        assert_eq!(PREFIX_PREF[BI_PREP_BIG_FIGHT as usize], b"prep big fight");
    }

    #[test]
    fn stat_tables_have_38_entries_ending_as_in_c() {
        assert_eq!(BORG_ADJ_MAG_MANA[37], 800);
        assert_eq!(BORG_ADJ_CON_MHP[0], -250);
        assert_eq!(BORG_ADJ_MAG_STAT[37], 57);
        assert_eq!(BORG_ADJ_STR_WGT[15], 20);
        assert_eq!(BORG_BLOWS_TABLE[11][11], 15);
    }
}

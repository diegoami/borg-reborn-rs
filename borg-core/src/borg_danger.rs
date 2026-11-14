//! Port of `borg-danger.c`: determine how dangerous a square is.
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
//! Integer semantics: all of the danger arithmetic is C `int` (i32), with
//! C's truncating division; `ac` is an `int16_t`. The monster comes from
//! `borg_kills` (`st.kills`, borg-flow-kill.c), copied once per monster,
//! and its race from the engine's `r_info`.

use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid, borg_feature_protected};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_magic::{borg_spell_legal, borg_spell_stat};
use crate::borg_projection::{borg_distance, borg_projectable, borg_projectable_pure};
// the MONBLOW_* blow effects (borg-danger.h)
use crate::borg_headers::borg_danger_h::*;
use crate::borg_flow_kill::BorgKill;
use crate::borg_headers::borg_magic_h::{REMEMBRANCE, RESTORATION, REVITALIZE, UNHOLY_REPRIEVE};
use crate::borg_headers::borg_trait_h::{
    BI_AFUEL, BI_ARMOR, BI_CCON, BI_CDEPTH, BI_CDEX, BI_CINT, BI_CLASS, BI_CLEVEL, BI_CSTR,
    BI_CURHP, BI_CURSP, BI_CWIS, BI_DAM_RED, BI_DEX_INDEX, BI_FOOD, BI_FRACT, BI_GOLD, BI_HLIFE,
    BI_IACID, BI_ICOLD, BI_IELEC, BI_IFIRE, BI_IPOIS, BI_ISHEAVYSTUN, BI_ISSTUN, BI_MAXDEPTH,
    BI_MAXHP, BI_MAXSP, BI_RACID, BI_RBLIND, BI_RCOLD, BI_RCONF, BI_RDARK, BI_RDIS, BI_RELEC,
    BI_RFEAR, BI_RFIRE, BI_RKAOS, BI_RLITE, BI_RNTHR, BI_RNXUS, BI_RPOIS, BI_RSHRD, BI_RSND,
    BI_SAV, BI_SCON, BI_SDEX, BI_SINT, BI_SPEED, BI_SSTR, BI_SWIS,
};
use crate::headers::list_mon_race_flags_h::{
    RF_EVIL, RF_KILL_WALL, RF_MULTIPLY, RF_NEVER_BLOW, RF_NEVER_MOVE, RF_NO_SLEEP, RF_PASS_WALL,
    RF_UNIQUE,
};
// the RSF_* monster spells (list-mon-spells.h)
use crate::headers::list_mon_spells_h::*;
use crate::headers::list_object_flags_h::OF_NO_FUEL;
use crate::headers::list_stats_h::{STAT_INT, STAT_WIS};
use crate::headers::list_terrain_h::{
    FEAT_CLOSED, FEAT_GRANITE, FEAT_MAGMA, FEAT_MAGMA_K, FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K,
    FEAT_RUBBLE,
};
use crate::state::BorgState;
use crate::engine;

/// `CLASS_MAGE`
const CLASS_MAGE: i32 = 1;

/// Rows of `borg_fear_region` (`AUTO_MAX_Y / 11 + 1`).
pub const FEAR_REGION_HGT: usize = (AUTO_MAX_Y / 11 + 1) as usize;
/// Columns of `borg_fear_region` (`AUTO_MAX_X / 11 + 1`).
pub const FEAR_REGION_WID: usize = (AUTO_MAX_X / 11 + 1) as usize;
/// Rows of `borg_fear_monsters` (`AUTO_MAX_Y + 1`).
pub const FEAR_MONSTERS_HGT: usize = (AUTO_MAX_Y + 1) as usize;
/// Columns of `borg_fear_monsters` (`AUTO_MAX_X + 1`).
pub const FEAR_MONSTERS_WID: usize = (AUTO_MAX_X + 1) as usize;

/// borg-danger.c globals.
pub struct DangerState {
    /// `borg_fear_region`: extra fear per "region" (11x11 grids)
    pub fear_region: [[u16; FEAR_REGION_WID]; FEAR_REGION_HGT],
    /// `borg_fear_monsters`: extra fear per grid induced from extra
    /// monsters
    pub fear_monsters: [[u16; FEAR_MONSTERS_WID]; FEAR_MONSTERS_HGT],
    /// `borg_danger_wipe`: recalculate danger
    pub wipe: bool,
}

impl Default for DangerState {
    fn default() -> Self {
        DangerState {
            fear_region: [[0; FEAR_REGION_WID]; FEAR_REGION_HGT],
            fear_monsters: [[0; FEAR_MONSTERS_WID]; FEAR_MONSTERS_HGT],
            wipe: false,
        }
    }
}

/// The borg-trait.c globals the danger code reads (they do
/// not change during a danger calculation).
#[derive(Clone, Copy)]
struct SpellFlags {
    /// `borg_attacking`
    attacking: bool,
    /// `borg_on_glyph`
    on_glyph: bool,
    /// `borg_create_door`
    create_door: bool,
    /// `borg_sleep_spell`
    sleep_spell: bool,
    /// `borg_sleep_spell_ii`
    sleep_spell_ii: bool,
    /// `borg_crush_spell`
    crush_spell: bool,
    /// `borg_slow_spell`
    slow_spell: bool,
    /// `borg_confuse_spell`
    confuse_spell: bool,
    /// `borg_fear_mon_spell`
    fear_mon_spell: bool,
}

impl SpellFlags {
    fn read(st: &mut BorgState) -> Self {
        SpellFlags {
            attacking: st.traits.attacking,
            on_glyph: st.traits.on_glyph,
            create_door: st.traits.create_door,
            sleep_spell: st.traits.sleep_spell,
            sleep_spell_ii: st.traits.sleep_spell_ii,
            crush_spell: st.traits.crush_spell,
            slow_spell: st.traits.slow_spell,
            confuse_spell: st.traits.confuse_spell,
            fear_mon_spell: st.traits.fear_mon_spell,
        }
    }
}

/// `rf_has(r_info[kill->r_idx].flags, flag)`
fn rf(kill: &BorgKill, flag: u32) -> bool {
    engine::race_flag_has(usize::from(kill.r_idx), flag)
}

/// `INVEN_LIGHT` (`z_info->pack_size + 5`)
fn inven_light() -> usize {
    usize::from(engine::z_info().pack_size) + 5
}

/// Does protection from evil work against this monster?
fn pfe_applies(st: &BorgState, kill: &BorgKill) -> bool {
    let r_ptr = engine::race(usize::from(kill.r_idx));
    st.borg.temp.prot_from_evil && rf(kill, RF_EVIL) && st.borg.tr(BI_CLEVEL) >= r_ptr.level
}

/// Calculate base danger from a monster's physical attacks
/// (`borg_danger_physical`).
///
/// We attempt to take account of various resistances, both in
/// terms of actual damage, and special effects, as appropriate.
///
/// We reduce the danger from distant "sleeping" monsters.
/// PFE reduces my fear of an area.
fn borg_danger_physical(
    st: &mut BorgState,
    kill: &BorgKill,
    flags: SpellFlags,
    full_damage: bool,
) -> i32 {
    let mut n: i32 = 0;
    let mut pfe = false;

    let mut ac: i16 = st.borg.tr(BI_ARMOR) as i16;

    // shields gives +50 to ac and deflects some missiles and balls
    if st.borg.temp.shield {
        ac = (i32::from(ac) + 50) as i16;
    }

    // PFE gives a protection.
    // Apply PROTECTION_FROM_EVIL
    if pfe_applies(st, kill) {
        pfe = true;
    }

    // Mega-Hack -- unknown monsters (or "player ghosts"
    if kill.r_idx == 0 {
        return 1000;
    }
    if i32::from(kill.r_idx) >= i32::from(engine::z_info().r_max) - 1 {
        return 1000;
    }

    let r_idx = usize::from(kill.r_idx);
    let level = engine::race(r_idx).level;
    let half = !flags.attacking && pfe;

    // Analyze each physical attack
    for k in 0..usize::from(engine::z_info().mon_blows_max) {
        let mut z: i32 = 0;

        // Extract the attack infomation
        let Some(blow) = engine::race_blow(r_idx, k) else {
            // Done
            break;
        };
        let d_dice = blow.dice;
        let d_side = blow.sides;

        let mut power: i32 = 0;

        // Analyze the attack
        let effect = crate::borg_fight_attack::borg_mon_blow_effect(blow.effect_name.to_bytes()) as u32;
        match effect {
            MONBLOW_HURT => {
                z = d_dice * d_side;
                // fudge- some baddies kick and they tend to KO.  Avoid close
                // combat like the plague.  10d2 is common but take any very
                // low sides and high dice count as dangerous
                if d_side < 3 && d_dice > 5 {
                    n += 400;
                }
                power = 60;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_POISON => 'b: {
                z = d_dice * d_side;
                power = 5;
                if st.borg.tr(BI_RPOIS) != 0 {
                    break 'b;
                }
                if st.borg.temp.res_pois {
                    break 'b;
                }
                // Add fear for the effect
                z += 10;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_DISENCHANT => 'b: {
                z = d_dice * d_side;
                power = 20;
                if st.borg.tr(BI_RDIS) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += 500;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_DRAIN_CHARGES => {
                z = d_dice * d_side;
                // Add fear for the effect
                z += 20;
                power = 15;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_EAT_GOLD => 'b: {
                z = d_dice * d_side;
                // if in town and low level avoid them stupid urchins
                if st.borg.tr(BI_CLEVEL) < 5 {
                    z += 50;
                }
                power = 5;
                if 100 <= engine::adj_dex_safe(st.borg.tr(BI_DEX_INDEX) as usize)
                    + st.borg.tr(BI_CLEVEL)
                {
                    break 'b;
                }
                if st.borg.tr(BI_GOLD) < 100 {
                    break 'b;
                }
                if st.borg.tr(BI_GOLD) > 100000 {
                    break 'b;
                }
                // Add fear for the effect
                z += 5;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_EAT_ITEM => 'b: {
                z = d_dice * d_side;
                power = 5;
                if 100 <= engine::adj_dex_safe(st.borg.tr(BI_DEX_INDEX) as usize)
                    + st.borg.tr(BI_CLEVEL)
                {
                    break 'b;
                }
                // Add fear for the effect
                z += 5;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_EAT_FOOD => 'b: {
                z = d_dice * d_side;
                power = 5;
                if st.borg.tr(BI_FOOD) > 5 {
                    break 'b;
                }
                // Add fear for the effect
                z += 5;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_EAT_LIGHT => 'b: {
                z = d_dice * d_side;
                power = 5;
                let light = &st.items.items[inven_light()];
                if light.timeout == 0 || engine::flag_has(&light.flags, OF_NO_FUEL as i32) {
                    break 'b;
                }
                if st.borg.tr(BI_AFUEL) > 5 {
                    break 'b;
                }
                // Add fear for the effect
                z += 5;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_ACID => 'b: {
                if st.borg.tr(BI_IACID) != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                if st.borg.tr(BI_RACID) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_acid {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                z += 200; // We don't want our armour corroded.
                if half {
                    z /= 2;
                }
            }

            MONBLOW_ELEC => 'b: {
                if st.borg.tr(BI_IELEC) != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = 10;
                if st.borg.tr(BI_RELEC) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_elec {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                z *= 2;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_FIRE => 'b: {
                if st.borg.tr(BI_IFIRE) != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = 10;
                if st.borg.tr(BI_RFIRE) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_fire {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                z *= 2;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_COLD => 'b: {
                if st.borg.tr(BI_ICOLD) != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = 10;
                if st.borg.tr(BI_RCOLD) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_cold {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                z *= 2;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_BLIND => 'b: {
                z = d_dice * d_side;
                power = 2;
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += 10;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    z += 75;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_CONFUSE => 'b: {
                z = d_dice * d_side;
                power = 10;
                if st.borg.tr(BI_RCONF) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += 200;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    z += 200;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_TERRIFY => 'b: {
                z = d_dice * d_side;
                power = 10;
                if st.borg.tr(BI_RFEAR) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z *= 2;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_PARALYZE => 'b: {
                z = d_dice * d_side;
                power = 2;
                if st.borg.tr(BI_FRACT) != 0 {
                    break 'b;
                }
                z += 200;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_STR => 'b: {
                z = d_dice * d_side;
                if st.borg.tr(BI_SSTR) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_CSTR) <= 3 {
                    break 'b;
                }
                if borg_spell_legal(st, RESTORATION) {
                    break 'b;
                }
                if borg_spell_legal(st, REVITALIZE) {
                    break 'b;
                }
                if borg_spell_legal(st, UNHOLY_REPRIEVE) {
                    break 'b;
                }
                z += 150;
                // extra scary to have str drain below 10
                if st.borg.tr(BI_CSTR) < 10 {
                    z += 100;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_DEX => 'b: {
                z = d_dice * d_side;
                if st.borg.tr(BI_SDEX) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_CDEX) <= 3 {
                    break 'b;
                }
                if borg_spell_legal(st, RESTORATION) {
                    break 'b;
                }
                if borg_spell_legal(st, REVITALIZE) {
                    break 'b;
                }
                z += 150;
                // extra scary to have drain below 10
                if st.borg.tr(BI_CDEX) < 10 {
                    z += 100;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_CON => 'b: {
                z = d_dice * d_side;
                if st.borg.tr(BI_SCON) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_CCON) <= 3 {
                    break 'b;
                }
                if borg_spell_legal(st, RESTORATION) {
                    break 'b;
                }
                if borg_spell_legal(st, REVITALIZE) {
                    break 'b;
                }
                if borg_spell_legal(st, UNHOLY_REPRIEVE) {
                    break 'b;
                }
                // Add fear for the effect
                z += 150;
                // extra scary to have con drain below 8
                if st.borg.tr(BI_CCON) < 8 {
                    z += 100;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_INT => 'b: {
                z = d_dice * d_side;
                if st.borg.tr(BI_SINT) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_CINT) <= 3 {
                    break 'b;
                }
                if borg_spell_legal(st, RESTORATION) {
                    break 'b;
                }
                if borg_spell_legal(st, REVITALIZE) {
                    break 'b;
                }
                if borg_spell_legal(st, UNHOLY_REPRIEVE) {
                    break 'b;
                }
                z += 150;
                // extra scary for spell caster
                if borg_spell_stat() == STAT_INT as i32 {
                    z += 50;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_WIS => 'b: {
                z = d_dice * d_side;
                if st.borg.tr(BI_SWIS) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_CWIS) <= 3 {
                    break 'b;
                }
                if borg_spell_legal(st, RESTORATION) {
                    break 'b;
                }
                if borg_spell_legal(st, REVITALIZE) {
                    break 'b;
                }
                z += 150;
                // extra scary for pray'er
                if borg_spell_stat() == STAT_WIS as i32 {
                    z += 50;
                }
                if half {
                    z /= 2;
                }
            }

            MONBLOW_LOSE_ALL => {
                z = d_dice * d_side;
                power = 2;
                // only morgoth to make it easier to fight him
            }

            MONBLOW_SHATTER => {
                z = d_dice * d_side;
                z -= z * i32::from(if ac < 150 { ac } else { 150 }) / 250;
                power = 60;
                // Add fear for the effect
                z += 150;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_EXP_10 | MONBLOW_EXP_20 | MONBLOW_EXP_40 | MONBLOW_EXP_80 => 'b: {
                let (fear, at_50) = match effect {
                    // do not worry about drain exp after level 50 (EXP_10
                    // tests `== 50`, the others `>= 50`)
                    MONBLOW_EXP_10 => (100, st.borg.tr(BI_CLEVEL) == 50),
                    MONBLOW_EXP_20 => (150, st.borg.tr(BI_CLEVEL) >= 50),
                    MONBLOW_EXP_40 => (200, st.borg.tr(BI_CLEVEL) >= 50),
                    _ => (250, st.borg.tr(BI_CLEVEL) >= 50),
                };
                z = d_dice * d_side;
                if st.borg.tr(BI_HLIFE) != 0 {
                    break 'b;
                }
                if at_50 {
                    break 'b;
                }
                if borg_spell_legal(st, REMEMBRANCE)
                    || borg_spell_legal(st, UNHOLY_REPRIEVE)
                    || borg_spell_legal(st, REVITALIZE)
                {
                    break 'b;
                }
                // Add fear for the effect
                z += fear;
                if half {
                    z /= 2;
                }
            }

            MONBLOW_HALLU => {
                z = d_dice * d_side;
                // Add fear for the effect
                z += 250;
                if half {
                    z /= 2;
                }
            }

            _ => {}
        }

        // reduce by damage reduction
        z -= st.borg.tr(BI_DAM_RED);
        if z < 0 {
            z = 0;
        }

        // if we are doing partial damage reduce for % chance that it will
        // hit you.
        if !full_damage {
            // figure out chance that monster will hit you.
            // add a 50% bonus in to account for bad luck.
            let chance = if (level + power) > 0 {
                150 - (((i32::from(ac) * 300) / 4) / ((level + power) * 3))
            } else {
                -1
            };

            // always have a 5% chance of hitting.
            let chance = if chance < 5 { 5 } else { chance };
            // never estimate more than the full damage of the blow.
            let chance = if chance > 100 { 100 } else { chance };

            // Modify the damage by the chance of getting hit
            z = (z * chance) / 100;
        }

        // Add in damage
        n += z;
    }

    // Danger
    n
}

/// The summon cases' count of the protected grids around the monster
/// (C repeats this loop in every summon case). `spot_safe` is shared by
/// all the spells of one monster, as in C. (C also leaves the function's
/// `x`/`y` parameters at the last grid scanned; they are not read again.)
fn count_safe_spots(st: &BorgState, kill: &BorgKill, spot_safe: &mut i32) {
    for spot_x in -1..=1 {
        for spot_y in -1..=1 {
            // Acquire location
            let x = spot_x + kill.pos.x;
            let y = spot_y + kill.pos.y;

            let ag: &BorgGrid = st.cave.grid(x, y);

            // skip our own spot
            if x == kill.pos.x && y == kill.pos.y {
                continue;
            }

            // track spaces already protected
            if borg_feature_protected(ag) {
                // track the safe areas for calculating danger
                *spot_safe += 1;

                // Just in case
                if *spot_safe == 0 {
                    *spot_safe = 1;
                }
                if *spot_safe == 8 {
                    *spot_safe = 100;
                }
                if st.borg.morgoth_position || st.borg.status.anti_summon {
                    *spot_safe = 1000;
                }
            }
        }
    }
}

/// Calculate base danger from a monster's spell attacks
/// (`borg_danger_spell`).
///
/// We attempt to take account of various resistances, both in
/// terms of actual damage, and special effects, as appropriate.
///
/// We reduce the danger from distant "sleeping" monsters.
///
/// We reduce the danger if the monster is immobile or not LOS
fn borg_danger_spell(
    st: &mut BorgState,
    kill: &BorgKill,
    flags: SpellFlags,
    y: i32,
    x: i32,
    d: i32,
    average: bool,
) -> i32 {
    let mut n: i32 = 0;
    let mut glyph = false;
    let mut spot_safe: i32 = 1;
    let mut total_dam: i32 = 0;

    // PFE gives a protection.
    // Apply PROTECTION_FROM_EVIL
    let pfe = pfe_applies(st, kill);

    // glyph of warding rune of protection provides some small
    // protection with some ranged atacks; mainly summon attacks.
    // We should reduce the danger commensurate to the probability of the
    // monster breaking the glyph as defined by melee2.c
    if flags.on_glyph {
        glyph = true;
    } else {
        let num = st.glyph.track.num;
        if num != 0 {
            // Check all existing glyphs
            for glyph_check in 0..i32::from(num) {
                let (gy, gx) = st.glyph.track.at(glyph_check as usize);
                if gy == y && gx == x {
                    // Reduce the danger
                    glyph = true;
                }
            }
        }
    }

    // Mega-Hack -- unknown monsters (or "player ghosts")
    if kill.r_idx == 0 {
        return 1000;
    }
    if i32::from(kill.r_idx) >= i32::from(engine::z_info().r_max) - 1 {
        return 1000;
    }

    // Paranoia -- Nothing to cast
    if kill.ranged_attack == 0 {
        return 0;
    }

    let spell_power = engine::race(usize::from(kill.r_idx)).spell_power;
    let unique = rf(kill, RF_UNIQUE);
    let near_unique = st.borg.near.unique != 0;
    let create_door = flags.create_door;

    // Extract hit-points
    let hp = i32::from(kill.power);

    // Analyze the spells
    for q in 0..usize::from(kill.ranged_attack) {
        let mut bolt = false;
        let mut p: i32 = 0;
        let mut z: i32 = 0;

        // Cast the spell.
        match u32::from(kill.spell[q]) {
            RSF_SHRIEK => {
                // Add fear for the effect
                p += 5;
            }

            RSF_WHIP => {
                if d < 3 {
                    z = 100;
                }
            }

            RSF_SPIT => {
                if d < 4 {
                    z = 100;
                }
            }

            RSF_SHOT => {
                z = ((spell_power / 8) + 1) * 5;
            }

            RSF_ARROW => {
                z = ((spell_power / 8) + 1) * 6;
            }

            RSF_BOLT => {
                z = ((spell_power / 8) + 1) * 7;
            }

            RSF_BR_ACID => 'b: {
                if st.borg.tr(BI_IACID) != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.borg.tr(BI_RACID) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_acid {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BR_ELEC => 'b: {
                if st.borg.tr(BI_IELEC) != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.borg.tr(BI_RELEC) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_elec {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BR_FIRE => 'b: {
                if st.borg.tr(BI_IFIRE) != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.borg.tr(BI_RFIRE) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_fire {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BR_COLD => 'b: {
                if st.borg.tr(BI_ICOLD) != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.borg.tr(BI_RCOLD) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_cold {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BR_POIS => 'b: {
                z = hp / 3;
                // max damage
                if z > 800 {
                    z = 800;
                }
                if st.borg.tr(BI_RPOIS) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_pois {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_pois {
                    break 'b;
                }
                if st.borg.tr(BI_RPOIS) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BR_NETH => 'b: {
                z = hp / 6;
                // max damage
                if z > 600 {
                    z = 600;
                }
                if st.borg.tr(BI_RNTHR) != 0 {
                    z = (z * 6) / 9;
                    break 'b;
                }
                // Add fear for the effect
                p += 125;
            }

            RSF_BR_LIGHT => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.borg.tr(BI_RLITE) != 0 {
                    z = (z * 2) / 3;
                    break 'b;
                }
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                p += 20;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_BR_DARK => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.borg.tr(BI_RDARK) != 0 {
                    z = (z * 2) / 3;
                }
                if st.borg.tr(BI_RDARK) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                p += 20;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_BR_SOUN => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.borg.tr(BI_RSND) != 0 {
                    z = (z * 5) / 9;
                }
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 1000;
                }
                // Add fear for the effect
                p += 50;
            }

            RSF_BR_CHAO => 'b: {
                z = hp / 6;
                // max damage
                if z > 600 {
                    z = 600;
                }
                if st.borg.tr(BI_RKAOS) != 0 {
                    z = (z * 6) / 9;
                }
                // Add fear for the effect
                p += 100;
                if st.borg.tr(BI_RKAOS) != 0 {
                    break 'b;
                }
                p += 200;
            }

            RSF_BR_DISE => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.borg.tr(BI_RDIS) != 0 {
                    z = (z * 6) / 10;
                }
                if st.borg.tr(BI_RDIS) != 0 {
                    break 'b;
                }
                p += 500;
            }

            RSF_BR_NEXU => 'b: {
                z = hp / 6;
                // max damage
                if z > 400 {
                    z = 400;
                }
                if st.borg.tr(BI_RNXUS) != 0 {
                    z = (z * 6) / 10;
                }
                if st.borg.tr(BI_RNXUS) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 100;
            }

            RSF_BR_TIME => {
                z = hp / 3;
                // max damage
                if z > 150 {
                    z = 150;
                }
                // Add fear for the effect
                p += 250;
            }

            RSF_BR_INER => {
                z = hp / 6;
                // max damage
                if z > 200 {
                    z = 200;
                }
                // Add fear for the effect
                p += 100;
            }

            RSF_BR_GRAV => 'b: {
                z = hp / 3;
                // max damage
                if z > 200 {
                    z = 200;
                }
                // Add fear for the effect
                p += 100;
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 1000;
                }
            }

            RSF_BR_SHAR => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.borg.tr(BI_RSHRD) != 0 {
                    z = (z * 6) / 9;
                }
                if st.borg.tr(BI_RSHRD) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 50;
            }

            RSF_BR_PLAS => 'b: {
                z = hp / 6;
                // max damage
                if z > 150 {
                    z = 150;
                }
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // Pump this up if you have goi so that the borg is sure
                // to be made nervous
                p += 100;
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 1000;
                }
            }

            RSF_BR_WALL => 'b: {
                z = hp / 6;
                // max damage
                if z > 200 {
                    z = 200;
                }
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 100;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 500;
                }
                // Add fear for the effect
                p += 50;
            }

            RSF_BR_MANA => {
                // Nothing currently breaths mana but best to have a check
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
            }

            RSF_BOULDER => {
                z = (1 + spell_power / 7) * 12;
                bolt = true;
            }

            RSF_WEAVE => {
                // annoying creation of traps
            }

            RSF_BA_ACID => 'b: {
                if st.borg.tr(BI_IACID) != 0 {
                    break 'b;
                }
                z = (spell_power * 3) + 15;
                if st.borg.tr(BI_RACID) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_acid {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BA_ELEC => 'b: {
                if st.borg.tr(BI_IELEC) != 0 {
                    break 'b;
                }
                z = (spell_power * 3) / 2 + 8;
                if st.borg.tr(BI_RELEC) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_elec {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BA_FIRE => 'b: {
                if st.borg.tr(BI_IFIRE) != 0 {
                    break 'b;
                }
                z = (spell_power * 7) / 2 + 10;
                if st.borg.tr(BI_RFIRE) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_fire {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BA_COLD => 'b: {
                if st.borg.tr(BI_ICOLD) != 0 {
                    break 'b;
                }
                z = (spell_power * 3 / 2) + 10;
                if st.borg.tr(BI_RCOLD) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_cold {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BA_POIS => 'b: {
                z = (spell_power / 2 + 3) * 4;
                if st.borg.tr(BI_RPOIS) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_pois {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_pois {
                    break 'b;
                }
                if st.borg.tr(BI_RPOIS) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BA_SHAR => 'b: {
                z = ((spell_power * 3) / 2) + 10;
                if st.borg.tr(BI_RSHRD) != 0 {
                    z = (z * 6) / 9;
                }
                if st.borg.tr(BI_RSHRD) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BA_NETH => 'b: {
                z = (spell_power * 4) + (10 * 10);
                if st.borg.tr(BI_RNTHR) != 0 {
                    z = (z * 6) / 8;
                }
                if st.borg.tr(BI_RNTHR) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 250;
            }

            RSF_BA_WATE => 'b: {
                z = (spell_power * 5) / 2 + 50;
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    p += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    p += 1000;
                }
                if st.borg.tr(BI_RCONF) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 50;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_BA_MANA => {
                z = (spell_power * 5) + (10 * 10);
                // Add fear for the effect
                p += 50;
            }

            RSF_BA_HOLY => {
                z = 10 + ((spell_power * 3) / 2 + 1) / 2;
                // Add fear for the effect
                p += 50;
            }

            RSF_BA_DARK => 'b: {
                z = (spell_power * 4) + (10 * 10);
                if st.borg.tr(BI_RDARK) != 0 {
                    z = (z * 6) / 9;
                }
                if st.borg.tr(BI_RDARK) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_BA_LIGHT => 'b: {
                z = 10 + (spell_power * 3 / 2);
                if st.borg.tr(BI_RLITE) != 0 {
                    z = (z * 6) / 9;
                }
                if st.borg.tr(BI_RLITE) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_STORM => 'b: {
                z = 70 + (spell_power * 5);
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    p += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    p += 1000;
                }
                // (C: `if (borg.trait[BI_RCONF]) break;` here, the last
                // statement of the case)
            }

            RSF_DRAIN_MANA => {
                if st.borg.tr(BI_MAXSP) != 0 {
                    p += 100;
                }
            }

            RSF_MIND_BLAST => {
                if st.borg.tr(BI_SAV) < 100 {
                    z = spell_power / 2 + 1;
                }
            }

            RSF_BRAIN_SMASH => {
                z = (12 * (15 + 1)) / 2;
                p += 200 - 2 * st.borg.tr(BI_SAV);
                if p < 0 {
                    p = 0;
                }
            }

            RSF_WOUND => 'b: {
                if st.borg.tr(BI_SAV) >= 100 {
                    break 'b;
                }
                z = (spell_power / 3 * 2) * 5;
                // reduce by % chance of save  (add 20% for fudge)
                z = z * (120 - st.borg.tr(BI_SAV)) / 100;
            }

            RSF_BO_ACID => 'b: {
                bolt = true;
                if st.borg.tr(BI_IACID) != 0 {
                    break 'b;
                }
                z = (7 * 8) + (spell_power / 3);
                if st.borg.tr(BI_RACID) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_acid {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BO_ELEC => 'b: {
                if st.borg.tr(BI_IELEC) != 0 {
                    break 'b;
                }
                bolt = true;
                z = (4 * 8) + (spell_power / 3);
                if st.borg.tr(BI_RELEC) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_elec {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BO_FIRE => 'b: {
                if st.borg.tr(BI_IFIRE) != 0 {
                    break 'b;
                }
                bolt = true;
                z = (9 * 8) + (spell_power / 3);
                if st.borg.tr(BI_RFIRE) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_fire {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 40;
            }

            RSF_BO_COLD => 'b: {
                if st.borg.tr(BI_ICOLD) != 0 {
                    break 'b;
                }
                bolt = true;
                z = (6 * 8) + (spell_power / 3);
                if st.borg.tr(BI_RCOLD) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_cold {
                    z = (z + 2) / 3;
                }
                // Add fear for the effect
                p += 20;
            }

            RSF_BO_POIS => 'b: {
                if st.borg.tr(BI_IPOIS) != 0 {
                    break 'b;
                }
                z = (9 * 8) + (spell_power / 3);
                if st.borg.tr(BI_RPOIS) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_pois {
                    z = (z + 2) / 3;
                }
                bolt = true;
            }

            RSF_BO_NETH => 'b: {
                bolt = true;
                z = (5 * 5) + (spell_power * 3 / 2) + 50;
                if st.borg.tr(BI_RNTHR) != 0 {
                    z = (z * 6) / 8;
                }
                if st.borg.tr(BI_RNTHR) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 200;
            }

            RSF_BO_WATE => 'b: {
                z = (10 * 10) + spell_power;
                bolt = true;
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    p += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    p += 1000;
                }
                if st.borg.tr(BI_RCONF) != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += 20;
                if st.borg.tr(BI_CLASS) == CLASS_MAGE {
                    p += 20;
                }
            }

            RSF_BO_MANA => {
                z = (spell_power * 5) / 2 + 50;
                bolt = true;
                // Add fear for the effect
                p += 50;
            }

            RSF_BO_PLAS => 'b: {
                z = 10 + (8 * 7) + spell_power;
                bolt = true;
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 500;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 1000;
                }
            }

            RSF_BO_ICE => 'b: {
                z = (6 * 6) + spell_power;
                bolt = true;
                // Add fear for the effect
                p += 20;
                if st.borg.tr(BI_RSND) != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.borg.tr(BI_ISSTUN) != 0 {
                    z += 50;
                }
                if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
                    z += 1000;
                }
            }

            RSF_MISSILE => {
                z = (2 * 6) + (spell_power / 3);
                bolt = true;
            }

            RSF_BE_ELEC => 'b: {
                if st.borg.tr(BI_IELEC) != 0 {
                    break 'b;
                }
                z = (5 * 5) + (spell_power * 2) + 30;
                if st.borg.tr(BI_RELEC) != 0 {
                    z = (z + 2) / 3;
                }
                if st.borg.temp.res_elec {
                    z = (z + 2) / 3;
                }
                bolt = true;
            }

            RSF_BE_NETH => {
                bolt = true;
                z = (5 * 5) + (spell_power * 2) + 30;
                if st.borg.tr(BI_RNTHR) != 0 {
                    z = (z * 6) / 8;
                }
            }

            RSF_SCARE => {
                if st.borg.tr(BI_SAV) < 100 {
                    // Add fear for the effect
                    p += 10;
                }
            }

            RSF_BLIND => 'b: {
                if st.borg.tr(BI_RBLIND) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_SAV) >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += 10;
            }

            RSF_CONF => 'b: {
                if st.borg.tr(BI_RCONF) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_SAV) >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += 10;
            }

            RSF_SLOW => 'b: {
                if st.borg.tr(BI_FRACT) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_SAV) >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += 5;
            }

            RSF_HOLD => 'b: {
                if st.borg.tr(BI_FRACT) != 0 {
                    break 'b;
                }
                if st.borg.tr(BI_SAV) >= 100 {
                    break 'b;
                }
                p += 150;
            }

            RSF_HASTE => {
                // Add fear for the effect
                p += 10;
            }

            RSF_HEAL => {
                // Add fear for the effect
                p += 10;
            }

            RSF_HEAL_KIN => {}

            RSF_BLINK => {}

            RSF_TPORT => {
                // Add fear for the effect
                p += 10;
            }

            RSF_TELE_TO => {
                // Add fear for the effect
                p += 20;
            }

            RSF_TELE_SELF_TO => {
                // Add fear for the effect
                p += 20;
            }

            RSF_TELE_AWAY => {
                // Add fear for the effect
                p += 10;
            }

            RSF_TELE_LEVEL => {
                if st.borg.tr(BI_SAV) < 100 {
                    // Add fear for the effect
                    p += 50;
                }
            }

            RSF_DARKNESS => {
                // Add fear for the effect
                p += 5;
            }

            RSF_TRAPS => {
                // Add fear for the effect
                p += 50;
            }

            RSF_FORGET => {
                if st.borg.tr(BI_SAV) < 100 {
                    // if you are a spell caster, this is a big issue
                    if st.borg.tr(BI_CURSP) < 15 {
                        p += 500;
                    } else {
                        p += 30;
                    }
                }
            }

            RSF_SHAPECHANGE => {
                // Add fear for the effect
                p += 200;
            }

            // Summoning is only as dangerous as the monster that is
            // actually summoned but the monsters that summon are a priority
            // to kill.  PFE reduces danger from some evil summoned monsters
            // One Problem with GOI and Create Door is that the GOI reduces
            // the fear so much that the borg won't cast the Create Door,
            // even though it would be a good idea.
            RSF_S_KIN => {
                // This is used to calculate the free squares next to us.
                // This is important when dealing with summoners.
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 3;
                    p /= spot_safe;
                } else {
                    p += spell_power * 7;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_HI_DEMON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 6;
                    p /= spot_safe;
                } else {
                    p += spell_power * 12;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_MONSTER => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * 5;
                    p /= spot_safe;
                }
            }

            RSF_S_MONSTERS => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * 7;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_ANIMAL | RSF_S_SPIDER | RSF_S_HOUND => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * 5;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_HYDRA => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 2;
                    p /= spot_safe;
                } else {
                    p += spell_power * 5;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_AINU => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || near_unique {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 3;
                    p /= spot_safe;
                } else {
                    p += spell_power * 7;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_DEMON | RSF_S_UNDEAD | RSF_S_DRAGON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 3;
                    p /= spot_safe;
                } else {
                    p += spell_power * 7;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = (p * 75) / 100;
                }
            }

            RSF_S_HI_UNDEAD | RSF_S_WRAITH => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 6;
                    p /= spot_safe;
                } else {
                    p += spell_power * 12;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_HI_DRAGON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    // (C: no `p += spell_power` here, unlike the others)
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * 6;
                    p /= spot_safe;
                } else {
                    p += spell_power * 12;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            RSF_S_UNIQUE => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power;
                    p /= spot_safe;
                } else if glyph || create_door {
                    p += spell_power * 3; // slightly reduced danger for unique
                    p /= spot_safe;
                } else {
                    p += spell_power * 6;
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * 75 / 100;
                }
            }

            _ => {}
        }

        // A bolt spell cannot jump monsters to hit the borg.
        if bolt && !borg_projectable_pure(st, kill.pos.y, kill.pos.x, st.borg.c.y, st.borg.c.x)
        {
            z = 0;
        }

        // Some borgs are concerned with the 'effects' of an attack.  ie, cold
        // attacks shatter potions, fire attacks burn scrolls, electric attacks
        // zap rings.
        if st.borg.tr(BI_MAXDEPTH) >= 75 {
            p = 0;
        }

        // Notice damage
        p += z;

        // Track the most dangerous spell
        if p > n {
            n = p;
        }

        // Track the damage of all the spells, used in averaging
        total_dam += p;
    }

    // reduce by damage reduction
    total_dam -= st.borg.tr(BI_DAM_RED);
    if total_dam < 0 {
        total_dam = 0;
    }

    // Slightly decrease the danger if the borg is sitting in
    // a sea of runes.
    if st.borg.morgoth_position || st.borg.status.anti_summon {
        total_dam = total_dam * 7 / 10;
    }

    // Average damage of all the spells & compare to most dangerous spell
    let av = total_dam / i32::from(kill.ranged_attack);

    // If the most dangerous spell is a lot bigger than the average,
    // then return the dangerous one.
    //
    // There is a problem when dealing with defense maneuvers.
    // If the borg is considering casting a spell like
    // Resistance and the monster also has a non
    // resistible attack (like Disenchant) then the damage
    // returned will be for that spell, since the danger of the
    // others (like fire, cold) will be greatly reduced by the
    // proposed defense spell.  The result will be the borg will
    // not cast the resistance spell even though it may be a very
    // good idea.
    //
    // Example: a monster has three breath attacks (Fire, Ice,
    // Disenchant) and each hits for 800 pts of damage.  The
    // borg currently resists all three, so the danger would be
    // 500. If the borg were to use a Res Heat Potion that would
    // decrease the danger to:
    // Fire:  333
    // Ice:   500
    // Disen: 500
    // Now the Average is 444.  Not really worth it, nominal change.
    // But if the resistance spell was both Fire and Ice, then
    // it would be:
    // Fire:  333
    // Ice:   333
    // Disen: 500
    // With an average of 388. Probably worth it, but the borg
    // would see that the Disen attack is a quite dangerous and
    // would return the result of 500.
    //
    // To fix this, the flag 'average' is added to the
    // borg_danger() to skip this check and return the average
    // damage.  If the flag is false then the formula below is
    // SKIPPED and the value returned with be the average.
    // If the flag is true, then the formula below will be used
    // to determine the returned value.  Currently the elemental
    // resistance spells and PFE have the flag set as false.
    if !average {
        return av;
    }
    if n >= av * 15 / 10 || n > st.borg.tr(BI_CURHP) * 8 / 10 {
        n
    } else {
        // Average Danger
        av
    }
}

/// Is the grid a wall/seam grid for the move simulation of
/// `borg_danger_one_kill`? Both loops test `feat == FEAT_GRANITE`: features
/// above granite (permanent wall, lava, passable rubble) are not walls here.
fn wall_like(feat: u32) -> bool {
    feat == FEAT_GRANITE
        || feat == FEAT_MAGMA
        || feat == FEAT_QUARTZ
        || feat == FEAT_MAGMA_K
        || feat == FEAT_QUARTZ_K
        || feat == FEAT_RUBBLE
}

/// The monster's sleep 2 spell condition on its level (v1 part).
fn sleepable(st: &BorgState, kill: &BorgKill) -> bool {
    !rf(kill, RF_NO_SLEEP)
        && !rf(kill, RF_UNIQUE)
        && i32::from(kill.level) <= st.borg.tr(BI_CLEVEL) - 15
}

/// Calculate the danger to a grid from a monster (`borg_danger_one_kill`).
///
/// Note that we are paranoid, especially about "monster speed",
/// since even if a monster is slower than us, it will occasionally
/// get one full turn to attack us.
///
/// Note that we assume that monsters can walk through walls and
/// other monsters to get to the player.  XXX XXX XXX
///
/// This function attempts to consider possibilities such as movement plus
/// spell attacks, physical attacks and spell attacks together,
/// and other similar situations.
///
/// Currently we assume that "sleeping" monsters are less dangerous
/// unless you get near them, which may wake them up.
///
/// We attempt to take into account things like monsters which sometimes
/// "stumble", and monsters which only "sometimes" use powerful spells.
pub fn borg_danger_one_kill(
    st: &mut BorgState,
    y: i32,
    x: i32,
    c: i32,
    i: i32,
    average: bool,
    full_damage: bool,
) -> i32 {
    let kill = st.kills.kills[i as usize];
    danger_one_kill(st, &kill, y, x, c, i, average, full_damage)
}

/// `borg_danger_one_kill()` on a copy of `borg_kills[i]`.
#[allow(clippy::too_many_arguments)]
fn danger_one_kill(
    st: &mut BorgState,
    kill: &BorgKill,
    y: i32,
    x: i32,
    c: i32,
    i: i32,
    average: bool,
    full_damage: bool,
) -> i32 {
    let x9 = kill.pos.x;
    let y9 = kill.pos.y;

    let mut q: i32;
    let mut v1: i32;
    let mut v2: i32;

    let mut fake_speed: i32 = st.borg.tr(BI_SPEED);
    let mut monster_speed: i32 = i32::from(kill.speed);

    // Paranoia
    if kill.r_idx == 0 {
        return 0;
    }

    // "player ghosts"
    if i32::from(kill.r_idx) >= i32::from(engine::z_info().r_max) - 1 {
        return 100;
    }

    let r_ptr = engine::race(usize::from(kill.r_idx));
    let flags = SpellFlags::read(st);

    // Skip certain monster indexes.
    // These have been listed mainly in Teleport Other
    // checks in borg6.c in the defense maneuvers.
    let tp_other_n = st.fight_attack.tp_other_n;
    if tp_other_n != 0 {
        for ii in 0..i32::from(tp_other_n) {
            // Is the current danger check same as a saved monster index?
            if i == st.fight_attack.tp_other_index[ii as usize] {
                return 0;
            }
        }
    }

    // Distance components
    let ax = if x9 > x { x9 - x } else { x - x9 };
    let ay = if y9 > y { y9 - y } else { y - y9 };

    // Distance
    let mut d = ax.max(ay);

    // Minimal distance
    if d < 1 {
        d = 1;
    }

    // Minimal distance
    if d > 20 {
        return 0;
    }

    // A very speedy borg will miscalculate danger of some monsters
    if st.borg.tr(BI_SPEED) >= 135 {
        fake_speed = if st.borg.near.unique != 0 { 120 } else { 125 };
    }

    // Consider the character haste and slow monster spells
    if st.borg.temp.fast {
        fake_speed += 10;
    }
    if flags.slow_spell {
        monster_speed -= 10;
    }

    // Assume monsters are a little fast when you are low level
    if st.borg.tr(BI_MAXHP) < 20 && st.borg.tr(BI_CDEPTH) != 0 {
        monster_speed += 3;
    }

    // Player energy per game turn
    let mut e = i32::from(engine::extract_energy(fake_speed as usize));

    // Game turns per player move
    let t = (100 + (e - 1)) / e;

    // Monster energy per game turn
    e = i32::from(engine::extract_energy(monster_speed as usize));

    // Monster moves
    q = c * ((t * e) / 10);

    // allow partial hits when not calculating full possible damage
    if full_damage {
        q = ((q + 9) / 10) * 10;
    }

    // Minimal energy.  Monsters get at least some energy.
    // If the borg is very fast relative to a monster, then the
    // monster danger is artificially low due to the way the borg
    // will calculate the danger and energy.  So the monsters must
    // be given some base energy to equate the borg's.
    // ie:  the borg with speed +40 (speed = 150) is attacking
    // a monster with normal speed (speed = 110).  One would
    // think that the borg gets 4 attacks per turn over the monster.
    // and this does happen.  What if the monster can deal out
    // 1000 damage pts per monster attack turn?  The borg will
    // reduce the danger to 250 because the borg is 4x faster
    // than the monster.  But eventually the borg will get hit
    // by that 1000 pt damage attack.  And when it does, its
    // going to hurt.
    // So we make sure the monster is at least as fast as us.
    // But the monster is allowed to be faster than us.
    if q <= 10 {
        q = 10;
    }

    // Danger from physical attacks

    // Physical attacks
    v1 = borg_danger_physical(st, kill, flags, full_damage);

    // If the Borg has been bouncing for a long time, or if the time this
    // level is very high, reduce the danger value. This helps prevent the
    // borg from getting stuck due to overestimating danger and refusing to
    // move in both this area of the dungeon and over the course of the game.
    if st.borg.antibounce_count > 1200 || st.borg.time.now - st.borg.time.level > 25000 {
        v1 /= 5;
    }

    // No attacks for some monsters
    if rf(kill, RF_NEVER_BLOW) {
        v1 = 0;
    }

    // No movement for some monsters
    if rf(kill, RF_NEVER_MOVE) && d > 1 {
        v1 = 0;
    }

    // multipliers yield some trouble when I am weak
    if rf(kill, RF_MULTIPLY) && st.borg.tr(BI_CLEVEL) < 20 {
        // extra 50%
        v1 += v1 * 15 / 10;
    }

    let has_friends = !r_ptr.friends.is_null() || !r_ptr.friends_base.is_null();

    // Friends yield some trouble when I am weak
    if has_friends && st.borg.tr(BI_CLEVEL) < 20 {
        if st.borg.tr(BI_CLEVEL) < 15 {
            // extra 80%
            v1 += v1 * 18 / 10;
        } else {
            // extra 30%
            v1 += v1 * 13 / 10;
        }
    }

    // Reduce danger from sleeping monsters
    if !kill.awake {
        let inc = r_ptr.sleep + 5;
        // Reduce the fear if Borg is higher level
        if st.borg.tr(BI_CLEVEL) >= 25 {
            v1 /= 2;
        }

        // Tweak danger based on the "alertness" of the monster
        // increase the danger for light sleepers
        v1 += v1 * inc / 100;
    }
    // Reduce danger from sleeping monsters with the sleep 2 spell
    if flags.sleep_spell_ii && d == 1 && kill.awake && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.borg.tr(BI_CLEVEL) < 20 && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2 {
            v1 = 0;
        } else {
            v1 /= 3;
        }
    }
    // Reduce danger from sleeping monsters with the sleep 1,3 spell
    if flags.sleep_spell && kill.awake && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.borg.tr(BI_CLEVEL) < 20 && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2 {
            v1 = 0;
        } else {
            v1 /= d + 2;
        }
    }
    if flags.crush_spell && crushable(st, kill) {
        v1 = 0;
    }

    // Reduce danger from confused monsters
    if kill.confused {
        v1 /= 2;
    }
    if kill.stunned {
        v1 = v1 * 10 / 13;
    }
    if flags.confuse_spell && kill.awake && !kill.confused && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.borg.tr(BI_CLEVEL) < 20 && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2 {
            v1 = 0;
        } else {
            v1 /= d + 2;
        }
    }
    // Perceive a reduce danger from scared monsters
    if flags.fear_mon_spell {
        v1 = 0;
    }

    // Physical attacks require proximity
    //
    // Note that we do try to consider a fast monster moving and attacking
    // in the same round.  We should consider monsters that have a speed 2 or
    // 3 classes higher than ours, but most times, the borg will only
    // encounter monsters with a single category higher speed.
    if q > 10 && d != 1 && !rf(kill, RF_NEVER_MOVE) {
        let mut b_v1 = 0;

        // Check for a single grid movement, simulating the monster's move
        // action.
        for ii in 0..8 {
            // Obtain a grid to which the monster might move
            let y_temp = y9 + i32::from(engine::ddy_ddd(ii));
            let x_temp = x9 + i32::from(engine::ddx_ddd(ii));

            // Check for legality
            if !engine::square_in_bounds_fully(x_temp, y_temp) {
                continue;
            }

            let g = *st.cave.grid(x_temp, y_temp);
            let feat = u32::from(g.feat);

            // Cannot occupy another monster's grid
            if g.kill != 0 {
                continue;
            }

            // Cannot occupy a closed door
            if feat == FEAT_CLOSED {
                continue;
            }

            // Cannot occupy a perma-wall
            if feat == FEAT_PERM {
                continue;
            }

            // Cannot occupy a wall/seam grid (unless pass_wall or kill_wall)
            if wall_like(feat) {
                // legally on a wall of some sort, check for closeness
                if rf(kill, RF_PASS_WALL) && borg_distance(y_temp, x_temp, y, x) == 1 {
                    b_v1 = v1;
                }
                if rf(kill, RF_KILL_WALL) && borg_distance(y_temp, x_temp, y, x) == 1 {
                    b_v1 = v1;
                }
            }

            // Is this grid being considered adjacent to the grid for which
            // the borg_danger() was called?
            if borg_distance(y_temp, x_temp, y, x) > 1 {
                continue;
            }

            // A legal floor grid
            if borg_cave_floor_bold(st, y_temp, x_temp) {
                // Really fast monster can hit me more than once after it's
                // move
                b_v1 = v1 * (q / (d * 10));
            }
        }

        // Monster is not able to move and threaten me in the same round
        v1 = b_v1;
    }

    // Consider a monster that is fast and can strike more than once per round
    if q > 10 && d == 1 {
        v1 = v1 * q / 10;
    }

    // Need to be close if you are normal speed
    if q == 10 && d > 1 {
        v1 = 0;
    }

    // Ranged Attacks
    v2 = borg_danger_spell(st, kill, flags, y, x, d, average);

    // Never cast spells
    if r_ptr.freq_innate == 0 && r_ptr.freq_spell == 0 {
        v2 = 0;
    }

    // Verify distance
    if borg_distance(y9, x9, y, x) > i32::from(engine::z_info().max_range) {
        v2 = 0;
    }

    // Verify line of sight (both ways) for monsters who can only move 1
    // grid.
    if q <= 10 && !borg_projectable(st, y9, x9, y, x) && !borg_projectable(st, y, x, y9, x9) {
        v2 = 0;
    }

    // Verify line of sight (both ways) for monsters who can only move >
    // 1 grid. Some fast monsters can take a move action and range attack in
    // the same round. Basically, we see how many grids the monster can move
    // and check LOS from each of those grids to determine the relative
    // danger.  We need to make sure that the monster is not passing through
    // walls unless he has that ability. Consider a fast monster who can move
    // and cast a spell in the same round. This is important for a borg
    // looking for a safe grid from a ranged attacker. If the attacker is
    // fast then he might be able to move to a grid which does have LOS to
    // the grid the borg is considering.
    //
    // ##############
    // #.....#.#.1#D#   Grids marked 2 are threatened by the D currently.
    // #####.#..##@##   Grids marked 1 are safe currently, but the fast D
    // #####.#..1221#   will be able to move to the borg's grid after he
    // ##############   moves and the D will be able to use a ranged attack
    //                  to grids 1, all in the same round. The borg should
    //                  not consider grid 1 as safe.
    if q >= 20 {
        let b_q = q;
        let mut b_v2 = 0;

        // Maximal speed check
        if q > 20 {
            q = 20;
        }

        // Check for a single grid movement, simulating the monster's move
        // action.
        for ii in 0..8 {
            // Obtain a grid to which the monster might move
            let y_temp = y9 + i32::from(engine::ddy_ddd(ii));
            let x_temp = x9 + i32::from(engine::ddx_ddd(ii));

            // Check for legality
            if !engine::square_in_bounds_fully(x_temp, y_temp) {
                continue;
            }

            let g = *st.cave.grid(x_temp, y_temp);
            let feat = u32::from(g.feat);

            // Cannot occupy another monster's grid
            if g.kill != 0 {
                continue;
            }

            // Cannot occupy a closed door
            if feat == FEAT_CLOSED {
                continue;
            }

            // Cannot occupy a perma-wall
            if feat == FEAT_PERM {
                continue;
            }

            // Cannot occupy a wall/seam grid (unless pass_wall or kill_wall)
            if wall_like(feat) {
                // legally on a wall of some sort, check for LOS
                if rf(kill, RF_PASS_WALL) && borg_projectable(st, y_temp, x_temp, y, x) {
                    b_v2 = v2 * b_q / 10;
                }
                if rf(kill, RF_KILL_WALL) && borg_projectable(st, y_temp, x_temp, y, x) {
                    b_v2 = v2 * b_q / 10;
                }
            }
            // Monster on a legal floor grid.  Can he see me?
            else if borg_projectable(st, y_temp, x_temp, y, x) {
                b_v2 = v2 * b_q / 10;
            }
        }

        // Monster is not able to move and threaten me in the same round
        v2 = b_v2;
    }

    // If the Borg has been bouncing for a long time, or if the time this
    // level is very high, reduce the danger value. This helps prevent the
    // borg from getting stuck due to overestimating danger and refusing to
    // move in both this area of the dungeon and over the course of the game.
    if st.borg.antibounce_count > 1200 || st.borg.time.now - st.borg.time.level > 25000 {
        v2 /= 5;
    }

    // multipliers yield some trouble when I am weak
    if rf(kill, RF_MULTIPLY) && st.borg.tr(BI_CLEVEL) < 20 {
        v2 += v2 * 12 / 10;
    }

    // Friends yield some trouble when I am weak
    if has_friends && st.borg.tr(BI_CLEVEL) < 20 {
        v2 += v2 * 12 / 10;
    }

    // Reduce danger from sleeping monsters
    if !kill.awake {
        let inc = r_ptr.sleep + 5;
        // weaklings and should still fear
        if st.borg.tr(BI_CLEVEL) >= 25 {
            v2 /= 2;
        }

        // Tweak danger based on the "alertness" of the monster
        // increase the danger for light sleepers
        v2 += v2 * inc / 100;
    }

    // Reduce danger from sleeping monsters with the sleep 2 spell
    if flags.sleep_spell_ii {
        let clevel = st.borg.tr(BI_CLEVEL);
        let max_level = if clevel < 15 { clevel } else { ((clevel - 10) / 4) * 3 + 10 };
        if d == 1
            && kill.awake
            && !rf(kill, RF_NO_SLEEP)
            && !rf(kill, RF_UNIQUE)
            && i32::from(kill.level) <= max_level
        {
            v2 /= 3;
        }
    }

    // Clear the ranged danger when Crush would kill the monster.
    if flags.crush_spell && crushable(st, kill) {
        v2 = 0;
    }

    // Reduce danger from sleeping monsters with the sleep 1,3 spell
    if flags.sleep_spell {
        v2 /= d + 2;
    }
    // Reduce danger from confused monsters
    if kill.confused {
        v2 /= 2;
    }
    // Reduce danger from stunned monsters
    if kill.stunned {
        v2 = v2 * 10 / 13;
    }
    if flags.confuse_spell {
        v2 /= 6;
    }

    // (C: `#if 0` -- scared monsters still cast spells, they are still
    // dangerous)
    if !full_damage {
        // reduce for frequency.
        let chance = (r_ptr.freq_innate + r_ptr.freq_spell) / 2;
        if chance < 11 {
            v2 = (v2 * 4) / 10;
        } else if chance < 26 {
            v2 = (v2 * 6) / 10;
        } else if chance < 51 {
            v2 = (v2 * 8) / 10;
        }
    }

    // Danger
    if v2 != 0 {
        // Full power
        let r = q;

        // Total danger
        v2 = v2 * r / 10;
    }

    // Maximal danger
    let p = v1.max(v2);
    if p > 2000 {
        2000
    } else {
        p
    }
}

/// The crush spell condition: the monster dies (less than 4x the borg's
/// level in hit points) and is in view on a floor grid.
fn crushable(st: &BorgState, kill: &BorgKill) -> bool {
    // Either it dies or it doesn't.  For it to die it must have less
    // than 4x the borgs leve in hp and be in view.
    // So power(total hp) times (100-injury(percent injured)) divided by 100
    if (i32::from(kill.power) * (100 - i32::from(kill.injury))) / 100 < st.borg.tr(BI_CLEVEL) * 4 {
        let ag = st.cave.grid(kill.pos.x, kill.pos.y);
        if ag.info & BORG_VIEW != 0 && borg_cave_floor_grid(ag) {
            return true;
        }
    }
    false
}

/// Calculate the "danger" of the given grid (`borg_danger`).
///
/// Currently based on the physical power of nearby monsters, as well
/// as the spell power of monsters which can target the given grid.
///
/// This function is extremely expensive, mostly due to the number of
/// times it is called, and also to the fact that it calls its helper
/// functions about thirty times each per call.
///
/// We need to do more intelligent processing with the "c" parameter,
/// since currently the Borg does not realize that backing into a
/// hallway is a good idea, since as far as he can tell, many of
/// the nearby monsters can "squeeze" into a single grid.
///
/// Note that we also take account of the danger of the "region" in
/// which the grid is located, which allows us to apply some "fear"
/// of invisible monsters and things of that nature.
///
/// Generally bool Average is true.
pub fn borg_danger(
    st: &mut BorgState,
    y: i32,
    x: i32,
    c: i32,
    average: bool,
    full_damage: bool,
) -> i32 {
    let mut p: i32 = 0;

    if !engine::square_in_bounds(x, y) {
        return 2000;
    }

    // Base danger (from regional fear) but not within a vault.  Cheating the
    // floor grid
    if !engine::square_isvault(x, y) && st.borg.tr(BI_CDEPTH) <= 80 {
        p += i32::from(st.danger.fear_region[(y / 11) as usize][(x / 11) as usize]) * c;
    }

    // Reduce regional fear on Depth 100
    if st.borg.tr(BI_CDEPTH) == 100 && p >= 300 {
        p = 300;
    }

    // Added danger (from a lot of monsters).
    // But do not add it if we have been bouncing for too long, or monster's
    // in a vault.  The fear_monsters[][] can induce some bouncy behavior.
    if st.borg.antibounce_count <= 200 && !engine::square_isvault(x, y) {
        p += i32::from(st.danger.fear_monsters[y as usize][x as usize]) * c;
    }

    // C overrides the argument
    let _ = full_damage;
    let full_damage = true;

    // Examine all the monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Collect danger from monster
        p += danger_one_kill(st, &kill, y, x, c, i, average, full_damage);
    }

    // Return the danger
    if p > 2000 {
        2000
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fear_map_sizes_match_c() {
        // C: [(AUTO_MAX_Y / 11) + 1][(AUTO_MAX_X / 11) + 1] and
        // [AUTO_MAX_Y + 1][AUTO_MAX_X + 1] with a 198x66 dungeon
        assert_eq!((FEAR_REGION_HGT, FEAR_REGION_WID), (7, 19));
        assert_eq!((FEAR_MONSTERS_HGT, FEAR_MONSTERS_WID), (67, 199));
    }

    #[test]
    fn wall_tests_match_v11() {
        assert!(wall_like(FEAT_GRANITE));
        assert!(wall_like(FEAT_RUBBLE));
        assert!(wall_like(FEAT_MAGMA_K));
        assert!(wall_like(FEAT_QUARTZ_K));
        // lava (23), permanent wall and closed doors are not wall/seam grids
        assert!(!wall_like(23));
        assert!(!wall_like(FEAT_CLOSED));
    }
}

//! Full danger assessment over the typed native-memory view.
//! Adapted from borg-rs M13 1443c2d (borg_danger.rs), with Track P named
//! parameters. Provenance in PROVENANCE.md.
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

// borg-reborn-rs (#220): named danger/escape policy values; defaults preserve arithmetic.
use crate::View;
use borg_rs::memory::{BorgGrid, BorgKill};
use borg_rs::threat_facts::{self as facts, param, Facts, Param};
use facts::constants::*;
const CLASS_MAGE: i32 = 1;
struct Assessment<'a> {
    view: &'a View<'a>,
    facts: Facts<'a>,
}
impl Assessment<'_> {
    fn grid(&self, x: i32, y: i32) -> &BorgGrid {
        self.view
            .cave
            .grid(x, y)
            .unwrap_or_else(|| crate::shadow::invalid_danger_grid(self.view, x, y))
    }
}
fn rf(kill: &BorgKill, flag: u32) -> bool {
    facts::race_flag_has(usize::from(kill.r_idx), flag)
}
type SpellFlags = crate::Simulation;
/// Does protection from evil work against this monster?
fn pfe_applies(st: &Assessment<'_>, kill: &BorgKill) -> bool {
    let r_ptr = facts::race(usize::from(kill.r_idx));
    st.view.threat.temporary.prot_from_evil
        && rf(kill, RF_EVIL)
        && st.view.character.clevel >= r_ptr.level
}

/// Calculate base danger from a monster's physical attacks
/// (`physical_estimate`).
///
/// We attempt to take account of various resistances, both in
/// terms of actual damage, and special effects, as appropriate.
///
/// We reduce the danger from distant "sleeping" monsters.
/// PFE reduces my fear of an area.
fn physical_estimate(
    st: &Assessment<'_>,
    kill: &BorgKill,
    flags: SpellFlags,
    full_damage: bool,
) -> i32 {
    let mut n: i32 = 0;
    let mut pfe = false;

    let mut ac: i16 = st.view.combat.armor as i16;

    // shields gives +50 to ac and deflects some missiles and balls
    if st.view.threat.temporary.shield {
        ac = (i32::from(ac) + 50) as i16;
    }

    // PFE gives a protection.
    // Apply PROTECTION_FROM_EVIL
    if pfe_applies(st, kill) {
        pfe = true;
    }

    // Mega-Hack -- unknown monsters (or "player ghosts"
    if kill.r_idx == 0 {
        return param(Param::danger_unknown_physical);
    }
    if i32::from(kill.r_idx) >= i32::from(facts::z_info().r_max) - 1 {
        return param(Param::danger_unknown_physical);
    }

    let r_idx = usize::from(kill.r_idx);
    let level = facts::race(r_idx).level;
    let half = !flags.attacking && pfe;

    // Analyze each physical attack
    for k in 0..usize::from(facts::z_info().mon_blows_max) {
        let mut z: i32 = 0;

        // Extract the attack infomation
        let Some(blow) = facts::blow(r_idx, k) else {
            // Done
            break;
        };
        let d_dice = blow.dice;
        let d_side = blow.sides;

        let mut power: i32 = param(Param::danger_default_hit_power);

        // Analyze the attack
        let effect = blow.effect;
        match effect {
            MONBLOW_HURT => {
                z = d_dice * d_side;
                // fudge- some baddies kick and they tend to KO.  Avoid close
                // combat like the plague.  10d2 is common but take any very
                // low sides and high dice count as dangerous
                if d_side < param(Param::danger_knockout_max_sides)
                    && d_dice > param(Param::danger_knockout_min_dice)
                {
                    n += param(Param::danger_monblow_hurt_fear);
                }
                power = param(Param::danger_monblow_hurt_hit_power);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_POISON => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_poison_hit_power);
                if st.view.resistances.rpois != 0 {
                    break 'b;
                }
                if st.view.threat.temporary.res_pois {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_poison_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_DISENCHANT => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_disenchant_hit_power);
                if st.view.resistances.rdis != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_disenchant_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_DRAIN_CHARGES => {
                z = d_dice * d_side;
                // Add fear for the effect
                z += param(Param::danger_monblow_drain_charges_fear);
                power = param(Param::danger_monblow_drain_charges_hit_power);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_EAT_GOLD => 'b: {
                z = d_dice * d_side;
                // if in town and low level avoid them stupid urchins
                if st.view.character.clevel < param(Param::danger_gold_theft_low_level) {
                    z += param(Param::danger_monblow_eat_gold_fear);
                }
                power = param(Param::danger_monblow_eat_gold_hit_power);
                if 100
                    <= facts::adj_dex_safe(st.view.attributes.dex_index as usize)
                        + st.view.character.clevel
                {
                    break 'b;
                }
                if st.view.abilities.gold < param(Param::danger_gold_theft_min_gold) {
                    break 'b;
                }
                if st.view.abilities.gold > param(Param::danger_gold_theft_max_gold) {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_eat_gold_fear_2);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_EAT_ITEM => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_eat_item_hit_power);
                if 100
                    <= facts::adj_dex_safe(st.view.attributes.dex_index as usize)
                        + st.view.character.clevel
                {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_eat_item_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_EAT_FOOD => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_eat_food_hit_power);
                if st.view.food.food > param(Param::danger_food_theft_reserve) {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_eat_food_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_EAT_LIGHT => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_eat_light_hit_power);
                if st.facts.light_needs_no_fuel() {
                    break 'b;
                }
                if st.view.supplies.afuel > param(Param::danger_fuel_theft_reserve) {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_eat_light_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_ACID => 'b: {
                if st.view.resistances.iacid != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                if st.view.resistances.racid != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_acid && st.view.resistances.vacid == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vacid != 0 && !st.view.threat.temporary.res_acid {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_acid_fear); // We don't want our armour corroded.
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_ELEC => 'b: {
                if st.view.resistances.ielec != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = param(Param::danger_monblow_elec_hit_power);
                if st.view.resistances.relec != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_elec && st.view.resistances.velec == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.velec != 0 && !st.view.threat.temporary.res_elec {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                z *= param(Param::danger_monblow_elec_damage_multiplier);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_FIRE => 'b: {
                if st.view.resistances.ifire != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = param(Param::danger_monblow_fire_hit_power);
                if st.view.resistances.rfire != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_fire && st.view.resistances.vfire == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vfire != 0 && !st.view.threat.temporary.res_fire {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                z *= param(Param::danger_monblow_fire_damage_multiplier);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_COLD => 'b: {
                if st.view.resistances.icold != 0 {
                    break 'b;
                }
                z = d_dice * d_side;
                power = param(Param::danger_monblow_cold_hit_power);
                if st.view.resistances.rcold != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_cold {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vcold != 0 && !st.view.threat.temporary.res_cold {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                z *= param(Param::danger_monblow_cold_damage_multiplier);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_BLIND => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_blind_hit_power);
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_blind_fear);
                if st.view.character.class == CLASS_MAGE {
                    z += param(Param::danger_monblow_blind_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_CONFUSE => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_confuse_hit_power);
                if st.view.resistances.rconf != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_confuse_fear);
                if st.view.character.class == CLASS_MAGE {
                    z += param(Param::danger_monblow_confuse_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_TERRIFY => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_terrify_hit_power);
                if st.view.resistances.rfear != 0 {
                    break 'b;
                }
                // Add fear for the effect
                z *= param(Param::danger_monblow_terrify_damage_multiplier);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_PARALYZE => 'b: {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_paralyze_hit_power);
                if st.view.resistances.fract != 0 {
                    break 'b;
                }
                z += param(Param::danger_monblow_paralyze_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_STR => 'b: {
                z = d_dice * d_side;
                if st.view.attributes.sstr != 0 {
                    break 'b;
                }
                if st.view.attributes.cstr <= 3 {
                    break 'b;
                }
                if st.facts.spell_legal(RESTORATION) {
                    break 'b;
                }
                if st.facts.spell_legal(REVITALIZE) {
                    break 'b;
                }
                if st.facts.spell_legal(UNHOLY_REPRIEVE) {
                    break 'b;
                }
                z += param(Param::danger_monblow_lose_str_fear);
                // extra scary to have str drain below 10
                if st.view.attributes.cstr < param(Param::danger_str_drain_low_stat) {
                    z += param(Param::danger_monblow_lose_str_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_DEX => 'b: {
                z = d_dice * d_side;
                if st.view.attributes.sdex != 0 {
                    break 'b;
                }
                if st.view.attributes.cdex <= 3 {
                    break 'b;
                }
                if st.facts.spell_legal(RESTORATION) {
                    break 'b;
                }
                if st.facts.spell_legal(REVITALIZE) {
                    break 'b;
                }
                z += param(Param::danger_monblow_lose_dex_fear);
                // extra scary to have drain below 10
                if st.view.attributes.cdex < param(Param::danger_dex_drain_low_stat) {
                    z += param(Param::danger_monblow_lose_dex_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_CON => 'b: {
                z = d_dice * d_side;
                if st.view.attributes.scon != 0 {
                    break 'b;
                }
                if st.view.attributes.ccon <= 3 {
                    break 'b;
                }
                if st.facts.spell_legal(RESTORATION) {
                    break 'b;
                }
                if st.facts.spell_legal(REVITALIZE) {
                    break 'b;
                }
                if st.facts.spell_legal(UNHOLY_REPRIEVE) {
                    break 'b;
                }
                // Add fear for the effect
                z += param(Param::danger_monblow_lose_con_fear);
                // extra scary to have con drain below 8
                if st.view.attributes.ccon < param(Param::danger_con_drain_low_stat) {
                    z += param(Param::danger_monblow_lose_con_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_INT => 'b: {
                z = d_dice * d_side;
                if st.view.attributes.sint != 0 {
                    break 'b;
                }
                if st.view.attributes.cint <= 3 {
                    break 'b;
                }
                if st.facts.spell_legal(RESTORATION) {
                    break 'b;
                }
                if st.facts.spell_legal(REVITALIZE) {
                    break 'b;
                }
                if st.facts.spell_legal(UNHOLY_REPRIEVE) {
                    break 'b;
                }
                z += param(Param::danger_monblow_lose_int_fear);
                // extra scary for spell caster
                if facts::spell_stat() == STAT_INT as i32 {
                    z += param(Param::danger_monblow_lose_int_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_WIS => 'b: {
                z = d_dice * d_side;
                if st.view.attributes.swis != 0 {
                    break 'b;
                }
                if st.view.attributes.cwis <= 3 {
                    break 'b;
                }
                if st.facts.spell_legal(RESTORATION) {
                    break 'b;
                }
                if st.facts.spell_legal(REVITALIZE) {
                    break 'b;
                }
                z += param(Param::danger_monblow_lose_wis_fear);
                // extra scary for pray'er
                if facts::spell_stat() == STAT_WIS as i32 {
                    z += param(Param::danger_monblow_lose_wis_fear_2);
                }
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_LOSE_ALL => {
                z = d_dice * d_side;
                power = param(Param::danger_monblow_lose_all_hit_power);
                // only morgoth to make it easier to fight him
            }

            MONBLOW_SHATTER => {
                z = d_dice * d_side;
                z -=
                    z * (if i32::from(ac) < param(Param::danger_shatter_ac_cap) {
                        i32::from(ac)
                    } else {
                        param(Param::danger_shatter_ac_cap)
                    }) / param(Param::danger_shatter_ac_divisor);
                power = param(Param::danger_monblow_shatter_hit_power);
                // Add fear for the effect
                z += param(Param::danger_monblow_shatter_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_EXP_10 | MONBLOW_EXP_20 | MONBLOW_EXP_40 | MONBLOW_EXP_80 => 'b: {
                let (fear, at_50) = match effect {
                    // do not worry about drain exp after level 50 (EXP_10
                    // tests `== 50`, the others `>= 50`)
                    MONBLOW_EXP_10 => (
                        param(Param::danger_exp10_fear),
                        st.view.character.clevel == param(Param::danger_exp_ignore_level),
                    ),
                    MONBLOW_EXP_20 => (
                        param(Param::danger_exp20_fear),
                        st.view.character.clevel >= param(Param::danger_exp_ignore_level),
                    ),
                    MONBLOW_EXP_40 => (
                        param(Param::danger_exp40_fear),
                        st.view.character.clevel >= param(Param::danger_exp_ignore_level),
                    ),
                    _ => (
                        param(Param::danger_exp80_fear),
                        st.view.character.clevel >= param(Param::danger_exp_ignore_level),
                    ),
                };
                z = d_dice * d_side;
                if st.view.resistances.hlife != 0 {
                    break 'b;
                }
                if at_50 {
                    break 'b;
                }
                if st.facts.spell_legal(REMEMBRANCE)
                    || st.facts.spell_legal(UNHOLY_REPRIEVE)
                    || st.facts.spell_legal(REVITALIZE)
                {
                    break 'b;
                }
                // Add fear for the effect
                z += fear;
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            MONBLOW_HALLU => {
                z = d_dice * d_side;
                // Add fear for the effect
                z += param(Param::danger_monblow_hallu_fear);
                if half {
                    z /= param(Param::danger_pfe_melee_divisor);
                }
            }

            _ => {}
        }

        // reduce by damage reduction
        z -= st.view.abilities.dam_red;
        if z < 0 {
            z = 0;
        }

        // if we are doing partial damage reduce for % chance that it will
        // hit you.
        if !full_damage {
            // figure out chance that monster will hit you.
            // add a 50% bonus in to account for bad luck.
            let chance = if (level + power) > 0 {
                param(Param::danger_melee_hit_luck_percent)
                    - (((i32::from(ac) * param(Param::danger_melee_hit_ac_scale))
                        / param(Param::danger_melee_hit_ac_divisor))
                        / ((level + power) * param(Param::danger_melee_hit_power_scale)))
            } else {
                -1
            };

            // always have a 5% chance of hitting.
            let chance = if chance < param(Param::danger_melee_hit_floor_percent) {
                param(Param::danger_melee_hit_floor_percent)
            } else {
                chance
            };
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
fn count_safe_spots(st: &Assessment<'_>, kill: &BorgKill, spot_safe: &mut i32) {
    for spot_x in -1..=1 {
        for spot_y in -1..=1 {
            // Acquire location
            let x = spot_x + kill.pos.x;
            let y = spot_y + kill.pos.y;

            let ag: &BorgGrid = st.grid(x, y);

            // skip our own spot
            if x == kill.pos.x && y == kill.pos.y {
                continue;
            }

            // track spaces already protected
            if facts::protected(ag) {
                // track the safe areas for calculating danger
                *spot_safe += 1;

                // Just in case
                if *spot_safe == 0 {
                    *spot_safe = 1;
                }
                if *spot_safe == 8 {
                    *spot_safe = param(Param::danger_summon_protected_divisor);
                }
                if st.view.threat.morgoth_position || st.view.core_status.anti_summon {
                    *spot_safe = param(Param::danger_summon_corridor_divisor);
                }
            }
        }
    }
}

/// Calculate base danger from a monster's spell attacks
/// (`spell_estimate`).
///
/// We attempt to take account of various resistances, both in
/// terms of actual damage, and special effects, as appropriate.
///
/// We reduce the danger from distant "sleeping" monsters.
///
/// We reduce the danger if the monster is immobile or not LOS
fn spell_estimate(
    st: &Assessment<'_>,
    kill: &BorgKill,
    flags: SpellFlags,
    y: i32,
    x: i32,
    d: i32,
    average: bool,
) -> i32 {
    let mut n: i32 = 0;
    let mut spot_safe: i32 = 1;
    let mut total_dam: i32 = 0;

    // PFE gives a protection.
    // Apply PROTECTION_FROM_EVIL
    let pfe = pfe_applies(st, kill);

    // glyph of warding rune of protection provides some small
    // protection with some ranged atacks; mainly summon attacks.
    // We should reduce the danger commensurate to the probability of the
    // monster breaking the glyph as defined by melee2.c
    let glyph = flags.on_glyph || st.view.threat.glyphs().any(|(gy, gx)| gy == y && gx == x);

    // Mega-Hack -- unknown monsters (or "player ghosts")
    if kill.r_idx == 0 {
        return param(Param::danger_unknown_spell);
    }
    if i32::from(kill.r_idx) >= i32::from(facts::z_info().r_max) - 1 {
        return param(Param::danger_unknown_spell);
    }

    // Paranoia -- Nothing to cast
    if kill.ranged_attack == 0 {
        return 0;
    }

    let spell_power = facts::race(usize::from(kill.r_idx)).spell_power;
    let unique = rf(kill, RF_UNIQUE);
    let near_unique = st.view.threat.nearby_unique != 0;
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
                p += param(Param::danger_rsf_shriek_fear);
            }

            RSF_WHIP => {
                if d < 3 {
                    z = param(Param::danger_rsf_lash_damage);
                }
            }

            RSF_SPIT => {
                if d < 4 {
                    z = param(Param::danger_rsf_spit_damage);
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
                if st.view.resistances.iacid != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.view.resistances.racid != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_acid && st.view.resistances.vacid == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vacid != 0 && !st.view.threat.temporary.res_acid {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_acid_fear);
            }

            RSF_BR_ELEC => 'b: {
                if st.view.resistances.ielec != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.view.resistances.relec != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_elec && st.view.resistances.velec == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.velec != 0 && !st.view.threat.temporary.res_elec {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_elec_fear);
            }

            RSF_BR_FIRE => 'b: {
                if st.view.resistances.ifire != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.view.resistances.rfire != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_fire && st.view.resistances.vfire == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vfire != 0 && !st.view.threat.temporary.res_fire {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_fire_fear);
            }

            RSF_BR_COLD => 'b: {
                if st.view.resistances.icold != 0 {
                    break 'b;
                }
                z = hp / 3;
                // max damage
                if z > 1600 {
                    z = 1600;
                }
                if st.view.resistances.rcold != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_cold && st.view.resistances.vcold == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vcold != 0 && !st.view.threat.temporary.res_cold {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_cold_fear);
            }

            RSF_BR_POIS => 'b: {
                z = hp / 3;
                // max damage
                if z > 800 {
                    z = 800;
                }
                if st.view.resistances.rpois != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_pois {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_pois {
                    break 'b;
                }
                if st.view.resistances.rpois != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_pois_fear);
            }

            RSF_BR_NETH => 'b: {
                z = hp / 6;
                // max damage
                if z > 600 {
                    z = 600;
                }
                if st.view.resistances.rnthr != 0 {
                    z = (z * 6) / 9;
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_neth_fear);
            }

            RSF_BR_LIGHT => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.view.resistances.rlite != 0 {
                    z = (z * 2) / 3;
                    break 'b;
                }
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                p += param(Param::danger_rsf_br_light_fear);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_br_light_fear_2);
                }
            }

            RSF_BR_DARK => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.view.resistances.rdark != 0 {
                    z = (z * 2) / 3;
                }
                if st.view.resistances.rdark != 0 {
                    break 'b;
                }
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                p += param(Param::danger_rsf_br_dark_fear);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_br_dark_fear_2);
                }
            }

            RSF_BR_SOUN => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.view.resistances.rsnd != 0 {
                    z = (z * 5) / 9;
                }
                if st.view.resistances.rsnd != 0 || st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_br_soun_fear);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_br_soun_fear_2);
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_soun_fear_3);
            }

            RSF_BR_CHAO => 'b: {
                z = hp / 6;
                // max damage
                if z > 600 {
                    z = 600;
                }
                if st.view.resistances.rkaos != 0 {
                    z = (z * 6) / 9;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_chao_fear);
                if st.view.resistances.rkaos != 0 {
                    break 'b;
                }
                p += param(Param::danger_rsf_br_chao_fear_2);
            }

            RSF_BR_DISE => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.view.resistances.rdis != 0 {
                    z = (z * 6) / 10;
                }
                if st.view.resistances.rdis != 0 {
                    break 'b;
                }
                p += param(Param::danger_rsf_br_dise_fear);
            }

            RSF_BR_NEXU => 'b: {
                z = hp / 6;
                // max damage
                if z > 400 {
                    z = 400;
                }
                if st.view.resistances.rnxus != 0 {
                    z = (z * 6) / 10;
                }
                if st.view.resistances.rnxus != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_nexu_fear);
            }

            RSF_BR_TIME => {
                z = hp / 3;
                // max damage
                if z > 150 {
                    z = 150;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_time_fear);
            }

            RSF_BR_INER => {
                z = hp / 6;
                // max damage
                if z > 200 {
                    z = 200;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_iner_fear);
            }

            RSF_BR_GRAV => 'b: {
                z = hp / 3;
                // max damage
                if z > 200 {
                    z = 200;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_grav_fear);
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_br_grav_fear_2);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_br_grav_fear_3);
                }
            }

            RSF_BR_SHAR => 'b: {
                z = hp / 6;
                // max damage
                if z > 500 {
                    z = 500;
                }
                if st.view.resistances.rshrd != 0 {
                    z = (z * 6) / 9;
                }
                if st.view.resistances.rshrd != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_shar_fear);
            }

            RSF_BR_PLAS => 'b: {
                z = hp / 6;
                // max damage
                if z > 150 {
                    z = 150;
                }
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // Pump this up if you have goi so that the borg is sure
                // to be made nervous
                p += param(Param::danger_rsf_br_plas_fear);
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_br_plas_fear_2);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_br_plas_fear_3);
                }
            }

            RSF_BR_WALL => 'b: {
                z = hp / 6;
                // max damage
                if z > 200 {
                    z = 200;
                }
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_br_wall_fear);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_br_wall_fear_2);
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_br_wall_fear_3);
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
                if st.view.resistances.iacid != 0 {
                    break 'b;
                }
                z = (spell_power * 3) + 15;
                if st.view.resistances.racid != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_acid && st.view.resistances.vacid == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vacid != 0 && !st.view.threat.temporary.res_acid {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_acid_fear);
            }

            RSF_BA_ELEC => 'b: {
                if st.view.resistances.ielec != 0 {
                    break 'b;
                }
                z = (spell_power * 3) / 2 + 8;
                if st.view.resistances.relec != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_elec && st.view.resistances.velec == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.velec != 0 && !st.view.threat.temporary.res_elec {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_elec_fear);
            }

            RSF_BA_FIRE => 'b: {
                if st.view.resistances.ifire != 0 {
                    break 'b;
                }
                z = (spell_power * 7) / 2 + 10;
                if st.view.resistances.rfire != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_fire && st.view.resistances.vfire == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vfire != 0 && !st.view.threat.temporary.res_fire {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_fire_fear);
            }

            RSF_BA_COLD => 'b: {
                if st.view.resistances.icold != 0 {
                    break 'b;
                }
                z = (spell_power * 3 / 2) + 10;
                if st.view.resistances.rcold != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_cold && st.view.resistances.vcold == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vcold != 0 && !st.view.threat.temporary.res_cold {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_cold_fear);
            }

            RSF_BA_POIS => 'b: {
                z = (spell_power / 2 + 3) * 4;
                if st.view.resistances.rpois != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_pois {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_pois {
                    break 'b;
                }
                if st.view.resistances.rpois != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_pois_fear);
            }

            RSF_BA_SHAR => 'b: {
                z = ((spell_power * 3) / 2) + 10;
                if st.view.resistances.rshrd != 0 {
                    z = (z * 6) / 9;
                }
                if st.view.resistances.rshrd != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_shar_fear);
            }

            RSF_BA_NETH => 'b: {
                z = (spell_power * 4) + (10 * 10);
                if st.view.resistances.rnthr != 0 {
                    z = (z * 6) / 8;
                }
                if st.view.resistances.rnthr != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_neth_fear);
            }

            RSF_BA_WATE => 'b: {
                z = (spell_power * 5) / 2 + 50;
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    p += param(Param::danger_rsf_ba_wate_fear);
                }
                if st.view.status.isheavystun != 0 {
                    p += param(Param::danger_rsf_ba_wate_fear_2);
                }
                if st.view.resistances.rconf != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_wate_fear_3);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_ba_wate_fear_4);
                }
            }

            RSF_BA_MANA => {
                z = (spell_power * 5) + (10 * 10);
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_mana_fear);
            }

            RSF_BA_HOLY => {
                z = 10 + ((spell_power * 3) / 2 + 1) / 2;
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_holy_fear);
            }

            RSF_BA_DARK => 'b: {
                z = (spell_power * 4) + (10 * 10);
                if st.view.resistances.rdark != 0 {
                    z = (z * 6) / 9;
                }
                if st.view.resistances.rdark != 0 {
                    break 'b;
                }
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_dark_fear);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_ba_dark_fear_2);
                }
            }

            RSF_BA_LIGHT => 'b: {
                z = 10 + (spell_power * 3 / 2);
                if st.view.resistances.rlite != 0 {
                    z = (z * 6) / 9;
                }
                if st.view.resistances.rlite != 0 {
                    break 'b;
                }
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_ba_light_fear);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_ba_light_fear_2);
                }
            }

            RSF_STORM => 'b: {
                z = 70 + (spell_power * 5);
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    p += param(Param::danger_rsf_storm_fear);
                }
                if st.view.status.isheavystun != 0 {
                    p += param(Param::danger_rsf_storm_fear_2);
                }
                // (C: `if (borg.trait[BI_RCONF]) break;` here, the last
                // statement of the case)
            }

            RSF_DRAIN_MANA => {
                if st.view.character.maxsp != 0 {
                    p += param(Param::danger_rsf_drain_mana_fear);
                }
            }

            RSF_MIND_BLAST => {
                if st.view.abilities.sav < 100 {
                    z = spell_power / 2 + 1;
                }
            }

            RSF_BRAIN_SMASH => {
                z = (12 * (15 + 1)) / 2;
                p += param(Param::danger_rsf_brain_smash_fear)
                    - param(Param::danger_brain_smash_save_weight) * st.view.abilities.sav;
                if p < 0 {
                    p = 0;
                }
            }

            RSF_WOUND => 'b: {
                if st.view.abilities.sav >= 100 {
                    break 'b;
                }
                z = (spell_power / 3 * 2) * 5;
                // reduce by % chance of save  (add 20% for fudge)
                z = z * (param(Param::danger_cause4_save_bias) - st.view.abilities.sav) / 100;
            }

            RSF_BO_ACID => 'b: {
                bolt = true;
                if st.view.resistances.iacid != 0 {
                    break 'b;
                }
                z = (7 * 8) + (spell_power / 3);
                if st.view.resistances.racid != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_acid && st.view.resistances.vacid == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vacid != 0 && !st.view.threat.temporary.res_acid {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_acid_fear);
            }

            RSF_BO_ELEC => 'b: {
                if st.view.resistances.ielec != 0 {
                    break 'b;
                }
                bolt = true;
                z = (4 * 8) + (spell_power / 3);
                if st.view.resistances.relec != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_elec && st.view.resistances.velec == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.velec != 0 && !st.view.threat.temporary.res_elec {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_elec_fear);
            }

            RSF_BO_FIRE => 'b: {
                if st.view.resistances.ifire != 0 {
                    break 'b;
                }
                bolt = true;
                z = (9 * 8) + (spell_power / 3);
                if st.view.resistances.rfire != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_fire && st.view.resistances.vfire == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vfire != 0 && !st.view.threat.temporary.res_fire {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_fire_fear);
            }

            RSF_BO_COLD => 'b: {
                if st.view.resistances.icold != 0 {
                    break 'b;
                }
                bolt = true;
                z = (6 * 8) + (spell_power / 3);
                if st.view.resistances.rcold != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_cold && st.view.resistances.vcold == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.vcold != 0 && !st.view.threat.temporary.res_cold {
                    z = z * 4 / 3;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_cold_fear);
            }

            RSF_BO_POIS => 'b: {
                if st.view.resistances.ipois != 0 {
                    break 'b;
                }
                z = (9 * 8) + (spell_power / 3);
                if st.view.resistances.rpois != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_pois {
                    z = (z + 2) / 3;
                }
                bolt = true;
            }

            RSF_BO_NETH => 'b: {
                bolt = true;
                z = (5 * 5) + (spell_power * 3 / 2) + 50;
                if st.view.resistances.rnthr != 0 {
                    z = (z * 6) / 8;
                }
                if st.view.resistances.rnthr != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_neth_fear);
            }

            RSF_BO_WATE => 'b: {
                z = (10 * 10) + spell_power;
                bolt = true;
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    p += param(Param::danger_rsf_bo_wate_fear);
                }
                if st.view.status.isheavystun != 0 {
                    p += param(Param::danger_rsf_bo_wate_fear_2);
                }
                if st.view.resistances.rconf != 0 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_wate_fear_3);
                if st.view.character.class == CLASS_MAGE {
                    p += param(Param::danger_rsf_bo_wate_fear_4);
                }
            }

            RSF_BO_MANA => {
                z = (spell_power * 5) / 2 + 50;
                bolt = true;
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_mana_fear);
            }

            RSF_BO_PLAS => 'b: {
                z = 10 + (8 * 7) + spell_power;
                bolt = true;
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_bo_plas_fear);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_bo_plas_fear_2);
                }
            }

            RSF_BO_ICE => 'b: {
                z = (6 * 6) + spell_power;
                bolt = true;
                // Add fear for the effect
                p += param(Param::danger_rsf_bo_ice_fear);
                if st.view.resistances.pstun != 0 {
                    break 'b;
                }
                // if already stunned be REALLY nervous about this
                if st.view.status.isstun != 0 {
                    z += param(Param::danger_rsf_bo_ice_fear_2);
                }
                if st.view.status.isheavystun != 0 {
                    z += param(Param::danger_rsf_bo_ice_fear_3);
                }
            }

            RSF_MISSILE => {
                z = (2 * 6) + (spell_power / 3);
                bolt = true;
            }

            RSF_BE_ELEC => 'b: {
                if st.view.resistances.ielec != 0 {
                    break 'b;
                }
                z = (5 * 5) + (spell_power * 2) + 30;
                if st.view.resistances.relec != 0 {
                    z = (z + 2) / 3;
                }
                if st.view.threat.temporary.res_elec && st.view.resistances.velec == 0 {
                    z = (z + 2) / 3;
                }
                if st.view.resistances.velec != 0 && !st.view.threat.temporary.res_elec {
                    z = z * 4 / 3;
                }
                bolt = true;
            }

            RSF_BE_NETH => {
                bolt = true;
                z = (5 * 5) + (spell_power * 2) + 30;
                if st.view.resistances.rnthr != 0 {
                    z = (z * 6) / 8;
                }
            }

            RSF_SCARE => {
                if st.view.abilities.sav < 100 {
                    // Add fear for the effect
                    p += param(Param::danger_rsf_scare_fear);
                }
            }

            RSF_BLIND => 'b: {
                if st.view.resistances.rblind != 0 {
                    break 'b;
                }
                if st.view.abilities.sav >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_blind_fear);
            }

            RSF_CONF => 'b: {
                if st.view.resistances.rconf != 0 {
                    break 'b;
                }
                if st.view.abilities.sav >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_conf_fear);
            }

            RSF_SLOW => 'b: {
                if st.view.resistances.fract != 0 {
                    break 'b;
                }
                if st.view.abilities.sav >= 100 {
                    break 'b;
                }
                // Add fear for the effect
                p += param(Param::danger_rsf_slow_fear);
            }

            RSF_HOLD => 'b: {
                if st.view.resistances.fract != 0 {
                    break 'b;
                }
                if st.view.abilities.sav >= 100 {
                    break 'b;
                }
                p += param(Param::danger_rsf_hold_fear);
            }

            RSF_HASTE => {
                // Add fear for the effect
                p += param(Param::danger_rsf_haste_fear);
            }

            RSF_HEAL => {
                // Add fear for the effect
                p += param(Param::danger_rsf_heal_fear);
            }

            RSF_HEAL_KIN => {}

            RSF_BLINK => {}

            RSF_TPORT => {
                // Add fear for the effect
                p += param(Param::danger_rsf_tport_fear);
            }

            RSF_TELE_TO => {
                // Add fear for the effect
                p += param(Param::danger_rsf_tele_to_fear);
            }

            RSF_TELE_SELF_TO => {
                // Add fear for the effect
                p += param(Param::danger_rsf_tele_self_to_fear);
            }

            RSF_TELE_AWAY => {
                // Add fear for the effect
                p += param(Param::danger_rsf_tele_away_fear);
            }

            RSF_TELE_LEVEL => {
                if st.view.abilities.sav < 100 {
                    // Add fear for the effect
                    p += param(Param::danger_rsf_tele_level_fear);
                }
            }

            RSF_DARKNESS => {
                // Add fear for the effect
                p += param(Param::danger_rsf_darkness_fear);
            }

            RSF_TRAPS => {
                // Add fear for the effect
                p += param(Param::danger_rsf_traps_fear);
            }

            RSF_FORGET => {
                if st.view.abilities.sav < 100 {
                    // if you are a spell caster, this is a big issue
                    if st.view.character.cursp < param(Param::danger_forget_low_sp) {
                        p += param(Param::danger_rsf_forget_fear);
                    } else {
                        p += param(Param::danger_rsf_forget_fear_2);
                    }
                }
            }

            RSF_SHAPECHANGE => {
                // Add fear for the effect
                p += param(Param::danger_rsf_shapechange_fear);
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
                    p += spell_power * param(Param::danger_rsf_s_kin_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_kin_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_kin_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_HI_DEMON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power
                        * param(Param::danger_rsf_s_hi_demon_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_hi_demon_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_hi_demon_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_MONSTER => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_monster_power_multiplier);
                    p /= spot_safe;
                }
            }

            RSF_S_MONSTERS => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_monsters_power_multiplier);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_ANIMAL | RSF_S_SPIDER | RSF_S_HOUND => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || glyph || create_door || near_unique {
                    p += 0;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_animal_power_multiplier);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_HYDRA => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power * param(Param::danger_rsf_s_hydra_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_hydra_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_hydra_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_AINU => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_ainu_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_ainu_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_ainu_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_DEMON | RSF_S_UNDEAD | RSF_S_DRAGON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power * param(Param::danger_rsf_s_demon_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_demon_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_demon_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = (p * param(Param::danger_summon_unique_percent)) / 100;
                }
            }

            RSF_S_HI_UNDEAD | RSF_S_WRAITH => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power
                        * param(Param::danger_rsf_s_hi_undead_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_hi_undead_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_hi_undead_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_HI_DRAGON => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    // (C: no `p += spell_power` here, unlike the others)
                    p /= spot_safe;
                } else if glyph || create_door || near_unique {
                    p += spell_power * param(Param::danger_rsf_s_hi_dragon_power_multiplier);
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_hi_dragon_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            RSF_S_UNIQUE => {
                count_safe_spots(st, kill, &mut spot_safe);
                if pfe {
                    p += spell_power * param(Param::danger_rsf_s_unique_protected_power_multiplier);
                    p /= spot_safe;
                } else if glyph || create_door {
                    p += spell_power * param(Param::danger_rsf_s_unique_power_multiplier); // slightly reduced danger for unique
                    p /= spot_safe;
                } else {
                    p += spell_power * param(Param::danger_rsf_s_unique_power_multiplier_2);
                    p /= spot_safe;
                }
                // reduce the fear if it is a unique
                if unique {
                    p = p * param(Param::danger_summon_unique_percent) / 100;
                }
            }

            _ => {}
        }

        // A bolt spell cannot jump monsters to hit the borg.
        if bolt
            && !st
                .facts
                .projectable_pure(kill.pos.y, kill.pos.x, st.view.y, st.view.x)
        {
            z = 0;
        }

        // Some borgs are concerned with the 'effects' of an attack.  ie, cold
        // attacks shatter potions, fire attacks burn scrolls, electric attacks
        // zap rings.
        if st.view.character.maxdepth >= param(Param::danger_ignore_side_effects_depth) {
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
    total_dam -= st.view.abilities.dam_red;
    if total_dam < 0 {
        total_dam = 0;
    }

    // Slightly decrease the danger if the borg is sitting in
    // a sea of runes.
    if st.view.threat.morgoth_position || st.view.core_status.anti_summon {
        total_dam = total_dam * param(Param::danger_corridor_spell_tenths) / 10;
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
    if n >= av * param(Param::danger_worst_spell_average_tenths) / 10
        || n > st.view.character.curhp * param(Param::danger_worst_spell_hp_tenths) / 10
    {
        n
    } else {
        // Average Danger
        av
    }
}

/// Is the grid a wall/seam grid for the move simulation of
/// `borg_danger_one_kill`? Both loops test `feat == FEAT_GRANITE`: features
/// above granite (permanent wall, lava, passable rubble) are not walls here.
/// The monster's sleep 2 spell condition on its level (v1 part).
fn sleepable(st: &Assessment<'_>, kill: &BorgKill) -> bool {
    !rf(kill, RF_NO_SLEEP)
        && !rf(kill, RF_UNIQUE)
        && i32::from(kill.level)
            <= st.view.character.clevel - param(Param::danger_sleep_level_margin)
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
/// `borg_estimate_monster()` on a copy of `borg_kills[i]`.
#[allow(clippy::too_many_arguments)]
fn estimate_monster(
    st: &Assessment<'_>,
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

    let mut fake_speed: i32 = st.view.character.speed;
    let mut monster_speed: i32 = i32::from(kill.speed);

    // Paranoia
    if kill.r_idx == 0 {
        return 0;
    }

    // "player ghosts"
    if i32::from(kill.r_idx) >= i32::from(facts::z_info().r_max) - 1 {
        return param(Param::danger_ghost_monster);
    }

    let r_ptr = facts::race(usize::from(kill.r_idx));
    let flags = st.view.threat.simulation;

    // Skip certain monster indexes.
    // These have been listed mainly in Teleport Other
    // checks in borg6.c in the defense maneuvers.
    if st.view.threat.teleport_ignored().contains(&i) {
        return 0;
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
    if d > param(Param::danger_monster_max_distance) {
        return 0;
    }

    // A very speedy borg will miscalculate danger of some monsters
    if st.view.character.speed >= param(Param::danger_fast_player_threshold) {
        fake_speed = if st.view.threat.nearby_unique != 0 {
            param(Param::danger_fast_player_unique_speed)
        } else {
            param(Param::danger_fast_player_normal_speed)
        };
    }

    // Consider the character haste and slow monster spells
    if st.view.threat.temporary.fast {
        fake_speed += 10;
    }
    if flags.slow_spell {
        monster_speed -= 10;
    }

    // Assume monsters are a little fast when you are low level
    if st.view.character.maxhp < param(Param::danger_low_hp_speed_limit)
        && st.view.character.cdepth != 0
    {
        monster_speed += param(Param::danger_low_hp_monster_speed_bonus);
    }

    // Player energy per game turn
    let mut e = i32::from(facts::extract_energy(fake_speed as usize));

    // Game turns per player move
    let t = (100 + (e - 1)) / e;

    // Monster energy per game turn
    e = i32::from(facts::extract_energy(monster_speed as usize));

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
    if q <= param(Param::danger_minimum_monster_energy) {
        q = param(Param::danger_minimum_monster_energy);
    }

    // Danger from physical attacks

    // Physical attacks
    v1 = physical_estimate(st, kill, flags, full_damage);

    // If the Borg has been bouncing for a long time, or if the time this
    // level is very high, reduce the danger value. This helps prevent the
    // borg from getting stuck due to overestimating danger and refusing to
    // move in both this area of the dungeon and over the course of the game.
    if i32::from(st.view.threat.antibounce_count) > param(Param::danger_stuck_bounce_limit)
        || st.view.threat.clock.now - st.view.threat.clock.level
            > param(Param::danger_stuck_level_turns)
    {
        v1 /= param(Param::danger_stuck_melee_divisor);
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
    if rf(kill, RF_MULTIPLY)
        && st.view.character.clevel < param(Param::danger_breeder_melee_max_level)
    {
        // extra 50%
        v1 += v1 * param(Param::danger_breeder_melee_extra_tenths) / 10;
    }

    let has_friends = r_ptr.has_friends;

    // Friends yield some trouble when I am weak
    if has_friends && st.view.character.clevel < param(Param::danger_friends_melee_max_level) {
        if st.view.character.clevel < param(Param::danger_friends_melee_low_level) {
            // extra 80%
            v1 += v1 * param(Param::danger_friends_melee_low_extra_tenths) / 10;
        } else {
            // extra 30%
            v1 += v1 * param(Param::danger_friends_melee_extra_tenths) / 10;
        }
    }

    // Reduce danger from sleeping monsters
    if !kill.awake {
        let inc = r_ptr.sleep + param(Param::danger_sleeping_melee_alertness);
        // Reduce the fear if Borg is higher level
        if st.view.character.clevel >= param(Param::danger_sleeping_melee_min_level) {
            v1 /= param(Param::danger_sleeping_melee_divisor);
        }

        // Tweak danger based on the "alertness" of the monster
        // increase the danger for light sleepers
        v1 += v1 * inc / 100;
    }
    // Reduce danger from sleeping monsters with the sleep 2 spell
    if flags.sleep_spell_ii && d == 1 && kill.awake && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.view.character.clevel < param(Param::danger_sleep2_melee_weak_level)
            && st.view.character.curhp
                < st.view.character.maxhp / param(Param::danger_sleep2_melee_weak_hp_divisor)
        {
            v1 = 0;
        } else {
            v1 /= param(Param::danger_sleep2_melee_divisor);
        }
    }
    // Reduce danger from sleeping monsters with the sleep 1,3 spell
    if flags.sleep_spell && kill.awake && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.view.character.clevel < param(Param::danger_sleep_melee_weak_level)
            && st.view.character.curhp
                < st.view.character.maxhp / param(Param::danger_sleep_melee_weak_hp_divisor)
        {
            v1 = 0;
        } else {
            v1 /= d + param(Param::danger_sleep_melee_distance_offset);
        }
    }
    if flags.crush_spell && crushable(st, kill) {
        v1 = 0;
    }

    // Reduce danger from confused monsters
    if kill.confused {
        v1 /= param(Param::danger_confused_melee_divisor);
    }
    if kill.stunned {
        v1 = v1 * param(Param::danger_stunned_melee_numerator)
            / param(Param::danger_stunned_melee_divisor);
    }
    if flags.confuse_spell && kill.awake && !kill.confused && sleepable(st, kill) {
        // Under special circumstances force the damage to 0
        if st.view.character.clevel < param(Param::danger_confuse_spell_weak_level)
            && st.view.character.curhp
                < st.view.character.maxhp / param(Param::danger_confuse_spell_weak_hp_divisor)
        {
            v1 = 0;
        } else {
            v1 /= d + param(Param::danger_confuse_spell_distance_offset);
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
            let y_temp = y9 + i32::from(facts::ddy_ddd(ii));
            let x_temp = x9 + i32::from(facts::ddx_ddd(ii));

            // Check for legality
            if !facts::square_in_bounds_fully(x_temp, y_temp) {
                continue;
            }

            let g = *st.grid(x_temp, y_temp);
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
            // borg-reborn-rs (#224): first pure shadow port.
            if crate::shadow::wall_like(feat) {
                // legally on a wall of some sort, check for closeness
                if rf(kill, RF_PASS_WALL) && facts::distance(y_temp, x_temp, y, x) == 1 {
                    b_v1 = v1;
                }
                if rf(kill, RF_KILL_WALL) && facts::distance(y_temp, x_temp, y, x) == 1 {
                    b_v1 = v1;
                }
            }

            // Is this grid being considered adjacent to the grid for which
            // the borg_danger() was called?
            if facts::distance(y_temp, x_temp, y, x) > 1 {
                continue;
            }

            // A legal floor grid
            if st.facts.floor(y_temp, x_temp) {
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
    v2 = spell_estimate(st, kill, flags, y, x, d, average);

    // Never cast spells
    if r_ptr.freq_innate == 0 && r_ptr.freq_spell == 0 {
        v2 = 0;
    }

    // Verify distance
    if facts::distance(y9, x9, y, x) > i32::from(facts::z_info().max_range) {
        v2 = 0;
    }

    // Verify line of sight (both ways) for monsters who can only move 1
    // grid.
    if q <= 10 && !st.facts.projectable(y9, x9, y, x) && !st.facts.projectable(y, x, y9, x9) {
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
            let y_temp = y9 + i32::from(facts::ddy_ddd(ii));
            let x_temp = x9 + i32::from(facts::ddx_ddd(ii));

            // Check for legality
            if !facts::square_in_bounds_fully(x_temp, y_temp) {
                continue;
            }

            let g = *st.grid(x_temp, y_temp);
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
            // borg-reborn-rs (#224): first pure shadow port.
            if crate::shadow::wall_like(feat) {
                // legally on a wall of some sort, check for LOS
                if rf(kill, RF_PASS_WALL) && st.facts.projectable(y_temp, x_temp, y, x) {
                    b_v2 = v2 * b_q / 10;
                }
                if rf(kill, RF_KILL_WALL) && st.facts.projectable(y_temp, x_temp, y, x) {
                    b_v2 = v2 * b_q / 10;
                }
            }
            // Monster on a legal floor grid.  Can he see me?
            else if st.facts.projectable(y_temp, x_temp, y, x) {
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
    if i32::from(st.view.threat.antibounce_count) > param(Param::danger_stuck_bounce_limit)
        || st.view.threat.clock.now - st.view.threat.clock.level
            > param(Param::danger_stuck_level_turns)
    {
        v2 /= param(Param::danger_stuck_spell_divisor);
    }

    // multipliers yield some trouble when I am weak
    if rf(kill, RF_MULTIPLY)
        && st.view.character.clevel < param(Param::danger_breeder_spell_max_level)
    {
        v2 += v2 * param(Param::danger_breeder_spell_extra_tenths) / 10;
    }

    // Friends yield some trouble when I am weak
    if has_friends && st.view.character.clevel < param(Param::danger_friends_spell_max_level) {
        v2 += v2 * param(Param::danger_friends_spell_extra_tenths) / 10;
    }

    // Reduce danger from sleeping monsters
    if !kill.awake {
        let inc = r_ptr.sleep + param(Param::danger_sleeping_spell_alertness);
        // weaklings and should still fear
        if st.view.character.clevel >= param(Param::danger_sleeping_spell_min_level) {
            v2 /= param(Param::danger_sleeping_spell_divisor);
        }

        // Tweak danger based on the "alertness" of the monster
        // increase the danger for light sleepers
        v2 += v2 * inc / 100;
    }

    // Reduce danger from sleeping monsters with the sleep 2 spell
    if flags.sleep_spell_ii {
        let clevel = st.view.character.clevel;
        let max_level = if clevel < param(Param::danger_sleep2_spell_linear_level) {
            clevel
        } else {
            ((clevel - param(Param::danger_sleep2_spell_level_subtract))
                / param(Param::danger_sleep2_spell_level_divisor))
                * param(Param::danger_sleep2_spell_level_multiplier)
                + param(Param::danger_sleep2_spell_level_add)
        };
        if d == 1
            && kill.awake
            && !rf(kill, RF_NO_SLEEP)
            && !rf(kill, RF_UNIQUE)
            && i32::from(kill.level) <= max_level
        {
            v2 /= param(Param::danger_sleep2_spell_divisor);
        }
    }

    // Clear the ranged danger when Crush would kill the monster.
    if flags.crush_spell && crushable(st, kill) {
        v2 = 0;
    }

    // Reduce danger from sleeping monsters with the sleep 1,3 spell
    if flags.sleep_spell {
        v2 /= d + param(Param::danger_sleep_spell_distance_offset);
    }
    // Reduce danger from confused monsters
    if kill.confused {
        v2 /= param(Param::danger_confused_spell_divisor);
    }
    // Reduce danger from stunned monsters
    if kill.stunned {
        v2 = v2 * param(Param::danger_stunned_spell_numerator)
            / param(Param::danger_stunned_spell_divisor);
    }
    if flags.confuse_spell {
        v2 /= param(Param::danger_confuse_spell_divisor);
    }

    // (C: `#if 0` -- scared monsters still cast spells, they are still
    // dangerous)
    if !full_damage {
        // reduce for frequency.
        let chance = (r_ptr.freq_innate + r_ptr.freq_spell) / 2;
        if chance < param(Param::danger_frequency_low_percent) {
            v2 = (v2 * param(Param::danger_frequency_low_tenths)) / 10;
        } else if chance < param(Param::danger_frequency_mid_percent) {
            v2 = (v2 * param(Param::danger_frequency_mid_tenths)) / 10;
        } else if chance < param(Param::danger_frequency_high_percent) {
            v2 = (v2 * param(Param::danger_frequency_high_tenths)) / 10;
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
    if p > param(Param::danger_maximum) {
        param(Param::danger_maximum)
    } else {
        p
    }
}

/// The crush spell condition: the monster dies (less than 4x the borg's
/// level in hit points) and is in view on a floor grid.
fn crushable(st: &Assessment<'_>, kill: &BorgKill) -> bool {
    // Either it dies or it doesn't.  For it to die it must have less
    // than 4x the borgs leve in hp and be in view.
    // So power(total hp) times (100-injury(percent injured)) divided by 100
    if (i32::from(kill.power) * (100 - i32::from(kill.injury))) / 100 < st.view.character.clevel * 4
    {
        let ag = st.grid(kill.pos.x, kill.pos.y);
        if ag.info & BORG_VIEW != 0 && facts::floor_grid(ag) {
            return true;
        }
    }
    false
}

fn aggregate(st: &Assessment<'_>, y: i32, x: i32, c: i32, average: bool, full_damage: bool) -> i32 {
    let mut p: i32 = 0;

    if !facts::square_in_bounds(x, y) {
        return param(Param::danger_maximum);
    }

    // Base danger (from regional fear) but not within a vault.  Cheating the
    // floor grid
    if !st.facts.is_vault(x, y)
        && st.view.character.cdepth <= param(Param::danger_regional_fear_max_depth)
    {
        p += i32::from(st.view.threat.regional_fear(y, x)) * c;
    }

    // Reduce regional fear on Depth 100
    if st.view.character.cdepth == 100 && p >= param(Param::danger_morgoth_regional_fear_cap) {
        p = param(Param::danger_morgoth_regional_fear_cap);
    }

    // Added danger (from a lot of monsters).
    // But do not add it if we have been bouncing for too long, or monster's
    // in a vault.  The fear_monsters[][] can induce some bouncy behavior.
    if i32::from(st.view.threat.antibounce_count) <= param(Param::danger_monster_fear_bounce_limit)
        && !st.facts.is_vault(x, y)
    {
        p += i32::from(st.view.threat.monster_fear(y, x)) * c;
    }

    // C overrides the argument. Grid danger always uses full (worst case)
    // damage; reduced estimates come only from direct monster calls.
    let _ = full_damage;
    let full_damage = true;

    // Examine all the monsters
    for (i, &kill) in st.view.threat.monster_slots().iter().enumerate().skip(1) {
        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Collect danger from monster
        p += estimate_monster(st, &kill, y, x, c, i as i32, average, full_damage);
    }

    // Return the danger
    if p > param(Param::danger_maximum) {
        param(Param::danger_maximum)
    } else {
        p
    }
}

pub fn assess(view: &View<'_>, input: borg_rs::shadow::DangerInput) -> i32 {
    let assessment = Assessment {
        view,
        facts: Facts::new(view.state),
    };
    aggregate(
        &assessment,
        input.y,
        input.x,
        input.turns,
        input.average,
        input.full,
    )
}

pub fn assess_monster(view: &View<'_>, input: borg_rs::shadow::DangerInput, slot: i32) -> i32 {
    let assessment = Assessment {
        view,
        facts: Facts::new(view.state),
    };
    let kill = &view.threat.monster_slots()[slot as usize];
    estimate_monster(
        &assessment,
        kill,
        input.y,
        input.x,
        input.turns,
        slot,
        input.average,
        input.full,
    )
}

#[cfg(feature = "test-m13-danger")]
pub(crate) fn fixture_estimate(
    state: &borg_rs::memory::BorgState,
    kill: &BorgKill,
    physical: bool,
) -> i32 {
    let view = View::new(state);
    let context = Assessment {
        view: &view,
        facts: Facts::new(state),
    };
    if physical {
        physical_estimate(&context, kill, view.threat.simulation, true)
    } else {
        spell_estimate(
            &context,
            kill,
            view.threat.simulation,
            state.borg.c.y,
            state.borg.c.x,
            1,
            true,
        )
    }
}

#[cfg(test)]
mod grid_failure_tests {
    use super::*;

    #[test]
    fn invalid_grid_records_before_controlled_exit() {
        const CHILD: &str = "BORG_TEST_INVALID_DANGER_GRID";
        if let Ok(coordinates) = std::env::var(CHILD) {
            let mut state = borg_rs::memory::BorgState::default();
            state.borg.trait_ = vec![0; borg_rs::memory::traits::BI_MAX as usize];
            state.borg.time.now = 37;
            let view = View::new(&state);
            let assessment = Assessment {
                view: &view,
                facts: Facts::new(&state),
            };
            let (x, y) = coordinates.split_once(',').unwrap();
            assessment.grid(x.parse().unwrap(), y.parse().unwrap());
            panic!("invalid grid returned a reference");
        }
        let root = std::env::temp_dir().join(format!(
            "borg-danger-grid-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for mode in ["check", "switch"] {
            for (index, (x, y)) in [(-1, 0), (borg_rs::memory::AUTO_MAX_X, 1), (0, 0)]
                .into_iter()
                .enumerate()
            {
                let dir = root.join(format!("{mode}-{index}"));
                std::fs::create_dir_all(&dir).unwrap();
                let log = dir.join("shadow.jsonl");
                std::fs::write(&log, "").unwrap();
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "danger::grid_failure_tests::invalid_grid_records_before_controlled_exit",
                        "--nocapture",
                    ])
                    .env(CHILD, format!("{x},{y}"))
                    .env("BORG_RUN_DIR", &dir)
                    .env("BORG_SHADOW", mode)
                    .output()
                    .unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(2),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let contents = std::fs::read_to_string(log).unwrap();
                assert_eq!(contents.lines().count(), 1);
                let row: serde_json::Value = serde_json::from_str(&contents).unwrap();
                assert_eq!(row["schema"], "borg-reborn-rs/shadow@1");
                assert_eq!(row["mode"], mode);
                assert_eq!(row["point"], "danger.grid");
                assert_eq!(row["inputs"], serde_json::json!({"x":x,"y":y}));
                assert_eq!(row["borg_turn"], 37);
                assert_eq!(row["validation_stopped"], true);
                assert!(row["original"].is_null() && row["ported"].is_null());
                assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

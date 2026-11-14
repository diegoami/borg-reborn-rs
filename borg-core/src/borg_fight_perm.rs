//! Port of `borg-fight-perm.c`: do moves we want as persistent during a
//! fight and beyond.
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

use crate::borg_inventory::borg_slot;
use crate::borg_io::borg_note;
use crate::borg_item_use::{borg_activate_item, borg_equips_item, borg_quaff_potion};
use crate::borg_magic::{
    borg_get_spell_power, borg_heroism_level, borg_spell, borg_spell_fail, borg_spell_okay_fail,
};
use crate::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, BLESS, HASTE_SELF, HEROISM, MANA_CHANNEL, PROTECTION_FROM_EVIL,
    RAPID_REGENERATION, RESISTANCE, RESIST_POISON, SENSE_INVISIBLE, SMITE_EVIL, VENOM,
};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CURSP, BI_ISBLIND, BI_ISCONFUSED, BI_ISFORGET, BI_LIGHT, BI_MAXHP,
    BI_MAXSP, BI_SINV, BI_WB_POIS, BI_WS_EVIL,
};
use crate::headers::list_tvals_h::TV_POTION;
use crate::state::BorgState;

/// `BORG_GLOW` (borg-cave.h grid info bit).
const BORG_GLOW: u16 = 0x02;

/*
 * Perma spells.  Some are cool to have on all the time, so long as their
 * mana cost is not too much.
 * There are several types of setup moves:
 *
 *   Temporary speed
 *   Protect From Evil
 *   Prayer
 *   Temp Resist (either all or just cold/fire?)
 *   Shield
 */
const BP_SPEED: i32 = 0;
const BP_PROT_FROM_EVIL: i32 = 1;
const BP_BLESS: i32 = 2;
const BP_RESIST_ALL: i32 = 3;
const BP_RESIST_ALL_COLLUIN: i32 = 4;
const BP_RESIST_P: i32 = 5;
const BP_FASTCAST: i32 = 6;
const BP_HERO: i32 = 7;
const BP_BERSERK: i32 = 8;
const BP_BERSERK_POTION: i32 = 9;
const BP_SMITE_EVIL: i32 = 10;
const BP_VENOM: i32 = 11;
const BP_REGEN: i32 = 12;
const BP_GLYPH: i32 = 13;
const BP_SEE_INV: i32 = 14;
const BP_MAX: i32 = 15;

/// The allowed fail rate: `base`, raised to `if_mon` when a unique is on
/// the level and to `if_near` when one is near.
fn fail_allowed(st: &BorgState, base: i32, if_mon: i32, if_near: i32) -> i32 {
    let mut fail_allowed = base;

    /* increase the threshold */
    if st.borg.mon.unique != 0 {
        fail_allowed = if_mon;
    }
    if st.borg.near.unique != 0 {
        fail_allowed = if_near;
    }
    fail_allowed
}

/// `cost >= (borg.mon.unique ? CURSP / 7 : CURSP / 10)`: the spell is not
/// cheap enough.
fn too_costly(st: &BorgState, cost: i32) -> bool {
    cost >= if st.borg.mon.unique != 0 {
        st.borg.tr(BI_CURSP) / 7
    } else {
        st.borg.tr(BI_CURSP) / 10
    }
}

/// `borg.trait[BI_ISBLIND] || borg.trait[BI_ISCONFUSED]`
fn blind_or_confused(st: &BorgState) -> bool {
    st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0
}

/// `borg.trait[BI_CLEVEL] * 1000` stored in `borg.no_rest_prep`.
fn level_times_1000(st: &BorgState) -> i32 {
    st.borg.tr(BI_CLEVEL) * 1000
}

/// Prayer to prepare for battle
fn borg_perma_aux_bless(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 15, 20, 25);

    /* already blessed */
    if st.borg.temp.bless {
        return 0;
    }

    /* Cant when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    /* XXX Dark */

    if !borg_spell_okay_fail(st, BLESS, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, BLESS);

    /* If its cheap, go ahead */
    if st.borg.tr(BI_CLEVEL) > 10 && too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* bless is a low priority */
    if st.traits.simulate {
        return 1;
    }

    /* do it! */
    borg_spell(st, BLESS);

    /* No resting to recoup mana */
    st.borg.no_rest_prep = 10000;

    1
}

/// all resists FECAP
fn borg_perma_aux_resist(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    let t = &st.borg.temp;
    if i32::from(t.res_fire) + i32::from(t.res_acid) + i32::from(t.res_elec)
        + i32::from(t.res_cold)
        >= 3
    {
        return 0;
    }

    if !borg_spell_okay_fail(st, RESISTANCE, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, RESISTANCE);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 2;
    }

    /* do it! */
    borg_spell_fail(st, RESISTANCE, fail_allowed);

    /* No resting to recoup mana */
    st.borg.no_rest_prep = 21000;

    /* default to can't do it. */
    2
}

/// all resists from the cloak
fn borg_perma_aux_resist_colluin(st: &mut BorgState) -> i32 {
    let t = &st.borg.temp;
    if i32::from(t.res_fire)
        + i32::from(t.res_acid)
        + i32::from(t.res_pois)
        + i32::from(t.res_elec)
        + i32::from(t.res_cold)
        >= 3
    {
        return 0;
    }

    /* Only use it when Unique is close */
    if st.borg.near.unique == 0 {
        return 0;
    }

    if !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
    {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 2;
    }

    /* do it! */
    if borg_activate_item(st, st.act.resist_all)
        || borg_activate_item(st, st.act.rage_bless_resist)
    {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 21000;
    }

    /* Value */
    2
}

/// resists--- Only bother if a Unique is on the level.
fn borg_perma_aux_resist_p(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    if st.borg.temp.res_pois || st.borg.mon.unique == 0 {
        return 0;
    }

    if !borg_spell_okay_fail(st, RESIST_POISON, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, RESIST_POISON);

    /* If its cheap, go ahead */
    if cost >= st.borg.tr(BI_CURSP) / 20 {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 1;
    }

    /* do it! */
    if borg_spell_fail(st, RESIST_POISON, fail_allowed) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 21000;

        /* Value */
        return 1;
    }

    /* default to can't do it. */
    0
}

/// Speed to prepare for battle
fn borg_perma_aux_speed(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 7, 10, 15);

    /* already fast */
    if st.borg.temp.fast {
        return 0;
    }

    /* only cast defense spells if fail rate is not too high */
    if !borg_spell_okay_fail(st, HASTE_SELF, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, HASTE_SELF);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 5;
    }

    /* do it! */
    if borg_spell_fail(st, HASTE_SELF, fail_allowed) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = level_times_1000(st);
        return 5;
    }

    /* default to can't do it. */
    0
}

/// PFE to prepare for battle
fn borg_perma_aux_prot_evil(st: &mut BorgState) -> i32 {
    /* if already protected */
    if st.borg.temp.prot_from_evil {
        return 0;
    }

    let fail_allowed = fail_allowed(st, 5, 10, 15);

    if !borg_spell_okay_fail(st, PROTECTION_FROM_EVIL, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, PROTECTION_FROM_EVIL);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 3;
    }

    /* do it! */
    if borg_spell_fail(st, PROTECTION_FROM_EVIL, fail_allowed) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = level_times_1000(st);

        /* Value */
        return 3;
    }

    /* default to can't do it. */
    0
}

/// Mana Channel to prepare for battle
fn borg_perma_aux_fastcast(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already fast */
    if st.borg.temp.fastcast {
        return 0;
    }

    /* Cant when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, MANA_CHANNEL, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, MANA_CHANNEL);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* fastcast is a low priority */
    if st.traits.simulate {
        return 5;
    }

    /* do it! */
    if borg_spell(st, MANA_CHANNEL) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 6000;
        return 1;
    }

    0
}

/// Hero to prepare for battle
fn borg_perma_aux_hero(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already heroism */
    if st.borg.temp.hero {
        return 0;
    }

    /* Can't when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    /* Heroism part of the Heroism spell doesn't kick in till later */
    if st.borg.tr(BI_CLEVEL) <= borg_heroism_level(st) {
        return 0;
    }

    /* XXX Dark */

    if !borg_spell_okay_fail(st, HEROISM, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, HEROISM);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* hero is a low priority */
    if st.traits.simulate {
        return 1;
    }

    /* do it! */
    if borg_spell(st, HEROISM) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 3000;
        return 1;
    }

    0
}

/// Rapid Regen to prepare for battle
///
/// C bug (kept): there is no simulation check, so the spell is cast while
/// `borg_perma_spell` is only simulating (see docs/c-bugs.md).
fn borg_perma_aux_regen(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already regenerating */
    if st.borg.temp.regen {
        return 0;
    }

    /* Cant when screwed */
    if blind_or_confused(st) || st.borg.tr(BI_ISFORGET) != 0 {
        return 0;
    }

    /* don't bother if not much to regenerate */
    if st.borg.tr(BI_MAXHP) < 100 {
        return 0;
    }

    if !borg_spell_okay_fail(st, RAPID_REGENERATION, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, RAPID_REGENERATION);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    if st.traits.simulate {
        return 1;
    }

    /* do it! */
    if borg_spell(st, RAPID_REGENERATION) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 6000;
        return 1;
    }

    0
}

/// Smite evil to prepare for battle
fn borg_perma_aux_smite_evil(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already smiting */
    if st.borg.temp.smite_evil || st.borg.tr(BI_WS_EVIL) != 0 {
        return 0;
    }

    /* Cant when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, SMITE_EVIL, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, SMITE_EVIL);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* smite evil is a low priority */
    if st.traits.simulate {
        return 3;
    }

    /* do it! */
    if borg_spell(st, SMITE_EVIL) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 21000;
        return 3;
    }

    0
}

/// Poison your weapon to prepare for battle
///
/// C bug (kept): the cost is that of Smite Evil, not Venom.
fn borg_perma_aux_venom(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already smiting */
    if st.borg.temp.venom || st.borg.tr(BI_WB_POIS) != 0 {
        return 0;
    }

    /* Cant when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, VENOM, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, SMITE_EVIL);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* smite evil is a low priority */
    if st.traits.simulate {
        return 3;
    }

    /* do it! */
    if borg_spell(st, VENOM) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 19000;
        return 3;
    }

    0
}

/// Berserk to prepare for battle
fn borg_perma_aux_berserk(st: &mut BorgState) -> i32 {
    let fail_allowed = fail_allowed(st, 5, 10, 15);

    /* already blessed */
    if st.borg.temp.berserk {
        return 0;
    }

    /* Cant when Blind */
    if blind_or_confused(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, BERSERK_STRENGTH, fail_allowed) {
        return 0;
    }

    /* Obtain the cost of the spell */
    let cost = borg_get_spell_power(st, BERSERK_STRENGTH);

    /* If its cheap, go ahead */
    if too_costly(st, cost) {
        return 0;
    }

    /* Simulation */
    /* Berserk is a low priority */
    if st.traits.simulate {
        return 2;
    }

    /* do it! */
    if borg_spell(st, BERSERK_STRENGTH) {
        /* No resting to recoup mana */
        st.borg.no_rest_prep = 11000;
        return 2;
    }

    0
}

/// Berserk to prepare for battle
fn borg_perma_aux_berserk_potion(st: &mut BorgState) -> i32 {
    /* Saver the potions */
    if st.borg.near.unique == 0 {
        return 0;
    }

    /* already blessed */
    if st.borg.temp.hero || st.borg.temp.berserk {
        return 0;
    }

    /* do I have any? */
    if -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_berserk) {
        return 0;
    }

    /* Simulation */
    /* Berserk is a low priority */
    if st.traits.simulate {
        return 2;
    }

    /* do it! */
    if borg_quaff_potion(st, st.item_val.sv_potion_berserk) {
        return 2;
    }

    0
}

/// Detect Inviso/Monsters
/// Casts detect invis.
fn borg_perma_aux_see_inv(st: &mut BorgState) -> i32 {
    let fail_allowed = 25;

    /* no need */
    if blind_or_confused(st) || st.borg.tr(BI_SINV) != 0 || st.borg.see_inv != 0 {
        return 0;
    }

    /* Do I have anything that will work? */
    if !borg_spell_okay_fail(st, SENSE_INVISIBLE, fail_allowed) {
        return 0;
    }

    /* Darkness */
    let info = st.cave.grid(st.borg.c.x, st.borg.c.y).info;
    if info & BORG_GLOW == 0 && st.borg.tr(BI_LIGHT) == 0 {
        return 0;
    }

    /* No real value known, but lets cast it to find the bad guys. */
    if st.traits.simulate {
        return 10;
    }

    /* long time */
    if borg_spell_fail(st, SENSE_INVISIBLE, fail_allowed) {
        st.borg.see_inv = 32000;
        st.borg.no_rest_prep = 16000;
        return 10;
    }

    /* ah crap, I guess I won't be able to see them */
    0
}

/// Simulate/Apply the optimal result of using the given "type" of set-up
fn borg_perma_aux(st: &mut BorgState, what: i32) -> i32 {
    /* Analyze */
    match what {
        BP_SPEED => borg_perma_aux_speed(st),
        BP_PROT_FROM_EVIL => borg_perma_aux_prot_evil(st),
        BP_RESIST_ALL => borg_perma_aux_resist(st),
        BP_RESIST_ALL_COLLUIN => borg_perma_aux_resist_colluin(st),
        BP_RESIST_P => borg_perma_aux_resist_p(st),
        BP_BLESS => borg_perma_aux_bless(st),
        BP_FASTCAST => borg_perma_aux_fastcast(st),
        BP_HERO => borg_perma_aux_hero(st),
        BP_BERSERK => borg_perma_aux_berserk(st),
        BP_BERSERK_POTION => borg_perma_aux_berserk_potion(st),
        BP_SMITE_EVIL => borg_perma_aux_smite_evil(st),
        BP_VENOM => borg_perma_aux_venom(st),
        BP_REGEN => borg_perma_aux_regen(st),
        /* return (borg_perma_aux_glyph()); Tends to use too much mana doing
         * this */
        BP_GLYPH => 0,
        BP_SEE_INV => borg_perma_aux_see_inv(st),
        _ => 0,
    }
}

/// Walk around with certain spells on if you can afford to do so.
pub fn borg_perma_spell(st: &mut BorgState) -> bool {
    let mut b_n = 0;
    let mut b_g = -1;

    /* Simulate */
    st.traits.simulate = true;

    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Not in shallow dungeon */
    if st.borg.tr(BI_CDEPTH) < st.borg.tr(BI_CLEVEL) / 3 || st.borg.tr(BI_CDEPTH) < 7 {
        return false;
    }

    /* Low Level, save your mana, use the defense maneuvers above */
    if st.borg.tr(BI_CLEVEL) <= 10 {
        return false;
    }

    /* Only when lots of mana is on hand */
    if st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) * 75 / 100 {
        return false;
    }

    /* Analyze the possible setup moves */
    for g in 0..BP_MAX {
        /* Simulate */
        let n = borg_perma_aux(st, g);

        /* Track "best" move */
        if n <= b_n {
            continue;
        }

        /* Track best */
        b_g = g;
        b_n = n;
    }

    /* Nothing good */
    if b_n <= 0 {
        return false;
    }

    /* Note */
    borg_note(st, &format!("# Performing perma-spell type {b_g} with value {b_n}"));

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    borg_perma_aux(st, b_g);

    /* Success */
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;

    #[test]
    fn no_rest_prep_does_not_wrap_at_high_level() {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_CLEVEL as usize] = 35;
        st.borg.no_rest_prep = level_times_1000(&st);
        assert_eq!(st.borg.no_rest_prep, 35_000);
        assert!(st.borg.no_rest_prep > 0);
    }
}

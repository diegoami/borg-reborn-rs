//! Port of `borg-attack-munchkin.c`: attacks while in "munchkin mode",
//! which is super cautious.
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
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_danger::borg_danger;
use crate::borg_fight_attack::borg_calculate_attack_effectiveness;
use crate::borg_io::borg_note;
use crate::borg_headers::borg_fight_attack_h::{BF_MAX, BF_THRUST};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLASS, BI_CURHP, BI_ISCONFUSED, BI_MAXHP, BI_SPEED,
};
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::state::BorgState;
use crate::engine;

/// `CLASS_MAGE`, `CLASS_NECROMANCER` (player-class indexes)
const CLASS_MAGE: i32 = 1;
const CLASS_NECROMANCER: i32 = 4;

/// `BORG_OKAY` (borg-cave.h grid info bit).
const BORG_OKAY: u16 = 0x08;

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// "Must be standing on a stair"
fn on_stair(st: &BorgState) -> bool {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    feat == FEAT_MORE || feat == FEAT_LESS
}

/// "Not if too dangerous": danger above 70% of the avoidance, or HP below
/// a third.
fn too_dangerous(st: &mut BorgState) -> bool {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger(st, y, x, 1, true, true) > i32::from(st.borg.avoidance) * 7 / 10
        || st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 3
}

/// Munchkin Attack - Magic
///
/// The early mages have a very difficult time surviving until they level up
/// some. This routine will allow the mage to do some very limited attacking
/// while he is doing the munchkin start (stair scumming for items).
///
/// Basically, he will rest on stairs to recuperate mana, then use MM to
/// attack some easy to kill monsters.  If the monster gets too close, he
/// will flee via the stairs. He hope to be able to kill the monster in two
/// shots from the MM. A perfect scenario would be a mold which does not
/// move, then he could rest/shoot/rest.
pub fn borg_munchkin_mage(st: &mut BorgState) -> bool {
    let mut b_dam = -1;
    let mut b_n = -1;

    /* Must be standing on a stair */
    if !on_stair(st) {
        return false;
    }

    /* Not if too dangerous */
    if too_dangerous(st) {
        return false;
    }
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* Nobody around */
    if st.kills.cnt == 0 {
        return false;
    }

    /* Set the attacking flag so that danger is boosted for monsters */
    /* we want to attack first. */
    st.traits.attacking = true;

    /* Reset list */
    st.flow.temp_n = 0;

    /* Find "nearby" monsters */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = &st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Acquire location */
        let a = kill.pos;

        /* Not in town.  This should not be reached, but just in case we add
         * it */
        if st.borg.tr(BI_CDEPTH) == 0 {
            continue;
        }

        /* Check if there is a monster adjacent to me or he's close and
         * fast. */
        if (i32::from(kill.speed) > st.borg.tr(BI_SPEED) && engine::distance(st.borg.c, a) <= 2)
            || engine::distance(st.borg.c, a) <= 1
        {
            /* (borg_attacking stays set, as in C) */
            return false;
        }

        /* no attacking most scaryguys, try to get off the level */
        if st.borg.mon.scary {
            return false;
        }

        /* Acquire location */
        let c = kill.pos;

        /* Get grid */
        let info = st.cave.grid(c.x, c.y).info;

        /* Never shoot off-screen */
        if info & BORG_OKAY == 0 {
            continue;
        }

        /* Never shoot through walls */
        if info & BORG_VIEW == 0 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, c) > i32::from(engine::z_info().max_range) {
            continue;
        }

        /* Save the location (careful) */
        st.flow.temp_push(c.y, c.x);
    }

    /* No destinations */
    if st.flow.temp_n == 0 {
        st.traits.attacking = false;
        return false;
    }

    /* Simulate */
    st.traits.simulate = true;

    /* Simulated */
    for i in 0..BF_MAX as i32 {
        /* Skip certain ones */
        if i <= 1 {
            continue;
        }

        let dam = borg_calculate_attack_effectiveness(st, i);

        /* Track the best attack method */
        if dam >= b_dam && dam > 0 {
            b_dam = dam;
            b_n = i;
        }
    }

    /* Nothing good */
    if b_n < 0 || b_dam <= 0 {
        /* (borg_simulate stays set, as in C) */
        st.traits.attacking = false;
        return false;
    }

    /* Note */
    borg_note(st, &format!("# Performing munchkin attack with value {b_dam}."));

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    borg_calculate_attack_effectiveness(st, b_n);

    st.traits.attacking = false;

    /* Success */
    true
}

/// Munchkin Attack - Melee
///
/// The early borgs have a very difficult time surviving until they level
/// up some. This routine will allow the borg to do some very limited
/// attacking while he is doing the munchking start (stair scumming for
/// items).
///
/// Basically, he will rest on stairs to recuperate HP, then use melee to
/// attack some easy to kill adjacent monsters.
pub fn borg_munchkin_melee(st: &mut BorgState) -> bool {
    /* No Mages for now */
    if st.borg.tr(BI_CLASS) == CLASS_MAGE || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER {
        return false;
    }

    /* Must be standing on a stair */
    if !on_stair(st) {
        return false;
    }

    /* Nobody around */
    if st.kills.cnt == 0 {
        return false;
    }

    /* Not if too dangerous */
    if too_dangerous(st) {
        return false;
    }
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* Set the attacking flag so that danger is boosted for monsters */
    /* we want to attack first. */
    st.traits.attacking = true;

    /* Reset list */
    st.flow.temp_n = 0;

    /* Find "nearby" monsters */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = &st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Not in town.  This should not be reached, but just in case we add
         * it */
        if st.borg.tr(BI_CDEPTH) == 0 {
            continue;
        }

        /* no attacking most scaryguys, try to get off the level */
        if st.borg.mon.scary {
            /* (borg_attacking stays set, as in C) */
            return false;
        }

        /* Get grid */
        let pos = kill.pos;
        let info = st.cave.grid(pos.x, pos.y).info;

        /* Never shoot off-screen */
        if info & BORG_OKAY == 0 {
            continue;
        }

        /* Never shoot through walls */
        if info & BORG_VIEW == 0 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, pos) != 1 {
            continue;
        }

        /* Save the location (careful) */
        st.flow.temp_push(pos.y, pos.x);
    }

    /* No destinations */
    if st.flow.temp_n == 0 {
        st.traits.attacking = false;
        return false;
    }

    /* Simulate */
    st.traits.simulate = true;

    /* Simulated */
    let n = borg_calculate_attack_effectiveness(st, BF_THRUST as i32);

    /* Nothing good */
    if n <= 0 {
        /* (borg_simulate stays set, as in C) */
        st.traits.attacking = false;
        return false;
    }

    /* Note */
    borg_note(st, &format!("# Performing munchkin attack with value {n}."));

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    borg_calculate_attack_effectiveness(st, BF_THRUST as i32);

    st.traits.attacking = false;

    /* Success */
    true
}

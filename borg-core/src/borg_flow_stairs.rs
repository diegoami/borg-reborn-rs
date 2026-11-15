//! Port of `borg-flow-stairs.c`: flow (move) toward stairs.
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
//! The known up and down staircases (`track_less`, `track_more`) live in
//! `StairsState`. The flow itself is `borg_flow`.

use crate::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, BLESS, DEMON_BANE, HASTE_SELF, HEROISM, MANA_CHANNEL, PROTECTION_FROM_EVIL,
    RAPID_REGENERATION, RESISTANCE, SMITE_EVIL, VENOM,
};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_ISHUNGRY, BI_ISWEAK, BI_LIGHT,
    BI_MAXSP, BI_WB_POIS, BI_WS_EVIL,
};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::state::BorgState;
use crate::{borg_flow, borg_io, borg_magic, engine};

/// A list of known stairs (`struct borg_track` as used for `track_less`
/// and `track_more`). C semantics: `size` entries are allocated (zeroed),
/// the first `num` are in use; `size == 0` and empty `Vec`s = freed.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct StairTrack {
    /// Number of stairs in use
    pub num: i16,
    /// Number of entries allocated
    pub size: i16,
    /// Their x coordinates (`size` entries)
    pub x: Vec<i32>,
    /// Their y coordinates (`size` entries)
    pub y: Vec<i32>,
}

impl StairTrack {
    /// `borg_init_track(track, size)`: `size` zeroed entries, none in use.
    pub(crate) fn init(&mut self, size: i16) {
        self.num = 0;
        self.size = size;
        self.x = vec![0; size as usize];
        self.y = vec![0; size as usize];
    }

    /// `borg_free_track(track)`.
    pub(crate) fn free(&mut self) {
        self.num = 0;
        self.size = 0;
        self.x = Vec::new();
        self.y = Vec::new();
    }

    /// `(track.y[i], track.x[i])`, for `i < track.num`.
    pub fn at(&self, i: usize) -> (i32, i32) {
        assert!(i < self.num as usize, "stair index past the list");
        (self.y[i], self.x[i])
    }
}

/// borg-flow-stairs.c globals.
#[derive(Default)]
pub struct StairsState {
    /// `track_less`: the "up" stairs
    pub less: StairTrack,
    /// `track_more`: the "down" stairs
    pub more: StairTrack,
}

/// Which stair list to enqueue.
#[derive(Clone, Copy)]
enum Which {
    Less,
    More,
}

fn track(st: &BorgState, which: Which) -> &StairTrack {
    match which {
        Which::Less => &st.stairs.less,
        Which::More => &st.stairs.more,
    }
}

/// The "Enqueue useful grids" loop: every stair of the list without a
/// monster parked on it, in list order.
fn enqueue_stairs(st: &mut BorgState, which: Which) {
    let mut i = 0;
    while i < i32::from(track(st, which).num) {
        let (y, x) = track(st, which).at(i as usize);

        /* Not if a monster is parked on the stair */
        if st.cave.grid(x, y).kill != 0 {
            i += 1;
            continue;
        }

        /* Enqueue the grid */
        borg_flow::borg_flow_enqueue_grid(st, y, x);
        i += 1;
    }
}

/// Commit the flow and take one step (the common end of the three flows).
fn commit_and_step(st: &mut BorgState, who: &std::ffi::CStr, why: i32) -> bool {
    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, who, why) {
        return false;
    }

    /* Take one step */
    if !borg_flow::borg_flow_old(st, why) {
        return false;
    }

    /* Success */
    true
}

/// Do a Stair-Flow. Look at how far away this grid is to my closest stair.
pub fn borg_flow_cost_stair(st: &mut BorgState, y: i32, x: i32, b_stair: i32) -> i32 {
    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Paranoid */
    if b_stair == -1 {
        return 0;
    }

    /* Enqueue the player's grid (C reads any allocated entry) */
    let sy = st.stairs.less.y[b_stair as usize];
    let sx = st.stairs.less.x[b_stair as usize];
    borg_flow::borg_flow_enqueue_grid(st, sy, sx);

    /* Spread, but do NOT optimize */
    borg_flow::borg_flow_spread(st, 250, false, false, false, b_stair, false);

    /* Distance from the grid to the stair */
    let cost = i32::from(st.flow.cost_at(y, x));

    /* if it is impossible to get to the stairs, go off leash */
    if cost == 255 {
        return 0;
    }

    cost
}

/// Prepare to flee the level via stairs.
pub fn borg_flow_stair_both(st: &mut BorgState, why: i32, sneak: bool) -> bool {
    /* None to flow to */
    if st.stairs.less.num == 0 && st.stairs.more.num == 0 {
        return false;
    }

    /* don't go down if hungry or low on food, unless fleeing a scary town */
    if !st.borg.goal.fleeing
        && !st.borg.mon.scary
        && st.stairs.less.num == 0
        && i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP) * 15 / 10
        && (st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_FOOD) < 2)
    {
        return false;
    }

    /* Absolutely no diving if no light */
    if st.borg.tr(BI_LIGHT) == 0 && st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Enqueue useful grids */
    enqueue_stairs(st, Which::Less);
    enqueue_stairs(st, Which::More);

    /* Spread the flow */
    borg_flow::borg_flow_spread(st, 250, false, false, false, -1, sneak);

    commit_and_step(st, c"stairs", why)
}

/// Prepare to flow towards "up" stairs.
pub fn borg_flow_stair_less(st: &mut BorgState, why: i32, sneak: bool) -> bool {
    /* forced to go up */
    if engine::player_opt(OPT_birth_force_descend) {
        return false;
    }

    /* None to flow to */
    if st.stairs.less.num == 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Enqueue useful grids */
    enqueue_stairs(st, Which::Less);

    if st.borg.tr(BI_CLEVEL) > 35 || st.borg.tr(BI_LIGHT) == 0 {
        /* Spread the flow */
        borg_flow::borg_flow_spread(st, 250, true, false, false, -1, sneak);
    } else {
        /* Spread the flow, No Optimize, Avoid */
        let avoid = !st.borg.status.desperate;
        borg_flow::borg_flow_spread(st, 250, false, avoid, false, -1, sneak);
    }

    commit_and_step(st, c"up-stairs", why)
}

/// Prepare to flow towards "down" stairs.
pub fn borg_flow_stair_more(st: &mut BorgState, why: i32, sneak: bool, brave: bool) -> bool {
    /* None to flow to */
    if st.stairs.more.num == 0 {
        return false;
    }

    /* if there are no down stairs, don't filter use of up stairs */
    if st.stairs.less.num != 0 {
        /* not unless safe or brave */
        if !brave {
            let depth = st.borg.tr(BI_CDEPTH) + 1;
            if crate::borg_prepared::borg_prepared_c(st, depth).is_some() {
                return false;
            }
        }

        /* dont go down if hungry or low on food, unless fleeing a scary town */
        if !brave
            && st.borg.tr(BI_CDEPTH) != 0
            && !st.borg.mon.scary
            && (st.borg.tr(BI_ISWEAK) != 0
                || st.borg.tr(BI_ISHUNGRY) != 0
                || st.borg.tr(BI_FOOD) < 2)
        {
            return false;
        }

        /* If I need to sell crap, then don't go down */
        if crate::borg_trip::legacy_sale_hold(st) {
            return false;
        }

        /* No diving if no light */
        if st.borg.tr(BI_LIGHT) == 0 {
            return false;
        }
    }

    /* don't head for the stairs if you are recalling,  */
    /* even if you are fleeing. */
    if st.borg.goal.recalling != 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Enqueue useful grids */
    enqueue_stairs(st, Which::More);

    /* Spread the flow */
    borg_flow::borg_flow_spread(st, 250, true, false, false, -1, sneak);

    commit_and_step(st, c"down-stairs", why)
}

/// Note a spell cast before leaving the level and keep from resting.
fn prep_cast(st: &mut BorgState, what: &str, no_rest: i32) -> bool {
    borg_io::borg_note(st, what);
    st.borg.no_rest_prep = no_rest;
    true
}

/// Cast spells before leaving the level.
pub fn borg_prep_leave_level_spells(st: &mut BorgState) -> bool {
    /* if we are running away, just run. */
    if st.borg.goal.fleeing {
        return false;
    }

    /* if we are low on mana, don't prep. */
    if st.borg.tr(BI_CURSP) < (st.borg.tr(BI_MAXSP) * 6) / 10 {
        return false;
    }

    /* Cast haste */
    if !st.borg.temp.fast && borg_magic::borg_spell_fail(st, HASTE_SELF, 15) {
        return prep_cast(st, "# Casting speed spell before leaving level.", 5000);
    }

    /* Cast resistance */
    let t = &st.borg.temp;
    let resists = i32::from(t.res_fire)
        + i32::from(t.res_acid)
        + i32::from(t.res_elec)
        + i32::from(t.res_cold);
    if resists < 3 && borg_magic::borg_spell_fail(st, RESISTANCE, 15) {
        return prep_cast(st, "# Casting Resistance spell before leaving level.", 21000);
    }

    /* Cast fastcast */
    if !st.borg.temp.fastcast && borg_magic::borg_spell_fail(st, MANA_CHANNEL, 15) {
        return prep_cast(st, "# Casting Mana Channel spell before leaving level.", 6000);
    }

    /* Cast Berserk Strength */
    if !st.borg.temp.berserk && borg_magic::borg_spell_fail(st, BERSERK_STRENGTH, 15) {
        return prep_cast(st, "# Casting Berserk Strength spell before leaving level.", 10000);
    }

    /* Cast heroism if above level it gives temp attribute */
    if !st.borg.temp.hero
        && borg_magic::borg_spell_fail(st, HEROISM, 15)
        && st.borg.tr(BI_CLEVEL) > borg_magic::borg_heroism_level(st)
    {
        return prep_cast(st, "# Casting Heroism spell before leaving level.", 3000);
    }

    /* Cast regen just before returning to dungeon */
    if !st.borg.temp.regen && borg_magic::borg_spell_fail(st, RAPID_REGENERATION, 15) {
        return prep_cast(st, "# Casting Regen before leaving level.", 6000);
    }

    /* Cast Smite Evil just before returning to dungeon */
    if !st.borg.temp.smite_evil
        && st.borg.tr(BI_WS_EVIL) == 0
        && borg_magic::borg_spell_fail(st, SMITE_EVIL, 15)
    {
        return prep_cast(st, "# Casting Smite Evil before leaving level.", 21000);
    }

    /* Cast Venom just before returning to dungeon */
    if !st.borg.temp.venom
        && st.borg.tr(BI_WB_POIS) == 0
        && borg_magic::borg_spell_fail(st, VENOM, 15)
    {
        return prep_cast(st, "# Casting Venom before leaving level.", 18000);
    }

    /* Cast PFE just before returning to dungeon */
    if !st.borg.temp.prot_from_evil && borg_magic::borg_spell_fail(st, PROTECTION_FROM_EVIL, 15) {
        /* C bug 27: wraps past clevel 32 */
        let no_rest = st.borg.tr(BI_CLEVEL) * 1000;
        return prep_cast(st, "# Casting PFE before leaving level.", no_rest);
    }

    /* Cast bless prep things */
    if !st.borg.temp.bless
        && (borg_magic::borg_spell_fail(st, BLESS, 15)
            || borg_magic::borg_spell_fail(st, DEMON_BANE, 15))
    {
        return prep_cast(st, "# Casting blessing before leaving level.", 11000);
    }
    false
}

/// `borg_init_flow_stairs()`: 16 entries for each list.
pub fn borg_init_flow_stairs(st: &mut BorgState) {
    /* Track "up" stairs */
    st.stairs.less.init(16);

    /* Track "down" stairs */
    st.stairs.more.init(16);
}

/// `borg_free_flow_stairs()`.
pub fn borg_free_flow_stairs(st: &mut BorgState) {
    st.stairs.more.free();
    st.stairs.less.free();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_free_keep_c_sizes() {
        let mut t = StairTrack::default();
        t.init(16);
        assert_eq!((t.num, t.size, t.x.len(), t.y.len()), (0, 16, 16, 16));
        assert!(t.x.iter().chain(&t.y).all(|&v| v == 0));
        t.num = 1;
        t.x[0] = 5;
        t.y[0] = 7;
        assert_eq!(t.at(0), (7, 5));
        t.free();
        assert_eq!(t, StairTrack::default());
    }

    #[test]
    #[should_panic]
    fn at_past_num_panics() {
        let mut t = StairTrack::default();
        t.init(16);
        t.at(0);
    }
}

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

/// `GOAL_FLEE` (borg-flow.h): flee the level.
const GOAL_FLEE: i32 = 7;
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
    let (enqueued, parked) = enqueue_free_stairs(st, which);
    enqueue_parked_if_alone(st, enqueued, parked);
}

/// borg-reborn-rs (#597): enqueue the staircases of `which` with no monster
/// on them. Returns how many entered the flow (a grid the flow refuses, icky
/// or too dangerous, does not count) and the parked ones.
fn enqueue_free_stairs(st: &mut BorgState, which: Which) -> (usize, Vec<(i32, i32)>) {
    let mut parked = Vec::new();
    let mut enqueued = 0;
    let mut i = 0;
    while i < i32::from(track(st, which).num) {
        let (y, x) = track(st, which).at(i as usize);

        /* Not if a monster is parked on the stair */
        if st.cave.grid(x, y).kill != 0 {
            parked.push((y, x));
            i += 1;
            continue;
        }

        /* Enqueue the grid */
        borg_flow::borg_flow_enqueue_grid(st, y, x);
        // An enqueued grid has cost 0; a refused one keeps the cleared cost.
        if st.flow.cost_at(y, x) == 0 {
            enqueued += 1;
        }
        i += 1;
    }
    (enqueued, parked)
}

/// borg-reborn-rs (#597): when no staircase entered the flow and a monster
/// is parked on the known ones, flow to those anyway: the walk there fights
/// the monster in the way, or finds the mark stale.
fn enqueue_parked_if_alone(st: &mut BorgState, enqueued: usize, parked: Vec<(i32, i32)>) {
    if crate::borg_caution::stair_parked_fallback(enqueued, parked.len()) {
        for (y, x) in parked {
            borg_flow::borg_flow_enqueue_grid(st, y, x);
        }
    }
}

/// Commit the flow and take one step (the common end of the three flows).
fn commit_and_step(st: &mut BorgState, who: &std::ffi::CStr, why: i32) -> bool {
    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, who, why) {
        return false;
    }
    // borg-reborn-rs (#614): a flight to a staircase (rests wait for it).
    st.borg.flight_to_stairs = true;

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
    borg_flow::borg_flow_spread_stairs(st, 250, false, false, false, b_stair, false);

    /* Distance from the grid to the stair */
    let cost = i32::from(st.flow.cost_at(y, x));

    /* if it is impossible to get to the stairs, go off leash */
    if cost == 255 {
        return 0;
    }

    cost
}

/// Prepare to flee the level via stairs.
fn stair_both(st: &mut BorgState, why: i32, sneak: bool) -> bool {
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
    // borg-reborn-rs (#597): the parked fallback is decided across both
    // kinds, so a parked up staircase is no goal while a down one is free.
    let (mut enqueued, mut parked) = enqueue_free_stairs(st, Which::Less);
    // borg-reborn-rs (#557): no down staircase for a flight while rising.
    if !(why == GOAL_FLEE && crate::borg_caution::rising_holds_flee_descent(st)) {
        let (more, more_parked) = enqueue_free_stairs(st, Which::More);
        enqueued += more;
        parked.extend(more_parked);
    }
    enqueue_parked_if_alone(st, enqueued, parked);

    /* Spread the flow */
    borg_flow::borg_flow_spread_stairs(st, 250, false, false, false, -1, sneak);

    commit_and_step(st, c"stairs", why)
}

/// borg-reborn-rs (#736): spread the desperate stair flow (to the up stairs,
/// the down stairs, or both with `up` `None`), as `desperate_flow` would,
/// run `f` on it, and put the flow state back as it was: an observation
/// only, so the rules after it read what they would have read.
fn with_stair_spread<R>(st: &mut BorgState, up: Option<bool>, f: impl FnOnce(&mut BorgState) -> R) -> R {
    let fl = &st.flow;
    let saved = (fl.head, fl.tail, fl.flow_x.clone(), fl.flow_y.clone(), fl.data_cost.clone(),
                 fl.data_know.clone(), fl.data_icky.clone(), fl.hunting, fl.to_stairs);
    // As `desperate_flow` spreads it: desperate (dangerous grids are
    // crossed), no avoidance.
    let was = std::mem::replace(&mut st.borg.status.desperate, true);
    borg_flow::borg_flow_clear(st);
    if up != Some(false) {
        enqueue_stairs(st, Which::Less);
    }
    if up != Some(true) {
        enqueue_stairs(st, Which::More);
    }
    let optimize = st.borg.tr(BI_CLEVEL) > 35 || st.borg.tr(BI_LIGHT) == 0;
    borg_flow::borg_flow_spread_stairs(st, 250, optimize, false, false, -1, false);
    let r = f(st);
    st.borg.status.desperate = was;
    let fl = &mut st.flow;
    (fl.head, fl.tail, fl.flow_x, fl.flow_y, fl.data_cost, fl.data_know, fl.data_icky, fl.hunting, fl.to_stairs) = saved;
    r
}

/// borg-reborn-rs (#736, owner: "the acolyte is going to be in the way. The
/// path to the stairs is not clean, so the 'desperate for stairs' should see
/// that the stairs are not reachable"): walk the desperate flow's path from
/// the Borg to the up (`up`) or down stairs. The first awake monster on a
/// grid of the path, or next to one, whose danger to the Borg is at least
/// `percent` of its HP, is returned (its index in `st.kills`). `None`: a
/// clean path, or no stairs to flow to.
pub fn stair_path_blocker(st: &mut BorgState, up: bool, percent: i32) -> Option<usize> {
    let track = if up { &st.stairs.less } else { &st.stairs.more };
    if track.num == 0 {
        return None;
    }
    with_stair_spread(st, Some(up), |st| path_blocker(st, percent))
}

/// borg-reborn-rs (#736, owner: "it stepped away in the wrong direction,
/// instead of W he should have stepped SE towards the exit"): the steps from
/// each of the eight grids around the Borg (in `ddd` order) to the nearest
/// staircase, up or down, by the desperate flow. 255: none known, or not
/// reachable from that grid.
pub fn stair_steps_around(st: &mut BorgState) -> [i32; 8] {
    if st.stairs.less.num == 0 && st.stairs.more.num == 0 {
        return [255; 8];
    }
    with_stair_spread(st, None, |st| {
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        std::array::from_fn(|i| {
            let x = cx + i32::from(engine::ddx_ddd(i));
            let y = cy + i32::from(engine::ddy_ddd(i));
            if engine::square_in_bounds(x, y) { i32::from(st.flow.cost_at(y, x)) } else { 255 }
        })
    })
}

/// The walk of `stair_path_blocker` over the spread flow costs.
fn path_blocker(st: &mut BorgState, percent: i32) -> Option<usize> {
    let hp = st.borg.tr(BI_CURHP).max(1);
    let (mut y, mut x) = (st.borg.c.y, st.borg.c.x);
    let mut seen = [false; 1024];
    for _ in 0..250 {
        let here = i32::from(st.flow.cost_at(y, x));
        if here == 0 || here >= 250 {
            break;
        }
        // The next grid of the path: the neighbour with the lowest cost.
        let mut next = None;
        for i in 0..8 {
            let nx = x + i32::from(engine::ddx_ddd(i));
            let ny = y + i32::from(engine::ddy_ddd(i));
            if !engine::square_in_bounds(nx, ny) {
                continue;
            }
            let c = i32::from(st.flow.cost_at(ny, nx));
            if c < here && next.is_none_or(|(_, _, b)| c < b) {
                next = Some((ny, nx, c));
            }
        }
        let Some((ny, nx, _)) = next else { break };
        (y, x) = (ny, nx);
        // A monster on this grid of the path, or next to it.
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (my, mx) = (y + dy, x + dx);
                if !engine::square_in_bounds(mx, my) {
                    continue;
                }
                let k = usize::from(st.cave.grid(mx, my).kill);
                if k == 0 || k >= seen.len() || seen[k] {
                    continue;
                }
                seen[k] = true;
                if !st.kills.kills[k].awake {
                    continue;
                }
                let (cy, cx) = (st.borg.c.y, st.borg.c.x);
                let danger = crate::borg_danger::borg_danger_one_kill(st, cy, cx, 1, k as i32, true, false);
                if danger * 100 >= hp * percent {
                    return Some(k);
                }
            }
        }
    }
    None
}

/// Prepare to flow towards "up" stairs.
fn stair_less(st: &mut BorgState, why: i32, sneak: bool) -> bool {
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
        borg_flow::borg_flow_spread_stairs(st, 250, true, false, false, -1, sneak);
    } else {
        /* Spread the flow, No Optimize, Avoid */
        let avoid = !st.borg.status.desperate;
        borg_flow::borg_flow_spread_stairs(st, 250, false, avoid, false, -1, sneak);
    }

    commit_and_step(st, c"up-stairs", why)
}

/// borg-reborn-rs (#548, review of #566): the refusals of
/// `borg_flow_stair_more` before its flow (with an up staircase known: not
/// prepared for the next depth unless `brave`, hungry, a sale held, no
/// light; and recalling), shared with #544's flight down a known staircase
/// so the two agree. True: no descent.
pub(crate) fn more_refused(st: &mut BorgState, brave: bool) -> bool {
    /* if there are no down stairs, don't filter use of up stairs */
    if st.stairs.less.num != 0 {
        /* not unless safe or brave */
        if !brave {
            let depth = st.borg.tr(BI_CDEPTH) + 1;
            if crate::borg_prepared::borg_prepared_c(st, depth).is_some() {
                return true;
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
            return true;
        }

        /* If I need to sell crap, then don't go down */
        if crate::borg_trip::legacy_sale_hold(st) {
            return true;
        }

        /* No diving if no light */
        if st.borg.tr(BI_LIGHT) == 0 {
            return true;
        }
    }

    /* don't head for the stairs if you are recalling,  */
    /* even if you are fleeing. */
    // borg-reborn-rs (#603): unless caution_recall_wait_as_usual.
    if crate::borg_caution::recall_restricts(st) {
        return true;
    }

    false
}

/// borg-reborn-rs (#548): enqueue the down staircases without a monster on
/// them (`enqueue_stairs`), for #544's capped flight.
pub(crate) fn enqueue_down_stairs(st: &mut BorgState) {
    enqueue_stairs(st, Which::More);
}

/// Prepare to flow towards "down" stairs.
fn stair_more(st: &mut BorgState, why: i32, sneak: bool, brave: bool) -> bool {
    /* None to flow to */
    if st.stairs.more.num == 0 {
        return false;
    }

    // borg-reborn-rs (#557): no flight down a staircase while rising.
    if why == GOAL_FLEE && crate::borg_caution::rising_holds_flee_descent(st) {
        return false;
    }

    if more_refused(st, brave) {
        return false;
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Enqueue useful grids */
    enqueue_stairs(st, Which::More);

    /* Spread the flow */
    borg_flow::borg_flow_spread_stairs(st, 250, true, false, false, -1, sneak);

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

/// Prepare to flee the level via stairs (`stair_both`, with #556's
/// fight-through fallback, `fight_through`).
pub fn borg_flow_stair_both(st: &mut BorgState, why: i32, sneak: bool) -> bool {
    // borg-reborn-rs (#743): a flight takes only the kinds of stairs whose
    // path is clean.
    match flee_clean_kinds(st, why) {
        CleanKinds::All => fight_through(st, why, |st| stair_both(st, why, sneak)),
        CleanKinds::Up => fight_through(st, why, |st| stair_less(st, why, sneak)),
        CleanKinds::Down => fight_through(st, why, |st| stair_more(st, why, sneak, false)),
        CleanKinds::None => false,
    }
}

/// Prepare to flow towards "up" stairs (`stair_less`, with #556's
/// fight-through fallback).
pub fn borg_flow_stair_less(st: &mut BorgState, why: i32, sneak: bool) -> bool {
    if flee_path_blocked(st, why, true) {
        return false;
    }
    fight_through(st, why, |st| stair_less(st, why, sneak))
}

/// Prepare to flow towards "down" stairs (`stair_more`, with #556's
/// fight-through fallback).
pub fn borg_flow_stair_more(st: &mut BorgState, why: i32, sneak: bool, brave: bool) -> bool {
    if flee_path_blocked(st, why, false) {
        return false;
    }
    fight_through(st, why, |st| stair_more(st, why, sneak, brave))
}

/// borg-reborn-rs (#743, the follow-up of #736): which kinds of stairs a
/// flight (`GOAL_FLEE`) may take, with `caution_stairs_clean_path_flee`: a
/// kind whose path has a dangerous awake monster on, or next to, it
/// (`stair_path_blocker`) is not taken; with none left the flight does not
/// start, and the escapes and the attack decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CleanKinds {
    All,
    Up,
    Down,
    None,
}

/// The kinds a flight to any stairs may take (`CleanKinds`), from which
/// kinds are known and which are blocked.
fn clean_kinds(up_known: bool, up_blocked: bool, down_known: bool, down_blocked: bool) -> CleanKinds {
    let up = up_known && !up_blocked;
    let down = down_known && !down_blocked;
    match (up, down) {
        (true, true) => CleanKinds::All,
        (true, false) => CleanKinds::Up,
        (false, true) => CleanKinds::Down,
        // No kind known: the flow finds nothing either way, as C.
        (false, false) if !up_known && !down_known => CleanKinds::All,
        (false, false) => CleanKinds::None,
    }
}

/// Whether the check applies: the switch on, a flight, and not the
/// desperate walk (`desperate_flow` checks its own path first, #736).
fn flee_check_applies(st: &BorgState, why: i32) -> bool {
    why == GOAL_FLEE
        && !st.borg.status.desperate
        && crate::params::value(crate::params::Param::caution_stairs_clean_path_flee) != 0
}

/// The blocker of the path to the up (`up`) or down stairs, noted once per
/// level as #736's "Stairs not reachable".
fn flee_blocker(st: &mut BorgState, up: bool) -> bool {
    let percent = crate::params::value(crate::params::Param::caution_stairs_block_danger_percent);
    let Some(k) = stair_path_blocker(st, up, percent) else {
        return false;
    };
    if !st.borg.flee_path_noted {
        st.borg.flee_path_noted = true;
        let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
        borg_io::borg_note(
            st,
            &format!("# Stairs not reachable: {} in the way (flight to the {} stairs)", String::from_utf8_lossy(&name), if up { "up" } else { "down" }),
        );
    }
    true
}

/// `borg_flow_stair_less/more`: a flight to a blocked kind does not start.
fn flee_path_blocked(st: &mut BorgState, why: i32, up: bool) -> bool {
    flee_check_applies(st, why) && flee_blocker(st, up)
}

/// `borg_flow_stair_both`: the kinds a flight may take.
fn flee_clean_kinds(st: &mut BorgState, why: i32) -> CleanKinds {
    if !flee_check_applies(st, why) {
        return CleanKinds::All;
    }
    let up_known = st.stairs.less.num > 0;
    let down_known = st.stairs.more.num > 0;
    let up_blocked = up_known && flee_blocker(st, true);
    let down_blocked = down_known && flee_blocker(st, false);
    clean_kinds(up_known, up_blocked, down_known, down_blocked)
}

/// borg-reborn-rs (#556, owner 2026-11-19: "fallback to fight our way
/// out"): a flight (`GOAL_FLEE`) whose stair flow found no route around the
/// remembered breeders (`caution_breeder_avoid`) takes the plain route
/// through them, with `caution_breeder_avoid_fight_through`; the fight-through
/// is noted in `breeder_through` so back-away and retreat do not undo it
/// (`crate::borg_caution::breeder_fighting_through`).
fn fight_through(st: &mut BorgState, why: i32, mut flow: impl FnMut(&mut BorgState) -> bool) -> bool {
    if flow(st) {
        return true;
    }
    if why != GOAL_FLEE
        || st.borg.breeder_block.is_empty()
        || !crate::borg_caution::breeder_fight_through_on()
    {
        return false;
    }
    let was = std::mem::replace(&mut st.flow.hunting, true);
    let ok = flow(st);
    st.flow.hunting = was;
    if ok {
        st.borg.breeder_through = Some(st.borg.time.now);
        borg_io::borg_note(st, "# Fighting through the breeders to the stairs");
    }
    ok
}

#[cfg(test)]
mod flee_clean_tests {
    use super::{clean_kinds, CleanKinds};

    /// #743: a flight takes only the clean kinds; with every known kind
    /// blocked it does not start; with none known, as C.
    #[test]
    fn a_flight_takes_only_clean_stairs() {
        assert_eq!(clean_kinds(true, false, true, false), CleanKinds::All);
        assert_eq!(clean_kinds(true, true, true, false), CleanKinds::Down, "up blocked");
        assert_eq!(clean_kinds(true, false, true, true), CleanKinds::Up, "down blocked");
        assert_eq!(clean_kinds(true, true, true, true), CleanKinds::None, "both blocked");
        assert_eq!(clean_kinds(true, true, false, false), CleanKinds::None, "the only kind blocked");
        assert_eq!(clean_kinds(false, false, true, false), CleanKinds::Down, "only down known");
        assert_eq!(clean_kinds(false, false, false, false), CleanKinds::All, "none known");
    }
}

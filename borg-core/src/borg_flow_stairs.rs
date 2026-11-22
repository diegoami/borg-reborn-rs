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
use crate::headers::z_type_h::loc;
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
    /// Down stair left while exploring for or following a route to an up stair.
    pub(crate) up_search_departure: Option<UpSearchDeparture>,
}

/// Let the up-stair route take a second step before a fallback can undo
/// its first one. A short window also covers a door or another one-turn action.
const UP_SEARCH_HANDOFF_TURNS: i32 = 3;

#[derive(Clone, Copy)]
pub(crate) struct UpSearchDeparture {
    pub stair: loc,
    pub next: loc,
    pub level_started: i32,
    pub turn: i32,
    pub expected_danger: i32,
    pub known: bool,
}

impl UpSearchDeparture {
    fn still_at_handoff(self, here: loc, level_started: i32, turn: i32) -> bool {
        here == self.next && level_started == self.level_started
            && (1..=UP_SEARCH_HANDOFF_TURNS).contains(&turn.wrapping_sub(self.turn))
    }

    fn permits_deferral(self, danger: i32, hp: i32) -> bool {
        danger <= self.expected_danger || danger.saturating_mul(2) < hp
    }

    pub(crate) fn can_renew(self, st: &BorgState, known: bool) -> bool {
        self.known == known && self.still_at_handoff(st.borg.c, st.borg.time.level, st.borg.time.now)
    }
}

/// A viable up-stair search outranks a down-stair fallback. Including an
/// alternate down stair can send the search one step away before the next
/// think chooses the original down stair again.
pub(crate) fn continuing_up_stair_search(st: &mut BorgState) -> bool {
    let Some(departure) = st.stairs.up_search_departure else { return false; };
    if departure.known { return false; }
    if !departure.can_renew(st, false)
        || !st.borg.stair_less || !(st.borg.goal.leaving || st.borg.goal.fleeing)
        || !borg_flow::up_stair_search_can_continue(st, departure.stair)
    {
        st.stairs.up_search_departure = None;
        return false;
    }
    let loc { x, y } = st.borg.c;
    // The danger calculation uses the Borg's mutable tactical scratch state.
    let danger = crate::borg_danger::borg_danger(st, y, x, 1, true, false);
    if !departure.permits_deferral(danger, st.borg.tr(BI_CURHP)) {
        st.stairs.up_search_departure = None;
        return false;
    }
    true
}

/// Follow a known up stair before trying a nearby down stair again. Replan
/// from the current square so a blocked route does not reserve the flight.
pub(crate) fn resume_known_up_stair_flight(st: &mut BorgState, danger: i32) -> bool {
    let Some(departure) = st.stairs.up_search_departure else { return false; };
    if !departure.known { return false; }
    if departure.can_renew(st, true)
        && st.borg.stair_less && (st.borg.goal.leaving || st.borg.goal.fleeing)
        && departure.permits_deferral(danger, st.borg.tr(BI_CURHP))
        && borg_flow_stair_less(st, GOAL_FLEE, false)
    {
        return true;
    }
    st.stairs.up_search_departure = None;
    false
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
    if matches!(which, Which::More) && continuing_up_stair_search(st) {
        // Even a monster-occupied down stair is unavailable during the search.
        return (0, Vec::new());
    }
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

/// #743: check the path in the spread being used to decide this flight.
fn flight_path_blocker(st: &mut BorgState, why: i32) -> Option<usize> {
    if !flee_check_applies(st, why) {
        return None;
    }
    let percent = crate::params::value(crate::params::Param::caution_stairs_block_danger_percent);
    path_blocker(st, percent)
}

/// #932: the guarded unweighted probe must not call a detour reachable
/// when the flight would reject it. An unguarded probe keeps its old rule.
pub(crate) fn unweighted_flight_reachable(st: &mut BorgState) -> bool {
    let cost = st.flow.cost_at(st.borg.c.y, st.borg.c.x);
    reachable_after_spread(cost, st.flow.has_retreat_exclusions(), || {
        flight_path_blocker(st, GOAL_FLEE)
    })
}

fn reachable_after_spread(cost: u8, guarded: bool, blocker: impl FnOnce() -> Option<usize>) -> bool {
    cost < 250 && (!guarded || blocker().is_none())
}

/// Commit the flow and take one step (the common end of the three flows).
fn commit_and_step(st: &mut BorgState, who: &std::ffi::CStr, why: i32) -> bool {
    if let Some(k) = flight_path_blocker(st, why) {
        st.borg.flee_path_refused = true;
        if !st.borg.flee_path_noted {
            st.borg.flee_path_noted = true;
            let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
            let to = who.to_string_lossy();
            borg_io::borg_note(st, &format!("# Stairs not reachable: {} in the way (flight to the {to})", String::from_utf8_lossy(&name)));
        }
        return false;
    }
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

/// #765: choose the cheapest stair route. If the clean-path check refuses
/// it, search the same stair set with the check applied to every entered
/// grid so a costlier clean detour can still be used.
fn weighted_route(
    st: &mut BorgState,
    kinds: &[Which],
    depth: i32,
    avoid: bool,
    sneak: bool,
    weight: i32,
) -> Result<Option<borg_flow::StairPath>, usize> {
    let search = kinds.iter().any(|which| matches!(which, Which::More))
        && continuing_up_stair_search(st);
    let mut stairs = Vec::new();
    for &which in kinds {
        if search && matches!(which, Which::More) {
            continue;
        }
        let track = track(st, which);
        for i in 0..track.num.max(0) as usize {
            let (y, x) = track.at(i);
            if st.cave.grid(x, y).kill == 0 {
                stairs.push(loc { x, y });
            }
        }
    }
    let Some(path) = borg_flow::weighted_stair_path(st, &stairs, depth, avoid, sneak, weight, |_, _| true) else {
        return Ok(None);
    };
    let Some(percent) = crate::borg_caution::weighted_stairs_blocker_percent(st) else {
        return Ok(Some(path));
    };
    let mode = crate::borg_caution::stairs_clean_path_mode();
    let mut check = WeightedPathCheck {
        mode,
        reach: blocker_reach(mode),
        hp: st.borg.tr(BI_CURHP).max(1),
        percent,
        blocks: [None; 256],
    };
    let target = *path.grids.last().expect("a stair path includes its target");
    let Some(blocker) = path.grids.iter().skip(1).find_map(|&grid| check.blocker_at(st, grid, target)) else {
        return Ok(Some(path));
    };

    // Mode 3's test of whether a nearby monster is ahead depends on the
    // destination. Check each staircase under its own rule.
    let search = |check: &mut WeightedPathCheck, st: &mut BorgState, goals: &[loc], target| {
        borg_flow::weighted_stair_path(st, goals, depth, avoid, sneak, weight, |st, grid| {
            check.blocker_at(st, grid, target).is_none()
        })
    };
    let clean = if mode == 3 {
        stairs.iter().filter_map(|stair| search(&mut check, st, std::slice::from_ref(stair), *stair))
            .min_by_key(|route| (route.cost, route.grids.len()))
    } else {
        search(&mut check, st, &stairs, target)
    };
    clean.map(Some).ok_or(blocker)
}

struct WeightedPathCheck {
    mode: i32,
    reach: i32,
    hp: i32,
    percent: i32,
    blocks: [Option<bool>; 256],
}

impl WeightedPathCheck {
    fn blocker_at(&mut self, st: &mut BorgState, grid: loc, target: loc) -> Option<usize> {
        let stairs = (target.y, target.x);
        let borg_far = grids_apart((st.borg.c.y, st.borg.c.x), stairs);
        for dy in -self.reach..=self.reach {
            for dx in -self.reach..=self.reach {
                let (y, x) = (grid.y + dy, grid.x + dx);
                if !engine::square_in_bounds(x, y) {
                    continue;
                }
                let k = usize::from(st.cave.grid(x, y).kill);
                if k == 0 || !st.kills.kills[k].awake {
                    continue;
                }
                if (dx != 0 || dy != 0) && !beside_blocks(self.mode, grids_apart((y, x), stairs), borg_far) {
                    continue;
                }
                if *self.blocks[k].get_or_insert_with(|| dangerous_path_blocker(st, k, self.hp, self.percent)) {
                    return Some(k);
                }
            }
        }
        None
    }
}

struct WeightedFlightOptions {
    depth: i32,
    avoid: bool,
    sneak: bool,
    weight: i32,
    ranged_reach: i32,
}

fn weighted_flight(
    st: &mut BorgState,
    why: i32,
    direction: Option<Which>,
    options: WeightedFlightOptions,
) -> bool {
    let (who, kinds): (_, &[_]) = match direction {
        Some(Which::Less) => (c"up-stairs", &[Which::Less]),
        Some(Which::More) => (c"down-stairs", &[Which::More]),
        None => (c"stairs", &[Which::Less, Which::More]),
    };
    match weighted_route(st, kinds, options.depth, options.avoid, options.sneak, options.weight) {
        Ok(Some(path)) => {
            if options.ranged_reach > 0 {
                let path_grids: Vec<_> = path.grids.iter().skip(1).map(|grid| (grid.y, grid.x)).collect();
                let percent = crate::params::value(crate::params::Param::caution_stairs_block_danger_percent);
                if let Some(k) = stairs_monster_on_path(st, &path_grids, percent, 3, options.ranged_reach) {
                    let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
                    let to = who.to_string_lossy();
                    borg_io::borg_note(st, &format!("# Stairs not reachable: {} ahead, fighting at range (flight to the {to})", String::from_utf8_lossy(&name)));
                    return false;
                }
            }
            borg_flow::commit_weighted_stair_path(st, who, why, &path.grids)
        }
        Ok(None) => false,
        Err(k) => {
            st.borg.flee_path_refused = true;
            if st.borg.status.desperate || !st.borg.flee_path_noted {
                if !st.borg.status.desperate {
                    st.borg.flee_path_noted = true;
                }
                let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
                let to = who.to_string_lossy();
                borg_io::borg_note(st, &format!("# Stairs not reachable: {} in the way (flight to the {to})", String::from_utf8_lossy(&name)));
            }
            false
        }
    }
}

pub(crate) fn weighted_down_flight(st: &mut BorgState, depth: i32, weight: i32) -> bool {
    borg_flow::with_retreat_avoidance(st, GOAL_FLEE, |st| {
        weighted_flight(st, GOAL_FLEE, Some(Which::More), WeightedFlightOptions {
            depth, avoid: false, sneak: false, weight, ranged_reach: 0,
        })
    })
}

/// Keep the desperate walk within its move limit, including clean detours.
/// Ranged-first still checks the route that would actually be committed.
pub(crate) fn weighted_desperate_flight(st: &mut BorgState, up: bool, max_moves: i32, ranged_reach: i32) -> bool {
    fight_through(st, GOAL_FLEE, |st| {
        if up {
            stair_less(st, GOAL_FLEE, false, max_moves, ranged_reach)
        } else {
            stair_more(st, GOAL_FLEE, false, false, max_moves, ranged_reach)
        }
    })
}

pub(crate) fn weighted_stair_reachable(st: &mut BorgState, up: bool, avoid: bool, weight: i32) -> bool {
    let which = if up { Which::Less } else { Which::More };
    matches!(weighted_route(st, &[which], 250, avoid, false, weight), Ok(Some(_)))
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
    stair_kinds(st, why, sneak, None)
}

/// `stair_both` for the staircases of `only` (`None`: both kinds). The
/// review of #757: #743's fallback tries each kind on its own under the
/// combined flight's rules, not `stair_less`/`stair_more`'s (whose
/// `more_refused` the combined flow never applied).
fn stair_kinds(st: &mut BorgState, why: i32, sneak: bool, only: Option<Which>) -> bool {
    let up = !matches!(only, Some(Which::More));
    let down = !matches!(only, Some(Which::Less));
    /* None to flow to */
    if (!up || st.stairs.less.num == 0) && (!down || st.stairs.more.num == 0) {
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

    // borg-reborn-rs (#557): no down staircase for a flight while rising.
    let down = down && !(why == GOAL_FLEE && crate::borg_caution::rising_holds_flee_descent(st));
    let weight = crate::borg_caution::stair_flight_weight(why, false);
    if weight > 0 {
        let direction = match (up, down) {
            (true, true) => None,
            (true, false) => Some(Which::Less),
            (false, true) => Some(Which::More),
            (false, false) => return false,
        };
        return weighted_flight(st, why, direction, WeightedFlightOptions {
            depth: 250, avoid: false, sneak, weight, ranged_reach: 0,
        });
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Enqueue useful grids */
    // borg-reborn-rs (#597): the parked fallback is decided across both
    // kinds, so a parked up staircase is no goal while a down one is free.
    let (mut enqueued, mut parked) = if up { enqueue_free_stairs(st, Which::Less) } else { (0, Vec::new()) };
    if down {
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

/// borg-reborn-rs (#751, owner on 247597266: the Mage walked at a cutpurse
/// and an apprentice between it and the stairs, with 22 Flasks of oil): the
/// first awake monster within `reach` grids of the desperate walk's path to
/// the up (`up`) or down stairs and nearer them than the Borg, whose danger
/// is at least `percent` of the HP. `None`: none, or no stairs.
pub fn stair_path_ahead(st: &mut BorgState, up: bool, percent: i32, reach: i32) -> Option<usize> {
    let track = if up { &st.stairs.less } else { &st.stairs.more };
    if track.num == 0 {
        return None;
    }
    with_stair_spread(st, Some(up), |st| path_blocker_with(st, percent, 3, reach))
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
    // borg-reborn-rs (#751, owner: "stop flight when other monsters are IN
    // BETWEEN the stairs, not close"): with caution_stairs_clean_path at 2,
    // only a monster on the path blocks it, for the desperate walk and for
    // every other caller of this walk (#743's flights). At 3 (owner,
    // 93034700: "he has to actually walk past his opponent"), a monster
    // next to the path blocks it only when it is ahead: nearer the
    // staircase at the path's end than the Borg.
    let mode = crate::borg_caution::stairs_clean_path_mode();
    path_blocker_with(st, percent, mode, blocker_reach(mode))
}

/// `path_blocker` for a given mode and reach (grids from the path).
fn path_blocker_with(st: &mut BorgState, percent: i32, mode: i32, reach: i32) -> Option<usize> {
    let borg = (st.borg.c.y, st.borg.c.x);
    let path = walk_path(
        borg,
        |y, x| engine::square_in_bounds(x, y).then(|| i32::from(st.flow.cost_at(y, x))),
        |i| (i32::from(engine::ddy_ddd(i)), i32::from(engine::ddx_ddd(i))),
    );
    stairs_monster_on_path(st, &path, percent, mode, reach)
}

fn stairs_monster_on_path(st: &mut BorgState, path: &[(i32, i32)], percent: i32, mode: i32, reach: i32) -> Option<usize> {
    let hp = st.borg.tr(BI_CURHP).max(1);
    first_blocker(
        st,
        path,
        (st.borg.c.y, st.borg.c.x),
        mode,
        reach,
        |st, y, x| if engine::square_in_bounds(x, y) { usize::from(st.cave.grid(x, y).kill) } else { 0 },
        |st, k| st.kills.kills[k].awake && dangerous_path_blocker(st, k, hp, percent),
    )
}

/// The flow's path from `start` (y, x): each step the neighbour with the
/// lowest cost below the current grid's, in `step(i)`'s order (the engine's
/// `ddy_ddd`/`ddx_ddd`), until a grid of cost 0 (the staircase) or none
/// lower. `cost` is `None` out of bounds. The review of #757 asked for
/// tests on constructed paths, hence no engine here.
fn walk_path(start: (i32, i32), cost: impl Fn(i32, i32) -> Option<i32>, step: impl Fn(usize) -> (i32, i32)) -> Vec<(i32, i32)> {
    let (mut y, mut x) = start;
    let mut path = Vec::new();
    for _ in 0..250 {
        let here = cost(y, x).unwrap_or(255);
        if here == 0 || here >= 250 {
            break;
        }
        let mut next = None;
        for i in 0..8 {
            let (dy, dx) = step(i);
            let (ny, nx) = (y + dy, x + dx);
            let Some(c) = cost(ny, nx) else { continue };
            if c < here && next.is_none_or(|(_, _, b)| c < b) {
                next = Some((ny, nx, c));
            }
        }
        let Some((ny, nx, _)) = next else { break };
        (y, x) = (ny, nx);
        path.push((y, x));
    }
    path
}

/// The first monster in the way of `path` (#736, #751): on a grid of the
/// path, or within `reach` grids of one (`blocker_reach`: 1 in modes 1 and
/// 3, 0 in mode 2), where mode 3 keeps only a monster
/// nearer the staircase (the path's last grid) than the Borg at `borg`.
/// `kill_at` gives the monster index on a grid (0: none, or out of bounds);
/// `blocks` says whether that monster stops the walk (awake and dangerous).
/// Each monster is judged once, at the first grid of the path it touches.
fn first_blocker<C>(
    ctx: &mut C,
    path: &[(i32, i32)],
    borg: (i32, i32),
    mode: i32,
    reach: i32,
    kill_at: impl Fn(&C, i32, i32) -> usize,
    mut blocks: impl FnMut(&mut C, usize) -> bool,
) -> Option<usize> {
    let stairs = path.last().copied().unwrap_or(borg);
    let borg_far = grids_apart(borg, stairs);
    let mut seen = [false; 1024];
    for &(y, x) in path {
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (my, mx) = (y + dy, x + dx);
                let k = kill_at(ctx, my, mx);
                if k == 0 || k >= seen.len() || seen[k] {
                    continue;
                }
                let beside = dy != 0 || dx != 0;
                // The review of #757: a monster beside the path but not
                // ahead is not marked seen, so the grid it stands on, if the
                // walk reaches it, still judges it as on the path.
                if beside && !beside_blocks(mode, grids_apart((my, mx), stairs), borg_far) {
                    continue;
                }
                seen[k] = true;
                if blocks(ctx, k) {
                    return Some(k);
                }
            }
        }
    }
    None
}

fn dangerous_path_blocker(st: &mut BorgState, k: usize, hp: i32, percent: i32) -> bool {
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    let danger = crate::borg_danger::borg_danger_one_kill(st, cy, cx, 1, k as i32, true, false);
    danger * 100 >= hp * percent
}

/// Grid steps between two grids (king moves, Chebyshev), for #751's mode 3;
/// not `engine::distance`, the game's `max + min / 2`.
fn grids_apart(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// borg-reborn-rs (#751): how far from a grid of the path a monster blocks
/// it: 0 = only on it (mode 2), 1 = on or next to it (#736's mode 1, and
/// mode 3, which then asks `beside_blocks`).
fn blocker_reach(mode: i32) -> i32 {
    if mode == 2 { 0 } else { 1 }
}

/// borg-reborn-rs (#751, mode 3): a monster next to the path, `monster_far`
/// steps from the staircase at its end, blocks it only when it is nearer
/// that staircase than the Borg (`borg_far`): one the walk has to pass.
/// (The flow cost cannot tell: the flow skips the grids of monsters and
/// dangerous grids.) In the other modes every monster next to the path
/// counts (mode 2 sees none).
fn beside_blocks(mode: i32, monster_far: i32, borg_far: i32) -> bool {
    mode != 3 || monster_far < borg_far
}

/// Prepare to flow towards "up" stairs.
fn stair_less(st: &mut BorgState, why: i32, sneak: bool, depth: i32, ranged_reach: i32) -> bool {
    /* forced to go up */
    if engine::player_opt(OPT_birth_force_descend) {
        return false;
    }

    /* None to flow to */
    if st.stairs.less.num == 0 {
        return false;
    }

    let weight = crate::borg_caution::stair_flight_weight(why, st.borg.goal.fleeing || st.borg.goal.leaving);
    if weight > 0 {
        let avoid = st.borg.tr(BI_CLEVEL) <= 35 && st.borg.tr(BI_LIGHT) != 0 && !st.borg.status.desperate;
        return weighted_flight(st, why, Some(Which::Less), WeightedFlightOptions {
            depth, avoid, sneak, weight, ranged_reach,
        });
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
            if crate::borg_prepared::prepared_c_for_decision(st, depth).is_some() {
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
fn stair_more(st: &mut BorgState, why: i32, sneak: bool, brave: bool, depth: i32, ranged_reach: i32) -> bool {
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

    let weight = crate::borg_caution::stair_flight_weight(why, false);
    if weight > 0 {
        return weighted_flight(st, why, Some(Which::More), WeightedFlightOptions {
            depth, avoid: false, sneak, weight, ranged_reach,
        });
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
mod blocker_reach_tests {
    use super::{beside_blocks, blocker_reach, first_blocker, walk_path};

    /// #751: mode 2 looks only at the path's grids, modes 1 and 3 also next
    /// to them.
    #[test]
    fn on_the_path_or_next_to_it() {
        assert_eq!(blocker_reach(2), 0);
        assert_eq!(blocker_reach(1), 1);
        assert_eq!(blocker_reach(3), 1);
    }

    /// #751 mode 3 (93034700): the apprentice beside the path 7 steps from
    /// the stairs, the Borg 8, blocks; one behind the Borg (9) or level with
    /// it does not. Mode 1 counts both.
    /// The review of #757 (#751's "Done when"): constructed paths. A
    /// corridor along row 5 from the Borg at (5,10) to the staircase at
    /// (5,2), its cost x - 2; the rows beside it are floor the flow did not
    /// reach (255: the flow skips monster grids). The cases the issue names:
    /// on the path, on the staircase, beside the path ahead, next to the Borg
    /// level with it, and a monster asleep.
    const DIRS: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

    fn corridor() -> Vec<(i32, i32)> {
        let cost = |y: i32, x: i32| {
            if !(0..20).contains(&y) || !(0..20).contains(&x) {
                None
            } else if y == 5 && (2..=10).contains(&x) {
                Some(x - 2)
            } else {
                Some(255)
            }
        };
        walk_path((5, 10), cost, |i| DIRS[i])
    }

    /// `first_blocker` over the corridor with one monster (index 1) at
    /// `at`, awake or not, every awake monster dangerous.
    fn blocked(mode: i32, at: (i32, i32), awake: bool) -> bool {
        let mut ctx = at;
        first_blocker(&mut ctx, &corridor(), (5, 10), mode, blocker_reach(mode), |m, y, x| usize::from(*m == (y, x)), |_, _| awake).is_some()
    }

    /// The review of #757, finding 1: a path that winds away from the
    /// staircase. The monster on its second grid (10,22) is first met beside
    /// the first grid, where it is not nearer the stairs than the Borg (8
    /// steps each); it still blocks when the walk reaches it, in every mode.
    #[test]
    fn a_monster_on_a_winding_walk_blocks() {
        let path = [(10, 20), (10, 21), (10, 22), (9, 23), (8, 24), (7, 25), (6, 26), (5, 26), (4, 26), (3, 26), (2, 26)];
        let cost = |y: i32, x: i32| {
            if !(0..30).contains(&y) || !(0..30).contains(&x) {
                return None;
            }
            Some(path.iter().position(|&g| g == (y, x)).map_or(255, |i| (path.len() - 1 - i) as i32))
        };
        let walk = walk_path((10, 20), cost, |i| DIRS[i]);
        assert_eq!(walk, path[1..].to_vec());
        for mode in [1, 2, 3] {
            let mut ctx = (10, 22);
            let hit = first_blocker(&mut ctx, &walk, (10, 20), mode, blocker_reach(mode), |m, y, x| usize::from(*m == (y, x)), |_, _| true);
            assert!(hit.is_some(), "on the walk, mode {mode}");
        }
    }

    /// The ranged-first test (`stair_path_ahead`): mode 3 with a wider
    /// reach. 247597266's shape: two grids beside the walk, ahead.
    #[test]
    fn a_monster_ahead_within_reach() {
        let at = |at: (i32, i32), reach: i32| {
            let mut ctx = at;
            first_blocker(&mut ctx, &corridor(), (5, 10), 3, reach, |m, y, x| usize::from(*m == (y, x)), |_, _| true).is_some()
        };
        assert!(!at((3, 7), 1), "two grids off the path: not next to it");
        assert!(at((3, 7), 2), "within two grids, nearer the stairs");
        assert!(!at((3, 11), 3), "behind the Borg");
    }

    #[test]
    fn the_corridor_path_ends_on_the_staircase() {
        assert_eq!(corridor(), (2..=9).rev().map(|x| (5, x)).collect::<Vec<_>>());
    }

    #[test]
    fn a_monster_on_the_path_or_the_staircase_blocks_in_every_mode() {
        for mode in [1, 2, 3] {
            assert!(blocked(mode, (5, 6), true), "on the path, mode {mode}");
            assert!(blocked(mode, (5, 2), true), "on the staircase, mode {mode}");
            assert!(!blocked(mode, (5, 6), false), "asleep, mode {mode}");
        }
    }

    #[test]
    fn a_monster_beside_the_path_ahead() {
        // 93034700's shape: beside the walk, nearer the stairs (6) than the
        // Borg (8).
        assert!(blocked(1, (4, 8), true));
        assert!(!blocked(2, (4, 8), true));
        assert!(blocked(3, (4, 8), true));
    }

    #[test]
    fn a_monster_next_to_the_borg_level_with_it() {
        // 2095979189's shape: next to the Borg and to the path's first step,
        // as far from the stairs as the Borg (8): only mode 1 counts it.
        assert!(blocked(1, (6, 10), true));
        assert!(!blocked(2, (6, 10), true));
        assert!(!blocked(3, (6, 10), true));
        // Behind the Borg, next to no grid of the path: nothing counts it.
        for mode in [1, 2, 3] {
            assert!(!blocked(mode, (5, 12), true), "behind, mode {mode}");
        }
    }

    #[test]
    fn mode_three_counts_only_a_monster_ahead() {
        assert!(beside_blocks(3, 7, 8));
        assert!(!beside_blocks(3, 8, 8));
        assert!(!beside_blocks(3, 9, 8));
        assert!(beside_blocks(1, 9, 8));
    }
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
    st.borg.flee_path_refused = false;
    if fight_through(st, why, |st| stair_both(st, why, sneak)) {
        return true;
    }
    // borg-reborn-rs (#743): the nearest stairs' path was not clean; the
    // other kind may be.
    if st.borg.flee_path_refused {
        return fight_through(st, why, |st| stair_kinds(st, why, sneak, Some(Which::Less)))
            || fight_through(st, why, |st| stair_kinds(st, why, sneak, Some(Which::More)));
    }
    false
}

/// Prepare to flow towards "up" stairs (`stair_less`, with #556's
/// fight-through fallback).
pub fn borg_flow_stair_less(st: &mut BorgState, why: i32, sneak: bool) -> bool {
    fight_through(st, why, |st| stair_less(st, why, sneak, 250, 0))
}

/// Prepare to flow towards "down" stairs (`stair_more`, with #556's
/// fight-through fallback).
pub fn borg_flow_stair_more(st: &mut BorgState, why: i32, sneak: bool, brave: bool) -> bool {
    fight_through(st, why, |st| stair_more(st, why, sneak, brave, 250, 0))
}

/// borg-reborn-rs (#556, owner 2026-11-19: "fallback to fight our way
/// out"): a flight (`GOAL_FLEE`) whose stair flow found no route around the
/// remembered breeders (`caution_breeder_avoid`) takes the plain route
/// through them, with `caution_breeder_avoid_fight_through`; the fight-through
/// is noted in `breeder_through` so back-away and retreat do not undo it
/// (`crate::borg_caution::breeder_fighting_through`).
/// borg-reborn-rs (#750, owner case 807683971: a black naga two grids
/// beside a clear corridor made the corridor's first grid "too dangerous",
/// so the flight never reached the up staircase 8 grids away). With
/// `caution_stairs_flight_ignore_danger`, a flight's stair flow (`GOAL_FLEE`,
/// or any `GOAL_BORE` stair flow, up or down, while fleeing or leaving) refuses no grid for its
/// danger; whether a monster stands in between is #751's path test.
fn fight_through(st: &mut BorgState, why: i32, flow: impl FnMut(&mut BorgState) -> bool) -> bool {
    let flight = crate::borg_caution::stairs_flight_ignores_danger(why, st.borg.goal.fleeing || st.borg.goal.leaving);
    let was = std::mem::replace(&mut st.flow.flight_no_danger, flight);
    let ok = borg_flow::with_retreat_avoidance(st, why, |st| fight_through_plain(st, why, flow));
    st.flow.flight_no_danger = was;
    ok
}

fn fight_through_plain(st: &mut BorgState, why: i32, mut flow: impl FnMut(&mut BorgState) -> bool) -> bool {
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

/// borg-reborn-rs (#743): whether a flight's path is checked: the switch on,
/// a flight (`GOAL_FLEE`), and not the desperate walk (`desperate_flow`
/// checks its own path first, #736).
fn flee_check_applies(st: &BorgState, why: i32) -> bool {
    flee_check_applies_with(crate::borg_caution::stairs_clean_path_flee_on(), why, st.borg.status.desperate)
}

/// `flee_check_applies` with the switch and the desperate flag given.
fn flee_check_applies_with(on: bool, why: i32, desperate: bool) -> bool {
    on && why == GOAL_FLEE && !desperate
}

#[cfg(test)]
mod flee_clean_tests {
    use super::{flee_check_applies_with, GOAL_FLEE};

    #[test]
    fn a_guarded_detour_with_a_blocker_does_not_suppress_fallbacks() {
        use super::{blocker_reach, first_blocker, reachable_after_spread};

        let origin = (4, 4);
        let shortcut = [(4, 5), (4, 6), (4, 7), (4, 8)];
        let detour = [(5, 4), (6, 5), (6, 6), (5, 7), (4, 8)];
        // The hold at (4,5) rules out the clean shortcut. An awake monster
        // on the longer path now blocks the only routed approach.
        let blocker = |path: &[(i32, i32)], awake| {
            first_blocker(&mut (6, 6), path, origin, 2, blocker_reach(2),
                |monster, y, x| usize::from(*monster == (y, x)), |_, _| awake)
        };
        assert!(reachable_after_spread(4, true, || blocker(&shortcut, true)));
        assert!(!reachable_after_spread(5, true, || blocker(&detour, true)));
        assert!(reachable_after_spread(5, true, || blocker(&detour, false)));
        assert!(reachable_after_spread(5, false, || panic!("unguarded probe")));
        assert!(!reachable_after_spread(255, true, || panic!("no path")));
    }

    /// #743: only a flight is checked, with the switch on, and not the
    /// desperate walk (which checks its own path, #736).
    #[test]
    fn only_a_flight_is_checked() {
        assert!(flee_check_applies_with(true, GOAL_FLEE, false));
        assert!(!flee_check_applies_with(false, GOAL_FLEE, false), "off");
        assert!(!flee_check_applies_with(true, GOAL_FLEE, true), "the desperate walk");
        assert!(!flee_check_applies_with(true, GOAL_FLEE - 1, false), "not a flight");
    }
}

#[cfg(test)]
mod up_search_handoff_tests {
    use super::*;

    #[test]
    fn an_unrenewed_step_expires_and_a_danger_increase_restores_fallback() {
        let stair = loc { x: 59, y: 42 };
        let next = loc { x: 58, y: 42 };
        let departure = UpSearchDeparture {
            stair, next, level_started: 10, turn: 100, expected_danger: 10, known: false,
        };
        assert!(departure.still_at_handoff(next, 10, 101));
        assert!(!departure.still_at_handoff(stair, 10, 101), "allow use of the stair from its square");
        assert!(!departure.still_at_handoff(loc { x: 57, y: 42 }, 10, 102));
        assert!(!departure.still_at_handoff(next, 11, 101));
        assert!(!departure.still_at_handoff(next, 10, 104));
        assert!(departure.permits_deferral(10, 40));
        assert!(departure.permits_deferral(12, 40));
        assert!(!departure.permits_deferral(21, 40));
    }
}

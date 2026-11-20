//! Port of `borg-flow.c`: the basic things used to flow (move) around.
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
//! The flow data (`borg_data_flow/cost/hard/know/icky`), the circular flow
//! queue (`borg_flow_x/y`, `flow_head`, `flow_tail`), the temporary grid
//! list (`borg_temp_*`) and the step/door tracks live in `FlowState`.
//! C bugs kept are listed in `docs/c-bugs.md` (section 31).

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ffi::CStr;

use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_danger::borg_danger;
use crate::borg_flow_stairs::StairTrack;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, borg_queue_direction};
use crate::borg_item_use::{borg_activate_item, borg_activate_ring, borg_equips_item, borg_equips_ring};
use crate::borg_magic::{borg_spell, borg_spell_okay, borg_spell_okay_fail};
use crate::borg_projection::borg_target;
use crate::borg_headers::borg_h::BORG_CHEST_FAIL_TOLERANCE;
use crate::borg_headers::borg_magic_h::{
    DISABLE_TRAPS_DESTROY_DOORS, ORB_OF_DRAINING, SHATTER_STONE, TURN_STONE_TO_MUD,
};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CRSFEAR, BI_CURHP, BI_CURSP, BI_DEV, BI_DIG, BI_DISM, BI_DISP,
    BI_FOOD, BI_IFIRE, BI_ISAFRAID, BI_ISBLIND, BI_ISCONFUSED, BI_ISHUNGRY, BI_ISWEAK, BI_LIGHT,
    BI_MAXCLEVEL, BI_MAXSP,
};
use crate::headers::list_mon_race_flags_h::RF_UNIQUE;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{
    FEAT_CLOSED, FEAT_GRANITE, FEAT_LAVA, FEAT_LESS, FEAT_NONE, FEAT_OPEN, FEAT_PASS_RUBBLE,
    FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K, FEAT_RUBBLE, FEAT_SECRET,
};
use crate::headers::list_tvals_h::{TV_DIGGING, TV_SHOT, TV_STAFF};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::{borg_flow_glyph, borg_flow_kill, borg_flow_misc, borg_flow_stairs, borg_flow_take, engine};

/// Size of the circular flow queue (`AUTO_FLOW_MAX`).
pub const AUTO_FLOW_MAX: i32 = 1536;

/// Size of the temporary grid list (`AUTO_TEMP_MAX`).
pub const AUTO_TEMP_MAX: usize = 9000;

/// Digging ability needed for rubble/magma, quartz and granite/doors.
const BORG_DIG: i32 = 10;
const BORG_DIG_MOD: i32 = 20;
const BORG_DIG_HARD: i32 = 40;

/// `GOAL_DARK` / `GOAL_DIGGING` (borg-flow.h).
const GOAL_DARK: i32 = 4;
const GOAL_DIGGING: i32 = 10;
/// `GOAL_FLEE` (borg-flow.h): flee the level (#551).
const GOAL_FLEE: i32 = 7;
/// Monster hunt (borg-flow.h), shared town certificate at every step (#788).
const GOAL_KILL: i32 = 1;

/// Size of the step/door tracks (`borg_init_flow()`).
const TRACK_SIZE: i16 = 100;

/// One `borg_data` grid map: `AUTO_MAX_Y` rows of `AUTO_MAX_X` bytes,
/// indexed `[y][x]`. Empty = not allocated (C `NULL`).
pub type BorgData = Vec<[u8; AUTO_MAX_X as usize]>;

/// ghijk  The borg will use the following ddx and ddy to search
/// d827a  for a suitable grid in an open room.
/// e4@3b
/// f615c
/// lmnop  24 grids
pub const BORG_DDX_DDD: [i16; 24] =
    [0, 0, 1, -1, 1, -1, 1, -1, 2, 2, 2, -2, -2, -2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2];
/// See `BORG_DDX_DDD`.
pub const BORG_DDY_DDD: [i16; 24] =
    [1, -1, 0, 0, 1, 1, -1, -1, -1, 0, 1, -1, 0, 1, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2];

/// `(borg_ddx_ddd[i], borg_ddy_ddd[i])`; panics past the 24 offsets.
pub fn borg_ddxy_ddd(i: usize) -> (i32, i32) {
    (i32::from(BORG_DDX_DDD[i]), i32::from(BORG_DDY_DDD[i]))
}

/// borg-flow.c globals.
pub struct FlowState {
    /// `flow_head`: next free slot of the queue
    pub head: i32,
    /// `flow_tail`: next entry to dequeue
    pub tail: i32,
    /// `borg_flow_n` (never read, in C either)
    #[allow(dead_code)]
    pub n: i16,
    /// `borg_flow_x`: the queue (`AUTO_FLOW_MAX` entries)
    pub flow_x: Vec<u8>,
    /// `borg_flow_y`
    pub flow_y: Vec<u8>,
    /// `borg_data_flow`: current "flow" data
    pub data_flow: BorgData,
    /// `borg_data_cost`: current "cost" data
    pub data_cost: BorgData,
    /// `borg_data_hard`: constant "hard" data (all 255)
    pub data_hard: BorgData,
    /// `borg_data_know`: current "know" flags
    pub data_know: BorgData,
    /// `borg_data_icky`: current "icky" flags
    pub data_icky: BorgData,
    /// borg-reborn-rs (#556): the spread running is a kill flow's
    /// (`borg_flow_spread_hunting`), which crosses remembered breeders.
    pub hunting: bool,
    /// borg-reborn-rs (#750): the spread running is a flight's stair flow
    /// with `caution_stairs_flight_ignore_danger`: no grid is refused for its
    /// danger (as the desperate walk); a monster on the path is the test.
    pub flight_no_danger: bool,
    /// borg-reborn-rs (#713): the spread running is a stair flow's
    /// (`borg_flow_spread_stairs`), which may pass through the reach of an
    /// immobile creature kept at range (never onto its grid).
    pub to_stairs: bool,
    /// A weighted stair route gives one step per observation.
    weighted_flight: WeightedFlight,
    /// `borg_temp_n`: entries in use in the temporary grid list
    pub temp_n: i16,
    /// `borg_temp_x` (`AUTO_TEMP_MAX` entries)
    pub temp_x: Vec<u8>,
    /// `borg_temp_y`
    pub temp_y: Vec<u8>,
    /// `track_step`: the Borg's steps
    pub track_step: StairTrack,
    /// `track_door`: doors the Borg closed
    pub track_door: StairTrack,
    /// `track_closed`: doors that started closed
    pub track_closed: StairTrack,
}

impl Default for FlowState {
    /// The C statics: zeroed fixed arrays, `NULL` grid maps, empty tracks.
    fn default() -> Self {
        FlowState {
            head: 0,
            tail: 0,
            n: 0,
            flow_x: vec![0; AUTO_FLOW_MAX as usize],
            flow_y: vec![0; AUTO_FLOW_MAX as usize],
            data_flow: Vec::new(),
            data_cost: Vec::new(),
            data_hard: Vec::new(),
            data_know: Vec::new(),
            data_icky: Vec::new(),
            hunting: false,
            flight_no_danger: false,
            to_stairs: false,
            weighted_flight: WeightedFlight::None,
            temp_n: 0,
            temp_x: vec![0; AUTO_TEMP_MAX],
            temp_y: vec![0; AUTO_TEMP_MAX],
            track_step: StairTrack::default(),
            track_door: StairTrack::default(),
            track_closed: StairTrack::default(),
        }
    }
}

impl FlowState {
    /// `borg_data_cost->data[y][x]`.
    pub fn cost_at(&self, y: i32, x: i32) -> u8 {
        self.data_cost[y as usize][x as usize]
    }

    /// `(borg_flow_y[0], borg_flow_x[0])`: the first grid enqueued by the
    /// last flow.
    pub fn first(&self) -> (i32, i32) {
        (i32::from(self.flow_y[0]), i32::from(self.flow_x[0]))
    }

    /// `borg_temp_x[borg_temp_n] = x; borg_temp_y[borg_temp_n] = y;
    /// borg_temp_n++;` (panics past `AUTO_TEMP_MAX`, where C overruns).
    pub fn temp_push(&mut self, y: i32, x: i32) {
        let n = self.temp_n as usize;
        self.temp_x[n] = x as u8;
        self.temp_y[n] = y as u8;
        self.temp_n += 1;
    }

    /// `(borg_temp_y[i], borg_temp_x[i])`.
    pub fn temp_at(&self, i: usize) -> (i32, i32) {
        (i32::from(self.temp_y[i]), i32::from(self.temp_x[i]))
    }

    /// Enqueue (x, y) at the head of the circular queue (overflow drops the
    /// entry, as in C).
    fn enqueue(&mut self, y: i32, x: i32) {
        /* Enqueue that entry */
        self.flow_x[self.head as usize] = x as u8;
        self.flow_y[self.head as usize] = y as u8;

        /* Circular queue -- memorize head */
        let old_head = self.head;

        /* Circular queue -- insert with wrap */
        self.head += 1;
        if self.head == AUTO_FLOW_MAX {
            self.head = 0;
        }

        /* Circular queue -- handle overflow (badly) */
        if self.head == self.tail {
            self.head = old_head;
        }
    }
}

/// `memcpy(dst, src, sizeof(borg_data))` (both must be allocated).
pub fn copy_data(dst: &mut BorgData, src: &BorgData) {
    assert!(!dst.is_empty() && !src.is_empty(), "borg_data not allocated");
    dst.copy_from_slice(src);
}

/// `memset(d, 0, sizeof(borg_data))` (must be allocated).
pub fn zero_data(d: &mut BorgData) {
    assert!(!d.is_empty(), "borg_data not allocated");
    for row in d.iter_mut() {
        row.fill(0);
    }
}

fn new_data(v: u8) -> BorgData {
    vec![[v; AUTO_MAX_X as usize]; AUTO_MAX_Y as usize]
}

/// `I2D(dir)`: the key of a direction.
fn i2d(dir: i32) -> keycode_t {
    (dir + i32::from(b'0')) as keycode_t
}

/// `strstr(hay, needle) != NULL` on C strings.
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    needle.is_empty() || hay.windows(needle.len()).any(|w| w == needle)
}

/// Check if the borg can dig.
///   check_fail = check if the spell failure rate is too high
///   feat = the type of terrain to dig
pub fn borg_can_dig(st: &mut BorgState, check_fail: bool, feat: u8) -> bool {
    let feat = u32::from(feat);

    /* No digging when hungry */
    if st.borg.tr(BI_ISHUNGRY) != 0 {
        return false;
    }

    /* some features can't be dug out */
    if feat == FEAT_PERM || feat == FEAT_LAVA || (feat < FEAT_SECRET && feat != FEAT_CLOSED) {
        return false;
    }

    let mut dig_check = if feat == FEAT_GRANITE || feat == FEAT_CLOSED || feat == FEAT_SECRET {
        BORG_DIG_HARD
    } else if feat == FEAT_QUARTZ || feat == FEAT_QUARTZ_K {
        BORG_DIG_MOD
    } else {
        BORG_DIG
    };

    /*
     * try digging even if it is hard when out of moves - without twitching,
     * the check is twenty over the threshold so this still prevents trying
     * the impossible
     */
    let twitch = i32::from(st.borg.times_twitch);
    if twitch > 10 {
        dig_check -= (twitch - 10).min(19);
    }

    let weapon_swap = st.trait_swap.weapon_swap;
    if (weapon_swap != 0
        && st.borg.tr(BI_DIG) >= dig_check
        && u32::from(st.items.items[(weapon_swap - 1) as usize].tval) == TV_DIGGING)
        || st.borg.tr(BI_DIG) >= dig_check + 20
    {
        return true;
    }

    if (feat == FEAT_RUBBLE || feat == FEAT_PASS_RUBBLE) && st.borg.tr(BI_ISWEAK) == 0 {
        return true;
    }

    if check_fail {
        if borg_spell_okay_fail(st, TURN_STONE_TO_MUD, 40)
            || borg_spell_okay_fail(st, SHATTER_STONE, 40)
            || borg_equips_item(st, st.act.stone_to_mud, true)
            || borg_equips_ring(st, st.item_val.sv_ring_digging)
        {
            return true;
        }
    } else if borg_spell_okay(st, TURN_STONE_TO_MUD)
        || borg_spell_okay(st, SHATTER_STONE)
        || borg_equips_item(st, st.act.stone_to_mud, false)
        || borg_equips_ring(st, st.item_val.sv_ring_digging)
    {
        return true;
    }

    false
}

/// Clear the "flow" information.
pub fn borg_flow_clear(st: &mut BorgState) {
    let f = &mut st.flow;

    /* Reset the "cost" fields */
    copy_data(&mut f.data_cost, &f.data_hard);

    /* Wipe costs and danger */
    if st.danger.wipe {
        /* Wipe the "know" flags */
        zero_data(&mut f.data_know);

        /* Wipe the "icky" flags */
        zero_data(&mut f.data_icky);

        /* Wipe complete */
        st.danger.wipe = false;
    }

    /* Start over */
    f.head = 0;
    f.tail = 0;
}

/// The "Increase bravery" chain: how much danger a grid may have before it
/// is marked "icky".
fn flow_fear(st: &BorgState) -> i32 {
    let b = &st.borg;
    let avoidance = i32::from(b.avoidance);
    let mut fear = 0;

    /* Increase bravery */
    if b.tr(BI_MAXCLEVEL) == 50 {
        fear = avoidance * 5 / 10;
    }
    if b.tr(BI_MAXCLEVEL) != 50 {
        fear = avoidance * 3 / 10;
    }
    if b.mon.scary {
        fear = avoidance * 2;
    }
    if b.mon.unique != 0 && b.status.vault && b.tr(BI_MAXCLEVEL) == 50 {
        fear = avoidance * 3;
    }
    if b.mon.scary && b.tr(BI_CLEVEL) <= 5 {
        fear = avoidance * 3;
    }
    if b.goal.ignoring {
        fear = avoidance * 5;
    }
    if b.time.now - b.time.level > 5000 {
        fear = avoidance * 25;
    }
    if b.tr(BI_FOOD) == 0 {
        fear = avoidance * 100;
    }

    /* Normal in town */
    if b.tr(BI_CLEVEL) == 0 {
        fear = avoidance * 3 / 10;
    }

    fear
}

/// Is any in-bounds neighbour of (x, y) holding a monster?
fn monster_adjacent(st: &BorgState, x: i32, y: i32) -> bool {
    /* Scan the neighbors */
    for ii in 0..8 {
        /* Neighbor grid */
        let xx = x + i32::from(engine::ddx_ddd(ii));
        let yy = y + i32::from(engine::ddy_ddd(ii));

        /* only on legal grids */
        if !engine::square_in_bounds_fully(xx, yy) {
            continue;
        }

        /* Make sure no monster is on this grid, which is
         * adjacent to the grid on which, I am thinking about
         * stepping.
         */
        if st.cave.grid(xx, yy).kill != 0 {
            return true;
        }
    }
    false
}

/// The grid tests of `borg_flow_spread()` before the icky/know/danger
/// checks: `true` if the flow may not enter `ag` at (x, y).
fn spread_blocked(
    st: &BorgState,
    ag: &BorgGrid,
    x: i32,
    y: i32,
    avoid: bool,
    tunneling: bool,
    twitchy: bool,
) -> bool {
    let b = &st.borg;
    let feat = u32::from(ag.feat);

    /* Avoid "wall" grids (not doors) unless tunneling*/
    /* HACK depends on FEAT order, kinda evil */
    if !tunneling && (feat >= FEAT_SECRET && feat != FEAT_PASS_RUBBLE && feat != FEAT_LAVA) {
        return true;
    }

    /* Avoid "perma-wall" grids */
    if feat == FEAT_PERM {
        return true;
    }

    /* Avoid "Lava" grids (for now) */
    if feat == FEAT_LAVA && b.tr(BI_IFIRE) == 0 {
        return true;
    }

    /* Avoid unknown grids (if requested or retreating)
     * unless twitchy.  In which case, explore it
     */
    if (avoid || b.status.desperate) && feat == FEAT_NONE && !twitchy {
        return true;
    }

    // borg-reborn-rs (#713): never into the reach of an immobile creature
    // kept at range: its grid (a step into it is a blow) or a grid next to
    // it (an item under a jelly drew the Borg back beside it, 60 touches).
    // Not the Borg's own grid, so a flow can always lead out (as
    // `breeder_blocks` keeps the grids next to the Borg open).
    if (x, y) != (b.c.x, b.c.y) && crate::borg_caution::in_reach_of_kept(st, x, y) {
        return true;
    }
    if ag.kill != 0 && crate::borg_caution::immobile_kept_at_range(st, usize::from(st.kills.kills[usize::from(ag.kill)].r_idx)) {
        return true;
    }

    /* flowing into monsters */
    if ag.kill != 0 {
        /* Avoid if Desperate */
        if b.status.desperate {
            return true;
        }

        /* Avoid if afraid */
        if b.tr(BI_ISAFRAID) != 0 {
            return true;
        }

        /* Avoid if low level, unless twitchy */
        if !twitchy && b.tr(BI_FOOD) >= 2 && b.tr(BI_MAXCLEVEL) < 5 {
            return true;
        }
    }

    /* Avoid shop entry points if I am not heading to that shop */
    if b.goal.shop >= 0
        && engine::feat_is_shop(i32::from(ag.feat))
        && i32::from(ag.store) != i32::from(b.goal.shop)
        && (y != b.c.y || x != b.c.x)
    {
        return true;
    }

    /* Avoid Traps if low level-- unless brave */
    if ag.trap && !ag.glyph && !twitchy {
        /* Do not disarm when you could end up dead */
        if b.tr(BI_CURHP) < 60 {
            return true;
        }

        /* Do not disarm when clumsy */
        /* since traps can be physical or magical, gotta check both */
        if b.tr(BI_DISP) < 30 && b.tr(BI_CLEVEL) < 20 {
            return true;
        }
        if b.tr(BI_DISP) < 45 && b.tr(BI_CLEVEL) < 10 {
            return true;
        }
        if b.tr(BI_DISM) < 30 && b.tr(BI_CLEVEL) < 20 {
            return true;
        }
        if b.tr(BI_DISM) < 45 && b.tr(BI_CLEVEL) < 10 {
            return true;
        }

        /* NOTE:  Traps are tough to deal with as a low
         * level character.  If any modifications are made above,
         * then the same changes must be made to borg_flow_direct()
         * and borg_flow_interesting()
         */
    }

    false
}

/// Spread a "flow" from the "destination" grids outwards
///
/// We fill in the "cost" field of every grid that the player can
/// "reach" with the number of steps needed to reach that grid,
/// if the grid is "reachable", and otherwise, with "255", which
/// is the largest possible value that can be stored in a byte.
///
/// Thus, certain grids which are actually "reachable" but only by
/// a path which is at least 255 steps in length will thus appear
/// to be "unreachable", but this is not a major concern.
///
/// We use the "flow" array as a "circular queue", and thus we must
/// be careful not to allow the "queue" to "overflow".  This could
/// only happen with a large number of distinct destination points,
/// each several units away from every other destination point, and
/// in a dungeon with no walls and no dangerous monsters.  But this
/// is technically possible, so we must check for it just in case.
///
/// We do not need a "priority queue" because the cost from grid to
/// grid is always "one" and we process them in order.  If we did
/// use a priority queue, this function might become unusably slow,
/// unless we reactivated the "room building" code.
///
/// We handle both "walls" and "danger" by marking every grid which
/// is "impassible", due to either walls, or danger, as "ICKY", and
/// marking every grid which has been "checked" as "KNOW", allowing
/// us to only check the wall/danger status of any grid once.  This
/// provides some important optimization, since many "flows" can be
/// done before the "ICKY" and "KNOW" flags must be reset.
///
/// Note that the "borg_enqueue_grid()" function should refuse to
/// enqueue "dangerous" destination grids, but does not need to set
/// the "KNOW" or "ICKY" flags, since having a "cost" field of zero
/// means that these grids will never be queued again.  In fact,
/// the "borg_enqueue_grid()" function can be used to enqueue grids
/// which are "walls", such as "doors" or "rubble".
///
/// This function is extremely expensive, and is a major bottleneck
/// in the code, due more to internal processing than to the use of
/// the "borg_danger()" function, especially now that the use of the
/// "borg_danger()" function has been optimized several times.
///
/// The "optimize" flag allows this function to stop as soon as it
/// finds any path which reaches the player, since in general we are
/// looking for paths to destination grids which the player can take,
/// and we can stop this function as soon as we find any usable path,
/// since it will always be as short a path as possible.
///
/// We queue the "children" in reverse order, to allow any "diagonal"
/// neighbors to be processed first, since this may boost efficiency.
///
/// Note that we should recalculate "danger", and reset all "flows"
/// if we notice that a wall has disappeared, and if one appears, we
/// must give it a maximal cost, and mark it as "icky", in case it
/// was currently included in any flow.
///
/// If a "depth" is given, then the flow will only be spread to that
/// depth, note that the maximum legal value of "depth" is 250.
///
/// "Avoid" flag means the borg will not move onto unknown grids,
/// nor to Monster grids if borg_desperate is set.
///
/// "Sneak" will have the borg avoid grids which are adjacent to a monster.
/// borg-reborn-rs (#556): a spread refuses grid (`x`, `y`) when it is near a
/// remembered breeder, unless the spread is a kill flow's (`hunting`) or the
/// grid is next to the Borg at `borg` (its first step).
fn breeder_blocks(near: bool, hunting: bool, borg: crate::headers::z_type_h::loc, x: i32, y: i32) -> bool {
    near && !hunting && (x - borg.x).abs().max((y - borg.y).abs()) > 1
}

/// #765: search from the Borg so entering a square pays its danger, including
/// the staircase. A second search can exclude grids refused by the clean-path
/// check. The old flow's cached fear veto does not apply to weighted routes.
pub(crate) fn weighted_stair_path(
    st: &mut BorgState,
    stairs: &[loc],
    depth: i32,
    avoid: bool,
    sneak: bool,
    weight: i32,
    mut allowed: impl FnMut(&mut BorgState, loc) -> bool,
) -> Option<StairPath> {
    let was = std::mem::replace(&mut st.flow.to_stairs, true);
    let twitchy = i32::from(st.borg.avoidance) > st.borg.tr(BI_CURHP);
    let hp = st.borg.tr(BI_CURHP);
    let path = least_cost_stair_path(st.borg.c, stairs, depth.min(249), |grid| {
        let loc { x, y } = grid;
        if !engine::square_in_bounds_fully(x, y) {
            return None;
        }
        let ag = *st.cave.grid(x, y);
        if (ag.kill != 0 && st.kills.kills[usize::from(ag.kill)].awake)
            || spread_blocked(st, &ag, x, y, avoid, false, twitchy)
            || (sneak && !st.borg.status.desperate && !twitchy && monster_adjacent(st, x, y))
            || breeder_blocks(crate::borg_caution::near_known_breeder(st, x, y), st.flow.hunting, st.borg.c, x, y)
            || !allowed(st, grid)
        {
            return None;
        }
        Some(stair_entry_cost(borg_danger(st, y, x, 1, true, false), hp, weight))
    });
    st.flow.to_stairs = was;
    path
}

pub(crate) struct StairPath {
    pub grids: Vec<loc>,
    pub cost: u64,
}

/// Hundredths of a step. The weight is the extra steps for danger equal to
/// current HP. Even fractional steps of danger can matter on low-weight runs.
fn stair_entry_cost(danger: i32, hp: i32, weight: i32) -> u64 {
    100 + 100 * weight as u64 * danger.max(0) as u64 / hp.max(1) as u64
}

struct StairLabel {
    grid: loc,
    cost: u64,
    steps: i32,
    previous: usize,
}

fn stair_grid_index(grid: loc) -> usize {
    (grid.y * AUTO_MAX_X + grid.x) as usize
}

/// The straight-line number of king moves is a lower bound on both the
/// remaining moves and the remaining cost (100 per move before danger).
fn stair_moves_lower_bound(grid: loc, stairs: &[loc]) -> i32 {
    stairs.iter().map(|goal| (goal.x - grid.x).abs().max((goal.y - grid.y).abs())).min().unwrap_or(0)
}

/// A* with both weighted cost and a move limit. A cheaper route to a
/// grid may use more moves, so keep each arrival that trades cost for moves.
/// Queue ties use fewer projected moves, then greater progress and insertion order.
fn least_cost_stair_path(
    start: loc,
    stairs: &[loc],
    depth: i32,
    mut enter: impl FnMut(loc) -> Option<u64>,
) -> Option<StairPath> {
    if stairs.is_empty() {
        return None;
    }
    let grid_count = (AUTO_MAX_Y * AUTO_MAX_X) as usize;
    let mut arrivals = vec![Vec::<usize>::new(); grid_count];
    let mut entry_costs = vec![None; grid_count];
    let mut labels = vec![StairLabel { grid: start, cost: 0, steps: 0, previous: 0 }];
    arrivals[stair_grid_index(start)].push(0);
    let initial_moves = stair_moves_lower_bound(start, stairs);
    let mut queue = BinaryHeap::from([Reverse((100 * initial_moves as u64, initial_moves, Reverse(0), 0))]);

    while let Some(Reverse((_, _, _, id))) = queue.pop() {
        let StairLabel { grid, cost, steps, .. } = labels[id];
        if !arrivals[stair_grid_index(grid)].contains(&id) {
            continue;
        }
        if stairs.contains(&grid) {
            let mut path = vec![grid];
            let mut previous = id;
            while previous != 0 {
                previous = labels[previous].previous;
                path.push(labels[previous].grid);
            }
            path.reverse();
            return Some(StairPath { grids: path, cost });
        }
        if steps >= depth {
            continue;
        }

        for i in 0..8 {
            let (dx, dy) = borg_ddxy_ddd(i);
            let next = loc { x: grid.x + dx, y: grid.y + dy };
            if !(1..AUTO_MAX_X - 1).contains(&next.x) || !(1..AUTO_MAX_Y - 1).contains(&next.y) {
                continue;
            }
            let index = stair_grid_index(next);
            let next_steps = steps + 1;
            let remaining = stair_moves_lower_bound(next, stairs);
            if next_steps + remaining > depth {
                continue;
            }
            let Some(entry) = *entry_costs[index].get_or_insert_with(|| enter(next)) else {
                continue;
            };
            let next_cost = cost + entry;
            let at = &mut arrivals[index];
            if at.iter().any(|&other| labels[other].cost <= next_cost && labels[other].steps <= next_steps) {
                continue;
            }
            at.retain(|&other| labels[other].cost < next_cost || labels[other].steps < next_steps);
            let next_id = labels.len();
            labels.push(StairLabel { grid: next, cost: next_cost, steps: next_steps, previous: id });
            at.push(next_id);
            queue.push(Reverse((next_cost + 100 * remaining as u64, next_steps + remaining, Reverse(next_cost), next_id)));
        }
    }
    None
}

/// borg-reborn-rs (#556): `borg_flow_spread` for a flow toward monsters
/// borg-reborn-rs (#713): `borg_flow_spread` for a flow to stairs: it may
/// pass through the reach of an immobile creature kept at range (owner: "do
/// not block the path through their reach if they need to go to their
/// stairs"); its own grid stays blocked.
pub fn borg_flow_spread_stairs(
    st: &mut BorgState,
    depth: i32,
    optimize: bool,
    avoid: bool,
    tunneling: bool,
    stair_idx: i32,
    sneak: bool,
) {
    let was = std::mem::replace(&mut st.flow.to_stairs, true);
    borg_flow_spread(st, depth, optimize, avoid, tunneling, stair_idx, sneak);
    st.flow.to_stairs = was;
}

/// (the kill flows, their digging): it may cross the grids near remembered
/// breeders (#503's hunt goes to them).
pub fn borg_flow_spread_hunting(
    st: &mut BorgState,
    depth: i32,
    optimize: bool,
    avoid: bool,
    tunneling: bool,
    stair_idx: i32,
    sneak: bool,
) {
    let was = std::mem::replace(&mut st.flow.hunting, true);
    borg_flow_spread(st, depth, optimize, avoid, tunneling, stair_idx, sneak);
    st.flow.hunting = was;
}

pub fn borg_flow_spread(
    st: &mut BorgState,
    depth: i32,
    mut optimize: bool,
    avoid: bool,
    tunneling: bool,
    stair_idx: i32,
    sneak: bool,
) {
    let mut o = 0;

    /* Default starting points */
    let mut origin_y = st.borg.c.y;
    let mut origin_x = st.borg.c.x;

    /* Is the borg moving under boosted bravery? */
    let twitchy = i32::from(st.borg.avoidance) > st.borg.tr(BI_CURHP);

    // borg-reborn-rs (#647): fleeing in town, no grid worse than here.
    let fear_floor = crate::borg_caution::town_flee_fear_floor(st);

    /* Use the closest stair for calculation distance (cost) from the stair to
     * the goal */
    if stair_idx >= 0 && st.borg.tr(BI_CLEVEL) < 15 {
        origin_y = st.stairs.less.y[stair_idx as usize];
        origin_x = st.stairs.less.x[stair_idx as usize];
        optimize = false;
    }

    /* Now process the queue */
    while st.flow.head != st.flow.tail {
        /* Extract the next entry */
        let x1 = i32::from(st.flow.flow_x[st.flow.tail as usize]);
        let y1 = i32::from(st.flow.flow_y[st.flow.tail as usize]);

        /* Circular queue -- dequeue the next entry */
        st.flow.tail += 1;
        if st.flow.tail == AUTO_FLOW_MAX {
            st.flow.tail = 0;
        }

        /* Cost (one per movement grid) */
        let n = i32::from(st.flow.data_cost[y1 as usize][x1 as usize]) + 1;

        /* New depth */
        if n > o {
            /* Optimize (if requested) */
            if optimize && n > i32::from(st.flow.data_cost[origin_y as usize][origin_x as usize]) {
                break;
            }

            /* Limit depth */
            if n > depth {
                break;
            }

            /* Save */
            o = n;
        }

        /* Queue the "children" */
        for i in 0..8 {
            /* Neighbor grid */
            let x = x1 + i32::from(engine::ddx_ddd(i));
            let y = y1 + i32::from(engine::ddy_ddd(i));

            /* only on legal grids */
            if !engine::square_in_bounds_fully(x, y) {
                continue;
            }

            /* Skip "reached" grids */
            if i32::from(st.flow.data_cost[y as usize][x as usize]) <= n {
                continue;
            }

            /* Access the grid */
            let ag = *st.cave.grid(x, y);

            /* The grid I am thinking about is adjacent to a monster */
            if sneak && !st.borg.status.desperate && !twitchy && monster_adjacent(st, x, y) {
                continue;
            }

            if spread_blocked(st, &ag, x, y, avoid, tunneling, twitchy) {
                continue;
            }

            // borg-reborn-rs (#556): no flow but a kill flow crosses the
            // grids near a remembered breeder; the grids next to the Borg
            // stay open, so its first step can still leave the colony.
            if breeder_blocks(crate::borg_caution::near_known_breeder(st, x, y), st.flow.hunting, st.borg.c, x, y) {
                continue;
            }

            /* Ignore "icky" grids */
            if st.flow.data_icky[y as usize][x as usize] != 0 {
                continue;
            }

            /* Analyze every grid once (unless this mode skips the danger
             * check) */
            if st.flow.data_know[y as usize][x as usize] == 0 && !borg_flow_ignores_danger(st) {
                /* Mark as known */
                st.flow.data_know[y as usize][x as usize] = 1;

                /* Get the danger */
                let p = borg_danger(st, y, x, 1, true, false);

                /* Increase bravery */
                let fear = flow_fear(st).max(fear_floor);

                /* Dangerous grid */
                if p > fear {
                    /* Mark as icky */
                    st.flow.data_icky[y as usize][x as usize] = 1;

                    /* Ignore this grid */
                    continue;
                }
            }

            /* Save the flow cost */
            st.flow.data_cost[y as usize][x as usize] = n as u8;

            /* Enqueue that entry */
            st.flow.enqueue(y, x);
        }
    }

    /* Forget the flow info */
    st.flow.head = 0;
    st.flow.tail = 0;
}

/// Do the flows of the current mode skip the danger check
/// (`borg_flow_ignores_danger`)?
///
/// Retreating flows and digging flows deliberately go
/// through grids whatever their danger. A grid they pass has therefore not
/// been checked, and must not be remembered as checked: a later flow in the
/// normal mode would otherwise accept it without ever assessing its danger.
fn borg_flow_ignores_danger(st: &BorgState) -> bool {
    st.borg.status.desperate || st.borg.status.digging || st.flow.flight_no_danger
}

/// Enqueue a fresh (legal) starting grid, if it is safe.
pub fn borg_flow_enqueue_grid(st: &mut BorgState, y: i32, x: i32) {
    /* Avoid icky grids */
    if st.flow.data_icky[y as usize][x as usize] != 0 {
        return;
    }

    /* Unknown */
    if st.flow.data_know[y as usize][x as usize] == 0 && !borg_flow_ignores_danger(st) {
        /* Mark as known */
        st.flow.data_know[y as usize][x as usize] = 1;

        /* Mark dangerous grids as icky */

        /* Get the danger */
        let p = borg_danger(st, y, x, 1, true, false);

        /* Increase bravery */
        // borg-reborn-rs (#647): fleeing in town, no grid worse than here.
        let fear = flow_fear(st).max(crate::borg_caution::town_flee_fear_floor(st));

        /* Dangerous grid */
        if p > fear {
            /* Icky */
            st.flow.data_icky[y as usize][x as usize] = 1;

            /* Avoid */
            return;
        }
    }

    /* Only enqueue a grid once */
    if st.flow.data_cost[y as usize][x as usize] == 0 {
        return;
    }

    /* Save the flow cost (zero) */
    st.flow.data_cost[y as usize][x as usize] = 0;

    /* Enqueue that entry */
    st.flow.enqueue(y, x);
}

/// Commit the current "flow".
pub fn borg_flow_commit(st: &mut BorgState, who: &CStr, why: i32) -> bool {
    flow_commit(st, Some(who.to_bytes()), why)
}

/// `borg_flow_commit(who, why)` with `who` possibly `NULL` (no note).
pub fn flow_commit(st: &mut BorgState, who: Option<&[u8]>, why: i32) -> bool {
    /* Cost of current grid */
    let cost = i32::from(st.flow.cost_at(st.borg.c.y, st.borg.c.x));

    /* Verify the total "cost" */
    if cost >= 250 {
        return false;
    }

    /* Message */
    if let Some(who) = who {
        let mut msg = b"# Flowing toward ".to_vec();
        msg.extend_from_slice(who);
        msg.extend_from_slice(format!(" at cost {cost}").as_bytes());
        borg_note_bytes(st, &msg);
    }

    /* Obtain the "flow" information */
    let f = &mut st.flow;
    copy_data(&mut f.data_flow, &f.data_cost);
    f.weighted_flight = WeightedFlight::None;

    /* Save the goal type */
    st.borg.goal.type_0 = why as i16;
    // borg-reborn-rs (#614, review of #616): only a stair flow sets it again.
    st.borg.flight_to_stairs = false;

    /* Success */
    true
}

#[derive(Clone, Copy)]
enum WeightedFlight {
    None,
    Step(loc),
    Replan,
}

/// Expose actual move counts to the existing flight and rest checks.
/// The chosen first grid also gets stored explicitly, since two distant
/// points on a weighted path can be adjacent on the map.
pub(crate) fn commit_weighted_stair_path(
    st: &mut BorgState,
    who: &CStr,
    why: i32,
    path: &[loc],
) -> bool {
    let Some(&next) = path.get(1) else { return false };
    copy_data(&mut st.flow.data_cost, &st.flow.data_hard);
    for (steps, grid) in path.iter().rev().enumerate() {
        st.flow.data_cost[grid.y as usize][grid.x as usize] = steps as u8;
    }
    if !borg_flow_commit(st, who, why) {
        return false;
    }
    st.flow.weighted_flight = WeightedFlight::Step(next);
    st.borg.flight_to_stairs = true;
    borg_flow_old(st, why)
}

/// A note `prefix` + `name` + `' at (y,x)`.
fn note_at(st: &mut BorgState, prefix: &str, name: &[u8], y: i32, x: i32) {
    let mut msg = prefix.as_bytes().to_vec();
    msg.extend_from_slice(name);
    msg.extend_from_slice(format!("' at ({y},{x})").as_bytes());
    borg_note_bytes(st, &msg);
}

/// Breeder levels: close an open door next to the Borg (not the goal
/// grid). `true` if a door was found (and a close queued).
fn close_breeder_door(st: &mut BorgState, y2: i32, x2: i32) -> bool {
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    let mut o_y = 0;
    let mut o_x = 0;
    let mut door_found = 0;

    /* scan the adjacent grids */
    for ox in -1..=1 {
        for oy in -1..=1 {
            /* skip our own spot */
            if oy + cy == cy && ox + cx == cx {
                continue;
            }

            /* Acquire location */
            let ag = *st.cave.grid(ox + cx, oy + cy);

            /* skip non open doors */
            if u32::from(ag.feat) != FEAT_OPEN {
                continue;
            }

            /* skip monster on door */
            if ag.kill != 0 {
                continue;
            }

            /* skip our original goal */
            if oy + cy == y2 && ox + cx == x2 {
                continue;
            }

            /* save this spot */
            o_y = oy;
            o_x = ox;
            door_found += 1;
        }
    }

    /* Is there a door to close? */
    if door_found == 0 {
        return false;
    }

    /* Get a direction, if possible */
    let dir = borg_flow_misc::borg_goto_dir(st, cy, cx, cy + o_y, cx + o_x);

    /* Obtain the destination */
    let x = st.borg.c.x + i32::from(engine::ddx(dir as usize));
    let y = st.borg.c.y + i32::from(engine::ddy(dir as usize));

    /* Set goal */
    st.borg.goal.g.x = x;
    st.borg.goal.g.y = y;

    /* Close */
    borg_note(st, "# Closing a door");
    borg_keypress(st, keycode_t::from(b'c'));
    borg_queue_direction(st, i2d(dir));

    /* Check for an existing flag */
    let t = &mut st.flow.track_door;
    let mut i = 0;
    while i < i32::from(t.num) {
        /* Stop if we already new about this door */
        if t.x[i as usize] == x && t.y[i as usize] == y {
            return true;
        }
        i += 1;
    }

    /* Track the newly closed door */
    if i == i32::from(st.flow.track_door.num) && i < i32::from(st.flow.track_door.size) {
        borg_note(st, "# Noting the closing of a door.");
        let t = &mut st.flow.track_door;
        t.num += 1;
        t.x[i as usize] = x;
        t.y[i as usize] = y;
    }
    true
}

/// Step onto an object grid (chests, Orb of Draining, pick up).
fn step_take(st: &mut BorgState, ag: &BorgGrid, dir: i32, y2: i32, x2: i32) -> bool {
    let t = usize::from(ag.take);
    let take = st.takes.takes[t];
    let name = engine::kind_name(take.kidx as usize);

    /*** Handle Chests ***/
    /* The borg will cheat when it comes to chests.
     * He does not have to but it makes him faster and
     * it does not give him any information that a
     * person would not know by looking at the trap.
     * So there is no advantage to the borg.
     */

    if contains(name, b"chest") && !contains(name, b"Ruined") {
        let Some(o) = engine::top_floor_object(&st.access, x2, y2) else {
            /* this should only happen when something picks up the chest */
            /* outside the borgs view.  */
            borg_flow_take::borg_delete_take(st, i32::from(ag.take));
            return false;
        };

        /* Traps. Disarm it w/ fail check */
        let pval = i32::from(o.pval());
        if pval > 1
            && o.has_known()
            && st.borg.tr(BI_DEV) - pval >= st.core.cfg[BORG_CHEST_FAIL_TOLERANCE as usize]
        {
            note_at(st, "# Disarming a '", name, i32::from(take.y), i32::from(take.x));

            /* Open it */
            borg_keypress(st, keycode_t::from(b'D'));
            borg_queue_direction(st, i2d(dir));
            return true;
        }

        /* No trap, or unknown trap that passed above checks - Open it */
        if pval < 0 || !o.has_known() {
            note_at(st, "# Opening a '", name, i32::from(take.y), i32::from(take.x));

            /* Open it */
            borg_keypress(st, keycode_t::from(b'o'));
            borg_queue_direction(st, i2d(dir));
            return true;
        }

        /* Empty chest */
        /* continue in routine and pick it up */
    }

    /*** Handle Orb of Draining ***/

    /* Priest/Paladin borgs who have very limited ID ability can save some
     * money and inventory space my casting Orb of Draining on objects.
     * Cursed objects will melt under the Orb of Draining spell.  This will
     * save the borg from carrying the item around until he can ID it.
     *
     * By default, the flag ORBED is set to false when an item is created.
     * If the borg gets close to an item, and the conditions are favorable,
     * he will cast OoD on the item and change the flag.
     */
    if !take.orbed
        && (take.tval >= TV_SHOT as i32 && take.tval < TV_STAFF as i32)
        && engine::distance(loc { x: i32::from(take.x), y: i32::from(take.y) }, st.borg.c) == 1
        && borg_spell_okay_fail(st, ORB_OF_DRAINING, 25)
    {
        /* Target the Take location */
        borg_target(st, loc { x: i32::from(take.x), y: i32::from(take.y) }, false);

        /* Cast the prayer */
        borg_spell(st, ORB_OF_DRAINING);

        /* Message */
        borg_note(st, "# Orbing an object to check for cursed item.");

        /* use the old target */
        borg_keypress(st, keycode_t::from(b'5'));

        /* Change the take flag */
        st.takes.takes[t].orbed = true;

        /* check the blast radius of the prayer for other items */
        for i in 0..24 {
            /* Extract the location */
            let (dx, dy) = borg_ddxy_ddd(i);
            let xx = i32::from(st.takes.takes[t].x) + dx;
            let yy = i32::from(st.takes.takes[t].y) + dy;

            /* Check the grid for a take */
            if !engine::square_in_bounds_fully(xx, yy) {
                continue;
            }
            let t2 = st.cave.grid(xx, yy).take;
            if t2 != 0 {
                /* This item was orbed (mostly true)*/
                st.takes.takes[usize::from(t2)].orbed = true;
            }
        }

        /* Return */
        return true;
    }

    /*** Handle other takes ***/
    /* Message */
    note_at(st, "# Walking onto and deleting a '", name, i32::from(take.y), i32::from(take.x));

    /* Delete the item from the list */
    borg_flow_take::borg_delete_take(st, i32::from(ag.take));

    /* Walk onto it */
    borg_keypress(st, i2d(dir));

    true
}

/// `borg_spell(TURN_STONE_TO_MUD) || borg_spell(SHATTER_STONE) ||
/// borg_activate_ring(sv_ring_digging) || borg_activate_item(act_stone_to_mud)`
fn melt(st: &mut BorgState) -> bool {
    borg_spell(st, TURN_STONE_TO_MUD)
        || borg_spell(st, SHATTER_STONE)
        || borg_activate_ring(st, st.item_val.sv_ring_digging)
        || borg_activate_item(st, st.act.stone_to_mud)
}

/// Step into a closed door: open (or melt) it.
fn step_closed_door(st: &mut BorgState, dir: i32) -> bool {
    /* Paranoia XXX XXX XXX */
    if engine::randint0(100) == 0 {
        return false;
    }

    /* Not a good idea to open locked doors if a monster
     * is next to the borg beating on him
     */

    /* scan the adjacent grids */
    for i in 0..8 {
        /* Grid in that direction */
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

        /* If monster adjacent to me and I'm weak, don't
         * even try to open the door
         */
        if st.cave.grid(x, y).kill != 0
            && st.borg.tr(BI_CLEVEL) < 15
            && st.borg.tr(BI_ISAFRAID) == 0
        {
            return false;
        }
    }

    /* Use other techniques from time to time */
    if engine::randint0(100) == 0 || st.borg.antibounce_count >= 500 {
        /* Mega-Hack -- allow "destroy doors" */
        if borg_spell(st, DISABLE_TRAPS_DESTROY_DOORS)
            || borg_activate_item(st, st.act.destroy_doors)
        {
            borg_note(st, "# Disable Traps, Destroy Doors");
            return true;
        }

        /* Mega-Hack -- allow "stone to mud" */
        if melt(st) {
            borg_note(st, "# Melting a door");
            borg_keypress(st, i2d(dir));

            /* Remove this closed door from the list.
             * Its faster to clear all doors from the list
             * then rebuild the list.
             */
            if st.flow.track_closed.num != 0 {
                st.flow.track_closed.num = 0;
            }
            return true;
        }
    }

    /* Open */
    if st.borg.status.need_alter {
        borg_keypress(st, keycode_t::from(b'+'));
        st.borg.status.need_alter = false;
    } else {
        borg_note(st, "# Opening a door");
        borg_keypress(st, keycode_t::from(b'o'));
    }
    borg_queue_direction(st, i2d(dir));

    /* Remove this closed door from the list.
     * Its faster to clear all doors from the list
     * then rebuild the list.
     */
    if st.flow.track_closed.num != 0 {
        st.flow.track_closed.num = 0;
    }

    true
}

/// Take one "step" towards the given location, return true if possible.
fn borg_play_step(st: &mut BorgState, y2: i32, x2: i32) -> bool {
    /* Breeder levels, close all doors */
    if st.borg.near.breeder && close_breeder_door(st, y2, x2) {
        return true;
    }

    /* Stand stairs up */
    if st.borg.goal.less {
        /* Define the grid we are looking at to be our own grid */
        let feat = st.cave.grid(st.borg.c.x, st.borg.c.y).feat;

        /* Up stairs. Cheat the game grid info in.
         * (cave_feat[borg.c.y][borg.c.x] == FEAT_LESS) */
        if u32::from(feat) == FEAT_LESS && !engine::player_opt(OPT_birth_force_descend) {
            st.borg.goal.less = false;
            borg_keypress(st, keycode_t::from(b'<'));

            /* Success */
            return true;
        }
    }

    /* Get a direction, if possible */
    let dir = borg_flow_misc::borg_goto_dir(st, st.borg.c.y, st.borg.c.x, y2, x2);

    /* We have arrived */
    if dir == 5 {
        return false;
    }

    /* Obtain the destination */
    let x = st.borg.c.x + i32::from(engine::ddx(dir as usize));
    let y = st.borg.c.y + i32::from(engine::ddy(dir as usize));

    /* Access the grid we are stepping on */
    let ag = *st.cave.grid(x, y);

    /* Set goal */
    st.borg.goal.g.x = x;
    st.borg.goal.g.y = y;

    /* Monsters -- Attack */
    if ag.kill != 0 {
        let kill = st.kills.kills[usize::from(ag.kill)];
        let r_idx = i32::from(kill.r_idx);

        /* dead monsters or "player ghosts" */
        if r_idx == 0 || r_idx >= i32::from(engine::z_info().r_max) - 1 {
            return false;
        }

        /* can't attack someone if afraid! */
        if st.borg.tr(BI_ISAFRAID) != 0 || st.borg.tr(BI_CRSFEAR) != 0 {
            return false;
        }

        // borg-reborn-rs (#713): no walking into an immobile creature kept
        // at range (a flow made before it was known may still lead there).
        if crate::borg_caution::immobile_kept_at_range(st, r_idx as usize) {
            return false;
        }

        /* Ignore Maggot until later. */
        if engine::race_flag_has(r_idx as usize, RF_UNIQUE)
            && st.borg.tr(BI_CDEPTH) == 0
            && st.borg.tr(BI_CLEVEL) < 5
        {
            return false;
        }

        /* Message */
        note_at(
            st,
            "# Walking into a '",
            borg_flow_kill::borg_race_name(r_idx).to_bytes(),
            kill.pos.y,
            kill.pos.x,
        );

        // borg-reborn-rs (#551, owner): a fox changes back before a blow.
        if crate::borg_escape::revert_fox_to_fight(st) {
            return true;
        }

        /* Walk into it */
        if st.borg.status.no_alter {
            borg_keypress(st, keycode_t::from(b';'));
            st.borg.status.no_alter = false;
        } else {
            borg_keypress(st, keycode_t::from(b'+'));
        }
        borg_keypress(st, i2d(dir));
        return true;
    }

    /* Objects -- Take */
    if ag.take != 0 && st.takes.takes[usize::from(ag.take)].kidx != 0 {
        return step_take(st, &ag, dir, y2, x2);
    }

    /* Glyph of Warding */
    if ag.glyph {
        /* Message */
        borg_note(st, "# Walking onto a glyph of warding.");

        /* Walk onto it */
        borg_keypress(st, i2d(dir));
        return true;
    }

    /* Traps -- disarm -- */
    /* NOTE: If a scary guy is on the level, we allow the borg to run over */
    /* the trap in order to escape this level. */
    if st.borg.tr(BI_LIGHT) != 0
        && st.borg.tr(BI_ISBLIND) == 0
        && st.borg.tr(BI_ISCONFUSED) == 0
        && !st.borg.mon.scary
        && ag.trap
    {
        /* allow "destroy doors" activation */
        if borg_activate_item(st, st.act.disable_traps) {
            borg_note(st, "# Activation to Disable Traps, Destroy Doors");
            st.cave.grid_mut(x, y).trap = false;
            /* since this just disables the trap and doesn't remove it, */
            /* don't rest next to it */
            st.borg.no_rest_prep = 3000;

            /* the activation needs to target the trap */
            borg_target(st, st.borg.goal.g, false);
            return true;
        }

        /* allow "destroy doors" spell */
        /* don't bother unless we are near full mana */
        if st.borg.tr(BI_CURSP) > (st.borg.tr(BI_MAXSP) * 4) / 5
            && borg_spell(st, DISABLE_TRAPS_DESTROY_DOORS)
        {
            borg_note(st, "# Disable Traps, Destroy Doors");
            st.cave.grid_mut(x, y).trap = false;
            /* since this just disables the trap and doesn't remove it, */
            /* don't rest next to it */
            st.borg.no_rest_prep = 3000;
            return true;
        }

        /* Disarm */
        borg_note(st, "# Disarming a trap");
        borg_keypress(st, keycode_t::from(b'D'));
        borg_queue_direction(st, i2d(dir));

        /* We are not sure if the trap will get 'untrapped'. pretend it will*/
        st.cave.grid_mut(x, y).trap = false;
        return true;
    }

    let feat = u32::from(ag.feat);

    /* Closed Doors -- Open */
    if feat == FEAT_CLOSED {
        return step_closed_door(st, dir);
    }

    /* eliminate things we can't step on */
    /* we only seem to get this far when trying to dig out vaults */
    if feat == FEAT_PERM {
        return false;
    }

    if feat == FEAT_LAVA && st.borg.tr(BI_IFIRE) == 0 {
        return false;
    }

    /* Rubble, Treasure, Seams, Walls -- Tunnel or Melt */
    /* HACK depends on FEAT order, kinda evil. */
    if (FEAT_SECRET..=FEAT_GRANITE).contains(&feat) {
        /* Don't dig walls and seams when exploring (do dig rubble) */
        if feat != FEAT_RUBBLE && i32::from(st.borg.goal.type_0) == GOAL_DARK {
            return false;
        }

        /* Don't bother digging without sufficient dig ability */
        if !borg_can_dig(st, false, ag.feat) {
            st.borg.goal.type_0 = 0;
            return false;
        }

        /* Use Stone to Mud when available */
        if melt(st) {
            borg_note(st, "# Melting a wall/etc");
            borg_keypress(st, i2d(dir));

            /* Forget number of mineral veins to force rebuild of vein list */
            st.flow_misc.track_vein.num = 0;

            return true;
        }

        /* Mega-Hack -- prevent infinite loops */
        if engine::randint0(500) <= 5 && !st.borg.status.vault {
            return false;
        }

        /* Switch to a digger if we have one is automatic */

        /* Dig */
        borg_note(st, "# Digging through wall/etc");
        borg_keypress(st, keycode_t::from(b'T'));
        borg_keypress(st, i2d(dir));

        /* Forget number of mineral veins to force rebuild of vein list */
        /* XXX Maybe only do this if successful? */
        st.flow_misc.track_vein.num = 0;

        return true;
    }

    /* Shops -- Enter */
    if engine::feat_is_shop(i32::from(ag.feat)) {
        // borg-reborn-rs (#620): no shop takes a fox.
        if crate::borg_escape::revert_fox_at_shop(st) {
            return true;
        }

        /* Message */
        borg_note(st, &format!("# Entering a '{}' shop", ag.store));

        /* Enter the shop */
        borg_keypress(st, i2d(dir));
        return true;
    }

    /* Walk in that direction */
    if st.borg.status.need_alter {
        borg_keypress(st, keycode_t::from(b'+'));
        st.borg.status.need_alter = false;
    }

    /* Actually enter the direction */
    borg_keypress(st, i2d(dir));

    /* I'm not in a store */
    st.borg.in_shop = false;

    /* (C resets a local, never-set event here: no effect) */

    /* Did something */
    true
}

/// Attempt to take an optimal step towards the current goal location
///
/// Note that the "borg_update()" routine notices new monsters and objects,
/// and movement of monsters and objects, and cancels any flow in progress.
///
/// Note that the "borg_update()" routine notices when a grid which was
/// not thought to block motion is discovered to in fact be a grid which
/// blocks motion, and removes that grid from any flow in progress.
///
/// When given multiple alternative steps, this function attempts to choose
/// the "safest" path, by penalizing grids containing embedded gold, monsters,
/// rubble, doors, traps, store doors, and even floors.  This allows the Borg
/// to "step around" dangerous grids, even if this means extending the path by
/// a step or two, and encourages him to prefer grids such as objects and stairs
/// which are not only interesting but which are known not to be invisible traps.
///
/// XXX XXX XXX XXX This function needs some work.  It should attempt to
/// analyze the "local region" around the player and determine the optimal
/// choice of locations based on some useful computations.
///
/// If it works, return true, otherwise, cancel the goal and return false.
pub fn borg_flow_old(st: &mut BorgState, why: i32) -> bool {
    /* Continue */
    if i32::from(st.borg.goal.type_0) != why {
        /* Nothing to do */
        return false;
    }

    let next = match st.flow.weighted_flight {
        WeightedFlight::None => {
            let i = select_step(st);
            (i >= 0).then(|| loc {
                x: st.borg.c.x + i32::from(engine::ddx_ddd(i as usize)),
                y: st.borg.c.y + i32::from(engine::ddy_ddd(i as usize)),
            })
        }
        WeightedFlight::Step(next) => {
            st.flow.weighted_flight = WeightedFlight::Replan;
            Some(next)
        }
        // Let the caller spread again using the current monster positions and HP.
        WeightedFlight::Replan => {
            st.borg.goal.type_0 = 0;
            return false;
        }
    };

    // #788: every town hunt step must still have a winning certificate;
    // a bump attack, closed door, trap or unknown grid is not a plain walk.
    if let Some(to) = next.filter(|_| why == GOAL_KILL) {
        let grid = *st.cave.grid(to.x, to.y);
        if !crate::borg_caution::town_hunt_step_allowed(st, grid) {
            borg_note(st, "# Town hunt declined: no current winning certificate (#788)");
            st.borg.goal.type_0 = 0;
            return false;
        }
    }

    // borg-reborn-rs (#551): a Druid's flight step: Fox Form first.
    if next.is_some() && why == GOAL_FLEE {
        let steps_left = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]);
        // borg-reborn-rs (#614): remember a flight to stairs (rests wait for them).
        if st.borg.flight_to_stairs {
            st.borg.flight_steps = steps_left;
            st.borg.flight_time = st.borg.time.now;
            st.borg.flight_depth = crate::engine::player_depth();
        }
        if crate::borg_escape::fox_form_before_flight_step(st, steps_left) {
            return true;
        }
    }
    // borg-reborn-rs (#620): a low-level Druid's walk in town: Fox Form first.
    if next.is_some() && crate::borg_escape::fox_form_before_town_step(st, why, i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize])) {
        return true;
    }

    /* Try it */
    if let Some(loc { x, y }) = next {
        /* Attempt motion */
        if borg_play_step(st, y, x) {
            return true;
        }
    }

    /* Mark a timestamp to wait on a anti-summon spot for a few turns */
    let (fy, fx) = st.flow.first();
    if i32::from(st.borg.goal.type_0) == GOAL_DIGGING && st.borg.c.y == fy && st.borg.c.x == fx {
        st.borg.time.antisummon = st.borg.time.now;
    }

    /* Cancel goal */
    st.borg.goal.type_0 = 0;

    /* Nothing to do */
    false
}

/// The "Look around" part of `borg_flow_old()`: the direction index
/// (`ddx_ddd`) of the best next grid along the committed flow, or -1.
fn select_step(st: &BorgState) -> i32 {
    let mut b_n = 0;
    let mut b_i: i32 = -1;

    /* Flow cost of current grid */
    let mut b_c = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]) * 10;

    /* Prevent loops */
    b_c -= 5;

    /* Look around */
    for i in 0..8 {
        /* Grid in that direction */
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

        /* Flow cost at that grid */
        let c = i32::from(st.flow.data_flow[y as usize][x as usize]) * 10;

        /* Never backtrack */
        if c > b_c {
            continue;
        }

        /* avoid screen edges */
        if !(1..AUTO_MAX_X).contains(&x) || !(1..AUTO_MAX_Y).contains(&y) {
            continue;
        }

        /* Notice new best value */
        if c < b_c {
            b_n = 0;
        }

        /* Apply the randomizer to equivalent values */
        if st.borg.tr(BI_CDEPTH) == 0 {
            b_n += 1;
            if b_n >= 2 && engine::randint0(b_n as u32) != 0 {
                continue;
            }
        } else if st.borg.tr(BI_CDEPTH) >= 1 {
            b_n += 1;
            if b_n >= 2 {
                continue;
            }
        }

        /* Special case when digging anti-summon corridor */
        if i32::from(st.borg.goal.type_0) == GOAL_DIGGING
            && (engine::ddx_ddd(i) == 0 || engine::ddy_ddd(i) == 0)
        {
            /* No straight lines */
            let (fy, fx) = st.flow.first();
            if engine::distance(st.borg.c, loc { x: fx, y: fy }) <= 2 {
                continue;
            }
        }

        /* Track it */
        b_i = i as i32;
        b_c = c;
    }

    b_i
}

/// `borg_init_flow()`: allocate the grid maps (`borg_data_hard` all 255)
/// and the tracks, then the stair, glyph and misc flow data.
pub fn borg_init_flow(st: &mut BorgState) {
    borg_free_flow(st);

    /*** Grid data ***/
    st.flow.data_flow = new_data(0);
    st.flow.data_cost = new_data(0);

    /* Prepare "borg_data_hard" */
    st.flow.data_hard = new_data(255);
    st.flow.data_know = new_data(0);
    st.flow.data_icky = new_data(0);

    /* Track Steps */
    st.flow.track_step.init(TRACK_SIZE);

    /* Track doors closed by borg */
    st.flow.track_door.init(TRACK_SIZE);

    /* Track closed doors on map */
    st.flow.track_closed.init(TRACK_SIZE);

    borg_flow_stairs::borg_init_flow_stairs(st);
    borg_flow_glyph::borg_init_flow_glyph(st);
    borg_flow_misc::borg_init_flow_misc(st);
}

/// `borg_free_flow()`.
pub fn borg_free_flow(st: &mut BorgState) {
    borg_flow_misc::borg_free_flow_misc(st);
    borg_flow_glyph::borg_free_flow_glyph(st);
    borg_flow_stairs::borg_free_flow_stairs(st);

    st.flow.track_closed.free();
    st.flow.track_door.free();
    st.flow.track_step.free();

    st.flow.data_flow = Vec::new();
    st.flow.data_icky = Vec::new();
    st.flow.data_know = Vec::new();
    st.flow.data_hard = Vec::new();
    st.flow.data_cost = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddd_tables_match_c() {
        let c_x: [i16; 24] =
            [0, 0, 1, -1, 1, -1, 1, -1, 2, 2, 2, -2, -2, -2, -2, -1, 0, 1, 2, -2, -1, 0, 1, 2];
        let c_y: [i16; 24] =
            [1, -1, 0, 0, 1, 1, -1, -1, -1, 0, 1, -1, 0, 1, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2];
        assert_eq!(BORG_DDX_DDD, c_x);
        assert_eq!(BORG_DDY_DDD, c_y);
    }

    #[test]
    fn queue_wraps_and_drops_on_overflow() {
        let mut f = FlowState::default();
        assert_eq!((f.flow_x.len(), f.temp_x.len()), (1536, 9000));
        for i in 0..AUTO_FLOW_MAX + 10 {
            f.enqueue(i % 60, i % 190);
        }
        /* The last slot before the tail is never used */
        assert_eq!(f.head, AUTO_FLOW_MAX - 1);
        f.tail = 5;
        f.head = AUTO_FLOW_MAX - 1;
        f.enqueue(7, 8);
        assert_eq!(f.head, 0);
        assert_eq!((f.flow_y[AUTO_FLOW_MAX as usize - 1], f.flow_x[AUTO_FLOW_MAX as usize - 1]), (7, 8));
    }

    #[test]
    fn temp_list_truncates_like_c() {
        let mut f = FlowState::default();
        f.temp_push(65, 300);
        assert_eq!((f.temp_n, f.temp_at(0)), (1, (65, 44)));
    }

    #[test]
    fn data_helpers() {
        let mut a = new_data(255);
        let mut b = new_data(0);
        b[3][4] = 7;
        copy_data(&mut a, &b);
        assert_eq!((a[3][4], a[0][0], a.len()), (7, 0, 66));
        zero_data(&mut a);
        assert!(a.iter().all(|r| r.iter().all(|&v| v == 0)));
    }

    #[test]
    fn contains_is_strstr() {
        assert!(contains(b"Small wooden chest", b"chest"));
        assert!(!contains(b"Ruined chest", b"chests"));
        assert!(contains(b"abc", b""));
    }
}

#[cfg(test)]
mod breeder_block_tests {
    use super::*;
    use crate::headers::z_type_h::loc;

    /// #556: near a remembered breeder a grid is refused, except for a kill
    /// flow and the grids next to the Borg (its first step).
    #[test]
    fn the_block_spares_kill_flows_and_the_first_step() {
        let borg = loc { x: 10, y: 10 };
        assert!(breeder_blocks(true, false, borg, 12, 10), "two steps away");
        assert!(!breeder_blocks(true, false, borg, 11, 11), "next to the Borg");
        assert!(!breeder_blocks(true, false, borg, 10, 10), "the Borg's grid");
        assert!(!breeder_blocks(true, true, borg, 12, 10), "a kill flow");
        assert!(!breeder_blocks(false, false, borg, 12, 10), "no breeder near");
    }

    #[test]
    fn weighted_stairs_take_a_longer_detour_within_the_move_cap() {
        let start = loc { x: 2, y: 3 };
        let stair = loc { x: 6, y: 3 };
        let enter = |loc { x, y }| match (x, y) {
            (3..=5, 3) => Some(stair_entry_cost(50, 25, 2)),
            (2 | 6, 4) | (3..=5, 5) | (6, 3) => Some(100),
            _ => None,
        };
        let route = least_cost_stair_path(start, &[stair], 6, enter).unwrap().grids;
        assert_eq!(route.len() - 1, 6);
        assert_eq!(route[1], loc { x: 2, y: 4 });
        let route = least_cost_stair_path(start, &[stair], 4, enter).unwrap().grids;
        assert_eq!(route[1], loc { x: 3, y: 3 });
        assert!(least_cost_stair_path(start, &[stair], 3, enter).is_none());
    }

    #[test]
    fn weighted_stairs_pay_for_entering_the_goal_and_keep_costs_above_a_byte() {
        let start = loc { x: 2, y: 3 };
        let near = loc { x: 3, y: 3 };
        let far = loc { x: 4, y: 4 };
        let enter = |grid| {
            if grid == near {
                Some(stair_entry_cost(5, 10, 5))
            } else {
                Some(stair_entry_cost(0, 10, 5))
            }
        };
        let route = least_cost_stair_path(start, &[near, far], 2, enter).unwrap();
        assert_eq!(route.grids.last(), Some(&far));
        let route = least_cost_stair_path(start, &[near], 1, enter).unwrap();
        assert_eq!(route.grids, vec![start, near]);
        assert_eq!(stair_entry_cost(1, 20, 1), 105);
    }

    #[test]
    fn weighted_stairs_route_around_blocked_grids_or_refuse_the_only_corridor() {
        let start = loc { x: 2, y: 3 };
        let stair = loc { x: 4, y: 3 };
        let blocked = loc { x: 3, y: 3 };
        let detour = loc { x: 3, y: 4 };
        let enter = |grid| [blocked, stair].contains(&grid).then_some(100);
        let route = least_cost_stair_path(start, &[stair], 2, enter).unwrap();
        assert_eq!(route.grids[1], blocked);
        let enter = |grid| [detour, stair].contains(&grid).then_some(100);
        let route = least_cost_stair_path(start, &[stair], 2, enter).unwrap();
        assert_eq!(route.grids[1], detour);
        assert!(least_cost_stair_path(start, &[stair], 2, |grid| (grid == stair).then_some(100)).is_none());
    }
}

/// borg-reborn-rs (#732; owner, 2026-11-20: "remove our leash completely and
/// expose the leash of C with configuration parameters"): C's leash numbers
/// (`borg_get_leash`, `borg_flow_far_from_stairs_dist`, `take_select`), each
/// a parameter whose compiled default is C's literal, read here (the
/// registry's `flow_` file).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CLeash {
    pub per_level: i32,
    pub base: i32,
    pub unbounded: i32,
    pub pickup_max_level: i32,
    pub depth_margin: i32,
    pub twitch_after: i32,
    pub stairs_max_level: i32,
    pub take_near_max_level: i32,
    pub take_nearness: i32,
    pub take_max_level: i32,
}

/// borg-reborn-rs (#761, owner: "make sure it picks it up"): the floor rule
/// for a Phase Door while the Borg holds fewer than `flow_take_phase_short_below`
/// of them (0 = off): the stair leash of `take_select` does not apply to it;
/// with `flow_take_phase_first` it is also the only object flowed to while one
/// is in reach. Returns (short_below, first).
pub(crate) fn take_phase_rule() -> (i32, bool) {
    use crate::params::{value, Param};
    (value(Param::flow_take_phase_short_below), value(Param::flow_take_phase_first) != 0)
}

/// The leash parameters (`CLeash`).
pub(crate) fn c_leash() -> CLeash {
    use crate::params::{value, Param};
    CLeash {
        per_level: value(Param::flow_leash_per_level),
        base: value(Param::flow_leash_base),
        unbounded: value(Param::flow_leash_unbounded),
        pickup_max_level: value(Param::flow_leash_pickup_max_level),
        depth_margin: value(Param::flow_leash_depth_margin),
        twitch_after: value(Param::flow_leash_twitch_after),
        stairs_max_level: value(Param::flow_leash_stairs_max_level),
        take_near_max_level: value(Param::flow_leash_take_near_max_level),
        take_nearness: value(Param::flow_leash_take_nearness),
        take_max_level: value(Param::flow_leash_take_max_level),
    }
}

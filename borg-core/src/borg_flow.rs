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
    FEAT_CLOSED, FEAT_GRANITE, FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN, FEAT_PASS_RUBBLE,
    FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K, FEAT_RUBBLE, FEAT_SECRET,
};
use crate::headers::list_tvals_h::{TV_DIGGING, TV_SHOT, TV_STAFF};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::borg::BorgTime;
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
pub(crate) const GOAL_DARK: i32 = 4;
const GOAL_DIGGING: i32 = 10;
/// `GOAL_FLEE` (borg-flow.h): flee the level (#551).
const GOAL_FLEE: i32 = 7;

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
    /// borg-reborn-rs (#1130, review of #1152): counts the flows committed
    /// (`flow_commit`): a brave walk is continued only while its own flow is
    /// the current one.
    pub commit_seq: u32,
    /// borg-reborn-rs (#556): the spread running is a kill flow's
    /// (`borg_flow_spread_hunting`), which crosses remembered breeders.
    pub hunting: bool,
    /// borg-reborn-rs (#750): the spread running is a flight's stair flow
    /// with `caution_stairs_flight_ignore_danger`: no grid is refused for its
    /// danger (as the desperate walk); a monster on the path is the test.
    pub flight_no_danger: bool,
    /// borg-reborn-rs (#1067): the spread running is `borg_trap_way`'s,
    /// which crosses known traps (to reopen a way back they cut).
    pub through_traps: bool,
    /// borg-reborn-rs (#713): the spread running is a stair flow's
    /// (`borg_flow_spread_stairs`), which may pass through the reach of an
    /// immobile creature kept at range (never onto its grid).
    pub to_stairs: bool,
    /// A weighted stair route gives one step per observation.
    weighted_flight: WeightedFlight,
    /// This committed flight explicitly chose an up staircase.
    up_stair_flight: bool,
    /// Recent tactical retreats made during a dungeon departure.
    retreats: Vec<FlightRetreat>,
    /// Squares excluded by the currently spreading stair route.
    retreat_avoid: Option<Vec<loc>>,
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
            commit_seq: 0,
            hunting: false,
            flight_no_danger: false,
            through_traps: false,
            to_stairs: false,
            weighted_flight: WeightedFlight::None,
            up_stair_flight: false,
            retreats: Vec::new(),
            retreat_avoid: None,
            temp_n: 0,
            temp_x: vec![0; AUTO_TEMP_MAX],
            temp_y: vec![0; AUTO_TEMP_MAX],
            track_step: StairTrack::default(),
            track_door: StairTrack::default(),
            track_closed: StairTrack::default(),
        }
    }
}

#[derive(Clone, Copy)]
struct FlightRetreat {
    from: loc,
    landing: loc,
    depth: i32,
    level_started: i32,
    turn: i32,
    state: RetreatLanding,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RetreatLanding {
    Pending,
    Confirmed,
    Reported,
}

impl FlightRetreat {
    fn still_applies(self, here: loc, depth: i32, level_started: i32, now: i32, window: i32) -> bool {
        self.depth == depth
            && self.level_started == level_started
            && (1..=window).contains(&now.wrapping_sub(self.turn))
            && here != self.from
            && (self.state != RetreatLanding::Pending || here == self.landing)
    }
}

/// The retreat is provisional until the next observation sees its landing.
pub(crate) fn remember_flight_retreat(st: &mut BorgState) {
    let depth = st.borg.tr(BI_CDEPTH);
    let window = crate::borg_caution::flight_retreat_ticks();
    if window == 0 || depth <= 0 || !(st.borg.goal.fleeing || st.borg.goal.leaving)
        || st.borg.c == st.borg.goal.g
    {
        return;
    }
    let level_started = st.borg.time.level;
    let turn = st.borg.time.now;
    let from = st.borg.c;
    st.flow.retreats.retain(|r| r.from != from && r.still_applies(from, depth, level_started, turn, window));
    st.flow.retreats.push(FlightRetreat {
        from,
        landing: st.borg.goal.g,
        depth,
        level_started,
        turn,
        state: RetreatLanding::Pending,
    });
}

/// Confirm the observed landing before caution can choose another retreat.
pub(crate) fn observe_flight_retreats(st: &mut BorgState) {
    if st.flow.retreats.is_empty() {
        return;
    }
    st.flow.observe_retreats(
        st.borg.c,
        st.borg.tr(BI_CDEPTH),
        st.borg.time.level,
        st.borg.time.now,
        crate::borg_caution::flight_retreat_ticks(),
    );
}

/// Exclude recent dangerous retreat squares from every candidate stair
/// route. Keep them out of the shared icky-grid cache.
fn active_retreat_grids(st: &mut BorgState, why: i32) -> Vec<loc> {
    let departing = st.borg.goal.fleeing || st.borg.goal.leaving;
    if st.flow.retreats.is_empty() || !departing || !crate::borg_caution::flight_goal(why, departing) {
        return Vec::new();
    }
    let depth = st.borg.tr(BI_CDEPTH);
    let window = crate::borg_caution::flight_retreat_ticks();
    let threshold = crate::borg_caution::retreat_danger_threshold(st);
    let mut retreats = std::mem::take(&mut st.flow.retreats);
    retreats.retain_mut(|retreat| {
        if retreat.state == RetreatLanding::Pending
            || !retreat.still_applies(st.borg.c, depth, st.borg.time.level, st.borg.time.now, window)
            || crate::borg_caution::retreat_hold_danger(st, retreat.from.y, retreat.from.x) <= threshold
        {
            return false;
        }
        if retreat.state == RetreatLanding::Confirmed {
            retreat.state = RetreatLanding::Reported;
            let note = format!("# Stair flight avoiding retreat grid ({},{})", retreat.from.x, retreat.from.y);
            borg_note_bytes(st, note.as_bytes());
        }
        true
    });
    let blocked = retreats.iter().map(|r| r.from).collect();
    st.flow.retreats = retreats;
    blocked
}

pub(crate) fn with_retreat_avoidance<R>(st: &mut BorgState, why: i32, plan: impl FnOnce(&mut BorgState) -> R) -> R {
    // Probes and the route they validate must use the same exclusions.
    if st.flow.retreat_avoid.is_some() {
        return plan(st);
    }
    st.flow.retreat_avoid = Some(active_retreat_grids(st, why));
    let result = plan(st);
    st.flow.retreat_avoid = None;
    result
}

impl FlowState {
    fn observe_retreats(&mut self, here: loc, depth: i32, level_started: i32, now: i32, window: i32) {
        self.retreats.retain_mut(|retreat| {
            if !retreat.still_applies(here, depth, level_started, now, window) {
                return false;
            }
            if retreat.state == RetreatLanding::Pending {
                retreat.state = RetreatLanding::Confirmed;
            }
            true
        });
    }

    pub(crate) fn has_retreat_exclusions(&self) -> bool {
        self.retreat_avoid.as_ref().is_some_and(|grids| !grids.is_empty())
    }

    fn retreat_blocks(&self, grid: loc) -> bool {
        self.retreat_avoid.as_ref().is_some_and(|grids| grids.contains(&grid))
    }

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
/// borg-reborn-rs (#1153, review of #1158): could a flow at the normal
/// avoidance enter (x, y)? The same test as every spread (`spread_blocked`):
/// a closed door is open to it, lava only with fire immunity, an unknown grid
/// not.
pub(crate) fn flow_open(st: &BorgState, x: i32, y: i32) -> bool {
    !spread_blocked(st, st.cave.grid(x, y), x, y, true, false, false)
}

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

    if st.flow.retreat_blocks(loc { x, y }) {
        return true;
    }

    // borg-reborn-rs (#781): observed unseen occupancy blocks every spread.
    if crate::borg_caution::unseen_bump_blocked(st, x, y) {
        return true;
    }

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
    // borg-reborn-rs (#837): nor back into the reach of an immobile attacker
    // it has just stepped away from (`caution_immobile_no_return`).
    if (x, y) != (b.c.x, b.c.y) && crate::borg_caution::in_reach_of_stepped_from(st, x, y) {
        return true;
    }
    // borg-reborn-rs (#1125, `caution_paralyser_no_reach`): without Free
    // Action, never onto a known paralyser's grid or next to it, stair
    // flights included. Not the Borg's own grid, so a flow can lead out.
    if (x, y) != (b.c.x, b.c.y) && crate::borg_caution::in_paralyser_reach(st, x, y) {
        return true;
    }
    // borg-reborn-rs (#1153, review of #1158): a loop's grids closed by the
    // loop breaker are closed to every spread, so a flow routes around them.
    if (x, y) != (b.c.x, b.c.y) && crate::borg_loop::step_banned(st, y, x) {
        return true;
    }
    // borg-reborn-rs (#1147, `caution_walk_away_immobile`): never into a
    // known walk-away monster's reach, stair flights included.
    if (x, y) != (b.c.x, b.c.y) && crate::borg_caution::in_walk_away_reach(st, x, y) {
        return true;
    }
    if ag.kill != 0 && crate::borg_caution::kept_at_range(st, usize::from(st.kills.kills[usize::from(ag.kill)].r_idx)) {
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
    // borg-reborn-rs (#1067): not for `trap_way_back`'s own spread.
    if ag.trap && !ag.glyph && !twitchy && !st.flow.through_traps && !crate::borg_trap_way::trap_underfoot_open(st, x, y) {
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
    let unknown_rule = crate::borg_caution::stair_flight_unknown_rule();
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
        // borg-reborn-rs (#881): an unknown grid is priced or refused.
        let unknown = flight_unknown_extra(u32::from(ag.feat) == FEAT_NONE, unknown_rule)?;
        Some(stair_entry_cost(borg_danger(st, y, x, 1, true, false), hp, weight) + unknown)
    });
    st.flow.to_stairs = was;
    path
}

/// borg-reborn-rs (#881): the extra cost, in hundredths of a step, of
/// entering a grid on a weighted stair flight; None refuses it. Unseen rock
/// behind the pillars of a lattice room looked like a shortcut to the
/// corridor (Priest 399497898: 58 of 68 flight paths crossed it).
fn flight_unknown_extra(unknown: bool, (mode, steps): (i32, i32)) -> Option<u64> {
    match (unknown, mode) {
        (false, _) | (true, 0) => Some(0),
        (true, 1) => Some(100 * steps.max(0) as u64),
        _ => None,
    }
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
    if st.flow.retreat_blocks(loc { x, y }) {
        return;
    }
    // borg-reborn-rs (#1153, review of #1158): no destination on a grid the
    // loop breaker closed.
    if crate::borg_loop::step_banned(st, y, x) {
        return;
    }
    // borg-reborn-rs (#1125, review of #1129, finding 1): a destination in a
    // known paralyser's reach is not seeded either (`spread_blocked` only
    // sees the grids the flow spreads to; a staircase next to a floating eye
    // was still a goal). Not the Borg's own grid.
    if (x, y) != (st.borg.c.x, st.borg.c.y) && crate::borg_caution::in_paralyser_reach(st, x, y) {
        return;
    }
    // borg-reborn-rs (#1147): nor in a walk-away monster's reach.
    if (x, y) != (st.borg.c.x, st.borg.c.y) && crate::borg_caution::in_walk_away_reach(st, x, y) {
        return;
    }
    // borg-reborn-rs (#781): a blocked staircase must not seed a flow.
    if crate::borg_caution::unseen_bump_blocked(st, x, y) {
        return;
    }
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
    f.up_stair_flight = why == GOAL_FLEE && who == Some(b"up-stairs");

    /* Save the goal type */
    st.borg.goal.type_0 = why as i16;
    st.flow.commit_seq = st.flow.commit_seq.wrapping_add(1); // borg-reborn-rs (#1130)
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
    // borg-reborn-rs (#1061): stair_scum peeks before walking through.
    crate::borg_stair_scum::opened_door(st, dir);

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

    // borg-reborn-rs (#781): also refuse an old flow made before the bump.
    if crate::borg_caution::unseen_bump_blocked(st, x, y) {
        st.borg.goal.type_0 = 0;
        return false;
    }

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
        if crate::borg_caution::kept_at_range(st, r_idx as usize) {
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
        crate::borg_caution::unseen_bump_step(st, x, y); // borg-reborn-rs (#781)
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
        crate::borg_trap_way::note_disarm(st, x, y); // borg-reborn-rs (#1067)
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
    // borg-reborn-rs (#781): only a real movement command supplies a grid.
    crate::borg_caution::unseen_bump_step(st, x, y);
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
/// borg-reborn-rs (#1130): a committed brave stair walk (`Borg::brave_walk`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BraveWalk {
    /// The level it was started on (`time.level`).
    pub level: BorgTime,
    /// The avoidance the walk was spread with.
    pub avoidance: i16,
    /// The think of its last step (`time.now`).
    pub last: BorgTime,
    /// The steps it may still take.
    pub steps_left: i32,
    /// The flow it committed (`FlowState::commit_seq`): another flow taking
    /// over ends the walk (review of #1152).
    pub seq: u32,
    /// The stair search that started it, used again to rebuild its route
    /// (review of #1152).
    pub kind: BraveStairs,
    /// Started by the loop breaker (#1153): continued under `loop_breaker`,
    /// at whatever avoidance it was spread with.
    pub from_loop: bool,
}

/// #1130 (review of #1152): which stair search a brave walk started with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BraveStairs {
    /// `borg_flow_stair_less`.
    Up,
    /// `borg_flow_stair_both` (the return leash's any staircase, #1062).
    Any,
    /// `borg_flow_stair_more` (brave).
    Down,
}

/// #1130: how many thinks after its last step a brave walk is still
/// continued (a think spent on something else, an attack or a rest, does not
/// drop it).
const BRAVE_WALK_GAP: BorgTime = 3;

/// #1130: steps a brave walk may take beyond the route's cost when it starts
/// (a route through unknown grids changes as they are seen).
const BRAVE_WALK_SLACK: i32 = 10;

/// #1130 (Blackguard 312081895: 60,000 turns between two grids): a stair
/// walk under boosted bravery took one step; the next think, at the normal
/// avoidance, its step was refused (the goal cancelled) and exploration took
/// the step back; the stuck think then boosted again. With
/// `flow_brave_walk_commit`, the walk is committed: `borg_flow_old`
/// continues it with the avoidance it was spread with, while it is recent, has
/// steps left, is on its own level, and the Borg is in no danger above half
/// its HP here (a monster that shows up is caution's).
pub(crate) fn brave_walk_avoidance(st: &BorgState, danger_here: i32) -> Option<i16> {
    use crate::params::{value, Param};
    let w = st.borg.brave_walk?;
    if !own_flow(w, st.flow.commit_seq) {
        return None;
    }
    let on = if w.from_loop { value(Param::loop_breaker) != 0 } else { value(Param::flow_brave_walk_commit) != 0 };
    brave_walk_live(
        on,
        w,
        st.borg.time.level,
        st.borg.time.now,
        danger_here,
        st.borg.tr(BI_CURHP),
    )
    .then_some(w.avoidance)
}

/// #1130 (review of #1152): is the walk's own flow the current one? Another
/// flow committed since (a breeder exit, a kill flow...) ends it.
fn own_flow(w: BraveWalk, commit_seq: u32) -> bool {
    w.seq == commit_seq
}

/// #1130 (review of #1152): the walk after one continued step, at think
/// `now`, its flow now `commit_seq` (a rebuild commits a new one): one step
/// is charged, whether the old route or a rebuilt one took it.
fn advance(w: BraveWalk, now: BorgTime, commit_seq: u32) -> BraveWalk {
    BraveWalk { last: now, steps_left: w.steps_left - 1, seq: commit_seq, ..w }
}

/// #1130: the conditions of `brave_walk_avoidance`, without the engine.
fn brave_walk_live(on: bool, w: BraveWalk, level: BorgTime, now: BorgTime, danger_here: i32, curhp: i32) -> bool {
    on && w.level == level && now.wrapping_sub(w.last) <= BRAVE_WALK_GAP && w.steps_left > 0 && danger_here <= curhp / 2
}

/// #1130 (review of #1152): record the brave walk the brave think's stair
/// search `kind` just committed, only while the avoidance is boosted (above
/// the HP): its budget is the route's cost here plus `BRAVE_WALK_SLACK`.
pub(crate) fn commit_brave_walk(st: &mut BorgState, kind: BraveStairs) {
    use crate::params::{value, Param};
    if value(Param::flow_brave_walk_commit) == 0
        || i32::from(st.borg.goal.type_0) != GOAL_FLEE
        || i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP)
    {
        return;
    }
    let cost = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]);
    st.borg.brave_walk = Some(BraveWalk {
        level: st.borg.time.level,
        avoidance: st.borg.avoidance,
        last: st.borg.time.now,
        steps_left: cost.saturating_add(BRAVE_WALK_SLACK),
        seq: st.flow.commit_seq,
        kind,
        from_loop: false,
    });
}

/// #1153: commit the stair walk the loop breaker just started (at the normal
/// avoidance, or boosted), so the next thinks continue it as a brave walk is
/// continued (#1130): a walk taken for one step and then dropped would let
/// the loop start again.
pub(crate) fn commit_loop_walk(st: &mut BorgState, kind: BraveStairs) {
    use crate::params::{value, Param};
    if value(Param::loop_breaker) == 0 || i32::from(st.borg.goal.type_0) != GOAL_FLEE {
        return;
    }
    let cost = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]);
    st.borg.brave_walk = Some(BraveWalk {
        level: st.borg.time.level,
        avoidance: st.borg.avoidance,
        last: st.borg.time.now,
        steps_left: cost.saturating_add(BRAVE_WALK_SLACK),
        seq: st.flow.commit_seq,
        kind,
        from_loop: true,
    });
}

pub fn borg_flow_old(st: &mut BorgState, why: i32) -> bool {
    // borg-reborn-rs (#1130): a committed brave stair walk continues with the
    // avoidance it was spread with.
    if why == GOAL_FLEE && i32::from(st.borg.goal.type_0) == GOAL_FLEE {
        // The rebuild's own first step (its stair search calls back here):
        // only the step; the outer call does the accounting (review of #1152).
        if BRAVE_WALK_RESPREAD.with(std::cell::Cell::get) {
            return borg_flow_old_step(st, why);
        }
        let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        if let (Some(to), Some(walk)) = (brave_walk_avoidance(st, here), st.borg.brave_walk) {
            let saved = st.borg.avoidance;
            st.borg.avoidance = to;
            st.danger.wipe = true;
            // The flow buffer is shared: other spreads earlier in this think
            // may have replaced the walk's, so the route is rebuilt with the
            // walk's own stair search when the old step does not go.
            let mut stepped = borg_flow_old_step(st, why);
            if !stepped {
                BRAVE_WALK_RESPREAD.with(|busy| busy.set(true));
                stepped = brave_walk_respread(st, walk.kind);
                BRAVE_WALK_RESPREAD.with(|busy| busy.set(false));
            }
            st.borg.avoidance = saved;
            st.danger.wipe = true;
            st.borg.brave_walk = stepped.then(|| advance(walk, st.borg.time.now, st.flow.commit_seq));
            if stepped {
                borg_note(st, "# Continuing the brave stair walk (#1130)");
            }
            return stepped;
        }
    }
    borg_flow_old_step(st, why)
}

thread_local! {
    /// #1130: set while a brave walk's route is spread again, whose own
    /// `borg_flow_old` must not spread it again in turn.
    static BRAVE_WALK_RESPREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// #1130 (review of #1152): rebuild a brave walk's route with the stair
/// search that started it.
fn brave_walk_respread(st: &mut BorgState, kind: BraveStairs) -> bool {
    match kind {
        BraveStairs::Up => borg_flow_stairs::borg_flow_stair_less(st, GOAL_FLEE, false),
        BraveStairs::Any => borg_flow_stairs::borg_flow_stair_both(st, GOAL_FLEE, false),
        BraveStairs::Down => borg_flow_stairs::borg_flow_stair_more(st, GOAL_FLEE, false, true),
    }
}

fn borg_flow_old_step(st: &mut BorgState, why: i32) -> bool {
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

    // A continued stair flow may have been spread before the retreat.
    if st.borg.flight_to_stairs && next.is_some_and(|grid| {
        st.flow.retreats.iter().any(|retreat| retreat.from == grid)
            && with_retreat_avoidance(st, why, |st| st.flow.retreat_blocks(grid))
    }) {
        st.borg.goal.type_0 = 0;
        return false;
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

    // borg-reborn-rs (#1061, review P1): stair_scum checks every voluntary
    // step, a resumed flow's too, against its safe area.
    if next.is_some_and(|n| !crate::borg_stair_scum::step_allowed(st, why, n.y, n.x)) {
        st.borg.goal.type_0 = 0;
        return false;
    }

    /* Try it */
    if let Some(loc { x, y }) = next {
        let here = st.borg.c;
        let known = why == GOAL_FLEE && st.flow.up_stair_flight;
        let departure = if (why == GOAL_DARK || known) && st.borg.tr(BI_CDEPTH) > 0
            && st.borg.stair_less && (st.borg.goal.leaving || st.borg.goal.fleeing)
        {
            let stair = if u32::from(st.cave.grid(here.x, here.y).feat) == FEAT_MORE {
                Some(here)
            } else if known {
                st.stairs.up_search_departure
                    .filter(|search| search.can_renew(st, true) && search.stair != (loc { x, y }))
                    .map(|search| search.stair)
            } else if borg_flow_stairs::continuing_up_stair_search(st) {
                st.stairs.up_search_departure.map(|search| search.stair)
            } else {
                None
            };
            stair.map(|stair| borg_flow_stairs::UpSearchDeparture {
                stair,
                next: loc { x, y },
                level_started: st.borg.time.level,
                turn: st.borg.time.now,
                expected_danger: borg_danger(st, y, x, 1, true, false),
                known,
            })
        } else {
            None
        };
        if borg_play_step(st, y, x) {
            match why {
                GOAL_DARK => borg_flow_misc::notice_interrupted_spastic_search(st, loc { x, y }),
                borg_flow_misc::GOAL_XTRA => borg_flow_misc::record_spastic_approach(st, loc { x, y }),
                _ => {}
            }
            st.stairs.up_search_departure = departure;
            if st.borg.tr(BI_CDEPTH) > 0 {
                let emergency_down = why == GOAL_FLEE && st.borg.flight_to_stairs
                    && st.borg.status.desperate && !known;
                borg_flow_stairs::record_off_stair_step(st, loc { x, y }, why == GOAL_DARK, emergency_down);
            }
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

/// A committed exploration flow has a next step that moves away from the
/// down staircase it just left. The normal flow validates that move again.
pub(crate) fn up_stair_search_can_continue(st: &BorgState, stair: loc) -> bool {
    if i32::from(st.borg.goal.type_0) != GOAL_DARK {
        return false;
    }
    let step = select_step(st);
    if step < 0 {
        return false;
    }
    let x = st.borg.c.x + i32::from(engine::ddx_ddd(step as usize));
    let y = st.borg.c.y + i32::from(engine::ddy_ddd(step as usize));
    loc { x, y } != stair && !crate::borg_caution::unseen_bump_blocked(st, x, y)
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

        // borg-reborn-rs (#1153, review of #1158): a cached route never steps
        // onto a grid the loop breaker closed.
        if crate::borg_loop::step_banned(st, y, x) {
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

    /// #881: priced, an unknown shortcut loses to a longer known route;
    /// refused, it is never entered; mode 0 keeps one step.
    #[test]
    fn weighted_stairs_price_or_refuse_unknown_grids() {
        assert_eq!(flight_unknown_extra(true, (0, 5)), Some(0));
        assert_eq!(flight_unknown_extra(false, (2, 5)), Some(0));
        assert_eq!(flight_unknown_extra(true, (1, 5)), Some(500));
        assert_eq!(flight_unknown_extra(true, (2, 5)), None);
        // Straight east through three unknown grids (row 3), or round by
        // known floor on row 5 (six moves).
        let start = loc { x: 2, y: 3 };
        let stair = loc { x: 6, y: 3 };
        let route = |rule| {
            let enter = |loc { x, y }| match (x, y) {
                (3..=5, 3) => flight_unknown_extra(true, rule).map(|extra| 100 + extra),
                (2 | 6, 4) | (3..=5, 5) | (6, 3) => Some(100),
                _ => None,
            };
            least_cost_stair_path(start, &[stair], 10, enter).map(|path| path.grids[1])
        };
        assert_eq!(route((0, 5)), Some(loc { x: 3, y: 3 }));
        assert_eq!(route((1, 5)), Some(loc { x: 2, y: 4 }));
        assert_eq!(route((1, 0)), Some(loc { x: 3, y: 3 }));
        assert_eq!(route((2, 5)), Some(loc { x: 2, y: 4 }));
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

/// borg-reborn-rs (#1062): the switches for the holes of C's leash
/// (`borg_leash.rs`), read here as the registry's `flow_` file wants.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LeashHoles {
    pub any_stair: bool,
    pub unknown_stair_steps: i32,
    pub teleported_recall: bool,
    pub recall_max_level: i32,
    pub recall_steps: i32,
    pub recall_min_jump: i32,
}

/// The `LeashHoles` parameters, on each use (not cached).
pub(crate) fn leash_holes() -> LeashHoles {
    use crate::params::{value, Param};
    LeashHoles {
        any_stair: value(Param::flow_leash_any_stair) != 0,
        unknown_stair_steps: value(Param::flow_leash_unknown_stair_steps),
        teleported_recall: value(Param::flow_leash_teleported_recall) != 0,
        recall_max_level: value(Param::flow_leash_recall_max_level),
        recall_steps: value(Param::flow_leash_recall_steps),
        recall_min_jump: value(Param::flow_leash_recall_min_jump),
    }
}

#[cfg(test)]
mod stair_loop_tests {
    use super::*;

    #[test]
    fn nearby_up_stair_with_a_long_walk_does_not_reverse_weighted_flight() {
        let before = loc { x: 77, y: 10 };
        let beside = loc { x: 76, y: 11 };
        let up = loc { x: 96, y: 10 };
        let down = loc { x: 97, y: 4 };
        // The up stair looks close across the wall. Its approach wraps
        // around the wall, and its first move opposes the ordinary flight.
        let enter = |grid: loc| {
            let loc { x, y } = grid;
            (grid == before
                || (x == 76 && (11..=60).contains(&y))
                || (y == 60 && (76..=96).contains(&x))
                || (x == 96 && (10..=60).contains(&y))
                || ((78..=83).contains(&x) && x + y == 87)
                || (y == 4 && (83..=97).contains(&x)))
                .then_some(100)
        };
        let up_path = least_cost_stair_path(before, &[up], 249, enter).unwrap();
        assert_eq!(up_path.grids.len() - 1, 118);
        assert_eq!(up_path.grids[1], beside);
        let any_path = least_cost_stair_path(beside, &[up, down], 249, enter).unwrap();
        assert_eq!(any_path.grids.len() - 1, 21);
        assert_eq!(any_path.grids[1], before);
        // The scary-level shortcut requires strictly fewer than 20 moves.
        assert!(least_cost_stair_path(before, &[up], 19, enter).is_none());
        let next = least_cost_stair_path(before, &[up, down], 249, enter).unwrap();
        assert_eq!(next.grids.last(), Some(&down));
        assert_ne!(next.grids[1], beside);
    }
}

#[cfg(test)]
mod retreat_flight_tests {
    use super::*;

    fn grid(x: i32, y: i32) -> loc {
        loc { x, y }
    }

    #[test]
    fn the_abandoned_grid_needs_a_confirmed_landing_on_this_level() {
        let retreat = FlightRetreat {
            from: grid(27, 22),
            landing: grid(26, 21),
            depth: 2,
            level_started: 100,
            turn: 105,
            state: RetreatLanding::Pending,
        };
        assert!(retreat.still_applies(retreat.landing, 2, 100, 106, 8));
        assert!(!retreat.still_applies(retreat.from, 2, 100, 106, 8));
        assert!(!retreat.still_applies(grid(28, 21), 2, 100, 106, 8));
        assert!(!retreat.still_applies(retreat.landing, 1, 100, 106, 8));
        assert!(!retreat.still_applies(retreat.landing, 2, 106, 107, 8));
        assert!(!retreat.still_applies(retreat.landing, 2, 100, 114, 8));
        assert!(!retreat.still_applies(retreat.landing, 2, 100, 106, 0));

        let confirmed = FlightRetreat { state: RetreatLanding::Confirmed, ..retreat };
        assert!(confirmed.still_applies(grid(25, 22), 2, 100, 107, 8));
        assert!(!confirmed.still_applies(retreat.from, 2, 100, 107, 8));
        assert!(confirmed.still_applies(grid(24, 22), 2, 100, 108, 8));
    }

    #[test]
    fn two_landings_are_confirmed_before_any_stair_plan() {
        let a = grid(95, 13);
        let b = grid(94, 13);
        let c = grid(93, 12);
        let mut flow = FlowState::default();
        let first = FlightRetreat {
            from: a,
            landing: b,
            depth: 2,
            level_started: 100,
            turn: 105,
            state: RetreatLanding::Pending,
        };
        flow.retreats.push(first);
        flow.observe_retreats(b, 2, 100, 106, 128);
        flow.retreats.push(FlightRetreat { from: b, landing: c, turn: 106, ..first });
        flow.observe_retreats(c, 2, 100, 107, 128);
        assert_eq!(flow.retreats.iter().map(|r| r.from).collect::<Vec<_>>(), vec![a, b]);
        assert!(flow.retreats.iter().all(|r| r.state == RetreatLanding::Confirmed));
    }

    #[test]
    fn a_second_retreat_does_not_open_the_first_square_again() {
        let start = grid(94, 14);
        let stair = grid(96, 12);
        let previous = grid(95, 13);
        let latest = grid(95, 14);
        let alternatives = [grid(94, 13), grid(95, 12)];
        let mut flow = FlowState::default();
        flow.retreat_avoid = Some(vec![previous, latest]);
        let path = least_cost_stair_path(start, &[stair], 3, |at| {
            (!flow.retreat_blocks(at) && (at == stair || alternatives.contains(&at))).then_some(100)
        }).unwrap();
        assert_eq!(path.grids, vec![start, alternatives[0], alternatives[1], stair]);
    }

    #[test]
    fn both_retreat_wings_take_a_detour_instead_of_reentering_the_middle() {
        // Mage: the stair flow returns to the middle from either retreat
        // landing. Ranger: the same graph with only the left landing.
        let middle = grid(27, 22);
        let stair = grid(27, 24);
        let corridor = [middle, grid(27, 23), stair];
        let left = grid(26, 21);
        let right = grid(28, 21);
        let detours = [grid(25, 22), grid(25, 23), grid(26, 24), grid(29, 22), grid(29, 23), grid(28, 24)];
        let mut flow = FlowState::default();

        for landing in [left, right] {
            let enter = |flow: &FlowState, at| {
                (!flow.retreat_blocks(at) && (corridor.contains(&at) || detours.contains(&at)))
                    .then_some(if at == middle { 101 } else { 100 })
            };
            flow.retreat_avoid = None;
            let original = least_cost_stair_path(landing, &[stair], 4, |at| enter(&flow, at)).unwrap();
            assert_eq!(original.grids[1], middle);

            flow.retreat_avoid = Some(vec![middle]);
            let detour = least_cost_stair_path(landing, &[stair], 4, |at| enter(&flow, at)).unwrap();
            assert_eq!(detour.grids.last(), Some(&stair));
            assert!(!detour.grids.contains(&middle));
            assert!(least_cost_stair_path(landing, &[stair], 4, |at| {
                (!flow.retreat_blocks(at) && corridor.contains(&at)).then_some(100)
            }).is_none());
        }
    }
}

#[cfg(test)]
mod brave_walk_tests {
    use super::*;

    fn walk() -> BraveWalk {
        BraveWalk { level: 1000, avoidance: 92, last: 1100, steps_left: 5, seq: 7, kind: BraveStairs::Up, from_loop: false }
    }

    #[test]
    fn a_committed_brave_walk_continues_on_its_level_while_recent() {
        // #1130: Blackguard 312081895, 46 HP, a walk spread at avoidance 92.
        assert!(brave_walk_live(true, walk(), 1000, 1101, 0, 46), "the next think");
        assert!(brave_walk_live(true, walk(), 1000, 1103, 0, 46), "a pause of a few thinks");
        assert!(!brave_walk_live(true, walk(), 1000, 1104, 0, 46), "too old");
        assert!(!brave_walk_live(true, walk(), 1200, 1101, 0, 46), "another level");
        assert!(!brave_walk_live(true, BraveWalk { steps_left: 0, ..walk() }, 1000, 1101, 0, 46), "no steps left");
        assert!(!brave_walk_live(true, walk(), 1000, 1101, 24, 46), "danger above half the HP: caution's");
        assert!(brave_walk_live(true, walk(), 1000, 1101, 23, 46));
        assert!(!brave_walk_live(false, walk(), 1000, 1101, 0, 46), "the switch off: C");
    }

    #[test]
    fn one_step_is_charged_per_continuation_and_another_flow_ends_the_walk() {
        // Review of #1152, findings 1 and 2.
        let w = walk();
        let next = advance(w, 1101, 9);
        assert_eq!(next.steps_left, 4, "one step, even when the route was rebuilt (a new flow)");
        assert_eq!(next.seq, 9, "the rebuilt flow is the walk's own");
        assert_eq!(next.kind, BraveStairs::Up, "the stair search is kept for the next rebuild");
        assert!(own_flow(next, 9));
        assert!(!own_flow(next, 10), "a breeder exit or any other flow committed since");
    }

    #[test]
    fn the_switch_ships_off_in_the_code() {
        assert_eq!(crate::params::value(crate::params::Param::flow_brave_walk_commit), 0);
    }
}


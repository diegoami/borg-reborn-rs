//! Port of `borg-flow-dark.c`: flow (move) toward interesting (dark) grids.
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
//! No module state. The flow arrays (`borg_data_cost/know/icky`) and
//! `track_door` of borg-flow.c (`st.flow`) are read through the small
//! accessors below; the stair distance tests are in `borg_flow_misc`.
//! C bugs kept are listed in `docs/c-bugs.md` (section 29).

use std::ffi::CStr;

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::borg_cave_floor_grid;
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_danger::borg_danger;
use crate::{borg_flow_misc, engine};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_DISM, BI_DISP, BI_FOOD, BI_GOLD, BI_IFIRE, BI_ISCONFUSED,
    BI_ISWEAK, BI_LIGHT, BI_MAXCLEVEL, BI_MAXHP,
};
use crate::headers::list_terrain_h::{
    FEAT_CLOSED, FEAT_GRANITE, FEAT_LAVA, FEAT_MAGMA, FEAT_MAGMA_K, FEAT_NONE, FEAT_PERM,
    FEAT_QUARTZ, FEAT_QUARTZ_K, FEAT_RUBBLE, FEAT_SECRET,
};
use crate::headers::z_type_h::loc;
use crate::params::{value, Param};
use crate::state::BorgState;

/// borg-reborn-rs (#1061)
type Scum = crate::borg_stair_scum::Map;


/// One of the flow arrays of borg-flow.c (`st.flow`).
#[derive(Clone, Copy)]
enum FlowData {
    /// `borg_data_cost`
    Cost,
    /// `borg_data_know`
    Know,
    /// `borg_data_icky`
    Icky,
}

/// `borg_data_<which>->data[y][x]`.
fn borg_data_at(st: &BorgState, which: FlowData, y: i32, x: i32) -> u8 {
    let f = &st.flow;
    let d = match which {
        FlowData::Cost => &f.data_cost,
        FlowData::Know => &f.data_know,
        FlowData::Icky => &f.data_icky,
    };
    assert!(!d.is_empty(), "flow array not allocated");
    d[y as usize][x as usize]
}

/// `borg_data_<which>->data[y][x] = v`.
fn set_borg_data(st: &mut BorgState, which: FlowData, y: i32, x: i32, v: u8) {
    let f = &mut st.flow;
    let d = match which {
        FlowData::Cost => &mut f.data_cost,
        FlowData::Know => &mut f.data_know,
        FlowData::Icky => &mut f.data_icky,
    };
    assert!(!d.is_empty(), "flow array not allocated");
    d[y as usize][x as usize] = v;
}

/// `(track_door.y[i], track_door.x[i])`, for `i < track_door.size`.
fn track_door_at(st: &BorgState, i: usize) -> (i32, i32) {
    let t = &st.flow.track_door;
    assert!(i < t.size.max(0) as usize, "track_door index past the list");
    (t.y[i], t.x[i])
}

/// `GOAL_DARK` (borg-flow.h)
const GOAL_DARK: i32 = 4;

/// `AUTO_TEMP_MAX` (borg-flow.h): size of the `borg_temp_*` arrays.
const AUTO_TEMP_MAX: i32 = 9000;

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// Determine if a grid is "interesting" (and should be explored)
///
/// A grid is "interesting" if it is a closed door, rubble, hidden treasure,
/// or an "unknown" grid.
/// or a non-perma-wall adjacent to a perma-wall. (GCV)
fn borg_flow_dark_interesting(st: &mut BorgState, y: i32, x: i32) -> bool {
    /* Get the borg_grid */
    let mut ag = *st.cave.grid(x, y);
    let feat = u32::from(ag.feat);

    /* Explore unknown grids */
    if feat == FEAT_NONE {
        return true;
    }

    /* Efficiency -- Ignore "boring" grids */
    if feat < FEAT_SECRET && feat != FEAT_CLOSED {
        return false;
    }

    /* Explore "known treasure" */
    if feat == FEAT_MAGMA_K || feat == FEAT_QUARTZ_K {
        /* Do not dig when confused */
        if st.borg.tr(BI_ISCONFUSED) != 0 {
            return false;
        }

        /* Do not bother if super rich */
        if st.borg.tr(BI_GOLD) >= 100000 {
            return false;
        }

        /* Not when darkened */
        if st.borg.tr(BI_LIGHT) == 0 {
            return false;
        }

        /* don't try to dig if we can't */
        if !crate::borg_flow::borg_can_dig(st, false, ag.feat) {
            return false;
        }

        /* Okay */
        return true;
    }

    /* "Vaults" Explore non perma-walls adjacent to a perma wall */
    if feat == FEAT_GRANITE || feat == FEAT_MAGMA || feat == FEAT_QUARTZ {
        /* Do not attempt when confused */
        if st.borg.tr(BI_ISCONFUSED) != 0 {
            return false;
        }

        /* hack and cheat.  No vaults  on this level */
        if !st.borg.status.vault {
            return false;
        }

        /* make sure we can dig */
        if !crate::borg_flow::borg_can_dig(st, false, ag.feat) {
            return false;
        }

        /* Do not attempt on the edge */
        if x < AUTO_MAX_X - 1 && y < AUTO_MAX_Y - 1 && x > 1 && y > 1 {
            /* scan the adjacent grids */
            for ox in -1..=1 {
                for oy in -1..=1 {
                    /* at least one perm wall next to this, dig it out */
                    if u32::from(st.cave.grid(ox + x, oy + y).feat) == FEAT_PERM {
                        return true;
                    }
                }
            }
        }

        /* not adjacent to a GCV,  Restore Grid */
        ag = *st.cave.grid(x, y);
    }
    let feat = u32::from(ag.feat);

    /* Explore "rubble" */
    if feat == FEAT_RUBBLE && st.borg.tr(BI_ISWEAK) == 0 {
        return true;
    }

    /* Explore "closed doors" */
    if feat == FEAT_CLOSED {
        /* some closed doors leave alone */
        if st.borg.near.breeder {
            /* Did I close this one */
            let mut i = 0;
            while i < i32::from(st.flow.track_door.num) {
                /* mark as icky if I closed this one */
                if track_door_at(st, i as usize) == (y, x) {
                    /* not interesting */
                    return false;
                }
                i += 1;
            }
        }
        /* this door should be ok to open */
        return true;
    }

    /* Ignore other grids */
    false
}

/// The "Do not disarm when clumsy" tests (same order as C).
fn clumsy(st: &BorgState) -> bool {
    let b = &st.borg;
    (b.tr(BI_DISP) < 30 && b.tr(BI_CLEVEL) < 20)
        || (b.tr(BI_DISP) < 45 && b.tr(BI_CLEVEL) < 10)
        || (b.tr(BI_DISM) < 30 && b.tr(BI_CLEVEL) < 20)
        || (b.tr(BI_DISM) < 45 && b.tr(BI_CLEVEL) < 10)
}

/// Determine if a grid is "reachable" (and can be explored)
fn borg_flow_dark_reachable(st: &BorgState, y: i32, x: i32) -> bool {
    /* Scan neighbors */
    for j in 0..8 {
        let y2 = y + i32::from(engine::ddy_ddd(j));
        let x2 = x + i32::from(engine::ddx_ddd(j));

        /* Get the grid */
        let ag = st.cave.grid(x2, y2);

        /* Skip unknown grids (important) */
        if u32::from(ag.feat) == FEAT_NONE {
            continue;
        }

        /* Accept known floor grids */
        if borg_cave_floor_grid(ag) {
            return true;
        }
    }

    /* Failure */
    false
}

/// The "Increase bravery" fear threshold of `borg_flow_direct()` (the same
/// sequence of overriding assignments as C).
fn direct_fear(st: &BorgState) -> i32 {
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
    if borg_timer(st, b.time.level) > 5000 {
        fear = avoidance * 25;
    }
    if b.tr(BI_FOOD) == 0 {
        fear = avoidance * 100;
    }

    /* Normal in town */
    if b.tr(BI_CLEVEL) == 0 {
        fear = avoidance / 10;
    }
    fear
}

/// The "Analyze every grid once" step of `borg_flow_direct()`: on the first
/// visit of a grid, mark it known and, if it is too dangerous, icky.
/// Returns false when the grid is (now) icky and the path must stop.
fn analyze_once(st: &mut BorgState, y: i32, x: i32) -> bool {
    if borg_data_at(st, FlowData::Know, y, x) == 0 {
        /* Mark as known */
        set_borg_data(st, FlowData::Know, y, x, 1);

        /* Get the danger */
        let p = borg_danger(st, y, x, 1, true, false);

        let fear = direct_fear(st);

        /* Avoid dangerous grids (forever) */
        if p > fear {
            /* Mark as icky */
            set_borg_data(st, FlowData::Icky, y, x, 1);

            /* Abort */
            return false;
        }
    }
    true
}

/// Place a "direct path" into the flow array, checking danger
///
/// Modify the "cost" array in such a way that from any point on
/// one "direct" path from the player to the given grid, as long
/// as the rest of the path is "safe" and "clear", the Borg will
/// walk along the path to the given grid.
///
/// This function is used by "borg_flow_dark_1()" to provide an
/// optimized "flow" during the initial exploration of a level.
/// It is also used by "borg_flow_dark_2()" in a similar fashion.
fn borg_flow_direct(st: &mut BorgState, mut y: i32, mut x: i32) {
    let mut n: i32 = 0;

    /* Avoid icky grids */
    if borg_data_at(st, FlowData::Icky, y, x) != 0 {
        return;
    }

    /* Unknown: mark as known, dangerous grids as icky */
    if !analyze_once(st, y, x) {
        return;
    }

    /* Save the flow cost (zero) */
    set_borg_data(st, FlowData::Cost, y, x, 0);

    /* Save "origin" */
    let y1 = y;
    let x1 = x;

    /* Save "destination" */
    let y2 = st.borg.c.y;
    let x2 = st.borg.c.x;

    /* Calculate distance components */
    let ay = if y2 < y1 { y1 - y2 } else { y2 - y1 };
    let ax = if x2 < x1 { x1 - x2 } else { x2 - x1 };

    /* Path */
    loop {
        /* Check for arrival at player */
        if x == x2 && y == y2 {
            return;
        }

        /* Next */
        n += 1;

        if ay > ax {
            /* Move mostly vertically */

            /* Extract a shift factor XXX */
            let shift = (n * ax + (ay - 1) / 2) / ay;

            /* Sometimes move along the minor axis */
            x = if x2 < x1 { x1 - shift } else { x1 + shift };

            /* Always move along major axis */
            y = if y2 < y1 { y1 - n } else { y1 + n };
        } else {
            /* Move mostly horizontally */

            /* Extract a shift factor XXX */
            let shift = (n * ay + (ax - 1) / 2) / ax;

            /* Sometimes move along the minor axis */
            y = if y2 < y1 { y1 - shift } else { y1 + shift };

            /* Always move along major axis */
            x = if x2 < x1 { x1 - n } else { x1 + n };
        }

        /* Stay on the map */
        if !(x >= 0 && y >= 0 && x < AUTO_MAX_X && y < AUTO_MAX_Y) {
            return;
        }

        /* Access the grid */
        let ag = *st.cave.grid(x, y);

        /* Ignore "wall" grids and lava (unless immune to fire) */
        if !borg_cave_floor_grid(&ag)
            || (u32::from(ag.feat) == FEAT_LAVA && st.borg.tr(BI_IFIRE) == 0)
        {
            return;
        }

        /* Avoid Traps if low level-- unless brave or scaryguy. */
        if ag.trap && i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP) && !st.borg.mon.scary {
            /* Do not disarm when you could end up dead */
            if st.borg.tr(BI_CURHP) < 60 {
                return;
            }

            /* Do not disarm when clumsy */
            if clumsy(st) {
                return;
            }
        }

        /* Abort at "icky" grids */
        if borg_data_at(st, FlowData::Icky, y, x) != 0 {
            return;
        }

        /* Analyze every grid once */
        if !analyze_once(st, y, x) {
            return;
        }

        /* Abort "pointless" paths if possible */
        if i32::from(borg_data_at(st, FlowData::Cost, y, x)) <= n {
            break;
        }

        /* Save the new flow cost (uint8_t, truncated as in C) */
        set_borg_data(st, FlowData::Cost, y, x, n as u8);
    }
}

/// Mark off the edges of a rectangle as "avoid" or "clear"
fn borg_flow_border(st: &mut BorgState, y1: i32, x1: i32, y2: i32, x2: i32, stop: bool) {
    let v = u8::from(stop);

    /* Scan west/east edges */
    for y in y1..=y2 {
        /* Avoid/Clear west edge */
        set_borg_data(st, FlowData::Know, y, x1, v);
        set_borg_data(st, FlowData::Icky, y, x1, v);

        /* Avoid/Clear east edge */
        set_borg_data(st, FlowData::Know, y, x2, v);
        set_borg_data(st, FlowData::Icky, y, x2, v);
    }

    /* Scan north/south edges */
    for x in x1..=x2 {
        /* Avoid/Clear north edge */
        set_borg_data(st, FlowData::Know, y1, x, v);
        set_borg_data(st, FlowData::Icky, y1, x, v);

        /* Avoid/Clear south edge */
        set_borg_data(st, FlowData::Know, y2, x, v);
        set_borg_data(st, FlowData::Icky, y2, x, v);
    }
}

/// "Wipe icky codes from grids if needed", then "Clear the flow codes"
/// (the common start of every method once grids were found).
fn wipe_and_clear(st: &mut BorgState) {
    /* Wipe icky codes from grids if needed */
    if st.borg.goal.ignoring || st.borg.mon.scary {
        st.danger.wipe = true;
    }

    /* Clear the flow codes */
    crate::borg_flow::borg_flow_clear(st);
}

/// Enqueue every grid of the `borg_temp_*` list, in list order.
fn enqueue_temp(st: &mut BorgState) {
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);

        /* Enqueue the grid */
        crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
        i += 1;
    }
}

/// Create a direct path to every grid of the `borg_temp_*` list, in list
/// order.
fn direct_temp(st: &mut BorgState) {
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);

        /* Create a path */
        borg_flow_direct(st, y, x);
        i += 1;
    }
}

/// Commit the flow and take one step (the common end of every method).
fn commit_and_step(st: &mut BorgState, who: &CStr) -> bool {
    /* Attempt to Commit the flow */
    if !crate::borg_flow::borg_flow_commit(st, who, GOAL_DARK) {
        return false;
    }

    /* Take one step */
    if !crate::borg_flow::borg_flow_old(st, GOAL_DARK) {
        return false;
    }

    /* Success */
    true
}

/// The local region `borg.c +- r`, restricted to "legal" grids:
/// `(y1, x1, y2, x2)`.
fn local_region(st: &BorgState, r: i32) -> (i32, i32, i32, i32) {
    let mut y1 = st.borg.c.y - r;
    let mut x1 = st.borg.c.x - r;
    let mut y2 = st.borg.c.y + r;
    let mut x2 = st.borg.c.x + r;

    /* Restrict to "legal" grids */
    if y1 < 1 {
        y1 = 1;
    }
    if x1 < 1 {
        x1 = 1;
    }
    if y2 > AUTO_MAX_Y - 2 {
        y2 = AUTO_MAX_Y - 2;
    }
    if x2 > AUTO_MAX_X - 2 {
        x2 = AUTO_MAX_X - 2;
    }
    (y1, x1, y2, x2)
}

/// Prepare to "flow" towards "interesting" grids (method 1)
///
/// This function examines the torch-lit grids for "interesting" grids.
fn borg_flow_dark_1(st: &mut BorgState, b_stair: i32, scum: Option<&Scum>) -> bool {
    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Reset */
    st.flow.temp_n = 0;

    /* Scan torch-lit grids */
    let mut i = 0;
    while i < i32::from(st.light.light_n) {
        let y = i32::from(st.light.light_y[i as usize]);
        let x = i32::from(st.light.light_x[i as usize]);
        i += 1;

        /* Skip "boring" grids (assume reachable) */
        if !borg_flow_dark_interesting(st, y, x) {
            continue;
        }

        /* don't go too far from the stairs */
        if borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair) {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    // borg-reborn-rs (#1061): stair_scum keeps the safe ones nearest the stairs.
    if let Some(m) = scum {
        crate::borg_stair_scum::filter_dark(st, m);
    }

    /* Nothing */
    if st.flow.temp_n == 0 {
        return false;
    }

    wipe_and_clear(st);

    /* Create paths to useful grids */
    direct_temp(st);

    commit_and_step(st, c"dark-1")
}

/// Prepare to "flow" towards "interesting" grids (method 2)
///
/// This function is only used when the player is at least 4 grids away
/// from the outer dungeon wall, to prevent any nasty memory errors.
///
/// This function examines the grids just outside the torch-lit grids
/// for "unknown" grids, and flows directly towards them (one step).
fn borg_flow_dark_2(st: &mut BorgState, b_stair: i32, scum: Option<&Scum>) -> bool {
    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Maximal radius */
    /* borg-reborn-rs (#884): just outside the Borg's own light radius
     * (BI_LIGHT, C's, at cave_light_engine_radius 0; BI_LIGHT - 1 at 1) */
    let r = crate::borg_cave_light::own_light_radius(st) + 1;

    /* Reset */
    st.flow.temp_n = 0;

    /* Four directions */
    for i in 0..4 {
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i)) * r;
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i)) * r;

        /* Check legality */
        if y < 1 || x < 1 || y > AUTO_MAX_Y - 2 || x > AUTO_MAX_X - 2 {
            continue;
        }

        /* Acquire grid */
        let ag = *st.cave.grid(x, y);

        /* Require unknown */
        if u32::from(ag.feat) != FEAT_NONE {
            continue;
        }

        /* Require viewable */
        if ag.info & BORG_VIEW == 0 {
            continue;
        }

        /* don't go too far from the stairs */
        if borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair) {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    // borg-reborn-rs (#1061): stair_scum keeps the safe ones nearest the stairs.
    if let Some(m) = scum {
        crate::borg_stair_scum::filter_dark(st, m);
    }

    /* Nothing */
    if st.flow.temp_n == 0 {
        return false;
    }

    wipe_and_clear(st);

    /* Create paths to useful grids */
    direct_temp(st);

    commit_and_step(st, c"dark-2")
}

/// How the region scans of methods 3-5 test the distance to the stairs.
#[derive(Clone, Copy)]
enum StairTest {
    /// `borg_flow_far_from_stairs(x, y, b_stair)`
    Leash,
    /// `borg_flow_far_from_stairs_dist(x, y, b_stair, leash)`
    Dist(i32),
}

/// Scan the region `(y1, x1)..=(y2, x2)` row by row for interesting,
/// reachable grids not too far from the stairs, into the `borg_temp_*`
/// list. With `stop_when_full`, stop once the list holds `AUTO_TEMP_MAX`
/// grids (method 5's paranoia check).
fn scan_region(
    st: &mut BorgState,
    (y1, x1, y2, x2): (i32, i32, i32, i32),
    b_stair: i32,
    test: StairTest,
    stop_when_full: bool,
) {
    for y in y1..=y2 {
        for x in x1..=x2 {
            /* Skip "boring" grids */
            if !borg_flow_dark_interesting(st, y, x) {
                continue;
            }

            /* Skip "unreachable" grids */
            if !borg_flow_dark_reachable(st, y, x) {
                continue;
            }

            /* don't go too far from the stairs */
            let far = match test {
                StairTest::Leash => borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair),
                StairTest::Dist(leash) => {
                    borg_flow_misc::borg_flow_far_from_stairs_dist(st, x, y, b_stair, leash)
                }
            };
            if far {
                continue;
            }

            /* Careful -- Remember it */
            st.flow.temp_push(y, x);

            /* Paranoia -- Check for overflow (double break) */
            if stop_when_full && i32::from(st.flow.temp_n) == AUTO_TEMP_MAX {
                return;
            }
        }
    }
}

/// Prepare to "flow" towards "interesting" grids (method 3)
///
/// Note the use of a limit on the "depth" of the flow, and of the flag
/// which avoids "unknown" grids when calculating the flow, both of which
/// help optimize this function to only handle "easily reachable" grids.
///
/// The "borg_temp" array is much larger than any "local region".
fn borg_flow_dark_3(st: &mut BorgState, b_stair: i32, scum: Option<&Scum>) -> bool {
    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Local region */
    let region = local_region(st, 4);

    /* Reset */
    st.flow.temp_n = 0;

    /* Examine the region */
    scan_region(st, region, b_stair, StairTest::Leash, false);

    // borg-reborn-rs (#1061): stair_scum keeps the safe ones nearest the stairs.
    if let Some(m) = scum {
        crate::borg_stair_scum::filter_dark(st, m);
    }

    /* Nothing interesting */
    if st.flow.temp_n == 0 {
        return false;
    }

    wipe_and_clear(st);

    /* Enqueue useful grids */
    enqueue_temp(st);

    /* Spread the flow (limit depth) */
    crate::borg_flow::borg_flow_spread(st, 5, false, true, false, -1, false);

    commit_and_step(st, c"dark-3")
}

/// Prepare to "flow" towards "interesting" grids (method 4)
///
/// Note that we avoid grids close to the edge of the panel, since they
/// induce panel scrolling, which is "expensive" in terms of CPU usage,
/// and because this allows us to "expand" the border by several grids
/// to lay down the "avoidance" border in known legal grids.
///
/// We avoid paths that would take us into different panels by setting
/// the "icky" flag for the "border" grids to prevent path construction,
/// and then clearing them when done, to prevent confusion elsewhere.
///
/// The "borg_temp" array is large enough to hold one panel full of grids.
fn borg_flow_dark_4(st: &mut BorgState, b_stair: i32, scum: Option<&Scum>) -> bool {
    let leash = borg_flow_misc::borg_get_leash(st, false);

    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Not if a vault is on the level */
    if st.borg.status.vault {
        return false;
    }

    /* Local region */
    let (y1, x1, y2, x2) = local_region(st, 11);

    /* Nothing yet */
    st.flow.temp_n = 0;

    /* Examine the panel */
    scan_region(st, (y1, x1, y2, x2), b_stair, StairTest::Dist(leash), false);

    // borg-reborn-rs (#1061): stair_scum keeps the safe ones nearest the stairs.
    if let Some(m) = scum {
        crate::borg_stair_scum::filter_dark(st, m);
    }

    /* Nothing useful */
    if st.flow.temp_n == 0 {
        return false;
    }

    wipe_and_clear(st);

    /* Enqueue useful grids */
    enqueue_temp(st);

    /* Expand borders */
    let (y1, x1, y2, x2) = (y1 - 1, x1 - 1, y2 + 1, x2 + 1);

    /* Avoid the edges */
    borg_flow_border(st, y1, x1, y2, x2, true);

    /* Spread the flow (limit depth Leash) */
    if st.borg.tr(BI_CLEVEL) < 15 {
        /* Short Leash */
        crate::borg_flow::borg_flow_spread(st, leash, true, true, false, -1, false);
    } else {
        /* Long Leash */
        crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);
    }

    /* Clear the edges (C bug 31: also forgets marks set before) */
    borg_flow_border(st, y1, x1, y2, x2, false);

    commit_and_step(st, c"dark-4")
}

/// Prepare to "flow" towards "interesting" grids (method 5)
fn borg_flow_dark_5(st: &mut BorgState, b_stair: i32, scum: Option<&Scum>) -> bool {
    let leash = borg_flow_misc::borg_get_leash(st, false);

    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Nothing yet */
    st.flow.temp_n = 0;

    /* Examine every "legal" grid */
    let region = (1, 1, AUTO_MAX_Y - 2, AUTO_MAX_X - 2);
    scan_region(st, region, b_stair, StairTest::Dist(leash), true);

    // borg-reborn-rs (#1061): stair_scum keeps the safe ones nearest the stairs.
    if let Some(m) = scum {
        crate::borg_stair_scum::filter_dark(st, m);
    }

    /* Nothing useful */
    if st.flow.temp_n == 0 {
        return false;
    }

    wipe_and_clear(st);

    /* Enqueue useful grids */
    enqueue_temp(st);

    /* Spread the flow */
    /* (C has two identical "Short Leash" branches, for clevel <= 5 and
     * clevel <= 30; the tests have no side effects) */
    let brave = i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP);
    if (st.borg.tr(BI_CLEVEL) <= 5 && brave) || (st.borg.tr(BI_CLEVEL) <= 30 && brave) {
        /* Short Leash */
        crate::borg_flow::borg_flow_spread(st, leash, true, true, false, -1, false);
    } else {
        /* Long Leash */
        crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);
    }

    commit_and_step(st, c"dark-5")
}

/// borg-reborn-rs (#1080, #1085; owner 2026-11-23: "a safer way would be
/// breadth-first"): is exploration breadth-first from the staircases?
/// `explore_breadth_first` on, in the dungeon, and the Borg's maximum HP
/// below `bfs_hp_threshold` (owner, 2026-11-23: by level and class).
pub(crate) fn breadth_first_active(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) > 0 && breadth_first_applies(st)
}

/// #1110: would breadth-first exploration apply to this character in the
/// dungeon (the switch on, maximum HP below its threshold)? The town planner
/// asks this in town.
pub(crate) fn breadth_first_applies(st: &BorgState) -> bool {
    value(Param::explore_breadth_first) != 0
        && st.borg.tr(BI_MAXHP)
            < bfs_hp_threshold(
                value(Param::explore_bfs_max_hp),
                value(Param::explore_bfs_hp_per_level),
                st.borg.tr(BI_CLEVEL),
            )
}

/// #1110 (owner, 2026-11-23: "plan a little more food and light if we do
/// that"): breadth-first exploration walks back toward the stairs between
/// targets, so a trip takes more turns. While it would apply, the town
/// planner's food target and light goal are this percent of today's
/// (`explore_bfs_food_percent`, `explore_bfs_light_percent`); else 100.
pub(crate) fn bfs_supply_percent(st: &BorgState, light: bool) -> i32 {
    if !breadth_first_applies(st) {
        return 100;
    }
    value(if light { Param::explore_bfs_light_percent } else { Param::explore_bfs_food_percent })
}

/// #1085: the maximum HP below which breadth-first applies at character
/// level `clevel`: `explore_bfs_max_hp + explore_bfs_hp_per_level x CL`,
/// both values per class. A level gains `randint1(hitdie)` HP
/// (`player-birth.c:295`), so the delta sets how many levels it lasts.
fn bfs_hp_threshold(max_hp: i32, per_level: i32, clevel: i32) -> i32 {
    max_hp.saturating_add(per_level.saturating_mul(clevel))
}

/// #1085 (review of #1088): a staircase newly seen changes every ring, so
/// a committed exploration flow is dropped and the next think plans again
/// (`borg_update`, when a staircase joins `st.stairs`).
pub(crate) fn replan_on_new_stair(st: &mut BorgState) {
    replan(st, "New staircase");
}

/// #1085 (re-review of #1088): a monster newly tracked, in view or not,
/// drops a committed exploration flow too (`borg_flow_kill`; C clears the
/// goal only for one in line of sight with a light).
pub(crate) fn replan_on_new_monster(st: &mut BorgState) {
    replan(st, "New monster");
}

fn replan(st: &mut BorgState, why: &str) {
    if breadth_first_active(st) && i32::from(st.borg.goal.type_0) == GOAL_DARK {
        crate::borg_io::borg_note(st, &format!("# {why}: planning the breadth-first exploration again (#1085)"));
        st.borg.goal.type_0 = 0;
    }
}

/// #1085 (re-review of #1088): while the Borg moves under boosted bravery
/// (its avoidance above its HP: stuck recovery), the flow crosses unknown
/// grids and known traps, so a Borg cut off from the staircases would look
/// connected. Exploration is C's then.
fn boosted_bravery(avoidance: i32, curhp: i32) -> bool {
    avoidance > curhp
}

/// #1085 (re-review of #1088): the candidates the Borg can reach (`reach`,
/// a spread from the Borg, read as `ring_of` reads the rings). The cap on
/// rings applies after this, so rings the Borg cannot reach (another region
/// with its own staircase) do not use up the tries.
fn reachable_only<R: AsRef<[u8]>>(cands: &mut Vec<(i32, i32, i32)>, reach: &[R]) {
    cands.retain(|&(y, x, _)| ring_of(reach, y, x).is_some());
}

/// No stair distance (`borg_data_hard`'s value, unreached by the spread).
const NO_RING: u8 = 255;

/// The eight neighbours (the pure helpers below do not call the engine).
const NEIGHBOURS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

/// #1080: the ring of a grid: its flow steps from the nearest known
/// staircase over known grids (`rings`, the spread from every staircase),
/// or, for a grid the spread does not enter (an unknown grid, rubble, a
/// vein, a known trap below 60 HP), one more than its nearest entered
/// neighbour. `None`: no route.
fn ring_of<R: AsRef<[u8]>>(rings: &[R], y: i32, x: i32) -> Option<i32> {
    let at = |y: i32, x: i32| -> i32 {
        rings.get(y as usize).and_then(|row| row.as_ref().get(x as usize)).map_or(i32::from(NO_RING), |&c| i32::from(c))
    };
    let mut best = at(y, x);
    for (dy, dx) in NEIGHBOURS {
        if y + dy >= 0 && x + dx >= 0 {
            best = best.min(at(y + dy, x + dx) + 1);
        }
    }
    (best < i32::from(NO_RING)).then_some(best)
}

/// #1080: the farthest ring a path to a goal of ring at most `d` may cross:
/// the goal's own, or the Borg's when it stands farther out (its way back
/// passes there). A grid beyond it is behind the Borg's back.
fn path_limit(d: i32, borg_ring: i32) -> i32 {
    d.max(borg_ring)
}

/// #1080: is a known grid of ring `ring` closed to a path whose limit is
/// `limit`?
fn beyond_limit(ring: u8, limit: i32) -> bool {
    ring != NO_RING && i32::from(ring) > limit
}

/// #1085: one try of the breadth-first plan: the goals (every candidate
/// from the base ring to the base ring + `explore_bfs_slack`) and the path
/// limit their spread keeps to.
#[derive(Debug, PartialEq)]
struct RingTry {
    base: i32,
    limit: i32,
    goals: Vec<(i32, i32)>,
}

/// #1085: what the breadth-first order does this think.
#[derive(Debug, PartialEq)]
enum Plan {
    /// The Borg has no route to any known staircase over known grids (none
    /// known, a teleport, a known trap across the way back, #1067): C's
    /// methods explore, as before (review of #1088).
    Fallback,
    /// The rings to try, nearest first (empty: nothing within the leash).
    Rings(Vec<RingTry>),
}

/// #1085: the plan from the candidates `(y, x, ring)`, the Borg's own ring,
/// the leash (`None`: none applies), the slack and the number of tries.
fn plan(cands: &[(i32, i32, i32)], borg_ring: Option<i32>, leash: Option<i32>, slack: i32, max: usize) -> Plan {
    let Some(borg_ring) = borg_ring else { return Plan::Fallback };
    let within: Vec<(i32, i32, i32)> = cands.iter().copied().filter(|&(_, _, r)| leash.is_none_or(|l| r <= l)).collect();
    let mut bases: Vec<i32> = within.iter().map(|&(_, _, r)| r).collect();
    bases.sort_unstable();
    bases.dedup();
    bases.truncate(max);
    Plan::Rings(
        bases
            .into_iter()
            .map(|base| RingTry {
                base,
                limit: path_limit(base + slack, borg_ring),
                goals: within.iter().filter(|&&(_, _, r)| r >= base && r <= base + slack).map(|&(y, x, _)| (y, x)).collect(),
            })
            .collect(),
    )
}

/// At most this many rings are tried in one think (one spread each).
const BREADTH_FIRST_MAX_RINGS: usize = 16;

/// #1080, #1085: breadth-first exploration from the staircases.
///
/// 1. One spread from every known staircase, of either kind, over known
///    grids gives each grid its ring (`ring_of`).
/// 2. The candidates are the interesting grids of the whole known map
///    (methods 3-5's, reachable, with method 1's torch-lit ones), within the
///    leash measured from that nearest staircase (where C's distance test
///    applies, `borg_flow_far_from_stairs_dist`), and within the return
///    leash of `borg_think_dungeon` while it applies.
/// 3. The nearest ring is explored first (`plan`): every candidate from it
///    to it + `explore_bfs_slack` is a goal, so the flow leads to the one
///    nearest the Borg (the tie-break). A path may not cross a grid of a
///    farther ring than that (or than the Borg's own, `path_limit`), nor one
///    past the return leash. When no path exists, the next ring.
///
/// `Some(stepped)`: this order decided (`false`: nothing to explore within
/// the leashes); `None`: the Borg has no route to a known staircase.
fn borg_flow_dark_breadth_first(st: &mut BorgState, scum: Option<&Scum>) -> Option<bool> {
    if st.stairs.less.num == 0 && st.stairs.more.num == 0 {
        return None;
    }
    if boosted_bravery(i32::from(st.borg.avoidance), st.borg.tr(BI_CURHP)) {
        return None;
    }

    /* Rings: one spread from every known staircase */
    wipe_and_clear(st);
    for i in 0..i32::from(st.stairs.less.num) {
        let (y, x) = st.stairs.less.at(i as usize);
        crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
    }
    for i in 0..i32::from(st.stairs.more.num) {
        let (y, x) = st.stairs.more.at(i as usize);
        crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
    }
    crate::borg_flow::borg_flow_spread_stairs(st, 250, false, true, false, -1, false);
    let rings = st.flow.data_cost.clone();

    /* No route from the Borg to a staircase: C's methods */
    let borg_ring = ring_of(&rings, st.borg.c.y, st.borg.c.x);
    if borg_ring.is_none() {
        return None;
    }

    /* The leash, where C's stair distance test applies */
    let c = crate::borg_flow::c_leash();
    let leash_on = (st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - c.depth_margin
        && st.borg.tr(BI_CLEVEL) < c.stairs_max_level)
        || crate::borg_kit::leash_active(st);
    let leash = leash_on.then(|| borg_flow_misc::borg_get_leash(st, false));

    /* The return leash ("wandered too far", from the nearest up staircase):
     * no target or path past it, or the Borg would walk out and back */
    let ret = crate::borg_think_dungeon::return_leash(st);
    let mut past_return = vec![vec![false; AUTO_MAX_X as usize]; AUTO_MAX_Y as usize];
    if let Some(r) = ret {
        for (y, row) in past_return.iter_mut().enumerate() {
            for (x, past) in row.iter_mut().enumerate() {
                let at = loc { x: x as i32, y: y as i32 };
                *past = crate::borg_think_dungeon::up_stair_distance(st, at).is_some_and(|d| d > r);
            }
        }
    }

    /* Candidates: torch-lit interesting grids, then every reachable one */
    let mut seen = vec![vec![false; AUTO_MAX_X as usize]; AUTO_MAX_Y as usize];
    let mut cands: Vec<(i32, i32, i32)> = Vec::new();
    let lit: Vec<(i32, i32)> = (0..st.light.light_n.max(0) as usize)
        .map(|i| (i32::from(st.light.light_y[i]), i32::from(st.light.light_x[i])))
        .collect();
    let mut consider = |st: &mut BorgState, y: i32, x: i32, lit: bool| {
        if seen[y as usize][x as usize] {
            return;
        }
        if !borg_flow_dark_interesting(st, y, x) || (!lit && !borg_flow_dark_reachable(st, y, x)) {
            return;
        }
        // borg-reborn-rs (#1061): with stair_scum, only targets in its safe area.
        if scum.is_some_and(|m| !m.admits_target(st, y, x)) {
            return;
        }
        seen[y as usize][x as usize] = true;
        if let Some(r) = ring_of(&rings, y, x) {
            if !past_return[y as usize][x as usize] {
                cands.push((y, x, r));
            }
        }
    };
    for (y, x) in lit {
        consider(st, y, x, true);
    }
    for y in 1..=AUTO_MAX_Y - 2 {
        for x in 1..=AUTO_MAX_X - 2 {
            consider(st, y, x, false);
        }
    }

    /* Only the candidates the Borg can reach (one spread from the Borg),
     * through no grid past the return leash: the route planner below closes
     * those too (second re-review of #1088) */
    crate::borg_flow::borg_flow_clear(st);
    crate::borg_flow::borg_flow_enqueue_grid(st, st.borg.c.y, st.borg.c.x);
    if st.flow.cost_at(st.borg.c.y, st.borg.c.x) == 0 {
        let mut closed = Vec::new();
        for (y, row) in past_return.iter().enumerate() {
            for (x, &past) in row.iter().enumerate() {
                if past && (y as i32, x as i32) != (st.borg.c.y, st.borg.c.x) {
                    st.flow.data_cost[y][x] = 0;
                    closed.push((y, x));
                }
            }
        }
        crate::borg_flow::borg_flow_spread(st, 250, false, true, false, -1, false);
        let mut reach = st.flow.data_cost.clone();
        for (y, x) in closed {
            st.flow.data_cost[y][x] = st.flow.data_hard[y][x];
            reach[y][x] = NO_RING;
        }
        reachable_only(&mut cands, &reach);
    }

    // borg-reborn-rs (#1061): with stair_scum, back to the staircase first
    // when the nearest target lies farther out than the Borg.
    if let Some(m) = scum {
        let goals: Vec<(i32, i32)> = cands.iter().map(|&(y, x, _)| (y, x)).collect();
        if crate::borg_stair_scum::return_before(st, m, &goals) {
            return Some(false);
        }
    }

    let slack = value(Param::explore_bfs_slack);
    let Plan::Rings(tries) = plan(&cands, borg_ring, leash, slack, BREADTH_FIRST_MAX_RINGS) else {
        return None;
    };
    for t in tries {
        crate::borg_flow::borg_flow_clear(st);

        /* Close the grids behind the Borg's back (a zero cost is "reached") */
        let mut closed = Vec::new();
        for (y, row) in rings.iter().enumerate() {
            for (x, &r) in row.iter().enumerate() {
                if (beyond_limit(r, t.limit) || past_return[y][x]) && (y as i32, x as i32) != (st.borg.c.y, st.borg.c.x) {
                    st.flow.data_cost[y][x] = 0;
                    closed.push((y, x));
                }
            }
        }

        /* Every goal of the ring is a starting grid */
        for &(y, x) in &t.goals {
            crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
        }
        crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);

        /* Reopen the closed grids (no goal of their own) */
        for (y, x) in closed {
            st.flow.data_cost[y][x] = st.flow.data_hard[y][x];
        }

        let who = std::ffi::CString::new(format!("dark-bfs (ring {})", t.base)).expect("no NUL");
        if commit_and_step(st, &who) {
            return Some(true);
        }
    }
    Some(false)
}

/// Prepare to "flow" towards "interesting" grids
///
/// The "exploration" routines are broken into "near" and "far"
/// exploration, and each set is chosen via the flag below.
pub fn borg_flow_dark(st: &mut BorgState, neer: bool) -> bool {
    let mut b_j = -1;
    let mut b_stair = -1;

    /* Not if sitting in a sea of runes and we saw Morgoth recently */
    if st.borg.morgoth_position && st.borg.near.morgoth {
        return false;
    }

    /* Paranoia */
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    if borg_flow_dark_interesting(st, cy, cx) {
        return false;
    }

    /* Check distance away from stairs, used later */
    /* Check for an existing "up stairs" (C bug 31: keeps the farthest) */
    let mut i = 0;
    while i < i32::from(st.stairs.less.num) {
        let (y, x) = st.stairs.less.at(i as usize);

        /* How far is the nearest up stairs */
        let j = engine::distance(st.borg.c, loc { x, y });

        /* skip the closer ones */
        if b_j >= j {
            i += 1;
            continue;
        }

        /* track it */
        b_j = j;
        b_stair = i;
        i += 1;
    }

    // borg-reborn-rs (#1061): stair_scum's map, measured once (None: off).
    let scum = crate::borg_stair_scum::Map::of(st);

    // borg-reborn-rs (#1080, #1085): below explore_bfs_max_hp, both calls
    // explore breadth-first from the staircases (`None`: the Borg has no
    // route to a known staircase; C's methods then run as before).
    if breadth_first_active(st) {
        if let Some(stepped) = borg_flow_dark_breadth_first(st, scum.as_ref()) {
            return stepped;
        }
    }

    if neer {
        /* Near */

        /* Method 1 */
        if borg_flow_dark_1(st, b_stair, scum.as_ref()) {
            return true;
        }

        /* Method 2 */
        if borg_flow_dark_2(st, b_stair, scum.as_ref()) {
            return true;
        }

        /* Method 3 */
        if borg_flow_dark_3(st, b_stair, scum.as_ref()) {
            return true;
        }
    } else {
        /* Far */

        /* Method 4 */
        if borg_flow_dark_4(st, b_stair, scum.as_ref()) {
            return true;
        }

        /* Method 5 */
        if borg_flow_dark_5(st, b_stair, scum.as_ref()) {
            return true;
        }
    }

    /* Fail */
    false
}

#[cfg(test)]
mod breadth_first_tests {
    use super::*;

    /// A synthetic map: '#' wall, '?' unknown, '^' a known trap (a wall to
    /// the flow below 60 HP), '<' or '>' a staircase, '@' the Borg, '.'
    /// known floor, 'P' known floor past the return leash (the rings cross
    /// it; the reach and the route do not).
    struct Map {
        g: Vec<Vec<u8>>,
        borg: (i32, i32),
    }

    impl Map {
        fn new(rows: &[&str]) -> Map {
            let g: Vec<Vec<u8>> = rows.iter().map(|r| r.as_bytes().to_vec()).collect();
            let mut borg = (-1, -1);
            for (y, row) in g.iter().enumerate() {
                for (x, &c) in row.iter().enumerate() {
                    if c == b'@' {
                        borg = (y as i32, x as i32);
                    }
                }
            }
            Map { g, borg }
        }

        fn at(&self, y: i32, x: i32) -> u8 {
            if y < 0 || x < 0 {
                return b'#';
            }
            self.g.get(y as usize).and_then(|r| r.get(x as usize)).copied().unwrap_or(b'#')
        }

        fn open(&self, y: i32, x: i32) -> bool {
            matches!(self.at(y, x), b'.' | b'<' | b'>' | b'@' | b'P')
        }

        /// Open to the reach and the route: not past the return leash.
        fn walkable(&self, y: i32, x: i32) -> bool {
            self.open(y, x) && self.at(y, x) != b'P'
        }

        /// The reach, as the spread from the Borg gives it (past-leash grids
        /// closed, as in the game).
        fn reach(&self) -> Vec<Vec<u8>> {
            let mut r = vec![vec![NO_RING; self.g[0].len()]; self.g.len()];
            r[self.borg.0 as usize][self.borg.1 as usize] = 0;
            let mut queue = std::collections::VecDeque::from([self.borg]);
            while let Some((y, x)) = queue.pop_front() {
                for (dy, dx) in NEIGHBOURS {
                    let (ny, nx) = (y + dy, x + dx);
                    if self.walkable(ny, nx) && r[ny as usize][nx as usize] == NO_RING {
                        r[ny as usize][nx as usize] = r[y as usize][x as usize] + 1;
                        queue.push_back((ny, nx));
                    }
                }
            }
            r
        }

        /// The rings, as the spread from every staircase gives them.
        fn rings(&self) -> Vec<Vec<u8>> {
            let mut r = vec![vec![NO_RING; self.g[0].len()]; self.g.len()];
            let mut queue = std::collections::VecDeque::new();
            for (y, row) in self.g.iter().enumerate() {
                for (x, &c) in row.iter().enumerate() {
                    if matches!(c, b'<' | b'>') {
                        r[y][x] = 0;
                        queue.push_back((y as i32, x as i32));
                    }
                }
            }
            while let Some((y, x)) = queue.pop_front() {
                for (dy, dx) in NEIGHBOURS {
                    let (ny, nx) = (y + dy, x + dx);
                    if self.open(ny, nx) && r[ny as usize][nx as usize] == NO_RING {
                        r[ny as usize][nx as usize] = r[y as usize][x as usize] + 1;
                        queue.push_back((ny, nx));
                    }
                }
            }
            r
        }

        /// The candidates: unknown grids next to known floor, with a ring.
        fn cands(&self, rings: &[Vec<u8>]) -> Vec<(i32, i32, i32)> {
            let mut c = Vec::new();
            for (y, row) in self.g.iter().enumerate() {
                for (x, &ch) in row.iter().enumerate() {
                    let (y, x) = (y as i32, x as i32);
                    if ch == b'?' && NEIGHBOURS.iter().any(|&(dy, dx)| self.open(y + dy, x + dx)) {
                        if let Some(r) = ring_of(rings, y, x) {
                            c.push((y, x, r));
                        }
                    }
                }
            }
            c
        }

        /// The goal the flow leads to: the one nearest the Borg over open
        /// grids within the limit (ties: the first in the list).
        fn nearest_goal(&self, rings: &[Vec<u8>], t: &RingTry) -> Option<(i32, i32)> {
            let mut dist = vec![vec![i32::MAX; self.g[0].len()]; self.g.len()];
            let mut queue = std::collections::VecDeque::from([self.borg]);
            dist[self.borg.0 as usize][self.borg.1 as usize] = 0;
            while let Some((y, x)) = queue.pop_front() {
                for (dy, dx) in NEIGHBOURS {
                    let (ny, nx) = (y + dy, x + dx);
                    if self.walkable(ny, nx)
                        && !beyond_limit(rings[ny as usize][nx as usize], t.limit)
                        && dist[ny as usize][nx as usize] == i32::MAX
                    {
                        dist[ny as usize][nx as usize] = dist[y as usize][x as usize] + 1;
                        queue.push_back((ny, nx));
                    }
                }
            }
            let to = |&(y, x): &(i32, i32)| {
                NEIGHBOURS
                    .iter()
                    .filter_map(|&(dy, dx)| dist.get((y + dy) as usize).and_then(|r| r.get((x + dx) as usize)).copied())
                    .filter(|&d| d != i32::MAX)
                    .min()
                    .map(|d| d + 1)
            };
            t.goals.iter().filter_map(|g| to(g).map(|d| (d, *g))).min_by_key(|&(d, _)| d).map(|(_, g)| g)
        }

        /// Explore until nothing is left: each step the plan's first ring
        /// with a reachable goal, the Borg walks there and the grid becomes
        /// floor. Returns the rings explored, in order (`None`: the plan
        /// fell back to C's methods).
        fn explore(&mut self, leash: Option<i32>, slack: i32) -> Option<Vec<i32>> {
            let mut order = Vec::new();
            loop {
                let rings = self.rings();
                let cands = self.cands(&rings);
                let Plan::Rings(tries) =
                    plan(&cands, ring_of(&rings, self.borg.0, self.borg.1), leash, slack, BREADTH_FIRST_MAX_RINGS)
                else {
                    return None;
                };
                let Some((ring, (y, x))) =
                    tries.iter().find_map(|t| self.nearest_goal(&rings, t).map(|g| (ring_of(&rings, g.0, g.1).unwrap(), g)))
                else {
                    return Some(order);
                };
                order.push(ring);
                let (by, bx) = self.borg;
                if self.g[by as usize][bx as usize] == b'@' {
                    self.g[by as usize][bx as usize] = b'.';
                }
                self.g[y as usize][x as usize] = b'.';
                self.borg = (y, x);
            }
        }
    }

    #[test]
    fn two_corridors_are_explored_to_depth_three_before_either_to_depth_four() {
        // A staircase with a corridor east and one west, each 6 unknown
        // grids long; the Borg starts on the staircase.
        let mut m = Map::new(&["###############", "#??????@??????#", "###############"]);
        m.g[1][7] = b'<';
        let order = m.explore(None, 0).unwrap();
        assert_eq!(order, vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6], "ring by ring, both corridors");
        // C's order (nearest the Borg) follows one corridor to its end.
        let mut c = Map::new(&["###############", "#??????@??????#", "###############"]);
        c.g[1][7] = b'<';
        let mut depth_first = Vec::new();
        loop {
            let rings = c.rings();
            let cands = c.cands(&rings);
            let all = RingTry { base: 0, limit: i32::from(NO_RING), goals: cands.iter().map(|&(y, x, _)| (y, x)).collect() };
            let Some((y, x)) = c.nearest_goal(&rings, &all) else { break };
            depth_first.push(ring_of(&rings, y, x).unwrap());
            c.g[y as usize][x as usize] = b'.';
            c.borg = (y, x);
        }
        assert_eq!(&depth_first[..7], &[1, 2, 3, 4, 5, 6, 1], "one corridor to its end, then the other");
    }

    #[test]
    fn the_borg_goes_back_to_the_nearer_ring_before_the_grid_ahead() {
        // A corridor east from the staircase, a side branch north at x = 2.
        // The Borg (x = 8, ring 8) has an unknown grid 3 steps ahead (ring
        // 11) and one at the end of the branch, 6 steps away (ring 5: the
        // first step north is diagonal).
        let m = Map::new(&["##?#########", "##.#########", "##.#########", "##.#########", "<.......@..?", "############"]);
        let rings = m.rings();
        assert_eq!(ring_of(&rings, 4, 11), Some(11), "ahead: past ring 10");
        assert_eq!(ring_of(&rings, 0, 2), Some(5), "the branch's end: past ring 4");
        let cands = m.cands(&rings);
        let Plan::Rings(tries) = plan(&cands, Some(8), Some(12), 0, 16) else { panic!("planned") };
        assert_eq!(tries[0], RingTry { base: 5, limit: 8, goals: vec![(0, 2)] }, "back to the branch first");
        assert_eq!(m.nearest_goal(&rings, &tries[0]), Some((0, 2)));
        assert_eq!(tries[1].base, 11);
        let Plan::Rings(tries) = plan(&cands, Some(8), Some(10), 0, 16) else { panic!("planned") };
        assert_eq!(tries.len(), 1, "the leash, from the nearest staircase, drops ring 11");
        assert_eq!(plan(&cands, Some(8), Some(4), 0, 16), Plan::Rings(vec![]), "nothing within the leash");
        let Plan::Rings(tries) = plan(&cands, Some(8), None, 0, 1) else { panic!("planned") };
        assert_eq!(tries.len(), 1, "at most max tries");
    }

    #[test]
    fn a_path_never_crosses_a_ring_beyond_its_goals_or_the_borg() {
        // A loop: the staircase on the west, the Borg next to it (ring 1),
        // an unknown grid at ring 2 reached directly; the way round the loop
        // (rings 3 and more) is closed to it.
        let m = Map::new(&["#######", "#<@..?#", "#.###.#", "#.....#", "#######"]);
        let rings = m.rings();
        let cands = m.cands(&rings);
        let Plan::Rings(tries) = plan(&cands, ring_of(&rings, m.borg.0, m.borg.1), None, 0, 16) else { panic!("planned") };
        let t = &tries[0];
        assert_eq!(t.limit, ring_of(&rings, 1, 5).unwrap(), "the goal's own ring");
        assert!(!beyond_limit(rings[1][4], t.limit), "the direct way stays open");
        assert!(beyond_limit(rings[3][5], t.limit), "the far side of the loop is closed");
        assert!(!beyond_limit(NO_RING, 0), "an unknown grid is left to the spread's own rules");
        assert_eq!(path_limit(5, 8), 8, "standing farther out, the way back passes the Borg's ring");
    }

    #[test]
    fn the_slack_takes_the_next_rings_too() {
        let cands = [(0, 0, 3), (0, 1, 4), (0, 2, 5), (0, 3, 6)];
        let Plan::Rings(tries) = plan(&cands, Some(0), None, 1, 16) else { panic!("planned") };
        assert_eq!(tries[0], RingTry { base: 3, limit: 4, goals: vec![(0, 0), (0, 1)] });
        assert_eq!(tries[3], RingTry { base: 6, limit: 7, goals: vec![(0, 3)] });
    }

    #[test]
    fn without_a_route_from_the_borg_to_a_staircase_c_explores() {
        // Review of #1088: a known trap across the only corridor (#1067).
        // Unknown grids on the staircase's side have rings, the Borg has
        // none: C's methods, not "nothing to explore".
        let m = Map::new(&["#########", "#?<.^.@?#", "#########"]);
        let rings = m.rings();
        assert_eq!(ring_of(&rings, m.borg.0, m.borg.1), None);
        assert!(!m.cands(&rings).is_empty(), "the staircase's side has candidates");
        assert_eq!(plan(&m.cands(&rings), None, None, 0, 16), Plan::Fallback);
        let mut m = Map::new(&["#########", "#?<.^.@?#", "#########"]);
        assert_eq!(m.explore(None, 0), None);
        // No staircase known at all: no grid has a ring.
        let m = Map::new(&["#######", "#?.@.?#", "#######"]);
        let rings = m.rings();
        assert_eq!(plan(&m.cands(&rings), ring_of(&rings, m.borg.0, m.borg.1), None, 0, 16), Plan::Fallback);
    }

    #[test]
    fn rings_the_borg_cannot_reach_do_not_use_up_the_tries() {
        // Re-review of #1088: two regions, each with its own staircase. The
        // other region's candidates take rings 1-16; the only one the Borg
        // can reach is in ring 17, next to it.
        let mut cands: Vec<(i32, i32, i32)> = (1..=16).map(|r| (0, r, r)).collect();
        cands.push((2, 0, 17));
        let mut reach = vec![vec![NO_RING; 20]; 3];
        reach[2][1] = 0; // the Borg, next to its candidate
        let Plan::Rings(capped) = plan(&cands, Some(18), None, 0, 16) else { panic!("planned") };
        assert!(capped.iter().all(|t| !t.goals.contains(&(2, 0))), "unfiltered, the cap drops it");
        reachable_only(&mut cands, &reach);
        assert_eq!(cands, vec![(2, 0, 17)]);
        let Plan::Rings(tries) = plan(&cands, Some(18), None, 0, 16) else { panic!("planned") };
        assert_eq!(tries[0].goals, vec![(2, 0)]);
    }

    #[test]
    fn a_ring_reachable_only_past_the_return_leash_does_not_use_up_the_tries() {
        // Second re-review of #1088: the near ring (2, by the staircase) is
        // reachable from the Borg only up a corridor past the return leash
        // ('P'), which the route planner closes; ring 4, next to the Borg, is
        // the one it can walk to. With one try, the reach must drop ring 2.
        let m = Map::new(&["######", "#<.?##", "#P####", "#P####", "#P####", "#@?###", "######"]);
        let rings = m.rings();
        let mut cands = m.cands(&rings);
        assert_eq!(cands, vec![(1, 3, 2), (5, 2, 4)]);
        let borg_ring = ring_of(&rings, m.borg.0, m.borg.1);
        let Plan::Rings(tries) = plan(&cands, borg_ring, None, 0, 1) else { panic!("planned") };
        assert_eq!(m.nearest_goal(&rings, &tries[0]), None, "unfiltered: ring 2 has no route, and uses the try");
        reachable_only(&mut cands, &m.reach());
        assert_eq!(cands, vec![(5, 2, 4)]);
        let Plan::Rings(tries) = plan(&cands, borg_ring, None, 0, 1) else { panic!("planned") };
        assert_eq!(m.nearest_goal(&rings, &tries[0]), Some((5, 2)));
    }

    #[test]
    fn boosted_bravery_leaves_exploration_to_c() {
        // Re-review of #1088: avoidance above HP, the flow crosses unknown
        // grids and traps (`spread_blocked`'s twitchy), so no rings then.
        assert!(boosted_bravery(30, 12));
        assert!(!boosted_bravery(12, 12), "the avoidance is reset to the HP each think");
    }

    #[test]
    fn the_threshold_grows_with_the_level() {
        // A Human Mage (hit die 10: 10 HP at CL1, 5.5 more per level on
        // average) at 25 + 1 x CL stays through CL4 (26.5 < 29), not CL5.
        assert_eq!(bfs_hp_threshold(25, 0, 7), 25, "no delta: the class's value");
        assert_eq!(bfs_hp_threshold(25, 1, 4), 29);
        assert_eq!(bfs_hp_threshold(25, 1, 5), 30);
        assert_eq!(bfs_hp_threshold(30, -2, 5), 20, "a negative delta shrinks it");
    }

    #[test]
    fn the_switch_ships_off() {
        assert_eq!(value(Param::explore_breadth_first), 0, "compiled 0: C's order");
        assert_eq!(value(Param::explore_bfs_max_hp), 25);
        assert_eq!(value(Param::explore_bfs_slack), 0, "strict rings");
        assert_eq!(value(Param::explore_bfs_hp_per_level), 0, "no level delta");
        assert_eq!(value(Param::explore_bfs_food_percent), 100, "#1110: today's food");
        assert_eq!(value(Param::explore_bfs_light_percent), 100, "#1110: today's light");
    }
}

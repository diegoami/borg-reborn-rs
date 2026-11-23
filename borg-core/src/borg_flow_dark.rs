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
fn borg_flow_dark_1(st: &mut BorgState, b_stair: i32) -> bool {
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
fn borg_flow_dark_2(st: &mut BorgState, b_stair: i32) -> bool {
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
fn borg_flow_dark_3(st: &mut BorgState, b_stair: i32) -> bool {
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
fn borg_flow_dark_4(st: &mut BorgState, b_stair: i32) -> bool {
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
fn borg_flow_dark_5(st: &mut BorgState, b_stair: i32) -> bool {
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

/// borg-reborn-rs (#1080, owner 2026-11-23: "a safer way would be
/// breadth-first"): is exploration breadth-first from the staircases?
/// `explore_breadth_first` on, in the dungeon, and the Borg's maximum HP
/// below `explore_breadth_first_max_hp` (a value per class).
fn breadth_first_active(st: &BorgState) -> bool {
    value(Param::explore_breadth_first) != 0
        && st.borg.tr(BI_CDEPTH) > 0
        && st.borg.tr(BI_MAXHP) < value(Param::explore_breadth_first_max_hp)
}

/// No stair distance (`borg_data_hard`'s value, unreached by the spread).
const NO_RING: u8 = 255;

/// The eight neighbours (the pure helpers below do not call the engine).
const NEIGHBOURS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

/// #1080: the ring of a grid: its flow steps from the nearest known
/// staircase over known grids (`rings`, the spread from every staircase),
/// or, for a grid the spread does not enter (an unknown grid, rubble, a
/// vein), one more than its nearest entered neighbour. `None`: no route.
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

/// #1080: the rings to try, nearest first, of the candidates within the
/// leash (`None`: no leash applies), at most `max` of them.
fn rings_in_order(cands: &[(i32, i32, i32)], leash: Option<i32>, max: usize) -> Vec<i32> {
    let mut rings: Vec<i32> = cands.iter().map(|&(_, _, r)| r).filter(|&r| leash.is_none_or(|l| r <= l)).collect();
    rings.sort_unstable();
    rings.dedup();
    rings.truncate(max);
    rings
}

/// #1080: the farthest ring a path to ring `d` may cross: the target's own,
/// or the Borg's when it stands farther out (its way back passes there). A
/// grid beyond it is behind the Borg's back.
fn path_limit(d: i32, borg_ring: Option<i32>) -> i32 {
    borg_ring.map_or(i32::from(NO_RING), |b| d.max(b))
}

/// #1080: is a known grid of ring `ring` closed to a path whose limit is
/// `limit`?
fn beyond_limit(ring: u8, limit: i32) -> bool {
    ring != NO_RING && i32::from(ring) > limit
}

/// At most this many rings are tried in one think (one spread each).
const BREADTH_FIRST_MAX_RINGS: usize = 16;

/// #1080: breadth-first exploration from the staircases.
///
/// 1. One spread from every known staircase, of either kind, over known
///    grids gives each grid its ring (`ring_of`).
/// 2. The candidates are the interesting grids of the whole known map
///    (methods 3-5's, reachable, with method 1's torch-lit ones), within the
///    leash measured from that nearest staircase (where C's distance test
///    applies, `borg_flow_far_from_stairs_dist`).
/// 3. The nearest ring is explored first: every candidate of the ring is a
///    goal, so the flow leads to the one nearest the Borg (the tie-break).
///    A path may not cross a grid of a farther ring than the target's (or
///    the Borg's own, `path_limit`). When no path exists, the next ring.
///
/// `Some(stepped)`: this order decided (`false`: nothing to explore within
/// the leash); `None`: no staircase known, or no candidate connected to one.
fn borg_flow_dark_breadth_first(st: &mut BorgState) -> Option<bool> {
    if st.stairs.less.num == 0 && st.stairs.more.num == 0 {
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

    /* The leash, where C's stair distance test applies */
    let c = crate::borg_flow::c_leash();
    let leash_on = (st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - c.depth_margin
        && st.borg.tr(BI_CLEVEL) < c.stairs_max_level)
        || crate::borg_kit::leash_active(st);
    let leash = leash_on.then(|| borg_flow_misc::borg_get_leash(st, false));

    /* Candidates: torch-lit interesting grids, then every reachable one */
    let mut seen = vec![vec![false; AUTO_MAX_X as usize]; AUTO_MAX_Y as usize];
    let mut cands: Vec<(i32, i32, i32)> = Vec::new();
    let mut connected = false;
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
        seen[y as usize][x as usize] = true;
        if let Some(r) = ring_of(&rings, y, x) {
            connected = true;
            cands.push((y, x, r));
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
    if !connected {
        return None;
    }

    let borg_ring = ring_of(&rings, st.borg.c.y, st.borg.c.x);
    for d in rings_in_order(&cands, leash, BREADTH_FIRST_MAX_RINGS) {
        let limit = path_limit(d, borg_ring);
        crate::borg_flow::borg_flow_clear(st);

        /* Close the grids behind the Borg's back (a zero cost is "reached") */
        let mut closed = Vec::new();
        for (y, row) in rings.iter().enumerate() {
            for (x, &r) in row.iter().enumerate() {
                if beyond_limit(r, limit) && (y as i32, x as i32) != (st.borg.c.y, st.borg.c.x) {
                    st.flow.data_cost[y][x] = 0;
                    closed.push((y, x));
                }
            }
        }

        /* Every candidate of the ring is a goal */
        for &(y, x, r) in &cands {
            if r == d {
                crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
            }
        }
        crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);

        /* Reopen the closed grids (no goal of their own) */
        for (y, x) in closed {
            st.flow.data_cost[y][x] = st.flow.data_hard[y][x];
        }

        let who = std::ffi::CString::new(format!("dark-bfs (ring {d})")).expect("no NUL");
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

    // borg-reborn-rs (#1080): below explore_breadth_first_max_hp, both calls
    // explore breadth-first from the staircases (`None`: no known staircase,
    // or no candidate connected to one; C's methods then run as before).
    if breadth_first_active(st) {
        if let Some(stepped) = borg_flow_dark_breadth_first(st) {
            return stepped;
        }
    }

    if neer {
        /* Near */

        /* Method 1 */
        if borg_flow_dark_1(st, b_stair) {
            return true;
        }

        /* Method 2 */
        if borg_flow_dark_2(st, b_stair) {
            return true;
        }

        /* Method 3 */
        if borg_flow_dark_3(st, b_stair) {
            return true;
        }
    } else {
        /* Far */

        /* Method 4 */
        if borg_flow_dark_4(st, b_stair) {
            return true;
        }

        /* Method 5 */
        if borg_flow_dark_5(st, b_stair) {
            return true;
        }
    }

    /* Fail */
    false
}

#[cfg(test)]
mod breadth_first_tests {
    use super::*;

    /// The rings of a map, as the spread from every staircase gives them:
    /// '#' wall, '?' unknown (never entered), '<' or '>' a staircase, else
    /// known floor. Breadth-first over the eight neighbours.
    fn rings(map: &[&str]) -> Vec<Vec<u8>> {
        let h = map.len();
        let w = map[0].len();
        let cell = |y: usize, x: usize| map[y].as_bytes()[x];
        let mut r = vec![vec![NO_RING; w]; h];
        let mut queue = std::collections::VecDeque::new();
        for y in 0..h {
            for x in 0..w {
                if matches!(cell(y, x), b'<' | b'>') {
                    r[y][x] = 0;
                    queue.push_back((y, x));
                }
            }
        }
        while let Some((y, x)) = queue.pop_front() {
            for (dy, dx) in NEIGHBOURS {
                let (ny, nx) = (y as i32 + dy, x as i32 + dx);
                if ny < 0 || nx < 0 || ny as usize >= h || nx as usize >= w {
                    continue;
                }
                let (ny, nx) = (ny as usize, nx as usize);
                if matches!(cell(ny, nx), b'#' | b'?') || r[ny][nx] != NO_RING {
                    continue;
                }
                r[ny][nx] = r[y][x] + 1;
                queue.push_back((ny, nx));
            }
        }
        r
    }

    // A corridor east from the up staircase, a side branch north at x = 2.
    // The Borg (at x = 8, ring 8) has an unknown grid 3 steps ahead (ring
    // 11) and one at the end of the branch, 6 steps away (ring 5: the
    // first step north is diagonal).
    const MAP: [&str; 6] = [
        "##?#########",
        "##.#########",
        "##.#########",
        "##.#########",
        "<..........?",
        "############",
    ];

    #[test]
    fn a_grid_the_spread_does_not_enter_is_one_more_than_its_nearest_neighbour() {
        let r = rings(&MAP);
        assert_eq!(ring_of(&r, 4, 0), Some(0), "the staircase");
        assert_eq!(ring_of(&r, 4, 8), Some(8), "the Borg's grid");
        assert_eq!(ring_of(&r, 4, 11), Some(11), "ahead: past ring 10");
        assert_eq!(ring_of(&r, 0, 2), Some(5), "the branch's end: past ring 4");
        assert_eq!(ring_of(&r, 0, 8), None, "walled off: no route to a staircase");
    }

    #[test]
    fn the_nearest_ring_to_the_staircase_is_explored_first_not_the_grid_nearest_the_borg() {
        let r = rings(&MAP);
        let cands: Vec<(i32, i32, i32)> =
            [(4, 11), (0, 2)].iter().map(|&(y, x)| (y, x, ring_of(&r, y, x).unwrap())).collect();
        // C's order takes the grid ahead (3 steps from the Borg); this one
        // the branch (ring 5), then the corridor's end.
        assert_eq!(rings_in_order(&cands, Some(12), 16), vec![5, 11]);
        assert_eq!(rings_in_order(&cands, None, 16), vec![5, 11], "no leash applies");
        assert_eq!(rings_in_order(&cands, Some(10), 16), vec![5], "the leash, from the nearest staircase");
        assert_eq!(rings_in_order(&cands, Some(4), 16), Vec::<i32>::new(), "nothing within the leash");
        assert_eq!(rings_in_order(&cands, Some(12), 1), vec![5], "at most max rings");
    }

    #[test]
    fn a_path_never_crosses_a_ring_beyond_its_target_or_the_borg() {
        // Going back to ring 5 from ring 8, the corridor up to ring 8 stays
        // open; ring 9 and farther (behind the Borg's back) are closed.
        let limit = path_limit(5, Some(8));
        assert_eq!(limit, 8);
        assert!(!beyond_limit(8, limit));
        assert!(beyond_limit(9, limit));
        // Standing at ring 2, a ring-5 target may not be reached around
        // through ring 6.
        assert_eq!(path_limit(5, Some(2)), 5);
        assert!(beyond_limit(6, path_limit(5, Some(2))));
        // An unknown grid (no ring) is left to the spread's own rules; a
        // Borg cut off from every staircase has no limit.
        assert!(!beyond_limit(NO_RING, 0));
        assert_eq!(path_limit(5, None), i32::from(NO_RING));
    }
}

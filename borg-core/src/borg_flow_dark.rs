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

/// borg-reborn-rs (#1080, #1085; owner 2026-11-23: "a safer way would be
/// breadth-first"): is exploration breadth-first from the staircases?
/// `explore_breadth_first` on, in the dungeon, and the Borg's maximum HP
/// below `bfs_hp_threshold` (owner, 2026-11-23: by level and class).
pub(crate) fn breadth_first_active(st: &BorgState) -> bool {
    value(Param::explore_breadth_first) != 0
        && st.borg.tr(BI_CDEPTH) > 0
        && st.borg.tr(BI_MAXHP)
            < bfs_hp_threshold(
                value(Param::explore_bfs_max_hp),
                value(Param::explore_bfs_hp_per_level),
                st.borg.tr(BI_CLEVEL),
            )
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
    if i32::from(st.borg.goal.type_0) != GOAL_DARK {
        return;
    }
    if breadth_first_active(st) {
        crate::borg_io::borg_note(st, &format!("# {why}: planning the breadth-first exploration again (#1085)"));
        st.borg.goal.type_0 = 0;
    } else if mixed_active(st) {
        // borg-reborn-rs (#1098): the mixed walk re-plans the same way.
        crate::borg_io::borg_note(st, &format!("# {why}: planning the mixed exploration again (#1098)"));
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

/// #1080: what both stair-anchored orders (breadth-first and mixed) read.
struct StairFrame {
    /// Each grid's flow steps from the nearest known staircase (`ring_of`).
    rings: crate::borg_flow::BorgData,
    /// The Borg's own ring.
    borg_ring: i32,
    /// The interesting grids `(y, x, ring)` the Borg can reach.
    cands: Vec<(i32, i32, i32)>,
    /// The grids past the "wandered too far" return leash.
    past_return: Vec<Vec<bool>>,
}

/// #1080, #1085: the frame of the stair-anchored orders.
///
/// 1. One spread from every known staircase, of either kind, over known
///    grids gives each grid its ring (`ring_of`).
/// 2. The candidates are the interesting grids of the whole known map
///    (methods 3-5's, reachable, with method 1's torch-lit ones) within the
///    return leash of `borg_think_dungeon` while it applies.
/// 3. Only the candidates the Borg can reach (one spread from the Borg,
///    through no grid past the return leash) are kept.
///
/// `None`: no staircase known, boosted bravery, or the Borg has no route to
/// a known staircase (C's methods explore then).
fn stair_frame(st: &mut BorgState) -> Option<StairFrame> {
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
    let borg_ring = ring_of(&rings, st.borg.c.y, st.borg.c.x)?;

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

    Some(StairFrame { rings, borg_ring, cands, past_return })
}

/// #1080, #1085: the tries of a stair-anchored order, in turn: each closes
/// the grids beyond its path limit (behind the Borg's back) and past the
/// return leash, spreads from all its goals, so the flow leads to the goal
/// nearest the Borg, and commits to the first that gives a path. `label`
/// names the try in the note (`# Flowing toward <label> at cost n`).
fn run_tries(st: &mut BorgState, frame: &StairFrame, tries: Vec<RingTry>, label: impl Fn(&RingTry) -> String) -> bool {
    for t in tries {
        crate::borg_flow::borg_flow_clear(st);

        /* Close the grids behind the Borg's back (a zero cost is "reached") */
        let mut closed = Vec::new();
        for (y, row) in frame.rings.iter().enumerate() {
            for (x, &r) in row.iter().enumerate() {
                if (beyond_limit(r, t.limit) || frame.past_return[y][x]) && (y as i32, x as i32) != (st.borg.c.y, st.borg.c.x) {
                    st.flow.data_cost[y][x] = 0;
                    closed.push((y, x));
                }
            }
        }

        /* Every goal of the try is a starting grid */
        for &(y, x) in &t.goals {
            crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
        }
        crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);

        /* Reopen the closed grids (no goal of their own) */
        for (y, x) in closed {
            st.flow.data_cost[y][x] = st.flow.data_hard[y][x];
        }

        let who = std::ffi::CString::new(label(&t)).expect("no NUL");
        if commit_and_step(st, &who) {
            return true;
        }
    }
    false
}

/// Is C's leash on for exploration? Where C's distance test applies
/// (`borg_flow_far_from_stairs_dist`: the depth at least the level less the
/// margin, below `flow_leash_stairs_max_level`), or while the kit is short.
fn c_leash_on(st: &BorgState) -> bool {
    let c = crate::borg_flow::c_leash();
    (st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - c.depth_margin && st.borg.tr(BI_CLEVEL) < c.stairs_max_level)
        || crate::borg_kit::leash_active(st)
}

/// #1080, #1085: breadth-first exploration from the staircases.
///
/// On the frame of `stair_frame`, within the leash measured from the nearest
/// staircase (where C's distance test applies), the nearest ring is explored
/// first (`plan`): every candidate from it to it + `explore_bfs_slack` is a
/// goal, so the flow leads to the one nearest the Borg (the tie-break). A
/// path may not cross a grid of a farther ring than that (or than the Borg's
/// own, `path_limit`), nor one past the return leash. When no path exists,
/// the next ring.
///
/// `Some(stepped)`: this order decided (`false`: nothing to explore within
/// the leashes); `None`: the Borg has no route to a known staircase.
fn borg_flow_dark_breadth_first(st: &mut BorgState) -> Option<bool> {
    let frame = stair_frame(st)?;
    let leash = c_leash_on(st).then(|| borg_flow_misc::borg_get_leash(st, false));
    let slack = value(Param::explore_bfs_slack);
    let Plan::Rings(tries) = plan(&frame.cands, Some(frame.borg_ring), leash, slack, BREADTH_FIRST_MAX_RINGS) else {
        return None;
    };
    Some(run_tries(st, &frame, tries, |t| format!("dark-bfs (ring {})", t.base)))
}

/// #1098 (second piece of #1080): is exploration mixed? `explore_mixed` on,
/// in the dungeon, breadth-first not applying (its HP threshold passed, or
/// its switch off), and the Borg not yet prepared for the depth. #1080's
/// readiness model is not written: until it is, "not prepared" is where
/// C's own leash applies (`c_leash_on`: the depth at least the level less
/// `flow_leash_depth_margin`, below `flow_leash_stairs_max_level`, or the
/// kit short). Elsewhere exploration is C's depth-first order, as today.
/// Not on a level where leaving after the walk failed
/// (`release_after_failed_exit`).
pub(crate) fn mixed_active(st: &BorgState) -> bool {
    value(Param::explore_mixed) != 0
        && st.borg.tr(BI_CDEPTH) > 0
        && !breadth_first_active(st)
        && c_leash_on(st)
        && st.borg.mix_released != Some(st.borg.time.level)
}

/// #1098: leaving after the walk was exhausted failed (`borg_leave_level`
/// found no staircase it could take): the walk stands down on this level,
/// so C's exploration can look for one.
pub(crate) fn release_after_failed_exit(st: &mut BorgState) {
    if st.borg.mix_released != Some(st.borg.time.level) {
        crate::borg_io::borg_note(st, "# Mixed exploration: no staircase to leave by, exploring on (#1098)");
    }
    st.borg.mix_released = Some(st.borg.time.level);
}

/// #1098: the numbers of the escape model of #1061 that bound the radius.
#[derive(Clone, Copy, Debug)]
struct MixModel {
    /// `explore_mix_radius`: the radius the walk starts at (and its floor).
    start: i32,
    /// `explore_mix_sight`: how far a monster is first seen, at least.
    sight: i32,
    /// The Borg's light radius: in a corridor it sees no farther.
    light: i32,
    /// `explore_mix_speed_ratio`: the speed ratio of the depth's fast monsters.
    ratio: i32,
    /// The Borg's maximum HP.
    max_hp: i32,
    /// `explore_mix_contact_damage`: the damage of one turn of contact at
    /// DL1-5, multiplied by the depth band.
    contact: i32,
    /// The current depth.
    depth: i32,
    /// `explore_mix_max_radius`: the cap.
    max: i32,
}

/// #1098: the depth band of the contact damage: 1 at DL1-5, 2 at DL6-10, ...
fn depth_band(depth: i32) -> i32 {
    ((depth.max(1) + 4) / 5).max(1)
}

/// #1098: the farthest the radius may grow, in flow steps from the nearest
/// staircase: the escape model of #1061, *R* = ⌊*D* / (*r* − 1)⌋ + *k*.
/// - *D* = max(`sight`, `light`): a monster first seen *D* grids away,
///   *r* times as fast, closes the gap after the Borg has walked *D* / (*r*
///   − 1) grids toward the stairs;
/// - *k* = ⌊HP / *c*⌋: the turns of contact it survives on the rest of the
///   way, with *c* = `contact` x the depth band.
///
/// No lower than the start, no higher than the cap.
fn mix_limit(m: MixModel) -> i32 {
    let d = m.sight.max(m.light).max(0);
    let run = d / (m.ratio - 1).max(1);
    let k = m.max_hp.max(0) / m.contact.max(1).saturating_mul(depth_band(m.depth));
    run.saturating_add(k).max(m.start).min(m.max)
}

/// #1098: the mixed plan from the candidates `(y, x, ring)` and the Borg's
/// own ring. The radius starts at `start` and grows by `step` up to `limit`;
/// the first radius that holds a candidate is tried first: every candidate
/// within it is a goal, so the flow leads to the one nearest the Borg
/// (depth-first inside the radius), and a path may not cross a ring beyond
/// the radius (or the Borg's own, `path_limit`). A branch whose frontier
/// lies beyond the radius is parked; the radius grows only when no
/// candidate is left inside it. The larger radii follow, for goals the
/// path limit cuts off (at most `max` distinct tries: a radius whose goals
/// and path limit are the last try's is skipped, review of #1117). Empty:
/// nothing within the limit, and the level is done (`MixOutcome::Exhausted`).
///
/// The radius is not stored: each think takes the smallest that holds a
/// candidate, so a branch newly found near the stairs is covered before the
/// walk goes deeper again.
fn mixed_plan(cands: &[(i32, i32, i32)], borg_ring: Option<i32>, start: i32, step: i32, limit: i32, max: usize) -> Plan {
    let Some(borg_ring) = borg_ring else { return Plan::Fallback };
    let mut tries = Vec::new();
    let mut r = start.min(limit);
    loop {
        let goals: Vec<(i32, i32)> = cands.iter().filter(|&&(_, _, g)| g <= r).map(|&(y, x, _)| (y, x)).collect();
        let limit_r = path_limit(r, borg_ring);
        // Review of #1117: a radius with the same goals and path limit as
        // the last try would spread the same way: it is no try of its own,
        // and does not use up the budget.
        let same = tries.last().is_some_and(|t: &RingTry| t.limit == limit_r && t.goals == goals);
        if !goals.is_empty() && !same && tries.len() < max {
            tries.push(RingTry { base: r, limit: limit_r, goals });
        }
        if r >= limit {
            break;
        }
        r = r.saturating_add(step.max(1)).min(limit);
    }
    Plan::Rings(tries)
}

/// #1098 (second piece of #1080): mixed exploration from the staircases:
/// depth-first inside a radius from the stairs that grows only when every
/// branch inside it is covered (`mixed_plan`), on the frame of
/// `stair_frame`. The limit is the escape model's (`mix_limit`), and never
/// more than C's leash where that applies.
///
/// `Some(stepped)`: this order decided (`false`: nothing to explore within
/// the limit, so the level is left); `None`: the Borg has no route to a
/// known staircase.
fn borg_flow_dark_mixed(st: &mut BorgState) -> Option<bool> {
    let frame = stair_frame(st)?;
    let start = value(Param::explore_mix_radius);
    let mut limit = mix_limit(MixModel {
        start,
        sight: value(Param::explore_mix_sight),
        light: crate::borg_cave_light::own_light_radius(st),
        ratio: value(Param::explore_mix_speed_ratio),
        max_hp: st.borg.tr(BI_MAXHP),
        contact: value(Param::explore_mix_contact_damage),
        depth: st.borg.tr(BI_CDEPTH),
        max: value(Param::explore_mix_max_radius),
    });
    if c_leash_on(st) {
        limit = limit.min(borg_flow_misc::borg_get_leash(st, false));
    }
    let step = value(Param::explore_mix_step);
    let plan = mixed_plan(&frame.cands, Some(frame.borg_ring), start, step, limit, BREADTH_FIRST_MAX_RINGS);
    let stepped = match plan {
        Plan::Fallback => return None,
        Plan::Rings(tries) => run_tries(st, &frame, tries, |t| format!("dark-mix (radius {} of {limit})", t.base)),
    };
    if mix_result(stepped) == MixOutcome::Stepped {
        return Some(true);
    }
    // Review of #1117: not an ordinary miss. Nothing within the limit, or
    // no allowed path to any of it (re-review: every try failed): the area
    // the walk may cover is done, so `borg_leave_level` takes the stairs
    // this think, before the secret-door search and the direct monster
    // flow (`exhausted_this_think`).
    if st.borg.mix_exhausted != Some(st.borg.time.now) {
        let left = frame.cands.len();
        crate::borg_io::borg_note(
            st,
            &format!("# Mixed exploration: nothing reachable left within radius {limit} ({left} frontier grids known), leaving the level (#1098)"),
        );
    }
    st.borg.mix_exhausted = Some(st.borg.time.now);
    Some(false)
}

/// #1098 (reviews of #1117): what a mixed plan that the Borg can route
/// from (not `Plan::Fallback`) means for the think.
#[derive(Debug, PartialEq)]
enum MixOutcome {
    /// A try gave a path: the Borg steps.
    Stepped,
    /// No try gave a path, or there was none (nothing within the limit): the
    /// level is left (the stair-exit policy, `borg_leave_level`), not
    /// searched for secret doors first.
    Exhausted,
}

fn mix_result(stepped: bool) -> MixOutcome {
    if stepped {
        MixOutcome::Stepped
    } else {
        MixOutcome::Exhausted
    }
}

/// #1098 (review of #1117): did the mixed walk find its area covered this
/// think? Read by `borg_leave_level` as a reason to leave, like the level
/// timer.
pub(crate) fn exhausted_this_think(st: &BorgState) -> bool {
    st.borg.mix_exhausted == Some(st.borg.time.now) && mixed_active(st)
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

    // borg-reborn-rs (#1080, #1085): below explore_bfs_max_hp, both calls
    // explore breadth-first from the staircases (`None`: the Borg has no
    // route to a known staircase; C's methods then run as before).
    if breadth_first_active(st) {
        if let Some(stepped) = borg_flow_dark_breadth_first(st) {
            return stepped;
        }
    }
    // borg-reborn-rs (#1098): above it, until prepared for the depth, both
    // calls explore depth-first inside a radius from the staircases that
    // grows only when every branch inside it is covered.
    if mixed_active(st) {
        if let Some(stepped) = borg_flow_dark_mixed(st) {
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

    impl Map {
        /// #1098: explore with the mixed plan until nothing is left within
        /// the limit; the rings explored, in order (`None`: fell back).
        fn explore_mixed(&mut self, start: i32, step: i32, limit: i32) -> Option<Vec<i32>> {
            let mut order = Vec::new();
            loop {
                let rings = self.rings();
                let cands = self.cands(&rings);
                let Plan::Rings(tries) =
                    mixed_plan(&cands, ring_of(&rings, self.borg.0, self.borg.1), start, step, limit, BREADTH_FIRST_MAX_RINGS)
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
    }

    /// Two corridors from a staircase, each 8 unknown grids long.
    fn two_corridors() -> Map {
        let mut m = Map::new(&["###################", "#????????@????????#", "###################"]);
        m.g[1][9] = b'<';
        m
    }

    #[test]
    fn mixed_two_corridors_are_both_followed_to_the_radius_before_either_beyond() {
        // #1098: R = 5. Depth-first inside it (one corridor to 5, then the
        // other), then R grows by 2 (to 7: the second corridor, where the Borg
        // stands, first), then to the limit 8.
        let mut m = two_corridors();
        let order = m.explore_mixed(5, 2, 8).unwrap();
        assert_eq!(order, vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 6, 7, 6, 7, 8, 8]);
        assert!(order[..10].iter().all(|&r| r <= 5), "every branch to 5 before any beyond");
        // Breadth-first alternates ring by ring; C goes to the end of one.
        let mut b = two_corridors();
        assert_eq!(&b.explore(None, 0).unwrap()[..4], &[1, 1, 2, 2]);
    }

    #[test]
    fn mixed_a_corridor_longer_than_the_radius_is_parked_then_resumed() {
        // #1098: a corridor east whose frontier (ring 6) lies past R = 5 where
        // the Borg stands, and a side branch north near the staircase (ring 2).
        // The Borg walks back to the branch (the corridor is parked), and
        // resumes the corridor only when R grows.
        let mut m = Map::new(&["#?##########", "#.##########", "<....@?#####", "############"]);
        let rings = m.rings();
        assert_eq!(ring_of(&rings, 2, 6), Some(6), "the corridor's frontier: past R");
        assert_eq!(ring_of(&rings, 0, 1), Some(2), "the branch");
        let cands = m.cands(&rings);
        let Plan::Rings(tries) = mixed_plan(&cands, Some(5), 5, 2, 9, 16) else { panic!("planned") };
        assert_eq!(tries[0], RingTry { base: 5, limit: 5, goals: vec![(0, 1)] }, "only the branch inside R");
        assert_eq!(m.nearest_goal(&rings, &tries[0]), Some((0, 1)));
        assert_eq!(tries[1].base, 7, "the corridor comes with a larger radius");
        assert_eq!(m.explore_mixed(5, 2, 9).unwrap(), vec![2, 6]);
    }

    #[test]
    fn mixed_nothing_within_the_limit_ends_the_walk() {
        // #1098: the only frontier is past the limit: no try, so the level is
        // left (borg_flow_dark returns false), not explored beyond.
        let cands = [(0, 0, 12)];
        assert_eq!(mixed_plan(&cands, Some(0), 5, 2, 10, 16), Plan::Rings(vec![]));
        let Plan::Rings(tries) = mixed_plan(&cands, Some(0), 5, 2, 12, 16) else { panic!("planned") };
        assert_eq!(tries.len(), 1);
        assert_eq!(tries[0].base, 12, "the last step stops at the limit");
        // A start beyond the limit starts at the limit.
        let Plan::Rings(tries) = mixed_plan(&[(0, 0, 3)], Some(0), 5, 2, 4, 16) else { panic!("planned") };
        assert_eq!(tries[0].base, 4);
    }

    #[test]
    fn mixed_the_radius_is_the_smallest_that_holds_a_candidate() {
        // #1098: the radius is not stored; a branch newly found near the
        // stairs (ring 2) comes before the far one the Borg was walking (ring
        // 9), and the path limit is the Borg's own ring when it stands
        // farther out.
        let cands = [(0, 0, 9), (0, 1, 2)];
        let Plan::Rings(tries) = mixed_plan(&cands, Some(8), 5, 2, 11, 16) else { panic!("planned") };
        assert_eq!(tries[0], RingTry { base: 5, limit: 8, goals: vec![(0, 1)] });
        assert_eq!(tries[1], RingTry { base: 9, limit: 9, goals: vec![(0, 0), (0, 1)] }, "radius 7 repeats radius 5: skipped");
        assert_eq!(tries.len(), 3, "and radius 11, a looser path limit");
    }

    #[test]
    fn mixed_identical_tries_do_not_use_up_the_budget() {
        // Review of #1117: two staircases 42 steps apart, the Borg at ring 20
        // from the west one, a lone frontier at ring 1 next to the east one.
        // Its path crosses ring 21 (the middle). Start 5, step 1: radii 5-20
        // all give the same goal and path limit 20, which has no path; the
        // budget of 16 must still reach radius 21.
        let mut row1 = vec![b'.'; 45];
        row1[0] = b'#';
        row1[44] = b'#';
        row1[1] = b'<';
        row1[43] = b'>';
        row1[21] = b'@';
        let mut row0 = vec![b'#'; 45];
        row0[43] = b'?';
        let rows = [String::from_utf8(row0).unwrap(), String::from_utf8(row1).unwrap(), "#".repeat(45)];
        let mut m = Map::new(&rows.iter().map(String::as_str).collect::<Vec<_>>());
        let rings = m.rings();
        assert_eq!(ring_of(&rings, m.borg.0, m.borg.1), Some(20));
        assert_eq!(ring_of(&rings, 1, 22), Some(21), "the middle");
        let cands = m.cands(&rings);
        assert_eq!(cands, vec![(0, 43, 1)]);
        let Plan::Rings(tries) = mixed_plan(&cands, Some(20), 5, 1, 30, 16) else { panic!("planned") };
        assert_eq!(tries[0], RingTry { base: 5, limit: 20, goals: vec![(0, 43)] });
        assert_eq!(m.nearest_goal(&rings, &tries[0]), None, "limit 20 cuts the way through the middle");
        assert_eq!(tries[1], RingTry { base: 21, limit: 21, goals: vec![(0, 43)] }, "radii 6-20 are no tries of their own");
        assert_eq!(m.nearest_goal(&rings, &tries[1]), Some((0, 43)));
        assert_eq!(m.explore_mixed(5, 1, 30), Some(vec![1]));
        // Re-review of #1117: with the limit at 20, the plan has a try, but
        // its path crosses ring 21: no try has a path, so the walk is
        // exhausted (and leaves), not an ordinary miss.
        let mut m = Map::new(&rows.iter().map(String::as_str).collect::<Vec<_>>());
        let Plan::Rings(tries) = mixed_plan(&cands, Some(20), 5, 1, 20, 16) else { panic!("planned") };
        assert_eq!(tries, vec![RingTry { base: 5, limit: 20, goals: vec![(0, 43)] }], "one try, the rest repeat it");
        let stepped = tries.iter().any(|t| m.nearest_goal(&rings, t).is_some());
        assert!(!stepped);
        assert_eq!(mix_result(stepped), MixOutcome::Exhausted);
        assert_eq!(m.explore_mixed(5, 1, 20), Some(vec![]), "nothing explored past the limit");
    }

    #[test]
    fn mixed_radius_exhaustion_is_its_own_outcome() {
        // Review of #1117: nothing within the limit, or no try with a path,
        // is not an ordinary miss: the level is left (borg_leave_level reads
        // `exhausted_this_think`).
        assert_eq!(mixed_plan(&[(0, 0, 12)], Some(0), 5, 2, 10, 16), Plan::Rings(vec![]), "no try: exhausted");
        assert_eq!(mix_result(false), MixOutcome::Exhausted);
        assert_eq!(mix_result(true), MixOutcome::Stepped);
        assert_eq!(mixed_plan(&[(0, 0, 3)], None, 5, 2, 10, 16), Plan::Fallback, "no route to a staircase: C");
    }

    #[test]
    fn mixed_without_a_route_to_a_staircase_c_explores() {
        // #1098, as breadth-first: no ring for the Borg, C's methods.
        let mut m = Map::new(&["#########", "#?<.^.@?#", "#########"]);
        assert_eq!(m.explore_mixed(5, 2, 10), None);
        assert_eq!(mixed_plan(&[(0, 0, 1)], None, 5, 2, 10, 16), Plan::Fallback);
    }

    /// The model at DL1-5 with the shipped numbers and `max_hp`.
    fn model(max_hp: i32) -> MixModel {
        MixModel { start: 5, sight: 5, light: 2, ratio: 2, max_hp, contact: 5, depth: 3, max: 30 }
    }

    #[test]
    fn mixed_the_limit_grows_with_hp_as_the_escape_model_says() {
        // #1098: R = D / (r - 1) + HP / c, D = max(sight, light).
        assert_eq!(mix_limit(model(17)), 5 + 3, "Mage CL4 584191223, 17 HP: 8 (C's leash: 21)");
        assert_eq!(mix_limit(model(26)), 5 + 5, "Mage CL4 645909552, 26 HP: 10");
        assert_eq!(mix_limit(model(60)), 5 + 12);
        assert!(mix_limit(model(60)) > mix_limit(model(26)), "a bigger HP pool, a bigger radius");
        assert_eq!(mix_limit(MixModel { light: 7, ..model(26) }), 7 + 5, "a light wider than the sight");
        assert_eq!(mix_limit(MixModel { ratio: 3, ..model(26) }), 2 + 5, "faster monsters: half the run");
        assert_eq!(mix_limit(MixModel { depth: 7, ..model(26) }), 5 + 2, "DL6-10: twice the contact damage");
        assert_eq!(mix_limit(model(500)), 30, "the cap");
        assert_eq!(mix_limit(MixModel { sight: 0, light: 0, ..model(0) }), 5, "never below the start");
        assert_eq!(depth_band(1), 1);
        assert_eq!(depth_band(5), 1);
        assert_eq!(depth_band(6), 2);
        assert_eq!(depth_band(11), 3);
    }

    #[test]
    fn mixed_ships_off() {
        // #1098: the switch at 0 is C's order: borg_flow_dark never calls the
        // mixed walk (`mixed_active` tests it first).
        assert_eq!(value(Param::explore_mixed), 0, "compiled 0: C's order");
        assert_eq!(value(Param::explore_mix_radius), 5);
        assert_eq!(value(Param::explore_mix_step), 2);
        assert_eq!(value(Param::explore_mix_sight), 5);
        assert_eq!(value(Param::explore_mix_speed_ratio), 2);
        assert_eq!(value(Param::explore_mix_contact_damage), 5);
        assert_eq!(value(Param::explore_mix_max_radius), 30);
    }
}

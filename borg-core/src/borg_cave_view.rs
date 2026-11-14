//! Port of `borg-cave-view.c`: routines to update the cave view.
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

use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::borg_cave_floor_grid;
use crate::borg_projection::borg_los;
use crate::state::BorgState;
use crate::engine;

/// Size of the view arrays (`AUTO_VIEW_MAX`).
pub const AUTO_VIEW_MAX: usize = 9000;

/// Grid info flag: the grid is in view (`BORG_VIEW`, borg-cave.h).
pub const BORG_VIEW: u16 = 0x20;
/// Grid info flag: scratch "easily viewable" mark (`BORG_XTRA`, borg-cave.h).
pub const BORG_XTRA: u16 = 0x80;

/// The set of grids marked as `BORG_VIEW` (`borg_view_n`, `borg_view_x`,
/// `borg_view_y`). The player grid is always entry 0 after
/// `borg_update_view()`.
pub struct ViewState {
    /// `borg_view_n`
    pub view_n: i16,
    /// `borg_view_x`
    pub view_x: Vec<u8>,
    /// `borg_view_y`
    pub view_y: Vec<u8>,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            view_n: 0,
            view_x: vec![0; AUTO_VIEW_MAX],
            view_y: vec![0; AUTO_VIEW_MAX],
        }
    }
}

/// Clear the viewable space (`borg_forget_view`).
pub fn borg_forget_view(st: &mut BorgState) {
    /* None to forget */
    if st.view.view_n == 0 {
        return;
    }

    /* Clear them all */
    for i in 0..i32::from(st.view.view_n) {
        let y = i32::from(st.view.view_y[i as usize]);
        let x = i32::from(st.view.view_x[i as usize]);

        /* Forget that the grid is viewable */
        st.cave.grid_mut(x, y).info &= !BORG_VIEW;
    }

    /* None left */
    st.view.view_n = 0;
}

/// Add a grid to the "view" array (the `borg_cave_view_hack` macro).
///
/// Never called for illegal grids, for grids already in the array, or
/// when the array is full.
fn borg_cave_view_hack(st: &mut BorgState, y: i32, x: i32) {
    st.cave.grid_mut(x, y).info |= BORG_VIEW;
    let n = st.view.view_n as usize;
    st.view.view_y[n] = y as u8;
    st.view.view_x[n] = x as u8;
    st.view.view_n += 1;
}

/// Helper for `borg_update_view()` (`borg_update_view_aux`).
///
/// See `update_view_aux()` in `cave.c` for complete documentation.
fn borg_update_view_aux(
    st: &mut BorgState,
    y: i32,
    x: i32,
    y1: i32,
    x1: i32,
    y2: i32,
    x2: i32,
) -> bool {
    /* Access the grids */
    let g1_ag = *st.cave.grid(x1, y1);
    let g2_ag = *st.cave.grid(x2, y2);

    /* Check for walls */
    let f1 = borg_cave_floor_grid(&g1_ag);
    let f2 = borg_cave_floor_grid(&g2_ag);

    /* Totally blocked by physical walls */
    if !f1 && !f2 {
        return true;
    }

    /* Check for visibility */
    let v1 = f1 && (g1_ag.info & BORG_VIEW) != 0;
    let v2 = f2 && (g2_ag.info & BORG_VIEW) != 0;

    /* Totally blocked by "unviewable neighbors" */
    if !v1 && !v2 {
        return true;
    }

    /* Check for walls */
    let wall = !borg_cave_floor_grid(st.cave.grid(x, y));

    /* Check the "ease" of visibility */
    let vis1 = v1 && (g1_ag.info & BORG_XTRA) != 0;
    let vis2 = v2 && (g2_ag.info & BORG_XTRA) != 0;

    /* "easy" plus "easy" yields "easy" */
    if vis1 && vis2 {
        st.cave.grid_mut(x, y).info |= BORG_XTRA;

        borg_cave_view_hack(st, y, x);

        return wall;
    }

    /* Primary "easy" yields "viewed" */
    if vis1 {
        borg_cave_view_hack(st, y, x);

        return wall;
    }

    /* "view" plus "view" yields "view" */
    if v1 && v2 {
        /* ag->info |= BORG_XTRA; */

        borg_cave_view_hack(st, y, x);

        return wall;
    }

    /* Mega-Hack -- the "borg_los()" function works poorly on walls */
    if wall {
        borg_cave_view_hack(st, y, x);

        return wall;
    }

    /* Check line of sight */
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    if borg_los(st, cy, cx, y, x) {
        borg_cave_view_hack(st, y, x);

        return wall;
    }

    /* Assume no line of sight. */
    true
}

/// Steps 2 and 3: scan from (x, y) in direction (dx, dy) for `d` in
/// `1..=limit`, marking grids as easily viewable until a wall. Grids not
/// fully in bounds are skipped (not a stop). Returns the final `d`.
fn scan_ray(st: &mut BorgState, y: i32, x: i32, dy: i32, dx: i32, limit: i32) -> i32 {
    let mut d = 1;
    while d <= limit {
        let gx = x + dx * d;
        let gy = y + dy * d;

        /* Caution */
        if engine::square_in_bounds_fully(gx, gy) {
            st.cave.grid_mut(gx, gy).info |= BORG_XTRA;
            borg_cave_view_hack(st, gy, gx);
            if !borg_cave_floor_grid(st.cave.grid(gx, gy)) {
                break;
            }
        }
        d += 1;
    }
    d
}

/// Step 4: scan grid `d` (1..=m) of strip `n`, noticing "blockage".
/// `aux(d)` gives the `borg_update_view_aux` arguments. Returns the new
/// limit for the next strip (`k + 1`).
fn scan_strip(
    st: &mut BorgState,
    n: i32,
    m: i32,
    limit: i32,
    aux: impl Fn(i32) -> (i32, i32, i32, i32, i32, i32),
) -> i32 {
    let mut k = n;
    let mut d = 1;
    while d <= m {
        let (y, x, y1, x1, y2, x2) = aux(d);

        /* Check grid "d" in strip "n", notice "blockage" */
        if borg_update_view_aux(st, y, x, y1, x1, y2, x2) {
            if n + d >= limit {
                break;
            }
        }
        /* Track most distant "non-blockage" */
        else {
            k = n + d;
        }
        d += 1;
    }

    /* Limit the next strip */
    k + 1
}

/// Calculate the region currently "viewable" by the player
/// (`borg_update_view`).
///
/// See `update_view()` in `cave.c` for complete documentation.
///
/// It is very important that the "player grid" be the first grid in the
/// array of `BORG_VIEW` grids, since this is assumed in several places.
pub fn borg_update_view(st: &mut BorgState) {
    /*** Initialize ***/

    let max_sight = i32::from(engine::z_info().max_sight);

    /* Full radius (20) */
    let full = max_sight;

    /* Octagon factor (30) */
    let over = max_sight * 3 / 2;

    /*** Step 0 -- Begin ***/

    /* Save the old "view" grids for later */
    for n in 0..i32::from(st.view.view_n) {
        let y = i32::from(st.view.view_y[n as usize]);
        let x = i32::from(st.view.view_x[n as usize]);

        /* Mark the grid as not in "view" */
        st.cave.grid_mut(x, y).info &= !BORG_VIEW;
    }

    /* Start over with the "view" array */
    st.view.view_n = 0;

    /*** Step 1 -- adjacent grids ***/

    /* Now start on the player */
    let y = st.borg.c.y;
    let x = st.borg.c.x;

    /* Assume the player grid is easily viewable */
    st.cave.grid_mut(x, y).info |= BORG_XTRA;

    /* Assume the player grid is viewable */
    borg_cave_view_hack(st, y, x);

    /*** Step 2 -- Major Diagonals ***/

    /* Limit */
    let z = full * 2 / 3;

    /* Scan south-east */
    scan_ray(st, y, x, 1, 1, z);

    /* Scan south-west */
    scan_ray(st, y, x, 1, -1, z);

    /* Scan north-east */
    scan_ray(st, y, x, -1, 1, z);

    /* Scan north-west */
    scan_ray(st, y, x, -1, -1, z);

    /*** Step 3 -- major axes ***/

    /* Scan south */
    let d = scan_ray(st, y, x, 1, 0, full);

    /* Initialize the "south strips" */
    let mut se = d;
    let mut sw = d;

    /* Scan north */
    let d = scan_ray(st, y, x, -1, 0, full);

    /* Initialize the "north strips" */
    let mut ne = d;
    let mut nw = d;

    /* Scan east */
    let d = scan_ray(st, y, x, 0, 1, full);

    /* Initialize the "east strips" */
    let mut es = d;
    let mut en = d;

    /* Scan west */
    let d = scan_ray(st, y, x, 0, -1, full);

    /* Initialize the "west strips" */
    let mut ws = d;
    let mut wn = d;

    /*** Step 4 -- Divide each "octant" into "strips" ***/

    /* Now check each "diagonal" (in parallel) */
    let mut n = 1;
    while n <= over / 2 {
        /* Acquire the "bounds" of the maximal circle */
        let mut z = over - n - n;
        if z > full - n {
            z = full - n;
        }
        while (z + n + (n >> 1)) > full {
            z -= 1;
        }

        /* Access the four diagonal grids */
        let ypn = y + n;
        let ymn = y - n;
        let xpn = x + n;
        let xmn = x - n;

        /* South strip */
        if ypn < AUTO_MAX_Y - 1 {
            /* Maximum distance */
            let m = z.min((AUTO_MAX_Y - 1) - ypn);

            /* East side */
            if (xpn < AUTO_MAX_X) && (n < se) {
                se = scan_strip(st, n, m, se, |d| {
                    (ypn + d, xpn, ypn + d - 1, xpn - 1, ypn + d - 1, xpn)
                });
            }

            /* West side */
            if (xmn >= 0) && (n < sw) {
                sw = scan_strip(st, n, m, sw, |d| {
                    (ypn + d, xmn, ypn + d - 1, xmn + 1, ypn + d - 1, xmn)
                });
            }
        }

        /* North strip */
        if ymn > 0 {
            /* Maximum distance */
            let m = z.min(ymn);

            /* East side */
            if (xpn < AUTO_MAX_X) && (n < ne) {
                ne = scan_strip(st, n, m, ne, |d| {
                    (ymn - d, xpn, ymn - d + 1, xpn - 1, ymn - d + 1, xpn)
                });
            }

            /* West side */
            if (xmn >= 0) && (n < nw) {
                nw = scan_strip(st, n, m, nw, |d| {
                    (ymn - d, xmn, ymn - d + 1, xmn + 1, ymn - d + 1, xmn)
                });
            }
        }

        /* East strip */
        if xpn < AUTO_MAX_X - 1 {
            /* Maximum distance */
            let m = z.min((AUTO_MAX_X - 1) - xpn);

            /* South side */
            if (ypn < AUTO_MAX_Y) && (n < es) {
                es = scan_strip(st, n, m, es, |d| {
                    (ypn, xpn + d, ypn - 1, xpn + d - 1, ypn, xpn + d - 1)
                });
            }

            /* North side */
            if (ymn >= 0) && (n < en) {
                en = scan_strip(st, n, m, en, |d| {
                    (ymn, xpn + d, ymn + 1, xpn + d - 1, ymn, xpn + d - 1)
                });
            }
        }

        /* West strip */
        if xmn > 0 {
            /* Maximum distance */
            let m = z.min(xmn);

            /* South side */
            if (ypn < AUTO_MAX_Y) && (n < ws) {
                ws = scan_strip(st, n, m, ws, |d| {
                    (ypn, xmn - d, ypn - 1, xmn - d + 1, ypn, xmn - d + 1)
                });
            }

            /* North side */
            if (ymn >= 0) && (n < wn) {
                wn = scan_strip(st, n, m, wn, |d| {
                    (ymn, xmn - d, ymn + 1, xmn - d + 1, ymn, xmn - d + 1)
                });
            }
        }
        n += 1;
    }

    /*** Step 5 -- Complete the algorithm ***/

    /* Update all the new grids */
    for n in 0..i32::from(st.view.view_n) {
        let y = i32::from(st.view.view_y[n as usize]);
        let x = i32::from(st.view.view_x[n as usize]);

        /* Clear the "BORG_XTRA" flag */
        st.cave.grid_mut(x, y).info &= !BORG_XTRA;
    }
}

//! Port of `borg-projection.c`: used to determine paths between two points.
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

use crate::borg_cave::BorgGrid;
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid};
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes};
use crate::borg_headers::borg_trait_h::{BI_CURHP, BI_MAXHP};
use crate::headers::list_mon_race_flags_h::RF_PASS_WALL;
use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE, FEAT_NONE, FEAT_PASS_RUBBLE};
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::engine;

/// The module's globals.
#[derive(Default)]
pub struct ProjectionState {
    /// `borg_target_loc`: current targetted location
    pub target_loc: loc,
}

fn feat_is(g: &BorgGrid, feat: u32) -> bool {
    u32::from(g.feat) == feat
}

/// A simple, fast, integer-based line-of-sight algorithm (`borg_los`).
///
/// See "los()" in "cave-view.c" for complete documentation
pub fn borg_los(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    los_with(
        y1,
        x1,
        y2,
        x2,
        || engine::square_in_bounds_fully(x1, y1),
        |y, x| borg_cave_floor_bold(st, y, x),
    )
}

/// `borg_los` with the origin check (`square_in_bounds_fully`) and the
/// floor test (`borg_cave_floor_bold(y, x)`) passed in.
fn los_with(
    y1: i32,
    x1: i32,
    y2: i32,
    x2: i32,
    origin_ok: impl FnOnce() -> bool,
    floor: impl Fn(i32, i32) -> bool,
) -> bool {
    /* Extract the offset */
    let dy = y2 - y1;
    let dx = x2 - x1;

    /* Extract the absolute offset */
    let ay = dy.abs();
    let ax = dx.abs();

    /* Handle adjacent (or identical) grids */
    if ax < 2 && ay < 2 {
        return true;
    }

    /* Paranoia -- require "safe" origin */
    if !origin_ok() {
        return false;
    }

    /* Directly South/North */
    if dx == 0 {
        if dy > 0 {
            /* South -- check for walls */
            let mut ty = y1 + 1;
            while ty < y2 {
                if !floor(ty, x1) {
                    return false;
                }
                ty += 1;
            }
        } else {
            /* North -- check for walls */
            let mut ty = y1 - 1;
            while ty > y2 {
                if !floor(ty, x1) {
                    return false;
                }
                ty -= 1;
            }
        }

        /* Assume los */
        return true;
    }

    /* Directly East/West */
    if dy == 0 {
        if dx > 0 {
            /* East -- check for walls */
            let mut tx = x1 + 1;
            while tx < x2 {
                if !floor(y1, tx) {
                    return false;
                }
                tx += 1;
            }
        } else {
            /* West -- check for walls */
            let mut tx = x1 - 1;
            while tx > x2 {
                if !floor(y1, tx) {
                    return false;
                }
                tx -= 1;
            }
        }

        /* Assume los */
        return true;
    }

    /* Extract some signs */
    let sx = if dx < 0 { -1 } else { 1 };
    let sy = if dy < 0 { -1 } else { 1 };

    if ax == 1 {
        /* Vertical "knights" */
        if ay == 2 && floor(y1 + sy, x1) {
            return true;
        }
    } else if ay == 1 {
        /* Horizontal "knights" */
        if ax == 2 && floor(y1, x1 + sx) {
            return true;
        }
    }

    /* Calculate scale factor div 2 */
    let f2 = ax * ay;

    /* Calculate scale factor */
    let f1 = f2 << 1;

    if ax >= ay {
        /* Travel horizontally */

        /* Let m = dy / dx * 2 * (dy * dx) = 2 * dy * dy */
        let mut qy = ay * ay;
        let m = qy << 1;

        let mut tx = x1 + sx;
        let mut ty;

        /* Consider the special case where slope == 1. */
        if qy == f2 {
            ty = y1 + sy;
            qy -= f1;
        } else {
            ty = y1;
        }

        /* Note (below) the case (qy == f2), where */
        /* the LOS exactly meets the corner of a tile. */
        while x2 - tx != 0 {
            if !floor(ty, tx) {
                return false;
            }

            qy += m;

            if qy < f2 {
                tx += sx;
            } else if qy > f2 {
                ty += sy;
                if !floor(ty, tx) {
                    return false;
                }
                qy -= f1;
                tx += sx;
            } else {
                ty += sy;
                qy -= f1;
                tx += sx;
            }
        }
    } else {
        /* Travel vertically */

        /* Let m = dx / dy * 2 * (dx * dy) = 2 * dx * dx */
        let mut qx = ax * ax;
        let m = qx << 1;

        let mut ty = y1 + sy;
        let mut tx;

        if qx == f2 {
            tx = x1 + sx;
            qx -= f1;
        } else {
            tx = x1;
        }

        /* Note (below) the case (qx == f2), where */
        /* the LOS exactly meets the corner of a tile. */
        while y2 - ty != 0 {
            if !floor(ty, tx) {
                return false;
            }

            qx += m;

            if qx < f2 {
                ty += sy;
            } else if qx > f2 {
                tx += sx;
                if !floor(ty, tx) {
                    return false;
                }
                qx -= f1;
                ty += sy;
            } else {
                tx += sx;
                qx -= f1;
                ty += sy;
            }
        }
    }

    /* Assume los */
    true
}

/// `z_info->max_range`
fn max_range() -> i32 {
    i32::from(engine::z_info().max_range)
}

/// `borg_fear_region[borg.c.y / 11][borg.c.x / 11] >= borg.avoidance / 20`
fn fear_region_scary(st: &mut BorgState) -> bool {
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    i32::from(st.danger.fear_region[(cy / 11) as usize][(cx / 11) as usize]) >= i32::from(st.borg.avoidance) / 20
}

/// Check the projection from (x1,y1) to (x2,y2) (`borg_projectable`).
/// Assume that there is no monster in the way.
/// HACK: We refuse to assume that unknown grids are floors
/// Adapted from "projectable()" in "spells1.c".
pub fn borg_projectable(st: &mut BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    /* Start at the initial location */
    let mut y = y1;
    let mut x = x1;

    /* Simulate the spell/missile path */
    let mut dist = 0;
    while dist <= max_range() {
        /* Get the grid */
        let ag = *st.cave.grid(x, y);

        if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 3
            || st.borg.morgoth_position
            || st.borg.mon.scary
        {
            /* Assume all unknown grids more than distance 20 from you
             * are walls--when I am wounded. This will make me more fearful
             * of the grids that are up to 19 spaces away.  I treat them as
             * regular floor grids.  Which means monsters are assumed to have
             * LOS on me.  I am also more likely to shoot into the dark grids.
             */
            if dist > 20 && feat_is(&ag, FEAT_NONE) {
                break;
            }
        } else if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2 {
            /* Assume all unknow grids more than distance 10 from you
             * are walls--when I am wounded. This will make me more fearful
             * of the grids that are up to 9 spaces away.  I treat them as
             * regular floor grids.
             */
            if dist > 10 && feat_is(&ag, FEAT_NONE) {
                break;
            }
        } else if fear_region_scary(st) {
            /* If a non-LOS monster is attacking me, then it is probably has
             * LOS to me, so do not place walls on unknown grids.  This will
             * allow me the chance to attack monsters.
             *
             * This does not work if the non-LOS monster is invisible.
             * This helps in a case like this:
             *####################          1.  Player has ESP and can sense the
             *                                  priest.
             *......@......      p          2.  Priest has cast a spell at the
             *                                  player.
             *#############                 3.  Unknown grids are between player
             *                                  and priest
             *                              4.  Borg has created regional fear
             *                                  from non-LOS priest.
             */
            if dist > max_range() && feat_is(&ag, FEAT_NONE) {
                break;
            }
        } else {
            /* Assume all unknow grids more than distance 3 from you
             * are walls.  This makes me brave and chancey.
             */
            if dist > 2 && feat_is(&ag, FEAT_NONE) {
                break;
            }
        }
        /* Never pass through walls/doors */
        if dist != 0 && !borg_cave_floor_grid(&ag) {
            break;
        }

        /* Check for arrival at "final target" */
        if x == x2 && y == y2 {
            return true;
        }

        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);
        dist += 1;
    }

    /* Assume obstruction */
    false
}

/// Check the projection from (x1,y1) to (x2,y2)
/// (`borg_offset_projectable`).
/// Assume that there is no monster in the way.
/// We refuse to assume that unknown grids are floors
/// Adapted from "projectable()" in "spells1.c".
/// This is used by borg_offset()
pub fn borg_offset_projectable(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    /* Start at the initial location */
    let mut y = y1;
    let mut x = x1;

    /* Simulate the spell/missile path */
    let mut dist = 0;
    while dist <= max_range() {
        /* Get the grid */
        let ag = st.cave.grid(x, y);

        /* Assume all unknown grids are walls. */
        if dist != 0 && feat_is(ag, FEAT_NONE) {
            break;
        }

        /* Never pass through rubble */
        if feat_is(ag, FEAT_PASS_RUBBLE) {
            break;
        }

        /* Never pass through walls/doors */
        if dist != 0 && !borg_cave_floor_grid(ag) {
            break;
        }

        /* Check for arrival at "final target" */
        if x == x2 && y == y2 {
            return true;
        }

        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);
        dist += 1;
    }

    /* Assume obstruction */
    false
}

/// Check the projection from (x1,y1) to (x2,y2) (`borg_projectable_pure`).
/// Assume that monsters in the way will stop the projection
/// We refuse to assume that unknown grids are floors
/// In fact, we assume they are walls.
/// Adapted from "projectable()" in "spells1.c".
pub fn borg_projectable_pure(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    /* Start at the initial location */
    let mut y = y1;
    let mut x = x1;

    /* Simulate the spell/missile path */
    let mut dist = 0;
    while dist <= max_range() {
        /* Get the grid */
        let ag = st.cave.grid(x, y);

        /* Assume unknown grids are walls */
        if dist != 0 && feat_is(ag, FEAT_NONE) {
            break;
        }

        /* Never pass through walls/doors */
        if dist != 0 && !borg_cave_floor_grid(ag) {
            break;
        }

        /* Check for arrival at "final target" */
        if x == x2 && y == y2 {
            return true;
        }

        /* Stop at monsters (not the caster's own grid) */
        if dist != 0 && ag.kill != 0 {
            break;
        }

        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);
        dist += 1;
    }

    /* Assume obstruction */
    false
}

/// Check the projection from (x1,y1) to (x2,y2) (`borg_projectable_dark`).
/// Assume that monsters in the way will stop the projection.
/// Assume that an unknown grid is a floor grid.
/// We want at least one unknown grid.
///
/// This routine is used mainly aiming beams of light and
/// shooting into darkness, testing the projection path.
pub fn borg_projectable_dark(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    let mut unknown = 0;

    /* Start at the initial location */
    let mut y = y1;
    let mut x = x1;

    /* Simulate the spell/missile path */
    let mut dist = 0;
    while dist <= max_range() {
        /* Get the grid */
        let ag = st.cave.grid(x, y);

        /* We want at least 1 unknown grid */
        if dist != 0 && feat_is(ag, FEAT_NONE) {
            unknown += 1;
        }

        /* Never pass through walls/doors */
        if dist != 0 && !borg_cave_floor_grid(ag) {
            break;
        }

        /* Check for arrival at "final target" */
        if x == x2 && y == y2 && unknown >= 1 {
            return true;
        }

        /* Stop at monsters */
        if ag.kill != 0 {
            break;
        }

        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);
        dist += 1;
    }

    /* Assume obstruction */
    false
}

/// Calculate "incremental motion" (`borg_inc_motion`). Used by project()
/// and shoot(). Assumes that (*y,*x) lies on the path from (y1,x1) to
/// (y2,x2).
///
/// changing this to be more like project_path
/// note that this is much slower but much more accurate
pub fn borg_inc_motion(py: &mut i32, px: &mut i32, y1: i32, x1: i32, y2: i32, x2: i32) {
    /* Extract the distance travelled */
    /* Analyze "dy" */
    let (dy, sy) = if y2 < y1 { (y1 - y2, -1) } else { (y2 - y1, 1) };

    /* Analyze "dx" */
    let (dx, sx) = if x2 < x1 { (x1 - x2, -1) } else { (x2 - x1, 1) };

    /* Paranoia -- no motion */
    if dy == 0 && dx == 0 {
        return;
    }

    /* Number of "units" in one "half" grid */
    let half = dy * dx;

    /* Number of "units" in one "full" grid */
    let full = half << 1;

    /* First step is fixed */
    if *px == x1 && *py == y1 {
        if dy > dx {
            *py += sy;
        } else if dx > dy {
            *px += sx;
        } else {
            *px += sx;
            *py += sy;
        }
        return;
    }

    if dy > dx {
        /* Move mostly vertically */
        let mut k = dy;

        /* Start at tile edge */
        let mut frac = dx * dx;

        /* Let m = ((dx/dy) * full) = (dx * dx * 2) = (frac * 2) */
        let m = frac << 1;

        /* Start */
        let mut y = y1 + sy;
        let mut x = x1;

        /* Create the projection path */
        loop {
            if x == *px && y == *py {
                k = 1;
            }

            /* Slant */
            if m != 0 {
                /* Advance (X) part 1 */
                frac += m;

                /* Horizontal change */
                if frac >= half {
                    /* Advance (X) part 2 */
                    x += sx;

                    /* Advance (X) part 3 */
                    frac -= full;
                }
            }

            /* Advance (Y) */
            y += sy;

            /* Track distance */
            k -= 1;

            if k == 0 {
                *px = x;
                *py = y;
                return;
            }
        }
    } else if dx > dy {
        /* Move mostly horizontally */

        /* Start at tile edge */
        let mut frac = dy * dy;

        /* Let m = ((dy/dx) * full) = (dy * dy * 2) = (frac * 2) */
        let m = frac << 1;

        /* Start */
        let mut y = y1;
        let mut x = x1 + sx;
        let mut k = dx;

        /* Create the projection path */
        loop {
            if x == *px && y == *py {
                k = 1;
            }

            /* Slant */
            if m != 0 {
                /* Advance (Y) part 1 */
                frac += m;

                /* Vertical change */
                if frac >= half {
                    /* Advance (Y) part 2 */
                    y += sy;

                    /* Advance (Y) part 3 */
                    frac -= full;
                }
            }

            /* Advance (X) */
            x += sx;

            /* Track distance */
            k -= 1;

            if k == 0 {
                *px = x;
                *py = y;
                return;
            }
        }
    } else {
        /* Diagonal */

        /* Start */
        let mut k = dy;
        let mut y = y1 + sy;
        let mut x = x1 + sx;

        /* Create the projection path */
        loop {
            if x == *px && y == *py {
                k = 1;
            }

            /* Advance (Y) */
            y += sy;

            /* Advance (X) */
            x += sx;

            /* Track distance */
            k -= 1;

            if k == 0 {
                *px = x;
                *py = y;
                return;
            }
        }
    }
}

/// Helper to change the old y/x/y2/x2 to loc/loc (`borg_distance`).
pub fn borg_distance(y: i32, x: i32, y2: i32, x2: i32) -> i32 {
    engine::distance(loc { x, y }, loc { x: x2, y: y2 })
}

/// Target a location (`borg_target`). Can be used alone or at
/// "Direction?" prompt.
///
/// Warning -- This will only work for locations on the current panel
pub fn borg_target(st: &mut BorgState, t: loc, require_monster: bool) -> bool {
    let kill_idx = st.cave.grid(t.x, t.y).kill;

    /* Log */
    /* Report a little bit */
    if kill_idx != 0 {
        let kill = st.kills.kills[usize::from(kill_idx)];
        let mut msg = b"# Targeting ".to_vec();
        msg.extend_from_slice(crate::borg_flow_kill::borg_race_name(i32::from(kill.r_idx)).to_bytes());
        msg.extend_from_slice(
            format!(" who has {} Hit Points ({},{}).", kill.power, t.y, t.x).as_bytes(),
        );
        borg_note_bytes(st, &msg);
    } else {
        borg_note(st, &format!("# Targeting location ({},{})", t.y, t.x));
    }

    /* Target mode */
    borg_keypress(st, u32::from(b'*'));

    /* Target a location */
    borg_keypress(st, u32::from(b'p'));

    /* Determine "path" */
    let mut x1 = st.borg.c.x;
    let mut y1 = st.borg.c.y;
    let x2 = t.x;
    let y2 = t.y;

    /* Move to the location (diagonals) */
    while y1 < y2 && x1 < x2 {
        borg_keypress(st, u32::from(b'3'));
        y1 += 1;
        x1 += 1;
    }
    while y1 < y2 && x1 > x2 {
        borg_keypress(st, u32::from(b'1'));
        y1 += 1;
        x1 -= 1;
    }
    while y1 > y2 && x1 < x2 {
        borg_keypress(st, u32::from(b'9'));
        y1 -= 1;
        x1 += 1;
    }
    while y1 > y2 && x1 > x2 {
        borg_keypress(st, u32::from(b'7'));
        y1 -= 1;
        x1 -= 1;
    }

    /* Move to the location */
    while y1 < y2 {
        borg_keypress(st, u32::from(b'2'));
        y1 += 1;
    }
    while y1 > y2 {
        borg_keypress(st, u32::from(b'8'));
        y1 -= 1;
    }
    while x1 < x2 {
        borg_keypress(st, u32::from(b'6'));
        x1 += 1;
    }
    while x1 > x2 {
        borg_keypress(st, u32::from(b'4'));
        x1 -= 1;
    }

    if require_monster {
        borg_keypress(st, u32::from(b'm'));
    }

    /* Select the target */
    borg_keypress(st, u32::from(b'5'));

    /* Carry these variables to be used on reporting spell
     * pathway
     */
    st.projection.target_loc.y = t.y;
    st.projection.target_loc.x = t.x;

    /* Success */
    true
}

/// `borg_grids[y][x].feat == FEAT_FLOOR`
fn is_floor(st: &BorgState, x: i32, y: i32) -> bool {
    feat_is(st.cave.grid(x, y), FEAT_FLOOR)
}

/// Mark spot along the target path a wall (`borg_target_unknown_wall`).
/// This will mark the unknown squares as a wall.  This might not be
/// the wall we ran into but also might be.
///
/// Warning -- This will only work for locations on the current panel
pub fn borg_target_unknown_wall(st: &mut BorgState, y: i32, x: i32) -> bool {
    let mut y_hall = false;
    let mut x_hall = false;

    borg_note(st, &format!("# Perhaps wall near targetted location ({},{})", y, x));

    /* Determine "path" */
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    let mut n_x = cx;
    let mut n_y = cy;

    /* check for 'in a hall' x axis */
    /* This check is for this: */
    /*
     *      x        x
     *    ..@.  or  .@..
     *      x        x
     *
     * 'x' being 'not a floor' and '.' being a floor.
     *
     * We would like to know if in a hall so we can place
     * the suspect wall off the hallway path.
     * like this:######x  P
     * ........@....
     * ##################
     * The shot may miss and we want the borg to guess the
     * wall to be at the X instead of first unkown grid which
     * is 3 west and 1 south of the X.
     */

    if (is_floor(st, cx, cy + 1)
        && is_floor(st, cx, cy - 1)
        && (is_floor(st, cx, cy + 2) || is_floor(st, cx, cy - 2)))
        && (!is_floor(st, cx + 1, cy) && !is_floor(st, cx - 1, cy))
    {
        x_hall = true;
    }

    /* check for 'in a hall' y axis.
     * Again, we want to place the suspected wall off our
     * hallway.
     */
    if (is_floor(st, cx + 1, cy)
        && is_floor(st, cx - 1, cy)
        && (is_floor(st, cx + 2, cy) || is_floor(st, cx - 2, cy)))
        && (!is_floor(st, cx, cy + 1) && !is_floor(st, cx, cy - 1))
    {
        y_hall = true;
    }

    loop {
        let kill_idx = st.cave.grid(n_x, n_y).kill;
        let kill = st.kills.kills[usize::from(kill_idx)];

        if engine::race_flag_has(usize::from(kill.r_idx), RF_PASS_WALL) {
            borg_note(
                st,
                &format!(
                    "# Guessing wall ({},{}) under ghostly target ({},{})",
                    n_y, n_x, n_y, n_x
                ),
            );
            st.cave.grid_mut(n_x, n_y).feat = FEAT_GRANITE as u8;
            return true; /* not sure... should we return here? */
        }

        if feat_is(st.cave.grid(n_x, n_y), FEAT_NONE)
            && (n_y != st.borg.c.y || !y_hall)
            && (n_x != st.borg.c.x || !x_hall)
        {
            borg_note(
                st,
                &format!("# Guessing wall ({},{}) near target ({},{})", n_y, n_x, y, x),
            );
            st.cave.grid_mut(n_x, n_y).feat = FEAT_GRANITE as u8;
            return true; /* not sure... should we return here?
                          maybe should mark ALL unknowns in path... */
        }

        /* Pathway found the target. */
        if n_x == x && n_y == y {
            /* end of the pathway */
            let (cy, cx) = (st.borg.c.y, st.borg.c.x);
            borg_inc_motion(&mut n_y, &mut n_x, y, x, cy, cx);
            borg_note(
                st,
                &format!("# Guessing wall ({},{}) near target ({},{})", n_y, n_x, y, x),
            );
            st.cave.grid_mut(n_x, n_y).feat = FEAT_GRANITE as u8;
            return true;
        }

        /* Calculate the new location */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        borg_inc_motion(&mut n_y, &mut n_x, cy, cx, y, x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Follow `borg_inc_motion` from (y1, x1) for `n` steps.
    fn path(y1: i32, x1: i32, y2: i32, x2: i32, n: usize) -> Vec<(i32, i32)> {
        let (mut y, mut x) = (y1, x1);
        let mut out = Vec::new();
        for _ in 0..n {
            borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);
            out.push((y, x));
        }
        out
    }

    #[test]
    fn inc_motion_straight_and_diagonal() {
        assert_eq!(path(5, 5, 5, 8, 3), vec![(5, 6), (5, 7), (5, 8)]);
        assert_eq!(path(5, 5, 2, 5, 3), vec![(4, 5), (3, 5), (2, 5)]);
        assert_eq!(path(5, 5, 8, 2, 3), vec![(6, 4), (7, 3), (8, 2)]);
        /* No motion */
        assert_eq!(path(5, 5, 5, 5, 1), vec![(5, 5)]);
    }

    #[test]
    fn inc_motion_slopes() {
        /* dx = 4, dy = 1: half = 4, frac starts 1, m = 2 */
        assert_eq!(path(10, 10, 11, 14, 4), vec![(10, 11), (10, 12), (11, 13), (11, 14)]);
        /* dy = 5, dx = 2: half = 10, frac starts 4, m = 8 */
        assert_eq!(
            path(10, 10, 5, 12, 5),
            vec![(9, 10), (8, 11), (7, 11), (6, 11), (5, 12)]
        );
        /* One step past the target the replay runs out of steps and the
         * position stays put (as in C) */
        assert_eq!(path(0, 0, 0, 2, 4), vec![(0, 1), (0, 2), (0, 3), (0, 3)]);
    }

    /// A hand-made map: '#' is a wall, anything else a floor.
    fn map_floor(rows: &'static [&'static str]) -> impl Fn(i32, i32) -> bool {
        move |y, x| {
            y >= 0
                && x >= 0
                && rows
                    .get(y as usize)
                    .and_then(|r| r.as_bytes().get(x as usize))
                    .is_some_and(|&c| c != b'#')
        }
    }

    const MAP: &[&str] = &[
        "##########", //
        "#........#", //
        "#..#.....#", //
        "#........#", //
        "#.....#..#", //
        "##########",
    ];

    #[test]
    fn los_on_hand_made_map() {
        let f = map_floor(MAP);
        let los = |y1, x1, y2, x2| los_with(y1, x1, y2, x2, || true, &f);
        /* Adjacent, even into walls */
        assert!(los(1, 1, 0, 0));
        /* Straight lines */
        assert!(los(1, 1, 1, 8));
        assert!(!los(2, 1, 2, 8));
        assert!(los(1, 3, 3, 3) == f(2, 3));
        assert!(!los(1, 3, 4, 3));
        /* Knight moves: open via the straight neighbour */
        assert!(los(1, 2, 3, 3));
        /* Shallow: open, blocked by (2,3), blocked by (4,6) after the
         * exact-corner case qy == f2 */
        assert!(los(1, 4, 3, 9));
        assert!(!los(1, 1, 4, 8));
        assert!(!los(3, 1, 4, 8));
        /* Steep: open through the corner case, blocked by (2,3) */
        assert!(los(1, 1, 4, 2));
        assert!(!los(1, 4, 4, 2));
        /* Slope == 1 */
        assert!(!los(1, 2, 3, 4));
        assert!(!los(1, 4, 4, 1));
        assert!(los(1, 5, 4, 2));
        /* Bad origin (adjacent grids never ask) */
        assert!(!los_with(1, 4, 3, 9, || false, &f));
        assert!(los_with(1, 1, 2, 2, || false, &f));
    }
}

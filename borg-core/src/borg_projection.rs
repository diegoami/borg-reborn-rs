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
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_FLOOR, FEAT_GRANITE, FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN,
    FEAT_PASS_RUBBLE,
};
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

/// borg-reborn-rs (#917): `borg_los` for sight, the view's line of sight
/// under `view_rubble_blocks_sight` = 1: C's walk test (`borg_cave_floor_bold`)
/// without passable rubble, which has no `LOS` flag
/// (lib/gamedata/terrain.txt:346-354), so the engine's `los()`
/// (src/cave-view.c:85-248, `square_isprojectable`) stops at it.
pub(crate) fn borg_los_sight(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    los_with(
        y1,
        x1,
        y2,
        x2,
        || engine::square_in_bounds_fully(x1, y1),
        |y, x| borg_cave_floor_bold(st, y, x) && !feat_is(st.cave.grid(x, y), FEAT_PASS_RUBBLE),
    )
}

/// Engine-style LOS on the Borg's known map. Both light and arc footprints
/// use `los()` (src/cave-view.c:los:93-241, src/project.c:project:856-863).
/// The terrain test is src/cave-square.c:square_isprojectable:671-674.
/// Passable rubble lacks
/// the PROJECT flag (lib/gamedata/terrain.txt), so it blocks intermediate rays.
/// Unknown terrain is treated conservatively.
pub(crate) fn borg_los_projectable(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> bool {
    los_with(
        y1,
        x1,
        y2,
        x2,
        || engine::square_in_bounds_fully(x1, y1),
        |y, x| {
            engine::square_in_bounds_fully(x, y) && {
                let g = st.cave.grid(x, y);
                known_projectable(u32::from(g.feat), g.trap, engine::feat_is_shop(i32::from(g.feat)))
            }
        },
    )
}

/// Whether a known grid allows a projection to pass through it.
fn known_projectable(feat: u32, trap: bool, shop: bool) -> bool {
    trap || shop || [FEAT_FLOOR, FEAT_OPEN, FEAT_BROKEN, FEAT_LESS, FEAT_MORE, FEAT_LAVA].contains(&feat)
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
/// Adapted from "projectable()" in "spells1.c". This forecast uses the
/// Borg's floor predicate, which allows known passable rubble. The engine
/// projectile path checks PROJECT terrain instead.
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
///
/// borg-reborn-rs (#929): with `skip_player` (`update_miss_wall_guess` =
/// 1) no grid is written under the Borg: the walk starts at its first step,
/// and the step back from a reached target is not written when it is the
/// Borg's grid (an adjacent target). C writes granite there.
pub fn borg_target_unknown_wall(st: &mut BorgState, y: i32, x: i32, skip_player: bool) -> bool {
    let mut y_hall = false;
    let mut x_hall = false;

    borg_note(st, &format!("# Perhaps wall near targetted location ({},{})", y, x));

    /* Determine "path" */
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);

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

    let (c, t) = (loc { x: cx, y: cy }, loc { x, y });
    let guess = wall_guess_walk(
        c,
        t,
        (x_hall, y_hall),
        skip_player,
        |g| {
            let kill = st.kills.kills[usize::from(st.cave.grid(g.x, g.y).kill)];
            engine::race_flag_has(usize::from(kill.r_idx), RF_PASS_WALL)
        },
        |g| feat_is(st.cave.grid(g.x, g.y), FEAT_NONE),
    );
    match guess {
        WallGuess::Ghost(g) => {
            borg_note(
                st,
                &format!("# Guessing wall ({},{}) under ghostly target ({},{})", g.y, g.x, g.y, g.x),
            );
        }
        /* not sure... should we return here? maybe should mark ALL
         * unknowns in path... */
        WallGuess::Unknown(g) | WallGuess::BeforeTarget(g) => {
            borg_note(st, &format!("# Guessing wall ({},{}) near target ({},{})", g.y, g.x, y, x));
        }
        WallGuess::NextToBorg => {
            borg_note(st, "# No wall to guess: the target is next to the Borg (#929)");
        }
    }
    if let Some(g) = wall_guess_grid(&guess) {
        st.cave.grid_mut(g.x, g.y).feat = FEAT_GRANITE as u8;
    }
    /* borg-reborn-rs (#929): `NextToBorg` returns true as well, so the
     * caller allows the next shot (successful_target 2), as C does after
     * every guess; false would leave -11 after a spell, a lockout until
     * the Borg moves (missed_before clears only -12 or less). */
    true
}

/// The grid `borg_target_unknown_wall` writes granite on (#929): none for
/// `NextToBorg`.
pub(crate) fn wall_guess_grid(guess: &WallGuess) -> Option<loc> {
    match *guess {
        WallGuess::Ghost(g) | WallGuess::Unknown(g) | WallGuess::BeforeTarget(g) => Some(g),
        WallGuess::NextToBorg => None,
    }
}

/// What `borg_target_unknown_wall` guesses (#929).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WallGuess {
    /// A wall-passing monster's grid ("under ghostly target").
    Ghost(loc),
    /// The first unknown grid of the path (off the hallway).
    Unknown(loc),
    /// No unknown grid: the step back from the target.
    BeforeTarget(loc),
    /// `skip_player` only: no unknown grid before the target, and the step
    /// back from the target is the Borg's grid. Nothing is written.
    NextToBorg,
}

/// The walk of `borg_target_unknown_wall` from the Borg's grid `c` to the
/// target `t`, without the map writes: `ghost` and `unknown` read the
/// Borg's map, `halls` is (x_hall, y_hall). C's walk starts at `c`, so an
/// unknown grid under the Borg, or the step back from an adjacent target,
/// is guessed there. borg-reborn-rs (#929): `skip_player` starts at the
/// first step, never returns `c`, and never guesses the target's own grid
/// as unknown (the monster stood there; for an adjacent target in the dark
/// the first step is the target, and granite there would block the next
/// shot at it, `borg_projectable`).
pub(crate) fn wall_guess_walk(
    c: loc,
    t: loc,
    (x_hall, y_hall): (bool, bool),
    skip_player: bool,
    ghost: impl Fn(loc) -> bool,
    unknown: impl Fn(loc) -> bool,
) -> WallGuess {
    let (mut n_y, mut n_x) = (c.y, c.x);
    if skip_player && (n_x != t.x || n_y != t.y) {
        borg_inc_motion(&mut n_y, &mut n_x, c.y, c.x, t.y, t.x);
    }

    loop {
        let n = loc { x: n_x, y: n_y };
        if ghost(n) {
            return WallGuess::Ghost(n);
        }

        let at_target = n_x == t.x && n_y == t.y;
        if unknown(n) && !(skip_player && at_target) && (n_y != c.y || !y_hall) && (n_x != c.x || !x_hall) {
            return WallGuess::Unknown(n);
        }

        /* Pathway found the target. */
        if at_target {
            /* end of the pathway */
            borg_inc_motion(&mut n_y, &mut n_x, t.y, t.x, c.y, c.x);
            if skip_player && n_x == c.x && n_y == c.y {
                return WallGuess::NextToBorg;
            }
            return WallGuess::BeforeTarget(loc { x: n_x, y: n_y });
        }

        /* Calculate the new location */
        borg_inc_motion(&mut n_y, &mut n_x, c.y, c.x, t.y, t.x);
    }
}

#[cfg(test)]
mod tests {
    /// #884 (re-review of #894): from the player at (0,0) to (2,1), with
    /// passable rubble at (1,0) and floor everywhere else. The walk test takes
    /// the knight's shortcut through the rubble; light, as in the engine's
    /// los(), does not pass it.
    #[test]
    fn light_does_not_pass_rubble_the_walk_test_crosses() {
        use crate::headers::list_terrain_h::{FEAT_CLOSED, FEAT_PASS_RUBBLE, FEAT_RUBBLE};
        let feat = |y: i32, x: i32| if (y, x) == (1, 0) { FEAT_PASS_RUBBLE } else { FEAT_FLOOR };
        let walk = |y, x| feat(y, x) == FEAT_FLOOR || feat(y, x) == FEAT_PASS_RUBBLE;
        let light = |y, x| known_projectable(feat(y, x), false, false);
        assert!(los_with(0, 0, 2, 1, || true, walk));
        assert!(!los_with(0, 0, 2, 1, || true, light));
        // Floor all the way: both see.
        assert!(los_with(0, 0, 2, 1, || true, |_, _| known_projectable(FEAT_FLOOR, false, false)));
        assert!(!known_projectable(FEAT_RUBBLE, false, false));
        assert!(!known_projectable(FEAT_CLOSED, false, false));
        assert!(!known_projectable(FEAT_NONE, false, false));
        assert!(known_projectable(FEAT_NONE, true, false));
        assert!(known_projectable(FEAT_OPEN, false, false) && known_projectable(FEAT_LESS, false, false));
    }

    #[test]
    fn acid_spray_rubble_blocks_only_the_dog_behind_it() {
        let player = (56, 133);
        let blocked_dog = (56, 126);
        let clear_dog = (54, 126);
        // Both targets are within 30 degrees of due west.
        let projectable = |y, x| {
            let feat = if (y, x) == (56, 127) { FEAT_PASS_RUBBLE } else { FEAT_FLOOR };
            known_projectable(feat, false, false)
        };
        let reachable = |(y, x)| los_with(player.0, player.1, y, x, || true, projectable);
        assert_eq!([blocked_dog, clear_dog].into_iter().filter(|&dog| reachable(dog)).count(), 1);
        assert!(reachable(clear_dog));
        assert!(!reachable(blocked_dog));
    }

    use super::*;

    /// #929: Fang next to the Borg, as in Paladin 1194051723 (Borg (50,87),
    /// target (51,88)) and Druid 1024822556 (Borg (39,112), target
    /// (39,111)). C guesses the Borg's own grid after a miss; with
    /// `skip_player` nothing is guessed. An unknown grid under the Borg (the
    /// Necromancer with light 0, 798380952) is skipped as well.
    #[test]
    fn wall_guess_never_lands_on_the_borg() {
        let at = |y, x| loc { x, y };
        let never = |_: loc| false;
        for (c, t) in [(at(50, 87), at(51, 88)), (at(39, 112), at(39, 111)), (at(49, 88), at(50, 87))] {
            assert_eq!(wall_guess_walk(c, t, (false, false), false, never, never), WallGuess::BeforeTarget(c));
            assert_eq!(wall_guess_walk(c, t, (false, false), true, never, never), WallGuess::NextToBorg);
            // the Borg's grid unknown: C guesses it, the guard looks past it
            let dark_borg = move |g: loc| g == c;
            assert_eq!(wall_guess_walk(c, t, (false, false), false, never, dark_borg), WallGuess::Unknown(c));
            assert_eq!(wall_guess_walk(c, t, (false, false), true, never, dark_borg), WallGuess::NextToBorg);
        }
        // A far target: both find the same first unknown grid on the path,
        // and with none, the same grid before the target, never the Borg's.
        let (c, t) = (at(10, 10), at(10, 14));
        let dark = |g: loc| g == at(10, 12);
        assert_eq!(wall_guess_walk(c, t, (false, false), false, never, dark), WallGuess::Unknown(at(10, 12)));
        assert_eq!(wall_guess_walk(c, t, (false, false), true, never, dark), WallGuess::Unknown(at(10, 12)));
        for skip in [false, true] {
            assert_eq!(wall_guess_walk(c, t, (false, false), skip, never, never), WallGuess::BeforeTarget(at(10, 13)));
        }
        // A wall-passing monster on the target is guessed by both.
        let ghost = |g: loc| g == t;
        assert_eq!(wall_guess_walk(c, t, (false, false), true, ghost, never), WallGuess::Ghost(t));
        // The written grid: never the Borg's with the guard (the caller
        // writes `wall_guess_grid`).
        assert_eq!(wall_guess_grid(&WallGuess::NextToBorg), None);
        assert_eq!(wall_guess_grid(&WallGuess::Unknown(t)), Some(t));
    }

    /// Review of #1005, finding 1: in the dark (light 0) the Borg's grid
    /// and the adjacent target's grid are both unknown. C guesses the
    /// Borg's grid; the guard must not move the guess onto the monster's
    /// grid. A far target whose own grid is unknown: C guesses the target,
    /// the guard the step before it.
    #[test]
    fn wall_guess_never_lands_on_the_target_with_the_guard() {
        let at = |y, x| loc { x, y };
        let never = |_: loc| false;
        let dark = |_: loc| true;
        for (c, t) in [(at(50, 87), at(51, 88)), (at(39, 112), at(39, 111))] {
            assert_eq!(wall_guess_walk(c, t, (false, false), false, never, dark), WallGuess::Unknown(c));
            let guess = wall_guess_walk(c, t, (false, false), true, never, dark);
            assert_eq!(guess, WallGuess::NextToBorg);
            assert_eq!(wall_guess_grid(&guess), None);
        }
        let (c, t) = (at(10, 10), at(10, 14));
        let dark_target = move |g: loc| g == t;
        assert_eq!(wall_guess_walk(c, t, (false, false), false, never, dark_target), WallGuess::Unknown(t));
        assert_eq!(wall_guess_walk(c, t, (false, false), true, never, dark_target), WallGuess::BeforeTarget(at(10, 13)));
        // Every grid unknown: the guard guesses the first step, never c or t.
        assert_eq!(wall_guess_walk(c, t, (false, false), true, never, dark), WallGuess::Unknown(at(10, 11)));
    }

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

//! Port of `borg-cave-light.c`: handle lighted squares in the cave.
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
use crate::borg_cave_util::borg_cave_floor_bold;
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_headers::borg_trait_h::{BI_CLASS, BI_LIGHT};
use crate::params::{value, Param};
use crate::state::BorgState;
use crate::engine;

/// Size of the light/glow arrays (`AUTO_LIGHT_MAX`).
pub const AUTO_LIGHT_MAX: usize = 1536;

/// Grid info flag: the grid is lit by the player's light (`BORG_LIGHT`,
/// borg-cave.h).
pub const BORG_LIGHT: u16 = 0x10;

/// The sets of grids marked as `BORG_LIGHT` (`borg_light_n`,
/// `borg_light_x`, `borg_light_y`) and as `BORG_GLOW` (`borg_glow_n`,
/// `borg_glow_x`, `borg_glow_y`; filled by `borg_flow_light()` in
/// borg-flow-misc.c). The player grid is always entry 0 of the light set
/// after `borg_update_light()` when the player has a light.
pub struct LightState {
    pub(crate) detected_entries: std::collections::BTreeSet<(i32, i32)>,
    /// `borg_light_n`
    pub light_n: i16,
    /// `borg_light_x`
    pub light_x: Vec<u8>,
    /// `borg_light_y`
    pub light_y: Vec<u8>,
    /// `borg_glow_n`
    pub glow_n: i16,
    /// `borg_glow_x`
    pub glow_x: Vec<u8>,
    /// `borg_glow_y`
    pub glow_y: Vec<u8>,
}

impl Default for LightState {
    fn default() -> Self {
        LightState {
            detected_entries: std::collections::BTreeSet::new(),
            light_n: 0,
            light_x: vec![0; AUTO_LIGHT_MAX],
            light_y: vec![0; AUTO_LIGHT_MAX],
            glow_n: 0,
            glow_x: vec![0; AUTO_LIGHT_MAX],
            glow_y: vec![0; AUTO_LIGHT_MAX],
        }
    }
}

/// Add a grid to the "lite" array (the `borg_cave_light_hack` macro).
///
/// Never called for illegal grids, or when the array is full. (Unlike the
/// C comment claims, a grid can be added twice; see c-bugs.md.)
fn borg_cave_light_hack(st: &mut BorgState, y: i32, x: i32) {
    st.cave.grid_mut(x, y).info |= BORG_LIGHT;
    let n = st.light.light_n as usize;
    st.light.light_y[n] = y as u8;
    st.light.light_x[n] = x as u8;
    st.light.light_n += 1;
}

/// `INVEN_LIGHT` (`z_info->pack_size + 5`, the light slot of `borg_items`).
fn inven_light() -> usize {
    usize::from(engine::z_info().pack_size) + 5
}

/// Update the set of grids "illuminated" by the player's lite
/// (`borg_update_light`).
///
/// See "update_LIGHT" in "cave.c" for complete documentation
///
/// It is very important that the "player grid" be the first grid in the
/// array of "BORG_LIGHT" grids, since this is assumed in several places.
/// #877: the radius the Borg marks as lit by its own light. The C Borg (and
/// switch 0) uses `BI_LIGHT`, the engine's `cur_light`, as the radius; the
/// engine lights `radius = ABS(light) - 1` (src/cave-view.c:658, add_light
/// :616-646), so a Wooden Torch (`LIGHT_2`, lib/gamedata/object.txt:241)
/// lights distance 1 only. Switch 1 takes the engine's radius, except for a
/// necromancer at `cur_light <= 1`, who sees by unlight
/// (src/cave-view.c:777-779) and keeps C's radius.
pub(crate) fn light_radius(light: i32, class: i32, engine: i32) -> i32 {
    const CLASS_NECROMANCER: i32 = 4;
    if engine == 0 || (class == CLASS_NECROMANCER && light <= 1) {
        return light;
    }
    (light - 1).max(0)
}

pub fn borg_update_light(st: &mut BorgState) {
    /*** Clear old grids ***/

    /* Clear them all */
    for i in 0..i32::from(st.light.light_n) {
        let y = i32::from(st.light.light_y[i as usize]);
        let x = i32::from(st.light.light_x[i as usize]);

        /* Mark the grid as not "lite" */
        st.cave.grid_mut(x, y).info &= !BORG_LIGHT;
    }

    /* None left */
    st.light.light_n = 0;

    /* Player has no lite */
    if st.items.items[inven_light()].iqty == 0 {
        return;
    }

    /*** Collect the new "lite" grids ***/

    let cy = st.borg.c.y;
    let cx = st.borg.c.x;
    let light = light_radius(st.borg.tr(BI_LIGHT), st.borg.tr(BI_CLASS), value(Param::cave_light_engine_radius));
    /* #884: with the engine's radius, the grids lit are also the engine's
     * shape: distance() <= radius (see engine_shape) */
    let engine_shape = value(Param::cave_light_engine_radius) != 0;

    /* Player grid */
    borg_cave_light_hack(st, cy, cx);

    /* Radius 1 -- torch radius */
    if light >= 1 {
        /* Adjacent grid */
        borg_cave_light_hack(st, cy + 1, cx);
        borg_cave_light_hack(st, cy - 1, cx);
        borg_cave_light_hack(st, cy, cx + 1);
        borg_cave_light_hack(st, cy, cx - 1);

        /* Diagonal grids */
        borg_cave_light_hack(st, cy + 1, cx + 1);
        borg_cave_light_hack(st, cy + 1, cx - 1);
        borg_cave_light_hack(st, cy - 1, cx + 1);
        borg_cave_light_hack(st, cy - 1, cx - 1);
    }

    /* Radius 2 -- lantern radius */
    if engine_shape && light >= 2 && cy + 2 < AUTO_MAX_Y && cy - 2 > 0 && cx + 2 < AUTO_MAX_X && cx - 2 > 0 {
        /* #884: the grids at distance 2, (2,0) and (2,1), behind an open
         * orthogonal grid (C's line-of-sight proxy) */
        for ((gy, gx), grids) in RADIUS_2 {
            if borg_cave_floor_bold(st, cy + gy, cx + gx) {
                for (dy, dx) in grids {
                    borg_cave_light_hack(st, cy + dy, cx + dx);
                }
            }
        }
    } else if light >= 2 && cy + 2 < AUTO_MAX_Y && cy - 2 > 0 && cx + 2 < AUTO_MAX_X && cx - 2 > 0 {
        /* South of the player */
        if borg_cave_floor_bold(st, cy + 2, cx) {
            borg_cave_light_hack(st, cy + 2, cx);
            borg_cave_light_hack(st, cy + 2, cx + 2);
            borg_cave_light_hack(st, cy + 2, cx - 2);
        }

        /* North of the player */
        if borg_cave_floor_bold(st, cy - 2, cx) {
            borg_cave_light_hack(st, cy - 2, cx);
            borg_cave_light_hack(st, cy - 2, cx + 2);
            borg_cave_light_hack(st, cy - 2, cx - 2);
        }

        /* East of the player */
        if borg_cave_floor_bold(st, cy, cx + 2) {
            borg_cave_light_hack(st, cy, cx + 2);
            borg_cave_light_hack(st, cy + 1, cx + 2);
            borg_cave_light_hack(st, cy - 1, cx + 2);
        }

        /* West of the player */
        if borg_cave_floor_bold(st, cy, cx - 2) {
            borg_cave_light_hack(st, cy, cx - 2);
            borg_cave_light_hack(st, cy + 1, cx - 2);
            borg_cave_light_hack(st, cy - 1, cx - 2);
        }
    }

    /* Radius 3+ -- artifact radius */
    if light >= 3 && cy + 3 < AUTO_MAX_Y && cy - 3 > 0 && cx + 3 < AUTO_MAX_X && cx - 3 > 0 {
        /* Maximal radius */
        /* Paranoia -- see "LITE_MAX" */
        let p = light.min(5);

        /* South-East of the player */
        /* #884: C's (3,3) corners are at distance 4; not with the engine's
         * shape (the box below lights them at radius 4+ when viewable) */
        if !engine_shape && borg_cave_floor_bold(st, cy + 3, cx + 3) {
            borg_cave_light_hack(st, cy + 3, cx + 3);
        }

        /* South-West of the player */
        if !engine_shape && borg_cave_floor_bold(st, cy + 3, cx - 3) {
            borg_cave_light_hack(st, cy + 3, cx - 3);
        }

        /* North-East of the player */
        if !engine_shape && borg_cave_floor_bold(st, cy - 3, cx + 3) {
            borg_cave_light_hack(st, cy - 3, cx + 3);
        }

        /* North-West of the player */
        if !engine_shape && borg_cave_floor_bold(st, cy - 3, cx - 3) {
            borg_cave_light_hack(st, cy - 3, cx - 3);
        }

        /* Maximal north */
        let min_y = (cy - p).max(0);

        /* Maximal south */
        let max_y = (cy + p).min(AUTO_MAX_Y - 1);

        /* Maximal west */
        let min_x = (cx - p).max(0);

        /* Maximal east */
        let max_x = (cx + p).min(AUTO_MAX_X - 1);

        /* Scan the maximal box */
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dy = (cy - y).abs();
                let dx = (cx - x).abs();

                /* Approximate the distance "as the bird flies" (straight-line).
                 * Ignores obstacles and does not follow walkable paths like
                 * pathfinding code elsewhere.
                 */
                let d = if dy > dx { dy + (dx >> 1) } else { dx + (dy >> 1) };

                /* Skip the "central" grids (above): C's 5x5 square, or
                 * (#884, engine shape) distance <= 2, so (2,2) at distance
                 * 3 is lit here */
                if if engine_shape { d <= 2 } else { dy <= 2 && dx <= 2 } {
                    continue;
                }

                /* Skip distant grids */
                if d > p {
                    continue;
                }

                /* Viewable, nearby, grids get "torch lit" */
                if st.cave.grid(x, y).info & BORG_VIEW != 0 {
                    /* This grid is "torch lit" */
                    borg_cave_light_hack(st, y, x);
                }
            }
        }
    }
}

/// #884: the Borg's own light radius under `cave_light_engine_radius`, for
/// the code that looks at or just beyond the edge of its light (the dark
/// flow, the dark frontier, Light Beam). Switch 0: `BI_LIGHT`, C's.
pub(crate) fn own_light_radius(st: &BorgState) -> i32 {
    light_radius(st.borg.tr(BI_LIGHT), st.borg.tr(BI_CLASS), value(Param::cave_light_engine_radius))
}

/// #884: the radius-2 grids of the engine's shape, `(dy, dx)` from the
/// player: each orthogonal grid at distance 2 (the gate, which must be open)
/// with the two grids beside it, (2,1) at `distance()` 2. C lights (2,2)
/// north and south, at distance 3, and misses (2,1).
const RADIUS_2: [((i32, i32), [(i32, i32); 3]); 4] = [
    ((2, 0), [(2, 0), (2, 1), (2, -1)]),
    ((-2, 0), [(-2, 0), (-2, 1), (-2, -1)]),
    ((0, 2), [(0, 2), (1, 2), (-1, 2)]),
    ((0, -2), [(0, -2), (1, -2), (-1, -2)]),
];

#[cfg(test)]
mod tests_884 {
    use super::RADIUS_2;

    /// The game's `distance()` (engine src/cave-view.c:38).
    fn distance(dy: i32, dx: i32) -> i32 {
        let (ay, ax) = (dy.abs(), dx.abs());
        if ay > ax { ay + (ax >> 1) } else { ax + (ay >> 1) }
    }

    #[test]
    fn radius_two_grids_are_the_distance_two_grids() {
        let mut lit: Vec<(i32, i32)> =
            RADIUS_2.iter().flat_map(|(_, grids)| grids.iter().copied()).collect();
        lit.sort_unstable();
        let mut want: Vec<(i32, i32)> = (-2..=2)
            .flat_map(|dy| (-2..=2).map(move |dx| (dy, dx)))
            .filter(|&(dy, dx)| distance(dy, dx) == 2)
            .collect();
        want.sort_unstable();
        assert_eq!(lit, want);
        for (gate, grids) in RADIUS_2 {
            assert!(grids.contains(&gate));
        }
    }
}

#[cfg(test)]
mod tests_877 {
    use super::light_radius;

    #[test]
    fn engine_radius_is_light_minus_one() {
        // switch 0: C's radius
        assert_eq!(light_radius(2, 0, 0), 2);
        assert_eq!(light_radius(3, 0, 0), 3);
        // switch 1: torch (2) -> 1, lantern/LIGHT_3 (3) -> 2, +1 light -> 0
        assert_eq!(light_radius(2, 0, 1), 1);
        assert_eq!(light_radius(3, 0, 1), 2);
        assert_eq!(light_radius(1, 0, 1), 0);
        assert_eq!(light_radius(0, 0, 1), 0);
        // necromancer by unlight keeps C's radius; with a real light it does not
        assert_eq!(light_radius(1, 4, 1), 1);
        assert_eq!(light_radius(3, 4, 1), 2);
    }
}

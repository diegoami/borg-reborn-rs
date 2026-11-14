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
use crate::borg_headers::borg_trait_h::BI_LIGHT;
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
    let light = st.borg.tr(BI_LIGHT);

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
    if light >= 2 && cy + 2 < AUTO_MAX_Y && cy - 2 > 0 && cx + 2 < AUTO_MAX_X && cx - 2 > 0 {
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
        if borg_cave_floor_bold(st, cy + 3, cx + 3) {
            borg_cave_light_hack(st, cy + 3, cx + 3);
        }

        /* South-West of the player */
        if borg_cave_floor_bold(st, cy + 3, cx - 3) {
            borg_cave_light_hack(st, cy + 3, cx - 3);
        }

        /* North-East of the player */
        if borg_cave_floor_bold(st, cy - 3, cx + 3) {
            borg_cave_light_hack(st, cy - 3, cx + 3);
        }

        /* North-West of the player */
        if borg_cave_floor_bold(st, cy - 3, cx - 3) {
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

                /* Skip the "central" grids (above) */
                if dy <= 2 && dx <= 2 {
                    continue;
                }

                /* Approximate the distance "as the bird flies" (straight-line).
                 * Ignores obstacles and does not follow walkable paths like
                 * pathfinding code elsewhere.
                 */
                let d = if dy > dx { dy + (dx >> 1) } else { dx + (dy >> 1) };

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

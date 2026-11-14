//! Port of `borg-cave-util.c`: grid and panel helpers.
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
use crate::engine;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_FLOOR, FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE, FEAT_OPEN,
    FEAT_PASS_RUBBLE, FEAT_PERM,
};
use crate::state::BorgState;

fn feat_is(g: &BorgGrid, feats: &[u32]) -> bool {
    feats.contains(&u32::from(g.feat))
}

/// Is the grid at (x, y) a "floor" the Borg can walk on
/// (`borg_cave_floor_bold`)?
pub fn borg_cave_floor_bold(st: &BorgState, y: i32, x: i32) -> bool {
    if engine::square_in_bounds_fully(x, y) {
        let g = st.cave.grid(x, y);
        if u32::from(g.feat) == FEAT_FLOOR
            || g.trap
            || feat_is(g, &[FEAT_LESS, FEAT_MORE, FEAT_BROKEN, FEAT_PASS_RUBBLE, FEAT_OPEN])
        {
            return true;
        }
    }
    false
}

/// Is the grid a "floor" (unknown grids count) (`borg_cave_floor_grid`)?
pub fn borg_cave_floor_grid(ag: &BorgGrid) -> bool {
    feat_is(
        ag,
        &[
            FEAT_NONE,
            FEAT_FLOOR,
            FEAT_OPEN,
            FEAT_MORE,
            FEAT_LESS,
            FEAT_BROKEN,
            FEAT_PASS_RUBBLE,
            FEAT_LAVA,
        ],
    )
}

/// Is the grid protected from a monster stepping in
/// (`borg_feature_protected`)?
pub fn borg_feature_protected(ag: &BorgGrid) -> bool {
    let f = u32::from(ag.feat);
    ag.glyph || ag.kill != 0 || (FEAT_CLOSED..=FEAT_PERM).contains(&f)
}

/// Height of the map panel in grids (`borg_panel_hgt`).
pub fn borg_panel_hgt() -> i32 {
    let t = engine::term_geometry();
    let panel_hgt = if t.is_main {
        (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height
    } else {
        t.hgt / t.tile_height
    };
    panel_hgt.max(1)
}

/// Width of the map panel in grids (`borg_panel_wid`).
pub fn borg_panel_wid() -> i32 {
    let t = engine::term_geometry();
    let panel_wid = if t.is_main {
        (t.wid - t.col_map - 1) / t.tile_width
    } else {
        t.wid / t.tile_width
    };
    panel_wid.max(1)
}

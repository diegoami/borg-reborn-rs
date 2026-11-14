//! Port of `borg-cave.c`: the Borg's map of the dungeon.
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

use crate::state::BorgState;
use crate::engine;

/// Dungeon size the Borg was written for (must match the game).
pub const DUNGEON_WID: i32 = 198;
pub const DUNGEON_HGT: i32 = 66;
pub const AUTO_MAX_X: i32 = DUNGEON_WID;
pub const AUTO_MAX_Y: i32 = DUNGEON_HGT;

/// What the Borg knows about one grid (`borg_grid`).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct BorgGrid {
    pub feat: u8,
    pub info: u16,
    pub trap: bool,
    pub glyph: bool,
    pub web: bool,
    pub store: u8,
    pub take: u8,
    pub kill: u8,
    pub xtra: u8,
}

/// The map (`borg_grids`). In C, an array of `AUTO_MAX_Y` row pointers,
/// each allocated (zeroed) by `borg_init_cave()`.
#[derive(Default)]
pub struct CaveState {
    grids: Vec<BorgGrid>,
}

impl CaveState {
    fn index(x: i32, y: i32) -> usize {
        assert!((0..AUTO_MAX_X).contains(&x) && (0..AUTO_MAX_Y).contains(&y));
        (y * AUTO_MAX_X + x) as usize
    }

    /// The grid at (x, y) (`borg_grids[y][x]`).
    pub fn grid(&self, x: i32, y: i32) -> &BorgGrid {
        &self.grids[Self::index(x, y)]
    }

    /// The grid at (x, y), mutable.
    pub fn grid_mut(&mut self, x: i32, y: i32) -> &mut BorgGrid {
        &mut self.grids[Self::index(x, y)]
    }

    /// Row `y` (`borg_grids[y]`).
    pub fn row_mut(&mut self, y: i32) -> &mut [BorgGrid] {
        let start = Self::index(0, y);
        &mut self.grids[start..start + AUTO_MAX_X as usize]
    }
}

/// Allocate the map (`borg_init_cave`).
pub fn borg_init_cave(st: &mut BorgState) {
    borg_free_cave(st);
    let z = engine::z_info();
    if DUNGEON_WID != i32::from(z.dungeon_wid) || DUNGEON_HGT != i32::from(z.dungeon_hgt) {
        crate::borg_io::borg_note(st, "**STARTUP FAILURE** dungeon size miss match");
        st.init.init_failure = true;
    }
    st.cave.grids = vec![BorgGrid::default(); (AUTO_MAX_X * AUTO_MAX_Y) as usize];
}

/// Free the map (`borg_free_cave`).
pub fn borg_free_cave(st: &mut BorgState) {
    st.cave.grids = Vec::new();
}

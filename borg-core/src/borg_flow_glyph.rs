//! Port of `borg-flow-glyph.c`: flow (move) to glyphs.
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
//! The known glyphs of warding (`track_glyph`), `borg_needs_new_sea` and
//! the file statics of the "Sea of Runes" builder live in `GlyphState`.
//! The flow core is `borg_flow`; `borg_flow_reverse` is in
//! `borg_flow_misc`.

use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_flow_stairs::StairTrack;
use crate::borg_headers::borg_magic_h::GLYPH_OF_WARDING;
use crate::headers::list_terrain_h::FEAT_FLOOR;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::{borg_flow, borg_flow_misc, borg_io, borg_item_use, borg_magic, engine};

/// `GOAL_MISC` (borg-flow.h): the goal of a miscellaneous flow.
const GOAL_MISC: i32 = 3;

/// `track_glyph` allocation size (`borg_init_flow_glyph()`).
const TRACK_GLYPH_SIZE: i16 = 200;

/// borg-flow-glyph.c globals and file statics.
#[derive(Default)]
pub struct GlyphState {
    /// `track_glyph`: the known glyphs of warding (the same
    /// `struct borg_track` as the stair lists; C `num`/`size` semantics)
    pub track: StairTrack,
    /// `borg_needs_new_sea`: the environment changed, a new Sea of Runes
    /// is needed (for Morgoth)
    pub needs_new_sea: bool,
    /// `glyph_x`: the grid the Borg flows to, to create a glyph (0 = none)
    pub glyph_x: u8,
    /// `glyph_y`
    pub glyph_y: u8,
    /// `glyph_x_center`: the center of the Sea of Runes (0 = none)
    pub center_x: u8,
    /// `glyph_y_center`
    pub center_y: u8,
}

/// The 24 grids around (x, y) in `borg_ddx_ddd`/`borg_ddy_ddd` order, as
/// `(feat, glyph)`.
fn ring(st: &BorgState, x: i32, y: i32) -> [(u8, bool); 24] {
    let mut out = [(0, false); 24];
    for (i, o) in out.iter_mut().enumerate() {
        /* Extract the location */
        let (dx, dy) = borg_flow::borg_ddxy_ddd(i);

        /* Get the grid contents */
        let g = st.cave.grid(x + dx, y + dy);
        *o = (g.feat, g.glyph);
    }
    out
}

/// Note a glyph created under the Borg (unless already known); `false`
/// if it was already known.
fn track_new_glyph(st: &mut BorgState) -> bool {
    let (bx, by) = (st.borg.c.x, st.borg.c.y);

    /* Check for an existing glyph */
    let mut i = 0;
    while i < i32::from(st.glyph.track.num) {
        /* Stop if we already new about this glyph */
        if st.glyph.track.x[i as usize] == bx && st.glyph.track.y[i as usize] == by {
            return false;
        }
        i += 1;
    }

    /* Track the newly discovered glyph */
    if st.glyph.track.num < st.glyph.track.size {
        borg_io::borg_note(st, "# Noting the creation of a glyph.");
        let n = st.glyph.track.num as usize;
        st.glyph.track.x[n] = bx;
        st.glyph.track.y[n] = by;
        st.glyph.track.num += 1;
    }
    true
}

/// Prepare to flow towards a location and create a special glyph of
/// warding pattern.
///
/// The borg will look for a room that is at least 7x7.
/// ```text
/// ##########
/// #3.......#
/// #2.xxxxx.#
/// #1.xxxxx.#
/// #0.xx@xx.#
/// #1.xxxxx.#
/// #2.xxxxx.#
/// #3.......#
/// # 3210123#
/// ##########
/// ```
/// and when he locates one, he will attempt to:
/// 1. flow to a central location and
/// 2. begin planting Runes in a pattern. When complete,
/// 3. move to the center of it.
///
/// The 24 grids around a grid are taken in `borg_ddx_ddd`/`borg_ddy_ddd`
/// order.
pub fn borg_flow_glyph(st: &mut BorgState) -> bool {
    let mut b_x = st.borg.c.x;
    let mut b_y = st.borg.c.y;
    let mut b_v: i32 = -1;

    let center = loc { x: i32::from(st.glyph.center_x), y: i32::from(st.glyph.center_y) };
    if (st.glyph.center_y == 0 && st.glyph.center_x == 0)
        || engine::distance(st.borg.c, center) >= 50
    {
        st.glyph.needs_new_sea = true;
    }

    /* We have arrived */
    if i32::from(st.glyph.glyph_x) == st.borg.c.x && i32::from(st.glyph.glyph_y) == st.borg.c.y {
        /* Cancel */
        st.glyph.glyph_x = 0;
        st.glyph.glyph_y = 0;

        /* Store the center of the glyphs */
        if st.glyph.needs_new_sea {
            st.glyph.center_y = st.borg.c.y as u8;
            st.glyph.center_x = st.borg.c.x as u8;
        }

        st.glyph.needs_new_sea = false;

        /* Take note */
        let note = format!("# Glyph Creating at ({},{})", st.borg.c.x, st.borg.c.y);
        borg_io::borg_note(st, &note);

        /* Create the Glyph */
        let scroll = st.item_val.sv_scroll_rune_of_protection;
        let act = st.act.glyph;
        if borg_magic::borg_spell_fail(st, GLYPH_OF_WARDING, 30)
            || borg_item_use::borg_read_scroll(st, scroll)
            || borg_item_use::borg_activate_item(st, act)
        {
            /* Success, unless the glyph was already known */
            return track_new_glyph(st);
        }

        /* Nope */
        return false;
    }

    // Reverse flow, not optimized: it starts at the borg's own grid (cost 0)
    // and an optimized spread stops before reaching any other grid
    // (better-borg-c milestone7 patch 0065, formerly local correction #345).
    borg_flow_misc::borg_flow_reverse(st, 250, false, false, false, -1, false);

    let needs_new_sea = st.glyph.needs_new_sea;
    let cx = i32::from(st.glyph.center_x);
    let cy = i32::from(st.glyph.center_y);

    /* Scan the entire map */
    for y in 15..AUTO_MAX_Y - 15 {
        for x in 50..AUTO_MAX_X - 50 {
            /* Acquire the grid */
            let here = *st.cave.grid(x, y);

            /* Skip every non floor/glyph (C bug 29: only glyphed non-floors) */
            if here.feat != FEAT_FLOOR as u8 && here.glyph {
                continue;
            }

            /* Acquire the cost */
            let cost = i32::from(st.flow.cost_at(y, x));

            /* Skip grids that are really far away.  He probably
             * won't be able to safely get there
             */
            if cost >= 75 {
                continue;
            }

            /* Extract adjacent locations to each considered grid */
            let around = ring(st, x, y);

            /* Center Grid */
            if needs_new_sea {
                let goal_glyph = 24;

                /* Count Adjacent Flooors */
                let floor = around
                    .iter()
                    .filter(|&&(feat, glyph)| feat == FEAT_FLOOR as u8 || glyph)
                    .count();

                /* Not a good location if not the center of the sea */
                if floor != 24 {
                    continue;
                }

                /* Count floors already glyphed */
                let tmp_glyph = around.iter().filter(|&&(_, glyph)| glyph).count() as i32;

                /* Tweak -- Reward certain floors, punish distance */
                let mut v = 100 + (tmp_glyph * 500) - cost;
                if here.feat == FEAT_FLOOR as u8 {
                    v += 3000;
                }

                /* If this grid is surrounded by glyphs, select it */
                if tmp_glyph == goal_glyph {
                    v += 5000;
                }

                /* If this grid is already glyphed but not
                 * surrounded by glyphs, then choose another.
                 */
                if tmp_glyph != goal_glyph && here.glyph {
                    v = -1;
                }

                /* The grid is not searchable */
                if v <= 0 {
                    continue;
                }

                /* Track "best" grid */
                if b_v >= 0 && v < b_v {
                    continue;
                }

                /* Save the data */
                b_v = v;
                b_x = x;
                b_y = y;
            }
            /* old center, making outlying glyphs, */
            else {
                /* C's `tmp_glyph` is always 0 here */
                let tmp_glyph = 0;

                for i in 0..24 {
                    /* Leave if this grid is not in good array */
                    let (dx, dy) = borg_flow::borg_ddxy_ddd(i);
                    if cx + dx != x {
                        continue;
                    }
                    if cy + dy != y {
                        continue;
                    }

                    /* Already got a glyph on it */
                    if here.glyph {
                        continue;
                    }

                    /* Tweak -- Reward certain floors, punish distance */
                    let v = 500 + (tmp_glyph * 500) - cost;

                    /* The grid is not searchable */
                    if v <= 0 {
                        continue;
                    }

                    /* Track "best" grid */
                    if b_v >= 0 && v < b_v {
                        continue;
                    }

                    /* Save the data */
                    b_v = v;
                    b_x = x;
                    b_y = y;
                }
            }
        }
    }

    /* Is the sea around the old center complete? */
    if cy != 0 && cx != 0 {
        let mut glyph = 0;
        for i in 0..24 {
            /* Extract the location */
            let (dx, dy) = borg_flow::borg_ddxy_ddd(i);

            /* Count the glyphs */
            if st.cave.grid(cx + dx, cy + dy).glyph {
                glyph += 1;
            }

            /* Save the data */
            if glyph == 24 {
                b_v = 5000;
                b_x = cx;
                b_y = cy;
            }
        }
    }

    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Nothing found */
    if b_v < 0 {
        return false;
    }

    /* Memorize */
    st.glyph.glyph_x = b_x as u8;
    st.glyph.glyph_y = b_y as u8;

    /* Enqueue the grid */
    borg_flow::borg_flow_enqueue_grid(st, b_y, b_x);

    /* Spread the flow */
    borg_flow::borg_flow_spread(st, 250, true, false, false, -1, false);

    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, c"Glyph", GOAL_MISC) {
        return false;
    }

    /* Take one step */
    if !borg_flow::borg_flow_old(st, GOAL_MISC) {
        return false;
    }

    /* Success */
    true
}

/// `borg_init_flow_glyph()`: 200 entries.
pub fn borg_init_flow_glyph(st: &mut BorgState) {
    st.glyph.track.init(TRACK_GLYPH_SIZE);
}

/// `borg_free_flow_glyph()`.
pub fn borg_free_flow_glyph(st: &mut BorgState) {
    st.glyph.track.free();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_free_keep_c_sizes() {
        let mut st = BorgState::default();
        borg_init_flow_glyph(&mut st);
        let t = &st.glyph.track;
        assert_eq!((t.num, t.size, t.x.len(), t.y.len()), (0, 200, 200, 200));
        borg_free_flow_glyph(&mut st);
        assert_eq!(st.glyph.track, StairTrack::default());
    }
}

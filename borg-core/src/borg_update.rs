//! Port of `borg-update.c`: update the Borg's knowledge from the current
//! frame.
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
//! `borg_update` runs once per Borg turn. It expires old monsters and
//! objects, reads the queued messages (detection, kills, hits, spells),
//! handles a new level, reads the map panel from the game, tracks the
//! monsters and objects it sees, and spreads the monster fear. Every step
//! keeps C's order: the monster and object passes decide which entries of
//! the kill and take lists are reused, and several callees draw from the
//! RNG.

use crate::borg::BorgTime;
use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_light::{borg_update_light, BORG_LIGHT};
use crate::borg_cave_util::{borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid};
use crate::borg_cave_view::{borg_forget_view, borg_update_view, BORG_VIEW};
use crate::borg_danger::borg_danger;
use crate::borg_flow::{borg_can_dig, borg_ddxy_ddd, copy_data, zero_data};
use crate::borg_flow_kill::{
    borg_count_death, borg_create_kill, borg_delete_kill, borg_follow_kill, borg_locate_kill,
    borg_race_name, borg_sleep_kill, observe_kill_diff, observe_kill_move, BorgKill,
};
use crate::borg_flow_stairs::StairTrack;
use crate::borg_flow_take::{
    borg_delete_take, borg_follow_take, observe_take_diff, observe_take_move, BorgTake,
};
use crate::borg_io::{borg_note, borg_note_bytes};
use crate::borg_item_wear::borg_clear_best;
use crate::borg_projection::{borg_los, borg_target_unknown_wall};
use crate::engine;
use crate::headers::__stddef_wchar_t_h::wchar_t;
use crate::borg_headers::borg_h::BORG_AUTOSAVE;
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_HLIFE, BI_IACID, BI_ICOLD, BI_IELEC, BI_IFIRE,
    BI_IPOIS, BI_ISAFRAID, BI_ISBLIND, BI_ISCONFUSED, BI_ISCUT, BI_ISHEAVYSTUN, BI_ISIMAGE,
    BI_ISPOISONED, BI_ISSTUN, BI_ISWEAK, BI_KING, BI_LIGHT, BI_MAXSP, BI_RBLIND, BI_RCONF,
    BI_RDARK, BI_RDIS, BI_RKAOS, BI_RLITE, BI_RNTHR, BI_RNXUS, BI_RPOIS, BI_RSHRD, BI_RSND,
};
use crate::headers::list_mon_race_flags_h::{
    RF_MULTIPLY, RF_NEVER_MOVE, RF_PASS_WALL, RF_QUESTOR, RF_SEASONAL, RF_UNIQUE,
};
use crate::headers::list_mon_spells_h::*;
use crate::headers::list_mon_timed_h::MON_TMD_SLEEP;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_FLOOR, FEAT_GRANITE, FEAT_LAVA, FEAT_LESS, FEAT_MAGMA,
    FEAT_MAGMA_K, FEAT_MORE, FEAT_NONE, FEAT_OPEN, FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K,
    FEAT_RUBBLE,
};
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// `AUTO_VIEW_MAX`: size of the monster/object tracking list.
pub const AUTO_VIEW_MAX: i32 = 9000;

/// `BORG_MARK` (borg-cave.h grid info bit): observed.
const BORG_MARK: u16 = 0x01;
/// `BORG_GLOW`: probably perma-lit.
const BORG_GLOW: u16 = 0x02;
/// `BORG_DARK`: probably not perma-lit.
const BORG_DARK: u16 = 0x04;
/// `BORG_OKAY`: on the current panel.
const BORG_OKAY: u16 = 0x08;
/// `BORG_IGNORE_MAP`: ignore the map (stone to mud was cast here).
const BORG_IGNORE_MAP: u16 = 0x100;
/// `BORG_HOME` (borg-store.h): the store number of the home.
const BORG_HOME: u8 = 7;

/// A monster or object seen on the map panel (`borg_wank`).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct BorgWank {
    pub x: u8,
    pub y: u8,
    /// Screen attribute
    pub t_a: u8,
    /// Screen character
    pub t_c: wchar_t,
    pub is_take: bool,
    pub is_kill: bool,
}

/// The panel detection arrays (`borg_detect_wall/trap/door/evil/obj`).
#[derive(Clone, Copy, Debug)]
pub enum Detect {
    Wall,
    Trap,
    Door,
    Evil,
    Obj,
}

/// The module's globals (borg-update.c).
///
/// The detection arrays have `old_panels.y + 1` rows of `old_panels.x + 1`
/// entries each (one more than the panels, as in C: "the code is sloppy
/// about current panel plus one"); empty = NULL.
pub struct UpdateState {
    /// `borg_wank_num`: entries of `wanks` in use
    pub wank_num: i32,
    /// `borg_wanks` (`AUTO_VIEW_MAX` entries; empty = NULL)
    pub wanks: Vec<BorgWank>,
    /// `borg_failure`: the last command failed (set by borg-messages.c)
    pub failure: bool,
    /// `borg_detect_wall`: panels where walls were detected
    pub detect_wall: Vec<Vec<bool>>,
    /// `borg_detect_trap`
    pub detect_trap: Vec<Vec<bool>>,
    /// `borg_detect_door`
    pub detect_door: Vec<Vec<bool>>,
    /// `borg_detect_evil`
    pub detect_evil: Vec<Vec<bool>>,
    /// `borg_detect_obj`
    pub detect_obj: Vec<Vec<bool>>,
    /// `old_panels`: the panel count the arrays were made for (-1 = none)
    pub old_panels: loc,
    /// `borg_do_update_view`: recalculate the view
    pub do_update_view: bool,
    /// `borg_do_update_lite`: recalculate the light
    pub do_update_lite: bool,
    /// `o_w_x`: the old panel
    pub o_w_x: i32,
    /// `o_w_y`
    pub o_w_y: i32,
    /// `old_c`: the old location
    pub old_c: loc,
}

impl Default for UpdateState {
    fn default() -> Self {
        UpdateState {
            wank_num: 0,
            wanks: Vec::new(),
            failure: false,
            detect_wall: Vec::new(),
            detect_trap: Vec::new(),
            detect_door: Vec::new(),
            detect_evil: Vec::new(),
            detect_obj: Vec::new(),
            old_panels: loc { x: -1, y: -1 },
            do_update_view: false,
            do_update_lite: false,
            o_w_x: -1,
            o_w_y: -1,
            old_c: loc { x: -1, y: -1 },
        }
    }
}

/// An array index from a C `int`; a negative one panics (C would write
/// outside the array).
fn idx(i: i32) -> usize {
    usize::try_from(i).expect("negative index")
}

impl UpdateState {
    fn arr(&self, what: Detect) -> &Vec<Vec<bool>> {
        match what {
            Detect::Wall => &self.detect_wall,
            Detect::Trap => &self.detect_trap,
            Detect::Door => &self.detect_door,
            Detect::Evil => &self.detect_evil,
            Detect::Obj => &self.detect_obj,
        }
    }

    fn arr_mut(&mut self, what: Detect) -> &mut Vec<Vec<bool>> {
        match what {
            Detect::Wall => &mut self.detect_wall,
            Detect::Trap => &mut self.detect_trap,
            Detect::Door => &mut self.detect_door,
            Detect::Evil => &mut self.detect_evil,
            Detect::Obj => &mut self.detect_obj,
        }
    }

    /// `borg_detect_<what>[y][x]`. Panics when the arrays are not
    /// allocated or the panel is past them (C would read through NULL or
    /// out of bounds).
    pub fn detect(&self, what: Detect, y: i32, x: i32) -> bool {
        let a = self.arr(what);
        assert!(!a.is_empty(), "borg_detect: arrays not allocated");
        a[idx(y)][idx(x)]
    }

    /// `borg_detect_<what>[y][x] = v` (panics as `detect`).
    fn set_detect(&mut self, what: Detect, y: i32, x: i32, v: bool) {
        let a = self.arr_mut(what);
        assert!(!a.is_empty(), "borg_detect: arrays not allocated");
        a[idx(y)][idx(x)] = v;
    }

    /// Mark the four panels from (`q_y`, `q_x`) as detected.
    fn mark4(&mut self, what: Detect, q_y: i32, q_x: i32, v: bool) {
        self.set_detect(what, q_y, q_x, v);
        self.set_detect(what, q_y, q_x + 1, v);
        self.set_detect(what, q_y + 1, q_x, v);
        self.set_detect(what, q_y + 1, q_x + 1, v);
    }
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `SCREEN_HGT` (ui-term.h), read from the term at each use.
fn screen_hgt() -> i32 {
    let t = engine::term_geometry();
    (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height
}

/// `SCREEN_WID` (ui-term.h), read from the term at each use.
fn screen_wid() -> i32 {
    let t = engine::term_geometry();
    (t.wid - t.col_map - 1) / t.tile_width
}

/// A `%s` argument as `format()` (z-form.c) prints it: at most 1023
/// bytes (it copies the argument into a 1024-byte buffer first).
fn pct_s(s: &[u8]) -> &[u8] {
    &s[..s.len().min(1023)]
}

/// `atoi(s)` (glibc: `(int) strtol(s, NULL, 10)`).
fn atoi(s: &[u8]) -> i32 {
    crate::borg_formulas::c_strtol(s).0 as i32
}

/// Add (x, y) to a track list unless it is there already or the list is
/// full (the copied C loop). Returns whether it was added.
fn track_add(t: &mut StairTrack, x: i32, y: i32) -> bool {
    let mut i = 0;
    while i < i32::from(t.num) {
        if t.x[idx(i)] == x && t.y[idx(i)] == y {
            break;
        }
        i += 1;
    }
    if i == i32::from(t.num) && i < i32::from(t.size) {
        t.x[idx(i)] = x;
        t.y[idx(i)] = y;
        t.num += 1;
        true
    } else {
        false
    }
}

/// Forget the map (`borg_forget_map`).
fn borg_forget_map(st: &mut BorgState) {
    // Clean up the grids
    for y in 0..AUTO_MAX_Y {
        for x in 0..AUTO_MAX_X {
            let ag = st.cave.grid_mut(x, y);

            // Wipe it
            *ag = BorgGrid::default();

            // Lay down the outer walls
            ag.feat = FEAT_PERM as u8;
        }
    }

    // Clean up the grids
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            let town = st.borg.tr(BI_CDEPTH) == 0;
            let ag = st.cave.grid_mut(x, y);

            // Forget the contents
            ag.feat = FEAT_NONE as u8;

            // Prepare the town
            if town {
                ag.feat = FEAT_FLOOR as u8;
            }
        }
    }

    let f = &mut st.flow;
    // Reset "borg_data_cost"
    copy_data(&mut f.data_cost, &f.data_hard);
    // Reset "borg_data_flow"
    copy_data(&mut f.data_flow, &f.data_hard);
    // Clear "borg_data_know"
    zero_data(&mut f.data_know);
    // Clear "borg_data_icky"
    zero_data(&mut f.data_icky);

    // Forget the view
    borg_forget_view(st);
}

/// Update the "map" based on visual info on the screen
/// (`borg_update_map`).
///
/// The grid under the player is assumed to be what the game says it is.
/// `FEAT_NONE` marks grids never seen (or always seen with an object or
/// monster on them), `BORG_MARK` observed grids, `BORG_OKAY` grids on the
/// current panel. When a grid changes between blocking and not blocking
/// motion it is removed from any flow, and the view/light are redone.
fn borg_update_map(st: &mut BorgState) {
    // Analyze the current map panel
    let mut dy = 0;
    while dy < screen_hgt() {
        // Scan the row
        let mut dx = 0;
        while dx < screen_wid() {
            update_map_grid(st, st.core.w_x + dx, st.core.w_y + dy);
            dx += 1;
        }
        dy += 1;
    }
}

/// One grid of `borg_update_map` (the body of its loop; a C `continue` is
/// a `return`).
fn update_map_grid(st: &mut BorgState, x: i32, y: i32) {
    // Since the map is now dynamically sized, double check we are in
    // bounds
    if !engine::square_in_bounds(x, y) {
        return;
    }

    // Cheat the exact information from the screen
    let g = engine::map_info(x, y);

    let ag = st.cave.grid_mut(x, y);

    // Notice "on-screen"
    ag.info |= BORG_OKAY;

    // Save the old "wall" or "door", before the feature below is
    // overwritten with what the game shows now
    let old_wall = !borg_cave_floor_grid(ag);

    // Notice "knowledge": if this square is not in view and the borg
    // previously cast stone to mud here, ignore the map info so repeated
    // stone to mud aren't cast
    if g.f_idx != FEAT_NONE && (g.in_view || ag.info & BORG_IGNORE_MAP == 0) {
        if g.in_view {
            ag.info &= !BORG_IGNORE_MAP;
        }
        ag.info |= BORG_MARK;
        ag.feat = g.f_idx as u8;
    }

    // default store to HOME
    ag.store = BORG_HOME;

    // Notice the player
    if g.is_player {
        // Memorize player location
        st.borg.c.x = x;
        st.borg.c.y = y;
    }

    // Analyze know information about grid
    if engine::feat_is_shop(g.f_idx as i32) {
        // Shop Doors
        let i = engine::square_shopnum(x, y);
        let ag = st.cave.grid_mut(x, y);
        ag.feat = g.f_idx as u8;
        ag.store = i as u8;

        // Save new information
        st.flow_misc.track_shop_x[idx(i)] = x;
        st.flow_misc.track_shop_y[idx(i)] = y;
    } else if engine::square_isdisarmabletrap(x, y) || g.glyph {
        // This checks that the trap is visible so it isn't really a cheat
        // of any information the borg couldn't get another way. A glyph
        // (rune of protection) is not a player trap, so it is taken from
        // the trap the player remembers on this grid (better-borg-c
        // milestone9 patch 0098). Minor cheat: in graphics mode he would
        // mis-id the glyph of warding as a trap.
        if engine::square_isdisarmabletrap(x, y) {
            st.cave.grid_mut(x, y).trap = true;
        }
        // FORK (0098 guard): a remembered rune may be gone from the cave
        // (broken out of view), leaving no trap to read.
        if engine::square_trap_is_glyph(x, y) {
            st.cave.grid_mut(x, y).glyph = true;

            // Track the newly discovered glyph
            track_add(&mut st.glyph.track, x, y);
        }
    } else if g.f_idx == FEAT_NONE {
        // Darkness
        let ag = st.cave.grid_mut(x, y);

        // The grid is not lit
        ag.info &= !BORG_GLOW;

        // Known grids must be dark floors
        if u32::from(ag.feat) != FEAT_NONE {
            ag.info |= BORG_DARK;
        }
    }
    else if g.f_idx == FEAT_FLOOR {
        // Floors (blind: nothing special)

        // Handle "dark" floors
        let glow = engine::square_isglow(x, y);
        let lit = engine::square_islit(x, y);
        let ag = st.cave.grid_mut(x, y);
        if !glow && !lit {
            // Dark floor grid
            ag.info |= BORG_DARK;
            ag.info &= !BORG_GLOW;
        } else if glow {
            // Perma Glowing Grid
            ag.info |= BORG_GLOW;

            // Assume not dark
            ag.info &= !BORG_DARK;
        } else {
            // torch-lit or line of sight grids
            ag.info |= BORG_LIGHT;

            // Assume not dark
            ag.info &= !BORG_DARK;
        }
    } else if g.f_idx == FEAT_OPEN || g.f_idx == FEAT_BROKEN {
        // Open doors
    } else if g.f_idx == FEAT_GRANITE || g.f_idx == FEAT_PERM {
        // Walls. A humongo cheat: is it a perma grid? Only counts for
        // being a vault if not in town and not on the edge of the map.
        let (width, height) = engine::cave_size();
        if u32::from(st.cave.grid(x, y).feat) == FEAT_PERM
            && st.borg.tr(BI_CDEPTH) != 0
            && x != 0
            && y != 0
            && x != width - 1
            && y != height - 1
        {
            st.borg.status.vault = true;
        }
    } else if g.f_idx == FEAT_LAVA {
        // lava
    } else if g.f_idx == FEAT_MAGMA || g.f_idx == FEAT_QUARTZ || g.f_idx == FEAT_RUBBLE {
        // Seams and rubble. If we are twitching around unable to go
        // anywhere, count regular veins as worth digging out
        if st.borg.times_twitch > 21 {
            // but only quartz if we can dig it
            if !borg_can_dig(st, true, FEAT_QUARTZ_K as u8) && g.f_idx == FEAT_QUARTZ {
                return;
            }

            // Track the newly discovered vein (do not overflow)
            if track_add(&mut st.flow_misc.track_vein, x, y) && st.flow_misc.track_vein.num > 99 {
                st.flow_misc.track_vein.num = 99;
            }
        }
    } else if g.f_idx == FEAT_MAGMA_K || g.f_idx == FEAT_QUARTZ_K {
        // Hidden: track the newly discovered vein (do not overflow)
        if track_add(&mut st.flow_misc.track_vein, x, y) && st.flow_misc.track_vein.num > 99 {
            st.flow_misc.track_vein.num = 99;
        }
    } else if g.f_idx == FEAT_CLOSED {
        // Doors, only while low level (do not overflow)
        if st.borg.tr(BI_CLEVEL) <= 5
            && track_add(&mut st.flow.track_closed, x, y)
            && st.flow.track_closed.num > 254
        {
            st.flow.track_closed.num = 254;
        }
    } else if g.f_idx == FEAT_LESS {
        // Up stairs
        track_add(&mut st.stairs.less, x, y);
    } else if g.f_idx == FEAT_MORE {
        // Down stairs
        track_add(&mut st.stairs.more, x, y);
    }

    let web = u32::from(st.cave.grid(x, y).feat) == FEAT_FLOOR && engine::square_iswebbed(x, y);
    st.cave.grid_mut(x, y).web = web;

    // Now do non-feature stuff: monsters/objects
    if (g.first_kind.is_some() || g.m_idx != 0) && st.borg.tr(BI_ISIMAGE) == 0 {
        let mut is_mimic = false;

        // Check for memory overflow
        if st.update.wank_num == AUTO_VIEW_MAX {
            let note = format!(
                "# Wank problem at grid ({},{}) m:{} o:{}, borg at ({},{})",
                y,
                x,
                g.m_idx,
                g.first_kind.map_or(0, |k| k.0),
                st.borg.c.y,
                st.borg.c.x
            );
            borg_note(st, &note);
            crate::borg_think::borg_oops(st, "too many objects...");
        }

        // Access next wank, advance
        let n = st.update.wank_num;
        st.update.wank_num += 1;

        let mut wank = BorgWank {
            x: x as u8,
            y: y as u8,
            ..BorgWank::default()
        };
        // Monster symbol takes priority
        if g.m_idx != 0 {
            let m = engine::cave_monster(g.m_idx as i32);
            wank.t_a = m.attr;
            let ridx = m
                .ridx
                .expect("visible monster without a race (C NULL dereference)");
            wank.t_c = engine::race(ridx as usize).d_char;

            // HACK Check if the monster is a mimic. The borg shouldn't
            // auto-know what is and isn't a mimic
            if m.mimics && m.m_timed[MON_TMD_SLEEP as usize] != 0 {
                is_mimic = true;
            }
        } else if let Some((_, d_attr, d_char)) = g.first_kind {
            wank.t_a = d_attr;
            wank.t_c = d_char;
        }
        wank.is_take = g.first_kind.is_some() || is_mimic;
        wank.is_kill = g.m_idx != 0 && !is_mimic;
        st.update.wanks[idx(n)] = wank;
    }

    // Save the new "wall" or "door"
    let ag = *st.cave.grid(x, y);
    let new_wall = !borg_cave_floor_grid(&ag);

    // Notice wall changes
    if old_wall != new_wall {
        // Remove this grid from any flow
        if new_wall {
            st.flow.data_flow[idx(y)][idx(x)] = 255;
        }
        st.flow.data_know[idx(y)][idx(x)] = 0;
        st.flow.data_icky[idx(y)][idx(x)] = 0;

        // Recalculate the view (if needed)
        if ag.info & BORG_VIEW != 0 {
            st.update.do_update_view = true;
        }

        // Recalculate the lite (if needed)
        if ag.info & BORG_LIGHT != 0 {
            st.update.do_update_lite = true;
        }

        // if the walls have changed we should detect again
        st.borg.time.wizard_light = 0;
        st.borg.time.detect_walls = 0;
    }
}

/// `v += d` on a `uint16_t` fear counter (computed in `int`, truncated).
fn add_fear(v: &mut u16, d: i32) {
    *v = (i32::from(*v) + d) as u16;
}

/// Increase the "grid danger" from lots of monsters (`borg_fear_grid`).
/// Each monster gives some danger to the grids it can see within 6
/// grids, more the closer they are.
fn borg_fear_grid(st: &mut BorgState, y: i32, x: i32, mut k: i32) {
    // Not in town
    if st.borg.tr(BI_CDEPTH) == 0 {
        return;
    }

    // In a Sea of Runes, no worry
    if st.borg.morgoth_position || st.borg.status.anti_summon {
        return;
    }

    // Do not add fear in a vault -- Cheating the cave info
    if engine::square_isvault(x, y) {
        return;
    }

    // Access the grid info
    let kill = st.kills.kills[usize::from(st.cave.grid(x, y).kill)].pos;

    // Level 50 borgs have greatly reduced Monster Fear
    if st.borg.tr(BI_CLEVEL) == 50 {
        k = k * 5 / 10;
    }

    // Collect "fear", spread around
    for x1 in -6..=6 {
        for y1 in -6..=6 {
            // careful
            if x + x1 <= 0 || x1 + x >= AUTO_MAX_X {
                continue;
            }
            if y + y1 <= 0 || y1 + y >= AUTO_MAX_Y {
                continue;
            }
            let (fy, fx) = (idx(y + y1), idx(x + x1));

            // Very Weak Fear at this range
            if borg_los(st, kill.y, kill.x, y + y1, x + x1) {
                add_fear(&mut st.danger.fear_monsters[fy][fx], k / 8);
            }

            // Next range set
            if x1 <= -5 || x1 >= 5 || y1 <= -5 || y1 >= 5 {
                continue;
            }

            // Weak Fear at this range
            if borg_los(st, kill.y, kill.x, y + y1, x + x1) {
                add_fear(&mut st.danger.fear_monsters[fy][fx], k / 5);
            }

            // Next range set
            if x1 <= -3 || x1 >= 3 || y1 <= -3 || y1 >= 3 {
                continue;
            }

            // Fear at this range
            if borg_los(st, kill.y, kill.x, y + y1, x + x1) {
                add_fear(&mut st.danger.fear_monsters[fy][fx], k / 3);
            }

            // Next range set
            if x1 <= -2 || x1 >= 2 || y1 <= -2 || y1 >= 2 {
                continue;
            }

            // Mild Fear at this range
            if borg_los(st, kill.y, kill.x, y + y1, x + x1) {
                add_fear(&mut st.danger.fear_monsters[fy][fx], k / 2);
            }

            // Next range set
            if x1 <= -1 || x1 >= 1 || y1 <= -1 || y1 >= 1 {
                continue;
            }

            // Full fear close to this monster
            if borg_los(st, kill.y, kill.x, y + y1, x + x1) {
                add_fear(&mut st.danger.fear_monsters[fy][fx], k);
            }
        }
    }
}

/// Increase the "region danger" (`borg_fear_regional`). Applied when the
/// borg cannot find the source of a message: he assumes it is an
/// invisible monster, which keeps him from resting while unseen guys
/// attack him.
fn borg_fear_regional(
    st: &mut BorgState,
    who: Option<&[u8]>,
    y: i32,
    x: i32,
    k: i32,
    seen_guy: bool,
) {
    // Do not add fear in a vault -- Cheating the cave info
    if engine::square_isvault(x, y) {
        return;
    }

    // Messages
    if seen_guy {
        borg_note(st, &format!("# Fearing region value {k}."));
    } else {
        let mut note =
            format!("# Fearing region ({y},{x}) value {k} because of a non-LOS ").into_bytes();
        note.extend_from_slice(pct_s(args(who)));
        borg_note_bytes(st, &note);
        st.borg.need_shift_panel = true;
    }

    // Current region
    let y0 = y / 11;
    let x0 = x / 11;

    // Nearby regions
    let y1 = if y0 > 0 { y0 - 1 } else { 0 };
    let x1 = if x0 > 0 { x0 - 1 } else { 0 };
    let y2 = if y0 < 5 { y0 + 1 } else { 5 };
    let x2 = if x0 < 17 { x0 + 1 } else { 17 };

    // Collect "fear", spread around
    let r = &mut st.danger.fear_region;
    add_fear(&mut r[idx(y0)][idx(x0)], k);
    add_fear(&mut r[idx(y0)][idx(x1)], k);
    add_fear(&mut r[idx(y0)][idx(x2)], k);
    add_fear(&mut r[idx(y1)][idx(x0)], k / 2);
    add_fear(&mut r[idx(y2)][idx(x0)], k / 2);
    add_fear(&mut r[idx(y1)][idx(x1)], k / 2);
    add_fear(&mut r[idx(y1)][idx(x2)], k / 3);
    add_fear(&mut r[idx(y2)][idx(x1)], k / 3);
    add_fear(&mut r[idx(y2)][idx(x2)], k / 3);
}

/// Base danger from a spell attack by an invisible monster
/// (`borg_fear_spell`), taking account of resistances.
fn borg_fear_spell(st: &mut BorgState, i: i32) -> i32 {
    let mut z = 0;
    let mut p = 0;

    let mut ouch = 0;

    let t = |st: &BorgState, bi: u32| st.borg.tr(bi) != 0;

    // Damage taken
    if i32::from(st.borg.oldchp) > st.borg.tr(BI_CURHP) {
        ouch = (i32::from(st.borg.oldchp) - st.borg.tr(BI_CURHP)) * 2;
    }

    // Check the spell (a C `break` out of a case is a `'case` break here)
    'case: {
        match i as u32 {
            RSF_SHRIEK => {
                p += 10;

                // If low level borg.  Get off the level now.
                if st.borg.tr(BI_CLEVEL) <= 5 {
                    st.borg.goal.leaving = true;
                    st.borg.goal.fleeing = st.borg.goal.leaving;
                }
            }
            RSF_WHIP => p = 100,
            RSF_SPIT => {}
            RSF_SHOT | RSF_ARROW | RSF_BOLT => z = ouch,
            RSF_BR_ACID | RSF_BA_ACID | RSF_BO_ACID => {
                if t(st, BI_IACID) {
                    break 'case;
                }
                z = ouch;
                p += 40;
            }
            RSF_BR_ELEC | RSF_BA_ELEC | RSF_BO_ELEC => {
                if t(st, BI_IELEC) {
                    break 'case;
                }
                z = ouch;
                p += 20;
            }
            RSF_BR_FIRE | RSF_BA_FIRE | RSF_BO_FIRE => {
                if t(st, BI_IFIRE) {
                    break 'case;
                }
                z = ouch;
                p += 40;
            }
            RSF_BR_COLD | RSF_BA_COLD | RSF_BO_COLD => {
                if t(st, BI_ICOLD) {
                    break 'case;
                }
                z = ouch;
                p += 20;
            }
            RSF_BR_POIS => {
                z = ouch;
                if t(st, BI_RPOIS) {
                    break 'case;
                }
                if st.borg.temp.res_pois {
                    break 'case;
                }
                p += 20;
            }
            RSF_BR_NETH => {
                z = ouch + 100;
                if t(st, BI_RNTHR) {
                    break 'case;
                }
                p += 50;
                if t(st, BI_HLIFE) {
                    break 'case;
                }
                // do not worry about drain exp after level 50
                if st.borg.tr(BI_CLEVEL) >= 50 {
                    break 'case;
                }
                p += 150;
            }
            RSF_BR_LIGHT | RSF_BA_LIGHT => {
                z = ouch;
                if t(st, BI_RLITE) {
                    break 'case;
                }
                if t(st, BI_RBLIND) {
                    break 'case;
                }
                p += 20;
            }
            RSF_BR_DARK | RSF_BA_DARK => {
                z = ouch;
                if t(st, BI_RDARK) {
                    break 'case;
                }
                if t(st, BI_RBLIND) {
                    break 'case;
                }
                p += 20;
            }
            RSF_BR_SOUN => {
                z = ouch;
                if t(st, BI_RSND) {
                    break 'case;
                }
                p += 50;
            }
            RSF_BR_CHAO => {
                z = ouch;
                if t(st, BI_RKAOS) {
                    break 'case;
                }
                p += 200;
                if !t(st, BI_RNTHR) {
                    p += 50;
                }
                if !t(st, BI_HLIFE) {
                    p += 50;
                }
                if !t(st, BI_RCONF) {
                    p += 50;
                }
                if st.borg.tr(BI_CLEVEL) == 50 {
                    break 'case;
                }
                p += 100;
            }
            RSF_BR_DISE => {
                z = ouch;
                if t(st, BI_RDIS) {
                    break 'case;
                }
                p += 500;
            }
            RSF_BR_NEXU => {
                z = ouch;
                if t(st, BI_RNXUS) {
                    break 'case;
                }
                p += 100;
            }
            RSF_BR_TIME => {
                z = ouch;
                p += 200;
            }
            RSF_BR_INER => {
                z = ouch;
                p += 50;
            }
            RSF_BR_GRAV => {
                z = ouch;
                p += 50;
                if t(st, BI_RSND) {
                    break 'case;
                }
                p += 50;
            }
            RSF_BR_SHAR => {
                z = ouch;
                if t(st, BI_RSHRD) {
                    break 'case;
                }
                p += 50;
            }
            RSF_BR_PLAS | RSF_BR_WALL => {
                z = ouch;
                if t(st, BI_RSND) {
                    break 'case;
                }
                p += 50;
            }
            // Nothing currently breathes mana but best to have a check
            RSF_BR_MANA => p += 50,
            RSF_BOULDER => {
                z = ouch;
                p += 40;
            }
            // annoying
            RSF_WEAVE => {}
            RSF_BA_POIS => {
                z = ouch;
                if t(st, BI_RPOIS) {
                    break 'case;
                }
                p += 20;
            }
            RSF_BA_SHAR => {
                z = ouch;
                if t(st, BI_RSHRD) {
                    break 'case;
                }
                p += 20;
            }
            RSF_BA_NETH => {
                z = ouch + 100;
                if t(st, BI_RNTHR) {
                    break 'case;
                }
                p += 300;
            }
            RSF_BA_WATE => {
                z = ouch;
                p += 50;
            }
            RSF_BA_MANA | RSF_BO_MANA | RSF_MISSILE => z = ouch,
            RSF_BA_HOLY => {
                z = ouch;
                p += 50;
            }
            RSF_STORM => {
                z = ouch;
                if t(st, BI_RSND) {
                    break 'case;
                }
                if t(st, BI_ISSTUN) {
                    p += 50;
                }
                if t(st, BI_ISHEAVYSTUN) {
                    p += 100;
                }
                if t(st, BI_RCONF) {
                    break 'case;
                }
                p += 100;
            }
            RSF_DRAIN_MANA => {
                if t(st, BI_MAXSP) {
                    p += 10;
                }
            }
            RSF_MIND_BLAST => z = 20,
            RSF_BRAIN_SMASH | RSF_WOUND => {
                z = 12 * 15;
                p += 100;
            }
            RSF_BO_POIS => {
                if t(st, BI_IPOIS) {
                    break 'case;
                }
                z = ouch;
                p += 20;
            }
            RSF_BO_NETH => {
                z = ouch + 100;
                if t(st, BI_RNTHR) {
                    break 'case;
                }
                p += 200;
            }
            RSF_BO_WATE | RSF_BO_PLAS | RSF_BO_ICE | RSF_BE_ELEC => {
                z = ouch;
                p += 20;
            }
            RSF_BE_NETH => {
                z = ouch;
                p += 50;
            }
            RSF_SCARE | RSF_BLIND | RSF_CONF | RSF_HEAL | RSF_TPORT | RSF_TELE_AWAY => p += 10,
            RSF_SLOW => p += 5,
            RSF_HOLD | RSF_TELE_TO | RSF_TELE_SELF_TO => p += 20,
            RSF_HASTE => p += 10 + st.borg.tr(BI_CDEPTH),
            RSF_HEAL_KIN | RSF_BLINK | RSF_DARKNESS => {}
            RSF_TELE_LEVEL => p += 25,
            RSF_TRAPS => p += 50,
            RSF_FORGET => {
                // if you are a spell caster, this is very scary.
                if st.borg.tr(BI_CURSP) > 10 {
                    p += 500;
                } else {
                    p += 30;
                }
            }
            RSF_SHAPECHANGE => p += 200,

            // Summoning is only as dangerous as the monster that is
            // actually summoned. This helps borgs kill summoners
            RSF_S_KIN | RSF_S_MONSTER => p += 55,
            RSF_S_HI_DEMON | RSF_S_HI_UNDEAD | RSF_S_HI_DRAGON | RSF_S_WRAITH => p += 95,
            RSF_S_MONSTERS => p += 30,
            RSF_S_ANIMAL => p += 15,
            RSF_S_SPIDER => p += 25,
            RSF_S_HOUND => p += 45,
            RSF_S_HYDRA => p += 70,
            RSF_S_AINU | RSF_S_DEMON | RSF_S_UNDEAD | RSF_S_DRAGON => p += 80,
            RSF_S_UNIQUE => p += 50,
            _ => {
                p += 1000;
                borg_note(st, "# Mystery attack.  BUG.");
            }
        }
    }

    // C then doubles `ouch` when it is at least half the hit points, but
    // never reads it again (no effect).

    // Notice damage
    p + z + st.borg.tr(BI_CDEPTH) * 2
}

/// Handle DETECTION spells and "call lite" (`borg_handle_self`). Uses the
/// "old" player location and panel.
fn borg_handle_self(st: &mut BorgState, s: &[u8]) {
    // Extract panel.  The offsets are from the panel the Borg last saw:
    // a reaction queued before a level change can address panels outside
    // the detection arrays resized for the new cave, so drop the panel
    // marks of such stale reactions.
    let q_x = st.update.o_w_x / borg_panel_wid();
    let q_y = st.update.o_w_y / borg_panel_hgt();
    let range = format!("({},{} to {},{})", q_y, q_x, q_y + 1, q_x + 1);
    let ok = detect_panel_ok(st, q_y, q_x);

    if st.update.failure {
        // Handle failure
        borg_note(st, "# Something failed");
    } else if s.starts_with(b"lite") {
        // Handle "call lite"
        let note = format!(
            "# Called lite at ({},{})",
            st.update.old_c.y, st.update.old_c.x
        );
        borg_note(st, &note);

        // If not holding a lite, then glow adjacent grids. The spells do
        // some goofy radius thing.
        if st.borg.tr(BI_LIGHT) == 0 {
            let mut y = st.borg.c.y - 1;
            while y <= st.borg.c.y + 1 {
                let mut x = st.borg.c.x - 1;
                while x <= st.borg.c.x + 1 {
                    let ag = st.cave.grid_mut(x, y);

                    // Mark as perma-lit, not dark
                    ag.info |= BORG_GLOW;
                    ag.info &= !BORG_DARK;
                    x += 1;
                }
                y += 1;
            }
        }

        // Convert torch-lit grids to perma-lit grids
        for i in 0..usize::try_from(st.light.light_n).expect("light count") {
            let x = i32::from(st.light.light_x[i]);
            let y = i32::from(st.light.light_y[i]);
            let ag = st.cave.grid_mut(x, y);

            // Mark as perma-lit, not dark
            ag.info |= BORG_GLOW;
            ag.info &= !BORG_DARK;
        }
    } else if ok && s.starts_with(b"wall") {
        // Handle "detect walls"
        borg_note(st, &format!("# Detected walls {range}"));
        st.update.mark4(Detect::Wall, q_y, q_x, true);
    } else if ok && s.starts_with(b"trap") {
        // Handle "detect traps"
        borg_note(st, &format!("# Detected traps {range}"));
        st.update.mark4(Detect::Trap, q_y, q_x, true);
    } else if ok && s.starts_with(b"door") {
        // Handle "detect doors"
        borg_note(st, &format!("# Detected doors {range}"));
        st.update.mark4(Detect::Door, q_y, q_x, true);
    } else if ok && s.starts_with(b"both") {
        // Handle "detect traps and doors"
        borg_note(st, &format!("# Detected traps and doors {range}"));
        st.update.mark4(Detect::Trap, q_y, q_x, true);
        st.update.mark4(Detect::Door, q_y, q_x, true);
    } else if ok && s.starts_with(b"TDE") {
        // Handle "detect traps and doors and evil"
        borg_note(st, &format!("# Detected traps, doors & evil {range}"));
        st.update.mark4(Detect::Trap, q_y, q_x, true);
        st.update.mark4(Detect::Door, q_y, q_x, true);
        st.update.mark4(Detect::Evil, q_y, q_x, true);
    } else if ok && s.starts_with(b"evil") {
        // Handle DETECT_EVIL
        borg_note(st, &format!("# Detected evil {range}"));
        st.update.mark4(Detect::Evil, q_y, q_x, true);
    } else if ok && s.starts_with(b"obj") {
        // Handle "detect objects"
        borg_note(st, &format!("# Detected objects {range}"));
        st.update.mark4(Detect::Obj, q_y, q_x, true);
    }
}

/// Whether the four panels from (`q_y`, `q_x`) lie inside the current
/// panel count, so a stale reaction cannot mark outside the arrays.
fn detect_panel_ok(st: &BorgState, q_y: i32, q_x: i32) -> bool {
    q_x >= 0 && q_y >= 0 && q_x < st.borg.panels.x && q_y < st.borg.panels.y
}

/// Message `i` of the queue (`borg_msg_buf + borg_msg_pos[i]`), without
/// its NUL.
fn msg_text(st: &BorgState, i: i32) -> Vec<u8> {
    let start = usize::try_from(st.msgs.pos[idx(i)]).expect("message position");
    let rest = &st.msgs.buf[start..];
    let n = rest
        .iter()
        .position(|&c| c == 0)
        .expect("message without a NUL");
    rest[..n].to_vec()
}

/// `strchr(msg, ':') + 1`: the arguments of a message. `None` stands for
/// C's `NULL + 1`, which C only dereferences (and crashes on) for a
/// `SPELL_` message without a ':'; the callers panic then.
fn msg_args(msg: &[u8]) -> Option<&[u8]> {
    msg.iter().position(|&c| c == b':').map(|p| &msg[p + 1..])
}

/// The arguments of a message that C dereferences.
fn args(what: Option<&[u8]>) -> &[u8] {
    what.expect("message without ':' (C reads through NULL + 1)")
}

/// `borg_msg_use[i]`
fn msg_used(st: &BorgState, i: i32) -> bool {
    st.msgs.use_[idx(i)] != 0
}

/// `borg_msg_use[i] = v`
fn set_msg_use(st: &mut BorgState, i: i32, v: i16) {
    st.msgs.use_[idx(i)] = v;
}

/// "Shooting through darkness worked": `if (successful_target < 0)
/// successful_target = 2;`
fn target_worked(st: &mut BorgState) {
    if st.fight_attack.successful_target < 0 {
        st.fight_attack.successful_target = 2;
    }
}

/// A kill was found and killed: count it, delete it, mark the message
/// and reset the anti-bounce count.
fn kill_found(st: &mut BorgState, k: i32, i: i32, used: i16, count: bool) {
    if count {
        borg_count_death(st, k);
    }
    borg_delete_kill(st, k);
    set_msg_use(st, i, used);
    // reset the anti-bounce count when doing things
    st.borg.antibounce_count = 1;
}

/// `borg_locate_kill(what, c, r) > 0`, marking the message as `used`.
fn locate(st: &mut BorgState, what: &[u8], c: loc, r: i32, i: i32, used: i16) -> i32 {
    let k = borg_locate_kill(st, what, c, r);
    if k > 0 {
        set_msg_use(st, i, used);
    }
    k
}

/// One tracking pass over the wanks (`Pass 1` .. `Pass 5` of
/// `borg_update`), from the last to the first: `observe` is tried on the
/// monster (`kill`) or object part of each wank; an entry with neither
/// left is replaced by the last one.
fn track_wanks(
    st: &mut BorgState,
    kill: bool,
    mut observe: impl FnMut(&mut BorgState, BorgWank) -> bool,
) {
    let mut i = st.update.wank_num - 1;
    while i >= 0 {
        let wank = st.update.wanks[idx(i)];
        let part = if kill { wank.is_kill } else { wank.is_take };
        if part && observe(st, wank) {
            // Excise the entry
            let w = &mut st.update.wanks[idx(i)];
            if kill {
                w.is_kill = false;
            } else {
                w.is_take = false;
            }
            if !w.is_take && !w.is_kill {
                st.update.wank_num -= 1;
                st.update.wanks[idx(i)] = st.update.wanks[idx(st.update.wank_num)];
            }
        }
        i -= 1;
    }
}

/// Look at the screen and update the borg (`borg_update`).
///
/// Uses the "panel" info (`w_x`, `w_y`) cheated from the game. All the
/// "important" messages that occurred after the last action have been
/// queued in a usable form; they are used to update the knowledge about
/// the world, which may have changed in drastic ways. `borg_t` roughly
/// counts player turns and is not incremented until the Borg is about to
/// act, so nothing ever has a time-stamp of the current time.
pub fn borg_update(st: &mut BorgState) {
    let mut reset = false;
    let mut floor_grid = 0;
    let mut floor_glyphed = 0;

    // *** cheat the panel information ***
    let g = engine::term_geometry();
    st.core.w_y = g.offset_y;
    st.core.w_x = g.offset_x;

    // get the current panel count
    let (width, height) = engine::cave_size();
    st.borg.panels.x = 1;
    if width > borg_panel_wid() {
        st.borg.panels.x = ((width - borg_panel_wid()) * 2) / borg_panel_wid() + 1;
    }
    st.borg.panels.y = 1;
    if height > borg_panel_hgt() {
        st.borg.panels.y = ((height - borg_panel_hgt()) * 2) / borg_panel_hgt() + 1;
    }

    // if the old panel info is larger than the cave, reset it. This can
    // happen if the player changes levels while there are messages
    // waiting to be processed (stale panels left in range are dropped by
    // the queued-detection bounds checks)
    if st.update.o_w_x > width || st.update.o_w_y > height {
        st.update.o_w_y = 0;
        st.update.o_w_x = 0;
    }

    // allocate the detection arrays
    borg_alloc_detection(st);

    // *** Process objects/monsters ***
    expire_kills(st);
    expire_takes(st);

    // *** Handle messages ***
    let hit_dist = handle_messages(st);

    // *** Handle new levels ***
    if i32::from(st.borg.status.old_depth) != st.borg.tr(BI_CDEPTH) {
        new_level(st);

        // Reset
        reset = true;

        // save once per level, but not if Lunal Scumming
        if st.core.cfg[BORG_AUTOSAVE as usize] != 0 && !st.borg.lunal_mode && !st.borg.munchkin_mode
        {
            st.borg.status.save = true;
        }

        // Save new depth
        st.borg.status.old_depth = st.borg.tr(BI_CDEPTH) as i16;

        st.borg.times_twitch = 0;
        st.borg.escapes = 0;
    } else {
        // Handle old level
        old_level(st);
    }

    // *** Update the map ***

    // Track floors and items
    st.update.wank_num = 0;

    // Update the map
    borg_update_map(st);

    // Mark this grid as having been stepped on
    let step = &mut st.flow.track_step;
    step.x[idx(i32::from(step.num))] = st.borg.c.x;
    step.y[idx(i32::from(step.num))] = st.borg.c.y;
    step.num += 1;

    // Clean the steps every so often
    if step.num >= 75 {
        for i in 0..=75 {
            // Move each step down one position
            step.x[i] = step.x[i + 1];
            step.y[i] = step.y[i + 1];
        }
        // reset the count
        step.num = 75;
    }

    // Reset
    if reset {
        // Fake old panel
        st.update.o_w_x = st.core.w_x;
        st.update.o_w_y = st.core.w_y;

        // Fake old location
        st.update.old_c = st.borg.c;

        // Fake goal location
        st.borg.goal.g = st.borg.c;
    }

    // Player moved
    if st.update.old_c.x != st.borg.c.x || st.update.old_c.y != st.borg.c.y {
        // Update view and lite
        st.update.do_update_view = true;
        st.update.do_update_lite = true;

        // Assume I can shoot here
        st.fight_attack.successful_target = 0;
    }

    // Update the view
    if st.update.do_update_view {
        borg_update_view(st);
        st.update.do_update_view = false;
    }

    // Update the lite
    if st.update.do_update_lite {
        borg_update_light(st);
        st.update.do_update_lite = false;
    }

    // make sure features that have monsters on them that can't passwall
    // are marked FEAT_FLOOR
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[idx(i)];
        i += 1;

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        // Ignore monsters who can pass through walls
        if engine::race_flag_has(usize::from(kill.r_idx), RF_PASS_WALL) {
            continue;
        }

        // Make sure this grid keeps Floor grid
        st.cave.grid_mut(kill.pos.x, kill.pos.y).feat = FEAT_FLOOR as u8;
    }

    // Let me know if I am correctly positioned for special Morgoth
    // routines; sea of runes (the 24 grids around me hold glyphs)
    st.borg.morgoth_position = false;
    if st.borg.tr(BI_KING) == 0 && st.borg.near.morgoth {
        let c = st.borg.c;
        // Must be in a fairly central region
        if c.y >= 15 && c.y <= AUTO_MAX_Y - 15 && c.x >= 50 && c.x <= AUTO_MAX_X - 50 {
            // Scan neighbors
            for j in 0..24 {
                let (dx, dy) = borg_ddxy_ddd(j);
                if st.cave.grid(c.x + dx, c.y + dy).glyph {
                    floor_glyphed += 1;
                }
            }

            // Number of perfect grids
            if floor_glyphed == 24 {
                st.borg.morgoth_position = true;
            }
        }
    }

    // Check to see if I am in a correct anti-summon corridor (a tunnel
    // which limits the LOS of summoned monsters)
    for j in 0..24 {
        let (dx, dy) = borg_ddxy_ddd(j);
        let y = st.borg.c.y + dy;
        let x = st.borg.c.x + dx;

        // Stay in the bounds
        if !engine::square_in_bounds(x, y) {
            floor_grid += 1;
            continue;
        }

        // Skip unknown grids (important to make sure next one in line is
        // not LOS floor)
        let feat = u32::from(st.cave.grid(x, y).feat);
        if j <= 7 && feat <= FEAT_MORE {
            floor_grid += 1;
        }
        if j >= 8 && feat <= FEAT_MORE && borg_los(st, st.borg.c.y, st.borg.c.x, y, x) {
            floor_grid += 1;
        }
    }

    // Number of perfect grids
    st.borg.status.anti_summon = floor_grid == 1;

    // Examine changing doors while shallow
    if st.borg.tr(BI_CLEVEL) <= 5 && st.borg.tr(BI_CDEPTH) != 0 && st.flow.track_closed.num != 0 {
        // Scan all known closed doors
        let mut i = 0;
        while i < i32::from(st.flow.track_closed.num) {
            let x = st.flow.track_closed.x[idx(i)];
            let y = st.flow.track_closed.y[idx(i)];
            i += 1;

            let feat = u32::from(st.cave.grid(x, y).feat);
            if feat == FEAT_OPEN || feat == FEAT_BROKEN {
                // This door was not opened by me
                borg_note(
                    st,
                    &format!("# Monster opened door at {y},{x}.  That's scary."),
                );
                st.borg.mon.scary = true;
            }
        }
    }

    // *** Track objects and monsters ***
    let y = |w: BorgWank| i32::from(w.y);
    let x = |w: BorgWank| i32::from(w.x);
    // Pass 1 -- stationary monsters
    track_wanks(st, true, |st, w| {
        observe_kill_move(st, y(w), x(w), 0, w.t_a, w.t_c, false)
    });
    // Pass 2 -- stationary objects
    track_wanks(st, false, |st, w| {
        observe_take_move(st, y(w), x(w), 0, w.t_a, w.t_c)
    });
    // Pass 3a -- moving monsters (distance 1)
    track_wanks(st, true, |st, w| {
        observe_kill_move(st, y(w), x(w), 1, w.t_a, w.t_c, false)
    });
    // Pass 3b -- moving monsters (distance 2)
    track_wanks(st, true, |st, w| {
        observe_kill_move(st, y(w), x(w), 2, w.t_a, w.t_c, false)
    });
    // Pass 3c -- moving monsters (distance 3)
    track_wanks(st, true, |st, w| {
        observe_kill_move(st, y(w), x(w), 3, w.t_a, w.t_c, false)
    });
    // Pass 3d -- moving monsters (distance 7, allow changes)
    track_wanks(st, true, |st, w| {
        observe_kill_move(st, y(w), x(w), 7, w.t_a, w.t_c, true)
    });
    // Pass 4 -- new objects
    track_wanks(st, false, |st, w| {
        observe_take_diff(st, y(w), x(w), w.t_a, w.t_c)
    });
    // Pass 5 -- new monsters (the screen attr/char are not used)
    track_wanks(st, true, |st, w| observe_kill_diff(st, y(w), x(w)));

    // *** Handle messages ***
    handle_late_messages(st, hit_dist);

    // *** Notice missing monsters ***
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[idx(i)];

        // Skip dead, seen, assigned monsters; blind or hallucinating
        if kill.r_idx != 0
            && kill.when != st.borg.time.now
            && !kill.seen
            && st.borg.tr(BI_ISBLIND) == 0
            && st.borg.tr(BI_ISIMAGE) == 0
        {
            // Predict the monster
            borg_follow_kill(st, i);
        }
        i += 1;
    }

    // Update the fear_grid_monsters[][] with the monsters' danger: a
    // 'regional' fear from the accumulated group of monsters (one Orc won't
    // be too dangerous, but 20 of them can be deadly); count up breeders
    spread_monster_fear(st);

    // *** Notice missing objects ***
    let mut i = 1;
    while i < i32::from(st.takes.nxt) {
        let take = st.takes.takes[idx(i)];

        // Skip dead and seen objects; blind or hallucinating
        if take.kidx != 0
            && borg_timer(st, take.when) > 2
            && st.borg.tr(BI_ISBLIND) == 0
            && st.borg.tr(BI_ISIMAGE) == 0
        {
            // Follow the object
            borg_follow_take(st, i);
        }
        i += 1;
    }

    // *** Various things ***

    // Forget goals while "impaired" in any way
    if st.borg.tr(BI_ISBLIND) != 0
        || st.borg.tr(BI_ISCONFUSED) != 0
        || st.borg.tr(BI_ISAFRAID) != 0
        || st.borg.tr(BI_ISIMAGE) != 0
    {
        st.borg.goal.type_0 = 0;
    }

    // Forget goals while "bleeding" in any way
    if st.borg.tr(BI_ISWEAK) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISSTUN) != 0
        || st.borg.tr(BI_ISHEAVYSTUN) != 0
    {
        st.borg.goal.type_0 = 0;
    }

    // Save the hit points
    st.borg.oldchp = st.borg.tr(BI_CURHP) as i16;

    // Forget failure
    st.update.failure = false;

    // Forget the messages
    st.msgs.len = 0;
    st.msgs.num = 0;

    // *** Save old info ***

    // Save the old "location"
    st.update.old_c = st.borg.c;

    // Save the old "panel"
    st.update.o_w_x = st.core.w_x;
    st.update.o_w_y = st.core.w_y;

    // *** Defaults ***

    // Default "goal" location
    st.borg.goal.g = st.borg.c;
}

/// Scan the monsters: clear their flags and expire the ones not seen for
/// a long time (kept while hallucinating and afraid, so the Borg does not
/// get trapped by monsters it is afraid to attack).
fn expire_kills(st: &mut BorgState) {
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let k = idx(i);

        // Skip dead monsters
        if st.kills.kills[k].r_idx != 0 {
            // Clear flags
            st.kills.kills[k].seen = false;
            st.kills.kills[k].used = false;

            let kill = st.kills.kills[k];
            // Skip recently seen monsters (unless hallucinating and afraid)
            let recent = borg_timer(st, kill.when) < 2000
                && !(st.borg.tr(BI_ISIMAGE) != 0 && st.borg.tr(BI_ISAFRAID) != 0);
            if !recent {
                let mut note = b"# Expiring a monster '".to_vec();
                note.extend_from_slice(pct_s(borg_race_name(i32::from(kill.r_idx)).to_bytes()));
                note.extend_from_slice(
                    format!("' ({}) at ({},{})", kill.r_idx, kill.pos.y, kill.pos.x).as_bytes(),
                );
                borg_note_bytes(st, &note);

                // Kill the monster
                borg_delete_kill(st, i);
            }
        }
        i += 1;
    }
}

/// Scan the objects: clear their flags, delete those under the Borg (now
/// or before) or all of them while hallucinating, and expire the ones not
/// seen for a long time.
fn expire_takes(st: &mut BorgState) {
    let mut i = 1;
    while i < i32::from(st.takes.nxt) {
        let k = idx(i);

        // Skip dead objects
        if st.takes.takes[k].kidx != 0 {
            // Clear flags
            st.takes.takes[k].seen = false;

            let take = st.takes.takes[k];
            let (ty, tx) = (i32::from(take.y), i32::from(take.x));
            if (ty == st.borg.c.y && tx == st.borg.c.x)
                || (ty == st.update.old_c.y && tx == st.update.old_c.x)
                || st.borg.tr(BI_ISIMAGE) != 0
            {
                // delete them if they are under me or if I am
                // Hallucinating
                borg_delete_take(st, i);
            } else if borg_timer(st, take.when) >= 2000 {
                let kidx = take.kidx as usize;
                let mut note = b"# Expiring an object '".to_vec();
                note.extend_from_slice(pct_s(engine::kind_name_or_empty(kidx)));
                note.extend_from_slice(
                    format!("' ({}) at ({},{})", engine::kind(kidx).kidx, ty, tx).as_bytes(),
                );
                borg_note_bytes(st, &note);

                // Kill the object
                borg_delete_take(st, i);
            }
        }
        i += 1;
    }
}

/// The message passes before the map update: notes, self info and
/// feelings, then the monsters near the goal, then near the old location
/// (with a wider hit distance after teleports or a quake). Returns that
/// hit distance.
fn handle_messages(st: &mut BorgState) -> i32 {
    // Process messages: note them
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        let mut note = b"# ".to_vec();
        note.extend_from_slice(pct_s(&msg_text(st, i)));
        note.extend_from_slice(b" (+)");
        borg_note_bytes(st, &note);
        i += 1;
    }

    // Process messages: self info and feelings
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        // Skip parsed messages
        if !msg_used(st, i) {
            let msg = msg_text(st, i);
            let what = msg_args(&msg);

            if msg.starts_with(b"SELF:") {
                // Handle "SELF" info
                borg_handle_self(st, args(what));
                set_msg_use(st, i, 1);
            } else if msg.starts_with(b"FEELING_DANGER:") {
                // Handle "You feel..."
                st.think_dungeon.feeling_danger = atoi(args(what));
                set_msg_use(st, i, 1);
            } else if msg.starts_with(b"FEELING_STUFF:") {
                // Handle "You feel..."
                st.think_dungeon.feeling_stuff = atoi(args(what));
                set_msg_use(st, i, 1);
            }
        }
        i += 1;
    }

    // Process messages: monsters at the goal
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        if !msg_used(st, i) {
            let msg = msg_text(st, i);
            goal_message(st, i, &msg);
        }
        i += 1;
    }

    // Process messages: getting distance to allow for 'hit's
    let mut hit_dist = 1;
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        // Skip parsed messages
        if !msg_used(st, i) {
            let msg = msg_text(st, i);

            // if you have been teleported then do not count the monsters
            // as unknown unless they are very far away
            if msg.starts_with(b"SPELL_") {
                let spell = atoi(&msg[6..]);
                if spell == RSF_TELE_AWAY as i32 || spell == RSF_TELE_TO as i32 {
                    hit_dist = 100;
                }
                break;
            }

            // monsters move from earthquake
            if msg.starts_with(b"QUAKE:") {
                hit_dist = 3;
                break;
            }
        }
        i += 1;
    }

    // Process messages: monsters near the old location. Once a monster
    // "cackles evilly" (creates traps), every later unparsed message
    // marks the traps again (C never resets the flag in this loop; C bug
    // 44).
    let mut created_traps = false;
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        if !msg_used(st, i) {
            let msg = msg_text(st, i);
            if old_location_message(st, i, &msg, hit_dist) {
                created_traps = true;
            }

            // Handle "cackles evilly"
            if created_traps {
                created_traps_here(st);
            }
        }
        i += 1;
    }

    // If we didn't successfully hit our target, mark the first unknown
    // grid in the path as a wall. Then reset our targeting count so the
    // borg can try it again. If no unknown grid is found, then allow the
    // borg to attempt another shot. A low number on successful_target
    // prohibits a ranged attack.
    if st.fight_attack.successful_target < 0 && st.fight_attack.successful_target > -10 {
        st.fight_attack.successful_target -= 10;
        if borg_target_unknown_wall(st, st.borg.goal.g.y, st.borg.goal.g.x) {
            st.fight_attack.successful_target = 2;
        }
    }

    hit_dist
}

/// One message of the "monsters at the goal" pass (use 2).
fn goal_message(st: &mut BorgState, i: i32, msg: &[u8]) {
    let what = msg_args(msg);
    let g = st.borg.goal.g;

    if msg.starts_with(b"HIT:") || msg.starts_with(b"MISS:") {
        // Handle "You hit xxx." / "You miss xxx."
        locate(st, args(what), g, 0, i, 2);
    } else if msg.starts_with(b"AFRAID:") {
        // Handle "You too afraid of xxx."
        if st.cave.grid(g.x, g.y).kill > 0 {
            set_msg_use(st, i, 2);
        } else {
            borg_create_kill(st, args(what), g);
        }
    } else if msg.starts_with(b"KILL:") {
        // Handle "You have killed xxx."
        let k = borg_locate_kill(st, args(what), g, 0);
        if k > 0 {
            kill_found(st, k, i, 2, true);
        }
        target_worked(st);
    } else if msg.starts_with(b"BLINK:") {
        // Handle "The xxx disappears!" via teleport other, and blinks away
        let k = borg_locate_kill(st, args(what), g, 0);
        if k > 0 {
            kill_found(st, k, i, 2, false);
        }
        target_worked(st);
    } else if msg.starts_with(b"DIED:") {
        // Handle "xxx dies."
        let k = borg_locate_kill(st, args(what), g, 3);
        if k > 0 {
            kill_found(st, k, i, 2, true);
        }
        target_worked(st);
    } else if msg.starts_with(b"PAIN:") {
        // Handle "xxx screams in pain."
        locate(st, args(what), g, 3, i, 2);
        target_worked(st);
    } else if msg.starts_with(b"STATE__FEAR:") || msg.starts_with(b"STATE__BOLD:") {
        // Handle "sleep"
        locate(st, args(what), g, 0, i, 2);
    } else if msg.starts_with(b"STATE_SLEEP:") || msg.starts_with(b"STATE_CONFUSED:") {
        // C tests "STATE__FEAR:" a second time between these two (never
        // reached, C bug 46)
        target_worked(st);
    }
}

/// One message of the "monsters near the old location" pass (use 3).
/// Returns whether it was a monster creating traps.
fn old_location_message(st: &mut BorgState, i: i32, msg: &[u8], hit_dist: i32) -> bool {
    let what = msg_args(msg);
    let g = st.borg.goal.g;
    let old_c = st.update.old_c;

    if msg.starts_with(b"HIT:") || msg.starts_with(b"MISS:") {
        // Handle "You hit xxx." / "You miss xxx."
        locate(st, args(what), g, hit_dist, i, 3);
    } else if msg.starts_with(b"KILL:") {
        // Handle "You have killed xxx."
        let k = borg_locate_kill(st, args(what), g, 1);
        if k > 0 {
            kill_found(st, k, i, 3, true);
        }
        target_worked(st);
    } else if msg.starts_with(b"BLINK:") {
        // Handle "The xxx disappears!" via teleport other, and blinks away
        let k = borg_locate_kill(st, args(what), g, 1);
        if k > 0 {
            kill_found(st, k, i, 3, false);
        }
        // Shooting through darkness worked
        if st.fight_attack.successful_target == -1 {
            st.fight_attack.successful_target = 2;
        }
    } else if msg.starts_with(b"DIED:") {
        // Handle "xxx dies."
        let k = borg_locate_kill(st, args(what), old_c, 20);
        if k > 0 {
            kill_found(st, k, i, 3, true);
        }
        target_worked(st);
    } else if msg.starts_with(b"PAIN:") {
        // Handle "xxx screams in pain."
        locate(st, args(what), old_c, 20, i, 3);
        target_worked(st);
    } else if msg.starts_with(b"HIT_BY:") || msg.starts_with(b"MISS_BY:") {
        // Handle "xxx hits you." / "xxx misses you."
        locate(st, args(what), old_c, 1, i, 3);
    } else if msg.starts_with(b"STATE_SLEEP:") {
        // Handle "sleep"
        let k = borg_locate_kill(st, args(what), old_c, 20);
        if k > 0 {
            borg_sleep_kill(st, k);
            set_msg_use(st, i, 3);
        }
    } else if msg.starts_with(b"STATE_AWAKE:")
        || msg.starts_with(b"STATE__FEAR:")
        || msg.starts_with(b"STATE__BOLD:")
    {
        // Handle "awake", "afraid", "bold"
        locate(st, args(what), old_c, 20, i, 3);
    } else if msg.starts_with(b"SPELL_") {
        // Handle "spell"
        locate(st, args(what), old_c, 20, i, 3);
        if atoi(&msg[6..]) == RSF_TRAPS as i32 {
            return true;
        }
    }
    false
}

/// A monster created traps: forget the trap detection of the old panel
/// and assume the known floor grids next to the Borg are traps.
fn created_traps_here(st: &mut BorgState) {
    // Remove the flag which tells borg that Trap Detection was done here.
    // The old offsets can address a panel outside the arrays resized for
    // a new cave.
    let q_y = st.update.o_w_y / borg_panel_hgt();
    let q_x = st.update.o_w_x / borg_panel_wid();
    if detect_panel_ok(st, q_y, q_x) {
        st.update.mark4(Detect::Trap, q_y, q_x, false);
    }

    // Leave a note
    borg_note(st, "# Logging need for use of Detect Traps.");

    // Mark Adjacent grids to borg as Traps
    for ii in 0..8 {
        // Grid in that direction
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(ii));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(ii));

        let ag = *st.cave.grid(x, y);

        // Skip unknown grids (important)
        if u32::from(ag.feat) == FEAT_NONE {
            continue;
        }

        // Mark known floor grids as trap
        if borg_cave_floor_grid(&ag) {
            st.cave.grid_mut(x, y).trap = true;

            // Leave a note
            borg_note(st, &format!("# Assuming a Traps at ({y},{x})."));
        }
    }
}

/// The level changed: reset the clocks, goals, lists and the map.
fn new_level(st: &mut BorgState) {
    let b = &mut st.borg;
    // Restart the clock
    b.time.level = b.time.now;
    b.time.morgoth = 0;
    b.time.antisummon = 0;

    // When leaving town, reset the town clock
    if b.status.old_depth == 0 {
        b.time.town = b.time.now;
    }

    // reset our anti-bounce count
    b.antibounce_count = 1;

    // reset our vault/unique check
    b.status.vault = false;
    b.mon.unique = 0;
    b.mon.scary = false;

    // reset our breeder flag
    b.near.breeder = false;

    // reset our need to see inviso clock
    b.need_see_invis = 0;

    // reset our 'shoot in the dark' flag
    st.fight_attack.successful_target = 0;

    // New danger thresh-hold
    st.borg.avoidance = st.borg.tr(BI_CURHP) as i16;

    // Wipe the danger
    st.danger.wipe = true;

    // Clear our bought/sold flags
    st.store_sell.sold_item_num = 0;
    st.store_sell.sold_item_nxt = 1;
    st.buy.bought_item_nxt = 1;
    st.buy.bought_item_num = 0;
    for i in 0..10 {
        st.store_sell.sold_item_tval[i] = 0;
        st.store_sell.sold_item_sval[i] = 0;
        st.store_sell.sold_item_pval[i] = 0;
        st.store_sell.sold_item_store[i] = 0;
        st.buy.bought_item_tval[i] = 0;
        st.buy.bought_item_sval[i] = 0;
        st.buy.bought_item_pval[i] = 0;
        st.buy.bought_item_store[i] = 0;
    }

    // Update some stuff
    st.update.do_update_view = true;
    st.update.do_update_lite = true;

    // Examine the world
    st.think.do_spell = true;

    // Enable some functions
    st.inventory.do_crush_junk = true;

    // Mega-Hack -- Clear "call lite", "wizard lite", "detect traps",
    // "detect doors", "detect walls", DETECT_EVIL and "detect obj" stamps
    let t = &mut st.borg.time;
    t.call_light = 0;
    t.wizard_light = 0;
    t.detect_traps = 0;
    t.detect_doors = 0;
    t.detect_walls = 0;
    t.detect_evil = 0;
    t.detect_obj = 0;

    // Clear "panel" flags
    for y in 0..st.borg.panels.y {
        for x in 0..st.borg.panels.x {
            st.update.set_detect(Detect::Wall, y, x, false);
            st.update.set_detect(Detect::Trap, y, x, false);
            st.update.set_detect(Detect::Door, y, x, false);
            st.update.set_detect(Detect::Evil, y, x, false);
            st.update.set_detect(Detect::Obj, y, x, false);
        }
    }

    // Clear "fear"
    let max_x = (AUTO_MAX_X / 11) + 1;
    let max_y = (AUTO_MAX_Y / 11) + 1;
    for y in 0..max_y {
        for x in 0..max_x {
            st.danger.fear_region[idx(y)][idx(x)] = 0;
        }
    }

    // Remove regional fear from monsters, it gets added back in later.
    for y in 0..AUTO_MAX_Y {
        for x in 0..AUTO_MAX_X {
            st.danger.fear_monsters[idx(y)][idx(x)] = 0;
        }
    }

    let b = &mut st.borg;
    // No goal yet
    b.goal.type_0 = 0;

    // Clear "shop" goals
    b.goal.item = -1;
    b.goal.ware = -1;
    b.goal.shop = -1;

    // Do not use any stairs
    b.stair_more = false;
    b.stair_less = false;

    // Cannot rise past town
    if b.tr(BI_CDEPTH) == 0 {
        b.goal.rising = false;
    }

    // Assume not leaving the level
    b.goal.leaving = false;

    // Assume not fleeing the level
    b.goal.fleeing = false;
    b.goal.fleeing_lunal = false;
    b.goal.fleeing_munchkin = false;

    // Assume not ignoring monsters
    b.goal.ignoring = false;

    // clear stuff feeling - danger feeling is automatic
    st.think_dungeon.feeling_stuff = 0;

    // clear our best shopping spree when entering town
    if st.borg.tr(BI_CDEPTH) == 0 {
        borg_clear_best(st);
    }

    // Assume not fleeing the level
    if st.borg.tr(BI_CDEPTH) == 0 {
        st.borg.goal.fleeing_to_town = false;
    }
    if st.borg.tr(BI_CDEPTH) >= 2 {
        st.borg.goal.fleeing_to_town = false;
    }

    // No known stairs, glyph, steps, doors, veins
    st.stairs.less.num = 0;
    st.stairs.more.num = 0;
    st.glyph.track.num = 0;
    st.flow.track_step.num = 0;
    st.flow.track_door.num = 0;
    st.flow.track_closed.num = 0;
    st.flow_misc.track_vein.num = 0;

    // No artifacts swapping in and out
    st.wear.track_worn_num = 0;

    // No objects here
    st.takes.cnt = 0;
    st.takes.nxt = 1;

    // Forget old objects
    st.takes.takes[..256].fill(BorgTake::default());

    // No monsters here
    st.kills.cnt = 0;
    st.kills.nxt = 1;

    // Hack- Assume that Morgoth is on Level 100 unless we know he is dead
    st.borg.near.morgoth = false;
    if (st.borg.tr(BI_CDEPTH) >= 100 && st.borg.tr(BI_KING) == 0)
        || st.borg.mon.unique == st.borg.id.morgoth
    {
        // We assume Morgoth is on this level
        st.borg.near.morgoth = true;

        // Must build a new sea of runes
        st.glyph.needs_new_sea = true;
    }

    // Reset nasties to 0
    for i in 0..usize::from(st.kills.nasties_num) {
        st.kills.nasties_count[i] = 0;

        // Assume there are some Hounds on the Level
        if st.kills.nasties[i] == b'Z' {
            st.kills.nasties_count[i] = 25;
        }
    }

    // Forget old monsters
    st.kills.kills[..256].fill(BorgKill::BLANK);

    // Forget race counters
    let r_max = usize::from(engine::z_info().r_max);
    st.kills.race_count[..r_max].fill(0);

    // Rarely, a Unique can die off screen and the borg will miss it. This
    // check will cheat to see if uniques are dead.
    count_live_uniques(st);

    // Forget the map
    borg_forget_map(st);
}

/// Clear the unique counters and count the live uniques, keeping the
/// depths of the shallowest three (the `borg_prepared` numbers).
///
/// The fourth unique found is stored unsorted, and later ones are only
/// sorted in when they are shallower than the third (C bug 46).
fn count_live_uniques(st: &mut BorgState) {
    // Clear our Uniques vars
    st.prepared.numb_live_unique = 0;
    st.prepared.first_living_unique = 0;
    st.prepared.depth_hunted_unique = 0;
    let mut unique_depths = [0i32; 4];

    // Extract dead uniques and set some Prep code numbers
    let r_max = i32::from(engine::z_info().r_max);
    let mut u_i: u32 = 1;
    while u_i < (r_max - 1) as u32 {
        let u = u_i as usize;
        u_i += 1;
        let r = engine::race(u);

        // Skip non-monsters
        if r.name.is_null() {
            continue;
        }

        // Skip non-uniques
        if !engine::race_flag_has(u, RF_UNIQUE) {
            continue;
        }

        // Mega-Hack -- Access "dead unique" list
        if r.max_num == 0 {
            st.kills.race_death[u] = 1;
        }

        // If any have been killed it is not a live unique
        if st.kills.race_death[u] != 0 {
            continue;
        }

        // skip certain questor or seasonal Monsters
        if engine::race_flag_has(u, RF_QUESTOR) || engine::race_flag_has(u, RF_SEASONAL) {
            continue;
        }

        // skip things that are just a shape of a different entry
        if r.rarity == 0 {
            continue;
        }

        // Keep track of the three shallowest uniques; they might not be in
        // depth order
        let p = &mut st.prepared;
        if p.numb_live_unique < 4 {
            // track the depth of the three shallowest live uniques
            unique_depths[idx(p.numb_live_unique)] = r.level;

            p.numb_live_unique += 1;

            // this is the first living unique
            if p.numb_live_unique == 1 {
                p.first_living_unique = u as u32;
            }

            // sort the top 3
            if p.numb_live_unique == 3 {
                unique_depths[..3].sort_unstable();
            }
        } else {
            p.numb_live_unique += 1;

            // sort this in to only keep top (lowest) 3
            if r.level < unique_depths[2] {
                unique_depths[3] = r.level;
                unique_depths.sort_unstable();
            }
        }
    }

    // slight cheat here since we know if there is less than 3 live uniques
    // the deepest one is likely to be morgoth and if there is 3 or more we
    // have sorted the list
    let p = &mut st.prepared;
    if p.numb_live_unique >= 3 {
        p.depth_hunted_unique = unique_depths[2];
    } else if p.numb_live_unique != 0 {
        p.depth_hunted_unique = unique_depths[idx(p.numb_live_unique - 1)];
    } else {
        p.depth_hunted_unique = 127;
    }
}

/// Still on the same level: count down the timers, guess where Morgoth
/// is, check the resistances, let the fear decay and notice a panel
/// change.
fn old_level(st: &mut BorgState) {
    // reduce Resistance count. NOTE: do not reduce below 1. That is done
    // when the spell is cast.
    if st.borg.resistance >= 1 {
        let r = st.traits.game_ratio;
        st.borg.resistance = (i32::from(st.borg.resistance) - i32::from(r)) as i16;
    }

    // reduce the No-resting-because-I-just-did-a-prep-spell
    if st.borg.no_rest_prep >= 1 {
        let r = st.traits.game_ratio;
        st.borg.no_rest_prep -= i32::from(r);
    }

    // Count down to blast off
    if st.borg.goal.recalling >= 1 {
        let r = st.traits.game_ratio;
        st.borg.goal.recalling -= i32::from(r);

        // dont let it get to 0 or borg will recast the spell
        if st.borg.goal.recalling <= 0 {
            st.borg.goal.recalling = 1;
        }
    }

    // when we need to cast this spell again
    if st.borg.see_inv >= 1 {
        let r = st.traits.game_ratio;
        st.borg.see_inv = (i32::from(st.borg.see_inv) - i32::from(r)) as i16;
    }

    // Hack- Assume that Morgoth is on Level 100
    st.borg.near.morgoth = false;
    if (st.borg.tr(BI_CDEPTH) >= 100 && st.borg.tr(BI_KING) == 0)
        || st.borg.mon.unique == st.borg.id.morgoth
    {
        // We assume Morgoth is on this level
        st.borg.near.morgoth = true;
    }

    // If been sitting on level 100 for a long time and Morgoth is a no
    // show, or was here but has not been around in a very long time, then
    // assume he is not here so borg can continue to explore the dungeon.
    if st.borg.near.morgoth && borg_timer(st, st.borg.time.level) >= 500 {
        // Morgoth is a no show
        if st.borg.mon.unique != st.borg.id.morgoth {
            st.borg.near.morgoth = false;
        }

        // Morgoth has not been seen in a long time
        if st.borg.mon.unique == st.borg.id.morgoth && borg_timer(st, st.borg.time.morgoth) > 500 {
            let t = borg_timer(st, st.borg.time.morgoth);
            borg_note(
                st,
                &format!("# Morgoth has not been seen in {t} turns.  Going to hunt him."),
            );
            st.borg.near.morgoth = false;
        }

        // Morgoth has not been seen in a very long time
        if borg_timer(st, st.borg.time.morgoth) > 2500 {
            let t = borg_timer(st, st.borg.time.morgoth);
            borg_note(
                st,
                &format!("# Morgoth has not been seen in {t} turns.  No show."),
            );
            st.borg.mon.unique = 0;
        }
    }

    // Slight Cheat for the borg. He would like to keep an eye on the
    // Resist All spell: if all resistances are on, give himself the flag,
    // but not if the flag is already on.
    let tm = st.borg.temp;
    if st.borg.resistance <= 0
        && tm.res_fire
        && tm.res_elec
        && tm.res_cold
        && tm.res_acid
        && tm.res_pois
    {
        // Set the flag on with some average count
        st.borg.resistance = 20000;
    }

    // Slight safety check for borg to make sure he really does resist all
    // if he thinks he does.
    if st.borg.resistance >= 1
        && i32::from(tm.res_fire)
            + i32::from(tm.res_elec)
            + i32::from(tm.res_cold)
            + i32::from(tm.res_acid)
            + i32::from(tm.res_pois)
            != 5
    {
        st.borg.resistance = 0;
    }

    // Reduce fear over time every 10 steps
    if st.borg.time.now % 10 == 0 {
        let max_x = (AUTO_MAX_X / 11) + 1;
        let max_y = (AUTO_MAX_Y / 11) + 1;

        for y in 0..max_y {
            for x in 0..max_x {
                let f = &mut st.danger.fear_region[idx(y)][idx(x)];
                if *f != 0 {
                    *f -= 1;
                }
            }
        }
    }

    // Remove regional fear from monsters, it gets added back in later.
    for y in 0..AUTO_MAX_Y {
        for x in 0..AUTO_MAX_X {
            st.danger.fear_monsters[idx(y)][idx(x)] = 0;
        }
    }

    // Handle changing map panel
    if st.update.o_w_x != st.core.w_x || st.update.o_w_y != st.core.w_y {
        // Forget the previous map panel
        let mut dy = 0;
        while dy < screen_hgt() {
            let mut dx = 0;
            while dx < screen_wid() {
                // Access the actual location
                let x = st.update.o_w_x + dx;
                let y = st.update.o_w_y + dy;
                dx += 1;

                if y >= AUTO_MAX_Y || x >= AUTO_MAX_X {
                    continue;
                }

                // Clear the "okay" field
                st.cave.grid_mut(x, y).info &= !BORG_OKAY;
            }
            dy += 1;
        }

        // reset the anti-bounce count to avoid a repeated motion bug
        st.borg.antibounce_count = 1;
    }
}

/// The message passes after the monster tracking: monsters near the Borg
/// (use 4), then regional fear for unseen attackers (use 5), then the
/// final notes.
fn handle_late_messages(st: &mut BorgState, hit_dist: i32) {
    // Process messages
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        if !msg_used(st, i) {
            let msg = msg_text(st, i);
            let what = msg_args(&msg);
            let c = st.borg.c;

            if msg.starts_with(b"DIED:") {
                // Handle "xxx dies."
                let k = borg_locate_kill(st, args(what), c, 20);
                if k > 0 {
                    borg_count_death(st, k);
                    borg_delete_kill(st, k);
                    set_msg_use(st, i, 4);
                }
            } else if msg.starts_with(b"PAIN:") {
                // Handle "xxx screams in pain."
                locate(st, args(what), c, 20, i, 4);
            } else if msg.starts_with(b"HIT_BY:") || msg.starts_with(b"MISS_BY:") {
                // Handle "xxx hits you." / "xxx misses you."
                locate(st, args(what), c, hit_dist, i, 4);
            } else if msg.starts_with(b"STATE_SLEEP:") {
                // Handle "sleep"
                let k = borg_locate_kill(st, args(what), c, 20);
                if k > 0 {
                    borg_sleep_kill(st, k);
                    set_msg_use(st, i, 4);
                }
            } else if msg.starts_with(b"STATE_AWAKE:")
                || msg.starts_with(b"STATE__FEAR:")
                || msg.starts_with(b"STATE__BOLD:")
                || msg.starts_with(b"SPELL_")
            {
                // Handle "awake", "afraid", "bold", "spell"
                locate(st, args(what), c, 20, i, 4);
            }
        }
        i += 1;
    }

    // Process messages
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        if !msg_used(st, i) {
            let msg = msg_text(st, i);
            let what = msg_args(&msg);
            let depth = st.borg.tr(BI_CDEPTH);

            if msg.starts_with(b"HIT_BY:") {
                // Handle "xxx hits you."
                let (y, x) = (st.borg.c.y, st.borg.c.x);
                borg_fear_regional(st, what, y, x, 4 * ((depth / 5) + 1), false);
                set_msg_use(st, i, 5);
            } else if msg.starts_with(b"MISS_BY:") {
                // Handle "xxx misses you."
                let (y, x) = (st.borg.c.y, st.borg.c.x);
                borg_fear_regional(st, what, y, x, 2 * ((depth / 5) + 1), false);
                set_msg_use(st, i, 5);
            } else if msg.starts_with(b"SPELL_") {
                // Handle "spell"
                let (y, x) = (st.borg.c.y, st.borg.c.x);
                let k = borg_fear_spell(st, atoi(&msg[6..]));
                borg_fear_regional(st, what, y, x, k, false);
                set_msg_use(st, i, 5);
            }
        }
        i += 1;
    }

    // Display messages
    let mut i = 0;
    while i < i32::from(st.msgs.num) {
        let mut note = b"# ".to_vec();
        note.extend_from_slice(pct_s(&msg_text(st, i)));
        note.extend_from_slice(format!(" ({})", st.msgs.use_[idx(i)]).as_bytes());
        borg_note_bytes(st, &note);
        i += 1;
    }
}

/// Spread the monsters' danger as regional fear and count the breeders
/// (monsters probably in a vault are skipped).
fn spread_monster_fear(st: &mut BorgState) {
    st.borg.mon.breeders = 0;
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[idx(i)];
        i += 1;

        // Reset the 'vault monsters
        let mut monster_in_vault = false;

        // Skip dead monsters
        if kill.r_idx == 0 {
            continue;
        }

        let r_idx = usize::from(kill.r_idx);
        if engine::race_flag_has(r_idx, RF_MULTIPLY) {
            st.borg.mon.breeders = st.borg.mon.breeders.wrapping_add(1);
        }

        // Skip monsters that dont chase
        if engine::race_flag_has(r_idx, RF_NEVER_MOVE) {
            continue;
        }

        // Skip monsters that are far away
        if engine::distance(kill.pos, st.borg.c) >= 20 {
            continue;
        }

        // Skip monsters in vaults
        if st.borg.status.vault {
            // Check adjacent grids to monster
            for ii in 0..8 {
                // Grid in that direction
                let x = kill.pos.x + i32::from(engine::ddx_ddd(ii));
                let y = kill.pos.y + i32::from(engine::ddy_ddd(ii));

                // Legal grid
                if !engine::square_in_bounds_fully(x, y) {
                    continue;
                }

                // Skip unknown grids (important); mark a Vault monster
                let feat = u32::from(st.cave.grid(x, y).feat);
                if feat == FEAT_NONE {
                    continue;
                }
                if feat == FEAT_PERM {
                    monster_in_vault = true;
                }
            }
        }

        // Monster is probably in a vault, ignore the regional fear
        if monster_in_vault {
            continue;
        }

        // Obtain some danger
        let p = borg_danger(st, kill.pos.y, kill.pos.x, 1, false, false) / 10;

        // Apply the Fear
        borg_fear_grid(st, kill.pos.y, kill.pos.x, p);
    }
}

/// Free the detection arrays (`borg_free_detection`). Nothing (not even
/// `old_panels`) changes when none is allocated.
pub fn borg_free_detection(st: &mut BorgState) {
    let u = &mut st.update;
    if u.detect_wall.is_empty()
        && u.detect_trap.is_empty()
        && u.detect_door.is_empty()
        && u.detect_evil.is_empty()
        && u.detect_obj.is_empty()
    {
        return;
    }

    u.detect_wall = Vec::new();
    u.detect_trap = Vec::new();
    u.detect_door = Vec::new();
    u.detect_evil = Vec::new();
    u.detect_obj = Vec::new();

    u.old_panels.x = -1;
    u.old_panels.y = -1;
}

/// (Re)allocate the detection arrays, cleared, when the number of panels
/// has changed (`borg_alloc_detection`). They are one larger than the
/// number of panels each way.
pub fn borg_alloc_detection(st: &mut BorgState) {
    let panels = st.borg.panels;
    let array_size_y = idx(panels.y + 1);
    let array_size_x = idx(panels.x + 1);

    // only reallocate the detection arrays if the number of panels has
    // changed
    if st.update.old_panels.x == panels.x && st.update.old_panels.y == panels.y {
        return;
    }

    borg_free_detection(st);

    let u = &mut st.update;
    let fresh = || vec![vec![false; array_size_x]; array_size_y];
    u.detect_wall = fresh();
    u.detect_trap = fresh();
    u.detect_door = fresh();
    u.detect_evil = fresh();
    u.detect_obj = fresh();

    u.old_panels = panels;
}

/// Allocate the tracking list and forget the map (`borg_init_update`).
pub fn borg_init_update(st: &mut BorgState) {
    borg_free_update(st);

    // Array of "wanks"
    st.update.wanks = vec![BorgWank::default(); idx(AUTO_VIEW_MAX)];

    // *** Reset the map ***
    borg_forget_map(st);
}

/// Free the tracking list (`borg_free_update`).
pub fn borg_free_update(st: &mut BorgState) {
    st.update.wanks = Vec::new();
}

//! Port of `borg-flow-misc.c`: misc movement (flow) routines.
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
//! The store door locations (`track_shop_x/y`), the mineral veins with
//! treasure (`track_vein`) and the spastic search target (`spastic_x/y`)
//! live in `FlowMiscState`. The flow core is `borg_flow`.
//! C bugs kept are listed in `docs/c-bugs.md` (section 39).

use crate::borg::BorgTime;
use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_light::AUTO_LIGHT_MAX;
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid};
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_danger::{borg_danger, borg_danger_one_kill};
use crate::borg_escape::{borg_allow_teleport, borg_caution_phase, borg_shadow_shift};
use crate::borg_flow::{
    borg_can_dig, borg_flow_clear, borg_flow_commit, borg_flow_enqueue_grid, borg_flow_old,
    borg_flow_spread, flow_commit,
};
use crate::borg_flow_stairs::{borg_flow_cost_stair, StairTrack};
use crate::borg_io::{borg_keypress, borg_note, KC_ENTER};
use crate::borg_item_use::{borg_activate_item, borg_read_scroll};
use crate::borg_magic::{borg_primarily_caster, borg_spell_fail};
use crate::borg_projection::{borg_distance, borg_los};
use crate::borg_update::Detect;
use crate::borg_headers::borg_magic_h::{PHASE_DOOR, PORTAL};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_GOLD, BI_IFIRE, BI_ISAFRAID, BI_ISCUT,
    BI_ISHEAVYSTUN, BI_ISHUNGRY, BI_ISSTUN, BI_ISWEAK, BI_LIGHT, BI_MAXHP, BI_MAXSP,
};
use crate::headers::list_mon_race_flags_h::{RF_KILL_WALL, RF_MULTIPLY, RF_NEVER_MOVE, RF_PASS_WALL};
use crate::headers::list_player_flags_h::PF_COMBAT_REGEN;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_GRANITE, FEAT_LAVA, FEAT_LESS, FEAT_MAGMA, FEAT_MAGMA_K,
    FEAT_MORE, FEAT_NONE, FEAT_OPEN, FEAT_PASS_RUBBLE, FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K,
    FEAT_RUBBLE, FEAT_SECRET,
};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;
use crate::engine;

/// Goals (borg-flow.h).
const GOAL_TAKE: i32 = 2;
const GOAL_MISC: i32 = 3;
const GOAL_XTRA: i32 = 5;
const GOAL_VAULT: i32 = 8;
const GOAL_RECOVER: i32 = 9;

/// `BORG_GLOW` (borg-cave.h grid info bit).
const BORG_GLOW: u16 = 0x02;

/// Number of store doors tracked (`borg_init_flow_misc()`).
const TRACK_SHOP_SIZE: usize = 9;

/// `track_vein` allocation size (`borg_init_flow_misc()`).
const TRACK_VEIN_SIZE: i16 = 100;

/// borg-flow-misc.c globals and file statics.
#[derive(Default)]
pub struct FlowMiscState {
    /// `track_shop_x`: the store doors (9 entries, empty = NULL)
    pub track_shop_x: Vec<i32>,
    /// `track_shop_y`
    pub track_shop_y: Vec<i32>,
    /// `track_vein`: the mineral veins with treasure (the same
    /// `struct borg_track` as the stair lists; C `num`/`size` semantics)
    pub track_vein: StairTrack,
    /// `spastic_x`: the grid the Borg flows to, to search (0 = none)
    pub spastic_x: u8,
    /// `spastic_y`
    pub spastic_y: u8,
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `SCREEN_HGT`, `SCREEN_WID` (ui-term.h)
fn screen_hgt_wid() -> (i32, i32) {
    let t = engine::term_geometry();
    (
        (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height,
        (t.wid - t.col_map - 1) / t.tile_width,
    )
}

/// `I2D(dir)`: the key of a direction.
fn i2d(dir: i32) -> keycode_t {
    (dir + i32::from(b'0')) as keycode_t
}

fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// The index of the farthest up staircase in `track_less` (the first of
/// equally far ones), or -1 (the "Check distance away from stairs" loop).
fn farthest_up_stair(st: &BorgState) -> i32 {
    let mut b_j = -1;
    let mut b_stair = -1;

    /* Check for an existing "up stairs" */
    let mut i = 0;
    while i < i32::from(st.stairs.less.num) {
        let x = st.stairs.less.x[i as usize];
        let y = st.stairs.less.y[i as usize];

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
    b_stair
}

/// Commit the flow and take one step (the common end of the flows).
fn commit_and_step(st: &mut BorgState, who: &std::ffi::CStr, why: i32) -> bool {
    /* Attempt to Commit the flow */
    if !borg_flow_commit(st, who, why) {
        return false;
    }

    /* Take one step */
    if !borg_flow_old(st, why) {
        return false;
    }

    /* Success */
    true
}

/// Enqueue the `borg_temp_*` grids in list order.
fn enqueue_temp(st: &mut BorgState) {
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        /* Enqueue the grid */
        let (y, x) = st.flow.temp_at(i as usize);
        borg_flow_enqueue_grid(st, y, x);
        i += 1;
    }
}

/// Do a "reverse" flow from the player outwards
pub fn borg_flow_reverse(
    st: &mut BorgState,
    depth: i32,
    optimize: bool,
    avoid: bool,
    tunneling: bool,
    stair_idx: i32,
    sneak: bool,
) {
    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Enqueue the player's grid */
    borg_flow_enqueue_grid(st, st.borg.c.y, st.borg.c.x);

    /* Spread, but do NOT optimize */
    borg_flow_spread(st, depth, optimize, avoid, tunneling, stair_idx, sneak);
}

/// Get the borgs "leash"
/// This is the distance from the stairs the borg can explore before
/// returning to the stairs and trying to explore in anohter direction.
/// The leash is different for exploring vs trying to get something.
pub fn borg_get_leash(st: &BorgState, pick_up: bool) -> i32 {
    let mut leash = 250;

    if pick_up && st.borg.tr(BI_CLEVEL) < 20 {
        leash = st.borg.tr(BI_CLEVEL) * 3 + 9;
    }

    if !pick_up && st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - 5 {
        leash = st.borg.tr(BI_CLEVEL) * 3 + 9;
    }

    /* if the borg has run out of things to do, allow him to go a */
    /* little further afield */
    if st.borg.times_twitch > 21 {
        leash += i32::from(st.borg.times_twitch);
    }

    leash
}

/// Check a floor grid for "happy" status
///
/// These grids are floor grids which contain stairs, or which
/// are non-corners in corridors, or which are directly adjacent
/// to pillars, or grids which we have stepped on before.
///  Stairs are good because they can be used to leave
/// the level.  Corridors are good because you can back into them
/// to avoid groups of monsters and because they can be used for
/// escaping.  Pillars are good because while standing next to a
/// pillar, you can walk "around" it in two different directions,
/// allowing you to retreat from a single normal monster forever.
/// Stepped on grids are good because they likely stem from an area
/// which has been cleared of monsters.
pub fn borg_happy_grid_bold(st: &BorgState, y: i32, x: i32) -> bool {
    /* Bounds Check */
    if y >= AUTO_MAX_Y - 2 || y <= 2 || x >= AUTO_MAX_X - 2 || x <= 2 {
        return false;
    }

    let ag = st.cave.grid(x, y);
    let feat = u32::from(ag.feat);

    /* Accept stairs */
    if feat == FEAT_LESS {
        return true;
    }
    if feat == FEAT_MORE {
        return true;
    }
    if ag.glyph {
        return true;
    }
    if feat == FEAT_LAVA && st.borg.tr(BI_IFIRE) == 0 {
        return false;
    }

    /* Weak/dark is very unhappy */
    if st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }

    /* Apply a control effect so that he does not get stuck in a loop */
    if borg_timer(st, st.borg.time.level) >= 2000 {
        return false;
    }

    let fl = |dy: i32, dx: i32| borg_cave_floor_bold(st, y + dy, x + dx);

    /* Case 1a: north-south corridor */
    if fl(-1, 0)
        && fl(1, 0)
        && !fl(0, -1)
        && !fl(0, 1)
        && !fl(1, -1)
        && !fl(1, 1)
        && !fl(-1, -1)
        && !fl(-1, 1)
    {
        /* Happy */
        return true;
    }

    /* Case 1b: east-west corridor */
    if fl(0, -1)
        && fl(0, 1)
        && !fl(-1, 0)
        && !fl(1, 0)
        && !fl(1, -1)
        && !fl(1, 1)
        && !fl(-1, -1)
        && !fl(-1, 1)
    {
        /* Happy */
        return true;
    }

    /* Case 1aa: north-south doorway */
    if fl(-1, 0) && fl(1, 0) && !fl(0, -1) && !fl(0, 1) {
        /* Happy */
        return true;
    }

    /* Case 1ba: east-west doorway */
    if fl(0, -1) && fl(0, 1) && !fl(-1, 0) && !fl(1, 0) {
        /* Happy */
        return true;
    }

    /* Case 2a: north pillar */
    if !fl(-1, 0) && fl(-1, -1) && fl(-1, 1) && fl(-2, 0) {
        /* Happy */
        return true;
    }

    /* Case 2b: south pillar */
    if !fl(1, 0) && fl(1, -1) && fl(1, 1) && fl(2, 0) {
        /* Happy */
        return true;
    }

    /* Case 2c: east pillar */
    if !fl(0, 1) && fl(-1, 1) && fl(1, 1) && fl(0, 2) {
        /* Happy */
        return true;
    }

    /* Case 2d: west pillar */
    if !fl(0, -1) && fl(-1, -1) && fl(1, -1) && fl(0, -2) {
        /* Happy */
        return true;
    }

    /* check for grids that have been stepped on before */
    let t = &st.flow.track_step;
    let mut i = 0;
    while i < i32::from(t.num) {
        /* Enqueue the grid */
        if t.y[i as usize] == y && t.x[i as usize] == x {
            /* Recent step is good */
            if i < 25 {
                return true;
            }
        }
        i += 1;
    }

    /* Not happy */
    false
}

/// Attempt to flow to a safe grid in order to rest up properly.  Following a
/// battle, a borg needs to heal up. He will attempt to heal up right where the
/// fight was, but if he cannot, then he needs to retreat a bit. This will help
/// him find a good safe place to hide.
pub fn borg_flow_recover(st: &mut BorgState, dist: i32) -> bool {
    let b = &st.borg;

    /* Sometimes we loop on this */
    if b.antibounce_count > 500 {
        return false;
    }

    /* No retreating and recovering when low level */
    if b.tr(BI_CLEVEL) <= 5 {
        return false;
    }

    /* Mana for spell casters */
    if borg_primarily_caster() {
        if b.tr(BI_CURHP) > b.tr(BI_MAXHP) / 3
            && (b.tr(BI_CURSP) > b.tr(BI_MAXSP) / 4 || b.tr(BI_MAXSP) == 0)
            /* Non spell casters? */
            && b.tr(BI_ISCUT) == 0
            && b.tr(BI_ISSTUN) == 0
            && b.tr(BI_ISHEAVYSTUN) == 0
            && b.tr(BI_ISAFRAID) == 0
        {
            return false;
        }
    } else
    /* Non Spell Casters */
    {
        /* do I need to recover some? */
        if b.tr(BI_CURHP) > b.tr(BI_MAXHP) / 3
            && b.tr(BI_ISCUT) == 0
            && b.tr(BI_ISSTUN) == 0
            && b.tr(BI_ISHEAVYSTUN) == 0
            && b.tr(BI_ISAFRAID) == 0
        {
            return false;
        }
    }

    /* If Fleeing, then do not rest */
    if b.goal.fleeing {
        return false;
    }

    /* If Scumming, then do not rest */
    if b.lunal_mode || b.munchkin_mode {
        return false;
    }

    /* No need if hungry */
    if b.tr(BI_ISHUNGRY) != 0 {
        return false;
    }

    /* Nothing found */
    st.flow.temp_n = 0;

    /* Scan some known Grids
     * Favor the following types of grids:
     * 1. Happy grids
     */

    /* look at grids within 20 grids of me */
    let mut y = st.borg.c.y - 25;
    while y < st.borg.c.y + 25 {
        let mut x = st.borg.c.x - 25;
        while x < st.borg.c.x + 25 {
            if recover_grid_ok(st, y, x) {
                /* Careful -- Remember it */
                st.flow.temp_push(y, x);
            }
            x += 1;
        }
        y += 1;
    }

    /* Nothing to kill */
    if st.flow.temp_n == 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Look through the good grids */
    enqueue_temp(st);

    /* Spread the flow */
    borg_flow_spread(st, dist, false, true, false, -1, false);

    commit_and_step(st, c"Recover Grid", GOAL_RECOVER)
}

/// The tests of one grid of `borg_flow_recover()`'s scan, in C's order.
fn recover_grid_ok(st: &mut BorgState, y: i32, x: i32) -> bool {
    /* Stay in bounds */
    if !engine::square_in_bounds(x, y) {
        return false;
    }

    /* Skip my own grid */
    if y == st.borg.c.y && x == st.borg.c.x {
        return false;
    }

    /* Skip grids that are too close to me */
    if engine::distance(st.borg.c, loc { x, y }) < 7 {
        return false;
    }

    /* Is this grid a happy grid? */
    if !borg_happy_grid_bold(st, y, x) {
        return false;
    }

    /* Can't rest on a wall grid. */
    /* HACK depends on FEAT order, kinda evil */
    let feat = u32::from(st.cave.grid(x, y).feat);
    if feat >= FEAT_SECRET && feat != FEAT_PASS_RUBBLE {
        return false;
    }

    /* Can I rest on that one? */
    borg_check_rest(st, y, x)
}

/// Prepare to "flow" towards mineral veins with treasure
pub fn borg_flow_vein(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    let leash = borg_get_leash(st, true);

    /* Efficiency -- Nothing to take */
    if st.flow_misc.track_vein.num == 0 {
        return false;
    }

    /* Not needed if rich */
    if st.borg.tr(BI_GOLD) >= 100000 {
        return false;
    }

    /* Require digger, capacity, or skill to dig  */
    /* note, if twitchy we will try digging out magma */
    let mut min_feat = FEAT_QUARTZ_K as u8;
    if st.borg.times_twitch > 21 {
        min_feat = FEAT_MAGMA_K as u8;
    }
    if !borg_can_dig(st, true, min_feat) {
        return false;
    }

    /* Nothing yet */
    st.flow.temp_n = 0;

    /* Check distance away from stairs, used later */
    let b_stair = farthest_up_stair(st);

    /* Scan the vein list */
    let mut i = 0;
    while i < i32::from(st.flow_misc.track_vein.num) {
        /* Access the location */
        let x = st.flow_misc.track_vein.x[i as usize];
        let y = st.flow_misc.track_vein.y[i as usize];
        i += 1;

        /* Get the grid */
        let ag = *st.cave.grid(x, y);

        /* Require line of sight if requested */
        if viewable && ag.info & BORG_VIEW == 0 {
            continue;
        }

        /* Clear the flow codes */
        borg_flow_clear(st);

        /* obtain the number of steps from this take to the stairs */
        if nearness > 5 && st.borg.tr(BI_CLEVEL) < 20 {
            let cost = borg_flow_cost_stair(st, y, x, b_stair);

            /* Check the distance to stair for this proposed grid, unless i am
             * looking for very close items (leash) */
            if cost > leash {
                continue;
            }
        }
        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    /* Nothing to mine */
    if st.flow.temp_n == 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Look for something to take */
    enqueue_temp(st);

    /* Spread the flow */
    /* if we are not flowing toward items that we can see, make sure they */
    /* are at least easily reachable.  The second flag is weather or not  */
    /* to avoid unkown squares.  This was for performance. */
    borg_flow_spread(st, nearness, true, !viewable, false, -1, false);

    commit_and_step(st, c"vein", GOAL_TAKE)
}

/// The secret door "value" of a reachable floor grid for
/// `borg_flow_spastic()`, or `None` when the grid is skipped. `nb` are the
/// 8 grids around it in `ddx_ddd` order; `xtra` is the grid's own search
/// count.
fn spastic_value(st: &BorgState, nb: &[BorgGrid; 8], xtra: u8, cost: i32, bored: bool) -> Option<i32> {
    /* Count possible door locations */
    let wall = nb[..4].iter().filter(|g| u32::from(g.feat) >= FEAT_GRANITE).count();

    /* No possible secret doors */
    if wall < 1 {
        return None;
    }

    /* Count supporting evidence for secret doors */
    let mut supp = 0;
    for g in &nb[..4] {
        let f = u32::from(g.feat);

        /* Rubble */
        if f == FEAT_RUBBLE {
            continue;
        }

        /* Walls, Doors */
        if (FEAT_SECRET..=FEAT_GRANITE).contains(&f)
            || (f == FEAT_OPEN || f == FEAT_BROKEN)
            || f == FEAT_CLOSED
        {
            supp += 1;
        }
    }

    /* Count supporting evidence for secret doors */
    let mut diag = 0;
    for g in &nb[4..] {
        let f = u32::from(g.feat);

        /* Rubble */
        if f == FEAT_RUBBLE {
            continue;
        }

        /* Walls */
        if f >= FEAT_SECRET {
            diag += 1;
        }
    }

    /* No possible secret doors */
    if diag < 2 {
        return None;
    }

    /* Count monsters */
    let monsters = nb.iter().filter(|g| g.kill != 0).count();

    /* No search near monsters */
    if monsters >= 1 {
        return None;
    }

    /* Tweak -- Reward walls, punish visitation, distance, time on level
     * (the candidate grid's own search count: better-borg-c milestone8
     * patch 0089) */
    let mut v = (supp * 500) + (diag * 100) - (i32::from(xtra) * 40) - (cost * 2)
        - borg_timer(st, st.borg.time.level);

    /* Punish low level and searching too much */
    v -= (50 - st.borg.tr(BI_CLEVEL)) * 5;

    /* The grid is not searchable */
    if v <= 0 {
        return None;
    }

    /* Tweak -- Minimal interest until bored */
    if !bored && v < 1500 {
        return None;
    }

    Some(v)
}

/// Search carefully for secret doors and such
pub fn borg_flow_spastic(st: &mut BorgState, bored: bool) -> bool {
    let mut b_x = st.borg.c.x;
    let mut b_y = st.borg.c.y;
    let mut b_v = -1;

    /* Not in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Not if starving */
    if st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Not if hopeless unless twitchy */
    if borg_timer(st, st.borg.time.level) > 3000 && i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP) {
        return false;
    }

    /* Not bored */
    if !bored {
        /* Look around for danger */
        let p = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);

        /* Avoid searching when in danger */
        if p > i32::from(st.borg.avoidance) / 4 {
            return false;
        }
    }

    /* Check distance away from stairs, used later */
    let b_stair = farthest_up_stair(st);

    /* We have arrived */
    if i32::from(st.flow_misc.spastic_x) == st.borg.c.x
        && i32::from(st.flow_misc.spastic_y) == st.borg.c.y
    {
        /* Cancel */
        st.flow_misc.spastic_x = 0;
        st.flow_misc.spastic_y = 0;

        let (cx, cy) = (st.borg.c.x, st.borg.c.y);
        let xtra = st.cave.grid(cx, cy).xtra;

        /* Take note */
        borg_note(st, &format!("# Spastic Searching at ({cx},{cy})...value:{xtra}"));

        /* Count searching */
        for i in 0..9 {
            /* Extract the location */
            let xx = cx + i32::from(engine::ddx_ddd(i));
            let yy = cy + i32::from(engine::ddy_ddd(i));

            /* Current grid */
            let ag = st.cave.grid_mut(xx, yy);

            /* Tweak -- Remember the search */
            if ag.xtra < 100 {
                ag.xtra += 5;
            }
        }

        /* we searched here */
        return false;
    }

    /* Reverse flow, not optimized: it starts at the borg's own grid (cost 0)
     * and an optimized spread stops before reaching any other grid. */
    borg_flow_reverse(st, 250, false, false, false, -1, false);

    let panel_hgt = borg_panel_hgt();
    let panel_wid = borg_panel_wid();
    let clevel = st.borg.tr(BI_CLEVEL);

    /* Scan the entire map */
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            /* Acquire the grid */
            let ag = *st.cave.grid(x, y);

            /* Skip unknown grids */
            if u32::from(ag.feat) == FEAT_NONE {
                continue;
            }

            /* Skip trap grids */
            if ag.trap {
                continue;
            }

            /* Skip walls/doors */
            if !borg_cave_floor_grid(&ag) {
                continue;
            }

            /* Acquire the cost */
            let cost = i32::from(st.flow.cost_at(y, x));

            /* Skip "unreachable" grids */
            if cost >= 250 {
                continue;
            }

            /* Skip grids that are really far away.  He probably
             * won't find anything and it takes lots of turns
             */
            if cost >= 25 && clevel < 30 {
                continue;
            }
            if cost >= 50 {
                continue;
            }

            /* Tweak -- Limit total searches */
            if ag.xtra >= 50 {
                continue;
            }
            if i32::from(ag.xtra) >= clevel {
                continue;
            }

            /* Limit initial searches until bored */
            if !bored && ag.xtra > 5 {
                continue;
            }

            /* Avoid searching detected sectors */
            if st.update.detect(Detect::Door, y / panel_hgt, x / panel_wid) {
                continue;
            }

            /* Skip ones that make me wander too far unless twitchy (Leash)*/
            if b_stair != -1 && clevel < 15 && i32::from(st.borg.avoidance) <= st.borg.tr(BI_CURHP) {
                let (sy, sx) = (st.stairs.less.y[b_stair as usize], st.stairs.less.x[b_stair as usize]);

                /* Check the distance of this grid to the stair */
                let j = borg_distance(sy, sx, y, x);
                /* Distance of me to the stairs */
                let b_j = borg_distance(st.borg.c.y, st.borg.c.x, sy, sx);

                /* skip far away grids while I am close to stair*/
                if b_j <= clevel * 3 + 9 && j >= clevel * 3 + 9 {
                    continue;
                }

                /* If really low level don't do this much */
                if clevel <= 3 && b_j <= clevel + 9 && j >= clevel + 9 {
                    continue;
                }

                /* Do not Venture too far from stair */
                if clevel <= 3 && j >= clevel + 5 {
                    continue;
                }

                /* Do not Venture too far from stair */
                if clevel <= 10 && j >= clevel + 9 {
                    continue;
                }
            }

            /* Extract adjacent locations */
            let mut nb = [BorgGrid::default(); 8];
            for (i, g) in nb.iter_mut().enumerate() {
                /* Extract the location */
                let xx = x + i32::from(engine::ddx_ddd(i));
                let yy = y + i32::from(engine::ddy_ddd(i));

                /* Get the grid contents */
                *g = *st.cave.grid(xx, yy);
            }

            let Some(v) = spastic_value(st, &nb, ag.xtra, cost, bored) else {
                continue;
            };

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

    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Nothing found */
    if b_v < 0 {
        return false;
    }

    /* Memorize */
    st.flow_misc.spastic_x = b_x as u8;
    st.flow_misc.spastic_y = b_y as u8;

    /* Enqueue the grid */
    borg_flow_enqueue_grid(st, b_y, b_x);

    /* Spread the flow */
    borg_flow_spread(st, 250, true, false, false, -1, false);

    commit_and_step(st, c"spastic", GOAL_XTRA)
}

/// Prepare to "flow" towards a specific shop entry
pub fn borg_flow_shop_entry(st: &mut BorgState, i: i32) -> bool {
    /* Must be in town */
    if st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }

    /* Obtain the location */
    let x = st.flow_misc.track_shop_x[i as usize];
    let y = st.flow_misc.track_shop_y[i as usize];

    /* Ignore shop with unset (zero) coordinates */
    if x == 0 || y == 0 {
        return false;
    }

    /* Re-enter a shop if needed */
    if x == st.borg.c.x && y == st.borg.c.y {
        /* Note */
        borg_note(st, "# Re-entering a shop");

        /* Enter the store */
        borg_keypress(st, key(b'5'));

        /* Success */
        return true;
    }

    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Enqueue the grid */
    borg_flow_enqueue_grid(st, y, x);

    /* Spread the flow */
    borg_flow_spread(st, 250, true, false, false, -1, false);

    /* Attempt to Commit the flow (as the store's feature name) */
    let name = engine::store_feat_name(i as usize);
    if !flow_commit(st, Some(&name), GOAL_MISC) {
        return false;
    }

    /* Take one step */
    if !borg_flow_old(st, GOAL_MISC) {
        return false;
    }

    /* Success */
    true
}

/// Prepare to flow towards light
pub fn borg_flow_light(st: &mut BorgState, why: i32) -> bool {
    let (screen_hgt, screen_wid) = screen_hgt_wid();
    let (w_x, w_y) = (st.core.w_x, st.core.w_y);

    /* reset counters */
    st.light.glow_n = 0;

    /* build the glow array */
    /* Scan map (a large terminal can show more than the map) */
    for y in w_y..(w_y + screen_hgt).min(AUTO_MAX_Y) {
        for x in w_x..(w_x + screen_wid).min(AUTO_MAX_X) {
            /* Not a perma-lit, and not our spot. */
            if st.cave.grid(x, y).info & BORG_GLOW == 0 {
                continue;
            }

            /* Stop when the glow arrays are full: the panel can hold
             * more glowing grids than AUTO_LIGHT_MAX. */
            if st.light.glow_n as usize >= AUTO_LIGHT_MAX {
                continue;
            }

            /* keep count */
            let l = &mut st.light;
            let n = l.glow_n as usize;
            l.glow_y[n] = y as u8;
            l.glow_x[n] = x as u8;
            l.glow_n += 1;
        }
    }
    /* None to flow to */
    if st.light.glow_n == 0 {
        return false;
    }

    /* Clear the flow codes */
    borg_flow_clear(st);

    /* Enqueue useful grids */
    let mut i = 0;
    while i < i32::from(st.light.glow_n) {
        /* Enqueue the grid */
        let y = i32::from(st.light.glow_y[i as usize]);
        let x = i32::from(st.light.glow_x[i as usize]);
        borg_flow_enqueue_grid(st, y, x);
        i += 1;
    }

    /* Spread the flow */
    borg_flow_spread(st, 250, true, false, false, -1, false);

    commit_and_step(st, c"a lighted area", why)
}

/// Prepare to flow towards a vault grid which can be excavated
pub fn borg_flow_vault(st: &mut BorgState, nearness: i32) -> bool {
    /* reset counters */
    st.flow.temp_n = 0;

    /* no need if no vault on level */
    if !st.borg.status.vault {
        return false;
    }

    /* no need if we can't dig at least quartz */
    if !borg_can_dig(st, false, FEAT_QUARTZ as u8) {
        return false;
    }

    let can_dig_hard = borg_can_dig(st, false, FEAT_GRANITE as u8);

    let (screen_hgt, screen_wid) = screen_hgt_wid();
    let (w_x, w_y) = (st.core.w_x, st.core.w_y);

    /* build the array -- Scan screen (a large terminal can show more than
     * the map) */
    for y in w_y..(w_y + screen_hgt).min(AUTO_MAX_Y) {
        for x in w_x..(w_x + screen_wid).min(AUTO_MAX_X) {
            /* only bother with near ones */
            if engine::distance(st.borg.c, loc { x, y }) > nearness {
                continue;
            }

            let feat = u32::from(st.cave.grid(x, y).feat);

            /* only deal with excavatable walls */
            if feat != FEAT_RUBBLE
                && feat != FEAT_QUARTZ
                && feat != FEAT_MAGMA
                && feat != FEAT_QUARTZ_K
                && feat != FEAT_MAGMA_K
            {
                /* only deal with granite if we are good diggers */
                if !can_dig_hard || feat != FEAT_GRANITE {
                    continue;
                }
            }

            /* Examine grids adjacent to this grid to see if there is a perma
             * wall adjacent (C bug 39: the grid is listed once per wall) */
            for i in 0..8 {
                let b_x = x + i32::from(engine::ddx_ddd(i));
                let b_y = y + i32::from(engine::ddy_ddd(i));

                /* Bounds check */
                if !engine::square_in_bounds_fully(b_x, b_y) {
                    continue;
                }

                /* Not a perma, and not our spot. */
                if u32::from(st.cave.grid(b_x, b_y).feat) != FEAT_PERM {
                    continue;
                }

                /* keep count */
                st.flow.temp_push(y, x);
            }
        }
    }

    /* None to flow to */
    if st.flow.temp_n == 0 {
        return false;
    }

    /* Examine each ones (C bug 39: the flow is not cleared first) */
    enqueue_temp(st);

    /* Spread the flow */
    borg_flow_spread(st, 250, true, false, false, -1, false);

    commit_and_step(st, c"vault excavation", GOAL_VAULT)
}

/// Is the grid a wall the Borg must not twitch into (`FEAT_SECRET` to
/// `FEAT_PERM`)?
fn is_wall(g: &BorgGrid) -> bool {
    (FEAT_SECRET..=FEAT_PERM).contains(&u32::from(g.feat))
}

/// Act twitchy
pub fn borg_twitchy(st: &mut BorgState) -> bool {
    let mut dir;

    /* This is a bad thing */
    borg_note(st, "# Twitchy!");

    /* try to phase out of it */
    if borg_allow_teleport(st)
        && borg_caution_phase(st, 15, 2)
        && (borg_spell_fail(st, PHASE_DOOR, 40)
            || borg_spell_fail(st, PORTAL, 40)
            || borg_shadow_shift(st, 40)
            || borg_activate_item(st, st.act.tele_phase)
            || borg_activate_item(st, st.act.tele_long)
            || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
    {
        /* We did something */
        return true;
    }

    let afraid = |st: &BorgState| st.borg.tr(BI_ISAFRAID) != 0;

    /* Pick a random direction */
    let mut count = 20;
    loop {
        dir = engine::randint0(10);
        if dir == 5 || dir == 0 {
            continue;
        }

        if count == 0 {
            break;
        }

        count -= 1;

        /* Set goal */
        st.borg.goal.g.x = st.borg.c.x + i32::from(engine::ddx(dir as usize));
        st.borg.goal.g.y = st.borg.c.y + i32::from(engine::ddy(dir as usize));

        if !engine::square_in_bounds_fully(st.borg.goal.g.x, st.borg.goal.g.y) {
            continue;
        }

        let grid = *st.cave.grid(st.borg.goal.g.x, st.borg.goal.g.y);

        /* don't twitch into walls */
        if is_wall(&grid) {
            continue;
        }

        /* monsters count as walls if afraid */
        if grid.kill != 0 && afraid(st) {
            continue;
        }

        break;
    }

    /* if we can't find a direction, maybe we are surrounded by walls
     * (C bug 39: also after a good 20th pick) */
    if count == 0 {
        let mut all_walls = true;
        dir = 1;
        while dir < 10 {
            if dir == 5 {
                dir += 1;
                continue;
            }

            /* get the location of position + direction */
            let lx = st.borg.c.x + i32::from(engine::ddx(dir as usize));
            let ly = st.borg.c.y + i32::from(engine::ddy(dir as usize));

            if !engine::square_in_bounds_fully(lx, ly) {
                dir += 1;
                continue;
            }

            let grid = *st.cave.grid(lx, ly);

            /* don't twitch into walls */
            /* unless afraid (and even then, not perm walls) */
            if is_wall(&grid) && (!afraid(st) || u32::from(grid.feat) == FEAT_PERM) {
                dir += 1;
                continue;
            }

            /* monsters count as walls if afraid */
            if grid.kill != 0 && afraid(st) {
                dir += 1;
                continue;
            }

            all_walls = false;
            break;
        }
        if all_walls {
            /* Rest until done */
            borg_keypress(st, key(b'R'));
            borg_keypress(st, key(b'1'));
            borg_keypress(st, key(b'0'));
            borg_keypress(st, key(b'0'));
            borg_keypress(st, KC_ENTER);
            /* We did something */
            return true;
        }
    }

    /* Normally move */

    /* if afraid, we need to try to dig. We have tried everything else */
    /* digging will at least take an action so maybe give time to not be */
    /* afraid */
    if afraid(st) {
        borg_keypress(st, key(b'+'));
    }

    /* Send direction */
    borg_keypress(st, i2d(dir));

    /* We did something */
    true
}

/// Given a "source" and "target" locations, extract a "direction",
/// which will move one step from the "source" towards the "target".
///
/// Note that we use "diagonal" motion whenever possible.
///
/// We return "5" if no motion is needed.
pub fn borg_extract_dir(y1: i32, x1: i32, y2: i32, x2: i32) -> i32 {
    /* No movement required */
    if y1 == y2 && x1 == x2 {
        return 5;
    }

    /* South or North */
    if x1 == x2 {
        return if y1 < y2 { 2 } else { 8 };
    }

    /* East or West */
    if y1 == y2 {
        return if x1 < x2 { 6 } else { 4 };
    }

    /* South-east or South-west */
    if y1 < y2 {
        return if x1 < x2 { 3 } else { 1 };
    }

    /* North-east or North-west */
    if y1 > y2 {
        return if x1 < x2 { 9 } else { 7 };
    }

    /* Paranoia */
    5
}

/// Given a "source" and "target" locations, travel in a "direction",
/// which will move one step from the "source" towards the "target".
///
/// We prefer "non-diagonal" motion, which allows us to save the
/// "diagonal" moves for avoiding pillars and other obstacles.
///
/// If no "obvious" path is available, we use "borg_extract_dir()".
///
/// We return "5" if no motion is needed.
pub fn borg_goto_dir(st: &BorgState, y1: i32, x1: i32, y2: i32, x2: i32) -> i32 {
    let ay = if y2 > y1 { y2 - y1 } else { y1 - y2 };
    let ax = if x2 > x1 { x2 - x1 } else { x1 - x2 };

    /* Is the grid one step in direction d a floor? */
    let open = |d: i32| {
        borg_cave_floor_bold(
            st,
            y1 + i32::from(engine::ddy(d as usize)),
            x1 + i32::from(engine::ddx(d as usize)),
        )
    };

    /* Default direction */
    let e = borg_extract_dir(y1, x1, y2, x2);

    /* Adjacent location, use default */
    if ax <= 1 && ay <= 1 {
        return e;
    }

    /* Try south/north (primary) */
    if ay > ax {
        let d = if y1 < y2 { 2 } else { 8 };
        if open(d) {
            return d;
        }
    }

    /* Try east/west (primary) */
    if ay < ax {
        let d = if x1 < x2 { 6 } else { 4 };
        if open(d) {
            return d;
        }
    }

    /* Try diagonal */
    let d = borg_extract_dir(y1, x1, y2, x2);

    /* Check for walls */
    if open(d) {
        return d;
    }

    /* Try south/north (secondary) */
    if ay <= ax {
        let d = if y1 < y2 { 2 } else { 8 };
        if open(d) {
            return d;
        }
    }

    /* Try east/west (secondary) */
    if ay >= ax {
        let d = if x1 < x2 { 6 } else { 4 };
        if open(d) {
            return d;
        }
    }

    /* Circle obstacles */
    if ay == 0 {
        /* Circle to the south */
        let d = if x1 < x2 { 3 } else { 1 };
        if open(d) {
            return d;
        }

        /* Circle to the north */
        let d = if x1 < x2 { 9 } else { 7 };
        if open(d) {
            return d;
        }
    }

    /* Circle obstacles */
    if ax == 0 {
        /* Circle to the east */
        let d = if y1 < y2 { 3 } else { 9 };
        if open(d) {
            return d;
        }

        /* Circle to the west */
        let d = if y1 < y2 { 1 } else { 7 };
        if open(d) {
            return d;
        }
    }

    /* Oops */
    e
}

/// check to make sure there are no monsters around that should prevent resting
pub fn borg_check_rest(st: &mut BorgState, y: i32, x: i32) -> bool {
    let mut borg_in_vault = false;

    /* never rest to recover SP (if HP at max) if you only recover */
    /* sp in combat */
    if st.borg.tr(BI_CURHP) == st.borg.tr(BI_MAXHP) && engine::player_has(PF_COMBAT_REGEN) {
        return false;
    }

    /* Do not rest recently after killing a multiplier */
    /* This will avoid the problem of resting next to */
    /* an unkown area full of breeders */
    if borg_timer(st, st.borg.time.last_kill_mult) < 4 && borg_timer(st, st.borg.time.last_kill_mult) > 0 {
        return false;
    }

    /* No resting if Blessed and good HP and good SP */
    /* don't rest for SP if you do combat regen */
    let b = &st.borg;
    let t = &b.temp;
    if (t.bless || t.hero || t.berserk || t.fastcast || t.regen || t.smite_evil)
        && !b.munchkin_mode
        && b.tr(BI_CURHP) >= b.tr(BI_MAXHP) * 8 / 10
        && b.tr(BI_CURSP) >= b.tr(BI_MAXSP) * 7 / 10
    {
        return false;
    }

    /* Most of the time, its ok to rest in a vault
     * (C bug 39: only the 2x2 grids up and left of the Borg) */
    if st.borg.status.vault {
        for i in -1..1 {
            for ii in -1..1 {
                let (gx, gy) = (st.borg.c.x + ii, st.borg.c.y + i);

                /* check bounds */
                if !engine::square_in_bounds_fully(gx, gy) {
                    continue;
                }

                if u32::from(st.cave.grid(gx, gy).feat) == FEAT_PERM {
                    borg_in_vault = true;
                }
            }
        }
    }

    let b = &st.borg;

    /* No resting to recover if I just cast a prepatory spell
     * which is what I like to do right before I take a stair,
     * Unless I am down by three quarters of my SP.
     */
    if b.no_rest_prep >= 1 && !b.munchkin_mode && b.tr(BI_CURSP) > b.tr(BI_MAXSP) / 4 && b.tr(BI_CDEPTH) < 85 {
        return false;
    }

    /* Don't rest on lava unless we are immune to fire */
    if u32::from(st.cave.grid(x, y).feat) == FEAT_LAVA && b.tr(BI_IFIRE) == 0 {
        return false;
    }

    /* Dont worry about fears if in a vault */
    if !borg_in_vault {
        /* Be concerned about the Regional Fear. */
        if i32::from(st.danger.fear_region[(y / 11) as usize][(x / 11) as usize]) > b.tr(BI_CURHP) / 20
            && b.tr(BI_CDEPTH) != 100
        {
            return false;
        }

        /* Be concerned about the Monster Fear. */
        if i32::from(st.danger.fear_monsters[y as usize][x as usize]) > b.tr(BI_CURHP) / 10
            && b.tr(BI_CDEPTH) != 100
        {
            return false;
        }

        /* Be concerned about the Monster Danger. */
        if borg_danger(st, y, x, 1, true, false) > st.borg.tr(BI_CURHP) / 40 && st.borg.tr(BI_CDEPTH) >= 85 {
            return false;
        }

        let b = &st.borg;

        /* Be concerned if low on food */
        if (b.tr(BI_LIGHT) == 0 || b.tr(BI_ISWEAK) != 0 || b.tr(BI_FOOD) < 2) && !b.munchkin_mode {
            return false;
        }
    }

    /* Examine all the monsters */
    let max_range = i32::from(engine::z_info().max_range);
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];
        let r_idx = usize::from(kill.r_idx);

        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            i += 1;
            continue;
        }

        /* Distance components */
        let ax = if x9 > x { x9 - x } else { x - x9 };
        let ay = if y9 > y { y9 - y } else { y - y9 };

        /* Distance */
        let d = ax.max(ay);

        /* Minimal distance */
        if d > max_range {
            i += 1;
            continue;
        }

        /* if too close to a Mold or other Never-Mover, don't rest */
        if d < 2 && !engine::race_flag_has(r_idx, RF_NEVER_MOVE) {
            return false;
        }
        if d == 1 {
            return false;
        }

        /* if too close to a Multiplier, don't rest */
        if d < 10 && engine::race_flag_has(r_idx, RF_MULTIPLY) {
            return false;
        }

        /* If monster is asleep, dont worry */
        if !kill.awake && d > 8 && !st.borg.munchkin_mode {
            i += 1;
            continue;
        }

        /* one call for dangers */
        let p = borg_danger_one_kill(st, y9, x9, 1, i, true, true);

        /* Ignore proximity checks while inside a vault */
        if !borg_in_vault {
            /* Real scary guys pretty close */
            if d < 5 && p > i32::from(st.borg.avoidance) / 3 && !st.borg.munchkin_mode {
                return false;
            }

            /* scary guys far away */
            /*if (d < 17 && d > 5 && (p > borg.avoidance/3)) return false; */
        }

        /* should check LOS... monster to me concerned for Ranged Attacks */
        if borg_los(st, y9, x9, y, x) && kill.ranged_attack != 0 {
            return false;
        }

        /* Special handling for the munchkin mode */
        if st.borg.munchkin_mode
            && borg_los(st, y9, x9, y, x)
            && (kill.awake && !engine::race_flag_has(r_idx, RF_NEVER_MOVE))
        {
            return false;
        }

        /* if it walks through walls, not safe */
        if engine::race_flag_has(r_idx, RF_PASS_WALL) && !borg_in_vault {
            return false;
        }
        if engine::race_flag_has(r_idx, RF_KILL_WALL) && !borg_in_vault {
            return false;
        }
        i += 1;
    }
    true
}

/// check if this spot is too far from the stairs
pub fn borg_flow_far_from_stairs(st: &mut BorgState, x: i32, y: i32, b_stair: i32) -> bool {
    let leash = borg_get_leash(st, false);
    borg_flow_far_from_stairs_dist(st, x, y, b_stair, leash)
}

/// check if this spot is too far from the stairs
pub fn borg_flow_far_from_stairs_dist(
    st: &mut BorgState,
    x: i32,
    y: i32,
    b_stair: i32,
    distance: i32,
) -> bool {
    if st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - 5 && st.borg.tr(BI_CLEVEL) < 20 {
        /* obtain the number of steps from this take to the stairs */
        let cost = borg_flow_cost_stair(st, y, x, b_stair);

        /* Check the distance to stair for this proposed grid */
        if cost > distance {
            return true;
        }
    }

    false
}

/// `borg_init_flow_misc()`.
pub fn borg_init_flow_misc(st: &mut BorgState) {
    let m = &mut st.flow_misc;

    /* Track the shop locations */
    m.track_shop_x = vec![0; TRACK_SHOP_SIZE];
    m.track_shop_y = vec![0; TRACK_SHOP_SIZE];

    /* Track mineral veins with treasure. */
    m.track_vein.init(TRACK_VEIN_SIZE);
}

/// `borg_free_flow_misc()`.
pub fn borg_free_flow_misc(st: &mut BorgState) {
    let m = &mut st.flow_misc;
    m.track_vein.free();
    m.track_shop_y = Vec::new();
    m.track_shop_x = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_dir_matches_c() {
        assert_eq!(borg_extract_dir(5, 5, 5, 5), 5);
        assert_eq!(borg_extract_dir(5, 5, 9, 5), 2);
        assert_eq!(borg_extract_dir(5, 5, 1, 5), 8);
        assert_eq!(borg_extract_dir(5, 5, 5, 9), 6);
        assert_eq!(borg_extract_dir(5, 5, 5, 1), 4);
        assert_eq!(borg_extract_dir(5, 5, 9, 9), 3);
        assert_eq!(borg_extract_dir(5, 5, 9, 1), 1);
        assert_eq!(borg_extract_dir(5, 5, 1, 9), 9);
        assert_eq!(borg_extract_dir(5, 5, 1, 1), 7);
    }

    #[test]
    fn leash() {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; 300];
        st.borg.trait_[BI_CLEVEL as usize] = 10;
        st.borg.trait_[BI_CDEPTH as usize] = 2;
        assert_eq!(borg_get_leash(&st, true), 39);
        assert_eq!(borg_get_leash(&st, false), 250);
        st.borg.trait_[BI_CDEPTH as usize] = 5;
        assert_eq!(borg_get_leash(&st, false), 39);
        st.borg.times_twitch = 22;
        assert_eq!(borg_get_leash(&st, false), 61);
        st.borg.trait_[BI_CLEVEL as usize] = 20;
        assert_eq!(borg_get_leash(&st, true), 272);
    }

    #[test]
    fn init_free() {
        let mut st = BorgState::default();
        borg_init_flow_misc(&mut st);
        assert_eq!((st.flow_misc.track_shop_x.len(), st.flow_misc.track_vein.size), (9, 100));
        assert_eq!(st.flow_misc.track_vein.x.len(), 100);
        borg_free_flow_misc(&mut st);
        assert!(st.flow_misc.track_shop_x.is_empty() && st.flow_misc.track_vein.x.is_empty());
    }
}

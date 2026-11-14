//! Port of `borg-flow-take.c`: flow (move) to an object (take).
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
//! The object list (`borg_takes`, 256 entries, entry 0 unused) lives in
//! `TakeState`. A grid's `take` field indexes it. The `Vec` is only
//! reallocated by `borg_init_flow_take` / `borg_free_flow_take`. The flow
//! itself is `borg_flow`.

use crate::borg::BorgTime;
use crate::borg_cave_view::BORG_VIEW;
use crate::borg_item::{quiver_end, quiver_start};
use crate::borg_projection::borg_distance;
use crate::headers::__stddef_wchar_t_h::wchar_t;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_MUNCHKIN_START, BORG_VERBOSE};
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_AMMO_TVAL, BI_CLEVEL, BI_FAST_SHOTS, BI_GOLD, BI_ISIMAGE, BI_LIGHT,
    BI_QUIVER_SLOTS,
};
use crate::headers::list_kind_flags_h::KF_INSTA_ART;
use crate::headers::list_mon_race_flags_h::{RF_KILL_ITEM, RF_TAKE_ITEM};
use crate::headers::list_terrain_h::FEAT_FLOOR;
use crate::headers::list_tvals_h::{
    TV_ARROW, TV_BOLT, TV_GOLD, TV_POTION, TV_ROD, TV_SCROLL, TV_SHOT,
};
use crate::state::BorgState;
use crate::{borg_flow, borg_flow_misc, borg_io, engine};

/// `GOAL_TAKE` (borg-flow.h): the goal of flowing towards an object.
pub const GOAL_TAKE: i16 = 2;

/// `BORG_OKAY` (borg-cave.h): the grid is on the screen.
const BORG_OKAY: u16 = 0x08;

/// The number of entries of the object list.
const MAX_TAKES: usize = 256;

/// An object the Borg has seen on the floor (`borg_take`).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct BorgTake {
    /// Kind index (0 = a dead entry)
    pub kidx: u32,
    /// Verified kind
    pub known: bool,
    /// Assigned motion
    pub seen: bool,
    /// Unused
    pub extra: bool,
    /// Orb of Draining cast on it
    pub orbed: bool,
    /// Location
    pub x: u8,
    pub y: u8,
    /// When last seen
    pub when: BorgTime,
    /// Estimated value of the item
    pub value: i32,
    /// Its tval
    pub tval: i32,
}

/// borg-flow-take.c globals.
#[derive(Default)]
pub struct TakeState {
    /// `borg_takes`: the object list (empty = NULL, else 256 entries)
    pub takes: Vec<BorgTake>,
    /// `borg_takes_cnt`: live entries
    pub cnt: i16,
    /// `borg_takes_nxt`: the first never-used entry
    pub nxt: i16,
}

/// `PACK_SLOTS`
fn pack_slots(st: &BorgState) -> i32 {
    i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS)
}

fn note(st: &mut BorgState, parts: &[&[u8]]) {
    let v = parts.concat();
    borg_io::borg_note_bytes(st, &v);
}

/// Delete an old "object" record (`borg_delete_take`).
pub fn borg_delete_take(st: &mut BorgState, i: i32) {
    let take = st.takes.takes[i as usize];

    /* Paranoia -- Already wiped */
    if take.kidx == 0 {
        return;
    }

    /* Note */
    if st.cfg(BORG_VERBOSE) != 0 {
        if take.kidx >= u32::from(engine::z_info().k_max) {
            borg_io::borg_note(
                st,
                &format!("# Forgetting an object at ({},{})", take.y, take.x),
            );
        } else {
            note(
                st,
                &[
                    b"# Forgetting an object '",
                    engine::kind_name(take.kidx as usize),
                    format!("' at ({},{})", take.y, take.x).as_bytes(),
                ],
            );
        }
    }

    /* Forget it */
    st.cave.grid_mut(i32::from(take.x), i32::from(take.y)).take = 0;

    /* Kill the object */
    st.takes.takes[i as usize] = BorgTake::default();

    /* One less object */
    st.takes.cnt = st.takes.cnt.wrapping_sub(1);

    /* Wipe goals */
    if st.borg.goal.type_0 == GOAL_TAKE {
        st.borg.goal.type_0 = 0;
    }
}

/// Determine if an object should be "viewable" (`borg_follow_take_aux`).
fn borg_follow_take_aux(st: &BorgState, i: i32) -> bool {
    let take = &st.takes.takes[i as usize];

    /* Not on-screen */
    st.cave.grid(i32::from(take.x), i32::from(take.y)).info & BORG_OKAY != 0
}

/// Attempt to "follow" a missing object (`borg_follow_take`).
///
/// This routine is not called when the player is blind or hallucinating.
/// It just deletes objects which have disappeared. We assume that a
/// monster walking onto an object destroys the object if it has the
/// appropriate flags.
///
/// As in C, an object under the Borg is deleted first and the rest then
/// reads the wiped entry (location 0,0); see docs/c-bugs.md.
pub fn borg_follow_take(st: &mut BorgState, i: i32) {
    let take = st.takes.takes[i as usize];
    /* the grid of the object (`ag`), kept across a deletion */
    let (gx, gy) = (i32::from(take.x), i32::from(take.y));

    /* Paranoia */
    if take.kidx == 0 {
        return;
    }

    /* Old location */
    let ox = i32::from(take.x);
    let oy = i32::from(take.y);
    let old_kidx = take.kidx as usize;

    /* delete them if they are under me */
    if i32::from(take.y) == st.borg.c.y && i32::from(take.x) == st.borg.c.x {
        borg_delete_take(st, i);
    }

    /* Out of sight */
    if !borg_follow_take_aux(st, i) {
        return;
    }

    /* Some monsters won't take or crush items */
    let kill = st.cave.grid(gx, gy).kill;
    if kill != 0 {
        let r_idx = usize::from(st.kills.kills[usize::from(kill)].r_idx);
        if !engine::race_flag_has(r_idx, RF_TAKE_ITEM)
            && !engine::race_flag_has(r_idx, RF_KILL_ITEM)
        {
            return;
        }
    }

    /* Note */
    note(
        st,
        &[
            b"# There was an object '",
            engine::kind_name(old_kidx),
            format!("' at ({},{})", ox, oy).as_bytes(),
        ],
    );

    /* Kill the object */
    borg_delete_take(st, i);
}

/// Obtain a new "take" index (`borg_new_take`).
fn borg_new_take(st: &mut BorgState, kind: &engine::ObjectKind, y: i32, x: i32) -> i32 {
    /* (C gets the top object here, `o_ptr`; it is looked up below, where it
     * is read: the engine object handle cannot be kept across the `&mut st`
     * calls in between, and nothing in between changes the floor) */

    /* Look for a "dead" object */
    let mut n: i32 = -1;
    let mut i: i32 = 1;
    while n < 0 && i < i32::from(st.takes.nxt) {
        /* Reuse "dead" objects */
        if st.takes.takes[i as usize].kidx == 0 {
            n = i;
        }
        i += 1;
    }

    /* Allocate a new object */
    if n < 0 && i32::from(st.takes.nxt) < MAX_TAKES as i32 {
        /* Acquire the entry, advance */
        n = i32::from(st.takes.nxt);
        st.takes.nxt += 1;
    }

    /* Steal an old object */
    if n < 0 {
        /* Note */
        borg_io::borg_note(st, "# Too many objects");

        /* Pick a random object */
        n = engine::randint0((i32::from(st.takes.nxt) - 1) as u32) + 1;

        /* Delete it */
        borg_delete_take(st, n);
    }

    /* Count new object */
    st.takes.cnt = st.takes.cnt.wrapping_add(1);

    /* Obtain the object */
    let now = st.borg.time.now;
    let take = &mut st.takes.takes[n as usize];

    /* Save the kind */
    take.kidx = kind.kidx;
    take.tval = kind.tval;

    /* Save the location */
    take.x = x as u8;
    take.y = y as u8;

    /* Timestamp */
    take.when = now;

    /* Not had Orb of Draining cast on it */
    take.orbed = false;

    /* Assess a estimated value */
    if kind.aware {
        /* Standard Value of an item */
        take.value = kind.cost;

        /* Money needs a value */
        if take.tval == TV_GOLD as i32 {
            take.value = 30;
        }
    } else {
        take.value = 1;
    }

    /* Cheat to see if this item is ID'd or not.  We use this cheat to
     * avoid dumping an item which we know to be bad then turning around and
     * picking it up again. (C dereferences the object without a check.)
     */
    let o = engine::top_floor_object(&st.access, x, y);
    let o = o.expect("borg_new_take: no object on the grid");
    let (to_a, to_h, to_d) = o.to_ahd();
    if o.fully_known() && (to_a < 0 || to_d < 0 || to_h < 0) {
        take.value = -10;
    }

    if o.note().is_some_and(|s| s.starts_with(b"borg ignore")) {
        take.value = -10;
    }
    let take = *take;

    /* Save the index */
    st.cave.grid_mut(x, y).take = n as u8;

    /* Note */
    note(
        st,
        &[
            b"# Creating an object '",
            engine::kind_name(take.kidx as usize),
            format!("' at ({},{})", take.x, take.y).as_bytes(),
        ],
    );

    /* Wipe goals only if I have some light source */
    if st.borg.tr(BI_LIGHT) != 0 {
        st.borg.goal.type_0 = 0;
    }

    /* Force the object to sit on a floor grid */
    st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;

    /* Result */
    n
}

/// Attempt to notice a changing "take" (`observe_take_diff`).
pub fn observe_take_diff(st: &mut BorgState, y: i32, x: i32, _a: u8, _c: wchar_t) -> bool {
    /* Guess the kind (borg_guess_kind: the actual top object, a cheat) */
    let Some(o) = engine::top_floor_object(&st.access, x, y) else {
        /* Oops */
        return false;
    };
    let kind = o.kind();

    /* no new takes if hallucinations */
    if st.borg.tr(BI_ISIMAGE) != 0 {
        return false;
    }

    /* Make a new object */
    let i = borg_new_take(st, &kind, y, x);

    /* Timestamp */
    st.takes.takes[i as usize].when = st.borg.time.now;

    /* Okay */
    true
}

/// Attempt to "track" a "take" at the given location
/// (`observe_take_move`). Assume that the object has not moved more than
/// "d" grids.
pub fn observe_take_move(st: &mut BorgState, y: i32, x: i32, d: i32, a: u8, c: wchar_t) -> bool {
    /* Scan the objects */
    let mut i: i32 = 1;
    while i < i32::from(st.takes.nxt) {
        let iu = i as usize;
        i += 1;
        let take = st.takes.takes[iu];

        /* Skip dead objects */
        if take.kidx == 0 {
            continue;
        }

        /* Skip assigned objects */
        if take.seen {
            continue;
        }

        /* Extract old location */
        let ox = i32::from(take.x);
        let oy = i32::from(take.y);

        /* Calculate distance */
        let z = borg_distance(oy, ox, y, x);

        /* Possible match */
        if z > d {
            continue;
        }

        /* Access the kind */
        let k = engine::kind(take.kidx as usize);
        let image = st.borg.tr(BI_ISIMAGE) != 0;

        /* Require matching char if not hallucinating*/
        if !image && c != k.d_char {
            continue;
        }

        /* Require matching attr if not hallucinating (as in C, only for
         * potions whose attr is not 11) */
        if !image && a != k.d_attr && (k.d_attr != 11 && k.d_char == b'!' as wchar_t) {
            /* Ignore "flavored" colors */
            if k.flavor.is_null() {
                continue;
            }
        }

        /* Actual movement (?) */
        if z != 0 {
            /* Update the grids */
            st.cave.grid_mut(ox, oy).take = 0;

            /* Track it */
            let t = &mut st.takes.takes[iu];
            t.x = x as u8;
            t.y = y as u8;
            let (tx, ty) = (t.x, t.y);

            /* Update the grids */
            st.cave.grid_mut(i32::from(tx), i32::from(ty)).take = iu as u8;

            /* Note */
            note(
                st,
                &[
                    b"# Tracking an object '",
                    engine::kind_name(take.kidx as usize),
                    format!("' at ({},{}) from ({},{})", ty, tx, ox, oy).as_bytes(),
                ],
            );

            /* Clear goals */
            if st.borg.goal.type_0 == GOAL_TAKE {
                st.borg.goal.type_0 = 0;
            }
        }

        let now = st.borg.time.now;
        let t = &mut st.takes.takes[iu];

        /* Timestamp */
        t.when = now;

        /* Mark as seen */
        t.seen = true;

        /* Mark floor underneath */
        let (tx, ty) = (t.x, t.y);
        st.cave.grid_mut(i32::from(tx), i32::from(ty)).feat = FEAT_FLOOR as u8;

        /* Done */
        return true;
    }

    /* Oops */
    false
}

/// The "up stairs" scan the three flows share: the index of the nearest up
/// stair (`track_less`, the first of equally near ones) and its distance.
/// `(-1, -1)` when there is none.
fn nearest_up_stair(st: &mut BorgState) -> (i32, i32) {
    let mut b_stair = -1;
    let mut b_j = -1;
    let mut i: i32 = 0;
    while i < i32::from(st.stairs.less.num) {
        let (y, x) = st.stairs.less.at(i as usize);

        /* How far is the nearest up stairs */
        let j = engine::distance(st.borg.c, crate::headers::z_type_h::loc { x, y });

        /* skip the farther ones */
        if b_j == -1 || j < b_j {
            /* track it */
            b_j = j;
            b_stair = i;
        }
        i += 1;
    }
    (b_stair, b_j)
}

/// Enqueue the grids of `borg_temp_*` into a fresh flow and spread it (the
/// common middle of the three flows).
fn spread_to_temp(st: &mut BorgState, nearness: i32, optimize: bool, viewable: bool, sneak: bool) {
    /* Clear the flow codes */
    borg_flow::borg_flow_clear(st);

    /* Look for something to take */
    let mut i: i32 = 0;
    while i < i32::from(st.flow.temp_n) {
        /* Enqueue the grid */
        let (y, x) = st.flow.temp_at(i as usize);
        borg_flow::borg_flow_enqueue_grid(st, y, x);
        i += 1;
    }

    /* Spread the flow */
    /* if we are not flowing toward items that we can see, make sure they */
    /* are at least easily reachable.  The second flag is weather or not  */
    /* to avoid unknown squares.  This was for performance. */
    borg_flow::borg_flow_spread(st, nearness, optimize, !viewable, false, -1, sneak);
}

/// Prepare to "flow" towards objects to "take" (`borg_flow_take`).
///
/// Note that objects under the player are always deleted
pub fn borg_flow_take(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    if !take_select(st, viewable, nearness) {
        return false;
    }
    take_flow(st, viewable, nearness)
}

/// The object selection of `borg_flow_take`: fills `borg_temp_*`; false when
/// there is nothing to take.
fn take_select(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    let leash = borg_flow_misc::borg_get_leash(st, true);

    /* Missile carry limit */
    /* allow shooters to two quiver slots full */
    let qss = i32::from(engine::z_info().quiver_slot_size);
    let full_quiver = if st.borg.tr(BI_FAST_SHOTS) != 0 { (qss - 1) * 2 } else { qss - 1 };

    /* Efficiency -- Nothing to take */
    if st.takes.cnt == 0 {
        return false;
    }

    /* Require one empty slot */
    if st.items.items[(pack_slots(st) - 1) as usize].iqty != 0 {
        return false;
    }

    /* If ScaryGuy, no chasing down items */
    if st.borg.mon.scary {
        return false;
    }

    /* If out of fuel, don't mess around */
    if st.borg.tr(BI_LIGHT) == 0 {
        return false;
    }

    /* Not if sitting in a sea of runes */
    if st.borg.morgoth_position {
        return false;
    }

    /* Starting over on count */
    st.flow.temp_n = 0;

    /* Check distance away from stairs, used later */
    let (b_stair, b_j) = nearest_up_stair(st);

    /* Scan the object list */
    let mut i: i32 = 1;
    while i < i32::from(st.takes.nxt) {
        let take = st.takes.takes[i as usize];
        i += 1;

        /* Skip dead objects */
        if take.kidx == 0 {
            continue;
        }

        /* Access the location */
        let x = i32::from(take.x);
        let y = i32::from(take.y);

        /* Skip ones that make me wander too far */
        if b_stair != -1 && st.borg.tr(BI_CLEVEL) < 10 {
            /* Check the distance of this 'take' to the stair */
            let (sy, sx) = st.stairs.less.at(b_stair as usize);
            let j = borg_distance(sy, sx, y, x);
            /* skip far away takes while I am close to stair*/
            if j != 255 && (b_j <= leash && j >= leash) {
                continue;
            }
        }

        /* skip worthless items */
        if take.value <= 0 {
            continue;
        }

        /* Require line of sight if requested */
        if viewable && st.cave.grid(x, y).info & BORG_VIEW == 0 {
            continue;
        }

        /* Don't bother with ammo if I am at capacity */
        if take.tval == st.borg.tr(BI_AMMO_TVAL) && st.borg.tr(BI_AMISSILES) >= full_quiver {
            continue;
        }

        /* No need to chase certain things down after a certain amount.
         * Don't chase: Money Other spell books Wrong ammo
         */
        if st.borg.tr(BI_GOLD) >= 500000 && st.cfg(BORG_MONEY_SCUM_AMOUNT) == 0 {
            if take.tval == TV_GOLD as i32 {
                continue;
            }
            let kidx = take.kidx as usize;
            if engine::tval_is_book_k(kidx) && !engine::obj_kind_can_browse(kidx) {
                continue;
            }
            if (take.tval == TV_SHOT as i32
                || take.tval == TV_ARROW as i32
                || take.tval == TV_BOLT as i32)
                && take.tval != st.borg.tr(BI_AMMO_TVAL)
            {
                continue;
            }
        }

        /* Clear the flow codes */
        borg_flow::borg_flow_clear(st);

        /* Check the distance to stair for this proposed grid and leash*/
        if nearness > 5
            && st.borg.tr(BI_CLEVEL) < 20
            && crate::borg_flow_stairs::borg_flow_cost_stair(st, y, x, b_stair) > leash
        {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    /* Nothing to take */
    st.flow.temp_n != 0
}

/// The flow part of `borg_flow_take`, after `take_select`.
fn take_flow(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    spread_to_temp(st, nearness, true, viewable, false);

    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, c"item", i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Take one step */
    if !borg_flow::borg_flow_old(st, i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Success */
    true
}

/// Prepare to "flow" towards special objects to "take"
/// (`borg_flow_take_scum`).
pub fn borg_flow_take_scum(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    if !scum_select(st, viewable) {
        return false;
    }
    scum_flow(st, viewable, nearness)
}

/// The object selection of `borg_flow_take_scum`: fills `borg_temp_*`; false when
/// there is nothing to take.
fn scum_select(st: &mut BorgState, viewable: bool) -> bool {
    /* Efficiency -- Nothing to take */
    if st.takes.cnt == 0 {
        return false;
    }

    /* Require one empty slot */
    if st.items.items[(pack_slots(st) - 1) as usize].iqty != 0 {
        return false;
    }

    /* Nothing yet */
    st.flow.temp_n = 0;

    /* Check for an existing "up stairs" */
    let (b_stair, _) = nearest_up_stair(st);

    /* Scan the object list -- set filter*/
    let mut i: i32 = 1;
    while i < i32::from(st.takes.nxt) {
        let take = st.takes.takes[i as usize];
        i += 1;

        /* Skip dead objects */
        if take.kidx == 0 {
            continue;
        }

        /* Access the location */
        let x = i32::from(take.x);
        let y = i32::from(take.y);

        /* skip worthless items */
        if take.value <= 0 {
            continue;
        }

        /* Require line of sight if requested */
        if viewable && st.cave.grid(x, y).info & BORG_VIEW == 0 {
            continue;
        }

        /* don't go too far from the stairs */
        if borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair) {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    /* Nothing to take */
    st.flow.temp_n != 0
}

/// The flow part of `borg_flow_take_scum`, after `scum_select`.
fn scum_flow(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    spread_to_temp(st, nearness, true, viewable, true);

    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, c"Scum item", i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Take one step */
    if !borg_flow::borg_flow_old(st, i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Success */
    true
}

/// Prepare to "flow" towards special objects to "take"
/// (`borg_flow_take_lunal`).
///
/// As in C, dead entries (`kidx == 0`) are skipped, and the "bad item"
/// verdict is used: an item the routine marked bad is not a target outside
/// munchkin mode (0101/0101a); see docs/c-bugs.md.
pub fn borg_flow_take_lunal(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    if !lunal_select(st, viewable) {
        return false;
    }
    lunal_flow(st, viewable, nearness)
}

/// The object selection of `borg_flow_take_lunal`: fills `borg_temp_*`; false when
/// there is nothing to take.
fn lunal_select(st: &mut BorgState, viewable: bool) -> bool {
    /* Efficiency -- Nothing to take */
    if st.takes.cnt == 0 {
        return false;
    }

    /* Check for an existing "up stairs" */
    let (b_stair, _) = nearest_up_stair(st);

    /* Nothing yet */
    st.flow.temp_n = 0;

    let z = engine::z_info();
    let qss = i32::from(z.quiver_slot_size);

    /* Scan the object list -- set filter*/
    let mut i: i32 = 1;
    while i < i32::from(st.takes.nxt) {
        let take = st.takes.takes[i as usize];
        i += 1;
        /* Skip dead objects */
        if take.kidx == 0 {
            continue;
        }
        let kidx = take.kidx as usize;
        let k = engine::kind(kidx);

        /* Access the location */
        let x = i32::from(take.x);
        let y = i32::from(take.y);
        let lunal_note = |st: &mut BorgState| {
            note(
                st,
                &[b"# Lunal Item ", engine::kind_name(kidx), format!(", at {},{}", y, x).as_bytes()],
            );
        };

        /* all items start bad */
        let mut item_bad = true;

        /* Gold is good to have */
        if take.tval == TV_GOLD as i32 {
            lunal_note(st);
            item_bad = false;
        }

        let pack_slots = pack_slots(st);

        /* If full can I absorb the item into an existing stack */
        if item_bad && take.value > 0 {
            if crate::borg_item_wear::borg_is_ammo(take.tval) {
                /* Scan the quiver */
                for ii in quiver_start()..quiver_end() {
                    let it = &st.items.items[ii as usize];

                    /* skip empty slots */
                    if it.iqty == 0 {
                        continue;
                    }

                    /* skip full slots */
                    if i32::from(it.iqty) == qss {
                        continue;
                    }

                    /* Both objects should have the same kind value */
                    if take.kidx != it.kind {
                        continue;
                    }

                    if k.sval == i32::from(it.sval) && k.tval == i32::from(it.tval) {
                        item_bad = false;
                    }
                }
            } else if st.items.items[(pack_slots - 1) as usize].iqty != 0 {
                /* Scan the inventory */
                for ii in 0..pack_slots {
                    let it = &st.items.items[ii as usize];

                    /* skip empty slots */
                    if it.iqty == 0 {
                        continue;
                    }

                    /* Both objects should have the same kind value */
                    if take.kidx != it.kind {
                        continue;
                    }

                    /* Certain types of items can stack */
                    if k.sval == i32::from(it.sval)
                        && k.tval == i32::from(it.tval)
                        && (u32::from(it.tval) == TV_POTION
                            || u32::from(it.tval) == TV_SCROLL
                            || u32::from(it.tval) == TV_ROD)
                    {
                        item_bad = false;
                    }
                }
            }
        }

        /* Require one empty slot */
        if st.items.items[(pack_slots - 1) as usize].iqty == 0 && item_bad {
            /* for ammo, make sure the quiver isn't full */
            if !crate::borg_item_wear::borg_is_ammo(take.tval)
                || st.items.items[(quiver_end() - 1) as usize].iqty == 0
            {
                /* Certain Potions are worthless */
                if take.tval == TV_POTION as i32
                    && k.sval >= st.item_val.sv_potion_inc_str
                    && k.sval <= st.item_val.sv_potion_detect_invis
                {
                    lunal_note(st);
                    item_bad = false;
                }

                /* Certain insta_arts are good.  Note that there is no top
                 * end of this.  So if an item were added after the last
                 * artifact, it would also be picked up.
                 */
                if engine::flag_has(&k.kind_flags, KF_INSTA_ART as i32) {
                    lunal_note(st);
                    item_bad = false;
                }

                /* if scumming the start of the game, take all items to
                 * sell them */
                if st.cfg(BORG_MUNCHKIN_START) != 0 {
                    /* Certain known items are junky and should be ignored.
                     * Grab only things of value
                     */
                    if take.value >= 1 {
                        item_bad = false;
                    }
                }
            }
        }
        /* Skip items the Borg does not want (outside munchkin collection) */
        if item_bad && !st.borg.munchkin_mode {
            continue;
        }

        /* Require line of sight if requested */
        if viewable && st.cave.grid(x, y).info & BORG_VIEW == 0 {
            continue;
        }

        /* don't go too far from the stairs */
        if borg_flow_misc::borg_flow_far_from_stairs(st, x, y, b_stair) {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
    }

    /* Nothing to take */
    st.flow.temp_n != 0
}

/// The flow part of `borg_flow_take_lunal`, after `lunal_select`.
fn lunal_flow(st: &mut BorgState, viewable: bool, nearness: i32) -> bool {
    spread_to_temp(st, nearness, false, viewable, true);

    /* Attempt to Commit the flow */
    if !borg_flow::borg_flow_commit(st, c"munchkin item", i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Check for monsters before walking over to the item */
    if crate::borg_light::borg_check_light(st) {
        return true;
    }

    /* Take one step */
    if !borg_flow::borg_flow_old(st, i32::from(GOAL_TAKE)) {
        return false;
    }

    /* Success */
    true
}

/// Allocate the object list (`borg_init_flow_take`).
pub fn borg_init_flow_take(st: &mut BorgState) {
    borg_free_flow_take(st);

    /*** Object tracking ***/

    /* No objects yet */
    st.takes.cnt = 0;
    st.takes.nxt = 1;

    /* Array of objects */
    st.takes.takes = vec![BorgTake::default(); MAX_TAKES];
}

/// Free the object list (`borg_free_flow_take`).
pub fn borg_free_flow_take(st: &mut BorgState) {
    st.takes.takes = Vec::new();
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn init_and_free() {
        let mut st = BorgState::default();
        borg_init_flow_take(&mut st);
        assert_eq!(st.takes.takes.len(), MAX_TAKES);
        assert_eq!((st.takes.cnt, st.takes.nxt), (0, 1));
        borg_free_flow_take(&mut st);
        assert!(st.takes.takes.is_empty());
    }

    #[test]
    fn nearest_up_stair_is_the_closest_and_first_on_a_tie() {
        let mut st = BorgState::default();
        st.borg.c.x = 0;
        st.borg.c.y = 0;
        st.stairs.less.init(3);
        st.stairs.less.num = 2;
        // Farther stair first: v11 keeps the nearer one.
        st.stairs.less.x[0] = 10;
        st.stairs.less.y[0] = 0;
        st.stairs.less.x[1] = 2;
        st.stairs.less.y[1] = 0;
        assert_eq!(nearest_up_stair(&mut st), (1, 2));

        // Equal distance: the first one wins.
        st.stairs.less.x[0] = 2;
        assert_eq!(nearest_up_stair(&mut st), (0, 2));
    }
}

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
use crate::borg_projection::borg_distance;
use crate::headers::__stddef_wchar_t_h::wchar_t;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_VERBOSE};
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_AMMO_TVAL, BI_CLEVEL, BI_FAST_SHOTS, BI_GOLD, BI_ISIMAGE, BI_LIGHT,
    BI_QUIVER_SLOTS,
};
use crate::headers::list_mon_race_flags_h::{RF_KILL_ITEM, RF_TAKE_ITEM};
use crate::headers::list_terrain_h::FEAT_FLOOR;
use crate::headers::list_tvals_h::{
    TV_ARROW, TV_BOLT, TV_GOLD, TV_SCROLL, TV_SHOT,
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
#[derive(Clone, PartialEq)]
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
    let mut phase_pruned: Vec<(i32, i32)> = Vec::new();
    if !take_select(st, viewable, nearness, &mut phase_pruned) {
        return false;
    }
    if take_flow(st, viewable, nearness) {
        return true;
    }
    // borg-reborn-rs (#761, review of #778): the Phase-only shortlist can
    // fail where the full list would not — a Phase Door past the flow's
    // depth (`borg_flow_spread`'s limit) or behind danger suppresses the
    // reachable loot. Restore what the rule dropped and flow once more.
    if phase_pruned.is_empty() {
        return false;
    }
    for &(y, x) in &phase_pruned {
        st.flow.temp_push(y, x);
    }
    take_flow(st, viewable, nearness)
}

/// The object selection of `borg_flow_take`: fills `borg_temp_*`; false when
/// there is nothing to take. `phase_pruned` gets the candidates the
/// Phase-first rule dropped, for the retry in `borg_flow_take`.
fn take_select(st: &mut BorgState, viewable: bool, nearness: i32, phase_pruned: &mut Vec<(i32, i32)>) -> bool {
    let leash = borg_flow_misc::borg_get_leash(st, true);
    // borg-reborn-rs (#840): a short kit puts both leash checks on at any level.
    let kit_leash = crate::borg_kit::leash_active(st);
    // borg-reborn-rs (#732): C's take-leash numbers are parameters (`c_leash`).
    let c = crate::borg_flow::c_leash();

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

    /* borg-reborn-rs (#761): short of Phase Doors, one on the floor is exempt from the leash */
    let (phase_short_below, phase_first) = crate::borg_flow::take_phase_rule();
    // borg-reborn-rs (#761, review of #778): the threshold is the held
    // scrolls, not `BI_APHASE` — the trait also counts a Phase Door spell
    // (+1000) and an equipped activation (+1) (`borg_trait.rs`, "Handle
    // PhaseDoor spell carefully"), which are not scrolls and hide a short
    // stock.
    let phase_short = short_of_phase(phase_short_below, carried_phase_scrolls(st));
    let mut phase_pushed: Vec<bool> = Vec::new();

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

        let is_phase = phase_short && take_is_phase(st, take.kidx);

        /* Skip ones that make me wander too far */
        if !is_phase && b_stair != -1 && (st.borg.tr(BI_CLEVEL) < c.take_near_max_level || kit_leash) {
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
        if !is_phase
            && nearness > c.take_nearness
            && (st.borg.tr(BI_CLEVEL) < c.take_max_level || kit_leash)
            && crate::borg_flow_stairs::borg_flow_cost_stair(st, y, x, b_stair) > leash
        {
            continue;
        }

        /* Careful -- Remember it */
        st.flow.temp_push(y, x);
        phase_pushed.push(is_phase);
    }

    /* borg-reborn-rs (#761): short of Phase Doors, only they, when one is in reach */
    if phase_first && phase_pushed.iter().any(|&p| p) && phase_pushed.iter().any(|&p| !p) {
        let n = st.flow.temp_n as usize;
        let kept: Vec<(i32, i32)> = (0..n).filter(|&k| phase_pushed[k]).map(|k| st.flow.temp_at(k)).collect();
        // borg-reborn-rs (#761, review of #778): remembered for the retry in
        // `borg_flow_take`, when the Phase-only flow finds nothing in reach.
        let dropped: Vec<(i32, i32)> = (0..n).filter(|&k| !phase_pushed[k]).map(|k| st.flow.temp_at(k)).collect();
        st.flow.temp_n = 0;
        for (y, x) in kept {
            st.flow.temp_push(y, x);
        }
        *phase_pruned = dropped;
    }

    /* Nothing to take */
    st.flow.temp_n != 0
}

/// borg-reborn-rs (#761): the Borg holds fewer Phase Doors than the switch asks (0 = off).
fn short_of_phase(below: i32, held: i32) -> bool {
    below > 0 && held < below
}

/// borg-reborn-rs (#761, review of #778): the Scrolls of Phase Door in the
/// pack — the count the documented threshold of `flow_take_phase_short_below`
/// asks for ("while the Borg holds fewer Scrolls of Phase Door than this").
/// Like `borg_notice_inventory`'s scan: the pack slots, aware items only.
fn carried_phase_scrolls(st: &BorgState) -> i32 {
    carried_phase_scrolls_in(st, pack_slots(st))
}

/// The counted part of `carried_phase_scrolls`, with the pack end a parameter
/// so a test can build a small pack.
fn carried_phase_scrolls_in(st: &BorgState, pack_end: i32) -> i32 {
    let mut n = 0;
    for i in 0..pack_end {
        let item = &st.items.items[i as usize];
        if item.iqty != 0
            && item.aware
            && u32::from(item.tval) == TV_SCROLL
            && i32::from(item.sval) == st.item_val.sv_scroll_phase_door
        {
            n += i32::from(item.iqty);
        }
    }
    n
}

/// borg-reborn-rs (#761): the object kind is a Scroll of Phase Door.
fn take_is_phase(st: &BorgState, kidx: u32) -> bool {
    let k = engine::kind(kidx as usize);
    i64::from(k.tval) == i64::from(crate::headers::list_tvals_h::TV_SCROLL) && i32::from(k.sval) == st.item_val.sv_scroll_phase_door
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
    use crate::borg_item::blank_item;

    #[test]
    fn short_of_phase_is_off_at_zero_and_below_the_count() {
        assert!(!short_of_phase(0, 0));
        assert!(short_of_phase(2, 0) && short_of_phase(2, 1));
        assert!(!short_of_phase(2, 2) && !short_of_phase(2, 5));
    }

    /// #761 (review of #778): the threshold counts the aware Scrolls of Phase
    /// Door in the pack — not `BI_APHASE`, which also counts a Phase Door
    /// spell (+1000) and an equipped activation (+1), and not a home or
    /// unaware copy.
    #[test]
    fn carried_phase_scrolls_counts_aware_scrolls_in_the_pack_only() {
        let mut st = BorgState::default();
        st.items.items = vec![blank_item(); 8];
        st.item_val.sv_scroll_phase_door = 10;
        let mut scroll = blank_item();
        scroll.tval = TV_SCROLL as u8;
        scroll.sval = 10;
        scroll.aware = true;
        let mut other = scroll;
        other.sval = 11;
        let mut unseen = scroll;
        unseen.aware = false;
        st.items.items[0] = scroll;
        st.items.items[0].iqty = 2;
        st.items.items[1] = other;
        st.items.items[1].iqty = 5;
        st.items.items[2] = unseen;
        st.items.items[2].iqty = 7;
        st.items.items[6] = scroll;
        st.items.items[6].iqty = 9;
        assert_eq!(carried_phase_scrolls_in(&st, 4), 2, "one stack in the pack; the wrong sval, the unseen copy and slot 6 are outside");
        st.items.items[3] = scroll;
        st.items.items[3].iqty = 3;
        assert_eq!(carried_phase_scrolls_in(&st, 4), 5, "stacks sum");
        assert_eq!(carried_phase_scrolls_in(&st, 8), 14, "the pack end moves the count");
        // A Phase Door spell would put 1000 into `BI_APHASE`; the count of
        // scrolls does not read the traits.
        st.borg.trait_ = vec![1000; 1];
        assert_eq!(carried_phase_scrolls_in(&st, 4), 5);
    }

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

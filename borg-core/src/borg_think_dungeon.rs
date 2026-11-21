//! Port of `borg-think-dungeon.c`: decide on an action while in the
//! dungeon.
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
//! `borg_think_dungeon` is the ordered list of things the Borg tries each
//! turn in the dungeon; the order of every step and every early return is
//! C's. The level feelings (`borg_feeling_danger`, `borg_feeling_stuff`)
//! live in `ThinkDungeonState`. C bugs kept are listed in
//! `docs/c-bugs.md` (section 49).

// borg-reborn-rs (#222): one selected-check hook per successful return.
use crate::core_trace::choice;
use crate::borg::BorgTime;
use crate::borg_caution::borg_caution;
use crate::borg_danger::borg_danger;
use crate::borg_escape::{borg_allow_teleport, borg_dimension_door, borg_recall, borg_shadow_shift};
use crate::borg_fight_defend::borg_defend;
use crate::borg_fight_perm::borg_perma_spell;
use crate::borg_flow::borg_flow_old;
use crate::borg_flow_dark::borg_flow_dark;
use crate::borg_flow_glyph::borg_flow_glyph;
use crate::borg_flow_kill::{
    borg_delete_kill, borg_flow_kill, borg_flow_kill_aim, borg_flow_kill_corridor,
    borg_flow_kill_direct, borg_near_monster_type,
};
use crate::borg_flow_misc::{
    borg_flow_recover, borg_flow_shop_entry, borg_flow_spastic, borg_flow_vault,
    borg_flow_vein, borg_twitchy,
};
use crate::borg_flow_stairs::{borg_flow_stair_both, borg_flow_stair_less, borg_flow_stair_more};
use crate::borg_flow_take::{borg_delete_take, borg_flow_take};
use crate::borg_io::{borg_keypress, borg_note, ESCAPE, KC_ENTER};
use crate::borg_item_enchant::borg_enchanting;
use crate::borg_item_use::{
    borg_activate_item, borg_read_scroll, borg_recharging, borg_use_staff, borg_use_things,
};
use crate::borg_item_wear::{
    borg_swap_rings, borg_test_stuff, borg_wear_recharge, borg_wear_rings, borg_wear_stuff,
};
use crate::borg_junk::{
    borg_destroy_floor, borg_drop_hole, borg_drop_junk, borg_drop_slow, borg_dump_quiver,
    borg_remove_stuff,
};
use crate::borg_light::{borg_check_light, borg_light_beam, borg_maintain_light, BorgNeed};
use crate::borg_magic::borg_spell;
use crate::borg_magic_play::borg_play_magic;
use crate::borg_prepared::borg_must_return_to_town;
use crate::borg_recover::borg_recover;
use crate::borg_store_sell::borg_count_sell;
use crate::borg_trip;
use crate::borg_think_dungeon_util::{
    borg_excavate_vault, borg_leave_level, borg_money_scum, borg_think_dungeon_light,
    borg_think_stair_scum,
};
use crate::borg_think_store::borg_choose_shop;
use crate::borg_trait::borg_notice;
use crate::engine;
use crate::borg_headers::borg_h::{
    BORG_DELAY_FACTOR, BORG_MONEY_SCUM_AMOUNT, BORG_PLAYS_RISKY,
};
use crate::borg_headers::borg_magic_h::{PHASE_DOOR, PORTAL, TELEPORT_SELF};
use crate::borg_headers::borg_trait_h::{
    BI_AGLYPH, BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_GOLD, BI_ISBLIND,
    BI_ISCONFUSED, BI_ISCUT, BI_ISPOISONED, BI_KING, BI_LIGHT, BI_MAXCLEVEL, BI_MAXHP, BI_MAXSP, BI_QUIVER_SLOTS, BI_RECALL,
};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_player_flags_h::PF_COMBAT_REGEN;
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::headers::list_tvals_h::TV_DIGGING;
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// borg-think-dungeon.c globals: the current level "feeling".
#[derive(Default)]
pub struct ThinkDungeonState {
    /// `borg_feeling_danger`
    pub feeling_danger: i32,
    /// `borg_feeling_stuff`
    pub feeling_stuff: i32,
    pub(crate) lightless: crate::borg_lightless::Recovery,
}

/// `GOAL_KILL` (borg-flow.h)
const GOAL_KILL: i32 = 1;
/// `GOAL_TAKE` (borg-flow.h)
const GOAL_TAKE: i32 = 2;
/// `GOAL_MISC` (borg-flow.h)
const GOAL_MISC: i32 = 3;
/// `GOAL_DARK` (borg-flow.h)
const GOAL_DARK: i32 = 4;
/// `GOAL_XTRA` (borg-flow.h)
const GOAL_XTRA: i32 = 5;
/// `GOAL_BORE` (borg-flow.h)
const GOAL_BORE: i32 = 6;
/// `GOAL_FLEE` (borg-flow.h)
const GOAL_FLEE: i32 = 7;
/// `GOAL_VAULT` (borg-flow.h)
const GOAL_VAULT: i32 = 8;
/// `GOAL_RECOVER` (borg-flow.h)
const GOAL_RECOVER: i32 = 9;
/// `GOAL_DIGGING` (borg-flow.h)
const GOAL_DIGGING: i32 = 10;

/// `BORG_TIME_MAX` (borg.h: `INT32_MAX`)
const BORG_TIME_MAX: BorgTime = i32::MAX;

/// `TERM_XTRA_DELAY` (ui-term.h)
const TERM_XTRA_DELAY: i32 = 13;

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// A literal key (`'<'`, `'R'`, ...).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `INVEN_WIELD` (`z_info->pack_size`).
fn inven_wield() -> usize {
    usize::from(engine::z_info().pack_size)
}

/// `borg_items[PACK_SLOTS - back].iqty` (`PACK_SLOTS` is
/// `z_info->pack_size - borg.trait[BI_QUIVER_SLOTS]`). A negative index
/// panics (C reads before `borg_items`).
fn pack_slot_iqty(st: &BorgState, back: i32) -> u8 {
    let i = i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS) - back;
    st.items.items[usize::try_from(i).expect("PACK_SLOTS index")].iqty
}

/// `borg_grids[y][x].feat` as a terrain index.
fn feat_at(st: &BorgState, x: i32, y: i32) -> u32 {
    u32::from(st.cave.grid(x, y).feat)
}

/// `OPT(player, birth_force_descend)`
fn force_descend() -> bool {
    engine::player_opt(OPT_birth_force_descend)
}

/// `(char *)NULL == borg_prepared(depth)`. Goes through
/// `borg_prepared::borg_prepared_buf`, so a formatted reason is written into
/// `borg_prepared_buffer` as in C.
#[track_caller]
fn prepared_null(st: &mut BorgState, depth: i32) -> bool {
    crate::borg_prepared::borg_prepared_buf(st, depth).is_none()
}

/// The distance to the nearest up staircase, starting from the best
/// distance `b_j` found so far (-1 = none). (The lunal and munchkin
/// thinkers also scanned the down list and skipped stairs they could not
/// project to; they were removed, borg-rs#60.)
fn nearest_up_stair(st: &BorgState, mut b_j: i32) -> i32 {
    let t = &st.stairs.less;
    for i in 0..usize::try_from(t.num).unwrap_or(0) {
        /* How far is the nearest stairs */
        let j = engine::distance(st.borg.c, loc { x: t.x[i], y: t.y[i] });

        /* skip the far ones */
        if b_j <= j && b_j != -1 {
            continue;
        }

        /* track it */
        b_j = j;
    }
    b_j
}

/// "Start leaving" and "Start fleeing", each with its note.
fn start_leaving_fleeing(st: &mut BorgState, why: &str) {
    /* Start leaving */
    if !st.borg.goal.leaving {
        /* Note */
        borg_note(st, &format!("# Leaving ({why})"));

        /* Start leaving */
        st.borg.goal.leaving = true;
    }

    /* Start fleeing */
    if !st.borg.goal.fleeing {
        /* Note */
        borg_note(st, &format!("# Fleeing ({why})"));

        /* Start fleeing */
        st.borg.goal.fleeing = true;
    }
}

/// `borg_check_light()` when the last door detection is unset or at least
/// 500 turns old ("Search for secret door via spell before spastic").
fn check_light_for_doors(st: &mut BorgState) -> bool {
    (st.borg.time.detect_doors == 0 || borg_timer(st, st.borg.time.detect_doors) >= 500)
        && borg_check_light(st)
}

/// Perform an action in the dungeon under boosted bravery
///
/// This function is a sub-set of the standard dungeon goals, and is
/// only executed when all of the standard dungeon goals fail, because
/// of excessive danger, or because the level is "bizarre".
fn borg_think_dungeon_brave(st: &mut BorgState) -> bool {
    /*** Local stuff ***/
    let p1 = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);

    /* Try a defense maneuver on 100 */
    if st.borg.tr(BI_CDEPTH) == 100 && borg_defend(st, p1) {
        return true;
    }

    /* Attack monsters */
    if crate::borg_fight_attack::borg_attack(st, true) {
        return true;
    }

    /* Cast a light beam to remove fear of an area */
    if borg_light_beam(st, false) {
        return true;
    }

    /*** Flee (or leave) the level ***/

    /* Take stairs down */
    /* Usable stairs */
    if feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_MORE {
        /* Take the stairs */
        borg_note(st, "# Fleeing via stairs.");
        borg_keypress(st, key(b'>'));

        /* Success */
        return true;
    }

    /* Return to Stairs, but not use them */
    if st.borg.goal.less {
        /* Continue fleeing to stair */
        if borg_flow_old(st, GOAL_FLEE) {
            return true;
        }

        /* Try to find some stairs */
        if st.borg.mon.scary
            && st.borg.tr(BI_CDEPTH) == 0
            && borg_flow_stair_both(st, GOAL_FLEE, false)
        {
            return true;
        }

        /* Try to find some stairs up */
        if borg_flow_stair_less(st, GOAL_FLEE, false) {
            borg_note(st, "# Looking for stairs. Goal_less, brave.");
            return true;
        }
    }

    /* Flee the level */
    if st.borg.goal.fleeing || st.borg.goal.leaving || st.borg.mon.scary {
        /* Take the next stairs */
        st.borg.stair_less = st.borg.goal.fleeing;

        if st.borg.ready_morgoth == 0 {
            st.borg.stair_less = true;
        }

        if force_descend() {
            st.borg.stair_less = false;
        }

        if st.borg.stair_less {
            borg_note(st, "# Fleeing and leaving the level. Brave Thinking.");
        }

        /* Go down if fleeing or prepared. */
        st.borg.stair_more = st.borg.goal.fleeing;
        let d = st.borg.tr(BI_CDEPTH) + 1;
        if prepared_null(st, d) {
            st.borg.stair_more = true;
        }

        /* Continue fleeing the level */
        if borg_flow_old(st, GOAL_FLEE) {
            return true;
        }

        /* Try to find some stairs up */
        if st.borg.stair_less && borg_flow_stair_less(st, GOAL_FLEE, false) {
            borg_note(st, "# Looking for stairs. Flee, brave.");
            return true;
        }

        /* Try to find some stairs down */
        if st.borg.stair_more && borg_flow_stair_more(st, GOAL_FLEE, false, true) {
            return true;
        }
    }

    /* Do short looks on special levels */
    if st.borg.status.vault {
        /* Continue flowing towards monsters */
        if borg_flow_old(st, GOAL_KILL) {
            return true;
        }

        /* Find a (viewable) monster */
        if borg_flow_kill(st, true, 35) {
            return true;
        }

        /* Continue flowing towards objects */
        if borg_flow_old(st, GOAL_TAKE) {
            return true;
        }

        /* Find a (viewable) object */
        if borg_flow_take(st, true, 35) {
            return true;
        }
        if borg_flow_vein(st, true, 35) {
            return true;
        }

        /* Continue to dig out a vault */
        if borg_flow_old(st, GOAL_VAULT) {
            return true;
        }

        /* Find a vault to excavate */
        if borg_flow_vault(st, 35) {
            return true;
        }
    }

    /* Continue flowing towards monsters */
    if borg_flow_old(st, GOAL_KILL) {
        return true;
    }

    /* Find a (viewable) monster */
    if borg_flow_kill(st, true, 250) {
        return true;
    }

    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return true;
    }

    /* Find a (viewable) object */
    if borg_flow_take(st, true, 250) {
        return true;
    }
    if borg_flow_vein(st, true, 250) {
        return true;
    }

    /*** Exploration ***/

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_MISC) {
        return true;
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_DARK) {
        return true;
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_XTRA) {
        return true;
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_BORE) {
        return true;
    }

    /*** Explore the dungeon ***/

    /* Explore interesting grids */
    if borg_flow_dark(st, true) {
        return true;
    }

    /* Explore interesting grids */
    if borg_flow_dark(st, false) {
        return true;
    }

    /* Search for secret door via spell before spastic */
    if check_light_for_doors(st) {
        return true;
    }

    /*** Track down old stuff ***/

    /* Chase old objects */
    if borg_flow_take(st, false, 250) {
        return true;
    }
    if borg_flow_vein(st, false, 250) {
        return true;
    }

    /* Chase old monsters */
    if borg_flow_kill(st, false, 250) {
        return true;
    }

    /* Search for secret door via spell before spastic */
    if check_light_for_doors(st) {
        return true;
    }

    /* Attempt to leave the level */
    if borg_leave_level(st, true) {
        return true;
    }

    /* Search for secret doors */
    if borg_flow_spastic(st, true) {
        return true;
    }

    /* Nothing */
    false
}

/// Prevent a "bouncing Borg" bug. Where borg with telepathy
/// will sit in a narrow area bouncing between 2 or 3 places
/// tracking and flowing to a bouncing monster behind a wall.
/// First, reset goals.
/// Second, clear all known monsters/takes
/// Third, Flee the level
fn borg_check_bounciness(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_CDEPTH) != 0 && (300..=303).contains(&st.borg.antibounce_count) {
        /* Clear Goals */
        st.borg.goal.type_0 = 0;
    }
    if st.borg.tr(BI_CDEPTH) != 0 && (500..=503).contains(&st.borg.antibounce_count) {
        /* Forget old objects */
        let mut i = 1;
        while i < i32::from(st.takes.nxt) {
            borg_delete_take(st, i);
            i += 1;
        }

        /* No objects here */
        st.takes.cnt = 0;
        st.takes.nxt = 1;

        /* Forget old monsters */
        let mut i = 1;
        while i < i32::from(st.kills.nxt) {
            borg_delete_kill(st, i);
            i += 1;
        }

        /* No monsters here */
        st.kills.cnt = 0;
        st.kills.nxt = 1;
    }

    if st.borg.tr(BI_CDEPTH) != 0 && st.borg.antibounce_count >= 700 {
        start_leaving_fleeing(st, "bouncing-borg");
    }
    false
}

fn borg_check_breeders(st: &mut BorgState) -> bool {
    // borg-reborn-rs (#722): a level lost to breeders, searched long enough
    // for stairs: Recall.
    if crate::borg_caution::breeder_recall_due(st) && crate::borg_escape::borg_recall(st) {
        borg_note(st, "# Leaving a level lost to breeders (recall): no stairs reached in time.");
        return true;
    }

    /* Close doors on breeder levels */
    if st.borg.mon.breeders >= 3 {
        /* set the flag to close doors */
        st.borg.near.breeder = true;
    }

    /* Caution from breeders */
    // borg-reborn-rs (#503): not without an easy escape: stand and fight.
    if (i32::from(st.borg.mon.breeders) >= (st.borg.tr(BI_CLEVEL) + 2).min(5))
        && (st.borg.tr(BI_RECALL) <= 0 || st.borg.tr(BI_CLEVEL) < 35)
        && !crate::borg_caution::breeder_stand(st)
        // borg-reborn-rs (#722, lead b): not from a one-think pause.
        && !crate::borg_caution::breeder_stand_paused(st)
    {
        /* Ignore monsters from caution */
        if !st.borg.goal.ignoring && borg_timer(st, st.borg.time.level) >= 2500 {
            /* Flee */
            borg_note(st, "# Ignoring breeders (no recall)");

            /* Ignore multipliers */
            st.borg.goal.ignoring = true;
        }

        start_leaving_fleeing(st, "no recall");
    }

    false
}

/// "Boost bravery": raise `borg.avoidance` to `to`, try
/// `borg_think_dungeon_brave`, then reset the avoidance to the current HP.
/// `before` runs after the danger fields are forgotten and before the
/// brave attempt.
fn boost_bravery(st: &mut BorgState, n: i32, to: i32, before: impl FnOnce(&mut BorgState)) -> bool {
    /* Note */
    let from = st.borg.avoidance;
    borg_note(st, &format!("# Boosting bravery ({n}) from {from} to {to}!"));

    /* Ignore some danger */
    st.borg.avoidance = to as i16;

    /* Forget the danger fields */
    st.danger.wipe = true;

    before(st);

    /* Try anything */
    let done = borg_think_dungeon_brave(st);

    /* Reset "avoidance" */
    st.borg.avoidance = st.borg.tr(BI_CURHP) as i16;

    /* Re-calculate danger */
    st.danger.wipe = true;

    /* Done */
    done
}

/// Perform an action in the dungeon
///
/// Return true if a "meaningful" action was performed
/// Otherwise, return false so we will be called again
///
/// Strategy:
///   Make sure we are happy with our "status" (see above)
///   Attack and kill visible monsters, if near enough
///   Open doors, disarm traps, tunnel through rubble
///   Pick up (or tunnel to) gold and useful objects
///   Explore "interesting" grids, to expand the map
///   Explore the dungeon and revisit old grids
///
/// Fleeing:
///   Use word of recall when level is "scary"
///   Flee to stairs when there is a chance of death
///   Avoid "stair bouncing" if at all possible
///
/// Note that the various "flow" actions allow the Borg to flow
/// "through" closed doors, which will be opened when he attempts
/// to pass through them, so we do not have to pursue terrain until
/// all monsters and objects have been dealt with.
///
/// XXX XXX XXX The poor Borg often kills a nasty monster, and
/// then takes a nap to recover from damage, but gets yanked
/// back to town before he can collect his reward.
pub fn borg_think_dungeon(st: &mut BorgState) -> bool {
    let mut b_j = -1;
    let lightless = crate::borg_lightless::update(st);

    /* HACK: Prevent clock wrapping Step 1 */
    let now = st.borg.time.now;
    if (12000..=12025).contains(&now) || (25000..=25025).contains(&now) {
        /* Clear Possible errors */
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);
        borg_keypress(st, ESCAPE);

        /* enter a special routine to handle this behavior.  Messing with
         * the old_level forces him to re-explore this level, and reshop,
         * if in town.
         */
        st.borg.status.old_depth = 126;

        /* Continue on */
        return choice!("clock_wrap_reset");
    }

    /* Prevent clock wrapping Step 2: reset the clock gracefully and keep
     * playing.  This must run before any panic: the borg_think() overflow
     * check remains as a failsafe in case the reset was somehow missed. */
    if st.borg.time.now >= BORG_TIME_MAX - 3000 {
        st.borg.time.now = 1;
        crate::borg_io::borg_warning(st, "*****WARNING***** Borg Clock Overflow");
    }

    /* no longer trying an unknown object */
    st.borg.trying_unknown = false;

    /* if standing on something valueless, destroy it */
    /* this can be done first because it takes no "turns" */
    if !lightless && borg_destroy_floor(st) {
        return choice!("borg_destroy_floor");
    }

    // borg-reborn-rs (#470): no Scroll of Deep Descent is kept.
    if !lightless && crate::borg_junk::borg_discard_deep_descent(st) {
        return choice!("borg_discard_deep_descent");
    }

    /* add a short pause to slow the borg down for viewing */
    let delay = i32::from(engine::player_delay_factor());
    let cfg_delay = st.cfg(BORG_DELAY_FACTOR);
    let msec = (delay * delay).wrapping_add(cfg_delay.wrapping_mul(cfg_delay));
    engine::term_xtra(TERM_XTRA_DELAY, msec);

    /* redraw the screen if we need to */
    if st.borg.status.redraw {
        borg_note(st, "#  Redrawing screen.");
        engine::do_cmd_redraw(st);
        st.borg.status.redraw = false;
    }

    /* Prevent clock overflow */
    if borg_timer(st, st.borg.time.level) >= 10000 {
        start_leaving_fleeing(st, "boredom");
    }

    /* am I fighting a unique or a summoner, or scaryguy? */
    let dist = if st.borg.tr(BI_MAXCLEVEL) < 15 {
        i32::from(engine::z_info().max_sight)
    } else {
        12
    };
    borg_near_monster_type(st, dist);

    /* Allow borg to jump back up to town if needed.  He probably fled town
     * because he saw a scaryguy (BSV, SER, Maggot).  Since he is here on depth
     * 1, do a quick check for items near the stairs that I can pick up before
     * returning to town.
     */
    if !lightless && st.borg.tr(BI_CDEPTH) == 1 && st.borg.goal.fleeing_to_town {
        /* Try to grab a close item while I'm down here */
        // borg-reborn-rs (#549): not while threatened
        // (`caution_stair_scum_threat`): go straight back up instead.
        if !crate::borg_caution::stair_scum_threatened(st) && borg_think_stair_scum(st) {
            return choice!("borg_think_stair_scum");
        }

        start_leaving_fleeing(st, "finish shopping");
    }

    /* check if we need to do something due to bouncing between spots
     * this will set flags to change behavior even if it doesn't force
     * an action */
    if !lightless && borg_check_bounciness(st) {
        return choice!("borg_check_bounciness");
    }

    /* check if we need to do something due to breeders */
    if !lightless && borg_check_breeders(st) {
        return choice!("borg_check_breeders");
    }

    /* Reset avoidance */
    if i32::from(st.borg.avoidance) != st.borg.tr(BI_CURHP) {
        /* Reset "avoidance" */
        st.borg.avoidance = st.borg.tr(BI_CURHP) as i16;

        /* Re-calculate danger */
        st.danger.wipe = true;

        /* Forget goals */
    }

    /* Keep borg on a short leash */
    if st.stairs.less.num != 0
        && (st.borg.tr(BI_MAXHP) < 30 || st.borg.tr(BI_CLEVEL) < 15)
        && st.borg.tr(BI_CDEPTH) >= st.borg.tr(BI_CLEVEL) - 5
    {
        /* Check for an existing "up stairs" */
        b_j = nearest_up_stair(st, b_j);

        let leash = st.borg.tr(BI_CLEVEL).wrapping_mul(3).wrapping_add(14);

        /* Return to the upstair-- too far away? */
        if !st.borg.goal.less && b_j > leash {
            /* Return to Stairs */
            if !st.borg.goal.less {
                /* Note */
                borg_note(st, &format!("# Return to Stair (wandered too far.  Leash: {leash})"));

                /* Start returning */
                st.borg.goal.less = true;
            }
        }
        /* Clear the flag to Return to the upstair-- we are close enough now */
        else if st.borg.goal.less && b_j < 3 {
            /* Note */
            borg_note(st, "# Close enough to Stair.");

            /* Clear the flag */
            st.borg.goal.less = false;
            st.borg.goal.type_0 = 0;
        }
    }

    /* examine equipment and swaps */
    borg_notice(st, true);

    // #760/#759: an attack-on-sight target or a winning fight owns the
    // action before even an out-of-light wait or a depth-return flight.
    if crate::borg_caution::fight_before_flight(st) {
        if st.borg.no_retreat > 0 {
            st.borg.no_retreat -= 1;
        }
        return choice!("fight_before_flight");
    }

    // borg-reborn-rs (#811): recovery owns the decision through handoff.
    if lightless {
        crate::borg_lightless::recover(st);
        return choice!("lightless_recovery");
    }

    /* Keep borg on a suitable level */
    if st.stairs.less.num != 0 && st.borg.tr(BI_CLEVEL) < 10 && !st.borg.goal.less && {
        let d = st.borg.tr(BI_CDEPTH);
        !prepared_null(st, d)
    } {
        /* Note */
        borg_note(st, "# Needing to get back on correct depth");

        /* Start returning */
        st.borg.goal.less = true;

        /* Take stairs */
        if feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_LESS && !force_descend() {
            borg_keypress(st, key(b'<'));
            return choice!("stair_leash");
        }
    }

    /*** crucial goals ***/

    /* require light-- Special handle for being out of a light source.*/
    if borg_think_dungeon_light(st, false) {
        return choice!("borg_think_dungeon_light");
    }

    /* Decrease the amount of time not allowed to retreat */
    if st.borg.no_retreat > 0 {
        st.borg.no_retreat -= 1;
    }

    /* if we twitch a lot, time to leave */
    if st.borg.times_twitch > 20 {
        st.borg.goal.fleeing = true;
    }

    /*** Important goals ***/

    /* Continue flowing towards good anti-summon grid */
    if borg_flow_old(st, GOAL_DIGGING) {
        return choice!("borg_flow_old");
    }

    /* Try not to die */
    if borg_caution(st) {
        return choice!("borg_caution");
    }

    // borg-reborn-rs (#208): an errand steers here, after caution.
    if crate::shell::steer(st) {
        return choice!("errand_steer");
    }

    /*** if returning from dungeon in bad shape...***/
    if st.borg.tr(BI_LIGHT) == 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
        || st.borg.tr(BI_FOOD) == 0
    {
        /* First try to wear something */
        if st.borg.tr(BI_LIGHT) == 0 {
            /* attempt to refuel/swap */
            if borg_maintain_light(st) == BorgNeed::MetNeed {
                return choice!("borg_maintain_light");
            }

            /* wear stuff and see if it glows */
            if borg_wear_stuff(st) {
                return choice!("borg_wear_stuff");
            }
        }

        /* Recover from damage */
        if borg_recover(st) {
            return choice!("borg_recover");
        }

        /* If full of items, we won't be able to buy stuff, crush stuff */
        if pack_slot_iqty(st, 1) != 0 && borg_drop_hole(st, false) {
            return choice!("borg_drop_hole");
        }

        if borg_choose_shop(st) {
            if crate::borg_town_live::execute_outside(st) { return true; }
            /* Try and visit a shop, if so desired */
            let shop = i32::from(st.borg.goal.shop);
            if shop >= 0 && borg_flow_shop_entry(st, shop) {
                return choice!("borg_flow_shop_entry");
            }
        }
    }

    /* if I must go to town without delay */
    if borg_must_return_to_town(st).is_some() && borg_leave_level(st, false) {
        return choice!("borg_leave_level");
    }

    /* Learn useful spells immediately */
    if borg_play_magic(st, false) {
        return choice!("borg_play_magic");
    }

    /* If using a digger, Wear "useful" equipment before fighting monsters */
    if u32::from(st.items.items[inven_wield()].tval) == TV_DIGGING && borg_wear_stuff(st) {
        return choice!("borg_wear_stuff");
    }

    /* If not using anything, Wear "useful" equipment before fighting monsters
     */
    if st.items.items[inven_wield()].tval == 0 && borg_wear_stuff(st) {
        return choice!("borg_wear_stuff");
    }

    /* Dig an anti-summon corridor */
    if borg_flow_kill_corridor(st) {
        return choice!("borg_flow_kill_corridor");
    }

    /* Attack monsters */
    if crate::borg_fight_attack::borg_attack(st, false) {
        return choice!("borg_attack");
    }

    /* Wear things that need to be worn, but try to avoid swap loops */
    /* if (borg_best_stuff()) return true; */
    if borg_wear_stuff(st) {
        return choice!("borg_wear_stuff");
    }
    if borg_swap_rings(st) {
        return choice!("borg_swap_rings");
    }
    if borg_wear_rings(st) {
        return choice!("borg_wear_rings");
    }

    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return choice!("borg_flow_old");
    }

    /* Find a really close object */
    if borg_flow_take(st, true, 5) {
        return choice!("borg_flow_take");
    }

    /* Remove "backwards" rings */
    /* Only do this in Stores to avoid loops     if (borg_swap_rings()) return
     * (true); */

    /* Repair "backwards" rings */
    if borg_wear_rings(st) {
        return choice!("borg_wear_rings");
    }

    /* Remove stuff that is useless or detrimental */
    if borg_remove_stuff(st) {
        return choice!("borg_remove_stuff");
    }
    if borg_dump_quiver(st) {
        return choice!("borg_dump_quiver");
    }

    /* Check the light */
    if borg_check_light(st) {
        return choice!("borg_check_light");
    }

    /* Continue flowing to a safe grid on which I may recover */
    // borg-reborn-rs (#614, review of #616): not while committed to stairs.
    if !crate::borg_recover::holding_for_stairs(st)
        && !crate::borg_recover::holding_for_monster(st)
        && borg_flow_old(st, GOAL_RECOVER) {
        return choice!("borg_flow_old");
    }

    /* Recover from damage */
    if borg_recover(st) {
        return choice!("borg_recover");
    }

    /* Attempt to find a grid which is safe and I can recover on it.  This
     * should work closely with borg_recover. */
    if !crate::borg_recover::holding_for_stairs(st)
        && !crate::borg_recover::holding_for_monster(st)
        && borg_flow_recover(st, 50) {
        return choice!("borg_flow_recover");
    }

    /* Perform "cool" perma spells */
    if borg_perma_spell(st) {
        return choice!("borg_perma_spell");
    }

    /* Try to stick close to stairs if weak */
    if st.borg.tr(BI_CLEVEL) < 10
        && st.borg.tr(BI_MAXSP) != 0
        && st.borg.tr(BI_CURSP) == 0
        && st.borg.no_rest_prep <= 1
        && !st.borg.temp.bless
        && !st.borg.temp.hero
        && !st.borg.temp.berserk
        && !st.borg.temp.fastcast
        && !engine::player_has(PF_COMBAT_REGEN)
    {
        if st.borg.tr(BI_CDEPTH) != 0 {
            /* Check for an existing "up stairs" */
            let mut tmp_i = 0;
            while tmp_i < i32::from(st.stairs.less.num) {
                let x = st.stairs.less.x[tmp_i as usize];
                let y = st.stairs.less.y[tmp_i as usize];
                tmp_i += 1;

                /* Not on a stair */
                if st.borg.c.y != y || st.borg.c.x != x {
                    continue;
                }

                /* I am standing on a stair */

                /* reset the goal_less flag */
                st.borg.goal.less = false;

                /* if not dangerous, wait here */
                if borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false) == 0 {
                    /* rest here a moment */
                    borg_note(st, "# Resting on stair to gain Mana.");
                    borg_keypress(st, key(b','));
                    return choice!("rest_on_upstair");
                }
            }
        } else
        /* in town */
        {
            /* Check for an existing "dn stairs" */
            let mut tmp_i = 0;
            while tmp_i < i32::from(st.stairs.more.num) {
                let x = st.stairs.more.x[tmp_i as usize];
                let y = st.stairs.more.y[tmp_i as usize];
                tmp_i += 1;

                /* Not on a stair */
                if st.borg.c.y != y || st.borg.c.x != x {
                    continue;
                }

                /* I am standing on a stair */

                /* if not dangerous, wait here */
                if borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false) == 0 {
                    /* rest here a moment */
                    borg_note(st, "# Resting on town stair to gain Mana.");
                    borg_keypress(st, key(b','));
                    return choice!("rest_on_town_stair");
                }
            }
        }

        /* In town, standing on stairs, sit tight */
        if borg_flow_old(st, GOAL_FLEE) {
            return choice!("borg_flow_old");
        }

        /* Try to find some stairs up */
        if borg_flow_stair_less(st, GOAL_FLEE, true) {
            borg_note(st, "# Looking for stairs. Stair hugging.");
            return choice!("borg_flow_stair_less");
        }
    }

    /* If in town and have no money, and nothing to sell,
     * then do not stay in town, its too dangerous.
     */
    if st.borg.tr(BI_CDEPTH) == 0
        && st.borg.tr(BI_CLEVEL) < 6
        && st.borg.tr(BI_GOLD) < 10
        && borg_count_sell(st) < 5
    {
        borg_note(st, "# Nothing to sell in town (leaving).");
        st.borg.goal.leaving = true;

        /* Continue fleeing the level */
        if borg_flow_old(st, GOAL_FLEE) {
            return choice!("borg_flow_old");
        }

        /* Try to find some stairs down */
        if borg_flow_stair_more(st, GOAL_FLEE, false, false) {
            return choice!("borg_flow_stair_more");
        }
    }

    /*** Flee the level XXX XXX XXX ***/

    /* Return to Stairs, but not use them */
    if st.borg.goal.less {
        /* Continue fleeing to stair */
        if borg_flow_old(st, GOAL_FLEE) {
            return choice!("borg_flow_old");
        }

        /* Try to find some stairs */
        if st.borg.mon.scary && borg_flow_stair_both(st, GOAL_FLEE, false) {
            return choice!("borg_flow_stair_both");
        }

        /* Try to find some stairs up */
        if borg_flow_stair_less(st, GOAL_FLEE, false) {
            borg_note(st, "# Looking for stairs. Goal_less, Fleeing.");
            return choice!("borg_flow_stair_less");
        }
    }

    /* Flee the level */
    // borg-reborn-rs (#603): flee as usual while Recall counts down with
    // caution_recall_wait_as_usual.
    if st.borg.goal.fleeing && !crate::borg_caution::recall_restricts(st) {
        /* Take the next stairs */
        st.borg.stair_more = true;
        st.borg.stair_less = true;
        if force_descend() {
            st.borg.stair_less = false;
        }

        borg_note(st, "# Fleeing and leaving the level. (Looking for any stair)");

        /* Continue fleeing the level */
        if borg_flow_old(st, GOAL_FLEE) {
            return choice!("borg_flow_old");
        }

        /* Try to find some stairs */
        if st.borg.mon.scary && borg_flow_stair_both(st, GOAL_FLEE, false) {
            return choice!("borg_flow_stair_both");
        }

        /* Try to find some stairs up */
        if borg_flow_stair_less(st, GOAL_FLEE, false) {
            borg_note(st, "# Looking for stairs. Fleeing.");
            return choice!("borg_flow_stair_less");
        }

        /* Try to find some stairs down */
        // borg-reborn-rs (#483): not while rising, unless in real danger.
        let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        if !crate::borg_caution::rising_holds_descent(st, here)
            && borg_flow_stair_more(st, GOAL_FLEE, false, false)
        {
            return choice!("borg_flow_stair_more");
        }
    }

    /* Flee to a safe Morgoth grid if appropriate */
    if st.borg.tr(BI_KING) == 0
        && st.borg.near.morgoth
        && !st.borg.morgoth_position
        && (st.borg.tr(BI_AGLYPH) >= 10
            && (st.borg.tr(BI_ISBLIND) == 0 && st.borg.tr(BI_ISCONFUSED) == 0))
    {
        /* Continue flowing towards good morgoth grid */
        if borg_flow_old(st, GOAL_MISC) {
            return choice!("borg_flow_old");
        }

        /* Attempt to locate a good Glyphed grid */
        if borg_flow_glyph(st) {
            return choice!("borg_flow_glyph");
        }

        /* Have the borg excavate the dungeon with Stone to Mud */
    }

    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return choice!("borg_flow_old");
    }

    /* Find a really close object */
    if borg_flow_take(st, false, 5) {
        return choice!("borg_flow_take");
    }
    if borg_flow_vein(st, true, 5) {
        return choice!("borg_flow_vein");
    }

    /* Continue flowing towards (the hopefully close) monsters */
    if borg_flow_old(st, GOAL_KILL) {
        return choice!("borg_flow_old");
    }

    /* Find a really close monster */
    if borg_flow_kill(st, true, 20) {
        return choice!("borg_flow_kill");
    }

    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return choice!("borg_flow_old");
    }

    /* Find a really close object */
    if borg_flow_take(st, false, 10) {
        return choice!("borg_flow_take");
    }
    if borg_flow_vein(st, false, 10) {
        return choice!("borg_flow_vein");
    }

    /* Continue flowing towards monsters */
    if borg_flow_old(st, GOAL_KILL) {
        return choice!("borg_flow_old");
    }

    /* Continue towards a vault */
    if borg_flow_old(st, GOAL_VAULT) {
        return choice!("borg_flow_old");
    }

    /* Find a viewable monster and line up a shot on him */
    if borg_flow_kill_aim(st, true) {
        return choice!("borg_flow_kill_aim");
    }

    /*** Deal with inventory objects ***/

    /* check for anything that should be inscribed */
    /* if (borg_inscribe_food()) return true; */

    /* Use things */
    if borg_use_things(st) {
        return choice!("borg_use_things");
    }

    /* Identify unknown things */
    if borg_test_stuff(st) {
        return choice!("borg_test_stuff");
    }

    /* Enchant things */
    if borg_enchanting(st) {
        return choice!("borg_enchanting");
    }

    /* Recharge things */
    if borg_recharging(st) {
        return choice!("borg_recharging");
    }

    /* Drop junk */
    if borg_drop_junk(st) {
        return choice!("borg_drop_junk");
    }

    /* Drop items to make space */
    if borg_drop_hole(st, false) {
        return choice!("borg_drop_hole");
    }

    /* Drop items if we are slow */
    if borg_drop_slow(st) {
        return choice!("borg_drop_slow");
    }

    /*** Flow towards objects ***/

    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return choice!("borg_flow_old");
    }

    /* Find a (viewable) object */
    if borg_flow_take(st, true, 250) {
        return choice!("borg_flow_take");
    }
    if borg_flow_vein(st, true, 250) {
        return choice!("borg_flow_vein");
    }

    /*** Leave the level XXX XXX XXX ***/

    /* Leave the level */
    if (st.borg.goal.leaving && !crate::borg_caution::recall_restricts(st) && st.borg.mon.unique == 0)
        || borg_trip::legacy_sale_hold(st)
    {
        if st.borg.ready_morgoth == 0 && !force_descend() {
            borg_note(st, "# Fleeing and leaving the level (Looking for Up Stair).");
            st.borg.stair_less = true;
        }

        /* Only go down if fleeing or prepared. */
        let d = st.borg.tr(BI_CDEPTH) + 1;
        if prepared_null(st, d) {
            st.borg.stair_more = true;
        }

        /* Continue leaving the level */
        if borg_flow_old(st, GOAL_FLEE) {
            return choice!("borg_flow_old");
        }

        /* Try to find some stairs up */
        if st.borg.stair_less && borg_flow_stair_less(st, GOAL_FLEE, false) {
            borg_note(st, "# Looking for stairs. Goal_Leaving.");

            return choice!("borg_flow_stair_less");
        }

        /* Only go up if needing to sell */
        if borg_trip::legacy_sale_hold(st) {
            st.borg.stair_more = false;
        }

        /* Try to find some stairs down */
        if st.borg.stair_more && borg_flow_stair_more(st, GOAL_FLEE, false, false) {
            return choice!("borg_flow_stair_more");
        }
    }

    /* Power dive if I am playing too shallow
     * This is also seen in leave_level().  If
     * this formula is modified here, change it
     * in leave_level too.
     */
    if st.borg.tr(BI_CDEPTH) != 0
        && {
            let d = st.borg.tr(BI_CDEPTH) + 5;
            prepared_null(st, d)
        }
        && !st.borg.stair_less
    {
        /* Take next stairs */
        st.borg.stair_more = true;

        /* Continue leaving the level */
        if borg_flow_old(st, GOAL_BORE) {
            return choice!("borg_flow_old");
        }

        /* No down if needing to sell */
        if borg_trip::legacy_sale_hold(st) {
            st.borg.stair_more = false;
        }

        /* Attempt to use those stairs */
        if st.borg.stair_more && borg_flow_stair_more(st, GOAL_BORE, true, false) {
            /* Leave a note */
            borg_note(st, "# Powerdiving.");
            return choice!("borg_flow_stair_more");
        }
    }

    /*** Exploration ***/

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_MISC) {
        return choice!("borg_flow_old");
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_DARK) {
        return choice!("borg_flow_old");
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_XTRA) {
        return choice!("borg_flow_old");
    }

    /* Continue flowing (see below) */
    if borg_flow_old(st, GOAL_BORE) {
        return choice!("borg_flow_old");
    }

    if borg_flow_old(st, GOAL_VAULT) {
        return choice!("borg_flow_old");
    }

    /*** Explore the dungeon ***/

    // borg-reborn-rs (#754): caution_recall_wait_no_explore.
    if crate::borg_caution::recall_wait_no_explore(st) {
        return choice!("wait_for_recall");
    }

    if st.borg.status.vault {
        /* Chase close monsters */
        if borg_flow_kill(st, false, i32::from(engine::z_info().max_range) + 1) {
            return choice!("borg_flow_kill");
        }

        /* Chase close objects */
        if borg_flow_take(st, false, 35) {
            return choice!("borg_flow_take");
        }
        if borg_flow_vein(st, false, 35) {
            return choice!("borg_flow_vein");
        }

        /* Excavate a vault safely */
        if borg_excavate_vault(st, i32::from(engine::z_info().max_range) - 2) {
            return choice!("borg_excavate_vault");
        }

        /* Find a vault to excavate */
        if borg_flow_vault(st, 35) {
            return choice!("borg_flow_vault");
        }

        /* Explore close interesting grids */
        if borg_flow_dark(st, true) {
            return choice!("borg_flow_dark");
        }
    }

    /* Chase old monsters */
    if borg_flow_kill(st, false, 250) {
        return choice!("borg_flow_kill");
    }

    /* Chase old objects */
    if borg_flow_take(st, false, 250) {
        return choice!("borg_flow_take");
    }
    if borg_flow_vein(st, false, 250) {
        return choice!("borg_flow_vein");
    }

    /* Explore interesting grids */
    if borg_flow_dark(st, true) {
        return choice!("borg_flow_dark");
    }

    /* Leave the level (if needed) */
    if st.borg.tr(BI_GOLD) < st.cfg(BORG_MONEY_SCUM_AMOUNT)
        && st.cfg(BORG_MONEY_SCUM_AMOUNT) != 0
        && st.borg.tr(BI_CDEPTH) == 0
        && st.borg.tr(BI_LIGHT) != 0
    {
        /* Stay in town and scum for money after shopping */
    } else if borg_leave_level(st, false) {
        return choice!("borg_leave_level");
    }

    /* Explore interesting grids */
    if borg_flow_dark(st, false) {
        return choice!("borg_flow_dark");
    }

    /*** Deal with shops ***/

    /* Visit the shops */
    if borg_choose_shop(st) {
        if crate::borg_town_live::execute_outside(st) { return true; }
        /* Try and visit a shop, if so desired */
        let shop = i32::from(st.borg.goal.shop);
        if shop >= 0 && borg_flow_shop_entry(st, shop) {
            return choice!("borg_flow_shop_entry");
        }
    }

    /*** Leave the Level ***/

    /* Study/Test boring spells/prayers */
    if !st.borg.goal.fleeing && borg_play_magic(st, true) {
        return choice!("borg_play_magic");
    }

    /* Search for secret door via spell before spastic */
    if check_light_for_doors(st) {
        return choice!("check_light_for_doors");
    }

    /* Search for secret doors */
    if borg_flow_spastic(st, false) {
        return choice!("borg_flow_spastic");
    }

    /* Flow directly to a monster if not able to be spastic */
    if borg_flow_kill_direct(st, false) {
        return choice!("borg_flow_kill_direct");
    }

    /* Recharge items before leaving the level */
    if borg_wear_recharge(st) {
        return choice!("borg_wear_recharge");
    }

    /* Leave the level (if possible) */
    if st.borg.tr(BI_GOLD) < st.cfg(BORG_MONEY_SCUM_AMOUNT)
        && st.cfg(BORG_MONEY_SCUM_AMOUNT) != 0
        && st.borg.tr(BI_CDEPTH) == 0
        && st.borg.tr(BI_LIGHT) != 0
        && st.cfg(BORG_PLAYS_RISKY) == 0
    /* risky borgs are in a hurry */
    {
        /* Stay in town, scum for money now that shopping is done. */
        if borg_money_scum(st) {
            return choice!("borg_money_scum");
        }
    } else if borg_leave_level(st, true) {
        return choice!("borg_leave_level");
    }

    /* Search for secret door via spell before spastic */
    if check_light_for_doors(st) {
        return choice!("check_light_for_doors");
    }

    /* Search for secret doors */
    if borg_flow_spastic(st, true) {
        return choice!("borg_flow_spastic");
    }

    /* Flow directly to a monster if not able to be spastic */
    if borg_flow_kill_direct(st, false) {
        return choice!("borg_flow_kill_direct");
    }

    /*** Wait for recall ***/

    /* Wait for recall, unless in danger */
    if st.borg.goal.recalling != 0
        && borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false) <= 0
    {
        // borg-reborn-rs (#567): a monster was near a moment ago: one turn.
        if crate::borg_caution::recall_rest_held(st) {
            return choice!("wait_for_recall");
        }

        /* Take note */
        borg_note(st, "# Waiting for Recall...");

        /* Rest until done */
        borg_keypress(st, key(b'R'));
        borg_keypress(st, key(b'9'));
        borg_keypress(st, KC_ENTER);

        /* Done */
        return choice!("wait_for_recall");
    }

    /*** Nothing to do ***/

    /* Twitching in town can be fatal.  Really he should not become twitchy
     * but sometimes he cant recall to the dungeon and that may induce the
     * twitchy behavior.  So we reset the level if this happens.  That will
     * force him to go shopping all over again.
     */
    if (st.borg.tr(BI_CDEPTH) == 0 && borg_timer(st, st.borg.time.level) > 800)
        || borg_timer(st, st.borg.time.level) > 28000
    {
        st.borg.status.old_depth = 126;
    }

    /* Set a flag that the borg is  not allowed to retreat for 5 rounds */
    st.borg.no_retreat = 5;

    // borg-reborn-rs (#556): stuck while remembered breeders block the
    // flows (caution_breeder_avoid): the marks are suspended for a few turns,
    // so the stuck think's flows below may cross them (fight its way out).
    crate::borg_caution::suspend_breeder_marks_when_stuck(st);

    /* Boost slightly */
    let to = st.borg.tr(BI_CURHP).wrapping_mul(2);
    if i32::from(st.borg.avoidance) < to && boost_bravery(st, 1, to, |_| {}) {
        return choice!("boost_bravery_1");
    }

    /* try phase before boosting bravery further and acting goofy */
    st.borg.times_twitch = st.borg.times_twitch.wrapping_add(1);

    /* Phase to get out of being twitchy up to 3 times per level. */
    if st.borg.times_twitch < 3 {
        borg_note(st, "# Considering Phase (twitchy)");

        /* Phase */
        /* borg-reborn-rs (#584): through the one Phase Door decision; no
         * check, as before. */
        if borg_allow_teleport(st)
            && crate::borg_escape::phase_door(
                st,
                crate::borg_escape::PhaseSite::TwitchyThink,
                crate::borg_escape::PhaseCheck::None,
                None,
                |st| {
                    borg_spell(st, PHASE_DOOR)
                        || {
                            let act = st.act.tele_phase;
                            borg_activate_item(st, act)
                        }
                        || {
                            let sv = st.item_val.sv_scroll_phase_door;
                            borg_read_scroll(st, sv)
                        }
                        || borg_dimension_door(st, 90)
                        || borg_spell(st, TELEPORT_SELF)
                        || borg_spell(st, PORTAL)
                        || borg_shadow_shift(st, 90)
                },
            )
        {
            /* Success */
            return choice!("twitch_phase");
        }
    }

    /* Set a flag that the borg is not allowed */
    /*  to retreat for 10 rounds */
    st.borg.no_retreat = 10;

    /* Boost some more */
    let to = st.borg.tr(BI_MAXHP).wrapping_mul(4);
    if i32::from(st.borg.avoidance) < to && boost_bravery(st, 2, to, |_| {}) {
        return choice!("boost_bravery_2");
    }

    /* Boost a lot */
    if st.borg.avoidance < 30000
        && boost_bravery(st, 3, 30000, |st| {
            /* Reset multiple factors to jumpstart the borg */
            st.borg.mon.unique = 0;
            crate::borg_caution::forget_scary(&mut st.borg.mon); // borg-reborn-rs (#530)

            /* reset our breeder flag */
            st.borg.near.breeder = false;

            /* Forget goals */
            st.borg.goal.type_0 = 0;

            /* Cannot rise past town */
            if st.borg.tr(BI_CDEPTH) == 0 {
                st.borg.goal.rising = false;
            }

            /* Assume not ignoring monsters */
            st.borg.goal.ignoring = false;

            /* No known stairs */
            // borg-reborn-rs (#544, caution_jumpstart_keep_stairs since #548): unless kept; forgetting them left the
            // Borg in a dead-end worm room with no way to the down staircase
            // it had arrived on (priest-1055640847).
            if !crate::borg_caution::jumpstart_keeps_stairs() {
                st.stairs.less.num = 0;
                st.stairs.more.num = 0;
            }

            /* No known glyph */
            st.glyph.track.num = 0;

            /* No known steps */
            st.flow.track_step.num = 0;

            /* No known doors */
            st.flow.track_door.num = 0;

            /* No known doors */
            st.flow.track_closed.num = 0;

            /* No mineral veins */
            st.flow_misc.track_vein.num = 0;

            /* No objects here */
            st.takes.cnt = 0;
            st.takes.nxt = 1;
        })
    {
        return choice!("boost_bravery_3");
    }

    /* try teleporting before acting goofy */
    st.borg.times_twitch = st.borg.times_twitch.wrapping_add(1);

    /* Teleport to get out of being twitchy up to 5 times per level. */
    if st.borg.times_twitch < 5 {
        /* Teleport */
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, 90)
                || borg_spell(st, TELEPORT_SELF)
                || borg_spell(st, PORTAL)
                || borg_shadow_shift(st, 90)
                || {
                    let sv = st.item_val.sv_staff_teleportation;
                    borg_use_staff(st, sv)
                }
                || {
                    let sv = st.item_val.sv_scroll_teleport;
                    borg_read_scroll(st, sv)
                }
                || {
                    let sv = st.item_val.sv_scroll_teleport_level;
                    borg_read_scroll(st, sv)
                }
                || {
                    let act = st.act.tele_level;
                    borg_activate_item(st, act)
                })
        {
            /* Success */
            borg_note(st, "# Teleport (twitchy)");
            return choice!("twitch_teleport");
        }
    }

    /* Recall to town */
    if st.borg.tr(BI_CDEPTH) != 0 && borg_recall(st) {
        /* Note */
        borg_note(st, "# Recalling (twitchy)");

        /* Success */
        return choice!("borg_recall");
    }

    /* Reset multiple factors to jumpstart the borg */
    st.borg.mon.unique = 0;
    crate::borg_caution::forget_scary(&mut st.borg.mon); // borg-reborn-rs (#530)

    /* reset our breeder flag */
    st.borg.near.breeder = false;

    /* No objects here */
    st.takes.cnt = 0;
    st.takes.nxt = 1;

    /* No monsters here */
    st.kills.cnt = 0;
    st.kills.nxt = 1;

    /* Attempt to dig to the center of the dungeon */
    if borg_flow_kill_direct(st, true) {
        return choice!("borg_flow_kill_direct");
    }

    /* Twitch around */
    if borg_twitchy(st) {
        return choice!("borg_twitchy");
    }

    /* Oops */
    false
}

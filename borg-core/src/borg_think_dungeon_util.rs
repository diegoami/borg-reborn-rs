//! Port of `borg-think-dungeon-util.c`: things to think about while in the
//! dungeon that don't fit elsewhere (money scumming, surviving without
//! light, stair scumming, leaving the level, excavating a vault).
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
//! The module has no state of its own (`borg_stuff_feeling` is a constant
//! table). `borg_feeling_stuff` is `st.think_dungeon.feeling_stuff`
//! (borg_think_dungeon).

use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg::BorgTime;
use crate::borg_escape::borg_recall;
use crate::borg_flow::{borg_flow_old, AUTO_TEMP_MAX};
use crate::borg_flow_misc::borg_flow_light;
use crate::borg_flow_stairs::{
    borg_flow_stair_both, borg_flow_stair_less, borg_flow_stair_more, borg_prep_leave_level_spells,
};
use crate::borg_flow_take::borg_flow_take_scum;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, ESCAPE, KC_ENTER};
use crate::borg_item_use::{
    borg_activate_item, borg_activate_ring, borg_read_scroll, borg_use_staff, borg_use_things,
    borg_zap_rod,
};
use crate::borg_item_wear::borg_wear_stuff;
use crate::borg_light::{borg_light_beam, borg_maintain_light, light_running_out, BorgNeed};
use crate::borg_magic::{borg_spell, borg_spell_legal, borg_spell_okay, borg_spell_okay_fail};
use crate::borg_messages_react::borg_react;
use crate::borg_prepared::{borg_must_return_to_town, borg_restock};
use crate::borg_projection::{borg_los, borg_target};
use crate::borg_store_sell::borg_count_sell;
use crate::borg_prepared::PreparedC;
use crate::engine;
use crate::borg_headers::borg_h::{
    BORG_MONEY_SCUM_AMOUNT, BORG_NO_DEEPER, BORG_PLAYS_RISKY, BORG_WORSHIPS_GOLD,
};
use crate::borg_headers::borg_magic_h::{
    CALL_LIGHT, CREATE_DARKNESS, HERBAL_CURING, LIGHT_ROOM, REMOVE_HUNGER, SHATTER_STONE,
    TURN_STONE_TO_MUD,
};
use crate::borg_headers::borg_trait_h::{
    BI_AEZHEAL, BI_ALIFE, BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_GOLD,
    BI_ISFIXEXP, BI_ISFIXLEV, BI_ISHUNGRY, BI_ISWEAK, BI_KING, BI_LIGHT, BI_MAXCLEVEL,
    BI_MAXDEPTH, BI_MAXSP, BI_PREP_BIG_FIGHT, BI_QUIVER_SLOTS, BI_RECALL, BI_REG,
    BI_SAURON_DEAD,
};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_FLOOR, FEAT_GRANITE, FEAT_LESS, FEAT_MAGMA, FEAT_MAGMA_K, FEAT_MORE,
    FEAT_NONE, FEAT_OPEN, FEAT_PERM, FEAT_QUARTZ, FEAT_QUARTZ_K, FEAT_RUBBLE,
};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// `GOAL_TAKE` (borg-flow.h)
const GOAL_TAKE: i32 = 2;
/// `GOAL_BORE` (borg-flow.h)
const GOAL_BORE: i32 = 6;
/// `GOAL_FLEE` (borg-flow.h)
const GOAL_FLEE: i32 = 7;

/// `CLASS_NECROMANCER` (list-classes.h order).
const CLASS_NECROMANCER: i32 = 4;

/// Grid info flags (borg-cave.h).
const BORG_MARK: u16 = 0x01;
const BORG_GLOW: u16 = 0x02;
const BORG_DARK: u16 = 0x04;
const BORG_IGNORE_MAP: u16 = 0x100;

/// `borg_stuff_feeling`: importance of the various "level feelings".
/// These values are arbitrary estimates. Try to explore the level for at
/// least this many turns.
const BORG_STUFF_FEELING: [i32; 12] = [
    50000, /* 0 is no feeling yet given, stick around to get one */
    8000, 8000, 8000, 8000, 5000, 5000, 100, 100, 100, 100, 0,
];

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// A literal key (`'R'`, `'5'`, ...).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `INVEN_LIGHT` (`z_info->pack_size + 5`).
fn inven_light() -> usize {
    usize::from(engine::z_info().pack_size) + 5
}

/// `PACK_SLOTS` (`z_info->pack_size - borg.trait[BI_QUIVER_SLOTS]`).
fn pack_slots(st: &BorgState) -> i32 {
    i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS)
}

/// `borg_grids[y][x].feat` as a terrain index.
fn feat_at(st: &BorgState, x: i32, y: i32) -> u32 {
    u32::from(st.cave.grid(x, y).feat)
}

/// `(ddx_ddd[i], ddy_ddd[i])`
fn dd(i: usize) -> (i32, i32) {
    (i32::from(engine::ddx_ddd(i)), i32::from(engine::ddy_ddd(i)))
}

/// money Scumming is a type of town scumming for money
pub fn borg_money_scum(st: &mut BorgState) -> bool {
    let mut divisor: i64 = 2;

    /* Just a quick check to make sure we are supposed to do this */
    if st.cfg(BORG_MONEY_SCUM_AMOUNT) == 0 {
        return false;
    }

    /* Take note */
    let need = st.cfg(BORG_MONEY_SCUM_AMOUNT).wrapping_sub(st.borg.tr(BI_GOLD));
    borg_note(st, &format!("# Waiting for towns people to breed.  I need {need}..."));

    /* I'm not in a store */
    st.borg.in_shop = false;

    /* Rest for 9 months */
    let keys: &[keycode_t] = if st.borg.tr(BI_CLEVEL) >= 35 {
        &[ESCAPE, key(b'R'), key(b'5'), key(b'0'), key(b'0'), KC_ENTER]
    } else if st.borg.tr(BI_CLEVEL) >= 15 {
        &[ESCAPE, key(b'R'), key(b'7'), key(b'5'), KC_ENTER]
    } else {
        /* Low level, dont want to get mobbed */
        &[ESCAPE, key(b'R'), key(b'2'), key(b'5'), KC_ENTER]
    };
    for &k in keys {
        borg_keypress(st, k);
    }

    /* Don't rest too long at night.  We tend to crash the game if too many
     * townsfolk are on the level
     */
    /* Night or day up in town */
    let day = 10 * i64::from(engine::z_info().day_length);
    if i64::from(engine::game_turn()) % day < day / 2 {
        divisor = 5;
    }

    /* sometimes twitch in order to move around some */
    if i64::from(st.borg.time.now) % divisor != 0 {
        let mut dirs = [0i32; 8];
        let mut count = 0usize;

        /* Enumerate candidates: there may be no adjacent floor at all. */
        for i in 0..8 {
            let (dx, dy) = dd(i);
            let (nx, ny) = (st.borg.c.x + dx, st.borg.c.y + dy);
            if engine::square_in_bounds_fully(nx, ny) && feat_at(st, nx, ny) == FEAT_FLOOR {
                dirs[count] = i32::from(engine::ddd(i));
                count += 1;
            }
        }

        /* With no destination, keep the rest command already queued. */
        if count != 0 {
            let dir = dirs[engine::randint0(count as u32) as usize];
            st.borg.goal.g = loc {
                x: st.borg.c.x + i32::from(engine::ddx(dir as usize)),
                y: st.borg.c.y + i32::from(engine::ddy(dir as usize)),
            };
            borg_keypress(st, ESCAPE);
            borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
        }
    }

    /* reset the clocks */
    st.borg.antibounce_count = 1;
    st.borg.time.level = 1;

    /* Done */
    true
}

/// Attempt a series of maneuvers to stay alive when you run out of light
pub fn borg_think_dungeon_light(st: &mut BorgState, lightless: bool) -> bool {
    /* Consume needed things */
    if !lightless && st.borg.tr(BI_ISHUNGRY) != 0 && borg_use_things(st) {
        return true;
    }

    let light = &st.items.items[inven_light()];
    let dying = light_running_out(light);
    if !lightless && !((st.borg.tr(BI_LIGHT) == 0 || dying) && st.borg.tr(BI_CDEPTH) >= 1) {
        /* Nothing to do */
        return false;
    }

    /* I am recalling, sit here till it engages. */
    if st.borg.goal.recalling != 0 {
        // borg-reborn-rs (#567, review of #640): the third Recall rest.
        if crate::borg_caution::recall_rest_held(st) {
            return true;
        }
        /* just wait */
        borg_keypress(st, key(b'R'));
        borg_keypress(st, key(b'9'));
        borg_keypress(st, KC_ENTER);
        return true;
    }

    if !lightless {
        /* wear stuff and see if it glows */
        if borg_wear_stuff(st) {
            return true;
        }

        /* attempt to refuel/swap */
        let need = borg_maintain_light(st);
        if need == BorgNeed::MetNeed {
            return true;
        }
        if need == BorgNeed::NoNeed {
            return false;
        }

        /* Can I recall out with a rod */
        if st.borg.goal.recalling == 0 {
            let sv = st.item_val.sv_rod_recall;
            if borg_zap_rod(st, sv) {
                return true;
            }
        }

        /* Can I recall out with a spell */
        if st.borg.goal.recalling == 0 && borg_recall(st) {
            return true;
        }

        /* Test for stairs */
        if !engine::player_opt(OPT_birth_force_descend)
            && feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_LESS
        {
            /* Usable stairs */
            borg_note(st, "# Testing for stairs .");
            borg_keypress(st, key(b'<'));

            /* Done */
            return true;
        }

    }

    /* If on a glowing grid, got some food, and low mana, then rest here */
    if (st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) && st.borg.tr(BI_MAXSP) > 0)
        && (st.cave.grid(st.borg.c.x, st.borg.c.y).info & BORG_GLOW != 0
            || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER)
        && st.borg.tr(BI_ISWEAK) == 0
        && (borg_spell_legal(st, HERBAL_CURING)
            || borg_spell_legal(st, REMOVE_HUNGER)
            || st.borg.tr(BI_FOOD) >= st.borg.tr(BI_CDEPTH))
        && (borg_spell_legal(st, CALL_LIGHT) || borg_spell_legal(st, LIGHT_ROOM))
        && (!borg_spell_okay(st, CALL_LIGHT) && !borg_spell_okay(st, LIGHT_ROOM))
    {
        let mut not_safe = false;

        /* Scan grids adjacent to me */
        for ii in 0..8 {
            let (dx, dy) = dd(ii);
            let (x, y) = (st.borg.c.x + dx, st.borg.c.y + dy);

            /* Bounds check */
            if !engine::square_in_bounds_fully(x, y) {
                continue;
            }

            /* Check for adjacent Monster */
            if st.cave.grid(x, y).kill != 0 {
                not_safe = true;
            }
        }

        let (cx, cy) = (st.borg.c.x, st.borg.c.y);
        let curhp10 = st.borg.tr(BI_CURHP) / 10;

        /* Be concerned about the Regional Fear. */
        if i32::from(st.danger.fear_region[(cy / 11) as usize][(cx / 11) as usize]) > curhp10 {
            not_safe = true;
        }

        /* Be concerned about the Monster Fear. */
        if i32::from(st.danger.fear_monsters[cy as usize][cx as usize]) > curhp10 {
            not_safe = true;
        }

        /* rest here to gain some mana */
        if !not_safe {
            borg_note(st, "# Resting on this Glowing Grid to gain mana.");
            borg_keypress(st, key(b'R'));
            borg_keypress(st, key(b'*'));
            borg_keypress(st, KC_ENTER);
            return true;
        }
    }

    /* If I have the capacity to Call Light, then do so if adjacent to a
     * dark grid. We can illuminate the entire dungeon, looking for stairs
     * but not if we just did so.
     */
    if st.borg.time.call_light == 0 || borg_timer(st, st.borg.time.call_light) > 7 {
        /* Scan the grids adjacent to me and my own grid (an open
         * door underfoot is handled below) */
        for ii in 0..=8 {
            let (dx, dy) = dd(ii);
            let (x, y) = (st.borg.c.x + dx, st.borg.c.y + dy);

            /* Bounds check */
            if !engine::square_in_bounds_fully(x, y) {
                continue;
            }

            /* Access the grid */
            let ag = *st.cave.grid(x, y);
            let feat = u32::from(ag.feat);

            /* skip the Wall grids */
            if (FEAT_RUBBLE..=FEAT_PERM).contains(&feat) {
                continue;
            }

            /* Problem with casting Call Light on Open Doors */
            if (feat == FEAT_OPEN || feat == FEAT_BROKEN)
                && (y == st.borg.c.y && x == st.borg.c.x)
            {
                /* Cheat the grid info to see if the door is lit */
                if engine::square_isglow(st.borg.c.x, st.borg.c.y) {
                    st.cave.grid_mut(x, y).info |= BORG_GLOW;
                }
                continue;
            }

            /* Look for a dark one */
            if st.borg.tr(BI_CLASS) != CLASS_NECROMANCER {
                if ag.info & BORG_DARK != 0 /* Known to be dark */
                    || feat == FEAT_NONE /* Nothing known about feature */
                    || ag.info & BORG_MARK == 0 /* Nothing known about info */
                    || ag.info & BORG_GLOW == 0
                /* not glowing */
                {
                    /* Attempt to Call Light */
                    let (act_illumination, act_light) = (st.act.illumination, st.act.light);
                    let sv_rod = st.item_val.sv_rod_illumination;
                    let sv_staff = st.item_val.sv_staff_light;
                    if borg_activate_item(st, act_illumination)
                        || borg_activate_item(st, act_light)
                        || borg_zap_rod(st, sv_rod)
                        || borg_use_staff(st, sv_staff)
                        // borg-reborn-rs (#1273): keep the old chain for recorded replays.
                        || (!crate::borg_light::light_scroll_idle_spend()
                            && borg_read_scroll(st, st.item_val.sv_scroll_light))
                        || borg_spell(st, CALL_LIGHT)
                        || borg_spell(st, LIGHT_ROOM)
                    {
                        borg_note(st, "# Illuminating the region while dark.");
                        borg_react(st, b"SELF:lite", b"SELF:lite");
                        st.borg.time.call_light = st.borg.time.now;

                        return true;
                    }

                    /* Attempt to use Light Beam requiring a direction. */
                    if borg_light_beam(st, false) {
                        return true;
                    }
                }
            } else if (ag.info & BORG_DARK == 0 /* Not dark */
                || feat == FEAT_NONE /* Nothing known about feature */
                || ag.info & BORG_MARK == 0 /* Nothing known about info */
                || ag.info & BORG_GLOW != 0/* glowing */)
                && borg_spell(st, CREATE_DARKNESS)
            {
                borg_note(st, "# Darkening the region that is lit.");
                st.borg.time.call_light = st.borg.time.now;
                return true;
            }
        }
    }

    if lightless {
        return false;
    }

    /* don't flee to the stairs if already fleeing */
    if borg_flow_old(st, GOAL_FLEE) {
        return true;
    }

    /* Try to flow to upstairs if on level one */
    if borg_flow_stair_less(st, GOAL_FLEE, false) && !engine::player_opt(OPT_birth_force_descend)
    {
        /* Usable stairs */
        if feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_LESS {
            /* Take the stairs */
            borg_note(st, "# Taking up Stairs (low Light).");
            borg_keypress(st, key(b'<'));
            return true;
        }
        borg_note(st, "# Flowing to stairs (low Light).");
        return true;
    }

    /* Try to flow to a lite */
    if st.borg.tr(BI_RECALL) != 0 && borg_flow_light(st, GOAL_FLEE) {
        return true;
    }

    /* Nothing to do */
    false
}

/// `track_more` or `track_less` (borg-flow-stairs.c).
#[derive(Clone, Copy)]
enum Stairs {
    More,
    Less,
}

fn stairs(st: &mut BorgState, which: Stairs) -> &mut crate::borg_flow_stairs::StairTrack {
    match which {
        Stairs::More => &mut st.stairs.more,
        Stairs::Less => &mut st.stairs.less,
    }
}

/// "Track the newly discovered stairs": append the Borg's grid as entry
/// `i` if `i` is the end of the list and there is room.
fn track_stair(st: &mut BorgState, which: Stairs, i: i32) {
    let c = st.borg.c;
    let t = stairs(st, which);
    if i == i32::from(t.num) && i < i32::from(t.size) {
        t.x[i as usize] = c.x;
        t.y[i as usize] = c.y;
        t.num += 1;
    }
}

/// The distance to the nearest stair of the list, starting from the best
/// distance `b_j` found so far (-1 = none).
fn nearest_stair(st: &BorgState, which: Stairs, mut b_j: i32) -> i32 {
    let t = match which {
        Stairs::More => &st.stairs.more,
        Stairs::Less => &st.stairs.less,
    };
    for i in 0..usize::try_from(t.num).unwrap_or(0) {
        let (x, y) = (t.x[i], t.y[i]);

        /* How far is the nearest stairs */
        let j = engine::distance(st.borg.c, loc { x, y });

        /* skip the far ones */
        if b_j <= j && b_j != -1 {
            continue;
        }

        /* track it */
        b_j = j;
    }
    b_j
}

/// This is an exploitation function.  The borg will stair scum
/// in the dungeon to grab items close to the stair.
pub fn borg_think_stair_scum(st: &mut BorgState) -> bool {
    let mut b_j = -1;

    /* `ag`: the Borg's grid (the Borg does not move during the call) */
    let (ax, ay) = (st.borg.c.x, st.borg.c.y);

    let feat = u32::from(engine::square_feat(st.borg.c.x, st.borg.c.y));

    /* examine equipment and swaps */
    crate::borg_trait::borg_notice(st, true);

    /* No scumming mode if starving or in town */
    if st.borg.tr(BI_CDEPTH) == 0 || st.borg.tr(BI_ISWEAK) != 0 {
        borg_note(st, "# Leaving Scumming Mode. (Town or Weak)");
        return false;
    }

    /* No scumming if inventory is full.  Require one empty slot */
    if st.items.items[(pack_slots(st) - 1) as usize].iqty != 0 {
        return false;
    }

    /* if borg is just starting on this level, he may not
     * know that a stair is under him.  Cheat to see if one is
     * there
     */
    if feat == FEAT_MORE && feat_at(st, ax, ay) != FEAT_MORE {
        /* Check for an existing "down stairs" */
        let mut i = 0;
        while i < i32::from(st.stairs.more.num) {
            /* We already knew about that one */
            if st.stairs.more.x[i as usize] == st.borg.c.x
                && st.stairs.more.y[i as usize] == st.borg.c.y
            {
                break;
            }
            i += 1;
        }

        /* Track the newly discovered "down stairs" */
        track_stair(st, Stairs::More, i);

        /* tell the array */
        st.cave.grid_mut(ax, ay).feat = FEAT_MORE as u8;
    }

    if feat == FEAT_LESS && feat_at(st, ax, ay) != FEAT_LESS {
        /* Check for an existing "up stairs" */
        let mut i = 0;
        while i < i32::from(st.stairs.less.num) {
            /* We already knew about this one */
            /* (break, like the down-stair loop above: continuing past a
             * match re-adds the known staircase as a duplicate) */
            if st.stairs.less.x[i as usize] == st.borg.c.x
                && st.stairs.less.y[i as usize] == st.borg.c.y
            {
                break;
            }
            i += 1;
        }

        /* Track the newly discovered "up stairs" */
        track_stair(st, Stairs::Less, i);

        /* Tell the array */
        st.cave.grid_mut(ax, ay).feat = FEAT_LESS as u8;
    }

    /* First deal with staying alive */

    /* Require light */
    let need = borg_maintain_light(st);
    if need == BorgNeed::MetNeed {
        return true;
    } else if need == BorgNeed::UnmetNeed {
        borg_note(st, "# Scum. (need fuel)");
    }

    /* Track down some interesting gear */
    /* XXX Should we allow him great flexibility in retrieving loot? (not
     * always safe?)*/
    /* Continue flowing towards objects */
    if borg_flow_old(st, GOAL_TAKE) {
        return true;
    }

    /* Find a (viewable) object */
    if borg_flow_take_scum(st, true, 6) {
        return true;
    }

    /*leave level right away. */
    borg_note(st, "# Fleeing level. Scumming Mode");
    st.borg.goal.fleeing = true;

    /* Scumming Mode - Going down */
    if st.stairs.more.num != 0
        && (feat_at(st, ax, ay) == FEAT_MORE
            || feat_at(st, ax, ay) == FEAT_LESS
            || st.borg.tr(BI_CDEPTH) < 30)
    {
        if st.stairs.more.num >= 2 {
            borg_note(st, "# Scumming Mode: I know of a down stair.");
        }

        /* Check for an existing "down stairs" */
        b_j = nearest_stair(st, Stairs::More, b_j);

        /* if the downstair is close and path is safe, continue on */
        if b_j < 8 || feat_at(st, ax, ay) == FEAT_MORE || st.borg.tr(BI_CDEPTH) < 30 {
            /* Note */
            borg_note(st, "# Scumming Mode.  Power Diving. ");

            /* Continue leaving the level */
            if borg_flow_old(st, GOAL_FLEE) {
                return true;
            }

            /* Flow to DownStair */
            if borg_flow_stair_more(st, GOAL_FLEE, false, false) {
                return true;
            }

            /* if standing on a stair */
            if feat_at(st, ax, ay) == FEAT_MORE {
                /* Take the DownStair */
                borg_keypress(st, key(b'>'));

                return true;
            }
        }
    }

    /* Scumming Mode - Going up */
    if st.stairs.less.num != 0
        && st.borg.tr(BI_CDEPTH) != 1
        && (feat_at(st, ax, ay) == FEAT_MORE || feat_at(st, ax, ay) == FEAT_LESS)
    {
        /* `tmp_ag`: the Borg's grid again */
        let (tx, ty) = (st.borg.c.x, st.borg.c.y);

        /* Check for an existing "up stairs" (`b_j` carries over from the
         * down stairs, C bug 47) */
        b_j = nearest_stair(st, Stairs::Less, b_j);

        /* if the upstair is close and safe path, continue */
        if b_j < 8 || feat_at(st, tx, ty) == FEAT_LESS {
            /* Note */
            borg_note(st, "# Scumming Mode.  Power Climb. ");

            /* Set to help borg move better */
            st.borg.goal.less = true;

            /* Continue leaving the level */
            if borg_flow_old(st, GOAL_FLEE) {
                return true;
            }

            /* Flow to UpStair */
            if borg_flow_stair_less(st, GOAL_FLEE, false) {
                borg_note(st, "# Looking for stairs. Scumming Mode.");

                /* Success */
                return true;
            }

            if feat_at(st, tx, ty) == FEAT_LESS && !engine::player_opt(OPT_birth_force_descend) {
                /* Take the Up Stair */
                borg_keypress(st, key(b'<'));
                return true;
            }
        }
    }

    /* Special case where the borg is off a stair and there
     * is a monster in LOS.  He could freeze and unhook, or
     * move to the closest stair and risk the run.
     */
    if st.borg.tr(BI_CDEPTH) >= 2 {
        /* Continue fleeing to stair */
        if borg_flow_old(st, GOAL_FLEE) {
            return true;
        }

        /* Note */
        borg_note(st, "# Scumming Mode.  Any Stair. ");

        /* Try to find some stairs */
        if borg_flow_stair_both(st, GOAL_FLEE, true) {
            return true;
        }
    }

    /* return to normal borg_think_dungeon */
    false
}

/// how long should the borg explore?
fn borg_time_to_stay_on_level(st: &mut BorgState, bored: bool) -> i32 {
    let feeling_stuff = st.think_dungeon.feeling_stuff;

    if st.borg.tr(BI_CLEVEL) < 5 || feeling_stuff == 0 {
        /* at very low level, stay less time */
        if st.borg.tr(BI_CLEVEL) < 10 {
            return st.borg.tr(BI_CLEVEL).wrapping_mul(50);
        }

        /* at slightly low level, try not to run out of food staying */
        if st.borg.tr(BI_CLEVEL) < 15 && st.borg.tr(BI_FOOD) < 3 {
            return if st.borg.tr(BI_REG) != 0 { 2000 } else { 2500 };
        }

        /* at low level, don't stay too long, */
        /* but long enough to hope for a feeling */
        if st.borg.tr(BI_MAXCLEVEL) < 20 && feeling_stuff == 0 {
            return i32::from(engine::z_info().feeling_need) * 10;
        }
    }

    /* (an index outside the table panics; C reads past it) */
    if bored {
        return BORG_STUFF_FEELING[feeling_stuff as usize] / 10;
    }

    BORG_STUFF_FEELING[feeling_stuff as usize]
}

/// `borg_prepared(depth)`, keeping C's pointer: a formatted reason is the
/// one static buffer, which later formatted reasons overwrite.
#[track_caller]
fn prepared(st: &mut BorgState, depth: i32) -> Option<PreparedC> {
    crate::borg_prepared::borg_prepared_buf(st, depth)
}

/// The text a `borg_prepared()` pointer reads now.
fn prep_text(st: &mut BorgState, p: &PreparedC) -> Vec<u8> {
    match p {
        PreparedC::Text(t) => t.clone(),
        PreparedC::Buffer => crate::borg_prepared::prepared_buffer(st),
    }
}

/// `borg_note(format("<prefix>%s<suffix>", text))`
fn note_with(st: &mut BorgState, prefix: &[u8], text: &[u8], suffix: &[u8]) {
    let note = [prefix, text, suffix].concat();
    borg_note_bytes(st, &note);
}

/// Leave the level if necessary (or bored)
/// Scumming defined in borg_prepared.
pub fn borg_leave_level(st: &mut BorgState, mut bored: bool) -> bool {
    if crate::borg_healing_restock::returning(st) {
        return crate::borg_healing_restock::follow_return(st);
    }
    // borg-reborn-rs (#208): in town, an errand's trip starts by the stairs.
    if let Some(r) = crate::shell::on_leave_level(st, bored) {
        return r;
    }
    let mut g = 0;
    let mut try_not_to_descend = false;

    let mut need_restock = false;

    /* Waiting for "recall" other than depth 1 */
    if st.borg.goal.recalling != 0 && st.borg.tr(BI_CDEPTH) != 1 {
        return false;
    }

    /* Not bored if I have seen Morgoth recently */
    if st.borg.tr(BI_CDEPTH) == 100
        && st.borg.near.morgoth
        && borg_timer(st, st.borg.time.morgoth) < 5000
    {
        st.borg.goal.leaving = false;
        st.borg.goal.rising = false;
        bored = false;
    }

    /* There is a great concern about recalling back to level 100.
     * Often the borg will fall down a trap door to level 100 when he is not
     * prepared to be there.  Some classes can use Teleport Level to get
     * back up to 99,  But Warriors cannot.  Realistically the borg needs
     * be be able to scum deep in the dungeon.  But he cannot risk being
     * on 100 and using the few *Healing* pots that he managed to collect.
     * It is better for warriors to walk all the way down to 98 and scum.
     * It seems like a long and nasty crawl, but it is the best way to
     * make sure the borg survives.  Along the way he will collect the
     * Healing, Life and *Healing* that he needs.
     *
     * The other classes (or at least those who can use the Teleport Level
     * spell) will not need to do this nasty crawl.  Risky Borgs will
     * not crawl either.
     */

    /* Town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return leave_town(st, bored);
    }

    /* In the Dungeon */
    let prep_cur_depth = prepared(st, st.borg.tr(BI_CDEPTH));
    /* borg_prepared() formats every reason into one static buffer, so the
     * calls below overwrite the current depth's reason before it is used.
     * Keep a private copy of the text. */
    let prep_cur_text = prep_cur_depth.as_ref().map(|p| prep_text(st, p));
    let prep_next_depth = prepared(st, st.borg.tr(BI_CDEPTH) + 1);

    /* if not prepared for this level, head upward */
    if let Some(t) = &prep_cur_text {
        g = -1;
        note_with(st, b"# heading up, not prep for current level: ", t, b")");
    }

    /* Count sellable items */
    let sellable_item_count = borg_count_sell(st);

    /* The trip policy (borg-rs#60): None unless borg_trip_policy = 1. With
     * the policy off it only writes BORG_TRIP_RECORD, if that is set. */
    let trip = crate::borg_trip::consult(st, sellable_item_count);

    /* Do not dive when "full" of items */
    if sellable_item_count >= 12 {
        try_not_to_descend = true;
    }

    /* Do not dive when drained (whenever the experience is drained, not
     * only when an earlier check already chose to head up) */
    if st.borg.tr(BI_ISFIXEXP) != 0 {
        try_not_to_descend = true;
    }

    if let (true, Some(p)) = (bored, &prep_next_depth) {
        /* Rise a level if bored and unable to dive. */
        g = -1;
        let t = prep_text(st, p);
        note_with(st, b"# heading up (bored and unable to dive: ", &t, b")");
    } else if bored && i32::from(st.borg.avoidance) > st.borg.tr(BI_CURHP) {
        /* Rise a level if bored and spastic. */
        if prep_next_depth.is_some() {
            g = -1;
            borg_note(st, "# heading up (bored and spastic).");
        } else {
            g = 1;
            borg_note(st, "# heading down (bored and spastic).");
        }
    } else if !try_not_to_descend
        && prepared(st, st.borg.tr(BI_CDEPTH) + 5).is_none()
        && sellable_item_count < 13
    {
        /* Power dive if I am playing too shallow*/
        g = 1;
        borg_note(st, "# power dive, playing too shallow.");
    } else if !try_not_to_descend
        && prep_next_depth.is_none()
        && st.borg.tr(BI_CDEPTH) >= 75
        && st.borg.tr(BI_CDEPTH) < 100
    {
        /* Power dive if I am playing deep */
        g = 1;
        borg_note(st, "# power dive, head deep.");
    }

    /* Power-climb upwards when needed */
    if let Some(t) = &prep_cur_text {
        /* Certain checks are bypassed if Unique monster on level */
        if st.borg.mon.unique == 0 {
            /* if I am really out of depth go to town */
            if g == 0
                && prepared(st, st.borg.tr(BI_MAXDEPTH) * 5 / 10).is_some()
                && st.borg.tr(BI_MAXDEPTH) > 65
            {
                note_with(st, b"# Returning to town (too deep: ", t, b")");
                st.borg.goal.rising = true;
            } else {
                note_with(st, b"# Climbing (too deep: ", t, b")");
                g = -1;
            }
        }

        /* if I must go to town without delay */
        if borg_must_return_to_town(st).is_some() {
            let t = match borg_must_return_to_town(st) {
                Some(r) => r.text(st).to_vec(),
                None => Vec::new(), // Angband format("%s", NULL) is empty.
            };
            note_with(st, b"# returning to town to restock(too deep: ", &t, b")");
            st.borg.goal.rising = true;
            need_restock = true;
        }

        /* I must return to collect stock from the house. */
        if t.windows(18).any(|w| w == b"Collect from house") {
            borg_note(st, "# Returning to town to Collect stock.");
            st.borg.goal.rising = true;
            need_restock = true;
        }
    }

    /* If playing way too shallow return to town to recall deeper. */
    if prepared(st, st.borg.tr(BI_CDEPTH) + 20).is_none()
        && prepared(st, st.borg.tr(BI_MAXDEPTH) * 6 / 10).is_none()
        && st.borg.tr(BI_MAXDEPTH) > st.borg.tr(BI_CDEPTH) + 20
        && (st.borg.tr(BI_RECALL) >= 3 || st.borg.tr(BI_GOLD) > 2000)
    {
        borg_note(st, "# returning to town to recall back down (too shallow)");
        st.borg.goal.rising = true;
    }

    /* Preserve the Borg's selling-pressure backstop with either policy.
     * The trip policy may request an earlier return, never veto this one. */
    if ((st.cfg(BORG_WORSHIPS_GOLD) != 0 || st.borg.tr(BI_MAXCLEVEL) < 15)
        && st.borg.tr(BI_MAXCLEVEL) <= 25)
        && sellable_item_count >= 12
    {
        borg_note(st, "# Going to town (Sell Stuff, Worshipping Gold).");
        st.borg.goal.rising = true;
    }

    /* Return to town to sell stuff (use Recall) */
    if (bored && st.borg.tr(BI_MAXCLEVEL) >= 26) && sellable_item_count >= 12 {
        borg_note(st, "# Going to town (Sell Stuff).");
        st.borg.goal.rising = true;
    }

    /* Return to town when level drained */
    if st.borg.tr(BI_ISFIXLEV) != 0 {
        borg_note(st, "# Going to town (Fix Level).");
        st.borg.goal.rising = true;
    }

    /* Return to town to restore experience */
    if bored && st.borg.tr(BI_ISFIXEXP) != 0 && st.borg.tr(BI_CLEVEL) != 50 {
        borg_note(st, "# Going to town (Fix Experience).");
        st.borg.goal.rising = true;
    }

    /* return to town if it has been a while */
    if !st.borg.goal.rising
        && bored
        && !st.borg.status.vault
        && st.borg.near.unique == 0
        && borg_timer(st, st.borg.time.town) > 8000
    {
        /* don't get bored when hunting uniques */
        if st.borg.tr(BI_MAXDEPTH) < 99 || st.borg.mon.unique == 0 {
            borg_note(st, "# Going to town (I miss my home).");
            st.borg.goal.rising = true;
        }
    }

    /* return to town if been scumming for a bit */
    if st.borg.tr(BI_MAXDEPTH) >= st.borg.tr(BI_CDEPTH) + 10
        && st.borg.tr(BI_CDEPTH) <= 12
        && borg_timer(st, st.borg.time.town) > 3500
    {
        borg_note(st, "# Going to town (scumming check).");
        st.borg.goal.rising = true;
    }

    /* Return to town to drop off some scumming stuff */
    if !st.borg.status.vault
        && st.borg.tr(BI_PREP_BIG_FIGHT) != 0
        && (st.borg.tr(BI_AEZHEAL) >= 3 || st.borg.tr(BI_ALIFE) >= 1)
    {
        borg_note(st, "# Going to town (Dropping off Potions).");
        st.borg.goal.rising = true;
    }

    /* It is much safer to scum for items on 98
     * Check to see if depth 99, if Sauron is dead and Im not read to fight
     * the final battle
     */
    if st.borg.tr(BI_CDEPTH) == 99 && st.borg.tr(BI_SAURON_DEAD) != 0 && st.borg.ready_morgoth != 1
    {
        borg_note(st, "# Returning to level 98 to scum for items.");
        g = -1;
    }

    /* Power dive if Morgoth is dead */
    if st.borg.tr(BI_KING) != 0 {
        g = 1;
    }

    /* Power dive to 99 if ready */
    if st.borg.tr(BI_CDEPTH) < 100 && prepared(st, 99).is_none() {
        g = 1;
    }

    /* Climb if deeper than I want to be */
    // borg-reborn-rs (#447): use the same overlay as preparedness.
    let n = crate::errands_plan::effective_no_deeper(st.cfg(BORG_NO_DEEPER));
    if g == 0 && st.borg.tr(BI_CDEPTH) > n {
        borg_note(st, &format!("# Going up a bit (No Deeper than {n})."));
        g = -1;
    }

    /* if returning to town, try to go upstairs */
    if g == 0 && st.borg.goal.rising {
        g = -1;
    }

    /* Mega-Hack -- spend time on the first level to rotate shops */
    if st.borg.tr(BI_CLEVEL) > 10
        && st.borg.tr(BI_CDEPTH) == 1
        && borg_timer(st, st.borg.time.level) < 200
        && g < 0
        && st.borg.tr(BI_FOOD) > 1
    {
        borg_note(st, "# Staying on level 1 to rotate shops.");
        g = 0;
    }

    /* do not hangout on boring levels for *too* long */
    // borg-reborn-rs (#1098, review of #1117): nor once the stair-anchored
    // exploration (breadth-first or mixed) has covered all it may of this
    // level (`exhausted_this_think`; owner, 2026-11-23: "once there is
    // nothing to explore, take the stairs"): the stairs are taken as when
    // the time is up, before the secret-door search.
    let exhausted = crate::borg_flow_dark::exhausted_this_think(st);
    // The direction chosen at random after the walk, when either is allowed
    // (re-review of #1117); `None` otherwise.
    let mut either_way: Option<i32> = None;
    if g == 0 && (exhausted || borg_timer(st, st.borg.time.level) > borg_time_to_stay_on_level(st, bored)) {
        /* don't get bored when hunting uniques */
        if st.borg.tr(BI_MAXDEPTH) < 99 || st.borg.mon.unique == 0 {
            /* Note */
            let t = i64::from(borg_timer(st, st.borg.time.level));
            if exhausted {
                borg_note(st, &format!("# Exploration from the stairs done ({t} on level), leaving (#1098)."));
            } else {
                borg_note(st, &format!("# Spent too long ({t}) on level, leaving."));
            }

            /* if we are trying not to go down, go up*/
            if try_not_to_descend {
                g = -1;
            } else {
                /* otherwise use random stairs */
                g = if engine::randint0(100) < 50 { -1 } else { 1 };
                // borg-reborn-rs (#1098, re-review of #1117): after the
                // stair-anchored exploration, either direction is allowed,
                // so a failed one tries the other.
                either_way = exhausted.then_some(g);
            }
        }
    }

    /* The added return policy can request town, but cannot replace the
     * inherited depth or return decisions. */
    if let Some(d) = &trip {
        let before = (g, st.borg.goal.rising);
        (g, st.borg.goal.rising) = crate::borg_trip::apply_return(d.action, before);
        crate::borg_trip::record_application(st, d, before, (g, st.borg.goal.rising));
    }
    // A direction the trip policy changed, or a rise it asked for, is no
    // longer free (re-review of #1117).
    if either_way != Some(g) || st.borg.goal.rising {
        either_way = None;
    }

    /* Go Up */
    // borg-reborn-rs (#208, review #227): with its own reasons weighed, a
    // core that is bored and free to go either way follows the errand.
    if let Some(r) = crate::shell::on_leave_choice(
        st,
        bored && !st.borg.goal.rising && !need_restock && prep_cur_depth.is_none(),
    ) {
        return r;
    }

    // borg-reborn-rs (#1173): did this call already try the up stairs? The
    // exhausted-dive fallback below must not repeat a failed one.
    let mut up_tried = false;
    if g < 0 {
        if !engine::player_opt(OPT_birth_force_descend) {
            /* Take next stairs */
            borg_note(st, "# Looking for up stairs.  Going up.");
            st.borg.stair_less = true;
            up_tried = true;
        }

        /* don't recall to town from 100 if we are prepared for 99 */
        if st.borg.tr(BI_CDEPTH) < 100 || prepared(st, 99).is_none() {
            /* Recall if going to town */
            if st.borg.goal.rising
                && borg_timer(st, st.borg.time.town) > 200
                && st.borg.tr(BI_CDEPTH) >= 5
                && borg_recall(st)
            {
                borg_note(st, "# Recalling to town (goal rising)");
                return true;
            }

            /* Recall if needing to Restock */
            if need_restock && st.borg.tr(BI_CDEPTH) >= 5 && borg_recall(st) {
                borg_note(st, "# Recalling to town (need to restock)");
                return true;
            }
        }

        /* Attempt to use stairs */
        if borg_flow_stair_less(st, GOAL_BORE, false) {
            borg_note(st, "# Going to stairs up. I'm bored.");
            return true;
        } else {
            borg_note(st, "# Bored but unable to flow to up stairs.");
        }

        /* Cannot find any stairs */
        if !crate::borg_think_dungeon::bravery_recovery_enabled()
            && st.borg.goal.rising
            && bored
            && borg_timer(st, st.borg.time.level) >= 1000
            && borg_recall(st)
        {
            borg_note(st, "# Recalling to town (no stairs)");
            return true;
        }

        /* No up stairs found. do down then back up */
        // borg-reborn-rs (#483): while rising, explore for up stairs first.
        let here = crate::borg_danger::borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        if st.stairs.less.num == 0 && crate::borg_caution::rising_holds_descent(st, here) {
            borg_note(st, "# no up stairs found, exploring for them (rising)");
        } else if st.stairs.less.num == 0 {
            borg_note(st, "# no up stairs found, going down");
            g = 1;
        } else if either_way.is_some() {
            // borg-reborn-rs (#1098, re-review of #1117): the up stairs known
            // cannot be reached; after the walk, down is allowed too.
            borg_note(st, "# Up stairs out of reach; trying down stairs (#1098)");
            g = 1;
        }
    }

    /* Go Down */
    // borg-reborn-rs (#483, review): every descent decided here (bored,
    // "spent too long on level", the trip policy, the fallback above) is
    // held while rising, unless in real danger.
    let here = crate::borg_danger::borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    if g > 0 && crate::borg_caution::rising_holds_descent(st, here) {
        borg_note(st, "# Holding a descent while rising to town");
    } else if g > 0 {
        /* Take next stairs */
        st.borg.stair_more = true;

        /* Attempt to use those stairs */
        if borg_flow_stair_more(st, GOAL_BORE, false, false) {
            return true;
        }

        // borg-reborn-rs (#1174, review): standing on the very down stair
        // the dive chose, the flow has no step to queue (cost 0) and falls
        // into the fallbacks below, which would walk the Borg off the
        // staircase toward an up stair. Take the stair underfoot, with the
        // guards of caution's own stair take (recall gates, prep spells).
        // borg-reborn-rs (#1177, review): with
        // stairs_descent_gates_without_upstair, a descent the gates refuse
        // is not completed underfoot either — the refusal above is then a
        // policy, not a missed step; 0 keeps the take as the #1174 review
        // left it.
        let gates_refuse = (crate::params::value(
            crate::params::Param::stairs_descent_gates_without_upstair,
        ) != 0 || crate::borg_think_dungeon::bravery_recovery_enabled())
            && crate::borg_flow_stairs::more_refused(st, false);
        if exhausted
            && feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_MORE
            && !crate::borg_caution::recall_restricts(st)
            && !gates_refuse
        {
            borg_note(st, "# Diving by the down stair underfoot (#1174)");
            if borg_prep_leave_level_spells(st) {
                return true;
            }
            borg_keypress(st, key(b'>'));
            return true;
        }

        // borg-reborn-rs (#1098, re-review of #1117): down was chosen at
        // random after the walk and cannot be reached: up is allowed too.
        if either_way == Some(1) && !engine::player_opt(OPT_birth_force_descend) {
            borg_note(st, "# Down stairs out of reach; trying up stairs (#1098)");
            st.borg.stair_less = true;
            if borg_flow_stair_less(st, GOAL_BORE, false) {
                borg_note(st, "# Going to stairs up. I'm bored.");
                return true;
            }
        } else if exhausted && !up_tried && !engine::player_opt(OPT_birth_force_descend) {
            // borg-reborn-rs (#1173): the exploration of this think is done,
            // the dive chose down, and no down stair can be reached (none
            // known, or its flow failed): take a known up stair instead of
            // standing down to the secret-door search. The recorded Ranger
            // Borg261124105138 spent 521 decisions walking between search
            // targets on DL3 with two reachable up stairs. A reachable down
            // stair keeps the dive above, an explicit retreat holds the
            // descent before this, and forced descent forbids the up stair.
            // The up flow failing leaves the stand-down of #1098 in place.
            borg_note(st, "# No down stair to dive by; trying the up stairs (#1173)");
            st.borg.stair_less = true;
            if borg_flow_stair_less(st, GOAL_BORE, false) {
                borg_note(st, "# Going to stairs up. I'm bored.");
                return true;
            }
        }
    }

    // borg-reborn-rs (#1098): the stair-anchored exploration's area is done
    // but no staircase can be taken, in any direction allowed: it stands
    // down on this level, and C explores for one.
    if exhausted {
        crate::borg_flow_dark::release_after_failed_exit(st);
    }

    /* Failure */
    false
}

/// The town part of `borg_leave_level` (C returns from it in every case).
fn leave_town(st: &mut BorgState, bored: bool) -> bool {
    /* Cancel rising */
    st.borg.goal.rising = false;

    /* Wait until bored */
    if !bored {
        return false;
    }

    let recall_depth = if st.cfg(BORG_WORSHIPS_GOLD) != 0 { 10 } else { 8 };

    /* Case for those who cannot Teleport Level */
    if st.borg.tr(BI_MAXDEPTH) == 100
        && st.cfg(BORG_PLAYS_RISKY) == 0
        && borg_restock(st, 100).is_some()
    {
        /* These pple must crawl down to 100, Sorry */
        st.borg.goal.fleeing = true;
        st.borg.goal.leaving = true;
        st.borg.stair_more = true;

        /* Note */
        borg_note(st, "# Borg must crawl to deep dungeon- no recall to 100.");

        /* Attempt to use those stairs */
        return borg_flow_stair_more(st, GOAL_BORE, false, false);
    }

    if crate::borg_think_dungeon::bravery_recovery_enabled() {
        if st.borg.tr(BI_MAXDEPTH) >= recall_depth && st.borg.tr(BI_RECALL) >= 3 {
            match engine::town_recall_destination() {
                Some(depth) => {
                    if let Some(reason) = prepared(st, depth) {
                        let text = prep_text(st, &reason);
                        note_with(st, format!("# Recall to DL{depth} blocked: ").as_bytes(), &text, b"");
                    } else if borg_recall(st) {
                        borg_note(st, &format!("# Recalling into dungeon to DL{depth}."));
                        return true;
                    }
                }
                None => borg_note(st, "# Automatic Recall from town blocked by engine rules or a destination prompt."),
            }
        }
        return start_at_depth_one(st);
    }

    /* Recall into dungeon */
    if st.borg.tr(BI_MAXDEPTH) >= recall_depth
        && st.borg.tr(BI_RECALL) >= 3
        && (prepared(st, st.borg.tr(BI_MAXDEPTH) * 6 / 10).is_none()
            || st.cfg(BORG_PLAYS_RISKY) != 0)
        && borg_recall(st)
    {
        /* Note */
        borg_note(st, "# Recalling into dungeon.");

        /* Give it a shot */
        return true;
    }

    /* note why we didn't recall. */
    if st.borg.tr(BI_MAXDEPTH) < recall_depth {
        borg_note(st, "# Not deep enough to recall");
    } else if st.borg.tr(BI_RECALL) <= 2 {
        borg_note(st, "# Not enough recalls to recall");
    } else if prepared(st, st.borg.tr(BI_MAXDEPTH) * 6 / 10).is_some() {
        /* recall unless way out of our league */
        let t = match prepared(st, st.borg.tr(BI_MAXDEPTH)) {
            Some(p) => prep_text(st, &p),
            None => Vec::new(), // Angband format("%s", NULL) is empty.
        };
        note_with(st, b"# Way too scary to recall down there!   ", &t, b"");
    } else {
        borg_note(st, "# failed to recall when I wanted to");
    }
    st.borg.goal.fleeing = true;
    st.borg.goal.leaving = true;

    st.borg.stair_more = true;

    /* Attempt to use those stairs */
    borg_flow_stair_more(st, GOAL_BORE, false, false)
}

/// The ordinary town staircase with the depth-one gates applied even when
/// already standing on it. A flow at cost zero queues no step.
pub(crate) fn start_at_depth_one(st: &mut BorgState) -> bool {
    if st.borg.goal.recalling != 0 {
        borg_note(st, "# Bravery recovery: DL1 departure blocked by pending Recall.");
        return false;
    }
    if crate::borg_flow_stairs::more_refused(st, false) {
        if let Some(reason) = prepared(st, 1) {
            let text = prep_text(st, &reason);
            note_with(st, b"# DL1 departure blocked: ", &text, b"");
        } else {
            borg_note(st, "# Bravery recovery: DL1 departure requires light and any sale hold to clear.");
        }
        return false;
    }
    if crate::shell::shopping_departure(st) {
        return true;
    }
    if feat_at(st, st.borg.c.x, st.borg.c.y) == FEAT_MORE {
        if !borg_prep_leave_level_spells(st) {
            borg_keypress(st, key(b'>'));
        }
        return true;
    }
    if borg_flow_stair_more(st, GOAL_BORE, false, false) {
        st.borg.stair_more = true;
        return true;
    }
    borg_note(st, "# Bravery recovery: no reachable town staircase.");
    false
}

/// Excavate an existing vault using ranged spells.
/// Stand where you are, use stone to mud to excavate the vault.  This will
/// allow the druid or blackgaurd borgs to get a few more attack spells on
/// the monster. Without this routine, he would approach the vault and use
/// Stone to Mud when he was adjacent to the wall, giving him only 1 or 2
/// shots before the monster is next to the borg.
pub fn borg_excavate_vault(st: &mut BorgState, range: i32) -> bool {
    /* reset counters */
    st.flow.temp_n = 0;

    /* no need if no vault on level */
    if !st.borg.status.vault {
        return false;
    }

    /* only if you can cast the spell */
    if !borg_spell_okay_fail(st, TURN_STONE_TO_MUD, 30)
        && !borg_spell_okay_fail(st, SHATTER_STONE, 30)
    {
        return false;
    }

    /* Danger/bad idea checks */

    /* build the array -- Scan screen (a large terminal can show more than
     * the map) */
    let (hgt, wid) = screen_hgt_wid();
    let (w_x, w_y) = (st.core.w_x, st.core.w_y);
    for y in w_y..(w_y + hgt).min(AUTO_MAX_Y) {
        for x in w_x..(w_x + wid).min(AUTO_MAX_X) {
            /* only bother with near ones */
            if engine::distance(st.borg.c, loc { x, y }) > range {
                continue;
            }

            /* only deal with excavatable walls */
            let feat = feat_at(st, x, y);
            if feat != FEAT_GRANITE
                && feat != FEAT_RUBBLE
                && feat != FEAT_QUARTZ
                && feat != FEAT_MAGMA
                && feat != FEAT_QUARTZ_K
                && feat != FEAT_MAGMA_K
            {
                continue;
            }

            /* Examine grids adjacent to this grid to see if there is a perma
             * wall adjacent */
            for i in 0..8 {
                let (dx, dy) = dd(i);
                let (b_x, b_y) = (x + dx, y + dy);

                /* Bounds check */
                if !engine::square_in_bounds_fully(b_x, b_y) {
                    continue;
                }

                /* Not a perma, and not our spot. */
                if feat_at(st, b_x, b_y) != FEAT_PERM {
                    continue;
                }

                /* Track the new grid */
                let f = &mut st.flow;
                let n = usize::try_from(f.temp_n).unwrap_or(0);
                let ii = (0..n)
                    .find(|&ii| i32::from(f.temp_y[ii]) == y && i32::from(f.temp_x[ii]) == x)
                    .unwrap_or(n);

                /* Track the newly discovered excavatable wall */
                if ii == n && ii < AUTO_TEMP_MAX {
                    f.temp_x[ii] = x as u8;
                    f.temp_y[ii] = y as u8;
                    f.temp_n += 1;
                }
            }
        }
    }

    /* None to excavate */
    if st.flow.temp_n == 0 {
        return false;
    }

    /* Review the useful grids */
    let mut i = 0;
    while i < usize::try_from(st.flow.temp_n).unwrap_or(0) {
        let (tx, ty) = (i32::from(st.flow.temp_x[i]), i32::from(st.flow.temp_y[i]));
        i += 1;

        /* skip non-projectable grids grid (I cant shoot them) */
        if !borg_los(st, st.borg.c.y, st.borg.c.x, ty, tx) {
            continue;
        }

        /* Attempt to target the grid */
        borg_target(st, loc { x: tx, y: ty }, false);

        /* Attempt to excavate it with "stone to mud" */
        let sv_ring = st.item_val.sv_ring_digging;
        let act = st.act.stone_to_mud;
        if borg_spell(st, TURN_STONE_TO_MUD)
            || borg_activate_ring(st, sv_ring)
            || borg_activate_item(st, act)
            || (engine::distance(st.borg.c, loc { x: tx, y: ty }) == 1
                && borg_spell(st, SHATTER_STONE))
        {
            borg_note(st, "# Excavation of vault");
            borg_keypress(st, key(b'5'));

            /* turn that wall into a floor grid.  If the spell failed and the
             * grid is visible, it will still look like a wall and the
             * borg_update routine will redefine it as a wall
             */
            st.update.do_update_view = true;
            st.update.do_update_lite = true;

            let ag = st.cave.grid_mut(tx, ty);
            /* Not Lit */
            ag.info &= !BORG_GLOW;
            /* Dark */
            ag.info |= BORG_DARK;
            /* Feat Floor */
            ag.feat = FEAT_FLOOR as u8;
            /*
             * If the grid is not seen, prefer what the borg remembers over
             * what map_info() returns (i.e. optimistically assume that the
             * excavation was successful.
             */
            ag.info |= BORG_IGNORE_MAP;
            /* Forget number of mineral veins to force rebuild of vein list */
            st.flow_misc.track_vein.num = 0;

            return true;
        }
    }

    /* No grid to excavate */
    false
}

/// `(SCREEN_HGT, SCREEN_WID)`: the map panel in grids.
fn screen_hgt_wid() -> (i32, i32) {
    let t = engine::term_geometry();
    (
        (t.hgt - t.row_top_map - t.row_bottom_map) / t.tile_height,
        (t.wid - t.col_map - 1) / t.tile_width,
    )
}

/// #1173 (`leave-level-upstairs`): the dungeon decision on a fully explored,
/// fabricated level, with the recorded incident's decision state
/// (`Borg261124105138`, turn 177,047): a power-dive preference (the Borg is
/// prepared for this depth and for five deeper), no down staircase, two
/// reachable up staircases, and the C exploration's exhaustion of this think
/// (`explore_c_leave_when_done`). The Borg must flow to an up staircase and
/// take it, not stand down to the secret-door search. A reachable down
/// staircase keeps the dive; standing on that down staircase the dive takes
/// it underfoot instead of walking off toward an up staircase (#1174
/// review); forced descent still forbids the up staircase;
/// an explicit retreat (`goal.rising`) holds its descent; and an explicit
/// leaving (`goal.leaving`, the restock decisions' way) still ends in the
/// up-staircase flow. The driving test pins the `prepared_*` gates (base
/// and `__warrior`) to 0 and `prepared_clevel_depth_margin` to 7, so a
/// birth-level Warrior is prepared for this depth and for five deeper and
/// dives.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_leave_level_regression(st: &mut BorgState) -> ! {
    use crate::headers::list_tvals_h::{TV_FOOD, TV_POTION, TV_SCROLL};

    let start = loc { x: 40, y: 20 };
    let up_near = loc { x: 43, y: 20 };
    let up_far = loc { x: 33, y: 20 };
    let down = loc { x: 46, y: 20 };

    // The pack: the real birth kit (its power bookkeeping stays consistent)
    // plus rations, a Phase Door and a Cure Light Wounds, so the static
    // `prepared()`/`restock()` gates read light (the equipped torch), food,
    // phase and cures, and nothing stands between the Borg and its power
    // dive (the `prepared_*` gates themselves are pinned by the test). The
    // items carry their real kinds and are aware: the notice counts only
    // aware items.
    let pack_kit = |st: &mut BorgState| {
        let kind_of = |tval: u32, sval: i32| -> u32 {
            for k in 0..engine::k_max() {
                let kind = engine::kind(k);
                if kind.tval as u32 == tval && i32::from(kind.sval) == sval {
                    return k as u32;
                }
            }
            panic!("no object kind for tval {tval} sval {sval}");
        };
        let pack = i32::from(engine::z_info().pack_size) as usize;
        // The birth kit's spare wearable would be wielded before the leave
        // decision; blank it, keep everything else (the power bookkeeping
        // stays on the real kit).
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        // The rations merge into the kit's own stack: a second zero-value
        // stack would be crushed as junk before the leave decision.
        let mut ration: Option<usize> = None;
        let mut free: Option<usize> = None;
        for (i, slot) in st.items.items[..pack].iter().enumerate() {
            if slot.iqty != 0 {
                if u32::from(slot.tval) == TV_FOOD as u32
                    && i32::from(slot.sval) == st.item_val.sv_food_ration
                {
                    ration = Some(i);
                }
            } else if free.is_none() {
                free = Some(i);
            }
        }
        match ration {
            Some(i) => st.items.items[i].iqty += 6,
            None => {
                let mut item = crate::borg_item::blank_item();
                item.aware = true;
                item.ident = true;
                item.kind = kind_of(TV_FOOD as u32, st.item_val.sv_food_ration);
                item.tval = TV_FOOD as u8;
                item.sval = st.item_val.sv_food_ration as u8;
                item.iqty = 6;
                item.value = engine::kind_cost_of(item.kind as usize);
                st.items.items[free.expect("a free pack slot")] = item;
            }
        }
        for (tval, sval) in [(TV_SCROLL as u32, st.item_val.sv_scroll_phase_door), (TV_POTION as u32, st.item_val.sv_potion_cure_light)] {
            let mut item = crate::borg_item::blank_item();
            item.aware = true;
            item.ident = true;
            item.kind = kind_of(tval, sval);
            item.tval = tval as u8;
            item.sval = sval as u8;
            item.iqty = 1;
            item.value = engine::kind_cost_of(item.kind as usize);
            st.items.items[free.expect("a free pack slot")] = item;
            free = Some(free.expect("a free pack slot") + 1);
        }
        // A sentinel power: no take-off decision can score above it, and the
        // wear decisions are out (the wearable pack slots are blank). The
        // leave decision below must not be hijacked by an item decision.
        st.borg.power = i32::MAX / 2;
    };

    // The fabricated level: a known floor sea, two up stairs within the
    // leash (12 steps at CL1), optionally a down stair, and no interesting
    // grid anywhere, so C's far exploration finds nothing.
    let setup = |st: &mut BorgState, down_stair: bool| {
        // The synthetic start's one-way contract sets this; the case's
        // phases decide their own ways (the forced-descent one sets it).
        assert!(
            engine::option_set(c"birth_force_descend", false),
            "the birth option exists"
        );
        let wid = i32::from(engine::z_info().dungeon_wid);
        let hgt = i32::from(engine::z_info().dungeon_hgt);
        crate::borg_cave::borg_init_cave(st);
        for y in 0..hgt {
            for x in 0..wid {
                st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;
            }
        }
        st.cave.grid_mut(up_near.x, up_near.y).feat = FEAT_LESS as u8;
        st.cave.grid_mut(up_far.x, up_far.y).feat = FEAT_LESS as u8;
        st.borg.c = start;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        st.borg.goal.leaving = false;
        st.borg.goal.fleeing = false;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        // The tail's twitchy phase would read the pack's Phase Door; two
        // twitches are already spent, so a failing think stops before it.
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        st.stairs.less.num = 2;
        st.stairs.less.x[0] = up_near.x;
        st.stairs.less.y[0] = up_near.y;
        st.stairs.less.x[1] = up_far.x;
        st.stairs.less.y[1] = up_far.y;
        st.stairs.more.num = i16::from(down_stair);
        if down_stair {
            st.stairs.more.x[0] = down.x;
            st.stairs.more.y[0] = down.y;
            st.cave.grid_mut(down.x, down.y).feat = FEAT_MORE as u8;
        }
        crate::borg_flow::borg_flow_clear(st);
    };

    // One queued movement key: consume it as the next observation (the
    // Borg's own map is the fabricated cave, so the step always lands).
    let take_step = |st: &mut BorgState| {
        assert_eq!(st.io.queued_keys().len(), 1, "one movement key per think");
        let dir = (crate::borg_io::borg_inkey(st, true) - u32::from(b'0')) as usize;
        st.borg.c = loc {
            x: st.borg.c.x + i32::from(engine::ddx(dir)),
            y: st.borg.c.y + i32::from(engine::ddy(dir)),
        };
        st.borg.time.now += 1;
    };
    let dist_to = |st: &BorgState, to: loc| {
        (st.borg.c.x - to.x).abs().max((st.borg.c.y - to.y).abs())
    };

    // --- The incident: exhausted exploration, the dive, no down stair. ---
    setup(st, false);
    pack_kit(st);
    // The first think takes the fallback: the way out is flagged, the flow
    // to the nearer up stair is committed and its first step queued.
    assert!(
        crate::borg_think_dungeon::borg_think_dungeon(st),
        "the exhausted dive flows to an up stair (#1173)"
    );
    assert_eq!(i32::from(st.borg.goal.type_0), GOAL_BORE, "the flow is the leave decision's");
    assert!(st.borg.stair_less, "the way out is flagged for the caution stair take");
    assert!(
        st.borg.stair_explore_released.is_none(),
        "a taken exit must not stand the level down"
    );
    take_step(st);
    let mut steps = 1;
    while st.borg.c != up_near && st.borg.c != up_far {
        steps += 1;
        assert!(steps < 10, "the walk reaches the up stair");
        assert!(crate::borg_think_dungeon::borg_think_dungeon(st), "the walk continues");
        take_step(st);
    }
    assert_eq!(st.io.queued_keys(), Vec::<u32>::new(), "nothing queued on arrival");
    // On the staircase the way out is already flagged (`stair_less`), so
    // the arrival think's caution takes the stairs. The exhausted leave
    // decision itself would flow at cost 0 and fail, and its stand-down
    // stays out: the exit is the staircase underfoot.
    assert!(st.borg.stair_less, "the way out is flagged on the stair");
    assert!(
        st.borg.stair_explore_released.is_none(),
        "standing on the exit, the level is not stood down"
    );
    assert!(
        crate::borg_think_dungeon::borg_think_dungeon(st),
        "the Borg takes the up stair from its square"
    );
    assert_eq!(st.io.queued_keys(), [u32::from(b'<')], "the up stair is taken");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- A reachable down stair keeps the dive. ---
    setup(st, true);
    pack_kit(st);
    assert!(
        crate::borg_think_dungeon::borg_think_dungeon(st),
        "the dive takes the reachable down stair"
    );
    assert_eq!(i32::from(st.borg.goal.type_0), GOAL_BORE, "the dive's flow");
    assert!(!st.borg.stair_less, "no up-stair fallback beside a working dive");
    let d0 = dist_to(st, down);
    take_step(st);
    assert!(dist_to(st, down) < d0, "the step goes toward the down stair");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- Standing on the down stair, the dive takes it underfoot
    // (#1174 review). ---
    setup(st, true);
    pack_kit(st);
    // On the staircase itself the dive's flow has no step to queue (cost
    // 0); it must take the stair, not walk off toward an up stair.
    st.borg.c = down;
    assert!(
        crate::borg_think_dungeon::borg_think_dungeon(st),
        "the dive takes the stair underfoot"
    );
    assert_eq!(
        st.io.queued_keys(),
        [u32::from(b'>')],
        "the down stair is taken: {:?}",
        st.io.queued_keys()
    );
    assert!(!st.borg.stair_less, "no up-stair fallback beside the stair underfoot");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- Forced descent still forbids the up stair. ---
    setup(st, false);
    pack_kit(st);
    assert!(
        engine::option_set(c"birth_force_descend", true),
        "the birth option exists"
    );
    // The think may end in a twitchy move (the tail), but the way out is
    // never flagged and the level stands down: no up stair was tried.
    let _ = crate::borg_think_dungeon::borg_think_dungeon(st);
    assert!(
        !st.io.queued_keys().contains(&u32::from(b'<')),
        "the up stair is not taken: {:?}",
        st.io.queued_keys()
    );
    assert!(!st.borg.stair_less, "forced descent flags no up stair");
    assert_eq!(
        st.borg.stair_explore_released,
        Some(st.borg.time.level),
        "the exit failed in the only allowed direction: the stand-down"
    );
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }
    assert!(
        engine::option_set(c"birth_force_descend", false),
        "the birth option resets"
    );

    // --- An explicit retreat holds its descent. ---
    setup(st, false);
    pack_kit(st);
    st.borg.goal.rising = true;
    // The hold keeps the descent (and the new fallback) out; the think may
    // end in a twitchy move, but never a stair take or flag.
    let _ = crate::borg_think_dungeon::borg_think_dungeon(st);
    assert!(
        !st.io.queued_keys().contains(&u32::from(b'<')),
        "the retreat is not turned into a stair take: {:?}",
        st.io.queued_keys()
    );
    assert!(!st.borg.stair_less, "the retreat's hold is not turned into an ascent");
    assert!(st.borg.goal.rising, "the retreat stands");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- An explicit leaving (the restock decisions' way) flows up. ---
    setup(st, false);
    pack_kit(st);
    st.borg.goal.leaving = true;
    assert!(
        crate::borg_think_dungeon::borg_think_dungeon(st),
        "the leaving decision flows to an up stair"
    );
    assert!(st.borg.stair_less, "the leaving decision flags the way out");
    let d0 = dist_to(st, up_near);
    take_step(st);
    assert!(dist_to(st, up_near) < d0, "the step goes toward the up stair");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    println!("leave-level-upstairs passed (#1173)");
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

/// #1177 (`more-refused-gates`): the dungeon decision with no up staircase
/// seen and a reachable down staircase, on a birth-level Warrior whose pack
/// carries no food. At the shipped `stairs_descent_gates_without_upstair` 0,
/// C's wrapper ("if there are no down stairs, don't filter use of up
/// stairs") skips the descent gates and the dive takes the down staircase;
/// at 1 the hunger gate refuses the descent and no staircase is taken. One
/// arm runs per driver case: the setting is fixed for the whole process.
/// The driving test pins the `prepared_*` gates (base and `__warrior`) to 0
/// and `prepared_clevel_depth_margin` to 7, so the character is dive-prone
/// as in the #1173 fixture; `explore_c_leave_when_done` = 1 lets the
/// underfoot phases mark the level exhausted for the leave decision's own
/// exhausted branch, and `borg_town_plan` = 0 (the driver pins it in
/// place) lets the notice count plain rations. The arm with the switch on
/// runs a food-less pack: the notice reads an empty larder, which is what
/// the descent gates refuse on.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_more_refused_regression(st: &mut BorgState) -> ! {
    use crate::headers::list_tvals_h::{TV_FOOD, TV_POTION, TV_SCROLL};

    let start = loc { x: 40, y: 20 };
    let down = loc { x: 46, y: 20 };

    // The pack: the real birth kit with the spare wearables blanked, and
    // the rations merged into the kit's own ration stack (a second
    // zero-value stack would be crushed as junk) when `food` — the arm
    // without food leaves the larder empty, so the notice's food count is
    // 0 and `more_refused`'s hunger gate has grounds. The Phase Door and
    // the Cure Light Wounds keep the static `prepared()`/`restock()` gates
    // light, as in the #1173 fixture; the items carry their real kinds and
    // are aware, and the power sentinel keeps the item decisions out of
    // the way.
    let pack_kit = |st: &mut BorgState, food: bool| {
        let kind_of = |tval: u32, sval: i32| -> u32 {
            for k in 0..engine::k_max() {
                let kind = engine::kind(k);
                if kind.tval as u32 == tval && i32::from(kind.sval) == sval {
                    return k as u32;
                }
            }
            panic!("no object kind for tval {tval} sval {sval}");
        };
        let pack = i32::from(engine::z_info().pack_size) as usize;
        // The spare wearable would be wielded; blank it. Without `food`
        // the kit's own rations go too, and the notice (which
        // `borg_leave_level` re-runs from `st.items`) reads an empty
        // larder.
        for slot in &mut st.items.items[..pack] {
            if slot.iqty == 0 {
                continue;
            }
            let wearable = crate::borg_inventory::borg_wield_slot(slot) >= 0;
            let food_stack = !food && u32::from(slot.tval) == TV_FOOD as u32;
            if wearable || food_stack {
                *slot = crate::borg_item::blank_item();
            }
        }
        // The rations merge into the kit's own ration stack (a second
        // zero-value stack would be crushed as junk); without `food` the
        // stacks stay blank, and the notice reads an empty larder.
        let mut ration: Option<usize> = None;
        let mut free: Option<usize> = None;
        for (i, slot) in st.items.items[..pack].iter().enumerate() {
            if slot.iqty != 0 {
                if u32::from(slot.tval) == TV_FOOD as u32
                    && i32::from(slot.sval) == st.item_val.sv_food_ration
                    && ration.is_none()
                {
                    ration = Some(i);
                }
            } else if free.is_none() {
                free = Some(i);
            }
        }
        if food {
            match ration {
                Some(i) => st.items.items[i].iqty += 6,
                None => {
                    let mut item = crate::borg_item::blank_item();
                    item.aware = true;
                    item.ident = true;
                    item.kind = kind_of(TV_FOOD as u32, st.item_val.sv_food_ration);
                    item.tval = TV_FOOD as u8;
                    item.sval = st.item_val.sv_food_ration as u8;
                    item.iqty = 6;
                    item.value = engine::kind_cost_of(item.kind as usize);
                    let slot = free.expect("a free pack slot");
                    st.items.items[slot] = item;
                    free = Some(slot + 1);
                }
            }
        }
        for (tval, sval) in [(TV_SCROLL as u32, st.item_val.sv_scroll_phase_door), (TV_POTION as u32, st.item_val.sv_potion_cure_light)] {
            let mut item = crate::borg_item::blank_item();
            item.aware = true;
            item.ident = true;
            item.kind = kind_of(tval, sval);
            item.tval = tval as u8;
            item.sval = sval as u8;
            item.iqty = 1;
            item.value = engine::kind_cost_of(item.kind as usize);
            st.items.items[free.expect("a free pack slot")] = item;
            free = Some(free.expect("a free pack slot") + 1);
        }
        st.borg.power = i32::MAX / 2;
    };

    // The fabricated level: a known floor sea, one down staircase within
    // the leash (12 steps at CL1), and no up staircase anywhere — not in
    // the cave, and none in the track (`less.num = 0`), which is the
    // wrapper's condition under test.
    let setup = |st: &mut BorgState| {
        assert!(
            engine::option_set(c"birth_force_descend", false),
            "the birth option exists"
        );
        let wid = i32::from(engine::z_info().dungeon_wid);
        let hgt = i32::from(engine::z_info().dungeon_hgt);
        crate::borg_cave::borg_init_cave(st);
        for y in 0..hgt {
            for x in 0..wid {
                st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;
            }
        }
        st.cave.grid_mut(down.x, down.y).feat = FEAT_MORE as u8;
        st.borg.c = start;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        st.borg.goal.leaving = false;
        st.borg.goal.fleeing = false;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        // The tail's twitchy phase would read the pack's Phase Door; two
        // twitches are already spent, so a failing think stops before it.
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        st.stairs.less.num = 0;
        st.stairs.more.num = 1;
        st.stairs.more.x[0] = down.x;
        st.stairs.more.y[0] = down.y;
        crate::borg_flow::borg_flow_clear(st);
    };

    // The note the #1174-review underfoot take emits; the underfoot phases
    // scan the messages a think added for it.
    let underfoot_note = "# Diving by the down stair underfoot (#1174)";
    let notes_added = |st: &mut BorgState, before: u16| -> Vec<String> {
        let after = engine::messages_num();
        (0..after.saturating_sub(before))
            .map(|age| String::from_utf8_lossy(&engine::message_str(age)).to_string())
            .collect()
    };

    // The two arms run one per driver case: the setting is fixed for the
    // whole process, and each arm asserts the decision its setting must
    // produce.
    if crate::params::value(crate::params::Param::stairs_descent_gates_without_upstair) == 0 {
        // --- The shipped gate (0): C's wrapper skips the descent gates
        // and the dive takes the reachable down staircase. ---
        setup(st);
        pack_kit(st, true);
        assert!(
            crate::borg_think_dungeon::borg_think_dungeon(st),
            "the dive takes the reachable down stair"
        );
        assert_eq!(i32::from(st.borg.goal.type_0), GOAL_BORE, "the dive's flow");
        assert!(
            !st.io.queued_keys().contains(&u32::from(b'<')),
            "no up staircase is taken: none is known"
        );
        assert_eq!(
            st.io.queued_keys().len(),
            1,
            "one step toward the down stair: {:?}",
            st.io.queued_keys()
        );
        let d0 = (st.borg.c.x - down.x).abs().max((st.borg.c.y - down.y).abs());
        let dir = (crate::borg_io::borg_inkey(st, true) - u32::from(b'0')) as usize;
        st.borg.c = loc {
            x: st.borg.c.x + i32::from(engine::ddx(dir)),
            y: st.borg.c.y + i32::from(engine::ddy(dir)),
        };
        let d1 = (st.borg.c.x - down.x).abs().max((st.borg.c.y - down.y).abs());
        assert!(d1 < d0, "the step goes toward the down stair");
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // --- Underfoot (#1177 review): standing on the down staircase on
        // an exhausted level, the take the #1174 review added completes
        // the dive. Driven through `borg_leave_level` directly: the
        // decision under test is the standing take, and the exhausted
        // mark is set by hand (the mode flag comes from the driver's
        // explore_c_leave_when_done pin). ---
        setup(st);
        pack_kit(st, true);
        st.borg.c = down;
        st.borg.stair_explore_exhausted = Some(st.borg.time.now);
        let before = engine::messages_num();
        assert!(
            crate::borg_think_dungeon_util::borg_leave_level(st, true),
            "the leave decision acts on the exhausted level"
        );
        assert!(
            notes_added(st, before)
                .iter()
                .any(|m| m.contains(underfoot_note)),
            "the underfoot take fires at the shipped 0"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'>')),
            "the down staircase is taken: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
    } else {
        // --- The gate applied (1): the hunger gate refuses the descent;
        // no descent flow is committed and no staircase is taken in any
        // think. ---
        setup(st);
        pack_kit(st, false);
        for _ in 0..2 {
            let _ = crate::borg_think_dungeon::borg_think_dungeon(st);
            assert_eq!(
                i32::from(st.borg.goal.type_0),
                0,
                "no descent flow is committed"
            );
            assert!(
                !st.io.queued_keys().contains(&u32::from(b'>')),
                "the refused descent takes no stair: {:?}",
                st.io.queued_keys()
            );
            while !st.io.queued_keys().is_empty() {
                crate::borg_io::borg_inkey(st, true);
            }
        }

        // --- Underfoot (#1177 review): standing on the down staircase on
        // an exhausted level, the #1174 take honors the refusal. Two
        // variants, driven through `borg_leave_level` directly (the mark
        // as above; the notice does not run inside the call, so the
        // fabricated `BI_FOOD` is what `more_refused` reads): with the
        // larder noticed full the gates pass and the take fires; with it
        // noticed empty the hunger gate refuses and the note stays out —
        // the descent is not completed underfoot. ---
        setup(st);
        pack_kit(st, true);
        st.borg.c = down;
        st.borg.stair_explore_exhausted = Some(st.borg.time.now);
        let before = engine::messages_num();
        assert!(
            crate::borg_think_dungeon_util::borg_leave_level(st, true),
            "with a full larder the leave decision acts"
        );
        assert!(
            notes_added(st, before)
                .iter()
                .any(|m| m.contains(underfoot_note)),
            "the gates passing, the underfoot take fires"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'>')),
            "the down staircase is taken: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        setup(st);
        pack_kit(st, false);
        st.borg.c = down;
        st.borg.stair_explore_exhausted = Some(st.borg.time.now);
        let before = engine::messages_num();
        // The direction is random once the dive is refused (the exhausted
        // branch), but both directions leave the underfoot note out: the
        // gated take skips, and the up-stair flows fail on none known.
        let _ = crate::borg_think_dungeon_util::borg_leave_level(st, true);
        assert!(
            !notes_added(st, before)
                .iter()
                .any(|m| m.contains(underfoot_note)),
            "the refused descent is not completed underfoot"
        );
        assert!(
            !st.io.queued_keys().contains(&u32::from(b'>')),
            "the refused descent takes no stair: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
    }

    println!("more-refused-gates passed (#1177)");
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

/// #1178 (`stale-stair-more`): the stale `stair_more` and caution's standing
/// "Take stairs down". Two thinks on one fabricated DL3 — an up staircase
/// known nearby (so `more_refused`'s wrapper applies its gates), no down
/// staircase in the track, and a birth Warrior with the driving test's
/// `prepared_*` pins (prepared for its depth and for five deeper, which
/// keeps the stair leashes out and is the power dive's own trigger). Think
/// 1 carries a full larder: its dive fails on no known down staircase and
/// the flag survives (C clears it only on a new level). The food is then
/// taken away — the notice inside the next `borg_think_dungeon` reads an
/// empty larder — and the Borg is placed on a `FEAT_MORE` grid. At the
/// shipped `caution_standing_descent_gated` 0, think 2's caution takes the
/// stair on the stale flag (`>`, C's defect); at 1 the descent gates
/// (hunger, and the failed restock gate of `prepared()`) refuse it. One
/// arm runs per driver case: the setting is fixed for the whole process.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_stale_stair_more_regression(st: &mut BorgState) -> ! {
    use crate::headers::list_tvals_h::{TV_FOOD, TV_POTION, TV_SCROLL};

    let start = loc { x: 40, y: 20 };
    let up = loc { x: 43, y: 20 };
    let down = loc { x: 46, y: 20 };

    // The pack: the real birth kit with its rations topped up to six, the
    // spare wearables blanked, and a Phase Door and a Cure Light Wounds
    // with real kinds, aware, as in the #1173 fixture; the power sentinel
    // keeps the item decisions away. Think 1 carries the food (the
    // character is fully prepared, so the power dive fires and the stair
    // leashes stay out); before think 2 the food is taken away and the
    // notice reads an empty larder, which is what the descent gates
    // refuse on.
    let pack_kit = |st: &mut BorgState| {
        let kind_of = |tval: u32, sval: i32| -> u32 {
            for k in 0..engine::k_max() {
                let kind = engine::kind(k);
                if kind.tval as u32 == tval && i32::from(kind.sval) == sval {
                    return k as u32;
                }
            }
            panic!("no object kind for tval {tval} sval {sval}");
        };
        let pack = i32::from(engine::z_info().pack_size) as usize;
        // The spare wearable would be wielded before the decision; blank
        // it. The rations merge into the kit's own ration stack (a second
        // zero-value stack would be crushed as junk), and six of them keep
        // the restock gate of `prepared()` fed for think 1.
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        let mut ration: Option<usize> = None;
        let mut free: Option<usize> = None;
        for (i, slot) in st.items.items[..pack].iter().enumerate() {
            if slot.iqty != 0 {
                if u32::from(slot.tval) == TV_FOOD as u32
                    && i32::from(slot.sval) == st.item_val.sv_food_ration
                    && ration.is_none()
                {
                    ration = Some(i);
                }
            } else if free.is_none() {
                free = Some(i);
            }
        }
        match ration {
            Some(i) => st.items.items[i].iqty += 6,
            None => {
                let mut item = crate::borg_item::blank_item();
                item.aware = true;
                item.ident = true;
                item.kind = kind_of(TV_FOOD as u32, st.item_val.sv_food_ration);
                item.tval = TV_FOOD as u8;
                item.sval = st.item_val.sv_food_ration as u8;
                item.iqty = 6;
                item.value = engine::kind_cost_of(item.kind as usize);
                st.items.items[free.expect("a free pack slot")] = item;
            }
        }
        for (tval, sval) in [(TV_SCROLL as u32, st.item_val.sv_scroll_phase_door), (TV_POTION as u32, st.item_val.sv_potion_cure_light)] {
            let mut item = crate::borg_item::blank_item();
            item.aware = true;
            item.ident = true;
            item.kind = kind_of(tval, sval);
            item.tval = tval as u8;
            item.sval = sval as u8;
            item.iqty = 1;
            item.value = engine::kind_cost_of(item.kind as usize);
            st.items.items[free.expect("a free pack slot")] = item;
            free = Some(free.expect("a free pack slot") + 1);
        }
        st.borg.power = i32::MAX / 2;
    };

    // Between the thinks: every food stack is emptied, so think 2's notice
    // (which runs inside `borg_think_dungeon`) reads an empty larder —
    // `BI_FOOD` 0 — and the descent gates refuse on it.
    let take_the_food = |st: &mut BorgState| {
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && u32::from(slot.tval) == TV_FOOD as u32 {
                *slot = crate::borg_item::blank_item();
            }
        }
    };

    // The fabricated level: a known floor sea, one up staircase known
    // nearby (the wrapper's condition), one FEAT_MORE grid with no down
    // staircase in the track (the flows fail on nothing known, and
    // caution's standing take reads the grid's feat).
    let setup = |st: &mut BorgState| {
        assert!(
            engine::option_set(c"birth_force_descend", false),
            "the birth option exists"
        );
        let wid = i32::from(engine::z_info().dungeon_wid);
        let hgt = i32::from(engine::z_info().dungeon_hgt);
        crate::borg_cave::borg_init_cave(st);
        for y in 0..hgt {
            for x in 0..wid {
                st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;
            }
        }
        st.cave.grid_mut(up.x, up.y).feat = FEAT_LESS as u8;
        st.cave.grid_mut(down.x, down.y).feat = FEAT_MORE as u8;
        st.borg.c = start;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        st.borg.goal.leaving = false;
        st.borg.goal.fleeing = false;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        // The tail's twitchy phase would read the pack's Phase Door; two
        // twitches are already spent, so a failing think stops before it.
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        st.stairs.less.num = 1;
        st.stairs.less.x[0] = up.x;
        st.stairs.less.y[0] = up.y;
        st.stairs.more.num = 0;
        crate::borg_flow::borg_flow_clear(st);
    };

    let gated = crate::params::value(crate::params::Param::caution_standing_descent_gated) != 0;

    // --- Think 1: the dive fires (the character is fully prepared with a
    // full larder) and fails on no known down staircase; the flag survives
    // (C clears it only on a new level). ---
    setup(st);
    pack_kit(st);
    let _ = crate::borg_think_dungeon::borg_think_dungeon(st);
    assert!(
        !st.io.queued_keys().contains(&u32::from(b'>')),
        "think 1 stands off no stair: {:?}",
        st.io.queued_keys()
    );
    assert!(st.borg.stair_more, "the failed dive leaves the flag set");
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- Think 2: the larder is empty and the Borg stands on the
    // FEAT_MORE grid with the stale flag. Think 2 drives caution directly
    // (as EmergencyStairHandoffTest does): the decision under test is
    // caution's standing take, and the barren level's full dungeon tail
    // (spastic searches, the twitchy recall) would only bury it.
    // `borg_caution` does not re-notice, so the traits are set as the
    // notice would have left them for an empty larder. ---
    take_the_food(st);
    st.borg.trait_[BI_FOOD as usize] = 0;
    st.borg.c = down;
    // Think 1's notices rebuild the staircase track from the cave's marks,
    // which these fabricated grids do not carry; re-assert the fabricated
    // knowledge that `more_refused`'s wrapper reads.
    st.stairs.less.num = 1;
    st.stairs.less.x[0] = up.x;
    st.stairs.less.y[0] = up.y;
    st.borg.time.now += 1;
    st.borg.goal.type_0 = 0;
    crate::borg_flow::borg_flow_clear(st);
    let _ = crate::borg_caution::borg_caution(st);
    if !gated {
        // C: the stale flag takes the stair.
        assert!(
            st.io.queued_keys().contains(&u32::from(b'>')),
            "the stale flag takes the standing descent: {:?}",
            st.io.queued_keys()
        );
    } else {
        // Gated: the hunger gate refuses the stale flag's descent.
        assert!(
            !st.io.queued_keys().contains(&u32::from(b'>')),
            "the gated stale flag takes no stair: {:?}",
            st.io.queued_keys()
        );
        assert_eq!(
            i32::from(st.borg.goal.type_0),
            0,
            "no descent flow commits under the gates"
        );
    }
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    // --- Think 3 (#1178 review): the fresh emergency intent stays
    // eligible. A recovering Borg (HP below the maximum) stands on the
    // down staircase a monster threatens (the fabricated fear map at the
    // Borg's square): the selection machinery — leave_stairs_on_monster,
    // driver-pinned on — sets the flag during this call, and the switch
    // must not veto that fresh escape the way it vetoes the stale flag.
    // Gated arm only: at the shipped 0 this is C's play either way. ---
    if gated {
        setup(st);
        pack_kit(st);
        // No cure to quaff: the recovery would otherwise drink it before
        // the leave rule acts.
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && u32::from(slot.tval) == TV_POTION as u32 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.c = down;
        st.borg.stair_explore_exhausted = Some(st.borg.time.now);
        st.borg.trait_[BI_CURHP as usize] =
            st.borg.tr(crate::borg_headers::borg_trait_h::BI_MAXHP) - 10;
        // The threat: above the leave rule's threshold (a tenth of the
        // current HP, so 1 here) but below the excessive-danger flee
        // (twice the current HP, so 20) — the selection machinery's own
        // escape must choose the stair, not the level flee.
        st.danger.fear_monsters[down.y as usize][down.x as usize] = 15;
        st.borg.time.now += 1;
        crate::borg_flow::borg_flow_clear(st);
        let before = engine::messages_num();
        assert!(
            crate::borg_caution::borg_caution(st),
            "the emergency descent acts"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'>')),
            "the fresh emergency intent takes the stair: {:?}",
            st.io.queued_keys()
        );
        assert!(
            st.borg.stair_more,
            "the selection machinery flagged the descent"
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // --- Think 3b (re-review of #1185): the same emergency with the
        // flag ALREADY set on entry (the failed dive's stale flag): the
        // leave rule re-selects the staircase underfoot, and the selector's
        // own decision — not the flag transition — keeps the escape
        // eligible. ---
        setup(st);
        pack_kit(st);
        // No cure to quaff (as in think 3): the recovery would otherwise
        // drink it before the leave rule acts.
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && u32::from(slot.tval) == TV_POTION as u32 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.c = down;
        st.borg.stair_explore_exhausted = Some(st.borg.time.now);
        st.borg.stair_more = true;
        st.borg.trait_[BI_CURHP as usize] =
            st.borg.tr(crate::borg_headers::borg_trait_h::BI_MAXHP) - 10;
        st.danger.fear_monsters[down.y as usize][down.x as usize] = 15;
        st.borg.time.now += 1;
        crate::borg_flow::borg_flow_clear(st);
        let before = engine::messages_num();
        assert!(
            crate::borg_caution::borg_caution(st),
            "the re-selected emergency descent acts"
        );
        let after = engine::messages_num();
        let _ = after;
        // The message ring saturates (messages_num stops growing), so scan
        // a fixed recent window rather than a before/after delta.
        let leave_rule_selected = (0..60u16).any(|age| {
            String::from_utf8_lossy(&engine::message_str(age))
                .contains("a monster while recovering")
        });
        assert!(
            leave_rule_selected,
            "the leave rule selected the staircase anew"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'>')),
            "the re-selected emergency intent takes the stair: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
    }

    println!("stale-stair-more passed (#1178)");
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

/// #1176 (`note-standing-flags`): caution's danger + standing-on-up-stair
/// trigger on a fabricated DL3. A birth Warrior with the driving test's
/// `prepared_*` pins stands on a tracked up staircase (the track and the
/// grid feat agree), with a fabricated fear map at its square sized between
/// the trigger's threshold (`pos_danger > avoidance /
/// caution_stair_danger_divisor`, and `avoidance = CURHP`) and the
/// excessive-danger flee (`pos_danger > CURHP *
/// caution_flee_danger_hp_multiplier`), and the traits noticed as a full
/// larder reads (`BI_FOOD` 6). Think 2 drives `borg_caution` directly (the
/// `EmergencyStairHandoffTest` pattern: the decision under test is the
/// standing take's trigger, and `borg_caution` does not re-notice, so the
/// fabricated traits hold). At the shipped `caution_note_standing_on_stairs`
/// 0, C's dead flags leave the trigger's arm dead: no staircase is flagged
/// or taken. At 1 the flags mean their intent, `choose_stairs` flags the
/// way out, and caution takes the stairs up (`<`). One arm runs per driver
/// case: the setting is fixed for the whole process.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_note_standing_regression(st: &mut BorgState) -> ! {
    use crate::borg_headers::borg_trait_h::{BI_CLEVEL, BI_CURHP, BI_FOOD};

    let up = loc { x: 43, y: 20 };

    // The pack: the real birth kit with the spare wearables blanked, the
    // rations merged into the kit's own ration stack, a Phase Door and a
    // Cure Light Wounds with real kinds, aware; the power sentinel keeps
    // the item decisions away.
    let pack_kit = |st: &mut BorgState| {
        let kind_of = |tval: u32, sval: i32| -> u32 {
            for k in 0..engine::k_max() {
                let kind = engine::kind(k);
                if kind.tval as u32 == tval && i32::from(kind.sval) == sval {
                    return k as u32;
                }
            }
            panic!("no object kind for tval {tval} sval {sval}");
        };
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        let mut ration: Option<usize> = None;
        let mut free: Option<usize> = None;
        for (i, slot) in st.items.items[..pack].iter().enumerate() {
            if slot.iqty != 0 {
                if u32::from(slot.tval) == crate::headers::list_tvals_h::TV_FOOD as u32
                    && i32::from(slot.sval) == st.item_val.sv_food_ration
                    && ration.is_none()
                {
                    ration = Some(i);
                }
            } else if free.is_none() {
                free = Some(i);
            }
        }
        match ration {
            Some(i) => st.items.items[i].iqty += 6,
            None => {
                let mut item = crate::borg_item::blank_item();
                item.aware = true;
                item.ident = true;
                item.kind = kind_of(crate::headers::list_tvals_h::TV_FOOD as u32, st.item_val.sv_food_ration);
                item.tval = crate::headers::list_tvals_h::TV_FOOD as u8;
                item.sval = st.item_val.sv_food_ration as u8;
                item.iqty = 6;
                item.value = engine::kind_cost_of(item.kind as usize);
                let slot = free.expect("a free pack slot");
                st.items.items[slot] = item;
                free = Some(slot + 1);
            }
        }
        for (tval, sval) in [(
            crate::headers::list_tvals_h::TV_SCROLL as u32,
            st.item_val.sv_scroll_phase_door,
        ), (
            crate::headers::list_tvals_h::TV_POTION as u32,
            st.item_val.sv_potion_cure_light,
        )] {
            let mut item = crate::borg_item::blank_item();
            item.aware = true;
            item.ident = true;
            item.kind = kind_of(tval, sval);
            item.tval = tval as u8;
            item.sval = sval as u8;
            item.iqty = 1;
            item.value = engine::kind_cost_of(item.kind as usize);
            let slot = free.expect("a free pack slot");
            st.items.items[slot] = item;
            free = Some(slot + 1);
        }
        st.borg.power = i32::MAX / 2;
    };

    // The fabricated level: a known floor sea and one up staircase — the
    // track and the grid feat agree, and the Borg stands on it.
    let setup = |st: &mut BorgState| {
        assert!(
            engine::option_set(c"birth_force_descend", false),
            "the birth option exists"
        );
        let wid = i32::from(engine::z_info().dungeon_wid);
        let hgt = i32::from(engine::z_info().dungeon_hgt);
        crate::borg_cave::borg_init_cave(st);
        for y in 0..hgt {
            for x in 0..wid {
                st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;
            }
        }
        st.cave.grid_mut(up.x, up.y).feat = FEAT_LESS as u8;
        st.borg.c = up;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        st.borg.goal.leaving = false;
        st.borg.goal.fleeing = false;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        // The threat, at the Borg's own square: above the trigger's
        // threshold (avoidance / caution_stair_danger_divisor = 20 / 2) and
        // below the excessive-danger flee (CURHP * 2 = 40).
        st.danger.fear_monsters[up.y as usize][up.x as usize] = 15;
        st.stairs.less.num = 1;
        st.stairs.less.x[0] = up.x;
        st.stairs.less.y[0] = up.y;
        st.stairs.more.num = 0;
        crate::borg_flow::borg_flow_clear(st);
    };

    // The traits a notice leaves for a full larder: food in the pack and
    // the character's own level (the trigger's low-level gate).
    let noticed = |st: &mut BorgState| {
        st.borg.trait_[BI_FOOD as usize] = 6;
        st.borg.trait_[BI_CLEVEL as usize] = 1;
    };

    let gated = crate::params::value(crate::params::Param::caution_note_standing_on_stairs) != 0;

    setup(st);
    pack_kit(st);
    noticed(st);
    if !gated {
        // --- The shipped 0: C's dead flags leave the trigger's arm dead,
        // and the case is handled by the escape decision instead (which
        // spends the escapes counter and clears the flows) — not by
        // caution's own stair take. ---
        assert!(!st.borg.stair_less, "no staircase is flagged on entry");
        let _ = crate::borg_caution::borg_caution(st);
        let notes = (0..60u16).map(|age| String::from_utf8_lossy(&engine::message_str(age)).to_string()).collect::<Vec<_>>();
        assert!(
            notes.iter().any(|m| m.contains("# Escaping level via stairs.")),
            "the escape decision handles the case at the shipped 0"
        );
        assert!(
            !notes.iter().any(|m| m.contains("# Leaving via up stairs.")),
            "caution's own stair take stays out at the shipped 0"
        );
    } else {
        // --- The flags' intent (1): the danger arm fires, choose_stairs
        // flags the way out, caution takes the stairs up — before the
        // escape decision would. ---
        assert!(!st.borg.stair_less, "no staircase is flagged on entry");
        assert!(
            crate::borg_caution::borg_caution(st),
            "the danger + standing-on-stair trigger acts"
        );
        let notes = (0..60u16).map(|age| String::from_utf8_lossy(&engine::message_str(age)).to_string()).collect::<Vec<_>>();
        assert!(
            notes.iter().any(|m| m.contains("# Leaving via up stairs.")),
            "caution's own stair take fires"
        );
        assert!(
            !notes.iter().any(|m| m.contains("# Escaping level via stairs.")),
            "the escape decision stays out"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'<')),
            "the up staircase is taken: {:?}",
            st.io.queued_keys()
        );
    }
    while !st.io.queued_keys().is_empty() {
        crate::borg_io::borg_inkey(st, true);
    }

    println!("note-standing-flags passed (#1176)");
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

/// #1189 (review of PR #1191, finding 2): the recorded DL1 geometry driven
/// through the production exploration — `borg_think_dungeon` on the
/// fabricated cave, not a test copy of the candidate filter. The region is
/// Borg261124143216's memory map at the pause (x = 104..126, y = 27..47;
/// the corridor rows are `borg_flow_dark.rs`'s `DL1_VEIN_LOOP`, extended
/// north to the memory's edge, where a cut would fabricate a frontier the
/// recording did not have). The corridor at y = 44 runs between (113,44)
/// and (110,44); the treasure vein west of it is at (109,44), the one
/// south-east at (114,45); the up staircase is at (112,36).
///
/// With `explore_stepable_goals = 1` the veins are no goals: the first
/// think commits one flow (to the unknown grids of ring 11, east and
/// north) and the Borg walks out of the corridor. With 0 the first think
/// commits twice — the recorded "dark-bfs (ring 10) at cost 1" beside the
/// vein, whose step never comes, then "ring 11 at cost 4" back east — and
/// the Borg walks 444666 inside the corridor still.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_dark_vein_loop_regression(st: &mut BorgState, on: bool) -> ! {
    use crate::borg_headers::borg_trait_h::{BI_CDEPTH, BI_CLEVEL, BI_CURHP, BI_FOOD, BI_LIGHT, BI_MAXHP};
    use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE, FEAT_LESS, FEAT_MAGMA_K, FEAT_NONE, FEAT_OPEN};

    assert_eq!(st.borg.tr(BI_CDEPTH), 1, "the fabricated level is DL1");

    const ROWS: [&str; 21] = [
        "?????????#.#???????????",
        "?????????#.##.#????????",
        "?????????#.##.##???????",
        "?????????#.##..#???????",
        "?????????#.##..#???????",
        "?????????#.##..#???????",
        "?????????#.#...#???????",
        "?????????#.#.#.####????",
        "???????###...#.....????",
        "???????#<....'..###????",
        "?????######.###.###?##?",
        "?????#.................",
        "?????#.................",
        "?????#.................",
        "?????#.................",
        "?????#.................",
        "?????#.................",
        "?????*...@.............",
        "?????.####*############",
        "???????????????????????",
        "???????????????????????",
    ];

    let setup = |st: &mut BorgState, at: loc| {
        // The synthetic start's one-way contract sets this; the case's
        // geometry decides its own ways (the recipe, step 4).
        assert!(engine::option_set(c"birth_force_descend", false), "the birth option exists");
        crate::borg_cave::borg_init_cave(st);
        for (row, y) in ROWS.iter().zip(27..) {
            for (ch, x) in row.bytes().zip(104..) {
                let feat = match ch {
                    b'?' => FEAT_NONE,
                    b'#' => FEAT_GRANITE,
                    b'.' | b'@' => FEAT_FLOOR,
                    b'<' => FEAT_LESS,
                    b'*' => FEAT_MAGMA_K,
                    b'\'' => FEAT_OPEN,
                    other => panic!("unexpected region char {other}"),
                };
                st.cave.grid_mut(x, y).feat = feat as u8;
            }
        }
        // A staircase exists twice (the recipe): the grid above and the
        // track the flows search.
        st.stairs.less.num = 1;
        st.stairs.less.x[0] = 112;
        st.stairs.less.y[0] = 36;
        st.stairs.more.num = 0;
        st.borg.c = at;
        // The recorded Half-Troll Warrior at CL1 has 23 max HP, below the
        // breadth-first threshold. The fixture disables stair scumming.
        st.borg.avoidance = 23;
        for (index, value) in [(BI_CDEPTH, 1), (BI_CLEVEL, 1), (BI_CURHP, 23), (BI_MAXHP, 23), (BI_LIGHT, 1), (BI_FOOD, 4)] {
            st.borg.trait_[index as usize] = value;
        }
        st.borg.status.vault = false;
        st.borg.near.breeder = false;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        st.borg.goal.leaving = false;
        st.borg.goal.fleeing = false;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        // A failing think stops before the twitchy tail (the recipe).
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        // The birth kit's spare wearable would be wielded before the
        // exploration (the recipe); the sentinel power keeps every item
        // decision out of the exploration think.
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.power = i32::MAX / 2;
        crate::borg_flow::borg_flow_clear(st);
    };

    // One queued movement key per think, consumed as the next observation
    // (the Borg's own map is the fabricated cave, so the step always
    // lands; a grid stepped onto unknown is revealed as the floor it is,
    // as the engine's screen would).
    let take_step = |st: &mut BorgState| {
        assert_eq!(st.io.queued_keys().len(), 1, "one movement key per think");
        let dir = (crate::borg_io::borg_inkey(st, true) - u32::from(b'0')) as usize;
        st.borg.c = loc {
            x: st.borg.c.x + i32::from(engine::ddx(dir)),
            y: st.borg.c.y + i32::from(engine::ddy(dir)),
        };
        let here = st.cave.grid_mut(st.borg.c.x, st.borg.c.y);
        if u32::from(here.feat) == FEAT_NONE {
            here.feat = FEAT_FLOOR as u8;
        }
        st.borg.time.now += 1;
    };
    // The loop's four grids, y = 44, x = 110..113.
    let in_corridor = |st: &BorgState| st.borg.c.y == 44 && (110..=113).contains(&st.borg.c.x);

    let think = |st: &mut BorgState, label: &str| {
        let before = st.flow.commit_seq;
        assert!(
            crate::borg_think_dungeon::borg_think_dungeon(st),
            "{label}: the dungeon thinker acts"
        );
        st.flow.commit_seq - before
    };

    // --- The recorded decision at the west end (110,44). ---
    setup(st, loc { x: 110, y: 44 });
    let commits = think(st, if on { "on" } else { "off" });
    if on {
        // One commit: the flow to the usable frontier (the unknown grids
        // of ring 11); no vein goal, so no second commit beside the
        // refused vein step. The step itself is the route's first one
        // (east or north, a tie of the spread): the walk below judges it.
        assert_eq!(commits, 1, "on: one flow, straight to the frontier");
        assert_eq!(i32::from(st.borg.goal.type_0), crate::borg_flow::GOAL_DARK);
    } else {
        // Two commits: the recorded "ring 10 at cost 1" beside the west
        // vein (no step follows) and "ring 11 at cost 4" back east.
        assert_eq!(commits, 2, "off: the vein aim, refused, then the aim back east");
        assert_eq!(st.io.queued_keys(), [u32::from(b'6')], "the recorded stroke goes east");
    }
    take_step(st);

    // --- The walk. ---
    if on {
        // The Borg leaves the recorded corridor for the frontier: no
        // 444666. (It may cross the corridor's west end again later, on
        // its way to the south-west pocket, so the walk is judged by its
        // leaving, and the level's end either way: explored away, or the
        // exhaustion mark and the stairs, #1098's exit.)
        let mut left = false;
        let mut left_level = false;
        for think_n in 1..=48 {
            let commits = think(st, &format!("on walk {think_n}"));
            assert!(commits <= 1, "on: at most one committed flow per think");
            if st.io.queued_keys().contains(&u32::from(b'<')) {
                left_level = true;
                break;
            }
            take_step(st);
            left |= !in_corridor(st);
            if think_n == 24 {
                assert!(left, "on: out of the recorded corridor within 24 thinks");
            }
        }
        assert!(left || left_level, "on: the corridor loop is gone (explored away, or the level left)");
    } else {
        // The recorded loop: the Borg walks 444666 inside the corridor,
        // learning nothing, for as long as the game runs (the recording:
        // 4,000 turns, 23 cycles from turn 4,472).
        for _ in 0..48 {
            think(st, "off walk");
            take_step(st);
            assert!(in_corridor(st), "off: the recorded loop stays in the corridor");
            assert!(
                st.borg.stair_explore_exhausted.is_none(),
                "off: no exhaustion while the veins are goals"
            );
        }
    }

    println!("dark vein loop passed: {}", if on { "on" } else { "off" });
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

/// #1206 (`breeder-door-loop`): the recorded Kobold Mage
/// `Borg261124194229` spent turns 3,638,689–3,639,379 alternating
/// `# Opening a door` and `# Closing a door` 35 times on a breeder level:
/// the containment's close skips only the currently selected next grid,
/// and the flow replans after each terrain change, so the just-opened door
/// lost its protection and was closed again. This fixture runs the stair
/// flight turn by turn on a fabricated breeder level — walls everywhere,
/// a carved corridor whose only passage is the flight path's closed door,
/// an unrelated open door in a niche beside it, the down stairs beyond —
/// driving `borg_play_step` with adjacent next grids, as the flows hand
/// them, ticking the escape context at every think as `borg_think_dungeon`
/// does, and applying the door state after each action along the
/// direction the Borg actually queued (`borg_queue_direction`), advancing
/// the turn for every action. During the flight the corridor door must
/// open exactly once and never be closed again (the #1206 suppression)
/// while the unrelated niche door stays closed. Afterwards the
/// containment must close that door: once the flight's escape context
/// ends; when a new flight re-arms inside the old window after ordinary
/// play reopened the door with no containment turn in between (review 3,
/// route A — the tick observed the disarm, the containment did not); when
/// a scare is withdrawn and re-armed within one think so the flags never
/// read off (review 3, route B — the withdrawal stamps the disarm); and
/// when the new flight's own open ages out of the undo window.
#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_breeder_door_regression(st: &mut BorgState) -> ! {
    use crate::borg_headers::borg_trait_h::BI_FOOD;
    use crate::headers::list_terrain_h::{FEAT_CLOSED, FEAT_GRANITE, FEAT_LESS, FEAT_OPEN};

    let start = loc { x: 42, y: 20 };
    let door = loc { x: 43, y: 20 };
    let side = loc { x: 43, y: 21 };
    let target = loc { x: 54, y: 20 };

    let pack_kit = |st: &mut BorgState| {
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.power = i32::MAX / 2;
    };

    let setup = |st: &mut BorgState| {
        assert!(
            engine::option_set(c"birth_force_descend", false),
            "the birth option exists"
        );
        let wid = i32::from(engine::z_info().dungeon_wid);
        let hgt = i32::from(engine::z_info().dungeon_hgt);
        crate::borg_cave::borg_init_cave(st);
        for y in 0..hgt {
            for x in 0..wid {
                st.cave.grid_mut(x, y).feat = FEAT_GRANITE as u8;
            }
        }
        // The corridor: the only passage from the Borg's room to the
        // stairs is through the flight path's door.
        for x in 41..=55 {
            st.cave.grid_mut(x, 20).feat = FEAT_FLOOR as u8;
        }
        // The niche with the unrelated open door, beside the corridor.
        st.cave.grid_mut(side.x, side.y).feat = FEAT_OPEN as u8;
        st.cave.grid_mut(door.x, door.y).feat = FEAT_CLOSED as u8;
        st.borg.c = start;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.rising = false;
        // The recorded flight had both escape flags set (issue #1206).
        st.borg.goal.leaving = true;
        st.borg.goal.fleeing = true;
        st.borg.goal.fleeing_to_town = false;
        st.borg.goal.less = false;
        st.borg.near.breeder = true;
        st.leash.return_any = false;
        st.borg.stair_explore_exhausted = None;
        st.borg.stair_explore_released = None;
        st.borg.times_twitch = 2;
        st.borg.no_retreat = 0;
        st.borg.time.now += 1;
        st.borg.time.level = st.borg.time.now;
        st.kills.cnt = 0;
        st.kills.nxt = 1;
        st.takes.cnt = 0;
        st.takes.nxt = 1;
        st.flow_misc.track_vein.num = 0;
        st.danger = Default::default();
        st.stairs.less.num = 0;
        st.stairs.more.num = 1;
        st.stairs.more.x[0] = target.x;
        st.stairs.more.y[0] = target.y;
        st.flow.track_door.num = 0;
        st.flow.track_closed.num = 0;
        st.flow.opened_doors.clear();
        crate::borg_flow::borg_flow_clear(st);
    };

    let door_open = |st: &BorgState| u32::from(st.cave.grid(door.x, door.y).feat) == FEAT_OPEN;
    let side_open = |st: &BorgState| u32::from(st.cave.grid(side.x, side.y).feat) == FEAT_OPEN;

    // One think of the game loop: the step toward the adjacent grid
    // `next` ran, now apply the game's response to the queued action. A
    // door action faces the direction `borg_queue_direction` remembered —
    // the grid the Borg actually picked, not whichever open door is
    // adjacent; a lone movement digit walks one grid; every action
    // advances the turn. `flight` marks the escape flight itself: a close
    // of the corridor door during it is the recorded alternation.
    fn apply_action(
        st: &mut BorgState,
        door: loc,
        side: loc,
        flight: bool,
        door_opened_yet: &mut bool,
        side_closed_seen: &mut bool,
        deliberate_closes: &mut usize,
    ) {
        use crate::headers::list_terrain_h::FEAT_CLOSED;

        let keys = st.io.queued_keys().clone();
        let faced = crate::borg_io::borg_get_queued_direction(st);
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
        let command = if keys.contains(&u32::from(b'c')) {
            Some(false)
        } else if keys.contains(&u32::from(b'o')) {
            Some(true)
        } else {
            None
        };
        match command {
            Some(open) => {
                assert!(
                    (u32::from(b'1')..=u32::from(b'9')).contains(&faced),
                    "a door action queues a direction (got {faced})"
                );
                let d = (faced - u32::from(b'0')) as usize;
                let at = loc {
                    x: st.borg.c.x + i32::from(engine::ddx(d)),
                    y: st.borg.c.y + i32::from(engine::ddy(d)),
                };
                assert!(at == door || at == side, "the door action faces a fabricated door (faced {at:?})");
                let open_now = u32::from(st.cave.grid(at.x, at.y).feat) == FEAT_OPEN;
                if open {
                    assert!(!open_now, "the open faces a closed door");
                    assert!(at == door, "the flight opens the corridor door");
                    st.cave.grid_mut(at.x, at.y).feat = FEAT_OPEN as u8;
                    *door_opened_yet = true;
                } else {
                    assert!(open_now, "the close faces an open door");
                    st.cave.grid_mut(at.x, at.y).feat = FEAT_CLOSED as u8;
                    if at == side {
                        *side_closed_seen = true;
                    } else if flight {
                        panic!("the containment closed the flight's door again");
                    } else {
                        *deliberate_closes += 1;
                    }
                }
            }
            None => {
                let digit = keys
                    .iter()
                    .find(|k| (u32::from(b'1')..=u32::from(b'9')).contains(k))
                    .expect("the think queues a door action or a walk");
                let d = (*digit - u32::from(b'0')) as usize;
                st.borg.c = loc {
                    x: st.borg.c.x + i32::from(engine::ddx(d)),
                    y: st.borg.c.y + i32::from(engine::ddy(d)),
                };
            }
        }
        // Every action is a game turn.
        st.borg.time.now += 1;
    }

    setup(st);
    pack_kit(st);
    st.borg.trait_[BI_FOOD as usize] = 6;

    let mut door_opened_yet = false;
    let mut side_closed_seen = false;
    let mut deliberate_closes = 0usize;

    // The flight: staged adjacent next grids, as the flows hand them,
    // with the escape-context tick each think (armed here, so it never
    // stamps). The first think closes the niche door, the second opens
    // the corridor door, the third walks onto it, and the replanned walks
    // past it — from the fifth think on the open door is a neighbour
    // behind the Borg while the next grid is the corridor ahead: the
    // shape that closed the recorded door 35 times.
    let path = [
        door,
        door,
        door,
        loc { x: 44, y: 20 },
        loc { x: 45, y: 20 },
        loc { x: 46, y: 20 },
        loc { x: 47, y: 20 },
        loc { x: 48, y: 20 },
        loc { x: 49, y: 20 },
    ];
    for next in path {
        assert!(
            engine::distance(st.borg.c, next) == 1,
            "the flows hand adjacent next grids (at {:?} for {next:?})",
            st.borg.c
        );
        crate::borg_flow::escape_context_tick(st);
        let _ = crate::borg_flow::borg_play_step(st, next.y, next.x);
        apply_action(
            st,
            door,
            side,
            true,
            &mut door_opened_yet,
            &mut side_closed_seen,
            &mut deliberate_closes,
        );
    }

    assert!(
        st.borg.c.x >= loc { x: 47, y: 20 }.x,
        "the flight walks past the door (at {:?})",
        st.borg.c
    );
    assert!(door_open(st), "the flight path's door stays open");
    assert!(!side_open(st), "the unrelated door stays closed");
    assert!(side_closed_seen, "the containment closed the side door");
    assert!(door_opened_yet, "the flight opened the corridor door");

    // A deliberate containment once the escape context ends: the
    // exemption must lapse with it (review of PR #1207, [P2]). The tick
    // observes the disarm at the think's start; the containment then
    // closes the door.
    st.borg.c = loc { x: 44, y: 20 };
    st.borg.goal.fleeing = false;
    st.borg.goal.leaving = false;
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        false,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert_eq!(
        deliberate_closes, 1,
        "the containment closes the door once the flight's context ended"
    );
    assert!(!door_open(st), "the deliberate containment closed the door");

    // Review 3, route A: a new flight opens the corridor door itself,
    // then ends without the containment ever getting a turn while the
    // context is disarmed — the tick is the only observer of the disarm.
    // Ordinary play closes and reopens the door; the record still holds
    // the flight's entry. A third flight re-arms inside the old window
    // and must close the door: the entry died with its escape.
    st.borg.goal.fleeing = true;
    st.borg.goal.leaving = true;
    st.borg.time.now += 5;
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, door.y, door.x);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert!(door_open(st), "the new flight opened the corridor door");
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert!(
        st.borg.c.x >= loc { x: 45, y: 20 }.x,
        "the new flight walks past the door (at {:?})",
        st.borg.c
    );
    st.borg.goal.fleeing = false;
    st.borg.goal.leaving = false;
    crate::borg_flow::escape_context_tick(st); // the disarm: no containment turn
    st.cave.grid_mut(door.x, door.y).feat = FEAT_CLOSED as u8;
    st.cave.grid_mut(door.x, door.y).feat = FEAT_OPEN as u8; // ordinary reopen
    st.borg.goal.fleeing = true;
    st.borg.goal.leaving = true;
    st.borg.time.now += 5;
    st.borg.c = loc { x: 44, y: 20 };
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        false,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert_eq!(
        deliberate_closes, 2,
        "a new flight closes a door reopened in ordinary play"
    );
    assert!(!door_open(st), "the new flight's containment closed the door");

    // Review 3, route B: the scare's goals are withdrawn and re-armed
    // within one think, through the production transition
    // (withdraw_scare_and_observe: the withdrawal disarms the context and
    // the tick stamps it). The re-armed scare is a new escape and must
    // close the door.
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, door.y, door.x);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert!(door_open(st), "the re-armed flight opened the corridor door");
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    st.borg.c = loc { x: 44, y: 20 };
    // The withdrawal runs through the production transition: the scare
    // owns both goals, so withdraw_scare_and_observe clears them and the
    // tick stamps the disarm — then the same think re-arms a new scare.
    st.borg.mon.scary_leaving = true;
    st.borg.mon.scary_fleeing = true;
    crate::borg_caution::withdraw_scare_and_observe(st);
    st.borg.goal.fleeing = true;
    st.borg.goal.leaving = true;
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        false,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert_eq!(
        deliberate_closes, 3,
        "the re-armed scare closes the door its predecessor's flight opened"
    );
    assert!(!door_open(st), "the re-armed scare's containment closed the door");

    // Review 4: a partial withdrawal through the production transition.
    // The scare owns only `fleeing`; the restock or breeder flight keeps
    // `leaving` armed — the aggregate escape continues, so the withdrawal
    // must not stamp the disarm, and the door the flight just opened
    // stays protected while the Borg routes away from it.
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, door.y, door.x);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert!(door_open(st), "the leaving flight opened the corridor door");
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    st.borg.mon.scary_fleeing = true; // the scare owns only its fleeing
    crate::borg_caution::withdraw_scare_and_observe(st);
    assert!(st.borg.goal.leaving, "the restock flight keeps the escape armed");
    assert!(!st.borg.goal.fleeing, "the scare's own fleeing was withdrawn");
    st.borg.c = loc { x: 44, y: 20 };
    crate::borg_flow::escape_context_tick(st); // armed: no stamp
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        true,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert_eq!(
        deliberate_closes, 3,
        "the partial withdrawal keeps the armed flight's door protected"
    );
    assert!(
        door_open(st),
        "the armed flight's door stays open through the partial withdrawal"
    );

    // The undo window itself, isolated from the context bound: the
    // leaving flight's open (recorded above, still armed and continuous)
    // ages out of the window with no disarm in between — the containment
    // closes it.
    st.borg.time.now += i32::from(crate::borg_flow::OPENED_DOOR_UNDO_TURNS) + 1;
    st.borg.c = loc { x: 44, y: 20 };
    crate::borg_flow::escape_context_tick(st);
    let _ = crate::borg_flow::borg_play_step(st, 20, 45);
    apply_action(
        st,
        door,
        side,
        false,
        &mut door_opened_yet,
        &mut side_closed_seen,
        &mut deliberate_closes,
    );
    assert_eq!(
        deliberate_closes, 4,
        "the containment closes the door once the undo window expired"
    );
    assert!(!door_open(st), "the deliberate containment closed the door");

    println!("breeder-door-loop passed (#1206)");
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

#[cfg(feature = "test-stair-handoff")]
pub(crate) fn run_recovery_mana_loop_regression(st: &mut BorgState, on: bool) -> ! {
    use crate::borg_flow::borg_flow_clear;
    use crate::borg_headers::borg_trait_h::{
        BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_CURHP, BI_CURSP, BI_FOOD, BI_LIGHT, BI_MAXHP, BI_MAXSP,
        BI_SPEED,
    };
    use crate::borg_headers::borg_magic_h::HEROISM;
    use crate::borg_magic::{BORG_MAGIC_KNOW, borg_get_spell_number};
    use crate::headers::list_mon_spells_h::RSF_ARROW;
    use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_GRANITE};

    // The corridor row, as the recording's memory: floor under the Borg and
    // under the orcs it had seen.
    const ROW: i32 = 55;
    let floor = |y: i32, x: i32| -> bool { y == ROW && (50..=85).contains(&x) };
    let orcs = [(ROW, 75), (ROW, 76), (ROW, 77), (ROW, 79), (ROW, 80)];

    let setup = |st: &mut BorgState, sp: i32| {
        assert!(engine::option_set(c"birth_force_descend", false), "the birth option exists");
        crate::borg_cave::borg_init_cave(st);
        for y in (ROW - 4)..=(ROW + 6) {
            for x in 40..=90 {
                st.cave.grid_mut(x, y).feat = if floor(y, x) { FEAT_FLOOR as u8 } else { FEAT_GRANITE as u8 };
            }
        }
        st.borg.c = loc { x: 66, y: ROW };
        st.borg.avoidance = 170;
        for (index, value) in [
            (BI_CDEPTH, 19), (BI_CLASS, 3), (BI_CLEVEL, 26), (BI_CURHP, 170), (BI_MAXHP, 170),
            (BI_CURSP, sp), (BI_MAXSP, 47), (BI_SPEED, 110), (BI_LIGHT, 1), (BI_FOOD, 5),
        ] {
            st.borg.trait_[index as usize] = value;
        }
        // Heroism (book 0, offset 5, cost 2 SP) known and carried: the
        // recorded Priest's first-book prayer (its cast note read "(0,5)",
        // book 0 on the cast screen, letter 'a', spell letter 'f'). The
        // book index is the cast screen's slot, only checked non-negative
        // on the decision path; the keys stay queued.
        let hero = borg_get_spell_number(st, HEROISM);
        assert!(hero >= 0, "the priest's spell table carries Heroism");
        st.magic.magics[hero as usize].status = BORG_MAGIC_KNOW;
        st.borg.book_idx[st.magic.magics[hero as usize].book as usize] = 0;
        // The five remembered orcs, awake, out of sight: what the recorded
        // kill list held. The fields are what borg_update_kill would set.
        let mut r_idx = 0;
        for i in 1..i32::from(engine::z_info().r_max) {
            if engine::race_name(i as usize) == b"cave orc" {
                r_idx = i;
                break;
            }
        }
        assert!(r_idx > 0, "the game data has the cave orc");
        let race = engine::race(r_idx as usize);
        st.kills.cnt = orcs.len() as i16;
        st.kills.nxt = 1;
        for (n, (y, x)) in orcs.iter().enumerate() {
            let kill = &mut st.kills.kills[1 + n];
            kill.r_idx = r_idx as u16;
            kill.pos = loc { x: *x, y: *y };
            kill.ox = *x as u8;
            kill.oy = *y as u8;
            kill.awake = false;
            kill.speed = 110;
            kill.level = race.level as i16;
            kill.power = 55;
            kill.when = st.borg.time.now;
            kill.ranged_attack = 1;
            kill.spell[0] = RSF_ARROW as u8;
            for (slot, word) in kill.spell_flags.iter_mut().zip(race.spell_flags) {
                *slot = u32::from(word);
            }
        }
        st.kills.nxt = 1 + orcs.len() as i16;
        st.danger = Default::default();
        st.borg.mon.scary = false;
        st.borg.mon.breeders = 0;
        st.borg.near.breeder = false;
        st.borg.antibounce_count = 0;
        st.borg.no_rest_prep = 0;
        st.borg.temp.hero = false;
        st.borg.temp.bless = false;
        st.borg.temp.berserk = false;
        st.borg.goal.type_0 = 0;
        st.borg.goal.fleeing = false;
        st.borg.goal.leaving = false;
        st.borg.stair_less = false;
        st.borg.stair_more = false;
        st.borg.times_twitch = 2;
        st.borg.status.vault = false;
        // Leave food in the pack and preserve equipped gear; a sentinel
        // power keeps every other item decision out of the think (the
        // recipe). The decision's book index is set above.
        let pack = i32::from(engine::z_info().pack_size) as usize;
        for slot in &mut st.items.items[..pack] {
            if slot.iqty != 0 && crate::borg_inventory::borg_wield_slot(slot) >= 0 {
                *slot = crate::borg_item::blank_item();
            }
        }
        st.borg.power = i32::MAX / 2;
        borg_flow_clear(st);
        crate::borg_io::borg_note(st, "#1211 scenario marker");
    };

    // One queued movement key per think, consumed as the next observation
    // (the recipe, as dark-vein-loop's driver).
    let take_step = |st: &mut BorgState| {
        assert_eq!(st.io.queued_keys().len(), 1, "one movement key per think");
        let dir = (crate::borg_io::borg_inkey(st, true) - u32::from(b'0')) as usize;
        st.borg.c = loc {
            x: st.borg.c.x + i32::from(engine::ddx(dir)),
            y: st.borg.c.y + i32::from(engine::ddy(dir)),
        };
        st.borg.time.now += 1;
        assert!(floor(st.borg.c.y, st.borg.c.x), "the step stays on the carved floor");
    };
    let notes = |age: u16| String::from_utf8_lossy(&engine::message_str(age)).to_string();
    // Notes since this scenario's marker only: the message ring keeps older
    // scenarios' notes, and the negative asserts must not read them.
    let had_note = |want: &str| {
        for age in 0..60u16 {
            let note = notes(age);
            if note.contains("#1211 scenario marker") {
                return false;
            }
            if note.contains(want) {
                return true;
            }
        }
        false
    };

    // The production decision points, driven in the recorded order (the
    // note-standing precedent: the dungeon think's own borg_notice re-reads
    // the live engine character, so the fabricated traits hold for the
    // decision calls, not for the whole think). The order is the dungeon
    // think's: borg_caution (the defense cast), borg_continue_recovery
    // (borg_flow_old), borg_recover (the rest), borg_flow_recover (the next
    // walk), each consuming its keys as the next observation.
    use crate::borg_caution::borg_caution;
    use crate::borg_flow_misc::{borg_continue_recovery, borg_flow_recover};
    use crate::borg_recover::borg_recover;

    if on {
        // --- The shipped 1s: the cast is declined and the rest happens. ---
        setup(st, 2);
        let acted = borg_caution(st);
        assert!(
            !had_note("# Attempting to cast Hero"),
            "the mana-starved prep gate holds the Heroism cast"
        );
        if acted {
            assert!(
                st.io.queued_keys().is_empty(),
                "caution declines: {:?}",
                st.io.queued_keys()
            );
            while !st.io.queued_keys().is_empty() {
                crate::borg_io::borg_inkey(st, true);
            }
        }
        assert!(
            borg_recover(st),
            "the recovery decision acts on the recorded state (SP 2)"
        );
        assert!(
            had_note("# Resting to gain Mana"),
            "the full-HP mana rest is the recovery decision"
        );
        assert!(
            st.io.queued_keys().contains(&u32::from(b'R')),
            "the rest is issued: {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        setup(st, 0);
        assert!(borg_recover(st), "the recovery decision acts (SP 0)");
        assert!(
            had_note("# Resting to gain Mana"),
            "the empty-mana rest is the recovery decision"
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // The production handoff with a route live (review of #1215, P3):
        // the walk runs in the think's own order — borg_continue_recovery
        // first, borg_recover only where the route ends — and at the
        // endpoint the mana rest queues instead of the next route. The
        // recorded pendulum's destination is the nearest eligible grid,
        // 7 west (at (59,55) four orcs are still within max_range and the
        // corridor prices 20): the general gate is exactly what turns this
        // arrival into the rest.
        setup(st, 0);
        assert!(borg_flow_recover(st, 50), "a route commits");
        take_step(st);
        let mut walked = 0;
        while borg_continue_recovery(st) {
            assert!(
                !had_note("# Resting to gain Mana"),
                "no rest mid-route: the walk continues"
            );
            take_step(st);
            walked += 1;
        }
        assert!(walked >= 6, "the route walks to its destination: {walked}");
        assert!(
            borg_recover(st),
            "the endpoint rests instead of reversing (the general gate)"
        );
        assert!(
            had_note("# Resting to gain Mana"),
            "the mana rest is the endpoint's decision"
        );
        assert!(
            st.io.queued_keys() == vec![u32::from(b'R'), u32::from(b'*'), u32::from(KC_ENTER)],
            "the rest command is queued and no movement is: the route is not restarted, keys {:?}",
            st.io.queued_keys()
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }
    } else {
        // --- C's 0s: the recorded loop reproduces. ---
        setup(st, 2);
        assert!(
            borg_caution(st),
            "caution acts on the recorded state (SP 2)"
        );
        assert!(
            had_note("# Attempting to cast Hero"),
            "the regenerated mana is spent on Heroism (defense type 12)"
        );
        assert_eq!(
            st.io.queued_keys(),
            vec![109, 97, 102],
            "the recorded cast keys m,a,f"
        );
        while !st.io.queued_keys().is_empty() {
            crate::borg_io::borg_inkey(st, true);
        }

        // The pendulum: SP 0, no goal. Each recovery walk ends, borg_recover
        // refuses the mana rest at danger 25, and the next walk starts back
        // the other way; no rest anywhere on the corridor.
        setup(st, 0);
        let mut flows = 0;
        let mut steps = 0;
        while steps < 30 {
            if borg_continue_recovery(st) {
                take_step(st);
                steps += 1;
                continue;
            }
            assert!(
                !borg_recover(st),
                "the off arm must not rest: the recorded veto at danger 25"
            );
            assert!(
                !had_note("# Resting to gain Mana"),
                "the off arm must not rest: the recorded veto"
            );
            if borg_flow_recover(st, 50) {
                take_step(st);
                steps += 1;
                if had_note("# Flowing toward Recover Grid at cost") {
                    flows += 1;
                }
                continue;
            }
            // Nothing acted: the recorded think would twitch. Count it and
            // keep the loop bounded.
            steps += 1;
        }
        assert!(
            flows >= 2,
            "the recovery walk restarts (the recorded pacing), got {flows}"
        );
    }

    println!("recovery mana loop passed: {}", if on { "on" } else { "off" });
    // This test-only process owns a fabricated Borg observation. End before
    // its queued keys can be applied to the live engine's unrelated cave.
    std::process::exit(0);
}

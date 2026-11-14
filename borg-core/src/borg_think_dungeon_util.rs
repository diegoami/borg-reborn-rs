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
use crate::borg_flow_stairs::{borg_flow_stair_both, borg_flow_stair_less, borg_flow_stair_more};
use crate::borg_flow_take::borg_flow_take_scum;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, ESCAPE, KC_ENTER};
use crate::borg_item_use::{
    borg_activate_item, borg_activate_ring, borg_read_scroll, borg_use_staff, borg_use_things,
    borg_zap_rod,
};
use crate::borg_item_wear::borg_wear_stuff;
use crate::borg_light::{borg_light_beam, borg_maintain_light, BorgNeed};
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
use crate::headers::list_object_flags_h::OF_NO_FUEL;
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
pub fn borg_think_dungeon_light(st: &mut BorgState) -> bool {
    /* Consume needed things */
    if st.borg.tr(BI_ISHUNGRY) != 0 && borg_use_things(st) {
        return true;
    }

    let light = &st.items.items[inven_light()];
    let dying = light.timeout <= 3 && !engine::flag_has(&light.flags, OF_NO_FUEL as i32);
    if !((st.borg.tr(BI_LIGHT) == 0 || dying) && st.borg.tr(BI_CDEPTH) >= 1) {
        /* Nothing to do */
        return false;
    }

    /* I am recalling, sit here till it engages. */
    if st.borg.goal.recalling != 0 {
        /* just wait */
        borg_keypress(st, key(b'R'));
        borg_keypress(st, key(b'9'));
        borg_keypress(st, KC_ENTER);
        return true;
    }

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
                    let sv_scroll = st.item_val.sv_scroll_light;
                    if borg_activate_item(st, act_illumination)
                        || borg_activate_item(st, act_light)
                        || borg_zap_rod(st, sv_rod)
                        || borg_use_staff(st, sv_staff)
                        || borg_read_scroll(st, sv_scroll)
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
        // The lunal mode is entered and left as a pair; clearing only
        // lunal_mode here left goal.fleeing_lunal set, and the caution
        // phase kept behaving as if the Borg were still lunal.
        st.borg.lunal_mode = false;
        st.borg.goal.fleeing_lunal = false;
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

    /* Return to town to sell stuff -- No recall allowed.*/
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
    if g == 0 && st.borg.tr(BI_CDEPTH) > st.cfg(BORG_NO_DEEPER) {
        let n = st.cfg(BORG_NO_DEEPER);
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
    if g == 0 && borg_timer(st, st.borg.time.level) > borg_time_to_stay_on_level(st, bored) {
        /* don't get bored when hunting uniques */
        if st.borg.tr(BI_MAXDEPTH) < 99 || st.borg.mon.unique == 0 {
            /* Note */
            let t = i64::from(borg_timer(st, st.borg.time.level));
            borg_note(st, &format!("# Spent too long ({t}) on level, leaving."));

            /* if we are trying not to go down, go up*/
            if try_not_to_descend {
                g = -1;
            } else {
                /* otherwise use random stairs */
                g = if engine::randint0(100) < 50 { -1 } else { 1 };
            }
        }
    }

    /* Go Up */
    if g < 0 {
        if !engine::player_opt(OPT_birth_force_descend) {
            /* Take next stairs */
            borg_note(st, "# Looking for up stairs.  Going up.");
            st.borg.stair_less = true;
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
        if st.borg.goal.rising
            && bored
            && borg_timer(st, st.borg.time.level) >= 1000
            && borg_recall(st)
        {
            borg_note(st, "# Recalling to town (no stairs)");
            return true;
        }

        /* No up stairs found. do down then back up */
        if st.stairs.less.num == 0 {
            borg_note(st, "# no up stairs found, going down");
            g = 1;
        }
    }

    /* Go Down */
    if g > 0 {
        /* Take next stairs */
        st.borg.stair_more = true;

        /* Attempt to use those stairs */
        if borg_flow_stair_more(st, GOAL_BORE, false, false) {
            return true;
        }
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

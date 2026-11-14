//! Port of `borg-escape.c`: various routines to help the borg run away.
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

use crate::borg::BorgTime;
use crate::borg_cave::{AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::{
    borg_cave_floor_bold, borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid,
};
use crate::borg_danger::borg_danger;
use crate::borg_flow_kill::borg_shoot_scoot_safe;
use crate::borg_io::{borg_keypress, borg_note, ESCAPE};
use crate::borg_item_use::{
    borg_activate_item, borg_read_scroll, borg_use_staff, borg_use_staff_fail, borg_zap_rod,
};
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_okay_fail};
use crate::borg_projection::{borg_distance, borg_los, borg_target};
use crate::borg_update::Detect;
use crate::engine;
use crate::borg_headers::borg_h::BORG_PLAYS_RISKY;
use crate::borg_headers::borg_magic_h::{
    DIMENSION_DOOR, PHASE_DOOR, PORTAL, SHADOW_SHIFT, TELEPORT_LEVEL, TELEPORT_SELF,
    WORD_OF_RECALL,
};
use crate::borg_headers::borg_trait_h::{
    BI_AESCAPE, BI_AMISSILES, BI_APHASE, BI_ATELEPORT, BI_CDEPTH, BI_CLASS, BI_CLEVEL,
    BI_CRSNOTEL, BI_CURHP, BI_CURSP, BI_ISAFRAID, BI_ISCUT, BI_ISHEAVYSTUN, BI_ISPOISONED,
    BI_ISWEAK, BI_KING, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP,
};
use crate::headers::list_mon_race_flags_h::{RF_KILL_WALL, RF_PASS_WALL};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{FEAT_LAVA, FEAT_LESS, FEAT_NONE};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;

/// A literal key (`borg_keypress('y')` etc.).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `double_distance(Y1, X1, Y2, X2)`: "twice" the distance between two
/// points, so diagonals look further than adjacent grids.
fn double_distance(y1: i32, x1: i32, y2: i32, x2: i32) -> i32 {
    borg_distance(y1 << 1, x1 << 1, y2 << 1, x2 << 1)
}

/// `rand_spread(a, d)` (z-rand.h): `a + randint0(1 + d + d) - d`.
fn rand_spread(a: i32, d: i32) -> i32 {
    a.wrapping_add(engine::randint0((1 + d + d) as u32)).wrapping_sub(d)
}

/// "Reset timer if borg was in a anti-summon corridor".
fn reset_antisummon(st: &mut BorgState) {
    if borg_timer(st, st.borg.time.antisummon) < 50 {
        st.borg.time.antisummon = 0;
    }
}

/// `borg.escapes--` ("a phase isn't really an escape"), `int16_t`.
fn phase_not_escape(st: &mut BorgState) {
    st.borg.escapes = st.borg.escapes.wrapping_sub(1);
}

/// Attempt to induce WORD_OF_RECALL
/// artifact activations added throughout this code
pub fn borg_recall(st: &mut BorgState) -> bool {
    // Multiple "recall" fails
    if st.borg.goal.recalling == 0 {
        // Try to "recall"
        if borg_zap_rod(st, st.item_val.sv_rod_recall)
            || borg_activate_item(st, st.act.recall)
            || borg_spell_fail(st, WORD_OF_RECALL, 60)
            || borg_read_scroll(st, st.item_val.sv_scroll_word_of_recall)
        {
            // do we need to answer "Set recall depth to current depth? [y/n]"
            if st.borg.tr(BI_CDEPTH) < st.borg.tr(BI_MAXDEPTH) && st.borg.tr(BI_CDEPTH) != 0 {
                let diff_max_from_desired_depth =
                    st.borg.tr(BI_MAXDEPTH) - st.prepared.depth_hunted_unique;
                let diff_cur_from_desired_depth =
                    st.borg.tr(BI_CDEPTH) - st.prepared.depth_hunted_unique;

                // Reset max depth when current depth is way deeper (5) than
                // the third deepest unique.
                if diff_max_from_desired_depth > 5
                    && diff_cur_from_desired_depth <= 5
                    && diff_cur_from_desired_depth > -5
                {
                    if st.borg.tr(BI_CDEPTH) >= 80
                        && st.borg.tr(BI_CDEPTH) < 100
                        // Deep, and Sauron is dead
                        && st.kills.race_death[st.borg.id.sauron as usize] != 0
                    {
                        // Do reset Depth
                        borg_note(st, "# Resetting recall depth.");
                        borg_keypress(st, key(b'y'));
                    } else if st.borg.goal.fleeing_munchkin {
                        // Do reset Depth
                        borg_note(st, "# Resetting recall depth during munchkin mode.");
                        borg_keypress(st, key(b'y'));
                    } else if st.borg.tr(BI_CDEPTH) >= 100 && st.borg.tr(BI_KING) == 0 {
                        // Do not reset Depth
                        borg_note(st, "# Not Resetting recall depth.");
                        borg_keypress(st, key(b'n'));
                    } else if st.borg.tr(BI_MAXDEPTH) == 99 && st.prepared.numb_live_unique < 5 {
                        // Do reset Depth, cleaning up last uniques to hit
                        // Morgoth.
                        borg_note(st, "# Resetting recall depth.");
                        borg_keypress(st, key(b'y'));
                    } else {
                        // Do reset Depth
                        borg_note(st, "# Resetting recall depth.");
                        borg_keypress(st, key(b'y'));
                    }
                } else {
                    // reset recall depth in dungeon?
                    // Do not reset Depth
                    borg_note(st, "# Not resetting recall depth.");
                    borg_keypress(st, key(b'n'));
                }
            }

            borg_keypress(st, ESCAPE);

            // Success
            return true;
        }
    }

    // Nothing
    false
}

/// Evaluate the likelihood of the borg getting surrounded
/// by a bunch of monsters.  This is called from borg_danger() when
/// he looking for a strategic retreat.  It is hopeful that the borg
/// will see that several monsters are approaching him and he may
/// become surrounded then die.  This routine looks at near-by monsters
/// and determines the likelihood of him getting surrounded.
pub fn borg_surrounded(st: &mut BorgState) -> bool {
    let mut safe_grids = 8;
    let mut non_safe_grids = 0;
    let mut monsters = 0;
    let mut adjacent_monsters = 0;

    let r_max = i32::from(engine::z_info().r_max);

    // Evaluate the local monsters
    for i in 1..i32::from(st.kills.nxt) {
        let kill = &st.kills.kills[i as usize];
        let r_idx = kill.r_idx;
        let awake = kill.awake;
        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        // Skip dead monsters
        if r_idx == 0 {
            continue;
        }

        // "player ghosts"
        if i32::from(r_idx) >= r_max - 1 {
            continue;
        }

        // Distance components
        let c = st.borg.c;
        let ax = if x9 > c.x { x9 - c.x } else { c.x - x9 };
        let ay = if y9 > c.y { y9 - c.y } else { c.y - y9 };

        // Distance
        let d = ax.max(ay);

        // if the monster is too far then skip it.
        if d > 3 {
            continue;
        }

        // if he cant see me then forget it.
        if !borg_los(st, c.y, c.x, y9, x9) {
            continue;
        }

        // if asleep, don't consider this one
        if !awake {
            continue;
        }

        // Monsters with Pass Wall are dangerous, no escape from them
        if engine::race_flag_has(usize::from(r_idx), RF_PASS_WALL) {
            continue;
        }
        if engine::race_flag_has(usize::from(r_idx), RF_KILL_WALL) {
            continue;
        }

        // keep track of monsters touching me
        if d == 1 {
            adjacent_monsters += 1;
        }

        // Add them up.
        monsters += 1;
    }

    // Evaluate the Non Safe Grids, (walls, closed doors, traps, monsters)
    for i in 0..8 {
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

        // Bound check
        if !engine::square_in_bounds_fully(x, y) {
            continue;
        }

        // Access the grid
        let ag = *st.cave.grid(x, y);

        if !borg_cave_floor_grid(&ag) {
            // Skip walls/doors
            non_safe_grids += 1;
        } else if u32::from(ag.feat) == FEAT_NONE {
            // Skip unknown grids
            non_safe_grids += 1;
        } else if ag.kill != 0 {
            // Skip monster grids
            non_safe_grids += 1;
        } else if engine::feat_is_shop(i32::from(ag.feat)) {
            // Mega-Hack -- skip stores XXX XXX XXX
            non_safe_grids += 1;
        } else if ag.trap && !ag.glyph {
            // Mega-Hack -- skip traps XXX XXX XXX
            // (part of the same chain: a trapped wall is one non-safe
            // grid, not two)
            non_safe_grids += 1;
        }
    }

    // Safe grids are decreased
    safe_grids -= non_safe_grids;

    // Am I in hallway? If so don't worry about it
    if safe_grids == 1 && adjacent_monsters == 1 {
        return false;
    }

    // I am likely to get surrounded
    if monsters > safe_grids {
        borg_note(
            st,
            &format!(
                "# Possibility of being surrounded (monsters/safegrids)({monsters}/{safe_grids})"
            ),
        );

        // The borg can get trapped by continuing to flee into a dead-end.
        // So he needs to be able to trump this routine (`borg.goal.ignoring`
        // keeps going, the "ignore" note is commented out in C).
        if !st.borg.goal.ignoring {
            return true;
        }
    }

    // Probably will not be surrounded
    false
}

/// Mega-Hack -- evaluate the "freedom" of the given location
///
/// The theory is that often, two grids will have equal "danger",
/// but one will be "safer" than the other, perhaps because it
/// is closer to stairs, or because it is in a corridor, or has
/// some other characteristic that makes it "safer".
///
/// Then, if the Borg is in danger, say, from a normal speed monster
/// which is standing next to him, he will know that walking away from
/// the monster is "pointless", because the monster will follow him,
/// but if the resulting location is "safer" for some reason, then
/// he will consider it.  This will allow him to flee towards stairs
/// in the town, and perhaps towards corridors in the dungeon.
///
/// This method is used in town to chase the stairs.
///
/// XXX XXX XXX We should attempt to walk "around" buildings.
pub fn borg_freedom(st: &mut BorgState, y: i32, x: i32) -> i32 {
    let mut f = 0;

    // Chase down stairs in town
    if st.borg.tr(BI_CDEPTH) == 0 && st.stairs.more.num != 0 {
        // Love the stairs!
        let d = double_distance(y, x, st.stairs.more.y[0], st.stairs.more.x[0]);

        // Proximity is good
        f += 1000 - d;

        // Close proximity is great
        if d < 4 {
            f += 2000 - d * 500;
        }
    }

    // Chase Up Stairs in dungeon
    if st.borg.tr(BI_CDEPTH) != 0 && st.stairs.less.num != 0 {
        // Love the stairs!
        let d = double_distance(y, x, st.stairs.less.y[0], st.stairs.less.x[0]);

        // Proximity is good
        f += 1000 - d;

        // Close proximity is great
        if d < 4 {
            f += 2000 - d * 500;
        }
    }

    // Freedom
    f
}

/// The common part of `borg_caution_phase` and `borg_caution_teleport`:
/// simulate 100 jumps of range `dis` and count the "scary" landing grids.
///
/// `skip_unknown` is the unknown-grid test for a candidate grid. As in C,
/// the grid checked after the search (`ag`) is the last one the search
/// looked at (it starts as the Borg's own grid and carries over from one
/// attempt to the next).
fn count_scary(
    st: &mut BorgState,
    dis: i32,
    turns: i32,
    mut skip_unknown: impl FnMut(&mut BorgState, i32, i32) -> bool,
) -> i32 {
    let min = dis / 2;

    // `ag` of C, as a position
    let mut ag = (st.borg.c.x, st.borg.c.y);

    // Simulate 100 attempts
    let mut n = 0;
    for _k in 0..100 {
        let mut x = 0;
        let mut y = 0;

        // Pick a location
        let mut i = 0;
        while i < 100 {
            // Pick a (possibly illegal) location
            loop {
                y = rand_spread(st.borg.c.y, dis);
                x = rand_spread(st.borg.c.x, dis);
                let d = engine::distance(st.borg.c, loc { x, y });
                if d >= min && d <= dis {
                    break;
                }
            }

            // Ignore illegal locations
            if y <= 0 || y >= AUTO_MAX_Y - 1 || x <= 0 || x >= AUTO_MAX_X - 1 {
                i += 1;
                continue;
            }

            // Access
            ag = (x, y);
            let g = *st.cave.grid(x, y);

            // Skip unknown grids
            if skip_unknown(st, x, y) {
                i += 1;
                continue;
            }

            // Skip walls
            if !borg_cave_floor_bold(st, y, x) {
                i += 1;
                continue;
            }

            // Skip monsters
            if g.kill != 0 {
                i += 1;
                continue;
            }

            // Skip webs
            if g.web {
                i += 1;
                continue;
            }

            // Stop looking
            break;
        }

        // If low level, unknown squares are scary
        if u32::from(st.cave.grid(ag.0, ag.1).feat) == FEAT_NONE && st.borg.tr(BI_MAXHP) < 30 {
            n += 1;
            continue;
        }

        // No location
        // in the real code it would keep trying but here we should
        // assume that there is unknown spots that you would be able
        // to go but may be dangerous.
        if i >= 100 {
            n += 1;
            continue;
        }

        // Examine
        let p = borg_danger(st, y, x, turns, true, false);

        // if *very* scary, do not allow jumps at all
        if p > st.borg.tr(BI_CURHP) {
            n += 1;
        }
    }
    n
}

/// Help determine if PHASE_DOOR seems like a good idea
pub fn borg_caution_phase(st: &mut BorgState, emergency: i32, turns: i32) -> bool {
    // must have the ability
    if st.borg.tr(BI_APHASE) == 0 {
        return false;
    }

    // Skip unknown grids
    let n = count_scary(st, 10, turns, |st, x, y| {
        u32::from(st.cave.grid(x, y).feat) == FEAT_NONE
    });

    // Too much danger
    // in an emergency try with extra danger allowed
    if n > emergency {
        borg_note(st, &format!("# No Phase. scary squares: {n}"));
        return false;
    } else {
        borg_note(st, &format!("# Safe to Phase. scary squares: {n}"));
    }

    // Okay
    true
}

/// Help determine if "Teleport" seems like a good idea
pub fn borg_caution_teleport(st: &mut BorgState, emergency: i32, turns: i32) -> bool {
    // Extract panel
    let q_x = st.core.w_x / borg_panel_wid();
    let q_y = st.core.w_y / borg_panel_hgt();

    // must have the ability
    if st.borg.tr(BI_ATELEPORT) == 0 || st.borg.tr(BI_AESCAPE) == 0 {
        return false;
    }

    // Skip unknown grids if explored, or been on level for a while,
    // otherwise, consider ok
    let n = count_scary(st, 100, turns, |st, x, y| {
        u32::from(st.cave.grid(x, y).feat) == FEAT_NONE
            && ((detect_wall(st, q_y, q_x)
                && detect_wall(st, q_y, q_x + 1)
                && detect_wall(st, q_y + 1, q_x)
                && detect_wall(st, q_y + 1, q_x + 1))
                || borg_timer(st, st.borg.time.level) > 2000)
    });

    // Too much danger
    // in an emergency try with extra danger allowed
    if n > emergency {
        borg_note(st, &format!("# No Teleport. scary squares: {n}"));
        return false;
    }

    // Okay
    true
}

/// `borg_detect_wall[y][x] == true` (borg-update.c's detection map).
fn detect_wall(st: &mut BorgState, y: i32, x: i32) -> bool {
    st.update.detect(Detect::Wall, y, x)
}

/// If the borg is standing on a stair and is in some danger, just leave
/// the level. No need to hang around on that level, try conserving the
/// teleport scrolls
fn borg_escape_stair(st: &mut BorgState) -> bool {
    // Current grid
    let feat = st.cave.grid(st.borg.c.x, st.borg.c.y).feat;

    // Usable stairs
    if u32::from(feat) == FEAT_LESS && !engine::player_opt(OPT_birth_force_descend) {
        // Take the stairs
        borg_note(st, "# Escaping level via stairs.");
        borg_keypress(st, key(b'<'));

        // Success
        return true;
    }

    false
}

/// Is teleport currently allowed
pub fn borg_allow_teleport(st: &mut BorgState) -> bool {
    // No teleporting in arena levels
    if engine::player_arena_level() {
        return false;
    }

    // Check for a no teleport grid
    if engine::square_isno_teleport(st.borg.c.x, st.borg.c.y) {
        return false;
    }

    // Check for a no teleport curse
    if st.borg.tr(BI_CRSNOTEL) != 0 {
        return false;
    }

    true
}

/// short range teleport + pain
pub fn borg_shadow_shift(st: &mut BorgState, allow_fail: i32) -> bool {
    // disallow if hp too low
    if st.borg.tr(BI_CURHP) < 12 {
        return false;
    }
    borg_spell_fail(st, SHADOW_SHIFT, allow_fail)
}

/// medium range teleport
pub fn borg_dimension_door(st: &mut BorgState, allow_fail: i32) -> bool {
    // for now keep the range at under 50, for performance
    let range = 50;

    // Require ability (right now)
    if !borg_spell_okay_fail(st, DIMENSION_DOOR, allow_fail) {
        return false;
    }

    // if we are attacking, calculate gains, but if this is just a teleport
    // the current danger is the starting point
    let region = |st: &BorgState| {
        i32::from(
            st.danger.fear_region[(st.borg.c.y / 11) as usize][(st.borg.c.x / 11) as usize],
        )
    };
    let mut best_d = region(st);
    let mut best = loc { x: 0, y: 0 };

    // Pick a location
    for x_off in -range..range {
        for y_off in -range..range {
            let t_x = st.borg.c.x + x_off;
            let t_y = st.borg.c.y + y_off;

            if t_x < 0 || t_y < 0 {
                continue;
            }

            if !engine::square_in_bounds_fully(t_x, t_y) {
                continue;
            }

            /* Only aim at a known floor grid without a monster, as the
             * game moves the player elsewhere when the target is not a
             * legal landing grid (better-borg-c milestone8 patch 0086) */
            let ag = st.cave.grid(t_x, t_y);
            if u32::from(ag.feat) == FEAT_NONE
                || u32::from(ag.feat) == FEAT_LAVA
                || !borg_cave_floor_grid(ag)
                || ag.kill != 0
            {
                continue;
            }

            let d = borg_danger(st, t_y, t_x, 2, true, false);
            if d < best_d {
                best_d = d;
                best.x = t_x;
                best.y = t_y;
            }
        }
    }

    if best_d < region(st) {
        borg_target(st, best, false);

        borg_spell(st, DIMENSION_DOOR);

        // pick target
        borg_keypress(st, key(b'5'));
        return true;
    }
    false
}

fn borg_teleport_off_level(st: &mut BorgState) -> bool {
    // don't if we are already recalling or waiting for
    // deep descent
    if !(st.borg.goal.recalling != 0 || st.borg.goal.descending != 0) {
        // teleport level before deep descent because there is no delay
        if borg_read_scroll(st, st.item_val.sv_scroll_teleport_level)
            || borg_activate_item(st, st.act.tele_level)
        {
            return true;
        }

        if borg_activate_item(st, st.act.deep_descent)
            || borg_read_scroll(st, st.item_val.sv_scroll_deep_descent)
        {
            return true;
        }
    }

    false
}

/// `borg.avoidance * (tenths + risky_boost) / 10`
fn avoid(st: &BorgState, tenths: i32, risky_boost: i32) -> i32 {
    i32::from(st.borg.avoidance) * (tenths + risky_boost) / 10
}

/// `borg.near.unique >= 1 && borg.near.unique <= 8`
fn near_unique_1_8(st: &BorgState) -> bool {
    st.borg.near.unique >= 1 && st.borg.near.unique <= 8
}

/// Try to phase door or teleport
/// b_q is the danger of the least dangerous square around us.
pub fn borg_escape(st: &mut BorgState, b_q: i32) -> bool {
    let mut risky_boost = 0;

    // only escape with spell if fail is low
    let mut allow_fail = 25;

    // if very healthy, allow extra fail
    if st.borg.tr(BI_CURHP) * 100 / st.borg.tr(BI_MAXHP) > 70 {
        allow_fail = 10;
    }

    // comprimised, get out of the fight
    if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
        allow_fail = 35;
    }

    // for emergencies
    let sv_mana = st.borg.tr(BI_CURSP);

    // Borgs who are bleeding to death or dying of poison may sometimes
    // phase around the last two hit points right before they enter a
    // shop.  He knows to make a bee-line for the temple but the danger
    // trips this routine.  So we must bypass this routine for some
    // particular circumstances.
    if st.borg.tr(BI_CDEPTH) == 0
        && (st.borg.tr(BI_ISPOISONED) != 0
            || st.borg.tr(BI_ISWEAK) != 0
            || st.borg.tr(BI_ISCUT) != 0)
    {
        return false;
    }

    // Borgs who are in a sea of runes or trying to build one
    // and mostly healthy stay put
    if st.borg.tr(BI_CDEPTH) == 100 && st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) * 5 / 10 {
        // In a sea of runes
        if st.borg.morgoth_position {
            return false;
        }

        // Scan neighbors
        let mut glyphs = 0;
        for j in 0..8 {
            let y = st.borg.c.y + i32::from(engine::ddy_ddd(j));
            let x = st.borg.c.x + i32::from(engine::ddx_ddd(j));

            // Skip unknown grids (important)
            if st.cave.grid(x, y).glyph {
                glyphs += 1;
            }
        }
        // Touching at least 3 glyphs
        if glyphs >= 3 {
            return false;
        }
    }

    // If the borg is weak (no food, starving) on depth 1 and he has no
    // idea where the stairs may be, run the risk of diving deeper against
    // the benefit of rising to town.
    if st.borg.tr(BI_ISWEAK) != 0
        && st.borg.tr(BI_CDEPTH) == 1
        && borg_teleport_off_level(st)
    {
        borg_note(st, "# Attempting to leave via teleport level/deep descent");
        return true;
    }

    // Risky borgs are more likely to stay in a fight
    if st.cfg(BORG_PLAYS_RISKY) != 0 {
        risky_boost = 3;
    }

    let rb = risky_boost;

    // 1. really scary, I'm about to die
    // Try an emergency teleport, or phase door as last resort
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        || b_q > avoid(st, 45, rb)
        || (b_q > avoid(st, 40, rb)
            && st.borg.near.unique >= 10
            && st.borg.tr(BI_CDEPTH) == 100
            && st.borg.tr(BI_CURHP) < 600)
        || (b_q > avoid(st, 30, rb)
            && st.borg.near.unique >= 10
            && st.borg.tr(BI_CDEPTH) == 99
            && st.borg.tr(BI_CURHP) < 600)
        || (b_q > avoid(st, 25, rb)
            && near_unique_1_8(st)
            && st.borg.tr(BI_CDEPTH) >= 95
            && st.borg.tr(BI_CURHP) < 550)
        || (b_q > avoid(st, 17, rb) && near_unique_1_8(st) && st.borg.tr(BI_CDEPTH) < 95)
        || (b_q > avoid(st, 15, rb) && st.borg.near.unique == 0)
    {
        let tmp_allow_fail = 15;

        if borg_escape_stair(st)
            || (borg_allow_teleport(st)
                && (borg_dimension_door(st, tmp_allow_fail - 10)
                    || borg_spell_fail(st, TELEPORT_SELF, tmp_allow_fail - 10)
                    || borg_spell_fail(st, PORTAL, tmp_allow_fail - 10)
                    || borg_shadow_shift(st, tmp_allow_fail - 10)
                    || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                    || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation)
                    || borg_activate_item(st, st.act.tele_long)
                    || borg_teleport_off_level(st)
                    // revisit spells, increased fail rate
                    || borg_dimension_door(st, tmp_allow_fail + 9)
                    || borg_spell_fail(st, TELEPORT_SELF, tmp_allow_fail + 9)
                    || borg_spell_fail(st, PORTAL, tmp_allow_fail + 9)
                    || borg_shadow_shift(st, tmp_allow_fail + 9)
                    // revisit teleport, increased fail rate
                    || borg_use_staff(st, st.item_val.sv_staff_teleportation)
                    // Attempt Teleport Level
                    || borg_spell_fail(st, TELEPORT_LEVEL, tmp_allow_fail + 9)
                    // try phase at least, with some hedging of the safety of
                    // landing zone
                    || (borg_caution_phase(st, 75, 2)
                        && (borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                            || borg_activate_item(st, st.act.tele_phase)
                            || borg_spell_fail(st, PHASE_DOOR, tmp_allow_fail)
                            || borg_spell_fail(st, PORTAL, tmp_allow_fail)))))
        {
            // Flee!
            borg_note(st, "# Danger Level 1.");
            reset_antisummon(st);
            return true;
        }

        // Pretend to have full mana for the last-ditch casts (C bug 35:
        // not restored when one of them succeeds).
        st.borg.trait_[BI_CURSP as usize] = st.borg.tr(BI_MAXSP);

        // try to teleport, get far away from here
        if st.borg.tr(BI_CDEPTH) != 0
            && st.borg.tr(BI_CLEVEL) < 10
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 10
            && borg_allow_teleport(st)
            && (borg_dimension_door(st, 90)
                || borg_spell(st, TELEPORT_SELF)
                || borg_spell(st, PORTAL))
        {
            // Flee!
            borg_note(st, "# Danger Level 1.1  Critical Attempt");
            reset_antisummon(st);
            return true;
        }

        // emergency phase activation no concern for safety of landing zone.
        if st.borg.tr(BI_CDEPTH) != 0
            && (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 10 || b_q > avoid(st, 45, rb))
            && (borg_activate_item(st, st.act.tele_phase)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 1.2  Critical Phase");
            reset_antisummon(st);
            return true;
        }

        // emergency phase spell
        if st.borg.tr(BI_CDEPTH) != 0
            && st.borg.tr(BI_CLEVEL) < 10
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 10
            && (borg_spell_fail(st, PHASE_DOOR, 15) || borg_spell(st, PORTAL))
        {
            // Flee!
            borg_note(st, "# Danger Level 1.3  Critical Attempt");
            reset_antisummon(st);
            return true;
        }

        // Restore the real mana level
        st.borg.trait_[BI_CURSP as usize] = sv_mana;
    }

    // If fighting a unique and at the end of the game try to stay and
    // finish the fight.  Only bail out in extreme danger as above.
    if b_q < avoid(st, 25, rb)
        && st.borg.near.unique >= 1
        && st.borg.near.unique <= 3
        && st.borg.tr(BI_CDEPTH) >= 97
    {
        return false;
    }

    // 2 - a bit more scary/
    // Attempt to teleport (usually)
    // do not escape from uniques so quick
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        || (b_q > avoid(st, 3, rb)
            && st.borg.tr(BI_CLASS) == CLASS_MAGE
            && st.borg.tr(BI_CURSP) <= 20
            && st.borg.tr(BI_MAXCLEVEL) >= 45)
        || (b_q > avoid(st, 13, rb) && near_unique_1_8(st) && st.borg.tr(BI_CDEPTH) != 99)
        || (b_q > avoid(st, 11, rb) && st.borg.near.unique == 0)
    {
        // Try teleportation
        if borg_escape_stair(st)
            || (borg_allow_teleport(st)
                && (borg_dimension_door(st, allow_fail - 10)
                    || borg_spell_fail(st, TELEPORT_SELF, allow_fail - 10)
                    || borg_spell_fail(st, PORTAL, allow_fail - 10)
                    || borg_shadow_shift(st, allow_fail - 10)
                    || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation)
                    || borg_activate_item(st, st.act.tele_long)
                    || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                    || borg_dimension_door(st, allow_fail)
                    || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_shadow_shift(st, allow_fail)
                    || borg_use_staff(st, st.item_val.sv_staff_teleportation)))
        {
            // Flee!
            borg_note(st, "# Danger Level 2.1");

            // Success
            reset_antisummon(st);
            return true;
        }
        // Phase door, if useful
        if borg_caution_phase(st, 50, 2)
            && borg_timer(st, st.borg.time.antisummon) > 50
            && (borg_spell(st, PHASE_DOOR)
                || borg_spell(st, PORTAL)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || borg_activate_item(st, st.act.tele_phase))
        {
            // Flee!
            borg_note(st, "# Danger Level 2.2");
            reset_antisummon(st);
            // Success
            return true;
        }
    }

    // 3- not too bad
    // also run if stunned or it is scary here
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        || (b_q > avoid(st, 13, rb) && st.borg.near.unique >= 2 && st.borg.near.unique <= 8)
        || (b_q > avoid(st, 10, rb) && st.borg.near.unique == 0)
        || (b_q > avoid(st, 10, rb)
            && st.borg.tr(BI_ISAFRAID) != 0
            && (st.borg.tr(BI_AMISSILES) <= 0 && st.borg.tr(BI_CLASS) == CLASS_WARRIOR))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (borg_caution_phase(st, 25, 2)
                && borg_timer(st, st.borg.time.antisummon) > 50
                && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 3.1");
            reset_antisummon(st);

            // Success
            return true;
        }

        // Teleport via spell
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, allow_fail)
                || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_shadow_shift(st, allow_fail)
                || borg_activate_item(st, st.act.tele_long)
                || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation)
                || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                || borg_activate_item(st, st.act.tele_phase))
        {
            // Flee!
            borg_note(st, "# Danger Level 3.2");
            reset_antisummon(st);

            // Success
            return true;
        }
        // Phase door, if useful
        if borg_caution_phase(st, 75, 2)
            && borg_timer(st, st.borg.time.antisummon) > 50
            && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_shadow_shift(st, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 3.3");
            reset_antisummon(st);

            // Success
            return true;
        }

        // Use Tport Level after the above attempts failed.
        if borg_teleport_off_level(st) {
            // Flee!
            borg_note(st, "# Danger Level 3.4");
            reset_antisummon(st);

            // Success
            return true;
        }

        // if we got this far we tried to escape but couldn't...
        // time to flee
        if !st.borg.goal.fleeing
            && (st.borg.near.unique == 0 || st.borg.tr(BI_CLEVEL) < 35)
            && !st.borg.status.vault
        {
            // Note
            borg_note(st, "# Fleeing (failed to teleport)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }

        // Flee now
        if !st.borg.goal.leaving
            && (st.borg.near.unique == 0 || st.borg.tr(BI_CLEVEL) < 35)
            && !st.borg.status.vault
        {
            // Flee!
            borg_note(st, "# Leaving (failed to teleport)");

            // Start leaving
            st.borg.goal.leaving = true;
        }
    }

    // 4- not too scary but I'm compromised
    if (b_q > avoid(st, 8, rb)
        && (st.borg.tr(BI_CLEVEL) < 35 || st.borg.tr(BI_CURHP) <= st.borg.tr(BI_MAXHP) / 3))
        || (b_q > avoid(st, 9, rb)
            && near_unique_1_8(st)
            && (st.borg.tr(BI_CLEVEL) < 35 || st.borg.tr(BI_CURHP) <= st.borg.tr(BI_MAXHP) / 3))
        || (b_q > avoid(st, 6, rb) && st.borg.tr(BI_CLEVEL) <= 20 && st.borg.near.unique == 0)
        || (b_q > avoid(st, 6, rb) && st.borg.tr(BI_CLEVEL) <= 35)
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (borg_caution_phase(st, 20, 2)
                && borg_timer(st, st.borg.time.antisummon) > 50
                && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_shadow_shift(st, allow_fail)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 4.1");
            reset_antisummon(st);

            // Success
            return true;
        }

        // Teleport via spell
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, allow_fail)
                || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_activate_item(st, st.act.tele_long)
                || borg_shadow_shift(st, allow_fail)
                || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation))
        {
            // Flee!
            borg_note(st, "# Danger Level 4.2");
            reset_antisummon(st);

            // Success
            return true;
        }

        // if we got this far we tried to escape but couldn't...
        // time to flee
        if !st.borg.goal.fleeing
            && st.borg.near.unique == 0
            && st.borg.tr(BI_CLEVEL) < 25
            && !st.borg.status.vault
        {
            // Note
            borg_note(st, "# Fleeing (failed to teleport)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }

        // Flee now
        if !st.borg.goal.leaving && st.borg.near.unique == 0 && !st.borg.status.vault {
            // Flee!
            borg_note(st, "# Leaving (failed to teleport)");

            // Start leaving
            st.borg.goal.leaving = true;
        }
        // Emergency Phase door if a weak mage
        if ((st.borg.tr(BI_CLASS) == CLASS_MAGE || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER)
            && st.borg.tr(BI_CLEVEL) <= 35)
            && borg_caution_phase(st, 65, 2)
            && borg_timer(st, st.borg.time.antisummon) > 50
            && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_activate_item(st, st.act.tele_long)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 4.3");
            reset_antisummon(st);

            // Success
            return true;
        }
    }

    // 5- not too scary but I'm very low level
    if st.borg.tr(BI_CLEVEL) < 10
        && (b_q > avoid(st, 5, rb) || (b_q > avoid(st, 7, rb) && near_unique_1_8(st)))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (borg_caution_phase(st, 20, 2)
                && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_shadow_shift(st, allow_fail)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)))
        {
            // Flee!
            borg_note(st, "# Danger Level 5.1");
            reset_antisummon(st);

            // Success
            return true;
        }

        // Teleport via spell
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, allow_fail)
                || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_shadow_shift(st, allow_fail)
                || borg_activate_item(st, st.act.tele_long)
                || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation))
        {
            // Flee!
            borg_note(st, "# Danger Level 5.2");
            reset_antisummon(st);

            // Success
            return true;
        }

        // if we got this far we tried to escape but couldn't...
        // time to flee
        if !st.borg.goal.fleeing && st.borg.near.unique == 0 {
            // Note
            borg_note(st, "# Fleeing (failed to teleport)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }

        // Flee now
        if !st.borg.goal.leaving && st.borg.near.unique == 0 {
            // Flee!
            borg_note(st, "# Leaving (failed to teleport)");

            // Start leaving
            st.borg.goal.leaving = true;
        }
        // Emergency Phase door if a weak mage
        if ((st.borg.tr(BI_CLASS) == CLASS_MAGE || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER)
            && st.borg.tr(BI_CLEVEL) <= 8)
            && borg_caution_phase(st, 65, 2)
            && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || borg_activate_item(st, st.act.tele_long))
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 5.3");
            reset_antisummon(st);

            // Success
            return true;
        }
    }

    // 6- not too scary but I'm out of mana
    if (st.borg.tr(BI_CLASS) == CLASS_MAGE
        || st.borg.tr(BI_CLASS) == CLASS_PRIEST
        || st.borg.tr(BI_CLASS) == CLASS_NECROMANCER)
        && (b_q > avoid(st, 6, rb) || (b_q > avoid(st, 8, rb) && near_unique_1_8(st)))
        && (st.borg.tr(BI_CURSP) <= st.borg.tr(BI_MAXSP) / 10 && st.borg.tr(BI_MAXSP) >= 100)
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (borg_caution_phase(st, 20, 2)
                && borg_timer(st, st.borg.time.antisummon) > 50
                && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)))
        {
            // Flee!
            borg_note(st, "# Danger Level 6.1");
            reset_antisummon(st);

            // Success
            return true;
        }

        // Teleport via spell
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, allow_fail)
                || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_activate_item(st, st.act.tele_long)
                || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation))
        {
            // Flee!
            borg_note(st, "# Danger Level 6.2");
            reset_antisummon(st);

            // Success
            return true;
        }
    }

    // 7- Shoot N Scoot
    if (borg_spell_okay_fail(st, PHASE_DOOR, allow_fail)
        || borg_spell_okay_fail(st, PORTAL, allow_fail))
        && borg_shoot_scoot_safe(st, 20, 2, b_q)
        // Phase door
        && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
            || borg_spell_fail(st, PORTAL, allow_fail))
    {
        // Flee!
        borg_note(st, "# Shoot N Scoot. (Danger Level 7.1)");
        phase_not_escape(st);
        reset_antisummon(st);

        // Success
        return true;
    }

    // 8- not too scary but twitching
    if st.borg.times_twitch > 50 {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (borg_caution_phase(st, 20, 2)
                && borg_timer(st, st.borg.time.antisummon) > 50
                && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)))
        {
            // Flee!
            borg_note(st, "# Danger Level 8");
            reset_antisummon(st);

            // no longer twitchy
            st.borg.times_twitch = 0;

            // Success
            return true;
        }

        // Teleport via spell
        if borg_allow_teleport(st)
            && (borg_dimension_door(st, allow_fail)
                || borg_spell_fail(st, TELEPORT_SELF, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_activate_item(st, st.act.tele_long)
                || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation)
                || borg_teleport_off_level(st))
        {
            // Flee!
            borg_note(st, "# Danger Level 8");
            reset_antisummon(st);

            // no longer twitchy
            st.borg.times_twitch = 0;

            // Success
            return true;
        }
    }

    false
}

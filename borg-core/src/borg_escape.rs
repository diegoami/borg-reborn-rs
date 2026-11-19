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

// borg-reborn-rs (#220): named danger/escape policy values; defaults preserve arithmetic.
use crate::params::{value as param, Param};
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
use crate::borg_cave_view::BORG_VIEW;
use crate::engine;
use crate::borg_headers::borg_h::BORG_PLAYS_RISKY;
use crate::borg_headers::borg_magic_h::{
    DIMENSION_DOOR, FOX_FORM, PHASE_DOOR, PORTAL, SHADOW_SHIFT, TELEPORT_LEVEL, TELEPORT_SELF,
    WORD_OF_RECALL,
};
/// `GOAL_FLEE` (borg-flow.h): flee the level.
const GOAL_FLEE: i32 = 7;
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
    if borg_timer(st, st.borg.time.antisummon) < param(Param::escape_antisummon_reset_turns) {
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
            || borg_spell_fail(st, WORD_OF_RECALL, param(Param::escape_recall_fail))
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
                if diff_max_from_desired_depth > param(Param::escape_recall_depth_window)
                    && diff_cur_from_desired_depth <= param(Param::escape_recall_depth_window)
                    && diff_cur_from_desired_depth > -param(Param::escape_recall_depth_window)
                {
                    if st.borg.tr(BI_CDEPTH) >= param(Param::escape_recall_reset_min_depth)
                        && st.borg.tr(BI_CDEPTH) < 100
                        // Deep, and Sauron is dead
                        && st.kills.race_death[st.borg.id.sauron as usize] != 0
                    {
                        // Do reset Depth
                        borg_note(st, "# Resetting recall depth.");
                        borg_keypress(st, key(b'y'));
                    } else if st.borg.tr(BI_CDEPTH) >= 100 && st.borg.tr(BI_KING) == 0 {
                        // Do not reset Depth
                        borg_note(st, "# Not Resetting recall depth.");
                        borg_keypress(st, key(b'n'));
                    } else if st.borg.tr(BI_MAXDEPTH) == 99 && st.prepared.numb_live_unique < param(Param::escape_recall_reset_unique_count) {
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
        if d > param(Param::escape_surround_distance) {
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
    if safe_grids == param(Param::escape_surround_single_exit_count) && adjacent_monsters == param(Param::escape_surround_single_monster_count) {
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
        f += param(Param::escape_town_stair_value) - d;

        // Close proximity is great
        if d < param(Param::escape_town_stair_close_distance) {
            f += param(Param::escape_town_stair_close_value) - d * param(Param::escape_town_stair_distance_penalty);
        }
    }

    // Chase Up Stairs in dungeon
    if st.borg.tr(BI_CDEPTH) != 0 && st.stairs.less.num != 0 {
        // Love the stairs!
        let d = double_distance(y, x, st.stairs.less.y[0], st.stairs.less.x[0]);

        // Proximity is good
        f += param(Param::escape_dungeon_stair_value) - d;

        // Close proximity is great
        if d < param(Param::escape_dungeon_stair_close_distance) {
            f += param(Param::escape_dungeon_stair_close_value) - d * param(Param::escape_dungeon_stair_distance_penalty);
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
    for _k in 0..param(Param::escape_landing_samples) {
        let mut x = 0;
        let mut y = 0;

        // Pick a location
        let mut i = 0;
        while i < param(Param::escape_landing_trials) {
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
        if u32::from(st.cave.grid(ag.0, ag.1).feat) == FEAT_NONE && st.borg.tr(BI_MAXHP) < param(Param::escape_unknown_landing_max_hp) {
            n += 1;
            continue;
        }

        // No location
        // in the real code it would keep trying but here we should
        // assume that there is unknown spots that you would be able
        // to go but may be dangerous.
        if i >= param(Param::escape_landing_trials) {
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

/// borg-reborn-rs (#539, owner): "do not phase door when a monster is still
/// distant". With `escape_phase_adjacent_only`, a Phase Door is pointless
/// unless a known awake monster is next to the Borg, or one in view can
/// hurt it from where it is (spells, breath, missiles): phasing away from a
/// monster that has yet to arrive only spends the scroll (Grip, speed +10,
/// found the Priest 809881907 again within a turn).
pub(crate) fn phase_is_pointless(st: &mut BorgState) -> bool {
    if param(Param::escape_phase_adjacent_only) == 0 {
        // #539: the ranged-attack rule stands on its own switch.
        return crate::borg_caution::ranged_before_recall(st) && !adjacent_awake(st);
    }
    let mut monsters = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        monsters.push((kill.awake, engine::distance(st.borg.c, kill.pos), kill.ranged_attack != 0, st.cave.grid(kill.pos.x, kill.pos.y).info & BORG_VIEW != 0));
    }
    // #539 (owner): with no stairs, the best ranged attack comes before a
    // Phase Door too, unless a monster is already next to the Borg.
    let adjacent = monsters.iter().any(|&(awake, d, _, _)| awake && d <= 1);
    !phase_helps(&monsters) || (!adjacent && crate::borg_caution::ranged_before_recall(st))
}

/// An awake known monster next to the Borg.
fn adjacent_awake(st: &BorgState) -> bool {
    (1..st.kills.nxt.max(0) as usize).any(|i| {
        let kill = &st.kills.kills[i];
        kill.r_idx != 0 && kill.awake && engine::distance(st.borg.c, kill.pos) <= 1
    })
}

/// The known monsters as (awake, distance, ranged attack, in view): a phase
/// helps against an awake adjacent monster or an awake one in view with a
/// ranged attack.
fn phase_helps(monsters: &[(bool, i32, bool, bool)]) -> bool {
    monsters.iter().any(|&(awake, d, ranged, view)| awake && (d <= 1 || (ranged && view)))
}

/// borg-reborn-rs (#551): `escape_fox_form` is on.
pub(crate) fn fox_form_on() -> bool {
    param(Param::escape_fox_form) != 0
}

/// borg-reborn-rs (#551): the player is in fox shape.
pub(crate) fn is_fox() -> bool {
    engine::player_is_shapechanged() && engine::player_shape_name() == b"fox"
}

/// borg-reborn-rs (#551): keep the fox shape this think: the switch is on,
/// the player is a fox, in the dungeon (no shop takes a fox, cmd-cave.c),
/// and still fleeing (a flight flow, or the flee or leave goal) or on a
/// scary level, standing included: #530's stand clears the flee goal for a
/// think, and reverting then recast the shape at the next flight (110 casts
/// in one game). Otherwise `borg_think` reverts as C does.
pub(crate) fn keep_fox_form(st: &BorgState) -> bool {
    let flight = i32::from(st.borg.goal.type_0) == GOAL_FLEE || st.borg.goal.fleeing || st.borg.goal.leaving;
    let scary = st.borg.mon.scary || st.borg.mon.scary_stand;
    // The game's depth, not the Borg's last update: right after a Recall
    // to town the trait still holds the old depth (review question 3).
    fox_form_on() && is_fox() && keep_fox_when(engine::player_depth(), flight, scary)
}

fn keep_fox_when(depth: i32, flight: bool, scary_level: bool) -> bool {
    depth > 0 && (flight || scary_level)
}

/// borg-reborn-rs (#551): the Druid is about to take a flight step with
/// `steps_left` flow steps to go: cast Fox Form first (one turn; the next
/// steps cost half the energy, `MOVES[1]`, player-util.c `energy_per_move`).
/// The manual lists it as the CL1-20 escape (14-class-playbooks.md:453).
pub(crate) fn fox_form_before_flight_step(st: &mut BorgState, steps_left: i32) -> bool {
    let fail = param(Param::escape_fox_form_fail);
    if !cast_fox_when(fox_form_on(), st.borg.tr(BI_CDEPTH), engine::player_is_shapechanged(), steps_left, param(Param::escape_fox_form_min_steps))
        || !borg_spell_okay_fail(st, FOX_FORM, fail)
    {
        return false;
    }
    borg_note(st, &format!("# Fox Form: fleeing ({steps_left} steps to go)"));
    borg_spell_fail(st, FOX_FORM, fail)
}

fn cast_fox_when(on: bool, depth: i32, shapechanged: bool, steps_left: i32, min_steps: i32) -> bool {
    on && depth > 0 && !shapechanged && steps_left >= min_steps
}

/// borg-reborn-rs (#551): the answer to the game's "Change back and <verb>
/// (y/n) or (r)eturn to normal?" (`player_get_resume_normal_shape`): `r`
/// for the Borg's own change back (`fox_revert`, queued as `v` alone), `y`
/// for an item or a spell command (change back and act, one command, no
/// extra energy).
pub(crate) fn change_back_answer(st: &mut BorgState) -> u8 {
    if std::mem::take(&mut st.borg.fox_revert) { b'r' } else { b'y' }
}

/// borg-reborn-rs (#551): queue the Borg's own change back from fox shape:
/// `v` (a command that asks), answered `r` by the prompt handler. Changing
/// back uses no energy (`player_resume_normal_shape`).
pub(crate) fn queue_fox_revert(st: &mut BorgState) {
    st.borg.fox_revert = true;
    borg_keypress(st, u32::from(b'v'));
}

/// borg-reborn-rs (#551, owner): a fox does not fight. Before a melee blow
/// (an attack or a step into a monster), the Borg changes back; it fights
/// in its own shape from the next think. Other attacks are item or spell
/// commands, which change back through the game's prompt.
pub(crate) fn revert_fox_to_fight(st: &mut BorgState) -> bool {
    if !(fox_form_on() && is_fox()) {
        return false;
    }
    borg_note(st, "# Fox Form: reverting to fight");
    queue_fox_revert(st);
    true
}

/// borg-reborn-rs (#551): the speed a flight is worth to the stand
/// decision (`MOVES[1]` halves a step's energy: +10 at normal speed,
/// `extract_energy` 10 to 20). A fox moves at `escape_fox_form_speed`
/// more. A Druid that can cast Fox Form, and will at the first flight step
/// at least `escape_fox_form_min_steps` from its goal, counts
/// `escape_fox_form_stand_speed` more (review of #594: granted on the
/// ability, so a parameter of its own, measured apart). More speed means
/// fewer pursuers, so fewer stands and more flights.
pub(crate) fn flight_speed_bonus(st: &BorgState) -> i32 {
    if !fox_form_on() || engine::player_depth() == 0 {
        return 0;
    }
    if is_fox() {
        param(Param::escape_fox_form_speed)
    } else if borg_spell_okay_fail(st, FOX_FORM, param(Param::escape_fox_form_fail)) {
        param(Param::escape_fox_form_stand_speed)
    } else {
        0
    }
}

/// borg-reborn-rs (#551): the speed bonus of a fox for `borg_danger`. A fox
/// only moves (it changes back to fight or to use an item or a spell), so
/// the monsters get half as many turns per step.
pub(crate) fn fox_moving_speed_bonus() -> i32 {
    if fox_form_on() && is_fox() {
        param(Param::escape_fox_form_speed)
    } else {
        0
    }
}

#[cfg(test)]
mod fox_form_tests {
    use super::{cast_fox_when, keep_fox_when};

    #[test]
    fn a_fox_form_is_cast_only_on_a_long_enough_flight_in_the_dungeon() {
        assert!(cast_fox_when(true, 3, false, 3, 3));
        assert!(!cast_fox_when(false, 3, false, 10, 3), "switch off");
        assert!(!cast_fox_when(true, 0, false, 10, 3), "town");
        assert!(!cast_fox_when(true, 3, true, 10, 3), "already shapechanged");
        assert!(!cast_fox_when(true, 3, false, 2, 3), "too close to the goal");
    }

    #[test]
    fn c_the_prompt_is_answered_r_only_for_the_borgs_own_change_back() {
        let mut st = crate::state::BorgState::default();
        assert_eq!(super::change_back_answer(&mut st), b'y', "an item or a spell: change back and act");
        st.borg.fox_revert = true;
        assert_eq!(super::change_back_answer(&mut st), b'r', "its own change back");
        assert!(!st.borg.fox_revert, "answered once");
        assert_eq!(super::change_back_answer(&mut st), b'y');
        // Review of #594: a flushed `v` takes its flag with it.
        st.borg.fox_revert = true;
        crate::borg_io::borg_flush(&mut st);
        assert_eq!(super::change_back_answer(&mut st), b'y', "a flushed change back answers no later prompt");
    }

    #[test]
    fn b_the_fox_shape_is_kept_while_fleeing_or_on_a_scary_level() {
        assert!(keep_fox_when(2, true, false), "fleeing");
        assert!(keep_fox_when(2, false, true), "a scary level, standing included");
        assert!(!keep_fox_when(2, false, false), "the flight is over, the level calm");
        assert!(!keep_fox_when(0, true, true), "town: no shop takes a fox");
    }
}

/// borg-reborn-rs (#584): every place where the Borg may read, cast or
/// activate Phase Door. Each asks `phase_door`, which writes one note naming
/// the site, so a game's messages say why every Phase Door happened.
/// docs/PHASE-DOOR.md lists each site's trigger and settings.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PhaseSite {
    /// `borg_escape` 1: danger far above the HP, after every teleport failed.
    Emergency,
    /// `borg_escape` 1.2: HP critical or danger extreme; no landing check.
    Critical,
    /// `borg_escape` 1.3: a low-level caster at critical HP; no landing check.
    LastDitch,
    /// `borg_escape` 2.2: very dangerous, the teleports failed.
    AfterTeleport,
    /// `borg_escape` 3.1: danger above the HP (the `escape_phase_*` settings).
    Danger,
    /// `borg_escape` 3.3: level 3's teleports failed.
    DangerFallback,
    /// `borg_escape` 4.1: danger near the HP while low level or hurt.
    Compromised,
    /// `borg_escape` 4.3: a weak Mage or Necromancer, the teleports failed.
    WeakCaster,
    /// `borg_escape` 5.1: below `escape_low_level_max_level`, half the HP in danger.
    LowLevel,
    /// `borg_escape` 5.3: a low Mage or Necromancer, the teleports failed.
    LowCaster,
    /// `borg_escape` 6.1: a caster out of mana.
    LowMana,
    /// `borg_escape` 7.1: Shoot N Scoot (phase away, then shoot).
    ShootScoot,
    /// `borg_escape` 8: twitching (stuck) too often.
    Twitch,
    /// `borg_caution`: buying time while Word of Recall counts down.
    RecallWait,
    /// `borg_twitchy` (`borg_flow_misc.rs`): stuck, before moving at random.
    TwitchyMove,
    /// `borg_think_dungeon`: stuck, up to 3 times a level; no check at all.
    TwitchyThink,
}

impl PhaseSite {
    /// The site's name in the notes (and in docs/PHASE-DOOR.md).
    pub(crate) fn label(self) -> &'static str {
        match self {
            PhaseSite::Emergency => "escape 1 emergency",
            PhaseSite::Critical => "escape 1.2 critical",
            PhaseSite::LastDitch => "escape 1.3 last ditch",
            PhaseSite::AfterTeleport => "escape 2.2 after teleport",
            PhaseSite::Danger => "escape 3.1 danger",
            PhaseSite::DangerFallback => "escape 3.3 danger fallback",
            PhaseSite::Compromised => "escape 4.1 compromised",
            PhaseSite::WeakCaster => "escape 4.3 weak caster",
            PhaseSite::LowLevel => "escape 5.1 low level",
            PhaseSite::LowCaster => "escape 5.3 low caster",
            PhaseSite::LowMana => "escape 6.1 low mana",
            PhaseSite::ShootScoot => "escape 7.1 shoot n scoot",
            PhaseSite::Twitch => "escape 8 twitch",
            PhaseSite::RecallWait => "recall wait",
            PhaseSite::TwitchyMove => "twitchy move",
            PhaseSite::TwitchyThink => "twitchy think",
        }
    }
}

/// borg-reborn-rs (#584): what a site checks before it phases. Each site keeps
/// the check it had (C's, plus #539's), so the refactor plays the same keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PhaseCheck {
    /// `borg_caution_phase`: the ability, #539's "a monster next to me", and
    /// at most `tolerance` scary landing grids of 100, looking `horizon`
    /// turns ahead.
    Landing { tolerance: i32, horizon: i32 },
    /// Only #539's "a monster next to me" (`phase_is_pointless`).
    MonsterNear,
    /// `borg_shoot_scoot_safe`, with the danger here.
    ShootScoot { tolerance: i32, horizon: i32, danger: i32 },
    /// No check (the twitchy think, as in C).
    None,
}

/// borg-reborn-rs (#584): the one Phase Door decision. `check` is the site's
/// own check; `cast` reads, casts or activates in the site's own order (a
/// site's list may also hold a teleport, as C's did). Notes the outcome with
/// the site, the HP, the danger here when the site knows it, and the nearest
/// awake monster. Returns whether the Borg acted.
pub(crate) fn phase_door(
    st: &mut BorgState,
    site: PhaseSite,
    check: PhaseCheck,
    danger: Option<i32>,
    cast: impl FnOnce(&mut BorgState) -> bool,
) -> bool {
    // borg-reborn-rs (#584, owner): the one rule, before the site's own check.
    // It only refuses the Phase Door: the site goes on with its other escapes,
    // and the rule's attack or boost waits until no escape acted
    // (`phase_rule_act`, from `borg_caution`).
    if param(Param::escape_phase_adjacent_rule) != 0
        && st.borg.tr(BI_APHASE) > 0
        && phase_rule_verdict(st, site, danger) != PhaseVerdict::Phase
    {
        return false;
    }
    let checked = match check {
        PhaseCheck::Landing { tolerance, horizon } => caution_phase(st, site, tolerance, horizon),
        PhaseCheck::MonsterNear => {
            // With #584's rule on, the rule decides instead of #539.
            let pointless = param(Param::escape_phase_adjacent_rule) == 0 && phase_is_pointless(st);
            if pointless {
                borg_note(st, &format!("# Phase ({}): no, no monster next to me", site.label()));
            }
            !pointless
        }
        PhaseCheck::ShootScoot { tolerance, horizon, danger } => {
            borg_shoot_scoot_safe(st, tolerance, horizon, danger)
        }
        PhaseCheck::None => true,
    };
    if !checked {
        return false;
    }
    let done = cast(st);
    if done {
        let note = phase_note(site, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), danger, nearest_awake(st));
        borg_note(st, &note);
    }
    done
}

/// borg-reborn-rs (#584, owner): what the rule decides in a think.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PhaseVerdict {
    /// An enemy on an adjacent square and the fight looks lost: the site's
    /// own check and Phase Door follow.
    Phase,
    /// An enemy on an adjacent square, the fight not lost: no Phase Door;
    /// if no escape acts, counter-attack.
    Fight,
    /// No enemy on an adjacent square: no Phase Door; if no escape acts, the
    /// best attack (necessarily at range), else a boost.
    Shoot,
}

/// The rule (`escape_phase_adjacent_rule`): Phase Door only with an awake
/// enemy on an adjacent square and the fight judged lost, the danger where
/// the Borg stands at or above `escape_phase_lost_danger_percent` of the
/// current HP.
fn phase_verdict(adjacent: bool, danger: i32, hp: i32, lost_percent: i32) -> PhaseVerdict {
    if !adjacent {
        PhaseVerdict::Shoot
    } else if i64::from(danger) * 100 >= i64::from(hp) * i64::from(lost_percent) {
        PhaseVerdict::Phase
    } else {
        PhaseVerdict::Fight
    }
}

/// borg-reborn-rs (#584, owner): whether the adjacent fight is lost, by
/// `escape_phase_lost_test` (owner, 2026-11-19: both comparisons behind a
/// setting):
/// - 0: the danger here at or above `escape_phase_lost_danger_percent` of
///   the HP (`phase_verdict`);
/// - 1 (compiled): the fight compared. The Borg's best attack now
///   (`borg_attack_estimate`, its expected damage) kills the adjacent awake
///   monsters (their HP, `kill.power`) in more turns than the danger here
///   kills the Borg (`fight_lost_by`);
/// - 2: as 1, and the expected landing is safer than here: the mean danger
///   of the sampled landing grids (`landing_danger`) below the danger here.
fn fight_lost(st: &mut BorgState, here: i32, hp: i32) -> bool {
    match param(Param::escape_phase_lost_test) {
        0 => phase_verdict(true, here, hp, param(Param::escape_phase_lost_danger_percent)) == PhaseVerdict::Phase,
        test => {
            let foes = adjacent_awake_hp(st);
            let value = crate::borg_fight_attack::borg_attack_estimate(st).unwrap_or(0);
            let lost = fight_lost_by(foes, value, here, hp);
            let note = format!("# Phase rule: the fight: adjacent HP {foes}, best attack {value} a turn, danger {here}, HP {hp}: {}", if lost { "lost" } else { "not lost" });
            borg_note(st, &note);
            if !lost || test < 2 {
                return lost;
            }
            let landing = landing_danger(st);
            borg_note(st, &format!("# Phase rule: the landing: mean danger {landing}, here {here}"));
            landing < here
        }
    }
}

/// The fight is lost when the Borg's attack (`value` a turn) needs more
/// turns to kill `foes` HP than the danger (`danger` a turn) needs to kill
/// its `hp`: foes / value > hp / danger. No attack: lost.
fn fight_lost_by(foes: i32, value: i32, danger: i32, hp: i32) -> bool {
    value <= 0 || i64::from(foes) * i64::from(danger) > i64::from(hp) * i64::from(value)
}

/// The summed HP (`kill.power`) of the awake known monsters next to the Borg.
fn adjacent_awake_hp(st: &BorgState) -> i32 {
    (1..st.kills.nxt.max(0) as usize)
        .filter(|&i| {
            let kill = &st.kills.kills[i];
            kill.r_idx != 0 && kill.awake && engine::distance(st.borg.c, kill.pos) <= 1
        })
        .map(|i| i32::from(st.kills.kills[i].power).max(1))
        .sum()
}

/// The mean danger (one turn) of the grids a Phase Door may land on: up to
/// `escape_landing_samples` floor grids without a monster, 5 to 10 steps
/// away, drawn as `count_scary` draws them. `i32::MAX` when none is found.
fn landing_danger(st: &mut BorgState) -> i32 {
    let dis = 10;
    let min = dis / 2;
    let (mut sum, mut n) = (0i64, 0i64);
    for _ in 0..param(Param::escape_landing_samples) {
        for _ in 0..param(Param::escape_landing_trials) {
            let y = rand_spread(st.borg.c.y, dis);
            let x = rand_spread(st.borg.c.x, dis);
            let d = engine::distance(st.borg.c, loc { x, y });
            if d < min || d > dis || y <= 0 || y >= AUTO_MAX_Y - 1 || x <= 0 || x >= AUTO_MAX_X - 1 {
                continue;
            }
            let g = *st.cave.grid(x, y);
            if !borg_cave_floor_bold(st, y, x) || g.kill != 0 || g.web {
                continue;
            }
            sum += i64::from(borg_danger(st, y, x, 1, true, false));
            n += 1;
            break;
        }
    }
    if n == 0 { i32::MAX } else { (sum / n) as i32 }
}

/// borg-reborn-rs (#584, owner): the rule's verdict this think. Decided and
/// noted once, at the first site that asks (re-review of #612: one note and
/// no attack simulation per site); later sites reuse it. `danger` is the
/// site's danger where the Borg stands (`borg_escape`'s `b_q` is
/// `borg_caution`'s `pos_danger`), else measured here.
fn phase_rule_verdict(st: &mut BorgState, site: PhaseSite, danger: Option<i32>) -> PhaseVerdict {
    let now = st.borg.time.now;
    if let Some((t, verdict, _)) = st.borg.phase_rule {
        if t == now {
            return verdict;
        }
    }
    let hp = st.borg.tr(BI_CURHP);
    let here = danger.unwrap_or_else(|| borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false));
    let adjacent = adjacent_awake(st);
    let lost = adjacent && fight_lost(st, here, hp);
    let verdict = if !adjacent {
        PhaseVerdict::Shoot
    } else if lost {
        PhaseVerdict::Phase
    } else {
        PhaseVerdict::Fight
    };
    let label = site.label();
    let why = match verdict {
        PhaseVerdict::Phase => format!("enemy adjacent and the fight looks lost (danger {here}, HP {hp}): the rule allows a Phase Door"),
        PhaseVerdict::Fight => format!("enemy adjacent but the fight is not lost (danger {here}, HP {hp}): no Phase Door; other escapes, else fight"),
        PhaseVerdict::Shoot => "no enemy on an adjacent square: no Phase Door; other escapes, else attack at range or boost".to_string(),
    };
    borg_note(st, &format!("# Phase rule (from {label}): {why}"));
    st.borg.phase_rule = Some((now, verdict, false));
    verdict
}

/// borg-reborn-rs (#584, owner): after `borg_escape` found no escape, act on
/// the rule's refusal this think, once: the Borg's best attack by its own
/// ranking (`borg_attack`; with no enemy adjacent only a ranged one can be
/// chosen), else, with an awake monster in view, a boost (`cornered_boost`:
/// Speed first, then Heroism, Berserk Strength, Bless; its Monster Confusion
/// needs a big monster adjacent, so never in the Shoot case). The boost is
/// the cornered one by its set, not by its premise: in the Fight case the
/// fight is not lost. Returns whether the Borg acted.
pub(crate) fn phase_rule_act(st: &mut BorgState) -> bool {
    let now = st.borg.time.now;
    let Some((t, verdict, tried)) = st.borg.phase_rule else {
        return false;
    };
    if t != now || tried || verdict == PhaseVerdict::Phase {
        return false;
    }
    st.borg.phase_rule = Some((t, verdict, true));
    if crate::borg_fight_attack::borg_attack(st, false) {
        borg_note(st, "# Phase rule: attacked instead of a Phase Door");
        return true;
    }
    if !awake_in_view(st) {
        return false;
    }
    if crate::borg_caution::cornered_boost(st, "Phase rule boost") {
        return true;
    }
    // The owner's "potions, scrolls, spells, staves": `borg_defend`'s
    // boosts (resistances, Protection from Evil, shields, the staves), at the
    // danger where the Borg stands.
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    if crate::borg_fight_defend::borg_defend(st, here) {
        borg_note(st, "# Phase rule boost: a defence (borg_defend)");
        return true;
    }
    false
}

/// An awake known monster in view (#584's boost waits for one).
fn awake_in_view(st: &BorgState) -> bool {
    (1..st.kills.nxt.max(0) as usize).any(|i| {
        let kill = &st.kills.kills[i];
        kill.r_idx != 0
            && engine::r_max() != 0
            && (kill.r_idx as i32) < engine::r_max() as i32 - 1
            && kill.awake
            && st.cave.grid(kill.pos.x, kill.pos.y).info & BORG_VIEW != 0
    })
}

/// The note of a Phase Door the Borg did (#584).
fn phase_note(site: PhaseSite, hp: i32, max_hp: i32, danger: Option<i32>, nearest: Option<i32>) -> String {
    let danger = danger.map_or(String::new(), |d| format!(", danger {d}"));
    let nearest = nearest.map_or("none".to_string(), |d| d.to_string());
    format!("# Phase ({}): done, HP {hp}/{max_hp}{danger}, nearest awake monster {nearest}", site.label())
}

/// The distance to the nearest known awake monster (#584's note).
fn nearest_awake(st: &BorgState) -> Option<i32> {
    (1..st.kills.nxt.max(0) as usize)
        .filter_map(|i| {
            let kill = &st.kills.kills[i];
            (kill.r_idx != 0 && kill.awake).then(|| engine::distance(st.borg.c, kill.pos))
        })
        .min()
}

/// Help determine if PHASE_DOOR seems like a good idea
/// (`borg_caution_phase`; #584: with the asking site in its notes).
fn caution_phase(st: &mut BorgState, site: PhaseSite, emergency: i32, turns: i32) -> bool {
    // must have the ability
    if st.borg.tr(BI_APHASE) == 0 {
        return false;
    }

    // borg-reborn-rs (#539): not while the monster is still distant. With
    // #584's rule on, the rule decides instead (it must not rely on #539).
    if param(Param::escape_phase_adjacent_rule) == 0 && phase_is_pointless(st) {
        borg_note(st, &format!("# Phase ({}): no, no monster next to me", site.label()));
        return false;
    }

    // Skip unknown grids
    let n = count_scary(st, 10, turns, |st, x, y| {
        u32::from(st.cave.grid(x, y).feat) == FEAT_NONE
    });

    // Too much danger
    // in an emergency try with extra danger allowed
    if n > emergency {
        borg_note(st, &format!("# Phase ({}): no, {n} scary landing grids of 100 (at most {emergency})", site.label()));
        return false;
    }
    borg_note(st, &format!("# Phase ({}): landing ok, {n} scary landing grids of 100 (at most {emergency})", site.label()));

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
                || borg_timer(st, st.borg.time.level) > param(Param::escape_unknown_teleport_level_turns))
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
    if st.borg.tr(BI_CURHP) < param(Param::escape_shadow_shift_min_hp) {
        return false;
    }
    borg_spell_fail(st, SHADOW_SHIFT, allow_fail)
}

/// medium range teleport
pub fn borg_dimension_door(st: &mut BorgState, allow_fail: i32) -> bool {
    // for now keep the range at under 50, for performance
    let range = param(Param::escape_dimension_door_range);

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

            let d = borg_danger(st, t_y, t_x, param(Param::escape_dimension_door_horizon), true, false);
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

        // borg-reborn-rs (#470): never Deep Descent. It lands five levels
        // below the deepest level so far, past any plan cap (borg-zoo#147).
    }

    false
}

/// `borg.avoidance * (tenths + risky_boost) / 10`
fn avoid(st: &BorgState, tenths: i32, risky_boost: i32) -> i32 {
    i32::from(st.borg.avoidance) * (tenths + risky_boost) / 10
}

/// `borg.near.unique >= 1 && borg.near.unique <= 8`
fn near_unique_1_8(st: &BorgState) -> bool {
    st.borg.near.unique >= 1 && i32::from(st.borg.near.unique) <= param(Param::escape_near_unique_max_count)
}

/// Try to phase door or teleport
/// b_q is the danger where the Borg stands (`borg_caution`'s `pos_danger`;
/// C's comment said "of the least dangerous square around us", but C passes
/// the same `pos_danger`, borg-caution.c).
pub fn borg_escape(st: &mut BorgState, b_q: i32) -> bool {
    let mut risky_boost = 0;

    // only escape with spell if fail is low
    let mut allow_fail = param(Param::escape_normal_fail);

    // if very healthy, allow extra fail
    if st.borg.tr(BI_CURHP) * 100 / st.borg.tr(BI_MAXHP) > param(Param::escape_healthy_hp_percent) {
        allow_fail = param(Param::escape_healthy_fail);
    }

    // comprimised, get out of the fight
    if st.borg.tr(BI_ISHEAVYSTUN) != 0 {
        allow_fail = param(Param::escape_stunned_fail);
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
    if st.borg.tr(BI_CDEPTH) == 100 && st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) * param(Param::escape_morgoth_hold_hp_tenths) / 10 {
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
        if glyphs >= param(Param::escape_morgoth_hold_glyph_count) {
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
        borg_note(st, "# Attempting to leave via teleport level");
        return true;
    }

    // Risky borgs are more likely to stay in a fight
    if st.cfg(BORG_PLAYS_RISKY) != 0 {
        risky_boost = param(Param::escape_risky_danger_extra_tenths);
    }

    let rb = risky_boost;

    // 1. really scary, I'm about to die
    // Try an emergency teleport, or phase door as last resort
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        || b_q > avoid(st, param(Param::escape_emergency_danger_tenths), rb)
        || (b_q > avoid(st, param(Param::escape_morgoth_danger_tenths), rb)
            && st.borg.near.unique >= 10
            && st.borg.tr(BI_CDEPTH) == 100
            && st.borg.tr(BI_CURHP) < param(Param::escape_morgoth_max_hp))
        || (b_q > avoid(st, param(Param::escape_sauron_danger_tenths), rb)
            && st.borg.near.unique >= 10
            && st.borg.tr(BI_CDEPTH) == 99
            && st.borg.tr(BI_CURHP) < param(Param::escape_sauron_max_hp))
        || (b_q > avoid(st, param(Param::escape_deep_unique_danger_tenths), rb)
            && near_unique_1_8(st)
            && st.borg.tr(BI_CDEPTH) >= param(Param::escape_deep_unique_depth)
            && st.borg.tr(BI_CURHP) < param(Param::escape_deep_unique_max_hp))
        || (b_q > avoid(st, param(Param::escape_unique_danger_tenths), rb) && near_unique_1_8(st) && st.borg.tr(BI_CDEPTH) < param(Param::escape_deep_unique_depth))
        || (b_q > avoid(st, param(Param::escape_ordinary_danger_tenths), rb) && st.borg.near.unique == 0)
    {
        let tmp_allow_fail = param(Param::escape_emergency_fail);

        if borg_escape_stair(st)
            || (borg_allow_teleport(st)
                && (borg_dimension_door(st, tmp_allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_spell_fail(st, TELEPORT_SELF, tmp_allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_spell_fail(st, PORTAL, tmp_allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_shadow_shift(st, tmp_allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_read_scroll(st, st.item_val.sv_scroll_teleport)
                    || borg_use_staff_fail(st, st.item_val.sv_staff_teleportation)
                    || borg_activate_item(st, st.act.tele_long)
                    || borg_teleport_off_level(st)
                    // revisit spells, increased fail rate
                    || borg_dimension_door(st, tmp_allow_fail + param(Param::escape_emergency_fail_extra))
                    || borg_spell_fail(st, TELEPORT_SELF, tmp_allow_fail + param(Param::escape_emergency_fail_extra))
                    || borg_spell_fail(st, PORTAL, tmp_allow_fail + param(Param::escape_emergency_fail_extra))
                    || borg_shadow_shift(st, tmp_allow_fail + param(Param::escape_emergency_fail_extra))
                    // revisit teleport, increased fail rate
                    || borg_use_staff(st, st.item_val.sv_staff_teleportation)
                    // Attempt Teleport Level
                    || borg_spell_fail(st, TELEPORT_LEVEL, tmp_allow_fail + param(Param::escape_emergency_fail_extra))
                    // try phase at least, with some hedging of the safety of
                    // landing zone
                    || (phase_door(st, PhaseSite::Emergency, PhaseCheck::Landing { tolerance: param(Param::escape_emergency_phase_tolerance), horizon: param(Param::escape_emergency_phase_horizon) }, Some(b_q), |st| {
                borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                            || borg_activate_item(st, st.act.tele_phase)
                            || borg_spell_fail(st, PHASE_DOOR, tmp_allow_fail)
                            || borg_spell_fail(st, PORTAL, tmp_allow_fail)
            }))))
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
            && st.borg.tr(BI_CLEVEL) < param(Param::escape_last_ditch_max_level)
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::escape_last_ditch_hp_divisor)
            && borg_allow_teleport(st)
            && (borg_dimension_door(st, param(Param::escape_last_ditch_dimension_fail))
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
            && (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::escape_critical_phase_hp_divisor) || b_q > avoid(st, param(Param::escape_emergency_danger_tenths), rb))
            && phase_door(st, PhaseSite::Critical, PhaseCheck::MonsterNear, Some(b_q), |st| {
                borg_activate_item(st, st.act.tele_phase) || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
            })
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 1.2  Critical Phase");
            reset_antisummon(st);
            return true;
        }

        // emergency phase spell
        if st.borg.tr(BI_CDEPTH) != 0
            && st.borg.tr(BI_CLEVEL) < param(Param::escape_last_ditch_phase_max_level)
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::escape_last_ditch_phase_hp_divisor)
            && phase_door(st, PhaseSite::LastDitch, PhaseCheck::MonsterNear, Some(b_q), |st| {
                borg_spell_fail(st, PHASE_DOOR, param(Param::escape_last_ditch_phase_fail)) || borg_spell(st, PORTAL)
            })
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
    if b_q < avoid(st, param(Param::escape_endgame_stay_danger_tenths), rb)
        && st.borg.near.unique >= 1
        && i32::from(st.borg.near.unique) <= param(Param::escape_endgame_stay_unique_count)
        && st.borg.tr(BI_CDEPTH) >= param(Param::escape_endgame_stay_min_depth)
    {
        return false;
    }

    // 2 - a bit more scary/
    // Attempt to teleport (usually)
    // do not escape from uniques so quick
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        || (b_q > avoid(st, param(Param::escape_mage_danger_tenths), rb)
            && st.borg.tr(BI_CLASS) == CLASS_MAGE
            && st.borg.tr(BI_CURSP) <= param(Param::escape_mage_max_sp)
            && st.borg.tr(BI_MAXCLEVEL) >= param(Param::escape_mage_min_level))
        || (b_q > avoid(st, param(Param::escape_teleport_unique_danger_tenths), rb) && near_unique_1_8(st) && st.borg.tr(BI_CDEPTH) != 99)
        || (b_q > avoid(st, param(Param::escape_teleport_ordinary_danger_tenths), rb) && st.borg.near.unique == 0)
    {
        // Try teleportation
        if borg_escape_stair(st)
            || (borg_allow_teleport(st)
                && (borg_dimension_door(st, allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_spell_fail(st, TELEPORT_SELF, allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_spell_fail(st, PORTAL, allow_fail - param(Param::escape_initial_fail_discount))
                    || borg_shadow_shift(st, allow_fail - param(Param::escape_initial_fail_discount))
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
        if phase_door(st, PhaseSite::AfterTeleport, PhaseCheck::Landing { tolerance: param(Param::escape_teleport_phase_tolerance), horizon: param(Param::escape_teleport_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell(st, PHASE_DOOR)
                || borg_spell(st, PORTAL)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || borg_activate_item(st, st.act.tele_phase))
            })
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
        || (b_q > avoid(st, param(Param::escape_phase_unique_danger_tenths), rb) && i32::from(st.borg.near.unique) >= param(Param::escape_phase_unique_min_count) && i32::from(st.borg.near.unique) <= param(Param::escape_near_unique_max_count))
        || (b_q > avoid(st, param(Param::escape_phase_ordinary_danger_tenths), rb) && st.borg.near.unique == 0)
        || (b_q > avoid(st, param(Param::escape_afraid_warrior_danger_tenths), rb)
            && st.borg.tr(BI_ISAFRAID) != 0
            && (st.borg.tr(BI_AMISSILES) <= 0 && st.borg.tr(BI_CLASS) == CLASS_WARRIOR))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (phase_door(st, PhaseSite::Danger, PhaseCheck::Landing { tolerance: param(Param::escape_phase_tolerance), horizon: param(Param::escape_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            }))
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
        if phase_door(st, PhaseSite::DangerFallback, PhaseCheck::Landing { tolerance: param(Param::escape_fallback_phase_tolerance), horizon: param(Param::escape_fallback_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_spell_fail(st, PORTAL, allow_fail)
                || borg_shadow_shift(st, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            })
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
            && (st.borg.near.unique == 0 || st.borg.tr(BI_CLEVEL) < param(Param::escape_failed_teleport_flee_max_level))
            && !st.borg.status.vault
        {
            // Note
            borg_note(st, "# Fleeing (failed to teleport)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }

        // Flee now
        if !st.borg.goal.leaving
            && (st.borg.near.unique == 0 || st.borg.tr(BI_CLEVEL) < param(Param::escape_failed_teleport_leave_max_level))
            && !st.borg.status.vault
        {
            // Flee!
            borg_note(st, "# Leaving (failed to teleport)");

            // Start leaving
            st.borg.goal.leaving = true;
        }
    }

    // 4- not too scary but I'm compromised
    if (b_q > avoid(st, param(Param::escape_compromised_danger_tenths), rb)
        && (st.borg.tr(BI_CLEVEL) < param(Param::escape_compromised_max_level) || st.borg.tr(BI_CURHP) <= st.borg.tr(BI_MAXHP) / param(Param::escape_compromised_hp_divisor)))
        || (b_q > avoid(st, param(Param::escape_compromised_unique_danger_tenths), rb)
            && near_unique_1_8(st)
            && (st.borg.tr(BI_CLEVEL) < param(Param::escape_compromised_unique_max_level) || st.borg.tr(BI_CURHP) <= st.borg.tr(BI_MAXHP) / param(Param::escape_compromised_unique_hp_divisor)))
        || (b_q > avoid(st, param(Param::escape_compromised_ordinary_danger_tenths), rb) && st.borg.tr(BI_CLEVEL) <= param(Param::escape_compromised_ordinary_max_level) && st.borg.near.unique == 0)
        || (b_q > avoid(st, param(Param::escape_compromised_general_danger_tenths), rb) && st.borg.tr(BI_CLEVEL) <= param(Param::escape_compromised_general_max_level))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (phase_door(st, PhaseSite::Compromised, PhaseCheck::Landing { tolerance: param(Param::escape_compromised_phase_tolerance), horizon: param(Param::escape_compromised_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_shadow_shift(st, allow_fail)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            }))
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
            && st.borg.tr(BI_CLEVEL) < param(Param::escape_failed_compromised_flee_max_level)
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
            && st.borg.tr(BI_CLEVEL) <= param(Param::escape_weak_caster_max_level))
            && phase_door(st, PhaseSite::WeakCaster, PhaseCheck::Landing { tolerance: param(Param::escape_weak_caster_phase_tolerance), horizon: param(Param::escape_weak_caster_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_activate_item(st, st.act.tele_long)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            })
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
    if st.borg.tr(BI_CLEVEL) < param(Param::escape_low_level_max_level)
        && (b_q > avoid(st, param(Param::escape_low_level_danger_tenths), rb) || (b_q > avoid(st, param(Param::escape_low_level_unique_danger_tenths), rb) && near_unique_1_8(st)))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (phase_door(st, PhaseSite::LowLevel, PhaseCheck::Landing { tolerance: param(Param::escape_low_level_phase_tolerance), horizon: param(Param::escape_low_level_phase_horizon) }, Some(b_q), |st| {
                borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_shadow_shift(st, allow_fail)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
            }))
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
            && st.borg.tr(BI_CLEVEL) <= param(Param::escape_low_caster_max_level))
            && phase_door(st, PhaseSite::LowCaster, PhaseCheck::Landing { tolerance: param(Param::escape_low_caster_phase_tolerance), horizon: param(Param::escape_low_caster_phase_horizon) }, Some(b_q), |st| {
                borg_spell_fail(st, PHASE_DOOR, allow_fail)
                || borg_activate_item(st, st.act.tele_phase)
                || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || borg_activate_item(st, st.act.tele_long)
            })
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
        && (b_q > avoid(st, param(Param::escape_low_mana_danger_tenths), rb) || (b_q > avoid(st, param(Param::escape_low_mana_unique_danger_tenths), rb) && near_unique_1_8(st)))
        && (st.borg.tr(BI_CURSP) <= st.borg.tr(BI_MAXSP) / param(Param::escape_low_mana_sp_divisor) && st.borg.tr(BI_MAXSP) >= param(Param::escape_low_mana_min_maxsp))
    {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (phase_door(st, PhaseSite::LowMana, PhaseCheck::Landing { tolerance: param(Param::escape_low_mana_phase_tolerance), horizon: param(Param::escape_low_mana_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            }))
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
        // Phase door
        && phase_door(
            st,
            PhaseSite::ShootScoot,
            PhaseCheck::ShootScoot { tolerance: param(Param::escape_scoot_tolerance), horizon: param(Param::escape_scoot_horizon), danger: b_q },
            Some(b_q),
            |st| borg_spell_fail(st, PHASE_DOOR, allow_fail) || borg_spell_fail(st, PORTAL, allow_fail),
        )
    {
        // Flee!
        borg_note(st, "# Shoot N Scoot. (Danger Level 7.1)");
        phase_not_escape(st);
        reset_antisummon(st);

        // Success
        return true;
    }

    // 8- not too scary but twitching
    if i32::from(st.borg.times_twitch) > param(Param::escape_twitch_limit) {
        // Phase door, if useful
        if borg_escape_stair(st)
            || (phase_door(st, PhaseSite::Twitch, PhaseCheck::Landing { tolerance: param(Param::escape_twitch_phase_tolerance), horizon: param(Param::escape_twitch_phase_horizon) }, Some(b_q), |st| {
                borg_timer(st, st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay) && (borg_spell_fail(st, PHASE_DOOR, allow_fail)
                    || borg_spell_fail(st, PORTAL, allow_fail)
                    || borg_activate_item(st, st.act.tele_phase)
                    || borg_read_scroll(st, st.item_val.sv_scroll_phase_door))
            }))
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

#[cfg(test)]
mod phase_tests {
    use super::*;

    /// #539: a phase helps only against an awake monster next to the Borg,
    /// or an awake one in view that hurts from range.
    #[test]
    fn a_phase_needs_a_monster_that_can_hurt_now() {
        assert!(!phase_helps(&[]), "no monster");
        assert!(!phase_helps(&[(true, 4, false, true)]), "Grip four grids away");
        assert!(phase_helps(&[(true, 1, false, true)]), "adjacent");
        assert!(!phase_helps(&[(false, 1, false, true)]), "adjacent but asleep");
        assert!(phase_helps(&[(true, 6, true, true)]), "a caster in view");
        assert!(!phase_helps(&[(true, 6, true, false)]), "a caster out of view");
        assert!(phase_helps(&[(true, 6, false, true), (true, 1, false, true)]), "one adjacent among others");
    }
}

#[cfg(test)]
mod phase_decision_tests {
    use super::*;

    const ALL: [PhaseSite; 16] = [
        PhaseSite::Emergency,
        PhaseSite::Critical,
        PhaseSite::LastDitch,
        PhaseSite::AfterTeleport,
        PhaseSite::Danger,
        PhaseSite::DangerFallback,
        PhaseSite::Compromised,
        PhaseSite::WeakCaster,
        PhaseSite::LowLevel,
        PhaseSite::LowCaster,
        PhaseSite::LowMana,
        PhaseSite::ShootScoot,
        PhaseSite::Twitch,
        PhaseSite::RecallWait,
        PhaseSite::TwitchyMove,
        PhaseSite::TwitchyThink,
    ];

    /// #584: every site has its own name in the notes.
    #[test]
    fn every_phase_site_has_its_own_label() {
        let mut seen = std::collections::BTreeSet::new();
        for site in ALL {
            assert!(seen.insert(site.label()), "{:?}", site);
        }
    }

    /// #584 (owner): Phase Door only with an enemy adjacent and the fight
    /// lost; adjacent and not lost: fight; nothing adjacent: shoot or boost.
    #[test]
    fn the_rule_phases_only_adjacent_and_lost() {
        assert_eq!(phase_verdict(false, 999, 10, 100), PhaseVerdict::Shoot, "nothing adjacent, however dangerous");
        assert_eq!(phase_verdict(true, 10, 10, 100), PhaseVerdict::Phase, "danger at the HP");
        assert_eq!(phase_verdict(true, 9, 10, 100), PhaseVerdict::Fight, "danger below the HP");
        assert_eq!(phase_verdict(true, 5, 10, 50), PhaseVerdict::Phase, "a lower percent phases sooner");
        assert_eq!(phase_verdict(true, 0, 10, 0), PhaseVerdict::Phase, "0 percent: any adjacent enemy");
    }

    /// #584 (owner): the fight is lost when the Borg needs more turns to kill
    /// the adjacent monsters than they need to kill it; no attack is lost.
    #[test]
    fn the_fight_is_lost_when_the_borg_dies_first() {
        // 20 HP of foes, 5 a turn: 4 turns to win; 12 danger on 40 HP: 3.3 turns to die.
        assert!(fight_lost_by(20, 5, 12, 40));
        // 10 danger on 40 HP: 4 turns to die; a draw is not lost.
        assert!(!fight_lost_by(20, 5, 10, 40));
        assert!(!fight_lost_by(20, 10, 12, 40), "a stronger attack wins");
        assert!(fight_lost_by(1, 0, 1, 100), "no attack");
        assert!(!fight_lost_by(20, 5, 0, 40), "no danger");
    }

    /// #584: the note of a Phase Door names the site, the HP, the danger
    /// when the site knows it, and the nearest awake monster.
    #[test]
    fn the_phase_note_says_where_and_why() {
        assert_eq!(
            phase_note(PhaseSite::Compromised, 31, 48, Some(40), Some(1)),
            "# Phase (escape 4.1 compromised): done, HP 31/48, danger 40, nearest awake monster 1"
        );
        assert_eq!(
            phase_note(PhaseSite::TwitchyThink, 48, 48, None, None),
            "# Phase (twitchy think): done, HP 48/48, nearest awake monster none"
        );
    }
}

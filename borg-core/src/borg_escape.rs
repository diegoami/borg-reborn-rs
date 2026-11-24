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
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_legal_fail, borg_spell_okay_fail};
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
/// `GOAL_KILL`, `GOAL_TAKE` (borg-flow.h): not walks (#620).
const GOAL_KILL: i32 = 1;
const GOAL_TAKE: i32 = 2;
/// `GOAL_RECOVER` (borg-flow.h): a walk to rest, not worth the shape.
const GOAL_RECOVER: i32 = 9;
use crate::borg_headers::borg_trait_h::{
    BI_AESCAPE, BI_AMISSILES, BI_APHASE, BI_ATELEPORT, BI_CDEPTH, BI_CLASS, BI_CLEVEL,
    BI_CRSNOTEL, BI_CURHP, BI_CURSP, BI_ISAFRAID, BI_ISCUT, BI_ISHEAVYSTUN, BI_ISPOISONED,
    BI_ISWEAK, BI_KING, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP,
};
use crate::headers::list_mon_race_flags_h::{RF_KILL_WALL, RF_PASS_WALL};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_terrain_h::{FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE};
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
pub(crate) fn reset_antisummon(st: &mut BorgState) {
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
/// #758: flight-only entry point. The failed-teleport fallback sets flight
/// flags; borg_caution later reaches this same entry point. Trip-home calls
/// continue to call borg_recall directly, without this gate.
pub(crate) fn borg_recall_for_flight(st: &mut BorgState) -> bool {
    // Match borg_recall's existing pending gate before forecasting: this
    // is not a new read and must not be logged as a blocked Recall.
    if st.borg.goal.recalling != 0 { return false; }
    let source_available = flight_recall_source_available(st);
    let already_noted = std::mem::replace(&mut st.borg.flight_recall_unavailable_noted, !source_available);
    if !source_available {
        if !already_noted {
            borg_note(st, "# Flight Recall unavailable: no usable source (#1142)");
        }
        return false;
    }
    crate::borg_caution::flight_recall_survivable(st) && borg_recall(st)
}

/// The sources `borg_recall` can try in the current think. Recall is not
/// subject to the anti-teleport checks for immediate escapes.
fn flight_recall_source_available(st: &mut BorgState) -> bool {
    use crate::borg_item_use::{charged_activation_slot, readable_scroll_slot, zappable_rod_slot};
    zappable_rod_slot(st, st.item_val.sv_rod_recall) >= 0
        || charged_activation_slot(st, st.act.recall).is_some()
        || borg_spell_okay_fail(st, WORD_OF_RECALL, param(Param::escape_recall_fail))
        || readable_scroll_slot(st, st.item_val.sv_scroll_word_of_recall) >= 0
}

/// Ordinary Recall, including trips home; no flight-survival policy here.
pub fn borg_recall(st: &mut BorgState) -> bool {
    // Multiple "recall" fails
    if st.borg.goal.recalling == 0 {
        // Try to "recall"
        if borg_zap_rod(st, st.item_val.sv_rod_recall)
            || borg_activate_item(st, st.act.recall)
            || borg_spell_fail(st, WORD_OF_RECALL, param(Param::escape_recall_fail))
            || borg_read_scroll(st, st.item_val.sv_scroll_word_of_recall)
        {
            st.borg.flight_recall_unavailable_noted = false;
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
///
/// borg-reborn-rs (#1066): with `unknown_scary`, a candidate grid that
/// `skip_unknown` would skip ends its attempt instead and counts as scary:
/// the Borg cannot know what waits there. Returns the scary count and how
/// many of them were such unknown grids.
fn count_scary(
    st: &mut BorgState,
    dis: i32,
    turns: i32,
    unknown_scary: bool,
    mut skip_unknown: impl FnMut(&mut BorgState, i32, i32) -> bool,
) -> (i32, i32) {
    let min = dis / 2;

    // `ag` of C, as a position
    let mut ag = (st.borg.c.x, st.borg.c.y);

    // Simulate 100 attempts
    let mut n = 0;
    let mut unknown = 0;
    for _k in 0..param(Param::escape_landing_samples) {
        let mut x = 0;
        let mut y = 0;
        let mut never_seen = false;

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
                // borg-reborn-rs (#1066): or count them as scary.
                if unknown_scary {
                    never_seen = true;
                    break;
                }
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

        // borg-reborn-rs (#1066): a landing never seen is scary.
        if never_seen {
            n += 1;
            unknown += 1;
            continue;
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
        // borg-reborn-rs (#556): a landing near a remembered breeder is
        // scary too (a Phase Door into the colony; no random numbers drawn).
        if p > st.borg.tr(BI_CURHP) || crate::borg_caution::near_known_breeder(st, x, y) {
            n += 1;
        }
    }
    (n, unknown)
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

/// borg-reborn-rs (#620): `escape_fox_form` or `escape_fox_form_town` is
/// on: the Borg casts the shape, so the change back is its own (the prompt
/// answer, the revert before a blow, at a shop door).
pub(crate) fn any_fox_form_on() -> bool {
    fox_form_on() || param(Param::escape_fox_form_town) != 0
}

/// borg-reborn-rs (#620): a walk in town with the shape: the switch is on,
/// in town, at most `escape_fox_form_town_max_level`.
fn town_fox_when(on: bool, depth: i32, clevel: i32, max_level: i32) -> bool {
    on && depth == 0 && clevel <= max_level
}

/// borg-reborn-rs (#620): the goal is a walk (to a shop, home, a staircase,
/// or a flight), not a monster to kill, an object to take or a place to rest
/// (review of #648, finding 3: a fox would rest at STR -3).
fn walk_goal(goal: i32) -> bool {
    goal != 0 && goal != GOAL_KILL && goal != GOAL_TAKE && goal != GOAL_RECOVER
}

fn town_fox_now(st: &BorgState) -> bool {
    town_fox_when(
        param(Param::escape_fox_form_town) != 0,
        engine::player_depth(),
        st.borg.tr(BI_CLEVEL),
        param(Param::escape_fox_form_town_max_level),
    )
}

/// borg-reborn-rs (#620, owner): a low-level Druid about to take a step of
/// a walk in town at least `escape_fox_form_town_min_steps` from its goal
/// casts Fox Form first (double movement): town is crossed faster, and the
/// town monsters get fewer turns. It changes back at the shop door
/// (`revert_fox_at_shop`) or before a blow (`revert_fox_to_fight`).
pub(crate) fn fox_form_before_town_step(st: &mut BorgState, why: i32, steps_left: i32) -> bool {
    let fail = param(Param::escape_fox_form_fail);
    if !town_fox_now(st)
        || !walk_goal(why)
        || engine::player_is_shapechanged()
        || steps_left < param(Param::escape_fox_form_town_min_steps)
        || on_stairs(st)
        || !borg_spell_okay_fail(st, FOX_FORM, fail)
        || !mana_kept(
            st.borg.tr(BI_CURSP) - crate::borg_magic::borg_get_spell_power(st, FOX_FORM),
            st.borg.tr(BI_MAXSP),
            st.borg.tr(BI_CLEVEL) <= param(Param::recover_mana_rest_max_level),
            param(Param::recover_mana_rest_sp_tenths),
        )
    {
        return false;
    }
    borg_note(st, &format!("# Fox Form: walking in town ({steps_left} steps to go)"));
    borg_spell_fail(st, FOX_FORM, fail)
}

/// borg-reborn-rs (#620): standing on a staircase. No cast there (it would
/// be spent on the step onto the stairs), and a town fox there changes
/// back (`keep_fox_form`) before the think that takes them.
/// The game's grid, not the Borg's: `borg_think` checks the shape before
/// this think's `borg_update`, when `st.borg.c` is still the grid before the
/// last step (as `keep_fox_form` reads the game's depth).
fn on_stairs(_st: &BorgState) -> bool {
    let at = engine::player_grid();
    let feat = u32::from(engine::square_feat(at.x, at.y));
    feat == FEAT_LESS || feat == FEAT_MORE
}

/// borg-reborn-rs (#620): the cast leaves the mana above `borg_recover`'s
/// mana rest (`recover_mana_rest_*`): a cast that sends the Druid to rest
/// costs more time in town than the fox saves (the first measure: twice the
/// town time, all of it resting for one SP).
fn mana_kept(sp_after: i32, max_sp: i32, rest_applies: bool, rest_tenths: i32) -> bool {
    sp_after >= 0 && (!rest_applies || sp_after >= max_sp * rest_tenths / 10)
}

/// borg-reborn-rs (#620, review of #648): the note of `borg_think`'s change
/// back, when it is the town fox's on a staircase (the other two reverts,
/// at the shop door and before a blow, write their own).
pub(crate) fn note_fox_revert(st: &mut BorgState) {
    if is_fox() && town_fox_now(st) && on_stairs(st) {
        borg_note(st, "# Fox Form: changing back on the staircase");
    }
}

/// borg-reborn-rs (#620): a fox about to step onto a shop entrance changes
/// back first: a shapechanged player on a shop entrance is refused
/// (`player-util.c` `player_handle_post_move`, from `move_player`; also
/// `cmd-cave.c` `do_cmd_hold`): "the door slams shut", and the home refuses
/// silently. Changing back uses no
/// energy; the step comes at the next think.
pub(crate) fn revert_fox_at_shop(st: &mut BorgState) -> bool {
    if !(any_fox_form_on() && is_fox()) {
        return false;
    }
    borg_note(st, "# Fox Form: changing back at the shop door");
    queue_fox_revert(st);
    true
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
    // borg-reborn-rs (#620): and in town while a walk lasts, but not on a
    // staircase: the change back comes before the think that takes the
    // stairs (review of #648, finding 1: a fox arrived in the dungeon).
    let town = town_fox_now(st) && walk_goal(i32::from(st.borg.goal.type_0)) && !on_stairs(st);
    is_fox() && ((fox_form_on() && keep_fox_when(engine::player_depth(), flight, scary)) || town)
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
    if !(any_fox_form_on() && is_fox()) {
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
    use super::{cast_fox_when, keep_fox_when, mana_kept, town_fox_when, walk_goal};

    #[test]
    fn e_a_town_cast_keeps_the_mana_above_the_rest() {
        assert!(!mana_kept(4, 7, true, 8), "7 SP, 4 left: below 5 (7 x 8 / 10), the cast sends it to rest");
        assert!(mana_kept(1, 2, true, 8), "the rest's own integer line: 2 x 8 / 10 = 1");
        assert!(mana_kept(7, 8, true, 8), "8 SP: 7 left is above 6.4");
        assert!(mana_kept(1, 2, false, 8), "above the mana rest's level");
        assert!(!mana_kept(-1, 2, false, 8), "not enough mana");
    }

    #[test]
    fn d_a_town_fox_is_a_low_level_walk_in_town() {
        assert!(town_fox_when(true, 0, 20, 20));
        assert!(!town_fox_when(false, 0, 5, 20), "switch off");
        assert!(!town_fox_when(true, 1, 5, 20), "the dungeon: #551's rules");
        assert!(!town_fox_when(true, 0, 21, 20), "above the level");
        assert!(walk_goal(3) && walk_goal(7), "a shop or stairs walk, a flight");
        assert!(!walk_goal(0) && !walk_goal(1) && !walk_goal(2) && !walk_goal(9), "no goal, a kill, a take, a rest");
    }

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
    /// #781: confirmed unseen bump, after bounded oil attempts.
    UnseenBump,
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
    /// #842: a fast-fragile monster next to the Borg, the next turn kills it
    /// and the kill needs more than one more turn (`borg_fast_fragile.rs`).
    FastFragile,
    /// #1064 (review of #1082): #969's floor declined the attack on a
    /// `kill-on-sight` entry and its cure does not outlive the next turn
    /// (`borg_fight_attack.rs` `floor_escape`).
    KillOnSightFloor,
    /// #1147 (re-review of #1148, finding 2): `caution_walk_away_immobile`,
    /// cornered next to a walk-away monster with no square out of its reach
    /// and not confused; the last try before a blow (`borg_caution.rs`
    /// `walk_away_phase`).
    WalkAwayCornered,
}

impl PhaseSite {
    /// The site's name in the notes (and in docs/PHASE-DOOR.md).
    pub(crate) fn label(self) -> &'static str {
        match self {
            PhaseSite::UnseenBump => "unseen bump",
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
            PhaseSite::FastFragile => "fast-fragile",
            PhaseSite::KillOnSightFloor => "kill-on-sight floor",
            PhaseSite::WalkAwayCornered => "walk-away cornered",
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
    // borg-reborn-rs (#829): no site may skip it. A teleport curse stops
    // Phase Door too (it is a short `effect_handler_TELEPORT`); a no-teleport
    // grid does not (`dis` 10 is not above 10), so the grid is not asked.
    // borg-reborn-rs (#1094): set again below only if the gate refuses.
    st.borg.phase_gate_refused = None;
    if teleport_stopped(st) {
        borg_note(st, &format!("# Phase ({}): no, teleport is blocked", site.label()));
        return false;
    }
    // borg-reborn-rs (#584, owner): the one rule, before the site's own check.
    // It only refuses the Phase Door: the site goes on with its other escapes,
    // and the rule's attack or boost waits until no escape acted
    // (`phase_rule_act`, from `borg_caution`).
    // Shoot N Scoot's own check answers first (re-review of #612): it only
    // passes with a monster next to the Borg, so asking the rule before it
    // made two thirds of the rule's notes on turns with nothing adjacent.
    let rule = site != PhaseSite::UnseenBump && param(Param::escape_phase_adjacent_rule) != 0 && has_phase(st, site);
    let site_first = matches!(check, PhaseCheck::ShootScoot { .. });
    if rule && !site_first && phase_rule_verdict(st, site, danger) != PhaseVerdict::Phase {
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
    // borg-reborn-rs (review of #1075, finding 2): the sites with no landing
    // check (the critical and last-ditch phases, the twitchy ones,
    // fast-fragile without its landing test) still refuse a jump into the
    // never seen, under #1066's policy.
    let policy = phase_unknown_policy(st);
    let tolerance = param(Param::escape_phase_unknown_tolerance);
    // Re-review of #1075 (finding 2): a landing check in this think, at
    // this grid, already counted the never seen; reuse it, do not redraw,
    // and keep that site's tolerance when it is stricter (#1094).
    let (record, now, at) = (st.borg.phase_unknown_seen, st.borg.time.now, st.borg.c);
    if let Some((n, at_most)) = unknown_gate(check, policy, || gate_count(record, now, at, tolerance, || never_seen_landings(st))) {
        borg_note(st, &format!("# Phase ({}): no, {n} never-seen landing grids of 100 (at most {at_most}) (#1066)", site.label()));
        st.borg.phase_gate_refused = Some(st.borg.time.now);
        return false;
    }
    if rule && site_first && phase_rule_verdict(st, site, danger) != PhaseVerdict::Phase {
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
/// - 1 (compiled): the fight compared. Killing the adjacent awake monsters
///   one after another (their HP left, `kill.power` less `kill.injury`
///   percent) takes more turns than the danger here needs to kill the Borg
///   (`fight_lost_by`). The Borg's damage a turn is the larger of its melee
///   against that monster (`borg_thrust_damage_one`) and the true damage of
///   the attack it would choose now, of any kind (`borg_attack_estimate`: a
///   Mage casts next to an enemy). (Review of #612's Druid death:
///   `borg_attack`'s ranking value adds the kill and danger bonuses to the
///   damage, so the true damage is tracked apart.)
/// - 2: as 1, and the expected landing is safer than here: the mean danger
///   of the sampled landing grids (`landing_danger`) below the danger here.
fn fight_lost(st: &mut BorgState, here: i32, hp: i32) -> bool {
    match param(Param::escape_phase_lost_test) {
        0 => phase_verdict(true, here, hp, param(Param::escape_phase_lost_danger_percent)) == PhaseVerdict::Phase,
        test => {
            // The attack the Borg would choose now, of any kind (a Mage casts
            // next to an enemy too; owner, 2026-11-19): its true damage.
            let best = crate::borg_fight_attack::borg_attack_estimate(st).unwrap_or(0);
            let lost = if param(Param::escape_phase_lost_mana) != 0 {
                // borg-reborn-rs (#625, owner): the chosen attack's damage
                // weighed by its success chance, for as many uses as the mana
                // (or the charges) pay for; then the best attack that is not a
                // spell, or melee.
                let cost = st.fight_attack.est_cost;
                let fallback = st.fight_attack.est_fallback;
                let foes = adjacent_foes(st);
                let primary = Primary::of(best, cost);
                let lost = fight_turns(&foes, primary, fallback).is_none_or(|t| t * f64::from(here) > f64::from(hp));
                let list: Vec<String> = foes.iter().map(|(left, melee)| format!("{left} HP (melee {melee})")).collect();
                let uses = primary.uses.map_or("no limit".to_string(), |u| format!("{u} use{}", if u == 1 { "" } else { "s" }));
                let note = format!(
                    "# Phase rule: the fight: {}; attack {best} at {}% fail ({uses}), then {fallback}; danger {here}, HP {hp}: {}",
                    list.join(", "),
                    cost.map_or(0, |c| c.fail_pct),
                    if lost { "lost" } else { "not lost" }
                );
                borg_note(st, &note);
                lost
            } else {
                let foes: Vec<(i32, i32)> = adjacent_foes(st).into_iter().map(|(left, melee)| (left, melee.max(best))).collect();
                let lost = fight_lost_by(&foes, here, hp);
                let list: Vec<String> = foes.iter().map(|(left, dam)| format!("{left} HP at {dam} a turn")).collect();
                let note = format!("# Phase rule: the fight: {}; danger {here}, HP {hp}: {}", list.join(", "), if lost { "lost" } else { "not lost" });
                borg_note(st, &note);
                lost
            };
            if !lost || test < 2 {
                return lost;
            }
            let landing = landing_danger(st);
            borg_note(st, &format!("# Phase rule: the landing: mean danger {landing}, here {here}"));
            landing < here
        }
    }
}

/// The fight is lost when killing every foe, one after another (HP left and
/// the Borg's damage a turn against it), takes more turns than the danger
/// (`danger` a turn) needs to kill the Borg's `hp`: sum(left / dam) >
/// hp / danger. A foe it cannot hurt (damage 0) loses the fight; no foe, or
/// no danger, does not.
fn fight_lost_by(foes: &[(i32, i32)], danger: i32, hp: i32) -> bool {
    if foes.iter().any(|&(_, dam)| dam <= 0) {
        return true;
    }
    // sum(left_i / dam_i) * danger > hp, in exact fractions: scaled by the
    // product of the damages would overflow; compare in f64.
    let turns: f64 = foes.iter().map(|&(left, dam)| f64::from(left) / f64::from(dam)).sum();
    turns * f64::from(danger) > f64::from(hp)
}

/// borg-reborn-rs (#625): the attack the Borg would choose now, for the
/// fight test: its expected damage a turn (true damage times the success
/// chance) and the uses it can pay for (`None`: no limit).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Primary {
    dam: f64,
    uses: Option<i32>,
}

impl Primary {
    /// From `borg_attack_estimate`'s true damage and the winner's cost
    /// (`None`: melee, missiles, throws: no fail chance, no limit).
    fn of(best: i32, cost: Option<crate::borg_fight_attack::AttackCost>) -> Primary {
        match cost {
            None => Primary { dam: f64::from(best.max(0)), uses: None },
            Some(c) => Primary {
                dam: f64::from(best.max(0)) * f64::from(100 - c.fail_pct.clamp(0, 100)) / 100.0,
                uses: c.uses.map(|u| u.max(0)),
            },
        }
    }
}

/// borg-reborn-rs (#625): the turns to kill the adjacent foes one after
/// another, each `(HP left, melee a turn)`. Each turn uses the chosen attack
/// (`primary`) while it does more than the foe's other damage (the larger of
/// melee and `fallback`, the best attack that is not a spell) and uses are
/// left (fractions count: a 2-use spell spends 0.5 of a use on a foe it
/// kills in half a cast); then the other damage. `None`: a foe the Borg
/// cannot hurt.
fn fight_turns(foes: &[(i32, i32)], primary: Primary, fallback: i32) -> Option<f64> {
    let mut uses = primary.uses.map(f64::from);
    let mut turns = 0.0;
    for &(left, melee) in foes {
        let mut left = f64::from(left);
        let other = f64::from(melee.max(fallback).max(0));
        if primary.dam > other {
            let needed = left / primary.dam;
            match uses {
                None => {
                    turns += needed;
                    continue;
                }
                Some(u) if needed <= u => {
                    turns += needed;
                    uses = Some(u - needed);
                    continue;
                }
                Some(u) => {
                    turns += u;
                    left -= u * primary.dam;
                    uses = Some(0.0);
                }
            }
        }
        if other <= 0.0 {
            return None;
        }
        turns += left / other;
    }
    Some(turns)
}

/// The awake known monsters next to the Borg, as (HP left, the Borg's melee
/// damage a turn against it): `kill.power` less `kill.injury` percent (at
/// least 1), and `borg_thrust_damage_one` without its unique bonus (its
/// Mage/Necromancer reduction stays: it errs towards the Phase Door).
fn adjacent_foes(st: &BorgState) -> Vec<(i32, i32)> {
    (1..st.kills.nxt.max(0) as usize)
        .filter(|&i| {
            let kill = &st.kills.kills[i];
            kill.r_idx != 0
                && engine::r_max() != 0
                && (kill.r_idx as i32) < engine::r_max() as i32 - 1
                && kill.awake
                && engine::distance(st.borg.c, kill.pos) <= 1
        })
        .map(|i| {
            let kill = &st.kills.kills[i];
            let left = (i32::from(kill.power) * (100 - i32::from(kill.injury).clamp(0, 100)) / 100).max(1);
            let mut dam = crate::borg_fight_attack::borg_thrust_damage_one(st, i as i32);
            // `borg_thrust_damage_one` multiplies the damage on a unique in the
            // dungeon by 1 + `attack_melee_unique_bonus` so the Borg targets
            // it: a preference, not damage (Grip, Fang). Taken back out.
            if engine::race_flag_has(usize::from(kill.r_idx), crate::headers::list_mon_race_flags_h::RF_UNIQUE) && st.borg.tr(BI_CDEPTH) >= 1 {
                dam /= 1 + param(Param::attack_melee_unique_bonus).max(0);
            }
            (left, dam)
        })
        .collect()
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
    // borg-reborn-rs (#878): with `escape_phase_lost_damage_only` the fight
    // is judged on the danger without fear (the site's own danger is not
    // used: it carries the fear prices).
    let here = if param(Param::escape_phase_lost_damage_only) != 0 {
        let feared = danger.unwrap_or_else(|| borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false));
        let here = crate::borg_danger::borg_danger_damage_only(st, st.borg.c.y, st.borg.c.x, 1, true, false);
        borg_note(st, &format!("# Phase rule: danger {here} without fear (with fear {feared}) (#878)"));
        here
    } else {
        danger.unwrap_or_else(|| borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false))
    };
    let adjacent = adjacent_awake(st);
    // borg-reborn-rs (#1125, `escape_paralyser_fight_lost`): next to an
    // awake paralyser without Free Action the fight is lost, whatever the
    // danger number says (a floating eye's gaze has no dice: danger 0).
    let paralyser = if adjacent && param(Param::escape_paralyser_fight_lost) != 0 {
        crate::borg_caution::awake_paralyser_adjacent(st)
    } else {
        None
    };
    if let Some(k) = paralyser {
        let name = String::from_utf8_lossy(engine::race_name(usize::from(st.kills.kills[k].r_idx))).into_owned();
        borg_note(st, &format!("# Phase rule: next to the paralysing {name} without Free Action: the fight is lost (#1125)"));
    }
    let lost = phase_lost(adjacent, paralyser.is_some(), || {
        let fight_hp = if param(Param::escape_phase_lost_poison_pending) != 0
            && st.borg.tr(BI_ISPOISONED) != 0
        {
            use crate::headers::{list_player_timed_h::TMD_POISONED, list_stats_h::STAT_CON};
            let pending = crate::borg_poison::pending_hp(
                i32::from(engine::player_timed(TMD_POISONED)),
                engine::player_stat_ind(STAT_CON),
            );
            borg_note(st, &format!("# Phase rule: {pending} HP of running poison against HP {hp} (#1083)"));
            hp - pending
        } else {
            hp
        };
        fight_lost(st, here, fight_hp) && !newcomer_wait(st, fight_hp)
    });
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

/// borg-reborn-rs (#1125, review of #1129, finding 2): with
/// `escape_paralyser_fight_lost`, an awake known paralyser next to the Borg
/// (no Free Action) defers the early fight (`fight_before_flight`: attack on
/// sight, the lone slow monster, the winning fight), so `borg_caution` and
/// the Phase rule, which judges that fight lost, come first.
pub(crate) fn paralyser_defers_early_fight(st: &mut BorgState) -> bool {
    if param(Param::escape_paralyser_fight_lost) == 0 {
        return false;
    }
    let Some(k) = crate::borg_caution::awake_paralyser_adjacent(st) else {
        return false;
    };
    let name = String::from_utf8_lossy(engine::race_name(usize::from(st.kills.kills[k].r_idx))).into_owned();
    borg_note(st, &format!("# Next to the paralysing {name} without Free Action: no early fight, caution first (#1125)"));
    true
}

/// The Phase rule's "lost": an awake enemy adjacent, and the fight lost
/// (`fight`, asked only when needed); borg-reborn-rs (#1125): next to an
/// awake paralyser without Free Action (`paralyser`) it is lost without the
/// fight's test.
fn phase_lost(adjacent: bool, paralyser: bool, fight: impl FnOnce() -> bool) -> bool {
    adjacent && (paralyser || fight())
}

/// borg-reborn-rs (#1066): with `escape_phase_newcomer_wait`, an adjacent
/// fight the rule judged lost is not lost when the rise in danger comes from
/// a newcomer at range: HP at least `escape_phase_newcomer_hp_percent` of
/// the maximum, an awake known monster farther than one grid that endangers
/// the Borg where it stands, and #759's fight race against the adjacent
/// monsters alone winning (`fight_race`, adjacent only; not certified is
/// not winning). The emergency Phase Door waits for the newcomer to arrive.
///
/// Review of #1075 (Codex, finding 1): the next turn must be survivable
/// too. The race's HP a turn from the adjacent monsters plus the danger of
/// the monsters at range (without fear when `escape_phase_lost_damage_only`
/// is on, as the rule reads it) must stay below
/// `escape_phase_newcomer_turn_percent` of HP (`newcomer_turn_safe`);
/// otherwise the Phase Door stays. Re-review of #1075 (finding 1): that
/// danger is the full possible one-turn damage (`full_damage`: no discount
/// for the spell frequency, `spell_frequency_discount`, and whole monster
/// turns), so an infrequent lethal cast still vetoes the wait.
fn newcomer_wait(st: &mut BorgState, hp: i32) -> bool {
    if param(Param::escape_phase_newcomer_wait) == 0 {
        return false;
    }
    let max_hp = st.borg.tr(BI_MAXHP);
    if !unknown_counts_scary(hp, max_hp, param(Param::escape_phase_newcomer_hp_percent)) {
        return false;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let damage_only = param(Param::escape_phase_lost_damage_only) != 0;
    let mut newcomers = 0;
    let mut at_range = 0i64;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || !kill.awake || engine::distance(st.borg.c, kill.pos) <= 1 {
            continue;
        }
        let one = |st: &mut BorgState| crate::borg_danger::borg_danger_one_kill(st, y, x, 1, i as i32, true, true);
        let danger = if damage_only { crate::params::damage_only(|| one(st)) } else { one(st) };
        if danger > 0 {
            newcomers += 1;
            at_range += i64::from(danger);
        }
    }
    if newcomers == 0 {
        return false;
    }
    match crate::borg_caution::fight_race(st, true) {
        None => {
            borg_note(st, &format!("# Phase rule: {newcomers} monster(s) at range, the race against the adjacent ones not certified: no wait (#1066)"));
            false
        }
        Some(race) => {
            let percent = param(Param::escape_phase_newcomer_turn_percent);
            let safe = newcomer_turn_safe(race.incoming, at_range, hp, percent);
            let verdict = match (race.won, safe) {
                (false, _) => "not winning".to_string(),
                (true, false) => format!("winning, but the next turn is not safe ({:.3} + {at_range} at range, at least {percent}% of HP): no wait", race.incoming),
                (true, true) => format!("winning and the next turn safe ({:.3} + {at_range} at range): the Phase Door waits for the newcomer", race.incoming),
            };
            borg_note(
                st,
                &format!(
                    "# Phase rule: {newcomers} monster(s) at range; the race against the {} adjacent: {:.3} turns to kill, {:.3} damage/turn, HP {hp}/{max_hp}: {verdict} (#1066)",
                    race.plans.len(),
                    race.turns,
                    race.incoming,
                ),
            );
            race.won && safe
        }
    }
}

/// Review of #1075 (finding 1): the next turn is survivable when the adjacent
/// monsters' HP a turn (`incoming`, #759's) plus the danger of the monsters
/// at range stays below `percent` of HP.
fn newcomer_turn_safe(incoming: f64, at_range: i64, hp: i32, percent: i32) -> bool {
    incoming.is_finite() && (incoming + at_range as f64) * 100.0 < f64::from(hp) * f64::from(percent)
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
    if crate::borg_combat_heal::quaff(st) {
        return true;
    }
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

/// The Borg has a Phase Door for `site`: `BI_APHASE` (a scroll, an
/// activation, or the spell at 3% failure or less, `borg_trait.rs`), or,
/// for #1147's cornered walk-away (re-review of #1148), the spell up to
/// `caution_walk_away_phase_fail`, the ceiling that site casts with.
fn has_phase(st: &BorgState, site: PhaseSite) -> bool {
    phase_available(st.borg.tr(BI_APHASE), site, || {
        borg_spell_legal_fail(st, PHASE_DOOR, param(Param::caution_walk_away_phase_fail))
    })
}

/// `has_phase` with the spell test given: every site but the cornered
/// walk-away asks `BI_APHASE` alone, as before.
fn phase_available(aphase: i32, site: PhaseSite, walk_away_spell: impl FnOnce() -> bool) -> bool {
    aphase > 0 || (site == PhaseSite::WalkAwayCornered && walk_away_spell())
}

/// Help determine if PHASE_DOOR seems like a good idea
/// (`borg_caution_phase`; #584: with the asking site in its notes).
fn caution_phase(st: &mut BorgState, site: PhaseSite, emergency: i32, turns: i32) -> bool {
    // must have the ability
    if !has_phase(st, site) {
        return false;
    }

    // borg-reborn-rs (#539): not while the monster is still distant. With
    // #584's rule on, the rule decides instead (it must not rely on #539).
    if site != PhaseSite::UnseenBump && param(Param::escape_phase_adjacent_rule) == 0 && phase_is_pointless(st) {
        borg_note(st, &format!("# Phase ({}): no, no monster next to me", site.label()));
        return false;
    }

    // borg-reborn-rs (#1066): with `escape_phase_unknown_scary`, a landing
    // grid the Borg has never seen is scary, unless HP is below
    // `escape_phase_unknown_hp_percent` of the maximum (a real emergency).
    let unknown_scary = phase_unknown_policy(st);

    // Skip unknown grids
    let (n, unknown) = count_scary(st, 10, turns, unknown_scary, |st, x, y| {
        u32::from(st.cave.grid(x, y).feat) == FEAT_NONE
    });
    let never_seen = if unknown_scary { format!(", {unknown} of them never seen (#1066)") } else { String::new() };
    if unknown_scary {
        // Re-review of #1075 (finding 2): for the gate of a later site in
        // this think (the critical phase after a refused emergency).
        st.borg.phase_unknown_seen = Some((st.borg.time.now, st.borg.c, unknown, emergency));
    }

    // Too much danger
    // in an emergency try with extra danger allowed
    if n > emergency {
        borg_note(st, &format!("# Phase ({}): no, {n} scary landing grids of 100 (at most {emergency}){never_seen}", site.label()));
        return false;
    }
    borg_note(st, &format!("# Phase ({}): landing ok, {n} scary landing grids of 100 (at most {emergency}){never_seen}", site.label()));

    // Okay
    true
}

/// borg-reborn-rs (#1066): #1066's policy is on for this think:
/// `escape_phase_unknown_scary` and HP at or above
/// `escape_phase_unknown_hp_percent` of the maximum. Read by the landing
/// check, the gate of the sites without one, and Shoot N Scoot's sampler.
pub(crate) fn phase_unknown_policy(st: &BorgState) -> bool {
    param(Param::escape_phase_unknown_scary) != 0
        && unknown_counts_scary(st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), param(Param::escape_phase_unknown_hp_percent))
}

/// Review of #1075 (finding 2): the never-seen gate of a Phase Door site.
/// It applies only to the checks that sample no landing (`MonsterNear`,
/// `None`; `Landing` counts the unknown itself, and Shoot N Scoot's sampler
/// does), only under the policy, and asks `count` only then: the
/// never-seen count of 100 and the tolerance to judge it by. Returns both
/// when it refuses (the count above the tolerance).
fn unknown_gate(check: PhaseCheck, policy: bool, count: impl FnOnce() -> (i32, i32)) -> Option<(i32, i32)> {
    if !policy || !matches!(check, PhaseCheck::MonsterNear | PhaseCheck::None) {
        return None;
    }
    let (n, at_most) = count();
    (n > at_most).then_some((n, at_most))
}

/// Re-review of #1075 (finding 2): the never-seen count the gate judges, and
/// its tolerance. The last landing check's (`record`) when it was made in
/// this think (`now`) at this grid (`at`), judged by the stricter of that
/// site's tolerance and the gate's (`tolerance`; #1094, the re-review's
/// P3): a scroll the landing site refused on the unknown alone stays
/// refused. Else a fresh draw (`sample`) at the gate's tolerance.
fn gate_count(record: Option<(BorgTime, loc, i32, i32)>, now: BorgTime, at: loc, tolerance: i32, sample: impl FnOnce() -> i32) -> (i32, i32) {
    match record {
        Some((t, grid, n, site)) if t == now && grid == at => (n, site.min(tolerance)),
        _ => (sample(), tolerance),
    }
}

/// Review of #1075 (finding 2): of `escape_landing_samples` Phase Door
/// landings drawn as `count_scary` draws them (range 10, at least 5), how
/// many first meet a legal grid the Borg has never seen. Known walls,
/// monsters and webs are drawn again, as there; a sample with no grid found
/// is not counted (only the unknown is asked here).
fn never_seen_landings(st: &mut BorgState) -> i32 {
    let dis = 10;
    let min = dis / 2;
    let mut n = 0;
    for _ in 0..param(Param::escape_landing_samples) {
        let mut i = 0;
        while i < param(Param::escape_landing_trials) {
            let (x, y) = loop {
                let y = rand_spread(st.borg.c.y, dis);
                let x = rand_spread(st.borg.c.x, dis);
                let d = engine::distance(st.borg.c, loc { x, y });
                if d >= min && d <= dis {
                    break (x, y);
                }
            };
            if y <= 0 || y >= AUTO_MAX_Y - 1 || x <= 0 || x >= AUTO_MAX_X - 1 {
                i += 1;
                continue;
            }
            let g = *st.cave.grid(x, y);
            if u32::from(g.feat) == FEAT_NONE {
                n += 1;
                break;
            }
            if !borg_cave_floor_bold(st, y, x) || g.kill != 0 || g.web {
                i += 1;
                continue;
            }
            break;
        }
    }
    n
}

/// borg-reborn-rs (#1066): whether the Phase Door landing check counts a
/// grid never seen as scary: HP at or above `percent` of the maximum. Below
/// it, a Phase Door into the unknown is a real emergency's, and unknown
/// grids are skipped as before.
fn unknown_counts_scary(hp: i32, max_hp: i32, percent: i32) -> bool {
    i64::from(hp) * 100 >= i64::from(max_hp) * i64::from(percent)
}

/// borg-reborn-rs (#724): whether `borg_caution_teleport` lacks the ability:
/// with `either` (upstream 95387b06) neither a teleport scroll or spell nor
/// a teleport staff or activation; without it (C), not both.
fn lacks_teleport(either: bool, teleport: i32, escape: i32) -> bool {
    if either {
        teleport == 0 && escape == 0
    } else {
        teleport == 0 || escape == 0
    }
}

/// Help determine if "Teleport" seems like a good idea
pub fn borg_caution_teleport(st: &mut BorgState, emergency: i32, turns: i32) -> bool {
    // Extract panel
    let q_x = st.core.w_x / borg_panel_wid();
    let q_y = st.core.w_y / borg_panel_hgt();

    // must have the ability
    // borg-reborn-rs (#724): upstream Angband 95387b06 needs a teleport
    // scroll or spell (BI_ATELEPORT) **or** a teleport staff or activation
    // (BI_AESCAPE). C (and borg-rs) needed both, so with only one of them the
    // landing check never ran, and the Destruction and Teleport Level
    // defences that ask it never saw a safe landing.
    if lacks_teleport(param(Param::escape_teleport_landing_either) != 0, st.borg.tr(BI_ATELEPORT), st.borg.tr(BI_AESCAPE)) {
        return false;
    }

    // Skip unknown grids if explored, or been on level for a while,
    // otherwise, consider ok
    let (n, _) = count_scary(st, 100, turns, false, |st, x, y| {
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
    // borg-reborn-rs (#829): known from the item's curses (`BI_CRSNOTEL`), or
    // learned from the engine's own messages (`TeleportBlock`).
    if teleport_curse(st) {
        return false;
    }

    true
}

/// borg-reborn-rs (#829): what the Borg learned from "Your X prevents you
/// teleporting." / "Teleportation forbidden!": the player cannot teleport
/// (effect-handler-general.c: `effect_handler_TELEPORT`,
/// `effect_handler_TELEPORT_TO`, `effect_handler_TELEPORT_LEVEL` at
/// 574b618c, all `player_of_has(player, OF_NO_TELEPORT)`), whatever the
/// identification state of the item that carries the curse (`BI_CRSNOTEL` is
/// set only from the curses the Borg's item read shows).
///
/// It lasts until the item that carries it changes, or a Remove Curse takes
/// the curse away.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TeleportBlock {
    /// The item the engine named (`ODESC_BASE`: "Cloak"), if it did.
    pub item: Option<String>,
    /// The equipment the curse may sit on, as it was when learned:
    /// `(slot, kind, artifact, weight)`. The named item's slot alone; every
    /// worn item when the message named none.
    watched: Vec<(usize, u32, u8, i16)>,
}

/// The identity of a worn item that survives learning its runes.
fn worn_identity(st: &BorgState, slot: usize) -> Option<(usize, u32, u8, i16)> {
    let item = st.items.items.get(slot)?;
    (item.iqty != 0).then_some((slot, item.kind, item.art_idx, item.weight))
}

/// `borg_item_desc` of `slot`, as text.
fn worn_desc(st: &BorgState, slot: usize) -> String {
    let desc = &st.items.items[slot].desc;
    let bytes: Vec<u8> = desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// What a message says about teleporting.
#[derive(Debug, PartialEq, Eq)]
enum TeleportMessage {
    /// "Your Cloak prevents you teleporting." (object_property.txt, the
    /// message of `OF_NO_TELEPORT`, printed once, when the rune is learned).
    Prevents(String),
    /// "Teleportation forbidden!": the curse, or a no-teleport grid.
    Forbidden,
    /// "The anti-teleportation curse is removed!" (`remove_object_curse`).
    Removed,
}

fn teleport_message(msg: &[u8]) -> Option<TeleportMessage> {
    let text = String::from_utf8_lossy(msg);
    let text = text.trim_end();
    if text == "The anti-teleportation curse is removed!" {
        return Some(TeleportMessage::Removed);
    }
    if text == "Teleportation forbidden!" {
        return Some(TeleportMessage::Forbidden);
    }
    let name = text.strip_prefix("Your ")?.strip_suffix(" prevents you teleporting.")?;
    Some(TeleportMessage::Prevents(name.to_owned()))
}

/// What `msg` (one parsed message) tells about teleporting
/// (`borg_parse_aux`): learn the block from "Your X prevents you
/// teleporting." and from "Teleportation forbidden!", lift it on "The
/// anti-teleportation curse is removed!".
pub(crate) fn teleport_block_message(st: &mut BorgState, msg: &[u8]) {
    if param(Param::escape_teleport_block) == 0 {
        return;
    }
    match teleport_message(msg) {
        Some(TeleportMessage::Removed) => {
            if st.borg.teleport_block.take().is_some() {
                borg_note(st, "# Teleport block lifted: the anti-teleportation curse is removed");
            }
        }
        Some(TeleportMessage::Prevents(name)) => learn_teleport_block(st, Some(&name)),
        Some(TeleportMessage::Forbidden) => {
            // A no-teleport grid forbids the long teleports as well; the grid
            // is asked for by `borg_allow_teleport`, and leaving it frees the
            // Borg. Phase Door (`dis` 10) is never stopped by a grid.
            if !engine::square_isno_teleport(st.borg.c.x, st.borg.c.y) {
                learn_teleport_block(st, None);
            }
        }
        None => {}
    }
}

impl TeleportBlock {
    /// Whether every item the curse may sit on is still the one worn when it
    /// was learned; `worn` is the identity of a slot now.
    fn holds(&self, worn: impl Fn(usize) -> Option<(usize, u32, u8, i16)>) -> bool {
        self.watched.iter().all(|&w| worn(w.0) == Some(w))
    }
}

fn learn_teleport_block(st: &mut BorgState, name: Option<&str>) {
    if st.borg.teleport_block.is_some() && name.is_none() {
        return;
    }
    let first = engine::z_info().pack_size as usize;
    let slots: Vec<usize> = (first..first + 12).filter(|&s| worn_identity(st, s).is_some()).collect();
    let named: Vec<usize> = match name {
        Some(n) => slots.iter().copied().filter(|&s| worn_desc(st, s).contains(n)).collect(),
        None => Vec::new(),
    };
    // One item the message names, else any of them could be the cursed one.
    let watched: Vec<usize> = if named.len() == 1 { named } else { slots };
    let block = TeleportBlock {
        item: name.map(str::to_owned),
        watched: watched.iter().filter_map(|&s| worn_identity(st, s)).collect(),
    };
    if st.borg.teleport_block.as_ref() != Some(&block) {
        let on = block.item.as_deref().map_or(String::new(), |i| format!(" on {i}"));
        borg_note(st, &format!("# Teleport blocked (anti-teleportation{on}): no phase/teleport"));
    }
    st.borg.teleport_block = Some(block);
}

/// borg-reborn-rs (#829, `escape_teleport_block`): whether a teleport curse
/// stops the Borg's teleports now. Off (0): never, as before.
pub(crate) fn teleport_stopped(st: &mut BorgState) -> bool {
    param(Param::escape_teleport_block) != 0 && teleport_curse(st)
}

/// borg-reborn-rs (#829): a scroll that teleports the player (Phase Door,
/// Teleportation, Teleport Level), read under a teleport curse. The choke
/// point of every site that reads one, `borg_read_scroll`.
pub(crate) fn teleport_scroll_blocked(st: &mut BorgState, sval: i32) -> bool {
    let v = &st.item_val;
    let teleports = sval == v.sv_scroll_phase_door || sval == v.sv_scroll_teleport || sval == v.sv_scroll_teleport_level;
    teleports && teleport_stopped(st)
}

/// borg-reborn-rs (#829): a staff of Teleportation (`borg_use_staff`).
pub(crate) fn teleport_staff_blocked(st: &mut BorgState, sval: i32) -> bool {
    sval == st.item_val.sv_staff_teleportation && teleport_stopped(st)
}

/// borg-reborn-rs (#829): an activation that teleports the player
/// (`borg_activate_item`).
pub(crate) fn teleport_activation_blocked(st: &mut BorgState, activation: i32) -> bool {
    let a = &st.act;
    (activation == a.tele_phase || activation == a.tele_long || activation == a.tele_level) && teleport_stopped(st)
}

/// borg-reborn-rs (#829): a spell whose effect teleports the player
/// (class.txt at 574b618c: `TELEPORT` for Phase Door, Teleport Self, Portal
/// and Shadow Shift; `TELEPORT_LEVEL`; `TELEPORT_TO` for Dimension Door), cast
/// by `borg_spell`.
pub(crate) fn teleport_spell_blocked(st: &mut BorgState, spell: crate::borg_headers::borg_magic_h::borg_spells) -> bool {
    let teleports = [PHASE_DOOR, TELEPORT_SELF, TELEPORT_LEVEL, DIMENSION_DOOR, PORTAL, SHADOW_SHIFT].contains(&spell);
    teleports && teleport_stopped(st)
}

/// borg-reborn-rs (#829): whether a teleport curse stops the Borg now: one it
/// knows from its items (`BI_CRSNOTEL`) or one the engine's messages told it
/// about, while the item that carries it is still worn.
pub fn teleport_curse(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_CRSNOTEL) != 0 {
        return true;
    }
    let Some(block) = &st.borg.teleport_block else { return false };
    if block.holds(|slot| worn_identity(st, slot)) {
        return true;
    }
    st.borg.teleport_block = None;
    borg_note(st, "# Teleport block lifted: the equipment changed");
    false
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
    // borg-reborn-rs (#603): a pending Recall blocks it only without
    // caution_recall_wait_as_usual (the countdown survives the level change).
    if !(crate::borg_caution::recall_restricts(st) || st.borg.goal.descending != 0) {
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
enum EscapeAction {
    Escaped,
    Healed,
}

/// `danger` is the danger where the Borg stands. A potion can take the turn
/// after the safer escapes decline, leaving the escape count and flow intact.
pub fn borg_escape(st: &mut BorgState, danger: i32) -> bool {
    match try_escape(st, danger) {
        Some(EscapeAction::Escaped) => {
            st.borg.escapes = st.borg.escapes.wrapping_add(1);
            st.borg.goal.type_0 = 0;
            true
        }
        Some(EscapeAction::Healed) => true,
        None => false,
    }
}

fn try_escape(st: &mut BorgState, b_q: i32) -> Option<EscapeAction> {
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
        return None;
    }

    // Borgs who are in a sea of runes or trying to build one
    // and mostly healthy stay put
    if st.borg.tr(BI_CDEPTH) == 100 && st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) * param(Param::escape_morgoth_hold_hp_tenths) / 10 {
        // In a sea of runes
        if st.borg.morgoth_position {
            return None;
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
            return None;
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
        return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
        }

        let critical_phase = st.borg.tr(BI_CDEPTH) != 0
            && (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::escape_critical_phase_hp_divisor)
                || b_q > avoid(st, param(Param::escape_emergency_danger_tenths), rb));
        let last_ditch_phase = st.borg.tr(BI_CDEPTH) != 0
            && st.borg.tr(BI_CLEVEL) < param(Param::escape_last_ditch_phase_max_level)
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::escape_last_ditch_phase_hp_divisor);

        // The checked Phase Door and the stronger escapes declined. Buy a
        // turn with healing before attempting a Phase Door without a landing check.
        if (critical_phase || last_ditch_phase)
            && has_phase(st, PhaseSite::Emergency)
            && crate::borg_combat_heal::quaff(st)
        {
            st.borg.trait_[BI_CURSP as usize] = sv_mana;
            return Some(EscapeAction::Healed);
        }

        // emergency phase activation no concern for safety of landing zone.
        if critical_phase
            && phase_door(st, PhaseSite::Critical, PhaseCheck::MonsterNear, Some(b_q), |st| {
                borg_activate_item(st, st.act.tele_phase) || borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
            })
        {
            // Flee!
            phase_not_escape(st);
            borg_note(st, "# Danger Level 1.2  Critical Phase");
            reset_antisummon(st);
            return Some(EscapeAction::Escaped);
        }

        // emergency phase spell
        if last_ditch_phase
            && phase_door(st, PhaseSite::LastDitch, PhaseCheck::MonsterNear, Some(b_q), |st| {
                borg_spell_fail(st, PHASE_DOOR, param(Param::escape_last_ditch_phase_fail)) || borg_spell(st, PORTAL)
            })
        {
            // Flee!
            borg_note(st, "# Danger Level 1.3  Critical Attempt");
            reset_antisummon(st);
            return Some(EscapeAction::Escaped);
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
        return None;
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
        }

        // Use Tport Level after the above attempts failed.
        if borg_teleport_off_level(st) {
            // Flee!
            borg_note(st, "# Danger Level 3.4");
            reset_antisummon(st);

            // Success
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
        return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
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
            return Some(EscapeAction::Escaped);
        }
    }

    None
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

    const ALL: [PhaseSite; 17] = [
        PhaseSite::UnseenBump,
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
    /// the adjacent monsters than they need to kill it; a foe it cannot hurt
    /// loses it.
    #[test]
    fn the_fight_is_lost_when_the_borg_dies_first() {
        // The Druid of #612's review: a wild cat with 8 HP left, a 1d3 whip at
        // about 2 a turn: 4 turns; danger 18 on 7 HP: under a turn. Lost.
        assert!(fight_lost_by(&[(8, 2)], 18, 7));
        // 20 HP at 5 a turn: 4 turns; 10 danger on 40 HP: 4 turns. Not lost.
        assert!(!fight_lost_by(&[(20, 5)], 10, 40));
        assert!(fight_lost_by(&[(20, 5)], 12, 40), "3.3 turns to die");
        // Two foes are killed one after another: 2 + 2 turns.
        assert!(fight_lost_by(&[(10, 5), (10, 5)], 12, 40));
        assert!(!fight_lost_by(&[(10, 5), (10, 5)], 10, 40));
        assert!(fight_lost_by(&[(5, 0)], 1, 100), "no damage");
        assert!(!fight_lost_by(&[(20, 5)], 0, 40), "no danger");
        assert!(!fight_lost_by(&[], 50, 10), "no foe");
    }

    /// #625 (owner): the chosen attack counts at its success chance, for the
    /// uses the mana pays for, and then the best attack that is not a spell.
    #[test]
    fn the_fight_counts_the_mana_and_the_fail_rate() {
        use crate::borg_fight_attack::AttackCost;
        let spell = |fail_pct, uses| Some(AttackCost { fail_pct, uses: Some(uses), spell: true });
        // A bolt of 10 at 20% fail: 8 a turn expected.
        assert_eq!(Primary::of(10, spell(20, 3)), Primary { dam: 8.0, uses: Some(3) });
        // Melee, missiles, throws: full damage, no limit.
        assert_eq!(Primary::of(6, None), Primary { dam: 6.0, uses: None });
        // A rod: fail only.
        let rod = Some(AttackCost { fail_pct: 50, uses: None, spell: false });
        assert_eq!(Primary::of(10, rod), Primary { dam: 5.0, uses: None });
        // 40 HP left, bolt 8 a turn for 5 casts: 5 turns.
        let p = Primary { dam: 8.0, uses: Some(5) };
        assert_eq!(fight_turns(&[(40, 2)], p, 0), Some(5.0));
        // Mana for 1 cast: 8 damage, then 32 at melee 2: 1 + 16 turns.
        let p1 = Primary { dam: 8.0, uses: Some(1) };
        assert_eq!(fight_turns(&[(40, 2)], p1, 0), Some(17.0));
        // The fallback (a sling, 4) beats melee after the mana: 1 + 8.
        assert_eq!(fight_turns(&[(40, 2)], p1, 4), Some(9.0));
        // The uses carry over foes: 0.5 cast on the first, 0.5 on the second.
        assert_eq!(fight_turns(&[(4, 2), (8, 2)], p1, 0), Some(0.5 + 0.5 + 2.0));
        // No mana and no other damage: the Borg cannot hurt it.
        assert_eq!(fight_turns(&[(10, 0)], Primary { dam: 8.0, uses: Some(0) }, 0), None);
        // Melee better than the expected spell: melee every turn, mana untouched.
        assert_eq!(fight_turns(&[(12, 6)], Primary { dam: 4.0, uses: Some(9) }, 0), Some(2.0));
        assert_eq!(fight_turns(&[], p1, 0), Some(0.0), "no foe");
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

#[cfg(test)]
mod landing_tests {
    use super::lacks_teleport;

    /// #724 (upstream 95387b06): either kind of teleport is enough.
    #[test]
    fn either_kind_of_teleport_checks_the_landing() {
        for (t, e) in [(1, 0), (0, 1), (1, 1)] {
            assert!(!lacks_teleport(true, t, e), "upstream: {t} {e}");
        }
        assert!(lacks_teleport(true, 0, 0), "upstream: neither");
        // C: both were needed.
        assert!(lacks_teleport(false, 1, 0));
        assert!(lacks_teleport(false, 0, 1));
        assert!(!lacks_teleport(false, 1, 1));
    }
}

#[cfg(test)]
mod unknown_landing_tests {
    use super::unknown_counts_scary;

    /// #1066: unknown landing grids are scary at or above the HP floor, and
    /// skipped (a real emergency) below it.
    #[test]
    fn unknown_grids_are_scary_above_the_hp_floor() {
        assert!(unknown_counts_scary(41, 55, 50), "the Priest 694417276 at 41/55");
        assert!(unknown_counts_scary(43, 60, 50), "the Blackguard 2025089708 at 43/60");
        assert!(unknown_counts_scary(25, 50, 50), "exactly half");
        assert!(!unknown_counts_scary(24, 50, 50), "below half: Phase Door into the unknown allowed");
        assert!(unknown_counts_scary(0, 50, 0), "0 = always scary");
        assert!(!unknown_counts_scary(55, 55, 101), "above 100: never");
    }

    /// #1066: `count_scary` and its callers need the engine (its RNG, the
    /// cave bounds), so this pins their sources (the `recall_restricts`
    /// pattern): only the Phase Door check passes the switch, the teleport
    /// check passes `false`, and an unknown grid is counted before the wall
    /// and danger tests can resample it.
    #[test]
    fn only_the_phase_door_landing_counts_unknown_grids() {
        let src = include_str!("borg_escape.rs");
        let body = |head: &str| {
            let at = &src[src.find(head).unwrap_or_else(|| panic!("{head}"))..];
            at[..at.find("\n}\n").unwrap()].to_string()
        };
        let phase = body(concat!("fn caution_", "phase(st: &mut BorgState"));
        let teleport = body(concat!("pub fn borg_caution_", "teleport("));
        assert!(phase.contains(concat!("count_scary(st, 10, turns, unknown_", "scary,")));
        assert!(phase.contains(concat!("phase_unknown_", "policy(st)")));
        let policy = body(concat!("pub(crate) fn phase_unknown_", "policy("));
        assert!(policy.contains(concat!("Param::escape_phase_", "unknown_scary")));
        assert!(policy.contains(concat!("Param::escape_phase_", "unknown_hp_percent")));
        // Review of #1075 (finding 2): Shoot N Scoot's sampler asks the policy.
        let flow = include_str!("borg_flow_kill.rs");
        let at = &flow[flow.find(concat!("pub fn borg_shoot_scoot_", "safe(")).unwrap()..];
        let scoot = &at[..at.find("\n}\n").unwrap()];
        assert!(scoot.contains(concat!("crate::borg_escape::phase_unknown_", "policy(st)")));
        assert!(scoot.find(concat!("never_seen = ", "true")).unwrap() < scoot.find(concat!("// Skip ", "walls")).unwrap());
        assert!(teleport.contains(concat!("count_scary(st, 100, turns, ", "false,")));
        let count = body(concat!("fn count_", "scary("));
        let unknown = count.find(concat!("never_seen = ", "true")).unwrap();
        assert!(unknown < count.find(concat!("// Skip ", "walls")).unwrap());
        assert!(count.find(concat!("if never_", "seen {")).unwrap() < count.find(concat!("let p = borg_", "danger(")).unwrap());
    }

    /// Review of #1075 (finding 2): the emergency fall-through. When the
    /// emergency site's landing check refuses, the critical Phase Door
    /// (`MonsterNear`, no landing check) asks the never-seen gate: under the
    /// policy (the Blackguard 2025089708 at 43/60) 88 never-seen landings of
    /// 100 refuse it; below the HP floor (15/60: no policy) the scroll is
    /// read, and the sampler is not even run. Sites that sample a landing
    /// themselves are not gated twice.
    #[test]
    fn the_critical_phase_after_a_refused_emergency_asks_the_gate() {
        use super::{unknown_counts_scary, unknown_gate, PhaseCheck};
        let landing = PhaseCheck::Landing { tolerance: 75, horizon: 2 };
        let scoot = PhaseCheck::ShootScoot { tolerance: 20, horizon: 2, danger: 50 };
        let mut runs = 0;
        let policy = unknown_counts_scary(43, 60, 50);
        assert_eq!(unknown_gate(PhaseCheck::MonsterNear, policy, || { runs += 1; (88, 75) }), Some((88, 75)), "critical, above the floor: refused");
        assert_eq!(unknown_gate(PhaseCheck::MonsterNear, policy, || { runs += 1; (75, 75) }), None, "at the tolerance: allowed");
        assert_eq!(unknown_gate(PhaseCheck::None, policy, || { runs += 1; (90, 75) }), Some((90, 75)), "twitchy and fast-fragile too");
        assert_eq!(runs, 3);
        let low = unknown_counts_scary(15, 60, 50);
        assert_eq!(unknown_gate(PhaseCheck::MonsterNear, low, || { runs += 1; (100, 75) }), None, "below the floor: a real emergency");
        assert_eq!(unknown_gate(landing, policy, || { runs += 1; (100, 75) }), None, "the landing check counts the unknown itself");
        assert_eq!(unknown_gate(scoot, policy, || { runs += 1; (100, 75) }), None, "so does Shoot N Scoot's sampler");
        assert_eq!(runs, 3, "the sampler runs only where the gate applies");
        // The order in `phase_door`: the site's own check, then the gate,
        // then the cast.
        let src = include_str!("borg_escape.rs");
        let at = &src[src.find(concat!("pub(crate) fn phase_", "door(")).unwrap()..];
        let door = &at[..at.find("\n}\n").unwrap()];
        let gate = door.find(concat!("unknown_gate(check, ", "policy,")).unwrap();
        assert!(door.find(concat!("if !checked ", "{")).unwrap() < gate && gate < door.find(concat!("let done = ", "cast(st)")).unwrap());
    }

    /// Re-review of #1075 (finding 2): the emergency check refused on 76
    /// never-seen landings of 100; the critical fall-through in the same
    /// think and grid reuses that count, so a lower redraw (75) cannot read
    /// the same scroll. Another think, or another grid, draws again.
    #[test]
    fn a_refused_emergency_is_not_undone_by_a_lower_redraw() {
        use super::{gate_count, unknown_gate, PhaseCheck};
        let here = super::loc { x: 25, y: 125 };
        let record = Some((40580, here, 76, 75));
        let mut draws = 0;
        let critical = |record, now, at, draws: &mut i32| {
            unknown_gate(PhaseCheck::MonsterNear, true, || gate_count(record, now, at, 75, || { *draws += 1; 75 }))
        };
        assert_eq!(critical(record, 40580, here, &mut draws), Some((76, 75)), "the emergency's verdict carries over");
        assert_eq!(draws, 0, "no redraw");
        assert_eq!(critical(record, 40590, here, &mut draws), None, "the next think draws again");
        assert_eq!(critical(record, 40580, super::loc { x: 26, y: 125 }, &mut draws), None, "another grid draws again");
        assert_eq!(critical(None, 40580, here, &mut draws), None, "no landing check this think");
        assert_eq!(draws, 3);
        // The landing check records its count under the policy.
        let src = include_str!("borg_escape.rs");
        let at = &src[src.find(concat!("fn caution_", "phase(st: &mut BorgState")).unwrap()..];
        let phase = &at[..at.find("\n}\n").unwrap()];
        assert!(phase.contains(concat!("st.borg.phase_unknown_seen = ", "Some((st.borg.time.now, st.borg.c, unknown, emergency))")));
    }

    /// #1094 (review of #1096): a never-seen refusal stops only the Phase
    /// Door. `phase_door` clears the flag on entry and sets it only when the
    /// gate refuses; the twitchy think then tries its longer-range teleports
    /// (Dimension Door, Teleport Self, Portal, Shadow Shift, in the order
    /// they had inside the cast) and no others. Any other refusal (the
    /// teleport curse, the #584 rule) still stops them. Needs the engine to
    /// play, so this pins the sources.
    #[test]
    fn a_gate_refusal_leaves_the_twitchy_teleports() {
        let src = include_str!("borg_escape.rs");
        let at = &src[src.find(concat!("pub(crate) fn phase_", "door(")).unwrap()..];
        let door = &at[..at.find("\n}\n").unwrap()];
        let clear = door.find(concat!("st.borg.phase_gate_refused = ", "None;")).unwrap();
        let set = door.find(concat!("st.borg.phase_gate_refused = ", "Some(st.borg.time.now);")).unwrap();
        assert!(clear < door.find(concat!("if teleport_", "stopped(st)")).unwrap(), "cleared before any refusal");
        assert!(door.find(concat!("unknown_gate(check, ", "policy,")).unwrap() < set, "set only by the gate");
        assert_eq!(door.matches(concat!("phase_gate_refused = ", "Some(")).count(), 1);
        let think = include_str!("borg_think_dungeon.rs");
        let at = &think[think.find(concat!("PhaseSite::Twitchy", "Think")).unwrap()..];
        let twitch = &at[..at.find(concat!("return choice!(\"twitch_", "phase\")")).unwrap()];
        assert!(twitch.contains(concat!("|| longer", "(st)\n")), "inside the cast, after the Phase Door ways");
        assert!(twitch.contains(concat!("st.borg.phase_gate_refused == Some(st.borg.time.now) && ", "longer(st)")));
        let before = &think[..think.find(concat!("PhaseSite::Twitchy", "Think")).unwrap()];
        let longer = &before[before.rfind(concat!("let longer = ", "|st")).unwrap()..];
        let order = ["borg_dimension_door(st, 90)", "borg_spell(st, TELEPORT_SELF)", "borg_spell(st, PORTAL)", "borg_shadow_shift(st, 90)"];
        let at: Vec<usize> = order.iter().map(|o| longer.find(o).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "the original order");
    }

    /// #1094 (re-review of #1075, P3): the emergency tolerance 50, the gate's
    /// 75, 60 never-seen landings. The emergency refused on the unknown
    /// alone, so the critical fall-through in the same think keeps the
    /// stricter 50 and refuses too. A looser landing site does not loosen
    /// the gate, and without an earlier check the gate's own 75 applies.
    #[test]
    fn the_stricter_tolerance_carries_into_the_fall_through() {
        use super::{gate_count, unknown_gate, PhaseCheck};
        let here = super::loc { x: 25, y: 125 };
        let gate = |record, tolerance, fresh: i32| {
            unknown_gate(PhaseCheck::MonsterNear, true, || gate_count(record, 40580, here, tolerance, || fresh))
        };
        assert_eq!(gate(Some((40580, here, 60, 50)), 75, 0), Some((60, 50)), "the emergency's 50 is kept");
        assert_eq!(gate(Some((40580, here, 60, 90)), 75, 0), None, "a looser site does not loosen the gate: 60 <= 75");
        assert_eq!(gate(Some((40580, here, 80, 90)), 75, 0), Some((80, 75)), "the gate's own 75 still binds");
        assert_eq!(gate(Some((40570, here, 60, 50)), 75, 60), None, "another think: a fresh draw at the gate's 75");
        assert_eq!(gate(None, 75, 76), Some((76, 75)), "no earlier check: the gate's own tolerance");
    }

    /// Re-review of #1075 (finding 1): an infrequent lethal cast vetoes the
    /// wait. A 60 HP spell from a caster casting 1 turn in 10 (below the
    /// 11% band) is 24 with the frequency discount (4 tenths) and 60 at full
    /// damage; at 40 HP with 10 HP a turn from the adjacent monsters, the
    /// discounted danger would let the wait through, the full one does not.
    /// `newcomer_wait` asks the full damage.
    #[test]
    fn an_infrequent_lethal_cast_vetoes_the_wait() {
        use super::newcomer_turn_safe;
        use crate::borg_danger::spell_frequency_discount;
        let discounted = spell_frequency_discount(60, 10, false);
        let full = spell_frequency_discount(60, 10, true);
        assert_eq!((discounted, full), (24, 60));
        assert!(newcomer_turn_safe(10.0, i64::from(discounted), 40, 100), "the discount would wait");
        assert!(!newcomer_turn_safe(10.0, i64::from(full), 40, 100), "the full damage keeps the Phase Door");
        assert_eq!(spell_frequency_discount(60, 60, false), 60, "a frequent caster (51% or more) is not discounted");
        let src = include_str!("borg_escape.rs");
        let at = &src[src.find(concat!("fn newcomer_", "wait(")).unwrap()..];
        let wait = &at[..at.find("\n}\n").unwrap()];
        assert!(wait.contains(concat!("borg_danger_one_kill(st, y, x, 1, i as i32, ", "true, true)")));
    }

    /// Review of #1075 (finding 1): the wait needs the next turn
    /// survivable. The Priest 694417276 at 41 HP: the soldier's 13.532 a
    /// turn; a ranged danger of 27 or more makes the next turn unsafe at
    /// 100%.
    #[test]
    fn the_newcomer_wait_needs_a_survivable_next_turn() {
        use super::newcomer_turn_safe;
        assert!(newcomer_turn_safe(13.532, 27, 41, 100));
        assert!(!newcomer_turn_safe(13.532, 28, 41, 100));
        assert!(!newcomer_turn_safe(13.532, 0, 13, 100), "the adjacent alone");
        assert!(newcomer_turn_safe(13.532, 28, 41, 200), "a looser percent");
        assert!(!newcomer_turn_safe(f64::INFINITY, 0, 41, 100));
    }

    /// #1066: the newcomer wait is asked only after the rule judged the
    /// fight lost, and it races the adjacent monsters alone.
    #[test]
    fn the_newcomer_wait_follows_a_lost_verdict() {
        let src = include_str!("borg_escape.rs");
        // #1125: the fight's test sits in `phase_lost`, asked only with an
        // awake enemy adjacent (and no paralyser, which loses it anyway).
        assert!(src.contains(concat!("phase_lost(adjacent, ", "paralyser.is_some(), || {")));
        assert!(src.contains(concat!("fight_lost(st, here, fight_hp) && !newcomer_", "wait(st, fight_hp)")));
        assert!(super::unknown_counts_scary(45, 60, 50));
        assert!(!super::unknown_counts_scary(45 - 40, 60, 50));
        assert!(src.contains(concat!("adjacent && (paralyser || ", "fight())")));
        let at = &src[src.find(concat!("fn newcomer_", "wait(")).unwrap()..];
        let wait = &at[..at.find("\n}\n").unwrap()];
        assert!(wait.contains(concat!("fight_race(st, ", "true)")));
        assert!(wait.contains(concat!("Param::escape_phase_", "newcomer_hp_percent")));
        assert!(wait.contains(concat!("race.won && ", "safe")), "review of #1075: winning is not enough");
        let caution = include_str!("borg_caution.rs");
        let at = &caution[caution.find(concat!("pub(crate) fn fight_", "race(")).unwrap()..];
        let race = &at[..at.find("\n}\n").unwrap()];
        let skip = race.find(concat!("if adjacent_only && ", "d > 1")).unwrap();
        assert!(race.find(concat!("return None; // an attacker without", " a current")).unwrap() < skip, "an unseen attacker anywhere still refuses");
    }
}


/// borg-reborn-rs (#781 review): confirmed bump supplies adjacency in place
/// of a known race. Keep the ordinary emergency landing tolerance/horizon.
pub(crate) fn unseen_bump_phase(st: &mut BorgState, danger: i32) -> bool {
    phase_door(st, PhaseSite::UnseenBump, PhaseCheck::Landing {
        tolerance: param(Param::escape_emergency_phase_tolerance),
        horizon: param(Param::escape_emergency_phase_horizon),
    }, Some(danger), |st| {
        borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
            || borg_activate_item(st, st.act.tele_phase) || borg_spell(st, PHASE_DOOR)
    })
}

/// borg-reborn-rs (#829): what the Borg learns of a teleport curse.
#[cfg(test)]
mod teleport_block_tests {
    use super::*;

    #[test]
    fn the_engine_s_teleport_messages_are_understood() {
        // The lines of Priest 354395849 (messages.log 15401-15403).
        assert_eq!(teleport_message(b"Your Cloak prevents you teleporting."), Some(TeleportMessage::Prevents("Cloak".into())));
        assert_eq!(
            teleport_message(b"Your Pair of Leather Sandals prevents you teleporting."),
            Some(TeleportMessage::Prevents("Pair of Leather Sandals".into()))
        );
        assert_eq!(teleport_message(b"Teleportation forbidden!"), Some(TeleportMessage::Forbidden));
        assert_eq!(teleport_message(b"The anti-teleportation curse is removed!"), Some(TeleportMessage::Removed));
        // Nothing else is one.
        assert_eq!(teleport_message(b"You have learned the rune of anti-teleportation curse."), None);
        assert_eq!(teleport_message(b"The vulnerability curse is removed!"), None);
        assert_eq!(teleport_message(b"Your Cloak glows."), None);
        assert_eq!(teleport_message(b""), None);
    }

    #[test]
    fn the_block_lasts_while_the_watched_items_are_worn_and_ends_when_one_changes() {
        // (slot, kind, artifact, weight): the Priest's Cloak in slot 28.
        let cloak = (28, 7, 0, 10);
        let block = TeleportBlock { item: Some("Cloak".into()), watched: vec![cloak] };
        assert!(block.holds(|_| Some(cloak)), "the same Cloak");
        assert!(!block.holds(|_| None), "taken off");
        assert!(!block.holds(|_| Some((28, 8, 0, 10))), "another kind of Cloak");
        assert!(!block.holds(|_| Some((28, 7, 3, 10))), "an artifact in its place");
        // A message that named no item watches everything worn.
        let any = TeleportBlock { item: None, watched: vec![(24, 1, 0, 50), cloak] };
        assert!(any.holds(|s| if s == 24 { Some((24, 1, 0, 50)) } else { Some(cloak) }));
        assert!(!any.holds(|s| if s == 24 { Some((24, 2, 0, 50)) } else { Some(cloak) }), "the weapon changed");
    }
}

#[cfg(test)]
mod paralyser_phase_tests {
    use super::phase_lost;

    /// #1125, point 1: the floating eye of 453254028 (danger 0 without
    /// fear, HP 7): the fight test says "not lost", the paralyser says lost.
    #[test]
    fn next_to_a_paralyser_the_fight_is_lost() {
        assert!(phase_lost(true, true, || false));
        assert!(!phase_lost(true, false, || false), "today's verdict");
        assert!(phase_lost(true, false, || true));
        assert!(!phase_lost(false, true, || true), "no awake enemy adjacent: no Phase Door");
        let mut asked = false;
        assert!(phase_lost(true, true, || {
            asked = true;
            false
        }));
        assert!(!asked, "the fight's test (and its notes) is skipped");
    }

    /// Review of #1129: the paralyser's deferral comes before every early
    /// fight, and every flow destination is checked for a paralyser's reach.
    #[test]
    fn the_early_fight_and_the_flow_goals_respect_the_paralyser() {
        let caution = include_str!("borg_caution.rs");
        let at = &caution[caution.find(concat!("pub(crate) fn fight_before_", "flight(")).unwrap()..];
        let body = &at[..at.find("\n}\n").unwrap()];
        let defer = body.find(concat!("paralyser_defers_", "early_fight(st)")).expect("the deferral");
        for early in [concat!("attack_on_", "sight(st)"), concat!("borg_lone_target::", "attack(st)"), concat!("winning_", "fight(st)")] {
            assert!(defer < body.find(early).expect(early), "{early} comes after the deferral");
        }
        let flow = include_str!("borg_flow.rs");
        let at = &flow[flow.find(concat!("pub fn borg_flow_enqueue_", "grid(")).unwrap()..];
        let body = &at[..at.find("\n}\n").unwrap()];
        let reach = body.find(concat!("in_paralyser_", "reach(st, x, y)")).expect("the seed check");
        assert!(reach < body.find(concat!("data_cost[y as usize][x as usize] = ", "0")).unwrap(), "checked before the seed");
    }
}

#[cfg(test)]
mod walk_away_phase_tests {
    use super::{phase_available, PhaseSite};

    #[test]
    fn a_cornered_caster_counts_its_spell_up_to_the_sites_ceiling() {
        // Re-review of #1148: a 20% Phase Door spell, no scroll, no
        // activation (BI_APHASE 0): the cornered walk-away counts it (the
        // spell test passes at its 30% ceiling); every other site does not.
        assert!(phase_available(0, PhaseSite::WalkAwayCornered, || true));
        assert!(!phase_available(0, PhaseSite::WalkAwayCornered, || false), "above the ceiling");
        assert!(!phase_available(0, PhaseSite::Danger, || true), "other sites keep BI_APHASE");
        assert!(phase_available(1000, PhaseSite::Danger, || false));
    }
}

/// Availability of the same four sources as `borg_recall`, before issuing keys.
pub(crate) fn recall_ready(st: &BorgState) -> bool {
    use crate::borg_inventory::borg_slot;
    use crate::borg_item_use::{borg_equips_item, can_read_now};
    use crate::headers::list_tvals_h::{TV_ROD, TV_SCROLL};

    let rod = borg_slot(st, TV_ROD as i32, st.item_val.sv_rod_recall);
    (rod >= 0 && st.items.items[rod as usize].pval != 0)
        || borg_equips_item(st, st.act.recall, true)
        || borg_spell_okay_fail(st, WORD_OF_RECALL, param(Param::escape_recall_fail))
        || (can_read_now(st) && borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_word_of_recall) >= 0)
}

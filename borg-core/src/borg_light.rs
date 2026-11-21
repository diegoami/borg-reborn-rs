//! Port of `borg-light.c`: handle light and lighting.
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
use crate::borg_cave_light::BORG_LIGHT;
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid};
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes};
use crate::borg_item::{inven_wield, BorgItem};
use crate::borg_item_use::{
    borg_activate_item, borg_aim_wand, borg_equips_rod, borg_read_scroll, borg_use_staff,
    borg_zap_rod,
};
use crate::borg_magic::{borg_get_spell_power, borg_spell, borg_spell_fail, borg_spell_legal_fail, borg_spell_okay_fail};
use crate::borg_messages_react::borg_react;
use crate::borg_update::Detect;
use crate::engine;
use crate::params::{value as param, Param};
use crate::borg_headers::borg_magic_h::{
    CALL_LIGHT, CLAIRVOYANCE, CREATE_DARKNESS, DETECTION, DETECT_EVIL, DETECT_MONSTERS,
    DETECT_STAIRS, FIND_TRAPS_DOORS_STAIRS, FUME_OF_MORDOR, LIGHT_ROOM, OBJECT_DETECTION,
    MAGIC_MISSILE, NETHER_BOLT, PHASE_DOOR, READ_MINDS, SEEK_BATTLE, SENSE_SURROUNDINGS,
    SPEAR_OF_LIGHT, TELEPORT_SELF,
};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLASS, BI_CLEVEL, BI_ESP, BI_ISBLIND, BI_ISCONFUSED, BI_ISCUT, BI_ISIMAGE, BI_ISPOISONED,
    BI_CURSP, BI_ISWEAK, BI_LIGHT, BI_MAXCLEVEL,
};
use crate::headers::list_object_flags_h::{OF_LIGHT_2, OF_LIGHT_3, OF_NO_FUEL, OF_STICKY, OF_TAKES_FUEL};
use crate::headers::list_object_modifiers_h::OBJ_MOD_LIGHT;
use crate::headers::list_terrain_h::FEAT_NONE;
use crate::headers::list_tvals_h::{TV_FLASK, TV_LIGHT, TV_WAND};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `CLASS_NECROMANCER` (list-classes.h order).
const CLASS_NECROMANCER: i32 = 4;

/// Grid info flag: ignore the game map for this grid (`BORG_IGNORE_MAP`).
const BORG_IGNORE_MAP: u16 = 0x100;

/// The result of `borg_maintain_light` (`enum borg_need`, same values).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(clippy::enum_variant_names)] // C's BORG_NO_NEED / BORG_MET_NEED / BORG_UNMET_NEED
pub enum BorgNeed {
    /// `BORG_NO_NEED`
    NoNeed = 0,
    /// `BORG_MET_NEED`
    MetNeed = 1,
    /// `BORG_UNMET_NEED`
    UnmetNeed = 2,
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `!t || borg_timer(t) >= n`: never done, or at least `n` turns ago.
fn due(st: &BorgState, t: BorgTime, n: BorgTime) -> bool {
    t == 0 || borg_timer(st, t) >= n
}

/// `INVEN_LIGHT`: the light source slot.
fn inven_light() -> i32 {
    inven_wield() + 5
}

/// `all_letters_nohjkl[i]` as a key (a C `char`, sign-extended to
/// `keycode_t`).
fn letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// A key for a character literal.
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// True if one of the four panels around (q_x, q_y) is not yet marked in
/// the given detection array (the four `if (!borg_detect_x[..][..])`
/// tests; all four are read, as in C).
fn undetected(st: &mut BorgState, what: Detect, q_y: i32, q_x: i32) -> bool {
    let a = st.update.detect(what, q_y, q_x);
    let b = st.update.detect(what, q_y, q_x + 1);
    let c = st.update.detect(what, q_y + 1, q_x);
    let d = st.update.detect(what, q_y + 1, q_x + 1);
    !a || !b || !c || !d
}

/// Check to see if the surrounding dungeon should be darkened
/// (`borg_check_dark_only`). This is only done for necromancers.
fn borg_check_dark_only(st: &mut BorgState) -> bool {
    /* Only necromancers like the dark */
    if st.borg.tr(BI_CLASS) != CLASS_NECROMANCER {
        return false;
    }

    /* Don't bother because we only just did it */
    /* necromancers borrow the call light counter for darkness */
    if st.borg.time.call_light != 0 && borg_timer(st, st.borg.time.call_light) < 7 {
        return false;
    }
    let mut floors = 0;

    /* Scan the surrounding 5x5 area for lit tiles. */
    let (cx, cy) = (st.borg.c.x, st.borg.c.y);
    for y in cy - 2..=cy + 2 {
        for x in cx - 2..=cx + 2 {
            /* Bounds check */
            if !engine::square_in_bounds_fully(x, y) {
                continue;
            }

            /* Must be a glowing floor grid */
            if borg_cave_floor_grid(st.cave.grid(x, y)) && engine::square_isglow(x, y) {
                floors += 1;
            }
        }
    }

    /* Don't bother unless there are enough unlit floors */
    /* 11 is the empirical cutoff point for sensible behaviour here */
    if floors < 11 {
        return false;
    }

    if borg_spell_fail(st, CREATE_DARKNESS, 40) {
        borg_note(st, "# Calling Darkness in the dungeon");
        st.borg.time.call_light = st.borg.time.now;
        return true;
    }

    false
}

/// Check to see if the surrounding dungeon should be illuminated, and if
/// it should, do it (`borg_check_light_only`).
///
/// This is when resting to heal.  I don't want him teleporting into a room,
/// resting to heal while there is a dragon sitting in a dark corner waiting
/// to breathe on him.
///
/// Returns true if it did something, false otherwise.
pub fn borg_check_light_only(st: &mut BorgState) -> bool {
    /* Never in town, when blind or when hallucinating */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISIMAGE) != 0 {
        return false;
    }

    /* Use wizard light sometimes */
    if due(st, st.borg.time.wizard_light, 1000)
        && (borg_activate_item(st, st.act.clairvoyance)
            || borg_activate_item(st, st.act.enlightenment)
            || borg_spell_fail(st, FUME_OF_MORDOR, 40)
            || borg_spell_fail(st, CLAIRVOYANCE, 40))
    {
        borg_note(st, "# Wizard lighting the dungeon");
        st.borg.time.wizard_light = st.borg.time.now;
        return true;
    }

    /* necromancers like the dark */
    if st.borg.tr(BI_CLASS) == CLASS_NECROMANCER {
        return borg_check_dark_only(st);
    }

    /* Work out if there's any reason to light */

    /* Don't bother because we only just did it */
    if st.borg.time.call_light != 0 && borg_timer(st, st.borg.time.call_light) < 7 {
        return false;
    }

    let (cx, cy) = (st.borg.c.x, st.borg.c.y);
    if st.borg.tr(BI_LIGHT) == 1 {
        let mut corners = 0;

        /* Scan diagonal neighbors.
         *
         * 4 corners   3 corners    2 corners    1 corner    0 corners
         * ###         ##.  #..     ##.  #..     .#.         .#.  ... .#.
         * .@.         .@.  .@.     .@.  .@.     .@.         #@#  .@. .@.
         * ###         ###  ###     ##.  #..     ##.         .#.  ... .#.
         *
         * There's actually no way to tell which are rooms and which are
         * corridors from diagonals except 4 (always a corridor) and
         * 0 (always a room).  This is why we only use it for radius 1 light.
         */
        for i in 4..8 {
            /* Get location */
            let x = cx + i32::from(engine::ddx_ddd(i));
            let y = cy + i32::from(engine::ddy_ddd(i));

            /* Bounds check */
            if !engine::square_in_bounds_fully(x, y) {
                continue;
            }

            let ag = st.cave.grid(x, y);

            /* Location must be known */
            if u32::from(ag.feat) == FEAT_NONE {
                corners += 1;
            }

            /* Location must be a wall/door */
            if !borg_cave_floor_grid(ag) {
                corners += 1;
            }
        }

        /* This is quite an arbitrary cutoff */
        if corners > 2 {
            return false;
        }
    } else if st.borg.tr(BI_LIGHT) > 1 {
        let mut floors = 0;

        /* Scan the surrounding 5x5 area for unlit tiles.
         *
         * Radius two light misses out the four corners but otherwise
         * illuminates a 5x5 grid, which is 21 grids illuminated incl player.
         */
        for y in cy - 2..=cy + 2 {
            for x in cx - 2..=cx + 2 {
                /* Bounds check */
                if !engine::square_in_bounds_fully(x, y) {
                    continue;
                }

                let ag = st.cave.grid(x, y);

                /* Must be a floor grid lit by torchlight, not by magic */
                if borg_cave_floor_grid(ag)
                    && (ag.info & BORG_LIGHT) != 0
                    && !engine::square_isglow(x, y)
                {
                    floors += 1;
                }
            }
        }

        /* Don't bother unless there are enough unlit floors */
        /* 11 is the empirical cutoff point for sensible behaviour here */
        if floors < 11 {
            return false;
        }
    }

    /* Light it up! */
    if borg_activate_item(st, st.act.illumination)
        || borg_activate_item(st, st.act.light)
        || borg_zap_rod(st, st.item_val.sv_rod_illumination)
        || borg_use_staff(st, st.item_val.sv_staff_light)
        || borg_read_scroll(st, st.item_val.sv_scroll_light)
        || borg_spell_fail(st, LIGHT_ROOM, 40)
        || borg_spell_fail(st, CALL_LIGHT, 40)
    {
        borg_note(st, "# Illuminating the dungeon");
        borg_react(st, b"SELF:lite", b"SELF:lite");
        st.borg.time.call_light = st.borg.time.now;
        return true;
    }

    false
}

/// The arrival square or the forward sector after a single step. A wait,
/// bump, teleport on the same level or redraw is not an entry.
fn entry_direction(previous: Option<(i32, i32)>, current: (i32, i32)) -> Option<(i32, i32)> {
    let Some((py, px)) = previous else { return Some((0, 0)); };
    let (dy, dx) = (current.0 - py, current.1 - px);
    (dy.abs().max(dx.abs()) == 1).then_some((dy, dx))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FrontierGrid {
    Floor,
    Unknown,
    Blocked,
}

fn dark_frontier(
    (y, x): (i32, i32), direction: (i32, i32), radius: i32,
    grid: impl Fn(i32, i32) -> FrontierGrid,
) -> bool {
    let (dy, dx) = direction;
    (-1..=1).any(|sy| (-1..=1).any(|sx| {
        if (sy == 0 && sx == 0) || ((dy != 0 || dx != 0) && sy * dy + sx * dx <= 0) {
            return false;
        }
        grid(y + sy * (radius + 1), x + sx * (radius + 1)) == FrontierGrid::Unknown
            && (1..=radius).all(|step| grid(y + sy * step, x + sx * step) == FrontierGrid::Floor)
    }))
}

fn entering_unexamined_dark(st: &BorgState) -> bool {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let steps = &st.flow.track_step;
    if steps.num == 0 || engine::square_isglow(x, y) {
        return false;
    }
    let previous = (steps.num >= 2).then(|| steps.at(steps.num as usize - 2));
    let Some(direction) = entry_direction(previous, (y, x)) else { return false };
    // The grid just beyond personal light must still be unknown, along a
    // continuous run of known floor. Do not inspect that grid in the engine.
    dark_frontier((y, x), direction, st.borg.tr(BI_LIGHT).max(1), |y, x| {
        if !engine::square_in_bounds_fully(x, y) {
            FrontierGrid::Blocked
        } else if u32::from(st.cave.grid(x, y).feat) == FEAT_NONE {
            FrontierGrid::Unknown
        } else if borg_cave_floor_bold(st, y, x) {
            FrontierGrid::Floor
        } else {
            FrontierGrid::Blocked
        }
    })
}

fn fits_detection_reserve(sp: i32, cost: i32, reserve: i32) -> bool {
    cost >= 0 && sp - cost >= reserve
}

/// #796: the two broad detectors behind #685's level gate. Detect Evil
/// misses non-evil monsters, and Seek Battle misses fear-immune ones (the
/// floating eye is both). Keep those on the ordinary branch.
fn detect_on_dark_entry(st: &mut BorgState, panel: (i32, i32)) -> bool {
    if param(Param::light_detect_dark_entry) == 0 || st.borg.tr(BI_CDEPTH) == 0
        || st.light.detected_entries.contains(&panel)
        || st.borg.goal.fleeing || st.borg.goal.leaving || !entering_unexamined_dark(st)
    {
        return false;
    }
    let Some(response_cost) = [PHASE_DOOR, TELEPORT_SELF, MAGIC_MISSILE, NETHER_BOLT].into_iter()
        .filter(|&spell| borg_spell_legal_fail(st, spell, 40))
        .map(|spell| borg_get_spell_power(st, spell))
        .min()
    else {
        return false;
    };
    let reserve = param(Param::light_detect_dark_entry_reserve_sp).max(response_cost)
        .max(crate::borg_recover::recovery_sp_floor(st));
    let sp = st.borg.tr(BI_CURSP);
    for (spell, name) in [(DETECT_MONSTERS, "Detect Monsters"), (READ_MINDS, "Read Minds")] {
        let cost = borg_get_spell_power(st, spell);
        if fits_detection_reserve(sp, cost, reserve) && borg_spell_fail(st, spell, 40) {
            st.light.detected_entries.insert(panel);
            borg_note(st, &format!("# Dark entry detection: {name} (SP {sp}, cost {cost}, reserve {reserve})"));
            return true;
        }
    }
    false
}

/// Refuel, call lite, detect traps/doors/walls/evil, etc
/// (`borg_check_light`).
///
/// Note that we refuel whenever our lite starts to get low.
///
/// Note that we detect traps/doors/walls/evil at least once in each
/// panel, as soon as possible after entering a new panel.
///
/// We use the special "SELF" messages to "borg_react()" to delay the
/// processing of DETECTION and "call lite" until we know if it has
/// worked or not.
pub fn borg_check_light(st: &mut BorgState) -> bool {
    /* Never in town when mature (scary guy) */
    if st.borg.tr(BI_MAXCLEVEL) > 10 && st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    /* Never when compromised, save your mana */
    if st.borg.tr(BI_ISBLIND) != 0
        || st.borg.tr(BI_ISCONFUSED) != 0
        || st.borg.tr(BI_ISIMAGE) != 0
        || st.borg.tr(BI_ISPOISONED) != 0
        || st.borg.tr(BI_ISCUT) != 0
        || st.borg.tr(BI_ISWEAK) != 0
    {
        return false;
    }

    /* Extract the panel */
    let q_x = st.core.w_x / borg_panel_wid();
    let q_y = st.core.w_y / borg_panel_hgt();

    /* Determine if we need to detect traps, doors, walls, evil, objects */
    let mut do_trap = undetected(st, Detect::Trap, q_y, q_x);
    let mut do_door = undetected(st, Detect::Door, q_y, q_x);
    let mut do_wall = undetected(st, Detect::Wall, q_y, q_x);
    let mut do_evil = undetected(st, Detect::Evil, q_y, q_x);
    let do_obj = undetected(st, Detect::Obj, q_y, q_x);

    /* Dont bother if I have ESP */
    if st.borg.tr(BI_ESP) != 0 {
        do_evil = false;
    }

    /* Only look for monsters in town, not walls, etc (scary guy) */
    if st.borg.tr(BI_CDEPTH) == 0 {
        do_trap = false;
        do_door = false;
        do_wall = false;
    }

    /*** Do Things ***/
    let t = st.borg.time;

    /* Find traps and doors and evil */
    if (do_trap || do_door || do_evil)
        && (due(st, t.detect_traps, 5) || due(st, t.detect_evil, 5) || due(st, t.detect_doors, 5))
        && st.borg.tr(BI_CDEPTH) != 0
        && (borg_activate_item(st, st.act.detect_all)
            || borg_activate_item(st, st.act.mapping)
            || borg_zap_rod(st, st.item_val.sv_rod_detection)
            || borg_spell_fail(st, SENSE_SURROUNDINGS, 40))
    {
        borg_note(st, "# Checking for traps, doors, and evil.");

        borg_react(st, b"SELF:TDE", b"SELF:TDE");

        st.borg.time.detect_traps = st.borg.time.now;
        st.borg.time.detect_doors = st.borg.time.now;
        st.borg.time.detect_evil = st.borg.time.now;
        st.borg.time.detect_obj = st.borg.time.now;

        return true;
    }

    /* Find evil */
    // borg-reborn-rs (#685): below light_detect_evil_spell_min_level the
    // panel check casts no detection spell. A low-level caster's few SP go
    // to it on every new panel and it then rests for mana; Detect Evil
    // misses the animals that kill it on DL1. 0 = C.
    let detect_spell_ok = st.borg.tr(BI_CLEVEL) >= param(Param::light_detect_evil_spell_min_level);
    if do_evil && due(st, st.borg.time.detect_evil, 20) && if detect_spell_ok {
        borg_spell_fail(st, DETECT_EVIL, 40)
            || borg_spell_fail(st, DETECT_MONSTERS, 40)
            || borg_spell_fail(st, READ_MINDS, 40)
            || borg_spell_fail(st, SEEK_BATTLE, 40)
    } else {
        detect_on_dark_entry(st, (q_y, q_x))
    } {
        borg_note(st, "# Checking for monsters.");

        borg_react(st, b"SELF:evil", b"SELF:evil");

        st.borg.time.detect_evil = st.borg.time.now;

        return true;
    }

    /* Find traps and doors (and stairs) */
    if (do_trap || do_door)
        && (due(st, st.borg.time.detect_traps, 5) || due(st, st.borg.time.detect_doors, 5))
        && st.borg.tr(BI_CDEPTH) != 0
        && (borg_activate_item(st, st.act.detect_all)
            || borg_activate_item(st, st.act.mapping)
            || borg_spell_fail(st, DETECTION, 40)
            || borg_spell_fail(st, FIND_TRAPS_DOORS_STAIRS, 40)
            || borg_spell_fail(st, DETECT_STAIRS, 40))
    {
        borg_note(st, "# Checking for traps, doors & stairs.");

        borg_react(st, b"SELF:both", b"SELF:both");

        st.borg.time.detect_traps = st.borg.time.now;
        st.borg.time.detect_doors = st.borg.time.now;

        return true;
    }

    /* Find traps */
    if do_trap
        && due(st, st.borg.time.detect_traps, 7)
        && st.borg.tr(BI_CDEPTH) != 0
        && (borg_spell_fail(st, DETECTION, 40) || borg_spell_fail(st, FIND_TRAPS_DOORS_STAIRS, 40))
    {
        borg_note(st, "# Checking for traps.");

        borg_react(st, b"SELF:trap", b"SELF:trap");

        st.borg.time.detect_traps = st.borg.time.now;

        return true;
    }

    /* Find doors */
    if do_door
        && due(st, st.borg.time.detect_doors, 9)
        && st.borg.tr(BI_CDEPTH) != 0
        && (borg_activate_item(st, st.act.detect_all)
            || borg_activate_item(st, st.act.mapping)
            || borg_spell_fail(st, DETECTION, 40)
            || borg_spell_fail(st, FIND_TRAPS_DOORS_STAIRS, 40))
    {
        borg_note(st, "# Checking for doors.");

        borg_react(st, b"SELF:door", b"SELF:door");

        st.borg.time.detect_doors = st.borg.time.now;

        return true;
    }

    /* Find walls */
    if do_wall
        && due(st, st.borg.time.detect_walls, 15)
        && st.borg.tr(BI_CDEPTH) != 0
        && (borg_activate_item(st, st.act.mapping)
            || borg_read_scroll(st, st.item_val.sv_scroll_mapping)
            || borg_use_staff(st, st.item_val.sv_staff_mapping)
            || borg_zap_rod(st, st.item_val.sv_rod_mapping)
            || borg_spell(st, SENSE_SURROUNDINGS))
    {
        borg_note(st, "# Checking for walls.");

        borg_react(st, b"SELF:wall", b"SELF:wall");

        st.borg.time.detect_walls = st.borg.time.now;

        /* Clear the BORG_IGNORE_MAP flag:  immediately after detection
         * want to use the map's information rather than what the borg
         * remembers. */
        for y in 0..AUTO_MAX_Y {
            for g in st.cave.row_mut(y) {
                g.info &= !BORG_IGNORE_MAP;
            }
        }
        return true;
    }

    /* Find objects */
    if do_obj
        && due(st, st.borg.time.detect_obj, 20)
        && (borg_activate_item(st, st.act.detect_objects)
            || borg_spell_fail(st, OBJECT_DETECTION, 40))
    {
        borg_note(st, "# Checking for objects.");

        borg_react(st, b"SELF:obj", b"SELF:obj");

        st.borg.time.detect_obj = st.borg.time.now;

        return true;
    }

    /* Do the actual illumination bit */
    borg_check_light_only(st)
}

/// Refuel a lantern (`borg_refuel_lantern`).
fn borg_refuel_lantern(st: &mut BorgState) -> bool {
    let pack_size = i32::from(engine::z_info().pack_size);
    let lantern = st.item_val.sv_light_lantern;

    /* Look for a flask of oil */
    let mut i = borg_slot(st, TV_FLASK as i32, st.item_val.sv_flask_oil);

    /* None available check for lantern */
    if i < 0 {
        /* get first lantern */
        i = (0..pack_size)
            .find(|&k| {
                let item = &st.items.items[k as usize];
                item.iqty != 0 && u32::from(item.tval) == TV_LIGHT && i32::from(item.sval) == lantern
            })
            .unwrap_or(-1);

        if i >= 0 {
            /* loop through lanterns (they should be next to each other) */
            while i < pack_size {
                let item = &st.items.items[i as usize];
                if u32::from(item.tval) != TV_LIGHT || i32::from(item.sval) != lantern {
                    i = -1;
                    break;
                }

                /* if this lantern is "Everburning" skip it, and it better
                 * have some oil left */
                if !engine::flag_has(&item.flags, OF_NO_FUEL as i32) && item.timeout > 0 {
                    break;
                }
                i += 1;
            }

            if i >= pack_size {
                i = -1;
            }
        }
    }

    /* Still none */
    if i < 0 {
        return false;
    }

    /* Cant refuel a torch with oil */
    if i32::from(st.items.items[inven_light() as usize].sval) != lantern {
        return false;
    }

    refuel_with(st, i);
    true
}

fn refuel_with(st: &mut BorgState, i: i32) {
    /* Log the message */
    let item = &st.items.items[i as usize];
    let mut note = b"# Refueling with ".to_vec();
    note.extend(item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8));
    note.push(b'.');
    borg_note_bytes(st, &note);

    /* Perform the action */
    borg_keypress(st, key(b'F'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals */
    st.borg.goal.item = -1;
    st.borg.goal.ware = st.borg.goal.item;
    st.borg.goal.shop = st.borg.goal.ware;

}

#[derive(Debug, PartialEq, Eq)]
enum LightAction {
    Refuel(usize),
    Wear(usize),
}

fn has_flag(item: &BorgItem, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

fn light_action(equipped: &BorgItem, pack: &[BorgItem], oil: i32, radius: i32) -> Option<LightAction> {
    // cmd-obj.c: do_cmd_refill; obj-util.c: obj_can_refill.
    let refillable = |item: &BorgItem| {
        item.iqty != 0
            && u32::from(item.tval) == TV_LIGHT
            && has_flag(item, OF_TAKES_FUEL)
            && !has_flag(item, OF_NO_FUEL)
    };
    let fuel = |item: &BorgItem| {
        item.iqty != 0
            && !has_flag(item, OF_NO_FUEL)
            && ((u32::from(item.tval) == TV_FLASK && i32::from(item.sval) == oil)
                || (refillable(item) && item.timeout > 0))
    };
    let fuel_slot = pack.iter().position(fuel);
    if refillable(equipped) && equipped.timeout == 0 {
        if let Some(slot) = fuel_slot {
            return Some(LightAction::Refuel(slot));
        }
    }

    if has_flag(equipped, OF_STICKY) {
        return None;
    }

    // player-calcs.c: calc_light. Replace a dimmed light only if the new
    // contribution makes the total positive.
    let contribution = |item: &BorgItem| {
        let innate = if has_flag(item, OF_LIGHT_2) {
            2
        } else if has_flag(item, OF_LIGHT_3) {
            3
        } else {
            0
        };
        innate + i32::from(item.modifiers[OBJ_MOD_LIGHT as usize])
    };
    let powered = |item: &BorgItem| item.timeout > 0 || has_flag(item, OF_NO_FUEL);
    let current = if equipped.iqty != 0 && powered(equipped) {
        contribution(equipped)
    } else {
        0
    };

    pack.iter()
        .enumerate()
        .filter(|(_, item)| {
            item.iqty != 0
                && u32::from(item.tval) == TV_LIGHT
                && !item.cursed
                && (powered(item) || (refillable(item) && fuel_slot.is_some()))
                && radius - current + contribution(item) > 0
        })
        .min_by_key(|(_, item)| !powered(item))
        .map(|(slot, _)| LightAction::Wear(slot))
}

/// Restore carried light without relying on ordinary equipment power or wear cooldowns.
pub(crate) fn borg_restore_light(st: &mut BorgState) -> bool {
    let action = light_action(
        &st.items.items[inven_light() as usize],
        &st.items.items[..inven_wield() as usize],
        st.item_val.sv_flask_oil,
        st.borg.tr(BI_LIGHT),
    );
    match action {
        Some(LightAction::Refuel(slot)) => refuel_with(st, slot as i32),
        Some(LightAction::Wear(slot)) => {
            borg_note(st, "# Wearing a working light source.");
            borg_keypress(st, key(b'w'));
            borg_keypress(st, letter(slot as i32));
        }
        None => return false,
    }
    true
}

/// Refuel the light if it is needed (`borg_maintain_light`).
pub fn borg_maintain_light(st: &mut BorgState) -> BorgNeed {
    let light = &st.items.items[inven_light() as usize];

    if engine::flag_has(&light.flags, OF_NO_FUEL as i32) {
        return BorgNeed::NoNeed;
    }

    /* necromancers like the dark */
    if st.borg.tr(BI_CLASS) == CLASS_NECROMANCER {
        return BorgNeed::NoNeed;
    }

    /* current torch */
    if u32::from(light.tval) == TV_LIGHT {
        if i32::from(light.sval) == st.item_val.sv_light_torch {
            if light.timeout > 250 {
                return BorgNeed::NoNeed;
            }
            /* Look for another torch */
            if borg_slot(st, TV_LIGHT as i32, st.item_val.sv_light_torch) < 0 {
                return BorgNeed::UnmetNeed;
            }

            /* Torches automatically disappear when they get to 0 turns
             * so we don't need to actively swap them out */
            return BorgNeed::NoNeed;
        }

        /* Refuel current lantern if needed */
        if i32::from(light.sval) == st.item_val.sv_light_lantern && light.timeout < 1000 {
            if borg_refuel_lantern(st) {
                return BorgNeed::MetNeed;
            }

            return BorgNeed::UnmetNeed;
        }
        BorgNeed::NoNeed
    } else {
        let mut i = borg_slot(st, TV_LIGHT as i32, st.item_val.sv_light_lantern);
        if i < 0 {
            i = borg_slot(st, TV_LIGHT as i32, st.item_val.sv_light_torch);
        }

        if i < 0 {
            BorgNeed::UnmetNeed
        } else {
            borg_keypress(st, key(b'w'));
            borg_keypress(st, letter(i));
            BorgNeed::MetNeed
        }
    }
}

/// One direction test of `borg_light_beam`: are the grids `0..=light`
/// steps (`first..=light`) from the Borg along `(dx, dy)` all floor or
/// unknown, with the grid `light + 1` steps away (`beyond`) unknown?
/// Returns `None` if the test does not apply (off the map or the panel),
/// else whether it is blocked. C bug 22: the west test starts at step 1.
fn beam_blocked(st: &BorgState, dx: i32, dy: i32, first: i32, light: i32) -> Option<bool> {
    let (cx, cy) = (st.borg.c.x, st.borg.c.y);
    let (bx, by) = (cx + dx * (light + 1), cy + dy * (light + 1));

    /* Look just beyond my light */
    let beyond_unknown = u32::from(st.cave.grid(bx, by).feat) == FEAT_NONE;

    /* Must be on the panel */
    if !engine::panel_contains(by, bx) {
        return None;
    }

    /* Check each grid in our light radius along the course */
    for i in first..=light {
        let (x, y) = (cx + dx * i, cy + dy * i);
        if !((borg_cave_floor_bold(st, y, x) || u32::from(st.cave.grid(x, y).feat) == FEAT_NONE)
            && beyond_unknown)
        {
            return Some(true);
        }
    }
    Some(false)
}

/// Look down a hallway and possibly light it up using the Light Beam
/// spell, a rod or a wand (`borg_light_beam`). With `simulation`, only
/// tell whether it would.
///
/// This spell is mostly used when the borg is moving through the dungeon
/// under boosted bravery. This will allow him to "see" if anyone is there.
///
/// Borg has a problem when not on map centering mode and casting the beam
/// repeatedly, down or up when at the edge of a panel.
pub fn borg_light_beam(st: &mut BorgState, simulation: bool) -> bool {
    let mut dir = 5;
    let mut spell_ok = false;
    let mut blocked = false;

    /* Weak/dark is very unhappy */
    if st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Require the ability */
    if borg_spell_okay_fail(st, SPEAR_OF_LIGHT, 20)
        || (-1 != borg_slot(st, TV_WAND as i32, st.item_val.sv_wand_light) && {
            /* C calls borg_slot() again for the index */
            let w = borg_slot(st, TV_WAND as i32, st.item_val.sv_wand_light);
            st.items.items[w as usize].pval != 0
        })
        || borg_equips_rod(st, st.item_val.sv_rod_light)
    {
        spell_ok = true;
    }

    let light = st.borg.tr(BI_LIGHT);
    let (cx, cy) = (st.borg.c.x, st.borg.c.y);

    /* The four directions in C's order: north, south, east, west. Each
     * later test runs only while no direction was found (`dir == 5`) and
     * resets `blocked`; a test that applies sets `dir` (or 5 and
     * `blocked` when the way is blocked). */
    let tests = [
        (cy - light - 1 > 0, 0, -1, 0, 8),
        (cy + light + 1 < AUTO_MAX_Y, 0, 1, 0, 2),
        (cx + light + 1 < AUTO_MAX_X, 1, 0, 0, 6),
        (cx - light - 1 > 0, -1, 0, 1, 4),
    ];
    for (k, &(in_map, dx, dy, first, d)) in tests.iter().enumerate() {
        /* Quick Boundary check */
        if !in_map || (k > 0 && dir != 5) {
            continue;
        }
        if k > 0 {
            blocked = false;
        }
        match beam_blocked(st, dx, dy, first, light) {
            None => {}
            Some(true) => {
                dir = 5;
                blocked = true;
            }
            /* a loop that ran at least once notes the direction */
            Some(false) if light >= first => dir = d,
            Some(false) => {}
        }
    }

    /* Don't do it if on the edge of shifting the panel. */
    if dir == 5
        || !spell_ok
        || blocked
        || (dir == 2 && cy - engine::term_geometry().offset_y > borg_panel_hgt() - 2)
        || (dir == 8 && cy - engine::term_geometry().offset_y > borg_panel_hgt() - 2)
    {
        return false;
    }

    /* simulation */
    if simulation {
        return true;
    }

    /* cast the light beam */
    if borg_spell_fail(st, SPEAR_OF_LIGHT, 20)
        || borg_zap_rod(st, st.item_val.sv_rod_light)
        || borg_aim_wand(st, st.item_val.sv_wand_light)
    {
        /* apply the direction */
        borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
        borg_note(st, "# Illuminating this hallway");
        return true;
    }

    /* cant do it */
    false
}


#[cfg(test)]
mod dark_entry_tests {
    use super::*;

    #[test]
    fn dark_entry_requires_a_new_level_or_a_single_step() {
        assert_eq!(entry_direction(None, (12, 8)), Some((0, 0)));
        assert_eq!(entry_direction(Some((12, 7)), (12, 8)), Some((0, 1)));
        assert_eq!(entry_direction(Some((11, 9)), (12, 8)), Some((1, -1)));
        assert_eq!(entry_direction(Some((12, 8)), (12, 8)), None);
        assert_eq!(entry_direction(Some((16, 1)), (12, 8)), None);
    }

    #[test]
    fn the_dark_frontier_is_unknown_forward_of_known_floor() {
        use FrontierGrid::{Blocked, Floor, Unknown};

        let grid = |y, x| match (y, x) {
            (10, 11) => Floor,
            (10, 12) => Unknown,
            _ => Blocked,
        };
        assert!(dark_frontier((10, 10), (0, 1), 1, grid));
        assert!(dark_frontier((10, 10), (0, 0), 1, grid));
        assert!(!dark_frontier((10, 10), (0, -1), 1, grid));
        assert!(!dark_frontier((10, 10), (0, 1), 2, grid));
        assert!(!dark_frontier((10, 10), (0, 1), 1, |_, _| Unknown));
        assert!(!dark_frontier((10, 10), (0, 1), 1, |_, _| Floor));
    }

    #[test]
    fn dark_entry_does_not_spend_its_response_reserve() {
        assert!(!fits_detection_reserve(3, 2, 2));
        assert!(fits_detection_reserve(4, 2, 2));
        assert!(!fits_detection_reserve(5, 2, 4));
        assert!(!fits_detection_reserve(10, -1, 2));
    }
}

#[cfg(test)]
mod lightless_tests {
    use super::*;
    use crate::borg_item::blank_item;

    fn light(flags: &[u32], timeout: i16) -> BorgItem {
        let mut item = blank_item();
        item.iqty = 1;
        item.tval = TV_LIGHT as u8;
        item.timeout = timeout;
        for flag in flags {
            let offset = (*flag as i32 - engine::FLAG_START) as usize;
            item.flags[offset / 8] |= 1 << (offset % 8);
        }
        item
    }

    #[test]
    fn refuels_an_empty_lantern_before_replacing_it() {
        let equipped = light(&[OF_TAKES_FUEL, OF_LIGHT_2], 0);
        let spare_torch = light(&[OF_LIGHT_2], 30);
        let mut flask = blank_item();
        flask.iqty = 1;
        flask.tval = TV_FLASK as u8;
        flask.sval = 1;
        assert_eq!(light_action(&equipped, &[spare_torch, flask], 1, 0), Some(LightAction::Refuel(1)));
        assert_eq!(light_action(&equipped, &[spare_torch], 1, 0), Some(LightAction::Wear(0)));
        assert_eq!(light_action(&equipped, &[equipped], 1, 0), None);
    }
}

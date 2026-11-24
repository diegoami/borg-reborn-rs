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
use crate::borg_cave_light::{engine_distance, engine_light_shape, own_light_radius, BORG_LIGHT};
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
use crate::borg_projection::known_projectable;
use crate::borg_update::Detect;
use crate::engine;
use crate::params::{value as param, Param};
use crate::borg_headers::borg_magic_h::{
    borg_spells, CALL_LIGHT, CLAIRVOYANCE, CREATE_DARKNESS, DETECTION, DETECT_EVIL, DETECT_MONSTERS,
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
use std::collections::{BTreeMap, BTreeSet};

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
pub(crate) fn inven_light() -> i32 { light_slot(inven_wield() as usize) as i32 }

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
    // borg-reborn-rs (#884): the radius branches read the Borg's own light
    // radius (BI_LIGHT at cave_light_engine_radius 0, C's test; the
    // engine's BI_LIGHT - 1 at 1, so a torch takes the radius-1 test).
    // BI_LIGHT 1 (a Ring of Light without a light source) is radius 0 at
    // switch 1 and keeps C's corner test; only no light at all (BI_LIGHT 0)
    // takes neither test and calls light, as in C (review of #894).
    let radius = own_light_radius(st);
    let uses_corner_test = radius == 1 || (radius == 0 && st.borg.tr(BI_LIGHT) >= 1);
    if uses_corner_test {
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
    } else if radius > 1 {
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
        || cast_room_lighting(
            st, uses_corner_test, param(Param::light_first_cast_require_dark) != 0,
            borg_spell_fail, unglowing_on_panel,
        )
    {
        borg_note(st, "# Illuminating the dungeon");
        borg_react(st, b"SELF:lite", b"SELF:lite");
        st.borg.time.call_light = st.borg.time.now;
        return true;
    }

    false
}

type GridPosition = (i32, i32);

#[derive(Default)]
pub(crate) struct RoomLighting {
    pending: Option<(GridPosition, bool)>,
    dark_after_cast: BTreeMap<GridPosition, BTreeSet<GridPosition>>,
}

impl RoomLighting {
    fn minimum_radius<I: Iterator<Item = GridPosition>>(
        &self,
        from: GridPosition,
        require_nearby_dark_floor: impl FnOnce() -> bool,
        dark_floors: impl FnOnce() -> I,
    ) -> Option<i32> {
        // A confirmed cast leaves a snapshot of the dark floor it did not
        // reach. Newly observed floor can justify trying the room flood again.
        if let Some(previous) = self.dark_after_cast.get(&from) {
            return dark_floors().any(|grid| !previous.contains(&grid)).then_some(0);
        }
        // An unglowing casting square can still be in an unlit room.
        if !require_nearby_dark_floor() {
            return Some(0);
        }
        // #1119: From a glowing square, look for evidence that the spell's
        // LIGHT_WEAK sphere can light more ground, including outside rooms.
        dark_floors().map(|(y, x)| engine_distance(y - from.0, x - from.1)).min()
    }

    pub(crate) fn confirm_message(&mut self, msg: &[u8]) {
        // effect-handler-general.c, effect_handler_LIGHT_AREA.
        if msg == b"You are surrounded by a white light." {
            if let Some((_, confirmed)) = &mut self.pending {
                *confirmed = true;
            }
        }
    }

    fn finish(&mut self, here: GridPosition, dark_floors: impl FnOnce() -> BTreeSet<GridPosition>) {
        if self.pending.take() == Some((here, true)) {
            self.dark_after_cast.insert(here, dark_floors());
        }
    }
}

fn unglowing_on_panel(y: i32, x: i32) -> bool {
    engine::panel_contains(y, x) && !engine::square_isglow(x, y)
}

fn observed_dark_floors<'a>(
    st: &'a BorgState,
    is_unglowing_on_panel: impl Fn(i32, i32) -> bool + 'a,
) -> impl Iterator<Item = GridPosition> + 'a {
    let count = st.view.view_n as usize;
    st.view.view_y[..count].iter().zip(&st.view.view_x[..count]).filter_map(move |(&y, &x)| {
        let (y, x) = (i32::from(y), i32::from(x));
        let grid = st.cave.grid(x, y);
        (u32::from(grid.feat) != FEAT_NONE
            && borg_cave_floor_grid(grid)
            && is_unglowing_on_panel(y, x)).then_some((y, x))
    })
}

fn cast_room_lighting(
    st: &mut BorgState,
    uses_corner_test: bool,
    require_dark_for_first_cast: bool,
    mut cast: impl FnMut(&mut BorgState, borg_spells, i32) -> bool,
    is_unglowing_on_panel: impl Fn(i32, i32) -> bool,
) -> bool {
    let here = (st.borg.c.y, st.borg.c.x);
    let minimum_radius = if uses_corner_test {
        let Some(radius) = st.light.room_lighting.minimum_radius(
            here,
            || require_dark_for_first_cast && !is_unglowing_on_panel(here.0, here.1),
            || observed_dark_floors(st, &is_unglowing_on_panel),
        ) else {
            return false;
        };
        radius
    } else {
        0
    };
    if !room_lighting_spells(minimum_radius).any(|spell| cast(st, spell, 40)) {
        return false;
    }
    if uses_corner_test {
        st.light.room_lighting.pending = Some((here, false));
    }
    true
}

fn room_lighting_spells(minimum_radius: i32) -> impl Iterator<Item = borg_spells> {
    // class.txt, Light Room and Call Light: LIGHT_AREA floods the room
    // (cave-map.c:light_room), and SPHERE:LIGHT_WEAK lights grids up to 2
    // and 1 away respectively (project-feat.c:project_feature_handler_LIGHT_WEAK).
    [(LIGHT_ROOM, 2), (CALL_LIGHT, 1)].into_iter()
        .filter_map(move |(spell, radius)| (radius >= minimum_radius).then_some(spell))
}

/// Resolve the attempt after the map and view have observed the spell's effect.
pub(crate) fn finish_room_lighting(st: &mut BorgState) {
    observe_room_lighting(st, unglowing_on_panel);
}

fn observe_room_lighting(st: &mut BorgState, is_unglowing_on_panel: impl Fn(i32, i32) -> bool) {
    let here = (st.borg.c.y, st.borg.c.x);
    if st.light.room_lighting.pending.is_some() {
        let mut lighting = std::mem::take(&mut st.light.room_lighting);
        lighting.finish(here, || observed_dark_floors(st, is_unglowing_on_panel).collect());
        st.light.room_lighting = lighting;
    }
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
    (y, x): (i32, i32), direction: (i32, i32), radius: i32, engine_shape: bool,
    grid: impl Fn(i32, i32) -> FrontierGrid,
) -> bool {
    let (dy, dx) = direction;
    (-1..=1).any(|sy| (-1..=1).any(|sx| {
        if (sy == 0 && sx == 0) || ((dy != 0 || dx != 0) && sy * dy + sx * dx <= 0) {
            return false;
        }
        let lit = lit_steps((sy, sx), radius, engine_shape);
        grid(y + sy * (lit + 1), x + sx * (lit + 1)) == FrontierGrid::Unknown
            && (1..=lit).all(|step| grid(y + sy * step, x + sx * step) == FrontierGrid::Floor)
    }))
}

/// borg-reborn-rs (#884, review of #894): how many steps along the ray
/// `(sy, sx)` are within the light. With the engine's shape, the steps at
/// engine `distance()` <= radius (a diagonal step at radius 2: only the
/// first, since (2,2) is at distance 3); otherwise the radius itself, the
/// shape of C's light (which lights (2,2)).
fn lit_steps((sy, sx): (i32, i32), radius: i32, engine_shape: bool) -> i32 {
    if !engine_shape {
        return radius;
    }
    (1..=radius).take_while(|&k| engine_distance(sy * k, sx * k) <= radius).count() as i32
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
    // borg-reborn-rs (#884): the Borg's own light radius (BI_LIGHT - 1
    // under cave_light_engine_radius 1, so a torch looks at distance 2).
    dark_frontier((y, x), direction, own_light_radius(st).max(1), engine_light_shape(), |y, x| {
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

/// Index of the light slot given the first equipment slot.
pub(crate) fn light_slot(first: usize) -> usize {
    first + 5
}

#[derive(Debug, PartialEq, Eq)]
enum LightAction {
    Refuel(usize),
    Wear(usize),
}

fn has_flag(item: &BorgItem, flag: u32) -> bool {
    engine::flag_has(&item.flags, flag as i32)
}

fn light_stack_turns(item: &BorgItem) -> i32 {
    if item.cursed || u32::from(item.tval) != TV_LIGHT { return 0; }
    // Angband's object_split copies the light's full timeout to the wielded
    // copy and leaves the other copies at the same timeout (obj-pile.c).
    i32::from(item.timeout.max(0)) * i32::from(item.iqty)
}

/// Fuel available to a character who is currently relying on a lit torch.
pub(crate) fn torch_fuel_turns(st: &BorgState) -> Option<i32> {
    let equipped = &st.items.items[inven_light() as usize];
    if equipped.iqty == 0 || u32::from(equipped.tval) != TV_LIGHT
        || i32::from(equipped.sval) != st.item_val.sv_light_torch || equipped.timeout <= 0
        || has_flag(equipped, OF_NO_FUEL) || engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT)
    {
        return None;
    }
    Some(light_stack_turns(equipped) + st.items.items[..inven_wield() as usize]
        .iter().map(light_stack_turns).sum::<i32>())
}

/// borg-reborn-rs (#1110): the light goal of a trip: the trip's turns or
/// `town_light_reserve_turns`, whichever is larger, scaled while
/// breadth-first exploration applies (`explore_bfs_light_percent`). The
/// town planner, the torch replenishment, the oil-throw guard and the
/// fuel-junking guard all read this one goal (review of #1113).
pub(crate) fn fuel_goal(st: &BorgState, journey: crate::borg_town_utility::Journey) -> i64 {
    scaled_goal(journey.fuel_goal(i64::from(param(Param::town_light_reserve_turns))),
        crate::borg_flow_dark::bfs_supply_percent(st, true))
}

/// `goal` x `percent` / 100 (100 = the goal itself).
pub(crate) fn scaled_goal(goal: i64, percent: i32) -> i64 {
    goal.saturating_mul(i64::from(percent.max(0))) / 100
}

#[cfg(test)]
mod bfs_fuel_goal_tests {
    use super::scaled_goal;

    #[test]
    fn twenty_thousand_turns_on_hand_fall_short_while_breadth_first_applies() {
        // Review of #1113: a short trip (3,000 turns) under the 20,000-turn
        // reserve. Off, 20,000 turns on hand meet the goal; at the shipped
        // 150 the goal is 30,000, so the replenishment buys torches and the
        // throw and junking guards keep the fuel (all read `fuel_goal`).
        let goal = 3_000_i64.max(20_000);
        let on_hand = 20_000;
        assert!(on_hand >= scaled_goal(goal, 100), "off: no replenishment");
        assert_eq!(scaled_goal(goal, 150), 30_000);
        assert!(on_hand < scaled_goal(goal, 150), "on: two more torches wanted");
        // A long trip past the reserve scales the same way.
        assert_eq!(scaled_goal(40_000_i64.max(20_000), 150), 60_000);
    }
}

/// Keep a usable fuel stack when discarding it would break the configured
/// reserve. Torch-only characters cannot count oil as light.
pub(crate) fn keep_fuel_stack(st: &BorgState, slot: usize) -> bool {
    let reserve = param(Param::town_light_reserve_turns);
    if reserve == 0 || !crate::borg_town_live::active(st) || slot >= inven_wield() as usize {
        return false;
    }
    let equipped = &st.items.items[inven_light() as usize];
    if has_flag(equipped, OF_NO_FUEL) || engine::player_has(crate::headers::list_player_flags_h::PF_UNLIGHT) {
        return false;
    }
    let lantern = has_flag(equipped, OF_TAKES_FUEL);
    let fuel = |item: &BorgItem| {
        if lantern && !item.cursed && u32::from(item.tval) == TV_FLASK
            && i32::from(item.sval) == st.item_val.sv_flask_oil
        {
            i32::from(item.pval.max(0)) * i32::from(item.iqty)
        } else {
            light_stack_turns(item)
        }
    };
    let pack = &st.items.items[..inven_wield() as usize];
    if fuel(&pack[slot]) == 0 { return false; }
    let remaining = fuel(equipped) + pack.iter().enumerate()
        .filter(|(index, _)| *index != slot).map(|(_, item)| fuel(item)).sum::<i32>();
    let goal = fuel_goal(st, crate::borg_town_utility::Journey::of(st));
    i64::from(remaining) < goal
}

fn light_action(equipped: &BorgItem, pack: &[BorgItem], oil: i32, radius: i32) -> Option<LightAction> {
    // cmd-obj.c:1071 (do_cmd_refill); obj-util.c:743 (obj_can_refill).
    let refillable = |item: &BorgItem| {
        item.iqty != 0
            && u32::from(item.tval) == TV_LIGHT
            && has_flag(item, OF_TAKES_FUEL)
            && !has_flag(item, OF_NO_FUEL)
    };
    let fuel_slot = pack.iter().position(|item| {
        item.iqty != 0
            && u32::from(item.tval) == TV_FLASK
            && i32::from(item.sval) == oil
            && !has_flag(item, OF_NO_FUEL)
    }).or_else(|| pack.iter().position(|item| refillable(item) && item.timeout > 0));
    if refillable(equipped) && equipped.timeout == 0 {
        if let Some(slot) = fuel_slot {
            return Some(LightAction::Refuel(slot));
        }
    }

    if has_flag(equipped, OF_STICKY) {
        return None;
    }

    let current = if equipped.iqty != 0 && light_powered(equipped) {
        light_contribution(equipped)
    } else {
        0
    };
    best_light(pack, radius - current, |item| refillable(item) && fuel_slot.is_some())
        .map(LightAction::Wear)
}

fn light_powered(item: &BorgItem) -> bool {
    item.timeout > 0 || has_flag(item, OF_NO_FUEL)
}

// Innate radius plus the light modifier (player-calcs.c:calc_light).
fn light_contribution(item: &BorgItem) -> i32 {
    let innate = if has_flag(item, OF_LIGHT_2) {
        2
    } else if has_flag(item, OF_LIGHT_3) {
        3
    } else {
        0
    };
    innate + i32::from(item.modifiers[OBJ_MOD_LIGHT as usize])
}

fn best_light(
    pack: &[BorgItem],
    other_radius: i32,
    can_refuel: impl Fn(&BorgItem) -> bool,
) -> Option<usize> {
    pack.iter()
        .enumerate()
        .filter(|(_, item)| {
            item.iqty != 0
                && u32::from(item.tval) == TV_LIGHT
                && !item.cursed
                && (light_powered(item) || can_refuel(item))
                && other_radius + light_contribution(item) > 0
        })
        .max_by_key(|(slot, item)| (light_powered(item), light_contribution(item), std::cmp::Reverse(*slot)))
        .map(|(slot, _)| slot)
}

/// Replace a harmful light in one wield command. The spare must already
/// have power, and its contribution plus the rest of the worn gear must
/// leave a positive radius (as in `light_action`).
pub(crate) fn uncursed_light_replacement(equipped: &BorgItem, pack: &[BorgItem], radius: i32) -> Option<usize> {
    let current = if light_powered(equipped) { light_contribution(equipped) } else { 0 };
    best_light(pack, radius - current, |_| false)
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

/// borg-reborn-rs (#995): remember the observed terrain for each aimed
/// direction at the last beam origin. Spell, rod and wand share the history.
#[derive(Clone, Default)]
pub struct HallwayBeams {
    origin: Option<(i32, i32)>,
    terrain: [Option<Vec<u8>>; 4],
}

impl HallwayBeams {
    fn tried(&self, origin: (i32, i32), direction: usize, terrain: &[u8]) -> bool {
        self.origin == Some(origin) && self.terrain[direction].as_deref() == Some(terrain)
    }

    fn record(&mut self, origin: (i32, i32), direction: usize, terrain: Vec<u8>) {
        if self.origin != Some(origin) {
            self.terrain = Default::default();
            self.origin = Some(origin);
        }
        self.terrain[direction] = Some(terrain);
    }
}

/// LINE uses the projection range and stops at a blocking grid
/// (src/effect-handler-attack.c:effect_handler_LINE:173-180,
/// src/project.c:project_path:293-335). Unknown terrain may be open.
fn hallway_beam_terrain(st: &BorgState, dx: i32, dy: i32) -> Vec<u8> {
    let (cx, cy) = (st.borg.c.x, st.borg.c.y);
    let mut terrain = Vec::new();
    for step in 1..=i32::from(engine::z_info().max_range) {
        let (x, y) = (cx + dx * step, cy + dy * step);
        if !(0..AUTO_MAX_X).contains(&x) || !(0..AUTO_MAX_Y).contains(&y) {
            break;
        }
        let grid = st.cave.grid(x, y);
        terrain.push(grid.feat);
        let feat = u32::from(grid.feat);
        if feat != FEAT_NONE && !known_projectable(feat, grid.trap, engine::feat_is_shop(i32::from(grid.feat))) {
            break;
        }
    }
    terrain
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

/// The shipped hallway fix is disabled only for legacy profiles and replays.
pub(crate) fn hallway_beam_guard_enabled() -> bool {
    param(Param::light_hallway_repeat_guard) != 0
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
    let mut spell_ok = false;

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

    if !spell_ok {
        return false;
    }

    let near_panel_edge = st.borg.c.y - engine::term_geometry().offset_y > borg_panel_hgt() - 2;
    let light = own_light_radius(st);
    let guard = hallway_beam_guard_enabled();
    try_hallway_beam(
        st, simulation, light, near_panel_edge, guard,
        |st, dx, dy, first| {
            if beam_blocked(st, dx, dy, first, light) != Some(false) {
                return None;
            }
            Some(if guard { hallway_beam_terrain(st, dx, dy) } else { Vec::new() })
        },
        |st, dir| {
            if borg_spell_fail(st, SPEAR_OF_LIGHT, 20)
                || borg_zap_rod(st, st.item_val.sv_rod_light)
                || borg_aim_wand(st, st.item_val.sv_wand_light)
            {
                borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
                borg_note(st, "# Illuminating this hallway");
                return true;
            }
            false
        },
    )
}

fn try_hallway_beam(
    st: &mut BorgState, simulation: bool, light: i32, near_panel_edge: bool, guard: bool,
    open_ray: impl Fn(&BorgState, i32, i32, i32) -> Option<Vec<u8>>,
    queue_beam: impl FnOnce(&mut BorgState, i32) -> bool,
) -> bool {
    let (cx, cy) = (st.borg.c.x, st.borg.c.y);
    /* Preserve C's first viable direction when deciding whether to yield. */
    let tests = [
        (cy - light - 1 > 0, 0, -1, 0, 8),
        (cy + light + 1 < AUTO_MAX_Y, 0, 1, 0, 2),
        (cx + light + 1 < AUTO_MAX_X, 1, 0, 0, 6),
        (cx - light - 1 > 0, -1, 0, 1, 4),
    ];
    let Some((direction, dir, terrain)) = tests.into_iter().enumerate().find_map(
        |(direction, (in_map, dx, dy, first, dir))| {
            if !in_map || light < first {
                return None;
            }
            open_ray(st, dx, dy, first).map(|terrain| (direction, dir, terrain))
        },
    ) else {
        return false;
    };
    if (dir == 2 || dir == 8) && near_panel_edge {
        return false;
    }

    if guard && st.borg.hallway_beams.tried((cx, cy), direction, &terrain) {
        return false;
    }
    if simulation {
        return true;
    }
    if !queue_beam(st, dir) {
        return false;
    }

    // Mana, rod recharge and rests leave a queued direction tried.
    if guard {
        st.borg.hallway_beams.record((cx, cy), direction, terrain);
    }
    true
}



#[cfg(test)]
mod room_lighting_tests {
    use super::{cast_room_lighting, observe_room_lighting, room_lighting_spells, GridPosition, RoomLighting};
    use crate::borg_headers::borg_magic_h::{CALL_LIGHT, LIGHT_ROOM};
    use crate::state::BorgState;
    use std::collections::BTreeSet;

    const CAST_FROM: GridPosition = (44, 46);

    #[test]
    fn both_room_lighting_spells_remember_a_confirmed_cast() {
        for learned_spell in [LIGHT_ROOM, CALL_LIGHT] {
            let mut st = BorgState::default();
            (st.borg.c.y, st.borg.c.x) = CAST_FROM;
            let mut cast = |_: &mut BorgState, spell, max_failure| {
                assert_eq!(max_failure, 40);
                spell == learned_spell
            };
            let no_observed_dark_floor = |_, _| false;
            let unglowing_casting_square = |y, x| (y, x) == CAST_FROM;

            assert!(cast_room_lighting(&mut st, true, true, &mut cast, unglowing_casting_square));
            st.light.room_lighting.confirm_message(b"You are surrounded by a white light.");
            observe_room_lighting(&mut st, no_observed_dark_floor);

            st.borg.c.x += 1;
            observe_room_lighting(&mut st, no_observed_dark_floor);
            st.borg.c.x -= 1;
            let unexpected_cast = |_: &mut BorgState, _, _| panic!("revisit tried a spell");
            assert!(!cast_room_lighting(&mut st, true, true, unexpected_cast, no_observed_dark_floor));
            st.borg.c.x += 1;
            assert!(!cast_room_lighting(&mut st, true, true, unexpected_cast, no_observed_dark_floor));
        }
    }

    #[test]
    fn first_cast_from_glow_only_tries_spells_that_can_reach_observed_dark_floor() {
        let (y, x) = CAST_FROM;
        let lighting = RoomLighting::default();

        for (dy, dx, expected_spells) in [
            (1, 1, &[LIGHT_ROOM, CALL_LIGHT][..]),
            (1, 2, &[LIGHT_ROOM][..]),
            (2, 2, &[][..]),
        ] {
            let radius = lighting.minimum_radius(CAST_FROM, || true, || [(y + dy, x + dx)].into_iter());
            let candidates: Vec<_> = radius.into_iter().flat_map(room_lighting_spells).collect();
            assert_eq!(candidates, expected_spells);
        }
    }

    #[test]
    fn cast_order_is_preserved_with_gate_off_or_larger_carried_light() {
        let mut st = BorgState::default();
        (st.borg.c.y, st.borg.c.x) = CAST_FROM;
        for (uses_corner_test, require_dark_for_first_cast) in [(true, false), (false, true)] {
            let mut attempted = Vec::new();
            let mut cast = |_: &mut BorgState, spell, _| {
                attempted.push(spell);
                false
            };
            assert!(!cast_room_lighting(&mut st, uses_corner_test, require_dark_for_first_cast, &mut cast, |_, _| {
                panic!("cast should not need darkness evidence");
            }));
            assert_eq!(attempted, [LIGHT_ROOM, CALL_LIGHT]);
        }
    }

    #[test]
    fn newly_observed_dark_floor_can_justify_a_recast() {
        let mut lighting = RoomLighting::default();
        let unchanged_dark_floor = (44, 50);
        lighting.pending = Some((CAST_FROM, false));
        lighting.confirm_message(b"You are surrounded by a white light.");
        lighting.finish(CAST_FROM, || BTreeSet::from([unchanged_dark_floor]));

        assert_eq!(lighting.minimum_radius(CAST_FROM, || true, || [unchanged_dark_floor].into_iter()), None);
        assert_eq!(lighting.minimum_radius(CAST_FROM, || true, || [unchanged_dark_floor, (44, 53)].into_iter()), Some(0));
    }

    #[test]
    fn failed_or_unobserved_casts_leave_the_square_eligible() {
        let mut lighting = RoomLighting::default();
        lighting.pending = Some((CAST_FROM, false));
        lighting.confirm_message(b"You failed to concentrate hard enough!");
        lighting.finish(CAST_FROM, BTreeSet::new);
        assert_eq!(lighting.minimum_radius(CAST_FROM, || false, std::iter::empty), Some(0));

        lighting.pending = Some((CAST_FROM, false));
        lighting.confirm_message(b"You are surrounded by a white light.");
        lighting.finish((44, 47), BTreeSet::new);
        assert_eq!(lighting.minimum_radius(CAST_FROM, || false, std::iter::empty), Some(0));
    }
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
        assert!(dark_frontier((10, 10), (0, 1), 1, false, grid));
        assert!(dark_frontier((10, 10), (0, 0), 1, false, grid));
        assert!(!dark_frontier((10, 10), (0, -1), 1, false, grid));
        assert!(!dark_frontier((10, 10), (0, 1), 2, false, grid));
        assert!(!dark_frontier((10, 10), (0, 1), 1, false, |_, _| Unknown));
        assert!(!dark_frontier((10, 10), (0, 1), 1, false, |_, _| Floor));
        // The engine's shape gives the same rays at radius 1.
        assert!(dark_frontier((10, 10), (0, 1), 1, true, grid));
        assert!(!dark_frontier((10, 10), (0, -1), 1, true, grid));
    }

    /// #884 (review of #894): with the engine's shape a lantern (radius 2)
    /// lights two orthogonal steps but one diagonal step, (2,2) being at
    /// distance 3; C's shape counts two steps on every ray.
    #[test]
    fn lit_steps_follow_the_engine_distance() {
        assert_eq!(lit_steps((0, 1), 2, true), 2);
        assert_eq!(lit_steps((1, 1), 2, true), 1);
        assert_eq!(lit_steps((1, 1), 1, true), 1);
        assert_eq!(lit_steps((1, 1), 3, true), 2);
        assert_eq!(lit_steps((1, 1), 2, false), 2);
        use FrontierGrid::{Blocked, Floor, Unknown};
        // Diagonal at radius 2: (1,1) floor and (2,2) unknown is the
        // frontier with the engine's shape, not with C's.
        let grid = |y, x| match (y, x) {
            (11, 11) => Floor,
            (12, 12) => Unknown,
            _ => Blocked,
        };
        assert!(dark_frontier((10, 10), (1, 1), 2, true, grid));
        assert!(!dark_frontier((10, 10), (1, 1), 2, false, grid));
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
    fn each_torch_in_a_stack_has_its_own_full_timeout() {
        let mut torch = light(&[OF_LIGHT_2], 5000);
        torch.iqty = 3;
        assert_eq!(light_stack_turns(&torch), 15000);
        torch.iqty -= 1;
        assert_eq!(light_stack_turns(&torch), 10000);
    }

    #[test]
    fn refuels_an_empty_lantern_before_replacing_it() {
        let equipped = light(&[OF_TAKES_FUEL, OF_LIGHT_2], 0);
        let spare_torch = light(&[OF_LIGHT_2], 30);
        let mut flask = blank_item();
        flask.iqty = 1;
        flask.tval = TV_FLASK as u8;
        flask.sval = 1;
        let spare_lantern = light(&[OF_TAKES_FUEL, OF_LIGHT_2], 30);
        assert_eq!(
            light_action(&equipped, &[spare_torch, spare_lantern, flask], 1, 0),
            Some(LightAction::Refuel(2)),
        );
        assert_eq!(
            light_action(&equipped, &[spare_torch], 1, 0),
            Some(LightAction::Wear(0)),
        );
        assert_eq!(light_action(&equipped, &[equipped], 1, 0), None);
    }
    #[test]
    fn cursed_light_swap_counts_light_from_other_gear() {
        let mut cursed_torch = light(&[OF_LIGHT_2], 100);
        cursed_torch.cursed = true;
        let dim_spare = light(&[], 30);

        // The spare cannot light the Borg on its own. Other gear can.
        assert_eq!(uncursed_light_replacement(&cursed_torch, &[dim_spare], 2), None);
        assert_eq!(uncursed_light_replacement(&cursed_torch, &[dim_spare], 3), Some(0));

        let empty_spare = light(&[OF_TAKES_FUEL, OF_LIGHT_2], 0);
        assert_eq!(uncursed_light_replacement(&cursed_torch, &[empty_spare], 3), None);
    }

    #[test]
    fn wears_the_brightest_powered_spare() {
        let empty = blank_item();
        let torch = light(&[OF_LIGHT_2], 30);
        let lantern = light(&[OF_LIGHT_3], 30);
        assert_eq!(
            light_action(&empty, &[torch, lantern], 1, 0),
            Some(LightAction::Wear(1)),
        );
    }

}

#[cfg(test)]
mod hallway_beam_tests {
    use super::*;
    use crate::headers::list_terrain_h::FEAT_FLOOR;

    #[test]
    fn unchanged_hallway_yields_and_a_changed_ray_can_be_lit() {
        let mut st = BorgState::default();
        (st.borg.c.x, st.borg.c.y) = (117, 22);
        let north = std::cell::Cell::new(FEAT_NONE as u8);
        let try_beam = |st: &mut BorgState, simulation, queue: fn(&mut BorgState, i32) -> bool| {
            try_hallway_beam(st, simulation, 1, false, true, |_, _, dy, _| {
                Some(vec![if dy == -1 { north.get() } else { FEAT_NONE as u8 }])
            }, queue)
        };
        let queue_north = |_: &mut BorgState, dir| {
            assert_eq!(dir, 8);
            true
        };
        let unexpected = |_: &mut BorgState, _| panic!("queued an unexpected beam");

        assert!(try_beam(&mut st, true, unexpected));
        assert!(!try_beam(&mut st, false, |_, _| false));
        assert!(try_beam(&mut st, false, queue_north));
        assert!(!try_beam(&mut st, true, unexpected));
        assert!(!try_beam(&mut st, false, unexpected));

        north.set(FEAT_FLOOR as u8);
        assert!(try_beam(&mut st, false, queue_north));
        assert!(!try_beam(&mut st, false, unexpected));
    }
}

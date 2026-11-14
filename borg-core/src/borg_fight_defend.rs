//! Port of `borg-fight-defend.c`: defensive moves during a fight.
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
//! There are several types of setup moves: temporary speed, Protection from
//! Evil, Bless/Prayer, Berserk/Heroism, temporary resists (all or some),
//! Shield, teleport away, Glyph of Warding, See Invisible, and many others.
//! `borg_defend` simulates each of them (`borg_simulate`) and then performs
//! the best one. The simulations pretend to be protected, look at the danger
//! again, and restore what they changed, in C's order.

use crate::borg::BorgTime;
use crate::borg_cave::BorgGrid;
use crate::borg_cave_util::{
    borg_cave_floor_bold, borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid,
};
use crate::borg_danger::{borg_danger, borg_danger_one_kill};
use crate::borg_escape::{borg_caution_teleport, borg_surrounded};
use crate::borg_fight_attack::borg_launch_bolt;
use crate::borg_flow_kill::{borg_delete_kill, borg_race_name};
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, ESCAPE};
use crate::borg_item_use::{
    borg_activate_item, borg_activate_ring, borg_aim_wand, borg_eat, borg_equips_item,
    borg_equips_ring, borg_equips_rod, borg_equips_staff_fail, borg_quaff_potion,
    borg_read_scroll, borg_use_staff, borg_zap_rod,
};
use crate::borg_light::borg_light_beam;
use crate::borg_magic::{borg_heroism_level, borg_spell, borg_spell_fail, borg_spell_okay_fail};
use crate::borg_projection::{
    borg_inc_motion, borg_los, borg_projectable, borg_projectable_dark, borg_projectable_pure,
    borg_target,
};
use crate::engine;
use crate::borg_headers::borg_fight_attack_h::{BORG_ATTACK_AWAY_ALL, BORG_ATTACK_AWAY_ALL_MORGOTH};
use crate::borg_headers::borg_h::BORG_MONEY_SCUM_AMOUNT;
use crate::borg_headers::borg_magic_h::{
    BANISHMENT, BANISH_EVIL, BERSERK_STRENGTH, BLESS, CLAIRVOYANCE, DETECTION, DOOR_CREATION,
    FUME_OF_MORDOR, GLYPH_OF_WARDING, GRIM_PURPOSE, GRONDS_BLOW, HASTE_SELF, HEROISM,
    MASS_BANISHMENT, PROTECTION_FROM_EVIL, QUAKE, RAPID_REGENERATION, RESISTANCE, RESIST_POISON,
    REVEAL_MONSTERS, SENSE_INVISIBLE, SMITE_EVIL, SPEAR_OF_LIGHT, TELEPORT_LEVEL, TELEPORT_OTHER,
    TREMOR, WORD_OF_DESTRUCTION,
};
use crate::borg_headers::borg_trait_h::{
    BI_AESCAPE, BI_AGLYPH, BI_AMASSBAN, BI_ATELEPORT, BI_ATELEPORTLVL, BI_CDEPTH, BI_CLEVEL,
    BI_CURHP, BI_CURSP, BI_FRACT, BI_ISBLIND, BI_ISCONFUSED, BI_ISFORGET, BI_ISIMAGE, BI_LIGHT,
    BI_MAXHP, BI_RCONF, BI_WS_EVIL,
};
use crate::headers::list_mon_race_flags_h::{RF_EVIL, RF_UNIQUE};
use crate::headers::list_player_timed_h::TMD_OPP_ACID;
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_CLOSED, FEAT_GRANITE, FEAT_LESS, FEAT_MORE, FEAT_OPEN, FEAT_PERM,
    FEAT_RUBBLE,
};
use crate::headers::list_tvals_h::{TV_MUSHROOM, TV_POTION, TV_SCROLL, TV_WAND};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// The module's globals (borg-fight-defend.c).
#[derive(Default)]
pub struct FightDefendState {
    /// `borg_attempting_refresh_resist`: for the Resistance spell
    pub attempting_refresh_resist: bool,
}

/// `BORG_GLOW` (borg-cave.h grid info bit).
const BORG_GLOW: u16 = 0x02;
/// `BORG_OKAY` (borg-cave.h grid info bit).
const BORG_OKAY: u16 = 0x08;
/// `BORG_VIEW` (borg-cave.h grid info bit).
const BORG_VIEW: u16 = 0x20;

/// `UCHAR_MAX`: the size of the per-character arrays of the genocide.
const UCHAR_MAX: usize = 255;

/* The defence types, in C's enum order (`borg_defend` tries them in this
 * order and keeps the first best). */
const BD_BLESS: i32 = 0;
const BD_SPEED: i32 = 1;
const BD_GRIM_PURPOSE: i32 = 2;
const BD_RESIST_FECAP: i32 = 3;
const BD_RESIST_F: i32 = 4;
const BD_RESIST_C: i32 = 5;
const BD_RESIST_A: i32 = 6;
const BD_RESIST_E: i32 = 7;
const BD_RESIST_P: i32 = 8;
const BD_PROT_FROM_EVIL: i32 = 9;
const BD_SHIELD: i32 = 10;
const BD_TELE_AWAY: i32 = 11;
const BD_HERO: i32 = 12;
const BD_BERSERK: i32 = 13;
const BD_SMITE_EVIL: i32 = 14;
const BD_REGEN: i32 = 15;
const BD_GLYPH: i32 = 16;
const BD_CREATE_DOOR: i32 = 17;
const BD_MASS_GENOCIDE: i32 = 18;
const BD_GENOCIDE: i32 = 19;
const BD_GENOCIDE_NASTIES: i32 = 20;
const BD_EARTHQUAKE: i32 = 21;
const BD_DESTRUCTION: i32 = 22;
const BD_TPORTLEVEL: i32 = 23;
const BD_BANISHMENT: i32 = 24;
const BD_DETECT_INVISO: i32 = 25;
const BD_LIGHT_BEAM: i32 = 26;
const BD_REST: i32 = 27;
const BD_TELE_AWAY_MORGOTH: i32 = 28;
const BD_BANISHMENT_MORGOTH: i32 = 29;
const BD_LIGHT_MORGOTH: i32 = 30;
const BD_MAX: i32 = 31;

/// A literal key (`borg_keypress('5')` etc.).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `I2D(dir)`: the key of a direction.
fn i2d(dir: i32) -> keycode_t {
    (dir + i32::from(b'0')) as keycode_t
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `borg.trait[i]`
fn tr(st: &BorgState, i: u32) -> i32 {
    st.borg.tr(i)
}

/// `borg.avoidance` (promoted to `int`).
fn avoid(st: &BorgState) -> i32 {
    i32::from(st.borg.avoidance)
}

/// `borg_simulate` (still a borg-trait.c global).
fn simulating(st: &mut BorgState) -> bool {
    st.traits.simulate
}

/// A copy of `borg_grids[y][x]`.
fn grid(st: &BorgState, x: i32, y: i32) -> BorgGrid {
    *st.cave.grid(x, y)
}

/// "Cant when screwed": blind, confused or amnesiac.
fn screwed(st: &BorgState) -> bool {
    tr(st, BI_ISBLIND) != 0 || tr(st, BI_ISCONFUSED) != 0 || tr(st, BI_ISFORGET) != 0
}

/// The usual fail-rate scaling: very scary (`p1 > avoidance`) allows 19
/// less, a little scary (`p1 > avoidance * 2 / 3`) `mid` less, not very
/// scary (`p1 < avoidance / 3`) `low` more.
fn scaled_fail(st: &BorgState, p1: i32, base: i32, mid: i32, low: i32) -> i32 {
    let a = avoid(st);
    if p1 > a {
        base - 19
    } else if p1 > (a * 2) / 3 {
        base - mid
    } else if p1 < a / 3 {
        base + low
    } else {
        base
    }
}

/// `(borg.near.unique ? ((borg.avoidance * 2) / 3) : (borg.avoidance / 2))`
fn limit(st: &BorgState) -> i32 {
    if st.borg.near.unique != 0 {
        (avoid(st) * 2) / 3
    } else {
        avoid(st) / 2
    }
}

/// The usual "is this an improvement" test: `p1 > p2`, `p2` within the
/// limit, and `p1 > avoidance / div`.
fn improves(st: &BorgState, p1: i32, p2: i32, div: i32) -> bool {
    p1 > p2 && p2 <= limit(st) && p1 > (avoid(st) / div)
}

/// The "some danger but not much" window of the low priority buffs:
/// `p1 > avoidance / 10` and `p1 < avoidance * (unique ? 7 : 5) / 10`.
fn some_danger(st: &BorgState, p1: i32) -> bool {
    let a = avoid(st);
    p1 > a / 10 && p1 < (a * if st.borg.near.unique != 0 { 7 } else { 5 }) / 10
}

/// `!(ag->info & BORG_GLOW) && borg.trait[BI_LIGHT] == 0` for the Borg's
/// grid.
fn dark_here(st: &BorgState) -> bool {
    let ag = grid(st, st.borg.c.x, st.borg.c.y);
    ag.info & BORG_GLOW == 0 && tr(st, BI_LIGHT) == 0
}

/// `kill->r_idx >= z_info->r_max - 1` ("player ghosts").
fn is_ghost(r_idx: u16) -> bool {
    i32::from(r_idx) >= i32::from(engine::z_info().r_max) - 1
}

/// `z_info->max_range`
fn max_range() -> i32 {
    i32::from(engine::z_info().max_range)
}

/// `borg_danger(borg.c.y, borg.c.x, 1, average, false)`
fn danger_here(st: &mut BorgState, average: bool) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger(st, y, x, 1, average, false)
}

/// `borg_danger_one_kill(borg.c.y, borg.c.x, 1, i, true, full_damage)`
fn danger_one_here(st: &mut BorgState, i: i32, full_damage: bool) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger_one_kill(st, y, x, 1, i, true, full_damage)
}

/// `streq(r_info[borg.mon.unique].name, name)`
fn unique_is(st: &BorgState, name: &[u8]) -> bool {
    engine::race_name(st.borg.mon.unique as usize) == name
}

/// `borg_temp_n = 0; borg_tp_other_n = 0;` ("Reset list").
fn reset_lists(st: &mut BorgState) {
    st.flow.temp_n = 0;
    st.fight_attack.tp_other_n = 0;
}

/// Log the pathway and feature of the spell pathway
/// (`borg_log_spellpath`). Useful for debugging beams and Tport Other spell.
fn borg_log_spellpath(st: &mut BorgState) {
    let y = st.projection.target_loc.y;
    let x = st.projection.target_loc.x;
    let mut n_x = st.borg.c.x;
    let mut n_y = st.borg.c.y;

    let mut dist = 0;

    loop {
        let ag = grid(st, n_x, n_y);
        let kill = st.kills.kills[usize::from(ag.kill)];

        /* Note the Pathway */
        if !borg_cave_floor_grid(&ag) {
            borg_note(st, &format!("# Logging Spell pathway ({},{}): Wall grid.", n_y, n_x));
            break;
        } else if ag.kill != 0 {
            let d = danger_one_here(st, i32::from(ag.kill), false);
            let mut m = format!("# Logging Spell pathway ({},{}): ", n_y, n_x).into_bytes();
            m.extend_from_slice(borg_race_name(i32::from(kill.r_idx)).to_bytes());
            m.extend_from_slice(format!(", danger {}", d).as_bytes());
            borg_note_bytes(st, &m);
        } else if n_y == st.borg.c.y && n_x == st.borg.c.x {
            borg_note(st, &format!("# Logging Spell pathway ({},{}): My grid.", n_y, n_x));
        } else {
            borg_note(st, &format!("# Logging Spell pathway ({},{}).", n_y, n_x));
        }

        /* Stop loop if we reach our target if using bolt */
        if n_x == x && n_y == y {
            break;
        }

        /* Safegaurd not to loop */
        dist += 1;
        if dist >= max_range() {
            break;
        }

        /* Calculate the new location */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        borg_inc_motion(&mut n_y, &mut n_x, cy, cx, y, x);
    }
}

/// Bless/Prayer to prepare for battle (`borg_defend_aux_bless`).
fn borg_defend_aux_bless(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 25;

    let mut borg_near_kill = false;

    /* already blessed */
    if st.borg.temp.bless {
        return 0;
    }

    /* Cant when Blind */
    if screwed(st) {
        return 0;
    }

    /* Dark */
    if dark_here(st) {
        return 0;
    }

    /* no spell */
    if !borg_spell_okay_fail(st, BLESS, fail_allowed)
        && !borg_equips_item(st, st.act.blessing, true)
        && !borg_equips_item(st, st.act.blessing2, true)
        && !borg_equips_item(st, st.act.blessing3, true)
        && -1 == borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_blessing)
        && -1 == borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_holy_chant)
        && -1 == borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_holy_prayer)
    {
        return 0;
    }

    /* Check if a monster is close to me . */
    for i in 1..i32::from(st.kills.nxt) {
        let kill = &st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 5 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, kill.pos) > 3 {
            continue;
        }

        /* kill near me */
        borg_near_kill = true;
    }

    /* if we are in some danger but not much, go for a quick bless */
    let a = avoid(st);
    if (p1 > a / 12 || tr(st, BI_CLEVEL) <= 15) && p1 > 0 && borg_near_kill && p1 < a / 2 {
        /* Simulation */
        /* bless is a low priority */
        if simulating(st) {
            return 1;
        }

        borg_note(st, "# Attempting to cast Bless");

        /* No resting to recoup mana */
        st.borg.no_rest_prep = 11000;

        /* do it! */
        if borg_spell(st, BLESS)
            || borg_activate_item(st, st.act.blessing)
            || borg_activate_item(st, st.act.blessing2)
            || borg_activate_item(st, st.act.blessing3)
            || borg_read_scroll(st, st.item_val.sv_scroll_blessing)
            || borg_read_scroll(st, st.item_val.sv_scroll_holy_chant)
            || borg_read_scroll(st, st.item_val.sv_scroll_holy_prayer)
        {
            return 1;
        }
    }

    0
}

/// Speed to prepare for battle (`borg_defend_aux_speed`).
fn borg_defend_aux_speed(st: &mut BorgState, p1: i32) -> i32 {
    let mut good_speed = false;
    let mut speed_spell = false;
    let mut speed_staff = false;
    let mut speed_rod = false;

    /* already fast */
    if st.borg.temp.fast {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    /* only cast defense spells if fail rate is not too high */
    if borg_spell_okay_fail(st, HASTE_SELF, fail_allowed) {
        speed_spell = true;
    }

    /* staff must have charges */
    if borg_equips_staff_fail(st, st.item_val.sv_staff_speed) {
        speed_staff = true;
    }

    /* rod can't be charging */
    if borg_equips_rod(st, st.item_val.sv_rod_speed) {
        speed_rod = true;
    }

    /* Need some form */
    if 0 > borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_speed)
        && !speed_staff
        && !speed_rod
        && !speed_spell
        && !borg_equips_item(st, st.act.haste, true)
        && !borg_equips_item(st, st.act.haste1, true)
        && !borg_equips_item(st, st.act.haste2, true)
    {
        return 0;
    }

    /* if we have an infinite/large suppy of speed we can */
    /* be generous with our use */
    if speed_rod
        || speed_spell
        || speed_staff
        || borg_equips_item(st, st.act.haste, true)
        || borg_equips_item(st, st.act.haste1, true)
        || borg_equips_item(st, st.act.haste2, true)
    {
        good_speed = true;
    }

    /* pretend we are protected and look again */
    st.borg.temp.fast = true;
    let mut p2 = danger_here(st, true);
    st.borg.temp.fast = false;

    /* if scaryguy around cast it. */
    if st.borg.mon.scary {
        /* Further reduce danger to encourage speed use against scary or
         * unique monsters. */
        p2 = p2 * 3 / 10;
    }

    /* if we are fighting a unique cast it. */
    if good_speed && st.borg.near.unique != 0 {
        p2 = p2 * 7 / 10;
    }
    /* if we are fighting a unique and a summoner cast it. */
    if st.borg.near.summoner && st.borg.near.unique != 0 {
        p2 = p2 * 7 / 10;
    }
    /* if the unique is Sauron cast it */
    if tr(st, BI_CDEPTH) == 99 && st.borg.near.unique >= 10 {
        p2 = p2 * 6 / 10;
    }

    /* if the unique is a rather nasty one. */
    if st.borg.near.unique != 0
        && (unique_is(st, b"Bullroarer the Hobbit")
            || unique_is(st, b"Mughash the Kobold Lord")
            || unique_is(st, b"Wormtongue, Agent of Saruman")
            || unique_is(st, b"Lagduf, the Snaga")
            || unique_is(st, b"Brodda, the Easterling")
            || unique_is(st, b"Orfax, Son of Boldor"))
    {
        p2 = p2 * 6 / 10;
    }

    /* if the unique is Morgoth cast it */
    if tr(st, BI_CDEPTH) == 100 && st.borg.near.unique >= 10 {
        p2 = p2 * 5 / 10;
    }

    /* Attempt to conserve Speed at end of game */
    if tr(st, BI_CDEPTH) >= 97 && st.borg.near.unique == 0 && !good_speed {
        p2 = 9999;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    let a = avoid(st);
    let u = st.borg.near.unique != 0;
    if ((p1 > p2) && p2 <= (if u { (a * 2) / 3 } else { a / 2 }) && (p1 > (a / 5)) && good_speed)
        || ((p1 > p2) && p2 <= (if u { (a * 2) / 3 } else { a / 3 }) && (p1 > (a / 7)))
    {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast Speed");

        /* No resting to recoup mana */
        st.borg.no_rest_prep = tr(st, BI_CLEVEL) * 1000;

        /* do it! */
        if borg_zap_rod(st, st.item_val.sv_rod_speed)
            || borg_activate_item(st, st.act.haste)
            || borg_activate_item(st, st.act.haste1)
            || borg_activate_item(st, st.act.haste2)
            || borg_use_staff(st, st.item_val.sv_staff_speed)
            || borg_quaff_potion(st, st.item_val.sv_potion_speed)
        {
            /* Value */
            return p1 - p2;
        }

        if borg_spell_fail(st, HASTE_SELF, fail_allowed) {
            return p1 - p2;
        }
    }
    /* default to can't do it. */
    0
}

/// Grim Purpose (`borg_defend_aux_grim_purpose`).
fn borg_defend_aux_grim_purpose(st: &mut BorgState, mut p1: i32) -> i32 {
    /* C keeps the two traits in `bool`s and writes them back as 0/1 */
    let save_conf = tr(st, BI_RCONF) != 0;
    let save_fa = tr(st, BI_FRACT) != 0;

    /* already protected */
    if save_conf && save_fa {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, GRIM_PURPOSE, fail_allowed) {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    /* pretend we are protected and look again */
    st.borg.trait_[BI_RCONF as usize] = 1;
    st.borg.trait_[BI_FRACT as usize] = 1;
    let p2 = danger_here(st, false);
    st.borg.trait_[BI_RCONF as usize] = i32::from(save_conf);
    st.borg.trait_[BI_FRACT as usize] = i32::from(save_fa);

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2 + 2;
        }

        borg_note(st, "# Attempting to cast Grim Purpose");

        /* do it! */
        if borg_spell(st, GRIM_PURPOSE) {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 13000;
        }

        /* Value */
        return p1 - p2 + 2;
    }

    /* default to can't do it. */
    0
}

/// All resists (`borg_defend_aux_resist_fecap`).
fn borg_defend_aux_resist_fecap(st: &mut BorgState, mut p1: i32) -> i32 {
    let t = &st.borg.temp;
    if t.res_fire && t.res_acid && t.res_pois && t.res_elec && t.res_cold {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    if !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    /* pretend we are protected and look again */
    let save_fire = st.borg.temp.res_fire;
    let save_elec = st.borg.temp.res_elec;
    let save_cold = st.borg.temp.res_cold;
    let save_acid = st.borg.temp.res_acid;
    let save_poison = st.borg.temp.res_pois;
    st.borg.temp.res_fire = true;
    st.borg.temp.res_elec = true;
    st.borg.temp.res_cold = true;
    st.borg.temp.res_acid = true;
    st.borg.temp.res_pois = true;
    let mut p2 = danger_here(st, false);
    st.borg.temp.res_fire = save_fire;
    st.borg.temp.res_elec = save_elec;
    st.borg.temp.res_cold = save_cold;
    st.borg.temp.res_acid = save_acid;
    st.borg.temp.res_pois = save_poison;

    /* If the borg is fighting a particular unique enhance the
     * benefit of the spell. */
    if st.borg.near.unique != 0 && unique_is(st, b"The Tarrasque") {
        p2 = p2 * 8 / 10;
    }

    /* If borg is high enough level, he does not need to worry
     * about mana consumption.  Cast the good spell. */
    if tr(st, BI_CLEVEL) >= 45 {
        p2 = p2 * 8 / 10;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2 + 2;
        }

        borg_note(st, "# Attempting to cast FECAP");

        /* do it! */
        if borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.rage_bless_resist)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2 + 2;
    }

    /* default to can't do it. */
    0
}

/// Fire (`borg_defend_aux_resist_f`).
fn borg_defend_aux_resist_f(st: &mut BorgState, mut p1: i32) -> i32 {
    let save_fire = st.borg.temp.res_fire;

    if st.borg.temp.res_fire {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, RESISTANCE, fail_allowed)
        && !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.resist_fire, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && !borg_equips_ring(st, st.item_val.sv_ring_flames)
        && !borg_equips_item(st, st.act.ring_flames, true)
        && -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_resist_heat)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    /* pretend we are protected and look again */
    st.borg.temp.res_fire = true;
    let mut p2 = danger_here(st, false);
    st.borg.temp.res_fire = save_fire;

    /* If the borg is fighting a particular unique enhance the
     * benefit of the spell. */
    if st.borg.near.unique != 0 && unique_is(st, b"The Tarrasque") {
        p2 = p2 * 8 / 10;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast RFire");
        /* do it! */
        if borg_activate_ring(st, st.item_val.sv_ring_flames)
            || borg_activate_item(st, st.act.ring_flames)
        {
            /* Ring also attacks so target self */
            borg_keypress(st, key(b'*'));
            borg_keypress(st, key(b'5'));
            return p1 - p2;
        }
        if borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.resist_fire)
            || borg_activate_item(st, st.act.rage_bless_resist)
            || borg_spell_fail(st, RESISTANCE, fail_allowed)
            || borg_quaff_potion(st, st.item_val.sv_potion_resist_heat)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2;
    }

    /* default to can't do it. */
    0
}

/// Cold (`borg_defend_aux_resist_c`).
fn borg_defend_aux_resist_c(st: &mut BorgState, mut p1: i32) -> i32 {
    if st.borg.temp.res_cold {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, RESISTANCE, fail_allowed)
        && !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && !borg_equips_item(st, st.act.resist_cold, true)
        && !borg_equips_ring(st, st.item_val.sv_ring_ice)
        && !borg_equips_item(st, st.act.ring_ice, true)
        && -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_resist_cold)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    let save_cold = st.borg.temp.res_cold;
    /* pretend we are protected and look again */
    st.borg.temp.res_cold = true;
    let mut p2 = danger_here(st, false);
    st.borg.temp.res_cold = save_cold;

    /* If the borg is fighting a particular unique enhance the
     * benefit of the spell. */
    if st.borg.near.unique != 0 && unique_is(st, b"The Tarrasque") {
        p2 = p2 * 8 / 10;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast RCold");

        /* do it! */
        if borg_activate_ring(st, st.item_val.sv_ring_ice) || borg_activate_item(st, st.act.ring_ice)
        {
            /* Ring also attacks so target self */
            borg_keypress(st, key(b'*'));
            borg_keypress(st, key(b'5'));
            return p1 - p2;
        }
        if borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.rage_bless_resist)
            || borg_activate_item(st, st.act.resist_cold)
            || borg_spell_fail(st, RESISTANCE, fail_allowed)
            || borg_quaff_potion(st, st.item_val.sv_potion_resist_cold)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2;
    }

    /* default to can't do it. */
    0
}

/// Acid (`borg_defend_aux_resist_a`).
fn borg_defend_aux_resist_a(st: &mut BorgState, mut p1: i32) -> i32 {
    if st.borg.temp.res_acid {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, RESISTANCE, fail_allowed)
        && !borg_equips_item(st, st.act.resist_acid, true)
        && !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && !borg_equips_ring(st, st.item_val.sv_ring_acid)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    let save_acid = st.borg.temp.res_acid;
    /* pretend we are protected and look again */
    st.borg.temp.res_acid = true;
    let p2 = danger_here(st, false);
    st.borg.temp.res_acid = save_acid;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast RAcid");

        /* do it! */
        if borg_spell(st, RESISTANCE) {
            return p1 - p2;
        }

        if borg_activate_ring(st, st.item_val.sv_ring_acid)
            || borg_activate_item(st, st.act.ring_acid)
        {
            /* Ring also attacks so target self */
            borg_keypress(st, key(b'*'));
            borg_keypress(st, key(b'5'));
            return p1 - p2;
        }

        if borg_activate_item(st, st.act.resist_acid)
            || borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.rage_bless_resist)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2;
    }
    /* default to can't do it. */
    0
}

/// Electricity (`borg_defend_aux_resist_e`).
fn borg_defend_aux_resist_e(st: &mut BorgState, mut p1: i32) -> i32 {
    if st.borg.temp.res_elec {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, RESISTANCE, fail_allowed)
        && !borg_equips_item(st, st.act.resist_elec, true)
        && !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && !borg_equips_ring(st, st.item_val.sv_ring_lightning)
        && !borg_equips_item(st, st.act.ring_lightning, true)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    let save_elec = st.borg.temp.res_elec;
    /* pretend we are protected and look again */
    st.borg.temp.res_elec = true;
    let p2 = danger_here(st, false);
    st.borg.temp.res_elec = save_elec;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        /* (C's note says RAcid here too) */
        borg_note(st, "# Attempting to cast RAcid");

        /* do it! */
        if borg_spell(st, RESISTANCE) {
            return p1 - p2;
        }

        if borg_activate_ring(st, st.item_val.sv_ring_lightning)
            || borg_activate_item(st, st.act.ring_lightning)
        {
            /* Ring also attacks so target self */
            borg_keypress(st, key(b'*'));
            borg_keypress(st, key(b'5'));
            return p1 - p2;
        }

        if borg_activate_item(st, st.act.resist_elec)
            || borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.rage_bless_resist)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2;
    }
    /* default to can't do it. */
    0
}

/// Poison (`borg_defend_aux_resist_p`).
fn borg_defend_aux_resist_p(st: &mut BorgState, mut p1: i32) -> i32 {
    if st.borg.temp.res_pois {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    if !borg_spell_okay_fail(st, RESIST_POISON, fail_allowed)
        && !borg_equips_item(st, st.act.resist_pois, true)
        && !borg_equips_item(st, st.act.resist_all, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_resist_pois)
    {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    let save_poison = st.borg.temp.res_pois;
    /* pretend we are protected and look again */
    st.borg.temp.res_pois = true;
    let p2 = danger_here(st, false);
    st.borg.temp.res_pois = save_poison;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast RPois");

        /* do it! */
        if borg_spell_fail(st, RESIST_POISON, fail_allowed)
            || borg_activate_item(st, st.act.resist_pois)
            || borg_activate_item(st, st.act.resist_all)
            || borg_activate_item(st, st.act.rage_bless_resist)
            || borg_quaff_potion(st, st.item_val.sv_potion_resist_pois)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 21000;
        }

        /* Value */
        return p1 - p2;
    }

    /* default to can't do it. */
    0
}

/// Protection from Evil (`borg_defend_aux_prot_evil`).
fn borg_defend_aux_prot_evil(st: &mut BorgState, mut p1: i32) -> i32 {
    let mut pfe_spell = false;

    /* if already protected */
    if st.borg.temp.prot_from_evil {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 5, 10);

    if borg_spell_okay_fail(st, PROTECTION_FROM_EVIL, fail_allowed) {
        pfe_spell = true;
    }

    if 0 <= borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_protection_from_evil) {
        pfe_spell = true;
    }

    if tr(st, BI_ISBLIND) != 0 || tr(st, BI_ISCONFUSED) != 0 || tr(st, BI_ISIMAGE) != 0 {
        pfe_spell = false;
    }

    if dark_here(st) {
        pfe_spell = false;
    }

    if borg_equips_item(st, st.act.protevil, true) {
        pfe_spell = true;
    }

    if !pfe_spell {
        return 0;
    }

    /* elemental and PFE use the 'averaging' method for danger.  Redefine p1 as
     * such. */
    p1 = danger_here(st, false);

    /* pretend we are protected and look again */
    st.borg.temp.prot_from_evil = true;
    let p2 = danger_here(st, false);
    st.borg.temp.prot_from_evil = false;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7)
        || (st.core.cfg[BORG_MONEY_SCUM_AMOUNT as usize] >= 1 && tr(st, BI_CDEPTH) == 0)
    {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to cast PFE");

        /* do it! */
        if borg_spell_fail(st, PROTECTION_FROM_EVIL, fail_allowed)
            || borg_activate_item(st, st.act.protevil)
            || borg_read_scroll(st, st.item_val.sv_scroll_protection_from_evil)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = tr(st, BI_CLEVEL) * 1000;
        }

        /* Value */
        return p1 - p2;
    }

    /* default to can't do it. */
    0
}

/// Shield (`borg_defend_aux_shield`).
fn borg_defend_aux_shield(st: &mut BorgState, p1: i32) -> i32 {
    /* if already protected */
    if st.borg.temp.shield {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    if st.borg.has[st.item_val.kv_mush_stoneskin as usize] <= 0
        && !borg_equips_item(st, st.act.shroom_stone, true)
    {
        return 0;
    }

    /* pretend we are protected and look again */
    st.borg.temp.shield = true;
    let mut p2 = danger_here(st, true);
    st.borg.temp.shield = false;

    /* slightly enhance the value if fighting a unique */
    if st.borg.near.unique != 0 {
        p2 = p2 * 7 / 10;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        borg_note(st, "# Attempting to eat a stone skin");

        /* do it! */
        if borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_stoneskin)
            || borg_activate_item(st, st.act.shroom_stone)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 2000;
            return p1 - p2;
        }
    }

    /* default to can't do it. */
    0
}

/// Try to get rid of all of the non-uniques around so you can go at it
/// 'mano-e-mano' with the unique. Teleport Other
/// (`borg_defend_aux_tele_away`).
fn borg_defend_aux_tele_away(st: &mut BorgState, p1: i32) -> i32 {
    let mut fail_allowed = 50;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    let a = avoid(st);

    /* Only tport monster away if scared or getting low on mana */
    if st.borg.near.unique != 0 {
        if p1 < a * 7 / 10 && tr(st, BI_CURSP) > 30 && simulating(st) {
            return 0;
        }
    } else if p1 < a * 5 / 10 && tr(st, BI_CURSP) > 30 && simulating(st) {
        return 0;
    }

    /* No real Danger to speak of */
    if p1 < a * 4 / 10 && simulating(st) {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    if p1 > a * 3 {
        fail_allowed -= 10;
    } else if p1 > a * 2 {
        /* scary */
        fail_allowed -= 5;
    } else if p1 > (a * 5) / 2 {
        /* a little scary */
        fail_allowed += 5;
    }

    /* do I have the ability? */
    let spell_ok = borg_spell_okay_fail(st, TELEPORT_OTHER, fail_allowed)
        || borg_equips_item(st, st.act.tele_other, true)
        || (-1 != borg_slot(st, TV_WAND as i32, st.item_val.sv_wand_teleport_away) && {
            let s = borg_slot(st, TV_WAND as i32, st.item_val.sv_wand_teleport_away);
            st.items.items[s as usize].pval != 0
        })
        || borg_equips_rod(st, st.item_val.sv_rod_teleport_other);

    if !spell_ok {
        return 0;
    }

    /* No Teleport Other if surrounded */
    if borg_surrounded(st) {
        return 0;
    }

    /* Borg_temp_n temporarily stores several things.
     * Some of the borg_attack() sub-routines use these numbers,
     * which would have been filled in borg_attack().
     * Since this is a defense maneuver which will move into
     * and borrow some of the borg_attack() subroutines, we need
     * to make sure that the borg_temp_n arrays are properly
     * filled.  Otherwise, the borg will attempt to consider
     * these grids which were left filled by some other routine.
     * Which was probably a flow routine which stored about 200
     * grids into the array.
     * Any change in inclusion/exclusion criteria for filling this
     * array in borg_attack() should be included here also.
     */
    /* Nobody around so don't worry */
    if st.kills.cnt == 0 && simulating(st) {
        return 0;
    }

    /* Reset list */
    reset_lists(st);

    /* Find "nearby" monsters */
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Get grid */
        let ag = grid(st, kill.pos.x, kill.pos.y);

        /* Never shoot off-screen */
        if ag.info & BORG_OKAY == 0 {
            continue;
        }

        /* Never shoot through walls */
        if ag.info & BORG_VIEW == 0 {
            continue;
        }
        if u32::from(ag.feat) >= FEAT_RUBBLE && u32::from(ag.feat) <= FEAT_PERM {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, kill.pos) > max_range() {
            continue;
        }

        /* Save the location (careful) */
        let n = st.flow.temp_n as usize;
        st.flow.temp_x[n] = kill.pos.x as u8;
        st.flow.temp_y[n] = kill.pos.y as u8;
        st.flow.temp_n += 1;
    }

    /* No targets for me. */
    if st.flow.temp_n == 0 && simulating(st) {
        return 0;
    }

    /* choose, then target a bad guy.
     * Damage will be the danger to my grid which the monster creates.
     * We are targeting the single most dangerous monster.
     * p1 will be the original danger. p2 is how much p1 is reduced by
     * the teleported monsters. ie:  if we are fighting only a single monster
     * is generating 500 danger and we target him, then p2 _should_
     * end up 500, since p1 - his danger is 500-0. If we are fighting two guys
     * each creating 500 danger, then p2 will be 500, since 1000-500 = 500.
     */
    let p2 = borg_launch_bolt(st, 0, p1, BORG_ATTACK_AWAY_ALL as i32, max_range(), 0);
    if p2 <= 0 {
        return 0;
    }

    /* check to see if I am left better off */
    if simulating(st) {
        /* Reset list */
        reset_lists(st);

        if p2 != 0 && p2 > avoid(st) / 2 {
            /* Simulation */
            return p2;
        } else {
            return 0;
        }
    }

    /* Log the Path for Debug */
    borg_log_spellpath(st);

    /* Log additional info for debug */
    let mut i = 0;
    while i < i32::from(st.fight_attack.tp_other_n) {
        let n = st.fight_attack.tp_other_n;
        let idx = st.fight_attack.tp_other_index[i as usize];
        let (ty, tx) = (st.fight_attack.tp_other_y[i as usize], st.fight_attack.tp_other_x[i as usize]);
        borg_note(st, &format!("# T.O. {}, index {} ({},{})", n, idx, ty, tx));
        i += 1;
    }

    /* Reset list */
    reset_lists(st);

    /* Cast the spell */
    if borg_spell(st, TELEPORT_OTHER)
        || borg_activate_item(st, st.act.tele_other)
        || borg_aim_wand(st, st.item_val.sv_wand_teleport_away)
        || borg_zap_rod(st, st.item_val.sv_rod_teleport_other)
    {
        /* Use target */
        borg_keypress(st, key(b'5'));

        /* Set our shooting flag */
        st.fight_attack.successful_target = -1;

        /* Value */
        return p2;
    }

    0
}

/// Hero to prepare for battle, +12 tohit (`borg_defend_aux_hero`).
fn borg_defend_aux_hero(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 15;

    /* already hero */
    if st.borg.temp.hero {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Heroism part of the heroism spell only kicks in after level 19 */
    let spell = borg_spell_okay_fail(st, HEROISM, fail_allowed)
        && tr(st, BI_CLEVEL) >= borg_heroism_level(st);
    let potion = -1 != borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_heroism);
    if !potion && !spell {
        return 0;
    }

    /* if we are in some danger but not much, go for a quick bless */
    /* "some danger" defined as "10% of x and not more than 50% of x */
    /* (not more than 70% when fighting a unique) */
    /* where x is the danger we are avoiding, usually current hp */
    if some_danger(st, p1) {
        /* Simulation */
        /* hero is a low priority */
        if simulating(st) {
            return 1;
        }

        borg_note(st, "# Attempting to cast Hero");

        /* do it! */
        if (spell && borg_spell(st, HEROISM))
            || borg_quaff_potion(st, st.item_val.sv_potion_heroism)
        {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 10000;
            return 1;
        }
    }

    0
}

/// Rapid Regen to prepare for battle (`borg_defend_aux_regen`).
fn borg_defend_aux_regen(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 15;

    /* already regenerating */
    if st.borg.temp.regen {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* don't bother if not much to regenerate */
    if tr(st, BI_MAXHP) < 100 {
        return 0;
    }

    if !borg_spell_okay_fail(st, RAPID_REGENERATION, fail_allowed) {
        return 0;
    }

    /* if we are in some danger but not much, go for a quick regen */
    if some_danger(st, p1) {
        /* Simulation */
        /* regen is a low priority */
        if simulating(st) {
            return 1;
        }

        /* do it! */
        if borg_spell(st, RAPID_REGENERATION) {
            /* No resting to recoup mana */
            st.borg.no_rest_prep = 10000;
            return 1;
        }
    }

    0
}

/// Berserk to prepare for battle, +24 tohit, -10 AC
/// (`borg_defend_aux_berserk`).
fn borg_defend_aux_berserk(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 15;

    /* already berserk */
    if st.borg.temp.berserk {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, BERSERK_STRENGTH, fail_allowed)
        && -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_berserk)
        && !borg_equips_item(st, st.act.berserker, true)
        && !borg_equips_item(st, st.act.rage_bless_resist, true)
        && !borg_equips_item(st, st.act.shero, true)
    {
        return 0;
    }

    /* if we are in some danger but not much, go for a quick bless */
    if some_danger(st, p1) {
        /* Simulation */
        /* berserk is a low priority */
        if simulating(st) {
            return 5;
        }

        /* do it! */
        if borg_spell(st, BERSERK_STRENGTH)
            || borg_activate_item(st, st.act.berserker)
            || borg_activate_item(st, st.act.rage_bless_resist)
            || borg_activate_item(st, st.act.shero)
            || borg_quaff_potion(st, st.item_val.sv_potion_berserk)
        {
            return 5;
        }
    }

    0
}

/// See if the borg is near something evil (`near_evil`).
fn near_evil(st: &BorgState) -> bool {
    /* Find "nearby" monsters */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = &st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(kill.r_idx) {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Get grid */
        let ag = st.cave.grid(kill.pos.x, kill.pos.y);

        /* don't count off-screen */
        if ag.info & BORG_OKAY == 0 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, kill.pos) > 3 {
            continue;
        }

        /* Monster race is evil */
        if engine::race_flag_has(usize::from(kill.r_idx), RF_EVIL) {
            return true;
        }
    }

    false
}

/// Smite Evil to prepare for battle (`borg_defend_aux_smite_evil`).
fn borg_defend_aux_smite_evil(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 15;

    /* already smiting evil */
    if st.borg.temp.smite_evil || tr(st, BI_WS_EVIL) != 0 {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    if !borg_spell_okay_fail(st, SMITE_EVIL, fail_allowed) {
        return 0;
    }

    /* if the borg is not about to fight something evil. */
    if !near_evil(st) {
        return 0;
    }

    /* if we are in some danger but not much, go for a quick smite */
    if some_danger(st, p1) {
        /* Simulation */
        /* smite evil is a low priority */
        if simulating(st) {
            return 5;
        }

        /* do it! */
        if borg_spell(st, SMITE_EVIL) {
            return 5;
        }
    }

    0
}

/// Glyph of Warding and Rune of Protection (`borg_defend_aux_glyph`).
fn borg_defend_aux_glyph(st: &mut BorgState, p1: i32) -> i32 {
    let mut glyph_spell = false;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* He should not cast it while on an object.
     * I have addressed this inadequately in borg9.c when dealing with
     * messages.  The message "the object resists" will delete the glyph
     * from the array.  Then I set a broken door on that spot, the borg ignores
     * broken doors, so he won't loop.
     */
    let ag = grid(st, st.borg.c.x, st.borg.c.y);
    let feat = u32::from(ag.feat);
    if ag.take != 0
        || ag.trap
        || feat == FEAT_LESS
        || feat == FEAT_MORE
        || feat == FEAT_OPEN
        || feat == FEAT_BROKEN
    {
        return 0;
    }

    /* Morgoth breaks these in one try so its a waste of mana against him */
    if st.borg.near.unique >= 10 {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 5, 20);

    if borg_spell_okay_fail(st, GLYPH_OF_WARDING, fail_allowed) {
        glyph_spell = true;
    }

    if 0 <= borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_rune_of_protection) {
        glyph_spell = true;
    }

    if borg_equips_item(st, st.act.glyph, true) {
        glyph_spell = true;
    }

    if (tr(st, BI_ISBLIND) != 0 || tr(st, BI_ISCONFUSED) != 0 || tr(st, BI_ISIMAGE) != 0)
        && glyph_spell
    {
        glyph_spell = false;
    }

    if dark_here(st) {
        glyph_spell = false;
    }

    if !glyph_spell {
        return 0;
    }

    /* pretend we are protected and look again */
    st.traits.on_glyph = true;
    let p2 = danger_here(st, true);
    st.traits.on_glyph = false;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        /* do it! */
        if borg_spell_fail(st, GLYPH_OF_WARDING, fail_allowed)
            || borg_read_scroll(st, st.item_val.sv_scroll_rune_of_protection)
            || borg_activate_item(st, st.act.glyph)
        {
            /* Check for an existing glyph */
            for i in 0..st.glyph.track.num as usize {
                /* Stop if we already new about this glyph */
                if st.glyph.track.x[i] == st.borg.c.x && st.glyph.track.y[i] == st.borg.c.y {
                    return p1 - p2;
                }
            }

            /* Track the newly discovered glyph */
            if st.glyph.track.num < st.glyph.track.size {
                borg_note(st, "# Noting the creation of a glyph.");
                let n = st.glyph.track.num as usize;
                st.glyph.track.x[n] = st.borg.c.x;
                st.glyph.track.y[n] = st.borg.c.y;
                st.glyph.track.num += 1;
            }
            return p1 - p2;
        }
    }

    /* default to can't do it. */
    0
}

/// Create Door (`borg_defend_aux_create_door`).
fn borg_defend_aux_create_door(st: &mut BorgState, p1: i32) -> i32 {
    let mut door_bad = 0;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* any summoners near?*/
    if !st.borg.near.summoner {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 30, 5, 20);

    if !borg_spell_okay_fail(st, DOOR_CREATION, fail_allowed) {
        return 0;
    }

    /* Do not cast if surounded by doors or something */
    /* Get grid */
    for door_x in -1..=1 {
        for door_y in -1..=1 {
            /* Acquire location */
            let x = door_x + st.borg.c.x;
            let y = door_y + st.borg.c.y;

            let ag = grid(st, x, y);
            let feat = u32::from(ag.feat);

            /* track spaces already protected */
            if ag.glyph
                || ag.kill != 0
                || (feat == FEAT_GRANITE || feat == FEAT_PERM || feat == FEAT_CLOSED)
            {
                door_bad += 1;
            }

            /* track spaces that cannot be protected */
            if ag.take != 0
                || ag.trap
                || feat == FEAT_LESS
                || feat == FEAT_MORE
                || feat == FEAT_OPEN
                || feat == FEAT_BROKEN
                || ag.kill != 0
            {
                door_bad += 1;
            }
        }
    }

    /* Track it */
    /* lets make sure that we going to be benifited */
    if door_bad >= 6 {
        /* not really worth it.  Only 2 spaces protected */
        return 0;
    }

    /* pretend we are protected and look again */
    st.traits.create_door = true;
    let p2 = danger_here(st, true);
    st.traits.create_door = false;

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if improves(st, p1, p2, 7) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        /* do it! */
        if borg_spell_fail(st, DOOR_CREATION, fail_allowed) {
            /* Set the breeder flag to keep doors closed. Avoid summons */
            st.borg.near.breeder = true;

            /* Must make a new Sea too */
            st.glyph.needs_new_sea = true;

            /* Value */
            return p1 - p2;
        }
    }

    /* default to can't do it. */
    0
}

/// This will simulate and cast the mass genocide spell
/// (`borg_defend_aux_mass_genocide`).
fn borg_defend_aux_mass_genocide(st: &mut BorgState, p1: i32) -> i32 {
    let mut hit = 0;
    let mut b_p = 0;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* see if prayer is legal */
    if !borg_spell_okay_fail(st, MASS_BANISHMENT, 40)
        && !borg_equips_item(st, st.act.banishment, true)
        && tr(st, BI_AMASSBAN) == 0
    {
        /* Mass Banishment scroll */
        return 0;
    }

    /* See if he is in real danger */
    if p1 < avoid(st) * 12 / 10 && simulating(st) {
        return 0;
    }

    /* Find a monster and calculate its danger */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(kill.r_idx) {
            continue;
        }

        /* Check the distance */
        if engine::distance(st.borg.c, kill.pos) > 20 {
            continue;
        }

        /* we try not to genocide uniques */
        if engine::race_flag_has(usize::from(kill.r_idx), RF_UNIQUE) {
            continue;
        }

        /* Calculate danger */
        let p = danger_one_here(st, i, true);

        /* store the danger for this type of monster */
        b_p += p;
        hit += 3;
    }

    /* normalize the value */
    let mut p2 = p1 - b_p;
    if p2 < 0 {
        p2 = 0;
    }

    /* if strain (plus a pad in case we did not know about some monsters)
     * is greater than hp, don't cast it
     */
    if (hit * 12 / 10) >= tr(st, BI_CURHP) {
        return 0;
    }

    /* Penalize the strain from casting the spell */
    p2 += hit;

    /* Be more likely to use this if fighting Morgoth */
    if st.borg.near.unique >= 10 && (hit / 3 > 8) {
        p2 = p2 * 6 / 10;
    }

    /* if this is an improvement and we may not avoid monster now and */
    /* we may have before */
    if p1 > p2 && p2 <= limit(st) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }

        /* Cast the spell */
        if borg_read_scroll(st, st.item_val.sv_scroll_mass_banishment)
            || borg_activate_item(st, st.act.banishment)
            || borg_spell(st, MASS_BANISHMENT)
        {
            /* Remove monsters from the borg_kill */
            for i in 1..i32::from(st.kills.nxt) {
                /* Monster */
                let r_idx = st.kills.kills[i as usize].r_idx;

                /* dead monsters */
                if r_idx == 0 {
                    continue;
                }

                /* "player ghosts" */
                if is_ghost(r_idx) {
                    continue;
                }

                /* Cant kill uniques like this */
                if engine::race_flag_has(usize::from(r_idx), RF_UNIQUE) {
                    continue;
                }

                /* remove this monster */
                borg_delete_kill(st, i);
            }

            /* Value */
            return p1 - p2;
        }
    }
    /* Not worth it */
    0
}

/// This will simulate and cast the genocide spell
/// (`borg_defend_aux_genocide`).
///
/// There are two separate functions happening here.
/// 1. will genocide the race which is immediately threatening the borg.
/// 2. will genocide the race which is most dangerous on the level.  Though it
///    may not be threatening the borg right now.  It was considered to nuke
///    the escorts of a unique. But it could also be used to nuke a race if it
///    becomes too dangerous, for example a summoner called up 15-20 hounds,
///    and they must be dealt with. The first option may be called at any
///    time.  While the 2nd option is only called when the borg is in
///    relatively good health.
fn borg_defend_aux_genocide(st: &mut BorgState, mut p1: i32) -> i32 {
    /* power of danger without this kill */
    let mut p_without_kill = 0;

    /* arrays based on kill's char (`UCHAR_MAX` entries, as C) */
    let mut b_kill_count = [0i32; UCHAR_MAX];
    /* current danger to the borg */
    let mut b_danger = [0i32; UCHAR_MAX];
    /* danger in general */
    let mut b_threat = [0i32; UCHAR_MAX];

    let mut total_danger_to_me = 0;

    /* C: `char`, so a race character >= 128 is negative */
    let mut tmp_genocide_target: i8 = 0;

    /* if very scary, do not allow for much chance of fail */
    let fail_allowed = scaled_fail(st, p1, 25, 10, 10);

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Normalize the p1 value.  It contains danger added from
     * regional fear and monster fear.  Which won't be counted
     * in the post-genocide checks
     */
    let (cy, cx) = (st.borg.c.y as usize, st.borg.c.x as usize);
    let region = i32::from(st.danger.fear_region[cy / 11][cx / 11]);
    if region != 0 {
        p1 -= region;
    }
    let fear = i32::from(st.danger.fear_monsters[cy][cx]);
    if fear != 0 {
        p1 -= fear;
    }

    /* Make sure I have the spell */
    let genocide_spell = borg_spell_okay_fail(st, BANISHMENT, fail_allowed)
        || borg_equips_item(st, st.act.banishment, true)
        || borg_equips_staff_fail(st, st.item_val.sv_staff_banishment)
        || (-1 != borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_banishment));

    if !genocide_spell {
        return 0;
    }

    /* Don't try it if really weak */
    if tr(st, BI_CURHP) <= 75 {
        return 0;
    }

    /* two methods to calculate the threat:
     * 1. cycle each character of monsters on the level
     *    sum collective threat of each char to the player
     * 2. cycle each character of monsters on the level
     *    sum collective threat of each char in general
     */

    /* Clear previous dangers */
    let mut biggest_danger: u8 = 0;
    let mut biggest_threat: u8 = 0;

    /* start with a danger of 1 so non-dangerous (0) will not be saved */
    let mut max_threat = 1;

    /* Find a monster and calculate its danger */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(kill.r_idx) {
            continue;
        }

        /* we try not to genocide uniques */
        if engine::race_flag_has(usize::from(kill.r_idx), RF_UNIQUE) {
            continue;
        }

        /* Calculate danger */
        /* Danger to me by this monster */
        let p = danger_one_here(st, i, true);

        /* Danger of this monster to his own grid */
        let p_threat = borg_danger_one_kill(st, kill.pos.y, kill.pos.x, 1, i, true, true);

        /* Our char of the monster.  This is used because genocide uses a */
        /* single char designation for selecting the kill */
        let kill_char = engine::race(usize::from(kill.r_idx)).d_char as u8 as usize;

        /* total danger to me */
        total_danger_to_me += p;

        /* store the danger for this type of monster */
        /* Danger to me */
        b_danger[kill_char] += p;
        /* Danger added to level */
        b_threat[kill_char] += p_threat;

        /* Store the count of this type of monster */
        b_kill_count[kill_char] += 1;
    }

    /* Now, see which race contributes the most danger
     * both to me and danger on the level
     */
    for i in 0..UCHAR_MAX {
        /* skip the ones not found on this level */
        if b_kill_count[i] == 0 {
            continue;
        }

        /* for the race threatening me right now */
        if b_danger[i] > max_threat {
            /* track the race */
            max_threat = b_danger[i];
            biggest_danger = i as u8;

            /* note the danger with this race gone.  Note that the borg does max
             * his danger from a single monster at 2000 points.  It could be
             * much, much higher at depth 99 or so. What the borg should do is
             * recalculate the danger without considering this monster instead
             * of this hack which does not yield the true danger.
             */
            p_without_kill = total_danger_to_me - b_danger[usize::from(biggest_danger)];
        }

        /* for this race on the whole level */
        if b_threat[i] > max_threat {
            /* track the race */
            max_threat = b_threat[i];
            biggest_threat = i as u8;
        }

        /* Leave an interesting note for debugging */
        if !simulating(st) {
            let mut m = b"# Race '".to_vec();
            m.push(i as u8);
            m.extend_from_slice(
                format!(
                    "' is a threat with total danger {} from {} individuals.",
                    b_threat[i], b_kill_count[i]
                )
                .as_bytes(),
            );
            borg_note_bytes(st, &m);
        }
    }

    /* This will track and decide if it is worth genociding this dangerous race
     * for the level */
    if biggest_threat != 0 {
        /* Not if I am weak (should have 400 HP really in case of a Pit) */
        if tr(st, BI_CURHP) < 375 {
            biggest_threat = 0;
        }

        /* The threat must be real */
        if b_threat[usize::from(biggest_threat)] < tr(st, BI_MAXHP) * 3 {
            biggest_threat = 0;
        }

        /* Too painful to cast it (padded to be safe in case of unknown monsters)
         */
        if (b_kill_count[usize::from(biggest_threat)] * 4) * 12 / 10 >= tr(st, BI_CURHP) {
            biggest_threat = 0;
        }

        /* Loads of monsters might be a pit, in which case, try not to nuke them
         */
        if b_kill_count[usize::from(biggest_threat)] >= 75 {
            biggest_threat = 0;
        }

        /* Do not perform in Danger */
        if p1 > avoid(st) / 5 {
            biggest_threat = 0;
        }

        /* report the danger and most dangerous race */
        if biggest_threat != 0 {
            let mut m = b"# Race '".to_vec();
            m.push(biggest_threat);
            m.extend_from_slice(
                format!(
                    "' is a real threat with total danger {} from {} individuals.",
                    b_threat[usize::from(biggest_threat)],
                    b_kill_count[usize::from(biggest_threat)]
                )
                .as_bytes(),
            );
            borg_note_bytes(st, &m);
        }

        /* Genociding this race would reduce the danger of the level */
        tmp_genocide_target = biggest_threat as i8;
    }

    /* Consider the immediate threat genocide */
    if biggest_danger != 0 {
        /* Too painful to cast it (padded to be safe in case of unknown monsters)
         */
        if (b_kill_count[usize::from(biggest_danger)] * 4) * 12 / 10 >= tr(st, BI_CURHP) {
            biggest_danger = 0;
        }

        /* See if he is in real danger, generally,
         * or deeper in the dungeon, conservatively,
         */
        let a = avoid(st);
        if p1 < a * 7 / 10 || (tr(st, BI_CDEPTH) > 75 && p1 < a * 6 / 10) {
            biggest_danger = 0;
        }

        /* Did this help improve my situation? */
        if p_without_kill <= (a / 2) {
            biggest_danger = 0;
        }

        /* Genociding this race would help me immediately */
        if biggest_danger != 0 {
            tmp_genocide_target = biggest_danger as i8;
        }
    }

    /* Complete the genocide routine */
    if tmp_genocide_target != 0 {
        if simulating(st) {
            /* Simulation for immediate danger */
            if biggest_danger != 0 {
                return p1 - p_without_kill;
            }

            /* Simulation for threat to level */
            if biggest_threat != 0 {
                return b_threat[usize::from(biggest_threat)];
            }
        }

        if biggest_danger != 0 {
            let mut m = b"# Banishing race '".to_vec();
            m.push(tmp_genocide_target as u8);
            m.extend_from_slice(
                format!(
                    "' (qty:{}).  Danger after spell:{}",
                    b_kill_count[usize::from(biggest_danger)],
                    p_without_kill
                )
                .as_bytes(),
            );
            borg_note_bytes(st, &m);
        }
        if biggest_threat != 0 {
            let mut m = b"# Banishing race '".to_vec();
            m.push(tmp_genocide_target as u8);
            m.extend_from_slice(
                format!(
                    "' (qty:{}).  Danger from them:{}",
                    b_kill_count[usize::from(biggest_threat)],
                    b_threat[usize::from(biggest_threat)]
                )
                .as_bytes(),
            );
            borg_note_bytes(st, &m);
        }

        /* do it! ---use scrolls first since they clutter inventory */
        if borg_read_scroll(st, st.item_val.sv_scroll_banishment)
            || borg_spell(st, BANISHMENT)
            || borg_activate_item(st, st.act.banishment)
            || borg_use_staff(st, st.item_val.sv_staff_banishment)
        {
            /* and the winner is.....*/
            borg_keypress(st, tmp_genocide_target as keycode_t);

            /* Remove this race from the borg_kill */
            for i in 1..i32::from(st.kills.nxt) {
                /* Monster (dead ones too: they read r_info[0]) */
                let r_idx = usize::from(st.kills.kills[i as usize].r_idx);

                /* Our char of the monster */
                if engine::race(r_idx).d_char != i32::from(tmp_genocide_target) {
                    continue;
                }

                /* we do not genocide uniques */
                if engine::race_flag_has(r_idx, RF_UNIQUE) {
                    continue;
                }

                /* remove this monster */
                borg_delete_kill(st, i);
            }
        }

        /* immediate danger to borg */
        if biggest_danger != 0 {
            return p1 - p_without_kill;
        }

        /* threat to level */
        if biggest_threat != 0 {
            return b_threat[usize::from(biggest_threat)];
        }
    }
    /* default to can't do it. */
    0
}

/// This will cast the genocide spell on Hounds and other really nasty guys
/// like Ainur, Demons, Dragons and Liches at the beginning of each level or
/// when they get too numerous (`borg_defend_aux_genocide_nasties`). The
/// acceptable numbers are `borg_nasties_limit[]`; the list is
/// `borg_nasties[7] = "ZAVULWD"`.
fn borg_defend_aux_genocide_nasties(st: &mut BorgState, p1: i32) -> i32 {
    let mut b_i: i32 = -1;

    /* Not if I am weak */
    if tr(st, BI_CURHP) < (tr(st, BI_MAXHP) * 7 / 10) || tr(st, BI_CURHP) < 250 {
        return 0;
    }

    /* only do it when Hounds start to show up, */
    if tr(st, BI_CDEPTH) < 25 {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Do not perform in Danger */
    if p1 > avoid(st) / 4 {
        return 0;
    }

    let genocide_spell = borg_spell_okay_fail(st, BANISHMENT, 35)
        || borg_equips_item(st, st.act.banishment, true)
        || borg_equips_staff_fail(st, st.item_val.sv_staff_banishment);

    if !genocide_spell {
        return 0;
    }

    /* Find the numerous nasty in order of nastiness */
    for i in 0..usize::from(st.kills.nasties_num) {
        if st.kills.nasties_count[i] >= st.kills.nasties_limit[i] {
            b_i = i as i32;
        }
    }

    /* Nothing good to Genocide */
    if b_i == -1 {
        return 0;
    }

    if simulating(st) {
        return 10;
    }

    let b = b_i as usize;
    let nasty = st.kills.nasties[b];

    /* Note it */
    let mut m = b"# Banishing nasties '".to_vec();
    m.push(nasty);
    m.extend_from_slice(format!("' (qty:{}).", st.kills.nasties_count[b]).as_bytes());
    borg_note_bytes(st, &m);

    /* Execute -- Nice pun*/
    if borg_activate_item(st, st.act.banishment)
        || borg_use_staff(st, st.item_val.sv_staff_banishment)
        || borg_spell(st, BANISHMENT)
    {
        /* and the winner is.....*/
        borg_keypress(st, keycode_t::from(st.kills.nasties[b]));

        /* set the count to not do it again */
        st.kills.nasties_count[b] = 0;

        /* Remove this race from the borg_kill */
        for i in 1..i32::from(st.kills.nxt) {
            /* Monster (dead ones too: they read r_info[0]) */
            let r_idx = usize::from(st.kills.kills[i as usize].r_idx);

            /* Our char of the monster */
            if engine::race(r_idx).d_char != i32::from(st.kills.nasties[b]) {
                continue;
            }

            /* remove this monster */
            borg_delete_kill(st, i);
        }

        return 10;
    }

    /* default to can't do it. */
    0
}

/// Earthquake, priest and mage spells (`borg_defend_aux_earthquake`).
fn borg_defend_aux_earthquake(st: &mut BorgState, p1: i32) -> i32 {
    let mut p2 = 9999;
    let mut threat_count = 0;

    /* Cast the spell */
    if !simulating(st) {
        /* Must make a new Sea too */
        st.glyph.needs_new_sea = true;

        /* for now aim Tremor around the borg */
        if borg_spell(st, TREMOR) {
            borg_keypress(st, key(b'*'));
            borg_keypress(st, key(b'5'));
        } else if borg_spell(st, QUAKE)
            || borg_spell(st, GRONDS_BLOW)
            || borg_activate_item(st, st.act.earthquakes)
        {
            /* other spells don't need aim */
            return p2;
        }
    }

    /* Can't when screwed */
    if screwed(st) {
        return 0;
    }

    /* Can I cast the spell? */
    if !borg_spell_okay_fail(st, TREMOR, 35)
        && !borg_spell_okay_fail(st, QUAKE, 35)
        && !borg_spell_okay_fail(st, GRONDS_BLOW, 35)
        && !borg_equips_item(st, st.act.earthquakes, true)
    {
        return 0;
    }

    /* See if he is in real danger or fighting summoner*/
    if p1 < avoid(st) * 6 / 10 && !st.borg.near.summoner {
        return 0;
    }

    /* Several monsters can see the borg and they have ranged attacks */
    for i in 0..usize::from(st.kills.nxt as u16) {
        let kill = &st.kills.kills[i];

        /* Look for threats */
        if borg_los(st, st.borg.c.y, st.borg.c.x, kill.pos.y, kill.pos.x)
            && kill.ranged_attack != 0
            && engine::distance(kill.pos, st.borg.c) >= 2
        {
            /* They can hit me */
            threat_count += 1;
        }
    }

    /* Real danger? */
    let a = avoid(st);
    if threat_count >= 4 && p1 > a * 7 / 10 {
        p2 = p1 / 3;
    }
    if threat_count == 3 && p1 > a * 7 / 10 {
        p2 = p1 * 6 / 10;
    }

    if improves(st, p1, p2, 5) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }
    }
    0
}

/// Word of Destruction, priest and mage spells.  Death is right around the
/// corner, so kill everything (`borg_defend_aux_destruction`).
fn borg_defend_aux_destruction(st: &mut BorgState, p1: i32) -> i32 {
    let mut spell = false;
    let mut real_danger = false;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Cast the spell */
    if !simulating(st) {
        if borg_spell(st, WORD_OF_DESTRUCTION)
            || borg_use_staff(st, st.item_val.sv_staff_destruction)
            || borg_activate_item(st, st.act.destruction2)
        {
            /* Must make a new Sea too */
            st.glyph.needs_new_sea = true;

            /* borg9.c will check for the success of the spell and remove the
             * danger from the grids.
             */
        }

        return 500;
    }

    /* Not if in a sea of runes */
    if st.borg.morgoth_position {
        return 0;
    }

    /* See if he is in real danger */
    let a = avoid(st);
    if p1 > a {
        real_danger = true;
    }
    if p1 > a * 8 / 10 && tr(st, BI_CDEPTH) >= 90 && tr(st, BI_CURHP) <= 300 {
        real_danger = true;
    }

    if !real_danger {
        return 0;
    }

    /* Borg_defend() is called before borg_escape().  He may have some
     * easy ways to escape (teleport scroll) but he may attempt this spell
     * of Destruction instead of using the scrolls.
     * Note that there will be some times when it is better for
     * the borg to use Destruction instead of Teleport;  too
     * often he will die out-of-the-frying-pan-into-the-fire.
     * So we have him to a quick check on safe landing zones.
     */

    /* Examine landing zones from teleport scrolls instead of WoD */
    if (tr(st, BI_ATELEPORT) != 0 || tr(st, BI_ATELEPORTLVL) != 0)
        && tr(st, BI_ISBLIND) == 0
        && tr(st, BI_ISCONFUSED) == 0
        && st.borg.near.unique <= 4
        && tr(st, BI_CURHP) >= 275
        && borg_caution_teleport(st, 75, 2)
    {
        return 0;
    }

    /* Examine Landing zones from teleport staff instead of WoD */
    if tr(st, BI_AESCAPE) >= 2 && tr(st, BI_CURHP) >= 275 && borg_caution_teleport(st, 75, 2) {
        return 0;
    }

    /* capable of casting the spell */
    if borg_spell_okay_fail(st, WORD_OF_DESTRUCTION, 55)
        || borg_equips_staff_fail(st, st.item_val.sv_staff_destruction)
        || borg_equips_item(st, st.act.destruction2, true)
    {
        spell = true;
    }

    /* Special check for super danger--no fail check */
    if (p1 > (a * 4) || (p1 > a && tr(st, BI_CURHP) <= 150))
        && borg_equips_staff_fail(st, st.item_val.sv_staff_destruction)
    {
        spell = true;
    }

    if !spell {
        return 0;
    }

    /* What effect is there? */
    let p2 = 0;

    /* value is d */
    let mut d = p1 - p2;

    /* Try not to cast this against uniques */
    if st.borg.near.unique <= 2 && p1 < a * 2 {
        d = 0;
    }
    if st.borg.near.unique >= 10 {
        d = 0;
    }

    /* Simulation */
    if simulating(st) {
        return d;
    }

    0
}

/// Teleport Level, priest and mage spells.  Death is right around the
/// corner, Get off the level now (`borg_defend_aux_teleportlevel`).
fn borg_defend_aux_teleportlevel(st: &mut BorgState, p1: i32) -> i32 {
    /* Cast the spell */
    if !simulating(st) && borg_spell(st, TELEPORT_LEVEL) {
        /* Must make a new Sea too */
        st.glyph.needs_new_sea = true;
        return 500;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* See if he is in real danger */
    if p1 < avoid(st) * 2 {
        return 0;
    }

    /* Borg_defend() is called before borg_escape().  He may have some
     * easy ways to escape (teleport scroll) but he may attempt this spell
     * of this spell instead of using the scrolls.
     * Note that there will be some times when it is better for
     * the borg to use this instead of Teleport;  too
     * often he will die out-of-the-frying-pan-into-the-fire.
     * So we have him to a quick check on safe landing zones.
     */

    /* Use teleport scrolls instead if safe to land */
    if (tr(st, BI_ATELEPORT) != 0 || tr(st, BI_ATELEPORTLVL) != 0)
        && tr(st, BI_ISBLIND) == 0
        && tr(st, BI_ISCONFUSED) == 0
        && borg_caution_teleport(st, 65, 2)
    {
        return 0;
    }

    /* Use teleport staff instead if safe to land */
    if tr(st, BI_AESCAPE) >= 2 && borg_caution_teleport(st, 65, 2) {
        return 0;
    }

    /* capable of casting the spell */
    if !borg_spell_okay_fail(st, TELEPORT_LEVEL, 55) {
        return 0;
    }

    /* Try not to cast this against special uniques */
    if st.borg.near.morgoth || (st.borg.near.unique >= 1 && st.borg.status.anti_summon) {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return p1;
    }

    0
}

/// The kill's danger to the Borg's grid for the banishment notes:
/// `borg_danger_one_kill(borg.c.y, borg.c.x, 1, ag->kill, true, full)`,
/// where `ag` is the grid the kill is on (C bug: not always kill `i`).
fn banish_note(st: &mut BorgState, kill_idx: i32, full_damage: bool, what: &str) {
    let kill = st.kills.kills[kill_idx as usize];
    let ag_kill = i32::from(grid(st, kill.pos.x, kill.pos.y).kill);
    let d = danger_one_here(st, ag_kill, full_damage);
    let mut m = format!("# Banishing Evil: ({},{}): ", kill.pos.y, kill.pos.x).into_bytes();
    m.extend_from_slice(borg_race_name(i32::from(kill.r_idx)).to_bytes());
    m.extend_from_slice(format!(", danger {}. {}", d, what).as_bytes());
    borg_note_bytes(st, &m);
}

/// Remove Evil guys within LOS.  The Priest Spell
/// (`borg_defend_aux_banishment`).
fn borg_defend_aux_banishment(st: &mut BorgState, mut p1: i32) -> i32 {
    let mut fail_allowed = 15;
    let mut banished_monsters = 0;

    let a = avoid(st);

    /* Only tell away if scared */
    if p1 < a / 10 {
        return 0;
    }

    /* if very scary, do not allow for much chance of fail */
    if p1 > a * 4 {
        fail_allowed -= 10;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    let using_artifact = borg_equips_item(st, st.act.loskill, true) && st.borg.tr(BI_CURHP) > 100;

    if !using_artifact && !borg_spell_okay_fail(st, BANISH_EVIL, fail_allowed) {
        return 0;
    }

    /* reset initial danger */
    p1 = 1;

    /* Two passes to determine exact danger */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Check the LOS */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        if !borg_projectable(st, cy, cx, kill.pos.y, kill.pos.x) {
            continue;
        }

        /* Calculate danger of who is left over */
        p1 += danger_one_here(st, i, true);
    }

    /* Set P2 to be P1 and subtract the danger from each monster
     * which will be booted.  Non booted monsters won't decrement
     * the p2
     */
    let mut p2 = p1;

    /* Pass two -- Find a monster and calculate its danger */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(kill.r_idx) {
            continue;
        }

        /* Check the LOS */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        if !borg_projectable(st, cy, cx, kill.pos.y, kill.pos.x) {
            continue;
        }

        /* Note who gets considered */
        if !simulating(st) {
            banish_note(st, i, false, "is considered.");
        }

        /* Non evil monsters*/
        if !engine::race_flag_has(usize::from(kill.r_idx), RF_EVIL) {
            /* Note who gets to stay */
            if !simulating(st) {
                banish_note(st, i, false, "Stays (not evil).");
            }

            continue;
        }

        /* Unique Monster in good health*/
        if engine::race_flag_has(usize::from(kill.r_idx), RF_UNIQUE) && kill.injury > 60 {
            /* Note who gets to stay */
            if !simulating(st) {
                let what = format!("Unique not considered: Injury {}.", kill.injury);
                banish_note(st, i, false, &what);
            }

            continue;
        }

        /* Monsters in walls cant be booted */
        if !borg_cave_floor_bold(st, kill.pos.y, kill.pos.x) {
            /* Note who gets banished */
            if !simulating(st) {
                banish_note(st, i, true, "Stays (in wall).");
            }
            continue;
        }

        /* Note who gets banished */
        if !simulating(st) {
            banish_note(st, i, true, "Booted.");
            borg_delete_kill(st, i);
        }

        /* Count */
        banished_monsters += 1;

        /* Calculate danger of who is left over */
        p2 -= danger_one_here(st, i, true);
    }

    if !simulating(st) {
        /* attempt the banish */
        if using_artifact && borg_activate_item(st, st.act.loskill) {
            return p1 - p2;
        }
        if borg_spell(st, BANISH_EVIL) {
            return p1 - p2;
        }
    }

    /* p2 is the danger after all the bad guys are removed. */
    /* no negatives */
    if p2 <= 0 {
        p2 = 0;
    }

    /* No monsters get booted */
    if banished_monsters == 0 {
        p2 = 9999;
    }

    /* Try not to cast this against Morgy/Sauron */
    if st.borg.near.unique >= 10 && tr(st, BI_CURHP) > 250 && tr(st, BI_CDEPTH) == 99 {
        p2 = 9999;
    }
    if st.borg.near.unique >= 10 && tr(st, BI_CURHP) > 350 && tr(st, BI_CDEPTH) == 100 {
        p2 = 9999;
    }

    /* check to see if I am left better off */
    if p1 > p2 && p2 <= limit(st) {
        /* Simulation */
        if simulating(st) {
            return p1 - p2;
        }
    }
    0
}

/// Detect Inviso/Monsters (`borg_defend_aux_inviso`). Used only if I am hit
/// by an unseen guy. Casts detect invis.
fn borg_defend_aux_inviso(st: &mut BorgState, p1: i32) -> i32 {
    let fail_allowed = 25;

    /* no need */
    if tr(st, BI_ISFORGET) != 0
        || tr(st, BI_ISBLIND) != 0
        || tr(st, BI_ISCONFUSED) != 0
        || st.borg.see_inv != 0
    {
        return 0;
    }

    /* not recent */
    if borg_timer(st, st.borg.need_see_invis) > 45 {
        return 0;
    }

    /* too dangerous to cast */
    if p1 > avoid(st) * 2 {
        return 0;
    }

    /* Do I have anything that will work? */
    if -1 == borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_detect_invis)
        && -1 == borg_slot(st, TV_SCROLL as i32, st.item_val.sv_scroll_detect_invis)
        && !borg_equips_staff_fail(st, st.item_val.sv_staff_detect_invis)
        && !borg_equips_staff_fail(st, st.item_val.sv_staff_detect_evil)
        && !borg_spell_okay_fail(st, SENSE_INVISIBLE, fail_allowed)
        && !borg_spell_okay_fail(st, DETECTION, fail_allowed)
        && !borg_spell_okay_fail(st, REVEAL_MONSTERS, fail_allowed)
        && !borg_equips_item(st, st.act.detect_invis, true)
        && !borg_equips_item(st, st.act.tmd_sinvis, true)
        && !borg_equips_item(st, st.act.tmd_esp, true)
        && !borg_equips_item(st, st.act.detect_evil, true)
    {
        return 0;
    }

    /* Darkness */
    if dark_here(st) {
        return 0;
    }

    /* No real value known, but lets cast it to find the bad guys. */
    if simulating(st) {
        return 10;
    }

    /* smoke em if you got em */
    /* short time */
    /* snap shot */
    if borg_spell_fail(st, REVEAL_MONSTERS, fail_allowed)
        || borg_spell_fail(st, DETECTION, fail_allowed)
        || borg_read_scroll(st, st.item_val.sv_scroll_detect_invis)
        || borg_use_staff(st, st.item_val.sv_staff_detect_invis)
        || borg_use_staff(st, st.item_val.sv_staff_detect_evil)
        || borg_activate_item(st, st.act.detect_invis)
        || borg_activate_item(st, st.act.tmd_sinvis)
        || borg_activate_item(st, st.act.tmd_esp)
        || borg_activate_item(st, st.act.detect_evil)
    {
        /* hack, actually a snap shot, no ignition message */
        st.borg.see_inv = 3000;
        return 10;
    }
    if borg_quaff_potion(st, st.item_val.sv_potion_detect_invis) {
        st.borg.see_inv = 18000;
        st.borg.no_rest_prep = 18000;
        return 10;
    }
    /* long time */
    if borg_spell_fail(st, SENSE_INVISIBLE, fail_allowed) {
        st.borg.see_inv = 30000;
        st.borg.no_rest_prep = 16000;
        return 10;
    }

    /* ah crap, I guess I won't be able to see them */
    0
}

/// Light Beam to spot lurkers (`borg_defend_aux_lbeam`). Used only if I am
/// hit by an unseen guy. Lights up a hallway.
fn borg_defend_aux_lbeam(st: &mut BorgState, p1: i32) -> i32 {
    let mut hallway = false;
    let x = st.borg.c.x;
    let y = st.borg.c.y;

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Light Beam section to spot non seen guys */
    /* not recent, don't bother */
    if borg_timer(st, st.borg.need_see_invis) < 48 {
        return 0;
    }

    let floor = |dy: i32, dx: i32| borg_cave_floor_bold(st, y + dy, x + dx);

    /* Check to see if I am in a hallway */
    /* Case 1a: north-south corridor */
    if floor(-1, 0)
        && floor(1, 0)
        && !floor(0, -1)
        && !floor(0, 1)
        && !floor(1, -1)
        && !floor(1, 1)
        && !floor(-1, -1)
        && !floor(-1, 1)
    {
        /* ok to light up */
        hallway = true;
    }

    /* Case 1b: east-west corridor */
    if floor(0, -1)
        && floor(0, 1)
        && !floor(-1, 0)
        && !floor(1, 0)
        && !floor(1, -1)
        && !floor(1, 1)
        && !floor(-1, -1)
        && !floor(-1, 1)
    {
        /* ok to light up */
        hallway = true;
    }

    /* Case 1aa: north-south doorway */
    if floor(-1, 0) && floor(1, 0) && !floor(0, -1) && !floor(0, 1) {
        /* ok to light up */
        hallway = true;
    }

    /* Case 1ba: east-west doorway */
    if floor(0, -1) && floor(0, 1) && !floor(-1, 0) && !floor(1, 0) {
        /* ok to light up */
        hallway = true;
    }

    /* not in a hallway */
    if !hallway {
        return 0;
    }

    /* Make sure I am not in too much danger */
    if simulating(st) && p1 > (avoid(st) * 3) / 4 {
        return 0;
    }

    /* test the beam function */
    if !borg_light_beam(st, true) {
        return 0;
    }

    /* return some value */
    if simulating(st) {
        return 10;
    }

    /* if in a hallway call the Light Beam routine */
    if borg_light_beam(st, false) {
        return 10;
    }
    0
}

/// The keys and note of a panel shift "as a precaution", with the time
/// mark (one of the four copied blocks of `borg_defend_panel_shift`).
fn shift_as_precaution(st: &mut BorgState, dir: i32) {
    /* Send action (view panel info) */
    borg_keypress(st, key(b'L'));
    borg_keypress(st, i2d(dir));
    borg_note(st, "# Shifted panel as a precaution.");
    /* Mark the time to avoid loops */
    st.borg.when_shift_panel = st.borg.time.now;
    /* Leave the panel shift mode */
    borg_keypress(st, ESCAPE);
}

/// Shift the panel to locate offscreen monsters
/// (`borg_defend_panel_shift`).
fn borg_defend_panel_shift(st: &mut BorgState) -> bool {
    let mut dir = 0;

    let mut panel_num_x = 1;
    let mut panel_num_y = 1;

    let term = engine::term_geometry();
    let pos_on_panel_x = st.borg.c.x - term.offset_x;
    let pos_on_panel_y = st.borg.c.y - term.offset_y;

    /* no need */
    if !st.borg.need_shift_panel && tr(st, BI_CDEPTH) < 70 {
        return false;
    }

    /* if Morgy is on my panel, dont do it */
    if tr(st, BI_CDEPTH) == 100
        && st.core.w_y == st.borg.mon.morgoth_panel.y
        && st.core.w_x == st.borg.mon.morgoth_panel.x
    {
        return false;
    }

    if term.offset_x > borg_panel_wid() {
        panel_num_x = ((term.offset_x * 2) / borg_panel_wid()) + 1;
    }

    if term.offset_y > borg_panel_hgt() {
        panel_num_y = ((term.offset_y * 2) / borg_panel_hgt()) + 1;
    }

    /* Shift if within 5 spaces of the edge */
    /* Which direction do we need to move? */
    /* Shift panel to the right */
    if (borg_panel_wid() - pos_on_panel_x) < 5 && panel_num_x < st.borg.panels.x {
        dir = 6;
    }
    /* Shift panel to the left */
    if pos_on_panel_x < 5 && panel_num_x > 1 {
        dir = 4;
    }
    /* Shift panel to the down */
    if (borg_panel_hgt() - pos_on_panel_y) < 5 && panel_num_y < st.borg.panels.y {
        dir = 2;
    }
    /* Shift panel to the up */
    if pos_on_panel_y < 5 && panel_num_y > 1 {
        dir = 8;
    }

    if dir == 0 {
        return false;
    }

    /* Do the Shift if needed, then note it,  reset the flag */
    if st.borg.need_shift_panel {
        /* Send action (view panel info) */
        borg_keypress(st, key(b'L'));
        borg_keypress(st, i2d(dir));

        borg_note(st, "# Shifted panel to locate offscreen monster.");
        st.borg.need_shift_panel = false;

        /* Leave the panel shift mode */
        borg_keypress(st, ESCAPE);
        return true;
    }

    /* check to make sure its appropriate */
    /* Not if I just did one */
    if st.borg.when_shift_panel != 0
        && (borg_timer(st, st.borg.when_shift_panel) <= 10
            || borg_timer(st, st.borg.time.morgoth) <= 10)
    {
        /* do nothing */
        return false;
    }

    /* if not the first step */
    let ts = &st.flow.track_step;
    if ts.num == 0 {
        return false;
    }
    let last = (ts.num - 1) as usize;
    let (last_x, last_y) = (ts.x[last], ts.y[last]);
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);

    /* shift up? only if a north corridor */
    if (dir == 8 && borg_projectable_pure(st, cy, cx, cy - 2, cx) && last_y != cy - 1)
        /* shift down? only if a south corridor */
        || (dir == 2 && borg_projectable_pure(st, cy, cx, cy + 2, cx) && last_y != cy + 1)
        /* shift Left? only if a west corridor */
        || (dir == 4 && borg_projectable_pure(st, cy, cx, cy, cx - 2) && last_x != cx - 1)
        /* shift Right? only if a east corridor */
        || (dir == 6 && borg_projectable_pure(st, cy, cx, cy, cx + 2) && last_x != cx + 1)
    {
        /* (the four copied blocks of C) */
        shift_as_precaution(st, dir);
        return true;
    }

    false
}

/// This and the next routine is used on level 100 and when attacking
/// Morgoth (`borg_defend_aux_rest`). The borg has found a safe place to wait
/// for Morgoth to show.
///
/// If the borg is not being threatened immediately by a monster, then rest
/// right here.
///
/// Only borgs with teleport away and a good attack spell do this routine.
fn borg_defend_aux_rest(st: &mut BorgState) -> i32 {
    if !st.borg.morgoth_position
        && (!st.borg.status.anti_summon || borg_timer(st, st.borg.time.antisummon) >= 50)
    {
        return 0;
    }

    /* Not if Morgoth is not on this level */
    if !st.borg.near.morgoth
        && (!st.borg.status.anti_summon || borg_timer(st, st.borg.time.antisummon) >= 50)
    {
        return 0;
    }

    /* never in town */
    if tr(st, BI_CDEPTH) == 0 {
        return 0;
    }

    /* Not if I can not teleport others away */
    if borg_spell_okay_fail(st, TELEPORT_OTHER, 30) {
        return 0;
    }

    /* Not if a monster can see me */
    /* Examine all the monsters */
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Distance components */
        let ax = if x9 > st.borg.c.x { x9 - st.borg.c.x } else { st.borg.c.x - x9 };
        let ay = if y9 > st.borg.c.y { y9 - st.borg.c.y } else { st.borg.c.y - y9 };

        /* Distance */
        let d = ax.max(ay);

        /* Minimal distance */
        if d > max_range() {
            continue;
        }

        /* If I can see Morgoth or a guy with Ranged Attacks, don't rest. */
        if borg_los(st, st.borg.c.y, st.borg.c.x, kill.pos.y, kill.pos.x)
            && (u32::from(kill.r_idx) == st.borg.id.morgoth || kill.ranged_attack != 0)
            && avoid(st) <= tr(st, BI_CURHP)
        {
            borg_note(st, "# Not resting. I can see Morgoth or a shooter.");
            return 0;
        }

        /* If a little twitchy, its ok to stay put (C: `continue` at the end
         * of the loop body) */
    }

    /* Return some value for this rest */
    if simulating(st) {
        return 200;
    }

    /* Rest */
    borg_keypress(st, key(b','));
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    borg_note(st, &format!("# Resting on grid ({}, {}), waiting for Morgoth.", cy, cx));

    /* All done */
    200
}

/// Try to get rid of all of the monsters while I build my Sea of Runes
/// (`borg_defend_aux_tele_away_morgoth`).
fn borg_defend_aux_tele_away_morgoth(st: &mut BorgState) -> i32 {
    let fail_allowed = 40;

    /* Only if on level 100 */
    if tr(st, BI_CDEPTH) != 100 {
        return 0;
    }

    /* Not if Morgoth is not on this level */
    if !st.borg.near.morgoth {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Do I have the T.O. spell? */
    if !borg_spell_okay_fail(st, TELEPORT_OTHER, fail_allowed) {
        return 0;
    }

    /* Do I have the Glyph spell? No good to use TO if I cant build the sea of
     * runes */
    if tr(st, BI_AGLYPH) < 10 {
        return 0;
    }

    /* No Teleport Other if surrounded */
    if borg_surrounded(st) {
        return 0;
    }

    /* (see borg_defend_aux_tele_away about borg_temp_n) */

    /* Nobody around so dont worry */
    if st.kills.cnt == 0 && simulating(st) {
        return 0;
    }

    /* Reset list */
    reset_lists(st);

    /* Find "nearby" monsters */
    for i in 0..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Get grid */
        let ag = grid(st, kill.pos.x, kill.pos.y);

        /* Never shoot off-screen */
        if ag.info & BORG_OKAY == 0 {
            continue;
        }

        /* Never shoot through walls */
        if ag.info & BORG_VIEW == 0 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, kill.pos) > max_range() {
            continue;
        }

        /* Check the LOS */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        if !borg_projectable(st, cy, cx, kill.pos.y, kill.pos.x) {
            continue;
        }

        /* Save the location (careful) */
        let n = st.flow.temp_n as usize;
        st.flow.temp_x[n] = kill.pos.x as u8;
        st.flow.temp_y[n] = kill.pos.y as u8;
        st.flow.temp_n += 1;
    }

    /* No destinations */
    if st.flow.temp_n == 0 && simulating(st) {
        return 0;
    }

    /* choose then target a bad guy or several
     * If left as bolt, he targets the single most nasty guy.
     * If left as beam, he targets the collection of monsters.
     */
    let mut p2 =
        borg_launch_bolt(st, 0, 50, BORG_ATTACK_AWAY_ALL_MORGOTH as i32, max_range(), 0);

    /* Normalize the value a bit */
    if p2 > 1000 {
        p2 = 1000;
    }

    /* Reset list */
    reset_lists(st);

    /* Return a good score to make him do it */
    if simulating(st) {
        return p2;
    }

    /* Log the Path for Debug */
    borg_log_spellpath(st);

    /* Log additional info for debug (the list was just reset, as in C) */
    let mut i = 0;
    while i < i32::from(st.fight_attack.tp_other_n) {
        let n = st.fight_attack.tp_other_n;
        let idx = st.fight_attack.tp_other_index[i as usize];
        let (ty, tx) = (st.fight_attack.tp_other_y[i as usize], st.fight_attack.tp_other_x[i as usize]);
        borg_note(st, &format!("# {}, index {} ({},{})", n, idx, ty, tx));
        i += 1;
    }

    borg_note(st, "# Attempting to cast T.O. for depth 100.");

    /* Cast the spell */
    if borg_spell(st, TELEPORT_OTHER)
        || borg_activate_item(st, st.act.tele_other)
        || borg_aim_wand(st, st.item_val.sv_wand_teleport_away)
    {
        /* Use target */
        borg_keypress(st, key(b'5'));

        /* Set our shooting flag */
        st.fight_attack.successful_target = -1;

        /* Value */
        return p2;
    }

    0
}

/// Try to get rid of all of the monsters while I build my Sea of Runes
/// (`borg_defend_aux_banishment_morgoth`).
fn borg_defend_aux_banishment_morgoth(st: &mut BorgState) -> i32 {
    let fail_allowed = 50;

    /* Not if Morgoth is not on this level */
    if !st.borg.near.morgoth {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* (C has a disabled "#if 0" glyph count here) */

    let banish_evil = borg_spell_okay_fail(st, BANISH_EVIL, fail_allowed);
    let banishment = borg_spell_okay_fail(st, MASS_BANISHMENT, fail_allowed);

    /* Do I have the spell? (Banish Evil) */
    if !banishment && !banish_evil {
        return 0;
    }

    /* Nobody around so dont worry */
    if st.kills.cnt == 0 && simulating(st) {
        return 0;
    }

    /* Find "nearby" monsters */
    let mut count = 0;
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(kill.r_idx) {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Never try on non-evil guys if doing banish evil */
        if banish_evil && !banishment && !engine::race_flag_has(usize::from(kill.r_idx), RF_EVIL)
        {
            continue;
        }

        /* Check the distance  */
        if engine::distance(st.borg.c, kill.pos) > max_range() {
            continue;
        }

        /* Monster must be LOS */
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        if !borg_projectable(st, cy, cx, kill.pos.y, kill.pos.x) {
            continue;
        }

        /* Count the number of monsters too close double*/
        if engine::distance(st.borg.c, kill.pos) <= 7 {
            count += 1;
        }

        /* Count the number of monster on screen */
        count += 1;
    }

    /* No destinations */
    if count <= 7 && simulating(st) {
        return 0;
    }

    /* Return a good score to make him do it */
    if simulating(st) {
        return 1500;
    }

    borg_note(
        st,
        &format!("# Attempting to cast Banishment for depth 100.  {} monsters ", count),
    );

    /* Cast the spell */
    if borg_spell(st, MASS_BANISHMENT) || borg_spell(st, BANISH_EVIL) {
        /* Remove this race from the borg_kill */
        for i in 0..i32::from(st.kills.nxt) {
            /* Monster */
            let r_idx = st.kills.kills[i as usize].r_idx;

            /* dead monsters or "player ghosts" */
            if r_idx == 0 || is_ghost(r_idx) {
                continue;
            }

            /* Cant kill uniques like this */
            if engine::race_flag_has(usize::from(r_idx), RF_UNIQUE) {
                continue;
            }

            /* remove this monster */
            borg_delete_kill(st, i);
        }

        /* Value */
        return 1000;
    }

    0
}

/// Sometimes the borg will not fire on Morgoth as he approaches while
/// tunneling through rock (`borg_defend_aux_light_morgoth`).  The borg
/// still remembers and assumes that the rock is unknown grid.
fn borg_defend_aux_light_morgoth(st: &mut BorgState) -> i32 {
    let fail_allowed = 50;
    /* C leaves `best` uninitialised; it is only read after a simulation of
     * the same state found Morgoth (count > 0), which sets it */
    let mut best = loc { x: 0, y: 0 };
    let mut count = 0;

    /* Only if on level 100 and in a sea of runes */
    if !st.borg.morgoth_position {
        return 0;
    }

    /* Not if Morgoth is not on this level */
    if !st.borg.near.morgoth {
        return 0;
    }

    /* Cant when screwed */
    if screwed(st) {
        return 0;
    }

    /* Do I have the spell? */
    if !borg_spell_okay_fail(st, SPEAR_OF_LIGHT, fail_allowed)
        && !borg_spell_okay_fail(st, CLAIRVOYANCE, fail_allowed)
        && !borg_spell_okay_fail(st, FUME_OF_MORDOR, fail_allowed)
    {
        return 0;
    }

    /* Nobody around so dont worry */
    if st.kills.cnt == 0 && simulating(st) {
        return 0;
    }

    /* Find "nearby" monsters */
    for i in 1..i32::from(st.kills.nxt) {
        /* Monster */
        let kill = &st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* Skip non- Morgoth monsters */
        if u32::from(kill.r_idx) != st.borg.id.morgoth {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Acquire location */
        let x = kill.pos.x;
        let y = kill.pos.y;

        /* Check the distance  */
        if engine::distance(st.borg.c, kill.pos) > max_range() {
            continue;
        }
        if engine::distance(st.borg.c, kill.pos) <= 5 {
            continue;
        }

        /* We want at least one dark spot on the path */
        if !borg_projectable_dark(st, st.borg.c.y, st.borg.c.x, y, x) {
            continue;
        }

        /* Count Morgoth so I try the spell */
        count += 1;
        best.y = y;
        best.x = x;
    }

    /* No destinations */
    if count <= 0 && simulating(st) {
        return 0;
    }

    /* Return a good score to make him do it */
    if simulating(st) {
        return 500;
    }

    borg_note(st, &format!("# Attempting to Illuminate a Pathway to ({}, {})", best.y, best.x));

    /* Target Morgoth Grid */
    let _ = borg_target(st, best, false);

    /* Cast the spell */
    if borg_spell(st, SPEAR_OF_LIGHT)
        || borg_spell(st, CLAIRVOYANCE)
        || borg_spell(st, FUME_OF_MORDOR)
    {
        /* Select the target */
        borg_keypress(st, key(b'5'));

        /* Value */
        return 200;
    }

    0
}

/// Simulate/Apply the optimal result of using the given "type" of defense
/// (`borg_calculate_defense_effectiveness`). p1 is the current danger level
/// (passed in for effiency).
fn borg_calculate_defense_effectiveness(st: &mut BorgState, what: i32, p1: i32) -> i32 {
    /* Analyze */
    match what {
        BD_SPEED => borg_defend_aux_speed(st, p1),
        BD_PROT_FROM_EVIL => borg_defend_aux_prot_evil(st, p1),
        BD_GRIM_PURPOSE => borg_defend_aux_grim_purpose(st, p1),
        BD_RESIST_FECAP => borg_defend_aux_resist_fecap(st, p1),
        BD_RESIST_F => borg_defend_aux_resist_f(st, p1),
        BD_RESIST_C => borg_defend_aux_resist_c(st, p1),
        BD_RESIST_A => borg_defend_aux_resist_a(st, p1),
        BD_RESIST_E => borg_defend_aux_resist_e(st, p1),
        BD_RESIST_P => borg_defend_aux_resist_p(st, p1),
        BD_BLESS => borg_defend_aux_bless(st, p1),
        BD_HERO => borg_defend_aux_hero(st, p1),
        BD_BERSERK => borg_defend_aux_berserk(st, p1),
        BD_SMITE_EVIL => borg_defend_aux_smite_evil(st, p1),
        BD_REGEN => borg_defend_aux_regen(st, p1),
        BD_SHIELD => borg_defend_aux_shield(st, p1),
        BD_TELE_AWAY => borg_defend_aux_tele_away(st, p1),
        BD_GLYPH => borg_defend_aux_glyph(st, p1),
        BD_CREATE_DOOR => borg_defend_aux_create_door(st, p1),
        BD_MASS_GENOCIDE => borg_defend_aux_mass_genocide(st, p1),
        BD_GENOCIDE => borg_defend_aux_genocide(st, p1),
        BD_GENOCIDE_NASTIES => borg_defend_aux_genocide_nasties(st, p1),
        BD_EARTHQUAKE => borg_defend_aux_earthquake(st, p1),
        BD_TPORTLEVEL => borg_defend_aux_teleportlevel(st, p1),
        BD_DESTRUCTION => borg_defend_aux_destruction(st, p1),
        BD_BANISHMENT => borg_defend_aux_banishment(st, p1),
        BD_DETECT_INVISO => borg_defend_aux_inviso(st, p1),
        BD_LIGHT_BEAM => borg_defend_aux_lbeam(st, p1),
        BD_REST => borg_defend_aux_rest(st),
        BD_TELE_AWAY_MORGOTH => borg_defend_aux_tele_away_morgoth(st),
        BD_BANISHMENT_MORGOTH => borg_defend_aux_banishment_morgoth(st),
        BD_LIGHT_MORGOTH => borg_defend_aux_light_morgoth(st),
        _ => 0,
    }
}

/// Prepare to attack... this is setup for a battle (`borg_defend`).
pub fn borg_defend(st: &mut BorgState, p1: i32) -> bool {
    let mut b_n = 0;
    let mut b_g = -1;

    /* first try shifting the panels around so we can see better */
    if borg_defend_panel_shift(st) {
        return true;
    }

    /* Simulate */
    st.traits.simulate = true;

    /* if you have Resist All and it is about to drop, */
    /* refresh it (if you can) */
    let ratio = i32::from(st.traits.game_ratio);
    if st.borg.resistance != 0 && i32::from(st.borg.resistance) < ratio * 2 {
        /* check 'true' danger. This will make sure we do not */
        /* refresh our Resistance if no-one is around */
        st.traits.attacking = true;
        let p = danger_here(st, false); /* Note false for danger!! */
        st.traits.attacking = false;
        let (cy, cx) = (st.borg.c.y as usize, st.borg.c.x as usize);
        if (p > i32::from(st.danger.fear_region[cy / 11][cx / 11]) || st.borg.near.unique != 0)
            && borg_spell(st, RESISTANCE)
        {
            let ratio = st.traits.game_ratio;
            borg_note(
                st,
                &format!(
                    "# Refreshing Resistance.  borg.resistance={}, player->={}, (ratio={})",
                    st.borg.resistance,
                    engine::player_timed(TMD_OPP_ACID),
                    ratio
                ),
            );
            st.fight_defend.attempting_refresh_resist = true;
            st.borg.resistance = 25000;
            return true;
        }
    }

    /* Analyze the possible setup moves */
    for g in 0..BD_MAX {
        /* Simulate */
        let n = borg_calculate_defense_effectiveness(st, g, p1);

        /* Track "best" attack */
        if n <= b_n {
            continue;
        }

        /* Track best */
        b_g = g;
        b_n = n;
    }

    /* Nothing good */
    if b_n <= 0 {
        return false;
    }

    /* Note */
    borg_note(st, &format!("# Performing defense type {} with value {}", b_g, b_n));

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    let _ = borg_calculate_defense_effectiveness(st, b_g, p1);

    /* Success */
    true
}

//! Port of `borg-item-use.c`: use potions, scrolls, rods, wands, staffs
//! and rings.
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

// borg-reborn-rs (#242): named item_use policy; default operations unchanged.
use crate::params::{value as param, Param};
use crate::borg::BorgTime;
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note_bytes, ESCAPE};
use crate::borg_item::inven_wield;
use crate::borg_item_analyze::borg_obj_has_effect;
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_okay_fail};
use crate::engine;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::{
    HERBAL_CURING, RECHARGING, REMEMBRANCE, REMOVE_HUNGER, REVITALIZE, UNHOLY_REPRIEVE,
};
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_CDEPTH, BI_CURHP, BI_DEV, BI_FOOD, BI_FRACT, BI_ISAFRAID, BI_ISBLIND,
    BI_ISCONFUSED, BI_ISCUT, BI_ISFIXCON, BI_ISFIXDEX, BI_ISFIXEXP, BI_ISFIXINT, BI_ISFIXSTR,
    BI_ISFIXWIS, BI_ISFORGET, BI_ISHEAVYSTUN, BI_ISHUNGRY, BI_ISIMAGE, BI_ISPARALYZED, BI_ISPOISONED, BI_ISSTUN,
    BI_ISWEAK, BI_MAXHP, BI_RBLIND, BI_RCONF, BI_SPEED,
};
use crate::headers::list_effects_h::EF_NOURISH;
use crate::headers::list_tvals_h::{
    TV_DRAG_ARMOR, TV_FOOD, TV_MUSHROOM, TV_POTION, TV_RING, TV_ROD, TV_SCROLL, TV_STAFF, TV_WAND,
};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// borg-item-use.c function statics.
#[derive(Default)]
pub struct ItemUseState {
    /// `when_last_quaff` (static in `borg_quaff_crit`).
    pub when_last_quaff: BorgTime,
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

/// The item description up to its NUL (`%s` of `item->desc`).
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `borg_note(format("<pre>%s<post>", item->desc))`.
fn note_desc(st: &mut BorgState, pre: &[u8], item_idx: usize, post: &[u8]) {
    let mut note = pre.to_vec();
    note.extend_from_slice(&desc(&st.items.items[item_idx]));
    note.extend_from_slice(post);
    borg_note_bytes(st, &note);
}

/// `borg.goal.shop = borg.goal.ware = borg.goal.item = -1;`
fn clear_shop_goals(st: &mut BorgState) {
    st.borg.goal.item = -1;
    st.borg.goal.ware = st.borg.goal.item;
    st.borg.goal.shop = st.borg.goal.ware;
}

/// `skill = BI_DEV`, cut by confusion: the device skill used for the
/// fail checks.
fn device_skill(st: &BorgState) -> i32 {
    /* Base chance of success */
    let mut skill = st.borg.tr(BI_DEV);

    /* Confusion hurts skill */
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        skill = skill * 75 / 100;
    }
    skill
}

/// `100 * ((skill - lev) - (141 - 1)) / ((lev - skill) - (100 - 10))`.
/// The divisor is 0 when `lev - skill == 90` (C bug 20: a division by
/// zero, a crash in C and a panic here).
fn device_fail(skill: i32, lev: i32) -> i32 {
    100 * ((skill - lev) - (141 - 1)) / ((lev - skill) - (100 - 10))
}

/// Quaff a potion of cure critical wounds. This is a special case
/// for several reasons.
///   1) it is usually the only healing potion we have on us
///   2) we should try to conserve a couple for when we really need them
///   3) if we are burning through them fast we should probably teleport out
///      of the fight.
///   4) When it is the only/best way out of danger, drink away
pub fn borg_quaff_crit(st: &mut BorgState, no_check: bool) -> bool {
    if no_check {
        if borg_quaff_potion(st, st.item_val.sv_potion_cure_critical) {
            st.item_use.when_last_quaff = st.borg.time.now;
            return true;
        }
        return false;
    }

    /* Avoid drinking CCW twice in a row. */
    let last = st.item_use.when_last_quaff;
    let elapsed = st.borg.time.now - last;
    if last != 0 && elapsed <= param(Param::item_use_ccw_recent_window) && elapsed > 0 && (engine::randint0(100) + 1 < param(Param::item_use_ccw_recent_skip_chance)) {
        return false;
    }

    /* Save the last two for when we really need them */
    if st.borg.tr(BI_ACCW) < param(Param::item_use_ccw_min_stock) {
        return false;
    }

    if borg_quaff_potion(st, st.item_val.sv_potion_cure_critical) {
        st.item_use.when_last_quaff = st.borg.time.now;
        return true;
    }
    false
}

/// Attempt to quaff the given potion (by sval).
pub fn borg_quaff_potion(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that potion */
    let i = borg_slot(st, TV_POTION as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Quaffing ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "quaff", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'q'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals. Inventory changed so goals need to be redone. */
    clear_shop_goals(st);

    /* Success */
    true
}

/// borg-reborn-rs (#983, owner 2026-11-22: "test when safe"): the last
/// potion in the pack the Borg is not aware of (an unlearned flavour), or -1.
fn last_unaware_potion(st: &BorgState) -> i32 {
    let mut n = -1;
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = &st.items.items[i as usize];
        if item.iqty != 0 && u32::from(item.tval) == TV_POTION && !item.aware {
            n = i;
        }
    }
    n
}

/// borg-reborn-rs (#983): whether a safe moment to learn a potion flavour
/// by drinking it: in the dungeon, no deeper than `max_depth`, no danger here
/// over the turns a bad potion can cost (`danger` is summed over
/// `item_test_unknown_danger_turns`: Sleep paralyses for 5+1d5 turns,
/// object.txt), at least `min_hp_percent` of max HP, no harmful status
/// (`bad_status`, paralysis included: a Borg asleep from the last test does
/// not test the next), food in the pack (Salt Water empties the stomach).
/// The harmful kinds a Borg can find: Sleep, Blindness, Confusion, Poison,
/// Slowness (DL0-10), Salt Water (DL3-15), Surprise (DL10-15), the stat swaps
/// Brawn to Toughness (DL20-40, one stat lost for another); none kills by
/// itself, but a paralysis or Poison is several turns without action or
/// regeneration. Dragon Breath (DL40+) asks for an aim: `max_depth` keeps the
/// test above it.
pub(crate) fn unknown_potion_test_safe(depth: i32, max_depth: i32, cur_hp: i32, max_hp: i32,
    min_hp_percent: i32, danger: i32, bad_status: bool, food: i32) -> bool {
    depth > 0 && depth <= max_depth && danger == 0 && !bad_status && food > 0
        && max_hp > 0 && cur_hp * 100 >= max_hp * min_hp_percent
}

/// borg-reborn-rs (#983, owner: "test when safe"): with
/// `item_test_unknown_potions`, drink one unknown potion at a safe moment,
/// so a found Cure Light Wounds is known before it is needed (seed
/// 17626119 died confused with 4 unlearned CLW). #470 still forbids every
/// other unidentified use: scrolls (Deep Descent), mushrooms, devices, and
/// any unknown item in an emergency.
pub fn borg_quaff_unknown_when_safe(st: &mut BorgState) -> bool {
    if param(Param::item_test_unknown_potions) == 0 {
        return false;
    }
    let n = last_unaware_potion(st);
    if n < 0 {
        return false;
    }
    let bad_status = [BI_ISCONFUSED, BI_ISBLIND, BI_ISAFRAID, BI_ISPOISONED, BI_ISCUT, BI_ISSTUN,
        BI_ISHEAVYSTUN, BI_ISIMAGE, BI_ISPARALYZED, BI_ISHUNGRY, BI_ISWEAK, BI_ISFORGET]
        .into_iter()
        .any(|f| st.borg.tr(f) != 0);
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let turns = param(Param::item_test_unknown_danger_turns).max(1);
    let danger = crate::borg_danger::borg_danger(st, y, x, turns, true, false);
    if !unknown_potion_test_safe(st.borg.tr(BI_CDEPTH), param(Param::item_test_unknown_max_depth),
        st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), param(Param::item_test_unknown_min_hp_percent),
        danger, bad_status, st.borg.tr(BI_FOOD))
    {
        return false;
    }
    // borg-reborn-rs (#1030, Codex on #1026): `borg_danger`'s `c` widens the
    // melee reach only; its ranged part counts a monster that can project to
    // the Borg now or after one move. A caster two moves behind a corner
    // counts 0, yet it can step out and cast during a Sleep. So no test while
    // a known, awake monster with a ranged attack could reach a firing grid
    // within the window: its moves in `turns` player turns plus the engine's
    // `max_range` (a shot needs line of sight within it).
    let player_energy = i32::from(engine::extract_energy(st.borg.tr(BI_SPEED).clamp(0, 199) as usize)).max(1);
    let moves_of = |speed: u8| {
        let e = i32::from(engine::extract_energy(usize::from(speed).min(199)));
        (turns * e + player_energy - 1) / player_energy
    };
    let nxt = st.kills.nxt.max(0) as usize;
    if let Some(i) = ranged_threat_in_window(&st.kills.kills[..nxt.min(st.kills.kills.len())],
        loc { x, y }, i32::from(engine::z_info().max_range), moves_of)
    {
        let name = String::from_utf8_lossy(engine::race_name(usize::from(st.kills.kills[i].r_idx))).into_owned();
        crate::borg_io::borg_note(st, &format!("# Not testing an unknown potion: {name} could shoot or cast within {turns} turns (#1030)"));
        return false;
    }
    note_desc(st, b"# Testing unknown potion (#983) ", n as usize, b".");
    crate::borg_object_log::log_use(st, "quaff", &st.items.items[n as usize]);
    borg_keypress(st, key(b'q'));
    borg_keypress(st, letter(n));
    clear_shop_goals(st);
    st.borg.trying_unknown = true;
    true
}

/// borg-reborn-rs (#1030): the index of a known, awake monster with a
/// ranged attack (`ranged_attack`, the test `borg_danger_spell` makes) that
/// could reach a grid in range of `c` within the window: its distance is at
/// most its moves in the window (`moves_of(speed)`) plus `max_range`, whether
/// or not it can project to `c` now (it may be behind a corner). A record kept
/// after the monster is lost counts too: the veto errs on the safe side.
pub(crate) fn ranged_threat_in_window(kills: &[crate::borg_flow_kill::BorgKill], c: loc, max_range: i32,
    moves_of: impl Fn(u8) -> i32) -> Option<usize> {
    kills.iter().position(|kill| {
        kill.r_idx != 0 && kill.awake && kill.ranged_attack != 0
            && engine::distance(c, kill.pos) <= moves_of(kill.speed) + max_range
    })
}

#[cfg(test)]
mod ranged_threat_tests {
    use super::ranged_threat_in_window as threat;
    use crate::borg_flow_kill::BorgKill;
    use crate::headers::z_type_h::loc;

    fn caster(x: i32, y: i32) -> BorgKill {
        BorgKill { r_idx: 40, awake: true, ranged_attack: 1, speed: 110, pos: loc { x, y }, ..BorgKill::BLANK }
    }

    #[test]
    fn a_caster_behind_a_corner_vetoes_the_test() {
        let borg = loc { x: 10, y: 10 };
        // Codex's case: two moves away, out of sight; with 10 moves and range
        // 20 it can step out and cast during a Sleep.
        assert_eq!(threat(&[caster(12, 12)], borg, 20, |_| 10), Some(0));
        assert_eq!(threat(&[caster(40, 10)], borg, 20, |_| 10), Some(0), "30 grids: 10 moves + range 20");
        assert_eq!(threat(&[caster(41, 10)], borg, 20, |_| 10), None, "31 grids: out of reach");
        assert_eq!(threat(&[BorgKill { awake: false, ..caster(12, 12) }], borg, 20, |_| 10), None, "asleep");
        assert_eq!(threat(&[BorgKill { ranged_attack: 0, ..caster(12, 12) }], borg, 20, |_| 10), None, "melee only: borg_danger's c covers it");
        assert_eq!(threat(&[BorgKill { r_idx: 0, ..caster(12, 12) }], borg, 20, |_| 10), None, "empty record");
        assert_eq!(threat(&[caster(50, 10)], borg, 20, |s| if s > 110 { 30 } else { 10 }), None);
        assert_eq!(threat(&[BorgKill { speed: 120, ..caster(50, 10) }], borg, 20, |s| if s > 110 { 30 } else { 10 }), Some(0), "a fast caster moves further");
    }
}

#[cfg(test)]
mod unknown_potion_tests {
    use super::unknown_potion_test_safe as safe;

    #[test]
    fn only_a_safe_moment_in_the_dungeon() {
        assert!(safe(1, 30, 21, 28, 50, 0, false, 3), "seed 17626119 at turn 7295: HP 21/28, danger 0");
        assert!(!safe(0, 30, 28, 28, 50, 0, false, 3), "not in town");
        assert!(!safe(31, 30, 28, 28, 50, 0, false, 3), "not below the depth cap (Dragon Breath)");
        assert!(safe(30, 30, 28, 28, 50, 0, false, 3), "at the depth cap");
        assert!(!safe(1, 30, 28, 28, 50, 5, false, 3), "a danger within the turns");
        assert!(!safe(1, 30, 13, 28, 50, 0, false, 3), "below the HP floor");
        assert!(safe(1, 30, 14, 28, 50, 0, false, 3), "at the floor");
        assert!(!safe(1, 30, 28, 28, 50, 0, true, 3), "a harmful status (paralysis included)");
        assert!(!safe(1, 30, 28, 28, 50, 0, false, 0), "no food after a Salt Water");
        assert!(!safe(1, 30, 0, 0, 50, 0, false, 3), "no max HP");
    }
}

/// Attempt to read the given scroll (by sval).
pub fn borg_read_scroll(st: &mut BorgState, sval: i32) -> bool {
    /* Dark */
    if engine::no_light() {
        return false;
    }

    /* Blind or Confused or Amnesia*/
    if st.borg.tr(BI_ISBLIND) != 0
        || st.borg.tr(BI_ISCONFUSED) != 0
        || st.borg.tr(BI_ISFORGET) != 0
    {
        return false;
    }

    /* borg-reborn-rs (#829): not a teleport under a teleport curse */
    if crate::borg_escape::teleport_scroll_blocked(st, sval) {
        return false;
    }

    /* Look for that scroll */
    let i = borg_slot(st, TV_SCROLL as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Reading ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "read", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, ESCAPE);
    borg_keypress(st, ESCAPE);
    borg_keypress(st, key(b'r'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals */
    clear_shop_goals(st);

    /* Success */
    true
}

/// Attempt to eat the given food or mushroom.
pub fn borg_eat(st: &mut BorgState, tval: i32, sval: i32) -> bool {
    eat_in_current_shape(st, tval, sval, engine::player_is_shapechanged())
}

#[inline]
fn eat_in_current_shape(st: &mut BorgState, tval: i32, sval: i32, shapechanged: bool) -> bool {
    // cmd-obj.c:do_cmd_eat_food has no change-back prompt, and
    // cmd-core.c:cmd_get_item hides the pack while shapechanged. As in
    // borg_deinscribe, stop before an item key can escape into the command
    // prompt. The junk callers can then fall through to dropping the item.
    if shapechanged {
        return false;
    }

    /* Look for that food */
    let i = borg_slot(st, tval, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Eating ", i as usize, b".");

    /* Perform the action */
    borg_keypress(st, key(b'E'));
    borg_keypress(st, letter(i));

    /* Clear "shop" goals */
    clear_shop_goals(st);

    /* Success */
    true
}

/// Whether a Remove Hunger scroll may be tried ahead of food: only under the
/// town plan, and only when the Borg can read (`borg_read_scroll` also
/// checks the light). A Borg that cannot read goes on to eat.
///
/// borg-reborn-rs (#451).
pub(crate) fn may_read_hunger_scroll_first(capability_mode: bool, blind: bool, confused: bool, forgetful: bool) -> bool {
    capability_mode && !blind && !confused && !forgetful
}

/// Relieve hunger before any food is eaten: the hunger spells, then a Remove
/// Hunger scroll (50% of a full stomach, `NOURISH:INC_TO` 50 in object.txt:
/// more than a ration's `INC_BY` 30 when read hungry, at 5 against 8 weight).
/// Only under the town plan, where the planner carries the scroll.
///
/// borg-reborn-rs (#451).
pub fn borg_satisfy_hunger_before_food(st: &mut BorgState) -> bool {
    let capability_mode = st.cfg(crate::borg_headers::borg_h::BORG_TOWN_PLAN) == 1;
    if !may_read_hunger_scroll_first(
        capability_mode,
        st.borg.tr(BI_ISBLIND) != 0,
        st.borg.tr(BI_ISCONFUSED) != 0,
        st.borg.tr(BI_ISFORGET) != 0,
    ) {
        return false;
    }
    borg_spell(st, REMOVE_HUNGER)
        || borg_spell(st, HERBAL_CURING)
        || borg_read_scroll(st, st.item_val.sv_scroll_satisfy_hunger)
}

/// Prevent starvation by any means possible.
pub fn borg_eat_food_any(st: &mut BorgState) -> bool {
    let pack_size = usize::from(engine::z_info().pack_size);

    /* Scan the inventory for "normal" food */
    for i in 0..pack_size {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip unknown food */
        if !item.aware {
            continue;
        }

        /* Skip non-food */
        if u32::from(item.tval) != TV_FOOD {
            continue;
        }

        /* Eat something of that type */
        let (tval, sval) = (i32::from(item.tval), i32::from(item.sval));
        if borg_eat(st, tval, sval) {
            return true;
        }
    }

    /* Scan the inventory for "okay" food */
    for i in 0..pack_size {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip unknown food */
        if !item.aware {
            continue;
        }

        /* Skip non-food */
        if u32::from(item.tval) != TV_FOOD && u32::from(item.tval) != TV_MUSHROOM {
            continue;
        }

        /* Skip non-food */
        if !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 0)
            && !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 2)
            && !borg_obj_has_effect(item.kind, EF_NOURISH as i32, 3)
        {
            continue;
        }

        /* Eat something of that type */
        let (tval, sval) = (i32::from(item.tval), i32::from(item.sval));
        if borg_eat(st, tval, sval) {
            return true;
        }
    }

    /*
     * Try potions that can provide nutrition.  First try ones that are
     * pure nutrition without additional effects.
     */
    if borg_quaff_potion(st, st.item_val.sv_potion_slime_mold) {
        return true;
    }
    /*
     * Then try those that, besides the nourishment, only have negative
     * effects.  But only try if there's protection against the negative effect.
     */
    if (st.borg.tr(BI_FRACT) != 0
        && (borg_quaff_potion(st, st.item_val.sv_potion_sleep)
            || borg_quaff_potion(st, st.item_val.sv_potion_slowness)))
        || (st.borg.tr(BI_RBLIND) != 0 && borg_quaff_potion(st, st.item_val.sv_potion_blindness))
        || (st.borg.tr(BI_RCONF) != 0 && borg_quaff_potion(st, st.item_val.sv_potion_confusion))
    {
        return true;
    }
    /* Consume in order, when hurting */
    if (st.borg.tr(BI_CURHP) < param(Param::item_use_food_heal_critical_hp) || st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP))
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing))
    {
        return true;
    }

    /* Nothing */
    false
}

/// Checks rod (by sval) and make a fail check on it.
pub fn borg_equips_rod(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_ROD as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage (at least 1/2 chance of success. */
    if fail > param(Param::item_use_rod_equip_max_failure) {
        return false;
    }

    /* Yep we got one */
    true
}

/// Attempt to zap the given (charged) rod (by sval).
pub fn borg_zap_rod(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that rod */
    let i = borg_slot(st, TV_ROD as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* Still charging */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage */
    if sval != st.item_val.sv_rod_recall && fail > param(Param::item_use_rod_use_max_failure) {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Zapping ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "zap", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'z'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Attempt to use the given (charged) staff (by sval).
pub fn borg_use_staff(st: &mut BorgState, sval: i32) -> bool {
    /* borg-reborn-rs (#829): not a teleport under a teleport curse */
    if crate::borg_escape::teleport_staff_blocked(st, sval) {
        return false;
    }

    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Using ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "use", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'u'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Attempt to use the given (charged) staff (by sval) and make a fail
/// check on it.
pub fn borg_use_staff_fail(st: &mut BorgState, sval: i32) -> bool {
    /* borg-reborn-rs (#829): not a teleport under a teleport curse */
    if crate::borg_escape::teleport_staff_blocked(st, sval) {
        return false;
    }

    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > param(Param::item_use_staff_use_max_failure) {
        if sval != st.item_val.sv_staff_teleportation {
            return false;
        }

        /* We need to give some "desperation attempt to teleport staff" */
        if st.borg.tr(BI_ISCONFUSED) == 0 && st.borg.tr(BI_ISBLIND) == 0 {
            /* We really have no chance, return false, attempt the scroll
             * (C tests `fail > 500` again here; it is true) */
            return false;
        }
        /* We might have a slight chance, or we cannot not read */
    }

    /* Log the message */
    note_desc(st, b"# Using ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "use", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'u'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// Checks staff (by sval) and make a fail check on it.
pub fn borg_equips_staff_fail(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that staff */
    let i = borg_slot(st, TV_STAFF as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Extract the item level */
    let lev = i32::from(st.items.items[i as usize].level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let fail = device_fail(skill, lev);

    /* If its a Destruction, we only use it in emergencies, attempt it */
    if sval == st.item_val.sv_staff_destruction {
        return true;
    }

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > param(Param::item_use_staff_equip_max_failure) {
        /* No real chance of success on other types of staffs */
        if sval != st.item_val.sv_staff_teleportation {
            return false;
        }

        /* We need to give some "desperation attempt to teleport staff" */
        if sval == st.item_val.sv_staff_teleportation && st.borg.tr(BI_ISCONFUSED) == 0 {
            /* We really have no chance, return false, attempt the scroll */
            if fail < param(Param::item_use_staff_teleport_max_failure) {
                return false;
            }
        }

        /* We might have a slight chance (or its a Destruction), continue on */
    }

    /* Yep we got one */
    true
}

/// Attempt to aim the given (charged) wand (by sval).
pub fn borg_aim_wand(st: &mut BorgState, sval: i32) -> bool {
    /* Look for that wand */
    let i = borg_slot(st, TV_WAND as i32, sval);

    /* None available */
    if i < 0 {
        return false;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Aiming ", i as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "aim", &st.items.items[i as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'a'));
    borg_keypress(st, letter(i));

    /* Success */
    true
}

/// `INVEN_RIGHT`
fn inven_right() -> i32 {
    inven_wield() + 2
}

/// `INVEN_LEFT`
fn inven_left() -> i32 {
    inven_wield() + 3
}

/// `INVEN_BODY`
fn inven_body() -> i32 {
    inven_wield() + 6
}

/// `INVEN_TOTAL`: `INVEN_WIELD + 12` (the c2rust constant).
fn inven_total() -> i32 {
    inven_wield() + 12
}

/// Check and see if borg is wielding a ring and if he will pass a fail
/// check.
pub fn borg_equips_ring(st: &mut BorgState, ring_sval: i32) -> bool {
    for i in inven_right()..=inven_left() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 || !item.aware {
            continue;
        }

        /* Skip incorrect armors */
        if u32::from(item.tval) != TV_RING {
            continue;
        }
        if i32::from(item.sval) != ring_sval {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /*  Make Sure is IDed */
        if !item.ident {
            continue;
        }

        /* check on fail rate */

        /* Extract the item level */
        let lev = i32::from(item.level);

        /* Base chance of success, confusion hurts skill */
        let skill = device_skill(st);

        /* High level objects are harder */
        let fail = device_fail(skill, lev);

        /* Roll for usage, but if its a Teleport be generous. */
        if fail > param(Param::item_use_ring_equip_max_failure) {
            continue;
        }

        /* Success */
        return true;
    }

    false
}

/// Attempt to use the given ring.
pub fn borg_activate_ring(st: &mut BorgState, ring_sval: i32) -> bool {
    /* Check the equipment */
    for i in inven_right()..=inven_left() {
        let item = &st.items.items[i as usize];

        /*  Make Sure item is IDed */
        if !item.aware {
            continue;
        }

        /* Skip incorrect mails */
        if u32::from(item.tval) != TV_RING {
            continue;
        }
        if i32::from(item.sval) != ring_sval {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /*  Make Sure item is IDed */
        if !item.ident {
            continue;
        }

        /* Log the message */
        note_desc(st, b"# Activating ring ", i as usize, b".");

        /* The session object log: what the Borg uses. */
        crate::borg_object_log::log_use(st, "activate", &st.items.items[i as usize]);

        /* Perform the action */
        borg_keypress(st, key(b'A'));
        borg_keypress(st, letter(i - inven_wield()));

        /* Success */
        return true;
    }

    false
}

/// The checks shared by `borg_equips_dragon` and `borg_activate_dragon`:
/// is the body armour an aware, identified, charged dragon armour of
/// this sval?
fn dragon_ready(item: &borg_item, drag_sval: i32) -> bool {
    if item.iqty == 0 || !item.aware {
        return false;
    }

    /* Skip incorrect armours */
    if u32::from(item.tval) != TV_DRAG_ARMOR {
        return false;
    }
    if i32::from(item.sval) != drag_sval {
        return false;
    }

    /* Check charge */
    if item.timeout != 0 {
        return false;
    }

    /*  Make Sure Mail is IDed */
    if !item.ident {
        return false;
    }
    true
}

/// Check and see if borg is wielding a dragon armor and if he will pass a
/// fail check.
pub fn borg_equips_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
    /* Check the equipment */
    let item = &st.items.items[inven_body() as usize];

    if !dragon_ready(item, drag_sval) {
        return false;
    }

    /* check on fail rate
     * The fail check is automatic for dragon armor.  It is an attack
     * item.  He should not sit around failing 5 or 6 times in a row.
     * he should attempt to activate it, and if he is likely to fail, then
     * eh should look at a different attack option.  We are assuming
     * that the fail rate is about 50%.  So He may still try to activate it
     * and fail.  But he will not even try if he has negative chance or
     * less than twice the USE_DEVICE variable
     */
    /* Extract the item level */
    let lev = i32::from(item.level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    let numerator = (skill - lev) - (141 - 1);
    let mut denominator = (lev - skill) - (100 - 10);

    /* Make sure that we don't divide by zero */
    if denominator == 0 {
        denominator = if numerator > 0 { 1 } else { -1 };
    }

    let fail = (100 * numerator) / denominator;

    /* Roll for usage, but if its a Teleport be generous. */
    if fail > param(Param::item_use_dragon_equip_max_failure) {
        return false;
    }

    /* Success */
    true
}

/// Attempt to use the given dragon armour.
pub fn borg_activate_dragon(st: &mut BorgState, drag_sval: i32) -> bool {
    /* Check the equipment */
    let body = inven_body();
    if !dragon_ready(&st.items.items[body as usize], drag_sval) {
        return false;
    }

    /* Log the message */
    note_desc(st, b"# Activating dragon scale ", body as usize, b".");

    /* The session object log: what the Borg uses. */
    crate::borg_object_log::log_use(st, "activate", &st.items.items[body as usize]);

    /* Perform the action */
    borg_keypress(st, key(b'A'));
    borg_keypress(st, letter(body - inven_wield()));

    /* Success */
    true
}

/// Attempt to use the given artifact.
pub fn borg_activate_item(st: &mut BorgState, activation: i32) -> bool {
    /* borg-reborn-rs (#829): not a teleport under a teleport curse */
    if crate::borg_escape::teleport_activation_blocked(st, activation) {
        return false;
    }

    /* a quick check of the array */
    if st.borg.activation[activation as usize] == 0 {
        return false;
    }

    /* Check the equipment */
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 {
            continue;
        }

        /* Skip wrong activation*/
        if item.activ_idx != activation {
            continue;
        }

        /* Check charge */
        if item.timeout != 0 {
            continue;
        }

        /* Log the message */
        note_desc(st, b"# Activating item ", i as usize, b".");

        /* The session object log: what the Borg uses. */
        crate::borg_object_log::log_use(st, "activate", &st.items.items[i as usize]);

        /* Perform the action */
        borg_keypress(st, key(b'A'));
        borg_keypress(st, letter(i - inven_wield()));

        /* Success */
        return true;
    }

    /* Oops */
    false
}

/// Check and see if borg is wielding an item with this activation.
pub fn borg_equips_item(st: &mut BorgState, activation: i32, check_charge: bool) -> bool {
    /* a quick check of the array */
    if st.borg.activation[activation as usize] == 0 {
        return false;
    } else if !check_charge {
        return true;
    }

    /* Check the equipment */
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];

        if item.iqty == 0 {
            continue;
        }

        /* Skip wrong activation */
        if item.activ_idx != activation {
            continue;
        }

        /* Check charge.  */
        if check_charge && item.timeout >= 1 {
            continue;
        }

        /* Success */
        return true;
    }

    /* I do not have it or it is not charged */
    false
}

/// Return the relative chance for failure to activate an item.
/// The lower the number, the better the chance of success
/// 200 is 80% of success
/// 600 is 40% chance of success
pub fn borg_activate_failure(st: &mut BorgState, tval: i32, sval: i32) -> i32 {
    /* Look for that item */
    let i = borg_slot(st, tval, sval);

    /* None available */
    if i < 0 {
        return 100;
    }

    let item = &st.items.items[i as usize];

    /* No charges */
    if item.pval == 0 {
        return 100;
    }

    /* no known activation */
    if item.activ_idx == 0 {
        return 100;
    }

    /* Extract the item level */
    let lev = i32::from(item.level);

    /* Base chance of success, confusion hurts skill */
    let skill = device_skill(st);

    /* High level objects are harder */
    device_fail(skill, lev)
}

/// Whether the known stack is the single-purpose RESTORE_EXP potion.
pub(crate) fn is_restore_life(item: &crate::borg_headers::borg_item_h::borg_item, sval: i32) -> bool {
    item.iqty > 0 && item.aware && u32::from(item.tval) == TV_POTION && i32::from(item.sval) == sval
}

/// Use things in a useful, but non-essential, manner.
pub fn borg_use_things(st: &mut BorgState) -> bool {
    /* Quaff experience restoration potion */
    if st.borg.tr(BI_ISFIXEXP) != 0
        && (borg_spell(st, REVITALIZE)
            || borg_spell(st, REMEMBRANCE)
            || (st.borg.tr(BI_CURHP) > param(Param::item_use_unholy_min_hp) && borg_spell(st, UNHOLY_REPRIEVE))
            || borg_activate_item(st, st.act.restore_exp)
            || borg_activate_item(st, st.act.restore_st_lev)
            || borg_activate_item(st, st.act.restore_life)
            || borg_quaff_potion(st, st.item_val.sv_potion_restore_life))
    {
        return true;
    }

    /* just drink the stat gains, at this dlevel we won't need cash */
    if borg_quaff_potion(st, st.item_val.sv_potion_inc_str)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_int)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_wis)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_dex)
        || borg_quaff_potion(st, st.item_val.sv_potion_inc_con)
    {
        return true;
    }

    let mush = TV_MUSHROOM as i32;

    /* Quaff potions of "restore" stat if needed */
    if (st.borg.tr(BI_ISFIXSTR) != 0
        && (borg_quaff_potion(st, st.item_val.sv_potion_inc_str)
            || borg_eat(st, mush, st.item_val.sv_mush_purging)
            || borg_activate_item(st, st.act.shroom_purging)
            || borg_activate_item(st, st.act.restore_str)
            || borg_activate_item(st, st.act.restore_all)
            || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXINT) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_int)
                || borg_activate_item(st, st.act.restore_int)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXWIS) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_wis)
                || borg_activate_item(st, st.act.restore_wis)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXDEX) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_dex)
                || borg_activate_item(st, st.act.restore_dex)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
        || (st.borg.tr(BI_ISFIXCON) != 0
            && (borg_quaff_potion(st, st.item_val.sv_potion_inc_con)
                || borg_activate_item(st, st.act.restore_con)
                || borg_activate_item(st, st.act.restore_all)
                || borg_eat(st, mush, st.item_val.sv_mush_purging)
                || borg_activate_item(st, st.act.shroom_purging)
                || borg_eat(st, mush, st.item_val.sv_mush_restoring)))
    {
        return true;
    }

    /* Use some items right away */
    for i in 0..usize::from(engine::z_info().pack_size) {
        let item = &st.items.items[i];

        /* Skip empty items */
        if item.iqty == 0 || !item.aware {
            continue;
        }

        let sval = i32::from(item.sval);

        /* Process "force" items */
        match u32::from(item.tval) {
            TV_POTION => {
                /* Check the potion */
                if sval == st.item_val.sv_potion_enlightenment {
                    /* Never quaff these in town */
                    if st.borg.tr(BI_CDEPTH) != 0 && borg_quaff_potion(st, sval) {
                        return true;
                    }
                } else if sval == st.item_val.sv_potion_inc_all {
                    /* Try quaffing the potion */
                    if borg_quaff_potion(st, sval) {
                        return true;
                    }
                }
            }
            TV_SCROLL => {
                /* Check Blind/Confused */
                if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
                    continue;
                }

                /* XXX XXX XXX Dark */

                /* Check the scroll */
                if sval == st.item_val.sv_scroll_mapping
                    || sval == st.item_val.sv_scroll_acquirement
                    || sval == st.item_val.sv_scroll_star_acquirement
                {
                    /* Never read these in town */
                    if st.borg.tr(BI_CDEPTH) == 0 {
                        continue;
                    }

                    /* Try reading the scroll */
                    if borg_read_scroll(st, sval) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    st.borg.tr(BI_ISHUNGRY) != 0 && borg_eat_when_hungry(st)
}

/// Satisfy hunger using known supplies without using unrelated items.
pub(crate) fn borg_eat_when_hungry(st: &mut BorgState) -> bool {
    let food = TV_FOOD as i32;
    /* Attempt to satisfy hunger */
    // borg-reborn-rs (#451): a Remove Hunger scroll goes before any food
    borg_satisfy_hunger_before_food(st)
        || borg_spell(st, REMOVE_HUNGER)
        || borg_spell(st, HERBAL_CURING)
        || borg_quaff_potion(st, st.item_val.sv_potion_slime_mold)
        || borg_eat(st, food, st.item_val.sv_food_slime_mold)
        || borg_eat(st, food, st.item_val.sv_food_slice)
        || borg_eat(st, food, st.item_val.sv_food_apple)
        || borg_eat(st, food, st.item_val.sv_food_pint)
        || borg_eat(st, food, st.item_val.sv_food_handful)
        || borg_eat(st, food, st.item_val.sv_food_honey_cake)
        || borg_eat(st, food, st.item_val.sv_food_ration)
        || borg_eat(st, food, st.item_val.sv_food_waybread)
        || borg_eat(st, food, st.item_val.sv_food_draught)
        || borg_activate_item(st, st.act.food_waybread)
}

/// Recharge things.
///
/// XXX XXX XXX Prioritize available items
pub fn borg_recharging(st: &mut BorgState) -> bool {
    /* Forbid blind/confused */
    if st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0 {
        return false;
    }

    /* XXX XXX XXX Dark */

    /* Look for an item to recharge */
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip non-identified items */
        if !item.ident || !item.aware {
            continue;
        }

        /* assume we can't charge it. */
        let mut charge = false;

        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);
        let pval = i32::from(item.pval);

        /* Wands with no charges can be charged */
        if tval == TV_WAND && pval <= param(Param::item_use_recharge_wand_max_charges) {
            charge = true;
        }

        /* recharge staves sometimes */
        if tval == TV_STAFF {
            /* Allow staves to be recharged at 2 charges if
             * the borg has the big recharge spell. And its not a *Dest*
             */
            if pval < param(Param::item_use_recharge_low_charges)
                && borg_spell_okay_fail(st, RECHARGING, param(Param::item_use_recharge_fail))
                && sval < st.item_val.sv_staff_power
            {
                charge = true;
            }

            /* recharge any staff at 0 charges */
            if pval <= param(Param::item_use_recharge_staff_max_charges) {
                charge = true;
            }

            /* Staves of teleport get recharged at 2 charges in town */
            if sval == st.item_val.sv_staff_teleportation && pval < param(Param::item_use_teleport_staff_min_charges) && st.borg.tr(BI_CDEPTH) == 0
            {
                charge = true;
            }

            /* They stack.  If quantity is 4 and pval is 1, then there are 4
             * charges. */
            if i32::from(item.iqty) + pval >= param(Param::item_use_recharge_min_total_stock) && pval >= param(Param::item_use_recharge_stacked_min_charges) {
                charge = false;
            }
        }

        /* get the next item if we are not to charge this one */
        if !charge {
            continue;
        }

        /* Attempt to recharge */
        if borg_read_scroll(st, st.item_val.sv_scroll_recharging)
            || borg_spell_fail(st, RECHARGING, param(Param::item_use_recharge_fail))
            || borg_activate_item(st, st.act.recharge)
        {
            /* Message (the item is unchanged by the calls above) */
            let mut note = b"Recharging ".to_vec();
            note.extend_from_slice(&desc(&st.items.items[i as usize]));
            note.extend_from_slice(
                format!(" with current charge of {}", st.items.items[i as usize].pval).as_bytes(),
            );
            borg_note_bytes(st, &note);

            /* Recharge the item */
            borg_keypress(st, letter(i));

            /* Success */
            return true;
        } else {
            /* if we fail once, no need to try again. */
            break;
        }
    }

    /* Nope */
    false
}

// borg-reborn-rs (#451)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_item::blank_item;

    #[test]
    fn eating_from_a_full_pack_in_fox_form_queues_nothing() {
        let mut st = BorgState::default();
        let mut item = blank_item();
        item.iqty = 1;
        st.items.items = vec![item; 23];

        let food = &mut st.items.items[2];
        food.iqty = 6;
        food.tval = TV_FOOD as u8;
        food.sval = 1;
        food.aware = true;

        let juice = &mut st.items.items[22];
        juice.iqty = 4;
        juice.tval = TV_POTION as u8;
        juice.sval = 1;
        juice.aware = true;

        assert!(!eat_in_current_shape(&mut st, TV_FOOD as i32, 1, true));
        assert!(st.io.queued_keys().is_empty());

        st.items.items[2].tval = TV_MUSHROOM as u8;
        assert!(!eat_in_current_shape(&mut st, TV_MUSHROOM as i32, 1, true));
        assert!(st.io.queued_keys().is_empty());
    }

    #[test]
    fn the_hunger_scroll_goes_before_food_only_under_the_plan_and_when_it_can_be_read() {
        assert!(may_read_hunger_scroll_first(true, false, false, false));
        // without the town plan the C Borg's order stands (spells, then food)
        assert!(!may_read_hunger_scroll_first(false, false, false, false));
        // blind, confused or forgetful: no scroll, the Borg goes on to eat
        assert!(!may_read_hunger_scroll_first(true, true, false, false));
        assert!(!may_read_hunger_scroll_first(true, false, true, false));
        assert!(!may_read_hunger_scroll_first(true, false, false, true));
    }
}

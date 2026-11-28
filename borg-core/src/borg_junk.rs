//! Port of `borg-junk.c`: handle junk items or clear the inventory to make
//! space.
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
//! The "what if" evaluations change `st.items.items` in place, call
//! `borg_notice` / `borg_power`, and restore the slots exactly as C does
//! (`memcpy` from `safe_items` is a struct copy; `borg_drop_slow` only
//! undoes its `iqty--`). C reads the item through a pointer after the
//! restore, so the item is read again from `st.items.items` there. The
//! swap slots (`weapon_swap`, `armour_swap`) are re-read at each use:
//! `borg_notice(true)` recomputes them.

use crate::borg_danger::borg_danger;
use crate::borg_inventory::{borg_first_empty_inventory_slot, borg_item_worth_id};
use crate::borg_io::{borg_keypress, borg_keypresses, borg_note, borg_note_bytes, KC_ENTER};
use crate::borg_item::{borg_item_weight, inven_feet, inven_wield, quiver_end, quiver_start};
use crate::borg_item_analyze::borg_ego_has_random_power;
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_item_use::{borg_eat, borg_quaff_potion, borg_read_scroll, is_restore_life};
use crate::borg_power::borg_power;
use crate::borg_town_capability::restore_life_reserve;
use crate::engine;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_WORSHIPS_GOLD};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_AMMO_POWER, BI_AMMO_TVAL, BI_ASTFDEST, BI_ATELEPORTLVL, BI_ATPORTOTHER, BI_CARRY, BI_CCON, BI_CDEPTH,
    BI_CDEX, BI_CINT, BI_CLASS, BI_CLEVEL, BI_CSTR, BI_CURHP, BI_CWIS, BI_FOOD, BI_GOLD,
    BI_GOOD_S_CHG, BI_ISFIXEXP, BI_ISFULL, BI_ISHUNGRY, BI_ISWEAK, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH,
    BI_MAXHP, BI_MAXSP, BI_QUIVER_SLOTS, BI_SPEED, BI_WEIGHT,
};
use crate::headers::list_options_h::OPT_birth_randarts;
use crate::headers::list_stats_h::{OBJ_MOD_CON, OBJ_MOD_DEX, OBJ_MOD_INT, OBJ_MOD_STR, OBJ_MOD_WIS};
use crate::headers::list_terrain_h::FEAT_FLOOR;
use crate::headers::list_tvals_h::*;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;
use std::sync::OnceLock;

/// `CLASS_WARRIOR` .. `CLASS_RANGER` (player-calcs.h class indexes)
const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_RANGER: i32 = 7;

pub(crate) fn exploration_first() -> bool {
    crate::params::value(crate::params::Param::junk_exploration_first) != 0
}

pub(crate) fn gear_alternatives() -> usize {
    use crate::params::{value, Param};
    if exploration_first() {
        value(Param::junk_gear_alternatives) as usize
    } else {
        0
    }
}

/// #1310: under the ranked Home backup policy `junk_gear_alternatives` is the
/// number of regular backups per equipment slot. Gear outside the ranked
/// plan (diggers) keeps the better-alternative cutoff the policy shipped with.
pub(crate) const UNPLANNED_GEAR_CUTOFF: usize = 2;

pub(crate) fn unplanned_gear_cutoff() -> usize {
    if crate::borg_town_live::gear_home_backups() { UNPLANNED_GEAR_CUTOFF } else { gear_alternatives() }
}

/// `PACK_SLOTS`: the pack slots not taken by the quiver.
fn pack_slots(st: &BorgState) -> i32 {
    i32::from(engine::z_info().pack_size) - st.borg.tr(BI_QUIVER_SLOTS)
}

/// `z_info->pack_size`
fn pack_size() -> i32 {
    i32::from(engine::z_info().pack_size)
}

/// `INVEN_TOTAL`
fn inven_total() -> i32 {
    inven_wield() + 12
}

/// `borg_timer(t)`: the turns since `t`.
fn borg_timer(st: &BorgState, t: i32) -> i32 {
    st.borg.time.now - t
}

/// `all_letters_nohjkl[i]` as a key (a C `char` widened to `keycode_t`).
fn letter(i: i32) -> keycode_t {
    engine::all_letters_nohjkl(i) as i8 as keycode_t
}

/// A character literal as a key.
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `'0' + n` as a key (C `int` converted to `keycode_t`).
fn digit(n: i32) -> keycode_t {
    (i32::from(b'0') + n) as keycode_t
}

/// The bytes of a C `desc` buffer, up to its NUL (`%s`).
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// `borg_note(format(fmt, item->desc, ...))` with the description between
/// `before` and `after`.
fn note_desc(st: &mut BorgState, before: &str, i: i32, after: &str) {
    let mut s = before.as_bytes().to_vec();
    s.extend_from_slice(&desc(&st.items.items[i as usize]));
    s.extend_from_slice(after.as_bytes());
    borg_note_bytes(st, &s);
}

/// `OPT(player, birth_randarts)`
fn randarts() -> bool {
    engine::player_opt(OPT_birth_randarts)
}

/// `borg_danger(borg.c.y, borg.c.x, 1, true, false)`: the danger here.
fn danger_here(st: &mut BorgState) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger(st, y, x, 1, true, false)
}

/// `borg_grids[borg.c.y][borg.c.x].feat != FEAT_FLOOR`
fn off_floor(st: &BorgState) -> bool {
    u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) != FEAT_FLOOR
}

/// `i == weapon_swap - 1` or `i == armour_swap - 1` (each swap is the
/// slot + 1, 0 = none; both read now, as C reads the globals).
pub(crate) fn is_swap(st: &BorgState, i: i32) -> bool {
    let w = st.trait_swap.weapon_swap;
    if w != 0 && i == w - 1 {
        return true;
    }
    let a = st.trait_swap.armour_swap;
    a != 0 && i == a - 1
}

/// `memcpy(&borg_items[to], &safe_items[from], sizeof(borg_item))`
fn from_safe(st: &mut BorgState, to: i32, from: i32) {
    st.items.items[to as usize] = st.items.safe_items[from as usize];
}

/// `borg_items[i].iqty += d` (an `uint8_t`).
fn add_iqty(st: &mut BorgState, i: i32, d: i8) {
    let q = &mut st.items.items[i as usize].iqty;
    *q = q.wrapping_add_signed(d);
}

/// `borg.antibounce_count++` (an `int16_t`).
fn antibounce(st: &mut BorgState) {
    st.borg.antibounce_count = st.borg.antibounce_count.wrapping_add(1);
}

/// `value += n` with a `long` constant, stored back into an `int32_t`.
fn add_long(value: i32, n: i64) -> i32 {
    (i64::from(value) + n) as i32
}

/// `iqty * n` with a `long` constant, stored into an `int32_t`.
fn qty_long(item: &borg_item, n: i64) -> i32 {
    (i64::from(item.iqty) * n) as i32
}

/// `iqty * v` in C `int` arithmetic.
fn qty_int(item: &borg_item, v: i32) -> i32 {
    i32::from(item.iqty).wrapping_mul(v)
}

/// `trait * n` with a `long` constant, stored into an `int32_t`.
fn trait_long(st: &BorgState, t: u32, n: i64) -> i32 {
    (i64::from(st.borg.tr(t)) * n) as i32
}

/// The item gives a bonus to a stat (`modifiers[OBJ_MOD_STR..CON] > 0`).
fn stat_bonus(item: &borg_item) -> bool {
    [OBJ_MOD_STR, OBJ_MOD_INT, OBJ_MOD_WIS, OBJ_MOD_DEX, OBJ_MOD_CON]
        .iter()
        .any(|&m| item.modifiers[m as usize] > 0)
}

/// `item->tval == tval && item->sval == sval`
fn is(item: &borg_item, tval: u32, sval: i32) -> bool {
    u32::from(item.tval) == tval && i32::from(item.sval) == sval
}

/// `TV_DIGGING <= tval <= TV_SWORD` while wielding a digger.
fn weapon_while_digging(st: &BorgState, item: &borg_item) -> bool {
    let t = u32::from(item.tval);
    (TV_DIGGING..=TV_SWORD).contains(&t)
        && u32::from(st.items.items[inven_wield() as usize].tval) == TV_DIGGING
}

/// `borg_keypress` of the item letter for slot `b_i` (pack, `/` +
/// equipment letter, or `|` + quiver digit), as in `borg_drop_slow`.
fn slot_keys(st: &mut BorgState, b_i: i32) {
    if b_i < inven_wield() {
        borg_keypress(st, letter(b_i));
    } else if b_i < quiver_start() {
        borg_keypress(st, key(b'/'));

        borg_keypress(st, letter(b_i - inven_wield()));
    } else {
        /* Quiver Slot */
        borg_keypress(st, key(b'|'));
        borg_keypress(st, digit(b_i - quiver_start()));
    }
}

/// Check if there is an object that needs to be trashed under the borg
/// and destroy it (`borg_destroy_floor`).
pub fn borg_destroy_floor(st: &mut BorgState) -> bool {
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    let t = st.cave.grid(x, y).take;
    /* double check that there is an object here.  This is simulating doing */
    /* "look" so it is okay */
    let Some(floor) = engine::top_floor_object(&st.access, x, y) else {
        return false;
    };
    // On stairs or in town the take tracker can have no entry for our
    // discard. Hide the specific floor stack before native autopickup.
    let surplus = floor.note() == Some(b"borg ignore surplus");
    if !surplus && (t == 0 || st.takes.takes[usize::from(t)].value != -10) {
        return false;
    }

    let kind = if surplus { floor.kind().kidx } else { st.takes.takes[usize::from(t)].kidx } as usize;
    let base = if kind != 0 { engine::kind_base_name(kind) } else { None };
    if let Some(base) = base {
        let s = [b"# Destroying floor item ", base, engine::kind_name(kind)].concat();
        borg_note_bytes(st, &s);
    } else {
        borg_note(st, "# Destroying floor item.");
    }

    /* ignore it now */
    borg_keypress(st, key(b'k'));
    borg_keypress(st, key(b'-'));
    borg_keypress(st, key(b'a'));
    borg_keypress(st, key(b'a'));
    true
}

/// borg-reborn-rs (#470): a Scroll of Deep Descent is never kept. It lands
/// five levels below the deepest level so far (`effect_handler_DEEP_DESCENT`
/// targets `max_depth` + 5), past any plan cap. Once its flavour is known,
/// ignore the kind ('k', the scroll, "All Scrolls of Deep Descent"): the
/// engine then drops every carried copy (`ignore_drop`) and hides those on
/// the floor, so none is picked up again. It takes no game time.
pub fn borg_discard_deep_descent(st: &mut BorgState) -> bool {
    ignore_scroll_kind(st, st.item_val.sv_scroll_deep_descent, " (Deep Descent is never read)")
}

/// borg-reborn-rs (#1273): Unlight characters discard the entire kind so future copies stay on the floor.
pub(crate) fn borg_discard_light_scrolls(st: &mut BorgState) -> bool {
    st.borg.tr(BI_CLASS) == crate::borg_light::CLASS_NECROMANCER
        && ignore_scroll_kind(st, st.item_val.sv_scroll_light, " (Necromancer)")
}

fn ignore_scroll_kind(st: &mut BorgState, sv: i32, reason: &str) -> bool {
    if sv < 0 {
        return false;
    }
    let Some(i) = (0..pack_size()).find(|&i| {
        let item = &st.items.items[i as usize];
        item.iqty > 0 && item.aware && u32::from(item.tval) == TV_SCROLL && i32::from(item.sval) == sv
    }) else {
        return false;
    };
    // Already ignored: the engine drops it on its own.
    if engine::kind(st.items.items[i as usize].kind as usize).ignore != 0 {
        return false;
    }
    note_desc(st, "# Discarding ", i, reason);
    borg_keypress(st, key(b'k'));
    slot_keys(st, i);
    // The ignore menu: a) This item only, b) All scrolls of this kind.
    borg_keypress(st, key(b'b'));
    true
}

/// Attempt to consume an item as a method of destroying it
/// (`borg_consume`).
fn borg_consume(st: &mut BorgState, i: i32) -> bool {
    let item = st.items.items[i as usize];
    let sval = i32::from(item.sval);
    let class = st.borg.tr(BI_CLASS);
    let v = &st.item_val;

    if !item.aware {
        return false;
    }

    /* Special destruction */
    match u32::from(item.tval) {
        TV_POTION => {
            // RESTORE_EXP only helps below max XP (src/effect-handler-general.c:effect_handler_RESTORE_EXP:893-904).
            if sval == v.sv_potion_restore_life {
                return st.borg.tr(BI_ISFIXEXP) != 0 && borg_quaff_potion(st, sval);
            }

            /* Check the potion */
            if [
                v.sv_potion_slime_mold,
                v.sv_potion_cure_light,
                v.sv_potion_cure_serious,
                v.sv_potion_cure_critical,
                v.sv_potion_healing,
                v.sv_potion_star_healing,
                v.sv_potion_life,
                v.sv_potion_restore_mana,
                v.sv_potion_heroism,
                v.sv_potion_berserk,
                v.sv_potion_resist_heat,
                v.sv_potion_resist_cold,
                v.sv_potion_infravision,
                v.sv_potion_detect_invis,
                v.sv_potion_cure_poison,
                v.sv_potion_speed,
                v.sv_potion_inc_exp,
            ]
            .contains(&sval)
            {
                /* Try quaffing the potion */
                return borg_quaff_potion(st, sval);
            }

            // borg-reborn-rs (#1371): under town_stat_trade_policy the town's
            // rule decides (`second_stat_potion_wanted`): a refused potion is
            // not drunk to destroy it, it is sold or dropped.
            if crate::borg_town_capability::stat_trade_policy() {
                if let Some(stat) = crate::borg_town_capability::second_stat_kind(st, sval) {
                    return crate::borg_town_capability::second_stat_potion_wanted(st, stat)
                        && borg_quaff_potion(st, sval);
                }
            }

            /* Gain one/lose one potions */
            let (str2, int2, wis2, dex2, con2) = (
                v.sv_potion_inc_str2,
                v.sv_potion_inc_int2,
                v.sv_potion_inc_wis2,
                v.sv_potion_inc_dex2,
                v.sv_potion_inc_con2,
            );
            if sval == str2 {
                /* Maxed out no need. Don't lose another stat */
                if st.borg.tr(BI_CSTR) >= 118 {
                    return false;
                }

                /* This class does not want to risk losing a different stat */
                if class == CLASS_MAGE || class == CLASS_DRUID || class == CLASS_NECROMANCER {
                    return false;
                }

                /* Otherwise, it should be ok */
                if borg_quaff_potion(st, sval) {
                    return true;
                }
                /* (no break in C: falls through to the other tests) */
            }

            if sval == int2 {
                /* Maxed out no need. Don't lose another stat */
                if st.borg.tr(BI_CINT) >= 118 {
                    return false;
                }

                /* This class does not want to risk losing a different stat */
                if class != CLASS_MAGE && class != CLASS_NECROMANCER {
                    return false;
                }

                /* Otherwise, it should be ok */
                return borg_quaff_potion(st, sval);
            }

            if sval == wis2 {
                /* Maxed out no need. Don't lose another stat */
                if st.borg.tr(BI_CWIS) >= 118 {
                    return false;
                }

                /* This class does not want to risk losing a different stat */
                if class != CLASS_PRIEST && class != CLASS_DRUID {
                    return false;
                }

                /* Otherwise, it should be ok */
                return borg_quaff_potion(st, sval);
            }

            if sval == dex2 {
                /* Maxed out no need. Don't lose another stat */
                if st.borg.tr(BI_CDEX) >= 118 {
                    return false;
                }

                /* This class does not want to risk losing a different stat */
                if class == CLASS_MAGE
                    || class == CLASS_PRIEST
                    || class == CLASS_DRUID
                    || class == CLASS_NECROMANCER
                {
                    return false;
                }

                /* Otherwise, it should be ok */
                return borg_quaff_potion(st, sval);
            }

            if sval == con2 {
                /* Maxed out no need. Don't lose another stat */
                if st.borg.tr(BI_CCON) >= 118 {
                    return false;
                }

                /* Otherwise, it should be ok */
                return borg_quaff_potion(st, sval);
            }
        }

        TV_SCROLL => {
            let scrolls = [
                v.sv_scroll_monster_confusion,
                v.sv_scroll_trap_door_destruction,
                v.sv_scroll_satisfy_hunger,
                v.sv_scroll_dispel_undead,
                v.sv_scroll_blessing,
                v.sv_scroll_holy_chant,
                v.sv_scroll_holy_prayer,
            ];

            /* Check the scroll */
            // borg-reborn-rs (#1273): quiet lighting uses the scroll first.
            let legacy_light = sval == v.sv_scroll_light
                && class != crate::borg_light::CLASS_NECROMANCER
                && !crate::borg_light::light_scroll_idle_spend();
            if legacy_light || scrolls.contains(&sval) {
                /* Try reading the scroll */
                return borg_read_scroll(st, sval);
            }
        }

        TV_FOOD => {
            let grub = [v.sv_food_ration, v.sv_food_slime_mold, v.sv_food_waybread];

            /* Check the grub */
            if grub.contains(&sval) {
                /* Try eating the food (unless Bloated) */
                return st.borg.tr(BI_ISFULL) == 0
                    && borg_eat(st, i32::from(item.tval), sval);
            }
        }

        TV_MUSHROOM => {
            let grub = [
                v.sv_mush_second_sight,
                v.sv_mush_fast_recovery,
                v.sv_mush_cure_mind,
                v.sv_mush_restoring,
                v.sv_mush_emergency,
                v.sv_mush_terror,
                v.sv_mush_stoneskin,
                v.sv_mush_debility,
                v.sv_mush_sprinting,
                v.sv_mush_purging,
            ];

            /* Check the grub */
            if grub.contains(&sval) {
                /* Try eating the food (unless Bloated) */
                return st.borg.tr(BI_ISFULL) == 0
                    && borg_eat(st, i32::from(item.tval), sval);
            }
        }

        _ => {}
    }

    /* Nope */
    false
}

// The first positive value threshold in the ordinary dungeon junk policy.
const EARLY_JUNK_VALUE: i32 = 15;

/// `borg_drop_junk`: the items worth keeping by value at this depth.
fn junk_worth_keeping(st: &BorgState, value: i32) -> bool {
    /* up to level 5, keep anything of any value */
    /* up to level 10, keep anything of any value */
    /* up to level 15, keep anything of value 100 or better */
    /* up to level 30, keep anything of value 500 or better */
    /* up to level 40, keep anything of value 1000 or better */
    /* up to level 60, keep anything of value 1200 or better */
    /* up to level 80, keep anything of value 1400 or better */
    /* up to level 90, keep anything of value 1600 or better */
    /* up to level 95, keep anything of value 4800 or better */
    /* below level 127, keep anything of value 2000 or better */
    const TIERS: [(i32, i32); 10] = [
        (5, 0),
        (10, EARLY_JUNK_VALUE),
        (15, 100),
        (30, 500),
        (40, 1000),
        (60, 1200),
        (80, 1400),
        (90, 1600),
        (95, 4800),
        (127, 5600),
    ];
    let depth = st.borg.tr(BI_CDEPTH);
    TIERS.iter().any(|&(d, v)| depth < d && value > v)
}

/// `borg_drop_junk`: "keep some stuff" (the +5000 list).
fn junk_keep_some(st: &BorgState, item: &borg_item) -> bool {
    let v = &st.item_val;
    let warrior = st.borg.tr(BI_CLASS) == CLASS_WARRIOR;
    item.aware
        && ((is(item, TV_POTION, v.sv_potion_restore_mana) && st.borg.tr(BI_MAXSP) >= 1)
            || is(item, TV_POTION, v.sv_potion_healing)
            || is(item, TV_POTION, v.sv_potion_star_healing)
            || is(item, TV_POTION, v.sv_potion_life)
            || is(item, TV_POTION, v.sv_potion_speed)
            || is(item, TV_ROD, v.sv_rod_drain_life)
            || is(item, TV_ROD, v.sv_rod_healing)
            || (is(item, TV_ROD, v.sv_rod_mapping) && warrior)
            || is(item, TV_STAFF, v.sv_staff_dispel_evil)
            || is(item, TV_STAFF, v.sv_staff_power)
            || is(item, TV_STAFF, v.sv_staff_holiness)
            || is(item, TV_WAND, v.sv_wand_drain_life)
            || is(item, TV_WAND, v.sv_wand_annihilation)
            || (is(item, TV_WAND, v.sv_wand_teleport_away) && warrior)
            || (item.ego_idx != 0 && borg_ego_has_random_power(usize::from(item.ego_idx)))
            || (is(item, TV_SCROLL, v.sv_scroll_teleport_level)
                && st.borg.tr(BI_ATELEPORTLVL) < 1000)
            || is(item, TV_SCROLL, v.sv_scroll_protection_from_evil)
            // borg-reborn-rs (#831, owner): Detect Invisible is not worthless; at 15
            // gold the value tiers would junk it from depth 5 (`value > 15` fails).
            || is(item, TV_SCROLL, v.sv_scroll_detect_invis))
}

/// Keep enough copies after the planned discard. Full-pack clearing drops
/// a whole stack, whereas ordinary and overweight junking drop one item.
fn keeps_restore_life(st: &BorgState, slot: usize, whole_stack: bool, desperate: bool) -> bool {
    let sval = st.item_val.sv_potion_restore_life;
    let item = &st.items.items[slot];
    if desperate || !is_restore_life(item, sval) {
        return false;
    }
    let reserve = restore_life_reserve(st);
    if reserve == 0 {
        return false;
    }
    protects_restore_life(&st.items.items[..inven_wield() as usize], slot, sval, reserve, whole_stack)
}

fn protects_restore_life(pack: &[borg_item], slot: usize, sval: i32, reserve: i32, whole_stack: bool) -> bool {
    let item = &pack[slot];
    if reserve == 0 || !is_restore_life(item, sval) {
        return false;
    }
    let carried: i32 = pack.iter().filter(|item| is_restore_life(item, sval))
        .map(|item| i32::from(item.iqty)).sum();
    let discarded = if whole_stack { i32::from(item.iqty) } else { 1 };
    carried - discarded < reserve
}

/// Choose a spare with a learned curse, even when other runes are unknown.
fn cursed_spare_slot(pack: &[borg_item], clean_light_available: impl Fn() -> bool) -> Option<usize> {
    pack.iter().position(|item| {
        item.iqty > 0
            && item.curses.iter().any(|&known| known)
            && !item.one_ring
            && crate::borg_inventory::borg_wield_slot(item) >= 0
            && (u32::from(item.tval) != TV_LIGHT || clean_light_available())
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CursedSpareAction {
    Refuel,
    Drop,
}

/// #1231: discard known cursed spares at a safe opportunity, including gear
/// that would otherwise be a swap or a candidate for Remove Curse.
pub(crate) fn borg_dispose_cursed_spare(st: &mut BorgState) -> Option<CursedSpareAction> {
    if st.borg.goal.recalling != 0 || off_floor(st) {
        return None;
    }
    let pack = &st.items.items[..pack_size() as usize];
    let slot = cursed_spare_slot(pack, || crate::borg_light::has_clean_light(st, pack))?;
    if danger_here(st) > st.borg.tr(BI_CURHP) / 10 {
        return None;
    }
    if crate::borg_light::borg_refuel_from_cursed_spare(st, slot) {
        return Some(CursedSpareAction::Refuel);
    }

    let slot = slot as i32;
    note_desc(st, "# Dropping cursed spare ", slot, ".");
    drop_ignored_pack_stack(st, slot);
    Some(CursedSpareAction::Drop)
}

fn drop_ignored_pack_stack(st: &mut BorgState, slot: i32) {
    borg_keypress(st, key(b'{'));
    borg_keypress(st, letter(slot));
    borg_keypresses(st, b"borg ignore");
    borg_keypress(st, KC_ENTER);

    borg_keypress(st, key(b'd'));
    borg_keypress(st, letter(slot));
    if st.items.items[slot as usize].iqty > 1 {
        borg_keypress(st, key(b'*'));
        borg_keypress(st, KC_ENTER);
    }
}

// object.txt and Strategy Manual ch. 7, "Bad items". The throw policy
// deliberately keeps a smaller set than the effect-based disposal policy.
const THROWABLE_HARMFUL_POTIONS: &[&str] = &[
    "Sleep", "Blindness", "Confusion", "Poison", "Slowness", "Surprise", "Salt Water",
];

/// Slime Mold Juice is expendable in a fight when its nutrition is unneeded.
pub(crate) fn junk_potion_to_throw(st: &BorgState, item: &borg_item) -> bool {
    if u32::from(item.tval) != TV_POTION || item.iqty == 0 || !item.aware || !item.flavour_aware {
        return false;
    }
    let sval = i32::from(item.sval);
    if sval == st.item_val.sv_potion_slime_mold {
        return st.borg.tr(BI_ISHUNGRY) == 0 && st.borg.tr(BI_ISWEAK) == 0;
    }
    throwable_junk_potion_kind(sval)
}

/// The Potion kind is one of the junk potions the throw policy keeps
/// (`THROWABLE_HARMFUL_POTIONS`).
pub(crate) fn throwable_junk_potion_kind(sval: i32) -> bool {
    static KINDS: OnceLock<Vec<i32>> = OnceLock::new();
    KINDS.get_or_init(|| THROWABLE_HARMFUL_POTIONS.iter().map(|name| {
        let sval = engine::lookup_sval(TV_POTION as i32, name);
        assert!(sval >= 0, "unknown throwable potion {name}");
        sval
    }).collect()).contains(&sval)
}

fn retain_for_throwing(st: &BorgState, item: &borg_item) -> bool {
    crate::params::value(crate::params::Param::attack_throw_junk_potions) != 0
        && junk_potion_to_throw(st, item)
}

/// Ignore only this stack. Angband drops it and filters it out of native
/// auto-pickup, while fresh copies of its kind can still be found later.
pub(crate) fn borg_discard_town_consumable(st: &mut BorgState) -> bool {
    use crate::params::{value as param, Param};

    if param(Param::junk_discard_harmful_in_town) == 0 || st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }
    if engine::feat_is_shop(i32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat)) {
        return false;
    }
    let Some(slot) = (0..pack_size() as usize).find(|&slot| {
        let item = &st.items.items[slot];
        harmful_item(item, engine::kind_effects)
            && !crate::borg_item::item_note(st, item).windows(2).any(|mark| matches!(mark, b"!d" | b"!*"))
            && engine::inven_object(&st.access, slot).is_some_and(|item| !item.marked_ignored())
    }) else {
        return false;
    };
    if danger_here(st) > st.borg.tr(BI_CURHP) / 10 {
        return false;
    }
    note_desc(st, "# Dropping harmful town consumable ", slot as i32, ".");
    borg_keypress(st, key(b'k'));
    slot_keys(st, slot as i32);
    borg_keypress(st, key(b'a')); // textui_cmd_ignore_menu: "This item only".
    true
}

/// borg-reborn-rs (#1371, review of #1372): in the dungeon, drop a known
/// gain-one/lose-one potion the character refuses (`town_stat_trade_floor`).
/// The engine's `pickup_always` picks up whatever the Borg steps on, so the
/// floor filter of `borg_flow_take` alone leaves the stack reaching the pack
/// on another route. Ignore this stack only ('k', "This item only"): the engine
/// drops it and filters it out of auto-pickup, while a stack of the same kind
/// can still be picked up if the character's stats change what it wants. In
/// town the planner sells or drops it (`unwanted_home_item`).
pub(crate) fn borg_discard_refused_stack(st: &mut BorgState) -> bool {
    if !crate::borg_town_capability::stat_trade_floor() || st.borg.tr(BI_CDEPTH) == 0 || off_floor(st) {
        return false;
    }
    let Some(slot) = (0..pack_size() as usize).find(|&slot| {
        let item = &st.items.items[slot];
        item.iqty > 0 && item.aware && item.flavour_aware && item.kind != 0
            && crate::borg_town_capability::refused_stat_trade_kind(st, u32::from(item.tval), i32::from(item.sval))
            && !crate::borg_item::item_note(st, item).windows(2).any(|mark| matches!(mark, b"!d" | b"!*"))
            && engine::inven_object(&st.access, slot).is_some_and(|item| !item.marked_ignored())
    }) else {
        return false;
    };
    if danger_here(st) > st.borg.tr(BI_CURHP) / 10 {
        return false;
    }
    note_desc(st, "# Dropping refused stat potion ", slot as i32, ".");
    borg_keypress(st, key(b'k'));
    slot_keys(st, slot as i32);
    borg_keypress(st, key(b'a')); // textui_cmd_ignore_menu: "This item only".
    true
}

#[derive(Clone, Copy)]
pub(crate) struct SurplusDrop {
    pub slot: usize,
    pub quantity: u8,
    pub sell_first: bool,
}

fn harmful_item(item: &borg_item, effects: impl FnOnce(usize) -> Vec<(u16, i32)>) -> bool {
    use crate::headers::list_effects_h::*;
    use crate::headers::list_player_timed_h::*;
    use crate::headers::list_projections_h::{PROJ_MAKE_TRAP, PROJ_MON_CLONE, PROJ_MON_HEAL, PROJ_MON_SPEED};
    // ui-options.c:sval_dependent supports kind-ignore for these item types.
    item.iqty > 0 && item.aware && item.flavour_aware && item.kind != 0
        && item.art_idx == 0
        && matches!(u32::from(item.tval), TV_POTION | TV_SCROLL | TV_MUSHROOM | TV_FOOD | TV_WAND | TV_STAFF | TV_ROD)
        && effects(item.kind as usize).into_iter().any(|(effect, subtype)| match u32::from(effect) {
            // object.txt: Salt Water always inflicts paralysis, and Surprise
            // can scramble or drain stats. Stat trades use LOSE_RANDOM_STAT.
            EF_TIMED_INC | EF_TIMED_INC_NO_RES | EF_TIMED_SET =>
                matches!(subtype as u32, TMD_SLOW | TMD_BLIND | TMD_PARALYZED
                    | TMD_CONFUSED | TMD_AFRAID | TMD_IMAGE | TMD_POISONED | TMD_CUT
                    | TMD_STUN | TMD_AMNESIA | TMD_SCRAMBLE | TMD_TERROR),
            EF_DRAIN_STAT | EF_CURSE_WEAPON | EF_CURSE_ARMOR | EF_SUMMON | EF_DARKEN_AREA => true,
            EF_TOUCH => subtype == PROJ_MAKE_TRAP as i32,
            EF_BOLT_STATUS | EF_PROJECT_LOS =>
                matches!(subtype as u32, PROJ_MON_CLONE | PROJ_MON_HEAL | PROJ_MON_SPEED),
            _ => false,
        })
}

fn harmful_item_drop(st: &BorgState, make_room: bool) -> Option<SurplusDrop> {
    let keep_throwables = !make_room && st.borg.tr(BI_CDEPTH) != 0;
    let slot = st.items.items[..pack_size() as usize].iter().position(|item| {
        harmful_item(item, engine::kind_effects) && (!keep_throwables || !retain_for_throwing(st, item))
    })?;
    Some(SurplusDrop { slot, quantity: st.items.items[slot].iqty, sell_first: false })
}

pub(crate) fn borg_discard_harmful_item(st: &mut BorgState, make_room: bool) -> bool {
    if st.borg.goal.recalling != 0 {
        return false;
    }
    if let Some(drop) = harmful_item_drop(st, make_room) {
        if danger_here(st) <= st.borg.tr(BI_CURHP) / 20 {
            return drop_surplus(st, drop);
        }
    }
    false
}

fn expendable_town_spare(item: &borg_item, worn: &borg_item) -> bool {
    if item.iqty == 0 || !item.aware {
        return false;
    }
    if item.curses.contains(&true) {
        return true;
    }
    let plain = item.to_a <= 0 && item.to_h <= 0 && item.to_d <= 0;
    (plain || item.value <= EARLY_JUNK_VALUE)
        && !crate::borg_town_home::known_special(item)
        && crate::borg_town_home::outclasses_spare(worn, item)
}

fn town_spare_drop(st: &mut BorgState) -> Option<SurplusDrop> {
    if let Some(drop) = surplus_light_drop(st) { return Some(drop); }
    let shared = exploration_first();
    let pack = &st.items.items[..pack_size() as usize];
    let legacy_drop = pack.iter().enumerate().filter(|&(slot, item)| {
        let cursed = item.curses.contains(&true);
        if shared && !cursed {
            return false;
        }
        if !matches!(u32::from(item.tval), TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOW
            | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR | TV_CLOAK | TV_SHIELD
            | TV_HELM | TV_CROWN | TV_GLOVES | TV_BOOTS) {
            return false;
        }
        let worn = crate::borg_inventory::borg_wield_slot(item) as usize;
        (cursed || !is_swap(st, slot as i32))
            && expendable_town_spare(item, &st.items.items[worn])
    }).min_by_key(|&(_, item)| {
        (!item.curses.contains(&true), item.value, std::cmp::Reverse(item.weight))
    }).map(|(slot, item)| SurplusDrop { slot, quantity: item.iqty, sell_first: false });
    if !shared || legacy_drop.is_some() {
        return legacy_drop;
    }
    let cutoff = gear_alternatives();
    if cutoff > 0 { return ranked_pack_spare(st, cutoff); }
    find_inferior_spare(st, |_, slot, item| {
        Some(SurplusDrop { slot, quantity: item.iqty, sell_first: true })
    })
}

/// Compare the fractional surplus above each target. Recheck the actual
/// inventory after every useful copy, so no stack keeps priority as it shrinks.
pub(crate) fn surplus_drop(st: &mut BorgState, lighten: bool) -> Option<SurplusDrop> {
    if st.borg.goal.recalling != 0 {
        return None;
    }
    if let Some(drop) = harmful_item_drop(st, false) {
        return Some(drop);
    }
    if st.borg.tr(BI_CDEPTH) == 0 {
        if let Some(drop) = town_spare_drop(st) {
            return Some(drop);
        }
    }
    if !lighten {
        return None;
    }
    let supplies = crate::borg_town_capability::carried_supplies(st);
    (0..pack_size() as usize).chain(quiver_start() as usize..quiver_end() as usize)
        .filter_map(|slot| {
            if exploration_first() && supplies.keeps_exploration_supply(slot, 1) {
                return None;
            }
            let (excess, target) = supplies.discard_surplus(st, slot)?;
            Some((slot, excess, target))
        })
        .reduce(|a, b| {
            let ratio = (i128::from(b.1) * i128::from(a.2)).cmp(&(i128::from(a.1) * i128::from(b.2)));
            let (first, second) = (&st.items.items[a.0], &st.items.items[b.0]);
            let order = ratio.then_with(|| {
                if first.tval == second.tval && matches!(u32::from(first.tval), TV_SHOT | TV_ARROW | TV_BOLT) {
                    let damage = |ammo: &borg_item| crate::borg_ranged::missile_damage_twice(ammo, 0);
                    damage(first).cmp(&damage(second))
                } else {
                    std::cmp::Ordering::Equal
                }
            }).then_with(|| second.weight.cmp(&first.weight));
            if order.is_gt() { b } else { a }
        })
        .map(|(slot, _, _)| SurplusDrop { slot, quantity: 1, sell_first: false })
}

/// borg-reborn-rs (#1397, review of #1398): a drop stays on a grid only on
/// open floor with no trap, no glyph and no visible pile.
/// obj-pile.c:drop_find_grid skips `square_istrap` grids, its own start
/// included, and a rune of protection is a trap there; the Borg's map keeps
/// it apart, in `glyph` (borg_update.rs), from disarmable traps in `trap`.
fn drop_lands_here(feat: u32, trap: bool, glyph: bool, pile: bool) -> bool {
    feat == FEAT_FLOOR && !trap && !glyph && !pile
}

/// Make sure a discard drop lands under the Borg. obj-pile.c:drop_find_grid
/// scatters a drop made off plain floor (a shop entrance or stairs, which
/// hold no objects: cave-square.c:square_isobjectholding) or onto a visible
/// pile. A scattered drop is never under the Borg, so `borg_destroy_floor`
/// cannot ignore it, and with `pickup_always` on (cmd-pickup.c:169) the
/// engine picks it up again when the Borg walks over it (#1397).
///
/// `None`: the Borg stands on clear floor, drop here. `Some(true)`: a step
/// to an adjacent clear floor grid was queued; plan the drop again from
/// there. `Some(false)`: no such grid; do not drop now.
pub(crate) fn step_to_drop_floor(st: &mut BorgState) -> Option<bool> {
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    let here = st.cave.grid(x, y);
    if drop_lands_here(u32::from(here.feat), here.trap, here.glyph,
        engine::top_floor_object(&st.access, x, y).is_some())
    {
        return None;
    }
    for direction in 0..8 {
        let nx = x + i32::from(engine::ddx_ddd(direction));
        let ny = y + i32::from(engine::ddy_ddd(direction));
        if !engine::square_in_bounds_fully(nx, ny) {
            continue;
        }
        let grid = st.cave.grid(nx, ny);
        if drop_lands_here(u32::from(grid.feat), grid.trap, grid.glyph,
                engine::top_floor_object(&st.access, nx, ny).is_some())
            && grid.kill == 0
            && grid.info & crate::borg_cave_view::BORG_VIEW != 0
            && borg_danger(st, ny, nx, 1, true, false) <= st.borg.tr(BI_CURHP) / 20
        {
            borg_note(st, "# Stepping to clear floor before dropping");
            borg_keypress(st, digit(i32::from(engine::ddd(direction))));
            return Some(true);
        }
    }
    Some(false)
}

pub(crate) fn drop_surplus(st: &mut BorgState, drop: SurplusDrop) -> bool {
    let item = &st.items.items[drop.slot];
    if harmful_item(item, engine::kind_effects)
        && engine::kind(item.kind as usize).ignore == 0
    {
        // ui-object.c:textui_cmd_ignore_menu offers the individual
        // stack first and the known kind second. Leave the kind available
        // when the Borg could throw a fresh copy on a later trip.
        let selection = if retain_for_throwing(st, item) { b'a' } else { b'b' };
        let slot = drop.slot as i32;
        note_desc(st, "# Discarding known harmful stack ", slot, ".");
        borg_keypress(st, key(b'k'));
        slot_keys(st, slot);
        borg_keypress(st, key(selection));
        return true;
    }
    if let Some(stepped) = step_to_drop_floor(st) {
        return stepped;
    }
    let slot = drop.slot as i32;
    let quantity = st.items.items[drop.slot].iqty;
    note_desc(st, "# Dropping surplus ", slot, &format!(" ({} copies).", drop.quantity));
    // Inventory refresh removes the pending Borg inscription before the
    // next action. Inscribe each drop so only the dropped copies keep it.
    borg_keypress(st, key(b'{'));
    slot_keys(st, slot);
    borg_keypresses(st, b"borg ignore surplus");
    borg_keypress(st, KC_ENTER);
    borg_keypress(st, key(b'd'));
    slot_keys(st, slot);
    if quantity > 1 {
        if drop.quantity == quantity {
            borg_keypress(st, key(b'*'));
        } else {
            borg_keypresses(st, drop.quantity.to_string().as_bytes());
        }
        borg_keypress(st, KC_ENTER);
    }
    true
}

#[cfg(feature = "test-cursed-spares")]
pub(crate) fn check_cursed_spares(st: &mut BorgState) -> ! {
    use crate::borg_headers::borg_item_h::BORG_CURSE_PARALYSIS;
    use crate::borg_io::borg_flush;
    use crate::headers::list_object_flags_h::{OF_LIGHT_2, OF_LIGHT_3, OF_TAKES_FUEL};
    use crate::headers::list_terrain_h::FEAT_MORE;

    let light = |timeout, flags: &[u32]| {
        let mut item = crate::borg_item::blank_item();
        item.iqty = 1;
        item.tval = TV_LIGHT as u8;
        item.timeout = timeout;
        for &flag in flags {
            let bit = (flag as i32 - engine::FLAG_START) as usize;
            item.flags[bit / 8] |= 1 << (bit % 8);
        }
        item
    };
    let curse = |mut item: borg_item| {
        item.cursed = true;
        item.needs_ident = true;
        item.curses[BORG_CURSE_PARALYSIS as usize] = true;
        item
    };

    let equipped = crate::borg_light::inven_light() as usize;
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    st.borg.goal.recalling = 0;
    st.borg.trait_[BI_CDEPTH as usize] = 5;
    st.borg.trait_[BI_LIGHT as usize] = 3;
    st.borg.trait_[BI_CURHP as usize] = 100;
    st.items.items.fill(crate::borg_item::blank_item());
    st.items.items[equipped] = light(1200, &[OF_LIGHT_3, OF_TAKES_FUEL]);
    st.items.items[0] = curse(light(7500, &[OF_LIGHT_3, OF_TAKES_FUEL]));
    st.kills.nxt = 1;
    st.borg.antibounce_count = 0;
    st.danger = Default::default();
    borg_flush(st);

    st.cave.grid_mut(x, y).feat = FEAT_MORE as u8;
    assert_eq!(borg_dispose_cursed_spare(st), None, "wait for ordinary floor");
    st.cave.grid_mut(x, y).feat = FEAT_FLOOR as u8;
    st.borg.goal.recalling = 1;
    assert_eq!(borg_dispose_cursed_spare(st), None, "wait for recall");
    st.borg.goal.recalling = 0;
    st.danger.fear_monsters[y as usize][x as usize] = 20;
    assert_eq!(borg_dispose_cursed_spare(st), None, "wait for low danger");
    assert!(st.io.queued_keys().is_empty());
    st.danger.fear_monsters[y as usize][x as usize] = 0;

    assert_eq!(borg_dispose_cursed_spare(st), Some(CursedSpareAction::Refuel));
    assert_eq!(st.io.queued_keys(), b"Fa".iter().copied().map(u32::from).collect::<Vec<_>>());
    borg_flush(st);

    st.borg.trait_[BI_LIGHT as usize] = 2;
    st.items.items[equipped] = light(3, &[OF_LIGHT_2]);
    st.items.items[0] = curse(light(5000, &[OF_LIGHT_2]));
    st.items.items[0].iqty = 2;
    assert_eq!(borg_dispose_cursed_spare(st), None, "keep a light through imminent burnout");
    assert!(st.io.queued_keys().is_empty());
    st.items.items[1] = light(5000, &[OF_LIGHT_2]);
    assert_eq!(borg_dispose_cursed_spare(st), Some(CursedSpareAction::Drop));
    let mut drop = b"{aborg ignore".iter().copied().map(u32::from).collect::<Vec<_>>();
    drop.extend([KC_ENTER, u32::from(b'd'), u32::from(b'a'), u32::from(b'*'), KC_ENTER]);
    assert_eq!(st.io.queued_keys(), drop);
    // Town arrival retains its separate cleanup for learned curses.
    assert!(exploration_first());
    st.borg.trait_[BI_CDEPTH as usize] = 0;
    st.items.items[0] = curse(borg_item { tval: TV_SWORD as u8, iqty: 1, aware: true,
        ..crate::borg_item::blank_item() });
    let arrival = surplus_drop(st, false).expect("town arrival should clear the cursed weapon");
    assert_eq!((arrival.slot, arrival.quantity, arrival.sell_first), (0, 1, false));
    println!("#1231 cursed spare actions passed");
    std::process::exit(0);
}

#[cfg(feature = "test-cursed-spares")]
pub(crate) fn check_cursed_lantern_in_game(st: &mut BorgState) {
    #[derive(Clone, Copy)]
    enum Step {
        Equip,
        Refuel,
        Drop { fuel_before: i16 },
        VerifyDrop,
    }
    thread_local! {
        static STEP: std::cell::Cell<Step> = const { std::cell::Cell::new(Step::Equip) };
    }

    let (x, y) = (st.borg.c.x, st.borg.c.y);
    assert_eq!(u32::from(engine::square_feat(x.into(), y.into())), FEAT_FLOOR);
    // Isolate the item action from the surrounding dungeon's danger.
    st.kills.nxt = 1;
    st.danger = Default::default();

    let equipped = crate::borg_light::inven_light() as usize;
    let lantern = st.item_val.sv_light_lantern;
    let is_lantern = |item: &borg_item| {
        item.iqty > 0 && u32::from(item.tval) == TV_LIGHT && i32::from(item.sval) == lantern
    };
    let pack_len = pack_size() as usize;
    let pack = &st.items.items[..pack_len];
    match STEP.get() {
        Step::Equip => {
            engine::carry_test_lanterns(st);
            crate::borg_item::borg_find_all_items(st);
            let slot = st.items.items[..pack_len].iter().position(|item| is_lantern(item) && !item.cursed)
                .expect("clean lantern in pack");
            borg_keypress(st, key(b'w'));
            borg_keypress(st, letter(slot as i32));
            STEP.set(Step::Refuel);
        }
        Step::Refuel => {
            let lamp = &st.items.items[equipped];
            assert!(is_lantern(lamp) && !lamp.cursed);
            let fuel_before = lamp.timeout;
            let (slot, donor) = pack.iter().enumerate().find(|(_, item)| is_lantern(item))
                .expect("donor in pack");
            assert!(donor.cursed && donor.needs_ident);
            assert_eq!(donor.timeout, 7500);
            let desc = engine::object_desc_full(engine::inven_object(&st.access, slot).unwrap());
            assert!(desc.ends_with(b"{cursed, ??}"), "{}", String::from_utf8_lossy(&desc));
            assert_eq!(borg_dispose_cursed_spare(st), Some(CursedSpareAction::Refuel));
            STEP.set(Step::Drop { fuel_before });
        }
        Step::Drop { fuel_before } => {
            let fuel_gained = st.items.items[equipped].timeout - fuel_before;
            // Angband can burn one turn of fuel before asking for the next key.
            assert!((7499..=7500).contains(&fuel_gained), "gained {fuel_gained} fuel");
            let donor = pack.iter().find(|item| is_lantern(item)).expect("drained donor in pack");
            assert_eq!(donor.timeout, 0);
            assert_eq!(borg_dispose_cursed_spare(st), Some(CursedSpareAction::Drop));
            STEP.set(Step::VerifyDrop);
        }
        Step::VerifyDrop => {
            assert!(!pack.iter().any(is_lantern));
            let dropped = engine::top_floor_object(&st.access, x.into(), y.into())
                .expect("lantern on the floor");
            let kind = dropped.kind();
            assert_eq!((kind.tval, kind.sval), (TV_LIGHT as i32, lantern));
            assert_eq!(dropped.note(), Some(b"borg ignore".as_slice()));
            println!("#1231 cursed lantern refuel and drop passed");
            std::process::exit(0);
        }
    }
}

/// Drop "junk" items, marking them for destruction (`borg_drop_junk`).
pub fn borg_drop_junk(st: &mut BorgState) -> bool {
    if st.borg.tr(crate::borg_headers::borg_trait_h::BI_CDEPTH) == 0 && crate::borg_town_live::active(st) && st.cfg(crate::borg_headers::borg_h::BORG_TOWN_NO_STORES) == 0 { return false; }
    let mut b_i: i32 = -1;
    let mut fix = false;
    let mut b_p: i32 = st.borg.power;
    let mut value: i32 = 0;

    /* No need */
    if !st.inventory.do_crush_junk {
        return false;
    }

    /* not while recalling since this is a two step process */
    /* and if the recall kicks in in the middle it confuses things */
    if st.borg.goal.recalling != 0 {
        return false;
    }

    /* No crush if even slightly dangerous */
    if danger_here(st) > st.borg.tr(BI_CURHP) / 10 {
        return false;
    }

    let reserves = exploration_first().then(|| crate::borg_town_capability::carried_supplies(st));

    /* Destroy actual "junk" items */
    for i in 0..pack_size() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);

        /* Skip empty items */
        if item.iqty == 0 || crate::borg_light::keep_fuel_stack(st, i as usize)
            || crate::borg_ranged::keep_stack(st, i as usize)
            || keeps_restore_life(st, i as usize, false, false)
            || crate::borg_town_home::keep_in_dungeon(st, i as usize) { // borg-reborn-rs (#1241): retain useful Home candidates until town.
            continue;
        }

        if reserves.as_ref().is_some_and(|m| m.keeps_exploration_supply(i as usize, item.iqty)) {
            continue;
        }

        /* dont crush the swap */
        if is_swap(st, i) {
            continue;
        }

        /* dont crush our spell books */
        if engine::obj_kind_can_browse(item.kind as usize) {
            continue;
        }

        /* Do not crush unID'd Scrolls, sell them in town */
        if tval == TV_SCROLL && !item.aware {
            continue;
        }

        /* Do not crush Boots, they could be SPEED */
        if tval == TV_BOOTS && !item.ident {
            continue;
        }

        /* save the items value */
        value = item.value;

        /* Crush Stacked Wands and Staves that are empty.
         * ie. 5 Staffs of Teleportation (2 charges).
         * Only 2 charges in 5 staves means top 3 are empty.
         */
        if (tval == TV_STAFF || tval == TV_WAND)
            && item.aware
            && i32::from(item.iqty) > i32::from(item.pval)
        {
            value = 0;
        }

        /* Skip non "worthless" items */
        if tval < TV_CHEST {
            continue;
        }

        /* unknown and not worthless */
        if !item.ident && value > 0 {
            continue;
        }

        /* skip items that are 'valuable'.  This is level dependent */
        /* try to make the borg junk +1,+1 dagger at level 40 */

        /* if the item gives a bonus to a stat, boost its value */
        if stat_bonus(&item) && value > 0 {
            value = add_long(value, 2000);
        }

        /* Keep some stuff */
        if i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL) && value > 0 {
            value = add_long(value, 5000);
        }

        if junk_keep_some(st, &item) {
            value = add_long(value, 5000);
        }

        /* Go Ahead and crush diggers */
        if tval == TV_DIGGING {
            value = 0;
        }

        /* Crush missiles that aren't mine */
        if (tval == TV_SHOT || tval == TV_ARROW || tval == TV_BOLT)
            && i32::from(item.tval) != st.borg.tr(BI_AMMO_TVAL)
        {
            value = 0;
        }

        /* borg_worships_gold will sell all kinds of stuff,
         * except {cursed} is junk
         */
        let scum = st.cfg(BORG_MONEY_SCUM_AMOUNT);
        if item.value > 0
            && ((st.cfg(BORG_WORSHIPS_GOLD) != 0 || st.borg.tr(BI_MAXCLEVEL) < 10)
                || (scum < st.borg.tr(BI_GOLD) && scum != 0))
            && st.borg.tr(BI_MAXCLEVEL) <= 20
            && !item.cursed
        {
            continue;
        }

        if junk_worth_keeping(st, value) {
            continue;
        }

        /* Skip good un-id'd "artifacts" */
        if borg_item_note_needs_id(st, &item) {
            continue;
        }

        /* With random artifacts some are good and bad so check them all */
        if randarts() && item.art_idx != 0 && !item.ident {
            continue;
        }

        /* Destroy one item */
        add_iqty(st, i, -1);

        /* Fix later */
        fix = true;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let p = borg_power(st);

        /* Restore the item */
        from_safe(st, i, i);

        /* skip things we are using */
        if p > b_p {
            b_p = p;
            b_i = i;
        }
    }

    /* if we found something junky, junk it */
    if b_i != -1 {
        /* Message (C prints the value of the last item scanned) */
        note_desc(st, "# Dropping junk ", b_i, &format!(" (valued at {})", value));

        /* inscribe "borg ignore". The borg crushes all items */
        /* on the floor that are inscribed this way */
        borg_keypress(st, key(b'{'));
        borg_keypress(st, letter(b_i));
        borg_keypresses(st, b"borg ignore");
        borg_keypress(st, KC_ENTER);

        /* drop it then ignore it */
        borg_keypress(st, key(b'd'));
        borg_keypress(st, letter(b_i));

        if st.items.items[b_i as usize].iqty > 1 {
            borg_keypress(st, key(b'1'));
            borg_keypress(st, KC_ENTER);
        }

        /* Success */
        return true;
    }

    /* re-examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* No need */
    st.inventory.do_crush_junk = false;

    /* Nothing to destroy */
    false
}

/// `borg_drop_hole`: "never crush cool stuff that we might be needing
/// later" (aware items only).
fn hole_cool_stuff(st: &BorgState, item: &borg_item) -> bool {
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    let warrior = st.borg.tr(BI_CLASS) == CLASS_WARRIOR;
    (is(item, TV_POTION, v.sv_potion_restore_mana) && st.borg.tr(BI_MAXSP) >= 1)
        || is(item, TV_POTION, v.sv_potion_healing)
        || is(item, TV_POTION, v.sv_potion_star_healing)
        || is(item, TV_POTION, v.sv_potion_life)
        || is(item, TV_POTION, v.sv_potion_speed)
        || is(item, TV_SCROLL, v.sv_scroll_protection_from_evil)
        || is(item, TV_SCROLL, v.sv_scroll_rune_of_protection)
        || (is(item, TV_SCROLL, v.sv_scroll_teleport_level)
            && st.borg.tr(BI_ATELEPORTLVL) < 1000)
        || (u32::from(item.tval) == TV_ROD
            && (sval == v.sv_rod_healing || (sval == v.sv_rod_mapping && warrior))
            && item.iqty <= 5)
        || (is(item, TV_WAND, v.sv_wand_teleport_away)
            && warrior
            && st.borg.tr(BI_ATPORTOTHER) <= 8)
        || (is(item, TV_ROD, v.sv_rod_light) && st.borg.tr(BI_LIGHT) <= 0)
}

/// `borg_drop_hole`: the value of an aware item when the Borg is above
/// level 35 (the `switch (item->tval)`; the `!=` chains are C's, see
/// docs/c-bugs.md).
fn hole_high_level_value(st: &BorgState, item: &borg_item, value: i32) -> i32 {
    let v = &st.item_val;
    let sval = i32::from(item.sval);
    match u32::from(item.tval) {
        /* rings are under valued. */
        TV_RING => qty_int(item, value).wrapping_mul(10),

        TV_AMULET | TV_BOW | TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOOTS | TV_GLOVES
        | TV_HELM | TV_CROWN | TV_SHIELD | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR => {
            qty_int(item, value).wrapping_mul(5)
        }

        TV_CLOAK => {
            if item.ego_idx != 0 && borg_ego_has_random_power(usize::from(item.ego_idx)) {
                qty_long(item, 300000)
            } else {
                qty_int(item, value)
            }
        }

        TV_ROD => {
            /* BIG HACK! don't crush cool stuff. */
            if (sval == v.sv_rod_drain_life)
                || (sval == v.sv_rod_acid_ball)
                || (sval == v.sv_rod_elec_ball)
                || (sval == v.sv_rod_fire_ball)
                || (sval == v.sv_rod_cold_ball)
            {
                qty_long(item, 300000) /* value at 30k */
            } else {
                qty_int(item, value)
            }
        }

        TV_STAFF => {
            /* BIG HACK! don't crush cool stuff. */
            if sval == v.sv_staff_dispel_evil
                || ((sval == v.sv_staff_power || sval == v.sv_staff_holiness)
                    && st.borg.tr(BI_GOOD_S_CHG) < 2)
                || (sval == v.sv_staff_destruction && st.borg.tr(BI_ASTFDEST) < 2)
            {
                qty_long(item, 300000) /* value at 30k */
            } else {
                qty_int(item, value / 2)
            }
        }

        TV_WAND => {
            /* BIG HACK! don't crush cool stuff. */
            if (sval == v.sv_wand_drain_life)
                || (sval == v.sv_wand_teleport_away)
                || (sval == v.sv_wand_acid_ball)
                || (sval == v.sv_wand_elec_ball)
                || (sval == v.sv_wand_fire_ball)
                || (sval == v.sv_wand_cold_ball)
                || (sval == v.sv_wand_annihilation)
                || (sval == v.sv_wand_dragon_fire)
                || (sval == v.sv_wand_dragon_cold)
            {
                qty_long(item, 300000) /* value at 30k */
            } else {
                qty_int(item, value / 2)
            }
        }

        /* scrolls and potions crush easy */
        TV_SCROLL => {
            if (sval == v.sv_scroll_protection_from_evil)
                || (sval == v.sv_scroll_rune_of_protection)
            {
                qty_long(item, 30000)
            } else {
                qty_int(item, value / 10)
            }
        }

        TV_POTION => {
            /* BIG HACK! don't crush heal/mana potions.  It could be */
            /* that we are in town and are collecting them. */
            if (sval == v.sv_potion_healing)
                || (sval == v.sv_potion_star_healing)
                || (sval == v.sv_potion_life)
                || (sval == v.sv_potion_restore_mana)
            {
                qty_long(item, 300000) /* value at 30k */
            } else {
                qty_int(item, value / 10)
            }
        }

        _ => qty_int(item, value / 3),
    }
}

/// Drop something to make a free inventory slot (`borg_drop_hole`).
///
/// This function evaluates the possible worlds that result from
/// the getting rid of each inventory slot, and attempts to drop from
/// that slot which causes the best possible resulting world.
///
/// We attempt to avoid dropping unknown items by "rewarding" the
/// presence of unknown items by a massively heuristic value.
///
/// The borg makes two attempts to find something to drop.  The first
/// time important things are never dropped.  The second time, things
/// that are somewhat important, like things not identified yet, can be
/// considered.
///
/// If the Borg cannot find something to drop, which should only
/// happen if he fills up with artifacts, then he will probably act
/// rather twitchy for a while.
pub fn borg_drop_hole(st: &mut BorgState, desperate: bool) -> bool {
    if st.borg.tr(crate::borg_headers::borg_trait_h::BI_CDEPTH) == 0 && crate::borg_town_live::active(st) && st.cfg(crate::borg_headers::borg_h::BORG_TOWN_NO_STORES) == 0 { return false; }
    let mut b_i: i32 = -1;
    let mut b_p: i32 = 0;
    let mut b_w: i32 = 0;

    let mut fix = false;

    /* Do not destroy items unless we need the space */
    if st.items.items[(pack_slots(st) - 1) as usize].iqty == 0 {
        return false;
    }

    /* only on the floor */
    if off_floor(st) {
        return false;
    }

    /* No crush if even slightly dangerous */
    if st.borg.tr(BI_CDEPTH) != 0
        && (danger_here(st) > st.borg.tr(BI_CURHP) / 10
            && (st.borg.tr(BI_CURHP) != st.borg.tr(BI_MAXHP)
                || danger_here(st) > (st.borg.tr(BI_CURHP) * 2) / 3))
    {
        return false;
    }

    let protect_supplies = exploration_first();
    if protect_supplies && !desperate && st.borg.goal.recalling == 0
        && !engine::player_is_shapechanged()
        && danger_here(st) <= st.borg.tr(BI_CURHP) / 20
    {
        if borg_discard_harmful_item(st, true) {
            return true;
        }
        let cutoff = gear_alternatives();
        let spare = if crate::borg_town_live::gear_home_backups() {
            surplus_light_drop(st).or_else(|| planned_pack_excess(st, true))
                .or_else(|| unplanned_pack_excess(st, true))
                .or_else(|| crate::borg_spare_gear::unknown_rune_excess(st, true)).map(|drop| drop.slot as i32)
        } else if cutoff > 0 {
            (0..pack_size()).find(|&slot| {
                let slot = slot as usize;
                let quantity = st.items.items[slot].iqty;
                quantity > 0 && crate::borg_spare_gear::redundant_copies(
                    st, crate::borg_spare_gear::GearLocation::Pack(slot), cutoff) == quantity
            })
        } else {
            (0..pack_size()).find(|&slot| crate::borg_spare_gear::inferior(st, slot as usize))
        };
        if let Some(slot) = spare {
            note_desc(st, "# Dropping inferior spare to make pack room ", slot, ".");
            drop_ignored_pack_stack(st, slot);
            return true;
        }
    }
    let reserves = protect_supplies.then(|| crate::borg_town_capability::carried_supplies(st));

    /* Scan the inventory */
    for i in 0..pack_size() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);

        let mut value_boost: i32 = 0;

        if reserves.as_ref().is_some_and(|m| m.keeps_exploration_supply(i as usize, item.iqty)) {
            continue;
        }
        if protect_supplies && crate::borg_spare_gear::inspection_pending(st, &item) {
            continue;
        }

        /* Skip empty items and legacy keepers. */
        if item.iqty == 0 || !desperate && (crate::borg_light::keep_fuel_stack(st, i as usize)
            || crate::borg_ranged::keep_stack(st, i as usize)
            || crate::borg_town_home::keep_in_dungeon(st, i as usize))
            || keeps_restore_life(st, i as usize, true, desperate) {
            continue;
        }

        /* Skip "artifacts" */
        if item.art_idx != 0 {
            continue;
        }

        /* skip food */
        if tval == TV_FOOD && st.borg.tr(BI_FOOD) < 5 {
            continue;
        }

        /* don't crush the swap weapon */
        if is_swap(st, i) {
            continue;
        }

        /* don't crush our spell books */
        if engine::obj_kind_can_browse(item.kind as usize) {
            continue;
        }

        /* Do not crush Boots, they could be SPEED */
        if tval == TV_BOOTS && !item.ident {
            if desperate {
                value_boost = 10000;
            } else {
                continue;
            }
        }

        /* Don't crush weapons if we are wielding a digger */
        if weapon_while_digging(st, &item) {
            if desperate {
                value_boost = 10000;
            } else {
                continue;
            }
        }

        /* Skip "artifacts" */
        if item.art_idx != 0 && !item.ident {
            continue;
        }

        /* things with unknown runes */
        if borg_item_note_needs_id(st, &item) {
            if desperate {
                value_boost = 5000;
            } else {
                continue;
            }
        }

        /* never crush cool stuff that we might be needing later */
        if !desperate {
            if item.aware && hole_cool_stuff(st, &item) {
                continue;
            }
        } else {
            value_boost = 7000;
        }

        /* a boost for things with random powers */
        if item.ego_idx != 0
            && borg_ego_has_random_power(usize::from(item.ego_idx))
            && !item.ident
        {
            if desperate {
                value_boost = 10000;
            } else {
                continue;
            }
        }

        /* save the items value */
        let mut value = item.value.wrapping_add(value_boost);

        /* save the items weight */
        let w = i32::from(borg_item_weight(&item));

        /* Destroy the item */
        st.items.items[i as usize].iqty = 0;

        /* Fix later */
        fix = true;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let mut p = borg_power(st);

        /* Restore the item */
        from_safe(st, i, i);

        /* (C reads the item through its pointer from here on) */
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);

        /* Penalize loss of "gold" */

        /* if the item gives a bonus to a stat, boost its value */
        if stat_bonus(&item) {
            value = add_long(value, 20000);
        }

        /* Keep the correct types of missiles which have value
         * if we do have have tons already. unless in town, you can junk em in
         * town.
         */
        if (i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL))
            && (value > 0)
            && (st.borg.tr(BI_AMISSILES) <= 35)
            && st.borg.tr(BI_CDEPTH) >= 1
        {
            value = add_long(value, 5000);
        }

        /* Show preference for destroying things we will not use */
        /* if we are high enough level not to worry about gold. */
        if st.borg.tr(BI_CLEVEL) > 35 && item.aware {
            value = hole_high_level_value(st, &item, value);
        } else {
            value = qty_int(&item, value);
        }

        /* Try not to destroy "unaware" items
         * unless deep
         */
        if !item.aware && (value > 0) {
            /* Reward "unaware" items */
            match tval {
                TV_RING | TV_AMULET => value = trait_long(st, BI_MAXDEPTH, 5000),
                TV_ROD => value = trait_long(st, BI_MAXDEPTH, 3000),
                TV_STAFF | TV_WAND => value = trait_long(st, BI_MAXDEPTH, 2000),
                TV_SCROLL | TV_POTION => value = trait_long(st, BI_MAXDEPTH, 500),
                TV_FOOD => value = trait_long(st, BI_MAXDEPTH, 10),
                _ => {}
            }
        }

        /* Try not to destroy "unknown" items */
        if !item.ident && (value > 0) && borg_item_worth_id(st, &item) {
            /* Reward "unknown" items */
            match tval {
                TV_SHOT | TV_ARROW | TV_BOLT => value = add_long(value, 100),
                TV_BOW => value = add_long(value, 20000),
                TV_DIGGING => value = add_long(value, 10),
                TV_HAFTED | TV_POLEARM | TV_SWORD => value = add_long(value, 10000),
                TV_BOOTS | TV_GLOVES | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK => {
                    value = add_long(value, 15000)
                }
                TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_DRAG_ARMOR => value = add_long(value, 15000),
                TV_AMULET | TV_RING => value = add_long(value, 5000),
                TV_STAFF | TV_WAND => value = add_long(value, 1000),
                _ => {}
            }
        }

        /* power is much more important than gold. */
        value /= 100;

        /* If I have no food, and in town, I must have a free spot to buy food
         */
        if st.borg.tr(BI_CDEPTH) == 0 && st.borg.tr(BI_FOOD) == 0 {
            /* Power is way more important than gold */
            value /= 500;
        }

        p = p.wrapping_sub(value);

        /* Ignore "bad" swaps */
        if (b_i >= 0) && (p < b_p) {
            continue;
        }

        /* all things being equal, get rid of heavy stuff first */
        if b_p == p && w < b_w {
            continue;
        }

        /* Maintain the "best" */
        b_i = i;
        b_p = p;
        b_w = w;
    }

    /* Examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* Attempt to destroy it */
    if b_i >= 0 {
        /* Debug */
        let gold = i64::from(st.borg.power) * 100 - i64::from(b_p);
        borg_note(st, &format!("# Junking {} gold (full)", gold));

        /* Try to consume the junk */
        if borg_consume(st, b_i) {
            return true;
        }

        /* Message */
        note_desc(st, "# Dropping ", b_i, ".");

        drop_ignored_pack_stack(st, b_i);

        /* Success */
        return true;
    }

    /* if we got to here, we need to make room but have nothing we can crush */
    /* try again but allow more things to be crushed */
    if !desperate {
        return borg_drop_hole(st, true);
    }

    false
}

/// Drop "junk" when slow (in the dungeon) (`borg_drop_slow`).
///
/// We penalize the loss of both power and monetary value, and reward
/// the loss of weight that may be slowing us down.  The weight loss
/// is worth one gold per tenth of a pound.  This causes things like
/// lanterns and chests and spikes to be considered "annoying".
pub fn borg_drop_slow(st: &mut BorgState) -> bool {
    if st.borg.tr(crate::borg_headers::borg_trait_h::BI_CDEPTH) == 0 && crate::borg_town_live::active(st) && st.cfg(crate::borg_headers::borg_h::BORG_TOWN_NO_STORES) == 0 { return false; }
    let mut b_i: i32 = -1;
    let mut b_p: i32 = 0;

    let mut fix = false;

    /* No crush if even slightly dangerous */
    if danger_here(st) > st.borg.tr(BI_CURHP) / 20 {
        return false;
    }

    /* Never in town */
    if st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }

    if crate::borg_town_live::unload_on_arrival(st) {
        // player-calcs.c:calc_bonuses charges the first point of slowdown at
        // half the weight limit plus one tenth of the limit.
        let carry = st.borg.tr(BI_CARRY);
        if carry > 0 && st.borg.tr(BI_WEIGHT) >= carry / 2 + carry / 10 {
            if let Some(drop) = surplus_drop(st, true) {
                return drop_surplus(st, drop);
            }
        }
        return false;
    }

    /* Do not crush items unless we are slow */
    if st.borg.tr(BI_SPEED) >= 110 {
        return false;
    }

    /* Calculate "greed" factor */
    let mut greed = (i64::from(st.borg.tr(BI_GOLD)) / 100 + 100) as i32;

    /* Minimal and maximal greed */
    if greed < 500 && st.borg.tr(BI_CLEVEL) > 35 {
        greed = 500;
    }
    if greed > 25000 {
        greed = 25000;
    }

    /* Decrease greed by our slowness */
    greed = greed.wrapping_sub((110 - st.borg.tr(BI_SPEED)).wrapping_mul(500));
    if greed <= 0 {
        greed = 0;
    }

    let reserves = exploration_first().then(|| crate::borg_town_capability::carried_supplies(st));
    /* Scan for junk */
    for i in 0..quiver_end() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);

        if reserves.as_ref().is_some_and(|m| m.keeps_exploration_supply(i as usize, 1)) {
            continue;
        }

        /* Skip empty items */
        if item.iqty == 0 || crate::borg_light::keep_fuel_stack(st, i as usize)
            || crate::borg_ranged::keep_stack(st, i as usize)
            || keeps_restore_life(st, i as usize, false, false) {
            continue;
        }

        /* Skip our equipment, but do not skip the quiver */
        if i >= pack_size() && i <= inven_feet() {
            continue;
        }

        /* don't crush the swap weapon */
        if is_swap(st, i) {
            continue;
        }

        /* Skip "good" unknown items (unless "icky") */
        if !item.ident && borg_item_worth_id(st, &item) {
            continue;
        }

        /* Do not crush Boots, they could be SPEED */
        if tval == TV_BOOTS && !item.ident {
            continue;
        }

        /* Skip artifacts */
        if randarts() && item.art_idx != 0 && !item.ident {
            continue;
        }
        if borg_item_note_needs_id(st, &item) {
            continue;
        }

        /* Don't crush weapons if we are wielding a digger */
        if weapon_while_digging(st, &item) {
            continue;
        }

        /* Don't crush it if it is our only source of light */
        if tval == TV_ROD
            && item.aware
            && (sval == st.item_val.sv_rod_light && st.borg.tr(BI_LIGHT) <= 0)
        {
            continue;
        }

        /* Rods of healing are too hard to come by */
        if tval == TV_ROD && item.aware && sval == st.item_val.sv_rod_healing {
            continue;
        }

        /* Destroy one of the items */
        add_iqty(st, i, -1);

        /* Fix later */
        fix = true;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let mut p = borg_power(st);

        /* Restore the item */
        add_iqty(st, i, 1);

        /* (C reads the item through its pointer from here on) */
        let item = st.items.items[i as usize];

        /* Obtain the base price */
        let mut temp = if item.value < 30000 { item.value } else { 30000 };

        /* Ignore very cheap items */
        if temp < greed {
            temp = 0;
        }

        /* Penalize */
        p = p.wrapping_sub(temp);

        /* Obtain the base weight */
        temp = i32::from(item.weight);

        /* Reward */
        p = p.wrapping_add(temp.wrapping_mul(100));

        /* Ignore "bad" swaps */
        if p < b_p {
            continue;
        }

        /* Maintain the "best" */
        b_i = i;
        b_p = p;
    }

    /* Examine the inventory */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* Destroy "useless" things */
    if (b_i >= 0) && (b_p >= st.borg.power) {
        /* Message */
        let gain = i64::from(b_p) - i64::from(st.borg.power);
        let value = st.items.items[b_i as usize].value;
        borg_note(st, &format!("# Junking {} power (slow) value {}", gain, value));

        /* Attempt to consume it */
        if borg_consume(st, b_i) {
            return true;
        }

        /* Message */
        note_desc(st, "# Dropping ", b_i, ".");

        /* inscribe "borg ignore". The borg crushes all items */
        /* on the floor that are inscribed this way */
        borg_keypress(st, key(b'{'));
        slot_keys(st, b_i);
        borg_keypresses(st, b"borg ignore");
        borg_keypress(st, KC_ENTER);

        /* Drop one item */
        borg_keypress(st, key(b'd'));
        slot_keys(st, b_i);
        if st.items.items[b_i as usize].iqty > 1 {
            borg_keypress(st, key(b'1'));
            borg_keypress(st, KC_ENTER);
        }

        return true;
    }
    /* Nothing to destroy */
    false
}

/// Find a complete plain stack that can be dropped while keeping the reserve.
/// Prefer the lightest stack that covers the shortfall, then the heaviest
/// smaller stack. The town planner will recompute after each drop.
fn surplus_quiver_stack(
    quiver: &[borg_item],
    ammo_tval: i32,
    reserve: i32,
    shortfall: i64,
) -> Option<usize> {
    if !matches!(ammo_tval as u32, TV_ARROW | TV_BOLT | TV_SHOT) {
        return None;
    }
    let plain = quiver.iter().enumerate().filter(|(_, item)| {
        i32::from(item.tval) == ammo_tval
            && item.iqty > 0
            && item.weight > 0
            && item.ident
            && !item.desc.contains(&(b'{' as core::ffi::c_char))
            && !item.cursed
            && item.ego_idx == 0
            && item.art_idx == 0
            && item.to_h == 0
            && item.to_d == 0
    });
    let surplus = plain.clone().map(|(_, item)| i32::from(item.iqty)).sum::<i32>() - reserve;
    plain
        .filter(|(_, item)| i32::from(item.iqty) <= surplus)
        .min_by_key(|(_, item)| {
            let weight = i64::from(item.iqty) * i64::from(item.weight);
            (weight < shortfall, weight.abs_diff(shortfall))
        })
        .map(|(slot, _)| slot)
}

pub(crate) fn town_quiver_surplus(st: &BorgState, reserve: i32, shortfall: i64) -> Option<usize> {
    let start = quiver_start() as usize;
    let quiver = st.items.items.get(start..quiver_end() as usize)?;
    surplus_quiver_stack(quiver, st.borg.tr(BI_AMMO_TVAL), reserve, shortfall)
        .map(|slot| start + slot)
}

fn usable_quiver_ammo(item: &borg_item, ammo_tval: i32) -> bool {
    matches!(ammo_tval as u32, TV_SHOT | TV_ARROW | TV_BOLT)
        && i32::from(item.tval) == ammo_tval && item.ident && !item.cursed
        && !item.curses.contains(&true) && item.to_h >= 0 && item.to_d >= 0
}

pub(crate) fn usable_quiver_count(st: &BorgState) -> i32 {
    let ammo_tval = st.borg.tr(BI_AMMO_TVAL);
    st.items.items[quiver_start() as usize..quiver_end() as usize].iter()
        .filter(|item| usable_quiver_ammo(item, ammo_tval))
        .map(|item| i32::from(item.iqty)).sum()
}

fn enchanted_quiver_stack(
    quiver: &[borg_item], ammo_tval: i32, reserve: i32, shortfall: i64,
    need_slot: bool, sizes: (u32, u32),
    protected: impl Fn(usize, u8) -> bool,
) -> Option<usize> {
    let (quiver_slot_size, thrown_quiver_mult) = sizes;
    let ammo = |item: &borg_item| usable_quiver_ammo(item, ammo_tval);
    let surplus = quiver.iter().filter(|item| ammo(item))
        .map(|item| i32::from(item.iqty)).sum::<i32>() - reserve;
    if surplus <= 0 { return None; }

    // The engine rounds up total quiver units across stacks. Dropping one
    // quiver entry alone may release no pack slot (obj-gear.c:pack_slots_used).
    let units: u32 = quiver.iter().map(|item| {
        let multiplier = if matches!(u32::from(item.tval), TV_SHOT | TV_ARROW | TV_BOLT) {
            1
        } else { thrown_quiver_mult };
        u32::from(item.iqty) * multiplier
    }).sum();
    let slots = units.div_ceil(quiver_slot_size);
    quiver.iter().enumerate().filter_map(|(slot, item)| {
        if !ammo(item) || item.iqty == 0 || i32::from(item.iqty) > surplus || item.weight <= 0
            || item.ego_idx != 0 || item.art_idx != 0 || (item.to_h == 0 && item.to_d == 0)
            || item.desc.contains(&(b'{' as core::ffi::c_char)) || protected(slot, item.iqty)
        { return None; }
        let releases_slot = (units - u32::from(item.iqty)).div_ceil(quiver_slot_size) < slots;
        if shortfall <= 0 && (!need_slot || !releases_slot) { return None; }
        let weight = i64::from(item.iqty) * i64::from(item.weight);
        Some(((need_slot && !releases_slot, weight < shortfall,
            crate::borg_ranged::missile_damage_twice(item, 0), item.to_h, weight.abs_diff(shortfall), slot), slot))
    }).min_by_key(|(score, _)| *score).map(|(_, slot)| slot)
}

pub(crate) fn town_enchanted_quiver_surplus(st: &BorgState, shortfall: i64, need_slot: bool) -> Option<usize> {
    use crate::params::{value, Param};
    let reserve = value(Param::town_quiver_trim_reserve);
    let start = quiver_start() as usize;
    let quiver = st.items.items.get(start..quiver_end() as usize)?;
    if !exploration_first() || reserve <= 0 || st.borg.tr(BI_AMMO_POWER) <= 0
        || (shortfall <= 0 && !need_slot) || !quiver.iter().any(|item| item.to_h > 0 || item.to_d > 0)
    { return None; }
    let reserve = reserve.max(value(Param::town_ranger_arrows_max)).max(value(Param::town_ranged_reserve));
    let sizes = engine::z_info();
    let supplies = crate::borg_town_capability::carried_supplies(st);
    enchanted_quiver_stack(quiver, st.borg.tr(BI_AMMO_TVAL), reserve, shortfall, need_slot,
        (u32::from(sizes.quiver_slot_size), u32::from(sizes.thrown_quiver_mult)),
        |slot, copies| supplies.keeps_exploration_supply(start + slot, copies))
        .map(|slot| start + slot)
}

pub(crate) fn town_drop_quiver_stack(st: &mut BorgState, slot: usize, qty: i64) {
    let slot = slot as i32;
    note_desc(st, "# Town headroom dropping ", slot, ".");
    borg_keypress(st, key(b'{'));
    slot_keys(st, slot);
    borg_keypresses(st, b"borg ignore");
    borg_keypress(st, KC_ENTER);
    borg_keypress(st, key(b'd'));
    slot_keys(st, slot);
    if qty > 1 {
        borg_keypress(st, key(b'*'));
        borg_keypress(st, KC_ENTER);
    }
}

/// Examine the quiver and dump any worthless items (`borg_dump_quiver`).
///
/// Borg will scan the quiver slots for items which are cursed or have
/// negative bonuses.  Then shoot those items to get rid of them.
/// He needs to do so when safe.
pub fn borg_dump_quiver(st: &mut BorgState) -> bool {
    let mut b_i: i32 = -1;

    /* Prevents swap loops when hungry or weak. This is a workaround for a
     * deeper bug where the Borg may swap items back and forth due to power
     * calculation issues and ultimately die from hunger.
     */
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 2000 {
        return false;
    }
    if st.borg.antibounce_count > 150 {
        return false;
    }

    /* don't crush stuff unless we are on a floor */
    if off_floor(st) {
        return false;
    }

    /* How many should I carry */
    let max_stack = engine::kb_info()[TV_ARROW as usize].max_stack;
    let class = st.borg.tr(BI_CLASS);
    let mut quiver_capacity = if class == CLASS_RANGER || class == CLASS_WARRIOR {
        (max_stack - 1) * 2
    } else {
        max_stack - 1
    };

    quiver_capacity *= i32::from(engine::z_info().quiver_size);

    /* Scan equip */
    for i in (quiver_start()..quiver_end()).rev() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip if it is not cursed and matches my ammo.  If it is not cursed
         * but does not match my ammo, then it is dumped.
         */
        if !item.cursed && i32::from(item.tval) == st.borg.tr(BI_AMMO_TVAL) {
            /* It has some value */
            if item.to_d > 0 && item.to_h > 0 {
                continue;
            }
            if borg_item_note_needs_id(st, item) {
                continue;
            }

            /* Limit the amount of missiles carried */
            if st.borg.tr(BI_AMISSILES) <= quiver_capacity && item.to_d >= 0 && item.to_h >= 0 {
                continue;
            }
        }

        /* Track a crappy one */
        b_i = i;
    }

    /* No item */
    if b_i >= 0 {
        /* Log */
        note_desc(st, "# Dumping ", b_i, ".  Bad ammo in quiver.");

        /* inscribe it */
        borg_keypress(st, key(b'{'));
        borg_keypress(st, key(b'|'));
        borg_keypress(st, digit(b_i - quiver_start()));
        borg_keypresses(st, b"borg ignore");
        borg_keypress(st, KC_ENTER);

        /* Drop it */
        borg_keypress(st, key(b'd'));
        borg_keypress(st, key(b'|'));
        borg_keypress(st, digit(b_i - quiver_start()));
        if st.items.items[b_i as usize].iqty > 1 {
            borg_keypress(st, key(b'*'));
            borg_keypress(st, KC_ENTER);
        }

        /* Did something */
        antibounce(st);
        return true;
    }

    /* Nope */
    false
}

// Leave a harmful light to direct replacement or decursing. Emptying
// the slot can make light maintenance re-wield the same item.
fn protects_cursed_light(slot: i32, light: i32, item: &borg_item) -> bool {
    slot == light && crate::borg_cursed::harmful_curse(item).is_some()
}

/// Remove useless equipment (`borg_remove_stuff`).
///
/// Look through the inventory for equipment that is reducing power.
///
/// Basically, we evaluate the world both with the current set of
/// equipment, and in the alternate world in which various items
/// are removed, and we take one step towards the world in which
/// we have the most "power".
pub fn borg_remove_stuff(st: &mut BorgState) -> bool {
    let hole = borg_first_empty_inventory_slot(st);

    let mut w_p: i32 = 0;

    let mut b_i: i32 = -1;

    let mut fix = false;

    /* if there was no hole, done */
    if hole == -1 {
        return false;
    }

    /* Prevents swap loops when hungry or weak. This is a workaround for a
     * deeper bug where the Borg may swap items back and forth due to power
     * calculation issues and ultimately die from hunger.
     */
    if st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0 {
        return false;
    }

    /* Forbid if been sitting on level forever */
    /*    Just come back and work through the loop later */
    if borg_timer(st, st.borg.time.level) > 2000 {
        return false;
    }
    if st.borg.antibounce_count > 150 {
        return false;
    }

    /* Start with good power */
    let b_p = st.borg.power;

    let light = crate::borg_light::inven_light();

    /* Scan equip */
    for i in inven_wield()..inven_total() {
        let item = &st.items.items[i as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Require aware */
        if !item.aware {
            continue;
        }

        /* Require "known" (or needs id) unless we know it is cursed */
        if borg_item_note_needs_id(st, item) && !item.cursed {
            continue;
        }

        /* skip it if it has not been decursed */
        if item.one_ring {
            continue;
        }

        if protects_cursed_light(i, light, item) {
            continue;
        }

        // borg-reborn-rs (#1361, `wear_digger_keeps_weapon`): bare hands
        // rank above a wielded digger here only because a carried digger
        // earns the tunnelling swap reward, which the wear trial then
        // undoes. A better weapon replaces the digger through the wear
        // paths instead.
        if crate::borg_item_wear::keeps_wielded_digger(i, item) {
            continue;
        }

        /* Take off the item */
        st.items.items[hole as usize] = st.items.items[i as usize];

        /* Erase the item from equip */
        /* note, copy the "hole" onto the item so it doesn't have attributes */
        from_safe(st, i, hole);

        /* Fix later */
        fix = true;

        /* Examine the inventory */
        crate::borg_trait::borg_notice(st, true);

        /* Evaluate the inventory */
        let p = borg_power(st);

        /* Restore the item */
        from_safe(st, i, i);

        /* Restore the hole */
        st.items.items[hole as usize].iqty = 0;

        /* Track the crappy items */
        /* crappy includes things that do not add to power */

        if p >= b_p {
            b_i = i;
            w_p = p;
        }
    }

    /* Restore bonuses */
    if fix {
        crate::borg_trait::borg_notice(st, true);
    }

    /* No item */
    if b_i >= 0 {
        /* Log */
        note_desc(
            st,
            "# Removing ",
            b_i,
            &format!(".  Power with: ({}) Power w/o ({})", b_p, w_p),
        );

        /* Wear it */
        borg_keypress(st, key(b't'));
        borg_keypress(st, letter(b_i - inven_wield()));

        /* Did something */
        antibounce(st);
        return true;
    }

    /* Nope */
    false
}

/// borg-reborn-rs (#1236): compare each spare to the item it replaces.
fn inferior_spare(spare: &borg_item, worn: &borg_item) -> bool {
    let tval = u32::from(spare.tval);
    if !matches!(tval, TV_BOOTS | TV_GLOVES | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK
        | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOW)
        || !spare.aware || spare.iqty == 0 || spare.cursed || spare.art_idx != 0
        || !spare.ident || spare.needs_ident || desc(spare).contains(&b'{')
        || !worn.ident || worn.needs_ident || worn.iqty == 0 || worn.cursed
        || spare.tval != worn.tval || spare.sval != worn.sval
        || spare.weight < worn.weight || spare.activ_idx != 0
        || spare.pval > worn.pval
        || spare.ac > worn.ac || spare.to_a > worn.to_a
        || spare.to_h > worn.to_h || spare.to_d > worn.to_d
        || spare.dd > worn.dd || spare.ds > worn.ds
    {
        return false;
    }

    // A known property, lighter weight or a distinct activation can make a
    // useful swap even when the Borg's overall power falls on wearing it.
    use crate::headers::object_h::EL_INFO_IGNORE;
    spare.flags.iter().zip(&worn.flags).all(|(&s, &w)| s & !w == 0)
        && spare.modifiers.iter().zip(&worn.modifiers).all(|(&s, &w)| s <= w)
        && spare.el_info.iter().zip(&worn.el_info).all(|(s, w)| {
            s.res_level <= w.res_level && u32::from(s.flags & !w.flags) & EL_INFO_IGNORE == 0
        })
        && spare.brands.iter().zip(&worn.brands).all(|(&s, &w)| !s || w)
        && spare.slays.iter().zip(&worn.slays).all(|(&s, &w)| s <= w)
        && (spare.ac < worn.ac || spare.to_a < worn.to_a
            || spare.to_h < worn.to_h || spare.to_d < worn.to_d
            || spare.dd < worn.dd || spare.ds < worn.ds)
}

fn inferior_spares(st: &BorgState) -> impl Iterator<Item = (usize, &borg_item)> {
    use crate::params::{value, Param};
    let pack_end = if value(Param::junk_inferior_spares) == 0 { 0 } else { pack_size() as usize };
    st.items.items.iter().take(pack_end).enumerate().filter(move |&(slot, item)| {
        if item.iqty == 0 || is_swap(st, slot as i32) {
            return false;
        }
        let worn_slot = crate::borg_inventory::borg_wield_slot(item);
        usize::try_from(worn_slot).ok().and_then(|i| st.items.items.get(i))
            .is_some_and(|worn| inferior_spare(item, worn))
    })
}

/// Find a whole inferior stack that meets the caller's disposal constraints.
pub(crate) fn find_inferior_spare<T>(
    st: &mut BorgState,
    mut outlet: impl FnMut(&BorgState, usize, &borg_item) -> Option<T>,
) -> Option<T> {
    if !exploration_first() {
        return inferior_spares(st).find_map(|(slot, item)| outlet(st, slot, item));
    }
    (0..pack_size() as usize).find_map(|slot| {
        if crate::borg_spare_gear::inferior(st, slot) {
            outlet(st, slot, &st.items.items[slot])
        } else {
            None
        }
    })
}

pub(crate) fn borg_drop_inferior_spare(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_CDEPTH) == 0 || st.borg.goal.recalling != 0 || st.borg.goal.fleeing || off_floor(st) {
        return false;
    }
    let cheap_stack = |item: &borg_item| {
        item.iqty > 0 && i64::from(item.value) * i64::from(item.iqty) <= i64::from(EARLY_JUNK_VALUE)
    };
    let safe_to_drop = |st: &mut BorgState| !engine::player_is_shapechanged() && danger_here(st) == 0;
    let light = surplus_light_drop(st);
    let cutoff = gear_alternatives();
    if cutoff > 0 {
        if light.is_none() && !(0..pack_size() as usize).any(|slot| {
            let item = &st.items.items[slot];
            crate::borg_spare_gear::can_rank(item) || crate::borg_town_live::gear_home_backups()
                && (crate::borg_spare_gear::cleanup_slot(st, item).is_some()
                    || crate::borg_spare_gear::unknown_rune_spare(st, slot))
        })
            || !safe_to_drop(st) { return false; }
        let Some(drop) = light.or_else(|| ranked_pack_spare(st, cutoff)) else { return false; };
        if drop.quantity < st.items.items[drop.slot].iqty { return drop_surplus(st, drop); }
        note_desc(st, "# Dropping inferior spare ", drop.slot as i32, ".");
        drop_ignored_pack_stack(st, drop.slot as i32);
        return true;
    }
    let slot = if exploration_first() {
        // Run the danger check only when a stack could qualify. Check it
        // once, before simulating any equipment changes.
        let pack_end = pack_size() as usize;
        let first = (0..pack_end).find(|&slot| {
            let item = &st.items.items[slot];
            item.iqty > 0 && u32::from(item.tval) == TV_BOW && item.ego_idx == 0
                || cheap_stack(item) && crate::borg_spare_gear::equipment_slot(item).is_some()
        });
        first.filter(|_| safe_to_drop(st)).and_then(|first| {
            (first..pack_end).find(|&slot| {
                let item = &st.items.items[slot];
                item.ego_idx == 0 && crate::borg_spare_gear::redundant_carried_launcher(st, slot)
                    || cheap_stack(item) && crate::borg_spare_gear::inferior(st, slot)
            })
        })
    } else {
        let legacy = inferior_spares(st).find_map(|(slot, item)| cheap_stack(item).then_some(slot));
        legacy.filter(|_| safe_to_drop(st))
    };
    let Some(slot) = slot else { return false; };
    note_desc(st, "# Dropping inferior spare ", slot as i32, ".");
    drop_ignored_pack_stack(st, slot as i32);
    true
}

pub(crate) fn ranked_pack_spare(st: &mut BorgState, cutoff: usize) -> Option<SurplusDrop> {
    if let Some(drop) = surplus_light_drop(st) { return Some(drop); }
    if crate::borg_town_live::gear_home_backups() {
        return planned_pack_excess(st, false).or_else(|| unplanned_pack_excess(st, false))
            .or_else(|| crate::borg_spare_gear::unknown_rune_excess(st, false));
    }
    (0..pack_size() as usize).find_map(|slot| {
        let quantity = crate::borg_spare_gear::redundant_copies(st, crate::borg_spare_gear::GearLocation::Pack(slot), cutoff);
        (quantity > 0).then_some(SurplusDrop { slot, quantity, sell_first: true })
    })
}

pub(crate) fn unplanned_pack_excess(st: &mut BorgState, whole_stack: bool) -> Option<SurplusDrop> {
    use crate::borg_spare_gear::{self as spare_gear, GearLocation};
    let cutoff = unplanned_gear_cutoff();
    (0..pack_size() as usize).find_map(|slot| {
        let item = &st.items.items[slot];
        if !spare_gear::can_rank(item) || spare_gear::planned_slot(item).is_some() { return None; }
        let total = item.iqty;
        let quantity = spare_gear::redundant_copies(st, GearLocation::Pack(slot), cutoff);
        (quantity > 0 && (!whole_stack || quantity == total))
            .then_some(SurplusDrop { slot, quantity, sell_first: true })
    })
}

pub(crate) fn surplus_light_drop(st: &BorgState) -> Option<SurplusDrop> {
    let slot = crate::borg_light::surplus_light(st)?;
    Some(SurplusDrop { slot, quantity: st.items.items[slot].iqty, sell_first: true })
}

fn planned_pack_excess(st: &mut BorgState, whole_stack: bool) -> Option<SurplusDrop> {
    use crate::borg_spare_gear::{cleanup_slot, gear_plan, GearLocation};
    let groups: std::collections::BTreeSet<_> = st.items.items[..pack_size() as usize]
        .iter().filter_map(|item| cleanup_slot(st, item)).collect();
    for group in groups {
        // #1307: the plan already keeps the planned upgrade and the worn
        // piece. Release what ranks below both even when the wear routine
        // cannot make the upgrade on this level.
        let plan = gear_plan(st, group, true);
        let pending = plan.upgrade.is_some();
        for entry in plan.stock {
            let GearLocation::Pack(slot) = entry.source else { continue; };
            if pending && entry.beats_worn { continue; }
            if entry.release > 0 && (!whole_stack || entry.release == st.items.items[slot].iqty) {
                return Some(SurplusDrop { slot, quantity: entry.release, sell_first: true });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_item_h::{BORG_CURSE_AIR_SWING, BORG_CURSE_PARALYSIS};
    use crate::borg_item::blank_item;
    use crate::borg_light::light_slot;

    fn sandals(to_a: i16, description: &[u8]) -> borg_item {
        let mut item = blank_item();
        item.tval = TV_BOOTS as u8;
        item.sval = 1;
        item.aware = true;
        item.iqty = 1;
        item.ident = !description.ends_with(b"{??}");
        item.needs_ident = !item.ident;
        item.ac = 1;
        item.to_a = to_a;
        item.weight = 15;
        for (dest, &src) in item.desc.iter_mut().zip(description) {
            *dest = src as core::ffi::c_char;
        }
        item
    }

    /// borg-reborn-rs (#1397, review of #1398): a drop stays under the Borg
    /// only on open floor with no trap, glyph or pile; a shop entrance,
    /// stairs, a trap, a glyph (`glyph`, not `trap`, in the Borg's map) or a
    /// pile makes the engine scatter it.
    #[test]
    fn drop_lands_only_on_clear_trap_free_floor() {
        use crate::headers::list_terrain_h::{FEAT_HOME, FEAT_LESS, FEAT_MORE, FEAT_STORE_ALCHEMY};
        assert!(drop_lands_here(FEAT_FLOOR, false, false, false));
        assert!(!drop_lands_here(FEAT_FLOOR, true, false, false), "a disarmable trap");
        assert!(!drop_lands_here(FEAT_FLOOR, false, true, false), "a glyph of warding");
        assert!(!drop_lands_here(FEAT_FLOOR, false, false, true), "a visible pile");
        for feat in [FEAT_STORE_ALCHEMY, FEAT_HOME, FEAT_LESS, FEAT_MORE] {
            assert!(!drop_lands_here(feat, false, false, false), "feature {feat}");
        }
    }

    #[test]
    fn only_identified_inferior_spares_are_disposable() {
        use crate::headers::list_elements_h::ELEM_NEXUS;
        use crate::headers::list_object_modifiers_h::OBJ_MOD_SPEED;

        let mut worn = sandals(7, b"a Pair of Leather Sandals of Slow Descent [1,+7]");
        worn.ego_idx = 1;
        let recorded = sandals(0, b"a Pair of Leather Sandals [1,+0] {??}");
        let identified = sandals(0, b"a Pair of Leather Sandals [1,+0]");
        let possible_ego = sandals(2, b"a Pair of Leather Sandals [1,+2] {??}");
        let mut resist_swap = sandals(3, b"a Pair of Leather Sandals of Stability [1,+3]");
        resist_swap.el_info[ELEM_NEXUS as usize].res_level = 1;
        let mut speed_swap = sandals(2, b"a Pair of Leather Sandals of Speed [1,+2]");
        speed_swap.modifiers[OBJ_MOD_SPEED as usize] = 3;
        let upgrade = sandals(8, b"a Pair of Leather Sandals [1,+8]");
        let pack = [recorded, identified, possible_ego, resist_swap, speed_swap, upgrade];

        let discarded: Vec<_> = pack.iter().enumerate()
            .filter_map(|(slot, item)| inferior_spare(item, &worn).then_some(slot))
            .collect();
        assert_eq!(discarded, [1]);
    }

    #[test]
    fn different_kinds_and_inscribed_spares_are_kept() {
        let worn = sandals(7, b"a Pair of Leather Sandals [1,+7]");
        let different_kind = borg_item { sval: 2, ..sandals(0, b"a Pair of Soft Leather Boots [2,+0]") };
        let inscribed = sandals(0, b"a Pair of Leather Sandals [1,+0] {keep}");
        assert!(!inferior_spare(&different_kind, &worn));
        assert!(!inferior_spare(&inscribed, &worn));
    }

    #[test]
    fn town_quiver_trim_protects_the_reserve_and_special_ammunition() {
        let mut plain = blank_item();
        plain.tval = TV_SHOT as u8;
        plain.ident = true;
        plain.iqty = 40;
        plain.weight = 4;
        let mut smaller = plain;
        smaller.iqty = 12;
        let mut enchanted = plain;
        enchanted.to_d = 1;
        let mut unknown = plain;
        unknown.ident = false;
        let mut inscribed = plain;
        inscribed.desc[0] = b'{' as core::ffi::c_char;
        let mut cursed = plain;
        cursed.cursed = true;
        let mut ego = plain;
        ego.ego_idx = 1;
        let mut artifact = plain;
        artifact.art_idx = 1;
        let mut wrong_type = plain;
        wrong_type.tval = TV_ARROW as u8;
        let mut quiver = [enchanted, plain, smaller, unknown, wrong_type, inscribed, cursed, ego, artifact];
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 40, 20), Some(2));
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 40, 90), Some(2));
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 41, 20), None);
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 12, 90), Some(1));
        quiver[2] = blank_item();
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 40, 90), None);
    }

    #[test]
    fn enchanted_quiver_trim_opens_a_pack_slot_and_keeps_the_reserve() {
        let shot = |iqty, to_h, to_d, ego_idx| borg_item {
            tval: TV_SHOT as u8, ident: true, iqty, weight: 4, dd: 1, ds: 2,
            to_h, to_d, ego_idx, ..blank_item()
        };
        let quiver = [shot(14, 3, 2, 0), shot(25, 2, 3, 0), shot(1, 5, 2, 0), shot(19, 1, 3, 1)];
        let choose = |reserve, shortfall, need_slot| enchanted_quiver_stack(&quiver,
            TV_SHOT as i32, reserve, shortfall, need_slot, (40, 5), |_, _| false);
        assert_eq!(choose(30, 0, false), None);
        assert_eq!(choose(30, 0, true), Some(1));
        assert_eq!(choose(35, 0, true), None);
        assert_eq!(choose(30, 50, false), Some(0));
        assert_eq!(choose(59, 50, false), None);
        let inscribed = borg_item { desc: [b'{' as core::ffi::c_char; 80], ..quiver[1] };
        let mut protected = quiver;
        protected[1] = inscribed;
        assert_eq!(enchanted_quiver_stack(&protected, TV_SHOT as i32, 30, 0, true, (40, 5), |_, _| false), None);
    }

    #[test]
    fn the_recorded_quiver_keeps_forty_plain_shots_and_all_enchanted_shots() {
        let mut quiver = [(22, 3, 1), (40, 0, 0), (40, 0, 0), (40, 0, 0),
            (40, 2, 2), (18, 1, 6), (10, 2, 3), (40, 0, 0), (40, 0, 0), (10, 0, 0)]
            .map(|(count, to_h, to_d)| {
                let mut stack = blank_item();
                stack.tval = TV_SHOT as u8;
                stack.ident = true;
                stack.iqty = count;
                stack.weight = 4;
                stack.to_h = to_h;
                stack.to_d = to_d;
                stack
            });
        let mut shortfall = 1362;
        for expected in [40, 40, 40, 40, 10] {
            let slot = surplus_quiver_stack(&quiver, TV_SHOT as i32, 40, shortfall).unwrap();
            assert_eq!(quiver[slot].iqty, expected);
            shortfall -= i64::from(expected) * 4;
            quiver[slot] = blank_item();
        }
        assert_eq!(surplus_quiver_stack(&quiver, TV_SHOT as i32, 40, shortfall), None);
        assert_eq!(quiver.iter().map(|stack| u16::from(stack.iqty)).sum::<u16>(), 130);
    }

    #[test]
    fn the_recorded_priest_potions_go_before_other_pack_contents() {
        use crate::headers::list_effects_h::{EF_CURE, EF_TIMED_INC};
        use crate::headers::list_player_timed_h::{TMD_BLIND, TMD_CONFUSED, TMD_PARALYZED, TMD_SLOW};
        let potion = |sval, iqty, known| {
            let mut item = blank_item();
            item.tval = TV_POTION as u8;
            item.sval = sval;
            item.kind = u32::from(sval);
            item.iqty = iqty;
            item.aware = true;
            item.flavour_aware = known;
            item
        };
        let mut oil = blank_item();
        oil.tval = TV_FLASK as u8;
        oil.iqty = 12;
        let mut torch = blank_item();
        torch.tval = TV_LIGHT as u8;
        torch.iqty = 26;
        let harmful = |item: &borg_item| harmful_item(item, |kind| {
            let (effect, status) = match kind {
                1 => (EF_TIMED_INC, TMD_PARALYZED),
                3 => (EF_TIMED_INC, TMD_CONFUSED),
                5 => (EF_TIMED_INC, TMD_SLOW),
                _ => (EF_CURE, TMD_BLIND),
            };
            vec![(effect as u16, status as i32)]
        });
        for (sleep, confusion) in [(3, 4), (14, 22)] {
            let mut pack = [
                oil, potion(1, 1, false), potion(10, 3, true), potion(11, 2, true),
                potion(1, sleep, true), potion(3, confusion, true), potion(5, 2, true), torch,
            ];
            for (slot, quantity) in [(4, sleep), (5, confusion), (6, 2)] {
                assert_eq!(pack.iter().position(harmful), Some(slot));
                assert_eq!(pack[slot].iqty, quantity);
                pack[slot].iqty = 0;
            }
            assert_eq!(pack.iter().position(harmful), None);
        }
    }

    #[test]
    fn adverse_item_effects_are_distinguished_from_cures_buffs_and_trades() {
        use crate::headers::list_effects_h::*;
        use crate::headers::list_player_timed_h::*;
        use crate::headers::list_projections_h::{PROJ_MAKE_TRAP, PROJ_MON_CONF, PROJ_MON_HEAL, PROJ_MON_SPEED};
        let classify = |tval: u32, effect: u32, subtype: u32| {
            let known = borg_item {
                tval: tval as u8, kind: 1, iqty: 1, aware: true, flavour_aware: true,
                ..blank_item()
            };
            harmful_item(&known, |_| vec![(effect as u16, subtype as i32)])
        };
        assert!(classify(TV_POTION, EF_TIMED_INC, TMD_BLIND));
        assert!(classify(TV_POTION, EF_TIMED_INC, TMD_POISONED));
        assert!(classify(TV_POTION, EF_TIMED_INC_NO_RES, TMD_PARALYZED));
        assert!(classify(TV_POTION, EF_TIMED_INC, TMD_SCRAMBLE));
        assert!(classify(TV_MUSHROOM, EF_DRAIN_STAT, 0));
        assert!(classify(TV_MUSHROOM, EF_TIMED_SET, TMD_TERROR));
        assert!(classify(TV_FOOD, EF_TIMED_INC, TMD_CONFUSED));
        assert!(classify(TV_SCROLL, EF_CURSE_ARMOR, 0));
        assert!(classify(TV_SCROLL, EF_TOUCH, PROJ_MAKE_TRAP));
        assert!(classify(TV_WAND, EF_BOLT_STATUS, PROJ_MON_HEAL));
        assert!(classify(TV_STAFF, EF_SUMMON, 0));
        assert!(classify(TV_STAFF, EF_PROJECT_LOS, PROJ_MON_SPEED));
        assert!(!classify(TV_WAND, EF_BOLT_STATUS, PROJ_MON_CONF));
        assert!(!classify(TV_MUSHROOM, EF_CURE, TMD_POISONED));
        assert!(!classify(TV_POTION, EF_TIMED_INC, TMD_HERO));
        assert!(!classify(TV_POTION, EF_LOSE_RANDOM_STAT, 0));
        assert!(!classify(TV_SCROLL, EF_TELEPORT, 0));
    }

    #[test]
    fn arrival_discards_obviously_inferior_gear_with_unknown_runes() {
        let worn = sandals(2, b"Pair of Leather Sandals [1,+2]");
        let plain = sandals(0, b"Pair of Leather Sandals [1,+0] {??}");
        assert!(expendable_town_spare(&plain, &worn));
        let cheap = borg_item { to_a: 1, value: EARLY_JUNK_VALUE, ..plain };
        assert!(expendable_town_spare(&cheap, &worn));
        let lighter = borg_item { weight: plain.weight - 1, ..plain };
        assert!(!expendable_town_spare(&lighter, &worn));
        let mut special = plain;
        special.modifiers[OBJ_MOD_STR as usize] = 1;
        assert!(!expendable_town_spare(&special, &worn));
        assert!(!expendable_town_spare(&plain, &blank_item()));
        special.curses[BORG_CURSE_PARALYSIS as usize] = true;
        assert!(expendable_town_spare(&special, &worn));
    }

    #[test]
    fn restore_life_junking_counts_known_pack_stacks_and_the_copies_removed() {
        let mut known = blank_item();
        known.tval = TV_POTION as u8;
        known.sval = 16;
        known.aware = true;
        known.iqty = 3;
        let mut unknown = known;
        unknown.aware = false;
        let mut pack = [known, unknown];
        assert!(!protects_restore_life(&pack, 0, 16, 1, false));
        assert!(protects_restore_life(&pack, 0, 16, 1, true));
        assert!(!protects_restore_life(&pack, 0, 16, 0, true));
        pack[0].iqty = 1;
        assert!(protects_restore_life(&pack, 0, 16, 1, false));
        pack[1].aware = true;
        assert!(!protects_restore_life(&pack, 0, 16, 1, true));
    }

    #[test]
    fn cursed_torch_loses_its_pack_slot_to_clean_spares_and_unknown_gear() {
        let mut clean = crate::borg_item::blank_item();
        clean.tval = TV_LIGHT as u8;
        clean.iqty = 6;
        clean.timeout = 5000;
        let mut cursed = clean;
        cursed.iqty = 1;
        cursed.needs_ident = true;
        cursed.cursed = true;
        cursed.curses[BORG_CURSE_PARALYSIS as usize] = true;
        let mut unknown = crate::borg_item::blank_item();
        unknown.tval = TV_SWORD as u8;
        unknown.iqty = 1;
        unknown.needs_ident = true;

        assert_eq!(cursed_spare_slot(&[clean, unknown, cursed], || true), Some(2));
        assert_eq!(cursed_spare_slot(&[unknown, cursed], || false), None);
        // Intrinsic drawbacks set the broad cursed flag during analysis.
        unknown.cursed = true;
        assert_eq!(cursed_spare_slot(&[unknown, cursed], || true), Some(1));
        unknown.curses[BORG_CURSE_AIR_SWING as usize] = true;
        unknown.uncursable = true;
        unknown.art_idx = 1;
        assert_eq!(cursed_spare_slot(&[unknown, cursed], || false), Some(0));
        unknown.one_ring = true;
        let mut consumable = cursed;
        consumable.tval = TV_POTION as u8;
        assert_eq!(cursed_spare_slot(&[unknown, consumable, cursed], || true), Some(2));
    }

    #[test]
    fn ordinary_take_off_skips_only_a_harmful_curse_in_the_light_slot() {
        let light = light_slot(23) as i32;
        let mut item = blank_item();
        item.iqty = 1;
        assert!(!protects_cursed_light(light, light, &item));
        item.cursed = true;
        item.curses[BORG_CURSE_AIR_SWING as usize] = true;
        assert!(!protects_cursed_light(light, light, &item));
        item.curses[BORG_CURSE_PARALYSIS as usize] = true;
        assert!(protects_cursed_light(light, light, &item));
        assert!(!protects_cursed_light(light + 1, light, &item));
    }
}

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
    BI_AMISSILES, BI_AMMO_TVAL, BI_ASTFDEST, BI_ATELEPORTLVL, BI_ATPORTOTHER, BI_CCON, BI_CDEPTH,
    BI_CDEX, BI_CINT, BI_CLASS, BI_CLEVEL, BI_CSTR, BI_CURHP, BI_CWIS, BI_FOOD, BI_GOLD,
    BI_GOOD_S_CHG, BI_ISFIXEXP, BI_ISFULL, BI_ISHUNGRY, BI_ISWEAK, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH,
    BI_MAXHP, BI_MAXSP, BI_QUIVER_SLOTS, BI_SPEED,
};
use crate::headers::list_options_h::OPT_birth_randarts;
use crate::headers::list_stats_h::{OBJ_MOD_CON, OBJ_MOD_DEX, OBJ_MOD_INT, OBJ_MOD_STR, OBJ_MOD_WIS};
use crate::headers::list_terrain_h::FEAT_FLOOR;
use crate::headers::list_tvals_h::*;
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `CLASS_WARRIOR` .. `CLASS_RANGER` (player-calcs.h class indexes)
const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_RANGER: i32 = 7;

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
fn is_swap(st: &BorgState, i: i32) -> bool {
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
    if t == 0 {
        return false;
    }

    /* double check that there is an object here.  This is simulating doing */
    /* "look" so it is okay */
    if engine::top_floor_object(&st.access, x, y).is_none() {
        return false;
    }

    let take = st.takes.takes[usize::from(t)];
    if take.value != -10 {
        return false;
    }

    let base = if take.kidx != 0 { engine::kind_base_name(take.kidx as usize) } else { None };
    if let Some(base) = base {
        let s = [b"# Destroying floor item ", base, engine::kind_name(take.kidx as usize)].concat();
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
    let sv = st.item_val.sv_scroll_deep_descent;
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
    note_desc(st, "# Discarding ", i, " (Deep Descent is never read)");
    borg_keypress(st, key(b'k'));
    slot_keys(st, i);
    // The ignore menu: a) This item only, b) All Scrolls of Deep Descent.
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
                v.sv_scroll_light,
                v.sv_scroll_monster_confusion,
                v.sv_scroll_trap_door_destruction,
                v.sv_scroll_satisfy_hunger,
                v.sv_scroll_dispel_undead,
                v.sv_scroll_blessing,
                v.sv_scroll_holy_chant,
                v.sv_scroll_holy_prayer,
            ];

            /* Check the scroll */
            if scrolls.contains(&sval) {
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

    /* Destroy actual "junk" items */
    for i in 0..pack_size() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);

        /* Skip empty items */
        if item.iqty == 0 || crate::borg_light::keep_fuel_stack(st, i as usize)
            || crate::borg_ranged::keep_stack(st, i as usize)
            || keeps_restore_life(st, i as usize, false, false) {
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

    /* Scan the inventory */
    for i in 0..pack_size() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);

        let mut value_boost: i32 = 0;

        /* Skip empty items. A desperate pack clear can spend the reserve. */
        if item.iqty == 0 || !desperate && (crate::borg_light::keep_fuel_stack(st, i as usize)
            || crate::borg_ranged::keep_stack(st, i as usize))
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

    /* Scan for junk */
    for i in 0..quiver_end() {
        let item = st.items.items[i as usize];
        let tval = u32::from(item.tval);
        let sval = i32::from(item.sval);

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
fn inferior_spare(
    spare: &borg_item,
    worn: &borg_item,
    sandal_sval: i32,
    max_depth: i32,
    sandal_ac_margin: i32,
) -> bool {
    let tval = u32::from(spare.tval);
    if !matches!(tval, TV_BOOTS | TV_GLOVES | TV_HELM | TV_CROWN | TV_SHIELD | TV_CLOAK
        | TV_SOFT_ARMOR | TV_HARD_ARMOR | TV_HAFTED | TV_POLEARM | TV_SWORD | TV_BOW)
        || !spare.aware || spare.iqty == 0 || spare.cursed || spare.art_idx != 0
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

    let description = desc(spare);
    if spare.ident && !spare.needs_ident {
        if description.contains(&b'{') {
            return false;
        }
    } else {
        // obj-desc.c:obj_desc_combat displays +0 only after assessment and
        // learning the AC rune. Leather Sandals start at [1,+0] (object.txt).
        // Their egos add no flat AC penalty (ego_item.txt), and obj-make.c:
        // apply_magic/armour adds at least +1 when creating ego armour.
        // This is a shallow-game heuristic. Elemental inventory damage can
        // lower the enchantment of carried armor (project-obj.c:inven_damage),
        // and disenchantment can affect a pair worn earlier. Keep unknowns
        // once the Borg has reached DL5 to limit the heuristic's exposure.
        if tval != TV_BOOTS || i32::from(spare.sval) != sandal_sval || max_depth >= 5
            || spare.ego_idx != 0 || spare.ac != 1 || spare.to_a != 0
            || !description.ends_with(b" [1,+0] {??}")
            || sandal_ac_margin <= 0 || i32::from(worn.to_a) < sandal_ac_margin
        {
            return false;
        }
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

pub(crate) fn inferior_spares(st: &BorgState) -> impl Iterator<Item = (usize, &borg_item)> {
    use crate::params::{value, Param};
    let max_depth = st.borg.tr(BI_MAXDEPTH);
    let pack_end = if value(Param::junk_inferior_spares) == 0 { 0 } else { pack_size() as usize };
    let sandal_ac_margin = value(Param::junk_inferior_sandal_ac_margin);
    let sandal_sval = if pack_end > 0 && sandal_ac_margin > 0 && max_depth < 5 {
        engine::lookup_sval(TV_BOOTS as i32, "Pair of Leather Sandals")
    } else {
        -1
    };
    st.items.items.iter().take(pack_end).enumerate().filter(move |&(slot, item)| {
        if item.iqty == 0 || is_swap(st, slot as i32) {
            return false;
        }
        let worn_slot = crate::borg_inventory::borg_wield_slot(item);
        usize::try_from(worn_slot).ok().and_then(|i| st.items.items.get(i))
            .is_some_and(|worn| inferior_spare(item, worn, sandal_sval, max_depth, sandal_ac_margin))
    })
}

pub(crate) fn borg_drop_inferior_spare(st: &mut BorgState) -> bool {
    if st.borg.tr(BI_CDEPTH) == 0 || st.borg.goal.recalling != 0 || st.borg.goal.fleeing || off_floor(st) {
        return false;
    }
    let Some((slot, _)) = inferior_spares(st).find(|(_, item)| {
        i64::from(item.value) * i64::from(item.iqty) <= i64::from(EARLY_JUNK_VALUE)
    }) else {
        return false;
    };
    if engine::player_is_shapechanged() || danger_here(st) != 0 {
        return false;
    }
    note_desc(st, "# Dropping inferior spare ", slot as i32, ".");
    drop_ignored_pack_stack(st, slot as i32);
    true
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

    #[test]
    fn recorded_sandals_free_one_unknown_slot_and_keep_useful_spares() {
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

        let discarded = |margin| pack.iter().enumerate()
            .filter_map(|(slot, item)| inferior_spare(item, &worn, 1, 3, margin).then_some(slot))
            .collect::<Vec<_>>();
        assert_eq!(discarded(0), [1], "unknown policy disabled: one slot freed");
        assert_eq!(discarded(7), [0, 1], "two slots freed, all four upgrade or swap options retained");
    }

    #[test]
    fn unsafe_inferences_about_a_zero_ac_spare_keep_the_item() {
        let worn = sandals(7, b"a Pair of Leather Sandals of Slow Descent [1,+7]");
        let spare = sandals(0, b"a Pair of Leather Sandals [1,+0] {??}");
        assert!(!inferior_spare(&spare, &worn, 1, 5, 7), "unknown-rune heuristic ends at max DL5");
        assert!(!inferior_spare(&spare, &worn, 1, 3, 8), "insufficient enchantment margin");

        let hidden_ac = sandals(0, b"a Pair of Leather Sandals [1] {??}");
        let reserved = sandals(0, b"a Pair of Leather Sandals [1,+0] {keep, ??}");
        let possible_ego = borg_item { ego_idx: 1, ..spare };
        let artifact = borg_item { art_idx: 1, ..spare };
        let lighter = borg_item { weight: 10, ..spare };
        for keep in [hidden_ac, reserved, possible_ego, artifact, lighter] {
            assert!(!inferior_spare(&keep, &worn, 1, 3, 7));
        }
        assert!(!inferior_spare(&spare, &blank_item(), 1, 3, 7));
        assert!(!inferior_spare(&spare, &worn, 2, 3, 7), "different kind");
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

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
use crate::borg_item_use::{borg_eat, borg_quaff_potion, borg_read_scroll};
use crate::borg_power::borg_power;
use crate::engine;
use crate::borg_headers::borg_h::{BORG_MONEY_SCUM_AMOUNT, BORG_WORSHIPS_GOLD};
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_trait_h::{
    BI_AMISSILES, BI_AMMO_TVAL, BI_ASTFDEST, BI_ATELEPORTLVL, BI_ATPORTOTHER, BI_CCON, BI_CDEPTH,
    BI_CDEX, BI_CINT, BI_CLASS, BI_CLEVEL, BI_CSTR, BI_CURHP, BI_CWIS, BI_FOOD, BI_GOLD,
    BI_GOOD_S_CHG, BI_ISFULL, BI_ISHUNGRY, BI_ISWEAK, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH,
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
fn is_swap(st: &mut BorgState, i: i32) -> bool {
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
            /* Check the potion */
            if [
                v.sv_potion_slime_mold,
                v.sv_potion_cure_light,
                v.sv_potion_cure_serious,
                v.sv_potion_cure_critical,
                v.sv_potion_healing,
                v.sv_potion_star_healing,
                v.sv_potion_life,
                v.sv_potion_restore_life,
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
        (10, 15),
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
            || is(item, TV_SCROLL, v.sv_scroll_protection_from_evil))
}

/// Drop "junk" items, marking them for destruction (`borg_drop_junk`).
pub fn borg_drop_junk(st: &mut BorgState) -> bool {
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
        if item.iqty == 0 {
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

        /* Skip empty items */
        if item.iqty == 0 {
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

        /* Drop that item */
        /* inscribe "borg ignore". The borg crushes all items */
        /* on the floor that are inscribed this way */
        borg_keypress(st, key(b'{'));
        borg_keypress(st, letter(b_i));
        borg_keypresses(st, b"borg ignore");
        borg_keypress(st, KC_ENTER);

        /* drop it */
        borg_keypress(st, key(b'd'));
        borg_keypress(st, letter(b_i));
        if st.items.items[b_i as usize].iqty > 1 {
            borg_keypress(st, key(b'*'));
            borg_keypress(st, KC_ENTER);
        }

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

    /* Not if in munchkin mode */
    if st.borg.munchkin_mode {
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
        if item.iqty == 0 {
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

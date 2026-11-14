//! Port of `borg-recover.c`: attempt to recover from damage and such after
//! a battle.
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

use crate::borg_danger::borg_danger;
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note, KC_ENTER};
use crate::borg_item_use::{
    borg_activate_item, borg_eat, borg_quaff_crit, borg_quaff_potion, borg_read_scroll,
    borg_use_staff_fail, borg_zap_rod,
};
use crate::borg_light::{borg_check_light_only, borg_maintain_light, BorgNeed};
use crate::borg_magic::{borg_spell, borg_spell_okay};
use crate::borg_flow_misc;
use crate::engine;
use crate::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, CURE_POISON, HEALING, HERBAL_CURING, HEROISM, HOLY_WORD, MINOR_HEALING,
    RECHARGING, REMEMBRANCE, REMOVE_HUNGER, RESTORATION, REVITALIZE, UNHOLY_REPRIEVE,
};
use crate::borg_headers::borg_trait_h::{
    BI_CDEPTH, BI_CLEVEL, BI_CRSFEAR, BI_CURHP, BI_CURSP, BI_FOOD, BI_ISAFRAID, BI_ISBLIND,
    BI_ISCONFUSED, BI_ISCUT, BI_ISFIXALL, BI_ISFIXCON, BI_ISFIXDEX, BI_ISFIXEXP, BI_ISFIXINT,
    BI_ISFIXSTR, BI_ISFIXWIS, BI_ISHEAVYSTUN, BI_ISHUNGRY, BI_ISIMAGE, BI_ISPOISONED, BI_ISSTUN,
    BI_ISWEAK, BI_MAXHP, BI_MAXSP,
};
use crate::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use crate::headers::list_tvals_h::{TV_FOOD, TV_MUSHROOM, TV_ROD};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `GOAL_RECOVER` (borg-flow.h): the goal of flowing somewhere safe to rest.
const GOAL_RECOVER: i16 = 9;

/// A literal key (`borg_keypress('R')` etc.).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// The rest keys `R`, `c`, Enter (`c` is `'&'` or `'*'`).
fn rest(st: &mut BorgState, c: u8) {
    borg_keypress(st, key(b'R'));
    borg_keypress(st, key(c));
    borg_keypress(st, KC_ENTER);
}

/// `borg_items[borg_slot(TV_ROD, sval)].pval == 0`: the rod is charging.
///
/// Like C, this assumes the slot exists (the caller has checked
/// `borg.has[kind]`); a missing rod would index `borg_items[-1]` in C and
/// panics here.
fn rod_charging(st: &mut BorgState, sval: i32) -> bool {
    let slot = borg_slot(st, TV_ROD as i32, sval);
    st.items.items[slot as usize].pval == 0
}

/// Attempt to recover from damage and such after a battle.
///
/// Note that resting while in danger is counter-productive, unless
/// the danger is low, in which case it may induce "farming".
///
/// Note that resting while recall is active will often cause you
/// to lose hard-won treasure from nasty monsters, so we disable
/// resting when waiting for recall in the dungeon near objects.
///
/// First we try spells/prayers, which are "free", and then we
/// try food, potions, scrolls, staffs, rods, artifacts, etc.
///
/// XXX XXX XXX
/// Currently, we use healing spells as if they were "free", but really,
/// this is only true if the "danger" is less than the "reward" of doing
/// the healing spell, and if there are no monsters which might soon step
/// around the corner and attack.
pub fn borg_recover(st: &mut BorgState) -> bool {
    /*** Handle annoying situations ***/
    let need = borg_maintain_light(st);
    if need == BorgNeed::MetNeed {
        return true;
    } else if need == BorgNeed::UnmetNeed {
        borg_note(st, "# Need to refuel but cant!");
    }

    /*** Do not recover when in danger ***/

    // Look around for danger
    let p = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);

    // Never recover in dangerous situations
    if p > i32::from(st.borg.avoidance) / 4 {
        return false;
    }

    /*** Roll for "paranoia" ***/

    // Base roll
    let mut q = engine::randint0(100);

    // Half dead
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2 {
        q -= 10;
    }

    // Almost dead
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 4 {
        q -= 10;
    }

    /*** Use "cheap" cures ***/

    // Cure stun
    if st.borg.tr(BI_ISSTUN) != 0
        && q < 75
        && (borg_activate_item(st, st.act.cure_body)
            || borg_activate_item(st, st.act.cure_critical)
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_temp)
            || borg_activate_item(st, st.act.heal3)
            || borg_spell(st, MINOR_HEALING)
            || borg_spell(st, HEALING)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, HOLY_WORD))
    {
        borg_note(st, &format!("# Cure Stun - danger {p}"));
        return true;
    }

    // Cure stun
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_activate_item(st, st.act.cure_body)
            || borg_activate_item(st, st.act.cure_critical)
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_temp)
            || borg_activate_item(st, st.act.heal3)
            || borg_spell(st, MINOR_HEALING)
            || borg_spell(st, HEALING)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, HOLY_WORD))
    {
        borg_note(st, &format!("# Cure Heavy Stun - danger {p}"));
        return true;
    }

    // Cure cuts
    if st.borg.tr(BI_ISCUT) != 0
        && q < 75
        && (borg_activate_item(st, st.act.cure_light)
            || borg_spell(st, MINOR_HEALING)
            || borg_spell(st, HEALING)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, HOLY_WORD))
    {
        borg_note(st, &format!("# Cure Cuts - danger {p}"));
        return true;
    }

    // Cure poison
    if st.borg.tr(BI_ISPOISONED) != 0
        && q < 75
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, CURE_POISON))
    {
        borg_note(st, &format!("# Cure poison - danger {p}"));
        return true;
    }

    // Cure fear
    if st.borg.tr(BI_ISAFRAID) != 0
        && st.borg.tr(BI_CRSFEAR) == 0
        && q < 75
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_spell(st, HEROISM)
            || borg_spell(st, BERSERK_STRENGTH)
            || borg_spell(st, HOLY_WORD))
    {
        borg_note(st, &format!("# Cure fear - danger {p}"));
        return true;
    }

    // Satisfy hunger
    if (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0)
        && q < 75
        && (borg_spell(st, REMOVE_HUNGER) || borg_spell(st, HERBAL_CURING))
    {
        return true;
    }

    // Hallucination
    if st.borg.tr(BI_ISIMAGE) != 0
        && q < 75
        && borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
    {
        return true;
    }

    // Heal damage
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2
        && q < 75
        && p == 0
        && st.borg.tr(BI_CURSP) > st.borg.tr(BI_MAXSP) / 4
        && (borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_activate_item(st, st.act.heal3)
            || borg_spell(st, HEALING)
            || borg_spell(st, HOLY_WORD)
            || borg_spell(st, MINOR_HEALING)
            || borg_spell(st, HEROISM))
    {
        borg_note(st, "# heal damage (recovering)");
        return true;
    }

    // cure experience loss with prayer
    if st.borg.tr(BI_ISFIXEXP) != 0
        && (borg_activate_item(st, st.act.restore_exp)
            || borg_activate_item(st, st.act.restore_st_lev)
            || borg_activate_item(st, st.act.restore_life)
            || borg_spell(st, REVITALIZE)
            || borg_spell(st, REMEMBRANCE)
            || (st.borg.tr(BI_CURHP) > 90 && borg_spell(st, UNHOLY_REPRIEVE)))
    {
        return true;
    }

    // cure stat drain with prayer
    if (st.borg.tr(BI_ISFIXSTR) != 0
        || st.borg.tr(BI_ISFIXINT) != 0
        || st.borg.tr(BI_ISFIXWIS) != 0
        || st.borg.tr(BI_ISFIXDEX) != 0
        || st.borg.tr(BI_ISFIXCON) != 0
        || st.borg.tr(BI_ISFIXALL) != 0)
        && (borg_spell(st, RESTORATION) || borg_spell(st, REVITALIZE))
    {
        return true;
    }

    // cure stat drain with prayer
    if (st.borg.tr(BI_ISFIXSTR) != 0 || st.borg.tr(BI_ISFIXINT) != 0 || st.borg.tr(BI_ISFIXCON) != 0)
        && st.borg.tr(BI_CURHP) > 90
        && borg_spell(st, UNHOLY_REPRIEVE)
    {
        return true;
    }

    /*** Use "expensive" cures ***/

    // Cure stun
    if st.borg.tr(BI_ISSTUN) != 0
        && q < 25
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_quaff_crit(st, false))
    {
        return true;
    }

    // Cure heavy stun
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        && q < 95
        && (borg_quaff_crit(st, true)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2))
    {
        return true;
    }

    // Cure cuts
    if st.borg.tr(BI_ISCUT) != 0
        && q < 25
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_quaff_crit(st, st.borg.tr(BI_CURHP) < 10))
    {
        return true;
    }

    // Cure poison
    if st.borg.tr(BI_ISPOISONED) != 0
        && q < 25
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_poison)
            || borg_eat(st, TV_FOOD as i32, st.item_val.sv_food_waybread)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_crit(st, st.borg.tr(BI_CURHP) < 10)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_activate_item(st, st.act.food_waybread))
    {
        return true;
    }

    // Cure blindness
    if st.borg.tr(BI_ISBLIND) != 0
        && q < 25
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_eat(st, TV_FOOD as i32, st.item_val.sv_food_waybread)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_crit(st, false)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_activate_item(st, st.act.food_waybread))
    {
        return true;
    }

    // Cure confusion
    if st.borg.tr(BI_ISCONFUSED) != 0
        && q < 25
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_crit(st, false)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_activate_item(st, st.act.cure_confusion)
            || borg_zap_rod(st, st.item_val.sv_rod_curing))
    {
        return true;
    }

    // Cure fear
    if st.borg.tr(BI_ISAFRAID) != 0
        && st.borg.tr(BI_CRSFEAR) == 0
        && q < 25
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_quaff_potion(st, st.item_val.sv_potion_boldness)
            || borg_quaff_potion(st, st.item_val.sv_potion_heroism)
            || borg_quaff_potion(st, st.item_val.sv_potion_berserk)
            || borg_activate_item(st, st.act.rem_fear_pois))
    {
        return true;
    }

    // Satisfy hunger
    if (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0)
        && q < 25
        && (borg_read_scroll(st, st.item_val.sv_scroll_satisfy_hunger)
            || borg_activate_item(st, st.act.satisfy))
    {
        return true;
    }

    // Heal damage
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / 2
        && q < 25
        && (borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_crit(st, false)
            || borg_activate_item(st, st.act.cure_serious))
    {
        return true;
    }

    // Rest to recharge Rods of Healing or Recall
    let kv_recall = st.item_val.kv_rod_recall as usize;
    let kv_healing = st.item_val.kv_rod_healing as usize;
    if (st.borg.has[kv_recall] != 0 || st.borg.has[kv_healing] != 0)
        // Step 1.  Recharge just 1 rod.
        && ((st.borg.has[kv_healing] != 0 && rod_charging(st, st.item_val.sv_rod_healing))
            || (st.borg.has[kv_recall] != 0 && rod_charging(st, st.item_val.sv_rod_recall)))
        // Mages can cast the recharge spell

        // Rest until at least one recharges
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && !borg_spell_okay(st, RECHARGING)
    {
        borg_note(st, "# Resting to recharge a rod...");

        // Reset the Bouncing-borg Timer
        st.borg.antibounce_count = 0;

        // Rest until done
        borg_keypress(st, key(b'R'));
        borg_keypress(st, key(b'1'));
        borg_keypress(st, key(b'0'));
        borg_keypress(st, key(b'0'));
        borg_keypress(st, KC_ENTER);

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    /*** Just Rest ***/

    // Rest until healed
    let sp_tenths = if st.borg.tr(BI_CDEPTH) > 85 { 7 } else { 6 };
    if st.borg.tr(BI_ISBLIND) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && (st.borg.tr(BI_ISCONFUSED) != 0
            || st.borg.tr(BI_ISIMAGE) != 0
            || st.borg.tr(BI_ISAFRAID) != 0
            || st.borg.tr(BI_ISSTUN) != 0
            || st.borg.tr(BI_ISHEAVYSTUN) != 0
            || st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP)
            || st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) * sp_tenths / 10)
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && !st.borg.mon.scary
        && p <= i32::from(
            st.danger.fear_region[(st.borg.c.y / 11) as usize][(st.borg.c.x / 11) as usize],
        )
        && st.borg.goal.type_0 != GOAL_RECOVER
    {
        // check for then call lite in dark room before resting
        if !borg_check_light_only(st) {
            borg_note(st, "# Resting to recover HP/SP...");

            // Rest until done
            rest(st, b'&');

            // Reset anti-bounce count, we need to be here
            st.borg.antibounce_count = 0;

            // reset the inviso clock to avoid loops
            st.borg.need_see_invis = st.borg.time.now;

            return true;
        } else {
            // Must have been a dark room
            borg_note(st, "# Lighted the darkened room instead of resting.");
            return true;
        }
    }

    // Hack to recharge mana if a low level mage or priest
    if st.borg.tr(BI_MAXSP) != 0
        && (st.borg.tr(BI_CLEVEL) <= 40 || st.borg.tr(BI_CDEPTH) >= 85)
        && st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) * 8 / 10
        && p < i32::from(st.borg.avoidance) / 10
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && st.borg.tr(BI_FOOD) > 2
        && !st.borg.munchkin_mode
    {
        borg_note(st, &format!("# Resting to gain Mana. (danger {p})..."));

        // Rest until done
        rest(st, b'*');

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // Hack to recharge mana if a low level mage in munchkin mode
    if st.borg.tr(BI_MAXSP) != 0
        && st.borg.munchkin_mode
        && (st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) || st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP))
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && st.borg.tr(BI_FOOD) > 2
        && (st.cave.grid(st.borg.c.x, st.borg.c.y).feat == FEAT_MORE as u8
            || st.cave.grid(st.borg.c.x, st.borg.c.y).feat == FEAT_LESS as u8)
    {
        borg_note(st, &format!("# Resting to gain munchkin HP/mana. (danger {p})..."));

        // Rest until done
        rest(st, b'*');

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // Hack to heal blindness if in munchkin mode
    if st.borg.tr(BI_ISBLIND) != 0 && st.borg.munchkin_mode {
        // C passes the format string to borg_note() without format(), so
        // the note keeps a literal "%d" (C bug 26).
        borg_note(st, "# Resting to cure problem. (danger %d)...");

        // Rest until done
        rest(st, b'*');

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // Nope
    false
}

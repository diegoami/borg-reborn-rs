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

// borg-reborn-rs (#221): named recovery values; default arithmetic/order preserved.
use borg_rs::decision_api::borg_danger::borg_danger;
use borg_rs::decision_api::borg_flow_misc;
use borg_rs::decision_api::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, CURE_POISON, HEALING, HERBAL_CURING, HEROISM, HOLY_WORD, MINOR_HEALING,
    RECHARGING, REMEMBRANCE, REMOVE_HUNGER, RESTORATION, REVITALIZE, UNHOLY_REPRIEVE,
};
use borg_rs::decision_api::borg_inventory::borg_slot;
use borg_rs::decision_api::borg_io::{borg_keypress, borg_note, KC_ENTER};
use borg_rs::decision_api::borg_item_use::{
    borg_activate_item, borg_eat, borg_quaff_crit, borg_quaff_potion, borg_read_scroll,
    borg_use_staff_fail, borg_zap_rod,
};
use borg_rs::decision_api::borg_light::{borg_check_light_only, borg_maintain_light, BorgNeed};
use borg_rs::decision_api::borg_magic::{borg_spell, borg_spell_okay};
use borg_rs::decision_api::engine;
use borg_rs::decision_api::headers::list_terrain_h::{FEAT_LESS, FEAT_MORE};
use borg_rs::decision_api::headers::list_tvals_h::{TV_FOOD, TV_MUSHROOM, TV_ROD};
use borg_rs::decision_api::headers::ui_event_h::keycode_t;
use borg_rs::memory::BorgState;
use borg_rs::threat_facts::{param, Param};

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
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark("policy::recovery.borg_recover", format!("{:?}", ()));
    }
    /*** Handle annoying situations ***/
    let need = borg_maintain_light(st);
    if need == BorgNeed::MetNeed {
        return true;
    } else if need == BorgNeed::UnmetNeed {
        borg_note(st, "# Need to refuel but cant!");
    }

    /*** Do not recover when in danger ***/

    // Look around for danger
    let p = borg_danger(
        st,
        st.borg.c.y,
        st.borg.c.x,
        param(Param::recover_danger_horizon),
        true,
        false,
    );

    // Never recover in dangerous situations
    if p > i32::from(st.borg.avoidance) / param(Param::recover_danger_divisor) {
        return false;
    }

    /*** Roll for "paranoia" ***/

    // Base roll
    let mut q = engine::randint0(100);

    // Half dead
    if crate::View::new(st).character.curhp
        < crate::View::new(st).character.maxhp / param(Param::recover_hurt_hp_divisor)
    {
        q -= param(Param::recover_hurt_chance_bonus);
    }

    // Almost dead
    if crate::View::new(st).character.curhp
        < crate::View::new(st).character.maxhp / param(Param::recover_critical_hp_divisor)
    {
        q -= param(Param::recover_critical_chance_bonus);
    }

    /*** Use "cheap" cures ***/

    // Cure stun
    if crate::View::new(st).status.isstun != 0
        && q < param(Param::recover_cheap_stun_chance)
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
    if crate::View::new(st).status.isheavystun != 0
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
    if crate::View::new(st).status.iscut != 0
        && q < param(Param::recover_cheap_cut_chance)
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
    if crate::View::new(st).status.ispoisoned != 0
        && q < param(Param::recover_cheap_poison_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, CURE_POISON))
    {
        borg_note(st, &format!("# Cure poison - danger {p}"));
        return true;
    }

    // Cure fear
    if crate::View::new(st).status.isafraid != 0
        && crate::View::new(st).curses.crsfear == 0
        && q < param(Param::recover_cheap_fear_chance)
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
    if (crate::View::new(st).status.ishungry != 0 || crate::View::new(st).status.isweak != 0)
        && q < param(Param::recover_cheap_hunger_chance)
        && (borg_spell(st, REMOVE_HUNGER) || borg_spell(st, HERBAL_CURING))
    {
        return true;
    }

    // Hallucination
    if crate::View::new(st).status.isimage != 0
        && q < param(Param::recover_cheap_image_chance)
        && borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
    {
        return true;
    }

    // Heal damage
    if crate::View::new(st).character.curhp
        < crate::View::new(st).character.maxhp / param(Param::recover_cheap_heal_hp_divisor)
        && q < param(Param::recover_cheap_heal_chance)
        && p == 0
        && crate::View::new(st).character.cursp
            > crate::View::new(st).character.maxsp / param(Param::recover_cheap_heal_sp_divisor)
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
    if crate::View::new(st).status.isfixexp != 0
        && (borg_activate_item(st, st.act.restore_exp)
            || borg_activate_item(st, st.act.restore_st_lev)
            || borg_activate_item(st, st.act.restore_life)
            || borg_spell(st, REVITALIZE)
            || borg_spell(st, REMEMBRANCE)
            || (crate::View::new(st).character.curhp > param(Param::recover_unholy_min_hp)
                && borg_spell(st, UNHOLY_REPRIEVE)))
    {
        return true;
    }

    // cure stat drain with prayer
    if (crate::View::new(st).status.isfixstr != 0
        || crate::View::new(st).status.isfixint != 0
        || crate::View::new(st).status.isfixwis != 0
        || crate::View::new(st).status.isfixdex != 0
        || crate::View::new(st).status.isfixcon != 0
        || crate::View::new(st).status.isfixall != 0)
        && (borg_spell(st, RESTORATION) || borg_spell(st, REVITALIZE))
    {
        return true;
    }

    // cure stat drain with prayer
    if (crate::View::new(st).status.isfixstr != 0
        || crate::View::new(st).status.isfixint != 0
        || crate::View::new(st).status.isfixcon != 0)
        && crate::View::new(st).character.curhp > param(Param::recover_unholy_min_hp)
        && borg_spell(st, UNHOLY_REPRIEVE)
    {
        return true;
    }

    /*** Use "expensive" cures ***/

    // Cure stun
    if crate::View::new(st).status.isstun != 0
        && q < param(Param::recover_costly_stun_chance)
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
    if crate::View::new(st).status.isheavystun != 0
        && q < param(Param::recover_costly_heavy_stun_chance)
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
    if crate::View::new(st).status.iscut != 0
        && q < param(Param::recover_costly_cut_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_quaff_crit(
                st,
                crate::View::new(st).character.curhp < param(Param::recover_critical_ccw_hp),
            ))
    {
        return true;
    }

    // Cure poison
    if crate::View::new(st).status.ispoisoned != 0
        && q < param(Param::recover_costly_poison_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_poison)
            || borg_eat(st, TV_FOOD as i32, st.item_val.sv_food_waybread)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_crit(
                st,
                crate::View::new(st).character.curhp < param(Param::recover_critical_ccw_hp),
            )
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_activate_item(st, st.act.food_waybread))
    {
        return true;
    }

    // Cure blindness
    if crate::View::new(st).status.isblind != 0
        && q < param(Param::recover_costly_blind_chance)
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
    if crate::View::new(st).status.isconfused != 0
        && q < param(Param::recover_costly_confusion_chance)
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
    if crate::View::new(st).status.isafraid != 0
        && crate::View::new(st).curses.crsfear == 0
        && q < param(Param::recover_costly_fear_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_quaff_potion(st, st.item_val.sv_potion_boldness)
            || borg_quaff_potion(st, st.item_val.sv_potion_heroism)
            || borg_quaff_potion(st, st.item_val.sv_potion_berserk)
            || borg_activate_item(st, st.act.rem_fear_pois))
    {
        return true;
    }

    // Satisfy hunger
    if (crate::View::new(st).status.ishungry != 0 || crate::View::new(st).status.isweak != 0)
        && q < param(Param::recover_costly_hunger_chance)
        && (borg_read_scroll(st, st.item_val.sv_scroll_satisfy_hunger)
            || borg_activate_item(st, st.act.satisfy))
    {
        return true;
    }

    // Heal damage
    if crate::View::new(st).character.curhp
        < crate::View::new(st).character.maxhp / param(Param::recover_costly_heal_hp_divisor)
        && q < param(Param::recover_costly_heal_chance)
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
        && crate::View::new(st).status.isweak == 0
        && crate::View::new(st).status.iscut == 0
        && crate::View::new(st).status.ishungry == 0
        && crate::View::new(st).status.ispoisoned == 0
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && !borg_spell_okay(st, RECHARGING)
    {
        borg_note(st, "# Resting to recharge a rod...");

        // Reset the Bouncing-borg Timer
        st.borg.antibounce_count = 0;

        // Rest until done
        borg_keypress(st, key(b'R'));
        for digit in param(Param::recover_rod_rest_turns).to_string().bytes() {
            borg_keypress(st, key(digit));
        }
        borg_keypress(st, KC_ENTER);

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    /*** Just Rest ***/

    // Rest until healed
    let sp_tenths = if crate::View::new(st).character.cdepth > param(Param::recover_rest_high_depth)
    {
        param(Param::recover_rest_high_sp_tenths)
    } else {
        param(Param::recover_rest_sp_tenths)
    };
    if crate::View::new(st).status.isblind == 0
        && crate::View::new(st).status.ispoisoned == 0
        && crate::View::new(st).status.iscut == 0
        && crate::View::new(st).status.isweak == 0
        && crate::View::new(st).status.ishungry == 0
        && (crate::View::new(st).status.isconfused != 0
            || crate::View::new(st).status.isimage != 0
            || crate::View::new(st).status.isafraid != 0
            || crate::View::new(st).status.isstun != 0
            || crate::View::new(st).status.isheavystun != 0
            || crate::View::new(st).character.curhp < crate::View::new(st).character.maxhp
            || crate::View::new(st).character.cursp
                < crate::View::new(st).character.maxsp * sp_tenths / 10)
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
    if crate::View::new(st).character.maxsp != 0
        && (crate::View::new(st).character.clevel <= param(Param::recover_mana_rest_max_level)
            || crate::View::new(st).character.cdepth >= param(Param::recover_mana_rest_deep_depth))
        && crate::View::new(st).character.cursp
            < crate::View::new(st).character.maxsp * param(Param::recover_mana_rest_sp_tenths) / 10
        && p < i32::from(st.borg.avoidance) / param(Param::recover_mana_rest_danger_divisor)
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && crate::View::new(st).status.isweak == 0
        && crate::View::new(st).status.iscut == 0
        && crate::View::new(st).status.ishungry == 0
        && crate::View::new(st).status.ispoisoned == 0
        && crate::View::new(st).food.food > param(Param::recover_mana_rest_min_food)
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
    if crate::View::new(st).character.maxsp != 0
        && st.borg.munchkin_mode
        && (crate::View::new(st).character.cursp < crate::View::new(st).character.maxsp
            || crate::View::new(st).character.curhp < crate::View::new(st).character.maxhp)
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && crate::View::new(st).status.isweak == 0
        && crate::View::new(st).status.iscut == 0
        && crate::View::new(st).status.ishungry == 0
        && crate::View::new(st).status.ispoisoned == 0
        && crate::View::new(st).food.food > param(Param::recover_munchkin_rest_min_food)
        && (st.cave.grid(st.borg.c.x, st.borg.c.y).feat == FEAT_MORE as u8
            || st.cave.grid(st.borg.c.x, st.borg.c.y).feat == FEAT_LESS as u8)
    {
        borg_note(
            st,
            &format!("# Resting to gain munchkin HP/mana. (danger {p})..."),
        );

        // Rest until done
        rest(st, b'*');

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // Hack to heal blindness if in munchkin mode
    if crate::View::new(st).status.isblind != 0 && st.borg.munchkin_mode {
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

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
use crate::params::{value as param, Param};
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
use crate::headers::list_tvals_h::{TV_FOOD, TV_MUSHROOM, TV_ROD};
use crate::headers::ui_event_h::keycode_t;
use crate::state::BorgState;

/// `GOAL_RECOVER` (borg-flow.h): the goal of flowing somewhere safe to rest.
const GOAL_RECOVER: i16 = 9;
/// `GOAL_FLEE` (borg-flow.h): the flight to a staircase (#614).
const GOAL_FLEE: i16 = 7;

/// borg-reborn-rs (#614, owner): "once committed to a stair, it must at
/// least reach that stair before stopping to rest". A Borg on a flight
/// flow (`GOAL_FLEE`) whose staircase is at most `recover_rest_reach_stairs`
/// flow steps away does not rest until it stands on it: no rod, HP or mana
/// rest. Potions and cures are not rests. A farther staircase is a long walk,
/// not a commitment: a hurt Borg that held its rests for 138 steps walked into
/// a crow and died (18 starts, first version). 0 = off (C). On the staircase
/// it may rest; a monster then sends it up or down
/// (`caution_stairs_leave_on_monster`). Noted when it holds a rest that
/// would otherwise start.
fn rest_held_for_stairs(st: &mut BorgState) -> bool {
    if !holding_for_stairs(st) {
        return false;
    }
    // Review of #616: the rod, HP and mana rests all ask; one note a think.
    if st.borg.rest_hold_noted != Some(st.borg.time.now) {
        st.borg.rest_hold_noted = Some(st.borg.time.now);
        let steps = committed_steps(st);
        borg_note(st, &format!("# No rest: committed to the stairs, {steps} steps to go."));
    }
    true
}

/// borg-reborn-rs (#614, review of #616): the Borg is committed to a
/// staircase, so neither a rest nor a flow to a recovery grid
/// (`borg_flow_recover`, `GOAL_RECOVER`) may take it off the flight.
pub(crate) fn holding_for_stairs(st: &BorgState) -> bool {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    let on_stairs = feat == crate::headers::list_terrain_h::FEAT_LESS || feat == crate::headers::list_terrain_h::FEAT_MORE;
    let (flight, steps) = flight_now(st);
    hold_rest_for_stairs(param(Param::recover_rest_reach_stairs), flight, on_stairs, steps)
}

fn committed_steps(st: &BorgState) -> i32 {
    flight_now(st).1
}

/// (on a flight, the flow steps it has to go). On the flight flow, the steps
/// left from here. Otherwise a flight that is still on (fleeing, leaving or
/// a scary level), on this level, whose last step was within
/// `recover_rest_reach_stairs_memory`: a new monster wipes the flow goal for
/// a think, and the rest must not slip in then (the Druid of #614 did, 11
/// steps from the stairs, in a first version).
fn flight_now(st: &BorgState) -> (bool, i32) {
    // Review of #616 (N4): only a flight to a staircase; the breeder exit and
    // the flow to light are flights to somewhere else.
    if st.borg.goal.type_0 == GOAL_FLEE {
        let steps = i32::from(st.flow.data_flow[st.borg.c.y as usize][st.borg.c.x as usize]);
        return flee_flow_flight(st.borg.flight_to_stairs, steps);
    }
    let still_fleeing = st.borg.goal.fleeing || st.borg.goal.leaving || st.borg.mon.scary;
    let recent = st.borg.time.now.saturating_sub(st.borg.flight_time) <= param(Param::recover_rest_reach_stairs_memory);
    let here = st.borg.flight_depth == crate::engine::player_depth();
    (still_fleeing && recent && here && st.borg.flight_time > 0, st.borg.flight_steps)
}

/// On a flee flow: a flight to a staircase, `steps` away, or none.
fn flee_flow_flight(to_stairs: bool, steps: i32) -> (bool, i32) {
    if to_stairs {
        (true, steps)
    } else {
        (false, 0)
    }
}

fn hold_rest_for_stairs(max_steps: i32, flight: bool, on_stairs: bool, steps: i32) -> bool {
    max_steps > 0 && flight && !on_stairs && steps <= max_steps
}

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
    let p = borg_danger(st, st.borg.c.y, st.borg.c.x, param(Param::recover_danger_horizon), true, false);

    // Never recover in dangerous situations
    if p > i32::from(st.borg.avoidance) / param(Param::recover_danger_divisor) {
        return false;
    }

    /*** Roll for "paranoia" ***/

    // Base roll
    let mut q = engine::randint0(100);

    // Half dead
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::recover_hurt_hp_divisor) {
        q -= param(Param::recover_hurt_chance_bonus);
    }

    // Almost dead
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::recover_critical_hp_divisor) {
        q -= param(Param::recover_critical_chance_bonus);
    }

    /*** Use "cheap" cures ***/

    // Cure stun
    if st.borg.tr(BI_ISSTUN) != 0
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
    if st.borg.tr(BI_ISPOISONED) != 0
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
    if st.borg.tr(BI_ISAFRAID) != 0
        && st.borg.tr(BI_CRSFEAR) == 0
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
    if (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0)
        && q < param(Param::recover_cheap_hunger_chance)
        && (borg_spell(st, REMOVE_HUNGER) || borg_spell(st, HERBAL_CURING))
    {
        return true;
    }

    // Hallucination
    if st.borg.tr(BI_ISIMAGE) != 0
        && q < param(Param::recover_cheap_image_chance)
        && borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
    {
        return true;
    }

    // Heal damage
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::recover_cheap_heal_hp_divisor)
        && q < param(Param::recover_cheap_heal_chance)
        && p == 0
        && st.borg.tr(BI_CURSP) > st.borg.tr(BI_MAXSP) / param(Param::recover_cheap_heal_sp_divisor)
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
            || (st.borg.tr(BI_CURHP) > param(Param::recover_unholy_min_hp) && borg_spell(st, UNHOLY_REPRIEVE)))
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
        && st.borg.tr(BI_CURHP) > param(Param::recover_unholy_min_hp)
        && borg_spell(st, UNHOLY_REPRIEVE)
    {
        return true;
    }

    /*** Use "expensive" cures ***/

    // Cure stun
    // (the HEAL1 and HEAL2 activations cure only cuts, not stun; patch 0134)
    if st.borg.tr(BI_ISSTUN) != 0
        && q < param(Param::recover_costly_stun_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_quaff_crit(st, false))
    {
        return true;
    }

    // Cure heavy stun
    if st.borg.tr(BI_ISHEAVYSTUN) != 0
        && q < param(Param::recover_costly_heavy_stun_chance)
        && (borg_quaff_crit(st, true)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing))
    {
        return true;
    }

    // Cure cuts
    if st.borg.tr(BI_ISCUT) != 0
        && q < param(Param::recover_costly_cut_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_quaff_crit(st, st.borg.tr(BI_CURHP) < param(Param::recover_critical_ccw_hp)))
    {
        return true;
    }

    // Cure poison
    if st.borg.tr(BI_ISPOISONED) != 0
        && q < param(Param::recover_costly_poison_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_poison)
            || borg_eat(st, TV_FOOD as i32, st.item_val.sv_food_waybread)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_crit(st, st.borg.tr(BI_CURHP) < param(Param::recover_critical_ccw_hp))
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_activate_item(st, st.act.food_waybread))
    {
        return true;
    }

    // Cure blindness
    // (Waybread cures only poison, so it is not tried here; patch 0134)
    if st.borg.tr(BI_ISBLIND) != 0
        && q < param(Param::recover_costly_blind_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_crit(st, false)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing))
    {
        return true;
    }

    // Cure confusion
    if st.borg.tr(BI_ISCONFUSED) != 0
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
    if st.borg.tr(BI_ISAFRAID) != 0
        && st.borg.tr(BI_CRSFEAR) == 0
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
    if (st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_ISWEAK) != 0)
        && q < param(Param::recover_costly_hunger_chance)
        && (borg_read_scroll(st, st.item_val.sv_scroll_satisfy_hunger)
            || borg_activate_item(st, st.act.satisfy))
    {
        return true;
    }

    // Heal damage
    if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / param(Param::recover_costly_heal_hp_divisor)
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
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && !borg_spell_okay(st, RECHARGING)
        && !rest_held_for_stairs(st)
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
    let sp_tenths = if st.borg.tr(BI_CDEPTH) > param(Param::recover_rest_high_depth) { param(Param::recover_rest_high_sp_tenths) } else { param(Param::recover_rest_sp_tenths) };
    let rest_hp = st.borg.tr(BI_ISBLIND) == 0
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
        && !rest_held_for_stairs(st);
    // borg-reborn-rs (#558): breeders multiply while the Borg rests; hunt
    // them first while it may and is not badly hurt. The breeder scan runs
    // only when a rest is due and the HP test passes.
    let hunt_first = rest_hp && hunt_now(st);
    // Noted once per stretch of skipped rests, not every think.
    let noted = NOT_RESTING_NOTED.with(|n| n.replace(rest_hp && hunt_first));
    if rest_hp && hunt_first && !noted {
        borg_note(st, "# Not resting: breeders to hunt.");
    }
    if rest_hp && !hunt_first {
        // check for then call lite in dark room before resting
        // borg-reborn-rs (#642): a staircase or a room corner first.
        if borg_flow_misc::borg_flow_rest_spot(st, param(Param::recover_rest_spot_steps)) {
            return true;
        }
        if !borg_check_light_only(st) {
            borg_note(st, "# Resting to recover HP/SP...");

            // Rest until done
            rest(st, b'&');

            // Reset anti-bounce count, we need to be here
            st.borg.antibounce_count = 0;

            // reset the inviso clock to avoid loops
            // borg-reborn-rs (#516): C's reset stamps the clock as recent,
            // which arms the Detect Invisible defence after every rest
            // (borg_defend_aux_inviso fires while it is recent). With
            // recover_rest_arms_see_invis 0 a rest leaves the clock alone.
            if crate::params::value(crate::params::Param::recover_rest_arms_see_invis) != 0 {
                st.borg.need_see_invis = st.borg.time.now;
            }

            return true;
        } else {
            // Must have been a dark room
            borg_note(st, "# Lighted the darkened room instead of resting.");
            return true;
        }
    }

    // Hack to recharge mana if a low level mage or priest
    if st.borg.tr(BI_MAXSP) != 0
        && (st.borg.tr(BI_CLEVEL) <= param(Param::recover_mana_rest_max_level) || st.borg.tr(BI_CDEPTH) >= param(Param::recover_mana_rest_deep_depth))
        && st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) * param(Param::recover_mana_rest_sp_tenths) / 10
        && p < i32::from(st.borg.avoidance) / param(Param::recover_mana_rest_danger_divisor)
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && st.borg.tr(BI_ISWEAK) == 0
        && st.borg.tr(BI_ISCUT) == 0
        && st.borg.tr(BI_ISHUNGRY) == 0
        && st.borg.tr(BI_ISPOISONED) == 0
        && st.borg.tr(BI_FOOD) > param(Param::recover_mana_rest_min_food)
        && !hunt_now(st)
        && !rest_held_for_stairs(st)
    {
        // borg-reborn-rs (#642): a staircase or a room corner first.
        if borg_flow_misc::borg_flow_rest_spot(st, param(Param::recover_rest_spot_steps)) {
            return true;
        }
        borg_note(st, &format!("# Resting to gain Mana. (danger {p})..."));

        // Rest until done
        rest(st, b'*');

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // Nope
    false
}

thread_local! {
    /// borg-reborn-rs (#558): the last think skipped a rest for the hunt.
    /// Only the note reads it.
    static NOT_RESTING_NOTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// borg-reborn-rs (#558): `hunt_before_rest` with the HP test first, so
/// `borg_flow_kill::breeder_hunt_now`'s scan runs only when it passes.
fn hunt_now(st: &mut BorgState) -> bool {
    hunt_before_rest(true, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), param(Param::recover_breeder_hunt_hp_percent))
        && crate::borg_flow_kill::breeder_hunt_now(st)
}

/// borg-reborn-rs (#558): skip resting to hunt breeders while `hunt` (the
/// breeder hunt applies) and HP is at least `hp_percent` of the maximum;
/// 0 = off (C's rest).
fn hunt_before_rest(hunt: bool, cur_hp: i32, max_hp: i32, hp_percent: i32) -> bool {
    hp_percent > 0 && hunt && i64::from(cur_hp) * 100 >= i64::from(max_hp) * i64::from(hp_percent)
}

#[cfg(test)]
mod rest_at_stairs_tests {
    use super::hold_rest_for_stairs;

    #[test]
    fn a_rest_waits_for_the_stairs_only_on_a_flight_off_the_stairs() {
        assert!(hold_rest_for_stairs(20, true, false, 11), "committed, not there yet");
        assert!(hold_rest_for_stairs(20, true, false, 20), "at the limit");
        assert!(!hold_rest_for_stairs(20, true, false, 21), "a long walk, not a commitment");
        assert!(!hold_rest_for_stairs(20, true, true, 0), "on the staircase: rest");
        assert!(!hold_rest_for_stairs(20, false, false, 11), "no flight");
        assert!(!hold_rest_for_stairs(0, true, false, 11), "off (C)");
    }

    /// Review of #616 (N4): a flee flow that is not to a staircase (the
    /// breeder exit, the flow to light) is not a commitment to the stairs.
    #[test]
    fn only_a_flight_to_stairs_commits_the_rest() {
        assert_eq!(super::flee_flow_flight(false, 11), (false, 0), "the breeder exit");
        assert_eq!(super::flee_flow_flight(true, 11), (true, 11), "a stair flow, 11 steps to go");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breeders_are_hunted_before_resting() {
        // The priest of priest-1055640847 at turn 88,147: 55/70 HP.
        assert!(hunt_before_rest(true, 55, 70, 50));
        assert!(!hunt_before_rest(true, 30, 70, 50), "badly hurt: rest");
        assert!(!hunt_before_rest(false, 55, 70, 50), "no hunt");
        assert!(!hunt_before_rest(true, 55, 70, 0), "off");
    }
}

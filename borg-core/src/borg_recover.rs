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
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// `GOAL_RECOVER` (borg-flow.h): the goal of flowing somewhere safe to rest.
const GOAL_RECOVER: i16 = 9;
/// `GOAL_FLEE` (borg-flow.h): the flight to a staircase (#614).
const GOAL_FLEE: i16 = 7;

/// borg-reborn-rs (#615 rule a, owner: "ok do a"): with
/// `recover_rest_monster_turns`, record the game turn while an awake monster
/// that can move is known within `recover_rest_monster_distance`. Called
/// every dungeon think (`borg_caution`) and before every rest, so a monster
/// forgotten out of view still counts.
pub(crate) fn watch_for_rest(st: &mut BorgState) {
    if param(Param::recover_rest_monster_turns) == 0 {
        return;
    }
    if let Some(k) = crate::borg_caution::nearest_mover_seen(st, param(Param::recover_rest_monster_distance)) {
        st.borg.rest_watch_turn = crate::engine::game_turn();
        st.borg.rest_watch_depth = crate::engine::player_depth();
        st.borg.rest_watch_kill = k;
    }
}

/// #615 rule a: no rest (rod, HP/SP, mana) while an awake monster was near
/// within the last `recover_rest_monster_turns` game turns on this level.
/// A rest then is broken at once by the game (#615's analysis: 63% of town
/// rests and 13% of dungeon rests lasted one player turn, mostly with an
/// awake monster in view, up to 32 in a row). The Borg does something else.
fn rest_held_for_monster(st: &mut BorgState) -> bool {
    let Some(ago) = holding_for_monster_ago(st) else {
        return false;
    };
    if st.borg.rest_watch_noted != Some(st.borg.time.now) {
        st.borg.rest_watch_noted = Some(st.borg.time.now);
        borg_note(st, &format!("# No rest: a monster was near {ago} game turns ago."));
    }
    true
}

/// #615 rule a: the rest is held (`Some(game turns since the monster)`),
/// without the note. `borg_think_dungeon` also holds the walk to a recovery
/// grid with it (review of #645), as #614 does with `holding_for_stairs`.
pub(crate) fn holding_for_monster(st: &mut BorgState) -> bool {
    holding_for_monster_ago(st).is_some()
}

fn holding_for_monster_ago(st: &mut BorgState) -> Option<i32> {
    let turns = param(Param::recover_rest_monster_turns);
    if turns == 0 {
        return None;
    }
    // Impaired (confused, blind, hallucinating, stunned, afraid), walking is
    // worse than a broken rest: a confused Ranger held its rest in town,
    // walked into lava and died (first local run); an afraid Borg twitches
    // in a random direction (review of #645).
    let impaired = [BI_ISCONFUSED, BI_ISBLIND, BI_ISIMAGE, BI_ISSTUN, BI_ISHEAVYSTUN, BI_ISAFRAID]
        .iter()
        .any(|&f| st.borg.tr(f) != 0);
    watch_for_rest(st);
    let ago = crate::engine::game_turn() - st.borg.rest_watch_turn;
    let here = st.borg.rest_watch_depth == crate::engine::player_depth();
    rest_monster_hold(turns, st.borg.rest_watch_turn, ago, here, impaired).then_some(ago)
}

/// The test of `holding_for_monster`: on, not impaired, a clock set (and
/// not cleared by the monster's death), on this level, within `turns`.
fn rest_monster_hold(turns: i32, seen_turn: i32, ago: i32, same_level: bool, impaired: bool) -> bool {
    !impaired && crate::borg_caution::recall_watch_holds(turns, seen_turn, ago, same_level)
}

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

    // borg-reborn-rs (#697): a monster the Borg cannot see is attacking
    // (no monster, so no danger): heal at low HP, and walk away from where
    // it struck before resting.
    if unseen_heal(st) || unseen_away(st) {
        return true;
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
        && !rest_held_for_monster(st)
        // borg-reborn-rs (#644): the rest is due; breeders may hold it.
        && !hold_for_breeders(st)
        // borg-reborn-rs (#697): not where an unseen attacker struck.
        && !unseen_rest_held(st)
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
        && !rest_held_for_stairs(st)
        && !rest_held_for_monster(st)
        // borg-reborn-rs (#644): the rest is due; breeders may hold it.
        && !hold_for_breeders(st)
        // borg-reborn-rs (#697): not where an unseen attacker struck.
        && !unseen_rest_held(st);
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
        && !rest_held_for_monster(st)
        // borg-reborn-rs (#644): the rest is due; breeders may hold it.
        && !hold_for_breeders(st)
        // borg-reborn-rs (#697): not where an unseen attacker struck.
        && !unseen_rest_held(st)
    {
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

/// borg-reborn-rs (#644, owner: "no rest if aware of any breeders, clear the
/// breeders or leave the level asap"): what to do instead of a rest while
/// breeders are known on the level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BreederRest {
    /// No breeder rule applies: rest as before.
    Rest,
    /// Hunt the breeders (#558's hunt applies).
    Hunt,
    /// Leave the level (`goal.leaving`).
    Leave,
}

/// The decision of `recover_rest_breeders`, read from the state only. The
/// hunt is judged as if this rule's own leave were not set (review of #656:
/// `breeder_engaged` reads `goal.leaving` as "not engaged", so a leave set
/// here stopped the hunt from ever resuming on the level).
fn breeder_rest_decision(st: &mut BorgState) -> BreederRest {
    let on = param(Param::recover_rest_breeders) != 0;
    if !on {
        return BreederRest::Rest;
    }
    let breeders = i32::from(st.borg.mon.breeders);
    if !breeder_hold(on, st.borg.tr(BI_CDEPTH), impaired(st), breeders) {
        return BreederRest::Rest;
    }
    let ours = st.borg.breeder_leaving && st.borg.goal.leaving;
    let engaged = crate::borg_caution::breeder_engaged_but_own_leave(st, ours);
    breeder_choice(crate::borg_flow_kill::breeder_hunt_now_with(st, engaged))
}

/// Hunt when the hunt applies, otherwise leave.
fn breeder_choice(hunt: bool) -> BreederRest {
    if hunt {
        BreederRest::Hunt
    } else {
        BreederRest::Leave
    }
}

/// Act on a `Hunt` or `Leave` decision: a hunt that applies again clears
/// this rule's own leave; a leave is set once and remembered as this rule's.
/// Noted once a think.
fn apply_breeder_rest(st: &mut BorgState, d: BreederRest) {
    match d {
        BreederRest::Rest => return,
        BreederRest::Hunt => {
            let g = &st.borg.goal;
            if withdraw_own_leave(st.borg.breeder_leaving, g.leaving, g.fleeing || g.fleeing_to_town) {
                st.borg.goal.leaving = false;
                borg_note(st, "# Not leaving: the breeder hunt applies again.");
            }
            st.borg.breeder_leaving = false;
        }
        BreederRest::Leave => {
            if !st.borg.goal.leaving {
                crate::borg_caution::start_leaving(st, "# Leaving (breeders on the level, no rest)");
                st.borg.breeder_leaving = true;
            }
        }
    }
    if st.borg.breeder_rest_noted != Some(st.borg.time.now) {
        st.borg.breeder_rest_noted = Some(st.borg.time.now);
        let then = if d == BreederRest::Hunt { "hunting them" } else { "leaving" };
        let n = st.borg.mon.breeders;
        borg_note(st, &format!("# No rest: {n} breeders known on the level, {then}."));
    }
}

/// Re-review of #656: a hunt that applies again withdraws the leave only if
/// it is this rule's (`own`) and the Borg is not fleeing. The low-hit-points
/// and bleeding/poison leaves (`borg_caution`) set `goal.fleeing` too; they
/// may join a leave this rule set first (they only set it when it is not
/// set yet), and must not lose it to the hunt.
fn withdraw_own_leave(own: bool, leaving: bool, fleeing: bool) -> bool {
    own && leaving && !fleeing
}

/// A rest that is due is held for breeders: decide, act on the decision,
/// and say whether the rest is held. Kept when confused, blind,
/// hallucinating or stunned: walking then is worse (#615 rule a).
fn hold_for_breeders(st: &mut BorgState) -> bool {
    let d = breeder_rest_decision(st);
    apply_breeder_rest(st, d);
    d != BreederRest::Rest
}

/// borg-reborn-rs (#697): whether `who` (a "HIT_BY:"/"MISS_BY:" name) is a
/// monster the Borg cannot see: "Something" or "It", as `borg_locate_kill`
/// tests them.
fn unseen_name(who: &[u8]) -> bool {
    let lower: Vec<u8> = who.iter().map(u8::to_ascii_lowercase).collect();
    lower.starts_with(b"something") || lower == b"it" || lower.starts_with(b"it ")
}

/// borg-reborn-rs (#697): remember an attack by a monster the Borg cannot
/// see: the game turn and the Borg's grid. One that strikes at least
/// `recover_unseen_attack_distance` grids from the last one, within
/// `recover_unseen_attack_turns`, has followed the Borg. State only: it
/// draws no random number and decides nothing.
pub(crate) fn note_unseen_attack(st: &mut BorgState, who: &[u8]) {
    if !unseen_name(who) {
        return;
    }
    let now = engine::game_turn();
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    let turns = param(Param::recover_unseen_attack_turns);
    if unseen_recent(turns, now, st.borg.unseen_turn)
        && engine::distance(loc { x, y }, loc { x: st.borg.unseen_x, y: st.borg.unseen_y })
            >= param(Param::recover_unseen_attack_distance)
    {
        st.borg.unseen_followed = true;
    } else if !unseen_recent(turns, now, st.borg.unseen_turn) {
        st.borg.unseen_followed = false;
    }
    st.borg.unseen_turn = now;
    st.borg.unseen_x = x;
    st.borg.unseen_y = y;
}

/// borg-reborn-rs (#697): an unseen attack at game turn `at` (0: none) is
/// recent at `now` under a window of `turns` game turns (0: off).
fn unseen_recent(turns: i32, now: i32, at: i32) -> bool {
    turns > 0 && at > 0 && now - at <= turns
}

/// borg-reborn-rs (#697): the rule applies: on, in the dungeon, an unseen
/// attack is recent.
fn unseen_active(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) > 0
        && unseen_recent(param(Param::recover_unseen_attack_turns), engine::game_turn(), st.borg.unseen_turn)
}

/// borg-reborn-rs (#697): the Borg is within `recover_unseen_attack_distance`
/// grids of the grid where the last unseen attack struck.
fn unseen_near(st: &BorgState) -> bool {
    engine::distance(st.borg.c, loc { x: st.borg.unseen_x, y: st.borg.unseen_y })
        < param(Param::recover_unseen_attack_distance)
}

/// borg-reborn-rs (#697): no rest while an unseen attack is recent and the
/// Borg is near where it struck, or the attacker has followed it.
fn unseen_rest_held(st: &BorgState) -> bool {
    unseen_active(st) && (st.borg.unseen_followed || unseen_near(st))
}

/// borg-reborn-rs (#697), `recover_unseen_heal_hp_percent`: while an unseen
/// attack is recent, below this share of the maximum HP, drink a cure: its
/// HP loss counts as no danger, so `borg_heal`'s tests never fire.
fn unseen_heal(st: &mut BorgState) -> bool {
    let percent = param(Param::recover_unseen_heal_hp_percent);
    if percent == 0 || !unseen_active(st) || st.borg.tr(BI_CURHP) * 100 >= st.borg.tr(BI_MAXHP) * percent {
        return false;
    }
    let healed = borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        || borg_quaff_crit(st, false);
    if healed {
        borg_note(st, "# Healing: an unseen attacker, HP low.");
    }
    healed
}

/// borg-reborn-rs (#697): while an unseen attack is recent and the Borg is
/// hurt (or short of mana) near where it struck, walk to a known grid at
/// least `recover_unseen_attack_distance` grids from there (a corridor or
/// doorway first, as #544's exit) and rest there. If the attacker has
/// followed it, or no such grid can be reached, leave the level.
fn unseen_away(st: &mut BorgState) -> bool {
    if !unseen_active(st) {
        return false;
    }
    if st.borg.unseen_followed {
        if !st.borg.goal.leaving {
            crate::borg_caution::start_leaving(st, "# Leaving (an unseen attacker followed)");
        }
        return false;
    }
    let wants_rest = st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) || st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP);
    if !wants_rest || !unseen_near(st) {
        return false;
    }
    let at = loc { x: st.borg.unseen_x, y: st.borg.unseen_y };
    let distance = param(Param::recover_unseen_attack_distance);
    if crate::borg_caution::walk_away_from(st, at, distance, c"unseen-away") {
        borg_note(st, &format!("# Walking away from an unseen attacker (struck at ({},{})).", at.y, at.x));
        return true;
    }
    if !st.borg.goal.leaving {
        crate::borg_caution::start_leaving(st, "# Leaving (an unseen attacker, nowhere to walk to)");
    }
    false
}

/// The test of `breeder_rest_decision`: on, in the dungeon, not impaired,
/// and a breeder known on the level.
fn breeder_hold(on: bool, depth: i32, impaired: bool, breeders: i32) -> bool {
    on && depth > 0 && !impaired && breeders > 0
}

/// Confused, blind, hallucinating or stunned: a rest clears it, and walking
/// is dangerous.
fn impaired(st: &BorgState) -> bool {
    [BI_ISCONFUSED, BI_ISBLIND, BI_ISIMAGE, BI_ISSTUN, BI_ISHEAVYSTUN].iter().any(|&f| st.borg.tr(f) != 0)
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
mod breeder_rest_tests {
    use super::{breeder_choice, breeder_hold, BreederRest};

    /// #644: no rest while a breeder is known on the level.
    #[test]
    fn no_rest_while_breeders_are_known() {
        assert!(breeder_hold(true, 3, false, 1), "a worm mass on DL3");
        assert!(!breeder_hold(true, 3, false, 0), "no breeder known");
        assert!(!breeder_hold(true, 0, false, 4), "town");
        assert!(!breeder_hold(true, 3, true, 4), "confused: the rest clears it");
        assert!(!breeder_hold(false, 3, false, 4), "off (C)");
        assert_eq!(breeder_choice(true), BreederRest::Hunt);
        assert_eq!(breeder_choice(false), BreederRest::Leave);
    }

    /// Review of #656: hunt, a transient failure (the HP below the hunt's
    /// percent) sets this rule's leave, and the hunt resumes when it applies
    /// again, because the hunt ignores the rule's own leave.
    #[test]
    fn the_hunt_resumes_after_the_rules_own_leave() {
        use crate::borg_caution::engaged_ignoring_own_leave as engaged;
        // (ignoring, fleeing, leaving, scary, leaving is ours)
        assert!(engaged(false, false, false, false, false), "hunting");
        assert!(engaged(false, false, true, false, true), "this rule's leave does not stop the hunt");
        assert!(!engaged(false, false, true, false, false), "another rule's leave still does");
        assert!(!engaged(true, false, true, false, true), "ignoring the breeders (#503) still does");
        assert!(!engaged(false, true, true, false, true), "fleeing still does");
    }

    /// Re-review of #656: the rule's leave, then the low-HP leave joins it
    /// (and sets fleeing): the resumed hunt keeps the leave.
    #[test]
    fn the_hunt_keeps_a_leave_another_rule_joined() {
        use super::withdraw_own_leave;
        assert!(withdraw_own_leave(true, true, false), "the rule's own leave: withdrawn");
        assert!(!withdraw_own_leave(true, true, true), "low HP or bleeding joined (fleeing): kept");
        assert!(!withdraw_own_leave(false, true, false), "another rule's leave: kept");
        assert!(!withdraw_own_leave(true, false, false), "nothing to withdraw");
    }
}

#[cfg(test)]
mod rest_monster_tests {
    use super::rest_monster_hold;

    /// #615 rule a (review of #645): the rest waits for a monster seen
    /// recently, on this level, unless impaired or the monster died.
    #[test]
    fn no_rest_soon_after_a_monster_was_near() {
        assert!(rest_monster_hold(100, 5000, 30, true, false), "a yeek 30 game turns ago");
        assert!(rest_monster_hold(100, 5000, 100, true, false), "at the limit");
        assert!(!rest_monster_hold(100, 5000, 101, true, false), "long enough ago");
        assert!(!rest_monster_hold(100, 5000, 30, false, false), "another level");
        assert!(!rest_monster_hold(100, 5000, 30, true, true), "impaired (afraid, confused...): rest");
        assert!(!rest_monster_hold(100, 0, 30, true, false), "the clock cleared: the monster died");
        assert!(!rest_monster_hold(0, 5000, 30, true, false), "off (C)");
    }
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

#[cfg(test)]
mod unseen_tests {
    use super::{unseen_name, unseen_recent};

    /// #697: the names the game gives a monster the Borg cannot see.
    #[test]
    fn unseen_names() {
        assert!(unseen_name(b"Something"));
        assert!(unseen_name(b"something"));
        assert!(unseen_name(b"It"));
        assert!(!unseen_name(b"Itchy thing"), "a name that starts with it");
        assert!(!unseen_name(b"the clear icky thing"));
    }

    /// #697: the window, in game turns; 0 is off, and no attack is none.
    #[test]
    fn recent_window() {
        assert!(unseen_recent(300, 1_300, 1_000));
        assert!(!unseen_recent(300, 1_301, 1_000));
        assert!(!unseen_recent(0, 1_000, 1_000), "0 is off");
        assert!(!unseen_recent(300, 1_000, 0), "no attack on this level");
    }
}

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
    BI_ISWEAK, BI_LIGHT, BI_MAXHP, BI_MAXSP,
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
    // (no monster, so no danger): walk away from where it struck, then, once
    // clear (or with nowhere to walk), heal at low HP. The walk comes first
    // so a low heal threshold cannot keep it drinking where it is struck
    // (review of #698).
    // borg-reborn-rs (#831): in the invisible-attacker mode, first detect the
    // attacker or strike where it must be.
    if unseen_deal(st) || unseen_away(st) || unseen_heal(st) {
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
    let rest_sp = rest_sp_floor(st);
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
            || st.borg.tr(BI_CURSP) < rest_sp)
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
            // borg-reborn-rs (#982): badly hurt, with a fast monster awake
            // near, drink a cure instead of the rest it would break (after
            // the light, as C lights before it rests; review of #999).
            if cure_before_rest(st) {
                return true;
            }
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

    // Rest until HP and SP are full (player-util.c:1500-1501).
    if mana_rest_sp_floor(st).is_some_and(|floor| st.borg.tr(BI_CURSP) < floor)
        && p < mana_rest_danger_gate(
            param(Param::recover_mana_rest_full_hp) != 0,
            st.borg.tr(BI_CURHP),
            st.borg.tr(BI_MAXHP),
            i32::from(st.borg.avoidance),
            param(Param::recover_danger_divisor),
            param(Param::recover_mana_rest_danger_divisor),
        )
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
pub(crate) fn hold_for_breeders(st: &mut BorgState) -> bool {
    let d = breeder_rest_decision(st);
    apply_breeder_rest(st, d);
    d != BreederRest::Rest
}

/// borg-reborn-rs (#697): whether `who` (a "HIT_BY:"/"MISS_BY:" name) is a
/// monster the Borg cannot see. The blow's name is
/// `monster_desc(..., MDESC_STANDARD)` (`mon-attack.c:778`): for a monster
/// that is not visible, "Something" if it is neuter and "Someone" if it is
/// male or female (`mon-desc.c:123-152`, `tests/monster/desc.c:390-392`).
/// "It" is `MDESC_DEFAULT`'s, never a blow's (review of #698).
fn unseen_name(who: &[u8]) -> bool {
    let lower: Vec<u8> = who.iter().map(u8::to_ascii_lowercase).collect();
    lower.starts_with(b"something") || lower.starts_with(b"someone")
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
    note_unseen(st, false);
}

/// borg-reborn-rs (#981): `recover_unseen_shot` is on and `msg` is the
/// engine's message for a projection that hit the Borg from a monster it
/// cannot see, or any projection while it is blind:
/// `msg("You are hit by %s!", projections[typ].blind_desc)`
/// (`project-player.c:798-799, 840-842, 886-888`, the only producer of the
/// message). It covers every projection (`projection.txt`'s `blind-desc`:
/// arrows, bolts, breaths, spit), so it says nothing about the distance. A
/// kobold archer out of the light is such a monster: not invisible, only
/// unseen (#981).
pub(crate) fn unseen_shot_message(msg: &[u8]) -> bool {
    param(Param::recover_unseen_shot) != 0 && unseen_shot_text(msg)
}

fn unseen_shot_text(msg: &[u8]) -> bool {
    msg.starts_with(b"You are hit by ") && msg.ends_with(b"!")
}

/// borg-reborn-rs (#981): an unseen monster's projection hit the Borg
/// (`SHOT_BY:`): an unseen attack, as #697's blow.
pub(crate) fn note_unseen_shot(st: &mut BorgState) {
    note_unseen(st, true);
}

/// #697's record of an unseen attack; `ranged`: a projection's hit (#981),
/// not a blow.
fn note_unseen(st: &mut BorgState, ranged: bool) {
    let now = engine::game_turn();
    let (x, y) = (st.borg.c.x, st.borg.c.y);
    let turns = param(Param::recover_unseen_attack_turns);
    let recent = unseen_recent(turns, now, st.borg.unseen_turn);
    let far = engine::distance(loc { x, y }, loc { x: st.borg.unseen_x, y: st.borg.unseen_y })
        >= param(Param::recover_unseen_attack_distance);
    // borg-reborn-rs (review of #998): only a blow struck far from the last
    // attack is news that the attacker moved; a projection hits from where it
    // stands, so a hit after the Borg walked away is not "followed".
    st.borg.unseen_followed = unseen_followed_next(st.borg.unseen_followed, recent, moved_far(far, ranged));
    // A new episode: the window lapsed, or the mode had ended (its end also
    // clears the clock, so `recent` is false then too; review of #998).
    let new_episode = turns > 0 && (!recent || !st.borg.unseen_mode);
    st.borg.unseen_shot_only = episode_shot_only(st.borg.unseen_shot_only, new_episode, ranged);
    st.borg.unseen_turn = now;
    st.borg.unseen_x = x;
    st.borg.unseen_y = y;
    // borg-reborn-rs (#831): the blow is news about where the attacker is:
    // the grids found empty are no longer known to be. A blow after the
    // window lapsed starts a new episode, with its own budgets.
    st.borg.unseen_empty.clear();
    st.borg.unseen_found = None;
    if turns > 0 && !recent {
        st.borg.unseen_detects = 0;
        st.borg.unseen_probes = 0;
        st.borg.unseen_probe = None;
    }
    if turns > 0 && !st.borg.unseen_mode {
        st.borg.unseen_mode = true;
        borg_note(st, &format!("# Mode: invisible attacker near ({y},{x})"));
    }
}

/// borg-reborn-rs (#697): "followed" after a new unseen attack: set when it
/// struck far (at least the distance) from the last one within the window;
/// kept while the attacks stay within the window (it stays set, so the Borg
/// leaves); cleared when the window had lapsed (and on a new level).
fn unseen_followed_next(followed: bool, recent: bool, far: bool) -> bool {
    recent && (followed || far)
}

/// borg-reborn-rs (#981, review of #998): the attack says the attacker moved
/// far: a blow struck far from the last attack. A hit does not.
fn moved_far(far: bool, ranged: bool) -> bool {
    far && !ranged
}

/// borg-reborn-rs (#981, review of #998): the episode's kind after an unseen
/// attack: set when the episode starts (`new_episode`), and lost for good by
/// the first blow in it.
fn episode_shot_only(shot_only: bool, new_episode: bool, ranged: bool) -> bool {
    ranged && (new_episode || shot_only)
}

/// borg-reborn-rs (#981, review of #998): the blind attack on the adjacent
/// grids. A monster in view on a grid the Borg sees is visible unless it is
/// invisible (`mon-util.c:359-388`), and Detect Invisible answers that case;
/// so in an episode of projection hits only, the attacker is known to be
/// adjacent no more than anywhere else, unless the Borg is blind or has no
/// light. A blow in the episode says it is adjacent.
fn probe_fits_episode(shot_only: bool, blind: bool, no_light: bool) -> bool {
    !shot_only || blind || no_light
}

/// borg-reborn-rs (#981, review of #998): an episode of projection hits
/// only ends when a known monster that casts or shoots (`ranged_attack`, its
/// race's spells) dies: a death the Borg saw and located, which may be the
/// attacker's. No other death ends it.
fn shooter_death_ends(mode: bool, shot_only: bool, ranged_attack: u8) -> bool {
    mode && shot_only && ranged_attack != 0
}

/// Called with the kill index of a located death (`DIED:`, before the record
/// is deleted).
pub(crate) fn unseen_monster_died(st: &mut BorgState, k: i32) {
    let kill = st.kills.kills[k as usize];
    if !shooter_death_ends(st.borg.unseen_mode, st.borg.unseen_shot_only, kill.ranged_attack) {
        return;
    }
    end_unseen_episode(st);
    let name = String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx)));
    borg_note(st, &format!("# Mode: over (a {name}, which casts or shoots, died after unseen hits)"));
}

/// The end of an episode: the mode, the clock and the episode's state.
fn end_unseen_episode(st: &mut BorgState) {
    st.borg.unseen_mode = false;
    st.borg.unseen_turn = 0;
    st.borg.unseen_followed = false;
    st.borg.unseen_probe = None;
    st.borg.unseen_found = None;
    st.borg.unseen_shot_only = false;
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
/// Borg is near where it struck, or the attacker has followed it. Noted once
/// a think, as the other rest holds (review of #698).
fn unseen_rest_held(st: &mut BorgState) -> bool {
    let held = unseen_active(st) && (st.borg.unseen_followed || unseen_near(st));
    if held && st.borg.unseen_rest_noted != Some(st.borg.time.now) {
        st.borg.unseen_rest_noted = Some(st.borg.time.now);
        let ago = engine::game_turn() - st.borg.unseen_turn;
        borg_note(st, &format!("# No rest: an unseen attacker struck {ago} game turns ago."));
    }
    held
}

/// borg-reborn-rs (#697): below `percent` of the maximum HP, strictly (0 =
/// off). `max * percent` stays in `i32` (HP up to a few thousand, percent up
/// to 100).
fn unseen_heal_due(percent: i32, cur: i32, max: i32) -> bool {
    hp_below_percent(percent, cur, max)
}

/// Strictly below `percent` of `max` (0 = never).
fn hp_below_percent(percent: i32, cur: i32, max: i32) -> bool {
    percent > 0 && cur * 100 < max * percent
}

/// borg-reborn-rs (#697), `recover_unseen_heal_hp_percent`: while an unseen
/// attack is recent, below this share of the maximum HP, drink a cure: its
/// HP loss counts as no danger, so `borg_heal`'s tests never fire.
fn unseen_heal(st: &mut BorgState) -> bool {
    let percent = param(Param::recover_unseen_heal_hp_percent);
    if !unseen_heal_due(percent, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP)) || !unseen_active(st) {
        return false;
    }
    let healed = borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        // The raw quaff, as #457's `borg_heal_outmatched`: `borg_quaff_crit`
        // keeps the last two and draws a number (review of #698).
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical);
    if healed {
        borg_note(st, "# Healing: an unseen attacker, HP low.");
    }
    healed
}

/// borg-reborn-rs (#982, owner's example: "below N% HP with a fast awake
/// monster known on the level, drink before resting"): an HP rest is due,
/// the HP is below `recover_rest_cure_hp_percent` of the maximum, and an
/// awake monster that can move is known within `recover_rest_cure_distance`
/// grids (in view or not: a record outlives its sight, and its `awake` is
/// the last state the Borg saw), faster than the Borg by at least
/// `recover_rest_cure_speed_edge` (`forecast_monster_speed`, the race's speed
/// plus 10 if it was seen faster, against the Borg's current speed). Its
/// danger here is 0 once it is out of
/// view, so neither `borg_heal` (no heal at danger 0) nor the costly heal
/// above (no Cure Light Wounds) drinks; the rest is broken by its return
/// (the Necromancer of #982 rested at 2/17 HP with 12 CLW, and the crow came
/// back during the rest). Drink one potion this decision, the first the
/// Borg has of Cure Light, Cure Serious and Cure Critical Wounds (the raw
/// quaff, as `unseen_heal`); the next decision looks again. With none, rest
/// as before.
fn cure_before_rest(st: &mut BorgState) -> bool {
    let percent = param(Param::recover_rest_cure_hp_percent);
    if !hp_below_percent(percent, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP)) {
        return false;
    }
    let edge = param(Param::recover_rest_cure_speed_edge);
    let range = param(Param::recover_rest_cure_distance);
    let me = engine::player_combat_speed();
    let kills = &st.kills.kills[1..st.kills.nxt.max(1) as usize];
    let Some(i) = fast_mover_known(
        kills,
        st.borg.c,
        range,
        |kill| fast_enough(kill, me, edge),
        |r| engine::race_flag_has(usize::from(r), crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE),
    ) else {
        return false;
    };
    let kill = kills[i];
    let healed = borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical);
    if healed {
        let name = String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx)));
        let d = engine::distance(st.borg.c, kill.pos);
        let over = crate::borg_caution::forecast_monster_speed(&kill) - me;
        let (cur, max) = (st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP));
        let seen = if kill.seen { "in view" } else { "out of view" };
        borg_note(st, &format!("# Cure before rest: HP {cur}/{max}, a fast awake {name} (speed +{over} over the Borg) known {d} grids away, {seen} (#982)."));
    }
    healed
}

/// #982: the monster is faster than the Borg (`me`, its current speed) by at
/// least `edge`: its race's speed, plus 10 if it was seen faster.
fn fast_enough(kill: &crate::borg_flow_kill::BorgKill, me: i32, edge: i32) -> bool {
    crate::borg_caution::forecast_monster_speed(kill) - me >= edge
}

/// #982: the position in `kills` of the nearest awake monster that can move
/// (`never_moves` is false) and is `fast` enough, within `range` grids (the
/// engine's distance).
fn fast_mover_known(
    kills: &[crate::borg_flow_kill::BorgKill],
    c: loc,
    range: i32,
    fast: impl Fn(&crate::borg_flow_kill::BorgKill) -> bool,
    never_moves: impl Fn(u16) -> bool,
) -> Option<usize> {
    kills
        .iter()
        .enumerate()
        .filter(|(_, kill)| {
            kill.r_idx != 0
                && kill.awake
                && engine::distance(c, kill.pos) <= range
                && !never_moves(kill.r_idx)
                && fast(kill)
        })
        .min_by_key(|(_, kill)| engine::distance(c, kill.pos))
        .map(|(i, _)| i)
}

/// borg-reborn-rs (#831): the directions a blind attack tries, in order, as
/// (direction key, dx, dy): the keypad's, north first (`ddx`/`ddy`).
const PROBE_DIRS: [(i32, i32, i32); 8] =
    [(8, 0, -1), (2, 0, 1), (4, -1, 0), (6, 1, 0), (7, -1, -1), (9, 1, -1), (1, -1, 1), (3, 1, 1)];

/// borg-reborn-rs (#831): the grids where an unseen attacker that has just
/// struck may stand, in the order of `PROBE_DIRS`: the open adjacent grids
/// (`open`) not found empty since the blow.
fn probe_candidates(at: loc, open: impl Fn(loc) -> bool, empty: &[loc]) -> Vec<loc> {
    PROBE_DIRS
        .iter()
        .map(|&(_, dx, dy)| loc { x: at.x + dx, y: at.y + dy })
        .filter(|&g| open(g) && !empty.contains(&g))
        .collect()
}

/// borg-reborn-rs (#831): the grid to attack, or none. The grid where a
/// blind attack already struck something, while it is a candidate; else the
/// first candidate, but only when there are at most `max` of them (in a
/// corridor, not in the open, where it would be guesswork).
fn probe_choice(candidates: &[loc], found: Option<loc>, max: i32) -> Option<loc> {
    if let Some(f) = found.filter(|f| candidates.contains(f)) {
        return Some(f);
    }
    if candidates.len() as i32 > max {
        return None;
    }
    candidates.first().copied()
}

/// borg-reborn-rs (#831): the grid is one `alter` (`+`) may attack: bare
/// floor or a staircase, with no known object, trap or monster. Alter would
/// tunnel, open, close or disarm anything else (`cmd-cave.c:do_cmd_alter_aux`,
/// lines 950-1001: the monster branch comes first, 975-977, so a monster on a
/// grid holding an object or a trap would still be attacked; leaving such a
/// grid out is deliberate caution, not the engine's rule).
fn probe_grid_open(st: &BorgState, g: loc) -> bool {
    use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_LESS, FEAT_MORE};
    if !engine::square_in_bounds_fully(g.x, g.y) {
        return false;
    }
    let ag = st.cave.grid(g.x, g.y);
    let feat = u32::from(ag.feat);
    (feat == FEAT_FLOOR || feat == FEAT_LESS || feat == FEAT_MORE)
        && !ag.trap
        && ag.take == 0
        && ag.kill == 0
}

/// borg-reborn-rs (#831): a blow this recent (game turns) means the attacker
/// is next to the Borg now, so it is worth acting on.
fn unseen_fresh(st: &BorgState) -> bool {
    let age = param(Param::recover_unseen_act_age);
    age > 0 && engine::game_turn() - st.borg.unseen_turn <= age
}

/// borg-reborn-rs (#831): the blind attacks of the episode are used up, and
/// it still strikes (the mode is active).
fn unseen_probes_spent(st: &BorgState) -> bool {
    let tries = param(Param::recover_unseen_probe_tries);
    tries > 0 && st.borg.unseen_probes >= tries
}

/// borg-reborn-rs (#831, owner: "when there is an invisible creature around,
/// switch mode and deal with it"): the "invisible attacker near" mode. It is
/// entered by an unseen blow (`note_unseen_attack`), holds the rest
/// (`unseen_rest_held`) and ends `recover_unseen_attack_turns` game turns
/// after the last blow, or on a new level. In it, a blow that has just
/// struck is dealt with: reveal the attacker (`detect_invisible_now`, at
/// most `recover_unseen_detect_reads` a stretch), else strike where it must
/// be (`unseen_probe`); what is left (walking away, healing, leaving) is
/// `unseen_away` and `unseen_heal`.
fn unseen_deal(st: &mut BorgState) -> bool {
    st.borg.unseen_probe = None;
    if !unseen_active(st) {
        if st.borg.unseen_mode {
            st.borg.unseen_mode = false;
            st.borg.unseen_shot_only = false;
            borg_note(st, "# Mode: invisible attacker gone (no blow for a while)");
        }
        return false;
    }
    if !unseen_fresh(st) {
        return false;
    }
    unseen_detect(st) || unseen_probe(st)
}

/// borg-reborn-rs (#831): read, quaff, use or cast a detection of invisible
/// monsters, at most `recover_unseen_detect_reads` times in the episode.
fn unseen_detect(st: &mut BorgState) -> bool {
    if st.borg.unseen_detects >= param(Param::recover_unseen_detect_reads) {
        return false;
    }
    if !crate::borg_fight_defend::detect_invisible_now(st) {
        return false;
    }
    st.borg.unseen_detects += 1;
    borg_note(st, "# Detecting the invisible attacker.");
    true
}

/// borg-reborn-rs (#831): strike the grid an unseen attacker that has just
/// struck may be in, with `alter`, which attacks a monster there, seen or not,
/// and otherwise only spends the turn ("You spin around."). At most
/// `recover_unseen_probe_tries` a stretch. Not while afraid (the game refuses
/// the blow), confused (it picks its own direction) or when the Borg is hurt
/// enough to drink a cure first (`recover_unseen_heal_hp_percent`).
fn unseen_probe(st: &mut BorgState) -> bool {
    if param(Param::recover_unseen_probe_tries) <= 0
        || !probe_fits_episode(st.borg.unseen_shot_only, st.borg.tr(BI_ISBLIND) != 0, st.borg.tr(BI_LIGHT) == 0)
        || unseen_probes_spent(st)
        || st.borg.tr(BI_ISAFRAID) != 0
        || st.borg.tr(BI_ISCONFUSED) != 0
        || unseen_heal_due(param(Param::recover_unseen_heal_hp_percent), st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP))
    {
        return false;
    }
    let candidates = probe_candidates(st.borg.c, |g| probe_grid_open(st, g), &st.borg.unseen_empty);
    let Some(g) = probe_choice(&candidates, st.borg.unseen_found, param(Param::recover_unseen_probe_max_grids)) else {
        return false;
    };
    let dir = borg_flow_misc::borg_extract_dir(st.borg.c.y, st.borg.c.x, g.y, g.x);
    borg_note(st, &format!("# Striking at ({},{}) for the unseen attacker.", g.y, g.x));
    borg_keypress(st, keycode_t::from(b'+'));
    borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
    st.borg.unseen_probe = Some(g);
    st.borg.unseen_probes += 1;
    true
}

/// borg-reborn-rs (#831): the game's answer to a blind attack, read from the
/// message: "You spin around." the grid was empty; a blow at "something" it
/// is there; "You are too afraid to attack" the Borg cannot; "You have
/// killed it/something/someone." the mode is over (so is the death of a
/// monster, once a detection was read in the episode); the note to write when the
/// episode has just ended, which says whether the unseen attacker is known to
/// be dead.
pub(crate) fn unseen_mode_message(st: &mut BorgState, msg: &[u8]) -> Option<&'static str> {
    if !st.borg.unseen_mode {
        return None;
    }
    if mode_ends_on(msg, st.borg.unseen_detects) {
        let note = mode_end_note(unseen_kill_message(msg), st.borg.unseen_shot_only);
        end_unseen_episode(st);
        return Some(note);
    }
    let Some(g) = st.borg.unseen_probe else { return None };
    if msg == b"You spin around." {
        st.borg.unseen_empty.push(g);
        st.borg.unseen_probe = None;
    } else if crate::borg_caution::unseen_melee_message(msg) {
        st.borg.unseen_found = Some(g);
        st.borg.unseen_probe = None;
    } else if msg.starts_with(b"You are too afraid to attack ") {
        // Do not try again this episode.
        st.borg.unseen_probes = param(Param::recover_unseen_probe_tries);
        st.borg.unseen_probe = None;
    }
    None
}

/// A kill message that ends the mode: that of an unseen monster, or (with a
/// detection read in the episode, `detects`) any of the game's death messages
/// of a monster the Borg did not necessarily see.
/// The note of an episode that ends on a message: a kill of a monster the
/// Borg cannot see, or a death after a detection; an episode of projection
/// hits only has no invisible attacker known (#981, review of #998).
fn mode_end_note(unseen_kill: bool, shot_only: bool) -> &'static str {
    match (unseen_kill, shot_only) {
        (true, true) => "# Mode: unseen attacker killed",
        (true, false) => "# Mode: invisible attacker killed",
        (false, _) => "# Mode: over (a monster died after a detection was read)",
    }
}

#[inline(always)]
fn mode_ends_on(msg: &[u8], detects: i32) -> bool {
    unseen_kill_message(msg) || (detects > 0 && revealed_kill_message(msg))
}

/// The game's message for the melee death of a monster the Borg cannot see
/// (`mon-util.c:player_kill_monster`, lines 1022-1071: "You have killed %s." at
/// 1070, named with `MDESC_DEFAULT` (1029), which gives "it" for a hidden monster,
/// `mon-desc.c:117-121`). "something" and "someone" are accepted defensively; the
/// engine at `574b618c` does not produce them here.
fn unseen_kill_message(msg: &[u8]) -> bool {
    [b"You have killed it.".as_slice(), b"You have killed something.", b"You have killed someone."].contains(&msg)
}

/// borg-reborn-rs (#831, owner: "make it consistent"): a death message of a
/// monster the Borg may have seen ("You have slain/destroyed %s." in melee,
/// "%s dies." or "%s is destroyed." from a distance). It is used once a
/// detection was read at some point in the episode (`unseen_detects` > 0: an
/// attempt, which the engine may still fail), when the attacker is probably
/// revealed and its death ends the mode as the kill of an unseen one does.
/// The engine also gives these forms for an unseen monster shot or thrown at
/// ("It dies.", `player-attack.c:1123,1191`) and for monsters killing each
/// other (`mon-util.c:1227`): the episode then ends too, and the next unseen
/// blow starts a new one.
#[inline(always)]
fn revealed_kill_message(msg: &[u8]) -> bool {
    msg.starts_with(b"You have slain ") || msg.starts_with(b"You have destroyed ")
        || msg.ends_with(b" dies.") || msg.ends_with(b" is destroyed.")
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
    // borg-reborn-rs (#831): the blind attacks are spent and it still strikes.
    if unseen_probes_spent(st) {
        crate::borg_caution::start_leaving(st, "# Leaving (an unseen attacker, not found)");
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

fn rest_sp_floor(st: &BorgState) -> i32 {
    let tenths = if st.borg.tr(BI_CDEPTH) > param(Param::recover_rest_high_depth) {
        param(Param::recover_rest_high_sp_tenths)
    } else {
        param(Param::recover_rest_sp_tenths)
    };
    st.borg.tr(BI_MAXSP) * tenths / 10
}

/// #1211: the danger gate of the "Resting to gain Mana" branch. At full HP
/// the only missing resource is mana, and the general recovery veto at the
/// top of `borg_recover` has already passed at this danger
/// (`recover_danger_divisor`) — so the stricter mana-only gate
/// (`recover_mana_rest_danger_divisor`) refuses exactly the state the
/// recovery walk was armed for: the recorded Priest (Borg261124230918) paced
/// between recovery grids forever because its corridor sat at danger 20-25,
/// above the mana gate's 17 but inside the recovery gate's 42. The rest
/// itself is not more exposing than the cures and preps the same danger
/// allows: it ends at the first monster seen or the first blow. 0 = C.
fn mana_rest_danger_gate(full_hp: bool, curhp: i32, maxhp: i32, avoidance: i32, general_div: i32, mana_div: i32) -> i32 {
    if full_hp && curhp >= maxhp {
        avoidance / general_div
    } else {
        avoidance / mana_div
    }
}

fn mana_rest_sp_floor(st: &BorgState) -> Option<i32> {
    let sp = st.borg.tr(BI_MAXSP);
    // Combat regeneration drains SP on world ticks (game-world.c:648,
    // player-util.c:507). The all-points rest cannot fill that mana pool.
    (sp != 0
        && !engine::player_has(crate::headers::list_player_flags_h::PF_COMBAT_REGEN)
        && (st.borg.tr(BI_CLEVEL) <= param(Param::recover_mana_rest_max_level)
            || st.borg.tr(BI_CDEPTH) >= param(Param::recover_mana_rest_deep_depth)))
    .then(|| sp * param(Param::recover_mana_rest_sp_tenths) / 10)
}

/// #796: a discretionary spell should not make either existing recovery
/// branch start resting solely because of mana.
pub(crate) fn recovery_sp_floor(st: &BorgState) -> i32 {
    rest_sp_floor(st).max(mana_rest_sp_floor(st).unwrap_or(0))
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
mod mana_rest_gate_tests {
    use super::mana_rest_danger_gate;

    /// #1211: at full HP the general recovery gate governs; C's stricter
    /// mana gate stays for everything else and at the off switch.
    #[test]
    fn full_hp_reads_the_general_recovery_gate() {
        // avoidance 170, general divisor 4 (42), mana divisor 10 (17).
        assert_eq!(mana_rest_danger_gate(true, 170, 170, 170, 4, 10), 42, "full HP: the recovery gate");
        assert_eq!(mana_rest_danger_gate(true, 169, 170, 170, 4, 10), 17, "hurt: C's mana gate");
        assert_eq!(mana_rest_danger_gate(false, 170, 170, 170, 4, 10), 17, "0 retains C's gate");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #982: the monster that makes a badly hurt Borg drink before resting:
    /// awake, able to move, fast enough, anywhere on the level; the nearest.
    #[test]
    fn a_fast_awake_monster_on_the_level() {
        use crate::borg_flow_kill::BorgKill;
        let kill = |r_idx: u16, awake: bool, x: i32, speed: u8| BorgKill {
            r_idx,
            awake,
            pos: loc { x, y: 10 },
            speed,
            ..Default::default()
        };
        let me = loc { x: 10, y: 10 };
        // The production predicate, the Borg at normal speed (110), edge 1.
        let fast = |k: &BorgKill| fast_enough(k, 110, 1);
        let mold = |r: u16| r == 9;
        let crow = kill(5, true, 19, 120);
        assert_eq!(fast_mover_known(&[crow], me, 20, fast, mold), Some(0), "the crow of #982, 9 grids away, out of view");
        assert_eq!(fast_mover_known(&[kill(5, true, 40, 120)], me, 20, fast, mold), None, "30 grids away: beyond the range");
        assert_eq!(fast_mover_known(&[kill(5, false, 12, 120)], me, 20, fast, mold), None, "asleep (last seen)");
        assert_eq!(fast_mover_known(&[kill(6, true, 12, 110)], me, 20, fast, mold), None, "as fast as the Borg");
        assert_eq!(fast_mover_known(&[kill(9, true, 12, 120)], me, 20, fast, mold), None, "never moves");
        assert_eq!(fast_mover_known(&[kill(0, true, 12, 120)], me, 20, fast, mold), None, "a dead record");
        let near = kill(7, true, 15, 130);
        assert_eq!(fast_mover_known(&[crow, near], me, 20, fast, mold), Some(1), "the nearest");
        let mut hasted = kill(6, true, 12, 110);
        hasted.observed_fast = true;
        assert!(fast_enough(&hasted, 110, 1), "a normal-speed race seen faster counts +10");
        assert!(!fast_enough(&crow, 120, 1), "a hasted Borg: the crow is no faster");
        assert!(fast_enough(&crow, 110, 10) && !fast_enough(&crow, 110, 11), "edge in engine speed");
        assert!(!hp_below_percent(0, 2, 17), "recover_rest_cure_hp_percent 0: off");
        assert!(hp_below_percent(25, 2, 17), "2/17 HP is below 25%");
        assert!(!hp_below_percent(50, 50, 100), "strictly below");
    }

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
    use super::{
        probe_candidates, probe_choice, unseen_followed_next, unseen_heal_due, unseen_kill_message, mode_ends_on,
        unseen_mode_message, unseen_name, unseen_recent,
    };
    use crate::headers::z_type_h::loc;
    use crate::state::BorgState;

    fn at(x: i32, y: i32) -> loc {
        loc { x, y }
    }

    /// #831: a corridor running east-west: two open neighbours, north first
    /// in the order of the directions; a found-empty grid is left out.
    #[test]
    fn candidates_in_a_corridor() {
        let me = at(10, 10);
        let open = |g: loc| g.y == 10 && (g.x == 9 || g.x == 11);
        assert_eq!(probe_candidates(me, open, &[]), vec![at(9, 10), at(11, 10)], "west before east");
        assert_eq!(probe_candidates(me, open, &[at(9, 10)]), vec![at(11, 10)]);
        assert!(probe_candidates(me, |_| false, &[]).is_empty());
        assert_eq!(probe_candidates(me, |_| true, &[]).len(), 8, "the open: all eight");
    }

    /// #831: the grid struck is kept; else the first candidate, but only
    /// when there are at most `max`; none in the open.
    #[test]
    fn choosing_the_grid_to_strike() {
        let (a, b, c, d) = (at(1, 1), at(2, 2), at(3, 3), at(4, 4));
        assert_eq!(probe_choice(&[a], None, 3), Some(a), "the only candidate");
        assert_eq!(probe_choice(&[a, b, c], None, 3), Some(a), "as many as the maximum");
        assert_eq!(probe_choice(&[a, b, c, d], None, 3), None, "the open: guesswork");
        assert_eq!(probe_choice(&[a, b, c, d], Some(c), 3), Some(c), "it struck something there: again");
        assert_eq!(probe_choice(&[a, b], Some(d), 3), Some(a), "that grid is no longer a candidate");
        assert_eq!(probe_choice(&[], None, 3), None);
    }

    /// #831: the game's answers to a blind attack.
    #[test]
    fn answers_to_a_blind_attack() {
        let mut st = BorgState::default();
        st.borg.unseen_mode = true;
        // Without a blind attack under way, nothing is learnt.
        unseen_mode_message(&mut st, b"You spin around.");
        assert!(st.borg.unseen_empty.is_empty());
        st.borg.unseen_probe = Some(at(5, 5));
        unseen_mode_message(&mut st, b"You spin around.");
        assert_eq!(st.borg.unseen_empty, vec![at(5, 5)]);
        assert_eq!(st.borg.unseen_probe, None);
        st.borg.unseen_probe = Some(at(6, 5));
        unseen_mode_message(&mut st, b"You hit something (3).");
        assert_eq!(st.borg.unseen_found, Some(at(6, 5)));
        assert_eq!(st.borg.unseen_probe, None);
        st.borg.unseen_probe = Some(at(7, 5));
        unseen_mode_message(&mut st, b"You miss someone.");
        assert_eq!(st.borg.unseen_found, Some(at(7, 5)));
        // A blow at something seen is not an answer.
        st.borg.unseen_probe = Some(at(8, 5));
        unseen_mode_message(&mut st, b"You hit the kobold.");
        assert_eq!(st.borg.unseen_found, Some(at(7, 5)));
        assert_eq!(st.borg.unseen_probe, Some(at(8, 5)));
        // Out of the mode, none of this is read.
        st.borg.unseen_mode = false;
        unseen_mode_message(&mut st, b"You spin around.");
        assert_eq!(st.borg.unseen_empty, vec![at(5, 5)]);
    }

    /// #831: afraid, the game refuses the blow (`player-attack.c:750-756`: "You are
    /// too afraid to attack %s!"): the pending attack is dropped, and the
    /// episode's blind attacks are spent (`recover_unseen_probe_tries`).
    #[test]
    fn afraid_ends_the_blind_attacks() {
        let mut st = BorgState::default();
        st.borg.unseen_mode = true;
        st.borg.unseen_probe = Some(at(5, 5));
        unseen_mode_message(&mut st, b"You are too afraid to attack something!");
        assert_eq!(st.borg.unseen_probe, None, "the pending attack is dropped");
    }

    /// #831: a death message of a monster the Borg may have seen ends the episode
    /// only when a detection was read in it (the attacker was probably revealed).
    #[test]
    fn the_death_of_a_revealed_attacker_ends_the_mode() {
        for m in [b"You have slain a clear icky thing.".as_slice(), b"You have destroyed a poltergeist.", b"The clear icky thing dies.", b"The poltergeist is destroyed."] {
            assert!(!mode_ends_on(m, 0), "no detection read: {}", String::from_utf8_lossy(m));
            assert!(mode_ends_on(m, 1), "{}", String::from_utf8_lossy(m));
        }
        assert!(mode_ends_on(b"You have killed it.", 0), "an unseen one's, with or without");
        assert!(!mode_ends_on(b"You hit the kobold.", 2), "a blow is not a death");
        let mut st = BorgState::default();
        st.borg.unseen_mode = true;
        st.borg.unseen_turn = 100;
        st.borg.unseen_detects = 1;
        assert_eq!(unseen_mode_message(&mut st, b"The clear icky thing dies."), Some("# Mode: over (a monster died after a detection was read)"));
        assert!(!st.borg.unseen_mode && st.borg.unseen_turn == 0, "the episode is over");
    }

    /// #981 (review of #998): an episode of projection hits only is not
    /// ended by a death message (the death of a known caster or shooter ends
    /// it, `unseen_monster_died`), and its notes name no invisible attacker.
    #[test]
    fn a_shot_only_episode_and_its_end() {
        use super::{episode_shot_only, mode_end_note, probe_fits_episode, shooter_death_ends};
        let mut st = BorgState::default();
        st.borg.unseen_mode = true;
        st.borg.unseen_turn = 100;
        st.borg.unseen_shot_only = true;
        assert_eq!(unseen_mode_message(&mut st, b"The kobold archer dies."), None, "no detection read: a death message does not end it");
        assert!(st.borg.unseen_mode);
        assert!(shooter_death_ends(true, true, 1), "a caster or shooter died");
        assert!(!shooter_death_ends(true, true, 0), "a grey mold died");
        assert!(!shooter_death_ends(true, false, 1), "a blow's episode");
        assert!(!shooter_death_ends(false, true, 1), "no episode");
        assert_eq!(mode_end_note(true, true), "# Mode: unseen attacker killed");
        assert_eq!(mode_end_note(true, false), "# Mode: invisible attacker killed");
        assert!(episode_shot_only(false, true, true), "a hit starts the episode");
        assert!(!episode_shot_only(true, false, false), "a blow in it: no longer");
        assert!(!episode_shot_only(false, false, true), "a hit after a blow does not make it one");
        assert!(episode_shot_only(true, false, true), "hits only");
        assert!(!episode_shot_only(true, true, false), "a blow starts a new episode");
        assert!(!probe_fits_episode(true, false, false), "hits only, with light: no blind attack");
        assert!(probe_fits_episode(true, true, false), "blind");
        assert!(probe_fits_episode(true, false, true), "no light");
        assert!(probe_fits_episode(false, false, false), "a blow");
    }

    /// #981 (review of #998): a hit far from the last attack is not "followed".
    #[test]
    fn a_hit_is_not_a_follow() {
        use super::moved_far;
        assert!(!moved_far(true, true), "a far hit");
        assert!(moved_far(true, false), "a far blow");
        assert!(!moved_far(false, false), "a near blow");
    }

    /// #831: a melee kill of a monster the Borg cannot see (`mon-util.c:1070`:
    /// "You have killed %s.", "it"; the other two names are accepted defensively); a seen one's is "slain" or "destroyed".
    #[test]
    fn kill_of_the_unseen() {
        for m in [b"You have killed it.".as_slice(), b"You have killed something.", b"You have killed someone."] {
            assert!(unseen_kill_message(m));
        }
        for m in [b"You have slain the kobold.".as_slice(), b"You have destroyed it.", b"You have killed the kobold.", b"It dies."] {
            assert!(!unseen_kill_message(m));
        }
    }

    /// #697 (re-review of #698): the heal threshold is strict.
    #[test]
    fn heal_threshold() {
        assert!(unseen_heal_due(50, 49, 100));
        assert!(!unseen_heal_due(50, 50, 100), "at half, not below it");
        assert!(!unseen_heal_due(0, 1, 100), "0 is off");
        assert!(unseen_heal_due(100, 99, 100), "100: below the maximum");
        assert!(!unseen_heal_due(100, 100, 100));
    }

    /// #981: the engine's message for a bolt from a monster the Borg cannot
    /// see is "You are hit by <blind-desc>!" (`project-player.c:887`, the
    /// blind descriptions of `projection.txt`); a blow's is not.
    #[test]
    fn unseen_shot_messages() {
        use super::{unseen_shot_message, unseen_shot_text};
        for m in [
            b"You are hit by something sharp!".as_slice(),
            b"You are hit by something hard!",
            b"You are hit by fire!",
            b"You are hit by something!",
        ] {
            assert!(unseen_shot_text(m), "{}", String::from_utf8_lossy(m));
        }
        for m in [
            b"Something hits you.".as_slice(),
            b"You hear a soft twang.",
            b"The kobold archer fires a small arrow.",
            b"You are hit by something sharp",
        ] {
            assert!(!unseen_shot_text(m), "{}", String::from_utf8_lossy(m));
        }
        assert!(!unseen_shot_message(b"You are hit by something sharp!"), "off by default (today's play)");
    }

    /// #697: the names the game gives a monster the Borg cannot see.
    #[test]
    fn unseen_names() {
        assert!(unseen_name(b"Something"));
        assert!(unseen_name(b"something"));
        assert!(unseen_name(b"Someone"), "a male or female monster");
        assert!(unseen_name(b"someone"));
        assert!(!unseen_name(b"It"), "MDESC_DEFAULT's, never a blow's");
        assert!(!unseen_name(b"the clear icky thing"));
        assert!(!unseen_name(b"Smeagol"));
    }

    /// #697: the window, in game turns; 0 is off, and no attack is none.
    #[test]
    fn recent_window() {
        assert!(unseen_recent(300, 1_300, 1_000));
        assert!(!unseen_recent(300, 1_301, 1_000));
        assert!(!unseen_recent(0, 1_000, 1_000), "0 is off");
        assert!(!unseen_recent(300, 1_000, 0), "no attack on this level");
    }

    /// #697: "followed" is set by a far attack within the window, stays set
    /// within it, and clears when the window has lapsed.
    #[test]
    fn followed_state() {
        assert!(!unseen_followed_next(false, true, false), "near, within the window");
        assert!(unseen_followed_next(false, true, true), "far, within the window");
        assert!(unseen_followed_next(true, true, false), "stays set within the window");
        assert!(!unseen_followed_next(true, false, true), "the window lapsed");
        assert!(!unseen_followed_next(false, false, true), "a first attack");
    }
}

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
use borg_rs::decision_api::headers::list_tvals_h::{TV_FOOD, TV_MUSHROOM, TV_ROD};
use borg_rs::decision_api::headers::ui_event_h::keycode_t;
use borg_rs::decision_api::headers::z_type_h::loc;
use borg_rs::memory::BorgState;
use borg_rs::threat_facts::{param, Param};

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
    if let Some(k) = crate::stateful::caution::nearest_mover_seen(
        st,
        param(Param::recover_rest_monster_distance),
    ) {
        st.borg.rest_watch_turn = borg_rs::decision_api::engine::game_turn();
        st.borg.rest_watch_depth = borg_rs::decision_api::engine::player_depth();
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
        borg_note(
            st,
            &format!("# No rest: a monster was near {ago} game turns ago."),
        );
    }
    true
}

/// #615 rule a: the rest is held (`Some(game turns since the monster)`),
/// without the note. `borg_think_dungeon` also holds the walk to a recovery
/// grid with it (review of #645), as #614 does with `holding_for_stairs`.
pub(crate) fn holding_for_monster(st: &mut BorgState) -> bool {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark("policy::recovery.holding_for_monster", format!("{:?}", ()));
    }
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
    let impaired = [
        crate::Read::new(st).status().isconfused(),
        crate::Read::new(st).status().isblind(),
        crate::Read::new(st).status().isimage(),
        crate::Read::new(st).status().isstun(),
        crate::Read::new(st).status().isheavystun(),
        crate::Read::new(st).status().isafraid(),
    ]
    .iter()
    .any(|&f| f != 0);
    watch_for_rest(st);
    let ago = borg_rs::decision_api::engine::game_turn() - st.borg.rest_watch_turn;
    let here = st.borg.rest_watch_depth == borg_rs::decision_api::engine::player_depth();
    rest_monster_hold(turns, st.borg.rest_watch_turn, ago, here, impaired).then_some(ago)
}

/// The test of `holding_for_monster`: on, not impaired, a clock set (and
/// not cleared by the monster's death), on this level, within `turns`.
fn rest_monster_hold(
    turns: i32,
    seen_turn: i32,
    ago: i32,
    same_level: bool,
    impaired: bool,
) -> bool {
    !impaired && crate::stateful::caution::recall_watch_holds(turns, seen_turn, ago, same_level)
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
        borg_note(
            st,
            &format!("# No rest: committed to the stairs, {steps} steps to go."),
        );
    }
    true
}

/// borg-reborn-rs (#614, review of #616): the Borg is committed to a
/// staircase, so neither a rest nor a flow to a recovery grid
/// (`borg_flow_recover`, `GOAL_RECOVER`) may take it off the flight.
pub(crate) fn holding_for_stairs(st: &BorgState) -> bool {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    let on_stairs = feat == borg_rs::decision_api::headers::list_terrain_h::FEAT_LESS
        || feat == borg_rs::decision_api::headers::list_terrain_h::FEAT_MORE;
    let (flight, steps) = flight_now(st);
    hold_rest_for_stairs(
        param(Param::recover_rest_reach_stairs),
        flight,
        on_stairs,
        steps,
    )
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
    let recent = st.borg.time.now.saturating_sub(st.borg.flight_time)
        <= param(Param::recover_rest_reach_stairs_memory);
    let here = st.borg.flight_depth == borg_rs::decision_api::engine::player_depth();
    (
        still_fleeing && recent && here && st.borg.flight_time > 0,
        st.borg.flight_steps,
    )
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
pub(crate) fn borg_recover(st: &mut BorgState) -> bool {
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
    if crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp() / param(Param::recover_hurt_hp_divisor)
    {
        q -= param(Param::recover_hurt_chance_bonus);
    }

    // Almost dead
    if crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp() / param(Param::recover_critical_hp_divisor)
    {
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
    if crate::Read::new(st).status().isstun() != 0
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
    if crate::Read::new(st).status().isheavystun() != 0
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
    if crate::Read::new(st).status().iscut() != 0
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
    if crate::Read::new(st).status().ispoisoned() != 0
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
    if crate::Read::new(st).status().isafraid() != 0
        && crate::Read::new(st).curses().crsfear() == 0
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
    if (crate::Read::new(st).status().ishungry() != 0
        || crate::Read::new(st).status().isweak() != 0)
        && q < param(Param::recover_cheap_hunger_chance)
        && (borg_spell(st, REMOVE_HUNGER) || borg_spell(st, HERBAL_CURING))
    {
        return true;
    }

    // Hallucination
    if crate::Read::new(st).status().isimage() != 0
        && q < param(Param::recover_cheap_image_chance)
        && borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
    {
        return true;
    }

    // Heal damage
    if crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp() / param(Param::recover_cheap_heal_hp_divisor)
        && q < param(Param::recover_cheap_heal_chance)
        && p == 0
        && crate::Read::new(st).character().cursp()
            > crate::Read::new(st).character().maxsp() / param(Param::recover_cheap_heal_sp_divisor)
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
    if crate::Read::new(st).status().isfixexp() != 0
        && (borg_activate_item(st, st.act.restore_exp)
            || borg_activate_item(st, st.act.restore_st_lev)
            || borg_activate_item(st, st.act.restore_life)
            || borg_spell(st, REVITALIZE)
            || borg_spell(st, REMEMBRANCE)
            || (crate::Read::new(st).character().curhp() > param(Param::recover_unholy_min_hp)
                && borg_spell(st, UNHOLY_REPRIEVE)))
    {
        return true;
    }

    // cure stat drain with prayer
    if (crate::Read::new(st).status().isfixstr() != 0
        || crate::Read::new(st).status().isfixint() != 0
        || crate::Read::new(st).status().isfixwis() != 0
        || crate::Read::new(st).status().isfixdex() != 0
        || crate::Read::new(st).status().isfixcon() != 0
        || crate::Read::new(st).status().isfixall() != 0)
        && (borg_spell(st, RESTORATION) || borg_spell(st, REVITALIZE))
    {
        return true;
    }

    // cure stat drain with prayer
    if (crate::Read::new(st).status().isfixstr() != 0
        || crate::Read::new(st).status().isfixint() != 0
        || crate::Read::new(st).status().isfixcon() != 0)
        && crate::Read::new(st).character().curhp() > param(Param::recover_unholy_min_hp)
        && borg_spell(st, UNHOLY_REPRIEVE)
    {
        return true;
    }

    /*** Use "expensive" cures ***/

    // Cure stun
    // (the HEAL1 and HEAL2 activations cure only cuts, not stun; patch 0134)
    if crate::Read::new(st).status().isstun() != 0
        && q < param(Param::recover_costly_stun_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_quaff_crit(st, false))
    {
        return true;
    }

    // Cure heavy stun
    if crate::Read::new(st).status().isheavystun() != 0
        && q < param(Param::recover_costly_heavy_stun_chance)
        && (borg_quaff_crit(st, true)
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing))
    {
        return true;
    }

    // Cure cuts
    if crate::Read::new(st).status().iscut() != 0
        && q < param(Param::recover_costly_cut_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_quaff_crit(
                st,
                crate::Read::new(st).character().curhp() < param(Param::recover_critical_ccw_hp),
            ))
    {
        return true;
    }

    // Cure poison
    if crate::Read::new(st).status().ispoisoned() != 0
        && q < param(Param::recover_costly_poison_chance)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_poison)
            || borg_eat(st, TV_FOOD as i32, st.item_val.sv_food_waybread)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_quaff_crit(
                st,
                crate::Read::new(st).character().curhp() < param(Param::recover_critical_ccw_hp),
            )
            || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
            || borg_zap_rod(st, st.item_val.sv_rod_curing)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_activate_item(st, st.act.food_waybread))
    {
        return true;
    }

    // Cure blindness
    // (Waybread cures only poison, so it is not tried here; patch 0134)
    if crate::Read::new(st).status().isblind() != 0
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
    if crate::Read::new(st).status().isconfused() != 0
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
    if crate::Read::new(st).status().isafraid() != 0
        && crate::Read::new(st).curses().crsfear() == 0
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
    if (crate::Read::new(st).status().ishungry() != 0
        || crate::Read::new(st).status().isweak() != 0)
        && q < param(Param::recover_costly_hunger_chance)
        && (borg_read_scroll(st, st.item_val.sv_scroll_satisfy_hunger)
            || borg_activate_item(st, st.act.satisfy))
    {
        return true;
    }

    // Heal damage
    if crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp() / param(Param::recover_costly_heal_hp_divisor)
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
        && crate::Read::new(st).status().isweak() == 0
        && crate::Read::new(st).status().iscut() == 0
        && crate::Read::new(st).status().ishungry() == 0
        && crate::Read::new(st).status().ispoisoned() == 0
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
    let rest_hp = crate::Read::new(st).status().isblind() == 0
        && crate::Read::new(st).status().ispoisoned() == 0
        && crate::Read::new(st).status().iscut() == 0
        && crate::Read::new(st).status().isweak() == 0
        && crate::Read::new(st).status().ishungry() == 0
        && (crate::Read::new(st).status().isconfused() != 0
            || crate::Read::new(st).status().isimage() != 0
            || crate::Read::new(st).status().isafraid() != 0
            || crate::Read::new(st).status().isstun() != 0
            || crate::Read::new(st).status().isheavystun() != 0
            || crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp()
            || crate::Read::new(st).character().cursp() < rest_sp)
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
    let noted = borg_rs::decision_api::borg_recover::note_rest_hunt(rest_hp && hunt_first);
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
            if borg_rs::threat_facts::param(
                borg_rs::threat_facts::Param::recover_rest_arms_see_invis,
            ) != 0
            {
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
    if mana_rest_sp_floor(st).is_some_and(|floor| crate::Read::new(st).character().cursp() < floor)
        && p < i32::from(st.borg.avoidance) / param(Param::recover_mana_rest_danger_divisor)
        && borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && crate::Read::new(st).status().isweak() == 0
        && crate::Read::new(st).status().iscut() == 0
        && crate::Read::new(st).status().ishungry() == 0
        && crate::Read::new(st).status().ispoisoned() == 0
        && crate::Read::new(st).food().food() > param(Param::recover_mana_rest_min_food)
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
    if !breeder_hold(
        on,
        crate::Read::new(st).character().cdepth(),
        impaired(st),
        breeders,
    ) {
        return BreederRest::Rest;
    }
    let ours = st.borg.breeder_leaving && st.borg.goal.leaving;
    let engaged = crate::stateful::caution::breeder_engaged_but_own_leave(st, ours);
    breeder_choice(borg_rs::decision_api::borg_flow_kill::breeder_hunt_now_with(st, engaged))
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
            if withdraw_own_leave(
                st.borg.breeder_leaving,
                g.leaving,
                g.fleeing || g.fleeing_to_town,
            ) {
                st.borg.goal.leaving = false;
                borg_note(st, "# Not leaving: the breeder hunt applies again.");
            }
            st.borg.breeder_leaving = false;
        }
        BreederRest::Leave => {
            if !st.borg.goal.leaving {
                crate::stateful::caution::start_leaving(
                    st,
                    "# Leaving (breeders on the level, no rest)",
                );
                st.borg.breeder_leaving = true;
            }
        }
    }
    if st.borg.breeder_rest_noted != Some(st.borg.time.now) {
        st.borg.breeder_rest_noted = Some(st.borg.time.now);
        let then = if d == BreederRest::Hunt {
            "hunting them"
        } else {
            "leaving"
        };
        let n = st.borg.mon.breeders;
        borg_note(
            st,
            &format!("# No rest: {n} breeders known on the level, {then}."),
        );
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

/// borg-reborn-rs (#697): an unseen attack at game turn `at` (0: none) is
/// recent at `now` under a window of `turns` game turns (0: off).
fn unseen_recent(turns: i32, now: i32, at: i32) -> bool {
    turns > 0 && at > 0 && now - at <= turns
}

/// borg-reborn-rs (#697): the rule applies: on, in the dungeon, an unseen
/// attack is recent.
fn unseen_active(st: &BorgState) -> bool {
    crate::Read::new(st).character().cdepth() > 0
        && unseen_recent(
            param(Param::recover_unseen_attack_turns),
            engine::game_turn(),
            st.borg.unseen_turn,
        )
}

/// borg-reborn-rs (#697): the Borg is within `recover_unseen_attack_distance`
/// grids of the grid where the last unseen attack struck.
fn unseen_near(st: &BorgState) -> bool {
    engine::distance(
        st.borg.c,
        loc {
            x: st.borg.unseen_x,
            y: st.borg.unseen_y,
        },
    ) < param(Param::recover_unseen_attack_distance)
}

/// borg-reborn-rs (#697): no rest while an unseen attack is recent and the
/// Borg is near where it struck, or the attacker has followed it. Noted once
/// a think, as the other rest holds (review of #698).
fn unseen_rest_held(st: &mut BorgState) -> bool {
    let held = unseen_active(st) && (st.borg.unseen_followed || unseen_near(st));
    if held && st.borg.unseen_rest_noted != Some(st.borg.time.now) {
        st.borg.unseen_rest_noted = Some(st.borg.time.now);
        let ago = engine::game_turn() - st.borg.unseen_turn;
        borg_note(
            st,
            &format!("# No rest: an unseen attacker struck {ago} game turns ago."),
        );
    }
    held
}

/// borg-reborn-rs (#697): below `percent` of the maximum HP, strictly (0 =
/// off). `max * percent` stays in `i32` (HP up to a few thousand, percent up
/// to 100).
fn unseen_heal_due(percent: i32, cur: i32, max: i32) -> bool {
    percent > 0 && cur * 100 < max * percent
}

/// borg-reborn-rs (#697), `recover_unseen_heal_hp_percent`: while an unseen
/// attack is recent, below this share of the maximum HP, drink a cure: its
/// HP loss counts as no danger, so `borg_heal`'s tests never fire.
fn unseen_heal(st: &mut BorgState) -> bool {
    let percent = param(Param::recover_unseen_heal_hp_percent);
    if !unseen_heal_due(
        percent,
        crate::Read::new(st).character().curhp(),
        crate::Read::new(st).character().maxhp(),
    ) || !unseen_active(st)
    {
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

/// borg-reborn-rs (#831): the directions a blind attack tries, in order, as
/// (direction key, dx, dy): the keypad's, north first (`ddx`/`ddy`).
const PROBE_DIRS: [(i32, i32, i32); 8] = [
    (8, 0, -1),
    (2, 0, 1),
    (4, -1, 0),
    (6, 1, 0),
    (7, -1, -1),
    (9, 1, -1),
    (1, -1, 1),
    (3, 1, 1),
];

/// borg-reborn-rs (#831): the grids where an unseen attacker that has just
/// struck may stand, in the order of `PROBE_DIRS`: the open adjacent grids
/// (`open`) not found empty since the blow.
fn probe_candidates(at: loc, open: impl Fn(loc) -> bool, empty: &[loc]) -> Vec<loc> {
    PROBE_DIRS
        .iter()
        .map(|&(_, dx, dy)| loc {
            x: at.x + dx,
            y: at.y + dy,
        })
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
    use borg_rs::decision_api::headers::list_terrain_h::{FEAT_FLOOR, FEAT_LESS, FEAT_MORE};
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
    if !crate::stateful::defence::detect_invisible_now(st) {
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
        || unseen_probes_spent(st)
        || crate::Read::new(st).status().isafraid() != 0
        || crate::Read::new(st).status().isconfused() != 0
        || unseen_heal_due(
            param(Param::recover_unseen_heal_hp_percent),
            crate::Read::new(st).character().curhp(),
            crate::Read::new(st).character().maxhp(),
        )
    {
        return false;
    }
    let candidates = probe_candidates(st.borg.c, |g| probe_grid_open(st, g), &st.borg.unseen_empty);
    let Some(g) = probe_choice(
        &candidates,
        st.borg.unseen_found,
        param(Param::recover_unseen_probe_max_grids),
    ) else {
        return false;
    };
    let dir = borg_flow_misc::borg_extract_dir(st.borg.c.y, st.borg.c.x, g.y, g.x);
    borg_note(
        st,
        &format!("# Striking at ({},{}) for the unseen attacker.", g.y, g.x),
    );
    borg_keypress(st, keycode_t::from(b'+'));
    borg_keypress(st, (dir + i32::from(b'0')) as keycode_t);
    st.borg.unseen_probe = Some(g);
    st.borg.unseen_probes += 1;
    true
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
            crate::stateful::caution::start_leaving(st, "# Leaving (an unseen attacker followed)");
        }
        return false;
    }
    // borg-reborn-rs (#831): the blind attacks are spent and it still strikes.
    if unseen_probes_spent(st) {
        crate::stateful::caution::start_leaving(st, "# Leaving (an unseen attacker, not found)");
        return false;
    }
    let wants_rest = crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp()
        || crate::Read::new(st).character().cursp() < crate::Read::new(st).character().maxsp();
    if !wants_rest || !unseen_near(st) {
        return false;
    }
    let at = loc {
        x: st.borg.unseen_x,
        y: st.borg.unseen_y,
    };
    let distance = param(Param::recover_unseen_attack_distance);
    if crate::stateful::caution::walk_away_from(st, at, distance, c"unseen-away") {
        borg_note(
            st,
            &format!(
                "# Walking away from an unseen attacker (struck at ({},{})).",
                at.y, at.x
            ),
        );
        return true;
    }
    if !st.borg.goal.leaving {
        crate::stateful::caution::start_leaving(
            st,
            "# Leaving (an unseen attacker, nowhere to walk to)",
        );
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
    [
        crate::Read::new(st).status().isconfused(),
        crate::Read::new(st).status().isblind(),
        crate::Read::new(st).status().isimage(),
        crate::Read::new(st).status().isstun(),
        crate::Read::new(st).status().isheavystun(),
    ]
    .iter()
    .any(|&f| f != 0)
}

/// borg-reborn-rs (#558): `hunt_before_rest` with the HP test first, so
/// `borg_flow_kill::breeder_hunt_now`'s scan runs only when it passes.
fn hunt_now(st: &mut BorgState) -> bool {
    hunt_before_rest(
        true,
        crate::Read::new(st).character().curhp(),
        crate::Read::new(st).character().maxhp(),
        param(Param::recover_breeder_hunt_hp_percent),
    ) && borg_rs::decision_api::borg_flow_kill::breeder_hunt_now(st)
}

/// borg-reborn-rs (#558): skip resting to hunt breeders while `hunt` (the
/// breeder hunt applies) and HP is at least `hp_percent` of the maximum;
/// 0 = off (C's rest).
fn hunt_before_rest(hunt: bool, cur_hp: i32, max_hp: i32, hp_percent: i32) -> bool {
    hp_percent > 0 && hunt && i64::from(cur_hp) * 100 >= i64::from(max_hp) * i64::from(hp_percent)
}

fn rest_sp_floor(st: &BorgState) -> i32 {
    let tenths =
        if crate::Read::new(st).character().cdepth() > param(Param::recover_rest_high_depth) {
            param(Param::recover_rest_high_sp_tenths)
        } else {
            param(Param::recover_rest_sp_tenths)
        };
    crate::Read::new(st).character().maxsp() * tenths / 10
}

fn mana_rest_sp_floor(st: &BorgState) -> Option<i32> {
    let sp = crate::Read::new(st).character().maxsp();
    (sp != 0
        && (crate::Read::new(st).character().clevel() <= param(Param::recover_mana_rest_max_level)
            || crate::Read::new(st).character().cdepth()
                >= param(Param::recover_mana_rest_deep_depth)))
    .then(|| sp * param(Param::recover_mana_rest_sp_tenths) / 10)
}

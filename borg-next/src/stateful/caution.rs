//! Port of `borg-caution.c`: try to be careful and not die when things are
//! dangerous.
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

// borg-reborn-rs (#219): named caution thresholds; arithmetic/order unchanged.
use crate::stateful::escape::{borg_escape, borg_freedom, borg_surrounded};
use crate::stateful::item_use::borg_satisfy_hunger_before_food;
use borg_rs::decision_api::borg::BorgTime;
use borg_rs::decision_api::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use borg_rs::decision_api::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid};
use borg_rs::decision_api::borg_danger::borg_danger;
use borg_rs::decision_api::borg_fight_defend::borg_defend;
use borg_rs::decision_api::borg_flow::GOAL_FLEE;
use borg_rs::decision_api::borg_flow_stairs::{
    borg_flow_stair_less, borg_flow_stair_more, borg_prep_leave_level_spells,
};
use borg_rs::decision_api::borg_headers::borg_h::BORG_PLAYS_RISKY;
use borg_rs::decision_api::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, BLESS, CURE_POISON, HASTE_SELF, HEALING, HERBAL_CURING, HEROISM, HOLY_WORD,
    MINOR_HEALING, PHASE_DOOR, PORTAL, REMOVE_HUNGER, TELEPORT_LEVEL,
};
use borg_rs::decision_api::borg_inventory::borg_slot;
use borg_rs::decision_api::borg_io::{borg_keypress, borg_note, borg_note_bytes, KC_ENTER};
use borg_rs::decision_api::borg_item_use::{
    borg_activate_failure, borg_activate_item, borg_eat, borg_eat_food_any, borg_equips_staff_fail,
    borg_quaff_crit, borg_quaff_potion, borg_read_scroll, borg_use_staff, borg_use_staff_fail,
    borg_zap_rod,
};
use borg_rs::decision_api::borg_item_val::borg_lookup_kind;
use borg_rs::decision_api::borg_item_wear::borg_backup_swap;
use borg_rs::decision_api::borg_light::{borg_maintain_light, BorgNeed};
use borg_rs::decision_api::borg_magic::{borg_spell, borg_spell_fail, borg_spell_legal};
use borg_rs::decision_api::borg_store_sell::borg_count_sell;
use borg_rs::decision_api::engine;
use borg_rs::decision_api::headers::list_mon_race_flags_h::{
    RF_MULTIPLY, RF_NEVER_MOVE, RF_NO_CONF, RF_UNIQUE,
};
use borg_rs::decision_api::headers::list_options_h::OPT_birth_force_descend;
use borg_rs::decision_api::headers::list_player_timed_h::{TMD_ATT_CONF, TMD_OPP_ACID};
use borg_rs::decision_api::headers::list_terrain_h::{FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE};
use borg_rs::decision_api::headers::list_tvals_h::{TV_MUSHROOM, TV_POTION, TV_ROD, TV_STAFF};
use borg_rs::decision_api::headers::ui_event_h::keycode_t;
use borg_rs::decision_api::headers::z_type_h::loc;
use borg_rs::decision_api::unseen_bumps::UnseenOilShot;
use borg_rs::memory::BorgState;
use borg_rs::threat_facts::{param as p, Param};

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;

/// `GOAL_BORE` (borg-flow.h): move the game along.
const GOAL_BORE: i32 = 6;

/// A literal key (`borg_keypress('R')` etc.).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `I2D(dir)`: the key of a direction.
fn i2d(dir: i32) -> keycode_t {
    (dir + i32::from(b'0')) as keycode_t
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `INVEN_LIGHT` (`z_info->pack_size + 5`, the light slot of `borg_items`).
fn inven_light() -> usize {
    usize::from(engine::z_info().pack_size) + 5
}

/// `borg.trait[BI_CURSP] = v` (the "pretend to have full mana" hacks).
fn set_cursp(st: &mut BorgState, v: i32) {
    super::set_mana(st, v);
}

/// The "am I bouncing between two grids" test of the retreat and back-away
/// scans: healthy, and (x, y) is where the Borg was two steps ago while it
/// was here three steps ago; or the anti-bounce counter is high.
fn bouncing(st: &BorgState, y: i32, x: i32) -> bool {
    let ts = &st.flow.track_step;
    (crate::Read::new(st).character().curhp()
        >= crate::Read::new(st).character().maxhp() * p(Param::caution_bounce_healthy_tenths) / 10
        && (ts.num > 2 && {
            let n2 = (ts.num - 2) as usize;
            let n3 = (ts.num - 3) as usize;
            ts.y[n2] == y && ts.x[n2] == x && ts.y[n3] == st.borg.c.y && ts.x[n3] == st.borg.c.x
        }))
        || i32::from(st.borg.antibounce_count) >= p(Param::caution_bounce_limit)
}

/// `!borg.trait[BI_ATELEPORT] && !borg.trait[BI_AESCAPE]) || rod_good`: may
/// the Borg use its healing rod (it is kept for when it cannot escape)?
fn rod_ok(st: &BorgState, rod_good: bool) -> bool {
    (crate::Read::new(st).supplies().ateleport() == 0
        && crate::Read::new(st).supplies().aescape() == 0)
        || rod_good
}

/// `borg.goal.leaving = true` with its note, if not leaving yet.
pub(crate) fn start_leaving(st: &mut BorgState, why: &str) {
    if !st.borg.goal.leaving {
        borg_note(st, why);
        st.borg.goal.leaving = true;
    }
}

/// `borg.goal.fleeing = true` with its note, if not fleeing yet.
fn start_fleeing(st: &mut BorgState, why: &str) {
    if !st.borg.goal.fleeing {
        borg_note(st, why);
        st.borg.goal.fleeing = true;
    }
}

/// `*_heal = (hp_down * pct / 100)`, at least `min`.
fn heal_amount(hp_down: i32, pct: i32, min: i32) -> i32 {
    let v = hp_down * pct / 100;
    if v < min {
        min
    } else {
        v
    }
}

/// How many stat drains are worth a Life potion against Morgoth
/// (`stats_needing_fix`).
fn stats_needing_fix(st: &BorgState) -> i32 {
    let mut n = 0;
    for f in [
        crate::Read::new(st).status().isfixstr(),
        crate::Read::new(st).status().isfixint(),
        crate::Read::new(st).status().isfixwis(),
        crate::Read::new(st).status().isfixdex(),
        crate::Read::new(st).status().isfixcon(),
    ] {
        if f != 0 {
            n += 1;
        }
    }

    // Special cases get a second vote
    let class = crate::Read::new(st).character().class();
    if class == CLASS_MAGE && crate::Read::new(st).status().isfixint() != 0 {
        n += p(Param::caution_caster_stat_vote);
    }
    if class == CLASS_PRIEST && crate::Read::new(st).status().isfixwis() != 0 {
        n += p(Param::caution_priest_stat_vote);
    }
    if class == CLASS_DRUID && crate::Read::new(st).status().isfixwis() != 0 {
        n += p(Param::caution_druid_stat_vote);
    }
    if class == CLASS_NECROMANCER && crate::Read::new(st).status().isfixint() != 0 {
        n += p(Param::caution_necromancer_stat_vote);
    }
    if class == CLASS_WARRIOR && crate::Read::new(st).status().isfixcon() != 0 {
        n += p(Param::caution_warrior_stat_vote);
    }
    if crate::Read::new(st).character().maxhp() <= p(Param::caution_stat_fix_hp_high)
        && crate::Read::new(st).status().isfixcon() != 0
    {
        n += p(Param::caution_stat_fix_vote_high);
    }
    if crate::Read::new(st).character().maxhp() <= p(Param::caution_stat_fix_hp_low)
        && crate::Read::new(st).status().isfixcon() != 0
    {
        n += p(Param::caution_stat_fix_vote_low);
    }
    if class == CLASS_PRIEST
        && crate::Read::new(st).character().maxsp() < p(Param::caution_priest_stat_fix_sp)
        && crate::Read::new(st).status().isfixwis() != 0
    {
        n += p(Param::caution_priest_stat_fix_votes);
    }
    if class == CLASS_MAGE
        && crate::Read::new(st).character().maxsp() < p(Param::caution_mage_stat_fix_sp)
        && crate::Read::new(st).status().isfixint() != 0
    {
        n += p(Param::caution_mage_stat_fix_votes);
    }
    n
}

/// borg-reborn-rs (#457): the outmatched-healing test, apart from the
/// engine so it can be unit tested. True when the flag is on, the Borg is
/// hurt by at least `min_loss` percent of its maximum HP, and the danger
/// estimate reaches its current HP (so the ordinary `borg_heal` tiers, which
/// need `danger < curhp + heal`, cannot have fired).
fn outmatched_heal_applies(
    enabled: i32,
    curhp: i32,
    maxhp: i32,
    danger: i32,
    min_loss: i32,
) -> bool {
    enabled != 0
        && maxhp > 0
        && curhp < maxhp
        && (maxhp - curhp) * 100 / maxhp >= min_loss
        && danger >= curhp
}

/// borg-reborn-rs (#457): the last stop before `borg_attack`. Reached only
/// at the end of `borg_caution`, after every escape, stair, retreat and
/// back-away option declined, a hurt Borg whose danger estimate reaches its
/// HP drinks the weakest cure potion it holds (CLW, CSW, CCW) instead of
/// attacking, even if the potion cannot "cover" the danger. The danger
/// estimate is a pessimistic worst case; the potion buys a turn.
/// Off (`caution_heal_when_outmatched` = 0) it does nothing.
fn borg_heal_outmatched(st: &mut BorgState, danger: i32) -> bool {
    if !outmatched_heal_applies(
        p(Param::caution_heal_when_outmatched),
        crate::Read::new(st).character().curhp(),
        crate::Read::new(st).character().maxhp(),
        danger,
        p(Param::caution_heal_outmatched_min_loss),
    ) {
        return false;
    }
    // borg-reborn-rs (#842): no cure in contact with a fast-fragile monster.
    if crate::stateful::fast_fragile::heal_banned(st) {
        return false;
    }
    if borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical)
    {
        borg_note(st, "# Healing (outmatched).");
        return true;
    }
    false
}

/// Try healing.
///
/// This function tries to heal the borg before trying to flee.
/// The ez_heal items (*Heal* and Life) are reserved for Morgoth.
/// In severe emergencies the borg can drink an ez_heal item but that is
/// checked in borg_caution().  He should bail out of the fight before
/// using an ez_heal.
///
/// Divides by `BI_MAXHP` like C (0 panics where C would trap).
fn borg_heal(st: &mut BorgState, danger: i32) -> bool {
    let allow_fail = p(Param::caution_heal_spell_fail);
    let avoidance = |st: &BorgState| i32::from(st.borg.avoidance);

    let hp_down =
        crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp();
    let pct_down =
        (crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp()) * 100
            / crate::Read::new(st).character().maxhp();
    let clw_heal = heal_amount(
        hp_down,
        p(Param::caution_clw_estimate_percent),
        p(Param::caution_clw_estimate_min),
    );
    let csw_heal = heal_amount(
        hp_down,
        p(Param::caution_csw_estimate_percent),
        p(Param::caution_csw_estimate_min),
    );
    let ccw_heal = heal_amount(
        hp_down,
        p(Param::caution_ccw_estimate_percent),
        p(Param::caution_ccw_estimate_min),
    );
    // cmw_heal (30%, at least 50) is computed in C but never used.
    let heal_heal = heal_amount(
        hp_down,
        p(Param::caution_heal_estimate_percent),
        p(Param::caution_heal_estimate_min),
    );

    // Quick check for rod success (used later on)
    let mut rod_good = false;
    if borg_slot(st, TV_ROD as i32, st.item_val.sv_rod_healing) != -1 {
        // Reasonable chance of success
        if borg_activate_failure(st, TV_ROD as i32, st.item_val.sv_rod_healing)
            < p(Param::caution_healing_rod_fail)
        {
            rod_good = true;
        }
    }

    // when fighting Morgoth, we want the borg to use Life potion to fix his
    // stats.  So we need to add up the ones that are dropped.
    let stats_needing_fix = stats_needing_fix(st);

    // Heal when confused. This is deadly.
    // This is checked twice, once, here, to see if he is in low danger
    // and again at the end of borg_caution, when all other avenues have
    // failed
    // borg-reborn-rs (#646): no cure next to an immobile confuser, which
    // confuses again at once (borg-cemetery#75), unless the monsters that
    // move are as dangerous as the HP left.
    let hold_cure = confusion_cure_held(st);
    if hold_cure {
        borg_note(
            st,
            "# Not fixing Confusion next to an immobile attacker (#646)",
        );
    }
    if crate::Read::new(st).status().isconfused() != 0 && !hold_cure {
        if pct_down >= p(Param::caution_confused_heal_loss)
            && danger - heal_heal < crate::Read::new(st).character().curhp()
            && borg_quaff_potion(st, st.item_val.sv_potion_healing)
        {
            borg_note(st, "# Fixing Confusion. Level 1");
            return true;
        }
        if pct_down >= p(Param::caution_confused_star_loss)
            && danger
                >= crate::Read::new(st).character().curhp()
                    * p(Param::caution_confused_star_danger_multiplier)
            && (borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_life))
        {
            borg_note(st, "# Fixing Confusion. Level 1.a");
            return true;
        }
        if danger < crate::Read::new(st).character().curhp() + csw_heal
            && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
                || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
                || borg_quaff_crit(st, false)
                || (confusion_clw_cure()
                    && borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
                || borg_quaff_potion(st, st.item_val.sv_potion_healing)
                || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
                || borg_activate_item(st, st.act.cure_confusion)
                || borg_use_staff_fail(st, st.item_val.sv_staff_curing))
        {
            borg_note(st, "# Fixing Confusion. Level 2");
            return true;
        }

        // If my ability to use a teleport staff is really
        // bad, then I should heal up then use the staff.
        // Check for a charged teleport staff
        if borg_equips_staff_fail(st, st.item_val.sv_staff_teleportation) {
            // check my skill, drink a potion
            if borg_activate_failure(st, TV_STAFF as i32, st.item_val.sv_staff_teleportation)
                > p(Param::caution_confused_staff_fail)
                && danger
                    < (avoidance(st) + ccw_heal) * p(Param::caution_confused_cure_danger_tenths)
                        / 10
                && (borg_quaff_crit(st, true)
                    || borg_quaff_potion(st, st.item_val.sv_potion_healing))
            {
                borg_note(st, "# Fixing Confusion. Level 3");
                return true;
            }
            // However, if I am in really big trouble and there is no way
            // I am going to be able to survive another round, take my
            // chances on the staff.
            else if danger > avoidance(st) * p(Param::caution_confused_escape_danger_multiplier) {
                borg_note(st, "# Too scary to fix Confusion. Level 4");
                return false;
            }
        } else {
            // If I do not have a staff to teleport, take the potion
            // and try to fix the confusion
            if borg_quaff_crit(st, true)
                || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
                || (confusion_clw_cure() && borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
                || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            {
                borg_note(st, "# Fixing Confusion. Level 5");
                return true;
            }
        }
    }
    // Heal when blind. This is deadly.
    if crate::Read::new(st).status().isblind() != 0
        && engine::randint0(100) < p(Param::caution_blind_cure_chance)
    {
        // if in extreme danger, use teleport then fix the
        // blindness later.
        // Check for a charged teleport staff
        if danger > avoidance(st) * p(Param::caution_blind_escape_danger_tenths) / 10
            && borg_equips_staff_fail(st, st.item_val.sv_staff_teleportation)
        {
            return false;
        }
        if hp_down >= p(Param::caution_blind_heal_loss)
            && borg_quaff_potion(st, st.item_val.sv_potion_healing)
        {
            return true;
        }
        // Warriors with ESP won't need it so quickly
        if !(crate::Read::new(st).character().class() == CLASS_WARRIOR
            && crate::Read::new(st).character().curhp()
                > crate::Read::new(st).character().maxhp()
                    / p(Param::caution_blind_warrior_hp_divisor)
            && crate::Read::new(st).character().esp() != 0)
            && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
                || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
                || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
                || borg_quaff_crit(st, true)
                || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
                || borg_use_staff_fail(st, st.item_val.sv_staff_curing)
                || borg_quaff_potion(st, st.item_val.sv_potion_healing))
        {
            borg_note(st, "# Fixing Blindness.");
            return true;
        }
    }

    // We generally try to conserve ez-heal pots
    if (crate::Read::new(st).status().isblind() != 0
        || crate::Read::new(st).status().isconfused() != 0)
        && (hp_down >= p(Param::caution_blind_confused_star_loss)
            || (danger
                > crate::Read::new(st).character().curhp()
                    * p(Param::caution_blind_confused_danger_multiplier)
                && hp_down > p(Param::caution_blind_confused_min_loss)))
        && borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
    {
        borg_note(st, "# Fixing Confusion/Blind.");
        return true;
    }

    // Healing and fighting Morgoth.
    if st.borg.near.unique >= 10
        && crate::Read::new(st).character().curhp() <= p(Param::caution_questor_heal_hp)
        && ((crate::Read::new(st).character().curhp() > p(Param::caution_questor_holy_word_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_holy_word_fail)))
            // Choose Life over *Healing* to fix stats
            || (stats_needing_fix >= p(Param::caution_questor_life_stat_votes) && borg_quaff_potion(st, st.item_val.sv_potion_life))
            // Choose Life over Healing if way down on pts
            || (hp_down > p(Param::caution_questor_life_loss)
                && {
                    let sv = st.item_val.sv_potion_star_healing;
                    let k = borg_lookup_kind(st, TV_POTION as i32, sv);
                    st.borg.has[k as usize]
                } <= 0
                && borg_quaff_potion(st, st.item_val.sv_potion_life))
            || borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_activate_item(st, st.act.heal3)
            || (crate::Read::new(st).character().curhp() < p(Param::caution_questor_emergency_holy_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_emergency_holy_fail)))
            || (crate::Read::new(st).character().curhp() > p(Param::caution_questor_safe_holy_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_safe_holy_fail)))
            || borg_spell_fail(st, HEALING, p(Param::caution_questor_healing_fail))
            || borg_quaff_potion(st, st.item_val.sv_potion_life)
            || borg_zap_rod(st, st.item_val.sv_rod_healing))
    {
        borg_note(st, "# Healing in Questor Combat.");
        return true;
    }

    // restore Mana
    // note, blow the staff charges easy because the staff will not last.
    if crate::Read::new(st).character().cursp()
        < crate::Read::new(st).character().maxsp() / p(Param::caution_magi_sp_divisor)
        && engine::randint0(100) < p(Param::caution_magi_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_the_magi)
            || borg_activate_item(st, st.act.staff_magi))
    {
        borg_note(st, "# Use Magi Staff");
        return true;
    }
    // blowing potions is harder
    // NOTE: must have enough mana to keep up or do a HEAL
    if (crate::Read::new(st).character().cursp() < crate::Read::new(st).character().maxsp() / p(Param::caution_restore_mana_sp_divisor)
        || (crate::Read::new(st).character().cursp() < p(Param::caution_restore_mana_sp) && crate::Read::new(st).character().maxsp() > p(Param::caution_restore_mana_maxsp)))
        // use the potion if battling a unique and not too dangerous
        && (st.borg.near.unique >= 10
            || (st.borg.near.unique != 0 && danger < avoidance(st) * p(Param::caution_restore_mana_danger_multiplier))
            || (crate::Read::new(st).supplies().ateleport() + crate::Read::new(st).supplies().aescape() == 0
                && danger > avoidance(st)))
        && (borg_use_staff_fail(st, st.item_val.sv_staff_the_magi)
            || borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana)
            || borg_activate_item(st, st.act.staff_magi))
    {
        borg_note(st, "# Restored My Mana");
        return true;
    }

    // if unhurt no healing needed
    if hp_down == 0 {
        return false;
    }

    // Don't bother healing if not in danger
    if danger == 0
        && crate::Read::new(st).status().ispoisoned() == 0
        && crate::Read::new(st).status().iscut() == 0
    {
        return false;
    }

    // Restoring while fighting Morgoth
    if stats_needing_fix >= p(Param::caution_restore_stat_votes)
        && st.borg.near.unique >= 10
        && crate::Read::new(st).character().curhp() > p(Param::caution_restore_stat_hp)
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_restoring)
            || borg_activate_item(st, st.act.restore_all))
    {
        borg_note(st, "# Trying to fix stats in combat.");
        return true;
    }

    // No further Healing considerations if fighting Questors
    if st.borg.near.unique >= 10 {
        // No further healing considerations right now
        return false;
    }

    // borg-reborn-rs (#842): no cure in contact with a fast-fragile monster.
    if crate::stateful::fast_fragile::heal_banned(st) {
        return false;
    }

    // Heal when wounded a percent of the time
    let mut chance = engine::randint0(100);

    // if we are fighting a unique increase the odds of healing
    if st.borg.near.unique != 0 {
        chance += p(Param::caution_heal_unique_chance_bonus);
    }

    // if danger is close to the hp and healing will help, do it
    if danger >= crate::Read::new(st).character().curhp()
        && danger < crate::Read::new(st).character().maxhp()
    {
        chance += p(Param::caution_heal_lethal_chance_bonus);
    } else if crate::Read::new(st).character().class() != CLASS_PRIEST
        && crate::Read::new(st).character().class() != CLASS_PALADIN
    {
        chance -= p(Param::caution_heal_nonpriest_chance_penalty);
    }

    // Risky Borgs are less likely to heal in the fight
    if st.cfg(BORG_PLAYS_RISKY) != 0 {
        chance -= p(Param::caution_heal_risky_chance_penalty);
    }

    if ((pct_down <= p(Param::caution_heal_band1_max_loss)
        && chance < p(Param::caution_heal_band1_skip_chance))
        || ((p(Param::caution_heal_band2_min_loss)..=p(Param::caution_heal_band2_max_loss))
            .contains(&pct_down)
            && chance < p(Param::caution_heal_band2_skip_chance))
        || ((p(Param::caution_heal_band3_min_loss)..=p(Param::caution_heal_band3_max_loss))
            .contains(&pct_down)
            && chance < p(Param::caution_heal_band3_skip_chance))
        || ((p(Param::caution_heal_band4_min_loss)..=p(Param::caution_heal_band4_max_loss))
            .contains(&pct_down)
            && chance < p(Param::caution_heal_band4_skip_chance))
        || ((p(Param::caution_heal_band5_min_loss)..=p(Param::caution_heal_band5_max_loss))
            .contains(&pct_down)
            && chance < p(Param::caution_heal_band5_skip_chance))
        || (pct_down >= p(Param::caution_heal_band6_min_loss)
            && chance < p(Param::caution_heal_band6_skip_chance)))
        && (crate::Read::new(st).status().isheavystun() == 0
            && crate::Read::new(st).status().isstun() == 0
            && crate::Read::new(st).status().ispoisoned() == 0
            && crate::Read::new(st).status().iscut() == 0)
    {
        return false;
    }

    // Cure light Wounds (20)
    if pct_down >= p(Param::caution_clw_min_loss)
        && (pct_down <= p(Param::caution_clw_max_loss) || crate::Read::new(st).character().clevel() < p(Param::caution_clw_max_level))
        && danger < crate::Read::new(st).character().curhp() + clw_heal
        && clw_heal > danger / p(Param::caution_clw_danger_divisor) // No rope-a-doping
        && (borg_spell_fail(st, MINOR_HEALING, allow_fail)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_activate_item(st, st.act.cure_light))
    {
        borg_note(st, "# Healing Level 1.");
        return true;
    }
    // Cure Serious Wounds (40)
    if pct_down >= p(Param::caution_csw_min_loss)
        && (pct_down <= p(Param::caution_csw_max_loss) || crate::Read::new(st).character().clevel() < p(Param::caution_csw_max_level))
        && danger < crate::Read::new(st).character().curhp() + csw_heal
        && csw_heal > danger / p(Param::caution_csw_danger_divisor) // No rope-a-doping
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_activate_item(st, st.act.cure_serious))
    {
        borg_note(st, "# Healing Level 2.");
        return true;
    }

    // Cure Critical Wounds (60)
    if (p(Param::caution_ccw_min_loss)..=p(Param::caution_ccw_max_loss)).contains(&pct_down)
        && danger < crate::Read::new(st).character().curhp() + ccw_heal
        && ccw_heal > danger / p(Param::caution_ccw_danger_divisor) // No rope-a-doping
        && (borg_activate_item(st, st.act.cure_critical) || borg_quaff_crit(st, false))
    {
        borg_note(st, "# Healing Level 3.");
        return true;
    }

    // If in danger try  one more Cure Critical if it will help
    if danger >= crate::Read::new(st).character().curhp()
        && danger < crate::Read::new(st).character().maxhp()
        && crate::Read::new(st).character().curhp() < p(Param::caution_emergency_ccw_hp)
        && danger < ccw_heal
        && borg_quaff_crit(st, true)
    {
        borg_note(st, "# Healing Level 5.");
        return true;
    }

    // if deep, and low on HP, but in a zero danger spot, drink some CCW to
    // add a few HP before resting
    if crate::Read::new(st).character().cdepth() >= p(Param::caution_deep_ccw_depth)
        && danger < p(Param::caution_deep_ccw_danger)
        && pct_down >= p(Param::caution_deep_ccw_loss)
        && borg_quaff_potion(st, st.item_val.sv_potion_cure_critical)
    {
        borg_note(st, "# Healing Level 5B.");
        return true;
    }

    // Heal step one (200hp)
    if pct_down >= p(Param::caution_heal_device_min_loss)
        && danger < crate::Read::new(st).character().curhp() + heal_heal
        && ((rod_ok(st, rod_good) && borg_zap_rod(st, st.item_val.sv_rod_healing))
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_nonorlybig)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_activate_item(st, st.act.heal3)
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
            || borg_spell_fail(st, HEALING, allow_fail))
    {
        borg_note(st, "# Healing Level 6.");
        return true;
    }

    // Generally continue to heal.  But if we are preparing for the end
    // game uniques, then bail out here in order to save our heal pots.
    // (unless morgoth is dead)
    // Priests won't need to bail, they have good heal spells.
    if crate::Read::new(st).character().maxdepth() >= p(Param::caution_conserve_heal_depth)
        && crate::Read::new(st).status().king() == 0
        && st.borg.near.unique == 0
        && crate::Read::new(st).character().class() != CLASS_PRIEST
    {
        // Bail out to save the heal pots for Morgoth
        return false;
    }

    // Heal step two (300hp)
    if pct_down > p(Param::caution_heal_potion_min_loss)
        && danger < crate::Read::new(st).character().curhp() + heal_heal
        && (borg_use_staff_fail(st, st.item_val.sv_staff_healing)
            || (st.borg.near.evil_unique && borg_spell_fail(st, HOLY_WORD, allow_fail))
            || borg_spell_fail(st, HEALING, allow_fail)
            || (rod_ok(st, rod_good) && borg_zap_rod(st, st.item_val.sv_rod_healing))
            || borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing))
    {
        borg_note(st, "# Healing Level 7.");
        return true;
    }

    // Healing step three (300hp).
    if pct_down > p(Param::caution_heal_all_min_loss)
        && danger < crate::Read::new(st).character().curhp() + heal_heal
        && ((st.borg.near.evil_unique && borg_spell_fail(st, HOLY_WORD, allow_fail))
            || (rod_ok(st, rod_good) && borg_zap_rod(st, st.item_val.sv_rod_healing))
            || borg_spell_fail(st, HEALING, allow_fail)
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_nonorlybig)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_activate_item(st, st.act.heal3))
    {
        borg_note(st, "# Healing Level 8.");
        return true;
    }

    // Healing.  First use of EZ_Heals
    if pct_down > p(Param::caution_heal_unique_reserve_min_loss)
        && danger < crate::Read::new(st).character().curhp() + heal_heal
        && ((st.borg.near.evil_unique && borg_spell_fail(st, HOLY_WORD, allow_fail))
            || borg_spell_fail(st, HEALING, allow_fail)
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
            || (rod_ok(st, rod_good) && borg_zap_rod(st, st.item_val.sv_rod_healing))
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_nonorlybig)
            || borg_activate_item(st, st.act.heal1)
            || borg_activate_item(st, st.act.heal2)
            || borg_activate_item(st, st.act.heal3)
            || (st.borg.near.unique != 0
                && (borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                    || borg_quaff_potion(st, st.item_val.sv_potion_healing)
                    || borg_quaff_potion(st, st.item_val.sv_potion_life))))
    {
        borg_note(st, "# Healing Level 9.");
        return true;
    }

    // Healing final check.  Note that *heal* and Life potions are not
    // wasted.  They are saved for Morgoth and emergencies.  The
    // Emergency check is at the end of borg_caution() which is after the
    // borg_escape() routine.
    if pct_down > p(Param::caution_heal_last_resort_loss)
        && danger > crate::Read::new(st).character().curhp()
        && crate::Read::new(st).supplies().ateleport() + crate::Read::new(st).supplies().aescape()
            <= 0
        && (borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_life))
    {
        borg_note(st, "# Healing Level 10.");
        return true;
    }

    /*** Cures ***/

    // Dont do these in the middle of a fight, teleport out then try it
    if danger > avoidance(st) * p(Param::caution_cure_safe_danger_tenths) / 10 {
        return false;
    }

    // Cure poison when poisoned
    // This was moved from borg_caution.
    if crate::Read::new(st).status().ispoisoned() != 0
        && crate::Read::new(st).character().curhp()
            < crate::Read::new(st).character().maxhp() / p(Param::caution_poison_cure_hp_divisor)
    {
        if borg_spell_fail(st, CURE_POISON, p(Param::caution_poison_cure_fail))
            || borg_spell_fail(st, HERBAL_CURING, p(Param::caution_poison_herbal_fail))
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_poison)
            || borg_activate_item(st, st.act.cure_body)
            || borg_activate_item(st, st.act.cure_critical)
            || borg_activate_item(st, st.act.cure_temp)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_activate_item(st, st.act.cure_full)
            || borg_activate_item(st, st.act.cure_full2)
            || borg_activate_item(st, st.act.cure_nonorlybig)
            || borg_use_staff(st, st.item_val.sv_staff_curing)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_fast_recovery)
            || borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_purging)
            || borg_activate_item(st, st.act.shroom_purging)
            // buy time
            || borg_quaff_crit(st, true)
            || borg_spell_fail(st, HEALING, p(Param::caution_poison_healing_fail))
            || borg_spell_fail(st, HOLY_WORD, p(Param::caution_poison_holy_fail))
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
        {
            borg_note(st, "# Curing.");
            return true;
        }

        // attempt to fix mana then poison on next round
        if (borg_spell_legal(st, CURE_POISON) || borg_spell_legal(st, HERBAL_CURING))
            && (borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
                || borg_activate_item(st, st.act.restore_mana))
        {
            borg_note(st, "# Curing next round.");
            return true;
        }
    }

    // Cure poison when poisoned CRITICAL CHECK
    if crate::Read::new(st).status().ispoisoned() != 0
        && (crate::Read::new(st).character().curhp() < p(Param::caution_poison_critical_hp)
            || crate::Read::new(st).character().curhp()
                < crate::Read::new(st).character().maxhp()
                    / p(Param::caution_poison_critical_hp_divisor))
    {
        let sv_mana = crate::Read::new(st).character().cursp();

        set_cursp(st, crate::Read::new(st).character().maxsp());

        if borg_spell(st, CURE_POISON)
            || borg_spell(st, HERBAL_CURING)
            || borg_spell(st, HOLY_WORD)
            || borg_spell(st, HEALING)
        {
            // verify use of spell
            // borg_keypress('y');

            // Flee!
            borg_note(st, "# Emergency Cure Poison! Gasp!!!....");

            return true;
        }
        set_cursp(st, sv_mana);

        // Quaff healing pots to buy some time- in this emergency.
        if borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        {
            return true;
        }

        // Try to Restore Mana
        if borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana)
        {
            return true;
        }

        // Emergency check on healing.  Borg_heal has already been checked
        // but we did not use our ez_heal potions.  All other attempts to
        // save ourself have failed.  Use the ez_heal if I have it.
        if crate::Read::new(st).character().curhp()
            < crate::Read::new(st).character().maxhp() / p(Param::caution_poison_reserve_hp_divisor)
            && (borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_life)
                || borg_quaff_potion(st, st.item_val.sv_potion_healing))
        {
            borg_note(st, "# Healing. Curing section.");
            return true;
        }

        // borg-reborn-rs (#470): no unknown potion, mushroom or staff in
        // this emergency; an unidentified item is never used.
    }

    // Cure wounds when bleeding, also critical check
    if crate::Read::new(st).status().iscut() != 0
        && (crate::Read::new(st).character().curhp()
            < crate::Read::new(st).character().maxhp() / p(Param::caution_cut_cure_hp_divisor)
            || engine::randint0(100) < p(Param::caution_cut_cure_chance))
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_crit(
                st,
                crate::Read::new(st).character().curhp() < p(Param::caution_cut_critical_ccw_hp),
            )
            || borg_spell(st, MINOR_HEALING)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical))
    {
        return true;
    }
    // bleeding and about to die CRITICAL CHECK
    if crate::Read::new(st).status().iscut() != 0
        && (crate::Read::new(st).character().curhp() < p(Param::caution_cut_critical_hp)
            || crate::Read::new(st).character().curhp()
                < crate::Read::new(st).character().maxhp()
                    / p(Param::caution_cut_critical_hp_divisor))
    {
        let sv_mana = crate::Read::new(st).character().cursp();

        set_cursp(st, crate::Read::new(st).character().maxsp());

        // Quaff healing pots to buy some time- in this emergency.
        if borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        {
            return true;
        }

        // Try to Restore Mana
        if borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana)
        {
            return true;
        }

        // Emergency check on healing.  Borg_heal has already been checked
        // but we did not use our ez_heal potions.  All other attempts to
        // save ourselves have failed.  Use the ez_heal if I have it.
        if crate::Read::new(st).character().curhp()
            < crate::Read::new(st).character().maxhp() / p(Param::caution_cut_reserve_hp_divisor)
            && (borg_quaff_potion(st, st.item_val.sv_potion_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_life))
        {
            borg_note(st, "# Healing.  Bleeding.");
            return true;
        }

        // Cast a spell, go into negative mana
        if borg_spell(st, MINOR_HEALING) {
            // verify use of spell
            // borg_keypress('y');

            // Flee!
            borg_note(st, "# Emergency Wound Patch! Gasp!!!....");

            return true;
        }
        set_cursp(st, sv_mana);

        // borg-reborn-rs (#470): no unknown potion, mushroom or staff in
        // this emergency; an unidentified item is never used.
    }

    // nothing to do
    false
}

/// Notes on where the Borg stands: on a glyph, on stairs. Returns
/// `(on_upstair, on_dnstair)`, which C sets to `false` when the Borg stands
/// on such stairs (and leaves `false` otherwise).
fn note_standing(st: &mut BorgState) -> (bool, bool) {
    let mut on_upstair = false;
    let mut on_dnstair = false;

    // Comment on glyph
    let mut i = 0;
    while i < i32::from(st.glyph.track.num) {
        let iu = i as usize;
        if st.glyph.track.y[iu] == st.borg.c.y && st.glyph.track.x[iu] == st.borg.c.x {
            // if standing on one
            borg_note(st, "# Standing on Glyph");
        }
        i += 1;
    }
    // Comment on stair
    let mut i = 0;
    while i < i32::from(st.stairs.less.num) {
        let iu = i as usize;
        if st.stairs.less.y[iu] == st.borg.c.y && st.stairs.less.x[iu] == st.borg.c.x {
            // if standing on one
            borg_note(st, "# Standing on up-stairs");
            on_upstair = false;
        }
        i += 1;
    }
    // Comment on stair
    let mut i = 0;
    while i < i32::from(st.stairs.more.num) {
        let iu = i as usize;
        if st.stairs.more.y[iu] == st.borg.c.y && st.stairs.more.x[iu] == st.borg.c.x {
            // if standing on one
            borg_note(st, "# Standing on dn-stairs");
            on_dnstair = false;
        }
        i += 1;
    }
    (on_upstair, on_dnstair)
}

/// The distance of the nearest stair of `track` without a monster on it,
/// or -1.
fn nearest_free_stair(st: &BorgState, up: bool) -> i32 {
    let track = if up { &st.stairs.less } else { &st.stairs.more };
    let mut best = -1;
    for i in 0..track.num.max(0) as usize {
        let x = track.x[i];
        let y = track.y[i];

        // How far is the nearest stair
        let j = engine::distance(st.borg.c, loc { x, y });

        // Skip stairs if a monster is on the stair
        if st.cave.grid(x, y).kill != 0 || unseen_bump_blocked(st, x, y) {
            continue;
        }

        // skip the farther ones
        if best <= j && best != -1 {
            continue;
        }

        // track it
        best = j;
    }
    best
}

/// An awake known monster next to the Borg (#751's ranged-first test).
fn awake_monster_adjacent(st: &BorgState) -> bool {
    (1..st.kills.nxt.max(0) as usize).any(|i| {
        let kill = &st.kills.kills[i];
        kill.r_idx != 0 && kill.awake && engine::distance(st.borg.c, kill.pos) <= 1
    })
}

/// `borg_flow_stair_less/more(GOAL_FLEE, ...)` with `borg.status.desperate`
/// set around the call; notes `why` when it worked.
fn desperate_flow(st: &mut BorgState, up: bool, range: DesperateStairRange, why: &str) -> bool {
    let weight = stair_flight_weight(GOAL_FLEE, false);
    // borg-reborn-rs (#736, owner): stairs whose path a dangerous monster is
    // on, or next to, are not reachable: the escapes and the attack decide
    // instead (`caution_stairs_clean_path`; 0 = C).
    if p(Param::caution_stairs_clean_path) != 0 && weight == 0 {
        let percent = p(Param::caution_stairs_block_danger_percent);
        if let Some(k) =
            borg_rs::decision_api::borg_flow_stairs::stair_path_blocker(st, up, percent)
        {
            let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
            borg_note(
                st,
                &format!(
                    "# Stairs not reachable: {} in the way ({})",
                    String::from_utf8_lossy(name),
                    why.trim_start_matches("# ")
                ),
            );
            return false;
        }
    }
    // borg-reborn-rs (#751, owner on 247597266: "he starts moving towards the
    // cutpurse even if it has much better options, such as flask of oil,
    // magic missile"): with a ranged attack and nothing adjacent, the walk
    // does not head toward a dangerous monster within
    // caution_desperate_ranged_first_reach grids of its path; the attack at
    // range decides (0 = C).
    let reach = p(Param::caution_desperate_ranged_first_reach);
    let ranged_reach =
        if reach > 0 && !awake_monster_adjacent(st) && crate::stateful::attack::ranged_means(st) {
            reach
        } else {
            0
        };
    if weight == 0 && ranged_reach > 0 {
        let percent = p(Param::caution_stairs_block_danger_percent);
        if let Some(k) =
            borg_rs::decision_api::borg_flow_stairs::stair_path_ahead(st, up, percent, reach)
        {
            let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
            borg_note(
                st,
                &format!(
                    "# Stairs not reachable: {} ahead, fighting at range ({})",
                    String::from_utf8_lossy(name),
                    why.trim_start_matches("# ")
                ),
            );
            return false;
        }
    }
    st.borg.status.desperate = true;
    let ok = if weight > 0 || matches!(range, DesperateStairRange::Remaining(_)) {
        // A stair that is close across the map may be far away on foot.
        // Keep the proximity limit on the route, including any detour.
        borg_rs::decision_api::borg_flow_stairs::bounded_desperate_flight(
            st,
            up,
            range.moves(),
            ranged_reach,
        )
    } else if up {
        borg_flow_stair_less(st, GOAL_FLEE, false)
    } else {
        borg_flow_stair_more(st, GOAL_FLEE, false, false)
    };
    if ok {
        // Note
        borg_note(st, why);

        st.borg.status.desperate = false;
        return true;
    }
    st.borg.status.desperate = false;
    false
}

/// Desperation: head for stairs (the "Flee on foot" block of
/// `borg_caution`). If you are low level and near the stairs and you can
/// hop onto them in very few steps, try to head to them out of
/// desperation.
fn desperate_for_stairs(st: &mut BorgState, pos_danger: i32) -> bool {
    // borg-reborn-rs (#837, owner): "while confused, don't start a Desperate
    // for Stairs walk": 75% of its steps go astray (`player_confuse_dir`,
    // `player-util.c:1353`; the walk into the wall of the Necromancer's last
    // turns). With a cure in the pack the cures at the end of `borg_caution`
    // come first, unless it is held next to an immobile attacker (#646).
    // At 2, no confused walk at all, cure or not.
    let walk_mode = p(Param::caution_confused_cure_before_walk);
    if walk_mode != 0
        && crate::Read::new(st).status().isconfused() != 0
        && (walk_mode >= 2 || (confusion_cure_carried(st) && !confusion_cure_held(st)))
    {
        borg_note(st, "# Confused: no walk to the stairs (#837)");
        return false;
    }
    // Check for an existing "up stairs"
    let b_j = nearest_free_stair(st, true);

    // Check for an existing "down stairs"
    let b_m = nearest_free_stair(st, false);

    // If you are within a few (3) steps of the stairs
    // and you can take some damage to get there
    // go for it
    let near_up = p(Param::caution_desperate_stair_distance) - 1;
    if b_j <= near_up
        && b_j != -1
        && pos_danger < crate::Read::new(st).character().curhp()
        && desperate_flow(
            st,
            true,
            DesperateStairRange::Nearby(near_up),
            "# Desperate for Stairs (one)",
        )
    {
        return true;
    }

    // If you are next to steps of the stairs go for it
    let adjacent_up = p(Param::caution_desperate_adjacent_up_distance);
    if b_j <= adjacent_up
        && b_j != -1
        && desperate_flow(
            st,
            true,
            DesperateStairRange::Nearby(adjacent_up),
            "# Desperate for Stairs (two)",
        )
    {
        return true;
    }

    // Low level guys tend to waste money reading the recall scrolls
    let scary_up = p(Param::caution_desperate_scary_distance) - 1;
    if b_j <= scary_up
        && b_j != -1
        && st.borg.mon.scary
        && crate::Read::new(st).character().clevel() < p(Param::caution_desperate_scary_level)
    {
        let mut safe = true;

        // do not attempt it if an adjacent monster is faster than me
        for i in 0..8 {
            let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
            let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

            // check for bounds
            if !engine::square_in_bounds(x, y) {
                continue;
            }

            // Monster there ?
            let kill = st.cave.grid(x, y).kill;
            if kill == 0 {
                continue;
            }

            // Access the monster and check it's speed
            // borg-reborn-rs (#747): with caution_desperate_fast_danger_percent,
            // only a faster monster that is also dangerous (a fruit bat
            // next to the Borg is not a reason to stay).
            if i32::from(st.kills.kills[usize::from(kill)].speed)
                > crate::Read::new(st).character().speed()
                && fast_neighbour_counts(st, usize::from(kill))
            {
                safe = false;
            }
        }

        // Don't run from Grip or Fang
        if (crate::Read::new(st).character().cdepth() <= p(Param::caution_desperate_unique_depth)
            && crate::Read::new(st).character().cdepth() != 0
            && st.borg.near.unique != 0)
            || !safe
        {
            // try to take them on, you cant outrun them
        } else if desperate_flow(
            st,
            true,
            DesperateStairRange::Nearby(scary_up),
            "# Desperate for Stairs (three)",
        ) {
            return true;
        }
    }

    // #871: keep a viable up-stair flight before trying an adjacent down stair.
    let adjacent_down = p(Param::caution_desperate_adjacent_down_distance);
    if b_m <= adjacent_down
        && b_m != -1
        && (borg_rs::decision_api::borg_flow_stairs::resume_known_up_stair_flight(st, pos_danger)
            || desperate_flow(
                st,
                false,
                DesperateStairRange::Nearby(adjacent_down),
                "# Desperate for Stairs (four)",
            ))
    {
        return true;
    }

    // borg-reborn-rs (#544, narrowed in #548): in the dungeon, fleeing with
    // no up staircase known, a known down staircase (an escape: the Borg
    // comes back up when not prepared for the depth). Only when cornered
    // (`caution_flee_known_down_cornered`) and within
    // `caution_flee_known_down_max_cost` grids of flow.
    p(Param::caution_flee_known_down_stairs) != 0
        && crate::Read::new(st).character().cdepth() > 0
        && (st.borg.goal.fleeing || st.borg.mon.scary)
        && b_j == -1
        && b_m != -1
        && (p(Param::caution_flee_known_down_cornered) == 0 || flee_down_cornered(st, pos_danger))
        && flee_known_down_flow(st, p(Param::caution_flee_known_down_max_cost))
}

/// Strategic retreat to a "happy" grid in view (the first "Retreat" block
/// of `borg_caution`).
fn retreat(st: &mut BorgState, pos_danger: i32, borg_surround: bool) -> bool {
    let mut b_r = -1;
    let mut b_p = -1;
    let mut b_x = st.borg.c.x;
    let mut b_y = st.borg.c.y;

    // Scan the useful viewable grids
    let mut j = 1;
    while j < i32::from(st.view.view_n) {
        let mut x1 = st.borg.c.x;
        let mut y1 = st.borg.c.y;

        let x2 = i32::from(st.view.view_x[j as usize]);
        let y2 = i32::from(st.view.view_y[j as usize]);
        j += 1;

        // Cant if confused: no way to predict motion
        if crate::Read::new(st).status().isconfused() != 0 {
            continue;
        }

        // Require "floor" grids
        if !borg_cave_floor_bold(st, y2, x2) {
            continue;
        }

        // Try to avoid pillar dancing if at good health
        if bouncing(st, y2, x2) {
            continue;
        }

        // Require "happy" grids (most of the time)
        if !borg_rs::decision_api::borg_flow_misc::borg_happy_grid_bold(st, y2, x2) {
            continue;
        }

        // Track "nearest" grid
        if b_r >= 0 {
            let ay = (y2 - y1).abs();
            let ax = (x2 - x1).abs();

            // Ignore "distant" locations
            if ax > b_r || ay > b_r {
                continue;
            }
        }

        // Reset
        let mut r = 0;

        // Simulate movement
        loop {
            // Obtain direction
            let d = borg_rs::decision_api::borg_flow_misc::borg_goto_dir(st, y1, x1, y2, x2);

            // Verify direction
            if d == 0 || d == 5 {
                break;
            }

            // Track distance
            r += 1;

            // Simulate the step
            y1 += i32::from(engine::ddy(d as usize));
            x1 += i32::from(engine::ddx(d as usize));

            // Lets make one more check that we are not bouncing
            if bouncing(st, y1, x1) {
                break;
            }

            // Require floor
            let ag = *st.cave.grid(x1, y1);
            if !borg_cave_floor_grid(&ag)
                || (u32::from(ag.feat) == FEAT_LAVA
                    && crate::Read::new(st).resistances().ifire() == 0)
            {
                break;
            }

            // Require it to be somewhat close
            if r >= p(Param::caution_retreat_max_distance) {
                break;
            }

            // Check danger of that spot
            let p1 = borg_danger(
                st,
                y1,
                x1,
                p(Param::caution_retreat_danger_horizon),
                true,
                false,
            );
            if p1 >= pos_danger {
                break;
            }

            // make sure it is not dangerous to take the first step; unless
            // surrounded.
            if r == 1 {
                // Not surrounded or surrounded and ignoring
                if !borg_surround || (borg_surround && st.borg.goal.ignoring) {
                    if p1
                        >= crate::Read::new(st).character().curhp()
                            * p(Param::caution_retreat_step_hp_tenths)
                            / 10
                    {
                        break;
                    }

                    // Ought to be worth it
                    if p1 > pos_danger * p(Param::caution_retreat_step_danger_tenths) / 10 {
                        break;
                    }
                } else {
                    // Surrounded, try to back-up
                    if crate::Read::new(st).character().clevel()
                        >= p(Param::caution_retreat_surrounded_level)
                    {
                        let limit = if b_r <= p(Param::caution_retreat_surrounded_distance) {
                            crate::Read::new(st).character().curhp()
                                * p(Param::caution_retreat_surrounded_hp_tenths)
                                / 10
                        } else {
                            crate::Read::new(st).character().curhp()
                        };
                        if p1 >= limit {
                            break;
                        }
                    } else if p1
                        >= crate::Read::new(st).character().curhp()
                            * p(Param::caution_retreat_low_level_hp_multiplier)
                    {
                        break;
                    }
                }

                // "Skip this grid if it is adjacent to a monster": the
                // `break` only leaves this scan, so the test has no effect
                // (kept, as in C).
                for ii in 1..i32::from(st.kills.nxt).max(1) {
                    let kill = &st.kills.kills[ii as usize];

                    // Skip dead monsters
                    if kill.r_idx == 0 {
                        continue;
                    }

                    // Require current knowledge
                    if borg_timer(st, kill.when) > p(Param::caution_retreat_monster_age) {
                        continue;
                    }

                    // Check distance -- 1 grid away
                    if engine::distance(kill.pos, loc { x: x1, y: y1 }) <= 1
                        && i32::from(kill.speed) > crate::Read::new(st).character().speed()
                        && !borg_surround
                    {
                        break;
                    }
                }
            }

            let ag = *st.cave.grid(x1, y1);

            // Skip monsters
            if ag.kill != 0 {
                break;
            }

            // Skip traps
            if ag.trap && !ag.glyph {
                break;
            }

            // Safe arrival
            if x1 == x2 && y1 == y2 {
                // Save distance
                b_r = r;
                b_p = p1;

                // Save location
                b_x = x2;
                b_y = y2;

                // Done
                break;
            }
        }
    }

    // Retreat
    if b_r >= 0 {
        // Save direction
        let b_d = borg_rs::decision_api::borg_flow_misc::borg_goto_dir(
            st,
            st.borg.c.y,
            st.borg.c.x,
            b_y,
            b_x,
        );

        // Set goal
        st.borg.goal.g.x = st.borg.c.x + i32::from(engine::ddx(b_d as usize));
        st.borg.goal.g.y = st.borg.c.y + i32::from(engine::ddy(b_d as usize));

        // Note
        let note = format!(
            "# Retreating to {},{} (distance {}) via {},{} ({} > {})",
            b_y, b_x, b_r, st.borg.goal.g.y, st.borg.goal.g.x, pos_danger, b_p
        );
        borg_note(st, &note);

        // Strategic retreat
        borg_keypress(st, i2d(b_d));

        // Reset my Movement and Flow Goals
        st.borg.goal.type_0 = 0;

        // Success
        return true;
    }
    false
}

/// Lava burns for 100+1d100 fire damage a turn: a Borg that is not immune to
/// fire must never choose it as a step (borg-reborn-rs#458).
fn lava_forbids_step(st: &BorgState, ag: &BorgGrid) -> bool {
    u32::from(ag.feat) == FEAT_LAVA && crate::Read::new(st).resistances().ifire() == 0
}

/// Back away from danger to an adjacent grid (the "Back away" block of
/// `borg_caution`).
fn back_away(st: &mut BorgState, pos_danger: i32, borg_surround: bool) -> bool {
    let mut b_i = -1;
    let mut g_k = 0;
    // Whether the grid being examined is next to a monster (reset per grid)
    let mut adjacent_monster;

    // Current danger
    let mut b_k = pos_danger;

    // Fake the danger down if surrounded so that he can move.
    if borg_surround {
        b_k = b_k * p(Param::caution_back_surrounded_danger_tenths) / 10;
    }

    // Check the freedom
    let mut b_f = borg_freedom(st, st.borg.c.y, st.borg.c.x);

    // Attempt to find a better grid
    for i in 0..8 {
        let x = st.borg.c.x + i32::from(engine::ddx_ddd(i));
        let y = st.borg.c.y + i32::from(engine::ddy_ddd(i));

        // Cant if confused: no way to predict motion
        if crate::Read::new(st).status().isconfused() != 0 {
            continue;
        }

        // Access the grid
        let ag = *st.cave.grid(x, y);

        // Skip walls/doors
        if !borg_cave_floor_grid(&ag) {
            continue;
        }

        // Skip lava grids unless immune to fire (it does 100+1d100 fire
        // damage per turn: borg-reborn-rs#458)
        if lava_forbids_step(st, &ag) {
            continue;
        }

        // Skip monster grids
        if ag.kill != 0 {
            continue;
        }

        // Mega-Hack -- skip stores XXX XXX XXX
        if engine::feat_is_shop(i32::from(ag.feat)) {
            continue;
        }

        // Mega-Hack -- skip traps XXX XXX XXX
        if ag.trap && !ag.glyph {
            continue;
        }

        // If i was here last round and 3 rounds ago, suggesting a "bounce"
        if bouncing(st, y, x) {
            continue;
        }

        // Skip this grid if it is adjacent to a monster.  He will just hit
        // me when I land on that grid.
        adjacent_monster = false;
        for ii in 1..i32::from(st.kills.nxt).max(1) {
            let kill = &st.kills.kills[ii as usize];

            // Skip dead monsters
            if kill.r_idx == 0 {
                continue;
            }

            // Require current knowledge
            if borg_timer(st, kill.when) > p(Param::caution_back_monster_age) {
                continue;
            }

            // Check distance -- 1 grid away
            if engine::distance(kill.pos, loc { x, y }) <= 1 && !borg_surround {
                adjacent_monster = true;
            }

            // Check distance -- 2 grids away and he is faster than me
            if engine::distance(kill.pos, loc { x, y })
                <= p(Param::caution_back_fast_monster_distance)
                && i32::from(kill.speed) > crate::Read::new(st).character().speed()
                && !borg_surround
            {
                adjacent_monster = true;
            }
        }

        // Skip this grid consideration because it is next to a monster
        if adjacent_monster {
            continue;
        }

        // Extract the danger there
        let k = borg_danger(st, y, x, p(Param::caution_back_danger_horizon), true, false);

        // Skip this grid if danger is higher than my HP.
        // Take my chances with fighting.
        if k > i32::from(st.borg.avoidance) {
            continue;
        }

        // Skip this grid if it is not really worth backing up.  Look for a
        // 40% reduction in the danger if higher level.  If the danger of
        // the new grid is close to the danger of my current grid, I'll stay
        // and fight.
        if crate::Read::new(st).character().maxclevel() >= p(Param::caution_back_high_level)
            && k > b_k * p(Param::caution_back_high_level_danger_tenths) / 10
        {
            continue;
        }

        // Skip this grid if it is not really worth backing up.  If the
        // danger of the new grid is close to the danger of my current grid,
        // I'll stay and fight unless I am low level and there is an
        // adjacent monster.
        if crate::Read::new(st).character().maxclevel() < p(Param::caution_back_low_level)
            && !adjacent_monster
            && k > b_k * p(Param::caution_back_low_level_danger_tenths) / 10
        {
            continue;
        }

        // Skip higher danger
        //     note: if surrounded, then b_k has been adjusted to a higher
        // number to make his current grid seem more dangerous.  This will
        // encourage him to Back-Up.
        if k > b_k {
            continue;
        }

        // Record the danger of this preferred grid
        g_k = k;

        // Check the freedom there
        let f = borg_freedom(st, y, x);

        // Danger is the same, so look at the nature of the grid
        if b_k == k {
            // If I am low level, reward backing-up if safe
            if crate::Read::new(st).character().clevel() <= p(Param::caution_back_injured_level)
                && crate::Read::new(st).character().cdepth() != 0
                && (crate::Read::new(st).character().curhp()
                    < crate::Read::new(st).character().maxhp()
                    || crate::Read::new(st).character().cursp()
                        < crate::Read::new(st).character().maxsp())
            {
                // do consider the retreat
            } else {
                // Freedom of my grid is better than the next grid
                // so stay put and fight.
                if b_f > f
                    || crate::Read::new(st).character().cdepth()
                        >= p(Param::caution_back_freedom_depth)
                {
                    continue;
                }
            }
        }

        // Save the info
        b_i = i as i32;
        b_k = k;
        b_f = f;
    }

    // Back away
    if b_i >= 0 {
        let bi = b_i as usize;

        // Set goal
        st.borg.goal.g.x = st.borg.c.x + i32::from(engine::ddx_ddd(bi));
        st.borg.goal.g.y = st.borg.c.y + i32::from(engine::ddy_ddd(bi));

        // Note
        let note = format!(
            "# Backing up to {},{} ({} > {})",
            st.borg.goal.g.x, st.borg.goal.g.y, pos_danger, g_k
        );
        borg_note(st, &note);

        // Back away from danger
        borg_keypress(st, i2d(i32::from(engine::ddd(bi))));

        // Reset my Movement and Flow Goals
        st.borg.goal.type_0 = 0;

        // Success
        return true;
    }
    false
}

/// borg-reborn-rs (#646, owner): a known monster that never moves and
/// attacks with its blows (a mushroom patch, a mold, a floating eye;
/// `monster.txt` `NEVER_MOVE` with a blow and without `NEVER_BLOW`).
fn immobile_attacker(r_idx: usize) -> bool {
    use borg_rs::decision_api::headers::list_mon_race_flags_h::{RF_NEVER_BLOW, RF_NEVER_MOVE};
    // borg-reborn-rs (#726): not the placeholder race past the last one
    // (no blow array), as the rest of the core skips it (`r_max - 1`).
    r_idx != 0
        && engine::r_max() != 0
        && r_idx < engine::r_max() - 1
        && engine::race_flag_has(r_idx, RF_NEVER_MOVE)
        && !engine::race_flag_has(r_idx, RF_NEVER_BLOW)
        && engine::race_blow(r_idx, 0).is_some()
}

/// borg-reborn-rs (#713): a blow effect worse than plain damage: poison,
/// confusion, paralysis, fear, blindness, hallucination, stat or experience
/// drain, theft, disenchantment, elemental damage (`blow_effects.txt`).
fn annoying_effect(name: &[u8]) -> bool {
    name != b"HURT" && name != b"NONE"
}

/// borg-reborn-rs (#713): `caution_ranged_vs_immobile` at 2 or 3 keeps race
/// `r_idx` at range: an immobile attacker (`immobile_attacker`) with an
/// annoying blow at 2 (23 of the 25 in `monster.txt`: 15 at DL10 or shallower,
/// 8 deeper, down to Wiruin at DL80),
/// any at 3. At 1 (#646) none is.
fn immobile_annoying(mode: i32, immobile: bool, annoying: bool) -> bool {
    mode >= 2 && immobile && (mode >= 3 || annoying)
}

/// borg-reborn-rs (#713, owner: "step away from immobile creature with
/// annoying effects and only attack them from a distance, if you have to"):
/// in the dungeon, the Borg keeps race `r_idx` at range: no melee
/// (`borg_thrust_damage_one`), no walk to it (`borg_flow_kill`), and a step
/// out of its reach when next to it (`immobile_step_away`).
pub(crate) fn immobile_kept_at_range(st: &BorgState, r_idx: usize) -> bool {
    let mode = p(Param::caution_ranged_vs_immobile);
    // borg-reborn-rs (#726): the blows are read only for an immobile
    // attacker (it has a blow array); a race without one, such as the
    // placeholder one past the last race, aborted in race_blow_effect_name.
    mode >= 2
        && crate::Read::new(st).character().cdepth() > 0
        && immobile_attacker(r_idx)
        && immobile_annoying(
            mode,
            true,
            (0..4).any(|k| engine::race_blow_effect_name(r_idx, k).is_some_and(annoying_effect)),
        )
}

/// borg-reborn-rs (#838, owner: "a list of monsters to never melee ... and
/// only attack from afar"): race `r_idx` is on the scary list's `ranged`
/// entries for this character (`caution_ranged_only`). Off: no lookup.
pub(crate) fn listed_ranged(st: &BorgState, r_idx: usize) -> bool {
    if p(Param::caution_ranged_only) == 0
        || r_idx == 0
        || r_idx >= engine::r_max().saturating_sub(1)
        || crate::Read::new(st).character().cdepth() == 0
    {
        return false;
    }
    let Some(list) = borg_rs::decision_api::scary_list::selected() else {
        return false;
    };
    let who = borg_rs::decision_api::scary_list::Situation {
        clevel: crate::Read::new(st).character().clevel(),
        class: crate::Read::new(st).character().class(),
        breeder_count: 0,
        breeder_stand: false,
        levels_less: 0,
    };
    borg_rs::decision_api::scary_list::ranged_reason(
        list,
        r_idx,
        || engine::race_base_name(r_idx),
        who,
    )
    .is_some()
}

/// borg-reborn-rs (#713, #838): race `r_idx` is kept at range: an immobile
/// attacker (`caution_ranged_vs_immobile` 2, 3) or a `ranged` entry of the
/// scary list (`caution_ranged_only`): never a melee target, no walk to it,
/// flows keep out of its reach.
pub(crate) fn kept_at_range(st: &BorgState, r_idx: usize) -> bool {
    immobile_kept_at_range(st, r_idx) || listed_ranged(st, r_idx)
}

/// borg-reborn-rs (#713): an immobile creature kept at range is next to the
/// Borg.
fn kept_at_range_next_to(st: &BorgState) -> bool {
    kept_attacker_next_to(st, st.borg.c.y, st.borg.c.x).is_some()
}

/// borg-reborn-rs (#713, review of #717): the first known immobile attacker
/// next to grid (`y`, `x`) that is kept at range, by kill index. A plain
/// damage one listed first must not hide a kept one beside it.
fn kept_attacker_next_to(st: &BorgState, y: i32, x: i32) -> Option<usize> {
    (1..usize::try_from(st.kills.nxt).unwrap_or(0)).find(|&i| {
        let kill = &st.kills.kills[i];
        immobile_kept_at_range(st, usize::from(kill.r_idx))
            && (kill.pos.y - y).abs() <= 1
            && (kill.pos.x - x).abs() <= 1
    })
}

/// The first known immobile attacker next to grid (`y`, `x`), by kill index.
fn immobile_attacker_next_to(st: &BorgState, y: i32, x: i32) -> Option<usize> {
    (1..usize::try_from(st.kills.nxt).unwrap_or(0)).find(|&i| {
        let kill = &st.kills.kills[i];
        immobile_attacker(usize::from(kill.r_idx))
            && (kill.pos.y - y).abs() <= 1
            && (kill.pos.x - x).abs() <= 1
    })
}

/// The danger at the Borg's grid from the monsters that can move and are
/// within two grids (they can reach it before a step and a cure are done;
/// review of #659: the whole level's spell danger kept the rule off on a
/// busy level): an immobile one is left by stepping away.
fn mobile_danger(st: &mut BorgState) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    mobile_danger_at(st, y, x)
}

/// `mobile_danger` for grid (`y`, `x`): what the monsters that move within
/// two grids of it would do there (borg-reborn-rs #837: the ranking of the
/// grids a step away may take).
fn mobile_danger_at(st: &mut BorgState, y: i32, x: i32) -> i32 {
    let mut danger = 0i32;
    for i in 1..i32::from(st.kills.nxt) {
        let kill = &st.kills.kills[i as usize];
        let r_idx = usize::from(kill.r_idx);
        if r_idx == 0
            || engine::race_flag_has(
                r_idx,
                borg_rs::decision_api::headers::list_mon_race_flags_h::RF_NEVER_MOVE,
            )
        {
            continue;
        }
        if (kill.pos.y - y).abs() > 2 || (kill.pos.x - x).abs() > 2 {
            continue;
        }
        danger = danger.saturating_add(borg_rs::decision_api::borg_danger::borg_danger_one_kill(
            st, y, x, 1, i, true, true,
        ));
    }
    danger
}

/// borg-reborn-rs (#646, reviews of #659): whether another step away from the
/// immobile attacker at grid `pos` is allowed on this level, and if so record
/// it. With a ranged attack one step per attacker; confused up to
/// `CONFUSED_STEPS` (a confused step goes astray 75% of the time), counted
/// apart, so the confused tries do not use up the ranged step ("move away,
/// drink cure, throw flask"). Keyed on the grid, as kill indices are reused
/// (six grey mushroom patches took index 18 on one level); a full memory
/// refuses the step (fails closed).
const CONFUSED_STEPS: u8 = 3;
fn step_allowed(
    mon: &mut borg_rs::decision_api::borg::BorgMonInfo,
    level: BorgTime,
    pos: (u8, u8),
    confused: bool,
) -> bool {
    if mon.immobile_stepped_level != level {
        mon.immobile_stepped = [[0; 4]; 16];
        mon.immobile_stepped_level = level;
    }
    let (count, limit) = if confused {
        (2, CONFUSED_STEPS)
    } else {
        (3, 1)
    };
    let used = |e: &[u8; 4]| e[2] > 0 || e[3] > 0;
    if let Some(e) = mon
        .immobile_stepped
        .iter_mut()
        .find(|e| used(e) && (e[0], e[1]) == pos)
    {
        if e[count] >= limit {
            return false;
        }
        e[count] += 1;
        return true;
    }
    match mon.immobile_stepped.iter_mut().find(|e| !used(e)) {
        Some(e) => {
            *e = [pos.0, pos.1, 0, 0];
            e[count] = 1;
            true
        }
        None => false,
    }
}

/// borg-reborn-rs (#837, owner): `caution_confused_clw_cure`. A Potion of
/// Cure Light Wounds takes 20 turns off confusion (`object.txt`
/// `TIMED_DEC:CONFUSED`), which clears a grey mushroom patch's 3 + 1d(level)
/// turns (`mon-blows.c:1001`); in C it cures blindness only.
fn confusion_clw_cure() -> bool {
    p(Param::caution_confused_clw_cure) != 0
}

/// borg-reborn-rs (#837, review): the Borg carries something the cures at the
/// end of `borg_caution` (the chain `caution_confused_cure_before_walk` hands
/// the turn to; see "# Healing.  Confusion.") would use: Clear Mind, Cure
/// Serious Wounds, Cure Critical Wounds (only above `item_use_ccw_min_stock`,
/// as `borg_quaff_crit` keeps the last ones), Cure Light Wounds with
/// `caution_confused_clw_cure`, a Healing, *Healing* or Life potion, a Staff
/// of Healing with a charge (`borg_use_staff_fail`). Keep it the same as that
/// chain. A Staff of Curing is not in it (it is in `borg_heal`'s level 2,
/// behind its danger gate), and an equipped activation is not carried in the
/// pack.
fn confusion_cure_carried(st: &mut BorgState) -> bool {
    let mut cures = vec![
        (TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind),
        (TV_POTION as i32, st.item_val.sv_potion_cure_serious),
        (TV_POTION as i32, st.item_val.sv_potion_healing),
        (TV_POTION as i32, st.item_val.sv_potion_star_healing),
        (TV_POTION as i32, st.item_val.sv_potion_life),
    ];
    if crate::Read::new(st).supplies().accw() >= p(Param::item_use_ccw_min_stock) {
        cures.push((TV_POTION as i32, st.item_val.sv_potion_cure_critical));
    }
    if confusion_clw_cure() {
        cures.push((TV_POTION as i32, st.item_val.sv_potion_cure_light));
    }
    if cures
        .into_iter()
        .any(|(tval, sval)| borg_slot(st, tval, sval) != -1)
    {
        return true;
    }
    // A staff the chain can use has a charge left (`borg_use_staff_fail`).
    let staff = borg_slot(st, TV_STAFF as i32, st.item_val.sv_staff_healing);
    staff != -1 && st.items.items[staff as usize].pval != 0
}

/// borg-reborn-rs (#837): whether `step_allowed` would allow another step
/// away from the attacker at `pos`, without recording one.
fn step_left(
    mon: &borg_rs::decision_api::borg::BorgMonInfo,
    level: BorgTime,
    pos: (u8, u8),
    confused: bool,
) -> bool {
    if mon.immobile_stepped_level != level {
        return true;
    }
    let (count, limit) = if confused {
        (2, CONFUSED_STEPS)
    } else {
        (3, 1)
    };
    let used = |e: &[u8; 4]| e[2] > 0 || e[3] > 0;
    match mon
        .immobile_stepped
        .iter()
        .find(|e| used(e) && (e[0], e[1]) == pos)
    {
        Some(e) => e[count] < limit,
        None => mon.immobile_stepped.iter().any(|e| !used(e)),
    }
}

/// borg-reborn-rs (#646, review of #659): the confusion cure is held next to
/// an immobile attacker, at both cure sites (`borg_heal` and the cures at
/// the end of `borg_caution`), unless the moving monsters near are as
/// dangerous as the HP left.
///
/// borg-reborn-rs (#837, `caution_confusion_cure_when_stuck`): held only
/// while `immobile_step_away` can still step (a qualifying grid, tries left).
/// With none the Borg neither stepped nor cured, and melee went on confused.
fn confusion_cure_held(st: &mut BorgState) -> bool {
    if p(Param::caution_ranged_vs_immobile) == 0 || crate::Read::new(st).status().isconfused() == 0
    {
        return false;
    }
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    let Some(who) =
        kept_attacker_next_to(st, cy, cx).or_else(|| immobile_attacker_next_to(st, cy, cx))
    else {
        return false;
    };
    if mobile_danger(st) >= crate::Read::new(st).character().curhp() {
        return false;
    }
    if p(Param::caution_confusion_cure_when_stuck) != 0 {
        let pos = (
            st.kills.kills[who].pos.y as u8,
            st.kills.kills[who].pos.x as u8,
        );
        // In town `immobile_step_away` never steps.
        let can_step = crate::Read::new(st).character().cdepth() != 0
            && step_left(&st.borg.mon, st.borg.time.level, pos, true)
            && best_step_away(st).is_some();
        if !can_step {
            borg_note(
                st,
                "# No step out of reach of the immobile attacker: curing confusion anyway (#837)",
            );
            return false;
        }
    }
    true
}

/// borg-reborn-rs (#646): whether to step away from an adjacent immobile
/// attacker: confused, or with a ranged attack, and the monsters that move
/// less dangerous than the HP left.
fn step_away_wanted(
    confused: bool,
    missiles: i32,
    mobile_danger: impl FnOnce() -> i32,
    hp: i32,
) -> bool {
    (confused || missiles > 0) && mobile_danger() < hp
}

/// borg-reborn-rs (#646, owner): "always choose ranged against immobile but
/// annoying monsters". With `caution_ranged_vs_immobile`, next to an
/// immobile attacker while confused (it strikes again before a cure is
/// worth anything) or while carrying a ranged attack (`BI_AMISSILES`:
/// flasks below CL15, ammunition), step to a free grid out of its reach;
/// the ranged attack follows from there. Not when the monsters that move
/// are as dangerous as the HP left. A confused step goes astray 75% of
/// the time (`player_confuse_dir`, `player-util.c:1358`); it is tried
/// again on the next turn.
pub(crate) fn immobile_step_away(st: &mut BorgState) -> bool {
    let ranged_rule = p(Param::caution_ranged_vs_immobile) != 0;
    if !ranged_rule || crate::Read::new(st).character().cdepth() == 0 {
        return false;
    }
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    // Review of #717: a kept attacker first, so a plain-damage one beside
    // it does not take its place.
    let Some(who) =
        kept_attacker_next_to(st, cy, cx).or_else(|| immobile_attacker_next_to(st, cy, cx))
    else {
        return false;
    };
    let confused = crate::Read::new(st).status().isconfused() != 0;
    let (missiles, hp) = (
        crate::Read::new(st).ammo().amissiles(),
        crate::Read::new(st).character().curhp(),
    );
    // borg-reborn-rs (#713): a creature kept at range (2 and 3) is stepped
    // away from whether or not the Borg has a ranged attack or is confused.
    let kept = immobile_kept_at_range(st, usize::from(st.kills.kills[who].r_idx));
    let wanted = if kept {
        mobile_danger(st) < hp
    } else {
        ranged_rule && step_away_wanted(confused, missiles, || mobile_danger(st), hp)
    };
    if !wanted {
        return false;
    }
    let Some(i) = best_step_away(st) else {
        return false;
    };
    let pos = (
        st.kills.kills[who].pos.y as u8,
        st.kills.kills[who].pos.x as u8,
    );
    // #713: a creature kept at range allows the confused tries' count of
    // steps (3), not the single ranged step.
    if !step_allowed(&mut st.borg.mon, st.borg.time.level, pos, confused || kept) {
        return false;
    }
    let name = engine::race_name(usize::from(st.kills.kills[who].r_idx));
    let why = if kept {
        ", kept at range (#713)"
    } else if confused {
        ", confused"
    } else {
        ", ranged first"
    };
    borg_note(
        st,
        &format!(
            "# Stepping away from immobile {}{} (#646)",
            String::from_utf8_lossy(name),
            why
        ),
    );
    borg_keypress(st, i2d(i32::from(engine::ddd(i))));
    st.borg.goal.type_0 = 0;
    true
}

/// The grid `immobile_step_away` steps to (an index into the eight
/// directions), if any: a free floor grid with no trap, out of the reach of
/// every immobile attacker, less dangerous than this one. Chosen by
/// `caution_immobile_step_to_exit` (#736) and `caution_immobile_step_mobile_first`
/// (#837); records nothing (`confusion_cure_held` asks it too).
fn best_step_away(st: &mut BorgState) -> Option<usize> {
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    // Review of #659: only a grid less dangerous than this one, and not the
    // grid of a bounce (as the back-away).
    let here = borg_danger(st, cy, cx, 1, true, false);
    // borg-reborn-rs (#736, owner): toward the exit. With
    // `caution_immobile_step_to_exit`, among the grids the step may take, the
    // one with the fewest steps to a staircase wins (then the least danger);
    // 0 = the least danger only.
    let to_exit = p(Param::caution_immobile_step_to_exit) != 0;
    let steps = if to_exit {
        borg_rs::decision_api::borg_flow_stairs::stair_steps_around(st)
    } else {
        [255; 8]
    };
    let mobile_first = p(Param::caution_immobile_step_mobile_first) != 0;
    let mut best: Option<(usize, (i32, i32, i32))> = None;
    for (i, step) in steps.into_iter().enumerate() {
        let x = cx + i32::from(engine::ddx_ddd(i));
        let y = cy + i32::from(engine::ddy_ddd(i));
        let ag = *st.cave.grid(x, y);
        if !borg_cave_floor_grid(&ag)
            || lava_forbids_step(st, &ag)
            || ag.kill != 0
            || (ag.trap && !ag.glyph)
            || bouncing(st, y, x)
        {
            continue;
        }
        if engine::feat_is_shop(i32::from(ag.feat)) || immobile_attacker_next_to(st, y, x).is_some()
        {
            continue;
        }
        let k = borg_danger(st, y, x, 1, true, false);
        if k >= here {
            continue;
        }
        // #837: with `caution_immobile_step_mobile_first`, the danger from the
        // monsters that move there comes before the stairs.
        let mobile = if mobile_first {
            mobile_danger_at(st, y, x)
        } else {
            0
        };
        let key = (mobile, step, k);
        if best.is_none_or(|(_, b)| key < b) {
            best = Some((i, key));
        }
    }
    best.map(|(i, _)| i)
}

/// The common start of the retreat and back-away tests: some danger and
/// not "nasty", or surrounded; not in a Sea of Runes or an anti-summon
/// corridor, not confused, not in a vault, and not too many HP.
///
/// `retreat` selects the strategic-retreat face: it additionally skips the
/// retreat when a lone adjacent monster would die in one blow
/// (`borg_adjacent_kill_in_one_blow`, consumer improvement unit I-001). The
/// back-away face has no such test, as in the C.
fn may_move_away(
    st: &BorgState,
    danger_test: bool,
    nasty: bool,
    borg_surround: bool,
    pos_danger: i32,
    retreat: bool,
) -> bool {
    // borg-reborn-rs (#503): standing against breeders, danger alone does
    // not move the Borg away (it ping-ponged between two grids while worms
    // bit it); being surrounded still does.
    ((danger_test && !nasty && st.borg.no_retreat == 0 && !breeder_stand_here(st))
        || (borg_surround && pos_danger != 0))
        && !st.borg.morgoth_position
        && borg_timer(st, st.borg.time.antisummon) >= p(Param::caution_move_antisummon_delay)
        && crate::Read::new(st).status().isconfused() == 0
        && !engine::square_isvault(st.borg.c.x, st.borg.c.y)
        && crate::Read::new(st).character().curhp() < p(Param::caution_move_max_hp)
        && !(retreat && borg_adjacent_kill_in_one_blow(st))
}

/// A lone adjacent known monster that one blow would kill does not justify
/// retreat: swinging removes the threat outright, while retreating hands the
/// initiative back and lets a flee route walk straight back into reach.
/// Requires exactly one adjacent known kill, so a killable monster never
/// masks a dangerous one.
///
/// (Consumer improvement unit I-001, better-borg-c `4.2.6-impact-v10`.)
fn borg_adjacent_kill_in_one_blow(st: &BorgState) -> bool {
    if crate::Read::new(st).status().isafraid() != 0 {
        return false;
    }

    let mut adjacent = 0;
    let mut killable = 0;

    for i in 1..st.kills.nxt {
        let i = i32::from(i);
        let kill = &st.kills.kills[i as usize];

        if kill.r_idx == 0 {
            continue;
        }
        // C: `kill->r_idx >= z_info->r_max - 1`. `r_max` is a `uint16_t`
        // promoted to `int`, so the comparison is signed; with an empty
        // race table `r_max - 1` is -1 and every kill is skipped.
        if engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        if borg_timer(st, kill.when) > p(Param::caution_one_blow_monster_age) {
            continue;
        }
        if engine::distance(st.borg.c, kill.pos) != 1 {
            continue;
        }

        adjacent += 1;
        if borg_rs::decision_api::borg_fight_attack::borg_thrust_damage_one(st, i)
            >= kill.power as i32
        {
            killable += 1;
        }
    }

    adjacent == 1 && killable == 1
}

/// Be "cautious" and attempt to prevent death or dishonor.
///
/// Strategy:
///
///   (1) Caution
///   (1a) Analyze the situation
///   (1a1) try to heal
///   (1a2) try a defense
///   (1b) Teleport from danger
///   (1c) Handle critical stuff
///   (1d) Retreat to happy grids
///   (1e) Back away from danger
///   (1f) Heal various conditions
///
///   (2) Attack
///   (2a) Simulate possible attacks
///   (2b) Perform optimal attack
///
///   (3) Recover
///   (3a) Recover by spells/prayers
///   (3b) Recover by items/etc
///   (3c) Recover by resting
///
/// When in danger, attempt to "flee" by "teleport" or "recall", and if
/// this is not possible, attempt to "heal" damage, if needed, and else
/// attempt to "flee" by "running".
///
/// Whenever we are in a situation in which, even when fully healed, we
/// could die in a single round, we set the "goal.fleeing" flag, and if
/// we could die in two rounds, we set the "goal.leaving" flag.
///
/// In town, whenever we could die in two rounds if we were to stay still,
/// we set the "goal.leaving" flag.
///
/// We will also take stairs if we happen to be standing on them, and we
/// could die in two rounds.
///
/// (See borg-caution.c for the full commentary.)
/// #759: time in Borg turns, rounded up to complete attack commands.
/// Damage and fallback are RATES in HP per Borg turn, not HP per shot.
/// Per-command damage is rate * duration; duration is turns per primary
/// command (a shot can be fractional). The unlimited melee fallback books
/// whole commands conservatively, even if an early kill saves blow energy
/// (player-attack.c:990-1020). Resource uses are whole commands as well.
// Remove only floating-point roundoff at an integer command boundary.
// This is a numerical error bound, not a gameplay/tuning tolerance.
fn complete_attack_commands(value: f64) -> f64 {
    let nearest = value.round();
    if (value - nearest).abs() <= 8.0 * f64::EPSILON * value.abs().max(1.0) {
        nearest
    } else {
        value.ceil()
    }
}

pub(crate) fn race_turns(
    hp: i32,
    hp_per_turn: f64,
    uses: Option<i32>,
    duration: f64,
    fallback_hp_per_turn: f64,
) -> Option<(f64, i32)> {
    if hp <= 0
        || !hp_per_turn.is_finite()
        || hp_per_turn <= 0.0
        || !duration.is_finite()
        || duration <= 0.0
    {
        return None;
    }
    let hp = f64::from(hp);
    if fallback_hp_per_turn >= hp_per_turn && fallback_hp_per_turn > 0.0 {
        return Some((complete_attack_commands(hp / fallback_hp_per_turn), 0));
    }
    let per_command = hp_per_turn * duration;
    let commands = complete_attack_commands(hp / per_command);
    let Some(uses) = uses else {
        return Some((commands * duration, 0));
    };
    let uses = uses.max(0);
    if commands <= f64::from(uses) {
        return Some((commands * duration, commands as i32));
    }
    if !fallback_hp_per_turn.is_finite() || fallback_hp_per_turn <= 0.0 {
        return None;
    }
    let primary_time = f64::from(uses) * duration;
    let remaining = hp - f64::from(uses) * per_command;
    Some((
        primary_time + complete_attack_commands(remaining / fallback_hp_per_turn),
        uses,
    ))
}

pub(crate) fn wins_race(turns: f64, incoming: f64, hp: i32) -> bool {
    turns.is_finite()
        && turns > 0.0
        && incoming.is_finite()
        && incoming >= 0.0
        && hp > 0
        && turns * incoming < f64::from(hp)
}

/// #758: shortest known/possible route to a grid adjacent to the player.
/// Unknown terrain is permitted by the caller as a conservative bound.
/// Other monsters do not permanently block a route: they may move during
/// Recall. This uses remembered terrain, never the engine's unseen map.
fn recall_contact_steps(
    from: loc,
    player: loc,
    max_moves: i32,
    passable: impl Fn(i32, i32) -> bool,
) -> Option<i32> {
    let adjacent = |p: loc| (p.x - player.x).abs().max((p.y - player.y).abs()) <= 1;
    if adjacent(from) {
        return Some(0);
    }
    let mut queue = std::collections::VecDeque::from([(from, 0)]);
    let mut seen = std::collections::BTreeSet::from([(from.x, from.y)]);
    while let Some((at, steps)) = queue.pop_front() {
        if steps >= max_moves {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let next = loc {
                    x: at.x + dx,
                    y: at.y + dy,
                };
                if next.x <= 0
                    || next.y <= 0
                    || next.x >= AUTO_MAX_X - 1
                    || next.y >= AUTO_MAX_Y - 1
                    || !passable(next.x, next.y)
                    || !seen.insert((next.x, next.y))
                {
                    continue;
                }
                if adjacent(next) {
                    return Some(steps + 1);
                }
                queue.push_back((next, steps + 1));
            }
        }
    }
    None
}

/// Ranged exposure needs line of fire OR a possible melee route. A
/// projectable immobile caster can attack even without a walking route.
fn recall_attack_steps(contact: Option<i32>, has_spells: bool, projectable: bool) -> Option<i32> {
    if has_spells && (projectable || contact.is_some()) {
        Some(0)
    } else {
        contact
    }
}

fn recall_exposure(
    damage_per_turn: f64,
    player_energy: f64,
    monster_energy: f64,
    ticks: i32,
    steps: i32,
) -> f64 {
    let duration = f64::from(ticks) * player_energy / 10.0;
    let travel = f64::from(steps.max(0)) * player_energy / monster_energy.max(1.0);
    damage_per_turn * (duration - travel).max(0.0)
}

fn recall_forecast_allows(contact_hp: f64, unknown_risks: i32, hp: i32) -> bool {
    unknown_risks == 0 && contact_hp.is_finite() && contact_hp >= 0.0 && contact_hp < f64::from(hp)
}

/// #758: only the flight's Recall entry point calls this. Evaluate every
/// known monster, including sleepers that may wake, and sum exposure. Doors
/// and walls obey known race movement flags. Casters that can fire now have
/// no approach discount. The maximum delay is conservative, not knowledge
/// of the engine's next random roll (effect-handler-general.c:1145).
pub(crate) fn flight_recall_survivable(st: &mut BorgState) -> bool {
    if p(Param::caution_recall_survival) == 0 {
        return true;
    }
    let ticks = p(Param::caution_recall_survival_delay);
    let player_energy = f64::from(engine::extract_energy(
        engine::player_combat_speed().clamp(0, 199) as usize,
    ))
    .max(1.0);
    let mut exposure = 0.0;
    let mut unknown_risks = 0;
    let (mut awake, mut asleep, mut reachable) = (0, 0, 0);
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        let r = usize::from(kill.r_idx);
        if r == 0 || r >= engine::r_max().saturating_sub(1) {
            continue;
        }
        if !kill.awake {
            asleep += 1;
        } else {
            awake += 1;
        }
        let monster_energy = f64::from(engine::extract_energy(
            forecast_monster_speed(&kill).clamp(0, 199) as usize,
        ))
        .max(1.0);
        let flag = |f| engine::race_flag_has(r, f);
        let never_move = flag(RF_NEVER_MOVE);
        let pass_wall = flag(borg_rs::decision_api::headers::list_mon_race_flags_h::RF_PASS_WALL)
            || flag(borg_rs::decision_api::headers::list_mon_race_flags_h::RF_KILL_WALL);
        let open_door = flag(borg_rs::decision_api::headers::list_mon_race_flags_h::RF_OPEN_DOOR)
            || flag(borg_rs::decision_api::headers::list_mon_race_flags_h::RF_BASH_DOOR);
        let steps = recall_contact_steps(
            kill.pos,
            st.borg.c,
            if never_move {
                0
            } else {
                (f64::from(ticks) * monster_energy / 10.0).ceil() as i32
            },
            |x, y| {
                let ag = st.cave.grid(x, y);
                let feat = u32::from(ag.feat);
                if feat == borg_rs::decision_api::headers::list_terrain_h::FEAT_PERM {
                    return false;
                }
                if feat == borg_rs::decision_api::headers::list_terrain_h::FEAT_CLOSED {
                    return open_door || pass_wall;
                }
                feat == FEAT_NONE || borg_cave_floor_grid(ag) || pass_wall
            },
        );
        // Literal contact HP uses the actual walking route. A harmless
        // HEAL/SCARE flag does not turn a distant bite into a ranged hit.
        let forecast = fight_incoming_with(st, i, true);
        let projectable = forecast.ranged_risk
            && borg_rs::decision_api::borg_projection::borg_projectable_pure(
                st,
                kill.pos.y,
                kill.pos.x,
                st.borg.c.y,
                st.borg.c.x,
            );
        let Some(risk_steps) = recall_attack_steps(steps, forecast.ranged_risk, projectable) else {
            continue;
        };
        reachable += 1;
        if forecast.unknown_damage
            && recall_exposure(1.0, player_energy, monster_energy, ticks, risk_steps) > 0.0
        {
            unknown_risks += 1;
        }
        if let Some(contact_steps) = steps {
            exposure += recall_exposure(
                forecast.hp_per_turn,
                player_energy,
                monster_energy,
                ticks,
                contact_steps,
            );
        }
    }
    let allowed = recall_forecast_allows(
        exposure,
        unknown_risks,
        crate::Read::new(st).character().curhp(),
    );
    if allowed && asleep > 0 {
        borg_note(st, &format!("# Flight Recall forecast: known {awake} awake + {asleep} asleep; {reachable} reachable among them; {exposure:.2} modelled contact HP, HP {}: allowed (#758)", crate::Read::new(st).character().curhp()));
    }
    if unknown_risks > 0 {
        borg_note(st, &format!("# Refusing flight Recall: unmodelled spell/status risk from {unknown_risks} reachable monsters; modelled contact exposure {exposure:.2} HP during {ticks} world ticks, HP {} (#758)", crate::Read::new(st).character().curhp()));
    } else if !allowed {
        borg_note(st, &format!("# Refusing flight Recall: modelled contact exposure {exposure:.2} HP during {ticks} world ticks, HP {} (#758)", crate::Read::new(st).character().curhp()));
    }
    allowed
}

/// #759/#758: quantities of HP and unmodelled risk are separate. A native
/// danger score includes fear prices and must never be multiplied by time
/// or compared to HP as if it were damage.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct IncomingDamage {
    pub hp_per_turn: f64,
    pub unknown_damage: bool,
    pub unknown_fight: bool,
    pub ranged_risk: bool,
}

/// mon-move.c:1929-1934 (574b618c): Haste adds ten. Do not assume Slow
/// persists during a future fight or the Recall delay.
pub(crate) fn forecast_monster_speed(
    kill: &borg_rs::decision_api::borg_flow_kill::BorgKill,
) -> i32 {
    i32::from(kill.speed) + if kill.observed_fast { 10 } else { 0 }
}

// Recall is already pending during exposure: these non-damaging spells do
// not stop its countdown. They CAN change the ability to execute attacks,
// so a winning fight with spells still remains uncertified. monster_spell.txt
// :655-663,894-927,959-977,1056-1065 at 574b618c defines these effects.
// Teleporting, HASTE, summons and damaging spells are
// not silently priced as HP: their unmodelled contribution is a risk flag.
fn recall_spell_has_unknown_damage(spell: u32) -> bool {
    use borg_rs::decision_api::headers::list_mon_spells_h::*;
    !matches!(
        spell,
        RSF_HEAL | RSF_HEAL_KIN | RSF_SCARE | RSF_BLIND | RSF_CONF | RSF_DRAIN_MANA | RSF_FORGET
    )
}

fn modelled_blow(effect: &[u8]) -> bool {
    matches!(
        effect,
        b"HURT" | b"ACID" | b"ELEC" | b"FIRE" | b"COLD" | b"NONE"
    )
}

fn blow_hp_per_action(average: f64, chance: i32, energy_ratio: f64, vulnerable: bool) -> f64 {
    // project-player.c:72-73: elemental vulnerability multiplies by 4/3.
    // Ignore AC, resistance, immunity and damage reduction conservatively;
    // element melee uses max(physical, elemental), not their sum
    // (mon-blows.c:442-460). RF_HURT/IM in :311-365 belongs to the
    // monster-versus-monster target path, not a blow against the player.
    // Neither fear nor item-damage prices are HP.
    average.max(0.0) * f64::from(chance) / 10_000.0
        * energy_ratio
        * if vulnerable { 4.0 / 3.0 } else { 1.0 }
}

pub(crate) fn fight_incoming(st: &BorgState, i: usize) -> IncomingDamage {
    fight_incoming_with(st, i, false)
}

/// Literal expected contact HP/Borg turn plus separate uncertainty flags.
/// Unknown spell/status HP prevents a win certificate and makes Recall
/// refuse explicitly for unmodelled risk, never a fabricated HP number.
pub(crate) fn fight_incoming_with(st: &BorgState, i: usize, wake: bool) -> IncomingDamage {
    let kill = st.kills.kills[i];
    if kill.r_idx == 0 || (!kill.awake && !wake) {
        return IncomingDamage::default();
    }
    let r = usize::from(kill.r_idx);
    let race = engine::race(r);
    let energy = |s: i32| f64::from(engine::extract_energy(s.clamp(0, 199) as usize)).max(1.0);
    let ratio = energy(forecast_monster_speed(&kill)) / energy(engine::player_combat_speed());
    let mut out = IncomingDamage {
        unknown_fight: kill.ranged_attack != 0,
        ..IncomingDamage::default()
    };
    for b in 0..usize::from(engine::z_info().mon_blows_max) {
        let Some(blow) = engine::race_blow(r, b) else {
            break;
        };
        let effect = blow.effect_name.to_bytes();
        out.unknown_fight |= effect != b"HURT";
        out.unknown_damage |= !modelled_blow(effect);
        let vulnerable = match effect {
            b"ACID" => crate::Read::new(st).resistances().vacid() != 0,
            b"ELEC" => crate::Read::new(st).resistances().velec() != 0,
            b"FIRE" => crate::Read::new(st).resistances().vfire() != 0,
            b"COLD" => crate::Read::new(st).resistances().vcold() != 0,
            _ => false,
        };
        let chance = crate::stateful::attack::hit_permyriad(
            3 * race.level.max(1) + blow.hit_power,
            crate::Read::new(st).combat().armor(),
        );
        let average =
            blow_average_tenths(blow.base, blow.dice, blow.sides, blow.m_bonus, race.level) as f64
                / 10.0;
        out.hp_per_turn += blow_hp_per_action(average, chance, ratio, vulnerable);
    }
    use borg_rs::decision_api::headers::list_mon_spells_h::RSF_MAX;
    for spell in 1..RSF_MAX {
        if engine::flag_has(&race.spell_flags, spell as i32) {
            out.ranged_risk |= recall_spell_has_unknown_damage(spell);
        }
    }
    out.unknown_damage |= out.ranged_risk;
    out
}

/// #759: a fresh comparison on every think, including during an existing
/// flight. Count every awake visible attacker, including those which can
/// join while another is fought. A target we cannot damage now prevents a
/// winning verdict; no optimistic missing-foe or unknown-HP assumptions.
fn winning_fight(st: &mut BorgState) -> bool {
    let mode = p(Param::caution_winning_fight);
    if mode == 0 {
        return false;
    }
    // The direct-damage race does not model random confused commands or
    // ongoing poison/bleeding damage. Let the ordinary cures act first.
    if [
        crate::Read::new(st).status().isconfused(),
        crate::Read::new(st).status().isblind(),
        crate::Read::new(st).status().isimage(),
        crate::Read::new(st).status().iscut(),
        crate::Read::new(st).status().ispoisoned(),
    ]
    .iter()
    .any(|&t| t != 0)
    {
        return false;
    }
    let mut plans = Vec::new();
    let mut incoming = 0.0;
    let mut in_contact = false;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0
            || !kill.awake
            || kill.r_idx as usize >= engine::r_max().saturating_sub(1)
        {
            continue;
        }
        if borg_rs::decision_api::borg_town_hunt::viewed_target(st, &kill) {
            return false;
        }
        let d = engine::distance(st.borg.c, kill.pos);
        if st.cave.grid(kill.pos.x, kill.pos.y).info
            & borg_rs::decision_api::borg_cave_view::BORG_VIEW
            == 0
        {
            if d <= 1
                || borg_rs::decision_api::borg_danger::borg_danger_one_kill(
                    st,
                    st.borg.c.y,
                    st.borg.c.x,
                    1,
                    i as i32,
                    true,
                    true,
                ) > 0
            {
                return false; // an attacker without a current observable target
            }
            continue;
        }
        if d > 1
            && engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE)
            && kill.ranged_attack == 0
        {
            continue;
        }
        let forecast = fight_incoming(st, i);
        if forecast.unknown_fight || forecast.unknown_damage {
            return false;
        }
        let danger = forecast.hp_per_turn;
        if danger <= 0.0 {
            continue;
        }
        in_contact |= d <= 1 || kill.ranged_attack != 0;
        incoming += danger;
        let Some(attack) = crate::stateful::attack::focused_attack(st, i) else {
            return false;
        };
        let Some(hp) = crate::stateful::attack::observed_kill_hp(&kill) else {
            return false;
        };
        plans.push((i, hp, attack));
    }
    if plans.is_empty() || !in_contact {
        return false;
    }
    // Remove the fastest-to-kill attacker first. Incoming remains the full
    // sum for the entire fight, a conservative bound as attackers die.
    plans.sort_by(|a, b| {
        let turns = |p: &(usize, i32, crate::stateful::attack::FocusedAttack)| {
            f64::from(p.1) / f64::from(p.2.hp_per_turn_milli.max(1))
        };
        turns(a).total_cmp(&turns(b)).then(a.0.cmp(&b.0))
    });
    let mana = crate::Read::new(st).character().cursp();
    let mut mana_left = mana;
    let mut spent = std::collections::BTreeMap::<i32, i32>::new();
    let mut turns = 0.0;
    for &(i, hp, attack) in &plans {
        let cost = attack.cost;
        let uses = cost.and_then(|c| c.uses).map(|u| {
            if cost.is_some_and(|c| c.spell) {
                if mana <= 0 {
                    0
                } else {
                    u * mana_left / mana
                }
            } else {
                (u - spent.get(&attack.kind).copied().unwrap_or(0)).max(0)
            }
        });
        // mon-move.c:1893-1897 gates regen on turn%100==0, regardless of
        // monster speed; :1914-1925 handles each monster once that turn.
        // regen_monster: at least 1 HP each 100 game turns,
        // doubled for REGENERATE. Charge its continuous mean immediately,
        // conservatively, rather than assuming a long fight has no healing.
        let kill = st.kills.kills[i];
        let regen = fight_regeneration(&kill);
        let damage = f64::from(attack.hp_per_turn_milli) / 1000.0
            * f64::from(100 - cost.map_or(0, |c| c.fail_pct))
            / 100.0
            - regen;
        let fallback = (f64::from(attack.fallback) / 1000.0 - regen).max(0.0);
        let Some((t, used)) = race_turns(hp, damage, uses, attack.duration, fallback) else {
            return false;
        };
        turns += t;
        if let Some(c) = cost {
            if c.spell {
                let per_cast = (mana + c.uses.unwrap_or(1).max(1) - 1) / c.uses.unwrap_or(1).max(1);
                mana_left = (mana_left - per_cast * used).max(0);
            } else {
                *spent.entry(attack.kind).or_default() += used;
            }
        }
    }
    let hp = crate::Read::new(st).character().curhp();
    let won = wins_race(turns, incoming, hp);
    let foes: Vec<String> = plans
        .iter()
        .map(|&(i, needed, a)| {
            format!(
                "{}:needs {needed} HP, attack {} {:.3} HP/turn",
                String::from_utf8_lossy(engine::race_name(usize::from(st.kills.kills[i].r_idx))),
                a.kind,
                f64::from(a.hp_per_turn_milli) / 1000.0
            )
        })
        .collect();
    borg_note(st, &format!("# Fight race: {} attackers, {turns:.3} turns to kill, {incoming:.3} damage/turn, HP {hp}: {} (#759)",
        plans.len(), if won { "winning" } else { "not winning" }));
    borg_note(st, &format!("# Fight race targets: {}", foes.join("; ")));
    // Mode 2 is a diagnostic: retain the unchanged control's actions so
    // estimates can be checked on its original losing AND winning fights.
    if !won || mode == 2 {
        return false;
    }
    let (i, _, attack) = plans[0];
    crate::stateful::attack::perform_focused_attack(st, i, attack)
}

pub(crate) fn fight_regeneration(kill: &borg_rs::decision_api::borg_flow_kill::BorgKill) -> f64 {
    let max_hp = kill.observed_health.expect("fight needs observed health").1;
    f64::from((i32::from(max_hp) / 100).max(1))
        * if engine::race_flag_has(
            usize::from(kill.r_idx),
            borg_rs::decision_api::headers::list_mon_race_flags_h::RF_REGENERATE,
        ) {
            2.0
        } else {
            1.0
        }
        / f64::from(engine::extract_energy(
            engine::player_combat_speed().clamp(0, 199) as usize,
        ))
        .max(1.0)
}

fn category_threat_safe(hp: i32, target_damage: f64, other_danger: i64, other_limit: i64) -> bool {
    hp > 0
        && target_damage.is_finite()
        && target_damage < f64::from(hp)
        && other_danger <= other_limit
}

/// Owner-approved review exception: the category itself never creates a
/// scare, but lethal contact or excessive danger from other foes can still
/// reach ordinary cures/escapes. Reuse today's threshold MULTIPLIER. The
/// quantity is other monsters' native danger, excluding the target and the
/// regional/fear-map terms; it is not today's full borg_danger sum.
pub(crate) fn category_attack_safe(st: &mut BorgState, target: usize) -> bool {
    category_attack_safe_with(st, target, true)
}

/// #842: `gate_target` false drops the target's own damage test (`target
/// damage < HP`, and its unmodelled-damage veto) for a fast-fragile monster,
/// which is attacked whatever it does; the other foes' danger still counts.
pub(crate) fn category_attack_safe_with(
    st: &mut BorgState,
    target: usize,
    gate_target: bool,
) -> bool {
    let hp = crate::Read::new(st).character().curhp();
    let target_damage = fight_incoming_with(st, target, true);
    let mut other_danger = 0_i64;
    for i in 1..st.kills.nxt.max(0) as usize {
        if i == target || st.kills.kills[i].r_idx == 0 {
            continue;
        }
        other_danger += i64::from(borg_rs::decision_api::borg_danger::borg_danger_one_kill(
            st,
            st.borg.c.y,
            st.borg.c.x,
            1,
            i as i32,
            true,
            true,
        ))
        .max(0);
    }
    let limit = i64::from(hp) * i64::from(p(Param::caution_flee_danger_hp_multiplier));
    if !gate_target {
        // HP 0 is alive (the engine kills below zero).
        return hp >= 0 && other_danger <= limit;
    }
    !target_damage.unknown_damage
        && category_threat_safe(hp, target_damage.hp_per_turn, other_danger, limit)
}

/// #760/#759: before any dungeon flight, including the light/countdown
/// and depth-return rungs which otherwise precede borg_caution.
pub(crate) fn fight_before_flight(st: &mut BorgState) -> bool {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark("policy::caution.fight_before_flight", format!("{:?}", ()));
    }
    crate::stateful::attack::attack_on_sight(st) || winning_fight(st)
}

pub(crate) fn borg_caution(st: &mut BorgState) -> bool {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark("policy::caution.borg_caution", format!("{:?}", ()));
    }
    watch_for_recall(st);
    let mut nasty = false;

    /*** Notice "nasty" situations ***/

    // About to run out of light is extremely nasty
    if crate::Read::new(st).character().light() == 0
        && i32::from(st.items.items[inven_light()].timeout) < p(Param::caution_nasty_light_timeout)
    {
        nasty = true;
    }

    // Starvation, blindness, confusion, hallucination are nasty
    for f in [
        crate::Read::new(st).status().isweak(),
        crate::Read::new(st).status().isblind(),
        crate::Read::new(st).status().isconfused(),
        crate::Read::new(st).status().isimage(),
    ] {
        if f != 0 {
            nasty = true;
        }
    }

    // if on level 100 and not ready for Morgoth, run
    if crate::Read::new(st).character().cdepth() == 100
        && borg_timer(st, st.borg.time.level) < p(Param::caution_questor_arrival_turns)
        && !st.borg.morgoth_position
        && st.borg.ready_morgoth <= 0
        && crate::Read::new(st).status().king() == 0
    {
        // teleport level up to 99 to finish uniques
        if borg_spell(st, TELEPORT_LEVEL)
            || borg_read_scroll(st, st.item_val.sv_scroll_teleport_level)
            || borg_activate_item(st, st.act.tele_level)
        {
            borg_note(st, "# Rising one dlevel (Not ready for Morgoth)");
            return true;
        }

        // Start leaving
        if !st.borg.goal.leaving {
            // Note
            borg_note(st, "# Leaving (Not ready for Morgoth now)");

            // Start leaving
            st.borg.goal.leaving = true;
            if borg_flow_stair_less(st, GOAL_FLEE, false) {
                return true;
            }
        }
    }

    /*** Evaluate local danger ***/

    // Monsters on all sides of me?
    let borg_surround = borg_surrounded(st);

    // Only allow three 'escapes' per level unless heading for morogoth
    // or fighting a unique, then allow 85.
    if ((i32::from(st.borg.escapes) > p(Param::caution_escape_limit) && st.borg.mon.unique == 0 && st.borg.ready_morgoth == 0)
        || i32::from(st.borg.escapes) > p(Param::caution_escape_absolute_limit))
        // No leaving if going after questors
        && crate::Read::new(st).character().cdepth() <= p(Param::caution_escape_limit_max_depth)
    {
        // Start leaving
        start_leaving(st, "# Leaving (Too many escapes)");

        // Start fleeing
        if i32::from(st.borg.escapes) > p(Param::caution_escape_flee_limit) {
            start_fleeing(st, "# Fleeing (Too many escapes)");
        }
    }

    // No hanging around if nasty here.
    if st.borg.mon.scary {
        // Note
        borg_note(st, "# Scary guy on level.");

        // borg-reborn-rs (#530): which goals the scare itself sets.
        let before = (
            st.borg.goal.leaving,
            st.borg.goal.fleeing,
            st.borg.goal.fleeing_to_town,
        );

        // Start leaving
        start_leaving(st, "# Leaving (Scary guy on level)");

        // Start fleeing
        start_fleeing(st, "# Fleeing (Scary guy on level)");

        // Return to town quickly after leaving town
        if crate::Read::new(st).character().cdepth() == 0 {
            st.borg.goal.fleeing_to_town = true;
        }
        let goal = st.borg.goal.clone();
        note_scary_goals(&mut st.borg.mon, before, &goal);
    }

    // Make a note if Ignoring monsters (no fighting)
    if st.borg.goal.ignoring {
        borg_note(st, "# Ignoring combat with monsters.");
    }

    // Note if ignorig messages
    if st.borg.dont_react {
        borg_note(st, "# Borg ignoring messges.");
    }

    // Look around
    let pos_danger = borg_danger(
        st,
        st.borg.c.y,
        st.borg.c.x,
        p(Param::caution_danger_horizon),
        true,
        false,
    );

    // Describe (briefly) the current situation
    // Danger (ignore stupid "fear" danger)
    if (pos_danger > i32::from(st.borg.avoidance) / 10
        || pos_danger
            > i32::from(
                st.danger.fear_region[(st.borg.c.y / 11) as usize][(st.borg.c.x / 11) as usize],
            )
        || st.borg.morgoth_position
        || crate::Read::new(st).status().isweak() != 0
        || crate::Read::new(st).character().cdepth() == 100)
        && crate::Read::new(st).status().king() == 0
    {
        // Describe (briefly) the current situation
        let note = format!(
            "# Loc:{},{} Dep:{} Lev:{} HP:{}/{} SP:{}/{} Danger:p={}",
            st.borg.c.y,
            st.borg.c.x,
            crate::Read::new(st).character().cdepth(),
            crate::Read::new(st).character().clevel(),
            crate::Read::new(st).character().curhp(),
            crate::Read::new(st).character().maxhp(),
            crate::Read::new(st).character().cursp(),
            crate::Read::new(st).character().maxsp(),
            pos_danger
        );
        borg_note(st, &note);
        if st.borg.resistance != 0 {
            // Divides by borg_game_ratio like C (0 panics where C traps).
            let ratio = i32::from(st.traits.game_ratio);
            let note = format!(
                "# Protected by Resistance (borg turns:{}; game turns:{})",
                i32::from(st.borg.resistance) / ratio,
                engine::player_timed(TMD_OPP_ACID)
            );
            borg_note(st, &note);
        }
        if st.borg.temp.shield {
            borg_note(st, "# Protected by Mystic Shield");
        }
        if st.borg.temp.prot_from_evil {
            borg_note(st, "# Protected by PFE");
        }
        if st.borg.morgoth_position {
            borg_note(st, "# Protected by Sea of Runes.");
        }
        if st.borg.near.unique >= 10 {
            borg_note(st, "# Questor Combat.");
        }
        if st.borg.status.anti_summon {
            borg_note(st, "# Protected by anti-summon corridor.");
        }
    }
    // Comment on glyph and stairs
    let (on_upstair, on_dnstair) = note_standing(st);

    // borg-reborn-rs (#713): a creature kept at range is stepped away from
    // while fleeing too (the jelly's Paladin was "fleeing" all along, with no
    // stairs it could reach).
    if st.borg.goal.fleeing && kept_at_range_next_to(st) && immobile_step_away(st) {
        return true;
    }
    if !st.borg.goal.fleeing {
        // borg-reborn-rs (#646): away from an immobile attacker first.
        if immobile_step_away(st) {
            return true;
        }

        // Start being cautious and trying to not die
        if crate::Read::new(st).character().class() == CLASS_MAGE
            && !st.borg.morgoth_position
            && !st.borg.status.anti_summon
            && crate::Read::new(st).status().isblind() == 0
            && crate::Read::new(st).status().iscut() == 0
            && crate::Read::new(st).status().ispoisoned() == 0
            && crate::Read::new(st).status().isconfused() == 0
        {
            // do some defense before running away
            if borg_defend(st, pos_danger) {
                return true;
            }

            // try healing before running away
            if borg_heal(st, pos_danger) {
                return true;
            }
        } else {
            // try healing before running away
            if borg_heal(st, pos_danger) {
                return true;
            }

            // do some defense before running away!
            if borg_defend(st, pos_danger) {
                return true;
            }
        }
    } else {
        // borg-reborn-rs (#369): fleeing must not exclude healing merely
        // because the current HP cannot absorb another round. borg_heal()
        // checks whether the chosen cure makes that round survivable and
        // retains the configured wound, level and resource limits.
        //
        // (Consumer improvement unit I-002, better-borg-c `4.2.6-impact-v10`.)
        if borg_heal(st, pos_danger) {
            return true;
        }
    }

    // do some swapping before running away!
    if borg_rs::decision_api::borg_trait::borg_uses_swaps(st)
        && pos_danger > i32::from(st.borg.avoidance) / p(Param::caution_swap_danger_divisor)
        && borg_backup_swap(st, pos_danger)
    {
        return true;
    }

    // If I am waiting for recall,  & safe, then stay put.
    if st.borg.goal.recalling != 0
        && borg_rs::decision_api::borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && crate::Read::new(st).character().cdepth() != 0
        && crate::Read::new(st).status().ishungry() == 0
    {
        // borg-reborn-rs (#567): a monster was near a moment ago: one turn.
        if recall_rest_held(st) {
            return true;
        }
        // rest here until lift off
        borg_note(st, "# Resting for Recall.");
        borg_keypress(st, key(b'R'));
        for digit in p(Param::caution_recall_rest_turns).to_string().bytes() {
            borg_keypress(st, key(digit));
        }
        borg_keypress(st, KC_ENTER);

        // I'm not in a store
        st.borg.in_shop = false;

        return true;
    }

    // If I am waiting for recall in town
    if st.borg.goal.recalling != 0
        && st.borg.goal.recalling
            <= i32::from(st.traits.game_ratio) * p(Param::caution_recall_prep_turns)
        && crate::Read::new(st).character().cdepth() == 0
        && borg_prep_leave_level_spells(st)
    {
        return true;
    }

    /*** Danger ***/

    // Impending doom
    // Don't take off in the middle of a fight
    // just to restock and it is useless to restock
    // if you have just left town.
    if st.borg.near.unique == 0
        && borg_timer(st, st.borg.time.town) > p(Param::caution_restock_town_delay)
        && borg_rs::decision_api::borg_prepared::borg_restock_c(
            st,
            crate::Read::new(st).character().cdepth(),
        )
        .is_some()
    {
        // Start leaving
        if !st.borg.goal.leaving {
            // Note
            let r = borg_rs::decision_api::borg_prepared::borg_restock_c(
                st,
                crate::Read::new(st).character().cdepth(),
            );
            note_with_reason(st, b"# Leaving (restock) ", r);

            // Start leaving
            st.borg.goal.leaving = true;
        }
        // Start fleeing
        if !st.borg.goal.fleeing
            && crate::Read::new(st).supplies().accw() < p(Param::caution_restock_ccw_count)
            && crate::Read::new(st).food().food() > p(Param::caution_restock_food_count)
            && crate::Read::new(st).supplies().afuel() > p(Param::caution_restock_fuel_count)
            && borg_timer(st, st.borg.time.level) > p(Param::caution_restock_level_delay)
        {
            // Flee
            let r = borg_rs::decision_api::borg_prepared::borg_restock_c(
                st,
                crate::Read::new(st).character().cdepth(),
            );
            note_with_reason(st, b"# Fleeing (restock) ", r);

            // Start fleeing
            st.borg.goal.fleeing = true;
        }
    }
    // Excessive danger
    else if pos_danger
        > crate::Read::new(st).character().curhp() * p(Param::caution_flee_danger_hp_multiplier)
    {
        // Start fleeing
        // do not flee level if going after Morgoth or fighting a unique
        if !st.borg.goal.fleeing
            && st.borg.near.unique == 0
            && crate::Read::new(st).character().clevel() < p(Param::caution_flee_max_level)
            && !st.borg.status.vault
            && (crate::Read::new(st).character().cdepth() < 100 && st.borg.ready_morgoth != 1)
        {
            // Note
            borg_note(st, "# Fleeing (excessive danger)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }
    }
    // Potential danger (near death) in town
    else if crate::Read::new(st).character().cdepth() == 0
        && pos_danger > crate::Read::new(st).character().curhp()
        && crate::Read::new(st).character().clevel() < p(Param::caution_town_flee_max_level)
    {
        // Flee now
        start_leaving(st, "# Leaving (potential danger)");
    }

    // borg-reborn-rs (#840): short of kit, a real danger sends it to the stairs.
    // Its own test, not a branch of the chain above: with `kit_restock` on the
    // restock branch fires first and would mask it (review of #844, finding 3).
    if borg_rs::decision_api::borg_kit::leave_danger_percent(st).is_some_and(|pct| {
        pos_danger >= (crate::Read::new(st).character().curhp() * pct / 100).max(1)
    }) && !st.borg.goal.fleeing
        && st.borg.near.unique == 0
        && !st.borg.status.vault
    {
        start_fleeing(st, "# Fleeing (short of kit)");
    }

    /*** Stairs ***/

    let avoidance = i32::from(st.borg.avoidance);
    // Leaving or Fleeing, take stairs
    // borg-reborn-rs (#614): the stairs rule sets the way itself; choose_stairs
    // is skipped then, so nothing cancels a departure the rule has noted
    // (review of #616, N1).
    let by_stairs_rule = !(st.borg.goal.leaving || st.borg.goal.fleeing || st.borg.mon.scary)
        && leave_stairs_on_monster(st, pos_danger);
    if !by_stairs_rule
        && (st.borg.goal.leaving
            || st.borg.goal.fleeing
            || st.borg.mon.scary
            || ((pos_danger > avoidance
                || (crate::Read::new(st).character().clevel()
                    < p(Param::caution_stair_low_level)
                    && pos_danger > avoidance / p(Param::caution_stair_danger_divisor)))
                && on_upstair))
    // danger and standing on stair
    {
        choose_stairs(st, pos_danger);
    }

    // Take stairs up
    if st.borg.stair_less && !engine::player_opt(OPT_birth_force_descend) {
        // Usable stairs
        if u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) == FEAT_LESS || on_upstair {
            // Log it
            borg_note(st, "# Leaving via up stairs.");

            // Take the stairs
            borg_keypress(st, key(b'<'));

            // Success
            return true;
        }
    }

    // Take stairs down
    if st.borg.stair_more && !recall_restricts(st) {
        // borg-reborn-rs (#208): /shopping is town-only, even if caution
        // chooses a direct descent before the ordinary town-leave hook.
        if borg_rs::decision_api::shell::shopping_departure(st) {
            return true;
        }
        // Usable stairs
        if u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) == FEAT_MORE || on_dnstair {
            if borg_prep_leave_level_spells(st) {
                return true;
            }

            // Take the stairs
            borg_keypress(st, key(b'>'));

            // Success
            return true;
        }
    }

    /*** Deal with critical situations ***/

    // Require light
    if crate::Read::new(st).character().light() == 0 {
        // No Lite, AND Not Glowing
        let need = borg_maintain_light(st);
        if need == BorgNeed::MetNeed {
            return true;
        } else if need == BorgNeed::UnmetNeed && crate::Read::new(st).character().cdepth() != 0 {
            // Flee for fuel
            start_leaving(st, "# Leaving (need fuel)");
        }
    }

    // Prevent starvation
    if crate::Read::new(st).status().isweak() != 0 {
        // Attempt to satisfy hunger
        // borg-reborn-rs (#451): a Remove Hunger scroll goes before any food
        if borg_satisfy_hunger_before_food(st)
            || borg_eat_food_any(st)
            || borg_spell(st, REMOVE_HUNGER)
            || borg_spell(st, HERBAL_CURING)
        {
            // Success
            return true;
        }

        // Try to restore mana then cast the spell next round
        if borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana)
        {
            return true;
        }

        // Flee for food
        if crate::Read::new(st).character().cdepth() != 0 {
            start_leaving(st, "# Leaving (need food)");
            start_fleeing(st, "# Fleeing (need food)");
        }
    }

    // Prevent breeder explosions when low level
    // borg-reborn-rs (#503): not without an easy escape: stand and fight.
    if st.borg.near.breeder
        && crate::Read::new(st).character().clevel() < p(Param::caution_breeder_flee_max_level)
        && !breeder_stand(st)
        // borg-reborn-rs (#722, lead b): not from a one-think pause.
        && !breeder_stand_paused(st)
    {
        start_fleeing(st, "# Fleeing (breeder level)");
    }

    // borg-reborn-rs (#781 review): ordinary healing, defence, Recall and
    // emergency food keep priority; the unseen response precedes stair walking.
    if unseen_bump_response(st, pos_danger) {
        return true;
    }

    /*** Flee on foot ***/

    // Desperation Head for stairs
    if (st.stairs.less.num != 0 || st.stairs.more.num != 0)
        && (st.borg.goal.fleeing
            || st.borg.mon.scary
            || (pos_danger > i32::from(st.borg.avoidance)
                && crate::Read::new(st).character().clevel()
                    < p(Param::caution_desperate_max_level)))
        && desperate_for_stairs(st, pos_danger)
    {
        return true;
    }

    // borg-reborn-rs (#544): only breeders near and outmatched, walk out of
    // their room instead of fighting where it stands.
    if breeder_exit(st, pos_danger) {
        return true;
    }

    // borg-reborn-rs (#690): surrounded by monsters slower than the Borg (or
    // that never move) at the danger cap: walk out instead of fighting.
    if slow_crowd_exit(st, pos_danger) {
        return true;
    }

    // borg-reborn-rs (#504): cornered (no escape, a fight on, danger at or
    // above the HP): every buff it carries or can cast, Monster Confusion
    // against a big monster, then a melee class fights without backing away.
    let cornered = fight_cornered(st, pos_danger);
    // The boosts. #457's outmatched heal stays at the end of `borg_caution`,
    // after retreat and back-away: tried first, it kept a cornered Borg
    // drinking in place where one step away saved it (#511 replays).
    if cornered && cornered_boost(st, "Cornered") {
        return true;
    }
    if !cornered && breeder_fight_hard(st, pos_danger) && cornered_boost(st, "Hard breeder fight") {
        return true;
    }
    let all_out = cornered && melee_class(st);

    // Strategic retreat
    //
    // Do not retreat if
    // 1) we are icky (poisoned, blind, confused etc
    // 2) we have boosted our avoidance because we are stuck
    // 3) we are in a Sea of Runes
    // 4) we are not in a vault
    let danger_test =
        pos_danger > i32::from(st.borg.avoidance) / p(Param::caution_retreat_danger_divisor);
    // borg-reborn-rs (#556): fighting through breeders to the stairs, no
    // retreat or back-away undoes the flight while the danger is below the HP.
    let through = breeder_fighting_through(st, pos_danger);
    if !all_out
        && !through
        && may_move_away(st, danger_test, nasty, borg_surround, pos_danger, true)
        && retreat(st, pos_danger, borg_surround)
    {
        return true;
    }

    /*** Escape if possible ***/

    // Attempt to escape via spells
    if borg_escape(st, pos_danger) {
        // increment the escapes this level counter
        st.borg.escapes = st.borg.escapes.wrapping_add(1);

        // Clear any Flow queues
        st.borg.goal.type_0 = 0;

        // Success
        return true;
    }

    // borg-reborn-rs (#584, owner): the Phase Door rule refused a Phase Door
    // and no escape acted: attack, at range when nothing is adjacent, else
    // boost (`phase_rule_act`, once per think).
    if crate::stateful::escape::phase_rule_act(st) {
        return true;
    }

    /*** Back away ***/
    // Do not back up if
    // 1) we are nasty (poisoned, blind, confused etc
    // 2) we are boosting our avoidance because we are stuck
    // 3) we are in a sweet Morgoth position (sea of runes)
    // 4) the monster causing concern is asleep
    // 5) we are not in a vault
    // 6) loads of HP
    let danger_test =
        pos_danger > i32::from(st.borg.avoidance) * p(Param::caution_back_danger_tenths) / 10;
    if !all_out
        && !through
        && may_move_away(st, danger_test, nasty, borg_surround, pos_danger, false)
        && back_away(st, pos_danger, borg_surround)
    {
        return true;
    }

    /*** Cures ***/

    // cure confusion, second check, first (slightly different) in
    // borg_heal; borg-reborn-rs (#646): held there too.
    let held = crate::Read::new(st).status().isconfused() != 0 && confusion_cure_held(st);
    if held {
        borg_note(
            st,
            "# Not fixing Confusion next to an immobile attacker (#646, cures)",
        );
    }
    if crate::Read::new(st).status().isconfused() != 0 && !held {
        if crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp()
            >= p(Param::caution_confused_emergency_loss)
            && (borg_quaff_potion(st, st.item_val.sv_potion_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_life))
        {
            borg_note(st, "# Healing.  Confusion.");
            return true;
        }
        if borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_crit(st, false)
            || (confusion_clw_cure() && borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_activate_item(st, st.act.cure_confusion)
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
        {
            borg_note(st, "# Healing.  Confusion.");
            return true;
        }
    }

    // Cure fear when afraid
    if (crate::Read::new(st).status().isafraid() != 0
        && crate::Read::new(st).curses().crsfear() == 0)
        && (engine::randint0(100) < p(Param::caution_fear_cure_chance)
            || (crate::Read::new(st).character().class() == CLASS_WARRIOR
                && crate::Read::new(st).ammo().amissiles() <= 0))
        && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
            || borg_quaff_potion(st, st.item_val.sv_potion_boldness)
            || borg_quaff_potion(st, st.item_val.sv_potion_heroism)
            || borg_quaff_potion(st, st.item_val.sv_potion_berserk)
            || borg_spell_fail(st, BERSERK_STRENGTH, p(Param::caution_fear_berserk_fail)) // berserk
            || borg_spell_fail(st, HEROISM, p(Param::caution_fear_hero_fail)) // hero
            || borg_activate_item(st, st.act.cure_paranoia)
            || borg_activate_item(st, st.act.hero)
            || borg_activate_item(st, st.act.shero)
            || borg_activate_item(st, st.act.cure_mind)
            || borg_activate_item(st, st.act.rage_bless_resist)
            || borg_activate_item(st, st.act.rem_fear_pois)
            || borg_spell_fail(st, HOLY_WORD, p(Param::caution_fear_holy_fail)))
    {
        return true;
    }

    /*** Note impending death XXX XXX XXX ***/

    // Flee from low hit-points
    if (crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_flee_low_hp_divisor)
        || (crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_flee_moderate_hp_divisor)
            && crate::Read::new(st).character().curhp() < crate::Read::new(st).character().clevel() * p(Param::caution_flee_hp_per_level)))
        && crate::Read::new(st).supplies().accw() < p(Param::caution_flee_ccw_count)
        && crate::Read::new(st).supplies().aheal() < p(Param::caution_flee_heal_count)
        // Flee from low hit-points
        && crate::Read::new(st).character().cdepth() != 0
        && engine::randint0(100) < p(Param::caution_flee_low_hp_chance)
    {
        start_leaving(st, "# Leaving (low hit-points)");
        start_fleeing(st, "# Fleeing (low hit-points)");
    }

    // Flee from bleeding wounds or poison and no heals
    if (crate::Read::new(st).status().iscut() != 0 || crate::Read::new(st).status().ispoisoned() != 0)
        && crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_flee_status_hp_divisor)
        // Flee from bleeding wounds
        && crate::Read::new(st).character().cdepth() != 0
        && engine::randint0(100) < p(Param::caution_flee_status_chance)
    {
        start_leaving(st, "# Leaving (bleeding/poison)");
        start_fleeing(st, "# Fleeing (bleeding/poison)");
    }

    // Emergency check on healing.  Borg_heal has already been checked but
    // but we did not use our ez_heal potions.  All other attempts to save
    // ourselves have failed.  Use the ez_heal if I have it.
    if (crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_reserve_critical_hp_divisor) // dangerously low HP -OR-
        || (pos_danger > crate::Read::new(st).character().curhp() // extreme danger -AND-
            && (crate::Read::new(st).supplies().ateleport() + crate::Read::new(st).supplies().aescape() <= p(Param::caution_reserve_escape_count)
                && crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_reserve_danger_hp_divisor)))
        // low on escapes
        || (crate::Read::new(st).supplies().aezheal() > p(Param::caution_reserve_heal_count) && crate::Read::new(st).character().curhp() < crate::Read::new(st).character().maxhp() / p(Param::caution_reserve_surplus_hp_divisor))
        // moderate danger, lots of heals
        || (crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp() >= p(Param::caution_reserve_unique_loss)
            && st.borg.near.unique != 0
            && crate::Read::new(st).character().cdepth() >= p(Param::caution_reserve_unique_depth)))
        // moderate danger, unique, deep
        && (borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_life))
    {
        borg_note(st, "# Using reserve EZ_Heal.");
        return true;
    }

    // Use "recall" to flee if possible
    // borg-reborn-rs (#539, owner): only when no staircase is known (a free
    // up or down staircase on the level). Otherwise the Borg keeps the
    // scroll and carries on: the flee flow walks it to the stairs, and it
    // still fights and heals on the way. Recall costs the scroll and 15-35
    // turns of waiting, often next to the monster it flees.
    if st.borg.goal.fleeing
        && !st.borg.goal.fleeing_to_town
        && crate::Read::new(st).character().cdepth() >= 1
        && !stairs_before_recall(st)
        && !ranged_before_recall(st)
        && !breeder_recall_held(st)
        && recall_escape_allowed(st, Some(pos_danger))
        && crate::stateful::escape::borg_recall_for_flight(st)
    {
        // Note
        borg_note(st, "# Fleeing the level (recall)");

        // Success
        return true;
    }

    // If I am waiting for recall,and in danger, buy time with
    // phase and cure_anythings.
    // borg-reborn-rs (#603): with caution_recall_wait_as_usual the ordinary
    // escape and attack decisions apply instead.
    if recall_restricts(st)
        && pos_danger > i32::from(st.borg.avoidance) * p(Param::caution_recall_danger_multiplier)
        && buy_time_for_recall(st)
    {
        return true;
    }

    // borg-reborn-rs (#470): C's last resort here used the unknown stuff
    // (device, scroll, potion, mushroom) when the next round looked fatal.
    // An unidentified item is never used: an unknown scroll can be Deep
    // Descent, five levels below the deepest so far (borg-zoo#147).

    // borg-reborn-rs (#457): outmatched, nothing above took a turn. Heal
    // rather than fall through to borg_attack.
    if borg_heal_outmatched(st, pos_danger) {
        return true;
    }

    // Give the interrupted stair walk a turn before ordinary movement.
    resume_emergency_down_flight(st, pos_danger)
}

/// borg-reborn-rs (#539, owner): "use recall only if a stair is not
/// reachable at all". With `caution_flee_stairs_before_recall`, a fleeing
/// Borg that can walk to a free staircase does not read Recall.
fn stairs_before_recall(st: &mut BorgState) -> bool {
    p(Param::caution_flee_stairs_before_recall) != 0 && stair_reachable(st)
}

/// borg-reborn-rs (#539 review): can the Borg walk to a free staircase now?
/// Not "is one known": the probe spreads the flee flows' own stair flow
/// from every free staircase (no monster on it) and asks whether it reaches
/// the Borg. The up flow is `borg_flow_stair_less`'s; the down flow is
/// `borg_flow_stair_both`'s on a scary level and `borg_flow_stair_more`'s,
/// with its gates (preparation, food, sale, light, the #483 rising hold),
/// elsewhere. Nothing is committed: the cost map is rebuilt by the next
/// flow, and the danger marks are the ones that flow would make.
fn stair_reachable(st: &mut BorgState) -> bool {
    (!engine::player_opt(OPT_birth_force_descend) && stair_flow_reaches(st, true))
        || (down_stair_allowed(st) && stair_flow_reaches(st, false))
}

/// Would the flee flows take a down staircase (their gates, not the path)?
fn down_stair_allowed(st: &mut BorgState) -> bool {
    if crate::Read::new(st).character().light() == 0
        && crate::Read::new(st).character().cdepth() != 0
    {
        return false;
    }
    if st.borg.mon.scary {
        // borg_flow_stair_both: light is its only gate on a fleeing Borg
        return true;
    }
    if recall_restricts(st) {
        return false;
    }
    if st.stairs.less.num != 0 {
        let depth = crate::Read::new(st).character().cdepth() + 1;
        if borg_rs::decision_api::borg_prepared::prepared_c_for_decision(st, depth).is_some()
            || (crate::Read::new(st).character().cdepth() != 0
                && (crate::Read::new(st).status().isweak() != 0
                    || crate::Read::new(st).status().ishungry() != 0
                    || crate::Read::new(st).food().food() < 2))
            || borg_rs::decision_api::borg_trip::legacy_sale_hold(st)
        {
            return false;
        }
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let here = borg_danger(st, y, x, 1, true, false);
    !rising_holds_descent(st, here)
}

/// Does the up (`up`) or down stair flow, spread as the flee flows spread
/// it, reach the Borg?
fn stair_flow_reaches(st: &mut BorgState, up: bool) -> bool {
    let avoid = up
        && !(crate::Read::new(st).character().clevel() > 35
            || crate::Read::new(st).character().light() == 0)
        && !st.borg.status.desperate;
    let weight = stair_flight_weight(GOAL_FLEE, false);
    if weight > 0 {
        return borg_rs::decision_api::borg_flow_stairs::weighted_stair_reachable(
            st, up, avoid, weight,
        );
    }
    let track = if up { &st.stairs.less } else { &st.stairs.more };
    let stairs: Vec<(i32, i32)> = (0..track.num.max(0) as usize)
        .map(|i| (track.y[i], track.x[i]))
        .collect();
    if stairs.is_empty() {
        return false;
    }
    borg_rs::decision_api::borg_flow::borg_flow_clear(st);
    for (y, x) in stairs {
        if st.cave.grid(x, y).kill == 0 {
            borg_rs::decision_api::borg_flow::borg_flow_enqueue_grid(st, y, x);
        }
    }
    borg_rs::decision_api::borg_flow::borg_flow_spread(st, 250, false, avoid, false, -1, false);
    i32::from(st.flow.cost_at(st.borg.c.y, st.borg.c.x)) < 250
}

/// borg-reborn-rs (#539, owner): "if it has no stairs, the second best
/// choice is the best available ranged attack". With
/// `caution_scary_ranged_no_stairs` and no staircase the Borg can walk to,
/// a scary level lets `borg_attack` target a `ranged_candidate`.
pub(crate) fn scary_ranged_no_stairs(st: &mut BorgState) -> bool {
    p(Param::caution_scary_ranged_no_stairs) != 0 && !stair_reachable(st)
}

/// borg-reborn-rs (#539): the monster the ranged rule is about: awake, in
/// view and not next to the Borg. Recall and Phase Door wait for one, and
/// `borg_attack` takes only these on a scary level without stairs.
pub(crate) fn ranged_candidate(
    st: &BorgState,
    kill: &borg_rs::decision_api::borg_flow_kill::BorgKill,
) -> bool {
    ranged_target(
        true,
        &[(
            kill.awake,
            engine::distance(st.borg.c, kill.pos),
            st.cave.grid(kill.pos.x, kill.pos.y).info
                & borg_rs::decision_api::borg_cave_view::BORG_VIEW
                != 0,
        )],
    )
}

/// borg-reborn-rs (#539): Recall and Phase Door wait while the Borg can
/// fight at range instead: switch on, a ranged attack the attack planner
/// would make (`borg_fight_attack::ranged_means`), a `ranged_candidate`,
/// and no staircase it can walk to.
/// borg-reborn-rs (#557, owner): is Word of Recall an escape from this
/// flight? `caution_flee_recall` 1: always (C). 0: only on a level swamped
/// by breeders (`caution_flee_recall_breeders` known on the level) or when
/// chased by several monsters (`caution_flee_recall_chasers` awake, able to
/// move, within
/// `caution_flee_recall_chase_distance`). Otherwise the scroll waits 15-35
/// turns next to the monster it flees and is gone for the trip home.
/// `danger`: the danger here, when the caller has it (the flight's read);
/// the stand tests pass `None` and get no danger exception.
fn recall_escape_allowed(st: &BorgState, danger: Option<i32>) -> bool {
    // borg-reborn-rs (#557, owner): shallower than
    // `caution_flee_recall_gate_below_depth` (2: on DL1) the stairs are a
    // short walk, so the exception is needed whatever `caution_flee_recall`
    // says.
    recall_allowed_with(
        p(Param::caution_flee_recall) != 0,
        crate::Read::new(st).character().cdepth(),
        p(Param::caution_flee_recall_gate_below_depth),
        || {
            recall_exception(st)
                || danger.is_some_and(|d| {
                    recall_danger_reaches(d, crate::Read::new(st).character().curhp())
                })
        },
    )
}

/// borg-reborn-rs (#587, owner): "Recall only from a certain danger". The
/// danger here at or above `caution_flee_recall_danger_percent` of the
/// current HP allows the flight's Recall where the gate holds it (0 = off).
fn recall_danger_reaches(danger: i32, hp: i32) -> bool {
    danger_reaches_percent(danger, hp, p(Param::caution_flee_recall_danger_percent))
}

fn danger_reaches_percent(danger: i32, hp: i32, percent: i32) -> bool {
    percent > 0 && i64::from(danger) * 100 >= i64::from(hp.max(1)) * i64::from(percent)
}

/// The breeder or chaser exception (#559).
fn recall_exception(st: &BorgState) -> bool {
    let range = p(Param::caution_flee_recall_chase_distance);
    let mut chasers = 0;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        // A monster that never moves (a mold) cannot chase.
        if kill.r_idx != 0
            && kill.awake
            && engine::distance(st.borg.c, kill.pos) <= range
            && !engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE)
        {
            chasers += 1;
        }
    }
    recall_escape_with(
        i32::from(st.borg.mon.breeders),
        chasers,
        p(Param::caution_flee_recall_breeders),
        p(Param::caution_flee_recall_chasers),
    )
}

/// Is a flight's Recall allowed? Without the need for the exception, yes;
/// otherwise only with it (`exception`, evaluated only then).
fn recall_allowed_with(
    c_recall: bool,
    depth: i32,
    gate_below: i32,
    exception: impl FnOnce() -> bool,
) -> bool {
    !recall_needs_exception(c_recall, depth, gate_below) || exception()
}

/// Does a flight's Recall need the breeder or chaser exception? Always with
/// the gate on (`c_recall` false); with C's Recall, only at depths shallower
/// than `gate_below` (a ceiling: 2 = DL1 only; 0 = off; 1 = off too, since
/// the town never reaches here).
fn recall_needs_exception(c_recall: bool, depth: i32, gate_below: i32) -> bool {
    !c_recall || (gate_below > 0 && depth < gate_below)
}

/// Swamped by breeders or chased by several monsters (a limit of 0 is off).
fn recall_escape_with(breeders: i32, chasers: i32, breeder_limit: i32, chaser_limit: i32) -> bool {
    (breeder_limit > 0 && breeders >= breeder_limit)
        || (chaser_limit > 0 && chasers >= chaser_limit)
}

pub(crate) fn ranged_before_recall(st: &mut BorgState) -> bool {
    if p(Param::caution_scary_ranged_no_stairs) == 0 {
        return false;
    }
    let means = crate::stateful::attack::ranged_means(st);
    let mut known = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0
            || engine::r_max() == 0
            || kill.r_idx as i32 >= engine::r_max() as i32 - 1
        {
            continue;
        }
        known.push((
            kill.awake,
            engine::distance(st.borg.c, kill.pos),
            st.cave.grid(kill.pos.x, kill.pos.y).info
                & borg_rs::decision_api::borg_cave_view::BORG_VIEW
                != 0,
        ));
    }
    ranged_target(means, &known) && !stair_reachable(st)
}

/// Something to throw or shoot, and a known monster as (awake, distance,
/// in view) that is awake, in view and not adjacent.
fn ranged_target(means: bool, known: &[(bool, i32, bool)]) -> bool {
    means && known.iter().any(|&(awake, d, view)| awake && view && d > 1)
}

/// `borg_note(format("<prefix>%s", reason))` for a `borg_restock()` reason.
/// A NULL reason prints as empty in Angband's `format()` (it cannot be NULL
/// here: the caller has just seen a non-NULL one for the same depth).
fn note_with_reason(st: &mut BorgState, prefix: &[u8], reason: Option<Vec<u8>>) {
    let mut note = prefix.to_vec();
    note.extend_from_slice(reason.as_deref().unwrap_or(b""));
    borg_note_bytes(st, &note);
}

/// borg-reborn-rs (#503): breeders on the level and no easy escape: stand
/// and fight them at once instead of fleeing while they multiply. An easy
/// escape is a free staircase within `caution_breeder_easy_escape_steps`
/// (the engine's `distance`), or a Phase Door, teleport, Teleport Level,
/// escape or Word of Recall in the pack or spells (recall is slow, but
/// breeders are slow and level-wide). With one, the breeder flees of the C
/// Borg stay (the breeder flee of `borg_caution`, the breeder scary trigger
/// of `borg_near_monster_type`, the "no recall" flee of
/// `borg_check_breeders`). The generic low-HP and bleeding/poison flees
/// stay too: a Borg losing the fight is #504's cornered mode.
pub(crate) fn breeder_stand(st: &BorgState) -> bool {
    // borg-reborn-rs (#556): with caution_breeder_stoppable, the Borg stands
    // while the breeding can be stopped, escape or not.
    if let Some(stop) = breeders_stoppable(st) {
        return stop;
    }
    breeder_stand_with(
        st,
        p(Param::caution_breeder_fight),
        p(Param::caution_breeder_easy_escape_steps),
        nearest_free_stair(st, true),
        nearest_free_stair(st, false),
    )
}

/// `breeder_stand` with its parameters and the nearest free up and down
/// staircases' distances (-1: none known) given.
fn breeder_stand_with(st: &BorgState, fight: i32, steps: i32, up: i32, down: i32) -> bool {
    if fight == 0 || st.borg.mon.breeders == 0 || crate::Read::new(st).character().cdepth() == 0 {
        return false;
    }
    !easy_escape_with(st, steps, up, down)
}

/// borg-reborn-rs (#503, #530): a free staircase within `steps` (the
/// engine's distance; -1: none known), or a Phase Door, teleport, Teleport
/// Level, escape or Word of Recall in the pack or spells.
fn easy_escape_with(st: &BorgState, steps: i32, up: i32, down: i32) -> bool {
    let near = |d: i32| d != -1 && d <= steps;
    near(up)
        || near(down)
        || crate::Read::new(st).supplies().aphase() > 0
        || crate::Read::new(st).supplies().ateleport() > 0
        || crate::Read::new(st).supplies().ateleportlvl() > 0
        || crate::Read::new(st).supplies().aescape() > 0
        || (crate::Read::new(st).character().recall() > 0 && recall_escape_allowed(st, None))
}

/// borg-reborn-rs (#530, review of #531): record which goals the scary check
/// itself set: those that were false `before` it and are set now. A goal
/// another caution had already set stays that caution's.
pub(crate) fn note_scary_goals(
    mon: &mut borg_rs::decision_api::borg::BorgMonInfo,
    before: (bool, bool, bool),
    goal: &borg_rs::decision_api::borg::BorgGoal,
) {
    mon.scary_leaving |= !before.0 && goal.leaving;
    mon.scary_fleeing |= !before.1 && goal.fleeing;
    mon.scary_town |= !before.2 && goal.fleeing_to_town;
}

/// borg-reborn-rs (#603, owner): does a pending Word of Recall switch the
/// Borg to its Recall-only rules (no flight by stairs or Teleport Level, the
/// buy-time escapes, fighting because recalling)? With
/// `caution_recall_wait_as_usual` it plays by its ordinary rules instead:
/// the countdown keeps running on a new level (`effect-handler-general.c`).
/// Resting or waiting when safe is kept either way. Only on DL2 and deeper:
/// in town and on DL1 a staircase crosses between town and dungeon, and the
/// Recall would fire on the other side and take the Borg back (owner, #627).
pub(crate) fn recall_restricts(st: &BorgState) -> bool {
    recall_restricts_with(
        st.borg.goal.recalling != 0,
        p(Param::caution_recall_wait_as_usual) != 0,
        crate::Read::new(st).character().cdepth(),
    )
}

fn recall_restricts_with(recalling: bool, as_usual: bool, depth: i32) -> bool {
    recalling && (!as_usual || depth <= 1)
}

/// borg-reborn-rs (#754, owner): with `caution_recall_wait_no_explore`, while
/// Word of Recall counts down in the dungeon the Borg does not explore or
/// chase far monsters and objects. It waits where it stands instead, and the
/// think ends there: the hook sits before `borg_think_dungeon`'s "Explore the
/// dungeon" block, so that block (vaults, `borg_flow_kill`/`borg_flow_take`/
/// `borg_flow_vein` at 250, `borg_flow_dark`) and the rest of the think
/// (`borg_leave_level`, the shops, C's own late "Wait for recall") are
/// skipped for the wait. The wait rests for Recall as caution's own site
/// does (`caution_recall_rest_turns`) only when `borg_check_rest` allows it
/// here and nothing threatens the Borg (`borg_danger` 0; #567's hold first).
/// Otherwise it stays still one turn (`,`), so each think sees what comes.
/// 0 = C (explore while waiting; the wait comes after exploration).
pub(crate) fn recall_wait_no_explore(st: &mut BorgState) -> bool {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark(
            "policy::caution.recall_wait_no_explore",
            format!("{:?}", ()),
        );
    }
    if !recall_wait_holds(
        p(Param::caution_recall_wait_no_explore) != 0,
        st.borg.goal.recalling,
        crate::Read::new(st).character().cdepth(),
    ) {
        return false;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    // Review of #762: the rest keeps `borg_check_rest`'s refusals (hungry,
    // weak, unlit, fear, a multiplier just killed, lava).
    let safe = borg_danger(st, y, x, 1, true, false) <= 0
        && borg_rs::decision_api::borg_flow_misc::borg_check_rest(st, y, x);
    // #567's hold (a monster was near a moment ago) presses its own `,`.
    if safe && recall_rest_held(st) {
        return true;
    }
    if safe {
        borg_note(st, "# Waiting for Recall (not exploring).");
        borg_keypress(st, key(b'R'));
        for digit in p(Param::caution_recall_rest_turns).to_string().bytes() {
            borg_keypress(st, key(digit));
        }
        borg_keypress(st, KC_ENTER);
    } else {
        // Review of #762: keep #567's watch current while a monster is near.
        watch_for_recall(st);
        borg_note(st, "# Waiting for Recall (not exploring): staying still.");
        borg_keypress(st, key(b','));
    }
    st.borg.in_shop = false;
    true
}

/// The test of `recall_wait_no_explore`: on, Recall pending, in the dungeon.
fn recall_wait_holds(on: bool, recalling: i32, depth: i32) -> bool {
    on && recalling != 0 && depth != 0
}

/// borg-reborn-rs (#503): standing against breeders (`breeder_stand`), at
/// least one breeder within `caution_breeder_stand_radius` grids, and no
/// other known monster within the danger horizon
/// (`danger_monster_max_distance` on the larger axis distance, the reach of
/// `borg_danger`): the danger
/// here is the breeders', so the Borg fights instead of retreating or
/// backing away.
pub(crate) fn breeder_stand_here(st: &BorgState) -> bool {
    if !breeder_stand(st) {
        return false;
    }
    let mut known = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0
            || engine::r_max() == 0
            || kill.r_idx as i32 >= engine::r_max() as i32 - 1
        {
            continue;
        }
        // The radius is the engine's distance; the horizon is `borg_danger`'s
        // reach, the larger axis distance (`danger_one_kill`).
        let reach = (kill.pos.x - st.borg.c.x)
            .abs()
            .max((kill.pos.y - st.borg.c.y).abs());
        known.push((
            engine::distance(st.borg.c, kill.pos),
            reach,
            engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY),
        ));
    }
    only_breeders_near(
        &known,
        p(Param::caution_breeder_stand_radius),
        p(Param::danger_monster_max_distance),
    )
}

/// borg-reborn-rs (#544): with only breeders near, one adjacent, and the
/// danger here at or above the HP or the Borg surrounded, walk with the
/// desperate flow (danger ignored, monster grids routed around) to the
/// nearest known corridor grid at least `caution_breeder_exit_distance`
/// grids from every known breeder. The danger of a crowd of breeders
/// saturates at `danger_maximum` on every grid near them, so retreat and
/// back-away find no better grid and the Borg fights on where it stands
/// (priest-1055640847: 236 decisions on one grid of a dead-end worm room).
fn breeder_exit(st: &mut BorgState, pos_danger: i32) -> bool {
    if p(Param::caution_breeder_exit) == 0 {
        return false;
    }
    let distance = p(Param::caution_breeder_exit_distance);
    let mut known = Vec::new();
    let mut breeders = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0
            || engine::r_max() == 0
            || kill.r_idx as i32 >= engine::r_max() as i32 - 1
        {
            continue;
        }
        let breeder = engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY);
        let reach = (kill.pos.x - st.borg.c.x)
            .abs()
            .max((kill.pos.y - st.borg.c.y).abs());
        known.push((engine::distance(st.borg.c, kill.pos), reach, breeder));
        if breeder {
            breeders.push(kill.pos);
        }
    }
    if !only_breeders_near(&known, 1, p(Param::danger_monster_max_distance)) {
        return false;
    }
    if pos_danger < crate::Read::new(st).character().curhp() && !borg_surrounded(st) {
        return false;
    }
    let passable = |x: i32, y: i32| {
        engine::square_in_bounds_fully(x, y) && {
            let ag = st.cave.grid(x, y);
            borg_cave_floor_grid(ag) && u32::from(ag.feat) != FEAT_NONE
        }
    };
    let mut exits = Vec::new();
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            if st.cave.grid(x, y).kill == 0
                && breeder_exit_grid(&passable, loc { x, y }, &breeders, distance)
            {
                exits.push(loc { x, y });
            }
        }
    }
    if exits.is_empty() {
        return false;
    }
    // The desperate flow (danger ignored, unknown and monster grids routed
    // around) is set for this flow only: `breeder_exit_flow` has no other
    // exit, so the flag is always restored.
    st.borg.status.desperate = true;
    let ok = breeder_exit_flow(st, &exits);
    st.borg.status.desperate = false;
    if ok {
        borg_note(
            st,
            &format!("# Leaving the breeders' room ({} exit grids)", exits.len()),
        );
    }
    ok
}

/// borg-reborn-rs (#690, generalising #544's breeder exit): surrounded,
/// the danger here at or above the HP, and every awake monster within
/// `borg_danger`'s reach slower than the Borg (`slower`) or never moving:
/// walk with the desperate flow to the nearest known chokepoint or
/// staircase at least `caution_slow_crowd_exit_distance` grids from every
/// monster within that reach. A crowd's danger saturates at the cap on every
/// grid near it, so retreat and back-away find no better grid, and with no
/// escape item the Borg fought on one grid until it died (borg-cemetery#78:
/// a Blackguard in a yellow worm mass colony, speed 100 against its 110).
fn slow_crowd_exit(st: &mut BorgState, pos_danger: i32) -> bool {
    if p(Param::caution_slow_crowd_exit) == 0 || crate::Read::new(st).character().cdepth() == 0 {
        return false;
    }
    if pos_danger < crate::Read::new(st).character().curhp() {
        return false;
    }
    let horizon = p(Param::danger_monster_max_distance);
    let energy = |speed: i32| i32::from(engine::extract_energy(speed.clamp(0, 199) as usize));
    let own = energy(crate::Read::new(st).character().speed());
    let margin = p(Param::caution_slow_crowd_exit_margin_percent);
    let mut known = Vec::new();
    let mut crowd = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0
            || engine::r_max() == 0
            || kill.r_idx as i32 >= engine::r_max() as i32 - 1
        {
            continue;
        }
        let reach = (kill.pos.x - st.borg.c.x)
            .abs()
            .max((kill.pos.y - st.borg.c.y).abs());
        if reach > horizon {
            continue;
        }
        let still = engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE);
        known.push(CrowdMonster {
            reach,
            awake: kill.awake,
            still,
            energy: energy(i32::from(kill.speed)),
        });
        crowd.push(kill.pos);
    }
    if !slow_crowd(&known, own, margin) {
        return false;
    }
    if !borg_surrounded(st) {
        return false;
    }
    let distance = p(Param::caution_slow_crowd_exit_distance);
    // The known map, read once: the flows below borrow `st` mutably.
    let w = AUTO_MAX_X as usize;
    let mut floor = vec![false; w * AUTO_MAX_Y as usize];
    let mut stair = vec![false; floor.len()];
    let mut taken = vec![false; floor.len()];
    for y in 0..AUTO_MAX_Y {
        for x in 0..AUTO_MAX_X {
            let i = y as usize * w + x as usize;
            let ag = st.cave.grid(x, y);
            let feat = u32::from(ag.feat);
            floor[i] = engine::square_in_bounds_fully(x, y)
                && borg_cave_floor_grid(ag)
                && feat != FEAT_NONE;
            stair[i] = feat == FEAT_LESS || feat == FEAT_MORE;
            taken[i] = ag.kill != 0;
        }
    }
    let at_index = |x: i32, y: i32| (0..AUTO_MAX_X).contains(&x) && (0..AUTO_MAX_Y).contains(&y);
    let passable = |x: i32, y: i32| at_index(x, y) && floor[y as usize * w + x as usize];
    let stairs = |x: i32, y: i32| at_index(x, y) && stair[y as usize * w + x as usize];
    // The preferred exits (a corridor, doorway or staircase), and every
    // other open grid beyond the distance: a crowd in a room at the edge of
    // the known map can leave no chokepoint the Borg can reach
    // (borg-cemetery#78: the nearest, 115 grids away, behind the worms).
    // A big crowd can leave no grid that far from all of it, so the
    // distance comes down to 2 (two steps clear of every monster).
    for d in (2..=distance.max(2)).rev() {
        let mut exits = Vec::new();
        let mut open = Vec::new();
        for y in 1..AUTO_MAX_Y - 1 {
            for x in 1..AUTO_MAX_X - 1 {
                let at = loc { x, y };
                if taken[y as usize * w + x as usize] {
                    continue;
                }
                if crowd_exit_grid(&passable, &stairs, at, &crowd, d) {
                    exits.push(at);
                } else if passable(x, y) && crowd.iter().all(|&m| engine::distance(m, at) >= d) {
                    open.push(at);
                }
            }
        }
        // As the breeder exit: the desperate flow for this flow only, restored.
        for (grids, what) in [(&exits, "exit grids"), (&open, "open grids")] {
            if grids.is_empty() {
                continue;
            }
            st.borg.status.desperate = true;
            let ok = breeder_exit_flow(st, grids);
            st.borg.status.desperate = false;
            if ok {
                borg_note(
                    st,
                    &format!(
                        "# Walking out of a slower crowd ({} monsters, danger {pos_danger}, {} {what} at distance {d})",
                        known.len(),
                        grids.len()
                    ),
                );
                return true;
            }
        }
    }
    false
}

/// A monster within reach, for `slow_crowd` (#690).
#[derive(Clone, Copy)]
struct CrowdMonster {
    /// `borg_danger`'s distance (the larger axis distance).
    reach: i32,
    awake: bool,
    /// The race never moves (`RF_NEVER_MOVE`).
    still: bool,
    /// `extract_energy` of its speed: energy per game turn.
    energy: i32,
}

/// The crowd can be walked away from (#690): a monster adjacent, and every
/// awake monster that moves acts less often than the Borg by the margin
/// (`own` energy at least `100 + margin_percent` % of its energy). A
/// sleeping monster does not chase; one that never moves cannot.
fn slow_crowd(known: &[CrowdMonster], own: i32, margin_percent: i32) -> bool {
    let adjacent = known.iter().any(|m| m.reach <= 1);
    adjacent
        && known
            .iter()
            .all(|m| !m.awake || m.still || own * 100 >= m.energy * (100 + margin_percent))
}

/// A crowd exit (#690): a passable grid at least `distance` grids from
/// every monster of the crowd that is a chokepoint (a corridor or doorway,
/// `chokepoint`) or a staircase.
fn crowd_exit_grid(
    passable: &dyn Fn(i32, i32) -> bool,
    stairs: &dyn Fn(i32, i32) -> bool,
    at: loc,
    crowd: &[loc],
    distance: i32,
) -> bool {
    passable(at.x, at.y)
        && (stairs(at.x, at.y) || chokepoint(passable, at))
        && crowd.iter().all(|&m| engine::distance(m, at) >= distance)
}

/// borg-reborn-rs (#697): walk to the nearest known grid at least `distance`
/// grids from `from`: a corridor or doorway (#544's `breeder_exit_grid`) if
/// one is known, else any known floor grid. Unlike #544's exit the flow is
/// not the desperate one: danger is counted, so a dangerous monster's grid
/// is refused; `avoid` keeps the path off unknown grids, as the desperate
/// flow does (review of #698). Takes the first step; false when no grid is
/// known or none can be reached.
pub(crate) fn walk_away_from(
    st: &mut BorgState,
    from: loc,
    distance: i32,
    why: &std::ffi::CStr,
) -> bool {
    let passable = |x: i32, y: i32| {
        engine::square_in_bounds_fully(x, y) && {
            let ag = st.cave.grid(x, y);
            borg_cave_floor_grid(ag) && u32::from(ag.feat) != FEAT_NONE
        }
    };
    let mut corridors = Vec::new();
    let mut floors = Vec::new();
    for y in 1..AUTO_MAX_Y - 1 {
        for x in 1..AUTO_MAX_X - 1 {
            let at = loc { x, y };
            if st.cave.grid(x, y).kill != 0
                || !passable(x, y)
                || engine::distance(from, at) < distance
            {
                continue;
            }
            if breeder_exit_grid(&passable, at, &[from], distance) {
                corridors.push(at);
            } else {
                floors.push(at);
            }
        }
    }
    let targets = if corridors.is_empty() {
        floors
    } else {
        corridors
    };
    if targets.is_empty() {
        return false;
    }
    borg_rs::decision_api::borg_flow::borg_flow_clear(st);
    for e in &targets {
        borg_rs::decision_api::borg_flow::borg_flow_enqueue_grid(st, e.y, e.x);
    }
    borg_rs::decision_api::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);
    borg_rs::decision_api::borg_flow::borg_flow_commit(st, why, GOAL_FLEE)
        && borg_rs::decision_api::borg_flow::borg_flow_old(st, GOAL_FLEE)
}

/// Flow to the nearest of `exits` and take its first step. When the flow
/// is committed but no step can be taken, `borg_flow_old` cancels the goal
/// (`goal.type_0 = 0`), so no `GOAL_FLEE` flow is left for the next think.
fn breeder_exit_flow(st: &mut BorgState, exits: &[loc]) -> bool {
    borg_rs::decision_api::borg_flow::borg_flow_clear(st);
    for e in exits {
        borg_rs::decision_api::borg_flow::borg_flow_enqueue_grid(st, e.y, e.x);
    }
    borg_rs::decision_api::borg_flow::borg_flow_spread(st, 250, true, false, false, -1, false);
    borg_rs::decision_api::borg_flow::borg_flow_commit(st, c"breeder-exit", GOAL_FLEE)
        && borg_rs::decision_api::borg_flow::borg_flow_old(st, GOAL_FLEE)
}

/// borg-reborn-rs (#722): with the policy at 3 or more, on a lost level,
/// whether Recall waits (the flight searches for stairs first) and whether
/// it is due (`caution_breeder_lost_recall_turns` game turns after the loss).
fn lost_recall(policy: i32, lost: bool, lost_turn: i32, now: i32, turns: i32) -> (bool, bool) {
    let applies = policy >= 3 && lost && lost_turn > 0;
    let due = applies && now - lost_turn >= turns;
    (applies && !due, due)
}

/// borg-reborn-rs (#722): Recall is held on a level lost to breeders until
/// the search for stairs has had its turns.
pub(crate) fn breeder_recall_held(st: &BorgState) -> bool {
    lost_recall(
        p(Param::caution_breeder_policy),
        st.borg.breeders_lost,
        st.borg.breeders_lost_turn,
        engine::game_turn(),
        p(Param::caution_breeder_lost_recall_turns),
    )
    .0
}

/// borg-reborn-rs (#556, owner): stand while the colony can be stopped; on a
/// level lost to the breeders, still stand while no staircase is known (a
/// flight with nowhere to go only wanders through the colony,
/// borg-cemetery#77); never while the danger here is at or above the HP.
/// borg-reborn-rs (#722, lead b): with `caution_breeder_stoppable`, the stand
/// is paused for this think only: the level is not lost, but the danger here
/// is at or above the HP. The level-long effects of not standing (the
/// "ignoring breeders" leave of `borg_check_breeders`, the breeder flee, the
/// breeder scary level) must not follow from a pause, or one dangerous turn
/// latches the flight for the whole level.
pub(crate) fn breeder_stand_paused(st: &BorgState) -> bool {
    stand_paused(
        p(Param::caution_breeder_stoppable) != 0,
        st.borg.breeders_lost,
        st.borg.breeders_in_danger,
        st.borg.mon.breeders > 0 && crate::Read::new(st).character().cdepth() != 0,
    )
}

/// `breeder_stand_paused` with its inputs given.
fn stand_paused(stoppable: bool, lost: bool, in_danger: bool, breeders_here: bool) -> bool {
    stoppable && breeders_here && !lost && in_danger
}

fn stand_against_breeders(lost: bool, no_stairs_known: bool, in_danger: bool) -> bool {
    !in_danger && (!lost || no_stairs_known)
}

/// borg-reborn-rs (#556): with `caution_breeder_stoppable`, the decision to
/// stand and hunt (`Some(stoppable)`); `None` with the switch off (the #503
/// stand and #558 hunt decide).
fn breeders_stoppable(st: &BorgState) -> Option<bool> {
    (p(Param::caution_breeder_stoppable) != 0).then(|| {
        stand_against_breeders(
            st.borg.breeders_lost,
            // borg-reborn-rs (#722, owner: "search for stairs first"):
            // with the policy at 3 or more a lost level is left even
            // with no stairs known; the flight searches for them.
            st.stairs.less.num == 0
                && st.stairs.more.num == 0
                && p(Param::caution_breeder_policy) < 3,
            st.borg.breeders_in_danger,
        ) && st.borg.mon.breeders > 0
            && crate::Read::new(st).character().cdepth() != 0
    })
}

/// borg-reborn-rs (#556): the marks are suspended
/// (`suspend_breeder_marks_when_stuck`).
fn breeder_marks_suspended(st: &BorgState) -> bool {
    st.borg
        .breeder_stuck
        .is_some_and(|t| st.borg.time.now.wrapping_sub(t) < p(Param::caution_breeder_stuck_turns))
}

/// borg-reborn-rs (#556): a flight took the plain route through remembered
/// breeders (`fight_through`, borg_flow_stairs.rs) within the last
/// `caution_breeder_through_turns` Borg turns, and the danger here is below
/// the HP: the Borg fights its way along it instead of backing away.
pub(crate) fn breeder_fighting_through(st: &BorgState, pos_danger: i32) -> bool {
    st.borg.breeder_through.is_some_and(|t| {
        st.borg.time.now.wrapping_sub(t) <= p(Param::caution_breeder_through_turns)
    }) && pos_danger < crate::Read::new(st).character().curhp()
}

/// borg-reborn-rs (#556): `(x, y)` is near a remembered breeder
/// (`breeder_block`; always false with `caution_breeder_avoid` 0).
pub(crate) fn near_known_breeder(st: &BorgState, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= AUTO_MAX_X || y >= AUTO_MAX_Y || breeder_marks_suspended(st) {
        return false;
    }
    st.borg
        .breeder_block
        .get((y * AUTO_MAX_X + x) as usize)
        .copied()
        .unwrap_or(false)
}

/// The 8 neighbours' offsets.
const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// A breeder exit (#544): a passable grid at least `distance` grids from
/// every known breeder that is a chokepoint (`chokepoint`). `passable` is
/// known floor: an unknown grid counts as a wall, and the desperate flow does
/// not cross unknown grids either, so every exit it reaches is on a known path.
fn breeder_exit_grid(
    passable: &dyn Fn(i32, i32) -> bool,
    at: loc,
    breeders: &[loc],
    distance: i32,
) -> bool {
    passable(at.x, at.y)
        && chokepoint(passable, at)
        && breeders
            .iter()
            .all(|&b| engine::distance(b, at) >= distance)
}

/// A chokepoint: the passable neighbours of `at` fall into at least two
/// groups that do not touch each other (8-connected), so the grid is a
/// corridor or doorway between them. A room grid, a room edge or corner, a
/// dead-end tip and an isolated grid have at most one group.
fn chokepoint(passable: &dyn Fn(i32, i32) -> bool, at: loc) -> bool {
    let open: Vec<(i32, i32)> = NEIGHBOURS
        .iter()
        .copied()
        .filter(|&(dx, dy)| passable(at.x + dx, at.y + dy))
        .collect();
    let mut group = vec![usize::MAX; open.len()];
    let mut groups = 0;
    for start in 0..open.len() {
        if group[start] != usize::MAX {
            continue;
        }
        group[start] = groups;
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            for j in 0..open.len() {
                if group[j] == usize::MAX
                    && (open[i].0 - open[j].0).abs() <= 1
                    && (open[i].1 - open[j].1).abs() <= 1
                {
                    group[j] = groups;
                    stack.push(j);
                }
            }
        }
        groups += 1;
    }
    groups >= 2
}

/// borg-reborn-rs (#557): with `caution_rising_hold_flee_descent`, while
/// rising in the dungeon a flight takes no down staircase unless the danger
/// here is at or above the HP. With `birth_connect_stairs` the down staircase
/// lands the Borg on DL+1's up staircase, it climbs back to a fresh level on
/// a down staircase again, and the walk home restarts: priest-1055640847
/// bounced DL2-DL3 eight times in 30,000 turns without reaching town.
pub(crate) fn rising_holds_flee_descent(st: &mut BorgState) -> bool {
    if p(Param::caution_rising_hold_flee_descent) == 0
        || !st.borg.goal.rising
        || crate::Read::new(st).character().cdepth() == 0
    {
        return false;
    }
    // borg-reborn-rs (#753): not for a fleeing Borg at low HP (214960633:
    // held at 2 of 47 HP on a down staircase, danger 0 there, beside a
    // grey mold it kept walking past).
    if fleeing_hurt(
        st.borg.goal.fleeing,
        crate::Read::new(st).character().curhp(),
        crate::Read::new(st).character().maxhp(),
        p(Param::caution_rising_hold_min_hp_percent),
    ) {
        return false;
    }
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    rising_hold(here, crate::Read::new(st).character().curhp())
}

/// #753: fleeing with less than `min_percent` of the max HP (0 = off).
fn fleeing_hurt(fleeing: bool, hp: i32, max_hp: i32, min_percent: i32) -> bool {
    fleeing
        && min_percent > 0
        && i64::from(hp) * 100 < i64::from(max_hp.max(1)) * i64::from(min_percent)
}

/// The danger test of `rising_holds_flee_descent` (#557): held unless the
/// danger here is at or above the HP.
fn rising_hold(danger: i32, hp: i32) -> bool {
    danger < hp
}

/// The known monsters as (distance, danger reach, breeder): a breeder
/// within `radius` (distance) and no other monster within `horizon` (reach).
fn only_breeders_near(known: &[(i32, i32, bool)], radius: i32, horizon: i32) -> bool {
    let mut breeders = 0;
    for &(d, reach, breeder) in known {
        if !breeder && reach <= horizon {
            return false;
        }
        if breeder && d <= radius {
            breeders += 1;
        }
    }
    breeders > 0
}

/// borg-reborn-rs (#505): `randcalc(dice, level, AVERAGE)` (z-rand.c) in
/// tenths of a point: `base + dice x (sides + 1) / 2 + m_bonus x level /
/// MAX_RAND_DEPTH`, keeping the dice average's half point that the engine's
/// integer `damcalc` drops (1d4 is 2.5, not 2). The engine rolls the same
/// `random_value` for a blow (mon-attack.c `make_attack_normal`).
fn blow_average_tenths(base: i32, dice: i32, sides: i32, m_bonus: i32, level: i32) -> i64 {
    const MAX_RAND_DEPTH: i64 = 128; // z-rand.h
    i64::from(base) * 10
        + i64::from(dice) * i64::from(sides + 1) * 5
        + i64::from(m_bonus) * i64::from(level) * 10 / MAX_RAND_DEPTH
}

/// borg-reborn-rs (#503, #558): the Borg hunts breeders while there are at
/// most `caution_breeder_hunt_max_count` of them, instead of avoiding them
/// below clevel 10 (`borg_flow_kill`; the gate is here too, since #558's
/// rest and acid-fear switches read it). Every class hunts; the count has
/// class values (the class files), and 0 turns a class's hunt off. Being
/// surrounded is left to
/// `borg_flow_kill_corridor`, which already counts a breeder next to the
/// Borg as a reason to move to a corridor.
pub(crate) fn breeder_hunt(st: &BorgState) -> bool {
    // borg-reborn-rs (#556): with caution_breeder_stoppable, the Borg hunts
    // while the breeding can be stopped, at any count and level.
    if let Some(stop) = breeders_stoppable(st) {
        return stop;
    }
    breeder_hunt_with(
        st,
        p(Param::caution_breeder_fight),
        p(Param::caution_breeder_hunt_max_count),
    )
}

/// `breeder_hunt` with its parameters given.
fn breeder_hunt_with(st: &BorgState, fight: i32, max_count: i32) -> bool {
    fight != 0
        && crate::Read::new(st).character().cdepth() != 0
        && crate::Read::new(st).character().clevel() < 10
        && st.borg.mon.breeders > 0
        && i32::from(st.borg.mon.breeders) <= max_count
}

/// `breeder_engaged`, except that a leave set by #644's rule
/// (`own_leave`) does not count (review of #656).
pub(crate) fn breeder_engaged_but_own_leave(st: &BorgState, own_leave: bool) -> bool {
    breeder_hunt(st)
        && engaged_ignoring_own_leave(
            st.borg.goal.ignoring,
            st.borg.goal.fleeing || st.borg.goal.fleeing_to_town,
            st.borg.goal.leaving,
            st.borg.mon.scary,
            own_leave,
        )
}

/// `engaged_with` with the leave split out: `own_leave` = the leave is #644's.
pub(crate) fn engaged_ignoring_own_leave(
    ignoring: bool,
    fleeing: bool,
    leaving: bool,
    scary: bool,
    own_leave: bool,
) -> bool {
    engaged_with(ignoring, fleeing || (leaving && !own_leave), scary)
}

fn engaged_with(ignoring: bool, fleeing: bool, scary: bool) -> bool {
    !ignoring && !fleeing && !scary
}

/// borg-reborn-rs (#504): Warriors, Paladins, Rogues and Blackguards.
fn melee_class(st: &BorgState) -> bool {
    matches!(crate::Read::new(st).character().class(), 0 | 5 | 6 | 8)
}

/// borg-reborn-rs (#504): cornered. In the dungeon, with no Phase Door,
/// teleport, Teleport Level or escape in the pack or spells (unlike #503's
/// easy escapes, Word of Recall does not count: it takes 15-35 turns, too
/// slow for a fight already lost), a known monster within
/// `caution_cornered_radius` grids, and danger here at or above
/// `caution_cornered_danger_tenths` / 10 of the current HP.
pub(crate) fn fight_cornered(st: &BorgState, pos_danger: i32) -> bool {
    fight_cornered_with(
        st,
        pos_danger,
        p(Param::caution_cornered),
        p(Param::caution_cornered_danger_tenths),
        nearest_kill(st),
        p(Param::caution_cornered_radius),
    )
}

/// `fight_cornered` with its parameters and the nearest known monster's
/// distance (-1: none) given.
fn fight_cornered_with(
    st: &BorgState,
    pos_danger: i32,
    on: i32,
    tenths: i32,
    nearest: i32,
    radius: i32,
) -> bool {
    on != 0
        && crate::Read::new(st).character().cdepth() != 0
        && i64::from(pos_danger) * 10
            >= i64::from(crate::Read::new(st).character().curhp()) * i64::from(tenths)
        && nearest != -1
        && nearest <= radius
        && crate::Read::new(st).supplies().aphase() <= 0
        && crate::Read::new(st).supplies().ateleport() <= 0
        && crate::Read::new(st).supplies().ateleportlvl() <= 0
        && crate::Read::new(st).supplies().aescape() <= 0
}

/// The distance to the nearest known monster that can move, -1 when none is
/// known. A monster that never moves (a mold) does not corner the Borg: one
/// step away leaves it.
fn nearest_kill(st: &BorgState) -> i32 {
    let mut best = -1;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0
            || engine::r_max() == 0
            || kill.r_idx as i32 >= engine::r_max() as i32 - 1
        {
            continue;
        }
        if engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE) {
            continue;
        }
        let d = engine::distance(st.borg.c, kill.pos);
        if best == -1 || d < best {
            best = d;
        }
    }
    best
}

/// borg-reborn-rs (#504, owner): standing against breeders (`breeder_stand`:
/// no easy escape) and the fight seems hard, danger here at or above
/// `caution_breeder_boost_danger_tenths` / 10 of the current HP: the same
/// boosts as cornered (Heroism, Berserk Strength, Blessing, Speed), which
/// `borg_defend`'s danger windows otherwise never use there (0 = off).
fn breeder_fight_hard(st: &BorgState, pos_danger: i32) -> bool {
    breeder_fight_hard_with(
        st,
        pos_danger,
        p(Param::caution_breeder_boost_danger_tenths),
    ) && breeder_stand(st)
}

/// The danger half of `breeder_fight_hard`, with its parameter given.
fn breeder_fight_hard_with(st: &BorgState, pos_danger: i32, tenths: i32) -> bool {
    tenths > 0
        && pos_danger > 0
        && i64::from(pos_danger) * 10
            >= i64::from(crate::Read::new(st).character().curhp()) * i64::from(tenths)
}

/// An adjacent known monster that is "big": unique, or above the Borg's
/// level by more than `caution_cornered_big_level_margin`, and not immune to
/// confusion (`RF_NO_CONF`).
fn big_monster_adjacent(st: &BorgState) -> bool {
    let margin = p(Param::caution_cornered_big_level_margin);
    (1..st.kills.nxt.max(0) as usize).any(|i| {
        let kill = &st.kills.kills[i];
        kill.r_idx != 0
            && engine::r_max() != 0
            && (kill.r_idx as i32) < engine::r_max() as i32 - 1
            && engine::distance(st.borg.c, kill.pos) == 1
            && !engine::race_flag_has(usize::from(kill.r_idx), RF_NO_CONF)
            && (engine::race_flag_has(usize::from(kill.r_idx), RF_UNIQUE)
                || i32::from(kill.level) > crate::Read::new(st).character().clevel() + margin)
    })
}

/// borg-reborn-rs (#504): a cornered boost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Boost {
    FearCure,
    Speed,
    Heroism,
    Berserk,
    Blessing,
    MonsterConfusion,
}

/// The boosts to try, in order, from what is active now: a fear cure when
/// afraid, Speed, Heroism, Berserk Strength, Blessing, then Monster
/// Confusion against an adjacent big monster. Each only while it is not
/// active.
fn boost_order(
    afraid: bool,
    fast: bool,
    hero: bool,
    berserk: bool,
    bless: bool,
    confusing: bool,
    big: bool,
) -> Vec<Boost> {
    let mut out = Vec::new();
    if afraid {
        out.push(Boost::FearCure);
    }
    if !fast {
        out.push(Boost::Speed);
    }
    if !hero {
        out.push(Boost::Heroism);
    }
    if !berserk {
        out.push(Boost::Berserk);
    }
    if !bless {
        out.push(Boost::Blessing);
    }
    if !confusing && big {
        out.push(Boost::MonsterConfusion);
    }
    out
}

/// borg-reborn-rs (#504): the cornered boosts (`boost_order`), the first one
/// carried or castable. Their usual danger windows (`borg_defend`'s "some
/// danger but not much") do not apply: the Borg is losing and cannot leave.
/// Spells (Haste Self, Heroism, Berserk Strength, Bless) are cast only at or
/// below `caution_cornered_spell_fail` percent failure,
/// so a failing cast is not retried turn after turn under fire.
pub(crate) fn cornered_boost(st: &mut BorgState, why: &str) -> bool {
    // borg-reborn-rs (#719, owner): "if I am cornered I should throw missile
    // and then boost myself".
    if throw_first(st, why) {
        return true;
    }
    let v = st.item_val;
    let fail = p(Param::caution_cornered_spell_fail);
    let order = boost_order(
        crate::Read::new(st).status().isafraid() != 0
            && crate::Read::new(st).curses().crsfear() == 0,
        st.borg.temp.fast,
        st.borg.temp.hero,
        st.borg.temp.berserk,
        st.borg.temp.bless,
        engine::player_timed(TMD_ATT_CONF) != 0,
        big_monster_adjacent(st),
    );
    for boost in order {
        let (done, what) = match boost {
            Boost::FearCure => (
                borg_quaff_potion(st, v.sv_potion_boldness)
                    || borg_eat(st, TV_MUSHROOM as i32, v.sv_mush_cure_mind)
                    || borg_quaff_potion(st, v.sv_potion_heroism)
                    || borg_quaff_potion(st, v.sv_potion_berserk),
                "curing fear",
            ),
            Boost::Speed => (
                borg_spell_fail(st, HASTE_SELF, fail) || borg_quaff_potion(st, v.sv_potion_speed),
                "Speed",
            ),
            Boost::Heroism => (
                borg_spell_fail(st, HEROISM, fail) || borg_quaff_potion(st, v.sv_potion_heroism),
                "Heroism",
            ),
            Boost::Berserk => (
                borg_spell_fail(st, BERSERK_STRENGTH, fail)
                    || borg_quaff_potion(st, v.sv_potion_berserk),
                "Berserk Strength",
            ),
            Boost::Blessing => (
                borg_spell_fail(st, BLESS, fail)
                    || borg_read_scroll(st, v.sv_scroll_holy_chant)
                    || borg_read_scroll(st, v.sv_scroll_blessing),
                "Blessing",
            ),
            Boost::MonsterConfusion => (
                borg_read_scroll(st, v.sv_scroll_monster_confusion),
                "Monster Confusion",
            ),
        };
        if done {
            borg_note(st, &format!("# {why}: {what}"));
            return true;
        }
    }
    false
}

/// borg-reborn-rs (#719, owner): with `caution_cornered_throw_first`, a
/// boost waits while no monster is adjacent yet and an attack from range
/// would do damage (`borg_attack_estimate`: throws, shots, attack spells,
/// wands; with nothing adjacent no melee blow is possible). The Borg attacks
/// instead, and buffs once a monster is next to it or nothing reaches. Mode 1
/// is the "Cornered" boost only; mode 2 also the hard breeder fight's (the
/// Phase Door rule's boost is reached only after its own attack found nothing,
/// so mode 2 changes nothing there).
fn throw_first(st: &mut BorgState, why: &str) -> bool {
    let mode = p(Param::caution_cornered_throw_first);
    if mode == 0 {
        return false;
    }
    let adjacent = (1..st.kills.nxt.max(0) as usize).any(|i| {
        st.kills.kills[i].r_idx != 0 && engine::distance(st.borg.c, st.kills.kills[i].pos) <= 1
    });
    // The dry run draws the Borg's random numbers and may clear the waiting
    // flag: both are restored, so a check that does not act changes nothing
    // (as the breeder estimate does).
    if !throw_first_applies(mode, why == "Cornered", adjacent, || {
        let (quick, value) = engine::rand_state();
        let waiting = st.borg.goal.waiting;
        let damage = crate::stateful::attack::borg_attack_estimate(st).unwrap_or(0);
        engine::set_rand_state(quick, value);
        st.borg.goal.waiting = waiting;
        damage
    }) {
        return false;
    }
    if crate::stateful::attack::borg_attack(st, false) {
        borg_note(
            st,
            &format!("# {why}: attacking at range before the boost (#719)"),
        );
        return true;
    }
    false
}

/// The test of `throw_first`: on for this caller, nothing adjacent, and a
/// ranged attack that does damage (asked last: the estimate runs a dry run).
fn throw_first_applies(
    mode: i32,
    cornered_caller: bool,
    adjacent: bool,
    ranged_damage: impl FnOnce() -> i32,
) -> bool {
    (mode >= 2 || (mode == 1 && cornered_caller)) && !adjacent && ranged_damage() > 0
}

/// borg-reborn-rs (#483): while rising to town (a trip or town return),
/// hold a descent that is not a desperate escape. With connected stairs a
/// Borg that arrives on a down staircase and takes it because the level is
/// "scary" (or because no up staircase is known) lands on an up staircase,
/// climbs back onto another down staircase and repeats: it never reaches
/// town. Held, it explores for an up staircase instead. Real danger here
/// (`caution_rising_descend_danger_tenths` of avoidance) or more than
/// `caution_rising_explore_ticks` Borg ticks on the level (one per think,
/// `borg_think`; restarted on each new level) lets the descent through.
pub(crate) fn rising_holds_descent(st: &BorgState, pos_danger: i32) -> bool {
    let budget = p(Param::caution_rising_explore_ticks);
    st.borg.goal.rising
        && crate::Read::new(st).character().cdepth() > 0
        && budget > 0
        && borg_timer(st, st.borg.time.level) < budget
        && pos_danger
            <= i32::from(st.borg.avoidance) * p(Param::caution_rising_descend_danger_tenths) / 10
}

/// `fleeing_or_leaving`: `goal.fleeing || goal.leaving` (the caller's).
fn flight_goal(why: i32, fleeing_or_leaving: bool) -> bool {
    why == GOAL_FLEE || (why == GOAL_BORE && fleeing_or_leaving)
}

/// borg-reborn-rs (#747, owner case 432972050: a fruit bat next to the Borg
/// cancelled its walk to stairs 6 steps away). With
/// `caution_desperate_fast_danger_percent` above 0, a faster adjacent
/// monster stops the desperate walk only when its danger is at least that
/// percent of the HP; 0 = C (any faster monster).
fn fast_neighbour_counts(st: &mut BorgState, kill: usize) -> bool {
    let percent = p(Param::caution_desperate_fast_danger_percent);
    if percent <= 0 {
        return true;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let danger = borg_rs::decision_api::borg_danger::borg_danger_one_kill(
        st,
        y,
        x,
        1,
        kill as i32,
        true,
        false,
    );
    danger_share_reaches(danger, crate::Read::new(st).character().curhp(), percent)
}

fn danger_share_reaches(danger: i32, hp: i32, percent: i32) -> bool {
    i64::from(danger) * 100 >= i64::from(hp.max(1)) * i64::from(percent)
}

/// borg-reborn-rs (#749): enough HP lost for a Recall buy-time cure: at
/// least `min_percent` of the max HP (0 = C: any loss, even none).
fn recall_cure_hurt_enough(lost: i32, max_hp: i32, min_percent: i32) -> bool {
    min_percent <= 0 || i64::from(lost) * 100 >= i64::from(max_hp.max(1)) * i64::from(min_percent)
}

/// Decide which stairs to take when leaving or fleeing (the body of the
/// "Leaving or Fleeing, take stairs" test of `borg_caution`): sets
/// `borg.stair_less` / `borg.stair_more`.
/// borg-reborn-rs (#614, owner): "he can rest on the stairs, and if a mob
/// appears he can climb up/down immediately". With
/// `caution_stairs_leave_on_monster`, a Borg standing on a staircase that
/// still needs to recover (HP or SP below the maximum) takes the stairs as
/// soon as a monster threatens it (danger here at least
/// `caution_stairs_leave_danger_percent` of the HP), instead of fighting or
/// reading an escape. The staircase under the Borg is the way; while rising,
/// a down staircase is held as in `choose_stairs`.
fn leave_stairs_on_monster(st: &mut BorgState, pos_danger: i32) -> bool {
    let feat = u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat);
    let recovering = crate::Read::new(st).character().curhp()
        < crate::Read::new(st).character().maxhp()
        || crate::Read::new(st).character().cursp() < crate::Read::new(st).character().maxsp();
    // Review of #616: a real threat (a share of the HP), not any danger; and
    // never in town, whose down staircase is a dive into DL1.
    let threshold = (crate::Read::new(st).character().curhp()
        * p(Param::caution_stairs_leave_danger_percent)
        / 100)
        .max(1);
    let on = p(Param::caution_stairs_leave_on_monster) != 0;
    let up = feat == FEAT_LESS;
    // Review of #616 (N1): while rising, the holds of #483 and #557 keep a
    // descent off, as they do in choose_stairs (asked only when the rule is on).
    let down = on
        && feat == FEAT_MORE
        && p(Param::caution_stairs_leave_down) != 0
        && !rising_holds_descent(st, pos_danger)
        && !rising_holds_flee_descent(st);
    // borg-reborn-rs (#840): a Borg short of kit, on an up staircase, leaves at
    // the first danger of `kit_leave_danger_percent` of its HP.
    let kit = borg_rs::decision_api::borg_kit::leave_danger_percent(st).is_some_and(|pct| {
        up && pos_danger >= (crate::Read::new(st).character().curhp() * pct / 100).max(1)
    });
    if !(kit
        || stairs_leave_when(
            on,
            crate::Read::new(st).character().cdepth(),
            up || down,
            recovering,
            pos_danger,
            threshold,
        ))
    {
        return false;
    }
    // The staircase under the Borg decides the way: choose_stairs alone may
    // not set it for a Borg that is recovering, not fleeing.
    if up {
        st.borg.stair_less = true;
    } else {
        st.borg.stair_more = true;
    }
    let why = if kit {
        "short of kit"
    } else {
        "while recovering"
    };
    borg_note(
        st,
        &format!(
            "# Leaving by the stairs: a monster {why} (danger {pos_danger}, {}).",
            if up { "up" } else { "down" }
        ),
    );
    true
}

fn stairs_leave_when(
    on: bool,
    depth: i32,
    usable_stairs: bool,
    recovering: bool,
    pos_danger: i32,
    threshold: i32,
) -> bool {
    on && depth > 0 && usable_stairs && recovering && pos_danger >= threshold
}

fn choose_stairs(st: &mut BorgState, pos_danger: i32) {
    let force_descend = engine::player_opt(OPT_birth_force_descend);
    if st.borg.ready_morgoth == 0 && crate::Read::new(st).status().king() == 0 && !force_descend {
        st.borg.stair_less = true;
        if st.borg.mon.scary {
            borg_note(st, "# Fleeing and leaving the level. (scaryguy)");
        }
        if pos_danger > i32::from(st.borg.avoidance)
            && crate::Read::new(st).character().clevel() <= p(Param::caution_stair_danger_max_level)
            && u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) == FEAT_LESS
        {
            borg_note(st, "# Leaving level,  Some danger but I'm on a stair.");
        }
    }

    if st.borg.mon.scary && !engine::player_opt(OPT_birth_force_descend) {
        st.borg.stair_less = true;
    }

    // Only go down if fleeing or prepared
    if st.borg.goal.fleeing {
        st.borg.stair_more = true;
    }

    if borg_rs::decision_api::borg_prepared::prepared_c_for_decision(
        st,
        crate::Read::new(st).character().cdepth() + 1,
    )
    .is_none()
    {
        st.borg.stair_more = true;
    }

    // don't go down if we can go up and are hungry
    if st.stairs.less.num != 0
        && (crate::Read::new(st).character().light() == 0
            || crate::Read::new(st).status().ishungry() != 0
            || crate::Read::new(st).status().isweak() != 0
            || crate::Read::new(st).food().food() < p(Param::caution_stair_min_food))
    {
        st.borg.stair_more = false;
    }

    // If I need to sell crap, then don't go down if I can go up
    if st.stairs.less.num != 0
        && crate::Read::new(st).character().cdepth() != 0
        && crate::Read::new(st).character().clevel() < p(Param::caution_sell_max_level)
        && crate::Read::new(st).abilities().gold() < p(Param::caution_sell_max_gold)
        && borg_count_sell(st) >= p(Param::caution_sell_min_count)
    {
        st.borg.stair_more = false;
    }

    // Its ok to go one level deep if evading scary guy
    if st.borg.mon.scary {
        st.borg.stair_more = true;
    }

    // if fleeing town, then dive
    if crate::Read::new(st).character().cdepth() == 0 {
        st.borg.stair_more = true;
    }

    // borg-reborn-rs (#483): not while rising, unless in real danger.
    if rising_holds_descent(st, pos_danger) {
        st.borg.stair_more = false;
    }

    // borg-reborn-rs (#557): no descent while rising, unless in real danger.
    if st.borg.stair_more && rising_holds_flee_descent(st) {
        st.borg.stair_more = false;
    }
}

/// borg-reborn-rs (#567): with `caution_recall_rest_monster_turns`, record
/// the game turn whenever an awake monster is known within
/// `caution_recall_rest_monster_distance`. priest-809881907 saw Grip 4 grids
/// away, read its only Word of Recall, forgot Grip a think later (out of
/// view: danger 0) and rested for Recall; Grip came back and bit.
fn watch_for_recall(st: &mut BorgState) {
    if p(Param::caution_recall_rest_monster_turns) == 0 {
        return;
    }
    let range = p(Param::caution_recall_rest_monster_distance);
    if let Some(k) = nearest_mover_seen(st, range) {
        st.borg.recall_watch_turn = engine::game_turn();
        st.borg.recall_watch_depth = engine::player_depth();
        st.borg.recall_watch_kill = k;
    }
    // #615 rule a: the same memory for every rest.
    crate::stateful::recovery::watch_for_rest(st);
}

/// An awake monster that can move, seen in this update, within `range` of
/// the Borg (`mover_seen_within`; #615 rule a).
/// The kill index of the nearest one, if any (review of #645: a clock set by
/// a monster is cleared when that monster dies, `monster_died`).
pub(crate) fn nearest_mover_seen(st: &BorgState, range: i32) -> Option<i32> {
    let kills = &st.kills.kills[1..st.kills.nxt.max(1) as usize];
    nearest_mover_seen_in(kills, st.borg.c, range, |r| {
        engine::race_flag_has(usize::from(r), RF_NEVER_MOVE)
    })
    .map(|i| i as i32 + 1)
}

/// An awake monster that can move, seen in this update (`kill.seen`; review
/// of #640: a record keeps its last position for up to 2,000 ticks after the
/// monster is lost, and refreshed the watch every think), within `range`
/// (the engine's distance) of `c`. A monster that never moves (a floating
/// eye, a mold) cannot come back.
///
/// The position in `kills` of the nearest such monster.
fn nearest_mover_seen_in(
    kills: &[borg_rs::decision_api::borg_flow_kill::BorgKill],
    c: loc,
    range: i32,
    never_moves: impl Fn(u16) -> bool,
) -> Option<usize> {
    kills
        .iter()
        .enumerate()
        .filter(|(_, kill)| {
            kill.r_idx != 0
                && kill.seen
                && kill.awake
                && engine::distance(c, kill.pos) <= range
                && !never_moves(kill.r_idx)
        })
        .min_by_key(|(_, kill)| engine::distance(c, kill.pos))
        .map(|(i, _)| i)
}

/// borg-reborn-rs (#567): instead of a long rest for Recall, stay still one
/// turn (`,`, the engine's "stay still"; a rest count of 1 would repeat the
/// last count) while an awake monster was near within the last
/// `caution_recall_rest_monster_turns` game turns on this level. Each think
/// then sees a monster that comes back before it bites.
pub(crate) fn recall_rest_held(st: &mut BorgState) -> bool {
    // Re-review of #640 (N2): the light path's wait runs before borg_caution,
    // so refresh the watch here too.
    watch_for_recall(st);
    let turns = p(Param::caution_recall_rest_monster_turns);
    let ago = engine::game_turn() - st.borg.recall_watch_turn;
    let here = st.borg.recall_watch_depth == engine::player_depth();
    if !recall_watch_holds(turns, st.borg.recall_watch_turn, ago, here) {
        return false;
    }
    borg_note(
        st,
        &format!("# Watching for Recall: a monster was near {ago} game turns ago."),
    );
    borg_keypress(st, key(b','));
    st.borg.in_shop = false;
    true
}

/// The test of `recall_rest_held`: on, a monster recorded (turn above 0), on
/// this level, within `turns` game turns.
pub(crate) fn recall_watch_holds(turns: i32, seen_turn: i32, ago: i32, same_level: bool) -> bool {
    turns > 0 && seen_turn > 0 && same_level && ago <= turns
}

/// Buy time while waiting for recall in danger: mana, phase door, cure
/// potions (the last block but one of `borg_caution`).
fn buy_time_for_recall(st: &mut BorgState) -> bool {
    // `CURSP < MAXSP / 4` (before milestone6 patch 0054 C compared the
    // current mana with itself, true only for negative mana).
    if crate::Read::new(st).status().isconfused() == 0
        && crate::Read::new(st).status().isblind() == 0
        && crate::Read::new(st).character().maxsp() > p(Param::caution_recall_mana_maxsp)
        && crate::Read::new(st).character().cursp()
            < crate::Read::new(st).character().maxsp() / p(Param::caution_recall_mana_divisor)
        && (borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana))
    {
        borg_note(st, "# Buying time waiting for Recall.(1)");
        return true;
    }

    // borg-reborn-rs (#584): through the one Phase Door decision.
    if crate::stateful::escape::phase_door(
        st,
        crate::stateful::escape::PhaseSite::RecallWait,
        crate::stateful::escape::PhaseCheck::Landing {
            tolerance: p(Param::caution_recall_phase_tolerance),
            horizon: p(Param::caution_recall_phase_horizon),
        },
        None,
        |st| {
            borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || borg_spell_fail(st, PHASE_DOOR, p(Param::caution_recall_phase_fail))
                || borg_spell_fail(st, PORTAL, p(Param::caution_recall_portal_fail))
                || borg_activate_item(st, st.act.tele_phase)
        },
    ) {
        borg_note(st, "# Buying time waiting for Recall.(2)");
        return true;
    }

    // borg-reborn-rs (#749): C's "less than 100 HP lost" is true at full HP for
    // a low-level character; with caution_recall_cure_min_loss_percent, a cure
    // needs at least that share of the max HP lost.
    let lost = crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp();
    if lost < p(Param::caution_recall_cure_loss)
        && recall_cure_hurt_enough(
            lost,
            crate::Read::new(st).character().maxhp(),
            p(Param::caution_recall_cure_min_loss_percent),
        )
        && (borg_quaff_crit(st, true)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
    {
        borg_note(st, "# Buying time waiting for Recall.(3)");
        return true;
    }

    if crate::Read::new(st).character().maxhp() - crate::Read::new(st).character().curhp()
        > p(Param::caution_recall_heal_loss)
        && (borg_zap_rod(st, st.item_val.sv_rod_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
            || borg_quaff_potion(st, st.item_val.sv_potion_life)
            || borg_quaff_crit(st, true)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
    {
        borg_note(st, "# Buying time waiting for Recall.(4)");
        return true;
    }
    false
}

/// borg-reborn-rs (#548): cornered for #544's flight down a known
/// staircase. A known monster that can move is adjacent (`nearest_kill` ==
/// 1, diagonals included; #504's `fight_cornered` takes any within
/// `caution_cornered_radius`), it has no Phase Door, teleport, Teleport Level
/// or escape in the pack (the traits count items, not spells; Recall is no
/// escape here, as in #504), and the danger here is at or above the HP or
/// the Borg is surrounded (as #544's `breeder_exit`). The caller has already
/// found no free up staircase known.
fn flee_down_cornered(st: &mut BorgState, pos_danger: i32) -> bool {
    let adjacent = nearest_kill(st);
    let no_escape = crate::Read::new(st).supplies().aphase() <= 0
        && crate::Read::new(st).supplies().ateleport() <= 0
        && crate::Read::new(st).supplies().ateleportlvl() <= 0
        && crate::Read::new(st).supplies().aescape() <= 0;
    let outmatched = pos_danger >= crate::Read::new(st).character().curhp();
    cornered_shape(adjacent, no_escape, outmatched, || borg_surrounded(st))
}

/// `flee_down_cornered`'s test: `adjacent == 1 && no_escape && (outmatched
/// || surrounded)`; `surrounded` (which notes) is asked only when needed.
fn cornered_shape(
    adjacent: i32,
    no_escape: bool,
    outmatched: bool,
    surrounded: impl FnOnce() -> bool,
) -> bool {
    adjacent == 1 && no_escape && (outmatched || surrounded())
}

/// borg-reborn-rs (#548): `borg_flow_stair_more(GOAL_FLEE)` for #544's
/// flight, with the flow's depth capped at `max_cost` steps of flow (0: 250,
/// any distance the flow reaches). It keeps that function's refusals
/// (`more_refused`: with an up staircase known, not prepared for the next
/// depth, hungry, a sale held or no light; recalling; review of #566) and
/// #557's rising hold. Desperate (danger ignored, monster grids routed
/// around). A staircase farther than the cap leaves the cost at the Borg's
/// grid unset, so the commit fails and nothing is noted.
fn flee_known_down_flow(st: &mut BorgState, max_cost: i32) -> bool {
    if rising_holds_flee_descent(st)
        || borg_rs::decision_api::borg_flow_stairs::more_refused(st, false)
    {
        return false;
    }
    // Desperate before the enqueue too: `borg_flow_enqueue_grid` rates an
    // unknown staircase grid's danger unless the flow ignores danger
    // (review of #566; `desperate_flow` set it before the whole flow).
    st.borg.status.desperate = true;
    let weight = stair_flight_weight(GOAL_FLEE, false);
    let depth = flee_down_depth(max_cost);
    let ok = if weight > 0 {
        borg_rs::decision_api::borg_flow_stairs::weighted_down_flight(st, depth, weight)
    } else {
        borg_rs::decision_api::borg_flow::borg_flow_clear(st);
        borg_rs::decision_api::borg_flow_stairs::enqueue_down_stairs(st);
        borg_rs::decision_api::borg_flow::borg_flow_spread(
            st, depth, true, false, false, -1, false,
        );
        borg_rs::decision_api::borg_flow::borg_flow_commit(st, c"down-stairs", GOAL_FLEE) && {
            st.borg.flight_to_stairs = true; // #614: a flight to a staircase
            borg_rs::decision_api::borg_flow::borg_flow_old(st, GOAL_FLEE)
        }
    };
    st.borg.status.desperate = false;
    if ok {
        borg_note(st, "# Desperate for Stairs (down, no up stairs known)");
    }
    ok
}

/// #765: the weight is also the opt-in switch. Case 807683971 takes the
/// bored ascent while fleeing, so the up-stair flow can opt in there too.
pub(crate) fn stair_flight_weight(why: i32, leaving_up: bool) -> i32 {
    if flight_goal(why, leaving_up) {
        p(Param::caution_stairs_flight_threat_weight)
    } else {
        0
    }
}

/// The flow depth for `caution_flee_known_down_max_cost`: 0 is any
/// distance (250, `borg_flow_stair_more`'s depth).
fn flee_down_depth(max_cost: i32) -> i32 {
    if max_cost <= 0 {
        250
    } else {
        max_cost.min(250)
    }
}

fn bump_recent(now: i32, at: i32, turns: i32) -> bool {
    turns > 0 && now >= at && now - at < turns
}

pub(crate) fn unseen_bump_blocked(st: &BorgState, x: i32, y: i32) -> bool {
    let turns = p(Param::caution_unseen_bump_turns);
    turns > 0
        && (x, y) != (st.borg.c.x, st.borg.c.y)
        && st.borg.unseen_bumps.grids.iter().any(|b| {
            b.grid.x == x && b.grid.y == y && bump_recent(engine::game_turn(), b.at, turns)
        })
}

fn bump_heal_due(hp: i32, max: i32, percent: i32) -> bool {
    percent > 0 && hp * 100 < max * percent
}

/// Before a desperate stair walk: cures do not require the unknown monster
/// to have a race (and therefore an estimated danger). Detection is tried
/// once per observation; oil is bounded per grid; then a Phase Door. None
/// of these invents a monster index, HP, armour or a clear target.
fn unseen_bump_response(st: &mut BorgState, danger: i32) -> bool {
    let turns = p(Param::caution_unseen_bump_turns);
    if turns == 0 || crate::Read::new(st).character().cdepth() == 0 {
        return false;
    }
    let now = engine::game_turn();
    let Some(i) = st.borg.unseen_bumps.grids.iter().position(|b| {
        bump_recent(now, b.at, turns)
            && engine::distance(st.borg.c, b.grid) == 1
            && st.cave.grid(b.grid.x, b.grid.y).kill == 0
    }) else {
        return false;
    };
    if bump_heal_due(
        crate::Read::new(st).character().curhp(),
        crate::Read::new(st).character().maxhp(),
        p(Param::caution_unseen_bump_heal_hp_percent),
    ) && (borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
        || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical))
    {
        borg_note(
            st,
            "# Healing before stairs: an unseen monster blocks the step",
        );
        return true;
    }
    if p(Param::caution_unseen_bump_response) == 0 {
        return false;
    }
    if !st.borg.unseen_bumps.grids[i].detection_read
        && crate::stateful::defence::unseen_detection_safe(st, danger)
        && borg_read_scroll(st, st.item_val.sv_scroll_detect_invis)
    {
        st.borg.unseen_bumps.grids[i].detection_read = true;
        borg_note(st, "# Detect Invisible: an unseen monster blocks the step");
        return true;
    }
    let g = st.borg.unseen_bumps.grids[i].grid;
    if crate::Read::new(st).status().isconfused() == 0
        && crate::Read::new(st).status().isblind() == 0
        && crate::Read::new(st).status().isimage() == 0
        && st.borg.unseen_bumps.grids[i].throws < p(Param::caution_unseen_bump_throw_tries)
        && crate::stateful::attack::throw_at_unseen_bump(st, g)
    {
        st.borg.unseen_bumps.grids[i].throws += 1;
        st.borg.unseen_bumps.shot = Some(UnseenOilShot {
            grid: g,
            dead: false,
            resolved: false,
        });
        return true;
    }
    // The ordinary adjacent-monster rule cannot rate a monster with no
    // known race. This site's explicit, bounded bump evidence is its gate.
    if crate::stateful::escape::unseen_bump_phase(st, danger) {
        st.borg.escapes = st.borg.escapes.wrapping_add(1);
        st.borg.goal.type_0 = 0;
        return true;
    }
    false
}

/// #811: apply self-care before routing in darkness. This cannot start a flow.
pub(crate) fn lightless_self_care(st: &mut BorgState, danger: i32) -> bool {
    if borg_rs::decision_tape::active() {
        borg_rs::decision_tape::mark(
            "policy::caution.lightless_self_care",
            format!("{:?}", (danger,)),
        );
    }
    // Cure known poison before paying for the walk with HP, even when the
    // ordinary fight rule would wait for lower HP.
    (crate::Read::new(st).status().ispoisoned() != 0
        && danger <= i32::from(st.borg.avoidance) * p(Param::caution_cure_safe_danger_tenths) / 10
        && borg_quaff_potion(st, st.item_val.sv_potion_cure_poison))
        || borg_heal(st, danger)
        || (crate::Read::new(st).status().ishungry() != 0
            && crate::stateful::item_use::borg_eat_when_hungry(st))
        || (crate::Read::new(st).status().isweak() != 0 && borg_eat_food_any(st))
}

enum DesperateStairRange {
    Nearby(i32),
    Remaining(i32),
}

impl DesperateStairRange {
    fn moves(&self) -> i32 {
        match *self {
            Self::Nearby(moves) | Self::Remaining(moves) => moves,
        }
    }
}

/// #1052: caution has had its full turn to act on the current danger.
/// Replan the remaining escape before the dungeon thinker resumes exploring.
fn resume_emergency_down_flight(st: &mut BorgState, danger: i32) -> bool {
    let Some(handoff) = st.stairs.emergency_descent() else {
        return false;
    };
    if !handoff.can_resume(
        st.borg.c,
        st.borg.time.level,
        st.borg.time.now,
        danger,
        crate::Read::new(st).character().curhp(),
    ) || crate::Read::new(st).status().isconfused() != 0
        || rising_holds_descent(st, danger)
    {
        st.stairs.clear_emergency_descent();
        return false;
    }

    if u32::from(st.cave.grid(st.borg.c.x, st.borg.c.y).feat) == FEAT_MORE {
        st.stairs.clear_emergency_descent();
        if !rising_holds_flee_descent(st)
            && !borg_rs::decision_api::borg_flow_stairs::more_refused(st, false)
        {
            borg_note(st, "# Taking down stairs after emergency flight (#1052)");
            borg_keypress(st, key(b'>'));
            return true;
        }
    } else if handoff.steps_left > 0
        && desperate_flow(
            st,
            false,
            DesperateStairRange::Remaining(handoff.steps_left),
            "# Continuing emergency down-stair flight (#1052)",
        )
    {
        return true;
    }

    st.stairs.clear_emergency_descent();
    false
}

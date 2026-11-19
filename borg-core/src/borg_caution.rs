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
use crate::borg::BorgTime;
use crate::borg_cave::{BorgGrid, AUTO_MAX_X, AUTO_MAX_Y};
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid};
use crate::borg_danger::borg_danger;
use crate::borg_escape::{
    borg_escape, borg_freedom, borg_recall, borg_surrounded,
};
use crate::borg_fight_defend::borg_defend;
use crate::borg_flow_stairs::{
    borg_flow_stair_less, borg_flow_stair_more, borg_prep_leave_level_spells,
};
use crate::borg_headers::borg_h::BORG_PLAYS_RISKY;
use crate::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, BLESS, CURE_POISON, HASTE_SELF, HEALING, HERBAL_CURING, HEROISM, HOLY_WORD, MINOR_HEALING,
    PHASE_DOOR, PORTAL, REMOVE_HUNGER, TELEPORT_LEVEL,
};
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_AESCAPE, BI_AEZHEAL, BI_AFUEL, BI_AHEAL, BI_AMISSILES, BI_APHASE, BI_ATELEPORT, BI_ATELEPORTLVL, BI_CDEPTH,
    BI_CLASS, BI_CLEVEL, BI_CRSFEAR, BI_CURHP, BI_CURSP, BI_ESP, BI_FOOD, BI_GOLD, BI_IFIRE,
    BI_ISAFRAID, BI_ISBLIND, BI_ISCONFUSED, BI_ISCUT, BI_ISFIXCON, BI_ISFIXDEX, BI_ISFIXINT,
    BI_ISFIXSTR, BI_ISFIXWIS, BI_ISHEAVYSTUN, BI_ISHUNGRY, BI_ISIMAGE, BI_ISPOISONED, BI_ISSTUN,
    BI_ISWEAK, BI_KING, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP, BI_RECALL, BI_SPEED,
};
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, KC_ENTER};
use crate::borg_item_use::{
    borg_activate_failure, borg_activate_item, borg_eat, borg_eat_food_any, borg_satisfy_hunger_before_food,
    borg_equips_staff_fail, borg_quaff_crit, borg_quaff_potion, borg_read_scroll, borg_use_staff, borg_use_staff_fail, borg_zap_rod,
};
use crate::borg_item_val::borg_lookup_kind;
use crate::borg_item_wear::borg_backup_swap;
use crate::borg_light::{borg_maintain_light, BorgNeed};
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_legal};
use crate::borg_store_sell::borg_count_sell;
use crate::engine;
use crate::headers::list_mon_race_flags_h::{RF_MULTIPLY, RF_NEVER_MOVE, RF_NO_CONF, RF_UNIQUE};
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_player_timed_h::{TMD_ATT_CONF, TMD_OPP_ACID};
use crate::headers::list_terrain_h::{FEAT_LAVA, FEAT_LESS, FEAT_MORE, FEAT_NONE};
use crate::headers::list_tvals_h::{TV_MUSHROOM, TV_POTION, TV_ROD, TV_STAFF};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::params::{value as p, Param};
use crate::state::BorgState;

const CLASS_WARRIOR: i32 = 0;
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;

/// `GOAL_FLEE` (borg-flow.h): flee the level.
const GOAL_FLEE: i32 = 7;
use crate::borg_flow_take::GOAL_TAKE;

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
    st.borg.trait_[BI_CURSP as usize] = v;
}

/// The "am I bouncing between two grids" test of the retreat and back-away
/// scans: healthy, and (x, y) is where the Borg was two steps ago while it
/// was here three steps ago; or the anti-bounce counter is high.
fn bouncing(st: &BorgState, y: i32, x: i32) -> bool {
    let ts = &st.flow.track_step;
    (st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) * p(Param::caution_bounce_healthy_tenths) / 10
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
    (st.borg.tr(BI_ATELEPORT) == 0 && st.borg.tr(BI_AESCAPE) == 0) || rod_good
}

/// `borg.goal.leaving = true` with its note, if not leaving yet.
fn start_leaving(st: &mut BorgState, why: &str) {
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
    let tr = |i| st.borg.tr(i);
    let mut n = 0;
    for f in [
        BI_ISFIXSTR,
        BI_ISFIXINT,
        BI_ISFIXWIS,
        BI_ISFIXDEX,
        BI_ISFIXCON,
    ] {
        if tr(f) != 0 {
            n += 1;
        }
    }

    // Special cases get a second vote
    let class = tr(BI_CLASS);
    if class == CLASS_MAGE && tr(BI_ISFIXINT) != 0 {
        n += p(Param::caution_caster_stat_vote);
    }
    if class == CLASS_PRIEST && tr(BI_ISFIXWIS) != 0 {
        n += p(Param::caution_priest_stat_vote);
    }
    if class == CLASS_DRUID && tr(BI_ISFIXWIS) != 0 {
        n += p(Param::caution_druid_stat_vote);
    }
    if class == CLASS_NECROMANCER && tr(BI_ISFIXINT) != 0 {
        n += p(Param::caution_necromancer_stat_vote);
    }
    if class == CLASS_WARRIOR && tr(BI_ISFIXCON) != 0 {
        n += p(Param::caution_warrior_stat_vote);
    }
    if tr(BI_MAXHP) <= p(Param::caution_stat_fix_hp_high) && tr(BI_ISFIXCON) != 0 {
        n += p(Param::caution_stat_fix_vote_high);
    }
    if tr(BI_MAXHP) <= p(Param::caution_stat_fix_hp_low) && tr(BI_ISFIXCON) != 0 {
        n += p(Param::caution_stat_fix_vote_low);
    }
    if class == CLASS_PRIEST
        && tr(BI_MAXSP) < p(Param::caution_priest_stat_fix_sp)
        && tr(BI_ISFIXWIS) != 0
    {
        n += p(Param::caution_priest_stat_fix_votes);
    }
    if class == CLASS_MAGE
        && tr(BI_MAXSP) < p(Param::caution_mage_stat_fix_sp)
        && tr(BI_ISFIXINT) != 0
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
        st.borg.tr(BI_CURHP),
        st.borg.tr(BI_MAXHP),
        danger,
        p(Param::caution_heal_outmatched_min_loss),
    ) {
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

    let hp_down = st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP);
    let pct_down = (st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP)) * 100 / st.borg.tr(BI_MAXHP);
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
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        if pct_down >= p(Param::caution_confused_heal_loss)
            && danger - heal_heal < st.borg.tr(BI_CURHP)
            && borg_quaff_potion(st, st.item_val.sv_potion_healing)
        {
            borg_note(st, "# Fixing Confusion. Level 1");
            return true;
        }
        if pct_down >= p(Param::caution_confused_star_loss)
            && danger >= st.borg.tr(BI_CURHP) * p(Param::caution_confused_star_danger_multiplier)
            && (borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
                || borg_quaff_potion(st, st.item_val.sv_potion_life))
        {
            borg_note(st, "# Fixing Confusion. Level 1.a");
            return true;
        }
        if danger < st.borg.tr(BI_CURHP) + csw_heal
            && (borg_eat(st, TV_MUSHROOM as i32, st.item_val.sv_mush_cure_mind)
                || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
                || borg_quaff_crit(st, false)
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
                || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            {
                borg_note(st, "# Fixing Confusion. Level 5");
                return true;
            }
        }
    }
    // Heal when blind. This is deadly.
    if st.borg.tr(BI_ISBLIND) != 0 && engine::randint0(100) < p(Param::caution_blind_cure_chance) {
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
        if !(st.borg.tr(BI_CLASS) == CLASS_WARRIOR
            && st.borg.tr(BI_CURHP)
                > st.borg.tr(BI_MAXHP) / p(Param::caution_blind_warrior_hp_divisor)
            && st.borg.tr(BI_ESP) != 0)
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
    if (st.borg.tr(BI_ISBLIND) != 0 || st.borg.tr(BI_ISCONFUSED) != 0)
        && (hp_down >= p(Param::caution_blind_confused_star_loss)
            || (danger > st.borg.tr(BI_CURHP) * p(Param::caution_blind_confused_danger_multiplier)
                && hp_down > p(Param::caution_blind_confused_min_loss)))
        && borg_quaff_potion(st, st.item_val.sv_potion_star_healing)
    {
        borg_note(st, "# Fixing Confusion/Blind.");
        return true;
    }

    // Healing and fighting Morgoth.
    if st.borg.near.unique >= 10
        && st.borg.tr(BI_CURHP) <= p(Param::caution_questor_heal_hp)
        && ((st.borg.tr(BI_CURHP) > p(Param::caution_questor_holy_word_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_holy_word_fail)))
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
            || (st.borg.tr(BI_CURHP) < p(Param::caution_questor_emergency_holy_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_emergency_holy_fail)))
            || (st.borg.tr(BI_CURHP) > p(Param::caution_questor_safe_holy_hp) && borg_spell_fail(st, HOLY_WORD, p(Param::caution_questor_safe_holy_fail)))
            || borg_spell_fail(st, HEALING, p(Param::caution_questor_healing_fail))
            || borg_quaff_potion(st, st.item_val.sv_potion_life)
            || borg_zap_rod(st, st.item_val.sv_rod_healing))
    {
        borg_note(st, "# Healing in Questor Combat.");
        return true;
    }

    // restore Mana
    // note, blow the staff charges easy because the staff will not last.
    if st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) / p(Param::caution_magi_sp_divisor)
        && engine::randint0(100) < p(Param::caution_magi_chance)
        && (borg_use_staff_fail(st, st.item_val.sv_staff_the_magi)
            || borg_activate_item(st, st.act.staff_magi))
    {
        borg_note(st, "# Use Magi Staff");
        return true;
    }
    // blowing potions is harder
    // NOTE: must have enough mana to keep up or do a HEAL
    if (st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) / p(Param::caution_restore_mana_sp_divisor)
        || (st.borg.tr(BI_CURSP) < p(Param::caution_restore_mana_sp) && st.borg.tr(BI_MAXSP) > p(Param::caution_restore_mana_maxsp)))
        // use the potion if battling a unique and not too dangerous
        && (st.borg.near.unique >= 10
            || (st.borg.near.unique != 0 && danger < avoidance(st) * p(Param::caution_restore_mana_danger_multiplier))
            || (st.borg.tr(BI_ATELEPORT) + st.borg.tr(BI_AESCAPE) == 0
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
    if danger == 0 && st.borg.tr(BI_ISPOISONED) == 0 && st.borg.tr(BI_ISCUT) == 0 {
        return false;
    }

    // Restoring while fighting Morgoth
    if stats_needing_fix >= p(Param::caution_restore_stat_votes)
        && st.borg.near.unique >= 10
        && st.borg.tr(BI_CURHP) > p(Param::caution_restore_stat_hp)
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

    // Heal when wounded a percent of the time
    let mut chance = engine::randint0(100);

    // if we are fighting a unique increase the odds of healing
    if st.borg.near.unique != 0 {
        chance += p(Param::caution_heal_unique_chance_bonus);
    }

    // if danger is close to the hp and healing will help, do it
    if danger >= st.borg.tr(BI_CURHP) && danger < st.borg.tr(BI_MAXHP) {
        chance += p(Param::caution_heal_lethal_chance_bonus);
    } else if st.borg.tr(BI_CLASS) != CLASS_PRIEST && st.borg.tr(BI_CLASS) != CLASS_PALADIN {
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
        && (st.borg.tr(BI_ISHEAVYSTUN) == 0
            && st.borg.tr(BI_ISSTUN) == 0
            && st.borg.tr(BI_ISPOISONED) == 0
            && st.borg.tr(BI_ISCUT) == 0)
    {
        return false;
    }

    // Cure light Wounds (20)
    if pct_down >= p(Param::caution_clw_min_loss)
        && (pct_down <= p(Param::caution_clw_max_loss) || st.borg.tr(BI_CLEVEL) < p(Param::caution_clw_max_level))
        && danger < st.borg.tr(BI_CURHP) + clw_heal
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
        && (pct_down <= p(Param::caution_csw_max_loss) || st.borg.tr(BI_CLEVEL) < p(Param::caution_csw_max_level))
        && danger < st.borg.tr(BI_CURHP) + csw_heal
        && csw_heal > danger / p(Param::caution_csw_danger_divisor) // No rope-a-doping
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_activate_item(st, st.act.cure_serious))
    {
        borg_note(st, "# Healing Level 2.");
        return true;
    }

    // Cure Critical Wounds (60)
    if (p(Param::caution_ccw_min_loss)..=p(Param::caution_ccw_max_loss)).contains(&pct_down)
        && danger < st.borg.tr(BI_CURHP) + ccw_heal
        && ccw_heal > danger / p(Param::caution_ccw_danger_divisor) // No rope-a-doping
        && (borg_activate_item(st, st.act.cure_critical) || borg_quaff_crit(st, false))
    {
        borg_note(st, "# Healing Level 3.");
        return true;
    }

    // If in danger try  one more Cure Critical if it will help
    if danger >= st.borg.tr(BI_CURHP)
        && danger < st.borg.tr(BI_MAXHP)
        && st.borg.tr(BI_CURHP) < p(Param::caution_emergency_ccw_hp)
        && danger < ccw_heal
        && borg_quaff_crit(st, true)
    {
        borg_note(st, "# Healing Level 5.");
        return true;
    }

    // if deep, and low on HP, but in a zero danger spot, drink some CCW to
    // add a few HP before resting
    if st.borg.tr(BI_CDEPTH) >= p(Param::caution_deep_ccw_depth)
        && danger < p(Param::caution_deep_ccw_danger)
        && pct_down >= p(Param::caution_deep_ccw_loss)
        && borg_quaff_potion(st, st.item_val.sv_potion_cure_critical)
    {
        borg_note(st, "# Healing Level 5B.");
        return true;
    }

    // Heal step one (200hp)
    if pct_down >= p(Param::caution_heal_device_min_loss)
        && danger < st.borg.tr(BI_CURHP) + heal_heal
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
    if st.borg.tr(BI_MAXDEPTH) >= p(Param::caution_conserve_heal_depth)
        && st.borg.tr(BI_KING) == 0
        && st.borg.near.unique == 0
        && st.borg.tr(BI_CLASS) != CLASS_PRIEST
    {
        // Bail out to save the heal pots for Morgoth
        return false;
    }

    // Heal step two (300hp)
    if pct_down > p(Param::caution_heal_potion_min_loss)
        && danger < st.borg.tr(BI_CURHP) + heal_heal
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
        && danger < st.borg.tr(BI_CURHP) + heal_heal
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
        && danger < st.borg.tr(BI_CURHP) + heal_heal
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
        && danger > st.borg.tr(BI_CURHP)
        && st.borg.tr(BI_ATELEPORT) + st.borg.tr(BI_AESCAPE) <= 0
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
    if st.borg.tr(BI_ISPOISONED) != 0
        && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_poison_cure_hp_divisor)
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
    if st.borg.tr(BI_ISPOISONED) != 0
        && (st.borg.tr(BI_CURHP) < p(Param::caution_poison_critical_hp)
            || st.borg.tr(BI_CURHP)
                < st.borg.tr(BI_MAXHP) / p(Param::caution_poison_critical_hp_divisor))
    {
        let sv_mana = st.borg.tr(BI_CURSP);

        set_cursp(st, st.borg.tr(BI_MAXSP));

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
        if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_poison_reserve_hp_divisor)
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
    if st.borg.tr(BI_ISCUT) != 0
        && (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_cut_cure_hp_divisor)
            || engine::randint0(100) < p(Param::caution_cut_cure_chance))
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_crit(
                st,
                st.borg.tr(BI_CURHP) < p(Param::caution_cut_critical_ccw_hp),
            )
            || borg_spell(st, MINOR_HEALING)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical))
    {
        return true;
    }
    // bleeding and about to die CRITICAL CHECK
    if st.borg.tr(BI_ISCUT) != 0
        && (st.borg.tr(BI_CURHP) < p(Param::caution_cut_critical_hp)
            || st.borg.tr(BI_CURHP)
                < st.borg.tr(BI_MAXHP) / p(Param::caution_cut_critical_hp_divisor))
    {
        let sv_mana = st.borg.tr(BI_CURSP);

        set_cursp(st, st.borg.tr(BI_MAXSP));

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
        if st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_cut_reserve_hp_divisor)
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
        if st.cave.grid(x, y).kill != 0 {
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

/// `borg_flow_stair_less/more(GOAL_FLEE, ...)` with `borg.status.desperate`
/// set around the call; notes `why` when it worked.
fn desperate_flow(st: &mut BorgState, up: bool, why: &str) -> bool {
    st.borg.status.desperate = true;
    let ok = if up {
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
    // Check for an existing "up stairs"
    let b_j = nearest_free_stair(st, true);

    // Check for an existing "down stairs"
    let b_m = nearest_free_stair(st, false);

    // If you are within a few (3) steps of the stairs
    // and you can take some damage to get there
    // go for it
    if b_j < p(Param::caution_desperate_stair_distance)
        && b_j != -1
        && pos_danger < st.borg.tr(BI_CURHP)
        && desperate_flow(st, true, "# Desperate for Stairs (one)")
    {
        return true;
    }

    // If you are next to steps of the stairs go for it
    if b_j <= p(Param::caution_desperate_adjacent_up_distance)
        && b_j != -1
        && desperate_flow(st, true, "# Desperate for Stairs (two)")
    {
        return true;
    }

    // Low level guys tend to waste money reading the recall scrolls
    if b_j < p(Param::caution_desperate_scary_distance)
        && b_j != -1
        && st.borg.mon.scary
        && st.borg.tr(BI_CLEVEL) < p(Param::caution_desperate_scary_level)
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
            if i32::from(st.kills.kills[usize::from(kill)].speed) > st.borg.tr(BI_SPEED) {
                safe = false;
            }
        }

        // Don't run from Grip or Fang
        if (st.borg.tr(BI_CDEPTH) <= p(Param::caution_desperate_unique_depth)
            && st.borg.tr(BI_CDEPTH) != 0
            && st.borg.near.unique != 0)
            || !safe
        {
            // try to take them on, you cant outrun them
        } else if desperate_flow(st, true, "# Desperate for Stairs (three)") {
            return true;
        }
    }

    // If you are next to steps of the down stairs go for it
    if b_m <= p(Param::caution_desperate_adjacent_down_distance)
        && b_m != -1
        && desperate_flow(st, false, "# Desperate for Stairs (four)")
    {
        return true;
    }

    // borg-reborn-rs (#544, narrowed in #548): in the dungeon, fleeing with
    // no up staircase known, a known down staircase (an escape: the Borg
    // comes back up when not prepared for the depth). Only when cornered
    // (`caution_flee_known_down_cornered`) and within
    // `caution_flee_known_down_max_cost` grids of flow.
    p(Param::caution_flee_known_down_stairs) != 0
        && st.borg.tr(BI_CDEPTH) > 0
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
    let mut p1 = -1;
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
        if st.borg.tr(BI_ISCONFUSED) != 0 {
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
        if !crate::borg_flow_misc::borg_happy_grid_bold(st, y2, x2) {
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
            let d = crate::borg_flow_misc::borg_goto_dir(st, y1, x1, y2, x2);

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
                || (u32::from(ag.feat) == FEAT_LAVA && st.borg.tr(BI_IFIRE) == 0)
            {
                break;
            }

            // Require it to be somewhat close
            if r >= p(Param::caution_retreat_max_distance) {
                break;
            }

            // Check danger of that spot
            p1 = borg_danger(
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
                    if p1 >= st.borg.tr(BI_CURHP) * p(Param::caution_retreat_step_hp_tenths) / 10 {
                        break;
                    }

                    // Ought to be worth it
                    if p1 > pos_danger * p(Param::caution_retreat_step_danger_tenths) / 10 {
                        break;
                    }
                } else {
                    // Surrounded, try to back-up
                    if st.borg.tr(BI_CLEVEL) >= p(Param::caution_retreat_surrounded_level) {
                        let limit = if b_r <= p(Param::caution_retreat_surrounded_distance) {
                            st.borg.tr(BI_CURHP) * p(Param::caution_retreat_surrounded_hp_tenths)
                                / 10
                        } else {
                            st.borg.tr(BI_CURHP)
                        };
                        if p1 >= limit {
                            break;
                        }
                    } else if p1
                        >= st.borg.tr(BI_CURHP) * p(Param::caution_retreat_low_level_hp_multiplier)
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
                        && i32::from(kill.speed) > st.borg.tr(BI_SPEED)
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
        let b_d = crate::borg_flow_misc::borg_goto_dir(st, st.borg.c.y, st.borg.c.x, b_y, b_x);

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
    u32::from(ag.feat) == FEAT_LAVA && st.borg.tr(BI_IFIRE) == 0
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
        if st.borg.tr(BI_ISCONFUSED) != 0 {
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
                && i32::from(kill.speed) > st.borg.tr(BI_SPEED)
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
        if st.borg.tr(BI_MAXCLEVEL) >= p(Param::caution_back_high_level)
            && k > b_k * p(Param::caution_back_high_level_danger_tenths) / 10
        {
            continue;
        }

        // Skip this grid if it is not really worth backing up.  If the
        // danger of the new grid is close to the danger of my current grid,
        // I'll stay and fight unless I am low level and there is an
        // adjacent monster.
        if st.borg.tr(BI_MAXCLEVEL) < p(Param::caution_back_low_level)
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
            if st.borg.tr(BI_CLEVEL) <= p(Param::caution_back_injured_level)
                && st.borg.tr(BI_CDEPTH) != 0
                && (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP)
                    || st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP))
            {
                // do consider the retreat
            } else {
                // Freedom of my grid is better than the next grid
                // so stay put and fight.
                if b_f > f || st.borg.tr(BI_CDEPTH) >= p(Param::caution_back_freedom_depth) {
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
        && st.borg.tr(BI_ISCONFUSED) == 0
        && !engine::square_isvault(st.borg.c.x, st.borg.c.y)
        && st.borg.tr(BI_CURHP) < p(Param::caution_move_max_hp)
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
    if st.borg.tr(BI_ISAFRAID) != 0 {
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
        if crate::borg_fight_attack::borg_thrust_damage_one(st, i) >= kill.power as i32 {
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
pub fn borg_caution(st: &mut BorgState) -> bool {
    let mut nasty = false;

    /*** Notice "nasty" situations ***/

    // About to run out of light is extremely nasty
    if st.borg.tr(BI_LIGHT) == 0
        && i32::from(st.items.items[inven_light()].timeout) < p(Param::caution_nasty_light_timeout)
    {
        nasty = true;
    }

    // Starvation, blindness, confusion, hallucination are nasty
    for f in [BI_ISWEAK, BI_ISBLIND, BI_ISCONFUSED, BI_ISIMAGE] {
        if st.borg.tr(f) != 0 {
            nasty = true;
        }
    }

    // if on level 100 and not ready for Morgoth, run
    if st.borg.tr(BI_CDEPTH) == 100
        && borg_timer(st, st.borg.time.level) < p(Param::caution_questor_arrival_turns)
        && !st.borg.morgoth_position
        && st.borg.ready_morgoth <= 0
        && st.borg.tr(BI_KING) == 0
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
        && st.borg.tr(BI_CDEPTH) <= p(Param::caution_escape_limit_max_depth)
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
        let before = (st.borg.goal.leaving, st.borg.goal.fleeing, st.borg.goal.fleeing_to_town);

        // Start leaving
        start_leaving(st, "# Leaving (Scary guy on level)");

        // Start fleeing
        start_fleeing(st, "# Fleeing (Scary guy on level)");

        // Return to town quickly after leaving town
        if st.borg.tr(BI_CDEPTH) == 0 {
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
        || st.borg.tr(BI_ISWEAK) != 0
        || st.borg.tr(BI_CDEPTH) == 100)
        && st.borg.tr(BI_KING) == 0
    {
        // Describe (briefly) the current situation
        let note = format!(
            "# Loc:{},{} Dep:{} Lev:{} HP:{}/{} SP:{}/{} Danger:p={}",
            st.borg.c.y,
            st.borg.c.x,
            st.borg.tr(BI_CDEPTH),
            st.borg.tr(BI_CLEVEL),
            st.borg.tr(BI_CURHP),
            st.borg.tr(BI_MAXHP),
            st.borg.tr(BI_CURSP),
            st.borg.tr(BI_MAXSP),
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

    if !st.borg.goal.fleeing {
        // Start being cautious and trying to not die
        if st.borg.tr(BI_CLASS) == CLASS_MAGE
            && !st.borg.morgoth_position
            && !st.borg.status.anti_summon
            && st.borg.tr(BI_ISBLIND) == 0
            && st.borg.tr(BI_ISCUT) == 0
            && st.borg.tr(BI_ISPOISONED) == 0
            && st.borg.tr(BI_ISCONFUSED) == 0
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
    if crate::borg_trait::borg_uses_swaps(st)
        && pos_danger > i32::from(st.borg.avoidance) / p(Param::caution_swap_danger_divisor)
        && borg_backup_swap(st, pos_danger)
    {
        return true;
    }

    // If I am waiting for recall,  & safe, then stay put.
    if st.borg.goal.recalling != 0
        && crate::borg_flow_misc::borg_check_rest(st, st.borg.c.y, st.borg.c.x)
        && st.borg.tr(BI_CDEPTH) != 0
        && st.borg.tr(BI_ISHUNGRY) == 0
    {
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
        && st.borg.tr(BI_CDEPTH) == 0
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
        && crate::borg_prepared::borg_restock_c(st, st.borg.tr(BI_CDEPTH)).is_some()
    {
        // Start leaving
        if !st.borg.goal.leaving {
            // Note
            let r = crate::borg_prepared::borg_restock_c(st, st.borg.tr(BI_CDEPTH));
            note_with_reason(st, b"# Leaving (restock) ", r);

            // Start leaving
            st.borg.goal.leaving = true;
        }
        // Start fleeing
        if !st.borg.goal.fleeing
            && st.borg.tr(BI_ACCW) < p(Param::caution_restock_ccw_count)
            && st.borg.tr(BI_FOOD) > p(Param::caution_restock_food_count)
            && st.borg.tr(BI_AFUEL) > p(Param::caution_restock_fuel_count)
            && borg_timer(st, st.borg.time.level) > p(Param::caution_restock_level_delay)
        {
            // Flee
            let r = crate::borg_prepared::borg_restock_c(st, st.borg.tr(BI_CDEPTH));
            note_with_reason(st, b"# Fleeing (restock) ", r);

            // Start fleeing
            st.borg.goal.fleeing = true;
        }
    }
    // Excessive danger
    else if pos_danger > st.borg.tr(BI_CURHP) * p(Param::caution_flee_danger_hp_multiplier) {
        // Start fleeing
        // do not flee level if going after Morgoth or fighting a unique
        if !st.borg.goal.fleeing
            && st.borg.near.unique == 0
            && st.borg.tr(BI_CLEVEL) < p(Param::caution_flee_max_level)
            && !st.borg.status.vault
            && (st.borg.tr(BI_CDEPTH) < 100 && st.borg.ready_morgoth != 1)
        {
            // Note
            borg_note(st, "# Fleeing (excessive danger)");

            // Start fleeing
            st.borg.goal.fleeing = true;
        }
    }
    // Potential danger (near death) in town
    else if st.borg.tr(BI_CDEPTH) == 0
        && pos_danger > st.borg.tr(BI_CURHP)
        && st.borg.tr(BI_CLEVEL) < p(Param::caution_town_flee_max_level)
    {
        // Flee now
        start_leaving(st, "# Leaving (potential danger)");
    }

    /*** Stairs ***/

    let avoidance = i32::from(st.borg.avoidance);
    // Leaving or Fleeing, take stairs
    if st.borg.goal.leaving
        || st.borg.goal.fleeing
        || st.borg.mon.scary
        || ((pos_danger > avoidance
            || (st.borg.tr(BI_CLEVEL) < p(Param::caution_stair_low_level)
                && pos_danger > avoidance / p(Param::caution_stair_danger_divisor)))
            && on_upstair)
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
    if st.borg.stair_more && st.borg.goal.recalling == 0 {
        // borg-reborn-rs (#208): /shopping is town-only, even if caution
        // chooses a direct descent before the ordinary town-leave hook.
        if crate::shell::shopping_departure(st) {
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
    if st.borg.tr(BI_LIGHT) == 0 {
        // No Lite, AND Not Glowing
        let need = borg_maintain_light(st);
        if need == BorgNeed::MetNeed {
            return true;
        } else if need == BorgNeed::UnmetNeed && st.borg.tr(BI_CDEPTH) != 0 {
            // Flee for fuel
            start_leaving(st, "# Leaving (need fuel)");
        }
    }

    // Prevent starvation
    if st.borg.tr(BI_ISWEAK) != 0 {
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
        if st.borg.tr(BI_CDEPTH) != 0 {
            start_leaving(st, "# Leaving (need food)");
            start_fleeing(st, "# Fleeing (need food)");
        }
    }

    // Prevent breeder explosions when low level
    // borg-reborn-rs (#503): not without an easy escape: stand and fight.
    if st.borg.near.breeder
        && st.borg.tr(BI_CLEVEL) < p(Param::caution_breeder_flee_max_level)
        && !breeder_stand(st)
    {
        start_fleeing(st, "# Fleeing (breeder level)");
    }

    /*** Flee on foot ***/

    // Desperation Head for stairs
    if (st.stairs.less.num != 0 || st.stairs.more.num != 0)
        && (st.borg.goal.fleeing
            || st.borg.mon.scary
            || (pos_danger > i32::from(st.borg.avoidance)
                && st.borg.tr(BI_CLEVEL) < p(Param::caution_desperate_max_level)))
        && desperate_for_stairs(st, pos_danger)
    {
        return true;
    }

    // borg-reborn-rs (#544): only breeders near and outmatched, walk out of
    // their room instead of fighting where it stands.
    if breeder_exit(st, pos_danger) {
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
    if !all_out
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
        && may_move_away(st, danger_test, nasty, borg_surround, pos_danger, false)
        && back_away(st, pos_danger, borg_surround)
    {
        return true;
    }

    /*** Cures ***/

    // cure confusion, second check, first (slightly different) in
    // borg_heal
    if st.borg.tr(BI_ISCONFUSED) != 0 {
        if st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP) >= p(Param::caution_confused_emergency_loss)
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
            || borg_quaff_potion(st, st.item_val.sv_potion_healing)
            || borg_activate_item(st, st.act.cure_confusion)
            || borg_use_staff_fail(st, st.item_val.sv_staff_healing)
        {
            borg_note(st, "# Healing.  Confusion.");
            return true;
        }
    }

    // Cure fear when afraid
    if (st.borg.tr(BI_ISAFRAID) != 0 && st.borg.tr(BI_CRSFEAR) == 0)
        && (engine::randint0(100) < p(Param::caution_fear_cure_chance)
            || (st.borg.tr(BI_CLASS) == CLASS_WARRIOR && st.borg.tr(BI_AMISSILES) <= 0))
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
    if (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_flee_low_hp_divisor)
        || (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_flee_moderate_hp_divisor)
            && st.borg.tr(BI_CURHP) < st.borg.tr(BI_CLEVEL) * p(Param::caution_flee_hp_per_level)))
        && st.borg.tr(BI_ACCW) < p(Param::caution_flee_ccw_count)
        && st.borg.tr(BI_AHEAL) < p(Param::caution_flee_heal_count)
        // Flee from low hit-points
        && st.borg.tr(BI_CDEPTH) != 0
        && engine::randint0(100) < p(Param::caution_flee_low_hp_chance)
    {
        start_leaving(st, "# Leaving (low hit-points)");
        start_fleeing(st, "# Fleeing (low hit-points)");
    }

    // Flee from bleeding wounds or poison and no heals
    if (st.borg.tr(BI_ISCUT) != 0 || st.borg.tr(BI_ISPOISONED) != 0)
        && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_flee_status_hp_divisor)
        // Flee from bleeding wounds
        && st.borg.tr(BI_CDEPTH) != 0
        && engine::randint0(100) < p(Param::caution_flee_status_chance)
    {
        start_leaving(st, "# Leaving (bleeding/poison)");
        start_fleeing(st, "# Fleeing (bleeding/poison)");
    }

    // Emergency check on healing.  Borg_heal has already been checked but
    // but we did not use our ez_heal potions.  All other attempts to save
    // ourselves have failed.  Use the ez_heal if I have it.
    if (st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_reserve_critical_hp_divisor) // dangerously low HP -OR-
        || (pos_danger > st.borg.tr(BI_CURHP) // extreme danger -AND-
            && (st.borg.tr(BI_ATELEPORT) + st.borg.tr(BI_AESCAPE) <= p(Param::caution_reserve_escape_count)
                && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_reserve_danger_hp_divisor)))
        // low on escapes
        || (st.borg.tr(BI_AEZHEAL) > p(Param::caution_reserve_heal_count) && st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) / p(Param::caution_reserve_surplus_hp_divisor))
        // moderate danger, lots of heals
        || (st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP) >= p(Param::caution_reserve_unique_loss)
            && st.borg.near.unique != 0
            && st.borg.tr(BI_CDEPTH) >= p(Param::caution_reserve_unique_depth)))
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
        && st.borg.tr(BI_CDEPTH) >= 1
        && !stairs_before_recall(st)
        && !ranged_before_recall(st)
        && recall_escape_allowed(st, Some(pos_danger))
        && borg_recall(st)
    {
        // Note
        borg_note(st, "# Fleeing the level (recall)");

        // Success
        return true;
    }

    // If I am waiting for recall,and in danger, buy time with
    // phase and cure_anythings.
    if st.borg.goal.recalling != 0
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

    // Nothing
    false
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
    if st.borg.tr(BI_LIGHT) == 0 && st.borg.tr(BI_CDEPTH) != 0 {
        return false;
    }
    if st.borg.mon.scary {
        // borg_flow_stair_both: light is its only gate on a fleeing Borg
        return true;
    }
    if st.borg.goal.recalling != 0 {
        return false;
    }
    if st.stairs.less.num != 0 {
        let depth = st.borg.tr(BI_CDEPTH) + 1;
        if crate::borg_prepared::borg_prepared_c(st, depth).is_some()
            || (st.borg.tr(BI_CDEPTH) != 0
                && (st.borg.tr(BI_ISWEAK) != 0 || st.borg.tr(BI_ISHUNGRY) != 0 || st.borg.tr(BI_FOOD) < 2))
            || crate::borg_trip::legacy_sale_hold(st)
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
    let track = if up { &st.stairs.less } else { &st.stairs.more };
    let stairs: Vec<(i32, i32)> = (0..track.num.max(0) as usize).map(|i| (track.y[i], track.x[i])).collect();
    if stairs.is_empty() {
        return false;
    }
    crate::borg_flow::borg_flow_clear(st);
    for (y, x) in stairs {
        if st.cave.grid(x, y).kill == 0 {
            crate::borg_flow::borg_flow_enqueue_grid(st, y, x);
        }
    }
    let avoid = up && !(st.borg.tr(BI_CLEVEL) > 35 || st.borg.tr(BI_LIGHT) == 0) && !st.borg.status.desperate;
    crate::borg_flow::borg_flow_spread(st, 250, false, avoid, false, -1, false);
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
pub(crate) fn ranged_candidate(st: &BorgState, kill: &crate::borg_flow_kill::BorgKill) -> bool {
    ranged_target(
        true,
        &[(
            kill.awake,
            engine::distance(st.borg.c, kill.pos),
            st.cave.grid(kill.pos.x, kill.pos.y).info & crate::borg_cave_view::BORG_VIEW != 0,
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
        st.borg.tr(BI_CDEPTH),
        p(Param::caution_flee_recall_gate_below_depth),
        || recall_exception(st) || danger.is_some_and(|d| recall_danger_reaches(d, st.borg.tr(BI_CURHP))),
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
fn recall_allowed_with(c_recall: bool, depth: i32, gate_below: i32, exception: impl FnOnce() -> bool) -> bool {
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
    (breeder_limit > 0 && breeders >= breeder_limit) || (chaser_limit > 0 && chasers >= chaser_limit)
}

pub(crate) fn ranged_before_recall(st: &mut BorgState) -> bool {
    if p(Param::caution_scary_ranged_no_stairs) == 0 {
        return false;
    }
    let means = crate::borg_fight_attack::ranged_means(st);
    let mut known = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        known.push((
            kill.awake,
            engine::distance(st.borg.c, kill.pos),
            st.cave.grid(kill.pos.x, kill.pos.y).info & crate::borg_cave_view::BORG_VIEW != 0,
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
    if fight == 0 || st.borg.mon.breeders == 0 || st.borg.tr(BI_CDEPTH) == 0 {
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
        || st.borg.tr(BI_APHASE) > 0
        || st.borg.tr(BI_ATELEPORT) > 0
        || st.borg.tr(BI_ATELEPORTLVL) > 0
        || st.borg.tr(BI_AESCAPE) > 0
        || (st.borg.tr(BI_RECALL) > 0 && recall_escape_allowed(st, None))
}

/// borg-reborn-rs (#530): whether a scary level is fled (C) or stood.
/// `caution_scary_stand` 1, in the dungeon: a monster that never moves is
/// never scary (it cannot follow), and the Borg flees a scary level only
/// while the escape is real. Real means an easy escape (`easy_escape_with`,
/// the breeder stand's `caution_breeder_easy_escape_steps`), or no scary
/// monster that is awake, can move and is at least as fast as the character
/// (the Borg outpaces every pursuer).
pub(crate) fn scary_stand() -> bool {
    p(Param::caution_scary_stand) != 0
}

/// borg-reborn-rs (#530): applies the decision after a scan of the known
/// monsters. `scary`: a scary monster that can move is known; `pursuer`:
/// one of them is awake and at least as fast as the character. Standing
/// clears the level's scary flag and withdraws the leave and flee the scare
/// set (other cautions set theirs again in this think).
pub(crate) fn apply_scary_stand(st: &mut BorgState, scary: bool, pursuer: bool) {
    // A door scare (an unseen monster opened or burst a door) is scary too,
    // and its monster may be fast: a pursuer (review of #531).
    // borg-reborn-rs (#590, review of #592): a scary monster still known
    // spends any earlier kill, so only the last one's death clears the flag.
    note_scan(&mut st.borg.mon, scary);
    let (scary, pursuer) = stand_inputs(scary, pursuer, st.borg.mon.scary_door);
    if !scary {
        // borg-reborn-rs (#515): without the latch, a level whose scary
        // monsters are gone (dead, forgotten, out of the known monsters)
        // stops being scary. Door scares stay latched (stand_inputs).
        // borg-reborn-rs (#590, owner): with `caution_scary_unlatch_kill`,
        // only after a scary monster was killed, latch on or off: a
        // forgotten monster has not left the level.
        let killed = st.borg.mon.scary_killed;
        if unlatch_when(p(Param::caution_scary_latch) != 0, p(Param::caution_scary_unlatch_kill) != 0, killed)
            && unlatch_scare(&mut st.borg.mon, &mut st.borg.goal)
        {
            borg_note(
                st,
                if killed {
                    "# Scary guy killed: the level is no longer scary."
                } else {
                    "# Scary guy gone: the level is no longer scary."
                },
            );
        }
        return;
    }
    // borg-reborn-rs (#557): while rising, a down staircase the flight may
    // not take is no escape.
    let down = if rising_holds_flee_descent(st) { -1 } else { nearest_free_stair(st, false) };
    let escape = st.borg.tr(BI_CDEPTH) == 0
        || easy_escape_with(st, p(Param::caution_breeder_easy_escape_steps), nearest_free_stair(st, true), down);
    let stand = stand_decision(escape, pursuer);
    if stand {
        st.borg.mon.scary = false;
        withdraw_scary_goals(&mut st.borg.mon, &mut st.borg.goal);
        if !st.borg.mon.scary_stand {
            borg_note(st, "# Scary guy on level, no escape: standing and fighting.");
        }
    } else {
        st.borg.mon.scary = true;
    }
    st.borg.mon.scary_stand = stand;
}

/// borg-reborn-rs (#530, review of #531): record which goals the scary check
/// itself set: those that were false `before` it and are set now. A goal
/// another caution had already set stays that caution's.
pub(crate) fn note_scary_goals(mon: &mut crate::borg::BorgMonInfo, before: (bool, bool, bool), goal: &crate::borg::BorgGoal) {
    mon.scary_leaving |= !before.0 && goal.leaving;
    mon.scary_fleeing |= !before.1 && goal.fleeing;
    mon.scary_town |= !before.2 && goal.fleeing_to_town;
}

/// borg-reborn-rs (#530, review of #531): withdraw only the goals the scare
/// set, and forget that it set them. Another caution that still wants a
/// goal sets it again in this think (the scan runs before `borg_caution`).
pub(crate) fn withdraw_scary_goals(mon: &mut crate::borg::BorgMonInfo, goal: &mut crate::borg::BorgGoal) {
    if std::mem::take(&mut mon.scary_leaving) {
        goal.leaving = false;
    }
    if std::mem::take(&mut mon.scary_fleeing) {
        goal.fleeing = false;
    }
    if std::mem::take(&mut mon.scary_town) {
        goal.fleeing_to_town = false;
    }
}

/// borg-reborn-rs (#515): the scan found nothing scary; clear the flag and
/// withdraw the goals the scare set. True when the level was scary.
/// The stand ends too, even when the last think stood (a stand already
/// cleared the flag), so the next stand prints its note (review of #562).
/// borg-reborn-rs (#590): may the scare clear, now that no scary monster is
/// known? With `kill` (`caution_scary_unlatch_kill`), only after a scary
/// race was killed, whatever the latch; without it, only with the latch off.
fn unlatch_when(latch: bool, kill: bool, killed: bool) -> bool {
    if kill {
        killed
    } else {
        !latch
    }
}

/// borg-reborn-rs (#590): remember a race that made the level scary.
pub(crate) fn note_scary_race(mon: &mut crate::borg::BorgMonInfo, r_idx: u16) {
    if let Some((word, bit)) = race_bit(r_idx) {
        mon.scary_races[word] |= bit;
    }
}

/// borg-reborn-rs (#590): a monster of race `r_idx` died; was it one that
/// made the level scary?
pub(crate) fn note_kill(mon: &mut crate::borg::BorgMonInfo, r_idx: u16) {
    if race_bit(r_idx).is_some_and(|(word, bit)| mon.scary_races[word] & bit != 0) {
        mon.scary_killed = true;
    }
}

/// borg-reborn-rs (#590, review of #592): a scan that finds a scary monster
/// known spends any earlier kill: the flag clears only when the last scary
/// monster known is gone because it was killed.
pub(crate) fn note_scan(mon: &mut crate::borg::BorgMonInfo, scary_known: bool) {
    if scary_known {
        mon.scary_killed = false;
    }
}

/// The word and bit of race `r_idx` in `scary_races` (none for 0 or past
/// the mask).
fn race_bit(r_idx: u16) -> Option<(usize, u64)> {
    let r = usize::from(r_idx);
    (r != 0 && r < 64 * 16).then(|| (r / 64, 1u64 << (r % 64)))
}

fn unlatch_scare(mon: &mut crate::borg::BorgMonInfo, goal: &mut crate::borg::BorgGoal) -> bool {
    mon.scary_stand = false;
    if !std::mem::take(&mut mon.scary) {
        return false;
    }
    mon.scary_killed = false;
    mon.scary_races = [0; 16];
    withdraw_scary_goals(mon, goal);
    true
}

/// borg-reborn-rs (#597): the twitch's random step refuses lava without
/// fire immunity (`caution_twitch_avoid_lava`; lava deals 100 + 1d100 fire
/// damage per step, `player-util.c:918`).
pub(crate) fn twitch_refuses(st: &BorgState, grid: &BorgGrid) -> bool {
    p(Param::caution_twitch_avoid_lava) != 0 && lava_forbids_step(st, grid)
}

/// borg-reborn-rs (#597): with `caution_stair_parked_fallback`, a stair flow
/// into which no staircase entered (`enqueued`), while a monster is marked
/// on known ones (`parked`), takes those staircases anyway.
pub(crate) fn stair_parked_fallback(enqueued: usize, parked: usize) -> bool {
    p(Param::caution_stair_parked_fallback) != 0 && only_parked(enqueued, parked)
}

fn only_parked(enqueued: usize, parked: usize) -> bool {
    enqueued == 0 && parked > 0
}

/// borg-reborn-rs (#530): forget every scary fact: the flag, the door scare,
/// the goals the scare owns and the stand (a new level, the jumpstarts).
pub(crate) fn forget_scary(mon: &mut crate::borg::BorgMonInfo) {
    mon.scary = false;
    mon.scary_door = false;
    mon.scary_leaving = false;
    mon.scary_fleeing = false;
    mon.scary_town = false;
    mon.scary_stand = false;
    mon.scary_killed = false;
    mon.scary_races = [0; 16];
}

/// borg-reborn-rs (#530, review of #531): the decision's inputs. A door
/// scare is scary, and its unseen monster counts as a pursuer.
fn stand_inputs(scan_scary: bool, scan_pursuer: bool, door: bool) -> (bool, bool) {
    (scan_scary || door, scan_pursuer || door)
}

/// borg-reborn-rs (#530): stand when there is no real escape: no easy
/// escape and a pursuer the Borg cannot outpace. A pursuer is a scary (or
/// calmed, #539) monster that is awake and at least as fast as the
/// character: it can catch a fleeing Borg, so with one the Borg stands and
/// fights; with none, walking away is a real escape (review of #569).
fn stand_decision(escape: bool, pursuer: bool) -> bool {
    !escape && pursuer
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
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        // The radius is the engine's distance; the horizon is `borg_danger`'s
        // reach, the larger axis distance (`danger_one_kill`).
        let reach = (kill.pos.x - st.borg.c.x).abs().max((kill.pos.y - st.borg.c.y).abs());
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
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        let breeder = engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY);
        let reach = (kill.pos.x - st.borg.c.x).abs().max((kill.pos.y - st.borg.c.y).abs());
        known.push((engine::distance(st.borg.c, kill.pos), reach, breeder));
        if breeder {
            breeders.push(kill.pos);
        }
    }
    if !only_breeders_near(&known, 1, p(Param::danger_monster_max_distance)) {
        return false;
    }
    if pos_danger < st.borg.tr(BI_CURHP) && !borg_surrounded(st) {
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
            if st.cave.grid(x, y).kill == 0 && breeder_exit_grid(&passable, loc { x, y }, &breeders, distance) {
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
        borg_note(st, &format!("# Leaving the breeders' room ({} exit grids)", exits.len()));
    }
    ok
}

/// Flow to the nearest of `exits` and take its first step. When the flow
/// is committed but no step can be taken, `borg_flow_old` cancels the goal
/// (`goal.type_0 = 0`), so no `GOAL_FLEE` flow is left for the next think.
fn breeder_exit_flow(st: &mut BorgState, exits: &[loc]) -> bool {
    crate::borg_flow::borg_flow_clear(st);
    for e in exits {
        crate::borg_flow::borg_flow_enqueue_grid(st, e.y, e.x);
    }
    crate::borg_flow::borg_flow_spread(st, 250, true, false, false, -1, false);
    crate::borg_flow::borg_flow_commit(st, c"breeder-exit", GOAL_FLEE) && crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
}

/// The 8 neighbours' offsets.
const NEIGHBOURS: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

/// A breeder exit (#544): a passable grid at least `distance` grids from
/// every known breeder that is a chokepoint (`chokepoint`). `passable` is
/// known floor: an unknown grid counts as a wall, and the desperate flow does
/// not cross unknown grids either, so every exit it reaches is on a known path.
fn breeder_exit_grid(passable: &dyn Fn(i32, i32) -> bool, at: loc, breeders: &[loc], distance: i32) -> bool {
    passable(at.x, at.y)
        && chokepoint(passable, at)
        && breeders.iter().all(|&b| engine::distance(b, at) >= distance)
}

/// A chokepoint: the passable neighbours of `at` fall into at least two
/// groups that do not touch each other (8-connected), so the grid is a
/// corridor or doorway between them. A room grid, a room edge or corner, a
/// dead-end tip and an isolated grid have at most one group.
fn chokepoint(passable: &dyn Fn(i32, i32) -> bool, at: loc) -> bool {
    let open: Vec<(i32, i32)> = NEIGHBOURS.iter().copied().filter(|&(dx, dy)| passable(at.x + dx, at.y + dy)).collect();
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

/// borg-reborn-rs (#549): with `caution_stair_scum_threat`, the stair scum
/// (after fleeing town, grabbing DL1 items near the stairs before going back
/// up, `borg_think_stair_scum`) is skipped while the Borg is threatened: the
/// level scary, the danger here above `caution_stair_scum_danger`, or the HP
/// below the maximum. The stair scum runs before `borg_caution` and checks
/// only light, so the Borg picked up items while Grip bit it, one step from
/// the up staircase home (ranger-2107852048).
///
/// When it reports a threat it also cancels a live `GOAL_TAKE` flow: the
/// stair scum's `borg_flow_old` keeps the goal after each step, and the
/// "Continue flowing towards objects" step in `borg_think_dungeon` comes
/// before the flee flows, so a flow committed on an earlier, unthreatened
/// turn would take the Borg one more step toward the item (review of #550).
pub(crate) fn stair_scum_threatened(st: &mut BorgState) -> bool {
    if p(Param::caution_stair_scum_threat) == 0 {
        return false;
    }
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    stair_scum_threatened_with(st, 1, here, p(Param::caution_stair_scum_danger))
}

/// `stair_scum_threatened` with the flag, the danger here and its limit given.
fn stair_scum_threatened_with(st: &mut BorgState, flag: i32, here: i32, danger_limit: i32) -> bool {
    if flag == 0 {
        return false;
    }
    let threat = stair_scum_threat(st.borg.mon.scary, here, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), danger_limit);
    if threat && i32::from(st.borg.goal.type_0) == i32::from(GOAL_TAKE) {
        st.borg.goal.type_0 = 0;
    }
    threat
}

/// The threat test of `stair_scum_threatened` (#549).
fn stair_scum_threat(scary: bool, danger: i32, hp: i32, max_hp: i32, danger_limit: i32) -> bool {
    scary || danger > danger_limit || hp < max_hp
}

/// borg-reborn-rs (#557): with `caution_rising_hold_flee_descent`, while
/// rising in the dungeon a flight takes no down staircase unless the danger
/// here is at or above the HP. With `birth_connect_stairs` the down staircase
/// lands the Borg on DL+1's up staircase, it climbs back to a fresh level on
/// a down staircase again, and the walk home restarts: priest-1055640847
/// bounced DL2-DL3 eight times in 30,000 turns without reaching town.
pub(crate) fn rising_holds_flee_descent(st: &mut BorgState) -> bool {
    if p(Param::caution_rising_hold_flee_descent) == 0 || !st.borg.goal.rising || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    rising_hold(here, st.borg.tr(BI_CURHP))
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

/// borg-reborn-rs (#503): whether the C Borg's hard-coded scary monsters by
/// character level (`borg_near_monster_type`) and the low-level door scares
/// (`borg_update`, `borg_messages`) mark the level scary.
/// borg-reborn-rs (#622, owner): how many character levels sooner a town
/// race (race level 0) stops being scary on the named list.
pub(crate) fn town_scary_levels_less(r_idx: usize) -> i32 {
    if crate::engine::race(r_idx).level == 0 { p(Param::caution_town_scary_levels_less) } else { 0 }
}

pub(crate) fn scary_named_monsters() -> bool {
    p(Param::caution_scary_named_monsters) != 0
}

/// borg-reborn-rs (#604): with `caution_town_never_scary`, no monster makes
/// the town scary: the Borg meets town monsters by its danger rules (fights
/// or avoids them) instead of fleeing to DL1.
pub(crate) fn town_never_scary(depth: i32) -> bool {
    town_never_scary_with(p(Param::caution_town_never_scary), depth)
}

fn town_never_scary_with(switch: i32, depth: i32) -> bool {
    switch != 0 && depth == 0
}

/// borg-reborn-rs (#505): a monster is scary when one of the character's
/// turns of its melee can take at least `caution_scary_damage_percent` of
/// the character's max HP (owner: scary is a monster that can one- or
/// two-shot you, as a percentage of HP). The damage is the monster's own:
/// the average of its blows (`randcalc` AVERAGE: base, dice and level bonus), times its moves per character turn
/// (`extract_energy` of both speeds, at least one). Not `borg_danger`, which
/// adds large penalties for stealing, paralysis or confusion (a street
/// urchin rated 245 against 10 HP). Spells are not counted. Max HP, not
/// current HP: a wounded character is the danger-against-avoidance flight's
/// business. 0 = off. `note`: write the scare's note (#539: not for a
/// calmed monster, which makes nothing scary).
pub(crate) fn scary_by_damage(st: &mut BorgState, i: i32, note: bool) -> bool {
    let percent = p(Param::caution_scary_damage_percent);
    let max_hp = st.borg.tr(BI_MAXHP);
    if percent <= 0 || max_hp <= 0 { return false; }
    let kill = st.kills.kills[i as usize];
    let r_idx = usize::from(kill.r_idx);
    if r_idx == 0 || r_idx >= usize::from(crate::engine::z_info().r_max) - 1 { return false; }
    let damage = melee_damage_per_turn(r_idx, i32::from(kill.speed), st.borg.tr(BI_SPEED));
    let scary = scary_damage_reaches(damage, max_hp, percent);
    if note && scary && !st.borg.mon.scary {
        let name = crate::borg_flow_kill::borg_race_name(r_idx as i32).to_string_lossy();
        borg_note(st, &format!("# Scary by damage: {name}, {damage} of max HP {max_hp}"));
    }
    scary
}

/// borg-reborn-rs (#539): a monster that is hurt or afraid is no named-list
/// or damage-rule scare (owner: Grip fled from one flask of oil). Hurt: it
/// has lost at least `caution_scary_hurt_percent` of its HP (`kill.injury`,
/// from the health bar); afraid: `kill.afraid`, with `caution_scary_afraid`.
/// The resist-based scares and door scares are not affected. A calmed
/// monster that is awake and fast still counts as a pursuer in stand mode:
/// it can follow (else, with other scary monsters asleep, a monster turning
/// afraid made a stand into a flight; two deaths on 18 starts). 0 and 0 = C.
pub(crate) fn scary_calmed(injury: i16, afraid: bool) -> bool {
    calmed(i32::from(injury), afraid, p(Param::caution_scary_hurt_percent), p(Param::caution_scary_afraid) != 0)
}

fn calmed(injury: i32, afraid: bool, hurt_percent: i32, afraid_counts: bool) -> bool {
    (hurt_percent > 0 && injury >= hurt_percent) || (afraid_counts && afraid)
}

/// borg-reborn-rs (#539, review of #569): one monster's scare in the scan
/// of `borg_near_monster_type`. `scary`: it makes the level scary.
/// `calmed_scare`: it is calmed but would have been a named-list or
/// damage-rule scare (it can still pursue).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scare {
    pub(crate) scary: bool,
    pub(crate) calmed_scare: bool,
}

/// borg-reborn-rs (#539, review of #569): the scan's rule, explicit.
/// `named`: the named list says scary; `resist`: a resist-based scare;
/// `calmed`: hurt or afraid (`scary_calmed`); `damage(note)`: the damage
/// rule, writing its note when `note`. A calmed monster is no named-list or
/// damage-rule scare; a resist-based scare always counts. The damage rule
/// is asked only when the monster is not already scary, so with `calmed`
/// false the calls (and notes) are C's: called unless named or resist.
pub(crate) fn classify_scare(named: bool, resist: bool, calmed: bool, damage: impl FnOnce(bool) -> bool) -> Scare {
    let mut scare = Scare { scary: (named && !calmed) || resist, calmed_scare: calmed && named };
    if !scare.scary && damage(!calmed) {
        if calmed {
            scare.calmed_scare = true;
        } else {
            scare.scary = true;
        }
    }
    scare
}

/// borg-reborn-rs (#530, #539): what one monster adds in stand mode:
/// (makes the level scary, is a pursuer). A monster that never moves is
/// neither: it cannot follow. A calmed one is only a pursuer, when it is
/// awake and at least as fast as the character.
pub(crate) fn stand_contribution(scare: Scare, never_move: bool, awake_and_fast: bool) -> (bool, bool) {
    if never_move {
        return (false, false);
    }
    (scare.scary, awake_and_fast && (scare.scary || scare.calmed_scare))
}

/// borg-reborn-rs (#505): the average melee damage of race `r_idx` in one
/// character turn, at these speeds.
fn melee_damage_per_turn(r_idx: usize, monster_speed: i32, player_speed: i32) -> i32 {
    let never_blow = crate::engine::race_flag_has(r_idx, crate::headers::list_mon_race_flags_h::RF_NEVER_BLOW);
    let level = i32::from(crate::engine::race(r_idx).level);
    let blows: Vec<_> = (0..usize::from(crate::engine::z_info().mon_blows_max))
        .map_while(|k| crate::engine::race_blow(r_idx, k))
        .map(|b| (b.base, b.dice, b.sides, b.m_bonus))
        .collect();
    let energy = |speed: i32| i32::from(crate::engine::extract_energy(speed.clamp(0, 199) as usize));
    melee_damage(&blows, level, never_blow, energy(monster_speed), energy(player_speed))
}

/// borg-reborn-rs (#505): `randcalc(dice, level, AVERAGE)` (z-rand.c) in
/// tenths of a point: `base + dice x (sides + 1) / 2 + m_bonus x level /
/// MAX_RAND_DEPTH`, keeping the dice average's half point that the engine's
/// integer `damcalc` drops (1d4 is 2.5, not 2). The engine rolls the same
/// `random_value` for a blow (mon-attack.c `make_attack_normal`).
fn blow_average_tenths(base: i32, dice: i32, sides: i32, m_bonus: i32, level: i32) -> i64 {
    const MAX_RAND_DEPTH: i64 = 128; // z-rand.h
    i64::from(base) * 10 + i64::from(dice) * i64::from(sides + 1) * 5
        + i64::from(m_bonus) * i64::from(level) * 10 / MAX_RAND_DEPTH
}

/// borg-reborn-rs (#505): the sum of the blows' averages (`(base, dice,
/// sides, m_bonus)`), times the monster's moves per character turn (its
/// energy over the character's, at least one move), in whole points;
/// nothing for a race that never blows.
fn melee_damage(blows: &[(i32, i32, i32, i32)], level: i32, never_blow: bool, monster_energy: i32, player_energy: i32) -> i32 {
    if never_blow { return 0; }
    let tenths: i64 = blows.iter().map(|&(b, d, s, m)| blow_average_tenths(b, d, s, m, level)).sum();
    let moves_tenths = (10 * i64::from(monster_energy.max(1)) / i64::from(player_energy.max(1))).max(10);
    i32::try_from(tenths * moves_tenths / 100).unwrap_or(i32::MAX)
}

/// borg-reborn-rs (#505): `damage` is at least `percent` of `max_hp`.
pub(crate) fn scary_damage_reaches(damage: i32, max_hp: i32, percent: i32) -> bool {
    percent > 0 && i64::from(damage) * 100 >= i64::from(max_hp) * i64::from(percent)
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
    breeder_hunt_with(st, p(Param::caution_breeder_fight), p(Param::caution_breeder_hunt_max_count))
}

/// `breeder_hunt` with its parameters given.
fn breeder_hunt_with(st: &BorgState, fight: i32, max_count: i32) -> bool {
    fight != 0
        && st.borg.tr(BI_CDEPTH) != 0
        && st.borg.tr(BI_CLEVEL) < 10
        && st.borg.mon.breeders > 0
        && i32::from(st.borg.mon.breeders) <= max_count
}

/// borg-reborn-rs (#558): the hunt applies and the Borg is engaging the
/// breeders: not ignoring them (`borg_check_breeders`), not fleeing or
/// leaving the level, and no scary monster on it (`borg_flow_kill` then
/// walks to no fight). Read only by #558's own switches, so the hunt of
/// #503 keeps its keys with them off.
pub(crate) fn breeder_engaged(st: &BorgState) -> bool {
    breeder_hunt(st)
        && engaged_with(
            st.borg.goal.ignoring,
            st.borg.goal.fleeing || st.borg.goal.fleeing_to_town || st.borg.goal.leaving,
            st.borg.mon.scary,
        )
}

fn engaged_with(ignoring: bool, fleeing: bool, scary: bool) -> bool {
    !ignoring && !fleeing && !scary
}

/// borg-reborn-rs (#504): Warriors, Paladins, Rogues and Blackguards.
fn melee_class(st: &BorgState) -> bool {
    matches!(st.borg.tr(BI_CLASS), 0 | 5 | 6 | 8)
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
fn fight_cornered_with(st: &BorgState, pos_danger: i32, on: i32, tenths: i32, nearest: i32, radius: i32) -> bool {
    on != 0
        && st.borg.tr(BI_CDEPTH) != 0
        && i64::from(pos_danger) * 10 >= i64::from(st.borg.tr(BI_CURHP)) * i64::from(tenths)
        && nearest != -1
        && nearest <= radius
        && st.borg.tr(BI_APHASE) <= 0
        && st.borg.tr(BI_ATELEPORT) <= 0
        && st.borg.tr(BI_ATELEPORTLVL) <= 0
        && st.borg.tr(BI_AESCAPE) <= 0
}

/// The distance to the nearest known monster that can move, -1 when none is
/// known. A monster that never moves (a mold) does not corner the Borg: one
/// step away leaves it.
fn nearest_kill(st: &BorgState) -> i32 {
    let mut best = -1;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
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
    breeder_fight_hard_with(st, pos_danger, p(Param::caution_breeder_boost_danger_tenths)) && breeder_stand(st)
}

/// The danger half of `breeder_fight_hard`, with its parameter given.
fn breeder_fight_hard_with(st: &BorgState, pos_danger: i32, tenths: i32) -> bool {
    tenths > 0 && pos_danger > 0 && i64::from(pos_danger) * 10 >= i64::from(st.borg.tr(BI_CURHP)) * i64::from(tenths)
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
                || i32::from(kill.level) > st.borg.tr(BI_CLEVEL) + margin)
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
fn boost_order(afraid: bool, fast: bool, hero: bool, berserk: bool, bless: bool, confusing: bool, big: bool) -> Vec<Boost> {
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
fn cornered_boost(st: &mut BorgState, why: &str) -> bool {
    let v = st.item_val;
    let fail = p(Param::caution_cornered_spell_fail);
    let order = boost_order(
        st.borg.tr(BI_ISAFRAID) != 0 && st.borg.tr(BI_CRSFEAR) == 0,
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
                borg_spell_fail(st, BERSERK_STRENGTH, fail) || borg_quaff_potion(st, v.sv_potion_berserk),
                "Berserk Strength",
            ),
            Boost::Blessing => (
                borg_spell_fail(st, BLESS, fail)
                    || borg_read_scroll(st, v.sv_scroll_holy_chant)
                    || borg_read_scroll(st, v.sv_scroll_blessing),
                "Blessing",
            ),
            Boost::MonsterConfusion => (borg_read_scroll(st, v.sv_scroll_monster_confusion), "Monster Confusion"),
        };
        if done {
            borg_note(st, &format!("# {why}: {what}"));
            return true;
        }
    }
    false
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
        && st.borg.tr(BI_CDEPTH) > 0
        && budget > 0
        && borg_timer(st, st.borg.time.level) < budget
        && pos_danger <= i32::from(st.borg.avoidance) * p(Param::caution_rising_descend_danger_tenths) / 10
}

/// Decide which stairs to take when leaving or fleeing (the body of the
/// "Leaving or Fleeing, take stairs" test of `borg_caution`): sets
/// `borg.stair_less` / `borg.stair_more`.
fn choose_stairs(st: &mut BorgState, pos_danger: i32) {
    let force_descend = engine::player_opt(OPT_birth_force_descend);
    if st.borg.ready_morgoth == 0 && st.borg.tr(BI_KING) == 0 && !force_descend {
        st.borg.stair_less = true;
        if st.borg.mon.scary {
            borg_note(st, "# Fleeing and leaving the level. (scaryguy)");
        }
        if pos_danger > i32::from(st.borg.avoidance)
            && st.borg.tr(BI_CLEVEL) <= p(Param::caution_stair_danger_max_level)
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

    if crate::borg_prepared::borg_prepared_c(st, st.borg.tr(BI_CDEPTH) + 1).is_none() {
        st.borg.stair_more = true;
    }

    // don't go down if we can go up and are hungry
    if st.stairs.less.num != 0
        && (st.borg.tr(BI_LIGHT) == 0
            || st.borg.tr(BI_ISHUNGRY) != 0
            || st.borg.tr(BI_ISWEAK) != 0
            || st.borg.tr(BI_FOOD) < p(Param::caution_stair_min_food))
    {
        st.borg.stair_more = false;
    }

    // If I need to sell crap, then don't go down if I can go up
    if st.stairs.less.num != 0
        && st.borg.tr(BI_CDEPTH) != 0
        && st.borg.tr(BI_CLEVEL) < p(Param::caution_sell_max_level)
        && st.borg.tr(BI_GOLD) < p(Param::caution_sell_max_gold)
        && borg_count_sell(st) >= p(Param::caution_sell_min_count)
    {
        st.borg.stair_more = false;
    }

    // Its ok to go one level deep if evading scary guy
    if st.borg.mon.scary {
        st.borg.stair_more = true;
    }

    // if fleeing town, then dive
    if st.borg.tr(BI_CDEPTH) == 0 {
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

/// Buy time while waiting for recall in danger: mana, phase door, cure
/// potions (the last block but one of `borg_caution`).
fn buy_time_for_recall(st: &mut BorgState) -> bool {
    // `CURSP < MAXSP / 4` (before milestone6 patch 0054 C compared the
    // current mana with itself, true only for negative mana).
    if st.borg.tr(BI_ISCONFUSED) == 0
        && st.borg.tr(BI_ISBLIND) == 0
        && st.borg.tr(BI_MAXSP) > p(Param::caution_recall_mana_maxsp)
        && st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP) / p(Param::caution_recall_mana_divisor)
        && (borg_quaff_potion(st, st.item_val.sv_potion_restore_mana)
            || borg_activate_item(st, st.act.restore_mana))
    {
        borg_note(st, "# Buying time waiting for Recall.(1)");
        return true;
    }

    // borg-reborn-rs (#584): through the one Phase Door decision.
    if crate::borg_escape::phase_door(
        st,
        crate::borg_escape::PhaseSite::RecallWait,
        crate::borg_escape::PhaseCheck::Landing {
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
    )
    {
        borg_note(st, "# Buying time waiting for Recall.(2)");
        return true;
    }

    if st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP) < p(Param::caution_recall_cure_loss)
        && (borg_quaff_crit(st, true)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
    {
        borg_note(st, "# Buying time waiting for Recall.(3)");
        return true;
    }

    if st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP) > p(Param::caution_recall_heal_loss)
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
    let no_escape = st.borg.tr(BI_APHASE) <= 0
        && st.borg.tr(BI_ATELEPORT) <= 0
        && st.borg.tr(BI_ATELEPORTLVL) <= 0
        && st.borg.tr(BI_AESCAPE) <= 0;
    let outmatched = pos_danger >= st.borg.tr(BI_CURHP);
    cornered_shape(adjacent, no_escape, outmatched, || borg_surrounded(st))
}

/// `flee_down_cornered`'s test: `adjacent == 1 && no_escape && (outmatched
/// || surrounded)`; `surrounded` (which notes) is asked only when needed.
fn cornered_shape(adjacent: i32, no_escape: bool, outmatched: bool, surrounded: impl FnOnce() -> bool) -> bool {
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
    if rising_holds_flee_descent(st) || crate::borg_flow_stairs::more_refused(st, false) {
        return false;
    }
    // Desperate before the enqueue too: `borg_flow_enqueue_grid` rates an
    // unknown staircase grid's danger unless the flow ignores danger
    // (review of #566; `desperate_flow` set it before the whole flow).
    st.borg.status.desperate = true;
    crate::borg_flow::borg_flow_clear(st);
    crate::borg_flow_stairs::enqueue_down_stairs(st);
    crate::borg_flow::borg_flow_spread(st, flee_down_depth(max_cost), true, false, false, -1, false);
    let ok = crate::borg_flow::borg_flow_commit(st, c"down-stairs", GOAL_FLEE)
        && crate::borg_flow::borg_flow_old(st, GOAL_FLEE);
    st.borg.status.desperate = false;
    if ok {
        borg_note(st, "# Desperate for Stairs (down, no up stairs known)");
    }
    ok
}

/// borg-reborn-rs (#548, K of #544): whether the stuck jumpstart (bravery
/// boost 3, `borg_think_dungeon.rs`) keeps the known staircases.
pub(crate) fn jumpstart_keeps_stairs() -> bool {
    p(Param::caution_jumpstart_keep_stairs) != 0
}

/// The flow depth for `caution_flee_known_down_max_cost`: 0 is any
/// distance (250, `borg_flow_stair_more`'s depth).
fn flee_down_depth(max_cost: i32) -> i32 {
    if max_cost <= 0 { 250 } else { max_cost.min(250) }
}

#[cfg(test)]
mod outmatched_tests {
    use super::*;

    #[test]
    fn outmatched_heal_needs_the_flag() {
        // 29 max HP, 10 HP left, danger 40: the Gnome Druid of #457.
        assert!(!outmatched_heal_applies(0, 10, 29, 40, 50));
        assert!(outmatched_heal_applies(1, 10, 29, 40, 50));
    }

    #[test]
    fn outmatched_heal_needs_a_hurt_borg_in_lethal_danger() {
        // not hurt enough: 15/29 is 48% down, below the 50% threshold
        assert!(!outmatched_heal_applies(1, 15, 29, 40, 50));
        // 14/29 is 51% down
        assert!(outmatched_heal_applies(1, 14, 29, 40, 50));
        // danger below current HP: the ordinary tiers decide
        assert!(!outmatched_heal_applies(1, 10, 29, 9, 50));
        assert!(outmatched_heal_applies(1, 10, 29, 10, 50));
        // unhurt (even with a zero threshold) and a degenerate maximum
        assert!(!outmatched_heal_applies(1, 29, 29, 40, 0));
        assert!(!outmatched_heal_applies(1, 0, 0, 40, 0));
    }

    #[test]
    fn outmatched_heal_is_off_by_default() {
        let spec = |name| crate::params::SPECS.iter().find(|s| s.name == name).unwrap();
        let flag = spec("caution_heal_when_outmatched");
        assert_eq!((flag.default, flag.min, flag.max), (0, 0, 1));
        assert_eq!(spec("caution_heal_outmatched_min_loss").default, 50);
    }
}

#[cfg(test)]
mod scary_stand_tests {
    use super::*;

    fn st() -> BorgState {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; crate::borg_headers::borg_trait_h::BI_MAX as usize];
        st
    }

    #[test]
    fn stand_only_without_a_real_escape() {
        assert!(stand_decision(false, true), "no escape, a pursuer as fast: stand");
        assert!(!stand_decision(true, true), "an easy escape: flee");
        assert!(!stand_decision(false, false), "every scary monster slower or asleep: flee");
    }

    use crate::borg::{BorgGoal, BorgMonInfo};

    fn caution_scare(mon: &mut BorgMonInfo, goal: &mut BorgGoal, town: bool) {
        // borg_caution's scary block: start_leaving, start_fleeing (each
        // only when not set yet), fleeing_to_town in town, then the note.
        let before = (goal.leaving, goal.fleeing, goal.fleeing_to_town);
        goal.leaving = true;
        goal.fleeing = true;
        if town {
            goal.fleeing_to_town = true;
        }
        note_scary_goals(mon, before, goal);
    }

    #[test]
    fn a_the_scare_sets_both_goals_and_standing_withdraws_them() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        caution_scare(&mut mon, &mut goal, false);
        assert!(mon.scary_leaving && mon.scary_fleeing && !mon.scary_town);
        withdraw_scary_goals(&mut mon, &mut goal);
        assert!(!goal.leaving && !goal.fleeing);
        assert!(!mon.scary_leaving && !mon.scary_fleeing, "ownership forgotten");
    }

    #[test]
    fn b_another_cautions_goal_set_first_survives_the_stand() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        goal.leaving = true; // e.g. the restock caution
        caution_scare(&mut mon, &mut goal, false);
        assert!(!mon.scary_leaving && mon.scary_fleeing, "the scare owns only fleeing");
        withdraw_scary_goals(&mut mon, &mut goal);
        assert!(goal.leaving, "the restock's leave survives");
        assert!(!goal.fleeing);
    }

    #[test]
    fn c_a_goal_another_caution_sets_after_the_scare_is_not_the_scares() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        goal.fleeing = true; // fleeing for another reason before the scare
        caution_scare(&mut mon, &mut goal, false);
        assert!(mon.scary_leaving && !mon.scary_fleeing);
        // Later the other caution still wants fleeing; the scare set only leaving.
        withdraw_scary_goals(&mut mon, &mut goal);
        assert!(!goal.leaving, "the scare's leave is withdrawn");
        assert!(goal.fleeing, "the other caution's flee is not");
    }

    #[test]
    fn e_a_scare_that_is_gone_unlatches_only_its_own_goals() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        assert!(!unlatch_scare(&mut mon, &mut goal), "never scary: nothing to clear");
        goal.leaving = true; // another caution's leave, set first
        mon.scary = true;
        caution_scare(&mut mon, &mut goal, true);
        assert!(unlatch_scare(&mut mon, &mut goal), "the scary monster is gone");
        assert!(!mon.scary);
        assert!(goal.leaving, "the other caution's leave survives");
        assert!(!goal.fleeing && !goal.fleeing_to_town, "the scare's goals are withdrawn");
        assert!(!unlatch_scare(&mut mon, &mut goal), "already clear");
    }

    /// Review of #569: off, the rule and the damage calls are C's.
    #[test]
    fn h_uncalmed_scares_and_damage_calls_are_cs() {
        for named in [false, true] {
            for resist in [false, true] {
                for dmg in [false, true] {
                    let mut asked = None;
                    let scare = classify_scare(named, resist, false, |note| {
                        asked = Some(note);
                        dmg
                    });
                    assert_eq!(scare, Scare { scary: named || resist || dmg, calmed_scare: false });
                    let expect = if named || resist { None } else { Some(true) };
                    assert_eq!(asked, expect, "named {named} resist {resist}");
                }
            }
        }
    }

    /// Review of #569: the four combinations, calmed.
    #[test]
    fn i_a_calmed_monster_keeps_only_its_resist_scare() {
        let never = |_: bool| -> bool { panic!("the damage rule is not asked") };
        // Damage rule only: not scary, a calmed scare, and no note.
        let mut note = None;
        let damage_only = classify_scare(false, false, true, |n| {
            note = Some(n);
            true
        });
        assert_eq!(damage_only, Scare { scary: false, calmed_scare: true });
        assert_eq!(note, Some(false), "a calmed monster writes no damage note");
        // Named list only.
        assert_eq!(classify_scare(true, false, true, |_| false), Scare { scary: false, calmed_scare: true });
        // Resist-based only: still scary.
        assert_eq!(classify_scare(false, true, true, never), Scare { scary: true, calmed_scare: false });
        // Named and resist-based: scary through the resist scare.
        assert_eq!(classify_scare(true, true, true, never), Scare { scary: true, calmed_scare: true });
        // Calmed and nothing scary about it: nothing.
        assert_eq!(classify_scare(false, false, true, |_| false), Scare { scary: false, calmed_scare: false });
    }

    /// Review of #569: the same in stand mode, through to the decision.
    #[test]
    fn j_in_stand_mode_a_calmed_monster_is_only_a_pursuer() {
        let calmed = Scare { scary: false, calmed_scare: true };
        let resist = Scare { scary: true, calmed_scare: false };
        assert_eq!(stand_contribution(calmed, false, true), (false, true));
        assert_eq!(stand_contribution(calmed, false, false), (false, false));
        assert_eq!(stand_contribution(resist, false, true), (true, true));
        assert_eq!(stand_contribution(resist, false, false), (true, false));
        assert_eq!(stand_contribution(calmed, true, true), (false, false), "never moves");
        assert_eq!(stand_contribution(resist, true, true), (false, false), "never moves");
        // A calmed monster alone makes no level scary.
        let (scary, pursuer) = stand_contribution(calmed, false, true);
        assert!(!stand_inputs(scary, pursuer, false).0);
        // A sleeping scary monster plus a calmed fast one: the Borg stands
        // (the two deaths of #539's first version fled instead).
        let (s1, p1) = stand_contribution(Scare { scary: true, calmed_scare: false }, false, false);
        let (s2, p2) = stand_contribution(calmed, false, true);
        let (scary, pursuer) = stand_inputs(s1 || s2, p1 || p2, false);
        assert!(scary && stand_decision(false, pursuer));
    }

    #[test]
    fn g_a_hurt_or_afraid_monster_is_calmed_only_when_switched_on() {
        // Off (C): never calmed.
        assert!(!calmed(100, true, 0, false));
        // Hurt: at the threshold and above, not below.
        assert!(!calmed(49, false, 50, false));
        assert!(calmed(50, false, 50, false));
        assert!(calmed(90, false, 50, false));
        // Afraid: only with its switch, whatever the injury.
        assert!(!calmed(0, true, 50, false));
        assert!(calmed(0, true, 50, true));
        assert!(calmed(0, true, 0, true));
        assert!(!calmed(0, false, 0, true));
    }

    #[test]
    fn f_a_stand_whose_scary_monster_is_gone_ends() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        // The last think stood: the flag is clear, the stand is recorded.
        mon.scary_stand = true;
        assert!(!unlatch_scare(&mut mon, &mut goal), "the flag was already clear");
        assert!(!mon.scary_stand, "the next stand prints its note again");
    }

    #[test]
    fn d_forgetting_the_scare_clears_every_fact() {
        let (mut mon, mut goal) = (BorgMonInfo::default(), BorgGoal::default());
        mon.scary = true;
        mon.scary_door = true;
        mon.scary_stand = true;
        caution_scare(&mut mon, &mut goal, true);
        assert!(mon.scary_town);
        forget_scary(&mut mon); // the jumpstarts, a new level
        assert!(!mon.scary && !mon.scary_door && !mon.scary_stand);
        assert!(!mon.scary_leaving && !mon.scary_fleeing && !mon.scary_town);
        // A later stand withdraws nothing the forgotten scare owned.
        withdraw_scary_goals(&mut mon, &mut goal);
        assert!(goal.leaving && goal.fleeing && goal.fleeing_to_town);
    }

    #[test]
    fn a_door_scare_enters_the_decision_as_a_pursuer() {
        // Nothing scary in the scan, but a door scare: scary, and a pursuer.
        assert_eq!(stand_inputs(false, false, true), (true, true));
        let (scary, pursuer) = stand_inputs(false, false, true);
        assert!(scary && stand_decision(false, pursuer), "no escape: stand");
        assert!(!stand_decision(true, pursuer), "an easy escape: flee");
        assert_eq!(stand_inputs(false, false, false), (false, false), "nothing scary: no decision");
        assert_eq!(stand_inputs(true, false, false), (true, false));
    }

    #[test]
    fn a_shallow_flight_recall_needs_the_exception() {
        assert!(!recall_needs_exception(true, 1, 0), "C, no minimum depth");
        assert!(recall_needs_exception(true, 1, 2), "C, DL1 below the minimum");
        assert!(!recall_needs_exception(true, 2, 2), "C, at the minimum");
        assert!(recall_needs_exception(false, 10, 2), "the gate on: any depth");
        assert!(recall_needs_exception(false, 10, 0));
    }

    #[test]
    fn the_shipped_dl1_gate_allows_recall_only_with_the_exception() {
        // Shipped: C's Recall (1) and the gate below depth 2.
        let never = || -> bool { panic!("the exception is not needed") };
        assert!(!recall_allowed_with(true, 1, 2, || false), "DL1, no breeders or chasers: no Recall");
        assert!(recall_allowed_with(true, 1, 2, || true), "DL1 with the exception: Recall");
        assert!(recall_allowed_with(true, 2, 2, never), "DL2: C's Recall, the exception not asked");
        assert!(recall_allowed_with(true, 1, 0, never), "the gate off: C everywhere");
        assert!(!recall_allowed_with(false, 5, 0, || false), "the full gate: any depth");
    }

    #[test]
    fn recall_is_an_escape_only_from_breeders_or_several_chasers() {
        assert!(!recall_escape_with(0, 0, 5, 3), "one scary monster: no Recall");
        assert!(!recall_escape_with(4, 2, 5, 3), "below both limits");
        assert!(recall_escape_with(5, 0, 5, 3), "a level swamped by breeders");
        assert!(recall_escape_with(0, 3, 5, 3), "chased by several monsters");
        assert!(!recall_escape_with(100, 100, 0, 0), "both limits off");
    }

    #[test]
    fn a_flight_recall_from_a_danger_share_of_the_hp() {
        assert!(!danger_reaches_percent(1000, 10, 0), "0 = off");
        assert!(danger_reaches_percent(50, 100, 50), "at the share");
        assert!(!danger_reaches_percent(49, 100, 50));
        assert!(danger_reaches_percent(200, 100, 150), "above the HP");
        assert!(danger_reaches_percent(1, 0, 100), "no HP left");
        assert!(!danger_reaches_percent(i32::MAX, i32::MAX, 101), "no overflow");
    }

    #[test]
    fn a_kill_clears_the_scare_whatever_the_latch() {
        assert!(!unlatch_when(true, false, true), "C: the latch holds even after a kill");
        assert!(unlatch_when(false, false, false), "latch off: whenever none is known");
        assert!(!unlatch_when(false, true, false), "kill rule: forgotten is not enough");
        assert!(unlatch_when(true, true, true), "kill rule: a kill clears with the latch on");
        assert!(unlatch_when(false, true, true));
        let mut mon = BorgMonInfo::default();
        note_scary_race(&mut mon, 42);
        note_kill(&mut mon, 7);
        assert!(!mon.scary_killed, "another race died");
        note_kill(&mut mon, 42);
        assert!(mon.scary_killed, "the scary race died");
        forget_scary(&mut mon);
        assert!(!mon.scary_killed && mon.scary_races == [0; 16], "a new level forgets");
        for r in [1u16, 40, 63, 64, 400, 623] {
            note_scary_race(&mut mon, r);
        }
        for r in [1u16, 40, 63, 64, 400, 623] {
            mon.scary_killed = false;
            note_kill(&mut mon, r);
            assert!(mon.scary_killed, "race {r}: no cap on the races kept");
        }
        note_scary_race(&mut mon, u16::MAX);
        note_kill(&mut mon, 0);
    }

    #[test]
    fn an_old_kill_does_not_clear_a_later_scare() {
        // The review's sequence: a stand, a kill, then a later monster of the
        // same race that is only forgotten.
        let mut mon = BorgMonInfo::default();
        note_scary_race(&mut mon, 42);
        note_scan(&mut mon, true);
        note_kill(&mut mon, 42);
        assert!(mon.scary_killed, "the first one killed");
        note_scan(&mut mon, true); // a later one of race 42 is known: the kill is spent
        assert!(!mon.scary_killed);
        note_scan(&mut mon, false); // then forgotten: nothing scary known
        assert!(!unlatch_when(true, true, mon.scary_killed), "forgotten, not killed: the flag stays");
        note_kill(&mut mon, 42);
        note_scan(&mut mon, false); // killed and none left known
        assert!(unlatch_when(true, true, mon.scary_killed), "the last one killed: the flag clears");
    }

    #[test]
    fn an_easy_escape_is_near_stairs_or_an_escape_item() {
        let mut s = st();
        assert!(!easy_escape_with(&s, 5, -1, -1), "nothing");
        assert!(easy_escape_with(&s, 5, 5, -1), "an up staircase within the steps");
        assert!(easy_escape_with(&s, 5, -1, 3), "a down staircase");
        assert!(!easy_escape_with(&s, 5, 6, 9), "both too far");
        for t in [BI_APHASE, BI_ATELEPORT, BI_ATELEPORTLVL, BI_AESCAPE, BI_RECALL] {
            s.borg.trait_[t as usize] = 1;
            assert!(easy_escape_with(&s, 5, -1, -1), "escape trait {t}");
            s.borg.trait_[t as usize] = 0;
        }
    }
}

#[cfg(test)]
mod scary_damage_tests {
    use super::*;

    #[test]
    fn a_monster_that_can_two_shot_you_is_scary() {
        // 50 percent: two of its turns kill a character at full health.
        assert!(scary_damage_reaches(5, 10, 50), "5 of 10 HP is half");
        assert!(!scary_damage_reaches(4, 10, 50));
        assert!(!scary_damage_reaches(12, 80, 50), "the same monster against 80 HP");
        assert!(scary_damage_reaches(10, 10, 100), "100 = one shot only");
        assert!(!scary_damage_reaches(9, 10, 100));
        assert!(!scary_damage_reaches(1000, 10, 0), "0 = off");
        assert!(scary_damage_reaches(i32::MAX, i32::MAX, 100), "no overflow");
        assert!(!scary_damage_reaches(i32::MAX, i32::MAX, 1000));
    }

    #[test]
    fn a_blow_averages_its_whole_random_value() {
        assert_eq!(blow_average_tenths(0, 1, 4, 0, 10), 25, "1d4 is 2.5");
        assert_eq!(blow_average_tenths(3, 1, 4, 0, 10), 55, "a fixed base adds");
        // m_bonus 64 at level 64 averages 64 x 64 / 128 = 32.
        assert_eq!(blow_average_tenths(0, 0, 0, 64, 64), 320, "the level-scaled bonus adds");
        assert_eq!(blow_average_tenths(5, 2, 6, 64, 32), 50 + 70 + 160);
    }

    #[test]
    fn melee_damage_sums_the_blows_times_the_moves() {
        let bite = (0, 1, 8, 0); // 4.5
        let claw = (0, 1, 3, 0); // 2.0
        assert_eq!(melee_damage(&[bite], 1, false, 10, 10), 4, "one move: 4.5 rounds down");
        assert_eq!(melee_damage(&[bite, claw], 1, false, 10, 10), 6, "several blows add");
        // Twice the character's energy per game turn: two moves per character turn.
        assert_eq!(melee_damage(&[bite], 1, false, 20, 10), 9, "a monster twice as fast doubles");
        assert_eq!(melee_damage(&[bite], 1, false, 5, 10), 4, "a slower monster still gets one move");
        assert_eq!(melee_damage(&[bite, claw], 1, true, 20, 10), 0, "RF_NEVER_BLOW");
        assert_eq!(melee_damage(&[(4, 1, 8, 0)], 1, false, 10, 10), 8, "a base counts");
        assert_eq!(melee_damage(&[], 1, false, 10, 10), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;
    use crate::headers::list_terrain_h::FEAT_FLOOR;

    fn grid_of(feat: u32) -> BorgGrid {
        BorgGrid {
            feat: feat as u8,
            ..BorgGrid::default()
        }
    }

    // borg-reborn-rs#458: lava does 100+1d100 fire damage a turn, so
    // backing away onto it killed full-HP low-level Borgs. (`back_away`
    // itself needs the engine, so the test is on its lava rule.)
    #[test]
    fn lava_is_a_forbidden_step_without_fire_immunity() {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        assert!(lava_forbids_step(&st, &grid_of(FEAT_LAVA)));
        assert!(!lava_forbids_step(&st, &grid_of(FEAT_FLOOR)));
        // an immune Borg may step on it
        st.borg.trait_[BI_IFIRE as usize] = 1;
        assert!(!lava_forbids_step(&st, &grid_of(FEAT_LAVA)));
    }
}

#[cfg(test)]
mod rising_tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;

    /// A Borg rising to town on DL2, 100 turns into the level, 20 avoidance.
    fn rising() -> BorgState {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_CDEPTH as usize] = 2;
        st.borg.goal.rising = true;
        st.borg.avoidance = 20;
        st.borg.time.now = 1100;
        st.borg.time.level = 1000;
        st
    }

    /// #483: while rising, a descent is held in low danger and early on the
    /// level, and released by real danger, the exploration budget, town or
    /// not rising.
    #[test]
    fn rising_holds_a_descent_until_danger_or_budget() {
        assert!(rising_holds_descent(&rising(), 0));
        assert!(rising_holds_descent(&rising(), 20), "danger at avoidance x 10/10 still holds");
        assert!(!rising_holds_descent(&rising(), 21), "real danger lets the descent through");
        let mut st = rising();
        st.borg.time.now = st.borg.time.level + 300;
        assert!(!rising_holds_descent(&st, 0), "the exploration budget is spent");
        let mut st = rising();
        st.borg.goal.rising = false;
        assert!(!rising_holds_descent(&st, 0), "not rising");
        let mut st = rising();
        st.borg.trait_[BI_CDEPTH as usize] = 0;
        assert!(!rising_holds_descent(&st, 0), "in town");
    }
}

#[cfg(test)]
mod town_scary_tests {
    use super::*;

    /// #604: only in town, and only with the switch on.
    #[test]
    fn town_is_never_scary_with_the_switch() {
        assert!(town_never_scary_with(1, 0));
        assert!(!town_never_scary_with(0, 0), "switch off");
        assert!(!town_never_scary_with(1, 1), "dungeon");
    }
}

#[cfg(test)]
mod breeder_tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;

    /// A CL3 Rogue on DL1 with 3 breeders known, no stairs known and
    /// nothing to escape with.
    fn cornered() -> BorgState {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_CDEPTH as usize] = 1;
        st.borg.trait_[BI_CLEVEL as usize] = 3;
        st.borg.trait_[BI_CLASS as usize] = 6;
        st.borg.mon.breeders = 3;
        st
    }

    /// #503: breeders and no easy escape: stand; an escape in the pack
    /// (Phase Door, teleport, escape, recall), no breeders, town or the
    /// switch off: the C Borg's flees stay.
    #[test]
    fn breeders_without_an_easy_escape_make_the_borg_stand() {
        assert!(breeder_stand_with(&cornered(), 1, 5, -1, -1));
        assert!(!breeder_stand_with(&cornered(), 0, 5, -1, -1), "switch off");
        for t in [BI_APHASE, BI_ATELEPORT, BI_ATELEPORTLVL, BI_AESCAPE, BI_RECALL] {
            let mut st = cornered();
            st.borg.trait_[t as usize] = 1;
            assert!(!breeder_stand_with(&st, 1, 5, -1, -1), "escape {t} in the pack");
        }
        let mut st = cornered();
        st.borg.mon.breeders = 0;
        assert!(!breeder_stand_with(&st, 1, 5, -1, -1), "no breeders");
        let mut st = cornered();
        st.borg.trait_[BI_CDEPTH as usize] = 0;
        assert!(!breeder_stand_with(&st, 1, 5, -1, -1), "town");
    }

    /// #503 review: a free staircase within the steps is an easy escape,
    /// up or down; one step beyond is not.
    #[test]
    fn a_near_staircase_is_an_easy_escape() {
        let st = cornered();
        assert!(!breeder_stand_with(&st, 1, 5, 5, -1), "up staircase at the boundary");
        assert!(!breeder_stand_with(&st, 1, 5, -1, 5), "down staircase at the boundary");
        assert!(!breeder_stand_with(&st, 1, 5, 0, -1), "standing on a staircase");
        assert!(breeder_stand_with(&st, 1, 5, 6, 6), "both one step beyond");
        assert!(breeder_stand_with(&st, 1, 0, 1, 1), "steps 0: only a staircase underfoot counts");
    }

    /// #503 review: no retreat only while the danger is the breeders': a
    /// breeder within the radius and no other monster within the horizon.
    #[test]
    fn standing_needs_only_breeders_within_the_danger_horizon() {
        assert!(only_breeders_near(&[(1, 1, true)], 5, 20));
        assert!(only_breeders_near(&[(1, 1, true), (5, 4, true)], 5, 20));
        assert!(!only_breeders_near(&[], 5, 20), "no monster");
        assert!(!only_breeders_near(&[(6, 5, true)], 5, 20), "breeder beyond the radius");
        assert!(!only_breeders_near(&[(1, 1, true), (12, 10, false)], 5, 20), "a caster within the horizon");
        assert!(only_breeders_near(&[(1, 1, true), (21, 21, false)], 5, 20), "a monster beyond the horizon");
        // #510 re-review: a breather at (+14,+14) is engine distance 21 but
        // within borg_danger's reach (14): it vetoes.
        assert!(!only_breeders_near(&[(1, 1, true), (21, 14, false)], 5, 20), "a diagonal monster within reach");
        assert!(!only_breeders_near(&[(1, 1, false), (2, 2, true)], 5, 20), "a non-breeder adjacent");
    }

    /// The Borg's map of priest-1055640847's worm room (its last `Borg.map`,
    /// #544): `#` and `%` walls, ` ` unknown, everything else known floor,
    /// `w` a worm. The only way out is the corridor north to `>`.
    const WORM_ROOM: [&str; 28] = [
        "#                 #.#",
        "#                 #>#",
        "#                 #.#",
        "#                 #.#",
        "#                 #.#####",
        "#                 #.....#",
        "#                 #####.#",
        "#                     #.#",
        "#                     #.#######",
        "#                     #.......#",
        "#                     #######.##",
        "#                          %%..%",
        "#                          %.%.%",
        "#                          %%#.#",
        "#                            #.%",
        "#                            #.#",
        "#                            #.#",
        "#                            #.#",
        "#                            #&#",
        "#                            #.#",
        "#                            #.#",
        "#                      #######.#############",
        "#                      #.ww..w.ww.w.w..w.w.#",
        "#                      #..w.w.wwwww.wwww..w#",
        "#                      #....w.w.wwww.ww..w.#",
        "#                      #....w..&ww@wwww.w..#",
        "#                      #ww.w.www.wwwwwww...#",
        "#                      #.w.......w&w.w.....#",
    ];

    fn map_passable(map: &'static [&'static str]) -> impl Fn(i32, i32) -> bool {
        move |x, y| {
            y >= 0
                && (y as usize) < map.len()
                && x >= 0
                && map[y as usize].as_bytes().get(x as usize).is_some_and(|&c| !matches!(c, b'#' | b'%' | b' '))
        }
    }

    #[test]
    fn breeder_exits_of_the_worm_room_are_on_its_corridor() {
        let passable = map_passable(&WORM_ROOM);
        let mut worms = Vec::new();
        for (y, row) in WORM_ROOM.iter().enumerate() {
            for (x, c) in row.bytes().enumerate() {
                if c == b'w' {
                    worms.push(loc { x: x as i32, y: y as i32 });
                }
            }
        }
        let room = |at: loc| at.y >= 22;
        let mut exits = Vec::new();
        for y in 0..WORM_ROOM.len() as i32 {
            for x in 0..50 {
                if breeder_exit_grid(&passable, loc { x, y }, &worms, 6) {
                    exits.push(loc { x, y });
                }
            }
        }
        assert!(!exits.is_empty(), "the corridor has exits");
        for e in &exits {
            assert!(!room(*e), "no exit inside the room: {},{}", e.x, e.y);
            assert!(worms.iter().all(|w| engine::distance(*w, *e) >= 6), "beyond the radius: {},{}", e.x, e.y);
        }
        // The corridor's dead-end tip (its unknown end, above `>`) and the
        // room's corners and edges are not exits.
        for at in [loc { x: 19, y: 0 }, loc { x: 24, y: 22 }, loc { x: 43, y: 22 }, loc { x: 24, y: 27 }, loc { x: 30, y: 22 }] {
            assert!(!breeder_exit_grid(&passable, at, &[], 6), "{},{}", at.x, at.y);
        }
        // The doorway and the corridor are chokepoints.
        assert!(chokepoint(&passable, loc { x: 30, y: 21 }), "the doorway");
        assert!(chokepoint(&passable, loc { x: 30, y: 16 }), "the corridor");
        assert!(breeder_exit_grid(&passable, loc { x: 30, y: 16 }, &worms, 6), "the corridor, 6 grids up");
        assert!(exits.contains(&loc { x: 30, y: 16 }), "found by the scan");
    }

    #[test]
    fn stair_scum_stops_under_any_threat() {
        assert!(!stair_scum_threat(false, 0, 14, 14, 0), "nothing near, full HP");
        assert!(stair_scum_threat(true, 0, 14, 14, 0), "the level is scary");
        assert!(stair_scum_threat(false, 4, 14, 14, 0), "Grip in view (ranger-2107852048, turn 191)");
        assert!(!stair_scum_threat(false, 4, 14, 14, 4), "danger at the limit");
        assert!(stair_scum_threat(false, 0, 6, 14, 0), "hurt");
    }

    #[test]
    fn rising_holds_a_flight_down_unless_in_real_danger() {
        assert!(rising_hold(0, 70), "nothing near");
        assert!(rising_hold(69, 70), "danger below the HP");
        assert!(!rising_hold(70, 70), "danger at the HP");
        assert!(!rising_hold(2000, 70), "a worm crowd");
    }

    /// Review of #550: the skip also ends the stair scum's own flow, so no
    /// `GOAL_TAKE` step follows it; other goals and the unthreatened case
    /// keep theirs.
    #[test]
    fn a_threat_cancels_the_stair_scum_flow() {
        let scum = || {
            let mut st = cornered();
            st.borg.trait_[BI_CURHP as usize] = 14;
            st.borg.trait_[BI_MAXHP as usize] = 14;
            st.borg.goal.type_0 = GOAL_TAKE as _;
            st
        };
        let mut st = scum();
        st.borg.mon.scary = true;
        assert!(stair_scum_threatened_with(&mut st, 1, 0, 0));
        assert_eq!(st.borg.goal.type_0, 0, "the scum flow is cancelled");

        let mut st = scum();
        assert!(stair_scum_threatened_with(&mut st, 1, 4, 0), "danger here");
        assert_eq!(st.borg.goal.type_0, 0);

        let mut st = scum();
        assert!(!stair_scum_threatened_with(&mut st, 1, 0, 0), "no threat");
        assert_eq!(i32::from(st.borg.goal.type_0), i32::from(GOAL_TAKE), "no threat: the scum goes on");

        let mut st = scum();
        st.borg.mon.scary = true;
        assert!(!stair_scum_threatened_with(&mut st, 0, 0, 0), "flag off");
        assert_eq!(i32::from(st.borg.goal.type_0), i32::from(GOAL_TAKE), "flag off: untouched");

        let mut st = scum();
        st.borg.mon.scary = true;
        st.borg.goal.type_0 = GOAL_FLEE as _;
        assert!(stair_scum_threatened_with(&mut st, 1, 0, 0));
        assert_eq!(i32::from(st.borg.goal.type_0), i32::from(GOAL_FLEE), "a flee flow is kept");
    }

    #[test]
    fn chokepoints_are_corridors_not_rooms() {
        const LIT_ROOM: [&str; 5] = ["#####", "#...#", "#...#", "#...#", "#####"];
        let passable = map_passable(&LIT_ROOM);
        for at in [loc { x: 1, y: 1 }, loc { x: 2, y: 1 }, loc { x: 2, y: 2 }] {
            assert!(!chokepoint(&passable, at), "corner, edge, middle: {},{}", at.x, at.y);
        }
        const CORRIDOR: [&str; 3] = ["####", "....", "####"];
        assert!(chokepoint(&map_passable(&CORRIDOR), loc { x: 1, y: 1 }), "a corridor");
        // A bend's corner is not one: its two neighbours touch diagonally,
        // and the flow steps across it.
        const BEND: [&str; 3] = ["#.##", "#..#", "####"];
        assert!(!chokepoint(&map_passable(&BEND), loc { x: 1, y: 1 }), "a bend's corner");
        const TIP: [&str; 3] = ["####", "#..#", "####"];
        assert!(!chokepoint(&map_passable(&TIP), loc { x: 1, y: 1 }), "a dead-end tip");
        const ALONE: [&str; 3] = ["###", "#.#", "###"];
        assert!(!chokepoint(&map_passable(&ALONE), loc { x: 1, y: 1 }), "an isolated grid");
    }

    /// #503, #558: every class hunts while few breeders are known; the
    /// count (a class value) decides, and 0 or the switch off stops it.
    #[test]
    fn every_class_hunts_few_breeders() {
        assert!(breeder_hunt_with(&cornered(), 1, 4));
        assert!(!breeder_hunt_with(&cornered(), 0, 4), "switch off");
        assert!(!breeder_hunt_with(&cornered(), 1, 2), "too many breeders");
        assert!(!breeder_hunt_with(&cornered(), 1, 0), "count 0: no hunt");
        let mut high = cornered();
        high.borg.trait_[BI_CLEVEL as usize] = 10;
        assert!(!breeder_hunt_with(&high, 1, 4), "CL10: no hunt");
        for class in 0..9 {
            let mut st = cornered();
            st.borg.trait_[BI_CLASS as usize] = class;
            assert!(breeder_hunt_with(&st, 1, 4), "class {class}");
        }
    }

    /// #558 review: no engagement while ignoring, fleeing or scared.
    #[test]
    fn breeders_are_engaged_only_while_staying() {
        assert!(engaged_with(false, false, false));
        assert!(!engaged_with(true, false, false), "ignoring breeders");
        assert!(!engaged_with(false, true, false), "fleeing or leaving");
        assert!(!engaged_with(false, false, true), "scary monster");
    }
}

#[cfg(test)]
mod cornered_tests {
    use super::*;
    use crate::borg_headers::borg_trait_h::BI_MAX;

    /// A Blackguard on DL1 at 5 HP, nothing to escape with.
    fn trapped() -> BorgState {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_CDEPTH as usize] = 1;
        st.borg.trait_[BI_CURHP as usize] = 5;
        st.borg.trait_[BI_CLASS as usize] = 8;
        st
    }

    /// #504: cornered with danger at or above the HP, a monster within the
    /// radius and no Phase Door, teleport or escape; not in town, not with
    /// an escape, not with the monster far or the danger low, not off.
    #[test]
    fn cornered_needs_danger_a_fight_and_no_escape() {
        let st = trapped();
        assert!(fight_cornered_with(&st, 5, 1, 10, 1, 2), "danger = HP, monster adjacent");
        assert!(!fight_cornered_with(&st, 4, 1, 10, 1, 2), "danger below the HP");
        assert!(fight_cornered_with(&st, 3, 1, 5, 1, 2), "half the HP at 5 tenths");
        assert!(!fight_cornered_with(&st, 5, 0, 10, 1, 2), "switch off");
        assert!(!fight_cornered_with(&st, 5, 1, 10, -1, 2), "no monster known");
        assert!(!fight_cornered_with(&st, 5, 1, 10, 3, 2), "monster beyond the radius");
        for t in [BI_APHASE, BI_ATELEPORT, BI_ATELEPORTLVL, BI_AESCAPE] {
            let mut st = trapped();
            st.borg.trait_[t as usize] = 1;
            assert!(!fight_cornered_with(&st, 5, 1, 10, 1, 2), "escape {t}");
        }
        let mut st = trapped();
        st.borg.trait_[BI_CDEPTH as usize] = 0;
        assert!(!fight_cornered_with(&st, 5, 1, 10, 1, 2), "town");
    }

    /// #504 (owner): a hard breeder fight boosts from danger at tenths x HP
    /// / 10; 0 = off; no danger, no boost.
    #[test]
    fn a_hard_breeder_fight_boosts() {
        let st = trapped();
        assert!(breeder_fight_hard_with(&st, 3, 5), "danger 3 >= half of 5 HP");
        assert!(!breeder_fight_hard_with(&st, 2, 5), "danger below half the HP");
        assert!(!breeder_fight_hard_with(&st, 9, 0), "off");
        assert!(!breeder_fight_hard_with(&st, 0, 5), "no danger");
    }

    /// #504 review: the boost order, each only while not active.
    #[test]
    fn boosts_come_in_order_and_skip_what_is_active() {
        use Boost::*;
        assert_eq!(
            boost_order(true, false, false, false, false, false, true),
            [FearCure, Speed, Heroism, Berserk, Blessing, MonsterConfusion]
        );
        assert_eq!(boost_order(false, false, false, false, false, false, false), [Speed, Heroism, Berserk, Blessing]);
        assert_eq!(boost_order(false, true, true, false, true, false, false), [Berserk]);
        assert_eq!(boost_order(false, true, true, true, true, true, true), [], "all active, hands already confusing");
    }

    #[test]
    fn melee_classes_fight_all_out() {
        for (class, melee) in [(0, true), (1, false), (2, false), (3, false), (4, false), (5, true), (6, true), (7, false), (8, true)] {
            let mut st = trapped();
            st.borg.trait_[BI_CLASS as usize] = class;
            assert_eq!(melee_class(&st), melee, "class {class}");
        }
    }
}

#[cfg(test)]
mod recall_tests {
    use super::*;

    /// #539: with no stairs, Recall waits for a ranged attack on a monster
    /// in view at range.
    #[test]
    fn a_ranged_target_comes_before_recall_without_stairs() {
        assert!(ranged_target(true, &[(true, 4, true)]), "Grip four grids up the corridor");
        assert!(!ranged_target(false, &[(true, 4, true)]), "nothing to throw or shoot");
        assert!(!ranged_target(true, &[(true, 1, true)]), "adjacent: melee's business");
        assert!(!ranged_target(true, &[(true, 4, false)]), "out of view");
        assert!(!ranged_target(true, &[(false, 4, true)]), "asleep");
        assert!(!ranged_target(true, &[]), "no monster");
    }

    /// #539 review: `borg_attack` on a scary level without stairs takes a
    /// monster only if `ranged_candidate` holds for it alone, so it targets
    /// exactly the monsters Recall and Phase Door wait for, not any monster
    /// the accumulated `adjacent_monster` happened to let through.
    #[test]
    fn the_attack_targets_the_monster_recall_waits_for() {
        let grip = (true, 4, true);
        let sleeper = (false, 5, true);
        let behind_wall = (true, 3, false);
        let next_to_me = (true, 1, true);
        let level = [sleeper, behind_wall, next_to_me, grip];
        let targets: Vec<_> = level.iter().filter(|&&m| ranged_target(true, &[m])).collect();
        assert_eq!(targets, vec![&grip], "only Grip, awake in view at range");
        assert!(ranged_target(true, &level), "so Recall waits");
        assert!(!ranged_target(true, &[sleeper, behind_wall, next_to_me]), "no candidate: Recall may go");
    }
}

#[cfg(test)]
mod flee_known_down_tests {
    use super::*;

    #[test]
    fn parked_stairs_are_taken_only_when_no_staircase_entered_the_flow() {
        assert!(only_parked(0, 1), "every known staircase parked");
        assert!(!only_parked(1, 1), "a free staircase entered the flow");
        assert!(only_parked(0, 2), "the free one was refused as icky: it does not count");
        assert!(!only_parked(0, 0), "none known");
    }

    /// #548: cornered means an adjacent moving monster, no escape item, and
    /// the danger here at or above the HP or surrounded.
    #[test]
    fn cornered_needs_an_adjacent_monster_no_escape_and_the_danger_or_a_surround() {
        assert!(cornered_shape(1, true, true, || false));
        assert!(cornered_shape(1, true, false, || true), "surrounded");
        assert!(!cornered_shape(1, true, false, || false), "neither");
        assert!(!cornered_shape(2, true, true, || true), "nothing adjacent");
        assert!(!cornered_shape(-1, true, true, || true), "no monster known");
        assert!(!cornered_shape(1, false, true, || true), "an escape in the pack");
        assert!(cornered_shape(1, true, true, || panic!("not asked when outmatched")));
    }

    /// #548: 0 keeps #545's any distance (the flow's 250); a cap is the
    /// flow's depth, never above 250.
    #[test]
    fn the_distance_cap_is_the_flow_depth() {
        assert_eq!(flee_down_depth(0), 250);
        assert_eq!(flee_down_depth(-3), 250);
        assert_eq!(flee_down_depth(40), 40);
        assert_eq!(flee_down_depth(1000), 250);
    }
}

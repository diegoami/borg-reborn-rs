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
    borg_escape, borg_freedom, borg_surrounded,
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
    BI_ISWEAK, BI_KING, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP, BI_RECALL, BI_RPOIS, BI_SPEED,
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
const GOAL_BORE: i32 = 6;
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
    // borg-reborn-rs (#842): no cure in contact with a fast-fragile monster.
    if crate::borg_fast_fragile::heal_banned(st) {
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
    // borg-reborn-rs (#646): no cure next to an immobile confuser, which
    // confuses again at once (borg-cemetery#75), unless the monsters that
    // move are as dangerous as the HP left.
    // borg-reborn-rs (#867): or next to an immobile confuser, the attack
    // first. One note, naming the reason (review of #868).
    let hold = confusion_cure_hold(st);
    match hold {
        Some(CureHold::Attack) => borg_note(st, "# Not fixing Confusion next to an immobile confuser: attacking first (#867)"),
        Some(CureHold::Step) => borg_note(st, "# Not fixing Confusion next to an immobile attacker (#646)"),
        None => {}
    }
    let hold_cure = hold.is_some();
    if st.borg.tr(BI_ISCONFUSED) != 0 && !hold_cure {
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
                || (confusion_clw_cure() && borg_quaff_potion(st, st.item_val.sv_potion_cure_light))
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

    // borg-reborn-rs (#842): no cure in contact with a fast-fragile monster.
    if crate::borg_fast_fragile::heal_banned(st) {
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

/// borg-reborn-rs (#743): whether every flight to the stairs needs a clean
/// path (`caution_stairs_clean_path_flee`, read here as the registry asks
/// of a caution parameter; the check is `borg_flow_stairs::commit_and_step`).
pub(crate) fn stairs_clean_path_flee_on() -> bool {
    p(Param::caution_stairs_clean_path_flee) != 0
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
fn desperate_flow(st: &mut BorgState, up: bool, max_moves: i32, why: &str) -> bool {
    let weight = stair_flight_weight(GOAL_FLEE, false);
    // borg-reborn-rs (#736, owner): stairs whose path a dangerous monster is
    // on, or next to, are not reachable: the escapes and the attack decide
    // instead (`caution_stairs_clean_path`; 0 = C).
    if p(Param::caution_stairs_clean_path) != 0 && weight == 0 {
        let percent = p(Param::caution_stairs_block_danger_percent);
        if let Some(k) = crate::borg_flow_stairs::stair_path_blocker(st, up, percent) {
            let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
            borg_note(st, &format!("# Stairs not reachable: {} in the way ({})", String::from_utf8_lossy(&name), why.trim_start_matches("# ")));
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
    let ranged_reach = if reach > 0 && !awake_monster_adjacent(st) && crate::borg_fight_attack::ranged_means(st) {
        reach
    } else {
        0
    };
    if weight == 0 && ranged_reach > 0 {
        let percent = p(Param::caution_stairs_block_danger_percent);
        if let Some(k) = crate::borg_flow_stairs::stair_path_ahead(st, up, percent, reach) {
            let name = engine::race_name(usize::from(st.kills.kills[k].r_idx));
            borg_note(st, &format!("# Stairs not reachable: {} ahead, fighting at range ({})", String::from_utf8_lossy(&name), why.trim_start_matches("# ")));
            return false;
        }
    }
    st.borg.status.desperate = true;
    let ok = if weight > 0 {
        // A stair that is close across the map may be far away on foot.
        // Keep the proximity limit on the route, including any detour.
        crate::borg_flow_stairs::weighted_desperate_flight(st, up, max_moves, ranged_reach)
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
    // Not the #867 hold (review of #868): it holds the cure for an attack,
    // which a confused walk would replace.
    // At 2, no confused walk at all, cure or not.
    let walk_mode = p(Param::caution_confused_cure_before_walk);
    if walk_mode != 0
        && st.borg.tr(BI_ISCONFUSED) != 0
        && (walk_mode >= 2 || (confusion_cure_carried(st) && !cure_held_for_step(st)))
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
        && pos_danger < st.borg.tr(BI_CURHP)
        && desperate_flow(st, true, near_up, "# Desperate for Stairs (one)")
    {
        return true;
    }

    // If you are next to steps of the stairs go for it
    let adjacent_up = p(Param::caution_desperate_adjacent_up_distance);
    if b_j <= adjacent_up
        && b_j != -1
        && desperate_flow(st, true, adjacent_up, "# Desperate for Stairs (two)")
    {
        return true;
    }

    // Low level guys tend to waste money reading the recall scrolls
    let scary_up = p(Param::caution_desperate_scary_distance) - 1;
    if b_j <= scary_up
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
            // borg-reborn-rs (#747): with caution_desperate_fast_danger_percent,
            // only a faster monster that is also dangerous (a fruit bat
            // next to the Borg is not a reason to stay).
            if i32::from(st.kills.kills[usize::from(kill)].speed) > st.borg.tr(BI_SPEED)
                && fast_neighbour_counts(st, usize::from(kill))
            {
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
        } else if desperate_flow(st, true, scary_up, "# Desperate for Stairs (three)") {
            return true;
        }
    }

    // #871: keep a viable up-stair flight before trying an adjacent down stair.
    let adjacent_down = p(Param::caution_desperate_adjacent_down_distance);
    if b_m <= adjacent_down
        && b_m != -1
        && (crate::borg_flow_stairs::resume_known_up_stair_flight(st, pos_danger) || desperate_flow(st, false, adjacent_down, "# Desperate for Stairs (four)"))
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

/// borg-reborn-rs (#646, owner): a known monster that never moves and
/// attacks with its blows (a mushroom patch, a mold, a floating eye;
/// `monster.txt` `NEVER_MOVE` with a blow and without `NEVER_BLOW`).
fn immobile_attacker(r_idx: usize) -> bool {
    use crate::headers::list_mon_race_flags_h::{RF_NEVER_BLOW, RF_NEVER_MOVE};
    // borg-reborn-rs (#726): not the placeholder race past the last one
    // (no blow array), as the rest of the core skips it (`r_max - 1`).
    r_idx != 0
        && engine::r_max() != 0
        && r_idx < usize::from(engine::r_max()) - 1
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
        && st.borg.tr(BI_CDEPTH) > 0
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
    if p(Param::caution_ranged_only) == 0 || r_idx == 0 || r_idx >= engine::r_max().saturating_sub(1) || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let Some(list) = crate::scary_list::selected() else { return false };
    let who = crate::scary_list::Situation {
        clevel: st.borg.tr(BI_CLEVEL),
        class: st.borg.tr(BI_CLASS),
        breeder_count: 0,
        breeder_stand: false,
        levels_less: 0,
    };
    crate::scary_list::ranged_reason(list, r_idx, || engine::race_base_name(r_idx), who).is_some()
}

/// borg-reborn-rs (#713, #838): race `r_idx` is kept at range: an immobile
/// attacker (`caution_ranged_vs_immobile` 2, 3) or a `ranged` entry of the
/// scary list (`caution_ranged_only`): never a melee target, no walk to it,
/// flows keep out of its reach.
pub(crate) fn kept_at_range(st: &BorgState, r_idx: usize) -> bool {
    immobile_kept_at_range(st, r_idx) || listed_ranged(st, r_idx)
}

/// borg-reborn-rs (#713): grid (`x`, `y`) is, or is next to, the grid of a
/// creature kept at range (its reach). Off: no scan.
pub(crate) fn in_reach_of_kept(st: &BorgState, x: i32, y: i32) -> bool {
    if (p(Param::caution_ranged_vs_immobile) < 2 && p(Param::caution_ranged_only) == 0) || st.flow.to_stairs {
        return false;
    }
    (-1..=1).any(|dy| {
        (-1..=1).any(|dx| {
            let (gx, gy) = (x + dx, y + dy);
            engine::square_in_bounds_fully(gx, gy) && {
                let k = st.cave.grid(gx, gy).kill;
                k != 0 && kept_at_range(st, usize::from(st.kills.kills[usize::from(k)].r_idx))
            }
        })
    })
}

/// borg-reborn-rs (#837, borg-zoo#576): `caution_immobile_no_return`. Grid
/// (`x`, `y`) is, or is next to, the grid of an immobile attacker the Borg has
/// stepped away from on this level and that is still known there. The step
/// away (#646) was undone by the next flow in the two grey mold deaths of
/// #576 (`Flowing toward dark-3` led back beside the mold in the dark), and
/// the monster is not "kept at range" (a grey mold's blows are plain damage),
/// so `in_reach_of_kept` did not see it. 1: only while the Borg is below its
/// full HP (it rests, then the usual rules decide again); 2: for the level.
/// Off: no scan. A flight to the stairs is not stopped, as for the kept ones.
pub(crate) fn in_reach_of_stepped_from(st: &BorgState, x: i32, y: i32) -> bool {
    let mode = p(Param::caution_immobile_no_return);
    if mode == 0 || st.flow.to_stairs || st.borg.mon.immobile_stepped_level != st.borg.time.level {
        return false;
    }
    if mode == 1 && st.borg.tr(BI_CURHP) >= st.borg.tr(BI_MAXHP) {
        return false;
    }
    st.borg.mon.immobile_stepped.iter().any(|e| {
        let (ey, ex) = (i32::from(e[0]), i32::from(e[1]));
        (e[2] > 0 || e[3] > 0)
            && (ey - y).abs() <= 1
            && (ex - x).abs() <= 1
            && engine::square_in_bounds_fully(ex, ey)
            && {
                let k = st.cave.grid(ex, ey).kill;
                k != 0 && immobile_attacker(usize::from(st.kills.kills[usize::from(k)].r_idx))
            }
    })
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
            && (i32::from(kill.pos.y) - y).abs() <= 1
            && (i32::from(kill.pos.x) - x).abs() <= 1
    })
}

/// The first known immobile attacker next to grid (`y`, `x`), by kill index.
fn immobile_attacker_next_to(st: &BorgState, y: i32, x: i32) -> Option<usize> {
    (1..usize::try_from(st.kills.nxt).unwrap_or(0)).find(|&i| {
        let kill = &st.kills.kills[i];
        immobile_attacker(usize::from(kill.r_idx))
            && (i32::from(kill.pos.y) - y).abs() <= 1
            && (i32::from(kill.pos.x) - x).abs() <= 1
    })
}

/// borg-reborn-rs (#867): the first known immobile attacker next to grid
/// (`y`, `x`) with a blow that confuses (`CONFUSE`, blow_effects.txt; the
/// grey mushroom patch's `SPORE:CONFUSE`), by kill index. Not a unique
/// (review of #868: Wiruin, the Maelstrom, `ENGULF:CONFUSE:3d6`, is no 2-HP
/// patch to swing at confused, and `mobile_danger` leaves its own blows out).
fn immobile_confuser_next_to(st: &BorgState, y: i32, x: i32) -> Option<usize> {
    (1..usize::try_from(st.kills.nxt).unwrap_or(0)).find(|&i| {
        let kill = &st.kills.kills[i];
        let r_idx = usize::from(kill.r_idx);
        immobile_attacker(r_idx)
            && !engine::race_flag_has(r_idx, RF_UNIQUE)
            && (i32::from(kill.pos.y) - y).abs() <= 1
            && (i32::from(kill.pos.x) - x).abs() <= 1
            && (0..usize::from(engine::z_info().mon_blows_max))
                .any(|k| engine::race_blow_effect_name(r_idx, k) == Some(b"CONFUSE".as_slice()))
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
        if r_idx == 0 || engine::race_flag_has(r_idx, crate::headers::list_mon_race_flags_h::RF_NEVER_MOVE) {
            continue;
        }
        if (i32::from(kill.pos.y) - y).abs() > 2 || (i32::from(kill.pos.x) - x).abs() > 2 {
            continue;
        }
        danger = danger.saturating_add(crate::borg_danger::borg_danger_one_kill(st, y, x, 1, i, true, true));
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
fn step_allowed(mon: &mut crate::borg::BorgMonInfo, level: BorgTime, pos: (u8, u8), confused: bool) -> bool {
    if mon.immobile_stepped_level != level {
        mon.immobile_stepped = [[0; 4]; 16];
        mon.immobile_stepped_level = level;
    }
    let (count, limit) = if confused { (2, CONFUSED_STEPS) } else { (3, 1) };
    let used = |e: &[u8; 4]| e[2] > 0 || e[3] > 0;
    if let Some(e) = mon.immobile_stepped.iter_mut().find(|e| used(e) && (e[0], e[1]) == pos) {
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
    if st.borg.tr(BI_ACCW) >= p(Param::item_use_ccw_min_stock) {
        cures.push((TV_POTION as i32, st.item_val.sv_potion_cure_critical));
    }
    if confusion_clw_cure() {
        cures.push((TV_POTION as i32, st.item_val.sv_potion_cure_light));
    }
    if cures.into_iter().any(|(tval, sval)| borg_slot(st, tval, sval) != -1) {
        return true;
    }
    // A staff the chain can use has a charge left (`borg_use_staff_fail`).
    let staff = borg_slot(st, TV_STAFF as i32, st.item_val.sv_staff_healing);
    staff != -1 && st.items.items[staff as usize].pval != 0
}

/// borg-reborn-rs (#837): whether `step_allowed` would allow another step
/// away from the attacker at `pos`, without recording one.
fn step_left(mon: &crate::borg::BorgMonInfo, level: BorgTime, pos: (u8, u8), confused: bool) -> bool {
    if mon.immobile_stepped_level != level {
        return true;
    }
    let (count, limit) = if confused { (2, CONFUSED_STEPS) } else { (3, 1) };
    let used = |e: &[u8; 4]| e[2] > 0 || e[3] > 0;
    match mon.immobile_stepped.iter().find(|e| used(e) && (e[0], e[1]) == pos) {
        Some(e) => e[count] < limit,
        None => mon.immobile_stepped.iter().any(|e| !used(e)),
    }
}

/// Why the confusion cure is held (`confusion_cure_hold`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CureHold {
    /// #867: next to an immobile confuser, the attack goes first.
    Attack,
    /// #646: next to an immobile attacker, the step away goes first.
    Step,
}

/// Whether the confusion cure is held, and why, at both cure sites
/// (`borg_heal` and the cures at the end of `borg_caution`), which note the
/// reason once each (review of #868: the hold no longer notes it itself).
/// The attack (#867) is asked first.
fn confusion_cure_hold(st: &mut BorgState) -> Option<CureHold> {
    if cure_held_for_attack(st) {
        Some(CureHold::Attack)
    } else if cure_held_for_step(st) {
        Some(CureHold::Step)
    } else {
        None
    }
}

/// borg-reborn-rs (#867, owner: "I threw flasks at the patch immediately"):
/// with `attack_confused_by_direction`, next to an immobile monster whose
/// blow confuses, the cure waits and the attack goes first: the monster
/// confuses again before a cure is worth anything (a grey mushroom patch
/// has speed +10 and 2 HP, monster.txt), while a confused attack still
/// lands 11/32 of the time. Not while the moving monsters near are as
/// dangerous as the HP left, and only while an attack is open (review of
/// #868: afraid, no blow; hallucinating, no item attack; a magic mushroom
/// patch does both, with `SCARE` and `SPORE:HALLU`, monster.txt).
fn cure_held_for_attack(st: &mut BorgState) -> bool {
    st.borg.tr(BI_ISCONFUSED) != 0
        && p(Param::attack_confused_by_direction) != 0
        && immobile_confuser_next_to(st, st.borg.c.y, st.borg.c.x).is_some()
        && mobile_danger(st) < st.borg.tr(BI_CURHP)
        && confused_attack_open(st)
}

/// borg-reborn-rs (#646, review of #659): the confusion cure is held next to
/// an immobile attacker, so the step away (`immobile_step_away`) comes
/// first, unless the moving monsters near are as dangerous as the HP left.
///
/// borg-reborn-rs (#837, `caution_confusion_cure_when_stuck`): held only
/// while `immobile_step_away` can still step (a qualifying grid, tries left).
/// With none the Borg neither stepped nor cured, and melee went on confused.
fn cure_held_for_step(st: &mut BorgState) -> bool {
    if p(Param::caution_ranged_vs_immobile) == 0 || st.borg.tr(BI_ISCONFUSED) == 0 {
        return false;
    }
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    let Some(who) = kept_attacker_next_to(st, cy, cx).or_else(|| immobile_attacker_next_to(st, cy, cx)) else {
        return false;
    };
    if mobile_danger(st) >= st.borg.tr(BI_CURHP) {
        return false;
    }
    if p(Param::caution_confusion_cure_when_stuck) != 0 {
        let pos = (st.kills.kills[who].pos.y as u8, st.kills.kills[who].pos.x as u8);
        // In town `immobile_step_away` never steps.
        let can_step = st.borg.tr(BI_CDEPTH) != 0
            && step_left(&st.borg.mon, st.borg.time.level, pos, true)
            && best_step_away(st).is_some();
        if !can_step {
            borg_note(st, "# No step out of reach of the immobile attacker: curing confusion anyway (#837)");
            return false;
        }
    }
    true
}

/// borg-reborn-rs (#867, review of #868): whether `borg_attack` has an
/// attack to make now: its estimate has a winner with damage or kills. The
/// kills count too: a 2-HP patch caps the damage at 2, which the confused
/// weight (11/32) truncates to 0. The dry run draws the Borg's random numbers
/// and may clear the waiting flag: both are restored, so the check changes
/// nothing (as `throw_first` does).
fn confused_attack_open(st: &mut BorgState) -> bool {
    let (quick, value) = engine::rand_state();
    let waiting = st.borg.goal.waiting;
    let estimate = crate::borg_fight_attack::borg_attack_estimate(st);
    engine::set_rand_state(quick, value);
    st.borg.goal.waiting = waiting;
    estimate.is_some_and(|damage| damage > 0 || st.fight_attack.est_kills > 0.0)
}

/// borg-reborn-rs (#646): whether to step away from an adjacent immobile
/// attacker: confused, or with a ranged attack, and the monsters that move
/// less dangerous than the HP left.
fn step_away_wanted(confused: bool, missiles: i32, mobile_danger: impl FnOnce() -> i32, hp: i32) -> bool {
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
    if !ranged_rule || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    let (cy, cx) = (st.borg.c.y, st.borg.c.x);
    // Review of #717: a kept attacker first, so a plain-damage one beside
    // it does not take its place.
    let Some(who) = kept_attacker_next_to(st, cy, cx).or_else(|| immobile_attacker_next_to(st, cy, cx)) else {
        return false;
    };
    let confused = st.borg.tr(BI_ISCONFUSED) != 0;
    let (missiles, hp) = (st.borg.tr(BI_AMISSILES), st.borg.tr(BI_CURHP));
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
    let Some(i) = best_step_away(st) else { return false };
    let pos = (st.kills.kills[who].pos.y as u8, st.kills.kills[who].pos.x as u8);
    // #713: a creature kept at range allows the confused tries' count of
    // steps (3), not the single ranged step.
    if !step_allowed(&mut st.borg.mon, st.borg.time.level, pos, confused || kept) {
        return false;
    }
    let name = engine::race_name(usize::from(st.kills.kills[who].r_idx));
    let why = if kept { ", kept at range (#713)" } else if confused { ", confused" } else { ", ranged first" };
    borg_note(st, &format!("# Stepping away from immobile {}{} (#646)", String::from_utf8_lossy(&name), why));
    borg_keypress(st, i2d(i32::from(engine::ddd(i))));
    st.borg.goal.type_0 = 0;
    true
}

/// The grid `immobile_step_away` steps to (an index into the eight
/// directions), if any: a free floor grid with no trap, out of the reach of
/// every immobile attacker, less dangerous than this one. Chosen by
/// `caution_immobile_step_to_exit` (#736) and `caution_immobile_step_mobile_first`
/// (#837); records nothing (`cure_held_for_step` asks it too).
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
    let steps = if to_exit { crate::borg_flow_stairs::stair_steps_around(st) } else { [255; 8] };
    let mobile_first = p(Param::caution_immobile_step_mobile_first) != 0;
    let mut best: Option<(usize, (i32, i32, i32))> = None;
    for i in 0..8 {
        let x = cx + i32::from(engine::ddx_ddd(i));
        let y = cy + i32::from(engine::ddy_ddd(i));
        let ag = *st.cave.grid(x, y);
        if !borg_cave_floor_grid(&ag) || lava_forbids_step(st, &ag) || ag.kill != 0 || (ag.trap && !ag.glyph) || bouncing(st, y, x) {
            continue;
        }
        if engine::feat_is_shop(i32::from(ag.feat)) || immobile_attacker_next_to(st, y, x).is_some() {
            continue;
        }
        let k = borg_danger(st, y, x, 1, true, false);
        if k >= here {
            continue;
        }
        // #837: with `caution_immobile_step_mobile_first`, the danger from the
        // monsters that move there comes before the stairs.
        let mobile = if mobile_first { mobile_danger_at(st, y, x) } else { 0 };
        let key = (mobile, i32::from(steps[i]), k);
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
    } else { value.ceil() }
}

pub(crate) fn race_turns(hp: i32, hp_per_turn: f64, uses: Option<i32>, duration: f64, fallback_hp_per_turn: f64) -> Option<(f64, i32)> {
    if hp <= 0 || !hp_per_turn.is_finite() || hp_per_turn <= 0.0 || !duration.is_finite() || duration <= 0.0 { return None; }
    let hp = f64::from(hp);
    if fallback_hp_per_turn >= hp_per_turn && fallback_hp_per_turn > 0.0 { return Some((complete_attack_commands(hp / fallback_hp_per_turn), 0)); }
    let per_command = hp_per_turn * duration;
    let commands = complete_attack_commands(hp / per_command);
    let Some(uses) = uses else { return Some((commands * duration, 0)); };
    let uses = uses.max(0);
    if commands <= f64::from(uses) { return Some((commands * duration, commands as i32)); }
    if !fallback_hp_per_turn.is_finite() || fallback_hp_per_turn <= 0.0 { return None; }
    let primary_time = f64::from(uses) * duration;
    let remaining = hp - f64::from(uses) * per_command;
    Some((primary_time + complete_attack_commands(remaining / fallback_hp_per_turn), uses))
}

pub(crate) fn wins_race(turns: f64, incoming: f64, hp: i32) -> bool {
    turns.is_finite() && turns > 0.0 && incoming.is_finite() && incoming >= 0.0
        && hp > 0 && turns * incoming < f64::from(hp)
}

#[cfg(test)]
mod winning_fight_tests {
    use super::{race_turns, wins_race};

    #[test]
    fn observed_haste_is_included_without_changing_legacy_speed() {
        let mut kill = crate::borg_flow_kill::BorgKill::default();
        kill.speed = 110;
        assert_eq!(super::forecast_monster_speed(&kill), 110);
        kill.observed_fast = true;
        assert_eq!(super::forecast_monster_speed(&kill), 120);
        assert_eq!(kill.speed, 110, "the off planner keeps its original base-speed field");
    }

    #[test]
    fn remaining_hp_is_used_once_and_ties_do_not_win() {
        let (turns, _) = race_turns(6, 3.0, None, 1.0, 0.0).unwrap();
        assert_eq!(turns, 2.0, "6 remaining HP, not another injury discount");
        assert!(!wins_race(turns, 4.0, 8), "equal races do not win");
        assert!(wins_race(turns, 4.0, 9));
        assert!(!wins_race(turns, 8.0, 9), "double monster speed reverses the race");
    }

    #[test]
    fn a_finite_attack_needs_a_real_fallback() {
        assert_eq!(race_turns(20, 8.0, Some(1), 1.0, 4.0), Some((4.0, 1)));
        assert_eq!(race_turns(20, 8.0, Some(1), 1.0, 0.0), None);
        assert_eq!(race_turns(4, 8.0, Some(1), 1.0, 0.0), Some((1.0, 1)), "a finishing cast consumes a whole action and use");
        assert_eq!(race_turns(4, 8.0, Some(0), 1.0, 2.0), Some((2.0, 0)));
        assert_eq!(race_turns(0, 8.0, None, 1.0, 0.0), None, "unknown HP cannot certify a win");
        assert_eq!(race_turns(8, 8.0, Some(1), 0.5, 0.0), None, "one half-turn shot is not two shots");
        assert_eq!(race_turns(4, 8.0, Some(1), 0.5, 0.0), Some((0.5, 1)));
        assert!(!wins_race(f64::INFINITY, 1.0, 100));
        assert!(!wins_race(1.0, f64::NAN, 100));
    }

    #[test]
    fn finishing_attacks_and_fallbacks_book_complete_commands() {
        assert_eq!(race_turns(5, 10.0, None, 1.0, 0.0), Some((1.0, 0)));
        assert_eq!(race_turns(5, 9.0, Some(3), 1.0 / 3.0, 0.0), Some((2.0 / 3.0, 2)));
        assert_eq!(race_turns(13, 8.0, Some(1), 1.0, 4.0), Some((3.0, 1)));
        assert_eq!(race_turns(1, 2.0, None, 1.0, 0.0), Some((1.0, 0)));
        assert!(!wins_race(race_turns(1, 2.0, None, 1.0, 0.0).unwrap().0, 1.0, 1));
        let three = 3.0 * race_turns(5, 10.0, None, 1.0, 0.0).unwrap().0;
        assert_eq!(three, 3.0, "each foe needs its own finishing command");
    }

    #[test]
    fn fractional_primary_does_not_invent_a_fallback_command() {
        let (turns, arrows) = race_turns(30, 9.0, Some(5), 1.0 / 3.0, 3.0).unwrap();
        assert_eq!(arrows, 5);
        assert!((turns - 20.0 / 3.0).abs() < 1e-12);
        assert_eq!(super::complete_attack_commands(5.0 + 1e-10), 6.0,
            "a real residual above the boundary still requires another command");
        assert_eq!(super::complete_attack_commands(5.0 + f64::EPSILON * 5.0), 5.0);
    }

    #[test]
    fn firing_rate_changes_command_time_not_an_arrows_damage() {
        let rate = crate::borg_fight_attack::focused_projectile_rate(18_000, 30) as f64 / 1000.0;
        assert_eq!(rate, 54.0, "one 18HP arrow, three shots per turn: 54HP/turn");
        assert_eq!(race_turns(30, rate, Some(2), 1.0 / 3.0, 0.0), Some((2.0 / 3.0, 2)));
        let rate = crate::borg_fight_attack::focused_projectile_rate(6000, 30) as f64 / 1000.0;
        assert_eq!(rate, 18.0);
        let (turns, arrows) = race_turns(30, rate, Some(5), 1.0 / 3.0, 0.0).unwrap();
        assert_eq!(arrows, 5, "18HP/turn is 6HP per third-turn arrow, not18 per arrow");
        assert!((turns - 5.0 / 3.0).abs() < 1e-12);
        let rate = crate::borg_fight_attack::focused_projectile_rate(12_000, 20) as f64 / 1000.0;
        assert_eq!(race_turns(30, rate, Some(5), 0.5, 0.0), Some((1.5, 3)));
        let cl20_rate = crate::borg_fight_attack::focused_projectile_rate(12_000, 16) as f64 / 1000.0;
        assert_eq!(race_turns(30, cl20_rate, Some(3), 10.0 / 16.0, 0.0), Some((1.875, 3)), "PF_FAST_SHOT adds level/3: CL20 adds6 shots-tenths, not10");
        assert_eq!(race_turns(30, rate, Some(1), 0.5, 6.0), Some((3.5, 1)), "one12HP shot leaves18HP for three melee commands");
    }

    #[test]
    fn several_attackers_share_the_damage_budget() {
        let first = race_turns(4, 4.0, None, 1.0, 0.0).unwrap().0;
        let second = race_turns(8, 4.0, None, 1.0, 0.0).unwrap().0;
        assert!(wins_race(first, 3.0, 8));
        assert!(!wins_race(first + second, 3.0 + 2.0, 8));
    }
}

/// #758: shortest known/possible route to a grid adjacent to the player.
/// Unknown terrain is permitted by the caller as a conservative bound.
/// Other monsters do not permanently block a route: they may move during
/// Recall. This uses remembered terrain, never the engine's unseen map.
fn recall_contact_steps(from: loc, player: loc, max_moves: i32, passable: impl Fn(i32, i32) -> bool) -> Option<i32> {
    let adjacent = |p: loc| (p.x - player.x).abs().max((p.y - player.y).abs()) <= 1;
    if adjacent(from) { return Some(0); }
    let mut queue = std::collections::VecDeque::from([(from, 0)]);
    let mut seen = std::collections::BTreeSet::from([(from.x, from.y)]);
    while let Some((at, steps)) = queue.pop_front() {
        if steps >= max_moves { continue; }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let next = loc { x: at.x + dx, y: at.y + dy };
                if next.x <= 0 || next.y <= 0 || next.x >= AUTO_MAX_X - 1 || next.y >= AUTO_MAX_Y - 1
                    || !passable(next.x, next.y) || !seen.insert((next.x, next.y)) { continue; }
                if adjacent(next) { return Some(steps + 1); }
                queue.push_back((next, steps + 1));
            }
        }
    }
    None
}

/// Ranged exposure needs line of fire OR a possible melee route. A
/// projectable immobile caster can attack even without a walking route.
fn recall_attack_steps(contact: Option<i32>, has_spells: bool, projectable: bool) -> Option<i32> {
    if has_spells && (projectable || contact.is_some()) { Some(0) } else { contact }
}

#[cfg(test)]
mod recall_ranged_route_tests {
    #[test]
    fn immobile_caster_uses_line_of_fire_without_a_melee_route() {
        assert_eq!(super::recall_attack_steps(None, true, true), Some(0));
        assert_eq!(super::recall_attack_steps(None, true, false), None);
        assert_eq!(super::recall_attack_steps(Some(8), true, false), Some(0));
        assert_eq!(super::recall_attack_steps(Some(8), false, false), Some(8));
        assert_eq!(super::recall_attack_steps(None, false, false), None);
    }
}

fn recall_exposure(damage_per_turn: f64, player_energy: f64, monster_energy: f64, ticks: i32, steps: i32) -> f64 {
    let duration = f64::from(ticks) * player_energy / 10.0;
    let travel = f64::from(steps.max(0)) * player_energy / monster_energy.max(1.0);
    damage_per_turn * (duration - travel).max(0.0)
}

fn recall_forecast_allows(contact_hp: f64, unknown_risks: i32, hp: i32) -> bool {
    unknown_risks == 0 && contact_hp.is_finite() && contact_hp >= 0.0 && contact_hp < f64::from(hp)
}

#[cfg(test)]
mod recall_quantity_tests {
    #[test]
    fn acid_hp_and_unmodelled_risk_are_separate_reasons() {
        assert!(super::recall_forecast_allows(19.0, 0, 20), "modelled1d3 acid exposure can be safe; no200-point fear price");
        assert!(!super::recall_forecast_allows(19.0, 1, 20), "uncertainty refuses qualitatively, not as invented HP");
        assert!(!super::recall_forecast_allows(20.0, 0, 20), "equality at HP still refuses");
        assert!(!super::recall_forecast_allows(0.0, 1, 20), "a projectable immobile caster's unknown spell still counts");
    }
}

/// #758: only the flight's Recall entry point calls this. Evaluate every
/// known monster, including sleepers that may wake, and sum exposure. Doors
/// and walls obey known race movement flags. Casters that can fire now have
/// no approach discount. The maximum delay is conservative, not knowledge
/// of the engine's next random roll (effect-handler-general.c:1145).
pub(crate) fn flight_recall_survivable(st: &mut BorgState) -> bool {
    if p(Param::caution_recall_survival) == 0 { return true; }
    let ticks = p(Param::caution_recall_survival_delay);
    // #880: 1 prices POISON as damage, status blows by their dice and
    // never vetoes on a breeder; 0 is #758's rule (every blow other than
    // HURT/ACID/ELEC/FIRE/COLD/NONE is an unmodelled risk).
    let blow_model = p(Param::caution_recall_blow_model) != 0;
    let mut poison_strike: Option<f64> = None;
    let mut poison_timer = 0.0;
    let mut breeders_waived = 0;
    let mut contact_rates: Vec<f64> = Vec::new();
    let mut through_walls = false;
    let player_energy = f64::from(engine::extract_energy(engine::player_combat_speed().clamp(0, 199) as usize)).max(1.0);
    let mut exposure = 0.0;
    let mut unknown_risks = 0;
    let (mut awake, mut asleep, mut reachable) = (0, 0, 0);
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        let r = usize::from(kill.r_idx);
        if r == 0 || r >= engine::r_max().saturating_sub(1) { continue; }
        if !kill.awake { asleep += 1; } else { awake += 1; }
        let monster_energy = f64::from(engine::extract_energy(forecast_monster_speed(&kill).clamp(0, 199) as usize)).max(1.0);
        let flag = |f| engine::race_flag_has(r, f);
        let never_move = flag(RF_NEVER_MOVE);
        let pass_wall = flag(crate::headers::list_mon_race_flags_h::RF_PASS_WALL)
            || flag(crate::headers::list_mon_race_flags_h::RF_KILL_WALL);
        let open_door = flag(crate::headers::list_mon_race_flags_h::RF_OPEN_DOOR)
            || flag(crate::headers::list_mon_race_flags_h::RF_BASH_DOOR);
        let steps = recall_contact_steps(kill.pos, st.borg.c,
            if never_move { 0 } else { (f64::from(ticks) * monster_energy / 10.0).ceil() as i32 },
            |x, y| {
                let ag = st.cave.grid(x, y);
                let feat = u32::from(ag.feat);
                if feat == crate::headers::list_terrain_h::FEAT_PERM { return false; }
                if feat == crate::headers::list_terrain_h::FEAT_CLOSED { return open_door || pass_wall; }
                feat == FEAT_NONE || borg_cave_floor_grid(ag) || pass_wall
            });
        // Literal contact HP uses the actual walking route. A harmless
        // HEAL/SCARE flag does not turn a distant bite into a ranged hit.
        let forecast = fight_incoming_with(st, i, true);
        let projectable = forecast.ranged_risk && crate::borg_projection::borg_projectable_pure(
            st, kill.pos.y, kill.pos.x, st.borg.c.y, st.borg.c.x);
        let Some(risk_steps) = recall_attack_steps(steps, forecast.ranged_risk, projectable) else { continue };
        reachable += 1;
        let unknown = if blow_model { forecast.ranged_risk || forecast.recall_unknown_blow } else { forecast.unknown_damage };
        if unknown && recall_exposure(1.0, player_energy, monster_energy, ticks, risk_steps) > 0.0 {
            // #880: a breeder is the reason to leave, not a reason to stay.
            // Every breeder's spells at 574b618c are harmless during Recall
            // (BLIND/CONF/SCARE, DRAIN_MANA); its dice still count as HP.
            if blow_model && flag(RF_MULTIPLY) { breeders_waived += 1; } else { unknown_risks += 1; }
        }
        if let Some(contact_steps) = steps {
            let rate = if blow_model {
                forecast.hp_per_turn * recall_stagger_attack_share(
                    flag(crate::headers::list_mon_race_flags_h::RF_RAND_25),
                    flag(crate::headers::list_mon_race_flags_h::RF_RAND_50))
            } else { forecast.hp_per_turn };
            let hp = recall_exposure(rate, player_energy, monster_energy, ticks, contact_steps);
            exposure += hp;
            if hp > 0.0 {
                contact_rates.push(rate);
                through_walls |= pass_wall;
            }
            if blow_model && forecast.poisons {
                let at = recall_travel_ticks(contact_steps, monster_energy);
                if at < f64::from(ticks) {
                    poison_strike = Some(poison_strike.map_or(at, |t: f64| t.min(at)));
                    poison_timer += recall_poison_timer_ticks(f64::from(ticks) - at, monster_energy, i32::from(engine::race(r).level));
                }
            }
        }
    }
    let mut capped = None;
    if blow_model {
        // #880: only the grids next to the Borg can hold an attacker
        // (make_attack_normal only from a step into the Borg's grid, mon-move.c:1620-1632), so the
        // contact HP is at most the k fastest damage rates for the whole
        // delay, k being the neighbours that are not known wall.
        let open = if through_walls { 8 } else { recall_open_neighbours(|x, y| u32::from(st.cave.grid(x, y).feat), st.borg.c) };
        let cap = recall_contact_cap(&contact_rates, open, player_energy, ticks);
        if cap < exposure { capped = Some((exposure, open)); exposure = cap; }
    }
    let poison_hp = if blow_model {
        recall_poison_hp(ticks, st.borg.tr(BI_ISPOISONED) != 0, poison_strike, poison_timer, st.borg.tr(BI_RPOIS) != 0)
    } else { 0.0 };
    exposure += poison_hp;
    if poison_hp > 0.0 || breeders_waived > 0 || capped.is_some() {
        let mut parts = Vec::new();
        if let Some((sum, open)) = capped { parts.push(format!("contact HP {sum:.2} capped at {open} adjacent attackers")); }
        if poison_hp > 0.0 { parts.push(format!("{poison_hp:.0} poison HP included")); }
        if breeders_waived > 0 { parts.push(format!("{breeders_waived} breeders' unmodelled risk waived")); }
        borg_note(st, &format!("# Flight Recall forecast: {} (#880)", parts.join("; ")));
    }
    let allowed = recall_forecast_allows(exposure, unknown_risks, st.borg.tr(BI_CURHP));
    if allowed && asleep > 0 {
        borg_note(st, &format!("# Flight Recall forecast: known {awake} awake + {asleep} asleep; {reachable} reachable among them; {exposure:.2} modelled contact HP, HP {}: allowed (#758)", st.borg.tr(BI_CURHP)));
    }
    if unknown_risks > 0 {
        borg_note(st, &format!("# Refusing flight Recall: unmodelled spell/status risk from {unknown_risks} reachable monsters; modelled contact exposure {exposure:.2} HP during {ticks} world ticks, HP {} (#758)", st.borg.tr(BI_CURHP)));
    } else if !allowed {
        borg_note(st, &format!("# Refusing flight Recall: modelled contact exposure {exposure:.2} HP during {ticks} world ticks, HP {} (#758)", st.borg.tr(BI_CURHP)));
    }
    allowed
}

#[cfg(test)]
mod recall_survival_tests {
    use super::*;

    #[test]
    fn recall_delay_uses_world_time_at_every_player_speed() {
        let normal = recall_exposure(8.0, 10.0, 20.0, 34, 0);
        let fast_player = recall_exposure(4.0, 20.0, 20.0, 34, 0);
        assert_eq!(normal, 272.0);
        assert_eq!(normal, fast_player, "same monster actions during Recall at either player speed");
        assert_eq!(recall_exposure(8.0, 10.0, 20.0, 15, 0), 120.0);
        assert_eq!(recall_exposure(8.0, 10.0, 20.0, 15, 30), 0.0, "cannot arrive during the delay");
    }

    #[test]
    fn a_wall_or_closed_door_can_prevent_contact() {
        let from = loc { x: 3, y: 5 };
        let player = loc { x: 7, y: 5 };
        let corridor = |x, y| y == 5 && (3..=7).contains(&x);
        assert_eq!(recall_contact_steps(from, player, 3, corridor), Some(3));
        assert_eq!(recall_contact_steps(from, player, 2, corridor), None);
        assert_eq!(recall_contact_steps(from, player, 34, |x, y| corridor(x, y) && x != 5), None);
        assert_eq!(recall_contact_steps(loc { x: 6, y: 5 }, player, 0, |_, _| false), Some(0), "immobile but adjacent");
        assert_eq!(recall_contact_steps(from, player, 0, corridor), None, "immobile at range");
    }
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
    /// #880: a blow whose HP is not its dice even during Recall (SHATTER's
    /// earthquake). Status blows are priced by their dice alone there.
    pub recall_unknown_blow: bool,
    /// #880: a POISON blow, which also sets the poison timer.
    pub poisons: bool,
}

/// mon-move.c:1929-1934 (574b618c): Haste adds ten. Do not assume Slow
/// persists during a future fight or the Recall delay.
pub(crate) fn forecast_monster_speed(kill: &crate::borg_flow_kill::BorgKill) -> i32 {
    i32::from(kill.speed) + if kill.observed_fast { 10 } else { 0 }
}

// Recall is already pending during exposure: these non-damaging spells do
// not stop its countdown. They CAN change the ability to execute attacks,
// so a winning fight with spells still remains uncertified. monster_spell.txt
// :655-663,894-927,959-977,1056-1065 at 574b618c defines these effects.
// Teleporting, HASTE, summons and damaging spells are
// not silently priced as HP: their unmodelled contribution is a risk flag.
fn recall_spell_has_unknown_damage(spell: u32) -> bool {
    use crate::headers::list_mon_spells_h::*;
    !matches!(spell, RSF_HEAL | RSF_HEAL_KIN | RSF_SCARE | RSF_BLIND | RSF_CONF
        | RSF_DRAIN_MANA | RSF_FORGET)
}

fn modelled_blow(effect: &[u8]) -> bool {
    matches!(effect, b"HURT" | b"ACID" | b"ELEC" | b"FIRE" | b"COLD" | b"NONE")
}

/// #880: during the Recall delay the Borg only waits for the countdown,
/// which no status stops (word_recall ticks down in process_world,
/// game-world.c:709-714, whatever the player's timed effects). A status
/// blow (BLIND, CONFUSE, TERRIFY, PARALYZE, HALLU, LOSE_*, EXP_*,
/// DISENCHANT, DRAIN_CHARGES, EAT_*, BLACK_BREATH) costs the HP of its
/// dice, already counted for every blow at every monster turn; POISON
/// adds the poison timer, priced by `recall_poison_hp`. Only SHATTER
/// (mon-blows.c:1086-1110: an earthquake, which can hit the player) has HP beyond
/// its dice. The fight certificate (`unknown_fight`) is unchanged.
fn recall_blow_unmodelled(effect: &[u8]) -> bool {
    effect == b"SHATTER"
}

/// #880: poison is one timer that costs 1 HP per world tick while it runs
/// (game-world.c:513-515; process_world every ten game turns,
/// game-world.c:1080-1081). Each POISON hit adds 5 + randint1(rlev) ticks
/// (mon-blows.c:683-684, rlev the race level); POISONED is not
/// NONSTACKING (player_timed.txt), so hits add up (player-timed.c:1059-
/// 1070). Several poisoners still cost at most one HP per tick. The bound
/// is the smaller of the window from the earliest possible strike to the
/// end of the delay and the most timer the poisoners can add in it
/// (`timer_ticks`, every action a hit at the largest roll). Already
/// poisoned, the whole delay counts: the Borg sees the flag, not the
/// remaining duration (BI_ISPOISONED). Resist poison blocks the timer
/// (player_timed.txt POISONED `fail:2:POIS`). Damage reduction is ignored.
fn recall_poison_hp(ticks: i32, already_poisoned: bool, earliest_strike_tick: Option<f64>, timer_ticks: f64, resists: bool) -> f64 {
    if resists || ticks <= 0 { return 0.0; }
    let ticks = f64::from(ticks);
    if already_poisoned { return ticks; }
    earliest_strike_tick.map_or(0.0, |t| (ticks - t.max(0.0)).max(0.0).min(timer_ticks.max(0.0)))
}

/// #880: the most poison timer one monster can add in `window` world ticks:
/// one blow per action, each at the largest roll, 5 + rlev ticks.
fn recall_poison_timer_ticks(window: f64, monster_energy: f64, rlev: i32) -> f64 {
    (window.max(0.0) * monster_energy / 10.0).ceil() * f64::from(5 + rlev.max(1))
}

/// #880: world ticks a monster needs to walk `steps` grids. Each game turn
/// it gains extract_energy[speed] and each move costs 100 energy
/// (constants.txt move_energy), so one world tick of ten game turns is
/// energy/10 moves, and a grid costs 10/energy ticks.
fn recall_travel_ticks(steps: i32, monster_energy: f64) -> f64 {
    f64::from(steps.max(0)) * 10.0 / monster_energy.max(1.0)
}

/// #880: the share of an adjacent monster's turns that are attacks.
/// RAND_25/RAND_50 add up to a stagger chance (mon-move.c:1087-1103); a
/// staggering monster tries up to five random directions and attacks if
/// one is the Borg's grid (mon-move.c:1571-1632), at most 1 - (7/8)^5 of
/// the time. Confusion is ignored (it only lowers the share).
fn recall_stagger_attack_share(rand_25: bool, rand_50: bool) -> f64 {
    let stagger = if rand_25 { 0.25 } else { 0.0 } + if rand_50 { 0.5 } else { 0.0 };
    let hit_when_staggering = 1.0 - (7.0_f64 / 8.0).powi(5);
    1.0 - stagger + stagger * hit_when_staggering
}

/// #880: grids next to the Borg a monster could stand on. Known rubble,
/// magma, quartz, granite and permanent rock are closed: none is PASSABLE
/// (terrain.txt; feat_is_passable, cave-square.c:131; the same range as
/// borg_think_dungeon_util.rs's wall test). Unknown grids, doors, passable
/// rubble (PASS_RUBBLE is PASSABLE) and floors count as open. A monster
/// that passes walls is handled by the caller (all eight).
fn recall_open_neighbours(feat: impl Fn(i32, i32) -> u32, at: loc) -> usize {
    use crate::headers::list_terrain_h::{FEAT_PERM, FEAT_RUBBLE};
    let mut open = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            if !(FEAT_RUBBLE..=FEAT_PERM).contains(&feat(at.x + dx, at.y + dy)) { open += 1; }
        }
    }
    open
}

/// #880: contact HP at most the `open` largest per-turn rates, each for
/// the whole delay, however many monsters can reach the Borg.
fn recall_contact_cap(rates: &[f64], open: usize, player_energy: f64, ticks: i32) -> f64 {
    let mut sorted = rates.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    // No approach discount: each of the fastest attackers is at the Borg for
    // the whole delay (ticks * player_energy / 10 player turns).
    let duration = f64::from(ticks) * player_energy / 10.0;
    sorted.iter().take(open).map(|&r| r * duration).sum()
}

fn blow_hp_per_action(average: f64, chance: i32, energy_ratio: f64, vulnerable: bool) -> f64 {
    // project-player.c:72-73: elemental vulnerability multiplies by 4/3.
    // Ignore AC, resistance, immunity and damage reduction conservatively;
    // element melee uses max(physical, elemental), not their sum
    // (mon-blows.c:442-460). RF_HURT/IM in :311-365 belongs to the
    // monster-versus-monster target path, not a blow against the player.
    // Neither fear nor item-damage prices are HP.
    average.max(0.0) * f64::from(chance) / 10_000.0 * energy_ratio
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
    if kill.r_idx == 0 || (!kill.awake && !wake) { return IncomingDamage::default(); }
    let r = usize::from(kill.r_idx);
    let race = engine::race(r);
    let energy = |s: i32| f64::from(engine::extract_energy(s.clamp(0, 199) as usize)).max(1.0);
    let ratio = energy(forecast_monster_speed(&kill)) / energy(engine::player_combat_speed());
    let mut out = IncomingDamage { unknown_fight: kill.ranged_attack != 0, ..IncomingDamage::default() };
    for b in 0..usize::from(engine::z_info().mon_blows_max) {
        let Some(blow) = engine::race_blow(r, b) else { break };
        let effect = blow.effect_name.to_bytes();
        out.unknown_fight |= effect != b"HURT";
        out.unknown_damage |= !modelled_blow(effect);
        out.recall_unknown_blow |= recall_blow_unmodelled(effect);
        out.poisons |= effect == b"POISON";
        let vulnerable = match effect {
            b"ACID" => st.borg.tr(crate::borg_headers::borg_trait_h::BI_VACID) != 0,
            b"ELEC" => st.borg.tr(crate::borg_headers::borg_trait_h::BI_VELEC) != 0,
            b"FIRE" => st.borg.tr(crate::borg_headers::borg_trait_h::BI_VFIRE) != 0,
            b"COLD" => st.borg.tr(crate::borg_headers::borg_trait_h::BI_VCOLD) != 0,
            _ => false,
        };
        let chance = crate::borg_fight_attack::hit_permyriad(3 * i32::from(race.level).max(1) + blow.hit_power, st.borg.tr(crate::borg_headers::borg_trait_h::BI_ARMOR));
        let average = blow_average_tenths(blow.base, blow.dice, blow.sides, blow.m_bonus, i32::from(race.level)) as f64 / 10.0;
        out.hp_per_turn += blow_hp_per_action(average, chance, ratio, vulnerable);
    }
    use crate::headers::list_mon_spells_h::RSF_MAX;
    for spell in 1..RSF_MAX {
        if engine::flag_has(&race.spell_flags, spell as i32) {
            out.ranged_risk |= recall_spell_has_unknown_damage(spell);
        }
    }
    out.unknown_damage |= out.ranged_risk;
    out
}

#[cfg(test)]
mod incoming_units_tests {
    #[test]
    fn elemental_blows_and_harmless_spells_are_not_fear_prices() {
        assert_eq!(super::blow_hp_per_action(2.0, 9500, 1.0, false), 1.9, "1d3 acid is not 200/630/1014 HP");
        assert!(super::blow_hp_per_action(2.0, 9500, 2.0, true) < 6.0);
        use crate::headers::list_mon_spells_h::*;
        assert!(!super::recall_spell_has_unknown_damage(RSF_HEAL));
        assert!(!super::recall_spell_has_unknown_damage(RSF_SCARE));
        assert!(super::recall_spell_has_unknown_damage(RSF_MISSILE));
        assert!(super::recall_spell_has_unknown_damage(RSF_HASTE));
        assert!(super::recall_spell_has_unknown_damage(RSF_BLINK), "can bypass the approach route");
        assert!(super::recall_spell_has_unknown_damage(RSF_DARKNESS), "has a DARK_WEAK projection too");
        assert!(super::modelled_blow(b"ACID"));
        assert!(!super::modelled_blow(b"POISON"));
    }

    #[test]
    fn recall_prices_poison_and_status_blows_as_hp() {
        assert!(!super::recall_blow_unmodelled(b"POISON"), "#880: the white worm mass's 1d2 bite is HP");
        assert!(!super::recall_blow_unmodelled(b"PARALYZE"));
        assert!(!super::recall_blow_unmodelled(b"EXP_10"));
        assert!(super::recall_blow_unmodelled(b"SHATTER"), "an earthquake's HP is not its dice");
        // One timer, 1 HP per world tick, bounded by the delay and by the timer the hits can add.
        assert_eq!(super::recall_poison_hp(34, false, Some(4.0), 99.0, false), 30.0);
        assert_eq!(super::recall_poison_hp(34, false, Some(0.0), 99.0, false), 34.0);
        assert_eq!(super::recall_poison_hp(34, false, Some(0.0), 6.0, false), 6.0, "one white worm hit: 5 + randint1(1)");
        assert_eq!(super::recall_poison_hp(34, false, Some(40.0), 99.0, false), 0.0, "arrives after Recall");
        assert_eq!(super::recall_poison_hp(34, false, None, 0.0, false), 0.0);
        assert_eq!(super::recall_poison_hp(34, true, None, 0.0, false), 34.0, "an existing timer still ticks");
        assert_eq!(super::recall_poison_hp(34, true, Some(0.0), 99.0, true), 0.0, "resist poison blocks the timer");
        // White worm mass: speed 100 = energy 5, DL1. One action per two
        // ticks, each hit at most 6 ticks of timer: stacking keeps it going.
        assert_eq!(super::recall_poison_timer_ticks(2.0, 5.0, 1), 6.0);
        assert_eq!(super::recall_poison_timer_ticks(34.0, 5.0, 1), 102.0);
        // Normal speed (energy 10 per game turn): one grid per world tick.
        assert_eq!(super::recall_travel_ticks(3, 10.0), 3.0);
        assert_eq!(super::recall_travel_ticks(3, 20.0), 1.5);
        assert_eq!(super::recall_travel_ticks(3, 5.0), 6.0, "the white worm mass: two ticks a grid");
    }

    #[test]
    fn recall_contact_is_capped_by_adjacent_grids_and_erratic_moves() {
        assert_eq!(super::recall_stagger_attack_share(false, false), 1.0);
        let worm = super::recall_stagger_attack_share(true, true);
        assert!((worm - (0.25 + 0.75 * (1.0 - 0.875_f64.powi(5)))).abs() < 1e-12, "RAND_25|RAND_50: 75% stagger");
        let corridor = |_x: i32, y: i32| if y == 5 { crate::headers::list_terrain_h::FEAT_FLOOR } else { crate::headers::list_terrain_h::FEAT_GRANITE };
        assert_eq!(super::recall_open_neighbours(corridor, super::loc { x: 5, y: 5 }), 2);
        assert_eq!(super::recall_open_neighbours(|_, _| 0, super::loc { x: 5, y: 5 }), 8, "unknown grids may hold an attacker");
        use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_PASS_RUBBLE, FEAT_RUBBLE};
        let rubble = |_x: i32, y: i32| if y == 5 { FEAT_FLOOR } else { FEAT_RUBBLE };
        assert_eq!(super::recall_open_neighbours(rubble, super::loc { x: 5, y: 5 }), 2, "rubble is not PASSABLE");
        let pass = |_x: i32, y: i32| if y == 5 { FEAT_FLOOR } else { FEAT_PASS_RUBBLE };
        assert_eq!(super::recall_open_neighbours(pass, super::loc { x: 5, y: 5 }), 8, "passable rubble is");
        // 50 worms at 1 HP/turn in a corridor: two can bite at a time.
        let rates = vec![1.0; 50];
        assert_eq!(super::recall_contact_cap(&rates, 2, 10.0, 34), 68.0);
        assert_eq!(super::recall_contact_cap(&[3.0, 1.0, 2.0], 2, 10.0, 34), 170.0, "the fastest two");
    }
}

/// #759: a fresh comparison on every think, including during an existing
/// flight. Count every awake visible attacker, including those which can
/// join while another is fought. A target we cannot damage now prevents a
/// winning verdict; no optimistic missing-foe or unknown-HP assumptions.
fn winning_fight(st: &mut BorgState) -> bool {
    let mode = p(Param::caution_winning_fight);
    if mode == 0 { return false; }
    // The direct-damage race does not model random confused commands or
    // ongoing poison/bleeding damage. Let the ordinary cures act first.
    if [BI_ISCONFUSED, BI_ISBLIND, BI_ISIMAGE, BI_ISCUT, BI_ISPOISONED].iter().any(|&t| st.borg.tr(t) != 0) { return false; }
    let mut plans = Vec::new();
    let mut incoming = 0.0;
    let mut in_contact = false;
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || !kill.awake || kill.r_idx as usize >= engine::r_max().saturating_sub(1) {
            continue;
        }
        if crate::borg_town_hunt::viewed_target(st, &kill) {
            return false;
        }
        let d = engine::distance(st.borg.c, kill.pos);
        if st.cave.grid(kill.pos.x, kill.pos.y).info & crate::borg_cave_view::BORG_VIEW == 0 {
            if d <= 1 || crate::borg_danger::borg_danger_one_kill(st, st.borg.c.y, st.borg.c.x, 1, i as i32, true, true) > 0 {
                return false; // an attacker without a current observable target
            }
            continue;
        }
        if d > 1 && engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE) && kill.ranged_attack == 0 {
            continue;
        }
        let forecast = fight_incoming(st, i);
        if forecast.unknown_fight || forecast.unknown_damage { return false; }
        let danger = forecast.hp_per_turn;
        if danger <= 0.0 { continue; }
        in_contact |= d <= 1 || kill.ranged_attack != 0;
        incoming += danger;
        let Some(attack) = crate::borg_fight_attack::focused_attack(st, i) else { return false };
        let Some(hp) = crate::borg_fight_attack::observed_kill_hp(&kill) else { return false };
        plans.push((i, hp, attack));
    }
    if plans.is_empty() || !in_contact { return false; }
    // Remove the fastest-to-kill attacker first. Incoming remains the full
    // sum for the entire fight, a conservative bound as attackers die.
    plans.sort_by(|a, b| {
        let turns = |p: &(usize, i32, crate::borg_fight_attack::FocusedAttack)| f64::from(p.1) / f64::from(p.2.hp_per_turn_milli.max(1));
        turns(a).total_cmp(&turns(b)).then(a.0.cmp(&b.0))
    });
    let mana = st.borg.tr(BI_CURSP);
    let mut mana_left = mana;
    let mut spent = std::collections::BTreeMap::<i32, i32>::new();
    let mut turns = 0.0;
    for &(i, hp, attack) in &plans {
        let cost = attack.cost;
        let uses = cost.and_then(|c| c.uses).map(|u| {
            if cost.is_some_and(|c| c.spell) {
                if mana <= 0 { 0 } else { u * mana_left / mana }
            } else { (u - spent.get(&attack.kind).copied().unwrap_or(0)).max(0) }
        });
        // mon-move.c:1893-1897 gates regen on turn%100==0, regardless of
        // monster speed; :1914-1925 handles each monster once that turn.
        // regen_monster: at least 1 HP each 100 game turns,
        // doubled for REGENERATE. Charge its continuous mean immediately,
        // conservatively, rather than assuming a long fight has no healing.
        let kill = st.kills.kills[i];
        let regen = fight_regeneration(&kill);
        let damage = f64::from(attack.hp_per_turn_milli) / 1000.0 * f64::from(100 - cost.map_or(0, |c| c.fail_pct)) / 100.0 - regen;
        let fallback = (f64::from(attack.fallback) / 1000.0 - regen).max(0.0);
        let Some((t, used)) = race_turns(hp, damage, uses, attack.duration, fallback) else { return false };
        turns += t;
        if let Some(c) = cost {
            if c.spell {
                let per_cast = (mana + c.uses.unwrap_or(1).max(1) - 1) / c.uses.unwrap_or(1).max(1);
                mana_left = (mana_left - per_cast * used).max(0);
            } else { *spent.entry(attack.kind).or_default() += used; }
        }
    }
    let hp = st.borg.tr(BI_CURHP);
    let won = wins_race(turns, incoming, hp);
    let foes: Vec<String> = plans.iter().map(|&(i, needed, a)| format!("{}:needs {needed} HP, attack {} {:.3} HP/turn",
        String::from_utf8_lossy(engine::race_name(usize::from(st.kills.kills[i].r_idx))), a.kind, f64::from(a.hp_per_turn_milli) / 1000.0)).collect();
    borg_note(st, &format!("# Fight race: {} attackers, {turns:.3} turns to kill, {incoming:.3} damage/turn, HP {hp}: {} (#759)",
        plans.len(), if won { "winning" } else { "not winning" }));
    borg_note(st, &format!("# Fight race targets: {}", foes.join("; ")));
    // Mode 2 is a diagnostic: retain the unchanged control's actions so
    // estimates can be checked on its original losing AND winning fights.
    if !won || mode == 2 { return false; }
    let (i, _, attack) = plans[0];
    crate::borg_fight_attack::perform_focused_attack(st, i, attack)
}

pub(crate) fn fight_regeneration(kill: &crate::borg_flow_kill::BorgKill) -> f64 {
    let max_hp = kill.observed_health.expect("fight needs observed health").1;
    f64::from((i32::from(max_hp) / 100).max(1))
        * if engine::race_flag_has(usize::from(kill.r_idx), crate::headers::list_mon_race_flags_h::RF_REGENERATE) { 2.0 } else { 1.0 }
        / f64::from(engine::extract_energy(engine::player_combat_speed().clamp(0, 199) as usize)).max(1.0)
}

fn category_threat_safe(hp: i32, target_damage: f64, other_danger: i64, other_limit: i64) -> bool {
    hp > 0 && target_damage.is_finite() && target_damage < f64::from(hp) && other_danger <= other_limit
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
pub(crate) fn category_attack_safe_with(st: &mut BorgState, target: usize, gate_target: bool) -> bool {
    let hp = st.borg.tr(BI_CURHP);
    let target_damage = fight_incoming_with(st, target, true);
    let mut other_danger = 0_i64;
    for i in 1..st.kills.nxt.max(0) as usize {
        if i == target || st.kills.kills[i].r_idx == 0 { continue; }
        other_danger += i64::from(crate::borg_danger::borg_danger_one_kill(st, st.borg.c.y, st.borg.c.x, 1, i as i32, true, true)).max(0);
    }
    let limit = i64::from(hp) * i64::from(p(Param::caution_flee_danger_hp_multiplier));
    if !gate_target {
        // HP 0 is alive (the engine kills below zero).
        return hp >= 0 && other_danger <= limit;
    }
    !target_damage.unknown_damage && category_threat_safe(hp, target_damage.hp_per_turn, other_danger, limit)
}

#[cfg(test)]
mod category_threat_tests {
    #[test]
    fn first_sight_attack_yields_to_real_danger() {
        assert!(super::category_threat_safe(20, 6.5, 0, 40), "healthy Borg shoots lone cat");
        assert!(!super::category_threat_safe(6, 6.5, 0, 12), "cat can kill before another action");
        assert!(!super::category_threat_safe(20, 6.5, 41, 40), "other foes exceed existing flight threshold");
        assert!(super::category_threat_safe(20, 6.5, 40, 40), "ordinary threshold is strict greater-than");
    }
}

/// #760/#759: before any dungeon flight, including the light/countdown
/// and depth-return rungs which otherwise precede borg_caution.
pub(crate) fn fight_before_flight(st: &mut BorgState) -> bool {
    crate::borg_fight_attack::attack_on_sight(st) || winning_fight(st)
}

pub fn borg_caution(st: &mut BorgState) -> bool {
    watch_for_recall(st);
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

    // borg-reborn-rs (#840): short of kit, a real danger sends it to the stairs.
    // Its own test, not a branch of the chain above: with `kit_restock` on the
    // restock branch fires first and would mask it (review of #844, finding 3).
    if crate::borg_kit::leave_danger_percent(st)
        .is_some_and(|pct| pos_danger >= (st.borg.tr(BI_CURHP) * pct / 100).max(1))
        && !st.borg.goal.fleeing
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
                || (st.borg.tr(BI_CLEVEL) < p(Param::caution_stair_low_level)
                    && pos_danger > avoidance / p(Param::caution_stair_danger_divisor)))
                && on_upstair))
    // danger and standing on stair
    {
        choose_stairs(st, pos_danger);
    }

    // Take stairs up
    if st.borg.stair_less && !engine::player_opt(OPT_birth_force_descend) && !recall_holds_up_stair(st, pos_danger) {
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
        // borg-reborn-rs (#722, lead b): not from a one-think pause.
        && !breeder_stand_paused(st)
    {
        start_fleeing(st, "# Fleeing (breeder level)");
    }

    // borg-reborn-rs (#781 review): ordinary healing, defence, Recall and
    // emergency food keep priority; the unseen response precedes stair walking.
    if unseen_bump_response(st, pos_danger) { return true; }

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
    if crate::borg_escape::phase_rule_act(st) {
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
    let hold = if st.borg.tr(BI_ISCONFUSED) != 0 { confusion_cure_hold(st) } else { None };
    match hold {
        Some(CureHold::Attack) => borg_note(st, "# Not fixing Confusion next to an immobile confuser: attacking first (#867, cures)"),
        Some(CureHold::Step) => borg_note(st, "# Not fixing Confusion next to an immobile attacker (#646, cures)"),
        None => {}
    }
    let held = hold.is_some();
    if st.borg.tr(BI_ISCONFUSED) != 0 && !held {
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
        && !breeder_recall_held(st)
        && recall_escape_allowed(st, Some(pos_danger))
        && crate::borg_escape::borg_recall_for_flight(st)
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
    if recall_restricts(st) {
        return false;
    }
    if st.stairs.less.num != 0 {
        let depth = st.borg.tr(BI_CDEPTH) + 1;
        if crate::borg_prepared::prepared_c_for_decision(st, depth).is_some()
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
    let avoid = up && !(st.borg.tr(BI_CLEVEL) > 35 || st.borg.tr(BI_LIGHT) == 0) && !st.borg.status.desperate;
    let weight = stair_flight_weight(GOAL_FLEE, false);
    if weight > 0 {
        return crate::borg_flow_stairs::weighted_stair_reachable(st, up, avoid, weight);
    }
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
        // borg-reborn-rs (#593, owner): `caution_scary_unlatch_calmed` also
        // clears it once a scary monster is known hurt or afraid.
        let killed = st.borg.mon.scary_killed;
        let calmed = st.borg.mon.scary_calmed;
        let rules = UnlatchRules {
            latch: p(Param::caution_scary_latch) != 0,
            kill: p(Param::caution_scary_unlatch_kill) != 0,
            calmed: p(Param::caution_scary_unlatch_calmed) != 0,
        };
        if unlatch_when(rules, killed, calmed) && unlatch_scare(&mut st.borg.mon, &mut st.borg.goal) {
            borg_note(
                st,
                if killed && (rules.kill || rules.calmed) {
                    "# Scary guy killed: the level is no longer scary."
                } else if calmed && rules.calmed {
                    "# Scary guy hurt or afraid: the level is no longer scary."
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
        || easy_escape_with(st, p(Param::caution_breeder_easy_escape_steps), nearest_free_stair(st, true), down)
        || recall_pending_escape(p(Param::caution_recall_pending_escape) != 0, st.borg.goal.recalling);
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
/// borg-reborn-rs (#590, #593): the settings that decide when the scare
/// clears.
#[derive(Clone, Copy)]
struct UnlatchRules {
    latch: bool,
    kill: bool,
    calmed: bool,
}

/// borg-reborn-rs (#590, #593): may the scare clear, now that no scary
/// monster is known? With the kill rule (`caution_scary_unlatch_kill`), only
/// after a scary race was killed; with the calmed rule
/// (`caution_scary_unlatch_calmed`), after one is known hurt or afraid, or
/// killed; whatever the latch. Without either, only with the latch off.
fn unlatch_when(rules: UnlatchRules, killed: bool, calmed: bool) -> bool {
    if rules.kill || rules.calmed {
        // A kill is the stronger signal: the calmed rule clears on it too
        // (review of #595).
        (rules.kill && killed) || (rules.calmed && (calmed || killed))
    } else {
        !rules.latch
    }
}

/// borg-reborn-rs (#593): this scan's hurt or afraid monsters (`races`):
/// is one of a race that made the level scary?
pub(crate) fn note_calmed(mon: &mut crate::borg::BorgMonInfo, races: &[u16]) {
    mon.scary_calmed = races
        .iter()
        .any(|r| race_bit(*r).is_some_and(|(word, bit)| mon.scary_races[word] & bit != 0));
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
    mon.scary_calmed = false;
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
        st.borg.tr(BI_CDEPTH),
    )
}

fn recall_restricts_with(recalling: bool, as_usual: bool, depth: i32) -> bool {
    recalling && (!as_usual || depth <= 1)
}

/// borg-reborn-rs (#754, owner): with `caution_recall_pending_escape`, a
/// Word of Recall already counting down (`goal.recalling`) is an escape that
/// is running, so a scary level keeps its flight instead of "standing and
/// fighting" (#530 counts only a Recall still in the pack). #603's gates
/// (`recall_restricts`) still hold the flee flows and the down stairs, but
/// an up staircase reached on foot is still taken (`choose_stairs`, "Take
/// stairs up"): on DL1 that is town, where the pending Recall fires and
/// returns the Borg to its maximum depth. 0 = #530.
fn recall_pending_escape(on: bool, recalling: i32) -> bool {
    on && recalling != 0
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
    if !recall_wait_holds(
        p(Param::caution_recall_wait_no_explore) != 0,
        st.borg.goal.recalling,
        st.borg.tr(BI_CDEPTH),
    ) {
        return false;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    // Review of #762: the rest keeps `borg_check_rest`'s refusals (hungry,
    // weak, unlit, fear, a multiplier just killed, lava).
    let safe = borg_danger(st, y, x, 1, true, false) <= 0 && crate::borg_flow_misc::borg_check_rest(st, y, x);
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
    mon.scary_calmed = false;
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
    if p(Param::caution_slow_crowd_exit) == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return false;
    }
    if pos_danger < st.borg.tr(BI_CURHP) {
        return false;
    }
    let horizon = p(Param::danger_monster_max_distance);
    let energy = |speed: i32| i32::from(engine::extract_energy(speed.clamp(0, 199) as usize));
    let own = energy(st.borg.tr(BI_SPEED));
    let margin = p(Param::caution_slow_crowd_exit_margin_percent);
    let mut known = Vec::new();
    let mut crowd = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = &st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        let reach = (kill.pos.x - st.borg.c.x).abs().max((kill.pos.y - st.borg.c.y).abs());
        if reach > horizon {
            continue;
        }
        let still = engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE);
        known.push(CrowdMonster { reach, awake: kill.awake, still, energy: energy(i32::from(kill.speed)) });
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
            floor[i] = engine::square_in_bounds_fully(x, y) && borg_cave_floor_grid(ag) && feat != FEAT_NONE;
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
pub(crate) fn walk_away_from(st: &mut BorgState, from: loc, distance: i32, why: &std::ffi::CStr) -> bool {
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
            if st.cave.grid(x, y).kill != 0 || !passable(x, y) || engine::distance(from, at) < distance {
                continue;
            }
            if breeder_exit_grid(&passable, at, &[from], distance) {
                corridors.push(at);
            } else {
                floors.push(at);
            }
        }
    }
    let targets = if corridors.is_empty() { floors } else { corridors };
    if targets.is_empty() {
        return false;
    }
    crate::borg_flow::borg_flow_clear(st);
    for e in &targets {
        crate::borg_flow::borg_flow_enqueue_grid(st, e.y, e.x);
    }
    crate::borg_flow::borg_flow_spread(st, 250, true, true, false, -1, false);
    crate::borg_flow::borg_flow_commit(st, why, GOAL_FLEE) && crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
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

/// borg-reborn-rs (#556): with `caution_breeder_avoid`, remember the grid of
/// every known breeder (with the kill's `when`), forget marks older than
/// `caution_breeder_avoid_turns`, and rebuild `breeder_block`: the grids
/// within `caution_breeder_avoid_radius` of a mark. Called once per think
/// after the monsters are updated; it draws no random numbers.
pub(crate) fn note_breeders(st: &mut BorgState) {
    if p(Param::caution_breeder_avoid) == 0 {
        return;
    }
    let now = st.borg.time.now;
    let turns = p(Param::caution_breeder_avoid_turns);
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        if !engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY) {
            continue;
        }
        remember_breeder(&mut st.borg.breeder_marks, kill.pos, kill.when);
    }
    st.borg.breeder_marks.retain(|&(_, when)| now.wrapping_sub(when) <= turns);
    // Only a flight avoids the colony (owner, 2026-11-19: "the flight should
    // be from places where the breeding cannot be stopped any more"); while
    // the Borg fights them, nothing is blocked.
    let mut block = std::mem::take(&mut st.borg.breeder_block);
    if st.borg.goal.fleeing || st.borg.mon.scary {
        breeder_block_grid(&mut block, &st.borg.breeder_marks, p(Param::caution_breeder_avoid_radius));
    } else {
        block.clear();
    }
    st.borg.breeder_block = block;
}

/// borg-reborn-rs (#556, owner 2026-11-19: "the first choice should be to
/// kill them first and only escape if it is not possible"): with
/// `caution_breeder_stoppable`, once per think after the monster update,
/// whether the known breeders can still be wiped out (`breeders_stoppable`).
/// Once they cannot, the level is lost to them until a new level
/// (`breeders_lost`), so the Borg does not swing between fighting and
/// fleeing as breeders leave its view. Draws no random numbers.
pub(crate) fn note_breeders_stoppable(st: &mut BorgState) {
    st.borg.breeders_in_danger = false;
    if p(Param::caution_breeder_stoppable) == 0 || st.borg.mon.breeders == 0 || st.borg.tr(BI_CDEPTH) == 0 {
        return;
    }
    if st.borg.breeders_lost {
        return;
    }
    let mut breeders = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || engine::r_max() == 0 || kill.r_idx as i32 >= engine::r_max() as i32 - 1 {
            continue;
        }
        if !engine::race_flag_has(usize::from(kill.r_idx), RF_MULTIPLY) {
            continue;
        }
        breeders.push(i);
    }
    if breeders.is_empty() {
        return;
    }
    // borg-reborn-rs (#722, owner: "assume the level is lost if you could not
    // kill all the breeders"): with the policy at 3 or more, more breeders
    // known than the class can clear (caution_breeder_hunt_max_count) lose
    // the level at once.
    let max = p(Param::caution_breeder_hunt_max_count);
    if breeder_level_full(p(Param::caution_breeder_policy), breeders.len(), max) {
        st.borg.breeders_lost = true;
        st.borg.breeders_lost_turn = engine::game_turn();
        borg_note(st, &format!("# Breeders: {} known, more than the {max} this class clears: the level is lost.", breeders.len()));
        return;
    }
    let positions: Vec<loc> = (1..st.kills.nxt.max(0) as usize)
        .map(|i| st.kills.kills[i])
        .filter(|k| k.r_idx != 0)
        .map(|k| k.pos)
        .collect();
    let energy = |speed: i32| f64::from(crate::engine::extract_energy(speed.clamp(0, 199) as usize));
    let mut births = 0.0;
    for &i in &breeders {
        let kill = st.kills.kills[i];
        let k = positions.iter().filter(|q| (q.x - kill.pos.x).abs() <= 1 && (q.y - kill.pos.y).abs() <= 1).count();
        births += breed_chance(k as i32) * energy(i32::from(kill.speed)) / energy(st.borg.tr(BI_SPEED)).max(1.0);
    }
    // Every attack the Borg has (owner, 2026-11-19: "count all attacks"): the
    // attack planner's own dry run (`borg_attack_estimate`: melee, missiles,
    // throws, every spell with its mana and fail checks, wands, rods, staves,
    // activations) and the kills of the attack it would choose: for each
    // monster it hits, its true damage over that monster's HP, at most 1, so a
    // finishing blow is a kill and a ball over three mice is three. The dry
    // run draws the Borg's random numbers; their state is restored after it
    // (it also leaves an attack ranking in the decision trace and refills the
    // temporary target list, as #584's estimate does).
    let (quick, value) = crate::engine::rand_state();
    let estimate = crate::borg_fight_attack::borg_attack_estimate(st);
    crate::engine::set_rand_state(quick, value);
    let kills = st.fight_attack.est_kills;
    // Only a turn with something to attack is judged: out of reach is not
    // "cannot be stopped".
    if estimate.unwrap_or(0) <= 0 || kills <= 0.0 {
        return;
    }
    // The danger where the Borg stands (one turn, averaged), against the HP:
    // too high stops the stand for this think only (`breeders_in_danger`);
    // only the breeding outpacing the kills loses the level.
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    st.borg.breeders_in_danger = here >= st.borg.tr(BI_CURHP);
    let ok = stoppable(births, kills, p(Param::caution_breeder_stoppable_births_percent));
    // Lost only when the breeding outpaces the kills on
    // caution_breeder_stoppable_confirm judged thinks in a row.
    st.borg.breeders_behind = if ok { 0 } else { st.borg.breeders_behind.saturating_add(1) };
    let ok = ok || st.borg.breeders_behind < p(Param::caution_breeder_stoppable_confirm);
    if !ok {
        st.borg.breeders_lost = true;
        st.borg.breeders_lost_turn = engine::game_turn();
        borg_note(
            st,
            &format!(
                "# Breeders cannot be stopped: {} known, {births:.2} births a turn, {kills:.2} kills a turn (best attack {} damage), {} thinks in a row, danger {here}, HP {}",
                breeders.len(),
                estimate.unwrap_or(0),
                st.borg.breeders_behind,
                st.borg.tr(BI_CURHP)
            ),
        );
    }
}

/// borg-reborn-rs (#722): with the policy at 3 or more, more breeders known
/// than the class clears (`max`) lose the level.
fn breeder_level_full(policy: i32, known: usize, max: i32) -> bool {
    policy >= 3 && known > usize::try_from(max.max(0)).unwrap_or(0)
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

/// borg-reborn-rs (#722, owner: "if paths are blocked by breeders, recall
/// at some point"): on a level lost to breeders, Recall is due once the
/// search for stairs has had `caution_breeder_lost_recall_turns`.
pub(crate) fn breeder_recall_due(st: &BorgState) -> bool {
    st.borg.tr(BI_CDEPTH) > 0
        && lost_recall(
            p(Param::caution_breeder_policy),
            st.borg.breeders_lost,
            st.borg.breeders_lost_turn,
            engine::game_turn(),
            p(Param::caution_breeder_lost_recall_turns),
        )
        .1
}

/// The engine's chance that a breeder with `k` monsters in its 3x3 block
/// (itself included) breeds on its turn: 1 in `k` x `mon-play:mult-rate` (8,
/// constants.txt) while `k` < 4, never with 4 or more
/// (`monster_turn_multiply`, mon-move.c).
fn breed_chance(k: i32) -> f64 {
    let rate = f64::from(crate::engine::z_info().repro_monster_rate.max(1));
    if (1..4).contains(&k) {
        1.0 / (f64::from(k) * rate)
    } else {
        0.0
    }
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
        st.borg.mon.breeders > 0 && st.borg.tr(BI_CDEPTH) != 0,
    )
}

/// `breeder_stand_paused` with its inputs given.
fn stand_paused(stoppable: bool, lost: bool, in_danger: bool, breeders_here: bool) -> bool {
    stoppable && breeders_here && !lost && in_danger
}

fn stand_against_breeders(lost: bool, no_stairs_known: bool, in_danger: bool) -> bool {
    !in_danger && (!lost || no_stairs_known)
}

/// The colony can be stopped: the Borg kills `kills` breeders a turn, more
/// than the known breeders are expected to breed (`births` a turn, weighted
/// by `births_percent` for the walk between them and the ones not seen).
fn stoppable(births: f64, kills: f64, births_percent: i32) -> bool {
    births * f64::from(births_percent) / 100.0 < kills
}

/// borg-reborn-rs (#556): with `caution_breeder_stoppable`, the decision to
/// stand and hunt (`Some(stoppable)`); `None` with the switch off (the #503
/// stand and #558 hunt decide).
fn breeders_stoppable(st: &BorgState) -> Option<bool> {
    (p(Param::caution_breeder_stoppable) != 0)
        .then(|| {
            stand_against_breeders(
                st.borg.breeders_lost,
                // borg-reborn-rs (#722, owner: "search for stairs first"):
                // with the policy at 3 or more a lost level is left even
                // with no stairs known; the flight searches for them.
                st.stairs.less.num == 0 && st.stairs.more.num == 0 && p(Param::caution_breeder_policy) < 3,
                st.borg.breeders_in_danger,
            ) && st.borg.mon.breeders > 0
                && st.borg.tr(BI_CDEPTH) != 0
        })
}

/// borg-reborn-rs (#556): the Borg is stuck (the dungeon think's bravery
/// boosts are next) and remembered breeders block its flows: with
/// `caution_breeder_stuck_turns` above 0, the marks are suspended for that
/// many Borg turns (`breeder_marks_suspended`), so the flows may cross the
/// colony instead of leaving the Borg boxed in (human-druid-102023799: no
/// staircase known, every exploration route marked, 1,200 turns stuck
/// until the mice killed it). Draws no random numbers.
pub(crate) fn suspend_breeder_marks_when_stuck(st: &mut BorgState) {
    let turns = p(Param::caution_breeder_stuck_turns);
    if turns == 0 || st.borg.breeder_block.is_empty() || breeder_marks_suspended(st) {
        return;
    }
    st.borg.breeder_stuck = Some(st.borg.time.now);
    borg_note(st, &format!("# Stuck behind remembered breeders: ignoring them for {turns} turns"));
}

/// borg-reborn-rs (#556): the marks are suspended
/// (`suspend_breeder_marks_when_stuck`).
fn breeder_marks_suspended(st: &BorgState) -> bool {
    st.borg.breeder_stuck.is_some_and(|t| st.borg.time.now.wrapping_sub(t) < p(Param::caution_breeder_stuck_turns))
}

/// borg-reborn-rs (#556): `caution_breeder_avoid_fight_through` is on (the
/// stair flows' fallback through the breeders, `fight_through`).
pub(crate) fn breeder_fight_through_on() -> bool {
    p(Param::caution_breeder_avoid_fight_through) != 0
}

/// borg-reborn-rs (#556): a flight took the plain route through remembered
/// breeders (`fight_through`, borg_flow_stairs.rs) within the last
/// `caution_breeder_through_turns` Borg turns, and the danger here is below
/// the HP: the Borg fights its way along it instead of backing away.
pub(crate) fn breeder_fighting_through(st: &BorgState, pos_danger: i32) -> bool {
    st.borg.breeder_through.is_some_and(|t| {
        st.borg.time.now.wrapping_sub(t) <= p(Param::caution_breeder_through_turns)
    }) && pos_danger < st.borg.tr(BI_CURHP)
}

/// Record a breeder seen at `at` at `when`, keeping the latest time per grid.
fn remember_breeder(marks: &mut Vec<(loc, BorgTime)>, at: loc, when: BorgTime) {
    match marks.iter_mut().find(|(g, _)| g.x == at.x && g.y == at.y) {
        Some(m) => m.1 = m.1.max(when),
        None => marks.push((at, when)),
    }
}

/// Fill `block` with the grids (`y * AUTO_MAX_X + x`) within `radius` (the
/// engine's distance) of a mark; empty when there is no mark. The buffer is
/// reused from think to think.
fn breeder_block_grid(block: &mut Vec<bool>, marks: &[(loc, BorgTime)], radius: i32) {
    block.clear();
    if marks.is_empty() {
        return;
    }
    block.resize((AUTO_MAX_X * AUTO_MAX_Y) as usize, false);
    for &(m, _) in marks {
        for y in (m.y - radius).max(0)..=(m.y + radius).min(AUTO_MAX_Y - 1) {
            for x in (m.x - radius).max(0)..=(m.x + radius).min(AUTO_MAX_X - 1) {
                if engine::distance(m, loc { x, y }) <= radius {
                    block[(y * AUTO_MAX_X + x) as usize] = true;
                }
            }
        }
    }
}

/// borg-reborn-rs (#556): `(x, y)` is near a remembered breeder
/// (`breeder_block`; always false with `caution_breeder_avoid` 0).
pub(crate) fn near_known_breeder(st: &BorgState, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= AUTO_MAX_X || y >= AUTO_MAX_Y || breeder_marks_suspended(st) {
        return false;
    }
    st.borg.breeder_block.get((y * AUTO_MAX_X + x) as usize).copied().unwrap_or(false)
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
    // borg-reborn-rs (#753): not for a fleeing Borg at low HP (214960633:
    // held at 2 of 47 HP on a down staircase, danger 0 there, beside a
    // grey mold it kept walking past).
    if fleeing_hurt(st.borg.goal.fleeing, st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), p(Param::caution_rising_hold_min_hp_percent)) {
        return false;
    }
    let here = borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false);
    rising_hold(here, st.borg.tr(BI_CURHP))
}

/// #753: fleeing with less than `min_percent` of the max HP (0 = off).
fn fleeing_hurt(fleeing: bool, hp: i32, max_hp: i32, min_percent: i32) -> bool {
    fleeing && min_percent > 0 && i64::from(hp) * 100 < i64::from(max_hp.max(1)) * i64::from(min_percent)
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

/// borg-reborn-rs (#622, owner): how many character levels sooner a town
/// race (race level 0) stops being scary on the named list.
pub(crate) fn town_scary_levels_less(r_idx: usize) -> i32 {
    if crate::engine::race(r_idx).level == 0 { p(Param::caution_town_scary_levels_less) } else { 0 }
}

/// borg-reborn-rs (#503): whether the C Borg's hard-coded scary monsters by
/// character level (`borg_near_monster_type`) and the low-level door scares
/// (`borg_update`, `borg_messages`) mark the level scary.
pub(crate) fn scary_named_monsters() -> bool {
    p(Param::caution_scary_named_monsters) != 0
}

/// borg-reborn-rs (#647): with `caution_town_flee_no_worse`, in town while
/// the level is scary or the Borg flees, the flow refuses no grid that is
/// at most as dangerous as the Borg's own: standing still is no safer. A
/// low-level Borg next to a filthy street urchin rated its own grid 406 and
/// the one toward the stairs 203, above the flow's fear (90): no path left
/// its grid, and it twitched in town for 70,000 turns. 0 when off, when not
/// in town or not fleeing (the flow's own fear applies alone). It applies to
/// every flow in town while scary or fleeing, not only the stair flow. A grid
/// marked icky under a lower floor stays icky until the level changes
/// (`data_icky` is cleared by `borg_forget_map`): the floor unblocks grids
/// assessed while it holds, not earlier marks (review of #660). The depth is
/// the game's, as `keep_fox_form`'s.
pub(crate) fn town_flee_fear_floor(st: &mut BorgState) -> i32 {
    if !town_flee_applies(
        p(Param::caution_town_flee_no_worse),
        crate::engine::player_depth(),
        st.borg.mon.scary || st.borg.goal.fleeing || st.borg.goal.leaving,
    ) {
        return 0;
    }
    borg_danger(st, st.borg.c.y, st.borg.c.x, 1, true, false)
}

fn town_flee_applies(switch: i32, depth: i32, fleeing: bool) -> bool {
    switch != 0 && depth == 0 && fleeing
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

#[cfg(test)]
mod town_flee_tests {
    #[test]
    fn the_no_worse_floor_is_for_a_flight_in_town() {
        assert!(super::town_flee_applies(1, 0, true));
        assert!(!super::town_flee_applies(0, 0, true), "switch off");
        assert!(!super::town_flee_applies(1, 1, true), "the dungeon");
        assert!(!super::town_flee_applies(1, 0, false), "not fleeing");
    }
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
    // borg-reborn-rs (#556): with caution_breeder_stoppable, the Borg hunts
    // while the breeding can be stopped, at any count and level.
    if let Some(stop) = breeders_stoppable(st) {
        return stop;
    }
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
pub(crate) fn engaged_ignoring_own_leave(ignoring: bool, fleeing: bool, leaving: bool, scary: bool, own_leave: bool) -> bool {
    engaged_with(ignoring, fleeing || (leaving && !own_leave), scary)
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
pub(crate) fn cornered_boost(st: &mut BorgState, why: &str) -> bool {
    // borg-reborn-rs (#719, owner): "if I am cornered I should throw missile
    // and then boost myself".
    if throw_first(st, why) {
        return true;
    }
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
    let adjacent = (1..st.kills.nxt.max(0) as usize)
        .any(|i| st.kills.kills[i].r_idx != 0 && engine::distance(st.borg.c, st.kills.kills[i].pos) <= 1);
    // The dry run draws the Borg's random numbers and may clear the waiting
    // flag: both are restored, so a check that does not act changes nothing
    // (as the breeder estimate does).
    if !throw_first_applies(mode, why == "Cornered", adjacent, || {
        let (quick, value) = engine::rand_state();
        let waiting = st.borg.goal.waiting;
        let damage = crate::borg_fight_attack::borg_attack_estimate(st).unwrap_or(0);
        engine::set_rand_state(quick, value);
        st.borg.goal.waiting = waiting;
        damage
    }) {
        return false;
    }
    if crate::borg_fight_attack::borg_attack(st, false) {
        borg_note(st, &format!("# {why}: attacking at range before the boost (#719)"));
        return true;
    }
    false
}

/// The test of `throw_first`: on for this caller, nothing adjacent, and a
/// ranged attack that does damage (asked last: the estimate runs a dry run).
fn throw_first_applies(mode: i32, cornered_caller: bool, adjacent: bool, ranged_damage: impl FnOnce() -> i32) -> bool {
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
        && st.borg.tr(BI_CDEPTH) > 0
        && budget > 0
        && borg_timer(st, st.borg.time.level) < budget
        && pos_danger <= i32::from(st.borg.avoidance) * p(Param::caution_rising_descend_danger_tenths) / 10
}

/// borg-reborn-rs (#750): whether a stair flow for `why` skips the danger
/// limit (`caution_stairs_flight_ignore_danger`): a flight (`GOAL_FLEE`), or
/// any stair flow of `GOAL_BORE` while fleeing or leaving: the bored walk up
/// (`borg_think_dungeon_util.rs:1047`), the bored walks down (`:1087`,
/// `:1122`, `:1160`) and the powerdive (`borg_think_dungeon.rs:1264`).
pub(crate) fn stairs_flight_ignores_danger(why: i32, fleeing_or_leaving: bool) -> bool {
    p(Param::caution_stairs_flight_ignore_danger) != 0 && flight_goal(why, fleeing_or_leaving)
}

/// `fleeing_or_leaving`: `goal.fleeing || goal.leaving` (the caller's).
fn flight_goal(why: i32, fleeing_or_leaving: bool) -> bool {
    why == GOAL_FLEE || (why == GOAL_BORE && fleeing_or_leaving)
}

/// borg-reborn-rs (#751): `caution_stairs_clean_path`, the mode of the path
/// test (read here, as the registry asks of a caution parameter): at 2 a
/// monster blocks the path to the stairs only when it stands on it; at 3
/// also when it is next to it and ahead of the Borg.
pub(crate) fn stairs_clean_path_mode() -> i32 {
    p(Param::caution_stairs_clean_path)
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
    let danger = crate::borg_danger::borg_danger_one_kill(st, y, x, 1, kill as i32, true, false);
    danger_share_reaches(danger, st.borg.tr(BI_CURHP), percent)
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
    let recovering = st.borg.tr(BI_CURHP) < st.borg.tr(BI_MAXHP) || st.borg.tr(BI_CURSP) < st.borg.tr(BI_MAXSP);
    // Review of #616: a real threat (a share of the HP), not any danger; and
    // never in town, whose down staircase is a dive into DL1.
    let threshold = (st.borg.tr(BI_CURHP) * p(Param::caution_stairs_leave_danger_percent) / 100).max(1);
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
    let kit = crate::borg_kit::leave_danger_percent(st)
        .is_some_and(|pct| up && pos_danger >= (st.borg.tr(BI_CURHP) * pct / 100).max(1));
    if !(kit || stairs_leave_when(on, st.borg.tr(BI_CDEPTH), up || down, recovering, pos_danger, threshold)) {
        return false;
    }
    // The staircase under the Borg decides the way: choose_stairs alone may
    // not set it for a Borg that is recovering, not fleeing.
    if up {
        st.borg.stair_less = true;
    } else {
        st.borg.stair_more = true;
    }
    let why = if kit { "short of kit" } else { "while recovering" };
    borg_note(st, &format!("# Leaving by the stairs: a monster {why} (danger {pos_danger}, {}).", if up { "up" } else { "down" }));
    true
}

fn stairs_leave_when(on: bool, depth: i32, usable_stairs: bool, recovering: bool, pos_danger: i32, threshold: i32) -> bool {
    on && depth > 0 && usable_stairs && recovering && pos_danger >= threshold
}

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

    if crate::borg_prepared::prepared_c_for_decision(st, st.borg.tr(BI_CDEPTH) + 1).is_none() {
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
    crate::borg_recover::watch_for_rest(st);
}

/// An awake monster that can move, seen in this update, within `range` of
/// the Borg (`mover_seen_within`; #615 rule a).
/// The kill index of the nearest one, if any (review of #645: a clock set by
/// a monster is cleared when that monster dies, `monster_died`).
pub(crate) fn nearest_mover_seen(st: &BorgState, range: i32) -> Option<i32> {
    let kills = &st.kills.kills[1..st.kills.nxt.max(1) as usize];
    nearest_mover_seen_in(kills, st.borg.c, range, |r| engine::race_flag_has(usize::from(r), RF_NEVER_MOVE))
        .map(|i| i as i32 + 1)
}

/// borg-reborn-rs (#567, review of #645): the monster that set a watch
/// clock died, so the clock no longer stands for a monster that may come
/// back; clear it. A monster forgotten out of view (not dead) keeps the
/// clock: that is #567's case (Grip came back).
pub(crate) fn monster_died(st: &mut BorgState, k: i32) {
    if st.borg.recall_watch_kill == k {
        st.borg.recall_watch_turn = 0;
        st.borg.recall_watch_kill = 0;
    }
    if st.borg.rest_watch_kill == k {
        st.borg.rest_watch_turn = 0;
        st.borg.rest_watch_kill = 0;
    }
}

/// An awake monster that can move, seen in this update (`kill.seen`; review
/// of #640: a record keeps its last position for up to 2,000 ticks after the
/// monster is lost, and refreshed the watch every think), within `range`
/// (the engine's distance) of `c`. A monster that never moves (a floating
/// eye, a mold) cannot come back.
#[cfg(test)]
fn mover_seen_within(kills: &[crate::borg_flow_kill::BorgKill], c: loc, range: i32, never_moves: impl Fn(u16) -> bool) -> bool {
    nearest_mover_seen_in(kills, c, range, never_moves).is_some()
}

/// The position in `kills` of the nearest such monster.
fn nearest_mover_seen_in(
    kills: &[crate::borg_flow_kill::BorgKill],
    c: loc,
    range: i32,
    never_moves: impl Fn(u16) -> bool,
) -> Option<usize> {
    kills
        .iter()
        .enumerate()
        .filter(|(_, kill)| {
            kill.r_idx != 0 && kill.seen && kill.awake && engine::distance(c, kill.pos) <= range && !never_moves(kill.r_idx)
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
    borg_note(st, &format!("# Watching for Recall: a monster was near {ago} game turns ago."));
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

    // borg-reborn-rs (#749): C's "less than 100 HP lost" is true at full HP for
    // a low-level character; with caution_recall_cure_min_loss_percent, a cure
    // needs at least that share of the max HP lost.
    let lost = st.borg.tr(BI_MAXHP) - st.borg.tr(BI_CURHP);
    if lost < p(Param::caution_recall_cure_loss)
        && recall_cure_hurt_enough(lost, st.borg.tr(BI_MAXHP), p(Param::caution_recall_cure_min_loss_percent))
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
    let weight = stair_flight_weight(GOAL_FLEE, false);
    let depth = flee_down_depth(max_cost);
    let ok = if weight > 0 {
        crate::borg_flow_stairs::weighted_down_flight(st, depth, weight)
    } else {
        crate::borg_flow::borg_flow_clear(st);
        crate::borg_flow_stairs::enqueue_down_stairs(st);
        crate::borg_flow::borg_flow_spread(st, depth, true, false, false, -1, false);
        crate::borg_flow::borg_flow_commit(st, c"down-stairs", GOAL_FLEE) && {
            st.borg.flight_to_stairs = true; // #614: a flight to a staircase
            crate::borg_flow::borg_flow_old(st, GOAL_FLEE)
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
    if flight_goal(why, leaving_up) { p(Param::caution_stairs_flight_threat_weight) } else { 0 }
}

/// The weighted route is checked after it is planned, including for a
/// desperate walk. A clean detour can satisfy the same check.
pub(crate) fn weighted_stairs_blocker_percent(st: &BorgState) -> Option<i32> {
    let check = if st.borg.status.desperate {
        p(Param::caution_stairs_clean_path) != 0
    } else {
        stairs_clean_path_flee_on()
    };
    check.then(|| p(Param::caution_stairs_block_danger_percent))
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
mod recall_watch_tests {
    use super::{mover_seen_within, recall_watch_holds};
    use crate::borg_flow_kill::BorgKill;
    use crate::headers::z_type_h::loc;

    /// Review of #640: only a monster seen now refreshes the watch; a record
    /// left at its last position does not.
    #[test]
    fn only_a_monster_seen_now_refreshes_the_watch() {
        let borg = loc { x: 10, y: 10 };
        let grip = BorgKill { r_idx: 54, awake: true, seen: true, pos: loc { x: 14, y: 10 }, ..BorgKill::BLANK };
        let moves = |_: u16| false;
        assert!(mover_seen_within(&[grip], borg, 20, moves), "Grip in view, 4 grids away");
        assert!(!mover_seen_within(&[BorgKill { seen: false, ..grip }], borg, 20, moves), "a stale record");
        assert!(!mover_seen_within(&[BorgKill { awake: false, ..grip }], borg, 20, moves), "asleep");
        assert!(!mover_seen_within(&[grip], borg, 3, moves), "out of range");
        assert!(!mover_seen_within(&[grip], borg, 20, |_| true), "never moves (a floating eye)");
        assert!(!mover_seen_within(&[BorgKill { r_idx: 0, ..grip }], borg, 20, moves), "a wiped record");
    }

    /// #567: priest-809881907 saw Grip at game turn 1,761 and rested for
    /// Recall at 1,771, 10 game turns later.
    #[test]
    fn no_recall_rest_soon_after_a_monster_was_near() {
        assert!(recall_watch_holds(100, 1761, 10, true), "Grip, 10 game turns ago");
        assert!(recall_watch_holds(100, 1761, 100, true), "at the limit");
        assert!(!recall_watch_holds(100, 1761, 101, true), "long enough ago: rest");
        assert!(!recall_watch_holds(100, 1761, 10, false), "another level");
        assert!(!recall_watch_holds(100, 0, 10, true), "no monster recorded yet");
        assert!(!recall_watch_holds(0, 1761, 10, true), "off (C)");
    }
}

#[cfg(test)]
mod stairs_leave_tests {
    use super::stairs_leave_when;

    #[test]
    fn a_recovering_borg_on_the_stairs_leaves_when_a_monster_threatens() {
        assert!(stairs_leave_when(true, 2, true, true, 5, 3));
        assert!(!stairs_leave_when(true, 2, true, true, 2, 3), "below the threat threshold");
        assert!(!stairs_leave_when(true, 0, true, true, 5, 3), "town: no dive into DL1");
        assert!(!stairs_leave_when(true, 2, true, false, 5, 3), "nothing to recover: fight as before");
        assert!(!stairs_leave_when(true, 2, false, true, 5, 3), "no usable staircase (or a down one with leave_down 0)");
        assert!(!stairs_leave_when(false, 2, true, true, 5, 3), "switch off (C)");
    }
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
        let r = |latch, kill, calmed| UnlatchRules { latch, kill, calmed };
        assert!(!unlatch_when(r(true, false, false), true, true), "C: the latch holds even after a kill");
        assert!(unlatch_when(r(false, false, false), false, false), "latch off: whenever none is known");
        assert!(!unlatch_when(r(false, true, false), false, false), "kill rule: forgotten is not enough");
        assert!(unlatch_when(r(true, true, false), true, false), "kill rule: a kill clears with the latch on");
        assert!(unlatch_when(r(false, true, false), true, false));
        assert!(!unlatch_when(r(true, true, false), false, true), "kill rule alone: calmed is not enough");
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
        assert!(
            !unlatch_when(UnlatchRules { latch: true, kill: true, calmed: false }, mon.scary_killed, false),
            "forgotten, not killed: the flag stays"
        );
        note_kill(&mut mon, 42);
        note_scan(&mut mon, false); // killed and none left known
        assert!(
            unlatch_when(UnlatchRules { latch: true, kill: true, calmed: false }, mon.scary_killed, false),
            "the last one killed: the flag clears"
        );
    }

    #[test]
    fn a_hurt_or_afraid_scary_monster_clears_the_scare() {
        let r = |latch, kill, calmed| UnlatchRules { latch, kill, calmed };
        assert!(unlatch_when(r(true, false, true), false, true), "calmed rule, latch on");
        assert!(unlatch_when(r(true, false, true), true, false), "calmed rule alone: a kill clears too");
        assert!(unlatch_when(r(true, true, true), true, false), "both rules: a kill");
        assert!(unlatch_when(r(true, true, true), false, true), "both rules: calmed");
        assert!(!unlatch_when(r(false, false, true), false, false), "calmed rule, latch off: forgotten is not enough");
        assert!(!unlatch_when(r(true, false, true), false, false), "calmed rule alone: forgotten is not enough");
        let mut mon = BorgMonInfo::default();
        note_scary_race(&mut mon, 42);
        note_calmed(&mut mon, &[7]);
        assert!(!mon.scary_calmed, "another race calmed");
        note_calmed(&mut mon, &[42]);
        assert!(mon.scary_calmed, "the scary race is hurt or afraid");
        note_calmed(&mut mon, &[]);
        assert!(!mon.scary_calmed, "each scan decides afresh");
    }

    #[test]
    fn only_races_that_made_the_level_scary_clear_it() {
        // The second review of #595: race 42 made the level scary; race 7 was
        // only ever seen hurt or afraid, so it is not in the record.
        let r = |latch, kill, calmed| UnlatchRules { latch, kill, calmed };
        let mut mon = BorgMonInfo::default();
        note_scary_race(&mut mon, 42);
        note_scan(&mut mon, true);
        note_kill(&mut mon, 7); // a hurt race-7 monster is killed
        assert!(!mon.scary_killed, "a race that never made the level scary is no kill");
        note_calmed(&mut mon, &[7]); // race 42 forgotten, race 7 known hurt
        assert!(!mon.scary_calmed, "nor is it calmed");
        assert!(!unlatch_when(r(true, true, true), mon.scary_killed, mon.scary_calmed), "the flag stays");
        note_calmed(&mut mon, &[42]);
        assert!(unlatch_when(r(true, false, true), mon.scary_killed, mon.scary_calmed), "race 42 hurt: it clears");
    }

    #[test]
    fn a_pending_recall_restricts_only_without_the_switch() {
        assert!(recall_restricts_with(true, false, 5), "C: Recall-only rules");
        assert!(!recall_restricts_with(true, true, 5), "the switch: the ordinary rules on DL5");
        assert!(!recall_restricts_with(true, true, 2), "and on DL2");
        assert!(recall_restricts_with(true, true, 1), "not on DL1: the up staircase leads to town");
        assert!(recall_restricts_with(true, true, 0), "not in town: the down staircase leads to the dungeon");
        assert!(!recall_restricts_with(false, false, 5), "no Recall pending");
        assert!(!recall_restricts_with(false, true, 0));
    }

    #[test]
    fn a_pending_recall_is_an_escape_only_with_the_switch() {
        assert!(!recall_pending_escape(false, 30), "#530: a pending Recall is no escape");
        assert!(recall_pending_escape(true, 30), "the switch: it is one");
        assert!(!recall_pending_escape(true, 0), "no Recall pending");
    }

    #[test]
    fn the_recall_wait_skips_exploration_only_in_the_dungeon() {
        assert!(!recall_wait_holds(false, 30, 1), "off: C explores while waiting");
        assert!(recall_wait_holds(true, 30, 1), "on, DL1");
        assert!(recall_wait_holds(true, 30, 7), "on, DL7");
        assert!(!recall_wait_holds(true, 30, 0), "not in town");
        assert!(!recall_wait_holds(true, 0, 3), "no Recall pending");
    }

    #[test]
    fn every_switched_site_asks_recall_restricts() {
        // The seven sites of #603 (review of #627): a site put back to
        // `goal.recalling != 0` would ignore the switch and still pass the
        // helper's test, so count the calls in each file.
        for (file, src, calls) in [
            ("borg_caution.rs", include_str!("borg_caution.rs"), 3),
            ("borg_flow_stairs.rs", include_str!("borg_flow_stairs.rs"), 1),
            ("borg_escape.rs", include_str!("borg_escape.rs"), 1),
            ("borg_fight_attack.rs", include_str!("borg_fight_attack.rs"), 1),
            ("borg_think_dungeon.rs", include_str!("borg_think_dungeon.rs"), 2),
        ] {
            assert_eq!(src.matches(concat!("recall_", "restricts(st)")).count(), calls, "{file}");
        }
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

    /// #690: the crowd can be walked away from only when a monster is
    /// adjacent and every awake monster that moves is slower by the margin.
    #[test]
    fn a_crowd_is_slow_when_every_awake_mover_is_slower_by_the_margin() {
        let m = |reach, awake, still, energy| CrowdMonster { reach, awake, still, energy };
        // The case (borg-cemetery#78): yellow worm masses, speed 100 (energy
        // 5) against the Borg's 110 (energy 10), with a grey mushroom patch.
        let worms = [m(1, true, false, 5), m(1, true, false, 5), m(2, true, false, 5), m(1, true, true, 10)];
        assert!(slow_crowd(&worms, 10, 25), "the worm colony with a mushroom patch");
        assert!(!slow_crowd(&[m(1, true, false, 10)], 10, 25), "as fast as the Borg");
        assert!(!slow_crowd(&[m(1, true, false, 5), m(4, true, false, 20)], 10, 25), "a faster monster near holds it");
        assert!(slow_crowd(&[m(1, true, false, 5), m(4, false, false, 20)], 10, 25), "a faster one asleep does not");
        assert!(slow_crowd(&[m(1, true, true, 30)], 10, 25), "a fast one that never moves");
        // Speed 105 is energy 7: 10 against 7 clears a 25% margin, not 50%.
        assert!(slow_crowd(&[m(1, true, false, 7)], 10, 25), "105 at 25%");
        assert!(!slow_crowd(&[m(1, true, false, 7)], 10, 50), "105 at 50%");
        assert!(!slow_crowd(&[m(2, true, false, 5)], 10, 25), "none adjacent");
        assert!(!slow_crowd(&[], 10, 25), "no monster");
    }

    /// #690: a crowd exit is a chokepoint or a staircase, beyond the distance
    /// from every monster of the crowd; a room corner is not, unless it holds
    /// a staircase.
    #[test]
    fn crowd_exits_are_corridors_and_staircases_beyond_the_crowd() {
        const ROOM: [&str; 7] = [
            "##############",
            "#ww.........>#",
            "#w@w.........#",
            "#ww..........#",
            "#............####",
            "#................",
            "#################",
        ];
        let passable = map_passable(&ROOM);
        let stairs = |x: i32, y: i32| ROOM.get(y as usize).and_then(|r| r.as_bytes().get(x as usize)) == Some(&b'>');
        let mut crowd = Vec::new();
        for (y, row) in ROOM.iter().enumerate() {
            for (x, c) in row.bytes().enumerate() {
                if c == b'w' {
                    crowd.push(loc { x: x as i32, y: y as i32 });
                }
            }
        }
        assert!(crowd_exit_grid(&passable, &stairs, loc { x: 12, y: 1 }, &crowd, 6), "the staircase in the far corner");
        assert!(!crowd_exit_grid(&passable, &stairs, loc { x: 12, y: 4 }, &crowd, 6), "a room grid");
        assert!(!crowd_exit_grid(&passable, &stairs, loc { x: 12, y: 1 }, &crowd, 12), "the staircase within the distance");
        assert!(crowd_exit_grid(&passable, &stairs, loc { x: 13, y: 5 }, &crowd, 6), "the doorway on the east wall");
        assert!(!breeder_exit_grid(&passable, loc { x: 12, y: 1 }, &crowd, 6), "the breeder exit takes no staircase");
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
mod throw_first_tests {
    use super::throw_first_applies;

    /// #719: the order decision.
    #[test]
    fn throw_before_the_boost() {
        assert!(!throw_first_applies(0, true, false, || 10), "off");
        assert!(throw_first_applies(1, true, false, || 10), "cornered, closing in, a throw");
        assert!(!throw_first_applies(1, false, false, || 10), "mode 1: not the breeder or Phase rule boost");
        assert!(throw_first_applies(2, false, false, || 10), "mode 2: every boost");
        assert!(!throw_first_applies(1, true, true, || 10), "adjacent: buff");
        assert!(!throw_first_applies(1, true, false, || 0), "nothing reaches: buff");
        assert!(!throw_first_applies(1, true, true, || panic!("no dry run when adjacent")));
    }
}

#[cfg(test)]
mod flight_tuning_tests {
    use super::{danger_share_reaches, flight_goal, recall_cure_hurt_enough, GOAL_BORE, GOAL_FLEE};

    /// #753: the rising hold lets a hurt fleeing Borg down.
    #[test]
    fn a_hurt_flight_is_not_held() {
        use super::fleeing_hurt;
        assert!(fleeing_hurt(true, 2, 47, 50), "2 of 47 HP");
        assert!(!fleeing_hurt(true, 30, 47, 50));
        assert!(!fleeing_hurt(false, 2, 47, 50), "not fleeing");
        assert!(!fleeing_hurt(true, 2, 47, 0), "0: today");
    }

    /// #750: which stair flows are flights.
    #[test]
    fn flight_goals() {
        assert!(flight_goal(GOAL_FLEE, false));
        assert!(flight_goal(GOAL_BORE, true), "heading up while fleeing");
        assert!(!flight_goal(GOAL_BORE, false), "a calm bored walk keeps the danger limit");
        assert!(!flight_goal(1, true));
    }

    /// #747: a faster neighbour counts only when dangerous enough.
    #[test]
    fn a_fast_neighbour_needs_danger() {
        assert!(danger_share_reaches(8, 15, 50), "half the HP");
        assert!(!danger_share_reaches(1, 15, 50), "a fruit bat's bite");
        assert!(danger_share_reaches(0, 15, 0), "0: any danger counts");
    }

    /// #749: a Recall buy-time cure needs real HP lost.
    #[test]
    fn a_cure_needs_hp_lost() {
        assert!(recall_cure_hurt_enough(0, 15, 0), "0 = C: even at full HP");
        assert!(!recall_cure_hurt_enough(0, 15, 30), "full HP: no cure");
        assert!(!recall_cure_hurt_enough(4, 15, 30));
        assert!(recall_cure_hurt_enough(5, 15, 30));
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

#[cfg(test)]
mod breeder_avoid_tests {
    use super::*;

    /// #556: one mark per grid, with the latest time it was seen there.
    #[test]
    fn a_breeder_grid_keeps_its_latest_sighting() {
        let mut marks = Vec::new();
        remember_breeder(&mut marks, loc { x: 5, y: 7 }, 10);
        remember_breeder(&mut marks, loc { x: 5, y: 7 }, 30);
        remember_breeder(&mut marks, loc { x: 5, y: 7 }, 20);
        remember_breeder(&mut marks, loc { x: 6, y: 7 }, 25);
        assert_eq!(marks.len(), 2);
        assert_eq!(marks[0].1, 30, "the latest sighting wins");
        assert_eq!(marks[1].1, 25);
    }

    /// #556 (owner): the engine's breeding chance, and the colony stoppable
    /// while the kills outpace the weighted births.
    #[test]
    fn a_colony_is_stoppable_while_the_borg_kills_faster_than_it_breeds() {
        // 4 lone mice: 4 x 1/8 = 0.5 births a turn; one kill a turn.
        assert!(stoppable(0.5, 1.0, 100));
        assert!(!stoppable(0.5, 1.0, 200), "doubled for the walk: 1.0, not below 1");
        assert!(stoppable(0.4, 1.0, 200));
        // Two turns a kill: 0.5 kills a turn.
        assert!(!stoppable(0.3, 0.5, 200));
        assert!(!stoppable(0.0, 0.0, 200), "cannot hurt them");
        // The kills of one attack (borg_fight_attack::kill_fraction): a
        // finishing blow on a 1 HP mouse is a kill; a flask (7) on a 2 HP
        // mouse is one, not 3.5; a 2-damage whip on a 5 HP worm is 0.4.
        use crate::borg_fight_attack::kill_fraction;
        assert_eq!(kill_fraction(1, 1), 1.0);
        assert_eq!(kill_fraction(7, 2), 1.0);
        assert_eq!(kill_fraction(2, 5), 0.4);
        assert_eq!(kill_fraction(3, 0), 0.0, "no HP known");
        // Lost, but no staircase known: fight on; with one, flee.
        assert!(stand_against_breeders(false, false, false));
        assert!(stand_against_breeders(true, true, false), "nowhere to flee");
        assert!(!stand_against_breeders(true, false, false));
        assert!(!stand_against_breeders(false, true, true), "danger at or above the HP");
    }

    /// #556: the blocked grids are those within the radius (the engine's
    /// distance) of a mark, clipped to the map; none without a mark.
    #[test]
    fn grids_near_a_mark_are_blocked_within_the_radius() {
        let grid = |marks: &[(loc, BorgTime)], radius| {
            let mut b = vec![true; 3];
            breeder_block_grid(&mut b, marks, radius);
            b
        };
        assert!(grid(&[], 1).is_empty(), "the buffer is cleared");
        let at = |b: &[bool], x: i32, y: i32| b[(y * AUTO_MAX_X + x) as usize];
        let b = grid(&[(loc { x: 10, y: 10 }, 0)], 1);
        assert!(at(&b, 10, 10) && at(&b, 11, 11) && at(&b, 9, 10));
        assert!(!at(&b, 12, 10) && !at(&b, 10, 8));
        let b0 = grid(&[(loc { x: 10, y: 10 }, 0)], 0);
        assert!(at(&b0, 10, 10) && !at(&b0, 11, 10), "radius 0: only its own grid");
        // distance((10,10), (12,11)) = 2 + 1/2 = 2: inside radius 2
        let b2 = grid(&[(loc { x: 10, y: 10 }, 0)], 2);
        assert!(at(&b2, 12, 11) && !at(&b2, 12, 12), "the engine's distance, not the square");
        let corner = grid(&[(loc { x: 0, y: 0 }, 0)], 3);
        assert!(at(&corner, 0, 0) && at(&corner, 2, 1));
        let far = grid(&[(loc { x: AUTO_MAX_X - 1, y: AUTO_MAX_Y - 1 }, 0)], 2);
        assert!(at(&far, AUTO_MAX_X - 1, AUTO_MAX_Y - 1) && at(&far, AUTO_MAX_X - 3, AUTO_MAX_Y - 1));
    }

    /// #556: with the switches off (compiled 0) nothing is remembered,
    /// nothing blocks, and the stand and hunt decisions are #503's and #558's.
    #[test]
    fn with_the_switches_off_nothing_changes() {
        let mut st = BorgState::default();
        note_breeders(&mut st);
        note_breeders_stoppable(&mut st);
        assert!(st.borg.breeder_marks.is_empty() && st.borg.breeder_block.is_empty());
        assert!(!near_known_breeder(&st, 5, 5));
        assert_eq!(breeders_stoppable(&st), None);
        assert!(!st.borg.breeders_lost && !st.borg.breeders_in_danger);
        // A block outside a flight is never built; a set one is honoured.
        st.borg.breeder_block = vec![false; (AUTO_MAX_X * AUTO_MAX_Y) as usize];
        st.borg.breeder_block[(5 * AUTO_MAX_X + 5) as usize] = true;
        assert!(near_known_breeder(&st, 5, 5) && !near_known_breeder(&st, 6, 6));
        assert!(!near_known_breeder(&st, -1, 5) && !near_known_breeder(&st, AUTO_MAX_X, 5), "off the map");
    }
}

#[cfg(test)]
mod immobile_tests {
    use super::*;

    #[test]
    fn step_away_from_an_immobile_attacker_only_when_it_helps() {
        // #646: confused (a cure next to it is wasted) or with a ranged
        // attack (fire from out of reach); never when the monsters that
        // move are as dangerous as the HP left.
        assert!(step_away_wanted(true, 0, || 0, 10), "confused");
        assert!(step_away_wanted(false, 3, || 0, 10), "flasks or ammunition");
        assert!(!step_away_wanted(false, 0, || 0, 10), "neither: melee it as before");
        assert!(!step_away_wanted(true, 3, || 10, 10), "the movers are as dangerous as the HP left");
    }

    #[test]
    fn steps_away_are_bounded_per_grid_and_fail_closed() {
        let mut mon = crate::borg::BorgMonInfo::default();
        assert!(step_allowed(&mut mon, 100, (5, 7), false));
        assert!(!step_allowed(&mut mon, 100, (5, 7), false), "ranged: once per attacker");
        assert!(step_allowed(&mut mon, 100, (9, 9), true));
        assert!(step_allowed(&mut mon, 100, (9, 9), true));
        assert!(step_allowed(&mut mon, 100, (9, 9), true));
        assert!(!step_allowed(&mut mon, 100, (9, 9), true), "confused: three tries");
        assert!(step_allowed(&mut mon, 200, (5, 7), false), "a new level");
        // Second review of #659: the confused tries do not use up the
        // ranged step ("move away, drink cure, throw flask").
        let mut mixed = crate::borg::BorgMonInfo::default();
        for _ in 0..3 { assert!(step_allowed(&mut mixed, 100, (4, 4), true)); }
        assert!(!step_allowed(&mut mixed, 100, (4, 4), true));
        assert!(step_allowed(&mut mixed, 100, (4, 4), false), "the ranged step is still there");
        assert!(!step_allowed(&mut mixed, 100, (4, 4), false));
        // Review of #659: a full memory refuses, it does not step unbounded.
        let mut full = crate::borg::BorgMonInfo::default();
        for k in 0..16u8 { assert!(step_allowed(&mut full, 100, (k, 1), false)); }
        assert!(!step_allowed(&mut full, 100, (20, 1), false), "the 17th fails closed");
        assert!(!step_allowed(&mut full, 100, (20, 1), false));
    }
}

#[cfg(test)]
mod immobile_range_tests {
    use super::{annoying_effect, immobile_annoying};

    /// #713: which blows are annoying, and the modes.
    #[test]
    fn kept_at_range() {
        for e in [&b"POISON"[..], b"CONFUSE", b"PARALYZE", b"TERRIFY", b"EXP_10", b"EAT_LIGHT", b"ACID"] {
            assert!(annoying_effect(e), "{}", String::from_utf8_lossy(e));
        }
        assert!(!annoying_effect(b"HURT"), "plain damage");
        assert!(immobile_annoying(2, true, true), "the yellow jelly at 2");
        assert!(!immobile_annoying(2, true, false), "2: a plain-damage one is fought");
        assert!(immobile_annoying(3, true, false), "3: any immobile attacker");
        assert!(!immobile_annoying(3, false, true), "a monster that moves");
        assert!(!immobile_annoying(1, true, true), "1 is #646: none kept at range");
        assert!(!immobile_annoying(0, true, true), "0 is C");
    }
}

#[cfg(test)]
mod breeder_policy_tests {
    use super::{breeder_level_full, lost_recall};

    /// #722: the level is full above the class's count, from policy 3.
    #[test]
    fn full_level() {
        assert!(breeder_level_full(3, 8, 7), "a warrior with 8 known");
        assert!(!breeder_level_full(3, 7, 7), "7 a warrior still clears");
        assert!(!breeder_level_full(2, 20, 4), "below policy 3 no count loses");
        assert!(!breeder_level_full(-1, 20, 4), "the switches as set");
    }

    /// #722: on a lost level Recall waits for the search, then is due.
    #[test]
    fn recall_after_the_search() {
        assert_eq!(lost_recall(3, true, 1_000, 1_100, 500), (true, false), "searching");
        assert_eq!(lost_recall(3, true, 1_000, 1_500, 500), (false, true), "due");
        assert_eq!(lost_recall(3, true, 1_000, 1_000, 0), (false, true), "0: at once");
        assert_eq!(lost_recall(3, false, 0, 9_000, 500), (false, false), "not lost");
        assert_eq!(lost_recall(2, true, 1_000, 9_000, 500), (false, false), "below 3");
    }
}

#[cfg(test)]
mod breeder_pause_tests {
    use super::stand_paused;

    /// #722 lead b: a pause is a stoppable, unlost level in danger this think.
    #[test]
    fn pause() {
        assert!(stand_paused(true, false, true, true));
        assert!(!stand_paused(true, true, true, true), "a lost level is not a pause");
        assert!(!stand_paused(true, false, false, true), "no danger: standing");
        assert!(!stand_paused(false, false, true, true), "stoppable off: C");
        assert!(!stand_paused(true, false, true, false), "no breeders");
    }
}

// borg-reborn-rs (#781): an unseen occupant is an observation, not a race
// guessed from the engine. Only a queued movement plus melee feedback plus
// an unchanged position establishes its grid. See player-attack.c:738-743,
// :767, :840-860 (574b618c), mon-desc.h:45 and mon-desc.c:143-165.
#[derive(Clone, Default)]
pub struct UnseenBumps {
    step: Option<(loc, loc, bool)>,
    shot: Option<UnseenOilShot>,
    grids: Vec<UnseenBump>,
}

#[derive(Clone, Copy)]
struct UnseenOilShot {
    grid: loc,
    dead: bool,
    resolved: bool,
}

#[derive(Clone)]
struct UnseenBump {
    grid: loc,
    at: i32,
    throws: i32,
    detection_read: bool,
}

fn same_grid(a: loc, b: loc) -> bool { a.x == b.x && a.y == b.y }

fn bump_recent(now: i32, at: i32, turns: i32) -> bool {
    turns > 0 && now >= at && now - at < turns
}

pub(crate) fn unseen_bump_step(st: &mut BorgState, x: i32, y: i32) {
    let usable = p(Param::caution_unseen_bump_turns) > 0 && st.borg.tr(BI_CDEPTH) > 0
        && st.borg.tr(BI_ISCONFUSED) == 0 && st.borg.tr(BI_ISIMAGE) == 0
        && st.borg.tr(BI_ISBLIND) == 0;
    st.borg.unseen_bumps.step = usable.then_some((st.borg.c, loc { x, y }, false));
}

pub(crate) fn unseen_melee_message(msg: &[u8]) -> bool {
    // Every melee verb at 574b618c: player-attack.c plus brand.txt,
    // slay.txt and shape.txt. Ranged feedback starts "Your", not "You".
    const VERBS: &[&[u8]] = &[
        b"bite",
        b"burn",
        b"chill",
        b"claw",
        b"clout",
        b"corrode",
        b"crush",
        b"cuff",
        b"dissolve",
        b"fail to harm",
        b"fiercely smite",
        b"freeze",
        b"hit",
        b"miss",
        b"peck",
        b"poison",
        b"punch",
        b"scratch",
        b"shock",
        b"sicken",
        b"singe",
        b"slash",
        b"smite",
        b"strike",
        b"swipe",
        b"tear",
        b"thump",
        b"zap",
    ];
    let Some(rest) = msg.strip_prefix(b"You ") else { return false };
    VERBS.iter().any(|verb| rest.strip_prefix(*verb).is_some_and(|who| {
        [b" something".as_slice(), b" someone"].iter().any(|name| {
            who.strip_prefix(*name).is_some_and(|tail| tail.starts_with(b".") || tail.starts_with(b" ("))
        })
    }))
}

pub(crate) fn unseen_bump_message(st: &mut BorgState, msg: &[u8]) {
    if unseen_melee_message(msg) {
        if let Some((_, _, hit)) = &mut st.borg.unseen_bumps.step { *hit = true; }
    }
    // Oil is a single-target throw into the adjacent square (not an area
    // spell). Its unseen victim's death identifies the now-cleared grid.
    if [b"It dies.".as_slice(), b"Something dies.", b"Someone dies.", b"It is destroyed."].contains(&msg) {
        if let Some(shot) = &mut st.borg.unseen_bumps.shot { shot.dead = true; }
    }
    // Explicit throw completion: a miss must not carry its attribution
    // into another command. The pinned oil object always breaks.
    if msg == b"The Flask of oil breaks." {
        if let Some(shot) = &mut st.borg.unseen_bumps.shot { shot.resolved = true; }
    }
}

/// Consume the command's evidence once, after the observed position changed.
/// A successful (possibly confused) move cannot mark its intended grid.
fn finish_bump(bumps: &mut UnseenBumps, pos: loc, now: i32, turns: i32) -> Option<loc> {
    bumps.grids.retain(|b| bump_recent(now, b.at, turns) && !same_grid(b.grid, pos));
    if let Some(shot) = bumps.shot {
        if shot.dead { bumps.grids.retain(|b| !same_grid(b.grid, shot.grid)); }
        if shot.dead || shot.resolved || !bumps.grids.iter().any(|b| same_grid(b.grid, shot.grid)) {
            bumps.shot = None;
        }
    }
    let (from, grid, hit) = bumps.step.take()?;
    if turns == 0 || !hit || !same_grid(from, pos) { return None; }
    if let Some(b) = bumps.grids.iter_mut().find(|b| same_grid(b.grid, grid)) {
        b.at = now;
    } else {
        bumps.grids.push(UnseenBump { grid, at: now, throws: 0, detection_read: false });
    }
    Some(grid)
}

pub(crate) fn unseen_bump_update(st: &mut BorgState) {
    let turns = p(Param::caution_unseen_bump_turns);
    if let Some(shot) = st.borg.unseen_bumps.shot.filter(|s| s.dead) {
        borg_note(st, &format!("# Unseen bump cleared: oil killed occupant ({},{})", shot.grid.y, shot.grid.x));
    }
    if let Some(g) = finish_bump(&mut st.borg.unseen_bumps, st.borg.c, engine::game_turn(), turns) {
        st.borg.goal.type_0 = 0;
        borg_note(st, &format!("# Unseen bump: ({},{}) occupied for {turns} game turns", g.y, g.x));
    }
}

pub(crate) fn unseen_bump_blocked(st: &BorgState, x: i32, y: i32) -> bool {
    let turns = p(Param::caution_unseen_bump_turns);
    turns > 0 && (x, y) != (st.borg.c.x, st.borg.c.y)
        && st.borg.unseen_bumps.grids.iter().any(|b| b.grid.x == x && b.grid.y == y
            && bump_recent(engine::game_turn(), b.at, turns))
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
    if turns == 0 || st.borg.tr(BI_CDEPTH) == 0 { return false; }
    let now = engine::game_turn();
    let Some(i) = st.borg.unseen_bumps.grids.iter().position(|b| {
        bump_recent(now, b.at, turns) && engine::distance(st.borg.c, b.grid) == 1
            && st.cave.grid(b.grid.x, b.grid.y).kill == 0
    }) else { return false };
    if bump_heal_due(st.borg.tr(BI_CURHP), st.borg.tr(BI_MAXHP), p(Param::caution_unseen_bump_heal_hp_percent))
        && (borg_quaff_potion(st, st.item_val.sv_potion_cure_light)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_serious)
            || borg_quaff_potion(st, st.item_val.sv_potion_cure_critical))
    {
        borg_note(st, "# Healing before stairs: an unseen monster blocks the step");
        return true;
    }
    if p(Param::caution_unseen_bump_response) == 0 { return false; }
    if !st.borg.unseen_bumps.grids[i].detection_read
        && crate::borg_fight_defend::unseen_detection_safe(st, danger)
        && borg_read_scroll(st, st.item_val.sv_scroll_detect_invis)
    {
        st.borg.unseen_bumps.grids[i].detection_read = true;
        borg_note(st, "# Detect Invisible: an unseen monster blocks the step");
        return true;
    }
    let g = st.borg.unseen_bumps.grids[i].grid;
    if st.borg.tr(BI_ISCONFUSED) == 0 && st.borg.tr(BI_ISBLIND) == 0 && st.borg.tr(BI_ISIMAGE) == 0
        && st.borg.unseen_bumps.grids[i].throws < p(Param::caution_unseen_bump_throw_tries)
        && crate::borg_fight_attack::throw_at_unseen_bump(st, g)
    {
        st.borg.unseen_bumps.grids[i].throws += 1;
        st.borg.unseen_bumps.shot = Some(UnseenOilShot { grid: g, dead: false, resolved: false });
        return true;
    }
    // The ordinary adjacent-monster rule cannot rate a monster with no
    // known race. This site's explicit, bounded bump evidence is its gate.
    if crate::borg_escape::unseen_bump_phase(st, danger) {
        st.borg.escapes = st.borg.escapes.wrapping_add(1);
        st.borg.goal.type_0 = 0;
        return true;
    }
    false
}

#[cfg(test)]
mod unseen_bump_tests {
    use super::*;
    fn pos(x: i32, y: i32) -> loc { loc { x, y } }
    #[test]
    fn only_unseen_player_melee_is_evidence() {
        for msg in [b"You fail to harm something.".as_slice(), b"You miss someone.", b"You hit something (3)."] {
            assert!(unseen_melee_message(msg));
        }
        for msg in [b"Something touches you.".as_slice(), b"You hit the kobold.", b"You feel something give way!", b"You hit someone's sword."] {
            assert!(!unseen_melee_message(msg));
        }
    }
    #[test]
    fn failed_step_is_consumed_and_expires_without_renewing_itself() {
        let mut bumps = UnseenBumps::default();
        bumps.step = Some((pos(40,35), pos(40,34), true));
        assert!(finish_bump(&mut bumps, pos(40,35), 36669, 100).is_some());
        assert_eq!(bumps.grids.len(), 1);
        assert!(finish_bump(&mut bumps, pos(40,35), 36768, 100).is_none());
        assert_eq!(bumps.grids.len(), 1);
        finish_bump(&mut bumps, pos(40,35), 36769, 100);
        assert!(bumps.grids.is_empty());
    }
    #[test]
    fn successful_movement_no_feedback_and_switch_off_never_mark_a_grid() {
        for (pos, hit, turns) in [(pos(40,34), true, 100), (pos(40,35), false, 100), (pos(40,35), true, 0)] {
            let mut bumps = UnseenBumps::default();
            bumps.step = Some((self::pos(40,35), self::pos(40,34), hit));
            assert!(finish_bump(&mut bumps, pos, 36669, turns).is_none());
            assert!(bumps.grids.is_empty());
            assert!(bumps.step.is_none());
        }
    }
    #[test]
    fn revisiting_a_grid_clears_it_and_repeated_bumps_keep_the_response_budget() {
        let mut bumps = UnseenBumps::default();
        bumps.step = Some((pos(40,35), pos(40,34), true));
        finish_bump(&mut bumps, pos(40,35), 100, 100);
        bumps.grids[0].throws = 3;
        bumps.grids[0].detection_read = true;
        bumps.step = Some((pos(40,35), pos(40,34), true));
        finish_bump(&mut bumps, pos(40,35), 110, 100);
        assert_eq!(bumps.grids[0].throws, 3);
        assert!(bumps.grids[0].detection_read);
        finish_bump(&mut bumps, pos(40,34), 120, 100);
        assert!(bumps.grids.is_empty());
    }
    #[test]
    fn emergency_cure_has_a_strict_configured_hp_boundary() {
        assert!(bump_heal_due(21, 44, 50));
        assert!(!bump_heal_due(22, 44, 50));
        assert!(!bump_heal_due(1, 44, 0));
        assert!(!bump_recent(99, 100, 100));
    }
}

#[cfg(test)]
mod unseen_bump_kill_tests {
    use super::*;
    #[test]
    fn a_directional_oil_kill_clears_only_its_grid() {
        let mut st = BorgState::default();
        st.borg.unseen_bumps.grids = vec![
            UnseenBump { grid: loc { x: 40, y: 34 }, at: 100, throws: 1, detection_read: false },
            UnseenBump { grid: loc { x: 41, y: 35 }, at: 100, throws: 0, detection_read: false },
        ];
        // An unassociated death cannot clear an inferred occupant.
        unseen_bump_message(&mut st, b"It dies.");
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 110, 100);
        assert_eq!(st.borg.unseen_bumps.grids.len(), 2);
        st.borg.unseen_bumps.shot = Some(UnseenOilShot { grid: loc { x: 40, y: 34 }, dead: false, resolved: false });
        unseen_bump_message(&mut st, b"It dies.");
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 120, 100);
        assert_eq!(st.borg.unseen_bumps.grids.len(), 1);
        assert_eq!(st.borg.unseen_bumps.grids[0].grid.x, 41);
        assert!(st.borg.unseen_bumps.shot.is_none());
    }
}


#[cfg(test)]
mod unseen_melee_vocabulary_tests {
    use super::unseen_melee_message;
    #[test]
    fn all_engine_brand_slay_and_shape_melee_verbs_are_recognized() {
        for (data, prefix) in [
            (include_str!("../../engine/angband/lib/gamedata/brand.txt"), "verb:"),
            (include_str!("../../engine/angband/lib/gamedata/slay.txt"), "melee-verb:"),
            (include_str!("../../engine/angband/lib/gamedata/shape.txt"), "blow:"),
        ] {
            for verb in data.lines().filter_map(|line| line.strip_prefix(prefix)) {
                assert!(unseen_melee_message(format!("You {verb} something.").as_bytes()), "{verb}");
                assert!(!unseen_melee_message(format!("Your weapon {verb} something.").as_bytes()));
            }
        }
    }
}

#[cfg(test)]
mod unseen_oil_lifecycle_tests {
    use super::*;
    fn pending() -> BorgState {
        let mut st = BorgState::default();
        let grid = loc { x: 40, y: 34 };
        st.borg.unseen_bumps.grids.push(UnseenBump { grid, at: 100, throws: 1, detection_read: false });
        st.borg.unseen_bumps.shot = Some(UnseenOilShot { grid, dead: false, resolved: false });
        st
    }
    #[test]
    fn pending_shot_survives_an_intermediate_update_until_feedback() {
        let mut st = pending();
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 110, 100);
        assert!(st.borg.unseen_bumps.shot.is_some());
        unseen_bump_message(&mut st, b"It dies.");
        unseen_bump_message(&mut st, b"The Flask of oil breaks.");
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 120, 100);
        assert!(st.borg.unseen_bumps.grids.is_empty());
        assert!(st.borg.unseen_bumps.shot.is_none());
    }
    #[test]
    fn a_resolved_miss_cannot_claim_a_later_unrelated_death() {
        let mut st = pending();
        unseen_bump_message(&mut st, b"The Flask of oil breaks.");
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 110, 100);
        assert!(st.borg.unseen_bumps.shot.is_none());
        unseen_bump_message(&mut st, b"It dies.");
        finish_bump(&mut st.borg.unseen_bumps, loc { x: 40, y: 35 }, 120, 100);
        assert_eq!(st.borg.unseen_bumps.grids.len(), 1);
    }
    #[test]
    fn pending_shots_cannot_outlive_their_grid_or_a_cancelled_command() {
        for (pos, now, turns) in [(loc { x: 40, y: 34 }, 110, 100),
            (loc { x: 40, y: 35 }, 200, 100), (loc { x: 40, y: 35 }, 110, 0)] {
            let mut st = pending();
            finish_bump(&mut st.borg.unseen_bumps, pos, now, turns);
            assert!(st.borg.unseen_bumps.shot.is_none());
        }
        let mut st = pending();
        crate::borg_io::borg_flush(&mut st);
        assert!(st.borg.unseen_bumps.shot.is_none());
    }
}

/// Cancelled input cannot later attribute another command's death to oil.
pub(crate) fn unseen_bump_cancel_shot(st: &mut BorgState) {
    st.borg.unseen_bumps.shot = None;
}

/// #811: apply self-care before routing in darkness. This cannot start a flow.
pub(crate) fn lightless_self_care(st: &mut BorgState, danger: i32) -> bool {
    // Cure known poison before paying for the walk with HP, even when the
    // ordinary fight rule would wait for lower HP.
    (st.borg.tr(BI_ISPOISONED) != 0
        && danger <= i32::from(st.borg.avoidance) * p(Param::caution_cure_safe_danger_tenths) / 10
        && borg_quaff_potion(st, st.item_val.sv_potion_cure_poison))
        || borg_heal(st, danger)
        || (st.borg.tr(BI_ISHUNGRY) != 0
            && crate::borg_item_use::borg_eat_when_hungry(st))
        || (st.borg.tr(BI_ISWEAK) != 0 && borg_eat_food_any(st))
}

#[cfg(test)]
mod confused_step_tests {
    use super::{step_allowed, step_left};

    /// #837: the peek agrees with the recording check, for the confused
    /// tries (three) and the ranged step (one), on a new level and a full memory.
    #[test]
    fn peek_matches_the_recorded_step() {
        let mut mon = crate::borg::BorgMonInfo::default();
        let pos = (10u8, 20u8);
        for confused in [true, false] {
            let level = if confused { 1 } else { 2 };
            let limit = if confused { 3 } else { 1 };
            for tried in 0..=limit {
                let peek = step_left(&mon, level, pos, confused);
                assert_eq!(peek, tried < limit, "confused {confused}, {tried} used");
                // `step_allowed` agrees, and only it records.
                assert_eq!(step_allowed(&mut mon, level, pos, confused), peek);
            }
        }
    }

    /// #837: `caution_immobile_no_return` ships 0: no grid is ever in the reach
    /// of an attacker the Borg stepped away from, however many it recorded.
    #[test]
    fn no_return_is_off_by_default() {
        let mut st = crate::state::BorgState::default();
        st.borg.mon.immobile_stepped_level = st.borg.time.level;
        st.borg.mon.immobile_stepped[0] = [10, 20, 1, 0];
        assert!(!super::in_reach_of_stepped_from(&st, 20, 10));
        assert!(!super::in_reach_of_stepped_from(&st, 21, 11));
    }

    /// A full memory (sixteen attackers) refuses a new one, as `step_allowed` does.
    #[test]
    fn full_memory_refuses_a_new_attacker() {
        let mut mon = crate::borg::BorgMonInfo::default();
        for k in 0..16u8 {
            assert!(step_allowed(&mut mon, 1, (k, k), true));
        }
        assert!(!step_left(&mon, 1, (99, 99), true));
        assert!(!step_allowed(&mut mon, 1, (99, 99), true));
        assert!(step_left(&mon, 1, (3, 3), true), "a known attacker keeps its tries");
        assert!(step_left(&mon, 2, (99, 99), true), "a new level forgets them");
    }
}

/// borg-reborn-rs (#984): with `caution_recall_dl1_hold_up`, a Word of
/// Recall counting down on DL1 holds caution's "Take stairs up" (the `if`
/// that calls this): the staircase leads to town, where the Recall fires and
/// takes the Borg back to its maximum depth (`game-world.c:707-731`), 15 to
/// 34 world ticks after the reading (`effect-handler-general.c:1145`).
/// 312081895 read its last scroll on DL1, took the stairs, had 42 game turns
/// in town and was pulled to DL2 with no Recall bought. Held, the Recall
/// fires on DL1 and the Borg reaches town with nothing pending. A danger at
/// least the avoidance still takes the stairs, and the danger escapes
/// (`borg_escape_stair`) are not held. 0 = C (`borg-caution.c:1171`).
pub(crate) fn recall_holds_up_stair(st: &BorgState, pos_danger: i32) -> bool {
    recall_holds_up_with(
        p(Param::caution_recall_dl1_hold_up) != 0,
        st.borg.goal.recalling,
        st.borg.tr(BI_CDEPTH),
        pos_danger,
        i32::from(st.borg.avoidance),
    )
}

fn recall_holds_up_with(on: bool, recalling: i32, depth: i32, danger: i32, avoidance: i32) -> bool {
    on && recalling != 0 && depth == 1 && danger < avoidance
}

#[cfg(test)]
mod recall_dl1_hold_up_tests {
    use super::recall_holds_up_with;

    #[test]
    fn a_pending_recall_holds_the_dl1_up_staircase_only_with_the_switch() {
        // 312081895 at 10948: Recall pending on DL1, danger 0, avoidance 40.
        assert!(!recall_holds_up_with(false, 140, 1, 0, 40), "C: the stairs are taken");
        assert!(recall_holds_up_with(true, 140, 1, 0, 40), "the switch: held, the Recall fires on DL1");
        assert!(!recall_holds_up_with(true, 140, 1, 40, 40), "a danger at the avoidance still takes them");
        assert!(!recall_holds_up_with(true, 140, 2, 0, 40), "DL2: the staircase leads to DL1, not town");
        assert!(!recall_holds_up_with(true, 0, 1, 0, 40), "no Recall pending");
    }
}

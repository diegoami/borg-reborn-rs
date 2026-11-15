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
use crate::borg_cave_util::{borg_cave_floor_bold, borg_cave_floor_grid};
use crate::borg_danger::borg_danger;
use crate::borg_escape::{
    borg_caution_phase, borg_escape, borg_freedom, borg_recall, borg_surrounded,
};
use crate::borg_fight_defend::borg_defend;
use crate::borg_flow_stairs::{
    borg_flow_stair_less, borg_flow_stair_more, borg_prep_leave_level_spells,
};
use crate::borg_headers::borg_h::BORG_PLAYS_RISKY;
use crate::borg_headers::borg_magic_h::{
    BERSERK_STRENGTH, CURE_POISON, HEALING, HERBAL_CURING, HEROISM, HOLY_WORD, MINOR_HEALING,
    PHASE_DOOR, PORTAL, REMOVE_HUNGER, TELEPORT_LEVEL,
};
use crate::borg_headers::borg_trait_h::{
    BI_ACCW, BI_AESCAPE, BI_AEZHEAL, BI_AFUEL, BI_AHEAL, BI_AMISSILES, BI_ATELEPORT, BI_CDEPTH,
    BI_CLASS, BI_CLEVEL, BI_CRSFEAR, BI_CURHP, BI_CURSP, BI_ESP, BI_FOOD, BI_GOLD, BI_IFIRE,
    BI_ISAFRAID, BI_ISBLIND, BI_ISCONFUSED, BI_ISCUT, BI_ISFIXCON, BI_ISFIXDEX, BI_ISFIXINT,
    BI_ISFIXSTR, BI_ISFIXWIS, BI_ISHEAVYSTUN, BI_ISHUNGRY, BI_ISIMAGE, BI_ISPOISONED, BI_ISSTUN,
    BI_ISWEAK, BI_KING, BI_LIGHT, BI_MAXCLEVEL, BI_MAXDEPTH, BI_MAXHP, BI_MAXSP, BI_SPEED,
};
use crate::borg_inventory::borg_slot;
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, KC_ENTER};
use crate::borg_item_use::{
    borg_activate_failure, borg_activate_item, borg_eat, borg_eat_food_any, borg_eat_unknown,
    borg_equips_staff_fail, borg_quaff_crit, borg_quaff_potion, borg_quaff_unknown,
    borg_read_scroll, borg_read_unknown, borg_use_staff, borg_use_staff_fail, borg_use_unknown,
    borg_zap_rod,
};
use crate::borg_item_val::borg_lookup_kind;
use crate::borg_item_wear::borg_backup_swap;
use crate::borg_light::{borg_maintain_light, BorgNeed};
use crate::borg_magic::{borg_spell, borg_spell_fail, borg_spell_legal};
use crate::borg_store_sell::borg_count_sell;
use crate::engine;
use crate::headers::list_options_h::OPT_birth_force_descend;
use crate::headers::list_player_timed_h::TMD_OPP_ACID;
use crate::headers::list_terrain_h::{FEAT_LAVA, FEAT_LESS, FEAT_MORE};
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

    // borg-reborn-rs (#350): known ordinary cures are survival resources,
    // including while fleeing. At useful wound sizes do not randomly skip
    // them, and cap their benefit at max HP when checking the next round.
    if danger > 0
        && (danger >= hp_down || danger >= st.borg.tr(BI_CURHP))
        && (pct_down >= p(Param::caution_clw_min_loss)
            || danger >= st.borg.tr(BI_CURHP))
    {
        for (sval, amount, divisor) in [
            (st.item_val.sv_potion_cure_light, clw_heal, p(Param::caution_clw_danger_divisor)),
            (st.item_val.sv_potion_cure_serious, csw_heal, p(Param::caution_csw_danger_divisor)),
            (st.item_val.sv_potion_cure_critical, ccw_heal, p(Param::caution_ccw_danger_divisor)),
        ] {
            let gain = amount.min(hp_down);
            if danger < st.borg.tr(BI_CURHP) + gain
                && gain > danger / divisor
                && borg_quaff_potion(st, sval)
            {
                borg_note(st, "# Healing before further movement or combat.");
                return true;
            }
        }
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

        // Quaff unknown potions / eat unknown mushrooms / use unknown
        // staffs in this emergency.  We might get lucky.
        if borg_quaff_unknown(st) || borg_eat_unknown(st) || borg_use_unknown(st) {
            return true;
        }
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

        // Quaff unknown potions / eat unknown mushrooms / use unknown
        // staffs in this emergency.  We might get lucky.
        if borg_quaff_unknown(st) || borg_eat_unknown(st) || borg_use_unknown(st) {
            return true;
        }
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
    b_m <= p(Param::caution_desperate_adjacent_down_distance)
        && b_m != -1
        && desperate_flow(st, false, "# Desperate for Stairs (four)")
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
    ((danger_test && !nasty && st.borg.no_retreat == 0) || (borg_surround && pos_danger != 0))
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

        // Start leaving
        start_leaving(st, "# Leaving (Scary guy on level)");

        // Start fleeing
        start_fleeing(st, "# Fleeing (Scary guy on level)");

        // Return to town quickly after leaving town
        if st.borg.tr(BI_CDEPTH) == 0 {
            st.borg.goal.fleeing_to_town = true;
        }
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

    // borg-reborn-rs (#350): use a viable short escape when the current
    // retreat is surrounded, rather than healing into repeated bump fights.
    if pos_danger > 0
        && (borg_surround || pos_danger >= st.borg.tr(BI_CURHP))
        && crate::borg_escape::borg_escape_trapped(st)
    {
        return true;
    }

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
        // borg-reborn-rs (#350): healing can make a lethal grid survivable.
        // Do not reject the attempt merely because current HP is too low.
        // borg_heal checks whether the actual cure buys enough HP; the
        // flee goal remains set and escape remains available afterward.
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
        if borg_eat_food_any(st) || borg_spell(st, REMOVE_HUNGER) || borg_spell(st, HERBAL_CURING) {
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
    if st.borg.near.breeder && st.borg.tr(BI_CLEVEL) < p(Param::caution_breeder_flee_max_level) {
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

    // Strategic retreat
    //
    // Do not retreat if
    // 1) we are icky (poisoned, blind, confused etc
    // 2) we have boosted our avoidance because we are stuck
    // 3) we are in a Sea of Runes
    // 4) we are not in a vault
    let danger_test =
        pos_danger > i32::from(st.borg.avoidance) / p(Param::caution_retreat_danger_divisor);
    if may_move_away(st, danger_test, nasty, borg_surround, pos_danger, true)
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
    if may_move_away(st, danger_test, nasty, borg_surround, pos_danger, false)
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
    if st.borg.goal.fleeing
        && !st.borg.goal.fleeing_to_town
        && st.borg.tr(BI_CDEPTH) >= 1
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

    // if I am gonna die next round, and I have no way to escape
    // use the unknown stuff (if I am low level).
    if pos_danger > st.borg.tr(BI_CURHP) * p(Param::caution_unknown_danger_multiplier)
        && st.borg.tr(BI_CLEVEL) < p(Param::caution_unknown_max_level)
        && st.borg.tr(BI_MAXSP) == 0
        && (borg_use_unknown(st)
            || borg_read_unknown(st)
            || borg_quaff_unknown(st)
            || borg_eat_unknown(st))
    {
        return true;
    }

    // Nothing
    false
}

/// `borg_note(format("<prefix>%s", reason))` for a `borg_restock()` reason.
/// A NULL reason prints as empty in Angband's `format()` (it cannot be NULL
/// here: the caller has just seen a non-NULL one for the same depth).
fn note_with_reason(st: &mut BorgState, prefix: &[u8], reason: Option<Vec<u8>>) {
    let mut note = prefix.to_vec();
    note.extend_from_slice(reason.as_deref().unwrap_or(b""));
    borg_note_bytes(st, &note);
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

    if borg_caution_phase(
        st,
        p(Param::caution_recall_phase_tolerance),
        p(Param::caution_recall_phase_horizon),
    ) && (borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
        || borg_spell_fail(st, PHASE_DOOR, p(Param::caution_recall_phase_fail))
        || borg_spell_fail(st, PORTAL, p(Param::caution_recall_portal_fail))
        || borg_activate_item(st, st.act.tele_phase))
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

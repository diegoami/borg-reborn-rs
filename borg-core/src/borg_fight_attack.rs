//! Port of `borg-fight-attack.c`: find the best attack.
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
//!
//! `borg_attack` collects the monsters in reach, simulates every attack
//! type (`borg_calculate_attack_effectiveness` with `borg_simulate` set),
//! and performs the best one. Most attacks go through `borg_launch_bolt`,
//! which scores bolts, beams, balls and "dispels" at every target (and the
//! grids next to it), or `borg_launch_arc` for arcs and cones.

use crate::borg::BorgTime;
use crate::borg_cave::{BorgGrid, DUNGEON_HGT, DUNGEON_WID};
use crate::borg_cave_util::{borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid};
use crate::borg_danger::{borg_danger, borg_danger_one_kill};
use crate::borg_flow_kill::borg_race_name;
use crate::borg_flow_misc::borg_extract_dir;
use crate::borg_inventory::{borg_slot, borg_wield_slot};
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, borg_queue_direction};
use crate::borg_item::{inven_wield, quiver_end, quiver_start};
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_item_use::{
    borg_activate_dragon, borg_activate_failure, borg_activate_item, borg_activate_ring,
    borg_aim_wand, borg_equips_dragon, borg_equips_item, borg_equips_ring, borg_equips_rod,
    borg_equips_staff_fail, borg_use_staff, borg_zap_rod,
};
use crate::borg_magic::{
    borg_get_spell_power, borg_spell, borg_spell_fail, borg_spell_legal, borg_spell_legal_fail,
    borg_spell_okay, borg_spell_okay_fail,
};
use crate::borg_projection::{
    borg_distance, borg_inc_motion, borg_los, borg_offset_projectable, borg_projectable_pure,
    borg_target,
};
use crate::borg_update::Detect;
use crate::engine;
use crate::borg_headers::borg_danger_h::*;
use crate::borg_headers::borg_fight_attack_h::*;
use crate::borg_headers::borg_item_h::borg_item;
use crate::borg_headers::borg_magic_h::*;
use crate::borg_headers::borg_trait_h::*;
use crate::headers::list_mon_race_flags_h::*;
use crate::headers::list_mon_spells_h::*;
use crate::headers::list_options_h::OPT_birth_randarts;
use crate::headers::list_stats_h::{STAT_CON, STAT_DEX, STAT_INT, STAT_STR, STAT_WIS};
use crate::headers::list_terrain_h::{
    FEAT_BROKEN, FEAT_FLOOR, FEAT_MAGMA, FEAT_NONE, FEAT_OPEN, FEAT_PASS_RUBBLE, FEAT_PERM,
};
use crate::headers::list_tvals_h::{
    TV_BOOTS, TV_FLASK, TV_POTION, TV_RING, TV_ROD, TV_WAND,
};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// The module's globals (borg-fight-attack.c).
pub struct FightAttackState {
    /// `successful_target`: zero is "haven't shot", positive is "good to
    /// shoot current target", negative is "shot and missed", -12 and lower
    /// is "do not shoot".
    pub successful_target: i32,
    /// `target_closest`
    pub target_closest: i32,
    /// `borg_tp_other_n`: the monsters in the line of a Teleport Other
    pub tp_other_n: i16,
    /// `borg_tp_other_x`
    pub tp_other_x: [u8; 255],
    /// `borg_tp_other_y`
    pub tp_other_y: [u8; 255],
    /// `borg_tp_other_index`
    pub tp_other_index: [i32; 255],
}

impl Default for FightAttackState {
    fn default() -> Self {
        FightAttackState {
            successful_target: 0,
            target_closest: 0,
            tp_other_n: 0,
            tp_other_x: [0; 255],
            tp_other_y: [0; 255],
            tp_other_index: [0; 255],
        }
    }
}

/// `BTH_PLUS_ADJ` (player-calcs.h)
const BTH_PLUS_ADJ: i32 = 3;
/// `GOAL_KILL` (borg.h)
const GOAL_KILL: i16 = 1;
/// `BORG_OKAY` (borg-cave.h grid info bit).
const BORG_OKAY: u16 = 0x08;
/// `BORG_VIEW` (borg-cave.h grid info bit).
const BORG_VIEW: u16 = 0x20;

/* Player classes (`CLASS_*`, class.txt order). */
const CLASS_MAGE: i32 = 1;
const CLASS_DRUID: i32 = 2;
const CLASS_PRIEST: i32 = 3;
const CLASS_NECROMANCER: i32 = 4;
const CLASS_PALADIN: i32 = 5;
const CLASS_ROGUE: i32 = 6;
const CLASS_RANGER: i32 = 7;

/// A literal key (`borg_keypress('5')` etc.).
fn key(c: u8) -> keycode_t {
    keycode_t::from(c)
}

/// `I2D(dir)`: the key of a direction.
fn i2d(dir: i32) -> keycode_t {
    (dir + i32::from(b'0')) as keycode_t
}

/// `borg.trait[i]`
fn tr(st: &BorgState, i: u32) -> i32 {
    st.borg.tr(i)
}

/// `borg.avoidance` (promoted to `int`).
fn avoid(st: &BorgState) -> i32 {
    i32::from(st.borg.avoidance)
}

/// `borg_timer(t)`: game turns since `t`.
fn borg_timer(st: &BorgState, t: BorgTime) -> BorgTime {
    st.borg.time.now.wrapping_sub(t)
}

/// `borg_simulate` (borg-trait.c)
fn simulating(st: &mut BorgState) -> bool {
    st.traits.simulate
}

/// A copy of `borg_grids[y][x]`.
fn grid(st: &BorgState, x: i32, y: i32) -> BorgGrid {
    *st.cave.grid(x, y)
}

/// `z_info->max_range`
fn max_range() -> i32 {
    i32::from(engine::z_info().max_range)
}

/// `kill->r_idx >= z_info->r_max - 1` ("player ghosts").
fn is_ghost(r_idx: u16) -> bool {
    i32::from(r_idx) >= i32::from(engine::z_info().r_max) - 1
}

/// `rf_has(r_info[r_idx].flags, flag)`
fn rf(r_idx: u16, flag: u32) -> bool {
    engine::race_flag_has(usize::from(r_idx), flag)
}

/// `rsf_has(r_info[r_idx].spell_flags, flag)`
fn rsf(r_idx: u16, flag: u32) -> bool {
    engine::flag_has(&engine::race(usize::from(r_idx)).spell_flags, flag as i32)
}

/// `r_ptr->friends || r_ptr->friends_base`
fn has_friends(r_idx: u16) -> bool {
    let r = engine::race(usize::from(r_idx));
    !r.friends.is_null() || !r.friends_base.is_null()
}

/// `strstr(hay, needle) != NULL` on C strings.
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    needle.is_empty() || hay.windows(needle.len()).any(|w| w == needle)
}

/// The bytes of an item's `desc` (up to its NUL, `%s`).
fn desc(item: &borg_item) -> Vec<u8> {
    item.desc.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
}

/// "No firing while blind, confused, or hallucinating"
fn cant_aim(st: &BorgState) -> bool {
    tr(st, BI_ISBLIND) != 0 || tr(st, BI_ISCONFUSED) != 0 || tr(st, BI_ISIMAGE) != 0
}

/// "Paranoia": `borg_simulate && randint0(100) < pct` (the RNG is drawn
/// only while simulating).
fn paranoia(st: &mut BorgState, pct: i32) -> bool {
    simulating(st) && engine::randint0(100) < pct
}

/// "Not if low on food": no food, weak and able to cure it by magic.
fn low_on_food(st: &BorgState) -> bool {
    tr(st, BI_FOOD) == 0
        && (tr(st, BI_ISWEAK) != 0
            && (borg_spell_legal(st, REMOVE_HUNGER) || borg_spell_legal(st, HERBAL_CURING)))
}

/// `borg_danger_one_kill(borg.c.y, borg.c.x, c, i, true, full)`
fn danger_one_here(st: &mut BorgState, c: i32, i: i32, full_damage: bool) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger_one_kill(st, y, x, c, i, true, full_damage)
}

/// `borg_danger(borg.c.y, borg.c.x, c, average, false)`
fn danger_here(st: &mut BorgState, c: i32, average: bool) -> i32 {
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    borg_danger(st, y, x, c, average, false)
}

/// `distance(borg.c, loc(x, y))`
fn dist_to(st: &BorgState, x: i32, y: i32) -> i32 {
    engine::distance(st.borg.c, loc { x, y })
}

/// The "power" of a monster, promoted to `int`.
fn power(st: &BorgState, i: usize) -> i32 {
    i32::from(st.kills.kills[i].power)
}

/// Record monster `i` in the Teleport Other list (`borg_tp_other_*`).
/// Panics past the 255 entries, where C writes past the arrays.
fn tp_other_add(st: &mut BorgState, i: i32) {
    let pos = st.kills.kills[i as usize].pos;
    let fa = &mut st.fight_attack;
    let n = fa.tp_other_n as usize;
    fa.tp_other_index[n] = i;
    fa.tp_other_y[n] = pos.y as u8;
    fa.tp_other_x[n] = pos.x as u8;
    fa.tp_other_n = fa.tp_other_n.wrapping_add(1);
}

/// The summoner test (`rsf_has(.., RSF_S_*) || ...`); `hi_dragon` adds
/// `RSF_S_HI_DRAGON`, which the melee version leaves out.
fn is_summoner(r_idx: u16, hi_dragon: bool) -> bool {
    rsf(r_idx, RSF_S_KIN)
        || rsf(r_idx, RSF_S_HI_DEMON)
        || rsf(r_idx, RSF_S_MONSTER)
        || rsf(r_idx, RSF_S_MONSTERS)
        || rsf(r_idx, RSF_S_ANIMAL)
        || rsf(r_idx, RSF_S_SPIDER)
        || rsf(r_idx, RSF_S_HOUND)
        || rsf(r_idx, RSF_S_HYDRA)
        || rsf(r_idx, RSF_S_AINU)
        || rsf(r_idx, RSF_S_DEMON)
        || rsf(r_idx, RSF_S_UNDEAD)
        || rsf(r_idx, RSF_S_DRAGON)
        || (hi_dragon && rsf(r_idx, RSF_S_HI_DRAGON))
        || rsf(r_idx, RSF_S_HI_UNDEAD)
        || rsf(r_idx, RSF_S_WRAITH)
        || rsf(r_idx, RSF_S_UNIQUE)
}

/// The monster-blow effect names and values (`monblow[]`), with C's NULL
/// end entry.
const MONBLOW: [(Option<&[u8]>, u32); 31] = [
    (Some(b"NONE"), MONBLOW_NONE),
    (Some(b"HURT"), MONBLOW_HURT),
    (Some(b"POISON"), MONBLOW_POISON),
    (Some(b"DISENCHANT"), MONBLOW_DISENCHANT),
    (Some(b"DRAIN_CHARGES"), MONBLOW_DRAIN_CHARGES),
    (Some(b"EAT_GOLD"), MONBLOW_EAT_GOLD),
    (Some(b"EAT_ITEM"), MONBLOW_EAT_ITEM),
    (Some(b"EAT_FOOD"), MONBLOW_EAT_FOOD),
    (Some(b"EAT_LIGHT"), MONBLOW_EAT_LIGHT),
    (Some(b"ACID"), MONBLOW_ACID),
    (Some(b"ELEC"), MONBLOW_ELEC),
    (Some(b"FIRE"), MONBLOW_FIRE),
    (Some(b"COLD"), MONBLOW_COLD),
    (Some(b"BLIND"), MONBLOW_BLIND),
    (Some(b"CONFUSE"), MONBLOW_CONFUSE),
    (Some(b"TERRIFY"), MONBLOW_TERRIFY),
    (Some(b"PARALYZE"), MONBLOW_PARALYZE),
    (Some(b"LOSE_STR"), MONBLOW_LOSE_STR),
    (Some(b"LOSE_INT"), MONBLOW_LOSE_INT),
    (Some(b"LOSE_WIS"), MONBLOW_LOSE_WIS),
    (Some(b"LOSE_DEX"), MONBLOW_LOSE_DEX),
    (Some(b"LOSE_CON"), MONBLOW_LOSE_CON),
    (Some(b"LOSE_ALL"), MONBLOW_LOSE_ALL),
    (Some(b"SHATTER"), MONBLOW_SHATTER),
    (Some(b"EXP_10"), MONBLOW_EXP_10),
    (Some(b"EXP_20"), MONBLOW_EXP_20),
    (Some(b"EXP_40"), MONBLOW_EXP_40),
    (Some(b"EXP_80"), MONBLOW_EXP_80),
    (Some(b"HALLU"), MONBLOW_HALLU),
    (Some(b"BLACK_BREATH"), MONBLOW_BLACK_BREATH),
    (None, MONBLOW_NONE),
];

/// What effect does a blow from a monster have? (`borg_mon_blow_effect`)
///
/// *HACK* must match code in mon-blows.c. The search stops before the
/// NULL end entry, so an unknown name gives `MONBLOW_NONE`.
pub fn borg_mon_blow_effect(name: &[u8]) -> i32 {
    for (n, val) in &MONBLOW[..MONBLOW.len() - 1] {
        let n = n.expect("borg_mon_blow_effect: only the end entry is NULL");
        if name == n {
            return *val as i32;
        }
    }
    MONBLOW_NONE as i32
}

/// Guess how much damage a physical attack will do to a monster
/// (`borg_thrust_damage_one`).
///
/// The C consumer made this non-static for its improvement unit I-001
/// (better-borg-c `4.2.6-impact-v10`): `borg_caution()`'s
/// `borg_adjacent_kill_in_one_blow()` needs the melee damage of a full
/// round against a tracked kill.
pub fn borg_thrust_damage_one(st: &BorgState, i: i32) -> i32 {
    /* Examine current weapon */
    let item = &st.items.items[inven_wield() as usize];

    /* Monster record */
    let kill = &st.kills.kills[i as usize];

    /* Monster race */
    let r = kill.r_idx;

    /* "player ghosts" */
    if is_ghost(r) {
        return 0;
    }
    let r_ptr = engine::race(usize::from(r));

    /* Damage */
    let mut dam = i32::from(item.dd) * (i32::from(item.ds) + 1) / 2;

    /* here is the place for slays and such */
    let mut mult = 1;

    if (tr(st, BI_WS_ANIMAL) != 0 && rf(r, RF_ANIMAL)) || (tr(st, BI_WS_EVIL) != 0 && rf(r, RF_EVIL))
    {
        mult = 2;
    }
    if (tr(st, BI_WS_UNDEAD) != 0 && rf(r, RF_UNDEAD))
        || (tr(st, BI_WS_DEMON) != 0 && rf(r, RF_DEMON))
        || (tr(st, BI_WS_ORC) != 0 && rf(r, RF_ORC))
        || (tr(st, BI_WS_TROLL) != 0 && rf(r, RF_TROLL))
        || (tr(st, BI_WS_GIANT) != 0 && rf(r, RF_GIANT))
        || (tr(st, BI_WS_DRAGON) != 0 && rf(r, RF_DRAGON))
        || (tr(st, BI_WB_ACID) != 0 && !rf(r, RF_IM_ACID))
        || (tr(st, BI_WB_FIRE) != 0 && !rf(r, RF_IM_FIRE))
        || (tr(st, BI_WB_COLD) != 0 && !rf(r, RF_IM_COLD))
        || (tr(st, BI_WB_POIS) != 0 && !rf(r, RF_IM_POIS))
        || (tr(st, BI_WB_ELEC) != 0 && !rf(r, RF_IM_ELEC))
    {
        mult = 3;
    }
    if (tr(st, BI_WK_UNDEAD) != 0 && rf(r, RF_UNDEAD))
        || (tr(st, BI_WK_DEMON) != 0 && rf(r, RF_DEMON))
        || (tr(st, BI_WK_DRAGON) != 0 && rf(r, RF_DRAGON))
    {
        mult = 5;
    }

    /* add the multiplier */
    dam *= mult;

    /* add weapon bonuses */
    dam += i32::from(item.to_d);

    /* add player bonuses */
    dam += tr(st, BI_TODAM);

    /* multiply the damage for the whole round of attacks */
    dam *= tr(st, BI_BLOWS);

    /* Bonuses for combat */
    let mut chance = tr(st, BI_THN) + ((tr(st, BI_TOHIT) + i32::from(item.to_h)) * 3);

    /* Chance of hitting the monsters AC */
    if chance < (r_ptr.ac * 3 / 4) * 8 / 10 {
        dam = 0;
    }

    /* 5% automatic success/fail */
    chance = chance.clamp(5, 95);

    /* add 10% to chance to give a bit more weight to weapons */
    if tr(st, BI_CLEVEL) > 15 {
        chance += 10;
    }

    /* Mages with Mana do not get that bonus, they should cast */
    if (tr(st, BI_CLASS) == CLASS_MAGE || tr(st, BI_CLASS) == CLASS_NECROMANCER)
        && tr(st, BI_CURSP) > 1
    {
        chance -= 10;
    }

    /* reduce damage by the % chance to hit */
    dam = (dam * chance) / 100;

    /* Try to place a minimal amount of damage */
    if dam <= 0 {
        dam = 1;
    }

    /* Limit damage to twice maximal hitpoints */
    if dam > i32::from(kill.power) * 2 && !rf(r, RF_UNIQUE) {
        dam = i32::from(kill.power) * 2;
    }

    /*
     * Reduce the damage if a mage, they should not melee if they can avoid it
     */
    if (tr(st, BI_CLASS) == CLASS_MAGE || tr(st, BI_CLASS) == CLASS_NECROMANCER)
        && tr(st, BI_MAXCLEVEL) < 40
        && tr(st, BI_CURSP) > 1
    {
        dam = (dam * 8 / 10) + 1;
    }

    /*
     * Enhance the perceived damage on Uniques.  This way we target them
     * Keep in mind that he should hit the uniques but if he has a
     * x5 great bane of dragons, he will tend attack the dragon since the
     * perceived (and actual) damage is higher.  But don't select
     * the town uniques (maggot does no damage)
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
        dam += dam * 5;
    }

    /* Ignore Maggot until later.  Player will chase Maggot
     * down all across the screen waking up all the monsters.  Then
     * he is stuck in a compromised situation.
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) == 0 {
        dam = dam * 2 / 3;

        /* Dont hunt maggot until later */
        if tr(st, BI_CLEVEL) < 5 {
            dam = 0;
        }
    }

    /* give a small bonus for whacking a breeder */
    if rf(r, RF_MULTIPLY) {
        dam = dam * 3 / 2;
    }

    /* Enhance the perceived damage to summoner in order to influence the
     * choice of targets.
     */
    if is_summoner(r, false) {
        dam += (dam * 3) / 2;
    }

    /*
     * Apply massive damage bonus to Questor monsters to
     * encourage borg to strike them.
     */
    if rf(r, RF_QUESTOR) {
        dam += dam * 5;
    }

    /* Damage */
    dam
}

/// The usual melee target scoring of thrust-like attacks: for an adjacent
/// (or reachable) monster with damage `d` at `(x, y)`, `None` to skip it
/// ("hard" sleeping monsters, sleeping town monsters), else the damage
/// plus the danger bonus.
fn melee_score(st: &mut BorgState, k: i32, x: i32, y: i32, mut d: i32) -> Option<i32> {
    /* Avoid waking most "hard" sleeping monsters */
    if !st.kills.kills[k as usize].awake && d <= power(st, k as usize) && !st.borg.munchkin_mode {
        /* Calculate danger */
        let p = borg_danger_one_kill(st, y, x, 1, k, true, true);

        if p > avoid(st) * 2 {
            return None;
        }
    }

    /* Ignore sleeping town monsters */
    if tr(st, BI_CDEPTH) == 0 && !st.kills.kills[k as usize].awake {
        return None;
    }

    /* Calculate "danger" to player */
    let mut p = danger_one_here(st, 2, k, true);

    /* Reduce "bonus" of partial kills when higher level */
    if d <= power(st, k as usize) && tr(st, BI_MAXCLEVEL) > 15 {
        p /= 10;
    }

    /* Add the danger-bonus to the damage */
    d += p;
    Some(d)
}

/// Simulate/Apply the optimal result of making a physical attack
/// (`borg_attack_aux_thrust`).
fn borg_attack_aux_thrust(st: &mut BorgState) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_d: i32 = -1;

    /* Too afraid to attack */
    if tr(st, BI_ISAFRAID) != 0 || tr(st, BI_CRSFEAR) != 0 {
        return 0;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        // FORK: reject stale targets on the player's square, matching C.
        // Queuing +5 there aborts play (docs/c-bugs.md section 54).
        if borg_distance(st.borg.c.y, st.borg.c.x, y, x) != 1 {
            continue;
        }

        /* Acquire grid */
        let k = i32::from(grid(st, x, y).kill);

        /* Calculate "average" damage */
        let d = borg_thrust_damage_one(st, k);

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(st.kills.kills[k as usize].r_idx) {
            continue;
        }

        let Some(d) = melee_score(st, k, x, y, d) else {
            continue;
        };

        /* Ignore lower damage */
        if b_i >= 0 && d < b_d {
            continue;
        }

        /* Save the info */
        b_i = i - 1;
        b_d = d;
    }

    /* Nothing to attack */
    if b_i < 0 {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_d;
    }

    /* Save the location */
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.borg.goal.g.x = tx;
    st.borg.goal.g.y = ty;

    let g = st.borg.goal.g;
    let kill = st.kills.kills[usize::from(grid(st, g.x, g.y).kill)];

    /* Note */
    let mut m = b"# Facing ".to_vec();
    m.extend_from_slice(borg_race_name(i32::from(kill.r_idx)).to_bytes());
    m.extend_from_slice(format!(" at ({},{}) who has {} Hit Points.", g.y, g.x, kill.power).as_bytes());
    borg_note_bytes(st, &m);
    let mut m = b"# Attacking with weapon '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[inven_wield() as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Get a direction for attacking */
    let dir = borg_extract_dir(st.borg.c.y, st.borg.c.x, g.y, g.x);

    /* Attack the grid */
    borg_keypress(st, key(b'+'));
    borg_keypress(st, i2d(dir));

    /* Success */
    b_d
}

/// The best brand/slay multiplier of an object (or of the player's
/// temporary brands and slays for an empty slot) against a race
/// (`borg_best_mult`, adapted from player_attack.c make_ranged_shot()).
fn borg_best_mult(obj: &borg_item, r_idx: u16) -> i32 {
    let flags = &engine::race(usize::from(r_idx)).flags;
    let mut max_mult = 1;

    /* Brands */
    for i in 1..i32::from(engine::z_info().brand_max) {
        let (resist_flag, vuln_flag, multiplier) = engine::brand_resist_vuln_multiplier(i as usize);
        if obj.iqty != 0 {
            /* Brand is on an object */
            if !obj.brands[i as usize] {
                continue;
            }
        } else {
            /* Temporary brand */
            if !engine::player_has_temporary_brand(i) {
                continue;
            }
        }

        /* Is the monster vulnerable? */
        if !engine::flag_has(flags, resist_flag) {
            let mut mult = multiplier;
            if vuln_flag != 0 && engine::flag_has(flags, vuln_flag) {
                mult *= 2;
            }
            max_mult = mult.max(max_mult);
        }
    }

    /* Slays */
    for i in 1..i32::from(engine::z_info().slay_max) {
        let (race_flag, multiplier) = engine::slay_race_flag_multiplier(i as usize);
        if obj.iqty != 0 {
            /* Slay is on an object */
            if obj.slays[i as usize] == 0 {
                continue;
            }
        } else {
            /* Temporary slay */
            if !engine::player_has_temporary_slay(i) {
                continue;
            }
        }

        if engine::flag_has(flags, race_flag) {
            max_mult = multiplier.max(max_mult);
        }
    }
    max_mult
}

/// `strchr("Egv", c)` with C's `char` conversion of the wide `d_char`
/// (the terminating NUL matches too).
fn strchr_egv(d_char: i32) -> bool {
    let c = d_char as u8;
    c == 0 || b"Egv".contains(&c)
}

/// The "out of commission" spells (confuse, scare, slow, sleep): the level
/// test shared by all of them.
fn too_high_level(st: &BorgState, level: i16) -> bool {
    let cl = tr(st, BI_CLEVEL);
    i32::from(level) > if cl < 13 { 10 } else { (((cl - 10) / 4) * 3) + 10 }
}

/// Which borg-trait.c spell flag a "put out of commission" simulation sets.
#[derive(Clone, Copy)]
enum SpellFlag {
    Confuse,
    FearMon,
    Slow,
    Sleep,
    SleepII,
    Crush,
}

/// `borg_<flag>_spell = v` (borg-trait.c globals).
fn set_flag(st: &mut BorgState, f: SpellFlag, v: bool) {
    match f {
        SpellFlag::Confuse => st.traits.confuse_spell = v,
        SpellFlag::FearMon => st.traits.fear_mon_spell = v,
        SpellFlag::Slow => st.traits.slow_spell = v,
        SpellFlag::Sleep => st.traits.sleep_spell = v,
        SpellFlag::SleepII => st.traits.sleep_spell_ii = v,
        SpellFlag::Crush => st.traits.crush_spell = v,
    }
}

/// The danger of monster `i` without and with the spell flag `f`, as C:
/// `flag = false; p1 = ...; [afraid bonus]; flag = true; p2 = ...;
/// flag = false; dam = p1 - p2`.
fn commission_value(st: &mut BorgState, i: i32, f: SpellFlag, afraid_bonus: bool) -> i32 {
    set_flag(st, f, false);
    let mut p1 = danger_one_here(st, 1, i, true);
    /* Make certain monsters appear to have more danger so the borg is more
     * likely to use this attack */
    if afraid_bonus && st.kills.kills[i as usize].afraid && tr(st, BI_CLEVEL) <= 10 {
        p1 += 20;
    }
    set_flag(st, f, true);
    let p2 = danger_one_here(st, 1, i, true);
    set_flag(st, f, false);
    p1 - p2
}

/// Guess how much damage a spell attack will do to a monster
/// (`borg_launch_damage_one`).
///
/// We only handle the "standard" damage types.
///
/// We are paranoid about monster resistances
///
/// He tends to waste all of his arrows on a monsters immediately adjacent
/// to him.  Then he has no arrows for the rest of the level.  We will
/// decrease the damage if the monster is adjacent and we are getting low
/// on missiles.
///
/// We will also decrease the value of the missile attack on breeders or
/// high clevel borgs town scumming.
fn borg_launch_damage_one(st: &mut BorgState, i: i32, mut dam: i32, typ: i32, ammo_location: i32) -> i32 {
    let mut borg_use_missile = false;
    let mut vault_grids = 0;
    let mut gold_eater = false;

    /* Monster record */
    let kill = st.kills.kills[i as usize];
    let r = kill.r_idx;

    /* "player ghosts" */
    if is_ghost(r) {
        return 0;
    }

    /* Monster race */
    let r_ptr = engine::race(usize::from(r));

    /* How far away is the target? */
    let cur_dis = engine::distance(st.borg.c, kill.pos);

    /* Calculation our chance of hitting.  Player bonuses, Bow bonuses, Ammo
     * Bonuses */
    let bow = st.items.items[(inven_wield() + 1) as usize];
    let ammo = st.items.items[ammo_location as usize];
    let bonus = tr(st, BI_TOHIT) + i32::from(bow.to_h) + i32::from(ammo.to_h);
    let chance = tr(st, BI_THB) + (bonus * BTH_PLUS_ADJ);
    let armor = r_ptr.ac + cur_dis;

    /* Very quickly look for gold eating monsters */
    for k in 0..4 {
        /* gold eater */
        if let Some(blow) = blow_effect_name(usize::from(r), k) {
            if borg_mon_blow_effect(blow) == MONBLOW_EAT_GOLD as i32 {
                gold_eater = true;
            }
        }
    }

    /* Analyze the damage type */
    match typ as u32 {
        /* Magic Missile */
        BORG_ATTACK_MISSILE => {}

        BORG_ATTACK_ARROW => {
            let mut mult = borg_best_mult(&bow, r);
            mult = mult.max(borg_best_mult(&ammo, r));
            dam *= mult;
            /* don't point blank non-uniques */
            if cur_dis == 1 && !rf(r, RF_UNIQUE) {
                dam /= 5;
            }
            /* Do I hit regularly? (80%)*/
            if chance < armor * 8 / 10 {
                dam = 0;
            }
        }

        /* Pure damage */
        BORG_ATTACK_MANA => {
            if st.borg.near.unique != 0
                && st.borg.has[st.item_val.kv_potion_restore_mana as usize] > 3
            {
                dam *= 2;
            }
        }

        /* Meteor -- powerful magic missile */
        BORG_ATTACK_METEOR => {}

        /* Acid */
        BORG_ATTACK_ACID => {
            if rf(r, RF_IM_ACID) {
                dam = 0;
            }
        }

        /* Electricity */
        BORG_ATTACK_ELEC => {
            if rf(r, RF_IM_ELEC) {
                dam = 0;
            }
        }

        /* Fire damage */
        BORG_ATTACK_FIRE => {
            if rf(r, RF_IM_FIRE) {
                dam = 0;
            }
            if rf(r, RF_HURT_FIRE) {
                dam *= 2;
            }
        }

        /* Cold */
        BORG_ATTACK_COLD => {
            if rf(r, RF_IM_COLD) {
                dam = 0;
            }
            if rf(r, RF_HURT_COLD) {
                dam *= 2;
            }
        }

        /* Poison */
        BORG_ATTACK_POIS => {
            if rf(r, RF_IM_POIS) {
                dam = 0;
            }
        }

        /* Ice */
        BORG_ATTACK_ICE => {
            if rf(r, RF_IM_COLD) {
                dam = 0;
            }
        }

        /* Holy Orb */
        BORG_ATTACK_HOLY_ORB => {
            if rf(r, RF_EVIL) {
                dam *= 2;
            }
        }

        /* dispel undead */
        BORG_ATTACK_DISP_UNDEAD => {
            if !rf(r, RF_UNDEAD) {
                dam = 0;
            }
        }

        /* dispel spirits */
        BORG_ATTACK_DISP_SPIRITS => {
            if !rf(r, RF_SPIRIT) {
                dam = 0;
            }
        }

        /*  Dispel Evil */
        BORG_ATTACK_DISP_EVIL => {
            if !rf(r, RF_EVIL) {
                dam = 0;
            }
        }

        /*  Dispel life: the game's drain only hurts living monsters
         * (nonliving is undead or nonliving). */
        BORG_ATTACK_DRAIN_LIFE => {
            if rf(r, RF_NONLIVING) || rf(r, RF_UNDEAD) {
                dam = 0;
            }
        }

        /*  Holy Word */
        BORG_ATTACK_HOLY_WORD => {
            if !rf(r, RF_EVIL) {
                dam = 0;
            }
        }

        /* Weak Lite */
        BORG_ATTACK_LIGHT_WEAK => {
            if !rf(r, RF_HURT_LIGHT) {
                dam = 0;
            }
        }

        /* Drain Life */
        BORG_ATTACK_OLD_DRAIN => {
            if engine::distance(st.borg.c, kill.pos) == 1 {
                dam /= 5;
            }
            if rf(r, RF_UNDEAD) || rf(r, RF_DEMON) || strchr_egv(r_ptr.d_char) {
                dam = 0;
            }
        }

        /* Stone to Mud */
        BORG_ATTACK_KILL_WALL => {
            if !rf(r, RF_HURT_ROCK) {
                dam = 0;
            }
        }

        /* New mage spell */
        BORG_ATTACK_NETHER => {
            if rf(r, RF_UNDEAD) {
                dam = 0;
            } else if rsf(r, RSF_BR_NETH) {
                dam *= 3;
                dam /= 9;
            } else if rf(r, RF_EVIL) {
                dam /= 2;
            }
        }

        /* New mage spell */
        BORG_ATTACK_CHAOS => {
            if rsf(r, RSF_BR_CHAO) {
                dam *= 3;
                dam /= 9;
            }
            /* If the monster is Unique full damage ok.
             * Otherwise, polymorphing will reset HP
             */
            if !rf(r, RF_UNIQUE) {
                dam = -999;
            }
        }

        /* New mage spell */
        BORG_ATTACK_GRAVITY => breath_third(&mut dam, r, RSF_BR_GRAV),
        /* New mage spell */
        BORG_ATTACK_SHARD => breath_third(&mut dam, r, RSF_BR_SHAR),
        /* New mage spell */
        BORG_ATTACK_SOUND => breath_third(&mut dam, r, RSF_BR_SOUN),
        /* Weird attacks */
        BORG_ATTACK_PLASMA => breath_third(&mut dam, r, RSF_BR_PLAS),

        BORG_ATTACK_CONFU => {
            if rf(r, RF_NO_CONF) {
                dam = 0;
            }
        }

        BORG_ATTACK_DISEN => breath_third(&mut dam, r, RSF_BR_DISE),
        BORG_ATTACK_NEXUS => breath_third(&mut dam, r, RSF_BR_NEXU),
        BORG_ATTACK_FORCE => breath_third(&mut dam, r, RSF_BR_WALL),
        BORG_ATTACK_INERTIA => breath_third(&mut dam, r, RSF_BR_INER),
        BORG_ATTACK_TIME => breath_third(&mut dam, r, RSF_BR_TIME),
        BORG_ATTACK_LIGHT => breath_third(&mut dam, r, RSF_BR_LIGHT),
        BORG_ATTACK_DARK => breath_third(&mut dam, r, RSF_BR_DARK),

        BORG_ATTACK_WATER => {
            breath_third(&mut dam, r, RSF_BA_WATE);
            dam /= 2;
        }

        /* Various */
        BORG_ATTACK_OLD_HEAL
        | BORG_ATTACK_OLD_CLONE
        | BORG_ATTACK_OLD_SPEED
        | BORG_ATTACK_DARK_WEAK
        | BORG_ATTACK_KILL_DOOR
        | BORG_ATTACK_KILL_TRAP
        | BORG_ATTACK_MAKE_WALL
        | BORG_ATTACK_MAKE_DOOR
        | BORG_ATTACK_MAKE_TRAP
        | BORG_ATTACK_AWAY_UNDEAD
        | BORG_ATTACK_TURN_EVIL => {
            dam = 0;
        }

        /* These spells which put the monster out of commission, we
         * look at the danger of the monster prior to and after being
         * put out of commission.  The difference is the damage.
         * The following factors are considered when we
         * consider the spell:
         *
         * 1. Is it already comprised by that spell?
         * 2. Is it compromised by another spell?
         * 3. Does it resist the modality?
         * 4. Will it make it's savings throw better than half the time?
         * 5. We generally ignore these spells for breeders.
         *
         * The spell sleep II and sanctuary have a special consideration
         * since the monsters must be adjacent to the player.
         */
        BORG_ATTACK_AWAY_ALL => {
            /* Teleport Other works differently.  Basically the borg
             * will keep a list of all the monsters in the line of
             * fire.  Then when he checks the danger, he will not
             * include those monsters.
             */

            /* try not to teleport away uniques. These are the guys you are
             * trying to kill! */
            if rf(r, RF_UNIQUE) {
                if kill.injury >= 60 {
                    /* This unique is low on HP, finish it off */
                    dam = -9999;
                } else if st.borg.status.anti_summon {
                    /* I am sitting pretty in an AS-Corridor */
                    dam = -9999;
                } else if dam > avoid(st) * 13 / 10 && tr(st, BI_CDEPTH) <= 98 {
                    /* If this unique is causing the danger, get rid of it */
                    tp_other_add(st, i);
                } else if st.borg.near.unique >= 2 && st.borg.near.unique <= 8 {
                    /* If fighting multiple uniques, get rid of one */
                    tp_other_add(st, i);
                } else if tr(st, BI_CLASS) == CLASS_MAGE && engine::distance(st.borg.c, kill.pos) <= 2 {
                    /* Unique is adjacent to Borg */
                    tp_other_add(st, i);
                } else if st.borg.status.vault {
                    /* Unique in a vault, get rid of it, clean vault */
                    /* Scan grids adjacent to monster */
                    for ii in 0..8 {
                        let x = kill.pos.x + i32::from(engine::ddx_ddd(ii));
                        let y = kill.pos.y + i32::from(engine::ddy_ddd(ii));

                        /* Access the grid */
                        let ag = grid(st, x, y);

                        /* Skip unknown grids (important) */
                        if u32::from(ag.feat) == FEAT_NONE {
                            continue;
                        }

                        /* Count adjacent Permas */
                        if u32::from(ag.feat) == FEAT_PERM {
                            vault_grids += 1;
                        }
                    }

                    /* Near enough perma grids? */
                    if vault_grids >= 2 {
                        /* get rid of unique next to perma grids */
                        tp_other_add(st, i);
                    }
                } else {
                    dam = -999;
                }
            } else {
                /* not a unique: get rid of this non-unique by storing his
                 * info */
                tp_other_add(st, i);
            }
        }

        /* This teleport away is used to teleport away all monsters
         * as the borg goes through his special attacks.
         */
        BORG_ATTACK_AWAY_ALL_MORGOTH => {
            /* Mostly no damage */
            dam = 0;

            /* If its touching a glyph grid, nail it. */
            for j in 0..8 {
                let y2 = kill.pos.y + i32::from(engine::ddy_ddd(j));
                let x2 = kill.pos.x + i32::from(engine::ddx_ddd(j));

                /* If its touching a glyph grid, nail it. */
                if grid(st, x2, y2).glyph {
                    /* get rid of this one by storing his info */
                    tp_other_add(st, i);
                    dam = 300;
                }
            }

            /* If the borg is not in a good position, do it */
            if st.borg.near.morgoth && !st.borg.morgoth_position {
                /* get rid of this one by storing his info */
                tp_other_add(st, i);
                dam = 100;
            }

            /* If the borg does not have enough Mana to attack this
             * round and cast Teleport Away next round, then do it now.
             */
            if tr(st, BI_CURSP) <= 35 {
                /* get rid of this unique by storing his info */
                tp_other_add(st, i);
                dam = 150;
            }
        }

        /* This BORG_ATTACK_ is hacked to work for Mass Genocide.  Since
         * we cannot mass gen uniques.
         */
        BORG_ATTACK_DISP_ALL => {
            if rf(r, RF_UNIQUE) {
                dam = 0;
            } else {
                dam = danger_one_here(st, 1, i, true);
            }
        }

        BORG_ATTACK_OLD_CONF => {
            dam = 0;
            if !(rf(r, RF_NO_CONF)
                || rf(r, RF_MULTIPLY)
                || i32::from(kill.speed) < r_ptr.speed - 5
                || kill.confused
                || !kill.awake
                || too_high_level(st, kill.level))
            {
                dam = -999;
                if !rf(r, RF_UNIQUE) {
                    dam = commission_value(st, i, SpellFlag::Confuse, true);
                }
            }
        }

        BORG_ATTACK_TURN_ALL => {
            dam = 0;
            if !(i32::from(kill.speed) < r_ptr.speed - 5
                || rf(r, RF_NO_FEAR)
                || kill.confused
                || !kill.awake
                || too_high_level(st, kill.level))
            {
                dam = -999;
                if !rf(r, RF_UNIQUE) {
                    dam = commission_value(st, i, SpellFlag::FearMon, true);
                }
            }
        }

        BORG_ATTACK_OLD_SLOW => {
            dam = 0;
            if !(i32::from(kill.speed) < r_ptr.speed - 5
                || kill.confused
                || !kill.awake
                || too_high_level(st, kill.level))
            {
                dam = -999;
                if !rf(r, RF_UNIQUE) {
                    dam = commission_value(st, i, SpellFlag::Slow, true);
                }
            }
        }

        BORG_ATTACK_OLD_SLEEP | BORG_ATTACK_SLEEP_EVIL => {
            dam = 0;
            if !(rf(r, RF_NO_SLEEP)
                || (!rf(r, RF_EVIL) && typ as u32 == BORG_ATTACK_SLEEP_EVIL)
                || i32::from(kill.speed) < r_ptr.speed - 5
                || kill.confused
                || !kill.awake
                || too_high_level(st, kill.level))
            {
                dam = -999;
                if !rf(r, RF_UNIQUE) {
                    dam = commission_value(st, i, SpellFlag::Sleep, true);
                }
            }
        }

        BORG_ATTACK_OLD_POLY => {
            dam = 0;
            if !too_high_level(st, kill.level) {
                dam = -999;
                if !rf(r, RF_UNIQUE) {
                    dam = danger_one_here(st, 2, i, true);
                    /* don't bother unless he is a scary monster */
                    if dam < avoid(st) * 2 && !kill.afraid {
                        dam = 0;
                    }
                }
            }
        }

        BORG_ATTACK_TURN_UNDEAD => {
            if rf(r, RF_UNDEAD) {
                dam = 0;
                if !(kill.confused
                    || i32::from(kill.speed) < r_ptr.speed - 5
                    || !kill.awake
                    || i32::from(kill.level) > tr(st, BI_CLEVEL) - 5)
                {
                    dam = commission_value(st, i, SpellFlag::FearMon, false);
                }
            } else {
                dam = 0;
            }
        }

        /* Banishment-- cast when in extreme danger (checked in borg_defense). */
        BORG_ATTACK_AWAY_EVIL => {
            if rf(r, RF_EVIL) {
                /* try not teleport away uniques. */
                if rf(r, RF_UNIQUE) {
                    /* Banish ones with escorts */
                    if has_friends(r) {
                        dam = 0;
                    } else {
                        /* try not Banish non escorted uniques */
                        dam = -500;
                    }
                } else {
                    /* damage is the danger of the baddie */
                    dam = danger_one_here(st, 1, i, true);
                }
            } else {
                dam = 0;
            }
        }

        BORG_ATTACK_TAP_UNLIFE => {
            /* for now ignore the gain in sp */
            if !rf(r, RF_UNDEAD) {
                dam = 0;
            } else {
                /* (C subtracts the current SP from itself: always 0) */
                let sp_drain = tr(st, BI_CURSP) - tr(st, BI_CURSP);
                if sp_drain < i32::from(kill.power) {
                    dam = i32::from(kill.power) - sp_drain;
                }
            }
        }

        BORG_ATTACK_CURSE => {
            dam = ((tr(st, BI_CLEVEL) / 12) * ((50 + i32::from(kill.injury)) + 1)) / 2;
        }

        BORG_ATTACK_ELEC_STRIKE => {
            if rf(r, RF_IM_ELEC) {
                dam = 0;
            } else if !borg_projectable_pure(st, st.borg.c.y, st.borg.c.x, kill.pos.y, kill.pos.x) {
                /* don't let lightning strike pass through unknown squares */
                dam = 0;
            }
        }

        _ => {}
    }

    /* use Missiles on certain types of monsters */
    if tr(st, BI_CDEPTH) >= 1
        && (borg_danger_one_kill(st, kill.pos.y, kill.pos.x, 1, i, true, true) > avoid(st) * 2 / 10
            || (has_friends(r) /* monster has friends*/
                && i32::from(kill.level) >= tr(st, BI_CLEVEL) - 5/* close levels */)
            || kill.ranged_attack != 0 /* monster has a ranged attack */
            || rf(r, RF_UNIQUE)
            || rf(r, RF_MULTIPLY)
            || gold_eater /* Monster can steal gold */
            || rf(r, RF_NEVER_MOVE) /* monster never moves */
            || tr(st, BI_CLEVEL) <= 20/* stil very weak */)
    {
        borg_use_missile = true;
    }

    /* Return Damage as pure danger of the monster */
    if typ as u32 == BORG_ATTACK_AWAY_ALL
        || typ as u32 == BORG_ATTACK_AWAY_EVIL
        || typ as u32 == BORG_ATTACK_AWAY_ALL_MORGOTH
    {
        return dam;
    }

    /* Limit damage to twice maximal hitpoints */
    if dam > i32::from(kill.power) * 2 && !rf(r, RF_UNIQUE) {
        dam = i32::from(kill.power) * 2;
    }

    /* give a small bonus for whacking a unique */
    /* this should be just enough to give preference to whacking uniques */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
        dam *= 3;
    }

    /* Ignore Maggot until later.  Player will chase Maggot
     * down all across the screen waking up all the monsters.  Then
     * he is stuck in a compromised situation.
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) == 0 {
        dam = dam * 2 / 3;

        /* Don't hunt maggot until later */
        if tr(st, BI_CLEVEL) < 5 {
            dam = 0;
        }
    }

    /* give a small bonus for whacking a breeder */
    if rf(r, RF_MULTIPLY) {
        dam = dam * 3 / 2;
    }

    /* Enhance the perceived damage to summoner in order to influence the
     * choice of targets.
     */
    if is_summoner(r, true) {
        dam += (dam * 3) / 2;
    }

    /*
     * Apply massive damage bonus to Questor monsters to
     * encourage borg to strike them.
     */
    if rf(r, RF_QUESTOR) {
        dam += dam * 9;
    }

    /*  Try to conserve missiles.
     */
    if typ as u32 == BORG_ATTACK_ARROW && !borg_use_missile {
        /* set damage to zero, force borg to melee attack */
        dam = 0;
    }

    /* Damage */
    dam
}

/// `if (rsf_has(r_ptr->spell_flags, flag)) { dam *= 3; dam /= 9; }`
fn breath_third(dam: &mut i32, r: u16, flag: u32) {
    if rsf(r, flag) {
        *dam *= 3;
        *dam /= 9;
    }
}

/// `r_ptr->blow[k].effect ? r_ptr->blow[k].effect->name : NULL` for the
/// gold-eater scan (blows past the race's last one have no effect).
fn blow_effect_name(r_idx: usize, k: usize) -> Option<&'static [u8]> {
    engine::race_blow_effect_name(r_idx, k)
}

/// Simulate / Invoke the launching of a bolt at a monster
/// (`borg_launch_bolt_aux_hack`).
fn borg_launch_bolt_aux_hack(st: &mut BorgState, i: i32, dam: i32, typ: i32, ammo_location: i32) -> i32 {
    let mut walls = 0;
    let mut unknown = 0;

    /* Monster */
    let kill = st.kills.kills[i as usize];
    let r = kill.r_idx;

    /* Skip dead monsters */
    if r == 0 {
        return 0;
    }

    /* "player ghosts" */
    if is_ghost(r) {
        return 0;
    }

    /* Require current knowledge */
    if borg_timer(st, kill.when) > 2 {
        return 0;
    }

    /* Acquire location */
    let x = kill.pos.x;
    let y = kill.pos.y;

    /* Acquire the grid */
    let ag = grid(st, x, y);

    /* Never shoot walls/doors */
    if !borg_cave_floor_grid(&ag) {
        return 0;
    }

    /* dont shoot at ghosts if not on known floor grid */
    let feat = u32::from(ag.feat);
    if rf(r, RF_PASS_WALL) && (feat != FEAT_FLOOR && feat != FEAT_OPEN && feat != FEAT_BROKEN && !ag.trap)
    {
        return 0;
    }

    /* dont shoot at ghosts in walls, not perfect */
    if rf(r, RF_PASS_WALL) {
        /* if 2 walls and 1 unknown skip this monster */
        for o_x in -1..=1 {
            for o_y in -1..=1 {
                /* Acquire location */
                let ag = grid(st, kill.pos.x + o_x, kill.pos.y + o_y);
                let f = u32::from(ag.feat);

                if (FEAT_MAGMA..=FEAT_PERM).contains(&f) {
                    walls += 1;
                }
                if f == FEAT_NONE {
                    unknown += 1;
                }
            }
        }
        /* Is the ghost likely in a wall? */
        if walls >= 2 && unknown >= 1 {
            return 0;
        }
    }

    /* Calculate damage */
    let mut d = borg_launch_damage_one(st, i, dam, typ, ammo_location);

    /* Return Damage, on Teleport Other, true damage is
     * calculated elsewhere */
    if typ as u32 == BORG_ATTACK_AWAY_ALL || typ as u32 == BORG_ATTACK_AWAY_ALL_MORGOTH {
        return d;
    }

    /* Return Damage as pure danger of the monster */
    if typ as u32 == BORG_ATTACK_AWAY_EVIL {
        return d;
    }

    /* Return 0 if the true damage (w/o the danger bonus) is 0 */
    if d <= 0 {
        return d;
    }

    /* Calculate danger (at the monster's grid: `x`, `y` are still the
     * monster's position, or its last neighbour for a ghost) */
    let (dy, dx) = if rf(r, RF_PASS_WALL) { (kill.pos.y + 1, kill.pos.x + 1) } else { (y, x) };
    let p2 = borg_danger_one_kill(st, dy, dx, 1, i, true, false);

    /* Avoid waking most "hard" sleeping monsters */
    if !kill.awake && p2 > avoid(st) / 2 && d < i32::from(kill.power) && !st.borg.munchkin_mode {
        return -999;
    }

    /* Ignore sleeping town monsters */
    if tr(st, BI_CDEPTH) == 0 && !kill.awake {
        return 0;
    }

    /* Calculate "danger" to player */
    let p1 = danger_one_here(st, 1, i, false);

    /* Extra "bonus" if attack kills */
    if d >= i32::from(kill.power) {
        d *= 2;
    }

    /* Add in dangers */
    d + p1
}

/// Determine the penalty for destroying stuff by launching a spell at it
/// (`borg_launch_destroy_stuff`).
fn borg_launch_destroy_stuff(st: &BorgState, ag: &BorgGrid, typ: i32) -> i32 {
    let take = &st.takes.takes[usize::from(ag.take)];
    let k_ptr = engine::kind(take.kidx as usize);
    let iv = &st.item_val;

    match typ as u32 {
        BORG_ATTACK_ACID => {
            /* rings/boots cost extra (might be speed!) */
            if k_ptr.tval == TV_BOOTS as i32 && !k_ptr.aware {
                return 20;
            }
        }
        BORG_ATTACK_ELEC => {
            /* rings/boots cost extra (might be speed!) */
            if k_ptr.tval == TV_RING as i32 && !k_ptr.aware {
                return 20;
            }
            if k_ptr.tval == TV_RING as i32 && k_ptr.sval == iv.sv_ring_speed {
                return 2000;
            }
        }
        BORG_ATTACK_FIRE => {
            /* rings/boots cost extra (might be speed!) */
            if k_ptr.tval == TV_BOOTS as i32 && !k_ptr.aware {
                return 20;
            }
        }
        BORG_ATTACK_COLD => {
            if k_ptr.tval == TV_POTION as i32 {
                let need = &st.borg.need_statgain;
                /* Extra penalty for cool potions */
                if !k_ptr.aware
                    || k_ptr.sval == iv.sv_potion_healing
                    || k_ptr.sval == iv.sv_potion_star_healing
                    || k_ptr.sval == iv.sv_potion_life
                    || (k_ptr.sval == iv.sv_potion_inc_str && need[STAT_STR as usize])
                    || (k_ptr.sval == iv.sv_potion_inc_int && need[STAT_INT as usize])
                    || (k_ptr.sval == iv.sv_potion_inc_wis && need[STAT_WIS as usize])
                    || (k_ptr.sval == iv.sv_potion_inc_dex && need[STAT_DEX as usize])
                    || (k_ptr.sval == iv.sv_potion_inc_con && need[STAT_CON as usize])
                {
                    return 2000;
                }

                return 20;
            }
        }
        /* Used against uniques, allow the stuff to burn */
        BORG_ATTACK_MANA => {}
        _ => {}
    }

    0
}

/// The "stop here" result of the path scans: beams (`rad == -1`) and
/// "dispels" (`rad == 10`) keep what they collected, everything else
/// scores 0.
fn path_stop(rad: i32, n: i32) -> i32 {
    if rad != -1 && rad != 10 {
        0
    } else {
        n
    }
}

/// "Stop at unseen walls / we just shot and missed": with
/// `successful_target < 0` the shot is skipped (a miss of -12 or worse is
/// forgotten, so only one round is skipped). `true` = stop.
fn missed_before(st: &mut BorgState) -> bool {
    if st.fight_attack.successful_target < 0 {
        /* When throwing things, it is common to just 'miss' */
        /* Skip only one round in this case */
        if st.fight_attack.successful_target <= -12 {
            st.fight_attack.successful_target = 0;
        }
        return true;
    }
    false
}

/// "if this area has been magic mapped, ok to shoot in the dark" is
/// false: no wall detection in the four panels around `(q_x, q_y)` and
/// the region's fear below `avoidance / 20` (C's `&&` order).
fn not_mapped(st: &BorgState, q_x: i32, q_y: i32) -> bool {
    !st.update.detect(Detect::Wall, q_y, q_x)
        && !st.update.detect(Detect::Wall, q_y, q_x + 1)
        && !st.update.detect(Detect::Wall, q_y + 1, q_x)
        && !st.update.detect(Detect::Wall, q_y + 1, q_x + 1)
        && i32::from(st.danger.fear_region[(st.borg.c.y / 11) as usize][(st.borg.c.x / 11) as usize])
            < avoid(st) / 20
}

/// Determine the "reward" of launching a beam/bolt/ball at a location
/// (`borg_launch_bolt_at_location`).
///
/// An "unreachable" location always has zero reward.
///
/// Basically, we sum the "rewards" of doing the appropriate amount of
/// damage to each of the "affected" monsters.
#[allow(clippy::too_many_arguments)]
fn borg_launch_bolt_at_location(
    st: &mut BorgState,
    y: i32,
    x: i32,
    rad: i32,
    dam: i32,
    typ: i32,
    max: i32,
    ammo_location: i32,
) -> i32 {
    /* Extract panel */
    let q_x = st.core.w_x / borg_panel_wid();
    let q_y = st.core.w_y / borg_panel_hgt();

    /* Reset damage */
    let mut n = 0;

    /* Initial location */
    let x1 = st.borg.c.x;
    let y1 = st.borg.c.y;

    /* Final location */
    let x2 = x;
    let y2 = y;

    /* Bounds Check */
    if !engine::square_in_bounds_fully(x, y) {
        return 0;
    }

    /* Start over */
    let mut x = x1;
    let mut y = y1;

    /* Get the grid of the targetted monster */
    let kill_r = st.kills.kills[usize::from(grid(st, x2, y2).kill)].r_idx;

    /* "player ghosts" */
    if is_ghost(kill_r) {
        return 0;
    }
    let r_light = engine::race(usize::from(kill_r)).light;

    /* Simulate the spell/missile path */
    let mut dist = 1;
    while dist < max {
        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);

        /* Bounds Check */
        if !engine::square_in_bounds_fully(x, y) {
            break;
        }

        /* Get the grid of the pathway */
        let ag = grid(st, x, y);

        /* Stop at walls */
        /* note if beam, this is the end of the beam */
        /* dispel spells act like beams (sort of) */
        if !borg_cave_floor_grid(&ag) || u32::from(ag.feat) == FEAT_PASS_RUBBLE {
            return path_stop(rad, n);
        }

        /* Collect damage (bolts/beams) */
        if rad <= 0 {
            n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam, typ, ammo_location);
        }

        /* Check for arrival at "final target" */
        /* except beams, which keep going. */
        if rad != -1 && (x == x2 && y == y2) {
            break;
        }

        /* Stop bolts at monsters  */
        if rad == 0 && ag.kill != 0 {
            return n;
        }

        /* The missile path can be complicated.  There are several checks
         * which need to be made.  First we assume that we targeting
         * a monster.  That monster could be known from either sight or
         * ESP.  If the entire pathway from us to the monster is known,
         * then there is no concern.  But if the borg is shooting through
         * unknown grids, then there is a concern when he has ESP; without
         * ESP he would not see that monster if the unknown grids
         * contained walls or closed doors.
         *
         * 1.  ESP Inactive
         *   A.  No Infravision
         *       -Then the monster must be in a lit grid. OK to shoot
         *   B.  Yes Infravision
         *       -Then the monster must be projectable()  OK to shoot
         * 2.  ESP Active
         *   A. No Infravision
         *       -Then the monster could be in a lit grid.  Try to shoot
         *       -Or I detect it with ESP and it's not projectable().
         *   B.  Yes Infravision
         *       -Then the monster could be projectable()
         *       -Or I detect it with ESP and it's not projectable().
         *   -In the cases of ESP Active, the borg will test fire a missile.
         *    Then wait for a 'painful ouch' from the monster.
         *
         * Low level borgs will not take the shot unless they have
         * a clean and known pathway.  Borgs over a certain clevel,
         * will attempt the shot and listen for the 'ouch' response
         * to know that the clear.  If no 'Ouch' is heard, then the
         * borg will assume there is a wall in the way.  Exception to
         * this is with arrows.  Arrows can miss the target or fall
         * fall short, in which case no 'ouch' is heard.  So the borg
         * allowed to miss two shots with arrows/bolts/thrown objects.
         */

        /* dont do the check if esp */
        if tr(st, BI_ESP) == 0 {
            /* Check the missile path--no Infra, no HAS_LIGHT */
            if tr(st, BI_INFRA) <= 0 && r_light <= 0 {
                /* Stop at unknown grids (see above) */
                /* note if beam, dispel, this is the end of the beam */
                if u32::from(ag.feat) == FEAT_NONE {
                    return path_stop(rad, n);
                }
            }

            /* Stop at unseen walls */
            /* We just shot and missed, this is our next shot */
            if missed_before(st) {
                return path_stop(rad, n);
            }
        } else {
            /* I do have ESP */
            /* if this area has been magic mapped,
             * ok to shoot in the dark
             */
            if not_mapped(st, q_x, q_y) {
                /* Stop at unknown grids (see above) */
                /* note if beam, dispel, this is the end of the beam */
                if u32::from(ag.feat) == FEAT_NONE {
                    return path_stop(rad, n);
                }
                /* Stop at unseen walls */
                /* We just shot and missed, this is our next shot */
                if missed_before(st) {
                    return path_stop(rad, n);
                }
            }

            /* Stop at unseen walls */
            /* We just shot and missed, this is our next shot */
            if missed_before(st) {
                return path_stop(rad, n);
            }
        }
        dist += 1;
    }

    /* Bolt/Beam attack */
    if rad <= 0 {
        return n;
    }

    /* Excessive distance */
    if dist >= max {
        return 0;
    }

    /* Check monsters and objects in blast radius */
    for ry in (y2 - rad)..=(y2 + rad) {
        for rx in (x2 - rad)..=(x2 + rad) {
            /* Bounds check */
            if !engine::square_in_bounds(rx, ry) {
                continue;
            }

            /* Get the grid */
            let ag = grid(st, rx, ry);

            /* Check distance */
            let mut r = borg_distance(y2, x2, ry, rx);

            /* Maximal distance */
            if r > rad {
                continue;
            }

            /* Never pass through walls*/
            if !borg_los(st, y2, x2, ry, rx) {
                continue;
            }

            /*  dispel spells should hurt the same no matter the rad: make r= y
             * and x */
            if rad == 10 {
                r = 0;
            }

            /* Collect damage, lowered by distance */
            n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam / (r + 1), typ, ammo_location);

            /* probable damage int was just changed by b_l_b_a_h*/

            /* check destroyed stuff. */
            let ag = grid(st, rx, ry);
            if ag.take != 0 && st.takes.takes[usize::from(ag.take)].kidx != 0 {
                n -= borg_launch_destroy_stuff(st, &ag, typ);
            }
        }
    }

    /* Result */
    n
}

/// Simulate/Apply the optimal result of launching a beam/bolt/ball
/// (`borg_launch_bolt`).
///
/// Note that "beams" have a "rad" of "-1", "bolts" have a "rad" of "0",
/// and "balls" have a "rad" of "2" or "3", depending on "blast radius".
///  dispel spells have a rad  of 10
pub fn borg_launch_bolt(st: &mut BorgState, rad: i32, dam: i32, typ: i32, max: i32, ammo_location: i32) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_n: i32 = -1;
    let mut b_o_y = 0;
    let mut b_o_x = 0;
    let mut b_d = max_range();

    /* Examine possible destinations */

    /* This will allow the borg to target places adjacent to a monster
     * in order to exploit and abuse a feature of the game.  Whereas,
     * the borg, while targeting a monster will not score d/t walls, he
     * could land a successful hit by targeting adjacent to the monster.
     * For example:
     * ######################
     * #####....@......######
     * ############Px........
     * ######################
     * In order to hit the P, the borg must target the x and not the P.
     *
     */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (ty, tx) = st.flow.temp_at(i as usize);

        /* Consider each adjacent spot to and on top of the monster */
        for o_x in -1..=1 {
            for o_y in -1..=1 {
                /* Acquire location */
                let x = tx + o_x;
                let y = ty + o_y;

                /* Reset Teleport Other variables */
                st.fight_attack.tp_other_n = 0;
                let mut n = 0;

                /* Bounds check */
                if !engine::square_in_bounds(x, y) {
                    continue;
                }

                /* Remember how far away the monster is */
                let d = dist_to(st, tx, ty);

                /* Skip certain types of Offset attacks */
                if (x != tx || y != ty) && typ as u32 == BORG_ATTACK_AWAY_ALL {
                    continue;
                }

                /* Skip places that are out of range */
                if dist_to(st, x, y) > max {
                    continue;
                }

                /* Consider it if its a ball spell or right on top of it */
                if (rad >= 2 && u32::from(grid(st, x, y).feat) != FEAT_NONE) || (y == ty && x == tx) {
                    n = borg_launch_bolt_at_location(st, y, x, rad, dam, typ, max, ammo_location);
                }

                /* Teleport Other is now considered */
                if typ as u32 == BORG_ATTACK_AWAY_ALL && n > 0 {
                    /* Consider danger with certain monsters removed
                     * from the danger check.  They were removed from the list
                     * of considered monsters (borg_tp_other array)
                     */
                    n = danger_here(st, 1, true);

                    /* since this is the danger after monster removal */
                    /* and dam for tel away is the previous danger */
                    /* we want the biggest reduction in danger */
                    n = dam - n;
                }

                /* Reset Teleport Other variables */
                st.fight_attack.tp_other_n = 0;

                /* Skip useless attacks */
                if n <= 0 {
                    continue;
                }

                /* The game forbids targeting the outside walls */
                if x == 0 || y == 0 || x == DUNGEON_WID - 1 || y == DUNGEON_HGT - 1 {
                    continue;
                }

                /* Collect best attack */
                if b_i >= 0 && n < b_n {
                    continue;
                }

                /* Skip attacking farther monster if rewards are equal. */
                if n == b_n && d > b_d {
                    continue;
                }

                /* Track it */
                b_i = i;
                b_n = n;
                b_o_y = o_y;
                b_o_x = o_x;
                b_d = d;
            }
        }
        i += 1;
    }
    if b_i == -1 {
        return 0;
    }

    /* Reset Teleport Other variables */
    st.fight_attack.tp_other_n = 0;

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Save the location */
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.borg.goal.g.x = tx + b_o_x;
    st.borg.goal.g.y = ty + b_o_y;

    /* will need to add SINGLE COMBAT and COMMAND if those end up coded */
    let require_monster = typ as u32 == BORG_ATTACK_CURSE;

    /* Target the location */
    let g = st.borg.goal.g;
    let _ = borg_target(st, g, require_monster);

    /* Result */
    b_n
}

/// Determine the "reward" of launching an arc/spray/cone at a location
/// (`borg_launch_arc_at_location`).
///
/// This code is copied from borg_launch_bolt_at_location
/// both need to be optimized for shared code !FIX
fn borg_launch_arc_at_location(st: &mut BorgState, y: i32, x: i32, degrees: i32, dam: i32, typ: i32, max: i32) -> i32 {
    let mut path_grids = [loc { x: 0, y: 0 }; 256];

    /* Extract panel */
    let q_x = st.core.w_x / borg_panel_wid();
    let q_y = st.core.w_y / borg_panel_hgt();

    /* Reset damage */
    let mut n = 0;

    /* Initial location */
    let x1 = st.borg.c.x;
    let y1 = st.borg.c.y;

    /* Final location */
    let x2 = x;
    let y2 = y;

    /* Bounds Check */
    if !engine::square_in_bounds_fully(x, y) {
        return 0;
    }

    /* Start over */
    let mut x = x1;
    let mut y = y1;

    /* Get the grid of the targetted monster */
    let kill_r = st.kills.kills[usize::from(grid(st, x2, y2).kill)].r_idx;

    /* "player ghosts" */
    if is_ghost(kill_r) {
        return 0;
    }
    let r_light = engine::race(usize::from(kill_r)).light;

    /* starting square is always good */
    path_grids[0].x = x;
    path_grids[0].y = y;

    /* Simulate the spell/missile path */
    let mut dist = 1;
    while dist < max {
        /* Calculate the new location */
        borg_inc_motion(&mut y, &mut x, y1, x1, y2, x2);

        /* Bounds Check */
        if !engine::square_in_bounds_fully(x, y) {
            break;
        }

        let ag = grid(st, x, y);

        /* Stop at walls */
        /* note if beam, this is the end of the beam */
        /* dispel spells act like beams (sort of) */
        if !borg_cave_floor_grid(&ag) || u32::from(ag.feat) == FEAT_PASS_RUBBLE {
            break;
        }

        path_grids[dist as usize].x = x;
        path_grids[dist as usize].y = y;

        /* Check for arrival at "final target" */
        /* except beams, which keep going. */
        if x == x2 && y == y2 {
            break;
        }

        /* check unmapped squares */
        if u32::from(ag.feat) == FEAT_NONE {
            /* Check if we have ESP */
            if tr(st, BI_ESP) != 0 {
                /* no Infra, no HAS_LIGHT */
                if tr(st, BI_INFRA) <= 0 && r_light <= 0 {
                    break;
                }
            } else {
                /* no ESP */
                /* if this area has been magic mapped,
                 * ok to shoot in the dark
                 */
                if not_mapped(st, q_x, q_y) {
                    break;
                }
            }

            /* Stop if we missed previously */
            if missed_before(st) {
                /* reset the "miss" so only one shot is skipped  */
                break;
            }
        }
        dist += 1;
    }

    if dist < 21 {
        dist -= 1;
    } else {
        dist = 20;
    }

    /* Use angle comparison to delineate an arc. */

    /* reorient arc beam */
    let n1y = path_grids[dist as usize].y - y1 + 20;
    let n1x = path_grids[dist as usize].x - x1 + 20;

    /* Check monsters and objects in blast radius and on the arc */
    /* for an arc, blast radius starts at player */
    for ry in (y1 - max)..=(y1 + max) {
        for rx in (x1 - max)..=(x1 + max) {
            /* Bounds check */
            if !engine::square_in_bounds(rx, ry) {
                continue;
            }

            /* Get the grid */
            let ag = grid(st, rx, ry);

            /* Check distance */
            let r = borg_distance(y1, x1, ry, rx);

            /* Maximal distance */
            if r > max {
                continue;
            }

            /* Never pass through walls*/
            if !borg_los(st, y1, x1, ry, rx) {
                continue;
            }

            /* check on angle */
            /* Reorient current grid for table access. */
            let n2y = ry - y1 + 20;
            let n2x = rx - x1 + 20;

            /* Find the angular difference (/2) between the lines to
             * the end of the arc's center-line and to the current grid.
             */
            let rotate = 90 - i32::from(engine::get_angle_to_grid(n1y as usize, n1x as usize));
            let tmp = (i32::from(engine::get_angle_to_grid(n2y as usize, n2x as usize)) + rotate).abs() % 180;
            let diff = (90 - tmp).abs();

            /* If difference is greater then that allowed, skip it,
             * unless it's on the target path */
            if diff >= (degrees + 6) / 4 {
                continue;
            }

            if ag.kill != 0 {
                /* Collect damage, lowered by distance */
                n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam / (r + 1), typ, 0);
            }

            let ag = grid(st, rx, ry);
            if ag.take != 0 && st.takes.takes[usize::from(ag.take)].kidx != 0 {
                n -= borg_launch_destroy_stuff(st, &ag, typ);
            }
        }
    }

    /* Result */
    n
}

/// Simulate/Apply the optimal result of launching an arc/cone/spray
/// (`borg_launch_arc`).
fn borg_launch_arc(st: &mut BorgState, degrees: i32, dam: i32, typ: i32, mut max: i32) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_n: i32 = -1;

    /* (the offsets are never changed: arcs aim at the monster) */
    let b_o_y = 0;
    let b_o_x = 0;
    let mut b_d = max_range();

    if max > 20 {
        max = 20;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        /* Remember how far away the monster is */
        let d = dist_to(st, x, y);

        /* Skip places that are out of range */
        if d > max {
            continue;
        }

        /* Consider it if its a ball spell or right on top of it */
        let n = borg_launch_arc_at_location(st, y, x, degrees, dam, typ, max);

        /* Skip useless attacks */
        if n <= 0 {
            continue;
        }

        /* Collect best attack */
        if b_i >= 0 && n < b_n {
            continue;
        }

        /* Skip attacking farther monster if rewards are equal. */
        if n == b_n && d > b_d {
            continue;
        }

        /* Track it */
        b_i = i - 1;
        b_n = n;
        b_d = d;
    }
    if b_i == -1 {
        return b_n;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Save the location */
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.borg.goal.g.x = tx + b_o_x;
    st.borg.goal.g.y = ty + b_o_y;

    /* Target the location */
    let g = st.borg.goal.g;
    let _ = borg_target(st, g, false);

    /* Result */
    b_n
}

/// Simulate/Apply the optimal result of launching a missile
/// (`borg_attack_aux_launch`).
///
/// When no missile is chosen (every `borg_launch_bolt` finds no target)
/// `b_k` stays -1 and nothing is used.
pub fn borg_attack_aux_launch(st: &mut BorgState) -> i32 {
    let mut b_n = 0;
    let mut b_k: i32 = -1;
    let mut b_v: i32 = -1;

    let bow = st.items.items[(inven_wield() + 1) as usize];

    /* skip if we don't have a bow */
    if bow.iqty == 0 {
        return 0;
    }

    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Scan the quiver */
    for k in quiver_start()..quiver_end() {
        let item = st.items.items[k as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            break;
        }

        /* Skip missiles that don't match the bow */
        if i32::from(item.tval) != tr(st, BI_AMMO_TVAL) {
            continue;
        }

        /* Skip worthless missiles */
        if item.value <= 0 {
            continue;
        }

        /* Determine average damage */
        let mut d = i32::from(item.dd) * (i32::from(item.ds) + 1) / 2;
        d = d + i32::from(item.to_d) + i32::from(bow.to_d);
        d = d * tr(st, BI_AMMO_POWER) * tr(st, BI_SHOTS);

        let v = item.value;

        /* Boost the perceived damage on unID'd ones so he can get a quick
         * pseudoID on it */
        if borg_item_note_needs_id(st, &item) {
            d *= 99;
        }

        /* Paranoia */
        if d <= 0 {
            continue;
        }

        /* Choose optimal target of bolt */
        let power = 6 + 2 * tr(st, BI_AMMO_POWER);
        let n = borg_launch_bolt(st, 0, d, BORG_ATTACK_ARROW as i32, power, k);

        /* if two attacks are equal, pick the cheaper ammo */
        if n == b_n && v >= b_v {
            continue;
        }

        if n >= b_n {
            b_n = n;
            b_v = v;
            b_k = k;
        }
    }

    /* Nothing to use: no candidate missile was chosen, and b_n never goes
     * negative once initialized to zero (candidates only raise it). */
    if b_k < 0 {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Do it */
    let mut m = b"# Firing missile '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[b_k as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Fire */
    borg_keypress(st, key(b'f'));

    /* Use the missile from the quiver */
    borg_keypress(st, ((b_k - quiver_start()) + i32::from(b'0')) as keycode_t);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -2;

    /* Value */
    b_n
}

/// Attempt to rest on the grid to allow the monster to approach me
/// (`borg_attack_aux_rest`). Make sure the monster does not have a ranged
/// attack and that I am inclined to attack him.
fn borg_attack_aux_rest(st: &mut BorgState) -> i32 {
    /* don't wait twice in a row */
    /* this is only checked and reset here which means if something else is */
    /* done before trying an attack this won't get reset. While not perfect */
    /* that should be fine */
    if simulating(st) && st.borg.goal.waiting {
        st.borg.goal.waiting = false;
        return 0;
    }

    let my_danger = danger_here(st, 1, false);

    /* Examine all the monsters */
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        let ok = 'check: {
            /* Skip dead monsters */
            if kill.r_idx == 0 {
                break 'check false;
            }

            /* "player ghosts" */
            if is_ghost(kill.r_idx) {
                break 'check false;
            }

            /* Distance components */
            let ax = if x9 > st.borg.c.x { x9 - st.borg.c.x } else { st.borg.c.x - x9 };
            let ay = if y9 > st.borg.c.y { y9 - st.borg.c.y } else { st.borg.c.y - y9 };

            /* Distance */
            let d = ax.max(ay);

            /* Minimal and maximal distance */
            if d != 2 {
                break 'check false;
            }

            /* Ranged Attacks, don't rest. */
            if kill.ranged_attack != 0 {
                break 'check false;
            }

            /* Skip the sleeping ones */
            if !kill.awake {
                break 'check false;
            }

            /* need to have seen it recently */
            if borg_timer(st, kill.when) > 10 {
                break 'check false;
            }

            /* Skip monsters that dont chase */
            if rf(kill.r_idx, RF_NEVER_MOVE) {
                break 'check false;
            }

            /* Monster better not be faster than me */
            if i32::from(kill.speed) - tr(st, BI_SPEED) >= 5 {
                break 'check false;
            }

            /* Monster can't have a status that prevents approach */
            if kill.afraid || kill.confused || kill.stunned {
                break 'check false;
            }

            /* Should be flowing towards the monster */
            if st.borg.goal.type_0 != GOAL_KILL || i32::from(st.flow.flow_y[0]) != kill.pos.y {
                break 'check false;
            }

            /* Cant have an obstacle between us */
            if !borg_los(st, st.borg.c.y, st.borg.c.x, kill.pos.y, kill.pos.x) {
                break 'check false;
            }

            /* Might be a little dangerous to just wait here */
            if my_danger > tr(st, BI_CURHP) {
                break 'check false;
            }

            true
        };
        if ok {
            break;
        }
        i += 1;
    }

    /* Not a good idea */
    if i >= i32::from(st.kills.nxt) {
        return 0;
    }

    /* Return some value for this rest */
    if simulating(st) {
        return 1;
    }

    /* Rest */
    borg_keypress(st, key(b','));
    let mut m = b"# Waiting for monster '".to_vec();
    m.extend_from_slice(engine::race_name(usize::from(st.kills.kills[i as usize].r_idx)));
    m.extend_from_slice(b"' to approach.");
    borg_note_bytes(st, &m);
    st.borg.goal.waiting = true;

    /* All done */
    1
}

/// Simulate/Apply the optimal result of throwing an object
/// (`borg_attack_aux_object`).
///
/// First choose the "best" object to throw, then check targets.
///
/// C has no `return` at the end (after the keys): the value of a real
/// call is whatever is left in the return register, but no caller uses
/// it (`borg_attack` and the munchkin attacks ignore the result of the
/// real call). Rust returns `b_n`, as the c2rust translation did.
fn borg_attack_aux_object(st: &mut BorgState) -> i32 {
    let mut b_k: i32 = -1;
    let mut b_d: i32 = -1;

    /* Scan the pack */
    for k in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[k as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            continue;
        }

        /* Skip my spell/prayer book */
        if engine::obj_kind_can_browse(item.kind as usize) {
            continue;
        }

        /* Skip "equipment" items (not ammo) */
        if borg_wield_slot(&item) >= 0 {
            continue;
        }

        /* Determine average damage from object */
        let kind = engine::kind(item.kind as usize);
        let d = kind.dd * (kind.ds + 1) / 2;

        /* Skip things that are worth money unless they do a lot of damage */
        if item.value > 100 && d < 5 {
            continue;
        }

        /* Skip useless stuff */
        if d <= 0 {
            continue;
        }

        /* Save Heals and cool stuff */
        if u32::from(item.tval) == TV_POTION {
            continue;
        }

        /* Save last flasks for fuel, if needed */
        if u32::from(item.tval) == TV_FLASK && (tr(st, BI_AFUEL) <= 1 && st.borg.near.unique == 0) {
            continue;
        }

        /* Dont throw wands or rods */
        if u32::from(item.tval) == TV_WAND || u32::from(item.tval) == TV_ROD {
            continue;
        }

        /* Ignore worse damage */
        if b_k >= 0 && d <= b_d {
            continue;
        }

        /* Track */
        b_k = k;
        b_d = d;

        /* (C also computes a throwing range here, `b_r`, which is never
         * used; it only reads `adj_str_blow`) */
        let _ = engine::adj_str_blow(tr(st, BI_STR_INDEX) as usize);
    }

    /* Nothing to use */
    if b_k < 0 {
        return 0;
    }

    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Choose optimal location */
    let power = 6 + 2 * tr(st, BI_AMMO_POWER);
    let b_n = borg_launch_bolt(st, 0, b_d, BORG_ATTACK_ARROW as i32, power, b_k);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Do it */
    let mut m = b"# Throwing painful object '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[b_k as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Fire */
    borg_keypress(st, key(b'v'));

    /* Use the object */
    borg_keypress(st, keycode_t::from(engine::all_letters_nohjkl(b_k)));

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -2;

    /* Value */
    b_n
}

/// The class's main attack spell, which is not penalized for its mana
/// (`primary_spell_for_class`).
fn primary_spell_for_class(st: &BorgState) -> borg_spells {
    match tr(st, BI_CLASS) {
        CLASS_MAGE => MAGIC_MISSILE,
        CLASS_DRUID => STINKING_CLOUD,
        CLASS_PRIEST => ORB_OF_DRAINING,
        CLASS_NECROMANCER => NETHER_BOLT,
        _ => MAGIC_MISSILE,
    }
}

/// "Really penalize use of mana needed for final teleport": the mana each
/// class keeps (`penalty`).
fn class_penalty(st: &BorgState) -> i32 {
    match tr(st, BI_CLASS) {
        CLASS_MAGE => 6,
        CLASS_RANGER => 22,
        CLASS_ROGUE => 20,
        CLASS_PRIEST => 8,
        CLASS_PALADIN => 20,
        CLASS_NECROMANCER => 10,
        _ => 0,
    }
}

/// Simulate/Apply the optimal result of using a "normal" attack spell
/// (`borg_attack_aux_spell_bolt`).
///
/// Take into account the failure rate of spells/objects/etc.  XXX XXX XXX
pub fn borg_attack_aux_spell_bolt(
    st: &mut BorgState,
    spell: borg_spells,
    rad: i32,
    dam: i32,
    typ: i32,
    max_range: i32,
    is_arc: bool,
) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if simulating(st)
        && ((tr(st, BI_CLASS) != CLASS_MAGE && tr(st, BI_CLASS) != CLASS_NECROMANCER)
            && tr(st, BI_CLEVEL) <= 2)
        && engine::randint0(100) < 1
    {
        return 0;
    }

    /* Not if low on food */
    if low_on_food(st) {
        return 0;
    }

    /* Require ability (right now) */
    if !borg_spell_okay_fail(st, spell, if st.borg.near.unique != 0 { 40 } else { 25 }) {
        return 0;
    }

    /* Choose optimal location */
    let mut b_n = if is_arc {
        borg_launch_arc(st, rad, dam, typ, max_range)
    } else {
        borg_launch_bolt(st, rad, dam, typ, max_range, 0)
    };

    let primary = primary_spell_for_class(st);

    /* weak mages need that spell, they don't get penalized */
    /* weak == those that can't teleport reliably anyway */
    if spell == primary
        && (!borg_spell_legal_fail(st, TELEPORT_SELF, 15) || tr(st, BI_MAXCLEVEL) <= 30)
        && simulating(st)
    {
        return b_n;
    }

    /* Penalize mana usage except on MM */
    let spell_power = borg_get_spell_power(st, spell);
    if spell != primary {
        /* Standard penalty */
        b_n -= spell_power;

        /* Extra penalty if the cost far outweighs the damage */
        if tr(st, BI_MAXSP) < 50 && spell_power > b_n {
            b_n -= spell_power;
        }

        /* Penalize use of reserve mana */
        if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 2 {
            b_n -= spell_power * 3;
        }

        /* Penalize use of deep reserve mana */
        if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 3 {
            b_n -= spell_power * 5;
        }
    }

    /* Really penalize use of mana needed for final teleport */
    let penalty = class_penalty(st);
    if tr(st, BI_MAXSP) > 30 && tr(st, BI_CURSP) - spell_power < penalty {
        b_n -= spell_power * 750;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the spell */
    let _ = borg_spell(st, spell);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* Value */
    b_n
}

/// The same as `borg_attack_aux_spell_bolt` only in an emergency case:
/// the borg will enter negative mana casting this
/// (`borg_attack_aux_spell_bolt_reserve`).
fn borg_attack_aux_spell_bolt_reserve(
    st: &mut BorgState,
    spell: borg_spells,
    rad: i32,
    dam: i32,
    typ: i32,
    max_range: i32,
) -> i32 {
    let mut near_monsters = 0;

    /* Fake our Mana */
    let sv_mana = tr(st, BI_CURSP);

    /* Only Weak guys should try this */
    if tr(st, BI_CLEVEL) >= 15 {
        return 0;
    }

    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Not if low on food */
    if tr(st, BI_FOOD) == 0 && (tr(st, BI_ISWEAK) != 0 && borg_spell_legal(st, REMOVE_HUNGER)) {
        return 0;
    }

    /* Must not have enough mana right now */
    if borg_spell_okay_fail(st, spell, 25) {
        return 0;
    }

    /* Must be dangerous */
    if danger_here(st, 1, true) < avoid(st) * 2 {
        return 0;
    }

    /* Find the monster */
    for i in 1..i32::from(st.kills.nxt) {
        let kill = st.kills.kills[i as usize];

        /* Skip dead monsters */
        if kill.r_idx == 0 {
            continue;
        }

        /* check the location */
        let x9 = kill.pos.x;
        let y9 = kill.pos.y;

        /* Distance components */
        let ax = if x9 > st.borg.c.x { x9 - st.borg.c.x } else { st.borg.c.x - x9 };
        let ay = if y9 > st.borg.c.y { y9 - st.borg.c.y } else { st.borg.c.y - y9 };

        /* Distance */
        let d = ax.max(ay);

        /* Count the number of close monsters
         * There should only be one close monster.
         * We do not want to risk fainting.
         */
        if d < 7 {
            near_monsters += 1;

            /* If it has too many hp to be taken out with this */
            /* spell, don't bother trying */
            /* NOTE: the +4 is because the damage is toned down
            as an 'average damage' */
            if i32::from(kill.power) > dam + 4 {
                return 0;
            }

            /* Do not use it in town */
            if tr(st, BI_CDEPTH) == 0 {
                return 0;
            }
        }
    }

    /* Should only be 1 near monster */
    if near_monsters > 1 {
        return 0;
    }

    /* Require ability (with faked mana) */
    st.borg.trait_[BI_CURSP as usize] = tr(st, BI_MAXSP);
    if !borg_spell_okay_fail(st, spell, 25) {
        /* Restore Mana */
        st.borg.trait_[BI_CURSP as usize] = sv_mana;
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_bolt(st, rad, dam, typ, max_range, 0);

    /* return the value */
    if simulating(st) {
        /* Restore Mana */
        st.borg.trait_[BI_CURSP as usize] = sv_mana;
        return b_n;
    }

    /* Cast the spell with fake mana */
    st.borg.trait_[BI_CURSP as usize] = tr(st, BI_MAXSP);
    if borg_spell_fail(st, spell, 25) {
        /* Note the use of the emergency spell */
        borg_note(st, "# Emergency use of an Attack Spell.");

        /* verify use of spell */
        /* borg_keypress('y'); */
    }

    /* Use target */
    borg_queue_direction(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* restore true mana */
    st.borg.trait_[BI_CURSP as usize] = 0;

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using a "dispel" attack spell
/// (`borg_attack_aux_spell_dispel`).
fn borg_attack_aux_spell_dispel(st: &mut BorgState, spell: borg_spells, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Not if low on food */
    if low_on_food(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Require ability */
    if !borg_spell_okay_fail(st, spell, 25) {
        return 0;
    }

    /* Choose optimal location--radius defined as 10 */
    let mut b_n = borg_launch_bolt(st, 10, dam, typ, max_range(), 0);

    let spell_power = borg_get_spell_power(st, spell);

    /* Penalize mana usage */
    b_n -= spell_power;

    /* Penalize use of reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 2 {
        b_n -= spell_power * 3;
    }

    /* Penalize use of deep reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 3 {
        b_n -= spell_power * 5;
    }

    /* Really penalize use of mana needed for final teleport */
    let penalty = class_penalty(st);
    if tr(st, BI_MAXSP) > 30 && tr(st, BI_CURSP) - spell_power < penalty {
        b_n -= spell_power * 750;
    }

    /* never tap if we previously missed a tap */
    if st.fight_attack.target_closest < 0 && BORG_ATTACK_TAP_UNLIFE as i32 == typ && b_n > 0 {
        st.fight_attack.target_closest = 0;
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the prayer */
    let _ = borg_spell(st, spell);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using a "dispel" staff
/// (`borg_attack_aux_staff_dispel`): dispel evil, power, holiness.
/// Genocide handled later.
fn borg_attack_aux_staff_dispel(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* look for the staff */
    if !borg_equips_staff_fail(st, sval) {
        return 0;
    }

    /* Choose optimal location--radius defined as 10 */
    let mut b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Big Penalize charge usage */
    b_n -= 50;

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the prayer */
    let _ = borg_use_staff(st, sval);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using a "normal" attack rod
/// (`borg_attack_aux_rod_bolt`).
fn borg_attack_aux_rod_bolt(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Not likely to be successful in the activation */
    if 500 < borg_activate_failure(st, TV_ROD as i32, sval) {
        return 0;
    }

    /* Look for that rod */
    if !borg_equips_rod(st, sval) {
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Zap the rod */
    let _ = borg_zap_rod(st, sval);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* Value */
    b_n
}

/// `borg_keypress('b' + selection)` for a breath choice (`selection` is
/// -1 for none).
fn select_effect(st: &mut BorgState, selection: i32) {
    if selection != -1 {
        borg_keypress(st, (i32::from(b'b') + selection) as keycode_t);
    }
}

/// Simulate/Apply the optimal result of using a "normal" attack wand
/// (`borg_attack_aux_wand_bolt`).
fn borg_attack_aux_wand_bolt(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32, selection: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Dont use wands in town, charges are too spendy */
    if tr(st, BI_CDEPTH) == 0 {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Look for that wand */
    let i = borg_slot(st, TV_WAND as i32, sval);

    /* None available */
    if i < 0 {
        return 0;
    }

    /* No charges */
    if st.items.items[i as usize].pval == 0 {
        return 0;
    }

    /* Not likely to be successful in the activation */
    if 500 < borg_activate_failure(st, TV_WAND as i32, sval) {
        return 0;
    }

    /* Choose optimal location */
    let mut b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Penalize charge usage */
    if tr(st, BI_CLEVEL) > 5 {
        b_n -= 5;
    }

    /* Wands of wonder are used in last ditch efforts.  They behave
     * randomly, so the best use of them is an emergency.  I have seen
     * borgs die from hill orcs with fully charged wonder wands.  Odds
     * are he could have taken the orcs with the wand.  So use them in
     * an emergency after all the borg_caution() steps have failed
     */
    if sval == st.item_val.sv_wand_wonder && !st.borg.munchkin_mode {
        /* check the danger */
        if b_n > 0 && danger_here(st, 1, true) >= (avoid(st) * 7 / 10) {
            /* make the wand appear deadly */
            b_n = 999;

            /* note the use of the wand in the emergency */
            borg_note(st, "# Emergency use of a Wand of Wonder.");
        } else {
            b_n = 0;
        }
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Aim the wand */
    let _ = borg_aim_wand(st, sval);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /*  select the correct effect */
    select_effect(st, selection);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using an un-id'd wand or rod
/// (`borg_attack_aux_wand_bolt_unknown`, `borg_attack_aux_rod_bolt_unknown`):
/// the same code with the tval, the "tried" test (rods), the command key
/// and the `trying_unknown` flag (wands).
fn borg_attack_aux_bolt_unknown(st: &mut BorgState, dam: i32, typ: i32, rod: bool) -> i32 {
    let mut b_i: i32 = -1;

    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 5) {
        return 0;
    }

    let tval = if rod { TV_ROD } else { TV_WAND };

    /* Look for an un-id'd wand */
    for i in 0..i32::from(engine::z_info().pack_size) {
        let item = &st.items.items[i as usize];
        if item.iqty == 0 {
            continue;
        }

        if u32::from(item.tval) != tval {
            continue;
        }

        /* known */
        if item.aware {
            continue;
        }

        /* No charges */
        if item.pval == 0 {
            continue;
        }

        /* Not an attacker */
        if rod && contains(&desc(item), b"tried") {
            continue;
        }

        /* Select this wand */
        b_i = i;
    }

    /* None available */
    if b_i < 0 {
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_bolt(st, 0, dam, typ, max_range(), 0);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Log the message */
    let mut m = if rod { b"# Aiming unknown rod '".to_vec() } else { b"# Aiming unknown wand '".to_vec() };
    m.extend_from_slice(&desc(&st.items.items[b_i as usize]));
    m.extend_from_slice(b".'");
    borg_note_bytes(st, &m);

    /* Perform the action */
    borg_keypress(st, key(if rod { b'z' } else { b'a' }));
    borg_keypress(st, keycode_t::from(engine::all_letters_nohjkl(b_i)));

    /* Use target */
    borg_keypress(st, key(b'5'));

    if !rod {
        st.borg.trying_unknown = true;
    }

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    if rod {
        st.borg.trying_unknown = true;
    }

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of ACTIVATING an attack artifact
/// (`borg_attack_aux_activation`).
fn borg_attack_aux_activation(
    st: &mut BorgState,
    activation: i32,
    rad: i32,
    dam: i32,
    typ: i32,
    aim: bool,
    selection: i32,
) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Look for and item with that activation and to see if it is charged */
    if !borg_equips_item(st, activation, true) {
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Activate the artifact */
    let _ = borg_activate_item(st, activation);

    /* Use target */
    if aim {
        borg_keypress(st, key(b'5'));

        /* Set our shooting flag */
        st.fight_attack.successful_target = -1;
    }

    /*  select the correct effect */
    select_effect(st, selection);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of ACTIVATING an attack ring
/// (`borg_attack_aux_ring`).
fn borg_attack_aux_ring(st: &mut BorgState, ring_name: i32, rad: i32, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Look for that ring and to see if it is charged */
    if !borg_equips_ring(st, ring_name) {
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Activate the artifact */
    let _ = borg_activate_ring(st, ring_name);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of ACTIVATING a DRAGON ARMOUR
/// (`borg_attack_aux_dragon`).
fn borg_attack_aux_dragon(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32, selection: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    if cant_aim(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, 2) {
        return 0;
    }

    /* Randart dragon armors do not activate for breath */
    if st.items.items[(inven_wield() + 6) as usize].art_idx != 0 {
        return 0;
    }

    /* Look for that scale mail and charged*/
    if !borg_equips_dragon(st, sval) {
        return 0;
    }

    /* Choose optimal location */
    let b_n = borg_launch_arc(st, rad, dam, typ, max_range());

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Activate the scale mail */
    let _ = borg_activate_dragon(st, sval);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /*  select the correct effect */
    select_effect(st, selection);

    /* Value */
    b_n
}

/// trying the Whirlwind Attack spell (`borg_attack_aux_whirlwind_attack`)
fn borg_attack_aux_whirlwind_attack(st: &mut BorgState) -> i32 {
    let mut total_d = 0;

    /* Can I do it */
    if !borg_spell_okay_fail(st, WHIRLWIND_ATTACK, if st.borg.near.unique != 0 { 40 } else { 25 }) {
        return 0;
    }

    let blows = (tr(st, BI_CLEVEL) + 10) / 15;

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        /* Require "adjacent" */
        if dist_to(st, x, y) > 1 {
            continue;
        }

        /* Acquire grid */
        let k = i32::from(grid(st, x, y).kill);

        /* Calculate "average" damage */
        let d = borg_thrust_damage_one(st, k);

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* get to do "blows" attacks */
        let d = d * blows;

        let Some(d) = melee_score(st, k, x, y, d) else {
            continue;
        };

        total_d += d;
    }

    /* Nothing to attack */
    if total_d < 0 {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return total_d;
    }

    /* try the spell */
    if borg_spell(st, WHIRLWIND_ATTACK) {
        return total_d;
    }
    0
}

/// trying the Leap into Battle spell (`borg_attack_aux_leap_into_battle`)
fn borg_attack_aux_leap_into_battle(st: &mut BorgState) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_d: i32 = -1;

    /* Can I do it */
    if !borg_spell_okay_fail(st, LEAP_INTO_BATTLE, if st.borg.near.unique != 0 { 40 } else { 25 }) {
        return 0;
    }

    /* Too afraid to attack */
    if tr(st, BI_ISAFRAID) != 0 || tr(st, BI_CRSFEAR) != 0 {
        return 0;
    }

    if st.fight_attack.target_closest < 10 {
        return 0;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        /* Require up to distance 4 */
        let m_dist = dist_to(st, x, y);
        if m_dist > 4 {
            continue;
        }

        /* Acquire grid */
        let k = i32::from(grid(st, x, y).kill);

        if k == 0 {
            continue;
        }

        /* Calculate "average" damage */
        let mut d = borg_thrust_damage_one(st, k);
        let mut blows = (tr(st, BI_CLEVEL) + 5) / 15;
        blows = ((blows * m_dist + 2) / 4) + 1;
        d *= blows;

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(st.kills.kills[k as usize].r_idx) {
            continue;
        }

        let Some(d) = melee_score(st, k, x, y, d) else {
            continue;
        };

        /* Ignore lower damage */
        if b_i >= 0 && d < b_d {
            continue;
        }

        /* Save the info */
        b_i = i - 1;
        b_d = d;
    }

    /* Nothing to attack */
    if b_i < 0 {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_d;
    }

    /* Save the location */
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.borg.goal.g.x = tx;
    st.borg.goal.g.y = ty;

    let g = st.borg.goal.g;
    let kill = st.kills.kills[usize::from(grid(st, g.x, g.y).kill)];

    /* Note */
    let mut m = b"# Leaping at ".to_vec();
    m.extend_from_slice(borg_race_name(i32::from(kill.r_idx)).to_bytes());
    m.extend_from_slice(
        format!(
            " at ({},{} dist {}) who has {} Hit Points.",
            g.y,
            g.x,
            engine::distance(st.borg.c, g),
            kill.power
        )
        .as_bytes(),
    );
    borg_note_bytes(st, &m);
    let mut m = b"# Attacking with weapon '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[inven_wield() as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Attack the grid */
    let _ = borg_target(st, g, true);
    let _ = borg_spell(st, LEAP_INTO_BATTLE);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* Success */
    b_d
}

/// trying the Maim Foe spell (`borg_attack_aux_maim_foe`): this is a
/// thrust but you get 1 blow/15 levels; it also has a chance to stun but
/// ignoring that for now.
///
/// A real call returns `d`, the score of the *last* target looked at (or
/// -1, or the damage of a skipped one), not the best one's `b_d`, as C.
fn borg_attack_aux_maim_foe(st: &mut BorgState) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_d: i32 = -1;
    let mut d: i32 = -1;

    /* Too afraid to attack */
    if tr(st, BI_ISAFRAID) != 0 || tr(st, BI_CRSFEAR) != 0 {
        return 0;
    }

    /* Can I do it */
    if !borg_spell_okay_fail(st, MAIM_FOE, if st.borg.near.unique != 0 { 40 } else { 25 }) {
        return 0;
    }

    let blows = tr(st, BI_CLEVEL) / 15;

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        /* Require "adjacent" */
        if dist_to(st, x, y) > 1 {
            continue;
        }

        /* Acquire grid */
        let k = i32::from(grid(st, x, y).kill);

        /* Calculate "average" damage */
        d = borg_thrust_damage_one(st, k) * blows;

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(st.kills.kills[k as usize].r_idx) {
            continue;
        }

        let Some(dd) = melee_score(st, k, x, y, d) else {
            continue;
        };
        d = dd;

        /* Ignore lower damage */
        if b_i >= 0 && d < b_d {
            continue;
        }

        /* Save the info */
        b_i = i - 1;
        b_d = d;
    }

    /* Nothing to attack */
    if b_i < 0 {
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_d;
    }

    /* Save the location */
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.borg.goal.g.x = tx;
    st.borg.goal.g.y = ty;

    /* Get a direction for attacking */
    let g = st.borg.goal.g;
    let dir = borg_extract_dir(st.borg.c.y, st.borg.c.x, g.y, g.x);

    /* Simulation */
    if simulating(st) {
        return d;
    }

    let _ = borg_spell(st, MAIM_FOE);
    borg_keypress(st, i2d(dir));

    b_d
}

/// trying the Vampire Strike spell (`borg_attack_aux_vampire_strike`)
fn borg_attack_aux_vampire_strike(st: &mut BorgState) -> i32 {
    let mut b_i: i32 = -1;
    let mut best_dist = max_range();

    /* Can I do it */
    if !borg_spell_okay_fail(st, VAMPIRE_STRIKE, if st.borg.near.unique != 0 { 40 } else { 25 }) {
        return 0;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);

        /* closest distance */
        let dist = dist_to(st, x, y);
        if dist <= best_dist {
            best_dist = dist;
            b_i = i;
        }
        i += 1;
    }

    /* if we didn't find anyone, done */
    if b_i == -1 {
        return 0;
    }

    /* Nothing to attack, require relatively close */
    if best_dist >= 20 {
        return 0;
    }

    let (y, x) = st.flow.temp_at(b_i as usize);

    /* Consider each adjacent spot to the monster */
    /* there must be an empty spot */
    let mut found = false;
    let mut o_x = -1;
    while o_x <= 1 && !found {
        let mut o_y = -1;
        while o_y <= 1 && !found {
            /* but not the monsters location */
            if o_x != 0 || o_y != 0 {
                /* Acquire location */
                let x2 = x + o_x;
                let y2 = y + o_y;

                let ag = grid(st, x2, y2);
                if ag.kill == 0
                    && u32::from(ag.feat) == FEAT_FLOOR
                    && !ag.web
                    && !ag.glyph
                    && (y2 != st.borg.c.y || x2 != st.borg.c.x)
                {
                    found = true;
                }
            }
            o_y += 1;
        }
        o_x += 1;
    }
    /* must have an empty square next to the monster */
    if !found {
        return 0;
    }

    /* Check the projectable, assume unknown grids are walls */
    if !borg_offset_projectable(st, st.borg.c.y, st.borg.c.x, y, x) {
        return 0;
    }

    /* Acquire grid */
    let k = i32::from(grid(st, x, y).kill);

    /* Calculate "average" damage */
    let mut d = tr(st, BI_CLEVEL) * 2;

    /* Obtain the monster */
    let kill = st.kills.kills[k as usize];

    /* "player ghosts" */
    if is_ghost(kill.r_idx) {
        return 0;
    }

    if rf(kill.r_idx, RF_NONLIVING) || rf(kill.r_idx, RF_UNDEAD) {
        return 0;
    }

    /* Avoid waking most "hard" sleeping monsters */
    if !kill.awake && d <= i32::from(kill.power) && !st.borg.munchkin_mode {
        /* Calculate danger */
        let p = borg_danger_one_kill(st, y, x, 1, k, true, true);

        if p > avoid(st) * 2 {
            return 0;
        }
    }

    /* Calculate "danger" to player */
    let mut p = danger_one_here(st, 2, k, true);

    /* Reduce "bonus" of partial kills when higher level */
    if d <= i32::from(kill.power) && tr(st, BI_MAXCLEVEL) > 15 {
        p /= 10;
    }

    /* Add the danger-bonus to the damage */
    d += p;

    /* never vampire strike if we previously missed a vampire strike (or other shot) */
    if st.fight_attack.target_closest < 0 && d > 0 {
        st.fight_attack.target_closest = 0;
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return d;
    }

    /* cast the spell */
    let _ = borg_spell(st, VAMPIRE_STRIKE);

    /* Success */
    d
}

/// The danger at the Borg's grid (radius 4, average) without and with a
/// borg-trait.c spell flag: `(p1, p2)`, the flag cleared afterwards.
fn danger_with_flag(st: &mut BorgState, f: SpellFlag) -> (i32, i32) {
    /* Obtain initial danger */
    set_flag(st, f, false);
    let p1 = danger_here(st, 4, true);

    /* What effect is there? */
    set_flag(st, f, true);
    let p2 = danger_here(st, 4, true);
    set_flag(st, f, false);
    (p1, p2)
}

/// trying the Crush spell (`borg_attack_aux_crush`): right now it is coded
/// so that if a monster is partially crushed, it is still fully a danger
fn borg_attack_aux_crush(st: &mut BorgState) -> i32 {
    /* Can I do it */
    if !borg_spell_okay(st, CRUSH) {
        return 0;
    }

    /* don't kill yourself or leave less than 10hp */
    if (tr(st, BI_CURHP) + 10) < (tr(st, BI_CLEVEL) * 4) {
        return 0;
    }

    let (p1, p2) = danger_with_flag(st, SpellFlag::Crush);

    /* damage is reduction in danger */
    let mut d = p1 - p2;

    /* if there is still danger afterward, make sure the reductioning in HP */
    /* doesn't make this put us in danger */
    let new_hp = tr(st, BI_CURHP) - (tr(st, BI_CLEVEL) * 2);
    if simulating(st) && (p2 >= (new_hp * 2) || new_hp <= 50) {
        return 0;
    }

    let spell_power = borg_get_spell_power(st, CRUSH);

    /* Penalize mana usage */
    d -= spell_power;

    /* Penalize use of reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 2 {
        d -= spell_power * 10;
    }

    /* Simulation */
    if simulating(st) {
        return d;
    }

    /* Cast the spell */
    if borg_spell(st, CRUSH) {
        d
    } else {
        0
    }
}

/// Try to sleep an adjacent bad guy (`borg_attack_aux_trance`).
/// This had been a defense maneuver, which explains the format.
/// This is used for the sleep ii spell and the sanctuary prayer,
/// also the holcolleth activation.
///
/// There is a slight concern with the level of the artifact and the
/// savings throw.  Currently the borg uses his own level to determine
/// the save.  The artifact level may be lower and the borg will have
/// the false impression that spell will work when in fact the monster
/// may easily save against the attack.
fn borg_attack_aux_trance(st: &mut BorgState) -> i32 {
    /* Can I do it */
    if !borg_spell_okay(st, TRANCE) {
        return 0;
    }

    let (p1, p2) = danger_with_flag(st, SpellFlag::SleepII);

    /* value is d, enhance the value for rogues and rangers so that
     * they can use their critical hits.
     */
    let mut d = p1 - p2;

    let spell_power = borg_get_spell_power(st, TRANCE);

    /* Penalize mana usage */
    d -= spell_power;

    /* Penalize use of reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / 2 {
        d -= spell_power * 10;
    }

    /* Simulation */
    if simulating(st) {
        return d;
    }

    /* Cast the spell */
    if borg_spell(st, TRANCE) {
        return d;
    }

    0
}

/// The sleep ii activation of Holcolleth (`borg_attack_aux_artifact_holcolleth`).
fn borg_attack_aux_artifact_holcolleth(st: &mut BorgState) -> i32 {
    if !borg_equips_item(st, st.act.sleepii, true) {
        return 0;
    }

    let (p1, p2) = danger_with_flag(st, SpellFlag::SleepII);

    /* value is d, enhance the value for rogues and rangers so that
     * they can use their critical hits.
     */
    let d = p1 - p2;

    /* Simulation */
    if simulating(st) {
        return d;
    }

    /* Cast the spell */
    if borg_activate_item(st, st.act.sleepii) {
        /* Value */
        d
    } else {
        borg_note(st, "# Failed to properly activate the artifact");
        0
    }
}

/// The breath weapons with several effects: simulate each effect (with
/// `borg_simulate` forced on), keep the first best, and for a real call
/// perform that one (`value[biggest]` in C).
fn best_of(st: &mut BorgState, types: &[u32], mut f: impl FnMut(&mut BorgState, i32, i32) -> i32) -> i32 {
    let mut value = [0i32; 5];
    let mut biggest = 0;
    let tmp_simulate = simulating(st);

    if !simulating(st) {
        st.traits.simulate = true;
    }
    for (x, &t) in types.iter().enumerate() {
        value[x] = f(st, t as i32, x as i32);
    }

    for x in 1..types.len() {
        if value[x] > value[biggest] {
            biggest = x;
        }
    }

    st.traits.simulate = tmp_simulate;
    if !simulating(st) {
        value[biggest] = f(st, types[biggest] as i32, biggest as i32);
    }

    value[biggest]
}

/// Simulate/Apply the optimal result of using the given "type" of attack
/// (`borg_calculate_attack_effectiveness`).
pub fn borg_calculate_attack_effectiveness(st: &mut BorgState, attack_type: i32) -> i32 {
    let cl = tr(st, BI_CLEVEL);
    let mr = max_range();
    let iv = st.item_val;
    let act = st.act;

    /* the wands, staffs, rings and dragon armour of this type */
    let wand = |st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: u32| {
        borg_attack_aux_wand_bolt(st, sval, rad, dam, typ as i32, -1)
    };
    let activation = |st: &mut BorgState, a: i32, rad: i32, dam: i32, typ: u32, aim: bool| {
        borg_attack_aux_activation(st, a, rad, dam, typ as i32, aim, -1)
    };
    let bolt = |st: &mut BorgState, spell: borg_spells, rad: i32, dam: i32, typ: u32, max: i32| {
        borg_attack_aux_spell_bolt(st, spell, rad, dam, typ as i32, max, false)
    };
    let dispel = |st: &mut BorgState, spell: borg_spells, dam: i32, typ: u32| {
        borg_attack_aux_spell_dispel(st, spell, dam, typ as i32)
    };
    let rod = |st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: u32| {
        borg_attack_aux_rod_bolt(st, sval, rad, dam, typ as i32)
    };
    let staff = |st: &mut BorgState, sval: i32, dam: i32, typ: u32| {
        borg_attack_aux_staff_dispel(st, sval, 10, dam, typ as i32)
    };
    let ring = |st: &mut BorgState, sval: i32, dam: i32, typ: u32| {
        borg_attack_aux_ring(st, sval, 2, dam, typ as i32)
    };
    let dragon = |st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: u32| {
        borg_attack_aux_dragon(st, sval, rad, dam, typ as i32, -1)
    };

    /* Analyze */
    match attack_type as u32 {
        /* Wait on grid for monster to approach me */
        BF_REST => borg_attack_aux_rest(st),

        /* Physical attack */
        BF_THRUST => borg_attack_aux_thrust(st),

        /* Fired missile attack */
        BF_LAUNCH => borg_attack_aux_launch(st),

        /* Object attack */
        BF_OBJECT => borg_attack_aux_object(st),

        /* Spell -- slow monster */
        BF_SPELL_SLOW_MONSTER => bolt(st, SLOW_MONSTER, 0, 10, BORG_ATTACK_OLD_SLOW, mr),

        /* Spell -- confuse monster */
        BF_SPELL_CONFUSE_MONSTER => bolt(st, CONFUSE_MONSTER, 0, 10, BORG_ATTACK_OLD_CONF, mr),

        BF_SPELL_SLEEP_III => dispel(st, MASS_SLEEP, 10, BORG_ATTACK_OLD_SLEEP),

        /* Spell -- magic missile */
        BF_SPELL_MAGIC_MISSILE => {
            bolt(st, MAGIC_MISSILE, 0, ((((cl - 1) / 5) + 3) * (4 + 1)) / 2, BORG_ATTACK_MISSILE, mr)
        }

        /* Spell -- magic missile EMERGENCY*/
        BF_SPELL_MAGIC_MISSILE_RESERVE => borg_attack_aux_spell_bolt_reserve(
            st,
            MAGIC_MISSILE,
            0,
            (((cl - 1) / 5) + 3) * (4 + 1),
            BORG_ATTACK_MISSILE as i32,
            mr,
        ),

        /* Spell -- cold bolt */
        BF_SPELL_COLD_BOLT => bolt(st, FROST_BOLT, 0, ((((cl - 5) / 3) + 6) * (8 + 1)) / 2, BORG_ATTACK_COLD, mr),

        /* Spell -- kill wall */
        BF_SPELL_STONE_TO_MUD => bolt(st, TURN_STONE_TO_MUD, 0, 20 + (30 / 2), BORG_ATTACK_KILL_WALL, mr),

        /* Spell -- light beam */
        BF_SPELL_LIGHT_BEAM => bolt(st, SPEAR_OF_LIGHT, -1, 6 * (8 + 1) / 2, BORG_ATTACK_LIGHT_WEAK, mr),

        /* Spell -- stinking cloud */
        BF_SPELL_STINK_CLOUD => bolt(st, STINKING_CLOUD, 2, 10 + (cl / 2), BORG_ATTACK_POIS, mr),

        /* Spell -- fire ball */
        BF_SPELL_FIRE_BALL => bolt(st, FIRE_BALL, 2, cl * 2, BORG_ATTACK_FIRE, mr),

        /* Spell -- Ice Storm */
        BF_SPELL_COLD_STORM => dispel(st, ICE_STORM, (3 * ((cl * 3) + 1)) / 2, BORG_ATTACK_ICE),

        /* Spell -- Meteor Swarm */
        BF_SPELL_METEOR_SWARM => {
            bolt(st, METEOR_SWARM, 1, (30 + cl / 2) + (cl / 20) + 2, BORG_ATTACK_METEOR, mr)
        }

        /* Spell -- Rift */
        BF_SPELL_RIFT => bolt(st, RIFT, -1, (cl * 3) + 40, BORG_ATTACK_GRAVITY, mr),

        /* Spell -- mana storm */
        BF_SPELL_MANA_STORM => bolt(st, MANA_STORM, 3, 300 + (cl * 2), BORG_ATTACK_MANA, mr),

        /* Spell -- Shock Wave */
        BF_SPELL_SHOCK_WAVE => bolt(st, SHOCK_WAVE, 2, cl * 2, BORG_ATTACK_SOUND, mr),

        /* Spell -- Explosion (hack pretend it is all shards) */
        BF_SPELL_EXPLOSION => bolt(st, EXPLOSION, 2, (cl * 2) + (cl / 5), BORG_ATTACK_SHARD, mr),

        /* Prayer -- orb of draining */
        BF_PRAYER_HOLY_ORB_BALL => bolt(
            st,
            ORB_OF_DRAINING,
            if cl >= 30 { 3 } else { 2 },
            ((cl * 3) / 2) + (3 * (6 + 1)) / 2,
            BORG_ATTACK_HOLY_ORB,
            mr,
        ),

        /* Prayer -- blind creature */
        /* Frighten is a fear effect; scoring it as confusion made the
         * Borg expect a confused monster where the game applies fear. */
        BF_SPELL_BLIND_CREATURE => bolt(st, FRIGHTEN, 0, 10, BORG_ATTACK_TURN_ALL, mr),

        /* Druid - Trance */
        BF_SPELL_TRANCE => borg_attack_aux_trance(st),

        /* Prayer -- Dispel Undead */
        BF_PRAYER_DISP_UNDEAD => dispel(st, DISPEL_UNDEAD, ((cl * 5) + 1) / 2, BORG_ATTACK_DISP_UNDEAD),

        /* Prayer -- Dispel Evil */
        BF_PRAYER_DISP_EVIL => dispel(st, DISPEL_EVIL, ((cl * 5) + 1) / 2, BORG_ATTACK_DISP_EVIL),

        /* Prayer -- Dispel Undead */
        BF_PRAYER_DISP_SPIRITS => dispel(st, BANISH_SPIRITS, 100, BORG_ATTACK_DISP_SPIRITS),

        /* Prayer -- Banishment (teleport evil away)*/
        /* This is a defense spell:  done in borg_defense() */

        /* Prayer -- Holy Word also has heal effect and is considered in
         * borg_heal */
        BF_PRAYER_HOLY_WORD => {
            if tr(st, BI_MAXHP) - tr(st, BI_CURHP) >= 300 {
                /* force him to think the spell is more deadly to get him to
                 * cast it.  This will provide some healing for him.
                 */
                dispel(st, HOLY_WORD, cl * 10, BORG_ATTACK_DISP_EVIL)
            } else {
                /* If he is not wounded don't cast this, use Disp Evil
                 * instead. */
                dispel(st, DISPEL_EVIL, ((cl * 3) / 2) - 50, BORG_ATTACK_DISP_EVIL)
            }
        }

        /* Prayer -- Annihilate */
        BF_SPELL_ANNIHILATE => bolt(st, ANNIHILATE, 0, cl * 4, BORG_ATTACK_OLD_DRAIN, mr),

        /* Spell -- Electric Arc */
        BF_SPELL_ELECTRIC_ARC => bolt(st, ELECTRIC_ARC, 0, ((((cl - 1) / 5) + 3) * (6 + 1)) / 2, BORG_ATTACK_ELEC, cl),

        BF_SPELL_ACID_SPRAY => borg_attack_aux_spell_bolt(
            st,
            ACID_SPRAY,
            60,
            ((cl / 2) * (8 + 1)) / 2,
            BORG_ATTACK_ACID as i32,
            10,
            true,
        ),

        /* Spell -- mana bolt */
        BF_SPELL_MANA_BOLT => bolt(st, MANA_BOLT, 0, (cl - 10) * (8 + 1) / 2, BORG_ATTACK_MANA, mr),

        /* Spell -- thrust away */
        BF_SPELL_THRUST_AWAY => bolt(st, THRUST_AWAY, 0, cl * (8 + 1) / 2, BORG_ATTACK_FORCE, (cl / 10) + 1),

        /* Spell -- Lightning Strike (HACK pretend it is all elec) */
        BF_SPELL_LIGHTNING_STRIKE => bolt(
            st,
            LIGHTNING_STRIKE,
            0,
            ((cl / 4) * (4 + 1) / 2) + cl + 5,
            BORG_ATTACK_ELEC_STRIKE,
            mr,
        ),

        /* Spell -- Earth Rising */
        BF_SPELL_EARTH_RISING => bolt(
            st,
            EARTH_RISING,
            0,
            (((cl / 3) + 2) * (6 + 1) / 2) + cl + 5,
            BORG_ATTACK_SHARD,
            (cl / 5) + 4,
        ),

        /* Spell -- Volcanic Eruption */
        /* just count the damage.  The earthquake defense is a side bennie,
         * perhaps... */
        BF_SPELL_VOLCANIC_ERUPTION => {
            bolt(st, VOLCANIC_ERUPTION, 0, (((cl * 3) / 2) * ((cl * 3) + 1)) / 2, BORG_ATTACK_FIRE, mr)
        }

        /* Spell -- River of Lightning */
        BF_SPELL_RIVER_OF_LIGHTNING => borg_attack_aux_spell_bolt(
            st,
            RIVER_OF_LIGHTNING,
            20,
            (cl + 10) * (8 + 1) / 2,
            BORG_ATTACK_PLASMA as i32,
            20,
            true,
        ),

        /* spell -- Spear of Orom(e + diaresis) */
        BF_SPELL_SPEAR_OF_OROME => bolt(st, SPEAR_OF_OROME, 0, ((cl / 2) + (8 + 1)) / 2, BORG_ATTACK_HOLY_ORB, mr),

        /* spell -- Light of Varda */
        BF_SPELL_LIGHT_OF_VARDA => bolt(st, LIGHT_OF_VARDA, 0, cl * 5 + 100, BORG_ATTACK_LIGHT, mr),

        /* spell -- Nether Bolt */
        BF_SPELL_NETHER_BOLT => bolt(st, NETHER_BOLT, 0, (((cl / 4) + 3) * (4 + 1)) / 2, BORG_ATTACK_NETHER, mr),

        /* spell -- Tap Unlife */
        BF_SPELL_TAP_UNLIFE => dispel(st, TAP_UNLIFE, (((cl / 4) + 3) * (4 + 1)) / 2, BORG_ATTACK_TAP_UNLIFE),

        /* Spell - Crush */
        BF_SPELL_CRUSH => borg_attack_aux_crush(st),

        BF_SPELL_SLEEP_EVIL => dispel(st, SLEEP_EVIL, cl * 10 + 500, BORG_ATTACK_SLEEP_EVIL),

        /* spell -- Disenchant */
        BF_SPELL_DISENCHANT => bolt(st, DISENCHANT, 0, ((((cl * 2) + 10) + 1) / 2) * 2, BORG_ATTACK_DISEN, mr),

        /* spell -- Frighten */
        BF_SPELL_FRIGHTEN => bolt(st, FRIGHTEN, 0, cl, BORG_ATTACK_TURN_ALL, mr),

        /* Spell - Vampire Strike*/
        BF_SPELL_VAMPIRE_STRIKE => borg_attack_aux_vampire_strike(st),

        /* Spell - Dispel Life */
        BF_PRAYER_DISPEL_LIFE => bolt(st, DISPEL_LIFE, 0, ((cl * 3) + 1) / 2, BORG_ATTACK_DRAIN_LIFE, mr),

        /* spell -- Dark Spear */
        BF_SPELL_DARK_SPEAR => bolt(st, DARK_SPEAR, 0, (((cl * 2) + 1) / 2) * 2, BORG_ATTACK_DARK, mr),

        /* spell -- Unleash Chaos */
        BF_SPELL_UNLEASH_CHAOS => bolt(st, UNLEASH_CHAOS, 0, ((cl + 1) / 2) * 8, BORG_ATTACK_CHAOS, mr),

        /* Spell -- Storm of Darkness */
        BF_SPELL_STORM_OF_DARKNESS => bolt(st, STORM_OF_DARKNESS, 4, (((cl * 2) + 1) / 2) * 4, BORG_ATTACK_DARK, mr),

        /* Spell - Curse */
        BF_SPELL_CURSE => {
            /* costs 100hp to cast.  Don't kill yourself doing it */
            if tr(st, BI_CURHP) < 120 {
                return 0;
            }
            bolt(st, CURSE, 0, -1, BORG_ATTACK_CURSE, mr)
        }

        /* spell - Whirlwind Attack */
        BF_SPELL_WHIRLWIND_ATTACK => borg_attack_aux_whirlwind_attack(st),

        /* spell - Leap into Battle */
        BF_SPELL_LEAP_INTO_BATTLE => borg_attack_aux_leap_into_battle(st),

        /* spell - Maim Foe */
        BF_SPELL_MAIM_FOE => borg_attack_aux_maim_foe(st),

        /* spell - Howl of the Damned */
        BF_SPELL_HOWL_OF_THE_DAMNED => dispel(st, HOWL_OF_THE_DAMNED, cl, BORG_ATTACK_TURN_ALL),

        /* ROD -- slow monster */
        BF_ROD_SLOW_MONSTER => rod(st, iv.sv_rod_slow_monster, 0, 10, BORG_ATTACK_OLD_SLOW),

        /* ROD -- sleep monster */
        BF_ROD_SLEEP_MONSTER => rod(st, iv.sv_rod_sleep_monster, 0, 10, BORG_ATTACK_OLD_SLEEP),

        /* Rod -- elec bolt */
        BF_ROD_ELEC_BOLT => rod(st, iv.sv_rod_elec_bolt, -1, 6 * (6 + 1) / 2, BORG_ATTACK_ELEC),

        /* Rod -- cold bolt */
        BF_ROD_COLD_BOLT => rod(st, iv.sv_rod_cold_bolt, 0, 12 * (8 + 1) / 2, BORG_ATTACK_COLD),

        /* Rod -- acid bolt */
        BF_ROD_ACID_BOLT => rod(st, iv.sv_rod_acid_bolt, 0, 12 * (8 + 1) / 2, BORG_ATTACK_ACID),

        /* Rod -- fire bolt */
        BF_ROD_FIRE_BOLT => rod(st, iv.sv_rod_fire_bolt, 0, 12 * (8 + 1) / 2, BORG_ATTACK_FIRE),

        /* Rod -- light beam */
        BF_ROD_LIGHT_BEAM => rod(st, iv.sv_rod_light, -1, 6 * (8 + 1) / 2, BORG_ATTACK_LIGHT_WEAK),

        /* Rod -- drain life */
        BF_ROD_DRAIN_LIFE => rod(st, iv.sv_rod_drain_life, 0, 150, BORG_ATTACK_OLD_DRAIN),

        /* Rod -- elec ball */
        BF_ROD_ELEC_BALL => rod(st, iv.sv_rod_elec_ball, 2, 64, BORG_ATTACK_ELEC),

        /* Rod -- acid ball */
        BF_ROD_COLD_BALL => rod(st, iv.sv_rod_cold_ball, 2, 100, BORG_ATTACK_COLD),

        /* Rod -- acid ball */
        BF_ROD_ACID_BALL => rod(st, iv.sv_rod_acid_ball, 2, 120, BORG_ATTACK_ACID),

        /* Rod -- fire ball */
        BF_ROD_FIRE_BALL => rod(st, iv.sv_rod_fire_ball, 2, 144, BORG_ATTACK_FIRE),

        /* Rod -- unid'd rod */
        BF_ROD_UNKNOWN => borg_attack_aux_bolt_unknown(st, 75, BORG_ATTACK_MISSILE as i32, true),

        /* Wand -- unid'd wand */
        BF_WAND_UNKNOWN => borg_attack_aux_bolt_unknown(st, 75, BORG_ATTACK_MISSILE as i32, false),

        /* Wand -- magic missile */
        BF_WAND_MAGIC_MISSILE => wand(st, iv.sv_wand_magic_missile, 0, 3 * (4 + 1) / 2, BORG_ATTACK_MISSILE),

        /* Wand -- slow monster */
        BF_WAND_SLOW_MONSTER => wand(st, iv.sv_wand_slow_monster, 0, 10, BORG_ATTACK_OLD_SLOW),

        /* Wand -- sleep monster */
        BF_WAND_HOLD_MONSTER => wand(st, iv.sv_wand_hold_monster, 0, 10, BORG_ATTACK_OLD_SLEEP),

        /* Wand -- fear monster */
        BF_WAND_FEAR_MONSTER => wand(st, iv.sv_wand_fear_monster, 0, 2 * (6 + 1) / 2, BORG_ATTACK_TURN_ALL),

        /* Wand -- conf monster */
        BF_WAND_CONFUSE_MONSTER => wand(st, iv.sv_wand_confuse_monster, 0, 2 * (6 + 1) / 2, BORG_ATTACK_OLD_CONF),

        /* Wand -- elec bolt */
        BF_WAND_ELEC_BOLT => wand(st, iv.sv_wand_elec_bolt, -1, 6 * (6 + 1) / 2, BORG_ATTACK_ELEC),

        /* Wand -- cold bolt */
        BF_WAND_COLD_BOLT => wand(st, iv.sv_wand_cold_bolt, 0, 12 * (8 + 1) / 2, BORG_ATTACK_COLD),

        /* Wand -- acid bolt */
        BF_WAND_ACID_BOLT => wand(st, iv.sv_wand_acid_bolt, 0, 5 * (8 + 1) / 2, BORG_ATTACK_ACID),

        /* Wand -- fire bolt */
        BF_WAND_FIRE_BOLT => wand(st, iv.sv_wand_fire_bolt, 0, 12 * (8 + 1) / 2, BORG_ATTACK_FIRE),

        /* Wand -- light beam */
        BF_WAND_LIGHT_BEAM => wand(st, iv.sv_wand_light, -1, 6 * (8 + 1) / 2, BORG_ATTACK_LIGHT_WEAK),

        /* Wand -- stinking cloud */
        BF_WAND_STINKING_CLOUD => wand(st, iv.sv_wand_stinking_cloud, 2, 12, BORG_ATTACK_POIS),

        /* Wand -- elec ball */
        BF_WAND_ELEC_BALL => wand(st, iv.sv_wand_elec_ball, 2, 64, BORG_ATTACK_ELEC),

        /* Wand -- acid ball */
        BF_WAND_COLD_BALL => wand(st, iv.sv_wand_cold_ball, 2, 100, BORG_ATTACK_COLD),

        /* Wand -- acid ball */
        BF_WAND_ACID_BALL => wand(st, iv.sv_wand_acid_ball, 2, 120, BORG_ATTACK_ACID),

        /* Wand -- fire ball */
        BF_WAND_FIRE_BALL => wand(st, iv.sv_wand_fire_ball, 2, 144, BORG_ATTACK_FIRE),

        /* Wand -- dragon cold */
        BF_WAND_DRAGON_COLD => wand(st, iv.sv_wand_dragon_cold, 3, 160, BORG_ATTACK_COLD),

        /* Wand -- dragon fire */
        BF_WAND_DRAGON_FIRE => wand(st, iv.sv_wand_dragon_fire, 3, 200, BORG_ATTACK_FIRE),

        /* Wand -- annihilation */
        BF_WAND_ANNIHILATION => wand(st, iv.sv_wand_annihilation, 0, 250, BORG_ATTACK_OLD_DRAIN),

        /* Wand -- drain life */
        BF_WAND_DRAIN_LIFE => wand(st, iv.sv_wand_drain_life, 0, 150, BORG_ATTACK_OLD_DRAIN),

        /* Wand -- wand of wonder */
        BF_WAND_WONDER => wand(st, iv.sv_wand_wonder, 0, 35, BORG_ATTACK_MISSILE),

        /* Staff -- Sleep Monsters */
        BF_STAFF_SLEEP_MONSTERS => staff(st, iv.sv_staff_sleep_monsters, 60, BORG_ATTACK_OLD_SLEEP),

        /* Staff -- Slow Monsters */
        BF_STAFF_SLOW_MONSTERS => staff(st, iv.sv_staff_slow_monsters, 60, BORG_ATTACK_OLD_SLOW),

        /* Staff -- Dispel Evil */
        BF_STAFF_DISPEL_EVIL => staff(st, iv.sv_staff_dispel_evil, 60, BORG_ATTACK_DISP_EVIL),

        /* Staff -- Power */
        BF_STAFF_POWER => staff(st, iv.sv_staff_power, 120, BORG_ATTACK_TURN_ALL),

        /* Staff -- holiness */
        BF_STAFF_HOLINESS => {
            let dam = if tr(st, BI_CURHP) < tr(st, BI_MAXHP) / 2 { 500 } else { 120 };
            staff(st, iv.sv_staff_holiness, dam, BORG_ATTACK_DISP_EVIL)
        }

        /* Artifact -- Narthanc- fire bolt 9d8*/
        BF_ACT_FIRE_BOLT => activation(st, act.fire_bolt, 0, 9 * (8 + 1) / 2, BORG_ATTACK_FIRE, true),

        /* Artifact -- Anduril & Firestar- fire ball 72*/
        BF_ACT_FIRE_BALL72 => activation(st, act.fire_ball72, 2, 72, BORG_ATTACK_FIRE, true),

        /* Artifact -- Gothmog- FIRE BALL 144 */
        BF_ACT_FIRE_BALL => activation(st, act.fire_ball, 2, 144, BORG_ATTACK_FIRE, true),

        /* Artifact -- Nimthanc & Paurnimmen- frost bolt 6d8*/
        BF_ACT_COLD_BOLT => activation(st, act.cold_bolt, 0, 6 * (8 + 1) / 2, BORG_ATTACK_COLD, true),

        /* Artifact -- Belangil- frost ball 50 */
        BF_ACT_COLD_BALL50 => activation(st, act.cold_ball50, 2, 50, BORG_ATTACK_COLD, true),

        /* Artifact -- Aranr(u + acute accent)th- frost bolt 12d8*/
        BF_ACT_COLD_BOLT2 => activation(st, act.cold_bolt2, 0, 12 * (8 + 1) / 2, BORG_ATTACK_COLD, true),

        /* Artifact -- Ringil- frost ball 100*/
        BF_ACT_COLD_BALL100 => activation(st, act.cold_ball100, 2, 100, BORG_ATTACK_COLD, true),

        /* Artifact -- Dethanc- electric bolt 6d6*/
        BF_ACT_ELEC_BOLT => activation(st, act.elec_bolt, -1, 6 * (6 + 1) / 2, BORG_ATTACK_ELEC, true),

        /* Artifact -- Rilia- poison gas 12*/
        BF_ACT_STINKING_CLOUD => activation(st, act.stinking_cloud, 2, 12, BORG_ATTACK_POIS, true),

        /* Artifact -- Theoden- drain Life 120*/
        BF_ACT_DRAIN_LIFE2 => activation(st, act.drain_life2, 0, 120, BORG_ATTACK_OLD_DRAIN, true),

        /* Artifact -- Totila- confustion */
        BF_ACT_CONFUSE2 => activation(st, act.confuse2, 0, 20, BORG_ATTACK_OLD_CONF, true),

        /* Artifact -- Holcolleth -- sleep ii and sanctuary */
        BF_ACT_SLEEPII => borg_attack_aux_artifact_holcolleth(st),

        /* Artifact -- TURMIL- drain life 90 */
        BF_ACT_DRAIN_LIFE1 => activation(st, act.drain_life1, 0, 90, BORG_ATTACK_OLD_DRAIN, true),

        /* Artifact -- Fingolfin- spikes 150 */
        BF_ACT_ARROW => activation(st, act.arrow, 0, 150, BORG_ATTACK_MISSILE, true),

        /* Artifact -- Cammithrim- Magic Missile 3d4 */
        BF_ACT_MISSILE => activation(st, act.missile, 0, 3 * (4 + 1) / 2, BORG_ATTACK_MISSILE, true),

        /* Artifact -- Paurnen- ACID bolt 5d8 */
        BF_ACT_ACID_BOLT => activation(st, act.acid_bolt, 0, 5 * (8 + 1) / 2, BORG_ATTACK_ACID, true),

        /* Artifact -- INGWE- DISPEL EVIL X5 */
        BF_ACT_DISPEL_EVIL => activation(st, act.dispel_evil, 10, 10 + (cl * 5) / 2, BORG_ATTACK_DISP_EVIL, true),

        /* Artifact -- E(o + diaresis)l -- Mana Bolt 12d8 */
        BF_ACT_MANA_BOLT => activation(st, act.mana_bolt, 0, (12 * (8 + 1)) / 2, BORG_ATTACK_MANA, true),

        /* Artifact -- Razorback and Mediator */
        BF_ACT_STAR_BALL => activation(st, act.star_ball, 3, 150, BORG_ATTACK_ELEC, true),

        /* Artifact -- Gil-galad */
        BF_ACT_STARLIGHT2 => activation(st, act.starlight2, 7, (10 * (8 + 1)) / 2, BORG_ATTACK_LIGHT, false),

        /* Artifact -- randarts */
        BF_ACT_STARLIGHT => activation(st, act.starlight, 7, (6 * (8 + 1)) / 2, BORG_ATTACK_LIGHT, false),

        BF_ACT_MON_SLOW => activation(st, act.mon_slow, 0, 20, BORG_ATTACK_OLD_SLOW, true),

        BF_ACT_MON_CONFUSE => activation(st, act.mon_confuse, 0, 2 * (6 + 1) / 2, BORG_ATTACK_OLD_CONF, true),

        BF_ACT_SLEEP_ALL => activation(st, act.sleep_all, 0, 60, BORG_ATTACK_OLD_SLEEP, true),

        BF_ACT_FEAR_MONSTER => activation(st, act.mon_scare, 0, 2 * (6 + 1) / 2, BORG_ATTACK_TURN_ALL, true),

        BF_ACT_LIGHT_BEAM => activation(st, act.light_line, -1, 6 * (8 + 1) / 2, BORG_ATTACK_LIGHT_WEAK, true),

        BF_ACT_DRAIN_LIFE3 => activation(st, act.drain_life3, 0, 150, BORG_ATTACK_OLD_DRAIN, true),

        BF_ACT_DRAIN_LIFE4 => activation(st, act.drain_life4, 0, 250, BORG_ATTACK_OLD_DRAIN, true),

        BF_ACT_ELEC_BALL => activation(st, act.elec_ball, 2, 64, BORG_ATTACK_ELEC, true),

        BF_ACT_ELEC_BALL2 => activation(st, act.elec_ball2, 2, 250, BORG_ATTACK_ELEC, true),

        BF_ACT_ACID_BOLT2 => activation(st, act.acid_bolt2, 0, 10 * (8 + 1) / 2, BORG_ATTACK_ACID, true),

        BF_ACT_ACID_BOLT3 => activation(st, act.acid_bolt3, 0, 12 * (8 + 1) / 2, BORG_ATTACK_ACID, true),

        BF_ACT_ACID_BALL => activation(st, act.acid_ball, 2, 120, BORG_ATTACK_ACID, true),

        BF_ACT_COLD_BALL160 => activation(st, act.cold_ball160, 2, 160, BORG_ATTACK_COLD, true),

        BF_ACT_COLD_BALL2 => activation(st, act.cold_ball2, 2, 200, BORG_ATTACK_COLD, true),

        BF_ACT_FIRE_BALL2 => activation(st, act.fire_ball2, 2, 120, BORG_ATTACK_FIRE, true),

        BF_ACT_FIRE_BALL200 => activation(st, act.fire_ball200, 2, 200, BORG_ATTACK_FIRE, true),

        BF_ACT_FIRE_BOLT2 => activation(st, act.fire_bolt2, 0, 12 * (8 + 1) / 2, BORG_ATTACK_FIRE, true),

        BF_ACT_FIRE_BOLT3 => activation(st, act.fire_bolt3, 0, 16 * (8 + 1) / 2, BORG_ATTACK_FIRE, true),

        BF_ACT_DISPEL_EVIL60 => activation(st, act.dispel_evil60, 10, 60, BORG_ATTACK_DISP_EVIL, false),

        BF_ACT_DISPEL_UNDEAD => activation(st, act.dispel_undead, 10, 60, BORG_ATTACK_DISP_UNDEAD, false),

        BF_ACT_DISPEL_ALL => activation(st, act.dispel_all, 10, 60, BORG_ATTACK_DISP_ALL, false),

        BF_ACT_LOSSLOW => activation(st, act.losslow, 10, 20, BORG_ATTACK_OLD_SLOW, false),

        BF_ACT_LOSSLEEP => activation(st, act.lossleep, 10, 20, BORG_ATTACK_OLD_SLEEP, false),

        BF_ACT_LOSCONF => activation(st, act.losconf, 10, 5 + ((5 + 1) / 2), BORG_ATTACK_OLD_CONF, false),

        /* (rad is still 0 here) */
        BF_ACT_WONDER => activation(st, act.wonder, 0, 5 + ((5 + 1) / 2), BORG_ATTACK_MISSILE, true),

        BF_ACT_STAFF_HOLY => {
            let dam = if tr(st, BI_CURHP) < tr(st, BI_MAXHP) / 2 { 500 } else { 120 };
            /* The effect floods the area with goodness: an area-of-sight
             * dispel, not a single-target bolt. */
            activation(st, act.staff_holy, 10, dam, BORG_ATTACK_DISP_EVIL, false)
        }

        BF_ACT_RING_ACID => activation(st, act.ring_acid, 2, 70, BORG_ATTACK_ACID, true),

        BF_ACT_RING_FIRE => activation(st, act.ring_flames, 2, 80, BORG_ATTACK_FIRE, true),

        BF_ACT_RING_ICE => activation(st, act.ring_ice, 2, 75, BORG_ATTACK_ICE, true),

        BF_ACT_RING_LIGHTNING => activation(st, act.ring_lightning, 2, 85, BORG_ATTACK_ELEC, true),

        BF_ACT_DRAGON_BLUE => activation(st, act.dragon_blue, 2, 150, BORG_ATTACK_ELEC, true),

        BF_ACT_DRAGON_GREEN => activation(st, act.dragon_green, 2, 150, BORG_ATTACK_POIS, true),

        BF_ACT_DRAGON_RED => activation(st, act.dragon_red, 2, 200, BORG_ATTACK_FIRE, true),

        BF_ACT_DRAGON_MULTIHUED => best_of(
            st,
            &[BORG_ATTACK_ELEC, BORG_ATTACK_COLD, BORG_ATTACK_ACID, BORG_ATTACK_POIS, BORG_ATTACK_FIRE],
            |st, t, x| borg_attack_aux_activation(st, act.dragon_multihued, 2, 250, t, true, x),
        ),

        BF_ACT_DRAGON_GOLD => activation(st, act.dragon_gold, 2, 150, BORG_ATTACK_SOUND, true),

        BF_ACT_DRAGON_CHAOS => best_of(st, &[BORG_ATTACK_CHAOS, BORG_ATTACK_DISEN], |st, t, x| {
            borg_attack_aux_activation(st, act.dragon_chaos, 2, 220, t, true, x)
        }),

        BF_ACT_DRAGON_LAW => best_of(st, &[BORG_ATTACK_SOUND, BORG_ATTACK_SHARD], |st, t, x| {
            borg_attack_aux_activation(st, act.dragon_law, 2, 220, t, true, x)
        }),

        BF_ACT_DRAGON_BALANCE => best_of(
            st,
            &[BORG_ATTACK_CHAOS, BORG_ATTACK_DISEN, BORG_ATTACK_SOUND, BORG_ATTACK_SHARD],
            |st, t, x| borg_attack_aux_activation(st, act.dragon_balance, 2, 250, t, true, x),
        ),

        BF_ACT_DRAGON_SHINING => best_of(st, &[BORG_ATTACK_LIGHT, BORG_ATTACK_DARK], |st, t, x| {
            borg_attack_aux_activation(st, act.dragon_shining, 2, 200, t, true, x)
        }),

        BF_ACT_DRAGON_POWER => activation(st, act.dragon_power, 2, 300, BORG_ATTACK_MISSILE, true),

        /* Ring of ACID */
        BF_RING_ACID => ring(st, iv.sv_ring_acid, 70, BORG_ATTACK_ACID),

        /* Ring of FLAMES */
        BF_RING_FIRE => ring(st, iv.sv_ring_flames, 80, BORG_ATTACK_FIRE),

        /* Ring of ICE */
        BF_RING_ICE => ring(st, iv.sv_ring_ice, 75, BORG_ATTACK_ICE),

        /* Ring of LIGHTNING */
        BF_RING_LIGHTNING => ring(st, iv.sv_ring_lightning, 85, BORG_ATTACK_ELEC),

        /* Dragon Scale Mail can be activated as well */
        BF_DRAGON_BLUE => dragon(st, iv.sv_dragon_blue, 20, 150, BORG_ATTACK_ELEC),

        BF_DRAGON_WHITE => dragon(st, iv.sv_dragon_white, 20, 100, BORG_ATTACK_COLD),

        BF_DRAGON_BLACK => dragon(st, iv.sv_dragon_black, 20, 120, BORG_ATTACK_ACID),

        BF_DRAGON_GREEN => dragon(st, iv.sv_dragon_green, 20, 150, BORG_ATTACK_POIS),

        BF_DRAGON_RED => dragon(st, iv.sv_dragon_red, 2, 200, BORG_ATTACK_FIRE),

        BF_DRAGON_MULTIHUED => best_of(
            st,
            &[BORG_ATTACK_ELEC, BORG_ATTACK_COLD, BORG_ATTACK_ACID, BORG_ATTACK_POIS, BORG_ATTACK_FIRE],
            |st, t, x| borg_attack_aux_dragon(st, iv.sv_dragon_multihued, 20, 250, t, x),
        ),

        BF_DRAGON_GOLD => dragon(st, iv.sv_dragon_gold, 20, 150, BORG_ATTACK_SOUND),

        BF_DRAGON_CHAOS => best_of(st, &[BORG_ATTACK_CHAOS, BORG_ATTACK_DISEN], |st, t, x| {
            borg_attack_aux_dragon(st, iv.sv_dragon_chaos, 20, 220, t, x)
        }),

        BF_DRAGON_LAW => best_of(st, &[BORG_ATTACK_SOUND, BORG_ATTACK_SHARD], |st, t, x| {
            borg_attack_aux_dragon(st, iv.sv_dragon_law, 20, 220, t, x)
        }),

        BF_DRAGON_BALANCE => best_of(
            st,
            &[BORG_ATTACK_CHAOS, BORG_ATTACK_DISEN, BORG_ATTACK_SOUND, BORG_ATTACK_SHARD],
            |st, t, x| borg_attack_aux_dragon(st, iv.sv_dragon_balance, 20, 250, t, x),
        ),

        BF_DRAGON_SHINING => best_of(st, &[BORG_ATTACK_LIGHT, BORG_ATTACK_DARK], |st, t, x| {
            borg_attack_aux_dragon(st, iv.sv_dragon_shining, 20, 200, t, x)
        }),

        BF_DRAGON_POWER => dragon(st, iv.sv_dragon_power, 20, 300, BORG_ATTACK_MISSILE),

        /* Oops */
        _ => 0,
    }
}

/// Attack nearby monsters, in the best possible way, if any
/// (`borg_attack`).
///
/// We consider a variety of possible attacks, including physical attacks
/// on adjacent monsters, missile attacks on nearby monsters, spell/prayer
/// attacks on nearby monsters, and wand/rod attacks on nearby monsters.
///
/// Basically, for each of the known "types" of attack, we "simulate" the
/// "optimal" result of using that attack, and then we "apply" the "type"
/// of attack which appears to have the "optimal" result.
///
/// When calculating the "result" of using an attack, we only consider the
/// effect of the attack on visible, on-screen, known monsters, which are
/// within 16 grids of the player.  This prevents most "spurious" attacks,
/// but we can still be fooled by situations like creeping coins which die
/// while out of sight, leaving behind a pile of coins, which we then find
/// again, and attack with distance attacks, which have no effect.  Perhaps
/// we should "expect" certain results, and take note of failure to observe
/// those effects.  XXX XXX XXX
pub fn borg_attack(st: &mut BorgState, boosted_bravery: bool) -> bool {
    let mut b_n = 0;
    let mut b_g: i32 = -1;
    let mut adjacent_monster = false;

    /* Nobody around */
    if st.kills.cnt == 0 {
        return false;
    }

    /* Set the attacking flag so that danger is boosted for monsters */
    /* we want to attack first. */
    st.traits.attacking = true;

    /* Reset list */
    st.flow.temp_n = 0;

    /* Find "nearby" monsters */
    let mut i = 1;
    while i < i32::from(st.kills.nxt) {
        let k = i;
        i += 1;

        /* Monster */
        let kill = st.kills.kills[k as usize];
        let r = kill.r_idx;

        /* Skip dead monsters */
        if r == 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(r) {
            continue;
        }

        /* Require current knowledge */
        if borg_timer(st, kill.when) > 2 {
            continue;
        }

        /* Ignore multiplying monsters and when fleeing from scaries*/
        if st.borg.goal.ignoring && tr(st, BI_ISAFRAID) == 0 && rf(r, RF_MULTIPLY) {
            continue;
        }

        /* Low level mages need to conserve the mana in town. These guys
         * don't fight back */
        let name = engine::race_name(usize::from(r));
        if tr(st, BI_CLASS) == CLASS_MAGE
            && tr(st, BI_MAXCLEVEL) < 10
            && tr(st, BI_CDEPTH) == 0
            && contains(name, b"Farmer")
        {
            continue;
        }

        /* Check if there is a monster adjacent to me or he's close and
         * fast. */
        if (i32::from(kill.speed) > tr(st, BI_SPEED) && engine::distance(st.borg.c, kill.pos) <= 2)
            || engine::distance(st.borg.c, kill.pos) <= 1
        {
            adjacent_monster = true;
        }

        /* no attacking most scaryguys, try to get off the level */
        if st.borg.mon.scary {
            if contains(name, b"Grip") || contains(name, b"Fang") {
                /* probably Grip or Fang. Try to fight Grip and Fang. */
            } else if tr(st, BI_CDEPTH) <= 5 && tr(st, BI_CDEPTH) != 0 && rf(r, RF_MULTIPLY) {
                /* Try to fight single worms and mice. */
            } else if borg_timer(st, st.borg.time.level) >= 2000 || borg_timer(st, st.borg.time.town) >= 3000 {
                /* Try to fight been there too long. */
            } else if boosted_bravery
                || st.borg.no_retreat >= 1
                || st.borg.goal.recalling != 0
                || st.borg.goal.descending != 0
            {
                /* Try to fight if being Boosted or recall engaged. */
                borg_note(st, "# Bored, or recalling and fighting a monster on Scaryguy Level.");
            } else if tr(st, BI_CDEPTH) * 4 <= tr(st, BI_CLEVEL) && tr(st, BI_CLEVEL) > 10 {
                /* Try to fight anyway. */
                borg_note(st, "# High clevel fighting monster on Scaryguy Level.");
            } else if adjacent_monster {
                /* Try to fight if there is a monster next to me */
                borg_note(st, "# Adjacent to monster on Scaryguy Level.");
            } else {
                /* Flee from other scary guys */
                continue;
            }
        }

        /* Get grid */
        let ag = grid(st, kill.pos.x, kill.pos.y);

        /* Never shoot off-screen */
        if ag.info & BORG_OKAY == 0 {
            continue;
        }

        /* Never shoot through walls */
        if ag.info & BORG_VIEW == 0 {
            continue;
        }

        /* Check the distance XXX XXX XXX */
        if engine::distance(st.borg.c, kill.pos) > max_range() {
            continue;
        }

        /* Sometimes the borg can lose a monster index in the grid if there
         * are lots of monsters on screen.  If he does lose one, reinject the
         * index here. */
        if ag.kill == 0 {
            st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = k as u8;
        }

        /* Save the location (careful) */
        st.flow.temp_push(kill.pos.y, kill.pos.x);
    }

    /* No destinations */
    if st.flow.temp_n == 0 {
        st.traits.attacking = false;
        return false;
    }

    /* Simulate */
    st.traits.simulate = true;

    /* some attacks are only possible for random artifacts */
    /* those are lumped at the end so people without random artifacts */
    /* don't look for them. */
    let max_attacks = if engine::player_opt(OPT_birth_randarts) {
        BF_MAX as i32
    } else {
        BF_ACT_STARLIGHT as i32
    };

    /* Analyze the possible attacks */
    for g in 0..max_attacks {
        /* Simulate */
        let n = borg_calculate_attack_effectiveness(st, g);

        /* Track "best" attack  <= */
        if n <= b_n {
            continue;
        }

        /* Track best */
        b_g = g;
        b_n = n;
    }

    /* Nothing good */
    if b_n <= 0 {
        st.traits.attacking = false;
        return false;
    }

    /* Note */
    borg_note(st, &format!("# Performing attack type {} with value {}.", b_g, b_n));

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    let _ = borg_calculate_attack_effectiveness(st, b_g);

    st.traits.attacking = false;

    /* Success */
    true
}

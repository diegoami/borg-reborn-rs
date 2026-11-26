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

// borg-reborn-rs (#240): named attack policy; default operations unchanged.
use crate::params::{value as param, Param};
use crate::borg::BorgTime;
use crate::borg_cave::{BorgGrid, DUNGEON_HGT, DUNGEON_WID};
use crate::borg_cave_util::{borg_cave_floor_grid, borg_panel_hgt, borg_panel_wid};
use crate::borg_danger::{borg_danger, borg_danger_one_kill};
use crate::borg_flow_kill::{borg_race_name, BorgKill};
use crate::borg_flow_misc::borg_extract_dir;
use crate::borg_inventory::{borg_slot, borg_wield_slot};
use crate::borg_io::{borg_keypress, borg_note, borg_note_bytes, borg_queue_direction};
use crate::borg_item::{inven_wield, quiver_end, quiver_start};
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_item_use::{
    borg_activate_dragon, borg_activate_failure, borg_activate_item, borg_activate_ring,
    borg_aim_wand, borg_equips_dragon, borg_equips_item, borg_equips_ring, borg_equips_rod,
    borg_equips_staff_fail, borg_quaff_potion, borg_use_staff, borg_zap_rod,
};
use crate::borg_magic::{
    borg_get_spell_power, borg_spell, borg_spell_fail, borg_spell_legal, borg_spell_legal_fail,
    borg_spell_okay, borg_spell_okay_fail, borg_spell_fail_rate,
};
use crate::borg_projection::{
    borg_distance, borg_inc_motion, borg_los, borg_los_projectable, borg_offset_projectable, borg_projectable_pure,
    borg_target,
};
use crate::borg_unseen_belief::BelievedTarget;
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
    TV_BOOTS, TV_FLASK, TV_HAFTED, TV_POLEARM, TV_POTION, TV_RING, TV_ROD, TV_SWORD, TV_WAND,
};
use crate::headers::list_object_flags_h::{OF_EXPLODE, OF_THROWING};
use crate::headers::ui_event_h::keycode_t;
use crate::headers::z_type_h::loc;
use crate::state::BorgState;

/// The module's globals (borg-fight-attack.c).
#[derive(Clone)]
pub struct FightAttackState {
    /// #760/#759: a focused damage planner; None retains the native planner.
    pub focused_target: Option<usize>,
    pub focused_kind: i32,
    /// #879: shots at a believed (unseen) monster since it was last seen, by
    /// kill index, with the record's `when` at the time (a new sighting or a
    /// reused index starts again).
    pub unseen_shots: std::collections::BTreeMap<usize, (BorgTime, i32)>,
    pub(crate) pending_unseen: crate::borg_unseen_belief::PendingAttack,
    /// borg-reborn-rs (#889): while a blast scores a grid, how many grids
    /// its reach exceeds that grid's distance from the blast's centre (0
    /// elsewhere): a believed monster that moved fewer grids is still hit
    /// (`unseen_spread`).
    pub(crate) unseen_blast_slack: i32,
    /// borg-reborn-rs (#889, Codex re-review of #1128): grids of records a
    /// ball could still reach but nothing else may aim at (`blast_only`).
    /// Filled by `borg_attack` apart from its aim list (`st.flow.temp_*`),
    /// read only by ball-shaped scoring (`choose_bolt`, `ball_shape`).
    pub(crate) blast_aims: Vec<(i32, i32)>,
    /// borg-reborn-rs (#889): whether the attack rated now projects a ball
    /// (not a dragon's breath cone), so a ball's slack and `blast_aims` apply.
    pub(crate) ball_shape: bool,
    /// borg-reborn-rs (#889, review of #1128 F3): Light Room cast at a
    /// believed record, by kill index, with the record's `when` at the time
    /// (once per sighting).
    pub(crate) unseen_reveals: std::collections::BTreeMap<usize, BorgTime>,
    /// `successful_target`: zero is "haven't shot", positive is "good to
    /// shoot current target", negative is "shot and missed", -12 and lower
    /// is "do not shoot".
    pub successful_target: i32,
    /// `target_closest`
    pub target_closest: i32,
    /// `borg_tp_other_n`: the monsters in the line of a Teleport Other
    pub tp_other_n: i16,
    /// borg-reborn-rs (#584): the true damage (no kill bonus, no danger
    /// bonus) of the attack being rated: summed over the monsters one aim
    /// hits (`dmg_acc`), of the best aim or target (`dmg_best`).
    pub dmg_acc: i32,
    pub dmg_best: i32,
    /// borg-reborn-rs (#625): what the attack being rated costs, set when a
    /// spell or a device passes its fail check (`AttackCost`); `None` for an
    /// attack with no fail chance or use limit (melee, missiles, throws).
    pub cost: Option<AttackCost>,
    /// borg-reborn-rs (#625): for `borg_attack_estimate`, the winner's cost
    /// and the best true damage of an attack that is not a spell (the
    /// fallback once the mana runs out).
    pub est_cost: Option<AttackCost>,
    pub est_fallback: i32,
    /// borg-reborn-rs (#556): the kills of the same aim or target: for each
    /// monster hit, its true damage over its HP, at most 1 (`kill_acc`, of the
    /// best aim `kill_best`); and the winner's, for `borg_attack_estimate`
    /// (`est_kills`).
    pub kill_acc: f64,
    pub kill_best: f64,
    pub est_kills: f64,
    /// borg-reborn-rs (#637): a wand is being rated in town: only a
    /// dangerous town monster is worth its charge.
    pub town_wand: bool,
    /// borg-reborn-rs (#568): a throw is being rated with
    /// `attack_throw_as_melee`: the engine's throw damage, hit chance and
    /// melee's danger term (`borg_launch_damage_one`,
    /// `borg_launch_bolt_aux_hack`).
    pub throwing: bool,
    /// borg-reborn-rs (#867): the attack being rated is aimed by a confused
    /// Borg under `attack_confused_by_direction`: only an adjacent monster,
    /// no offset aim, and its damage weighted by the aim chance
    /// (`CONFUSED_AIM`). Cleared by `borg_calculate_attack_effectiveness`.
    pub confused_aim: bool,
    /// borg-reborn-rs (#867): the direction (1-9) a confused aimed attack is
    /// sent with instead of the target key; 0 = none.
    pub confused_dir: i32,
    /// `borg_tp_other_x`
    pub tp_other_x: [u8; 255],
    /// `borg_tp_other_y`
    pub tp_other_y: [u8; 255],
    /// `borg_tp_other_index`
    pub tp_other_index: [i32; 255],
}

/// borg-reborn-rs (#625): an attack's cost for the Phase Door rule's fight
/// test: its fail chance and how many uses the Borg can pay for now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackCost {
    /// The chance it fails, in percent (a spell's fail rate; a device's
    /// `borg_activate_failure` per mille, divided by 10).
    pub fail_pct: i32,
    /// The uses left: casts the current mana pays for (a spell), charges (a
    /// wand), one (a rod: it recharges over many turns); `None`: no limit
    /// within a fight (a staff, an activation).
    pub uses: Option<i32>,
    /// A spell: after its last cast the Borg falls back to an attack that is
    /// not a spell.
    pub spell: bool,
}

impl Default for FightAttackState {
    fn default() -> Self {
        FightAttackState {
            focused_target: None,
            focused_kind: 0,
            unseen_shots: std::collections::BTreeMap::new(),
            pending_unseen: crate::borg_unseen_belief::PendingAttack::default(),
            unseen_blast_slack: 0,
            blast_aims: Vec::new(),
            ball_shape: false,
            unseen_reveals: std::collections::BTreeMap::new(),
            successful_target: 0,
            target_closest: 0,
            tp_other_n: 0,
            dmg_acc: 0,
            dmg_best: 0,
            cost: None,
            est_cost: None,
            est_fallback: 0,
            kill_acc: 0.0,
            kill_best: 0.0,
            est_kills: 0.0,
            town_wand: false,
            throwing: false,
            confused_aim: false,
            confused_dir: 0,
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

/// borg-reborn-rs (#867, owner: "it should choose the best attack available,
/// and in case of mage without spells, that would be the flask"): with
/// `attack_confused_by_direction`, a Borg that is confused, but neither blind
/// nor hallucinating, may still throw, shoot and use a wand, rod or staff at
/// an adjacent monster. The engine allows them: it refuses only spells
/// (`player_can_cast`, player-util.c:1102) and scrolls (:1182) to a confused
/// player. 0 = C (no item attack while confused).
fn confused_by_direction(st: &BorgState) -> bool {
    by_direction(
        param(Param::attack_confused_by_direction) != 0,
        tr(st, BI_ISBLIND) != 0,
        tr(st, BI_ISCONFUSED) != 0,
        tr(st, BI_ISIMAGE) != 0,
    )
}

/// `confused_by_direction` on the switch and the three states.
fn by_direction(on: bool, blind: bool, confused: bool, image: bool) -> bool {
    on && confused && !blind && !image
}

/// `cant_aim`, but for the item attacks (missiles, throws, wands, rods,
/// staffs), which `attack_confused_by_direction` allows while confused.
fn cant_use_item_attack(st: &BorgState) -> bool {
    cant_aim(st) && !confused_by_direction(st)
}

/// borg-reborn-rs (#867): the chance a confused player's direction is kept,
/// as a fraction: `player_confuse_dir` (player-util.c:1353-1377) keeps a
/// direction key 25% of the time and otherwise draws one of the 8 directions
/// (`ddd[randint0(8)]`), so 1/4 + 3/4 * 1/8 = 11/32. Melee (walking into the
/// monster, cmd-cave.c:1301), missiles (player-attack.c:1350), throws (:1393)
/// and aimed devices (cmd-obj.c:440) all pass through it. A target
/// (direction 5) is always redrawn, so a confused aimed attack is sent by
/// direction key. A game constant.
const CONFUSED_AIM: (i32, i32) = (11, 32);

/// borg-reborn-rs (#867): mark the attack being rated as aimed by a confused
/// Borg, when `attack_confused_by_direction` applies (`confused_aim`).
fn mark_confused_aim(st: &mut BorgState) {
    if confused_by_direction(st) {
        st.fight_attack.confused_aim = true;
    }
}

/// borg-reborn-rs (#867, review of #868): once an attack is sent, its
/// confused aim is over; nothing after it may read the flag.
fn clear_confused_aim(st: &mut BorgState) {
    st.fight_attack.confused_aim = false;
    st.fight_attack.confused_dir = 0;
}

/// borg-reborn-rs (#867): a confused aimed attack's value weighted by
/// `CONFUSED_AIM`, and its damage and kills too when `damage_too`; anything
/// else unchanged. `borg_attack` weights a blow's value only (review of
/// #868): its damage is what the fight-or-flee tests read
/// (`borg_attack_estimate`), and C rated a confused blow at full damage, so
/// a confused fight that melee wins is judged as in C. The focused ranking
/// ranks on the damage, so it weights it for every aimed attack.
fn confused_weight(st: &mut BorgState, raw: i32, damage_too: bool) -> i32 {
    if !st.fight_attack.confused_aim {
        return raw;
    }
    let (num, den) = CONFUSED_AIM;
    if damage_too {
        st.fight_attack.dmg_best = st.fight_attack.dmg_best * num / den;
        st.fight_attack.kill_best *= f64::from(num) / f64::from(den);
    }
    raw * num / den
}

/// The key that answers an aimed command's direction prompt: the target
/// (`5`), or, for a confused aimed attack (#867), the direction toward the
/// adjacent monster, which the engine keeps more often (`CONFUSED_AIM`).
/// Returns the `successful_target` to set: `shot` after a target, 0 after a
/// direction (no target to track, as `throw_at_unseen_bump`).
fn aim_key(st: &mut BorgState, shot: i32) -> i32 {
    let dir = std::mem::take(&mut st.fight_attack.confused_dir);
    if dir != 0 {
        borg_note(st, &format!("# Confused: aiming by direction {dir}, not at a target (#867)"));
    }
    let (k, flag) = aim_answer(dir, shot);
    borg_keypress(st, k);
    flag
}

/// `aim_key`'s key and `successful_target` for a direction (0: none).
fn aim_answer(dir: i32, shot: i32) -> (keycode_t, i32) {
    if dir == 0 { (key(b'5'), shot) } else { (i2d(dir), 0) }
}

/// "Paranoia": `borg_simulate && randint0(100) < pct` (the RNG is drawn
/// only while simulating).
fn paranoia(st: &mut BorgState, pct: i32) -> bool {
    st.fight_attack.focused_target.is_none() && simulating(st) && engine::randint0(100) < pct
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

/// borg-reborn-rs (#618, #621, owner): a monster on the scary list's ignore
/// entries (`harness/scary_monsters.txt`, flag `ignore`) up to its level for
/// the class: no attack on it is worth anything, and the hunt skips it. One
/// in the Borg's way is still attacked (the walk step is unchanged). The
/// race's base name is read only for a base entry (the race copy is not
/// free, and this runs on every damage estimate).
pub(crate) fn ignored(st: &BorgState, r_idx: usize) -> bool {
    if crate::borg_town_hunt::avoids_maggot(st, r_idx) { return true; }
    let Some(list) = crate::scary_list::selected() else { return false };
    let who = crate::scary_list::Situation {
        clevel: tr(st, BI_CLEVEL),
        class: tr(st, BI_CLASS),
        breeder_count: 0,
        breeder_stand: false,
        levels_less: 0,
    };
    crate::scary_list::is_ignored(list, r_idx, || engine::race_base_name(r_idx), who)
}

/// borg-reborn-rs (#619): a dangerous town monster: a town race (level 0)
/// with a blow that steals (gold or an item) or hurts for at least
/// `attack_town_dangerous_min_damage` (dice x sides): the battle-scarred
/// veteran, the mean-looking mercenary, the squint-eyed rogue, the filthy
/// street urchin; not the cats, dogs and beggars.
pub(crate) fn dangerous_town_race(r_idx: usize) -> bool {
    if engine::race(r_idx).level != 0 {
        return false;
    }
    let min = param(Param::attack_town_dangerous_min_damage);
    (0..usize::from(engine::z_info().mon_blows_max))
        .map_while(|k| engine::race_blow(r_idx, k))
        .any(|b| dangerous_blow(borg_mon_blow_effect(b.effect_name.to_bytes()) as u32, i32::from(b.dice) * i32::from(b.sides), min))
}

fn dangerous_blow(effect: u32, max_damage: i32, min_damage: i32) -> bool {
    effect == MONBLOW_EAT_GOLD || effect == MONBLOW_EAT_ITEM || (effect == MONBLOW_HURT && max_damage >= min_damage)
}

/// borg-reborn-rs (#619, owner): with `attack_town_dangerous_on_sight`, in
/// town, a dangerous town monster is killed on sight: it is the preferred
/// target and the hunt walks to it. The scare is the scary list's: while the
/// town is scary (below a listed monster's level for the class) the Borg
/// flees as before; from that level on it kills on sight.
pub(crate) fn town_kill_on_sight(st: &BorgState, r_idx: usize) -> bool {
    param(Param::attack_town_dangerous_on_sight) != 0 && tr(st, BI_CDEPTH) == 0 && dangerous_town_race(r_idx)
}

/// borg-reborn-rs (#619): the target bonus for a monster killed on sight.
fn town_kill_bonus(st: &BorgState, r_idx: usize, dam: i32) -> i32 {
    if town_kill_on_sight(st, r_idx) { dam * param(Param::attack_town_dangerous_bonus_percent) / 100 } else { dam }
}

/// borg-reborn-rs (#637): with `attack_town_dangerous_bonus_whole`, the
/// bonus also multiplies the danger term an attack's value adds (melee and
/// bolts), so a dangerous town monster is the target for certain, not only
/// when its damage term wins.
fn town_danger_bonus(st: &BorgState, r_idx: usize, p: i32) -> i32 {
    if param(Param::attack_town_dangerous_bonus_whole) != 0 { town_kill_bonus(st, r_idx, p) } else { p }
}

/// borg-reborn-rs (#637): with `attack_town_dangerous_caster_walk`, a Mage
/// or Necromancer below the level of `borg_flow_kill`'s caster rule walks to
/// a dangerous town monster too (its town hunt is otherwise breeders only,
/// and no town race breeds).
pub(crate) fn town_caster_walk() -> bool {
    param(Param::attack_town_dangerous_caster_walk) != 0
}

/// borg-reborn-rs (#637): with `attack_town_dangerous_wand`, a wand may be
/// aimed in town when a dangerous town monster (killed on sight) is known;
/// only those are worth a charge (`town_wand` in `borg_launch_damage_one`).
/// Known means seen within `attack_max_target_age`, as `borg_attack`'s own
/// targets (review of #653, finding 3: a record can outlive the monster).
fn town_wand_allowed(st: &BorgState) -> bool {
    param(Param::attack_town_dangerous_wand) != 0
        && (1..usize::try_from(st.kills.nxt).unwrap_or(0)).any(|k| {
            let kill = &st.kills.kills[k];
            kill.r_idx != 0
                && borg_timer(st, kill.when) <= param(Param::attack_max_target_age)
                && town_kill_on_sight(st, usize::from(kill.r_idx))
        })
}

/// Guess how much damage a physical attack will do to a monster
/// (`borg_thrust_damage_one`).
///
/// The C consumer made this non-static for its improvement unit I-001
/// (better-borg-c `4.2.6-impact-v10`): `borg_caution()`'s
/// `borg_adjacent_kill_in_one_blow()` needs the melee damage of a full
/// round against a tracked kill.
pub fn borg_thrust_damage_one(st: &BorgState, i: i32) -> i32 {
    thrust_damage(st, i, 1)
}

fn thrust_damage(st: &BorgState, i: i32, spell_blows: i32) -> i32 {
    let kind = st.fight_attack.focused_target.map(|_| st.fight_attack.focused_kind);
    thrust_damage_with(st, i, spell_blows, kind)
}

fn thrust_damage_with(st: &BorgState, i: i32, spell_blows: i32, focused_kind: Option<i32>) -> i32 {
    /* Examine current weapon */
    let item = &st.items.items[inven_wield() as usize];

    /* Monster record */
    let kill = &st.kills.kills[i as usize];

    // borg-reborn-rs (#879): no blow at a monster the Borg only believes in.
    if crate::borg_unseen_belief::believed_weight(st, i as usize).is_some() {
        return 0;
    }

    /* Monster race */
    let r = kill.r_idx;

    /* "player ghosts" */
    if is_ghost(r) {
        return 0;
    }
    let r_ptr = engine::race(usize::from(r));

    /* Damage */
    // borg-reborn-rs (#713): no blow at an immobile creature kept at range
    // (`caution_ranged_vs_immobile` 2, 3): it is left, or attacked from afar.
    if crate::borg_caution::kept_at_range(st, usize::from(r)) {
        return 0;
    }

    let focused = focused_kind.is_some();
    let unit = if focused { 1000 } else { 1 };
    let mut dam = i32::from(item.dd) * (i32::from(item.ds) + 1) * unit / 2;

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
    dam += i32::from(item.to_d) * unit;

    /* add player bonuses */
    dam += tr(st, BI_TODAM) * unit;

    let focused_blow_damage = dam;
    /* multiply the damage for the whole round of attacks */
    dam *= tr(st, BI_BLOWS);

    /* Bonuses for combat */
    let mut chance = tr(st, BI_THN) + ((tr(st, BI_TOHIT) + i32::from(item.to_h)) * 3);
    if focused {
        // player-attack.c:99-109,196-219: real hit chance, no targeting
        // or class preference bonuses. Millihitpoints preserve fractions.
        chance += 3 * engine::player_blessed_weapon_bonus();
        if tr(st, BI_ISBLIND) != 0 { chance /= 2; }
        // Fixed spell blows share a capped HP budget BEFORE hit chance.
        // This credits their count without treating separate chances of an
        // overkill hit as guaranteed damage (effect-handler-attack.c:1940-1974).
        let hp = if st.fight_attack.focused_target.is_some() {
            focused_power(st, kill.power)
        } else {
            observed_kill_hp(kill).unwrap_or(i32::from(kill.power).max(1)) * 1000
        };
        let per_blow = focused_blow_expectation(focused_blow_damage, hp,
            hit_permyriad(chance, r_ptr.ac), spell_blows);
        // Ordinary melee takes fractional energy. Weapon spells specify
        // their own number of py_attack_real blows and take a full action.
        let rate = if focused_kind == Some(BF_THRUST as i32) { engine::player_attack_rates().0 } else { 100 * spell_blows.max(0) };
        return (per_blow * i64::from(rate.max(0)) / 100) as i32;
    }

    /* Chance of hitting the monsters AC */
    if chance < (r_ptr.ac * 3 / 4) * param(Param::attack_melee_min_hit_tenths) / 10 {
        dam = 0;
    }

    /* 5% automatic success/fail */
    chance = chance.clamp(5, 95);

    /* add 10% to chance to give a bit more weight to weapons */
    if tr(st, BI_CLEVEL) > param(Param::attack_melee_bonus_min_level) {
        chance += param(Param::attack_melee_hit_bonus);
    }

    /* Mages with Mana do not get that bonus, they should cast */
    if (tr(st, BI_CLASS) == CLASS_MAGE || tr(st, BI_CLASS) == CLASS_NECROMANCER)
        && tr(st, BI_CURSP) > param(Param::attack_mage_melee_min_sp)
    {
        chance -= param(Param::attack_mage_hit_penalty);
    }

    /* reduce damage by the % chance to hit */
    dam = (dam * chance) / 100;

    /* Try to place a minimal amount of damage */
    if dam <= 0 {
        dam = 1;
    }

    /* Limit damage to twice maximal hitpoints */
    if dam > i32::from(kill.power) * param(Param::attack_melee_overkill_multiplier) && !rf(r, RF_UNIQUE) {
        dam = i32::from(kill.power) * param(Param::attack_melee_overkill_multiplier);
    }

    /*
     * Reduce the damage if a mage, they should not melee if they can avoid it
     */
    if (tr(st, BI_CLASS) == CLASS_MAGE || tr(st, BI_CLASS) == CLASS_NECROMANCER)
        && tr(st, BI_MAXCLEVEL) < param(Param::attack_mage_melee_max_level)
        && tr(st, BI_CURSP) > param(Param::attack_mage_melee_min_sp)
    {
        dam = (dam * param(Param::attack_mage_melee_damage_tenths) / 10) + 1;
    }

    /*
     * Enhance the perceived damage on Uniques.  This way we target them
     * Keep in mind that he should hit the uniques but if he has a
     * x5 great bane of dragons, he will tend attack the dragon since the
     * perceived (and actual) damage is higher.  But don't select
     * the town uniques (maggot does no damage)
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
        dam += dam * param(Param::attack_melee_unique_bonus);
    }

    /* Ignore Maggot until later.  Player will chase Maggot
     * down all across the screen waking up all the monsters.  Then
     * he is stuck in a compromised situation.
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) == 0 {
        dam = dam * param(Param::attack_melee_town_unique_numerator) / param(Param::attack_melee_town_unique_divisor);

        /* Dont hunt maggot until later */
        if tr(st, BI_CLEVEL) < param(Param::attack_town_unique_min_level) {
            dam = 0;
        }
    }
    if ignored(st, usize::from(r)) {
        dam = 0;
    }
    // borg-reborn-rs (#619): a dangerous town monster comes first.
    dam = town_kill_bonus(st, usize::from(r), dam);

    /* give a small bonus for whacking a breeder */
    if rf(r, RF_MULTIPLY) {
        dam = dam * param(Param::attack_melee_breeder_numerator) / param(Param::attack_melee_breeder_divisor);
    }

    /* Enhance the perceived damage to summoner in order to influence the
     * choice of targets.
     */
    if is_summoner(r, false) {
        dam += (dam * param(Param::attack_melee_summoner_numerator)) / param(Param::attack_melee_summoner_divisor);
    }

    /*
     * Apply massive damage bonus to Questor monsters to
     * encourage borg to strike them.
     */
    if rf(r, RF_QUESTOR) {
        dam += dam * param(Param::attack_melee_questor_bonus);
    }

    /* Damage */
    dam
}

/// The usual melee target scoring of thrust-like attacks: for an adjacent
/// (or reachable) monster with damage `d` at `(x, y)`, `None` to skip it
/// ("hard" sleeping monsters, sleeping town monsters), else the damage
/// plus the danger bonus.
fn melee_score(st: &mut BorgState, k: i32, x: i32, y: i32, mut d: i32) -> Option<i32> {
    if st.fight_attack.focused_target.is_some() { return Some(d); }
    /* Avoid waking most "hard" sleeping monsters */
    if !st.kills.kills[k as usize].awake && d <= power(st, k as usize) {
        /* Calculate danger */
        let p = borg_danger_one_kill(st, y, x, param(Param::attack_melee_sleep_danger_horizon), k, true, true);

        if p > avoid(st) * param(Param::attack_melee_sleep_danger_multiplier) {
            return None;
        }
    }

    /* Ignore sleeping town monsters */
    if tr(st, BI_CDEPTH) == 0 && !st.kills.kills[k as usize].awake {
        return None;
    }

    /* Calculate "danger" to player */
    let mut p = danger_one_here(st, param(Param::attack_melee_danger_horizon), k, true);

    /* Reduce "bonus" of partial kills when higher level */
    if d <= power(st, k as usize) && tr(st, BI_MAXCLEVEL) > param(Param::attack_melee_partial_min_level) {
        p /= param(Param::attack_melee_partial_danger_divisor);
    }

    // borg-reborn-rs (#637): the town bonus on the danger term too, after
    // the partial-kill division above (the bonus multiplies what is added).
    p = town_danger_bonus(st, usize::from(st.kills.kills[k as usize].r_idx), p);

    /* Add the danger-bonus to the damage */
    d += p;
    Some(d)
}

/// Simulate/Apply the optimal result of making a physical attack
/// (`borg_attack_aux_thrust`).
/// `borg_spell_okay_fail`; when it passes, the spell's cost is noted for the
/// attack being rated (#625: its fail rate, and the casts the current mana
/// pays for). Draws no random numbers.
fn spell_okay_fail_noted(st: &mut BorgState, spell: borg_spells, allow_fail: i32) -> bool {
    if !borg_spell_okay_fail(st, spell, allow_fail) {
        return false;
    }
    let power = borg_get_spell_power(st, spell).max(1);
    st.fight_attack.cost = Some(AttackCost {
        fail_pct: borg_spell_fail_rate(st, spell).clamp(0, 100),
        uses: Some(tr(st, BI_CURSP) / power),
        spell: true,
    });
    true
}

/// borg-reborn-rs (#625, review of #657): the best true damage among the
/// candidates that need no mana (`(attack, damage)`), the winner excluded,
/// so a wand never outlives its charges as its own fallback.
fn fallback_without(no_mana: &[(i32, i32)], winner: i32) -> i32 {
    no_mana.iter().filter(|&&(g, _)| g != winner).map(|&(_, d)| d).max().unwrap_or(0)
}

/// A device's cost for the attack being rated (#625): `failure` is
/// `borg_activate_failure`'s per mille; `uses` the charges (a wand), one (a
/// rod), or `None` (no limit within a fight).
fn note_device_cost(st: &mut BorgState, failure: i32, uses: Option<i32>) {
    st.fight_attack.cost = Some(AttackCost { fail_pct: (failure / 10).clamp(0, 100), uses, spell: false });
}

/// borg-reborn-rs (#556): the share of a monster of `hp` that `damage`
/// kills: damage over HP, at most 1 (0 for no HP known).
pub(crate) fn kill_fraction(damage: i32, hp: i32) -> f64 {
    if hp <= 0 {
        return 0.0;
    }
    (f64::from(damage.max(0)) / f64::from(hp)).min(1.0)
}

fn borg_attack_aux_thrust(st: &mut BorgState) -> i32 {
    let mut b_i: i32 = -1;
    let mut b_d: i32 = -1;

    /* Too afraid to attack */
    if tr(st, BI_ISAFRAID) != 0 || tr(st, BI_CRSFEAR) != 0 {
        return 0;
    }

    // borg-reborn-rs (#867): a confused blow goes astray as often as a
    // confused throw; weighted alike, so the best attack wins on damage.
    mark_confused_aim(st);

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

        let true_d = d;
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
        // borg-reborn-rs (#584): the true damage, without the unique
        // targeting bonus of `borg_thrust_damage_one`.
        st.fight_attack.dmg_best = if st.fight_attack.focused_target.is_none() && rf(st.kills.kills[k as usize].r_idx, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
            true_d / (1 + param(Param::attack_melee_unique_bonus).max(0))
        } else {
            true_d
        };
        st.fight_attack.kill_best = kill_fraction(st.fight_attack.dmg_best, i32::from(st.kills.kills[k as usize].power));
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
    i32::from(level) > if cl < param(Param::attack_status_low_level_cutoff) { param(Param::attack_status_low_level_limit) } else { (((cl - param(Param::attack_status_level_offset)) / param(Param::attack_status_level_divisor)) * param(Param::attack_status_level_numerator)) + param(Param::attack_status_level_base) }
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
    let mut p1 = danger_one_here(st, param(Param::attack_status_danger_horizon), i, true);
    /* Make certain monsters appear to have more danger so the borg is more
     * likely to use this attack */
    if afraid_bonus && st.kills.kills[i as usize].afraid && tr(st, BI_CLEVEL) <= param(Param::attack_fear_bonus_max_level) {
        p1 += param(Param::attack_fear_danger_bonus);
    }
    set_flag(st, f, true);
    let p2 = danger_one_here(st, param(Param::attack_status_danger_horizon), i, true);
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
    if st.fight_attack.focused_target.is_some_and(|k| k != i as usize) {
        return 0;
    }
    if st.fight_attack.focused_target.is_some() {
        return focused_launch_damage(st, i as usize, dam, typ, ammo_location);
    }
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
    let junk_potion = is_potion_throw(st, typ, ammo_location);
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

        // borg-reborn-rs (#568): a throw, with `attack_throw_as_melee`: no
        // launcher, the engine's brand multiplier for a throw (`1 + brand`,
        // `ranged_damage`) and the expected damage over the engine's hit
        // chance, as melee scales by its own (`borg_thrust_damage_one`).
        BORG_ATTACK_ARROW if st.fight_attack.throwing => {
            let mult = borg_best_mult(&ammo, r);
            if mult > 1 {
                dam *= 1 + mult;
            }
            let throwing = engine::flag_has(&ammo.flags, OF_THROWING as i32);
            let bonus = i32::from(ammo.to_h) + if throwing { tr(st, BI_TOHIT) } else { 0 };
            let divisor = if junk_potion { 1 } else {
                throw_point_blank_divisor(param(Param::attack_throw_point_blank_divisor),
                    point_blank_divisor(tr(st, BI_CLASS)))
            };
            dam = rated_throw_damage(dam, cur_dis, rf(r, RF_UNIQUE), divisor,
                throw_hit_percent(tr(st, BI_THT), bonus, throwing, cur_dis, r_ptr.ac));
        }

        BORG_ATTACK_ARROW => {
            let mut mult = borg_best_mult(&bow, r);
            mult = mult.max(borg_best_mult(&ammo, r));
            dam *= mult;
            /* don't point blank non-uniques */
            if cur_dis == 1 && !rf(r, RF_UNIQUE) {
                dam /= point_blank_divisor(tr(st, BI_CLASS));
            }
            /* Do I hit regularly? (80%)*/
            if chance < armor * param(Param::attack_arrow_min_hit_tenths) / 10 {
                dam = 0;
            }
        }

        /* Pure damage */
        BORG_ATTACK_MANA => {
            if st.borg.near.unique != 0
                && st.borg.has[st.item_val.kv_potion_restore_mana as usize] > param(Param::attack_mana_unique_min_potions)
            {
                dam *= param(Param::attack_mana_unique_multiplier);
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
                dam /= param(Param::attack_nether_point_blank_divisor);
            }
            if rf(r, RF_UNDEAD) || rf(r, RF_DEMON) || strchr_egv(r_ptr.d_char as i32) { // borg-reborn-rs (#823): wchar_t is 16 bits on Windows
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
                if i32::from(kill.injury) >= param(Param::attack_teleport_unique_injury) {
                    /* This unique is low on HP, finish it off */
                    dam = -9999;
                } else if st.borg.status.anti_summon {
                    /* I am sitting pretty in an AS-Corridor */
                    dam = -9999;
                } else if dam > avoid(st) * param(Param::attack_teleport_unique_danger_tenths) / 10 && tr(st, BI_CDEPTH) <= param(Param::attack_teleport_unique_max_depth) {
                    /* If this unique is causing the danger, get rid of it */
                    tp_other_add(st, i);
                } else if i32::from(st.borg.near.unique) >= param(Param::attack_teleport_unique_min_count) && i32::from(st.borg.near.unique) <= param(Param::attack_teleport_unique_max_count) {
                    /* If fighting multiple uniques, get rid of one */
                    tp_other_add(st, i);
                } else if tr(st, BI_CLASS) == CLASS_MAGE && engine::distance(st.borg.c, kill.pos) <= param(Param::attack_teleport_mage_unique_distance) {
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
                    if vault_grids >= param(Param::attack_teleport_vault_min_walls) {
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
                    dam = param(Param::attack_teleport_glyph_value);
                }
            }

            /* If the borg is not in a good position, do it */
            if st.borg.near.morgoth && !st.borg.morgoth_position {
                /* get rid of this one by storing his info */
                tp_other_add(st, i);
                dam = param(Param::attack_teleport_morgoth_value);
            }

            /* If the borg does not have enough Mana to attack this
             * round and cast Teleport Away next round, then do it now.
             */
            if tr(st, BI_CURSP) <= param(Param::attack_teleport_low_sp) {
                /* get rid of this unique by storing his info */
                tp_other_add(st, i);
                dam = param(Param::attack_teleport_low_sp_value);
            }
        }

        /* This BORG_ATTACK_ is hacked to work for Mass Genocide.  Since
         * we cannot mass gen uniques.
         */
        BORG_ATTACK_DISP_ALL => {
            if rf(r, RF_UNIQUE) {
                dam = 0;
            } else {
                dam = danger_one_here(st, param(Param::attack_turn_danger_horizon), i, true);
            }
        }

        BORG_ATTACK_OLD_CONF => {
            dam = 0;
            if !(rf(r, RF_NO_CONF)
                || rf(r, RF_MULTIPLY)
                || i32::from(kill.speed) < r_ptr.speed - param(Param::attack_status_slow_speed_margin)
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
            if !(i32::from(kill.speed) < r_ptr.speed - param(Param::attack_status_slow_speed_margin)
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
            if !(i32::from(kill.speed) < r_ptr.speed - param(Param::attack_status_slow_speed_margin)
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
                || i32::from(kill.speed) < r_ptr.speed - param(Param::attack_status_slow_speed_margin)
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
                    dam = danger_one_here(st, param(Param::attack_fear_danger_horizon), i, true);
                    /* don't bother unless he is a scary monster */
                    if dam < avoid(st) * param(Param::attack_fear_danger_multiplier) && !kill.afraid {
                        dam = 0;
                    }
                }
            }
        }

        BORG_ATTACK_TURN_UNDEAD => {
            if rf(r, RF_UNDEAD) {
                dam = 0;
                if !(kill.confused
                    || i32::from(kill.speed) < r_ptr.speed - param(Param::attack_status_slow_speed_margin)
                    || !kill.awake
                    || i32::from(kill.level) > tr(st, BI_CLEVEL) - param(Param::attack_crush_monster_level_margin))
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
                    dam = danger_one_here(st, param(Param::attack_sleep_danger_horizon), i, true);
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
        && (borg_danger_one_kill(st, kill.pos.y, kill.pos.x, param(Param::attack_missile_danger_horizon), i, true, true) > avoid(st) * param(Param::attack_missile_danger_tenths) / 10
            || (has_friends(r) /* monster has friends*/
                && i32::from(kill.level) >= tr(st, BI_CLEVEL) - param(Param::attack_missile_level_margin)/* close levels */)
            || kill.ranged_attack != 0 /* monster has a ranged attack */
            || rf(r, RF_UNIQUE)
            || rf(r, RF_MULTIPLY)
            || gold_eater /* Monster can steal gold */
            || rf(r, RF_NEVER_MOVE) /* monster never moves */
            || tr(st, BI_CLEVEL) <= param(Param::attack_missile_weak_max_level)/* stil very weak */)
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
    if dam > i32::from(kill.power) * param(Param::attack_ranged_overkill_multiplier) && !rf(r, RF_UNIQUE) {
        dam = i32::from(kill.power) * param(Param::attack_ranged_overkill_multiplier);
    }

    /* give a small bonus for whacking a unique */
    /* this should be just enough to give preference to whacking uniques */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
        dam *= param(Param::attack_ranged_unique_multiplier);
    }

    /* Ignore Maggot until later.  Player will chase Maggot
     * down all across the screen waking up all the monsters.  Then
     * he is stuck in a compromised situation.
     */
    if rf(r, RF_UNIQUE) && tr(st, BI_CDEPTH) == 0 {
        dam = dam * param(Param::attack_ranged_town_unique_numerator) / param(Param::attack_ranged_town_unique_divisor);

        /* Don't hunt maggot until later */
        if tr(st, BI_CLEVEL) < param(Param::attack_town_unique_min_level) {
            dam = 0;
        }
    }
    if ignored(st, usize::from(r)) {
        dam = 0;
    }
    // borg-reborn-rs (#619): a dangerous town monster comes first.
    dam = town_kill_bonus(st, usize::from(r), dam);
    // borg-reborn-rs (#637): a wand in town: nothing else is worth a charge.
    if st.fight_attack.town_wand && !town_kill_on_sight(st, usize::from(r)) {
        dam = 0;
    }

    /* give a small bonus for whacking a breeder */
    if rf(r, RF_MULTIPLY) {
        dam = dam * param(Param::attack_ranged_breeder_numerator) / param(Param::attack_ranged_breeder_divisor);
    }

    /* Enhance the perceived damage to summoner in order to influence the
     * choice of targets.
     */
    if is_summoner(r, true) {
        dam += (dam * param(Param::attack_ranged_summoner_numerator)) / param(Param::attack_ranged_summoner_divisor);
    }

    /*
     * Apply massive damage bonus to Questor monsters to
     * encourage borg to strike them.
     */
    if rf(r, RF_QUESTOR) {
        dam += dam * param(Param::attack_ranged_questor_bonus);
    }

    /*  Try to conserve missiles.
     */
    if typ as u32 == BORG_ATTACK_ARROW && !borg_use_missile && !junk_potion {
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

/// One aim and the believed records in its projected path or area.
#[derive(Default)]
struct ProjectedAttack {
    score: i32,
    aim: Option<loc>,
    confused_aim: bool,
    unseen: Vec<BelievedTarget>,
}

impl ProjectedAttack {
    fn target(self, st: &mut BorgState, require_monster: bool) {
        st.fight_attack.pending_unseen.clear();
        let Some(aim) = self.aim else { return };
        st.borg.goal.g = aim;
        // #867: a confused attack sends an adjacent direction.
        if self.confused_aim {
            st.fight_attack.confused_dir = borg_extract_dir(st.borg.c.y, st.borg.c.x, aim.y, aim.x);
            return;
        }
        if borg_target(st, aim, require_monster) {
            st.fight_attack.pending_unseen.stage(self.unseen);
        }
    }
}

fn expected_score(score: i32, belief: Option<i32>) -> i32 {
    match belief {
        Some(weight) => (i64::from(score) * i64::from(weight) / 100) as i32,
        None => score,
    }
}

fn collect_believed_target(
    unseen: &mut Vec<BelievedTarget>, index: usize, kill: &BorgKill, belief: Option<i32>, age: BorgTime,
) {
    if let Some(weight) = belief {
        unseen.push(BelievedTarget { index, race: kill.r_idx, pos: kill.pos, weight, age });
    }
}

/// Simulate / Invoke the launching of a bolt at a monster
/// (`borg_launch_bolt_aux_hack`).
fn borg_launch_bolt_aux_hack(
    st: &mut BorgState, i: i32, dam: i32, typ: i32, ammo_location: i32,
    unseen: &mut Vec<BelievedTarget>,
) -> i32 {
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
    // borg-reborn-rs (#879): or a monster believed at its last grid; #889:
    // with the blast's slack, when a blast scores the grid.
    let belief = crate::borg_unseen_belief::believed_weight_with(st, i as usize, st.fight_attack.unseen_blast_slack);
    let age = borg_timer(st, kill.when);
    if belief.is_none()
        && (age > param(Param::attack_bolt_max_target_age)
            || (param(Param::unseen_belief) != 0 && !kill.seen && age > param(Param::attack_max_target_age)))
    {
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
        if walls >= param(Param::attack_bolt_unknown_min_walls) && unknown >= param(Param::attack_bolt_unknown_min_grids) {
            return 0;
        }
    }

    if typ as u32 == BORG_ATTACK_CURSE && belief.is_some() {
        return 0;
    }
    let junk_potion = is_potion_throw(st, typ, ammo_location);
    if junk_potion {
        // New records have no seen bit yet. An estimate can run during
        // the scan, while ordinary attacks run one Borg turn after it.
        const MAX_CURRENT_SCAN_AGE: BorgTime = 1;
        let current = kill.seen || (0..=MAX_CURRENT_SCAN_AGE).contains(&age);
        let visible = current && ag.info & BORG_VIEW != 0 && belief.is_none();
        if !kill.awake || !visible || ignored(st, usize::from(r)) {
            return 0;
        }
    }
    // A projected hit can wake a sleeper even when it reduces the score.
    collect_believed_target(unseen, i as usize, &kill, belief, age);

    /* Calculate damage */
    let mut d = borg_launch_damage_one(st, i, dam, typ, ammo_location);

    /* Teleport Other's danger reduction is calculated by the caller.
     * A bolt stops at the first record on the path. */
    if typ as u32 == BORG_ATTACK_AWAY_ALL {
        return d;
    }

    /* These attacks already score the monster's danger. */
    if typ as u32 == BORG_ATTACK_AWAY_ALL_MORGOTH || typ as u32 == BORG_ATTACK_AWAY_EVIL {
        return expected_score(d, belief);
    }

    /* Return 0 if the true damage (w/o the danger bonus) is 0 */
    if d <= 0 {
        // Some junk potions have no damage dice. A reachable target still
        // lets us spend one, after attacks that can do useful damage.
        if d == 0 && junk_potion {
            return 1;
        }
        return expected_score(d, belief);
    }

    /* Calculate danger (at the monster's grid: `x`, `y` are still the
     * monster's position, or its last neighbour for a ghost) */
    let (dy, dx) = if rf(r, RF_PASS_WALL) { (kill.pos.y + 1, kill.pos.x + 1) } else { (y, x) };
    let p2 = borg_danger_one_kill(st, dy, dx, param(Param::attack_bolt_sleep_danger_horizon), i, true, false);

    /* Avoid waking most "hard" sleeping monsters */
    if !kill.awake && p2 > avoid(st) / param(Param::attack_bolt_sleep_danger_divisor) && d < i32::from(kill.power) {
        return expected_score(-999, belief);
    }

    /* Ignore sleeping town monsters */
    if tr(st, BI_CDEPTH) == 0 && !kill.awake {
        return 0;
    }

    /* Calculate "danger" to player */
    let p1 = if st.fight_attack.throwing {
        // borg-reborn-rs (#568): a throw takes melee's danger term
        // (`melee_score`): melee's horizon, the full damage, and the
        // partial-kill division.
        let mut p = danger_one_here(st, param(Param::attack_melee_danger_horizon), i, true);
        if d <= i32::from(kill.power) && tr(st, BI_MAXCLEVEL) > param(Param::attack_melee_partial_min_level) {
            p /= param(Param::attack_melee_partial_danger_divisor);
        }
        p
    } else {
        danger_one_here(st, param(Param::attack_bolt_danger_horizon), i, false)
    };
    // borg-reborn-rs (#637): the town bonus on the danger term too.
    let p1 = town_danger_bonus(st, usize::from(r), p1);

    // borg-reborn-rs (#584): the true damage, at most the monster's HP.
    let damage = if st.fight_attack.focused_target.is_some() { d.max(0) }
        else { d.clamp(0, i32::from(kill.power).max(0)) };
    st.fight_attack.dmg_acc += expected_score(damage, belief);
    st.fight_attack.kill_acc += kill_fraction(d, i32::from(kill.power))
        * (f64::from(belief.unwrap_or(100)) / 100.0);

    /* Extra "bonus" if attack kills */
    if d >= i32::from(kill.power) {
        d *= param(Param::attack_bolt_kill_multiplier);
    }

    /* Score damage and danger conditional on the monster being there. */
    expected_score(d + p1, belief)
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
                return param(Param::attack_destroy_unknown_boots_value);
            }
        }
        BORG_ATTACK_ELEC => {
            /* rings/boots cost extra (might be speed!) */
            if k_ptr.tval == TV_RING as i32 && !k_ptr.aware {
                return param(Param::attack_destroy_unknown_ring_value);
            }
            if k_ptr.tval == TV_RING as i32 && k_ptr.sval == iv.sv_ring_speed {
                return param(Param::attack_destroy_speed_ring_value);
            }
        }
        BORG_ATTACK_FIRE => {
            /* rings/boots cost extra (might be speed!) */
            if k_ptr.tval == TV_BOOTS as i32 && !k_ptr.aware {
                return param(Param::attack_destroy_unknown_boots_value);
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
                    return param(Param::attack_destroy_valuable_potion_value);
                }

                return param(Param::attack_destroy_unknown_potion_value);
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
            < avoid(st) / param(Param::attack_unmapped_fear_divisor)
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
    unseen: &mut Vec<BelievedTarget>,
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
    // A native throw can hit a monster on its final grid. Apply that
    // boundary to the new potion policy and preserve existing throw play.
    let end = max + i32::from(is_potion_throw(st, typ, ammo_location));
    while dist < end {
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
            n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam, typ, ammo_location, unseen);
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

            // borg-reborn-rs (#889): the monster steps the ball still covers
            // from this grid (read by the belief only under unseen_spread).
            // Only a ball's: a dispel or other line-of-sight effect (rad 10)
            // and Starlight's lines (rad 7) reach what the player sees, not a
            // radius around the aim (re-review of #1128, F1).
            let slack = if st.fight_attack.ball_shape {
                crate::borg_unseen_belief::blast_slack(rad, ry - y2, rx - x2)
            } else {
                0
            };

            /*  dispel spells should hurt the same no matter the rad: make r= y
             * and x */
            if rad == 10 {
                r = 0;
            }

            /* Collect damage, lowered by distance */
            st.fight_attack.unseen_blast_slack = slack;
            n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam / (r + 1), typ, ammo_location, unseen);
            st.fight_attack.unseen_blast_slack = 0;

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
    let attack = choose_bolt(st, rad, dam, typ, max, ammo_location);
    let score = attack.score;
    if !simulating(st) {
        attack.target(st, typ as u32 == BORG_ATTACK_CURSE);
    }
    score
}

fn choose_bolt(st: &mut BorgState, rad: i32, dam: i32, typ: i32, max: i32, ammo_location: i32) -> ProjectedAttack {
    let mut unseen = Vec::new();
    let mut best_unseen = Vec::new();
    let mut b_i: i32 = -1;
    let mut b_n: i32 = -1;
    let mut b_o_y = 0;
    let mut b_o_x = 0;
    let mut b_d = max_range();
    // borg-reborn-rs (#867, review of #868): a confused aim applies only to
    // the attack being rated, and only while the Borg is confused: a flag
    // left by an earlier think never restricts another caller (Teleport
    // Other, `borg_fight_defend.rs`).
    let confused_aim = st.fight_attack.confused_aim && confused_by_direction(st);

    // borg-reborn-rs (#889, Codex re-review of #1128): a ball (not a breath,
    // not a focused attack) also considers the blast-only aims, appended to
    // the aim list for this call only.
    let base_n = st.flow.temp_n;
    if st.fight_attack.ball_shape
        && st.fight_attack.focused_target.is_none()
        && (1..=crate::borg_unseen_belief::MAX_BALL_RADIUS).contains(&rad)
    {
        for (y, x) in st.fight_attack.blast_aims.clone() {
            st.flow.temp_push(y, x);
        }
    }

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

                // borg-reborn-rs (#867): confused, only a direction: the
                // adjacent monster itself, no offset aim.
                if confused_aim && (o_x != 0 || o_y != 0 || dist_to(st, tx, ty) != 1) {
                    continue;
                }

                /* Reset Teleport Other variables */
                st.fight_attack.tp_other_n = 0;
                let mut n = 0;
                unseen.clear();
                st.fight_attack.dmg_acc = 0;
                st.fight_attack.kill_acc = 0.0;

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
                    n = borg_launch_bolt_at_location(st, y, x, rad, dam, typ, max, ammo_location, &mut unseen);
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
                    n = expected_score(dam - n, unseen.first().map(|target| target.weight));
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
                std::mem::swap(&mut best_unseen, &mut unseen);
                st.fight_attack.dmg_best = st.fight_attack.dmg_acc;
                st.fight_attack.kill_best = st.fight_attack.kill_acc;
            }
        }
        i += 1;
    }
    if b_i == -1 {
        st.flow.temp_n = base_n;
        return ProjectedAttack::default();
    }

    st.fight_attack.tp_other_n = 0;
    let (ty, tx) = st.flow.temp_at(b_i as usize);
    st.flow.temp_n = base_n;
    ProjectedAttack {
        score: b_n,
        aim: Some(loc { x: tx + b_o_x, y: ty + b_o_y }),
        confused_aim,
        unseen: best_unseen,
    }
}

/// Determine the "reward" of launching an arc/spray/cone at a location
/// (`borg_launch_arc_at_location`).
///
/// This code is copied from borg_launch_bolt_at_location
/// both need to be optimized for shared code !FIX
#[allow(clippy::too_many_arguments)]
fn borg_launch_arc_at_location(
    st: &mut BorgState, y: i32, x: i32, degrees: i32, dam: i32, typ: i32, max: i32,
    unseen: &mut Vec<BelievedTarget>,
) -> i32 {
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

            /* project.c:856-863 uses LOS through PROJECT terrain for
             * the arc footprint. Walkable rubble still blocks that ray. */
            if !borg_los_projectable(st, y1, x1, ry, rx) {
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
                n += borg_launch_bolt_aux_hack(st, i32::from(ag.kill), dam / (r + 1), typ, 0, unseen);
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

    let mut unseen = Vec::new();
    let mut best_unseen = Vec::new();
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
        st.fight_attack.dmg_acc = 0;
        st.fight_attack.kill_acc = 0.0;
        unseen.clear();
        let n = borg_launch_arc_at_location(st, y, x, degrees, dam, typ, max, &mut unseen);

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
        std::mem::swap(&mut best_unseen, &mut unseen);
        st.fight_attack.dmg_best = st.fight_attack.dmg_acc;
        st.fight_attack.kill_best = st.fight_attack.kill_acc;
    }
    if b_i == -1 {
        return b_n;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    let (y, x) = st.flow.temp_at(b_i as usize);
    ProjectedAttack { score: b_n, aim: Some(loc { x, y }), confused_aim: false, unseen: best_unseen }.target(st, false);

    /* Result */
    b_n
}

/// Simulate/Apply the optimal result of launching a missile
/// (`borg_attack_aux_launch`).
///
/// When no missile is chosen (every `borg_launch_bolt` finds no target)
/// `b_k` stays -1 and nothing is used.
/// The average damage of firing quiver `item` from `bow`, or `None` when
/// `borg_attack_aux_launch` skips it (borg-reborn-rs #539: shared with
/// `ranged_means`).
fn launch_damage(st: &BorgState, item: &borg_item, bow: &borg_item) -> Option<i32> {
    /* Skip missiles that don't match the bow */
    if i32::from(item.tval) != tr(st, BI_AMMO_TVAL) {
        return None;
    }

    /* Skip worthless missiles */
    if item.value <= 0 {
        return None;
    }

    if st.fight_attack.focused_target.is_some() {
        // Keep the half point in 1d4 ammunition until after the launcher
        // multiplier. Integer truncation before x2 understates it by 20%.
        let d = (i32::from(item.dd) * (i32::from(item.ds) + 1) * 500
            + 1000 * (i32::from(item.to_d) + i32::from(bow.to_d)))
            * tr(st, BI_AMMO_POWER);
        return (d > 0).then_some(d);
    }

    /* Determine average damage */
    let mut d = i32::from(item.dd) * (i32::from(item.ds) + 1) / 2;
    d = d + i32::from(item.to_d) + i32::from(bow.to_d);
    d = d * tr(st, BI_AMMO_POWER) * tr(st, BI_SHOTS);

    /* Boost the perceived damage on unID'd ones so he can get a quick
     * pseudoID on it */
    if st.fight_attack.focused_target.is_none() && borg_item_note_needs_id(st, item) {
        d *= param(Param::attack_ego_ammo_multiplier);
    }

    /* Paranoia */
    (d > 0).then_some(d)
}

/// borg-reborn-rs (#539 review): something the attack planner would shoot
/// or throw now, by its own rules: a quiver missile `borg_attack_aux_launch`
/// fires from the bow, or the object `borg_attack_aux_object` throws (a
/// flask kept for lantern fuel or worth money for little damage is not one),
/// and nothing that stops aiming.
pub(crate) fn ranged_means(st: &BorgState) -> bool {
    if cant_aim(st) {
        return false;
    }
    let bow = st.items.items[(inven_wield() + 1) as usize];
    let shoot = bow.iqty != 0
        && (quiver_start()..quiver_end())
            .map(|k| st.items.items[k as usize])
            .take_while(|item| item.iqty != 0)
            .any(|item| launch_damage(st, &item, &bow).is_some());
    shoot || best_throw(st).0 >= 0
}

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
    // borg-reborn-rs (#867): confused, by direction at an adjacent monster.
    if cant_use_item_attack(st) {
        return 0;
    }
    mark_confused_aim(st);

    // With belief enabled, only the winning quiver stack may aim and
    // charge a shot. Preserve the original missile path at switch 0.
    let choose_before_aiming = param(Param::unseen_belief) != 0 && !simulating(st);
    let mut best_attack = None;

    /* Scan the quiver */
    for k in quiver_start()..quiver_end() {
        let item = st.items.items[k as usize];

        /* Skip empty items */
        if item.iqty == 0 {
            break;
        }

        let Some(d) = launch_damage(st, &item, &bow) else {
            continue;
        };
        let v = item.value;

        /* Choose optimal target of bolt */
        let power = 6 + 2 * tr(st, BI_AMMO_POWER);
        let attack = choose_before_aiming.then(|| choose_bolt(st, 0, d, BORG_ATTACK_ARROW as i32, power, k));
        let n = match &attack {
            Some(attack) => attack.score,
            None => borg_launch_bolt(st, 0, d, BORG_ATTACK_ARROW as i32, power, k),
        };

        /* if two attacks are equal, pick the cheaper ammo */
        if n == b_n && v >= b_v {
            continue;
        }

        if n >= b_n {
            b_n = n;
            b_v = v;
            b_k = k;
            best_attack = attack;
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
    if let Some(attack) = best_attack {
        attack.target(st, false);
    }
    let mut m = b"# Firing missile '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[b_k as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Fire */
    borg_keypress(st, key(b'f'));

    /* Use the missile from the quiver */
    borg_keypress(st, ((b_k - quiver_start()) + i32::from(b'0')) as keycode_t);

    /* Use target */
    /* Set our shooting flag */
    st.fight_attack.successful_target = aim_key(st, -2);
    st.fight_attack.pending_unseen.command_queued(true);

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
            if borg_timer(st, kill.when) > param(Param::attack_throw_max_target_age) {
                break 'check false;
            }

            /* Skip monsters that dont chase */
            if rf(kill.r_idx, RF_NEVER_MOVE) {
                break 'check false;
            }

            /* Monster better not be faster than me */
            if i32::from(kill.speed) - tr(st, BI_SPEED) >= param(Param::attack_throw_fast_monster_margin) {
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

            /* Do not wait for an approach across an obstructed sight line.
             * Passable rubble can leave the monster waiting on its side. */
            if !borg_los_projectable(st, st.borg.c.y, st.borg.c.x, kill.pos.y, kill.pos.x) {
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
/// it (`borg_attack` ignores the result of the real call; so did the
/// munchkin attacks, removed with the mode in borg-rs#60).
/// Rust returns `b_n`, as the c2rust translation did.
fn borg_attack_aux_object(st: &mut BorgState) -> i32 {
    if cant_use_item_attack(st) {
        return 0;
    }
    let surplus = param(Param::attack_throw_surplus) != 0;
    let junk_potions = param(Param::attack_throw_junk_potions) != 0
        && tr(st, BI_CDEPTH) > 0
        && !st.borg.goal.fleeing && !st.borg.goal.fleeing_to_town
        && tr(st, BI_ISPOISONED) == 0 && tr(st, BI_ISCUT) == 0
        && st.items.items[..usize::from(engine::z_info().pack_size)]
            .iter().any(|item| crate::borg_junk::junk_potion_to_throw(st, item))
        && low_risk_potion_throw(tr(st, BI_CURHP), danger_here(st, 1, true));
    if surplus || junk_potions {
        mark_confused_aim(st);
        return attack_throw_candidates(st, surplus, junk_potions);
    }
    let (slot, damage) = best_throw(st);
    if slot < 0 {
        return 0;
    }
    mark_confused_aim(st);
    borg_attack_aux_object_fire(st, slot, damage)
}

// A miss consumes a full turn. Start with the same small danger budget
// as safe junk disposal, measured against current rather than maximum HP.
fn low_risk_potion_throw(hp: i32, danger: i32) -> bool {
    hp > 0 && danger <= hp / 10
}

fn is_potion_throw(st: &BorgState, typ: i32, slot: i32) -> bool {
    st.fight_attack.throwing && typ as u32 == BORG_ATTACK_ARROW
        && u32::from(st.items.items[slot as usize].tval) == TV_POTION
}

/// #1200: Only a fully known plain weapon dominated by the equipped weapon
/// can be spent. Preserve combat bonuses, blows, protection and swap roles.
fn surplus_weapon(st: &BorgState, slot: i32, item: &borg_item, worn: &borg_item) -> bool {
    let tval = u32::from(item.tval);
    let same_divine_role = tval != TV_HAFTED || u32::from(worn.tval) == TV_HAFTED;
    matches!(tval, TV_SWORD | TV_POLEARM | TV_HAFTED)
        && item.iqty > 0 && item.ident && !item.needs_ident && !item.cursed
        && item.art_idx == 0 && item.ego_idx == 0 && item.activ_idx == 0 && crate::borg_item::item_note(st, item).is_empty()
        && (0..=param(Param::attack_throw_surplus_max_value)).contains(&item.value)
        && engine::flag_has(&item.flags, OF_THROWING as i32)
        && item.flags.iter().map(|byte| byte.count_ones()).sum::<u32>() == 1
        && item.modifiers.iter().all(|&value| value == 0)
        && item.el_info.iter().all(|element| element.res_level == 0)
        && !item.brands.iter().any(|&brand| brand)
        && !item.slays.iter().any(|&slay| slay != 0)
        && st.trait_swap.weapon_swap != slot + 1
        && worn.iqty > 0 && worn.ident && !worn.cursed && same_divine_role
        && i32::from(worn.weight) / 10 <= engine::adj_str_hold(tr(st, BI_STR_INDEX) as usize)
        && item.dd <= worn.dd && item.ds <= worn.ds
        && item.to_h <= worn.to_h && item.to_d <= worn.to_d && item.to_a <= worn.to_a
        && crate::borg_trait::borg_calc_blows(st, item) <= tr(st, BI_BLOWS)
}

/// Engine player-attack.c:do_cmd_throw at 574b618c, in tenths of a pound.
fn native_throw_range(strength: i32, weight: i32) -> i32 {
    ((strength + 20) * 10 / weight.max(10)).min(10)
}

fn attack_throw_candidates(st: &mut BorgState, surplus: bool, junk_potions: bool) -> i32 {
    let native_rating = surplus || native_throw_rating(st);
    let (legacy_slot, legacy_damage) = best_throw_of(st, false, native_rating);
    let strength = engine::adj_str_blow(tr(st, BI_STR_INDEX) as usize);
    let mut slot = legacy_slot;
    let mut range = 0;
    let mut best = ProjectedAttack::default();
    if legacy_slot >= 0 {
        let weight = st.items.items[legacy_slot as usize].weight;
        range = if surplus { native_throw_range(strength, i32::from(weight)) }
            else { 6 + 2 * tr(st, BI_AMMO_POWER) };
        st.fight_attack.throwing = native_rating;
        best = choose_bolt(st, 0, legacy_damage, BORG_ATTACK_ARROW as i32, range, legacy_slot);
    }
    let mut damage = st.fight_attack.dmg_best;
    let mut kills = st.fight_attack.kill_best;
    let worn = st.items.items[inven_wield() as usize];
    st.fight_attack.throwing = true;
    for k in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[k as usize];
        let potion = junk_potions && crate::borg_junk::junk_potion_to_throw(st, &item);
        if !potion && !(surplus && surplus_weapon(st, k, &item, &worn)) {
            continue;
        }
        let max = native_throw_range(strength, i32::from(item.weight));
        let per_hit = thrown_damage(i32::from(item.dd), i32::from(item.ds),
            i32::from(item.to_d), i32::from(item.weight), !potion, false);
        if per_hit <= 0 && !potion {
            continue;
        }
        let mut candidate = choose_bolt(st, 0, per_hit, BORG_ATTACK_ARROW as i32, max, k);
        if potion && st.fight_attack.dmg_best > 0 && candidate.score > 0 {
            candidate.score += 2;
        }
        let smaller_potion_stack = potion && slot >= 0 && slot != legacy_slot
            && u32::from(st.items.items[slot as usize].tval) == TV_POTION
            && item.iqty < st.items.items[slot as usize].iqty;
        if candidate.score > best.score
            || (candidate.score > 0 && candidate.score == best.score && smaller_potion_stack)
        {
            best = candidate;
            slot = k;
            range = max;
            damage = st.fight_attack.dmg_best;
            kills = st.fight_attack.kill_best;
        }
    }
    st.fight_attack.throwing = false;
    st.fight_attack.dmg_best = damage;
    st.fight_attack.kill_best = kills;
    if best.score <= 0 {
        return best.score;
    }
    let item = st.items.items[slot as usize];
    st.fight_attack.cost = Some(AttackCost { fail_pct: 0, uses: Some(i32::from(item.iqty)), spell: false });
    let score = best.score;
    if !simulating(st) {
        if u32::from(item.tval) == TV_POTION {
            borg_note(st, &format!("# Junk potion throw: pack slot {slot}, score {score}."));
        } else if slot != legacy_slot {
            if let Some(aim) = best.aim {
                let distance = dist_to(st, aim.x, aim.y);
                let target = grid(st, aim.x, aim.y).kill;
                let hit = if target == 0 { 0 } else {
                    let ac = engine::race(usize::from(st.kills.kills[usize::from(target)].r_idx)).ac;
                    throw_hit_percent(tr(st, BI_THT), tr(st, BI_TOHIT) + i32::from(item.to_h), true, distance, ac)
                };
                borg_note(st, &format!("# Surplus throw: pack slot {slot}, distance {distance}/{range}, hit {hit}%, score {score}."));
            }
        }
        if surplus || slot != legacy_slot {
            crate::borg_object_log::log_use(st, "throw", &item);
        }
        best.target(st, false);
        queue_object_throw(st, slot);
    }
    score
}

/// borg-reborn-rs (#539): the pack scan of `borg_attack_aux_object`, the
/// object it would throw and its average damage, as `(slot, damage)`
/// (`(-1, -1)`: nothing). `ranged_means` asks it too.
fn best_throw(st: &BorgState) -> (i32, i32) {
    best_throw_of(st, false, native_throw_rating(st))
}

fn native_throw_rating(st: &BorgState) -> bool {
    st.fight_attack.focused_target.is_some() || param(Param::attack_throw_as_melee) != 0
}

// borg-reborn-rs (#781): same fuel reserves, restricted to oil for a bump.
fn best_throw_of(st: &BorgState, flasks_only: bool, native_rating: bool) -> (i32, i32) {
    // The shopping supply estimate is needed only for a throwable flask
    // while a fuel-burning lantern is worn. It is independent of the return
    // policy and reused within this evaluation.
    let mut supply_journey: Option<crate::borg_town_utility::Journey> = None;
    let mut b_k: i32 = -1;
    let mut b_d: i32 = -1;

    /* Scan the pack */
    for k in 0..i32::from(engine::z_info().pack_size) {
        let item = st.items.items[k as usize];

        if flasks_only && u32::from(item.tval) != TV_FLASK { continue; }

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
        if item.value > param(Param::attack_throw_min_item_value) && d < param(Param::attack_throw_low_damage) {
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
        if u32::from(item.tval) == TV_FLASK {
            let light = st.items.items[(crate::borg_item::inven_wield() + 5) as usize];
            let lantern = u32::from(light.tval) == crate::headers::list_tvals_h::TV_LIGHT
                && i32::from(light.sval) == st.item_val.sv_light_lantern
                && !engine::flag_has(&light.flags, crate::headers::list_object_flags_h::OF_NO_FUEL as i32);
            let policy = st.cfg(crate::borg_headers::borg_h::BORG_TOWN_PLAN) == 1;
            let reserve = if policy && lantern {
                let journey = supply_journey.get_or_insert_with(||
                    crate::borg_town_utility::Journey::of(st));
                // borg-reborn-rs (#1110): the planner's goal, scaled while
                // breadth-first exploration applies.
                let goal = crate::borg_light::fuel_goal(st, *journey);
                let shortage = (goal - i64::from(light.timeout)).max(0) as u64;
                shortage.div_ceil(i64::from(item.pval).max(1) as u64) as i32
            } else { 0 };
            if oil_reserved(policy, reserve, tr(st, BI_AFUEL), st.borg.near.unique != 0) { continue; }
        }

        /* Dont throw wands or rods */
        if u32::from(item.tval) == TV_WAND || u32::from(item.tval) == TV_ROD {
            continue;
        }

        // borg-reborn-rs (#568): with `attack_throw_as_melee`, the engine's
        // damage of the throw ranks the objects and is the attack's damage
        // (the gates above keep the dice average).
        let d = if native_rating {
            thrown_damage(
                i32::from(item.dd),
                i32::from(item.ds),
                i32::from(item.to_d),
                i32::from(item.weight),
                engine::flag_has(&item.flags, OF_THROWING as i32),
                engine::flag_has(&item.flags, OF_EXPLODE as i32),
            )
        } else {
            d
        };

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
    (b_k, b_d)
}

/// borg-reborn-rs (#781): throw oil in the observed bump's direction. A
/// direction can hit an unseen monster; a monster target cannot be set.
/// Uses the normal chooser's fuel reservation; never throws an unknown item.
pub(crate) fn throw_at_unseen_bump(st: &mut BorgState, grid: loc) -> bool {
    let (slot, _) = best_throw_of(st, true, native_throw_rating(st));
    if slot < 0 || !st.items.items[slot as usize].aware { return false; }
    let dir = crate::borg_flow_misc::borg_goto_dir(st, st.borg.c.y, st.borg.c.x, grid.y, grid.x);
    borg_note(st, &format!("# Throwing oil at unseen bump ({},{})", grid.y, grid.x));
    crate::borg_object_log::log_use(st, "throw", &st.items.items[slot as usize]);
    borg_keypress(st, key(b'v'));
    borg_keypress(st, keycode_t::from(engine::all_letters_nohjkl(slot)));
    crate::borg_io::borg_queue_direction(st, (dir + i32::from(b'0')) as keycode_t);
    // This is a direction, not a tracked target: no unknown-wall inference.
    st.fight_attack.successful_target = 0;
    true
}

/// `borg_attack_aux_object` after its scan: aim and throw `b_k`.
fn borg_attack_aux_object_fire(st: &mut BorgState, b_k: i32, b_d: i32) -> i32 {
    /* Choose optimal location */
    let power = 6 + 2 * tr(st, BI_AMMO_POWER);
    st.fight_attack.throwing = native_throw_rating(st);
    let b_n = borg_launch_bolt(st, 0, b_d, BORG_ATTACK_ARROW as i32, power, b_k);
    st.fight_attack.throwing = false;

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    queue_object_throw(st, b_k);
    b_n
}

fn queue_object_throw(st: &mut BorgState, b_k: i32) {
    let mut m = b"# Throwing painful object '".to_vec();
    m.extend_from_slice(&desc(&st.items.items[b_k as usize]));
    m.push(b'\'');
    borg_note_bytes(st, &m);

    /* Fire */
    borg_keypress(st, key(b'v'));

    // borg_messages_react switches the native compact throw list to the
    // pack before the queued slot letter is read.
    /* Use the object */
    borg_keypress(st, keycode_t::from(engine::all_letters_nohjkl(b_k)));

    /* Use target */
    /* Set our shooting flag */
    st.fight_attack.successful_target = aim_key(st, -2);
    st.fight_attack.pending_unseen.command_queued(true);

}

/// borg-reborn-rs (#568): the engine's average damage of an object thrown
/// by hand, before brands, criticals and the hit roll (`ranged_damage`,
/// `player-attack.c:554-582`, and `make_ranged_throw`, `:1296`): the dice
/// average plus to-dam, times `2 + weight / 12` for a THROWING object
/// (weight in tenths of a pound), and three times that for EXPLODE. A flask
/// of oil (1d4, weight 20, `EXPLODE | THROWING`) does 22.
pub(crate) fn thrown_damage(dd: i32, ds: i32, to_d: i32, weight: i32, throwing: bool, explode: bool) -> i32 {
    let mut dam10 = dd * (ds + 1) * 5 + to_d * 10;
    if throwing {
        dam10 *= 2 + weight.max(0) / 12;
    }
    if explode {
        dam10 *= 3;
    }
    dam10 / 10
}

/// #870: zero inherits the resolved arrow policy, including class overrides.
fn throw_point_blank_divisor(configured: i32, arrow_divisor: i32) -> i32 {
    if configured == 0 { arrow_divisor } else { configured }
}

/// #870: the native throw rating keeps integer division before hit chance.
/// This is a conservation policy, not engine damage: player-attack.c:156-165
/// at 574b618c applies distance to hit chance, without an adjacent divisor.
/// The focused planner bypasses this policy and continues to rate real damage.
fn rated_throw_damage(mut damage: i32, distance: i32, unique: bool, divisor: i32, hit_percent: i32) -> i32 {
    if distance == 1 && !unique { damage /= divisor; }
    damage * hit_percent / 100
}

/// borg-reborn-rs (#568): the engine's chance, in percent, that an object
/// thrown by hand hits a monster of armour `ac` at `dist` grids
/// (`chance_of_missile_hit_base` without a launcher, `chance_of_missile_hit`
/// and `hit_chance`, `player-attack.c:121-219`): the throwing skill plus
/// `bonus` × `BTH_PLUS_ADJ` for a THROWING object, else 3/2 of the skill;
/// less the distance, at least 9; 12% always hit, 5% always miss.
fn throw_hit_percent(tht: i32, bonus: i32, throwing: bool, dist: i32, ac: i32) -> i32 {
    let skill = if throwing { tht } else { 3 * tht / 2 };
    let to_hit = (skill + bonus * BTH_PLUS_ADJ - dist).max(9);
    let rate = 10_000 * (to_hit - ac * 2 / 3).max(0) / to_hit;
    (rate * (10_000 - 500 - 1_200) / 10_000 + 1_200) / 100
}

/// The experimental supply model may stock oil for throwing with a torch.
/// Preserve the control's gate; reserve fuel only for a fuel-burning lantern
/// under the new policy. The unique-fight emergency exception stays intact.
fn oil_reserved(capability_policy: bool, required: i32, fuel: i32, unique: bool) -> bool {
    !unique && fuel <= if capability_policy { required } else { 1 } && (!capability_policy || required > 0)
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
        CLASS_MAGE => param(Param::attack_reserve_mage_sp),
        CLASS_RANGER => param(Param::attack_reserve_ranger_sp),
        CLASS_ROGUE => param(Param::attack_reserve_rogue_sp),
        CLASS_PRIEST => param(Param::attack_reserve_priest_sp),
        CLASS_PALADIN => param(Param::attack_reserve_paladin_sp),
        CLASS_NECROMANCER => param(Param::attack_reserve_necromancer_sp),
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
            && tr(st, BI_CLEVEL) <= param(Param::attack_spell_paranoia_max_level))
        && engine::randint0(100) < param(Param::attack_spell_paranoia_chance)
    {
        return 0;
    }

    /* Not if low on food */
    if low_on_food(st) {
        return 0;
    }

    /* Require ability (right now) */
    if !spell_okay_fail_noted(st, spell, if st.borg.near.unique != 0 { param(Param::attack_spell_unique_fail) } else { param(Param::attack_spell_fail) }) {
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
        && (!borg_spell_legal_fail(st, TELEPORT_SELF, param(Param::attack_primary_teleport_fail)) || tr(st, BI_MAXCLEVEL) <= param(Param::attack_primary_max_level))
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
        if tr(st, BI_MAXSP) < param(Param::attack_spell_low_max_sp) && spell_power > b_n {
            b_n -= spell_power;
        }

        /* Penalize use of reserve mana */
        if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_spell_reserve_divisor) {
            b_n -= spell_power * param(Param::attack_spell_reserve_penalty);
        }

        /* Penalize use of deep reserve mana */
        if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_spell_deep_reserve_divisor) {
            b_n -= spell_power * param(Param::attack_spell_deep_reserve_penalty);
        }
    }

    /* Really penalize use of mana needed for final teleport */
    let penalty = class_penalty(st);
    if tr(st, BI_MAXSP) > param(Param::attack_escape_reserve_min_max_sp) && tr(st, BI_CURSP) - spell_power < penalty {
        b_n -= spell_power * param(Param::attack_escape_reserve_penalty);
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the spell */
    let queued = borg_spell(st, spell);
    st.fight_attack.pending_unseen.command_queued(queued);

    /* Use target */
    borg_keypress(st, key(b'5'));

    /* Set our shooting flag */
    st.fight_attack.successful_target = -1;

    /* Value */
    b_n
}

/// borg-reborn-rs (#878): the mean paralysis of a cast `oops` mana short:
/// `randint1(5 * oops + 1)` (player-spell.c:541-553, player-util.c:835-842 at
/// 574b618c), whose mean is `(5 * oops + 2) / 2`. The counter falls in
/// `decrease_timeouts`, from `process_world`, once every 10 game turns
/// (game-world.c:651, :1080): world ticks, not the Borg's turns.
pub(crate) fn reserve_faint_turns(oops: i32) -> f64 {
    f64::from(5 * oops.max(0) + 2) / 2.0
}

/// borg-reborn-rs (#878, review of #891, F2): world ticks of paralysis as
/// Borg turns. A tick is 10 game turns; the Borg gains `extract_energy` of
/// its speed each game turn and needs 100 for a turn, so a tick is
/// `energy / 10` of its turns (1 at normal speed, 2 at +10, game-world.c:68-90).
/// The monsters' damage a Borg turn (`fight_incoming`) times this is their
/// damage over the faint.
pub(crate) fn faint_borg_turns(ticks: f64, player_energy: f64) -> f64 {
    ticks * player_energy / 10.0
}

/// borg-reborn-rs (#878): the expected damage taken while fainted for
/// `turns` Borg turns, from monsters `(HP a Borg turn, distance, moves a Borg
/// turn, weight)`: each from the turn it can reach the Borg (`distance - 1`
/// steps at its speed), weighted (the target only by the chance the cast
/// fails).
pub(crate) fn faint_contact_damage(monsters: &[(f64, i32, f64, f64)], turns: f64) -> f64 {
    monsters.iter().map(|&(hp, d, moves, weight)| {
        let reach = if d <= 1 { 0.0 } else if moves > 0.0 { f64::from(d - 1) / moves } else { f64::INFINITY };
        hp.max(0.0) * weight * (turns - reach).max(0.0)
    }).sum()
}

/// borg-reborn-rs (#878, review of #891): whether a monster `d` grids away is
/// not excluded outright from the faint's damage: adjacent, or able to move;
/// whether a mobile one arrives in time is the caller's `reach`.
pub(crate) fn faint_reaches(d: i32, never_move: bool) -> bool {
    d <= 1 || !never_move
}

/// borg-reborn-rs (#878): `attack_reserve_faint_percent`. The reserve spell
/// is refused when what the awake monsters deal during the mean faint
/// (`reserve_faint_turns`, in Borg turns by `faint_borg_turns`) reaches that
/// percent of the current HP. The target (`target`: the one monster the
/// reserve spell's own loop found within `attack_reserve_spell_near_distance`)
/// counts only if the cast fails; its kill is what the cast gains. Distances
/// are the reserve loop's (the larger of the two axes: the steps a monster
/// needs). Expected blow damage (`fight_incoming`), not danger: no fear
/// prices. Poison taken over time is not counted.
fn reserve_faint_refused(st: &mut BorgState, spell: borg_spells, mana: i32, target: Option<usize>) -> bool {
    let percent = param(Param::attack_reserve_faint_percent);
    if percent <= 0 {
        return false;
    }
    let oops = borg_get_spell_power(st, spell) - mana;
    if oops <= 0 {
        return false;
    }
    let fail = f64::from(crate::borg_magic::borg_spell_fail_rate(st, spell).clamp(0, 100)) / 100.0;
    let energy = |s: i32| f64::from(engine::extract_energy(s.clamp(0, 199) as usize)).max(1.0);
    let player = energy(engine::player_combat_speed());
    let turns = faint_borg_turns(reserve_faint_turns(oops), player);
    let mut monsters = Vec::new();
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 || !kill.awake {
            continue;
        }
        let d = (kill.pos.x - st.borg.c.x).abs().max((kill.pos.y - st.borg.c.y).abs());
        if !faint_reaches(d, engine::race_flag_has(usize::from(kill.r_idx), RF_NEVER_MOVE)) {
            continue;
        }
        let hp = crate::borg_caution::fight_incoming(st, i).hp_per_turn;
        let moves = energy(crate::borg_caution::forecast_monster_speed(&kill)) / player;
        let weight = if target == Some(i) { fail } else { 1.0 };
        monsters.push((hp, d, moves, weight));
    }
    let damage = faint_contact_damage(&monsters, turns);
    let hp = tr(st, BI_CURHP);
    if damage * 100.0 < f64::from(percent) * f64::from(hp) {
        return false;
    }
    // Once a think: the attack is simulated several times.
    thread_local! { static NOTED: std::cell::Cell<i32> = const { std::cell::Cell::new(i32::MIN) }; }
    let now = st.borg.time.now;
    if NOTED.with(|n| n.replace(now)) != now {
        borg_note(st, &format!("# No reserve spell: {damage:.1} damage expected over a {turns:.1}-turn faint, HP {hp} (#878)"));
    }
    true
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
    // borg-reborn-rs (#878): the one near monster, the cast's target.
    let mut near_target = None;

    /* Fake our Mana */
    let sv_mana = tr(st, BI_CURSP);

    /* Only Weak guys should try this */
    if tr(st, BI_CLEVEL) >= param(Param::attack_reserve_spell_max_level) {
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
    if spell_okay_fail_noted(st, spell, param(Param::attack_reserve_spell_fail)) {
        return 0;
    }

    // borg-reborn-rs (#980): this is the C Borg's deliberate cast beyond the
    // current mana. The engine always faints the caster for it
    // (player-spell.c:540-553: player_over_exert(PY_EXERT_FAINT, 100,
    // 5 * oops + 1), paralysis that bypasses free action) and adds 5% fail
    // per missing point (player-spell.c:404-406), which the faked mana below
    // hides. With attack_reserve_spell_overexert = 0 the Borg never casts
    // beyond its current SP.
    if !reserve_cast_allowed(param(Param::attack_reserve_spell_overexert), sv_mana, borg_get_spell_power(st, spell).max(1)) {
        return 0;
    }

    /* Must be dangerous */
    // borg-reborn-rs (#878): with `attack_reserve_spell_damage_only`, the
    // danger without its fear prices.
    let danger = if param(Param::attack_reserve_spell_damage_only) != 0 {
        let (y, x) = (st.borg.c.y, st.borg.c.x);
        crate::borg_danger::borg_danger_damage_only(st, y, x, param(Param::attack_reserve_spell_danger_horizon), true, false)
    } else {
        danger_here(st, param(Param::attack_reserve_spell_danger_horizon), true)
    };
    if danger < avoid(st) * param(Param::attack_reserve_spell_danger_multiplier) {
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
        if d < param(Param::attack_reserve_spell_near_distance) {
            near_monsters += 1;
            near_target = Some(i as usize);

            /* If it has too many hp to be taken out with this */
            /* spell, don't bother trying */
            /* NOTE: the +4 is because the damage is toned down
            as an 'average damage' */
            if i32::from(kill.power) > dam + param(Param::attack_reserve_spell_damage_margin) {
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

    // borg-reborn-rs (#878): no fainting next to what bites.
    if reserve_faint_refused(st, spell, sv_mana, near_target) {
        return 0;
    }

    /* Require ability (with faked mana) */
    st.borg.trait_[BI_CURSP as usize] = tr(st, BI_MAXSP);
    if !spell_okay_fail_noted(st, spell, param(Param::attack_reserve_spell_fail)) {
        /* Restore Mana */
        st.borg.trait_[BI_CURSP as usize] = sv_mana;
        return 0;
    }
    // borg-reborn-rs (review of #657): the cost was noted on the faked mana;
    // the casts the Borg can pay for come from its real mana.
    let power = borg_get_spell_power(st, spell).max(1);
    if let Some(c) = st.fight_attack.cost.as_mut() {
        c.uses = Some(sv_mana / power);
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
    let queued = borg_spell_fail(st, spell, param(Param::attack_reserve_spell_fail));
    st.fight_attack.pending_unseen.command_queued(queued);
    if queued {
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

/// borg-reborn-rs (#980): whether the emergency attack spell may be cast
/// with `cursp` mana for a spell of `cost` mana: always when `overexert` is
/// on (the C Borg's play), else only within the current mana.
fn reserve_cast_allowed(overexert: i32, cursp: i32, cost: i32) -> bool {
    overexert != 0 || cursp >= cost
}

#[cfg(test)]
mod reserve_tests {
    use super::reserve_cast_allowed;

    #[test]
    fn emergency_spell_never_casts_beyond_current_mana_when_overexertion_is_off() {
        // 592754909: Magic Missile (1 SP) with 0 SP faints and paralyses.
        assert!(!reserve_cast_allowed(0, 0, 1));
        assert!(!reserve_cast_allowed(0, 2, 3));
        assert!(reserve_cast_allowed(0, 1, 1), "within the current mana it is an ordinary cast");
        assert!(reserve_cast_allowed(1, 0, 1), "default: the C Borg's overexertion cast, today's play");
    }

    #[test]
    fn overexertion_default_retains_todays_play() {
        use crate::params::{Param, SPECS};
        assert_eq!(SPECS[Param::attack_reserve_spell_overexert as usize].default, 1);
    }
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
    if paranoia(st, param(Param::attack_dispel_paranoia_chance)) {
        return 0;
    }

    /* Require ability */
    if !spell_okay_fail_noted(st, spell, param(Param::attack_dispel_fail)) {
        return 0;
    }

    /* Choose optimal location--radius defined as 10 */
    let mut b_n = borg_launch_bolt(st, 10, dam, typ, max_range(), 0);

    let spell_power = borg_get_spell_power(st, spell);

    /* Penalize mana usage */
    b_n -= spell_power;

    /* Penalize use of reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_dispel_reserve_divisor) {
        b_n -= spell_power * param(Param::attack_dispel_reserve_penalty);
    }

    /* Penalize use of deep reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_dispel_deep_reserve_divisor) {
        b_n -= spell_power * param(Param::attack_dispel_deep_reserve_penalty);
    }

    /* Really penalize use of mana needed for final teleport */
    let penalty = class_penalty(st);
    if tr(st, BI_MAXSP) > param(Param::attack_dispel_escape_min_max_sp) && tr(st, BI_CURSP) - spell_power < penalty {
        b_n -= spell_power * param(Param::attack_dispel_escape_penalty);
    }

    /* never tap if we previously missed a tap */
    if st.fight_attack.target_closest < 0 && BORG_ATTACK_TAP_UNLIFE as i32 == typ && b_n > 0 {
        st.fight_attack.pending_unseen.clear();
        st.fight_attack.target_closest = 0;
        return 0;
    }

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the prayer */
    let queued = borg_spell(st, spell);
    st.fight_attack.pending_unseen.command_queued(queued);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using a "dispel" staff
/// (`borg_attack_aux_staff_dispel`): dispel evil, power, holiness.
/// Genocide handled later.
fn borg_attack_aux_staff_dispel(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    // borg-reborn-rs (#867): a staff needs no aim (no `obj_needs_aim`, so no
    // `player_confuse_dir`, cmd-obj.c:431-441): full value while confused.
    if cant_use_item_attack(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, param(Param::attack_wand_paranoia_chance)) {
        return 0;
    }

    /* look for the staff */
    if !borg_equips_staff_fail(st, sval) {
        return 0;
    }

    /* Choose optimal location--radius defined as 10 */
    let mut b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Big Penalize charge usage */
    b_n -= param(Param::attack_wand_charge_penalty);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Cast the prayer */
    let queued = borg_use_staff(st, sval);
    st.fight_attack.pending_unseen.command_queued(queued);

    /* Value */
    b_n
}

/// Simulate/Apply the optimal result of using a "normal" attack rod
/// (`borg_attack_aux_rod_bolt`).
fn borg_attack_aux_rod_bolt(st: &mut BorgState, sval: i32, rad: i32, dam: i32, typ: i32) -> i32 {
    /* No firing while blind, confused, or hallucinating */
    // borg-reborn-rs (#867): confused, by direction at an adjacent monster.
    if cant_use_item_attack(st) {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, param(Param::attack_rod_paranoia_chance)) {
        return 0;
    }

    /* Not likely to be successful in the activation */
    let failure = borg_activate_failure(st, TV_ROD as i32, sval);
    if param(Param::attack_rod_max_failure) < failure {
        return 0;
    }
    // A rod recharges over many turns: one zap in a fight (review of #657).
    note_device_cost(st, failure, Some(1));

    /* Look for that rod */
    if !borg_equips_rod(st, sval) {
        return 0;
    }

    /* Choose optimal location */
    mark_confused_aim(st);
    let b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);

    /* Simulation */
    if simulating(st) {
        return b_n;
    }

    /* Zap the rod */
    let queued = borg_zap_rod(st, sval);
    st.fight_attack.pending_unseen.command_queued(queued);

    /* Use target */
    /* Set our shooting flag */
    st.fight_attack.successful_target = aim_key(st, -1);

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
    // borg-reborn-rs (#867): confused, by direction at an adjacent monster.
    if cant_use_item_attack(st) {
        return 0;
    }

    /* Dont use wands in town, charges are too spendy */
    // borg-reborn-rs (#637): except at a dangerous town monster.
    let town = tr(st, BI_CDEPTH) == 0;
    if town && (!town_wand_allowed(st) || sval == st.item_val.sv_wand_wonder) {
        return 0;
    }
    // borg-reborn-rs (#637): in town, no wand of this kind means no
    // paranoia roll either. The roll draws the Borg's own RNG (`rand_local`,
    // swapped in around every think and carried across thinks, borg.rs
    // `generate_keypress`), so one extra roll shifts every later random Borg
    // decision: without the wand the Borg must play as with the switch off.
    if town && borg_slot(st, TV_WAND as i32, sval) < 0 {
        return 0;
    }

    /* Paranoia */
    if paranoia(st, param(Param::attack_unknown_device_paranoia_chance)) {
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
    let failure = borg_activate_failure(st, TV_WAND as i32, sval);
    if param(Param::attack_unknown_wand_max_failure) < failure {
        return 0;
    }
    note_device_cost(st, failure, Some(i32::from(st.items.items[i as usize].pval)));

    /* Choose optimal location */
    st.fight_attack.town_wand = town;
    mark_confused_aim(st);
    let mut b_n = borg_launch_bolt(st, rad, dam, typ, max_range(), 0);
    st.fight_attack.town_wand = false;

    /* Penalize charge usage */
    if tr(st, BI_CLEVEL) > param(Param::attack_unknown_wand_penalty_min_level) {
        b_n -= param(Param::attack_unknown_wand_penalty);
    }

    /* Wands of wonder are used in last ditch efforts.  They behave
     * randomly, so the best use of them is an emergency.  I have seen
     * borgs die from hill orcs with fully charged wonder wands.  Odds
     * are he could have taken the orcs with the wand.  So use them in
     * an emergency after all the borg_caution() steps have failed
     */
    if sval == st.item_val.sv_wand_wonder {
        /* check the danger */
        if b_n > 0 && danger_here(st, param(Param::attack_unknown_wand_danger_horizon), true) >= (avoid(st) * param(Param::attack_unknown_wand_danger_tenths) / 10) {
            /* make the wand appear deadly */
            b_n = param(Param::attack_unknown_wand_emergency_value);

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
    let queued = borg_aim_wand(st, sval);
    st.fight_attack.pending_unseen.command_queued(queued);

    /* Use target */
    /* Set our shooting flag */
    st.fight_attack.successful_target = aim_key(st, -1);

    /*  select the correct effect */
    select_effect(st, selection);

    /* Value */
    b_n
}

/// borg-reborn-rs (#485, #520): the divisor of a shot at an adjacent
/// non-unique: `attack_arrow_point_blank_divisor`, the class's own value
/// when it has one.
pub(crate) fn point_blank_divisor(_class: i32) -> i32 {
    // borg-reborn-rs (#520): the class's own divisor, if any, comes through
    // `param` (attack_arrow_point_blank_divisor@<class> or the class file).
    param(Param::attack_arrow_point_blank_divisor)
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
    if paranoia(st, param(Param::attack_activation_paranoia_chance)) {
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
    let queued = borg_activate_item(st, activation);
    st.fight_attack.pending_unseen.command_queued(queued);

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
    if paranoia(st, param(Param::attack_ring_paranoia_chance)) {
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
    let queued = borg_activate_ring(st, ring_name);
    st.fight_attack.pending_unseen.command_queued(queued);

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
    if paranoia(st, param(Param::attack_dragon_paranoia_chance)) {
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
    let queued = borg_activate_dragon(st, sval);
    st.fight_attack.pending_unseen.command_queued(queued);

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
    let mut true_best = 0;
    let mut kills_total = 0.0;

    /* Can I do it */
    if !spell_okay_fail_noted(st, WHIRLWIND_ATTACK, if st.borg.near.unique != 0 { param(Param::attack_spell_unique_fail) } else { param(Param::attack_spell_fail) }) {
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
        let d = if st.fight_attack.focused_target.is_some() { thrust_damage(st, k, blows) } else { borg_thrust_damage_one(st, k) };

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* get to do "blows" attacks; focused estimate already counts them. */
        let d = if st.fight_attack.focused_target.is_some() { d } else { d * blows };

        // borg-reborn-rs (re-review of #657): the fight test applies the
        // primary's damage to one foe at a time, so its true damage is the best
        // single foe's (at most its HP); the sum stays the ranking's. Each foe's
        // kills count for #556 (a sweep can kill several).
        let power = focused_power(st, st.kills.kills[k as usize].power);
        true_best = true_best.max(d.clamp(0, power.max(0)));
        kills_total += kill_fraction(d, power);

        let Some(d) = melee_score(st, k, x, y, d) else {
            continue;
        };

        total_d += d;
    }
    st.fight_attack.dmg_best = true_best;
    st.fight_attack.kill_best = kills_total;

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
    if !spell_okay_fail_noted(st, LEAP_INTO_BATTLE, if st.borg.near.unique != 0 { param(Param::attack_spell_unique_fail) } else { param(Param::attack_spell_fail) }) {
        return 0;
    }

    /* Too afraid to attack */
    if tr(st, BI_ISAFRAID) != 0 || tr(st, BI_CRSFEAR) != 0 {
        return 0;
    }

    if st.fight_attack.target_closest < param(Param::attack_leap_min_target_score) {
        return 0;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);
        i += 1;

        /* Require up to distance 4 */
        let m_dist = dist_to(st, x, y);
        if m_dist > param(Param::attack_leap_max_distance) {
            continue;
        }

        /* Acquire grid */
        let k = i32::from(grid(st, x, y).kill);

        if k == 0 {
            continue;
        }

        /* Calculate "average" damage */
        let mut blows = (tr(st, BI_CLEVEL) + 5) / 15;
        blows = if st.fight_attack.focused_target.is_some() {
            // Certify only an adjacent leap: no uncertain MOVE_ATTACK route
            // or distance-scaled blow count (effect-handler-attack.c:1789-1850).
            if m_dist != 1 { continue; }
            blows
        } else { ((blows * m_dist + 2) / 4) + 1 };
        let d = if st.fight_attack.focused_target.is_some() { thrust_damage(st, k, blows) } else { borg_thrust_damage_one(st, k) * blows };

        /* No damage */
        if d <= 0 {
            continue;
        }

        /* "player ghosts" */
        if is_ghost(st.kills.kills[k as usize].r_idx) {
            continue;
        }

        let true_d = d;
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
        // borg-reborn-rs (#584): the true damage, without the unique
        // targeting bonus of `borg_thrust_damage_one`.
        st.fight_attack.dmg_best = if st.fight_attack.focused_target.is_none() && rf(st.kills.kills[k as usize].r_idx, RF_UNIQUE) && tr(st, BI_CDEPTH) >= 1 {
            true_d / (1 + param(Param::attack_melee_unique_bonus).max(0))
        } else {
            true_d
        };
        st.fight_attack.kill_best = kill_fraction(st.fight_attack.dmg_best, i32::from(st.kills.kills[k as usize].power));
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
    if !spell_okay_fail_noted(st, MAIM_FOE, if st.borg.near.unique != 0 { param(Param::attack_spell_unique_fail) } else { param(Param::attack_spell_fail) }) {
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
        d = if st.fight_attack.focused_target.is_some() { thrust_damage(st, k, blows) } else { borg_thrust_damage_one(st, k) * blows };

        /* No damage */
        if d <= 0 {
            continue;
        }
        let true_d = d.clamp(0, focused_power(st, st.kills.kills[k as usize].power));

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
        // borg-reborn-rs (review of #657): the true damage of the best target.
        st.fight_attack.dmg_best = true_d;
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
    if !spell_okay_fail_noted(st, VAMPIRE_STRIKE, if st.borg.near.unique != 0 { param(Param::attack_spell_unique_fail) } else { param(Param::attack_spell_fail) }) {
        return 0;
    }

    /* Examine possible destinations */
    let mut i = 0;
    while i < i32::from(st.flow.temp_n) {
        let (y, x) = st.flow.temp_at(i as usize);

        // borg-reborn-rs (#621 review): never strike at an ignored monster.
        let k = grid(st, x, y).kill;
        if k != 0 && ignored(st, usize::from(st.kills.kills[usize::from(k)].r_idx)) {
            i += 1;
            continue;
        }

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
    if best_dist >= param(Param::attack_vampire_max_distance) {
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
    let mut d = tr(st, BI_CLEVEL) * 2 * if st.fight_attack.focused_target.is_some() { 1000 } else { 1 };

    /* Obtain the monster */
    let kill = st.kills.kills[k as usize];

    // borg-reborn-rs (review of #657): the true damage, before the danger
    // bonus below.
    st.fight_attack.dmg_best = d.clamp(0, focused_power(st, kill.power));

    /* "player ghosts" */
    if is_ghost(kill.r_idx) {
        return 0;
    }

    if rf(kill.r_idx, RF_NONLIVING) || rf(kill.r_idx, RF_UNDEAD) {
        return 0;
    }

    /* Avoid waking most "hard" sleeping monsters */
    if !kill.awake && d <= i32::from(kill.power) {
        /* Calculate danger */
        let p = borg_danger_one_kill(st, y, x, param(Param::attack_vampire_sleep_danger_horizon), k, true, true);

        if p > avoid(st) * param(Param::attack_vampire_sleep_danger_multiplier) {
            return 0;
        }
    }

    /* Calculate "danger" to player */
    let mut p = danger_one_here(st, param(Param::attack_vampire_danger_horizon), k, true);

    /* Reduce "bonus" of partial kills when higher level */
    if d <= i32::from(kill.power) && tr(st, BI_MAXCLEVEL) > param(Param::attack_vampire_partial_min_level) {
        p /= param(Param::attack_vampire_partial_danger_divisor);
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
    let p1 = danger_here(st, param(Param::attack_flag_danger_horizon), true);

    /* What effect is there? */
    set_flag(st, f, true);
    let p2 = danger_here(st, param(Param::attack_flag_danger_horizon), true);
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
    if (tr(st, BI_CURHP) + param(Param::attack_crush_hp_margin)) < (tr(st, BI_CLEVEL) * param(Param::attack_crush_hp_level_multiplier)) {
        return 0;
    }

    let (p1, p2) = danger_with_flag(st, SpellFlag::Crush);

    /* damage is reduction in danger */
    let mut d = p1 - p2;

    /* if there is still danger afterward, make sure the reductioning in HP */
    /* doesn't make this put us in danger */
    let new_hp = tr(st, BI_CURHP) - (tr(st, BI_CLEVEL) * 2);
    if simulating(st) && (p2 >= (new_hp * param(Param::attack_crush_post_danger_multiplier)) || new_hp <= param(Param::attack_crush_min_post_hp)) {
        return 0;
    }

    let spell_power = borg_get_spell_power(st, CRUSH);

    /* Penalize mana usage */
    d -= spell_power;

    /* Penalize use of reserve mana */
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_crush_reserve_divisor) {
        d -= spell_power * param(Param::attack_crush_reserve_penalty);
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
    if tr(st, BI_CURSP) - spell_power < tr(st, BI_MAXSP) / param(Param::attack_trance_reserve_divisor) {
        d -= spell_power * param(Param::attack_trance_reserve_penalty);
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
    // borg-reborn-rs (#889, Codex re-review of #1128 F2): only a ball gets a
    // ball's slack and blast-only aims; a dragon's breath is a cone.
    st.fight_attack.ball_shape = crate::borg_unseen_belief::ball_shaped(attack_type);
    let value = calculate_attack_effectiveness(st, attack_type);
    st.fight_attack.ball_shape = false;
    value
}

fn calculate_attack_effectiveness(st: &mut BorgState, attack_type: i32) -> i32 {
    // borg-reborn-rs (#867): set by the attack rated now, if it is aimed by
    // a confused Borg.
    clear_confused_aim(st);
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
            if tr(st, BI_MAXHP) - tr(st, BI_CURHP) >= param(Param::attack_holy_word_min_loss) {
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
            if tr(st, BI_CURHP) < param(Param::attack_curse_min_hp) {
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

        /* Rod / wand -- unid'd: borg-reborn-rs (#470) never uses an
         * unidentified device; town sells one to learn it. */
        BF_ROD_UNKNOWN | BF_WAND_UNKNOWN => 0,

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
            let dam = if tr(st, BI_CURHP) < tr(st, BI_MAXHP) / param(Param::attack_holy_staff_hp_divisor) { param(Param::attack_holy_staff_hurt_value) } else { param(Param::attack_holy_staff_value) };
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
            let dam = if tr(st, BI_CURHP) < tr(st, BI_MAXHP) / param(Param::attack_holy_activation_hp_divisor) { param(Param::attack_holy_activation_hurt_value) } else { param(Param::attack_holy_activation_value) };
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

/// borg-reborn-rs (#879, reviews of #890 F1/R4): `borg_attack`'s
/// adjacent-monster test, "next to me, or close and fast". A record only
/// believed at its last grid never counts, so it cannot turn a scary level's
/// flight into a fight. With `unseen_belief` = 0 nothing is believed and this
/// is C's test.
fn counts_as_adjacent(believed: bool, faster: bool, distance: i32, fast_distance: i32) -> bool {
    !believed && ((faster && distance <= fast_distance) || distance <= 1)
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
    let mut ranking = crate::attack_preference::Ranking::default();
    let mut adjacent_monster = false;
    // borg-reborn-rs (#539): `scary_ranged_no_stairs`, probed at most once
    let mut no_stairs: Option<bool> = None;

    /* Nobody around */
    if st.kills.cnt == 0 {
        return false;
    }

    /* Set the attacking flag so that danger is boosted for monsters */
    /* we want to attack first. */
    st.traits.attacking = true;

    /* Reset list */
    st.flow.temp_n = 0;
    st.fight_attack.blast_aims.clear();

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

        // #789 owns these targets while the town hunt is enabled.
        if crate::borg_town_hunt::controls_target(st, usize::from(r)) {
            continue;
        }

        /* Require current knowledge */
        // borg-reborn-rs (#879): or a monster believed at its last grid;
        // #889 (review of #1128 F4): or one a blast could still reach.
        if borg_timer(st, kill.when) > param(Param::attack_max_target_age)
            && crate::borg_unseen_belief::believed_for_aim(st, k as usize).is_none()
        {
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
            && tr(st, BI_MAXCLEVEL) < param(Param::attack_town_mage_max_level)
            && tr(st, BI_CDEPTH) == 0
            && contains(name, b"Farmer")
        {
            continue;
        }

        /* Check if there is a monster adjacent to me or he's close and
         * fast. */
        // borg-reborn-rs (#879, review of #890 F1): a monster only believed
        // at its last grid is not "next to me": it never turns a scary
        // level's flight into a fight.
        if counts_as_adjacent(
            crate::borg_unseen_belief::believed_for_aim(st, k as usize).is_some(),
            i32::from(kill.speed) > tr(st, BI_SPEED),
            engine::distance(st.borg.c, kill.pos),
            param(Param::attack_fast_monster_distance),
        ) {
            adjacent_monster = true;
        }

        /* no attacking most scaryguys, try to get off the level */
        if st.borg.mon.scary {
            if contains(name, b"Grip") || contains(name, b"Fang") {
                /* probably Grip or Fang. Try to fight Grip and Fang. */
            } else if tr(st, BI_CDEPTH) <= param(Param::attack_scary_breeder_max_depth) && tr(st, BI_CDEPTH) != 0 && rf(r, RF_MULTIPLY) {
                /* Try to fight single worms and mice. */
            } else if borg_timer(st, st.borg.time.level) >= param(Param::attack_scary_max_level_time) || borg_timer(st, st.borg.time.town) >= param(Param::attack_scary_max_town_time) {
                /* Try to fight been there too long. */
            } else if boosted_bravery
                || st.borg.no_retreat >= 1
                || crate::borg_caution::recall_restricts(st)
                || st.borg.goal.descending != 0
            {
                /* Try to fight if being Boosted or recall engaged. */
                borg_note(st, "# Bored, or recalling and fighting a monster on Scaryguy Level.");
            } else if tr(st, BI_CDEPTH) * param(Param::attack_scary_depth_multiplier) <= tr(st, BI_CLEVEL) && tr(st, BI_CLEVEL) > param(Param::attack_scary_min_level) {
                /* Try to fight anyway. */
                borg_note(st, "# High clevel fighting monster on Scaryguy Level.");
            } else if adjacent_monster {
                /* Try to fight if there is a monster next to me */
                borg_note(st, "# Adjacent to monster on Scaryguy Level.");
            } else if crate::borg_caution::ranged_candidate(st, &kill)
                && *no_stairs.get_or_insert_with(|| crate::borg_caution::scary_ranged_no_stairs(st))
            {
                /* borg-reborn-rs (#539, owner): no staircase to flee to, so
                 * the best attack on an awake monster in view at range
                 * (the one Recall and Phase Door wait for). */
                borg_note(st, "# No stairs: fighting at range on Scaryguy Level.");
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
        // borg-reborn-rs (#889, Codex re-review of #1128 F1): a record only
        // a ball could reach goes to the ball's own list, never to the aim
        // list every other attack (Vampire Strike, melee, bolts) walks.
        if crate::borg_unseen_belief::blast_only(st, k as usize) {
            st.fight_attack.blast_aims.push((kill.pos.y, kill.pos.x));
            continue;
        }
        st.flow.temp_push(kill.pos.y, kill.pos.x);
    }

    // borg-reborn-rs (#736, owner: "choosing the enemy you can take down is
    // the wrong idea. Here it is obvious better to target the mobile one ...
    // I would flask the acolyte, who can kill me"): with
    // `attack_mobile_threat_first`, while a mobile monster among the targets
    // is a threat (its danger at least `attack_mobile_threat_percent` of the
    // HP), the immobile ones are not targets. They cannot follow; the Borg
    // keeps away from them instead. 0 = C.
    if param(Param::attack_mobile_threat_first) != 0 && st.flow.temp_n > 1 {
        let percent = param(Param::attack_mobile_threat_percent);
        let hp = tr(st, BI_CURHP).max(1);
        let (cy, cx) = (st.borg.c.y, st.borg.c.x);
        let targets: Vec<(i32, i32, usize)> = (0..st.flow.temp_n as usize)
            .map(|t| {
                let (y, x) = (i32::from(st.flow.temp_y[t]), i32::from(st.flow.temp_x[t]));
                (y, x, usize::from(grid(st, x, y).kill))
            })
            .collect();
        let immobile = |st: &BorgState, k: usize| k != 0 && rf(st.kills.kills[k].r_idx, RF_NEVER_MOVE);
        let mut threat = false;
        for &(_, _, k) in &targets {
            if k != 0 && !immobile(st, k) && crate::borg_danger::borg_danger_one_kill(st, cy, cx, 1, k as i32, true, false) * 100 >= hp * percent {
                threat = true;
                break;
            }
        }
        if threat && targets.iter().any(|&(_, _, k)| immobile(st, k)) {
            st.flow.temp_n = 0;
            for &(y, x, k) in &targets {
                if !immobile(st, k) {
                    st.flow.temp_push(y, x);
                }
            }
            borg_note(st, "# Targeting the mobile threat, not the immobile monsters (#736)");
        }
    }

    /* No destinations */
    // borg-reborn-rs (#889): a ball may still have a blast-only aim.
    if st.flow.temp_n == 0 && st.fight_attack.blast_aims.is_empty() {
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

    let mut attack_trace = crate::core_trace::attack_ranking();
    let mut winning_raw = 0;
    let mut winning_damage = 0;
    let mut winning_cost = None;
    let mut no_mana: Vec<(i32, i32)> = Vec::new();
    let mut winning_kills = 0.0;
    /* Analyze the possible attacks */
    for g in 0..max_attacks {
        /* Simulate */
        st.fight_attack.dmg_best = 0;
        st.fight_attack.cost = None;
        st.fight_attack.kill_best = 0.0;
        let raw = borg_calculate_attack_effectiveness(st, g);
        // borg-reborn-rs (#867): a confused aimed attack at its aim chance.
        // Not a blow's damage (`confused_weight`): fight-or-flee as in C.
        let raw = confused_weight(st, raw, g != BF_THRUST as i32);
        // borg-reborn-rs (#625): the candidates that need no mana, for the
        // fight test's fallback once the chosen attack's uses run out.
        if raw > 0 && !st.fight_attack.cost.is_some_and(|c| c.spell) {
            no_mana.push((g, st.fight_attack.dmg_best));
        }

        // borg-reborn-rs (#403): apply preference once, only during ranking.
        // Helpers keep eligibility, RNG, costs and internal target selection.
        let class = tr(st, BI_CLASS);
        let candidate = crate::attack_preference::evaluate(class, g, raw);
        let changed = ranking.consider(g, &candidate);
        if changed {
            winning_raw = raw;
            winning_damage = st.fight_attack.dmg_best;
            winning_cost = st.fight_attack.cost;
            winning_kills = st.fight_attack.kill_best;
        }
        if let Some(trace) = attack_trace.as_mut() {
            trace.candidate(class, g, raw, &candidate, changed);
        }
    }

    crate::core_trace::record_attack_ranking(attack_trace, ranking.winner);
    let Some(b_g) = ranking.winner else {
        st.traits.attacking = false;
        return false;
    };
    let b_n = ranking.utility;

    // borg-reborn-rs (#584): an estimate only (`borg_attack_estimate`): the
    // winner's true damage, nothing done.
    if st.traits.estimate_only {
        st.traits.estimate = Some(winning_damage);
        st.fight_attack.est_cost = winning_cost;
        // The fallback is never the winner itself (review of #657: a wand
        // would otherwise outlive its charges).
        st.fight_attack.est_fallback = fallback_without(&no_mana, b_g);
        st.fight_attack.est_kills = winning_kills;
        st.traits.simulate = false;
        st.traits.attacking = false;
        return false;
    }

    // borg-reborn-rs (#551, owner): a fox changes back before a melee blow.
    // The other attacks are item or spell commands, changed back by the
    // game's prompt; Maim Foe's blow is a Blackguard spell, and only a
    // Druid has Fox Form (class.txt), so a fox never gets there.
    if b_g == BF_THRUST as i32 && crate::borg_escape::revert_fox_to_fight(st) {
        st.traits.simulate = false;
        st.traits.attacking = false;
        return true;
    }

    // borg-reborn-rs (#469): committed to this fight (not waiting); when the
    // danger is very high, drink Speed first and attack next turn.
    if b_g != BF_REST as i32 {
        // The ranking above ran simulated (simulate was set true before it);
        // the quaff must act. Not restored: every path from here acts, and
        // the attack below clears it the same way ("Instantiate").
        st.traits.simulate = false;
        if borg_fight_speed(st) {
            st.traits.attacking = false;
            return true;
        }
    }

    /* Note */
    if winning_raw == b_n {
        borg_note(st, &format!("# Performing attack type {} with value {}.", b_g, b_n));
    } else {
        borg_note(st, &format!("# Performing attack type {} with raw utility {} and weighted utility {}.", b_g, winning_raw, b_n));
    }

    /* Instantiate */
    st.traits.simulate = false;

    /* Instantiate */
    let _ = borg_calculate_attack_effectiveness(st, b_g);
    clear_confused_aim(st);

    st.traits.attacking = false;

    /* Success */
    true
}

/// player-attack.c:196-219 at 574b618c. Game constants, not policy cutoffs.
pub(crate) fn hit_permyriad(chance: i32, ac: i32) -> i32 {
    let chance = i64::from(chance.max(9));
    let rate = 10_000 * (chance - i64::from(ac) * 2 / 3).max(0) / chance;
    (rate * 8300 / 10_000 + 1200) as i32
}

fn focused_power(st: &BorgState, power: i16) -> i32 {
    match st.fight_attack.focused_target {
        None => i32::from(power).max(0),
        Some(i) => observed_kill_hp(&st.kills.kills[i]).unwrap_or(i32::from(power).max(1)) * 1000,
    }
}

/// mon-util.c:1300 at 574b618c: a monster dies below zero HP, so one at
/// zero still needs a point of damage. Unknown health cannot certify a win.
pub(crate) fn observed_kill_hp(kill: &crate::borg_flow_kill::BorgKill) -> Option<i32> {
    kill.observed_health.filter(|&(hp, max)| hp >= 0 && max > 0).map(|(hp, _)| i32::from(hp) + 1)
}

/// HP/shot -> HP/Borg turn. Engine ranged_damage does not multiply the
/// damage of an arrow by shooting rate (player-attack.c:554-582); firing
/// instead costs 10/num_shots of a turn (player-attack.c:1087).
pub(crate) fn focused_projectile_rate(per_shot: i64, num_shots: i32) -> i64 {
    per_shot * i64::from(num_shots.max(0)) / 10
}

/// #759: real damage for the focused planner, in millihitpoints. The
/// native planner's targeting bonuses, missile conservation and point-blank
/// preferences are not damage. Non-damaging projections score zero.
fn focused_launch_damage(st: &BorgState, i: usize, damage: i32, typ: i32, slot: i32) -> i32 {
    let kill = st.kills.kills[i];
    let r = kill.r_idx;
    let race = engine::race(usize::from(r));
    let mut damage = i64::from(damage.max(0)) * if typ as u32 == BORG_ATTACK_ARROW && !st.fight_attack.throwing { 1 } else { 1000 };
    match typ as u32 {
        BORG_ATTACK_ARROW => {
            let item = st.items.items[slot as usize];
            let dist = engine::distance(st.borg.c, kill.pos);
            let chance;
            if st.fight_attack.throwing {
                let throwing = engine::flag_has(&item.flags, OF_THROWING as i32);
                chance = if throwing { tr(st, BI_THT) + 3 * (tr(st, BI_TOHIT) + i32::from(item.to_h)) }
                    else { 3 * tr(st, BI_THT) / 2 + 3 * i32::from(item.to_h) } - dist;
                let mult = borg_best_mult(&item, r);
                if mult > 1 { damage *= i64::from(1 + mult); }
            } else {
                let bow = st.items.items[(inven_wield() + 1) as usize];
                chance = tr(st, BI_THB) + 3 * (tr(st, BI_TOHIT) + i32::from(item.to_h) + i32::from(bow.to_h)) - dist;
                // player-attack.c:554-582: a brand/slay ADDS to the
                // launcher multiplier; it does not multiply it again.
                let mult = borg_best_mult(&bow, r).max(borg_best_mult(&item, r));
                let power = tr(st, BI_AMMO_POWER).max(1);
                if mult > 1 { damage = damage * i64::from(power + mult) / i64::from(power); }
            }
            // Cap damage per hit BEFORE hit chance: overkill never buys
            // probability of killing, especially on a finishing shot.
            damage = damage.min(i64::from(focused_power(st, kill.power)));
            damage = damage * i64::from(hit_permyriad(chance, race.ac)) / 10_000;
            if !st.fight_attack.throwing {
                damage = focused_projectile_rate(damage, engine::player_attack_rates().1);
            }
        }
        BORG_ATTACK_ACID if rf(r, RF_IM_ACID) => damage = 0,
        BORG_ATTACK_ELEC if rf(r, RF_IM_ELEC) => damage = 0,
        BORG_ATTACK_FIRE if rf(r, RF_IM_FIRE) => damage = 0,
        BORG_ATTACK_COLD | BORG_ATTACK_ICE if rf(r, RF_IM_COLD) => damage = 0,
        BORG_ATTACK_POIS if rf(r, RF_IM_POIS) => damage = 0,
        BORG_ATTACK_FIRE if rf(r, RF_HURT_FIRE) => damage *= 2,
        BORG_ATTACK_COLD | BORG_ATTACK_ICE if rf(r, RF_HURT_COLD) => damage *= 2,
        BORG_ATTACK_HOLY_ORB if rf(r, RF_EVIL) => damage *= 2,
        BORG_ATTACK_OLD_DRAIN | BORG_ATTACK_DRAIN_LIFE
            if rf(r, RF_UNDEAD) || rf(r, RF_NONLIVING) => damage = 0,
        BORG_ATTACK_TAP_UNLIFE if !rf(r, RF_UNDEAD) => damage = 0,
        BORG_ATTACK_DISP_UNDEAD if !rf(r, RF_UNDEAD) => damage = 0,
        BORG_ATTACK_DISP_EVIL if !rf(r, RF_EVIL) => damage = 0,
        BORG_ATTACK_DISP_SPIRITS if !rf(r, RF_SPIRIT) => damage = 0,
        BORG_ATTACK_LIGHT_WEAK if !rf(r, RF_HURT_LIGHT) => damage = 0,
        BORG_ATTACK_KILL_WALL if !rf(r, RF_HURT_ROCK) => damage = 0,
        BORG_ATTACK_NETHER if rf(r, RF_UNDEAD) => damage = 0,
        BORG_ATTACK_NETHER if rsf(r, RSF_BR_NETH) => damage /= 3,
        BORG_ATTACK_NETHER if rf(r, RF_EVIL) => damage /= 2,
        BORG_ATTACK_HOLY_WORD if !rf(r, RF_EVIL) => damage = 0,
        BORG_ATTACK_GRAVITY if rsf(r, RSF_BR_GRAV) => damage /= 3,
        BORG_ATTACK_SHARD if rsf(r, RSF_BR_SHAR) => damage /= 3,
        BORG_ATTACK_SOUND if rsf(r, RSF_BR_SOUN) => damage /= 3,
        BORG_ATTACK_PLASMA if rsf(r, RSF_BR_PLAS) => damage /= 3,
        BORG_ATTACK_DISEN if rf(r, RF_IM_DISEN) => damage /= 3,
        BORG_ATTACK_NEXUS if rsf(r, RSF_BR_NEXU) => damage /= 3,
        BORG_ATTACK_FORCE if rsf(r, RSF_BR_WALL) => damage /= 3,
        BORG_ATTACK_INERTIA if rsf(r, RSF_BR_INER) => damage /= 3,
        BORG_ATTACK_TIME if rsf(r, RSF_BR_TIME) => damage /= 3,
        BORG_ATTACK_LIGHT if rsf(r, RSF_BR_LIGHT) => damage /= 3,
        BORG_ATTACK_DARK if rsf(r, RSF_BR_DARK) => damage /= 3,
        BORG_ATTACK_WATER if rf(r, RF_IM_WATER) => damage = 0,
        BORG_ATTACK_MISSILE | BORG_ATTACK_MANA | BORG_ATTACK_METEOR |
        BORG_ATTACK_ACID | BORG_ATTACK_ELEC | BORG_ATTACK_FIRE | BORG_ATTACK_COLD |
        BORG_ATTACK_POIS | BORG_ATTACK_ICE | BORG_ATTACK_HOLY_ORB |
        BORG_ATTACK_DISP_UNDEAD | BORG_ATTACK_DISP_EVIL | BORG_ATTACK_DISP_SPIRITS |
        BORG_ATTACK_LIGHT_WEAK | BORG_ATTACK_KILL_WALL |
        BORG_ATTACK_OLD_DRAIN | BORG_ATTACK_DRAIN_LIFE | BORG_ATTACK_TAP_UNLIFE |
        BORG_ATTACK_HOLY_WORD | BORG_ATTACK_NETHER | BORG_ATTACK_GRAVITY |
        BORG_ATTACK_SHARD | BORG_ATTACK_SOUND | BORG_ATTACK_PLASMA |
        BORG_ATTACK_DISEN | BORG_ATTACK_NEXUS | BORG_ATTACK_FORCE |
        BORG_ATTACK_INERTIA | BORG_ATTACK_TIME | BORG_ATTACK_LIGHT | BORG_ATTACK_DARK |
        BORG_ATTACK_WATER => {},
        // Unmodelled resistance/status interactions cannot certify a win.
        _ => damage = 0,
    }
    if typ as u32 != BORG_ATTACK_ARROW {
        damage = damage.min(i64::from(focused_power(st, kill.power)));
    }
    damage.min(i64::from(i32::MAX)) as i32
}

/// #760: category lookup. The switch is checked before loading the list or
/// consulting breeder policy, so the control never performs extra work.
pub(crate) fn attack_on_sight_reason(st: &BorgState, r: usize) -> Option<&'static str> {
    if param(Param::attack_on_sight) == 0 || r == 0 || r >= engine::r_max().saturating_sub(1) {
        return None;
    }
    let list = crate::scary_list::selected()?;
    crate::scary_list::attack_reason(list, r, engine::race_base_name(r), crate::scary_list::Situation {
        clevel: tr(st, BI_CLEVEL), class: tr(st, BI_CLASS),
        breeder_count: i32::from(st.borg.mon.breeders),
        breeder_stand: !st.borg.breeders_lost && crate::borg_caution::breeder_stand(st),
        levels_less: 0,
    }).filter(|reason| *reason != crate::borg_town_hunt::REASON)
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FocusedAttack {
    pub kind: i32,
    /// Expected millihitpoints per Borg turn, AFTER firing/blow rate scaling.
    pub hp_per_turn_milli: i32,
    pub cost: Option<AttackCost>,
    pub fallback: i32,
    /// Borg turns consumed by one use (shots use fractional energy).
    pub duration: f64,
}

fn focused_priority(kind: i32) -> i32 {
    use crate::attack_preference::Family;
    match crate::attack_preference::classify(kind).map(|k| k.family) {
        Some(Family::Projectile) => 4,
        Some(Family::Spell | Family::Device | Family::Activation) => 3,
        Some(Family::Thrown) => 2,
        Some(Family::Melee) => 1,
        _ => 0,
    }
}

// Share the HP cap before hit chance; count all fixed spell blows without
// counting the same possible overkill repeatedly. Values are millihitpoints.
fn focused_blow_expectation(damage: i32, hp: i32, chance: i32, blows: i32) -> i64 {
    i64::from(damage.max(0).min(hp.max(0) / blows.max(1))) * i64::from(chance) / 10_000
}

fn focused_score(a: FocusedAttack) -> (i64, i32) {
    let expected = i64::from(a.hp_per_turn_milli) * i64::from(100 - a.cost.map_or(0, |c| c.fail_pct));
    (expected, focused_priority(a.kind))
}

pub(crate) fn town_melee_damage(st: &BorgState, target: usize) -> i32 {
    thrust_damage_with(st, target as i32, 1, Some(BF_THRUST as i32))
}

/// borg-reborn-rs (#933, review of #967): the expected melee damage a turn
/// at `target`, in millihitpoints, without the targeting preferences of
/// `borg_thrust_damage_one` (the unique bonus and the rest): the real hit
/// chance and blows (`thrust_damage_with`'s focused branch).
pub(crate) fn melee_milli_per_turn(st: &BorgState, target: usize) -> i32 {
    thrust_damage_with(st, target as i32, 1, Some(BF_THRUST as i32))
}

pub(crate) fn focused_attack_is_ranged(attack: FocusedAttack) -> bool {
    focused_priority(attack.kind) > 1 && !matches!(attack.kind as u32,
        BF_SPELL_WHIRLWIND_ATTACK | BF_SPELL_LEAP_INTO_BATTLE | BF_SPELL_MAIM_FOE)
}

fn town_focused_score(attack: FocusedAttack, bonus: i32) -> (i64, i32) {
    let (damage, priority) = focused_score(attack);
    let percent = if focused_attack_is_ranged(attack) { 100 + i64::from(bonus) } else { 100 };
    (damage * percent, priority)
}

pub(crate) fn town_focused_attack(
    st: &mut BorgState, target: usize, bonus: i32, allowed: impl Fn(FocusedAttack) -> bool,
) -> Option<FocusedAttack> {
    focused_attack_ranked(st, target, |attack| {
        allowed(attack).then(|| town_focused_score(attack, bonus))
    })
}

#[cfg(test)]
mod focused_tests {
    use super::*;

    #[test]
    fn fixed_spell_blows_are_counted_without_repeating_the_hp_budget() {
        let blow = |hp, n| focused_blow_expectation(4000, hp, 5000, n) * i64::from(n);
        assert_eq!(blow(100_000, 1), 2000);
        assert_eq!(blow(100_000, 3), 6000, "CL45 Maim/Sweep: three real blows, not one");
        assert_eq!(blow(6000, 3), 3000, "cap before hit chance, not after adding expected blows");
        assert!(blow(1000, 3) <= 500);
    }

    #[test]
    fn expected_damage_precedes_family_tiebreak() {
        let missile = FocusedAttack { kind: BF_LAUNCH as i32, hp_per_turn_milli: 2000, cost: None, fallback: 0, duration: 1.0 };
        let melee = FocusedAttack { kind: BF_THRUST as i32, hp_per_turn_milli: 9000, cost: None, fallback: 0, duration: 1.0 };
        assert!(focused_score(melee) > focused_score(missile), "best damage first, including attack-on-sight");
        let equal_missile = FocusedAttack { hp_per_turn_milli: 9000, ..missile };
        assert!(focused_score(equal_missile) > focused_score(melee), "missile wins only a damage tie");
        let oil = FocusedAttack { kind: BF_OBJECT as i32, hp_per_turn_milli: 8000, ..missile };
        assert!(focused_score(oil) > focused_score(missile), "pilot crow regression: oil must beat weak pebbles");
        let spell = FocusedAttack { kind: BF_SPELL_MAGIC_MISSILE as i32, hp_per_turn_milli: 10_000,
            cost: Some(AttackCost { fail_pct: 90, uses: Some(1), spell: true }), fallback: 0, duration: 1.0 };
        assert!(focused_score(missile) > focused_score(spell), "spell failure counts");
        assert_eq!(hit_permyriad(100, 0), 9500);
        assert_eq!(hit_permyriad(9, 99), 1200);
        assert_eq!(hit_permyriad(100, 60), 6180);
    }

    #[test]
    fn town_ranged_bonus_can_break_a_close_comparison_but_melee_can_win() {
        let melee = FocusedAttack { kind: BF_THRUST as i32, hp_per_turn_milli: 9000,
            cost: None, fallback: 0, duration: 1.0 };
        let oil = FocusedAttack { kind: BF_OBJECT as i32, hp_per_turn_milli: 8000, ..melee };
        let arrow = FocusedAttack { kind: BF_LAUNCH as i32, hp_per_turn_milli: 4000, ..melee };
        assert!(town_focused_score(melee, 0) > town_focused_score(oil, 0));
        assert!(town_focused_score(oil, 50) > town_focused_score(melee, 50));
        assert!(town_focused_score(melee, 50) > town_focused_score(arrow, 50));
        assert!(!focused_attack_is_ranged(FocusedAttack { kind: BF_SPELL_MAIM_FOE as i32, ..melee }));
    }

    #[test]
    fn observed_hp_is_not_the_rounded_health_bar() {
        let mut kill = crate::borg_flow_kill::BorgKill::default();
        assert_eq!(observed_kill_hp(&kill), None);
        kill.power = 3;
        kill.injury = 60;
        kill.observed_health = Some((4, 11));
        assert_eq!(observed_kill_hp(&kill), Some(5), "engine dies below zero, not at zero");
        kill.observed_health = Some((0, 11));
        assert_eq!(observed_kill_hp(&kill), Some(1), "zero-HP monsters are still alive");
        kill.observed_health = Some((-1, 11));
        assert_eq!(observed_kill_hp(&kill), None);
    }
}

/// #760/#759: rate every available attack against one observed monster.
/// Restore RNG, simulation flags, targets and goals after the dry run.
/// Native eligibility and aim checks remain in force; no attack is issued.
pub(crate) fn focused_attack(st: &mut BorgState, target: usize) -> Option<FocusedAttack> {
    // borg-reborn-rs (#869): only the attack kinds the monster's attack entry
    // allows (all without one); the best of them by the usual score.
    let kinds = listed_attack_kinds(st, usize::from(st.kills.kills[target].r_idx));
    focused_attack_ranked(st, target, |attack| kind_allowed(kinds, attack.kind).then(|| focused_score(attack)))
}

/// borg-reborn-rs (#878): the best ranged attack at `target` (missiles,
/// throws, spells, devices, activations: `focused_attack_is_ranged`), among
/// the kinds its scary-list entry allows, by the usual score.
pub(crate) fn focused_ranged_attack(st: &mut BorgState, target: usize) -> Option<FocusedAttack> {
    let kinds = listed_attack_kinds(st, usize::from(st.kills.kills[target].r_idx));
    focused_attack_ranked(st, target, |attack| {
        (kind_allowed(kinds, attack.kind) && focused_attack_is_ranged(attack)).then(|| focused_score(attack))
    })
}

/// #869: the attack kinds race `r` may be attacked with, from the scary
/// list's attack entry for the character (`scary_list::attack_kinds_for`);
/// every kind without one.
fn listed_attack_kinds(st: &BorgState, r: usize) -> u8 {
    if r == 0 || r >= engine::r_max().saturating_sub(1) {
        return crate::scary_list::ALL_ATTACK_KINDS;
    }
    crate::scary_list::selected().map_or(crate::scary_list::ALL_ATTACK_KINDS, |list| {
        crate::scary_list::attack_kinds_for(list, r, engine::race_base_name(r), crate::scary_list::Situation {
            clevel: tr(st, BI_CLEVEL), class: tr(st, BI_CLASS),
            breeder_count: i32::from(st.borg.mon.breeders),
            breeder_stand: !st.borg.breeders_lost && crate::borg_caution::breeder_stand(st),
            levels_less: 0,
        })
    })
}

/// #869: whether attack `kind` is among `kinds` (`scary_list::ATTACK_KINDS`
/// bits, by `attack_preference::Family`). A wait is never a focused attack.
fn kind_allowed(kinds: u8, kind: i32) -> bool {
    use crate::attack_preference::Family;
    let bit = match crate::attack_preference::classify(kind).map(|k| k.family) {
        Some(Family::Melee) => 0,
        Some(Family::Projectile) => 1,
        Some(Family::Thrown) => 2,
        Some(Family::Spell) => 3,
        Some(Family::Device) => 4,
        Some(Family::Activation) => 5,
        Some(Family::Wait) | None => return false,
    };
    kinds & (1 << bit) != 0
}

fn focused_attack_ranked(
    st: &mut BorgState, target: usize, rank: impl Fn(FocusedAttack) -> Option<(i64, i32)>,
) -> Option<FocusedAttack> {
    let kill = st.kills.kills[target];
    if kill.r_idx == 0 || is_ghost(kill.r_idx) || borg_timer(st, kill.when) > param(Param::attack_max_target_age) {
        return None;
    }
    let ag = grid(st, kill.pos.x, kill.pos.y);
    if ag.info & (BORG_OKAY | BORG_VIEW) != (BORG_OKAY | BORG_VIEW) || engine::distance(st.borg.c, kill.pos) > max_range() {
        return None;
    }
    let rng = engine::rand_state();
    let traits = st.traits.clone();
    let fight = st.fight_attack.clone();
    let goal = st.borg.goal.clone();
    let projection = st.projection.target_loc;
    let temp = (st.flow.temp_n, st.flow.temp_x[0], st.flow.temp_y[0]);
    st.flow.temp_n = 0;
    st.flow.temp_push(kill.pos.y, kill.pos.x);
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = target as u8;
    st.fight_attack.focused_target = Some(target);
    st.traits.simulate = true;
    st.traits.attacking = true;
    let mut best: Option<((i64, i32), FocusedAttack)> = None;
    let mut fallback = 0;
    let max = if engine::player_opt(OPT_birth_randarts) { BF_MAX } else { BF_ACT_STARLIGHT };
    for kind in 0..max as i32 {
        st.fight_attack.focused_kind = kind;
        st.fight_attack.dmg_best = 0;
        st.fight_attack.cost = None;
        let utility = borg_calculate_attack_effectiveness(st, kind);
        // borg-reborn-rs (#867): a confused aimed attack at its aim chance.
        let utility = confused_weight(st, utility, true);
        let damage = st.fight_attack.dmg_best;
        let mut cost = st.fight_attack.cost;
        if kind == BF_OBJECT as i32 && cost.is_none() && param(Param::attack_throw_surplus) == 0 {
            let (slot, _) = best_throw(st);
            cost = Some(AttackCost { fail_pct: 0, uses: Some(if slot < 0 { 0 } else { i32::from(st.items.items[slot as usize].iqty) }), spell: false });
        } else if kind == BF_LAUNCH as i32 {
            let uses = (quiver_start()..quiver_end()).filter_map(|slot| {
                let item = st.items.items[slot as usize];
                (item.iqty > 0 && i32::from(item.tval) == tr(st, BI_AMMO_TVAL)).then_some(i32::from(item.iqty))
            }).min().unwrap_or(0);
            cost = Some(AttackCost { fail_pct: 0, uses: Some(uses), spell: false });
        } else if focused_priority(kind) != 1 && cost.is_none() {
            cost = Some(AttackCost { fail_pct: 0, uses: Some(1), spell: false });
        }
        if utility <= 0 || damage <= 0 || focused_priority(kind) == 0 {
            continue;
        }
        if cost.is_none() { fallback = fallback.max(damage); }
        let duration = if kind == BF_LAUNCH as i32 { 10.0 / f64::from(engine::player_attack_rates().1.max(1)) } else { 1.0 };
        let candidate = FocusedAttack { kind, hp_per_turn_milli: damage, cost, fallback: 0, duration };
        if let Some(score) = rank(candidate).filter(|&score| best.is_none_or(|b| score > b.0)) {
            best = Some((score, candidate));
        }
    }
    engine::set_rand_state(rng.0, rng.1);
    st.traits = traits;
    st.fight_attack = fight;
    st.borg.goal = goal;
    st.projection.target_loc = projection;
    st.flow.temp_n = temp.0;
    st.flow.temp_x[0] = temp.1;
    st.flow.temp_y[0] = temp.2;
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = ag.kill;
    best.map(|(_, mut b)| { b.fallback = fallback; b })
}

pub(crate) fn perform_focused_attack(st: &mut BorgState, target: usize, attack: FocusedAttack) -> bool {
    // As in the native planner: a fox changes back before a weapon blow.
    // The revert costs no energy; recompute the plan in normal form on the
    // next think rather than executing the weapon estimate as a fox bite.
    if attack.kind == BF_THRUST as i32 && crate::borg_escape::revert_fox_to_fight(st) {
        return true;
    }
    let kill = st.kills.kills[target];
    st.flow.temp_n = 0;
    st.flow.temp_push(kill.pos.y, kill.pos.x);
    let prior_kill = st.cave.grid(kill.pos.x, kill.pos.y).kill;
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = target as u8;
    st.fight_attack.focused_target = Some(target);
    st.fight_attack.focused_kind = attack.kind;
    st.traits.simulate = false;
    st.traits.attacking = true;
    let result = borg_calculate_attack_effectiveness(st, attack.kind);
    clear_confused_aim(st);
    st.cave.grid_mut(kill.pos.x, kill.pos.y).kill = prior_kill;
    st.fight_attack.focused_target = None;
    st.traits.attacking = false;
    result > 0
}

/// borg-reborn-rs (#869): the scary-list reason of an attack entry that is
/// attacked on sight whatever its own damage (`attack_on_sight`): for pests
/// like the grey mushroom patch, which confuse again before a cure is worth
/// anything and die to one hit, so the attack comes before the cures and
/// Phase Door, confused or not.
pub(crate) const KILL_ON_SIGHT: &str = "kill-on-sight";

/// borg-reborn-rs (#969): whether the HP is below the floor under which a
/// `kill-on-sight` entry is left to the ordinary attack and caution, with a
/// cure potion in the pack or an escape at hand (`borg_fast_fragile`'s
/// `has_cure` and `escape_available`).
fn kill_on_sight_hp_low(st: &mut BorgState) -> bool {
    below_kill_on_sight_floor(param(Param::attack_kill_on_sight_min_hp_percent), tr(st, BI_CURHP), tr(st, BI_MAXHP))
        && (crate::borg_fast_fragile::has_cure(st) || crate::borg_fast_fragile::escape_available(st))
}

/// The HP half of `kill_on_sight_hp_low`: below `floor` percent of the
/// maximum (0 is no floor).
fn below_kill_on_sight_floor(floor: i32, hp: i32, max_hp: i32) -> bool {
    floor > 0 && hp * 100 < max_hp * floor
}

/// borg-reborn-rs (#1064): whether #969's floor declines the attack on sight
/// of the monster at kill index `i` in this think (a `kill-on-sight` entry,
/// the HP below the floor, a cure or an escape at hand), with
/// `attack_kill_on_sight_floor_cure`: then #867's "attacking first" hold on
/// the confusion cure does not apply either (`borg_caution`'s
/// `cure_held_for_attack`), or the Borg neither attacks nor cures.
pub(crate) fn kill_on_sight_floor_declines(st: &mut BorgState, i: usize) -> bool {
    param(Param::attack_kill_on_sight_floor_cure) != 0
        && attack_on_sight_reason(st, usize::from(st.kills.kills[i].r_idx)) == Some(KILL_ON_SIGHT)
        && kill_on_sight_hp_low(st)
}

/// borg-reborn-rs (#1064): which of the cure potions #969's floor counted
/// (`borg_fast_fragile::has_cure`: Cure Light, Serious, Critical Wounds) is
/// drunk once the floor has declined the attack, `carried` telling which are
/// in the pack. Confused, Cure Serious and then Critical first, which cure
/// the confusion (`CURE:CONFUSED`, `object.txt`), then Cure Light, which heals
/// 20 and shortens it by 20 turns (`TIMED_DEC:CONFUSED`); not confused, the
/// cheapest heal first.
fn floor_cure_choice(confused: bool, [clw, csw, ccw]: [i32; 3], mut carried: impl FnMut(i32) -> bool) -> Option<i32> {
    let order = if confused { [csw, ccw, clw] } else { [clw, csw, ccw] };
    order.into_iter().find(|&sval| carried(sval))
}

/// borg-reborn-rs (#1064, `attack_kill_on_sight_floor_cure`): drink the cure
/// #969's floor counted when it declined the attack on sight. Without it the
/// caution's own heal gates could decline it too, and #867's hold the
/// confusion cure: Druid 781905420 died two steps from the stairs, 8 CLW in
/// the pack, neither attacking nor drinking. False when no cure potion is
/// carried (the floor counted an escape): the caution decides, its cures
/// included.
fn kill_on_sight_floor_cure(st: &mut BorgState) -> bool {
    if param(Param::attack_kill_on_sight_floor_cure) == 0 {
        return false;
    }
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let plan = floor_cure_plan(
        st,
        |st, sval| borg_slot(st, TV_POTION as i32, sval) >= 0,
        // #878: hit points alone, as `borg_escape`'s own outlive test; the
        // full danger prices a spore's confusion at 408 against 12 HP.
        |st| crate::borg_danger::borg_danger_damage_only(st, y, x, 1, true, false),
        crate::borg_fast_fragile::escape_available,
    );
    let (hp, max_hp) = (tr(st, BI_CURHP), tr(st, BI_MAXHP));
    let confused = tr(st, BI_ISCONFUSED) != 0;
    floor_cure_act(
        st,
        plan,
        |st, plan| match plan {
            FloorCure::Nothing => {}
            FloorCure::Escape { danger, heal, .. } => borg_note(st, &format!(
                "# Kill-on-sight floor prefers the escape to its cure: damage {danger} against HP {hp} + {heal} (#1064)")),
            FloorCure::Drink { danger, .. } => borg_note(st, &format!(
                "# Taking the cure the kill-on-sight floor counted: HP {hp}/{max_hp}{}, damage {danger} (#1064)",
                if confused { ", confused" } else { "" })),
        },
        // Review of #1082 (third review, P3): the drink after a failed escape
        // has its own marker, so a count of the floor's drinks includes it.
        |st| borg_note(st, &format!(
            "# Kill-on-sight floor: no escape usable, taking the cure after all: HP {hp}/{max_hp}{} (#1064)",
            if confused { ", confused" } else { "" })),
        floor_escape,
        |st, sval| if sval == st.item_val.sv_potion_cure_critical {
            crate::borg_item_use::borg_quaff_crit(st, true)
        } else {
            crate::borg_item_use::borg_quaff_potion(st, sval)
        },
    )
}

/// Review of #1082 (re-review, P2): the escape `FloorCure::Escape` chose,
/// taken in the same think, so no later rule (the caution's confusion cure)
/// drinks the potion the floor rejected: the staircase underfoot, else Phase
/// Door through the one Phase Door decision (`phase_door`, its own gates:
/// the owner's adjacent-enemy rule, the anti-summon delay; no landing test,
/// as the fast-fragile site ships). False when neither can be used now.
fn floor_escape(st: &mut BorgState) -> bool {
    if let Some(key) = crate::borg_fast_fragile::stair_key_here(st) {
        borg_note(st, &format!("# Kill-on-sight floor: stairs underfoot ('{}') before the cure (#1064)", key as char));
        borg_keypress(st, keycode_t::from(key));
        return true;
    }
    if crate::borg_escape::phase_door(st, crate::borg_escape::PhaseSite::KillOnSightFloor, crate::borg_escape::PhaseCheck::None, None, |st| {
        st.borg.time.now.wrapping_sub(st.borg.time.antisummon) > param(Param::escape_phase_antisummon_delay)
            && crate::borg_escape::borg_allow_teleport(st)
            && (crate::borg_item_use::borg_read_scroll(st, st.item_val.sv_scroll_phase_door)
                || crate::borg_magic::borg_spell_fail(
                    st, crate::borg_headers::borg_magic_h::PHASE_DOOR, param(Param::escape_last_ditch_phase_fail)))
    }) {
        crate::borg_escape::reset_antisummon(st);
        borg_note(st, "# Kill-on-sight floor: Phase Door before the cure (#1064)");
        return true;
    }
    false
}

/// What `kill_on_sight_floor_cure` does with its plan (review of #1082,
/// re-review P2: the decision is carried to the action): `Escape` takes the
/// escape, and when it cannot be used drinks after all (the potion is then
/// the best move left); `Drink` quaffs. The think is spent when an action
/// was made (true). `note` names the plan first; `fallback_note` marks the
/// drink after an escape that could not be used. The decision tests call it
/// with the engine's parts given.
fn floor_cure_act(
    st: &mut BorgState,
    plan: FloorCure,
    note: impl FnOnce(&mut BorgState, &FloorCure),
    fallback_note: impl FnOnce(&mut BorgState),
    escape: impl FnOnce(&mut BorgState) -> bool,
    quaff: impl FnOnce(&mut BorgState, i32) -> bool,
) -> bool {
    note(st, &plan);
    match plan {
        FloorCure::Nothing => false,
        FloorCure::Escape { sval, .. } => escape(st) || {
            fallback_note(st);
            quaff(st, sval)
        },
        FloorCure::Drink { sval, .. } => quaff(st, sval),
    }
}

/// What a cure potion heals (`HEAL_HP`, `object.txt`: 20, 40 and 60, no
/// percentage), capped at the HP missing (`effect-handler-attack.c`
/// `effect_handler_HEAL_HP`: `player->chp` stops at `player->mhp`).
fn floor_cure_heal(sval: i32, [clw, csw, ccw]: [i32; 3], hp: i32, max_hp: i32) -> i32 {
    let heal = if sval == clw { 20 } else if sval == csw { 40 } else if sval == ccw { 60 } else { 0 };
    heal.min((max_hp - hp).max(0))
}

/// Review of #1082 (P2): the potion is drunk when the Borg outlives the next
/// turn with it (the damage-only danger at its grid below the HP it will have), or when
/// no escape is at hand (the potion is then the best move left). Otherwise
/// the escape outranks it: `floor_escape` takes the stairs underfoot or reads
/// Phase Door, which a heal that the next turn undoes would only delay.
fn floor_cure_wanted(danger: i32, hp: i32, heal: i32, escape: impl FnOnce() -> bool) -> bool {
    danger < hp + heal || !escape()
}

/// What `kill_on_sight_floor_cure` does once the floor has declined.
#[derive(Debug, PartialEq, Eq)]
enum FloorCure {
    /// No cure potion carried (the floor counted an escape): the caution
    /// decides, its own cures included.
    Nothing,
    /// The potion does not outlive the next turn and an escape is at hand:
    /// `floor_escape` takes it, else the potion `sval` is drunk after all.
    Escape { sval: i32, danger: i32, heal: i32 },
    /// Quaff `sval`; the think is spent.
    Drink { sval: i32, danger: i32 },
}

/// `kill_on_sight_floor_cure`'s decision, with the engine's parts given (the
/// pack, the danger at the Borg's grid, the escape test: `borg_slot` reads
/// the engine's tables). The danger is asked only when a potion is carried,
/// the escape only when the potion does not outlive the next turn. The
/// decision tests (review of #1082, P3) call it directly.
fn floor_cure_plan(
    st: &mut BorgState,
    mut carried: impl FnMut(&mut BorgState, i32) -> bool,
    danger: impl FnOnce(&mut BorgState) -> i32,
    escape: impl FnOnce(&mut BorgState) -> bool,
) -> FloorCure {
    let confused = tr(st, BI_ISCONFUSED) != 0;
    let svals = [st.item_val.sv_potion_cure_light, st.item_val.sv_potion_cure_serious, st.item_val.sv_potion_cure_critical];
    let Some(sval) = floor_cure_choice(confused, svals, |sval| carried(st, sval)) else {
        return FloorCure::Nothing;
    };
    let heal = floor_cure_heal(sval, svals, tr(st, BI_CURHP), tr(st, BI_MAXHP));
    let danger = danger(st);
    if floor_cure_wanted(danger, tr(st, BI_CURHP), heal, || escape(st)) {
        FloorCure::Drink { sval, danger }
    } else {
        FloorCure::Escape { sval, danger, heal }
    }
}

/// #760: the category target comes before flight and remains the target on
/// subsequent thinks. A breeder colony already lost is not an attack entry.
pub(crate) fn attack_on_sight(st: &mut BorgState) -> bool {
    if param(Param::attack_on_sight) == 0 { return false; }
    for i in 1..st.kills.nxt.max(0) as usize {
        let kill = st.kills.kills[i];
        if kill.r_idx == 0 { continue; }
        // borg-reborn-rs (#842, owner): a fast-fragile monster is attacked
        // with the best attack at hand, or Phase Door / the stairs underfoot
        // when the next turn kills; no `target damage < HP` gate.
        if crate::borg_fast_fragile::policy() {
            if let Some((source, facts, attack)) = crate::borg_fast_fragile::classify(st, i) {
                if !crate::borg_caution::category_attack_safe_with(st, i, false) { continue; }
                // #914: a group in contact skips this attack, on a marked target in
                // contact or not; later entries, then caution, decide (no hand-off).
                if let Some(n) = crate::borg_fast_fragile::crowded(st) {
                    borg_note(st, &format!("# Fast-fragile: {n} marked attackers in contact, caution decides (#914)"));
                    continue;
                }
                let note = crate::borg_fast_fragile::describe(st, i, source, &facts);
                borg_note(st, &note);
                if crate::borg_fast_fragile::escape_instead(st, i, &facts) { return true; }
                // The exceptions to the heal ban: cornered (#842) or at the
                // heal floor (#926), a cure comes first; the note names which.
                if let Some(why) = crate::borg_fast_fragile::yield_to_cure(st, i, &facts) {
                    borg_note(st, why);
                    continue;
                }
                borg_note(st, &format!("# Attack on sight: {} (fast-fragile), {} HP, attack {} (#842)",
                    String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx))), kill.power, attack.kind));
                if perform_focused_attack(st, i, attack) { return true; }
                continue;
            }
        }
        let Some(reason) = attack_on_sight_reason(st, usize::from(kill.r_idx)) else { continue };
        let Some(attack) = focused_attack(st, i) else { continue };
        // borg-reborn-rs (#869, owner: "Always attack, confused or not.
        // Unless there is something more dangerous"): a `kill-on-sight`
        // entry drops the target's own damage test, as #842 does for
        // fast-fragile monsters; the other monsters' danger still decides.
        // No heal ban and no fast-fragile escape: the target does not chase.
        let safe = if reason == KILL_ON_SIGHT {
            crate::borg_caution::category_attack_safe_with(st, i, false)
        } else {
            crate::borg_caution::category_attack_safe(st, i)
        };
        if !safe { continue; }
        // borg-reborn-rs (#969): below `attack_kill_on_sight_min_hp_percent`
        // of max HP a `kill-on-sight` entry is not attacked on sight: the
        // ordinary attack and caution (the heals, Phase Door, flight)
        // decide, as for any monster, when a cure potion is in the pack or
        // an escape is at hand (stairs underfoot, a Phase Door that can be
        // read). Confused next to a grey mushroom patch a swing reaches it
        // 11/32 of the time (player_confuse_dir) and every spore costs 1d4,
        // so the attack first had no end. With neither the attack goes on:
        // confused, no scroll can be read and a step goes astray 75% of
        // the time, so the swing is the best chance left.
        if reason == KILL_ON_SIGHT && kill_on_sight_hp_low(st) {
            borg_note(st, &format!("# Not attacking on sight: {} (kill-on-sight), HP {}/{} below {}%, a cure or an escape at hand (#969)",
                String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx))),
                tr(st, BI_CURHP), tr(st, BI_MAXHP), param(Param::attack_kill_on_sight_min_hp_percent)));
            // borg-reborn-rs (#1064): the cure the floor counted is taken.
            if kill_on_sight_floor_cure(st) { return true; }
            continue;
        }
        borg_note(st, &format!("# Attack on sight: {} ({reason}), {} HP, attack {} (#760)",
            String::from_utf8_lossy(engine::race_name(usize::from(kill.r_idx))), kill.power, attack.kind));
        if perform_focused_attack(st, i, attack) { return true; }
    }
    false
}

/// borg-reborn-rs (#584): the true damage (no kill bonus, no danger bonus,
/// no unique targeting bonus) of the attack `borg_attack` would choose now,
/// of any kind (melee, missiles, throws, spells, wands, rods, activations),
/// without making it; `None` when it would choose none. The simulation draws
/// the random numbers `borg_attack` would.
pub(crate) fn borg_attack_estimate(st: &mut BorgState) -> Option<i32> {
    st.traits.estimate = None;
    st.fight_attack.est_cost = None;
    st.fight_attack.est_fallback = 0;
    st.fight_attack.est_kills = 0.0;
    st.traits.estimate_only = true;
    let _ = borg_attack(st, false);
    st.traits.estimate_only = false;
    st.traits.estimate.take()
}

/// borg-reborn-rs (#469): whether to drink Speed before a fight the Borg has
/// committed to (owner, 2026-11-17: when the danger is very high and it fights
/// rather than retreats). `borg_defend_aux_speed` drinks only when being fast
/// would make the fight safe; this covers the fights speed cannot make safe.
/// Off (threshold 0) it never applies.
fn fight_speed_wanted(threshold_tenths: i32, fast: bool, depth: i32, has_speed: bool, danger: i32, avoidance: i32) -> bool {
    threshold_tenths > 0 && !fast && depth > 0 && has_speed
        && i64::from(danger) > i64::from(avoidance) * i64::from(threshold_tenths) / 10
}

/// borg-reborn-rs (#469): drink a carried Potion of Speed (+10 speed for
/// 20+2d10 turns, object.txt TIMED_INC:FAST) before a very dangerous fight,
/// the danger summed over the monsters that can move.
fn borg_fight_speed(st: &mut BorgState) -> bool {
    let threshold = param(Param::attack_speed_fight_danger_tenths);
    if threshold == 0 {
        return false;
    }
    let has_speed = borg_slot(st, TV_POTION as i32, st.item_val.sv_potion_speed) >= 0;
    if st.borg.temp.fast || tr(st, BI_CDEPTH) == 0 || !has_speed {
        return false;
    }
    // Only monsters that can move count (owner, 2026-11-17): a never-moving
    // one (mushroom patch, floating eye; monster.txt NEVER_MOVE) can simply
    // be left, and its spores or gaze inflated the danger past any threshold.
    let (y, x) = (st.borg.c.y, st.borg.c.x);
    let horizon = param(Param::defend_danger_horizon);
    let mut danger = 0i32;
    for i in 1..i32::from(st.kills.nxt) {
        let r_idx = st.kills.kills[i as usize].r_idx;
        if r_idx == 0 || engine::race_flag_has(usize::from(r_idx), RF_NEVER_MOVE) {
            continue;
        }
        danger = danger.saturating_add(borg_danger_one_kill(st, y, x, horizon, i, true, true));
    }
    if !fight_speed_wanted(threshold, false, tr(st, BI_CDEPTH), true, danger, avoid(st)) {
        return false;
    }
    if borg_quaff_potion(st, st.item_val.sv_potion_speed) {
        borg_note(st, &format!("# Speed for a dangerous fight (danger {} of avoidance {})", danger, avoid(st)));
        return true;
    }
    false
}

#[cfg(test)]
mod reserve_faint_tests {
    use super::*;

    /// #878: `randint1(5 * oops + 1)`: Magic Missile (1 mana) at 0 SP faints
    /// 1 to 6 world ticks, 3.5 on average.
    #[test]
    fn the_faint_lasts_its_mean() {
        assert_eq!(reserve_faint_turns(1), 3.5);
        assert_eq!(reserve_faint_turns(2), 6.0);
        assert_eq!(reserve_faint_turns(0), 1.0);
    }

    /// #878 review F2: a tick of paralysis is more Borg turns for a fast
    /// Borg (extract_energy 10 at normal speed, 20 at +10, 30 at +20).
    #[test]
    fn a_fast_borg_takes_more_blows_while_fainted() {
        assert_eq!(faint_borg_turns(3.5, 10.0), 3.5);
        assert_eq!(faint_borg_turns(3.5, 20.0), 7.0);
        let adjacent = |e| faint_contact_damage(&[(0.9, 1, 0.5, 1.0)], faint_borg_turns(3.5, e));
        assert!((adjacent(30.0) - 3.0 * adjacent(10.0)).abs() < 1e-9);
    }

    #[test]
    fn walkers_join_the_faint_late_and_the_target_only_if_the_cast_fails() {
        // Two worms (0.9 a turn, half speed): adjacent, and three grids away.
        let adjacent = faint_contact_damage(&[(0.9, 1, 0.5, 1.0)], 3.5);
        assert!((adjacent - 3.15).abs() < 1e-9);
        let far = faint_contact_damage(&[(0.9, 3, 0.5, 1.0)], 3.5);
        assert_eq!(far, 0.0, "two steps at half speed take 4 turns");
        let near = faint_contact_damage(&[(0.9, 2, 0.5, 1.0)], 3.5);
        assert!((near - 0.9 * 1.5).abs() < 1e-9);
        // The target at a 5% fail rate.
        let target = faint_contact_damage(&[(0.9, 1, 0.5, 0.05)], 3.5);
        assert!((target - 0.1575).abs() < 1e-9);
        // Defensive only: `reserve_faint_refused` clamps the energy, so a
        // monster's moves are never 0 there; it skips immobile ones instead.
        assert_eq!(faint_contact_damage(&[(5.0, 4, 0.0, 1.0)], 3.5), 0.0, "no moves, never arrives");
        // The skip that runs: a mold strikes only when adjacent.
        assert!(faint_reaches(1, true), "an adjacent mold");
        assert!(!faint_reaches(2, true), "a mold two grids away");
        assert!(faint_reaches(4, false), "a worm walks in");
    }
}

#[cfg(test)]
mod floor_cure_tests {
    use super::floor_cure_choice;

    const CLW: i32 = 1;
    const CSW: i32 = 2;
    const CCW: i32 = 3;

    /// #1064: Druid 781905420, confused next to a grey mushroom patch at HP
    /// 5/12 with 8 CLW and nothing else: the floor declined the attack, and
    /// the potion it counted is the action.
    #[test]
    fn confused_with_only_cure_light_drinks_it() {
        assert_eq!(floor_cure_choice(true, [CLW, CSW, CCW], |s| s == CLW), Some(CLW));
    }

    #[test]
    fn confused_prefers_the_potions_that_cure_confusion() {
        assert_eq!(floor_cure_choice(true, [CLW, CSW, CCW], |_| true), Some(CSW));
        assert_eq!(floor_cure_choice(true, [CLW, CSW, CCW], |s| s != CSW), Some(CCW));
    }

    #[test]
    fn not_confused_drinks_the_cheapest_heal() {
        assert_eq!(floor_cure_choice(false, [CLW, CSW, CCW], |_| true), Some(CLW));
        assert_eq!(floor_cure_choice(false, [CLW, CSW, CCW], |s| s == CCW), Some(CCW));
    }

    /// The floor counted an escape only: no potion, nothing drunk.
    #[test]
    fn no_cure_carried_drinks_nothing() {
        assert_eq!(floor_cure_choice(true, [CLW, CSW, CCW], |_| false), None);
        assert_eq!(floor_cure_choice(false, [CLW, CSW, CCW], |_| false), None);
    }

    /// The flag is off when compiled, shipped on in `harness/pborg.yaml`.
    #[test]
    fn the_floor_cure_is_compiled_off_and_shipped_on() {
        let spec = crate::params::SPECS.iter().find(|s| s.name == "attack_kill_on_sight_floor_cure").unwrap();
        assert_eq!((spec.default, spec.min, spec.max), (0, 0, 1));
        let shipped = include_str!("../../harness/pborg.yaml");
        assert!(shipped.lines().any(|l| l.trim() == "borg_param_attack_kill_on_sight_floor_cure: 1"));
    }

    /// #1064: the floor's decline drinks before it moves on (the wiring;
    /// the decision itself is `floor_cure_act`'s tests).
    #[test]
    fn the_decline_drinks_before_it_moves_on() {
        let src = include_str!("borg_fight_attack.rs");
        let decline = &src[src.find(concat!("a cure or an escape at hand (", "#969)")).unwrap()..];
        let decline = &decline[..decline.find("continue;").unwrap()];
        assert!(decline.contains(concat!("if kill_on_sight_floor_", "cure(st) { return true; }")), "the decline drinks first");
    }

    use super::{below_kill_on_sight_floor, floor_cure_act, floor_cure_heal, floor_cure_plan, floor_cure_wanted, FloorCure};
    use crate::borg_headers::borg_trait_h::{BI_CURHP, BI_ISCONFUSED, BI_MAX, BI_MAXHP};
    use crate::state::BorgState;

    /// Druid 781905420's think at turn 13,311: CL1, HP 5/12, confused, next
    /// to a grey mushroom patch (a spore: at most 4, `SPORE:CONFUSE:1d4`),
    /// 8 CLW and nothing else.
    fn druid(confused: bool) -> BorgState {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_CURHP as usize] = 5;
        st.borg.trait_[BI_MAXHP as usize] = 12;
        st.borg.trait_[BI_ISCONFUSED as usize] = i32::from(confused);
        st.item_val.sv_potion_cure_light = CLW;
        st.item_val.sv_potion_cure_serious = CSW;
        st.item_val.sv_potion_cure_critical = CCW;
        st
    }

    /// Review of #1082 (P3): the 5/12 case drinks the CLW; the escape is not
    /// asked, since the potion outlives the turn.
    #[test]
    fn the_druid_drinks_its_clw() {
        let mut st = druid(true);
        let plan = floor_cure_plan(&mut st, |_, s| s == CLW, |_| 4, |_| panic!("no escape test"));
        assert_eq!(plan, FloorCure::Drink { sval: CLW, danger: 4 });
    }

    /// Review of #1082 (P2): a second foe that kills through the heal, with
    /// the stairs underfoot: no potion, the caution takes the escape. With no
    /// escape the potion is the best move left.
    #[test]
    fn an_escape_outranks_a_heal_the_next_turn_undoes() {
        // 30/80 with a CLW: 50 after it, against 55.
        let mut st = druid(false);
        st.borg.trait_[BI_CURHP as usize] = 30;
        st.borg.trait_[BI_MAXHP as usize] = 80;
        assert_eq!(floor_cure_plan(&mut st, |_, s| s == CLW, |_| 55, |_| true), FloorCure::Escape { sval: CLW, danger: 55, heal: 20 });
        assert_eq!(floor_cure_plan(&mut st, |_, s| s == CLW, |_| 55, |_| false), FloorCure::Drink { sval: CLW, danger: 55 });
        // A CSW (40) outlives it: drink, the escape not asked.
        let plan = floor_cure_plan(&mut st, |_, s| s == CSW, |_| 55, |_| panic!("no escape test"));
        assert_eq!(plan, FloorCure::Drink { sval: CSW, danger: 55 });
    }

    /// No cure potion (the floor counted an escape): nothing is asked or
    /// drunk, and the caution decides.
    #[test]
    fn no_potion_asks_nothing() {
        let mut st = druid(true);
        assert_eq!(floor_cure_plan(&mut st, |_, _| false, |_| panic!("no danger"), |_| panic!("no escape")), FloorCure::Nothing);
    }

    /// Re-review of #1082 (P2): the escape the plan chose is the think's
    /// action: the stair key is queued and the rejected potion is not drunk.
    #[test]
    fn the_escape_is_taken_ahead_of_the_rejected_potion() {
        // The reviewer's case: confused on an up staircase at 45/120, only a
        // CSW (40), 88 damage incoming. (`borg_keypress` links the engine's
        // message log, so the escape records its key here.)
        let mut st = druid(true);
        let mut keys = Vec::new();
        let plan = FloorCure::Escape { sval: CSW, danger: 88, heal: 40 };
        let spent = floor_cure_act(&mut st, plan, |_, _| {}, |_| panic!("no fallback"), |_| { keys.push(b'<'); true }, |_, _| panic!("the rejected potion"));
        assert!(spent, "the think is spent");
        assert_eq!(keys, vec![b'<']);
    }

    /// No escape can be used after all (the Phase Door gates refuse it):
    /// the potion is drunk, the best move left.
    #[test]
    fn an_escape_that_cannot_be_used_drinks_after_all() {
        let mut st = druid(true);
        let (mut quaffed, mut marked) = (None, false);
        let plan = FloorCure::Escape { sval: CSW, danger: 88, heal: 40 };
        assert!(floor_cure_act(&mut st, plan, |_, _| {}, |_| marked = true, |_| false, |_, s| { quaffed = Some(s); true }));
        assert_eq!(quaffed, Some(CSW));
        assert!(marked, "the fallback drink has its own marker (third review, P3)");
    }

    /// `Drink` quaffs and spends the think; `Nothing` asks nothing.
    #[test]
    fn drink_quaffs_and_nothing_acts_on_nothing() {
        let mut st = druid(true);
        let mut quaffed = None;
        assert!(floor_cure_act(&mut st, FloorCure::Drink { sval: CLW, danger: 8 }, |_, _| {}, |_| panic!("no fallback"), |_| panic!("no escape"), |_, s| { quaffed = Some(s); true }));
        assert_eq!(quaffed, Some(CLW));
        assert!(!floor_cure_act(&mut st, FloorCure::Nothing, |_, _| {}, |_| panic!("no fallback"), |_| panic!("no escape"), |_, _| panic!("no quaff")));
    }

    /// The stair key `floor_escape` presses, as `escape_available` counts the
    /// staircase underfoot.
    #[test]
    fn the_stair_key_follows_the_staircase() {
        use crate::borg_fast_fragile::stair_key;
        use crate::headers::list_terrain_h::{FEAT_FLOOR, FEAT_LESS, FEAT_MORE};
        assert_eq!(stair_key(FEAT_LESS, false), Some(b'<'));
        assert_eq!(stair_key(FEAT_LESS, true), None, "birth_force_descend: no way up");
        assert_eq!(stair_key(FEAT_MORE, true), Some(b'>'));
        assert_eq!(stair_key(FEAT_FLOOR, false), None);
    }

    #[test]
    fn the_heal_and_the_survival_test_follow_the_engine() {
        let svals = [CLW, CSW, CCW];
        assert_eq!(floor_cure_heal(CLW, svals, 5, 12), 7, "capped at the HP missing");
        assert_eq!(floor_cure_heal(CLW, svals, 30, 80), 20);
        assert_eq!(floor_cure_heal(CSW, svals, 30, 80), 40);
        assert_eq!(floor_cure_heal(CCW, svals, 10, 200), 60);
        assert!(floor_cure_wanted(11, 5, 7, || panic!()), "12 HP after it, against 11");
        assert!(!floor_cure_wanted(12, 5, 7, || true));
        assert!(floor_cure_wanted(12, 5, 7, || false));
        // #969's floor at the shipped 50: 5/12 is below it, 6/12 is not.
        assert!(below_kill_on_sight_floor(50, 5, 12) && !below_kill_on_sight_floor(50, 6, 12));
        assert!(!below_kill_on_sight_floor(0, 1, 12), "0 is no floor");
    }
}

#[cfg(test)]
mod attack_kind_tests {
    use super::*;

    /// #869: the scary list's attack kinds, by attack family.
    #[test]
    fn attack_kinds_follow_the_families() {
        let only = |name: &str| 1u8 << crate::scary_list::ATTACK_KINDS.iter().position(|k| *k == name).unwrap();
        let all = crate::scary_list::ALL_ATTACK_KINDS;
        for kind in [BF_THRUST, BF_LAUNCH, BF_OBJECT, BF_SPELL_MAGIC_MISSILE, BF_WAND_HOLD_MONSTER, BF_ACT_ARROW] {
            assert!(kind_allowed(all, kind as i32), "{kind}: all kinds");
        }
        assert!(kind_allowed(only("melee"), BF_THRUST as i32));
        assert!(!kind_allowed(only("melee"), BF_OBJECT as i32));
        assert!(kind_allowed(only("missiles"), BF_LAUNCH as i32));
        assert!(kind_allowed(only("throws"), BF_OBJECT as i32));
        assert!(kind_allowed(only("spells"), BF_SPELL_MAGIC_MISSILE as i32));
        assert!(kind_allowed(only("devices"), BF_WAND_HOLD_MONSTER as i32));
        assert!(!kind_allowed(only("devices"), BF_SPELL_MAGIC_MISSILE as i32));
        assert!(kind_allowed(only("activations"), BF_ACT_ARROW as i32));
        assert!(!kind_allowed(all, BF_REST as i32), "a wait is never a focused attack");
    }
}

#[cfg(test)]
mod town_kill_tests {
    use super::*;

    /// #619: thieves always, hurting blows from the minimum damage on.
    #[test]
    fn dangerous_town_blows() {
        assert!(dangerous_blow(MONBLOW_EAT_GOLD, 0, 6), "the urchin");
        assert!(dangerous_blow(MONBLOW_EAT_ITEM, 0, 6), "the rogue's touch");
        assert!(dangerous_blow(MONBLOW_HURT, 12, 6), "the veteran, 2d6");
        assert!(dangerous_blow(MONBLOW_HURT, 6, 6), "the rogue, 1d6");
        assert!(!dangerous_blow(MONBLOW_HURT, 3, 6), "the merchant, 1d3");
        assert!(!dangerous_blow(MONBLOW_HURT, 1, 6), "a cat or a dog");
    }
}

#[cfg(test)]
mod fight_speed_tests {
    use super::fight_speed_wanted;

    #[test]
    fn speed_before_a_very_dangerous_fight() {
        // threshold 20 tenths: danger above twice the avoidance
        assert!(fight_speed_wanted(20, false, 5, true, 201, 100));
        assert!(!fight_speed_wanted(20, false, 5, true, 200, 100));
        assert!(!fight_speed_wanted(0, false, 5, true, 10_000, 100), "0 is off");
        assert!(!fight_speed_wanted(20, true, 5, true, 1000, 100), "already fast");
        assert!(!fight_speed_wanted(20, false, 0, true, 1000, 100), "town");
        assert!(!fight_speed_wanted(20, false, 5, false, 1000, 100), "no potion");
    }
}

#[cfg(test)]
mod town_oil_tests {
    use super::oil_reserved;
    #[test]
    fn throwing_oil_does_not_wait_for_unneeded_lantern_fuel_with_a_torch() {
        assert!(!oil_reserved(true, 0, 0, false));
        assert!(oil_reserved(true, 1, 1, false));
        assert!(!oil_reserved(true, 1, 2, false));
        assert!(!oil_reserved(true, 1, 1, true));
        assert!(oil_reserved(true, 3, 3, false), "a longer forecast can need several fuel bottles");
        assert!(!oil_reserved(true, 0, 1, false), "a filled lantern can cover the forecast already");
        assert!(oil_reserved(false, 0, 0, false), "control retains its original behaviour");
    }
}

#[cfg(test)]
mod point_blank_tests {
    use super::*;

    /// #485: with no class override every class uses the default divisor
    /// (compiled 5); an unknown class index too.
    #[test]
    fn every_class_falls_back_to_the_default_divisor() {
        for class in 0..9 {
            assert_eq!(point_blank_divisor(class), 5, "class {class}");
        }
        assert_eq!(point_blank_divisor(42), 5);
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;

    #[test]
    fn a_launched_ball_counts_and_wakes_a_penalized_sleeper_once() {
        let mut st = BorgState::default();
        st.kills.kills = vec![crate::borg_flow_kill::BorgKill::BLANK; 3];
        st.kills.kills[1].r_idx = 1;
        st.kills.kills[2].r_idx = 2;
        st.kills.kills[1].pos = loc { x: 4, y: 5 };
        st.kills.kills[2].pos = loc { x: 6, y: 7 };
        let mut unseen = Vec::new();
        collect_believed_target(&mut unseen, 1, &st.kills.kills[1], Some(100), 3);
        collect_believed_target(&mut unseen, 2, &st.kills.kills[2], Some(50), 5);
        collect_believed_target(&mut unseen, 1, &st.kills.kills[1], Some(100), 3);
        let score = expected_score(-999, Some(100)) + expected_score(2500, Some(50));
        assert_eq!(score, 251);

        // Stub out command I/O. Feed the same ledger the ball scorer builds
        // through queuing and the next normal command input.
        st.fight_attack.pending_unseen.stage(unseen);
        st.fight_attack.pending_unseen.command_queued(true);
        st.borg.time.now = 10;
        st.kills.kills[1].pos = loc { x: 8, y: 9 };
        let mut trace = Vec::new();
        let mut log = |_: &mut BorgState, target: BelievedTarget| {
            trace.push((target.index, target.race, target.pos.x, target.pos.y, target.weight, target.age));
        };
        crate::borg_unseen_belief::finish_attack(&mut st, &mut log);
        crate::borg_unseen_belief::finish_attack(&mut st, log);
        assert_eq!(trace, [(1, 1, 4, 5, 100, 3), (2, 2, 6, 7, 50, 5)]);
        assert_eq!(st.fight_attack.unseen_shots[&1], (0, 1));
        assert_eq!(st.fight_attack.unseen_shots[&2], (0, 1));
        assert!(st.kills.kills[1].awake);
        assert!(st.kills.kills[2].awake);
    }

    /// Review of #657: the winner is never its own fallback.
    #[test]
    fn the_winner_is_not_its_own_fallback() {
        // A wand (10) won; melee 1 and a sling 4 need no mana either.
        let no_mana = [(30, 10), (1, 1), (5, 4)];
        assert_eq!(fallback_without(&no_mana, 30), 4);
        assert_eq!(fallback_without(&no_mana, 99), 10, "a spell won: the wand is a fallback");
        assert_eq!(fallback_without(&[(1, 3)], 1), 0, "nothing else");
    }
}

#[cfg(test)]
mod throw_tests {
    use super::{low_risk_potion_throw, native_throw_range, throw_hit_percent, thrown_damage};

    #[test]
    fn spending_a_potion_requires_enough_current_hp_to_survive_a_miss() {
        assert!(low_risk_potion_throw(72, 7));
        assert!(!low_risk_potion_throw(72, 8));
        assert!(!low_risk_potion_throw(7, 7));
        assert!(!low_risk_potion_throw(0, 0));
    }

    /// #568: the engine's throw damage (`player-attack.c:554-582`, `:1296`).
    #[test]
    fn thrown_damage_is_the_engines() {
        assert_eq!(thrown_damage(1, 4, 0, 20, true, true), 22, "a flask of oil: 2.5 x 3 x 3");
        assert_eq!(thrown_damage(1, 4, 0, 5, true, false), 5, "an Iron Shot: 2.5 x 2");
        assert_eq!(thrown_damage(1, 4, 0, 12, true, false), 7, "a Dagger: 2.5 x 3");
        assert_eq!(thrown_damage(1, 9, 0, 9, false, false), 5, "an Arrow, not THROWING: 5");
        assert_eq!(thrown_damage(1, 4, 2, 5, true, false), 9, "to-dam before the multiplier");
    }

    /// #870: oil (22 damage) against the adjacent crow (AC 14), using the
    /// named Half-Troll Necromancer's throwing skill 10 and no hit bonus.
    #[test]
    fn adjacent_crow_oil_rating_has_its_own_divisor() {
        use super::{rated_throw_damage, throw_point_blank_divisor};
        let oil = thrown_damage(1, 4, 0, 20, true, true);
        let chance = throw_hit_percent(10, 0, true, 1, 14);
        assert_eq!(chance, 12);
        assert_eq!(rated_throw_damage(oil, 1, false, 5, chance), 0);
        assert_eq!(rated_throw_damage(oil, 1, false, 1, chance), 2);
        for arrow in [1, 2, 3, 5, 1000] {
            assert_eq!(throw_point_blank_divisor(0, arrow), arrow);
            assert_eq!(throw_point_blank_divisor(1, arrow), 1);
            assert_eq!(throw_point_blank_divisor(5, arrow), 5);
            assert_eq!(rated_throw_damage(oil, 2, false, arrow, chance), 2, "not adjacent");
            assert_eq!(rated_throw_damage(oil, 1, true, arrow, chance), 2, "unique");
        }
        assert_eq!(crate::params::value(crate::params::Param::attack_throw_point_blank_divisor), 0);
    }

    /// #568: `hit_chance` (`player-attack.c:196-219`): 12% always hits, 5%
    /// always misses, the rest scales with (to-hit - 2/3 AC) / to-hit.
    #[test]
    fn native_throw_range_uses_single_item_weight_and_caps_at_ten() {
        assert_eq!(native_throw_range(9, 12), 10);
        assert_eq!(native_throw_range(9, 50), 5);
        assert_eq!(native_throw_range(9, 55), 5);
        assert_eq!(native_throw_range(9, 0), 10);
    }

    #[test]
    fn throw_hit_chance_is_the_engines() {
        assert_eq!(throw_hit_percent(30, 0, true, 0, 0), 95, "no armour: all but the 5% miss");
        assert_eq!(throw_hit_percent(30, 0, true, 0, 45), 12, "armour beyond reach: the 12% hit");
        assert_eq!(throw_hit_percent(30, 0, true, 0, 30), 39, "(30 - 20) / 30 of 83, plus 12");
        assert_eq!(throw_hit_percent(20, 0, false, 0, 30), 39, "not THROWING: 3/2 of the skill");
        assert_eq!(throw_hit_percent(30, 0, true, 3, 30), 33, "the distance lowers it");
        assert_eq!(throw_hit_percent(20, 2, true, 0, 30), 31, "bonus x BTH_PLUS_ADJ");
    }
}

#[cfg(test)]
mod confused_attack_tests {
    use super::*;

    /// #867: `player_confuse_dir` (player-util.c:1353-1377): a direction key
    /// is redrawn when `randint0(100) < 75`, then `ddd[randint0(8)]`; counted
    /// over every pair of draws, the key survives 11/32 of the time.
    #[test]
    fn the_aim_chance_is_player_confuse_dirs() {
        let wanted = 3;
        let mut kept = 0;
        for roll in 0..100 {
            for draw in 0..8 {
                let dir = if roll < 75 { draw } else { wanted };
                kept += i32::from(dir == wanted);
            }
        }
        let (num, den) = CONFUSED_AIM;
        assert_eq!(kept * den, num * 800, "kept {kept} of 800");
    }

    #[test]
    fn only_a_confused_borg_that_sees_aims_by_direction() {
        assert!(by_direction(true, false, true, false));
        assert!(!by_direction(false, false, true, false), "switch 0: C");
        assert!(!by_direction(true, false, false, false), "not confused: the usual aim");
        assert!(!by_direction(true, true, true, false), "blind: nothing aimed");
        assert!(!by_direction(true, false, true, true), "hallucinating: nothing aimed");
    }

    /// The shipped 0 keeps C: confused, no item attack at all.
    #[test]
    fn shipped_off_keeps_every_item_attack_out_while_confused() {
        let mut st = BorgState::default();
        st.borg.trait_ = vec![0; BI_MAX as usize];
        st.borg.trait_[BI_ISCONFUSED as usize] = 1;
        assert!(!confused_by_direction(&st));
        assert!(cant_use_item_attack(&st));
        mark_confused_aim(&mut st);
        assert!(!st.fight_attack.confused_aim);
        st.borg.trait_[BI_ISCONFUSED as usize] = 0;
        assert!(!cant_use_item_attack(&st));
    }

    /// A target is `5`, which confusion always redraws; a direction is its
    /// digit, with no target to track afterwards.
    #[test]
    fn a_confused_aim_is_sent_as_a_direction_digit() {
        assert_eq!(aim_answer(0, -2), (key(b'5'), -2));
        assert_eq!(aim_answer(0, -1), (key(b'5'), -1));
        for dir in [1, 2, 3, 4, 6, 7, 8, 9] {
            assert_eq!(aim_answer(dir, -2), (i2d(dir), 0));
            assert_ne!(aim_answer(dir, -2).0, key(b'5'));
        }
    }

    /// Every aimed attack loses the same share, so the chooser still ranks
    /// them on damage: a caster's weak blow loses to oil, a fighter's blow
    /// beats it; a staff (not marked) keeps its full value.
    #[test]
    fn the_weight_keeps_the_ranking_on_damage() {
        let mut st = BorgState::default();
        let mut weigh = |raw: i32, dmg: i32, aimed: bool| {
            st.fight_attack.confused_aim = aimed;
            st.fight_attack.dmg_best = dmg;
            st.fight_attack.kill_best = 1.0;
            let v = confused_weight(&mut st, raw, true);
            (v, st.fight_attack.dmg_best, st.fight_attack.kill_best)
        };
        assert_eq!(weigh(64, 64, true), (22, 22, 11.0 / 32.0));
        assert_eq!(weigh(64, 64, false), (64, 64, 1.0), "not aimed: unchanged");
        let (mage_blow, _, _) = weigh(3, 3, true);
        let (oil, _, _) = weigh(22, 22, true);
        let (warrior_blow, _, _) = weigh(40, 40, true);
        assert!(oil > mage_blow);
        assert!(warrior_blow > oil);
        let (staff, _, _) = weigh(20, 20, false);
        assert!(staff > oil, "a staff needs no aim");
    }

    /// Review of #868: `borg_attack` weights a blow's value, not the damage
    /// the fight-or-flee tests read, which stays C's.
    #[test]
    fn a_blows_damage_stays_cs_for_fight_or_flee() {
        let mut st = BorgState::default();
        st.fight_attack.confused_aim = true;
        st.fight_attack.dmg_best = 64;
        st.fight_attack.kill_best = 1.0;
        assert_eq!(confused_weight(&mut st, 64, false), 22, "the value is weighted");
        assert_eq!((st.fight_attack.dmg_best, st.fight_attack.kill_best), (64, 1.0), "the damage is not");
    }
}

#[cfg(test)]
mod tests_890_adjacent {
    use super::counts_as_adjacent;

    /// Review of #890, F1/R4: a believed record is never adjacent; a seen
    /// one keeps C's test.
    #[test]
    fn a_believed_record_is_never_the_adjacent_monster() {
        // Seen: next to me, or fast and within attack_fast_monster_distance (2).
        assert!(counts_as_adjacent(false, false, 1, 2));
        assert!(counts_as_adjacent(false, true, 2, 2));
        assert!(!counts_as_adjacent(false, false, 2, 2));
        assert!(!counts_as_adjacent(false, true, 3, 2));
        // Believed: never, at any distance or speed.
        assert!(!counts_as_adjacent(true, true, 2, 2));
        assert!(!counts_as_adjacent(true, false, 1, 2));
        assert!(!counts_as_adjacent(true, true, 1, 2));
    }
}
